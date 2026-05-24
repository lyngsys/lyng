//! asm-visible state record + Rust-only context per design §5.

#![allow(
    clippy::pub_underscore_fields,
    reason = "LlIntState is a repr(C) asm ABI record with explicit public padding fields for stable offsets"
)]

use std::sync::Arc;

use lyng_env::{Agent, ExecutableId};
use lyng_host::HostHooks;
use lyng_objects::{
    FunctionEntryIdentity, FunctionObjectData, FunctionThisMode, NativeFunctionRegistry,
};
use lyng_types::{ObjectRef, Value};

use crate::dsl::feedback_flat::FeedbackEntry;
use crate::error::VmError;
use crate::vm::dispatch_state::DispatchState;
use crate::vm::install::InstalledFunction;
use crate::{FrameRecord, Vm};

pub const LLINT_FRAME_INFO_HEADROOM: usize = 64;
pub const LLINT_REGISTER_STACK_HEADROOM: usize = 1024;
pub const LLINT_MAX_BYTECODE_CALL_DEPTH: usize = 8_192;
pub const LLINT_FRAME_INFO_FAST_RETURN_SAFE: u32 = 1;
pub const LLINT_FRAME_INFO_STRICT: u32 = 1 << 1;
pub const LLINT_FRAME_INFO_TAIL_CALL_RECYCLE_SAFE: u32 = 1 << 2;
pub const LLINT_RETURN_REGISTER_NONE: u32 = u32::MAX;
pub const LLINT_CALL_TARGET_ENABLED: u32 = 1;
pub const LLINT_CALL_TARGET_FAST_RETURN_SAFE: u32 = 1 << 1;
pub const LLINT_CALL_TARGET_THIS_GLOBAL: u32 = 1 << 2;
pub const LLINT_CALL_TARGET_STRICT: u32 = 1 << 3;
pub const LLINT_CALL_TARGET_TAIL_CALL_RECYCLE_SAFE: u32 = 1 << 4;

/// Compact asm-facing frame metadata for no-cleanup call/return paths.
///
/// The canonical `FrameRecord` stays Rust-owned. This mirror contains
/// only the fields the `LLInt` hit paths need while they defer
/// materializing pushed/popped frames back into `Vm::frames` until the
/// next Rust slow-path boundary.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LlIntFrameInfo {
    pub pb_base: *const u8,
    pub regs_base: *mut Value,
    pub fv_base: *mut FeedbackEntry,
    pub const_base: *const Value,
    pub this_value: Value,
    pub pc_offset: u32,
    pub return_register: u32,
    pub flags: u32,
    pub register_base: u32,
    pub register_len: u32,
    pub code_raw: u32,
    pub realm_raw: u32,
    pub lexical_env_raw: u32,
    pub variable_env_raw: u32,
    pub private_env_raw: u32,
    pub callee_raw: u32,
    pub parameter_initializer_end_offset: u32,
    pub frame_flags_raw: u32,
    pub tail_caller_raw: u32,
    pub tail_caller_strict: u32,
    pub pad: [u64; 3],
}

impl Default for LlIntFrameInfo {
    fn default() -> Self {
        Self {
            pb_base: std::ptr::null(),
            regs_base: std::ptr::null_mut(),
            fv_base: std::ptr::null_mut(),
            const_base: std::ptr::null(),
            this_value: Value::undefined(),
            pc_offset: 0,
            return_register: LLINT_RETURN_REGISTER_NONE,
            flags: 0,
            register_base: 0,
            register_len: 0,
            code_raw: 0,
            realm_raw: 0,
            lexical_env_raw: 0,
            variable_env_raw: 0,
            private_env_raw: 0,
            callee_raw: 0,
            parameter_initializer_end_offset: 0,
            frame_flags_raw: 0,
            tail_caller_raw: 0,
            tail_caller_strict: 0,
            pad: [0; 3],
        }
    }
}

/// Compact asm-facing metadata for bytecode function objects eligible
/// for direct `LLInt` frame entry.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LlIntCallTarget {
    pub callee_bits: u64,
    pub pb_base: *const u8,
    pub fv_base: *mut FeedbackEntry,
    pub const_base: *const Value,
    pub global_this: Value,
    pub code_raw: u32,
    pub register_len: u32,
    pub parameter_count: u32,
    pub flags: u32,
    pub realm_raw: u32,
    pub lexical_env_raw: u32,
    pub variable_env_raw: u32,
    pub private_env_raw: u32,
    pub callee_raw: u32,
    pub parameter_initializer_end_offset: u32,
    pub _pad1: u32,
    pub _pad: [u64; 5],
}

impl Default for LlIntCallTarget {
    fn default() -> Self {
        Self {
            callee_bits: 0,
            pb_base: std::ptr::null(),
            fv_base: std::ptr::null_mut(),
            const_base: std::ptr::null(),
            global_this: Value::undefined(),
            code_raw: 0,
            register_len: 0,
            parameter_count: 0,
            flags: 0,
            realm_raw: 0,
            lexical_env_raw: 0,
            variable_env_raw: 0,
            private_env_raw: 0,
            callee_raw: 0,
            parameter_initializer_end_offset: 0,
            _pad1: 0,
            _pad: [0; 5],
        }
    }
}

/// Opaque marker for the Rust-side context pointer in [`LlIntState`].
///
/// The asm layer never reads through this pointer — it round-trips
/// the value through `state.rust_context` so the slow-path bridge can
/// reconstruct `&mut LlIntRustContext<'vm>`.
#[repr(C)]
pub struct LlIntRustContextOpaque {
    _private: [u8; 0],
}

/// asm-visible per-frame state. Stable across rustc versions because
/// it contains only thin pointers + integers (`repr(C)`).
///
/// Field order is part of the ABI; the const offsets in
/// [`crate::dsl::reg_convention`] are derived from this layout via
/// `offset_of!` and exercised by `tests::ll_int_state_offsets_stable`.
#[repr(C)]
pub struct LlIntState {
    pub frame_pc_offset: u32,
    pub _pad1: u32,
    pub frame_pb_base: *const u8,
    pub frame_regs_base: *mut Value,
    pub frame_fv_base: *mut FeedbackEntry,
    pub object_records_base: *const *const lyng_gc::RuntimeObjectRecord,
    pub object_slots_base: *const *const Value,
    // Phase 1.B.1: asm-visible frame context. `frame_const_base`
    // points into the active code record's pre-resolved constants
    // array (`RuntimeCodeRecord::constants` → `CodeSlotsRef`,
    // `&[Value]` from `heap.view().code_slots()`).
    // `frame_this_value` is a mirror of `frame.this_value()` for
    // `ThisState::Value(v)`, or `Value::uninitialized_lexical()` as
    // the bail-to-slow-path sentinel for
    // `ThisState::Uninitialized`/`Lexical`.
    //
    // Both fields are valid only between Refresh egress events; GC
    // can only happen during slow-path bridges, which refresh both
    // fields on egress. See spec §5 mirror discipline.
    pub frame_const_base: *const Value,
    pub frame_this_value: Value,
    pub frame_depth: u32,
    pub frame_check_epoch: u32,
    pub frame_info_base: *mut LlIntFrameInfo,
    pub frame_info_len: u32,
    pub register_stack_top: u32,
    pub register_stack_len: u32,
    pub register_stack_base: *mut Value,
    pub call_targets_base: *const LlIntCallTarget,
    pub call_targets_len: u32,
    pub _pad3: u32,
    pub rust_context: *mut LlIntRustContextOpaque,
    pub prefix: u8,
    pub _pad2: [u8; 7],
}

/// Rust-only per-call context the asm trampoline cannot observe
/// directly. The asm bridge gets to this struct through
/// `LlIntState::rust_context` (an opaque pointer), and only via the
/// reconstruction in `LlIntDispatchState::from_raw`.
///
/// DSL-0c restructure: the per-call Rust state lives inside a
/// [`DispatchState`] held here, rather than as flat fields on the
/// context. This lets the asm-path slow-path bridge call
/// [`crate::dsl::slow_path::LlIntDispatchState::dispatch_state`]
/// uniformly across α and asm — the semantic bodies under
/// `crate::vm::semantics::` all consume `DispatchState` directly,
/// so threading the same type through both dispatch paths keeps the
/// single-implementation invariant intact.
///
/// lyng-rmho restructure: the `dispatch` field is now lazy. At
/// trampoline entry the context stashes the constituent references
/// ([`DeferredDispatch`]) without building a `DispatchState`. The
/// first slow-shim that needs the dispatch state materializes it via
/// [`LazyDispatchState::ensure_built`]; subsequent slow shims within
/// the same `run_via_dsl` invocation reuse the cached
/// `DispatchState`. Pure-fast-path runs (no slow shim invocations)
/// pay nothing for the construction.
///
/// The lifetime `'vm` is the borrow on `Vm`/`Agent`/`HostHooks`/`Registry`
/// taken by `crate::dsl::entry::run_via_dsl` for the duration of one
/// trampoline invocation.
pub struct LlIntRustContext<'vm> {
    pub(crate) dispatch: LazyDispatchState<'vm>,
    pub(crate) exit: LlIntExitSlot,
    pub(crate) frame_infos: Vec<LlIntFrameInfo>,
    pub(crate) frame_info_register_stack_base: *mut Value,
}

/// Constituent references stashed at entry, used to construct a
/// [`DispatchState`] on the first slow-shim invocation. Mirrors
/// `DispatchState`'s public field set 1:1; only the lifetime of
/// construction differs.
///
/// This is a `pub(crate)` helper — the asm trampoline never observes
/// this type; it is reachable only through
/// [`LazyDispatchState::ensure_built`] inside slow-shim preamble.
pub(crate) struct DeferredDispatch<'vm> {
    pub(crate) vm: &'vm mut Vm,
    pub(crate) agent: &'vm mut Agent,
    pub(crate) host: &'vm dyn HostHooks,
    pub(crate) registry: &'vm mut (dyn NativeFunctionRegistry + 'vm),
    pub(crate) installed: Arc<InstalledFunction>,
    pub(crate) frame: FrameRecord,
    pub(crate) frame_depth: usize,
    pub(crate) frame_check_epoch: u32,
}

impl<'vm> DeferredDispatch<'vm> {
    /// Materialize the `DispatchState`. Consumes the deferred
    /// references — the `&mut`s move into the `DispatchState`'s
    /// fields, preventing aliasing.
    #[inline]
    pub(crate) fn into_dispatch_state(self) -> DispatchState<'vm> {
        DispatchState::new_for_dsl_entry(
            self.vm,
            self.agent,
            self.host,
            self.registry,
            self.installed,
            self.frame,
            self.frame_depth,
            self.frame_check_epoch,
        )
    }
}

/// Lazy `DispatchState` holder. Constructed in the `Pending` variant
/// by `dsl::entry::run_via_dsl`; transitions to `Built` on the first
/// slow-shim invocation that calls
/// [`crate::dsl::slow_path::LlIntDispatchState::dispatch_state`] (or
/// any other accessor that needs the built form). The `Poisoned`
/// variant is a transient sentinel used by [`Self::ensure_built`] to
/// move the deferred references out of `self` without violating the
/// borrow checker; it should never be externally observable.
pub(crate) enum LazyDispatchState<'vm> {
    Pending(DeferredDispatch<'vm>),
    Built(DispatchState<'vm>),
    /// Transient sentinel during `ensure_built` — only the move
    /// between the call to `mem::replace` and the assignment back to
    /// `Built(...)` can observe this variant. External accessors
    /// treat it as a bug.
    Poisoned,
}

impl<'vm> LazyDispatchState<'vm> {
    /// Lazily build the `DispatchState` and return a mutable reference
    /// to it. Subsequent calls return the cached state without
    /// rebuilding.
    #[inline]
    pub(crate) fn ensure_built(&mut self) -> &mut DispatchState<'vm> {
        if matches!(self, LazyDispatchState::Pending(_)) {
            let prev = core::mem::replace(self, LazyDispatchState::Poisoned);
            match prev {
                LazyDispatchState::Pending(components) => {
                    *self = LazyDispatchState::Built(components.into_dispatch_state());
                }
                // Unreachable: outer `matches!` confirmed `Pending`.
                _ => unreachable!(),
            }
        }
        match self {
            LazyDispatchState::Built(state) => state,
            LazyDispatchState::Pending(_) => {
                // The block above just transitioned out of Pending.
                unreachable!("ensure_built post-condition: state must be Built")
            }
            LazyDispatchState::Poisoned => {
                unreachable!(
                    "LazyDispatchState::Poisoned observed — ensure_built lost its components"
                )
            }
        }
    }

    /// Read the active frame's instruction offset without forcing the
    /// `DispatchState` to be built. Used by
    /// [`crate::dsl::slow_path::LlIntDispatchState::current_instruction_offset`]
    /// when callers want PC inspection without paying for full
    /// `DispatchState` construction.
    #[inline]
    pub(crate) fn frame_instruction_offset(&self) -> u32 {
        match self {
            LazyDispatchState::Pending(c) => c.frame.instruction_offset(),
            LazyDispatchState::Built(s) => s.frame.instruction_offset(),
            LazyDispatchState::Poisoned => unreachable!(
                "LazyDispatchState::Poisoned observed outside ensure_built transition"
            ),
        }
    }

    /// Update the active frame's instruction offset without forcing
    /// the `DispatchState` to be built. Used by
    /// [`crate::dsl::slow_path::LlIntDispatchState::sync_from_asm`]
    /// to mirror the asm-side `frame_pc_offset` into the Rust-side
    /// snapshot before any semantic body observes it.
    #[inline]
    pub(crate) fn set_frame_instruction_offset(&mut self, offset: u32) {
        match self {
            LazyDispatchState::Pending(c) => c.frame.set_instruction_offset(offset),
            LazyDispatchState::Built(s) => s.frame.set_instruction_offset(offset),
            LazyDispatchState::Poisoned => unreachable!(),
        }
    }
}

/// Slot the slow-path bridge writes when a semantic body chooses to
/// exit the trampoline. Read by `run_via_dsl` after the trampoline
/// returns; the discriminant maps directly to `VmResult<Value>`.
pub struct LlIntExitSlot {
    pub kind: ExitKind,
    pub done_value: Value,
    pub error: Option<Box<VmError>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExitKind {
    None,
    Done,
    Error,
}

impl Default for LlIntExitSlot {
    fn default() -> Self {
        Self {
            kind: ExitKind::None,
            done_value: Value::undefined(),
            error: None,
        }
    }
}

/// Lower-level helper: maps a (`ThisState`, frame-`this`-value
/// fallback) pair to the mirror value stored in
/// [`LlIntState::frame_this_value`]. Pure / no side effects /
/// trivially unit-testable.
///
/// Phase 1.B.1 sentinel rule:
/// - `ThisState::Value(v)` → `v` (real `this` binding)
/// - `ThisState::Uninitialized` → `Value::uninitialized_lexical()` (bail)
/// - `ThisState::Lexical` → `Value::uninitialized_lexical()` (bail)
/// - `None` (no current execution context) → fallback
///
/// The sentinel is observed by inline `op_load_this` handlers (landed
/// in Phase 1.B.2); on match the handler bails to the slow path,
/// which handles the throw / lex-env walk as appropriate.
#[inline]
pub(crate) const fn resolve_this_state_to_mirror(
    this_state: Option<lyng_env::ThisState>,
    fallback_frame_this: Value,
) -> Value {
    match this_state {
        Some(lyng_env::ThisState::Value(v)) => v,
        Some(lyng_env::ThisState::Uninitialized | lyng_env::ThisState::Lexical) => {
            Value::uninitialized_lexical()
        }
        None => fallback_frame_this,
    }
}

/// Top-level helper: derives the mirror from an `Agent` + a
/// `FrameRecord`. Mirrors the read path in
/// `crates/vm/src/vm/semantics/names.rs` so the pre-resolution
/// matches `op_load_this` semantics exactly.
///
/// Called from:
/// - `crate::dsl::entry::run_via_dsl` (initial population)
/// - `crate::dsl::slow_path::LlIntDispatchState::translate_outcome`
///   (Refresh arm)
#[inline]
pub(crate) fn resolve_initial_this_value(
    agent: &lyng_env::Agent,
    frame: &crate::FrameRecord,
) -> Value {
    let this_state = agent
        .current_execution_context()
        .map(lyng_env::ExecutionContext::this_state);
    let fallback = frame.this_value();
    resolve_this_state_to_mirror(this_state, fallback)
}

pub(crate) fn refresh_frame_infos(
    frame_infos: &mut Vec<LlIntFrameInfo>,
    vm: &mut crate::Vm,
    agent: &lyng_env::Agent,
) -> *mut Value {
    let register_stack_base = vm.register_stack_storage_mut_ptr();
    let target_len = vm
        .frames()
        .len()
        .saturating_add(LLINT_FRAME_INFO_HEADROOM)
        .min(LLINT_MAX_BYTECODE_CALL_DEPTH);
    if frame_infos.len() < target_len {
        frame_infos.resize(target_len, LlIntFrameInfo::default());
    }

    let mut bytecode_contexts = agent
        .execution_contexts()
        .iter()
        .copied()
        .filter(|context| matches!(context.executable(), ExecutableId::Bytecode(_)));
    for (index, frame) in vm.frames().iter().enumerate() {
        let context_this_state = bytecode_contexts
            .next()
            .filter(|context| context.executable() == ExecutableId::Bytecode(frame.code()))
            .map(lyng_env::ExecutionContext::this_state);
        let Some(installed) = vm.installed_for_dsl_runtime(frame.code()) else {
            continue;
        };
        let function = installed.function();
        let pb_base = function.instruction_bytes().as_ptr();
        let fv_base = {
            let index = crate::vm::code_index_for_dsl(frame.code());
            vm.feedback_flat_storage[index].as_ptr().cast_mut()
        };
        let const_base = agent
            .heap()
            .view()
            .code(frame.code())
            .and_then(lyng_gc::RuntimeCodeRecord::constants)
            .and_then(|slots| agent.heap().view().code_slots(slots))
            .map_or(std::ptr::null(), <[_]>::as_ptr);
        let regs_base = {
            let base = frame.registers().base() as usize;
            // SAFETY: the register window belongs to a live frame and
            // is within the reserved register-stack backing storage.
            unsafe { register_stack_base.add(base) }
        };
        let return_register = frame
            .return_register()
            .map_or(LLINT_RETURN_REGISTER_NONE, u32::from);
        let mut flags = 0;
        if installed.llint_simple_return_safe()
            && !frame.flags().contains(crate::FrameFlags::construct())
            && !frame
                .flags()
                .contains(crate::FrameFlags::derived_construct())
        {
            flags |= LLINT_FRAME_INFO_FAST_RETURN_SAFE;
        }
        if function.flags().strict() {
            flags |= LLINT_FRAME_INFO_STRICT;
        }
        if installed.llint_static_tail_recycle_safe()
            && !frame.flags().contains(crate::FrameFlags::construct())
            && !frame
                .flags()
                .contains(crate::FrameFlags::derived_construct())
            && vm.llint_frame_window_is_clear(frame)
        {
            flags |= LLINT_FRAME_INFO_TAIL_CALL_RECYCLE_SAFE;
        }
        if let Some(info) = frame_infos.get_mut(index) {
            *info = LlIntFrameInfo {
                pb_base,
                regs_base,
                fv_base,
                const_base,
                this_value: resolve_this_state_to_mirror(context_this_state, frame.this_value()),
                pc_offset: frame.instruction_offset(),
                return_register,
                flags,
                register_base: frame.registers().base(),
                register_len: u32::from(frame.registers().len()),
                code_raw: frame.code().get(),
                realm_raw: frame.realm().get(),
                lexical_env_raw: frame.lexical_env().get(),
                variable_env_raw: frame.variable_env().get(),
                private_env_raw: 0,
                callee_raw: frame.callee().map_or(0, ObjectRef::get),
                parameter_initializer_end_offset: frame.parameter_initializer_end_offset(),
                frame_flags_raw: u32::from(frame.flags().raw()),
                tail_caller_raw: frame.tail_caller().map_or(0, ObjectRef::get),
                tail_caller_strict: u32::from(frame.tail_caller_strict()),
                pad: [0; 3],
            };
        }
    }
    register_stack_base
}

pub(crate) fn llint_call_target_for_function(
    vm: &mut crate::Vm,
    agent: &lyng_env::Agent,
    object: ObjectRef,
    data: &FunctionObjectData,
) -> Option<LlIntCallTarget> {
    let FunctionEntryIdentity::Bytecode(code) = data.entry()? else {
        return None;
    };
    let installed = vm.installed_for_dsl_runtime(code)?;
    let function = installed.function();
    let flags = function.flags();
    if flags.generator()
        || flags.async_function()
        || flags.class_constructor()
        || flags.derived_class_constructor()
        || function.needs_environment()
        || function.arguments_mode() != lyng_bytecode::ArgumentsMode::None
        || function.has_rest_parameter()
        || !function.direct_eval_lexical_sites().is_empty()
        || !installed.llint_direct_entry_safe()
    {
        return None;
    }

    let this_mode = data.this_mode();
    if matches!(this_mode, FunctionThisMode::Lexical) {
        return None;
    }

    let realm = data.realm()?;
    let environment = data.environment()?;
    let register_len = function
        .register_count()
        .checked_add(function.hidden_register_count())?;
    let fv_base = {
        let index = crate::vm::code_index_for_dsl(code);
        vm.feedback_flat_storage[index].as_mut_ptr()
    };
    let const_base = agent
        .heap()
        .view()
        .code(code)
        .and_then(lyng_gc::RuntimeCodeRecord::constants)
        .and_then(|slots| agent.heap().view().code_slots(slots))
        .map_or(std::ptr::null(), <[_]>::as_ptr);
    let global_this = agent
        .realm(realm)
        .map(|record| record.global_object())
        .map_or_else(Value::undefined, Value::from_object_ref);
    let mut target_flags = LLINT_CALL_TARGET_ENABLED;
    if installed.llint_simple_return_safe() {
        target_flags |= LLINT_CALL_TARGET_FAST_RETURN_SAFE;
    }
    if this_mode == FunctionThisMode::Global {
        target_flags |= LLINT_CALL_TARGET_THIS_GLOBAL;
    }
    if flags.strict() {
        target_flags |= LLINT_CALL_TARGET_STRICT;
    }
    if installed.llint_static_tail_recycle_safe() {
        target_flags |= LLINT_CALL_TARGET_TAIL_CALL_RECYCLE_SAFE;
    }

    Some(LlIntCallTarget {
        callee_bits: Value::from_object_ref(object).bits(),
        pb_base: function.instruction_bytes().as_ptr(),
        fv_base,
        const_base,
        global_this,
        code_raw: code.get(),
        register_len: u32::from(register_len),
        parameter_count: u32::from(function.parameter_count()),
        flags: target_flags,
        realm_raw: realm.get(),
        lexical_env_raw: environment.get(),
        variable_env_raw: environment.get(),
        private_env_raw: data
            .private_env()
            .map_or(0, lyng_types::EnvironmentRef::get),
        callee_raw: object.get(),
        parameter_initializer_end_offset: function.parameter_initializer_end_offset(),
        _pad1: 0,
        _pad: [0; 5],
    })
}

pub(crate) fn llint_simple_return_safe(function: &lyng_bytecode::BytecodeFunction) -> bool {
    let flags = function.flags();
    if flags.class_constructor()
        || flags.derived_class_constructor()
        || flags.generator()
        || flags.async_function()
        || function.arguments_mode() != lyng_bytecode::ArgumentsMode::None
        || function.has_rest_parameter()
        || function.needs_environment()
        || !function.exception_handlers().is_empty()
    {
        return false;
    }

    no_dynamic_cleanup_opcodes(function)
}

pub(crate) fn llint_static_tail_recycle_safe(function: &lyng_bytecode::BytecodeFunction) -> bool {
    let flags = function.flags();
    if flags.class_constructor()
        || flags.derived_class_constructor()
        || flags.generator()
        || flags.async_function()
        || function.arguments_mode() != lyng_bytecode::ArgumentsMode::None
        || function.has_rest_parameter()
        || function.needs_environment()
        || !function.exception_handlers().is_empty()
        || !function.direct_eval_lexical_sites().is_empty()
        || !function.loop_iteration_environment_sites().is_empty()
    {
        return false;
    }

    no_dynamic_cleanup_opcodes(function)
}

pub(crate) fn llint_direct_entry_safe(function: &lyng_bytecode::BytecodeFunction) -> bool {
    function.instructions().iter().all(|instruction| {
        matches!(
            instruction.opcode(),
            lyng_bytecode::Opcode::Nop
                | lyng_bytecode::Opcode::LoadUndefined
                | lyng_bytecode::Opcode::LoadNull
                | lyng_bytecode::Opcode::LoadTrue
                | lyng_bytecode::Opcode::LoadFalse
                | lyng_bytecode::Opcode::LoadZero
                | lyng_bytecode::Opcode::LoadOne
                | lyng_bytecode::Opcode::LoadSmi
                | lyng_bytecode::Opcode::LoadConst
                | lyng_bytecode::Opcode::LdaUndefined
                | lyng_bytecode::Opcode::LdaNull
                | lyng_bytecode::Opcode::LdaTrue
                | lyng_bytecode::Opcode::LdaFalse
                | lyng_bytecode::Opcode::LdaZero
                | lyng_bytecode::Opcode::LdaOne
                | lyng_bytecode::Opcode::LdaSmi8
                | lyng_bytecode::Opcode::LdaConst8
                | lyng_bytecode::Opcode::Star0
                | lyng_bytecode::Opcode::Star1
                | lyng_bytecode::Opcode::Star2
                | lyng_bytecode::Opcode::Star3
                | lyng_bytecode::Opcode::Star4
                | lyng_bytecode::Opcode::Star5
                | lyng_bytecode::Opcode::Star6
                | lyng_bytecode::Opcode::Star7
                | lyng_bytecode::Opcode::Move
                | lyng_bytecode::Opcode::Return
                | lyng_bytecode::Opcode::ReturnUndefined
        )
    })
}

fn no_dynamic_cleanup_opcodes(function: &lyng_bytecode::BytecodeFunction) -> bool {
    !function.instructions().iter().any(|instruction| {
        matches!(
            instruction.opcode(),
            lyng_bytecode::Opcode::CreateForIn
                | lyng_bytecode::Opcode::AdvanceForIn
                | lyng_bytecode::Opcode::CloseForIn
                | lyng_bytecode::Opcode::CreateIterator
                | lyng_bytecode::Opcode::AdvanceIterator
                | lyng_bytecode::Opcode::CloseIterator
                | lyng_bytecode::Opcode::PushClosureEnv
                | lyng_bytecode::Opcode::PopClosureEnv
                | lyng_bytecode::Opcode::EnterEnvScope
                | lyng_bytecode::Opcode::LeaveEnvScope
                | lyng_bytecode::Opcode::PushWithEnv
                | lyng_bytecode::Opcode::PopWithEnv
                | lyng_bytecode::Opcode::Throw
                | lyng_bytecode::Opcode::EnterHandler
                | lyng_bytecode::Opcode::LeaveHandler
                | lyng_bytecode::Opcode::SuspendGeneratorStart
                | lyng_bytecode::Opcode::Yield
                | lyng_bytecode::Opcode::Await
                | lyng_bytecode::Opcode::DelegateYield
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::reg_convention as r;
    use lyng_env::ThisState;
    use lyng_types::Value;

    #[test]
    fn ll_int_state_offsets_stable() {
        // Lock in the asm-DSL ABI layout. Values were determined from
        // the first build of the `#[repr(C)]` struct above; the test
        // catches drift across rustc versions.
        assert_eq!(r::LLINT_STATE_FRAME_PC_OFFSET, 0);
        assert_eq!(r::LLINT_STATE_FRAME_PB_BASE, 8);
        assert_eq!(r::LLINT_STATE_FRAME_REGS_BASE, 16);
        assert_eq!(r::LLINT_STATE_FRAME_FV_BASE, 24);
        assert_eq!(r::LLINT_STATE_OBJECT_RECORDS_BASE, 32);
        assert_eq!(r::LLINT_STATE_OBJECT_SLOTS_BASE, 40);
        // Phase 1.B.1 plus outline-slot LLInt substrate: const/this
        // mirrors plus the two heap pointer tables occupy four 8-byte
        // slots before the scalar block.
        assert_eq!(r::LLINT_STATE_FRAME_CONST_BASE, 48);
        assert_eq!(r::LLINT_STATE_FRAME_THIS_VALUE, 56);
        assert_eq!(r::LLINT_STATE_FRAME_DEPTH, 64);
        assert_eq!(r::LLINT_STATE_FRAME_INFO_BASE, 72);
        assert_eq!(r::LLINT_STATE_FRAME_INFO_LEN, 80);
        assert_eq!(r::LLINT_STATE_REGISTER_STACK_TOP, 84);
        assert_eq!(r::LLINT_STATE_REGISTER_STACK_LEN, 88);
        assert_eq!(r::LLINT_STATE_REGISTER_STACK_BASE, 96);
        assert_eq!(r::LLINT_STATE_CALL_TARGETS_BASE, 104);
        assert_eq!(r::LLINT_STATE_CALL_TARGETS_LEN, 112);
        assert_eq!(r::LLINT_STATE_PREFIX, 128);
        assert_eq!(core::mem::size_of::<LlIntState>(), 136);
        assert_eq!(core::mem::size_of::<LlIntFrameInfo>(), 128);
        assert_eq!(core::mem::size_of::<LlIntCallTarget>(), 128);
    }

    #[test]
    fn resolve_this_state_value_passthrough() {
        let v = Value::from_smi(42);
        let result = resolve_this_state_to_mirror(Some(ThisState::Value(v)), v);
        assert_eq!(result, v);
    }

    #[test]
    fn resolve_this_state_uninitialized_returns_sentinel() {
        let fallback = Value::from_smi(99); // arbitrary; should be ignored.
        let result = resolve_this_state_to_mirror(Some(ThisState::Uninitialized), fallback);
        assert_eq!(result, Value::uninitialized_lexical());
    }

    #[test]
    fn resolve_this_state_lexical_returns_sentinel() {
        let fallback = Value::from_smi(99); // arbitrary; should be ignored.
        let result = resolve_this_state_to_mirror(Some(ThisState::Lexical), fallback);
        assert_eq!(result, Value::uninitialized_lexical());
    }

    #[test]
    fn resolve_this_state_none_falls_back_to_frame_this() {
        let fallback = Value::from_smi(7);
        let result = resolve_this_state_to_mirror(None, fallback);
        assert_eq!(result, fallback);
    }
}

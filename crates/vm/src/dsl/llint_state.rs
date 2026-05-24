//! asm-visible state record + Rust-only context per design §5.

use lyng_objects::{FunctionEntryIdentity, FunctionThisMode};
use lyng_types::{ObjectRef, Value};

use crate::dsl::feedback_flat::FeedbackEntry;
use crate::error::VmError;
use crate::vm::dispatch_state::DispatchState;
use lyng_env::ExecutableId;

pub const LLINT_FRAME_INFO_FAST_RETURN_SAFE: u32 = 1;
pub const LLINT_RETURN_REGISTER_NONE: u32 = u32::MAX;
pub const LLINT_MAX_BYTECODE_CALL_DEPTH: usize = 8_192;
pub const LLINT_REGISTER_STACK_SCRATCH_VALUES: usize = 65_536;
pub const LLINT_CALL_TARGET_ENABLED: u32 = 1;
pub const LLINT_CALL_TARGET_FAST_RETURN_SAFE: u32 = 1 << 1;
pub const LLINT_CALL_TARGET_THIS_GLOBAL: u32 = 1 << 2;

/// Compact asm-facing frame metadata for frame-return `LLInt` paths.
///
/// The canonical `FrameRecord` stays Rust-owned. This mirror contains
/// only fields a no-cleanup nested `Return` needs in order to restore
/// the caller frame and store the result without crossing into Rust.
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
    pub pad: [u64; 4],
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
            pad: [0; 4],
        }
    }
}

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
    pub pad1: u32,
    pub pad: [u64; 5],
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
            pad1: 0,
            pad: [0; 5],
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
    pub pad1: u32,
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
    pub pad3: u32,
    pub rust_context: *mut LlIntRustContextOpaque,
    pub prefix: u8,
    pub pad2: [u8; 7],
}

/// Rust-only per-call context the asm trampoline cannot observe directly.
///
/// The asm bridge gets to this struct through `LlIntState::rust_context`
/// (an opaque pointer), and only via the reconstruction in
/// `LlIntDispatchState::from_raw`.
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
/// The lifetime `'vm` is the borrow on `Vm`/`Agent`/`HostHooks`/`Registry`
/// taken by `crate::dsl::entry::run_via_dsl` for the duration of one
/// trampoline invocation.
pub struct LlIntRustContext<'vm> {
    pub(crate) dispatch: DispatchState<'vm>,
    pub(crate) exit: LlIntExitSlot,
    pub(crate) frame_infos: Vec<LlIntFrameInfo>,
    pub(crate) call_targets: Vec<LlIntCallTarget>,
    pub(crate) frame_info_register_stack_base: *mut Value,
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
    let mut bytecode_contexts = agent
        .execution_contexts()
        .iter()
        .copied()
        .filter(|context| matches!(context.executable(), ExecutableId::Bytecode(_)));
    frame_infos.clear();
    frame_infos.resize(LLINT_MAX_BYTECODE_CALL_DEPTH, LlIntFrameInfo::default());
    for (index, frame) in vm.frames().iter().enumerate() {
        let context_this_state = bytecode_contexts
            .next()
            .filter(|context| context.executable() == ExecutableId::Bytecode(frame.code()))
            .map(lyng_env::ExecutionContext::this_state);
        let Some(installed) = vm.installed_for_dsl_runtime(frame.code()) else {
            continue;
        };
        let pb_base = installed.function().instruction_bytes().as_ptr();
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
            // SAFETY: the register window belongs to an installed live
            // frame and is within the reserved register stack storage.
            unsafe { register_stack_base.add(base) }
        };
        let return_register = frame
            .return_register()
            .map_or(LLINT_RETURN_REGISTER_NONE, u32::from);
        let simple_return_safe = installed.llint_simple_return_safe()
            && !frame.flags().contains(crate::FrameFlags::construct())
            && !frame
                .flags()
                .contains(crate::FrameFlags::derived_construct());
        frame_infos[index] = LlIntFrameInfo {
            pb_base,
            regs_base,
            fv_base,
            const_base,
            this_value: resolve_this_state_to_mirror(context_this_state, frame.this_value()),
            pc_offset: frame.instruction_offset(),
            return_register,
            flags: if simple_return_safe {
                LLINT_FRAME_INFO_FAST_RETURN_SAFE
            } else {
                0
            },
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
            pad: [0; 4],
        };
    }
    register_stack_base
}

pub(crate) fn refresh_call_targets(
    call_targets: &mut Vec<LlIntCallTarget>,
    vm: &mut crate::Vm,
    agent: &lyng_env::Agent,
) {
    call_targets.clear();
    call_targets.push(LlIntCallTarget::default());
    if vm.debug_poll_enabled() {
        return;
    }
    for (object, data) in agent.objects().function_data_entries() {
        let index = object.get() as usize;
        if call_targets.len() <= index {
            call_targets.resize(index + 1, LlIntCallTarget::default());
        }
        let Some(target) = llint_call_target_for_function(vm, agent, object, data) else {
            continue;
        };
        call_targets[index] = target;
    }
}

fn llint_call_target_for_function(
    vm: &mut crate::Vm,
    agent: &lyng_env::Agent,
    object: ObjectRef,
    data: &lyng_objects::FunctionObjectData,
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
        pad1: 0,
        pad: [0; 5],
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

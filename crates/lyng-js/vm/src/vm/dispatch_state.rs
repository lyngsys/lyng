#![allow(
    improper_ctypes_definitions,
    reason = "extern \"C\" handlers carry Rust enums by value as an ABI-stability choice, not as a real FFI boundary"
)]

//! Phase 1 Option α dispatch primitives — the per-handler ABI specified in
//! `reports/js/lyng-js/jsc-aligned-engine-roadmap.md` and verified by the
//! `lyng-33i2` trampoline spike (see `reports/js/lyng-js/phase-1-spike.md`).
//!
//! `DispatchState<'vm>` bundles every reference a handler needs — the live
//! `Vm`, `Agent`, host hooks, native-function registry, the active
//! `FrameRecord`, and the `Arc<InstalledFunction>` whose `instruction_bytes()`
//! the handler is decoding. Handlers are `extern "C" fn`s receiving
//! `&mut DispatchState` and returning `Step`.
//!
//! DSL-0c (Tasks C2–C5) deleted the α trampoline (`run_trampoline*`,
//! `Vm::run_via_trampoline`, `still_active`) because `Vm::run` now routes
//! through the asm-DSL path (`run_via_dsl`). The α `dispatch_handlers/`
//! family modules + `DISPATCH_TABLE` + `Step` + `DispatchState` survive
//! specifically for the wide-form prefix bridge in
//! `crate::dsl::handlers::warm::op_prefix_via_alpha`, which delegates
//! wide-form opcodes through the α DISPATCH_TABLE until proper wide-form
//! DSL decoders land in a future batch (see the comment block on
//! `dsl/handlers/warm.rs` `op_wide` / `op_extra_wide`). Semantic bodies
//! in `vm/semantics/` also consume `DispatchState` directly.

use std::sync::Arc;

use lyng_js_bytecode::Opcode;
use lyng_js_env::Agent;
use lyng_js_host::HostHooks;
use lyng_js_objects::NativeFunctionRegistry;
use lyng_js_types::{CodeRef, Value};

use crate::error::{VmError, VmResult};
use crate::FrameRecord;

use super::dispatch_handlers;
use super::install::InstalledFunction;
use super::{code_index, Vm};

/// Per-frame execution state threaded through every handler call.
///
/// All references share the `'vm` lifetime — the state exists only for one
/// dispatch invocation. Handlers split-borrow the fields when they
/// need both `&mut vm` and another `&mut` field at once:
///
/// ```ignore
/// let DispatchState { vm, agent, host, registry, frame, .. } = &mut *state;
/// let result = vm.execute_add_opcode(agent, host, registry, frame, b, c);
/// ```
///
/// DSL-0c kept `DispatchState` because (a) every semantic body in
/// `vm/semantics/` consumes it through `LlIntDispatchState::dispatch_state()`,
/// (b) the `LlIntRustContext::dispatch` field on the asm side holds one,
/// and (c) the wide-form prefix bridge passes one to α handlers.
pub struct DispatchState<'vm> {
    pub(crate) vm: &'vm mut Vm,
    pub(crate) agent: &'vm mut Agent,
    pub(crate) host: &'vm dyn HostHooks,
    pub(crate) registry: &'vm mut (dyn NativeFunctionRegistry + 'vm),
    pub(crate) installed: Arc<InstalledFunction>,
    pub(crate) frame: FrameRecord,
    pub(crate) frame_depth: usize,
    pub(crate) frame_check_epoch: u32,
    /// Set by `op_wide` / `op_extra_wide` to widen the next handler's
    /// operand decoding. The semantic handler consumes the prefix via
    /// `state.prefix.take()` so subsequent handlers see `None`.
    pub(crate) prefix: Option<Opcode>,
}

impl<'vm> DispatchState<'vm> {
    /// DSL-0b validation-case helper: construct a `DispatchState` from
    /// pre-built components for the `crate::dsl::test_helpers` harness.
    /// Production code now builds `DispatchState` inline inside
    /// [`Self::new_for_dsl_entry`] (called from `dsl::entry::run_via_dsl`);
    /// the harness needs a public path because
    /// [`crate::dsl::test_helpers::DslHarness::with_alpha_dispatch`]
    /// lives in a module that the integration tests can see.
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_for_dsl_harness(
        vm: &'vm mut Vm,
        agent: &'vm mut Agent,
        host: &'vm dyn HostHooks,
        registry: &'vm mut (dyn NativeFunctionRegistry + 'vm),
        installed: Arc<InstalledFunction>,
        frame: FrameRecord,
        init_prefix: Option<Opcode>,
    ) -> Self {
        Self {
            vm,
            agent,
            host,
            registry,
            installed,
            frame,
            frame_depth: 0,
            frame_check_epoch: 0,
            prefix: init_prefix,
        }
    }

    /// DSL-0c entry helper: construct a `DispatchState` for the
    /// [`crate::dsl::entry::run_via_dsl`] path. The DSL entry shim
    /// then wraps it in an [`crate::dsl::llint_state::LlIntRustContext`]
    /// so the asm-path slow-path bridge can call
    /// [`crate::dsl::slow_path::LlIntDispatchState::dispatch_state`]
    /// and reach the same `DispatchState` α handlers see.
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_for_dsl_entry(
        vm: &'vm mut Vm,
        agent: &'vm mut Agent,
        host: &'vm dyn HostHooks,
        registry: &'vm mut (dyn NativeFunctionRegistry + 'vm),
        installed: Arc<InstalledFunction>,
        frame: FrameRecord,
        frame_depth: usize,
        frame_check_epoch: u32,
    ) -> Self {
        Self {
            vm,
            agent,
            host,
            registry,
            installed,
            frame,
            frame_depth,
            frame_check_epoch,
            prefix: None,
        }
    }

    /// Bytes from the active PC to the end of the function's instruction
    /// stream. Handlers slice this to decode their operands and look up the
    /// next opcode byte for `dispatch_next!`.
    #[inline]
    pub(crate) fn current_bytes(&self) -> &[u8] {
        let pc = self.frame.instruction_offset() as usize;
        &self.installed.function.instruction_bytes()[pc..]
    }

    // DSL-0c C5: first_opcode_byte deleted with `run_trampoline*` — the
    // only caller. α handlers reach the next opcode byte through
    // `next_opcode_byte` instead.

    /// Hot-path read of the byte at the current `pc`, with the slice
    /// bounds check elided. Mirrors JSC LLInt's `loadb [PB, PC, 1], t0`
    /// pattern: the bytecode validator guarantees that any opcode
    /// reachable via dispatch is followed by another valid opcode byte
    /// (every script-completion path ends in `Return` / `ReturnUndefined`,
    /// which exit via `Step::Done` rather than continuing dispatch).
    ///
    /// # Safety
    ///
    /// Caller must guarantee `self.frame.instruction_offset() <
    /// self.installed.function.instruction_bytes().len()`. The dispatch
    /// path satisfies this via the bytecode-emitter invariant and
    /// terminal-opcode semantics described above.
    #[inline]
    pub(crate) fn next_opcode_byte(&self) -> u8 {
        let bytes = self.installed.function.instruction_bytes();
        let pc = self.frame.instruction_offset() as usize;
        debug_assert!(
            pc < bytes.len(),
            "dispatch_next! reached past end of bytecode — terminal opcode invariant violated"
        );
        // SAFETY: contract above — every dispatched opcode is followed
        // by another opcode byte; terminal opcodes (Return /
        // ReturnUndefined) exit via Step::Done, not dispatch_next!.
        unsafe { *bytes.as_ptr().add(pc) }
    }

    /// Hot-path PC advance, with the u32-overflow check elided.
    /// Validated bytecode is bounded far below `u32::MAX`, so
    /// `wrapping_add` is functionally equivalent to `checked_add` for
    /// any in-spec bytecode. Mirrors JSC LLInt's `addp Imm, PC` pattern
    /// (no overflow trap).
    #[inline]
    pub(crate) fn advance(&mut self, n: u32) {
        let next = self.frame.instruction_offset().wrapping_add(n);
        self.frame.set_instruction_offset(next);
    }

    #[inline]
    pub(crate) fn code(&self) -> CodeRef {
        self.frame.code()
    }

    /// Write `self.frame` back to `vm.frames[frame_depth - 1]`. Used before
    /// any handler operation that may inspect the live frame stack
    /// (return-from-frame, debugger safepoints, etc.).
    #[inline]
    pub(crate) fn sync_active_frame(&mut self) {
        let frame_depth = self.frame_depth;
        let frame = self.frame;
        self.vm.sync_dispatch_frame(frame_depth, frame);
    }

    /// Pop the agent's top execution context. Mirror of the
    /// `let _ = agent.pop_execution_context();` line in the legacy match.
    #[inline]
    pub(crate) fn pop_execution_context(&mut self) {
        let _ = self.agent.pop_execution_context();
    }

    /// Wrap `Vm::finish_frame` with the split borrow of `vm` and `agent` that
    /// the borrow checker requires through `&mut DispatchState`.
    #[inline]
    pub(crate) fn finish_active_frame(&mut self, value: Value) -> VmResult<Option<Value>> {
        let DispatchState { vm, agent, .. } = self;
        vm.finish_frame(agent, value)
    }

    /// Read constant `bx` from the active function's constant pool. Splits
    /// the `&mut vm` + `&mut agent` borrow that `Vm::read_constant`
    /// requires.
    #[inline]
    pub(crate) fn read_constant(&mut self, bx: u32) -> VmResult<Value> {
        let code = self.frame.code();
        let DispatchState { vm, agent, .. } = self;
        vm.read_constant(agent, code, bx)
    }

    /// Route a possibly-abrupt operation result through the exception
    /// transfer machinery. Returns `Ok(Some(value))` for success,
    /// `Ok(None)` if the abrupt completion was caught by an active handler
    /// (the handler should `dispatch_next!` to continue at the new PC), or
    /// `Err(error)` if the abrupt completion escapes the current code.
    ///
    /// Frame-state refresh after a cross-frame catch happens in
    /// `run_trampoline`'s epoch check, not here — `Vm::handle_dispatch_result`
    /// bumps the dispatch-frame-check epoch via `request_dispatch_frame_check`,
    /// so the trampoline picks up the unwind on the next iteration.
    #[inline]
    pub(crate) fn handle_dispatch_result<T>(&mut self, result: VmResult<T>) -> VmResult<Option<T>> {
        let DispatchState {
            vm,
            agent,
            frame,
            frame_depth,
            ..
        } = self;
        vm.handle_dispatch_result(agent, *frame_depth, frame, result)
    }

    /// Re-snapshot frame/depth/installed/epoch after a frame-changing
    /// operation. Required after a return that didn't terminate the script
    /// (caller frame is now active) or after a call (callee frame is now
    /// active).
    pub(crate) fn refresh_from_active_frame(&mut self) -> VmResult<()> {
        self.frame_depth = self.vm.frames().len();
        let frame = self
            .vm
            .frames()
            .last()
            .copied()
            .ok_or(VmError::MissingActiveFrame)?;
        self.frame = frame;
        let code = frame.code();
        let installed = self
            .vm
            .installed_for_code(code)
            .ok_or(VmError::MissingInstalledCode(code))?;
        self.installed = installed;
        self.frame_check_epoch = self.vm.dispatch_frame_check_epoch();
        Ok(())
    }
}

/// Per-opcode handler ABI. Each handler returns a `Step` describing what the
/// trampoline should do next.
pub type Handler = extern "C" fn(&mut DispatchState) -> Step;

/// Trampoline control-flow value. The trampoline keeps the active handler in
/// a local variable and only inspects this enum's discriminant.
pub enum Step {
    Continue(Handler),
    Done(Value),
    Error(VmError),
}

// DSL-0c C5: dispatch_next! and dispatch_next_with_value! macros deleted
// with the α trampoline. The α handlers in `dispatch_handlers/` no longer
// terminate with `dispatch_next!` — `translate_outcome_to_step` in
// `dispatch_handlers/mod.rs` constructs the `Step::Continue(next)`
// value directly. The prefix bridge in `dsl::handlers::warm`
// (`op_prefix_via_alpha`) likewise reads `DISPATCH_TABLE` directly.

/// `?`-like early-return for handlers. `Result<T, VmError>` → `T` on Ok, or
/// `return Step::Error(e)` on Err.
#[macro_export]
macro_rules! try_step {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(error) => return $crate::vm::dispatch_state::Step::Error(error),
        }
    };
}

/// Sized to cover every byte that could land in `bytes[pc]`. The first
/// `lyng_js_bytecode::OPCODE_COUNT` slots map to real or stub handlers; the
/// rest are `op_stub`, so an invalid byte fails cleanly rather than indexing
/// past the table.
pub const DISPATCH_TABLE_LEN: usize = 256;

/// Static dispatch table — one `Handler` per opcode byte value.
pub static DISPATCH_TABLE: [Handler; DISPATCH_TABLE_LEN] =
    dispatch_handlers::build_dispatch_table();

// DSL-0c C5: run_trampoline, run_trampoline_counted, still_active deleted.
// The α trampoline was the production dispatch loop until DSL-0c (Task C1)
// flipped `Vm::run` to `run_via_dsl`. The asm-DSL trampoline replaces
// `run_trampoline`; the epoch-check + `still_active` logic is now in
// `LlIntDispatchState::translate_outcome` in `dsl/slow_path.rs` (mirrors
// the same policy: only refresh on a true frame-stack change, never on a
// same-frame epoch bump). Wide-form prefix dispatch — the only remaining
// α consumer — invokes handlers directly through `DISPATCH_TABLE` from
// `dsl::handlers::warm::op_prefix_via_alpha` without going through a
// trampoline loop.

impl Vm {
    /// Look up the `Arc<InstalledFunction>` for a given `CodeRef`. Used by
    /// `DispatchState::refresh_from_active_frame` after a frame transition.
    #[inline]
    pub(in crate::vm) fn installed_for_code(&self, code: CodeRef) -> Option<Arc<InstalledFunction>> {
        self.installed
            .get(code_index(code))
            .and_then(Option::as_ref)
            .cloned()
    }

    /// DSL-0b validation-case helper: same lookup as
    /// [`Vm::installed_for_code`] but visible from the
    /// `crate::dsl::test_helpers` module (which is `pub` for
    /// integration-test consumption).
    #[doc(hidden)]
    #[inline]
    pub(crate) fn installed_for_dsl_harness(
        &self,
        code: CodeRef,
    ) -> Option<Arc<InstalledFunction>> {
        self.installed_for_code(code)
    }

    /// DSL-0c: crate-visible wrapper around
    /// [`Vm::installed_for_code`] for the slow-path `Refresh`
    /// machinery in [`crate::dsl::slow_path`]. Returns the
    /// `Arc<InstalledFunction>` for the post-frame-switch active
    /// frame so the asm bridge can recompute `pb_base` /
    /// `frame_fv_base` from a single source of truth.
    #[inline]
    pub(crate) fn installed_for_dsl_runtime(
        &self,
        code: CodeRef,
    ) -> Option<Arc<InstalledFunction>> {
        self.installed_for_code(code)
    }

    /// Read the for-in enumerator slot off the side table. Mirrors the
    /// legacy `self.for_in_states.advance(agent, base, register)` direct
    /// access from a trampoline-safe wrapper.
    #[inline]
    pub(in crate::vm) fn for_in_advance(
        &mut self,
        agent: &mut lyng_js_env::Agent,
        base: u32,
        register: u16,
    ) -> VmResult<Option<lyng_js_types::PropertyKey>> {
        self.for_in_states.advance(agent, base, register)
    }

    /// Insert an iterator enumerator into the for-in side table.
    #[inline]
    pub(in crate::vm) fn for_in_insert(
        &mut self,
        base: u32,
        register: u16,
        enumerator: lyng_js_ops::enumeration::ForInEnumerator,
    ) {
        self.for_in_states.insert(base, register, enumerator);
    }

    /// Drop the for-in enumerator at `register`.
    #[inline]
    pub(in crate::vm) fn for_in_remove(&mut self, base: u32, register: u16) {
        let _ = self.for_in_states.remove(base, register);
    }

    /// Insert an iterator state into the iterator side table.
    #[inline]
    pub(in crate::vm) fn iterator_insert(
        &mut self,
        base: u32,
        register: u16,
        iterator: lyng_js_ops::iterator::IteratorRecord,
    ) {
        self.iterator_states.insert(base, register, iterator);
    }

    /// Read the current exception (used by `LoadException`).
    #[inline]
    pub(in crate::vm) fn current_exception_value(&self) -> Value {
        self.current_exception().unwrap_or_else(Value::undefined)
    }

    // DSL-0c C5: run_via_trampoline deleted with the α trampoline.
    // `Vm::run` routes through `Vm::run_via_dsl` (see `dsl/entry.rs`).
}

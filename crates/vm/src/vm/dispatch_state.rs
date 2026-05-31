//! Per-frame dispatch state shared by semantic bodies.
//!
//! `DispatchState<'vm>` bundles every reference a semantic body needs — the
//! live `Vm`, `Agent`, host hooks, native-function registry, the active
//! `FrameRecord`, and the `Arc<InstalledFunction>` whose `instruction_bytes()`
//! the handler is decoding.
//!
//! DSL-0c finished α deletion: the `run_trampoline*` / `Vm::run_via_trampoline`
//! / `still_active` α trampoline, the per-handler `extern "C"` ABI (`Step` /
//! `Handler` / `DISPATCH_TABLE`), and the `dispatch_handlers/` family modules
//! were all removed. `DispatchState` survives because (a) every semantic body
//! in `vm/semantics/` consumes it through
//! `LlIntDispatchState::dispatch_state()`, (b) the
//! `LlIntRustContext::dispatch` field on the asm side holds one, and (c) the
//! wide-form prefix bridge passes one to the codegen-emitted
//! `dispatch_wide_form` function.

use std::sync::Arc;

use lyng_bytecode::Opcode;
use lyng_env::Agent;
use lyng_host::HostHooks;
use lyng_objects::NativeFunctionRegistry;
use lyng_types::{CodeRef, Value};

use crate::error::{VmError, VmResult};
use crate::FrameRecord;

use super::install::InstalledFunction;
use super::{code_index, Vm};

/// Per-frame execution state threaded through every semantic body.
///
/// All references share the `'vm` lifetime — the state exists only for one
/// dispatch invocation. Semantic bodies split-borrow the fields when they
/// need both `&mut vm` and another `&mut` field at once:
///
/// ```ignore
/// let DispatchState { vm, agent, host, registry, frame, .. } = &mut *state;
/// let result = vm.execute_add_opcode(agent, host, registry, frame, b, c);
/// ```
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

    #[inline]
    pub(crate) const fn code(&self) -> CodeRef {
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

    #[inline]
    pub(crate) fn refresh_dsl_poll_pending(&mut self) {
        self.vm.refresh_dsl_poll_pending_for_agent(self.agent);
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
    /// (the semantic body should continue at the new PC), or
    /// `Err(error)` if the abrupt completion escapes the current code.
    ///
    /// Frame-state refresh after a cross-frame catch happens in the asm
    /// dispatch loop's epoch check (mirrored in
    /// `LlIntDispatchState::translate_outcome`), not here —
    /// `Vm::handle_dispatch_result` bumps the dispatch-frame-check epoch via
    /// `request_dispatch_frame_check`, so the dispatcher picks up the unwind
    /// on the next iteration.
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

impl Vm {
    /// Look up the `Arc<InstalledFunction>` for a given `CodeRef`. Used by
    /// `DispatchState::refresh_from_active_frame` after a frame transition.
    #[inline]
    pub(in crate::vm) fn installed_for_code(
        &self,
        code: CodeRef,
    ) -> Option<Arc<InstalledFunction>> {
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
    /// frame so the asm bridge can recompute `pb_base` and
    /// `frame_metadata_table_base` from a single source of truth.
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
        agent: &mut lyng_env::Agent,
        base: u32,
        register: u16,
    ) -> VmResult<Option<lyng_types::PropertyKey>> {
        self.for_in_states.advance(agent, base, register)
    }

    /// Insert an iterator enumerator into the for-in side table.
    #[inline]
    pub(in crate::vm) fn for_in_insert(
        &mut self,
        base: u32,
        register: u16,
        enumerator: lyng_ops::enumeration::ForInEnumerator,
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
        iterator: lyng_ops::iterator::IteratorRecord,
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

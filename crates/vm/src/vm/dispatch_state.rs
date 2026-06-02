//! Per-frame dispatch state shared by semantic bodies.
//!
//! `DispatchState<'vm>` bundles every reference a semantic body needs — the
//! live `Vm`, `Agent`, host hooks, native-function registry, and the
//! `Arc<InstalledFunction>` whose `instruction_bytes()` the handler is decoding.
//! Every semantic body in `vm/semantics/` consumes it through
//! `LlIntDispatchState::dispatch_state()`; the `LlIntRustContext::dispatch`
//! field on the asm side holds one; the wide-form prefix bridge passes one to
//! the codegen-emitted `dispatch_wide_form`.

use std::sync::Arc;

use lyng_bytecode::Opcode;
use lyng_env::Agent;
use lyng_host::HostHooks;
use lyng_objects::NativeFunctionRegistry;
use lyng_types::{CodeRef, Value};

use crate::FrameRecord;
use crate::error::{VmError, VmResult};

use super::install::InstalledFunction;
use super::{Vm, code_index};

/// Per-frame execution state threaded through every semantic body.
///
/// All references share the `'vm` lifetime. Frame fields are read on demand from
/// the arena overlay via `vm.frame_header(self.cfr)`; the register window, live
/// PC, and code come from the thin view (`cfr`/`regs_len`/`pc`/`code_ref`).
/// Semantic bodies split-borrow the fields when they need both `&mut vm` and
/// another `&mut` field at once — pull any needed overlay value into a `Copy`
/// local *before* the destructure:
///
/// ```ignore
/// let registers = inner.registers();
/// let DispatchState { vm, agent, host, registry, .. } = &mut *state;
/// let result = vm.execute_add_opcode(agent, host, registry, registers, b, c);
/// ```
pub struct DispatchState<'vm> {
    pub(crate) vm: &'vm mut Vm,
    pub(crate) agent: &'vm mut Agent,
    pub(crate) host: &'vm dyn HostHooks,
    pub(crate) registry: &'vm mut (dyn NativeFunctionRegistry + 'vm),
    pub(crate) installed: Arc<InstalledFunction>,
    /// Active frame's cfr (arena slot index of its `FrameHeader`).
    pub(crate) cfr: u32,
    /// Live program counter. Not parked in the overlay mid-frame; synced
    /// to/from the asm side's `LlIntState.frame_pc_offset` at slow-path
    /// boundaries.
    pub(crate) pc: u32,
    /// Cached active `CodeRef`. Hot in the decode + jump paths.
    pub(crate) code_ref: CodeRef,
    /// Active register-window length (window base = `cfr + HEADER_SLOTS`).
    pub(crate) regs_len: u16,
    pub(crate) frame_depth: usize,
    pub(crate) frame_check_epoch: u32,
    /// Set by `op_wide` / `op_extra_wide` to widen the next handler's
    /// operand decoding. The semantic handler consumes the prefix via
    /// `state.prefix.take()` so subsequent handlers see `None`.
    pub(crate) prefix: Option<Opcode>,
}

impl<'vm> DispatchState<'vm> {
    /// Construct a `DispatchState` for the `crate::dsl::test_helpers` harness.
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
            cfr: Vm::cfr_of(&frame),
            pc: frame.instruction_offset(),
            code_ref: frame.code(),
            regs_len: frame.registers().len(),
            frame_depth: 0,
            frame_check_epoch: 0,
            prefix: init_prefix,
        }
    }

    /// Construct a `DispatchState` for the [`crate::dsl::entry::run_via_dsl`]
    /// path.
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
            cfr: Vm::cfr_of(&frame),
            pc: frame.instruction_offset(),
            code_ref: frame.code(),
            regs_len: frame.registers().len(),
            frame_depth,
            frame_check_epoch,
            prefix: None,
        }
    }

    /// Cold-table index of the active (top) frame: `frame_depth - 1`.
    #[inline]
    const fn cold_index(&self) -> usize {
        self.frame_depth - 1
    }

    #[inline]
    pub(crate) fn resume_kind(&self) -> crate::frame::GeneratorResumeKind {
        self.vm.frame_cold.get(self.cold_index()).resume_kind
    }

    #[inline]
    pub(crate) fn resume_value(&self) -> Value {
        self.vm.frame_cold.get(self.cold_index()).resume_value
    }

    #[allow(
        dead_code,
        reason = "completes the cold resume reader set; mirror of resume_kind/value"
    )]
    #[inline]
    pub(crate) fn resume_active(&self) -> bool {
        self.vm.frame_cold.get(self.cold_index()).resume_active
    }

    #[inline]
    pub(crate) fn clear_resume(&mut self) {
        let i = self.cold_index();
        self.vm.frame_cold.get_mut(i).resume_active = false;
    }

    #[inline]
    pub(crate) const fn frame_view(&self) -> crate::frame::FrameView {
        crate::frame::FrameView::new(self.cfr, self.pc, self.regs_len, self.code_ref)
    }

    #[inline]
    pub(crate) const fn code(&self) -> CodeRef {
        self.code_ref
    }

    #[inline]
    pub(crate) const fn pc(&self) -> u32 {
        self.pc
    }

    #[inline]
    pub(crate) const fn registers(&self) -> crate::frame::RegisterWindow {
        crate::frame::RegisterWindow::new(
            self.cfr + crate::frame_header::HEADER_SLOTS as u32,
            self.regs_len,
        )
    }

    /// Park the live PC into the active frame's overlay `saved_pc`. Call before
    /// any handler operation that may inspect or switch the active frame.
    #[inline]
    pub(crate) fn sync_active_frame(&mut self) {
        if self.frame_depth != 0 {
            self.vm.park_caller_pc(self.cfr, self.pc);
        }
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
        let code = self.code_ref;
        let DispatchState { vm, agent, .. } = self;
        vm.read_constant(agent, code, bx)
    }

    /// Route a possibly-abrupt result through exception-transfer machinery.
    ///
    /// Returns `Ok(Some(value))` on success, `Ok(None)` if the abrupt
    /// completion was caught by an active handler (continue at new PC), or
    /// `Err(error)` if it escapes. Cross-frame catch triggers a
    /// dispatch-frame-check epoch bump so the dispatcher refreshes on the
    /// next iteration.
    #[inline]
    pub(crate) fn handle_dispatch_result<T>(&mut self, result: VmResult<T>) -> VmResult<Option<T>> {
        let cfr = self.cfr;
        let pc = self.pc;
        let frame_depth = self.frame_depth;
        let handled = {
            let DispatchState { vm, agent, .. } = &mut *self;
            vm.handle_dispatch_result(agent, frame_depth, cfr, pc, result)?
        };
        // On a same-frame caught throw (`handled.is_none()`), `transfer_to_exception_handler`
        // parked the handler PC into the overlay `saved_pc` — reload from it. On success the
        // thin-view PC is already correct. On a cross-frame result leave the thin view stale
        // so the slow-path egress promotes Continue→Refresh.
        if self.vm.frame_depth() == self.frame_depth && handled.is_none() {
            self.pc = self.vm.frame_header(self.cfr).saved_pc();
        }
        Ok(handled)
    }

    /// Reload thin-view state from the now-active frame after a call or return.
    pub(crate) fn refresh_from_active_frame(&mut self) -> VmResult<()> {
        let depth = self.vm.frame_depth();
        self.frame_depth = depth;
        let cfr = self
            .vm
            .current_cfr_opt()
            .ok_or(VmError::MissingActiveFrame)?;
        let code = self.vm.frame_header(cfr).code();
        self.cfr = cfr;
        self.pc = self.vm.frame_header(cfr).saved_pc();
        self.code_ref = code;
        self.regs_len = self.vm.frame_window_len(cfr);
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

    /// Same as [`Vm::installed_for_code`] but visible from
    /// `crate::dsl::test_helpers`.
    #[doc(hidden)]
    #[inline]
    pub(crate) fn installed_for_dsl_harness(
        &self,
        code: CodeRef,
    ) -> Option<Arc<InstalledFunction>> {
        self.installed_for_code(code)
    }

    /// Same as [`Vm::installed_for_code`]; used by the slow-path `Refresh`
    /// machinery in [`crate::dsl::slow_path`].
    #[inline]
    pub(crate) fn installed_for_dsl_runtime(
        &self,
        code: CodeRef,
    ) -> Option<Arc<InstalledFunction>> {
        self.installed_for_code(code)
    }

    /// Advance the for-in enumerator at `(base, register)`.
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
}

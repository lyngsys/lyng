use super::call::finalize_frame_result;
use super::{Agent, RegisterWindow, Value, Vm, VmError, VmResult};
use lyng_types::AbruptCompletion;

impl Vm {
    #[inline]
    pub(super) fn read_register(&self, registers: RegisterWindow, register: u16) -> Value {
        let absolute = absolute_register(registers, register);
        debug_assert!(
            absolute < self.arena.top(),
            "validated register window should be reserved on the VM stack"
        );
        self.arena.slots()[absolute]
    }

    #[inline]
    pub(super) fn write_register(
        &mut self,
        registers: RegisterWindow,
        register: u16,
        value: Value,
    ) {
        let absolute = absolute_register(registers, register);
        debug_assert!(
            absolute < self.arena.top(),
            "validated register window should be reserved on the VM stack"
        );
        self.arena.slots_mut()[absolute] = value;
    }

    /// Frame-register read with the slice bounds check elided.
    ///
    /// Matches JSC `LLInt`'s `loadq [cfr, index, 8], value` pattern: the
    /// bytecode validator guarantees `register < window.len()` at
    /// compile time, and `reserve_frame` reserves
    /// `arena.top() >= absolute` before the frame executes. With
    /// both invariants held, the slice bounds check is dead work the
    /// hot dispatch path can shed.
    ///
    /// # Safety
    ///
    /// Caller must guarantee `register` came from a validated bytecode
    /// operand for the active frame, and the active frame's register
    /// window has been reserved via `reserve_frame`. Both hold
    /// for every operand decoded by the dispatch path because the
    /// emitter and frame-entry helpers enforce them; in release builds
    /// the `debug_assert!` in `absolute_register` plus the
    /// `debug_assert!` here are the only remaining checks.
    #[inline]
    pub(in crate::vm) fn read_register_unchecked(
        &self,
        registers: RegisterWindow,
        register: u16,
    ) -> Value {
        let absolute = absolute_register(registers, register);
        debug_assert!(
            absolute < self.arena.top(),
            "validated register window should be reserved on the VM stack"
        );
        // SAFETY: contract above — bytecode validation + reserved window.
        unsafe { *self.arena.slots().get_unchecked(absolute) }
    }

    /// Frame-register write with the slice bounds check elided. See
    /// [`read_register_unchecked`] for the safety contract.
    ///
    /// # Safety
    ///
    /// Same as [`read_register_unchecked`].
    #[inline]
    pub(in crate::vm) fn write_register_unchecked(
        &mut self,
        registers: RegisterWindow,
        register: u16,
        value: Value,
    ) {
        let absolute = absolute_register(registers, register);
        debug_assert!(
            absolute < self.arena.top(),
            "validated register window should be reserved on the VM stack"
        );
        // SAFETY: contract above — bytecode validation + reserved window.
        unsafe {
            *self.arena.slots_mut().get_unchecked_mut(absolute) = value;
        }
    }

    pub(super) fn clear_active_resume(&mut self) {
        let cold = self
            .current_cold_mut()
            .expect("clearing resume state requires one active frame");
        cold.resume_active = false;
    }

    pub(super) fn finish_frame(
        &mut self,
        agent: &mut Agent,
        result: Value,
    ) -> VmResult<Option<Value>> {
        // Decrement depth first so `close_*`/`internal_completion_targets` reads see
        // the caller's depth; the run stays mapped at `current_cfr` until
        // `release_frame_to_caller` reclaims it.
        debug_assert!(
            self.current_cfr != u32::MAX && self.frame_depth > 0,
            "finish requires one active frame"
        );
        let cfr = self.current_cfr;
        self.pop_frame_depth();
        let (this_value, lexical_env, construct_this, flags, return_register) = {
            let header = self.frame_header(cfr);
            (
                header.this_value(),
                header.lexical_env(),
                header.construct_this(),
                crate::frame::FrameFlags::from_raw(header.flags_bits()),
                header.return_register(),
            )
        };
        let window = RegisterWindow::new(
            cfr + crate::frame_header::HEADER_SLOTS as u32,
            self.frame_window_len(cfr),
        );
        // Derive the running context from the caller (the frame becoming current),
        // not the still-mapped finishing frame: spec-mandated derived-construct
        // throws must honor the caller's realm.
        self.refresh_running_context_to_caller(agent, cfr);
        self.request_dispatch_frame_check();
        self.close_loop_iteration_frames(self.frame_depth());
        self.close_with_environment_frames(self.frame_depth());
        self.close_direct_eval_frames(self.frame_depth());
        self.close_env_scope_frames(self.frame_depth());
        let finalized = finalize_frame_result(agent, flags, this_value, construct_this, result);
        self.for_in_states.clear_window(window);
        self.iterator_states.clear_window(window);
        self.captured_name_references.clear_window(window);
        self.finalize_mapped_arguments(agent, lexical_env)?;
        self.release_frame_to_caller(cfr);
        let _ = self.current_exception.take();

        let internal_completion_target =
            self.internal_completion_targets.last().copied() == Some(self.frame_depth());
        let result = match finalized {
            Ok(result) => result,
            Err(VmError::Abrupt(AbruptCompletion::Throw(value))) => {
                if internal_completion_target {
                    let _ = self.internal_completion_targets.pop();
                    return Err(VmError::Abrupt(AbruptCompletion::Throw(value)));
                }
                if self.transfer_to_exception_handler(agent, value)? {
                    return Ok(None);
                }
                return Err(VmError::Abrupt(AbruptCompletion::Throw(value)));
            }
            Err(error) => {
                if internal_completion_target {
                    let _ = self.internal_completion_targets.pop();
                }
                return Err(error);
            }
        };

        if internal_completion_target {
            let _ = self.internal_completion_targets.pop();
            return Ok(Some(result));
        }

        if let Some(caller_cfr) = self.current_cfr_opt() {
            if let Some(return_register) = return_register {
                let caller_window = RegisterWindow::new(
                    caller_cfr + crate::frame_header::HEADER_SLOTS as u32,
                    self.frame_window_len(caller_cfr),
                );
                self.write_register(caller_window, return_register, result);
            }
            return Ok(None);
        }

        Ok(Some(result))
    }
}

#[inline]
pub(in crate::vm) fn absolute_register(registers: RegisterWindow, register: u16) -> usize {
    debug_assert!(
        register < registers.len(),
        "bytecode register operand should be validated before execution"
    );
    let absolute = registers.base() + u32::from(register);
    debug_assert!(
        absolute < registers.end(),
        "register should remain inside the active frame window"
    );
    absolute as usize
}

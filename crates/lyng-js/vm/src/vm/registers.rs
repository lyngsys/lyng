use super::call::finalize_frame_result;
use super::{Agent, RegisterWindow, Value, Vm, VmError, VmResult};
use lyng_js_types::AbruptCompletion;

impl Vm {
    #[inline]
    pub(super) fn read_register(&self, registers: RegisterWindow, register: u16) -> Value {
        let absolute = absolute_register(registers, register);
        debug_assert!(
            absolute < self.register_stack_top(),
            "validated register window should be reserved on the VM stack"
        );
        self.register_stack[absolute]
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
            absolute < self.register_stack_top(),
            "validated register window should be reserved on the VM stack"
        );
        self.register_stack[absolute] = value;
    }

    pub(super) fn clear_active_resume(&mut self) {
        let frame = self
            .frames
            .last_mut()
            .expect("clearing resume state requires one active frame");
        frame.clear_resume();
    }

    pub(super) fn finish_frame(
        &mut self,
        agent: &mut Agent,
        result: Value,
    ) -> VmResult<Option<Value>> {
        let frame = self.frames.pop().expect("return requires one active frame");
        self.request_dispatch_frame_check();
        self.close_loop_iteration_frames(self.frames.len());
        self.close_with_environment_frames(self.frames.len());
        self.close_direct_eval_frames(self.frames.len());
        self.close_env_scope_frames(self.frames.len());
        let finalized = finalize_frame_result(agent, &frame, result);
        self.for_in_states.clear_window(frame.registers());
        self.iterator_states.clear_window(frame.registers());
        self.captured_name_references
            .clear_window(frame.registers());
        self.finalize_mapped_arguments(agent, frame.lexical_env())?;
        self.release_register_window(frame.registers().base());
        let _ = self.current_exception.take();

        let internal_completion_target =
            self.internal_completion_targets.last().copied() == Some(self.frames.len());
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

        if let Some(caller) = self.frames.last().copied() {
            if let Some(return_register) = frame.return_register() {
                self.write_register(caller.registers(), return_register, result);
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

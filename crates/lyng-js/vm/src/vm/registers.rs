use super::call::finalize_frame_result;
use super::{Agent, FrameRecord, Value, Vm, VmError, VmResult};
use lyng_js_types::AbruptCompletion;

impl Vm {
    pub(super) fn read_register(&self, frame: FrameRecord, register: u16) -> Value {
        let absolute = absolute_register(frame, register);
        debug_assert!(
            absolute < self.register_stack_top(),
            "validated register window should be reserved on the VM stack"
        );
        self.register_stack[absolute]
    }

    pub(super) fn write_register(&mut self, frame: FrameRecord, register: u16, value: Value) {
        let absolute = absolute_register(frame, register);
        debug_assert!(
            absolute < self.register_stack_top(),
            "validated register window should be reserved on the VM stack"
        );
        self.register_stack[absolute] = value;
    }

    pub(super) fn advance_instruction(&mut self) {
        let encoded_len = self.current_instruction_encoded_len();
        let frame = self
            .frames
            .last_mut()
            .expect("advance requires one active frame");
        let next = frame
            .instruction_offset()
            .checked_add(encoded_len)
            .expect("instruction offset should stay within u32");
        frame.set_instruction_offset(next);
    }

    pub(super) fn clear_active_resume(&mut self) {
        let frame = self
            .frames
            .last_mut()
            .expect("clearing resume state requires one active frame");
        frame.clear_resume();
    }

    pub(super) fn jump_by(&mut self, delta: i32) -> VmResult<()> {
        let encoded_len = i64::from(self.current_instruction_encoded_len());
        let frame = self
            .frames
            .last_mut()
            .expect("jump requires one active frame");
        let next = i64::from(frame.instruction_offset()) + encoded_len + i64::from(delta);
        if next < 0 {
            return Err(VmError::InvalidJumpTarget {
                code: frame.code(),
                instruction_offset: frame.instruction_offset(),
                target_offset: next,
            });
        }
        frame.set_instruction_offset(u32::try_from(next).map_err(|_| {
            VmError::InvalidJumpTarget {
                code: frame.code(),
                instruction_offset: frame.instruction_offset(),
                target_offset: next,
            }
        })?);
        Ok(())
    }

    /// Encoded byte length of the instruction the dispatch loop is currently executing.
    ///
    /// The dispatch loop writes this once per opcode after decoding from the byte stream so
    /// opcode handlers (and the `advance_instruction` / `jump_by` helpers they call) do not
    /// have to re-decode the same bytes to learn how far to move the program counter.
    #[inline]
    fn current_instruction_encoded_len(&self) -> u32 {
        debug_assert!(
            self.current_instruction_len > 0,
            "current_instruction_len should be set by the dispatch loop before advance_instruction or jump_by",
        );
        self.current_instruction_len
    }

    /// Byte offset of the instruction immediately following the one the dispatch loop is
    /// currently executing. Used by generator/yield/await suspends to record where the
    /// suspended frame should resume.
    pub(super) fn next_instruction_offset(&self, frame: FrameRecord) -> u32 {
        frame
            .instruction_offset()
            .checked_add(self.current_instruction_encoded_len())
            .expect("instruction offset should stay within u32")
    }

    pub(super) fn finish_frame(
        &mut self,
        agent: &mut Agent,
        result: Value,
    ) -> VmResult<Option<Value>> {
        let frame = self.frames.pop().expect("return requires one active frame");
        self.close_loop_iteration_frames(self.frames.len());
        self.close_with_environment_frames(self.frames.len());
        self.close_direct_eval_frames(self.frames.len());
        self.close_env_scope_frames(self.frames.len());
        let finalized = finalize_frame_result(agent, frame, result);
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
                self.write_register(caller, return_register, result);
            }
            return Ok(None);
        }

        Ok(Some(result))
    }
}

#[inline]
fn absolute_register(frame: FrameRecord, register: u16) -> usize {
    debug_assert!(
        register < frame.registers().len(),
        "bytecode register operand should be validated before execution"
    );
    let absolute = frame.registers().base() + u32::from(register);
    debug_assert!(
        absolute < frame.registers().end(),
        "register should remain inside the active frame window"
    );
    absolute as usize
}

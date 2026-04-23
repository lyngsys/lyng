use super::call::finalize_frame_result;
use super::*;
use lyng_js_types::AbruptCompletion;

impl Vm {
    pub(super) fn read_register(&self, frame: FrameRecord, register: u16) -> VmResult<Value> {
        let absolute = absolute_register(frame, register)?;
        self.register_stack
            .get(absolute)
            .copied()
            .ok_or(VmError::RegisterOutOfBounds {
                code: frame.code(),
                register,
            })
    }

    pub(super) fn write_register(
        &mut self,
        frame: FrameRecord,
        register: u16,
        value: Value,
    ) -> VmResult<()> {
        let absolute = absolute_register(frame, register)?;
        let slot = self
            .register_stack
            .get_mut(absolute)
            .ok_or(VmError::RegisterOutOfBounds {
                code: frame.code(),
                register,
            })?;
        *slot = value;
        Ok(())
    }

    pub(super) fn advance_instruction(&mut self) -> VmResult<()> {
        let frame = self
            .frames
            .last_mut()
            .expect("advance requires one active frame");
        let next = frame
            .instruction_offset()
            .checked_add(1)
            .expect("instruction offset should stay within u32");
        frame.set_instruction_offset(next);
        Ok(())
    }

    pub(super) fn clear_active_resume(&mut self) {
        let frame = self
            .frames
            .last_mut()
            .expect("clearing resume state requires one active frame");
        frame.clear_resume();
    }

    pub(super) fn jump_by(&mut self, delta: i32) -> VmResult<()> {
        let frame = self
            .frames
            .last_mut()
            .expect("jump requires one active frame");
        let next = i64::from(frame.instruction_offset()) + 1 + i64::from(delta);
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

    pub(super) fn finish_frame(
        &mut self,
        agent: &mut Agent,
        result: Value,
    ) -> VmResult<Option<Value>> {
        let frame = self.frames.pop().expect("return requires one active frame");
        self.close_loop_iteration_frames(self.frames.len());
        self.close_with_environment_frames(self.frames.len());
        self.close_direct_eval_frames(self.frames.len());
        let finalized = finalize_frame_result(agent, frame, result);
        self.for_in_states.clear_window(frame.registers());
        self.iterator_states.clear_window(frame.registers());
        self.captured_name_references
            .clear_window(frame.registers());
        self.finalize_mapped_arguments(agent, frame.lexical_env())?;
        self.register_stack.truncate(
            usize::try_from(frame.registers().base()).expect("base should fit into usize"),
        );
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
                self.write_register(caller, return_register, result)?;
            }
            return Ok(None);
        }

        Ok(Some(result))
    }
}

#[inline]
fn absolute_register(frame: FrameRecord, register: u16) -> VmResult<usize> {
    if register >= frame.registers().len() {
        return Err(VmError::RegisterOutOfBounds {
            code: frame.code(),
            register,
        });
    }
    usize::try_from(frame.registers().base() + u32::from(register)).map_err(|_| {
        VmError::RegisterOutOfBounds {
            code: frame.code(),
            register,
        }
    })
}

use super::{code_index, Agent, FrameRecord, InstalledFunction, Value, Vm, VmResult};
use lyng_js_bytecode::{ExceptionHandler, ExceptionHandlerKind, Instruction, Opcode};

impl Vm {
    pub(super) fn transfer_to_exception_handler(
        &mut self,
        agent: &mut Agent,
        thrown: Value,
    ) -> VmResult<bool> {
        loop {
            let Some(frame) = self.frames.last().copied() else {
                return Ok(false);
            };
            if self
                .internal_completion_targets
                .last()
                .copied()
                .is_some_and(|depth| self.frames.len() <= depth)
            {
                return Ok(false);
            }
            if let Some((index, handler)) = self.select_exception_handler(frame) {
                self.current_exception = Some(thrown);
                let frame = self
                    .frames
                    .last_mut()
                    .expect("checked above that one frame is active");
                frame.set_instruction_offset(handler.handler());
                frame.set_handler_cursor(u16::try_from(index + 1).unwrap_or(u16::MAX));
                return Ok(matches!(
                    handler.kind(),
                    ExceptionHandlerKind::Catch | ExceptionHandlerKind::Finally
                ));
            }
            if self.frames.len() == 1 {
                return Ok(false);
            }
            self.unwind_exception_frame(agent)?;
        }
    }

    fn select_exception_handler(&self, frame: FrameRecord) -> Option<(usize, ExceptionHandler)> {
        let installed = self
            .installed
            .get(code_index(frame.code()))
            .and_then(Option::as_ref)?;
        Self::suspended_call_instruction_offset(frame, installed)
            .and_then(|offset| Self::handler_covering_offset(installed, offset))
            .or_else(|| Self::handler_covering_offset(installed, frame.instruction_offset()))
    }

    fn handler_covering_offset(
        installed: &InstalledFunction,
        instruction_offset: u32,
    ) -> Option<(usize, ExceptionHandler)> {
        installed
            .function
            .exception_handlers()
            .iter()
            .copied()
            .enumerate()
            .find(|(_, handler)| {
                handler.protected_start() <= instruction_offset
                    && instruction_offset < handler.protected_end()
            })
    }

    fn suspended_call_instruction_offset(
        frame: FrameRecord,
        installed: &InstalledFunction,
    ) -> Option<u32> {
        let instruction_offset = frame.instruction_offset().checked_sub(1)?;
        match installed
            .function
            .instructions()
            .get(usize::try_from(instruction_offset).ok()?)
            .copied()?
        {
            Instruction::Abc {
                opcode:
                    Opcode::Call0
                    | Opcode::Call1
                    | Opcode::Call2
                    | Opcode::Call3
                    | Opcode::Call
                    | Opcode::TailCall
                    | Opcode::Construct,
                ..
            } => Some(instruction_offset),
            _ => None,
        }
    }

    fn unwind_exception_frame(&mut self, agent: &mut Agent) -> VmResult<()> {
        let frame = self
            .frames
            .pop()
            .expect("exception unwinding requires one active frame");
        self.close_loop_iteration_frames(self.frames.len());
        self.close_direct_eval_frames(self.frames.len());
        self.for_in_states.clear_window(frame.registers());
        self.iterator_states.clear_window(frame.registers());
        self.captured_name_references
            .clear_window(frame.registers());
        self.finalize_mapped_arguments(agent, frame.lexical_env())?;
        self.release_register_window(frame.registers().base());
        let _ = self.current_exception.take();
        let _ = agent.pop_execution_context();
        Ok(())
    }
}

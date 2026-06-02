use super::{Agent, InstalledFunction, Value, Vm, VmResult, code_index};
use crate::frame::FrameView;
use lyng_bytecode::{ExceptionHandler, ExceptionHandlerKind, Instruction, Opcode};

impl Vm {
    pub(in crate::vm) fn transfer_to_exception_handler(
        &mut self,
        agent: &mut Agent,
        thrown: Value,
    ) -> VmResult<bool> {
        loop {
            // The PC is the parked `saved_pc`: callers sync before calling here,
            // so the handler search runs against the correct PC.
            let Some(cfr) = self.current_cfr_opt() else {
                return Ok(false);
            };
            let frame = FrameView::new(
                cfr,
                self.frame_header(cfr).saved_pc(),
                self.frame_window_len(cfr),
                self.frame_header(cfr).code(),
            );
            if self
                .internal_completion_targets
                .last()
                .copied()
                .is_some_and(|depth| self.frame_depth() <= depth)
            {
                return Ok(false);
            }
            if let Some((index, handler)) = self.select_exception_handler(frame) {
                self.current_exception = Some(thrown);
                // Park the handler PC so the Refresh arm reloads `instruction_offset`
                // from it on the next frame switch.
                self.frame_header_mut(cfr).set_saved_pc(handler.handler());
                let handler_cursor = u16::try_from(index + 1).unwrap_or(u16::MAX);
                if let Some(cold) = self.current_cold_mut() {
                    cold.handler_cursor = handler_cursor;
                }
                let handled = matches!(
                    handler.kind(),
                    ExceptionHandlerKind::Catch | ExceptionHandlerKind::Finally
                );
                if handled {
                    self.request_dispatch_frame_check();
                }
                return Ok(handled);
            }
            if self.frame_depth() == 1 {
                return Ok(false);
            }
            self.unwind_exception_frame(agent)?;
        }
    }

    fn select_exception_handler(&self, frame: FrameView) -> Option<(usize, ExceptionHandler)> {
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
        frame: FrameView,
        installed: &InstalledFunction,
    ) -> Option<u32> {
        let (instruction_offset, instruction) =
            installed.instruction_before(frame.instruction_offset())?;
        match instruction {
            Instruction::Abc {
                opcode: Opcode::Call0 | Opcode::Call1 | Opcode::Call2 | Opcode::Call3,
                ..
            }
            | Instruction::AbcSlot {
                opcode: Opcode::Call0 | Opcode::Call1 | Opcode::Call2 | Opcode::Call3,
                ..
            }
            | Instruction::CallRange {
                opcode: Opcode::Call | Opcode::TailCall | Opcode::Construct,
                ..
            } => Some(instruction_offset),
            _ => None,
        }
    }

    fn unwind_exception_frame(&mut self, agent: &mut Agent) -> VmResult<()> {
        let frame = self.pop_current_frame();
        self.close_loop_iteration_frames(self.frame_depth());
        self.close_direct_eval_frames(self.frame_depth());
        self.for_in_states.clear_window(frame.registers());
        self.iterator_states.clear_window(frame.registers());
        self.captured_name_references
            .clear_window(frame.registers());
        let lexical_env = self.frame_header(Self::cfr_of(&frame)).lexical_env();
        self.finalize_mapped_arguments(agent, lexical_env)?;
        self.release_frame_to_caller(Self::cfr_of(&frame));
        let _ = self.current_exception.take();
        self.refresh_running_context(agent);
        Ok(())
    }
}

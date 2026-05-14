//! Exception handlers for the trampoline dispatch path (lyng-59e6 round 3).
//!
//! Throw routes through `Vm::transfer_to_exception_handler`. EnterHandler /
//! LeaveHandler are dispatch markers that just advance pc — the bytecode
//! emitter encodes the active try-stack via metadata that the helper reads
//! at throw time, not via runtime state managed by the dispatch loop.
//! LoadException reads `Vm::current_exception` and writes it to a register.

use lyng_js_types::AbruptCompletion;

use crate::error::VmError;
use crate::vm::dispatch::decode_ax_operands;
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::{dispatch_next, try_step};

pub extern "C" fn op_throw(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, _instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let register = match u16::try_from(ax) {
        Ok(r) => r,
        Err(_) => {
            return Step::Error(VmError::RegisterOutOfBounds {
                code: state.frame.code(),
                register: 0,
            });
        }
    };
    let value = state.vm.read_register_unchecked(state.frame.registers(), register);
    state.sync_active_frame();
    let transferred = {
        let DispatchState { vm, agent, .. } = &mut *state;
        vm.transfer_to_exception_handler(agent, value)
    };
    match transferred {
        Ok(true) => {
            try_step!(state.refresh_from_active_frame());
            dispatch_next!(state);
        }
        Ok(false) => Step::Error(VmError::Abrupt(AbruptCompletion::Throw(value))),
        Err(e) => Step::Error(e),
    }
}

pub extern "C" fn op_enter_handler(state: &mut DispatchState) -> Step {
    op_handler_marker(state)
}

pub extern "C" fn op_leave_handler(state: &mut DispatchState) -> Step {
    op_handler_marker(state)
}

#[inline]
fn op_handler_marker(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (_ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_load_exception(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let register = match u16::try_from(ax) {
        Ok(r) => r,
        Err(_) => {
            return Step::Error(VmError::RegisterOutOfBounds {
                code: state.frame.code(),
                register: 0,
            });
        }
    };
    let value = state.vm.current_exception_value();
    let registers = state.frame.registers();
    state.vm.write_register_unchecked(registers, register, value);
    state.advance(instruction_len);
    dispatch_next!(state);
}

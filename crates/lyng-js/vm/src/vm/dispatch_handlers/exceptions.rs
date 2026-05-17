//! Exception handlers for the trampoline dispatch path (lyng-59e6 round 3).
//!
//! Post-A17: each α handler in this file is a thin shim that
//!   1. decodes the instruction's operands,
//!   2. constructs `OpExceptionsXxxArgs` and calls into
//!      `crate::vm::semantics::exceptions::op_xxx_semantic`,
//!   3. translates the returned `SemanticOutcome` to `Step` via
//!      `translate_outcome_to_step`.
//!
//! Throw routing flows through
//! `SemanticOutcome::Refresh` (caught — same or cross-frame) or
//! `SemanticOutcome::ExitError { error: VmError::Abrupt(Throw(value)) }`
//! (uncaught). EnterHandler / LeaveHandler are dispatch markers that
//! just advance PC. LoadException reads `Vm::current_exception` and
//! writes it to a register.

use crate::dsl::slow_path::LlIntDispatchState;
use crate::try_step;
use crate::vm::dispatch::decode_ax_operands;
use crate::vm::dispatch_handlers::translate_outcome_to_step;
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::vm::semantics::exceptions;

#[inline]
fn ax_to_register(state: &DispatchState, ax: i32) -> Result<u16, Step> {
    u16::try_from(ax)
        .map_err(|_| Step::Error(exceptions::ax_register_out_of_bounds_error(state)))
}

pub extern "C" fn op_throw(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let register = match ax_to_register(state, ax) {
        Ok(r) => r,
        Err(step) => return step,
    };
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = exceptions::op_throw_semantic(
        &mut ll_state,
        exceptions::OpExceptionsAxArgs {
            register,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

pub extern "C" fn op_enter_handler(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (_ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = exceptions::op_enter_handler_semantic(
        &mut ll_state,
        exceptions::OpHandlerMarkerArgs { instruction_len },
    );
    translate_outcome_to_step(state, outcome)
}

pub extern "C" fn op_leave_handler(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (_ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = exceptions::op_leave_handler_semantic(
        &mut ll_state,
        exceptions::OpHandlerMarkerArgs { instruction_len },
    );
    translate_outcome_to_step(state, outcome)
}

pub extern "C" fn op_load_exception(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let register = match ax_to_register(state, ax) {
        Ok(r) => r,
        Err(step) => return step,
    };
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = exceptions::op_load_exception_semantic(
        &mut ll_state,
        exceptions::OpExceptionsAxArgs {
            register,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

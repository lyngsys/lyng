//! Control-flow handlers for the trampoline dispatch path (lyng-5zrf).
//!
//! Covers:
//! - `Jump` (Ax form, signed-i24 relative offset).
//! - `Jump8` (Ax8 form, signed-i8 relative offset).
//! - `LoopHeader` — marker plus tier-backedge + incremental-mark safepoint.
//! - `Return` (Ax form, returns the value in the operand-specified register).
//! - `ReturnUndefined` (Ax form, returns `Value::undefined()`).
//!
//! `Return` and `ReturnUndefined` use `finish_active_frame`, which routes
//! through `Vm::finish_frame`. For entry-frame returns (`Some(result)`), the
//! handler emits `Step::Done`; for nested returns, it refreshes the
//! `DispatchState` to the caller frame and continues dispatching.
//!
//! Conditional jumps (`JumpIfTrue`, `JumpIfFalse`, `JumpIfTrue8`,
//! `JumpIfFalse8`) land in follow-up commits.

use lyng_js_ops::read;
use lyng_js_types::Value;

use crate::error::VmError;
use crate::vm::dispatch::{
    advance_dispatch_frame, decode_abx8_operands, decode_abx_operands, decode_ax8_operands,
    decode_ax_operands, jump_dispatch_frame,
};
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::vm::Vm;
use crate::{dispatch_next, try_step};

pub extern "C" fn op_jump(state: &mut DispatchState) -> Step {
    let code = state.frame.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));

    if ax < 0 {
        state.vm.observe_tier_backedge_event(code);
        Vm::poll_incremental_mark_safepoint(state.agent);
    }
    try_step!(jump_dispatch_frame(
        &mut state.frame,
        instruction_len,
        ax,
    ));
    dispatch_next!(state);
}

pub extern "C" fn op_jump8(state: &mut DispatchState) -> Step {
    let code = state.frame.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax8_operands(state.current_bytes(), false, code, pc));

    if ax < 0 {
        state.vm.observe_tier_backedge_event(code);
        Vm::poll_incremental_mark_safepoint(state.agent);
    }
    try_step!(jump_dispatch_frame(
        &mut state.frame,
        instruction_len,
        ax,
    ));
    dispatch_next!(state);
}

pub extern "C" fn op_loop_header(state: &mut DispatchState) -> Step {
    let code = state.frame.code();
    let pc = state.frame.instruction_offset();
    let (_ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));

    state.vm.observe_tier_backedge_event(code);
    Vm::poll_incremental_mark_safepoint(state.agent);
    advance_dispatch_frame(&mut state.frame, instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_return(state: &mut DispatchState) -> Step {
    let code = state.frame.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, _instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let register = match u16::try_from(ax) {
        Ok(r) => r,
        Err(_) => {
            return Step::Error(VmError::RegisterOutOfBounds {
                code,
                register: 0,
            });
        }
    };
    let value = state.vm.read_register(state.frame.registers(), register);
    finish_return(state, value)
}

// =====================================================================
// Conditional jumps — JumpIfTrue / JumpIfFalse + 8-byte variants.
// =====================================================================

/// Shared body for `JumpIfTrue` / `JumpIfFalse` / their 8-byte variants.
/// `condition_register` reads the value, `delta` is the relative offset,
/// and `take_if_truthy` selects between the two opcode behaviors.
#[inline]
fn op_jump_if_impl(
    state: &mut DispatchState,
    condition_register: u16,
    delta: i32,
    instruction_len: u32,
    take_if_truthy: bool,
) -> Step {
    let condition = state.vm.read_register(state.frame.registers(), condition_register);
    let truthy_result = read::to_boolean_agent(state.agent, condition).map_err(VmError::Abrupt);
    let truthy = match try_step!(state.handle_dispatch_result(truthy_result)) {
        Some(t) => t,
        None => {
            // The abrupt completion was caught — handler PC was rewritten by
            // transfer_to_exception_handler. Resume dispatch at the new PC.
            dispatch_next!(state);
        }
    };
    let should_jump = if take_if_truthy { truthy } else { !truthy };
    if should_jump {
        if delta < 0 {
            Vm::poll_incremental_mark_safepoint(state.agent);
        }
        try_step!(jump_dispatch_frame(
            &mut state.frame,
            instruction_len,
            delta,
        ));
    } else {
        advance_dispatch_frame(&mut state.frame, instruction_len);
    }
    dispatch_next!(state);
}

pub extern "C" fn op_jump_if_true(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let prefix = state.prefix.take();
    let (a, bx, _feedback_slot, instruction_len) = try_step!(decode_abx_operands(
        state.current_bytes(),
        prefix,
        false,
        code,
        pc,
    ));
    let delta = i32::from_le_bytes(bx.to_le_bytes());
    op_jump_if_impl(state, a, delta, instruction_len, true)
}

pub extern "C" fn op_jump_if_false(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let prefix = state.prefix.take();
    let (a, bx, _feedback_slot, instruction_len) = try_step!(decode_abx_operands(
        state.current_bytes(),
        prefix,
        false,
        code,
        pc,
    ));
    let delta = i32::from_le_bytes(bx.to_le_bytes());
    op_jump_if_impl(state, a, delta, instruction_len, false)
}

pub extern "C" fn op_jump_if_true8(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (a, bx, _feedback_slot, instruction_len) =
        try_step!(decode_abx8_operands(state.current_bytes(), false, code, pc));
    let delta = i32::from(i8::from_le_bytes([bx.to_le_bytes()[0]]));
    op_jump_if_impl(state, a, delta, instruction_len, true)
}

pub extern "C" fn op_jump_if_false8(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (a, bx, _feedback_slot, instruction_len) =
        try_step!(decode_abx8_operands(state.current_bytes(), false, code, pc));
    let delta = i32::from(i8::from_le_bytes([bx.to_le_bytes()[0]]));
    op_jump_if_impl(state, a, delta, instruction_len, false)
}

pub extern "C" fn op_return_undefined(state: &mut DispatchState) -> Step {
    let code = state.frame.code();
    let pc = state.frame.instruction_offset();
    let (_ax, _feedback_slot, _instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    finish_return(state, Value::undefined())
}

/// Shared epilogue for `Return` / `ReturnUndefined`.
///
/// `Vm::finish_frame` returns `Some(result)` for the entry frame (script
/// completed) and `None` when a nested function returns to its caller.
/// In the latter case the trampoline must re-snapshot from the caller's
/// frame and continue dispatching.
fn finish_return(state: &mut DispatchState, value: Value) -> Step {
    state.sync_active_frame();
    state.pop_execution_context();
    match state.finish_active_frame(value) {
        Ok(Some(result)) => Step::Done(result),
        Ok(None) => {
            try_step!(state.refresh_from_active_frame());
            dispatch_next!(state);
        }
        Err(error) => Step::Error(error),
    }
}

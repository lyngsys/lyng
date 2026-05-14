//! Control-flow handlers for the trampoline dispatch path (lyng-5zrf).
//!
//! Covers:
//! - `Jump` (Ax form, signed-i24 relative offset).
//! - `Jump8` (Ax8 form, signed-i8 relative offset).
//! - `LoopHeader` — marker plus tier-backedge + incremental-mark safepoint.
//!
//! Conditional jumps (`JumpIfTrue`, `JumpIfFalse`, `JumpIfTrue8`,
//! `JumpIfFalse8`) and `Return` / `ReturnUndefined` land in follow-up
//! commits — the Return family requires frame-transition handling that
//! crosses into sub-6 (Calls).

use crate::vm::dispatch::{
    advance_dispatch_frame, decode_ax8_operands, decode_ax_operands, jump_dispatch_frame,
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

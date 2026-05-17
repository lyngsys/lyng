//! Control-flow handlers for the trampoline dispatch path (lyng-5zrf).
//!
//! Post-A10: each α handler in this file is a thin shim that
//!   1. decodes the instruction's operands,
//!   2. constructs `OpXxxArgs` and calls into
//!      `crate::vm::semantics::control_flow::op_xxx_semantic`,
//!   3. translates the returned `SemanticOutcome` to `Step` via
//!      `translate_outcome_to_step`.
//!
//! The PC arithmetic, tier-backedge bookkeeping, debug-poll safepoint, and
//! `finish_active_frame` routing all live in the semantic body. The α
//! handler only owns operand decode + (for `Return`) the `ax → u16`
//! range-check that surfaces as `VmError::RegisterOutOfBounds`.

use crate::dsl::slow_path::LlIntDispatchState;
use crate::error::VmError;
use crate::try_step;
use crate::vm::dispatch::{
    decode_abx8_operands, decode_abx_operands, decode_ax8_operands, decode_ax_operands,
};
use crate::vm::dispatch_handlers::translate_outcome_to_step;
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::vm::semantics::control_flow;

// =====================================================================
// Nop — Ax decode; no operands consumed.
// =====================================================================

pub extern "C" fn op_nop(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (_ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = control_flow::op_nop_semantic(
        &mut ll_state,
        control_flow::OpNopArgs { instruction_len },
    );
    translate_outcome_to_step(state, outcome)
}

// =====================================================================
// Unconditional jumps — Jump (Ax) / Jump8 (Ax8).
// =====================================================================

pub extern "C" fn op_jump(state: &mut DispatchState) -> Step {
    let code = state.frame.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = control_flow::op_jump_semantic(
        &mut ll_state,
        control_flow::OpJumpArgs {
            delta: ax,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

pub extern "C" fn op_jump8(state: &mut DispatchState) -> Step {
    let code = state.frame.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax8_operands(state.current_bytes(), false, code, pc));
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = control_flow::op_jump8_semantic(
        &mut ll_state,
        control_flow::OpJumpArgs {
            delta: ax,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

// =====================================================================
// Conditional jumps — JumpIfTrue / JumpIfFalse (+ *8 variants).
//
// Wide-prefix variants consume `state.prefix.take()` before decode; the
// 8-byte variants are narrow-only and don't consult the prefix.
// =====================================================================

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
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = control_flow::op_jump_if_true_semantic(
        &mut ll_state,
        control_flow::OpJumpIfArgs {
            condition_register: a,
            delta,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
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
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = control_flow::op_jump_if_false_semantic(
        &mut ll_state,
        control_flow::OpJumpIfArgs {
            condition_register: a,
            delta,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

pub extern "C" fn op_jump_if_true8(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (a, bx, _feedback_slot, instruction_len) =
        try_step!(decode_abx8_operands(state.current_bytes(), false, code, pc));
    let delta = i32::from(i8::from_le_bytes([bx.to_le_bytes()[0]]));
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = control_flow::op_jump_if_true8_semantic(
        &mut ll_state,
        control_flow::OpJumpIfArgs {
            condition_register: a,
            delta,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

pub extern "C" fn op_jump_if_false8(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (a, bx, _feedback_slot, instruction_len) =
        try_step!(decode_abx8_operands(state.current_bytes(), false, code, pc));
    let delta = i32::from(i8::from_le_bytes([bx.to_le_bytes()[0]]));
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = control_flow::op_jump_if_false8_semantic(
        &mut ll_state,
        control_flow::OpJumpIfArgs {
            condition_register: a,
            delta,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

// =====================================================================
// LoopHeader — Ax decode; semantic body runs the tier-backedge event,
// the incremental-mark safepoint, and (when the debug hook is installed)
// the debug-poll safepoint. The `ax` operand is unused.
// =====================================================================

pub extern "C" fn op_loop_header(state: &mut DispatchState) -> Step {
    let code = state.frame.code();
    let pc = state.frame.instruction_offset();
    let (_ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = control_flow::op_loop_header_semantic(
        &mut ll_state,
        control_flow::OpLoopHeaderArgs { instruction_len },
    );
    translate_outcome_to_step(state, outcome)
}

// =====================================================================
// Return / ReturnUndefined — Ax decode. The α handler is responsible for
// the `ax → u16` range-check that surfaces as `RegisterOutOfBounds`; the
// semantic body assumes the register index is already validated.
// =====================================================================

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
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = control_flow::op_return_semantic(
        &mut ll_state,
        control_flow::OpReturnArgs { register },
    );
    translate_outcome_to_step(state, outcome)
}

pub extern "C" fn op_return_undefined(state: &mut DispatchState) -> Step {
    let code = state.frame.code();
    let pc = state.frame.instruction_offset();
    let (_ax, _feedback_slot, _instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = control_flow::op_return_undefined_semantic(
        &mut ll_state,
        control_flow::OpReturnUndefinedArgs,
    );
    translate_outcome_to_step(state, outcome)
}

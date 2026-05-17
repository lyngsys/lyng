//! Call-family handlers for the trampoline dispatch path (lyng-1fie).
//!
//! Post-A14: each α handler in this file is a thin shim that
//!   1. decodes the instruction's operands,
//!   2. constructs `OpXxxArgs` and calls into
//!      `crate::vm::semantics::calls::op_xxx_semantic`,
//!   3. translates the returned `SemanticOutcome` to `Step` via
//!      `translate_outcome_to_step`.
//!
//! Frame transitions (push callee, sync caller, refresh after return) live
//! inside the `Vm::call_value*` / `tail_call_value` / `construct_value`
//! helpers — the semantic body just routes their outcome via
//! `handle_dispatch_result` and returns `SemanticOutcome::Refresh` so the
//! translator re-snapshots PC/REGS/FV from the now-active frame.
//!
//! `TailCall` is special: the helper returns `VmResult<Option<Value>>`
//! where `Some(value)` means the script's entry frame just unwound and the
//! semantic body returns `SemanticOutcome::ExitDone`. `record_feedback_slot`
//! runs only after the helper returns a non-error result — matching the
//! pre-A14 α body's ordering.
//!
//! `CreateClosure` doesn't transfer frames; it allocates a function
//! object and writes it to a register, then returns
//! `SemanticOutcome::Continue { pc_advance: instruction_len }`.

use lyng_js_bytecode::Opcode;

use crate::dsl::slow_path::LlIntDispatchState;
use crate::try_step;
use crate::vm::dispatch::{decode_abc_operands, decode_abx_operands, decode_call_range_operands};
use crate::vm::dispatch_handlers::translate_outcome_to_step;
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::vm::semantics::calls;

// =====================================================================
// Call0..3 — fixed-arity calls via decode_abc_operands.
// =====================================================================

#[inline]
fn op_call_small_handler(
    state: &mut DispatchState,
    arity: u8,
    semantic: fn(
        &mut LlIntDispatchState<'_, '_>,
        calls::OpCallSmallArgs,
    ) -> crate::dsl::slow_path::SemanticOutcome,
) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let prefix = state.prefix.take();
    let (a, b, c, feedback_slot, instruction_len) = try_step!(decode_abc_operands(
        state.current_bytes(),
        prefix,
        true,
        code,
        pc,
    ));
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = semantic(
        &mut ll_state,
        calls::OpCallSmallArgs {
            a,
            b,
            c,
            arity,
            feedback_slot,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

pub extern "C" fn op_call0(state: &mut DispatchState) -> Step {
    op_call_small_handler(state, 0, calls::op_call0_semantic)
}

pub extern "C" fn op_call1(state: &mut DispatchState) -> Step {
    op_call_small_handler(state, 1, calls::op_call1_semantic)
}

pub extern "C" fn op_call2(state: &mut DispatchState) -> Step {
    op_call_small_handler(state, 2, calls::op_call2_semantic)
}

pub extern "C" fn op_call3(state: &mut DispatchState) -> Step {
    op_call_small_handler(state, 3, calls::op_call3_semantic)
}

// =====================================================================
// Call / Construct — variable-arity via decode_call_range_operands.
// =====================================================================

#[inline]
fn op_call_range_handler(
    state: &mut DispatchState,
    opcode: Opcode,
    semantic: fn(
        &mut LlIntDispatchState<'_, '_>,
        calls::OpCallRangeArgs,
    ) -> crate::dsl::slow_path::SemanticOutcome,
) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (a, b, c, call_range, feedback_slot, instruction_len) = try_step!(
        decode_call_range_operands(state.current_bytes(), true, code, pc,)
    );
    let range = try_step!(calls::require_call_range_semantic(state, call_range, opcode));
    let spread_mask = calls::spread_mask_for_semantic(state, feedback_slot);
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = semantic(
        &mut ll_state,
        calls::OpCallRangeArgs {
            a,
            b,
            c,
            range,
            spread_mask,
            feedback_slot,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

pub extern "C" fn op_call(state: &mut DispatchState) -> Step {
    op_call_range_handler(state, Opcode::Call, calls::op_call_semantic)
}

pub extern "C" fn op_construct(state: &mut DispatchState) -> Step {
    op_call_range_handler(state, Opcode::Construct, calls::op_construct_semantic)
}

// =====================================================================
// TailCall — variable-arity via decode_call_range_operands; replaces or
// unwinds the active frame.
// =====================================================================

pub extern "C" fn op_tail_call(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (a, b, _c, call_range, feedback_slot, _instruction_len) = try_step!(
        decode_call_range_operands(state.current_bytes(), true, code, pc,)
    );
    let range = try_step!(calls::require_call_range_semantic(
        state,
        call_range,
        Opcode::TailCall,
    ));
    let spread_mask = calls::spread_mask_for_semantic(state, feedback_slot);
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = calls::op_tail_call_semantic(
        &mut ll_state,
        calls::OpTailCallArgs {
            a,
            b,
            range,
            spread_mask,
            feedback_slot,
        },
    );
    translate_outcome_to_step(state, outcome)
}

// =====================================================================
// CreateClosure — Abx layout; no frame transition.
// =====================================================================

pub extern "C" fn op_create_closure(state: &mut DispatchState) -> Step {
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
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = calls::op_create_closure_semantic(
        &mut ll_state,
        calls::OpCreateClosureArgs {
            a,
            bx,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

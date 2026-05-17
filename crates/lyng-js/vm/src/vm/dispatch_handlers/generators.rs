//! Generator + async handlers for the trampoline dispatch path
//! (lyng-59e6 round 4). All `#[cold]` — rare opcodes per JSC's hot/cold
//! analysis.
//!
//! Post-A16: each α handler in this file is a thin shim that
//!   1. decodes the instruction's operands,
//!   2. constructs `OpGeneratorsXxxArgs` and calls into
//!      `crate::vm::semantics::generators::op_xxx_semantic`,
//!   3. translates the returned `SemanticOutcome` to `Step` via
//!      `translate_outcome_to_step`.
//!
//! Suspension routing — `Yield`, `Await`, `SuspendGeneratorStart`,
//! `DelegateYield` (Suspend outcome) — flows through
//! `SemanticOutcome::ExitError { error }` where `error` carries the
//! special suspension variant (`VmError::GeneratorYield`,
//! `VmError::GeneratorStart`, `VmError::AsyncSuspend`). The caller of
//! `Vm::run` distinguishes those from genuine abrupt completions per the
//! existing trampoline contract.
//!
//! `LoadResumeKind` / `LoadResumeValue` are register-only opcodes; they
//! read the per-frame resume slot populated by
//! `Vm::restore_suspended_execution` at resume entry and produce a
//! `SemanticOutcome::Continue { pc_advance: instruction_len }` outcome.

use crate::dsl::slow_path::LlIntDispatchState;
use crate::try_step;
use crate::vm::dispatch::{decode_abc_operands, decode_ax_operands};
use crate::vm::dispatch_handlers::translate_outcome_to_step;
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::vm::semantics::generators;

#[inline]
fn ax_to_register(state: &DispatchState, ax: i32) -> Result<u16, Step> {
    u16::try_from(ax)
        .map_err(|_| Step::Error(generators::ax_register_out_of_bounds_error(state)))
}

#[cold]
pub extern "C" fn op_suspend_generator_start(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (_ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = generators::op_suspend_generator_start_semantic(
        &mut ll_state,
        generators::OpSuspendGeneratorStartArgs { instruction_len },
    );
    translate_outcome_to_step(state, outcome)
}

#[cold]
pub extern "C" fn op_yield(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let register = match ax_to_register(state, ax) {
        Ok(r) => r,
        Err(step) => return step,
    };
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = generators::op_yield_semantic(
        &mut ll_state,
        generators::OpGeneratorsAxArgs {
            register,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

#[cold]
pub extern "C" fn op_delegate_yield(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let prefix = state.prefix.take();
    let (a, b, c, _feedback_slot, instruction_len) = try_step!(decode_abc_operands(
        state.current_bytes(),
        prefix,
        false,
        code,
        pc,
    ));
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = generators::op_delegate_yield_semantic(
        &mut ll_state,
        generators::OpDelegateYieldArgs {
            a,
            b,
            c,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

#[cold]
pub extern "C" fn op_await(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let register = match ax_to_register(state, ax) {
        Ok(r) => r,
        Err(step) => return step,
    };
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = generators::op_await_semantic(
        &mut ll_state,
        generators::OpGeneratorsAxArgs {
            register,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

pub extern "C" fn op_load_resume_kind(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let register = match ax_to_register(state, ax) {
        Ok(r) => r,
        Err(step) => return step,
    };
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = generators::op_load_resume_kind_semantic(
        &mut ll_state,
        generators::OpGeneratorsAxArgs {
            register,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

pub extern "C" fn op_load_resume_value(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let register = match ax_to_register(state, ax) {
        Ok(r) => r,
        Err(step) => return step,
    };
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = generators::op_load_resume_value_semantic(
        &mut ll_state,
        generators::OpGeneratorsAxArgs {
            register,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

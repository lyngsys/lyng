//! Prefix family semantic bodies.
//!
//! `Wide` / `ExtraWide` widen the operand encoding of the next opcode.
//! They set `state.prefix` and return `Continue { pc_advance: 0 }` so the
//! next opcode handler runs with the prefix set. Double-prefix is rejected
//! with `VmError::DoublePrefix`.
//!
//! Wide-form dispatch in production is driven by
//! `crate::vm::dispatch::run_wide_form_instruction`; these bodies are
//! retained for the DSL validation harness and double-prefix rejection.

use lyng_bytecode::Opcode;

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::error::VmError;

/// Operand shape for prefix opcodes (no operands; exists for uniform signature).
pub struct OpPrefixArgs;

/// Returns a `DoublePrefix` error. A stacked prefix indicates a corrupted
/// instruction stream — the emitter never produces `Wide; Wide; ...`.
#[inline]
const fn double_prefix_error(state: &mut LlIntDispatchState<'_, '_>) -> VmError {
    let inner = state.dispatch_state();
    VmError::DoublePrefix {
        code: inner.code(),
        instruction_offset: inner.pc(),
    }
}

// =====================================================================
// Wide
// =====================================================================

pub const fn op_wide_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    _args: OpPrefixArgs,
) -> SemanticOutcome {
    if state.dispatch_state().prefix.is_some() {
        return SemanticOutcome::ExitError {
            error: double_prefix_error(state),
        };
    }
    state.dispatch_state().prefix = Some(Opcode::Wide);
    SemanticOutcome::Continue { pc_advance: 0 }
}

// =====================================================================
// ExtraWide
// =====================================================================

pub const fn op_extra_wide_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    _args: OpPrefixArgs,
) -> SemanticOutcome {
    if state.dispatch_state().prefix.is_some() {
        return SemanticOutcome::ExitError {
            error: double_prefix_error(state),
        };
    }
    state.dispatch_state().prefix = Some(Opcode::ExtraWide);
    SemanticOutcome::Continue { pc_advance: 0 }
}

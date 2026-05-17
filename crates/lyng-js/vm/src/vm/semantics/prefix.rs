//! Prefix family semantic bodies (DSL-0a Task A18).
//!
//! The `Wide` / `ExtraWide` prefix opcodes widen the operand encoding of
//! the *next* opcode. They are unusual among DSL-0a opcodes: they have
//! no operands of their own, do not advance PC, and instead set
//! `state.prefix` for the semantic-opcode handler that runs immediately
//! after. The semantic handler consumes the prefix via
//! `state.prefix.take()` and uses the widened decoder shape.
//!
//! Because the prefix carries no operands and "dispatch tail" semantics
//! (run the next byte's handler with the same PC, not pc+1), the α
//! handler in `dispatch_handlers/prefix.rs` does not route through
//! `translate_outcome_to_step` — instead it inspects the
//! `SemanticOutcome` directly and performs a same-PC dispatch on
//! `Continue { pc_advance: 0 }`. The semantic body's only jobs are:
//!   1. Reject a stacked prefix (`state.prefix.is_some()`) by returning
//!      `ExitError { error: VmError::DoublePrefix }`.
//!   2. Record `state.prefix = Some(opcode)` and return
//!      `Continue { pc_advance: 0 }` to signal "α handler, do the
//!      same-PC dispatch tail".
//!
//! The DSL-0b cold-stub shim in `dsl/handlers/cold/prefix.rs` will reach
//! the same functions from the asm-DSL path, where the asm-side dispatch
//! macro performs the same "no PC advance, peek bytes[pc+1]" tail.

use lyng_js_bytecode::Opcode;

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::error::VmError;

/// Operand shape for both prefix opcodes. The prefix has no operands;
/// the struct exists only so the family-extraction signature shape is
/// uniform across families.
pub struct OpPrefixArgs;

/// Reject a stacked prefix. Mirrors the α handler's double-prefix
/// rejection: the bytecode emitter never produces `Wide; Wide; ...` or
/// `Wide; ExtraWide; ...`, so encountering one indicates a corrupted
/// instruction stream rather than a valid program.
#[inline]
fn double_prefix_error(state: &mut LlIntDispatchState<'_, '_>) -> VmError {
    let inner = state.dispatch_state();
    VmError::DoublePrefix {
        code: inner.frame.code(),
        instruction_offset: inner.frame.instruction_offset(),
    }
}

// =====================================================================
// Wide — record `state.prefix = Some(Opcode::Wide)` and return
// `Continue { pc_advance: 0 }`. The α handler (or DSL-0b cold-stub
// shim) then dispatches to `bytes[pc+1]`'s handler without advancing
// PC, so the widened decoder reads from the prefix byte.
// =====================================================================

pub(crate) fn op_wide_semantic(
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
// ExtraWide — same shape as `Wide`, but the widened decoder reads four
// bytes per `Bx` operand (vs. three for `Wide`); see
// `decode_abx_operands_wide` for the encoding split.
// =====================================================================

pub(crate) fn op_extra_wide_semantic(
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

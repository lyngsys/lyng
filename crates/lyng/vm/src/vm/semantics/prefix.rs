//! Prefix family semantic bodies (DSL-0a Task A18).
//!
//! The `Wide` / `ExtraWide` prefix opcodes widen the operand encoding of
//! the *next* opcode. They are unusual: they have no operands of their
//! own, do not advance PC, and instead set `state.prefix` for the
//! semantic-opcode handler that runs immediately after. The semantic
//! handler consumes the prefix via `state.prefix.take()` and uses the
//! widened decoder shape.
//!
//! Pre-DSL-0c (α path): the α prefix handler in
//! `dispatch_handlers/prefix.rs` set `state.prefix`, then performed a
//! same-PC dispatch to `DISPATCH_TABLE[bytes[pc+1]]`. The next
//! semantic handler ran with PC still at the prefix byte; widened
//! decoders read operands from `bytes[2..]`.
//!
//! Post-DSL-0c (α deletion): the asm-DSL `op_wide` / `op_extra_wide`
//! shims drive wide-form dispatch entirely through
//! `crate::vm::dispatch::run_wide_form_instruction`, which decodes the
//! wide instruction in Rust, calls the matching semantic body, and
//! returns the full instruction length so the asm trampoline advances
//! past the entire wide-form instruction. The semantic body below is
//! retained for the DSL validation harness (double-prefix rejection)
//! and conceptually documents the "set prefix, do not advance" contract
//! even though no production path now reaches it.
//!
//! The semantic body's two jobs:
//!   1. Reject a stacked prefix (`state.prefix.is_some()`) by returning
//!      `ExitError { error: VmError::DoublePrefix }`.
//!   2. Record `state.prefix = Some(opcode)` and return
//!      `Continue { pc_advance: 0 }` (same-PC dispatch tail; the
//!      α/DSL caller is responsible for picking the semantic byte).

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
// `Continue { pc_advance: 0 }`. The caller (DSL `op_wide` shim or α
// handler) is responsible for picking the semantic byte and decoding
// wide-form operands; this body's role is purely the prefix bit-flip
// + double-prefix guard.
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

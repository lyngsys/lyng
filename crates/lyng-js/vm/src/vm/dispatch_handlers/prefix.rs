//! Wide / ExtraWide prefix handlers (lyng-59e6 round 3, DSL-0a Task A18).
//!
//! When the bytecode emitter needs to express an operand wider than its
//! narrow form can fit (registers ≥ 256 or jump deltas ≥ ±128 / signed-i24),
//! it prepends a `Wide` or `ExtraWide` byte. In the trampoline dispatch path
//! the prefix is its own opcode: `op_wide` / `op_extra_wide` set
//! `state.prefix` and re-dispatch (without advancing PC) to the semantic
//! opcode handler at `bytes[pc+1]`. Each semantic handler consumes the
//! prefix via `state.prefix.take()` so the next dispatch starts fresh.
//!
//! Post-A18 shape: the α handler is a thin shim that builds an
//! `LlIntDispatchState` and calls into
//! `crate::vm::semantics::prefix::op_xxx_semantic`, which records the
//! prefix on `state` (or returns `ExitError { VmError::DoublePrefix }`
//! if a prefix is already pending). Because the prefix has "same-PC,
//! next-byte dispatch" semantics — distinct from the standard
//! "advance-then-read" tail used by every other opcode — the α handler
//! does NOT route through `translate_outcome_to_step`. Instead it
//! inspects the `SemanticOutcome` directly:
//!   - `Continue { pc_advance: 0 }` → dispatch to `DISPATCH_TABLE[bytes[1]]`.
//!   - `ExitError { error }`        → `Step::Error(error)`.
//!   - `Continue { pc_advance: _ }` / `Refresh` / `ExitDone` → unreachable
//!     for the prefix bodies; treated as a logic error in debug.

use lyng_js_bytecode::Opcode;

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::error::VmError;
use crate::vm::dispatch_state::{DispatchState, Step, DISPATCH_TABLE};
use crate::vm::semantics::prefix;

#[inline]
fn dispatch_prefixed(state: &mut DispatchState<'_>, prefix_opcode: Opcode) -> Step {
    // Read the semantic byte at bytes[pc+1] before handing off to the
    // semantic body. If the instruction stream is truncated, surface
    // the same error the pre-A18 handler did.
    let semantic_byte = match state.current_bytes().get(1).copied() {
        Some(b) => b,
        None => {
            return Step::Error(VmError::InstructionOutOfBounds {
                code: state.frame.code(),
                instruction_offset: state.frame.instruction_offset(),
            });
        }
    };

    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = match prefix_opcode {
        Opcode::Wide => prefix::op_wide_semantic(&mut ll_state, prefix::OpPrefixArgs),
        Opcode::ExtraWide => prefix::op_extra_wide_semantic(&mut ll_state, prefix::OpPrefixArgs),
        // Unreachable: only Wide / ExtraWide reach this helper.
        _ => unreachable!("dispatch_prefixed called with non-prefix opcode {:?}", prefix_opcode),
    };

    match outcome {
        SemanticOutcome::Continue { pc_advance: 0 } => {
            Step::Continue(DISPATCH_TABLE[semantic_byte as usize])
        }
        SemanticOutcome::ExitError { error } => Step::Error(error),
        // The prefix semantics never produce Refresh / ExitDone / a
        // non-zero PC advance. If one shows up here the semantic body
        // has drifted out of sync with the α handler — assert in debug.
        other => {
            debug_assert!(
                false,
                "prefix semantic returned unexpected outcome variant: {}",
                match other {
                    SemanticOutcome::Continue { .. } => "Continue { pc_advance != 0 }",
                    SemanticOutcome::Refresh => "Refresh",
                    SemanticOutcome::ExitDone { .. } => "ExitDone",
                    SemanticOutcome::ExitError { .. } => unreachable!(),
                },
            );
            Step::Error(VmError::InstructionOutOfBounds {
                code: state.frame.code(),
                instruction_offset: state.frame.instruction_offset(),
            })
        }
    }
}

pub extern "C" fn op_wide(state: &mut DispatchState) -> Step {
    dispatch_prefixed(state, Opcode::Wide)
}

pub extern "C" fn op_extra_wide(state: &mut DispatchState) -> Step {
    dispatch_prefixed(state, Opcode::ExtraWide)
}

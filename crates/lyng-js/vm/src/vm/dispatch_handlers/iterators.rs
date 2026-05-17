//! Iterator + for-in handlers for the trampoline dispatch path
//! (lyng-59e6 round 2).
//!
//! Post-A15: each α handler in this file is a thin shim that
//!   1. decodes the instruction's operands,
//!   2. constructs `OpIteratorAbcArgs` / `OpIteratorAbxArgs` and calls
//!      into `crate::vm::semantics::iterators::op_xxx_semantic`,
//!   3. translates the returned `SemanticOutcome` to `Step` via
//!      `translate_outcome_to_step`.
//!
//! The iterator-protocol helpers (enumerator construction, advance,
//! close, return-method invocation) live in `crate::vm::loop_iteration`
//! and are reached through thin `Vm`-level wrappers
//! (`for_in_*` / `iterator_*` / `create_*_for_value` /
//! `advance_iterator_state` / `close_iterator_state`). The α handler owns
//! operand decode only; side-table mutation and `handle_dispatch_result`
//! routing live in the semantic body.

use crate::dsl::slow_path::LlIntDispatchState;
use crate::try_step;
use crate::vm::dispatch::{decode_abc_operands, decode_abx_operands};
use crate::vm::dispatch_handlers::translate_outcome_to_step;
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::vm::semantics::iterators;

// =====================================================================
// Abc-form iterator opcodes — CreateForIn, AdvanceForIn, CreateIterator,
// AdvanceIterator. All share the same unprofiled Abc decode; the
// semantic body interprets each operand per opcode.
// =====================================================================

macro_rules! op_iterator_abc_handler {
    ($name:ident, $semantic:path) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
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
            let outcome = $semantic(
                &mut ll_state,
                iterators::OpIteratorAbcArgs {
                    a,
                    b,
                    c,
                    instruction_len,
                },
            );
            translate_outcome_to_step(state, outcome)
        }
    };
}

op_iterator_abc_handler!(op_create_for_in, iterators::op_create_for_in_semantic);
op_iterator_abc_handler!(op_advance_for_in, iterators::op_advance_for_in_semantic);
op_iterator_abc_handler!(op_create_iterator, iterators::op_create_iterator_semantic);
op_iterator_abc_handler!(op_advance_iterator, iterators::op_advance_iterator_semantic);

// =====================================================================
// Abx-form iterator opcodes — CloseForIn, CloseIterator. Both share the
// same unprofiled Abx decode; `bx` is unused for `CloseForIn` and
// signals an already-pending abrupt completion for `CloseIterator`.
// =====================================================================

macro_rules! op_iterator_abx_handler {
    ($name:ident, $semantic:path) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
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
            let outcome = $semantic(
                &mut ll_state,
                iterators::OpIteratorAbxArgs {
                    a,
                    bx,
                    instruction_len,
                },
            );
            translate_outcome_to_step(state, outcome)
        }
    };
}

op_iterator_abx_handler!(op_close_for_in, iterators::op_close_for_in_semantic);
op_iterator_abx_handler!(op_close_iterator, iterators::op_close_iterator_semantic);

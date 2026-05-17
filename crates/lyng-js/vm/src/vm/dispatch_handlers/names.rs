//! Global + name resolution handlers for the trampoline dispatch path
//! (lyng-5mqv).
//!
//! Post-A12: each α handler in this file is a thin shim that
//!   1. decodes the instruction's operands,
//!   2. constructs `OpAtomArgs` / `OpCapturedNameArgs` and calls into
//!      `crate::vm::semantics::names::op_xxx_semantic`,
//!   3. translates the returned `SemanticOutcome` to `Step` via
//!      `translate_outcome_to_step`.
//!
//! All Abx-encoded; operand `bx` is either an atom-constant-pool index
//! (globals, names, `CaptureName`) or a captured-name reference register
//! index (`LoadCapturedName`, `LoadCapturedNameThis`,
//! `AssignCapturedName`). The semantic body interprets it accordingly.
//!
//! Also hosts LoadThis / LoadCallee / LoadNewTarget — they read frame
//! state directly without an atom operand, but they live in the
//! "name & global" family from the spec's perspective.
//!
//! The IC fast-path/slow-path layout for the globals-with-feedback opcodes
//! is unchanged — DSL-0a's job is only to lift the call site out of the α
//! handler. The Phase 3 IC machinery still lives in
//! `Vm::*_with_feedback` in `vm/names.rs`; DSL-1 lands the IC mode-byte
//! refactor and DSL-0b the flat-array refactor.

use crate::dsl::slow_path::LlIntDispatchState;
use crate::vm::dispatch::decode_abx_operands;
use crate::vm::dispatch_handlers::translate_outcome_to_step;
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::vm::semantics::names;
use crate::try_step;

// ---- Globals (with feedback) ----

macro_rules! op_names_profiled_handler {
    ($name:ident, $semantic:path) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
            let code = state.code();
            let pc = state.frame.instruction_offset();
            let prefix = state.prefix.take();
            let (a, bx, feedback_slot, instruction_len) = try_step!(decode_abx_operands(
                state.current_bytes(),
                prefix,
                true,
                code,
                pc,
            ));
            let mut ll_state = LlIntDispatchState::from_alpha(state);
            let outcome = $semantic(
                &mut ll_state,
                names::OpAtomArgs {
                    a,
                    bx,
                    instruction_len,
                    feedback_slot,
                },
            );
            translate_outcome_to_step(state, outcome)
        }
    };
}

op_names_profiled_handler!(op_load_global, names::op_load_global_semantic);
op_names_profiled_handler!(op_store_global, names::op_store_global_semantic);
op_names_profiled_handler!(op_assign_global, names::op_assign_global_semantic);

// ---- Names (lexical scope walk) and DeleteGlobal — Abx without feedback ----
//
// These share the unprofiled Abx decode (the α form passes `is_profiled =
// false` to `decode_abx_operands`). LoadThis / LoadCallee / LoadNewTarget
// also use this decode but discard `bx`; they reuse the same handler
// macro and ignore `args.bx` in their semantic body.

macro_rules! op_names_atom_handler {
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
                names::OpAtomArgs {
                    a,
                    bx,
                    instruction_len,
                    feedback_slot: None,
                },
            );
            translate_outcome_to_step(state, outcome)
        }
    };
}

op_names_atom_handler!(op_delete_global, names::op_delete_global_semantic);
op_names_atom_handler!(op_load_name, names::op_load_name_semantic);
op_names_atom_handler!(op_resolve_name, names::op_resolve_name_semantic);
op_names_atom_handler!(op_resolve_global, names::op_resolve_global_semantic);
op_names_atom_handler!(op_assign_name, names::op_assign_name_semantic);
op_names_atom_handler!(
    op_assign_variable_name,
    names::op_assign_variable_name_semantic
);
op_names_atom_handler!(op_delete_name, names::op_delete_name_semantic);
op_names_atom_handler!(op_capture_name, names::op_capture_name_semantic);

// ---- Frame-state loads: This / Callee / NewTarget ----
//
// Same Abx-without-feedback decode; `bx` is ignored. We pipe it through
// the same handler macro for uniformity.

op_names_atom_handler!(op_load_this, names::op_load_this_semantic);
op_names_atom_handler!(op_load_callee, names::op_load_callee_semantic);
op_names_atom_handler!(op_load_new_target, names::op_load_new_target_semantic);

// ---- Captured names (closures) ----
//
// `bx` is a captured-name reference register index (bounds-checked in the
// semantic body). `CaptureName` instead reads `bx` as an atom-constant
// index, so it uses `op_names_atom_handler` above.

macro_rules! op_captured_name_handler {
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
                names::OpCapturedNameArgs {
                    a,
                    bx,
                    instruction_len,
                },
            );
            translate_outcome_to_step(state, outcome)
        }
    };
}

op_captured_name_handler!(
    op_load_captured_name,
    names::op_load_captured_name_semantic
);
op_captured_name_handler!(
    op_load_captured_name_this,
    names::op_load_captured_name_this_semantic
);
op_captured_name_handler!(
    op_assign_captured_name,
    names::op_assign_captured_name_semantic
);

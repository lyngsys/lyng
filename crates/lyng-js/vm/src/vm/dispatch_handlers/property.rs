//! Property access handlers for the trampoline dispatch path (lyng-5mqv).
//!
//! Post-A11: each α handler in this file is a thin shim that
//!   1. decodes the instruction's operands,
//!   2. constructs `OpXxxArgs` and calls into
//!      `crate::vm::semantics::property::op_xxx_semantic`,
//!   3. translates the returned `SemanticOutcome` to `Step` via
//!      `translate_outcome_to_step`.
//!
//! The IC fast-path/slow-path layout is unchanged — DSL-0a's job is only
//! to lift the call site out of the α handler. The Phase 3a/3e/3f IC
//! machinery still lives in `Vm::execute_*_opcode` in
//! `vm/dispatch/property.rs`; DSL-1 lands the IC mode-byte refactor and
//! DSL-0b the flat-array refactor.
//!
//! The Set/Assign variants split into separate semantic functions
//! (`op_set_named_property_semantic`, `op_assign_named_property_semantic`,
//! `op_strict_assign_named_property_semantic`, and their keyed twins)
//! that internally thread the appropriate `Opcode` into the shared helper
//! — preserving the existing strict-mode / assignment / property-define
//! semantics fan-out without exposing it through the operand struct.

use crate::dsl::slow_path::LlIntDispatchState;
use crate::vm::dispatch::{decode_abc_operands, decode_abx_operands};
use crate::vm::dispatch_handlers::translate_outcome_to_step;
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::vm::semantics::property;
use crate::try_step;

// =====================================================================
// Named property reads — `GetNamedProperty`.
// =====================================================================

pub extern "C" fn op_get_named_property(state: &mut DispatchState) -> Step {
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
    let outcome = property::op_get_named_property_semantic(
        &mut ll_state,
        property::OpPropertyAccessArgs {
            a,
            b,
            c,
            feedback_slot,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

// =====================================================================
// Named property writes — `SetNamedProperty`, `AssignNamedProperty`,
// `StrictAssignNamedProperty`. All three share the Abc operand decode
// (with feedback slot); the semantic body picks the strict-mode /
// assignment / property-define variant.
// =====================================================================

macro_rules! op_property_set_handler {
    ($name:ident, $semantic:path) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
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
            let outcome = $semantic(
                &mut ll_state,
                property::OpPropertyAccessArgs {
                    a,
                    b,
                    c,
                    feedback_slot,
                    instruction_len,
                },
            );
            translate_outcome_to_step(state, outcome)
        }
    };
}

op_property_set_handler!(op_set_named_property, property::op_set_named_property_semantic);
op_property_set_handler!(
    op_assign_named_property,
    property::op_assign_named_property_semantic
);
op_property_set_handler!(
    op_strict_assign_named_property,
    property::op_strict_assign_named_property_semantic
);

// =====================================================================
// Keyed property reads + writes — `GetKeyedProperty`, `SetKeyedProperty`,
// `AssignKeyedProperty`, `StrictAssignKeyedProperty`. Same Abc-with-
// feedback decode as the named family.
// =====================================================================

op_property_set_handler!(op_get_keyed_property, property::op_get_keyed_property_semantic);
op_property_set_handler!(op_set_keyed_property, property::op_set_keyed_property_semantic);
op_property_set_handler!(
    op_assign_keyed_property,
    property::op_assign_keyed_property_semantic
);
op_property_set_handler!(
    op_strict_assign_keyed_property,
    property::op_strict_assign_keyed_property_semantic
);

// =====================================================================
// Define-data + misc Abc-without-feedback opcodes —
// `DefineNamedProperty`, `DefineKeyedProperty`, `DeleteProperty`, `In`,
// `CopyDataProperties`, `StoreDenseElement`, `LoadDenseElement`. All
// share Abc operand decode with `is_profiled = false`.
// =====================================================================

macro_rules! op_property_abc_handler {
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
                property::OpPropertyAbcArgs {
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

op_property_abc_handler!(
    op_define_named_property,
    property::op_define_named_property_semantic
);
op_property_abc_handler!(
    op_define_keyed_property,
    property::op_define_keyed_property_semantic
);
op_property_abc_handler!(op_delete_property, property::op_delete_property_semantic);
op_property_abc_handler!(op_in, property::op_in_semantic);
op_property_abc_handler!(
    op_copy_data_properties,
    property::op_copy_data_properties_semantic
);
op_property_abc_handler!(
    op_store_dense_element,
    property::op_store_dense_element_semantic
);
op_property_abc_handler!(
    op_load_dense_element,
    property::op_load_dense_element_semantic
);

// =====================================================================
// `ToPropertyKey` and `SetFunctionName` — Abc decode, `c` operand unused.
// =====================================================================

macro_rules! op_property_ab_handler {
    ($name:ident, $semantic:path) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
            let code = state.code();
            let pc = state.frame.instruction_offset();
            let prefix = state.prefix.take();
            let (a, b, _c, _feedback_slot, instruction_len) = try_step!(decode_abc_operands(
                state.current_bytes(),
                prefix,
                false,
                code,
                pc,
            ));
            let mut ll_state = LlIntDispatchState::from_alpha(state);
            let outcome = $semantic(
                &mut ll_state,
                property::OpPropertyAbArgs {
                    a,
                    b,
                    instruction_len,
                },
            );
            translate_outcome_to_step(state, outcome)
        }
    };
}

op_property_ab_handler!(op_to_property_key, property::op_to_property_key_semantic);
op_property_ab_handler!(op_set_function_name, property::op_set_function_name_semantic);

// =====================================================================
// `CreateObject`, `CreateArray`, `CheckObjectCoercible`,
// `ThrowIfUninitialized` — Abx decode (one register operand + a 16-bit
// extended operand).
// =====================================================================

macro_rules! op_property_abx_handler {
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
                property::OpPropertyAbxArgs {
                    a,
                    bx,
                    instruction_len,
                },
            );
            translate_outcome_to_step(state, outcome)
        }
    };
}

op_property_abx_handler!(op_create_object, property::op_create_object_semantic);
op_property_abx_handler!(op_create_array, property::op_create_array_semantic);
op_property_abx_handler!(
    op_check_object_coercible,
    property::op_check_object_coercible_semantic
);
op_property_abx_handler!(
    op_throw_if_uninitialized,
    property::op_throw_if_uninitialized_semantic
);

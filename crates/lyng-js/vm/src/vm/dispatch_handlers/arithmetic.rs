//! Arithmetic family handlers for the trampoline dispatch path (lyng-54em).
//!
//! Post-A9: each α handler in this file is a thin shim that
//!   1. decodes the instruction's operands,
//!   2. constructs `OpXxxArgs` and calls into
//!      `crate::vm::semantics::arithmetic::op_xxx_semantic`,
//!   3. translates the returned `SemanticOutcome` to `Step` via
//!      `translate_outcome_to_step`.
//!
//! The full SMI fast path + slow-helper logic lives in the semantic body.
//! `*Smi` variants (`AddSmi`, `SubSmi`, `MulSmi`, `BitAndSmi`, `DivSmi`,
//! `ModSmi`) carry the raw `c` operand as `imm_raw`; the semantic body
//! decodes it via `decode_smi_immediate`.

use crate::dsl::slow_path::LlIntDispatchState;
use crate::vm::dispatch::decode_abc_operands;
use crate::vm::dispatch_handlers::translate_outcome_to_step;
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::vm::semantics::arithmetic;
use crate::try_step;

// =====================================================================
// Add / Sub / Mul — two-register Abc with SMI fast path and feedback slot
// =====================================================================

macro_rules! op_binary_handler {
    ($name:ident, $semantic:path) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
            let code = state.code();
            let pc = state.frame.instruction_offset();
            let prefix = state.prefix.take();
            let (dst, lhs, rhs, feedback_slot, instruction_len) = try_step!(decode_abc_operands(
                state.current_bytes(),
                prefix,
                true,
                code,
                pc,
            ));
            let mut ll_state = LlIntDispatchState::from_alpha(state);
            let outcome = $semantic(
                &mut ll_state,
                arithmetic::OpBinaryArgs {
                    dst,
                    lhs,
                    rhs,
                    feedback_slot,
                    instruction_len,
                },
            );
            translate_outcome_to_step(state, outcome)
        }
    };
}

op_binary_handler!(op_add, arithmetic::op_add_semantic);
op_binary_handler!(op_sub, arithmetic::op_sub_semantic);
op_binary_handler!(op_mul, arithmetic::op_mul_semantic);

// =====================================================================
// AddSmi / SubSmi / MulSmi / BitAndSmi / DivSmi / ModSmi — register +
// i16 immediate (Abc-encoded, operand `c` carries the raw `u16`).
// =====================================================================

macro_rules! op_binary_smi_handler {
    ($name:ident, $semantic:path) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
            let code = state.code();
            let pc = state.frame.instruction_offset();
            let prefix = state.prefix.take();
            let (dst, lhs, c, feedback_slot, instruction_len) = try_step!(decode_abc_operands(
                state.current_bytes(),
                prefix,
                true,
                code,
                pc,
            ));
            let mut ll_state = LlIntDispatchState::from_alpha(state);
            let outcome = $semantic(
                &mut ll_state,
                arithmetic::OpBinarySmiArgs {
                    dst,
                    lhs,
                    imm_raw: c,
                    feedback_slot,
                    instruction_len,
                },
            );
            translate_outcome_to_step(state, outcome)
        }
    };
}

op_binary_smi_handler!(op_add_smi, arithmetic::op_add_smi_semantic);
op_binary_smi_handler!(op_sub_smi, arithmetic::op_sub_smi_semantic);
op_binary_smi_handler!(op_mul_smi, arithmetic::op_mul_smi_semantic);
op_binary_smi_handler!(op_bit_and_smi, arithmetic::op_bit_and_smi_semantic);
op_binary_smi_handler!(op_div_smi, arithmetic::op_div_smi_semantic);
op_binary_smi_handler!(op_mod_smi, arithmetic::op_mod_smi_semantic);

// =====================================================================
// Div / Mod / Exp + Bitwise + Shifts + Comparisons — same Abc decode as
// Add/Sub/Mul; semantic body chooses fast vs. slow internally.
// =====================================================================

op_binary_handler!(op_div, arithmetic::op_div_semantic);
op_binary_handler!(op_mod, arithmetic::op_mod_semantic);
op_binary_handler!(op_exp, arithmetic::op_exp_semantic);

op_binary_handler!(op_bit_and, arithmetic::op_bit_and_semantic);
op_binary_handler!(op_bit_or, arithmetic::op_bit_or_semantic);
op_binary_handler!(op_bit_xor, arithmetic::op_bit_xor_semantic);
op_binary_handler!(op_shift_left, arithmetic::op_shift_left_semantic);
op_binary_handler!(op_shift_right, arithmetic::op_shift_right_semantic);
op_binary_handler!(
    op_unsigned_shift_right,
    arithmetic::op_unsigned_shift_right_semantic
);

op_binary_handler!(op_equal, arithmetic::op_equal_semantic);
op_binary_handler!(op_strict_equal, arithmetic::op_strict_equal_semantic);
op_binary_handler!(op_less_than, arithmetic::op_less_than_semantic);
op_binary_handler!(op_less_equal, arithmetic::op_less_equal_semantic);
op_binary_handler!(op_greater_than, arithmetic::op_greater_than_semantic);
op_binary_handler!(op_greater_equal, arithmetic::op_greater_equal_semantic);

// =====================================================================
// EqualZero — Abc decode but the `c` operand is unused; cannot raise.
// =====================================================================

pub extern "C" fn op_equal_zero(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let prefix = state.prefix.take();
    let (dst, src, _c, feedback_slot, instruction_len) = try_step!(decode_abc_operands(
        state.current_bytes(),
        prefix,
        true,
        code,
        pc,
    ));
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = arithmetic::op_equal_zero_semantic(
        &mut ll_state,
        arithmetic::OpEqualZeroArgs {
            dst,
            src,
            feedback_slot,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

// =====================================================================
// Unary — Negate / BitNot / Increment / Decrement. Abc-decoded with the
// `c` operand unused; the semantic body routes results through
// `handle_dispatch_result`.
// =====================================================================

macro_rules! op_unary_handler {
    ($name:ident, $semantic:path) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
            let code = state.code();
            let pc = state.frame.instruction_offset();
            let prefix = state.prefix.take();
            let (dst, src, _c, feedback_slot, instruction_len) = try_step!(decode_abc_operands(
                state.current_bytes(),
                prefix,
                true,
                code,
                pc,
            ));
            let mut ll_state = LlIntDispatchState::from_alpha(state);
            let outcome = $semantic(
                &mut ll_state,
                arithmetic::OpUnaryArgs {
                    dst,
                    src,
                    feedback_slot,
                    instruction_len,
                },
            );
            translate_outcome_to_step(state, outcome)
        }
    };
}

op_unary_handler!(op_negate, arithmetic::op_negate_semantic);
op_unary_handler!(op_bit_not, arithmetic::op_bit_not_semantic);
op_unary_handler!(op_increment, arithmetic::op_increment_semantic);
op_unary_handler!(op_decrement, arithmetic::op_decrement_semantic);

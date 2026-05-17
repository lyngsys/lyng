//! Loads + register-window moves family handlers for the trampoline dispatch
//! path (lyng-5zrf).
//!
//! Post-A8: each α handler in this file is a thin shim that
//!   1. decodes the instruction's operands,
//!   2. constructs `OpXxxArgs` and calls into
//!      `crate::vm::semantics::loads::op_xxx_semantic`,
//!   3. translates the returned `SemanticOutcome` to `Step` via
//!      `translate_outcome_to_step` (or its accumulator-fusion variant).
//!
//! Family coverage (35 opcodes):
//! - `Move` (Abc form).
//! - Lda*-constant family: `LdaUndefined`, `LdaNull`, `LdaTrue`, `LdaFalse`,
//!   `LdaZero`, `LdaOne` — 1-byte opcodes that write a fixed value to
//!   register 0; fusion-aware.
//! - Load*-constant family: `LoadUndefined`, `LoadNull`, `LoadTrue`,
//!   `LoadFalse`, `LoadZero`, `LoadOne`, `LoadUninitializedLexical` —
//!   Abx-form opcodes that write a fixed value to register `a`.
//! - `Star0`..`Star7` — 1-byte opcodes that copy register 0 to a
//!   fixed-index register.
//! - Lda* with operand: `LdaSmi8`, `LdaConst8`, `Ldar` — fusion-aware.
//! - Load* with operand (Abx / Abx8): `LoadSmi`, `LoadSmi8`, `LoadConst`,
//!   `LoadConst8`.
//! - `LoadLocal0..3`, `StoreLocal0..3` — fixed-local-index ↔ explicit
//!   register.

use crate::vm::dispatch::{
    decode_abc_operands, decode_abx8_operands, decode_abx_operands,
    decode_accumulator_byte_operands, decode_accumulator_operands,
    decode_accumulator_register_operands, decode_local_operands,
};
use crate::vm::dispatch_handlers::{translate_outcome_to_step, translate_outcome_to_step_with_acc_fusion};
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::vm::semantics::loads;
use crate::{dsl::slow_path::LlIntDispatchState, try_step};

// =====================================================================
// Move (Abc form, no feedback slot)
// =====================================================================

pub extern "C" fn op_move(state: &mut DispatchState) -> Step {
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
    let outcome = loads::op_move_semantic(
        &mut ll_state,
        loads::OpMoveArgs {
            dst: a,
            src: b,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

// =====================================================================
// Lda* family — write fixed value to register 0 (accumulator).
// =====================================================================

macro_rules! op_lda_constant {
    ($name:ident, $semantic:path) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
            let code = state.code();
            let pc = state.frame.instruction_offset();
            let (_, instruction_len) = try_step!(decode_accumulator_operands(
                state.current_bytes(),
                false,
                code,
                pc,
            ));
            let mut ll_state = LlIntDispatchState::from_alpha(state);
            let outcome = $semantic(
                &mut ll_state,
                loads::OpLdaConstantArgs { instruction_len },
            );
            translate_outcome_to_step_with_acc_fusion(state, outcome)
        }
    };
}

op_lda_constant!(op_lda_undefined, loads::op_lda_undefined_semantic);
op_lda_constant!(op_lda_null, loads::op_lda_null_semantic);
op_lda_constant!(op_lda_true, loads::op_lda_true_semantic);
op_lda_constant!(op_lda_false, loads::op_lda_false_semantic);
op_lda_constant!(op_lda_zero, loads::op_lda_zero_semantic);
op_lda_constant!(op_lda_one, loads::op_lda_one_semantic);

// =====================================================================
// Load* family — Abx form, writes fixed value to explicit register a.
// =====================================================================

macro_rules! op_load_constant_abx {
    ($name:ident, $semantic:path) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
            let code = state.code();
            let pc = state.frame.instruction_offset();
            let prefix = state.prefix.take();
            let (a, _bx, _feedback_slot, instruction_len) = try_step!(decode_abx_operands(
                state.current_bytes(),
                prefix,
                false,
                code,
                pc,
            ));
            let mut ll_state = LlIntDispatchState::from_alpha(state);
            let outcome = $semantic(
                &mut ll_state,
                loads::OpLoadConstantArgs { a, instruction_len },
            );
            translate_outcome_to_step(state, outcome)
        }
    };
}

op_load_constant_abx!(op_load_undefined, loads::op_load_undefined_semantic);
op_load_constant_abx!(op_load_null, loads::op_load_null_semantic);
op_load_constant_abx!(op_load_true, loads::op_load_true_semantic);
op_load_constant_abx!(op_load_false, loads::op_load_false_semantic);
op_load_constant_abx!(op_load_zero, loads::op_load_zero_semantic);
op_load_constant_abx!(op_load_one, loads::op_load_one_semantic);
op_load_constant_abx!(
    op_load_uninitialized_lexical,
    loads::op_load_uninitialized_lexical_semantic
);

// =====================================================================
// Star0..Star7 — copy register 0 (accumulator) to a fixed-index register.
// =====================================================================

macro_rules! op_star_n {
    ($name:ident, $semantic:path) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
            let code = state.code();
            let pc = state.frame.instruction_offset();
            let (_, instruction_len) = try_step!(decode_accumulator_operands(
                state.current_bytes(),
                false,
                code,
                pc,
            ));
            let mut ll_state = LlIntDispatchState::from_alpha(state);
            let outcome = $semantic(&mut ll_state, loads::OpStarArgs { instruction_len });
            translate_outcome_to_step(state, outcome)
        }
    };
}

op_star_n!(op_star_0, loads::op_star_0_semantic);
op_star_n!(op_star_1, loads::op_star_1_semantic);
op_star_n!(op_star_2, loads::op_star_2_semantic);
op_star_n!(op_star_3, loads::op_star_3_semantic);
op_star_n!(op_star_4, loads::op_star_4_semantic);
op_star_n!(op_star_5, loads::op_star_5_semantic);
op_star_n!(op_star_6, loads::op_star_6_semantic);
op_star_n!(op_star_7, loads::op_star_7_semantic);

// =====================================================================
// Lda* with operands — small SMI, constant pool, register-to-accumulator.
// =====================================================================

pub extern "C" fn op_lda_smi8(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (bx, _feedback_slot, instruction_len) = try_step!(decode_accumulator_byte_operands(
        state.current_bytes(),
        false,
        code,
        pc,
    ));
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = loads::op_lda_smi8_semantic(
        &mut ll_state,
        loads::OpLdaSmi8Args {
            bx,
            instruction_len,
        },
    );
    translate_outcome_to_step_with_acc_fusion(state, outcome)
}

pub extern "C" fn op_lda_const8(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (bx, _feedback_slot, instruction_len) = try_step!(decode_accumulator_byte_operands(
        state.current_bytes(),
        false,
        code,
        pc,
    ));
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = loads::op_lda_const8_semantic(
        &mut ll_state,
        loads::OpLdaConst8Args {
            bx,
            instruction_len,
        },
    );
    translate_outcome_to_step_with_acc_fusion(state, outcome)
}

pub extern "C" fn op_ldar(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (a, _feedback_slot, instruction_len) = try_step!(decode_accumulator_register_operands(
        state.current_bytes(),
        false,
        code,
        pc,
    ));
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = loads::op_ldar_semantic(
        &mut ll_state,
        loads::OpLdarArgs { a, instruction_len },
    );
    translate_outcome_to_step_with_acc_fusion(state, outcome)
}

// =====================================================================
// Load* with operands — SMI, constant, all into an explicit register a.
// =====================================================================

pub extern "C" fn op_load_smi(state: &mut DispatchState) -> Step {
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
    let outcome = loads::op_load_smi_semantic(
        &mut ll_state,
        loads::OpLoadSmiArgs {
            a,
            bx,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

pub extern "C" fn op_load_smi8(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (a, bx, _feedback_slot, instruction_len) = try_step!(decode_abx8_operands(
        state.current_bytes(),
        false,
        code,
        pc,
    ));
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = loads::op_load_smi8_semantic(
        &mut ll_state,
        loads::OpLoadSmi8Args {
            a,
            bx,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

pub extern "C" fn op_load_const(state: &mut DispatchState) -> Step {
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
    let outcome = loads::op_load_const_semantic(
        &mut ll_state,
        loads::OpLoadConstArgs {
            a,
            bx,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

pub extern "C" fn op_load_const8(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (a, bx, _feedback_slot, instruction_len) = try_step!(decode_abx8_operands(
        state.current_bytes(),
        false,
        code,
        pc,
    ));
    let mut ll_state = LlIntDispatchState::from_alpha(state);
    let outcome = loads::op_load_const8_semantic(
        &mut ll_state,
        loads::OpLoadConst8Args {
            a,
            bx,
            instruction_len,
        },
    );
    translate_outcome_to_step(state, outcome)
}

// =====================================================================
// LoadLocal0..3 / StoreLocal0..3 — fixed local-index ↔ explicit register.
// =====================================================================

macro_rules! op_load_local_n {
    ($name:ident, $semantic:path) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
            let code = state.code();
            let pc = state.frame.instruction_offset();
            let (a, _feedback_slot, instruction_len) = try_step!(decode_local_operands(
                state.current_bytes(),
                false,
                code,
                pc,
            ));
            let mut ll_state = LlIntDispatchState::from_alpha(state);
            let outcome = $semantic(
                &mut ll_state,
                loads::OpLoadLocalArgs { a, instruction_len },
            );
            translate_outcome_to_step(state, outcome)
        }
    };
}

op_load_local_n!(op_load_local_0, loads::op_load_local_0_semantic);
op_load_local_n!(op_load_local_1, loads::op_load_local_1_semantic);
op_load_local_n!(op_load_local_2, loads::op_load_local_2_semantic);
op_load_local_n!(op_load_local_3, loads::op_load_local_3_semantic);

macro_rules! op_store_local_n {
    ($name:ident, $semantic:path) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
            let code = state.code();
            let pc = state.frame.instruction_offset();
            let (a, _feedback_slot, instruction_len) = try_step!(decode_local_operands(
                state.current_bytes(),
                false,
                code,
                pc,
            ));
            let mut ll_state = LlIntDispatchState::from_alpha(state);
            let outcome = $semantic(
                &mut ll_state,
                loads::OpStoreLocalArgs { a, instruction_len },
            );
            translate_outcome_to_step(state, outcome)
        }
    };
}

op_store_local_n!(op_store_local_0, loads::op_store_local_0_semantic);
op_store_local_n!(op_store_local_1, loads::op_store_local_1_semantic);
op_store_local_n!(op_store_local_2, loads::op_store_local_2_semantic);
op_store_local_n!(op_store_local_3, loads::op_store_local_3_semantic);

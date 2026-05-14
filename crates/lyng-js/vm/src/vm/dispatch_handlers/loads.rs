//! Loads + register-window moves family handlers for the trampoline dispatch
//! path (lyng-5zrf).
//!
//! Covers:
//! - `Move` (Abc form, no feedback slot).
//! - The accumulator-load family: `LdaUndefined`, `LdaNull`, `LdaTrue`,
//!   `LdaFalse`, `LdaZero`, `LdaOne` — 1-byte opcodes that write a fixed
//!   value to register 0.
//! - The explicit-target load family: `LoadUndefined`, `LoadNull`,
//!   `LoadTrue`, `LoadFalse`, `LoadZero`, `LoadOne`, `LoadUninitializedLexical`
//!   — Abx-form opcodes that write a fixed value to register `a`.
//! - The accumulator-store family: `Star0`..`Star7` — 1-byte opcodes that
//!   copy register 0 to a fixed-index register.
//!
//! Conditional-load (`LdaSmi8`, `LdaConst8`, `Ldar`), full-form loads with a
//! payload (`LoadSmi`, `LoadConst`), and the local-load/store family land in
//! follow-up commits.

use lyng_js_types::Value;

use crate::vm::dispatch::{decode_abc_operands, decode_abx_operands, decode_accumulator_operands};
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::{dispatch_next, try_step};

// =====================================================================
// Move (Abc form, no feedback slot)
// =====================================================================

pub extern "C" fn op_move(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (a, b, _c, _feedback_slot, instruction_len) = try_step!(decode_abc_operands(
        state.current_bytes(),
        None,
        false,
        code,
        pc,
    ));

    let registers = state.frame.registers();
    let value = state.vm.read_register(registers, b);
    state.vm.write_register(registers, a, value);
    state.advance(instruction_len);
    dispatch_next!(state);
}

// =====================================================================
// Lda* family — write fixed value to register 0 (accumulator).
// =====================================================================

macro_rules! op_lda_constant {
    ($name:ident, $value:expr) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
            let code = state.code();
            let pc = state.frame.instruction_offset();
            let (_, instruction_len) = try_step!(decode_accumulator_operands(
                state.current_bytes(),
                false,
                code,
                pc,
            ));
            let registers = state.frame.registers();
            state.vm.write_register(registers, 0, $value);
            state.advance(instruction_len);
            dispatch_next!(state);
        }
    };
}

op_lda_constant!(op_lda_undefined, Value::undefined());
op_lda_constant!(op_lda_null, Value::null());
op_lda_constant!(op_lda_true, Value::from_bool(true));
op_lda_constant!(op_lda_false, Value::from_bool(false));
op_lda_constant!(op_lda_zero, Value::from_smi(0));
op_lda_constant!(op_lda_one, Value::from_smi(1));

// =====================================================================
// Load* family — Abx form, writes fixed value to explicit register a.
// =====================================================================

macro_rules! op_load_constant_abx {
    ($name:ident, $value:expr) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
            let code = state.code();
            let pc = state.frame.instruction_offset();
            let (a, _bx, _feedback_slot, instruction_len) = try_step!(decode_abx_operands(
                state.current_bytes(),
                None,
                false,
                code,
                pc,
            ));
            let registers = state.frame.registers();
            state.vm.write_register(registers, a, $value);
            state.advance(instruction_len);
            dispatch_next!(state);
        }
    };
}

op_load_constant_abx!(op_load_undefined, Value::undefined());
op_load_constant_abx!(op_load_null, Value::null());
op_load_constant_abx!(op_load_true, Value::from_bool(true));
op_load_constant_abx!(op_load_false, Value::from_bool(false));
op_load_constant_abx!(op_load_zero, Value::from_smi(0));
op_load_constant_abx!(op_load_one, Value::from_smi(1));
op_load_constant_abx!(op_load_uninitialized_lexical, Value::uninitialized_lexical());

// =====================================================================
// Star0..Star7 — copy register 0 (accumulator) to a fixed-index register.
// =====================================================================

macro_rules! op_star_n {
    ($name:ident, $target:expr) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
            let code = state.code();
            let pc = state.frame.instruction_offset();
            let (_, instruction_len) = try_step!(decode_accumulator_operands(
                state.current_bytes(),
                false,
                code,
                pc,
            ));
            let registers = state.frame.registers();
            let value = state.vm.read_register(registers, 0);
            state.vm.write_register(registers, $target, value);
            state.advance(instruction_len);
            dispatch_next!(state);
        }
    };
}

op_star_n!(op_star_0, 0);
op_star_n!(op_star_1, 1);
op_star_n!(op_star_2, 2);
op_star_n!(op_star_3, 3);
op_star_n!(op_star_4, 4);
op_star_n!(op_star_5, 5);
op_star_n!(op_star_6, 6);
op_star_n!(op_star_7, 7);

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

use crate::vm::dispatch::{
    decode_abc_operands, decode_abx8_operands, decode_abx_operands,
    decode_accumulator_byte_operands, decode_accumulator_operands,
    decode_accumulator_register_operands, decode_local_operands,
};
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::{dispatch_next, try_step};

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

    let registers = state.frame.registers();
    let value = state.vm.read_register_unchecked(registers, b);
    state.vm.write_register_unchecked(registers, a, value);
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
            state.vm.write_register_unchecked(registers, 0, $value);
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
            let prefix = state.prefix.take();
            let (a, _bx, _feedback_slot, instruction_len) = try_step!(decode_abx_operands(
                state.current_bytes(),
                prefix,
                false,
                code,
                pc,
            ));
            let registers = state.frame.registers();
            state.vm.write_register_unchecked(registers, a, $value);
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
            let value = state.vm.read_register_unchecked(registers, 0);
            state.vm.write_register_unchecked(registers, $target, value);
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
    let value = i8::from_le_bytes([bx.to_le_bytes()[0]]);
    let registers = state.frame.registers();
    state
        .vm
        .write_register(registers, 0, Value::from_smi(i32::from(value)));
    state.advance(instruction_len);
    dispatch_next!(state);
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
    let value = try_step!(state.read_constant(bx));
    let registers = state.frame.registers();
    state.vm.write_register_unchecked(registers, 0, value);
    state.advance(instruction_len);
    dispatch_next!(state);
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
    let registers = state.frame.registers();
    let value = state.vm.read_register_unchecked(registers, a);
    state.vm.write_register_unchecked(registers, 0, value);
    state.advance(instruction_len);
    dispatch_next!(state);
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
    let bytes = bx.to_le_bytes();
    let value = i16::from_le_bytes([bytes[0], bytes[1]]);
    let registers = state.frame.registers();
    state
        .vm
        .write_register(registers, a, Value::from_smi(i32::from(value)));
    state.advance(instruction_len);
    dispatch_next!(state);
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
    let value = i8::from_le_bytes([bx.to_le_bytes()[0]]);
    let registers = state.frame.registers();
    state
        .vm
        .write_register(registers, a, Value::from_smi(i32::from(value)));
    state.advance(instruction_len);
    dispatch_next!(state);
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
    let value = try_step!(state.read_constant(bx));
    let registers = state.frame.registers();
    state.vm.write_register_unchecked(registers, a, value);
    state.advance(instruction_len);
    dispatch_next!(state);
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
    let value = try_step!(state.read_constant(bx));
    let registers = state.frame.registers();
    state.vm.write_register_unchecked(registers, a, value);
    state.advance(instruction_len);
    dispatch_next!(state);
}

// =====================================================================
// LoadLocal0..3 / StoreLocal0..3 — fixed local-index ↔ explicit register.
// =====================================================================

macro_rules! op_load_local_n {
    ($name:ident, $local:expr) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
            let code = state.code();
            let pc = state.frame.instruction_offset();
            let (a, _feedback_slot, instruction_len) = try_step!(decode_local_operands(
                state.current_bytes(),
                false,
                code,
                pc,
            ));
            let registers = state.frame.registers();
            let value = state.vm.read_register_unchecked(registers, $local);
            state.vm.write_register_unchecked(registers, a, value);
            state.advance(instruction_len);
            dispatch_next!(state);
        }
    };
}

op_load_local_n!(op_load_local_0, 0);
op_load_local_n!(op_load_local_1, 1);
op_load_local_n!(op_load_local_2, 2);
op_load_local_n!(op_load_local_3, 3);

macro_rules! op_store_local_n {
    ($name:ident, $local:expr) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
            let code = state.code();
            let pc = state.frame.instruction_offset();
            let (a, _feedback_slot, instruction_len) = try_step!(decode_local_operands(
                state.current_bytes(),
                false,
                code,
                pc,
            ));
            let registers = state.frame.registers();
            let value = state.vm.read_register_unchecked(registers, a);
            state.vm.write_register_unchecked(registers, $local, value);
            state.advance(instruction_len);
            dispatch_next!(state);
        }
    };
}

op_store_local_n!(op_store_local_0, 0);
op_store_local_n!(op_store_local_1, 1);
op_store_local_n!(op_store_local_2, 2);
op_store_local_n!(op_store_local_3, 3);

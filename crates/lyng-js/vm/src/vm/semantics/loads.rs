//! Loads + register-window moves family semantic bodies (DSL-0a Task A8).
//!
//! Each `op_xxx_semantic` function implements the semantic effect of one
//! loads-family opcode. The α handler in `dispatch_handlers/loads.rs`
//! decodes operands, constructs `OpXxxArgs`, calls the semantic body, and
//! translates the returned `SemanticOutcome` to `Step`. The DSL-0b cold-stub
//! shim in `dsl/handlers/cold/loads.rs` will reach the same functions from
//! the asm-DSL path.
//!
//! Family coverage (35 opcodes):
//! - `Move`
//! - Lda* (constant): `LdaUndefined`, `LdaNull`, `LdaTrue`, `LdaFalse`,
//!   `LdaZero`, `LdaOne` — write fixed value to register 0.
//! - Load* (Abx, constant): `LoadUndefined`, `LoadNull`, `LoadTrue`,
//!   `LoadFalse`, `LoadZero`, `LoadOne`, `LoadUninitializedLexical` — write
//!   fixed value to explicit register `a`.
//! - `Star0`..`Star7` — copy register 0 to a fixed-index register.
//! - Lda* (with operand): `LdaSmi8`, `LdaConst8`, `Ldar`.
//! - Load* (with operand, Abx): `LoadSmi`, `LoadSmi8`, `LoadConst`,
//!   `LoadConst8`.
//! - `LoadLocal0..3`, `StoreLocal0..3` — fixed-local-index ↔ explicit
//!   register.
//!
//! Accumulator-producing handlers (those whose α form uses
//! `dispatch_next_with_value!` for V8 Ignition's Star-fusion peephole) return
//! `SemanticOutcome::Continue { pc_advance }` after writing register 0. The
//! α thunk preserves the fusion behavior by reading register 0 after the
//! semantic returns and going through `dispatch_next_with_value!`. The
//! semantic body itself stays fusion-agnostic — fusion is a dispatch-time
//! optimization, not part of opcode semantics.

use lyng_js_types::Value;

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};

// =====================================================================
// Move (Abc form, no feedback slot)
// =====================================================================

pub struct OpMoveArgs {
    pub dst: u16,
    pub src: u16,
    pub instruction_len: u32,
}

pub(crate) fn op_move_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpMoveArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let registers = inner.frame.registers();
    let value = inner.vm.read_register_unchecked(registers, args.src);
    inner.vm.write_register_unchecked(registers, args.dst, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// Lda* family — write fixed value to register 0 (accumulator).
// =====================================================================

pub struct OpLdaConstantArgs {
    pub instruction_len: u32,
}

fn op_lda_constant_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLdaConstantArgs,
    value: Value,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let registers = inner.frame.registers();
    inner.vm.write_register_unchecked(registers, 0, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub(crate) fn op_lda_undefined_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLdaConstantArgs,
) -> SemanticOutcome {
    op_lda_constant_semantic(state, args, Value::undefined())
}

pub(crate) fn op_lda_null_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLdaConstantArgs,
) -> SemanticOutcome {
    op_lda_constant_semantic(state, args, Value::null())
}

pub(crate) fn op_lda_true_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLdaConstantArgs,
) -> SemanticOutcome {
    op_lda_constant_semantic(state, args, Value::from_bool(true))
}

pub(crate) fn op_lda_false_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLdaConstantArgs,
) -> SemanticOutcome {
    op_lda_constant_semantic(state, args, Value::from_bool(false))
}

pub(crate) fn op_lda_zero_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLdaConstantArgs,
) -> SemanticOutcome {
    op_lda_constant_semantic(state, args, Value::from_smi(0))
}

pub(crate) fn op_lda_one_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLdaConstantArgs,
) -> SemanticOutcome {
    op_lda_constant_semantic(state, args, Value::from_smi(1))
}

// =====================================================================
// Load* family — Abx form, writes fixed value to explicit register a.
// =====================================================================

pub struct OpLoadConstantArgs {
    pub a: u16,
    pub instruction_len: u32,
}

fn op_load_constant_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoadConstantArgs,
    value: Value,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let registers = inner.frame.registers();
    inner.vm.write_register_unchecked(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub(crate) fn op_load_undefined_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoadConstantArgs,
) -> SemanticOutcome {
    op_load_constant_semantic(state, args, Value::undefined())
}

pub(crate) fn op_load_null_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoadConstantArgs,
) -> SemanticOutcome {
    op_load_constant_semantic(state, args, Value::null())
}

pub(crate) fn op_load_true_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoadConstantArgs,
) -> SemanticOutcome {
    op_load_constant_semantic(state, args, Value::from_bool(true))
}

pub(crate) fn op_load_false_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoadConstantArgs,
) -> SemanticOutcome {
    op_load_constant_semantic(state, args, Value::from_bool(false))
}

pub(crate) fn op_load_zero_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoadConstantArgs,
) -> SemanticOutcome {
    op_load_constant_semantic(state, args, Value::from_smi(0))
}

pub(crate) fn op_load_one_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoadConstantArgs,
) -> SemanticOutcome {
    op_load_constant_semantic(state, args, Value::from_smi(1))
}

pub(crate) fn op_load_uninitialized_lexical_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoadConstantArgs,
) -> SemanticOutcome {
    op_load_constant_semantic(state, args, Value::uninitialized_lexical())
}

// =====================================================================
// Star0..Star7 — copy register 0 (accumulator) to a fixed-index register.
// =====================================================================

pub struct OpStarArgs {
    pub instruction_len: u32,
}

fn op_star_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpStarArgs,
    target: u16,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let registers = inner.frame.registers();
    let value = inner.vm.read_register_unchecked(registers, 0);
    inner.vm.write_register_unchecked(registers, target, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub(crate) fn op_star_0_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpStarArgs,
) -> SemanticOutcome {
    op_star_semantic(state, args, 0)
}

pub(crate) fn op_star_1_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpStarArgs,
) -> SemanticOutcome {
    op_star_semantic(state, args, 1)
}

pub(crate) fn op_star_2_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpStarArgs,
) -> SemanticOutcome {
    op_star_semantic(state, args, 2)
}

pub(crate) fn op_star_3_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpStarArgs,
) -> SemanticOutcome {
    op_star_semantic(state, args, 3)
}

pub(crate) fn op_star_4_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpStarArgs,
) -> SemanticOutcome {
    op_star_semantic(state, args, 4)
}

pub(crate) fn op_star_5_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpStarArgs,
) -> SemanticOutcome {
    op_star_semantic(state, args, 5)
}

pub(crate) fn op_star_6_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpStarArgs,
) -> SemanticOutcome {
    op_star_semantic(state, args, 6)
}

pub(crate) fn op_star_7_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpStarArgs,
) -> SemanticOutcome {
    op_star_semantic(state, args, 7)
}

// =====================================================================
// Lda* with operands — small SMI, constant pool, register-to-accumulator.
// =====================================================================

pub struct OpLdaSmi8Args {
    /// Decoded immediate byte from the bytecode (raw u32 for symmetry with
    /// `decode_accumulator_byte_operands`; only the low byte is meaningful).
    pub bx: u32,
    pub instruction_len: u32,
}

pub(crate) fn op_lda_smi8_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLdaSmi8Args,
) -> SemanticOutcome {
    let raw = i8::from_le_bytes([args.bx.to_le_bytes()[0]]);
    let value = Value::from_smi(i32::from(raw));
    let inner = state.dispatch_state();
    let registers = inner.frame.registers();
    inner.vm.write_register(registers, 0, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub struct OpLdaConst8Args {
    pub bx: u32,
    pub instruction_len: u32,
}

pub(crate) fn op_lda_const8_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLdaConst8Args,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = match inner.read_constant(args.bx) {
        Ok(v) => v,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.frame.registers();
    inner.vm.write_register_unchecked(registers, 0, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub struct OpLdarArgs {
    pub a: u16,
    pub instruction_len: u32,
}

pub(crate) fn op_ldar_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLdarArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let registers = inner.frame.registers();
    let value = inner.vm.read_register_unchecked(registers, args.a);
    inner.vm.write_register_unchecked(registers, 0, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// Load* with operands — SMI, constant, all into an explicit register a.
// =====================================================================

pub struct OpLoadSmiArgs {
    pub a: u16,
    /// Decoded `bx` operand (low 16 bits hold the i16 immediate).
    pub bx: u32,
    pub instruction_len: u32,
}

pub(crate) fn op_load_smi_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoadSmiArgs,
) -> SemanticOutcome {
    let bytes = args.bx.to_le_bytes();
    let value = i16::from_le_bytes([bytes[0], bytes[1]]);
    let inner = state.dispatch_state();
    let registers = inner.frame.registers();
    inner
        .vm
        .write_register(registers, args.a, Value::from_smi(i32::from(value)));
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub struct OpLoadSmi8Args {
    pub a: u16,
    pub bx: u32,
    pub instruction_len: u32,
}

pub(crate) fn op_load_smi8_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoadSmi8Args,
) -> SemanticOutcome {
    let value = i8::from_le_bytes([args.bx.to_le_bytes()[0]]);
    let inner = state.dispatch_state();
    let registers = inner.frame.registers();
    inner
        .vm
        .write_register(registers, args.a, Value::from_smi(i32::from(value)));
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub struct OpLoadConstArgs {
    pub a: u16,
    pub bx: u32,
    pub instruction_len: u32,
}

pub(crate) fn op_load_const_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoadConstArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = match inner.read_constant(args.bx) {
        Ok(v) => v,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.frame.registers();
    inner.vm.write_register_unchecked(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub struct OpLoadConst8Args {
    pub a: u16,
    pub bx: u32,
    pub instruction_len: u32,
}

pub(crate) fn op_load_const8_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoadConst8Args,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = match inner.read_constant(args.bx) {
        Ok(v) => v,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.frame.registers();
    inner.vm.write_register_unchecked(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// LoadLocal0..3 / StoreLocal0..3 — fixed local-index ↔ explicit register.
// =====================================================================

pub struct OpLoadLocalArgs {
    pub a: u16,
    pub instruction_len: u32,
}

fn op_load_local_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoadLocalArgs,
    local: u16,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let registers = inner.frame.registers();
    let value = inner.vm.read_register_unchecked(registers, local);
    inner.vm.write_register_unchecked(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub(crate) fn op_load_local_0_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoadLocalArgs,
) -> SemanticOutcome {
    op_load_local_semantic(state, args, 0)
}

pub(crate) fn op_load_local_1_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoadLocalArgs,
) -> SemanticOutcome {
    op_load_local_semantic(state, args, 1)
}

pub(crate) fn op_load_local_2_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoadLocalArgs,
) -> SemanticOutcome {
    op_load_local_semantic(state, args, 2)
}

pub(crate) fn op_load_local_3_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoadLocalArgs,
) -> SemanticOutcome {
    op_load_local_semantic(state, args, 3)
}

pub struct OpStoreLocalArgs {
    pub a: u16,
    pub instruction_len: u32,
}

fn op_store_local_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpStoreLocalArgs,
    local: u16,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let registers = inner.frame.registers();
    let value = inner.vm.read_register_unchecked(registers, args.a);
    inner.vm.write_register_unchecked(registers, local, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub(crate) fn op_store_local_0_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpStoreLocalArgs,
) -> SemanticOutcome {
    op_store_local_semantic(state, args, 0)
}

pub(crate) fn op_store_local_1_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpStoreLocalArgs,
) -> SemanticOutcome {
    op_store_local_semantic(state, args, 1)
}

pub(crate) fn op_store_local_2_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpStoreLocalArgs,
) -> SemanticOutcome {
    op_store_local_semantic(state, args, 2)
}

pub(crate) fn op_store_local_3_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpStoreLocalArgs,
) -> SemanticOutcome {
    op_store_local_semantic(state, args, 3)
}

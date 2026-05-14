#![allow(
    improper_ctypes_definitions,
    reason = "extern \"C\" handlers carry Rust enums by value as an ABI-stability choice, not as a real FFI boundary"
)]
#![allow(
    dead_code,
    reason = "Phase 1 trampoline spike exercises these via tests and cargo-asm only; production cutover lives in follow-up sub-issues"
)]

//! Phase 1 trampoline spike — handler family modules.
//!
//! Each family file holds the `extern "C" fn` handlers for a related group of
//! opcodes. The spike implements one representative handler per family so the
//! trampoline shape can be measured; Phase 1 proper scales this to all 152
//! opcodes per the JSC-aligned roadmap.

use lyng_js_bytecode::{Opcode, OPCODE_COUNT};

use super::dispatch_state::{Handler, DISPATCH_TABLE_LEN};

pub mod arithmetic;
pub mod control_flow;
pub mod loads;
pub mod opcode_stubs;
pub mod prefix;
pub mod stub;

pub use arithmetic::{
    op_add, op_add_smi, op_bit_and, op_bit_and_smi, op_bit_not, op_bit_or, op_bit_xor,
    op_decrement, op_equal, op_equal_zero, op_greater_equal, op_greater_than, op_increment,
    op_less_equal, op_less_than, op_mul, op_mul_smi, op_negate, op_shift_left, op_shift_right,
    op_strict_equal, op_sub, op_sub_smi, op_unsigned_shift_right,
};
pub use control_flow::{
    op_jump, op_jump8, op_jump_if_false, op_jump_if_false8, op_jump_if_true, op_jump_if_true8,
    op_loop_header, op_return, op_return_undefined,
};
pub use loads::{
    op_lda_const8, op_lda_false, op_lda_null, op_lda_one, op_lda_smi8, op_lda_true,
    op_lda_undefined, op_lda_zero, op_ldar, op_load_const, op_load_const8, op_load_false,
    op_load_local_0, op_load_local_1, op_load_local_2, op_load_local_3, op_load_null, op_load_one,
    op_load_smi, op_load_smi8, op_load_true, op_load_undefined, op_load_uninitialized_lexical,
    op_load_zero, op_move, op_star_0, op_star_1, op_star_2, op_star_3, op_star_4, op_star_5,
    op_star_6, op_star_7, op_store_local_0, op_store_local_1, op_store_local_2, op_store_local_3,
};
pub use prefix::{op_extra_wide, op_wide};

/// Build the dispatch table at compile time.
///
/// 1. Every byte 0..256 starts as `op_stub` — handles invalid bytes outside
///    the valid `Opcode` range (152..256).
/// 2. Every valid opcode slot (0..OPCODE_COUNT) is overwritten with a
///    `op_unimplemented::<N>` monomorphization, giving 152 distinct symbols
///    (Phase 1's structural invariant from lyng-33i2).
/// 3. The real handlers from sub-3 onward override the slots for opcodes
///    we've ported. Family-conversion sub-issues (lyng-5zrf, lyng-54em,
///    lyng-5mqv, lyng-1fie, lyng-59e6) overwrite progressively more slots
///    until every valid opcode lands on a real handler.
///
/// Adding a real handler in a family file without registering it here is a
/// silent bug — the handler would be dead code and the opcode would still
/// dispatch through `op_unimplemented`.
pub const fn build_dispatch_table() -> [Handler; DISPATCH_TABLE_LEN] {
    let mut table: [Handler; DISPATCH_TABLE_LEN] = [stub::op_stub; DISPATCH_TABLE_LEN];

    // (2) Populate every valid opcode slot with its distinct stub.
    let mut i: usize = 0;
    while i < OPCODE_COUNT as usize {
        table[i] = opcode_stubs::UNIMPLEMENTED_HANDLERS[i];
        i += 1;
    }

    // (3) Real handlers — overrides the unimplemented stubs above.
    //
    // sub-3 (lyng-5zrf): loads + control_flow family.
    table[Opcode::Move as u8 as usize] = op_move;

    table[Opcode::LdaUndefined as u8 as usize] = op_lda_undefined;
    table[Opcode::LdaNull as u8 as usize] = op_lda_null;
    table[Opcode::LdaTrue as u8 as usize] = op_lda_true;
    table[Opcode::LdaFalse as u8 as usize] = op_lda_false;
    table[Opcode::LdaZero as u8 as usize] = op_lda_zero;
    table[Opcode::LdaOne as u8 as usize] = op_lda_one;

    table[Opcode::LoadUndefined as u8 as usize] = op_load_undefined;
    table[Opcode::LoadNull as u8 as usize] = op_load_null;
    table[Opcode::LoadTrue as u8 as usize] = op_load_true;
    table[Opcode::LoadFalse as u8 as usize] = op_load_false;
    table[Opcode::LoadZero as u8 as usize] = op_load_zero;
    table[Opcode::LoadOne as u8 as usize] = op_load_one;
    table[Opcode::LoadUninitializedLexical as u8 as usize] = op_load_uninitialized_lexical;

    table[Opcode::Star0 as u8 as usize] = op_star_0;
    table[Opcode::Star1 as u8 as usize] = op_star_1;
    table[Opcode::Star2 as u8 as usize] = op_star_2;
    table[Opcode::Star3 as u8 as usize] = op_star_3;
    table[Opcode::Star4 as u8 as usize] = op_star_4;
    table[Opcode::Star5 as u8 as usize] = op_star_5;
    table[Opcode::Star6 as u8 as usize] = op_star_6;
    table[Opcode::Star7 as u8 as usize] = op_star_7;

    table[Opcode::LdaSmi8 as u8 as usize] = op_lda_smi8;
    table[Opcode::LdaConst8 as u8 as usize] = op_lda_const8;
    table[Opcode::Ldar as u8 as usize] = op_ldar;

    table[Opcode::LoadSmi as u8 as usize] = op_load_smi;
    table[Opcode::LoadSmi8 as u8 as usize] = op_load_smi8;
    table[Opcode::LoadConst as u8 as usize] = op_load_const;
    table[Opcode::LoadConst8 as u8 as usize] = op_load_const8;

    table[Opcode::LoadLocal0 as u8 as usize] = op_load_local_0;
    table[Opcode::LoadLocal1 as u8 as usize] = op_load_local_1;
    table[Opcode::LoadLocal2 as u8 as usize] = op_load_local_2;
    table[Opcode::LoadLocal3 as u8 as usize] = op_load_local_3;

    table[Opcode::StoreLocal0 as u8 as usize] = op_store_local_0;
    table[Opcode::StoreLocal1 as u8 as usize] = op_store_local_1;
    table[Opcode::StoreLocal2 as u8 as usize] = op_store_local_2;
    table[Opcode::StoreLocal3 as u8 as usize] = op_store_local_3;

    table[Opcode::Jump as u8 as usize] = op_jump;
    table[Opcode::Jump8 as u8 as usize] = op_jump8;
    table[Opcode::JumpIfTrue as u8 as usize] = op_jump_if_true;
    table[Opcode::JumpIfFalse as u8 as usize] = op_jump_if_false;
    table[Opcode::JumpIfTrue8 as u8 as usize] = op_jump_if_true8;
    table[Opcode::JumpIfFalse8 as u8 as usize] = op_jump_if_false8;
    table[Opcode::LoopHeader as u8 as usize] = op_loop_header;
    table[Opcode::Return as u8 as usize] = op_return;
    table[Opcode::ReturnUndefined as u8 as usize] = op_return_undefined;

    table[Opcode::Wide as u8 as usize] = op_wide;
    table[Opcode::ExtraWide as u8 as usize] = op_extra_wide;

    // sub-4 (lyng-54em): arithmetic family.
    table[Opcode::Add as u8 as usize] = op_add;
    table[Opcode::AddSmi as u8 as usize] = op_add_smi;
    table[Opcode::Sub as u8 as usize] = op_sub;
    table[Opcode::SubSmi as u8 as usize] = op_sub_smi;
    table[Opcode::Mul as u8 as usize] = op_mul;
    table[Opcode::MulSmi as u8 as usize] = op_mul_smi;
    table[Opcode::Negate as u8 as usize] = op_negate;
    table[Opcode::BitNot as u8 as usize] = op_bit_not;
    table[Opcode::Increment as u8 as usize] = op_increment;
    table[Opcode::Decrement as u8 as usize] = op_decrement;
    table[Opcode::Equal as u8 as usize] = op_equal;
    table[Opcode::StrictEqual as u8 as usize] = op_strict_equal;
    table[Opcode::EqualZero as u8 as usize] = op_equal_zero;
    table[Opcode::LessThan as u8 as usize] = op_less_than;
    table[Opcode::LessEqual as u8 as usize] = op_less_equal;
    table[Opcode::GreaterThan as u8 as usize] = op_greater_than;
    table[Opcode::GreaterEqual as u8 as usize] = op_greater_equal;

    table[Opcode::BitOr as u8 as usize] = op_bit_or;
    table[Opcode::BitXor as u8 as usize] = op_bit_xor;
    table[Opcode::BitAnd as u8 as usize] = op_bit_and;
    table[Opcode::BitAndSmi as u8 as usize] = op_bit_and_smi;
    table[Opcode::ShiftLeft as u8 as usize] = op_shift_left;
    table[Opcode::ShiftRight as u8 as usize] = op_shift_right;
    table[Opcode::UnsignedShiftRight as u8 as usize] = op_unsigned_shift_right;

    table
}

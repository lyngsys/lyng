//! Cold-stub codegen for DSL-0b (B46–B47).
//!
//! Emits `crates/vm/src/dsl/handlers/cold.rs`, populating one
//! `llint_handler!` block + matching `op_xxx_slow_rs` shim per Cold
//! opcode listed in [`COLD_STUBS`].
//!
//! ## Why a separate codegen tool?
//!
//! The cold stubs are deeply uniform (asm prologue, `call_slow!`,
//! `dispatch_after_slow!`, then a Rust shim that constructs the args
//! struct and dispatches to the semantic body) but their _shape_ varies
//! per-opcode: layout (Abc / Abx / Ax / ...), instruction length,
//! arguments struct path, and the field-by-field operand → field
//! conversion. Encoding this as a `macro_rules!` would require either
//! one macro per args shape (and one invocation per opcode → still
//! verbose) or a single very-complex macro. A standalone codegen tool
//! is simpler, debuggable as plain Rust, and makes the per-opcode
//! conversion explicit in source instead of buried in macro arms.
//!
//! ## Metadata table
//!
//! [`COLD_STUBS`] holds 140 rows, one per Cold opcode. Each row carries
//! enough to emit:
//!
//! 1. `llint_handler! { op_xxx_dsl, opcode_byte = N, layout = L, length = N, |...| {
//!    call_slow!(op_xxx_slow_rs, args = [...]); dispatch_after_slow!(); }
//!    }`
//! 2. `extern "C" fn op_xxx_slow_rs(state, op0, ...) -> SlowPathReturn`
//!    that reconstructs the args struct and dispatches to
//!    `crate::vm::semantics::<family>::op_xxx_semantic`.
//!
//! The tool's emit logic is dumb — it just expands templates per-row.
//! All correctness lives in the metadata table; the test suite's
//! manifest-walk + cargo build catch mismatches.
//!
//! ## Adding a new Cold opcode
//!
//! 1. Add a `Stub { ... }` row to [`COLD_STUBS`].
//! 2. Re-run `cargo run -p lyng-dsl-codegen`.
//! 3. Verify `cargo build -p lyng-vm` is clean.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use lyng_bytecode::Opcode;

/// Operand-decode layout the asm side uses. Mirrors
/// `lyng_vm_dsl::layouts::Layout`. The proc-macro consumes the
/// `layout = X` token in the emitted `llint_handler!` block.
#[derive(Clone, Copy, Debug)]
enum Layout {
    /// No operand bytes after the opcode (1-byte instruction). Used for
    /// the `Lda*` constant family and `Star0..7`.
    None,
    /// Single 1-byte operand at `[PC+1]` (2-byte instruction).
    A,
    /// Two 1-byte operands at `[PC+1, PC+2]` (3-byte instruction).
    Ab,
    /// Three 1-byte operands at `[PC+1..PC+4]` (4-byte instruction).
    Abc,
    /// `Abc` + 16-bit feedback slot at `[PC+4, PC+5]` (6-byte instruction).
    AbcSlot,
    /// 1-byte + 16-bit operand at `[PC+1, PC+2..PC+4]` (4-byte instruction).
    /// For the `Abx + feedback` opcodes (LoadGlobal etc.) the actual
    /// encoding is 6 bytes; the asm layout still maps the named
    /// operands `a, bx` to the first 4 bytes — the slot is recovered
    /// by the slow shim from PC.
    Abx,
    /// 32-bit operand at `[PC+1..PC+5]` (4-byte instruction; the
    /// `decode_ax!` macro reads `ldr w?, [x19, #1]` which spans
    /// PC+1..PC+5, overlapping the next opcode byte; the high byte is
    /// masked off by the slow shim).
    Ax,
}

impl Layout {
    /// Rust ident used inside `layout = X` in the emitted handler.
    fn ident(self) -> &'static str {
        match self {
            Layout::None => "None",
            Layout::A => "A",
            Layout::Ab => "Ab",
            Layout::Abc => "Abc",
            Layout::AbcSlot => "AbcSlot",
            Layout::Abx => "Abx",
            Layout::Ax => "Ax",
        }
    }

    /// Operand names exposed to the `|...|` parameter list of the
    /// emitted `llint_handler!` body. Order matches the decoder's
    /// argument order.
    fn operand_names(self) -> &'static [&'static str] {
        match self {
            Layout::None => &[],
            Layout::A => &["a"],
            Layout::Ab => &["a", "b"],
            Layout::Abc => &["a", "b", "c"],
            Layout::AbcSlot => &["a", "b", "c", "slot"],
            Layout::Abx => &["a", "bx"],
            Layout::Ax => &["ax"],
        }
    }
}

/// One Cold opcode → its emitted shim.
#[derive(Clone, Copy)]
struct Stub {
    /// `Opcode` variant — links the stub back to the bytecode enum.
    opcode: Opcode,
    /// Family module path (e.g. `loads`, `arithmetic`) under
    /// `crate::vm::semantics::`. Embedded into the semantic call.
    family: &'static str,
    /// Bare name of the semantic function inside the family module
    /// (e.g. `op_load_global_semantic`).
    semantic: &'static str,
    /// Bare name of the args struct inside the family module
    /// (e.g. `OpAtomArgs`).
    args: &'static str,
    /// Operand-decode layout for the asm prologue.
    layout: Layout,
    /// Encoded instruction length (narrow form) for the `length =`
    /// binding in the emitted `llint_handler!` body.
    length: u32,
    /// Per-stub fixups for the slow-path shim. Each entry maps an
    /// args-struct field to a conversion expression that builds it
    /// from the raw u32 operands. Field names must match the args
    /// struct definition exactly; missing fields are written as
    /// `Default::default()` if absent from this list.
    fields: &'static [Field],
}

/// One field in the args struct.
///
/// `expr` is spliced after `: ` in the emitted shim — anything that
/// reads `state, a, b, c, slot, ax, bx` (the operand bindings) is
/// valid, plus `instruction_len` (a `u32` literal substituted from
/// `Stub::length`).
#[derive(Clone, Copy)]
struct Field {
    name: &'static str,
    expr: &'static str,
}

/// Convenience: one short field. Use for fields whose expression is
/// just a cast like `a as u16`.
const fn f(name: &'static str, expr: &'static str) -> Field {
    Field { name, expr }
}

/// The metadata table.
///
/// Order doesn't matter — the tool sorts by Opcode discriminant before
/// emitting to keep the output stable. Each entry must cover one
/// Cold-categorized opcode from `crates/vm/src/dsl/opcode_manifest.rs`.
///
/// Mismatch with the manifest is caught at link time by the
/// `dsl_handler_symbol` resolution test (Test 3).
const COLD_STUBS: &[Stub] = &[
    // =================================================================
    // loads family — Lda* / Load* / Star* / LoadLocal* / StoreLocal*
    // =================================================================

    // Lda* fixed constants: 1-byte opcodes that write to register 0.
    Stub {
        opcode: Opcode::LdaUndefined,
        family: "loads",
        semantic: "op_lda_undefined_semantic",
        args: "OpLdaConstantArgs",
        layout: Layout::None,
        length: 1,
        fields: &[f("instruction_len", "1u32")],
    },
    Stub {
        opcode: Opcode::LdaNull,
        family: "loads",
        semantic: "op_lda_null_semantic",
        args: "OpLdaConstantArgs",
        layout: Layout::None,
        length: 1,
        fields: &[f("instruction_len", "1u32")],
    },
    Stub {
        opcode: Opcode::LdaTrue,
        family: "loads",
        semantic: "op_lda_true_semantic",
        args: "OpLdaConstantArgs",
        layout: Layout::None,
        length: 1,
        fields: &[f("instruction_len", "1u32")],
    },
    Stub {
        opcode: Opcode::LdaFalse,
        family: "loads",
        semantic: "op_lda_false_semantic",
        args: "OpLdaConstantArgs",
        layout: Layout::None,
        length: 1,
        fields: &[f("instruction_len", "1u32")],
    },
    Stub {
        opcode: Opcode::LdaZero,
        family: "loads",
        semantic: "op_lda_zero_semantic",
        args: "OpLdaConstantArgs",
        layout: Layout::None,
        length: 1,
        fields: &[f("instruction_len", "1u32")],
    },
    Stub {
        opcode: Opcode::LdaOne,
        family: "loads",
        semantic: "op_lda_one_semantic",
        args: "OpLdaConstantArgs",
        layout: Layout::None,
        length: 1,
        fields: &[f("instruction_len", "1u32")],
    },
    // Load* fixed constants (Abx form, write to register `a`).
    Stub {
        opcode: Opcode::LoadUndefined,
        family: "loads",
        semantic: "op_load_undefined_semantic",
        args: "OpLoadConstantArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[f("a", "a as u16"), f("instruction_len", "4u32")],
    },
    Stub {
        opcode: Opcode::LoadNull,
        family: "loads",
        semantic: "op_load_null_semantic",
        args: "OpLoadConstantArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[f("a", "a as u16"), f("instruction_len", "4u32")],
    },
    Stub {
        opcode: Opcode::LoadTrue,
        family: "loads",
        semantic: "op_load_true_semantic",
        args: "OpLoadConstantArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[f("a", "a as u16"), f("instruction_len", "4u32")],
    },
    Stub {
        opcode: Opcode::LoadFalse,
        family: "loads",
        semantic: "op_load_false_semantic",
        args: "OpLoadConstantArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[f("a", "a as u16"), f("instruction_len", "4u32")],
    },
    Stub {
        opcode: Opcode::LoadZero,
        family: "loads",
        semantic: "op_load_zero_semantic",
        args: "OpLoadConstantArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[f("a", "a as u16"), f("instruction_len", "4u32")],
    },
    Stub {
        opcode: Opcode::LoadOne,
        family: "loads",
        semantic: "op_load_one_semantic",
        args: "OpLoadConstantArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[f("a", "a as u16"), f("instruction_len", "4u32")],
    },
    Stub {
        opcode: Opcode::LoadUninitializedLexical,
        family: "loads",
        semantic: "op_load_uninitialized_lexical_semantic",
        args: "OpLoadConstantArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[f("a", "a as u16"), f("instruction_len", "4u32")],
    },
    // Star0..Star7 — 1-byte opcodes copying register 0 to fixed dest.
    Stub {
        opcode: Opcode::Star0,
        family: "loads",
        semantic: "op_star_0_semantic",
        args: "OpStarArgs",
        layout: Layout::None,
        length: 1,
        fields: &[f("instruction_len", "1u32")],
    },
    Stub {
        opcode: Opcode::Star1,
        family: "loads",
        semantic: "op_star_1_semantic",
        args: "OpStarArgs",
        layout: Layout::None,
        length: 1,
        fields: &[f("instruction_len", "1u32")],
    },
    Stub {
        opcode: Opcode::Star2,
        family: "loads",
        semantic: "op_star_2_semantic",
        args: "OpStarArgs",
        layout: Layout::None,
        length: 1,
        fields: &[f("instruction_len", "1u32")],
    },
    Stub {
        opcode: Opcode::Star3,
        family: "loads",
        semantic: "op_star_3_semantic",
        args: "OpStarArgs",
        layout: Layout::None,
        length: 1,
        fields: &[f("instruction_len", "1u32")],
    },
    Stub {
        opcode: Opcode::Star4,
        family: "loads",
        semantic: "op_star_4_semantic",
        args: "OpStarArgs",
        layout: Layout::None,
        length: 1,
        fields: &[f("instruction_len", "1u32")],
    },
    Stub {
        opcode: Opcode::Star5,
        family: "loads",
        semantic: "op_star_5_semantic",
        args: "OpStarArgs",
        layout: Layout::None,
        length: 1,
        fields: &[f("instruction_len", "1u32")],
    },
    Stub {
        opcode: Opcode::Star6,
        family: "loads",
        semantic: "op_star_6_semantic",
        args: "OpStarArgs",
        layout: Layout::None,
        length: 1,
        fields: &[f("instruction_len", "1u32")],
    },
    Stub {
        opcode: Opcode::Star7,
        family: "loads",
        semantic: "op_star_7_semantic",
        args: "OpStarArgs",
        layout: Layout::None,
        length: 1,
        fields: &[f("instruction_len", "1u32")],
    },
    // Lda* with operand: LdaSmi8/LdaConst8/Ldar (2-byte form).
    Stub {
        opcode: Opcode::LdaSmi8,
        family: "loads",
        semantic: "op_lda_smi8_semantic",
        args: "OpLdaSmi8Args",
        layout: Layout::A,
        length: 2,
        fields: &[f("bx", "a"), f("instruction_len", "2u32")],
    },
    Stub {
        opcode: Opcode::LdaConst8,
        family: "loads",
        semantic: "op_lda_const8_semantic",
        args: "OpLdaConst8Args",
        layout: Layout::A,
        length: 2,
        fields: &[f("bx", "a"), f("instruction_len", "2u32")],
    },
    Stub {
        opcode: Opcode::Ldar,
        family: "loads",
        semantic: "op_ldar_semantic",
        args: "OpLdarArgs",
        layout: Layout::A,
        length: 2,
        fields: &[f("a", "a as u16"), f("instruction_len", "2u32")],
    },
    // Load* with operand: LoadSmi/LoadConst (4-byte Abx), LoadSmi8/LoadConst8 (3-byte Ab).
    Stub {
        opcode: Opcode::LoadSmi,
        family: "loads",
        semantic: "op_load_smi_semantic",
        args: "OpLoadSmiArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::LoadSmi8,
        family: "loads",
        semantic: "op_load_smi8_semantic",
        args: "OpLoadSmi8Args",
        layout: Layout::Ab,
        length: 3,
        fields: &[
            f("a", "a as u16"),
            f("bx", "b"),
            f("instruction_len", "3u32"),
        ],
    },
    Stub {
        opcode: Opcode::LoadConst,
        family: "loads",
        semantic: "op_load_const_semantic",
        args: "OpLoadConstArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::LoadConst8,
        family: "loads",
        semantic: "op_load_const8_semantic",
        args: "OpLoadConst8Args",
        layout: Layout::Ab,
        length: 3,
        fields: &[
            f("a", "a as u16"),
            f("bx", "b"),
            f("instruction_len", "3u32"),
        ],
    },
    // LoadLocal0..3 / StoreLocal0..3 — 2-byte form, A layout (one register).
    Stub {
        opcode: Opcode::LoadLocal0,
        family: "loads",
        semantic: "op_load_local_0_semantic",
        args: "OpLoadLocalArgs",
        layout: Layout::A,
        length: 2,
        fields: &[f("a", "a as u16"), f("instruction_len", "2u32")],
    },
    Stub {
        opcode: Opcode::LoadLocal1,
        family: "loads",
        semantic: "op_load_local_1_semantic",
        args: "OpLoadLocalArgs",
        layout: Layout::A,
        length: 2,
        fields: &[f("a", "a as u16"), f("instruction_len", "2u32")],
    },
    Stub {
        opcode: Opcode::LoadLocal2,
        family: "loads",
        semantic: "op_load_local_2_semantic",
        args: "OpLoadLocalArgs",
        layout: Layout::A,
        length: 2,
        fields: &[f("a", "a as u16"), f("instruction_len", "2u32")],
    },
    Stub {
        opcode: Opcode::LoadLocal3,
        family: "loads",
        semantic: "op_load_local_3_semantic",
        args: "OpLoadLocalArgs",
        layout: Layout::A,
        length: 2,
        fields: &[f("a", "a as u16"), f("instruction_len", "2u32")],
    },
    Stub {
        opcode: Opcode::StoreLocal0,
        family: "loads",
        semantic: "op_store_local_0_semantic",
        args: "OpStoreLocalArgs",
        layout: Layout::A,
        length: 2,
        fields: &[f("a", "a as u16"), f("instruction_len", "2u32")],
    },
    Stub {
        opcode: Opcode::StoreLocal1,
        family: "loads",
        semantic: "op_store_local_1_semantic",
        args: "OpStoreLocalArgs",
        layout: Layout::A,
        length: 2,
        fields: &[f("a", "a as u16"), f("instruction_len", "2u32")],
    },
    Stub {
        opcode: Opcode::StoreLocal2,
        family: "loads",
        semantic: "op_store_local_2_semantic",
        args: "OpStoreLocalArgs",
        layout: Layout::A,
        length: 2,
        fields: &[f("a", "a as u16"), f("instruction_len", "2u32")],
    },
    Stub {
        opcode: Opcode::StoreLocal3,
        family: "loads",
        semantic: "op_store_local_3_semantic",
        args: "OpStoreLocalArgs",
        layout: Layout::A,
        length: 2,
        fields: &[f("a", "a as u16"), f("instruction_len", "2u32")],
    },
    // =================================================================
    // arithmetic family — Sub / Mul / Div / Mod / Exp / Bit* / Shift* /
    // comparisons / EqualZero / Unary.
    //
    // Binary opcodes (AbcSlot, 6-byte) use `OpBinaryArgs { dst, lhs,
    // rhs, feedback_slot, instruction_len }`. The asm side passes
    // `a, b, c, slot` raw; the shim narrows + builds the
    // `Option<FeedbackSlotId>`.
    // =================================================================
    Stub {
        opcode: Opcode::Sub,
        family: "arithmetic",
        semantic: "op_sub_semantic",
        args: "OpBinaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("rhs", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::Mul,
        family: "arithmetic",
        semantic: "op_mul_semantic",
        args: "OpBinaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("rhs", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::Div,
        family: "arithmetic",
        semantic: "op_div_semantic",
        args: "OpBinaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("rhs", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::Mod,
        family: "arithmetic",
        semantic: "op_mod_semantic",
        args: "OpBinaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("rhs", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::Exp,
        family: "arithmetic",
        semantic: "op_exp_semantic",
        args: "OpBinaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("rhs", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::BitOr,
        family: "arithmetic",
        semantic: "op_bit_or_semantic",
        args: "OpBinaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("rhs", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::BitXor,
        family: "arithmetic",
        semantic: "op_bit_xor_semantic",
        args: "OpBinaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("rhs", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::BitAnd,
        family: "arithmetic",
        semantic: "op_bit_and_semantic",
        args: "OpBinaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("rhs", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::ShiftLeft,
        family: "arithmetic",
        semantic: "op_shift_left_semantic",
        args: "OpBinaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("rhs", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::ShiftRight,
        family: "arithmetic",
        semantic: "op_shift_right_semantic",
        args: "OpBinaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("rhs", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::UnsignedShiftRight,
        family: "arithmetic",
        semantic: "op_unsigned_shift_right_semantic",
        args: "OpBinaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("rhs", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::Equal,
        family: "arithmetic",
        semantic: "op_equal_semantic",
        args: "OpBinaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("rhs", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::StrictEqual,
        family: "arithmetic",
        semantic: "op_strict_equal_semantic",
        args: "OpBinaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("rhs", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::LessThan,
        family: "arithmetic",
        semantic: "op_less_than_semantic",
        args: "OpBinaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("rhs", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::LessEqual,
        family: "arithmetic",
        semantic: "op_less_equal_semantic",
        args: "OpBinaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("rhs", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::GreaterThan,
        family: "arithmetic",
        semantic: "op_greater_than_semantic",
        args: "OpBinaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("rhs", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::GreaterEqual,
        family: "arithmetic",
        semantic: "op_greater_equal_semantic",
        args: "OpBinaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("rhs", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    // Binary SMI variants — `OpBinarySmiArgs { dst, lhs, imm_raw, ...
    // }`. `c` operand is the raw u16 immediate.
    Stub {
        opcode: Opcode::AddSmi,
        family: "arithmetic",
        semantic: "op_add_smi_semantic",
        args: "OpBinarySmiArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("imm_raw", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::SubSmi,
        family: "arithmetic",
        semantic: "op_sub_smi_semantic",
        args: "OpBinarySmiArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("imm_raw", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::MulSmi,
        family: "arithmetic",
        semantic: "op_mul_smi_semantic",
        args: "OpBinarySmiArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("imm_raw", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::DivSmi,
        family: "arithmetic",
        semantic: "op_div_smi_semantic",
        args: "OpBinarySmiArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("imm_raw", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::ModSmi,
        family: "arithmetic",
        semantic: "op_mod_smi_semantic",
        args: "OpBinarySmiArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("imm_raw", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::BitAndSmi,
        family: "arithmetic",
        semantic: "op_bit_and_smi_semantic",
        args: "OpBinarySmiArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("imm_raw", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    // Unary variants — `OpUnaryArgs { dst, src, feedback_slot, instruction_len }`.
    Stub {
        opcode: Opcode::Negate,
        family: "arithmetic",
        semantic: "op_negate_semantic",
        args: "OpUnaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("src", "b as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::BitNot,
        family: "arithmetic",
        semantic: "op_bit_not_semantic",
        args: "OpUnaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("src", "b as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::Increment,
        family: "arithmetic",
        semantic: "op_increment_semantic",
        args: "OpUpdateArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("src", "b as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::Decrement,
        family: "arithmetic",
        semantic: "op_decrement_semantic",
        args: "OpUpdateArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("src", "b as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    // EqualZero — `OpEqualZeroArgs { dst, src, feedback_slot, instruction_len }`.
    Stub {
        opcode: Opcode::EqualZero,
        family: "arithmetic",
        semantic: "op_equal_zero_semantic",
        args: "OpEqualZeroArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("src", "b as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    // =================================================================
    // control_flow family — ReturnUndefined, Nop.
    // =================================================================
    Stub {
        opcode: Opcode::ReturnUndefined,
        family: "control_flow",
        semantic: "op_return_undefined_semantic",
        args: "OpReturnUndefinedArgs",
        layout: Layout::Ax,
        length: 4,
        // Unit struct — no fields to fill in.
        fields: &[],
    },
    Stub {
        opcode: Opcode::Nop,
        family: "control_flow",
        semantic: "op_nop_semantic",
        args: "OpNopArgs",
        layout: Layout::Ax,
        length: 4,
        fields: &[f("instruction_len", "4u32")],
    },
    // =================================================================
    // property family.
    //
    // OpPropertyAccessArgs (Get*/Set*/Assign*/Strict* Named/Keyed Property):
    // AbcSlot layout, length 6, feedback slot present.
    // =================================================================
    Stub {
        opcode: Opcode::GetNamedProperty,
        family: "property",
        semantic: "op_get_named_property_semantic",
        args: "OpPropertyAccessArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::SetNamedProperty,
        family: "property",
        semantic: "op_set_named_property_semantic",
        args: "OpPropertyAccessArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::AssignNamedProperty,
        family: "property",
        semantic: "op_assign_named_property_semantic",
        args: "OpPropertyAccessArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::StrictAssignNamedProperty,
        family: "property",
        semantic: "op_strict_assign_named_property_semantic",
        args: "OpPropertyAccessArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::GetKeyedProperty,
        family: "property",
        semantic: "op_get_keyed_property_semantic",
        args: "OpPropertyAccessArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::SetKeyedProperty,
        family: "property",
        semantic: "op_set_keyed_property_semantic",
        args: "OpPropertyAccessArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::AssignKeyedProperty,
        family: "property",
        semantic: "op_assign_keyed_property_semantic",
        args: "OpPropertyAccessArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::StrictAssignKeyedProperty,
        family: "property",
        semantic: "op_strict_assign_keyed_property_semantic",
        args: "OpPropertyAccessArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    // OpPropertyAbcArgs (no feedback): DefineNamedProperty, DefineKeyedProperty,
    // StoreDenseElement, LoadDenseElement, DeleteProperty, In,
    // CopyDataProperties.
    Stub {
        opcode: Opcode::DefineNamedProperty,
        family: "property",
        semantic: "op_define_named_property_semantic",
        args: "OpPropertyAbcArgs",
        layout: Layout::Abc,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::DefineKeyedProperty,
        family: "property",
        semantic: "op_define_keyed_property_semantic",
        args: "OpPropertyAbcArgs",
        layout: Layout::Abc,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::StoreDenseElement,
        family: "property",
        semantic: "op_store_dense_element_semantic",
        args: "OpPropertyAbcArgs",
        layout: Layout::Abc,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::LoadDenseElement,
        family: "property",
        semantic: "op_load_dense_element_semantic",
        args: "OpPropertyAbcArgs",
        layout: Layout::Abc,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::DeleteProperty,
        family: "property",
        semantic: "op_delete_property_semantic",
        args: "OpPropertyAbcArgs",
        layout: Layout::Abc,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::In,
        family: "property",
        semantic: "op_in_semantic",
        args: "OpPropertyAbcArgs",
        layout: Layout::Abc,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::CopyDataProperties,
        family: "property",
        semantic: "op_copy_data_properties_semantic",
        args: "OpPropertyAbcArgs",
        layout: Layout::Abc,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    // OpPropertyAbArgs: ToPropertyKey, SetFunctionName.
    Stub {
        opcode: Opcode::ToPropertyKey,
        family: "property",
        semantic: "op_to_property_key_semantic",
        args: "OpPropertyAbArgs",
        layout: Layout::Abc,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::SetFunctionName,
        family: "property",
        semantic: "op_set_function_name_semantic",
        args: "OpPropertyAbArgs",
        layout: Layout::Abc,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    // OpPropertyAbxArgs: CreateObject, CreateArray, CheckObjectCoercible,
    // ThrowIfUninitialized.
    Stub {
        opcode: Opcode::CreateObject,
        family: "property",
        semantic: "op_create_object_semantic",
        args: "OpPropertyAbxArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::CreateArray,
        family: "property",
        semantic: "op_create_array_semantic",
        args: "OpPropertyAbxArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::CheckObjectCoercible,
        family: "property",
        semantic: "op_check_object_coercible_semantic",
        args: "OpPropertyAbxArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::ThrowIfUninitialized,
        family: "property",
        semantic: "op_throw_if_uninitialized_semantic",
        args: "OpPropertyAbxArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
        ],
    },
    // =================================================================
    // names family.
    //
    // OpAtomArgs { a, bx, instruction_len, feedback_slot }: LoadGlobal /
    // StoreGlobal / AssignGlobal (with feedback, length 6); DeleteGlobal,
    // LoadName, ResolveName, ResolveGlobal, AssignName, AssignVariableName,
    // DeleteName, CaptureName, LoadThis, LoadCallee, LoadNewTarget (no
    // feedback, length 4).
    // =================================================================
    Stub {
        opcode: Opcode::LoadGlobal,
        family: "names",
        semantic: "op_load_global_semantic",
        args: "OpAtomArgs",
        layout: Layout::Abx,
        length: 6,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "6u32"),
            f("feedback_slot", "Self::feedback_slot_from_pc(state, 4)"),
        ],
    },
    Stub {
        opcode: Opcode::StoreGlobal,
        family: "names",
        semantic: "op_store_global_semantic",
        args: "OpAtomArgs",
        layout: Layout::Abx,
        length: 6,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "6u32"),
            f("feedback_slot", "Self::feedback_slot_from_pc(state, 4)"),
        ],
    },
    Stub {
        opcode: Opcode::AssignGlobal,
        family: "names",
        semantic: "op_assign_global_semantic",
        args: "OpAtomArgs",
        layout: Layout::Abx,
        length: 6,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "6u32"),
            f("feedback_slot", "Self::feedback_slot_from_pc(state, 4)"),
        ],
    },
    Stub {
        opcode: Opcode::DeleteGlobal,
        family: "names",
        semantic: "op_delete_global_semantic",
        args: "OpAtomArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
            f("feedback_slot", "None"),
        ],
    },
    Stub {
        opcode: Opcode::LoadName,
        family: "names",
        semantic: "op_load_name_semantic",
        args: "OpAtomArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
            f("feedback_slot", "None"),
        ],
    },
    Stub {
        opcode: Opcode::ResolveName,
        family: "names",
        semantic: "op_resolve_name_semantic",
        args: "OpAtomArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
            f("feedback_slot", "None"),
        ],
    },
    Stub {
        opcode: Opcode::ResolveGlobal,
        family: "names",
        semantic: "op_resolve_global_semantic",
        args: "OpAtomArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
            f("feedback_slot", "None"),
        ],
    },
    Stub {
        opcode: Opcode::AssignName,
        family: "names",
        semantic: "op_assign_name_semantic",
        args: "OpAtomArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
            f("feedback_slot", "None"),
        ],
    },
    Stub {
        opcode: Opcode::AssignVariableName,
        family: "names",
        semantic: "op_assign_variable_name_semantic",
        args: "OpAtomArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
            f("feedback_slot", "None"),
        ],
    },
    Stub {
        opcode: Opcode::DeleteName,
        family: "names",
        semantic: "op_delete_name_semantic",
        args: "OpAtomArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
            f("feedback_slot", "None"),
        ],
    },
    Stub {
        opcode: Opcode::CaptureName,
        family: "names",
        semantic: "op_capture_name_semantic",
        args: "OpAtomArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
            f("feedback_slot", "None"),
        ],
    },
    Stub {
        opcode: Opcode::LoadThis,
        family: "names",
        semantic: "op_load_this_semantic",
        args: "OpAtomArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
            f("feedback_slot", "None"),
        ],
    },
    Stub {
        opcode: Opcode::LoadCallee,
        family: "names",
        semantic: "op_load_callee_semantic",
        args: "OpAtomArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
            f("feedback_slot", "None"),
        ],
    },
    Stub {
        opcode: Opcode::LoadNewTarget,
        family: "names",
        semantic: "op_load_new_target_semantic",
        args: "OpAtomArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
            f("feedback_slot", "None"),
        ],
    },
    // OpCapturedNameArgs: LoadCapturedName, LoadCapturedNameThis,
    // AssignCapturedName.
    Stub {
        opcode: Opcode::LoadCapturedName,
        family: "names",
        semantic: "op_load_captured_name_semantic",
        args: "OpCapturedNameArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::LoadCapturedNameThis,
        family: "names",
        semantic: "op_load_captured_name_this_semantic",
        args: "OpCapturedNameArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::AssignCapturedName,
        family: "names",
        semantic: "op_assign_captured_name_semantic",
        args: "OpCapturedNameArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
        ],
    },
    // =================================================================
    // scope family.
    //
    // OpScopeAbxArgs: LoadEnvSlot, StoreEnvSlot, AssignEnvSlot,
    // EnterEnvScope, LeaveEnvScope.
    // OpScopeAxArgs: PushClosureEnv, PopClosureEnv, PushWithEnv,
    // PopWithEnv, TypeOf.
    // =================================================================
    Stub {
        opcode: Opcode::LoadEnvSlot,
        family: "scope",
        semantic: "op_load_env_slot_semantic",
        args: "OpScopeAbxArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::StoreEnvSlot,
        family: "scope",
        semantic: "op_store_env_slot_semantic",
        args: "OpScopeAbxArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::AssignEnvSlot,
        family: "scope",
        semantic: "op_assign_env_slot_semantic",
        args: "OpScopeAbxArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::EnterEnvScope,
        family: "scope",
        semantic: "op_enter_env_scope_semantic",
        args: "OpScopeAbxArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::LeaveEnvScope,
        family: "scope",
        semantic: "op_leave_env_scope_semantic",
        args: "OpScopeAbxArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::PushClosureEnv,
        family: "scope",
        semantic: "op_push_closure_env_semantic",
        args: "OpScopeAxArgs",
        layout: Layout::Ax,
        length: 4,
        fields: &[
            // sign-extend low 24 bits of the 32-bit ax read.
            f("ax", "(((ax & 0x00ff_ffff) as i32) << 8) >> 8"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::PopClosureEnv,
        family: "scope",
        semantic: "op_pop_closure_env_semantic",
        args: "OpScopeAxArgs",
        layout: Layout::Ax,
        length: 4,
        fields: &[
            f("ax", "(((ax & 0x00ff_ffff) as i32) << 8) >> 8"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::PushWithEnv,
        family: "scope",
        semantic: "op_push_with_env_semantic",
        args: "OpScopeAxArgs",
        layout: Layout::Ax,
        length: 4,
        fields: &[
            f("ax", "(((ax & 0x00ff_ffff) as i32) << 8) >> 8"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::PopWithEnv,
        family: "scope",
        semantic: "op_pop_with_env_semantic",
        args: "OpScopeAxArgs",
        layout: Layout::Ax,
        length: 4,
        fields: &[
            f("ax", "(((ax & 0x00ff_ffff) as i32) << 8) >> 8"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::TypeOf,
        family: "scope",
        semantic: "op_type_of_semantic",
        args: "OpScopeAxArgs",
        layout: Layout::Ax,
        length: 4,
        fields: &[
            f("ax", "(((ax & 0x00ff_ffff) as i32) << 8) >> 8"),
            f("instruction_len", "4u32"),
        ],
    },
    // =================================================================
    // calls family.
    //
    // Call0..3 — OpCallSmallArgs (AbcSlot, length 6). `arity` is a
    // per-opcode literal (0..=3); the others are register operands.
    // Call / Construct / TailCall — OpCallRangeArgs / OpTailCallArgs
    // (encoded length 10, layout Abc for the asm prologue — the
    // CallRange + spread_mask are stubbed to default values, since
    // the asm trampoline is not active yet and these opcodes only
    // need to link).
    // CreateClosure — OpCreateClosureArgs (Abx, length 4).
    // =================================================================
    Stub {
        opcode: Opcode::Call0,
        family: "calls",
        semantic: "op_call0_semantic",
        args: "OpCallSmallArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f("arity", "0u8"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::Call1,
        family: "calls",
        semantic: "op_call1_semantic",
        args: "OpCallSmallArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f("arity", "1u8"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::Call2,
        family: "calls",
        semantic: "op_call2_semantic",
        args: "OpCallSmallArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f("arity", "2u8"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::Call3,
        family: "calls",
        semantic: "op_call3_semantic",
        args: "OpCallSmallArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f("arity", "3u8"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    Stub {
        opcode: Opcode::Call,
        family: "calls",
        semantic: "op_call_semantic",
        args: "OpCallRangeArgs",
        layout: Layout::Abc,
        length: 10,
        // Bytecode layout (10 bytes): [op][a][b][c][count_lo][count_hi]
        // [base_lo][base_hi][slot_lo][slot_hi]. The asm prologue can
        // only forward three u32 operands (a/b/c), so the shim re-reads
        // CallRange (bytes 4..8) and feedback slot (bytes 8..10) from
        // PC — mirroring α's `decode_call_range_operands` path. The
        // shim is the live cold path post-DSL-0c.
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f("range", "Self::call_range_from_pc(state, 4)"),
            f("feedback_slot", "Self::feedback_slot_from_pc(state, 8)"),
            f("spread_mask", "Self::spread_mask_from_pc(state, 8)"),
            f("instruction_len", "10u32"),
        ],
    },
    Stub {
        opcode: Opcode::Construct,
        family: "calls",
        semantic: "op_construct_semantic",
        args: "OpCallRangeArgs",
        layout: Layout::Abc,
        length: 10,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f("range", "Self::call_range_from_pc(state, 4)"),
            f("feedback_slot", "Self::feedback_slot_from_pc(state, 8)"),
            f("spread_mask", "Self::spread_mask_from_pc(state, 8)"),
            f("instruction_len", "10u32"),
        ],
    },
    Stub {
        opcode: Opcode::TailCall,
        family: "calls",
        semantic: "op_tail_call_semantic",
        args: "OpTailCallArgs",
        layout: Layout::Abc,
        length: 10,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("range", "Self::call_range_from_pc(state, 4)"),
            f("feedback_slot", "Self::feedback_slot_from_pc(state, 8)"),
            f("spread_mask", "Self::spread_mask_from_pc(state, 8)"),
        ],
    },
    Stub {
        opcode: Opcode::CreateClosure,
        family: "calls",
        semantic: "op_create_closure_semantic",
        args: "OpCreateClosureArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
        ],
    },
    // =================================================================
    // iterators family.
    //
    // OpIteratorAbcArgs (Abc, length 4): CreateForIn, AdvanceForIn,
    // CreateIterator, AdvanceIterator.
    // OpIteratorAbxArgs (Abx, length 4): CloseForIn, CloseIterator.
    // =================================================================
    Stub {
        opcode: Opcode::CreateForIn,
        family: "iterators",
        semantic: "op_create_for_in_semantic",
        args: "OpIteratorAbcArgs",
        layout: Layout::Abc,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::AdvanceForIn,
        family: "iterators",
        semantic: "op_advance_for_in_semantic",
        args: "OpIteratorAbcArgs",
        layout: Layout::Abc,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::CloseForIn,
        family: "iterators",
        semantic: "op_close_for_in_semantic",
        args: "OpIteratorAbxArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::CreateIterator,
        family: "iterators",
        semantic: "op_create_iterator_semantic",
        args: "OpIteratorAbcArgs",
        layout: Layout::Abc,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::AdvanceIterator,
        family: "iterators",
        semantic: "op_advance_iterator_semantic",
        args: "OpIteratorAbcArgs",
        layout: Layout::Abc,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::CloseIterator,
        family: "iterators",
        semantic: "op_close_iterator_semantic",
        args: "OpIteratorAbxArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("bx", "bx"),
            f("instruction_len", "4u32"),
        ],
    },
    // =================================================================
    // generators family.
    //
    // OpSuspendGeneratorStartArgs: SuspendGeneratorStart (Ax, length 4).
    // OpGeneratorsAxArgs: Yield, Await, LoadResumeKind, LoadResumeValue
    // (Ax, length 4). `register` is masked from the low 24 bits of ax.
    // OpDelegateYieldArgs: DelegateYield (Abc, length 4).
    // =================================================================
    Stub {
        opcode: Opcode::SuspendGeneratorStart,
        family: "generators",
        semantic: "op_suspend_generator_start_semantic",
        args: "OpSuspendGeneratorStartArgs",
        layout: Layout::Ax,
        length: 4,
        fields: &[f("instruction_len", "4u32")],
    },
    Stub {
        opcode: Opcode::Yield,
        family: "generators",
        semantic: "op_yield_semantic",
        args: "OpGeneratorsAxArgs",
        layout: Layout::Ax,
        length: 4,
        fields: &[
            f("register", "(ax & 0x00ff_ffff) as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::Await,
        family: "generators",
        semantic: "op_await_semantic",
        args: "OpGeneratorsAxArgs",
        layout: Layout::Ax,
        length: 4,
        fields: &[
            f("register", "(ax & 0x00ff_ffff) as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::DelegateYield,
        family: "generators",
        semantic: "op_delegate_yield_semantic",
        args: "OpDelegateYieldArgs",
        layout: Layout::Abc,
        length: 4,
        fields: &[
            f("a", "a as u16"),
            f("b", "b as u16"),
            f("c", "c as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::LoadResumeKind,
        family: "generators",
        semantic: "op_load_resume_kind_semantic",
        args: "OpGeneratorsAxArgs",
        layout: Layout::Ax,
        length: 4,
        fields: &[
            f("register", "(ax & 0x00ff_ffff) as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::LoadResumeValue,
        family: "generators",
        semantic: "op_load_resume_value_semantic",
        args: "OpGeneratorsAxArgs",
        layout: Layout::Ax,
        length: 4,
        fields: &[
            f("register", "(ax & 0x00ff_ffff) as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    // =================================================================
    // exceptions family.
    //
    // OpExceptionsAxArgs: Throw, LoadException (Ax, length 4).
    // OpHandlerMarkerArgs: EnterHandler, LeaveHandler (Ax, length 4).
    // =================================================================
    Stub {
        opcode: Opcode::Throw,
        family: "exceptions",
        semantic: "op_throw_semantic",
        args: "OpExceptionsAxArgs",
        layout: Layout::Ax,
        length: 4,
        fields: &[
            f("register", "(ax & 0x00ff_ffff) as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    Stub {
        opcode: Opcode::EnterHandler,
        family: "exceptions",
        semantic: "op_enter_handler_semantic",
        args: "OpHandlerMarkerArgs",
        layout: Layout::Ax,
        length: 4,
        fields: &[f("instruction_len", "4u32")],
    },
    Stub {
        opcode: Opcode::LeaveHandler,
        family: "exceptions",
        semantic: "op_leave_handler_semantic",
        args: "OpHandlerMarkerArgs",
        layout: Layout::Ax,
        length: 4,
        fields: &[f("instruction_len", "4u32")],
    },
    Stub {
        opcode: Opcode::LoadException,
        family: "exceptions",
        semantic: "op_load_exception_semantic",
        args: "OpExceptionsAxArgs",
        layout: Layout::Ax,
        length: 4,
        fields: &[
            f("register", "(ax & 0x00ff_ffff) as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    // =================================================================
    // misc orphans — InstanceOf, CallMethod. Unit-struct args.
    // =================================================================
    Stub {
        opcode: Opcode::InstanceOf,
        family: "misc",
        semantic: "op_instance_of_semantic",
        args: "OpMiscStubArgs",
        layout: Layout::Ax,
        length: 4,
        fields: &[],
    },
    Stub {
        opcode: Opcode::CallMethod,
        family: "misc",
        semantic: "op_call_method_semantic",
        args: "OpMiscStubArgs",
        layout: Layout::Ax,
        length: 4,
        fields: &[],
    },
];

/// Convert an `Opcode` variant to its `snake_case` opcode name (the
/// form used as the function name prefix: `op_load_global` etc.).
///
/// The opcode-name convention is hand-tuned to match the manifest:
/// `LoadLocal0` → `op_load_local_0` (underscore before digit suffix
/// that denotes a fixed local index), but `Call0` → `op_call0`
/// (`Call0..3` are arity-mnemonic suffixes, not generic indices).
/// Similarly `Smi8` / `Const8` / `Jump8` etc. keep `8` glued because
/// the `8` is part of the type tag, not an index. The exception
/// table below enumerates the underscored cases explicitly.
fn opcode_snake_name(op: Opcode) -> String {
    // Hand-coded snake-case mapping per the manifest's existing
    // semantic-symbol names. Single source of truth: the
    // `semantic_symbol` strings in
    // `crates/vm/src/dsl/opcode_manifest.rs`.
    match op {
        Opcode::Star0 => "op_star_0".to_string(),
        Opcode::Star1 => "op_star_1".to_string(),
        Opcode::Star2 => "op_star_2".to_string(),
        Opcode::Star3 => "op_star_3".to_string(),
        Opcode::Star4 => "op_star_4".to_string(),
        Opcode::Star5 => "op_star_5".to_string(),
        Opcode::Star6 => "op_star_6".to_string(),
        Opcode::Star7 => "op_star_7".to_string(),
        Opcode::LoadLocal0 => "op_load_local_0".to_string(),
        Opcode::LoadLocal1 => "op_load_local_1".to_string(),
        Opcode::LoadLocal2 => "op_load_local_2".to_string(),
        Opcode::LoadLocal3 => "op_load_local_3".to_string(),
        Opcode::StoreLocal0 => "op_store_local_0".to_string(),
        Opcode::StoreLocal1 => "op_store_local_1".to_string(),
        Opcode::StoreLocal2 => "op_store_local_2".to_string(),
        Opcode::StoreLocal3 => "op_store_local_3".to_string(),
        _ => {
            // Default: insert `_` between camel-case word boundaries,
            // leave digits attached.
            let camel = op.name();
            let mut out = String::with_capacity(camel.len() + 4);
            out.push_str("op_");
            for (i, ch) in camel.chars().enumerate() {
                if ch.is_ascii_uppercase() {
                    if i > 0 {
                        out.push('_');
                    }
                    out.push(ch.to_ascii_lowercase());
                } else {
                    out.push(ch);
                }
            }
            out
        }
    }
}

/// Build the header comment + imports for the emitted file.
fn write_header(out: &mut String) {
    out.push_str("//! Cold DSL handlers. Auto-generated by `lyng-dsl-codegen` (B46–B47).\n");
    out.push_str("//!\n");
    out.push_str("//! Each Cold opcode gets:\n");
    out.push_str("//! 1. A `#[unsafe(naked)] extern \"C\" fn op_xxx_dsl()` handler\n");
    out.push_str("//!    emitted by `llint_handler!`. The body is a single\n");
    out.push_str("//!    `call_slow!(op_xxx_slow_rs, args = [...]); dispatch_after_slow!();`\n");
    out.push_str("//!    pair — every cold opcode pays the slow-path round-trip.\n");
    out.push_str("//! 2. A `extern \"C\" fn op_xxx_slow_rs(state, ...) -> SlowPathReturn`\n");
    out.push_str("//!    shim that reconstructs the args struct from raw u32 operand\n");
    out.push_str("//!    slots, calls the semantic body in\n");
    out.push_str("//!    `crate::vm::semantics::<family>::op_xxx_semantic`, and\n");
    out.push_str("//!    translates the outcome back to `SlowPathReturn`.\n");
    out.push_str("//!\n");
    out.push_str("//! DO NOT EDIT BY HAND. Re-run `cargo run -p lyng-dsl-codegen`\n");
    out.push_str("//! after touching `tools/lyng-dsl-codegen/src/main.rs`.\n");
    out.push('\n');

    // aarch64 brings in every macro the proc-macro-emitted handlers
    // reference plus the layout-decode prologue macros.
    out.push_str("#[cfg(target_arch = \"aarch64\")]\n");
    out.push_str("use crate::{\n");
    out.push_str("    call_slow, decode_a, decode_ab, decode_abc, decode_abc_slot, decode_abx,\n");
    out.push_str("    decode_ax, dispatch_after_slow,\n");
    out.push_str("};\n");
    out.push('\n');
    out.push_str("#[cfg(target_arch = \"aarch64\")]\n");
    out.push_str("use lyng_vm_dsl::llint_handler;\n");
    out.push('\n');

    // Helper for the feedback-slot extraction shared by LoadGlobal /
    // StoreGlobal / AssignGlobal. The asm prologue only reads 4 bytes
    // (Abx layout), but the actual encoding is 6 bytes — the slot
    // lives at PC+4..PC+6. The shim reads it from the post-sync
    // frame state via `current_instruction_offset` + bytecode lookup.
    //
    // Wrapped in a struct namespace (`Self::feedback_slot_from_pc`)
    // so the per-shim metadata references it uniformly.
    out.push_str("/// Helper namespace for cold-stub shim utilities.\n");
    out.push_str("#[cfg(target_arch = \"aarch64\")]\n");
    out.push_str("struct ColdShimHelpers;\n");
    out.push('\n');
    out.push_str("#[cfg(target_arch = \"aarch64\")]\n");
    out.push_str("impl ColdShimHelpers {\n");
    out.push_str("    /// Read a 2-byte feedback-slot index from `[PC + offset]`\n");
    out.push_str("    /// of the active frame's bytecode, returning the typed slot id.\n");
    out.push_str("    /// Used by IC-bearing opcodes whose asm prologue can't fit\n");
    out.push_str("    /// the slot operand (e.g. `LoadGlobal`'s Abx layout reads\n");
    out.push_str("    /// the first 4 bytes; the slot is at bytes 4..6).\n");
    out.push_str("    ///\n");
    out.push_str("    /// `FeedbackSlotId::from_raw(0)` returns `None` (slot 0 is\n");
    out.push_str("    /// the sentinel for \"no feedback\"); a non-zero raw index\n");
    out.push_str("    /// returns `Some`. Either way the caller receives an\n");
    out.push_str("    /// `Option<FeedbackSlotId>` that maps directly into the\n");
    out.push_str("    /// `feedback_slot` field on the args struct.\n");
    out.push_str("    #[inline]\n");
    out.push_str("    fn feedback_slot_from_pc(\n");
    out.push_str("        state: &mut crate::dsl::slow_path::LlIntDispatchState<'_, '_>,\n");
    out.push_str("        offset: u32,\n");
    out.push_str("    ) -> Option<lyng_types::FeedbackSlotId> {\n");
    out.push_str("        let inner = state.dispatch_state();\n");
    out.push_str("        let pc = inner.frame.instruction_offset();\n");
    out.push_str("        // SAFETY: `installed.function.instruction_bytes()` is a\n");
    out.push_str("        // pub(crate) field reachable through the active frame's\n");
    out.push_str("        // installed function (held in `DispatchState.installed`).\n");
    out.push_str("        let bytes = inner.installed.function().instruction_bytes();\n");
    out.push_str("        let lo = bytes.get((pc + offset) as usize).copied()? as u32;\n");
    out.push_str("        let hi = bytes.get((pc + offset + 1) as usize).copied()? as u32;\n");
    out.push_str("        let raw = lo | (hi << 8);\n");
    out.push_str("        lyng_types::FeedbackSlotId::from_raw(raw)\n");
    out.push_str("    }\n");
    out.push('\n');
    out.push_str("    /// Read the inline `CallRange` (4 bytes — count, base) from the\n");
    out.push_str("    /// current instruction at `[PC + offset]`. Used by the variable-arity\n");
    out.push_str("    /// `Call` / `TailCall` / `Construct` cold shims to mirror the α\n");
    out.push_str("    /// handler's `decode_call_range_operands` path. Layout per\n");
    out.push_str("    /// `decode_call_range_operands`: bytes 4,5 = count_lo/hi; bytes 6,7 =\n");
    out.push_str("    /// base_lo/hi. The slot operand follows at bytes 8,9.\n");
    out.push_str("    #[inline]\n");
    out.push_str("    fn call_range_from_pc(\n");
    out.push_str("        state: &mut crate::dsl::slow_path::LlIntDispatchState<'_, '_>,\n");
    out.push_str("        offset: u32,\n");
    out.push_str("    ) -> lyng_bytecode::CallRange {\n");
    out.push_str("        let inner = state.dispatch_state();\n");
    out.push_str("        let pc = inner.frame.instruction_offset();\n");
    out.push_str("        let bytes = inner.installed.function().instruction_bytes();\n");
    out.push_str(
        "        let count_lo = bytes.get((pc + offset) as usize).copied().unwrap_or(0);\n",
    );
    out.push_str(
        "        let count_hi = bytes.get((pc + offset + 1) as usize).copied().unwrap_or(0);\n",
    );
    out.push_str(
        "        let base_lo = bytes.get((pc + offset + 2) as usize).copied().unwrap_or(0);\n",
    );
    out.push_str(
        "        let base_hi = bytes.get((pc + offset + 3) as usize).copied().unwrap_or(0);\n",
    );
    out.push_str("        let count = u16::from_le_bytes([count_lo, count_hi]);\n");
    out.push_str("        let base = u16::from_le_bytes([base_lo, base_hi]);\n");
    out.push_str("        lyng_bytecode::CallRange::new(base, count)\n");
    out.push_str("    }\n");
    out.push('\n');
    out.push_str("    /// Look up the `spread_mask` metadata for an optional feedback slot.\n");
    out.push_str("    /// Mirrors `calls::spread_mask_for_semantic` in the α path: returns\n");
    out.push_str("    /// `None` when there's no slot or no spread metadata.\n");
    out.push_str("    #[inline]\n");
    out.push_str("    fn spread_mask_for(\n");
    out.push_str("        state: &mut crate::dsl::slow_path::LlIntDispatchState<'_, '_>,\n");
    out.push_str("        feedback_slot: Option<lyng_types::FeedbackSlotId>,\n");
    out.push_str("    ) -> Option<u64> {\n");
    out.push_str("        let slot = feedback_slot?;\n");
    out.push_str("        let inner = state.dispatch_state();\n");
    out.push_str("        let descriptor = inner.installed.feedback_descriptor_for_slot(slot)?;\n");
    out.push_str("        descriptor.metadata().spread_mask()\n");
    out.push_str("    }\n");
    out.push('\n');
    out.push_str("    /// Convenience: look up `spread_mask` directly from the feedback\n");
    out.push_str("    /// slot at `[PC + offset]`. Combines `feedback_slot_from_pc` with\n");
    out.push_str("    /// `spread_mask_for` so the cold shim can express the lookup as a\n");
    out.push_str("    /// single field expression in the args struct literal (without\n");
    out.push_str("    /// needing a let-binding for the intermediate slot).\n");
    out.push_str("    #[inline]\n");
    out.push_str("    fn spread_mask_from_pc(\n");
    out.push_str("        state: &mut crate::dsl::slow_path::LlIntDispatchState<'_, '_>,\n");
    out.push_str("        offset: u32,\n");
    out.push_str("    ) -> Option<u64> {\n");
    out.push_str("        let feedback_slot = Self::feedback_slot_from_pc(state, offset);\n");
    out.push_str("        Self::spread_mask_for(state, feedback_slot)\n");
    out.push_str("    }\n");
    out.push_str("}\n");
    out.push('\n');
}

/// Which prefix-aware decode helper to emit for an opcode (DSL-0c C2).
///
/// Only opcodes with bytecode instruction form `Abc` or `Abx` accept a
/// `Wide` / `ExtraWide` prefix; all other forms reject prefix at decode
/// time. The codegen emits a wide-form dispatch match arm for each
/// `Abc`/`Abx`-form opcode (cold + hot/warm); the resulting function
/// `dispatch_wide_form` replaces the legacy `op_prefix_via_alpha`
/// bridge that delegated wide-form dispatch through the α dispatch
/// table.
#[derive(Clone, Copy, Debug)]
enum PrefixKind {
    /// `decode_abc_operands` — three register operands, optional inline
    /// feedback slot. Covers `Layout::Abc, length=4` (no slot) and
    /// `Layout::AbcSlot, length=6` (slot).
    Abc,
    /// `decode_abx_operands` — one register operand + one immediate (Bx),
    /// optional inline feedback slot. Covers `Layout::Abx, length=4`
    /// (no slot) and `Layout::Abx, length=6` (slot).
    Abx,
}

/// Classify a stub: returns `Some(PrefixKind)` when the stub's bytecode
/// form accepts a `Wide` / `ExtraWide` prefix, `None` otherwise.
///
/// `Layout::Abc, length=10` is CallRange (`Call` / `TailCall` /
/// `Construct`), which the bytecode decoder explicitly rejects prefixes
/// for. Treat as non-prefix-aware here.
fn prefix_kind_for(layout: Layout, length: u32) -> Option<PrefixKind> {
    match (layout, length) {
        (Layout::Abc, 4) | (Layout::AbcSlot, 6) => Some(PrefixKind::Abc),
        (Layout::Abx, 4) | (Layout::Abx, 6) => Some(PrefixKind::Abx),
        _ => None,
    }
}

/// Whether the stub's bytecode form carries a feedback slot.
fn is_profiled_for(layout: Layout, length: u32) -> bool {
    matches!((layout, length), (Layout::AbcSlot, _) | (Layout::Abx, 6))
}

/// Hot/warm opcodes the codegen does NOT own — their asm/Rust handlers
/// live in `dsl/handlers/hot.rs` and `dsl/handlers/warm.rs` — but whose
/// bytecode form accepts a prefix and which therefore need an entry in
/// the wide-form dispatch match. Manually mirrored from those files so
/// the wide-form dispatcher covers every prefix-accepting opcode.
const HOT_WARM_STUBS: &[Stub] = &[
    // hot.rs — op_move: Layout::Ab in asm (operand_names = ["a", "b"]),
    // but the bytecode form is Abc (3 operands; the third is unused at
    // narrow encoding). For wide-form decoding we treat it as Abc so
    // `decode_abc_operands` reads both registers correctly.
    Stub {
        opcode: Opcode::Move,
        family: "loads",
        semantic: "op_move_semantic",
        args: "OpMoveArgs",
        layout: Layout::Abc,
        length: 4,
        fields: &[
            f("dst", "a as u16"),
            f("src", "b as u16"),
            f("instruction_len", "4u32"),
        ],
    },
    // hot.rs — op_add: Abc + feedback slot.
    Stub {
        opcode: Opcode::Add,
        family: "arithmetic",
        semantic: "op_add_semantic",
        args: "OpBinaryArgs",
        layout: Layout::AbcSlot,
        length: 6,
        fields: &[
            f("dst", "a as u16"),
            f("lhs", "b as u16"),
            f("rhs", "c as u16"),
            f(
                "feedback_slot",
                "lyng_types::FeedbackSlotId::from_raw(slot)",
            ),
            f("instruction_len", "6u32"),
        ],
    },
    // warm.rs — op_jump_if_true: Abx (register + i16 delta), no slot.
    Stub {
        opcode: Opcode::JumpIfTrue,
        family: "control_flow",
        semantic: "op_jump_if_true_semantic",
        args: "OpJumpIfArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("condition_register", "a as u16"),
            f("delta", "(bx as i16) as i32"),
            f("instruction_len", "4u32"),
        ],
    },
    // warm.rs — op_jump_if_false: same shape as op_jump_if_true.
    Stub {
        opcode: Opcode::JumpIfFalse,
        family: "control_flow",
        semantic: "op_jump_if_false_semantic",
        args: "OpJumpIfArgs",
        layout: Layout::Abx,
        length: 4,
        fields: &[
            f("condition_register", "a as u16"),
            f("delta", "(bx as i16) as i32"),
            f("instruction_len", "4u32"),
        ],
    },
];

/// Emit the centralized wide-form dispatcher `dispatch_wide_form`.
///
/// The asm-DSL `op_wide` / `op_extra_wide` shims call this function with
/// the prefix opcode and a `&mut LlIntDispatchState`. It reads the
/// semantic byte at `bytes[pc+1]`, dispatches to the matching opcode's
/// decoder + semantic call, and returns the resulting `SemanticOutcome`
/// (with `pc_advance` = full wide-form instruction length). The caller
/// translates the outcome via `LlIntDispatchState::translate_outcome`
/// so the asm bridge advances PC past the entire wide instruction.
///
/// Replaces the legacy `op_prefix_via_alpha` bridge that delegated
/// wide-form dispatch through the α dispatch table.
fn write_wide_form_dispatcher(out: &mut String, cold_stubs: &[Stub]) {
    let mut all_stubs: Vec<&Stub> = cold_stubs.iter().collect();
    all_stubs.extend(HOT_WARM_STUBS.iter());

    out.push_str("// =====================================================================\n");
    out.push_str("// dispatch_wide_form — centralized wide-form instruction dispatcher\n");
    out.push_str("// (DSL-0c C2 replacement for the α `op_prefix_via_alpha` bridge).\n");
    out.push_str("// =====================================================================\n\n");

    out.push_str("/// Centralized wide-form dispatcher invoked by `op_wide` /\n");
    out.push_str("/// `op_extra_wide`'s DSL slow-path shims. Reads the semantic byte\n");
    out.push_str("/// at `bytes[pc+1]`, decodes the wide-form operands via\n");
    out.push_str("/// `crate::vm::dispatch::decode_*_operands`, calls the matching\n");
    out.push_str("/// semantic body, and returns the resulting `SemanticOutcome`\n");
    out.push_str("/// (with `pc_advance` = full wide-form instruction length).\n");
    out.push_str("///\n");
    out.push_str("/// Auto-generated from `tools/lyng-dsl-codegen/src/main.rs`'s\n");
    out.push_str("/// `COLD_STUBS` + `HOT_WARM_STUBS` tables.\n");
    out.push_str("#[cfg(target_arch = \"aarch64\")]\n");
    // The match arms unpack the full decoder tuple even when the
    // corresponding `is_profiled` branch is false (no slot field on the
    // args struct). Suppress the per-arm "unused variable" lints at the
    // function level rather than littering the emitted match with
    // per-arm `_` prefixes.
    out.push_str("#[allow(unused_variables)]\n");
    out.push_str("pub(crate) fn dispatch_wide_form(\n");
    out.push_str("    dispatch: &mut crate::dsl::slow_path::LlIntDispatchState<'_, '_>,\n");
    out.push_str("    prefix: lyng_bytecode::Opcode,\n");
    out.push_str(") -> crate::dsl::slow_path::SemanticOutcome {\n");
    out.push_str("    use crate::dsl::slow_path::SemanticOutcome;\n");
    out.push_str("    use crate::error::VmError;\n");
    out.push_str("    use lyng_bytecode::Opcode;\n");
    out.push_str("    // Peek the semantic byte at bytes[pc+1] without holding\n");
    out.push_str("    // a `&` borrow of `dispatch` across the match — the per-\n");
    out.push_str("    // opcode arms borrow `dispatch` mutably to call the\n");
    out.push_str("    // semantic body. Each arm re-acquires the byte slice\n");
    out.push_str("    // through `dispatch.dispatch_state()` after the borrow is\n");
    out.push_str("    // released; the `bytes.to_vec()` allocation in the prior\n");
    out.push_str("    // shape would have shown up as a per-wide-instruction\n");
    out.push_str("    // hot-loop cost in profiling.\n");
    out.push_str("    let (pc, code, semantic_byte) = {\n");
    out.push_str("        let inner = dispatch.dispatch_state();\n");
    out.push_str("        let pc = inner.frame.instruction_offset();\n");
    out.push_str("        let code = inner.frame.code();\n");
    out.push_str("        let full_bytes = inner.installed.function().instruction_bytes();\n");
    out.push_str("        let bytes = &full_bytes[pc as usize..];\n");
    out.push_str("        let sb = match bytes.get(1).copied() {\n");
    out.push_str("            Some(b) => b,\n");
    out.push_str(
        "            None => return SemanticOutcome::ExitError { error: VmError::InstructionOutOfBounds { code, instruction_offset: pc } },\n",
    );
    out.push_str("        };\n");
    out.push_str("        (pc, code, sb)\n");
    out.push_str("    };\n");
    out.push_str("    let semantic_opcode = match Opcode::from_byte(semantic_byte) {\n");
    out.push_str("        Some(op) => op,\n");
    out.push_str(
        "        None => return SemanticOutcome::ExitError { error: VmError::InstructionOutOfBounds { code, instruction_offset: pc } },\n",
    );
    out.push_str("    };\n");
    out.push_str("    match semantic_opcode {\n");

    // One arm per prefix-accepting opcode.
    let mut emitted: Vec<u8> = Vec::new();
    for stub in &all_stubs {
        let Some(kind) = prefix_kind_for(stub.layout, stub.length) else {
            continue;
        };
        let op_byte = stub.opcode as u8;
        if emitted.contains(&op_byte) {
            continue;
        }
        emitted.push(op_byte);
        let profiled = is_profiled_for(stub.layout, stub.length);
        let opcode_variant = stub.opcode.name();
        match kind {
            PrefixKind::Abc => {
                writeln!(out, "        Opcode::{opcode_variant} => {{").unwrap();
                // Re-acquire the bytes slice from `dispatch` in each arm
                // so we never hold an immutable borrow across the
                // semantic-body mutable call below.
                out.push_str("            let decoded = {\n");
                out.push_str("                let inner = dispatch.dispatch_state();\n");
                out.push_str(
                    "                let bytes = &inner.installed.function().instruction_bytes()[pc as usize..];\n",
                );
                writeln!(
                    out,
                    "                crate::vm::dispatch::decode_abc_operands(bytes, Some(prefix), {profiled}, code, pc)"
                )
                .unwrap();
                out.push_str("            };\n");
                out.push_str("            match decoded {\n");
                out.push_str(
                    "                Ok((a16, b16, c16, slot_opt, instruction_len)) => {\n",
                );
                // Bind to names the existing field expressions expect:
                // `a`, `b`, `c` as u32 (matching asm-passed convention);
                // `slot` as raw u32 for `FeedbackSlotId::from_raw(slot)`
                // field expressions.
                out.push_str("                    let a: u32 = a16 as u32;\n");
                out.push_str("                    let b: u32 = b16 as u32;\n");
                out.push_str("                    let c: u32 = c16 as u32;\n");
                if profiled {
                    out.push_str(
                        "                    let slot: u32 = slot_opt.map_or(0u32, |s| s.raw().get());\n",
                    );
                }
                writeln!(
                    out,
                    "                    let args = crate::vm::semantics::{family}::{args_ty} {{",
                    family = stub.family,
                    args_ty = stub.args,
                )
                .unwrap();
                for field in stub.fields {
                    if field.name == "instruction_len" {
                        out.push_str("                        instruction_len: instruction_len,\n");
                        continue;
                    }
                    let expr = field
                        .expr
                        .replace("state", "dispatch")
                        .replace("Self::", "ColdShimHelpers::");
                    writeln!(
                        out,
                        "                        {name}: {expr},",
                        name = field.name,
                        expr = expr,
                    )
                    .unwrap();
                }
                out.push_str("                    };\n");
                writeln!(
                    out,
                    "                    crate::vm::semantics::{family}::{semantic}(dispatch, args)",
                    family = stub.family,
                    semantic = stub.semantic,
                )
                .unwrap();
                out.push_str("                }\n");
                out.push_str(
                    "                Err(error) => SemanticOutcome::ExitError { error },\n",
                );
                out.push_str("            }\n");
                out.push_str("        }\n");
            }
            PrefixKind::Abx => {
                writeln!(out, "        Opcode::{opcode_variant} => {{").unwrap();
                out.push_str("            let decoded = {\n");
                out.push_str("                let inner = dispatch.dispatch_state();\n");
                out.push_str(
                    "                let bytes = &inner.installed.function().instruction_bytes()[pc as usize..];\n",
                );
                writeln!(
                    out,
                    "                crate::vm::dispatch::decode_abx_operands(bytes, Some(prefix), {profiled}, code, pc)"
                )
                .unwrap();
                out.push_str("            };\n");
                out.push_str("            match decoded {\n");
                out.push_str("                Ok((a16, bx_val, slot_opt, instruction_len)) => {\n");
                out.push_str("                    let a: u32 = a16 as u32;\n");
                out.push_str("                    let bx: u32 = bx_val;\n");
                if profiled {
                    // For Abx length=6, the slot is decoded by the helper.
                    // For narrow path the args expects `Option<FeedbackSlotId>`
                    // via `Self::feedback_slot_from_pc(state, 4)`; the wide
                    // path overrides that expr to use the decoded slot.
                    out.push_str("                    let feedback_slot = slot_opt;\n");
                }
                writeln!(
                    out,
                    "                    let args = crate::vm::semantics::{family}::{args_ty} {{",
                    family = stub.family,
                    args_ty = stub.args,
                )
                .unwrap();
                for field in stub.fields {
                    if field.name == "instruction_len" {
                        out.push_str("                        instruction_len: instruction_len,\n");
                        continue;
                    }
                    if field.name == "feedback_slot" && profiled {
                        out.push_str("                        feedback_slot: feedback_slot,\n");
                        continue;
                    }
                    let expr = field
                        .expr
                        .replace("state", "dispatch")
                        .replace("Self::", "ColdShimHelpers::");
                    writeln!(
                        out,
                        "                        {name}: {expr},",
                        name = field.name,
                        expr = expr,
                    )
                    .unwrap();
                }
                out.push_str("                    };\n");
                writeln!(
                    out,
                    "                    crate::vm::semantics::{family}::{semantic}(dispatch, args)",
                    family = stub.family,
                    semantic = stub.semantic,
                )
                .unwrap();
                out.push_str("                }\n");
                out.push_str(
                    "                Err(error) => SemanticOutcome::ExitError { error },\n",
                );
                out.push_str("            }\n");
                out.push_str("        }\n");
            }
        }
    }

    // Fallback: any non-prefix-accepting opcode is an error.
    out.push_str(
        "        _ => SemanticOutcome::ExitError { error: VmError::DoublePrefix { code, instruction_offset: pc } },\n",
    );
    out.push_str("    }\n");
    out.push_str("}\n\n");
}

/// Emit one `llint_handler!` block + its `op_xxx_slow_rs` shim.
fn write_stub(out: &mut String, stub: &Stub) {
    let opcode_name = opcode_snake_name(stub.opcode);
    let dsl_handler = format!("{opcode_name}_dsl");
    let slow_shim = format!("{opcode_name}_slow_rs");

    let operand_idents = stub.layout.operand_names();
    let operand_list = operand_idents.join(", ");
    let args_list = if operand_idents.is_empty() {
        "[]".to_string()
    } else {
        format!("[{operand_list}]")
    };
    let pipe_args = if operand_idents.is_empty() {
        "||".to_string()
    } else {
        format!("|{operand_list}|")
    };

    // =====================================================================
    let title = format!("// {}", stub.opcode.name());
    let underline = "// =====================================================================";
    writeln!(out, "{underline}").unwrap();
    writeln!(out, "{title}").unwrap();
    writeln!(out, "{underline}").unwrap();
    out.push('\n');

    // Emit `llint_handler!` body. The `opcode_byte = N` arg encodes
    // the discriminant of the matching `Opcode` variant so the proc-
    // macro lowerer can splice it into the leading
    // `inc_dispatch_counter!(N)` body fragment (no-op when the
    // `opcode-counters` feature is off; bumps `Vm::dispatch_counters`'
    // dispatch bank otherwise).
    let opcode_byte = stub.opcode as u8;
    out.push_str("#[cfg(target_arch = \"aarch64\")]\n");
    out.push_str("llint_handler! {\n");
    writeln!(
        out,
        "    {dsl}, opcode_byte = {opcode_byte}, layout = {layout}, length = {length}, {pipe} {{",
        dsl = dsl_handler,
        opcode_byte = opcode_byte,
        layout = stub.layout.ident(),
        length = stub.length,
        pipe = pipe_args,
    )
    .unwrap();
    writeln!(out, "        call_slow!({slow_shim}, args = {args_list});").unwrap();
    out.push_str("        dispatch_after_slow!();\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    // Emit slow-path shim signature. The shim accepts every operand
    // the asm side passes regardless of whether the args struct reads
    // it (e.g. `op_throw`'s asm reads the full 32-bit Ax word but the
    // semantic only takes the low 16 bits as `register`). Suppress
    // the unused-variable warning at the function level — picking
    // which operands are actually used per-opcode would bloat the
    // metadata table without buying anything.
    out.push_str("#[cfg(target_arch = \"aarch64\")]\n");
    out.push_str("#[allow(unused_variables)]\n");
    out.push_str("#[unsafe(no_mangle)]\n");
    writeln!(out, "pub extern \"C\" fn {slow_shim}(").unwrap();
    out.push_str("    state: *mut crate::dsl::llint_state::LlIntState,\n");
    for name in operand_idents {
        writeln!(out, "    {name}: u32,").unwrap();
    }
    out.push_str(") -> crate::dsl::slow_path::SlowPathReturn {\n");
    out.push_str(
        "    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };\n",
    );
    out.push_str("    dispatch.sync_from_asm();\n");

    // Build args struct, with field exprs that read operand names + a
    // `state: &mut dispatch` reference if the field expr needs it.
    // We swap "state" → "&mut dispatch" in the helper call below by
    // emitting the field expression verbatim; helper exprs already
    // accept `state` (which we pass via a let-binding above).
    if stub.fields.is_empty() {
        writeln!(
            out,
            "    let args = crate::vm::semantics::{family}::{args_ty};",
            family = stub.family,
            args_ty = stub.args,
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "    let args = crate::vm::semantics::{family}::{args_ty} {{",
            family = stub.family,
            args_ty = stub.args,
        )
        .unwrap();
        for field in stub.fields {
            // Inline the field expression. The expressions may
            // reference `state` (e.g.
            // `Self::feedback_slot_from_pc(state, 4)`) — rewrite to
            // `&mut dispatch` since that's the binding in scope.
            let expr = field
                .expr
                .replace("state", "&mut dispatch")
                .replace("Self::", "ColdShimHelpers::");
            writeln!(
                out,
                "        {name}: {expr},",
                name = field.name,
                expr = expr
            )
            .unwrap();
        }
        out.push_str("    };\n");
    }

    writeln!(
        out,
        "    let outcome = crate::vm::semantics::{family}::{semantic}(&mut dispatch, args);",
        family = stub.family,
        semantic = stub.semantic,
    )
    .unwrap();
    out.push_str("    dispatch.translate_outcome(outcome)\n");
    out.push_str("}\n\n");
}

/// Emit non-aarch64 fallback stubs at the bottom of the file — each
/// cold handler gets a placeholder `pub unsafe extern "C" fn` so the
/// dispatch table can still be assembled on host platforms (x86_64
/// dev machines, etc.).
fn write_non_aarch64_stubs(out: &mut String, stubs: &[Stub]) {
    out.push_str("// =====================================================================\n");
    out.push_str("// Non-aarch64 stubs (link-only placeholders).\n");
    out.push_str("// =====================================================================\n\n");
    for stub in stubs {
        let opcode_name = opcode_snake_name(stub.opcode);
        let dsl_handler = format!("{opcode_name}_dsl");
        out.push_str("#[cfg(not(target_arch = \"aarch64\"))]\n");
        writeln!(out, "pub unsafe extern \"C\" fn {dsl_handler}() -> ! {{").unwrap();
        out.push_str("    loop {}\n");
        out.push_str("}\n\n");
    }
}

fn main() {
    // Verify the table covers every Cold opcode exactly once.
    let mut covered: Vec<Opcode> = COLD_STUBS.iter().map(|s| s.opcode).collect();
    covered.sort_by_key(|op| *op as u8);
    covered.dedup_by_key(|op| *op as u8);
    if covered.len() != COLD_STUBS.len() {
        panic!(
            "COLD_STUBS contains duplicate opcode entries (got {}, deduped {})",
            COLD_STUBS.len(),
            covered.len(),
        );
    }
    eprintln!("[codegen] {} cold stubs queued", COLD_STUBS.len());

    // Cross-check every stub's `length` against the canonical narrow
    // encoded length from `Opcode::encoded_len()`. A mismatch here means
    // the emitted handler would advance PC by the wrong number of bytes,
    // misaligning subsequent dispatch (cf. the op_move length=3 bug
    // fixed in commit "DSL-0c: fix op_move length (3 → 4)…"). Catching
    // this at codegen time prevents the broken cold.rs from ever
    // reaching the workspace.
    let mut mismatches: Vec<(Opcode, u32, u32)> = Vec::new();
    for stub in COLD_STUBS {
        let canonical = u32::from(stub.opcode.encoded_len());
        if stub.length != canonical {
            mismatches.push((stub.opcode, stub.length, canonical));
        }
    }
    if !mismatches.is_empty() {
        eprintln!(
            "[codegen] FATAL: {} length mismatches found:",
            mismatches.len()
        );
        for (op, declared, canonical) in &mismatches {
            eprintln!(
                "  - {:?} (discriminant {}): declared length = {}, canonical = {}",
                op, *op as u8, declared, canonical,
            );
        }
        panic!(
            "COLD_STUBS metadata has {} length mismatch(es) — fix the `length:` field(s) above and re-run",
            mismatches.len(),
        );
    }
    eprintln!("[codegen] length-consistency check passed");

    // Sort by opcode discriminant for deterministic output.
    let mut sorted: Vec<&Stub> = COLD_STUBS.iter().collect();
    sorted.sort_by_key(|s| s.opcode as u8);

    let mut out = String::with_capacity(64 * 1024);
    write_header(&mut out);
    for stub in &sorted {
        write_stub(&mut out, stub);
    }

    // DSL-0c C2: emit the centralized wide-form dispatcher after the
    // per-opcode cold stubs. The dispatcher's match arms refer to the
    // semantic bodies (already imported via the per-stub shims above)
    // plus `crate::vm::dispatch::decode_*_operands`.
    write_wide_form_dispatcher(
        &mut out,
        &sorted.iter().copied().copied().collect::<Vec<_>>(),
    );

    write_non_aarch64_stubs(
        &mut out,
        &sorted.iter().copied().copied().collect::<Vec<_>>(),
    );

    // Determine output path. The tool runs from the workspace root
    // (cargo's working-directory convention), so the relative path
    // resolves to `crates/vm/src/dsl/handlers/cold.rs`.
    let cwd = std::env::current_dir().expect("current_dir");
    let output_path = cwd.join("crates/vm/src/dsl/handlers/cold.rs");
    if !output_path.parent().map(Path::exists).unwrap_or(false) {
        panic!(
            "expected cold.rs parent dir to exist: {}",
            output_path.display()
        );
    }
    fs::write(&output_path, &out).expect("write cold.rs");
    eprintln!(
        "[codegen] wrote {} bytes to {}",
        out.len(),
        output_path.display(),
    );
}

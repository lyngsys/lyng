//! Single-implementation invariant manifest.
//!
//! `OPCODES` enumerates every `Opcode` variant with its semantic body
//! and DSL handler symbol names. Structural tests verify exhaustive
//! coverage and symbol resolution.

use lyng_bytecode::Opcode;

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpcodeCategory {
    /// Inline fast paths for the highest-frequency opcodes.
    Hot,
    /// Backedge safepoint poll or prefix decode.
    Warm,
    /// `call_slow!`-only stub delegating to a slow-path Rust shim.
    Cold,
}

#[derive(Clone, Copy, Debug)]
pub struct OpcodeEntry {
    pub opcode: Opcode,
    pub semantic_symbol: &'static str,
    pub dsl_handler_symbol: &'static str,
    pub category: OpcodeCategory,
}

/// Every opcode variant exactly once. Adding an `Opcode` variant without
/// extending this slice fails the exhaustive-coverage test.
pub const OPCODES: &[OpcodeEntry] = &[
    // loads family (35 opcodes).
    OpcodeEntry {
        opcode: Opcode::Move,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_move_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::hot::op_move",
        category: OpcodeCategory::Hot,
    },
    OpcodeEntry {
        opcode: Opcode::LdaUndefined,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_lda_undefined_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_lda_undefined_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LdaNull,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_lda_null_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_lda_null_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LdaTrue,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_lda_true_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_lda_true_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LdaFalse,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_lda_false_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_lda_false_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LdaZero,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_lda_zero_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_lda_zero_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LdaOne,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_lda_one_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_lda_one_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadUndefined,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_load_undefined_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_undefined_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadNull,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_load_null_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_null_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadTrue,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_load_true_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_true_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadFalse,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_load_false_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_false_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadZero,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_load_zero_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_zero_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadOne,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_load_one_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_one_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadUninitializedLexical,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_load_uninitialized_lexical_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_uninitialized_lexical_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Star0,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_star_0_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_star_0_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Star1,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_star_1_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_star_1_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Star2,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_star_2_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_star_2_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Star3,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_star_3_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_star_3_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Star4,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_star_4_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_star_4_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Star5,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_star_5_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_star_5_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Star6,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_star_6_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_star_6_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Star7,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_star_7_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_star_7_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LdaSmi8,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_lda_smi8_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_lda_smi8_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LdaConst8,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_lda_const8_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_lda_const8_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Ldar,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_ldar_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_ldar_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadSmi,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_load_smi_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_smi_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadSmi8,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_load_smi8_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_smi8_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadConst,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_load_const_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_const_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadConst8,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_load_const8_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_const8_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadLocal0,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_load_local_0_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_local_0_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadLocal1,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_load_local_1_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_local_1_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadLocal2,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_load_local_2_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_local_2_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadLocal3,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_load_local_3_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_local_3_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StoreLocal0,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_store_local_0_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_store_local_0_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StoreLocal1,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_store_local_1_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_store_local_1_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StoreLocal2,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_store_local_2_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_store_local_2_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StoreLocal3,
        semantic_symbol: "lyng_vm::vm::semantics::loads::op_store_local_3_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_store_local_3_dsl",
        category: OpcodeCategory::Cold,
    },
    // arithmetic family (29 opcodes).
    OpcodeEntry {
        opcode: Opcode::Add,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_add_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::hot::op_add",
        category: OpcodeCategory::Hot,
    },
    OpcodeEntry {
        opcode: Opcode::AddSmi,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_add_smi_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_add_smi_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Sub,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_sub_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_sub_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::SubSmi,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_sub_smi_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_sub_smi_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Mul,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_mul_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_mul_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::MulSmi,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_mul_smi_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_mul_smi_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Div,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_div_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_div_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::DivSmi,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_div_smi_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_div_smi_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Mod,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_mod_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_mod_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::ModSmi,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_mod_smi_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_mod_smi_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Exp,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_exp_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_exp_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::BitOr,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_bit_or_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_bit_or_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::BitXor,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_bit_xor_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_bit_xor_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::BitAnd,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_bit_and_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_bit_and_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::BitAndSmi,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_bit_and_smi_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_bit_and_smi_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::BitNot,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_bit_not_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_bit_not_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::ShiftLeft,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_shift_left_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_shift_left_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::ShiftRight,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_shift_right_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_shift_right_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::UnsignedShiftRight,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_unsigned_shift_right_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_unsigned_shift_right_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Negate,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_negate_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_negate_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Increment,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_increment_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_increment_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Decrement,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_decrement_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_decrement_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Equal,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_equal_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_equal_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StrictEqual,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_strict_equal_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_strict_equal_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::EqualZero,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_equal_zero_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_equal_zero_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LessThan,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_less_than_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_less_than_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LessEqual,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_less_equal_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_less_equal_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::GreaterThan,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_greater_than_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_greater_than_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::GreaterEqual,
        semantic_symbol: "lyng_vm::vm::semantics::arithmetic::op_greater_equal_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_greater_equal_dsl",
        category: OpcodeCategory::Cold,
    },
    // control_flow family (10 opcodes).
    OpcodeEntry {
        opcode: Opcode::Jump,
        semantic_symbol: "lyng_vm::vm::semantics::control_flow::op_jump_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::hot::op_jump",
        category: OpcodeCategory::Hot,
    },
    OpcodeEntry {
        opcode: Opcode::Jump8,
        semantic_symbol: "lyng_vm::vm::semantics::control_flow::op_jump8_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::warm::op_jump8",
        category: OpcodeCategory::Warm,
    },
    OpcodeEntry {
        opcode: Opcode::JumpIfTrue,
        semantic_symbol: "lyng_vm::vm::semantics::control_flow::op_jump_if_true_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::warm::op_jump_if_true",
        category: OpcodeCategory::Warm,
    },
    OpcodeEntry {
        opcode: Opcode::JumpIfTrue8,
        semantic_symbol: "lyng_vm::vm::semantics::control_flow::op_jump_if_true8_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::warm::op_jump_if_true8",
        category: OpcodeCategory::Warm,
    },
    OpcodeEntry {
        opcode: Opcode::JumpIfFalse,
        semantic_symbol: "lyng_vm::vm::semantics::control_flow::op_jump_if_false_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::warm::op_jump_if_false",
        category: OpcodeCategory::Warm,
    },
    OpcodeEntry {
        opcode: Opcode::JumpIfFalse8,
        semantic_symbol: "lyng_vm::vm::semantics::control_flow::op_jump_if_false8_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::warm::op_jump_if_false8",
        category: OpcodeCategory::Warm,
    },
    OpcodeEntry {
        opcode: Opcode::LoopHeader,
        semantic_symbol: "lyng_vm::vm::semantics::control_flow::op_loop_header_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::warm::op_loop_header",
        category: OpcodeCategory::Warm,
    },
    OpcodeEntry {
        opcode: Opcode::Return,
        semantic_symbol: "lyng_vm::vm::semantics::control_flow::op_return_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::hot::op_return",
        category: OpcodeCategory::Hot,
    },
    OpcodeEntry {
        opcode: Opcode::ReturnUndefined,
        semantic_symbol: "lyng_vm::vm::semantics::control_flow::op_return_undefined_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_return_undefined_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Nop,
        semantic_symbol: "lyng_vm::vm::semantics::control_flow::op_nop_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_nop_dsl",
        category: OpcodeCategory::Cold,
    },
    // property family (21 opcodes).
    OpcodeEntry {
        opcode: Opcode::GetNamedProperty,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_get_named_property_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_get_named_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::SetNamedProperty,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_set_named_property_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_set_named_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::AssignNamedProperty,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_assign_named_property_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_assign_named_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StrictAssignNamedProperty,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_strict_assign_named_property_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_strict_assign_named_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::GetKeyedProperty,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_get_keyed_property_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_get_keyed_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::SetKeyedProperty,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_set_keyed_property_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_set_keyed_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::AssignKeyedProperty,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_assign_keyed_property_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_assign_keyed_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StrictAssignKeyedProperty,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_strict_assign_keyed_property_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_strict_assign_keyed_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::DefineNamedProperty,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_define_named_property_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_define_named_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::DefineKeyedProperty,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_define_keyed_property_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_define_keyed_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::CreateObject,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_create_object_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_create_object_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::CreateArray,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_create_array_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_create_array_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StoreDenseElement,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_store_dense_element_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_store_dense_element_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadDenseElement,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_load_dense_element_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_dense_element_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::DeleteProperty,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_delete_property_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_delete_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::In,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_in_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_in_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::ToPropertyKey,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_to_property_key_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_to_property_key_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::CopyDataProperties,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_copy_data_properties_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_copy_data_properties_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::SetFunctionName,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_set_function_name_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_set_function_name_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::CheckObjectCoercible,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_check_object_coercible_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_check_object_coercible_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::ThrowIfUninitialized,
        semantic_symbol: "lyng_vm::vm::semantics::property::op_throw_if_uninitialized_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_throw_if_uninitialized_dsl",
        category: OpcodeCategory::Cold,
    },
    // names family (17 opcodes).
    OpcodeEntry {
        opcode: Opcode::LoadGlobal,
        semantic_symbol: "lyng_vm::vm::semantics::names::op_load_global_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_global_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StoreGlobal,
        semantic_symbol: "lyng_vm::vm::semantics::names::op_store_global_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_store_global_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::AssignGlobal,
        semantic_symbol: "lyng_vm::vm::semantics::names::op_assign_global_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_assign_global_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::DeleteGlobal,
        semantic_symbol: "lyng_vm::vm::semantics::names::op_delete_global_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_delete_global_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadName,
        semantic_symbol: "lyng_vm::vm::semantics::names::op_load_name_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_name_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::ResolveName,
        semantic_symbol: "lyng_vm::vm::semantics::names::op_resolve_name_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_resolve_name_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::ResolveGlobal,
        semantic_symbol: "lyng_vm::vm::semantics::names::op_resolve_global_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_resolve_global_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::AssignName,
        semantic_symbol: "lyng_vm::vm::semantics::names::op_assign_name_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_assign_name_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::AssignVariableName,
        semantic_symbol: "lyng_vm::vm::semantics::names::op_assign_variable_name_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_assign_variable_name_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::DeleteName,
        semantic_symbol: "lyng_vm::vm::semantics::names::op_delete_name_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_delete_name_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::CaptureName,
        semantic_symbol: "lyng_vm::vm::semantics::names::op_capture_name_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_capture_name_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadCapturedName,
        semantic_symbol: "lyng_vm::vm::semantics::names::op_load_captured_name_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_captured_name_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadCapturedNameThis,
        semantic_symbol: "lyng_vm::vm::semantics::names::op_load_captured_name_this_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_captured_name_this_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::AssignCapturedName,
        semantic_symbol: "lyng_vm::vm::semantics::names::op_assign_captured_name_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_assign_captured_name_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadThis,
        semantic_symbol: "lyng_vm::vm::semantics::names::op_load_this_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_this_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadCallee,
        semantic_symbol: "lyng_vm::vm::semantics::names::op_load_callee_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_callee_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadNewTarget,
        semantic_symbol: "lyng_vm::vm::semantics::names::op_load_new_target_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_new_target_dsl",
        category: OpcodeCategory::Cold,
    },
    // scope family (10 opcodes).
    OpcodeEntry {
        opcode: Opcode::LoadEnvSlot,
        semantic_symbol: "lyng_vm::vm::semantics::scope::op_load_env_slot_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_env_slot_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StoreEnvSlot,
        semantic_symbol: "lyng_vm::vm::semantics::scope::op_store_env_slot_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_store_env_slot_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::AssignEnvSlot,
        semantic_symbol: "lyng_vm::vm::semantics::scope::op_assign_env_slot_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_assign_env_slot_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::EnterEnvScope,
        semantic_symbol: "lyng_vm::vm::semantics::scope::op_enter_env_scope_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_enter_env_scope_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LeaveEnvScope,
        semantic_symbol: "lyng_vm::vm::semantics::scope::op_leave_env_scope_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_leave_env_scope_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::PushClosureEnv,
        semantic_symbol: "lyng_vm::vm::semantics::scope::op_push_closure_env_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_push_closure_env_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::PopClosureEnv,
        semantic_symbol: "lyng_vm::vm::semantics::scope::op_pop_closure_env_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_pop_closure_env_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::PushWithEnv,
        semantic_symbol: "lyng_vm::vm::semantics::scope::op_push_with_env_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_push_with_env_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::PopWithEnv,
        semantic_symbol: "lyng_vm::vm::semantics::scope::op_pop_with_env_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_pop_with_env_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::TypeOf,
        semantic_symbol: "lyng_vm::vm::semantics::scope::op_type_of_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_type_of_dsl",
        category: OpcodeCategory::Cold,
    },
    // calls family (8 opcodes).
    OpcodeEntry {
        opcode: Opcode::Call0,
        semantic_symbol: "lyng_vm::vm::semantics::calls::op_call0_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_call0_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Call1,
        semantic_symbol: "lyng_vm::vm::semantics::calls::op_call1_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_call1_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Call2,
        semantic_symbol: "lyng_vm::vm::semantics::calls::op_call2_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_call2_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Call3,
        semantic_symbol: "lyng_vm::vm::semantics::calls::op_call3_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_call3_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Call,
        semantic_symbol: "lyng_vm::vm::semantics::calls::op_call_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_call_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::TailCall,
        semantic_symbol: "lyng_vm::vm::semantics::calls::op_tail_call_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_tail_call_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Construct,
        semantic_symbol: "lyng_vm::vm::semantics::calls::op_construct_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_construct_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::CreateClosure,
        semantic_symbol: "lyng_vm::vm::semantics::calls::op_create_closure_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_create_closure_dsl",
        category: OpcodeCategory::Cold,
    },
    // iterators family (6 opcodes).
    OpcodeEntry {
        opcode: Opcode::CreateForIn,
        semantic_symbol: "lyng_vm::vm::semantics::iterators::op_create_for_in_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_create_for_in_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::AdvanceForIn,
        semantic_symbol: "lyng_vm::vm::semantics::iterators::op_advance_for_in_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_advance_for_in_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::CloseForIn,
        semantic_symbol: "lyng_vm::vm::semantics::iterators::op_close_for_in_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_close_for_in_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::CreateIterator,
        semantic_symbol: "lyng_vm::vm::semantics::iterators::op_create_iterator_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_create_iterator_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::AdvanceIterator,
        semantic_symbol: "lyng_vm::vm::semantics::iterators::op_advance_iterator_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_advance_iterator_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::CloseIterator,
        semantic_symbol: "lyng_vm::vm::semantics::iterators::op_close_iterator_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_close_iterator_dsl",
        category: OpcodeCategory::Cold,
    },
    // generators / async family (6 opcodes).
    OpcodeEntry {
        opcode: Opcode::SuspendGeneratorStart,
        semantic_symbol: "lyng_vm::vm::semantics::generators::op_suspend_generator_start_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_suspend_generator_start_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Yield,
        semantic_symbol: "lyng_vm::vm::semantics::generators::op_yield_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_yield_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::DelegateYield,
        semantic_symbol: "lyng_vm::vm::semantics::generators::op_delegate_yield_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_delegate_yield_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Await,
        semantic_symbol: "lyng_vm::vm::semantics::generators::op_await_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_await_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadResumeKind,
        semantic_symbol: "lyng_vm::vm::semantics::generators::op_load_resume_kind_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_resume_kind_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadResumeValue,
        semantic_symbol: "lyng_vm::vm::semantics::generators::op_load_resume_value_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_resume_value_dsl",
        category: OpcodeCategory::Cold,
    },
    // exceptions family (4 opcodes).
    OpcodeEntry {
        opcode: Opcode::Throw,
        semantic_symbol: "lyng_vm::vm::semantics::exceptions::op_throw_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_throw_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::EnterHandler,
        semantic_symbol: "lyng_vm::vm::semantics::exceptions::op_enter_handler_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_enter_handler_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LeaveHandler,
        semantic_symbol: "lyng_vm::vm::semantics::exceptions::op_leave_handler_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_leave_handler_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadException,
        semantic_symbol: "lyng_vm::vm::semantics::exceptions::op_load_exception_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_load_exception_dsl",
        category: OpcodeCategory::Cold,
    },
    // prefix family (2 opcodes — Warm).
    OpcodeEntry {
        opcode: Opcode::Wide,
        semantic_symbol: "lyng_vm::vm::semantics::prefix::op_wide_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::warm::op_wide",
        category: OpcodeCategory::Warm,
    },
    OpcodeEntry {
        opcode: Opcode::ExtraWide,
        semantic_symbol: "lyng_vm::vm::semantics::prefix::op_extra_wide_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::warm::op_extra_wide",
        category: OpcodeCategory::Warm,
    },
    // misc / orphan opcodes (2). Semantic stubs return `UnsupportedOpcode`.
    OpcodeEntry {
        opcode: Opcode::InstanceOf,
        semantic_symbol: "lyng_vm::vm::semantics::misc::op_instance_of_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_instance_of_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::CallMethod,
        semantic_symbol: "lyng_vm::vm::semantics::misc::op_call_method_semantic",
        dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_call_method_dsl",
        category: OpcodeCategory::Cold,
    },
];

/// Type-erased semantic function pointer. The linker-resolution test checks
/// `is_null()`. Wrapped in a `#[repr(transparent)]` newtype so the slice is
/// `Sync` (raw `*const ()` is not, but immutable code pointers are).
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SemanticFnPtr(pub *const ());

// SAFETY: holds the address of a real Rust function; read-only, never
// dereferenced. No shared mutable state.
unsafe impl Sync for SemanticFnPtr {}

impl SemanticFnPtr {
    /// Returns `true` if the wrapped pointer is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Parallel slice to `OPCODES` with type-erased function pointers for each
/// `op_xxx_semantic`. Adding an `OpcodeEntry` without adding the fn-ptr fails
/// the length-equality test.
pub static SEMANTIC_FN_PTRS: &[SemanticFnPtr] = &[
    // loads family (35 opcodes).
    SemanticFnPtr(
        crate::vm::semantics::loads::op_move_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpMoveArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_lda_undefined_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLdaConstantArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_lda_null_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLdaConstantArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_lda_true_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLdaConstantArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_lda_false_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLdaConstantArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_lda_zero_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLdaConstantArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_lda_one_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLdaConstantArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_load_undefined_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLoadConstantArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_load_null_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLoadConstantArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_load_true_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLoadConstantArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_load_false_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLoadConstantArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_load_zero_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLoadConstantArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_load_one_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLoadConstantArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_load_uninitialized_lexical_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLoadConstantArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_star_0_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpStarArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_star_1_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpStarArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_star_2_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpStarArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_star_3_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpStarArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_star_4_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpStarArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_star_5_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpStarArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_star_6_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpStarArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_star_7_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpStarArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_lda_smi8_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLdaSmi8Args,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_lda_const8_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLdaConst8Args,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_ldar_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLdarArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_load_smi_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLoadSmiArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_load_smi8_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLoadSmi8Args,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_load_const_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLoadConstArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_load_const8_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLoadConst8Args,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_load_local_0_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLoadLocalArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_load_local_1_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLoadLocalArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_load_local_2_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLoadLocalArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_load_local_3_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpLoadLocalArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_store_local_0_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpStoreLocalArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_store_local_1_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpStoreLocalArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_store_local_2_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpStoreLocalArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::loads::op_store_local_3_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::loads::OpStoreLocalArgs,
            ) -> SemanticOutcome as *const (),
    ),
    // arithmetic family (29 opcodes).
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_add_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_add_smi_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinarySmiArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_sub_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_sub_smi_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinarySmiArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_mul_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_mul_smi_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinarySmiArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_div_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_div_smi_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinarySmiArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_mod_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_mod_smi_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinarySmiArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_exp_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_bit_or_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_bit_xor_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_bit_and_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_bit_and_smi_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinarySmiArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_bit_not_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpUnaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_shift_left_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_shift_right_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_unsigned_shift_right_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_negate_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpUnaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_increment_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpUpdateArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_decrement_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpUpdateArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_equal_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_strict_equal_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_equal_zero_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpEqualZeroArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_less_than_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_less_equal_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_greater_than_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::arithmetic::op_greater_equal_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::arithmetic::OpBinaryArgs,
            ) -> SemanticOutcome as *const (),
    ),
    // control_flow family (10 opcodes).
    SemanticFnPtr(
        crate::vm::semantics::control_flow::op_jump_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::control_flow::OpJumpArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::control_flow::op_jump8_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::control_flow::OpJumpArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::control_flow::op_jump_if_true_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::control_flow::OpJumpIfArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::control_flow::op_jump_if_true8_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::control_flow::OpJumpIfArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::control_flow::op_jump_if_false_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::control_flow::OpJumpIfArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::control_flow::op_jump_if_false8_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::control_flow::OpJumpIfArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::control_flow::op_loop_header_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::control_flow::OpLoopHeaderArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::control_flow::op_return_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::control_flow::OpReturnArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::control_flow::op_return_undefined_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::control_flow::OpReturnUndefinedArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::control_flow::op_nop_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::control_flow::OpNopArgs,
            ) -> SemanticOutcome as *const (),
    ),
    // property family (21 opcodes).
    SemanticFnPtr(
        crate::vm::semantics::property::op_get_named_property_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAccessArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_set_named_property_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAccessArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_assign_named_property_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAccessArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_strict_assign_named_property_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAccessArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_get_keyed_property_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAccessArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_set_keyed_property_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAccessArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_assign_keyed_property_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAccessArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_strict_assign_keyed_property_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAccessArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_define_named_property_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAbcArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_define_keyed_property_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAbcArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_create_object_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAbxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_create_array_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAbxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_store_dense_element_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAbcArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_load_dense_element_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAbcArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_delete_property_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAbcArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_in_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAbcArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_to_property_key_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAbArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_copy_data_properties_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAbcArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_set_function_name_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAbArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_check_object_coercible_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAbxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::property::op_throw_if_uninitialized_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::property::OpPropertyAbxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    // names family (17 opcodes).
    SemanticFnPtr(
        crate::vm::semantics::names::op_load_global_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::names::OpAtomArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::names::op_store_global_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::names::OpAtomArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::names::op_assign_global_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::names::OpAtomArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::names::op_delete_global_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::names::OpAtomArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::names::op_load_name_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::names::OpAtomArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::names::op_resolve_name_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::names::OpAtomArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::names::op_resolve_global_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::names::OpAtomArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::names::op_assign_name_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::names::OpAtomArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::names::op_assign_variable_name_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::names::OpAtomArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::names::op_delete_name_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::names::OpAtomArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::names::op_capture_name_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::names::OpAtomArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::names::op_load_captured_name_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::names::OpCapturedNameArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::names::op_load_captured_name_this_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::names::OpCapturedNameArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::names::op_assign_captured_name_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::names::OpCapturedNameArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::names::op_load_this_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::names::OpAtomArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::names::op_load_callee_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::names::OpAtomArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::names::op_load_new_target_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::names::OpAtomArgs,
            ) -> SemanticOutcome as *const (),
    ),
    // scope family (10 opcodes).
    SemanticFnPtr(
        crate::vm::semantics::scope::op_load_env_slot_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::scope::OpScopeAbxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::scope::op_store_env_slot_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::scope::OpScopeAbxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::scope::op_assign_env_slot_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::scope::OpScopeAbxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::scope::op_enter_env_scope_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::scope::OpScopeAbxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::scope::op_leave_env_scope_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::scope::OpScopeAbxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::scope::op_push_closure_env_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::scope::OpScopeAxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::scope::op_pop_closure_env_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::scope::OpScopeAxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::scope::op_push_with_env_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::scope::OpScopeAxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::scope::op_pop_with_env_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::scope::OpScopeAxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::scope::op_type_of_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::scope::OpScopeAxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    // calls family (8 opcodes).
    SemanticFnPtr(
        crate::vm::semantics::calls::op_call0_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::calls::OpCallSmallArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::calls::op_call1_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::calls::OpCallSmallArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::calls::op_call2_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::calls::OpCallSmallArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::calls::op_call3_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::calls::OpCallSmallArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::calls::op_call_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::calls::OpCallRangeArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::calls::op_tail_call_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::calls::OpTailCallArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::calls::op_construct_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::calls::OpCallRangeArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::calls::op_create_closure_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::calls::OpCreateClosureArgs,
            ) -> SemanticOutcome as *const (),
    ),
    // iterators family (6 opcodes).
    SemanticFnPtr(
        crate::vm::semantics::iterators::op_create_for_in_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::iterators::OpIteratorAbcArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::iterators::op_advance_for_in_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::iterators::OpIteratorAbcArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::iterators::op_close_for_in_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::iterators::OpIteratorAbxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::iterators::op_create_iterator_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::iterators::OpIteratorAbcArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::iterators::op_advance_iterator_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::iterators::OpIteratorAbcArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::iterators::op_close_iterator_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::iterators::OpIteratorAbxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    // generators / async family (6 opcodes).
    SemanticFnPtr(
        crate::vm::semantics::generators::op_suspend_generator_start_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::generators::OpSuspendGeneratorStartArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::generators::op_yield_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::generators::OpGeneratorsAxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::generators::op_delegate_yield_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::generators::OpDelegateYieldArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::generators::op_await_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::generators::OpGeneratorsAxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::generators::op_load_resume_kind_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::generators::OpGeneratorsAxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::generators::op_load_resume_value_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::generators::OpGeneratorsAxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    // exceptions family (4 opcodes).
    SemanticFnPtr(
        crate::vm::semantics::exceptions::op_throw_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::exceptions::OpExceptionsAxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::exceptions::op_enter_handler_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::exceptions::OpHandlerMarkerArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::exceptions::op_leave_handler_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::exceptions::OpHandlerMarkerArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::exceptions::op_load_exception_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::exceptions::OpExceptionsAxArgs,
            ) -> SemanticOutcome as *const (),
    ),
    // prefix family (2 opcodes).
    SemanticFnPtr(
        crate::vm::semantics::prefix::op_wide_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::prefix::OpPrefixArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::prefix::op_extra_wide_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::prefix::OpPrefixArgs,
            ) -> SemanticOutcome as *const (),
    ),
    // misc / orphan opcodes (2).
    SemanticFnPtr(
        crate::vm::semantics::misc::op_instance_of_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::misc::OpMiscStubArgs,
            ) -> SemanticOutcome as *const (),
    ),
    SemanticFnPtr(
        crate::vm::semantics::misc::op_call_method_semantic
            as fn(
                &mut LlIntDispatchState<'_, '_>,
                crate::vm::semantics::misc::OpMiscStubArgs,
            ) -> SemanticOutcome as *const (),
    ),
];

/// Filter `OPCODES` by category.
pub fn by_category(category: OpcodeCategory) -> impl Iterator<Item = &'static OpcodeEntry> {
    OPCODES
        .iter()
        .filter(move |entry| entry.category == category)
}

#[cfg(test)]
mod manifest_tests {
    use super::*;
    use lyng_bytecode::{OPCODE_COUNT, Opcode};
    use std::collections::HashSet;

    /// Every `Opcode` variant appears in `OPCODES` exactly once.
    #[test]
    fn opcodes_manifest_is_exhaustive() {
        let count = OPCODE_COUNT as usize;
        assert_eq!(
            OPCODES.len(),
            count,
            "OPCODES has {} entries, expected {} (OPCODE_COUNT)",
            OPCODES.len(),
            count,
        );

        let mut seen: HashSet<u8> = HashSet::new();
        for entry in OPCODES {
            let byte = entry.opcode as u8;
            assert!(
                byte < OPCODE_COUNT,
                "OPCODES entry for {:?} has byte {} outside [0, {})",
                entry.opcode,
                byte,
                OPCODE_COUNT,
            );
            assert!(
                seen.insert(byte),
                "OPCODES has duplicate entry for opcode byte {} ({:?})",
                byte,
                entry.opcode,
            );
        }

        for byte in 0..OPCODE_COUNT {
            assert!(
                seen.contains(&byte),
                "OPCODES missing entry for opcode byte {}: {:?}",
                byte,
                Opcode::from_byte(byte),
            );
        }
    }

    /// Every `OpcodeEntry.semantic_symbol` names a real Rust function.
    /// `SEMANTIC_FN_PTRS` holds the linker-resolved pointer for each entry
    /// in the same order.
    #[test]
    fn semantic_fn_ptrs_resolve() {
        assert_eq!(
            SEMANTIC_FN_PTRS.len(),
            OPCODES.len(),
            "SEMANTIC_FN_PTRS has {} entries, OPCODES has {}",
            SEMANTIC_FN_PTRS.len(),
            OPCODES.len(),
        );
        for (idx, ptr) in SEMANTIC_FN_PTRS.iter().enumerate() {
            assert!(
                !ptr.is_null(),
                "SEMANTIC_FN_PTRS[{idx}] is null (opcode = {:?})",
                OPCODES[idx].opcode,
            );
        }
    }
}

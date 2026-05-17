//! Single-implementation invariant manifest per design §10.
//!
//! `OPCODES` enumerates every `Opcode` variant exactly once with the
//! resolvable symbol names for its semantic body and (post-DSL-0b) its
//! DSL handler. Seven structural tests use this manifest to verify the
//! invariant — see the `manifest_tests` module.

use lyng_js_bytecode::Opcode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpcodeCategory {
    /// Full DSL body with inline fast paths (5 opcodes from DSL-0b plus
    /// 25 more in DSL-1).
    Hot,
    /// Full DSL body that includes a safepoint poll on its backedge
    /// (loop header + backward-jump variants + prefix opcodes).
    Warm,
    /// Three-line DSL stub delegating to a slow-path Rust shim.
    Cold,
}

#[derive(Clone, Copy, Debug)]
pub struct OpcodeEntry {
    pub opcode: Opcode,
    pub semantic_symbol: &'static str,
    pub dsl_handler_symbol: &'static str,
    pub category: OpcodeCategory,
}

/// The single source of truth for the single-implementation invariant.
///
/// Tests A6 / A19 / C9 / C10 / C11 walk this slice to verify exhaustive
/// coverage and symbol resolution. Adding an `Opcode` variant without
/// extending this slice fails Test 1 (exhaustive coverage).
pub const OPCODES: &[OpcodeEntry] = &[
    // Task A8 — loads family (35 opcodes).
    OpcodeEntry {
        opcode: Opcode::Move,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_move_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_move_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LdaUndefined,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_lda_undefined_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_lda_undefined_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LdaNull,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_lda_null_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_lda_null_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LdaTrue,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_lda_true_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_lda_true_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LdaFalse,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_lda_false_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_lda_false_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LdaZero,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_lda_zero_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_lda_zero_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LdaOne,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_lda_one_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_lda_one_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadUndefined,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_load_undefined_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_undefined_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadNull,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_load_null_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_null_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadTrue,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_load_true_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_true_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadFalse,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_load_false_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_false_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadZero,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_load_zero_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_zero_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadOne,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_load_one_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_one_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadUninitializedLexical,
        semantic_symbol:
            "lyng_js_vm::vm::semantics::loads::op_load_uninitialized_lexical_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_uninitialized_lexical_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Star0,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_star_0_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_star_0_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Star1,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_star_1_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_star_1_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Star2,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_star_2_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_star_2_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Star3,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_star_3_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_star_3_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Star4,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_star_4_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_star_4_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Star5,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_star_5_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_star_5_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Star6,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_star_6_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_star_6_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Star7,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_star_7_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_star_7_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LdaSmi8,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_lda_smi8_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_lda_smi8_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LdaConst8,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_lda_const8_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_lda_const8_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Ldar,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_ldar_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_ldar_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadSmi,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_load_smi_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_smi_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadSmi8,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_load_smi8_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_smi8_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadConst,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_load_const_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_const_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadConst8,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_load_const8_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_const8_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadLocal0,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_load_local_0_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_local_0_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadLocal1,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_load_local_1_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_local_1_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadLocal2,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_load_local_2_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_local_2_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadLocal3,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_load_local_3_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_local_3_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StoreLocal0,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_store_local_0_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_store_local_0_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StoreLocal1,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_store_local_1_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_store_local_1_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StoreLocal2,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_store_local_2_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_store_local_2_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StoreLocal3,
        semantic_symbol: "lyng_js_vm::vm::semantics::loads::op_store_local_3_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_store_local_3_dsl",
        category: OpcodeCategory::Cold,
    },
    // Task A9 — arithmetic family (29 opcodes).
    OpcodeEntry {
        opcode: Opcode::Add,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_add_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_add_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::AddSmi,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_add_smi_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_add_smi_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Sub,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_sub_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_sub_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::SubSmi,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_sub_smi_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_sub_smi_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Mul,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_mul_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_mul_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::MulSmi,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_mul_smi_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_mul_smi_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Div,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_div_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_div_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::DivSmi,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_div_smi_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_div_smi_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Mod,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_mod_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_mod_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::ModSmi,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_mod_smi_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_mod_smi_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Exp,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_exp_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_exp_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::BitOr,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_bit_or_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_bit_or_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::BitXor,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_bit_xor_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_bit_xor_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::BitAnd,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_bit_and_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_bit_and_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::BitAndSmi,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_bit_and_smi_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_bit_and_smi_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::BitNot,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_bit_not_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_bit_not_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::ShiftLeft,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_shift_left_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_shift_left_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::ShiftRight,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_shift_right_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_shift_right_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::UnsignedShiftRight,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_unsigned_shift_right_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_unsigned_shift_right_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Negate,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_negate_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_negate_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Increment,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_increment_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_increment_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Decrement,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_decrement_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_decrement_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Equal,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_equal_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_equal_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StrictEqual,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_strict_equal_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_strict_equal_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::EqualZero,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_equal_zero_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_equal_zero_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LessThan,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_less_than_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_less_than_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LessEqual,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_less_equal_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_less_equal_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::GreaterThan,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_greater_than_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_greater_than_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::GreaterEqual,
        semantic_symbol: "lyng_js_vm::vm::semantics::arithmetic::op_greater_equal_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_greater_equal_dsl",
        category: OpcodeCategory::Cold,
    },
    // Task A10 — control_flow family (10 opcodes). `Jump` and `Return` are
    // upgraded to `Hot` in DSL-0b tasks B41 / B42; `LoopHeader` to `Warm` in
    // B43. DSL-0a registers all ten as `Cold`.
    OpcodeEntry {
        opcode: Opcode::Jump,
        semantic_symbol: "lyng_js_vm::vm::semantics::control_flow::op_jump_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_jump_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Jump8,
        semantic_symbol: "lyng_js_vm::vm::semantics::control_flow::op_jump8_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_jump8_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::JumpIfTrue,
        semantic_symbol: "lyng_js_vm::vm::semantics::control_flow::op_jump_if_true_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_jump_if_true_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::JumpIfTrue8,
        semantic_symbol: "lyng_js_vm::vm::semantics::control_flow::op_jump_if_true8_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_jump_if_true8_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::JumpIfFalse,
        semantic_symbol: "lyng_js_vm::vm::semantics::control_flow::op_jump_if_false_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_jump_if_false_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::JumpIfFalse8,
        semantic_symbol: "lyng_js_vm::vm::semantics::control_flow::op_jump_if_false8_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_jump_if_false8_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoopHeader,
        semantic_symbol: "lyng_js_vm::vm::semantics::control_flow::op_loop_header_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_loop_header_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Return,
        semantic_symbol: "lyng_js_vm::vm::semantics::control_flow::op_return_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_return_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::ReturnUndefined,
        semantic_symbol: "lyng_js_vm::vm::semantics::control_flow::op_return_undefined_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_return_undefined_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::Nop,
        semantic_symbol: "lyng_js_vm::vm::semantics::control_flow::op_nop_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_nop_dsl",
        category: OpcodeCategory::Cold,
    },
    // Task A11 — property family (21 opcodes).
    OpcodeEntry {
        opcode: Opcode::GetNamedProperty,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_get_named_property_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_get_named_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::SetNamedProperty,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_set_named_property_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_set_named_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::AssignNamedProperty,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_assign_named_property_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_assign_named_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StrictAssignNamedProperty,
        semantic_symbol:
            "lyng_js_vm::vm::semantics::property::op_strict_assign_named_property_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_strict_assign_named_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::GetKeyedProperty,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_get_keyed_property_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_get_keyed_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::SetKeyedProperty,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_set_keyed_property_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_set_keyed_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::AssignKeyedProperty,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_assign_keyed_property_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_assign_keyed_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StrictAssignKeyedProperty,
        semantic_symbol:
            "lyng_js_vm::vm::semantics::property::op_strict_assign_keyed_property_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_strict_assign_keyed_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::DefineNamedProperty,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_define_named_property_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_define_named_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::DefineKeyedProperty,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_define_keyed_property_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_define_keyed_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::CreateObject,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_create_object_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_create_object_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::CreateArray,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_create_array_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_create_array_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StoreDenseElement,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_store_dense_element_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_store_dense_element_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadDenseElement,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_load_dense_element_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_dense_element_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::DeleteProperty,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_delete_property_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_delete_property_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::In,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_in_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_in_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::ToPropertyKey,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_to_property_key_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_to_property_key_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::CopyDataProperties,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_copy_data_properties_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_copy_data_properties_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::SetFunctionName,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_set_function_name_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_set_function_name_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::CheckObjectCoercible,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_check_object_coercible_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_check_object_coercible_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::ThrowIfUninitialized,
        semantic_symbol: "lyng_js_vm::vm::semantics::property::op_throw_if_uninitialized_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_throw_if_uninitialized_dsl",
        category: OpcodeCategory::Cold,
    },
    // Task A12 — names family (17 opcodes).
    OpcodeEntry {
        opcode: Opcode::LoadGlobal,
        semantic_symbol: "lyng_js_vm::vm::semantics::names::op_load_global_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_global_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StoreGlobal,
        semantic_symbol: "lyng_js_vm::vm::semantics::names::op_store_global_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_store_global_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::AssignGlobal,
        semantic_symbol: "lyng_js_vm::vm::semantics::names::op_assign_global_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_assign_global_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::DeleteGlobal,
        semantic_symbol: "lyng_js_vm::vm::semantics::names::op_delete_global_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_delete_global_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadName,
        semantic_symbol: "lyng_js_vm::vm::semantics::names::op_load_name_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_name_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::ResolveName,
        semantic_symbol: "lyng_js_vm::vm::semantics::names::op_resolve_name_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_resolve_name_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::ResolveGlobal,
        semantic_symbol: "lyng_js_vm::vm::semantics::names::op_resolve_global_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_resolve_global_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::AssignName,
        semantic_symbol: "lyng_js_vm::vm::semantics::names::op_assign_name_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_assign_name_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::AssignVariableName,
        semantic_symbol: "lyng_js_vm::vm::semantics::names::op_assign_variable_name_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_assign_variable_name_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::DeleteName,
        semantic_symbol: "lyng_js_vm::vm::semantics::names::op_delete_name_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_delete_name_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::CaptureName,
        semantic_symbol: "lyng_js_vm::vm::semantics::names::op_capture_name_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_capture_name_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadCapturedName,
        semantic_symbol: "lyng_js_vm::vm::semantics::names::op_load_captured_name_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_captured_name_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadCapturedNameThis,
        semantic_symbol: "lyng_js_vm::vm::semantics::names::op_load_captured_name_this_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_captured_name_this_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::AssignCapturedName,
        semantic_symbol: "lyng_js_vm::vm::semantics::names::op_assign_captured_name_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_assign_captured_name_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadThis,
        semantic_symbol: "lyng_js_vm::vm::semantics::names::op_load_this_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_this_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadCallee,
        semantic_symbol: "lyng_js_vm::vm::semantics::names::op_load_callee_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_callee_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LoadNewTarget,
        semantic_symbol: "lyng_js_vm::vm::semantics::names::op_load_new_target_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_new_target_dsl",
        category: OpcodeCategory::Cold,
    },
    // Task A13 — scope family (10 opcodes).
    OpcodeEntry {
        opcode: Opcode::LoadEnvSlot,
        semantic_symbol: "lyng_js_vm::vm::semantics::scope::op_load_env_slot_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_load_env_slot_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::StoreEnvSlot,
        semantic_symbol: "lyng_js_vm::vm::semantics::scope::op_store_env_slot_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_store_env_slot_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::AssignEnvSlot,
        semantic_symbol: "lyng_js_vm::vm::semantics::scope::op_assign_env_slot_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_assign_env_slot_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::EnterEnvScope,
        semantic_symbol: "lyng_js_vm::vm::semantics::scope::op_enter_env_scope_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_enter_env_scope_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::LeaveEnvScope,
        semantic_symbol: "lyng_js_vm::vm::semantics::scope::op_leave_env_scope_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_leave_env_scope_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::PushClosureEnv,
        semantic_symbol: "lyng_js_vm::vm::semantics::scope::op_push_closure_env_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_push_closure_env_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::PopClosureEnv,
        semantic_symbol: "lyng_js_vm::vm::semantics::scope::op_pop_closure_env_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_pop_closure_env_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::PushWithEnv,
        semantic_symbol: "lyng_js_vm::vm::semantics::scope::op_push_with_env_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_push_with_env_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::PopWithEnv,
        semantic_symbol: "lyng_js_vm::vm::semantics::scope::op_pop_with_env_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_pop_with_env_dsl",
        category: OpcodeCategory::Cold,
    },
    OpcodeEntry {
        opcode: Opcode::TypeOf,
        semantic_symbol: "lyng_js_vm::vm::semantics::scope::op_type_of_semantic",
        dsl_handler_symbol: "lyng_js_vm::dsl::handlers::cold::op_type_of_dsl",
        category: OpcodeCategory::Cold,
    },
];

/// Subset filter for the DSL_DISPATCH_TABLE assembly in DSL-0b.
pub fn by_category(category: OpcodeCategory) -> impl Iterator<Item = &'static OpcodeEntry> {
    OPCODES.iter().filter(move |entry| entry.category == category)
}

#[cfg(test)]
mod manifest_tests {
    use super::*;
    use lyng_js_bytecode::{Opcode, OPCODE_COUNT};
    use std::collections::HashSet;

    /// Test 1 from design §10 DSL-0a: every `Opcode` variant appears in
    /// `OPCODES` exactly once.
    #[test]
    #[ignore = "Enabled by Task A18 once all family extractions are complete"]
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
}

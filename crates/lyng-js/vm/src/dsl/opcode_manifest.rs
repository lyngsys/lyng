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

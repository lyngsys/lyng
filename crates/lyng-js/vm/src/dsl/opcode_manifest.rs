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
    // Populated by family-extraction tasks A8–A18.
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

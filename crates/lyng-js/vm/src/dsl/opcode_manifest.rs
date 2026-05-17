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

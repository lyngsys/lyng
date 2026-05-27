//! asm-DSL substrate per docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md.
//!
//! This module hosts the DSL runtime support — opcode manifest, slow-path
//! bridge types, register-pin convention, `LlIntState` ABI, entry/exit
//! shims, and the per-arch DSL operation backend. The proc-macro that
//! consumes these lives in the separate `lyng-vm-dsl` crate.
//!
//! During DSL-0a the only module populated is `opcode_manifest` plus the
//! transitional `LlIntDispatchState` wrapper in `slow_path`. DSL-0b adds
//! every other module.

#![allow(
    clippy::too_long_first_doc_paragraph,
    reason = "DSL docs intentionally preserve design-note paragraphs that describe ABI and asm invariants in one place"
)]

pub mod backend;
pub mod entry;
pub mod handlers;
pub mod llint_state;
pub mod opcode_manifest;
pub mod poll;
pub mod reg_convention;
pub mod slow_path;

// DSL-0b validation harness shared by tasks B31–B38. The module is
// `#[doc(hidden)] pub` rather than `#[cfg(test)]` because the
// `tests/dsl_validation_*.rs` integration tests live in separate
// crates and only see `pub` items. Production paths never touch this
// module; the symbols compile in but optimize out of release builds.
#[doc(hidden)]
pub mod test_helpers;

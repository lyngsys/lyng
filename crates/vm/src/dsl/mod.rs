//! asm-DSL substrate: opcode manifest, slow-path bridge, register-pin
//! convention, `LlIntState` ABI, entry/exit shims, and the per-arch
//! backend. The proc-macro that consumes these lives in `lyng-vm-dsl`.

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

// Validation harness used by integration tests in `tests/dsl_validation_*.rs`.
// `#[doc(hidden)] pub` so those test crates can reach it; symbols optimize out
// of release builds.
#[doc(hidden)]
pub mod test_helpers;

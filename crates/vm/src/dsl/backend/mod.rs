//! Per-arch DSL backend dispatch. Today: `AArch64` only.
//!
//! Each `backend::*` module exports `macro_rules!` macros that emit
//! `&'static str` asm fragments (via `concat!`). The DSL proc-macro
//! lowerer collects those fragments into a single
//! `core::arch::naked_asm!(...)` block per handler. See
//! `crates/vm/src/dsl/ops.md` for the full vocabulary.

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

// Re-export the proc-macro-facing entry point.
#[cfg(target_arch = "aarch64")]
pub use aarch64::__llint_handler_body;

/// Escape hatch for arch-specific bridges that need raw asm at a
/// non-naked Rust call site (e.g. slow-path shims that hop into asm
/// before returning normally). The proc-macro never uses this — only
/// hand-written bridge code does.
#[cfg(target_arch = "aarch64")]
#[macro_export]
macro_rules! raw_asm {
    ($body:literal) => {
        ::core::arch::asm!($body);
    };
}

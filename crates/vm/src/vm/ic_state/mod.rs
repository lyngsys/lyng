//! Per-IC-kind Rust-only state machines. Phase D side-table for the
//! IC kind's state-machine bits that don't need to be asm-readable.
//! Asm-readable bits (mode, generation, handler_bits, aux_bits,
//! execution_count) live on `MetadataTable.*Metadata` structs.

pub mod property;

pub use property::PropertyIcState;
// Re-export InlineCacheState so tests and future callers can compare against it
// without reaching into the private `feedback` module.
#[allow(
    unused_imports,
    reason = "Phase D.1.1 test surface; consumed in tests::inline_caches D1-D4"
)]
pub(crate) use crate::vm::feedback::InlineCacheState;

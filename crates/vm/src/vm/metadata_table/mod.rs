//! JSC-style per-code-object `MetadataTable`. Spec 2 Phase C.
//!
//! Layout: header + per-kind offset table + slot→in-kind-index table + per-kind runs.
//! Phase C.1 lands the type + allocator; reads and writes wire up in C.2/C.4.

pub mod kind;

#[allow(unused_imports)]
pub use kind::{MetadataKind, METADATA_KIND_COUNT};

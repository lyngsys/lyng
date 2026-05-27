//! `ComparisonStatus` projection (Spec 2 Phase E).

use super::ScalarObserved;

/// Status projection for one `Comparison` IC slot.
///
/// Comparison feedback is MetadataTable-owned (parallel to `ArithStatus`).
/// At time of writing no asm caller drives the observed bits, so the
/// counts/flags are always zero in practice; the struct is still provided
/// so the public API is symmetric and so the field placeholders are ready
/// when asm/IC writes are added.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ComparisonStatus {
    pub observed: ScalarObserved,
    pub execution_count: u32,
}

impl ComparisonStatus {
    /// Build from the raw `ComparisonMetadata` fields.
    #[must_use]
    pub const fn from_metadata(observed_bits: u32, execution_count: u32) -> Self {
        Self {
            observed: ScalarObserved::from_bits(observed_bits),
            execution_count,
        }
    }
}

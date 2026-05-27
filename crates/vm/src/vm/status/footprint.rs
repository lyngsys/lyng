//! `MetadataTableFootprint` projection (Spec 2 Phase E).
//!
//! The bytes counted include both the per-code `MetadataTable` buffer and the
//! Vec-indexed side-tables (`PropertyIcState`, `CallIcState`, `ConstructIcState`,
//! `KeyedPropertyIcState`, `PolymorphicChain`) so consumers see the full
//! IC memory cost.

use crate::vm::metadata_table::METADATA_KIND_COUNT;

/// Memory + per-kind site-count footprint for one installed code object's
/// IC state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MetadataTableFootprint {
    /// Whether the code's IC side-tables have been allocated (i.e. the
    /// warmup threshold has been crossed). When `false`, the per-kind
    /// counts may still be non-zero (the metadata buffer is sized at
    /// install time), but the cumulative execution count is zero.
    pub allocated: bool,
    /// Number of bytes used by the `MetadataTable` buffer plus the per-code
    /// side-table allocations.
    pub allocated_bytes: usize,
    /// Per-kind populated site counts. Indexed by [`MetadataKind`] as
    /// defined in `metadata_table::kind`:
    ///   0 = Property
    ///   1 = Call
    ///   2 = Arith
    ///   3 = Comparison
    ///   4 = KeyedProperty
    pub live_site_count_by_kind: [usize; METADATA_KIND_COUNT],
    /// Sum of execution counts across all IC slots for this code object.
    pub total_execution_count: u64,
    /// Warmup counter (number of recorded executions before the IC
    /// allocation threshold). Useful for tier-up heuristics that watch the
    /// warmup phase.
    pub warmup_counter: u16,
    /// Total number of compiled-in feedback slots (including unpopulated ones).
    pub slot_count: usize,
    /// Number of compiled-in feedback sites with a non-empty descriptor
    /// (compile-time live sites, not runtime-populated ones).
    pub live_site_count: usize,
}

impl MetadataTableFootprint {
    #[inline]
    #[must_use]
    pub const fn allocated(self) -> bool {
        self.allocated
    }

    #[inline]
    #[must_use]
    pub const fn allocated_bytes(self) -> usize {
        self.allocated_bytes
    }

    #[inline]
    #[must_use]
    pub const fn live_site_count_by_kind(self) -> [usize; METADATA_KIND_COUNT] {
        self.live_site_count_by_kind
    }

    #[inline]
    #[must_use]
    pub const fn total_execution_count(self) -> u64 {
        self.total_execution_count
    }

    #[inline]
    #[must_use]
    pub const fn warmup_counter(self) -> u16 {
        self.warmup_counter
    }

    #[inline]
    #[must_use]
    pub const fn slot_count(self) -> usize {
        self.slot_count
    }

    #[inline]
    #[must_use]
    pub const fn live_site_count(self) -> usize {
        self.live_site_count
    }
}

//! Out-of-line storage for polymorphic IC chain entries beyond `POLY_LIMIT`.
//! Spec 2 Phase B.
//!
//! Each `(CodeRef, FeedbackSlotId)` that grows into a 3+ entry polymorphic
//! state gets a `PolymorphicChain` entry in `Vm::polymorphic_chains`.
//! The chain holds entries [`POLY_LIMIT..POLYMORPHIC_PROPERTY_CACHE_LIMIT`].
//! Entries [`0..POLY_LIMIT`] stay inline in `NamedPropertyFeedback.entries`
//! to keep the asm fast path's sidecar (`polymorphic_own_data_handlers`)
//! addressable in the existing layout.
//!
//! On 9th distinct shape the IC transitions to Megamorphic and the chain
//! entry is dropped (caller's responsibility).
//!
//! Sorted-by-shape invariant: the chain is kept in ascending `receiver_shape`
//! order so that the snapshot iterator (inline ++ chain) yields the entries
//! sorted. Inline shapes are guaranteed strictly less than chain shapes by
//! the install logic (an entry in the chain came from a logical position
//! `>= POLY_LIMIT`).

use std::cmp::Ordering;

use lyng_objects::NamedPropertyCacheEntry;
use lyng_types::ShapeId;

use super::{POLYMORPHIC_PROPERTY_CACHE_LIMIT, POLY_LIMIT};

/// Maximum number of out-of-line entries per chain.
/// `POLYMORPHIC_PROPERTY_CACHE_LIMIT - POLY_LIMIT` (= 6).
pub const POLYMORPHIC_CHAIN_CAP: usize = POLYMORPHIC_PROPERTY_CACHE_LIMIT - POLY_LIMIT;

pub struct PolymorphicChain {
    entries: Vec<NamedPropertyCacheEntry>,
}

impl PolymorphicChain {
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::with_capacity(POLYMORPHIC_CHAIN_CAP),
        }
    }

    pub(crate) const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Linear search by receiver shape. Chain is small (≤6) so linear beats
    /// binary search hash overhead. Returns `None` if no entry matches.
    pub(crate) fn find_by_shape(
        &self,
        receiver_shape: ShapeId,
    ) -> Option<&NamedPropertyCacheEntry> {
        self.entries
            .iter()
            .find(|entry| entry.receiver_shape() == receiver_shape)
    }

    /// Searches the chain for `receiver_shape`. `Ok(index)` if found,
    /// `Err(insertion_point)` otherwise — same semantics as
    /// `slice::binary_search`. The chain is kept sorted by ascending
    /// `receiver_shape`, so linear search returns the correct
    /// insertion point.
    pub(crate) fn search_sorted(&self, receiver_shape: ShapeId) -> Result<usize, usize> {
        for (index, entry) in self.entries.iter().enumerate() {
            match entry.receiver_shape().cmp(&receiver_shape) {
                Ordering::Equal => return Ok(index),
                Ordering::Greater => return Err(index),
                Ordering::Less => {}
            }
        }
        Err(self.entries.len())
    }

    /// Replaces the entry at `index`. Caller must have verified the index is
    /// in range (typically via a prior `search_sorted` returning `Ok(index)`).
    pub(crate) fn replace_at(&mut self, index: usize, entry: NamedPropertyCacheEntry) {
        self.entries[index] = entry;
    }

    /// Inserts `entry` at `index`, shifting later entries right. Caller must
    /// have verified `self.len() < POLYMORPHIC_CHAIN_CAP` and that
    /// `index <= self.len()`.
    pub(crate) fn insert_at(&mut self, index: usize, entry: NamedPropertyCacheEntry) {
        debug_assert!(self.entries.len() < POLYMORPHIC_CHAIN_CAP);
        debug_assert!(index <= self.entries.len());
        self.entries.insert(index, entry);
    }

    /// Iterator over entries — used by the slow path for fallback walks
    /// and by GC tracing to visit holder references.
    #[allow(
        dead_code,
        reason = "TODO(Phase E): snapshot restoration will walk chain entries"
    )]
    pub(crate) fn entries(&self) -> impl Iterator<Item = &NamedPropertyCacheEntry> {
        self.entries.iter()
    }
}

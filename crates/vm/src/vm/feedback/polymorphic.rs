#![allow(dead_code, reason = "Spec 2 Phase B.1.1: type defined here, callers land in B.1.2+")]
//! Out-of-line storage for polymorphic IC chain entries beyond `POLY_LIMIT`.
//! Spec 2 Phase B.
//!
//! Each `(CodeRef, FeedbackSlotId)` that grows into a 3+ entry polymorphic
//! state gets a `PolymorphicChain` entry in `Vm::polymorphic_chains`.
//! The chain holds entries [POLY_LIMIT..POLYMORPHIC_PROPERTY_CACHE_LIMIT].
//! Entries [0..POLY_LIMIT] stay inline in `NamedPropertyFeedback.entries`
//! to keep the asm fast path's sidecar (`polymorphic_own_data_handlers`)
//! addressable in the existing layout.
//!
//! On 9th distinct shape the IC transitions to Megamorphic and the chain
//! entry is dropped (caller's responsibility).

use lyng_objects::NamedPropertyCacheEntry;
use lyng_types::ShapeId;

use super::POLYMORPHIC_PROPERTY_CACHE_LIMIT;

/// Maximum number of out-of-line entries per chain.
/// `POLYMORPHIC_PROPERTY_CACHE_LIMIT - POLY_LIMIT` once flattened.
pub(crate) const POLYMORPHIC_CHAIN_CAP: usize = POLYMORPHIC_PROPERTY_CACHE_LIMIT - 2; // = 6

pub(crate) struct PolymorphicChain {
    entries: Vec<NamedPropertyCacheEntry>,
}

impl PolymorphicChain {
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::with_capacity(POLYMORPHIC_CHAIN_CAP),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn is_full(&self) -> bool {
        self.entries.len() >= POLYMORPHIC_CHAIN_CAP
    }

    /// Linear search by receiver shape. Chain is small (≤6) so linear beats
    /// binary search hash overhead. Returns `None` if no entry matches.
    pub(crate) fn find_by_shape(&self, receiver_shape: ShapeId) -> Option<&NamedPropertyCacheEntry> {
        self.entries
            .iter()
            .find(|entry| entry.receiver_shape() == receiver_shape)
    }

    /// Pushes a new entry. Caller must verify `!is_full()` before calling.
    pub(crate) fn push(&mut self, entry: NamedPropertyCacheEntry) {
        debug_assert!(self.entries.len() < POLYMORPHIC_CHAIN_CAP);
        self.entries.push(entry);
    }

    /// Iterator over entries — used by the slow path for fallback walks
    /// and by GC tracing to visit holder references.
    pub(crate) fn entries(&self) -> impl Iterator<Item = &NamedPropertyCacheEntry> {
        self.entries.iter()
    }
}

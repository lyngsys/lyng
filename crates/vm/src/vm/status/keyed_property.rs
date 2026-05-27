//! `KeyedPropertyStatus` projection (Spec 2 Phase E).

use lyng_common::AtomId;
use lyng_objects::ObjectFlags;
use lyng_types::ShapeId;

use crate::vm::feedback::{FeedbackInlineCacheState, FeedbackKeyedPropertyFamily};

use super::named_property::NamedPropertyHandlerSummary;

/// One entry in a `KeyedPropertyStatus`'s named-atom track.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedPropertyNamedStatusEntry {
    /// Atom name guarded by this entry.
    pub atom: AtomId,
    /// Receiver shape guarded by this entry.
    pub receiver_shape: ShapeId,
    /// Underlying named-property handler summary. `None` when the inline IC
    /// state tracks only the `(atom, shape)` pair without a cached entry
    /// payload (e.g. polymorphic sidecar slots).
    pub handler_summary: Option<NamedPropertyHandlerSummary>,
}

impl KeyedPropertyNamedStatusEntry {
    /// Convenience accessor — the cached atom name.
    #[inline]
    #[must_use]
    pub const fn atom(&self) -> AtomId {
        self.atom
    }

    /// Convenience accessor — the receiver shape guarded.
    #[inline]
    #[must_use]
    pub const fn receiver_shape(&self) -> ShapeId {
        self.receiver_shape
    }
}

/// One entry in a `KeyedPropertyStatus`'s dense-index track.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyedPropertyDenseStatusEntry {
    /// Receiver shape guarded by this entry.
    pub receiver_shape: ShapeId,
    /// Receiver flags guarded by this entry.
    pub receiver_flags: ObjectFlags,
}

impl KeyedPropertyDenseStatusEntry {
    /// Convenience accessor — the receiver shape guarded.
    #[inline]
    #[must_use]
    pub const fn receiver_shape(&self) -> ShapeId {
        self.receiver_shape
    }

    /// Convenience accessor — the receiver flags guarded.
    #[inline]
    #[must_use]
    pub const fn receiver_flags(&self) -> ObjectFlags {
        self.receiver_flags
    }
}

/// Status projection for one `KeyedPropertyAccess` IC slot.
///
/// The two tracks (`named_entries` and `dense_entries`) reflect the slot's
/// active family (`KeyedAtomMono` vs `KeyedDenseMono` etc.). At most one
/// track is non-empty in practice; both can be empty for Uninitialized /
/// Megamorphic / Generic states.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedPropertyStatus {
    pub state: FeedbackInlineCacheState,
    pub generation: u32,
    pub execution_count: u32,
    pub family: Option<FeedbackKeyedPropertyFamily>,
    pub named_entries: Vec<KeyedPropertyNamedStatusEntry>,
    pub dense_entries: Vec<KeyedPropertyDenseStatusEntry>,
}

impl KeyedPropertyStatus {
    /// Convenience accessor — the IC state machine variant.
    #[inline]
    #[must_use]
    pub const fn state(&self) -> FeedbackInlineCacheState {
        self.state
    }

    /// Convenience accessor — the IC family classifier, if any.
    #[inline]
    #[must_use]
    pub const fn family(&self) -> Option<FeedbackKeyedPropertyFamily> {
        self.family
    }

    /// Convenience accessor — named-atom entries.
    #[inline]
    #[must_use]
    pub fn named_entries(&self) -> &[KeyedPropertyNamedStatusEntry] {
        &self.named_entries
    }

    /// Convenience accessor — dense-index entries.
    #[inline]
    #[must_use]
    pub fn dense_entries(&self) -> &[KeyedPropertyDenseStatusEntry] {
        &self.dense_entries
    }
}

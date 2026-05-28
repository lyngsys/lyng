//! Rust-only state machine for a `NamedProperty` IC slot (Phase D.1.1).
//!
//! The asm-readable fields — `mode`, `generation`, `handler_bits`, `aux_bits`,
//! and `execution_count` — live on the slot's `PropertyMetadata` entry inside
//! `MetadataTable`. This struct holds everything else that the Rust slow-path
//! state machine needs.

use std::cmp::Ordering;

use lyng_objects::{
    NamedPropertyCacheEntry, NamedPropertyCachePath, NamedPropertyHandler,
    NamedPropertyInlineWriteHandler, NamedPropertyProtoHandler,
};
use lyng_types::ShapeId;

use crate::vm::feedback::{InlineCacheState, POLY_LIMIT};

/// Rust-only state-machine state for a Property IC slot.
///
/// The asm-readable fields (`mode`, `generation`, `handler_bits`, `aux_bits`,
/// `execution_count`) live on the slot's `PropertyMetadata` entry inside
/// `MetadataTable`. This struct is the SOLE source of truth for IC state-machine
/// transitions as of Phase D.2.4. After every transition, slow-path callers
/// must flush `PropertyMetadata` directly from this struct.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyIcState {
    pub cache_state: InlineCacheState,
    /// Total active entries (inline + out-of-line chain). The first
    /// `min(entry_count, POLY_LIMIT)` live inline in `entries`; the
    /// remainder live in `Vm::polymorphic_chains[(code, slot)]`.
    pub entry_count: u8,
    /// Inline polymorphic entries — at most `POLY_LIMIT` entries.
    pub entries: [Option<NamedPropertyCacheEntry>; POLY_LIMIT],
    /// Monomorphic OwnData sidecar. `NamedPropertyHandler::NONE` when not
    /// applicable (non-OwnData, Poly, Mega, or Uninit).
    pub monomorphic_own_data_handler: NamedPropertyHandler,
    /// Monomorphic one-hop PrototypeData sidecar.
    pub monomorphic_proto_data_handler: NamedPropertyProtoHandler,
    /// Monomorphic `OwnDataInlineWrite` sidecar (Spec 3 transition IC).
    /// `NamedPropertyInlineWriteHandler::NONE` when not applicable:
    /// non-OwnData/non-OwnDataTransition path, polymorphic, megamorphic,
    /// uninitialized, or out-of-line slot — see
    /// [`NamedPropertyInlineWriteHandler::from_entry`]. Asm-readable bits
    /// project into `PropertyMetadata` mode = 5 in Task 4; the write fast
    /// path in `op_assign_named_property_dsl` consumes them in Task 8.
    pub monomorphic_own_inline_write_handler: NamedPropertyInlineWriteHandler,
    /// Polymorphic OwnData sidecar — mirrors the first `POLY_LIMIT` entries
    /// when the cache is Polymorphic and the entry packs into a valid handler.
    pub polymorphic_own_data_handlers: [NamedPropertyHandler; POLY_LIMIT],
    /// Per-site install generation. Bumped on every install / re-install.
    /// `AdaptiveProtoLoad` watchpoints carry the generation at registration
    /// time and no-op when this has advanced past it (Phase A).
    pub generation: u32,
    /// Running execution count for this slot (not asm-readable; for Rust
    /// slow-path accounting and test helpers). Phase D.2.4 owns this field.
    pub execution_count: u32,
    /// When `true` the slot was cleared by a watchpoint fire and is awaiting
    /// re-initialization (replaces `FeedbackVector::clear_site` semantics).
    pub is_cleared: bool,
}

impl PropertyIcState {
    /// Constructs a fresh `PropertyIcState` in `Uninitialized` state.
    pub const fn new() -> Self {
        Self {
            cache_state: InlineCacheState::Uninitialized,
            entry_count: 0,
            entries: [None; POLY_LIMIT],
            monomorphic_own_data_handler: NamedPropertyHandler::NONE,
            monomorphic_proto_data_handler: NamedPropertyProtoHandler::NONE,
            monomorphic_own_inline_write_handler: NamedPropertyInlineWriteHandler::NONE,
            polymorphic_own_data_handlers: [NamedPropertyHandler::NONE; POLY_LIMIT],
            generation: 0,
            execution_count: 0,
            is_cleared: false,
        }
    }

    /// Number of inline entries currently populated: `min(entry_count, POLY_LIMIT)`.
    #[inline]
    pub fn inline_count(&self) -> usize {
        usize::from(self.entry_count).min(POLY_LIMIT)
    }

    /// Recompute every cache-hit sidecar from the current cache state.
    ///
    /// Mirrors `NamedPropertyFeedback::refresh_monomorphic_own_data_handler`.
    #[inline]
    pub const fn refresh_sidecars(&mut self) {
        self.monomorphic_own_data_handler = NamedPropertyHandler::NONE;
        self.monomorphic_proto_data_handler = NamedPropertyProtoHandler::NONE;
        self.monomorphic_own_inline_write_handler = NamedPropertyInlineWriteHandler::NONE;
        let mut i = 0;
        while i < POLY_LIMIT {
            self.polymorphic_own_data_handlers[i] = NamedPropertyHandler::NONE;
            i += 1;
        }
        match self.cache_state {
            InlineCacheState::Monomorphic => {
                let Some(entry) = self.entries[0] else {
                    return;
                };
                match entry.path() {
                    NamedPropertyCachePath::OwnData => {
                        self.monomorphic_own_data_handler = NamedPropertyHandler::from_entry(entry);
                        self.monomorphic_own_inline_write_handler =
                            NamedPropertyInlineWriteHandler::from_entry(entry);
                    }
                    NamedPropertyCachePath::OwnDataTransition => {
                        self.monomorphic_own_inline_write_handler =
                            NamedPropertyInlineWriteHandler::from_entry(entry);
                    }
                    NamedPropertyCachePath::PrototypeData => {
                        let handler = NamedPropertyProtoHandler::from_entry(entry);
                        if handler.is_valid() {
                            self.monomorphic_proto_data_handler = handler;
                        }
                    }
                }
            }
            InlineCacheState::Polymorphic => {
                let active = self.entry_count as usize;
                let limit = if active < POLY_LIMIT {
                    active
                } else {
                    POLY_LIMIT
                };
                let mut idx = 0;
                while idx < limit {
                    if let Some(entry) = self.entries[idx] {
                        let handler = NamedPropertyHandler::from_entry(entry);
                        if handler.is_valid() {
                            self.polymorphic_own_data_handlers[idx] = handler;
                        }
                    }
                    idx += 1;
                }
            }
            InlineCacheState::Uninitialized | InlineCacheState::Megamorphic => {}
        }
    }

    /// Transitions `Uninitialized` → `Monomorphic` with the first entry.
    /// Caller must only invoke this when `cache_state == Uninitialized`.
    #[inline]
    pub const fn install_first_entry(&mut self, entry: NamedPropertyCacheEntry) {
        self.entries[0] = Some(entry);
        self.entry_count = 1;
        self.cache_state = InlineCacheState::Monomorphic;
        self.refresh_sidecars();
    }

    /// Transitions the cache to `Megamorphic` and clears inline entries.
    ///
    /// The out-of-line `Vm::polymorphic_chains[(code, slot)]` entry — if any
    /// — is the **caller's** responsibility to drop.
    #[inline]
    pub const fn promote_to_megamorphic(&mut self) {
        self.cache_state = InlineCacheState::Megamorphic;
        self.entry_count = 0;
        self.entries = [None; POLY_LIMIT];
        self.refresh_sidecars();
    }

    /// Binary search over the inline entries slice. Returns `Ok(index)` on a
    /// shape hit, `Err(insertion_point)` otherwise — same semantics as
    /// `slice::binary_search_by`.
    #[inline]
    pub fn search_entry_index(&self, receiver_shape: ShapeId) -> Result<usize, usize> {
        let inline = self.inline_count();
        for index in 0..inline {
            let Some(entry) = self.entries[index] else {
                continue;
            };
            match entry.receiver_shape().cmp(&receiver_shape) {
                Ordering::Equal => return Ok(index),
                Ordering::Greater => return Err(index),
                Ordering::Less => {}
            }
        }
        Err(inline)
    }
}

impl Default for PropertyIcState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use lyng_objects::{
        INLINE_SLOT_OFFSET_FLAG, NamedPropertyCacheEntry, NamedPropertyCachePath,
        NamedPropertyInlineWriteHandler, PROPERTY_CACHE_MAX_DEPENDENCIES,
    };
    use lyng_types::{DescriptorAttributes, ObjectRef, ShapeId};

    use super::*;

    fn writable_attrs() -> DescriptorAttributes {
        let mut attrs = DescriptorAttributes::empty();
        attrs.set_writable(true);
        attrs
    }

    #[test]
    fn refresh_sidecars_populates_monomorphic_own_inline_write_handler() {
        let source = ShapeId::from_raw(7).expect("non-zero");
        let target = ShapeId::from_raw(11).expect("non-zero");
        let entry = NamedPropertyCacheEntry::new_for_test(
            source,
            ObjectRef::from_raw(1).expect("non-zero"),
            target,
            INLINE_SLOT_OFFSET_FLAG | 2,
            writable_attrs(),
            NamedPropertyCachePath::OwnDataTransition,
            // dependency_count = 1 is required by NamedPropertyInlineWriteHandler::from_entry
            // (it returns NONE for any count != 1). The dependency slots are left as None because
            // from_entry only checks the count; this fixture predates a full dependency mock.
            1,
            [None; PROPERTY_CACHE_MAX_DEPENDENCIES],
        );

        let mut state = PropertyIcState::new();
        state.cache_state = InlineCacheState::Monomorphic;
        state.entry_count = 1;
        state.entries[0] = Some(entry);
        state.refresh_sidecars();

        let expected = NamedPropertyInlineWriteHandler::from_entry(entry);
        assert_eq!(state.monomorphic_own_inline_write_handler, expected);
        assert!(state.monomorphic_own_inline_write_handler.is_valid());
    }

    #[test]
    fn refresh_sidecars_populates_inline_write_handler_for_own_data_too() {
        // Non-transitioning write: receiver_shape == holder_shape.
        let shape = ShapeId::from_raw(42).expect("non-zero");
        let entry = NamedPropertyCacheEntry::new_for_test(
            shape,
            ObjectRef::from_raw(1).expect("non-zero"),
            shape, // holder == receiver — no transition
            INLINE_SLOT_OFFSET_FLAG | 0,
            writable_attrs(),
            NamedPropertyCachePath::OwnData,
            // dependency_count = 1 is required by NamedPropertyInlineWriteHandler::from_entry
            // (it returns NONE for any count != 1). The dependency slots are left as None because
            // from_entry only checks the count; this fixture predates a full dependency mock.
            1,
            [None; PROPERTY_CACHE_MAX_DEPENDENCIES],
        );

        let mut state = PropertyIcState::new();
        state.cache_state = InlineCacheState::Monomorphic;
        state.entry_count = 1;
        state.entries[0] = Some(entry);
        state.refresh_sidecars();

        let expected = NamedPropertyInlineWriteHandler::from_entry(entry);
        assert_eq!(state.monomorphic_own_inline_write_handler, expected);
        assert!(state.monomorphic_own_inline_write_handler.is_valid());
        // Verify source == target for the non-transition case.
        assert_eq!(
            state.monomorphic_own_inline_write_handler.source_shape(),
            state.monomorphic_own_inline_write_handler.target_shape(),
        );
    }
}

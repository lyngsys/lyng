//! Rust-only state machine for a `KeyedProperty` IC slot (Phase D.1.3).
//!
//! The asm-readable fields — `mode`, `generation`, `handler_bits`, and
//! `execution_count` — live on the slot's `KeyedPropertyMetadata` entry inside
//! `MetadataTable`. This struct holds the remaining Rust-only state-machine
//! fields. Both named-atom and dense-index keyed access share this type; the
//! family distinction is encoded by the `family` field.

use lyng_objects::{
    KeyedDenseIndexHandler, NamedPropertyHandler, NamedPropertyProtoHandler, ObjectFlags,
};
use lyng_types::ShapeId;

use crate::vm::feedback::{InlineCacheState, POLY_LIMIT};

/// Full polymorphic named/dense entry capacity, mirroring the private
/// `KEYED_ENTRY_LIMIT` in `feedback`.
const KEYED_ENTRY_LIMIT: usize = 8;

/// An entry in the named-atom keyed IC cache: pairs a runtime atom with a
/// named-property cache entry for a receiver shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyedIcNamedEntry {
    /// Raw `AtomId` value. Compared against the runtime atom on cache-hit.
    pub atom_raw: u32,
    /// Receiver shape for this entry. Used for binary-search / linear scan.
    pub receiver_shape: ShapeId,
}

/// An entry in the dense-index keyed IC cache.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyedIcDenseEntry {
    /// Receiver shape guarded by this entry.
    pub receiver_shape: ShapeId,
    /// Receiver flags guarded by this entry.
    pub receiver_flags: ObjectFlags,
}

/// Which family of keys this keyed-property IC slot is tracking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyedIcFamily {
    DenseIndex,
    NamedAtom,
    Generic,
}

/// Rust-only state-machine state for a `KeyedProperty` IC slot.
///
/// The asm-readable fields (`mode`, `generation`, `handler_bits`,
/// `execution_count`) live on the slot's `KeyedPropertyMetadata` entry inside
/// `MetadataTable`. This struct is the SOLE source of truth for IC state-machine
/// transitions as of Phase D.2.4. After every transition, slow-path callers
/// must flush `KeyedPropertyMetadata` directly from this struct.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedPropertyIcState {
    pub cache_state: InlineCacheState,
    /// Which key family this site has converged on (`None` = Uninitialized).
    pub family: Option<KeyedIcFamily>,
    /// Number of active named-atom cache entries (`named_entries` head).
    pub named_entry_count: u8,
    /// Inline named-atom entries (up to `KEYED_ENTRY_LIMIT`).
    pub named_entries: [Option<KeyedIcNamedEntry>; KEYED_ENTRY_LIMIT],
    /// Number of active dense-index cache entries.
    pub dense_entry_count: u8,
    /// Inline dense-index entries.
    pub dense_entries: [Option<KeyedIcDenseEntry>; KEYED_ENTRY_LIMIT],
    /// Monomorphic named-atom OwnData sidecar.
    pub monomorphic_named_own_data_handler: NamedPropertyHandler,
    /// Raw atom id for the monomorphic named-atom entry. `0` = absent.
    pub monomorphic_named_atom: u32,
    /// Monomorphic named-atom one-hop PrototypeData sidecar.
    pub monomorphic_named_proto_data_handler: NamedPropertyProtoHandler,
    /// Monomorphic dense-index sidecar.
    pub monomorphic_dense_index_handler: KeyedDenseIndexHandler,
    /// Polymorphic named-atom OwnData sidecars (first `POLY_LIMIT` entries).
    pub polymorphic_named_own_data_handlers: [NamedPropertyHandler; POLY_LIMIT],
    /// Raw atom ids paired with `polymorphic_named_own_data_handlers`. `0` = absent.
    pub polymorphic_named_atoms: [u32; POLY_LIMIT],
    /// Polymorphic dense-index sidecars (first `POLY_LIMIT` entries).
    pub polymorphic_dense_index_handlers: [KeyedDenseIndexHandler; POLY_LIMIT],
    /// Running execution count for this slot (Rust-side accounting). Phase D.2.4 owns this field.
    pub execution_count: u32,
}

impl KeyedPropertyIcState {
    /// Constructs a fresh `KeyedPropertyIcState` in `Uninitialized` state.
    pub const fn new() -> Self {
        Self {
            cache_state: InlineCacheState::Uninitialized,
            family: None,
            named_entry_count: 0,
            named_entries: [None; KEYED_ENTRY_LIMIT],
            dense_entry_count: 0,
            dense_entries: [None; KEYED_ENTRY_LIMIT],
            monomorphic_named_own_data_handler: NamedPropertyHandler::NONE,
            monomorphic_named_atom: 0,
            monomorphic_named_proto_data_handler: NamedPropertyProtoHandler::NONE,
            monomorphic_dense_index_handler: KeyedDenseIndexHandler::NONE,
            polymorphic_named_own_data_handlers: [NamedPropertyHandler::NONE; POLY_LIMIT],
            polymorphic_named_atoms: [0; POLY_LIMIT],
            polymorphic_dense_index_handlers: [KeyedDenseIndexHandler::NONE; POLY_LIMIT],
            execution_count: 0,
        }
    }
}

impl Default for KeyedPropertyIcState {
    fn default() -> Self {
        Self::new()
    }
}

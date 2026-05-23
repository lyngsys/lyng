//! Allocation, rooting, tracing, and storage ownership for lyng-js runtime domains.
//!
//! Ownership: `lyng_js_gc` owns heap policy and dereference paths for runtime
//! data types defined in `lyng_js_types`.

#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    reason = "GC substrate APIs expose compact handles and reporting accessors where names must stay domain-specific"
)]

use std::{cell::OnceCell, fmt};

use lyng_js_common::{AtomCollection, AtomId, AtomSweepStats, AtomTable};
use lyng_js_types::StringRef;

mod arena;
mod card_table;
mod collection;
mod concurrent_sweep;
mod mutator;
mod nursery;
mod rooting;
mod weak;
mod writer;

pub use arena::{
    AllocationLifetime, BigIntSign, CodeSlotsRef, EnvironmentSlotsRef, FunctionPayloadRef,
    HeapGeneration, ObjectSlotsRef, PrimitiveBigIntRecord, PrimitiveBigIntView,
    PrimitiveDomainStats, PrimitiveHeap, PrimitiveSymbolClass, PrimitiveSymbolRecord,
    PrimitiveSymbolView, PrimitiveValueCellRecord, PrimitiveValueCellRef,
    RuntimeBoundFunctionRecord, RuntimeCodeRecord, RuntimeEnvironmentRecord, RuntimeFunctionRecord,
    RuntimeObjectRecord, RuntimeRealmRecord, RuntimeShapeRecord, RuntimeSuspendedExecutionRecord,
    SideAllocationClass, SideAllocationRef, SideAllocationStats, SuspendedRegistersRef,
    SymbolFlags, PRIMITIVE_SLOTS_PER_PAGE, RUNTIME_OBJECT_INLINE_SLOT_COUNT,
};
pub use collection::{
    PrimitiveCollectionKind, PrimitiveCollectionReport, PrimitiveCollectionTrigger,
    PrimitiveDomainAccounting, PrimitiveHeapAccounting, PrimitiveMinorCollectionStats,
};
pub use concurrent_sweep::PrimitiveBackgroundSweepStats;
pub use mutator::{
    CodeHandleStoreTarget, EnvironmentHandleStoreTarget, ObjectHandleStoreTarget,
    ObjectSlotsHandleStoreTarget, PrimitiveHeapView, PrimitiveMutator, RealmHandleStoreTarget,
    ShapeHandleStoreTarget, StringHandleStoreTarget, ValueStoreTarget,
};
pub use nursery::{NurseryStats, PrimitiveAllocationProfile};
pub use rooting::{
    PrimitiveCollectionStats, PrimitiveIncrementalMark, PrimitiveMarkProgress, PrimitiveMarkStep,
    PrimitiveRootGuard, PrimitiveRootScope, PrimitiveRoots, PrimitiveTraceStats, PrimitiveTracer,
    TraceHeapEdges,
};
pub use weak::WeakHeapRef;
pub use writer::{HeapRef, HeapWriter};

/// GC-side driver for the shared atom-table collection contract.
pub struct AtomGcSweep<'a> {
    collection: AtomCollection<'a>,
}

/// String encoding used by immutable runtime string records.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StringEncoding {
    Latin1,
    Utf16,
}

/// Storage backing an immutable runtime string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PrimitiveStringStorage {
    Empty,
    Flat(SideAllocationRef),
    Cons { left: StringRef, right: StringRef },
}

/// Shared tracing hook for primitive records or metadata that retain `AtomId` edges.
pub trait TraceAtomEdges {
    fn trace_atom_edges(&self, sweep: &mut AtomGcSweep<'_>);
}

/// Minimal runtime string record with a cached atom edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimitiveStringRecord {
    encoding: StringEncoding,
    code_unit_len: u32,
    cached_hash: Option<u32>,
    cached_atom: Option<AtomId>,
    storage: PrimitiveStringStorage,
}

/// Read-only borrowed view over an immutable runtime string record.
pub struct PrimitiveStringView<'a> {
    record: PrimitiveStringRecord,
    payload: Option<&'a [u8]>,
    heap: Option<&'a PrimitiveHeap>,
    flattened_payload: OnceCell<Option<Vec<u8>>>,
}

impl fmt::Debug for PrimitiveStringView<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PrimitiveStringView")
            .field("record", &self.record)
            .field("has_flat_payload", &self.payload.is_some())
            .finish_non_exhaustive()
    }
}

/// Minimal stand-in for other `AtomId`-bearing primitive metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimitiveAtomMetadata {
    retained_atom: Option<AtomId>,
}

impl<'a> AtomGcSweep<'a> {
    #[inline]
    pub fn new(atoms: &'a mut AtomTable) -> Self {
        Self {
            collection: atoms.start_collection(),
        }
    }

    #[inline]
    pub fn visit_atom(&mut self, id: AtomId) {
        self.collection.visit_atom(id);
    }

    #[inline]
    pub fn sweep(self) -> AtomSweepStats {
        self.collection.sweep()
    }
}

impl PrimitiveStringRecord {
    const fn from_parts(
        encoding: StringEncoding,
        code_unit_len: u32,
        cached_hash: Option<u32>,
        cached_atom: Option<AtomId>,
        storage: PrimitiveStringStorage,
    ) -> Self {
        Self {
            encoding,
            code_unit_len,
            cached_hash,
            cached_atom,
            storage,
        }
    }

    #[inline]
    pub const fn new(encoding: StringEncoding, code_unit_len: u32) -> Self {
        Self::from_parts(
            encoding,
            code_unit_len,
            None,
            None,
            PrimitiveStringStorage::Empty,
        )
    }

    #[inline]
    pub const fn with_cached_atom(
        encoding: StringEncoding,
        code_unit_len: u32,
        cached_atom: AtomId,
    ) -> Self {
        Self::from_parts(
            encoding,
            code_unit_len,
            None,
            Some(cached_atom),
            PrimitiveStringStorage::Empty,
        )
    }

    #[inline]
    pub const fn with_payload(
        encoding: StringEncoding,
        code_unit_len: u32,
        cached_atom: Option<AtomId>,
        payload: SideAllocationRef,
    ) -> Self {
        Self::from_parts(
            encoding,
            code_unit_len,
            None,
            cached_atom,
            PrimitiveStringStorage::Flat(payload),
        )
    }

    #[inline]
    pub const fn with_cons(
        encoding: StringEncoding,
        code_unit_len: u32,
        left: StringRef,
        right: StringRef,
    ) -> Self {
        Self::from_parts(
            encoding,
            code_unit_len,
            None,
            None,
            PrimitiveStringStorage::Cons { left, right },
        )
    }

    #[inline]
    pub const fn encoding(self) -> StringEncoding {
        self.encoding
    }

    #[inline]
    pub const fn code_unit_len(self) -> u32 {
        self.code_unit_len
    }

    #[inline]
    pub const fn cached_hash(self) -> Option<u32> {
        self.cached_hash
    }

    #[inline]
    pub const fn cached_atom(self) -> Option<AtomId> {
        self.cached_atom
    }

    #[inline]
    pub const fn payload(self) -> Option<SideAllocationRef> {
        match self.storage {
            PrimitiveStringStorage::Flat(payload) => Some(payload),
            PrimitiveStringStorage::Empty | PrimitiveStringStorage::Cons { .. } => None,
        }
    }

    #[inline]
    pub const fn is_flat(self) -> bool {
        matches!(
            self.storage,
            PrimitiveStringStorage::Empty | PrimitiveStringStorage::Flat(_)
        )
    }

    #[inline]
    pub const fn is_cons(self) -> bool {
        matches!(self.storage, PrimitiveStringStorage::Cons { .. })
    }

    #[inline]
    pub const fn cons_children(self) -> Option<(StringRef, StringRef)> {
        match self.storage {
            PrimitiveStringStorage::Cons { left, right } => Some((left, right)),
            PrimitiveStringStorage::Empty | PrimitiveStringStorage::Flat(_) => None,
        }
    }
}

impl<'a> PrimitiveStringView<'a> {
    pub(crate) fn new(record: PrimitiveStringRecord, payload: &'a [u8]) -> Self {
        debug_assert_eq!(
            payload.len(),
            expected_string_payload_len(record.encoding(), record.code_unit_len()),
            "string view payload length must match record encoding and code-unit length"
        );

        Self {
            record,
            payload: Some(payload),
            heap: None,
            flattened_payload: OnceCell::new(),
        }
    }

    pub(crate) fn with_heap(
        record: PrimitiveStringRecord,
        payload: Option<&'a [u8]>,
        heap: &'a PrimitiveHeap,
    ) -> Self {
        if let Some(payload) = payload {
            debug_assert_eq!(
                payload.len(),
                expected_string_payload_len(record.encoding(), record.code_unit_len()),
                "string view payload length must match record encoding and code-unit length"
            );
        } else {
            debug_assert!(
                record.is_cons(),
                "only cons string views may defer payload materialization"
            );
        }

        Self {
            record,
            payload,
            heap: Some(heap),
            flattened_payload: OnceCell::new(),
        }
    }

    #[inline]
    pub const fn record(&self) -> PrimitiveStringRecord {
        self.record
    }

    #[inline]
    pub const fn encoding(&self) -> StringEncoding {
        self.record.encoding()
    }

    #[inline]
    pub const fn code_unit_len(&self) -> u32 {
        self.record.code_unit_len()
    }

    #[inline]
    pub const fn cached_hash(&self) -> Option<u32> {
        self.record.cached_hash()
    }

    #[inline]
    pub const fn cached_atom(&self) -> Option<AtomId> {
        self.record.cached_atom()
    }

    #[inline]
    pub fn payload_bytes(&self) -> Option<&[u8]> {
        self.materialized_payload()
    }

    #[inline]
    pub const fn flat_latin1_bytes(&self) -> Option<&'a [u8]> {
        match self.encoding() {
            StringEncoding::Latin1 => self.payload,
            StringEncoding::Utf16 => None,
        }
    }

    #[inline]
    pub fn latin1_bytes(&self) -> Option<&[u8]> {
        match self.encoding() {
            StringEncoding::Latin1 => self.materialized_payload(),
            StringEncoding::Utf16 => None,
        }
    }

    #[inline]
    pub fn utf16_bytes(&self) -> Option<&[u8]> {
        match self.encoding() {
            StringEncoding::Latin1 => None,
            StringEncoding::Utf16 => self.materialized_payload(),
        }
    }

    pub fn code_unit_at(&self, index: usize) -> Option<u16> {
        if index >= self.code_unit_len() as usize {
            return None;
        }

        if let Some(payload) = self.payload {
            return match self.encoding() {
                StringEncoding::Latin1 => payload.get(index).copied().map(u16::from),
                StringEncoding::Utf16 => utf16_code_unit_at(payload, index),
            };
        }

        let heap = self.heap?;
        let (left, right) = self.record.cons_children()?;
        let left_record = heap.string(left)?;
        let left_len = left_record.code_unit_len() as usize;
        if index < left_len {
            heap.string_view(left)?.code_unit_at(index)
        } else {
            heap.string_view(right)?.code_unit_at(index - left_len)
        }
    }

    #[inline]
    pub fn compute_hash(&self) -> u32 {
        deterministic_string_hash(self)
    }

    pub fn equals(&self, other: &PrimitiveStringView<'_>) -> bool {
        if self.code_unit_len() != other.code_unit_len() {
            return false;
        }

        match (self.cached_hash(), other.cached_hash()) {
            (Some(left), Some(right)) if left != right => return false,
            _ => {}
        }

        if self.encoding() == other.encoding()
            && let (Some(left), Some(right)) =
                (self.materialized_payload(), other.materialized_payload())
        {
            return left == right;
        }

        for index in 0..self.code_unit_len() as usize {
            if self.code_unit_at(index) != other.code_unit_at(index) {
                return false;
            }
        }

        true
    }

    fn materialized_payload(&self) -> Option<&[u8]> {
        if let Some(payload) = self.payload {
            return Some(payload);
        }

        self.flattened_payload
            .get_or_init(|| {
                let heap = self.heap?;
                heap.flatten_string_payload(self.record)
            })
            .as_deref()
    }
}

impl PrimitiveAtomMetadata {
    #[inline]
    pub const fn new(retained_atom: Option<AtomId>) -> Self {
        Self { retained_atom }
    }

    #[inline]
    pub const fn retained_atom(self) -> Option<AtomId> {
        self.retained_atom
    }
}

impl TraceAtomEdges for AtomId {
    fn trace_atom_edges(&self, sweep: &mut AtomGcSweep<'_>) {
        sweep.visit_atom(*self);
    }
}

impl TraceAtomEdges for PrimitiveStringRecord {
    fn trace_atom_edges(&self, sweep: &mut AtomGcSweep<'_>) {
        self.cached_atom.trace_atom_edges(sweep);
    }
}

impl TraceAtomEdges for PrimitiveAtomMetadata {
    fn trace_atom_edges(&self, sweep: &mut AtomGcSweep<'_>) {
        self.retained_atom.trace_atom_edges(sweep);
    }
}

impl TraceAtomEdges for Option<AtomId> {
    fn trace_atom_edges(&self, sweep: &mut AtomGcSweep<'_>) {
        if let Some(id) = self {
            sweep.visit_atom(*id);
        }
    }
}

fn utf16_code_unit_at(payload: &[u8], index: usize) -> Option<u16> {
    let start = index.checked_mul(2)?;
    let lo = *payload.get(start)?;
    let hi = *payload.get(start + 1)?;
    Some(u16::from_le_bytes([lo, hi]))
}

fn deterministic_string_hash(view: &PrimitiveStringView<'_>) -> u32 {
    const OFFSET_BASIS: u32 = 2_166_136_261;
    const FNV_PRIME: u32 = 16_777_619;

    let mut hash = OFFSET_BASIS;
    for index in 0..view.code_unit_len() as usize {
        let unit = view
            .code_unit_at(index)
            .expect("string view iteration must stay in bounds");
        let [lo, hi] = unit.to_le_bytes();
        hash ^= u32::from(lo);
        hash = hash.wrapping_mul(FNV_PRIME);
        hash ^= u32::from(hi);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash ^= view.code_unit_len();
    hash.wrapping_mul(FNV_PRIME)
}

const fn expected_string_payload_len(encoding: StringEncoding, code_unit_len: u32) -> usize {
    match encoding {
        StringEncoding::Latin1 => code_unit_len as usize,
        StringEncoding::Utf16 => (code_unit_len as usize) * 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_common::AtomLifetime;
    use lyng_js_types::{ObjectRef, Value};

    #[test]
    fn heap_writer_updates_value_and_typed_ref_slots_as_chokepoint() {
        let mut writer = HeapWriter::new();

        let mut value_slot = Value::undefined();
        writer.write_value(&mut value_slot, Value::from_smi(7));
        assert_eq!(value_slot, Value::from_smi(7));

        let object = ObjectRef::from_raw(1).expect("test object ref should be non-zero");
        let mut optional_ref = None;
        writer.write_ref(&mut optional_ref, Some(object));
        assert_eq!(optional_ref, Some(object));

        let replacement = ObjectRef::from_raw(2).expect("test object ref should be non-zero");
        let mut direct_ref = object;
        writer.write_ref(&mut direct_ref, replacement);
        assert_eq!(direct_ref, replacement);
    }

    #[test]
    fn gc_preserves_live_collectible_atoms_and_reclaims_dead_ones() {
        let mut atoms = AtomTable::new();
        let permanent = atoms.intern("frontend-name");
        let live = atoms.intern_collectible("live-name");
        let dead = atoms.intern_collectible("dead-name");

        let mut sweep = AtomGcSweep::new(&mut atoms);
        sweep.visit_atom(permanent);
        sweep.visit_atom(live);
        let stats = sweep.sweep();

        assert_eq!(stats.reclaimed_collectible, 1);
        assert_eq!(stats.retained_collectible, 1);
        assert_eq!(atoms.resolve(live), "live-name");
        assert_eq!(atoms.lifetime(live), Some(AtomLifetime::Collectible));
        assert_eq!(atoms.get(dead), None);
        assert_eq!(atoms.lifetime(permanent), Some(AtomLifetime::Permanent));
    }

    #[test]
    fn string_atom_cache_traces_collectible_atom_liveness() {
        let mut atoms = AtomTable::new();
        let cached = atoms.intern_collectible("cache-key");
        let record = PrimitiveStringRecord::with_cached_atom(StringEncoding::Latin1, 9, cached);

        let mut sweep = AtomGcSweep::new(&mut atoms);
        record.trace_atom_edges(&mut sweep);
        let stats = sweep.sweep();

        assert_eq!(record.encoding(), StringEncoding::Latin1);
        assert_eq!(record.code_unit_len(), 9);
        assert_eq!(record.cached_hash(), None);
        assert_eq!(record.cached_atom(), Some(cached));
        assert_eq!(stats.reclaimed_collectible, 0);
        assert_eq!(stats.retained_collectible, 1);
        assert_eq!(atoms.resolve(cached), "cache-key");
        assert_eq!(atoms.lifetime(cached), Some(AtomLifetime::Collectible));
    }

    #[test]
    fn mixed_primitive_atom_edges_keep_live_atoms_and_release_dead_ones() {
        let mut atoms = AtomTable::new();
        let cached_collectible = atoms.intern_collectible("live-cache");
        let dead_collectible = atoms.intern_collectible("dead-cache");
        let permanent = atoms.intern("builtin-name");

        let string_record =
            PrimitiveStringRecord::with_cached_atom(StringEncoding::Utf16, 10, cached_collectible);
        let metadata = PrimitiveAtomMetadata::new(Some(permanent));

        let mut sweep = AtomGcSweep::new(&mut atoms);
        string_record.trace_atom_edges(&mut sweep);
        metadata.trace_atom_edges(&mut sweep);
        let stats = sweep.sweep();

        assert_eq!(stats.reclaimed_collectible, 1);
        assert_eq!(stats.retained_collectible, 1);
        assert_eq!(atoms.resolve(cached_collectible), "live-cache");
        assert_eq!(
            atoms.lifetime(cached_collectible),
            Some(AtomLifetime::Collectible)
        );
        assert_eq!(atoms.get(dead_collectible), None);
        assert_eq!(atoms.lifetime(permanent), Some(AtomLifetime::Permanent));
        assert_eq!(metadata.retained_atom(), Some(permanent));

        let replacement = atoms.intern_collectible("replacement-cache");
        assert_eq!(replacement, dead_collectible);
        assert_eq!(
            atoms.resolve(string_record.cached_atom().unwrap()),
            "live-cache"
        );
    }

    #[test]
    fn borrowed_string_views_hash_and_compare_by_utf16_code_units() {
        let latin1 = PrimitiveStringView::new(
            PrimitiveStringRecord::new(StringEncoding::Latin1, 3),
            b"abc",
        );
        let utf16 = PrimitiveStringView::new(
            PrimitiveStringRecord::new(StringEncoding::Utf16, 3),
            &[0x61, 0x00, 0x62, 0x00, 0x63, 0x00],
        );
        let different = PrimitiveStringView::new(
            PrimitiveStringRecord::new(StringEncoding::Latin1, 3),
            b"abd",
        );

        assert_eq!(latin1.code_unit_at(0), Some(0x61));
        assert_eq!(latin1.code_unit_at(2), Some(0x63));
        assert_eq!(utf16.code_unit_at(1), Some(0x62));
        assert_eq!(utf16.code_unit_at(3), None);
        assert_eq!(latin1.compute_hash(), utf16.compute_hash());
        assert!(latin1.equals(&utf16));
        assert!(!latin1.equals(&different));
    }
}

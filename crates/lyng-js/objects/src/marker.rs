use crate::{ObjectFlags, ObjectKind};
use lyng_js_common::AtomId;
use lyng_js_gc::{AtomGcSweep, PrimitiveHeapMarker, TraceAtomEdges};

/// Minimal placeholder proving `lyng_js_objects` layers on the Phase 2 heap
/// substrate without depending on the later Phase 3 env/host crates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectSubstrateMarker {
    heap: PrimitiveHeapMarker,
    property_name: AtomId,
    kind: ObjectKind,
    flags: ObjectFlags,
}

impl ObjectSubstrateMarker {
    #[inline]
    pub const fn new(
        heap: PrimitiveHeapMarker,
        property_name: AtomId,
        kind: ObjectKind,
        flags: ObjectFlags,
    ) -> Self {
        Self {
            heap,
            property_name,
            kind,
            flags,
        }
    }

    #[inline]
    pub const fn heap(self) -> PrimitiveHeapMarker {
        self.heap
    }

    #[inline]
    pub const fn property_name(self) -> AtomId {
        self.property_name
    }

    #[inline]
    pub const fn kind(self) -> ObjectKind {
        self.kind
    }

    #[inline]
    pub const fn flags(self) -> ObjectFlags {
        self.flags
    }
}

impl TraceAtomEdges for ObjectSubstrateMarker {
    fn trace_atom_edges(&self, sweep: &mut AtomGcSweep<'_>) {
        self.heap.trace_atom_edges(sweep);
        self.property_name.trace_atom_edges(sweep);
    }
}

use lyng_js_common::AtomId;
use lyng_js_gc::{AtomGcSweep, PrimitiveHeapMarker, TraceAtomEdges};

/// Minimal placeholder proving `lyng_js_ops` layers on the Phase 2 runtime crates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimitiveOpsMarker {
    heap: PrimitiveHeapMarker,
    property_name: AtomId,
}

impl PrimitiveOpsMarker {
    #[inline]
    pub const fn new(heap: PrimitiveHeapMarker, property_name: AtomId) -> Self {
        Self {
            heap,
            property_name,
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
}

impl TraceAtomEdges for PrimitiveOpsMarker {
    fn trace_atom_edges(&self, sweep: &mut AtomGcSweep<'_>) {
        self.heap.trace_atom_edges(sweep);
        self.property_name.trace_atom_edges(sweep);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_common::{AtomLifetime, AtomTable, SourceId};
    use lyng_js_types::TypeOwnershipMarker;
    use std::mem::size_of;

    #[test]
    fn ops_marker_round_trips_heap_and_property_name() {
        let property_name = AtomId::from_raw(31);
        let heap =
            PrimitiveHeapMarker::new(TypeOwnershipMarker::new(property_name), SourceId::new(5));
        let marker = PrimitiveOpsMarker::new(heap, property_name);

        assert_eq!(marker.heap(), heap);
        assert_eq!(marker.property_name(), property_name);
        assert_eq!(
            size_of::<PrimitiveOpsMarker>(),
            size_of::<PrimitiveHeapMarker>() + size_of::<AtomId>()
        );
    }

    #[test]
    fn ops_marker_traces_nested_atom_edges() {
        let mut atoms = AtomTable::new();
        let type_atom = atoms.intern_collectible("type-atom");
        let property_atom = atoms.intern_collectible("property-atom");
        let dead_atom = atoms.intern_collectible("dead-atom");
        let heap = PrimitiveHeapMarker::new(TypeOwnershipMarker::new(type_atom), SourceId::new(9));
        let marker = PrimitiveOpsMarker::new(heap, property_atom);

        let mut sweep = AtomGcSweep::new(&mut atoms);
        marker.trace_atom_edges(&mut sweep);
        let stats = sweep.sweep();

        assert_eq!(stats.reclaimed_collectible, 1);
        assert_eq!(stats.retained_collectible, 2);
        assert_eq!(atoms.lifetime(type_atom), Some(AtomLifetime::Collectible));
        assert_eq!(
            atoms.lifetime(property_atom),
            Some(AtomLifetime::Collectible)
        );
        assert_eq!(atoms.get(dead_atom), None);
    }
}

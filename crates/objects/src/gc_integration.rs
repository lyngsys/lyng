//! GC integration for the objects layer.
//!
//! The GC mark walk (`crates/gc`) only traces edges that live in a heap-allocated
//! `RuntimeObjectRecord` (named slots, inline slots, prototype, elements, private
//! slots, payloads). Dictionary-mode property values, however, live OUTSIDE the GC
//! heap — in `ObjectRuntime::object_metadata`, an agent-side Rust side-table. Without
//! a dedicated hook the GC cannot see them, so any value reachable ONLY through a
//! dictionary entry would be collected (see Task 0.1's characterization).
//!
//! `lyng_gc::TraceObjectMetadataEdges` is the callback the GC mark loop invokes for
//! every live object as it is processed. `ObjectRuntime` implements it here, walking
//! the object's dictionary entries (if any) and marking each entry's `Value`s. The
//! env layer wires `&self.objects` into the collection as the metadata tracer (see
//! `crates/env/src/agent/weak_finalization.rs`).
//!
//! ## Scope: major collection only (known minor-GC limitation)
//!
//! Major collection traces dictionary metadata edges via this hook. Minor (nursery)
//! collection does NOT — dictionary writes don't dirty cards and `PrimitiveMinorTracer`
//! doesn't visit metadata. A young value reachable only through a dictionary entry
//! can still be collected by a minor GC. Tracked as a known limitation; global
//! property cells avoid it by allocating cells tenured.

use lyng_gc::{PrimitiveTracer, TraceObjectMetadataEdges};
use lyng_types::ObjectRef;

use crate::object_metadata::NamedPropertyStorage;
use crate::runtime::ObjectRuntime;
use crate::shapes::NamedPropertyValue;

impl TraceObjectMetadataEdges for ObjectRuntime {
    fn trace_object_metadata_edges(&self, object: ObjectRef, tracer: &mut PrimitiveTracer<'_>) {
        let Some(metadata) = self.object_metadata(object) else {
            return;
        };
        let NamedPropertyStorage::Dictionary(dict) = &metadata.named_properties else {
            return;
        };
        for entry in dict.entries.values() {
            match entry.payload {
                NamedPropertyValue::Data(value) => tracer.mark_value(value),
                NamedPropertyValue::Accessor { get, set } => {
                    tracer.mark_value(get);
                    tracer.mark_value(set);
                }
            }
        }
    }
}

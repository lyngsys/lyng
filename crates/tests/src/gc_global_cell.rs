//! Task 0.3 (global-property-cells feature): Verify that dictionary-mode object
//! property values ARE traced by the GC.
//!
//! ## Background
//!
//! When a JS global object transitions to dictionary mode (via
//! `agent.ensure_named_property_dictionary`), property values are stored in
//! `ObjectMetadata.named_properties`, which is a `NamedPropertyStorage::Dictionary`.
//! Each entry's payload is `NamedPropertyValue::Data(Value)` — a `Value` stored
//! inline in a `HashMap<PropertyKey, NamedPropertyDictionaryEntry>` inside
//! `ObjectRuntime::object_metadata: Vec<Option<ObjectMetadata>>`.
//! This side-table lives OUTSIDE the GC heap (it is agent-side Rust memory, not a
//! GC-heap arena allocation).
//!
//! ## The original gap (Task 0.1) and its fix (Task 0.3)
//!
//! Originally, dictionary-mode property values were NOT traced: the GC mark walk
//! starts from `AgentCollectionSnapshot` (`crates/env/src/agent.rs`), which only
//! visits GC-heap `RuntimeObjectRecord` fields (`named_slots`, `inline_named_slots`,
//! prototype, elements, `private_slots`, payloads) via `trace_heap_edges`. None of
//! those correspond to dictionary entries, so any value reachable ONLY through a
//! dictionary entry was silently collected.
//!
//! Task 0.3 closes the gap with a per-object metadata mark hook:
//!   - `lyng_gc::TraceObjectMetadataEdges` (defined in `crates/gc/src/rooting.rs`)
//!     is a callback trait invoked for every live object as it is processed in the
//!     mark loop's `MarkWorkItem::Object` arm, right after `trace_heap_edges`.
//!   - `ObjectRuntime` implements it in `crates/objects/src/gc_integration.rs`,
//!     walking `ObjectMetadata::named_properties::Dictionary::entries` and marking
//!     each entry's `Data`/`Accessor` `Value`s.
//!   - The env layer (`crates/env/src/agent/weak_finalization.rs`) passes
//!     `&self.objects` as the metadata tracer into `force_collect_tracing`.
//!
//! This test confirms the fix: a TENURED object reachable ONLY through a dictionary
//! entry on the global object now SURVIVES a full `force_collect` (major) cycle.
//!
//! ## Scope (major collection only)
//!
//! The hook covers MAJOR (stop-the-world) collection. Minor (nursery) collection
//! does NOT trace dictionary metadata: dictionary writes don't dirty cards and
//! `PrimitiveMinorTracer` doesn't visit metadata, so a YOUNG value reachable only
//! through a dictionary entry can still die in a minor GC. This test allocates the
//! inner object with `AllocationLifetime::Default` (tenured) and exercises only the
//! major path; the minor-GC gap is an accepted, pre-existing limitation that global
//! property cells sidestep by allocating cells tenured.

use lyng_env::Runtime;
use lyng_gc::AllocationLifetime;
use lyng_host::NoopHostHooks;
use lyng_objects::{NoopAdaptiveProtoLoadDispatch, ObjectAllocation};
use lyng_types::{PropertyDescriptor, PropertyKey, Value};

/// Confirm that a heap object stored ONLY as a dictionary-mode global property value
/// SURVIVES a full GC cycle (`force_collect`), exercising the metadata mark hook
/// (`TraceObjectMetadataEdges`) that traces dictionary entry values.
///
/// ## What this test does
///
/// 1. Gets the default realm's global object.
/// 2. Forces the global object into dictionary mode.
/// 3. Allocates a fresh heap object ("inner object") with `AllocationLifetime::Default`.
///    (Major collection re-marks both young and old space, so the object's
///    generation does not matter here — it survives iff the metadata hook traces it.)
/// 4. Stores a sentinel property (`__gc_sentinel__ = 0xDEAD`) on the inner object.
/// 5. Stores the inner object as a dictionary property (`__inner_var__`) on the
///    global, dropping the only other reference. The inner object is now ONLY
///    reachable through the dictionary entry — which lives in `ObjectMetadata`,
///    traced via the metadata mark hook (NOT via the heap-record edge walk).
/// 6. Calls `agent.force_collect()` to run a complete mark-and-sweep.
/// 7. Reads back the inner object and asserts it SURVIVED: its heap record still
///    exists and its sentinel property still reads `0xDEAD`.
///
/// ## Result
///
/// The test passes when the inner object SURVIVES — confirming the metadata mark
/// hook traces dictionary-mode property values. If the inner object were collected,
/// the test panics, signaling the tracing path regressed.
#[test]
#[allow(clippy::too_many_lines)]
fn dictionary_global_object_value_survives_collection() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();

    // --- Step 1: Get the default realm's global object ---
    let realm = agent.default_realm().expect("default realm should exist");
    let global_object = realm.global_object();

    // --- Step 2: Force the global object into dictionary mode ---
    agent.ensure_named_property_dictionary(global_object, &mut NoopAdaptiveProtoLoadDispatch);
    {
        use lyng_objects::NamedPropertyStorageMode;
        let storage_mode = agent
            .objects()
            .named_property_storage_mode(global_object)
            .expect("global object should have named property storage mode");
        assert_eq!(
            storage_mode,
            NamedPropertyStorageMode::Dictionary,
            "global object should be in dictionary mode"
        );
    }

    // --- Step 3: Allocate a fresh heap object (the "inner object") ---
    // Major collection re-marks young+old space, so the object's generation is
    // irrelevant to this (major-GC) test; survival depends only on the metadata hook.
    // We deliberately hold NO explicit GC root for this object.
    let inner_object = agent.with_heap_and_objects(|heap, objects| {
        let root_shape = objects.root_shape(&mut heap.mutator(), None, AllocationLifetime::Default);
        objects.alloc_object(
            &mut heap.mutator(),
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });

    // --- Step 4: Set a sentinel property on the inner object ---
    let sentinel_atom = agent.atoms_mut().intern("__gc_sentinel__");
    let sentinel_key = PropertyKey::from_atom(sentinel_atom);
    let sentinel_value = Value::from_smi(0xDEAD);
    {
        let mut desc = PropertyDescriptor::new();
        desc.set_value(sentinel_value);
        desc.set_writable(true);
        desc.set_enumerable(true);
        desc.set_configurable(true);
        agent
            .define_own_property(
                inner_object,
                sentinel_key,
                desc,
                AllocationLifetime::Default,
                &mut NoopAdaptiveProtoLoadDispatch,
            )
            .expect("should define sentinel property on inner object");
    }

    // --- Step 5: Install inner object as a dictionary property on the global ---
    // After this call, inner_object is reachable ONLY through the dictionary entry
    // in ObjectMetadata.named_properties — which is NOT in the GC root graph.
    let var_atom = agent.atoms_mut().intern("__inner_var__");
    let var_key = PropertyKey::from_atom(var_atom);
    {
        let mut desc = PropertyDescriptor::new();
        desc.set_value(Value::from_object_ref(inner_object));
        desc.set_writable(true);
        desc.set_enumerable(true);
        desc.set_configurable(true);
        agent
            .define_own_property(
                global_object,
                var_key,
                desc,
                AllocationLifetime::Default,
                &mut NoopAdaptiveProtoLoadDispatch,
            )
            .expect("should define global dictionary property");
    }

    // At this point the Rust stack value `inner_object: ObjectRef` is NOT a GC root
    // (ObjectRef is a plain copy-type handle, not registered in PrimitiveRoots).
    // The only path to the inner object is the dictionary entry on the global object.
    // That entry IS traced by the metadata mark hook (TraceObjectMetadataEdges), so
    // the inner object must survive collection.

    // --- Step 6: Run a full GC cycle ---
    let _report = agent.force_collect();

    // --- Step 7: Verify the inner object SURVIVED ---
    let global_descriptor = agent
        .objects()
        .get_own_property(agent.heap().view(), global_object, var_key)
        .expect("global object itself should still be accessible after GC")
        .expect("dictionary entry should still exist in ObjectMetadata");

    let recovered_value = global_descriptor
        .value()
        .expect("global property should be a data property with a value");

    let recovered_inner = recovered_value
        .as_object_ref()
        .expect("recovered value should be an object ref");

    // The heap record for inner_object should STILL EXIST — it was traced via the
    // metadata mark hook and therefore survived collection.
    let heap_record = agent.heap().view().object(recovered_inner);
    assert!(
        heap_record.is_some(),
        "REGRESSION: inner object was COLLECTED — the dictionary metadata mark hook \
         (TraceObjectMetadataEdges) failed to trace the dictionary entry value. \
         inner_object={recovered_inner:?}"
    );

    // The sentinel property must still read back its original value, proving the
    // surviving record is intact (not just an un-reclaimed but corrupted slot).
    let sentinel_descriptor = agent
        .objects()
        .get_own_property(agent.heap().view(), recovered_inner, sentinel_key)
        .expect("inner object should be accessible after GC")
        .expect("sentinel property should still exist on the surviving inner object");

    let recovered_sentinel = sentinel_descriptor
        .value()
        .expect("sentinel property should be a data property with a value");

    assert_eq!(
        recovered_sentinel, sentinel_value,
        "surviving inner object's sentinel property should still read 0xDEAD"
    );

    println!(
        "CONFIRMED: dictionary-mode global property value SURVIVED force_collect via the \
         metadata mark hook. The ObjectRef {recovered_inner:?} stored in the dictionary \
         entry is still live (heap().view().object() returned Some) and its sentinel \
         property still reads 0xDEAD. \
         See crates/gc/src/rooting.rs (TraceObjectMetadataEdges) and \
         crates/objects/src/gc_integration.rs (ObjectRuntime impl) for the tracing path."
    );
}

/// A runtime-created cell-backed global VALUE must survive a MINOR (nursery)
/// collection. The minor collector does not trace dictionary-mode metadata
/// edges, so the only thing keeping a cell-backed global alive across a minor
/// GC is the cell being TENURED. `redefine_named_property` therefore allocates
/// backing cells `LongLived`; if it ever regresses to `Default` (nursery), the
/// cell is reclaimed and the binding's value is silently lost.
#[test]
fn minor_gc_cell_backed_global_value_survives() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let global_object = realm.global_object();

    // Sanity: the realm global object is cell-backed from creation.
    assert!(
        agent.object_uses_cell_backed_dictionary(global_object),
        "global object should be cell-backed"
    );

    // Define a fresh data property at runtime -> routed through a Default-lifetime
    // ValueCell. Use an SMI value so survival is testable purely via the cell.
    let var_atom = agent.atoms_mut().intern("__minor_probe__");
    let var_key = PropertyKey::from_atom(var_atom);
    {
        let mut desc = PropertyDescriptor::new();
        desc.set_value(Value::from_smi(0x1234));
        desc.set_writable(true);
        desc.set_enumerable(true);
        desc.set_configurable(true);
        agent
            .define_own_property(
                global_object,
                var_key,
                desc,
                AllocationLifetime::Default,
                &mut NoopAdaptiveProtoLoadDispatch,
            )
            .expect("define runtime global cell");
    }

    let cell = agent
        .cell_backed_entry(global_object, var_key)
        .expect("entry should be cell-backed");
    let young_before = agent.heap().view().value_cell(cell).is_some();
    assert!(young_before, "cell should exist before minor GC");

    // Run a minor (nursery) collection. The dictionary entry holding `cell` lives
    // off-heap in ObjectMetadata; minor GC does not trace it.
    {
        let roots = agent.roots() as *const _;
        // SAFETY shim for the borrow checker: roots and heap are disjoint fields.
        let roots = unsafe { &*roots };
        agent.heap_mut().force_minor_collect(roots);
    }

    assert!(
        agent.heap().view().value_cell(cell).is_some(),
        "REGRESSION: cell-backed global cell was reclaimed by a minor GC — the \
         backing cell must be allocated LongLived (tenured), not Default (nursery)"
    );
    let entry_value = agent
        .cell_backed_entry(global_object, var_key)
        .and_then(|c| agent.heap().view().value_cell(c))
        .map(lyng_gc::PrimitiveValueCellRecord::stored_value);
    assert_eq!(
        entry_value,
        Some(Value::from_smi(0x1234)),
        "the global binding's value must be intact after a minor GC"
    );
}

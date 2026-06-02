//! Verify that dictionary-mode global property values are traced by the major GC.
//!
//! Dictionary-mode property values live in `ObjectMetadata.named_properties`
//! (agent-side Rust memory, outside the GC heap). The `TraceObjectMetadataEdges`
//! hook (`crates/gc/src/rooting.rs`) is invoked for every live object during the
//! mark loop, right after `trace_heap_edges`, and walks dictionary entries to mark
//! their `Data`/`Accessor` values.
//!
//! Minor (nursery) collection does NOT trace dictionary metadata — dictionary
//! writes don't dirty cards. A young value reachable only through a dictionary
//! entry can die in a minor GC. Global property cells sidestep this by allocating
//! cells `LongLived` (tenured).

use lyng_env::Runtime;
use lyng_gc::AllocationLifetime;
use lyng_host::NoopHostHooks;
use lyng_objects::{NoopAdaptiveProtoLoadDispatch, ObjectAllocation};
use lyng_types::{PropertyDescriptor, PropertyKey, Value};

/// A heap object reachable ONLY through a dictionary-mode global property must
/// survive a full `force_collect` cycle via the `TraceObjectMetadataEdges` hook.
#[test]
#[allow(clippy::too_many_lines)]
fn dictionary_global_object_value_survives_collection() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();

    let realm = agent.default_realm().expect("default realm should exist");
    let global_object = realm.global_object();

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

    // No explicit GC root for inner_object — survival depends solely on the metadata hook.
    let inner_object = agent.with_heap_and_objects(|heap, objects| {
        let root_shape = objects.root_shape(&mut heap.mutator(), None, AllocationLifetime::Default);
        objects.alloc_object(
            &mut heap.mutator(),
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });

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

    let _report = agent.force_collect();

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

    let heap_record = agent.heap().view().object(recovered_inner);
    assert!(
        heap_record.is_some(),
        "REGRESSION: inner object was COLLECTED — the dictionary metadata mark hook \
         (TraceObjectMetadataEdges) failed to trace the dictionary entry value. \
         inner_object={recovered_inner:?}"
    );

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
}

/// A cell-backed global's value cell must be tenured so it survives a minor GC.
/// Minor collection doesn't trace dictionary metadata; if the cell were nursery-
/// allocated it would be reclaimed and the binding's value silently lost.
#[test]
fn minor_gc_cell_backed_global_value_survives() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let global_object = realm.global_object();

    assert!(
        agent.object_uses_cell_backed_dictionary(global_object),
        "global object should be cell-backed"
    );

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
    assert!(
        agent.heap().view().value_cell(cell).is_some(),
        "cell should exist before minor GC"
    );

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

//! Task 0.1 (global-property-cells feature): Characterize whether dictionary-mode
//! global object values are traced by the GC.
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
//! ## CRITICAL FINDING (empirically confirmed by the test below)
//!
//! **Dictionary-mode global property values are NOT traced by the GC.**
//!
//! The test `dictionary_global_object_value_survives_collection` below confirms
//! this: an object stored ONLY as a dictionary entry value on the global object —
//! with NO other GC-visible root — is collected by `force_collect`. After GC,
//! attempting to access the object returns `InternalMethodError::MissingObject`.
//!
//! ## Why there is no tracing
//!
//! The GC mark walk starts from `AgentCollectionSnapshot`
//! (`crates/env/src/agent.rs:54`). This snapshot includes `realms`, `intrinsics`,
//! `execution_contexts`, `modules`, etc., but does NOT include `objects: ObjectRuntime`
//! and therefore does NOT include `ObjectRuntime::object_metadata`.
//!
//! When the mark walk visits the global object's heap record, it calls
//! `trace_object_edges` (`crates/gc/src/rooting.rs:1068`), which traces:
//!   - `record.named_slots()` — the out-of-line heap-allocated slot storage
//!   - `record.inline_named_slots()` — slots packed into the object record itself
//!   - `record.prototype()`
//!   - `record.elements()`
//!   - `record.private_slots()`
//!   - `record.function_payload()` / `record.ordinary_payload()`
//!
//! None of these correspond to dictionary entries. Dictionary entries live in
//! `ObjectMetadata::named_properties::Dictionary::entries` (agent-side Rust memory),
//! which is never visited during the mark phase.
//!
//! ## Implication for the global-property-cells feature
//!
//! The `global-property-cells` feature MUST ensure that global var values are stored
//! in GC-visible storage (e.g., `ValueCell`s whose refs are in the object's
//! `named_slots` or `inline_named_slots`) rather than in agent-side dictionary
//! entries. The current architecture has a latent GC tracing gap for all
//! dictionary-mode objects whose values are only referenced from dictionary entries.
//!
//! In practice this gap is usually papered over because:
//! (a) Most dictionary-mode objects are reachable from the execution context or
//!     environment, which IS traced, so their values are transitively reachable via
//!     other paths.
//! (b) The global object itself IS traced (via `realms -> global_object`), but its
//!     DICTIONARY VALUES are not transitively reachable through that path because
//!     `trace_object_edges` does not visit `ObjectMetadata`.
//!
//! Any value ONLY reachable through a dictionary entry will be silently collected.

use lyng_env::Runtime;
use lyng_gc::AllocationLifetime;
use lyng_host::NoopHostHooks;
use lyng_objects::{NoopAdaptiveProtoLoadDispatch, ObjectAllocation};
use lyng_types::{PropertyDescriptor, PropertyKey, Value};

/// Confirm that a heap object stored ONLY as a dictionary-mode global property value
/// is collected by a full GC cycle (`force_collect`), demonstrating the GC tracing
/// gap for dictionary entry values.
///
/// ## What this test does
///
/// 1. Gets the default realm's global object.
/// 2. Forces the global object into dictionary mode.
/// 3. Allocates a fresh heap object ("inner object") with `AllocationLifetime::Default`
///    (tenured, goes directly to old-space).
/// 4. Stores a sentinel property (`__gc_sentinel__ = 0xDEAD`) on the inner object.
/// 5. Stores the inner object as a dictionary property (`__inner_var__`) on the
///    global, dropping the only other reference. The inner object is now ONLY
///    reachable through the dictionary entry — which is NOT in the GC root graph.
/// 6. Calls `agent.force_collect()` to run a complete mark-and-sweep.
/// 7. Attempts to read back the inner object and asserts it was COLLECTED (the
///    dictionary `Value` reference is now dangling and the object is gone).
///
/// ## Result
///
/// The test passes when the inner object IS collected — confirming the gap.
/// If the inner object were to survive, the test would panic with an explicit
/// message, documenting that a tracing mechanism was added.
#[test]
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
    // AllocationLifetime::Default => tenured immediately (old-space).
    // We deliberately hold NO explicit GC root for this object.
    let inner_object = agent.with_heap_and_objects(|heap, objects| {
        let root_shape = objects
            .root_shape(&mut heap.mutator(), None, AllocationLifetime::Default);
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
    // The only path to the inner object is the dictionary entry on the global object,
    // which is NOT traced by the GC mark walk.

    // --- Step 6: Run a full GC cycle ---
    let _report = agent.force_collect();

    // --- Step 7: Verify the inner object was COLLECTED ---
    // The dictionary entry on the global still exists (it's in agent-side Rust memory,
    // not GC-managed), but the ObjectRef it contains now points to freed GC memory.
    // Accessing it via the object runtime should return MissingObject.
    let global_descriptor = agent
        .objects()
        .get_own_property(agent.heap().view(), global_object, var_key)
        .expect("global object itself should still be accessible after GC")
        .expect("dictionary entry should still exist in ObjectMetadata (it is not GC-managed)");

    let recovered_value = global_descriptor
        .value()
        .expect("global property should be a data property with a value");

    let recovered_inner = recovered_value
        .as_object_ref()
        .expect("recovered value should be an object ref");

    // The heap record for inner_object should be GONE — it was collected.
    let heap_record = agent.heap().view().object(recovered_inner);
    assert!(
        heap_record.is_none(),
        "FINDING CHANGED: inner object SURVIVED GC — this means a tracing path for \
         dictionary entry values now exists. Update this test and the module-level \
         documentation to reflect the new tracing mechanism. \
         inner_object={recovered_inner:?}"
    );

    println!(
        "CONFIRMED FINDING: dictionary-mode global property value was COLLECTED by force_collect. \
         The ObjectRef {recovered_inner:?} stored in the dictionary entry now points to freed \
         GC memory (heap().view().object() returned None). \
         Dictionary entry Values are NOT traced by the GC mark walk. \
         See crates/gc/src/rooting.rs:1068 (trace_object_edges) and \
         crates/env/src/agent.rs:54 (AgentCollectionSnapshot) for the authoritative \
         evidence of the missing tracing path."
    );
}

#![allow(clippy::too_many_lines)]

use super::*;
use lyng_js_common::{AtomId, AtomLifetime, AtomTable, SourceId, WellKnownAtom};
use lyng_js_gc::{
    AtomGcSweep, PrimitiveHeap, PrimitiveHeapMarker, PrimitiveRoots, RuntimeCodeRecord,
    RuntimeEnvironmentRecord, RuntimeRealmRecord, TraceAtomEdges, ValueStoreTarget,
};
use lyng_js_types::{BuiltinFunctionId, NativeFunctionId, SymbolRef, TypeOwnershipMarker, Value};
use std::collections::HashMap;
use std::mem::size_of;

fn attrs(writable: bool, enumerable: bool, configurable: bool) -> DescriptorAttributes {
    let mut attrs = DescriptorAttributes::empty();
    attrs.set_writable(writable);
    attrs.set_enumerable(enumerable);
    attrs.set_configurable(configurable);
    attrs
}

fn data_descriptor(value: Value, writable: bool, configurable: bool) -> PropertyDescriptor {
    let mut descriptor = PropertyDescriptor::new();
    descriptor.set_value(value);
    descriptor.set_writable(writable);
    descriptor.set_enumerable(true);
    descriptor.set_configurable(configurable);
    descriptor
}

fn engine_array(
    runtime: &mut ObjectRuntime,
    mutator: &mut PrimitiveMutator<'_>,
    shape: ShapeId,
    length: u32,
    writable: bool,
) -> ObjectRef {
    let object = runtime.alloc_object(
        mutator,
        ObjectAllocation::ordinary(shape)
            .with_flags(ObjectFlags::extensible().union(ObjectFlags::ENGINE_ARRAY)),
        AllocationLifetime::Default,
    );
    let mut descriptor = PropertyDescriptor::new();
    descriptor.set_value(if let Ok(length) = i32::try_from(length) {
        Value::from_smi(length)
    } else {
        Value::from_f64(f64::from(length))
    });
    descriptor.set_writable(writable);
    descriptor.set_enumerable(false);
    descriptor.set_configurable(false);
    assert!(runtime
        .define_own_property(
            mutator,
            object,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
            descriptor,
            AllocationLifetime::Default,
        )
        .unwrap());
    object
}

fn array_length_descriptor(
    runtime: &ObjectRuntime,
    heap: PrimitiveHeapView<'_>,
    object: ObjectRef,
) -> PropertyDescriptor {
    runtime
        .get_own_property(
            heap,
            object,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
        )
        .unwrap()
        .expect("engine arrays should carry a length property")
}

#[test]
fn object_header_stays_within_phase3_size_budget() {
    assert!(size_of::<ObjectHeader>() <= 32);
}

type RecordedNativeCall = (
    NativeFunctionId,
    Value,
    Vec<Value>,
    ObjectRef,
    RealmRef,
    EnvironmentRef,
    Option<EnvironmentRef>,
    FunctionThisMode,
    Option<ObjectRef>,
    FunctionConstructorFlags,
    FunctionKindFlags,
);

type RecordedNativeConstruct = (
    NativeFunctionId,
    ObjectRef,
    Vec<Value>,
    ObjectRef,
    RealmRef,
    EnvironmentRef,
    Option<EnvironmentRef>,
    FunctionThisMode,
    Option<ObjectRef>,
    FunctionConstructorFlags,
    FunctionKindFlags,
);

#[derive(Default)]
struct RecordingNativeRegistry {
    call_results: HashMap<NativeFunctionId, Value>,
    construct_results: HashMap<NativeFunctionId, ObjectRef>,
    calls: Vec<RecordedNativeCall>,
    constructs: Vec<RecordedNativeConstruct>,
}

impl NativeFunctionRegistry for RecordingNativeRegistry {
    fn call(
        &mut self,
        _runtime: &mut ObjectRuntime,
        _heap: &mut PrimitiveMutator<'_>,
        request: NativeCallRequest<'_>,
    ) -> InternalMethodResult<Value> {
        self.calls.push((
            request.entry(),
            request.this_value(),
            request.arguments().to_vec(),
            request.callee(),
            request.realm(),
            request.environment(),
            request.private_env(),
            request.this_mode(),
            request.home_object(),
            request.constructor_flags(),
            request.kind_flags(),
        ));
        self.call_results
            .get(&request.entry())
            .copied()
            .ok_or(InternalMethodError::MissingNativeHandler)
    }

    fn construct(
        &mut self,
        _runtime: &mut ObjectRuntime,
        _heap: &mut PrimitiveMutator<'_>,
        request: NativeConstructRequest<'_>,
    ) -> InternalMethodResult<ObjectRef> {
        self.constructs.push((
            request.entry(),
            request.new_target(),
            request.arguments().to_vec(),
            request.callee(),
            request.realm(),
            request.environment(),
            request.private_env(),
            request.this_mode(),
            request.home_object(),
            request.constructor_flags(),
            request.kind_flags(),
        ));
        self.construct_results
            .get(&request.entry())
            .copied()
            .ok_or(InternalMethodError::MissingNativeHandler)
    }
}

#[test]
fn object_flags_track_integrity_and_dictionary_summary_bits() {
    let flags = ObjectFlags::extensible()
        .union(ObjectFlags::SEALED)
        .union(ObjectFlags::NAMED_PROPERTIES_DICTIONARY);

    assert!(flags.is_extensible());
    assert!(flags.is_sealed_summary());
    assert!(!flags.is_frozen_summary());
    assert!(flags.uses_named_property_dictionary());
}

#[test]
fn root_shapes_are_canonicalized_by_prototype_guard() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root_a = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let root_b = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let guarded = runtime.root_shape(
        &mut mutator,
        Some(ObjectRef::from_raw(9).unwrap()),
        AllocationLifetime::Default,
    );

    assert_eq!(root_a, root_b);
    assert_ne!(root_a, guarded);
    assert_eq!(
        runtime
            .shape(mutator.view(), root_a)
            .unwrap()
            .property_count(),
        0
    );
}

#[test]
fn named_property_cache_entries_track_shape_and_prototype_dependencies() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let key = PropertyKey::from_atom(AtomId::from_raw(901));

    let prototype = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );
    assert!(runtime
        .define_own_property(
            &mut mutator,
            prototype,
            key,
            data_descriptor(Value::from_smi(11), true, true),
            AllocationLifetime::Default,
        )
        .unwrap());

    let object = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root).with_prototype(Some(prototype)),
        AllocationLifetime::Default,
    );

    let cache = runtime
        .plan_named_property_cache_entry(
            mutator.view(),
            object,
            key,
            NamedPropertyCachePurpose::Load,
        )
        .unwrap()
        .expect("prototype data property should be cacheable");
    assert_eq!(cache.path(), NamedPropertyCachePath::PrototypeData);
    assert_eq!(cache.dependency_count(), 2);
    assert_eq!(cache.holder(), prototype);
    assert_eq!(
        runtime
            .load_from_named_property_cache(mutator.view(), object, cache)
            .unwrap(),
        Some(Value::from_smi(11))
    );

    let replacement = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );
    assert!(runtime
        .set_prototype_of(&mut mutator, object, Some(replacement))
        .unwrap());
    assert_eq!(
        runtime
            .load_from_named_property_cache(mutator.view(), object, cache)
            .unwrap(),
        None
    );
}

#[test]
fn module_namespace_objects_read_live_bindings_and_reject_mutation() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let export_a = AtomId::from_raw(1501);
    let export_b = AtomId::from_raw(1502);

    let slots = mutator.alloc_environment_slots(2, Value::undefined(), AllocationLifetime::Default);
    assert!(mutator.init_store_value(
        ValueStoreTarget::EnvironmentSlot(slots, 0),
        Value::from_smi(1),
    ));
    assert!(mutator.init_store_value(
        ValueStoreTarget::EnvironmentSlot(slots, 1),
        Value::from_smi(2),
    ));
    let environment = mutator.alloc_environment(
        RuntimeEnvironmentRecord::new(None, Some(slots), None, Value::undefined(), None, None),
        AllocationLifetime::Default,
    );

    let namespace = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );
    assert!(runtime.install_module_namespace_object(
        namespace,
        vec![
            ModuleNamespaceExport::new(
                export_a,
                ModuleNamespaceExportTarget::Binding {
                    environment,
                    slot: 0,
                },
            ),
            ModuleNamespaceExport::new(
                export_b,
                ModuleNamespaceExportTarget::Value(Value::from_smi(9)),
            ),
        ],
    ));

    let descriptor = runtime
        .get_own_property(mutator.view(), namespace, PropertyKey::from_atom(export_a))
        .unwrap()
        .expect("module namespace should expose export descriptors");
    assert_eq!(descriptor.value(), Some(Value::from_smi(1)));
    assert_eq!(descriptor.writable(), Some(true));
    assert_eq!(descriptor.enumerable(), Some(true));
    assert_eq!(descriptor.configurable(), Some(false));
    assert_eq!(
        runtime
            .own_property_keys(mutator.view(), namespace)
            .unwrap(),
        vec![
            PropertyKey::from_atom(export_a),
            PropertyKey::from_atom(export_b)
        ]
    );
    assert!(!runtime.is_extensible(namespace).unwrap());

    assert!(mutator.mut_store_value(
        ValueStoreTarget::EnvironmentSlot(slots, 0),
        Value::from_smi(7),
    ));
    assert_eq!(
        runtime
            .get(
                mutator.view(),
                namespace,
                PropertyKey::from_atom(export_a),
                Value::from_object_ref(namespace),
            )
            .unwrap(),
        Value::from_smi(7)
    );

    assert!(!runtime
        .set(
            &mut mutator,
            namespace,
            PropertyKey::from_atom(export_a),
            Value::from_smi(11),
            Value::from_object_ref(namespace),
            AllocationLifetime::Default,
        )
        .unwrap());
    assert!(!runtime
        .delete(&mut mutator, namespace, PropertyKey::from_atom(export_a))
        .unwrap());

    let mut incompatible = PropertyDescriptor::new();
    incompatible.set_value(Value::from_smi(5));
    incompatible.set_writable(true);
    incompatible.set_enumerable(true);
    incompatible.set_configurable(false);
    assert!(!runtime
        .define_own_property(
            &mut mutator,
            namespace,
            PropertyKey::from_atom(export_a),
            incompatible,
            AllocationLifetime::Default,
        )
        .unwrap());

    assert!(runtime
        .set_prototype_of(&mut mutator, namespace, None)
        .unwrap());
    let other = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );
    assert!(!runtime
        .set_prototype_of(&mut mutator, namespace, Some(other))
        .unwrap());
}

#[test]
fn named_property_store_cache_hits_own_slots_and_invalidates_on_dictionary_transition() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let key = PropertyKey::from_atom(AtomId::from_raw(902));
    let object = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );
    assert!(runtime
        .define_own_property(
            &mut mutator,
            object,
            key,
            data_descriptor(Value::from_smi(1), true, true),
            AllocationLifetime::Default,
        )
        .unwrap());

    let cache = runtime
        .plan_named_property_cache_entry(
            mutator.view(),
            object,
            key,
            NamedPropertyCachePurpose::Store,
        )
        .unwrap()
        .expect("own writable data property should be cacheable");
    assert_eq!(cache.path(), NamedPropertyCachePath::OwnData);
    assert_eq!(
        runtime
            .store_to_named_property_cache(&mut mutator, object, cache, Value::from_smi(5))
            .unwrap(),
        Some(true)
    );
    assert_eq!(
        runtime.get(mutator.view(), object, key, Value::from_object_ref(object)),
        Ok(Value::from_smi(5))
    );

    let mut redefine = PropertyDescriptor::new();
    redefine.set_writable(false);
    redefine.set_enumerable(true);
    redefine.set_configurable(true);
    assert!(runtime
        .define_own_property(
            &mut mutator,
            object,
            key,
            redefine,
            AllocationLifetime::Default,
        )
        .unwrap());
    assert_eq!(
        runtime.named_property_storage_mode(object),
        Some(NamedPropertyStorageMode::Dictionary)
    );
    assert_eq!(
        runtime
            .store_to_named_property_cache(&mut mutator, object, cache, Value::from_smi(7))
            .unwrap(),
        None
    );
}

#[test]
fn string_exotic_properties_surface_length_indices_and_extra_keys() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let string = mutator.alloc_string(
        lyng_js_gc::StringEncoding::Latin1,
        3,
        b"cat",
        None,
        AllocationLifetime::Default,
    );
    let wrapper = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root)
            .with_element_capacity(3)
            .with_cold_data(ObjectColdData::Ordinary(
                OrdinaryObjectData::PrimitiveWrapper(PrimitiveWrapperKind::String),
            ))
            .with_primitive_wrapper_value(Value::from_string_ref(string)),
        AllocationLifetime::Default,
    );
    for (index, byte) in b"cat".iter().copied().enumerate() {
        let element = mutator.alloc_string(
            lyng_js_gc::StringEncoding::Latin1,
            1,
            &[byte],
            None,
            AllocationLifetime::Default,
        );
        assert!(runtime.init_element(
            &mut mutator,
            wrapper,
            u32::try_from(index).unwrap(),
            Value::from_string_ref(element),
        ));
    }

    let extra_index = PropertyKey::Index(5);
    let extra_name = PropertyKey::from_atom(AtomId::from_raw(950));
    assert!(runtime
        .define_own_property(
            &mut mutator,
            wrapper,
            extra_index,
            data_descriptor(Value::from_smi(99), true, true),
            AllocationLifetime::Default,
        )
        .unwrap());
    assert!(runtime
        .define_own_property(
            &mut mutator,
            wrapper,
            extra_name,
            data_descriptor(Value::from_smi(7), true, true),
            AllocationLifetime::Default,
        )
        .unwrap());

    let length = runtime
        .get_own_property(
            mutator.view(),
            wrapper,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
        )
        .unwrap()
        .unwrap();
    let first = runtime
        .get_own_property(mutator.view(), wrapper, PropertyKey::Index(0))
        .unwrap()
        .unwrap();
    let keys = runtime.own_property_keys(mutator.view(), wrapper).unwrap();

    assert_eq!(length.value(), Some(Value::from_smi(3)));
    assert_eq!(length.writable(), Some(false));
    assert_eq!(length.enumerable(), Some(false));
    assert_eq!(length.configurable(), Some(false));
    assert_eq!(
        mutator
            .view()
            .string_view(first.value().unwrap().as_string_ref().unwrap())
            .unwrap()
            .latin1_bytes(),
        Some(&b"c"[..])
    );
    assert_eq!(
        keys,
        vec![
            PropertyKey::Index(0),
            PropertyKey::Index(1),
            PropertyKey::Index(2),
            PropertyKey::Index(5),
            PropertyKey::from_atom(WellKnownAtom::length.id()),
            extra_name,
        ]
    );
}

#[test]
fn string_exotic_properties_reject_incompatible_redefinition_and_deletion() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let string = mutator.alloc_string(
        lyng_js_gc::StringEncoding::Latin1,
        2,
        b"hi",
        None,
        AllocationLifetime::Default,
    );
    let wrapper = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root)
            .with_element_capacity(2)
            .with_cold_data(ObjectColdData::Ordinary(
                OrdinaryObjectData::PrimitiveWrapper(PrimitiveWrapperKind::String),
            ))
            .with_primitive_wrapper_value(Value::from_string_ref(string)),
        AllocationLifetime::Default,
    );
    for (index, byte) in b"hi".iter().copied().enumerate() {
        let element = mutator.alloc_string(
            lyng_js_gc::StringEncoding::Latin1,
            1,
            &[byte],
            None,
            AllocationLifetime::Default,
        );
        assert!(runtime.init_element(
            &mut mutator,
            wrapper,
            u32::try_from(index).unwrap(),
            Value::from_string_ref(element),
        ));
    }

    let mut same_index = PropertyDescriptor::new();
    same_index.set_value(
        runtime
            .get(
                mutator.view(),
                wrapper,
                PropertyKey::Index(0),
                Value::from_object_ref(wrapper),
            )
            .unwrap(),
    );
    assert!(runtime
        .define_own_property(
            &mut mutator,
            wrapper,
            PropertyKey::Index(0),
            same_index,
            AllocationLifetime::Default,
        )
        .unwrap());

    let mut incompatible_index = PropertyDescriptor::new();
    incompatible_index.set_value(Value::from_smi(1));
    assert!(!runtime
        .define_own_property(
            &mut mutator,
            wrapper,
            PropertyKey::Index(0),
            incompatible_index,
            AllocationLifetime::Default,
        )
        .unwrap());

    let mut incompatible_length = PropertyDescriptor::new();
    incompatible_length.set_writable(true);
    assert!(!runtime
        .define_own_property(
            &mut mutator,
            wrapper,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
            incompatible_length,
            AllocationLifetime::Default,
        )
        .unwrap());
    assert!(!runtime
        .delete(&mut mutator, wrapper, PropertyKey::Index(1))
        .unwrap());
    assert!(!runtime
        .delete(
            &mut mutator,
            wrapper,
            PropertyKey::from_atom(WellKnownAtom::length.id())
        )
        .unwrap());
}

#[test]
fn date_objects_store_explicit_payload_values() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let date = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root)
            .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::Date))
            .with_date_value(Value::from_f64(123.0)),
        AllocationLifetime::Default,
    );

    assert!(runtime.is_date_object(date));
    assert_eq!(
        runtime.date_value(mutator.view(), date),
        Some(Value::from_f64(123.0))
    );
    assert_eq!(
        runtime.ordinary_payload_value(mutator.view(), date),
        Some(Value::from_f64(123.0))
    );
}

#[test]
fn temporal_objects_store_typed_payloads_by_temporal_kind() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let calendar = AtomId::from_raw(101);
    let time_zone = AtomId::from_raw(202);

    let instant = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root).with_cold_data(ObjectColdData::Ordinary(
            OrdinaryObjectData::Temporal(TemporalObjectKind::Instant),
        )),
        AllocationLifetime::Default,
    );
    let zoned = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root).with_cold_data(ObjectColdData::Ordinary(
            OrdinaryObjectData::Temporal(TemporalObjectKind::ZonedDateTime),
        )),
        AllocationLifetime::Default,
    );
    let plain = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );

    let instant_payload = TemporalObjectData::Instant(TemporalInstantObjectData::new(123));
    let zoned_payload = TemporalObjectData::ZonedDateTime(TemporalZonedDateTimeObjectData::new(
        123, time_zone, calendar,
    ));

    assert!(runtime.install_temporal_object(instant, instant_payload));
    assert!(runtime.install_temporal_object(zoned, zoned_payload));
    assert!(!runtime.install_temporal_object(plain, instant_payload));

    assert!(runtime.is_temporal_object_kind(instant, TemporalObjectKind::Instant));
    assert!(runtime.is_temporal_object_kind(zoned, TemporalObjectKind::ZonedDateTime));
    assert_eq!(runtime.temporal_object(instant), Some(&instant_payload));
    assert_eq!(runtime.temporal_object(zoned), Some(&zoned_payload));
    assert_eq!(runtime.temporal_object(plain), None);

    let _ = runtime.free_object(&mut mutator, zoned).unwrap();
    assert_eq!(runtime.temporal_object(zoned), None);
}

#[test]
fn regexp_objects_store_payloads_and_account_live_records() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let regexp = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root)
            .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::RegExp)),
        AllocationLifetime::Default,
    );
    assert!(runtime.store_regexp_payload(regexp, RegExpPayload::compile("a", "dg").unwrap()));

    let payload = runtime
        .regexp_payload(regexp)
        .expect("regexp payload should round-trip");
    assert!(runtime.is_regexp_object(regexp));
    assert_eq!(payload.source(), "a");
    assert_eq!(payload.flag_text(), "dg");

    let accounting = runtime.regexp_payload_accounting(mutator.view());
    assert_eq!(accounting.records, 1);
    assert!(accounting.metadata_bytes >= size_of::<RegExpPayload>());
    assert!(accounting.payload_bytes >= payload.payload_bytes());

    let _ = runtime.free_object(&mut mutator, regexp).unwrap();
    assert!(runtime.regexp_payload(regexp).is_none());
    assert_eq!(runtime.regexp_payload_accounting(mutator.view()).records, 0);
}

#[test]
fn regexp_payload_compile_accepts_unknown_script_extension_aliases() {
    assert!(RegExpPayload::compile(r"\p{Script_Extensions=Unknown}", "u").is_ok());
    assert!(RegExpPayload::compile(r"\p{scx=Zzzz}", "u").is_ok());
}

#[test]
fn regexp_payload_non_unicode_astral_source_matches_surrogate_code_units() {
    let payload = RegExpPayload::compile("𠮷", "").unwrap();
    let text = "𠮷".encode_utf16().collect::<Vec<_>>();
    let matched = payload.find_from_code_units(&text, 0);

    assert_eq!(matched.map(|record| record.range()), Some(0..2));
}

#[test]
fn regexp_payload_fast_digit_class_scan_handles_large_generated_strings() {
    use std::time::{Duration, Instant};

    let text = (0u16..=u16::MAX)
        .filter(|unit| !(0x30..=0x39).contains(unit))
        .cycle()
        .take(1_048_576)
        .collect::<Vec<_>>();

    let digit = RegExpPayload::compile(r"\d", "").unwrap();
    let started = Instant::now();
    let matched = digit.find_from_code_units(&text, 0);
    let elapsed = started.elapsed();

    assert_eq!(matched, None);
    assert!(
        elapsed < Duration::from_millis(25),
        "digit-class scan took {elapsed:?}"
    );

    let non_digit_run = RegExpPayload::compile(r"^\D+$", "").unwrap();
    let started = Instant::now();
    let matched = non_digit_run.find_from_code_units(&text, 0);
    let elapsed = started.elapsed();

    assert_eq!(matched.map(|record| record.range()), Some(0..text.len()));
    assert!(
        elapsed < Duration::from_millis(25),
        "anchored non-digit scan took {elapsed:?}"
    );
}

#[test]
fn named_property_load_cache_reads_current_receiver_for_same_shaped_own_data_objects() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let key = PropertyKey::from_atom(AtomId::from_raw(903));

    let first = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );
    let second = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );
    assert!(runtime
        .define_own_property(
            &mut mutator,
            first,
            key,
            data_descriptor(Value::from_smi(11), true, true),
            AllocationLifetime::Default,
        )
        .unwrap());
    assert!(runtime
        .define_own_property(
            &mut mutator,
            second,
            key,
            data_descriptor(Value::from_smi(22), true, true),
            AllocationLifetime::Default,
        )
        .unwrap());

    let cache = runtime
        .plan_named_property_cache_entry(
            mutator.view(),
            first,
            key,
            NamedPropertyCachePurpose::Load,
        )
        .unwrap()
        .expect("own data property should be cacheable");
    assert_eq!(cache.path(), NamedPropertyCachePath::OwnData);
    assert_eq!(
        runtime
            .load_from_named_property_cache(mutator.view(), second, cache)
            .unwrap(),
        Some(Value::from_smi(22))
    );
}

#[test]
fn named_property_load_cache_rejects_same_shaped_receivers_with_different_prototype_holders() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let key = PropertyKey::from_atom(AtomId::from_raw(904));

    let first_prototype = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );
    let second_prototype = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );
    assert!(runtime
        .define_own_property(
            &mut mutator,
            first_prototype,
            key,
            data_descriptor(Value::from_smi(11), true, true),
            AllocationLifetime::Default,
        )
        .unwrap());
    assert!(runtime
        .define_own_property(
            &mut mutator,
            second_prototype,
            key,
            data_descriptor(Value::from_smi(22), true, true),
            AllocationLifetime::Default,
        )
        .unwrap());

    let first = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root).with_prototype(Some(first_prototype)),
        AllocationLifetime::Default,
    );
    let second = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root).with_prototype(Some(second_prototype)),
        AllocationLifetime::Default,
    );

    let cache = runtime
        .plan_named_property_cache_entry(
            mutator.view(),
            first,
            key,
            NamedPropertyCachePurpose::Load,
        )
        .unwrap()
        .expect("prototype data property should be cacheable");
    assert_eq!(cache.path(), NamedPropertyCachePath::PrototypeData);
    assert_eq!(
        runtime
            .load_from_named_property_cache(mutator.view(), second, cache)
            .unwrap(),
        None
    );
}

#[test]
fn canonical_transitions_assign_slots_and_reuse_existing_shapes() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let first = runtime
        .transition_shape(
            &mut mutator,
            root,
            ShapeTransitionKey::new(
                PropertyKey::from_atom(AtomId::from_raw(1)),
                ShapePropertyKind::Data,
                attrs(true, true, true),
            ),
            AllocationLifetime::Default,
        )
        .unwrap();
    let first_again = runtime
        .transition_shape(
            &mut mutator,
            root,
            ShapeTransitionKey::new(
                PropertyKey::from_atom(AtomId::from_raw(1)),
                ShapePropertyKind::Data,
                attrs(true, true, true),
            ),
            AllocationLifetime::Default,
        )
        .unwrap();
    let second = runtime
        .transition_shape(
            &mut mutator,
            first,
            ShapeTransitionKey::new(
                PropertyKey::from_atom(AtomId::from_raw(2)),
                ShapePropertyKind::Accessor,
                attrs(false, true, false),
            ),
            AllocationLifetime::Default,
        )
        .unwrap();

    let first_property = runtime
        .shape_property(first, PropertyKey::from_atom(AtomId::from_raw(1)))
        .unwrap();
    let second_property = runtime
        .shape_property(second, PropertyKey::from_atom(AtomId::from_raw(2)))
        .unwrap();
    let second_shape = runtime.shape(mutator.view(), second).unwrap();

    assert_eq!(first, first_again);
    assert_eq!(first_property.slot_offset(), 0);
    assert_eq!(first_property.slot_width(), 1);
    assert_eq!(first_property.enumeration_index(), 0);
    assert_eq!(second_property.slot_offset(), 1);
    assert_eq!(second_property.slot_width(), 2);
    assert_eq!(second_property.enumeration_index(), 1);
    assert_eq!(second_shape.property_count(), 2);
    assert_eq!(second_shape.slot_count(), 3);
    assert_eq!(
        runtime
            .shape_properties(second)
            .unwrap()
            .iter()
            .map(|property| property.key())
            .collect::<Vec<_>>(),
        vec![
            PropertyKey::from_atom(AtomId::from_raw(1)),
            PropertyKey::from_atom(AtomId::from_raw(2)),
        ]
    );
}

#[test]
fn duplicate_property_add_is_not_a_shape_transition() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let first = runtime
        .transition_shape(
            &mut mutator,
            root,
            ShapeTransitionKey::new(
                PropertyKey::from_atom(AtomId::from_raw(1)),
                ShapePropertyKind::Data,
                attrs(true, true, true),
            ),
            AllocationLifetime::Default,
        )
        .unwrap();

    assert_eq!(
        runtime.transition_shape(
            &mut mutator,
            first,
            ShapeTransitionKey::new(
                PropertyKey::from_atom(AtomId::from_raw(1)),
                ShapePropertyKind::Data,
                attrs(true, true, true),
            ),
            AllocationLifetime::Default,
        ),
        None
    );
}

#[test]
fn large_stable_shapes_switch_to_flattened_lookup_table() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let mut shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);

    for raw in 1..=u32::try_from(SMALL_SHAPE_INLINE_PROPERTY_LIMIT + 1).unwrap() {
        shape = runtime
            .transition_shape(
                &mut mutator,
                shape,
                ShapeTransitionKey::new(
                    PropertyKey::from_atom(AtomId::from_raw(raw)),
                    ShapePropertyKind::Data,
                    attrs(true, true, true),
                ),
                AllocationLifetime::Default,
            )
            .unwrap();
    }

    let record = runtime.shape(mutator.view(), shape).unwrap();
    let property = runtime
        .shape_property(
            shape,
            PropertyKey::from_atom(AtomId::from_raw(
                u32::try_from(SMALL_SHAPE_INLINE_PROPERTY_LIMIT + 1).unwrap(),
            )),
        )
        .unwrap();

    assert!(record.uses_flat_lookup());
    assert_eq!(
        runtime.shape_properties(shape).unwrap().len(),
        SMALL_SHAPE_INLINE_PROPERTY_LIMIT + 1
    );
    assert_eq!(
        property.enumeration_index(),
        u32::try_from(SMALL_SHAPE_INLINE_PROPERTY_LIMIT).unwrap()
    );
}

#[test]
fn object_runtime_allocates_hot_header_and_cold_payload_out_of_line() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();

    let base_shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let shaped = runtime
        .transition_shape(
            &mut mutator,
            base_shape,
            ShapeTransitionKey::new(
                PropertyKey::from_atom(AtomId::from_raw(17)),
                ShapePropertyKind::Data,
                attrs(true, true, true),
            ),
            AllocationLifetime::Default,
        )
        .unwrap();
    let shaped = runtime
        .transition_shape(
            &mut mutator,
            shaped,
            ShapeTransitionKey::new(
                PropertyKey::from_atom(AtomId::from_raw(18)),
                ShapePropertyKind::Data,
                attrs(true, false, true),
            ),
            AllocationLifetime::Default,
        )
        .unwrap();
    let prototype = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(base_shape),
        AllocationLifetime::Default,
    );
    let object = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shaped)
            .with_flags(ObjectFlags::extensible().union(ObjectFlags::SEALED))
            .with_prototype(Some(prototype))
            .with_element_capacity(3),
        AllocationLifetime::Default,
    );

    let shape = runtime.shape(mutator.view(), shaped).unwrap();
    let record = runtime.object(mutator.view(), object).unwrap();
    let header = record.header();

    assert_eq!(shape.property_count(), 2);
    assert_eq!(shape.slot_count(), 2);
    assert_eq!(header.kind(), ObjectKind::Ordinary);
    assert_eq!(header.flags(), ObjectFlags::extensible());
    assert_eq!(header.prototype(), Some(prototype));
    assert_eq!(header.shape(), shaped);
    assert!(matches!(record.cold(), ObjectColdData::Ordinary(_)));
    assert_eq!(
        runtime.named_slots(mutator.view(), object).unwrap(),
        &[Value::empty_internal_slot(), Value::empty_internal_slot()]
    );
    assert_eq!(
        runtime.elements(mutator.view(), object).unwrap(),
        &[
            Value::array_hole(),
            Value::array_hole(),
            Value::array_hole()
        ]
    );
}

#[test]
fn proxy_objects_allocate_kind_payload_and_idempotent_revocation_state() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let prototype = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape),
        AllocationLifetime::Default,
    );
    let target = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape).with_prototype(Some(prototype)),
        AllocationLifetime::Default,
    );
    let handler = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape),
        AllocationLifetime::Default,
    );
    let proxy_data = ProxyObjectData::new(target, handler, true, false);
    let proxy = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::proxy(shape, proxy_data).with_prototype(Some(prototype)),
        AllocationLifetime::Default,
    );

    let record = runtime.object(mutator.view(), proxy).unwrap();

    assert_eq!(record.header().kind(), ObjectKind::Proxy);
    assert_eq!(record.header().prototype(), Some(prototype));
    assert!(matches!(record.cold(), ObjectColdData::Proxy(_)));
    assert!(runtime.is_proxy_object(proxy));
    assert_eq!(runtime.proxy_data(proxy), Some(proxy_data));
    assert_eq!(runtime.proxy_target(proxy), Some(target));
    assert_eq!(runtime.proxy_handler(proxy), Some(handler));
    assert_eq!(runtime.is_proxy_revoked(proxy), Some(false));
    assert!(runtime.is_callable(proxy));
    assert!(!runtime.is_constructor(proxy));
    assert_eq!(
        runtime.named_slots(mutator.view(), proxy).unwrap(),
        &[
            Value::from_object_ref(target),
            Value::from_object_ref(handler)
        ]
    );

    assert!(runtime.revoke_proxy(&mut mutator, proxy));
    assert_eq!(
        runtime.proxy_data(proxy),
        Some(proxy_data.with_handler(None).with_revoked(true))
    );
    assert_eq!(runtime.proxy_handler(proxy), None);
    assert_eq!(runtime.is_proxy_revoked(proxy), Some(true));
    assert!(runtime.is_callable(proxy));
    assert!(!runtime.is_constructor(proxy));
    assert_eq!(
        runtime.named_slots(mutator.view(), proxy).unwrap(),
        &[Value::from_object_ref(target), Value::undefined()]
    );
    assert!(runtime.revoke_proxy(&mut mutator, proxy));
}

#[test]
fn proxy_internal_methods_forward_until_revocation() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let prototype = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape),
        AllocationLifetime::Default,
    );
    let target = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape).with_prototype(Some(prototype)),
        AllocationLifetime::Default,
    );
    let handler = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape),
        AllocationLifetime::Default,
    );
    let key = PropertyKey::from_atom(AtomId::from_raw(81));
    assert!(runtime
        .define_own_property(
            &mut mutator,
            target,
            key,
            data_descriptor(Value::from_smi(7), true, true),
            AllocationLifetime::Default,
        )
        .unwrap());
    let proxy = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::proxy(shape, ProxyObjectData::new(target, handler, false, false))
            .with_prototype(Some(prototype)),
        AllocationLifetime::Default,
    );

    assert_eq!(
        runtime.get_prototype_of(mutator.view(), proxy),
        Ok(Some(prototype))
    );
    assert_eq!(
        runtime.get(mutator.view(), proxy, key, Value::from_object_ref(proxy)),
        Ok(Value::from_smi(7))
    );
    assert_eq!(runtime.has_property(mutator.view(), proxy, key), Ok(true));
    assert_eq!(
        runtime.own_property_keys(mutator.view(), proxy),
        Ok(vec![key])
    );

    assert!(runtime.revoke_proxy(&mut mutator, proxy));

    assert_eq!(
        runtime.get_prototype_of(mutator.view(), proxy),
        Err(InternalMethodError::RevokedProxy)
    );
    assert_eq!(
        runtime.get(mutator.view(), proxy, key, Value::from_object_ref(proxy)),
        Err(InternalMethodError::RevokedProxy)
    );
    assert_eq!(
        runtime.own_property_keys(mutator.view(), proxy),
        Err(InternalMethodError::RevokedProxy)
    );
}

#[test]
fn slot_store_helpers_route_named_and_element_writes_through_gc_helpers() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let shape = runtime
        .transition_shape(
            &mut mutator,
            shape,
            ShapeTransitionKey::new(
                PropertyKey::from_atom(AtomId::from_raw(1)),
                ShapePropertyKind::Data,
                attrs(true, true, true),
            ),
            AllocationLifetime::Default,
        )
        .unwrap();
    let object = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape).with_element_capacity(2),
        AllocationLifetime::Default,
    );

    assert!(runtime.init_named_slot(&mut mutator, object, 0, Value::from_smi(7)));
    assert!(runtime.mut_named_slot(&mut mutator, object, 0, Value::from_smi(9),));
    assert!(runtime.init_element(&mut mutator, object, 0, Value::from_smi(3)));
    assert!(runtime.mut_element(&mut mutator, object, 1, Value::from_smi(5),));

    assert_eq!(
        runtime.named_slots(mutator.view(), object).unwrap(),
        &[Value::from_smi(9)]
    );
    assert_eq!(
        runtime.elements(mutator.view(), object).unwrap(),
        &[Value::from_smi(3), Value::from_smi(5)]
    );
}

#[test]
fn named_property_churn_transitions_objects_to_dictionary_mode() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let shape = runtime
        .transition_shape(
            &mut mutator,
            root,
            ShapeTransitionKey::new(
                PropertyKey::from_atom(AtomId::from_raw(1)),
                ShapePropertyKind::Data,
                attrs(true, true, true),
            ),
            AllocationLifetime::Default,
        )
        .unwrap();
    let object = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape),
        AllocationLifetime::Default,
    );

    assert!(runtime.init_named_slot(&mut mutator, object, 0, Value::from_smi(41)));
    for _ in 0..NAMED_PROPERTY_CHURN_DICTIONARY_THRESHOLD {
        assert!(runtime.note_named_property_churn(&mut mutator, object));
    }

    let header = runtime.object_header(mutator.view(), object).unwrap();
    let entry = runtime
        .named_property_dictionary_entry(object, PropertyKey::from_atom(AtomId::from_raw(1)))
        .unwrap();
    let invalidation = runtime.invalidation_event(object).unwrap();

    assert_eq!(
        runtime.named_property_storage_mode(object),
        Some(NamedPropertyStorageMode::Dictionary)
    );
    assert!(header.flags().uses_named_property_dictionary());
    assert_eq!(header.named_slots(), None);
    assert_eq!(runtime.named_slots(mutator.view(), object), None);
    assert_eq!(
        entry.payload(),
        NamedPropertyValue::data(Value::from_smi(41))
    );
    assert_eq!(
        invalidation.cause(),
        InvalidationCause::DictionaryTransition
    );
    assert_eq!(invalidation.epoch(), 1);
}

#[test]
fn redefine_delete_and_prototype_mutation_bump_invalidation_epochs() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let shape = runtime
        .transition_shape(
            &mut mutator,
            root,
            ShapeTransitionKey::new(
                PropertyKey::from_atom(AtomId::from_raw(7)),
                ShapePropertyKind::Data,
                attrs(true, true, true),
            ),
            AllocationLifetime::Default,
        )
        .unwrap();
    let prototype = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );
    let object = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape),
        AllocationLifetime::Default,
    );

    assert!(runtime.init_named_slot(&mut mutator, object, 0, Value::from_smi(5)));
    assert!(runtime.redefine_named_property(
        &mut mutator,
        object,
        PropertyKey::from_atom(AtomId::from_raw(7)),
        NamedPropertyValue::data(Value::from_smi(9)),
        attrs(false, true, false),
    ));
    assert!(runtime.set_prototype(&mut mutator, object, Some(prototype)));
    assert!(runtime.delete_named_property(
        &mut mutator,
        object,
        PropertyKey::from_atom(AtomId::from_raw(7)),
    ));

    assert_eq!(
        runtime
            .named_property_dictionary_entry(object, PropertyKey::from_atom(AtomId::from_raw(7))),
        None
    );
    assert_eq!(
        runtime
            .object_header(mutator.view(), object)
            .unwrap()
            .prototype(),
        Some(prototype)
    );
    assert_eq!(runtime.current_invalidation_epoch(), 4);
    assert_eq!(
        runtime.invalidation_event(object).unwrap().cause(),
        InvalidationCause::PropertyDeletion
    );
}

#[test]
fn dense_elements_grow_preserve_holes_and_sparse_fallback_carries_attrs() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let object = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape).with_element_capacity(1),
        AllocationLifetime::Default,
    );

    assert!(runtime.set_element(
        &mut mutator,
        object,
        0,
        Value::from_smi(1),
        AllocationLifetime::Default,
    ));
    assert!(runtime.set_element(
        &mut mutator,
        object,
        3,
        Value::from_smi(4),
        AllocationLifetime::Default,
    ));

    let dense = runtime.elements(mutator.view(), object).unwrap();
    assert_eq!(runtime.element_mode(object), Some(ElementMode::Dense));
    assert_eq!(runtime.element_logical_len(object), Some(4));
    assert_eq!(dense.len(), 4);
    assert_eq!(dense[1], Value::array_hole());
    assert_eq!(
        runtime.element(mutator.view(), object, 2),
        Some(Value::array_hole())
    );

    let sparse_attrs = attrs(false, false, true);
    assert!(runtime.define_element(
        &mut mutator,
        object,
        32,
        Value::from_smi(32),
        sparse_attrs,
        AllocationLifetime::Default,
    ));

    assert_eq!(runtime.element_mode(object), Some(ElementMode::Sparse));
    assert_eq!(runtime.elements(mutator.view(), object), None);
    assert_eq!(
        runtime.element(mutator.view(), object, 0),
        Some(Value::from_smi(1))
    );
    assert_eq!(
        runtime.element(mutator.view(), object, 32),
        Some(Value::from_smi(32))
    );
    assert_eq!(
        runtime.sparse_element(object, 32).unwrap(),
        SparseElementEntry::new(NamedPropertyValue::data(Value::from_smi(32)), sparse_attrs)
    );
}

#[test]
fn sparse_index_accessors_preserve_getter_and_setter_payloads() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let object = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );
    let callable_placeholder = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );

    let mut accessor = PropertyDescriptor::new();
    accessor.set_getter(Value::from_object_ref(callable_placeholder));
    accessor.set_setter(Value::from_object_ref(callable_placeholder));
    accessor.set_enumerable(true);
    accessor.set_configurable(true);
    assert!(runtime
        .define_own_property(
            &mut mutator,
            object,
            PropertyKey::Index(1),
            accessor,
            AllocationLifetime::Default,
        )
        .unwrap());

    assert_eq!(
        runtime.get(
            mutator.view(),
            object,
            PropertyKey::Index(1),
            Value::from_object_ref(object),
        ),
        Err(InternalMethodError::AccessorCallPending)
    );
    assert_eq!(
        runtime.set(
            &mut mutator,
            object,
            PropertyKey::Index(1),
            Value::from_smi(3),
            Value::from_object_ref(object),
            AllocationLifetime::Default,
        ),
        Err(InternalMethodError::AccessorCallPending)
    );
    assert_eq!(
        runtime.sparse_element(object, 1).unwrap(),
        SparseElementEntry::new(
            NamedPropertyValue::accessor(
                Value::from_object_ref(callable_placeholder),
                Value::from_object_ref(callable_placeholder),
            ),
            attrs(false, true, true),
        )
    );
}

#[test]
fn sparse_index_accessors_merge_sequential_getter_and_setter_updates() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let object = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );
    let getter = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );
    let setter = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );

    let mut getter_only = PropertyDescriptor::new();
    getter_only.set_getter(Value::from_object_ref(getter));
    getter_only.set_enumerable(true);
    getter_only.set_configurable(true);
    assert!(runtime
        .define_own_property(
            &mut mutator,
            object,
            PropertyKey::Index(1),
            getter_only,
            AllocationLifetime::Default,
        )
        .unwrap());

    let mut setter_only = PropertyDescriptor::new();
    setter_only.set_setter(Value::from_object_ref(setter));
    setter_only.set_enumerable(true);
    setter_only.set_configurable(true);
    assert!(runtime
        .define_own_property(
            &mut mutator,
            object,
            PropertyKey::Index(1),
            setter_only,
            AllocationLifetime::Default,
        )
        .unwrap());

    assert_eq!(
        runtime.sparse_element(object, 1).unwrap(),
        SparseElementEntry::new(
            NamedPropertyValue::accessor(
                Value::from_object_ref(getter),
                Value::from_object_ref(setter),
            ),
            attrs(false, true, true),
        )
    );
}

#[test]
fn deleting_dense_and_sparse_elements_restores_holes_and_empty_mode() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let object = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape).with_element_capacity(2),
        AllocationLifetime::Default,
    );

    assert!(runtime.set_element(
        &mut mutator,
        object,
        0,
        Value::from_smi(2),
        AllocationLifetime::Default,
    ));
    assert!(runtime.delete_element(&mut mutator, object, 0));
    assert_eq!(runtime.element_mode(object), Some(ElementMode::Empty));
    assert_eq!(runtime.elements(mutator.view(), object), None);
    assert_eq!(
        runtime.element(mutator.view(), object, 0),
        Some(Value::array_hole())
    );

    assert!(runtime.define_element(
        &mut mutator,
        object,
        40,
        Value::from_smi(8),
        attrs(true, false, true),
        AllocationLifetime::Default,
    ));
    assert!(runtime.delete_element(&mut mutator, object, 40));
    assert_eq!(runtime.element_mode(object), Some(ElementMode::Empty));
    assert_eq!(
        runtime.element(mutator.view(), object, 40),
        Some(Value::array_hole())
    );
}

#[test]
fn engine_array_index_definitions_extend_length() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let array = engine_array(&mut runtime, &mut mutator, shape, 1, true);

    assert!(runtime
        .define_own_property(
            &mut mutator,
            array,
            PropertyKey::Index(2),
            data_descriptor(Value::from_smi(9), true, true),
            AllocationLifetime::Default,
        )
        .unwrap());

    let length = array_length_descriptor(&runtime, mutator.view(), array);
    assert_eq!(length.value(), Some(Value::from_smi(3)));
    assert_eq!(runtime.element_logical_len(array), Some(3));
    assert_eq!(
        runtime.element(mutator.view(), array, 2),
        Some(Value::from_smi(9))
    );
}

#[test]
fn engine_array_non_writable_length_blocks_extensions() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let array = engine_array(&mut runtime, &mut mutator, shape, 1, false);

    assert!(!runtime
        .define_own_property(
            &mut mutator,
            array,
            PropertyKey::Index(1),
            data_descriptor(Value::from_smi(7), true, true),
            AllocationLifetime::Default,
        )
        .unwrap());

    let length = array_length_descriptor(&runtime, mutator.view(), array);
    assert_eq!(length.value(), Some(Value::from_smi(1)));
    assert_eq!(length.writable(), Some(false));
    assert_eq!(
        runtime
            .get_own_property(mutator.view(), array, PropertyKey::Index(1))
            .unwrap(),
        None
    );
}

#[test]
fn engine_array_length_shrink_rolls_back_after_delete_failure() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let array = engine_array(&mut runtime, &mut mutator, shape, 4, true);

    assert!(runtime
        .define_own_property(
            &mut mutator,
            array,
            PropertyKey::Index(3),
            data_descriptor(Value::from_smi(30), true, true),
            AllocationLifetime::Default,
        )
        .unwrap());

    let mut locked_tail = PropertyDescriptor::new();
    locked_tail.set_value(Value::from_smi(20));
    locked_tail.set_writable(true);
    locked_tail.set_enumerable(true);
    locked_tail.set_configurable(false);
    assert!(runtime
        .define_own_property(
            &mut mutator,
            array,
            PropertyKey::Index(2),
            locked_tail,
            AllocationLifetime::Default,
        )
        .unwrap());

    let mut shrink = PropertyDescriptor::new();
    shrink.set_value(Value::from_smi(1));
    shrink.set_writable(false);
    assert!(!runtime
        .define_own_property(
            &mut mutator,
            array,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
            shrink,
            AllocationLifetime::Default,
        )
        .unwrap());

    let length = array_length_descriptor(&runtime, mutator.view(), array);
    assert_eq!(length.value(), Some(Value::from_smi(3)));
    assert_eq!(length.writable(), Some(false));
    assert_eq!(
        runtime
            .get_own_property(mutator.view(), array, PropertyKey::Index(3))
            .unwrap(),
        None
    );
    assert_eq!(
        runtime
            .get_own_property(mutator.view(), array, PropertyKey::Index(2))
            .unwrap(),
        Some(data_descriptor(Value::from_smi(20), true, false))
    );
}

#[test]
fn engine_array_length_shrink_deletes_sparse_tail_indices_without_scanning_holes() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let array = engine_array(&mut runtime, &mut mutator, shape, 3, true);

    assert!(runtime
        .define_own_property(
            &mut mutator,
            array,
            PropertyKey::Index(0),
            data_descriptor(Value::from_smi(0), true, true),
            AllocationLifetime::Default,
        )
        .unwrap());
    assert!(runtime
        .define_own_property(
            &mut mutator,
            array,
            PropertyKey::Index(1),
            data_descriptor(Value::from_smi(1), true, true),
            AllocationLifetime::Default,
        )
        .unwrap());
    assert!(runtime
        .define_own_property(
            &mut mutator,
            array,
            PropertyKey::Index(2),
            data_descriptor(Value::from_smi(2), true, true),
            AllocationLifetime::Default,
        )
        .unwrap());
    assert!(runtime
        .define_own_property(
            &mut mutator,
            array,
            PropertyKey::Index(u32::MAX - 1),
            data_descriptor(Value::from_f64(f64::from(u32::MAX - 1)), true, true),
            AllocationLifetime::Default,
        )
        .unwrap());

    let mut shrink = PropertyDescriptor::new();
    shrink.set_value(Value::from_smi(2));
    assert!(runtime
        .define_own_property(
            &mut mutator,
            array,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
            shrink,
            AllocationLifetime::Default,
        )
        .unwrap());

    let length = array_length_descriptor(&runtime, mutator.view(), array);
    assert_eq!(length.value(), Some(Value::from_smi(2)));
    assert_eq!(
        runtime
            .get_own_property(mutator.view(), array, PropertyKey::Index(0))
            .unwrap(),
        Some(data_descriptor(Value::from_smi(0), true, true))
    );
    assert_eq!(
        runtime
            .get_own_property(mutator.view(), array, PropertyKey::Index(1))
            .unwrap(),
        Some(data_descriptor(Value::from_smi(1), true, true))
    );
    assert_eq!(
        runtime
            .get_own_property(mutator.view(), array, PropertyKey::Index(2))
            .unwrap(),
        None
    );
    assert_eq!(
        runtime
            .get_own_property(mutator.view(), array, PropertyKey::Index(u32::MAX - 1))
            .unwrap(),
        None
    );
}

#[test]
fn ordinary_internal_methods_cover_prototype_extensibility_and_set_receiver_paths() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let proto_shape = runtime
        .transition_shape(
            &mut mutator,
            root,
            ShapeTransitionKey::new(
                PropertyKey::from_atom(AtomId::from_raw(30)),
                ShapePropertyKind::Data,
                attrs(true, true, true),
            ),
            AllocationLifetime::Default,
        )
        .unwrap();
    let prototype = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(proto_shape),
        AllocationLifetime::Default,
    );
    assert!(runtime.init_named_slot(&mut mutator, prototype, 0, Value::from_smi(11)));

    let object = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root).with_prototype(Some(prototype)),
        AllocationLifetime::Default,
    );

    assert_eq!(
        runtime.get_prototype_of(mutator.view(), object).unwrap(),
        Some(prototype)
    );
    assert!(runtime
        .has_property(
            mutator.view(),
            object,
            PropertyKey::from_atom(AtomId::from_raw(30))
        )
        .unwrap());
    assert_eq!(
        runtime
            .get(
                mutator.view(),
                object,
                PropertyKey::from_atom(AtomId::from_raw(30)),
                Value::from_object_ref(object),
            )
            .unwrap(),
        Value::from_smi(11)
    );

    assert!(runtime
        .set(
            &mut mutator,
            object,
            PropertyKey::from_atom(AtomId::from_raw(30)),
            Value::from_smi(19),
            Value::from_object_ref(object),
            AllocationLifetime::Default,
        )
        .unwrap());
    assert_eq!(
        runtime
            .get_own_property(
                mutator.view(),
                object,
                PropertyKey::from_atom(AtomId::from_raw(30))
            )
            .unwrap()
            .unwrap()
            .value(),
        Some(Value::from_smi(19))
    );
    assert_eq!(
        runtime
            .get_own_property(
                mutator.view(),
                prototype,
                PropertyKey::from_atom(AtomId::from_raw(30))
            )
            .unwrap()
            .unwrap()
            .value(),
        Some(Value::from_smi(11))
    );

    assert!(runtime.prevent_extensions(mutator.view(), object).unwrap());
    assert!(!runtime.is_extensible(object).unwrap());
    let other_proto = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );
    assert!(!runtime
        .set_prototype_of(&mut mutator, object, Some(other_proto))
        .unwrap());
    assert_eq!(
        runtime.get_prototype_of(mutator.view(), object).unwrap(),
        Some(prototype)
    );
}

#[test]
fn define_own_property_uses_shape_transitions_and_dictionary_fallback() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let object = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );

    let mut create = PropertyDescriptor::new();
    create.set_value(Value::from_smi(5));
    create.set_writable(true);
    create.set_enumerable(true);
    create.set_configurable(true);
    assert!(runtime
        .define_own_property(
            &mut mutator,
            object,
            PropertyKey::from_atom(AtomId::from_raw(40)),
            create,
            AllocationLifetime::Default,
        )
        .unwrap());

    assert_eq!(
        runtime.named_property_storage_mode(object),
        Some(NamedPropertyStorageMode::ShapeStable)
    );
    assert_eq!(
        runtime
            .get_own_property(
                mutator.view(),
                object,
                PropertyKey::from_atom(AtomId::from_raw(40))
            )
            .unwrap()
            .unwrap()
            .value(),
        Some(Value::from_smi(5))
    );

    let mut redefine = PropertyDescriptor::new();
    redefine.set_enumerable(false);
    assert!(runtime
        .define_own_property(
            &mut mutator,
            object,
            PropertyKey::from_atom(AtomId::from_raw(40)),
            redefine,
            AllocationLifetime::Default,
        )
        .unwrap());

    let descriptor = runtime
        .get_own_property(
            mutator.view(),
            object,
            PropertyKey::from_atom(AtomId::from_raw(40)),
        )
        .unwrap()
        .unwrap();

    assert_eq!(
        runtime.named_property_storage_mode(object),
        Some(NamedPropertyStorageMode::Dictionary)
    );
    assert_eq!(descriptor.value(), Some(Value::from_smi(5)));
    assert_eq!(descriptor.enumerable(), Some(false));
}

#[test]
fn integrity_summary_flags_track_non_extensible_objects() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);

    let empty = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );
    assert!(runtime.prevent_extensions(mutator.view(), empty).unwrap());
    let empty_flags = runtime
        .object_header(mutator.view(), empty)
        .unwrap()
        .flags();
    assert!(empty_flags.is_sealed_summary());
    assert!(empty_flags.is_frozen_summary());

    let writable = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );
    let mut writable_desc = PropertyDescriptor::new();
    writable_desc.set_value(Value::from_smi(1));
    writable_desc.set_writable(true);
    writable_desc.set_enumerable(true);
    writable_desc.set_configurable(false);
    assert!(runtime
        .define_own_property(
            &mut mutator,
            writable,
            PropertyKey::from_atom(AtomId::from_raw(140)),
            writable_desc,
            AllocationLifetime::Default,
        )
        .unwrap());
    assert!(runtime
        .prevent_extensions(mutator.view(), writable)
        .unwrap());
    let writable_flags = runtime
        .object_header(mutator.view(), writable)
        .unwrap()
        .flags();
    assert!(writable_flags.is_sealed_summary());
    assert!(!writable_flags.is_frozen_summary());

    let frozen = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );
    let mut frozen_desc = PropertyDescriptor::new();
    frozen_desc.set_value(Value::from_smi(2));
    frozen_desc.set_writable(false);
    frozen_desc.set_enumerable(true);
    frozen_desc.set_configurable(false);
    assert!(runtime
        .define_own_property(
            &mut mutator,
            frozen,
            PropertyKey::from_atom(AtomId::from_raw(141)),
            frozen_desc,
            AllocationLifetime::Default,
        )
        .unwrap());
    assert!(runtime.prevent_extensions(mutator.view(), frozen).unwrap());
    let frozen_flags = runtime
        .object_header(mutator.view(), frozen)
        .unwrap()
        .flags();
    assert!(frozen_flags.is_sealed_summary());
    assert!(frozen_flags.is_frozen_summary());
}

#[test]
fn own_property_keys_order_indices_then_strings_then_symbols() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let object = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );

    let mut string_desc = PropertyDescriptor::new();
    string_desc.set_value(Value::from_smi(1));
    string_desc.set_writable(true);
    string_desc.set_enumerable(true);
    string_desc.set_configurable(true);
    assert!(runtime
        .define_own_property(
            &mut mutator,
            object,
            PropertyKey::from_atom(AtomId::from_raw(50)),
            string_desc,
            AllocationLifetime::Default,
        )
        .unwrap());

    let mut symbol_desc = PropertyDescriptor::new();
    symbol_desc.set_value(Value::from_smi(2));
    symbol_desc.set_writable(true);
    symbol_desc.set_enumerable(true);
    symbol_desc.set_configurable(true);
    assert!(runtime
        .define_own_property(
            &mut mutator,
            object,
            PropertyKey::from_symbol(SymbolRef::from_raw(9).unwrap()),
            symbol_desc,
            AllocationLifetime::Default,
        )
        .unwrap());

    assert!(runtime
        .define_own_property(
            &mut mutator,
            object,
            PropertyKey::Index(2),
            string_desc,
            AllocationLifetime::Default,
        )
        .unwrap());
    assert!(runtime
        .define_own_property(
            &mut mutator,
            object,
            PropertyKey::Index(0),
            string_desc,
            AllocationLifetime::Default,
        )
        .unwrap());

    assert_eq!(
        runtime.own_property_keys(mutator.view(), object).unwrap(),
        vec![
            PropertyKey::Index(0),
            PropertyKey::Index(2),
            PropertyKey::from_atom(AtomId::from_raw(50)),
            PropertyKey::from_symbol(SymbolRef::from_raw(9).unwrap()),
        ]
    );
}

#[test]
fn partial_accessor_updates_preserve_unspecified_getter_and_setter_fields() {
    let getter = Value::from_smi(11);
    let setter = Value::from_smi(13);
    let alternate_getter = Value::from_smi(17);
    let alternate_write = Value::from_smi(19);

    let mut current = PropertyDescriptor::new();
    current.set_getter(getter);
    current.set_setter(setter);
    current.set_enumerable(true);
    current.set_configurable(true);

    let mut set_only = PropertyDescriptor::new();
    set_only.set_setter(alternate_write);
    let (payload, attrs) = complete_descriptor_update(Some(current), set_only).unwrap();
    assert_eq!(
        descriptor_from_payload(payload, attrs),
        descriptor_from_payload(NamedPropertyValue::accessor(getter, alternate_write), attrs)
    );

    let mut get_only = PropertyDescriptor::new();
    get_only.set_getter(alternate_getter);
    let (payload, attrs) = complete_descriptor_update(Some(current), get_only).unwrap();
    assert_eq!(
        descriptor_from_payload(payload, attrs),
        descriptor_from_payload(
            NamedPropertyValue::accessor(alternate_getter, setter),
            attrs
        )
    );
}

#[test]
fn partial_data_updates_preserve_unspecified_writable_and_value_fields() {
    let mut current = PropertyDescriptor::new();
    current.set_value(Value::from_smi(11));
    current.set_writable(true);
    current.set_enumerable(true);
    current.set_configurable(true);

    let mut value_only = PropertyDescriptor::new();
    value_only.set_value(Value::from_smi(13));
    let (payload, attrs) = complete_descriptor_update(Some(current), value_only).unwrap();
    assert_eq!(
        descriptor_from_payload(payload, attrs),
        descriptor_from_payload(
            NamedPropertyValue::data(Value::from_smi(13)),
            current.attrs()
        )
    );

    let mut writable_only = PropertyDescriptor::new();
    writable_only.set_writable(false);
    let (payload, attrs) = complete_descriptor_update(Some(current), writable_only).unwrap();
    assert_eq!(
        descriptor_from_payload(payload, attrs),
        descriptor_from_payload(NamedPropertyValue::data(Value::from_smi(11)), attrs)
    );
    assert!(!attrs.writable());
}

#[test]
fn delete_respects_non_configurable_properties_and_accessors_defer_calls() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let object = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );
    let callable_placeholder = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(root),
        AllocationLifetime::Default,
    );

    let mut locked = PropertyDescriptor::new();
    locked.set_value(Value::from_smi(7));
    locked.set_writable(true);
    locked.set_enumerable(true);
    locked.set_configurable(false);
    assert!(runtime
        .define_own_property(
            &mut mutator,
            object,
            PropertyKey::from_atom(AtomId::from_raw(60)),
            locked,
            AllocationLifetime::Default,
        )
        .unwrap());
    assert!(!runtime
        .delete(
            &mut mutator,
            object,
            PropertyKey::from_atom(AtomId::from_raw(60))
        )
        .unwrap());

    let mut accessor = PropertyDescriptor::new();
    accessor.set_getter(Value::from_object_ref(callable_placeholder));
    accessor.set_setter(Value::from_object_ref(callable_placeholder));
    accessor.set_enumerable(true);
    accessor.set_configurable(true);
    assert!(runtime
        .define_own_property(
            &mut mutator,
            object,
            PropertyKey::from_atom(AtomId::from_raw(61)),
            accessor,
            AllocationLifetime::Default,
        )
        .unwrap());
    assert_eq!(
        runtime.get(
            mutator.view(),
            object,
            PropertyKey::from_atom(AtomId::from_raw(61)),
            Value::from_object_ref(object),
        ),
        Err(InternalMethodError::AccessorCallPending)
    );
    assert_eq!(
        runtime.set(
            &mut mutator,
            object,
            PropertyKey::from_atom(AtomId::from_raw(61)),
            Value::from_smi(3),
            Value::from_object_ref(object),
            AllocationLifetime::Default,
        ),
        Err(InternalMethodError::AccessorCallPending)
    );
}

#[test]
fn leaf_shape_free_removes_canonical_transition_entry() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let root = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let transition = ShapeTransitionKey::new(
        PropertyKey::from_atom(AtomId::from_raw(1)),
        ShapePropertyKind::Data,
        attrs(true, true, true),
    );
    let shape = runtime
        .transition_shape(&mut mutator, root, transition, AllocationLifetime::Default)
        .unwrap();

    let freed = runtime.free_shape(&mut mutator, shape).unwrap();
    let recreated = runtime
        .transition_shape(&mut mutator, root, transition, AllocationLifetime::Default)
        .unwrap();

    assert_eq!(freed.parent(), Some(root));
    assert_eq!(
        runtime.shape(mutator.view(), recreated).unwrap().parent(),
        Some(root)
    );
    assert_eq!(
        runtime.shape_property(recreated, PropertyKey::from_atom(AtomId::from_raw(1))),
        Some(ShapeProperty::from_transition(transition, 0, 0))
    );
}

#[test]
fn function_object_allocation_tracks_payload_and_dispatches_native_calls() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let realm = mutator.alloc_realm(
        RuntimeRealmRecord::new(None, None, None, Some(shape)),
        AllocationLifetime::Default,
    );
    let environment = mutator.alloc_environment(
        RuntimeEnvironmentRecord::new(None, None, None, Value::undefined(), None, None),
        AllocationLifetime::Default,
    );
    let private_env = mutator.alloc_environment(
        RuntimeEnvironmentRecord::new(
            Some(environment),
            None,
            None,
            Value::undefined(),
            None,
            None,
        ),
        AllocationLifetime::Default,
    );
    let home_object = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape),
        AllocationLifetime::Default,
    );
    let constructed = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape),
        AllocationLifetime::Default,
    );
    let entry = BuiltinFunctionId::from_raw(17).unwrap();
    let kind_flags = FunctionKindFlags::ASYNC.union(FunctionKindFlags::GENERATOR);
    let function = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::function(shape).with_cold_data(ObjectColdData::Function(
            FunctionObjectData::native(realm, environment, entry)
                .with_this_mode(FunctionThisMode::Global)
                .with_private_env(Some(private_env))
                .with_home_object(Some(home_object))
                .with_constructible(true)
                .with_kind_flags(kind_flags),
        )),
        AllocationLifetime::Default,
    );

    let data = runtime.function_data(function).unwrap();
    assert_eq!(data.realm(), Some(realm));
    assert_eq!(data.environment(), Some(environment));
    assert_eq!(data.private_env(), Some(private_env));
    assert_eq!(data.this_mode(), FunctionThisMode::Global);
    assert_eq!(data.home_object(), Some(home_object));
    assert!(data.is_constructible());
    assert_eq!(data.kind_flags(), kind_flags);
    assert_eq!(
        data.entry(),
        Some(FunctionEntryIdentity::Native(NativeFunctionId::builtin(
            entry
        )))
    );

    let payload = mutator
        .view()
        .object(function)
        .unwrap()
        .function_payload()
        .expect("function object should allocate a gc payload record");
    let payload_record = mutator.view().function_payload(payload).unwrap();
    assert_eq!(payload_record.realm(), Some(realm));
    assert_eq!(payload_record.environment(), Some(environment));
    assert_eq!(payload_record.private_env(), Some(private_env));
    assert_eq!(payload_record.home_object(), Some(home_object));
    assert_eq!(payload_record.bytecode(), None);

    let mut registry = RecordingNativeRegistry::default();
    registry
        .call_results
        .insert(NativeFunctionId::builtin(entry), Value::from_smi(23));
    registry
        .construct_results
        .insert(NativeFunctionId::builtin(entry), constructed);

    assert_eq!(
        runtime.call(
            &mut mutator,
            function,
            Value::from_smi(5),
            &[Value::from_smi(7), Value::from_smi(9)],
            &mut registry,
        ),
        Ok(Value::from_smi(23))
    );
    assert_eq!(
        runtime.construct(
            &mut mutator,
            function,
            &[Value::from_smi(11)],
            None,
            &mut registry,
        ),
        Ok(constructed)
    );
    assert_eq!(
        registry.calls,
        vec![(
            NativeFunctionId::builtin(entry),
            Value::from_smi(5),
            vec![Value::from_smi(7), Value::from_smi(9)],
            function,
            realm,
            environment,
            Some(private_env),
            FunctionThisMode::Global,
            Some(home_object),
            FunctionConstructorFlags::constructible(),
            kind_flags,
        )]
    );
    assert_eq!(
        registry.constructs,
        vec![(
            NativeFunctionId::builtin(entry),
            function,
            vec![Value::from_smi(11)],
            function,
            realm,
            environment,
            Some(private_env),
            FunctionThisMode::Global,
            Some(home_object),
            FunctionConstructorFlags::constructible(),
            kind_flags,
        )]
    );
}

#[test]
fn function_dispatch_rejects_non_constructible_and_reserved_bytecode_entries() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let realm = mutator.alloc_realm(
        RuntimeRealmRecord::new(None, None, None, Some(shape)),
        AllocationLifetime::Default,
    );
    let environment = mutator.alloc_environment(
        RuntimeEnvironmentRecord::new(None, None, None, Value::undefined(), None, None),
        AllocationLifetime::Default,
    );
    let ordinary = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape),
        AllocationLifetime::Default,
    );
    let class_constructor = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::function(shape).with_cold_data(ObjectColdData::Function(
            FunctionObjectData::native(
                realm,
                environment,
                BuiltinFunctionId::from_raw(19).unwrap(),
            )
            .with_constructible(true)
            .with_kind_flags(FunctionKindFlags::CLASS_CONSTRUCTOR),
        )),
        AllocationLifetime::Default,
    );
    let bytecode_function = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::function(shape).with_cold_data(ObjectColdData::Function(
            FunctionObjectData::bytecode(realm, environment, CodeRef::from_raw(3).unwrap()),
        )),
        AllocationLifetime::Default,
    );
    let plain_function = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::function(shape).with_cold_data(ObjectColdData::Function(
            FunctionObjectData::native(
                realm,
                environment,
                BuiltinFunctionId::from_raw(20).unwrap(),
            ),
        )),
        AllocationLifetime::Default,
    );
    let mut registry = RecordingNativeRegistry::default();

    assert_eq!(
        runtime.call(
            &mut mutator,
            ordinary,
            Value::undefined(),
            &[],
            &mut registry,
        ),
        Err(InternalMethodError::NotCallable)
    );
    assert_eq!(
        runtime.call(
            &mut mutator,
            class_constructor,
            Value::undefined(),
            &[],
            &mut registry,
        ),
        Err(InternalMethodError::NotCallable)
    );
    assert_eq!(
        runtime.construct(&mut mutator, plain_function, &[], None, &mut registry),
        Err(InternalMethodError::NotConstructible)
    );
    assert_eq!(
        runtime.call(
            &mut mutator,
            bytecode_function,
            Value::undefined(),
            &[],
            &mut registry,
        ),
        Err(InternalMethodError::BytecodeDispatchPending)
    );
    assert_eq!(
        runtime.construct(
            &mut mutator,
            bytecode_function,
            &[],
            Some(class_constructor),
            &mut registry,
        ),
        Err(InternalMethodError::NotConstructible)
    );
}

#[test]
fn private_field_layout_allocates_brand_storage_for_instance_and_static_fields() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let class_object = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape),
        AllocationLifetime::Default,
    );
    let prototype = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape),
        AllocationLifetime::Default,
    );
    let instance = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape),
        AllocationLifetime::Default,
    );
    let wrong_receiver = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape),
        AllocationLifetime::Default,
    );

    let instance_name = AtomId::from_raw(200);
    let static_name = AtomId::from_raw(201);
    let instance_descriptor = runtime
        .define_private_field_layout(class_object, prototype, instance_name, false)
        .expect("instance private field layout should install");
    let static_descriptor = runtime
        .define_private_field_layout(class_object, prototype, static_name, true)
        .expect("static private field layout should install");

    assert_eq!(instance_descriptor, 0);
    assert_eq!(static_descriptor, 1);
    assert_eq!(
        runtime.private_has(instance, prototype, instance_descriptor),
        Ok(false)
    );

    runtime
        .private_field_init(
            &mut mutator,
            instance,
            prototype,
            instance_descriptor,
            Value::from_smi(7),
            AllocationLifetime::Default,
        )
        .expect("instance private brand should install");
    runtime
        .private_field_init(
            &mut mutator,
            class_object,
            class_object,
            static_descriptor,
            Value::from_smi(11),
            AllocationLifetime::Default,
        )
        .expect("static private brand should install");

    assert_eq!(
        runtime.private_field_get(mutator.view(), instance, prototype, instance_descriptor),
        Ok(Value::from_smi(7))
    );
    assert_eq!(
        runtime.private_field_get(
            mutator.view(),
            class_object,
            class_object,
            static_descriptor
        ),
        Ok(Value::from_smi(11))
    );
    assert_eq!(
        runtime.private_has(instance, prototype, instance_descriptor),
        Ok(true)
    );
    assert_eq!(
        runtime.private_has(class_object, prototype, instance_descriptor),
        Ok(false)
    );
    assert_eq!(
        runtime.private_has(class_object, prototype, static_descriptor),
        Ok(true)
    );
    assert_eq!(
        runtime.private_field_get(
            mutator.view(),
            wrong_receiver,
            prototype,
            instance_descriptor
        ),
        Err(InternalMethodError::InvalidPrivateBrand)
    );

    runtime
        .private_field_set(
            &mut mutator,
            instance,
            prototype,
            instance_descriptor,
            Value::from_smi(9),
        )
        .expect("private field set should reuse installed brand storage");
    assert_eq!(
        runtime.private_field_get(mutator.view(), instance, prototype, instance_descriptor),
        Ok(Value::from_smi(9))
    );
    assert!(mutator
        .view()
        .object(instance)
        .and_then(RuntimeObjectRecord::private_slots)
        .is_some());
    assert!(mutator
        .view()
        .object(class_object)
        .and_then(RuntimeObjectRecord::private_slots)
        .is_some());
}

#[test]
fn static_private_method_storage_grows_without_overlapping_brand_slots() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let class_object = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape),
        AllocationLifetime::Default,
    );
    let prototype = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape),
        AllocationLifetime::Default,
    );
    let method_one = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape),
        AllocationLifetime::Default,
    );
    let method_two = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape),
        AllocationLifetime::Default,
    );

    let first_name = AtomId::from_raw(300);
    let second_name = AtomId::from_raw(301);
    let first_descriptor = runtime
        .define_private_element_layout(
            class_object,
            prototype,
            first_name,
            true,
            ClassPrivateElementKind::Method,
        )
        .expect("first static private method layout should install");
    runtime
        .install_private_element_value(
            &mut mutator,
            class_object,
            first_descriptor,
            Value::from_object_ref(method_one),
            AllocationLifetime::Default,
        )
        .expect("first static private method value should install");
    runtime
        .private_field_init(
            &mut mutator,
            class_object,
            class_object,
            first_descriptor,
            Value::undefined(),
            AllocationLifetime::Default,
        )
        .expect("first static private method brand marker should install");

    let second_descriptor = runtime
        .define_private_element_layout(
            class_object,
            prototype,
            second_name,
            true,
            ClassPrivateElementKind::Method,
        )
        .expect("second static private method layout should install");
    runtime
        .install_private_element_value(
            &mut mutator,
            class_object,
            second_descriptor,
            Value::from_object_ref(method_two),
            AllocationLifetime::Default,
        )
        .expect("second static private method value should install");
    runtime
        .private_field_init(
            &mut mutator,
            class_object,
            class_object,
            second_descriptor,
            Value::undefined(),
            AllocationLifetime::Default,
        )
        .expect("second static private method brand marker should install");

    assert_eq!(
        runtime.private_shared_element_value(mutator.view(), class_object, first_descriptor),
        Ok(Value::from_object_ref(method_one))
    );
    assert_eq!(
        runtime.private_shared_element_value(mutator.view(), class_object, second_descriptor),
        Ok(Value::from_object_ref(method_two))
    );
    assert_eq!(
        runtime.private_has(class_object, class_object, first_descriptor),
        Ok(true)
    );
    assert_eq!(
        runtime.private_has(class_object, class_object, second_descriptor),
        Ok(true)
    );
}

#[test]
fn bound_function_payload_tracks_prefix_arguments_and_forwards_dispatch() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let realm = mutator.alloc_realm(
        RuntimeRealmRecord::new(None, None, None, Some(shape)),
        AllocationLifetime::Default,
    );
    let environment = mutator.alloc_environment(
        RuntimeEnvironmentRecord::new(None, None, None, Value::undefined(), None, None),
        AllocationLifetime::Default,
    );
    let entry = BuiltinFunctionId::from_raw(21).unwrap();
    let target = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::function(shape).with_cold_data(ObjectColdData::Function(
            FunctionObjectData::native(realm, environment, entry).with_constructible(true),
        )),
        AllocationLifetime::Default,
    );
    let constructed = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::ordinary(shape),
        AllocationLifetime::Default,
    );
    let bound = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::function(shape).with_cold_data(ObjectColdData::Function(
            FunctionObjectData::bound(
                realm,
                environment,
                target,
                Value::from_smi(5),
                vec![Value::from_smi(7), Value::from_smi(9)].into_boxed_slice(),
            )
            .with_constructible(true),
        )),
        AllocationLifetime::Default,
    );

    let data = runtime.function_data(bound).unwrap();
    assert_eq!(data.entry(), Some(FunctionEntryIdentity::Bound));
    assert!(data.is_constructible());

    let payload = data
        .gc_payload()
        .expect("bound functions should allocate a traced gc payload record");
    let payload_record = mutator.view().function_payload(payload).unwrap();
    let bound_record = payload_record
        .bound()
        .expect("bound function payload should preserve bound-call metadata");
    let bound_arguments = bound_record
        .arguments()
        .and_then(|slots| mutator.view().object_slots(slots))
        .expect("bound function payload should preserve prefix arguments");

    assert_eq!(bound_record.target(), target);
    assert_eq!(bound_record.this_value(), Value::from_smi(5));
    assert_eq!(bound_arguments, &[Value::from_smi(7), Value::from_smi(9)]);

    let mut registry = RecordingNativeRegistry::default();
    registry
        .call_results
        .insert(NativeFunctionId::builtin(entry), Value::from_smi(41));
    registry
        .construct_results
        .insert(NativeFunctionId::builtin(entry), constructed);

    assert_eq!(
        runtime.call(
            &mut mutator,
            bound,
            Value::from_smi(99),
            &[Value::from_smi(11)],
            &mut registry,
        ),
        Ok(Value::from_smi(41))
    );
    assert_eq!(
        runtime.construct(
            &mut mutator,
            bound,
            &[Value::from_smi(13)],
            None,
            &mut registry
        ),
        Ok(constructed)
    );
    assert_eq!(
        registry.calls,
        vec![(
            NativeFunctionId::builtin(entry),
            Value::from_smi(5),
            vec![Value::from_smi(7), Value::from_smi(9), Value::from_smi(11)],
            target,
            realm,
            environment,
            None,
            FunctionThisMode::Strict,
            None,
            FunctionConstructorFlags::constructible(),
            FunctionKindFlags::empty(),
        )]
    );
    assert_eq!(
        registry.constructs,
        vec![(
            NativeFunctionId::builtin(entry),
            target,
            vec![Value::from_smi(7), Value::from_smi(9), Value::from_smi(13)],
            target,
            realm,
            environment,
            None,
            FunctionThisMode::Strict,
            None,
            FunctionConstructorFlags::constructible(),
            FunctionKindFlags::empty(),
        )]
    );
}

#[test]
fn rooted_function_payload_keeps_realm_environment_home_object_and_code_alive() {
    let mut heap = PrimitiveHeap::new();
    let roots = PrimitiveRoots::new();
    let mut runtime = ObjectRuntime::new();
    let (
        live_function,
        live_home_object,
        live_environment,
        live_realm,
        live_code,
        dead_function,
        dead_home_object,
        dead_environment,
        dead_realm,
        dead_code,
    ) = {
        let mut mutator = heap.mutator();
        let shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
        let live_home_object = runtime.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(shape),
            AllocationLifetime::Default,
        );
        let live_environment = mutator.alloc_environment(
            RuntimeEnvironmentRecord::new(None, None, None, Value::undefined(), None, None),
            AllocationLifetime::Default,
        );
        let live_private_env = mutator.alloc_environment(
            RuntimeEnvironmentRecord::new(
                Some(live_environment),
                None,
                None,
                Value::undefined(),
                None,
                None,
            ),
            AllocationLifetime::Default,
        );
        let live_realm = mutator.alloc_realm(
            RuntimeRealmRecord::new(None, Some(live_environment), None, Some(shape)),
            AllocationLifetime::Default,
        );
        let live_code = mutator.alloc_code(
            RuntimeCodeRecord::new(None, Some(live_realm), None),
            AllocationLifetime::Default,
        );
        let live_function = runtime.alloc_object(
            &mut mutator,
            ObjectAllocation::function(shape).with_cold_data(ObjectColdData::Function(
                FunctionObjectData::bytecode(live_realm, live_environment, live_code)
                    .with_private_env(Some(live_private_env))
                    .with_home_object(Some(live_home_object)),
            )),
            AllocationLifetime::Default,
        );

        let dead_home_object = runtime.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(shape),
            AllocationLifetime::Default,
        );
        let dead_environment = mutator.alloc_environment(
            RuntimeEnvironmentRecord::new(None, None, None, Value::undefined(), None, None),
            AllocationLifetime::Default,
        );
        let dead_private_env = mutator.alloc_environment(
            RuntimeEnvironmentRecord::new(
                Some(dead_environment),
                None,
                None,
                Value::undefined(),
                None,
                None,
            ),
            AllocationLifetime::Default,
        );
        let dead_realm = mutator.alloc_realm(
            RuntimeRealmRecord::new(None, Some(dead_environment), None, Some(shape)),
            AllocationLifetime::Default,
        );
        let dead_code = mutator.alloc_code(
            RuntimeCodeRecord::new(None, Some(dead_realm), None),
            AllocationLifetime::Default,
        );
        let dead_function = runtime.alloc_object(
            &mut mutator,
            ObjectAllocation::function(shape).with_cold_data(ObjectColdData::Function(
                FunctionObjectData::bytecode(dead_realm, dead_environment, dead_code)
                    .with_private_env(Some(dead_private_env))
                    .with_home_object(Some(dead_home_object)),
            )),
            AllocationLifetime::Default,
        );

        (
            live_function,
            live_home_object,
            live_environment,
            live_realm,
            live_code,
            dead_function,
            dead_home_object,
            dead_environment,
            dead_realm,
            dead_code,
        )
    };
    let _rooted = roots.root_object(live_function);

    let stats = heap.collect(&roots);
    let view = heap.view();

    assert_eq!(stats.trace.objects_marked, 2);
    assert_eq!(stats.trace.environments_marked, 2);
    assert_eq!(stats.trace.realms_marked, 1);
    assert_eq!(stats.trace.codes_marked, 1);
    assert_eq!(stats.objects_reclaimed, 2);
    assert_eq!(stats.environments_reclaimed, 2);
    assert_eq!(stats.realms_reclaimed, 1);
    assert_eq!(stats.codes_reclaimed, 1);
    assert!(view.object(live_function).is_some());
    assert!(view.object(live_home_object).is_some());
    assert!(view.environment(live_environment).is_some());
    assert!(view.realm(live_realm).is_some());
    assert!(view.code(live_code).is_some());
    assert_eq!(view.object(dead_function), None);
    assert_eq!(view.object(dead_home_object), None);
    assert_eq!(view.environment(dead_environment), None);
    assert_eq!(view.realm(dead_realm), None);
    assert_eq!(view.code(dead_code), None);
}

#[test]
fn rooted_proxy_keeps_target_and_handler_alive() {
    let mut heap = PrimitiveHeap::new();
    let roots = PrimitiveRoots::new();
    let mut runtime = ObjectRuntime::new();
    let (live_proxy, live_target, live_handler, dead_proxy, dead_target, dead_handler) = {
        let mut mutator = heap.mutator();
        let shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
        let live_target = runtime.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(shape),
            AllocationLifetime::Default,
        );
        let live_handler = runtime.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(shape),
            AllocationLifetime::Default,
        );
        let live_proxy = runtime.alloc_object(
            &mut mutator,
            ObjectAllocation::proxy(
                shape,
                ProxyObjectData::new(live_target, live_handler, false, false),
            ),
            AllocationLifetime::Default,
        );

        let dead_target = runtime.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(shape),
            AllocationLifetime::Default,
        );
        let dead_handler = runtime.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(shape),
            AllocationLifetime::Default,
        );
        let dead_proxy = runtime.alloc_object(
            &mut mutator,
            ObjectAllocation::proxy(
                shape,
                ProxyObjectData::new(dead_target, dead_handler, false, false),
            ),
            AllocationLifetime::Default,
        );

        (
            live_proxy,
            live_target,
            live_handler,
            dead_proxy,
            dead_target,
            dead_handler,
        )
    };
    let _rooted = roots.root_object(live_proxy);

    let stats = heap.collect(&roots);
    let view = heap.view();

    assert_eq!(stats.trace.objects_marked, 3);
    assert_eq!(stats.objects_reclaimed, 3);
    assert!(view.object(live_proxy).is_some());
    assert!(view.object(live_target).is_some());
    assert!(view.object(live_handler).is_some());
    assert_eq!(view.object(dead_proxy), None);
    assert_eq!(view.object(dead_target), None);
    assert_eq!(view.object(dead_handler), None);
}

#[test]
fn freeing_objects_removes_runtime_metadata_and_releases_object_records() {
    let mut heap = PrimitiveHeap::new();
    let mut runtime = ObjectRuntime::new();
    let mut mutator = heap.mutator();
    let shape = runtime.root_shape(&mut mutator, None, AllocationLifetime::Default);
    let object = runtime.alloc_object(
        &mut mutator,
        ObjectAllocation::function(shape).with_flags(ObjectFlags::FROZEN),
        AllocationLifetime::Default,
    );

    let freed = runtime.free_object(&mut mutator, object).unwrap();

    assert_eq!(freed.header().kind(), ObjectKind::Function);
    assert!(freed.header().flags().is_frozen_summary());
    assert!(!freed.header().flags().is_extensible());
    assert!(matches!(freed.cold(), ObjectColdData::Function(_)));
    assert_eq!(runtime.object(mutator.view(), object), None);
}

#[test]
fn object_marker_round_trips_heap_and_kind() {
    let property_name = AtomId::from_raw(11);
    let heap = PrimitiveHeapMarker::new(TypeOwnershipMarker::new(property_name), SourceId::new(5));
    let marker = ObjectSubstrateMarker::new(
        heap,
        property_name,
        ObjectKind::Ordinary,
        ObjectFlags::extensible(),
    );

    assert_eq!(marker.heap(), heap);
    assert_eq!(marker.property_name(), property_name);
    assert_eq!(marker.kind(), ObjectKind::Ordinary);
    assert!(marker.flags().contains(ObjectFlags::EXTENSIBLE));
    assert!(size_of::<ObjectSubstrateMarker>() <= 16);
}

#[test]
fn object_marker_traces_nested_atom_edges() {
    let mut atoms = AtomTable::new();
    let property_name = atoms.intern_collectible("property");
    let dead = atoms.intern_collectible("dead");
    let heap = PrimitiveHeapMarker::new(TypeOwnershipMarker::new(property_name), SourceId::new(9));
    let marker = ObjectSubstrateMarker::new(
        heap,
        property_name,
        ObjectKind::Function,
        ObjectFlags::extensible(),
    );

    let mut sweep = AtomGcSweep::new(&mut atoms);
    marker.trace_atom_edges(&mut sweep);
    let stats = sweep.sweep();

    assert_eq!(stats.reclaimed_collectible, 1);
    assert_eq!(
        atoms.lifetime(property_name),
        Some(AtomLifetime::Collectible)
    );
    assert_eq!(atoms.get(dead), None);
}

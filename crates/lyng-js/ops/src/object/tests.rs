use super::*;
use crate::errors::throw_type_error;
use lyng_js_common::WellKnownAtom;
use lyng_js_env::{Agent, Runtime};
use lyng_js_gc::{AllocationLifetime, BigIntSign, SymbolFlags};
use lyng_js_host::NoopHostHooks;
use lyng_js_objects::{
    ArrayBufferObjectData, FunctionConstructorFlags, FunctionObjectData, FunctionThisMode,
    InternalMethodResult, NativeCallRequest, NativeConstructRequest, NativeFunctionRegistry,
    ObjectAllocation, ObjectColdData, ObjectRuntime, OrdinaryObjectData, PrimitiveWrapperKind,
    TemporalInstantObjectData, TemporalObjectData, TemporalObjectKind, TypedArrayElementKind,
    TypedArrayObjectData,
};
use lyng_js_types::{
    AbruptCompletion, BuiltinFunctionId, EnvironmentRef, PropertyKey, RealmRef, WellKnownSymbolId,
};

fn install_test_wrapper_prototypes(agent: &mut Agent) -> RealmRef {
    let default_realm = agent.default_realm().expect("default realm should exist");
    let root_shape = default_realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let (
        object_prototype,
        string_prototype,
        number_prototype,
        bigint_prototype,
        boolean_prototype,
        symbol_prototype,
    ) = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let object_prototype = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        );
        let string_prototype = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(object_prototype)),
            AllocationLifetime::Default,
        );
        let number_prototype = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(object_prototype)),
            AllocationLifetime::Default,
        );
        let bigint_prototype = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(object_prototype)),
            AllocationLifetime::Default,
        );
        let boolean_prototype = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(object_prototype)),
            AllocationLifetime::Default,
        );
        let symbol_prototype = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(object_prototype)),
            AllocationLifetime::Default,
        );
        (
            object_prototype,
            string_prototype,
            number_prototype,
            bigint_prototype,
            boolean_prototype,
            symbol_prototype,
        )
    });
    let intrinsics = default_realm
        .intrinsics()
        .with_object_prototype(Some(object_prototype))
        .with_string_prototype(Some(string_prototype))
        .with_number_prototype(Some(number_prototype))
        .with_bigint_prototype(Some(bigint_prototype))
        .with_boolean_prototype(Some(boolean_prototype))
        .with_symbol_prototype(Some(symbol_prototype));
    assert!(agent.set_realm_intrinsics(default_realm.id(), intrinsics));
    default_realm.id()
}

fn install_test_uint8_array(agent: &mut Agent, elements: &[u8]) -> ObjectRef {
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let backing_store = agent
        .allocate_backing_store(elements.len())
        .expect("typed-array test backing store should allocate");
    for (index, byte) in elements.iter().copied().enumerate() {
        assert!(agent.backing_store_set_byte(backing_store, index, byte));
    }

    agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let buffer = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::ArrayBuffer)),
            AllocationLifetime::Default,
        );
        assert!(
            objects.install_array_buffer_object(buffer, ArrayBufferObjectData::new(backing_store))
        );

        let typed_array = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_element_capacity(elements.len())
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::TypedArray(
                    TypedArrayElementKind::Uint8,
                ))),
            AllocationLifetime::Default,
        );
        assert!(objects.install_typed_array_object(
            typed_array,
            TypedArrayObjectData::new(
                buffer,
                backing_store,
                0,
                elements.len(),
                TypedArrayElementKind::Uint8,
            ),
        ));
        for (index, byte) in elements.iter().copied().enumerate() {
            assert!(objects.init_element(
                &mut mutator,
                typed_array,
                u32::try_from(index).expect("typed-array index should fit into u32"),
                Value::from_smi(i32::from(byte)),
            ));
        }
        typed_array
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RecordedCall {
    callee: ObjectRef,
    this_value: Value,
    arguments: Vec<Value>,
    realm: RealmRef,
    environment: EnvironmentRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RecordedConstruct {
    callee: ObjectRef,
    new_target: ObjectRef,
    arguments: Vec<Value>,
    realm: RealmRef,
    environment: EnvironmentRef,
}

#[derive(Default)]
struct RecordingRegistry {
    last_call: Option<RecordedCall>,
    last_construct: Option<RecordedConstruct>,
}

impl NativeFunctionRegistry for RecordingRegistry {
    fn call(
        &mut self,
        _runtime: &mut ObjectRuntime,
        _heap: &mut lyng_js_gc::PrimitiveMutator<'_>,
        request: NativeCallRequest<'_>,
    ) -> InternalMethodResult<Value> {
        self.last_call = Some(RecordedCall {
            callee: request.callee(),
            this_value: request.this_value(),
            arguments: request.arguments().to_vec(),
            realm: request.realm(),
            environment: request.environment(),
        });
        Ok(Value::from_smi(77))
    }

    fn construct(
        &mut self,
        runtime: &mut ObjectRuntime,
        heap: &mut lyng_js_gc::PrimitiveMutator<'_>,
        request: NativeConstructRequest<'_>,
    ) -> InternalMethodResult<ObjectRef> {
        self.last_construct = Some(RecordedConstruct {
            callee: request.callee(),
            new_target: request.new_target(),
            arguments: request.arguments().to_vec(),
            realm: request.realm(),
            environment: request.environment(),
        });
        let root = runtime.root_shape(heap, None, AllocationLifetime::Default);
        Ok(runtime.alloc_object(
            heap,
            ObjectAllocation::ordinary(root).with_prototype(Some(request.new_target())),
            AllocationLifetime::Default,
        ))
    }
}

#[test]
fn ordinary_only_object_helpers_delegate_to_internal_methods() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let default_realm = agent.default_realm().expect("default realm should exist");
    let key = PropertyKey::from_atom(lyng_js_common::AtomId::from_raw(501));
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let root_shape = default_realm
            .root_shape()
            .expect("default realm should expose a root shape");
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });

    assert!(ordinary_create_data_property(
        agent,
        object,
        key,
        Value::from_smi(11),
        AllocationLifetime::Default,
    )
    .unwrap());
    assert!(ordinary_has_property(agent, object, key).unwrap());
    assert_eq!(
        ordinary_get(agent, object, key).unwrap(),
        Value::from_smi(11)
    );
    assert!(ordinary_set(
        agent,
        object,
        key,
        Value::from_smi(13),
        AllocationLifetime::Default,
    )
    .unwrap());
    assert_eq!(
        ordinary_get(agent, object, key).unwrap(),
        Value::from_smi(13)
    );
    assert!(ordinary_delete_property(agent, object, key).unwrap());
    assert!(!ordinary_has_property(agent, object, key).unwrap());
}

#[test]
fn typed_array_index_reads_observe_live_backing_store_bytes() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let typed_array = install_test_uint8_array(agent, &[1]);
    let typed_array_record = agent
        .objects()
        .typed_array(typed_array)
        .expect("test typed array should install its view record");
    assert!(agent.backing_store_set_byte(typed_array_record.backing_store(), 0, 7));

    let descriptor = ordinary_get_own_property(agent, typed_array, PropertyKey::Index(0))
        .expect("typed-array descriptor lookup should succeed")
        .expect("typed-array index should still exist");
    let value = ordinary_get(agent, typed_array, PropertyKey::Index(0))
        .expect("typed-array indexed get should succeed");

    assert_eq!(descriptor.value(), Some(Value::from_smi(7)));
    assert_eq!(value, Value::from_smi(7));
}

#[test]
fn typed_array_index_reads_hide_fixed_length_view_after_backing_store_shrink() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let typed_array = install_test_uint8_array(agent, &[1, 2, 3, 4]);
    let typed_array_record = agent
        .objects()
        .typed_array(typed_array)
        .expect("test typed array should install its view record");
    assert!(agent.backing_store_resize(typed_array_record.backing_store(), 2));

    assert!(!ordinary_has_property(agent, typed_array, PropertyKey::Index(0)).unwrap());
    assert!(
        ordinary_get_own_property(agent, typed_array, PropertyKey::Index(0))
            .unwrap()
            .is_none()
    );
    assert_eq!(
        ordinary_get(agent, typed_array, PropertyKey::Index(0)).unwrap(),
        Value::undefined()
    );
}

#[test]
fn call_and_construct_wrappers_delegate_to_function_runtime() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let default_realm = agent.default_realm().expect("default realm should exist");
    let entry = BuiltinFunctionId::from_raw(7).expect("builtin id");
    let function = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let root_shape = default_realm
            .root_shape()
            .expect("default realm should expose a root shape");
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::function(root_shape).with_cold_data(ObjectColdData::Function(
                FunctionObjectData::native(default_realm.id(), default_realm.global_env(), entry)
                    .with_this_mode(FunctionThisMode::Global)
                    .with_constructor_flags(FunctionConstructorFlags::constructible()),
            )),
            AllocationLifetime::Default,
        )
    });
    let mut registry = RecordingRegistry::default();

    let call_result = call(
        agent,
        function,
        Value::from_smi(3),
        &[Value::from_smi(4), Value::from_smi(5)],
        &mut registry,
    )
    .unwrap();
    let constructed =
        construct(agent, function, &[Value::from_smi(9)], None, &mut registry).unwrap();

    assert_eq!(call_result, Value::from_smi(77));
    assert_eq!(
        registry.last_call,
        Some(RecordedCall {
            callee: function,
            this_value: Value::from_smi(3),
            arguments: vec![Value::from_smi(4), Value::from_smi(5)],
            realm: default_realm.id(),
            environment: default_realm.global_env(),
        })
    );
    assert_eq!(
        registry.last_construct,
        Some(RecordedConstruct {
            callee: function,
            new_target: function,
            arguments: vec![Value::from_smi(9)],
            realm: default_realm.id(),
            environment: default_realm.global_env(),
        })
    );
    assert_eq!(
        agent
            .objects()
            .get_prototype_of(agent.heap().view(), constructed)
            .unwrap(),
        Some(function)
    );
}

#[test]
fn to_object_wraps_primitive_and_completion_builtin_families() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = install_test_wrapper_prototypes(agent);
    let string = agent.alloc_runtime_string("x", None, AllocationLifetime::Default);
    let bigint = agent.heap_mut().mutator().alloc_bigint(
        BigIntSign::NonNegative,
        &[17],
        AllocationLifetime::Default,
    );
    let symbol = agent.heap_mut().mutator().alloc_symbol(
        None,
        SymbolFlags::ordinary(),
        AllocationLifetime::Default,
    );
    let string_wrapper =
        to_object(agent, realm, Value::from_string_ref(string)).expect("String should wrap");
    let number_wrapper = to_object(agent, realm, Value::from_smi(41)).expect("Number should wrap");
    let boolean_wrapper =
        to_object(agent, realm, Value::from_bool(true)).expect("Boolean should wrap");
    let symbol_wrapper =
        to_object(agent, realm, Value::from_symbol_ref(symbol)).expect("Symbol should wrap");
    let bigint_wrapper =
        to_object(agent, realm, Value::from_bigint_ref(bigint)).expect("BigInt should wrap");

    assert_eq!(
        agent.objects().primitive_wrapper_kind(string_wrapper),
        Some(PrimitiveWrapperKind::String)
    );
    assert_eq!(
        agent.objects().primitive_wrapper_kind(number_wrapper),
        Some(PrimitiveWrapperKind::Number)
    );
    assert_eq!(
        agent.objects().primitive_wrapper_kind(boolean_wrapper),
        Some(PrimitiveWrapperKind::Boolean)
    );
    assert_eq!(
        agent
            .objects()
            .primitive_wrapper_value(agent.heap().view(), string_wrapper),
        Some(Value::from_string_ref(string))
    );
    assert_eq!(
        agent
            .objects()
            .primitive_wrapper_value(agent.heap().view(), number_wrapper),
        Some(Value::from_smi(41))
    );
    assert_eq!(
        agent
            .objects()
            .primitive_wrapper_value(agent.heap().view(), boolean_wrapper),
        Some(Value::from_bool(true))
    );
    assert_eq!(
        agent.objects().primitive_wrapper_kind(symbol_wrapper),
        Some(PrimitiveWrapperKind::Symbol)
    );
    assert_eq!(
        agent.objects().primitive_wrapper_kind(bigint_wrapper),
        Some(PrimitiveWrapperKind::BigInt)
    );
    assert_eq!(
        agent
            .objects()
            .primitive_wrapper_value(agent.heap().view(), symbol_wrapper),
        Some(Value::from_symbol_ref(symbol))
    );
    assert_eq!(
        agent
            .objects()
            .primitive_wrapper_value(agent.heap().view(), bigint_wrapper),
        Some(Value::from_bigint_ref(bigint))
    );
    assert!(to_object(agent, realm, Value::null()).is_err());
}

#[test]
fn primitive_wrapper_value_accepts_primitive_or_matching_wrapper() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = install_test_wrapper_prototypes(agent);
    let symbol = agent
        .well_known_symbol(WellKnownSymbolId::ToPrimitive)
        .expect("well-known symbol should exist");
    let symbol_wrapper = wrap_primitive_value(
        agent,
        realm,
        Value::from_symbol_ref(symbol),
        AllocationLifetime::Default,
    )
    .expect("Symbol wrapper should allocate");

    assert_eq!(
        require_primitive_wrapper_value(
            agent,
            Value::from_bool(false),
            PrimitiveWrapperKind::Boolean
        ),
        Ok(Value::from_bool(false))
    );
    assert_eq!(
        require_primitive_wrapper_value(
            agent,
            Value::from_symbol_ref(symbol),
            PrimitiveWrapperKind::Symbol
        ),
        Ok(Value::from_symbol_ref(symbol))
    );
    assert_eq!(
        require_primitive_wrapper_value(
            agent,
            Value::from_object_ref(symbol_wrapper),
            PrimitiveWrapperKind::Symbol
        ),
        Ok(Value::from_symbol_ref(symbol))
    );
    assert!(require_primitive_wrapper_value(
        agent,
        Value::from_object_ref(symbol_wrapper),
        PrimitiveWrapperKind::Boolean
    )
    .is_err());
}

#[test]
fn string_wrapper_install_preserves_length_and_elements() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = install_test_wrapper_prototypes(agent);
    let string = agent.alloc_runtime_string("abc", None, AllocationLifetime::Default);
    let wrapper = wrap_primitive_value(
        agent,
        realm,
        Value::from_string_ref(string),
        AllocationLifetime::Default,
    )
    .expect("String wrapper should allocate");

    let length = ordinary_get(
        agent,
        wrapper,
        PropertyKey::from_atom(WellKnownAtom::length.id()),
    )
    .unwrap();
    let first = ordinary_get(agent, wrapper, PropertyKey::Index(0)).unwrap();

    assert_eq!(
        agent.objects().primitive_wrapper_kind(wrapper),
        Some(PrimitiveWrapperKind::String)
    );
    assert_eq!(
        agent
            .objects()
            .primitive_wrapper_value(agent.heap().view(), wrapper),
        Some(Value::from_string_ref(string))
    );
    assert_eq!(length, Value::from_smi(3));
    assert_eq!(
        agent
            .heap()
            .view()
            .string(first.as_string_ref().unwrap())
            .unwrap()
            .cached_atom(),
        None
    );
}

struct WrapperToPrimitiveContext<'a> {
    agent: &'a mut Agent,
    to_string: ObjectRef,
    value_of: ObjectRef,
}

impl ToPrimitiveContext for WrapperToPrimitiveContext<'_> {
    type Error = AbruptCompletion;

    fn agent(&mut self) -> &mut Agent {
        self.agent
    }

    fn abrupt(&mut self, completion: AbruptCompletion) -> Self::Error {
        completion
    }

    fn type_error(&mut self) -> Self::Error {
        throw_type_error(self.agent)
    }

    fn get_property_value(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> Result<Value, Self::Error> {
        ordinary_get(self.agent, object, key)
    }

    fn require_callable_object(&mut self, value: Value) -> Result<ObjectRef, Self::Error> {
        let Some(object) = value.as_object_ref() else {
            return Err(throw_type_error(self.agent));
        };
        if self.agent.objects().function_data(object).is_some() {
            Ok(object)
        } else {
            Err(throw_type_error(self.agent))
        }
    }

    fn call_to_completion(
        &mut self,
        callee_object: ObjectRef,
        this_value: Value,
        _arguments: &[Value],
    ) -> Result<Value, Self::Error> {
        if callee_object == self.value_of {
            return Ok(this_value);
        }
        if callee_object == self.to_string {
            let string = self.agent.alloc_runtime_string(
                "object-fallback",
                None,
                AllocationLifetime::Default,
            );
            return Ok(Value::from_string_ref(string));
        }
        panic!("unexpected wrapper fallback method")
    }
}

fn install_default_object_to_primitive_methods(
    agent: &mut Agent,
    realm: RealmRef,
) -> (ObjectRef, ObjectRef) {
    let realm_record = agent.realm(realm).expect("realm should exist");
    let root_shape = realm_record
        .root_shape()
        .expect("realm should expose a root shape");
    let global_env = realm_record.global_env();
    let object_prototype = realm_record
        .intrinsics()
        .object_prototype()
        .expect("wrapper test realm should expose Object.prototype");
    let (to_string, value_of) = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let to_string = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::function(root_shape).with_cold_data(ObjectColdData::Function(
                FunctionObjectData::native(
                    realm,
                    global_env,
                    BuiltinFunctionId::from_raw(40).expect("builtin id"),
                ),
            )),
            AllocationLifetime::Default,
        );
        let value_of = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::function(root_shape).with_cold_data(ObjectColdData::Function(
                FunctionObjectData::native(
                    realm,
                    global_env,
                    BuiltinFunctionId::from_raw(41).expect("builtin id"),
                ),
            )),
            AllocationLifetime::Default,
        );
        (to_string, value_of)
    });
    assert!(ordinary_create_data_property(
        agent,
        object_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        Value::from_object_ref(to_string),
        AllocationLifetime::Default,
    )
    .unwrap());
    assert!(ordinary_create_data_property(
        agent,
        object_prototype,
        PropertyKey::from_atom(WellKnownAtom::valueOf.id()),
        Value::from_object_ref(value_of),
        AllocationLifetime::Default,
    )
    .unwrap());
    (to_string, value_of)
}

#[test]
fn to_primitive_calls_inherited_object_prototype_methods_for_wrappers() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = install_test_wrapper_prototypes(agent);
    let (to_string, value_of) = install_default_object_to_primitive_methods(agent, realm);
    let string = agent.alloc_runtime_string("abc", None, AllocationLifetime::Default);
    let number_wrapper = wrap_primitive_value(
        agent,
        realm,
        Value::from_smi(7),
        AllocationLifetime::Default,
    )
    .expect("Number wrapper should allocate");
    let string_wrapper = wrap_primitive_value(
        agent,
        realm,
        Value::from_string_ref(string),
        AllocationLifetime::Default,
    )
    .expect("String wrapper should allocate");
    let mut context = WrapperToPrimitiveContext {
        agent,
        to_string,
        value_of,
    };

    let number_text = to_primitive(
        &mut context,
        Value::from_object_ref(number_wrapper),
        ToPrimitiveHint::Number,
    )
    .expect("number wrapper should use inherited Object.prototype methods");
    let string_text = to_primitive(
        &mut context,
        Value::from_object_ref(string_wrapper),
        ToPrimitiveHint::String,
    )
    .expect("string wrapper should use inherited Object.prototype methods");
    assert_eq!(
        context
            .agent
            .heap()
            .view()
            .string_view(number_text.as_string_ref().unwrap())
            .unwrap()
            .latin1_bytes(),
        Some(&b"object-fallback"[..])
    );
    assert_eq!(
        context
            .agent
            .heap()
            .view()
            .string_view(string_text.as_string_ref().unwrap())
            .unwrap()
            .latin1_bytes(),
        Some(&b"object-fallback"[..])
    );
    assert_ne!(number_text, Value::from_smi(7));
    assert_ne!(string_text, Value::from_string_ref(string));
}

#[test]
fn require_temporal_object_returns_installed_payload_for_matching_kind() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let default_realm = agent.default_realm().expect("default realm should exist");
    let instant = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let root_shape = default_realm
            .root_shape()
            .expect("default realm should expose a root shape");
        let instant = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_cold_data(ObjectColdData::Ordinary(
                OrdinaryObjectData::Temporal(TemporalObjectKind::Instant),
            )),
            AllocationLifetime::Default,
        );
        assert!(objects.install_temporal_object(
            instant,
            TemporalObjectData::Instant(TemporalInstantObjectData::new(77)),
        ));
        instant
    });

    let payload = require_temporal_object(
        agent,
        Value::from_object_ref(instant),
        TemporalObjectKind::Instant,
    )
    .expect("Temporal.Instant should expose its typed payload");

    assert_eq!(
        payload,
        TemporalObjectData::Instant(TemporalInstantObjectData::new(77))
    );
}

#[test]
fn require_temporal_object_rejects_plain_objects_and_wrong_temporal_kind() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let default_realm = agent.default_realm().expect("default realm should exist");
    let (plain, instant) = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let root_shape = default_realm
            .root_shape()
            .expect("default realm should expose a root shape");
        let plain = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        );
        let instant = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_cold_data(ObjectColdData::Ordinary(
                OrdinaryObjectData::Temporal(TemporalObjectKind::Instant),
            )),
            AllocationLifetime::Default,
        );
        assert!(objects.install_temporal_object(
            instant,
            TemporalObjectData::Instant(TemporalInstantObjectData::new(77)),
        ));
        (plain, instant)
    });

    assert!(require_temporal_object(
        agent,
        Value::from_object_ref(plain),
        TemporalObjectKind::Instant,
    )
    .is_err());
    assert!(require_temporal_object(
        agent,
        Value::from_object_ref(instant),
        TemporalObjectKind::ZonedDateTime,
    )
    .is_err());
}

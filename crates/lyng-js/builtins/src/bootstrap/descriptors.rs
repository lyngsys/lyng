use super::*;

pub(crate) fn install_descriptor_tables(
    agent: &mut Agent,
    builtin_cache: &mut BuiltinCache,
    realm: RealmRef,
    descriptor_tables: &[BuiltinDescriptorTable<'_>],
) -> Result<(), BuiltinBootstrapError> {
    for table in descriptor_tables {
        let target = resolve_install_target(agent, realm, table.target())?;
        for descriptor in table.descriptors() {
            install_descriptor(agent, builtin_cache, realm, target, *descriptor)?;
        }
    }
    Ok(())
}

fn resolve_install_target(
    agent: &Agent,
    realm: RealmRef,
    target: BuiltinInstallTarget,
) -> Result<ObjectRef, BuiltinBootstrapError> {
    match target {
        BuiltinInstallTarget::GlobalObject => agent
            .realm(realm)
            .map(RealmRecord::global_object)
            .ok_or(BuiltinBootstrapError::MissingRealm(realm)),
        BuiltinInstallTarget::Intrinsic(intrinsic) => resolve_intrinsic(agent, realm, intrinsic),
        BuiltinInstallTarget::Object(object) => Ok(object),
    }
}

fn resolve_intrinsic(
    agent: &Agent,
    realm: RealmRef,
    intrinsic: BuiltinIntrinsic,
) -> Result<ObjectRef, BuiltinBootstrapError> {
    let intrinsics = agent
        .realm(realm)
        .ok_or(BuiltinBootstrapError::MissingRealm(realm))?
        .intrinsics();
    let resolved = match intrinsic {
        BuiltinIntrinsic::Object => intrinsics.object(),
        BuiltinIntrinsic::ObjectPrototype => intrinsics.object_prototype(),
        BuiltinIntrinsic::Function => intrinsics.function(),
        BuiltinIntrinsic::FunctionPrototype => intrinsics.function_prototype(),
        BuiltinIntrinsic::AsyncFunction => intrinsics.async_function(),
        BuiltinIntrinsic::AsyncFunctionPrototype => intrinsics.async_function_prototype(),
        BuiltinIntrinsic::AsyncGeneratorFunction => intrinsics.async_generator_function(),
        BuiltinIntrinsic::AsyncGeneratorFunctionPrototype => {
            intrinsics.async_generator_function_prototype()
        }
        BuiltinIntrinsic::AsyncGeneratorPrototype => intrinsics.async_generator_prototype(),
        BuiltinIntrinsic::GeneratorFunction => intrinsics.generator_function(),
        BuiltinIntrinsic::GeneratorFunctionPrototype => intrinsics.generator_function_prototype(),
        BuiltinIntrinsic::GeneratorPrototype => intrinsics.generator_prototype(),
        BuiltinIntrinsic::Array => intrinsics.array(),
        BuiltinIntrinsic::ArrayPrototype => intrinsics.array_prototype(),
        BuiltinIntrinsic::Map => intrinsics.map(),
        BuiltinIntrinsic::MapPrototype => intrinsics.map_prototype(),
        BuiltinIntrinsic::MapIteratorPrototype => intrinsics.map_iterator_prototype(),
        BuiltinIntrinsic::Set => intrinsics.set(),
        BuiltinIntrinsic::SetPrototype => intrinsics.set_prototype(),
        BuiltinIntrinsic::SetIteratorPrototype => intrinsics.set_iterator_prototype(),
        BuiltinIntrinsic::WeakMap => intrinsics.weak_map(),
        BuiltinIntrinsic::WeakMapPrototype => intrinsics.weak_map_prototype(),
        BuiltinIntrinsic::WeakSet => intrinsics.weak_set(),
        BuiltinIntrinsic::WeakSetPrototype => intrinsics.weak_set_prototype(),
        BuiltinIntrinsic::WeakRef => intrinsics.weak_ref(),
        BuiltinIntrinsic::WeakRefPrototype => intrinsics.weak_ref_prototype(),
        BuiltinIntrinsic::FinalizationRegistry => intrinsics.finalization_registry(),
        BuiltinIntrinsic::FinalizationRegistryPrototype => {
            intrinsics.finalization_registry_prototype()
        }
        BuiltinIntrinsic::ArrayBuffer => intrinsics.array_buffer(),
        BuiltinIntrinsic::ArrayBufferPrototype => intrinsics.array_buffer_prototype(),
        BuiltinIntrinsic::SharedArrayBuffer => intrinsics.shared_array_buffer(),
        BuiltinIntrinsic::SharedArrayBufferPrototype => intrinsics.shared_array_buffer_prototype(),
        BuiltinIntrinsic::DataView => intrinsics.data_view(),
        BuiltinIntrinsic::DataViewPrototype => intrinsics.data_view_prototype(),
        BuiltinIntrinsic::Atomics => intrinsics.atomics(),
        BuiltinIntrinsic::TypedArray => intrinsics.typed_array(),
        BuiltinIntrinsic::TypedArrayPrototype => intrinsics.typed_array_prototype(),
        BuiltinIntrinsic::Int8Array => intrinsics.int8_array(),
        BuiltinIntrinsic::Int8ArrayPrototype => intrinsics.int8_array_prototype(),
        BuiltinIntrinsic::Int16Array => intrinsics.int16_array(),
        BuiltinIntrinsic::Int16ArrayPrototype => intrinsics.int16_array_prototype(),
        BuiltinIntrinsic::Int32Array => intrinsics.int32_array(),
        BuiltinIntrinsic::Int32ArrayPrototype => intrinsics.int32_array_prototype(),
        BuiltinIntrinsic::Float32Array => intrinsics.float32_array(),
        BuiltinIntrinsic::Float32ArrayPrototype => intrinsics.float32_array_prototype(),
        BuiltinIntrinsic::Float64Array => intrinsics.float64_array(),
        BuiltinIntrinsic::Float64ArrayPrototype => intrinsics.float64_array_prototype(),
        BuiltinIntrinsic::BigInt64Array => intrinsics.big_int64_array(),
        BuiltinIntrinsic::BigInt64ArrayPrototype => intrinsics.big_int64_array_prototype(),
        BuiltinIntrinsic::BigUint64Array => intrinsics.big_uint64_array(),
        BuiltinIntrinsic::BigUint64ArrayPrototype => intrinsics.big_uint64_array_prototype(),
        BuiltinIntrinsic::Uint32Array => intrinsics.uint32_array(),
        BuiltinIntrinsic::Uint32ArrayPrototype => intrinsics.uint32_array_prototype(),
        BuiltinIntrinsic::Uint16Array => intrinsics.uint16_array(),
        BuiltinIntrinsic::Uint16ArrayPrototype => intrinsics.uint16_array_prototype(),
        BuiltinIntrinsic::Uint8ClampedArray => intrinsics.uint8_clamped_array(),
        BuiltinIntrinsic::Uint8ClampedArrayPrototype => intrinsics.uint8_clamped_array_prototype(),
        BuiltinIntrinsic::Uint8Array => intrinsics.uint8_array(),
        BuiltinIntrinsic::Uint8ArrayPrototype => intrinsics.uint8_array_prototype(),
        BuiltinIntrinsic::Iterator => intrinsics.iterator(),
        BuiltinIntrinsic::IteratorPrototype => intrinsics.iterator_prototype(),
        BuiltinIntrinsic::AsyncIteratorPrototype => intrinsics.async_iterator_prototype(),
        BuiltinIntrinsic::AsyncFromSyncIteratorPrototype => {
            intrinsics.async_from_sync_iterator_prototype()
        }
        BuiltinIntrinsic::IteratorHelperPrototype => intrinsics.iterator_helper_prototype(),
        BuiltinIntrinsic::ArrayIteratorPrototype => intrinsics.array_iterator_prototype(),
        BuiltinIntrinsic::String => intrinsics.string(),
        BuiltinIntrinsic::StringPrototype => intrinsics.string_prototype(),
        BuiltinIntrinsic::StringIteratorPrototype => intrinsics.string_iterator_prototype(),
        BuiltinIntrinsic::RegExp => intrinsics.regexp(),
        BuiltinIntrinsic::RegExpPrototype => intrinsics.regexp_prototype(),
        BuiltinIntrinsic::RegExpStringIteratorPrototype => {
            intrinsics.regexp_string_iterator_prototype()
        }
        BuiltinIntrinsic::Date => intrinsics.date(),
        BuiltinIntrinsic::DatePrototype => intrinsics.date_prototype(),
        BuiltinIntrinsic::Number => intrinsics.number(),
        BuiltinIntrinsic::NumberPrototype => intrinsics.number_prototype(),
        BuiltinIntrinsic::Math => intrinsics.math(),
        BuiltinIntrinsic::BigInt => intrinsics.bigint(),
        BuiltinIntrinsic::BigIntPrototype => intrinsics.bigint_prototype(),
        BuiltinIntrinsic::Boolean => intrinsics.boolean(),
        BuiltinIntrinsic::BooleanPrototype => intrinsics.boolean_prototype(),
        BuiltinIntrinsic::Symbol => intrinsics.symbol(),
        BuiltinIntrinsic::SymbolPrototype => intrinsics.symbol_prototype(),
        BuiltinIntrinsic::Json => intrinsics.json(),
        BuiltinIntrinsic::Reflect => intrinsics.reflect(),
        BuiltinIntrinsic::Proxy => intrinsics.proxy(),
        BuiltinIntrinsic::Error => intrinsics.error(),
        BuiltinIntrinsic::ErrorPrototype => intrinsics.error_prototype(),
        BuiltinIntrinsic::EvalError => intrinsics.eval_error(),
        BuiltinIntrinsic::EvalErrorPrototype => intrinsics.eval_error_prototype(),
        BuiltinIntrinsic::RangeError => intrinsics.range_error(),
        BuiltinIntrinsic::RangeErrorPrototype => intrinsics.range_error_prototype(),
        BuiltinIntrinsic::ReferenceError => intrinsics.reference_error(),
        BuiltinIntrinsic::ReferenceErrorPrototype => intrinsics.reference_error_prototype(),
        BuiltinIntrinsic::SyntaxError => intrinsics.syntax_error(),
        BuiltinIntrinsic::SyntaxErrorPrototype => intrinsics.syntax_error_prototype(),
        BuiltinIntrinsic::TypeError => intrinsics.type_error(),
        BuiltinIntrinsic::TypeErrorPrototype => intrinsics.type_error_prototype(),
        BuiltinIntrinsic::UriError => intrinsics.uri_error(),
        BuiltinIntrinsic::UriErrorPrototype => intrinsics.uri_error_prototype(),
        BuiltinIntrinsic::AggregateError => intrinsics.aggregate_error(),
        BuiltinIntrinsic::AggregateErrorPrototype => intrinsics.aggregate_error_prototype(),
        BuiltinIntrinsic::SuppressedError => intrinsics.suppressed_error(),
        BuiltinIntrinsic::SuppressedErrorPrototype => intrinsics.suppressed_error_prototype(),
        BuiltinIntrinsic::Promise => intrinsics.promise(),
        BuiltinIntrinsic::PromisePrototype => intrinsics.promise_prototype(),
        BuiltinIntrinsic::DisposableStack => intrinsics.disposable_stack(),
        BuiltinIntrinsic::DisposableStackPrototype => intrinsics.disposable_stack_prototype(),
        BuiltinIntrinsic::AsyncDisposableStack => intrinsics.async_disposable_stack(),
        BuiltinIntrinsic::AsyncDisposableStackPrototype => {
            intrinsics.async_disposable_stack_prototype()
        }
        BuiltinIntrinsic::ThrowTypeError => intrinsics.throw_type_error(),
    };
    resolved.ok_or(BuiltinBootstrapError::MissingIntrinsic(intrinsic, realm))
}

fn install_descriptor(
    agent: &mut Agent,
    builtin_cache: &mut BuiltinCache,
    realm: RealmRef,
    target: ObjectRef,
    descriptor: BuiltinPropertyDescriptor,
) -> Result<(), BuiltinBootstrapError> {
    let key = resolve_property_key(agent, descriptor.key())?;
    let attributes = descriptor.attributes();
    let mut property_descriptor = PropertyDescriptor::new();
    match descriptor.value() {
        BuiltinPropertyValueSpec::Data(value) => {
            property_descriptor.set_value(value);
            property_descriptor.set_writable(attributes.writable());
        }
        BuiltinPropertyValueSpec::BuiltinFunction(entry) => {
            property_descriptor.set_value(resolve_builtin_function_value(
                agent,
                builtin_cache,
                realm,
                entry,
            )?);
            property_descriptor.set_writable(attributes.writable());
        }
        BuiltinPropertyValueSpec::Accessor { get, set } => {
            property_descriptor.set_getter(resolve_accessor_builtin_value(
                agent,
                builtin_cache,
                realm,
                get,
            )?);
            property_descriptor.set_setter(resolve_accessor_builtin_value(
                agent,
                builtin_cache,
                realm,
                set,
            )?);
        }
    }
    property_descriptor.set_enumerable(attributes.enumerable());
    property_descriptor.set_configurable(attributes.configurable());
    let defined = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.define_own_property(
            &mut mutator,
            target,
            key,
            property_descriptor,
            AllocationLifetime::Default,
        )
    });
    if matches!(defined, Ok(true)) {
        return Ok(());
    }

    Err(BuiltinBootstrapError::DefinePropertyRejected {
        target,
        key: descriptor.key(),
    })
}

fn resolve_property_key(
    agent: &Agent,
    key: BuiltinPropertyKeySpec,
) -> Result<PropertyKey, BuiltinBootstrapError> {
    match key {
        BuiltinPropertyKeySpec::Index(index) => Ok(PropertyKey::Index(index)),
        BuiltinPropertyKeySpec::Atom(atom) => Ok(PropertyKey::from_atom(atom)),
        BuiltinPropertyKeySpec::WellKnownSymbol(symbol_id) => agent
            .well_known_symbol(symbol_id)
            .map(PropertyKey::from_symbol)
            .ok_or(BuiltinBootstrapError::MissingWellKnownSymbol(symbol_id)),
    }
}

fn resolve_builtin_function_value(
    agent: &mut Agent,
    builtin_cache: &mut BuiltinCache,
    realm: RealmRef,
    entry: lyng_js_types::BuiltinFunctionId,
) -> Result<Value, BuiltinBootstrapError> {
    builtin_cache
        .builtin_constant(agent, realm, entry)
        .ok_or(BuiltinBootstrapError::MissingBuiltinFunction(entry, realm))
}

fn resolve_accessor_builtin_value(
    agent: &mut Agent,
    builtin_cache: &mut BuiltinCache,
    realm: RealmRef,
    entry: Option<lyng_js_types::BuiltinFunctionId>,
) -> Result<Value, BuiltinBootstrapError> {
    let Some(entry) = entry else {
        return Ok(Value::undefined());
    };
    resolve_builtin_function_value(agent, builtin_cache, realm, entry)
}

use super::{
    length_value_u64, normalize_relative_index_u64, range_error, to_index_for_builtin,
    to_integer_or_infinity_for_builtin, type_error, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_common::WellKnownAtom;
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::{
    ArrayBufferObjectData, ObjectAllocation, ObjectColdData, OrdinaryObjectData,
};
use lyng_js_types::{
    BuiltinFunctionId, ObjectRef, PropertyKey, RealmRef, Value, WellKnownSymbolId,
};

pub(super) fn dispatch_buffer_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::super::array_buffer_builtin() {
        return array_buffer_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_buffer_is_view_builtin() {
        return array_buffer_is_view_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_buffer_byte_length_getter_builtin() {
        return array_buffer_byte_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_buffer_detached_getter_builtin() {
        return array_buffer_detached_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_buffer_max_byte_length_getter_builtin() {
        return array_buffer_max_byte_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_buffer_resizable_getter_builtin() {
        return array_buffer_resizable_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_buffer_slice_builtin() {
        return array_buffer_slice_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_buffer_resize_builtin() {
        return array_buffer_resize_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_buffer_transfer_builtin() {
        return array_buffer_transfer_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_buffer_transfer_to_fixed_length_builtin() {
        return array_buffer_transfer_to_fixed_length_builtin(context, invocation).map(Some);
    }
    if entry == super::super::shared_array_buffer_builtin() {
        return shared_array_buffer_builtin(context, invocation).map(Some);
    }
    if entry == super::super::shared_array_buffer_byte_length_getter_builtin() {
        return shared_array_buffer_byte_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::super::shared_array_buffer_slice_builtin() {
        return shared_array_buffer_slice_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn allocate_array_buffer_family_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
    data: ArrayBufferObjectData,
    kind: OrdinaryObjectData,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let root_shape = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(lyng_js_env::RealmRecord::root_shape)
    }
    .ok_or_else(|| type_error(cx))?;
    Ok(cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_prototype(Some(prototype))
                .with_cold_data(ObjectColdData::Ordinary(kind)),
            AllocationLifetime::Default,
        );
        let installed = objects.install_array_buffer_object(object, data);
        debug_assert!(
            installed,
            "fresh buffer object should install its backing store"
        );
        object
    }))
}

pub(super) fn allocate_array_buffer_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
    backing_store: lyng_js_types::BackingStoreRef,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    allocate_array_buffer_object_with_data(
        cx,
        realm,
        prototype,
        ArrayBufferObjectData::new(backing_store),
    )
}

fn allocate_array_buffer_object_with_data<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
    data: ArrayBufferObjectData,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    allocate_array_buffer_family_object(cx, realm, prototype, data, OrdinaryObjectData::ArrayBuffer)
}

fn allocate_shared_array_buffer_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
    backing_store: lyng_js_types::BackingStoreRef,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    allocate_array_buffer_family_object(
        cx,
        realm,
        prototype,
        ArrayBufferObjectData::new(backing_store),
        OrdinaryObjectData::SharedArrayBuffer,
    )
}

fn array_buffer_this_store<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<lyng_js_types::BackingStoreRef, Cx::Error> {
    Ok(array_buffer_this_data(cx, value)?.1.backing_store())
}

fn array_buffer_this_data<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<(ObjectRef, ArrayBufferObjectData), Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if !cx.agent().objects().is_array_buffer_object(object) {
        return Err(type_error(cx));
    }
    let buffer = cx
        .agent()
        .objects()
        .array_buffer(object)
        .ok_or_else(|| type_error(cx))?;
    Ok((object, buffer))
}

fn shared_array_buffer_this_store<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<lyng_js_types::BackingStoreRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if !cx.agent().objects().is_shared_array_buffer_object(object) {
        return Err(type_error(cx));
    }
    cx.agent()
        .objects()
        .array_buffer(object)
        .map(ArrayBufferObjectData::backing_store)
        .ok_or_else(|| type_error(cx))
}

fn array_buffer_family_default_constructor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    shared: bool,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    cx.agent()
        .realm(realm)
        .and_then(|realm| {
            if shared {
                realm.intrinsics().shared_array_buffer()
            } else {
                realm.intrinsics().array_buffer()
            }
        })
        .ok_or_else(|| type_error(cx))
}

fn array_buffer_family_species_constructor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    array_buffer: ObjectRef,
    shared: bool,
) -> Result<ObjectRef, Cx::Error> {
    let default_constructor = array_buffer_family_default_constructor(cx, shared)?;
    let constructor = cx.get_property_value(
        Value::from_object_ref(array_buffer),
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
    )?;
    if constructor.is_undefined() {
        return Ok(default_constructor);
    }
    let constructor = constructor.as_object_ref().ok_or_else(|| type_error(cx))?;
    let species_symbol = cx
        .agent()
        .well_known_symbol(WellKnownSymbolId::Species)
        .ok_or_else(|| type_error(cx))?;
    let species = cx.get_property_value(
        Value::from_object_ref(constructor),
        PropertyKey::from_symbol(species_symbol),
    )?;
    if species.is_undefined() || species.is_null() {
        return Ok(default_constructor);
    }
    let species = species.as_object_ref().ok_or_else(|| type_error(cx))?;
    if !cx.agent().objects().is_constructor(species) {
        return Err(type_error(cx));
    }
    Ok(species)
}

fn array_buffer_species_constructor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    array_buffer: ObjectRef,
) -> Result<ObjectRef, Cx::Error> {
    array_buffer_family_species_constructor(cx, array_buffer, false)
}

fn shared_array_buffer_species_constructor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    array_buffer: ObjectRef,
) -> Result<ObjectRef, Cx::Error> {
    array_buffer_family_species_constructor(cx, array_buffer, true)
}

fn array_buffer_max_byte_length_option<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Option<Value>,
) -> Result<Option<usize>, Cx::Error> {
    let Some(options) = value else {
        return Ok(None);
    };
    if options.is_undefined() {
        return Ok(None);
    }
    let Some(options_object) = options.as_object_ref() else {
        return Ok(None);
    };
    let key = {
        let agent = cx.agent();
        PropertyKey::from_atom(agent.atoms_mut().intern_collectible("maxByteLength"))
    };
    let max = cx.get_property_value(Value::from_object_ref(options_object), key)?;
    if max.is_undefined() {
        return Ok(None);
    }
    let max = to_index_for_builtin(cx, max)?;
    Ok(Some(usize::try_from(max).map_err(|_| range_error(cx))?))
}

fn array_buffer_family_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    shared: bool,
) -> Result<Value, Cx::Error> {
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let byte_length = to_index_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let byte_length = usize::try_from(byte_length).map_err(|_| range_error(cx))?;
    let max_byte_length = if shared {
        None
    } else {
        array_buffer_max_byte_length_option(cx, invocation.arguments().get(1).copied())?
    };
    if max_byte_length.is_some_and(|max| byte_length > max) {
        return Err(range_error(cx));
    }
    if max_byte_length.is_some_and(|max| max > cx.agent().backing_store_allocation_limit()) {
        return Err(range_error(cx));
    }
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent.realm(realm).and_then(|record| {
            if shared {
                record.intrinsics().shared_array_buffer_prototype()
            } else {
                record.intrinsics().array_buffer_prototype()
            }
        })
    }
    .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let backing_store = {
        let agent = cx.agent();
        if shared {
            agent.allocate_shared_backing_store(byte_length)
        } else {
            agent.allocate_backing_store(byte_length)
        }
    }
    .ok_or_else(|| range_error(cx))?;
    let object = if shared {
        allocate_shared_array_buffer_object(cx, realm, prototype, backing_store)?
    } else if let Some(max_byte_length) = max_byte_length {
        allocate_array_buffer_object_with_data(
            cx,
            realm,
            prototype,
            ArrayBufferObjectData::new_resizable(backing_store, max_byte_length),
        )?
    } else {
        allocate_array_buffer_object(cx, realm, prototype, backing_store)?
    };
    Ok(Value::from_object_ref(object))
}

fn array_buffer_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    array_buffer_family_builtin(cx, invocation, false)
}

fn shared_array_buffer_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    array_buffer_family_builtin(cx, invocation, true)
}

fn array_buffer_is_view_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let is_view = value.as_object_ref().is_some_and(|object| {
        let objects = cx.agent().objects();
        objects.is_data_view_object(object) || objects.is_typed_array_object(object)
    });
    Ok(Value::from_bool(is_view))
}

fn array_buffer_byte_length_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let store = array_buffer_this_store(cx, invocation.this_value())?;
    shared_buffer_byte_length_value(cx, store)
}

fn array_buffer_detached_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let store = array_buffer_this_store(cx, invocation.this_value())?;
    let detached = cx
        .agent()
        .backing_store_is_detached(store)
        .ok_or_else(|| type_error(cx))?;
    Ok(Value::from_bool(detached))
}

fn array_buffer_max_byte_length_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (_, buffer) = array_buffer_this_data(cx, invocation.this_value())?;
    if cx
        .agent()
        .backing_store_is_detached(buffer.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Ok(Value::from_smi(0));
    }
    let byte_length = match buffer.max_byte_length() {
        Some(max_byte_length) => max_byte_length,
        None => cx
            .agent()
            .backing_store_byte_length(buffer.backing_store())
            .ok_or_else(|| type_error(cx))?,
    };
    Ok(length_value_u64(
        u64::try_from(byte_length).unwrap_or(u64::MAX),
    ))
}

fn array_buffer_resizable_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (_, buffer) = array_buffer_this_data(cx, invocation.this_value())?;
    Ok(Value::from_bool(buffer.is_resizable()))
}

fn shared_array_buffer_byte_length_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let store = shared_array_buffer_this_store(cx, invocation.this_value())?;
    shared_buffer_byte_length_value(cx, store)
}

fn shared_buffer_byte_length_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    store: lyng_js_types::BackingStoreRef,
) -> Result<Value, Cx::Error> {
    let byte_length = cx
        .agent()
        .backing_store_byte_length(store)
        .ok_or_else(|| type_error(cx))?;
    Ok(length_value_u64(
        u64::try_from(byte_length).unwrap_or(u64::MAX),
    ))
}

fn refresh_length_tracking_views<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    buffer_object: ObjectRef,
    byte_length: usize,
) {
    let views = {
        let agent = cx.agent();
        agent
            .objects()
            .typed_array_views_of_buffer(agent.heap().view(), buffer_object)
    };
    for (view, record) in views {
        if !record.is_length_tracking() {
            continue;
        }
        let element_size = record.kind().bytes_per_element();
        let length = byte_length
            .checked_sub(record.byte_offset())
            .map_or(0, |remaining| remaining / element_size);
        let _ = cx
            .agent()
            .objects_mut()
            .install_typed_array_object(view, record.with_length(length));
    }
}

fn array_buffer_resize_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let buffer = cx
        .agent()
        .objects()
        .array_buffer(object)
        .ok_or_else(|| type_error(cx))?;
    let max_byte_length = buffer.max_byte_length().ok_or_else(|| type_error(cx))?;
    let new_byte_length = to_index_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let new_byte_length = usize::try_from(new_byte_length).map_err(|_| range_error(cx))?;
    if new_byte_length > max_byte_length {
        return Err(range_error(cx));
    }
    if cx
        .agent()
        .backing_store_is_detached(buffer.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    if !cx
        .agent()
        .backing_store_resize(buffer.backing_store(), new_byte_length)
    {
        return Err(type_error(cx));
    }
    refresh_length_tracking_views(cx, object, new_byte_length);
    Ok(Value::undefined())
}

fn array_buffer_transfer_family_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    to_fixed_length: bool,
) -> Result<Value, Cx::Error> {
    let (source_object, source_buffer) = array_buffer_this_data(cx, invocation.this_value())?;
    let source_store = source_buffer.backing_store();
    if cx
        .agent()
        .backing_store_is_detached(source_store)
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    let source_length = cx
        .agent()
        .backing_store_byte_length(source_store)
        .ok_or_else(|| type_error(cx))?;
    let new_byte_length = match invocation.arguments().first().copied() {
        None => source_length,
        Some(value) if value.is_undefined() => source_length,
        Some(value) => {
            let index = to_index_for_builtin(cx, value)?;
            usize::try_from(index).map_err(|_| range_error(cx))?
        }
    };
    let new_max_byte_length = if to_fixed_length {
        None
    } else {
        source_buffer.max_byte_length()
    };
    if new_max_byte_length.is_some_and(|max_byte_length| new_byte_length > max_byte_length) {
        return Err(range_error(cx));
    }
    let new_store = cx
        .agent()
        .allocate_backing_store(new_byte_length)
        .ok_or_else(|| range_error(cx))?;
    let copy_length = source_length.min(new_byte_length);
    for index in 0..copy_length {
        let byte = cx
            .agent()
            .backing_store_get_byte(source_store, index)
            .ok_or_else(|| type_error(cx))?;
        if !cx.agent().backing_store_set_byte(new_store, index, byte) {
            return Err(type_error(cx));
        }
    }
    if !cx.agent().detach_backing_store(source_store) {
        return Err(type_error(cx));
    }
    refresh_length_tracking_views(cx, source_object, 0);

    let realm = cx.builtin_realm();
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().array_buffer_prototype())
        .ok_or_else(|| type_error(cx))?;
    let data = match new_max_byte_length {
        Some(max_byte_length) => ArrayBufferObjectData::new_resizable(new_store, max_byte_length),
        None => ArrayBufferObjectData::new(new_store),
    };
    let object = allocate_array_buffer_object_with_data(cx, realm, prototype, data)?;
    Ok(Value::from_object_ref(object))
}

fn array_buffer_transfer_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    array_buffer_transfer_family_builtin(cx, invocation, false)
}

fn array_buffer_transfer_to_fixed_length_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    array_buffer_transfer_family_builtin(cx, invocation, true)
}

fn array_buffer_family_slice_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    shared: bool,
) -> Result<Value, Cx::Error> {
    let source_object = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let store = if shared {
        shared_array_buffer_this_store(cx, invocation.this_value())?
    } else {
        array_buffer_this_store(cx, invocation.this_value())?
    };
    if !shared
        && cx
            .agent()
            .backing_store_is_detached(store)
            .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    let source_length = cx
        .agent()
        .backing_store_byte_length(store)
        .ok_or_else(|| type_error(cx))?;
    let source_length = u64::try_from(source_length).unwrap_or(u64::MAX);
    let start = normalize_relative_index_u64(
        source_length,
        to_integer_or_infinity_for_builtin(
            cx,
            invocation
                .arguments()
                .first()
                .copied()
                .unwrap_or(Value::undefined()),
        )?,
    );
    let end = match invocation.arguments().get(1).copied() {
        Some(value) if value.is_undefined() => source_length,
        Some(value) => normalize_relative_index_u64(
            source_length,
            to_integer_or_infinity_for_builtin(cx, value)?,
        ),
        None => source_length,
    };
    let copy_end = end.max(start);
    let start_index = usize::try_from(start).map_err(|_| range_error(cx))?;
    let end_index = usize::try_from(copy_end).map_err(|_| range_error(cx))?;
    let new_length = end_index.saturating_sub(start_index);
    let constructor = if shared {
        shared_array_buffer_species_constructor(cx, source_object)?
    } else {
        array_buffer_species_constructor(cx, source_object)?
    };
    let result = cx.construct_to_completion(
        constructor,
        &[length_value_u64(
            u64::try_from(new_length).unwrap_or(u64::MAX),
        )],
        Some(constructor),
    )?;
    if result == source_object {
        return Err(type_error(cx));
    }
    let new_store = cx
        .agent()
        .objects()
        .array_buffer(result)
        .map(ArrayBufferObjectData::backing_store)
        .ok_or_else(|| type_error(cx))?;
    if !shared
        && cx
            .agent()
            .backing_store_is_detached(new_store)
            .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    if shared
        && !cx
            .agent()
            .backing_store_is_shared(new_store)
            .unwrap_or(false)
    {
        return Err(type_error(cx));
    }
    let target_length = cx
        .agent()
        .backing_store_byte_length(new_store)
        .ok_or_else(|| type_error(cx))?;
    if target_length < new_length {
        return Err(type_error(cx));
    }
    for (target_index, source_index) in (start_index..end_index).enumerate() {
        let byte = cx
            .agent()
            .backing_store_get_byte(store, source_index)
            .ok_or_else(|| type_error(cx))?;
        if !cx
            .agent()
            .backing_store_set_byte(new_store, target_index, byte)
        {
            return Err(type_error(cx));
        }
    }
    Ok(Value::from_object_ref(result))
}

fn array_buffer_slice_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    array_buffer_family_slice_builtin(cx, invocation, false)
}

fn shared_array_buffer_slice_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    array_buffer_family_slice_builtin(cx, invocation, true)
}

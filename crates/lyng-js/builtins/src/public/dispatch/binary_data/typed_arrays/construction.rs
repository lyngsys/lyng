use super::super::super::{
    big_int64_array_builtin, big_uint64_array_builtin, float16_array_builtin,
    float32_array_builtin, float64_array_builtin, int16_array_builtin, int32_array_builtin,
    int8_array_builtin, set_property_on_object, typed_array_builtin, typed_array_from_builtin,
    typed_array_of_builtin, uint16_array_builtin, uint32_array_builtin, uint8_array_builtin,
    uint8_clamped_array_builtin,
};
use super::super::{
    array_like_index_property_key, array_like_length_u64, buffers::allocate_array_buffer_object,
    get_property_from_object, iterable_to_values_list, length_value_u64, range_error,
    to_index_for_builtin, type_error, PublicBuiltinDispatchContext,
};
use super::{
    allocate_typed_array_object, typed_array_default_prototype,
    typed_array_storage_bits_from_builtin_value, typed_array_this_record,
};
use crate::BuiltinInvocation;
use lyng_js_objects::{TypedArrayElementKind, TypedArrayObjectData};
use lyng_js_types::{
    BackingStoreRef, BuiltinFunctionId, ObjectRef, PropertyKey, Value, WellKnownSymbolId,
};

pub(in crate::public::dispatch::binary_data) fn dispatch_typed_array_constructor_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == typed_array_builtin() {
        return abstract_typed_array_builtin(context, invocation).map(Some);
    }
    if entry == typed_array_from_builtin() {
        return typed_array_from_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_of_builtin() {
        return typed_array_of_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == int8_array_builtin() {
        return int8_array_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == int16_array_builtin() {
        return int16_array_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == int32_array_builtin() {
        return int32_array_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == float16_array_builtin() {
        return float16_array_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == float32_array_builtin() {
        return float32_array_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == float64_array_builtin() {
        return float64_array_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == big_int64_array_builtin() {
        return big_int64_array_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == big_uint64_array_builtin() {
        return big_uint64_array_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == uint32_array_builtin() {
        return uint32_array_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == uint16_array_builtin() {
        return uint16_array_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == uint8_clamped_array_builtin() {
        return uint8_clamped_array_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == uint8_array_builtin() {
        return uint8_array_builtin_dispatch(context, invocation).map(Some);
    }
    Ok(None)
}

fn typed_array_constructor_receiver<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    this_value: Value,
) -> Result<ObjectRef, Cx::Error> {
    this_value.as_object_ref().ok_or_else(|| type_error(cx))
}

enum TypedArrayFromSource {
    Iterable(Vec<Value>),
    ArrayLike { object: ObjectRef, length: usize },
}

fn typed_array_from_source<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    source: Value,
) -> Result<TypedArrayFromSource, Cx::Error> {
    if let Some(iterator_symbol) = cx.agent().well_known_symbol(WellKnownSymbolId::Iterator) {
        let iterator_method =
            cx.get_property_value(source, PropertyKey::from_symbol(iterator_symbol))?;
        if !(iterator_method.is_undefined() || iterator_method.is_null()) {
            return Ok(TypedArrayFromSource::Iterable(iterable_to_values_list(
                cx, source,
            )?));
        }
    }
    let object = cx.to_object_for_builtin_value(cx.builtin_realm(), source)?;
    let length = array_like_length_u64(cx, object)?;
    let length = usize::try_from(length).map_err(|_| range_error(cx))?;
    Ok(TypedArrayFromSource::ArrayLike { object, length })
}

fn typed_array_construct_from_receiver<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor: ObjectRef,
    length: usize,
) -> Result<(ObjectRef, TypedArrayObjectData), Cx::Error> {
    let object = cx.construct_to_completion(
        constructor,
        &[length_value_u64(u64::try_from(length).unwrap_or(u64::MAX))],
        None,
    )?;
    let record = typed_array_this_record(cx, Value::from_object_ref(object))?;
    if record.length() < length {
        return Err(type_error(cx));
    }
    Ok((object, record))
}

fn typed_array_allocation_byte_length<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    length: usize,
    element_size: usize,
) -> Result<usize, Cx::Error> {
    let byte_length = length
        .checked_mul(element_size)
        .ok_or_else(|| range_error(cx))?;
    if byte_length > cx.agent().backing_store_allocation_limit() {
        return Err(range_error(cx));
    }
    Ok(byte_length)
}

fn typed_array_allocation_shape<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    length: u64,
    element_size: usize,
) -> Result<(usize, usize), Cx::Error> {
    let length = usize::try_from(length).map_err(|_| range_error(cx))?;
    let byte_length = typed_array_allocation_byte_length(cx, length, element_size)?;
    Ok((length, byte_length))
}

fn typed_array_buffer_byte_length<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    store: BackingStoreRef,
) -> Result<usize, Cx::Error> {
    cx.agent()
        .backing_store_byte_length(store)
        .ok_or_else(|| type_error(cx))
}

fn typed_array_buffer_is_detached<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    store: BackingStoreRef,
) -> Result<bool, Cx::Error> {
    cx.agent()
        .backing_store_is_detached(store)
        .ok_or_else(|| type_error(cx))
}

fn typed_array_from_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let constructor = typed_array_constructor_receiver(cx, invocation.this_value())?;
    let source = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let mapper = match invocation.arguments().get(1).copied() {
        Some(mapper) if !mapper.is_undefined() => Some(cx.require_callable_object(mapper)?),
        _ => None,
    };
    let this_arg = invocation
        .arguments()
        .get(2)
        .copied()
        .unwrap_or(Value::undefined());
    let source = typed_array_from_source(cx, source)?;
    let length = match &source {
        TypedArrayFromSource::Iterable(values) => values.len(),
        TypedArrayFromSource::ArrayLike { length, .. } => *length,
    };
    let (object, _record) = typed_array_construct_from_receiver(cx, constructor, length)?;
    for index in 0..length {
        let value = match &source {
            TypedArrayFromSource::Iterable(values) => values[index],
            TypedArrayFromSource::ArrayLike {
                object: source_object,
                ..
            } => {
                let key =
                    array_like_index_property_key(cx, u64::try_from(index).unwrap_or(u64::MAX));
                get_property_from_object(cx, *source_object, key)?
            }
        };
        let mapped = if let Some(mapper) = mapper {
            cx.call_to_completion(
                mapper,
                this_arg,
                &[
                    value,
                    length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)),
                ],
            )?
        } else {
            value
        };
        let key = array_like_index_property_key(cx, u64::try_from(index).unwrap_or(u64::MAX));
        set_property_on_object(cx, object, key, mapped)?;
    }
    Ok(Value::from_object_ref(object))
}

fn typed_array_of_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let constructor = typed_array_constructor_receiver(cx, invocation.this_value())?;
    let values = invocation.arguments();
    let (object, _record) = typed_array_construct_from_receiver(cx, constructor, values.len())?;
    for (index, value) in values.iter().copied().enumerate() {
        let key = array_like_index_property_key(cx, u64::try_from(index).unwrap_or(u64::MAX));
        set_property_on_object(cx, object, key, value)?;
    }
    Ok(Value::from_object_ref(object))
}

fn typed_array_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: TypedArrayElementKind,
) -> Result<Value, Cx::Error> {
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let argument = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let element_size = kind.bytes_per_element();
    let realm = cx.builtin_realm();
    let array_buffer_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().array_buffer_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let default_prototype = typed_array_default_prototype(cx, realm, kind)?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let (buffer_object, store, byte_offset, length, length_tracking) = if let Some(buffer_object) =
        argument.as_object_ref()
    {
        if let Some(buffer) = cx.agent().objects().array_buffer(buffer_object) {
            let store = buffer.backing_store();
            let byte_offset = to_index_for_builtin(
                cx,
                invocation
                    .arguments()
                    .get(1)
                    .copied()
                    .unwrap_or(Value::undefined()),
            )?;
            let byte_offset = usize::try_from(byte_offset).map_err(|_| range_error(cx))?;
            if byte_offset % element_size != 0 {
                return Err(range_error(cx));
            }
            let explicit_length = invocation
                .arguments()
                .get(2)
                .copied()
                .filter(|value| !value.is_undefined());
            let length_tracking = buffer.is_resizable() && explicit_length.is_none();
            let explicit_length = if let Some(value) = explicit_length {
                Some(
                    usize::try_from(to_index_for_builtin(cx, value)?)
                        .map_err(|_| range_error(cx))?,
                )
            } else {
                None
            };
            if typed_array_buffer_is_detached(cx, store)? {
                return Err(type_error(cx));
            }
            let store_len = typed_array_buffer_byte_length(cx, store)?;
            let length = if let Some(length) = explicit_length {
                length
            } else {
                if byte_offset > store_len {
                    return Err(range_error(cx));
                }
                let remaining_bytes = store_len - byte_offset;
                if remaining_bytes % element_size != 0 && !length_tracking {
                    return Err(range_error(cx));
                }
                remaining_bytes / element_size
            };
            if byte_offset > store_len {
                return Err(range_error(cx));
            }
            let byte_length = length
                .checked_mul(element_size)
                .ok_or_else(|| range_error(cx))?;
            if byte_offset.saturating_add(byte_length) > store_len {
                return Err(range_error(cx));
            }
            (buffer_object, store, byte_offset, length, length_tracking)
        } else {
            let iterator_method = if let Some(iterator_symbol) =
                cx.agent().well_known_symbol(WellKnownSymbolId::Iterator)
            {
                cx.get_property_value(argument, PropertyKey::from_symbol(iterator_symbol))?
            } else {
                Value::undefined()
            };
            let from_iterator = !(iterator_method.is_undefined() || iterator_method.is_null());
            let elements = if from_iterator {
                Some(iterable_to_values_list(cx, argument)?)
            } else {
                None
            };
            let (length, byte_length) = if let Some(elements) = &elements {
                (
                    elements.len(),
                    typed_array_allocation_byte_length(cx, elements.len(), element_size)?,
                )
            } else {
                let length = array_like_length_u64(cx, buffer_object)?;
                typed_array_allocation_shape(cx, length, element_size)?
            };
            let store = cx
                .agent()
                .allocate_backing_store(byte_length)
                .ok_or_else(|| range_error(cx))?;
            if let Some(elements) = elements {
                for (index, element) in elements.iter().copied().enumerate() {
                    let bits = typed_array_storage_bits_from_builtin_value(cx, kind, element)?;
                    let start = index
                        .checked_mul(element_size)
                        .ok_or_else(|| range_error(cx))?;
                    for offset in 0..element_size {
                        let byte_index =
                            start.checked_add(offset).ok_or_else(|| range_error(cx))?;
                        let shift = offset * 8;
                        let byte =
                            u8::try_from((bits >> shift) & 0xff).expect("element byte should fit");
                        if !cx.agent().backing_store_set_byte(store, byte_index, byte) {
                            return Err(range_error(cx));
                        }
                    }
                }
            } else {
                for index in 0..length {
                    let key =
                        array_like_index_property_key(cx, u64::try_from(index).unwrap_or(u64::MAX));
                    let element = get_property_from_object(cx, buffer_object, key)?;
                    let bits = typed_array_storage_bits_from_builtin_value(cx, kind, element)?;
                    let start = index
                        .checked_mul(element_size)
                        .ok_or_else(|| range_error(cx))?;
                    for offset in 0..element_size {
                        let byte_index =
                            start.checked_add(offset).ok_or_else(|| range_error(cx))?;
                        let shift = offset * 8;
                        let byte =
                            u8::try_from((bits >> shift) & 0xff).expect("element byte should fit");
                        if !cx.agent().backing_store_set_byte(store, byte_index, byte) {
                            return Err(range_error(cx));
                        }
                    }
                }
            }
            let buffer_object =
                allocate_array_buffer_object(cx, realm, array_buffer_prototype, store)?;
            (buffer_object, store, 0, length, false)
        }
    } else {
        let length = to_index_for_builtin(cx, argument)?;
        let length = usize::try_from(length).map_err(|_| range_error(cx))?;
        let byte_length = typed_array_allocation_byte_length(cx, length, element_size)?;
        let store = cx
            .agent()
            .allocate_backing_store(byte_length)
            .ok_or_else(|| range_error(cx))?;
        let buffer_object = allocate_array_buffer_object(cx, realm, array_buffer_prototype, store)?;
        (buffer_object, store, 0, length, false)
    };
    let record = if length_tracking {
        TypedArrayObjectData::new_length_tracking(buffer_object, store, byte_offset, length, kind)
    } else {
        TypedArrayObjectData::new(buffer_object, store, byte_offset, length, kind)
    };
    let object = allocate_typed_array_object(cx, realm, prototype, record)?;
    Ok(Value::from_object_ref(object))
}

fn int8_array_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Int8)
}

fn int16_array_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Int16)
}

fn int32_array_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Int32)
}

fn float16_array_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Float16)
}

fn float32_array_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Float32)
}

fn float64_array_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Float64)
}

fn big_int64_array_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::BigInt64)
}

fn big_uint64_array_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::BigUint64)
}

fn uint16_array_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Uint16)
}

fn uint32_array_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Uint32)
}

fn uint8_clamped_array_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Uint8Clamped)
}

fn uint8_array_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Uint8)
}

fn abstract_typed_array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Err(type_error(cx))
}

use super::super::super::{
    big_int64_array_builtin, big_uint64_array_builtin, float32_array_builtin,
    float64_array_builtin, int16_array_builtin, int32_array_builtin, int8_array_builtin,
    typed_array_builtin, typed_array_from_builtin, typed_array_of_builtin, uint16_array_builtin,
    uint32_array_builtin, uint8_array_builtin, uint8_clamped_array_builtin,
};
use super::super::{
    buffers::allocate_array_buffer_object, collect_array_like_values_for_from_builtin,
    iterable_to_values_list, length_value_u64, range_error, to_index_for_builtin, type_error,
    PublicBuiltinDispatchContext,
};
use super::{
    allocate_typed_array_object, typed_array_default_prototype,
    typed_array_storage_bits_from_builtin_value, typed_array_this_record,
    typed_array_write_storage_bits,
};
use crate::BuiltinInvocation;
use lyng_js_objects::{TypedArrayElementKind, TypedArrayObjectData};
use lyng_js_types::{BuiltinFunctionId, ObjectRef, PropertyKey, Value, WellKnownSymbolId};

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

fn typed_array_collect_from_source<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    source: Value,
) -> Result<Vec<Value>, Cx::Error> {
    if let Some(iterator_symbol) = cx.agent().well_known_symbol(WellKnownSymbolId::Iterator) {
        let iterator_method =
            cx.get_property_value(source, PropertyKey::from_symbol(iterator_symbol))?;
        if !(iterator_method.is_undefined() || iterator_method.is_null()) {
            return iterable_to_values_list(cx, source);
        }
    }
    collect_array_like_values_for_from_builtin(cx, source)
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
    Ok((object, record))
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
    let values = typed_array_collect_from_source(cx, source)?;
    let (object, record) = typed_array_construct_from_receiver(cx, constructor, values.len())?;
    for (index, value) in values.iter().copied().enumerate() {
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
        let bits = typed_array_storage_bits_from_builtin_value(cx, record.kind(), mapped)?;
        typed_array_write_storage_bits(cx, record, index, bits)?;
    }
    Ok(Value::from_object_ref(object))
}

fn typed_array_of_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let constructor = typed_array_constructor_receiver(cx, invocation.this_value())?;
    let values = invocation.arguments();
    let (object, record) = typed_array_construct_from_receiver(cx, constructor, values.len())?;
    for (index, value) in values.iter().copied().enumerate() {
        let bits = typed_array_storage_bits_from_builtin_value(cx, record.kind(), value)?;
        typed_array_write_storage_bits(cx, record, index, bits)?;
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
    let (buffer_object, store, byte_offset, length) = if let Some(buffer_object) =
        argument.as_object_ref()
    {
        if let Some(buffer) = cx.agent().objects().array_buffer(buffer_object) {
            let store = buffer.backing_store();
            if cx
                .agent()
                .backing_store_is_detached(store)
                .ok_or_else(|| type_error(cx))?
            {
                return Err(type_error(cx));
            }
            let store_len = cx
                .agent()
                .backing_store_byte_length(store)
                .ok_or_else(|| type_error(cx))?;
            let byte_offset = to_index_for_builtin(
                cx,
                invocation
                    .arguments()
                    .get(1)
                    .copied()
                    .unwrap_or(Value::undefined()),
            )?;
            let byte_offset = usize::try_from(byte_offset).map_err(|_| range_error(cx))?;
            if byte_offset > store_len || byte_offset % element_size != 0 {
                return Err(range_error(cx));
            }
            let length = if let Some(value) = invocation.arguments().get(2).copied() {
                let requested = to_index_for_builtin(cx, value)?;
                usize::try_from(requested).map_err(|_| range_error(cx))?
            } else {
                let remaining_bytes = store_len - byte_offset;
                if remaining_bytes % element_size != 0 {
                    return Err(range_error(cx));
                }
                remaining_bytes / element_size
            };
            let byte_length = length
                .checked_mul(element_size)
                .ok_or_else(|| range_error(cx))?;
            if byte_offset.saturating_add(byte_length) > store_len {
                return Err(range_error(cx));
            }
            (buffer_object, store, byte_offset, length)
        } else {
            let elements = if let Some(iterator_symbol) =
                cx.agent().well_known_symbol(WellKnownSymbolId::Iterator)
            {
                let iterator_method =
                    cx.get_property_value(argument, PropertyKey::from_symbol(iterator_symbol))?;
                if iterator_method.is_undefined() || iterator_method.is_null() {
                    cx.collect_array_like_arguments(realm, argument)?
                } else {
                    iterable_to_values_list(cx, argument)?
                }
            } else {
                cx.collect_array_like_arguments(realm, argument)?
            };
            let length = elements.len();
            let byte_length = length
                .checked_mul(element_size)
                .ok_or_else(|| range_error(cx))?;
            let store = cx
                .agent()
                .allocate_backing_store(byte_length)
                .ok_or_else(|| range_error(cx))?;
            for (index, element) in elements.iter().copied().enumerate() {
                let bits = typed_array_storage_bits_from_builtin_value(cx, kind, element)?;
                let start = index
                    .checked_mul(element_size)
                    .ok_or_else(|| range_error(cx))?;
                for offset in 0..element_size {
                    let byte_index = start.checked_add(offset).ok_or_else(|| range_error(cx))?;
                    let shift = offset * 8;
                    let byte =
                        u8::try_from((bits >> shift) & 0xff).expect("element byte should fit");
                    if !cx.agent().backing_store_set_byte(store, byte_index, byte) {
                        return Err(range_error(cx));
                    }
                }
            }
            let buffer_object =
                allocate_array_buffer_object(cx, realm, array_buffer_prototype, store)?;
            (buffer_object, store, 0, length)
        }
    } else {
        let length = to_index_for_builtin(cx, argument)?;
        let length = usize::try_from(length).map_err(|_| range_error(cx))?;
        let byte_length = length
            .checked_mul(element_size)
            .ok_or_else(|| range_error(cx))?;
        let store = cx
            .agent()
            .allocate_backing_store(byte_length)
            .ok_or_else(|| range_error(cx))?;
        let buffer_object = allocate_array_buffer_object(cx, realm, array_buffer_prototype, store)?;
        (buffer_object, store, 0, length)
    };
    let default_prototype = typed_array_default_prototype(cx, realm, kind)?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let object = allocate_typed_array_object(
        cx,
        realm,
        prototype,
        TypedArrayObjectData::new(buffer_object, store, byte_offset, length, kind),
    )?;
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

use super::super::super::{
    typed_array_at_builtin, typed_array_to_locale_string_builtin, typed_array_to_string_builtin,
    typed_array_to_string_tag_getter_builtin, uint8_array_buffer_getter_builtin,
    uint8_array_byte_length_getter_builtin, uint8_array_byte_offset_getter_builtin,
    uint8_array_entries_builtin, uint8_array_keys_builtin, uint8_array_length_getter_builtin,
    uint8_array_values_builtin,
};
use super::super::{
    iterators, length_value_u64, normalize_relative_index_u64, property_key_from_text, range_error,
    string_value, to_integer_or_infinity_for_builtin, type_error, PublicBuiltinDispatchContext,
};
use super::{
    typed_array_is_out_of_bounds, typed_array_read_element_value, typed_array_read_storage_bits,
    typed_array_storage_bits_to_value, typed_array_this_record,
    typed_array_validated_record_and_length,
};
use crate::BuiltinInvocation;
use lyng_js_objects::TypedArrayElementKind;
use lyng_js_types::{BuiltinFunctionId, Value};

pub(in crate::public::dispatch::binary_data) fn dispatch_typed_array_access_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == uint8_array_buffer_getter_builtin() {
        return typed_array_buffer_getter_builtin(context, invocation).map(Some);
    }
    if entry == uint8_array_byte_length_getter_builtin() {
        return typed_array_byte_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == uint8_array_byte_offset_getter_builtin() {
        return typed_array_byte_offset_getter_builtin(context, invocation).map(Some);
    }
    if entry == uint8_array_length_getter_builtin() {
        return typed_array_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == uint8_array_values_builtin() {
        return typed_array_values_builtin(context, invocation).map(Some);
    }
    if entry == uint8_array_keys_builtin() {
        return typed_array_keys_builtin(context, invocation).map(Some);
    }
    if entry == uint8_array_entries_builtin() {
        return typed_array_entries_builtin(context, invocation).map(Some);
    }
    if entry == typed_array_at_builtin() {
        return typed_array_at_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_to_locale_string_builtin() {
        return typed_array_to_locale_string_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_to_string_builtin() {
        return typed_array_to_string_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_to_string_tag_getter_builtin() {
        return Ok(Some(typed_array_to_string_tag_getter_value(
            context, invocation,
        )));
    }
    Ok(None)
}

fn typed_array_buffer_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_this_record(cx, invocation.this_value())?;
    Ok(Value::from_object_ref(record.viewed_array_buffer()))
}

fn typed_array_byte_length_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_this_record(cx, invocation.this_value())?;
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Ok(Value::from_smi(0));
    }
    if typed_array_is_out_of_bounds(cx.agent(), record) {
        return Ok(Value::from_smi(0));
    }
    Ok(length_value_u64(
        u64::try_from(record.byte_length()).unwrap_or(u64::MAX),
    ))
}

fn typed_array_byte_offset_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_this_record(cx, invocation.this_value())?;
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Ok(Value::from_smi(0));
    }
    if typed_array_is_out_of_bounds(cx.agent(), record) {
        return Ok(Value::from_smi(0));
    }
    Ok(length_value_u64(
        u64::try_from(record.byte_offset()).unwrap_or(u64::MAX),
    ))
}

fn typed_array_length_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_this_record(cx, invocation.this_value())?;
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Ok(Value::from_smi(0));
    }
    if typed_array_is_out_of_bounds(cx.agent(), record) {
        return Ok(Value::from_smi(0));
    }
    Ok(length_value_u64(
        u64::try_from(record.length()).unwrap_or(u64::MAX),
    ))
}

fn typed_array_values_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    iterators::typed_array_iterator_factory_builtin(
        cx,
        invocation,
        iterators::ArrayIterationKind::Value,
    )
}

fn typed_array_keys_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    iterators::typed_array_iterator_factory_builtin(
        cx,
        invocation,
        iterators::ArrayIterationKind::Key,
    )
}

fn typed_array_entries_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    iterators::typed_array_iterator_factory_builtin(
        cx,
        invocation,
        iterators::ArrayIterationKind::Entry,
    )
}

fn typed_array_at_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (record, length) = typed_array_validated_record_and_length(cx, invocation.this_value())?;
    let length = u64::try_from(length).unwrap_or(u64::MAX);
    let relative_index = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let index = normalize_relative_index_u64(length, relative_index);
    if relative_index >= 0.0 {
        if index >= length {
            return Ok(Value::undefined());
        }
    } else if relative_index.is_infinite() || {
        #[allow(
            clippy::cast_precision_loss,
            reason = "TypedArray.prototype.at compares ECMAScript Number indices with array length"
        )]
        let length_number = length as f64;
        relative_index.abs() > length_number
    } {
        return Ok(Value::undefined());
    }
    let element_index = usize::try_from(index).map_err(|_| range_error(cx))?;
    Ok(
        typed_array_read_storage_bits(cx.agent(), record, element_index)
            .map_or(Value::undefined(), |bits| {
                typed_array_storage_bits_to_value(cx.agent(), record.kind(), bits)
            }),
    )
}

fn typed_array_to_locale_string_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (record, length) = typed_array_validated_record_and_length(cx, invocation.this_value())?;
    let to_locale_string_key = property_key_from_text(cx, "toLocaleString");
    let intl_key = property_key_from_text(cx, "Intl");
    let realm = cx.builtin_realm();
    let Some(global_object) = cx
        .agent()
        .realm(realm)
        .map(lyng_js_env::RealmRecord::global_object)
    else {
        return Err(type_error(cx));
    };
    let intl_value = cx.get_property_value(Value::from_object_ref(global_object), intl_key)?;
    let empty_arguments = [];
    let element_arguments = if intl_value.as_object_ref().is_some() {
        invocation.arguments()
    } else {
        &empty_arguments
    };
    let mut parts = Vec::with_capacity(length);
    for index in 0..length {
        let value = typed_array_read_element_value(cx.agent(), record, index);
        let text = if value.is_undefined() || value.is_null() {
            String::new()
        } else {
            let method_value = cx.get_property_value(value, to_locale_string_key)?;
            let method = cx.require_callable_object(method_value)?;
            let result = cx.call_to_completion(method, value, element_arguments)?;
            cx.value_to_string_text(result)?
        };
        parts.push(text);
    }
    Ok(string_value(cx, &parts.join(",")))
}

fn typed_array_to_string_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let join_key = property_key_from_text(cx, "join");
    let join_value = cx.get_property_value(invocation.this_value(), join_key)?;
    let join = cx.require_callable_object(join_value)?;
    cx.call_to_completion(join, invocation.this_value(), &[])
}

fn typed_array_to_string_tag_getter_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Value {
    let Some(object) = invocation.this_value().as_object_ref() else {
        return Value::undefined();
    };
    let Some(record) = cx.agent().objects().typed_array(object) else {
        return Value::undefined();
    };
    match record.kind() {
        TypedArrayElementKind::BigInt64 => string_value(cx, "BigInt64Array"),
        TypedArrayElementKind::BigUint64 => string_value(cx, "BigUint64Array"),
        TypedArrayElementKind::Int8 => string_value(cx, "Int8Array"),
        TypedArrayElementKind::Int16 => string_value(cx, "Int16Array"),
        TypedArrayElementKind::Int32 => string_value(cx, "Int32Array"),
        TypedArrayElementKind::Float16 => string_value(cx, "Float16Array"),
        TypedArrayElementKind::Float32 => string_value(cx, "Float32Array"),
        TypedArrayElementKind::Float64 => string_value(cx, "Float64Array"),
        TypedArrayElementKind::Uint32 => string_value(cx, "Uint32Array"),
        TypedArrayElementKind::Uint16 => string_value(cx, "Uint16Array"),
        TypedArrayElementKind::Uint8Clamped => string_value(cx, "Uint8ClampedArray"),
        TypedArrayElementKind::Uint8 => string_value(cx, "Uint8Array"),
    }
}

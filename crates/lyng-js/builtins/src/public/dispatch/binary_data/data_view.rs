use super::{
    length_value_u64, range_error, to_boolean_for_builtin, to_index_for_builtin,
    to_number_for_builtin, to_uint32_for_builtin, to_uint8_for_builtin, type_error,
    PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::{DataViewObjectData, ObjectAllocation, ObjectColdData, OrdinaryObjectData};
use lyng_js_types::{BuiltinFunctionId, RealmRef, Value};

pub(super) fn dispatch_data_view_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::super::data_view_builtin() {
        return data_view_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_buffer_getter_builtin() {
        return data_view_buffer_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_byte_length_getter_builtin() {
        return data_view_byte_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_byte_offset_getter_builtin() {
        return data_view_byte_offset_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_get_float32_builtin() {
        return data_view_get_float32_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_get_float64_builtin() {
        return data_view_get_float64_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_get_int16_builtin() {
        return data_view_get_int16_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_get_int32_builtin() {
        return data_view_get_int32_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_get_int8_builtin() {
        return data_view_get_int8_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_get_uint16_builtin() {
        return data_view_get_uint16_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_get_uint32_builtin() {
        return data_view_get_uint32_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_get_uint8_builtin() {
        return data_view_get_uint8_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_set_float32_builtin() {
        return data_view_set_float32_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_set_float64_builtin() {
        return data_view_set_float64_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_set_int16_builtin() {
        return data_view_set_int16_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_set_int32_builtin() {
        return data_view_set_int32_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_set_int8_builtin() {
        return data_view_set_int8_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_set_uint16_builtin() {
        return data_view_set_uint16_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_set_uint32_builtin() {
        return data_view_set_uint32_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_set_uint8_builtin() {
        return data_view_set_uint8_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn allocate_data_view_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
    data_view: DataViewObjectData,
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
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::DataView)),
            AllocationLifetime::Default,
        );
        let installed = objects.install_data_view_object(object, data_view);
        debug_assert!(
            installed,
            "fresh DataView object should install its view record"
        );
        object
    }))
}

fn data_view_this_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<DataViewObjectData, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    cx.agent()
        .objects()
        .data_view(object)
        .ok_or_else(|| type_error(cx))
}

fn data_view_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let buffer_object = invocation
        .arguments()
        .first()
        .and_then(|value| value.as_object_ref())
        .ok_or_else(|| type_error(cx))?;
    let buffer = cx
        .agent()
        .objects()
        .array_buffer(buffer_object)
        .ok_or_else(|| type_error(cx))?;
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
    if byte_offset > store_len {
        return Err(range_error(cx));
    }
    let byte_length = if let Some(value) = invocation
        .arguments()
        .get(2)
        .copied()
        .filter(|value| !value.is_undefined())
    {
        let requested = to_index_for_builtin(cx, value)?;
        usize::try_from(requested).map_err(|_| range_error(cx))?
    } else {
        store_len - byte_offset
    };
    if byte_offset.saturating_add(byte_length) > store_len {
        return Err(range_error(cx));
    }
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().data_view_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let object = allocate_data_view_object(
        cx,
        realm,
        prototype,
        DataViewObjectData::new(buffer_object, store, byte_offset, byte_length),
    )?;
    Ok(Value::from_object_ref(object))
}

fn data_view_buffer_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    Ok(Value::from_object_ref(record.viewed_array_buffer()))
}

fn data_view_byte_length_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    Ok(length_value_u64(
        u64::try_from(record.byte_length()).unwrap_or(u64::MAX),
    ))
}

fn data_view_byte_offset_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    Ok(length_value_u64(
        u64::try_from(record.byte_offset()).unwrap_or(u64::MAX),
    ))
}

fn data_view_checked_access<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    index_value: Value,
) -> Result<usize, Cx::Error> {
    let index = to_index_for_builtin(cx, index_value)?;
    usize::try_from(index).map_err(|_| range_error(cx))
}

fn data_view_checked_byte_offset<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    record: DataViewObjectData,
    index: usize,
    byte_length: usize,
) -> Result<usize, Cx::Error> {
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    let end_index = index
        .checked_add(byte_length)
        .ok_or_else(|| range_error(cx))?;
    if end_index > record.byte_length() {
        return Err(range_error(cx));
    }
    record
        .byte_offset()
        .checked_add(index)
        .ok_or_else(|| range_error(cx))
}

fn data_view_read_unsigned<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    record: DataViewObjectData,
    absolute_index: usize,
    byte_length: usize,
    little_endian: bool,
) -> Result<u64, Cx::Error> {
    let mut value = 0_u64;
    for offset in 0..byte_length {
        let byte_index = absolute_index
            .checked_add(offset)
            .ok_or_else(|| range_error(cx))?;
        let byte = cx
            .agent()
            .backing_store_get_byte(record.backing_store(), byte_index)
            .ok_or_else(|| range_error(cx))?;
        if little_endian {
            value |= u64::from(byte) << (offset * 8);
        } else {
            value = (value << 8) | u64::from(byte);
        }
    }
    Ok(value)
}

fn data_view_write_unsigned<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    record: DataViewObjectData,
    absolute_index: usize,
    byte_length: usize,
    value: u64,
    little_endian: bool,
) -> Result<(), Cx::Error> {
    for offset in 0..byte_length {
        let byte_index = absolute_index
            .checked_add(offset)
            .ok_or_else(|| range_error(cx))?;
        let shift = if little_endian {
            offset * 8
        } else {
            (byte_length - 1 - offset) * 8
        };
        let byte = u8::try_from((value >> shift) & 0xff).expect("byte extraction should fit");
        if !cx
            .agent()
            .backing_store_set_byte(record.backing_store(), byte_index, byte)
        {
            return Err(range_error(cx));
        }
    }
    Ok(())
}

fn data_view_get_uint8_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    let index = data_view_checked_access(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 1)?;
    let value = cx
        .agent()
        .backing_store_get_byte(record.backing_store(), absolute_index)
        .ok_or_else(|| range_error(cx))?;
    Ok(Value::from_smi(i32::from(value)))
}

fn data_view_get_int8_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    let index = data_view_checked_access(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 1)?;
    let value = cx
        .agent()
        .backing_store_get_byte(record.backing_store(), absolute_index)
        .ok_or_else(|| range_error(cx))?;
    Ok(Value::from_smi(i32::from(value as i8)))
}

fn data_view_get_uint16_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    let index = data_view_checked_access(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 2)?;
    let little_endian = to_boolean_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let value = data_view_read_unsigned(cx, record, absolute_index, 2, little_endian)?;
    Ok(Value::from_smi(i32::try_from(value).unwrap_or(i32::MAX)))
}

fn data_view_get_int16_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    let index = data_view_checked_access(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 2)?;
    let little_endian = to_boolean_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let value = data_view_read_unsigned(cx, record, absolute_index, 2, little_endian)? as u16;
    Ok(Value::from_smi(i32::from(value as i16)))
}

fn data_view_get_int32_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    let index = data_view_checked_access(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 4)?;
    let little_endian = to_boolean_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let value = data_view_read_unsigned(cx, record, absolute_index, 4, little_endian)?;
    Ok(Value::from_smi(value as i32))
}

fn data_view_get_float32_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    let index = data_view_checked_access(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 4)?;
    let little_endian = to_boolean_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let bits = data_view_read_unsigned(cx, record, absolute_index, 4, little_endian)? as u32;
    Ok(Value::from_f64(f64::from(f32::from_bits(bits))))
}

fn data_view_get_float64_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    let index = data_view_checked_access(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 8)?;
    let little_endian = to_boolean_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let bits = data_view_read_unsigned(cx, record, absolute_index, 8, little_endian)?;
    Ok(Value::from_f64(f64::from_bits(bits)))
}

fn data_view_get_uint32_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    let index = data_view_checked_access(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 4)?;
    let little_endian = to_boolean_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let value = data_view_read_unsigned(cx, record, absolute_index, 4, little_endian)?;
    Ok(length_value_u64(value))
}

fn data_view_set_uint8_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    let index = data_view_checked_access(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let byte = to_uint8_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 1)?;
    data_view_write_unsigned(cx, record, absolute_index, 1, u64::from(byte), true)?;
    Ok(Value::undefined())
}

fn data_view_set_int8_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    let index = data_view_checked_access(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let byte = to_uint8_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 1)?;
    data_view_write_unsigned(cx, record, absolute_index, 1, u64::from(byte), true)?;
    Ok(Value::undefined())
}

fn data_view_set_uint16_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    let index = data_view_checked_access(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let value = to_uint32_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 2)?;
    let little_endian = to_boolean_for_builtin(
        cx,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    data_view_write_unsigned(
        cx,
        record,
        absolute_index,
        2,
        u64::from(value),
        little_endian,
    )?;
    Ok(Value::undefined())
}

fn data_view_set_int16_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    let index = data_view_checked_access(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let value = to_uint32_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 2)?;
    let little_endian = to_boolean_for_builtin(
        cx,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    data_view_write_unsigned(
        cx,
        record,
        absolute_index,
        2,
        u64::from(value),
        little_endian,
    )?;
    Ok(Value::undefined())
}

fn data_view_set_int32_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    let index = data_view_checked_access(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let value = to_uint32_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 4)?;
    let little_endian = to_boolean_for_builtin(
        cx,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    data_view_write_unsigned(
        cx,
        record,
        absolute_index,
        4,
        u64::from(value),
        little_endian,
    )?;
    Ok(Value::undefined())
}

fn data_view_set_float32_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    let index = data_view_checked_access(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let value = to_number_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 4)?;
    let little_endian = to_boolean_for_builtin(
        cx,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let bits = f32::to_bits(value as f32);
    data_view_write_unsigned(
        cx,
        record,
        absolute_index,
        4,
        u64::from(bits),
        little_endian,
    )?;
    Ok(Value::undefined())
}

fn data_view_set_float64_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    let index = data_view_checked_access(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let value = to_number_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 8)?;
    let little_endian = to_boolean_for_builtin(
        cx,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    data_view_write_unsigned(
        cx,
        record,
        absolute_index,
        8,
        value.to_bits(),
        little_endian,
    )?;
    Ok(Value::undefined())
}

fn data_view_set_uint32_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    let index = data_view_checked_access(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let value = to_uint32_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 4)?;
    let little_endian = to_boolean_for_builtin(
        cx,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    data_view_write_unsigned(
        cx,
        record,
        absolute_index,
        4,
        u64::from(value),
        little_endian,
    )?;
    Ok(Value::undefined())
}

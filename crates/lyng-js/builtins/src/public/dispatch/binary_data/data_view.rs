use super::{
    length_value_u64, range_error, to_bigint_for_builtin, to_boolean_for_builtin,
    to_index_for_builtin, to_number_for_builtin, to_uint32_for_builtin, to_uint8_for_builtin,
    type_error, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_env::Agent;
use lyng_js_gc::{AllocationLifetime, BigIntSign};
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
    if entry == super::super::data_view_get_big_int64_builtin() {
        return data_view_get_big_int64_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_get_big_uint64_builtin() {
        return data_view_get_big_uint64_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_set_big_int64_builtin() {
        return data_view_set_big_int64_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_set_big_uint64_builtin() {
        return data_view_set_big_uint64_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_get_float16_builtin() {
        return data_view_get_float16_builtin(context, invocation).map(Some);
    }
    if entry == super::super::data_view_set_float16_builtin() {
        return data_view_set_float16_builtin(context, invocation).map(Some);
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
    let explicit_byte_length = if let Some(value) = invocation
        .arguments()
        .get(2)
        .copied()
        .filter(|value| !value.is_undefined())
    {
        let requested = to_index_for_builtin(cx, value)?;
        Some(usize::try_from(requested).map_err(|_| range_error(cx))?)
    } else {
        None
    };
    if explicit_byte_length.is_some_and(|byte_length| {
        byte_offset
            .checked_add(byte_length)
            .is_none_or(|end| end > store_len)
    }) {
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
    if cx
        .agent()
        .backing_store_is_detached(store)
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    let current_store_len = cx
        .agent()
        .backing_store_byte_length(store)
        .ok_or_else(|| type_error(cx))?;
    if byte_offset > current_store_len {
        return Err(range_error(cx));
    }
    if explicit_byte_length.is_some_and(|byte_length| {
        byte_offset
            .checked_add(byte_length)
            .is_none_or(|end| end > current_store_len)
    }) {
        return Err(range_error(cx));
    }
    let data = match explicit_byte_length {
        Some(byte_length) => {
            DataViewObjectData::new(buffer_object, store, byte_offset, byte_length)
        }
        None if buffer.is_resizable() => {
            DataViewObjectData::new_length_tracking(buffer_object, store, byte_offset)
        }
        None => DataViewObjectData::new(
            buffer_object,
            store,
            byte_offset,
            current_store_len - byte_offset,
        ),
    };
    let object = allocate_data_view_object(cx, realm, prototype, data)?;
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
    let byte_length = data_view_current_byte_length(cx, record)?;
    Ok(length_value_u64(
        u64::try_from(byte_length).unwrap_or(u64::MAX),
    ))
}

fn data_view_byte_offset_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = data_view_this_record(cx, invocation.this_value())?;
    let _ = data_view_current_byte_length(cx, record)?;
    Ok(length_value_u64(
        u64::try_from(record.byte_offset()).unwrap_or(u64::MAX),
    ))
}

fn data_view_current_byte_length<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    record: DataViewObjectData,
) -> Result<usize, Cx::Error> {
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    let store_len = cx
        .agent()
        .backing_store_byte_length(record.backing_store())
        .ok_or_else(|| type_error(cx))?;
    if record.is_length_tracking() {
        return store_len
            .checked_sub(record.byte_offset())
            .ok_or_else(|| type_error(cx));
    }
    let end = record
        .byte_offset()
        .checked_add(record.byte_length())
        .ok_or_else(|| type_error(cx))?;
    if end > store_len {
        return Err(type_error(cx));
    }
    Ok(record.byte_length())
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
    let view_byte_length = data_view_current_byte_length(cx, record)?;
    let end_index = index
        .checked_add(byte_length)
        .ok_or_else(|| range_error(cx))?;
    if end_index > view_byte_length {
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
    let bits = cx
        .agent()
        .backing_store_load_bits(record.backing_store(), absolute_index, byte_length)
        .ok_or_else(|| range_error(cx))?;
    Ok(if little_endian {
        bits
    } else {
        reverse_data_view_byte_order(bits, byte_length)
    })
}

fn data_view_write_unsigned<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    record: DataViewObjectData,
    absolute_index: usize,
    byte_length: usize,
    value: u64,
    little_endian: bool,
) -> Result<(), Cx::Error> {
    let bits = if little_endian {
        value
    } else {
        reverse_data_view_byte_order(value, byte_length)
    };
    if !cx.agent().backing_store_store_bits(
        record.backing_store(),
        absolute_index,
        byte_length,
        bits,
    ) {
        return Err(range_error(cx));
    }
    Ok(())
}

fn reverse_data_view_byte_order(bits: u64, byte_length: usize) -> u64 {
    let mut reversed = 0_u64;
    for offset in 0..byte_length {
        reversed <<= 8;
        reversed |= (bits >> (offset * 8)) & 0xff;
    }
    reversed
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

fn bigint_to_uint64_bits(agent: &Agent, value: Value) -> Option<u64> {
    let bigint = value.as_bigint_ref()?;
    let view = agent.heap().view().bigint_view(bigint)?;
    let low = view.limb_at(0).unwrap_or(0);
    Some(match view.sign() {
        BigIntSign::NonNegative => low,
        BigIntSign::Negative => 0_u64.wrapping_sub(low),
    })
}

fn data_view_biguint64_value(agent: &mut Agent, bits: u64) -> Value {
    let bigint = agent.heap_mut().mutator().alloc_bigint(
        BigIntSign::NonNegative,
        &[bits],
        AllocationLifetime::Default,
    );
    Value::from_bigint_ref(bigint)
}

fn data_view_bigint64_value(agent: &mut Agent, bits: u64) -> Value {
    let (sign, limbs) = if bits >> 63 == 0 {
        (BigIntSign::NonNegative, [bits])
    } else {
        (BigIntSign::Negative, [bits.wrapping_neg()])
    };
    let bigint = agent
        .heap_mut()
        .mutator()
        .alloc_bigint(sign, &limbs, AllocationLifetime::Default);
    Value::from_bigint_ref(bigint)
}

fn data_view_get_big_int64_builtin<Cx: PublicBuiltinDispatchContext>(
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
    Ok(data_view_bigint64_value(cx.agent(), bits))
}

fn data_view_get_big_uint64_builtin<Cx: PublicBuiltinDispatchContext>(
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
    Ok(data_view_biguint64_value(cx.agent(), bits))
}

fn data_view_set_big_int64_builtin<Cx: PublicBuiltinDispatchContext>(
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
    let bigint_value = to_bigint_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let bits = bigint_to_uint64_bits(cx.agent(), bigint_value).ok_or_else(|| type_error(cx))?;
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 8)?;
    let little_endian = to_boolean_for_builtin(
        cx,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    data_view_write_unsigned(cx, record, absolute_index, 8, bits, little_endian)?;
    Ok(Value::undefined())
}

fn data_view_set_big_uint64_builtin<Cx: PublicBuiltinDispatchContext>(
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
    let bigint_value = to_bigint_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let bits = bigint_to_uint64_bits(cx.agent(), bigint_value).ok_or_else(|| type_error(cx))?;
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 8)?;
    let little_endian = to_boolean_for_builtin(
        cx,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    data_view_write_unsigned(cx, record, absolute_index, 8, bits, little_endian)?;
    Ok(Value::undefined())
}

// ---------------- IEEE-754 binary16 (Float16) conversions ----------------

/// Decode a 16-bit IEEE-754 half-precision float into a 64-bit double.
///
/// Sign, NaN-ness, and infinities are preserved. Subnormal halves are
/// renormalized to normal binary64 values; canonical NaNs map to a quiet
/// f64 NaN. Test262 asserts only NaN-ness, sign of zero, and exact value of
/// finite results, so any quiet NaN bit pattern is acceptable.
fn float16_bits_to_f64(bits: u16) -> f64 {
    let sign = (bits >> 15) & 0x1;
    let exp = ((bits >> 10) & 0x1F) as i32;
    let mant = (bits & 0x3FF) as u32;

    let sign_bit_f64: u64 = u64::from(sign) << 63;

    if exp == 0 {
        if mant == 0 {
            // Sign-preserved zero.
            return f64::from_bits(sign_bit_f64);
        }
        // Subnormal half: value = (-1)^sign * 2^-14 * (mant / 2^10).
        // Renormalize. f64 can represent every binary16 subnormal exactly
        // since it has both higher precision and a wider exponent range.
        let mut mantissa = mant;
        let mut shift: i32 = 0;
        // Find the position of the high bit; binary16 mantissa width is 10 bits
        // so the high bit is at position <= 9.
        while (mantissa & 0x400) == 0 {
            mantissa <<= 1;
            shift += 1;
        }
        // Drop the implicit leading 1.
        mantissa &= 0x3FF;
        // Half unbiased exponent for renormalized number:
        // original is 1 - 15 (denormal exponent), then -shift to undo the shifts.
        let half_unbiased = 1 - 15 - shift;
        let f64_biased = (half_unbiased + 1023) as u64;
        let f64_mant = u64::from(mantissa) << (52 - 10);
        return f64::from_bits(sign_bit_f64 | (f64_biased << 52) | f64_mant);
    }

    if exp == 0x1F {
        if mant == 0 {
            return f64::from_bits(sign_bit_f64 | 0x7FF0_0000_0000_0000);
        }
        // NaN. Produce a canonical quiet NaN; preserve sign and a non-zero
        // payload bit so the value is still NaN.
        return f64::from_bits(sign_bit_f64 | 0x7FF8_0000_0000_0000);
    }

    // Normal half: rebias exponent and zero-extend mantissa.
    let f64_biased = (exp - 15 + 1023) as u64;
    let f64_mant = u64::from(mant) << (52 - 10);
    f64::from_bits(sign_bit_f64 | (f64_biased << 52) | f64_mant)
}

/// Round an `f64` to the nearest IEEE-754 binary16, ties-to-even.
///
/// Overflow rounds to sign-preserved Infinity (`exp = 0x1F`, `mant = 0`).
/// Underflow rounds to sign-preserved zero. NaN inputs yield a canonical
/// quiet NaN.
fn f64_to_float16_bits(value: f64) -> u16 {
    let bits = value.to_bits();
    let sign16: u16 = ((bits >> 63) as u16) & 0x1;
    let exp64 = ((bits >> 52) & 0x7FF) as i32;
    let mant64 = bits & 0x000F_FFFF_FFFF_FFFF;

    // Special: NaN / Infinity.
    if exp64 == 0x7FF {
        if mant64 == 0 {
            return (sign16 << 15) | 0x7C00;
        }
        // Canonical quiet NaN: sign + max exp + leading mantissa bit.
        return (sign16 << 15) | 0x7E00;
    }

    // Treat zero (subnormal f64 too) as zero output.
    if exp64 == 0 && mant64 == 0 {
        return sign16 << 15;
    }

    // Unbiased exponent of the input.
    // For f64 normals: e = exp64 - 1023.
    // For f64 subnormals: e = -1022, and there is no implicit 1 bit. They are
    // far below the f16 minimum subnormal (~2^-24) so they always round to 0.
    if exp64 == 0 {
        return sign16 << 15;
    }
    let unbiased = exp64 - 1023;
    let f16_unbiased = unbiased + 15;

    // Build the 53-bit significand: implicit leading 1 + 52 mantissa bits.
    let signif: u64 = (1_u64 << 52) | mant64;

    if f16_unbiased >= 0x1F {
        // Overflow: result is sign-preserved Infinity.
        return (sign16 << 15) | 0x7C00;
    }

    if f16_unbiased <= 0 {
        // Subnormal output (or underflow to zero).
        // We want a 10-bit subnormal mantissa. The "needed" right shift on the
        // 53-bit significand to land its high bit in position 10 (the f16
        // subnormal mantissa MSB) when f16_unbiased == 0 is 53 - 11 = 42.
        // Each step of decreasing f16_unbiased by 1 shifts one more bit out.
        let shift = (53 - 11 + (1 - f16_unbiased)) as u32;
        if shift > 63 {
            return sign16 << 15;
        }
        let rounded = round_shift_signif(signif, shift);
        // rounded is at most 0x400 (subnormal mantissa overflow into normal).
        if rounded == 0 {
            return sign16 << 15;
        }
        if rounded == 0x400 {
            // Promote to smallest normal: exp = 1, mant = 0.
            return (sign16 << 15) | (1 << 10);
        }
        return (sign16 << 15) | (rounded as u16);
    }

    // Normal output: shift the 53-bit significand right by 53 - 11 = 42 bits,
    // rounding ties to even.
    let rounded = round_shift_signif(signif, 53 - 11);
    // `rounded` has up to 11 bits; the implicit leading 1 lives at bit 10.
    // After rounding, it may have overflowed to 0x800 (carry out of bit 10).
    if rounded == 0x800 {
        let new_exp = f16_unbiased + 1;
        if new_exp >= 0x1F {
            return (sign16 << 15) | 0x7C00;
        }
        return (sign16 << 15) | ((new_exp as u16) << 10);
    }
    let mant10 = (rounded as u16) & 0x3FF;
    (sign16 << 15) | ((f16_unbiased as u16) << 10) | mant10
}

/// Round-to-nearest, ties-to-even right shift of a non-zero u64 significand.
///
/// `shift` is the number of bits to discard. Returns the rounded high-half of
/// the value. The caller guarantees `signif` has its bit 52 set (or, for
/// subnormal codepaths, that the value is non-zero).
fn round_shift_signif(signif: u64, shift: u32) -> u64 {
    if shift == 0 {
        return signif;
    }
    if shift >= 64 {
        // Everything shifted out; rounding contribution is just "is anything
        // non-zero?". This produces 0 (all bits below half) or 1 (round up).
        // For the f16 boundary cases we hit (shift up to ~63 for tiny
        // subnormals), this collapses to 0. Caller already filters > 63.
        return 0;
    }
    let high = signif >> shift;
    let mask: u64 = (1_u64 << shift) - 1;
    let low = signif & mask;
    let half = 1_u64 << (shift - 1);
    if low < half {
        return high;
    }
    if low > half {
        return high + 1;
    }
    // Exactly half: ties-to-even.
    if (high & 1) == 1 {
        high + 1
    } else {
        high
    }
}

fn data_view_get_float16_builtin<Cx: PublicBuiltinDispatchContext>(
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
    let bits = data_view_read_unsigned(cx, record, absolute_index, 2, little_endian)? as u16;
    Ok(Value::from_f64(float16_bits_to_f64(bits)))
}

fn data_view_set_float16_builtin<Cx: PublicBuiltinDispatchContext>(
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
    let absolute_index = data_view_checked_byte_offset(cx, record, index, 2)?;
    let little_endian = to_boolean_for_builtin(
        cx,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let bits = f64_to_float16_bits(value);
    data_view_write_unsigned(
        cx,
        record,
        absolute_index,
        2,
        u64::from(bits),
        little_endian,
    )?;
    Ok(Value::undefined())
}

#[cfg(test)]
mod float16_tests {
    use super::{f64_to_float16_bits, float16_bits_to_f64};

    fn round_trip_via_f16(value: f64) -> f64 {
        let bits = f64_to_float16_bits(value);
        float16_bits_to_f64(bits)
    }

    #[test]
    fn round_trips_zeros_and_specials() {
        assert_eq!(f64_to_float16_bits(0.0), 0x0000);
        assert_eq!(f64_to_float16_bits(-0.0), 0x8000);
        assert_eq!(f64_to_float16_bits(f64::INFINITY), 0x7C00);
        assert_eq!(f64_to_float16_bits(f64::NEG_INFINITY), 0xFC00);
        let nan_bits = f64_to_float16_bits(f64::NAN);
        assert_eq!(nan_bits & 0x7C00, 0x7C00);
        assert_ne!(nan_bits & 0x03FF, 0);
    }

    #[test]
    fn handles_normal_round_trip() {
        // The Test262 vectors from byteConversionValues.js / setFloat16.
        assert_eq!(round_trip_via_f16(127.0), 127.0);
        assert_eq!(round_trip_via_f16(255.0), 255.0);
        assert_eq!(round_trip_via_f16(0.5), 0.5);
        assert_eq!(round_trip_via_f16(-1.0), -1.0);
        assert_eq!(round_trip_via_f16(65504.0), 65504.0);
        // Overflow.
        assert_eq!(f64_to_float16_bits(65520.0), 0x7C00);
        assert_eq!(f64_to_float16_bits(65519.99999999999), 0x7BFF);
        assert_eq!(round_trip_via_f16(65519.99999999999), 65504.0);
    }

    #[test]
    fn round_to_even_at_normal_boundary() {
        // 2049 -> 2048, 2051 -> 2052 (ties round to even at f16 mantissa LSB).
        assert_eq!(round_trip_via_f16(2049.0), 2048.0);
        assert_eq!(round_trip_via_f16(2051.0), 2052.0);
    }

    #[test]
    fn handles_subnormal_round_to_even() {
        // 5.960464477539063e-8 is the smallest f16 subnormal.
        assert_eq!(
            round_trip_via_f16(5.960464477539063e-8),
            5.960464477539063e-8
        );
        // 2.9802322387695312e-8 is exactly half the smallest subnormal -> 0.
        assert_eq!(round_trip_via_f16(2.9802322387695312e-8), 0.0);
        // 2.980232238769532e-8 is just above half -> rounds up.
        assert_eq!(
            round_trip_via_f16(2.980232238769532e-8),
            5.960464477539063e-8
        );
    }

    #[test]
    fn renormalizes_known_subnormals() {
        // 0.00006097555160522461 is a representable f16 subnormal.
        let v = 0.00006097555160522461_f64;
        assert_eq!(round_trip_via_f16(v), v);
    }
}

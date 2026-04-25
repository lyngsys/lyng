use super::{
    array_like_index_property_key, array_like_length_u64, arrays,
    collect_array_like_values_for_from_builtin, create_data_property_or_throw,
    get_property_from_object, iterable_to_values_list, iterators, length_value_u64, map_completion,
    normalize_relative_index_u64, promises, property_key_from_text, range_error, string_value,
    to_bigint_for_builtin, to_boolean_for_builtin, to_index_for_builtin,
    to_integer_or_infinity_for_builtin, to_number_for_builtin, to_uint32_for_builtin,
    to_uint8_clamp_for_builtin, to_uint8_for_builtin, type_error, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_common::WellKnownAtom;
use lyng_js_env::{Agent, AsyncWaiterRecord, ParkedAgentRecord, WaiterKind};
use lyng_js_gc::{AllocationLifetime, BigIntSign};
use lyng_js_host::{ParkAgentRequest, ParkAgentStatus, UnparkAgentRequest};
use lyng_js_objects::{
    ArrayBufferObjectData, DataViewObjectData, ObjectAllocation, ObjectColdData,
    OrdinaryObjectData, TypedArrayElementKind, TypedArrayObjectData,
};
use lyng_js_ops::{promise, read, shared_memory as shared_memory_ops};
use lyng_js_types::{
    BuiltinFunctionId, ObjectRef, PropertyKey, RealmRef, Value, WellKnownSymbolId,
};

pub(super) fn dispatch_binary_data_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_buffer_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_data_view_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_atomics_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_typed_array_constructor_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_typed_array_prototype_builtin(context, entry, invocation)
}

fn dispatch_buffer_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_array_buffer_builtin() {
        return array_buffer_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_buffer_is_view_builtin() {
        return array_buffer_is_view_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_buffer_byte_length_getter_builtin() {
        return array_buffer_byte_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_buffer_slice_builtin() {
        return array_buffer_slice_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_shared_array_buffer_builtin() {
        return shared_array_buffer_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_shared_array_buffer_byte_length_getter_builtin() {
        return shared_array_buffer_byte_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_shared_array_buffer_slice_builtin() {
        return shared_array_buffer_slice_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_data_view_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_data_view_builtin() {
        return data_view_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_buffer_getter_builtin() {
        return data_view_buffer_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_byte_length_getter_builtin() {
        return data_view_byte_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_byte_offset_getter_builtin() {
        return data_view_byte_offset_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_get_float32_builtin() {
        return data_view_get_float32_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_get_float64_builtin() {
        return data_view_get_float64_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_get_int16_builtin() {
        return data_view_get_int16_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_get_int32_builtin() {
        return data_view_get_int32_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_get_int8_builtin() {
        return data_view_get_int8_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_get_uint16_builtin() {
        return data_view_get_uint16_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_get_uint32_builtin() {
        return data_view_get_uint32_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_get_uint8_builtin() {
        return data_view_get_uint8_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_set_float32_builtin() {
        return data_view_set_float32_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_set_float64_builtin() {
        return data_view_set_float64_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_set_int16_builtin() {
        return data_view_set_int16_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_set_int32_builtin() {
        return data_view_set_int32_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_set_int8_builtin() {
        return data_view_set_int8_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_set_uint16_builtin() {
        return data_view_set_uint16_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_set_uint32_builtin() {
        return data_view_set_uint32_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_set_uint8_builtin() {
        return data_view_set_uint8_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_atomics_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_atomics_load_builtin() {
        return atomics_load_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_store_builtin() {
        return atomics_store_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_add_builtin() {
        return atomics_add_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_sub_builtin() {
        return atomics_sub_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_and_builtin() {
        return atomics_and_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_or_builtin() {
        return atomics_or_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_xor_builtin() {
        return atomics_xor_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_exchange_builtin() {
        return atomics_exchange_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_compare_exchange_builtin() {
        return atomics_compare_exchange_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_notify_builtin() {
        return atomics_notify_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_wait_builtin() {
        return atomics_wait_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_wait_async_builtin() {
        return atomics_wait_async_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_is_lock_free_builtin() {
        return atomics_is_lock_free_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_typed_array_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_typed_array_builtin() {
        return typed_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_from_builtin() {
        return typed_array_from_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_of_builtin() {
        return typed_array_of_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_int8_array_builtin() {
        return int8_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_int16_array_builtin() {
        return int16_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_int32_array_builtin() {
        return int32_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_float32_array_builtin() {
        return float32_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_float64_array_builtin() {
        return float64_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_big_int64_array_builtin() {
        return big_int64_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_big_uint64_array_builtin() {
        return big_uint64_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint32_array_builtin() {
        return uint32_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint16_array_builtin() {
        return uint16_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_clamped_array_builtin() {
        return uint8_clamped_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_array_builtin() {
        return uint8_array_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_typed_array_prototype_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_typed_array_access_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_typed_array_iteration_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_typed_array_mutation_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_typed_array_search_builtin(context, entry, invocation)
}

fn dispatch_typed_array_access_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_uint8_array_buffer_getter_builtin() {
        return typed_array_buffer_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_array_byte_length_getter_builtin() {
        return typed_array_byte_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_array_byte_offset_getter_builtin() {
        return typed_array_byte_offset_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_array_length_getter_builtin() {
        return typed_array_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_array_values_builtin() {
        return typed_array_values_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_array_keys_builtin() {
        return typed_array_keys_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_array_entries_builtin() {
        return typed_array_entries_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_at_builtin() {
        return typed_array_at_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_to_locale_string_builtin() {
        return typed_array_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_to_string_builtin() {
        return typed_array_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_to_string_tag_getter_builtin() {
        return typed_array_to_string_tag_getter_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_typed_array_mutation_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_uint8_array_set_builtin() {
        return uint8_array_set_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_array_slice_builtin() {
        return uint8_array_slice_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_array_subarray_builtin() {
        return uint8_array_subarray_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_every_builtin() {
        return typed_array_every_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_fill_builtin() {
        return typed_array_fill_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_copy_within_builtin() {
        return typed_array_copy_within_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_reverse_builtin() {
        return typed_array_reverse_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_sort_builtin() {
        return typed_array_sort_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_to_reversed_builtin() {
        return typed_array_to_reversed_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_to_sorted_builtin() {
        return typed_array_to_sorted_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_with_builtin() {
        return typed_array_with_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_typed_array_iteration_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_typed_array_filter_builtin() {
        return typed_array_filter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_for_each_builtin() {
        return typed_array_for_each_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_map_builtin() {
        return typed_array_map_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_reduce_builtin() {
        return typed_array_reduce_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_reduce_right_builtin() {
        return typed_array_reduce_right_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_typed_array_search_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_typed_array_includes_builtin() {
        return typed_array_includes_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_index_of_builtin() {
        return typed_array_index_of_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_join_builtin() {
        return typed_array_join_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_last_index_of_builtin() {
        return typed_array_last_index_of_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_some_builtin() {
        return typed_array_some_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_find_builtin() {
        return typed_array_find_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_find_index_builtin() {
        return typed_array_find_index_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_find_last_builtin() {
        return typed_array_find_last_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_find_last_index_builtin() {
        return typed_array_find_last_index_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn allocate_array_buffer_family_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
    backing_store: lyng_js_types::BackingStoreRef,
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
        let installed =
            objects.install_array_buffer_object(object, ArrayBufferObjectData::new(backing_store));
        debug_assert!(
            installed,
            "fresh buffer object should install its backing store"
        );
        object
    }))
}

fn allocate_array_buffer_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
    backing_store: lyng_js_types::BackingStoreRef,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    allocate_array_buffer_family_object(
        cx,
        realm,
        prototype,
        backing_store,
        OrdinaryObjectData::ArrayBuffer,
    )
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
        backing_store,
        OrdinaryObjectData::SharedArrayBuffer,
    )
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

fn typed_array_biguint64_value(agent: &mut Agent, bits: u64) -> Value {
    let bigint = agent.heap_mut().mutator().alloc_bigint(
        BigIntSign::NonNegative,
        &[bits],
        AllocationLifetime::Default,
    );
    Value::from_bigint_ref(bigint)
}

fn typed_array_bigint64_value(agent: &mut Agent, bits: u64) -> Value {
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

fn typed_array_storage_bits_to_value(
    agent: &mut Agent,
    kind: TypedArrayElementKind,
    bits: u64,
) -> Value {
    match kind {
        TypedArrayElementKind::BigInt64 => typed_array_bigint64_value(agent, bits),
        TypedArrayElementKind::BigUint64 => typed_array_biguint64_value(agent, bits),
        TypedArrayElementKind::Int8 => Value::from_smi(i32::from((bits as u8) as i8)),
        TypedArrayElementKind::Int16 => Value::from_smi(i32::from((bits as u16) as i16)),
        TypedArrayElementKind::Int32 => Value::from_smi(bits as u32 as i32),
        TypedArrayElementKind::Float32 => Value::from_f64(f64::from(f32::from_bits(bits as u32))),
        TypedArrayElementKind::Float64 => Value::from_f64(f64::from_bits(bits)),
        TypedArrayElementKind::Uint32 => {
            let value = bits as u32;
            i32::try_from(value).map_or_else(|_| Value::from_f64(f64::from(value)), Value::from_smi)
        }
        TypedArrayElementKind::Uint16 => Value::from_smi(i32::from(bits as u16)),
        TypedArrayElementKind::Uint8Clamped | TypedArrayElementKind::Uint8 => {
            Value::from_smi(i32::from(bits as u8))
        }
    }
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

pub(super) fn typed_array_storage_bits_from_builtin_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: TypedArrayElementKind,
    value: Value,
) -> Result<u64, Cx::Error> {
    match kind {
        TypedArrayElementKind::BigInt64 => {
            let bigint = to_bigint_for_builtin(cx, value)?;
            bigint_to_uint64_bits(cx.agent(), bigint).ok_or_else(|| type_error(cx))
        }
        TypedArrayElementKind::BigUint64 => {
            let bigint = to_bigint_for_builtin(cx, value)?;
            bigint_to_uint64_bits(cx.agent(), bigint).ok_or_else(|| type_error(cx))
        }
        TypedArrayElementKind::Int8 | TypedArrayElementKind::Uint8 => {
            Ok(u64::from(to_uint8_for_builtin(cx, value)?))
        }
        TypedArrayElementKind::Uint8Clamped => {
            Ok(u64::from(to_uint8_clamp_for_builtin(cx, value)?))
        }
        TypedArrayElementKind::Int16 | TypedArrayElementKind::Uint16 => {
            Ok(u64::from(to_uint32_for_builtin(cx, value)? as u16))
        }
        TypedArrayElementKind::Float32 => Ok(u64::from(f32::to_bits(to_number_for_builtin(
            cx, value,
        )? as f32))),
        TypedArrayElementKind::Float64 => Ok(to_number_for_builtin(cx, value)?.to_bits()),
        TypedArrayElementKind::Int32 | TypedArrayElementKind::Uint32 => {
            Ok(u64::from(to_uint32_for_builtin(cx, value)?))
        }
    }
}

fn typed_array_read_storage_bits(
    agent: &Agent,
    typed_array: TypedArrayObjectData,
    element_index: usize,
) -> Option<u64> {
    let element_size = typed_array.kind().bytes_per_element();
    let start = typed_array
        .byte_offset()
        .checked_add(element_index.checked_mul(element_size)?)?;
    let mut bits = 0_u64;
    for offset in 0..element_size {
        let byte_index = start.checked_add(offset)?;
        let byte = agent.backing_store_get_byte(typed_array.backing_store(), byte_index)?;
        bits |= u64::from(byte) << (offset * 8);
    }
    Some(bits)
}

pub(super) fn typed_array_write_storage_bits<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    record: TypedArrayObjectData,
    element_index: usize,
    bits: u64,
) -> Result<(), Cx::Error> {
    let element_size = record.kind().bytes_per_element();
    let start = record
        .byte_offset()
        .checked_add(
            element_index
                .checked_mul(element_size)
                .ok_or_else(|| range_error(cx))?,
        )
        .ok_or_else(|| range_error(cx))?;
    for offset in 0..element_size {
        let byte_index = start.checked_add(offset).ok_or_else(|| range_error(cx))?;
        let shift = offset * 8;
        let byte = u8::try_from((bits >> shift) & 0xff).expect("element byte should fit");
        if !cx
            .agent()
            .backing_store_set_byte(record.backing_store(), byte_index, byte)
        {
            return Err(range_error(cx));
        }
    }
    Ok(())
}

fn allocate_typed_array_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
    typed_array: TypedArrayObjectData,
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
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::TypedArray(
                    typed_array.kind(),
                ))),
            AllocationLifetime::Default,
        );
        let installed = objects.install_typed_array_object(object, typed_array);
        debug_assert!(
            installed,
            "fresh typed array should install its view record"
        );
        object
    }))
}

fn array_buffer_this_store<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<lyng_js_types::BackingStoreRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    cx.agent()
        .objects()
        .array_buffer(object)
        .map(ArrayBufferObjectData::backing_store)
        .ok_or_else(|| type_error(cx))
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

fn typed_array_this_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TypedArrayObjectData, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    cx.agent()
        .objects()
        .typed_array(object)
        .ok_or_else(|| type_error(cx))
}

fn typed_array_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if cx.agent().objects().typed_array(object).is_none() {
        return Err(type_error(cx));
    }
    Ok(object)
}

fn typed_array_validated_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TypedArrayObjectData, Cx::Error> {
    let record = typed_array_this_record(cx, value)?;
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    Ok(record)
}

pub(super) fn typed_array_validated_object_and_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<(ObjectRef, TypedArrayObjectData), Cx::Error> {
    let object = typed_array_this_object(cx, value)?;
    let record = typed_array_validated_record(cx, value)?;
    Ok((object, record))
}

fn typed_array_default_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    kind: TypedArrayElementKind,
) -> Result<ObjectRef, Cx::Error> {
    let getter: fn(lyng_js_env::Intrinsics) -> Option<ObjectRef> = match kind {
        TypedArrayElementKind::Int8 => lyng_js_env::Intrinsics::int8_array_prototype,
        TypedArrayElementKind::Int16 => lyng_js_env::Intrinsics::int16_array_prototype,
        TypedArrayElementKind::Int32 => lyng_js_env::Intrinsics::int32_array_prototype,
        TypedArrayElementKind::Float32 => lyng_js_env::Intrinsics::float32_array_prototype,
        TypedArrayElementKind::Float64 => lyng_js_env::Intrinsics::float64_array_prototype,
        TypedArrayElementKind::BigInt64 => lyng_js_env::Intrinsics::big_int64_array_prototype,
        TypedArrayElementKind::BigUint64 => lyng_js_env::Intrinsics::big_uint64_array_prototype,
        TypedArrayElementKind::Uint32 => lyng_js_env::Intrinsics::uint32_array_prototype,
        TypedArrayElementKind::Uint16 => lyng_js_env::Intrinsics::uint16_array_prototype,
        TypedArrayElementKind::Uint8Clamped => {
            lyng_js_env::Intrinsics::uint8_clamped_array_prototype
        }
        TypedArrayElementKind::Uint8 => lyng_js_env::Intrinsics::uint8_array_prototype,
    };
    let prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .map(lyng_js_env::RealmRecord::intrinsics)
            .and_then(getter)
    };
    prototype.ok_or_else(|| type_error(cx))
}

fn typed_array_default_constructor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    kind: TypedArrayElementKind,
) -> Result<ObjectRef, Cx::Error> {
    let getter: fn(lyng_js_env::Intrinsics) -> Option<ObjectRef> = match kind {
        TypedArrayElementKind::Int8 => lyng_js_env::Intrinsics::int8_array,
        TypedArrayElementKind::Int16 => lyng_js_env::Intrinsics::int16_array,
        TypedArrayElementKind::Int32 => lyng_js_env::Intrinsics::int32_array,
        TypedArrayElementKind::Float32 => lyng_js_env::Intrinsics::float32_array,
        TypedArrayElementKind::Float64 => lyng_js_env::Intrinsics::float64_array,
        TypedArrayElementKind::BigInt64 => lyng_js_env::Intrinsics::big_int64_array,
        TypedArrayElementKind::BigUint64 => lyng_js_env::Intrinsics::big_uint64_array,
        TypedArrayElementKind::Uint32 => lyng_js_env::Intrinsics::uint32_array,
        TypedArrayElementKind::Uint16 => lyng_js_env::Intrinsics::uint16_array,
        TypedArrayElementKind::Uint8Clamped => lyng_js_env::Intrinsics::uint8_clamped_array,
        TypedArrayElementKind::Uint8 => lyng_js_env::Intrinsics::uint8_array,
    };
    let constructor = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .map(lyng_js_env::RealmRecord::intrinsics)
            .and_then(getter)
    };
    constructor.ok_or_else(|| type_error(cx))
}

fn typed_array_species_constructor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    exemplar: ObjectRef,
    kind: TypedArrayElementKind,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    let default_constructor = typed_array_default_constructor(cx, realm, kind)?;
    let constructor = cx.get_property_value(
        Value::from_object_ref(exemplar),
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

fn typed_array_species_create_with_arguments<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    exemplar: ObjectRef,
    kind: TypedArrayElementKind,
    arguments: &[Value],
    minimum_length: Option<usize>,
) -> Result<(ObjectRef, TypedArrayObjectData), Cx::Error> {
    let constructor = typed_array_species_constructor(cx, exemplar, kind)?;
    let object = cx.construct_to_completion(constructor, arguments, None)?;
    let record = typed_array_validated_record(cx, Value::from_object_ref(object))?;
    if let Some(length) = minimum_length {
        if record.length() < length {
            return Err(type_error(cx));
        }
    }
    Ok((object, record))
}

fn typed_array_species_create<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    exemplar: ObjectRef,
    kind: TypedArrayElementKind,
    length: usize,
) -> Result<(ObjectRef, TypedArrayObjectData), Cx::Error> {
    let arguments = [length_value_u64(u64::try_from(length).unwrap_or(u64::MAX))];
    typed_array_species_create_with_arguments(cx, exemplar, kind, &arguments, Some(length))
}

fn typed_array_same_kind_create<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: TypedArrayElementKind,
    length: usize,
) -> Result<(ObjectRef, TypedArrayObjectData), Cx::Error> {
    let constructor = typed_array_default_constructor(cx, cx.builtin_realm(), kind)?;
    let arguments = [length_value_u64(u64::try_from(length).unwrap_or(u64::MAX))];
    let object = cx.construct_to_completion(constructor, &arguments, None)?;
    let record = typed_array_validated_record(cx, Value::from_object_ref(object))?;
    if record.kind() != kind || record.length() != length {
        return Err(type_error(cx));
    }
    Ok((object, record))
}

fn typed_array_snapshot_storage_bits(agent: &Agent, record: TypedArrayObjectData) -> Vec<u64> {
    (0..record.length())
        .map(|index| typed_array_read_storage_bits(agent, record, index).unwrap_or(0))
        .collect()
}

fn compare_typed_array_float_values(left: f64, right: f64) -> std::cmp::Ordering {
    if left.is_nan() {
        return if right.is_nan() {
            std::cmp::Ordering::Equal
        } else {
            std::cmp::Ordering::Greater
        };
    }
    if right.is_nan() {
        return std::cmp::Ordering::Less;
    }
    if left < right {
        return std::cmp::Ordering::Less;
    }
    if left > right {
        return std::cmp::Ordering::Greater;
    }
    if left == 0.0 && right == 0.0 {
        return match (left.is_sign_negative(), right.is_sign_negative()) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        };
    }
    std::cmp::Ordering::Equal
}

fn compare_typed_array_default_elements(
    kind: TypedArrayElementKind,
    left_bits: u64,
    right_bits: u64,
) -> std::cmp::Ordering {
    match kind {
        TypedArrayElementKind::BigInt64 => (left_bits as i64).cmp(&(right_bits as i64)),
        TypedArrayElementKind::BigUint64 => left_bits.cmp(&right_bits),
        TypedArrayElementKind::Int8 => (left_bits as u8 as i8).cmp(&(right_bits as u8 as i8)),
        TypedArrayElementKind::Int16 => (left_bits as u16 as i16).cmp(&(right_bits as u16 as i16)),
        TypedArrayElementKind::Int32 => (left_bits as u32 as i32).cmp(&(right_bits as u32 as i32)),
        TypedArrayElementKind::Uint8 | TypedArrayElementKind::Uint8Clamped => {
            (left_bits as u8).cmp(&(right_bits as u8))
        }
        TypedArrayElementKind::Uint16 => (left_bits as u16).cmp(&(right_bits as u16)),
        TypedArrayElementKind::Uint32 => (left_bits as u32).cmp(&(right_bits as u32)),
        TypedArrayElementKind::Float32 => compare_typed_array_float_values(
            f64::from(f32::from_bits(left_bits as u32)),
            f64::from(f32::from_bits(right_bits as u32)),
        ),
        TypedArrayElementKind::Float64 => {
            compare_typed_array_float_values(f64::from_bits(left_bits), f64::from_bits(right_bits))
        }
    }
}

fn compare_typed_array_sort_elements<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: TypedArrayElementKind,
    compare_fn: Option<lyng_js_types::ObjectRef>,
    left_bits: u64,
    right_bits: u64,
) -> Result<std::cmp::Ordering, Cx::Error> {
    if let Some(compare_fn) = compare_fn {
        let left = typed_array_storage_bits_to_value(cx.agent(), kind, left_bits);
        let right = typed_array_storage_bits_to_value(cx.agent(), kind, right_bits);
        return arrays::compare_array_sort_values(cx, Some(compare_fn), left, right);
    }
    Ok(compare_typed_array_default_elements(
        kind, left_bits, right_bits,
    ))
}

fn typed_array_read_element_value(
    agent: &mut Agent,
    record: TypedArrayObjectData,
    index: usize,
) -> Value {
    typed_array_read_storage_bits(agent, record, index).map_or(Value::undefined(), |bits| {
        typed_array_storage_bits_to_value(agent, record.kind(), bits)
    })
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

fn typed_array_from_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn typed_array_of_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn atomics_typed_array<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    waitable: bool,
    require_shared: bool,
) -> Result<TypedArrayObjectData, Cx::Error> {
    let typed_array = invocation
        .arguments()
        .first()
        .and_then(|value| value.as_object_ref())
        .ok_or_else(|| type_error(cx))?;
    shared_memory_ops::validate_atomic_typed_array(
        cx.agent(),
        typed_array,
        waitable,
        require_shared,
    )
    .map_err(|error| match error {
        shared_memory_ops::AtomicAccessError::Type => type_error(cx),
        shared_memory_ops::AtomicAccessError::Range => range_error(cx),
    })
}

fn atomics_access_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    waitable: bool,
    require_shared: bool,
) -> Result<shared_memory_ops::AtomicAccessRecord, Cx::Error> {
    let typed_array = atomics_typed_array(cx, invocation, waitable, require_shared)?;
    let index = to_index_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let element_index = shared_memory_ops::validate_atomic_index(typed_array, index).map_err(
        |error| match error {
            shared_memory_ops::AtomicAccessError::Type => type_error(cx),
            shared_memory_ops::AtomicAccessError::Range => range_error(cx),
        },
    )?;
    Ok(shared_memory_ops::atomic_access_record(
        typed_array,
        element_index,
    ))
}

fn atomics_value_argument<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    record: shared_memory_ops::AtomicAccessRecord,
    value: Value,
) -> Result<u64, Cx::Error> {
    typed_array_storage_bits_from_builtin_value(cx, record.typed_array().kind(), value)
}

fn atomics_load_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = atomics_access_record(cx, invocation, false, false)?;
    let bits =
        shared_memory_ops::read_atomic_bits(cx.agent(), record).ok_or_else(|| type_error(cx))?;
    Ok(shared_memory_ops::atomic_value_from_bits(
        cx.agent(),
        record.typed_array().kind(),
        bits,
    ))
}

fn atomics_store_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = atomics_access_record(cx, invocation, false, false)?;
    let value = atomics_value_argument(
        cx,
        record,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let bits = shared_memory_ops::atomic_store_bits(cx.agent(), record, value)
        .ok_or_else(|| type_error(cx))?;
    Ok(shared_memory_ops::atomic_value_from_bits(
        cx.agent(),
        record.typed_array().kind(),
        bits,
    ))
}

fn atomics_rmw_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    op: shared_memory_ops::AtomicRmwOp,
) -> Result<Value, Cx::Error> {
    let record = atomics_access_record(cx, invocation, false, false)?;
    let value = atomics_value_argument(
        cx,
        record,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let bits = shared_memory_ops::atomic_rmw_bits(cx.agent(), record, value, op)
        .ok_or_else(|| type_error(cx))?;
    Ok(shared_memory_ops::atomic_value_from_bits(
        cx.agent(),
        record.typed_array().kind(),
        bits,
    ))
}

fn atomics_add_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    atomics_rmw_builtin(cx, invocation, shared_memory_ops::AtomicRmwOp::Add)
}

fn atomics_sub_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    atomics_rmw_builtin(cx, invocation, shared_memory_ops::AtomicRmwOp::Sub)
}

fn atomics_and_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    atomics_rmw_builtin(cx, invocation, shared_memory_ops::AtomicRmwOp::And)
}

fn atomics_or_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    atomics_rmw_builtin(cx, invocation, shared_memory_ops::AtomicRmwOp::Or)
}

fn atomics_xor_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    atomics_rmw_builtin(cx, invocation, shared_memory_ops::AtomicRmwOp::Xor)
}

fn atomics_exchange_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    atomics_rmw_builtin(cx, invocation, shared_memory_ops::AtomicRmwOp::Exchange)
}

fn atomics_compare_exchange_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = atomics_access_record(cx, invocation, false, false)?;
    let expected = atomics_value_argument(
        cx,
        record,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let replacement = atomics_value_argument(
        cx,
        record,
        invocation
            .arguments()
            .get(3)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let bits =
        shared_memory_ops::atomic_compare_exchange_bits(cx.agent(), record, expected, replacement)
            .ok_or_else(|| type_error(cx))?;
    Ok(shared_memory_ops::atomic_value_from_bits(
        cx.agent(),
        record.typed_array().kind(),
        bits,
    ))
}

fn atomics_notify_count<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    count: Option<Value>,
) -> Result<u32, Cx::Error> {
    let Some(count) = count.filter(|value| !value.is_undefined()) else {
        return Ok(u32::MAX);
    };
    let integer = to_integer_or_infinity_for_builtin(cx, count)?;
    if !integer.is_finite() {
        return Ok(u32::MAX);
    }
    if integer <= 0.0 {
        return Ok(0);
    }
    Ok(integer.min(f64::from(u32::MAX)) as u32)
}

fn atomics_notify_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let typed_array = atomics_typed_array(cx, invocation, true, false)?;
    let index = to_index_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let element_index = shared_memory_ops::validate_atomic_index(typed_array, index).map_err(
        |error| match error {
            shared_memory_ops::AtomicAccessError::Type => type_error(cx),
            shared_memory_ops::AtomicAccessError::Range => range_error(cx),
        },
    )?;
    let record = shared_memory_ops::atomic_access_record(typed_array, element_index);
    let count = atomics_notify_count(cx, invocation.arguments().get(2).copied())?;
    if !cx
        .agent()
        .backing_store_is_shared(record.typed_array().backing_store())
        .unwrap_or(false)
    {
        return Ok(length_value_u64(0));
    }
    if count == 0 {
        return Ok(length_value_u64(0));
    }
    let location = shared_memory_ops::wait_location(record);
    let waiters = cx.agent().wake_shared_memory_waiters(location, count);
    let mut blocking_count = 0_u32;
    for waiter in &waiters {
        match waiter.kind() {
            WaiterKind::Blocking(_) => {
                blocking_count = blocking_count.saturating_add(1);
            }
            WaiterKind::Async(record) => {
                fulfill_wait_async_promise(cx, record.promise(), "ok")?;
            }
        }
    }
    if blocking_count > 0 {
        let _ = cx.unpark_agent(&UnparkAgentRequest {
            location,
            max_count: blocking_count,
        })?;
    }
    Ok(length_value_u64(
        u64::try_from(waiters.len()).unwrap_or(u64::MAX),
    ))
}

fn atomics_wait_timeout_ns<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    timeout: Option<Value>,
) -> Result<Option<u64>, Cx::Error> {
    let Some(timeout) = timeout.filter(|value| !value.is_undefined()) else {
        return Ok(None);
    };
    let timeout_ms = to_number_for_builtin(cx, timeout)?;
    if timeout_ms.is_nan() || timeout_ms.is_infinite() && timeout_ms.is_sign_positive() {
        return Ok(None);
    }
    if timeout_ms <= 0.0 || timeout_ms.is_sign_negative() {
        return Ok(Some(0));
    }
    let timeout_ns = (timeout_ms * 1_000_000.0).min(u64::MAX as f64);
    Ok(Some(timeout_ns as u64))
}

fn wait_async_result_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    is_async: bool,
    value: Value,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().object_prototype())
        .ok_or_else(|| type_error(cx))?;
    let object = cx.allocate_ordinary_object_with_prototype(realm, Some(prototype))?;
    let async_key = property_key_from_text(cx, "async");
    let value_key = property_key_from_text(cx, "value");
    create_data_property_or_throw(cx, object, async_key, Value::from_bool(is_async))?;
    create_data_property_or_throw(cx, object, value_key, value)?;
    Ok(Value::from_object_ref(object))
}

fn fulfill_wait_async_promise<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    promise_object: ObjectRef,
    result: &str,
) -> Result<(), Cx::Error> {
    let value = string_value(cx, result);
    let completion = promise::fulfill_promise(cx.agent(), promise_object, value);
    map_completion(cx, completion)
}

fn atomics_wait_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = atomics_access_record(cx, invocation, true, true)?;
    let expected = atomics_value_argument(
        cx,
        record,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let current =
        shared_memory_ops::read_atomic_bits(cx.agent(), record).ok_or_else(|| type_error(cx))?;
    if current != expected {
        return Ok(string_value(cx, "not-equal"));
    }
    let timeout_ns = atomics_wait_timeout_ns(cx, invocation.arguments().get(3).copied())?;
    if timeout_ns == Some(0) {
        return Ok(string_value(cx, "timed-out"));
    }
    let Some(host_id) = cx.agent().host_id() else {
        return if timeout_ns.is_some() {
            Ok(string_value(cx, "timed-out"))
        } else {
            Err(type_error(cx))
        };
    };
    let location = shared_memory_ops::wait_location(record);
    let agent_id = cx.agent().id();
    let thread_id = cx.agent().bound_thread();
    let token = cx
        .agent()
        .park_shared_memory_waiter(location, ParkedAgentRecord::new(agent_id, thread_id, false))
        .ok_or_else(|| type_error(cx))?;
    let result = cx.park_agent(&ParkAgentRequest {
        agent_id: host_id,
        thread_id,
        location,
        timeout_ns,
        allow_async: false,
    })?;
    let _ = cx.agent().remove_shared_memory_waiter(location, token);
    Ok(match result.status {
        ParkAgentStatus::Parked => string_value(cx, "ok"),
        ParkAgentStatus::TimedOut | ParkAgentStatus::Interrupted => string_value(cx, "timed-out"),
    })
}

fn atomics_wait_async_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = atomics_access_record(cx, invocation, true, true)?;
    let expected = atomics_value_argument(
        cx,
        record,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let current =
        shared_memory_ops::read_atomic_bits(cx.agent(), record).ok_or_else(|| type_error(cx))?;
    if current != expected {
        let value = string_value(cx, "not-equal");
        return wait_async_result_object(cx, false, value);
    }
    let timeout_ns = atomics_wait_timeout_ns(cx, invocation.arguments().get(3).copied())?;
    if timeout_ns == Some(0) {
        let value = string_value(cx, "timed-out");
        return wait_async_result_object(cx, false, value);
    }
    let promise_constructor = promises::promise_default_constructor(cx)?;
    let capability = promises::new_promise_capability(cx, promise_constructor)?;
    let promise_object = promises::promise_capability_promise(cx, capability)?;
    if timeout_ns.is_some() {
        fulfill_wait_async_promise(cx, promise_object, "timed-out")?;
    } else {
        let location = shared_memory_ops::wait_location(record);
        let agent_id = cx.agent().id();
        let _ = cx.agent().park_async_shared_memory_waiter(
            location,
            AsyncWaiterRecord::new(agent_id, promise_object),
        );
    }
    wait_async_result_object(cx, true, Value::from_object_ref(promise_object))
}

fn atomics_is_lock_free_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let size = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let integer = to_integer_or_infinity_for_builtin(cx, size)?;
    if !integer.is_finite() || integer <= 0.0 {
        return Ok(Value::from_bool(false));
    }
    Ok(Value::from_bool(shared_memory_ops::atomics_is_lock_free(
        integer as u64,
    )))
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

fn int8_array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Int8)
}

fn int16_array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Int16)
}

fn int32_array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Int32)
}

fn float32_array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Float32)
}

fn float64_array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Float64)
}

fn big_int64_array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::BigInt64)
}

fn big_uint64_array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::BigUint64)
}

fn uint16_array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Uint16)
}

fn uint32_array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Uint32)
}

fn uint8_clamped_array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Uint8Clamped)
}

fn uint8_array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_constructor_builtin(cx, invocation, TypedArrayElementKind::Uint8)
}

fn typed_array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Err(type_error(cx))
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
    Ok(length_value_u64(
        u64::try_from(record.length()).unwrap_or(u64::MAX),
    ))
}

#[derive(Clone, Copy)]
enum TypedArrayPredicateKind {
    Every,
    Some,
    Find,
    FindIndex,
    FindLast,
    FindLastIndex,
}

fn typed_array_predicate_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: TypedArrayPredicateKind,
) -> Result<Value, Cx::Error> {
    let this_value = invocation.this_value();
    let record = typed_array_validated_record(cx, this_value)?;
    let callback = cx.require_callable_object(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let this_arg = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let mut indices: Box<dyn Iterator<Item = usize>> = match kind {
        TypedArrayPredicateKind::FindLast | TypedArrayPredicateKind::FindLastIndex => {
            Box::new((0..record.length()).rev())
        }
        _ => Box::new(0..record.length()),
    };
    for index in indices.by_ref() {
        let element = typed_array_read_element_value(cx.agent(), record, index);
        let selected = cx.call_to_completion(
            callback,
            this_arg,
            &[
                element,
                length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)),
                this_value,
            ],
        )?;
        let selected = to_boolean_for_builtin(cx, selected)?;
        match kind {
            TypedArrayPredicateKind::Every => {
                if !selected {
                    return Ok(Value::from_bool(false));
                }
            }
            TypedArrayPredicateKind::Some => {
                if selected {
                    return Ok(Value::from_bool(true));
                }
            }
            TypedArrayPredicateKind::Find | TypedArrayPredicateKind::FindLast => {
                if selected {
                    return Ok(element);
                }
            }
            TypedArrayPredicateKind::FindIndex | TypedArrayPredicateKind::FindLastIndex => {
                if selected {
                    return Ok(length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)));
                }
            }
        }
    }
    Ok(match kind {
        TypedArrayPredicateKind::Every => Value::from_bool(true),
        TypedArrayPredicateKind::Some => Value::from_bool(false),
        TypedArrayPredicateKind::Find | TypedArrayPredicateKind::FindLast => Value::undefined(),
        TypedArrayPredicateKind::FindIndex | TypedArrayPredicateKind::FindLastIndex => {
            Value::from_smi(-1)
        }
    })
}

fn typed_array_every_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_predicate_builtin(cx, invocation, TypedArrayPredicateKind::Every)
}

fn typed_array_some_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_predicate_builtin(cx, invocation, TypedArrayPredicateKind::Some)
}

fn typed_array_find_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_predicate_builtin(cx, invocation, TypedArrayPredicateKind::Find)
}

fn typed_array_find_index_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_predicate_builtin(cx, invocation, TypedArrayPredicateKind::FindIndex)
}

fn typed_array_find_last_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_predicate_builtin(cx, invocation, TypedArrayPredicateKind::FindLast)
}

fn typed_array_find_last_index_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_predicate_builtin(cx, invocation, TypedArrayPredicateKind::FindLastIndex)
}

fn typed_array_filter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (object, record) = typed_array_validated_object_and_record(cx, invocation.this_value())?;
    let callback = cx.require_callable_object(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let this_arg = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let this_value = invocation.this_value();
    let mut kept = Vec::with_capacity(record.length());
    for index in 0..record.length() {
        let value = typed_array_read_element_value(cx.agent(), record, index);
        let selected = cx.call_to_completion(
            callback,
            this_arg,
            &[
                value,
                length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)),
                this_value,
            ],
        )?;
        if to_boolean_for_builtin(cx, selected)? {
            kept.push(value);
        }
    }
    let (result_object, result_record) =
        typed_array_species_create(cx, object, record.kind(), kept.len())?;
    for (index, value) in kept.into_iter().enumerate() {
        let bits = typed_array_storage_bits_from_builtin_value(cx, result_record.kind(), value)?;
        typed_array_write_storage_bits(cx, result_record, index, bits)?;
    }
    Ok(Value::from_object_ref(result_object))
}

fn typed_array_for_each_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_validated_record(cx, invocation.this_value())?;
    let callback = cx.require_callable_object(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let this_arg = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let this_value = invocation.this_value();
    for index in 0..record.length() {
        let value = typed_array_read_element_value(cx.agent(), record, index);
        let _ = cx.call_to_completion(
            callback,
            this_arg,
            &[
                value,
                length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)),
                this_value,
            ],
        )?;
    }
    Ok(Value::undefined())
}

fn typed_array_join_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_validated_record(cx, invocation.this_value())?;
    let separator = match invocation.arguments().first().copied() {
        Some(value) if !value.is_undefined() => cx.value_to_string_text(value)?,
        _ => ",".to_owned(),
    };
    let mut text = String::new();
    for index in 0..record.length() {
        if index != 0 {
            text.push_str(&separator);
        }
        let value = typed_array_read_element_value(cx.agent(), record, index);
        if value.is_undefined() || value.is_null() {
            continue;
        }
        text.push_str(&cx.value_to_string_text(value)?);
    }
    Ok(string_value(cx, &text))
}

fn typed_array_map_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (object, record) = typed_array_validated_object_and_record(cx, invocation.this_value())?;
    let callback = cx.require_callable_object(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let this_arg = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let (result_object, result_record) =
        typed_array_species_create(cx, object, record.kind(), record.length())?;
    let this_value = invocation.this_value();
    for index in 0..record.length() {
        let value = typed_array_read_element_value(cx.agent(), record, index);
        let mapped = cx.call_to_completion(
            callback,
            this_arg,
            &[
                value,
                length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)),
                this_value,
            ],
        )?;
        let bits = typed_array_storage_bits_from_builtin_value(cx, result_record.kind(), mapped)?;
        typed_array_write_storage_bits(cx, result_record, index, bits)?;
    }
    Ok(Value::from_object_ref(result_object))
}

#[derive(Clone, Copy)]
enum TypedArrayReduceDirection {
    Forward,
    Reverse,
}

fn typed_array_reduce_common<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    direction: TypedArrayReduceDirection,
) -> Result<Value, Cx::Error> {
    let record = typed_array_validated_record(cx, invocation.this_value())?;
    let callback = cx.require_callable_object(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let this_value = invocation.this_value();
    let len = record.length();
    let mut accumulator;
    let mut next_index;
    match invocation.arguments().get(1).copied() {
        Some(initial_value) => {
            accumulator = initial_value;
            next_index = match direction {
                TypedArrayReduceDirection::Forward => Some(0),
                TypedArrayReduceDirection::Reverse => len.checked_sub(1),
            };
        }
        None => {
            if len == 0 {
                return Err(type_error(cx));
            }
            let initial_index = match direction {
                TypedArrayReduceDirection::Forward => 0,
                TypedArrayReduceDirection::Reverse => len - 1,
            };
            accumulator = typed_array_read_element_value(cx.agent(), record, initial_index);
            next_index = match direction {
                TypedArrayReduceDirection::Forward => initial_index.checked_add(1),
                TypedArrayReduceDirection::Reverse => initial_index.checked_sub(1),
            };
        }
    }

    match direction {
        TypedArrayReduceDirection::Forward => {
            while let Some(index) = next_index {
                if index >= len {
                    break;
                }
                let value = typed_array_read_element_value(cx.agent(), record, index);
                accumulator = cx.call_to_completion(
                    callback,
                    Value::undefined(),
                    &[
                        accumulator,
                        value,
                        length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)),
                        this_value,
                    ],
                )?;
                next_index = index.checked_add(1);
            }
        }
        TypedArrayReduceDirection::Reverse => {
            while let Some(index) = next_index {
                let value = typed_array_read_element_value(cx.agent(), record, index);
                accumulator = cx.call_to_completion(
                    callback,
                    Value::undefined(),
                    &[
                        accumulator,
                        value,
                        length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)),
                        this_value,
                    ],
                )?;
                next_index = index.checked_sub(1);
            }
        }
    }

    Ok(accumulator)
}

fn typed_array_reduce_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_reduce_common(cx, invocation, TypedArrayReduceDirection::Forward)
}

fn typed_array_reduce_right_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_reduce_common(cx, invocation, TypedArrayReduceDirection::Reverse)
}

fn typed_array_reverse_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_validated_record(cx, invocation.this_value())?;
    let half_len = record.length() / 2;
    let last_index = record.length().saturating_sub(1);
    for lower in 0..half_len {
        let upper = last_index - lower;
        let lower_bits = typed_array_read_storage_bits(cx.agent(), record, lower)
            .ok_or_else(|| type_error(cx))?;
        let upper_bits = typed_array_read_storage_bits(cx.agent(), record, upper)
            .ok_or_else(|| type_error(cx))?;
        typed_array_write_storage_bits(cx, record, lower, upper_bits)?;
        typed_array_write_storage_bits(cx, record, upper, lower_bits)?;
    }
    Ok(invocation.this_value())
}

fn typed_array_sort_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_validated_record(cx, invocation.this_value())?;
    let compare_fn = match invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined())
    {
        value if value.is_undefined() => None,
        value => Some(cx.require_callable_object(value)?),
    };
    let mut elements = typed_array_snapshot_storage_bits(cx.agent(), record);
    for i in 1..elements.len() {
        let mut j = i;
        while j > 0
            && compare_typed_array_sort_elements(
                cx,
                record.kind(),
                compare_fn,
                elements[j - 1],
                elements[j],
            )? == std::cmp::Ordering::Greater
        {
            elements.swap(j - 1, j);
            j -= 1;
        }
    }
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Ok(invocation.this_value());
    }
    for (index, bits) in elements.into_iter().enumerate() {
        typed_array_write_storage_bits(cx, record, index, bits)?;
    }
    Ok(invocation.this_value())
}

fn typed_array_to_locale_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_validated_record(cx, invocation.this_value())?;
    let to_locale_string_key = property_key_from_text(cx, "toLocaleString");
    let mut parts = Vec::with_capacity(record.length());
    for index in 0..record.length() {
        let value = typed_array_read_element_value(cx.agent(), record, index);
        let text = if value.is_undefined() || value.is_null() {
            String::new()
        } else {
            let method_value = cx.get_property_value(value, to_locale_string_key)?;
            let method = cx.require_callable_object(method_value)?;
            let result = cx.call_to_completion(method, value, invocation.arguments())?;
            cx.value_to_string_text(result)?
        };
        parts.push(text);
    }
    Ok(string_value(cx, &parts.join(",")))
}

fn typed_array_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let join_key = property_key_from_text(cx, "join");
    let join_value = cx.get_property_value(invocation.this_value(), join_key)?;
    let join = cx.require_callable_object(join_value)?;
    cx.call_to_completion(join, invocation.this_value(), &[])
}

fn typed_array_to_reversed_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_validated_record(cx, invocation.this_value())?;
    let length = record.length();
    let (result_object, result_record) = typed_array_same_kind_create(cx, record.kind(), length)?;
    let source = typed_array_snapshot_storage_bits(cx.agent(), record);
    for (index, bits) in source.into_iter().rev().enumerate() {
        typed_array_write_storage_bits(cx, result_record, index, bits)?;
    }
    Ok(Value::from_object_ref(result_object))
}

fn typed_array_to_sorted_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_validated_record(cx, invocation.this_value())?;
    let compare_fn = match invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined())
    {
        value if value.is_undefined() => None,
        value => Some(cx.require_callable_object(value)?),
    };
    let length = record.length();
    let (result_object, result_record) = typed_array_same_kind_create(cx, record.kind(), length)?;
    let mut elements = typed_array_snapshot_storage_bits(cx.agent(), record);
    for i in 1..elements.len() {
        let mut j = i;
        while j > 0
            && compare_typed_array_sort_elements(
                cx,
                record.kind(),
                compare_fn,
                elements[j - 1],
                elements[j],
            )? == std::cmp::Ordering::Greater
        {
            elements.swap(j - 1, j);
            j -= 1;
        }
    }
    for (index, bits) in elements.into_iter().enumerate() {
        typed_array_write_storage_bits(cx, result_record, index, bits)?;
    }
    Ok(Value::from_object_ref(result_object))
}

fn typed_array_with_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_validated_record(cx, invocation.this_value())?;
    let length = record.length();
    let relative_index = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let replacement_bits = typed_array_storage_bits_from_builtin_value(
        cx,
        record.kind(),
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let actual_index = if relative_index < 0.0 {
        length as f64 + relative_index
    } else {
        relative_index
    };
    if !actual_index.is_finite() || actual_index < 0.0 || actual_index >= length as f64 {
        return Err(range_error(cx));
    }
    let actual_index = usize::try_from(actual_index as u64).map_err(|_| range_error(cx))?;
    let (result_object, result_record) = typed_array_same_kind_create(cx, record.kind(), length)?;
    let source = typed_array_snapshot_storage_bits(cx.agent(), record);
    for (index, mut bits) in source.into_iter().enumerate() {
        if index == actual_index {
            bits = replacement_bits;
        }
        typed_array_write_storage_bits(cx, result_record, index, bits)?;
    }
    Ok(Value::from_object_ref(result_object))
}

#[derive(Clone, Copy)]
enum TypedArraySearchKind {
    Includes,
    IndexOf,
    LastIndexOf,
}

fn typed_array_search_matches<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: TypedArraySearchKind,
    search_element: Value,
    element: Value,
) -> Result<bool, Cx::Error> {
    let heap_view = cx.agent().heap().view();
    let same = match kind {
        TypedArraySearchKind::Includes => read::same_value_zero(heap_view, search_element, element),
        TypedArraySearchKind::IndexOf | TypedArraySearchKind::LastIndexOf => {
            read::is_strictly_equal(heap_view, search_element, element)
        }
    };
    map_completion(cx, same)
}

fn typed_array_search_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: TypedArraySearchKind,
) -> Result<Value, Cx::Error> {
    let record = typed_array_this_record(cx, invocation.this_value())?;
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    let length = u64::try_from(record.length()).unwrap_or(u64::MAX);
    let search_element = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if length == 0 {
        return Ok(match kind {
            TypedArraySearchKind::Includes => Value::from_bool(false),
            TypedArraySearchKind::IndexOf | TypedArraySearchKind::LastIndexOf => {
                Value::from_smi(-1)
            }
        });
    }

    match kind {
        TypedArraySearchKind::Includes | TypedArraySearchKind::IndexOf => {
            let relative_index = to_integer_or_infinity_for_builtin(
                cx,
                invocation
                    .arguments()
                    .get(1)
                    .copied()
                    .unwrap_or(Value::undefined()),
            )?;
            if relative_index == f64::INFINITY {
                return Ok(match kind {
                    TypedArraySearchKind::Includes => Value::from_bool(false),
                    TypedArraySearchKind::IndexOf => Value::from_smi(-1),
                    TypedArraySearchKind::LastIndexOf => unreachable!(),
                });
            }
            let start = if relative_index == f64::NEG_INFINITY {
                0
            } else {
                normalize_relative_index_u64(length, relative_index)
            };
            if start >= length {
                return Ok(match kind {
                    TypedArraySearchKind::Includes => Value::from_bool(false),
                    TypedArraySearchKind::IndexOf => Value::from_smi(-1),
                    TypedArraySearchKind::LastIndexOf => unreachable!(),
                });
            }
            if cx
                .agent()
                .backing_store_is_detached(record.backing_store())
                .ok_or_else(|| type_error(cx))?
            {
                return Ok(match kind {
                    TypedArraySearchKind::Includes => {
                        Value::from_bool(search_element.is_undefined())
                    }
                    TypedArraySearchKind::IndexOf => Value::from_smi(-1),
                    TypedArraySearchKind::LastIndexOf => unreachable!(),
                });
            }
            for index in start..length {
                let index = usize::try_from(index).map_err(|_| range_error(cx))?;
                let bits = typed_array_read_storage_bits(cx.agent(), record, index)
                    .ok_or_else(|| type_error(cx))?;
                let element = typed_array_storage_bits_to_value(cx.agent(), record.kind(), bits);
                if typed_array_search_matches(cx, kind, search_element, element)? {
                    return Ok(match kind {
                        TypedArraySearchKind::Includes => Value::from_bool(true),
                        TypedArraySearchKind::IndexOf => {
                            length_value_u64(u64::try_from(index).unwrap_or(u64::MAX))
                        }
                        TypedArraySearchKind::LastIndexOf => unreachable!(),
                    });
                }
            }
            Ok(match kind {
                TypedArraySearchKind::Includes => Value::from_bool(false),
                TypedArraySearchKind::IndexOf => Value::from_smi(-1),
                TypedArraySearchKind::LastIndexOf => unreachable!(),
            })
        }
        TypedArraySearchKind::LastIndexOf => {
            let relative_index = match invocation.arguments().get(1).copied() {
                Some(value) => to_integer_or_infinity_for_builtin(cx, value)?,
                None => (length.saturating_sub(1)) as f64,
            };
            if relative_index == f64::NEG_INFINITY {
                return Ok(Value::from_smi(-1));
            }
            let start = if relative_index == f64::INFINITY {
                length.saturating_sub(1)
            } else if relative_index >= 0.0 {
                (relative_index.min((length.saturating_sub(1)) as f64)) as u64
            } else {
                let computed = (length as f64) + relative_index;
                if computed < 0.0 {
                    return Ok(Value::from_smi(-1));
                }
                computed as u64
            };
            if cx
                .agent()
                .backing_store_is_detached(record.backing_store())
                .ok_or_else(|| type_error(cx))?
            {
                return Ok(Value::from_smi(-1));
            }
            let mut index = usize::try_from(start).map_err(|_| range_error(cx))?;
            loop {
                let bits = typed_array_read_storage_bits(cx.agent(), record, index)
                    .ok_or_else(|| type_error(cx))?;
                let element = typed_array_storage_bits_to_value(cx.agent(), record.kind(), bits);
                if typed_array_search_matches(cx, kind, search_element, element)? {
                    return Ok(length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)));
                }
                if index == 0 {
                    break;
                }
                index -= 1;
            }
            Ok(Value::from_smi(-1))
        }
    }
}

fn typed_array_includes_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_search_builtin(cx, invocation, TypedArraySearchKind::Includes)
}

fn typed_array_index_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_search_builtin(cx, invocation, TypedArraySearchKind::IndexOf)
}

fn typed_array_last_index_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_search_builtin(cx, invocation, TypedArraySearchKind::LastIndexOf)
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

fn typed_array_at_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_this_record(cx, invocation.this_value())?;
    let length = u64::try_from(record.length()).unwrap_or(u64::MAX);
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
    } else if relative_index.is_infinite() || relative_index.abs() > (length as f64) {
        return Ok(Value::undefined());
    }
    let element_index = usize::try_from(index).map_err(|_| range_error(cx))?;
    let bits = typed_array_read_storage_bits(cx.agent(), record, element_index)
        .ok_or_else(|| type_error(cx))?;
    Ok(typed_array_storage_bits_to_value(
        cx.agent(),
        record.kind(),
        bits,
    ))
}

fn typed_array_fill_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_this_record(cx, invocation.this_value())?;
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    let length = u64::try_from(record.length()).unwrap_or(u64::MAX);
    let relative_start = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let start = normalize_relative_index_u64(length, relative_start);
    let end = match invocation.arguments().get(2).copied() {
        Some(value) if !value.is_undefined() => {
            let relative_end = to_integer_or_infinity_for_builtin(cx, value)?;
            normalize_relative_index_u64(length, relative_end)
        }
        _ => length,
    };
    let fill_bits = typed_array_storage_bits_from_builtin_value(
        cx,
        record.kind(),
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    for index in start..end {
        let index = usize::try_from(index).map_err(|_| range_error(cx))?;
        typed_array_write_storage_bits(cx, record, index, fill_bits)?;
    }
    Ok(invocation.this_value())
}

fn typed_array_copy_within_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_this_record(cx, invocation.this_value())?;
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    let length = u64::try_from(record.length()).unwrap_or(u64::MAX);
    let relative_target = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let to = normalize_relative_index_u64(length, relative_target);
    let relative_start = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let from = normalize_relative_index_u64(length, relative_start);
    let final_index = match invocation.arguments().get(2).copied() {
        Some(value) if !value.is_undefined() => {
            let relative_end = to_integer_or_infinity_for_builtin(cx, value)?;
            normalize_relative_index_u64(length, relative_end)
        }
        _ => length,
    };
    let count = final_index
        .saturating_sub(from)
        .min(length.saturating_sub(to));
    if count == 0 {
        return Ok(invocation.this_value());
    }
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    let from_usize = usize::try_from(from).map_err(|_| range_error(cx))?;
    let to_usize = usize::try_from(to).map_err(|_| range_error(cx))?;
    let count_usize = usize::try_from(count).map_err(|_| range_error(cx))?;
    let mut copied_bits = Vec::with_capacity(count_usize);
    for offset in 0..count_usize {
        let index = from_usize
            .checked_add(offset)
            .ok_or_else(|| range_error(cx))?;
        let bits = typed_array_read_storage_bits(cx.agent(), record, index)
            .ok_or_else(|| type_error(cx))?;
        copied_bits.push(bits);
    }
    for (offset, bits) in copied_bits.into_iter().enumerate() {
        let index = to_usize
            .checked_add(offset)
            .ok_or_else(|| range_error(cx))?;
        typed_array_write_storage_bits(cx, record, index, bits)?;
    }
    Ok(invocation.this_value())
}

fn typed_array_to_string_tag_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let Some(object) = invocation.this_value().as_object_ref() else {
        return Ok(Value::undefined());
    };
    let Some(record) = cx.agent().objects().typed_array(object) else {
        return Ok(Value::undefined());
    };
    Ok(match record.kind() {
        TypedArrayElementKind::BigInt64 => string_value(cx, "BigInt64Array"),
        TypedArrayElementKind::BigUint64 => string_value(cx, "BigUint64Array"),
        TypedArrayElementKind::Int8 => string_value(cx, "Int8Array"),
        TypedArrayElementKind::Int16 => string_value(cx, "Int16Array"),
        TypedArrayElementKind::Int32 => string_value(cx, "Int32Array"),
        TypedArrayElementKind::Float32 => string_value(cx, "Float32Array"),
        TypedArrayElementKind::Float64 => string_value(cx, "Float64Array"),
        TypedArrayElementKind::Uint32 => string_value(cx, "Uint32Array"),
        TypedArrayElementKind::Uint16 => string_value(cx, "Uint16Array"),
        TypedArrayElementKind::Uint8Clamped => string_value(cx, "Uint8ClampedArray"),
        TypedArrayElementKind::Uint8 => string_value(cx, "Uint8Array"),
    })
}

fn uint8_array_set_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_this_record(cx, invocation.this_value())?;
    let source = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let offset = to_index_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let offset = usize::try_from(offset).map_err(|_| range_error(cx))?;

    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }

    if let Some(source_object) = source
        .as_object_ref()
        .filter(|object| cx.agent().objects().typed_array(*object).is_some())
    {
        let source_record = typed_array_this_record(cx, Value::from_object_ref(source_object))?;
        if cx
            .agent()
            .backing_store_is_detached(source_record.backing_store())
            .ok_or_else(|| type_error(cx))?
        {
            return Err(type_error(cx));
        }
        if offset > record.length()
            || source_record.length() > record.length().saturating_sub(offset)
        {
            return Err(range_error(cx));
        }
        let mut values = Vec::with_capacity(source_record.length());
        for index in 0..source_record.length() {
            values.push(typed_array_read_element_value(
                cx.agent(),
                source_record,
                index,
            ));
        }
        for (index, value) in values.into_iter().enumerate() {
            let bits = typed_array_storage_bits_from_builtin_value(cx, record.kind(), value)?;
            let target_index = offset.checked_add(index).ok_or_else(|| range_error(cx))?;
            typed_array_write_storage_bits(cx, record, target_index, bits)?;
        }
        return Ok(Value::undefined());
    }

    let source_object = cx.to_object_for_builtin_value(cx.builtin_realm(), source)?;
    let source_length = array_like_length_u64(cx, source_object)?;
    let source_length = usize::try_from(source_length).map_err(|_| range_error(cx))?;
    if offset > record.length() || source_length > record.length().saturating_sub(offset) {
        return Err(range_error(cx));
    }
    for index in 0..source_length {
        let key = array_like_index_property_key(cx, u64::try_from(index).unwrap_or(u64::MAX));
        let value = get_property_from_object(cx, source_object, key)?;
        let bits = typed_array_storage_bits_from_builtin_value(cx, record.kind(), value)?;
        if cx
            .agent()
            .backing_store_is_detached(record.backing_store())
            .ok_or_else(|| type_error(cx))?
        {
            continue;
        }
        let target_index = offset.checked_add(index).ok_or_else(|| range_error(cx))?;
        typed_array_write_storage_bits(cx, record, target_index, bits)?;
    }
    Ok(Value::undefined())
}

fn uint8_array_slice_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (object, record) = typed_array_validated_object_and_record(cx, invocation.this_value())?;
    let source_length = u64::try_from(record.length()).unwrap_or(u64::MAX);
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
    let new_end = end.max(start);
    let length = usize::try_from(new_end.saturating_sub(start)).map_err(|_| range_error(cx))?;
    let start_index = usize::try_from(start).map_err(|_| range_error(cx))?;
    let (result_object, result_record) =
        typed_array_species_create(cx, object, record.kind(), length)?;
    if length > 0
        && cx
            .agent()
            .backing_store_is_detached(record.backing_store())
            .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    for offset in 0..length {
        let source_index = start_index
            .checked_add(offset)
            .ok_or_else(|| range_error(cx))?;
        let value = typed_array_read_element_value(cx.agent(), record, source_index);
        let bits = typed_array_storage_bits_from_builtin_value(cx, result_record.kind(), value)?;
        typed_array_write_storage_bits(cx, result_record, offset, bits)?;
    }
    Ok(Value::from_object_ref(result_object))
}

fn uint8_array_subarray_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = typed_array_this_object(cx, invocation.this_value())?;
    let record = typed_array_this_record(cx, invocation.this_value())?;
    let source_length = u64::try_from(record.length()).unwrap_or(u64::MAX);
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
    let new_end = end.max(start);
    let byte_offset = record
        .byte_offset()
        .checked_add(
            usize::try_from(start)
                .map_err(|_| range_error(cx))?
                .checked_mul(record.kind().bytes_per_element())
                .ok_or_else(|| range_error(cx))?,
        )
        .ok_or_else(|| range_error(cx))?;
    let length = usize::try_from(new_end.saturating_sub(start)).map_err(|_| range_error(cx))?;
    let arguments = [
        Value::from_object_ref(record.viewed_array_buffer()),
        length_value_u64(u64::try_from(byte_offset).unwrap_or(u64::MAX)),
        length_value_u64(u64::try_from(length).unwrap_or(u64::MAX)),
    ];
    let (result_object, _) =
        typed_array_species_create_with_arguments(cx, object, record.kind(), &arguments, None)?;
    Ok(Value::from_object_ref(result_object))
}

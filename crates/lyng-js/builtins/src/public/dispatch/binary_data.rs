use super::PublicBuiltinDispatchContext;
use crate::BuiltinInvocation;
use lyng_js_types::{BuiltinFunctionId, Value};

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
        return super::array_buffer_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_buffer_is_view_builtin() {
        return super::array_buffer_is_view_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_buffer_byte_length_getter_builtin() {
        return super::array_buffer_byte_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_buffer_slice_builtin() {
        return super::array_buffer_slice_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_shared_array_buffer_builtin() {
        return super::shared_array_buffer_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_shared_array_buffer_byte_length_getter_builtin() {
        return super::shared_array_buffer_byte_length_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == super::js3_shared_array_buffer_slice_builtin() {
        return super::shared_array_buffer_slice_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_data_view_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_data_view_builtin() {
        return super::data_view_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_buffer_getter_builtin() {
        return super::data_view_buffer_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_byte_length_getter_builtin() {
        return super::data_view_byte_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_byte_offset_getter_builtin() {
        return super::data_view_byte_offset_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_get_float32_builtin() {
        return super::data_view_get_float32_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_get_float64_builtin() {
        return super::data_view_get_float64_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_get_int16_builtin() {
        return super::data_view_get_int16_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_get_int32_builtin() {
        return super::data_view_get_int32_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_get_int8_builtin() {
        return super::data_view_get_int8_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_get_uint16_builtin() {
        return super::data_view_get_uint16_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_get_uint32_builtin() {
        return super::data_view_get_uint32_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_get_uint8_builtin() {
        return super::data_view_get_uint8_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_set_float32_builtin() {
        return super::data_view_set_float32_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_set_float64_builtin() {
        return super::data_view_set_float64_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_set_int16_builtin() {
        return super::data_view_set_int16_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_set_int32_builtin() {
        return super::data_view_set_int32_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_set_int8_builtin() {
        return super::data_view_set_int8_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_set_uint16_builtin() {
        return super::data_view_set_uint16_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_set_uint32_builtin() {
        return super::data_view_set_uint32_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_data_view_set_uint8_builtin() {
        return super::data_view_set_uint8_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_atomics_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_atomics_load_builtin() {
        return super::atomics_load_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_store_builtin() {
        return super::atomics_store_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_add_builtin() {
        return super::atomics_add_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_sub_builtin() {
        return super::atomics_sub_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_and_builtin() {
        return super::atomics_and_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_or_builtin() {
        return super::atomics_or_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_xor_builtin() {
        return super::atomics_xor_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_exchange_builtin() {
        return super::atomics_exchange_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_compare_exchange_builtin() {
        return super::atomics_compare_exchange_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_notify_builtin() {
        return super::atomics_notify_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_wait_builtin() {
        return super::atomics_wait_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_wait_async_builtin() {
        return super::atomics_wait_async_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_atomics_is_lock_free_builtin() {
        return super::atomics_is_lock_free_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_typed_array_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_typed_array_builtin() {
        return super::typed_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_from_builtin() {
        return super::typed_array_from_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_of_builtin() {
        return super::typed_array_of_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_int8_array_builtin() {
        return super::int8_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_int16_array_builtin() {
        return super::int16_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_int32_array_builtin() {
        return super::int32_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_float32_array_builtin() {
        return super::float32_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_float64_array_builtin() {
        return super::float64_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_big_int64_array_builtin() {
        return super::big_int64_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_big_uint64_array_builtin() {
        return super::big_uint64_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint32_array_builtin() {
        return super::uint32_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint16_array_builtin() {
        return super::uint16_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_clamped_array_builtin() {
        return super::uint8_clamped_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_array_builtin() {
        return super::uint8_array_builtin(context, invocation).map(Some);
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
        return super::typed_array_buffer_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_array_byte_length_getter_builtin() {
        return super::typed_array_byte_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_array_byte_offset_getter_builtin() {
        return super::typed_array_byte_offset_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_array_length_getter_builtin() {
        return super::typed_array_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_array_values_builtin() {
        return super::typed_array_values_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_array_keys_builtin() {
        return super::typed_array_keys_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_array_entries_builtin() {
        return super::typed_array_entries_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_at_builtin() {
        return super::typed_array_at_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_to_locale_string_builtin() {
        return super::typed_array_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_to_string_builtin() {
        return super::typed_array_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_to_string_tag_getter_builtin() {
        return super::typed_array_to_string_tag_getter_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_typed_array_mutation_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_uint8_array_set_builtin() {
        return super::uint8_array_set_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_array_slice_builtin() {
        return super::uint8_array_slice_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uint8_array_subarray_builtin() {
        return super::uint8_array_subarray_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_every_builtin() {
        return super::typed_array_every_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_fill_builtin() {
        return super::typed_array_fill_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_copy_within_builtin() {
        return super::typed_array_copy_within_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_reverse_builtin() {
        return super::typed_array_reverse_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_sort_builtin() {
        return super::typed_array_sort_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_to_reversed_builtin() {
        return super::typed_array_to_reversed_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_to_sorted_builtin() {
        return super::typed_array_to_sorted_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_with_builtin() {
        return super::typed_array_with_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_typed_array_iteration_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_typed_array_filter_builtin() {
        return super::typed_array_filter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_for_each_builtin() {
        return super::typed_array_for_each_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_map_builtin() {
        return super::typed_array_map_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_reduce_builtin() {
        return super::typed_array_reduce_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_reduce_right_builtin() {
        return super::typed_array_reduce_right_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_typed_array_search_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_typed_array_includes_builtin() {
        return super::typed_array_includes_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_index_of_builtin() {
        return super::typed_array_index_of_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_join_builtin() {
        return super::typed_array_join_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_last_index_of_builtin() {
        return super::typed_array_last_index_of_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_some_builtin() {
        return super::typed_array_some_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_find_builtin() {
        return super::typed_array_find_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_find_index_builtin() {
        return super::typed_array_find_index_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_find_last_builtin() {
        return super::typed_array_find_last_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_typed_array_find_last_index_builtin() {
        return super::typed_array_find_last_index_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

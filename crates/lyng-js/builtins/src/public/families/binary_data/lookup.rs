use crate::public::PublicRealmBuiltins;
use lyng_js_types::{
    array_buffer_builtin, array_buffer_byte_length_getter_builtin, array_buffer_is_view_builtin,
    array_buffer_slice_builtin, atomics_add_builtin, atomics_and_builtin,
    atomics_compare_exchange_builtin, atomics_exchange_builtin, atomics_is_lock_free_builtin,
    atomics_load_builtin, atomics_notify_builtin, atomics_or_builtin, atomics_pause_builtin,
    atomics_store_builtin, atomics_sub_builtin, atomics_wait_async_builtin, atomics_wait_builtin,
    atomics_xor_builtin, big_int64_array_builtin, big_uint64_array_builtin,
    data_view_buffer_getter_builtin, data_view_builtin, data_view_byte_length_getter_builtin,
    data_view_byte_offset_getter_builtin, data_view_get_big_int64_builtin,
    data_view_get_big_uint64_builtin, data_view_get_float32_builtin, data_view_get_float64_builtin,
    data_view_get_int16_builtin, data_view_get_int32_builtin, data_view_get_int8_builtin,
    data_view_get_uint16_builtin, data_view_get_uint32_builtin, data_view_get_uint8_builtin,
    data_view_set_big_int64_builtin, data_view_set_big_uint64_builtin,
    data_view_set_float32_builtin, data_view_set_float64_builtin, data_view_set_int16_builtin,
    data_view_set_int32_builtin, data_view_set_int8_builtin, data_view_set_uint16_builtin,
    data_view_set_uint32_builtin, data_view_set_uint8_builtin, float32_array_builtin,
    float64_array_builtin, int16_array_builtin, int32_array_builtin, int8_array_builtin,
    shared_array_buffer_builtin, shared_array_buffer_byte_length_getter_builtin,
    shared_array_buffer_slice_builtin, typed_array_at_builtin, typed_array_builtin,
    typed_array_copy_within_builtin, typed_array_every_builtin, typed_array_fill_builtin,
    typed_array_filter_builtin, typed_array_find_builtin, typed_array_find_index_builtin,
    typed_array_find_last_builtin, typed_array_find_last_index_builtin,
    typed_array_for_each_builtin, typed_array_from_builtin, typed_array_includes_builtin,
    typed_array_index_of_builtin, typed_array_join_builtin, typed_array_last_index_of_builtin,
    typed_array_map_builtin, typed_array_of_builtin, typed_array_reduce_builtin,
    typed_array_reduce_right_builtin, typed_array_reverse_builtin, typed_array_some_builtin,
    typed_array_sort_builtin, typed_array_to_locale_string_builtin,
    typed_array_to_reversed_builtin, typed_array_to_sorted_builtin, typed_array_to_string_builtin,
    typed_array_to_string_tag_getter_builtin, typed_array_with_builtin, uint16_array_builtin,
    uint32_array_builtin, uint8_array_buffer_getter_builtin, uint8_array_builtin,
    uint8_array_byte_length_getter_builtin, uint8_array_byte_offset_getter_builtin,
    uint8_array_entries_builtin, uint8_array_keys_builtin, uint8_array_length_getter_builtin,
    uint8_array_set_builtin, uint8_array_slice_builtin, uint8_array_subarray_builtin,
    uint8_array_values_builtin, uint8_clamped_array_builtin, BuiltinFunctionId, ObjectRef,
};

#[allow(clippy::too_many_lines)]
pub(in crate::public) fn binary_data_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    if entry == array_buffer_builtin() {
        return Some(builtins.array_buffer);
    }
    if entry == array_buffer_is_view_builtin() {
        return Some(builtins.array_buffer_is_view);
    }
    if entry == shared_array_buffer_builtin() {
        return Some(builtins.shared_array_buffer);
    }
    if entry == data_view_builtin() {
        return Some(builtins.data_view);
    }
    if entry == typed_array_builtin() {
        return Some(builtins.typed_array);
    }
    if entry == typed_array_from_builtin() {
        return Some(builtins.typed_array_from);
    }
    if entry == typed_array_of_builtin() {
        return Some(builtins.typed_array_of);
    }
    if entry == int8_array_builtin() {
        return Some(builtins.int8_array);
    }
    if entry == int16_array_builtin() {
        return Some(builtins.int16_array);
    }
    if entry == int32_array_builtin() {
        return Some(builtins.int32_array);
    }
    if entry == float32_array_builtin() {
        return Some(builtins.float32_array);
    }
    if entry == float64_array_builtin() {
        return Some(builtins.float64_array);
    }
    if entry == big_int64_array_builtin() {
        return Some(builtins.big_int64_array);
    }
    if entry == big_uint64_array_builtin() {
        return Some(builtins.big_uint64_array);
    }
    if entry == uint32_array_builtin() {
        return Some(builtins.uint32_array);
    }
    if entry == uint16_array_builtin() {
        return Some(builtins.uint16_array);
    }
    if entry == uint8_clamped_array_builtin() {
        return Some(builtins.uint8_clamped_array);
    }
    if entry == uint8_array_builtin() {
        return Some(builtins.uint8_array);
    }
    if entry == array_buffer_byte_length_getter_builtin() {
        return Some(builtins.array_buffer_byte_length_getter);
    }
    if entry == array_buffer_slice_builtin() {
        return Some(builtins.array_buffer_slice);
    }
    if entry == shared_array_buffer_byte_length_getter_builtin() {
        return Some(builtins.shared_array_buffer_byte_length_getter);
    }
    if entry == shared_array_buffer_slice_builtin() {
        return Some(builtins.shared_array_buffer_slice);
    }
    if entry == atomics_load_builtin() {
        return Some(builtins.atomics_load);
    }
    if entry == atomics_store_builtin() {
        return Some(builtins.atomics_store);
    }
    if entry == atomics_add_builtin() {
        return Some(builtins.atomics_add);
    }
    if entry == atomics_sub_builtin() {
        return Some(builtins.atomics_sub);
    }
    if entry == atomics_and_builtin() {
        return Some(builtins.atomics_and);
    }
    if entry == atomics_or_builtin() {
        return Some(builtins.atomics_or);
    }
    if entry == atomics_xor_builtin() {
        return Some(builtins.atomics_xor);
    }
    if entry == atomics_exchange_builtin() {
        return Some(builtins.atomics_exchange);
    }
    if entry == atomics_compare_exchange_builtin() {
        return Some(builtins.atomics_compare_exchange);
    }
    if entry == atomics_notify_builtin() {
        return Some(builtins.atomics_notify);
    }
    if entry == atomics_wait_builtin() {
        return Some(builtins.atomics_wait);
    }
    if entry == atomics_wait_async_builtin() {
        return Some(builtins.atomics_wait_async);
    }
    if entry == atomics_pause_builtin() {
        return Some(builtins.atomics_pause);
    }
    if entry == atomics_is_lock_free_builtin() {
        return Some(builtins.atomics_is_lock_free);
    }
    if entry == data_view_buffer_getter_builtin() {
        return Some(builtins.data_view_buffer_getter);
    }
    if entry == data_view_byte_length_getter_builtin() {
        return Some(builtins.data_view_byte_length_getter);
    }
    if entry == data_view_byte_offset_getter_builtin() {
        return Some(builtins.data_view_byte_offset_getter);
    }
    if entry == data_view_get_float32_builtin() {
        return Some(builtins.data_view_get_float32);
    }
    if entry == data_view_get_float64_builtin() {
        return Some(builtins.data_view_get_float64);
    }
    if entry == data_view_get_int16_builtin() {
        return Some(builtins.data_view_get_int16);
    }
    if entry == data_view_get_int32_builtin() {
        return Some(builtins.data_view_get_int32);
    }
    if entry == data_view_get_int8_builtin() {
        return Some(builtins.data_view_get_int8);
    }
    if entry == data_view_get_uint16_builtin() {
        return Some(builtins.data_view_get_uint16);
    }
    if entry == data_view_get_uint32_builtin() {
        return Some(builtins.data_view_get_uint32);
    }
    if entry == data_view_get_uint8_builtin() {
        return Some(builtins.data_view_get_uint8);
    }
    if entry == data_view_set_float32_builtin() {
        return Some(builtins.data_view_set_float32);
    }
    if entry == data_view_set_float64_builtin() {
        return Some(builtins.data_view_set_float64);
    }
    if entry == data_view_set_int16_builtin() {
        return Some(builtins.data_view_set_int16);
    }
    if entry == data_view_set_int32_builtin() {
        return Some(builtins.data_view_set_int32);
    }
    if entry == data_view_set_int8_builtin() {
        return Some(builtins.data_view_set_int8);
    }
    if entry == data_view_set_uint16_builtin() {
        return Some(builtins.data_view_set_uint16);
    }
    if entry == data_view_set_uint32_builtin() {
        return Some(builtins.data_view_set_uint32);
    }
    if entry == data_view_set_uint8_builtin() {
        return Some(builtins.data_view_set_uint8);
    }
    if entry == data_view_get_big_int64_builtin() {
        return Some(builtins.data_view_get_big_int64);
    }
    if entry == data_view_get_big_uint64_builtin() {
        return Some(builtins.data_view_get_big_uint64);
    }
    if entry == data_view_set_big_int64_builtin() {
        return Some(builtins.data_view_set_big_int64);
    }
    if entry == data_view_set_big_uint64_builtin() {
        return Some(builtins.data_view_set_big_uint64);
    }
    if entry == uint8_array_buffer_getter_builtin() {
        return Some(builtins.uint8_array_buffer_getter);
    }
    if entry == uint8_array_byte_length_getter_builtin() {
        return Some(builtins.uint8_array_byte_length_getter);
    }
    if entry == uint8_array_byte_offset_getter_builtin() {
        return Some(builtins.uint8_array_byte_offset_getter);
    }
    if entry == uint8_array_length_getter_builtin() {
        return Some(builtins.uint8_array_length_getter);
    }
    if entry == uint8_array_values_builtin() {
        return Some(builtins.uint8_array_values);
    }
    if entry == uint8_array_keys_builtin() {
        return Some(builtins.uint8_array_keys);
    }
    if entry == uint8_array_entries_builtin() {
        return Some(builtins.uint8_array_entries);
    }
    if entry == uint8_array_set_builtin() {
        return Some(builtins.uint8_array_set);
    }
    if entry == uint8_array_slice_builtin() {
        return Some(builtins.uint8_array_slice);
    }
    if entry == uint8_array_subarray_builtin() {
        return Some(builtins.uint8_array_subarray);
    }
    if entry == typed_array_every_builtin() {
        return Some(builtins.typed_array_every);
    }
    if entry == typed_array_some_builtin() {
        return Some(builtins.typed_array_some);
    }
    if entry == typed_array_find_builtin() {
        return Some(builtins.typed_array_find);
    }
    if entry == typed_array_find_index_builtin() {
        return Some(builtins.typed_array_find_index);
    }
    if entry == typed_array_find_last_builtin() {
        return Some(builtins.typed_array_find_last);
    }
    if entry == typed_array_find_last_index_builtin() {
        return Some(builtins.typed_array_find_last_index);
    }
    if entry == typed_array_fill_builtin() {
        return Some(builtins.typed_array_fill);
    }
    if entry == typed_array_copy_within_builtin() {
        return Some(builtins.typed_array_copy_within);
    }
    if entry == typed_array_filter_builtin() {
        return Some(builtins.typed_array_filter);
    }
    if entry == typed_array_for_each_builtin() {
        return Some(builtins.typed_array_for_each);
    }
    if entry == typed_array_includes_builtin() {
        return Some(builtins.typed_array_includes);
    }
    if entry == typed_array_index_of_builtin() {
        return Some(builtins.typed_array_index_of);
    }
    if entry == typed_array_join_builtin() {
        return Some(builtins.typed_array_join);
    }
    if entry == typed_array_last_index_of_builtin() {
        return Some(builtins.typed_array_last_index_of);
    }
    if entry == typed_array_map_builtin() {
        return Some(builtins.typed_array_map);
    }
    if entry == typed_array_reduce_builtin() {
        return Some(builtins.typed_array_reduce);
    }
    if entry == typed_array_reduce_right_builtin() {
        return Some(builtins.typed_array_reduce_right);
    }
    if entry == typed_array_reverse_builtin() {
        return Some(builtins.typed_array_reverse);
    }
    if entry == typed_array_sort_builtin() {
        return Some(builtins.typed_array_sort);
    }
    if entry == typed_array_to_locale_string_builtin() {
        return Some(builtins.typed_array_to_locale_string);
    }
    if entry == typed_array_to_string_builtin() {
        return Some(builtins.typed_array_to_string);
    }
    if entry == typed_array_to_reversed_builtin() {
        return Some(builtins.typed_array_to_reversed);
    }
    if entry == typed_array_to_sorted_builtin() {
        return Some(builtins.typed_array_to_sorted);
    }
    if entry == typed_array_with_builtin() {
        return Some(builtins.typed_array_with);
    }
    if entry == typed_array_at_builtin() {
        return Some(builtins.typed_array_at);
    }
    if entry == typed_array_to_string_tag_getter_builtin() {
        return Some(builtins.typed_array_to_string_tag_getter);
    }
    None
}

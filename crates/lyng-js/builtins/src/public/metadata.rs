use super::temporal;
use crate::BuiltinEntryMetadata;
use lyng_js_types::{
    js3_abstract_module_source_builtin, js3_abstract_module_source_to_string_tag_getter_builtin,
    js3_add_async_disposable_resource_builtin, js3_add_sync_disposable_resource_builtin,
    js3_aggregate_error_builtin, js3_array_at_builtin, js3_array_buffer_builtin,
    js3_array_buffer_byte_length_getter_builtin, js3_array_buffer_is_view_builtin,
    js3_array_buffer_slice_builtin, js3_array_builtin, js3_array_concat_builtin,
    js3_array_copy_within_builtin, js3_array_entries_builtin, js3_array_every_builtin,
    js3_array_fill_builtin, js3_array_filter_builtin, js3_array_find_builtin,
    js3_array_find_index_builtin, js3_array_find_last_builtin, js3_array_find_last_index_builtin,
    js3_array_flat_builtin, js3_array_flat_map_builtin, js3_array_for_each_builtin,
    js3_array_from_async_builtin, js3_array_from_builtin, js3_array_includes_builtin,
    js3_array_index_of_builtin, js3_array_is_array_builtin, js3_array_iterator_next_builtin,
    js3_array_join_builtin, js3_array_keys_builtin, js3_array_last_index_of_builtin,
    js3_array_map_builtin, js3_array_of_builtin, js3_array_pop_builtin, js3_array_push_builtin,
    js3_array_reduce_builtin, js3_array_reduce_right_builtin, js3_array_reverse_builtin,
    js3_array_shift_builtin, js3_array_slice_builtin, js3_array_some_builtin,
    js3_array_sort_builtin, js3_array_species_getter_builtin, js3_array_splice_builtin,
    js3_array_to_locale_string_builtin, js3_array_to_reversed_builtin, js3_array_to_sorted_builtin,
    js3_array_to_spliced_builtin, js3_array_to_string_builtin, js3_array_unshift_builtin,
    js3_array_values_builtin, js3_array_with_builtin, js3_async_disposable_stack_adopt_builtin,
    js3_async_disposable_stack_builtin, js3_async_disposable_stack_defer_builtin,
    js3_async_disposable_stack_dispose_async_builtin,
    js3_async_disposable_stack_disposed_getter_builtin, js3_async_disposable_stack_move_builtin,
    js3_async_disposable_stack_use_builtin, js3_async_disposal_resume_builtin,
    js3_async_function_builtin, js3_async_generator_function_builtin,
    js3_async_generator_next_builtin, js3_async_generator_return_builtin,
    js3_async_generator_throw_builtin, js3_atomics_add_builtin, js3_atomics_and_builtin,
    js3_atomics_compare_exchange_builtin, js3_atomics_exchange_builtin,
    js3_atomics_is_lock_free_builtin, js3_atomics_load_builtin, js3_atomics_notify_builtin,
    js3_atomics_or_builtin, js3_atomics_store_builtin, js3_atomics_sub_builtin,
    js3_atomics_wait_async_builtin, js3_atomics_wait_builtin, js3_atomics_xor_builtin,
    js3_big_int64_array_builtin, js3_big_uint64_array_builtin, js3_bigint_as_int_n_builtin,
    js3_bigint_as_uint_n_builtin, js3_bigint_builtin, js3_bigint_to_string_builtin,
    js3_bigint_value_of_builtin, js3_boolean_builtin, js3_boolean_to_string_builtin,
    js3_boolean_value_of_builtin, js3_create_async_disposal_scope_builtin,
    js3_create_sync_disposal_scope_builtin, js3_data_view_buffer_getter_builtin,
    js3_data_view_builtin, js3_data_view_byte_length_getter_builtin,
    js3_data_view_byte_offset_getter_builtin, js3_data_view_get_float32_builtin,
    js3_data_view_get_float64_builtin, js3_data_view_get_int16_builtin,
    js3_data_view_get_int32_builtin, js3_data_view_get_int8_builtin,
    js3_data_view_get_uint16_builtin, js3_data_view_get_uint32_builtin,
    js3_data_view_get_uint8_builtin, js3_data_view_set_float32_builtin,
    js3_data_view_set_float64_builtin, js3_data_view_set_int16_builtin,
    js3_data_view_set_int32_builtin, js3_data_view_set_int8_builtin,
    js3_data_view_set_uint16_builtin, js3_data_view_set_uint32_builtin,
    js3_data_view_set_uint8_builtin, js3_date_builtin, js3_date_get_date_builtin,
    js3_date_get_day_builtin, js3_date_get_full_year_builtin, js3_date_get_hours_builtin,
    js3_date_get_milliseconds_builtin, js3_date_get_minutes_builtin, js3_date_get_month_builtin,
    js3_date_get_seconds_builtin, js3_date_get_time_builtin, js3_date_get_timezone_offset_builtin,
    js3_date_get_utc_date_builtin, js3_date_get_utc_day_builtin,
    js3_date_get_utc_full_year_builtin, js3_date_get_utc_hours_builtin,
    js3_date_get_utc_milliseconds_builtin, js3_date_get_utc_minutes_builtin,
    js3_date_get_utc_month_builtin, js3_date_get_utc_seconds_builtin, js3_date_now_builtin,
    js3_date_parse_builtin, js3_date_set_date_builtin, js3_date_set_full_year_builtin,
    js3_date_set_hours_builtin, js3_date_set_milliseconds_builtin, js3_date_set_minutes_builtin,
    js3_date_set_month_builtin, js3_date_set_seconds_builtin, js3_date_set_time_builtin,
    js3_date_set_utc_date_builtin, js3_date_set_utc_full_year_builtin,
    js3_date_set_utc_hours_builtin, js3_date_set_utc_milliseconds_builtin,
    js3_date_set_utc_minutes_builtin, js3_date_set_utc_month_builtin,
    js3_date_set_utc_seconds_builtin, js3_date_to_date_string_builtin,
    js3_date_to_iso_string_builtin, js3_date_to_json_builtin,
    js3_date_to_locale_date_string_builtin, js3_date_to_locale_string_builtin,
    js3_date_to_locale_time_string_builtin, js3_date_to_primitive_builtin,
    js3_date_to_string_builtin, js3_date_to_temporal_instant_builtin,
    js3_date_to_time_string_builtin, js3_date_to_utc_string_builtin, js3_date_utc_builtin,
    js3_date_value_of_builtin, js3_decode_uri_builtin, js3_decode_uri_component_builtin,
    js3_disposable_stack_adopt_builtin, js3_disposable_stack_builtin,
    js3_disposable_stack_defer_builtin, js3_disposable_stack_dispose_builtin,
    js3_disposable_stack_disposed_getter_builtin, js3_disposable_stack_move_builtin,
    js3_disposable_stack_use_builtin, js3_dispose_scope_async_builtin, js3_dispose_scope_builtin,
    js3_encode_uri_builtin, js3_encode_uri_component_builtin, js3_error_builtin,
    js3_error_to_string_builtin, js3_eval_builtin, js3_eval_error_builtin,
    js3_finalization_registry_builtin, js3_finalization_registry_register_builtin,
    js3_finalization_registry_unregister_builtin, js3_float32_array_builtin,
    js3_float64_array_builtin, js3_function_apply_builtin, js3_function_bind_builtin,
    js3_function_builtin, js3_function_call_builtin, js3_function_prototype_builtin,
    js3_function_symbol_has_instance_builtin, js3_function_to_string_builtin,
    js3_generator_function_builtin, js3_generator_next_builtin, js3_generator_return_builtin,
    js3_generator_throw_builtin, js3_int16_array_builtin, js3_int32_array_builtin,
    js3_int8_array_builtin, js3_is_finite_builtin, js3_is_nan_builtin,
    js3_iterator_prototype_iterator_builtin, js3_json_is_raw_json_builtin, js3_json_parse_builtin,
    js3_json_raw_json_builtin, js3_json_stringify_builtin, js3_map_builtin, js3_map_clear_builtin,
    js3_map_delete_builtin, js3_map_entries_builtin, js3_map_for_each_builtin, js3_map_get_builtin,
    js3_map_has_builtin, js3_map_iterator_next_builtin, js3_map_keys_builtin, js3_map_set_builtin,
    js3_map_size_getter_builtin, js3_map_values_builtin, js3_math_abs_builtin,
    js3_math_acos_builtin, js3_math_acosh_builtin, js3_math_asin_builtin, js3_math_asinh_builtin,
    js3_math_atan2_builtin, js3_math_atan_builtin, js3_math_atanh_builtin, js3_math_cbrt_builtin,
    js3_math_ceil_builtin, js3_math_clz32_builtin, js3_math_cos_builtin, js3_math_cosh_builtin,
    js3_math_exp_builtin, js3_math_expm1_builtin, js3_math_f16round_builtin,
    js3_math_floor_builtin, js3_math_fround_builtin, js3_math_hypot_builtin, js3_math_imul_builtin,
    js3_math_log10_builtin, js3_math_log1p_builtin, js3_math_log2_builtin, js3_math_log_builtin,
    js3_math_max_builtin, js3_math_min_builtin, js3_math_pow_builtin, js3_math_random_builtin,
    js3_math_round_builtin, js3_math_sign_builtin, js3_math_sin_builtin, js3_math_sinh_builtin,
    js3_math_sqrt_builtin, js3_math_sum_precise_builtin, js3_math_tan_builtin,
    js3_math_tanh_builtin, js3_math_trunc_builtin, js3_number_builtin,
    js3_number_is_finite_builtin, js3_number_is_integer_builtin, js3_number_is_nan_builtin,
    js3_number_is_safe_integer_builtin, js3_number_to_exponential_builtin,
    js3_number_to_fixed_builtin, js3_number_to_locale_string_builtin,
    js3_number_to_precision_builtin, js3_number_to_string_builtin, js3_number_value_of_builtin,
    js3_object_assign_builtin, js3_object_builtin, js3_object_create_builtin,
    js3_object_define_getter_builtin, js3_object_define_properties_builtin,
    js3_object_define_property_builtin, js3_object_define_setter_builtin,
    js3_object_entries_builtin, js3_object_freeze_builtin, js3_object_from_entries_builtin,
    js3_object_get_own_property_descriptor_builtin,
    js3_object_get_own_property_descriptors_builtin, js3_object_get_own_property_names_builtin,
    js3_object_get_own_property_symbols_builtin, js3_object_get_prototype_of_builtin,
    js3_object_group_by_builtin, js3_object_has_own_builtin, js3_object_has_own_property_builtin,
    js3_object_is_builtin, js3_object_is_extensible_builtin, js3_object_is_frozen_builtin,
    js3_object_is_prototype_of_builtin, js3_object_is_sealed_builtin, js3_object_keys_builtin,
    js3_object_lookup_getter_builtin, js3_object_lookup_setter_builtin,
    js3_object_prevent_extensions_builtin, js3_object_property_is_enumerable_builtin,
    js3_object_proto_getter_builtin, js3_object_proto_setter_builtin, js3_object_seal_builtin,
    js3_object_set_prototype_of_builtin, js3_object_to_locale_string_builtin,
    js3_object_to_string_builtin, js3_object_value_of_builtin, js3_object_values_builtin,
    js3_parse_float_builtin, js3_parse_int_builtin, js3_promise_all_builtin,
    js3_promise_all_resolve_element_builtin, js3_promise_all_settled_builtin,
    js3_promise_all_settled_reject_element_builtin,
    js3_promise_all_settled_resolve_element_builtin, js3_promise_any_builtin,
    js3_promise_any_reject_element_builtin, js3_promise_builtin,
    js3_promise_capability_executor_builtin, js3_promise_catch_builtin,
    js3_promise_finally_builtin, js3_promise_finally_function_builtin, js3_promise_race_builtin,
    js3_promise_reject_builtin, js3_promise_reject_function_builtin, js3_promise_resolve_builtin,
    js3_promise_resolve_function_builtin, js3_promise_species_getter_builtin,
    js3_promise_then_builtin, js3_proxy_builtin, js3_proxy_revocable_builtin,
    js3_proxy_revoke_builtin, js3_range_error_builtin, js3_reference_error_builtin,
    js3_reflect_apply_builtin, js3_reflect_construct_builtin, js3_reflect_define_property_builtin,
    js3_reflect_delete_property_builtin, js3_reflect_get_builtin,
    js3_reflect_get_own_property_descriptor_builtin, js3_reflect_get_prototype_of_builtin,
    js3_reflect_has_builtin, js3_reflect_is_extensible_builtin, js3_reflect_own_keys_builtin,
    js3_reflect_prevent_extensions_builtin, js3_reflect_set_builtin,
    js3_reflect_set_prototype_of_builtin, js3_regexp_builtin, js3_regexp_dot_all_getter_builtin,
    js3_regexp_escape_builtin, js3_regexp_exec_builtin, js3_regexp_flags_getter_builtin,
    js3_regexp_global_getter_builtin, js3_regexp_has_indices_getter_builtin,
    js3_regexp_ignore_case_getter_builtin, js3_regexp_multiline_getter_builtin,
    js3_regexp_source_getter_builtin, js3_regexp_species_getter_builtin,
    js3_regexp_sticky_getter_builtin, js3_regexp_symbol_match_all_builtin,
    js3_regexp_symbol_match_builtin, js3_regexp_symbol_replace_builtin,
    js3_regexp_symbol_search_builtin, js3_regexp_symbol_split_builtin, js3_regexp_test_builtin,
    js3_regexp_to_string_builtin, js3_regexp_unicode_getter_builtin, js3_set_add_builtin,
    js3_set_builtin, js3_set_clear_builtin, js3_set_delete_builtin, js3_set_entries_builtin,
    js3_set_for_each_builtin, js3_set_has_builtin, js3_set_iterator_next_builtin,
    js3_set_keys_builtin, js3_set_size_getter_builtin, js3_set_values_builtin,
    js3_shared_array_buffer_builtin, js3_shared_array_buffer_byte_length_getter_builtin,
    js3_shared_array_buffer_slice_builtin, js3_string_at_builtin, js3_string_builtin,
    js3_string_char_at_builtin, js3_string_char_code_at_builtin, js3_string_code_point_at_builtin,
    js3_string_concat_builtin, js3_string_ends_with_builtin, js3_string_from_char_code_builtin,
    js3_string_from_code_point_builtin, js3_string_includes_builtin, js3_string_index_of_builtin,
    js3_string_is_well_formed_builtin, js3_string_iterator_builtin,
    js3_string_iterator_next_builtin, js3_string_last_index_of_builtin,
    js3_string_locale_compare_builtin, js3_string_match_all_builtin, js3_string_match_builtin,
    js3_string_normalize_builtin, js3_string_pad_end_builtin, js3_string_pad_start_builtin,
    js3_string_raw_builtin, js3_string_repeat_builtin, js3_string_replace_all_builtin,
    js3_string_replace_builtin, js3_string_search_builtin, js3_string_slice_builtin,
    js3_string_split_builtin, js3_string_starts_with_builtin, js3_string_substring_builtin,
    js3_string_to_locale_lower_case_builtin, js3_string_to_locale_upper_case_builtin,
    js3_string_to_lower_case_builtin, js3_string_to_string_builtin,
    js3_string_to_upper_case_builtin, js3_string_to_well_formed_builtin, js3_string_trim_builtin,
    js3_string_trim_end_builtin, js3_string_trim_start_builtin, js3_string_value_of_builtin,
    js3_suppressed_error_builtin, js3_symbol_builtin, js3_symbol_description_getter_builtin,
    js3_symbol_for_builtin, js3_symbol_key_for_builtin, js3_symbol_to_primitive_builtin,
    js3_symbol_to_string_builtin, js3_symbol_value_of_builtin, js3_syntax_error_builtin,
    js3_type_error_builtin, js3_typed_array_at_builtin, js3_typed_array_builtin,
    js3_typed_array_copy_within_builtin, js3_typed_array_every_builtin,
    js3_typed_array_fill_builtin, js3_typed_array_filter_builtin, js3_typed_array_find_builtin,
    js3_typed_array_find_index_builtin, js3_typed_array_find_last_builtin,
    js3_typed_array_find_last_index_builtin, js3_typed_array_for_each_builtin,
    js3_typed_array_from_builtin, js3_typed_array_includes_builtin,
    js3_typed_array_index_of_builtin, js3_typed_array_join_builtin,
    js3_typed_array_last_index_of_builtin, js3_typed_array_map_builtin, js3_typed_array_of_builtin,
    js3_typed_array_reduce_builtin, js3_typed_array_reduce_right_builtin,
    js3_typed_array_reverse_builtin, js3_typed_array_some_builtin, js3_typed_array_sort_builtin,
    js3_typed_array_to_locale_string_builtin, js3_typed_array_to_reversed_builtin,
    js3_typed_array_to_sorted_builtin, js3_typed_array_to_string_builtin,
    js3_typed_array_to_string_tag_getter_builtin, js3_typed_array_with_builtin,
    js3_uint16_array_builtin, js3_uint32_array_builtin, js3_uint8_array_buffer_getter_builtin,
    js3_uint8_array_builtin, js3_uint8_array_byte_length_getter_builtin,
    js3_uint8_array_byte_offset_getter_builtin, js3_uint8_array_entries_builtin,
    js3_uint8_array_keys_builtin, js3_uint8_array_length_getter_builtin,
    js3_uint8_array_set_builtin, js3_uint8_array_slice_builtin, js3_uint8_array_subarray_builtin,
    js3_uint8_array_values_builtin, js3_uint8_clamped_array_builtin, js3_uri_error_builtin,
    js3_weak_map_builtin, js3_weak_map_delete_builtin, js3_weak_map_get_builtin,
    js3_weak_map_has_builtin, js3_weak_map_set_builtin, js3_weak_ref_builtin,
    js3_weak_ref_deref_builtin, js3_weak_set_add_builtin, js3_weak_set_builtin,
    js3_weak_set_delete_builtin, js3_weak_set_has_builtin, BuiltinFunctionId,
};

/// Compatibility metadata for the public core builtin namespace.
#[derive(Clone, Copy, Debug)]
struct PublicBuiltinMetadataRow {
    entry: fn() -> BuiltinFunctionId,
    metadata: BuiltinEntryMetadata,
}

impl PublicBuiltinMetadataRow {
    #[inline]
    const fn new(entry: fn() -> BuiltinFunctionId, metadata: BuiltinEntryMetadata) -> Self {
        Self { entry, metadata }
    }

    #[inline]
    fn metadata_for(self, entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
        if (self.entry)() == entry {
            Some(self.metadata)
        } else {
            None
        }
    }
}

const PUBLIC_OBJECT_BUILTIN_METADATA: &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        js3_object_builtin,
        BuiltinEntryMetadata::new("Object", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_create_builtin,
        BuiltinEntryMetadata::new("create", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_get_prototype_of_builtin,
        BuiltinEntryMetadata::new("getPrototypeOf", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_set_prototype_of_builtin,
        BuiltinEntryMetadata::new("setPrototypeOf", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_get_own_property_descriptor_builtin,
        BuiltinEntryMetadata::new("getOwnPropertyDescriptor", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_get_own_property_descriptors_builtin,
        BuiltinEntryMetadata::new("getOwnPropertyDescriptors", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_get_own_property_names_builtin,
        BuiltinEntryMetadata::new("getOwnPropertyNames", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_get_own_property_symbols_builtin,
        BuiltinEntryMetadata::new("getOwnPropertySymbols", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_define_properties_builtin,
        BuiltinEntryMetadata::new("defineProperties", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_define_property_builtin,
        BuiltinEntryMetadata::new("defineProperty", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_assign_builtin,
        BuiltinEntryMetadata::new("assign", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_from_entries_builtin,
        BuiltinEntryMetadata::new("fromEntries", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_group_by_builtin,
        BuiltinEntryMetadata::new("groupBy", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_prevent_extensions_builtin,
        BuiltinEntryMetadata::new("preventExtensions", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_is_extensible_builtin,
        BuiltinEntryMetadata::new("isExtensible", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_is_builtin,
        BuiltinEntryMetadata::new("is", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_seal_builtin,
        BuiltinEntryMetadata::new("seal", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_freeze_builtin,
        BuiltinEntryMetadata::new("freeze", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_is_sealed_builtin,
        BuiltinEntryMetadata::new("isSealed", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_is_frozen_builtin,
        BuiltinEntryMetadata::new("isFrozen", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_to_locale_string_builtin,
        BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_to_string_builtin,
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_value_of_builtin,
        BuiltinEntryMetadata::new("valueOf", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_has_own_property_builtin,
        BuiltinEntryMetadata::new("hasOwnProperty", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_is_prototype_of_builtin,
        BuiltinEntryMetadata::new("isPrototypeOf", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_property_is_enumerable_builtin,
        BuiltinEntryMetadata::new("propertyIsEnumerable", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_define_getter_builtin,
        BuiltinEntryMetadata::new("__defineGetter__", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_define_setter_builtin,
        BuiltinEntryMetadata::new("__defineSetter__", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_lookup_getter_builtin,
        BuiltinEntryMetadata::new("__lookupGetter__", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_lookup_setter_builtin,
        BuiltinEntryMetadata::new("__lookupSetter__", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_proto_getter_builtin,
        BuiltinEntryMetadata::new("get __proto__", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_proto_setter_builtin,
        BuiltinEntryMetadata::new("set __proto__", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_keys_builtin,
        BuiltinEntryMetadata::new("keys", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_entries_builtin,
        BuiltinEntryMetadata::new("entries", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_values_builtin,
        BuiltinEntryMetadata::new("values", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_object_has_own_builtin,
        BuiltinEntryMetadata::new("hasOwn", 2, false, false),
    ),
];

const PUBLIC_FUNCTION_BUILTIN_METADATA: &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        js3_function_builtin,
        BuiltinEntryMetadata::new("Function", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_function_prototype_builtin,
        BuiltinEntryMetadata::new("", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_function_call_builtin,
        BuiltinEntryMetadata::new("call", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_function_apply_builtin,
        BuiltinEntryMetadata::new("apply", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_function_bind_builtin,
        BuiltinEntryMetadata::new("bind", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_function_to_string_builtin,
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_function_symbol_has_instance_builtin,
        BuiltinEntryMetadata::new("[Symbol.hasInstance]", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_async_function_builtin,
        BuiltinEntryMetadata::new("AsyncFunction", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_async_generator_function_builtin,
        BuiltinEntryMetadata::new("AsyncGeneratorFunction", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_async_generator_next_builtin,
        BuiltinEntryMetadata::new("next", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_async_generator_return_builtin,
        BuiltinEntryMetadata::new("return", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_async_generator_throw_builtin,
        BuiltinEntryMetadata::new("throw", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_generator_function_builtin,
        BuiltinEntryMetadata::new("GeneratorFunction", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_generator_next_builtin,
        BuiltinEntryMetadata::new("next", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_generator_return_builtin,
        BuiltinEntryMetadata::new("return", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_generator_throw_builtin,
        BuiltinEntryMetadata::new("throw", 1, false, false),
    ),
];

const PUBLIC_ARRAY_BUILTIN_METADATA: &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        js3_array_builtin,
        BuiltinEntryMetadata::new("Array", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_from_builtin,
        BuiltinEntryMetadata::new("from", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_from_async_builtin,
        BuiltinEntryMetadata::new("fromAsync", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_of_builtin,
        BuiltinEntryMetadata::new("of", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_is_array_builtin,
        BuiltinEntryMetadata::new("isArray", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_at_builtin,
        BuiltinEntryMetadata::new("at", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_concat_builtin,
        BuiltinEntryMetadata::new("concat", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_copy_within_builtin,
        BuiltinEntryMetadata::new("copyWithin", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_fill_builtin,
        BuiltinEntryMetadata::new("fill", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_join_builtin,
        BuiltinEntryMetadata::new("join", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_pop_builtin,
        BuiltinEntryMetadata::new("pop", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_push_builtin,
        BuiltinEntryMetadata::new("push", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_shift_builtin,
        BuiltinEntryMetadata::new("shift", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_unshift_builtin,
        BuiltinEntryMetadata::new("unshift", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_every_builtin,
        BuiltinEntryMetadata::new("every", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_filter_builtin,
        BuiltinEntryMetadata::new("filter", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_flat_builtin,
        BuiltinEntryMetadata::new("flat", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_flat_map_builtin,
        BuiltinEntryMetadata::new("flatMap", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_find_builtin,
        BuiltinEntryMetadata::new("find", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_find_index_builtin,
        BuiltinEntryMetadata::new("findIndex", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_find_last_builtin,
        BuiltinEntryMetadata::new("findLast", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_find_last_index_builtin,
        BuiltinEntryMetadata::new("findLastIndex", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_for_each_builtin,
        BuiltinEntryMetadata::new("forEach", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_includes_builtin,
        BuiltinEntryMetadata::new("includes", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_index_of_builtin,
        BuiltinEntryMetadata::new("indexOf", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_map_builtin,
        BuiltinEntryMetadata::new("map", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_reduce_builtin,
        BuiltinEntryMetadata::new("reduce", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_reduce_right_builtin,
        BuiltinEntryMetadata::new("reduceRight", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_reverse_builtin,
        BuiltinEntryMetadata::new("reverse", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_slice_builtin,
        BuiltinEntryMetadata::new("slice", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_some_builtin,
        BuiltinEntryMetadata::new("some", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_last_index_of_builtin,
        BuiltinEntryMetadata::new("lastIndexOf", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_sort_builtin,
        BuiltinEntryMetadata::new("sort", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_splice_builtin,
        BuiltinEntryMetadata::new("splice", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_to_reversed_builtin,
        BuiltinEntryMetadata::new("toReversed", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_to_sorted_builtin,
        BuiltinEntryMetadata::new("toSorted", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_to_spliced_builtin,
        BuiltinEntryMetadata::new("toSpliced", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_to_string_builtin,
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_to_locale_string_builtin,
        BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_values_builtin,
        BuiltinEntryMetadata::new("values", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_keys_builtin,
        BuiltinEntryMetadata::new("keys", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_entries_builtin,
        BuiltinEntryMetadata::new("entries", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_with_builtin,
        BuiltinEntryMetadata::new("with", 2, false, false),
    ),
];

const PUBLIC_KEYED_COLLECTION_BUILTIN_METADATA: &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        js3_map_builtin,
        BuiltinEntryMetadata::new("Map", 0, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_set_builtin,
        BuiltinEntryMetadata::new("Set", 0, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_weak_map_builtin,
        BuiltinEntryMetadata::new("WeakMap", 0, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_weak_set_builtin,
        BuiltinEntryMetadata::new("WeakSet", 0, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_map_get_builtin,
        BuiltinEntryMetadata::new("get", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_map_set_builtin,
        BuiltinEntryMetadata::new("set", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_map_has_builtin,
        BuiltinEntryMetadata::new("has", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_map_delete_builtin,
        BuiltinEntryMetadata::new("delete", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_map_clear_builtin,
        BuiltinEntryMetadata::new("clear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_map_entries_builtin,
        BuiltinEntryMetadata::new("entries", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_map_values_builtin,
        BuiltinEntryMetadata::new("values", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_map_keys_builtin,
        BuiltinEntryMetadata::new("keys", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_map_for_each_builtin,
        BuiltinEntryMetadata::new("forEach", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_map_size_getter_builtin,
        BuiltinEntryMetadata::new("get size", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_set_add_builtin,
        BuiltinEntryMetadata::new("add", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_set_has_builtin,
        BuiltinEntryMetadata::new("has", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_set_delete_builtin,
        BuiltinEntryMetadata::new("delete", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_set_clear_builtin,
        BuiltinEntryMetadata::new("clear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_set_entries_builtin,
        BuiltinEntryMetadata::new("entries", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_set_values_builtin,
        BuiltinEntryMetadata::new("values", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_set_keys_builtin,
        BuiltinEntryMetadata::new("keys", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_set_for_each_builtin,
        BuiltinEntryMetadata::new("forEach", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_set_size_getter_builtin,
        BuiltinEntryMetadata::new("get size", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_weak_map_get_builtin,
        BuiltinEntryMetadata::new("get", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_weak_map_set_builtin,
        BuiltinEntryMetadata::new("set", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_weak_map_has_builtin,
        BuiltinEntryMetadata::new("has", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_weak_map_delete_builtin,
        BuiltinEntryMetadata::new("delete", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_weak_set_add_builtin,
        BuiltinEntryMetadata::new("add", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_weak_set_has_builtin,
        BuiltinEntryMetadata::new("has", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_weak_set_delete_builtin,
        BuiltinEntryMetadata::new("delete", 1, false, false),
    ),
];

const PUBLIC_WEAK_REF_BUILTIN_METADATA: &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        js3_weak_ref_builtin,
        BuiltinEntryMetadata::new("WeakRef", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_finalization_registry_builtin,
        BuiltinEntryMetadata::new("FinalizationRegistry", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_weak_ref_deref_builtin,
        BuiltinEntryMetadata::new("deref", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_finalization_registry_register_builtin,
        BuiltinEntryMetadata::new("register", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_finalization_registry_unregister_builtin,
        BuiltinEntryMetadata::new("unregister", 1, false, false),
    ),
];

const PUBLIC_BINARY_DATA_BUILTIN_METADATA: &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        js3_array_buffer_builtin,
        BuiltinEntryMetadata::new("ArrayBuffer", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_buffer_is_view_builtin,
        BuiltinEntryMetadata::new("isView", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_shared_array_buffer_builtin,
        BuiltinEntryMetadata::new("SharedArrayBuffer", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_builtin,
        BuiltinEntryMetadata::new("DataView", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_builtin,
        BuiltinEntryMetadata::new("TypedArray", 0, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_from_builtin,
        BuiltinEntryMetadata::new("from", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_of_builtin,
        BuiltinEntryMetadata::new("of", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_every_builtin,
        BuiltinEntryMetadata::new("every", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_some_builtin,
        BuiltinEntryMetadata::new("some", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_find_builtin,
        BuiltinEntryMetadata::new("find", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_find_index_builtin,
        BuiltinEntryMetadata::new("findIndex", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_find_last_builtin,
        BuiltinEntryMetadata::new("findLast", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_find_last_index_builtin,
        BuiltinEntryMetadata::new("findLastIndex", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_fill_builtin,
        BuiltinEntryMetadata::new("fill", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_copy_within_builtin,
        BuiltinEntryMetadata::new("copyWithin", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_filter_builtin,
        BuiltinEntryMetadata::new("filter", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_for_each_builtin,
        BuiltinEntryMetadata::new("forEach", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_includes_builtin,
        BuiltinEntryMetadata::new("includes", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_index_of_builtin,
        BuiltinEntryMetadata::new("indexOf", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_join_builtin,
        BuiltinEntryMetadata::new("join", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_last_index_of_builtin,
        BuiltinEntryMetadata::new("lastIndexOf", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_map_builtin,
        BuiltinEntryMetadata::new("map", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_reduce_builtin,
        BuiltinEntryMetadata::new("reduce", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_reduce_right_builtin,
        BuiltinEntryMetadata::new("reduceRight", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_reverse_builtin,
        BuiltinEntryMetadata::new("reverse", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_sort_builtin,
        BuiltinEntryMetadata::new("sort", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_to_locale_string_builtin,
        BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_to_string_builtin,
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_to_reversed_builtin,
        BuiltinEntryMetadata::new("toReversed", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_to_sorted_builtin,
        BuiltinEntryMetadata::new("toSorted", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_with_builtin,
        BuiltinEntryMetadata::new("with", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_int8_array_builtin,
        BuiltinEntryMetadata::new("Int8Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_int16_array_builtin,
        BuiltinEntryMetadata::new("Int16Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_int32_array_builtin,
        BuiltinEntryMetadata::new("Int32Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_float32_array_builtin,
        BuiltinEntryMetadata::new("Float32Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_float64_array_builtin,
        BuiltinEntryMetadata::new("Float64Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_big_int64_array_builtin,
        BuiltinEntryMetadata::new("BigInt64Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_big_uint64_array_builtin,
        BuiltinEntryMetadata::new("BigUint64Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_uint32_array_builtin,
        BuiltinEntryMetadata::new("Uint32Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_uint16_array_builtin,
        BuiltinEntryMetadata::new("Uint16Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_uint8_clamped_array_builtin,
        BuiltinEntryMetadata::new("Uint8ClampedArray", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_uint8_array_builtin,
        BuiltinEntryMetadata::new("Uint8Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_buffer_byte_length_getter_builtin,
        BuiltinEntryMetadata::new("get byteLength", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_buffer_slice_builtin,
        BuiltinEntryMetadata::new("slice", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_shared_array_buffer_byte_length_getter_builtin,
        BuiltinEntryMetadata::new("get byteLength", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_shared_array_buffer_slice_builtin,
        BuiltinEntryMetadata::new("slice", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_atomics_load_builtin,
        BuiltinEntryMetadata::new("load", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_atomics_store_builtin,
        BuiltinEntryMetadata::new("store", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_atomics_add_builtin,
        BuiltinEntryMetadata::new("add", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_atomics_sub_builtin,
        BuiltinEntryMetadata::new("sub", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_atomics_and_builtin,
        BuiltinEntryMetadata::new("and", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_atomics_or_builtin,
        BuiltinEntryMetadata::new("or", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_atomics_xor_builtin,
        BuiltinEntryMetadata::new("xor", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_atomics_exchange_builtin,
        BuiltinEntryMetadata::new("exchange", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_atomics_compare_exchange_builtin,
        BuiltinEntryMetadata::new("compareExchange", 4, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_atomics_notify_builtin,
        BuiltinEntryMetadata::new("notify", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_atomics_wait_builtin,
        BuiltinEntryMetadata::new("wait", 4, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_atomics_wait_async_builtin,
        BuiltinEntryMetadata::new("waitAsync", 4, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_atomics_is_lock_free_builtin,
        BuiltinEntryMetadata::new("isLockFree", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_buffer_getter_builtin,
        BuiltinEntryMetadata::new("get buffer", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_byte_length_getter_builtin,
        BuiltinEntryMetadata::new("get byteLength", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_byte_offset_getter_builtin,
        BuiltinEntryMetadata::new("get byteOffset", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_get_float32_builtin,
        BuiltinEntryMetadata::new("getFloat32", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_get_float64_builtin,
        BuiltinEntryMetadata::new("getFloat64", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_get_int16_builtin,
        BuiltinEntryMetadata::new("getInt16", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_get_int32_builtin,
        BuiltinEntryMetadata::new("getInt32", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_get_int8_builtin,
        BuiltinEntryMetadata::new("getInt8", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_get_uint16_builtin,
        BuiltinEntryMetadata::new("getUint16", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_get_uint32_builtin,
        BuiltinEntryMetadata::new("getUint32", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_get_uint8_builtin,
        BuiltinEntryMetadata::new("getUint8", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_set_float32_builtin,
        BuiltinEntryMetadata::new("setFloat32", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_set_float64_builtin,
        BuiltinEntryMetadata::new("setFloat64", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_set_int16_builtin,
        BuiltinEntryMetadata::new("setInt16", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_set_int32_builtin,
        BuiltinEntryMetadata::new("setInt32", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_set_int8_builtin,
        BuiltinEntryMetadata::new("setInt8", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_set_uint16_builtin,
        BuiltinEntryMetadata::new("setUint16", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_set_uint32_builtin,
        BuiltinEntryMetadata::new("setUint32", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_data_view_set_uint8_builtin,
        BuiltinEntryMetadata::new("setUint8", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_uint8_array_buffer_getter_builtin,
        BuiltinEntryMetadata::new("get buffer", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_uint8_array_byte_length_getter_builtin,
        BuiltinEntryMetadata::new("get byteLength", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_uint8_array_byte_offset_getter_builtin,
        BuiltinEntryMetadata::new("get byteOffset", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_uint8_array_length_getter_builtin,
        BuiltinEntryMetadata::new("get length", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_uint8_array_values_builtin,
        BuiltinEntryMetadata::new("values", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_uint8_array_keys_builtin,
        BuiltinEntryMetadata::new("keys", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_uint8_array_entries_builtin,
        BuiltinEntryMetadata::new("entries", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_uint8_array_set_builtin,
        BuiltinEntryMetadata::new("set", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_uint8_array_slice_builtin,
        BuiltinEntryMetadata::new("slice", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_uint8_array_subarray_builtin,
        BuiltinEntryMetadata::new("subarray", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_at_builtin,
        BuiltinEntryMetadata::new("at", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_typed_array_to_string_tag_getter_builtin,
        BuiltinEntryMetadata::new("get [Symbol.toStringTag]", 0, false, false),
    ),
];

const PUBLIC_OBJECT_REFLECTION_BUILTIN_METADATA: &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        js3_json_parse_builtin,
        BuiltinEntryMetadata::new("parse", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_json_stringify_builtin,
        BuiltinEntryMetadata::new("stringify", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_json_raw_json_builtin,
        BuiltinEntryMetadata::new("rawJSON", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_json_is_raw_json_builtin,
        BuiltinEntryMetadata::new("isRawJSON", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_reflect_apply_builtin,
        BuiltinEntryMetadata::new("apply", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_reflect_construct_builtin,
        BuiltinEntryMetadata::new("construct", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_reflect_define_property_builtin,
        BuiltinEntryMetadata::new("defineProperty", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_reflect_delete_property_builtin,
        BuiltinEntryMetadata::new("deleteProperty", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_reflect_get_builtin,
        BuiltinEntryMetadata::new("get", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_reflect_get_own_property_descriptor_builtin,
        BuiltinEntryMetadata::new("getOwnPropertyDescriptor", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_reflect_get_prototype_of_builtin,
        BuiltinEntryMetadata::new("getPrototypeOf", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_reflect_has_builtin,
        BuiltinEntryMetadata::new("has", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_reflect_is_extensible_builtin,
        BuiltinEntryMetadata::new("isExtensible", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_reflect_own_keys_builtin,
        BuiltinEntryMetadata::new("ownKeys", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_reflect_prevent_extensions_builtin,
        BuiltinEntryMetadata::new("preventExtensions", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_reflect_set_builtin,
        BuiltinEntryMetadata::new("set", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_reflect_set_prototype_of_builtin,
        BuiltinEntryMetadata::new("setPrototypeOf", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_proxy_builtin,
        BuiltinEntryMetadata::new("Proxy", 2, true, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_proxy_revocable_builtin,
        BuiltinEntryMetadata::new("revocable", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_proxy_revoke_builtin,
        BuiltinEntryMetadata::new("", 0, false, false),
    ),
];

const PUBLIC_TEXT_BUILTIN_METADATA: &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        js3_iterator_prototype_iterator_builtin,
        BuiltinEntryMetadata::new("[Symbol.iterator]", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_array_iterator_next_builtin,
        BuiltinEntryMetadata::new("next", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_map_iterator_next_builtin,
        BuiltinEntryMetadata::new("next", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_set_iterator_next_builtin,
        BuiltinEntryMetadata::new("next", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_builtin,
        BuiltinEntryMetadata::new("String", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_iterator_builtin,
        BuiltinEntryMetadata::new("[Symbol.iterator]", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_iterator_next_builtin,
        BuiltinEntryMetadata::new("next", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_to_string_builtin,
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_value_of_builtin,
        BuiltinEntryMetadata::new("valueOf", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_concat_builtin,
        BuiltinEntryMetadata::new("concat", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_char_at_builtin,
        BuiltinEntryMetadata::new("charAt", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_char_code_at_builtin,
        BuiltinEntryMetadata::new("charCodeAt", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_from_char_code_builtin,
        BuiltinEntryMetadata::new("fromCharCode", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_from_code_point_builtin,
        BuiltinEntryMetadata::new("fromCodePoint", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_raw_builtin,
        BuiltinEntryMetadata::new("raw", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_at_builtin,
        BuiltinEntryMetadata::new("at", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_code_point_at_builtin,
        BuiltinEntryMetadata::new("codePointAt", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_ends_with_builtin,
        BuiltinEntryMetadata::new("endsWith", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_includes_builtin,
        BuiltinEntryMetadata::new("includes", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_index_of_builtin,
        BuiltinEntryMetadata::new("indexOf", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_is_well_formed_builtin,
        BuiltinEntryMetadata::new("isWellFormed", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_locale_compare_builtin,
        BuiltinEntryMetadata::new("localeCompare", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_match_builtin,
        BuiltinEntryMetadata::new("match", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_match_all_builtin,
        BuiltinEntryMetadata::new("matchAll", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_normalize_builtin,
        BuiltinEntryMetadata::new("normalize", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_last_index_of_builtin,
        BuiltinEntryMetadata::new("lastIndexOf", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_pad_end_builtin,
        BuiltinEntryMetadata::new("padEnd", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_pad_start_builtin,
        BuiltinEntryMetadata::new("padStart", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_repeat_builtin,
        BuiltinEntryMetadata::new("repeat", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_replace_builtin,
        BuiltinEntryMetadata::new("replace", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_replace_all_builtin,
        BuiltinEntryMetadata::new("replaceAll", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_search_builtin,
        BuiltinEntryMetadata::new("search", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_split_builtin,
        BuiltinEntryMetadata::new("split", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_slice_builtin,
        BuiltinEntryMetadata::new("slice", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_substring_builtin,
        BuiltinEntryMetadata::new("substring", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_starts_with_builtin,
        BuiltinEntryMetadata::new("startsWith", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_to_locale_lower_case_builtin,
        BuiltinEntryMetadata::new("toLocaleLowerCase", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_to_locale_upper_case_builtin,
        BuiltinEntryMetadata::new("toLocaleUpperCase", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_to_lower_case_builtin,
        BuiltinEntryMetadata::new("toLowerCase", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_to_upper_case_builtin,
        BuiltinEntryMetadata::new("toUpperCase", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_to_well_formed_builtin,
        BuiltinEntryMetadata::new("toWellFormed", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_trim_builtin,
        BuiltinEntryMetadata::new("trim", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_trim_end_builtin,
        BuiltinEntryMetadata::new("trimEnd", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_string_trim_start_builtin,
        BuiltinEntryMetadata::new("trimStart", 0, false, false),
    ),
];

const PUBLIC_REGEXP_BUILTIN_METADATA: &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        js3_regexp_builtin,
        BuiltinEntryMetadata::new("RegExp", 2, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_escape_builtin,
        BuiltinEntryMetadata::new("escape", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_to_string_builtin,
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_exec_builtin,
        BuiltinEntryMetadata::new("exec", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_test_builtin,
        BuiltinEntryMetadata::new("test", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_global_getter_builtin,
        BuiltinEntryMetadata::new("get global", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_ignore_case_getter_builtin,
        BuiltinEntryMetadata::new("get ignoreCase", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_multiline_getter_builtin,
        BuiltinEntryMetadata::new("get multiline", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_dot_all_getter_builtin,
        BuiltinEntryMetadata::new("get dotAll", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_unicode_getter_builtin,
        BuiltinEntryMetadata::new("get unicode", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_sticky_getter_builtin,
        BuiltinEntryMetadata::new("get sticky", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_source_getter_builtin,
        BuiltinEntryMetadata::new("get source", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_flags_getter_builtin,
        BuiltinEntryMetadata::new("get flags", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_has_indices_getter_builtin,
        BuiltinEntryMetadata::new("get hasIndices", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_species_getter_builtin,
        BuiltinEntryMetadata::new("get [Symbol.species]", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_symbol_match_builtin,
        BuiltinEntryMetadata::new("[Symbol.match]", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_symbol_replace_builtin,
        BuiltinEntryMetadata::new("[Symbol.replace]", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_symbol_search_builtin,
        BuiltinEntryMetadata::new("[Symbol.search]", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_symbol_split_builtin,
        BuiltinEntryMetadata::new("[Symbol.split]", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        js3_regexp_symbol_match_all_builtin,
        BuiltinEntryMetadata::new("[Symbol.matchAll]", 1, false, false),
    ),
];

fn public_builtin_metadata_from_rows(
    entry: BuiltinFunctionId,
    rows: &[PublicBuiltinMetadataRow],
) -> Option<BuiltinEntryMetadata> {
    rows.iter().find_map(|row| row.metadata_for(entry))
}

fn object_public_builtin_metadata(entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
    public_builtin_metadata_from_rows(entry, PUBLIC_OBJECT_BUILTIN_METADATA)
}

fn function_public_builtin_metadata(entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
    public_builtin_metadata_from_rows(entry, PUBLIC_FUNCTION_BUILTIN_METADATA)
}

fn array_public_builtin_metadata(entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
    public_builtin_metadata_from_rows(entry, PUBLIC_ARRAY_BUILTIN_METADATA)
}

fn keyed_collection_public_builtin_metadata(
    entry: BuiltinFunctionId,
) -> Option<BuiltinEntryMetadata> {
    public_builtin_metadata_from_rows(entry, PUBLIC_KEYED_COLLECTION_BUILTIN_METADATA)
}

fn weak_ref_public_builtin_metadata(entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
    public_builtin_metadata_from_rows(entry, PUBLIC_WEAK_REF_BUILTIN_METADATA)
}

fn binary_data_public_builtin_metadata(entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
    public_builtin_metadata_from_rows(entry, PUBLIC_BINARY_DATA_BUILTIN_METADATA)
}

fn object_reflection_public_builtin_metadata(
    entry: BuiltinFunctionId,
) -> Option<BuiltinEntryMetadata> {
    public_builtin_metadata_from_rows(entry, PUBLIC_OBJECT_REFLECTION_BUILTIN_METADATA)
}

fn text_public_builtin_metadata(entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
    public_builtin_metadata_from_rows(entry, PUBLIC_TEXT_BUILTIN_METADATA)
}

fn regexp_public_builtin_metadata(entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
    public_builtin_metadata_from_rows(entry, PUBLIC_REGEXP_BUILTIN_METADATA)
}

/// Compatibility metadata for the public core builtin namespace.
#[inline]
pub fn public_builtin_metadata(entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
    if let Some(metadata) = object_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if let Some(metadata) = function_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if let Some(metadata) = array_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if let Some(metadata) = keyed_collection_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if let Some(metadata) = weak_ref_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if entry == js3_abstract_module_source_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "AbstractModuleSource",
            0,
            true,
            true,
        ));
    }
    if entry == js3_abstract_module_source_to_string_tag_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get [Symbol.toStringTag]",
            0,
            false,
            false,
        ));
    }
    if let Some(metadata) = binary_data_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if let Some(metadata) = object_reflection_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if let Some(metadata) = text_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if let Some(metadata) = regexp_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if entry == js3_date_builtin() {
        return Some(BuiltinEntryMetadata::new("Date", 7, true, true));
    }
    if entry == js3_date_now_builtin() {
        return Some(BuiltinEntryMetadata::new("now", 0, false, false));
    }
    if entry == js3_date_parse_builtin() {
        return Some(BuiltinEntryMetadata::new("parse", 1, false, false));
    }
    if entry == js3_date_utc_builtin() {
        return Some(BuiltinEntryMetadata::new("UTC", 7, false, false));
    }
    if entry == js3_date_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == js3_date_to_date_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toDateString", 0, false, false));
    }
    if entry == js3_date_to_time_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toTimeString", 0, false, false));
    }
    if entry == js3_date_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == js3_date_to_locale_date_string_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "toLocaleDateString",
            0,
            false,
            false,
        ));
    }
    if entry == js3_date_to_locale_time_string_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "toLocaleTimeString",
            0,
            false,
            false,
        ));
    }
    if entry == js3_date_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == js3_date_get_time_builtin() {
        return Some(BuiltinEntryMetadata::new("getTime", 0, false, false));
    }
    if entry == js3_date_get_full_year_builtin() {
        return Some(BuiltinEntryMetadata::new("getFullYear", 0, false, false));
    }
    if entry == js3_date_get_utc_full_year_builtin() {
        return Some(BuiltinEntryMetadata::new("getUTCFullYear", 0, false, false));
    }
    if entry == js3_date_get_month_builtin() {
        return Some(BuiltinEntryMetadata::new("getMonth", 0, false, false));
    }
    if entry == js3_date_get_utc_month_builtin() {
        return Some(BuiltinEntryMetadata::new("getUTCMonth", 0, false, false));
    }
    if entry == js3_date_get_date_builtin() {
        return Some(BuiltinEntryMetadata::new("getDate", 0, false, false));
    }
    if entry == js3_date_get_utc_date_builtin() {
        return Some(BuiltinEntryMetadata::new("getUTCDate", 0, false, false));
    }
    if entry == js3_date_get_day_builtin() {
        return Some(BuiltinEntryMetadata::new("getDay", 0, false, false));
    }
    if entry == js3_date_get_utc_day_builtin() {
        return Some(BuiltinEntryMetadata::new("getUTCDay", 0, false, false));
    }
    if entry == js3_date_get_hours_builtin() {
        return Some(BuiltinEntryMetadata::new("getHours", 0, false, false));
    }
    if entry == js3_date_get_utc_hours_builtin() {
        return Some(BuiltinEntryMetadata::new("getUTCHours", 0, false, false));
    }
    if entry == js3_date_get_minutes_builtin() {
        return Some(BuiltinEntryMetadata::new("getMinutes", 0, false, false));
    }
    if entry == js3_date_get_utc_minutes_builtin() {
        return Some(BuiltinEntryMetadata::new("getUTCMinutes", 0, false, false));
    }
    if entry == js3_date_get_seconds_builtin() {
        return Some(BuiltinEntryMetadata::new("getSeconds", 0, false, false));
    }
    if entry == js3_date_get_utc_seconds_builtin() {
        return Some(BuiltinEntryMetadata::new("getUTCSeconds", 0, false, false));
    }
    if entry == js3_date_get_milliseconds_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "getMilliseconds",
            0,
            false,
            false,
        ));
    }
    if entry == js3_date_get_utc_milliseconds_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "getUTCMilliseconds",
            0,
            false,
            false,
        ));
    }
    if entry == js3_date_get_timezone_offset_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "getTimezoneOffset",
            0,
            false,
            false,
        ));
    }
    if entry == js3_date_set_time_builtin() {
        return Some(BuiltinEntryMetadata::new("setTime", 1, false, false));
    }
    if entry == js3_date_set_milliseconds_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "setMilliseconds",
            1,
            false,
            false,
        ));
    }
    if entry == js3_date_set_utc_milliseconds_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "setUTCMilliseconds",
            1,
            false,
            false,
        ));
    }
    if entry == js3_date_set_seconds_builtin() {
        return Some(BuiltinEntryMetadata::new("setSeconds", 2, false, false));
    }
    if entry == js3_date_set_utc_seconds_builtin() {
        return Some(BuiltinEntryMetadata::new("setUTCSeconds", 2, false, false));
    }
    if entry == js3_date_set_minutes_builtin() {
        return Some(BuiltinEntryMetadata::new("setMinutes", 3, false, false));
    }
    if entry == js3_date_set_utc_minutes_builtin() {
        return Some(BuiltinEntryMetadata::new("setUTCMinutes", 3, false, false));
    }
    if entry == js3_date_set_hours_builtin() {
        return Some(BuiltinEntryMetadata::new("setHours", 4, false, false));
    }
    if entry == js3_date_set_utc_hours_builtin() {
        return Some(BuiltinEntryMetadata::new("setUTCHours", 4, false, false));
    }
    if entry == js3_date_set_date_builtin() {
        return Some(BuiltinEntryMetadata::new("setDate", 1, false, false));
    }
    if entry == js3_date_set_utc_date_builtin() {
        return Some(BuiltinEntryMetadata::new("setUTCDate", 1, false, false));
    }
    if entry == js3_date_set_month_builtin() {
        return Some(BuiltinEntryMetadata::new("setMonth", 2, false, false));
    }
    if entry == js3_date_set_utc_month_builtin() {
        return Some(BuiltinEntryMetadata::new("setUTCMonth", 2, false, false));
    }
    if entry == js3_date_set_full_year_builtin() {
        return Some(BuiltinEntryMetadata::new("setFullYear", 3, false, false));
    }
    if entry == js3_date_set_utc_full_year_builtin() {
        return Some(BuiltinEntryMetadata::new("setUTCFullYear", 3, false, false));
    }
    if entry == js3_date_to_utc_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toUTCString", 0, false, false));
    }
    if entry == js3_date_to_iso_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toISOString", 0, false, false));
    }
    if entry == js3_date_to_json_builtin() {
        return Some(BuiltinEntryMetadata::new("toJSON", 1, false, false));
    }
    if entry == js3_date_to_temporal_instant_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "toTemporalInstant",
            0,
            false,
            false,
        ));
    }
    if entry == js3_date_to_primitive_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "[Symbol.toPrimitive]",
            1,
            false,
            false,
        ));
    }
    if let Some(metadata) = temporal::temporal_builtin_metadata(entry) {
        return Some(metadata);
    }
    if entry == js3_number_builtin() {
        return Some(BuiltinEntryMetadata::new("Number", 1, true, true));
    }
    if entry == js3_number_is_finite_builtin() {
        return Some(BuiltinEntryMetadata::new("isFinite", 1, false, false));
    }
    if entry == js3_number_is_integer_builtin() {
        return Some(BuiltinEntryMetadata::new("isInteger", 1, false, false));
    }
    if entry == js3_number_is_nan_builtin() {
        return Some(BuiltinEntryMetadata::new("isNaN", 1, false, false));
    }
    if entry == js3_number_is_safe_integer_builtin() {
        return Some(BuiltinEntryMetadata::new("isSafeInteger", 1, false, false));
    }
    if entry == js3_number_to_exponential_builtin() {
        return Some(BuiltinEntryMetadata::new("toExponential", 1, false, false));
    }
    if entry == js3_number_to_fixed_builtin() {
        return Some(BuiltinEntryMetadata::new("toFixed", 1, false, false));
    }
    if entry == js3_number_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == js3_number_to_precision_builtin() {
        return Some(BuiltinEntryMetadata::new("toPrecision", 1, false, false));
    }
    if entry == js3_number_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 1, false, false));
    }
    if entry == js3_number_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == js3_math_abs_builtin() {
        return Some(BuiltinEntryMetadata::new("abs", 1, false, false));
    }
    if entry == js3_math_acos_builtin() {
        return Some(BuiltinEntryMetadata::new("acos", 1, false, false));
    }
    if entry == js3_math_acosh_builtin() {
        return Some(BuiltinEntryMetadata::new("acosh", 1, false, false));
    }
    if entry == js3_math_asin_builtin() {
        return Some(BuiltinEntryMetadata::new("asin", 1, false, false));
    }
    if entry == js3_math_asinh_builtin() {
        return Some(BuiltinEntryMetadata::new("asinh", 1, false, false));
    }
    if entry == js3_math_atan_builtin() {
        return Some(BuiltinEntryMetadata::new("atan", 1, false, false));
    }
    if entry == js3_math_atan2_builtin() {
        return Some(BuiltinEntryMetadata::new("atan2", 2, false, false));
    }
    if entry == js3_math_atanh_builtin() {
        return Some(BuiltinEntryMetadata::new("atanh", 1, false, false));
    }
    if entry == js3_math_cbrt_builtin() {
        return Some(BuiltinEntryMetadata::new("cbrt", 1, false, false));
    }
    if entry == js3_math_ceil_builtin() {
        return Some(BuiltinEntryMetadata::new("ceil", 1, false, false));
    }
    if entry == js3_math_clz32_builtin() {
        return Some(BuiltinEntryMetadata::new("clz32", 1, false, false));
    }
    if entry == js3_math_cos_builtin() {
        return Some(BuiltinEntryMetadata::new("cos", 1, false, false));
    }
    if entry == js3_math_cosh_builtin() {
        return Some(BuiltinEntryMetadata::new("cosh", 1, false, false));
    }
    if entry == js3_math_exp_builtin() {
        return Some(BuiltinEntryMetadata::new("exp", 1, false, false));
    }
    if entry == js3_math_expm1_builtin() {
        return Some(BuiltinEntryMetadata::new("expm1", 1, false, false));
    }
    if entry == js3_math_f16round_builtin() {
        return Some(BuiltinEntryMetadata::new("f16round", 1, false, false));
    }
    if entry == js3_math_floor_builtin() {
        return Some(BuiltinEntryMetadata::new("floor", 1, false, false));
    }
    if entry == js3_math_fround_builtin() {
        return Some(BuiltinEntryMetadata::new("fround", 1, false, false));
    }
    if entry == js3_math_hypot_builtin() {
        return Some(BuiltinEntryMetadata::new("hypot", 2, false, false));
    }
    if entry == js3_math_imul_builtin() {
        return Some(BuiltinEntryMetadata::new("imul", 2, false, false));
    }
    if entry == js3_math_log_builtin() {
        return Some(BuiltinEntryMetadata::new("log", 1, false, false));
    }
    if entry == js3_math_log10_builtin() {
        return Some(BuiltinEntryMetadata::new("log10", 1, false, false));
    }
    if entry == js3_math_log1p_builtin() {
        return Some(BuiltinEntryMetadata::new("log1p", 1, false, false));
    }
    if entry == js3_math_log2_builtin() {
        return Some(BuiltinEntryMetadata::new("log2", 1, false, false));
    }
    if entry == js3_math_max_builtin() {
        return Some(BuiltinEntryMetadata::new("max", 2, false, false));
    }
    if entry == js3_math_min_builtin() {
        return Some(BuiltinEntryMetadata::new("min", 2, false, false));
    }
    if entry == js3_math_pow_builtin() {
        return Some(BuiltinEntryMetadata::new("pow", 2, false, false));
    }
    if entry == js3_math_random_builtin() {
        return Some(BuiltinEntryMetadata::new("random", 0, false, false));
    }
    if entry == js3_math_round_builtin() {
        return Some(BuiltinEntryMetadata::new("round", 1, false, false));
    }
    if entry == js3_math_sign_builtin() {
        return Some(BuiltinEntryMetadata::new("sign", 1, false, false));
    }
    if entry == js3_math_sin_builtin() {
        return Some(BuiltinEntryMetadata::new("sin", 1, false, false));
    }
    if entry == js3_math_sinh_builtin() {
        return Some(BuiltinEntryMetadata::new("sinh", 1, false, false));
    }
    if entry == js3_math_sqrt_builtin() {
        return Some(BuiltinEntryMetadata::new("sqrt", 1, false, false));
    }
    if entry == js3_math_sum_precise_builtin() {
        return Some(BuiltinEntryMetadata::new("sumPrecise", 1, false, false));
    }
    if entry == js3_math_tan_builtin() {
        return Some(BuiltinEntryMetadata::new("tan", 1, false, false));
    }
    if entry == js3_math_tanh_builtin() {
        return Some(BuiltinEntryMetadata::new("tanh", 1, false, false));
    }
    if entry == js3_math_trunc_builtin() {
        return Some(BuiltinEntryMetadata::new("trunc", 1, false, false));
    }
    if entry == js3_bigint_builtin() {
        return Some(BuiltinEntryMetadata::new("BigInt", 1, true, true));
    }
    if entry == js3_bigint_as_int_n_builtin() {
        return Some(BuiltinEntryMetadata::new("asIntN", 2, false, false));
    }
    if entry == js3_bigint_as_uint_n_builtin() {
        return Some(BuiltinEntryMetadata::new("asUintN", 2, false, false));
    }
    if entry == js3_bigint_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == js3_bigint_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == js3_boolean_builtin() {
        return Some(BuiltinEntryMetadata::new("Boolean", 1, true, true));
    }
    if entry == js3_boolean_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == js3_boolean_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == js3_symbol_builtin() {
        return Some(BuiltinEntryMetadata::new("Symbol", 0, false, true));
    }
    if entry == js3_symbol_for_builtin() {
        return Some(BuiltinEntryMetadata::new("for", 1, false, false));
    }
    if entry == js3_symbol_key_for_builtin() {
        return Some(BuiltinEntryMetadata::new("keyFor", 1, false, false));
    }
    if entry == js3_symbol_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == js3_symbol_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == js3_symbol_to_primitive_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "[Symbol.toPrimitive]",
            1,
            false,
            false,
        ));
    }
    if entry == js3_array_species_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get [Symbol.species]",
            0,
            false,
            false,
        ));
    }
    if entry == js3_symbol_description_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get description",
            0,
            false,
            false,
        ));
    }
    if entry == js3_error_builtin() {
        return Some(BuiltinEntryMetadata::new("Error", 1, true, true));
    }
    if entry == js3_error_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == js3_eval_error_builtin() {
        return Some(BuiltinEntryMetadata::new("EvalError", 1, true, true));
    }
    if entry == js3_range_error_builtin() {
        return Some(BuiltinEntryMetadata::new("RangeError", 1, true, true));
    }
    if entry == js3_reference_error_builtin() {
        return Some(BuiltinEntryMetadata::new("ReferenceError", 1, true, true));
    }
    if entry == js3_syntax_error_builtin() {
        return Some(BuiltinEntryMetadata::new("SyntaxError", 1, true, true));
    }
    if entry == js3_type_error_builtin() {
        return Some(BuiltinEntryMetadata::new("TypeError", 1, true, true));
    }
    if entry == js3_uri_error_builtin() {
        return Some(BuiltinEntryMetadata::new("URIError", 1, true, true));
    }
    if entry == js3_aggregate_error_builtin() {
        return Some(BuiltinEntryMetadata::new("AggregateError", 2, true, true));
    }
    if entry == js3_suppressed_error_builtin() {
        return Some(BuiltinEntryMetadata::new("SuppressedError", 3, true, true));
    }
    if entry == js3_eval_builtin() {
        return Some(BuiltinEntryMetadata::new("eval", 1, false, false));
    }
    if entry == js3_promise_builtin() {
        return Some(BuiltinEntryMetadata::new("Promise", 1, true, true));
    }
    if entry == js3_disposable_stack_builtin() {
        return Some(BuiltinEntryMetadata::new("DisposableStack", 0, true, true));
    }
    if entry == js3_disposable_stack_use_builtin() {
        return Some(BuiltinEntryMetadata::new("use", 1, false, false));
    }
    if entry == js3_disposable_stack_adopt_builtin() {
        return Some(BuiltinEntryMetadata::new("adopt", 2, false, false));
    }
    if entry == js3_disposable_stack_defer_builtin() {
        return Some(BuiltinEntryMetadata::new("defer", 1, false, false));
    }
    if entry == js3_disposable_stack_move_builtin() {
        return Some(BuiltinEntryMetadata::new("move", 0, false, false));
    }
    if entry == js3_disposable_stack_disposed_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get disposed", 0, false, false));
    }
    if entry == js3_disposable_stack_dispose_builtin() {
        return Some(BuiltinEntryMetadata::new("dispose", 0, false, false));
    }
    if entry == js3_async_disposable_stack_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "AsyncDisposableStack",
            0,
            true,
            true,
        ));
    }
    if entry == js3_async_disposable_stack_use_builtin() {
        return Some(BuiltinEntryMetadata::new("use", 1, false, false));
    }
    if entry == js3_async_disposable_stack_adopt_builtin() {
        return Some(BuiltinEntryMetadata::new("adopt", 2, false, false));
    }
    if entry == js3_async_disposable_stack_defer_builtin() {
        return Some(BuiltinEntryMetadata::new("defer", 1, false, false));
    }
    if entry == js3_async_disposable_stack_move_builtin() {
        return Some(BuiltinEntryMetadata::new("move", 0, false, false));
    }
    if entry == js3_async_disposable_stack_disposed_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get disposed", 0, false, false));
    }
    if entry == js3_async_disposable_stack_dispose_async_builtin() {
        return Some(BuiltinEntryMetadata::new("disposeAsync", 0, false, false));
    }
    if entry == js3_async_disposal_resume_builtin() {
        return Some(BuiltinEntryMetadata::new("", 1, false, false));
    }
    if entry == js3_create_sync_disposal_scope_builtin() {
        return Some(BuiltinEntryMetadata::new("", 0, false, false));
    }
    if entry == js3_create_async_disposal_scope_builtin() {
        return Some(BuiltinEntryMetadata::new("", 0, false, false));
    }
    if entry == js3_add_sync_disposable_resource_builtin() {
        return Some(BuiltinEntryMetadata::new("", 2, false, false));
    }
    if entry == js3_add_async_disposable_resource_builtin() {
        return Some(BuiltinEntryMetadata::new("", 2, false, false));
    }
    if entry == js3_dispose_scope_builtin() {
        return Some(BuiltinEntryMetadata::new("", 1, false, false));
    }
    if entry == js3_dispose_scope_async_builtin() {
        return Some(BuiltinEntryMetadata::new("", 1, false, false));
    }
    if entry == js3_promise_then_builtin() {
        return Some(BuiltinEntryMetadata::new("then", 2, false, false));
    }
    if entry == js3_promise_catch_builtin() {
        return Some(BuiltinEntryMetadata::new("catch", 1, false, false));
    }
    if entry == js3_promise_finally_builtin() {
        return Some(BuiltinEntryMetadata::new("finally", 1, false, false));
    }
    if entry == js3_promise_finally_function_builtin() {
        return Some(BuiltinEntryMetadata::new("", 1, false, false));
    }
    if entry == js3_promise_resolve_builtin() {
        return Some(BuiltinEntryMetadata::new("resolve", 1, false, false));
    }
    if entry == js3_promise_reject_builtin() {
        return Some(BuiltinEntryMetadata::new("reject", 1, false, false));
    }
    if entry == js3_promise_all_builtin() {
        return Some(BuiltinEntryMetadata::new("all", 1, false, false));
    }
    if entry == js3_promise_all_settled_builtin() {
        return Some(BuiltinEntryMetadata::new("allSettled", 1, false, false));
    }
    if entry == js3_promise_race_builtin() {
        return Some(BuiltinEntryMetadata::new("race", 1, false, false));
    }
    if entry == js3_promise_any_builtin() {
        return Some(BuiltinEntryMetadata::new("any", 1, false, false));
    }
    if entry == js3_promise_species_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get [Symbol.species]",
            0,
            false,
            false,
        ));
    }
    if entry == js3_promise_capability_executor_builtin() {
        return Some(BuiltinEntryMetadata::new("", 2, false, false));
    }
    if entry == js3_promise_resolve_function_builtin() {
        return Some(BuiltinEntryMetadata::new("", 1, false, false));
    }
    if entry == js3_promise_reject_function_builtin() {
        return Some(BuiltinEntryMetadata::new("", 1, false, false));
    }
    if entry == js3_promise_all_resolve_element_builtin() {
        return Some(BuiltinEntryMetadata::new("", 1, false, false));
    }
    if entry == js3_promise_all_settled_resolve_element_builtin() {
        return Some(BuiltinEntryMetadata::new("", 1, false, false));
    }
    if entry == js3_promise_all_settled_reject_element_builtin() {
        return Some(BuiltinEntryMetadata::new("", 1, false, false));
    }
    if entry == js3_promise_any_reject_element_builtin() {
        return Some(BuiltinEntryMetadata::new("", 1, false, false));
    }
    if entry == js3_parse_int_builtin() {
        return Some(BuiltinEntryMetadata::new("parseInt", 2, false, false));
    }
    if entry == js3_parse_float_builtin() {
        return Some(BuiltinEntryMetadata::new("parseFloat", 1, false, false));
    }
    if entry == js3_is_nan_builtin() {
        return Some(BuiltinEntryMetadata::new("isNaN", 1, false, false));
    }
    if entry == js3_is_finite_builtin() {
        return Some(BuiltinEntryMetadata::new("isFinite", 1, false, false));
    }
    if entry == js3_encode_uri_builtin() {
        return Some(BuiltinEntryMetadata::new("encodeURI", 1, false, false));
    }
    if entry == js3_encode_uri_component_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "encodeURIComponent",
            1,
            false,
            false,
        ));
    }
    if entry == js3_decode_uri_builtin() {
        return Some(BuiltinEntryMetadata::new("decodeURI", 1, false, false));
    }
    if entry == js3_decode_uri_component_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "decodeURIComponent",
            1,
            false,
            false,
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_public_metadata_table_matches_public_lookup() {
        let expected = [
            (
                js3_object_builtin(),
                BuiltinEntryMetadata::new("Object", 1, true, true),
            ),
            (
                js3_object_create_builtin(),
                BuiltinEntryMetadata::new("create", 2, false, false),
            ),
            (
                js3_object_get_prototype_of_builtin(),
                BuiltinEntryMetadata::new("getPrototypeOf", 1, false, false),
            ),
            (
                js3_object_set_prototype_of_builtin(),
                BuiltinEntryMetadata::new("setPrototypeOf", 2, false, false),
            ),
            (
                js3_object_get_own_property_descriptor_builtin(),
                BuiltinEntryMetadata::new("getOwnPropertyDescriptor", 2, false, false),
            ),
            (
                js3_object_get_own_property_descriptors_builtin(),
                BuiltinEntryMetadata::new("getOwnPropertyDescriptors", 1, false, false),
            ),
            (
                js3_object_get_own_property_names_builtin(),
                BuiltinEntryMetadata::new("getOwnPropertyNames", 1, false, false),
            ),
            (
                js3_object_get_own_property_symbols_builtin(),
                BuiltinEntryMetadata::new("getOwnPropertySymbols", 1, false, false),
            ),
            (
                js3_object_define_properties_builtin(),
                BuiltinEntryMetadata::new("defineProperties", 2, false, false),
            ),
            (
                js3_object_define_property_builtin(),
                BuiltinEntryMetadata::new("defineProperty", 3, false, false),
            ),
            (
                js3_object_assign_builtin(),
                BuiltinEntryMetadata::new("assign", 2, false, false),
            ),
            (
                js3_object_from_entries_builtin(),
                BuiltinEntryMetadata::new("fromEntries", 1, false, false),
            ),
            (
                js3_object_group_by_builtin(),
                BuiltinEntryMetadata::new("groupBy", 2, false, false),
            ),
            (
                js3_object_prevent_extensions_builtin(),
                BuiltinEntryMetadata::new("preventExtensions", 1, false, false),
            ),
            (
                js3_object_is_extensible_builtin(),
                BuiltinEntryMetadata::new("isExtensible", 1, false, false),
            ),
            (
                js3_object_is_builtin(),
                BuiltinEntryMetadata::new("is", 2, false, false),
            ),
            (
                js3_object_seal_builtin(),
                BuiltinEntryMetadata::new("seal", 1, false, false),
            ),
            (
                js3_object_freeze_builtin(),
                BuiltinEntryMetadata::new("freeze", 1, false, false),
            ),
            (
                js3_object_is_sealed_builtin(),
                BuiltinEntryMetadata::new("isSealed", 1, false, false),
            ),
            (
                js3_object_is_frozen_builtin(),
                BuiltinEntryMetadata::new("isFrozen", 1, false, false),
            ),
            (
                js3_object_to_locale_string_builtin(),
                BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
            ),
            (
                js3_object_to_string_builtin(),
                BuiltinEntryMetadata::new("toString", 0, false, false),
            ),
            (
                js3_object_value_of_builtin(),
                BuiltinEntryMetadata::new("valueOf", 0, false, false),
            ),
            (
                js3_object_has_own_property_builtin(),
                BuiltinEntryMetadata::new("hasOwnProperty", 1, false, false),
            ),
            (
                js3_object_is_prototype_of_builtin(),
                BuiltinEntryMetadata::new("isPrototypeOf", 1, false, false),
            ),
            (
                js3_object_property_is_enumerable_builtin(),
                BuiltinEntryMetadata::new("propertyIsEnumerable", 1, false, false),
            ),
            (
                js3_object_define_getter_builtin(),
                BuiltinEntryMetadata::new("__defineGetter__", 2, false, false),
            ),
            (
                js3_object_define_setter_builtin(),
                BuiltinEntryMetadata::new("__defineSetter__", 2, false, false),
            ),
            (
                js3_object_lookup_getter_builtin(),
                BuiltinEntryMetadata::new("__lookupGetter__", 1, false, false),
            ),
            (
                js3_object_lookup_setter_builtin(),
                BuiltinEntryMetadata::new("__lookupSetter__", 1, false, false),
            ),
            (
                js3_object_proto_getter_builtin(),
                BuiltinEntryMetadata::new("get __proto__", 0, false, false),
            ),
            (
                js3_object_proto_setter_builtin(),
                BuiltinEntryMetadata::new("set __proto__", 1, false, false),
            ),
            (
                js3_object_keys_builtin(),
                BuiltinEntryMetadata::new("keys", 1, false, false),
            ),
            (
                js3_object_entries_builtin(),
                BuiltinEntryMetadata::new("entries", 1, false, false),
            ),
            (
                js3_object_values_builtin(),
                BuiltinEntryMetadata::new("values", 1, false, false),
            ),
            (
                js3_object_has_own_builtin(),
                BuiltinEntryMetadata::new("hasOwn", 2, false, false),
            ),
        ];

        assert_eq!(PUBLIC_OBJECT_BUILTIN_METADATA.len(), expected.len());
        for (entry, metadata) in expected {
            assert_eq!(object_public_builtin_metadata(entry), Some(metadata));
            assert_eq!(public_builtin_metadata(entry), Some(metadata));
        }
    }

    #[test]
    fn function_public_metadata_table_matches_public_lookup() {
        let expected = [
            (
                js3_function_builtin(),
                BuiltinEntryMetadata::new("Function", 1, true, true),
            ),
            (
                js3_function_prototype_builtin(),
                BuiltinEntryMetadata::new("", 0, false, false),
            ),
            (
                js3_function_call_builtin(),
                BuiltinEntryMetadata::new("call", 1, false, false),
            ),
            (
                js3_function_apply_builtin(),
                BuiltinEntryMetadata::new("apply", 2, false, false),
            ),
            (
                js3_function_bind_builtin(),
                BuiltinEntryMetadata::new("bind", 1, false, false),
            ),
            (
                js3_function_to_string_builtin(),
                BuiltinEntryMetadata::new("toString", 0, false, false),
            ),
            (
                js3_function_symbol_has_instance_builtin(),
                BuiltinEntryMetadata::new("[Symbol.hasInstance]", 1, false, false),
            ),
            (
                js3_async_function_builtin(),
                BuiltinEntryMetadata::new("AsyncFunction", 1, true, true),
            ),
            (
                js3_async_generator_function_builtin(),
                BuiltinEntryMetadata::new("AsyncGeneratorFunction", 1, true, true),
            ),
            (
                js3_async_generator_next_builtin(),
                BuiltinEntryMetadata::new("next", 1, false, false),
            ),
            (
                js3_async_generator_return_builtin(),
                BuiltinEntryMetadata::new("return", 1, false, false),
            ),
            (
                js3_async_generator_throw_builtin(),
                BuiltinEntryMetadata::new("throw", 1, false, false),
            ),
            (
                js3_generator_function_builtin(),
                BuiltinEntryMetadata::new("GeneratorFunction", 1, true, true),
            ),
            (
                js3_generator_next_builtin(),
                BuiltinEntryMetadata::new("next", 1, false, false),
            ),
            (
                js3_generator_return_builtin(),
                BuiltinEntryMetadata::new("return", 1, false, false),
            ),
            (
                js3_generator_throw_builtin(),
                BuiltinEntryMetadata::new("throw", 1, false, false),
            ),
        ];

        assert_eq!(PUBLIC_FUNCTION_BUILTIN_METADATA.len(), expected.len());
        for (entry, metadata) in expected {
            assert_eq!(function_public_builtin_metadata(entry), Some(metadata));
            assert_eq!(public_builtin_metadata(entry), Some(metadata));
        }
    }

    #[test]
    fn array_public_metadata_table_matches_public_lookup() {
        let expected = [
            (
                js3_array_builtin(),
                BuiltinEntryMetadata::new("Array", 1, true, true),
            ),
            (
                js3_array_from_builtin(),
                BuiltinEntryMetadata::new("from", 1, false, false),
            ),
            (
                js3_array_from_async_builtin(),
                BuiltinEntryMetadata::new("fromAsync", 1, false, false),
            ),
            (
                js3_array_of_builtin(),
                BuiltinEntryMetadata::new("of", 0, false, false),
            ),
            (
                js3_array_is_array_builtin(),
                BuiltinEntryMetadata::new("isArray", 1, false, false),
            ),
            (
                js3_array_at_builtin(),
                BuiltinEntryMetadata::new("at", 1, false, false),
            ),
            (
                js3_array_concat_builtin(),
                BuiltinEntryMetadata::new("concat", 1, false, false),
            ),
            (
                js3_array_copy_within_builtin(),
                BuiltinEntryMetadata::new("copyWithin", 2, false, false),
            ),
            (
                js3_array_fill_builtin(),
                BuiltinEntryMetadata::new("fill", 1, false, false),
            ),
            (
                js3_array_join_builtin(),
                BuiltinEntryMetadata::new("join", 1, false, false),
            ),
            (
                js3_array_pop_builtin(),
                BuiltinEntryMetadata::new("pop", 0, false, false),
            ),
            (
                js3_array_push_builtin(),
                BuiltinEntryMetadata::new("push", 1, false, false),
            ),
            (
                js3_array_shift_builtin(),
                BuiltinEntryMetadata::new("shift", 0, false, false),
            ),
            (
                js3_array_unshift_builtin(),
                BuiltinEntryMetadata::new("unshift", 1, false, false),
            ),
            (
                js3_array_every_builtin(),
                BuiltinEntryMetadata::new("every", 1, false, false),
            ),
            (
                js3_array_filter_builtin(),
                BuiltinEntryMetadata::new("filter", 1, false, false),
            ),
            (
                js3_array_flat_builtin(),
                BuiltinEntryMetadata::new("flat", 0, false, false),
            ),
            (
                js3_array_flat_map_builtin(),
                BuiltinEntryMetadata::new("flatMap", 1, false, false),
            ),
            (
                js3_array_find_builtin(),
                BuiltinEntryMetadata::new("find", 1, false, false),
            ),
            (
                js3_array_find_index_builtin(),
                BuiltinEntryMetadata::new("findIndex", 1, false, false),
            ),
            (
                js3_array_find_last_builtin(),
                BuiltinEntryMetadata::new("findLast", 1, false, false),
            ),
            (
                js3_array_find_last_index_builtin(),
                BuiltinEntryMetadata::new("findLastIndex", 1, false, false),
            ),
            (
                js3_array_for_each_builtin(),
                BuiltinEntryMetadata::new("forEach", 1, false, false),
            ),
            (
                js3_array_includes_builtin(),
                BuiltinEntryMetadata::new("includes", 1, false, false),
            ),
            (
                js3_array_index_of_builtin(),
                BuiltinEntryMetadata::new("indexOf", 1, false, false),
            ),
            (
                js3_array_map_builtin(),
                BuiltinEntryMetadata::new("map", 1, false, false),
            ),
            (
                js3_array_reduce_builtin(),
                BuiltinEntryMetadata::new("reduce", 1, false, false),
            ),
            (
                js3_array_reduce_right_builtin(),
                BuiltinEntryMetadata::new("reduceRight", 1, false, false),
            ),
            (
                js3_array_reverse_builtin(),
                BuiltinEntryMetadata::new("reverse", 0, false, false),
            ),
            (
                js3_array_slice_builtin(),
                BuiltinEntryMetadata::new("slice", 2, false, false),
            ),
            (
                js3_array_some_builtin(),
                BuiltinEntryMetadata::new("some", 1, false, false),
            ),
            (
                js3_array_last_index_of_builtin(),
                BuiltinEntryMetadata::new("lastIndexOf", 1, false, false),
            ),
            (
                js3_array_sort_builtin(),
                BuiltinEntryMetadata::new("sort", 1, false, false),
            ),
            (
                js3_array_splice_builtin(),
                BuiltinEntryMetadata::new("splice", 2, false, false),
            ),
            (
                js3_array_to_reversed_builtin(),
                BuiltinEntryMetadata::new("toReversed", 0, false, false),
            ),
            (
                js3_array_to_sorted_builtin(),
                BuiltinEntryMetadata::new("toSorted", 1, false, false),
            ),
            (
                js3_array_to_spliced_builtin(),
                BuiltinEntryMetadata::new("toSpliced", 2, false, false),
            ),
            (
                js3_array_to_string_builtin(),
                BuiltinEntryMetadata::new("toString", 0, false, false),
            ),
            (
                js3_array_to_locale_string_builtin(),
                BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
            ),
            (
                js3_array_values_builtin(),
                BuiltinEntryMetadata::new("values", 0, false, false),
            ),
            (
                js3_array_keys_builtin(),
                BuiltinEntryMetadata::new("keys", 0, false, false),
            ),
            (
                js3_array_entries_builtin(),
                BuiltinEntryMetadata::new("entries", 0, false, false),
            ),
            (
                js3_array_with_builtin(),
                BuiltinEntryMetadata::new("with", 2, false, false),
            ),
        ];

        assert_eq!(PUBLIC_ARRAY_BUILTIN_METADATA.len(), expected.len());
        for (entry, metadata) in expected {
            assert_eq!(array_public_builtin_metadata(entry), Some(metadata));
            assert_eq!(public_builtin_metadata(entry), Some(metadata));
        }
    }

    #[test]
    fn keyed_collection_public_metadata_table_matches_public_lookup() {
        let expected = [
            (
                js3_map_builtin(),
                BuiltinEntryMetadata::new("Map", 0, true, true),
            ),
            (
                js3_set_builtin(),
                BuiltinEntryMetadata::new("Set", 0, true, true),
            ),
            (
                js3_weak_map_builtin(),
                BuiltinEntryMetadata::new("WeakMap", 0, true, true),
            ),
            (
                js3_weak_set_builtin(),
                BuiltinEntryMetadata::new("WeakSet", 0, true, true),
            ),
            (
                js3_map_get_builtin(),
                BuiltinEntryMetadata::new("get", 1, false, false),
            ),
            (
                js3_map_set_builtin(),
                BuiltinEntryMetadata::new("set", 2, false, false),
            ),
            (
                js3_map_has_builtin(),
                BuiltinEntryMetadata::new("has", 1, false, false),
            ),
            (
                js3_map_delete_builtin(),
                BuiltinEntryMetadata::new("delete", 1, false, false),
            ),
            (
                js3_map_clear_builtin(),
                BuiltinEntryMetadata::new("clear", 0, false, false),
            ),
            (
                js3_map_entries_builtin(),
                BuiltinEntryMetadata::new("entries", 0, false, false),
            ),
            (
                js3_map_values_builtin(),
                BuiltinEntryMetadata::new("values", 0, false, false),
            ),
            (
                js3_map_keys_builtin(),
                BuiltinEntryMetadata::new("keys", 0, false, false),
            ),
            (
                js3_map_for_each_builtin(),
                BuiltinEntryMetadata::new("forEach", 1, false, false),
            ),
            (
                js3_map_size_getter_builtin(),
                BuiltinEntryMetadata::new("get size", 0, false, false),
            ),
            (
                js3_set_add_builtin(),
                BuiltinEntryMetadata::new("add", 1, false, false),
            ),
            (
                js3_set_has_builtin(),
                BuiltinEntryMetadata::new("has", 1, false, false),
            ),
            (
                js3_set_delete_builtin(),
                BuiltinEntryMetadata::new("delete", 1, false, false),
            ),
            (
                js3_set_clear_builtin(),
                BuiltinEntryMetadata::new("clear", 0, false, false),
            ),
            (
                js3_set_entries_builtin(),
                BuiltinEntryMetadata::new("entries", 0, false, false),
            ),
            (
                js3_set_values_builtin(),
                BuiltinEntryMetadata::new("values", 0, false, false),
            ),
            (
                js3_set_keys_builtin(),
                BuiltinEntryMetadata::new("keys", 0, false, false),
            ),
            (
                js3_set_for_each_builtin(),
                BuiltinEntryMetadata::new("forEach", 1, false, false),
            ),
            (
                js3_set_size_getter_builtin(),
                BuiltinEntryMetadata::new("get size", 0, false, false),
            ),
            (
                js3_weak_map_get_builtin(),
                BuiltinEntryMetadata::new("get", 1, false, false),
            ),
            (
                js3_weak_map_set_builtin(),
                BuiltinEntryMetadata::new("set", 2, false, false),
            ),
            (
                js3_weak_map_has_builtin(),
                BuiltinEntryMetadata::new("has", 1, false, false),
            ),
            (
                js3_weak_map_delete_builtin(),
                BuiltinEntryMetadata::new("delete", 1, false, false),
            ),
            (
                js3_weak_set_add_builtin(),
                BuiltinEntryMetadata::new("add", 1, false, false),
            ),
            (
                js3_weak_set_has_builtin(),
                BuiltinEntryMetadata::new("has", 1, false, false),
            ),
            (
                js3_weak_set_delete_builtin(),
                BuiltinEntryMetadata::new("delete", 1, false, false),
            ),
        ];

        assert_eq!(
            PUBLIC_KEYED_COLLECTION_BUILTIN_METADATA.len(),
            expected.len()
        );
        for (entry, metadata) in expected {
            assert_eq!(
                keyed_collection_public_builtin_metadata(entry),
                Some(metadata)
            );
            assert_eq!(public_builtin_metadata(entry), Some(metadata));
        }
    }

    #[test]
    fn binary_data_public_metadata_table_matches_public_lookup() {
        let expected = [
            (
                js3_array_buffer_builtin(),
                BuiltinEntryMetadata::new("ArrayBuffer", 1, true, true),
            ),
            (
                js3_array_buffer_is_view_builtin(),
                BuiltinEntryMetadata::new("isView", 1, false, false),
            ),
            (
                js3_shared_array_buffer_builtin(),
                BuiltinEntryMetadata::new("SharedArrayBuffer", 1, true, true),
            ),
            (
                js3_data_view_builtin(),
                BuiltinEntryMetadata::new("DataView", 1, true, true),
            ),
            (
                js3_typed_array_builtin(),
                BuiltinEntryMetadata::new("TypedArray", 0, true, true),
            ),
            (
                js3_typed_array_from_builtin(),
                BuiltinEntryMetadata::new("from", 1, false, false),
            ),
            (
                js3_typed_array_of_builtin(),
                BuiltinEntryMetadata::new("of", 0, false, false),
            ),
            (
                js3_typed_array_every_builtin(),
                BuiltinEntryMetadata::new("every", 1, false, false),
            ),
            (
                js3_typed_array_some_builtin(),
                BuiltinEntryMetadata::new("some", 1, false, false),
            ),
            (
                js3_typed_array_find_builtin(),
                BuiltinEntryMetadata::new("find", 1, false, false),
            ),
            (
                js3_typed_array_find_index_builtin(),
                BuiltinEntryMetadata::new("findIndex", 1, false, false),
            ),
            (
                js3_typed_array_find_last_builtin(),
                BuiltinEntryMetadata::new("findLast", 1, false, false),
            ),
            (
                js3_typed_array_find_last_index_builtin(),
                BuiltinEntryMetadata::new("findLastIndex", 1, false, false),
            ),
            (
                js3_typed_array_fill_builtin(),
                BuiltinEntryMetadata::new("fill", 1, false, false),
            ),
            (
                js3_typed_array_copy_within_builtin(),
                BuiltinEntryMetadata::new("copyWithin", 2, false, false),
            ),
            (
                js3_typed_array_filter_builtin(),
                BuiltinEntryMetadata::new("filter", 1, false, false),
            ),
            (
                js3_typed_array_for_each_builtin(),
                BuiltinEntryMetadata::new("forEach", 1, false, false),
            ),
            (
                js3_typed_array_includes_builtin(),
                BuiltinEntryMetadata::new("includes", 1, false, false),
            ),
            (
                js3_typed_array_index_of_builtin(),
                BuiltinEntryMetadata::new("indexOf", 1, false, false),
            ),
            (
                js3_typed_array_join_builtin(),
                BuiltinEntryMetadata::new("join", 1, false, false),
            ),
            (
                js3_typed_array_last_index_of_builtin(),
                BuiltinEntryMetadata::new("lastIndexOf", 1, false, false),
            ),
            (
                js3_typed_array_map_builtin(),
                BuiltinEntryMetadata::new("map", 1, false, false),
            ),
            (
                js3_typed_array_reduce_builtin(),
                BuiltinEntryMetadata::new("reduce", 1, false, false),
            ),
            (
                js3_typed_array_reduce_right_builtin(),
                BuiltinEntryMetadata::new("reduceRight", 1, false, false),
            ),
            (
                js3_typed_array_reverse_builtin(),
                BuiltinEntryMetadata::new("reverse", 0, false, false),
            ),
            (
                js3_typed_array_sort_builtin(),
                BuiltinEntryMetadata::new("sort", 1, false, false),
            ),
            (
                js3_typed_array_to_locale_string_builtin(),
                BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
            ),
            (
                js3_typed_array_to_string_builtin(),
                BuiltinEntryMetadata::new("toString", 0, false, false),
            ),
            (
                js3_typed_array_to_reversed_builtin(),
                BuiltinEntryMetadata::new("toReversed", 0, false, false),
            ),
            (
                js3_typed_array_to_sorted_builtin(),
                BuiltinEntryMetadata::new("toSorted", 1, false, false),
            ),
            (
                js3_typed_array_with_builtin(),
                BuiltinEntryMetadata::new("with", 2, false, false),
            ),
            (
                js3_int8_array_builtin(),
                BuiltinEntryMetadata::new("Int8Array", 3, true, true),
            ),
            (
                js3_int16_array_builtin(),
                BuiltinEntryMetadata::new("Int16Array", 3, true, true),
            ),
            (
                js3_int32_array_builtin(),
                BuiltinEntryMetadata::new("Int32Array", 3, true, true),
            ),
            (
                js3_float32_array_builtin(),
                BuiltinEntryMetadata::new("Float32Array", 3, true, true),
            ),
            (
                js3_float64_array_builtin(),
                BuiltinEntryMetadata::new("Float64Array", 3, true, true),
            ),
            (
                js3_big_int64_array_builtin(),
                BuiltinEntryMetadata::new("BigInt64Array", 3, true, true),
            ),
            (
                js3_big_uint64_array_builtin(),
                BuiltinEntryMetadata::new("BigUint64Array", 3, true, true),
            ),
            (
                js3_uint32_array_builtin(),
                BuiltinEntryMetadata::new("Uint32Array", 3, true, true),
            ),
            (
                js3_uint16_array_builtin(),
                BuiltinEntryMetadata::new("Uint16Array", 3, true, true),
            ),
            (
                js3_uint8_clamped_array_builtin(),
                BuiltinEntryMetadata::new("Uint8ClampedArray", 3, true, true),
            ),
            (
                js3_uint8_array_builtin(),
                BuiltinEntryMetadata::new("Uint8Array", 3, true, true),
            ),
            (
                js3_array_buffer_byte_length_getter_builtin(),
                BuiltinEntryMetadata::new("get byteLength", 0, false, false),
            ),
            (
                js3_array_buffer_slice_builtin(),
                BuiltinEntryMetadata::new("slice", 2, false, false),
            ),
            (
                js3_shared_array_buffer_byte_length_getter_builtin(),
                BuiltinEntryMetadata::new("get byteLength", 0, false, false),
            ),
            (
                js3_shared_array_buffer_slice_builtin(),
                BuiltinEntryMetadata::new("slice", 2, false, false),
            ),
            (
                js3_atomics_load_builtin(),
                BuiltinEntryMetadata::new("load", 2, false, false),
            ),
            (
                js3_atomics_store_builtin(),
                BuiltinEntryMetadata::new("store", 3, false, false),
            ),
            (
                js3_atomics_add_builtin(),
                BuiltinEntryMetadata::new("add", 3, false, false),
            ),
            (
                js3_atomics_sub_builtin(),
                BuiltinEntryMetadata::new("sub", 3, false, false),
            ),
            (
                js3_atomics_and_builtin(),
                BuiltinEntryMetadata::new("and", 3, false, false),
            ),
            (
                js3_atomics_or_builtin(),
                BuiltinEntryMetadata::new("or", 3, false, false),
            ),
            (
                js3_atomics_xor_builtin(),
                BuiltinEntryMetadata::new("xor", 3, false, false),
            ),
            (
                js3_atomics_exchange_builtin(),
                BuiltinEntryMetadata::new("exchange", 3, false, false),
            ),
            (
                js3_atomics_compare_exchange_builtin(),
                BuiltinEntryMetadata::new("compareExchange", 4, false, false),
            ),
            (
                js3_atomics_notify_builtin(),
                BuiltinEntryMetadata::new("notify", 3, false, false),
            ),
            (
                js3_atomics_wait_builtin(),
                BuiltinEntryMetadata::new("wait", 4, false, false),
            ),
            (
                js3_atomics_wait_async_builtin(),
                BuiltinEntryMetadata::new("waitAsync", 4, false, false),
            ),
            (
                js3_atomics_is_lock_free_builtin(),
                BuiltinEntryMetadata::new("isLockFree", 1, false, false),
            ),
            (
                js3_data_view_buffer_getter_builtin(),
                BuiltinEntryMetadata::new("get buffer", 0, false, false),
            ),
            (
                js3_data_view_byte_length_getter_builtin(),
                BuiltinEntryMetadata::new("get byteLength", 0, false, false),
            ),
            (
                js3_data_view_byte_offset_getter_builtin(),
                BuiltinEntryMetadata::new("get byteOffset", 0, false, false),
            ),
            (
                js3_data_view_get_float32_builtin(),
                BuiltinEntryMetadata::new("getFloat32", 1, false, false),
            ),
            (
                js3_data_view_get_float64_builtin(),
                BuiltinEntryMetadata::new("getFloat64", 1, false, false),
            ),
            (
                js3_data_view_get_int16_builtin(),
                BuiltinEntryMetadata::new("getInt16", 1, false, false),
            ),
            (
                js3_data_view_get_int32_builtin(),
                BuiltinEntryMetadata::new("getInt32", 1, false, false),
            ),
            (
                js3_data_view_get_int8_builtin(),
                BuiltinEntryMetadata::new("getInt8", 1, false, false),
            ),
            (
                js3_data_view_get_uint16_builtin(),
                BuiltinEntryMetadata::new("getUint16", 1, false, false),
            ),
            (
                js3_data_view_get_uint32_builtin(),
                BuiltinEntryMetadata::new("getUint32", 1, false, false),
            ),
            (
                js3_data_view_get_uint8_builtin(),
                BuiltinEntryMetadata::new("getUint8", 1, false, false),
            ),
            (
                js3_data_view_set_float32_builtin(),
                BuiltinEntryMetadata::new("setFloat32", 2, false, false),
            ),
            (
                js3_data_view_set_float64_builtin(),
                BuiltinEntryMetadata::new("setFloat64", 2, false, false),
            ),
            (
                js3_data_view_set_int16_builtin(),
                BuiltinEntryMetadata::new("setInt16", 2, false, false),
            ),
            (
                js3_data_view_set_int32_builtin(),
                BuiltinEntryMetadata::new("setInt32", 2, false, false),
            ),
            (
                js3_data_view_set_int8_builtin(),
                BuiltinEntryMetadata::new("setInt8", 2, false, false),
            ),
            (
                js3_data_view_set_uint16_builtin(),
                BuiltinEntryMetadata::new("setUint16", 2, false, false),
            ),
            (
                js3_data_view_set_uint32_builtin(),
                BuiltinEntryMetadata::new("setUint32", 2, false, false),
            ),
            (
                js3_data_view_set_uint8_builtin(),
                BuiltinEntryMetadata::new("setUint8", 2, false, false),
            ),
            (
                js3_uint8_array_buffer_getter_builtin(),
                BuiltinEntryMetadata::new("get buffer", 0, false, false),
            ),
            (
                js3_uint8_array_byte_length_getter_builtin(),
                BuiltinEntryMetadata::new("get byteLength", 0, false, false),
            ),
            (
                js3_uint8_array_byte_offset_getter_builtin(),
                BuiltinEntryMetadata::new("get byteOffset", 0, false, false),
            ),
            (
                js3_uint8_array_length_getter_builtin(),
                BuiltinEntryMetadata::new("get length", 0, false, false),
            ),
            (
                js3_uint8_array_values_builtin(),
                BuiltinEntryMetadata::new("values", 0, false, false),
            ),
            (
                js3_uint8_array_keys_builtin(),
                BuiltinEntryMetadata::new("keys", 0, false, false),
            ),
            (
                js3_uint8_array_entries_builtin(),
                BuiltinEntryMetadata::new("entries", 0, false, false),
            ),
            (
                js3_uint8_array_set_builtin(),
                BuiltinEntryMetadata::new("set", 1, false, false),
            ),
            (
                js3_uint8_array_slice_builtin(),
                BuiltinEntryMetadata::new("slice", 2, false, false),
            ),
            (
                js3_uint8_array_subarray_builtin(),
                BuiltinEntryMetadata::new("subarray", 2, false, false),
            ),
            (
                js3_typed_array_at_builtin(),
                BuiltinEntryMetadata::new("at", 1, false, false),
            ),
            (
                js3_typed_array_to_string_tag_getter_builtin(),
                BuiltinEntryMetadata::new("get [Symbol.toStringTag]", 0, false, false),
            ),
        ];

        assert_eq!(PUBLIC_BINARY_DATA_BUILTIN_METADATA.len(), expected.len());
        for (entry, metadata) in expected {
            assert_eq!(binary_data_public_builtin_metadata(entry), Some(metadata));
            assert_eq!(public_builtin_metadata(entry), Some(metadata));
        }
    }

    #[test]
    fn object_reflection_public_metadata_table_matches_public_lookup() {
        let expected = [
            (
                js3_json_parse_builtin(),
                BuiltinEntryMetadata::new("parse", 2, false, false),
            ),
            (
                js3_json_stringify_builtin(),
                BuiltinEntryMetadata::new("stringify", 3, false, false),
            ),
            (
                js3_json_raw_json_builtin(),
                BuiltinEntryMetadata::new("rawJSON", 1, false, false),
            ),
            (
                js3_json_is_raw_json_builtin(),
                BuiltinEntryMetadata::new("isRawJSON", 1, false, false),
            ),
            (
                js3_reflect_apply_builtin(),
                BuiltinEntryMetadata::new("apply", 3, false, false),
            ),
            (
                js3_reflect_construct_builtin(),
                BuiltinEntryMetadata::new("construct", 2, false, false),
            ),
            (
                js3_reflect_define_property_builtin(),
                BuiltinEntryMetadata::new("defineProperty", 3, false, false),
            ),
            (
                js3_reflect_delete_property_builtin(),
                BuiltinEntryMetadata::new("deleteProperty", 2, false, false),
            ),
            (
                js3_reflect_get_builtin(),
                BuiltinEntryMetadata::new("get", 2, false, false),
            ),
            (
                js3_reflect_get_own_property_descriptor_builtin(),
                BuiltinEntryMetadata::new("getOwnPropertyDescriptor", 2, false, false),
            ),
            (
                js3_reflect_get_prototype_of_builtin(),
                BuiltinEntryMetadata::new("getPrototypeOf", 1, false, false),
            ),
            (
                js3_reflect_has_builtin(),
                BuiltinEntryMetadata::new("has", 2, false, false),
            ),
            (
                js3_reflect_is_extensible_builtin(),
                BuiltinEntryMetadata::new("isExtensible", 1, false, false),
            ),
            (
                js3_reflect_own_keys_builtin(),
                BuiltinEntryMetadata::new("ownKeys", 1, false, false),
            ),
            (
                js3_reflect_prevent_extensions_builtin(),
                BuiltinEntryMetadata::new("preventExtensions", 1, false, false),
            ),
            (
                js3_reflect_set_builtin(),
                BuiltinEntryMetadata::new("set", 3, false, false),
            ),
            (
                js3_reflect_set_prototype_of_builtin(),
                BuiltinEntryMetadata::new("setPrototypeOf", 2, false, false),
            ),
            (
                js3_proxy_builtin(),
                BuiltinEntryMetadata::new("Proxy", 2, true, false),
            ),
            (
                js3_proxy_revocable_builtin(),
                BuiltinEntryMetadata::new("revocable", 2, false, false),
            ),
            (
                js3_proxy_revoke_builtin(),
                BuiltinEntryMetadata::new("", 0, false, false),
            ),
        ];

        assert_eq!(
            PUBLIC_OBJECT_REFLECTION_BUILTIN_METADATA.len(),
            expected.len()
        );
        for (entry, metadata) in expected {
            assert_eq!(
                object_reflection_public_builtin_metadata(entry),
                Some(metadata)
            );
            assert_eq!(public_builtin_metadata(entry), Some(metadata));
        }
    }

    #[test]
    fn text_public_metadata_table_matches_public_lookup() {
        let expected = [
            (
                js3_iterator_prototype_iterator_builtin(),
                BuiltinEntryMetadata::new("[Symbol.iterator]", 0, false, false),
            ),
            (
                js3_array_iterator_next_builtin(),
                BuiltinEntryMetadata::new("next", 0, false, false),
            ),
            (
                js3_map_iterator_next_builtin(),
                BuiltinEntryMetadata::new("next", 0, false, false),
            ),
            (
                js3_set_iterator_next_builtin(),
                BuiltinEntryMetadata::new("next", 0, false, false),
            ),
            (
                js3_string_builtin(),
                BuiltinEntryMetadata::new("String", 1, true, true),
            ),
            (
                js3_string_iterator_builtin(),
                BuiltinEntryMetadata::new("[Symbol.iterator]", 0, false, false),
            ),
            (
                js3_string_iterator_next_builtin(),
                BuiltinEntryMetadata::new("next", 0, false, false),
            ),
            (
                js3_string_to_string_builtin(),
                BuiltinEntryMetadata::new("toString", 0, false, false),
            ),
            (
                js3_string_value_of_builtin(),
                BuiltinEntryMetadata::new("valueOf", 0, false, false),
            ),
            (
                js3_string_concat_builtin(),
                BuiltinEntryMetadata::new("concat", 1, false, false),
            ),
            (
                js3_string_char_at_builtin(),
                BuiltinEntryMetadata::new("charAt", 1, false, false),
            ),
            (
                js3_string_char_code_at_builtin(),
                BuiltinEntryMetadata::new("charCodeAt", 1, false, false),
            ),
            (
                js3_string_from_char_code_builtin(),
                BuiltinEntryMetadata::new("fromCharCode", 1, false, false),
            ),
            (
                js3_string_from_code_point_builtin(),
                BuiltinEntryMetadata::new("fromCodePoint", 1, false, false),
            ),
            (
                js3_string_raw_builtin(),
                BuiltinEntryMetadata::new("raw", 1, false, false),
            ),
            (
                js3_string_at_builtin(),
                BuiltinEntryMetadata::new("at", 1, false, false),
            ),
            (
                js3_string_code_point_at_builtin(),
                BuiltinEntryMetadata::new("codePointAt", 1, false, false),
            ),
            (
                js3_string_ends_with_builtin(),
                BuiltinEntryMetadata::new("endsWith", 1, false, false),
            ),
            (
                js3_string_includes_builtin(),
                BuiltinEntryMetadata::new("includes", 1, false, false),
            ),
            (
                js3_string_index_of_builtin(),
                BuiltinEntryMetadata::new("indexOf", 1, false, false),
            ),
            (
                js3_string_is_well_formed_builtin(),
                BuiltinEntryMetadata::new("isWellFormed", 0, false, false),
            ),
            (
                js3_string_locale_compare_builtin(),
                BuiltinEntryMetadata::new("localeCompare", 1, false, false),
            ),
            (
                js3_string_match_builtin(),
                BuiltinEntryMetadata::new("match", 1, false, false),
            ),
            (
                js3_string_match_all_builtin(),
                BuiltinEntryMetadata::new("matchAll", 1, false, false),
            ),
            (
                js3_string_normalize_builtin(),
                BuiltinEntryMetadata::new("normalize", 0, false, false),
            ),
            (
                js3_string_last_index_of_builtin(),
                BuiltinEntryMetadata::new("lastIndexOf", 1, false, false),
            ),
            (
                js3_string_pad_end_builtin(),
                BuiltinEntryMetadata::new("padEnd", 1, false, false),
            ),
            (
                js3_string_pad_start_builtin(),
                BuiltinEntryMetadata::new("padStart", 1, false, false),
            ),
            (
                js3_string_repeat_builtin(),
                BuiltinEntryMetadata::new("repeat", 1, false, false),
            ),
            (
                js3_string_replace_builtin(),
                BuiltinEntryMetadata::new("replace", 2, false, false),
            ),
            (
                js3_string_replace_all_builtin(),
                BuiltinEntryMetadata::new("replaceAll", 2, false, false),
            ),
            (
                js3_string_search_builtin(),
                BuiltinEntryMetadata::new("search", 1, false, false),
            ),
            (
                js3_string_split_builtin(),
                BuiltinEntryMetadata::new("split", 2, false, false),
            ),
            (
                js3_string_slice_builtin(),
                BuiltinEntryMetadata::new("slice", 2, false, false),
            ),
            (
                js3_string_substring_builtin(),
                BuiltinEntryMetadata::new("substring", 2, false, false),
            ),
            (
                js3_string_starts_with_builtin(),
                BuiltinEntryMetadata::new("startsWith", 1, false, false),
            ),
            (
                js3_string_to_locale_lower_case_builtin(),
                BuiltinEntryMetadata::new("toLocaleLowerCase", 0, false, false),
            ),
            (
                js3_string_to_locale_upper_case_builtin(),
                BuiltinEntryMetadata::new("toLocaleUpperCase", 0, false, false),
            ),
            (
                js3_string_to_lower_case_builtin(),
                BuiltinEntryMetadata::new("toLowerCase", 0, false, false),
            ),
            (
                js3_string_to_upper_case_builtin(),
                BuiltinEntryMetadata::new("toUpperCase", 0, false, false),
            ),
            (
                js3_string_to_well_formed_builtin(),
                BuiltinEntryMetadata::new("toWellFormed", 0, false, false),
            ),
            (
                js3_string_trim_builtin(),
                BuiltinEntryMetadata::new("trim", 0, false, false),
            ),
            (
                js3_string_trim_end_builtin(),
                BuiltinEntryMetadata::new("trimEnd", 0, false, false),
            ),
            (
                js3_string_trim_start_builtin(),
                BuiltinEntryMetadata::new("trimStart", 0, false, false),
            ),
        ];

        assert_eq!(PUBLIC_TEXT_BUILTIN_METADATA.len(), expected.len());
        for (entry, metadata) in expected {
            assert_eq!(text_public_builtin_metadata(entry), Some(metadata));
            assert_eq!(public_builtin_metadata(entry), Some(metadata));
        }
    }

    #[test]
    fn regexp_public_metadata_table_matches_public_lookup() {
        let expected = [
            (
                js3_regexp_builtin(),
                BuiltinEntryMetadata::new("RegExp", 2, true, true),
            ),
            (
                js3_regexp_escape_builtin(),
                BuiltinEntryMetadata::new("escape", 1, false, false),
            ),
            (
                js3_regexp_to_string_builtin(),
                BuiltinEntryMetadata::new("toString", 0, false, false),
            ),
            (
                js3_regexp_exec_builtin(),
                BuiltinEntryMetadata::new("exec", 1, false, false),
            ),
            (
                js3_regexp_test_builtin(),
                BuiltinEntryMetadata::new("test", 1, false, false),
            ),
            (
                js3_regexp_global_getter_builtin(),
                BuiltinEntryMetadata::new("get global", 0, false, false),
            ),
            (
                js3_regexp_ignore_case_getter_builtin(),
                BuiltinEntryMetadata::new("get ignoreCase", 0, false, false),
            ),
            (
                js3_regexp_multiline_getter_builtin(),
                BuiltinEntryMetadata::new("get multiline", 0, false, false),
            ),
            (
                js3_regexp_dot_all_getter_builtin(),
                BuiltinEntryMetadata::new("get dotAll", 0, false, false),
            ),
            (
                js3_regexp_unicode_getter_builtin(),
                BuiltinEntryMetadata::new("get unicode", 0, false, false),
            ),
            (
                js3_regexp_sticky_getter_builtin(),
                BuiltinEntryMetadata::new("get sticky", 0, false, false),
            ),
            (
                js3_regexp_source_getter_builtin(),
                BuiltinEntryMetadata::new("get source", 0, false, false),
            ),
            (
                js3_regexp_flags_getter_builtin(),
                BuiltinEntryMetadata::new("get flags", 0, false, false),
            ),
            (
                js3_regexp_has_indices_getter_builtin(),
                BuiltinEntryMetadata::new("get hasIndices", 0, false, false),
            ),
            (
                js3_regexp_species_getter_builtin(),
                BuiltinEntryMetadata::new("get [Symbol.species]", 0, false, false),
            ),
            (
                js3_regexp_symbol_match_builtin(),
                BuiltinEntryMetadata::new("[Symbol.match]", 1, false, false),
            ),
            (
                js3_regexp_symbol_replace_builtin(),
                BuiltinEntryMetadata::new("[Symbol.replace]", 2, false, false),
            ),
            (
                js3_regexp_symbol_search_builtin(),
                BuiltinEntryMetadata::new("[Symbol.search]", 1, false, false),
            ),
            (
                js3_regexp_symbol_split_builtin(),
                BuiltinEntryMetadata::new("[Symbol.split]", 2, false, false),
            ),
            (
                js3_regexp_symbol_match_all_builtin(),
                BuiltinEntryMetadata::new("[Symbol.matchAll]", 1, false, false),
            ),
        ];

        assert_eq!(PUBLIC_REGEXP_BUILTIN_METADATA.len(), expected.len());
        for (entry, metadata) in expected {
            assert_eq!(regexp_public_builtin_metadata(entry), Some(metadata));
            assert_eq!(public_builtin_metadata(entry), Some(metadata));
        }
    }

    #[test]
    fn weak_ref_public_metadata_table_matches_public_lookup() {
        let expected = [
            (
                js3_weak_ref_builtin(),
                BuiltinEntryMetadata::new("WeakRef", 1, true, true),
            ),
            (
                js3_finalization_registry_builtin(),
                BuiltinEntryMetadata::new("FinalizationRegistry", 1, true, true),
            ),
            (
                js3_weak_ref_deref_builtin(),
                BuiltinEntryMetadata::new("deref", 0, false, false),
            ),
            (
                js3_finalization_registry_register_builtin(),
                BuiltinEntryMetadata::new("register", 2, false, false),
            ),
            (
                js3_finalization_registry_unregister_builtin(),
                BuiltinEntryMetadata::new("unregister", 1, false, false),
            ),
        ];

        assert_eq!(PUBLIC_WEAK_REF_BUILTIN_METADATA.len(), expected.len());
        for (entry, metadata) in expected {
            assert_eq!(weak_ref_public_builtin_metadata(entry), Some(metadata));
            assert_eq!(public_builtin_metadata(entry), Some(metadata));
        }
    }
}

use crate::{internal::internal_builtin_metadata, BuiltinEntryMetadata};
use lyng_js_types::{
    abstract_module_source_builtin, abstract_module_source_to_string_tag_getter_builtin,
    add_async_disposable_resource_builtin, add_sync_disposable_resource_builtin,
    aggregate_error_builtin, array_at_builtin, array_buffer_builtin,
    array_buffer_byte_length_getter_builtin, array_buffer_is_view_builtin,
    array_buffer_slice_builtin, array_builtin, array_concat_builtin, array_copy_within_builtin,
    array_entries_builtin, array_every_builtin, array_fill_builtin, array_filter_builtin,
    array_find_builtin, array_find_index_builtin, array_find_last_builtin,
    array_find_last_index_builtin, array_flat_builtin, array_flat_map_builtin,
    array_for_each_builtin, array_from_async_builtin, array_from_builtin, array_includes_builtin,
    array_index_of_builtin, array_is_array_builtin, array_iterator_next_builtin,
    array_join_builtin, array_keys_builtin, array_last_index_of_builtin, array_map_builtin,
    array_of_builtin, array_pop_builtin, array_push_builtin, array_reduce_builtin,
    array_reduce_right_builtin, array_reverse_builtin, array_shift_builtin, array_slice_builtin,
    array_some_builtin, array_sort_builtin, array_species_getter_builtin, array_splice_builtin,
    array_to_locale_string_builtin, array_to_reversed_builtin, array_to_sorted_builtin,
    array_to_spliced_builtin, array_to_string_builtin, array_unshift_builtin, array_values_builtin,
    array_with_builtin, async_disposable_stack_adopt_builtin, async_disposable_stack_builtin,
    async_disposable_stack_defer_builtin, async_disposable_stack_dispose_async_builtin,
    async_disposable_stack_disposed_getter_builtin, async_disposable_stack_move_builtin,
    async_disposable_stack_use_builtin, async_disposal_resume_builtin, async_function_builtin,
    async_generator_function_builtin, async_generator_next_builtin, async_generator_return_builtin,
    async_generator_throw_builtin, atomics_add_builtin, atomics_and_builtin,
    atomics_compare_exchange_builtin, atomics_exchange_builtin, atomics_is_lock_free_builtin,
    atomics_load_builtin, atomics_notify_builtin, atomics_or_builtin, atomics_store_builtin,
    atomics_sub_builtin, atomics_wait_async_builtin, atomics_wait_builtin, atomics_xor_builtin,
    big_int64_array_builtin, big_uint64_array_builtin, bigint_as_int_n_builtin,
    bigint_as_uint_n_builtin, bigint_builtin, bigint_to_string_builtin, bigint_value_of_builtin,
    boolean_builtin, boolean_to_string_builtin, boolean_value_of_builtin,
    create_async_disposal_scope_builtin, create_sync_disposal_scope_builtin,
    data_view_buffer_getter_builtin, data_view_builtin, data_view_byte_length_getter_builtin,
    data_view_byte_offset_getter_builtin, data_view_get_float32_builtin,
    data_view_get_float64_builtin, data_view_get_int16_builtin, data_view_get_int32_builtin,
    data_view_get_int8_builtin, data_view_get_uint16_builtin, data_view_get_uint32_builtin,
    data_view_get_uint8_builtin, data_view_set_float32_builtin, data_view_set_float64_builtin,
    data_view_set_int16_builtin, data_view_set_int32_builtin, data_view_set_int8_builtin,
    data_view_set_uint16_builtin, data_view_set_uint32_builtin, data_view_set_uint8_builtin,
    date_builtin, date_get_date_builtin, date_get_day_builtin, date_get_full_year_builtin,
    date_get_hours_builtin, date_get_milliseconds_builtin, date_get_minutes_builtin,
    date_get_month_builtin, date_get_seconds_builtin, date_get_time_builtin,
    date_get_timezone_offset_builtin, date_get_utc_date_builtin, date_get_utc_day_builtin,
    date_get_utc_full_year_builtin, date_get_utc_hours_builtin, date_get_utc_milliseconds_builtin,
    date_get_utc_minutes_builtin, date_get_utc_month_builtin, date_get_utc_seconds_builtin,
    date_now_builtin, date_parse_builtin, date_set_date_builtin, date_set_full_year_builtin,
    date_set_hours_builtin, date_set_milliseconds_builtin, date_set_minutes_builtin,
    date_set_month_builtin, date_set_seconds_builtin, date_set_time_builtin,
    date_set_utc_date_builtin, date_set_utc_full_year_builtin, date_set_utc_hours_builtin,
    date_set_utc_milliseconds_builtin, date_set_utc_minutes_builtin, date_set_utc_month_builtin,
    date_set_utc_seconds_builtin, date_to_date_string_builtin, date_to_iso_string_builtin,
    date_to_json_builtin, date_to_locale_date_string_builtin, date_to_locale_string_builtin,
    date_to_locale_time_string_builtin, date_to_primitive_builtin, date_to_string_builtin,
    date_to_temporal_instant_builtin, date_to_time_string_builtin, date_to_utc_string_builtin,
    date_utc_builtin, date_value_of_builtin, decode_uri_builtin, decode_uri_component_builtin,
    disposable_stack_adopt_builtin, disposable_stack_builtin, disposable_stack_defer_builtin,
    disposable_stack_dispose_builtin, disposable_stack_disposed_getter_builtin,
    disposable_stack_move_builtin, disposable_stack_use_builtin, dispose_scope_async_builtin,
    dispose_scope_builtin, encode_uri_builtin, encode_uri_component_builtin, error_builtin,
    error_to_string_builtin, eval_builtin, eval_error_builtin, finalization_registry_builtin,
    finalization_registry_register_builtin, finalization_registry_unregister_builtin,
    float32_array_builtin, float64_array_builtin, function_apply_builtin, function_bind_builtin,
    function_builtin, function_call_builtin, function_prototype_builtin,
    function_symbol_has_instance_builtin, function_to_string_builtin, generator_function_builtin,
    generator_next_builtin, generator_return_builtin, generator_throw_builtin, int16_array_builtin,
    int32_array_builtin, int8_array_builtin, is_finite_builtin, is_nan_builtin,
    iterator_prototype_iterator_builtin, json_is_raw_json_builtin, json_parse_builtin,
    json_raw_json_builtin, json_stringify_builtin, map_builtin, map_clear_builtin,
    map_delete_builtin, map_entries_builtin, map_for_each_builtin, map_get_builtin,
    map_has_builtin, map_iterator_next_builtin, map_keys_builtin, map_set_builtin,
    map_size_getter_builtin, map_values_builtin, math_abs_builtin, math_acos_builtin,
    math_acosh_builtin, math_asin_builtin, math_asinh_builtin, math_atan2_builtin,
    math_atan_builtin, math_atanh_builtin, math_cbrt_builtin, math_ceil_builtin,
    math_clz32_builtin, math_cos_builtin, math_cosh_builtin, math_exp_builtin, math_expm1_builtin,
    math_f16round_builtin, math_floor_builtin, math_fround_builtin, math_hypot_builtin,
    math_imul_builtin, math_log10_builtin, math_log1p_builtin, math_log2_builtin, math_log_builtin,
    math_max_builtin, math_min_builtin, math_pow_builtin, math_random_builtin, math_round_builtin,
    math_sign_builtin, math_sin_builtin, math_sinh_builtin, math_sqrt_builtin,
    math_sum_precise_builtin, math_tan_builtin, math_tanh_builtin, math_trunc_builtin,
    number_builtin, number_is_finite_builtin, number_is_integer_builtin, number_is_nan_builtin,
    number_is_safe_integer_builtin, number_to_exponential_builtin, number_to_fixed_builtin,
    number_to_locale_string_builtin, number_to_precision_builtin, number_to_string_builtin,
    number_value_of_builtin, object_assign_builtin, object_builtin, object_create_builtin,
    object_define_getter_builtin, object_define_properties_builtin, object_define_property_builtin,
    object_define_setter_builtin, object_entries_builtin, object_freeze_builtin,
    object_from_entries_builtin, object_get_own_property_descriptor_builtin,
    object_get_own_property_descriptors_builtin, object_get_own_property_names_builtin,
    object_get_own_property_symbols_builtin, object_get_prototype_of_builtin,
    object_group_by_builtin, object_has_own_builtin, object_has_own_property_builtin,
    object_is_builtin, object_is_extensible_builtin, object_is_frozen_builtin,
    object_is_prototype_of_builtin, object_is_sealed_builtin, object_keys_builtin,
    object_lookup_getter_builtin, object_lookup_setter_builtin, object_prevent_extensions_builtin,
    object_property_is_enumerable_builtin, object_proto_getter_builtin,
    object_proto_setter_builtin, object_seal_builtin, object_set_prototype_of_builtin,
    object_to_locale_string_builtin, object_to_string_builtin, object_value_of_builtin,
    object_values_builtin, parse_float_builtin, parse_int_builtin, promise_all_builtin,
    promise_all_resolve_element_builtin, promise_all_settled_builtin,
    promise_all_settled_reject_element_builtin, promise_all_settled_resolve_element_builtin,
    promise_any_builtin, promise_any_reject_element_builtin, promise_builtin,
    promise_capability_executor_builtin, promise_catch_builtin, promise_finally_builtin,
    promise_finally_function_builtin, promise_race_builtin, promise_reject_builtin,
    promise_reject_function_builtin, promise_resolve_builtin, promise_resolve_function_builtin,
    promise_species_getter_builtin, promise_then_builtin, proxy_builtin, proxy_revocable_builtin,
    proxy_revoke_builtin, range_error_builtin, reference_error_builtin, reflect_apply_builtin,
    reflect_construct_builtin, reflect_define_property_builtin, reflect_delete_property_builtin,
    reflect_get_builtin, reflect_get_own_property_descriptor_builtin,
    reflect_get_prototype_of_builtin, reflect_has_builtin, reflect_is_extensible_builtin,
    reflect_own_keys_builtin, reflect_prevent_extensions_builtin, reflect_set_builtin,
    reflect_set_prototype_of_builtin, regexp_builtin, regexp_dot_all_getter_builtin,
    regexp_escape_builtin, regexp_exec_builtin, regexp_flags_getter_builtin,
    regexp_global_getter_builtin, regexp_has_indices_getter_builtin,
    regexp_ignore_case_getter_builtin, regexp_multiline_getter_builtin,
    regexp_source_getter_builtin, regexp_species_getter_builtin, regexp_sticky_getter_builtin,
    regexp_symbol_match_all_builtin, regexp_symbol_match_builtin, regexp_symbol_replace_builtin,
    regexp_symbol_search_builtin, regexp_symbol_split_builtin, regexp_test_builtin,
    regexp_to_string_builtin, regexp_unicode_getter_builtin, set_add_builtin, set_builtin,
    set_clear_builtin, set_delete_builtin, set_entries_builtin, set_for_each_builtin,
    set_has_builtin, set_iterator_next_builtin, set_keys_builtin, set_size_getter_builtin,
    set_values_builtin, shared_array_buffer_builtin,
    shared_array_buffer_byte_length_getter_builtin, shared_array_buffer_slice_builtin,
    string_at_builtin, string_builtin, string_char_at_builtin, string_char_code_at_builtin,
    string_code_point_at_builtin, string_concat_builtin, string_ends_with_builtin,
    string_from_char_code_builtin, string_from_code_point_builtin, string_includes_builtin,
    string_index_of_builtin, string_is_well_formed_builtin, string_iterator_builtin,
    string_iterator_next_builtin, string_last_index_of_builtin, string_locale_compare_builtin,
    string_match_all_builtin, string_match_builtin, string_normalize_builtin,
    string_pad_end_builtin, string_pad_start_builtin, string_raw_builtin, string_repeat_builtin,
    string_replace_all_builtin, string_replace_builtin, string_search_builtin,
    string_slice_builtin, string_split_builtin, string_starts_with_builtin,
    string_substring_builtin, string_to_locale_lower_case_builtin,
    string_to_locale_upper_case_builtin, string_to_lower_case_builtin, string_to_string_builtin,
    string_to_upper_case_builtin, string_to_well_formed_builtin, string_trim_builtin,
    string_trim_end_builtin, string_trim_start_builtin, string_value_of_builtin,
    suppressed_error_builtin, symbol_builtin, symbol_description_getter_builtin,
    symbol_for_builtin, symbol_key_for_builtin, symbol_to_primitive_builtin,
    symbol_to_string_builtin, symbol_value_of_builtin, syntax_error_builtin, type_error_builtin,
    typed_array_at_builtin, typed_array_builtin, typed_array_copy_within_builtin,
    typed_array_every_builtin, typed_array_fill_builtin, typed_array_filter_builtin,
    typed_array_find_builtin, typed_array_find_index_builtin, typed_array_find_last_builtin,
    typed_array_find_last_index_builtin, typed_array_for_each_builtin, typed_array_from_builtin,
    typed_array_includes_builtin, typed_array_index_of_builtin, typed_array_join_builtin,
    typed_array_last_index_of_builtin, typed_array_map_builtin, typed_array_of_builtin,
    typed_array_reduce_builtin, typed_array_reduce_right_builtin, typed_array_reverse_builtin,
    typed_array_some_builtin, typed_array_sort_builtin, typed_array_to_locale_string_builtin,
    typed_array_to_reversed_builtin, typed_array_to_sorted_builtin, typed_array_to_string_builtin,
    typed_array_to_string_tag_getter_builtin, typed_array_with_builtin, uint16_array_builtin,
    uint32_array_builtin, uint8_array_buffer_getter_builtin, uint8_array_builtin,
    uint8_array_byte_length_getter_builtin, uint8_array_byte_offset_getter_builtin,
    uint8_array_entries_builtin, uint8_array_keys_builtin, uint8_array_length_getter_builtin,
    uint8_array_set_builtin, uint8_array_slice_builtin, uint8_array_subarray_builtin,
    uint8_array_values_builtin, uint8_clamped_array_builtin, uri_error_builtin, weak_map_builtin,
    weak_map_delete_builtin, weak_map_get_builtin, weak_map_has_builtin, weak_map_set_builtin,
    weak_ref_builtin, weak_ref_deref_builtin, weak_set_add_builtin, weak_set_builtin,
    weak_set_delete_builtin, weak_set_has_builtin, BuiltinFunctionId,
};

mod binary_data;
mod core;
mod temporal;

use self::binary_data::PUBLIC_BINARY_DATA_BUILTIN_METADATA;
use self::core::{
    PUBLIC_ARRAY_BUILTIN_METADATA, PUBLIC_DATE_BUILTIN_METADATA, PUBLIC_FUNCTION_BUILTIN_METADATA,
    PUBLIC_KEYED_COLLECTION_BUILTIN_METADATA, PUBLIC_LANGUAGE_SUPPORT_BUILTIN_METADATA,
    PUBLIC_MODULE_BUILTIN_METADATA, PUBLIC_OBJECT_BUILTIN_METADATA,
    PUBLIC_OBJECT_REFLECTION_BUILTIN_METADATA, PUBLIC_PRIMITIVE_BUILTIN_METADATA,
    PUBLIC_REGEXP_BUILTIN_METADATA, PUBLIC_TEXT_BUILTIN_METADATA, PUBLIC_WEAK_REF_BUILTIN_METADATA,
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

fn date_public_builtin_metadata(entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
    public_builtin_metadata_from_rows(entry, PUBLIC_DATE_BUILTIN_METADATA)
}

fn primitive_public_builtin_metadata(entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
    public_builtin_metadata_from_rows(entry, PUBLIC_PRIMITIVE_BUILTIN_METADATA)
}

fn module_public_builtin_metadata(entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
    public_builtin_metadata_from_rows(entry, PUBLIC_MODULE_BUILTIN_METADATA)
}

fn language_support_public_builtin_metadata(
    entry: BuiltinFunctionId,
) -> Option<BuiltinEntryMetadata> {
    public_builtin_metadata_from_rows(entry, PUBLIC_LANGUAGE_SUPPORT_BUILTIN_METADATA)
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
    if let Some(metadata) = module_public_builtin_metadata(entry) {
        return Some(metadata);
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
    if let Some(metadata) = date_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if let Some(metadata) = temporal::temporal_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if let Some(metadata) = primitive_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if let Some(metadata) = language_support_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    None
}

#[inline]
pub fn builtin_metadata(entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
    public_builtin_metadata(entry).or_else(|| internal_builtin_metadata(entry))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_public_metadata_table_matches_public_lookup() {
        let expected = [
            (
                object_builtin(),
                BuiltinEntryMetadata::new("Object", 1, true, true),
            ),
            (
                object_create_builtin(),
                BuiltinEntryMetadata::new("create", 2, false, false),
            ),
            (
                object_get_prototype_of_builtin(),
                BuiltinEntryMetadata::new("getPrototypeOf", 1, false, false),
            ),
            (
                object_set_prototype_of_builtin(),
                BuiltinEntryMetadata::new("setPrototypeOf", 2, false, false),
            ),
            (
                object_get_own_property_descriptor_builtin(),
                BuiltinEntryMetadata::new("getOwnPropertyDescriptor", 2, false, false),
            ),
            (
                object_get_own_property_descriptors_builtin(),
                BuiltinEntryMetadata::new("getOwnPropertyDescriptors", 1, false, false),
            ),
            (
                object_get_own_property_names_builtin(),
                BuiltinEntryMetadata::new("getOwnPropertyNames", 1, false, false),
            ),
            (
                object_get_own_property_symbols_builtin(),
                BuiltinEntryMetadata::new("getOwnPropertySymbols", 1, false, false),
            ),
            (
                object_define_properties_builtin(),
                BuiltinEntryMetadata::new("defineProperties", 2, false, false),
            ),
            (
                object_define_property_builtin(),
                BuiltinEntryMetadata::new("defineProperty", 3, false, false),
            ),
            (
                object_assign_builtin(),
                BuiltinEntryMetadata::new("assign", 2, false, false),
            ),
            (
                object_from_entries_builtin(),
                BuiltinEntryMetadata::new("fromEntries", 1, false, false),
            ),
            (
                object_group_by_builtin(),
                BuiltinEntryMetadata::new("groupBy", 2, false, false),
            ),
            (
                object_prevent_extensions_builtin(),
                BuiltinEntryMetadata::new("preventExtensions", 1, false, false),
            ),
            (
                object_is_extensible_builtin(),
                BuiltinEntryMetadata::new("isExtensible", 1, false, false),
            ),
            (
                object_is_builtin(),
                BuiltinEntryMetadata::new("is", 2, false, false),
            ),
            (
                object_seal_builtin(),
                BuiltinEntryMetadata::new("seal", 1, false, false),
            ),
            (
                object_freeze_builtin(),
                BuiltinEntryMetadata::new("freeze", 1, false, false),
            ),
            (
                object_is_sealed_builtin(),
                BuiltinEntryMetadata::new("isSealed", 1, false, false),
            ),
            (
                object_is_frozen_builtin(),
                BuiltinEntryMetadata::new("isFrozen", 1, false, false),
            ),
            (
                object_to_locale_string_builtin(),
                BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
            ),
            (
                object_to_string_builtin(),
                BuiltinEntryMetadata::new("toString", 0, false, false),
            ),
            (
                object_value_of_builtin(),
                BuiltinEntryMetadata::new("valueOf", 0, false, false),
            ),
            (
                object_has_own_property_builtin(),
                BuiltinEntryMetadata::new("hasOwnProperty", 1, false, false),
            ),
            (
                object_is_prototype_of_builtin(),
                BuiltinEntryMetadata::new("isPrototypeOf", 1, false, false),
            ),
            (
                object_property_is_enumerable_builtin(),
                BuiltinEntryMetadata::new("propertyIsEnumerable", 1, false, false),
            ),
            (
                object_define_getter_builtin(),
                BuiltinEntryMetadata::new("__defineGetter__", 2, false, false),
            ),
            (
                object_define_setter_builtin(),
                BuiltinEntryMetadata::new("__defineSetter__", 2, false, false),
            ),
            (
                object_lookup_getter_builtin(),
                BuiltinEntryMetadata::new("__lookupGetter__", 1, false, false),
            ),
            (
                object_lookup_setter_builtin(),
                BuiltinEntryMetadata::new("__lookupSetter__", 1, false, false),
            ),
            (
                object_proto_getter_builtin(),
                BuiltinEntryMetadata::new("get __proto__", 0, false, false),
            ),
            (
                object_proto_setter_builtin(),
                BuiltinEntryMetadata::new("set __proto__", 1, false, false),
            ),
            (
                object_keys_builtin(),
                BuiltinEntryMetadata::new("keys", 1, false, false),
            ),
            (
                object_entries_builtin(),
                BuiltinEntryMetadata::new("entries", 1, false, false),
            ),
            (
                object_values_builtin(),
                BuiltinEntryMetadata::new("values", 1, false, false),
            ),
            (
                object_has_own_builtin(),
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
                function_builtin(),
                BuiltinEntryMetadata::new("Function", 1, true, true),
            ),
            (
                function_prototype_builtin(),
                BuiltinEntryMetadata::new("", 0, false, false),
            ),
            (
                function_call_builtin(),
                BuiltinEntryMetadata::new("call", 1, false, false),
            ),
            (
                function_apply_builtin(),
                BuiltinEntryMetadata::new("apply", 2, false, false),
            ),
            (
                function_bind_builtin(),
                BuiltinEntryMetadata::new("bind", 1, false, false),
            ),
            (
                function_to_string_builtin(),
                BuiltinEntryMetadata::new("toString", 0, false, false),
            ),
            (
                function_symbol_has_instance_builtin(),
                BuiltinEntryMetadata::new("[Symbol.hasInstance]", 1, false, false),
            ),
            (
                async_function_builtin(),
                BuiltinEntryMetadata::new("AsyncFunction", 1, true, true),
            ),
            (
                async_generator_function_builtin(),
                BuiltinEntryMetadata::new("AsyncGeneratorFunction", 1, true, true),
            ),
            (
                async_generator_next_builtin(),
                BuiltinEntryMetadata::new("next", 1, false, false),
            ),
            (
                async_generator_return_builtin(),
                BuiltinEntryMetadata::new("return", 1, false, false),
            ),
            (
                async_generator_throw_builtin(),
                BuiltinEntryMetadata::new("throw", 1, false, false),
            ),
            (
                generator_function_builtin(),
                BuiltinEntryMetadata::new("GeneratorFunction", 1, true, true),
            ),
            (
                generator_next_builtin(),
                BuiltinEntryMetadata::new("next", 1, false, false),
            ),
            (
                generator_return_builtin(),
                BuiltinEntryMetadata::new("return", 1, false, false),
            ),
            (
                generator_throw_builtin(),
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
                array_builtin(),
                BuiltinEntryMetadata::new("Array", 1, true, true),
            ),
            (
                array_from_builtin(),
                BuiltinEntryMetadata::new("from", 1, false, false),
            ),
            (
                array_from_async_builtin(),
                BuiltinEntryMetadata::new("fromAsync", 1, false, false),
            ),
            (
                array_of_builtin(),
                BuiltinEntryMetadata::new("of", 0, false, false),
            ),
            (
                array_is_array_builtin(),
                BuiltinEntryMetadata::new("isArray", 1, false, false),
            ),
            (
                array_at_builtin(),
                BuiltinEntryMetadata::new("at", 1, false, false),
            ),
            (
                array_concat_builtin(),
                BuiltinEntryMetadata::new("concat", 1, false, false),
            ),
            (
                array_copy_within_builtin(),
                BuiltinEntryMetadata::new("copyWithin", 2, false, false),
            ),
            (
                array_fill_builtin(),
                BuiltinEntryMetadata::new("fill", 1, false, false),
            ),
            (
                array_join_builtin(),
                BuiltinEntryMetadata::new("join", 1, false, false),
            ),
            (
                array_pop_builtin(),
                BuiltinEntryMetadata::new("pop", 0, false, false),
            ),
            (
                array_push_builtin(),
                BuiltinEntryMetadata::new("push", 1, false, false),
            ),
            (
                array_shift_builtin(),
                BuiltinEntryMetadata::new("shift", 0, false, false),
            ),
            (
                array_unshift_builtin(),
                BuiltinEntryMetadata::new("unshift", 1, false, false),
            ),
            (
                array_every_builtin(),
                BuiltinEntryMetadata::new("every", 1, false, false),
            ),
            (
                array_filter_builtin(),
                BuiltinEntryMetadata::new("filter", 1, false, false),
            ),
            (
                array_flat_builtin(),
                BuiltinEntryMetadata::new("flat", 0, false, false),
            ),
            (
                array_flat_map_builtin(),
                BuiltinEntryMetadata::new("flatMap", 1, false, false),
            ),
            (
                array_find_builtin(),
                BuiltinEntryMetadata::new("find", 1, false, false),
            ),
            (
                array_find_index_builtin(),
                BuiltinEntryMetadata::new("findIndex", 1, false, false),
            ),
            (
                array_find_last_builtin(),
                BuiltinEntryMetadata::new("findLast", 1, false, false),
            ),
            (
                array_find_last_index_builtin(),
                BuiltinEntryMetadata::new("findLastIndex", 1, false, false),
            ),
            (
                array_for_each_builtin(),
                BuiltinEntryMetadata::new("forEach", 1, false, false),
            ),
            (
                array_includes_builtin(),
                BuiltinEntryMetadata::new("includes", 1, false, false),
            ),
            (
                array_index_of_builtin(),
                BuiltinEntryMetadata::new("indexOf", 1, false, false),
            ),
            (
                array_map_builtin(),
                BuiltinEntryMetadata::new("map", 1, false, false),
            ),
            (
                array_reduce_builtin(),
                BuiltinEntryMetadata::new("reduce", 1, false, false),
            ),
            (
                array_reduce_right_builtin(),
                BuiltinEntryMetadata::new("reduceRight", 1, false, false),
            ),
            (
                array_reverse_builtin(),
                BuiltinEntryMetadata::new("reverse", 0, false, false),
            ),
            (
                array_slice_builtin(),
                BuiltinEntryMetadata::new("slice", 2, false, false),
            ),
            (
                array_some_builtin(),
                BuiltinEntryMetadata::new("some", 1, false, false),
            ),
            (
                array_last_index_of_builtin(),
                BuiltinEntryMetadata::new("lastIndexOf", 1, false, false),
            ),
            (
                array_sort_builtin(),
                BuiltinEntryMetadata::new("sort", 1, false, false),
            ),
            (
                array_splice_builtin(),
                BuiltinEntryMetadata::new("splice", 2, false, false),
            ),
            (
                array_to_reversed_builtin(),
                BuiltinEntryMetadata::new("toReversed", 0, false, false),
            ),
            (
                array_to_sorted_builtin(),
                BuiltinEntryMetadata::new("toSorted", 1, false, false),
            ),
            (
                array_to_spliced_builtin(),
                BuiltinEntryMetadata::new("toSpliced", 2, false, false),
            ),
            (
                array_to_string_builtin(),
                BuiltinEntryMetadata::new("toString", 0, false, false),
            ),
            (
                array_to_locale_string_builtin(),
                BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
            ),
            (
                array_values_builtin(),
                BuiltinEntryMetadata::new("values", 0, false, false),
            ),
            (
                array_keys_builtin(),
                BuiltinEntryMetadata::new("keys", 0, false, false),
            ),
            (
                array_entries_builtin(),
                BuiltinEntryMetadata::new("entries", 0, false, false),
            ),
            (
                array_with_builtin(),
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
                map_builtin(),
                BuiltinEntryMetadata::new("Map", 0, true, true),
            ),
            (
                set_builtin(),
                BuiltinEntryMetadata::new("Set", 0, true, true),
            ),
            (
                weak_map_builtin(),
                BuiltinEntryMetadata::new("WeakMap", 0, true, true),
            ),
            (
                weak_set_builtin(),
                BuiltinEntryMetadata::new("WeakSet", 0, true, true),
            ),
            (
                map_get_builtin(),
                BuiltinEntryMetadata::new("get", 1, false, false),
            ),
            (
                map_set_builtin(),
                BuiltinEntryMetadata::new("set", 2, false, false),
            ),
            (
                map_has_builtin(),
                BuiltinEntryMetadata::new("has", 1, false, false),
            ),
            (
                map_delete_builtin(),
                BuiltinEntryMetadata::new("delete", 1, false, false),
            ),
            (
                map_clear_builtin(),
                BuiltinEntryMetadata::new("clear", 0, false, false),
            ),
            (
                map_entries_builtin(),
                BuiltinEntryMetadata::new("entries", 0, false, false),
            ),
            (
                map_values_builtin(),
                BuiltinEntryMetadata::new("values", 0, false, false),
            ),
            (
                map_keys_builtin(),
                BuiltinEntryMetadata::new("keys", 0, false, false),
            ),
            (
                map_for_each_builtin(),
                BuiltinEntryMetadata::new("forEach", 1, false, false),
            ),
            (
                map_size_getter_builtin(),
                BuiltinEntryMetadata::new("get size", 0, false, false),
            ),
            (
                set_add_builtin(),
                BuiltinEntryMetadata::new("add", 1, false, false),
            ),
            (
                set_has_builtin(),
                BuiltinEntryMetadata::new("has", 1, false, false),
            ),
            (
                set_delete_builtin(),
                BuiltinEntryMetadata::new("delete", 1, false, false),
            ),
            (
                set_clear_builtin(),
                BuiltinEntryMetadata::new("clear", 0, false, false),
            ),
            (
                set_entries_builtin(),
                BuiltinEntryMetadata::new("entries", 0, false, false),
            ),
            (
                set_values_builtin(),
                BuiltinEntryMetadata::new("values", 0, false, false),
            ),
            (
                set_keys_builtin(),
                BuiltinEntryMetadata::new("keys", 0, false, false),
            ),
            (
                set_for_each_builtin(),
                BuiltinEntryMetadata::new("forEach", 1, false, false),
            ),
            (
                set_size_getter_builtin(),
                BuiltinEntryMetadata::new("get size", 0, false, false),
            ),
            (
                weak_map_get_builtin(),
                BuiltinEntryMetadata::new("get", 1, false, false),
            ),
            (
                weak_map_set_builtin(),
                BuiltinEntryMetadata::new("set", 2, false, false),
            ),
            (
                weak_map_has_builtin(),
                BuiltinEntryMetadata::new("has", 1, false, false),
            ),
            (
                weak_map_delete_builtin(),
                BuiltinEntryMetadata::new("delete", 1, false, false),
            ),
            (
                weak_set_add_builtin(),
                BuiltinEntryMetadata::new("add", 1, false, false),
            ),
            (
                weak_set_has_builtin(),
                BuiltinEntryMetadata::new("has", 1, false, false),
            ),
            (
                weak_set_delete_builtin(),
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
                array_buffer_builtin(),
                BuiltinEntryMetadata::new("ArrayBuffer", 1, true, true),
            ),
            (
                array_buffer_is_view_builtin(),
                BuiltinEntryMetadata::new("isView", 1, false, false),
            ),
            (
                shared_array_buffer_builtin(),
                BuiltinEntryMetadata::new("SharedArrayBuffer", 1, true, true),
            ),
            (
                data_view_builtin(),
                BuiltinEntryMetadata::new("DataView", 1, true, true),
            ),
            (
                typed_array_builtin(),
                BuiltinEntryMetadata::new("TypedArray", 0, true, true),
            ),
            (
                typed_array_from_builtin(),
                BuiltinEntryMetadata::new("from", 1, false, false),
            ),
            (
                typed_array_of_builtin(),
                BuiltinEntryMetadata::new("of", 0, false, false),
            ),
            (
                typed_array_every_builtin(),
                BuiltinEntryMetadata::new("every", 1, false, false),
            ),
            (
                typed_array_some_builtin(),
                BuiltinEntryMetadata::new("some", 1, false, false),
            ),
            (
                typed_array_find_builtin(),
                BuiltinEntryMetadata::new("find", 1, false, false),
            ),
            (
                typed_array_find_index_builtin(),
                BuiltinEntryMetadata::new("findIndex", 1, false, false),
            ),
            (
                typed_array_find_last_builtin(),
                BuiltinEntryMetadata::new("findLast", 1, false, false),
            ),
            (
                typed_array_find_last_index_builtin(),
                BuiltinEntryMetadata::new("findLastIndex", 1, false, false),
            ),
            (
                typed_array_fill_builtin(),
                BuiltinEntryMetadata::new("fill", 1, false, false),
            ),
            (
                typed_array_copy_within_builtin(),
                BuiltinEntryMetadata::new("copyWithin", 2, false, false),
            ),
            (
                typed_array_filter_builtin(),
                BuiltinEntryMetadata::new("filter", 1, false, false),
            ),
            (
                typed_array_for_each_builtin(),
                BuiltinEntryMetadata::new("forEach", 1, false, false),
            ),
            (
                typed_array_includes_builtin(),
                BuiltinEntryMetadata::new("includes", 1, false, false),
            ),
            (
                typed_array_index_of_builtin(),
                BuiltinEntryMetadata::new("indexOf", 1, false, false),
            ),
            (
                typed_array_join_builtin(),
                BuiltinEntryMetadata::new("join", 1, false, false),
            ),
            (
                typed_array_last_index_of_builtin(),
                BuiltinEntryMetadata::new("lastIndexOf", 1, false, false),
            ),
            (
                typed_array_map_builtin(),
                BuiltinEntryMetadata::new("map", 1, false, false),
            ),
            (
                typed_array_reduce_builtin(),
                BuiltinEntryMetadata::new("reduce", 1, false, false),
            ),
            (
                typed_array_reduce_right_builtin(),
                BuiltinEntryMetadata::new("reduceRight", 1, false, false),
            ),
            (
                typed_array_reverse_builtin(),
                BuiltinEntryMetadata::new("reverse", 0, false, false),
            ),
            (
                typed_array_sort_builtin(),
                BuiltinEntryMetadata::new("sort", 1, false, false),
            ),
            (
                typed_array_to_locale_string_builtin(),
                BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
            ),
            (
                typed_array_to_string_builtin(),
                BuiltinEntryMetadata::new("toString", 0, false, false),
            ),
            (
                typed_array_to_reversed_builtin(),
                BuiltinEntryMetadata::new("toReversed", 0, false, false),
            ),
            (
                typed_array_to_sorted_builtin(),
                BuiltinEntryMetadata::new("toSorted", 1, false, false),
            ),
            (
                typed_array_with_builtin(),
                BuiltinEntryMetadata::new("with", 2, false, false),
            ),
            (
                int8_array_builtin(),
                BuiltinEntryMetadata::new("Int8Array", 3, true, true),
            ),
            (
                int16_array_builtin(),
                BuiltinEntryMetadata::new("Int16Array", 3, true, true),
            ),
            (
                int32_array_builtin(),
                BuiltinEntryMetadata::new("Int32Array", 3, true, true),
            ),
            (
                float32_array_builtin(),
                BuiltinEntryMetadata::new("Float32Array", 3, true, true),
            ),
            (
                float64_array_builtin(),
                BuiltinEntryMetadata::new("Float64Array", 3, true, true),
            ),
            (
                big_int64_array_builtin(),
                BuiltinEntryMetadata::new("BigInt64Array", 3, true, true),
            ),
            (
                big_uint64_array_builtin(),
                BuiltinEntryMetadata::new("BigUint64Array", 3, true, true),
            ),
            (
                uint32_array_builtin(),
                BuiltinEntryMetadata::new("Uint32Array", 3, true, true),
            ),
            (
                uint16_array_builtin(),
                BuiltinEntryMetadata::new("Uint16Array", 3, true, true),
            ),
            (
                uint8_clamped_array_builtin(),
                BuiltinEntryMetadata::new("Uint8ClampedArray", 3, true, true),
            ),
            (
                uint8_array_builtin(),
                BuiltinEntryMetadata::new("Uint8Array", 3, true, true),
            ),
            (
                array_buffer_byte_length_getter_builtin(),
                BuiltinEntryMetadata::new("get byteLength", 0, false, false),
            ),
            (
                array_buffer_slice_builtin(),
                BuiltinEntryMetadata::new("slice", 2, false, false),
            ),
            (
                shared_array_buffer_byte_length_getter_builtin(),
                BuiltinEntryMetadata::new("get byteLength", 0, false, false),
            ),
            (
                shared_array_buffer_slice_builtin(),
                BuiltinEntryMetadata::new("slice", 2, false, false),
            ),
            (
                atomics_load_builtin(),
                BuiltinEntryMetadata::new("load", 2, false, false),
            ),
            (
                atomics_store_builtin(),
                BuiltinEntryMetadata::new("store", 3, false, false),
            ),
            (
                atomics_add_builtin(),
                BuiltinEntryMetadata::new("add", 3, false, false),
            ),
            (
                atomics_sub_builtin(),
                BuiltinEntryMetadata::new("sub", 3, false, false),
            ),
            (
                atomics_and_builtin(),
                BuiltinEntryMetadata::new("and", 3, false, false),
            ),
            (
                atomics_or_builtin(),
                BuiltinEntryMetadata::new("or", 3, false, false),
            ),
            (
                atomics_xor_builtin(),
                BuiltinEntryMetadata::new("xor", 3, false, false),
            ),
            (
                atomics_exchange_builtin(),
                BuiltinEntryMetadata::new("exchange", 3, false, false),
            ),
            (
                atomics_compare_exchange_builtin(),
                BuiltinEntryMetadata::new("compareExchange", 4, false, false),
            ),
            (
                atomics_notify_builtin(),
                BuiltinEntryMetadata::new("notify", 3, false, false),
            ),
            (
                atomics_wait_builtin(),
                BuiltinEntryMetadata::new("wait", 4, false, false),
            ),
            (
                atomics_wait_async_builtin(),
                BuiltinEntryMetadata::new("waitAsync", 4, false, false),
            ),
            (
                atomics_is_lock_free_builtin(),
                BuiltinEntryMetadata::new("isLockFree", 1, false, false),
            ),
            (
                data_view_buffer_getter_builtin(),
                BuiltinEntryMetadata::new("get buffer", 0, false, false),
            ),
            (
                data_view_byte_length_getter_builtin(),
                BuiltinEntryMetadata::new("get byteLength", 0, false, false),
            ),
            (
                data_view_byte_offset_getter_builtin(),
                BuiltinEntryMetadata::new("get byteOffset", 0, false, false),
            ),
            (
                data_view_get_float32_builtin(),
                BuiltinEntryMetadata::new("getFloat32", 1, false, false),
            ),
            (
                data_view_get_float64_builtin(),
                BuiltinEntryMetadata::new("getFloat64", 1, false, false),
            ),
            (
                data_view_get_int16_builtin(),
                BuiltinEntryMetadata::new("getInt16", 1, false, false),
            ),
            (
                data_view_get_int32_builtin(),
                BuiltinEntryMetadata::new("getInt32", 1, false, false),
            ),
            (
                data_view_get_int8_builtin(),
                BuiltinEntryMetadata::new("getInt8", 1, false, false),
            ),
            (
                data_view_get_uint16_builtin(),
                BuiltinEntryMetadata::new("getUint16", 1, false, false),
            ),
            (
                data_view_get_uint32_builtin(),
                BuiltinEntryMetadata::new("getUint32", 1, false, false),
            ),
            (
                data_view_get_uint8_builtin(),
                BuiltinEntryMetadata::new("getUint8", 1, false, false),
            ),
            (
                data_view_set_float32_builtin(),
                BuiltinEntryMetadata::new("setFloat32", 2, false, false),
            ),
            (
                data_view_set_float64_builtin(),
                BuiltinEntryMetadata::new("setFloat64", 2, false, false),
            ),
            (
                data_view_set_int16_builtin(),
                BuiltinEntryMetadata::new("setInt16", 2, false, false),
            ),
            (
                data_view_set_int32_builtin(),
                BuiltinEntryMetadata::new("setInt32", 2, false, false),
            ),
            (
                data_view_set_int8_builtin(),
                BuiltinEntryMetadata::new("setInt8", 2, false, false),
            ),
            (
                data_view_set_uint16_builtin(),
                BuiltinEntryMetadata::new("setUint16", 2, false, false),
            ),
            (
                data_view_set_uint32_builtin(),
                BuiltinEntryMetadata::new("setUint32", 2, false, false),
            ),
            (
                data_view_set_uint8_builtin(),
                BuiltinEntryMetadata::new("setUint8", 2, false, false),
            ),
            (
                uint8_array_buffer_getter_builtin(),
                BuiltinEntryMetadata::new("get buffer", 0, false, false),
            ),
            (
                uint8_array_byte_length_getter_builtin(),
                BuiltinEntryMetadata::new("get byteLength", 0, false, false),
            ),
            (
                uint8_array_byte_offset_getter_builtin(),
                BuiltinEntryMetadata::new("get byteOffset", 0, false, false),
            ),
            (
                uint8_array_length_getter_builtin(),
                BuiltinEntryMetadata::new("get length", 0, false, false),
            ),
            (
                uint8_array_values_builtin(),
                BuiltinEntryMetadata::new("values", 0, false, false),
            ),
            (
                uint8_array_keys_builtin(),
                BuiltinEntryMetadata::new("keys", 0, false, false),
            ),
            (
                uint8_array_entries_builtin(),
                BuiltinEntryMetadata::new("entries", 0, false, false),
            ),
            (
                uint8_array_set_builtin(),
                BuiltinEntryMetadata::new("set", 1, false, false),
            ),
            (
                uint8_array_slice_builtin(),
                BuiltinEntryMetadata::new("slice", 2, false, false),
            ),
            (
                uint8_array_subarray_builtin(),
                BuiltinEntryMetadata::new("subarray", 2, false, false),
            ),
            (
                typed_array_at_builtin(),
                BuiltinEntryMetadata::new("at", 1, false, false),
            ),
            (
                typed_array_to_string_tag_getter_builtin(),
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
                json_parse_builtin(),
                BuiltinEntryMetadata::new("parse", 2, false, false),
            ),
            (
                json_stringify_builtin(),
                BuiltinEntryMetadata::new("stringify", 3, false, false),
            ),
            (
                json_raw_json_builtin(),
                BuiltinEntryMetadata::new("rawJSON", 1, false, false),
            ),
            (
                json_is_raw_json_builtin(),
                BuiltinEntryMetadata::new("isRawJSON", 1, false, false),
            ),
            (
                reflect_apply_builtin(),
                BuiltinEntryMetadata::new("apply", 3, false, false),
            ),
            (
                reflect_construct_builtin(),
                BuiltinEntryMetadata::new("construct", 2, false, false),
            ),
            (
                reflect_define_property_builtin(),
                BuiltinEntryMetadata::new("defineProperty", 3, false, false),
            ),
            (
                reflect_delete_property_builtin(),
                BuiltinEntryMetadata::new("deleteProperty", 2, false, false),
            ),
            (
                reflect_get_builtin(),
                BuiltinEntryMetadata::new("get", 2, false, false),
            ),
            (
                reflect_get_own_property_descriptor_builtin(),
                BuiltinEntryMetadata::new("getOwnPropertyDescriptor", 2, false, false),
            ),
            (
                reflect_get_prototype_of_builtin(),
                BuiltinEntryMetadata::new("getPrototypeOf", 1, false, false),
            ),
            (
                reflect_has_builtin(),
                BuiltinEntryMetadata::new("has", 2, false, false),
            ),
            (
                reflect_is_extensible_builtin(),
                BuiltinEntryMetadata::new("isExtensible", 1, false, false),
            ),
            (
                reflect_own_keys_builtin(),
                BuiltinEntryMetadata::new("ownKeys", 1, false, false),
            ),
            (
                reflect_prevent_extensions_builtin(),
                BuiltinEntryMetadata::new("preventExtensions", 1, false, false),
            ),
            (
                reflect_set_builtin(),
                BuiltinEntryMetadata::new("set", 3, false, false),
            ),
            (
                reflect_set_prototype_of_builtin(),
                BuiltinEntryMetadata::new("setPrototypeOf", 2, false, false),
            ),
            (
                proxy_builtin(),
                BuiltinEntryMetadata::new("Proxy", 2, true, false),
            ),
            (
                proxy_revocable_builtin(),
                BuiltinEntryMetadata::new("revocable", 2, false, false),
            ),
            (
                proxy_revoke_builtin(),
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
                iterator_prototype_iterator_builtin(),
                BuiltinEntryMetadata::new("[Symbol.iterator]", 0, false, false),
            ),
            (
                array_iterator_next_builtin(),
                BuiltinEntryMetadata::new("next", 0, false, false),
            ),
            (
                map_iterator_next_builtin(),
                BuiltinEntryMetadata::new("next", 0, false, false),
            ),
            (
                set_iterator_next_builtin(),
                BuiltinEntryMetadata::new("next", 0, false, false),
            ),
            (
                string_builtin(),
                BuiltinEntryMetadata::new("String", 1, true, true),
            ),
            (
                string_iterator_builtin(),
                BuiltinEntryMetadata::new("[Symbol.iterator]", 0, false, false),
            ),
            (
                string_iterator_next_builtin(),
                BuiltinEntryMetadata::new("next", 0, false, false),
            ),
            (
                string_to_string_builtin(),
                BuiltinEntryMetadata::new("toString", 0, false, false),
            ),
            (
                string_value_of_builtin(),
                BuiltinEntryMetadata::new("valueOf", 0, false, false),
            ),
            (
                string_concat_builtin(),
                BuiltinEntryMetadata::new("concat", 1, false, false),
            ),
            (
                string_char_at_builtin(),
                BuiltinEntryMetadata::new("charAt", 1, false, false),
            ),
            (
                string_char_code_at_builtin(),
                BuiltinEntryMetadata::new("charCodeAt", 1, false, false),
            ),
            (
                string_from_char_code_builtin(),
                BuiltinEntryMetadata::new("fromCharCode", 1, false, false),
            ),
            (
                string_from_code_point_builtin(),
                BuiltinEntryMetadata::new("fromCodePoint", 1, false, false),
            ),
            (
                string_raw_builtin(),
                BuiltinEntryMetadata::new("raw", 1, false, false),
            ),
            (
                string_at_builtin(),
                BuiltinEntryMetadata::new("at", 1, false, false),
            ),
            (
                string_code_point_at_builtin(),
                BuiltinEntryMetadata::new("codePointAt", 1, false, false),
            ),
            (
                string_ends_with_builtin(),
                BuiltinEntryMetadata::new("endsWith", 1, false, false),
            ),
            (
                string_includes_builtin(),
                BuiltinEntryMetadata::new("includes", 1, false, false),
            ),
            (
                string_index_of_builtin(),
                BuiltinEntryMetadata::new("indexOf", 1, false, false),
            ),
            (
                string_is_well_formed_builtin(),
                BuiltinEntryMetadata::new("isWellFormed", 0, false, false),
            ),
            (
                string_locale_compare_builtin(),
                BuiltinEntryMetadata::new("localeCompare", 1, false, false),
            ),
            (
                string_match_builtin(),
                BuiltinEntryMetadata::new("match", 1, false, false),
            ),
            (
                string_match_all_builtin(),
                BuiltinEntryMetadata::new("matchAll", 1, false, false),
            ),
            (
                string_normalize_builtin(),
                BuiltinEntryMetadata::new("normalize", 0, false, false),
            ),
            (
                string_last_index_of_builtin(),
                BuiltinEntryMetadata::new("lastIndexOf", 1, false, false),
            ),
            (
                string_pad_end_builtin(),
                BuiltinEntryMetadata::new("padEnd", 1, false, false),
            ),
            (
                string_pad_start_builtin(),
                BuiltinEntryMetadata::new("padStart", 1, false, false),
            ),
            (
                string_repeat_builtin(),
                BuiltinEntryMetadata::new("repeat", 1, false, false),
            ),
            (
                string_replace_builtin(),
                BuiltinEntryMetadata::new("replace", 2, false, false),
            ),
            (
                string_replace_all_builtin(),
                BuiltinEntryMetadata::new("replaceAll", 2, false, false),
            ),
            (
                string_search_builtin(),
                BuiltinEntryMetadata::new("search", 1, false, false),
            ),
            (
                string_split_builtin(),
                BuiltinEntryMetadata::new("split", 2, false, false),
            ),
            (
                string_slice_builtin(),
                BuiltinEntryMetadata::new("slice", 2, false, false),
            ),
            (
                string_substring_builtin(),
                BuiltinEntryMetadata::new("substring", 2, false, false),
            ),
            (
                string_starts_with_builtin(),
                BuiltinEntryMetadata::new("startsWith", 1, false, false),
            ),
            (
                string_to_locale_lower_case_builtin(),
                BuiltinEntryMetadata::new("toLocaleLowerCase", 0, false, false),
            ),
            (
                string_to_locale_upper_case_builtin(),
                BuiltinEntryMetadata::new("toLocaleUpperCase", 0, false, false),
            ),
            (
                string_to_lower_case_builtin(),
                BuiltinEntryMetadata::new("toLowerCase", 0, false, false),
            ),
            (
                string_to_upper_case_builtin(),
                BuiltinEntryMetadata::new("toUpperCase", 0, false, false),
            ),
            (
                string_to_well_formed_builtin(),
                BuiltinEntryMetadata::new("toWellFormed", 0, false, false),
            ),
            (
                string_trim_builtin(),
                BuiltinEntryMetadata::new("trim", 0, false, false),
            ),
            (
                string_trim_end_builtin(),
                BuiltinEntryMetadata::new("trimEnd", 0, false, false),
            ),
            (
                string_trim_start_builtin(),
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
                regexp_builtin(),
                BuiltinEntryMetadata::new("RegExp", 2, true, true),
            ),
            (
                regexp_escape_builtin(),
                BuiltinEntryMetadata::new("escape", 1, false, false),
            ),
            (
                regexp_to_string_builtin(),
                BuiltinEntryMetadata::new("toString", 0, false, false),
            ),
            (
                regexp_exec_builtin(),
                BuiltinEntryMetadata::new("exec", 1, false, false),
            ),
            (
                regexp_test_builtin(),
                BuiltinEntryMetadata::new("test", 1, false, false),
            ),
            (
                regexp_global_getter_builtin(),
                BuiltinEntryMetadata::new("get global", 0, false, false),
            ),
            (
                regexp_ignore_case_getter_builtin(),
                BuiltinEntryMetadata::new("get ignoreCase", 0, false, false),
            ),
            (
                regexp_multiline_getter_builtin(),
                BuiltinEntryMetadata::new("get multiline", 0, false, false),
            ),
            (
                regexp_dot_all_getter_builtin(),
                BuiltinEntryMetadata::new("get dotAll", 0, false, false),
            ),
            (
                regexp_unicode_getter_builtin(),
                BuiltinEntryMetadata::new("get unicode", 0, false, false),
            ),
            (
                regexp_sticky_getter_builtin(),
                BuiltinEntryMetadata::new("get sticky", 0, false, false),
            ),
            (
                regexp_source_getter_builtin(),
                BuiltinEntryMetadata::new("get source", 0, false, false),
            ),
            (
                regexp_flags_getter_builtin(),
                BuiltinEntryMetadata::new("get flags", 0, false, false),
            ),
            (
                regexp_has_indices_getter_builtin(),
                BuiltinEntryMetadata::new("get hasIndices", 0, false, false),
            ),
            (
                regexp_species_getter_builtin(),
                BuiltinEntryMetadata::new("get [Symbol.species]", 0, false, false),
            ),
            (
                regexp_symbol_match_builtin(),
                BuiltinEntryMetadata::new("[Symbol.match]", 1, false, false),
            ),
            (
                regexp_symbol_replace_builtin(),
                BuiltinEntryMetadata::new("[Symbol.replace]", 2, false, false),
            ),
            (
                regexp_symbol_search_builtin(),
                BuiltinEntryMetadata::new("[Symbol.search]", 1, false, false),
            ),
            (
                regexp_symbol_split_builtin(),
                BuiltinEntryMetadata::new("[Symbol.split]", 2, false, false),
            ),
            (
                regexp_symbol_match_all_builtin(),
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
    fn date_public_metadata_table_matches_public_lookup() {
        let expected = [
            (
                date_builtin(),
                BuiltinEntryMetadata::new("Date", 7, true, true),
            ),
            (
                date_now_builtin(),
                BuiltinEntryMetadata::new("now", 0, false, false),
            ),
            (
                date_parse_builtin(),
                BuiltinEntryMetadata::new("parse", 1, false, false),
            ),
            (
                date_utc_builtin(),
                BuiltinEntryMetadata::new("UTC", 7, false, false),
            ),
            (
                date_to_string_builtin(),
                BuiltinEntryMetadata::new("toString", 0, false, false),
            ),
            (
                date_to_date_string_builtin(),
                BuiltinEntryMetadata::new("toDateString", 0, false, false),
            ),
            (
                date_to_time_string_builtin(),
                BuiltinEntryMetadata::new("toTimeString", 0, false, false),
            ),
            (
                date_to_locale_string_builtin(),
                BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
            ),
            (
                date_to_locale_date_string_builtin(),
                BuiltinEntryMetadata::new("toLocaleDateString", 0, false, false),
            ),
            (
                date_to_locale_time_string_builtin(),
                BuiltinEntryMetadata::new("toLocaleTimeString", 0, false, false),
            ),
            (
                date_value_of_builtin(),
                BuiltinEntryMetadata::new("valueOf", 0, false, false),
            ),
            (
                date_get_time_builtin(),
                BuiltinEntryMetadata::new("getTime", 0, false, false),
            ),
            (
                date_get_full_year_builtin(),
                BuiltinEntryMetadata::new("getFullYear", 0, false, false),
            ),
            (
                date_get_utc_full_year_builtin(),
                BuiltinEntryMetadata::new("getUTCFullYear", 0, false, false),
            ),
            (
                date_get_month_builtin(),
                BuiltinEntryMetadata::new("getMonth", 0, false, false),
            ),
            (
                date_get_utc_month_builtin(),
                BuiltinEntryMetadata::new("getUTCMonth", 0, false, false),
            ),
            (
                date_get_date_builtin(),
                BuiltinEntryMetadata::new("getDate", 0, false, false),
            ),
            (
                date_get_utc_date_builtin(),
                BuiltinEntryMetadata::new("getUTCDate", 0, false, false),
            ),
            (
                date_get_day_builtin(),
                BuiltinEntryMetadata::new("getDay", 0, false, false),
            ),
            (
                date_get_utc_day_builtin(),
                BuiltinEntryMetadata::new("getUTCDay", 0, false, false),
            ),
            (
                date_get_hours_builtin(),
                BuiltinEntryMetadata::new("getHours", 0, false, false),
            ),
            (
                date_get_utc_hours_builtin(),
                BuiltinEntryMetadata::new("getUTCHours", 0, false, false),
            ),
            (
                date_get_minutes_builtin(),
                BuiltinEntryMetadata::new("getMinutes", 0, false, false),
            ),
            (
                date_get_utc_minutes_builtin(),
                BuiltinEntryMetadata::new("getUTCMinutes", 0, false, false),
            ),
            (
                date_get_seconds_builtin(),
                BuiltinEntryMetadata::new("getSeconds", 0, false, false),
            ),
            (
                date_get_utc_seconds_builtin(),
                BuiltinEntryMetadata::new("getUTCSeconds", 0, false, false),
            ),
            (
                date_get_milliseconds_builtin(),
                BuiltinEntryMetadata::new("getMilliseconds", 0, false, false),
            ),
            (
                date_get_utc_milliseconds_builtin(),
                BuiltinEntryMetadata::new("getUTCMilliseconds", 0, false, false),
            ),
            (
                date_get_timezone_offset_builtin(),
                BuiltinEntryMetadata::new("getTimezoneOffset", 0, false, false),
            ),
            (
                date_set_time_builtin(),
                BuiltinEntryMetadata::new("setTime", 1, false, false),
            ),
            (
                date_set_milliseconds_builtin(),
                BuiltinEntryMetadata::new("setMilliseconds", 1, false, false),
            ),
            (
                date_set_utc_milliseconds_builtin(),
                BuiltinEntryMetadata::new("setUTCMilliseconds", 1, false, false),
            ),
            (
                date_set_seconds_builtin(),
                BuiltinEntryMetadata::new("setSeconds", 2, false, false),
            ),
            (
                date_set_utc_seconds_builtin(),
                BuiltinEntryMetadata::new("setUTCSeconds", 2, false, false),
            ),
            (
                date_set_minutes_builtin(),
                BuiltinEntryMetadata::new("setMinutes", 3, false, false),
            ),
            (
                date_set_utc_minutes_builtin(),
                BuiltinEntryMetadata::new("setUTCMinutes", 3, false, false),
            ),
            (
                date_set_hours_builtin(),
                BuiltinEntryMetadata::new("setHours", 4, false, false),
            ),
            (
                date_set_utc_hours_builtin(),
                BuiltinEntryMetadata::new("setUTCHours", 4, false, false),
            ),
            (
                date_set_date_builtin(),
                BuiltinEntryMetadata::new("setDate", 1, false, false),
            ),
            (
                date_set_utc_date_builtin(),
                BuiltinEntryMetadata::new("setUTCDate", 1, false, false),
            ),
            (
                date_set_month_builtin(),
                BuiltinEntryMetadata::new("setMonth", 2, false, false),
            ),
            (
                date_set_utc_month_builtin(),
                BuiltinEntryMetadata::new("setUTCMonth", 2, false, false),
            ),
            (
                date_set_full_year_builtin(),
                BuiltinEntryMetadata::new("setFullYear", 3, false, false),
            ),
            (
                date_set_utc_full_year_builtin(),
                BuiltinEntryMetadata::new("setUTCFullYear", 3, false, false),
            ),
            (
                date_to_utc_string_builtin(),
                BuiltinEntryMetadata::new("toUTCString", 0, false, false),
            ),
            (
                date_to_iso_string_builtin(),
                BuiltinEntryMetadata::new("toISOString", 0, false, false),
            ),
            (
                date_to_json_builtin(),
                BuiltinEntryMetadata::new("toJSON", 1, false, false),
            ),
            (
                date_to_temporal_instant_builtin(),
                BuiltinEntryMetadata::new("toTemporalInstant", 0, false, false),
            ),
            (
                date_to_primitive_builtin(),
                BuiltinEntryMetadata::new("[Symbol.toPrimitive]", 1, false, false),
            ),
        ];

        assert_eq!(PUBLIC_DATE_BUILTIN_METADATA.len(), expected.len());
        for (entry, metadata) in expected {
            assert_eq!(date_public_builtin_metadata(entry), Some(metadata));
            assert_eq!(public_builtin_metadata(entry), Some(metadata));
        }
    }

    #[test]
    fn primitive_public_metadata_table_matches_public_lookup() {
        let expected = [
            (
                number_builtin(),
                BuiltinEntryMetadata::new("Number", 1, true, true),
            ),
            (
                number_is_finite_builtin(),
                BuiltinEntryMetadata::new("isFinite", 1, false, false),
            ),
            (
                number_is_integer_builtin(),
                BuiltinEntryMetadata::new("isInteger", 1, false, false),
            ),
            (
                number_is_nan_builtin(),
                BuiltinEntryMetadata::new("isNaN", 1, false, false),
            ),
            (
                number_is_safe_integer_builtin(),
                BuiltinEntryMetadata::new("isSafeInteger", 1, false, false),
            ),
            (
                number_to_exponential_builtin(),
                BuiltinEntryMetadata::new("toExponential", 1, false, false),
            ),
            (
                number_to_fixed_builtin(),
                BuiltinEntryMetadata::new("toFixed", 1, false, false),
            ),
            (
                number_to_locale_string_builtin(),
                BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
            ),
            (
                number_to_precision_builtin(),
                BuiltinEntryMetadata::new("toPrecision", 1, false, false),
            ),
            (
                number_to_string_builtin(),
                BuiltinEntryMetadata::new("toString", 1, false, false),
            ),
            (
                number_value_of_builtin(),
                BuiltinEntryMetadata::new("valueOf", 0, false, false),
            ),
            (
                math_abs_builtin(),
                BuiltinEntryMetadata::new("abs", 1, false, false),
            ),
            (
                math_acos_builtin(),
                BuiltinEntryMetadata::new("acos", 1, false, false),
            ),
            (
                math_acosh_builtin(),
                BuiltinEntryMetadata::new("acosh", 1, false, false),
            ),
            (
                math_asin_builtin(),
                BuiltinEntryMetadata::new("asin", 1, false, false),
            ),
            (
                math_asinh_builtin(),
                BuiltinEntryMetadata::new("asinh", 1, false, false),
            ),
            (
                math_atan_builtin(),
                BuiltinEntryMetadata::new("atan", 1, false, false),
            ),
            (
                math_atan2_builtin(),
                BuiltinEntryMetadata::new("atan2", 2, false, false),
            ),
            (
                math_atanh_builtin(),
                BuiltinEntryMetadata::new("atanh", 1, false, false),
            ),
            (
                math_cbrt_builtin(),
                BuiltinEntryMetadata::new("cbrt", 1, false, false),
            ),
            (
                math_ceil_builtin(),
                BuiltinEntryMetadata::new("ceil", 1, false, false),
            ),
            (
                math_clz32_builtin(),
                BuiltinEntryMetadata::new("clz32", 1, false, false),
            ),
            (
                math_cos_builtin(),
                BuiltinEntryMetadata::new("cos", 1, false, false),
            ),
            (
                math_cosh_builtin(),
                BuiltinEntryMetadata::new("cosh", 1, false, false),
            ),
            (
                math_exp_builtin(),
                BuiltinEntryMetadata::new("exp", 1, false, false),
            ),
            (
                math_expm1_builtin(),
                BuiltinEntryMetadata::new("expm1", 1, false, false),
            ),
            (
                math_f16round_builtin(),
                BuiltinEntryMetadata::new("f16round", 1, false, false),
            ),
            (
                math_floor_builtin(),
                BuiltinEntryMetadata::new("floor", 1, false, false),
            ),
            (
                math_fround_builtin(),
                BuiltinEntryMetadata::new("fround", 1, false, false),
            ),
            (
                math_hypot_builtin(),
                BuiltinEntryMetadata::new("hypot", 2, false, false),
            ),
            (
                math_imul_builtin(),
                BuiltinEntryMetadata::new("imul", 2, false, false),
            ),
            (
                math_log_builtin(),
                BuiltinEntryMetadata::new("log", 1, false, false),
            ),
            (
                math_log10_builtin(),
                BuiltinEntryMetadata::new("log10", 1, false, false),
            ),
            (
                math_log1p_builtin(),
                BuiltinEntryMetadata::new("log1p", 1, false, false),
            ),
            (
                math_log2_builtin(),
                BuiltinEntryMetadata::new("log2", 1, false, false),
            ),
            (
                math_max_builtin(),
                BuiltinEntryMetadata::new("max", 2, false, false),
            ),
            (
                math_min_builtin(),
                BuiltinEntryMetadata::new("min", 2, false, false),
            ),
            (
                math_pow_builtin(),
                BuiltinEntryMetadata::new("pow", 2, false, false),
            ),
            (
                math_random_builtin(),
                BuiltinEntryMetadata::new("random", 0, false, false),
            ),
            (
                math_round_builtin(),
                BuiltinEntryMetadata::new("round", 1, false, false),
            ),
            (
                math_sign_builtin(),
                BuiltinEntryMetadata::new("sign", 1, false, false),
            ),
            (
                math_sin_builtin(),
                BuiltinEntryMetadata::new("sin", 1, false, false),
            ),
            (
                math_sinh_builtin(),
                BuiltinEntryMetadata::new("sinh", 1, false, false),
            ),
            (
                math_sqrt_builtin(),
                BuiltinEntryMetadata::new("sqrt", 1, false, false),
            ),
            (
                math_sum_precise_builtin(),
                BuiltinEntryMetadata::new("sumPrecise", 1, false, false),
            ),
            (
                math_tan_builtin(),
                BuiltinEntryMetadata::new("tan", 1, false, false),
            ),
            (
                math_tanh_builtin(),
                BuiltinEntryMetadata::new("tanh", 1, false, false),
            ),
            (
                math_trunc_builtin(),
                BuiltinEntryMetadata::new("trunc", 1, false, false),
            ),
            (
                bigint_builtin(),
                BuiltinEntryMetadata::new("BigInt", 1, true, true),
            ),
            (
                bigint_as_int_n_builtin(),
                BuiltinEntryMetadata::new("asIntN", 2, false, false),
            ),
            (
                bigint_as_uint_n_builtin(),
                BuiltinEntryMetadata::new("asUintN", 2, false, false),
            ),
            (
                bigint_to_string_builtin(),
                BuiltinEntryMetadata::new("toString", 0, false, false),
            ),
            (
                bigint_value_of_builtin(),
                BuiltinEntryMetadata::new("valueOf", 0, false, false),
            ),
            (
                boolean_builtin(),
                BuiltinEntryMetadata::new("Boolean", 1, true, true),
            ),
            (
                boolean_to_string_builtin(),
                BuiltinEntryMetadata::new("toString", 0, false, false),
            ),
            (
                boolean_value_of_builtin(),
                BuiltinEntryMetadata::new("valueOf", 0, false, false),
            ),
            (
                symbol_builtin(),
                BuiltinEntryMetadata::new("Symbol", 0, false, true),
            ),
            (
                symbol_for_builtin(),
                BuiltinEntryMetadata::new("for", 1, false, false),
            ),
            (
                symbol_key_for_builtin(),
                BuiltinEntryMetadata::new("keyFor", 1, false, false),
            ),
            (
                symbol_to_string_builtin(),
                BuiltinEntryMetadata::new("toString", 0, false, false),
            ),
            (
                symbol_value_of_builtin(),
                BuiltinEntryMetadata::new("valueOf", 0, false, false),
            ),
            (
                symbol_to_primitive_builtin(),
                BuiltinEntryMetadata::new("[Symbol.toPrimitive]", 1, false, false),
            ),
            (
                array_species_getter_builtin(),
                BuiltinEntryMetadata::new("get [Symbol.species]", 0, false, false),
            ),
            (
                symbol_description_getter_builtin(),
                BuiltinEntryMetadata::new("get description", 0, false, false),
            ),
        ];

        assert_eq!(PUBLIC_PRIMITIVE_BUILTIN_METADATA.len(), expected.len());
        for (entry, metadata) in expected {
            assert_eq!(primitive_public_builtin_metadata(entry), Some(metadata));
            assert_eq!(public_builtin_metadata(entry), Some(metadata));
        }
    }

    #[test]
    fn module_public_metadata_table_matches_public_lookup() {
        let expected = [
            (
                abstract_module_source_builtin(),
                BuiltinEntryMetadata::new("AbstractModuleSource", 0, true, true),
            ),
            (
                abstract_module_source_to_string_tag_getter_builtin(),
                BuiltinEntryMetadata::new("get [Symbol.toStringTag]", 0, false, false),
            ),
        ];

        assert_eq!(PUBLIC_MODULE_BUILTIN_METADATA.len(), expected.len());
        for (entry, metadata) in expected {
            assert_eq!(module_public_builtin_metadata(entry), Some(metadata));
            assert_eq!(public_builtin_metadata(entry), Some(metadata));
        }
    }

    #[test]
    fn language_support_public_metadata_table_matches_public_lookup() {
        let expected = [
            (
                error_builtin(),
                BuiltinEntryMetadata::new("Error", 1, true, true),
            ),
            (
                error_to_string_builtin(),
                BuiltinEntryMetadata::new("toString", 0, false, false),
            ),
            (
                eval_error_builtin(),
                BuiltinEntryMetadata::new("EvalError", 1, true, true),
            ),
            (
                range_error_builtin(),
                BuiltinEntryMetadata::new("RangeError", 1, true, true),
            ),
            (
                reference_error_builtin(),
                BuiltinEntryMetadata::new("ReferenceError", 1, true, true),
            ),
            (
                syntax_error_builtin(),
                BuiltinEntryMetadata::new("SyntaxError", 1, true, true),
            ),
            (
                type_error_builtin(),
                BuiltinEntryMetadata::new("TypeError", 1, true, true),
            ),
            (
                uri_error_builtin(),
                BuiltinEntryMetadata::new("URIError", 1, true, true),
            ),
            (
                aggregate_error_builtin(),
                BuiltinEntryMetadata::new("AggregateError", 2, true, true),
            ),
            (
                suppressed_error_builtin(),
                BuiltinEntryMetadata::new("SuppressedError", 3, true, true),
            ),
            (
                eval_builtin(),
                BuiltinEntryMetadata::new("eval", 1, false, false),
            ),
            (
                promise_builtin(),
                BuiltinEntryMetadata::new("Promise", 1, true, true),
            ),
            (
                disposable_stack_builtin(),
                BuiltinEntryMetadata::new("DisposableStack", 0, true, true),
            ),
            (
                disposable_stack_use_builtin(),
                BuiltinEntryMetadata::new("use", 1, false, false),
            ),
            (
                disposable_stack_adopt_builtin(),
                BuiltinEntryMetadata::new("adopt", 2, false, false),
            ),
            (
                disposable_stack_defer_builtin(),
                BuiltinEntryMetadata::new("defer", 1, false, false),
            ),
            (
                disposable_stack_move_builtin(),
                BuiltinEntryMetadata::new("move", 0, false, false),
            ),
            (
                disposable_stack_disposed_getter_builtin(),
                BuiltinEntryMetadata::new("get disposed", 0, false, false),
            ),
            (
                disposable_stack_dispose_builtin(),
                BuiltinEntryMetadata::new("dispose", 0, false, false),
            ),
            (
                async_disposable_stack_builtin(),
                BuiltinEntryMetadata::new("AsyncDisposableStack", 0, true, true),
            ),
            (
                async_disposable_stack_use_builtin(),
                BuiltinEntryMetadata::new("use", 1, false, false),
            ),
            (
                async_disposable_stack_adopt_builtin(),
                BuiltinEntryMetadata::new("adopt", 2, false, false),
            ),
            (
                async_disposable_stack_defer_builtin(),
                BuiltinEntryMetadata::new("defer", 1, false, false),
            ),
            (
                async_disposable_stack_move_builtin(),
                BuiltinEntryMetadata::new("move", 0, false, false),
            ),
            (
                async_disposable_stack_disposed_getter_builtin(),
                BuiltinEntryMetadata::new("get disposed", 0, false, false),
            ),
            (
                async_disposable_stack_dispose_async_builtin(),
                BuiltinEntryMetadata::new("disposeAsync", 0, false, false),
            ),
            (
                async_disposal_resume_builtin(),
                BuiltinEntryMetadata::new("", 1, false, false),
            ),
            (
                create_sync_disposal_scope_builtin(),
                BuiltinEntryMetadata::new("", 0, false, false),
            ),
            (
                create_async_disposal_scope_builtin(),
                BuiltinEntryMetadata::new("", 0, false, false),
            ),
            (
                add_sync_disposable_resource_builtin(),
                BuiltinEntryMetadata::new("", 2, false, false),
            ),
            (
                add_async_disposable_resource_builtin(),
                BuiltinEntryMetadata::new("", 2, false, false),
            ),
            (
                dispose_scope_builtin(),
                BuiltinEntryMetadata::new("", 1, false, false),
            ),
            (
                dispose_scope_async_builtin(),
                BuiltinEntryMetadata::new("", 1, false, false),
            ),
            (
                promise_then_builtin(),
                BuiltinEntryMetadata::new("then", 2, false, false),
            ),
            (
                promise_catch_builtin(),
                BuiltinEntryMetadata::new("catch", 1, false, false),
            ),
            (
                promise_finally_builtin(),
                BuiltinEntryMetadata::new("finally", 1, false, false),
            ),
            (
                promise_finally_function_builtin(),
                BuiltinEntryMetadata::new("", 1, false, false),
            ),
            (
                promise_resolve_builtin(),
                BuiltinEntryMetadata::new("resolve", 1, false, false),
            ),
            (
                promise_reject_builtin(),
                BuiltinEntryMetadata::new("reject", 1, false, false),
            ),
            (
                promise_all_builtin(),
                BuiltinEntryMetadata::new("all", 1, false, false),
            ),
            (
                promise_all_settled_builtin(),
                BuiltinEntryMetadata::new("allSettled", 1, false, false),
            ),
            (
                promise_race_builtin(),
                BuiltinEntryMetadata::new("race", 1, false, false),
            ),
            (
                promise_any_builtin(),
                BuiltinEntryMetadata::new("any", 1, false, false),
            ),
            (
                promise_species_getter_builtin(),
                BuiltinEntryMetadata::new("get [Symbol.species]", 0, false, false),
            ),
            (
                promise_capability_executor_builtin(),
                BuiltinEntryMetadata::new("", 2, false, false),
            ),
            (
                promise_resolve_function_builtin(),
                BuiltinEntryMetadata::new("", 1, false, false),
            ),
            (
                promise_reject_function_builtin(),
                BuiltinEntryMetadata::new("", 1, false, false),
            ),
            (
                promise_all_resolve_element_builtin(),
                BuiltinEntryMetadata::new("", 1, false, false),
            ),
            (
                promise_all_settled_resolve_element_builtin(),
                BuiltinEntryMetadata::new("", 1, false, false),
            ),
            (
                promise_all_settled_reject_element_builtin(),
                BuiltinEntryMetadata::new("", 1, false, false),
            ),
            (
                promise_any_reject_element_builtin(),
                BuiltinEntryMetadata::new("", 1, false, false),
            ),
            (
                parse_int_builtin(),
                BuiltinEntryMetadata::new("parseInt", 2, false, false),
            ),
            (
                parse_float_builtin(),
                BuiltinEntryMetadata::new("parseFloat", 1, false, false),
            ),
            (
                is_nan_builtin(),
                BuiltinEntryMetadata::new("isNaN", 1, false, false),
            ),
            (
                is_finite_builtin(),
                BuiltinEntryMetadata::new("isFinite", 1, false, false),
            ),
            (
                encode_uri_builtin(),
                BuiltinEntryMetadata::new("encodeURI", 1, false, false),
            ),
            (
                encode_uri_component_builtin(),
                BuiltinEntryMetadata::new("encodeURIComponent", 1, false, false),
            ),
            (
                decode_uri_builtin(),
                BuiltinEntryMetadata::new("decodeURI", 1, false, false),
            ),
            (
                decode_uri_component_builtin(),
                BuiltinEntryMetadata::new("decodeURIComponent", 1, false, false),
            ),
        ];

        assert_eq!(
            PUBLIC_LANGUAGE_SUPPORT_BUILTIN_METADATA.len(),
            expected.len()
        );
        for (entry, metadata) in expected {
            assert_eq!(
                language_support_public_builtin_metadata(entry),
                Some(metadata)
            );
            assert_eq!(public_builtin_metadata(entry), Some(metadata));
        }
    }

    #[test]
    fn weak_ref_public_metadata_table_matches_public_lookup() {
        let expected = [
            (
                weak_ref_builtin(),
                BuiltinEntryMetadata::new("WeakRef", 1, true, true),
            ),
            (
                finalization_registry_builtin(),
                BuiltinEntryMetadata::new("FinalizationRegistry", 1, true, true),
            ),
            (
                weak_ref_deref_builtin(),
                BuiltinEntryMetadata::new("deref", 0, false, false),
            ),
            (
                finalization_registry_register_builtin(),
                BuiltinEntryMetadata::new("register", 2, false, false),
            ),
            (
                finalization_registry_unregister_builtin(),
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

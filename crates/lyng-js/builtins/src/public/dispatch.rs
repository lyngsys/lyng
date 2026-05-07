mod arrays;
mod binary_data;
mod collections;
mod date;
mod disposal;
mod error_objects;
mod functions;
mod iterators;
mod json;
mod language;
mod object_reflection;
mod objects;
mod primitives;
mod promises;
mod regexp;
mod strings;
mod support;
mod temporal;
use crate::internal::{dispatch_internal_builtin, InternalBuiltinDispatchContext};
use crate::{BuiltinInvocation, DynamicFunctionKind};
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_host::{
    TemporalCivilTime, TemporalCivilToInstantRequest, TemporalCurrentInstantRequest,
    TemporalDefaultTimeZone, TemporalDefaultTimeZoneRequest, TemporalInstant,
    TemporalInstantToCivilRequest, TemporalInstantWithOffset,
};
use lyng_js_types::{
    internal_array_index_of_builtin, internal_array_pop_builtin, internal_array_push_builtin,
    internal_object_has_own_property_builtin, internal_object_to_string_builtin,
    internal_regexp_literal_builtin, internal_string_index_of_builtin,
    internal_string_replace_builtin, is_date_builtin, AbruptCompletion, BuiltinFunctionId,
    PropertyDescriptor, PropertyKey, RealmRef, Value,
};
use support::{
    allocate_array_like_result, allocate_json_raw_object, allocate_proxy_object,
    append_string_ref_code_units, argument_to_number, array_like_index_property_key,
    array_like_join_value_for_length, array_like_length, array_like_length_u64,
    array_result_capacity_hint, array_species_create_for_length, builtin_function_entry,
    callable_object_from_value, close_iterator_after_error, code_unit_range_value,
    create_array_from_values, create_array_result, create_array_result_for_length,
    create_array_result_with_prototype, create_data_property_or_throw, define_array_length,
    define_data_property_with_attrs, delete_property_from_object, format_to_exponential,
    format_to_precision, get_property_from_object, get_property_from_object_with_receiver,
    has_property_on_object, is_array_for_species, is_concat_spreadable, is_integral_number,
    iterable_to_values_list, length_value, length_value_u64, map_completion,
    normalize_relative_index_u64, number_value, primitive_wrapper_constructor,
    property_key_from_text, property_key_string_value, property_key_value, proxy_define_property,
    proxy_delete_property, proxy_get_own_property, proxy_get_prototype_of, proxy_is_extensible,
    proxy_own_property_keys, proxy_prevent_extensions, proxy_set_prototype_of, radix_argument,
    range_error, reference_error, require_constructor_object, require_object_argument,
    require_proxy_argument_object, set_data_property_value, set_length_property,
    set_property_on_object, set_property_on_object_with_receiver, string_from_code_units,
    string_from_string_ref_range, string_ref_code_unit_len, string_ref_code_units, string_ref_text,
    string_this_ref, string_value, symbol_descriptive_string, syntax_error, to_bigint_for_builtin,
    to_boolean_for_builtin, to_index_for_builtin, to_integer_or_infinity_for_builtin,
    to_length_for_builtin, to_number_for_builtin, to_number_value_for_builtin,
    to_string_string_ref, to_uint32_for_builtin, to_uint8_clamp_for_builtin, to_uint8_for_builtin,
    try_create_data_property, try_delete_property_from_object, type_error,
    typed_array_index_is_valid, uri_error, usize_index_value, valid_array_length,
    with_string_ref_code_units, BuiltinIteratorBridge, BuiltinProxyBridge,
    BuiltinToPrimitiveBridge, MAX_SAFE_INTEGER_U64,
};

use super::{
    abstract_module_source_builtin, abstract_module_source_to_string_tag_getter_builtin,
    aggregate_error_builtin, array_at_builtin, array_buffer_builtin,
    array_buffer_byte_length_getter_builtin, array_buffer_detached_getter_builtin,
    array_buffer_is_view_builtin, array_buffer_max_byte_length_getter_builtin,
    array_buffer_resizable_getter_builtin, array_buffer_resize_builtin, array_buffer_slice_builtin,
    array_buffer_transfer_builtin, array_buffer_transfer_to_fixed_length_builtin, array_builtin,
    array_concat_builtin, array_copy_within_builtin, array_entries_builtin, array_every_builtin,
    array_fill_builtin, array_filter_builtin, array_find_builtin, array_find_index_builtin,
    array_find_last_builtin, array_find_last_index_builtin, array_flat_builtin,
    array_flat_map_builtin, array_for_each_builtin, array_from_async_builtin, array_from_builtin,
    array_includes_builtin, array_index_of_builtin, array_is_array_builtin,
    array_iterator_next_builtin, array_join_builtin, array_keys_builtin,
    array_last_index_of_builtin, array_map_builtin, array_of_builtin, array_pop_builtin,
    array_push_builtin, array_reduce_builtin, array_reduce_right_builtin, array_reverse_builtin,
    array_shift_builtin, array_slice_builtin, array_some_builtin, array_sort_builtin,
    array_species_getter_builtin, array_splice_builtin, array_to_locale_string_builtin,
    array_to_reversed_builtin, array_to_sorted_builtin, array_to_spliced_builtin,
    array_to_string_builtin, array_unshift_builtin, array_values_builtin, array_with_builtin,
    async_function_builtin, async_generator_function_builtin, async_generator_next_builtin,
    async_generator_return_builtin, async_generator_throw_builtin, async_iterator_dispose_builtin,
    atomics_add_builtin, atomics_and_builtin, atomics_compare_exchange_builtin,
    atomics_exchange_builtin, atomics_is_lock_free_builtin, atomics_load_builtin,
    atomics_notify_builtin, atomics_or_builtin, atomics_pause_builtin, atomics_store_builtin,
    atomics_sub_builtin, atomics_wait_async_builtin, atomics_wait_builtin, atomics_xor_builtin,
    big_int64_array_builtin, big_uint64_array_builtin, bigint_as_int_n_builtin,
    bigint_as_uint_n_builtin, bigint_builtin, bigint_to_string_builtin, bigint_value_of_builtin,
    boolean_builtin, boolean_to_string_builtin, boolean_value_of_builtin,
    data_view_buffer_getter_builtin, data_view_builtin, data_view_byte_length_getter_builtin,
    data_view_byte_offset_getter_builtin, data_view_get_big_int64_builtin,
    data_view_get_big_uint64_builtin, data_view_get_float16_builtin, data_view_get_float32_builtin,
    data_view_get_float64_builtin, data_view_get_int16_builtin, data_view_get_int32_builtin,
    data_view_get_int8_builtin, data_view_get_uint16_builtin, data_view_get_uint32_builtin,
    data_view_get_uint8_builtin, data_view_set_big_int64_builtin, data_view_set_big_uint64_builtin,
    data_view_set_float16_builtin, data_view_set_float32_builtin, data_view_set_float64_builtin,
    data_view_set_int16_builtin, data_view_set_int32_builtin, data_view_set_int8_builtin,
    data_view_set_uint16_builtin, data_view_set_uint32_builtin, data_view_set_uint8_builtin,
    date_builtin, date_get_date_builtin, date_get_day_builtin, date_get_full_year_builtin,
    date_get_hours_builtin, date_get_milliseconds_builtin, date_get_minutes_builtin,
    date_get_month_builtin, date_get_seconds_builtin, date_get_time_builtin,
    date_get_timezone_offset_builtin, date_get_utc_date_builtin, date_get_utc_day_builtin,
    date_get_utc_full_year_builtin, date_get_utc_hours_builtin, date_get_utc_milliseconds_builtin,
    date_get_utc_minutes_builtin, date_get_utc_month_builtin, date_get_utc_seconds_builtin,
    date_get_year_builtin, date_now_builtin, date_parse_builtin, date_set_date_builtin,
    date_set_full_year_builtin, date_set_hours_builtin, date_set_milliseconds_builtin,
    date_set_minutes_builtin, date_set_month_builtin, date_set_seconds_builtin,
    date_set_time_builtin, date_set_utc_date_builtin, date_set_utc_full_year_builtin,
    date_set_utc_hours_builtin, date_set_utc_milliseconds_builtin, date_set_utc_minutes_builtin,
    date_set_utc_month_builtin, date_set_utc_seconds_builtin, date_set_year_builtin,
    date_to_date_string_builtin, date_to_iso_string_builtin, date_to_json_builtin,
    date_to_locale_date_string_builtin, date_to_locale_string_builtin,
    date_to_locale_time_string_builtin, date_to_primitive_builtin, date_to_string_builtin,
    date_to_temporal_instant_builtin, date_to_time_string_builtin, date_to_utc_string_builtin,
    date_utc_builtin, date_value_of_builtin, decode_uri_builtin, decode_uri_component_builtin,
    encode_uri_builtin, encode_uri_component_builtin, error_builtin, error_is_error_builtin,
    error_to_string_builtin, escape_builtin, eval_builtin, eval_error_builtin,
    finalization_registry_builtin, finalization_registry_register_builtin,
    finalization_registry_unregister_builtin, float16_array_builtin, float32_array_builtin,
    float64_array_builtin, function_apply_builtin, function_bind_builtin, function_builtin,
    function_call_builtin, function_prototype_builtin, function_symbol_has_instance_builtin,
    function_to_string_builtin, generator_function_builtin, generator_next_builtin,
    generator_return_builtin, generator_throw_builtin, int16_array_builtin, int32_array_builtin,
    int8_array_builtin, is_finite_builtin, is_nan_builtin, iterator_builtin,
    iterator_concat_builtin, iterator_constructor_getter_builtin,
    iterator_constructor_setter_builtin, iterator_dispose_builtin, iterator_drop_builtin,
    iterator_every_builtin, iterator_filter_builtin, iterator_find_builtin,
    iterator_flat_map_builtin, iterator_for_each_builtin, iterator_from_builtin,
    iterator_helper_next_builtin, iterator_helper_return_builtin, iterator_map_builtin,
    iterator_prototype_iterator_builtin, iterator_reduce_builtin, iterator_some_builtin,
    iterator_take_builtin, iterator_to_array_builtin, iterator_to_string_tag_getter_builtin,
    iterator_to_string_tag_setter_builtin, iterator_zip_builtin, iterator_zip_keyed_builtin,
    json_is_raw_json_builtin, json_parse_builtin, json_raw_json_builtin, json_stringify_builtin,
    map_builtin, map_clear_builtin, map_delete_builtin, map_entries_builtin, map_for_each_builtin,
    map_get_builtin, map_get_or_insert_builtin, map_get_or_insert_computed_builtin,
    map_group_by_builtin, map_has_builtin, map_iterator_next_builtin, map_keys_builtin,
    map_set_builtin, map_size_getter_builtin, map_values_builtin, math_abs_builtin,
    math_acos_builtin, math_acosh_builtin, math_asin_builtin, math_asinh_builtin,
    math_atan2_builtin, math_atan_builtin, math_atanh_builtin, math_cbrt_builtin,
    math_ceil_builtin, math_clz32_builtin, math_cos_builtin, math_cosh_builtin, math_exp_builtin,
    math_expm1_builtin, math_f16round_builtin, math_floor_builtin, math_fround_builtin,
    math_hypot_builtin, math_imul_builtin, math_log10_builtin, math_log1p_builtin,
    math_log2_builtin, math_log_builtin, math_max_builtin, math_min_builtin, math_pow_builtin,
    math_random_builtin, math_round_builtin, math_sign_builtin, math_sin_builtin,
    math_sinh_builtin, math_sqrt_builtin, math_sum_precise_builtin, math_tan_builtin,
    math_tanh_builtin, math_trunc_builtin, number_builtin, number_is_finite_builtin,
    number_is_integer_builtin, number_is_nan_builtin, number_is_safe_integer_builtin,
    number_to_exponential_builtin, number_to_fixed_builtin, number_to_locale_string_builtin,
    number_to_precision_builtin, number_to_string_builtin, number_value_of_builtin,
    object_assign_builtin, object_builtin, object_create_builtin, object_define_getter_builtin,
    object_define_properties_builtin, object_define_property_builtin, object_define_setter_builtin,
    object_entries_builtin, object_freeze_builtin, object_from_entries_builtin,
    object_get_own_property_descriptor_builtin, object_get_own_property_descriptors_builtin,
    object_get_own_property_names_builtin, object_get_own_property_symbols_builtin,
    object_get_prototype_of_builtin, object_group_by_builtin, object_has_own_builtin,
    object_has_own_property_builtin, object_is_builtin, object_is_extensible_builtin,
    object_is_frozen_builtin, object_is_prototype_of_builtin, object_is_sealed_builtin,
    object_keys_builtin, object_lookup_getter_builtin, object_lookup_setter_builtin,
    object_prevent_extensions_builtin, object_property_is_enumerable_builtin,
    object_proto_getter_builtin, object_proto_setter_builtin, object_seal_builtin,
    object_set_prototype_of_builtin, object_to_locale_string_builtin, object_to_string_builtin,
    object_value_of_builtin, object_values_builtin, parse_float_builtin, parse_int_builtin,
    promise_all_builtin, promise_all_resolve_element_builtin, promise_all_settled_builtin,
    promise_all_settled_reject_element_builtin, promise_all_settled_resolve_element_builtin,
    promise_any_builtin, promise_any_reject_element_builtin, promise_builtin,
    promise_capability_executor_builtin, promise_catch_builtin, promise_finally_builtin,
    promise_finally_continuation_builtin, promise_finally_function_builtin, promise_race_builtin,
    promise_reject_builtin, promise_reject_function_builtin, promise_resolve_builtin,
    promise_resolve_function_builtin, promise_species_getter_builtin, promise_then_builtin,
    promise_try_builtin, promise_with_resolvers_builtin, range_error_builtin,
    reference_error_builtin, regexp_builtin, regexp_compile_builtin, regexp_dot_all_getter_builtin,
    regexp_escape_builtin, regexp_exec_builtin, regexp_flags_getter_builtin,
    regexp_global_getter_builtin, regexp_has_indices_getter_builtin,
    regexp_ignore_case_getter_builtin, regexp_legacy_input_getter_builtin,
    regexp_legacy_input_setter_builtin, regexp_legacy_last_match_getter_builtin,
    regexp_legacy_last_paren_getter_builtin, regexp_legacy_left_context_getter_builtin,
    regexp_legacy_paren1_getter_builtin, regexp_legacy_paren2_getter_builtin,
    regexp_legacy_paren3_getter_builtin, regexp_legacy_paren4_getter_builtin,
    regexp_legacy_paren5_getter_builtin, regexp_legacy_paren6_getter_builtin,
    regexp_legacy_paren7_getter_builtin, regexp_legacy_paren8_getter_builtin,
    regexp_legacy_paren9_getter_builtin, regexp_legacy_right_context_getter_builtin,
    regexp_multiline_getter_builtin, regexp_source_getter_builtin, regexp_species_getter_builtin,
    regexp_sticky_getter_builtin, regexp_string_iterator_next_builtin,
    regexp_symbol_match_all_builtin, regexp_symbol_match_builtin, regexp_symbol_replace_builtin,
    regexp_symbol_search_builtin, regexp_symbol_split_builtin, regexp_test_builtin,
    regexp_to_string_builtin, regexp_unicode_getter_builtin, regexp_unicode_sets_getter_builtin,
    set_add_builtin, set_builtin, set_clear_builtin, set_delete_builtin, set_difference_builtin,
    set_entries_builtin, set_for_each_builtin, set_has_builtin, set_intersection_builtin,
    set_is_disjoint_from_builtin, set_is_subset_of_builtin, set_is_superset_of_builtin,
    set_iterator_next_builtin, set_keys_builtin, set_size_getter_builtin,
    set_symmetric_difference_builtin, set_union_builtin, set_values_builtin,
    shared_array_buffer_builtin, shared_array_buffer_byte_length_getter_builtin,
    shared_array_buffer_grow_builtin, shared_array_buffer_growable_getter_builtin,
    shared_array_buffer_max_byte_length_getter_builtin, shared_array_buffer_slice_builtin,
    string_anchor_builtin, string_at_builtin, string_big_builtin, string_blink_builtin,
    string_bold_builtin, string_builtin, string_char_at_builtin, string_char_code_at_builtin,
    string_code_point_at_builtin, string_concat_builtin, string_ends_with_builtin,
    string_fixed_builtin, string_fontcolor_builtin, string_fontsize_builtin,
    string_from_char_code_builtin, string_from_code_point_builtin, string_includes_builtin,
    string_index_of_builtin, string_is_well_formed_builtin, string_italics_builtin,
    string_iterator_builtin, string_iterator_next_builtin, string_last_index_of_builtin,
    string_link_builtin, string_locale_compare_builtin, string_match_all_builtin,
    string_match_builtin, string_normalize_builtin, string_pad_end_builtin,
    string_pad_start_builtin, string_raw_builtin, string_repeat_builtin,
    string_replace_all_builtin, string_replace_builtin, string_search_builtin,
    string_slice_builtin, string_small_builtin, string_split_builtin, string_starts_with_builtin,
    string_strike_builtin, string_sub_builtin, string_substr_builtin, string_substring_builtin,
    string_sup_builtin, string_to_locale_lower_case_builtin, string_to_locale_upper_case_builtin,
    string_to_lower_case_builtin, string_to_string_builtin, string_to_upper_case_builtin,
    string_to_well_formed_builtin, string_trim_builtin, string_trim_end_builtin,
    string_trim_start_builtin, string_value_of_builtin, symbol_builtin,
    symbol_description_getter_builtin, symbol_for_builtin, symbol_key_for_builtin,
    symbol_to_primitive_builtin, symbol_to_string_builtin, symbol_value_of_builtin,
    syntax_error_builtin, type_error_builtin, typed_array_at_builtin, typed_array_builtin,
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
    uint8_array_entries_builtin, uint8_array_from_base64_builtin, uint8_array_from_hex_builtin,
    uint8_array_keys_builtin, uint8_array_length_getter_builtin, uint8_array_set_builtin,
    uint8_array_set_from_base64_builtin, uint8_array_set_from_hex_builtin,
    uint8_array_slice_builtin, uint8_array_subarray_builtin, uint8_array_to_base64_builtin,
    uint8_array_to_hex_builtin, uint8_array_values_builtin, uint8_clamped_array_builtin,
    unescape_builtin, uri_error_builtin, weak_map_builtin, weak_map_delete_builtin,
    weak_map_get_builtin, weak_map_get_or_insert_builtin, weak_map_get_or_insert_computed_builtin,
    weak_map_has_builtin, weak_map_set_builtin, weak_ref_builtin, weak_ref_deref_builtin,
    weak_set_add_builtin, weak_set_builtin, weak_set_delete_builtin, weak_set_has_builtin,
};

/// VM and host services required by public builtin dispatch.
///
/// Methods that return `Result` report failures through the implementing
/// context's error channel. That error type is intentionally owned by the
/// embedding VM so dispatch helpers can propagate both ECMAScript abrupt
/// completions and host/VM failures without coupling this crate to VM internals.
#[allow(clippy::missing_errors_doc)]
pub trait PublicBuiltinDispatchContext: InternalBuiltinDispatchContext {
    fn agent(&mut self) -> &mut Agent;

    fn take_string_code_units_scratch(&mut self) -> Vec<u16> {
        Vec::new()
    }

    fn recycle_string_code_units_scratch(&mut self, _units: Vec<u16>) {}

    fn callee_object(&self) -> lyng_js_types::ObjectRef;

    fn builtin_realm(&self) -> RealmRef;

    fn caller_realm(&self) -> RealmRef;

    fn caller_is_strict(&self) -> bool;

    fn abrupt(&mut self, completion: AbruptCompletion) -> Self::Error;

    fn extract_thrown_value(&mut self, error: Self::Error) -> Result<Option<Value>, Self::Error>;

    fn value_to_string_text(&mut self, value: Value) -> Result<String, Self::Error>;

    fn to_property_key(&mut self, value: Value) -> Result<PropertyKey, Self::Error>;

    fn get_property_value(
        &mut self,
        receiver: Value,
        key: PropertyKey,
    ) -> Result<Value, Self::Error>;

    fn get_property_from_object_with_receiver(
        &mut self,
        object: lyng_js_types::ObjectRef,
        key: PropertyKey,
        receiver: Value,
    ) -> Result<Value, Self::Error>;

    fn get_own_property_from_object(
        &mut self,
        object: lyng_js_types::ObjectRef,
        key: PropertyKey,
    ) -> Result<Option<PropertyDescriptor>, Self::Error>;

    fn set_property_on_object_with_receiver(
        &mut self,
        object: lyng_js_types::ObjectRef,
        key: PropertyKey,
        value: Value,
        receiver: Value,
    ) -> Result<bool, Self::Error>;

    fn define_property_on_object(
        &mut self,
        object: lyng_js_types::ObjectRef,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
    ) -> Result<bool, Self::Error>;

    fn try_fast_create_data_property(
        &mut self,
        _object: lyng_js_types::ObjectRef,
        _index: u32,
        _value: Value,
    ) -> Result<bool, Self::Error> {
        Ok(false)
    }

    fn try_fast_has_own_index_property(
        &mut self,
        _object: lyng_js_types::ObjectRef,
        _index: u32,
    ) -> Result<Option<bool>, Self::Error> {
        Ok(None)
    }

    fn delete_property_from_object(
        &mut self,
        object: lyng_js_types::ObjectRef,
        key: PropertyKey,
    ) -> Result<bool, Self::Error>;

    fn prepare_own_property_keys_from_object(
        &mut self,
        _object: lyng_js_types::ObjectRef,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn prepare_has_property_from_object(
        &mut self,
        _object: lyng_js_types::ObjectRef,
        _key: PropertyKey,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn to_object_for_builtin_value(
        &mut self,
        realm: RealmRef,
        value: Value,
    ) -> Result<lyng_js_types::ObjectRef, Self::Error>;

    fn allocate_ordinary_object_with_prototype(
        &mut self,
        realm: RealmRef,
        prototype: Option<lyng_js_types::ObjectRef>,
    ) -> Result<lyng_js_types::ObjectRef, Self::Error>;

    fn allocate_builtin_function(
        &mut self,
        entry: BuiltinFunctionId,
    ) -> Result<lyng_js_types::ObjectRef, Self::Error>;

    fn create_array_object(
        &mut self,
        realm: RealmRef,
        element_capacity: usize,
    ) -> Result<lyng_js_types::ObjectRef, Self::Error>;

    fn ordinary_constructor_prototype(
        &mut self,
        realm: RealmRef,
        new_target: Option<lyng_js_types::ObjectRef>,
        default_prototype: lyng_js_types::ObjectRef,
    ) -> Result<lyng_js_types::ObjectRef, Self::Error>;

    fn descriptor_object_from_descriptor(
        &mut self,
        realm: RealmRef,
        descriptor: PropertyDescriptor,
    ) -> Result<Value, Self::Error>;

    fn to_property_descriptor(
        &mut self,
        descriptor_object: lyng_js_types::ObjectRef,
    ) -> Result<PropertyDescriptor, Self::Error>;

    fn set_integrity_level(
        &mut self,
        object: lyng_js_types::ObjectRef,
        freeze: bool,
    ) -> Result<bool, Self::Error>;

    fn test_integrity_level(
        &mut self,
        object: lyng_js_types::ObjectRef,
        frozen: bool,
    ) -> Result<bool, Self::Error>;

    fn park_agent(
        &mut self,
        request: &lyng_js_host::ParkAgentRequest,
    ) -> Result<lyng_js_host::ParkAgentResult, Self::Error>;

    fn unpark_agent(
        &mut self,
        request: &lyng_js_host::UnparkAgentRequest,
    ) -> Result<lyng_js_host::UnparkAgentResult, Self::Error>;

    fn temporal_current_instant(
        &mut self,
        request: &TemporalCurrentInstantRequest,
    ) -> Result<TemporalInstant, Self::Error>;

    fn temporal_default_time_zone(
        &mut self,
        request: &TemporalDefaultTimeZoneRequest,
    ) -> Result<TemporalDefaultTimeZone, Self::Error>;

    fn temporal_default_time_zone_is_utc(
        &mut self,
        request: &TemporalDefaultTimeZoneRequest,
    ) -> Result<bool, Self::Error>;

    fn temporal_instant_to_civil_time(
        &mut self,
        request: &TemporalInstantToCivilRequest,
    ) -> Result<TemporalCivilTime, Self::Error>;

    fn temporal_civil_time_to_instant(
        &mut self,
        request: &TemporalCivilToInstantRequest,
    ) -> Result<TemporalInstantWithOffset, Self::Error>;

    fn require_callable_object(
        &mut self,
        value: Value,
    ) -> Result<lyng_js_types::ObjectRef, Self::Error>;

    fn call_to_completion(
        &mut self,
        callee_object: lyng_js_types::ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> Result<Value, Self::Error>;

    fn try_fast_apply_builtin(
        &mut self,
        _target: lyng_js_types::ObjectRef,
        _this_value: Value,
        _arguments: Value,
    ) -> Result<Option<Value>, Self::Error> {
        Ok(None)
    }

    fn construct_to_completion(
        &mut self,
        callee_object: lyng_js_types::ObjectRef,
        arguments: &[Value],
        new_target: Option<lyng_js_types::ObjectRef>,
    ) -> Result<lyng_js_types::ObjectRef, Self::Error>;

    fn collect_array_like_arguments(
        &mut self,
        realm: RealmRef,
        value: Value,
    ) -> Result<Vec<Value>, Self::Error>;

    fn create_bound_function(
        &mut self,
        target: lyng_js_types::ObjectRef,
        bound_this: Value,
        bound_arguments: &[Value],
    ) -> Result<lyng_js_types::ObjectRef, Self::Error>;

    fn create_dynamic_function(
        &mut self,
        realm: RealmRef,
        parameters_source: &str,
        body_source: &str,
        strict_caller: bool,
        kind: DynamicFunctionKind,
        new_target: Option<lyng_js_types::ObjectRef>,
    ) -> Result<lyng_js_types::ObjectRef, Self::Error>;

    fn generator_next(
        &mut self,
        generator: lyng_js_types::ObjectRef,
        value: Value,
    ) -> Result<Value, Self::Error>;

    fn generator_return(
        &mut self,
        generator: lyng_js_types::ObjectRef,
        value: Value,
    ) -> Result<Value, Self::Error>;

    fn generator_throw(
        &mut self,
        generator: lyng_js_types::ObjectRef,
        value: Value,
    ) -> Result<Value, Self::Error>;

    fn async_generator_next(
        &mut self,
        this_value: Value,
        value: Value,
    ) -> Result<Value, Self::Error>;

    fn async_generator_return(
        &mut self,
        this_value: Value,
        value: Value,
    ) -> Result<Value, Self::Error>;

    fn async_generator_throw(
        &mut self,
        this_value: Value,
        value: Value,
    ) -> Result<Value, Self::Error>;

    fn evaluate_script_in_realm(
        &mut self,
        realm: RealmRef,
        source_text: &str,
    ) -> Result<Value, Self::Error>;

    fn function_to_string_text(
        &mut self,
        function: lyng_js_types::ObjectRef,
    ) -> Result<String, Self::Error>;
}

/// Delegates spec-like reserved internal helper IDs to their public builtin semantics.
///
/// The reserved IDs remain useful as compatibility function identities for older
/// lowering/bootstrap paths, but the semantic algorithms are owned by the
/// public builtin family modules.
pub(crate) fn dispatch_internal_spec_like_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == internal_string_replace_builtin() {
        return strings::string_replace_builtin(context, invocation).map(Some);
    }
    if entry == internal_string_index_of_builtin() {
        return strings::string_index_of_builtin(context, invocation).map(Some);
    }
    if entry == internal_array_index_of_builtin() {
        return arrays::array_index_of_builtin(context, invocation).map(Some);
    }
    if entry == internal_array_push_builtin() {
        return arrays::array_push_builtin(context, invocation).map(Some);
    }
    if entry == internal_array_pop_builtin() {
        return arrays::array_pop_builtin(context, invocation).map(Some);
    }
    if entry == internal_object_to_string_builtin() {
        return objects::object_to_string_builtin(context, invocation).map(Some);
    }
    if entry == internal_object_has_own_property_builtin() {
        return objects::object_has_own_property_builtin(context, invocation).map(Some);
    }
    if entry == internal_regexp_literal_builtin() {
        return regexp::regexp_literal_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

/// Dispatches a public builtin by builtin ID.
///
/// Returns `Ok(None)` when `entry` is not owned by the public builtin dispatcher.
///
/// # Errors
///
/// Propagates errors reported by the active dispatch context, including abrupt
/// ECMAScript completions and host or VM failures surfaced through
/// [`PublicBuiltinDispatchContext::Error`].
pub fn dispatch_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if is_date_builtin(entry) {
        return date::dispatch_date_builtin(context, entry, invocation);
    }
    if let Some(result) = dispatch_internal_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = error_objects::dispatch_error_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = disposal::dispatch_disposal_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = language::dispatch_language_support_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = objects::dispatch_object_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = functions::dispatch_function_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = collections::dispatch_collection_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = arrays::dispatch_array_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = binary_data::dispatch_binary_data_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = json::dispatch_json_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) =
        object_reflection::dispatch_object_reflection_builtin(context, entry, invocation)?
    {
        return Ok(Some(result));
    }
    if let Some(result) = promises::dispatch_promise_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = iterators::dispatch_iterator_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = strings::dispatch_string_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = regexp::dispatch_regexp_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = temporal::dispatch_temporal_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = primitives::dispatch_primitive_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    Ok(None)
}

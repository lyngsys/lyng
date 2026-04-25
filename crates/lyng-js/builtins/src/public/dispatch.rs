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
    AbruptCompletion, BuiltinFunctionId, PropertyDescriptor, PropertyKey, RealmRef, Value,
};
use support::{
    allocate_array_like_result, allocate_json_raw_object, allocate_proxy_object,
    argument_to_number, array_like_index_property_key, array_like_join_text, array_like_length,
    array_like_length_u64, array_result_capacity_hint, array_species_create_for_length,
    callable_object_from_value, close_iterator_after_error, code_unit_range_value,
    collect_array_like_values_for_from_builtin, create_array_from_values, create_array_result,
    create_array_result_for_length, create_array_result_with_prototype,
    create_data_property_or_throw, define_array_length, define_data_property_with_attrs,
    delete_property_from_object, format_to_exponential, format_to_precision,
    get_property_from_object, get_property_from_object_with_receiver, has_property_on_object,
    is_array_for_species, is_concat_spreadable, is_engine_array, is_integral_number,
    iterable_to_values_list, length_value, length_value_u64, map_completion,
    normalize_relative_index_u64, number_value, primitive_wrapper_constructor,
    property_key_from_text, property_key_string_value, property_key_value, proxy_define_property,
    proxy_delete_property, proxy_get_own_property, proxy_get_prototype_of, proxy_is_extensible,
    proxy_own_property_keys, proxy_prevent_extensions, proxy_set_prototype_of, radix_argument,
    range_error, reference_error, require_constructor_object, require_object_argument,
    require_proxy_argument_object, set_data_property_value, set_length_property,
    set_property_on_object, set_property_on_object_with_receiver, string_from_code_units,
    string_ref_code_units, string_ref_text, string_this_ref, string_value,
    symbol_descriptive_string, syntax_error, to_bigint_for_builtin, to_boolean_for_builtin,
    to_index_for_builtin, to_integer_or_infinity_for_builtin, to_length_for_builtin,
    to_number_for_builtin, to_number_value_for_builtin, to_string_string_ref,
    to_uint32_for_builtin, to_uint8_clamp_for_builtin, to_uint8_for_builtin,
    try_create_data_property, try_delete_property_from_object, type_error, uri_error,
    usize_index_value, valid_array_length, BuiltinIteratorBridge, BuiltinProxyBridge,
    BuiltinToPrimitiveBridge, MAX_SAFE_INTEGER_U64,
};

use super::{
    js3_abstract_module_source_builtin, js3_abstract_module_source_to_string_tag_getter_builtin,
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
    js3_array_values_builtin, js3_array_with_builtin, js3_async_function_builtin,
    js3_async_generator_function_builtin, js3_async_generator_next_builtin,
    js3_async_generator_return_builtin, js3_async_generator_throw_builtin, js3_atomics_add_builtin,
    js3_atomics_and_builtin, js3_atomics_compare_exchange_builtin, js3_atomics_exchange_builtin,
    js3_atomics_is_lock_free_builtin, js3_atomics_load_builtin, js3_atomics_notify_builtin,
    js3_atomics_or_builtin, js3_atomics_store_builtin, js3_atomics_sub_builtin,
    js3_atomics_wait_async_builtin, js3_atomics_wait_builtin, js3_atomics_xor_builtin,
    js3_big_int64_array_builtin, js3_big_uint64_array_builtin, js3_bigint_as_int_n_builtin,
    js3_bigint_as_uint_n_builtin, js3_bigint_builtin, js3_bigint_to_string_builtin,
    js3_bigint_value_of_builtin, js3_boolean_builtin, js3_boolean_to_string_builtin,
    js3_boolean_value_of_builtin, js3_data_view_buffer_getter_builtin, js3_data_view_builtin,
    js3_data_view_byte_length_getter_builtin, js3_data_view_byte_offset_getter_builtin,
    js3_data_view_get_float32_builtin, js3_data_view_get_float64_builtin,
    js3_data_view_get_int16_builtin, js3_data_view_get_int32_builtin,
    js3_data_view_get_int8_builtin, js3_data_view_get_uint16_builtin,
    js3_data_view_get_uint32_builtin, js3_data_view_get_uint8_builtin,
    js3_data_view_set_float32_builtin, js3_data_view_set_float64_builtin,
    js3_data_view_set_int16_builtin, js3_data_view_set_int32_builtin,
    js3_data_view_set_int8_builtin, js3_data_view_set_uint16_builtin,
    js3_data_view_set_uint32_builtin, js3_data_view_set_uint8_builtin, js3_date_builtin,
    js3_date_get_date_builtin, js3_date_get_day_builtin, js3_date_get_full_year_builtin,
    js3_date_get_hours_builtin, js3_date_get_milliseconds_builtin, js3_date_get_minutes_builtin,
    js3_date_get_month_builtin, js3_date_get_seconds_builtin, js3_date_get_time_builtin,
    js3_date_get_timezone_offset_builtin, js3_date_get_utc_date_builtin,
    js3_date_get_utc_day_builtin, js3_date_get_utc_full_year_builtin,
    js3_date_get_utc_hours_builtin, js3_date_get_utc_milliseconds_builtin,
    js3_date_get_utc_minutes_builtin, js3_date_get_utc_month_builtin,
    js3_date_get_utc_seconds_builtin, js3_date_now_builtin, js3_date_parse_builtin,
    js3_date_set_date_builtin, js3_date_set_full_year_builtin, js3_date_set_hours_builtin,
    js3_date_set_milliseconds_builtin, js3_date_set_minutes_builtin, js3_date_set_month_builtin,
    js3_date_set_seconds_builtin, js3_date_set_time_builtin, js3_date_set_utc_date_builtin,
    js3_date_set_utc_full_year_builtin, js3_date_set_utc_hours_builtin,
    js3_date_set_utc_milliseconds_builtin, js3_date_set_utc_minutes_builtin,
    js3_date_set_utc_month_builtin, js3_date_set_utc_seconds_builtin,
    js3_date_to_date_string_builtin, js3_date_to_iso_string_builtin, js3_date_to_json_builtin,
    js3_date_to_locale_date_string_builtin, js3_date_to_locale_string_builtin,
    js3_date_to_locale_time_string_builtin, js3_date_to_primitive_builtin,
    js3_date_to_string_builtin, js3_date_to_temporal_instant_builtin,
    js3_date_to_time_string_builtin, js3_date_to_utc_string_builtin, js3_date_utc_builtin,
    js3_date_value_of_builtin, js3_decode_uri_builtin, js3_decode_uri_component_builtin,
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
    js3_promise_then_builtin, js3_range_error_builtin, js3_reference_error_builtin,
    js3_regexp_builtin, js3_regexp_dot_all_getter_builtin, js3_regexp_escape_builtin,
    js3_regexp_exec_builtin, js3_regexp_flags_getter_builtin, js3_regexp_global_getter_builtin,
    js3_regexp_has_indices_getter_builtin, js3_regexp_ignore_case_getter_builtin,
    js3_regexp_multiline_getter_builtin, js3_regexp_source_getter_builtin,
    js3_regexp_species_getter_builtin, js3_regexp_sticky_getter_builtin,
    js3_regexp_symbol_match_all_builtin, js3_regexp_symbol_match_builtin,
    js3_regexp_symbol_replace_builtin, js3_regexp_symbol_search_builtin,
    js3_regexp_symbol_split_builtin, js3_regexp_test_builtin, js3_regexp_to_string_builtin,
    js3_regexp_unicode_getter_builtin, js3_set_add_builtin, js3_set_builtin, js3_set_clear_builtin,
    js3_set_delete_builtin, js3_set_entries_builtin, js3_set_for_each_builtin, js3_set_has_builtin,
    js3_set_iterator_next_builtin, js3_set_keys_builtin, js3_set_size_getter_builtin,
    js3_set_values_builtin, js3_shared_array_buffer_builtin,
    js3_shared_array_buffer_byte_length_getter_builtin, js3_shared_array_buffer_slice_builtin,
    js3_string_at_builtin, js3_string_builtin, js3_string_char_at_builtin,
    js3_string_char_code_at_builtin, js3_string_code_point_at_builtin, js3_string_concat_builtin,
    js3_string_ends_with_builtin, js3_string_from_char_code_builtin,
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
    js3_symbol_builtin, js3_symbol_description_getter_builtin, js3_symbol_for_builtin,
    js3_symbol_key_for_builtin, js3_symbol_to_primitive_builtin, js3_symbol_to_string_builtin,
    js3_symbol_value_of_builtin, js3_syntax_error_builtin, js3_type_error_builtin,
    js3_typed_array_at_builtin, js3_typed_array_builtin, js3_typed_array_copy_within_builtin,
    js3_typed_array_every_builtin, js3_typed_array_fill_builtin, js3_typed_array_filter_builtin,
    js3_typed_array_find_builtin, js3_typed_array_find_index_builtin,
    js3_typed_array_find_last_builtin, js3_typed_array_find_last_index_builtin,
    js3_typed_array_for_each_builtin, js3_typed_array_from_builtin,
    js3_typed_array_includes_builtin, js3_typed_array_index_of_builtin,
    js3_typed_array_join_builtin, js3_typed_array_last_index_of_builtin,
    js3_typed_array_map_builtin, js3_typed_array_of_builtin, js3_typed_array_reduce_builtin,
    js3_typed_array_reduce_right_builtin, js3_typed_array_reverse_builtin,
    js3_typed_array_some_builtin, js3_typed_array_sort_builtin,
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
    js3_weak_set_delete_builtin, js3_weak_set_has_builtin,
};

pub trait PublicBuiltinDispatchContext: InternalBuiltinDispatchContext {
    fn agent(&mut self) -> &mut Agent;

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

    fn delete_property_from_object(
        &mut self,
        object: lyng_js_types::ObjectRef,
        key: PropertyKey,
    ) -> Result<bool, Self::Error>;

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

pub fn dispatch_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
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
    if let Some(result) = date::dispatch_date_builtin(context, entry, invocation)? {
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

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
mod temporal;
use crate::internal::{dispatch_internal_builtin, InternalBuiltinDispatchContext};
use crate::{BuiltinInvocation, DynamicFunctionKind};
use lyng_js_common::WellKnownAtom;
use lyng_js_env::{Agent, AsyncWaiterRecord, ParkedAgentRecord, WaiterKind};
use lyng_js_gc::{AllocationLifetime, BigIntSign, StringEncoding};
use lyng_js_host::{
    ParkAgentRequest, ParkAgentStatus, TemporalCivilTime, TemporalCivilToInstantRequest,
    TemporalCurrentInstantRequest, TemporalDefaultTimeZone, TemporalDefaultTimeZoneRequest,
    TemporalInstant, TemporalInstantToCivilRequest, TemporalInstantWithOffset, UnparkAgentRequest,
};
use lyng_js_objects::{
    ArrayBufferObjectData, DataViewObjectData, FunctionEntryIdentity, ObjectAllocation,
    ObjectColdData, ObjectFlags, ObjectKind, OrdinaryObjectData, PrimitiveWrapperKind,
    ProxyObjectData, TypedArrayElementKind, TypedArrayObjectData,
};
use lyng_js_ops::{
    errors, iterator, object, promise, proxy, read, shared_memory as shared_memory_ops,
};
use lyng_js_parser::validate_regexp_literal;
use lyng_js_types::{
    AbruptCompletion, BuiltinFunctionId, ObjectRef, PropertyDescriptor, PropertyKey, RealmRef,
    StringRef, Value, WellKnownSymbolId,
};
use std::fmt::Write as _;

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

struct BuiltinToPrimitiveBridge<'a, Cx: PublicBuiltinDispatchContext> {
    cx: &'a mut Cx,
}

impl<Cx: PublicBuiltinDispatchContext> object::ToPrimitiveContext
    for BuiltinToPrimitiveBridge<'_, Cx>
{
    type Error = Cx::Error;

    fn agent(&mut self) -> &mut Agent {
        self.cx.agent()
    }

    fn abrupt(&mut self, completion: AbruptCompletion) -> Self::Error {
        self.cx.abrupt(completion)
    }

    fn type_error(&mut self) -> Self::Error {
        type_error(self.cx)
    }

    fn get_property_value(
        &mut self,
        object: lyng_js_types::ObjectRef,
        key: PropertyKey,
    ) -> Result<Value, Self::Error> {
        self.cx
            .get_property_value(Value::from_object_ref(object), key)
    }

    fn require_callable_object(
        &mut self,
        value: Value,
    ) -> Result<lyng_js_types::ObjectRef, Self::Error> {
        self.cx.require_callable_object(value)
    }

    fn call_to_completion(
        &mut self,
        callee_object: lyng_js_types::ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> Result<Value, Self::Error> {
        self.cx
            .call_to_completion(callee_object, this_value, arguments)
    }

    fn default_to_primitive_result(
        &mut self,
        object: lyng_js_types::ObjectRef,
        method_name: lyng_js_common::AtomId,
        method_object: lyng_js_types::ObjectRef,
    ) -> Result<Option<Value>, Self::Error> {
        if method_name != WellKnownAtom::toString.id()
            || builtin_function_entry(self.cx.agent(), method_object)
                != Some(js3_object_to_string_builtin())
            || !is_engine_array(self.cx, object)
        {
            return Ok(None);
        }

        let text = array_like_to_string_fallback(self.cx, object)?;
        let value = {
            let agent = self.cx.agent();
            Value::from_string_ref(agent.alloc_runtime_string(
                &text,
                None,
                AllocationLifetime::Default,
            ))
        };
        Ok(Some(value))
    }
}

struct BuiltinIteratorBridge<'a, Cx: PublicBuiltinDispatchContext> {
    cx: &'a mut Cx,
}

impl<Cx: PublicBuiltinDispatchContext> iterator::IteratorOpsContext
    for BuiltinIteratorBridge<'_, Cx>
{
    type Error = Cx::Error;

    fn agent(&mut self) -> &mut Agent {
        self.cx.agent()
    }

    fn realm(&self) -> RealmRef {
        self.cx.caller_realm()
    }

    fn abrupt(&mut self, completion: AbruptCompletion) -> Self::Error {
        self.cx.abrupt(completion)
    }

    fn type_error(&mut self) -> Self::Error {
        type_error(self.cx)
    }

    fn get_property_value(
        &mut self,
        receiver: Value,
        key: PropertyKey,
    ) -> Result<Value, Self::Error> {
        self.cx.get_property_value(receiver, key)
    }

    fn require_callable_object(&mut self, value: Value) -> Result<ObjectRef, Self::Error> {
        self.cx.require_callable_object(value)
    }

    fn call_to_completion(
        &mut self,
        callee_object: ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> Result<Value, Self::Error> {
        self.cx
            .call_to_completion(callee_object, this_value, arguments)
    }
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

fn type_error<Cx: PublicBuiltinDispatchContext>(cx: &mut Cx) -> Cx::Error {
    intrinsic_error(cx, errors::ErrorKind::Type)
}

fn range_error<Cx: PublicBuiltinDispatchContext>(cx: &mut Cx) -> Cx::Error {
    intrinsic_error(cx, errors::ErrorKind::Range)
}

fn reference_error<Cx: PublicBuiltinDispatchContext>(cx: &mut Cx) -> Cx::Error {
    intrinsic_error(cx, errors::ErrorKind::Reference)
}

fn syntax_error<Cx: PublicBuiltinDispatchContext>(cx: &mut Cx) -> Cx::Error {
    intrinsic_error(cx, errors::ErrorKind::Syntax)
}

fn uri_error<Cx: PublicBuiltinDispatchContext>(cx: &mut Cx) -> Cx::Error {
    intrinsic_error(cx, errors::ErrorKind::Uri)
}

fn intrinsic_error<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: errors::ErrorKind,
) -> Cx::Error {
    let realm = cx.builtin_realm();
    let completion = {
        let agent = cx.agent();
        errors::create_intrinsic_error_object(agent, realm, kind, None)
            .map(Value::from_object_ref)
            .map(AbruptCompletion::throw)
            .unwrap_or_else(|completion| completion)
    };
    cx.abrupt(completion)
}

fn map_completion<Cx: PublicBuiltinDispatchContext, T>(
    cx: &mut Cx,
    result: lyng_js_types::Completion<T>,
) -> Result<T, Cx::Error> {
    result.map_err(|completion| cx.abrupt(completion))
}

fn string_value<Cx: PublicBuiltinDispatchContext>(cx: &mut Cx, text: &str) -> Value {
    let string = {
        let agent = cx.agent();
        agent.alloc_runtime_string(text, None, AllocationLifetime::Default)
    };
    Value::from_string_ref(string)
}

fn allocate_json_raw_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    raw_text: StringRef,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let root_shape = {
        let agent = cx.agent();
        agent.realm(realm).and_then(|record| record.root_shape())
    }
    .ok_or_else(|| type_error(cx))?;
    let object = cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_prototype(None)
                .with_ordinary_payload_value(Value::from_string_ref(raw_text))
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::JsonRaw)),
            AllocationLifetime::Default,
        )
    });
    let key = property_key_from_text(cx, "rawJSON");
    create_data_property_or_throw(cx, object, key, Value::from_string_ref(raw_text))?;
    Ok(object)
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
        agent.realm(realm).and_then(|record| record.root_shape())
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

fn allocate_proxy_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    target: ObjectRef,
    handler: ObjectRef,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let root_shape = {
        let agent = cx.agent();
        agent.realm(realm).and_then(|record| record.root_shape())
    }
    .ok_or_else(|| type_error(cx))?;
    let prototype = {
        let agent = cx.agent();
        agent
            .objects()
            .object_header(agent.heap().view(), target)
            .and_then(|header| header.prototype())
    };
    let (callable, constructible) = {
        let objects = cx.agent().objects();
        (objects.is_callable(target), objects.is_constructor(target))
    };
    Ok(cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::proxy(
                root_shape,
                ProxyObjectData::new(target, handler, callable, constructible),
            )
            .with_prototype(prototype),
            AllocationLifetime::Default,
        )
    }))
}

fn require_object_argument<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    index: usize,
) -> Result<ObjectRef, Cx::Error> {
    invocation
        .arguments()
        .get(index)
        .copied()
        .and_then(Value::as_object_ref)
        .ok_or_else(|| type_error(cx))
}

fn require_proxy_argument_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    index: usize,
) -> Result<ObjectRef, Cx::Error> {
    require_object_argument(cx, invocation, index)
}

fn allocate_data_view_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
    data_view: DataViewObjectData,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let root_shape = {
        let agent = cx.agent();
        agent.realm(realm).and_then(|record| record.root_shape())
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
            i32::try_from(value)
                .map(Value::from_smi)
                .unwrap_or_else(|_| Value::from_f64(f64::from(value)))
        }
        TypedArrayElementKind::Uint16 => Value::from_smi(i32::from(bits as u16)),
        TypedArrayElementKind::Uint8Clamped => Value::from_smi(i32::from(bits as u8)),
        TypedArrayElementKind::Uint8 => Value::from_smi(i32::from(bits as u8)),
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

fn to_bigint_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Value, Cx::Error> {
    let primitive = {
        let mut bridge = BuiltinToPrimitiveBridge { cx };
        object::to_primitive(&mut bridge, value, object::ToPrimitiveHint::Number)?
    };
    if primitive.is_number() {
        return Err(type_error(cx));
    }
    let bigint = {
        let agent = cx.agent();
        object::primitive_to_bigint(agent, primitive)
    };
    map_completion(cx, bigint)
}

fn typed_array_storage_bits_from_builtin_value<Cx: PublicBuiltinDispatchContext>(
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

fn typed_array_write_storage_bits<Cx: PublicBuiltinDispatchContext>(
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
        agent.realm(realm).and_then(|record| record.root_shape())
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

fn define_data_property_with_attrs<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    key: PropertyKey,
    value: Value,
    writable: bool,
    enumerable: bool,
    configurable: bool,
) -> Result<(), Cx::Error> {
    let mut descriptor = PropertyDescriptor::new();
    descriptor.set_value(value);
    descriptor.set_writable(writable);
    descriptor.set_enumerable(enumerable);
    descriptor.set_configurable(configurable);
    let defined =
        { proxy_define_property(cx, object_ref, key, descriptor, AllocationLifetime::Default) };
    if !defined? {
        return Err(type_error(cx));
    }
    Ok(())
}

fn set_data_property_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    key: PropertyKey,
    value: Value,
) -> Result<(), Cx::Error> {
    let updated = {
        let agent = cx.agent();
        object::set(agent, object_ref, key, value, AllocationLifetime::Default)
    };
    if !map_completion(cx, updated)? {
        return Err(type_error(cx));
    }
    Ok(())
}

fn is_regexp_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
) -> Result<bool, Cx::Error> {
    Ok(cx.agent().objects().is_regexp_object(object_ref))
}

fn well_known_symbol_key<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    symbol: WellKnownSymbolId,
) -> Result<PropertyKey, Cx::Error> {
    let symbol = {
        let agent = cx.agent();
        agent.well_known_symbol(symbol)
    }
    .ok_or_else(|| type_error(cx))?;
    Ok(PropertyKey::from_symbol(symbol))
}

fn get_method_for_well_known_symbol<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    symbol: WellKnownSymbolId,
) -> Result<Option<ObjectRef>, Cx::Error> {
    let key = well_known_symbol_key(cx, symbol)?;
    let method = cx.get_property_value(value, key)?;
    if method.is_undefined() || method.is_null() {
        return Ok(None);
    }
    cx.require_callable_object(method).map(Some)
}

fn is_regexp_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<bool, Cx::Error> {
    let Some(object_ref) = value.as_object_ref() else {
        return Ok(false);
    };
    let key = well_known_symbol_key(cx, WellKnownSymbolId::Match)?;
    let matcher = cx.get_property_value(value, key)?;
    if !matcher.is_undefined() {
        return to_boolean_for_builtin(cx, matcher);
    }
    is_regexp_object(cx, object_ref)
}

fn current_intrinsic_regexp_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Option<lyng_js_types::ObjectRef> {
    let realm = cx.builtin_realm();
    let agent = cx.agent();
    agent
        .realm(realm)
        .and_then(|record| record.intrinsics().regexp_prototype())
}

fn regexp_matcher_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if !is_regexp_object(cx, object_ref)? {
        return Err(type_error(cx));
    }
    Ok(object_ref)
}

fn regexp_last_index_key<Cx: PublicBuiltinDispatchContext>(cx: &mut Cx) -> PropertyKey {
    let last_index = {
        let agent = cx.agent();
        agent.bootstrap_atoms().last_index()
    };
    PropertyKey::from_atom(last_index)
}

fn boolean_property_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    receiver: Value,
    key: PropertyKey,
) -> Result<bool, Cx::Error> {
    let value = cx.get_property_value(receiver, key)?;
    to_boolean_for_builtin(cx, value)
}

fn length_value(length: u32) -> Value {
    i32::try_from(length)
        .map(Value::from_smi)
        .unwrap_or_else(|_| Value::from_f64(f64::from(length)))
}

fn length_value_u64(length: u64) -> Value {
    u32::try_from(length)
        .map(length_value)
        .unwrap_or_else(|_| Value::from_f64(length as f64))
}

fn is_engine_array<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: lyng_js_types::ObjectRef,
) -> bool {
    let agent = cx.agent();
    agent
        .objects()
        .object_header(agent.heap().view(), object)
        .is_some_and(|header| header.flags().is_engine_array())
}

fn is_array_for_species<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: lyng_js_types::ObjectRef,
) -> Result<bool, Cx::Error> {
    if is_engine_array(cx, object) {
        return Ok(true);
    }
    let proxy_target = {
        let agent = cx.agent();
        agent.objects().proxy_data(object).map(|proxy| {
            if proxy.revoked() {
                None
            } else {
                Some(proxy.target())
            }
        })
    };
    match proxy_target {
        Some(Some(target)) => is_array_for_species(cx, target),
        Some(None) => Err(type_error(cx)),
        None => Ok(false),
    }
}

fn is_any_realm_array_constructor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: lyng_js_types::ObjectRef,
) -> bool {
    let agent = cx.agent();
    agent.realm_refs().iter().copied().any(|realm| {
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().array())
            == Some(object)
    })
}

fn builtin_function_entry(
    agent: &Agent,
    object: lyng_js_types::ObjectRef,
) -> Option<BuiltinFunctionId> {
    let data = agent.objects().function_data(object)?;
    let FunctionEntryIdentity::Native(entry) = data.entry()? else {
        return None;
    };
    entry.builtin_entry()
}

fn array_like_join_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: lyng_js_types::ObjectRef,
    separator: &str,
) -> Result<String, Cx::Error> {
    let length = array_like_length(cx, object)?;
    let mut text = String::new();
    for index in 0..length {
        if index != 0 {
            text.push_str(separator);
        }
        let element =
            cx.get_property_value(Value::from_object_ref(object), PropertyKey::Index(index))?;
        if element.is_undefined() || element.is_null() {
            continue;
        }
        text.push_str(&cx.value_to_string_text(element)?);
    }
    Ok(text)
}

fn array_like_to_string_fallback<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: lyng_js_types::ObjectRef,
) -> Result<String, Cx::Error> {
    array_like_join_text(cx, object, ",")
}

fn to_number_value_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Value, Cx::Error> {
    let primitive = {
        let mut bridge = BuiltinToPrimitiveBridge { cx };
        object::to_primitive(&mut bridge, value, object::ToPrimitiveHint::Number)?
    };
    let number = {
        let agent = cx.agent();
        read::to_number(agent.heap().view(), primitive)
    };
    match number {
        Ok(number) => Ok(number),
        Err(_) => Err(type_error(cx)),
    }
}

fn to_number_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<f64, Cx::Error> {
    let numeric = to_number_value_for_builtin(cx, value)?;
    if let Some(value) = numeric.as_smi() {
        return Ok(f64::from(value));
    }
    if let Some(value) = numeric.as_f64() {
        return Ok(value);
    }
    Err(type_error(cx))
}

fn valid_array_length(number: f64) -> Option<u32> {
    if !number.is_finite() || number < 0.0 || number.trunc() != number {
        return None;
    }
    if number > f64::from(u32::MAX) {
        return None;
    }
    Some(number as u32)
}

fn to_uint32_length(number: f64) -> u32 {
    if !number.is_finite() || number == 0.0 {
        return 0;
    }
    const TWO_32: f64 = 4_294_967_296.0;
    number.trunc().rem_euclid(TWO_32) as u32
}

fn normalize_engine_array_length_descriptor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    descriptor: PropertyDescriptor,
) -> Result<PropertyDescriptor, Cx::Error> {
    if !descriptor.has_value() {
        return Ok(descriptor);
    }
    let value = descriptor.value().unwrap_or(Value::undefined());
    let _ = to_number_for_builtin(cx, value)?;
    let number_len = to_number_for_builtin(cx, value)?;
    let new_len = to_uint32_length(number_len);
    if number_len != f64::from(new_len) {
        return Err(range_error(cx));
    }
    let mut normalized = descriptor;
    normalized.set_value(length_value(new_len));
    Ok(normalized)
}

const MAX_SAFE_INTEGER_U64: u64 = (1_u64 << 53) - 1;
const ARRAY_RESULT_CAPACITY_HINT_LIMIT: usize = 4096;

struct BuiltinProxyBridge<'a, Cx> {
    cx: &'a mut Cx,
}

impl<Cx: PublicBuiltinDispatchContext> proxy::ProxyTrapContext for BuiltinProxyBridge<'_, Cx> {
    type Error = Cx::Error;

    fn agent(&mut self) -> &mut Agent {
        self.cx.agent()
    }

    fn abrupt(&mut self, completion: AbruptCompletion) -> Self::Error {
        self.cx.abrupt(completion)
    }

    fn type_error(&mut self) -> Self::Error {
        type_error(self.cx)
    }

    fn get_property_value(
        &mut self,
        receiver: Value,
        key: PropertyKey,
    ) -> Result<Value, Self::Error> {
        self.cx.get_property_value(receiver, key)
    }

    fn get_property_from_object_with_receiver(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
        receiver: Value,
    ) -> Result<Value, Self::Error> {
        self.cx
            .get_property_from_object_with_receiver(object, key, receiver)
    }

    fn get_own_property_from_object(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> Result<Option<PropertyDescriptor>, Self::Error> {
        self.cx.get_own_property_from_object(object, key)
    }

    fn set_property_on_object_with_receiver(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
        value: Value,
        receiver: Value,
        _lifetime: AllocationLifetime,
    ) -> Result<bool, Self::Error> {
        self.cx
            .set_property_on_object_with_receiver(object, key, value, receiver)
    }

    fn define_property_on_object(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
        mut descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
    ) -> Result<bool, Self::Error> {
        if is_engine_array(self.cx, object)
            && key == PropertyKey::from_atom(WellKnownAtom::length.id())
        {
            descriptor = normalize_engine_array_length_descriptor(self.cx, descriptor)?;
        }
        self.cx
            .define_property_on_object(object, key, descriptor, lifetime)
    }

    fn delete_property_from_object(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> Result<bool, Self::Error> {
        self.cx.delete_property_from_object(object, key)
    }

    fn call_to_completion(
        &mut self,
        callee_object: ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> Result<Value, Self::Error> {
        self.cx
            .call_to_completion(callee_object, this_value, arguments)
    }

    fn construct_to_completion(
        &mut self,
        callee_object: ObjectRef,
        arguments: &[Value],
        new_target: Option<ObjectRef>,
    ) -> Result<ObjectRef, Self::Error> {
        self.cx
            .construct_to_completion(callee_object, arguments, new_target)
    }

    fn to_property_key(&mut self, value: Value) -> Result<PropertyKey, Self::Error> {
        self.cx.to_property_key(value)
    }

    fn to_property_descriptor(
        &mut self,
        descriptor_object: ObjectRef,
    ) -> Result<PropertyDescriptor, Self::Error> {
        self.cx.to_property_descriptor(descriptor_object)
    }

    fn descriptor_object_from_descriptor(
        &mut self,
        descriptor: PropertyDescriptor,
    ) -> Result<Value, Self::Error> {
        self.cx
            .descriptor_object_from_descriptor(self.cx.builtin_realm(), descriptor)
    }

    fn create_array_from_values(&mut self, values: &[Value]) -> Result<ObjectRef, Self::Error> {
        create_array_from_values(self.cx, values)
    }
}

fn proxy_get_prototype_of<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
) -> Result<Option<ObjectRef>, Cx::Error> {
    object::get_prototype_of_in_context(&mut BuiltinProxyBridge { cx }, object_ref)
}

fn proxy_set_prototype_of<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    prototype: Option<ObjectRef>,
) -> Result<bool, Cx::Error> {
    object::set_prototype_of_in_context(&mut BuiltinProxyBridge { cx }, object_ref, prototype)
}

fn proxy_get_own_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    key: PropertyKey,
) -> Result<Option<PropertyDescriptor>, Cx::Error> {
    object::get_own_property_in_context(&mut BuiltinProxyBridge { cx }, object_ref, key)
}

fn proxy_define_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    key: PropertyKey,
    descriptor: PropertyDescriptor,
    lifetime: AllocationLifetime,
) -> Result<bool, Cx::Error> {
    object::define_property_in_context(
        &mut BuiltinProxyBridge { cx },
        object_ref,
        key,
        descriptor,
        lifetime,
    )
}

fn proxy_has_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    key: PropertyKey,
) -> Result<bool, Cx::Error> {
    object::has_property_in_context(&mut BuiltinProxyBridge { cx }, object_ref, key)
}

fn proxy_own_property_keys<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
) -> Result<Vec<PropertyKey>, Cx::Error> {
    object::own_property_keys_in_context(&mut BuiltinProxyBridge { cx }, object_ref)
}

fn proxy_is_extensible<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
) -> Result<bool, Cx::Error> {
    proxy::is_extensible(&mut BuiltinProxyBridge { cx }, object_ref)
}

fn proxy_prevent_extensions<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
) -> Result<bool, Cx::Error> {
    proxy::prevent_extensions(&mut BuiltinProxyBridge { cx }, object_ref)
}

fn proxy_delete_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    key: PropertyKey,
) -> Result<bool, Cx::Error> {
    proxy::delete_property(&mut BuiltinProxyBridge { cx }, object_ref, key)
}

fn get_property_from_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    key: PropertyKey,
) -> Result<Value, Cx::Error> {
    cx.get_property_value(Value::from_object_ref(object_ref), key)
}

fn get_property_from_object_with_receiver<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    key: PropertyKey,
    receiver: Value,
) -> Result<Value, Cx::Error> {
    cx.get_property_from_object_with_receiver(object_ref, key, receiver)
}

fn property_key_from_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    text: &str,
) -> PropertyKey {
    let atom = {
        let agent = cx.agent();
        agent.atoms_mut().intern_collectible(text)
    };
    PropertyKey::from_atom(atom)
}

fn property_key_string_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    key: PropertyKey,
) -> Value {
    match key {
        PropertyKey::Index(index) => string_value(cx, &index.to_string()),
        PropertyKey::Atom(atom) => {
            let string = {
                let agent = cx.agent();
                agent.alloc_runtime_string("", Some(atom), AllocationLifetime::Default)
            };
            Value::from_string_ref(string)
        }
        PropertyKey::Symbol(_) => {
            unreachable!("symbol keys are filtered before list materialization")
        }
    }
}

fn property_key_value<Cx: PublicBuiltinDispatchContext>(cx: &mut Cx, key: PropertyKey) -> Value {
    match key {
        PropertyKey::Symbol(symbol) => Value::from_symbol_ref(symbol),
        _ => property_key_string_value(cx, key),
    }
}

fn has_property_on_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    key: PropertyKey,
) -> Result<bool, Cx::Error> {
    proxy_has_property(cx, object_ref, key)
}

fn set_property_on_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    key: PropertyKey,
    value: Value,
) -> Result<(), Cx::Error> {
    if let Some(index) = key.as_index() {
        let typed_array = cx.agent().objects().typed_array(object_ref);
        if let Some(record) = typed_array {
            let element_index = usize::try_from(index).unwrap_or(usize::MAX);
            if element_index >= record.length()
                || cx
                    .agent()
                    .backing_store_is_detached(record.backing_store())
                    .ok_or_else(|| type_error(cx))?
            {
                return Err(type_error(cx));
            }
            let bits = typed_array_storage_bits_from_builtin_value(cx, record.kind(), value)?;
            if cx
                .agent()
                .backing_store_is_detached(record.backing_store())
                .ok_or_else(|| type_error(cx))?
            {
                return Err(type_error(cx));
            }
            typed_array_write_storage_bits(cx, record, element_index, bits)?;
            return Ok(());
        }
    }
    if !set_property_on_object_with_receiver(
        cx,
        object_ref,
        key,
        value,
        Value::from_object_ref(object_ref),
    )? {
        return Err(type_error(cx));
    }
    Ok(())
}

fn set_property_on_object_with_receiver<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    key: PropertyKey,
    value: Value,
    receiver: Value,
) -> Result<bool, Cx::Error> {
    cx.set_property_on_object_with_receiver(object_ref, key, value, receiver)
}

fn delete_property_from_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    key: PropertyKey,
) -> Result<(), Cx::Error> {
    if !try_delete_property_from_object(cx, object_ref, key)? {
        return Err(type_error(cx));
    }
    Ok(())
}

fn try_delete_property_from_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    key: PropertyKey,
) -> Result<bool, Cx::Error> {
    proxy_delete_property(cx, object_ref, key)
}

fn define_array_length<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    length: u32,
) -> Result<(), Cx::Error> {
    let mut descriptor = PropertyDescriptor::new();
    descriptor.set_value(length_value(length));
    let defined = {
        let agent = cx.agent();
        object::define_property(
            agent,
            object_ref,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
            descriptor,
            AllocationLifetime::Default,
        )
    };
    if !map_completion(cx, defined)? {
        return Err(type_error(cx));
    }
    Ok(())
}

fn to_length_u32<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<u32, Cx::Error> {
    let integer = to_integer_or_infinity_for_builtin(cx, value)?;
    if integer <= 0.0 {
        return Ok(0);
    }
    if !integer.is_finite() {
        return Ok(u32::MAX);
    }
    Ok(integer.min(f64::from(u32::MAX)) as u32)
}

fn to_length_u64<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<u64, Cx::Error> {
    let integer = to_integer_or_infinity_for_builtin(cx, value)?;
    if integer <= 0.0 {
        return Ok(0);
    }
    if !integer.is_finite() {
        return Ok(MAX_SAFE_INTEGER_U64);
    }
    Ok(integer.min(MAX_SAFE_INTEGER_U64 as f64) as u64)
}

fn to_boolean_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<bool, Cx::Error> {
    let boolean = {
        let agent = cx.agent();
        read::to_boolean(agent.heap().view(), value)
    };
    map_completion(cx, boolean)
}

fn array_like_length<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
) -> Result<u32, Cx::Error> {
    let length = get_property_from_object(
        cx,
        object_ref,
        PropertyKey::from_atom(WellKnownAtom::length.id()),
    )?;
    to_length_u32(cx, length)
}

fn array_like_length_u64<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
) -> Result<u64, Cx::Error> {
    let length = get_property_from_object(
        cx,
        object_ref,
        PropertyKey::from_atom(WellKnownAtom::length.id()),
    )?;
    to_length_u64(cx, length)
}

fn normalize_relative_index_u64(length: u64, relative: f64) -> u64 {
    if relative.is_nan() {
        return 0;
    }
    if relative < 0.0 {
        if !relative.is_finite() {
            return 0;
        }
        let computed = (length as f64) + relative;
        if computed <= 0.0 {
            0
        } else {
            computed as u64
        }
    } else if !relative.is_finite() {
        length
    } else {
        (relative.min(length as f64)) as u64
    }
}

fn array_like_index_property_key<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    index: u64,
) -> PropertyKey {
    if let Some(key) = PropertyKey::from_array_index(index) {
        return key;
    }
    property_key_from_text(cx, &index.to_string())
}

fn create_array_result<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    length_hint: usize,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    cx.create_array_object(cx.builtin_realm(), length_hint)
}

fn create_array_result_with_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: ObjectRef,
    length_hint: usize,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let root_shape = {
        let agent = cx.agent();
        agent.realm(realm).and_then(|record| record.root_shape())
    }
    .ok_or_else(|| type_error(cx))?;
    let array = cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_flags(ObjectFlags::extensible().union(ObjectFlags::ENGINE_ARRAY))
                .with_prototype(Some(prototype))
                .with_element_capacity(length_hint),
            AllocationLifetime::Default,
        )
    });
    let mut descriptor = PropertyDescriptor::new();
    descriptor.set_value(length_value(0));
    descriptor.set_writable(true);
    descriptor.set_enumerable(false);
    descriptor.set_configurable(false);
    let defined = {
        let agent = cx.agent();
        object::define_property(
            agent,
            array,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
            descriptor,
            AllocationLifetime::Default,
        )
    };
    if !map_completion(cx, defined)? {
        return Err(type_error(cx));
    }
    Ok(array)
}

fn array_result_capacity_hint(length: u64) -> usize {
    usize::try_from(length)
        .unwrap_or(ARRAY_RESULT_CAPACITY_HINT_LIMIT)
        .min(ARRAY_RESULT_CAPACITY_HINT_LIMIT)
}

fn create_array_result_for_length<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    length: u64,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let length = u32::try_from(length).map_err(|_| range_error(cx))?;
    let array = create_array_result(cx, array_result_capacity_hint(u64::from(length)))?;
    define_array_length(cx, array, length)?;
    Ok(array)
}

fn array_species_create_for_length<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    original: lyng_js_types::ObjectRef,
    length: u64,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    if !is_array_for_species(cx, original)? {
        return create_array_result_for_length(cx, length);
    }

    let constructor = get_property_from_object(
        cx,
        original,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
    )?;
    if constructor.is_undefined() {
        return create_array_result_for_length(cx, length);
    }
    let Some(constructor_object) = constructor.as_object_ref() else {
        return Err(type_error(cx));
    };

    let default_array = {
        let realm = cx.builtin_realm();
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().array())
    };
    if Some(constructor_object) == default_array {
        return create_array_result_for_length(cx, length);
    }
    if is_any_realm_array_constructor(cx, constructor_object) {
        return create_array_result_for_length(cx, length);
    }

    let species_symbol = {
        let agent = cx.agent();
        agent.well_known_symbol(WellKnownSymbolId::Species)
    };
    let Some(species_symbol) = species_symbol else {
        return create_array_result_for_length(cx, length);
    };
    let species = get_property_from_object(
        cx,
        constructor_object,
        PropertyKey::from_symbol(species_symbol),
    )?;
    if species.is_undefined() || species.is_null() || species.as_object_ref() == default_array {
        return create_array_result_for_length(cx, length);
    }

    let species_object = species.as_object_ref().ok_or_else(|| type_error(cx))?;
    cx.construct_to_completion(species_object, &[length_value_u64(length)], None)
}

fn is_concat_spreadable<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<bool, Cx::Error> {
    let Some(object_ref) = value.as_object_ref() else {
        return Ok(false);
    };
    let spreadable_symbol = {
        let agent = cx.agent();
        agent.well_known_symbol(WellKnownSymbolId::IsConcatSpreadable)
    };
    if let Some(spreadable_symbol) = spreadable_symbol {
        let spreadable =
            get_property_from_object(cx, object_ref, PropertyKey::from_symbol(spreadable_symbol))?;
        if !spreadable.is_undefined() {
            return to_boolean_for_builtin(cx, spreadable);
        }
    }
    is_array_for_species(cx, object_ref)
}

fn set_length_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    length: u64,
) -> Result<(), Cx::Error> {
    let key = PropertyKey::from_atom(WellKnownAtom::length.id());
    if !set_property_on_object_with_receiver(
        cx,
        object_ref,
        key,
        length_value_u64(length),
        Value::from_object_ref(object_ref),
    )? {
        return Err(type_error(cx));
    }
    Ok(())
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

fn typed_array_validated_object_and_record<Cx: PublicBuiltinDispatchContext>(
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
    let prototype = {
        let agent = cx.agent();
        let intrinsics = agent.realm(realm).map(|record| record.intrinsics());
        match kind {
            TypedArrayElementKind::Int8 => {
                intrinsics.and_then(|intrinsics| intrinsics.int8_array_prototype())
            }
            TypedArrayElementKind::Int16 => {
                intrinsics.and_then(|intrinsics| intrinsics.int16_array_prototype())
            }
            TypedArrayElementKind::Int32 => {
                intrinsics.and_then(|intrinsics| intrinsics.int32_array_prototype())
            }
            TypedArrayElementKind::Float32 => {
                intrinsics.and_then(|intrinsics| intrinsics.float32_array_prototype())
            }
            TypedArrayElementKind::Float64 => {
                intrinsics.and_then(|intrinsics| intrinsics.float64_array_prototype())
            }
            TypedArrayElementKind::BigInt64 => {
                intrinsics.and_then(|intrinsics| intrinsics.big_int64_array_prototype())
            }
            TypedArrayElementKind::BigUint64 => {
                intrinsics.and_then(|intrinsics| intrinsics.big_uint64_array_prototype())
            }
            TypedArrayElementKind::Uint32 => {
                intrinsics.and_then(|intrinsics| intrinsics.uint32_array_prototype())
            }
            TypedArrayElementKind::Uint16 => {
                intrinsics.and_then(|intrinsics| intrinsics.uint16_array_prototype())
            }
            TypedArrayElementKind::Uint8Clamped => {
                intrinsics.and_then(|intrinsics| intrinsics.uint8_clamped_array_prototype())
            }
            TypedArrayElementKind::Uint8 => {
                intrinsics.and_then(|intrinsics| intrinsics.uint8_array_prototype())
            }
        }
    };
    prototype.ok_or_else(|| type_error(cx))
}

fn typed_array_default_constructor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    kind: TypedArrayElementKind,
) -> Result<ObjectRef, Cx::Error> {
    let constructor = {
        let agent = cx.agent();
        let intrinsics = agent.realm(realm).map(|record| record.intrinsics());
        match kind {
            TypedArrayElementKind::Int8 => {
                intrinsics.and_then(|intrinsics| intrinsics.int8_array())
            }
            TypedArrayElementKind::Int16 => {
                intrinsics.and_then(|intrinsics| intrinsics.int16_array())
            }
            TypedArrayElementKind::Int32 => {
                intrinsics.and_then(|intrinsics| intrinsics.int32_array())
            }
            TypedArrayElementKind::Float32 => {
                intrinsics.and_then(|intrinsics| intrinsics.float32_array())
            }
            TypedArrayElementKind::Float64 => {
                intrinsics.and_then(|intrinsics| intrinsics.float64_array())
            }
            TypedArrayElementKind::BigInt64 => {
                intrinsics.and_then(|intrinsics| intrinsics.big_int64_array())
            }
            TypedArrayElementKind::BigUint64 => {
                intrinsics.and_then(|intrinsics| intrinsics.big_uint64_array())
            }
            TypedArrayElementKind::Uint32 => {
                intrinsics.and_then(|intrinsics| intrinsics.uint32_array())
            }
            TypedArrayElementKind::Uint16 => {
                intrinsics.and_then(|intrinsics| intrinsics.uint16_array())
            }
            TypedArrayElementKind::Uint8Clamped => {
                intrinsics.and_then(|intrinsics| intrinsics.uint8_clamped_array())
            }
            TypedArrayElementKind::Uint8 => {
                intrinsics.and_then(|intrinsics| intrinsics.uint8_array())
            }
        }
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
    typed_array_read_storage_bits(agent, record, index)
        .map(|bits| typed_array_storage_bits_to_value(agent, record.kind(), bits))
        .unwrap_or(Value::undefined())
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

fn collect_array_like_values_for_from_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    source: Value,
) -> Result<Vec<Value>, Cx::Error> {
    let source_object = cx.to_object_for_builtin_value(cx.builtin_realm(), source)?;
    let length = array_like_length_u64(cx, source_object)?;
    let mut values = Vec::new();
    for index in 0..length {
        let key = array_like_index_property_key(cx, index);
        values.push(get_property_from_object(cx, source_object, key)?);
    }
    Ok(values)
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
            TypedArrayPredicateKind::Find => {
                if selected {
                    return Ok(element);
                }
            }
            TypedArrayPredicateKind::FindIndex => {
                if selected {
                    return Ok(length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)));
                }
            }
            TypedArrayPredicateKind::FindLast => {
                if selected {
                    return Ok(element);
                }
            }
            TypedArrayPredicateKind::FindLastIndex => {
                if selected {
                    return Ok(length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)));
                }
            }
        }
    }
    Ok(match kind {
        TypedArrayPredicateKind::Every => Value::from_bool(true),
        TypedArrayPredicateKind::Some => Value::from_bool(false),
        TypedArrayPredicateKind::Find => Value::undefined(),
        TypedArrayPredicateKind::FindIndex => Value::from_smi(-1),
        TypedArrayPredicateKind::FindLast => Value::undefined(),
        TypedArrayPredicateKind::FindLastIndex => Value::from_smi(-1),
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

fn regexp_species_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    _cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Ok(invocation.this_value())
}

fn require_constructor_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if !cx.agent().objects().is_constructor(object) {
        return Err(type_error(cx));
    }
    Ok(object)
}

fn close_iterator_after_error<Cx: PublicBuiltinDispatchContext, T>(
    cx: &mut Cx,
    iterator_record: &mut iterator::IteratorRecord,
    error: Cx::Error,
) -> Result<T, Cx::Error> {
    let Some(thrown) = cx.extract_thrown_value(error)? else {
        unreachable!("non-abrupt builtin error should propagate")
    };
    let mut bridge = BuiltinIteratorBridge { cx };
    iterator::iterator_close(
        &mut bridge,
        iterator_record,
        Err(AbruptCompletion::throw(thrown)),
    )
}

fn create_array_from_values<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    values: &[Value],
) -> Result<ObjectRef, Cx::Error> {
    let array = create_array_result(cx, values.len())?;
    for (index, value) in values.iter().copied().enumerate() {
        let key = array_like_index_property_key(
            cx,
            u64::try_from(index).expect("array index should fit into u64"),
        );
        create_data_property_or_throw(cx, array, key, value)?;
    }
    define_array_length(
        cx,
        array,
        u32::try_from(values.len()).expect("Promise combinator result length should fit into u32"),
    )?;
    Ok(array)
}

fn iterable_to_values_list<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    iterable: Value,
) -> Result<Vec<Value>, Cx::Error> {
    let mut iterator_record = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::get_iterator(&mut bridge, iterable)?
    };
    let mut values = Vec::new();
    loop {
        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut iterator_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => {
                iterator_record.set_done(true);
                return Err(error);
            }
        };
        let Some(next) = next else {
            return Ok(values);
        };
        let next_value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let next_value = match next_value {
            Ok(next_value) => next_value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        values.push(next_value);
    }
}

fn try_create_data_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    key: PropertyKey,
    value: Value,
) -> Result<bool, Cx::Error> {
    let mut descriptor = PropertyDescriptor::new();
    descriptor.set_value(value);
    descriptor.set_writable(true);
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    proxy_define_property(cx, object_ref, key, descriptor, AllocationLifetime::Default)
}

fn create_data_property_or_throw<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    key: PropertyKey,
    value: Value,
) -> Result<(), Cx::Error> {
    if !try_create_data_property(cx, object_ref, key, value)? {
        return Err(type_error(cx));
    }
    Ok(())
}

fn is_integral_number(number: f64) -> bool {
    number.is_finite() && number == number.trunc()
}

fn scientific_digits(number: f64) -> Option<(Vec<u8>, i32)> {
    let formatted = format!("{:.110e}", number.abs());
    let (mantissa, exponent) = formatted.split_once('e')?;
    let exponent = exponent.parse::<i32>().ok()?;
    let digits = mantissa
        .bytes()
        .filter(|byte| *byte != b'.')
        .map(|byte| byte - b'0')
        .collect::<Vec<_>>();
    Some((digits, exponent))
}

fn increment_decimal_digits(digits: &mut Vec<u8>) -> bool {
    for digit in digits.iter_mut().rev() {
        if *digit < 9 {
            *digit += 1;
            return false;
        }
        *digit = 0;
    }
    digits.insert(0, 1);
    true
}

fn format_to_exponential(number: f64, fraction_digits: usize) -> Option<String> {
    if number == 0.0 {
        let sign = if number.is_sign_negative() { "-" } else { "" };
        if fraction_digits == 0 {
            return Some(format!("{sign}0e+0"));
        }
        return Some(format!("{sign}0.{}e+0", "0".repeat(fraction_digits)));
    }

    let negative = number.is_sign_negative();
    let (mut digits, mut exponent) = scientific_digits(number)?;
    let precision = fraction_digits + 1;
    let needs_round = digits
        .get(precision)
        .copied()
        .is_some_and(|digit| digit >= 5);
    digits.truncate(precision);
    while digits.len() < precision {
        digits.push(0);
    }
    if needs_round && increment_decimal_digits(&mut digits) {
        exponent += 1;
    }
    if digits.len() > precision {
        digits.truncate(precision);
    }

    let mut text = String::new();
    if negative {
        text.push('-');
    }
    text.push(char::from(b'0' + digits[0]));
    if fraction_digits > 0 {
        text.push('.');
        for digit in digits.iter().skip(1) {
            text.push(char::from(b'0' + *digit));
        }
    }
    text.push('e');
    if exponent >= 0 {
        text.push('+');
    }
    text.push_str(&exponent.to_string());
    Some(text)
}

fn format_to_precision(number: f64, precision: usize) -> Option<String> {
    if number == 0.0 {
        if precision == 1 {
            return Some("0".to_owned());
        }
        return Some(format!("0.{}", "0".repeat(precision - 1)));
    }

    let negative = number.is_sign_negative();
    let exponential = format_to_exponential(number.abs(), precision - 1)?;
    let (mantissa, exponent_text) = exponential.split_once('e')?;
    let exponent = exponent_text.parse::<i32>().ok()?;
    let signed_exponential = || {
        if negative {
            format!("-{exponential}")
        } else {
            exponential.clone()
        }
    };
    if exponent < -6 || exponent >= i32::try_from(precision).ok()? {
        return Some(signed_exponential());
    }

    let mut digits: String = mantissa.chars().filter(|ch| *ch != '.').collect();
    while digits.len() < precision {
        digits.push('0');
    }

    let mut text = String::new();
    if negative {
        text.push('-');
    }
    if exponent >= 0 {
        let integer_digits = usize::try_from(exponent + 1).ok()?;
        if integer_digits >= digits.len() {
            text.push_str(&digits);
            text.push_str(&"0".repeat(integer_digits - digits.len()));
        } else {
            text.push_str(&digits[..integer_digits]);
            text.push('.');
            text.push_str(&digits[integer_digits..]);
        }
    } else {
        text.push_str("0.");
        text.push_str(&"0".repeat(usize::try_from(-exponent - 1).ok()?));
        text.push_str(&digits);
    }
    Some(text)
}

fn number_value(number: f64) -> Value {
    if number == 0.0 && number.is_sign_negative() {
        Value::from_f64(-0.0)
    } else {
        Value::from_f64(number)
    }
}

fn argument_to_number<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<f64, Cx::Error> {
    to_number_for_builtin(cx, value)
}

fn radix_argument<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<u32, Cx::Error> {
    if value.is_undefined() {
        return Ok(10);
    }
    let radix = argument_to_number(cx, value)?;
    if !radix.is_finite() || radix != radix.trunc() || !(2.0..=36.0).contains(&radix) {
        return Err(range_error(cx));
    }
    Ok(radix as u32)
}

fn symbol_descriptive_string<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    symbol: lyng_js_types::SymbolRef,
) -> Result<String, Cx::Error> {
    let description = {
        let agent = cx.agent();
        let heap_view = agent.heap().view();
        heap_view
            .symbol_view(symbol)
            .and_then(|view| view.description())
    };
    if let Some(description) = description {
        let description_text = cx.value_to_string_text(Value::from_string_ref(description))?;
        Ok(format!("Symbol({description_text})"))
    } else {
        Ok("Symbol()".to_owned())
    }
}

fn string_value_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Value, Cx::Error> {
    let primitive = if value.is_object() {
        let mut bridge = BuiltinToPrimitiveBridge { cx };
        object::to_primitive(&mut bridge, value, object::ToPrimitiveHint::String)?
    } else {
        value
    };

    if let Some(string) = primitive.as_string_ref() {
        return Ok(Value::from_string_ref(string));
    }
    if let Some(symbol) = primitive.as_symbol_ref() {
        let text = symbol_descriptive_string(cx, symbol)?;
        return Ok(string_value(cx, &text));
    }
    if primitive.is_bigint() {
        let text = {
            let agent = cx.agent();
            object::bigint_to_string(agent, primitive, 10)
        };
        let text = map_completion(cx, text)?;
        return Ok(string_value(cx, &text));
    }

    let text = cx.value_to_string_text(primitive)?;
    Ok(string_value(cx, &text))
}

fn string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.new_target().is_none() {
        let value = if invocation.arguments().is_empty() {
            string_value(cx, "")
        } else {
            string_value_from_value(cx, invocation.arguments()[0])?
        };
        return Ok(value);
    }
    let value = if invocation.arguments().is_empty() {
        string_value(cx, "")
    } else {
        Value::from_string_ref(to_string_string_ref(cx, invocation.arguments()[0])?)
    };
    let realm = cx.builtin_realm();
    let default_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().string_prototype())
        .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?;
    primitive_wrapper_constructor(cx, realm, prototype, PrimitiveWrapperKind::String, value)
}

fn string_wrapper_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Value, Cx::Error> {
    if value.as_string_ref().is_some() {
        return Ok(value);
    }
    let Some(object_ref) = value.as_object_ref() else {
        return Err(type_error(cx));
    };
    let payload = {
        let agent = cx.agent();
        if agent.objects().primitive_wrapper_kind(object_ref) == Some(PrimitiveWrapperKind::String)
        {
            agent
                .objects()
                .primitive_wrapper_value(agent.heap().view(), object_ref)
        } else {
            None
        }
    };
    payload.ok_or_else(|| type_error(cx))
}

fn string_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    string_wrapper_value(cx, invocation.this_value())
}

fn string_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    string_wrapper_value(cx, invocation.this_value())
}

fn string_concat_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let this_string = string_this_ref(cx, invocation.this_value())?;
    let mut text = string_ref_text(cx, this_string)?;
    for argument in invocation.arguments() {
        let argument_string = to_string_string_ref(cx, *argument)?;
        text.push_str(&string_ref_text(cx, argument_string)?);
    }
    Ok(string_value(cx, &text))
}

fn to_string_string_ref<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<StringRef, Cx::Error> {
    let primitive = if value.is_object() {
        let mut bridge = BuiltinToPrimitiveBridge { cx };
        object::to_primitive(&mut bridge, value, object::ToPrimitiveHint::String)?
    } else {
        value
    };

    if let Some(string) = primitive.as_string_ref() {
        return Ok(string);
    }
    if primitive.as_symbol_ref().is_some() {
        return Err(type_error(cx));
    }
    if primitive.is_bigint() {
        let text = {
            let agent = cx.agent();
            object::bigint_to_string(agent, primitive, 10)
        };
        let text = map_completion(cx, text)?;
        let value = string_value(cx, &text);
        return Ok(value
            .as_string_ref()
            .expect("string_value should always allocate a StringRef"));
    }

    let text = cx.value_to_string_text(primitive)?;
    let value = string_value(cx, &text);
    Ok(value
        .as_string_ref()
        .expect("string_value should always allocate a StringRef"))
}

fn string_this_ref<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<StringRef, Cx::Error> {
    if value.is_null() || value.is_undefined() {
        return Err(type_error(cx));
    }
    to_string_string_ref(cx, value)
}

fn string_ref_code_units<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    string: StringRef,
) -> Result<Vec<u16>, Cx::Error> {
    let Some(view) = ({
        let agent = cx.agent();
        agent.heap().view().string_view(string)
    }) else {
        return Err(type_error(cx));
    };

    if let Some(bytes) = view.latin1_bytes() {
        return Ok(bytes.iter().copied().map(u16::from).collect());
    }

    let Some(bytes) = view.utf16_bytes() else {
        return Ok(Vec::new());
    };
    Ok(bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect())
}

fn string_ref_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    string: StringRef,
) -> Result<String, Cx::Error> {
    cx.value_to_string_text(Value::from_string_ref(string))
}

fn string_from_code_units<Cx: PublicBuiltinDispatchContext>(cx: &mut Cx, units: &[u16]) -> Value {
    let string = {
        let agent = cx.agent();
        if units.iter().all(|unit| u8::try_from(*unit).is_ok()) {
            let bytes: Vec<u8> = units
                .iter()
                .map(|unit| u8::try_from(*unit).expect("Latin-1 unit should fit into u8"))
                .collect();
            agent.heap_mut().mutator().alloc_string(
                StringEncoding::Latin1,
                u32::try_from(bytes.len()).expect("string length must fit into u32"),
                &bytes,
                None,
                AllocationLifetime::Default,
            )
        } else {
            let mut bytes = Vec::with_capacity(units.len() * 2);
            for unit in units {
                bytes.extend_from_slice(&unit.to_le_bytes());
            }
            agent.heap_mut().mutator().alloc_string(
                StringEncoding::Utf16,
                u32::try_from(units.len()).expect("string length must fit into u32"),
                &bytes,
                None,
                AllocationLifetime::Default,
            )
        }
    };
    Value::from_string_ref(string)
}

fn to_integer_or_infinity_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<f64, Cx::Error> {
    let number = to_number_for_builtin(cx, value)?;
    if number.is_nan() || number == 0.0 {
        return Ok(0.0);
    }
    if !number.is_finite() {
        return Ok(number);
    }
    Ok(number.trunc())
}

fn to_uint32_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<u32, Cx::Error> {
    let number = to_number_for_builtin(cx, value)?;
    if number.is_nan() || number == 0.0 || !number.is_finite() {
        return Ok(0);
    }
    let integer = number.trunc();
    let mut modulo = integer % 4_294_967_296.0;
    if modulo < 0.0 {
        modulo += 4_294_967_296.0;
    }
    Ok(modulo as u32)
}

fn to_uint8_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<u8, Cx::Error> {
    let number = to_number_for_builtin(cx, value)?;
    if number.is_nan() || number == 0.0 || !number.is_finite() {
        return Ok(0);
    }
    let integer = number.trunc();
    let mut modulo = integer % 256.0;
    if modulo < 0.0 {
        modulo += 256.0;
    }
    Ok(modulo as u8)
}

fn to_uint8_clamp_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<u8, Cx::Error> {
    let number = to_number_for_builtin(cx, value)?;
    if number.is_nan() || number <= 0.0 {
        return Ok(0);
    }
    if number >= 255.0 {
        return Ok(255);
    }
    let floor = number.floor();
    if floor + 0.5 < number {
        return Ok((floor as u8).saturating_add(1));
    }
    if number < floor + 0.5 {
        return Ok(floor as u8);
    }
    let floor_u8 = floor as u8;
    if floor_u8 % 2 == 1 {
        Ok(floor_u8.saturating_add(1))
    } else {
        Ok(floor_u8)
    }
}

fn to_length_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<usize, Cx::Error> {
    let integer = to_integer_or_infinity_for_builtin(cx, value)?;
    if integer <= 0.0 {
        return Ok(0);
    }
    if !integer.is_finite() {
        return Ok(usize::MAX);
    }
    Ok(integer.min(usize::MAX as f64) as usize)
}

fn to_index_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<u64, Cx::Error> {
    const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

    if value.is_undefined() {
        return Ok(0);
    }
    let integer = to_integer_or_infinity_for_builtin(cx, value)?;
    if !integer.is_finite() || integer < 0.0 || integer > MAX_SAFE_INTEGER {
        return Err(range_error(cx));
    }
    Ok(integer as u64)
}

fn string_position_index(position: f64, length: usize) -> Option<usize> {
    if position == 0.0 {
        return (length > 0).then_some(0);
    }
    if !position.is_finite() || position < 0.0 {
        return None;
    }
    let index = position as usize;
    (index < length).then_some(index)
}

fn allocate_array_like_result<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    length: u32,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let object = create_array_result(cx, array_result_capacity_hint(u64::from(length)))?;
    define_array_length(cx, object, length)?;
    Ok(object)
}

fn callable_object_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Option<lyng_js_types::ObjectRef> {
    let object_ref = value.as_object_ref()?;
    let header = {
        let agent = cx.agent();
        agent
            .objects()
            .object_header(agent.heap().view(), object_ref)
    }?;
    (header.kind() == ObjectKind::Function).then_some(object_ref)
}

fn find_subsequence(haystack: &[u16], needle: &[u16], start: usize) -> Option<usize> {
    if needle.is_empty() {
        return Some(start.min(haystack.len()));
    }
    if needle.len() > haystack.len() || start > haystack.len().saturating_sub(needle.len()) {
        return None;
    }
    haystack[start..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|offset| offset + start)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RegExpExecState {
    flags: lyng_js_objects::RegExpObjectFlags,
    matched: lyng_js_objects::RegExpMatchRecord,
}

fn usize_index_value(index: usize) -> Value {
    i32::try_from(index)
        .map(Value::from_smi)
        .unwrap_or_else(|_| Value::from_f64(index as f64))
}

fn code_unit_range_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    units: &[u16],
    range: std::ops::Range<usize>,
) -> Value {
    string_from_code_units(cx, &units[range.start..range.end])
}

fn allocate_regexp_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
    pattern: &str,
    flags: &str,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let root_shape = {
        let agent = cx.agent();
        agent.realm(realm).and_then(|record| record.root_shape())
    }
    .ok_or_else(|| type_error(cx))?;
    let payload =
        lyng_js_objects::RegExpPayload::compile(pattern, flags).map_err(|_| syntax_error(cx))?;
    let object = cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_prototype(Some(prototype))
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::RegExp)),
            AllocationLifetime::Default,
        );
        let stored = objects.store_regexp_payload(object, payload);
        debug_assert!(stored, "fresh RegExp objects should accept payload storage");
        object
    });
    let last_index_key = regexp_last_index_key(cx);
    define_data_property_with_attrs(
        cx,
        object,
        last_index_key,
        Value::from_smi(0),
        true,
        false,
        false,
    )?;
    Ok(object)
}

fn regexp_object_flags<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
) -> Result<lyng_js_objects::RegExpObjectFlags, Cx::Error> {
    let flags = {
        let agent = cx.agent();
        agent
            .objects()
            .regexp_payload(object_ref)
            .map(|payload| payload.flags())
    };
    if let Some(flags) = flags {
        return Ok(flags);
    }
    if current_intrinsic_regexp_prototype(cx) == Some(object_ref) {
        return Ok(lyng_js_objects::RegExpObjectFlags::default());
    }
    Err(type_error(cx))
}

fn regexp_object_flag_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
) -> Result<String, Cx::Error> {
    let text = {
        let agent = cx.agent();
        agent
            .objects()
            .regexp_payload(object_ref)
            .map(|payload| payload.flag_text().to_owned())
    };
    if let Some(text) = text {
        return Ok(text);
    }
    if current_intrinsic_regexp_prototype(cx) == Some(object_ref) {
        return Ok(String::new());
    }
    Err(type_error(cx))
}

fn regexp_object_source_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
) -> Result<String, Cx::Error> {
    let text = {
        let agent = cx.agent();
        agent.objects().regexp_payload(object_ref).map(|payload| {
            if payload.source().is_empty() {
                "(?:)".to_owned()
            } else {
                payload.source().to_owned()
            }
        })
    };
    if let Some(text) = text {
        return Ok(text);
    }
    if current_intrinsic_regexp_prototype(cx) == Some(object_ref) {
        return Ok("(?:)".to_owned());
    }
    Err(type_error(cx))
}

fn advance_string_index(units: &[u16], index: usize, unicode_aware: bool) -> usize {
    if !unicode_aware || index + 1 >= units.len() {
        return index.saturating_add(1);
    }
    let first = units[index];
    let second = units[index + 1];
    if (0xD800..=0xDBFF).contains(&first) && (0xDC00..=0xDFFF).contains(&second) {
        index + 2
    } else {
        index + 1
    }
}

fn allocate_named_capture_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    captures: &[lyng_js_objects::RegExpNamedCapture],
    units: &[u16],
    use_indices: bool,
) -> Result<Option<Value>, Cx::Error> {
    if captures.is_empty() {
        return Ok(None);
    }
    let object = cx.allocate_ordinary_object_with_prototype(cx.builtin_realm(), None)?;
    for capture in captures {
        let atom = {
            let agent = cx.agent();
            agent.atoms_mut().intern_collectible(capture.name())
        };
        let value = match capture.range() {
            Some(range) if use_indices => {
                let pair = allocate_array_like_result(cx, 2)?;
                define_data_property_with_attrs(
                    cx,
                    pair,
                    PropertyKey::Index(0),
                    usize_index_value(range.start),
                    true,
                    true,
                    true,
                )?;
                define_data_property_with_attrs(
                    cx,
                    pair,
                    PropertyKey::Index(1),
                    usize_index_value(range.end),
                    true,
                    true,
                    true,
                )?;
                Value::from_object_ref(pair)
            }
            Some(range) => code_unit_range_value(cx, units, range),
            None => Value::undefined(),
        };
        define_data_property_with_attrs(
            cx,
            object,
            PropertyKey::from_atom(atom),
            value,
            true,
            true,
            true,
        )?;
    }
    Ok(Some(Value::from_object_ref(object)))
}

fn build_regexp_indices_result<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    units: &[u16],
    state: &RegExpExecState,
) -> Result<Value, Cx::Error> {
    let matched = &state.matched;
    let object = allocate_array_like_result(
        cx,
        u32::try_from(matched.captures().len() + 1).unwrap_or(u32::MAX),
    )?;
    let pair = allocate_array_like_result(cx, 2)?;
    define_data_property_with_attrs(
        cx,
        pair,
        PropertyKey::Index(0),
        usize_index_value(matched.start()),
        true,
        true,
        true,
    )?;
    define_data_property_with_attrs(
        cx,
        pair,
        PropertyKey::Index(1),
        usize_index_value(matched.end()),
        true,
        true,
        true,
    )?;
    define_data_property_with_attrs(
        cx,
        object,
        PropertyKey::Index(0),
        Value::from_object_ref(pair),
        true,
        true,
        true,
    )?;
    for (offset, capture) in matched.captures().iter().enumerate() {
        let value = match capture {
            Some(range) => {
                let pair = allocate_array_like_result(cx, 2)?;
                define_data_property_with_attrs(
                    cx,
                    pair,
                    PropertyKey::Index(0),
                    usize_index_value(range.start),
                    true,
                    true,
                    true,
                )?;
                define_data_property_with_attrs(
                    cx,
                    pair,
                    PropertyKey::Index(1),
                    usize_index_value(range.end),
                    true,
                    true,
                    true,
                )?;
                Value::from_object_ref(pair)
            }
            None => Value::undefined(),
        };
        define_data_property_with_attrs(
            cx,
            object,
            PropertyKey::Index(u32::try_from(offset + 1).unwrap_or(u32::MAX)),
            value,
            true,
            true,
            true,
        )?;
    }
    let groups_atom = {
        let agent = cx.agent();
        agent.atoms_mut().intern_collectible("groups")
    };
    let groups = allocate_named_capture_object(cx, matched.named_captures(), units, true)?
        .unwrap_or(Value::undefined());
    define_data_property_with_attrs(
        cx,
        object,
        PropertyKey::from_atom(groups_atom),
        groups,
        true,
        true,
        true,
    )?;
    Ok(Value::from_object_ref(object))
}

fn build_regexp_match_result<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    units: &[u16],
    input_value: Value,
    state: &RegExpExecState,
) -> Result<Value, Cx::Error> {
    let matched = &state.matched;
    let object = allocate_array_like_result(
        cx,
        u32::try_from(matched.captures().len() + 1).unwrap_or(u32::MAX),
    )?;
    let matched_value = code_unit_range_value(cx, units, matched.range());
    define_data_property_with_attrs(
        cx,
        object,
        PropertyKey::Index(0),
        matched_value,
        true,
        true,
        true,
    )?;
    for (offset, capture) in matched.captures().iter().enumerate() {
        let value = capture
            .clone()
            .map(|range| code_unit_range_value(cx, units, range))
            .unwrap_or(Value::undefined());
        define_data_property_with_attrs(
            cx,
            object,
            PropertyKey::Index(u32::try_from(offset + 1).unwrap_or(u32::MAX)),
            value,
            true,
            true,
            true,
        )?;
    }
    let (index_atom, input_atom, groups_atom, indices_atom) = {
        let agent = cx.agent();
        (
            agent.atoms_mut().intern_collectible("index"),
            agent.atoms_mut().intern_collectible("input"),
            agent.atoms_mut().intern_collectible("groups"),
            agent.atoms_mut().intern_collectible("indices"),
        )
    };
    define_data_property_with_attrs(
        cx,
        object,
        PropertyKey::from_atom(index_atom),
        usize_index_value(matched.start()),
        true,
        true,
        true,
    )?;
    define_data_property_with_attrs(
        cx,
        object,
        PropertyKey::from_atom(input_atom),
        input_value,
        true,
        true,
        true,
    )?;
    let groups = allocate_named_capture_object(cx, matched.named_captures(), units, false)?
        .unwrap_or(Value::undefined());
    define_data_property_with_attrs(
        cx,
        object,
        PropertyKey::from_atom(groups_atom),
        groups,
        true,
        true,
        true,
    )?;
    if state.flags.has_indices() {
        let indices = build_regexp_indices_result(cx, units, state)?;
        define_data_property_with_attrs(
            cx,
            object,
            PropertyKey::from_atom(indices_atom),
            indices,
            true,
            true,
            true,
        )?;
    }
    Ok(Value::from_object_ref(object))
}

fn build_regexp_global_match_result<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    units: &[u16],
    matches: &[lyng_js_objects::RegExpMatchRecord],
) -> Result<Value, Cx::Error> {
    let object = allocate_array_like_result(cx, u32::try_from(matches.len()).unwrap_or(u32::MAX))?;
    for (index, matched) in matches.iter().enumerate() {
        let matched_value = code_unit_range_value(cx, units, matched.range());
        define_data_property_with_attrs(
            cx,
            object,
            PropertyKey::Index(u32::try_from(index).unwrap_or(u32::MAX)),
            matched_value,
            true,
            true,
            true,
        )?;
    }
    Ok(Value::from_object_ref(object))
}

fn regexp_exec_state<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    units: &[u16],
) -> Result<Option<RegExpExecState>, Cx::Error> {
    let flags = regexp_object_flags(cx, object_ref)?;
    let last_index_key = regexp_last_index_key(cx);
    let last_index = cx.get_property_value(Value::from_object_ref(object_ref), last_index_key)?;
    let last_index = to_length_for_builtin(cx, last_index)?;
    let uses_stateful_last_index = flags.global() || flags.sticky();
    let start_index = if uses_stateful_last_index {
        last_index
    } else {
        0
    };
    if uses_stateful_last_index && start_index > units.len() {
        set_data_property_value(cx, object_ref, last_index_key, Value::from_smi(0))?;
        return Ok(None);
    }

    let matched = {
        let agent = cx.agent();
        agent
            .objects()
            .regexp_payload(object_ref)
            .and_then(|payload| payload.find_from_code_units(units, start_index))
    };
    let matched = matched.filter(|matched| !flags.sticky() || matched.start() == start_index);
    if let Some(matched) = matched {
        if uses_stateful_last_index {
            let next_index = if matched.start() == matched.end() {
                advance_string_index(units, matched.end(), flags.unicode_aware())
            } else {
                matched.end()
            };
            set_data_property_value(
                cx,
                object_ref,
                last_index_key,
                usize_index_value(next_index),
            )?;
        }
        return Ok(Some(RegExpExecState { flags, matched }));
    }

    if uses_stateful_last_index {
        set_data_property_value(cx, object_ref, last_index_key, Value::from_smi(0))?;
    }
    Ok(None)
}

fn capture_range_for_name(
    captures: &[lyng_js_objects::RegExpNamedCapture],
    name: &str,
) -> Option<std::ops::Range<usize>> {
    captures
        .iter()
        .find(|capture| capture.name() == name)
        .and_then(lyng_js_objects::RegExpNamedCapture::range)
}

fn code_unit_ascii(unit: u16) -> Option<u8> {
    u8::try_from(unit).ok().filter(u8::is_ascii)
}

fn expand_regexp_replacement_template(
    template_units: &[u16],
    source_units: &[u16],
    state: &RegExpExecState,
) -> Vec<u16> {
    let mut result = Vec::with_capacity(template_units.len());
    let matched = &state.matched;
    let captures = matched.captures();
    let named_captures = matched.named_captures();
    let mut index = 0;
    while index < template_units.len() {
        if template_units[index] != u16::from(b'$') {
            result.push(template_units[index]);
            index += 1;
            continue;
        }
        let Some(next) = template_units.get(index + 1).copied() else {
            result.push(u16::from(b'$'));
            index += 1;
            continue;
        };
        match code_unit_ascii(next).map(char::from) {
            Some('$') => {
                result.push(u16::from(b'$'));
                index += 2;
            }
            Some('&') => {
                result.extend_from_slice(&source_units[matched.start()..matched.end()]);
                index += 2;
            }
            Some('`') => {
                result.extend_from_slice(&source_units[..matched.start()]);
                index += 2;
            }
            Some('\'') => {
                result.extend_from_slice(&source_units[matched.end()..]);
                index += 2;
            }
            Some('<') => {
                let mut end = index + 2;
                while end < template_units.len() && template_units[end] != u16::from(b'>') {
                    end += 1;
                }
                if end == template_units.len() {
                    result.push(u16::from(b'$'));
                    index += 1;
                    continue;
                }
                let name = String::from_utf16_lossy(&template_units[index + 2..end]);
                if let Some(range) = capture_range_for_name(named_captures, &name) {
                    result.extend_from_slice(&source_units[range.start..range.end]);
                }
                index = end + 1;
            }
            Some(digit @ '0'..='9') => {
                let first = usize::from((digit as u8) - b'0');
                let mut capture_index = first;
                let mut digit_count = 1;
                if let Some(second) = template_units
                    .get(index + 2)
                    .and_then(|unit| code_unit_ascii(*unit))
                    .filter(u8::is_ascii_digit)
                {
                    let candidate = first * 10 + usize::from(second - b'0');
                    digit_count = 2;
                    capture_index = candidate;
                    if capture_index > captures.len() && first != 0 {
                        digit_count = 1;
                        capture_index = first;
                    }
                }
                if (1..=captures.len()).contains(&capture_index) {
                    if let Some(range) = captures[capture_index - 1].clone() {
                        result.extend_from_slice(&source_units[range.start..range.end]);
                    }
                    index += 1 + digit_count;
                } else {
                    result.push(u16::from(b'$'));
                    result.extend_from_slice(&template_units[index + 1..index + 1 + digit_count]);
                    index += 1 + digit_count;
                }
            }
            _ => {
                result.push(u16::from(b'$'));
                index += 1;
            }
        }
    }
    result
}

fn string_char_at_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let position = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let Some(index) = string_position_index(position, units.len()) else {
        return Ok(string_from_code_units(cx, &[]));
    };
    Ok(string_from_code_units(cx, &units[index..index + 1]))
}

fn string_char_code_at_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let position = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let Some(index) = string_position_index(position, units.len()) else {
        return Ok(Value::from_f64(f64::NAN));
    };
    Ok(Value::from_smi(i32::from(units[index])))
}

fn string_from_char_code_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let mut units = Vec::with_capacity(invocation.arguments().len());
    for value in invocation.arguments().iter().copied() {
        units.push((to_uint32_for_builtin(cx, value)? & 0xffff) as u16);
    }
    Ok(string_from_code_units(cx, &units))
}

fn append_code_point_units(units: &mut Vec<u16>, code_point: u32) {
    if code_point <= 0xFFFF {
        units.push(code_point as u16);
        return;
    }

    let adjusted = code_point - 0x1_0000;
    units.push(0xD800 | ((adjusted >> 10) as u16));
    units.push(0xDC00 | ((adjusted as u16) & 0x03FF));
}

fn string_from_code_point_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let mut units = Vec::with_capacity(invocation.arguments().len());
    for value in invocation.arguments().iter().copied() {
        let number = to_number_for_builtin(cx, value)?;
        if !number.is_finite() || number.trunc() != number || !(0.0..=1_114_111.0).contains(&number)
        {
            return Err(range_error(cx));
        }
        append_code_point_units(&mut units, number as u32);
    }
    Ok(string_from_code_units(cx, &units))
}

fn string_raw_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let template_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let template = cx.to_object_for_builtin_value(cx.builtin_realm(), template_value)?;
    let raw_key = property_key_from_text(cx, "raw");
    let raw_value = cx.get_property_value(Value::from_object_ref(template), raw_key)?;
    let raw = cx.to_object_for_builtin_value(cx.builtin_realm(), raw_value)?;
    let length_value = cx.get_property_value(
        Value::from_object_ref(raw),
        PropertyKey::from_atom(WellKnownAtom::length.id()),
    )?;
    let literal_segments = to_length_for_builtin(cx, length_value)?;
    if literal_segments == 0 {
        return Ok(string_from_code_units(cx, &[]));
    }

    let mut result = Vec::new();
    for index in 0..literal_segments {
        let key = array_like_index_property_key(
            cx,
            u64::try_from(index).expect("raw template index must fit into u64"),
        );
        let segment = cx.get_property_value(Value::from_object_ref(raw), key)?;
        let segment = to_string_string_ref(cx, segment)?;
        result.extend_from_slice(&string_ref_code_units(cx, segment)?);

        if index + 1 == literal_segments {
            break;
        }
        if let Some(substitution) = invocation.arguments().get(index + 1).copied() {
            let substitution = to_string_string_ref(cx, substitution)?;
            result.extend_from_slice(&string_ref_code_units(cx, substitution)?);
        }
    }

    Ok(string_from_code_units(cx, &result))
}

fn string_at_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let relative_index = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let index = if relative_index < 0.0 {
        units.len() as f64 + relative_index
    } else {
        relative_index
    };
    if !index.is_finite() || index < 0.0 || index >= units.len() as f64 {
        return Ok(Value::undefined());
    }
    let index = index as usize;
    Ok(string_from_code_units(cx, &units[index..index + 1]))
}

fn string_code_point_at_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let position = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let Some(index) = string_position_index(position, units.len()) else {
        return Ok(Value::undefined());
    };
    let first = units[index];
    let code_point = if (0xD800..=0xDBFF).contains(&first) {
        if let Some(second) = units.get(index + 1).copied() {
            if (0xDC00..=0xDFFF).contains(&second) {
                0x1_0000 + ((u32::from(first - 0xD800)) << 10) + u32::from(second - 0xDC00)
            } else {
                u32::from(first)
            }
        } else {
            u32::from(first)
        }
    } else {
        u32::from(first)
    };
    Ok(Value::from_smi(
        i32::try_from(code_point).expect("Unicode code points fit into i32"),
    ))
}

fn regexp_search_value_is_rejected<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<bool, Cx::Error> {
    is_regexp_value(cx, value)
}

fn string_index_of_units(source: &[u16], search: &[u16], position: usize) -> i32 {
    find_subsequence(source, search, position)
        .and_then(|index| i32::try_from(index).ok())
        .unwrap_or(-1)
}

fn string_index_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let search_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let search_units = string_ref_code_units(cx, search_ref)?;
    let position = if let Some(value) = invocation.arguments().get(1).copied() {
        to_integer_or_infinity_for_builtin(cx, value)?
    } else {
        0.0
    };
    let start = if position.is_nan() || position <= 0.0 {
        0
    } else if !position.is_finite() {
        source_units.len()
    } else {
        (position as usize).min(source_units.len())
    };
    Ok(Value::from_smi(string_index_of_units(
        &source_units,
        &search_units,
        start,
    )))
}

fn string_includes_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let search_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if regexp_search_value_is_rejected(cx, search_value)? {
        return Err(type_error(cx));
    }
    let search_ref = to_string_string_ref(cx, search_value)?;
    let search_units = string_ref_code_units(cx, search_ref)?;
    let position = if let Some(value) = invocation.arguments().get(1).copied() {
        to_integer_or_infinity_for_builtin(cx, value)?
    } else {
        0.0
    };
    let start = if position.is_nan() || position <= 0.0 {
        0
    } else if !position.is_finite() {
        source_units.len()
    } else {
        (position as usize).min(source_units.len())
    };
    Ok(Value::from_bool(
        find_subsequence(&source_units, &search_units, start).is_some(),
    ))
}

fn string_ends_with_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let search_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if regexp_search_value_is_rejected(cx, search_value)? {
        return Err(type_error(cx));
    }
    let search_ref = to_string_string_ref(cx, search_value)?;
    let search_units = string_ref_code_units(cx, search_ref)?;
    let end_position = if let Some(value) = invocation.arguments().get(1).copied() {
        to_integer_or_infinity_for_builtin(cx, value)?
    } else {
        source_units.len() as f64
    };
    let end = if end_position.is_nan() || end_position <= 0.0 {
        0
    } else if !end_position.is_finite() {
        source_units.len()
    } else {
        (end_position as usize).min(source_units.len())
    };
    let Some(start) = end.checked_sub(search_units.len()) else {
        return Ok(Value::from_bool(false));
    };
    Ok(Value::from_bool(
        source_units[start..end] == search_units[..],
    ))
}

fn string_locale_compare_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_text = string_ref_text(cx, source_ref)?;
    let that_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let that_text = string_ref_text(cx, that_ref)?;
    let source_key = normalize_text_for_form(&source_text, "NFD").ok_or_else(|| range_error(cx))?;
    let that_key = normalize_text_for_form(&that_text, "NFD").ok_or_else(|| range_error(cx))?;
    let result = match source_key.cmp(&that_key) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    };
    Ok(Value::from_smi(result))
}

fn is_well_formed_utf16(units: &[u16]) -> bool {
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit) {
            if !matches!(units.get(index + 1), Some(next) if (0xDC00..=0xDFFF).contains(next)) {
                return false;
            }
            index += 2;
            continue;
        }
        if (0xDC00..=0xDFFF).contains(&unit) {
            return false;
        }
        index += 1;
    }
    true
}

fn string_is_well_formed_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    Ok(Value::from_bool(is_well_formed_utf16(&units)))
}

fn to_well_formed_utf16(units: &[u16]) -> Vec<u16> {
    let mut result = Vec::with_capacity(units.len());
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit) {
            if matches!(units.get(index + 1), Some(next) if (0xDC00..=0xDFFF).contains(next)) {
                result.push(unit);
                result.push(units[index + 1]);
                index += 2;
                continue;
            }
            result.push(0xFFFD);
            index += 1;
            continue;
        }
        if (0xDC00..=0xDFFF).contains(&unit) {
            result.push(0xFFFD);
        } else {
            result.push(unit);
        }
        index += 1;
    }
    result
}

fn string_to_well_formed_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    Ok(string_from_code_units(cx, &to_well_formed_utf16(&units)))
}

fn is_ecmascript_trim_unit(unit: u16) -> bool {
    matches!(
        unit,
        0x0009 | 0x000A | 0x000B | 0x000C | 0x000D | 0x0020 | 0x00A0 | 0x1680 | 0x2000
            ..=0x200A | 0x2028 | 0x2029 | 0x202F | 0x205F | 0x3000 | 0xFEFF
    )
}

fn string_trim_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    trim_start: bool,
    trim_end: bool,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let mut start = 0;
    let mut end = units.len();
    if trim_start {
        while start < end && is_ecmascript_trim_unit(units[start]) {
            start += 1;
        }
    }
    if trim_end {
        while end > start && is_ecmascript_trim_unit(units[end - 1]) {
            end -= 1;
        }
    }
    Ok(string_from_code_units(cx, &units[start..end]))
}

enum StringCaseMapping {
    Lower,
    Upper,
}

fn push_char_units(output: &mut Vec<u16>, ch: char) {
    let mut buffer = [0_u16; 2];
    output.extend_from_slice(ch.encode_utf16(&mut buffer));
}

fn push_code_point_units(output: &mut Vec<u16>, code_point: u32) {
    if let Some(ch) = char::from_u32(code_point) {
        push_char_units(output, ch);
    } else if let Ok(unit) = u16::try_from(code_point) {
        output.push(unit);
    }
}

fn utf16_code_points(units: &[u16]) -> Vec<u32> {
    let mut points = Vec::with_capacity(units.len());
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit)
            && matches!(units.get(index + 1), Some(next) if (0xDC00..=0xDFFF).contains(next))
        {
            let trailing = units[index + 1];
            points
                .push(0x1_0000 + ((u32::from(unit - 0xD800)) << 10) + u32::from(trailing - 0xDC00));
            index += 2;
        } else {
            points.push(u32::from(unit));
            index += 1;
        }
    }
    points
}

fn is_case_ignorable_code_point(code_point: u32) -> bool {
    matches!(
        code_point,
        0x00AD | 0x0345 | 0x180E | 0x0300..=0x036F | 0x1D242
    )
}

fn is_cased_code_point(code_point: u32) -> bool {
    if is_case_ignorable_code_point(code_point) {
        return false;
    }
    if matches!(
        code_point,
        0x0041..=0x005A
            | 0x0061..=0x007A
            | 0x00C0..=0x024F
            | 0x0391..=0x03A9
            | 0x03B1..=0x03C9
            | 0x1D4A2
    ) {
        return true;
    }
    let Some(ch) = char::from_u32(code_point) else {
        return false;
    };
    let lower: String = ch.to_lowercase().collect();
    let upper: String = ch.to_uppercase().collect();
    lower != ch.to_string() || upper != ch.to_string()
}

fn is_final_sigma_context(points: &[u32], index: usize) -> bool {
    points[..index].iter().copied().any(is_cased_code_point)
        && !points[index + 1..].iter().copied().any(is_cased_code_point)
}

fn string_case_mapping_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    mapping: StringCaseMapping,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let mut mapped = Vec::with_capacity(units.len());
    let points = utf16_code_points(&units);
    for (index, code_point) in points.iter().copied().enumerate() {
        if matches!(mapping, StringCaseMapping::Lower) && code_point == 0x03A3 {
            push_code_point_units(
                &mut mapped,
                if is_final_sigma_context(&points, index) {
                    0x03C2
                } else {
                    0x03C3
                },
            );
            continue;
        }
        let Some(ch) = char::from_u32(code_point) else {
            push_code_point_units(&mut mapped, code_point);
            continue;
        };
        match mapping {
            StringCaseMapping::Lower => {
                for ch in ch.to_lowercase() {
                    push_char_units(&mut mapped, ch);
                }
            }
            StringCaseMapping::Upper => {
                for ch in ch.to_uppercase() {
                    push_char_units(&mut mapped, ch);
                }
            }
        }
    }
    Ok(string_from_code_units(cx, &mapped))
}

fn canonical_combining_class(code_point: u32) -> u8 {
    match code_point {
        0x093C => 7,
        0x031B => 216,
        0x0323 => 220,
        0x0327 => 202,
        0x0301 | 0x0302 | 0x0306 | 0x0307 | 0x0308 | 0x030A => 230,
        _ => 0,
    }
}

fn decompose_hangul(code_point: u32, output: &mut Vec<u32>) -> bool {
    const S_BASE: u32 = 0xAC00;
    const L_BASE: u32 = 0x1100;
    const V_BASE: u32 = 0x1161;
    const T_BASE: u32 = 0x11A7;
    const L_COUNT: u32 = 19;
    const V_COUNT: u32 = 21;
    const T_COUNT: u32 = 28;
    const N_COUNT: u32 = V_COUNT * T_COUNT;
    const S_COUNT: u32 = L_COUNT * N_COUNT;

    if !(S_BASE..S_BASE + S_COUNT).contains(&code_point) {
        return false;
    }
    let s_index = code_point - S_BASE;
    output.push(L_BASE + s_index / N_COUNT);
    output.push(V_BASE + (s_index % N_COUNT) / T_COUNT);
    let trailing = s_index % T_COUNT;
    if trailing != 0 {
        output.push(T_BASE + trailing);
    }
    true
}

fn decompose_code_point(code_point: u32, compatibility: bool, output: &mut Vec<u32>) {
    if decompose_hangul(code_point, output) {
        return;
    }
    let decomposition: Option<&'static [u32]> = match code_point {
        0x00C5 => Some(&[0x0041, 0x030A]),
        0x00C7 => Some(&[0x0043, 0x0327]),
        0x00C9 => Some(&[0x0045, 0x0301]),
        0x00E4 => Some(&[0x0061, 0x0308]),
        0x00E9 => Some(&[0x0065, 0x0301]),
        0x00F4 => Some(&[0x006F, 0x0302]),
        0x00F6 => Some(&[0x006F, 0x0308]),
        0x0103 => Some(&[0x0061, 0x0306]),
        0x01B0 => Some(&[0x0075, 0x031B]),
        0x0344 => Some(&[0x0308, 0x0301]),
        0x0958 => Some(&[0x0915, 0x093C]),
        0x1E0B => Some(&[0x0064, 0x0307]),
        0x1E0D => Some(&[0x0064, 0x0323]),
        0x1E63 => Some(&[0x0073, 0x0323]),
        0x1E69 => Some(&[0x0073, 0x0323, 0x0307]),
        0x1E9B => Some(&[0x017F, 0x0307]),
        0x1EA1 => Some(&[0x0061, 0x0323]),
        0x1EE5 => Some(&[0x0075, 0x0323]),
        0x1EF1 => Some(&[0x0075, 0x031B, 0x0323]),
        0x2126 => Some(&[0x03A9]),
        0x212B => Some(&[0x00C5]),
        0x2ADC => Some(&[0x2ADD, 0x0338]),
        0x017F if compatibility => Some(&[0x0073]),
        _ => None,
    };
    if let Some(decomposition) = decomposition {
        for point in decomposition {
            decompose_code_point(*point, compatibility, output);
        }
    } else {
        output.push(code_point);
    }
}

fn reorder_combining_marks(points: &mut [u32]) {
    let mut index = 1;
    while index < points.len() {
        let class = canonical_combining_class(points[index]);
        if class == 0 {
            index += 1;
            continue;
        }
        let mut scan = index;
        while scan > 0 {
            let previous = canonical_combining_class(points[scan - 1]);
            if previous == 0 || previous <= class {
                break;
            }
            points.swap(scan - 1, scan);
            scan -= 1;
        }
        index += 1;
    }
}

fn compose_hangul_pair(left: u32, right: u32) -> Option<u32> {
    const S_BASE: u32 = 0xAC00;
    const L_BASE: u32 = 0x1100;
    const V_BASE: u32 = 0x1161;
    const T_BASE: u32 = 0x11A7;
    const L_COUNT: u32 = 19;
    const V_COUNT: u32 = 21;
    const T_COUNT: u32 = 28;
    const N_COUNT: u32 = V_COUNT * T_COUNT;
    const S_COUNT: u32 = L_COUNT * N_COUNT;

    if (L_BASE..L_BASE + L_COUNT).contains(&left) && (V_BASE..V_BASE + V_COUNT).contains(&right) {
        return Some(S_BASE + (left - L_BASE) * N_COUNT + (right - V_BASE) * T_COUNT);
    }
    if (S_BASE..S_BASE + S_COUNT).contains(&left)
        && (left - S_BASE) % T_COUNT == 0
        && (T_BASE + 1..T_BASE + T_COUNT).contains(&right)
    {
        return Some(left + (right - T_BASE));
    }
    None
}

fn compose_pair(left: u32, right: u32) -> Option<u32> {
    if let Some(hangul) = compose_hangul_pair(left, right) {
        return Some(hangul);
    }
    match (left, right) {
        (0x0041, 0x030A) => Some(0x00C5),
        (0x0043, 0x0327) => Some(0x00C7),
        (0x0045, 0x0301) => Some(0x00C9),
        (0x0061, 0x0306) => Some(0x0103),
        (0x0061, 0x0308) => Some(0x00E4),
        (0x0061, 0x0323) => Some(0x1EA1),
        (0x0064, 0x0307) => Some(0x1E0B),
        (0x0064, 0x0323) => Some(0x1E0D),
        (0x0065, 0x0301) => Some(0x00E9),
        (0x006F, 0x0302) => Some(0x00F4),
        (0x006F, 0x0308) => Some(0x00F6),
        (0x0073, 0x0323) => Some(0x1E63),
        (0x0075, 0x031B) => Some(0x01B0),
        (0x0075, 0x0323) => Some(0x1EE5),
        (0x017F, 0x0307) => Some(0x1E9B),
        (0x01B0, 0x0323) => Some(0x1EF1),
        (0x1E63, 0x0307) => Some(0x1E69),
        _ => None,
    }
}

fn compose_normalized_code_points(points: &[u32]) -> Vec<u32> {
    let mut result: Vec<u32> = Vec::with_capacity(points.len());
    let mut starter_index: Option<usize> = None;
    let mut previous_class = 0;
    for point in points {
        let class = canonical_combining_class(*point);
        if let Some(starter) = starter_index {
            if previous_class == 0 || previous_class < class {
                if let Some(composed) = compose_pair(result[starter], *point) {
                    result[starter] = composed;
                    continue;
                }
            }
        }
        if class == 0 {
            starter_index = Some(result.len());
        }
        previous_class = class;
        result.push(*point);
    }
    result
}

fn code_points_to_string(points: &[u32]) -> String {
    let mut text = String::new();
    for point in points {
        if let Some(ch) = char::from_u32(*point) {
            text.push(ch);
        } else {
            text.push('\u{FFFD}');
        }
    }
    text
}

fn normalize_text_for_form(text: &str, form: &str) -> Option<String> {
    let (compatibility, compose) = match form {
        "NFC" => (false, true),
        "NFD" => (false, false),
        "NFKC" => (true, true),
        "NFKD" => (true, false),
        _ => return None,
    };
    let mut points = Vec::with_capacity(text.chars().count());
    for ch in text.chars() {
        decompose_code_point(ch as u32, compatibility, &mut points);
    }
    reorder_combining_marks(&mut points);
    let normalized = if compose {
        compose_normalized_code_points(&points)
    } else {
        points
    };
    Some(code_points_to_string(&normalized))
}

fn string_normalize_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let text = string_ref_text(cx, string)?;
    let form = if let Some(value) = invocation.arguments().first().copied() {
        if value.is_undefined() {
            "NFC".to_owned()
        } else {
            let form_ref = to_string_string_ref(cx, value)?;
            string_ref_text(cx, form_ref)?
        }
    } else {
        "NFC".to_owned()
    };
    let normalized = normalize_text_for_form(&text, &form).ok_or_else(|| range_error(cx))?;
    Ok(string_value(cx, &normalized))
}

fn expand_string_replacement_template(
    template_units: &[u16],
    source_units: &[u16],
    matched: std::ops::Range<usize>,
) -> Vec<u16> {
    let mut result = Vec::with_capacity(template_units.len());
    let mut index = 0;
    while index < template_units.len() {
        if template_units[index] != u16::from(b'$') {
            result.push(template_units[index]);
            index += 1;
            continue;
        }
        let Some(next) = template_units.get(index + 1).copied() else {
            result.push(u16::from(b'$'));
            index += 1;
            continue;
        };
        match code_unit_ascii(next).map(char::from) {
            Some('$') => {
                result.push(u16::from(b'$'));
                index += 2;
            }
            Some('&') => {
                result.extend_from_slice(&source_units[matched.clone()]);
                index += 2;
            }
            Some('`') => {
                result.extend_from_slice(&source_units[..matched.start]);
                index += 2;
            }
            Some('\'') => {
                result.extend_from_slice(&source_units[matched.end..]);
                index += 2;
            }
            _ => {
                result.push(u16::from(b'$'));
                index += 1;
            }
        }
    }
    result
}

fn string_replace_all_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let search_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let replacement = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());

    if search_value.as_object_ref().is_some() {
        if is_regexp_value(cx, search_value)? {
            let flags_key = {
                let agent = cx.agent();
                PropertyKey::from_atom(agent.bootstrap_atoms().flags())
            };
            let flags_value = cx.get_property_value(search_value, flags_key)?;
            let flags = cx.value_to_string_text(flags_value)?;
            if !flags.contains('g') {
                return Err(type_error(cx));
            }
        }
        if let Some(replacer) =
            get_method_for_well_known_symbol(cx, search_value, WellKnownSymbolId::Replace)?
        {
            return cx.call_to_completion(
                replacer,
                search_value,
                &[invocation.this_value(), replacement],
            );
        }
    }

    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let source_value = Value::from_string_ref(source_ref);
    let search_ref = to_string_string_ref(cx, search_value)?;
    let search_units = string_ref_code_units(cx, search_ref)?;
    let callable_replacement = callable_object_from_value(cx, replacement);
    let replacement_template = if callable_replacement.is_none() {
        let replacement_ref = to_string_string_ref(cx, replacement)?;
        Some(string_ref_code_units(cx, replacement_ref)?)
    } else {
        None
    };

    let mut positions = Vec::new();
    if search_units.is_empty() {
        positions.extend(0..=source_units.len());
    } else {
        let mut search_start = 0;
        while let Some(position) = find_subsequence(&source_units, &search_units, search_start) {
            positions.push(position);
            search_start = position + search_units.len();
            if search_start > source_units.len() {
                break;
            }
        }
    }

    if positions.is_empty() {
        return Ok(Value::from_string_ref(source_ref));
    }

    let mut result = Vec::with_capacity(source_units.len());
    let mut cursor = 0;
    for position in positions {
        result.extend_from_slice(&source_units[cursor..position]);
        let end = position + search_units.len();
        let replacement_units = if let Some(callee) = callable_replacement {
            let matched = string_from_code_units(cx, &source_units[position..end]);
            let arguments = [matched, usize_index_value(position), source_value];
            let replaced = cx.call_to_completion(callee, Value::undefined(), &arguments)?;
            let replaced = to_string_string_ref(cx, replaced)?;
            string_ref_code_units(cx, replaced)?
        } else {
            expand_string_replacement_template(
                replacement_template
                    .as_deref()
                    .expect("replacement template should exist for string replacements"),
                &source_units,
                position..end,
            )
        };
        result.extend_from_slice(&replacement_units);
        cursor = end;
    }
    result.extend_from_slice(&source_units[cursor..]);
    Ok(string_from_code_units(cx, &result))
}

fn regexp_match_with_string<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    regexp_object: ObjectRef,
    source_ref: StringRef,
) -> Result<Value, Cx::Error> {
    let source_units = string_ref_code_units(cx, source_ref)?;
    let source_value = Value::from_string_ref(source_ref);
    let flags = regexp_object_flags(cx, regexp_object)?;
    if flags.global() {
        let last_index_key = regexp_last_index_key(cx);
        set_data_property_value(cx, regexp_object, last_index_key, Value::from_smi(0))?;
        let mut matches = Vec::new();
        while let Some(state) = regexp_exec_state(cx, regexp_object, &source_units)? {
            matches.push(state.matched);
        }
        if matches.is_empty() {
            return Ok(Value::null());
        }
        return build_regexp_global_match_result(cx, &source_units, &matches);
    }

    let Some(state) = regexp_exec_state(cx, regexp_object, &source_units)? else {
        return Ok(Value::null());
    };
    build_regexp_match_result(cx, &source_units, source_value, &state)
}

fn string_match_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let pattern_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if pattern_value.as_object_ref().is_some() {
        if let Some(matcher) =
            get_method_for_well_known_symbol(cx, pattern_value, WellKnownSymbolId::Match)?
        {
            let source_ref = string_this_ref(cx, invocation.this_value())?;
            let source_value = Value::from_string_ref(source_ref);
            return cx.call_to_completion(matcher, pattern_value, &[source_value]);
        }
    }

    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let pattern_text = if pattern_value.is_undefined() {
        String::new()
    } else {
        cx.value_to_string_text(pattern_value)?
    };
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().regexp_prototype())
    };
    let default_prototype = default_prototype.ok_or_else(|| type_error(cx))?;
    let regexp_object = allocate_regexp_object(cx, realm, default_prototype, &pattern_text, "")?;
    let matcher = get_method_for_well_known_symbol(
        cx,
        Value::from_object_ref(regexp_object),
        WellKnownSymbolId::Match,
    )?
    .ok_or_else(|| type_error(cx))?;
    cx.call_to_completion(
        matcher,
        Value::from_object_ref(regexp_object),
        &[Value::from_string_ref(source_ref)],
    )
}

fn regexp_match_all_with_string<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    regexp_object: ObjectRef,
    source_ref: StringRef,
) -> Result<Value, Cx::Error> {
    let source_units = string_ref_code_units(cx, source_ref)?;
    let source_value = Value::from_string_ref(source_ref);
    let flags = regexp_object_flags(cx, regexp_object)?;
    if flags.global() {
        let last_index_key = regexp_last_index_key(cx);
        set_data_property_value(cx, regexp_object, last_index_key, Value::from_smi(0))?;
    }

    let mut matches = Vec::new();
    while let Some(state) = regexp_exec_state(cx, regexp_object, &source_units)? {
        matches.push(build_regexp_match_result(
            cx,
            &source_units,
            source_value,
            &state,
        )?);
        if !flags.global() {
            break;
        }
    }
    let array = create_array_from_values(cx, &matches)?;
    iterators::array_iterator_factory_builtin(
        cx,
        BuiltinInvocation::new(Value::from_object_ref(array), &[], None),
        iterators::ArrayIterationKind::Value,
    )
}

fn string_match_all_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let pattern_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if pattern_value.as_object_ref().is_some() {
        if is_regexp_value(cx, pattern_value)? {
            let flags_key = {
                let agent = cx.agent();
                PropertyKey::from_atom(agent.bootstrap_atoms().flags())
            };
            let flags_value = cx.get_property_value(pattern_value, flags_key)?;
            let flags = cx.value_to_string_text(flags_value)?;
            if !flags.contains('g') {
                return Err(type_error(cx));
            }
        }
        if let Some(matcher) =
            get_method_for_well_known_symbol(cx, pattern_value, WellKnownSymbolId::MatchAll)?
        {
            let source_ref = string_this_ref(cx, invocation.this_value())?;
            return cx.call_to_completion(
                matcher,
                pattern_value,
                &[Value::from_string_ref(source_ref)],
            );
        }
    }

    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let pattern_text = if let Some(object_ref) = pattern_value.as_object_ref() {
        if is_regexp_object(cx, object_ref)? {
            regexp_object_source_text(cx, object_ref)?
        } else {
            cx.value_to_string_text(pattern_value)?
        }
    } else if pattern_value.is_undefined() {
        String::new()
    } else {
        cx.value_to_string_text(pattern_value)?
    };
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().regexp_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let regexp_object = allocate_regexp_object(cx, realm, default_prototype, &pattern_text, "g")?;
    let matcher = get_method_for_well_known_symbol(
        cx,
        Value::from_object_ref(regexp_object),
        WellKnownSymbolId::MatchAll,
    )?
    .ok_or_else(|| type_error(cx))?;
    cx.call_to_completion(
        matcher,
        Value::from_object_ref(regexp_object),
        &[Value::from_string_ref(source_ref)],
    )
}

fn regexp_search_with_string<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    regexp_object: ObjectRef,
    source_ref: StringRef,
) -> Result<Value, Cx::Error> {
    let source_units = string_ref_code_units(cx, source_ref)?;
    let payload = {
        let agent = cx.agent();
        agent.objects().regexp_payload(regexp_object).cloned()
    }
    .ok_or_else(|| type_error(cx))?;
    Ok(payload
        .find_from_code_units(&source_units, 0)
        .map(|record| usize_index_value(record.range().start))
        .unwrap_or_else(|| Value::from_smi(-1)))
}

fn string_search_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let pattern_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if pattern_value.as_object_ref().is_some() {
        if let Some(searcher) =
            get_method_for_well_known_symbol(cx, pattern_value, WellKnownSymbolId::Search)?
        {
            let source_ref = string_this_ref(cx, invocation.this_value())?;
            return cx.call_to_completion(
                searcher,
                pattern_value,
                &[Value::from_string_ref(source_ref)],
            );
        }
    }

    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let pattern_text = if pattern_value.is_undefined() {
        String::new()
    } else {
        cx.value_to_string_text(pattern_value)?
    };
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().regexp_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let regexp_object = allocate_regexp_object(cx, realm, default_prototype, &pattern_text, "")?;
    let searcher = get_method_for_well_known_symbol(
        cx,
        Value::from_object_ref(regexp_object),
        WellKnownSymbolId::Search,
    )?
    .ok_or_else(|| type_error(cx))?;
    cx.call_to_completion(
        searcher,
        Value::from_object_ref(regexp_object),
        &[Value::from_string_ref(source_ref)],
    )
}

fn string_last_index_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let search_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let search_units = string_ref_code_units(cx, search_ref)?;

    let position = if let Some(value) = invocation.arguments().get(1).copied() {
        let number = to_number_for_builtin(cx, value)?;
        if number.is_nan() {
            f64::INFINITY
        } else if number == 0.0 {
            0.0
        } else if !number.is_finite() {
            number
        } else {
            number.trunc()
        }
    } else {
        f64::INFINITY
    };

    let source_len = source_units.len();
    let search_len = search_units.len();
    let start = if position.is_nan() || position == f64::INFINITY {
        source_len
    } else if position <= 0.0 {
        0
    } else {
        (position as usize).min(source_len)
    };

    if search_units.is_empty() {
        return Ok(Value::from_smi(i32::try_from(start).unwrap_or(i32::MAX)));
    }
    if search_len > source_len {
        return Ok(Value::from_smi(-1));
    }

    let max_index = start.min(source_len - search_len);
    for index in (0..=max_index).rev() {
        if source_units[index..index + search_len] == search_units[..] {
            return Ok(Value::from_smi(i32::try_from(index).unwrap_or(i32::MAX)));
        }
    }
    Ok(Value::from_smi(-1))
}

fn string_pad_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    at_start: bool,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, string)?;
    let max_length = to_length_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if max_length <= source_units.len() {
        return Ok(Value::from_string_ref(string));
    }

    let fill_units = if let Some(value) = invocation.arguments().get(1).copied() {
        if value.is_undefined() {
            vec![u16::from(b' ')]
        } else {
            let fill = to_string_string_ref(cx, value)?;
            string_ref_code_units(cx, fill)?
        }
    } else {
        vec![u16::from(b' ')]
    };
    if fill_units.is_empty() {
        return Ok(Value::from_string_ref(string));
    }

    let fill_len = max_length - source_units.len();
    let mut padding = Vec::with_capacity(fill_len);
    while padding.len() < fill_len {
        let remaining = fill_len - padding.len();
        if remaining >= fill_units.len() {
            padding.extend_from_slice(&fill_units);
        } else {
            padding.extend_from_slice(&fill_units[..remaining]);
        }
    }

    let mut result = Vec::with_capacity(max_length);
    if at_start {
        result.extend_from_slice(&padding);
        result.extend_from_slice(&source_units);
    } else {
        result.extend_from_slice(&source_units);
        result.extend_from_slice(&padding);
    }
    Ok(string_from_code_units(cx, &result))
}

fn string_pad_end_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    string_pad_builtin(cx, invocation, false)
}

fn string_pad_start_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    string_pad_builtin(cx, invocation, true)
}

fn string_repeat_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let count = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if count < 0.0 || !count.is_finite() {
        return Err(range_error(cx));
    }

    let repeat_count = count as usize;
    if repeat_count == 0 || units.is_empty() {
        return Ok(string_from_code_units(cx, &[]));
    }
    let result_len = units
        .len()
        .checked_mul(repeat_count)
        .ok_or_else(|| range_error(cx))?;
    if u32::try_from(result_len).is_err() {
        return Err(range_error(cx));
    }

    let mut result = Vec::with_capacity(result_len);
    for _ in 0..repeat_count {
        result.extend_from_slice(&units);
    }
    Ok(string_from_code_units(cx, &result))
}

fn regexp_replace_with_string<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    regexp_object: ObjectRef,
    source_ref: StringRef,
    replacement: Value,
) -> Result<Value, Cx::Error> {
    let source_units = string_ref_code_units(cx, source_ref)?;
    let source_value = Value::from_string_ref(source_ref);
    let callable_replacement = callable_object_from_value(cx, replacement);
    let flags = regexp_object_flags(cx, regexp_object)?;
    if flags.global() {
        let last_index_key = regexp_last_index_key(cx);
        set_data_property_value(cx, regexp_object, last_index_key, Value::from_smi(0))?;
    }

    let replacement_template_units = if callable_replacement.is_none() {
        let replacement_ref = to_string_string_ref(cx, replacement)?;
        Some(string_ref_code_units(cx, replacement_ref)?)
    } else {
        None
    };

    let mut matches = Vec::new();
    while let Some(state) = regexp_exec_state(cx, regexp_object, &source_units)? {
        matches.push(state);
        if !flags.global() {
            break;
        }
    }
    if matches.is_empty() {
        return Ok(Value::from_string_ref(source_ref));
    }

    let mut result = Vec::with_capacity(source_units.len());
    let mut cursor = 0;
    for state in matches {
        let matched = &state.matched;
        result.extend_from_slice(&source_units[cursor..matched.start()]);
        let replacement_units = if let Some(callee) = callable_replacement {
            let mut arguments = Vec::with_capacity(matched.captures().len() + 4);
            arguments.push(code_unit_range_value(cx, &source_units, matched.range()));
            for capture in matched.captures() {
                let value = capture
                    .clone()
                    .map(|range| code_unit_range_value(cx, &source_units, range))
                    .unwrap_or(Value::undefined());
                arguments.push(value);
            }
            arguments.push(usize_index_value(matched.start()));
            arguments.push(source_value);
            if let Some(groups) =
                allocate_named_capture_object(cx, matched.named_captures(), &source_units, false)?
            {
                arguments.push(groups);
            }
            let replaced = cx.call_to_completion(callee, Value::undefined(), &arguments)?;
            let replaced_ref = to_string_string_ref(cx, replaced)?;
            string_ref_code_units(cx, replaced_ref)?
        } else {
            expand_regexp_replacement_template(
                replacement_template_units
                    .as_deref()
                    .expect("template units should exist for non-callable replacements"),
                &source_units,
                &state,
            )
        };
        result.extend_from_slice(&replacement_units);
        cursor = matched.end();
    }
    result.extend_from_slice(&source_units[cursor..]);
    Ok(string_from_code_units(cx, &result))
}

fn string_replace_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let search_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let replacement = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    if search_value.as_object_ref().is_some() {
        if let Some(replacer) =
            get_method_for_well_known_symbol(cx, search_value, WellKnownSymbolId::Replace)?
        {
            return cx.call_to_completion(
                replacer,
                search_value,
                &[invocation.this_value(), replacement],
            );
        }
    }

    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let source_value = Value::from_string_ref(source_ref);
    let callable_replacement = callable_object_from_value(cx, replacement);
    let pattern_ref = to_string_string_ref(cx, search_value)?;
    let pattern_units = string_ref_code_units(cx, pattern_ref)?;
    let replacement_template = if callable_replacement.is_none() {
        let replacement_ref = to_string_string_ref(cx, replacement)?;
        Some(string_ref_code_units(cx, replacement_ref)?)
    } else {
        None
    };

    let Some(start) = find_subsequence(&source_units, &pattern_units, 0) else {
        return Ok(Value::from_string_ref(source_ref));
    };
    let end = start + pattern_units.len();
    let replacement_units = if let Some(callee) = callable_replacement {
        let matched_value = string_from_code_units(cx, &source_units[start..end]);
        let arguments = [matched_value, usize_index_value(start), source_value];
        let replaced = cx.call_to_completion(callee, Value::undefined(), &arguments)?;
        let replaced_ref = to_string_string_ref(cx, replaced)?;
        string_ref_code_units(cx, replaced_ref)?
    } else {
        expand_string_replacement_template(
            replacement_template
                .as_deref()
                .expect("template units should exist for non-callable string replacement"),
            &source_units,
            start..end,
        )
    };

    let mut result = Vec::with_capacity(source_units.len() + replacement_units.len());
    result.extend_from_slice(&source_units[..start]);
    result.extend_from_slice(&replacement_units);
    result.extend_from_slice(&source_units[end..]);
    Ok(string_from_code_units(cx, &result))
}

fn string_split_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let separator_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let limit_value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    if separator_value.as_object_ref().is_some() {
        if let Some(splitter) =
            get_method_for_well_known_symbol(cx, separator_value, WellKnownSymbolId::Split)?
        {
            return cx.call_to_completion(
                splitter,
                separator_value,
                &[invocation.this_value(), limit_value],
            );
        }
    }

    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let limit = if let Some(value) = invocation.arguments().get(1).copied() {
        if value.is_undefined() {
            u32::MAX
        } else {
            to_uint32_for_builtin(cx, value)?
        }
    } else {
        u32::MAX
    };
    let separator_units = if separator_value.is_undefined() {
        None
    } else {
        let separator_ref = to_string_string_ref(cx, separator_value)?;
        Some(string_ref_code_units(cx, separator_ref)?)
    };

    let mut parts: Vec<Vec<u16>> = Vec::new();
    if limit == 0 {
        return Ok(Value::from_object_ref(allocate_array_like_result(cx, 0)?));
    }

    match separator_units {
        None => parts.push(source_units.clone()),
        Some(ref separator) if separator.is_empty() => {
            for unit in &source_units {
                if parts.len() >= usize::try_from(limit).unwrap_or(usize::MAX) {
                    break;
                }
                parts.push(vec![*unit]);
            }
        }
        Some(separator) => {
            let mut start = 0;
            loop {
                if parts.len() >= usize::try_from(limit).unwrap_or(usize::MAX) {
                    break;
                }
                let Some(index) = find_subsequence(&source_units, &separator, start) else {
                    parts.push(source_units[start..].to_vec());
                    break;
                };
                parts.push(source_units[start..index].to_vec());
                start = index + separator.len();
                if start > source_units.len() {
                    if parts.len() < usize::try_from(limit).unwrap_or(usize::MAX) {
                        parts.push(Vec::new());
                    }
                    break;
                }
            }
        }
    }

    let object = allocate_array_like_result(cx, u32::try_from(parts.len()).unwrap_or(u32::MAX))?;
    for (index, part) in parts.iter().enumerate() {
        let part_value = string_from_code_units(cx, part);
        define_data_property_with_attrs(
            cx,
            object,
            PropertyKey::Index(u32::try_from(index).unwrap_or(u32::MAX)),
            part_value,
            true,
            true,
            true,
        )?;
    }
    Ok(Value::from_object_ref(object))
}

fn string_slice_index(value: f64, length: usize) -> usize {
    if value.is_nan() || value == f64::NEG_INFINITY {
        return 0;
    }
    if value < 0.0 {
        let offset = (-value).min(length as f64) as usize;
        return length.saturating_sub(offset);
    }
    if !value.is_finite() {
        return length;
    }
    (value as usize).min(length)
}

fn string_slice_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let length = units.len();
    let start = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let end = if let Some(value) = invocation.arguments().get(1).copied() {
        if value.is_undefined() {
            length as f64
        } else {
            to_integer_or_infinity_for_builtin(cx, value)?
        }
    } else {
        length as f64
    };
    let from = string_slice_index(start, length);
    let to = string_slice_index(end, length);
    if to <= from {
        return Ok(string_from_code_units(cx, &[]));
    }
    Ok(string_from_code_units(cx, &units[from..to]))
}

fn string_substring_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let length = units.len();
    let start = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let end = if let Some(value) = invocation.arguments().get(1).copied() {
        if value.is_undefined() {
            length as f64
        } else {
            to_integer_or_infinity_for_builtin(cx, value)?
        }
    } else {
        length as f64
    };

    let clamp = |value: f64| -> usize {
        if value.is_nan() || value <= 0.0 {
            0
        } else if !value.is_finite() {
            length
        } else {
            (value as usize).min(length)
        }
    };

    let start_index = clamp(start);
    let end_index = clamp(end);
    let (from, to) = if start_index <= end_index {
        (start_index, end_index)
    } else {
        (end_index, start_index)
    };
    Ok(string_from_code_units(cx, &units[from..to]))
}

fn string_starts_with_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let search_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if regexp_search_value_is_rejected(cx, search_value)? {
        return Err(type_error(cx));
    }
    let search_ref = to_string_string_ref(cx, search_value)?;
    let search_units = string_ref_code_units(cx, search_ref)?;
    let position = if let Some(value) = invocation.arguments().get(1).copied() {
        to_integer_or_infinity_for_builtin(cx, value)?
    } else {
        0.0
    };
    let start = if position.is_nan() || position <= 0.0 {
        0
    } else if !position.is_finite() {
        source_units.len()
    } else {
        (position as usize).min(source_units.len())
    };
    let end = start.saturating_add(search_units.len());
    let matches = end <= source_units.len() && source_units[start..end] == search_units[..];
    Ok(Value::from_bool(matches))
}

fn normalize_regexp_constructor_pattern_text(pattern: &str) -> String {
    let mut normalized = String::with_capacity(pattern.len());
    for ch in pattern.chars() {
        match ch {
            '\n' => normalized.push_str("\\n"),
            '\r' => normalized.push_str("\\r"),
            '\u{2028}' => normalized.push_str("\\u2028"),
            '\u{2029}' => normalized.push_str("\\u2029"),
            _ => normalized.push(ch),
        }
    }
    normalized
}

fn regexp_escape_push_hex(output: &mut String, unit: u16) {
    let _ = write!(output, "\\x{unit:02x}");
}

fn regexp_escape_push_unicode(output: &mut String, unit: u16) {
    let _ = write!(output, "\\u{unit:04x}");
}

fn regexp_escape_is_ascii_letter_or_digit(unit: u16) -> bool {
    (u16::from(b'0')..=u16::from(b'9')).contains(&unit)
        || (u16::from(b'A')..=u16::from(b'Z')).contains(&unit)
        || (u16::from(b'a')..=u16::from(b'z')).contains(&unit)
}

fn regexp_escape_is_other_punctuator(unit: u16) -> bool {
    [
        u16::from(b','),
        u16::from(b'-'),
        u16::from(b'='),
        u16::from(b'<'),
        u16::from(b'>'),
        u16::from(b'#'),
        u16::from(b'&'),
        u16::from(b'!'),
        u16::from(b'%'),
        u16::from(b':'),
        u16::from(b';'),
        u16::from(b'@'),
        u16::from(b'~'),
        u16::from(b'\''),
        u16::from(b'`'),
        u16::from(b'"'),
    ]
    .contains(&unit)
}

fn regexp_escape_is_syntax_character(unit: u16) -> bool {
    [
        u16::from(b'^'),
        u16::from(b'$'),
        u16::from(b'\\'),
        u16::from(b'.'),
        u16::from(b'*'),
        u16::from(b'+'),
        u16::from(b'?'),
        u16::from(b'('),
        u16::from(b')'),
        u16::from(b'['),
        u16::from(b']'),
        u16::from(b'{'),
        u16::from(b'}'),
        u16::from(b'|'),
        u16::from(b'/'),
    ]
    .contains(&unit)
}

fn regexp_escape_is_whitespace_or_line_terminator(unit: u16) -> bool {
    matches!(
        unit,
        0x0009
            | 0x000A
            | 0x000B
            | 0x000C
            | 0x000D
            | 0x0020
            | 0x00A0
            | 0x1680
            | 0x2000
            | 0x2001
            | 0x2002
            | 0x2003
            | 0x2004
            | 0x2005
            | 0x2006
            | 0x2007
            | 0x2008
            | 0x2009
            | 0x200A
            | 0x2028
            | 0x2029
            | 0x202F
            | 0x205F
            | 0x3000
            | 0xFEFF
    )
}

fn regexp_escape_append_encoded_unit(output: &mut String, unit: u16) {
    match unit {
        0x0009 => output.push_str("\\t"),
        0x000A => output.push_str("\\n"),
        0x000B => output.push_str("\\v"),
        0x000C => output.push_str("\\f"),
        0x000D => output.push_str("\\r"),
        _ if regexp_escape_is_syntax_character(unit) => {
            output.push('\\');
            output.push(char::from(
                u8::try_from(unit).expect("syntax characters stay ASCII"),
            ));
        }
        _ if regexp_escape_is_other_punctuator(unit)
            || regexp_escape_is_whitespace_or_line_terminator(unit) =>
        {
            if unit <= 0x00FF {
                regexp_escape_push_hex(output, unit);
            } else {
                regexp_escape_push_unicode(output, unit);
            }
        }
        _ if (0xD800..=0xDFFF).contains(&unit) => regexp_escape_push_unicode(output, unit),
        _ => {
            output.push(
                char::from_u32(u32::from(unit))
                    .expect("non-surrogate UTF-16 code unit should convert to Unicode scalar"),
            );
        }
    }
}

fn regexp_escape_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let input = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let string_ref = input.as_string_ref().ok_or_else(|| type_error(cx))?;
    let units = string_ref_code_units(cx, string_ref)?;
    let mut escaped = String::with_capacity(units.len() * 2);
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if index == 0 && regexp_escape_is_ascii_letter_or_digit(unit) {
            regexp_escape_push_hex(&mut escaped, unit);
            index += 1;
            continue;
        }
        if (0xD800..=0xDBFF).contains(&unit)
            && matches!(units.get(index + 1), Some(next) if (0xDC00..=0xDFFF).contains(next))
        {
            let high = u32::from(unit - 0xD800);
            let low = u32::from(units[index + 1] - 0xDC00);
            let code_point = 0x1_0000 + ((high << 10) | low);
            escaped.push(
                char::from_u32(code_point)
                    .expect("valid surrogate pair should convert to Unicode scalar"),
            );
            index += 2;
            continue;
        }
        regexp_escape_append_encoded_unit(&mut escaped, unit);
        index += 1;
    }
    Ok(string_value(cx, &escaped))
}

fn regexp_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().regexp_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let pattern_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let flags_value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());

    let pattern_regexp = match pattern_value.as_object_ref() {
        Some(object_ref) if is_regexp_object(cx, object_ref)? => Some(object_ref),
        _ => None,
    };

    if let Some(object_ref) = pattern_regexp {
        if flags_value.is_undefined() && invocation.new_target().is_none() {
            let active_constructor = {
                let agent = cx.agent();
                agent
                    .realm(realm)
                    .and_then(|record| record.intrinsics().regexp())
            };
            let constructor = cx.get_property_value(
                Value::from_object_ref(object_ref),
                PropertyKey::from_atom(WellKnownAtom::constructor.id()),
            )?;
            if constructor.as_object_ref() == active_constructor {
                return Ok(Value::from_object_ref(object_ref));
            }
        }
    }

    let pattern_text = if let Some(object_ref) = pattern_regexp {
        regexp_object_source_text(cx, object_ref)?
    } else if pattern_value.is_undefined() {
        String::new()
    } else {
        normalize_regexp_constructor_pattern_text(&cx.value_to_string_text(pattern_value)?)
    };
    let flags_text = if flags_value.is_undefined() {
        if let Some(object_ref) = pattern_regexp {
            regexp_object_flag_text(cx, object_ref)?
        } else {
            String::new()
        }
    } else {
        cx.value_to_string_text(flags_value)?
    };
    if validate_regexp_literal(&pattern_text, &flags_text).is_err() {
        return Err(syntax_error(cx));
    }

    let prototype = if invocation.new_target().is_some() {
        cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?
    } else {
        default_prototype
    };
    let regexp = allocate_regexp_object(cx, realm, prototype, &pattern_text, &flags_text)?;
    Ok(Value::from_object_ref(regexp))
}

fn regexp_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let receiver = invocation.this_value();
    if receiver.as_object_ref().is_none() {
        return Err(type_error(cx));
    }
    let (source_key, flags_key) = {
        let agent = cx.agent();
        (
            PropertyKey::from_atom(agent.bootstrap_atoms().source()),
            PropertyKey::from_atom(agent.bootstrap_atoms().flags()),
        )
    };
    let source_value = cx.get_property_value(receiver, source_key)?;
    let source = cx.value_to_string_text(source_value)?;
    let flags_value = cx.get_property_value(receiver, flags_key)?;
    let flags = cx.value_to_string_text(flags_value)?;
    Ok(string_value(cx, &format!("/{source}/{flags}")))
}

fn regexp_exec_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = regexp_matcher_this_object(cx, invocation.this_value())?;
    let input_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let input_units = string_ref_code_units(cx, input_ref)?;
    let input_value = Value::from_string_ref(input_ref);
    let Some(state) = regexp_exec_state(cx, object_ref, &input_units)? else {
        return Ok(Value::null());
    };
    build_regexp_match_result(cx, &input_units, input_value, &state)
}

fn regexp_test_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let matched = regexp_exec_builtin(cx, invocation)?;
    Ok(Value::from_bool(!matched.is_null()))
}

fn regexp_symbol_match_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = regexp_matcher_this_object(cx, invocation.this_value())?;
    let input_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    regexp_match_with_string(cx, object_ref, input_ref)
}

fn regexp_symbol_replace_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = regexp_matcher_this_object(cx, invocation.this_value())?;
    let input_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let replacement = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    regexp_replace_with_string(cx, object_ref, input_ref, replacement)
}

fn regexp_symbol_search_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = regexp_matcher_this_object(cx, invocation.this_value())?;
    let input_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    regexp_search_with_string(cx, object_ref, input_ref)
}

fn regexp_symbol_split_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = regexp_matcher_this_object(cx, invocation.this_value())?;
    let input_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let limit = if let Some(value) = invocation.arguments().get(1).copied() {
        if value.is_undefined() {
            u32::MAX
        } else {
            to_uint32_for_builtin(cx, value)?
        }
    } else {
        u32::MAX
    };
    if limit == 0 {
        return Ok(Value::from_object_ref(allocate_array_like_result(cx, 0)?));
    }

    let source_units = string_ref_code_units(cx, input_ref)?;
    let flags = regexp_object_flags(cx, object_ref)?;
    let payload = {
        let agent = cx.agent();
        agent.objects().regexp_payload(object_ref).cloned()
    }
    .ok_or_else(|| type_error(cx))?;

    if payload.source().is_empty() {
        let part_count = source_units
            .len()
            .min(usize::try_from(limit).unwrap_or(usize::MAX));
        let object = allocate_array_like_result(cx, u32::try_from(part_count).unwrap_or(u32::MAX))?;
        for index in 0..part_count {
            let value = code_unit_range_value(cx, &source_units, index..index + 1);
            define_data_property_with_attrs(
                cx,
                object,
                PropertyKey::Index(u32::try_from(index).unwrap_or(u32::MAX)),
                value,
                true,
                true,
                true,
            )?;
        }
        return Ok(Value::from_object_ref(object));
    }

    let mut parts = Vec::new();
    let mut last_end = 0;
    let mut search_start = 0;
    let mut suppress_trailing_empty = false;
    let limit_len = usize::try_from(limit).unwrap_or(usize::MAX);
    while search_start <= source_units.len() && parts.len() < limit_len {
        let Some(matched) = payload.find_from_code_units(&source_units, search_start) else {
            break;
        };
        if matched.start() < last_end {
            search_start = advance_string_index(&source_units, search_start, flags.unicode_aware());
            continue;
        }
        if matched.start() == matched.end() && matched.start() == search_start {
            search_start = advance_string_index(&source_units, search_start, flags.unicode_aware());
            continue;
        }

        parts.push(Some(last_end..matched.start()));
        if parts.len() >= limit_len {
            break;
        }
        for capture in matched.captures() {
            parts.push(capture.clone());
            if parts.len() >= limit_len {
                break;
            }
        }
        last_end = matched.end();
        search_start = matched.end();
        suppress_trailing_empty =
            matched.start() == matched.end() && matched.end() == source_units.len();
    }
    if parts.len() < limit_len && !suppress_trailing_empty {
        parts.push(Some(last_end..source_units.len()));
    }

    let object = allocate_array_like_result(cx, u32::try_from(parts.len()).unwrap_or(u32::MAX))?;
    for (index, part) in parts.into_iter().enumerate() {
        let value = part
            .map(|range| code_unit_range_value(cx, &source_units, range))
            .unwrap_or(Value::undefined());
        define_data_property_with_attrs(
            cx,
            object,
            PropertyKey::Index(u32::try_from(index).unwrap_or(u32::MAX)),
            value,
            true,
            true,
            true,
        )?;
    }
    Ok(Value::from_object_ref(object))
}

fn regexp_symbol_match_all_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = regexp_matcher_this_object(cx, invocation.this_value())?;
    let input_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    regexp_match_all_with_string(cx, object_ref, input_ref)
}

fn regexp_flag_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    flag: char,
) -> Result<Value, Cx::Error> {
    let object_ref = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let flags = {
        let agent = cx.agent();
        agent
            .objects()
            .regexp_payload(object_ref)
            .map(|payload| payload.flags())
    };
    let Some(flags) = flags else {
        if current_intrinsic_regexp_prototype(cx) == Some(object_ref) {
            return Ok(Value::undefined());
        }
        return Err(type_error(cx));
    };
    let value = match flag {
        'd' => flags.has_indices(),
        'g' => flags.global(),
        'i' => flags.ignore_case(),
        'm' => flags.multiline(),
        's' => flags.dot_all(),
        'u' => flags.unicode(),
        'v' => flags.unicode_sets(),
        'y' => flags.sticky(),
        _ => false,
    };
    Ok(Value::from_bool(value))
}

enum RegExpSource {
    Text(String),
    Units(Vec<u16>),
}

fn regexp_source_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let source = {
        let agent = cx.agent();
        agent.objects().regexp_payload(object_ref).map(|payload| {
            if let Some(units) = payload.source_units() {
                if units.is_empty() {
                    RegExpSource::Text("(?:)".to_owned())
                } else {
                    RegExpSource::Units(units.to_vec())
                }
            } else if payload.source().is_empty() {
                RegExpSource::Text("(?:)".to_owned())
            } else {
                RegExpSource::Text(payload.source().to_owned())
            }
        })
    };
    if let Some(source) = source {
        return Ok(match source {
            RegExpSource::Text(source) => string_value(cx, &source),
            RegExpSource::Units(units) => string_from_code_units(cx, &units),
        });
    }
    if current_intrinsic_regexp_prototype(cx) == Some(object_ref) {
        return Ok(string_value(cx, "(?:)"));
    }
    Err(type_error(cx))
}

fn regexp_flags_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let receiver = Value::from_object_ref(
        invocation
            .this_value()
            .as_object_ref()
            .ok_or_else(|| type_error(cx))?,
    );
    let (
        global_key,
        ignore_case_key,
        multiline_key,
        dot_all_key,
        unicode_key,
        unicode_sets_key,
        sticky_key,
        has_indices_key,
    ) = {
        let agent = cx.agent();
        (
            PropertyKey::from_atom(agent.atoms_mut().intern_collectible("global")),
            PropertyKey::from_atom(agent.atoms_mut().intern_collectible("ignoreCase")),
            PropertyKey::from_atom(agent.atoms_mut().intern_collectible("multiline")),
            PropertyKey::from_atom(agent.atoms_mut().intern_collectible("dotAll")),
            PropertyKey::from_atom(agent.atoms_mut().intern_collectible("unicode")),
            PropertyKey::from_atom(agent.atoms_mut().intern_collectible("unicodeSets")),
            PropertyKey::from_atom(agent.atoms_mut().intern_collectible("sticky")),
            PropertyKey::from_atom(agent.bootstrap_atoms().has_indices()),
        )
    };
    let mut flags = String::with_capacity(8);
    let has_indices = boolean_property_value(cx, receiver, has_indices_key)?;
    if has_indices {
        flags.push('d');
    }
    let global = boolean_property_value(cx, receiver, global_key)?;
    if global {
        flags.push('g');
    }
    let ignore_case = boolean_property_value(cx, receiver, ignore_case_key)?;
    if ignore_case {
        flags.push('i');
    }
    let multiline = boolean_property_value(cx, receiver, multiline_key)?;
    if multiline {
        flags.push('m');
    }
    let dot_all = boolean_property_value(cx, receiver, dot_all_key)?;
    if dot_all {
        flags.push('s');
    }
    let unicode = boolean_property_value(cx, receiver, unicode_key)?;
    if unicode {
        flags.push('u');
    }
    let unicode_sets = if let Some(object_ref) = receiver.as_object_ref() {
        let payload_flags = {
            let agent = cx.agent();
            agent
                .objects()
                .regexp_payload(object_ref)
                .map(|payload| payload.flags())
        };
        if let Some(payload_flags) = payload_flags {
            payload_flags.unicode_sets()
        } else {
            boolean_property_value(cx, receiver, unicode_sets_key)?
        }
    } else {
        boolean_property_value(cx, receiver, unicode_sets_key)?
    };
    if unicode_sets {
        flags.push('v');
    }
    let sticky = boolean_property_value(cx, receiver, sticky_key)?;
    if sticky {
        flags.push('y');
    }
    Ok(string_value(cx, &flags))
}

fn regexp_has_indices_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    regexp_flag_getter_builtin(cx, invocation, 'd')
}

fn primitive_wrapper_constructor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: ObjectRef,
    wrapper_kind: PrimitiveWrapperKind,
    value: Value,
) -> Result<Value, Cx::Error> {
    let root_shape = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.root_shape())
        .ok_or_else(|| type_error(cx))?;
    let wrapper = {
        let agent = cx.agent();
        object::allocate_primitive_wrapper_object(
            agent,
            root_shape,
            Some(prototype),
            wrapper_kind,
            value,
            AllocationLifetime::Default,
        )
    };
    Ok(Value::from_object_ref(map_completion(cx, wrapper)?))
}

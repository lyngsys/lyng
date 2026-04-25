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
use lyng_js_env::Agent;
use lyng_js_gc::{AllocationLifetime, StringEncoding};
use lyng_js_host::{
    TemporalCivilTime, TemporalCivilToInstantRequest, TemporalCurrentInstantRequest,
    TemporalDefaultTimeZone, TemporalDefaultTimeZoneRequest, TemporalInstant,
    TemporalInstantToCivilRequest, TemporalInstantWithOffset,
};
use lyng_js_objects::{
    FunctionEntryIdentity, ObjectAllocation, ObjectColdData, ObjectFlags, ObjectKind,
    OrdinaryObjectData, PrimitiveWrapperKind, ProxyObjectData,
};
use lyng_js_ops::{errors, iterator, object, proxy, read};
use lyng_js_types::{
    AbruptCompletion, BuiltinFunctionId, ObjectRef, PropertyDescriptor, PropertyKey, RealmRef,
    StringRef, Value, WellKnownSymbolId,
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
            let bits =
                binary_data::typed_array_storage_bits_from_builtin_value(cx, record.kind(), value)?;
            if cx
                .agent()
                .backing_store_is_detached(record.backing_store())
                .ok_or_else(|| type_error(cx))?
            {
                return Err(type_error(cx));
            }
            binary_data::typed_array_write_storage_bits(cx, record, element_index, bits)?;
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

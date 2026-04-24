mod temporal;
use crate::internal::{dispatch_internal_builtin, InternalBuiltinDispatchContext};
use crate::{BuiltinInvocation, DynamicFunctionKind};
use lyng_js_common::WellKnownAtom;
use lyng_js_env::{
    Agent, AsyncWaiterRecord, ParkedAgentRecord, PromiseCombinatorElementKind,
    PromiseCombinatorElementRecord, PromiseCombinatorKind, PromiseFinallyFunctionKind,
    PromiseFinallyFunctionRecord, PromiseReactionHandler, PromiseReactionKind,
    PromiseResolvingFunctionKind, PromiseState, WaiterKind,
};
use lyng_js_gc::{AllocationLifetime, BigIntSign, StringEncoding, SymbolFlags, WeakHeapRef};
use lyng_js_host::{
    ParkAgentRequest, ParkAgentStatus, TemporalCivilDateTime, TemporalCivilTime,
    TemporalCivilToInstantRequest, TemporalCurrentInstantRequest, TemporalDefaultTimeZone,
    TemporalDefaultTimeZoneRequest, TemporalDisambiguation, TemporalInstant,
    TemporalInstantToCivilRequest, TemporalInstantWithOffset, UnparkAgentRequest,
};
use lyng_js_objects::{
    ArrayBufferObjectData, DataViewObjectData, FunctionEntryIdentity, MapEntry, MapObjectData,
    ObjectAllocation, ObjectColdData, ObjectKind, OrdinaryObjectData, PrimitiveWrapperKind,
    ProxyObjectData, SetObjectData, TypedArrayElementKind, TypedArrayObjectData,
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
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    js3_aggregate_error_builtin, js3_array_buffer_builtin,
    js3_array_buffer_byte_length_getter_builtin, js3_array_buffer_is_view_builtin,
    js3_array_buffer_slice_builtin, js3_array_builtin, js3_array_concat_builtin,
    js3_array_copy_within_builtin, js3_array_entries_builtin, js3_array_fill_builtin,
    js3_array_filter_builtin, js3_array_for_each_builtin, js3_array_from_builtin,
    js3_array_is_array_builtin, js3_array_iterator_next_builtin, js3_array_join_builtin,
    js3_array_keys_builtin, js3_array_map_builtin, js3_array_reverse_builtin,
    js3_array_shift_builtin, js3_array_slice_builtin, js3_array_sort_builtin,
    js3_array_species_getter_builtin, js3_array_splice_builtin, js3_array_to_locale_string_builtin,
    js3_array_to_string_builtin, js3_array_unshift_builtin, js3_array_values_builtin,
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
    js3_function_to_string_builtin, js3_generator_function_builtin, js3_generator_next_builtin,
    js3_generator_return_builtin, js3_generator_throw_builtin, js3_int16_array_builtin,
    js3_int32_array_builtin, js3_int8_array_builtin, js3_is_finite_builtin, js3_is_nan_builtin,
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
    js3_object_builtin, js3_object_create_builtin, js3_object_define_properties_builtin,
    js3_object_define_property_builtin, js3_object_entries_builtin, js3_object_freeze_builtin,
    js3_object_get_own_property_descriptor_builtin,
    js3_object_get_own_property_descriptors_builtin, js3_object_get_own_property_names_builtin,
    js3_object_get_own_property_symbols_builtin, js3_object_get_prototype_of_builtin,
    js3_object_has_own_builtin, js3_object_has_own_property_builtin, js3_object_is_builtin,
    js3_object_is_extensible_builtin, js3_object_is_frozen_builtin,
    js3_object_is_prototype_of_builtin, js3_object_is_sealed_builtin, js3_object_keys_builtin,
    js3_object_prevent_extensions_builtin, js3_object_property_is_enumerable_builtin,
    js3_object_seal_builtin, js3_object_set_prototype_of_builtin,
    js3_object_to_locale_string_builtin, js3_object_to_string_builtin, js3_object_value_of_builtin,
    js3_object_values_builtin, js3_parse_float_builtin, js3_parse_int_builtin,
    js3_promise_all_builtin, js3_promise_all_resolve_element_builtin,
    js3_promise_all_settled_builtin, js3_promise_all_settled_reject_element_builtin,
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
    js3_regexp_sticky_getter_builtin, js3_regexp_test_builtin, js3_regexp_to_string_builtin,
    js3_regexp_unicode_getter_builtin, js3_set_add_builtin, js3_set_builtin, js3_set_clear_builtin,
    js3_set_delete_builtin, js3_set_entries_builtin, js3_set_for_each_builtin, js3_set_has_builtin,
    js3_set_iterator_next_builtin, js3_set_keys_builtin, js3_set_size_getter_builtin,
    js3_set_values_builtin, js3_shared_array_buffer_builtin,
    js3_shared_array_buffer_byte_length_getter_builtin, js3_shared_array_buffer_slice_builtin,
    js3_string_builtin, js3_string_char_at_builtin, js3_string_char_code_at_builtin,
    js3_string_concat_builtin, js3_string_from_char_code_builtin, js3_string_iterator_builtin,
    js3_string_iterator_next_builtin, js3_string_last_index_of_builtin, js3_string_match_builtin,
    js3_string_pad_end_builtin, js3_string_pad_start_builtin, js3_string_repeat_builtin,
    js3_string_replace_builtin, js3_string_search_builtin, js3_string_slice_builtin,
    js3_string_split_builtin, js3_string_starts_with_builtin, js3_string_substring_builtin,
    js3_string_to_string_builtin, js3_string_value_of_builtin, js3_symbol_builtin,
    js3_symbol_description_getter_builtin, js3_symbol_for_builtin, js3_symbol_key_for_builtin,
    js3_symbol_to_primitive_builtin, js3_symbol_to_string_builtin, js3_symbol_value_of_builtin,
    js3_syntax_error_builtin, js3_type_error_builtin, js3_typed_array_at_builtin,
    js3_typed_array_builtin, js3_typed_array_copy_within_builtin, js3_typed_array_every_builtin,
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
    if entry == js3_object_builtin() {
        return object_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_create_builtin() {
        return object_create_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_get_prototype_of_builtin() {
        return object_get_prototype_of_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_set_prototype_of_builtin() {
        return object_set_prototype_of_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_get_own_property_descriptor_builtin() {
        return object_get_own_property_descriptor_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_get_own_property_descriptors_builtin() {
        return object_get_own_property_descriptors_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_get_own_property_names_builtin() {
        return object_get_own_property_names_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_get_own_property_symbols_builtin() {
        return object_get_own_property_symbols_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_define_properties_builtin() {
        return object_define_properties_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_define_property_builtin() {
        return object_define_property_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_prevent_extensions_builtin() {
        return object_prevent_extensions_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_is_extensible_builtin() {
        return object_is_extensible_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_is_builtin() {
        return object_is_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_seal_builtin() {
        return object_seal_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_freeze_builtin() {
        return object_freeze_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_is_sealed_builtin() {
        return object_is_sealed_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_is_frozen_builtin() {
        return object_is_frozen_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_to_locale_string_builtin() {
        return object_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_to_string_builtin() {
        return object_to_string_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_value_of_builtin() {
        return object_value_of_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_has_own_property_builtin() {
        return object_has_own_property_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_is_prototype_of_builtin() {
        return object_is_prototype_of_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_property_is_enumerable_builtin() {
        return object_property_is_enumerable_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_keys_builtin() {
        return object_keys_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_entries_builtin() {
        return object_entries_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_values_builtin() {
        return object_values_builtin(context, invocation).map(Some);
    }
    if entry == js3_object_has_own_builtin() {
        return object_has_own_builtin(context, invocation).map(Some);
    }
    if entry == js3_function_builtin() {
        return function_builtin(context, invocation).map(Some);
    }
    if entry == js3_function_prototype_builtin() {
        return function_prototype_builtin(context, invocation).map(Some);
    }
    if entry == js3_function_call_builtin() {
        return function_call_builtin(context, invocation).map(Some);
    }
    if entry == js3_function_apply_builtin() {
        return function_apply_builtin(context, invocation).map(Some);
    }
    if entry == js3_function_bind_builtin() {
        return function_bind_builtin(context, invocation).map(Some);
    }
    if entry == js3_function_to_string_builtin() {
        return function_to_string_builtin(context, invocation).map(Some);
    }
    if entry == js3_async_function_builtin() {
        return async_function_builtin(context, invocation).map(Some);
    }
    if entry == js3_async_generator_function_builtin() {
        return async_generator_function_builtin(context, invocation).map(Some);
    }
    if entry == js3_async_generator_next_builtin() {
        return async_generator_next_builtin(context, invocation).map(Some);
    }
    if entry == js3_async_generator_return_builtin() {
        return async_generator_return_builtin(context, invocation).map(Some);
    }
    if entry == js3_async_generator_throw_builtin() {
        return async_generator_throw_builtin(context, invocation).map(Some);
    }
    if entry == js3_generator_function_builtin() {
        return generator_function_builtin(context, invocation).map(Some);
    }
    if entry == js3_generator_next_builtin() {
        return generator_next_builtin(context, invocation).map(Some);
    }
    if entry == js3_generator_return_builtin() {
        return generator_return_builtin(context, invocation).map(Some);
    }
    if entry == js3_generator_throw_builtin() {
        return generator_throw_builtin(context, invocation).map(Some);
    }
    if entry == js3_map_builtin() {
        return map_builtin(context, invocation).map(Some);
    }
    if entry == js3_set_builtin() {
        return set_builtin(context, invocation).map(Some);
    }
    if entry == js3_weak_map_builtin() {
        return weak_map_builtin(context, invocation).map(Some);
    }
    if entry == js3_weak_set_builtin() {
        return weak_set_builtin(context, invocation).map(Some);
    }
    if entry == js3_weak_ref_builtin() {
        return weak_ref_builtin(context, invocation).map(Some);
    }
    if entry == js3_finalization_registry_builtin() {
        return finalization_registry_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_buffer_builtin() {
        return array_buffer_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_buffer_is_view_builtin() {
        return array_buffer_is_view_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_buffer_byte_length_getter_builtin() {
        return array_buffer_byte_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_buffer_slice_builtin() {
        return array_buffer_slice_builtin(context, invocation).map(Some);
    }
    if entry == js3_shared_array_buffer_builtin() {
        return shared_array_buffer_builtin(context, invocation).map(Some);
    }
    if entry == js3_shared_array_buffer_byte_length_getter_builtin() {
        return shared_array_buffer_byte_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == js3_shared_array_buffer_slice_builtin() {
        return shared_array_buffer_slice_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_builtin() {
        return data_view_builtin(context, invocation).map(Some);
    }
    if entry == js3_atomics_load_builtin() {
        return atomics_load_builtin(context, invocation).map(Some);
    }
    if entry == js3_atomics_store_builtin() {
        return atomics_store_builtin(context, invocation).map(Some);
    }
    if entry == js3_atomics_add_builtin() {
        return atomics_add_builtin(context, invocation).map(Some);
    }
    if entry == js3_atomics_sub_builtin() {
        return atomics_sub_builtin(context, invocation).map(Some);
    }
    if entry == js3_atomics_and_builtin() {
        return atomics_and_builtin(context, invocation).map(Some);
    }
    if entry == js3_atomics_or_builtin() {
        return atomics_or_builtin(context, invocation).map(Some);
    }
    if entry == js3_atomics_xor_builtin() {
        return atomics_xor_builtin(context, invocation).map(Some);
    }
    if entry == js3_atomics_exchange_builtin() {
        return atomics_exchange_builtin(context, invocation).map(Some);
    }
    if entry == js3_atomics_compare_exchange_builtin() {
        return atomics_compare_exchange_builtin(context, invocation).map(Some);
    }
    if entry == js3_atomics_notify_builtin() {
        return atomics_notify_builtin(context, invocation).map(Some);
    }
    if entry == js3_atomics_wait_builtin() {
        return atomics_wait_builtin(context, invocation).map(Some);
    }
    if entry == js3_atomics_wait_async_builtin() {
        return atomics_wait_async_builtin(context, invocation).map(Some);
    }
    if entry == js3_atomics_is_lock_free_builtin() {
        return atomics_is_lock_free_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_builtin() {
        return typed_array_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_from_builtin() {
        return typed_array_from_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_of_builtin() {
        return typed_array_of_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_buffer_getter_builtin() {
        return data_view_buffer_getter_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_byte_length_getter_builtin() {
        return data_view_byte_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_byte_offset_getter_builtin() {
        return data_view_byte_offset_getter_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_get_float32_builtin() {
        return data_view_get_float32_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_get_float64_builtin() {
        return data_view_get_float64_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_get_int16_builtin() {
        return data_view_get_int16_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_get_int32_builtin() {
        return data_view_get_int32_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_get_int8_builtin() {
        return data_view_get_int8_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_get_uint16_builtin() {
        return data_view_get_uint16_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_get_uint32_builtin() {
        return data_view_get_uint32_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_get_uint8_builtin() {
        return data_view_get_uint8_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_set_float32_builtin() {
        return data_view_set_float32_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_set_float64_builtin() {
        return data_view_set_float64_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_set_int16_builtin() {
        return data_view_set_int16_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_set_int32_builtin() {
        return data_view_set_int32_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_set_int8_builtin() {
        return data_view_set_int8_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_set_uint16_builtin() {
        return data_view_set_uint16_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_set_uint32_builtin() {
        return data_view_set_uint32_builtin(context, invocation).map(Some);
    }
    if entry == js3_data_view_set_uint8_builtin() {
        return data_view_set_uint8_builtin(context, invocation).map(Some);
    }
    if entry == js3_int8_array_builtin() {
        return int8_array_builtin(context, invocation).map(Some);
    }
    if entry == js3_int16_array_builtin() {
        return int16_array_builtin(context, invocation).map(Some);
    }
    if entry == js3_int32_array_builtin() {
        return int32_array_builtin(context, invocation).map(Some);
    }
    if entry == js3_float32_array_builtin() {
        return float32_array_builtin(context, invocation).map(Some);
    }
    if entry == js3_float64_array_builtin() {
        return float64_array_builtin(context, invocation).map(Some);
    }
    if entry == js3_big_int64_array_builtin() {
        return big_int64_array_builtin(context, invocation).map(Some);
    }
    if entry == js3_big_uint64_array_builtin() {
        return big_uint64_array_builtin(context, invocation).map(Some);
    }
    if entry == js3_uint32_array_builtin() {
        return uint32_array_builtin(context, invocation).map(Some);
    }
    if entry == js3_uint16_array_builtin() {
        return uint16_array_builtin(context, invocation).map(Some);
    }
    if entry == js3_uint8_clamped_array_builtin() {
        return uint8_clamped_array_builtin(context, invocation).map(Some);
    }
    if entry == js3_uint8_array_builtin() {
        return uint8_array_builtin(context, invocation).map(Some);
    }
    if entry == js3_uint8_array_buffer_getter_builtin() {
        return typed_array_buffer_getter_builtin(context, invocation).map(Some);
    }
    if entry == js3_uint8_array_byte_length_getter_builtin() {
        return typed_array_byte_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == js3_uint8_array_byte_offset_getter_builtin() {
        return typed_array_byte_offset_getter_builtin(context, invocation).map(Some);
    }
    if entry == js3_uint8_array_length_getter_builtin() {
        return typed_array_length_getter_builtin(context, invocation).map(Some);
    }
    if entry == js3_uint8_array_values_builtin() {
        return typed_array_values_builtin(context, invocation).map(Some);
    }
    if entry == js3_uint8_array_keys_builtin() {
        return typed_array_keys_builtin(context, invocation).map(Some);
    }
    if entry == js3_uint8_array_entries_builtin() {
        return typed_array_entries_builtin(context, invocation).map(Some);
    }
    if entry == js3_uint8_array_set_builtin() {
        return uint8_array_set_builtin(context, invocation).map(Some);
    }
    if entry == js3_uint8_array_slice_builtin() {
        return uint8_array_slice_builtin(context, invocation).map(Some);
    }
    if entry == js3_uint8_array_subarray_builtin() {
        return uint8_array_subarray_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_every_builtin() {
        return typed_array_every_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_fill_builtin() {
        return typed_array_fill_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_copy_within_builtin() {
        return typed_array_copy_within_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_filter_builtin() {
        return typed_array_filter_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_for_each_builtin() {
        return typed_array_for_each_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_includes_builtin() {
        return typed_array_includes_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_index_of_builtin() {
        return typed_array_index_of_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_join_builtin() {
        return typed_array_join_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_last_index_of_builtin() {
        return typed_array_last_index_of_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_map_builtin() {
        return typed_array_map_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_reduce_builtin() {
        return typed_array_reduce_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_reduce_right_builtin() {
        return typed_array_reduce_right_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_reverse_builtin() {
        return typed_array_reverse_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_sort_builtin() {
        return typed_array_sort_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_to_locale_string_builtin() {
        return typed_array_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_to_string_builtin() {
        return typed_array_to_string_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_to_reversed_builtin() {
        return typed_array_to_reversed_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_to_sorted_builtin() {
        return typed_array_to_sorted_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_some_builtin() {
        return typed_array_some_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_find_builtin() {
        return typed_array_find_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_find_index_builtin() {
        return typed_array_find_index_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_find_last_builtin() {
        return typed_array_find_last_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_find_last_index_builtin() {
        return typed_array_find_last_index_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_at_builtin() {
        return typed_array_at_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_with_builtin() {
        return typed_array_with_builtin(context, invocation).map(Some);
    }
    if entry == js3_typed_array_to_string_tag_getter_builtin() {
        return typed_array_to_string_tag_getter_builtin(context, invocation).map(Some);
    }
    if entry == js3_json_parse_builtin() {
        return json_parse_builtin(context, invocation).map(Some);
    }
    if entry == js3_json_stringify_builtin() {
        return json_stringify_builtin(context, invocation).map(Some);
    }
    if entry == js3_json_raw_json_builtin() {
        return json_raw_json_builtin(context, invocation).map(Some);
    }
    if entry == js3_json_is_raw_json_builtin() {
        return json_is_raw_json_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_apply_builtin() {
        return reflect_apply_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_construct_builtin() {
        return reflect_construct_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_define_property_builtin() {
        return reflect_define_property_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_delete_property_builtin() {
        return reflect_delete_property_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_get_builtin() {
        return reflect_get_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_get_own_property_descriptor_builtin() {
        return reflect_get_own_property_descriptor_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_get_prototype_of_builtin() {
        return reflect_get_prototype_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_has_builtin() {
        return reflect_has_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_is_extensible_builtin() {
        return reflect_is_extensible_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_own_keys_builtin() {
        return reflect_own_keys_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_prevent_extensions_builtin() {
        return reflect_prevent_extensions_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_set_builtin() {
        return reflect_set_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_set_prototype_of_builtin() {
        return reflect_set_prototype_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_proxy_builtin() {
        return proxy_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_proxy_revocable_builtin() {
        return proxy_revocable_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_proxy_revoke_builtin() {
        return proxy_revoke_builtin(context).map(Some);
    }
    if entry == js3_promise_builtin() {
        return promise_builtin(context, invocation).map(Some);
    }
    if entry == js3_promise_then_builtin() {
        return promise_then_builtin(context, invocation).map(Some);
    }
    if entry == js3_promise_catch_builtin() {
        return promise_catch_builtin(context, invocation).map(Some);
    }
    if entry == js3_promise_finally_builtin() {
        return promise_finally_builtin(context, invocation).map(Some);
    }
    if entry == js3_promise_resolve_builtin() {
        return promise_resolve_builtin(context, invocation).map(Some);
    }
    if entry == js3_promise_reject_builtin() {
        return promise_reject_builtin(context, invocation).map(Some);
    }
    if entry == js3_promise_all_builtin() {
        return promise_all_builtin(context, invocation).map(Some);
    }
    if entry == js3_promise_all_settled_builtin() {
        return promise_all_settled_builtin(context, invocation).map(Some);
    }
    if entry == js3_promise_race_builtin() {
        return promise_race_builtin(context, invocation).map(Some);
    }
    if entry == js3_promise_any_builtin() {
        return promise_any_builtin(context, invocation).map(Some);
    }
    if entry == js3_promise_species_getter_builtin() {
        return promise_species_getter_builtin(context, invocation).map(Some);
    }
    if entry == js3_promise_capability_executor_builtin() {
        return promise_capability_executor_builtin(context, invocation).map(Some);
    }
    if entry == js3_promise_resolve_function_builtin() {
        return promise_resolve_function_builtin(context, invocation).map(Some);
    }
    if entry == js3_promise_reject_function_builtin() {
        return promise_reject_function_builtin(context, invocation).map(Some);
    }
    if entry == js3_promise_finally_function_builtin() {
        return promise_finally_function_builtin(context, invocation).map(Some);
    }
    if entry == js3_promise_all_resolve_element_builtin() {
        return promise_combinator_element_builtin(
            context,
            invocation,
            PromiseCombinatorElementKind::AllResolve,
        )
        .map(Some);
    }
    if entry == js3_promise_all_settled_resolve_element_builtin() {
        return promise_combinator_element_builtin(
            context,
            invocation,
            PromiseCombinatorElementKind::AllSettledResolve,
        )
        .map(Some);
    }
    if entry == js3_promise_all_settled_reject_element_builtin() {
        return promise_combinator_element_builtin(
            context,
            invocation,
            PromiseCombinatorElementKind::AllSettledReject,
        )
        .map(Some);
    }
    if entry == js3_promise_any_reject_element_builtin() {
        return promise_combinator_element_builtin(
            context,
            invocation,
            PromiseCombinatorElementKind::AnyReject,
        )
        .map(Some);
    }
    if entry == js3_array_builtin() {
        return array_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_from_builtin() {
        return array_from_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_is_array_builtin() {
        return array_is_array_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_concat_builtin() {
        return array_concat_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_copy_within_builtin() {
        return array_copy_within_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_fill_builtin() {
        return array_fill_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_join_builtin() {
        return array_join_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_shift_builtin() {
        return array_shift_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_unshift_builtin() {
        return array_unshift_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_filter_builtin() {
        return array_filter_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_for_each_builtin() {
        return array_for_each_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_map_builtin() {
        return array_map_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_reverse_builtin() {
        return array_reverse_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_slice_builtin() {
        return array_slice_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_sort_builtin() {
        return array_sort_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_splice_builtin() {
        return array_splice_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_to_string_builtin() {
        return array_to_string_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_to_locale_string_builtin() {
        return array_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_values_builtin() {
        return array_iterator_factory_builtin(context, invocation, ArrayIterationKind::Value)
            .map(Some);
    }
    if entry == js3_array_keys_builtin() {
        return array_iterator_factory_builtin(context, invocation, ArrayIterationKind::Key)
            .map(Some);
    }
    if entry == js3_array_entries_builtin() {
        return array_iterator_factory_builtin(context, invocation, ArrayIterationKind::Entry)
            .map(Some);
    }
    if entry == js3_map_get_builtin() {
        return map_get_builtin(context, invocation).map(Some);
    }
    if entry == js3_map_set_builtin() {
        return map_set_builtin(context, invocation).map(Some);
    }
    if entry == js3_map_has_builtin() {
        return map_has_builtin(context, invocation).map(Some);
    }
    if entry == js3_map_delete_builtin() {
        return map_delete_builtin(context, invocation).map(Some);
    }
    if entry == js3_map_clear_builtin() {
        return map_clear_builtin(context, invocation).map(Some);
    }
    if entry == js3_map_entries_builtin() {
        return map_iterator_factory_builtin(context, invocation, ArrayIterationKind::Entry)
            .map(Some);
    }
    if entry == js3_map_values_builtin() {
        return map_iterator_factory_builtin(context, invocation, ArrayIterationKind::Value)
            .map(Some);
    }
    if entry == js3_map_keys_builtin() {
        return map_iterator_factory_builtin(context, invocation, ArrayIterationKind::Key)
            .map(Some);
    }
    if entry == js3_map_for_each_builtin() {
        return map_for_each_builtin(context, invocation).map(Some);
    }
    if entry == js3_map_size_getter_builtin() {
        return map_size_getter_builtin(context, invocation).map(Some);
    }
    if entry == js3_set_add_builtin() {
        return set_add_builtin(context, invocation).map(Some);
    }
    if entry == js3_set_has_builtin() {
        return set_has_builtin(context, invocation).map(Some);
    }
    if entry == js3_set_delete_builtin() {
        return set_delete_builtin(context, invocation).map(Some);
    }
    if entry == js3_set_clear_builtin() {
        return set_clear_builtin(context, invocation).map(Some);
    }
    if entry == js3_set_entries_builtin() {
        return set_iterator_factory_builtin(context, invocation, ArrayIterationKind::Entry)
            .map(Some);
    }
    if entry == js3_set_values_builtin() {
        return set_iterator_factory_builtin(context, invocation, ArrayIterationKind::Value)
            .map(Some);
    }
    if entry == js3_set_keys_builtin() {
        return set_iterator_factory_builtin(context, invocation, ArrayIterationKind::Key)
            .map(Some);
    }
    if entry == js3_set_for_each_builtin() {
        return set_for_each_builtin(context, invocation).map(Some);
    }
    if entry == js3_set_size_getter_builtin() {
        return set_size_getter_builtin(context, invocation).map(Some);
    }
    if entry == js3_weak_map_get_builtin() {
        return weak_map_get_builtin(context, invocation).map(Some);
    }
    if entry == js3_weak_map_set_builtin() {
        return weak_map_set_builtin(context, invocation).map(Some);
    }
    if entry == js3_weak_map_has_builtin() {
        return weak_map_has_builtin(context, invocation).map(Some);
    }
    if entry == js3_weak_map_delete_builtin() {
        return weak_map_delete_builtin(context, invocation).map(Some);
    }
    if entry == js3_weak_set_add_builtin() {
        return weak_set_add_builtin(context, invocation).map(Some);
    }
    if entry == js3_weak_set_has_builtin() {
        return weak_set_has_builtin(context, invocation).map(Some);
    }
    if entry == js3_weak_set_delete_builtin() {
        return weak_set_delete_builtin(context, invocation).map(Some);
    }
    if entry == js3_weak_ref_deref_builtin() {
        return weak_ref_deref_builtin(context, invocation).map(Some);
    }
    if entry == js3_finalization_registry_register_builtin() {
        return finalization_registry_register_builtin(context, invocation).map(Some);
    }
    if entry == js3_finalization_registry_unregister_builtin() {
        return finalization_registry_unregister_builtin(context, invocation).map(Some);
    }
    if entry == js3_iterator_prototype_iterator_builtin() {
        return iterator_prototype_iterator_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_iterator_next_builtin() {
        return array_iterator_next_builtin(context, invocation).map(Some);
    }
    if entry == js3_map_iterator_next_builtin() {
        return map_iterator_next_builtin(context, invocation).map(Some);
    }
    if entry == js3_set_iterator_next_builtin() {
        return set_iterator_next_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_builtin() {
        return string_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_iterator_builtin() {
        return string_iterator_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_iterator_next_builtin() {
        return string_iterator_next_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_to_string_builtin() {
        return string_to_string_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_value_of_builtin() {
        return string_value_of_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_concat_builtin() {
        return string_concat_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_char_at_builtin() {
        return string_char_at_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_char_code_at_builtin() {
        return string_char_code_at_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_from_char_code_builtin() {
        return string_from_char_code_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_match_builtin() {
        return string_match_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_last_index_of_builtin() {
        return string_last_index_of_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_pad_end_builtin() {
        return string_pad_end_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_pad_start_builtin() {
        return string_pad_start_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_repeat_builtin() {
        return string_repeat_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_replace_builtin() {
        return string_replace_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_search_builtin() {
        return string_search_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_split_builtin() {
        return string_split_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_slice_builtin() {
        return string_slice_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_substring_builtin() {
        return string_substring_builtin(context, invocation).map(Some);
    }
    if entry == js3_string_starts_with_builtin() {
        return string_starts_with_builtin(context, invocation).map(Some);
    }
    if entry == js3_regexp_builtin() {
        return regexp_builtin(context, invocation).map(Some);
    }
    if entry == js3_regexp_escape_builtin() {
        return regexp_escape_builtin(context, invocation).map(Some);
    }
    if entry == js3_regexp_to_string_builtin() {
        return regexp_to_string_builtin(context, invocation).map(Some);
    }
    if entry == js3_regexp_exec_builtin() {
        return regexp_exec_builtin(context, invocation).map(Some);
    }
    if entry == js3_regexp_test_builtin() {
        return regexp_test_builtin(context, invocation).map(Some);
    }
    if entry == js3_regexp_global_getter_builtin() {
        return regexp_flag_getter_builtin(context, invocation, 'g').map(Some);
    }
    if entry == js3_regexp_ignore_case_getter_builtin() {
        return regexp_flag_getter_builtin(context, invocation, 'i').map(Some);
    }
    if entry == js3_regexp_multiline_getter_builtin() {
        return regexp_flag_getter_builtin(context, invocation, 'm').map(Some);
    }
    if entry == js3_regexp_dot_all_getter_builtin() {
        return regexp_flag_getter_builtin(context, invocation, 's').map(Some);
    }
    if entry == js3_regexp_unicode_getter_builtin() {
        return regexp_flag_getter_builtin(context, invocation, 'u').map(Some);
    }
    if entry == js3_regexp_sticky_getter_builtin() {
        return regexp_flag_getter_builtin(context, invocation, 'y').map(Some);
    }
    if entry == js3_regexp_source_getter_builtin() {
        return regexp_source_getter_builtin(context, invocation).map(Some);
    }
    if entry == js3_regexp_flags_getter_builtin() {
        return regexp_flags_getter_builtin(context, invocation).map(Some);
    }
    if entry == js3_regexp_has_indices_getter_builtin() {
        return regexp_has_indices_getter_builtin(context, invocation).map(Some);
    }
    if entry == js3_date_builtin() {
        return date_builtin(context, invocation).map(Some);
    }
    if entry == js3_date_now_builtin() {
        return date_now_builtin(context, invocation).map(Some);
    }
    if entry == js3_date_parse_builtin() {
        return date_parse_builtin(context, invocation).map(Some);
    }
    if entry == js3_date_utc_builtin() {
        return date_utc_builtin(context, invocation).map(Some);
    }
    if entry == js3_date_to_string_builtin() {
        return date_to_string_builtin(context, invocation, DateStringKind::Full).map(Some);
    }
    if entry == js3_date_to_date_string_builtin() {
        return date_to_string_builtin(context, invocation, DateStringKind::Date).map(Some);
    }
    if entry == js3_date_to_time_string_builtin() {
        return date_to_string_builtin(context, invocation, DateStringKind::Time).map(Some);
    }
    if entry == js3_date_to_locale_string_builtin() {
        return date_to_string_builtin(context, invocation, DateStringKind::Full).map(Some);
    }
    if entry == js3_date_to_locale_date_string_builtin() {
        return date_to_string_builtin(context, invocation, DateStringKind::Date).map(Some);
    }
    if entry == js3_date_to_locale_time_string_builtin() {
        return date_to_string_builtin(context, invocation, DateStringKind::Time).map(Some);
    }
    if entry == js3_date_value_of_builtin() {
        return date_value_of_builtin(context, invocation).map(Some);
    }
    if entry == js3_date_get_time_builtin() {
        return date_value_of_builtin(context, invocation).map(Some);
    }
    if entry == js3_date_get_full_year_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::FullYear, false)
            .map(Some);
    }
    if entry == js3_date_get_utc_full_year_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::FullYear, true)
            .map(Some);
    }
    if entry == js3_date_get_month_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Month, false)
            .map(Some);
    }
    if entry == js3_date_get_utc_month_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Month, true)
            .map(Some);
    }
    if entry == js3_date_get_date_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Date, false)
            .map(Some);
    }
    if entry == js3_date_get_utc_date_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Date, true)
            .map(Some);
    }
    if entry == js3_date_get_day_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Day, false)
            .map(Some);
    }
    if entry == js3_date_get_utc_day_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Day, true).map(Some);
    }
    if entry == js3_date_get_hours_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Hours, false)
            .map(Some);
    }
    if entry == js3_date_get_utc_hours_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Hours, true)
            .map(Some);
    }
    if entry == js3_date_get_minutes_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Minutes, false)
            .map(Some);
    }
    if entry == js3_date_get_utc_minutes_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Minutes, true)
            .map(Some);
    }
    if entry == js3_date_get_seconds_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Seconds, false)
            .map(Some);
    }
    if entry == js3_date_get_utc_seconds_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Seconds, true)
            .map(Some);
    }
    if entry == js3_date_get_milliseconds_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Milliseconds, false)
            .map(Some);
    }
    if entry == js3_date_get_utc_milliseconds_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Milliseconds, true)
            .map(Some);
    }
    if entry == js3_date_get_timezone_offset_builtin() {
        return date_get_timezone_offset_builtin(context, invocation).map(Some);
    }
    if entry == js3_date_set_time_builtin() {
        return date_set_time_builtin(context, invocation).map(Some);
    }
    if entry == js3_date_set_milliseconds_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Milliseconds, false)
            .map(Some);
    }
    if entry == js3_date_set_utc_milliseconds_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Milliseconds, true)
            .map(Some);
    }
    if entry == js3_date_set_seconds_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Seconds, false)
            .map(Some);
    }
    if entry == js3_date_set_utc_seconds_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Seconds, true)
            .map(Some);
    }
    if entry == js3_date_set_minutes_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Minutes, false)
            .map(Some);
    }
    if entry == js3_date_set_utc_minutes_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Minutes, true)
            .map(Some);
    }
    if entry == js3_date_set_hours_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Hours, false)
            .map(Some);
    }
    if entry == js3_date_set_utc_hours_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Hours, true).map(Some);
    }
    if entry == js3_date_set_date_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Date, false).map(Some);
    }
    if entry == js3_date_set_utc_date_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Date, true).map(Some);
    }
    if entry == js3_date_set_month_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Month, false)
            .map(Some);
    }
    if entry == js3_date_set_utc_month_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Month, true).map(Some);
    }
    if entry == js3_date_set_full_year_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::FullYear, false)
            .map(Some);
    }
    if entry == js3_date_set_utc_full_year_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::FullYear, true)
            .map(Some);
    }
    if entry == js3_date_to_utc_string_builtin() {
        return date_to_utc_string_builtin(context, invocation).map(Some);
    }
    if entry == js3_date_to_iso_string_builtin() {
        return date_to_iso_string_builtin(context, invocation).map(Some);
    }
    if entry == js3_date_to_json_builtin() {
        return date_to_json_builtin(context, invocation).map(Some);
    }
    if entry == js3_date_to_temporal_instant_builtin() {
        return date_to_temporal_instant_builtin(context, invocation).map(Some);
    }
    if entry == js3_date_to_primitive_builtin() {
        return date_to_primitive_builtin(context, invocation).map(Some);
    }
    if let Some(result) = temporal::dispatch_temporal_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if entry == js3_number_builtin() {
        return number_builtin(context, invocation).map(Some);
    }
    if entry == js3_number_is_finite_builtin() {
        return number_is_finite_builtin(context, invocation).map(Some);
    }
    if entry == js3_number_is_integer_builtin() {
        return number_is_integer_builtin(context, invocation).map(Some);
    }
    if entry == js3_number_is_nan_builtin() {
        return number_is_nan_builtin(context, invocation).map(Some);
    }
    if entry == js3_number_is_safe_integer_builtin() {
        return number_is_safe_integer_builtin(context, invocation).map(Some);
    }
    if entry == js3_number_to_exponential_builtin() {
        return number_to_exponential_builtin(context, invocation).map(Some);
    }
    if entry == js3_number_to_fixed_builtin() {
        return number_to_fixed_builtin(context, invocation).map(Some);
    }
    if entry == js3_number_to_locale_string_builtin() {
        return number_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == js3_number_to_precision_builtin() {
        return number_to_precision_builtin(context, invocation).map(Some);
    }
    if entry == js3_number_to_string_builtin() {
        return number_to_string_builtin(context, invocation).map(Some);
    }
    if entry == js3_number_value_of_builtin() {
        return number_value_of_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_abs_builtin() {
        return math_abs_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_acos_builtin() {
        return math_acos_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_acosh_builtin() {
        return math_acosh_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_asin_builtin() {
        return math_asin_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_asinh_builtin() {
        return math_asinh_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_atan_builtin() {
        return math_atan_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_atan2_builtin() {
        return math_atan2_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_atanh_builtin() {
        return math_atanh_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_cbrt_builtin() {
        return math_cbrt_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_ceil_builtin() {
        return math_ceil_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_clz32_builtin() {
        return math_clz32_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_cos_builtin() {
        return math_cos_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_cosh_builtin() {
        return math_cosh_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_exp_builtin() {
        return math_exp_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_expm1_builtin() {
        return math_expm1_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_f16round_builtin() {
        return math_f16round_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_floor_builtin() {
        return math_floor_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_fround_builtin() {
        return math_fround_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_hypot_builtin() {
        return math_hypot_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_imul_builtin() {
        return math_imul_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_log_builtin() {
        return math_log_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_log10_builtin() {
        return math_log10_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_log1p_builtin() {
        return math_log1p_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_log2_builtin() {
        return math_log2_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_max_builtin() {
        return math_max_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_min_builtin() {
        return math_min_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_pow_builtin() {
        return math_pow_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_random_builtin() {
        return math_random_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_round_builtin() {
        return math_round_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_sign_builtin() {
        return math_sign_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_sin_builtin() {
        return math_sin_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_sinh_builtin() {
        return math_sinh_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_sqrt_builtin() {
        return math_sqrt_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_sum_precise_builtin() {
        return math_sum_precise_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_tan_builtin() {
        return math_tan_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_tanh_builtin() {
        return math_tanh_builtin(context, invocation).map(Some);
    }
    if entry == js3_math_trunc_builtin() {
        return math_trunc_builtin(context, invocation).map(Some);
    }
    if entry == js3_bigint_builtin() {
        return bigint_builtin(context, invocation).map(Some);
    }
    if entry == js3_bigint_as_int_n_builtin() {
        return bigint_as_int_n_builtin(context, invocation).map(Some);
    }
    if entry == js3_bigint_as_uint_n_builtin() {
        return bigint_as_uint_n_builtin(context, invocation).map(Some);
    }
    if entry == js3_bigint_to_string_builtin() {
        return bigint_to_string_builtin(context, invocation).map(Some);
    }
    if entry == js3_bigint_value_of_builtin() {
        return bigint_value_of_builtin(context, invocation).map(Some);
    }
    if entry == js3_boolean_builtin() {
        return boolean_builtin(context, invocation).map(Some);
    }
    if entry == js3_boolean_to_string_builtin() {
        return boolean_to_string_builtin(context, invocation).map(Some);
    }
    if entry == js3_boolean_value_of_builtin() {
        return boolean_value_of_builtin(context, invocation).map(Some);
    }
    if entry == js3_symbol_builtin() {
        return symbol_builtin(context, invocation).map(Some);
    }
    if entry == js3_symbol_for_builtin() {
        return symbol_for_builtin(context, invocation).map(Some);
    }
    if entry == js3_symbol_key_for_builtin() {
        return symbol_key_for_builtin(context, invocation).map(Some);
    }
    if entry == js3_symbol_to_string_builtin() {
        return symbol_to_string_builtin(context, invocation).map(Some);
    }
    if entry == js3_symbol_value_of_builtin() {
        return symbol_value_of_builtin(context, invocation).map(Some);
    }
    if entry == js3_symbol_to_primitive_builtin() {
        return symbol_to_primitive_builtin(context, invocation).map(Some);
    }
    if entry == js3_array_species_getter_builtin() {
        return array_species_getter_builtin(context, invocation).map(Some);
    }
    if entry == js3_symbol_description_getter_builtin() {
        return symbol_description_getter_builtin(context, invocation).map(Some);
    }
    if entry == js3_error_builtin() {
        return error_builtin(context, invocation).map(Some);
    }
    if entry == js3_error_to_string_builtin() {
        return error_to_string_builtin(context, invocation).map(Some);
    }
    if entry == js3_eval_error_builtin() {
        return eval_error_builtin(context, invocation).map(Some);
    }
    if entry == js3_range_error_builtin() {
        return range_error_builtin(context, invocation).map(Some);
    }
    if entry == js3_reference_error_builtin() {
        return reference_error_builtin(context, invocation).map(Some);
    }
    if entry == js3_syntax_error_builtin() {
        return syntax_error_builtin(context, invocation).map(Some);
    }
    if entry == js3_type_error_builtin() {
        return type_error_builtin(context, invocation).map(Some);
    }
    if entry == js3_uri_error_builtin() {
        return uri_error_builtin(context, invocation).map(Some);
    }
    if entry == js3_aggregate_error_builtin() {
        return aggregate_error_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_suppressed_error_builtin() {
        return suppressed_error_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_disposable_stack_builtin() {
        return disposable_stack_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_disposable_stack_use_builtin() {
        return disposable_stack_use_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_disposable_stack_adopt_builtin() {
        return disposable_stack_adopt_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_disposable_stack_defer_builtin() {
        return disposable_stack_defer_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_disposable_stack_move_builtin() {
        return disposable_stack_move_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_disposable_stack_disposed_getter_builtin() {
        return disposable_stack_disposed_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_disposable_stack_dispose_builtin() {
        return disposal_stack_dispose_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_async_disposable_stack_builtin() {
        return async_disposable_stack_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_async_disposable_stack_use_builtin() {
        return async_disposable_stack_use_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_async_disposable_stack_adopt_builtin() {
        return async_disposable_stack_adopt_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_async_disposable_stack_defer_builtin() {
        return async_disposable_stack_defer_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_async_disposable_stack_move_builtin() {
        return async_disposable_stack_move_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_async_disposable_stack_disposed_getter_builtin() {
        return async_disposable_stack_disposed_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_async_disposable_stack_dispose_async_builtin() {
        return async_disposable_stack_dispose_async_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_async_disposal_resume_builtin() {
        return async_disposal_resume_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_create_sync_disposal_scope_builtin() {
        return create_disposal_scope_builtin(context, lyng_js_env::DisposalCapabilityKind::Sync)
            .map(Some);
    }
    if entry == lyng_js_types::js3_create_async_disposal_scope_builtin() {
        return create_disposal_scope_builtin(context, lyng_js_env::DisposalCapabilityKind::Async)
            .map(Some);
    }
    if entry == lyng_js_types::js3_add_sync_disposable_resource_builtin() {
        return add_disposal_scope_resource_builtin(context, invocation, false).map(Some);
    }
    if entry == lyng_js_types::js3_add_async_disposable_resource_builtin() {
        return add_disposal_scope_resource_builtin(context, invocation, true).map(Some);
    }
    if entry == lyng_js_types::js3_dispose_scope_builtin() {
        return dispose_scope_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_dispose_scope_async_builtin() {
        return dispose_scope_async_builtin(context, invocation).map(Some);
    }
    if entry == js3_eval_builtin() {
        return eval_builtin(context, invocation).map(Some);
    }
    if entry == js3_parse_int_builtin() {
        return parse_int_builtin(context, invocation).map(Some);
    }
    if entry == js3_parse_float_builtin() {
        return parse_float_builtin(context, invocation).map(Some);
    }
    if entry == js3_is_nan_builtin() {
        return is_nan_builtin(context, invocation).map(Some);
    }
    if entry == js3_is_finite_builtin() {
        return is_finite_builtin(context, invocation).map(Some);
    }
    if entry == js3_encode_uri_builtin() {
        return encode_uri_builtin(context, invocation, false).map(Some);
    }
    if entry == js3_encode_uri_component_builtin() {
        return encode_uri_builtin(context, invocation, true).map(Some);
    }
    if entry == js3_decode_uri_builtin() {
        return decode_uri_builtin(context, invocation, false).map(Some);
    }
    if entry == js3_decode_uri_component_builtin() {
        return decode_uri_builtin(context, invocation, true).map(Some);
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

fn allocate_date_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
    value: Value,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let root_shape = {
        let agent = cx.agent();
        agent.realm(realm).and_then(|record| record.root_shape())
    }
    .ok_or_else(|| type_error(cx))?;
    Ok(cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_prototype(Some(prototype))
                .with_date_value(value)
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::Date)),
            AllocationLifetime::Default,
        )
    }))
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

fn allocate_map_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
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
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::Map)),
            AllocationLifetime::Default,
        );
        let installed = objects.install_map_object(object, MapObjectData::new());
        debug_assert!(installed, "fresh Map object should install ordered storage");
        object
    }))
}

fn allocate_set_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
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
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::Set)),
            AllocationLifetime::Default,
        );
        let installed = objects.install_set_object(object, SetObjectData::new());
        debug_assert!(installed, "fresh Set object should install ordered storage");
        object
    }))
}

fn allocate_weak_map_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
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
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::WeakMap)),
            AllocationLifetime::Default,
        );
        let initialized = mutator.init_weak_map(object);
        debug_assert!(
            initialized,
            "fresh WeakMap object should install weak state"
        );
        object
    }))
}

fn allocate_weak_set_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
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
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::WeakSet)),
            AllocationLifetime::Default,
        );
        let initialized = mutator.init_weak_set(object);
        debug_assert!(
            initialized,
            "fresh WeakSet object should install weak state"
        );
        object
    }))
}

fn allocate_weak_ref_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
    target: WeakHeapRef,
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
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::WeakRef)),
            AllocationLifetime::Default,
        );
        let initialized = mutator.init_weak_ref(object, target);
        debug_assert!(
            initialized,
            "fresh WeakRef object should install weak state"
        );
        object
    }))
}

fn allocate_finalization_registry_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
    callback: ObjectRef,
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
                .with_cold_data(ObjectColdData::Ordinary(
                    OrdinaryObjectData::FinalizationRegistry,
                ))
                .with_ordinary_payload_value(Value::from_object_ref(callback)),
            AllocationLifetime::Default,
        );
        let initialized = mutator.init_finalization_registry(object);
        debug_assert!(
            initialized,
            "fresh FinalizationRegistry object should install weak state"
        );
        object
    }))
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

fn bigint_to_number_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Value, Cx::Error> {
    let text = {
        let agent = cx.agent();
        object::bigint_to_string(agent, value, 10)
    };
    let text = map_completion(cx, text)?;
    let number = text.parse::<f64>().unwrap_or_else(|_| {
        if text.starts_with('-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        }
    });
    Ok(Value::from_f64(number))
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

fn current_time_value() -> Value {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_millis() as f64)
        .unwrap_or(f64::NAN);
    Value::from_f64(millis)
}

fn date_display_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<String, Cx::Error> {
    date_format_local(cx, value, DateStringKind::Full)
}

fn to_int32(number: f64) -> i32 {
    if !number.is_finite() || number == 0.0 {
        return 0;
    }
    let truncated = number.trunc();
    let modulo = truncated.rem_euclid(4_294_967_296.0);
    if modulo >= 2_147_483_648.0 {
        (modulo - 4_294_967_296.0) as i32
    } else {
        modulo as i32
    }
}

fn is_ecmascript_whitespace(ch: char) -> bool {
    matches!(
        ch,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{0020}' | '\u{00A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
                | '\n'
                | '\r'
    )
}

fn parse_ascii_digit(byte: u8, radix: u32) -> Option<u32> {
    let digit = match byte {
        b'0'..=b'9' => u32::from(byte - b'0'),
        b'a'..=b'z' => u32::from(byte - b'a') + 10,
        b'A'..=b'Z' => u32::from(byte - b'A') + 10,
        _ => return None,
    };
    (digit < radix).then_some(digit)
}

fn parse_int_string(text: &str, radix_number: f64) -> f64 {
    let mut input = text.trim_start_matches(is_ecmascript_whitespace);
    let mut sign: f64 = 1.0;
    if let Some(rest) = input.strip_prefix('-') {
        sign = -1.0;
        input = rest;
    } else if let Some(rest) = input.strip_prefix('+') {
        input = rest;
    }

    let mut radix = to_int32(radix_number);
    let mut strip_prefix = true;
    if radix != 0 {
        if !(2..=36).contains(&radix) {
            return f64::NAN;
        }
        if radix != 16 {
            strip_prefix = false;
        }
    } else {
        radix = 10;
    }

    if strip_prefix && (input.starts_with("0x") || input.starts_with("0X")) {
        input = &input[2..];
        radix = 16;
    }

    let mut value: f64 = 0.0;
    let mut consumed = 0_usize;
    for byte in input.bytes() {
        let Some(digit) = parse_ascii_digit(byte, radix as u32) else {
            break;
        };
        value = value.mul_add(f64::from(radix), f64::from(digit));
        consumed += 1;
    }
    if consumed == 0 {
        return f64::NAN;
    }
    let result = sign * value;
    if result == 0.0 && sign.is_sign_negative() {
        -0.0
    } else {
        result
    }
}

fn parse_float_string(text: &str) -> f64 {
    let input = text.trim_start_matches(is_ecmascript_whitespace);
    if input.is_empty() {
        return f64::NAN;
    }
    if let Some(rest) = input.strip_prefix("+Infinity") {
        if rest.is_empty() || !rest.starts_with(['n', 'N']) {
            return f64::INFINITY;
        }
    }
    if let Some(rest) = input.strip_prefix("-Infinity") {
        if rest.is_empty() || !rest.starts_with(['n', 'N']) {
            return f64::NEG_INFINITY;
        }
    }
    if let Some(rest) = input.strip_prefix("Infinity") {
        if rest.is_empty() || !rest.starts_with(['n', 'N']) {
            return f64::INFINITY;
        }
    }

    let bytes = input.as_bytes();
    let mut index = 0_usize;
    if matches!(bytes.first(), Some(b'+') | Some(b'-')) {
        index += 1;
    }
    let mut seen_digit = false;
    while bytes
        .get(index)
        .copied()
        .is_some_and(|byte| byte.is_ascii_digit())
    {
        index += 1;
        seen_digit = true;
    }
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        while bytes
            .get(index)
            .copied()
            .is_some_and(|byte| byte.is_ascii_digit())
        {
            index += 1;
            seen_digit = true;
        }
    }
    if !seen_digit {
        return f64::NAN;
    }

    let exponent_start = index;
    if matches!(bytes.get(index), Some(b'e') | Some(b'E')) {
        let mut exponent_index = index + 1;
        if matches!(bytes.get(exponent_index), Some(b'+') | Some(b'-')) {
            exponent_index += 1;
        }
        let exponent_digits_start = exponent_index;
        while bytes
            .get(exponent_index)
            .copied()
            .is_some_and(|byte| byte.is_ascii_digit())
        {
            exponent_index += 1;
        }
        if exponent_index > exponent_digits_start {
            index = exponent_index;
        } else {
            index = exponent_start;
        }
    }

    input[..index].parse::<f64>().unwrap_or(f64::NAN)
}

fn is_uri_unescaped(component: bool, ch: char) -> bool {
    matches!(ch, 'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '!' | '~' | '*' | '\'' | '(' | ')')
        || (!component
            && matches!(
                ch,
                ';' | ',' | '/' | '?' | ':' | '@' | '&' | '=' | '+' | '$' | '#'
            ))
}

fn is_uri_reserved(ch: char) -> bool {
    matches!(
        ch,
        ';' | ',' | '/' | '?' | ':' | '@' | '&' | '=' | '+' | '$' | '#'
    )
}

fn uri_hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn utf8_sequence_len(first: u8) -> Option<usize> {
    match first {
        0x00..=0x7F => Some(1),
        0xC2..=0xDF => Some(2),
        0xE0..=0xEF => Some(3),
        0xF0..=0xF4 => Some(4),
        _ => None,
    }
}

fn encode_uri_text(text: &str, component: bool) -> String {
    let mut encoded = String::new();
    for ch in text.chars() {
        if is_uri_unescaped(component, ch) {
            encoded.push(ch);
            continue;
        }
        let mut buffer = [0_u8; 4];
        for byte in ch.encode_utf8(&mut buffer).as_bytes() {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

fn decode_uri_text(text: &str, component: bool) -> Result<String, ()> {
    let bytes = text.as_bytes();
    let mut index = 0_usize;
    let mut decoded = String::new();
    while index < bytes.len() {
        if bytes[index] != b'%' {
            let ch = text[index..].chars().next().ok_or(())?;
            decoded.push(ch);
            index += ch.len_utf8();
            continue;
        }

        if index + 2 >= bytes.len() {
            return Err(());
        }
        let high = uri_hex_value(bytes[index + 1]).ok_or(())?;
        let low = uri_hex_value(bytes[index + 2]).ok_or(())?;
        let first = (high << 4) | low;
        if first < 0x80 {
            let ch = char::from(first);
            if !component && is_uri_reserved(ch) {
                decoded.push_str(&text[index..index + 3]);
            } else {
                decoded.push(ch);
            }
            index += 3;
            continue;
        }

        let length = utf8_sequence_len(first).ok_or(())?;
        let end = index + length * 3;
        if end > bytes.len() {
            return Err(());
        }
        let mut sequence = Vec::with_capacity(length);
        sequence.push(first);
        for offset in 1..length {
            let byte_index = index + offset * 3;
            if bytes.get(byte_index) != Some(&b'%') {
                return Err(());
            }
            let high = uri_hex_value(bytes[byte_index + 1]).ok_or(())?;
            let low = uri_hex_value(bytes[byte_index + 2]).ok_or(())?;
            sequence.push((high << 4) | low);
        }
        let chunk = std::str::from_utf8(&sequence).map_err(|_| ())?;
        decoded.push_str(chunk);
        index = end;
    }
    Ok(decoded)
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
) -> bool {
    if is_engine_array(cx, object) {
        return true;
    }
    cx.agent()
        .objects()
        .proxy_data(object)
        .is_some_and(|proxy| !proxy.revoked() && is_engine_array(cx, proxy.target()))
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
    let length = cx.get_property_value(
        Value::from_object_ref(object),
        PropertyKey::from_atom(WellKnownAtom::length.id()),
    )?;
    let length = length.as_f64().ok_or_else(|| type_error(cx))?.max(0.0) as u32;
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

fn normalize_engine_array_length_descriptor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    descriptor: PropertyDescriptor,
) -> Result<PropertyDescriptor, Cx::Error> {
    if !descriptor.has_value() {
        return Ok(descriptor);
    }
    let new_len_number =
        to_number_for_builtin(cx, descriptor.value().unwrap_or(Value::undefined()))?;
    let Some(new_len) = valid_array_length(new_len_number) else {
        return Err(range_error(cx));
    };
    let mut normalized = descriptor;
    normalized.set_value(length_value(new_len));
    Ok(normalized)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArrayIterationKind {
    Key = 0,
    Value = 1,
    Entry = 2,
}

const MAX_SAFE_INTEGER_U64: u64 = (1_u64 << 53) - 1;
const ARRAY_RESULT_CAPACITY_HINT_LIMIT: usize = 4096;

const ARRAY_ITERATOR_TARGET_SLOT: u32 = 0;
const ARRAY_ITERATOR_INDEX_SLOT: u32 = 1;
const ARRAY_ITERATOR_KIND_SLOT: u32 = 2;
const MAP_ITERATOR_TARGET_SLOT: u32 = 0;
const MAP_ITERATOR_INDEX_SLOT: u32 = 1;
const MAP_ITERATOR_KIND_SLOT: u32 = 2;
const SET_ITERATOR_TARGET_SLOT: u32 = 0;
const SET_ITERATOR_INDEX_SLOT: u32 = 1;
const SET_ITERATOR_KIND_SLOT: u32 = 2;
const STRING_ITERATOR_STRING_SLOT: u32 = 0;
const STRING_ITERATOR_INDEX_SLOT: u32 = 1;

impl ArrayIterationKind {
    #[inline]
    const fn from_value(value: Value) -> Option<Self> {
        match value.as_smi() {
            Some(0) => Some(Self::Key),
            Some(1) => Some(Self::Value),
            Some(2) => Some(Self::Entry),
            _ => None,
        }
    }

    #[inline]
    const fn into_value(self) -> Value {
        Value::from_smi(self as i32)
    }
}

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
        descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
    ) -> Result<bool, Self::Error> {
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
    proxy::get_prototype_of(&mut BuiltinProxyBridge { cx }, object_ref)
}

fn proxy_set_prototype_of<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    prototype: Option<ObjectRef>,
) -> Result<bool, Cx::Error> {
    proxy::set_prototype_of(&mut BuiltinProxyBridge { cx }, object_ref, prototype)
}

fn proxy_get_own_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    key: PropertyKey,
) -> Result<Option<PropertyDescriptor>, Cx::Error> {
    proxy::get_own_property(&mut BuiltinProxyBridge { cx }, object_ref, key)
}

fn proxy_define_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    key: PropertyKey,
    descriptor: PropertyDescriptor,
    lifetime: AllocationLifetime,
) -> Result<bool, Cx::Error> {
    proxy::define_property(
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
    proxy::has_property(&mut BuiltinProxyBridge { cx }, object_ref, key)
}

fn proxy_own_property_keys<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
) -> Result<Vec<PropertyKey>, Cx::Error> {
    proxy::own_property_keys(&mut BuiltinProxyBridge { cx }, object_ref)
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
    let stored = {
        let agent = cx.agent();
        object::set(agent, object_ref, key, value, AllocationLifetime::Default)
    };
    if !map_completion(cx, stored)? {
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

fn array_species_create<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    original: lyng_js_types::ObjectRef,
    length_hint: usize,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    if !is_engine_array(cx, original) {
        return create_array_result(cx, length_hint);
    }

    let constructor = get_property_from_object(
        cx,
        original,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
    )?;
    if constructor.is_undefined() {
        return create_array_result(cx, length_hint);
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
        return create_array_result(cx, length_hint);
    }

    let species_symbol = {
        let agent = cx.agent();
        agent.well_known_symbol(WellKnownSymbolId::Species)
    };
    let Some(species_symbol) = species_symbol else {
        return create_array_result(cx, length_hint);
    };
    let species = get_property_from_object(
        cx,
        constructor_object,
        PropertyKey::from_symbol(species_symbol),
    )?;
    if species.is_undefined() || species.is_null() {
        return create_array_result(cx, length_hint);
    }
    if species.as_object_ref() == default_array {
        return create_array_result(cx, length_hint);
    }
    Err(type_error(cx))
}

fn array_species_create_for_length<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    original: lyng_js_types::ObjectRef,
    length: u64,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    if !is_array_for_species(cx, original) {
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
    Ok(is_engine_array(cx, object_ref))
}

fn snapshot_indexed_entries<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    length: u32,
) -> Result<Vec<Option<Value>>, Cx::Error> {
    let mut entries = Vec::with_capacity(usize::try_from(length).unwrap_or(usize::MAX));
    for index in 0..length {
        let key = PropertyKey::Index(index);
        if has_property_on_object(cx, object_ref, key)? {
            entries.push(Some(get_property_from_object(cx, object_ref, key)?));
        } else {
            entries.push(None);
        }
    }
    Ok(entries)
}

fn store_indexed_entries<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    entries: &[Option<Value>],
) -> Result<(), Cx::Error> {
    let current_len = array_like_length(cx, object_ref)?;
    for (index, entry) in entries.iter().copied().enumerate() {
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        match entry {
            Some(value) => {
                set_property_on_object(cx, object_ref, PropertyKey::Index(index), value)?
            }
            None => delete_property_from_object(cx, object_ref, PropertyKey::Index(index))?,
        }
    }
    let new_len = u32::try_from(entries.len()).unwrap_or(u32::MAX);
    for index in new_len..current_len {
        delete_property_from_object(cx, object_ref, PropertyKey::Index(index))?;
    }
    define_array_length(cx, object_ref, new_len)
}

fn set_length_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    length: u64,
) -> Result<(), Cx::Error> {
    set_property_on_object(
        cx,
        object_ref,
        PropertyKey::from_atom(WellKnownAtom::length.id()),
        length_value_u64(length),
    )
}

fn create_iterator_result_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    done: bool,
) -> Result<Value, Cx::Error> {
    let result = {
        let realm = cx.builtin_realm();
        let agent = cx.agent();
        iterator::create_iterator_result_object(agent, realm, value, done)
    };
    Ok(Value::from_object_ref(map_completion(cx, result)?))
}

fn allocate_iterator_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: lyng_js_types::ObjectRef,
    cold_data: OrdinaryObjectData,
    slot_values: &[Value],
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    let root_shape = {
        let agent = cx.agent();
        agent.realm(realm).and_then(|record| record.root_shape())
    }
    .ok_or_else(|| type_error(cx))?;
    let iterator_object = cx
        .agent()
        .with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let iterator_object = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape)
                    .with_prototype(Some(prototype))
                    .with_named_slot_count(slot_values.len())
                    .with_cold_data(ObjectColdData::Ordinary(cold_data)),
                AllocationLifetime::Default,
            );
            for (slot_index, slot_value) in slot_values.iter().copied().enumerate() {
                let slot_index =
                    u32::try_from(slot_index).expect("iterator slot index must fit into u32");
                if !objects.init_named_slot(&mut mutator, iterator_object, slot_index, slot_value) {
                    return None;
                }
            }
            Some(iterator_object)
        })
        .ok_or_else(|| type_error(cx))?;
    Ok(iterator_object)
}

fn iterator_slot_value(
    agent: &Agent,
    object_ref: lyng_js_types::ObjectRef,
    expected_kind: OrdinaryObjectData,
    slot_index: u32,
) -> Option<Value> {
    let heap_view = agent.heap().view();
    let matches_kind = matches!(
        agent.objects().object(heap_view, object_ref),
        Some(record)
            if matches!(
                record.cold(),
                ObjectColdData::Ordinary(data) if *data == expected_kind
            )
    );
    if !matches_kind {
        return None;
    }
    let value = agent
        .objects()
        .named_slots(heap_view, object_ref)?
        .get(slot_index as usize)
        .copied()?;
    (value != Value::empty_internal_slot()).then_some(value)
}

fn iterator_slot_value_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    expected_kind: OrdinaryObjectData,
    slot_index: u32,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        iterator_slot_value(agent, object_ref, expected_kind, slot_index)
    }
    .ok_or_else(|| type_error(cx))?;
    Ok(value)
}

fn set_iterator_slot_value_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    expected_kind: OrdinaryObjectData,
    slot_index: u32,
    value: Value,
) -> Result<(), Cx::Error> {
    let updated = cx.agent().with_heap_and_objects(|heap, objects| {
        let matches_kind = matches!(
            objects.object(heap.view(), object_ref),
            Some(record)
                if matches!(
                    record.cold(),
                    ObjectColdData::Ordinary(data) if *data == expected_kind
                )
        );
        if !matches_kind {
            return false;
        }
        let mut mutator = heap.mutator();
        objects.mut_named_slot(&mut mutator, object_ref, slot_index, value)
    });
    if updated {
        Ok(())
    } else {
        Err(type_error(cx))
    }
}

fn array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let arguments = invocation.arguments();
    let array = create_array_result(cx, arguments.len())?;
    if arguments.is_empty() {
        return Ok(Value::from_object_ref(array));
    }

    if arguments.len() == 1 {
        if arguments[0].as_smi().is_some() || arguments[0].as_f64().is_some() {
            let number = to_number_for_builtin(cx, arguments[0])?;
            let Some(length) = valid_array_length(number) else {
                return Err(range_error(cx));
            };
            define_array_length(cx, array, length)?;
            return Ok(Value::from_object_ref(array));
        }
    }

    for (index, value) in arguments.iter().copied().enumerate() {
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        set_property_on_object(cx, array, PropertyKey::Index(index), value)?;
    }
    Ok(Value::from_object_ref(array))
}

fn array_is_array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let is_array = value
        .as_object_ref()
        .is_some_and(|object| is_engine_array(cx, object));
    Ok(Value::from_bool(is_array))
}

fn get_sync_iterator_from_method<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    iterable: Value,
    iterator_method: ObjectRef,
) -> Result<iterator::IteratorRecord, Cx::Error> {
    let iterable_object = cx.to_object_for_builtin_value(cx.caller_realm(), iterable)?;
    let iterator = cx.call_to_completion(
        iterator_method,
        Value::from_object_ref(iterable_object),
        &[],
    )?;
    let iterator_object = iterator.as_object_ref().ok_or_else(|| type_error(cx))?;
    let next_key = property_key_from_text(cx, "next");
    let next_value = cx.get_property_value(Value::from_object_ref(iterator_object), next_key)?;
    let next_method = cx.require_callable_object(next_value)?;
    Ok(iterator::IteratorRecord::new(iterator_object, next_method))
}

fn array_from_iterable_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor_receiver: Value,
    iterable: Value,
    iterator_method: ObjectRef,
    mapper: Option<ObjectRef>,
    this_arg: Value,
) -> Result<Value, Cx::Error> {
    let array = array_from_result_object(cx, constructor_receiver, 0, true)?;
    let mut iterator_record = get_sync_iterator_from_method(cx, iterable, iterator_method)?;
    let mut index = 0_u64;

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
            set_length_property(cx, array, index)?;
            return Ok(Value::from_object_ref(array));
        };
        let next_value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let next_value = match next_value {
            Ok(next_value) => next_value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        if index >= MAX_SAFE_INTEGER_U64 {
            let error = type_error(cx);
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
        let mapped = if let Some(mapper) = mapper {
            match cx.call_to_completion(mapper, this_arg, &[next_value, length_value_u64(index)]) {
                Ok(mapped) => mapped,
                Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
            }
        } else {
            next_value
        };
        let key = array_like_index_property_key(cx, index);
        if let Err(error) = create_data_property_or_throw(cx, array, key, mapped) {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
        index += 1;
    }
}

fn array_from_result_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor_receiver: Value,
    source_len: usize,
    used_iterator: bool,
) -> Result<ObjectRef, Cx::Error> {
    let constructor = constructor_receiver
        .as_object_ref()
        .filter(|object| cx.agent().objects().is_constructor(*object));
    match constructor {
        Some(constructor) if used_iterator => cx.construct_to_completion(constructor, &[], None),
        Some(constructor) => cx.construct_to_completion(
            constructor,
            &[length_value_u64(
                u64::try_from(source_len).unwrap_or(u64::MAX),
            )],
            None,
        ),
        None => create_array_result(cx, source_len),
    }
}

fn array_from_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
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
    if let Some(iterator_symbol) = cx.agent().well_known_symbol(WellKnownSymbolId::Iterator) {
        let iterator_method =
            cx.get_property_value(source, PropertyKey::from_symbol(iterator_symbol))?;
        if !(iterator_method.is_undefined() || iterator_method.is_null()) {
            let iterator_method = cx.require_callable_object(iterator_method)?;
            return array_from_iterable_builtin(
                cx,
                invocation.this_value(),
                source,
                iterator_method,
                mapper,
                this_arg,
            );
        }
    }
    let values = collect_array_like_values_for_from_builtin(cx, source)?;
    let array = array_from_result_object(cx, invocation.this_value(), values.len(), false)?;
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
        let key = array_like_index_property_key(cx, u64::try_from(index).unwrap_or(u64::MAX));
        create_data_property_or_throw(cx, array, key, mapped)?;
    }
    set_length_property(cx, array, u64::try_from(values.len()).unwrap_or(u64::MAX))?;
    Ok(Value::from_object_ref(array))
}

fn map_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if cx.agent().objects().is_map_object(object) {
        Ok(object)
    } else {
        Err(type_error(cx))
    }
}

fn set_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if cx.agent().objects().is_set_object(object) {
        Ok(object)
    } else {
        Err(type_error(cx))
    }
}

fn weak_map_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if cx.agent().objects().is_weak_map_object(object) {
        Ok(object)
    } else {
        Err(type_error(cx))
    }
}

fn weak_set_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if cx.agent().objects().is_weak_set_object(object) {
        Ok(object)
    } else {
        Err(type_error(cx))
    }
}

fn weak_ref_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if cx.agent().objects().is_weak_ref_object(object) {
        Ok(object)
    } else {
        Err(type_error(cx))
    }
}

fn finalization_registry_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if cx.agent().objects().is_finalization_registry_object(object) {
        Ok(object)
    } else {
        Err(type_error(cx))
    }
}

#[inline]
fn weak_heap_ref_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Option<WeakHeapRef> {
    match WeakHeapRef::from_value(value) {
        Some(WeakHeapRef::Symbol(symbol)) if cx.agent().global_symbol_key_for(symbol).is_some() => {
            None
        }
        other => other,
    }
}

#[inline]
fn same_weak_heap_ref_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    left: WeakHeapRef,
    right: Value,
) -> bool {
    weak_heap_ref_from_value(cx, right) == Some(left)
}

fn map_entry_index<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    key: Value,
) -> Result<Option<usize>, Cx::Error> {
    let entries = cx
        .agent()
        .objects()
        .map(object)
        .map(|map| map.entries().to_vec())
        .ok_or_else(|| type_error(cx))?;
    for (index, entry) in entries.iter().copied().enumerate() {
        let Some(entry) = entry else {
            continue;
        };
        let heap_view = cx.agent().heap().view();
        let same = read::same_value_zero(heap_view, entry.key(), key);
        if map_completion(cx, same)? {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

fn set_entry_index<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    value: Value,
) -> Result<Option<usize>, Cx::Error> {
    let entries = cx
        .agent()
        .objects()
        .set_object_data(object)
        .map(|set| set.entries().to_vec())
        .ok_or_else(|| type_error(cx))?;
    for (index, entry) in entries.iter().copied().enumerate() {
        let Some(entry) = entry else {
            continue;
        };
        let heap_view = cx.agent().heap().view();
        let same = read::same_value_zero(heap_view, entry, value);
        if map_completion(cx, same)? {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

fn weak_map_store_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    key: Value,
    value: Value,
) -> Result<(), Cx::Error> {
    let key = weak_heap_ref_from_value(cx, key).ok_or_else(|| type_error(cx))?;
    let stored = cx
        .agent()
        .with_heap_and_objects(|heap, _| heap.mutator().weak_map_set(object, key, value));
    if stored {
        Ok(())
    } else {
        Err(type_error(cx))
    }
}

fn weak_set_add_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    value: Value,
) -> Result<(), Cx::Error> {
    let value = weak_heap_ref_from_value(cx, value).ok_or_else(|| type_error(cx))?;
    let inserted = cx
        .agent()
        .with_heap_and_objects(|heap, _| heap.mutator().weak_set_insert(object, value));
    if inserted {
        Ok(())
    } else {
        Err(type_error(cx))
    }
}

fn perform_weak_map_constructor_entries<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    iterable: Value,
) -> Result<(), Cx::Error> {
    let set_atom = cx.agent().atoms_mut().intern_collectible("set");
    let adder = cx.get_property_value(
        Value::from_object_ref(object),
        PropertyKey::from_atom(set_atom),
    )?;
    let adder = cx.require_callable_object(adder)?;
    let mut iterator_record = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::get_iterator(&mut bridge, iterable)?
    };
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
            return Ok(());
        };
        let next_value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let next_value = match next_value {
            Ok(next_value) => next_value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let Some(entry) = next_value.as_object_ref() else {
            let error = type_error(cx);
            return close_iterator_after_error(cx, &mut iterator_record, error);
        };
        let key = match get_property_from_object(cx, entry, PropertyKey::Index(0)) {
            Ok(key) => key,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let value = match get_property_from_object(cx, entry, PropertyKey::Index(1)) {
            Ok(value) => value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        if let Err(error) =
            cx.call_to_completion(adder, Value::from_object_ref(object), &[key, value])
        {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    }
}

fn perform_weak_set_constructor_values<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    iterable: Value,
) -> Result<(), Cx::Error> {
    let add_atom = cx.agent().atoms_mut().intern_collectible("add");
    let adder = cx.get_property_value(
        Value::from_object_ref(object),
        PropertyKey::from_atom(add_atom),
    )?;
    let adder = cx.require_callable_object(adder)?;
    let mut iterator_record = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::get_iterator(&mut bridge, iterable)?
    };
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
            return Ok(());
        };
        let next_value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let next_value = match next_value {
            Ok(next_value) => next_value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        if let Err(error) =
            cx.call_to_completion(adder, Value::from_object_ref(object), &[next_value])
        {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    }
}

fn map_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().map_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let object = allocate_map_object(cx, realm, prototype)?;
    let iterable = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if !(iterable.is_undefined() || iterable.is_null()) {
        perform_weak_map_constructor_entries(cx, object, iterable)?;
    }
    Ok(Value::from_object_ref(object))
}

fn set_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().set_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let object = allocate_set_object(cx, realm, prototype)?;
    let iterable = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if !(iterable.is_undefined() || iterable.is_null()) {
        perform_weak_set_constructor_values(cx, object, iterable)?;
    }
    Ok(Value::from_object_ref(object))
}

fn weak_map_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().weak_map_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let object = allocate_weak_map_object(cx, realm, prototype)?;
    let iterable = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if !(iterable.is_undefined() || iterable.is_null()) {
        perform_weak_map_constructor_entries(cx, object, iterable)?;
    }
    Ok(Value::from_object_ref(object))
}

fn weak_set_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().weak_set_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let object = allocate_weak_set_object(cx, realm, prototype)?;
    let iterable = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if !(iterable.is_undefined() || iterable.is_null()) {
        perform_weak_set_constructor_values(cx, object, iterable)?;
    }
    Ok(Value::from_object_ref(object))
}

fn weak_ref_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let target = invocation
        .arguments()
        .first()
        .copied()
        .and_then(|value| weak_heap_ref_from_value(cx, value))
        .ok_or_else(|| type_error(cx))?;
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().weak_ref_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let object = allocate_weak_ref_object(cx, realm, prototype, target)?;
    Ok(Value::from_object_ref(object))
}

fn finalization_registry_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let cleanup_callback = invocation
        .arguments()
        .first()
        .copied()
        .ok_or_else(|| type_error(cx))
        .and_then(|value| cx.require_callable_object(value))?;
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().finalization_registry_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let object = allocate_finalization_registry_object(cx, realm, prototype, cleanup_callback)?;
    Ok(Value::from_object_ref(object))
}

fn proxy_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let target = require_proxy_argument_object(cx, invocation, 0)?;
    let handler = require_proxy_argument_object(cx, invocation, 1)?;
    let proxy = allocate_proxy_object(cx, cx.builtin_realm(), target, handler)?;
    Ok(Value::from_object_ref(proxy))
}

fn proxy_revocable_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target = require_proxy_argument_object(cx, invocation, 0)?;
    let handler = require_proxy_argument_object(cx, invocation, 1)?;
    let realm = cx.builtin_realm();
    let proxy = allocate_proxy_object(cx, realm, target, handler)?;
    let revoke = cx.allocate_builtin_function(lyng_js_types::js3_proxy_revoke_builtin())?;
    let object_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().object_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let pair = cx.allocate_ordinary_object_with_prototype(realm, Some(object_prototype))?;
    let installed = cx.agent().with_heap_and_objects(|heap, objects| {
        objects.set_function_home_object(&mut heap.mutator(), revoke, Some(proxy))
    });
    if !installed {
        return Err(type_error(cx));
    }
    let proxy_key = property_key_from_text(cx, "proxy");
    create_data_property_or_throw(cx, pair, proxy_key, Value::from_object_ref(proxy))?;
    let revoke_key = property_key_from_text(cx, "revoke");
    create_data_property_or_throw(cx, pair, revoke_key, Value::from_object_ref(revoke))?;
    Ok(Value::from_object_ref(pair))
}

fn proxy_revoke_builtin<Cx: PublicBuiltinDispatchContext>(cx: &mut Cx) -> Result<Value, Cx::Error> {
    let callee = cx.callee_object();
    let proxy = {
        let agent = cx.agent();
        agent
            .objects()
            .function_data(callee)
            .and_then(|data| data.home_object())
    };
    let Some(proxy) = proxy else {
        return Ok(Value::undefined());
    };
    let (revoked, cleared) = cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let revoked = objects.revoke_proxy(&mut mutator, proxy);
        let cleared = objects.set_function_home_object(&mut mutator, callee, None);
        (revoked, cleared)
    });
    if !revoked || !cleared {
        return Err(type_error(cx));
    }
    Ok(Value::undefined())
}

fn map_get_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = map_this_object(cx, invocation.this_value())?;
    let key = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(index) = map_entry_index(cx, object, key)? else {
        return Ok(Value::undefined());
    };
    cx.agent()
        .objects()
        .map(object)
        .and_then(|map| map.entries().get(index).copied().flatten())
        .map(MapEntry::value)
        .ok_or_else(|| type_error(cx))
}

fn map_set_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = map_this_object(cx, invocation.this_value())?;
    let key = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let existing = map_entry_index(cx, object, key)?;
    let updated = cx.agent().with_heap_and_objects(|_, objects| {
        objects.with_map_mut(object, |map| {
            if let Some(index) = existing {
                if let Some(Some(entry)) = map.entries_mut().get_mut(index) {
                    entry.set_value(value);
                    true
                } else {
                    false
                }
            } else {
                map.push(MapEntry::new(key, value));
                true
            }
        })
    });
    if updated == Some(true) {
        Ok(invocation.this_value())
    } else {
        Err(type_error(cx))
    }
}

fn map_has_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = map_this_object(cx, invocation.this_value())?;
    let key = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    Ok(Value::from_bool(
        map_entry_index(cx, object, key)?.is_some(),
    ))
}

fn map_delete_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = map_this_object(cx, invocation.this_value())?;
    let key = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(index) = map_entry_index(cx, object, key)? else {
        return Ok(Value::from_bool(false));
    };
    let deleted = cx.agent().with_heap_and_objects(|_, objects| {
        objects.with_map_mut(object, |map| map.delete_index(index))
    });
    match deleted {
        Some(true) => Ok(Value::from_bool(true)),
        Some(false) => Ok(Value::from_bool(false)),
        None => Err(type_error(cx)),
    }
}

fn map_clear_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = map_this_object(cx, invocation.this_value())?;
    let cleared = cx
        .agent()
        .with_heap_and_objects(|_, objects| objects.with_map_mut(object, MapObjectData::clear));
    if cleared.is_some() {
        Ok(Value::undefined())
    } else {
        Err(type_error(cx))
    }
}

fn map_size_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = map_this_object(cx, invocation.this_value())?;
    let size = cx
        .agent()
        .objects()
        .map(object)
        .map(MapObjectData::len)
        .ok_or_else(|| type_error(cx))?;
    Ok(length_value_u64(u64::try_from(size).unwrap_or(u64::MAX)))
}

fn set_add_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if set_entry_index(cx, object, value)?.is_none() {
        let inserted = cx.agent().with_heap_and_objects(|_, objects| {
            objects.with_set_mut(object, |set| {
                set.push(value);
                true
            })
        });
        if inserted != Some(true) {
            return Err(type_error(cx));
        }
    }
    Ok(invocation.this_value())
}

fn set_has_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    Ok(Value::from_bool(
        set_entry_index(cx, object, value)?.is_some(),
    ))
}

fn set_delete_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(index) = set_entry_index(cx, object, value)? else {
        return Ok(Value::from_bool(false));
    };
    let deleted = cx.agent().with_heap_and_objects(|_, objects| {
        objects.with_set_mut(object, |set| set.delete_index(index))
    });
    match deleted {
        Some(true) => Ok(Value::from_bool(true)),
        Some(false) => Ok(Value::from_bool(false)),
        None => Err(type_error(cx)),
    }
}

fn set_clear_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
    let cleared = cx
        .agent()
        .with_heap_and_objects(|_, objects| objects.with_set_mut(object, SetObjectData::clear));
    if cleared.is_some() {
        Ok(Value::undefined())
    } else {
        Err(type_error(cx))
    }
}

fn set_size_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
    let size = cx
        .agent()
        .objects()
        .set_object_data(object)
        .map(SetObjectData::len)
        .ok_or_else(|| type_error(cx))?;
    Ok(length_value_u64(u64::try_from(size).unwrap_or(u64::MAX)))
}

fn weak_map_get_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = weak_map_this_object(cx, invocation.this_value())?;
    let key = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(key) = weak_heap_ref_from_value(cx, key) else {
        return Ok(Value::undefined());
    };
    let value = cx
        .agent()
        .heap()
        .view()
        .weak_map_get(object, key)
        .ok_or_else(|| type_error(cx))?
        .unwrap_or(Value::undefined());
    Ok(value)
}

fn weak_map_set_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = weak_map_this_object(cx, invocation.this_value())?;
    let key = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    weak_map_store_value(cx, object, key, value)?;
    Ok(invocation.this_value())
}

fn weak_map_has_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = weak_map_this_object(cx, invocation.this_value())?;
    let key = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(key) = weak_heap_ref_from_value(cx, key) else {
        return Ok(Value::from_bool(false));
    };
    let has = cx
        .agent()
        .heap()
        .view()
        .weak_map_get(object, key)
        .ok_or_else(|| type_error(cx))?
        .is_some();
    Ok(Value::from_bool(has))
}

fn weak_map_delete_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = weak_map_this_object(cx, invocation.this_value())?;
    let key = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(key) = weak_heap_ref_from_value(cx, key) else {
        return Ok(Value::from_bool(false));
    };
    let deleted = cx
        .agent()
        .with_heap_and_objects(|heap, _| heap.mutator().weak_map_delete(object, key));
    deleted.map(Value::from_bool).ok_or_else(|| type_error(cx))
}

fn weak_set_add_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = weak_set_this_object(cx, invocation.this_value())?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    weak_set_add_value(cx, object, value)?;
    Ok(invocation.this_value())
}

fn weak_set_has_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = weak_set_this_object(cx, invocation.this_value())?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(value) = weak_heap_ref_from_value(cx, value) else {
        return Ok(Value::from_bool(false));
    };
    let has = cx
        .agent()
        .heap()
        .view()
        .weak_set_contains(object, value)
        .ok_or_else(|| type_error(cx))?;
    Ok(Value::from_bool(has))
}

fn weak_set_delete_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = weak_set_this_object(cx, invocation.this_value())?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(value) = weak_heap_ref_from_value(cx, value) else {
        return Ok(Value::from_bool(false));
    };
    let deleted = cx
        .agent()
        .with_heap_and_objects(|heap, _| heap.mutator().weak_set_delete(object, value));
    deleted.map(Value::from_bool).ok_or_else(|| type_error(cx))
}

fn weak_ref_deref_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = weak_ref_this_object(cx, invocation.this_value())?;
    let target = cx
        .agent()
        .weak_ref_target(object)
        .ok_or_else(|| type_error(cx))?
        .map(|target| {
            cx.agent().keep_weak_target_alive(target);
            match target {
                WeakHeapRef::Object(object) => Value::from_object_ref(object),
                WeakHeapRef::Symbol(symbol) => Value::from_symbol_ref(symbol),
            }
        })
        .unwrap_or(Value::undefined());
    Ok(target)
}

fn finalization_registry_register_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let registry = finalization_registry_this_object(cx, invocation.this_value())?;
    let target = invocation
        .arguments()
        .first()
        .copied()
        .and_then(|value| weak_heap_ref_from_value(cx, value))
        .ok_or_else(|| type_error(cx))?;
    let holdings = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    if same_weak_heap_ref_value(cx, target, holdings) {
        return Err(type_error(cx));
    }
    let unregister_token = match invocation.arguments().get(2).copied() {
        Some(value) if value.is_undefined() => None,
        Some(value) => Some(weak_heap_ref_from_value(cx, value).ok_or_else(|| type_error(cx))?),
        None => None,
    };

    let registered = cx.agent().with_heap_and_objects(|heap, _| {
        heap.mutator()
            .finalization_registry_register(registry, target, holdings, unregister_token)
    });
    if !registered {
        return Err(type_error(cx));
    }
    Ok(Value::undefined())
}

fn finalization_registry_unregister_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let registry = finalization_registry_this_object(cx, invocation.this_value())?;
    let unregister_token = invocation
        .arguments()
        .first()
        .copied()
        .and_then(|value| weak_heap_ref_from_value(cx, value))
        .ok_or_else(|| type_error(cx))?;
    let unregistered = cx
        .agent()
        .with_heap_and_objects(|heap, _| {
            heap.mutator()
                .finalization_registry_unregister(registry, unregister_token)
        })
        .ok_or_else(|| type_error(cx))?;
    Ok(Value::from_bool(unregistered))
}

fn map_for_each_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = map_this_object(cx, invocation.this_value())?;
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
    let self_value = Value::from_object_ref(object);
    let mut index = 0_usize;
    loop {
        let next = {
            let agent = cx.agent();
            let Some(map) = agent.objects().map(object) else {
                return Err(type_error(cx));
            };
            map.entries().get(index).copied()
        };
        let Some(next) = next else {
            break;
        };
        index = index.saturating_add(1);
        let Some(entry) = next else {
            continue;
        };
        let arguments = [entry.value(), entry.key(), self_value];
        let _ = cx.call_to_completion(callback, this_arg, &arguments)?;
    }
    Ok(Value::undefined())
}

fn set_for_each_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
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
    let self_value = Value::from_object_ref(object);
    let mut index = 0_usize;
    loop {
        let next = {
            let agent = cx.agent();
            let Some(set) = agent.objects().set_object_data(object) else {
                return Err(type_error(cx));
            };
            set.entries().get(index).copied()
        };
        let Some(next) = next else {
            break;
        };
        index = index.saturating_add(1);
        let Some(entry) = next else {
            continue;
        };
        let arguments = [entry, entry, self_value];
        let _ = cx.call_to_completion(callback, this_arg, &arguments)?;
    }
    Ok(Value::undefined())
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
        return compare_array_sort_values(cx, Some(compare_fn), left, right);
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
    let promise_constructor = promise_default_constructor(cx)?;
    let capability = new_promise_capability(cx, promise_constructor)?;
    let promise_object = promise_capability_promise(cx, capability)?;
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
    typed_array_iterator_factory_builtin(cx, invocation, ArrayIterationKind::Value)
}

fn typed_array_keys_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_iterator_factory_builtin(cx, invocation, ArrayIterationKind::Key)
}

fn typed_array_entries_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_iterator_factory_builtin(cx, invocation, ArrayIterationKind::Entry)
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

fn json_raw_json_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let raw_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let raw_string = to_string_string_ref(cx, raw_value)?;
    let raw_text = string_ref_text(cx, raw_string)?;
    if !json_raw_text_is_valid(&raw_text) {
        return Err(syntax_error(cx));
    }
    let object = allocate_json_raw_object(cx, cx.builtin_realm(), raw_string)?;
    let frozen = cx.set_integrity_level(object, true)?;
    if !frozen {
        return Err(type_error(cx));
    }
    Ok(Value::from_object_ref(object))
}

fn json_is_raw_json_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let is_raw = value
        .as_object_ref()
        .is_some_and(|object| cx.agent().objects().is_json_raw_object(object));
    Ok(Value::from_bool(is_raw))
}

fn json_raw_text_is_valid(text: &str) -> bool {
    let bytes = text.as_bytes();
    let Some((&first, &last)) = bytes.first().zip(bytes.last()) else {
        return false;
    };
    if matches!(first, b'\t' | b'\n' | b'\r' | b' ') || matches!(last, b'\t' | b'\n' | b'\r' | b' ')
    {
        return false;
    }
    match first {
        b'n' => text == "null",
        b't' => text == "true",
        b'f' => text == "false",
        b'"' => json_raw_string_literal_is_valid(bytes),
        b'-' | b'0'..=b'9' => json_raw_number_literal_is_valid(bytes),
        _ => false,
    }
}

fn json_raw_string_literal_is_valid(bytes: &[u8]) -> bool {
    if bytes.len() < 2 || bytes.first() != Some(&b'"') || bytes.last() != Some(&b'"') {
        return false;
    }
    let mut index = 1;
    while index < bytes.len() - 1 {
        let byte = bytes[index];
        if byte == b'"' || byte < 0x20 {
            return false;
        }
        if byte == b'\\' {
            index += 1;
            let Some(&escape) = bytes.get(index) else {
                return false;
            };
            match escape {
                b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {
                    index += 1;
                }
                b'u' => {
                    let Some(hex_digits) = bytes.get(index + 1..index + 5) else {
                        return false;
                    };
                    if !hex_digits.iter().all(u8::is_ascii_hexdigit) {
                        return false;
                    }
                    index += 5;
                }
                _ => return false,
            }
            continue;
        }
        if byte < 0x80 {
            index += 1;
            continue;
        }
        let Ok(text) = std::str::from_utf8(&bytes[index..]) else {
            return false;
        };
        let Some(ch) = text.chars().next() else {
            return false;
        };
        if u32::from(ch) <= 0x1F {
            return false;
        }
        index += ch.len_utf8();
    }
    true
}

fn json_raw_number_literal_is_valid(bytes: &[u8]) -> bool {
    let mut index = 0;
    if bytes.get(index) == Some(&b'-') {
        index += 1;
    }
    match bytes.get(index) {
        Some(b'0') => {
            index += 1;
        }
        Some(b'1'..=b'9') => {
            index += 1;
            while matches!(bytes.get(index), Some(b'0'..=b'9')) {
                index += 1;
            }
        }
        _ => return false,
    }
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        if !matches!(bytes.get(index), Some(b'0'..=b'9')) {
            return false;
        }
        while matches!(bytes.get(index), Some(b'0'..=b'9')) {
            index += 1;
        }
    }
    if matches!(bytes.get(index), Some(b'e') | Some(b'E')) {
        index += 1;
        if matches!(bytes.get(index), Some(b'+') | Some(b'-')) {
            index += 1;
        }
        if !matches!(bytes.get(index), Some(b'0'..=b'9')) {
            return false;
        }
        while matches!(bytes.get(index), Some(b'0'..=b'9')) {
            index += 1;
        }
    }
    index == bytes.len()
}

const JSON_CHAR_QUOTE: u16 = b'"' as u16;
const JSON_CHAR_PLUS: u16 = b'+' as u16;
const JSON_CHAR_COMMA: u16 = b',' as u16;
const JSON_CHAR_MINUS: u16 = b'-' as u16;
const JSON_CHAR_DOT: u16 = b'.' as u16;
const JSON_CHAR_SLASH: u16 = b'/' as u16;
const JSON_CHAR_COLON: u16 = b':' as u16;
const JSON_CHAR_LEFT_BRACKET: u16 = b'[' as u16;
const JSON_CHAR_BACKSLASH: u16 = b'\\' as u16;
const JSON_CHAR_RIGHT_BRACKET: u16 = b']' as u16;
const JSON_CHAR_LEFT_BRACE: u16 = b'{' as u16;
const JSON_CHAR_RIGHT_BRACE: u16 = b'}' as u16;
const JSON_CHAR_ZERO: u16 = b'0' as u16;
const JSON_CHAR_NINE: u16 = b'9' as u16;
const JSON_CHAR_UPPER_A: u16 = b'A' as u16;
const JSON_CHAR_UPPER_E: u16 = b'E' as u16;
const JSON_CHAR_UPPER_F: u16 = b'F' as u16;
const JSON_CHAR_B: u16 = b'b' as u16;
const JSON_CHAR_E: u16 = b'e' as u16;
const JSON_CHAR_F: u16 = b'f' as u16;
const JSON_CHAR_N: u16 = b'n' as u16;
const JSON_CHAR_R: u16 = b'r' as u16;
const JSON_CHAR_T: u16 = b't' as u16;
const JSON_CHAR_U: u16 = b'u' as u16;
const JSON_CHAR_LOWER_A: u16 = b'a' as u16;
const JSON_CHAR_LOWER_F: u16 = b'f' as u16;
const JSON_CHAR_ONE: u16 = b'1' as u16;

enum JsonParseNode {
    Primitive {
        value: Value,
        source_units: Vec<u16>,
    },
    Array {
        object: ObjectRef,
        elements: Vec<JsonParseNode>,
    },
    Object {
        object: ObjectRef,
        entries: Vec<(PropertyKey, JsonParseNode)>,
    },
}

impl JsonParseNode {
    fn value(&self) -> Value {
        match self {
            Self::Primitive { value, .. } => *value,
            Self::Array { object, .. } | Self::Object { object, .. } => {
                Value::from_object_ref(*object)
            }
        }
    }

    fn source_units_for_value(&self, value: Value) -> Option<&[u16]> {
        match self {
            Self::Primitive {
                value: original,
                source_units,
            } if *original == value => Some(source_units),
            _ => None,
        }
    }

    fn array_elements_for_object(&self, object: ObjectRef) -> Option<&[JsonParseNode]> {
        match self {
            Self::Array {
                object: original,
                elements,
            } if *original == object => Some(elements),
            _ => None,
        }
    }

    fn object_entry_for_key(&self, object: ObjectRef, key: PropertyKey) -> Option<&JsonParseNode> {
        match self {
            Self::Object {
                object: original,
                entries,
            } if *original == object => entries
                .iter()
                .find(|(candidate, _)| *candidate == key)
                .map(|(_, node)| node),
            _ => None,
        }
    }
}

struct JsonParser<'a> {
    units: &'a [u16],
    index: usize,
}

impl<'a> JsonParser<'a> {
    fn new(units: &'a [u16]) -> Self {
        Self { units, index: 0 }
    }

    fn parse_text<Cx: PublicBuiltinDispatchContext>(
        &mut self,
        cx: &mut Cx,
    ) -> Result<JsonParseNode, Cx::Error> {
        self.skip_whitespace();
        let value = self.parse_value(cx)?;
        self.skip_whitespace();
        if self.index != self.units.len() {
            return Err(syntax_error(cx));
        }
        Ok(value)
    }

    fn parse_value<Cx: PublicBuiltinDispatchContext>(
        &mut self,
        cx: &mut Cx,
    ) -> Result<JsonParseNode, Cx::Error> {
        self.skip_whitespace();
        match self.peek() {
            Some(JSON_CHAR_N) => self.parse_keyword(cx, b"null", Value::null()),
            Some(JSON_CHAR_T) => self.parse_keyword(cx, b"true", Value::from_bool(true)),
            Some(JSON_CHAR_F) => self.parse_keyword(cx, b"false", Value::from_bool(false)),
            Some(JSON_CHAR_QUOTE) => {
                let start = self.index;
                let units = self.parse_string_units(cx)?;
                Ok(JsonParseNode::Primitive {
                    value: string_from_code_units(cx, &units),
                    source_units: self.units[start..self.index].to_vec(),
                })
            }
            Some(JSON_CHAR_LEFT_BRACKET) => self.parse_array(cx),
            Some(JSON_CHAR_LEFT_BRACE) => self.parse_object(cx),
            Some(JSON_CHAR_MINUS) | Some(JSON_CHAR_ZERO..=JSON_CHAR_NINE) => self.parse_number(cx),
            _ => Err(syntax_error(cx)),
        }
    }

    fn parse_keyword<Cx: PublicBuiltinDispatchContext>(
        &mut self,
        cx: &mut Cx,
        keyword: &[u8],
        value: Value,
    ) -> Result<JsonParseNode, Cx::Error> {
        let start = self.index;
        for expected in keyword {
            if self.advance() != Some(u16::from(*expected)) {
                return Err(syntax_error(cx));
            }
        }
        Ok(JsonParseNode::Primitive {
            value,
            source_units: self.units[start..self.index].to_vec(),
        })
    }

    fn parse_array<Cx: PublicBuiltinDispatchContext>(
        &mut self,
        cx: &mut Cx,
    ) -> Result<JsonParseNode, Cx::Error> {
        self.expect(cx, JSON_CHAR_LEFT_BRACKET)?;
        self.skip_whitespace();
        let mut elements = Vec::new();
        if self.peek() == Some(JSON_CHAR_RIGHT_BRACKET) {
            self.index += 1;
        } else {
            loop {
                elements.push(self.parse_value(cx)?);
                self.skip_whitespace();
                match self.advance() {
                    Some(JSON_CHAR_COMMA) => self.skip_whitespace(),
                    Some(JSON_CHAR_RIGHT_BRACKET) => break,
                    _ => return Err(syntax_error(cx)),
                }
            }
        }

        let array = create_array_result(cx, elements.len())?;
        for (index, element) in elements.iter().enumerate() {
            let key = array_like_index_property_key(cx, u64::try_from(index).unwrap_or(u64::MAX));
            create_data_property_or_throw(cx, array, key, element.value())?;
        }
        Ok(JsonParseNode::Array {
            object: array,
            elements,
        })
    }

    fn parse_object<Cx: PublicBuiltinDispatchContext>(
        &mut self,
        cx: &mut Cx,
    ) -> Result<JsonParseNode, Cx::Error> {
        self.expect(cx, JSON_CHAR_LEFT_BRACE)?;
        self.skip_whitespace();
        let mut entries = Vec::new();
        let prototype = json_object_prototype(cx)?;
        let object =
            cx.allocate_ordinary_object_with_prototype(cx.builtin_realm(), Some(prototype))?;
        if self.peek() == Some(JSON_CHAR_RIGHT_BRACE) {
            self.index += 1;
        } else {
            loop {
                let key_units = self.parse_string_units(cx)?;
                self.skip_whitespace();
                self.expect(cx, JSON_CHAR_COLON)?;
                let key_value = string_from_code_units(cx, &key_units);
                let key = cx.to_property_key(key_value)?;
                let value = self.parse_value(cx)?;
                create_data_property_or_throw(cx, object, key, value.value())?;
                if let Some(existing) = entries.iter_mut().find(|(candidate, _)| *candidate == key)
                {
                    existing.1 = value;
                } else {
                    entries.push((key, value));
                }
                self.skip_whitespace();
                match self.advance() {
                    Some(JSON_CHAR_COMMA) => self.skip_whitespace(),
                    Some(JSON_CHAR_RIGHT_BRACE) => break,
                    _ => return Err(syntax_error(cx)),
                }
            }
        }
        Ok(JsonParseNode::Object { object, entries })
    }

    fn parse_number<Cx: PublicBuiltinDispatchContext>(
        &mut self,
        cx: &mut Cx,
    ) -> Result<JsonParseNode, Cx::Error> {
        let start = self.index;
        if self.peek() == Some(JSON_CHAR_MINUS) {
            self.index += 1;
        }
        match self.peek() {
            Some(JSON_CHAR_ZERO) => {
                self.index += 1;
            }
            Some(JSON_CHAR_ONE..=JSON_CHAR_NINE) => {
                self.index += 1;
                while matches!(self.peek(), Some(JSON_CHAR_ZERO..=JSON_CHAR_NINE)) {
                    self.index += 1;
                }
            }
            _ => return Err(syntax_error(cx)),
        }
        if self.peek() == Some(JSON_CHAR_DOT) {
            self.index += 1;
            if !matches!(self.peek(), Some(JSON_CHAR_ZERO..=JSON_CHAR_NINE)) {
                return Err(syntax_error(cx));
            }
            while matches!(self.peek(), Some(JSON_CHAR_ZERO..=JSON_CHAR_NINE)) {
                self.index += 1;
            }
        }
        if matches!(self.peek(), Some(JSON_CHAR_E) | Some(JSON_CHAR_UPPER_E)) {
            self.index += 1;
            if matches!(self.peek(), Some(JSON_CHAR_PLUS) | Some(JSON_CHAR_MINUS)) {
                self.index += 1;
            }
            if !matches!(self.peek(), Some(JSON_CHAR_ZERO..=JSON_CHAR_NINE)) {
                return Err(syntax_error(cx));
            }
            while matches!(self.peek(), Some(JSON_CHAR_ZERO..=JSON_CHAR_NINE)) {
                self.index += 1;
            }
        }

        let text: String = self.units[start..self.index]
            .iter()
            .map(|unit| char::from_u32(u32::from(*unit)).expect("JSON numbers should stay ASCII"))
            .collect();
        let number = text.parse::<f64>().map_err(|_| syntax_error(cx))?;
        Ok(JsonParseNode::Primitive {
            value: number_value(number),
            source_units: self.units[start..self.index].to_vec(),
        })
    }

    fn parse_string_units<Cx: PublicBuiltinDispatchContext>(
        &mut self,
        cx: &mut Cx,
    ) -> Result<Vec<u16>, Cx::Error> {
        self.expect(cx, JSON_CHAR_QUOTE)?;
        let mut units = Vec::new();
        loop {
            let Some(unit) = self.advance() else {
                return Err(syntax_error(cx));
            };
            match unit {
                JSON_CHAR_QUOTE => return Ok(units),
                JSON_CHAR_BACKSLASH => {
                    let escaped = self.advance().ok_or_else(|| syntax_error(cx))?;
                    match escaped {
                        JSON_CHAR_QUOTE => units.push(JSON_CHAR_QUOTE),
                        JSON_CHAR_BACKSLASH => units.push(JSON_CHAR_BACKSLASH),
                        JSON_CHAR_SLASH => units.push(JSON_CHAR_SLASH),
                        JSON_CHAR_B => units.push(0x0008),
                        JSON_CHAR_F => units.push(0x000C),
                        JSON_CHAR_N => units.push(0x000A),
                        JSON_CHAR_R => units.push(0x000D),
                        JSON_CHAR_T => units.push(0x0009),
                        JSON_CHAR_U => units.push(self.parse_hex_escape(cx)?),
                        _ => return Err(syntax_error(cx)),
                    }
                }
                unit if unit < 0x0020 => return Err(syntax_error(cx)),
                unit => units.push(unit),
            }
        }
    }

    fn parse_hex_escape<Cx: PublicBuiltinDispatchContext>(
        &mut self,
        cx: &mut Cx,
    ) -> Result<u16, Cx::Error> {
        let mut value = 0u16;
        for _ in 0..4 {
            let digit = self.advance().ok_or_else(|| syntax_error(cx))?;
            let nibble = match digit {
                JSON_CHAR_ZERO..=JSON_CHAR_NINE => digit - JSON_CHAR_ZERO,
                JSON_CHAR_LOWER_A..=JSON_CHAR_LOWER_F => digit - JSON_CHAR_LOWER_A + 10,
                JSON_CHAR_UPPER_A..=JSON_CHAR_UPPER_F => digit - JSON_CHAR_UPPER_A + 10,
                _ => return Err(syntax_error(cx)),
            };
            value = (value << 4) | nibble;
        }
        Ok(value)
    }

    fn expect<Cx: PublicBuiltinDispatchContext>(
        &mut self,
        cx: &mut Cx,
        expected: u16,
    ) -> Result<(), Cx::Error> {
        if self.advance() == Some(expected) {
            Ok(())
        } else {
            Err(syntax_error(cx))
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(0x0009 | 0x000A | 0x000D | 0x0020)) {
            self.index += 1;
        }
    }

    fn peek(&self) -> Option<u16> {
        self.units.get(self.index).copied()
    }

    fn advance(&mut self) -> Option<u16> {
        let unit = self.peek()?;
        self.index += 1;
        Some(unit)
    }
}

fn json_object_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    cx.agent()
        .realm(realm)
        .and_then(|realm| realm.intrinsics().object_prototype())
        .ok_or_else(|| type_error(cx))
}

fn json_enumerable_string_keys<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
) -> Result<Vec<PropertyKey>, Cx::Error> {
    let keys = { proxy_own_property_keys(cx, object) };
    let keys = keys?;
    let mut enumerable = Vec::with_capacity(keys.len());
    for key in keys {
        if key.is_symbol() {
            continue;
        }
        let descriptor = { proxy_get_own_property(cx, object, key) };
        let Some(descriptor) = descriptor? else {
            continue;
        };
        if descriptor.enumerable() == Some(true) {
            enumerable.push(key);
        }
    }
    Ok(enumerable)
}

fn json_reviver_context_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    metadata: Option<&JsonParseNode>,
    value: Value,
) -> Result<Value, Cx::Error> {
    let prototype = json_object_prototype(cx)?;
    let context =
        cx.allocate_ordinary_object_with_prototype(cx.builtin_realm(), Some(prototype))?;
    if let Some(source_units) = metadata.and_then(|node| node.source_units_for_value(value)) {
        let source_key = property_key_from_text(cx, "source");
        let source_value = string_from_code_units(cx, source_units);
        create_data_property_or_throw(cx, context, source_key, source_value)?;
    }
    Ok(Value::from_object_ref(context))
}

fn json_internalize_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    holder: ObjectRef,
    name: PropertyKey,
    reviver: ObjectRef,
    metadata: Option<&JsonParseNode>,
) -> Result<Value, Cx::Error> {
    let value = get_property_from_object(cx, holder, name)?;
    if let Some(object) = value.as_object_ref() {
        if is_engine_array(cx, object) {
            let array_elements = metadata.and_then(|node| node.array_elements_for_object(object));
            let length = array_like_length(cx, object)?;
            for index in 0..length {
                let key = array_like_index_property_key(cx, u64::from(index));
                let child_metadata =
                    array_elements.and_then(|elements| elements.get(index as usize));
                let replacement =
                    json_internalize_property(cx, object, key, reviver, child_metadata)?;
                if replacement.is_undefined() {
                    let _ = try_delete_property_from_object(cx, object, key)?;
                } else {
                    let _ = try_create_data_property(cx, object, key, replacement)?;
                }
            }
        } else {
            let keys = json_enumerable_string_keys(cx, object)?;
            for key in keys {
                let child_metadata =
                    metadata.and_then(|node| node.object_entry_for_key(object, key));
                let replacement =
                    json_internalize_property(cx, object, key, reviver, child_metadata)?;
                if replacement.is_undefined() {
                    let _ = try_delete_property_from_object(cx, object, key)?;
                } else {
                    let _ = try_create_data_property(cx, object, key, replacement)?;
                }
            }
        }
    }

    let name_value = property_key_string_value(cx, name);
    let context = json_reviver_context_object(cx, metadata, value)?;
    cx.call_to_completion(
        reviver,
        Value::from_object_ref(holder),
        &[name_value, value, context],
    )
}

fn json_parse_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let text = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let text = to_string_string_ref(cx, text)?;
    let text_units = string_ref_code_units(cx, text)?;
    let mut parser = JsonParser::new(&text_units);
    let unfiltered = parser.parse_text(cx)?;

    let Some(reviver) = invocation
        .arguments()
        .get(1)
        .copied()
        .and_then(|value| callable_object_from_value(cx, value))
    else {
        return Ok(unfiltered.value());
    };

    let object_prototype = json_object_prototype(cx)?;
    let root =
        cx.allocate_ordinary_object_with_prototype(cx.builtin_realm(), Some(object_prototype))?;
    let root_name = property_key_from_text(cx, "");
    create_data_property_or_throw(cx, root, root_name, unfiltered.value())?;
    json_internalize_property(cx, root, root_name, reviver, Some(&unfiltered))
}

fn json_stringify_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let replacer = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let replacer_function = callable_object_from_value(cx, replacer);
    let property_list = if replacer_function.is_some() {
        None
    } else {
        json_property_list_from_replacer(cx, replacer)?
    };
    let gap = json_gap_from_value(
        cx,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let object_prototype = json_object_prototype(cx)?;
    let wrapper =
        cx.allocate_ordinary_object_with_prototype(cx.builtin_realm(), Some(object_prototype))?;
    let root_name = property_key_from_text(cx, "");
    create_data_property_or_throw(cx, wrapper, root_name, value)?;
    let mut state = JsonStringifyState {
        stack: Vec::new(),
        indent: String::new(),
        gap,
        replacer_function,
        property_list,
    };
    let Some(text) = json_serialize_property(cx, wrapper, root_name, &mut state)? else {
        return Ok(Value::undefined());
    };
    Ok(string_value(cx, &text))
}

struct JsonStringifyState {
    stack: Vec<ObjectRef>,
    indent: String,
    gap: String,
    replacer_function: Option<ObjectRef>,
    property_list: Option<Vec<PropertyKey>>,
}

fn json_property_list_from_replacer<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Option<Vec<PropertyKey>>, Cx::Error> {
    let Some(object) = value.as_object_ref() else {
        return Ok(None);
    };
    if !is_engine_array(cx, object) {
        return Ok(None);
    }

    let length = array_like_length(cx, object)?;
    let mut property_list = Vec::new();
    for index in 0..length {
        let item = get_property_from_object(cx, object, PropertyKey::Index(index))?;
        let Some(key) = json_property_list_item(cx, item)? else {
            continue;
        };
        if !property_list.contains(&key) {
            property_list.push(key);
        }
    }
    Ok(Some(property_list))
}

fn json_property_list_item<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Option<PropertyKey>, Cx::Error> {
    let include = value.as_string_ref().is_some()
        || value.as_smi().is_some()
        || value.as_f64().is_some()
        || value.as_object_ref().is_some_and(|object| {
            matches!(
                cx.agent().objects().primitive_wrapper_kind(object),
                Some(PrimitiveWrapperKind::String | PrimitiveWrapperKind::Number)
            )
        });
    if !include {
        return Ok(None);
    }
    let item = to_string_string_ref(cx, value)?;
    let key = cx.to_property_key(Value::from_string_ref(item))?;
    Ok(Some(key))
}

fn json_gap_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    mut value: Value,
) -> Result<String, Cx::Error> {
    if let Some(object) = value.as_object_ref() {
        value = match cx.agent().objects().primitive_wrapper_kind(object) {
            Some(PrimitiveWrapperKind::Number) => to_number_value_for_builtin(cx, value)?,
            Some(PrimitiveWrapperKind::String) => {
                Value::from_string_ref(to_string_string_ref(cx, value)?)
            }
            _ => Value::undefined(),
        };
    }

    if value.as_smi().is_some() || value.as_f64().is_some() {
        let count = to_integer_or_infinity_for_builtin(cx, value)?;
        let count = if count <= 0.0 {
            0
        } else {
            usize::try_from(count.min(10.0) as u64).unwrap_or(10)
        };
        return Ok(" ".repeat(count));
    }

    let Some(string) = value.as_string_ref() else {
        return Ok(String::new());
    };
    let mut units = string_ref_code_units(cx, string)?;
    units.truncate(10);
    Ok(String::from_utf16_lossy(&units))
}

fn json_normalize_wrapper_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    object: ObjectRef,
) -> Result<Option<Value>, Cx::Error> {
    let boolean_value = {
        let agent = cx.agent();
        agent
            .objects()
            .primitive_wrapper_value(agent.heap().view(), object)
    };
    let normalized = match cx.agent().objects().primitive_wrapper_kind(object) {
        Some(PrimitiveWrapperKind::Number) => Some(to_number_value_for_builtin(cx, value)?),
        Some(PrimitiveWrapperKind::String) => {
            Some(Value::from_string_ref(to_string_string_ref(cx, value)?))
        }
        Some(PrimitiveWrapperKind::Boolean) => boolean_value,
        _ => None,
    };
    Ok(normalized)
}

fn json_raw_json_string<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
) -> Result<Option<String>, Cx::Error> {
    let raw_json_string = {
        let agent = cx.agent();
        if !agent.objects().is_json_raw_object(object) {
            None
        } else {
            agent
                .objects()
                .ordinary_payload_value(agent.heap().view(), object)
                .and_then(Value::as_string_ref)
        }
    };
    raw_json_string
        .map(|string| string_ref_text(cx, string))
        .transpose()
}

fn json_serialize_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    holder: ObjectRef,
    key: PropertyKey,
    state: &mut JsonStringifyState,
) -> Result<Option<String>, Cx::Error> {
    let mut value = get_property_from_object(cx, holder, key)?;
    if let Some(object) = value.as_object_ref() {
        let to_json_key = property_key_from_text(cx, "toJSON");
        let to_json = get_property_from_object(cx, object, to_json_key)?;
        if let Some(to_json) = callable_object_from_value(cx, to_json) {
            let key_value = property_key_string_value(cx, key);
            value = cx.call_to_completion(to_json, Value::from_object_ref(object), &[key_value])?;
        }
    }
    if let Some(replacer) = state.replacer_function {
        let key_value = property_key_string_value(cx, key);
        value = cx.call_to_completion(
            replacer,
            Value::from_object_ref(holder),
            &[key_value, value],
        )?;
    }
    if let Some(object) = value.as_object_ref() {
        if let Some(normalized) = json_normalize_wrapper_value(cx, value, object)? {
            value = normalized;
        }
    }

    if value.is_undefined() || value.as_symbol_ref().is_some() {
        return Ok(None);
    }
    if value.is_null() {
        return Ok(Some("null".to_owned()));
    }
    if let Some(boolean) = value.as_bool() {
        return Ok(Some(if boolean { "true" } else { "false" }.to_owned()));
    }
    if value.as_smi().is_some() || value.as_f64().is_some() {
        let number = to_number_for_builtin(cx, value)?;
        if !number.is_finite() {
            return Ok(Some("null".to_owned()));
        }
        return Ok(Some(cx.value_to_string_text(value)?));
    }
    if value.is_bigint() {
        return Err(type_error(cx));
    }
    if let Some(string) = value.as_string_ref() {
        return Ok(Some(json_quote_string_ref(cx, string)?));
    }

    let Some(object) = value.as_object_ref() else {
        return Ok(None);
    };
    if let Some(raw_json) = json_raw_json_string(cx, object)? {
        return Ok(Some(raw_json));
    }

    let is_function = {
        let agent = cx.agent();
        agent
            .objects()
            .object_header(agent.heap().view(), object)
            .is_some_and(|header| header.kind() == ObjectKind::Function)
    };
    if is_function {
        return Ok(None);
    }
    if state.stack.contains(&object) {
        return Err(type_error(cx));
    }

    state.stack.push(object);
    let result = if is_engine_array(cx, object) {
        json_serialize_array(cx, object, state)
    } else {
        json_serialize_object(cx, object, state)
    };
    let _ = state.stack.pop();
    result.map(Some)
}

fn json_serialize_array<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    state: &mut JsonStringifyState,
) -> Result<String, Cx::Error> {
    let length = array_like_length(cx, object)?;
    let stepback = state.indent.clone();
    state.indent.push_str(&state.gap);
    let indent = state.indent.clone();
    let gap_empty = state.gap.is_empty();
    let mut parts = Vec::with_capacity(usize::try_from(length).unwrap_or(usize::MAX));
    for index in 0..length {
        let key = array_like_index_property_key(cx, u64::from(index));
        parts.push(
            json_serialize_property(cx, object, key, state)?.unwrap_or_else(|| "null".to_owned()),
        );
    }
    state.indent = stepback.clone();
    if parts.is_empty() {
        return Ok("[]".to_owned());
    }
    if gap_empty {
        return Ok(format!("[{}]", parts.join(",")));
    }
    let separator = format!(",\n{}", indent);
    Ok(format!(
        "[\n{}{}\n{}]",
        indent,
        parts.join(&separator),
        stepback
    ))
}

fn json_serialize_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    state: &mut JsonStringifyState,
) -> Result<String, Cx::Error> {
    let keys = if let Some(property_list) = state.property_list.as_ref() {
        property_list.clone()
    } else {
        json_enumerable_string_keys(cx, object)?
    };
    let stepback = state.indent.clone();
    state.indent.push_str(&state.gap);
    let indent = state.indent.clone();
    let gap_empty = state.gap.is_empty();
    let mut parts = Vec::new();
    for key in keys {
        let Some(serialized) = json_serialize_property(cx, object, key, state)? else {
            continue;
        };
        let separator = if gap_empty { ":" } else { ": " };
        parts.push(format!(
            "{}{}{}",
            json_quote_property_key(cx, key)?,
            separator,
            serialized
        ));
    }
    state.indent = stepback.clone();
    if parts.is_empty() {
        return Ok("{}".to_owned());
    }
    if gap_empty {
        return Ok(format!("{{{}}}", parts.join(",")));
    }
    let separator = format!(",\n{}", indent);
    Ok(format!(
        "{{\n{}{}\n{}}}",
        indent,
        parts.join(&separator),
        stepback
    ))
}

fn json_quote_property_key<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    key: PropertyKey,
) -> Result<String, Cx::Error> {
    let name = property_key_string_value(cx, key);
    let Some(string) = name.as_string_ref() else {
        return Err(type_error(cx));
    };
    json_quote_string_ref(cx, string)
}

fn json_quote_string_ref<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    string: StringRef,
) -> Result<String, Cx::Error> {
    let units = string_ref_code_units(cx, string)?;
    Ok(json_quote_string_units(&units))
}

fn json_quote_string_units(units: &[u16]) -> String {
    let mut quoted = String::with_capacity(units.len() + 2);
    quoted.push('"');
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        match unit {
            0x0022 => quoted.push_str("\\\""),
            0x005C => quoted.push_str("\\\\"),
            0x0008 => quoted.push_str("\\b"),
            0x0009 => quoted.push_str("\\t"),
            0x000A => quoted.push_str("\\n"),
            0x000C => quoted.push_str("\\f"),
            0x000D => quoted.push_str("\\r"),
            0x0000..=0x001F => {
                let _ = write!(quoted, "\\u{:04x}", unit);
            }
            0xD800..=0xDBFF => {
                if let Some(next) = units.get(index + 1).copied() {
                    if (0xDC00..=0xDFFF).contains(&next) {
                        let scalar =
                            0x10000 + (u32::from(unit - 0xD800) << 10) + u32::from(next - 0xDC00);
                        quoted.push(
                            char::from_u32(scalar)
                                .expect("valid UTF-16 surrogate pair should decode"),
                        );
                        index += 1;
                    } else {
                        let _ = write!(quoted, "\\u{:04x}", unit);
                    }
                } else {
                    let _ = write!(quoted, "\\u{:04x}", unit);
                }
            }
            0xDC00..=0xDFFF => {
                let _ = write!(quoted, "\\u{:04x}", unit);
            }
            unit => quoted.push(
                char::from_u32(u32::from(unit))
                    .expect("basic multilingual plane unit should decode"),
            ),
        }
        index += 1;
    }
    quoted.push('"');
    quoted
}

fn array_species_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    _cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Ok(invocation.this_value())
}

fn array_concat_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let result = array_species_create(
        cx,
        object_ref,
        invocation.arguments().len().saturating_add(1),
    )?;
    let mut next_index = 0_u64;
    for value in
        std::iter::once(invocation.this_value()).chain(invocation.arguments().iter().copied())
    {
        if let Some(source_object) = value.as_object_ref() {
            if is_concat_spreadable(cx, value)? {
                let length = array_like_length_u64(cx, source_object)?;
                let Some(limit) = next_index.checked_add(length) else {
                    return Err(type_error(cx));
                };
                if limit > MAX_SAFE_INTEGER_U64 {
                    return Err(type_error(cx));
                }
                for index in 0..length {
                    let source_key = array_like_index_property_key(cx, index);
                    if has_property_on_object(cx, source_object, source_key)? {
                        let item = get_property_from_object(cx, source_object, source_key)?;
                        let target_key = array_like_index_property_key(cx, next_index);
                        set_property_on_object(cx, result, target_key, item)?;
                    }
                    next_index = next_index.saturating_add(1);
                }
                continue;
            }
        }
        if next_index >= MAX_SAFE_INTEGER_U64 {
            return Err(type_error(cx));
        }
        let target_key = array_like_index_property_key(cx, next_index);
        set_property_on_object(cx, result, target_key, value)?;
        next_index = next_index.saturating_add(1);
    }
    let result_length = u32::try_from(next_index).map_err(|_| type_error(cx))?;
    define_array_length(cx, result, result_length)?;
    Ok(Value::from_object_ref(result))
}

fn array_copy_within_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    let target = normalize_relative_index_u64(
        length,
        to_integer_or_infinity_for_builtin(
            cx,
            invocation
                .arguments()
                .first()
                .copied()
                .unwrap_or(Value::undefined()),
        )?,
    );
    let start = normalize_relative_index_u64(
        length,
        to_integer_or_infinity_for_builtin(
            cx,
            invocation
                .arguments()
                .get(1)
                .copied()
                .unwrap_or(Value::undefined()),
        )?,
    );
    let end = normalize_relative_index_u64(
        length,
        match invocation.arguments().get(2).copied() {
            Some(value) if value.is_undefined() => length as f64,
            Some(value) => to_integer_or_infinity_for_builtin(cx, value)?,
            None => length as f64,
        },
    );
    let count = end.saturating_sub(start).min(length.saturating_sub(target));
    if count == 0 {
        return Ok(Value::from_object_ref(object_ref));
    }

    let (mut from, mut to, forward) = if start < target && target < start.saturating_add(count) {
        (start + count - 1, target + count - 1, false)
    } else {
        (start, target, true)
    };
    let mut remaining = count;
    while remaining > 0 {
        let from_key = array_like_index_property_key(cx, from);
        let to_key = array_like_index_property_key(cx, to);
        if has_property_on_object(cx, object_ref, from_key)? {
            let value = get_property_from_object(cx, object_ref, from_key)?;
            set_property_on_object(cx, object_ref, to_key, value)?;
        } else {
            delete_property_from_object(cx, object_ref, to_key)?;
        }

        remaining -= 1;
        if remaining == 0 {
            break;
        }
        if forward {
            from += 1;
            to += 1;
        } else {
            from -= 1;
            to -= 1;
        }
    }
    Ok(Value::from_object_ref(object_ref))
}

fn array_fill_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    let start = normalize_relative_index_u64(
        length,
        to_integer_or_infinity_for_builtin(
            cx,
            invocation
                .arguments()
                .get(1)
                .copied()
                .unwrap_or(Value::undefined()),
        )?,
    );
    let end = match invocation.arguments().get(2).copied() {
        Some(value) if !value.is_undefined() => {
            normalize_relative_index_u64(length, to_integer_or_infinity_for_builtin(cx, value)?)
        }
        _ => length,
    };
    let fill_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    for index in start..end {
        let key = array_like_index_property_key(cx, index);
        set_property_on_object(cx, object_ref, key, fill_value)?;
    }
    Ok(Value::from_object_ref(object_ref))
}

fn array_filter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
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
    let length = array_like_length(cx, object_ref)?;
    let result = array_species_create(
        cx,
        object_ref,
        usize::try_from(length).unwrap_or(usize::MAX),
    )?;
    let mut to = 0_u32;
    for index in 0..length {
        let key = PropertyKey::Index(index);
        if !has_property_on_object(cx, object_ref, key)? {
            continue;
        }
        let value = get_property_from_object(cx, object_ref, key)?;
        let selected = cx.call_to_completion(
            callback,
            this_arg,
            &[
                value,
                length_value(index),
                Value::from_object_ref(object_ref),
            ],
        )?;
        if to_boolean_for_builtin(cx, selected)? {
            set_property_on_object(cx, result, PropertyKey::Index(to), value)?;
            to = to.saturating_add(1);
        }
    }
    define_array_length(cx, result, to)?;
    Ok(Value::from_object_ref(result))
}

fn array_for_each_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
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
    let length = array_like_length(cx, object_ref)?;
    for index in 0..length {
        let key = PropertyKey::Index(index);
        if !has_property_on_object(cx, object_ref, key)? {
            continue;
        }
        let value = get_property_from_object(cx, object_ref, key)?;
        let _ = cx.call_to_completion(
            callback,
            this_arg,
            &[
                value,
                length_value(index),
                Value::from_object_ref(object_ref),
            ],
        )?;
    }
    Ok(Value::undefined())
}

fn array_map_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
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
    let length = array_like_length_u64(cx, object_ref)?;
    let result = array_species_create_for_length(cx, object_ref, length)?;
    for index in 0..length {
        let key = array_like_index_property_key(cx, index);
        if !has_property_on_object(cx, object_ref, key)? {
            continue;
        }
        let value = get_property_from_object(cx, object_ref, key)?;
        let mapped = cx.call_to_completion(
            callback,
            this_arg,
            &[
                value,
                length_value_u64(index),
                Value::from_object_ref(object_ref),
            ],
        )?;
        set_property_on_object(cx, result, key, mapped)?;
    }
    Ok(Value::from_object_ref(result))
}

fn array_reverse_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    let middle = length / 2;
    let mut lower = 0_u64;
    while lower < middle {
        let upper = length - lower - 1;
        let lower_key = array_like_index_property_key(cx, lower);
        let upper_key = array_like_index_property_key(cx, upper);
        let lower_present = has_property_on_object(cx, object_ref, lower_key)?;
        let upper_present = has_property_on_object(cx, object_ref, upper_key)?;

        match (lower_present, upper_present) {
            (true, true) => {
                let lower_value = get_property_from_object(cx, object_ref, lower_key)?;
                let upper_value = get_property_from_object(cx, object_ref, upper_key)?;
                set_property_on_object(cx, object_ref, lower_key, upper_value)?;
                set_property_on_object(cx, object_ref, upper_key, lower_value)?;
            }
            (false, true) => {
                let upper_value = get_property_from_object(cx, object_ref, upper_key)?;
                set_property_on_object(cx, object_ref, lower_key, upper_value)?;
                delete_property_from_object(cx, object_ref, upper_key)?;
            }
            (true, false) => {
                let lower_value = get_property_from_object(cx, object_ref, lower_key)?;
                delete_property_from_object(cx, object_ref, lower_key)?;
                set_property_on_object(cx, object_ref, upper_key, lower_value)?;
            }
            (false, false) => {}
        }

        lower += 1;
    }
    Ok(Value::from_object_ref(object_ref))
}

fn array_slice_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    let start = normalize_relative_index_u64(
        length,
        to_integer_or_infinity_for_builtin(
            cx,
            invocation
                .arguments()
                .first()
                .copied()
                .unwrap_or(Value::undefined()),
        )?,
    );
    let end = normalize_relative_index_u64(
        length,
        match invocation.arguments().get(1).copied() {
            Some(value) if value.is_undefined() => length as f64,
            Some(value) => to_integer_or_infinity_for_builtin(cx, value)?,
            None => length as f64,
        },
    );
    let count = end.saturating_sub(start);
    if count > u64::from(u32::MAX) {
        return Err(range_error(cx));
    }
    let result = array_species_create(cx, object_ref, array_result_capacity_hint(count))?;
    for offset in 0..count {
        let source_key = array_like_index_property_key(cx, start.saturating_add(offset));
        if !has_property_on_object(cx, object_ref, source_key)? {
            continue;
        }
        let value = get_property_from_object(cx, object_ref, source_key)?;
        let target_index = u32::try_from(offset).expect("slice result length already validated");
        set_property_on_object(cx, result, PropertyKey::Index(target_index), value)?;
    }
    define_array_length(
        cx,
        result,
        u32::try_from(count).expect("slice result length already validated"),
    )?;
    Ok(Value::from_object_ref(result))
}

fn compare_array_sort_values<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    compare_fn: Option<lyng_js_types::ObjectRef>,
    left: Value,
    right: Value,
) -> Result<std::cmp::Ordering, Cx::Error> {
    if let Some(compare_fn) = compare_fn {
        let compared = cx.call_to_completion(compare_fn, Value::undefined(), &[left, right])?;
        let number = to_number_for_builtin(cx, compared)?;
        return Ok(if number.is_nan() || number == 0.0 {
            std::cmp::Ordering::Equal
        } else if number < 0.0 {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        });
    }
    let left_text = cx.value_to_string_text(left)?;
    let right_text = cx.value_to_string_text(right)?;
    Ok(left_text.cmp(&right_text))
}

fn array_sort_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length(cx, object_ref)?;
    let compare_fn = match invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined())
    {
        value if value.is_undefined() => None,
        value => Some(cx.require_callable_object(value)?),
    };
    let entries = snapshot_indexed_entries(cx, object_ref, length)?;
    let mut present = entries.into_iter().flatten().collect::<Vec<_>>();
    let present_len = present.len();
    for i in 1..present_len {
        let mut j = i;
        while j > 0
            && compare_array_sort_values(cx, compare_fn, present[j - 1], present[j])?
                == std::cmp::Ordering::Greater
        {
            present.swap(j - 1, j);
            j -= 1;
        }
    }
    let mut sorted = present.into_iter().map(Some).collect::<Vec<_>>();
    sorted.resize(usize::try_from(length).unwrap_or(usize::MAX), None);
    store_indexed_entries(cx, object_ref, &sorted)?;
    Ok(Value::from_object_ref(object_ref))
}

fn array_splice_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let arguments = invocation.arguments();
    let length = array_like_length_u64(cx, object_ref)?;
    let start = normalize_relative_index_u64(
        length,
        to_integer_or_infinity_for_builtin(
            cx,
            arguments.first().copied().unwrap_or(Value::undefined()),
        )?,
    );
    let insert_count = u64::try_from(arguments.len().saturating_sub(2)).unwrap_or(u64::MAX);
    let delete_count = if arguments.is_empty() {
        0
    } else if arguments.len() == 1 {
        length.saturating_sub(start)
    } else {
        let requested = to_integer_or_infinity_for_builtin(
            cx,
            arguments.get(1).copied().unwrap_or(Value::undefined()),
        )?;
        if requested <= 0.0 {
            0
        } else {
            requested.min(length.saturating_sub(start) as f64) as u64
        }
    };
    let items = if arguments.len() > 2 {
        &arguments[2..]
    } else {
        &[]
    };
    let Some(new_length) = length
        .checked_add(insert_count)
        .and_then(|value| value.checked_sub(delete_count))
    else {
        return Err(type_error(cx));
    };
    if new_length > MAX_SAFE_INTEGER_U64 {
        return Err(type_error(cx));
    }

    let removed = array_species_create_for_length(cx, object_ref, delete_count)?;
    for offset in 0..delete_count {
        let from_key = array_like_index_property_key(cx, start.saturating_add(offset));
        if !has_property_on_object(cx, object_ref, from_key)? {
            continue;
        }
        let value = get_property_from_object(cx, object_ref, from_key)?;
        let to_key = array_like_index_property_key(cx, offset);
        create_data_property_or_throw(cx, removed, to_key, value)?;
    }
    set_length_property(cx, removed, delete_count)?;

    if insert_count < delete_count {
        let mut index = start;
        let shift_limit = length - delete_count;
        while index < shift_limit {
            let from_key = array_like_index_property_key(cx, index + delete_count);
            let to_key = array_like_index_property_key(cx, index + insert_count);
            if has_property_on_object(cx, object_ref, from_key)? {
                let value = get_property_from_object(cx, object_ref, from_key)?;
                set_property_on_object(cx, object_ref, to_key, value)?;
            } else {
                delete_property_from_object(cx, object_ref, to_key)?;
            }
            index += 1;
        }

        let mut index = length;
        let delete_from = length - delete_count + insert_count;
        while index > delete_from {
            let key = array_like_index_property_key(cx, index - 1);
            delete_property_from_object(cx, object_ref, key)?;
            index -= 1;
        }
    } else if insert_count > delete_count {
        let mut index = length - delete_count;
        while index > start {
            let from_key = array_like_index_property_key(cx, index + delete_count - 1);
            let to_key = array_like_index_property_key(cx, index + insert_count - 1);
            if has_property_on_object(cx, object_ref, from_key)? {
                let value = get_property_from_object(cx, object_ref, from_key)?;
                set_property_on_object(cx, object_ref, to_key, value)?;
            } else {
                delete_property_from_object(cx, object_ref, to_key)?;
            }
            index -= 1;
        }
    }

    for (offset, value) in items.iter().copied().enumerate() {
        let key = array_like_index_property_key(
            cx,
            start.saturating_add(u64::try_from(offset).unwrap_or(u64::MAX)),
        );
        set_property_on_object(cx, object_ref, key, value)?;
    }
    set_length_property(cx, object_ref, new_length)?;
    Ok(Value::from_object_ref(removed))
}

fn array_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let join_key = property_key_from_text(cx, "join");
    let join_value = cx.get_property_value(invocation.this_value(), join_key)?;
    let join = if let Some(object) = join_value.as_object_ref() {
        let is_callable = {
            let agent = cx.agent();
            agent.objects().is_callable(object)
        };
        is_callable.then_some(object)
    } else {
        None
    };
    if let Some(join) = join {
        return cx.call_to_completion(join, invocation.this_value(), &[]);
    }
    object_to_string_builtin(cx, invocation)
}

fn array_join_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let separator_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let separator = if separator_value.is_undefined() {
        ",".to_owned()
    } else {
        cx.value_to_string_text(separator_value)?
    };
    let text = array_like_join_text(cx, object_ref, &separator)?;
    Ok(string_value(cx, &text))
}

fn array_shift_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    if length == 0 {
        set_property_on_object(
            cx,
            object_ref,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
            Value::from_smi(0),
        )?;
        return Ok(Value::undefined());
    }

    let first_key = array_like_index_property_key(cx, 0);
    let first = get_property_from_object(cx, object_ref, first_key)?;
    for index in 1..length {
        let from = array_like_index_property_key(cx, index);
        let to = array_like_index_property_key(cx, index - 1);
        if has_property_on_object(cx, object_ref, from)? {
            let value = get_property_from_object(cx, object_ref, from)?;
            set_property_on_object(cx, object_ref, to, value)?;
        } else {
            delete_property_from_object(cx, object_ref, to)?;
        }
    }

    let last = array_like_index_property_key(cx, length - 1);
    delete_property_from_object(cx, object_ref, last)?;
    let new_length = length - 1;
    let length_value = if new_length <= u64::from(i32::MAX as u32) {
        Value::from_smi(i32::try_from(new_length).unwrap_or(i32::MAX))
    } else {
        Value::from_f64(new_length as f64)
    };
    set_property_on_object(
        cx,
        object_ref,
        PropertyKey::from_atom(WellKnownAtom::length.id()),
        length_value,
    )?;
    Ok(first)
}

fn array_unshift_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    let item_count = u64::try_from(invocation.arguments().len()).unwrap_or(u64::MAX);
    let Some(new_length) = length.checked_add(item_count) else {
        return Err(type_error(cx));
    };
    if new_length > MAX_SAFE_INTEGER_U64 {
        return Err(type_error(cx));
    }

    if item_count > 0 {
        let mut index = length;
        while index > 0 {
            let from_index = index - 1;
            let from = array_like_index_property_key(cx, from_index);
            let to = array_like_index_property_key(cx, from_index + item_count);
            if has_property_on_object(cx, object_ref, from)? {
                let value = get_property_from_object(cx, object_ref, from)?;
                set_property_on_object(cx, object_ref, to, value)?;
            } else {
                delete_property_from_object(cx, object_ref, to)?;
            }
            index -= 1;
        }

        for (index, value) in invocation.arguments().iter().copied().enumerate() {
            let key = array_like_index_property_key(cx, u64::try_from(index).unwrap_or(u64::MAX));
            set_property_on_object(cx, object_ref, key, value)?;
        }
    }

    set_length_property(cx, object_ref, new_length)?;
    Ok(length_value_u64(new_length))
}

fn array_to_locale_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length(cx, object_ref)?;
    let to_locale_string_key = property_key_from_text(cx, "toLocaleString");
    let mut parts = Vec::with_capacity(usize::try_from(length).unwrap_or(usize::MAX));
    for index in 0..length {
        let key = PropertyKey::Index(index);
        let text = if !has_property_on_object(cx, object_ref, key)? {
            String::new()
        } else {
            let value = get_property_from_object(cx, object_ref, key)?;
            if value.is_undefined() || value.is_null() {
                String::new()
            } else {
                let method_value = cx.get_property_value(value, to_locale_string_key)?;
                let method = cx.require_callable_object(method_value)?;
                let result = cx.call_to_completion(method, value, &[])?;
                cx.value_to_string_text(result)?
            }
        };
        parts.push(text);
    }
    Ok(string_value(cx, &parts.join(",")))
}

fn array_iterator_factory_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: ArrayIterationKind,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let prototype = {
        let realm = cx.builtin_realm();
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().array_iterator_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let slot_values = [
        Value::from_object_ref(object_ref),
        Value::from_smi(0),
        kind.into_value(),
    ];
    let iterator_object = allocate_iterator_object(
        cx,
        prototype,
        OrdinaryObjectData::ArrayIterator,
        &slot_values,
    )?;
    Ok(Value::from_object_ref(iterator_object))
}

fn typed_array_iterator_factory_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: ArrayIterationKind,
) -> Result<Value, Cx::Error> {
    let (object_ref, _) = typed_array_validated_object_and_record(cx, invocation.this_value())?;
    let prototype = {
        let realm = cx.builtin_realm();
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().array_iterator_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let slot_values = [
        Value::from_object_ref(object_ref),
        Value::from_smi(0),
        kind.into_value(),
    ];
    let iterator_object = allocate_iterator_object(
        cx,
        prototype,
        OrdinaryObjectData::ArrayIterator,
        &slot_values,
    )?;
    Ok(Value::from_object_ref(iterator_object))
}

fn map_iterator_factory_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: ArrayIterationKind,
) -> Result<Value, Cx::Error> {
    let object_ref = map_this_object(cx, invocation.this_value())?;
    let prototype = {
        let realm = cx.builtin_realm();
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().map_iterator_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let slot_values = [
        Value::from_object_ref(object_ref),
        Value::from_smi(0),
        kind.into_value(),
    ];
    let iterator_object =
        allocate_iterator_object(cx, prototype, OrdinaryObjectData::MapIterator, &slot_values)?;
    Ok(Value::from_object_ref(iterator_object))
}

fn set_iterator_factory_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: ArrayIterationKind,
) -> Result<Value, Cx::Error> {
    let object_ref = set_this_object(cx, invocation.this_value())?;
    let prototype = {
        let realm = cx.builtin_realm();
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().set_iterator_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let slot_values = [
        Value::from_object_ref(object_ref),
        Value::from_smi(0),
        kind.into_value(),
    ];
    let iterator_object =
        allocate_iterator_object(cx, prototype, OrdinaryObjectData::SetIterator, &slot_values)?;
    Ok(Value::from_object_ref(iterator_object))
}

fn iterator_prototype_iterator_builtin<Cx: PublicBuiltinDispatchContext>(
    _cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Ok(invocation.this_value())
}

fn array_iterator_next_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let iterator_object = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let target = iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::ArrayIterator,
        ARRAY_ITERATOR_TARGET_SLOT,
    )?;
    let Some(target_object) = target.as_object_ref() else {
        return create_iterator_result_value(cx, Value::undefined(), true);
    };
    let index = iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::ArrayIterator,
        ARRAY_ITERATOR_INDEX_SLOT,
    )?
    .as_smi()
    .and_then(|value| u32::try_from(value).ok())
    .ok_or_else(|| type_error(cx))?;
    let kind = ArrayIterationKind::from_value(iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::ArrayIterator,
        ARRAY_ITERATOR_KIND_SLOT,
    )?)
    .ok_or_else(|| type_error(cx))?;
    let length = array_like_length(cx, target_object)?;
    if index >= length {
        set_iterator_slot_value_for_builtin(
            cx,
            iterator_object,
            OrdinaryObjectData::ArrayIterator,
            ARRAY_ITERATOR_TARGET_SLOT,
            Value::undefined(),
        )?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    }
    set_iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::ArrayIterator,
        ARRAY_ITERATOR_INDEX_SLOT,
        length_value(index.saturating_add(1)),
    )?;
    let value = match kind {
        ArrayIterationKind::Key => length_value(index),
        ArrayIterationKind::Value => {
            get_property_from_object(cx, target_object, PropertyKey::Index(index))?
        }
        ArrayIterationKind::Entry => {
            let pair = create_array_result(cx, 2)?;
            let entry_value =
                get_property_from_object(cx, target_object, PropertyKey::Index(index))?;
            set_property_on_object(cx, pair, PropertyKey::Index(0), length_value(index))?;
            set_property_on_object(cx, pair, PropertyKey::Index(1), entry_value)?;
            Value::from_object_ref(pair)
        }
    };
    create_iterator_result_value(cx, value, false)
}

fn map_iterator_next_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let iterator_object = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let target = iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::MapIterator,
        MAP_ITERATOR_TARGET_SLOT,
    )?;
    let Some(target_object) = target.as_object_ref() else {
        return create_iterator_result_value(cx, Value::undefined(), true);
    };
    let index = iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::MapIterator,
        MAP_ITERATOR_INDEX_SLOT,
    )?
    .as_smi()
    .and_then(|value| usize::try_from(value).ok())
    .ok_or_else(|| type_error(cx))?;
    let kind = ArrayIterationKind::from_value(iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::MapIterator,
        MAP_ITERATOR_KIND_SLOT,
    )?)
    .ok_or_else(|| type_error(cx))?;
    let next_entry = {
        let agent = cx.agent();
        let Some(map) = agent.objects().map(target_object) else {
            return Err(type_error(cx));
        };
        map.entries()
            .iter()
            .enumerate()
            .skip(index)
            .find_map(|(entry_index, entry)| entry.map(|entry| (entry_index, entry)))
    };
    let Some((entry_index, entry)) = next_entry else {
        set_iterator_slot_value_for_builtin(
            cx,
            iterator_object,
            OrdinaryObjectData::MapIterator,
            MAP_ITERATOR_TARGET_SLOT,
            Value::undefined(),
        )?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    };
    set_iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::MapIterator,
        MAP_ITERATOR_INDEX_SLOT,
        length_value(u32::try_from(entry_index.saturating_add(1)).unwrap_or(u32::MAX)),
    )?;
    let value = match kind {
        ArrayIterationKind::Key => entry.key(),
        ArrayIterationKind::Value => entry.value(),
        ArrayIterationKind::Entry => {
            let pair = create_array_result(cx, 2)?;
            set_property_on_object(cx, pair, PropertyKey::Index(0), entry.key())?;
            set_property_on_object(cx, pair, PropertyKey::Index(1), entry.value())?;
            Value::from_object_ref(pair)
        }
    };
    create_iterator_result_value(cx, value, false)
}

fn set_iterator_next_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let iterator_object = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let target = iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::SetIterator,
        SET_ITERATOR_TARGET_SLOT,
    )?;
    let Some(target_object) = target.as_object_ref() else {
        return create_iterator_result_value(cx, Value::undefined(), true);
    };
    let index = iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::SetIterator,
        SET_ITERATOR_INDEX_SLOT,
    )?
    .as_smi()
    .and_then(|value| usize::try_from(value).ok())
    .ok_or_else(|| type_error(cx))?;
    let kind = ArrayIterationKind::from_value(iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::SetIterator,
        SET_ITERATOR_KIND_SLOT,
    )?)
    .ok_or_else(|| type_error(cx))?;
    let next_entry = {
        let agent = cx.agent();
        let Some(set) = agent.objects().set_object_data(target_object) else {
            return Err(type_error(cx));
        };
        set.entries()
            .iter()
            .enumerate()
            .skip(index)
            .find_map(|(entry_index, entry)| entry.map(|entry| (entry_index, entry)))
    };
    let Some((entry_index, entry)) = next_entry else {
        set_iterator_slot_value_for_builtin(
            cx,
            iterator_object,
            OrdinaryObjectData::SetIterator,
            SET_ITERATOR_TARGET_SLOT,
            Value::undefined(),
        )?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    };
    set_iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::SetIterator,
        SET_ITERATOR_INDEX_SLOT,
        length_value(u32::try_from(entry_index.saturating_add(1)).unwrap_or(u32::MAX)),
    )?;
    let value = match kind {
        ArrayIterationKind::Key | ArrayIterationKind::Value => entry,
        ArrayIterationKind::Entry => {
            let pair = create_array_result(cx, 2)?;
            set_property_on_object(cx, pair, PropertyKey::Index(0), entry)?;
            set_property_on_object(cx, pair, PropertyKey::Index(1), entry)?;
            Value::from_object_ref(pair)
        }
    };
    create_iterator_result_value(cx, value, false)
}

fn string_iterator_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string_ref = string_this_ref(cx, invocation.this_value())?;
    let prototype = {
        let realm = cx.builtin_realm();
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().string_iterator_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let slot_values = [Value::from_string_ref(string_ref), Value::from_smi(0)];
    let iterator_object = allocate_iterator_object(
        cx,
        prototype,
        OrdinaryObjectData::StringIterator,
        &slot_values,
    )?;
    Ok(Value::from_object_ref(iterator_object))
}

fn string_iterator_next_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let iterator_object = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let source = iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::StringIterator,
        STRING_ITERATOR_STRING_SLOT,
    )?;
    let Some(string_ref) = source.as_string_ref() else {
        return create_iterator_result_value(cx, Value::undefined(), true);
    };
    let index = iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::StringIterator,
        STRING_ITERATOR_INDEX_SLOT,
    )?
    .as_smi()
    .and_then(|value| usize::try_from(value).ok())
    .ok_or_else(|| type_error(cx))?;
    let units = string_ref_code_units(cx, string_ref)?;
    if index >= units.len() {
        set_iterator_slot_value_for_builtin(
            cx,
            iterator_object,
            OrdinaryObjectData::StringIterator,
            STRING_ITERATOR_STRING_SLOT,
            Value::undefined(),
        )?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    }
    let mut next_index = index + 1;
    let first = units[index];
    if (0xD800..=0xDBFF).contains(&first)
        && units
            .get(index + 1)
            .is_some_and(|second| (0xDC00..=0xDFFF).contains(second))
    {
        next_index += 1;
    }
    set_iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::StringIterator,
        STRING_ITERATOR_INDEX_SLOT,
        length_value(u32::try_from(next_index).unwrap_or(u32::MAX)),
    )?;
    let value = string_from_code_units(cx, &units[index..next_index]);
    create_iterator_result_value(cx, value, false)
}

fn function_constructor_source<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    arguments: &[Value],
) -> Result<(String, String), Cx::Error> {
    if arguments.is_empty() {
        return Ok((String::new(), String::new()));
    }
    let body_index = arguments.len().saturating_sub(1);
    let mut parameters = String::new();
    for (index, value) in arguments[..body_index].iter().copied().enumerate() {
        if index != 0 {
            parameters.push(',');
        }
        parameters.push_str(&cx.value_to_string_text(value)?);
    }
    let body = cx.value_to_string_text(arguments[body_index])?;
    Ok((parameters, body))
}

fn is_error_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
) -> Result<bool, Cx::Error> {
    let error_prototypes = {
        let agent = cx.agent();
        let mut prototypes = Vec::new();
        for realm in agent.realm_refs().iter().copied() {
            let Some(intrinsics) = agent.realm(realm).map(|record| record.intrinsics()) else {
                continue;
            };
            prototypes.extend(
                [
                    intrinsics.error_prototype(),
                    intrinsics.eval_error_prototype(),
                    intrinsics.range_error_prototype(),
                    intrinsics.reference_error_prototype(),
                    intrinsics.syntax_error_prototype(),
                    intrinsics.type_error_prototype(),
                    intrinsics.uri_error_prototype(),
                    intrinsics.aggregate_error_prototype(),
                ]
                .into_iter()
                .flatten(),
            );
        }
        prototypes
    };

    let mut current = Some(object_ref);
    while let Some(candidate) = current {
        if error_prototypes
            .iter()
            .copied()
            .any(|prototype| prototype == candidate)
        {
            return Ok(true);
        }
        let next = { proxy_get_prototype_of(cx, candidate) };
        current = next?;
    }
    Ok(false)
}

fn object_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let argument = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if let Some(object) = argument.as_object_ref() {
        return Ok(Value::from_object_ref(object));
    }
    let realm = cx.builtin_realm();
    if argument.is_null() || argument.is_undefined() {
        let default_prototype = {
            let agent = cx.agent();
            agent
                .realm(realm)
                .and_then(|record| record.intrinsics().object_prototype())
        }
        .ok_or_else(|| type_error(cx))?;
        let prototype =
            cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?;
        let object = cx.allocate_ordinary_object_with_prototype(realm, Some(prototype))?;
        return Ok(Value::from_object_ref(object));
    }
    Ok(Value::from_object_ref(
        cx.to_object_for_builtin_value(realm, argument)?,
    ))
}

fn object_create_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let prototype_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let prototype = if prototype_value.is_null() {
        None
    } else if let Some(object) = prototype_value.as_object_ref() {
        Some(object)
    } else {
        return Err(type_error(cx));
    };
    let object = cx.allocate_ordinary_object_with_prototype(cx.builtin_realm(), prototype)?;
    if let Some(properties) = invocation.arguments().get(1).copied() {
        if !properties.is_undefined() {
            define_properties_from_source(cx, object, properties)?;
        }
    }
    Ok(Value::from_object_ref(object))
}

fn define_properties_from_source<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    target: lyng_js_types::ObjectRef,
    properties: Value,
) -> Result<(), Cx::Error> {
    let props = cx.to_object_for_builtin_value(cx.builtin_realm(), properties)?;
    let keys = { proxy_own_property_keys(cx, props) };
    let keys = keys?;
    let mut descriptors = Vec::with_capacity(keys.len());

    for key in keys {
        let property = { proxy_get_own_property(cx, props, key) };
        let Some(property) = property? else {
            continue;
        };
        if property.enumerable() != Some(true) {
            continue;
        }

        let descriptor_value = cx.get_property_value(Value::from_object_ref(props), key)?;
        let descriptor_object = descriptor_value
            .as_object_ref()
            .ok_or_else(|| type_error(cx))?;
        let descriptor = cx.to_property_descriptor(descriptor_object)?;
        descriptors.push((key, descriptor));
    }

    for (key, descriptor) in descriptors {
        define_property_or_throw_builtin(cx, target, key, descriptor)?;
    }

    Ok(())
}

fn define_property_or_throw_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    target: lyng_js_types::ObjectRef,
    key: PropertyKey,
    mut descriptor: PropertyDescriptor,
) -> Result<(), Cx::Error> {
    if is_engine_array(cx, target) && key == PropertyKey::from_atom(WellKnownAtom::length.id()) {
        descriptor = normalize_engine_array_length_descriptor(cx, descriptor)?;
    }
    if let Some(index) = key.as_index() {
        let typed_array = cx.agent().objects().typed_array(target);
        if let Some(record) = typed_array {
            let element_index = usize::try_from(index).unwrap_or(usize::MAX);
            if element_index >= record.length()
                || cx
                    .agent()
                    .backing_store_is_detached(record.backing_store())
                    .ok_or_else(|| type_error(cx))?
                || descriptor.has_get()
                || descriptor.has_set()
                || descriptor.configurable() == Some(false)
                || descriptor.enumerable() == Some(false)
                || descriptor.writable() == Some(false)
            {
                return Err(type_error(cx));
            }
            if let Some(value) = descriptor.value() {
                let bits = typed_array_storage_bits_from_builtin_value(cx, record.kind(), value)?;
                if cx
                    .agent()
                    .backing_store_is_detached(record.backing_store())
                    .ok_or_else(|| type_error(cx))?
                {
                    return Err(type_error(cx));
                }
                typed_array_write_storage_bits(cx, record, element_index, bits)?;
            }
            return Ok(());
        }
    }
    let defined =
        { proxy_define_property(cx, target, key, descriptor, AllocationLifetime::Default) };
    if !defined? {
        return Err(type_error(cx));
    }
    Ok(())
}

fn object_get_prototype_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let object = cx.to_object_for_builtin_value(cx.builtin_realm(), value)?;
    Ok(proxy_get_prototype_of(cx, object)?.map_or(Value::null(), Value::from_object_ref))
}

fn object_set_prototype_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let object = cx.to_object_for_builtin_value(cx.builtin_realm(), value)?;
    let prototype_value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let prototype = if prototype_value.is_null() {
        None
    } else if let Some(object) = prototype_value.as_object_ref() {
        Some(object)
    } else {
        return Err(type_error(cx));
    };
    if !proxy_set_prototype_of(cx, object, prototype)? {
        return Err(type_error(cx));
    }
    Ok(Value::from_object_ref(object))
}

fn object_get_own_property_descriptor_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), target_value)?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let Some(descriptor) = proxy_get_own_property(cx, object_ref, key)? else {
        return Ok(Value::undefined());
    };
    cx.descriptor_object_from_descriptor(cx.builtin_realm(), descriptor)
}

fn object_get_own_property_descriptors_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let realm = cx.builtin_realm();
    let object_ref = cx.to_object_for_builtin_value(realm, target_value)?;
    let object_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|realm| realm.intrinsics().object_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let result = cx.allocate_ordinary_object_with_prototype(realm, Some(object_prototype))?;
    let keys = proxy_own_property_keys(cx, object_ref)?;

    for key in keys {
        let Some(descriptor) = proxy_get_own_property(cx, object_ref, key)? else {
            continue;
        };
        let descriptor_value = cx.descriptor_object_from_descriptor(realm, descriptor)?;
        create_data_property_or_throw(cx, result, key, descriptor_value)?;
    }

    Ok(Value::from_object_ref(result))
}

fn own_property_name_list_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    enumerable_only: bool,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), value)?;
    let keys = proxy_own_property_keys(cx, object_ref)?;
    let mut names = Vec::with_capacity(keys.len());

    for key in keys {
        if key.is_symbol() {
            continue;
        }
        if enumerable_only {
            let Some(descriptor) = proxy_get_own_property(cx, object_ref, key)? else {
                continue;
            };
            if descriptor.enumerable() != Some(true) {
                continue;
            }
        }
        names.push(key);
    }

    let result = create_array_result(cx, names.len())?;
    for (index, key) in names.into_iter().enumerate() {
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        let value = property_key_string_value(cx, key);
        set_property_on_object(cx, result, PropertyKey::Index(index), value)?;
    }
    Ok(Value::from_object_ref(result))
}

fn object_get_own_property_names_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    own_property_name_list_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
        false,
    )
}

fn object_get_own_property_symbols_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(
        cx.builtin_realm(),
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let keys = proxy_own_property_keys(cx, object_ref)?;
    let symbols: Vec<_> = keys
        .into_iter()
        .filter_map(PropertyKey::as_symbol)
        .collect();

    let result = create_array_result(cx, symbols.len())?;
    for (index, symbol) in symbols.into_iter().enumerate() {
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        set_property_on_object(
            cx,
            result,
            PropertyKey::Index(index),
            Value::from_symbol_ref(symbol),
        )?;
    }
    Ok(Value::from_object_ref(result))
}

fn object_define_property_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let object_ref = target_value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let descriptor_object = invocation
        .arguments()
        .get(2)
        .copied()
        .and_then(Value::as_object_ref)
        .ok_or_else(|| type_error(cx))?;
    let descriptor = cx.to_property_descriptor(descriptor_object)?;
    define_property_or_throw_builtin(cx, object_ref, key, descriptor)?;
    Ok(Value::from_object_ref(object_ref))
}

fn object_define_properties_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), target_value)?;
    let properties = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    define_properties_from_source(cx, object_ref, properties)?;
    Ok(Value::from_object_ref(object_ref))
}

fn object_prevent_extensions_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(object_ref) = value.as_object_ref() else {
        return Ok(value);
    };
    if !proxy_prevent_extensions(cx, object_ref)? {
        return Err(type_error(cx));
    }
    Ok(Value::from_object_ref(object_ref))
}

fn object_is_extensible_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(object_ref) = value.as_object_ref() else {
        return Ok(Value::from_bool(false));
    };
    Ok(Value::from_bool(proxy_is_extensible(cx, object_ref)?))
}

fn object_is_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let right = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let same = {
        let agent = cx.agent();
        read::same_value(agent.heap().view(), left, right)
    };
    Ok(Value::from_bool(map_completion(cx, same)?))
}

fn object_seal_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(object_ref) = value.as_object_ref() else {
        return Ok(value);
    };
    let _ = cx.set_integrity_level(object_ref, false)?;
    Ok(Value::from_object_ref(object_ref))
}

fn object_freeze_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(object_ref) = value.as_object_ref() else {
        return Ok(value);
    };
    let _ = cx.set_integrity_level(object_ref, true)?;
    Ok(Value::from_object_ref(object_ref))
}

fn object_is_sealed_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(object_ref) = value.as_object_ref() else {
        return Ok(Value::from_bool(true));
    };
    Ok(Value::from_bool(
        cx.test_integrity_level(object_ref, false)?,
    ))
}

fn object_is_frozen_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(object_ref) = value.as_object_ref() else {
        return Ok(Value::from_bool(true));
    };
    Ok(Value::from_bool(cx.test_integrity_level(object_ref, true)?))
}

fn object_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Ok(Value::from_object_ref(cx.to_object_for_builtin_value(
        cx.builtin_realm(),
        invocation.this_value(),
    )?))
}

fn object_to_locale_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.this_value().is_undefined() || invocation.this_value().is_null() {
        return Err(type_error(cx));
    }
    let key = PropertyKey::from_atom(WellKnownAtom::toString.id());
    let method_value = cx.get_property_value(invocation.this_value(), key)?;
    let method = cx.require_callable_object(method_value)?;
    cx.call_to_completion(method, invocation.this_value(), &[])
}

fn object_is_prototype_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let prototype_object =
        cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let Some(mut current) = invocation
        .arguments()
        .first()
        .copied()
        .and_then(Value::as_object_ref)
    else {
        return Ok(Value::from_bool(false));
    };
    loop {
        let Some(next) = proxy_get_prototype_of(cx, current)? else {
            return Ok(Value::from_bool(false));
        };
        if next == prototype_object {
            return Ok(Value::from_bool(true));
        }
        current = next;
    }
}

fn object_property_is_enumerable_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(
        proxy_get_own_property(cx, object_ref, key)?
            .is_some_and(|descriptor| descriptor.enumerable() == Some(true)),
    ))
}

fn object_keys_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    own_property_name_list_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
        true,
    )
}

fn object_entries_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(
        cx.builtin_realm(),
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let keys = proxy_own_property_keys(cx, object_ref)?;
    let mut names = Vec::with_capacity(keys.len());

    for key in keys {
        if key.is_symbol() {
            continue;
        }
        let Some(descriptor) = proxy_get_own_property(cx, object_ref, key)? else {
            continue;
        };
        if descriptor.enumerable() != Some(true) {
            continue;
        }
        names.push(key);
    }

    let result = create_array_result(cx, names.len())?;
    for (index, key) in names.into_iter().enumerate() {
        let entry = create_array_result(cx, 2)?;
        let key_value = property_key_string_value(cx, key);
        let value = get_property_from_object(cx, object_ref, key)?;
        set_property_on_object(cx, entry, PropertyKey::Index(0), key_value)?;
        set_property_on_object(cx, entry, PropertyKey::Index(1), value)?;
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        set_property_on_object(
            cx,
            result,
            PropertyKey::Index(index),
            Value::from_object_ref(entry),
        )?;
    }
    Ok(Value::from_object_ref(result))
}

fn object_values_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(
        cx.builtin_realm(),
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let keys = proxy_own_property_keys(cx, object_ref)?;
    let mut names = Vec::with_capacity(keys.len());

    for key in keys {
        if key.is_symbol() {
            continue;
        }
        let Some(descriptor) = proxy_get_own_property(cx, object_ref, key)? else {
            continue;
        };
        if descriptor.enumerable() != Some(true) {
            continue;
        }
        names.push(key);
    }

    let result = create_array_result(cx, names.len())?;
    for (index, key) in names.into_iter().enumerate() {
        let value = get_property_from_object(cx, object_ref, key)?;
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        set_property_on_object(cx, result, PropertyKey::Index(index), value)?;
    }
    Ok(Value::from_object_ref(result))
}

fn object_has_own_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(
        cx.builtin_realm(),
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(
        proxy_get_own_property(cx, object_ref, key)?.is_some(),
    ))
}

fn object_has_own_property_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(
        proxy_get_own_property(cx, object_ref, key)?.is_some(),
    ))
}

fn reflect_apply_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target = cx.require_callable_object(
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
    let arguments_list = require_object_argument(cx, invocation, 2)?;
    let arguments = cx
        .collect_array_like_arguments(cx.builtin_realm(), Value::from_object_ref(arguments_list))?;
    cx.call_to_completion(target, this_arg, &arguments)
}

fn reflect_construct_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target = require_constructor_object(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let arguments_list = require_object_argument(cx, invocation, 1)?;
    let arguments = cx
        .collect_array_like_arguments(cx.builtin_realm(), Value::from_object_ref(arguments_list))?;
    let new_target = invocation
        .arguments()
        .get(2)
        .copied()
        .map(|value| require_constructor_object(cx, value))
        .transpose()?;
    let object = cx.construct_to_completion(target, &arguments, new_target)?;
    Ok(Value::from_object_ref(object))
}

fn reflect_define_property_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let descriptor_object = require_object_argument(cx, invocation, 2)?;
    let descriptor = cx.to_property_descriptor(descriptor_object)?;
    let defined =
        proxy_define_property(cx, object_ref, key, descriptor, AllocationLifetime::Default)?;
    Ok(Value::from_bool(defined))
}

fn reflect_delete_property_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(proxy_delete_property(
        cx, object_ref, key,
    )?))
}

fn reflect_get_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let receiver = invocation
        .arguments()
        .get(2)
        .copied()
        .unwrap_or_else(|| Value::from_object_ref(object_ref));
    get_property_from_object_with_receiver(cx, object_ref, key, receiver)
}

fn reflect_get_own_property_descriptor_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let Some(descriptor) = proxy_get_own_property(cx, object_ref, key)? else {
        return Ok(Value::undefined());
    };
    cx.descriptor_object_from_descriptor(cx.builtin_realm(), descriptor)
}

fn reflect_get_prototype_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    Ok(proxy_get_prototype_of(cx, object_ref)?.map_or(Value::null(), Value::from_object_ref))
}

fn reflect_has_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(has_property_on_object(
        cx, object_ref, key,
    )?))
}

fn reflect_is_extensible_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    Ok(Value::from_bool(proxy_is_extensible(cx, object_ref)?))
}

fn reflect_own_keys_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    let keys = proxy_own_property_keys(cx, object_ref)?;
    let result = create_array_result(cx, keys.len())?;
    for (index, key) in keys.into_iter().enumerate() {
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        let value = property_key_value(cx, key);
        set_property_on_object(cx, result, PropertyKey::Index(index), value)?;
    }
    Ok(Value::from_object_ref(result))
}

fn reflect_prevent_extensions_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    Ok(Value::from_bool(proxy_prevent_extensions(cx, object_ref)?))
}

fn reflect_set_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let value = invocation
        .arguments()
        .get(2)
        .copied()
        .unwrap_or(Value::undefined());
    let receiver = invocation
        .arguments()
        .get(3)
        .copied()
        .unwrap_or_else(|| Value::from_object_ref(object_ref));
    Ok(Value::from_bool(set_property_on_object_with_receiver(
        cx, object_ref, key, value, receiver,
    )?))
}

fn reflect_set_prototype_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    let prototype_value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let prototype = if prototype_value.is_null() {
        None
    } else if let Some(object_ref) = prototype_value.as_object_ref() {
        Some(object_ref)
    } else {
        return Err(type_error(cx));
    };
    Ok(Value::from_bool(proxy_set_prototype_of(
        cx, object_ref, prototype,
    )?))
}

fn object_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.this_value().is_undefined() {
        return Ok(string_value(cx, "[object Undefined]"));
    }
    if invocation.this_value().is_null() {
        return Ok(string_value(cx, "[object Null]"));
    }
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let default_tag = {
        let is_function = {
            let agent = cx.agent();
            agent.objects().is_callable(object_ref)
        };
        if is_function {
            "Function"
        } else {
            let is_array = {
                let agent = cx.agent();
                agent
                    .objects()
                    .object_header(agent.heap().view(), object_ref)
                    .is_some_and(|header| header.flags().is_engine_array())
            };
            if is_array {
                "Array"
            } else if {
                let agent = cx.agent();
                agent.objects().is_date_object(object_ref)
            } {
                "Date"
            } else if {
                let agent = cx.agent();
                agent.objects().is_regexp_object(object_ref)
            } {
                "RegExp"
            } else if let Some(kind) = {
                let agent = cx.agent();
                agent.objects().primitive_wrapper_kind(object_ref)
            } {
                match kind {
                    PrimitiveWrapperKind::String => "String",
                    PrimitiveWrapperKind::Number => "Number",
                    PrimitiveWrapperKind::Boolean => "Boolean",
                    PrimitiveWrapperKind::Symbol => "Symbol",
                    PrimitiveWrapperKind::BigInt => "BigInt",
                }
            } else if {
                let agent = cx.agent();
                agent
                    .objects()
                    .object_header(agent.heap().view(), object_ref)
                    .is_some_and(|header| header.flags().is_arguments_object())
            } {
                "Arguments"
            } else if is_error_object(cx, object_ref)? {
                "Error"
            } else {
                "Object"
            }
        }
    };
    let to_string_tag = {
        let key = {
            let agent = cx.agent();
            agent
                .well_known_symbol(WellKnownSymbolId::ToStringTag)
                .map(PropertyKey::from_symbol)
        };
        if let Some(key) = key {
            let value = cx.get_property_value(Value::from_object_ref(object_ref), key)?;
            if value.is_string() {
                Some(cx.value_to_string_text(value)?)
            } else {
                None
            }
        } else {
            None
        }
    };
    Ok(string_value(
        cx,
        &format!(
            "[object {}]",
            to_string_tag.as_deref().unwrap_or(default_tag)
        ),
    ))
}

fn function_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let (parameters_source, body_source) = function_constructor_source(cx, invocation.arguments())?;
    let function = cx.create_dynamic_function(
        realm,
        &parameters_source,
        &body_source,
        cx.caller_is_strict(),
        DynamicFunctionKind::Ordinary,
        invocation.new_target(),
    )?;
    Ok(Value::from_object_ref(function))
}

fn generator_function_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let (parameters_source, body_source) = function_constructor_source(cx, invocation.arguments())?;
    let function = cx.create_dynamic_function(
        realm,
        &parameters_source,
        &body_source,
        cx.caller_is_strict(),
        DynamicFunctionKind::Generator,
        invocation.new_target(),
    )?;
    Ok(Value::from_object_ref(function))
}

fn async_function_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let (parameters_source, body_source) = function_constructor_source(cx, invocation.arguments())?;
    let function = cx.create_dynamic_function(
        realm,
        &parameters_source,
        &body_source,
        cx.caller_is_strict(),
        DynamicFunctionKind::Async,
        invocation.new_target(),
    )?;
    Ok(Value::from_object_ref(function))
}

fn async_generator_function_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let (parameters_source, body_source) = function_constructor_source(cx, invocation.arguments())?;
    let function = cx.create_dynamic_function(
        realm,
        &parameters_source,
        &body_source,
        cx.caller_is_strict(),
        DynamicFunctionKind::AsyncGenerator,
        invocation.new_target(),
    )?;
    Ok(Value::from_object_ref(function))
}

fn function_prototype_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.new_target().is_some() {
        return Err(type_error(cx));
    }
    Ok(Value::undefined())
}

fn function_call_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target = cx.require_callable_object(invocation.this_value())?;
    let rebound_this = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    cx.call_to_completion(
        target,
        rebound_this,
        invocation.arguments().get(1..).unwrap_or(&[]),
    )
}

fn function_apply_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target = cx.require_callable_object(invocation.this_value())?;
    let rebound_this = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let apply_arguments = cx.collect_array_like_arguments(
        cx.builtin_realm(),
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    cx.call_to_completion(target, rebound_this, &apply_arguments)
}

fn function_bind_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target = cx.require_callable_object(invocation.this_value())?;
    let bound_this = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let function = cx.create_bound_function(
        target,
        bound_this,
        invocation.arguments().get(1..).unwrap_or(&[]),
    )?;
    Ok(Value::from_object_ref(function))
}

fn function_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let function = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let text = cx.function_to_string_text(function)?;
    Ok(string_value(cx, &text))
}

fn async_generator_next_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    cx.async_generator_next(
        invocation.this_value(),
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )
}

fn async_generator_return_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    cx.async_generator_return(
        invocation.this_value(),
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )
}

fn async_generator_throw_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    cx.async_generator_throw(
        invocation.this_value(),
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )
}

fn generator_next_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let generator = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    cx.generator_next(
        generator,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )
}

fn generator_return_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let generator = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    cx.generator_return(
        generator,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )
}

fn generator_throw_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let generator = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    cx.generator_throw(
        generator,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )
}

fn promise_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let executor = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let executor = cx.require_callable_object(executor)?;
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let realm = cx.builtin_realm();
    let default_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|realm| realm.intrinsics().promise_prototype())
        .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let promise_object = cx.allocate_ordinary_object_with_prototype(realm, Some(prototype))?;
    let _ = cx.agent().alloc_promise(promise_object, realm);
    let capability = cx.agent().alloc_promise_capability();
    let _ = cx
        .agent()
        .set_promise_capability_promise(capability, promise_object);
    let resolve = cx.allocate_builtin_function(js3_promise_resolve_function_builtin())?;
    let reject = cx.allocate_builtin_function(js3_promise_reject_function_builtin())?;
    let _ = cx
        .agent()
        .set_promise_capability_resolve(capability, resolve);
    let _ = cx.agent().set_promise_capability_reject(capability, reject);
    let _ = cx.agent().alloc_promise_resolving_function(
        resolve,
        lyng_js_env::PromiseResolvingFunctionRecord::new(
            PromiseResolvingFunctionKind::Resolve,
            capability,
        ),
    );
    let _ = cx.agent().alloc_promise_resolving_function(
        reject,
        lyng_js_env::PromiseResolvingFunctionRecord::new(
            PromiseResolvingFunctionKind::Reject,
            capability,
        ),
    );
    if let Err(error) = cx.call_to_completion(
        executor,
        Value::undefined(),
        &[
            Value::from_object_ref(resolve),
            Value::from_object_ref(reject),
        ],
    ) {
        if let Some(thrown) = cx.extract_thrown_value(error)? {
            let _ = cx.call_to_completion(reject, Value::undefined(), &[thrown])?;
        }
    }
    Ok(Value::from_object_ref(promise_object))
}

fn promise_then_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let promise = require_promise_receiver(cx, invocation.this_value())?;
    let on_fulfilled = reaction_handler_for_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
        PromiseReactionHandler::Identity,
    );
    let on_rejected = reaction_handler_for_value(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
        PromiseReactionHandler::Thrower,
    );
    perform_promise_then(cx, promise, on_fulfilled, on_rejected)
}

fn promise_catch_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let promise = invocation.this_value();
    let on_rejected = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    invoke_then_method(cx, promise, Value::undefined(), on_rejected)
}

fn promise_finally_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let promise = invocation.this_value();
    let on_finally = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(on_finally_object) = on_finally
        .as_object_ref()
        .filter(|object| cx.agent().objects().is_callable(*object))
    else {
        return invoke_then_method(cx, promise, on_finally, on_finally);
    };
    let promise_object = promise.as_object_ref().ok_or_else(|| type_error(cx))?;
    let constructor = promise_species_constructor(cx, promise_object)?;
    let then_finally = allocate_promise_finally_function(
        cx,
        PromiseFinallyFunctionKind::Then,
        on_finally_object,
        constructor,
    )?;
    let catch_finally = allocate_promise_finally_function(
        cx,
        PromiseFinallyFunctionKind::Catch,
        on_finally_object,
        constructor,
    )?;
    invoke_then_method(
        cx,
        promise,
        Value::from_object_ref(then_finally),
        Value::from_object_ref(catch_finally),
    )
}

fn promise_resolve_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let constructor = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let resolution = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if let Some(promise_object) = resolution
        .as_object_ref()
        .filter(|object| cx.agent().promise_record(*object).is_some())
    {
        let constructor_value = cx.get_property_value(
            Value::from_object_ref(promise_object),
            PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        )?;
        if constructor_value.as_object_ref() == Some(constructor) {
            return Ok(resolution);
        }
    }

    let capability = new_promise_capability(cx, constructor)?;
    let promise_object = promise_capability_promise(cx, capability)?;
    let resolve = promise_capability_resolve(cx, capability)?;
    let _ = cx.call_to_completion(resolve, Value::undefined(), &[resolution])?;
    Ok(Value::from_object_ref(promise_object))
}

fn promise_reject_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let constructor = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let reason = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let capability = new_promise_capability(cx, constructor)?;
    let promise_object = promise_capability_promise(cx, capability)?;
    let reject = promise_capability_reject(cx, capability)?;
    let _ = cx.call_to_completion(reject, Value::undefined(), &[reason])?;
    Ok(Value::from_object_ref(promise_object))
}

fn promise_all_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    promise_collecting_combinator_builtin(cx, invocation, PromiseCombinatorKind::All)
}

fn promise_all_settled_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    promise_collecting_combinator_builtin(cx, invocation, PromiseCombinatorKind::AllSettled)
}

fn promise_race_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let constructor = require_constructor_object(cx, invocation.this_value())?;
    let capability = new_promise_capability(cx, constructor)?;
    let promise = Value::from_object_ref(promise_capability_promise(cx, capability)?);
    let iterable = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if let Err(error) = perform_promise_race(cx, constructor, capability, iterable) {
        reject_promise_capability_error(cx, capability, error)?;
    }
    Ok(promise)
}

fn promise_any_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    promise_collecting_combinator_builtin(cx, invocation, PromiseCombinatorKind::Any)
}

fn promise_species_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    _cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Ok(invocation.this_value())
}

fn promise_capability_executor_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let function = cx.callee_object();
    let record = cx
        .agent()
        .promise_resolving_function(function)
        .ok_or_else(|| type_error(cx))?;
    if record.kind() != PromiseResolvingFunctionKind::CapabilityExecutor {
        return Err(type_error(cx));
    }
    let capability = record.capability();
    if cx
        .agent()
        .promise_capability(capability)
        .is_some_and(|record| {
            record
                .resolve_value()
                .is_some_and(|value| !value.is_undefined())
                || record
                    .reject_value()
                    .is_some_and(|value| !value.is_undefined())
        })
    {
        return Err(type_error(cx));
    }
    let resolve = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let reject = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let _ = cx
        .agent()
        .set_promise_capability_resolve_value(capability, resolve);
    let _ = cx
        .agent()
        .set_promise_capability_reject_value(capability, reject);
    Ok(Value::undefined())
}

fn promise_resolve_function_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    promise_resolving_function_builtin(cx, invocation, PromiseResolvingFunctionKind::Resolve)
}

fn promise_reject_function_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    promise_resolving_function_builtin(cx, invocation, PromiseResolvingFunctionKind::Reject)
}

fn promise_combinator_element_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    expected_kind: PromiseCombinatorElementKind,
) -> Result<Value, Cx::Error> {
    let function = cx.callee_object();
    let record = cx
        .agent()
        .promise_combinator_element(function)
        .ok_or_else(|| type_error(cx))?;
    if record.kind() != expected_kind {
        return Err(type_error(cx));
    }
    let combinator_id = record.combinator();
    if cx
        .agent()
        .promise_combinator_already_called(combinator_id, record.index())
        .ok_or_else(|| type_error(cx))?
    {
        return Ok(Value::undefined());
    }
    let _ = cx
        .agent()
        .set_promise_combinator_already_called(combinator_id, record.index(), true);
    let capability = cx
        .agent()
        .promise_combinator(combinator_id)
        .map(lyng_js_env::PromiseCombinatorRecord::capability)
        .ok_or_else(|| type_error(cx))?;
    let argument = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    match expected_kind {
        PromiseCombinatorElementKind::AllResolve => {
            if !cx
                .agent()
                .set_promise_combinator_value(combinator_id, record.index(), argument)
            {
                return Err(type_error(cx));
            }
            let remaining = cx
                .agent()
                .decrement_promise_combinator_remaining(combinator_id)
                .ok_or_else(|| type_error(cx))?;
            if remaining == 0 {
                let values = promise_combinator_values_array(cx, combinator_id)?;
                let resolve = promise_capability_resolve(cx, capability)?;
                let _ = cx.call_to_completion(
                    resolve,
                    Value::undefined(),
                    &[Value::from_object_ref(values)],
                )?;
            }
        }
        PromiseCombinatorElementKind::AllSettledResolve
        | PromiseCombinatorElementKind::AllSettledReject => {
            let settled = promise_all_settled_result_object(cx, expected_kind, argument)?;
            if !cx.agent().set_promise_combinator_value(
                combinator_id,
                record.index(),
                Value::from_object_ref(settled),
            ) {
                return Err(type_error(cx));
            }
            let remaining = cx
                .agent()
                .decrement_promise_combinator_remaining(combinator_id)
                .ok_or_else(|| type_error(cx))?;
            if remaining == 0 {
                let values = promise_combinator_values_array(cx, combinator_id)?;
                let resolve = promise_capability_resolve(cx, capability)?;
                let _ = cx.call_to_completion(
                    resolve,
                    Value::undefined(),
                    &[Value::from_object_ref(values)],
                )?;
            }
        }
        PromiseCombinatorElementKind::AnyReject => {
            if !cx
                .agent()
                .set_promise_combinator_value(combinator_id, record.index(), argument)
            {
                return Err(type_error(cx));
            }
            let remaining = cx
                .agent()
                .decrement_promise_combinator_remaining(combinator_id)
                .ok_or_else(|| type_error(cx))?;
            if remaining == 0 {
                reject_promise_any_errors(cx, capability, combinator_id)?;
            }
        }
    }
    Ok(Value::undefined())
}

fn promise_collecting_combinator_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: PromiseCombinatorKind,
) -> Result<Value, Cx::Error> {
    let constructor = require_constructor_object(cx, invocation.this_value())?;
    let capability = new_promise_capability(cx, constructor)?;
    let promise = Value::from_object_ref(promise_capability_promise(cx, capability)?);
    let iterable = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let combinator = cx.agent().alloc_promise_combinator(kind, capability);
    let result = match kind {
        PromiseCombinatorKind::All => {
            perform_promise_all(cx, constructor, capability, combinator, iterable)
        }
        PromiseCombinatorKind::AllSettled => {
            perform_promise_all_settled(cx, constructor, capability, combinator, iterable)
        }
        PromiseCombinatorKind::Any => {
            perform_promise_any(cx, constructor, capability, combinator, iterable)
        }
    };
    if let Err(error) = result {
        reject_promise_capability_error(cx, capability, error)?;
    }
    Ok(promise)
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

fn perform_promise_all<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor: ObjectRef,
    capability: lyng_js_env::PromiseCapabilityId,
    combinator: lyng_js_env::PromiseCombinatorId,
    iterable: Value,
) -> Result<(), Cx::Error> {
    let promise_resolve = promise_resolve_method(cx, constructor)?;
    let reject = promise_capability_reject(cx, capability)?;
    let mut iterator_record = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::get_iterator(&mut bridge, iterable)?
    };
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
            let remaining = cx
                .agent()
                .decrement_promise_combinator_remaining(combinator)
                .ok_or_else(|| type_error(cx))?;
            if remaining == 0 {
                let values = promise_combinator_values_array(cx, combinator)?;
                let resolve = promise_capability_resolve(cx, capability)?;
                let _ = cx.call_to_completion(
                    resolve,
                    Value::undefined(),
                    &[Value::from_object_ref(values)],
                )?;
            }
            return Ok(());
        };
        let next_value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let next_value = match next_value {
            Ok(next_value) => next_value,
            Err(error) => {
                iterator_record.set_done(true);
                return Err(error);
            }
        };
        let index = cx
            .agent()
            .push_promise_combinator_placeholder(combinator)
            .ok_or_else(|| type_error(cx))?;
        let next_promise = match cx.call_to_completion(
            promise_resolve,
            Value::from_object_ref(constructor),
            &[next_value],
        ) {
            Ok(next_promise) => next_promise,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let resolve_element = allocate_promise_combinator_element(
            cx,
            combinator,
            index,
            PromiseCombinatorElementKind::AllResolve,
        )?;
        if let Err(error) = invoke_then_method(
            cx,
            next_promise,
            Value::from_object_ref(resolve_element),
            Value::from_object_ref(reject),
        ) {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    }
}

fn perform_promise_all_settled<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor: ObjectRef,
    capability: lyng_js_env::PromiseCapabilityId,
    combinator: lyng_js_env::PromiseCombinatorId,
    iterable: Value,
) -> Result<(), Cx::Error> {
    let promise_resolve = promise_resolve_method(cx, constructor)?;
    let mut iterator_record = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::get_iterator(&mut bridge, iterable)?
    };
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
            let remaining = cx
                .agent()
                .decrement_promise_combinator_remaining(combinator)
                .ok_or_else(|| type_error(cx))?;
            if remaining == 0 {
                let values = promise_combinator_values_array(cx, combinator)?;
                let resolve = promise_capability_resolve(cx, capability)?;
                let _ = cx.call_to_completion(
                    resolve,
                    Value::undefined(),
                    &[Value::from_object_ref(values)],
                )?;
            }
            return Ok(());
        };
        let next_value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let next_value = match next_value {
            Ok(next_value) => next_value,
            Err(error) => {
                iterator_record.set_done(true);
                return Err(error);
            }
        };
        let index = cx
            .agent()
            .push_promise_combinator_placeholder(combinator)
            .ok_or_else(|| type_error(cx))?;
        let next_promise = match cx.call_to_completion(
            promise_resolve,
            Value::from_object_ref(constructor),
            &[next_value],
        ) {
            Ok(next_promise) => next_promise,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let resolve_element = allocate_promise_combinator_element(
            cx,
            combinator,
            index,
            PromiseCombinatorElementKind::AllSettledResolve,
        )?;
        let reject_element = allocate_promise_combinator_element(
            cx,
            combinator,
            index,
            PromiseCombinatorElementKind::AllSettledReject,
        )?;
        if let Err(error) = invoke_then_method(
            cx,
            next_promise,
            Value::from_object_ref(resolve_element),
            Value::from_object_ref(reject_element),
        ) {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    }
}

fn perform_promise_any<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor: ObjectRef,
    capability: lyng_js_env::PromiseCapabilityId,
    combinator: lyng_js_env::PromiseCombinatorId,
    iterable: Value,
) -> Result<(), Cx::Error> {
    let promise_resolve = promise_resolve_method(cx, constructor)?;
    let resolve = promise_capability_resolve(cx, capability)?;
    let mut iterator_record = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::get_iterator(&mut bridge, iterable)?
    };
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
            let remaining = cx
                .agent()
                .decrement_promise_combinator_remaining(combinator)
                .ok_or_else(|| type_error(cx))?;
            if remaining == 0 {
                reject_promise_any_errors(cx, capability, combinator)?;
            }
            return Ok(());
        };
        let next_value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let next_value = match next_value {
            Ok(next_value) => next_value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let index = cx
            .agent()
            .push_promise_combinator_placeholder(combinator)
            .ok_or_else(|| type_error(cx))?;
        let next_promise = match cx.call_to_completion(
            promise_resolve,
            Value::from_object_ref(constructor),
            &[next_value],
        ) {
            Ok(next_promise) => next_promise,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let reject_element = allocate_promise_combinator_element(
            cx,
            combinator,
            index,
            PromiseCombinatorElementKind::AnyReject,
        )?;
        if let Err(error) = invoke_then_method(
            cx,
            next_promise,
            Value::from_object_ref(resolve),
            Value::from_object_ref(reject_element),
        ) {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    }
}

fn perform_promise_race<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor: ObjectRef,
    capability: lyng_js_env::PromiseCapabilityId,
    iterable: Value,
) -> Result<(), Cx::Error> {
    let promise_resolve = promise_resolve_method(cx, constructor)?;
    let resolve = promise_capability_resolve(cx, capability)?;
    let reject = promise_capability_reject(cx, capability)?;
    let mut iterator_record = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::get_iterator(&mut bridge, iterable)?
    };
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
            return Ok(());
        };
        let next_value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let next_value = match next_value {
            Ok(next_value) => next_value,
            Err(error) => {
                iterator_record.set_done(true);
                return Err(error);
            }
        };
        let next_promise = match cx.call_to_completion(
            promise_resolve,
            Value::from_object_ref(constructor),
            &[next_value],
        ) {
            Ok(next_promise) => next_promise,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        if let Err(error) = invoke_then_method(
            cx,
            next_promise,
            Value::from_object_ref(resolve),
            Value::from_object_ref(reject),
        ) {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    }
}

fn reject_promise_any_errors<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    capability: lyng_js_env::PromiseCapabilityId,
    combinator: lyng_js_env::PromiseCombinatorId,
) -> Result<(), Cx::Error> {
    let reasons = cx
        .agent()
        .promise_combinator(combinator)
        .map(lyng_js_env::PromiseCombinatorRecord::values)
        .map(<[Value]>::to_vec)
        .ok_or_else(|| type_error(cx))?;
    let aggregate_error = create_aggregate_error_from_values(cx, &reasons, None)?;
    let reject = promise_capability_reject(cx, capability)?;
    let _ = cx.call_to_completion(
        reject,
        Value::undefined(),
        &[Value::from_object_ref(aggregate_error)],
    )?;
    Ok(())
}

fn promise_resolve_method<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor: ObjectRef,
) -> Result<ObjectRef, Cx::Error> {
    let resolve_key = property_key_from_text(cx, "resolve");
    let resolve = cx.get_property_value(Value::from_object_ref(constructor), resolve_key)?;
    cx.require_callable_object(resolve)
}

fn invoke_then_method<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    promise: Value,
    on_fulfilled: Value,
    on_rejected: Value,
) -> Result<Value, Cx::Error> {
    let then_key = property_key_from_text(cx, "then");
    let then = cx.get_property_value(promise, then_key)?;
    let then = cx.require_callable_object(then)?;
    cx.call_to_completion(then, promise, &[on_fulfilled, on_rejected])
}

fn allocate_promise_finally_function<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: PromiseFinallyFunctionKind,
    on_finally: ObjectRef,
    constructor: ObjectRef,
) -> Result<ObjectRef, Cx::Error> {
    let function = cx.allocate_builtin_function(js3_promise_finally_function_builtin())?;
    let _ = cx.agent().alloc_promise_finally_function(
        function,
        PromiseFinallyFunctionRecord::new(kind, on_finally, constructor),
    );
    Ok(function)
}

fn allocate_promise_combinator_element<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    combinator: lyng_js_env::PromiseCombinatorId,
    index: usize,
    kind: PromiseCombinatorElementKind,
) -> Result<ObjectRef, Cx::Error> {
    let entry = promise_combinator_element_entry(kind);
    let function = cx.allocate_builtin_function(entry)?;
    let _ = cx.agent().alloc_promise_combinator_element(
        function,
        PromiseCombinatorElementRecord::new(kind, combinator, index),
    );
    Ok(function)
}

fn promise_combinator_element_entry(kind: PromiseCombinatorElementKind) -> BuiltinFunctionId {
    match kind {
        PromiseCombinatorElementKind::AllResolve => js3_promise_all_resolve_element_builtin(),
        PromiseCombinatorElementKind::AllSettledResolve => {
            js3_promise_all_settled_resolve_element_builtin()
        }
        PromiseCombinatorElementKind::AllSettledReject => {
            js3_promise_all_settled_reject_element_builtin()
        }
        PromiseCombinatorElementKind::AnyReject => js3_promise_any_reject_element_builtin(),
    }
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

fn reject_promise_capability_error<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    capability: lyng_js_env::PromiseCapabilityId,
    error: Cx::Error,
) -> Result<(), Cx::Error> {
    let Some(thrown) = cx.extract_thrown_value(error)? else {
        unreachable!("non-abrupt builtin error should propagate")
    };
    let reject = promise_capability_reject(cx, capability)?;
    let _ = cx.call_to_completion(reject, Value::undefined(), &[thrown])?;
    Ok(())
}

fn promise_combinator_values_array<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    combinator: lyng_js_env::PromiseCombinatorId,
) -> Result<ObjectRef, Cx::Error> {
    let values = cx
        .agent()
        .promise_combinator(combinator)
        .map(lyng_js_env::PromiseCombinatorRecord::values)
        .map(<[Value]>::to_vec)
        .ok_or_else(|| type_error(cx))?;
    create_array_from_values(cx, &values)
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

fn create_aggregate_error_from_values<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    values: &[Value],
    message: Option<Value>,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().aggregate_error_prototype())
        .ok_or_else(|| type_error(cx))?;
    let error = {
        let agent = cx.agent();
        errors::create_error_object(agent, realm, Some(prototype), message)
    };
    let error = map_completion(cx, error)?;
    let errors_array = create_array_from_values(cx, values)?;
    let errors_key = property_key_from_text(cx, "errors");
    define_data_property_with_attrs(
        cx,
        error,
        errors_key,
        Value::from_object_ref(errors_array),
        true,
        false,
        true,
    )?;
    Ok(error)
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

fn promise_all_settled_result_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: PromiseCombinatorElementKind,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    let object_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|realm| realm.intrinsics().object_prototype())
        .ok_or_else(|| type_error(cx))?;
    let result = cx.allocate_ordinary_object_with_prototype(realm, Some(object_prototype))?;
    let status = match kind {
        PromiseCombinatorElementKind::AllSettledResolve => string_value(cx, "fulfilled"),
        PromiseCombinatorElementKind::AllSettledReject => string_value(cx, "rejected"),
        _ => return Err(type_error(cx)),
    };
    let status_key = property_key_from_text(cx, "status");
    create_data_property_or_throw(cx, result, status_key, status)?;
    let key = match kind {
        PromiseCombinatorElementKind::AllSettledResolve => property_key_from_text(cx, "value"),
        PromiseCombinatorElementKind::AllSettledReject => property_key_from_text(cx, "reason"),
        _ => return Err(type_error(cx)),
    };
    create_data_property_or_throw(cx, result, key, value)?;
    Ok(result)
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

fn require_promise_receiver<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if cx.agent().promise_record(object).is_none() {
        return Err(type_error(cx));
    }
    Ok(object)
}

fn reaction_handler_for_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    fallback: PromiseReactionHandler,
) -> PromiseReactionHandler {
    value
        .as_object_ref()
        .filter(|object| cx.agent().objects().is_callable(*object))
        .map(PromiseReactionHandler::Callable)
        .unwrap_or(fallback)
}

fn promise_default_constructor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    cx.agent()
        .realm(realm)
        .and_then(|realm| realm.intrinsics().promise())
        .ok_or_else(|| type_error(cx))
}

fn promise_species_constructor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    promise_object: ObjectRef,
) -> Result<ObjectRef, Cx::Error> {
    let default_constructor = promise_default_constructor(cx)?;
    let constructor = cx.get_property_value(
        Value::from_object_ref(promise_object),
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

fn new_promise_capability<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor: ObjectRef,
) -> Result<lyng_js_env::PromiseCapabilityId, Cx::Error> {
    let capability = cx.agent().alloc_promise_capability();
    let executor = cx.allocate_builtin_function(js3_promise_capability_executor_builtin())?;
    let _ = cx.agent().alloc_promise_resolving_function(
        executor,
        lyng_js_env::PromiseResolvingFunctionRecord::new(
            PromiseResolvingFunctionKind::CapabilityExecutor,
            capability,
        ),
    );
    let promise = cx.construct_to_completion(
        constructor,
        &[Value::from_object_ref(executor)],
        Some(constructor),
    )?;
    let _ = cx
        .agent()
        .set_promise_capability_promise(capability, promise);
    let (resolve, reject) = cx
        .agent()
        .promise_capability(capability)
        .map(|record| (record.resolve_value(), record.reject_value()))
        .ok_or_else(|| type_error(cx))?;
    let resolve = resolve.ok_or_else(|| type_error(cx))?;
    let reject = reject.ok_or_else(|| type_error(cx))?;
    let resolve = cx.require_callable_object(resolve)?;
    let reject = cx.require_callable_object(reject)?;
    let _ = cx
        .agent()
        .set_promise_capability_resolve(capability, resolve);
    let _ = cx.agent().set_promise_capability_reject(capability, reject);
    Ok(capability)
}

fn promise_capability_promise<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    capability: lyng_js_env::PromiseCapabilityId,
) -> Result<ObjectRef, Cx::Error> {
    cx.agent()
        .promise_capability(capability)
        .and_then(|record| record.promise())
        .ok_or_else(|| type_error(cx))
}

fn promise_capability_resolve<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    capability: lyng_js_env::PromiseCapabilityId,
) -> Result<ObjectRef, Cx::Error> {
    cx.agent()
        .promise_capability(capability)
        .and_then(|record| record.resolve())
        .ok_or_else(|| type_error(cx))
}

fn promise_capability_reject<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    capability: lyng_js_env::PromiseCapabilityId,
) -> Result<ObjectRef, Cx::Error> {
    cx.agent()
        .promise_capability(capability)
        .and_then(|record| record.reject())
        .ok_or_else(|| type_error(cx))
}

fn perform_promise_then<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    promise_object: ObjectRef,
    on_fulfilled: PromiseReactionHandler,
    on_rejected: PromiseReactionHandler,
) -> Result<Value, Cx::Error> {
    let constructor = promise_species_constructor(cx, promise_object)?;
    let capability = new_promise_capability(cx, constructor)?;
    let fulfill_reaction = promise::create_promise_reaction(
        cx.agent(),
        PromiseReactionKind::Fulfill,
        on_fulfilled,
        Some(capability),
    );
    let reject_reaction = promise::create_promise_reaction(
        cx.agent(),
        PromiseReactionKind::Reject,
        on_rejected,
        Some(capability),
    );
    let record = cx
        .agent()
        .promise_record(promise_object)
        .cloned()
        .ok_or_else(|| type_error(cx))?;
    let _ = cx.agent().set_promise_handled(promise_object, true);
    match record.state() {
        PromiseState::Pending => {
            let _ = cx.agent().push_promise_reaction(
                promise_object,
                PromiseReactionKind::Fulfill,
                fulfill_reaction,
            );
            let _ = cx.agent().push_promise_reaction(
                promise_object,
                PromiseReactionKind::Reject,
                reject_reaction,
            );
        }
        PromiseState::Fulfilled => {
            enqueue_promise_reaction_job(
                cx.agent(),
                record.realm(),
                fulfill_reaction,
                record.result(),
            );
        }
        PromiseState::Rejected => {
            enqueue_promise_reaction_job(
                cx.agent(),
                record.realm(),
                reject_reaction,
                record.result(),
            );
        }
    }
    Ok(Value::from_object_ref(promise_capability_promise(
        cx, capability,
    )?))
}

fn enqueue_promise_reaction_job(
    agent: &mut Agent,
    realm: RealmRef,
    reaction: lyng_js_env::PromiseReactionId,
    argument: Value,
) {
    let _ = agent.enqueue_job_with_payload(
        lyng_js_host::HostJobKind::Promise,
        lyng_js_env::ExecutableId::Builtin,
        lyng_js_env::RuntimeJobPayload::PromiseReaction { reaction, argument },
        Some(realm),
        Some("PromiseReaction".into()),
    );
}

fn promise_resolving_function_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    expected_kind: PromiseResolvingFunctionKind,
) -> Result<Value, Cx::Error> {
    let function = cx.callee_object();
    let record = cx
        .agent()
        .promise_resolving_function(function)
        .ok_or_else(|| type_error(cx))?;
    if record.kind() != expected_kind {
        return Err(type_error(cx));
    }
    let capability = record.capability();
    if cx
        .agent()
        .promise_capability(capability)
        .is_some_and(|record| record.already_resolved())
    {
        return Ok(Value::undefined());
    }
    let _ = cx
        .agent()
        .set_promise_capability_already_resolved(capability, true);
    let promise_object = promise_capability_promise(cx, capability)?;
    let resolution = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if expected_kind == PromiseResolvingFunctionKind::Reject {
        promise::reject_promise(cx.agent(), promise_object, resolution)
            .map_err(|abrupt| cx.abrupt(abrupt))?;
        return Ok(Value::undefined());
    }
    if resolution.as_object_ref() == Some(promise_object) {
        let reason = errors::throw_type_error(cx.agent())
            .thrown_value()
            .unwrap_or(Value::undefined());
        promise::reject_promise(cx.agent(), promise_object, reason)
            .map_err(|abrupt| cx.abrupt(abrupt))?;
        return Ok(Value::undefined());
    }
    let Some(thenable) = resolution.as_object_ref() else {
        promise::fulfill_promise(cx.agent(), promise_object, resolution)
            .map_err(|abrupt| cx.abrupt(abrupt))?;
        return Ok(Value::undefined());
    };
    let then_key = PropertyKey::from_atom(cx.agent().atoms_mut().intern_collectible("then"));
    let then = match cx.get_property_value(Value::from_object_ref(thenable), then_key) {
        Ok(then) => then,
        Err(error) => {
            if let Some(thrown) = cx.extract_thrown_value(error)? {
                promise::reject_promise(cx.agent(), promise_object, thrown)
                    .map_err(|abrupt| cx.abrupt(abrupt))?;
                return Ok(Value::undefined());
            }
            unreachable!("non-abrupt builtin error should propagate")
        }
    };
    let Some(then) = then
        .as_object_ref()
        .filter(|object| cx.agent().objects().is_callable(*object))
    else {
        promise::fulfill_promise(cx.agent(), promise_object, resolution)
            .map_err(|abrupt| cx.abrupt(abrupt))?;
        return Ok(Value::undefined());
    };
    let realm = cx
        .agent()
        .promise_record(promise_object)
        .map(|record| record.realm())
        .unwrap_or(cx.builtin_realm());
    promise::enqueue_thenable_job(cx.agent(), realm, promise_object, thenable, then);
    Ok(Value::undefined())
}

fn promise_finally_function_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let function = cx.callee_object();
    let record = cx
        .agent()
        .promise_finally_function(function)
        .ok_or_else(|| type_error(cx))?;
    let argument = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let result = cx.call_to_completion(record.on_finally(), Value::undefined(), &[])?;
    let resolve = promise_resolve_method(cx, record.constructor())?;
    let promise = cx.call_to_completion(
        resolve,
        Value::from_object_ref(record.constructor()),
        &[result],
    )?;
    let promise_object = promise
        .as_object_ref()
        .filter(|object| cx.agent().promise_record(*object).is_some())
        .ok_or_else(|| type_error(cx))?;
    let on_fulfilled = match record.kind() {
        PromiseFinallyFunctionKind::Then => PromiseReactionHandler::PassThrough(argument),
        PromiseFinallyFunctionKind::Catch => PromiseReactionHandler::ThrowWith(argument),
    };
    perform_promise_then(
        cx,
        promise_object,
        on_fulfilled,
        PromiseReactionHandler::Thrower,
    )
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
    let value = if invocation.arguments().is_empty() {
        string_value(cx, "")
    } else {
        string_value_from_value(cx, invocation.arguments()[0])?
    };
    if invocation.new_target().is_none() {
        return Ok(value);
    }
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

fn string_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::String,
        )
    };
    map_completion(cx, value)
}

fn string_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::String,
        )
    };
    map_completion(cx, value)
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
    let realm = cx.builtin_realm();
    let prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().array_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let object = cx.allocate_ordinary_object_with_prototype(realm, Some(prototype))?;
    define_data_property_with_attrs(
        cx,
        object,
        PropertyKey::from_atom(WellKnownAtom::length.id()),
        length_value(length),
        true,
        false,
        false,
    )?;
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
            Some(digit @ '1'..='9') => {
                let first = usize::from((digit as u8) - b'0');
                let mut capture_index = first;
                let mut consumed = 2;
                if let Some(second) = template_units
                    .get(index + 2)
                    .and_then(|unit| code_unit_ascii(*unit))
                    .filter(u8::is_ascii_digit)
                {
                    let candidate = first * 10 + usize::from(second - b'0');
                    if candidate != 0 && candidate <= captures.len() {
                        capture_index = candidate;
                        consumed = 3;
                    }
                }
                if capture_index <= captures.len() {
                    if let Some(range) = captures[capture_index - 1].clone() {
                        result.extend_from_slice(&source_units[range.start..range.end]);
                    }
                    index += consumed;
                } else {
                    result.push(u16::from(b'$'));
                    index += 1;
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

fn string_match_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let source_value = Value::from_string_ref(source_ref);
    let pattern_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());

    let regexp_object = match pattern_value.as_object_ref() {
        Some(object_ref) if is_regexp_object(cx, object_ref)? => object_ref,
        _ => {
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
            allocate_regexp_object(cx, realm, default_prototype, &pattern_text, "")?
        }
    };

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

fn string_search_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let pattern_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());

    let matched = if let Some(object_ref) = pattern_value.as_object_ref() {
        if is_regexp_object(cx, object_ref)? {
            let payload = {
                let agent = cx.agent();
                agent.objects().regexp_payload(object_ref).cloned()
            }
            .ok_or_else(|| type_error(cx))?;
            payload.find_from_code_units(&source_units, 0)
        } else {
            let pattern_text = cx.value_to_string_text(pattern_value)?;
            let payload = lyng_js_objects::RegExpPayload::compile(&pattern_text, "")
                .map_err(|_| syntax_error(cx))?;
            payload.find_from_code_units(&source_units, 0)
        }
    } else {
        let pattern_text = if pattern_value.is_undefined() {
            String::new()
        } else {
            cx.value_to_string_text(pattern_value)?
        };
        let payload = lyng_js_objects::RegExpPayload::compile(&pattern_text, "")
            .map_err(|_| syntax_error(cx))?;
        payload.find_from_code_units(&source_units, 0)
    };

    Ok(matched
        .map(|record| usize_index_value(record.range().start))
        .unwrap_or_else(|| Value::from_smi(-1)))
}

fn string_last_index_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source = string_ref_text(cx, source_ref)?;
    let search_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let search = string_ref_text(cx, search_ref)?;

    let position = if let Some(value) = invocation.arguments().get(1).copied() {
        to_integer_or_infinity_for_builtin(cx, value)?
    } else {
        f64::INFINITY
    };

    let source_len = source.len();
    let search_len = search.len();
    let start = if position.is_nan() {
        0
    } else if position == f64::INFINITY {
        source_len
    } else if position <= 0.0 {
        0
    } else {
        (position as usize).min(source_len)
    };

    if search.is_empty() {
        return Ok(Value::from_smi(i32::try_from(start).unwrap_or(i32::MAX)));
    }
    if search_len > source_len {
        return Ok(Value::from_smi(-1));
    }

    let max_index = start.min(source_len - search_len);
    let bytes = source.as_bytes();
    let needle = search.as_bytes();
    for index in (0..=max_index).rev() {
        if &bytes[index..index + search_len] == needle {
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

fn string_replace_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let source_value = Value::from_string_ref(source_ref);
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

    let callable_replacement = callable_object_from_value(cx, replacement);
    let regexp_object = match search_value.as_object_ref() {
        Some(object_ref) if is_regexp_object(cx, object_ref)? => Some(object_ref),
        _ => None,
    };
    if let Some(object_ref) = regexp_object {
        let flags = regexp_object_flags(cx, object_ref)?;
        if flags.global() {
            let last_index_key = regexp_last_index_key(cx);
            set_data_property_value(cx, object_ref, last_index_key, Value::from_smi(0))?;
        }

        let replacement_template_units = if callable_replacement.is_none() {
            let replacement_ref = to_string_string_ref(cx, replacement)?;
            Some(string_ref_code_units(cx, replacement_ref)?)
        } else {
            None
        };

        let mut matches = Vec::new();
        while let Some(state) = regexp_exec_state(cx, object_ref, &source_units)? {
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
                if let Some(groups) = allocate_named_capture_object(
                    cx,
                    matched.named_captures(),
                    &source_units,
                    false,
                )? {
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
        return Ok(string_from_code_units(cx, &result));
    }

    let pattern_ref = to_string_string_ref(cx, search_value)?;
    let pattern_units = string_ref_code_units(cx, pattern_ref)?;
    let Some(start) = find_subsequence(&source_units, &pattern_units, 0) else {
        return Ok(Value::from_string_ref(source_ref));
    };
    let end = start + pattern_units.len();
    let replacement_template = if callable_replacement.is_none() {
        let replacement_ref = to_string_string_ref(cx, replacement)?;
        Some(string_ref_code_units(cx, replacement_ref)?)
    } else {
        None
    };
    let replacement_units = if let Some(callee) = callable_replacement {
        let matched_value = string_from_code_units(cx, &source_units[start..end]);
        let arguments = [matched_value, usize_index_value(start), source_value];
        let replaced = cx.call_to_completion(callee, Value::undefined(), &arguments)?;
        let replaced_ref = to_string_string_ref(cx, replaced)?;
        string_ref_code_units(cx, replaced_ref)?
    } else {
        replacement_template
            .expect("template units should exist for non-callable string replacement")
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
    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let separator_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let limit = if let Some(value) = invocation.arguments().get(1).copied() {
        to_uint32_for_builtin(cx, value)?
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
        to_integer_or_infinity_for_builtin(cx, value)?
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
        to_integer_or_infinity_for_builtin(cx, value)?
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
    if let Some(object_ref) = search_value.as_object_ref() {
        if is_regexp_object(cx, object_ref)? {
            return Err(type_error(cx));
        }
    }
    let search_ref = to_string_string_ref(cx, search_value)?;
    let search_units = string_ref_code_units(cx, search_ref)?;
    let position = if let Some(value) = invocation.arguments().get(1).copied() {
        to_integer_or_infinity_for_builtin(cx, value)?
    } else {
        0.0
    };
    let start = if !position.is_finite() || position <= 0.0 {
        0
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

const DATE_MS_PER_SECOND: i64 = 1_000;
const DATE_MS_PER_MINUTE: i64 = 60 * DATE_MS_PER_SECOND;
const DATE_MS_PER_HOUR: i64 = 60 * DATE_MS_PER_MINUTE;
const DATE_MS_PER_DAY: i64 = 24 * DATE_MS_PER_HOUR;
const DATE_NANOS_PER_MILLISECOND: i128 = 1_000_000;
const DATE_NANOS_PER_MINUTE: i64 = 60 * 1_000_000_000;
const DATE_CLIP_LIMIT_MS: f64 = 8_640_000_000_000_000.0;
const DATE_WEEKDAY_NAMES: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const DATE_MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

#[derive(Clone, Copy, Debug)]
struct DateParts {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    millisecond: u16,
    weekday: u8,
    offset_minutes: i32,
}

#[derive(Clone, Copy, Debug)]
enum DateStringKind {
    Full,
    Date,
    Time,
}

#[derive(Clone, Copy, Debug)]
enum DateComponent {
    FullYear,
    Month,
    Date,
    Day,
    Hours,
    Minutes,
    Seconds,
    Milliseconds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DateSetKind {
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Date,
    Month,
    FullYear,
}

fn date_number_argument<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Option<Value>,
    default: f64,
) -> Result<f64, Cx::Error> {
    value.map_or(Ok(default), |value| to_number_for_builtin(cx, value))
}

fn date_finite_integer(value: f64) -> Option<i64> {
    if !value.is_finite() || value < i64::MIN as f64 || value > i64::MAX as f64 {
        return None;
    }
    Some(value.trunc() as i64)
}

fn date_time_clip_value(time: f64) -> Value {
    if !time.is_finite() || time.abs() > DATE_CLIP_LIMIT_MS {
        return Value::from_f64(f64::NAN);
    }
    Value::from_f64(time.trunc() + 0.0)
}

fn date_apply_legacy_year_offset(year: f64) -> f64 {
    if !year.is_finite() {
        return year;
    }
    let integer = year.trunc();
    if (0.0..=99.0).contains(&integer) {
        1900.0 + integer
    } else {
        year
    }
}

fn date_balance_year_month(year: i64, month: i64) -> Option<(i32, u8)> {
    let balanced_year = year.checked_add(month.div_euclid(12))?;
    let balanced_month = month.rem_euclid(12) + 1;
    Some((
        i32::try_from(balanced_year).ok()?,
        u8::try_from(balanced_month).ok()?,
    ))
}

fn date_is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn date_days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if date_is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn date_days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let year = i64::from(year) - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month_prime = i64::from(month) + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + i64::from(day) - 1;
    let day_of_era = yoe * 365 + yoe / 4 - yoe / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn date_civil_from_days(days_since_epoch: i64) -> Option<(i32, u8, u8)> {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    Some((
        i32::try_from(year).ok()?,
        u8::try_from(month).ok()?,
        u8::try_from(day).ok()?,
    ))
}

fn date_split_time_millis(total_millis: i64) -> (u8, u8, u8, u16) {
    let hour = total_millis / DATE_MS_PER_HOUR;
    let minute = (total_millis % DATE_MS_PER_HOUR) / DATE_MS_PER_MINUTE;
    let second = (total_millis % DATE_MS_PER_MINUTE) / DATE_MS_PER_SECOND;
    let millisecond = total_millis % DATE_MS_PER_SECOND;
    (
        u8::try_from(hour).unwrap(),
        u8::try_from(minute).unwrap(),
        u8::try_from(second).unwrap(),
        u16::try_from(millisecond).unwrap(),
    )
}

fn date_weekday_from_days(days_since_epoch: i64) -> u8 {
    u8::try_from((days_since_epoch + 4).rem_euclid(7)).unwrap()
}

fn date_value_epoch_nanoseconds(value: Value) -> Option<i128> {
    let millis = value.as_f64()?;
    if !millis.is_finite() {
        return None;
    }
    Some((millis.trunc() as i128) * DATE_NANOS_PER_MILLISECOND)
}

fn date_make_day(year: f64, month: f64, date: f64) -> Option<i64> {
    let year = date_finite_integer(year)?;
    let month = date_finite_integer(month)?;
    let date = date_finite_integer(date)?;
    let (year, month) = date_balance_year_month(year, month)?;
    date_days_from_civil(year, month, 1).checked_add(date - 1)
}

fn date_make_time(hour: f64, minute: f64, second: f64, millisecond: f64) -> Option<f64> {
    if !hour.is_finite() || !minute.is_finite() || !second.is_finite() || !millisecond.is_finite() {
        return None;
    }
    Some(
        hour.trunc() * DATE_MS_PER_HOUR as f64
            + minute.trunc() * DATE_MS_PER_MINUTE as f64
            + second.trunc() * DATE_MS_PER_SECOND as f64
            + millisecond.trunc(),
    )
}

fn date_make_utc_value(
    year: f64,
    month: f64,
    date: f64,
    hour: f64,
    minute: f64,
    second: f64,
    millisecond: f64,
) -> Value {
    let Some(day) = date_make_day(year, month, date) else {
        return Value::from_f64(f64::NAN);
    };
    let Some(time) = date_make_time(hour, minute, second, millisecond) else {
        return Value::from_f64(f64::NAN);
    };
    date_time_clip_value(day as f64 * DATE_MS_PER_DAY as f64 + time)
}

fn date_make_local_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    year: f64,
    month: f64,
    date: f64,
    hour: f64,
    minute: f64,
    second: f64,
    millisecond: f64,
) -> Result<Value, Cx::Error> {
    let Some(base_day) = date_make_day(year, month, date) else {
        return Ok(Value::from_f64(f64::NAN));
    };
    let Some(time) = date_make_time(hour, minute, second, millisecond) else {
        return Ok(Value::from_f64(f64::NAN));
    };
    let Some(time_millis) = date_finite_integer(time) else {
        return Ok(Value::from_f64(f64::NAN));
    };
    let Some(day) = base_day.checked_add(time_millis.div_euclid(DATE_MS_PER_DAY)) else {
        return Ok(Value::from_f64(f64::NAN));
    };
    let Some((year, month, day_of_month)) = date_civil_from_days(day) else {
        return Ok(Value::from_f64(f64::NAN));
    };
    let (hour, minute, second, millisecond) =
        date_split_time_millis(time_millis.rem_euclid(DATE_MS_PER_DAY));
    let time_zone = cx.temporal_default_time_zone(&TemporalDefaultTimeZoneRequest {})?;
    let instant = cx.temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
        time_zone_id: time_zone.time_zone_id,
        date_time: TemporalCivilDateTime::new(
            year,
            month,
            day_of_month,
            hour,
            minute,
            second,
            millisecond,
            0,
            0,
        ),
        disambiguation: TemporalDisambiguation::Compatible,
    })?;
    Ok(date_time_clip_value(
        (instant.epoch_nanoseconds / DATE_NANOS_PER_MILLISECOND) as f64,
    ))
}

fn date_local_time_value_from_arguments<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    arguments: &[Value],
) -> Result<Value, Cx::Error> {
    let year = date_apply_legacy_year_offset(date_number_argument(
        cx,
        arguments.first().copied(),
        f64::NAN,
    )?);
    let month = date_number_argument(cx, arguments.get(1).copied(), f64::NAN)?;
    let date = date_number_argument(cx, arguments.get(2).copied(), 1.0)?;
    let hour = date_number_argument(cx, arguments.get(3).copied(), 0.0)?;
    let minute = date_number_argument(cx, arguments.get(4).copied(), 0.0)?;
    let second = date_number_argument(cx, arguments.get(5).copied(), 0.0)?;
    let millisecond = date_number_argument(cx, arguments.get(6).copied(), 0.0)?;
    date_make_local_value(cx, year, month, date, hour, minute, second, millisecond)
}

fn date_utc_time_value_from_arguments<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    arguments: &[Value],
) -> Result<Value, Cx::Error> {
    let year = date_apply_legacy_year_offset(date_number_argument(
        cx,
        arguments.first().copied(),
        f64::NAN,
    )?);
    let month = date_number_argument(cx, arguments.get(1).copied(), 0.0)?;
    let date = date_number_argument(cx, arguments.get(2).copied(), 1.0)?;
    let hour = date_number_argument(cx, arguments.get(3).copied(), 0.0)?;
    let minute = date_number_argument(cx, arguments.get(4).copied(), 0.0)?;
    let second = date_number_argument(cx, arguments.get(5).copied(), 0.0)?;
    let millisecond = date_number_argument(cx, arguments.get(6).copied(), 0.0)?;
    Ok(date_make_utc_value(
        year,
        month,
        date,
        hour,
        minute,
        second,
        millisecond,
    ))
}

fn date_utc_parts_from_millis(millis: f64) -> Option<DateParts> {
    let millis = date_finite_integer(millis)?;
    let day = millis.div_euclid(DATE_MS_PER_DAY);
    let time = millis.rem_euclid(DATE_MS_PER_DAY);
    let (year, month, date) = date_civil_from_days(day)?;
    let (hour, minute, second, millisecond) = date_split_time_millis(time);
    Some(DateParts {
        year,
        month,
        day: date,
        hour,
        minute,
        second,
        millisecond,
        weekday: date_weekday_from_days(day),
        offset_minutes: 0,
    })
}

fn date_local_parts_from_millis<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    millis: f64,
) -> Result<Option<DateParts>, Cx::Error> {
    let Some(epoch_nanoseconds) = date_value_epoch_nanoseconds(Value::from_f64(millis)) else {
        return Ok(None);
    };
    let time_zone = cx.temporal_default_time_zone(&TemporalDefaultTimeZoneRequest {})?;
    let civil_time = cx.temporal_instant_to_civil_time(&TemporalInstantToCivilRequest {
        time_zone_id: time_zone.time_zone_id,
        epoch_nanoseconds,
    })?;
    let date_time = civil_time.date_time;
    let day = date_days_from_civil(date_time.year, date_time.month, date_time.day);
    Ok(Some(DateParts {
        year: date_time.year,
        month: date_time.month,
        day: date_time.day,
        hour: date_time.hour,
        minute: date_time.minute,
        second: date_time.second,
        millisecond: date_time.millisecond,
        weekday: date_weekday_from_days(day),
        offset_minutes: i32::try_from(civil_time.offset_nanoseconds / DATE_NANOS_PER_MINUTE)
            .unwrap_or(0),
    }))
}

fn date_parts_for_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    utc: bool,
) -> Result<Option<DateParts>, Cx::Error> {
    let Some(millis) = value.as_f64().filter(|millis| millis.is_finite()) else {
        return Ok(None);
    };
    if utc {
        Ok(date_utc_parts_from_millis(millis))
    } else {
        date_local_parts_from_millis(cx, millis)
    }
}

fn date_format_year_for_date_string(year: i32) -> String {
    if (0..=9999).contains(&year) {
        format!("{year:04}")
    } else if year < 0 && year > -10_000 {
        format!("-{:04}", year.abs())
    } else {
        year.to_string()
    }
}

fn date_format_year_for_iso(year: i32) -> String {
    if (0..=9999).contains(&year) {
        format!("{year:04}")
    } else if year < 0 {
        format!("-{:06}", year.abs())
    } else {
        format!("+{year:06}")
    }
}

fn date_format_date(parts: DateParts) -> String {
    format!(
        "{} {} {:02} {}",
        DATE_WEEKDAY_NAMES[usize::from(parts.weekday)],
        DATE_MONTH_NAMES[usize::from(parts.month - 1)],
        parts.day,
        date_format_year_for_date_string(parts.year)
    )
}

fn date_format_time(parts: DateParts) -> String {
    let offset = parts.offset_minutes;
    let sign = if offset < 0 { '-' } else { '+' };
    let abs_offset = offset.abs();
    format!(
        "{:02}:{:02}:{:02} GMT{}{:02}{:02}",
        parts.hour,
        parts.minute,
        parts.second,
        sign,
        abs_offset / 60,
        abs_offset % 60
    )
}

fn date_format_local<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    kind: DateStringKind,
) -> Result<String, Cx::Error> {
    let Some(parts) = date_parts_for_value(cx, value, false)? else {
        return Ok("Invalid Date".to_owned());
    };
    Ok(match kind {
        DateStringKind::Full => format!("{} {}", date_format_date(parts), date_format_time(parts)),
        DateStringKind::Date => date_format_date(parts),
        DateStringKind::Time => date_format_time(parts),
    })
}

fn date_format_utc(value: Value) -> String {
    let Some(millis) = value.as_f64().filter(|millis| millis.is_finite()) else {
        return "Invalid Date".to_owned();
    };
    let Some(parts) = date_utc_parts_from_millis(millis) else {
        return "Invalid Date".to_owned();
    };
    format!(
        "{}, {:02} {} {} {:02}:{:02}:{:02} GMT",
        DATE_WEEKDAY_NAMES[usize::from(parts.weekday)],
        parts.day,
        DATE_MONTH_NAMES[usize::from(parts.month - 1)],
        date_format_year_for_date_string(parts.year),
        parts.hour,
        parts.minute,
        parts.second
    )
}

fn date_format_iso(value: Value) -> Option<String> {
    let millis = value.as_f64().filter(|millis| millis.is_finite())?;
    let parts = date_utc_parts_from_millis(millis)?;
    Some(format!(
        "{}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        date_format_year_for_iso(parts.year),
        parts.month,
        parts.day,
        parts.hour,
        parts.minute,
        parts.second,
        parts.millisecond
    ))
}

fn date_this_object_and_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<(ObjectRef, Value), Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let date_value = {
        let agent = cx.agent();
        if !agent.objects().is_date_object(object) {
            return Err(type_error(cx));
        }
        agent.objects().date_value(agent.heap().view(), object)
    };
    Ok((object, date_value.ok_or_else(|| type_error(cx))?))
}

fn date_store_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    value: Value,
) -> Result<(), Cx::Error> {
    let stored = cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.set_date_value(&mut mutator, object, value)
    });
    if stored {
        Ok(())
    } else {
        Err(type_error(cx))
    }
}

fn date_parse_two_digits(bytes: &[u8], index: usize) -> Option<u32> {
    let tens = *bytes.get(index)?;
    let ones = *bytes.get(index + 1)?;
    if !tens.is_ascii_digit() || !ones.is_ascii_digit() {
        return None;
    }
    Some(u32::from(tens - b'0') * 10 + u32::from(ones - b'0'))
}

fn date_parse_fixed_digits(bytes: &[u8], index: usize, len: usize) -> Option<i32> {
    let mut value = 0_i32;
    for offset in 0..len {
        let byte = *bytes.get(index + offset)?;
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(i32::from(byte - b'0'))?;
    }
    Some(value)
}

fn date_month_name_index(name: &str) -> Option<u8> {
    DATE_MONTH_NAMES
        .iter()
        .position(|candidate| *candidate == name)
        .and_then(|index| u8::try_from(index + 1).ok())
}

fn date_parse_time(text: &str) -> Option<(u32, u32, u32)> {
    let bytes = text.as_bytes();
    if bytes.len() != 8 || bytes.get(2) != Some(&b':') || bytes.get(5) != Some(&b':') {
        return None;
    }
    Some((
        date_parse_two_digits(bytes, 0)?,
        date_parse_two_digits(bytes, 3)?,
        date_parse_two_digits(bytes, 6)?,
    ))
}

fn date_parse_timezone_offset_colon(text: &str) -> Option<i32> {
    let bytes = text.as_bytes();
    if bytes.len() != 6 || bytes.get(3) != Some(&b':') {
        return None;
    }
    let sign = match bytes[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let hour = i32::try_from(date_parse_two_digits(bytes, 1)?).ok()?;
    let minute = i32::try_from(date_parse_two_digits(bytes, 4)?).ok()?;
    if hour > 23 || minute > 59 {
        return None;
    }
    Some(sign * (hour * 60 + minute))
}

fn date_parse_timezone_offset_compact(text: &str) -> Option<i32> {
    let bytes = text.as_bytes();
    if bytes.len() != 5 {
        return None;
    }
    let sign = match bytes[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let hour = i32::try_from(date_parse_two_digits(bytes, 1)?).ok()?;
    let minute = i32::try_from(date_parse_two_digits(bytes, 3)?).ok()?;
    if hour > 23 || minute > 59 {
        return None;
    }
    Some(sign * (hour * 60 + minute))
}

fn date_validate_iso_date(year: i32, month: u32, day: u32) -> bool {
    if !(1..=12).contains(&month) {
        return false;
    }
    let Ok(month_u8) = u8::try_from(month) else {
        return false;
    };
    (1..=u32::from(date_days_in_month(year, month_u8))).contains(&day)
}

fn date_parse_iso_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    text: &str,
) -> Result<Option<Value>, Cx::Error> {
    let bytes = text.as_bytes();
    let mut index = 0;
    let mut sign = 1_i32;
    let year_digits = match bytes.first().copied() {
        Some(b'+') => {
            index = 1;
            6
        }
        Some(b'-') => {
            index = 1;
            sign = -1;
            6
        }
        _ => 4,
    };
    let Some(mut year) = date_parse_fixed_digits(bytes, index, year_digits) else {
        return Ok(None);
    };
    if sign == -1 && year == 0 && year_digits == 6 {
        return Ok(None);
    }
    year *= sign;
    index += year_digits;

    let mut month = 1_u32;
    let mut day = 1_u32;
    let mut date_only = true;
    if bytes.get(index) == Some(&b'-') {
        index += 1;
        month = date_parse_two_digits(bytes, index).unwrap_or(0);
        index += 2;
        if bytes.get(index) == Some(&b'-') {
            index += 1;
            day = date_parse_two_digits(bytes, index).unwrap_or(0);
            index += 2;
        }
    }
    if !date_validate_iso_date(year, month, day) {
        return Ok(None);
    }

    let mut hour = 0_u32;
    let mut minute = 0_u32;
    let mut second = 0_u32;
    let mut millisecond = 0_u32;
    let mut offset_minutes: Option<i32> = None;

    if bytes.get(index) == Some(&b'T') {
        date_only = false;
        index += 1;
        hour = date_parse_two_digits(bytes, index).unwrap_or(u32::MAX);
        index += 2;
        if bytes.get(index) != Some(&b':') {
            return Ok(None);
        }
        index += 1;
        minute = date_parse_two_digits(bytes, index).unwrap_or(u32::MAX);
        index += 2;
        if bytes.get(index) == Some(&b':') {
            index += 1;
            second = date_parse_two_digits(bytes, index).unwrap_or(u32::MAX);
            index += 2;
            if bytes.get(index) == Some(&b'.') {
                index += 1;
                let mut scale = 100;
                while let Some(byte) = bytes.get(index).copied() {
                    if !byte.is_ascii_digit() {
                        break;
                    }
                    if scale > 0 {
                        millisecond += u32::from(byte - b'0') * scale;
                        scale /= 10;
                    }
                    index += 1;
                }
            }
        }
        if hour > 24
            || minute > 59
            || second > 59
            || (hour == 24 && (minute != 0 || second != 0 || millisecond != 0))
        {
            return Ok(None);
        }
        match bytes.get(index).copied() {
            Some(b'Z') => {
                offset_minutes = Some(0);
                index += 1;
            }
            Some(b'+') | Some(b'-') => {
                let offset_text = &text[index..];
                offset_minutes = date_parse_timezone_offset_colon(offset_text);
                if offset_minutes.is_none() {
                    return Ok(None);
                }
                index = text.len();
            }
            _ => {}
        }
    }
    if index != text.len() {
        return Ok(None);
    }

    let value = if let Some(offset) = offset_minutes {
        let utc = date_make_utc_value(
            f64::from(year),
            f64::from(month - 1),
            f64::from(day),
            f64::from(hour),
            f64::from(minute),
            f64::from(second),
            f64::from(millisecond),
        );
        let Some(millis) = utc.as_f64().filter(|millis| millis.is_finite()) else {
            return Ok(Some(Value::from_f64(f64::NAN)));
        };
        date_time_clip_value(millis - f64::from(offset) * DATE_MS_PER_MINUTE as f64)
    } else if date_only {
        date_make_utc_value(
            f64::from(year),
            f64::from(month - 1),
            f64::from(day),
            0.0,
            0.0,
            0.0,
            0.0,
        )
    } else {
        date_make_local_value(
            cx,
            f64::from(year),
            f64::from(month - 1),
            f64::from(day),
            f64::from(hour),
            f64::from(minute),
            f64::from(second),
            f64::from(millisecond),
        )?
    };
    Ok(Some(value))
}

fn date_parse_utc_string(text: &str) -> Option<Value> {
    let parts: Vec<_> = text.split_whitespace().collect();
    if parts.len() != 6 || parts[5] != "GMT" {
        return None;
    }
    let day = parts[1].parse::<u32>().ok()?;
    let month = date_month_name_index(parts[2])?;
    let year = parts[3].parse::<i32>().ok()?;
    let (hour, minute, second) = date_parse_time(parts[4])?;
    Some(date_make_utc_value(
        f64::from(year),
        f64::from(month - 1),
        f64::from(day),
        f64::from(hour),
        f64::from(minute),
        f64::from(second),
        0.0,
    ))
}

fn date_parse_local_string(text: &str) -> Option<Value> {
    let parts: Vec<_> = text.split_whitespace().collect();
    if parts.len() < 6 || !parts[5].starts_with("GMT") {
        return None;
    }
    let month = date_month_name_index(parts[1])?;
    let day = parts[2].parse::<u32>().ok()?;
    let year = parts[3].parse::<i32>().ok()?;
    let (hour, minute, second) = date_parse_time(parts[4])?;
    let offset = date_parse_timezone_offset_compact(&parts[5][3..])?;
    let utc = date_make_utc_value(
        f64::from(year),
        f64::from(month - 1),
        f64::from(day),
        f64::from(hour),
        f64::from(minute),
        f64::from(second),
        0.0,
    );
    let millis = utc.as_f64().filter(|millis| millis.is_finite())?;
    Some(date_time_clip_value(
        millis - f64::from(offset) * DATE_MS_PER_MINUTE as f64,
    ))
}

fn date_parse_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    text: &str,
) -> Result<Value, Cx::Error> {
    if let Some(value) = date_parse_iso_text(cx, text)? {
        return Ok(value);
    }
    if let Some(value) = date_parse_utc_string(text) {
        return Ok(value);
    }
    if let Some(value) = date_parse_local_string(text) {
        return Ok(value);
    }
    Ok(Value::from_f64(f64::NAN))
}

fn date_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.new_target().is_none() {
        let text = date_display_text(cx, current_time_value())?;
        return Ok(string_value(cx, &text));
    }

    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().date_prototype())
    }
    .ok_or_else(|| type_error(cx))?;

    let time_value = if invocation.arguments().is_empty() {
        current_time_value()
    } else if invocation.arguments().len() == 1 {
        let argument = invocation.arguments()[0];
        if let Some(object) = argument.as_object_ref() {
            let date_value = {
                let agent = cx.agent();
                agent.objects().date_value(agent.heap().view(), object)
            };
            if let Some(date_value) = date_value {
                date_value
            } else {
                let primitive = {
                    let mut bridge = BuiltinToPrimitiveBridge { cx };
                    object::to_primitive(&mut bridge, argument, object::ToPrimitiveHint::Default)?
                };
                if primitive.is_string() {
                    let text = cx.value_to_string_text(primitive)?;
                    date_parse_text(cx, &text)?
                } else {
                    let number = {
                        let agent = cx.agent();
                        read::to_number(agent.heap().view(), primitive)
                    };
                    match number {
                        Ok(number) => date_time_clip_value(number.as_f64().unwrap_or(f64::NAN)),
                        Err(_) => return Err(type_error(cx)),
                    }
                }
            }
        } else if argument.is_string() {
            let text = cx.value_to_string_text(argument)?;
            date_parse_text(cx, &text)?
        } else {
            let number = to_number_for_builtin(cx, argument)?;
            date_time_clip_value(number)
        }
    } else {
        date_local_time_value_from_arguments(cx, invocation.arguments())?
    };

    let prototype =
        cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?;
    Ok(Value::from_object_ref(allocate_date_object(
        cx, realm, prototype, time_value,
    )?))
}

fn date_now_builtin<Cx: PublicBuiltinDispatchContext>(
    _cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Ok(current_time_value())
}

fn date_parse_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let text = cx.value_to_string_text(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    date_parse_text(cx, &text)
}

fn date_utc_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    date_utc_time_value_from_arguments(cx, invocation.arguments())
}

fn date_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: DateStringKind,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_date_value(agent, invocation.this_value())
    };
    let value = map_completion(cx, value)?;
    let text = date_format_local(cx, value, kind)?;
    Ok(string_value(cx, &text))
}

fn date_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_date_value(agent, invocation.this_value())
    };
    map_completion(cx, value)
}

fn date_get_component_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    component: DateComponent,
    utc: bool,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_date_value(agent, invocation.this_value())
    };
    let value = map_completion(cx, value)?;
    let Some(parts) = date_parts_for_value(cx, value, utc)? else {
        return Ok(Value::from_f64(f64::NAN));
    };
    let value = match component {
        DateComponent::FullYear => parts.year,
        DateComponent::Month => i32::from(parts.month - 1),
        DateComponent::Date => i32::from(parts.day),
        DateComponent::Day => i32::from(parts.weekday),
        DateComponent::Hours => i32::from(parts.hour),
        DateComponent::Minutes => i32::from(parts.minute),
        DateComponent::Seconds => i32::from(parts.second),
        DateComponent::Milliseconds => i32::from(parts.millisecond),
    };
    Ok(Value::from_smi(value))
}

fn date_get_timezone_offset_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_date_value(agent, invocation.this_value())
    };
    let value = map_completion(cx, value)?;
    let Some(epoch_nanoseconds) = date_value_epoch_nanoseconds(value) else {
        return Ok(Value::from_f64(f64::NAN));
    };
    let time_zone = cx.temporal_default_time_zone(&TemporalDefaultTimeZoneRequest {})?;
    let civil_time = cx.temporal_instant_to_civil_time(&TemporalInstantToCivilRequest {
        time_zone_id: time_zone.time_zone_id,
        epoch_nanoseconds,
    })?;
    let offset_minutes = -((civil_time.offset_nanoseconds / DATE_NANOS_PER_MINUTE) as f64);
    Ok(Value::from_f64(offset_minutes + 0.0))
}

fn date_set_time_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (object, _) = date_this_object_and_value(cx, invocation.this_value())?;
    let time = to_number_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let value = date_time_clip_value(time);
    date_store_value(cx, object, value)?;
    Ok(value)
}

fn date_set_component_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: DateSetKind,
    utc: bool,
) -> Result<Value, Cx::Error> {
    let (object, old_value) = date_this_object_and_value(cx, invocation.this_value())?;
    let old_millis = old_value.as_f64().unwrap_or(f64::NAN);
    let first = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let first_number = to_number_for_builtin(cx, first)?;
    let second_number = match invocation.arguments().get(1).copied() {
        Some(value) => Some(to_number_for_builtin(cx, value)?),
        None => None,
    };
    let third_number = match invocation.arguments().get(2).copied() {
        Some(value) => Some(to_number_for_builtin(cx, value)?),
        None => None,
    };
    let fourth_number = match invocation.arguments().get(3).copied() {
        Some(value) => Some(to_number_for_builtin(cx, value)?),
        None => None,
    };

    let base_millis = if kind == DateSetKind::FullYear && old_millis.is_nan() {
        0.0
    } else {
        old_millis
    };
    if kind != DateSetKind::FullYear && !base_millis.is_finite() {
        return Ok(Value::from_f64(f64::NAN));
    }
    let parts = if utc {
        date_utc_parts_from_millis(base_millis)
    } else {
        date_local_parts_from_millis(cx, base_millis)?
    };
    let Some(parts) = parts else {
        return Ok(Value::from_f64(f64::NAN));
    };

    let mut year = f64::from(parts.year);
    let mut month = f64::from(parts.month - 1);
    let mut date = f64::from(parts.day);
    let mut hour = f64::from(parts.hour);
    let mut minute = f64::from(parts.minute);
    let mut second = f64::from(parts.second);
    let mut millisecond = f64::from(parts.millisecond);

    match kind {
        DateSetKind::Milliseconds => {
            millisecond = first_number;
        }
        DateSetKind::Seconds => {
            second = first_number;
            millisecond = second_number.unwrap_or(millisecond);
        }
        DateSetKind::Minutes => {
            minute = first_number;
            second = second_number.unwrap_or(second);
            millisecond = third_number.unwrap_or(millisecond);
        }
        DateSetKind::Hours => {
            hour = first_number;
            minute = second_number.unwrap_or(minute);
            second = third_number.unwrap_or(second);
            millisecond = fourth_number.unwrap_or(millisecond);
        }
        DateSetKind::Date => {
            date = first_number;
        }
        DateSetKind::Month => {
            month = first_number;
            date = second_number.unwrap_or(date);
        }
        DateSetKind::FullYear => {
            year = first_number;
            month = second_number.unwrap_or(month);
            date = third_number.unwrap_or(date);
        }
    }

    let value = if utc {
        date_make_utc_value(year, month, date, hour, minute, second, millisecond)
    } else {
        date_make_local_value(cx, year, month, date, hour, minute, second, millisecond)?
    };
    date_store_value(cx, object, value)?;
    Ok(value)
}

fn date_to_utc_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_date_value(agent, invocation.this_value())
    };
    let value = map_completion(cx, value)?;
    Ok(string_value(cx, &date_format_utc(value)))
}

fn date_to_iso_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_date_value(agent, invocation.this_value())
    };
    let value = map_completion(cx, value)?;
    let Some(text) = date_format_iso(value) else {
        return Err(range_error(cx));
    };
    Ok(string_value(cx, &text))
}

fn date_to_json_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = {
        let realm = cx.builtin_realm();
        let agent = cx.agent();
        object::to_object(agent, realm, invocation.this_value())
    };
    let object = map_completion(cx, object)?;
    let primitive = {
        let mut bridge = BuiltinToPrimitiveBridge { cx };
        object::to_primitive(
            &mut bridge,
            Value::from_object_ref(object),
            object::ToPrimitiveHint::Number,
        )?
    };
    if primitive.as_f64().is_some_and(|number| !number.is_finite()) {
        return Ok(Value::null());
    }
    let key = {
        let agent = cx.agent();
        PropertyKey::from_atom(agent.atoms_mut().intern_collectible("toISOString"))
    };
    let method = cx.get_property_value(Value::from_object_ref(object), key)?;
    let method = cx.require_callable_object(method)?;
    cx.call_to_completion(method, Value::from_object_ref(object), &[])
}

fn date_to_temporal_instant_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_date_value(agent, invocation.this_value())
    };
    let value = map_completion(cx, value)?;
    let Some(epoch_nanoseconds) = date_value_epoch_nanoseconds(value) else {
        return Err(range_error(cx));
    };
    temporal::create_temporal_instant_object(cx, epoch_nanoseconds)
}

fn date_to_primitive_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let hint_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let hint = hint_value.as_string_ref().ok_or_else(|| type_error(cx))?;
    let hint_text = string_ref_text(cx, hint)?;
    let hint = match hint_text.as_str() {
        "string" | "default" => object::ToPrimitiveHint::String,
        "number" => object::ToPrimitiveHint::Number,
        _ => return Err(type_error(cx)),
    };
    object::ordinary_to_primitive(&mut BuiltinToPrimitiveBridge { cx }, object, hint)
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

fn number_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let number = if let Some(argument) = invocation.arguments().first().copied() {
        let primitive = {
            let mut bridge = BuiltinToPrimitiveBridge { cx };
            object::to_primitive(&mut bridge, argument, object::ToPrimitiveHint::Number)?
        };
        if primitive.is_bigint() {
            bigint_to_number_value(cx, primitive)?
        } else {
            let number = {
                let agent = cx.agent();
                read::to_number(agent.heap().view(), primitive)
            };
            match number {
                Ok(number) => number,
                Err(_) => return Err(type_error(cx)),
            }
        }
    } else {
        Value::from_smi(0)
    };
    if invocation.new_target().is_none() {
        return Ok(number);
    }
    let realm = cx.builtin_realm();
    let default_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().number_prototype())
        .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?;
    primitive_wrapper_constructor(cx, realm, prototype, PrimitiveWrapperKind::Number, number)
}

fn number_is_finite_builtin<Cx: PublicBuiltinDispatchContext>(
    _cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let result = invocation
        .arguments()
        .first()
        .copied()
        .and_then(Value::as_f64)
        .is_some_and(f64::is_finite);
    Ok(Value::from_bool(result))
}

fn number_is_integer_builtin<Cx: PublicBuiltinDispatchContext>(
    _cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let result = invocation
        .arguments()
        .first()
        .copied()
        .and_then(Value::as_f64)
        .is_some_and(is_integral_number);
    Ok(Value::from_bool(result))
}

fn number_is_nan_builtin<Cx: PublicBuiltinDispatchContext>(
    _cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let result = invocation
        .arguments()
        .first()
        .copied()
        .and_then(Value::as_f64)
        .is_some_and(f64::is_nan);
    Ok(Value::from_bool(result))
}

fn number_is_safe_integer_builtin<Cx: PublicBuiltinDispatchContext>(
    _cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let result = invocation
        .arguments()
        .first()
        .copied()
        .and_then(Value::as_f64)
        .is_some_and(|number| {
            is_integral_number(number) && number.abs() <= 9_007_199_254_740_991.0
        });
    Ok(Value::from_bool(result))
}

fn number_this_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: &BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::Number,
        )
    };
    map_completion(cx, value)
}

fn number_this_f64<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: &BuiltinInvocation<'_>,
) -> Result<(Value, f64), Cx::Error> {
    let value = number_this_value(cx, invocation)?;
    let number = value.as_f64().ok_or_else(|| type_error(cx))?;
    Ok((value, number))
}

fn number_format_digits<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    min: i32,
    max: i32,
) -> Result<usize, Cx::Error> {
    let digits = to_integer_or_infinity_for_builtin(cx, value)?;
    if !digits.is_finite() || digits < f64::from(min) || digits > f64::from(max) {
        return Err(range_error(cx));
    }
    usize::try_from(digits as i32).map_err(|_| range_error(cx))
}

fn number_to_exponential_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (number_value, number) = number_this_f64(cx, &invocation)?;
    let fraction_digits = match invocation.arguments().first().copied() {
        None => None,
        Some(value) if value.is_undefined() => None,
        Some(value) => Some(to_integer_or_infinity_for_builtin(cx, value)?),
    };
    if number.is_nan() || number.is_infinite() {
        let text = cx.value_to_string_text(number_value)?;
        return Ok(string_value(cx, &text));
    }
    let fraction_digits = match fraction_digits {
        None => None,
        Some(digits) if digits.is_finite() && (0.0..=100.0).contains(&digits) => {
            Some(usize::try_from(digits as i32).map_err(|_| range_error(cx))?)
        }
        Some(_) => return Err(range_error(cx)),
    };
    let normalized = if number == 0.0 { 0.0 } else { number };
    let formatted = if let Some(fraction_digits) = fraction_digits {
        format_to_exponential(normalized, fraction_digits).ok_or_else(|| type_error(cx))?
    } else {
        format!("{normalized:e}")
    };
    let (mantissa, exponent) = formatted.split_once('e').ok_or_else(|| type_error(cx))?;
    let exponent = exponent.parse::<i32>().map_err(|_| type_error(cx))?;
    Ok(string_value(cx, &format!("{mantissa}e{exponent:+}")))
}

fn number_to_fixed_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (number_value, number) = number_this_f64(cx, &invocation)?;
    let fraction_digits = match invocation.arguments().first().copied() {
        None => 0,
        Some(value) => number_format_digits(cx, value, 0, 100)?,
    };
    if number.is_nan() || number.is_infinite() || number.abs() >= 1e21 {
        let text = cx.value_to_string_text(number_value)?;
        return Ok(string_value(cx, &text));
    }
    let normalized = if number == 0.0 { 0.0 } else { number };
    Ok(string_value(cx, &format!("{normalized:.fraction_digits$}")))
}

fn number_to_locale_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = number_this_value(cx, &invocation)?;
    let text = cx.value_to_string_text(value)?;
    Ok(string_value(cx, &text))
}

fn number_to_precision_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (number_value, number) = number_this_f64(cx, &invocation)?;
    let Some(precision_value) = invocation.arguments().first().copied() else {
        let text = cx.value_to_string_text(number_value)?;
        return Ok(string_value(cx, &text));
    };
    if precision_value.is_undefined() {
        let text = cx.value_to_string_text(number_value)?;
        return Ok(string_value(cx, &text));
    }

    let precision_integer = to_integer_or_infinity_for_builtin(cx, precision_value)?;
    if number.is_nan() || number.is_infinite() {
        let text = cx.value_to_string_text(number_value)?;
        return Ok(string_value(cx, &text));
    }
    if !precision_integer.is_finite() || !(1.0..=100.0).contains(&precision_integer) {
        return Err(range_error(cx));
    }
    let precision = usize::try_from(precision_integer as i32).map_err(|_| range_error(cx))?;
    let text = format_to_precision(number, precision).ok_or_else(|| type_error(cx))?;
    Ok(string_value(cx, &text))
}

fn number_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let number_value = number_this_value(cx, &invocation)?;
    let number = number_value.as_f64().ok_or_else(|| type_error(cx))?;
    let radix = radix_argument(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let text = if radix == 10 || !number.is_finite() {
        cx.value_to_string_text(number_value)?
    } else {
        object::integral_number_to_radix_string(number, radix).unwrap_or_else(|| number.to_string())
    };
    Ok(string_value(cx, &text))
}

fn number_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::Number,
        )
    };
    map_completion(cx, value)
}

fn math_argument(invocation: &BuiltinInvocation<'_>, index: usize) -> Value {
    invocation
        .arguments()
        .get(index)
        .copied()
        .unwrap_or(Value::undefined())
}

fn math_unary_number_builtin<Cx, F>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    op: F,
) -> Result<Value, Cx::Error>
where
    Cx: PublicBuiltinDispatchContext,
    F: FnOnce(f64) -> f64,
{
    let number = argument_to_number(cx, math_argument(&invocation, 0))?;
    Ok(number_value(op(number)))
}

fn math_abs_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::abs)
}

fn math_acos_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::acos)
}

fn math_acosh_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::acosh)
}

fn math_asin_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::asin)
}

fn math_asinh_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::asinh)
}

fn math_atan_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::atan)
}

fn math_atan2_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let y = argument_to_number(cx, math_argument(&invocation, 0))?;
    let x = argument_to_number(cx, math_argument(&invocation, 1))?;
    Ok(number_value(y.atan2(x)))
}

fn math_atanh_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::atanh)
}

fn math_cbrt_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::cbrt)
}

fn math_ceil_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::ceil)
}

fn math_clz32_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = to_uint32_for_builtin(cx, math_argument(&invocation, 0))?;
    Ok(Value::from_smi(
        i32::try_from(value.leading_zeros()).expect("leading zero count should fit in i32"),
    ))
}

fn math_cos_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::cos)
}

fn math_cosh_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::cosh)
}

fn math_exp_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::exp)
}

fn math_expm1_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::exp_m1)
}

fn math_f16round_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let number = argument_to_number(cx, math_argument(&invocation, 0))?;
    Ok(number_value(round_to_float16(number)))
}

fn math_floor_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::floor)
}

fn math_fround_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let number = argument_to_number(cx, math_argument(&invocation, 0))?;
    Ok(number_value(f64::from(number as f32)))
}

fn math_hypot_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let mut values = Vec::with_capacity(invocation.arguments().len());
    let mut saw_infinity = false;
    let mut saw_nan = false;
    for argument in invocation.arguments() {
        let number = argument_to_number(cx, *argument)?;
        if number.is_infinite() {
            saw_infinity = true;
        } else if number.is_nan() {
            saw_nan = true;
        } else {
            values.push(number.abs());
        }
    }
    if saw_infinity {
        return Ok(Value::from_f64(f64::INFINITY));
    }
    if saw_nan {
        return Ok(Value::from_f64(f64::NAN));
    }
    let scale = values.iter().copied().fold(0.0_f64, f64::max);
    if scale == 0.0 {
        return Ok(Value::from_smi(0));
    }
    let scaled_sum = values
        .iter()
        .map(|value| {
            let scaled = *value / scale;
            scaled * scaled
        })
        .sum::<f64>();
    Ok(number_value(scale * scaled_sum.sqrt()))
}

fn math_imul_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = to_uint32_for_builtin(cx, math_argument(&invocation, 0))?;
    let right = to_uint32_for_builtin(cx, math_argument(&invocation, 1))?;
    Ok(Value::from_smi(left.wrapping_mul(right) as i32))
}

fn math_log_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::ln)
}

fn math_log10_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::log10)
}

fn math_log1p_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::ln_1p)
}

fn math_log2_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::log2)
}

fn math_max_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.arguments().is_empty() {
        return Ok(Value::from_f64(f64::NEG_INFINITY));
    }
    let mut result = f64::NEG_INFINITY;
    let mut saw_nan = false;
    for argument in invocation.arguments() {
        let number = argument_to_number(cx, *argument)?;
        if number.is_nan() {
            saw_nan = true;
            continue;
        }
        if number > result
            || (number == 0.0
                && result == 0.0
                && result.is_sign_negative()
                && number.is_sign_positive())
        {
            result = number;
        }
    }
    if saw_nan {
        return Ok(Value::from_f64(f64::NAN));
    }
    Ok(number_value(result))
}

fn math_min_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.arguments().is_empty() {
        return Ok(Value::from_f64(f64::INFINITY));
    }
    let mut result = f64::INFINITY;
    let mut saw_nan = false;
    for argument in invocation.arguments() {
        let number = argument_to_number(cx, *argument)?;
        if number.is_nan() {
            saw_nan = true;
            continue;
        }
        if number < result
            || (number == 0.0
                && result == 0.0
                && result.is_sign_positive()
                && number.is_sign_negative())
        {
            result = number;
        }
    }
    if saw_nan {
        return Ok(Value::from_f64(f64::NAN));
    }
    Ok(number_value(result))
}

fn math_pow_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let base = argument_to_number(cx, math_argument(&invocation, 0))?;
    let exponent = argument_to_number(cx, math_argument(&invocation, 1))?;
    if exponent.is_nan() || (base.abs() == 1.0 && exponent.is_infinite()) {
        return Ok(Value::from_f64(f64::NAN));
    }
    Ok(number_value(base.powf(exponent)))
}

fn math_random_builtin<Cx: PublicBuiltinDispatchContext>(
    _cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0);
    let mut mixed = seed ^ seed.rotate_left(25) ^ 0x9e37_79b9_7f4a_7c15;
    mixed ^= mixed >> 30;
    mixed = mixed.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    mixed ^= mixed >> 27;
    mixed = mixed.wrapping_mul(0x94d0_49bb_1331_11eb);
    mixed ^= mixed >> 31;
    let mantissa = mixed >> 11;
    Ok(Value::from_f64(mantissa as f64 / ((1_u64 << 53) as f64)))
}

fn math_round_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let number = argument_to_number(cx, math_argument(&invocation, 0))?;
    if !number.is_finite() || number == 0.0 {
        return Ok(number_value(number));
    }
    if number < 0.0 && number >= -0.5 {
        return Ok(Value::from_f64(-0.0));
    }
    let floor = number.floor();
    if number - floor < 0.5 {
        Ok(number_value(floor))
    } else {
        Ok(number_value(floor + 1.0))
    }
}

fn math_sign_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let number = argument_to_number(cx, math_argument(&invocation, 0))?;
    if number.is_nan() || number == 0.0 {
        return Ok(number_value(number));
    }
    Ok(number_value(if number.is_sign_negative() {
        -1.0
    } else {
        1.0
    }))
}

fn math_sin_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::sin)
}

fn math_sinh_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::sinh)
}

fn math_sqrt_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::sqrt)
}

fn math_sum_precise_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let iterable = math_argument(&invocation, 0);
    let mut iterator_record = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::get_iterator(&mut bridge, iterable)?
    };
    let mut numbers = Vec::new();
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
            return Ok(number_value(math_sum_precise_numbers(&numbers)));
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let Some(number) = value.as_f64() else {
            let error = type_error(cx);
            return close_iterator_after_error(cx, &mut iterator_record, error);
        };
        numbers.push(number);
    }
}

fn math_tan_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::tan)
}

fn math_tanh_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::tanh)
}

fn math_trunc_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::trunc)
}

fn round_to_float16(number: f64) -> f64 {
    if number.is_nan() || number == 0.0 || number.is_infinite() {
        return number;
    }

    const MIN_SUBNORMAL: f64 = 5.960_464_477_539_063e-8;
    const MIN_NORMAL: f64 = 0.000_061_035_156_25;
    const MAX_FINITE: f64 = 65_504.0;
    const INFINITY_THRESHOLD: f64 = 65_520.0;

    let negative = number.is_sign_negative();
    let magnitude = number.abs();
    let rounded = if magnitude >= INFINITY_THRESHOLD {
        f64::INFINITY
    } else if magnitude < MIN_NORMAL {
        round_ties_to_even(magnitude / MIN_SUBNORMAL) * MIN_SUBNORMAL
    } else {
        let exponent = magnitude.log2().floor() as i32;
        let step = 2.0_f64.powi(exponent - 10);
        let candidate = round_ties_to_even(magnitude / step) * step;
        candidate.min(MAX_FINITE)
    };

    if negative {
        -rounded
    } else {
        rounded
    }
}

fn round_ties_to_even(value: f64) -> f64 {
    let floor = value.floor();
    let fraction = value - floor;
    if fraction < 0.5 {
        floor
    } else if fraction > 0.5 || (floor as u64) % 2 == 1 {
        floor + 1.0
    } else {
        floor
    }
}

fn math_sum_precise_numbers(numbers: &[f64]) -> f64 {
    let mut saw_nan = false;
    let mut saw_positive_infinity = false;
    let mut saw_negative_infinity = false;
    let mut saw_positive_zero = false;
    let mut saw_negative_zero = false;
    let mut finite = Vec::new();

    for number in numbers {
        if number.is_nan() {
            saw_nan = true;
        } else if *number == f64::INFINITY {
            saw_positive_infinity = true;
        } else if *number == f64::NEG_INFINITY {
            saw_negative_infinity = true;
        } else if *number == 0.0 {
            if number.is_sign_negative() {
                saw_negative_zero = true;
            } else {
                saw_positive_zero = true;
            }
        } else {
            finite.push(*number);
        }
    }

    if saw_nan || (saw_positive_infinity && saw_negative_infinity) {
        return f64::NAN;
    }
    if saw_positive_infinity {
        return f64::INFINITY;
    }
    if saw_negative_infinity {
        return f64::NEG_INFINITY;
    }
    if finite.is_empty() {
        if saw_positive_zero {
            return 0.0;
        }
        return if saw_negative_zero || numbers.is_empty() {
            -0.0
        } else {
            0.0
        };
    }

    let result = math_precise_finite_sum(&finite);
    if result == 0.0 {
        0.0
    } else {
        result
    }
}

fn math_precise_finite_sum(numbers: &[f64]) -> f64 {
    let mut terms = Vec::with_capacity(numbers.len());
    let mut min_exponent = 0;
    for number in numbers {
        if let Some(term) = MathFiniteTerm::from_f64(*number) {
            if terms.is_empty() || term.exponent < min_exponent {
                min_exponent = term.exponent;
            }
            terms.push(term);
        }
    }

    let mut exact = MathExactSum::default();
    for term in terms {
        exact.add_term(
            term.negative,
            term.mantissa,
            usize::try_from(term.exponent - min_exponent)
                .expect("finite term shift should be non-negative"),
        );
    }
    exact.to_f64(min_exponent)
}

struct MathFiniteTerm {
    negative: bool,
    mantissa: u64,
    exponent: i32,
}

impl MathFiniteTerm {
    fn from_f64(number: f64) -> Option<Self> {
        let bits = number.to_bits();
        let negative = (bits >> 63) != 0;
        let exponent_bits = ((bits >> 52) & 0x7ff) as u16;
        let fraction = bits & ((1_u64 << 52) - 1);
        if exponent_bits == 0 {
            if fraction == 0 {
                return None;
            }
            return Some(Self {
                negative,
                mantissa: fraction,
                exponent: -1074,
            });
        }

        Some(Self {
            negative,
            mantissa: (1_u64 << 52) | fraction,
            exponent: i32::from(exponent_bits) - 1075,
        })
    }
}

#[derive(Default)]
struct MathExactSum {
    sign: i8,
    limbs: Vec<u64>,
}

impl MathExactSum {
    fn add_term(&mut self, negative: bool, mantissa: u64, shift: usize) {
        let magnitude = shifted_magnitude(mantissa, shift);
        if magnitude.is_empty() {
            return;
        }
        let term_sign = if negative { -1 } else { 1 };
        if self.sign == 0 {
            self.sign = term_sign;
            self.limbs = magnitude;
            return;
        }
        if self.sign == term_sign {
            add_magnitude(&mut self.limbs, &magnitude);
            return;
        }

        match compare_magnitude(&self.limbs, &magnitude) {
            std::cmp::Ordering::Greater => {
                subtract_magnitude(&mut self.limbs, &magnitude);
            }
            std::cmp::Ordering::Less => {
                let mut replacement = magnitude;
                subtract_magnitude(&mut replacement, &self.limbs);
                self.limbs = replacement;
                self.sign = term_sign;
            }
            std::cmp::Ordering::Equal => {
                self.limbs.clear();
                self.sign = 0;
            }
        }
    }

    fn to_f64(&self, exponent: i32) -> f64 {
        if self.sign == 0 {
            return 0.0;
        }

        let bit_len = magnitude_bit_len(&self.limbs);
        let highest_exponent =
            exponent + i32::try_from(bit_len).expect("bit length should fit") - 1;
        let magnitude = if highest_exponent < -1022 {
            round_subnormal_magnitude_to_f64(&self.limbs, exponent)
        } else {
            round_normal_magnitude_to_f64(&self.limbs, bit_len, highest_exponent)
        };

        if self.sign < 0 {
            -magnitude
        } else {
            magnitude
        }
    }
}

fn shifted_magnitude(mantissa: u64, shift: usize) -> Vec<u64> {
    let limb_shift = shift / 64;
    let bit_shift = shift % 64;
    let mut limbs = vec![0; limb_shift];
    if bit_shift == 0 {
        limbs.push(mantissa);
    } else {
        limbs.push(mantissa << bit_shift);
        let high = mantissa >> (64 - bit_shift);
        if high != 0 {
            limbs.push(high);
        }
    }
    normalize_magnitude(&mut limbs);
    limbs
}

fn normalize_magnitude(limbs: &mut Vec<u64>) {
    while limbs.last().copied() == Some(0) {
        limbs.pop();
    }
}

fn compare_magnitude(left: &[u64], right: &[u64]) -> std::cmp::Ordering {
    if left.len() != right.len() {
        return left.len().cmp(&right.len());
    }
    for (left_limb, right_limb) in left.iter().zip(right.iter()).rev() {
        if left_limb != right_limb {
            return left_limb.cmp(right_limb);
        }
    }
    std::cmp::Ordering::Equal
}

fn add_magnitude(target: &mut Vec<u64>, addend: &[u64]) {
    if target.len() < addend.len() {
        target.resize(addend.len(), 0);
    }
    let mut carry = 0_u128;
    for index in 0..target.len() {
        let addend_limb = addend.get(index).copied().unwrap_or(0);
        let sum = u128::from(target[index]) + u128::from(addend_limb) + carry;
        target[index] = sum as u64;
        carry = sum >> 64;
    }
    if carry != 0 {
        target.push(carry as u64);
    }
}

fn subtract_magnitude(target: &mut Vec<u64>, subtrahend: &[u64]) {
    let mut borrow = 0_i128;
    for (index, target_limb) in target.iter_mut().enumerate() {
        let left = i128::from(*target_limb) - borrow;
        let right = i128::from(subtrahend.get(index).copied().unwrap_or(0));
        if left >= right {
            *target_limb = (left - right) as u64;
            borrow = 0;
        } else {
            *target_limb = ((1_i128 << 64) + left - right) as u64;
            borrow = 1;
        }
    }
    debug_assert_eq!(borrow, 0);
    normalize_magnitude(target);
}

fn magnitude_bit_len(limbs: &[u64]) -> usize {
    let Some(last) = limbs.last() else {
        return 0;
    };
    (limbs.len() - 1) * 64
        + usize::try_from(64 - last.leading_zeros()).expect("bit count should fit in usize")
}

fn round_normal_magnitude_to_f64(limbs: &[u64], bit_len: usize, mut exponent: i32) -> f64 {
    if exponent > 1023 {
        return f64::INFINITY;
    }

    let mut significand = if bit_len > 53 {
        round_shift_to_u64(limbs, bit_len - 53)
    } else {
        magnitude_to_u64(limbs) << (53 - bit_len)
    };
    if significand == (1_u64 << 53) {
        significand >>= 1;
        exponent += 1;
    }
    if exponent > 1023 {
        return f64::INFINITY;
    }

    let exponent_bits = u64::try_from(exponent + 1023).expect("normal exponent should fit");
    let fraction = significand - (1_u64 << 52);
    f64::from_bits((exponent_bits << 52) | fraction)
}

fn round_subnormal_magnitude_to_f64(limbs: &[u64], exponent: i32) -> f64 {
    let scale = exponent + 1074;
    let fraction = if scale >= 0 {
        magnitude_to_u64(limbs) << usize::try_from(scale).expect("scale should fit")
    } else {
        round_shift_to_u64(
            limbs,
            usize::try_from(-scale).expect("right shift should fit"),
        )
    };
    if fraction == 0 {
        return 0.0;
    }
    if fraction >= (1_u64 << 52) {
        return f64::from_bits(1_u64 << 52);
    }
    f64::from_bits(fraction)
}

fn magnitude_to_u64(limbs: &[u64]) -> u64 {
    debug_assert!(limbs.iter().skip(1).all(|limb| *limb == 0));
    limbs.first().copied().unwrap_or(0)
}

fn round_shift_to_u64(limbs: &[u64], shift: usize) -> u64 {
    if shift == 0 {
        return magnitude_to_u64(limbs);
    }

    let mut quotient = magnitude_shr_to_u64(limbs, shift);
    let half = magnitude_bit(limbs, shift - 1);
    let sticky = magnitude_any_bits_below(limbs, shift - 1);
    if half && (sticky || quotient % 2 == 1) {
        quotient += 1;
    }
    quotient
}

fn magnitude_shr_to_u64(limbs: &[u64], shift: usize) -> u64 {
    let limb_shift = shift / 64;
    let bit_shift = shift % 64;
    let Some(low) = limbs.get(limb_shift).copied() else {
        return 0;
    };
    let mut result = low >> bit_shift;
    if bit_shift != 0 {
        if let Some(high) = limbs.get(limb_shift + 1).copied() {
            result |= high << (64 - bit_shift);
        }
    }
    result
}

fn magnitude_bit(limbs: &[u64], bit: usize) -> bool {
    let limb = bit / 64;
    let offset = bit % 64;
    limbs
        .get(limb)
        .map(|value| ((value >> offset) & 1) != 0)
        .unwrap_or(false)
}

fn magnitude_any_bits_below(limbs: &[u64], bit_count: usize) -> bool {
    let full_limbs = bit_count / 64;
    for limb in limbs.iter().take(full_limbs) {
        if *limb != 0 {
            return true;
        }
    }
    let remaining_bits = bit_count % 64;
    if remaining_bits == 0 {
        return false;
    }
    let mask = (1_u64 << remaining_bits) - 1;
    limbs
        .get(full_limbs)
        .map(|limb| (limb & mask) != 0)
        .unwrap_or(false)
}

fn bigint_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.new_target().is_some() {
        return Err(type_error(cx));
    }
    let argument = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let primitive = {
        let mut bridge = BuiltinToPrimitiveBridge { cx };
        object::to_primitive(&mut bridge, argument, object::ToPrimitiveHint::Number)?
    };
    let bigint = {
        let agent = cx.agent();
        object::primitive_to_bigint(agent, primitive)
    };
    map_completion(cx, bigint)
}

const BIGINT_WIDTH_EXACT_LIMIT_BITS: u64 = 4_096;

fn bigint_as_int_n_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let bits = to_index_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let bigint = to_bigint_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if bits == 0 {
        return Ok(bigint_zero_value(cx));
    }
    let (sign, limbs) = bigint_parts(cx.agent(), bigint).ok_or_else(|| type_error(cx))?;
    if bigint_fits_signed_width(sign, &limbs, bits) {
        return Ok(bigint);
    }
    if bits > BIGINT_WIDTH_EXACT_LIMIT_BITS {
        return Err(range_error(cx));
    }
    let unsigned = bigint_to_uint_n_limbs(sign, &limbs, bits);
    let negative = bigint_width_sign_bit(&unsigned, bits);
    if negative {
        let magnitude = twos_complement_width_magnitude(unsigned, bits);
        Ok(bigint_from_parts(cx, BigIntSign::Negative, &magnitude))
    } else {
        Ok(bigint_from_parts(cx, BigIntSign::NonNegative, &unsigned))
    }
}

fn bigint_as_uint_n_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let bits = to_index_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let bigint = to_bigint_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if bits == 0 {
        return Ok(bigint_zero_value(cx));
    }
    let (sign, limbs) = bigint_parts(cx.agent(), bigint).ok_or_else(|| type_error(cx))?;
    if sign == BigIntSign::NonNegative && bigint_magnitude_bit_length(&limbs) <= bits {
        return Ok(bigint);
    }
    if bits > BIGINT_WIDTH_EXACT_LIMIT_BITS {
        return Err(range_error(cx));
    }
    let unsigned = bigint_to_uint_n_limbs(sign, &limbs, bits);
    Ok(bigint_from_parts(cx, BigIntSign::NonNegative, &unsigned))
}

fn bigint_parts(agent: &Agent, value: Value) -> Option<(BigIntSign, Vec<u64>)> {
    let bigint = value.as_bigint_ref()?;
    let view = agent.heap().view().bigint_view(bigint)?;
    let mut limbs = Vec::with_capacity(view.limb_count() as usize);
    for index in 0..view.limb_count() {
        limbs.push(view.limb_at(index as usize).unwrap_or(0));
    }
    normalize_bigint_limbs(&mut limbs);
    Some((normalize_bigint_sign(view.sign(), &limbs), limbs))
}

fn bigint_zero_value<Cx: PublicBuiltinDispatchContext>(cx: &mut Cx) -> Value {
    bigint_from_parts(cx, BigIntSign::NonNegative, &[])
}

fn bigint_from_parts<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    sign: BigIntSign,
    limbs: &[u64],
) -> Value {
    let mut normalized = limbs.to_vec();
    normalize_bigint_limbs(&mut normalized);
    let sign = normalize_bigint_sign(sign, &normalized);
    let bigint = cx.agent().heap_mut().mutator().alloc_bigint(
        sign,
        &normalized,
        AllocationLifetime::Default,
    );
    Value::from_bigint_ref(bigint)
}

fn bigint_to_uint_n_limbs(sign: BigIntSign, limbs: &[u64], bits: u64) -> Vec<u64> {
    let width = usize::try_from(bits.div_ceil(64)).expect("exact BigInt width should fit");
    let mut result = vec![0; width.max(1)];
    let copied = result.len().min(limbs.len());
    result[..copied].copy_from_slice(&limbs[..copied]);
    if normalize_bigint_sign(sign, limbs) == BigIntSign::Negative {
        for limb in &mut result {
            *limb = !*limb;
        }
        add_one_to_limbs(&mut result);
    }
    mask_bigint_width(&mut result, bits);
    normalize_bigint_limbs(&mut result);
    result
}

fn twos_complement_width_magnitude(mut bits: Vec<u64>, width_bits: u64) -> Vec<u64> {
    for limb in &mut bits {
        *limb = !*limb;
    }
    mask_bigint_width(&mut bits, width_bits);
    add_one_to_limbs(&mut bits);
    mask_bigint_width(&mut bits, width_bits);
    normalize_bigint_limbs(&mut bits);
    bits
}

fn bigint_width_sign_bit(limbs: &[u64], bits: u64) -> bool {
    let bit = bits - 1;
    let limb_index = usize::try_from(bit / 64).expect("exact BigInt width should fit");
    let bit_index = bit % 64;
    limbs
        .get(limb_index)
        .is_some_and(|limb| (limb & (1_u64 << bit_index)) != 0)
}

fn mask_bigint_width(limbs: &mut [u64], bits: u64) {
    let remainder = bits % 64;
    if remainder == 0 {
        return;
    }
    if let Some(last) = limbs.last_mut() {
        *last &= (1_u64 << remainder) - 1;
    }
}

fn add_one_to_limbs(limbs: &mut [u64]) {
    let mut carry = true;
    for limb in limbs {
        if !carry {
            return;
        }
        let (next, overflow) = limb.overflowing_add(1);
        *limb = next;
        carry = overflow;
    }
}

fn normalize_bigint_limbs(limbs: &mut Vec<u64>) {
    while limbs.last() == Some(&0) {
        limbs.pop();
    }
}

fn normalize_bigint_sign(sign: BigIntSign, limbs: &[u64]) -> BigIntSign {
    if limbs.is_empty() {
        BigIntSign::NonNegative
    } else {
        sign
    }
}

fn bigint_magnitude_bit_length(limbs: &[u64]) -> u64 {
    let Some(last) = limbs.last().copied() else {
        return 0;
    };
    let high_bits = 64 - u64::from(last.leading_zeros());
    ((limbs.len() as u64 - 1) * 64) + high_bits
}

fn bigint_fits_signed_width(sign: BigIntSign, limbs: &[u64], bits: u64) -> bool {
    match normalize_bigint_sign(sign, limbs) {
        BigIntSign::NonNegative => bigint_magnitude_bit_length(limbs) < bits,
        BigIntSign::Negative => bigint_magnitude_le_power_of_two(limbs, bits - 1),
    }
}

fn bigint_magnitude_le_power_of_two(limbs: &[u64], exponent: u64) -> bool {
    let limb_index = usize::try_from(exponent / 64).expect("BigInt exponent should fit");
    if limbs.len() > limb_index + 1 {
        return false;
    }
    if limbs.len() <= limb_index {
        return true;
    }
    let bit = exponent % 64;
    let limit = 1_u64 << bit;
    let high = limbs[limb_index];
    high < limit || (high == limit && limbs[..limb_index].iter().all(|limb| *limb == 0))
}

fn bigint_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::BigInt,
        )
    };
    let bigint_value = map_completion(cx, value)?;
    let radix = radix_argument(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let text = {
        let agent = cx.agent();
        object::bigint_to_string(agent, bigint_value, radix)
    };
    let text = map_completion(cx, text)?;
    Ok(string_value(cx, &text))
}

fn bigint_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::BigInt,
        )
    };
    map_completion(cx, value)
}

fn boolean_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let argument = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let boolean = {
        let agent = cx.agent();
        read::to_boolean(agent.heap().view(), argument)
    };
    let boolean = map_completion(cx, boolean)?;
    if invocation.new_target().is_none() {
        return Ok(Value::from_bool(boolean));
    }
    let realm = cx.builtin_realm();
    let default_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().boolean_prototype())
        .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?;
    primitive_wrapper_constructor(
        cx,
        realm,
        prototype,
        PrimitiveWrapperKind::Boolean,
        Value::from_bool(boolean),
    )
}

fn boolean_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::Boolean,
        )
    };
    let value = map_completion(cx, value)?;
    Ok(string_value(
        cx,
        if value.as_bool() == Some(true) {
            "true"
        } else {
            "false"
        },
    ))
}

fn boolean_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::Boolean,
        )
    };
    map_completion(cx, value)
}

fn symbol_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.new_target().is_some() {
        return Err(type_error(cx));
    }
    let description = match invocation.arguments().first().copied() {
        Some(value) if !value.is_undefined() => {
            let text = cx.value_to_string_text(value)?;
            let description = {
                let agent = cx.agent();
                agent.alloc_runtime_string(&text, None, AllocationLifetime::Default)
            };
            Some(description)
        }
        _ => None,
    };
    let symbol = {
        let agent = cx.agent();
        agent.heap_mut().mutator().alloc_symbol(
            description,
            SymbolFlags::ordinary(),
            AllocationLifetime::Default,
        )
    };
    Ok(Value::from_symbol_ref(symbol))
}

fn symbol_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let key_text = cx.value_to_string_text(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let symbol = {
        let agent = cx.agent();
        let key = agent.atoms_mut().intern_collectible(&key_text);
        agent.global_symbol_for(key, AllocationLifetime::Default)
    };
    Ok(Value::from_symbol_ref(symbol))
}

fn symbol_key_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let symbol = invocation
        .arguments()
        .first()
        .copied()
        .and_then(Value::as_symbol_ref)
        .ok_or_else(|| type_error(cx))?;
    let Some(key) = ({
        let agent = cx.agent();
        agent.global_symbol_key_for(symbol)
    }) else {
        return Ok(Value::undefined());
    };
    let value = {
        let agent = cx.agent();
        Value::from_string_ref(agent.alloc_runtime_string(
            "",
            Some(key),
            AllocationLifetime::Default,
        ))
    };
    Ok(value)
}

fn symbol_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::Symbol,
        )
    };
    let symbol = map_completion(cx, value)?
        .as_symbol_ref()
        .ok_or_else(|| type_error(cx))?;
    let text = symbol_descriptive_string(cx, symbol)?;
    Ok(string_value(cx, &text))
}

fn symbol_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::Symbol,
        )
    };
    map_completion(cx, value)
}

fn symbol_description_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::Symbol,
        )
    };
    let symbol = map_completion(cx, value)?
        .as_symbol_ref()
        .ok_or_else(|| type_error(cx))?;
    let description = {
        let agent = cx.agent();
        let heap_view = agent.heap().view();
        heap_view
            .symbol_view(symbol)
            .and_then(|view| view.description())
            .map(Value::from_string_ref)
            .unwrap_or(Value::undefined())
    };
    Ok(description)
}

fn symbol_to_primitive_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    symbol_value_of_builtin(cx, invocation)
}

fn error_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: errors::ErrorKind,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        errors::intrinsic_error_prototype_for_realm(agent, realm, kind)
    }
    .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?;
    let message = optional_error_message_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let options = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let error_object = {
        let agent = cx.agent();
        errors::create_error_object(agent, realm, Some(prototype), message)
    };
    let error_object = map_completion(cx, error_object)?;
    install_error_cause(cx, error_object, options)?;
    Ok(Value::from_object_ref(error_object))
}

fn optional_error_message_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Option<Value>, Cx::Error> {
    if value.is_undefined() {
        return Ok(None);
    }
    let text = cx.value_to_string_text(value)?;
    Ok(Some(string_value(cx, &text)))
}

fn install_error_cause<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    error: ObjectRef,
    options: Value,
) -> Result<(), Cx::Error> {
    let Some(options_object) = options.as_object_ref() else {
        return Ok(());
    };
    let cause_key = property_key_from_text(cx, "cause");
    if !has_property_on_object(cx, options_object, cause_key)? {
        return Ok(());
    }
    let cause = cx.get_property_value(Value::from_object_ref(options_object), cause_key)?;
    define_data_property_with_attrs(cx, error, cause_key, cause, true, false, true)
}

fn error_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    error_constructor_builtin(cx, invocation, errors::ErrorKind::Error)
}

fn eval_error_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    error_constructor_builtin(cx, invocation, errors::ErrorKind::Eval)
}

fn range_error_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    error_constructor_builtin(cx, invocation, errors::ErrorKind::Range)
}

fn reference_error_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    error_constructor_builtin(cx, invocation, errors::ErrorKind::Reference)
}

fn syntax_error_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    error_constructor_builtin(cx, invocation, errors::ErrorKind::Syntax)
}

fn type_error_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    error_constructor_builtin(cx, invocation, errors::ErrorKind::Type)
}

fn uri_error_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    error_constructor_builtin(cx, invocation, errors::ErrorKind::Uri)
}

fn aggregate_error_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let default_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().aggregate_error_prototype())
        .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?;
    let errors_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let message = optional_error_message_value(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let options = invocation
        .arguments()
        .get(2)
        .copied()
        .unwrap_or(Value::undefined());
    let error = {
        let agent = cx.agent();
        errors::create_error_object(agent, realm, Some(prototype), message)
    };
    let error = map_completion(cx, error)?;
    install_error_cause(cx, error, options)?;
    let values = iterable_to_values_list(cx, errors_value)?;
    let errors_array = create_array_from_values(cx, &values)?;
    let errors_key = property_key_from_text(cx, "errors");
    define_data_property_with_attrs(
        cx,
        error,
        errors_key,
        Value::from_object_ref(errors_array),
        true,
        false,
        true,
    )?;
    Ok(Value::from_object_ref(error))
}

fn suppressed_error_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let default_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().suppressed_error_prototype())
        .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?;
    let error_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let suppressed_value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let message = optional_error_message_value(
        cx,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let options = invocation
        .arguments()
        .get(3)
        .copied()
        .unwrap_or(Value::undefined());
    let error = create_suppressed_error_with_prototype(
        cx,
        prototype,
        error_value,
        suppressed_value,
        message,
        options,
    )?;
    Ok(Value::from_object_ref(error))
}

fn disposal_capability_payload_value(id: lyng_js_env::DisposalCapabilityId) -> Value {
    i32::try_from(id.get())
        .map(Value::from_smi)
        .unwrap_or_else(|_| Value::from_f64(f64::from(id.get())))
}

fn disposal_stack_default_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<ObjectRef, Cx::Error> {
    let intrinsics = cx
        .agent()
        .realm(realm)
        .map(lyng_js_env::RealmRecord::intrinsics);
    let prototype = match kind {
        lyng_js_env::DisposalCapabilityKind::Sync => {
            intrinsics.and_then(lyng_js_env::Intrinsics::disposable_stack_prototype)
        }
        lyng_js_env::DisposalCapabilityKind::Async => {
            intrinsics.and_then(lyng_js_env::Intrinsics::async_disposable_stack_prototype)
        }
    };
    prototype.ok_or_else(|| type_error(cx))
}

fn create_disposal_stack_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: ObjectRef,
    capability: lyng_js_env::DisposalCapabilityId,
) -> Result<ObjectRef, Cx::Error> {
    let root_shape = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.root_shape())
        .ok_or_else(|| type_error(cx))?;
    let payload = disposal_capability_payload_value(capability);
    let object = cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_prototype(Some(prototype))
                .with_ordinary_payload_value(payload),
            AllocationLifetime::Default,
        )
    });
    let _ = cx
        .agent()
        .bind_disposal_capability_object(object, capability);
    Ok(object)
}

fn create_disposal_scope_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    capability: lyng_js_env::DisposalCapabilityId,
) -> Result<ObjectRef, Cx::Error> {
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().object_prototype())
        .ok_or_else(|| type_error(cx))?;
    create_disposal_stack_object(cx, realm, prototype, capability)
}

fn require_disposal_stack_receiver<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<
    (
        ObjectRef,
        lyng_js_env::DisposalCapabilityId,
        lyng_js_env::DisposalCapabilityState,
    ),
    Cx::Error,
> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let capability = cx
        .agent()
        .disposal_capability_id_for_object(object)
        .ok_or_else(|| type_error(cx))?;
    let record = match cx.agent().disposal_capability(capability) {
        Some(record) => record,
        None => return Err(type_error(cx)),
    };
    if record.kind() != kind {
        return Err(type_error(cx));
    }
    Ok((object, capability, record.state()))
}

fn require_pending_disposal_stack<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<(ObjectRef, lyng_js_env::DisposalCapabilityId), Cx::Error> {
    let (object, capability, state) = require_disposal_stack_receiver(cx, value, kind)?;
    if matches!(state, lyng_js_env::DisposalCapabilityState::Disposed) {
        return Err(reference_error(cx));
    }
    Ok((object, capability))
}

fn require_disposal_scope_receiver<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<
    (
        ObjectRef,
        lyng_js_env::DisposalCapabilityId,
        lyng_js_env::DisposalCapabilityRecord,
    ),
    Cx::Error,
> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let capability = cx
        .agent()
        .disposal_capability_id_for_object(object)
        .ok_or_else(|| type_error(cx))?;
    let Some(record) = cx.agent().disposal_capability(capability).cloned() else {
        return Err(type_error(cx));
    };
    Ok((object, capability, record))
}

fn require_pending_disposal_scope<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<
    (
        ObjectRef,
        lyng_js_env::DisposalCapabilityId,
        lyng_js_env::DisposalCapabilityRecord,
    ),
    Cx::Error,
> {
    let (object, capability, record) = require_disposal_scope_receiver(cx, value)?;
    if record.is_disposed() {
        return Err(reference_error(cx));
    }
    Ok((object, capability, record))
}

fn dispose_method_for_hint<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    async_hint: bool,
) -> Result<Option<(ObjectRef, lyng_js_env::DisposalMethodKind)>, Cx::Error> {
    if value.is_undefined() || value.is_null() {
        return Ok(None);
    }
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if async_hint {
        let async_dispose = cx
            .agent()
            .well_known_symbol(WellKnownSymbolId::AsyncDispose)
            .ok_or_else(|| type_error(cx))?;
        let method = cx.get_property_value(
            Value::from_object_ref(object),
            PropertyKey::from_symbol(async_dispose),
        )?;
        if !(method.is_undefined() || method.is_null()) {
            return Ok(Some((
                cx.require_callable_object(method)?,
                lyng_js_env::DisposalMethodKind::Async,
            )));
        }
    }
    let dispose = cx
        .agent()
        .well_known_symbol(WellKnownSymbolId::Dispose)
        .ok_or_else(|| type_error(cx))?;
    let method = cx.get_property_value(
        Value::from_object_ref(object),
        PropertyKey::from_symbol(dispose),
    )?;
    if method.is_undefined() || method.is_null() {
        return Err(type_error(cx));
    }
    Ok(Some((
        cx.require_callable_object(method)?,
        if async_hint {
            lyng_js_env::DisposalMethodKind::AsyncFromSync
        } else {
            lyng_js_env::DisposalMethodKind::Sync
        },
    )))
}

fn dispose_method_for_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<Option<(ObjectRef, lyng_js_env::DisposalMethodKind)>, Cx::Error> {
    dispose_method_for_hint(
        cx,
        value,
        matches!(kind, lyng_js_env::DisposalCapabilityKind::Async),
    )
}

fn create_suppressed_error_with_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: ObjectRef,
    error_value: Value,
    suppressed_value: Value,
    message: Option<Value>,
    options: Value,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    let error = {
        let agent = cx.agent();
        errors::create_error_object(agent, realm, Some(prototype), message)
    };
    let error = map_completion(cx, error)?;
    install_error_cause(cx, error, options)?;
    let error_key = property_key_from_text(cx, "error");
    define_data_property_with_attrs(cx, error, error_key, error_value, true, false, true)?;
    let suppressed_key = property_key_from_text(cx, "suppressed");
    define_data_property_with_attrs(
        cx,
        error,
        suppressed_key,
        suppressed_value,
        true,
        false,
        true,
    )?;
    Ok(error)
}

fn create_suppressed_error_from_values<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    error_value: Value,
    suppressed_value: Value,
    message: Option<Value>,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().suppressed_error_prototype())
        .ok_or_else(|| type_error(cx))?;
    create_suppressed_error_with_prototype(
        cx,
        prototype,
        error_value,
        suppressed_value,
        message,
        Value::undefined(),
    )
}

fn append_disposal_error<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    existing: Option<Value>,
    new_error: Value,
) -> Result<Value, Cx::Error> {
    let Some(existing) = existing else {
        return Ok(new_error);
    };
    let error = create_suppressed_error_from_values(cx, new_error, existing, None)?;
    Ok(Value::from_object_ref(error))
}

fn call_disposal_resource<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    resource: lyng_js_env::DisposableResourceRecord,
) -> Result<Value, Cx::Error> {
    match resource.kind() {
        lyng_js_env::DisposableResourceKind::UseMethod => {
            cx.call_to_completion(resource.callable(), resource.value(), &[])
        }
        lyng_js_env::DisposableResourceKind::CallbackWithValue => {
            cx.call_to_completion(resource.callable(), Value::undefined(), &[resource.value()])
        }
        lyng_js_env::DisposableResourceKind::CallbackWithoutValue => {
            cx.call_to_completion(resource.callable(), Value::undefined(), &[])
        }
    }
}

fn promise_for_async_disposal_result<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let constructor = promise_default_constructor(cx)?;
    let resolve = promise_resolve_method(cx, constructor)?;
    let promise = cx.call_to_completion(resolve, Value::from_object_ref(constructor), &[value])?;
    promise
        .as_object_ref()
        .filter(|object| cx.agent().promise_record(*object).is_some())
        .ok_or_else(|| type_error(cx))
}

fn allocate_async_disposal_resume_function<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    operation: lyng_js_env::AsyncDisposalOperationId,
    reject: bool,
) -> Result<ObjectRef, Cx::Error> {
    let function =
        cx.allocate_builtin_function(lyng_js_types::js3_async_disposal_resume_builtin())?;
    let _ = cx.agent().alloc_async_disposal_resume(
        function,
        lyng_js_env::AsyncDisposalResumeRecord::new(operation, reject),
    );
    Ok(function)
}

fn continue_async_disposal<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    operation_id: lyng_js_env::AsyncDisposalOperationId,
) -> Result<(), Cx::Error> {
    loop {
        let operation = cx
            .agent()
            .async_disposal_operation(operation_id)
            .ok_or_else(|| type_error(cx))?;
        if operation.completed() {
            return Ok(());
        }
        let capability = operation.capability();
        let Some(resource) = cx.agent().pop_disposal_resource(capability) else {
            let _ = cx
                .agent()
                .set_async_disposal_operation_completed(operation_id, true);
            if operation.has_disposal_error() {
                let error = operation
                    .pending_error()
                    .expect("disposal failures should seed a pending error");
                let reject = promise_capability_reject(cx, operation.promise_capability())?;
                let _ = cx.call_to_completion(reject, Value::undefined(), &[error])?;
            } else {
                let resolve = promise_capability_resolve(cx, operation.promise_capability())?;
                let _ =
                    cx.call_to_completion(resolve, Value::undefined(), &[Value::undefined()])?;
            }
            return Ok(());
        };

        let result = match call_disposal_resource(cx, resource) {
            Ok(result) => result,
            Err(error) => {
                let Some(thrown) = cx.extract_thrown_value(error)? else {
                    unreachable!("non-abrupt builtin error should propagate")
                };
                let pending = append_disposal_error(cx, operation.pending_error(), thrown)?;
                let _ = cx
                    .agent()
                    .set_async_disposal_operation_pending_error(operation_id, Some(pending));
                let _ = cx
                    .agent()
                    .set_async_disposal_operation_has_disposal_error(operation_id, true);
                continue;
            }
        };

        let promise = match promise_for_async_disposal_result(cx, result) {
            Ok(promise) => promise,
            Err(error) => {
                let Some(thrown) = cx.extract_thrown_value(error)? else {
                    unreachable!("non-abrupt builtin error should propagate")
                };
                let pending = append_disposal_error(cx, operation.pending_error(), thrown)?;
                let _ = cx
                    .agent()
                    .set_async_disposal_operation_pending_error(operation_id, Some(pending));
                let _ = cx
                    .agent()
                    .set_async_disposal_operation_has_disposal_error(operation_id, true);
                continue;
            }
        };

        let promise_record = cx
            .agent()
            .promise_record(promise)
            .cloned()
            .ok_or_else(|| type_error(cx))?;
        match promise_record.state() {
            PromiseState::Fulfilled => continue,
            PromiseState::Rejected => {
                let pending =
                    append_disposal_error(cx, operation.pending_error(), promise_record.result())?;
                let _ = cx
                    .agent()
                    .set_async_disposal_operation_pending_error(operation_id, Some(pending));
                let _ = cx
                    .agent()
                    .set_async_disposal_operation_has_disposal_error(operation_id, true);
            }
            PromiseState::Pending => {
                let on_fulfilled =
                    allocate_async_disposal_resume_function(cx, operation_id, false)?;
                let on_rejected = allocate_async_disposal_resume_function(cx, operation_id, true)?;
                let _ = cx
                    .agent()
                    .set_async_disposal_operation_waiting(operation_id, true);
                match invoke_then_method(
                    cx,
                    Value::from_object_ref(promise),
                    Value::from_object_ref(on_fulfilled),
                    Value::from_object_ref(on_rejected),
                ) {
                    Ok(_) => return Ok(()),
                    Err(error) => {
                        let _ = cx
                            .agent()
                            .set_async_disposal_operation_waiting(operation_id, false);
                        let Some(thrown) = cx.extract_thrown_value(error)? else {
                            unreachable!("non-abrupt builtin error should propagate")
                        };
                        let pending = append_disposal_error(cx, operation.pending_error(), thrown)?;
                        let _ = cx.agent().set_async_disposal_operation_pending_error(
                            operation_id,
                            Some(pending),
                        );
                        let _ = cx
                            .agent()
                            .set_async_disposal_operation_has_disposal_error(operation_id, true);
                    }
                }
            }
        }
    }
}

fn create_disposal_scope_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let capability = cx.agent().alloc_disposal_capability(kind);
    let object = create_disposal_scope_object(cx, realm, capability)?;
    Ok(Value::from_object_ref(object))
}

fn add_disposal_scope_resource_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    async_hint: bool,
) -> Result<Value, Cx::Error> {
    let scope = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let (_, capability, record) = require_pending_disposal_scope(cx, scope)?;
    if async_hint && record.kind() != lyng_js_env::DisposalCapabilityKind::Async {
        return Err(type_error(cx));
    }
    let Some((callable, method_kind)) = dispose_method_for_hint(cx, value, async_hint)? else {
        return Ok(value);
    };
    let _ = cx.agent().push_disposal_resource(
        capability,
        lyng_js_env::DisposableResourceRecord::use_method(value, callable, method_kind),
    );
    Ok(value)
}

fn dispose_scope_capability<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    capability: lyng_js_env::DisposalCapabilityId,
    prior_error: Option<Value>,
) -> Result<Value, Cx::Error> {
    let _ = cx
        .agent()
        .set_disposal_capability_state(capability, lyng_js_env::DisposalCapabilityState::Disposed);
    let mut pending_error = prior_error;
    let mut saw_disposal_error = false;
    while let Some(resource) = cx.agent().pop_disposal_resource(capability) {
        match call_disposal_resource(cx, resource) {
            Ok(_) => {}
            Err(error) => {
                let Some(thrown) = cx.extract_thrown_value(error)? else {
                    unreachable!("non-abrupt builtin error should propagate")
                };
                saw_disposal_error = true;
                pending_error = Some(append_disposal_error(cx, pending_error, thrown)?);
            }
        }
    }
    if saw_disposal_error {
        let thrown = pending_error.expect("disposal errors should seed a pending error");
        return Err(cx.abrupt(AbruptCompletion::throw(thrown)));
    }
    Ok(Value::undefined())
}

fn dispose_scope_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let scope = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let (_, capability, record) = require_disposal_scope_receiver(cx, scope)?;
    if record.is_disposed() {
        return Ok(Value::undefined());
    }
    if record.kind() != lyng_js_env::DisposalCapabilityKind::Sync {
        return Err(type_error(cx));
    }
    dispose_scope_capability(cx, capability, invocation.arguments().get(1).copied())
}

fn dispose_scope_async_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let promise_constructor = promise_default_constructor(cx)?;
    let promise_capability = new_promise_capability(cx, promise_constructor)?;
    let promise = promise_capability_promise(cx, promise_capability)?;
    let scope = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let (_, capability, record) = match require_disposal_scope_receiver(cx, scope) {
        Ok(record) => record,
        Err(_) => {
            let reject = promise_capability_reject(cx, promise_capability)?;
            let reason = errors::throw_type_error(cx.agent())
                .thrown_value()
                .unwrap_or(Value::undefined());
            let _ = cx.call_to_completion(reject, Value::undefined(), &[reason])?;
            return Ok(Value::from_object_ref(promise));
        }
    };
    if record.is_disposed() {
        let resolve = promise_capability_resolve(cx, promise_capability)?;
        let _ = cx.call_to_completion(resolve, Value::undefined(), &[Value::undefined()])?;
        return Ok(Value::from_object_ref(promise));
    }
    let _ = cx
        .agent()
        .set_disposal_capability_state(capability, lyng_js_env::DisposalCapabilityState::Disposed);
    let operation = cx
        .agent()
        .alloc_async_disposal_operation(capability, promise_capability);
    let prior_error = invocation.arguments().get(1).copied();
    let _ = cx
        .agent()
        .set_async_disposal_operation_pending_error(operation, prior_error);
    let _ = cx
        .agent()
        .set_async_disposal_operation_has_disposal_error(operation, false);
    continue_async_disposal(cx, operation)?;
    Ok(Value::from_object_ref(promise))
}

fn disposal_stack_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let default_prototype = disposal_stack_default_prototype(cx, realm, kind)?;
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let capability = cx.agent().alloc_disposal_capability(kind);
    let object = create_disposal_stack_object(cx, realm, prototype, capability)?;
    Ok(Value::from_object_ref(object))
}

fn disposable_stack_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_constructor_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Sync)
}

fn async_disposable_stack_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_constructor_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Async)
}

fn disposable_stack_disposed_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (_, capability, state) = require_disposal_stack_receiver(
        cx,
        invocation.this_value(),
        lyng_js_env::DisposalCapabilityKind::Sync,
    )?;
    let disposed = matches!(state, lyng_js_env::DisposalCapabilityState::Disposed)
        && cx.agent().disposal_capability(capability).is_some();
    Ok(Value::from_bool(disposed))
}

fn async_disposable_stack_disposed_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (_, capability, state) = require_disposal_stack_receiver(
        cx,
        invocation.this_value(),
        lyng_js_env::DisposalCapabilityKind::Async,
    )?;
    let disposed = matches!(state, lyng_js_env::DisposalCapabilityState::Disposed)
        && cx.agent().disposal_capability(capability).is_some();
    Ok(Value::from_bool(disposed))
}

fn disposal_stack_use_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<Value, Cx::Error> {
    let (_, capability) = require_pending_disposal_stack(cx, invocation.this_value(), kind)?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some((callable, method_kind)) = dispose_method_for_value(cx, value, kind)? else {
        return Ok(value);
    };
    let _ = cx.agent().push_disposal_resource(
        capability,
        lyng_js_env::DisposableResourceRecord::use_method(value, callable, method_kind),
    );
    Ok(value)
}

fn disposable_stack_use_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_use_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Sync)
}

fn async_disposable_stack_use_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_use_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Async)
}

fn disposal_stack_adopt_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<Value, Cx::Error> {
    let (_, capability) = require_pending_disposal_stack(cx, invocation.this_value(), kind)?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let on_dispose = cx.require_callable_object(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let method_kind = match kind {
        lyng_js_env::DisposalCapabilityKind::Sync => lyng_js_env::DisposalMethodKind::Sync,
        lyng_js_env::DisposalCapabilityKind::Async => lyng_js_env::DisposalMethodKind::Async,
    };
    let _ = cx.agent().push_disposal_resource(
        capability,
        lyng_js_env::DisposableResourceRecord::callback_with_value(value, on_dispose, method_kind),
    );
    Ok(value)
}

fn disposable_stack_adopt_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_adopt_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Sync)
}

fn async_disposable_stack_adopt_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_adopt_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Async)
}

fn disposal_stack_defer_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<Value, Cx::Error> {
    let (_, capability) = require_pending_disposal_stack(cx, invocation.this_value(), kind)?;
    let on_dispose = cx.require_callable_object(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let method_kind = match kind {
        lyng_js_env::DisposalCapabilityKind::Sync => lyng_js_env::DisposalMethodKind::Sync,
        lyng_js_env::DisposalCapabilityKind::Async => lyng_js_env::DisposalMethodKind::Async,
    };
    let _ = cx.agent().push_disposal_resource(
        capability,
        lyng_js_env::DisposableResourceRecord::callback_without_value(on_dispose, method_kind),
    );
    Ok(Value::undefined())
}

fn disposable_stack_defer_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_defer_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Sync)
}

fn async_disposable_stack_defer_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_defer_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Async)
}

fn disposal_stack_move_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<Value, Cx::Error> {
    let (_, capability) = require_pending_disposal_stack(cx, invocation.this_value(), kind)?;
    let realm = cx.builtin_realm();
    let prototype = disposal_stack_default_prototype(cx, realm, kind)?;
    let resources = cx
        .agent()
        .take_disposal_resources(capability)
        .ok_or_else(|| type_error(cx))?;
    let _ = cx
        .agent()
        .set_disposal_capability_state(capability, lyng_js_env::DisposalCapabilityState::Disposed);
    let new_capability = cx.agent().alloc_disposal_capability(kind);
    let _ = cx
        .agent()
        .replace_disposal_resources(new_capability, resources);
    let object = create_disposal_stack_object(cx, realm, prototype, new_capability)?;
    Ok(Value::from_object_ref(object))
}

fn disposable_stack_move_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_move_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Sync)
}

fn async_disposable_stack_move_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_move_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Async)
}

fn disposal_stack_dispose_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (_, capability, state) = require_disposal_stack_receiver(
        cx,
        invocation.this_value(),
        lyng_js_env::DisposalCapabilityKind::Sync,
    )?;
    if matches!(state, lyng_js_env::DisposalCapabilityState::Disposed) {
        return Ok(Value::undefined());
    }
    let _ = cx
        .agent()
        .set_disposal_capability_state(capability, lyng_js_env::DisposalCapabilityState::Disposed);
    let mut pending_error = None;
    while let Some(resource) = cx.agent().pop_disposal_resource(capability) {
        match call_disposal_resource(cx, resource) {
            Ok(_) => {}
            Err(error) => {
                let Some(thrown) = cx.extract_thrown_value(error)? else {
                    unreachable!("non-abrupt builtin error should propagate")
                };
                pending_error = Some(append_disposal_error(cx, pending_error, thrown)?);
            }
        }
    }
    if let Some(thrown) = pending_error {
        return Err(cx.abrupt(AbruptCompletion::throw(thrown)));
    }
    Ok(Value::undefined())
}

fn async_disposable_stack_dispose_async_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let promise_constructor = promise_default_constructor(cx)?;
    let promise_capability = new_promise_capability(cx, promise_constructor)?;
    let promise = promise_capability_promise(cx, promise_capability)?;
    let receiver = match invocation.this_value().as_object_ref() {
        Some(receiver) => receiver,
        None => {
            let reject = promise_capability_reject(cx, promise_capability)?;
            let reason = errors::throw_type_error(cx.agent())
                .thrown_value()
                .unwrap_or(Value::undefined());
            let _ = cx.call_to_completion(reject, Value::undefined(), &[reason])?;
            return Ok(Value::from_object_ref(promise));
        }
    };
    let Some(capability) = cx.agent().disposal_capability_id_for_object(receiver) else {
        let reject = promise_capability_reject(cx, promise_capability)?;
        let reason = errors::throw_type_error(cx.agent())
            .thrown_value()
            .unwrap_or(Value::undefined());
        let _ = cx.call_to_completion(reject, Value::undefined(), &[reason])?;
        return Ok(Value::from_object_ref(promise));
    };
    let Some(record) = cx.agent().disposal_capability(capability) else {
        let reject = promise_capability_reject(cx, promise_capability)?;
        let reason = errors::throw_type_error(cx.agent())
            .thrown_value()
            .unwrap_or(Value::undefined());
        let _ = cx.call_to_completion(reject, Value::undefined(), &[reason])?;
        return Ok(Value::from_object_ref(promise));
    };
    if !matches!(record.kind(), lyng_js_env::DisposalCapabilityKind::Async) {
        let reject = promise_capability_reject(cx, promise_capability)?;
        let reason = errors::throw_type_error(cx.agent())
            .thrown_value()
            .unwrap_or(Value::undefined());
        let _ = cx.call_to_completion(reject, Value::undefined(), &[reason])?;
        return Ok(Value::from_object_ref(promise));
    }
    if record.is_disposed() {
        let resolve = promise_capability_resolve(cx, promise_capability)?;
        let _ = cx.call_to_completion(resolve, Value::undefined(), &[Value::undefined()])?;
        return Ok(Value::from_object_ref(promise));
    }
    let _ = cx
        .agent()
        .set_disposal_capability_state(capability, lyng_js_env::DisposalCapabilityState::Disposed);
    let operation = cx
        .agent()
        .alloc_async_disposal_operation(capability, promise_capability);
    continue_async_disposal(cx, operation)?;
    Ok(Value::from_object_ref(promise))
}

fn async_disposal_resume_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let callee = cx.callee_object();
    let record = match cx.agent().async_disposal_resume(callee) {
        Some(record) => record,
        None => return Err(type_error(cx)),
    };
    let operation = match cx.agent().async_disposal_operation(record.operation()) {
        Some(operation) => operation,
        None => return Ok(Value::undefined()),
    };
    if operation.completed() || !operation.waiting() {
        return Ok(Value::undefined());
    }
    let _ = cx
        .agent()
        .set_async_disposal_operation_waiting(record.operation(), false);
    if record.reject() {
        let argument = invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined());
        let pending = append_disposal_error(cx, operation.pending_error(), argument)?;
        let _ = cx
            .agent()
            .set_async_disposal_operation_pending_error(record.operation(), Some(pending));
        let _ = cx
            .agent()
            .set_async_disposal_operation_has_disposal_error(record.operation(), true);
    }
    continue_async_disposal(cx, record.operation())?;
    Ok(Value::undefined())
}

fn eval_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(source_ref) = source.as_string_ref() else {
        return Ok(source);
    };
    let source_text = cx.value_to_string_text(Value::from_string_ref(source_ref))?;
    cx.evaluate_script_in_realm(cx.builtin_realm(), &source_text)
}

fn parse_int_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let input = cx.value_to_string_text(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let radix = to_number_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(number_value(parse_int_string(&input, radix)))
}

fn parse_float_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let input = cx.value_to_string_text(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(number_value(parse_float_string(&input)))
}

fn is_nan_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let numeric = to_number_value_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(numeric.as_f64().is_some_and(f64::is_nan)))
}

fn is_finite_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let numeric = to_number_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(numeric.is_finite()))
}

fn encode_uri_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    component: bool,
) -> Result<Value, Cx::Error> {
    let input = cx.value_to_string_text(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(string_value(cx, &encode_uri_text(&input, component)))
}

fn decode_uri_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    component: bool,
) -> Result<Value, Cx::Error> {
    let input = cx.value_to_string_text(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let decoded = decode_uri_text(&input, component).map_err(|_| uri_error(cx))?;
    Ok(string_value(cx, &decoded))
}

fn error_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let name = cx.get_property_value(
        Value::from_object_ref(object_ref),
        PropertyKey::from_atom(WellKnownAtom::name.id()),
    )?;
    let message = {
        let message_atom = {
            let agent = cx.agent();
            agent.bootstrap_atoms().message()
        };
        cx.get_property_value(
            Value::from_object_ref(object_ref),
            PropertyKey::from_atom(message_atom),
        )?
    };
    let name_text = if name.is_undefined() {
        "Error".to_owned()
    } else {
        cx.value_to_string_text(name)?
    };
    let message_text = if message.is_undefined() {
        String::new()
    } else {
        cx.value_to_string_text(message)?
    };
    let text = if name_text.is_empty() {
        message_text
    } else if message_text.is_empty() {
        name_text
    } else {
        format!("{name_text}: {message_text}")
    };
    Ok(string_value(cx, &text))
}

mod dispatch;
mod families;
mod metadata;
mod temporal;

pub use metadata::public_builtin_metadata;

use crate::bootstrap::install_descriptor_tables;
use crate::internal::{internal_builtin_metadata, InternalBuiltinCache, InternalRealmBuiltins};
use crate::{
    BuiltinAttributes, BuiltinDescriptorTable, BuiltinEntryMetadata, BuiltinInstallTarget,
    BuiltinIntrinsic, BuiltinPropertyDescriptor, BuiltinPropertyKeySpec, BuiltinPropertyValueSpec,
};
use lyng_js_common::{AtomId, WellKnownAtom};
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::{
    FunctionConstructorFlags, FunctionObjectData, FunctionThisMode, ObjectAllocation,
    ObjectColdData, PrimitiveWrapperKind,
};
use lyng_js_types::{
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
    js3_weak_set_delete_builtin, js3_weak_set_has_builtin, BuiltinFunctionId, EnvironmentRef,
    ObjectRef, PropertyKey, RealmRef, ShapeId, Value, WellKnownSymbolId,
};
use std::collections::HashMap;

pub use dispatch::{dispatch_builtin, PublicBuiltinDispatchContext};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RealmBuiltins {
    internal: InternalRealmBuiltins,
    public: PublicRealmBuiltins,
}

impl RealmBuiltins {
    #[inline]
    pub const fn internal(self) -> InternalRealmBuiltins {
        self.internal
    }

    #[inline]
    pub const fn public(self) -> PublicRealmBuiltins {
        self.public
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublicRealmBuiltins {
    object: ObjectRef,
    object_prototype: ObjectRef,
    object_create: ObjectRef,
    object_get_prototype_of: ObjectRef,
    object_set_prototype_of: ObjectRef,
    object_get_own_property_descriptor: ObjectRef,
    object_get_own_property_descriptors: ObjectRef,
    object_get_own_property_names: ObjectRef,
    object_get_own_property_symbols: ObjectRef,
    object_define_properties: ObjectRef,
    object_define_property: ObjectRef,
    object_assign: ObjectRef,
    object_from_entries: ObjectRef,
    object_group_by: ObjectRef,
    object_prevent_extensions: ObjectRef,
    object_is_extensible: ObjectRef,
    object_is: ObjectRef,
    object_seal: ObjectRef,
    object_freeze: ObjectRef,
    object_is_sealed: ObjectRef,
    object_is_frozen: ObjectRef,
    object_to_locale_string: ObjectRef,
    object_to_string: ObjectRef,
    object_value_of: ObjectRef,
    object_has_own_property: ObjectRef,
    object_is_prototype_of: ObjectRef,
    object_property_is_enumerable: ObjectRef,
    object_define_getter: ObjectRef,
    object_define_setter: ObjectRef,
    object_lookup_getter: ObjectRef,
    object_lookup_setter: ObjectRef,
    object_proto_getter: ObjectRef,
    object_proto_setter: ObjectRef,
    object_keys: ObjectRef,
    object_entries: ObjectRef,
    object_values: ObjectRef,
    object_has_own: ObjectRef,
    function: ObjectRef,
    function_prototype: ObjectRef,
    function_call: ObjectRef,
    function_apply: ObjectRef,
    function_bind: ObjectRef,
    function_to_string: ObjectRef,
    function_symbol_has_instance: ObjectRef,
    async_function: ObjectRef,
    async_function_prototype: ObjectRef,
    async_generator_function: ObjectRef,
    async_generator_function_prototype: ObjectRef,
    async_generator_prototype: ObjectRef,
    async_generator_next: ObjectRef,
    async_generator_return: ObjectRef,
    async_generator_throw: ObjectRef,
    generator_function: ObjectRef,
    generator_function_prototype: ObjectRef,
    generator_prototype: ObjectRef,
    generator_next: ObjectRef,
    generator_return: ObjectRef,
    generator_throw: ObjectRef,
    async_iterator_prototype: ObjectRef,
    array: ObjectRef,
    array_from: ObjectRef,
    array_from_async: ObjectRef,
    array_of: ObjectRef,
    array_unscopables: ObjectRef,
    map: ObjectRef,
    set: ObjectRef,
    weak_map: ObjectRef,
    weak_set: ObjectRef,
    weak_ref: ObjectRef,
    finalization_registry: ObjectRef,
    array_buffer: ObjectRef,
    shared_array_buffer: ObjectRef,
    atomics: ObjectRef,
    array_buffer_is_view: ObjectRef,
    data_view: ObjectRef,
    typed_array: ObjectRef,
    typed_array_from: ObjectRef,
    typed_array_of: ObjectRef,
    int8_array: ObjectRef,
    int16_array: ObjectRef,
    int32_array: ObjectRef,
    float32_array: ObjectRef,
    float64_array: ObjectRef,
    big_int64_array: ObjectRef,
    big_uint64_array: ObjectRef,
    uint32_array: ObjectRef,
    uint16_array: ObjectRef,
    uint8_clamped_array: ObjectRef,
    uint8_array: ObjectRef,
    array_is_array: ObjectRef,
    array_at: ObjectRef,
    array_concat: ObjectRef,
    array_copy_within: ObjectRef,
    array_fill: ObjectRef,
    array_flat: ObjectRef,
    array_flat_map: ObjectRef,
    array_join: ObjectRef,
    array_pop: ObjectRef,
    array_push: ObjectRef,
    array_shift: ObjectRef,
    array_unshift: ObjectRef,
    array_every: ObjectRef,
    array_filter: ObjectRef,
    array_find: ObjectRef,
    array_find_index: ObjectRef,
    array_find_last: ObjectRef,
    array_find_last_index: ObjectRef,
    array_for_each: ObjectRef,
    array_includes: ObjectRef,
    array_index_of: ObjectRef,
    array_map: ObjectRef,
    array_reduce: ObjectRef,
    array_reduce_right: ObjectRef,
    array_reverse: ObjectRef,
    array_slice: ObjectRef,
    array_some: ObjectRef,
    array_last_index_of: ObjectRef,
    array_sort: ObjectRef,
    array_splice: ObjectRef,
    array_to_reversed: ObjectRef,
    array_to_sorted: ObjectRef,
    array_to_spliced: ObjectRef,
    array_to_string: ObjectRef,
    array_to_locale_string: ObjectRef,
    array_values: ObjectRef,
    array_keys: ObjectRef,
    array_entries: ObjectRef,
    array_with: ObjectRef,
    map_get: ObjectRef,
    map_set: ObjectRef,
    map_has: ObjectRef,
    map_delete: ObjectRef,
    map_clear: ObjectRef,
    map_entries: ObjectRef,
    map_values: ObjectRef,
    map_keys: ObjectRef,
    map_for_each: ObjectRef,
    map_size_getter: ObjectRef,
    set_add: ObjectRef,
    set_has: ObjectRef,
    set_delete: ObjectRef,
    set_clear: ObjectRef,
    set_entries: ObjectRef,
    set_values: ObjectRef,
    set_keys: ObjectRef,
    set_for_each: ObjectRef,
    set_size_getter: ObjectRef,
    weak_map_get: ObjectRef,
    weak_map_set: ObjectRef,
    weak_map_has: ObjectRef,
    weak_map_delete: ObjectRef,
    weak_set_add: ObjectRef,
    weak_set_has: ObjectRef,
    weak_set_delete: ObjectRef,
    weak_ref_deref: ObjectRef,
    finalization_registry_register: ObjectRef,
    finalization_registry_unregister: ObjectRef,
    map_prototype: ObjectRef,
    set_prototype: ObjectRef,
    weak_map_prototype: ObjectRef,
    weak_set_prototype: ObjectRef,
    weak_ref_prototype: ObjectRef,
    finalization_registry_prototype: ObjectRef,
    array_buffer_prototype: ObjectRef,
    shared_array_buffer_prototype: ObjectRef,
    array_buffer_byte_length_getter: ObjectRef,
    array_buffer_slice: ObjectRef,
    shared_array_buffer_byte_length_getter: ObjectRef,
    shared_array_buffer_slice: ObjectRef,
    atomics_load: ObjectRef,
    atomics_store: ObjectRef,
    atomics_add: ObjectRef,
    atomics_sub: ObjectRef,
    atomics_and: ObjectRef,
    atomics_or: ObjectRef,
    atomics_xor: ObjectRef,
    atomics_exchange: ObjectRef,
    atomics_compare_exchange: ObjectRef,
    atomics_notify: ObjectRef,
    atomics_wait: ObjectRef,
    atomics_wait_async: ObjectRef,
    atomics_is_lock_free: ObjectRef,
    data_view_prototype: ObjectRef,
    data_view_buffer_getter: ObjectRef,
    data_view_byte_length_getter: ObjectRef,
    data_view_byte_offset_getter: ObjectRef,
    data_view_get_float32: ObjectRef,
    data_view_get_float64: ObjectRef,
    data_view_get_int16: ObjectRef,
    data_view_get_int32: ObjectRef,
    data_view_get_int8: ObjectRef,
    data_view_get_uint16: ObjectRef,
    data_view_get_uint32: ObjectRef,
    data_view_get_uint8: ObjectRef,
    data_view_set_float32: ObjectRef,
    data_view_set_float64: ObjectRef,
    data_view_set_int16: ObjectRef,
    data_view_set_int32: ObjectRef,
    data_view_set_int8: ObjectRef,
    data_view_set_uint16: ObjectRef,
    data_view_set_uint32: ObjectRef,
    data_view_set_uint8: ObjectRef,
    typed_array_prototype: ObjectRef,
    int8_array_prototype: ObjectRef,
    int16_array_prototype: ObjectRef,
    int32_array_prototype: ObjectRef,
    float32_array_prototype: ObjectRef,
    float64_array_prototype: ObjectRef,
    big_int64_array_prototype: ObjectRef,
    big_uint64_array_prototype: ObjectRef,
    uint32_array_prototype: ObjectRef,
    uint16_array_prototype: ObjectRef,
    uint8_clamped_array_prototype: ObjectRef,
    uint8_array_prototype: ObjectRef,
    uint8_array_buffer_getter: ObjectRef,
    uint8_array_byte_length_getter: ObjectRef,
    uint8_array_byte_offset_getter: ObjectRef,
    uint8_array_length_getter: ObjectRef,
    uint8_array_values: ObjectRef,
    uint8_array_keys: ObjectRef,
    uint8_array_entries: ObjectRef,
    uint8_array_set: ObjectRef,
    uint8_array_slice: ObjectRef,
    uint8_array_subarray: ObjectRef,
    typed_array_every: ObjectRef,
    typed_array_some: ObjectRef,
    typed_array_find: ObjectRef,
    typed_array_find_index: ObjectRef,
    typed_array_find_last: ObjectRef,
    typed_array_find_last_index: ObjectRef,
    typed_array_fill: ObjectRef,
    typed_array_copy_within: ObjectRef,
    typed_array_filter: ObjectRef,
    typed_array_for_each: ObjectRef,
    typed_array_includes: ObjectRef,
    typed_array_index_of: ObjectRef,
    typed_array_join: ObjectRef,
    typed_array_last_index_of: ObjectRef,
    typed_array_map: ObjectRef,
    typed_array_reduce: ObjectRef,
    typed_array_reduce_right: ObjectRef,
    typed_array_reverse: ObjectRef,
    typed_array_sort: ObjectRef,
    typed_array_to_locale_string: ObjectRef,
    typed_array_to_string: ObjectRef,
    typed_array_to_reversed: ObjectRef,
    typed_array_to_sorted: ObjectRef,
    typed_array_with: ObjectRef,
    typed_array_at: ObjectRef,
    typed_array_to_string_tag_getter: ObjectRef,
    iterator_prototype_iterator: ObjectRef,
    async_iterator_method: ObjectRef,
    array_iterator_next: ObjectRef,
    map_iterator_next: ObjectRef,
    set_iterator_next: ObjectRef,
    string: ObjectRef,
    string_prototype: ObjectRef,
    string_iterator: ObjectRef,
    string_iterator_next: ObjectRef,
    string_to_string: ObjectRef,
    string_value_of: ObjectRef,
    string_concat: ObjectRef,
    string_char_at: ObjectRef,
    string_char_code_at: ObjectRef,
    string_from_char_code: ObjectRef,
    string_from_code_point: ObjectRef,
    string_raw: ObjectRef,
    string_at: ObjectRef,
    string_code_point_at: ObjectRef,
    string_ends_with: ObjectRef,
    string_includes: ObjectRef,
    string_index_of: ObjectRef,
    string_is_well_formed: ObjectRef,
    string_locale_compare: ObjectRef,
    string_match: ObjectRef,
    string_match_all: ObjectRef,
    string_normalize: ObjectRef,
    string_last_index_of: ObjectRef,
    string_pad_end: ObjectRef,
    string_pad_start: ObjectRef,
    string_repeat: ObjectRef,
    string_replace: ObjectRef,
    string_replace_all: ObjectRef,
    string_search: ObjectRef,
    string_split: ObjectRef,
    string_slice: ObjectRef,
    string_substring: ObjectRef,
    string_starts_with: ObjectRef,
    string_to_locale_lower_case: ObjectRef,
    string_to_locale_upper_case: ObjectRef,
    string_to_lower_case: ObjectRef,
    string_to_upper_case: ObjectRef,
    string_to_well_formed: ObjectRef,
    string_trim: ObjectRef,
    string_trim_end: ObjectRef,
    string_trim_start: ObjectRef,
    regexp: ObjectRef,
    regexp_escape: ObjectRef,
    regexp_prototype: ObjectRef,
    regexp_to_string: ObjectRef,
    regexp_exec: ObjectRef,
    regexp_test: ObjectRef,
    regexp_global_getter: ObjectRef,
    regexp_ignore_case_getter: ObjectRef,
    regexp_multiline_getter: ObjectRef,
    regexp_dot_all_getter: ObjectRef,
    regexp_unicode_getter: ObjectRef,
    regexp_sticky_getter: ObjectRef,
    regexp_source_getter: ObjectRef,
    regexp_flags_getter: ObjectRef,
    regexp_has_indices_getter: ObjectRef,
    regexp_species_getter: ObjectRef,
    regexp_symbol_match: ObjectRef,
    regexp_symbol_replace: ObjectRef,
    regexp_symbol_search: ObjectRef,
    regexp_symbol_split: ObjectRef,
    regexp_symbol_match_all: ObjectRef,
    date: ObjectRef,
    date_prototype: ObjectRef,
    date_now: ObjectRef,
    date_parse: ObjectRef,
    date_utc: ObjectRef,
    date_to_string: ObjectRef,
    date_to_date_string: ObjectRef,
    date_to_time_string: ObjectRef,
    date_to_locale_string: ObjectRef,
    date_to_locale_date_string: ObjectRef,
    date_to_locale_time_string: ObjectRef,
    date_value_of: ObjectRef,
    date_get_time: ObjectRef,
    date_get_full_year: ObjectRef,
    date_get_utc_full_year: ObjectRef,
    date_get_month: ObjectRef,
    date_get_utc_month: ObjectRef,
    date_get_date: ObjectRef,
    date_get_utc_date: ObjectRef,
    date_get_day: ObjectRef,
    date_get_utc_day: ObjectRef,
    date_get_hours: ObjectRef,
    date_get_utc_hours: ObjectRef,
    date_get_minutes: ObjectRef,
    date_get_utc_minutes: ObjectRef,
    date_get_seconds: ObjectRef,
    date_get_utc_seconds: ObjectRef,
    date_get_milliseconds: ObjectRef,
    date_get_utc_milliseconds: ObjectRef,
    date_get_timezone_offset: ObjectRef,
    date_set_time: ObjectRef,
    date_set_milliseconds: ObjectRef,
    date_set_utc_milliseconds: ObjectRef,
    date_set_seconds: ObjectRef,
    date_set_utc_seconds: ObjectRef,
    date_set_minutes: ObjectRef,
    date_set_utc_minutes: ObjectRef,
    date_set_hours: ObjectRef,
    date_set_utc_hours: ObjectRef,
    date_set_date: ObjectRef,
    date_set_utc_date: ObjectRef,
    date_set_month: ObjectRef,
    date_set_utc_month: ObjectRef,
    date_set_full_year: ObjectRef,
    date_set_utc_full_year: ObjectRef,
    date_to_utc_string: ObjectRef,
    date_to_iso_string: ObjectRef,
    date_to_json: ObjectRef,
    date_to_primitive: ObjectRef,
    date_to_temporal_instant: ObjectRef,
    number: ObjectRef,
    number_prototype: ObjectRef,
    number_is_finite: ObjectRef,
    number_is_integer: ObjectRef,
    number_is_nan: ObjectRef,
    number_is_safe_integer: ObjectRef,
    number_to_exponential: ObjectRef,
    number_to_fixed: ObjectRef,
    number_to_locale_string: ObjectRef,
    number_to_precision: ObjectRef,
    number_to_string: ObjectRef,
    number_value_of: ObjectRef,
    math: ObjectRef,
    math_abs: ObjectRef,
    math_acos: ObjectRef,
    math_acosh: ObjectRef,
    math_asin: ObjectRef,
    math_asinh: ObjectRef,
    math_atan: ObjectRef,
    math_atan2: ObjectRef,
    math_atanh: ObjectRef,
    math_cbrt: ObjectRef,
    math_ceil: ObjectRef,
    math_clz32: ObjectRef,
    math_cos: ObjectRef,
    math_cosh: ObjectRef,
    math_exp: ObjectRef,
    math_expm1: ObjectRef,
    math_f16round: ObjectRef,
    math_floor: ObjectRef,
    math_fround: ObjectRef,
    math_hypot: ObjectRef,
    math_imul: ObjectRef,
    math_log: ObjectRef,
    math_log10: ObjectRef,
    math_log1p: ObjectRef,
    math_log2: ObjectRef,
    math_max: ObjectRef,
    math_min: ObjectRef,
    math_pow: ObjectRef,
    math_random: ObjectRef,
    math_round: ObjectRef,
    math_sign: ObjectRef,
    math_sin: ObjectRef,
    math_sinh: ObjectRef,
    math_sqrt: ObjectRef,
    math_sum_precise: ObjectRef,
    math_tan: ObjectRef,
    math_tanh: ObjectRef,
    math_trunc: ObjectRef,
    bigint: ObjectRef,
    bigint_as_int_n: ObjectRef,
    bigint_as_uint_n: ObjectRef,
    bigint_prototype: ObjectRef,
    bigint_to_string: ObjectRef,
    bigint_value_of: ObjectRef,
    boolean: ObjectRef,
    boolean_prototype: ObjectRef,
    boolean_to_string: ObjectRef,
    boolean_value_of: ObjectRef,
    symbol: ObjectRef,
    symbol_prototype: ObjectRef,
    symbol_for: ObjectRef,
    symbol_key_for: ObjectRef,
    symbol_to_string: ObjectRef,
    symbol_value_of: ObjectRef,
    symbol_to_primitive: ObjectRef,
    array_species_getter: ObjectRef,
    symbol_description_getter: ObjectRef,
    json: ObjectRef,
    json_parse: ObjectRef,
    json_stringify: ObjectRef,
    json_raw_json: ObjectRef,
    json_is_raw_json: ObjectRef,
    reflect: ObjectRef,
    reflect_apply: ObjectRef,
    reflect_construct: ObjectRef,
    reflect_define_property: ObjectRef,
    reflect_delete_property: ObjectRef,
    reflect_get: ObjectRef,
    reflect_get_own_property_descriptor: ObjectRef,
    reflect_get_prototype_of: ObjectRef,
    reflect_has: ObjectRef,
    reflect_is_extensible: ObjectRef,
    reflect_own_keys: ObjectRef,
    reflect_prevent_extensions: ObjectRef,
    reflect_set: ObjectRef,
    reflect_set_prototype_of: ObjectRef,
    proxy: ObjectRef,
    proxy_revocable: ObjectRef,
    abstract_module_source: ObjectRef,
    abstract_module_source_prototype: ObjectRef,
    abstract_module_source_to_string_tag_getter: ObjectRef,
    error: ObjectRef,
    error_prototype: ObjectRef,
    error_to_string: ObjectRef,
    eval_error: ObjectRef,
    eval_error_prototype: ObjectRef,
    range_error: ObjectRef,
    range_error_prototype: ObjectRef,
    reference_error: ObjectRef,
    reference_error_prototype: ObjectRef,
    syntax_error: ObjectRef,
    syntax_error_prototype: ObjectRef,
    type_error: ObjectRef,
    type_error_prototype: ObjectRef,
    uri_error: ObjectRef,
    uri_error_prototype: ObjectRef,
    aggregate_error: ObjectRef,
    aggregate_error_prototype: ObjectRef,
    suppressed_error: ObjectRef,
    suppressed_error_prototype: ObjectRef,
    promise: ObjectRef,
    promise_prototype: ObjectRef,
    disposable_stack: ObjectRef,
    disposable_stack_prototype: ObjectRef,
    async_disposable_stack: ObjectRef,
    async_disposable_stack_prototype: ObjectRef,
    disposable_stack_use: ObjectRef,
    disposable_stack_adopt: ObjectRef,
    disposable_stack_defer: ObjectRef,
    disposable_stack_move: ObjectRef,
    disposable_stack_disposed_getter: ObjectRef,
    disposable_stack_dispose: ObjectRef,
    async_disposable_stack_use: ObjectRef,
    async_disposable_stack_adopt: ObjectRef,
    async_disposable_stack_defer: ObjectRef,
    async_disposable_stack_move: ObjectRef,
    async_disposable_stack_disposed_getter: ObjectRef,
    async_disposable_stack_dispose_async: ObjectRef,
    create_sync_disposal_scope: ObjectRef,
    create_async_disposal_scope: ObjectRef,
    add_sync_disposable_resource: ObjectRef,
    add_async_disposable_resource: ObjectRef,
    dispose_scope: ObjectRef,
    dispose_scope_async: ObjectRef,
    promise_then: ObjectRef,
    promise_catch: ObjectRef,
    promise_finally: ObjectRef,
    promise_resolve: ObjectRef,
    promise_reject: ObjectRef,
    promise_all: ObjectRef,
    promise_all_settled: ObjectRef,
    promise_race: ObjectRef,
    promise_any: ObjectRef,
    promise_species_getter: ObjectRef,
    eval: ObjectRef,
    parse_int: ObjectRef,
    parse_float: ObjectRef,
    is_nan: ObjectRef,
    is_finite: ObjectRef,
    encode_uri: ObjectRef,
    encode_uri_component: ObjectRef,
    decode_uri: ObjectRef,
    decode_uri_component: ObjectRef,
}

impl PublicRealmBuiltins {
    #[inline]
    pub const fn object(self) -> ObjectRef {
        self.object
    }

    #[inline]
    pub const fn object_prototype(self) -> ObjectRef {
        self.object_prototype
    }

    #[inline]
    pub const fn function(self) -> ObjectRef {
        self.function
    }

    #[inline]
    pub const fn function_prototype(self) -> ObjectRef {
        self.function_prototype
    }

    #[inline]
    pub const fn string(self) -> ObjectRef {
        self.string
    }

    #[inline]
    pub const fn string_prototype(self) -> ObjectRef {
        self.string_prototype
    }

    #[inline]
    pub const fn regexp(self) -> ObjectRef {
        self.regexp
    }

    #[inline]
    pub const fn regexp_prototype(self) -> ObjectRef {
        self.regexp_prototype
    }

    #[inline]
    pub const fn date(self) -> ObjectRef {
        self.date
    }

    #[inline]
    pub const fn date_prototype(self) -> ObjectRef {
        self.date_prototype
    }

    #[inline]
    pub const fn number(self) -> ObjectRef {
        self.number
    }

    #[inline]
    pub const fn number_prototype(self) -> ObjectRef {
        self.number_prototype
    }

    #[inline]
    pub const fn math(self) -> ObjectRef {
        self.math
    }

    #[inline]
    pub const fn bigint(self) -> ObjectRef {
        self.bigint
    }

    #[inline]
    pub const fn bigint_prototype(self) -> ObjectRef {
        self.bigint_prototype
    }

    #[inline]
    pub const fn boolean(self) -> ObjectRef {
        self.boolean
    }

    #[inline]
    pub const fn boolean_prototype(self) -> ObjectRef {
        self.boolean_prototype
    }

    #[inline]
    pub const fn symbol(self) -> ObjectRef {
        self.symbol
    }

    #[inline]
    pub const fn symbol_prototype(self) -> ObjectRef {
        self.symbol_prototype
    }

    #[inline]
    pub const fn error(self) -> ObjectRef {
        self.error
    }

    #[inline]
    pub const fn error_prototype(self) -> ObjectRef {
        self.error_prototype
    }

    #[inline]
    pub fn builtin_object(self, entry: BuiltinFunctionId) -> Option<ObjectRef> {
        if let Some(object) = families::object_builtin_object(&self, entry) {
            return Some(object);
        }
        if let Some(object) = families::function_builtin_object(&self, entry) {
            return Some(object);
        }
        if let Some(object) = families::array_builtin_object(&self, entry) {
            return Some(object);
        }
        if let Some(object) = families::collection_builtin_object(&self, entry) {
            return Some(object);
        }
        if let Some(object) = families::binary_data_builtin_object(&self, entry) {
            return Some(object);
        }
        if let Some(object) = families::iterator_builtin_object(&self, entry) {
            return Some(object);
        }
        if let Some(object) = families::string_builtin_object(&self, entry) {
            return Some(object);
        }
        if let Some(object) = families::regexp_builtin_object(&self, entry) {
            return Some(object);
        }
        if let Some(object) = families::date_builtin_object(&self, entry) {
            return Some(object);
        }
        if let Some(object) = families::primitive_builtin_object(&self, entry) {
            return Some(object);
        }
        if let Some(object) = families::json_builtin_object(&self, entry) {
            return Some(object);
        }
        if let Some(object) = families::object_reflection_builtin_object(&self, entry) {
            return Some(object);
        }
        if let Some(object) = families::module_builtin_object(&self, entry) {
            return Some(object);
        }
        if let Some(object) = families::error_builtin_object(&self, entry) {
            return Some(object);
        }
        if let Some(object) = families::promise_disposal_builtin_object(&self, entry) {
            return Some(object);
        }
        if let Some(object) = families::global_function_builtin_object(&self, entry) {
            return Some(object);
        }
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuiltinCache {
    internal: InternalBuiltinCache,
    public: HashMap<RealmRef, PublicRealmBuiltins>,
}

impl BuiltinCache {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ensure_realm_builtins(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
    ) -> Option<RealmBuiltins> {
        let internal = self.internal.ensure_realm_builtins(agent, realm)?;
        let public = self.ensure_public_realm_builtins(agent, realm, internal)?;
        Some(RealmBuiltins { internal, public })
    }

    pub fn builtin_constant(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        entry: BuiltinFunctionId,
    ) -> Option<Value> {
        let builtins = self.ensure_realm_builtins(agent, realm)?;
        builtins
            .public()
            .builtin_object(entry)
            .or_else(|| builtins.internal().builtin_object(entry))
            .map(Value::from_object_ref)
    }

    fn ensure_public_realm_builtins(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        internal: InternalRealmBuiltins,
    ) -> Option<PublicRealmBuiltins> {
        if let Some(existing) = self.public.get(&realm).copied() {
            return Some(existing);
        }

        let realm_record = agent.realm(realm)?;
        let root_shape = realm_record.root_shape()?;
        let global_env = realm_record.global_env();
        let existing_intrinsics = realm_record.intrinsics();
        let scaffolding = families::allocate_public_realm_scaffolding(
            agent,
            families::ScaffoldingRequest {
                realm,
                global_env,
                root_shape,
                internal,
                intrinsics: &existing_intrinsics,
            },
        );
        let family_context = scaffolding.cx;
        let object_family = families::install_object_family(agent, family_context);
        let function_family =
            families::install_function_family(agent, family_context, scaffolding.function);
        let iterator_family =
            families::install_iterator_family(agent, family_context, scaffolding.iterator);
        let collection_family =
            families::install_collection_family(agent, family_context, scaffolding.collection);
        let binary_data_family =
            families::install_binary_data_family(agent, family_context, scaffolding.binary_data);
        let array_family = families::install_array_family(agent, family_context, scaffolding.array);
        let string_family =
            families::install_string_family(agent, family_context, scaffolding.string);
        let regexp_family =
            families::install_regexp_family(agent, family_context, scaffolding.regexp);
        let date_family = families::install_date_family(agent, family_context, scaffolding.date);
        let primitive_family = families::install_primitive_family(
            agent,
            family_context,
            scaffolding.primitive,
            scaffolding.primitive_objects,
        );
        let json_family = families::install_json_family(agent, family_context, scaffolding.json);
        let object_reflection_family = families::install_object_reflection_family(
            agent,
            family_context,
            scaffolding.object_reflection,
        );
        let module_family =
            families::install_module_family(agent, family_context, scaffolding.module);
        let error_family = families::install_error_family(agent, family_context, scaffolding.error);
        let promise_disposal_family = families::install_promise_disposal_family(
            agent,
            family_context,
            scaffolding.promise_disposal,
        );
        let global_function_family =
            families::install_global_function_family(agent, family_context);

        let builtins = families::InstalledBuiltinFamilies {
            object: object_family,
            function: function_family,
            iterator: iterator_family,
            collection: collection_family,
            binary_data: binary_data_family,
            array: array_family,
            string: string_family,
            regexp: regexp_family,
            date: date_family,
            primitive: primitive_family,
            json: json_family,
            object_reflection: object_reflection_family,
            module: module_family,
            error: error_family,
            promise_disposal: promise_disposal_family,
            global_function: global_function_family,
        }
        .public_realm_builtins();
        families::link_installed_family_prototypes(agent, &builtins);
        if !families::install_public_realm_intrinsics(
            agent,
            realm,
            &existing_intrinsics,
            &builtins,
            &scaffolding.intrinsics,
        ) {
            return None;
        }
        reparent_builtin_object(
            agent,
            realm_record.global_object(),
            Some(builtins.object_prototype),
        );
        if temporal::install_temporal_public_objects(
            agent,
            realm,
            global_env,
            root_shape,
            builtins.function_prototype,
            builtins.object_prototype,
            realm_record.global_object(),
        )
        .is_none()
        {
            return None;
        }
        families::install_module_family_descriptors(agent, &builtins);

        let bootstrap_atoms = agent.bootstrap_atoms();
        let empty_string = Value::from_string_ref(agent.alloc_runtime_string(
            "",
            Some(WellKnownAtom::Empty.id()),
            AllocationLifetime::Default,
        ));
        let bigint_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "BigInt",
            Some(bootstrap_atoms.bigint()),
            AllocationLifetime::Default,
        ));
        let error_name = Value::from_string_ref(agent.alloc_runtime_string(
            "Error",
            Some(bootstrap_atoms.error()),
            AllocationLifetime::Default,
        ));
        let eval_error_name = Value::from_string_ref(agent.alloc_runtime_string(
            "EvalError",
            Some(bootstrap_atoms.eval_error()),
            AllocationLifetime::Default,
        ));
        let range_error_name = Value::from_string_ref(agent.alloc_runtime_string(
            "RangeError",
            Some(bootstrap_atoms.range_error()),
            AllocationLifetime::Default,
        ));
        let reference_error_name = Value::from_string_ref(agent.alloc_runtime_string(
            "ReferenceError",
            Some(bootstrap_atoms.reference_error()),
            AllocationLifetime::Default,
        ));
        let syntax_error_name = Value::from_string_ref(agent.alloc_runtime_string(
            "SyntaxError",
            Some(bootstrap_atoms.syntax_error()),
            AllocationLifetime::Default,
        ));
        let math_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Math",
            Some(bootstrap_atoms.math()),
            AllocationLifetime::Default,
        ));
        let array_buffer_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "ArrayBuffer",
            Some(bootstrap_atoms.array_buffer()),
            AllocationLifetime::Default,
        ));
        let shared_array_buffer_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "SharedArrayBuffer",
            Some(bootstrap_atoms.shared_array_buffer()),
            AllocationLifetime::Default,
        ));
        let data_view_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "DataView",
            Some(bootstrap_atoms.data_view()),
            AllocationLifetime::Default,
        ));
        let atomics_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Atomics",
            Some(bootstrap_atoms.atomics()),
            AllocationLifetime::Default,
        ));
        let symbol_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Symbol",
            Some(bootstrap_atoms.symbol()),
            AllocationLifetime::Default,
        ));
        let int8_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Int8Array",
            Some(bootstrap_atoms.int8_array()),
            AllocationLifetime::Default,
        ));
        let int16_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Int16Array",
            Some(bootstrap_atoms.int16_array()),
            AllocationLifetime::Default,
        ));
        let int32_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Int32Array",
            Some(bootstrap_atoms.int32_array()),
            AllocationLifetime::Default,
        ));
        let float32_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Float32Array",
            Some(bootstrap_atoms.float32_array()),
            AllocationLifetime::Default,
        ));
        let float64_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Float64Array",
            Some(bootstrap_atoms.float64_array()),
            AllocationLifetime::Default,
        ));
        let big_int64_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "BigInt64Array",
            Some(bootstrap_atoms.big_int64_array()),
            AllocationLifetime::Default,
        ));
        let big_uint64_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "BigUint64Array",
            Some(bootstrap_atoms.big_uint64_array()),
            AllocationLifetime::Default,
        ));
        let uint32_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Uint32Array",
            Some(bootstrap_atoms.uint32_array()),
            AllocationLifetime::Default,
        ));
        let uint16_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Uint16Array",
            Some(bootstrap_atoms.uint16_array()),
            AllocationLifetime::Default,
        ));
        let uint8_clamped_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Uint8ClampedArray",
            Some(bootstrap_atoms.uint8_clamped_array()),
            AllocationLifetime::Default,
        ));
        let uint8_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Uint8Array",
            Some(bootstrap_atoms.uint8_array()),
            AllocationLifetime::Default,
        ));
        let string_iterator_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "String Iterator",
            None,
            AllocationLifetime::Default,
        ));
        let type_error_name = Value::from_string_ref(agent.alloc_runtime_string(
            "TypeError",
            Some(bootstrap_atoms.type_error()),
            AllocationLifetime::Default,
        ));
        let uri_error_name = Value::from_string_ref(agent.alloc_runtime_string(
            "URIError",
            Some(bootstrap_atoms.uri_error()),
            AllocationLifetime::Default,
        ));
        let aggregate_error_name = Value::from_string_ref(agent.alloc_runtime_string(
            "AggregateError",
            Some(bootstrap_atoms.aggregate_error()),
            AllocationLifetime::Default,
        ));
        let suppressed_error_name_atom = agent.atoms_mut().intern_collectible("SuppressedError");
        let suppressed_error_name = Value::from_string_ref(agent.alloc_runtime_string(
            "SuppressedError",
            Some(suppressed_error_name_atom),
            AllocationLifetime::Default,
        ));
        let disposable_stack_tag_atom = agent.atoms_mut().intern_collectible("DisposableStack");
        let disposable_stack_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "DisposableStack",
            Some(disposable_stack_tag_atom),
            AllocationLifetime::Default,
        ));
        let async_disposable_stack_tag_atom =
            agent.atoms_mut().intern_collectible("AsyncDisposableStack");
        let async_disposable_stack_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "AsyncDisposableStack",
            Some(async_disposable_stack_tag_atom),
            AllocationLifetime::Default,
        ));
        let abs_atom = agent.atoms_mut().intern_collectible("abs");
        let acos_atom = agent.atoms_mut().intern_collectible("acos");
        let acosh_atom = agent.atoms_mut().intern_collectible("acosh");
        let asin_atom = agent.atoms_mut().intern_collectible("asin");
        let asinh_atom = agent.atoms_mut().intern_collectible("asinh");
        let atan_atom = agent.atoms_mut().intern_collectible("atan");
        let atan2_atom = agent.atoms_mut().intern_collectible("atan2");
        let atanh_atom = agent.atoms_mut().intern_collectible("atanh");
        let cbrt_atom = agent.atoms_mut().intern_collectible("cbrt");
        let ceil_atom = agent.atoms_mut().intern_collectible("ceil");
        let clz32_atom = agent.atoms_mut().intern_collectible("clz32");
        let cos_atom = agent.atoms_mut().intern_collectible("cos");
        let cosh_atom = agent.atoms_mut().intern_collectible("cosh");
        let escape_atom = agent.atoms_mut().intern_collectible("escape");
        let epsilon_atom = agent.atoms_mut().intern_collectible("EPSILON");
        let e_atom = agent.atoms_mut().intern_collectible("E");
        let exp_atom = agent.atoms_mut().intern_collectible("exp");
        let expm1_atom = agent.atoms_mut().intern_collectible("expm1");
        let f16round_atom = agent.atoms_mut().intern_collectible("f16round");
        let floor_atom = agent.atoms_mut().intern_collectible("floor");
        let fround_atom = agent.atoms_mut().intern_collectible("fround");
        let global_atom = agent.atoms_mut().intern_collectible("global");
        let get_date_atom = agent.atoms_mut().intern_collectible("getDate");
        let get_day_atom = agent.atoms_mut().intern_collectible("getDay");
        let get_full_year_atom = agent.atoms_mut().intern_collectible("getFullYear");
        let get_hours_atom = agent.atoms_mut().intern_collectible("getHours");
        let get_milliseconds_atom = agent.atoms_mut().intern_collectible("getMilliseconds");
        let get_minutes_atom = agent.atoms_mut().intern_collectible("getMinutes");
        let get_month_atom = agent.atoms_mut().intern_collectible("getMonth");
        let get_seconds_atom = agent.atoms_mut().intern_collectible("getSeconds");
        let get_time_atom = agent.atoms_mut().intern_collectible("getTime");
        let get_timezone_offset_atom = agent.atoms_mut().intern_collectible("getTimezoneOffset");
        let get_utc_date_atom = agent.atoms_mut().intern_collectible("getUTCDate");
        let get_utc_day_atom = agent.atoms_mut().intern_collectible("getUTCDay");
        let get_utc_full_year_atom = agent.atoms_mut().intern_collectible("getUTCFullYear");
        let get_utc_hours_atom = agent.atoms_mut().intern_collectible("getUTCHours");
        let get_utc_milliseconds_atom = agent.atoms_mut().intern_collectible("getUTCMilliseconds");
        let get_utc_minutes_atom = agent.atoms_mut().intern_collectible("getUTCMinutes");
        let get_utc_month_atom = agent.atoms_mut().intern_collectible("getUTCMonth");
        let get_utc_seconds_atom = agent.atoms_mut().intern_collectible("getUTCSeconds");
        let hypot_atom = agent.atoms_mut().intern_collectible("hypot");
        let imul_atom = agent.atoms_mut().intern_collectible("imul");
        let is_finite_atom = agent.atoms_mut().intern_collectible("isFinite");
        let is_integer_atom = agent.atoms_mut().intern_collectible("isInteger");
        let is_nan_atom = agent.atoms_mut().intern_collectible("isNaN");
        let is_safe_integer_atom = agent.atoms_mut().intern_collectible("isSafeInteger");
        let ignore_case_atom = agent.atoms_mut().intern_collectible("ignoreCase");
        let last_index_of_atom = agent.atoms_mut().intern_collectible("lastIndexOf");
        let match_atom = agent.atoms_mut().intern_collectible("match");
        let match_all_atom = agent.atoms_mut().intern_collectible("matchAll");
        let multiline_atom = agent.atoms_mut().intern_collectible("multiline");
        let log_atom = agent.atoms_mut().intern_collectible("log");
        let log10_atom = agent.atoms_mut().intern_collectible("log10");
        let log1p_atom = agent.atoms_mut().intern_collectible("log1p");
        let log2_atom = agent.atoms_mut().intern_collectible("log2");
        let ln10_atom = agent.atoms_mut().intern_collectible("LN10");
        let ln2_atom = agent.atoms_mut().intern_collectible("LN2");
        let log10e_atom = agent.atoms_mut().intern_collectible("LOG10E");
        let log2e_atom = agent.atoms_mut().intern_collectible("LOG2E");
        let max_atom = agent.atoms_mut().intern_collectible("max");
        let max_safe_integer_atom = agent.atoms_mut().intern_collectible("MAX_SAFE_INTEGER");
        let max_value_atom = agent.atoms_mut().intern_collectible("MAX_VALUE");
        let min_atom = agent.atoms_mut().intern_collectible("min");
        let min_safe_integer_atom = agent.atoms_mut().intern_collectible("MIN_SAFE_INTEGER");
        let min_value_atom = agent.atoms_mut().intern_collectible("MIN_VALUE");
        let negative_infinity_atom = agent.atoms_mut().intern_collectible("NEGATIVE_INFINITY");
        let pad_end_atom = agent.atoms_mut().intern_collectible("padEnd");
        let pad_start_atom = agent.atoms_mut().intern_collectible("padStart");
        let parse_atom = agent.atoms_mut().intern_collectible("parse");
        let parse_float_atom = agent.atoms_mut().intern_collectible("parseFloat");
        let parse_int_atom = agent.atoms_mut().intern_collectible("parseInt");
        let pi_atom = agent.atoms_mut().intern_collectible("PI");
        let positive_infinity_atom = agent.atoms_mut().intern_collectible("POSITIVE_INFINITY");
        let pow_atom = agent.atoms_mut().intern_collectible("pow");
        let random_atom = agent.atoms_mut().intern_collectible("random");
        let repeat_atom = agent.atoms_mut().intern_collectible("repeat");
        let replace_atom = agent.atoms_mut().intern_collectible("replace");
        let round_atom = agent.atoms_mut().intern_collectible("round");
        let sign_atom = agent.atoms_mut().intern_collectible("sign");
        let sin_atom = agent.atoms_mut().intern_collectible("sin");
        let sinh_atom = agent.atoms_mut().intern_collectible("sinh");
        let set_date_atom = agent.atoms_mut().intern_collectible("setDate");
        let set_full_year_atom = agent.atoms_mut().intern_collectible("setFullYear");
        let set_hours_atom = agent.atoms_mut().intern_collectible("setHours");
        let set_milliseconds_atom = agent.atoms_mut().intern_collectible("setMilliseconds");
        let set_minutes_atom = agent.atoms_mut().intern_collectible("setMinutes");
        let set_month_atom = agent.atoms_mut().intern_collectible("setMonth");
        let set_seconds_atom = agent.atoms_mut().intern_collectible("setSeconds");
        let set_time_atom = agent.atoms_mut().intern_collectible("setTime");
        let set_utc_date_atom = agent.atoms_mut().intern_collectible("setUTCDate");
        let set_utc_full_year_atom = agent.atoms_mut().intern_collectible("setUTCFullYear");
        let set_utc_hours_atom = agent.atoms_mut().intern_collectible("setUTCHours");
        let set_utc_milliseconds_atom = agent.atoms_mut().intern_collectible("setUTCMilliseconds");
        let set_utc_minutes_atom = agent.atoms_mut().intern_collectible("setUTCMinutes");
        let set_utc_month_atom = agent.atoms_mut().intern_collectible("setUTCMonth");
        let set_utc_seconds_atom = agent.atoms_mut().intern_collectible("setUTCSeconds");
        let split_atom = agent.atoms_mut().intern_collectible("split");
        let starts_with_atom = agent.atoms_mut().intern_collectible("startsWith");
        let sticky_atom = agent.atoms_mut().intern_collectible("sticky");
        let sum_precise_atom = agent.atoms_mut().intern_collectible("sumPrecise");
        let tan_atom = agent.atoms_mut().intern_collectible("tan");
        let tanh_atom = agent.atoms_mut().intern_collectible("tanh");
        let to_date_string_atom = agent.atoms_mut().intern_collectible("toDateString");
        let to_iso_string_atom = agent.atoms_mut().intern_collectible("toISOString");
        let to_json_atom = agent.atoms_mut().intern_collectible("toJSON");
        let to_locale_date_string_atom = agent.atoms_mut().intern_collectible("toLocaleDateString");
        let to_locale_time_string_atom = agent.atoms_mut().intern_collectible("toLocaleTimeString");
        let to_temporal_instant_atom = agent.atoms_mut().intern_collectible("toTemporalInstant");
        let to_time_string_atom = agent.atoms_mut().intern_collectible("toTimeString");
        let to_utc_string_atom = agent.atoms_mut().intern_collectible("toUTCString");
        let substring_atom = agent.atoms_mut().intern_collectible("substring");
        let utc_atom = agent.atoms_mut().intern_collectible("UTC");
        let concat_atom = agent.atoms_mut().intern_collectible("concat");
        let copy_within_atom = agent.atoms_mut().intern_collectible("copyWithin");
        let entries_atom = agent.atoms_mut().intern_collectible("entries");
        let every_atom = agent.atoms_mut().intern_collectible("every");
        let fill_atom = agent.atoms_mut().intern_collectible("fill");
        let filter_atom = agent.atoms_mut().intern_collectible("filter");
        let find_atom = agent.atoms_mut().intern_collectible("find");
        let find_index_atom = agent.atoms_mut().intern_collectible("findIndex");
        let find_last_atom = agent.atoms_mut().intern_collectible("findLast");
        let find_last_index_atom = agent.atoms_mut().intern_collectible("findLastIndex");
        let from_atom = agent.atoms_mut().intern_collectible("from");
        let for_each_atom = agent.atoms_mut().intern_collectible("forEach");
        let includes_atom = agent.atoms_mut().intern_collectible("includes");
        let index_of_atom = agent.atoms_mut().intern_collectible("indexOf");
        let from_code_point_atom = agent.atoms_mut().intern_collectible("fromCodePoint");
        let raw_atom = agent.atoms_mut().intern_collectible("raw");
        let code_point_at_atom = agent.atoms_mut().intern_collectible("codePointAt");
        let ends_with_atom = agent.atoms_mut().intern_collectible("endsWith");
        let is_well_formed_atom = agent.atoms_mut().intern_collectible("isWellFormed");
        let locale_compare_atom = agent.atoms_mut().intern_collectible("localeCompare");
        let normalize_atom = agent.atoms_mut().intern_collectible("normalize");
        let replace_all_atom = agent.atoms_mut().intern_collectible("replaceAll");
        let to_locale_lower_case_atom = agent.atoms_mut().intern_collectible("toLocaleLowerCase");
        let to_locale_upper_case_atom = agent.atoms_mut().intern_collectible("toLocaleUpperCase");
        let to_lower_case_atom = agent.atoms_mut().intern_collectible("toLowerCase");
        let to_upper_case_atom = agent.atoms_mut().intern_collectible("toUpperCase");
        let to_well_formed_atom = agent.atoms_mut().intern_collectible("toWellFormed");
        let trim_atom = agent.atoms_mut().intern_collectible("trim");
        let trim_end_atom = agent.atoms_mut().intern_collectible("trimEnd");
        let trim_start_atom = agent.atoms_mut().intern_collectible("trimStart");
        let join_atom = agent.atoms_mut().intern_collectible("join");
        let keys_atom = agent.atoms_mut().intern_collectible("keys");
        let map_atom = agent.atoms_mut().intern_collectible("map");
        let next_atom = agent.atoms_mut().intern_collectible("next");
        let of_atom = agent.atoms_mut().intern_collectible("of");
        let reduce_atom = agent.atoms_mut().intern_collectible("reduce");
        let reduce_right_atom = agent.atoms_mut().intern_collectible("reduceRight");
        let reverse_atom = agent.atoms_mut().intern_collectible("reverse");
        let some_atom = agent.atoms_mut().intern_collectible("some");
        let at_atom = agent.atoms_mut().intern_collectible("at");
        let slice_atom = agent.atoms_mut().intern_collectible("slice");
        let buffer_atom = agent.atoms_mut().intern_collectible("buffer");
        let byte_length_atom = agent.atoms_mut().intern_collectible("byteLength");
        let byte_offset_atom = agent.atoms_mut().intern_collectible("byteOffset");
        let bytes_per_element_atom = agent.atoms_mut().intern_collectible("BYTES_PER_ELEMENT");
        let is_view_atom = agent.atoms_mut().intern_collectible("isView");
        let sort_atom = agent.atoms_mut().intern_collectible("sort");
        let search_atom = agent.atoms_mut().intern_collectible("search");
        let to_locale_string_atom = agent.atoms_mut().intern_collectible("toLocaleString");
        let to_reversed_atom = agent.atoms_mut().intern_collectible("toReversed");
        let to_sorted_atom = agent.atoms_mut().intern_collectible("toSorted");
        let values_atom = agent.atoms_mut().intern_collectible("values");
        let with_atom = agent.atoms_mut().intern_collectible("with");
        let sqrt1_2_atom = agent.atoms_mut().intern_collectible("SQRT1_2");
        let sqrt2_atom = agent.atoms_mut().intern_collectible("SQRT2");
        let sqrt_atom = agent.atoms_mut().intern_collectible("sqrt");
        let char_at_atom = agent.atoms_mut().intern_collectible("charAt");
        let char_code_at_atom = agent.atoms_mut().intern_collectible("charCodeAt");
        let from_char_code_atom = agent.atoms_mut().intern_collectible("fromCharCode");
        let dot_all_atom = agent.atoms_mut().intern_collectible("dotAll");
        let exec_atom = agent.atoms_mut().intern_collectible("exec");
        let get_float32_atom = agent.atoms_mut().intern_collectible("getFloat32");
        let get_float64_atom = agent.atoms_mut().intern_collectible("getFloat64");
        let get_int16_atom = agent.atoms_mut().intern_collectible("getInt16");
        let get_int32_atom = agent.atoms_mut().intern_collectible("getInt32");
        let get_int8_atom = agent.atoms_mut().intern_collectible("getInt8");
        let get_uint16_atom = agent.atoms_mut().intern_collectible("getUint16");
        let get_uint32_atom = agent.atoms_mut().intern_collectible("getUint32");
        let get_uint8_atom = agent.atoms_mut().intern_collectible("getUint8");
        let has_indices_atom = bootstrap_atoms.has_indices();
        let add_atom = agent.atoms_mut().intern_collectible("add");
        let and_atom = agent.atoms_mut().intern_collectible("and");
        let as_int_n_atom = agent.atoms_mut().intern_collectible("asIntN");
        let as_uint_n_atom = agent.atoms_mut().intern_collectible("asUintN");
        let compare_exchange_atom = agent.atoms_mut().intern_collectible("compareExchange");
        let exchange_atom = agent.atoms_mut().intern_collectible("exchange");
        let is_lock_free_atom = agent.atoms_mut().intern_collectible("isLockFree");
        let load_atom = agent.atoms_mut().intern_collectible("load");
        let notify_atom = agent.atoms_mut().intern_collectible("notify");
        let now_atom = agent.atoms_mut().intern_collectible("now");
        let or_atom = agent.atoms_mut().intern_collectible("or");
        let set_atom = agent.atoms_mut().intern_collectible("set");
        let set_float32_atom = agent.atoms_mut().intern_collectible("setFloat32");
        let set_float64_atom = agent.atoms_mut().intern_collectible("setFloat64");
        let set_int16_atom = agent.atoms_mut().intern_collectible("setInt16");
        let set_int32_atom = agent.atoms_mut().intern_collectible("setInt32");
        let set_int8_atom = agent.atoms_mut().intern_collectible("setInt8");
        let set_uint16_atom = agent.atoms_mut().intern_collectible("setUint16");
        let set_uint32_atom = agent.atoms_mut().intern_collectible("setUint32");
        let set_uint8_atom = agent.atoms_mut().intern_collectible("setUint8");
        let store_atom = agent.atoms_mut().intern_collectible("store");
        let sub_atom = agent.atoms_mut().intern_collectible("sub");
        let subarray_atom = agent.atoms_mut().intern_collectible("subarray");
        let test_atom = agent.atoms_mut().intern_collectible("test");
        let trunc_atom = agent.atoms_mut().intern_collectible("trunc");
        let to_exponential_atom = agent.atoms_mut().intern_collectible("toExponential");
        let to_fixed_atom = agent.atoms_mut().intern_collectible("toFixed");
        let to_precision_atom = agent.atoms_mut().intern_collectible("toPrecision");
        let unicode_atom = agent.atoms_mut().intern_collectible("unicode");
        let wait_atom = agent.atoms_mut().intern_collectible("wait");
        let wait_async_atom = agent.atoms_mut().intern_collectible("waitAsync");
        let xor_atom = agent.atoms_mut().intern_collectible("xor");
        let array_buffer_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(is_view_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_buffer_is_view_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Species),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_array_species_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let shared_array_buffer_descriptors = [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Species),
            BuiltinPropertyValueSpec::Accessor {
                get: Some(js3_array_species_getter_builtin()),
                set: None,
            },
            BuiltinAttributes::new(false, false, true),
        )];
        let array_buffer_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.array_buffer)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_length_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_array_buffer_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(slice_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_buffer_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(array_buffer_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let shared_array_buffer_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(
                    builtins.shared_array_buffer,
                )),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_length_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_shared_array_buffer_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(slice_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_shared_array_buffer_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(shared_array_buffer_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let atomics_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(add_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_atomics_add_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(and_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_atomics_and_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(compare_exchange_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_atomics_compare_exchange_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(exchange_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_atomics_exchange_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(is_lock_free_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_atomics_is_lock_free_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(load_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_atomics_load_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(notify_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_atomics_notify_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(or_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_atomics_or_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(store_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_atomics_store_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(sub_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_atomics_sub_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(wait_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_atomics_wait_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(wait_async_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_atomics_wait_async_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(xor_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_atomics_xor_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(atomics_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let data_view_descriptors: [BuiltinPropertyDescriptor; 0] = [];
        let data_view_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.data_view)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(buffer_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_data_view_buffer_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_length_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_data_view_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_offset_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_data_view_byte_offset_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_float32_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_data_view_get_float32_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_float64_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_data_view_get_float64_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_int16_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_data_view_get_int16_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_int32_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_data_view_get_int32_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_int8_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_data_view_get_int8_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_uint16_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_data_view_get_uint16_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_uint32_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_data_view_get_uint32_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_uint8_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_data_view_get_uint8_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_float32_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_data_view_set_float32_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_float64_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_data_view_set_float64_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_int16_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_data_view_set_int16_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_int32_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_data_view_set_int32_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_int8_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_data_view_set_int8_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_uint16_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_data_view_set_uint16_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_uint32_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_data_view_set_uint32_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_uint8_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_data_view_set_uint8_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(data_view_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let typed_array_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(from_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_from_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(of_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Species),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_array_species_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let typed_array_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.typed_array)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(buffer_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_buffer_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_length_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_offset_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_offset_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::length.id()),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(copy_within_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_copy_within_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(every_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_every_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(fill_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_fill_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(filter_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_filter_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(includes_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_includes_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(index_of_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_index_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(for_each_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_for_each_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(join_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_join_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(map_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_map_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(some_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_some_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(find_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_find_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(find_index_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_find_index_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(find_last_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_find_last_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(find_last_index_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_find_last_index_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(values_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(keys_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_keys_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(entries_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_entries_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_set_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(slice_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(last_index_of_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_last_index_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(reduce_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_reduce_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(reduce_right_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_reduce_right_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(subarray_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_subarray_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(reverse_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_reverse_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(sort_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_sort_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_reversed_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_to_reversed_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_sorted_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_to_sorted_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(with_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_with_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(at_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_at_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::toString.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_to_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_locale_string_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    js3_typed_array_to_locale_string_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_typed_array_to_string_tag_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let int8_array_descriptors = [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(1)),
            BuiltinAttributes::new(false, false, false),
        )];
        let int8_array_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.int8_array)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
                BuiltinPropertyValueSpec::Data(Value::from_smi(1)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(buffer_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_buffer_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_length_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_offset_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_offset_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::length.id()),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(values_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(keys_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_keys_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(entries_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_entries_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_set_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(slice_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(subarray_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_subarray_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(int8_array_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let int16_array_descriptors = [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(2)),
            BuiltinAttributes::new(false, false, false),
        )];
        let int16_array_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.int16_array)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
                BuiltinPropertyValueSpec::Data(Value::from_smi(2)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(buffer_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_buffer_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_length_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_offset_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_offset_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::length.id()),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(values_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(keys_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_keys_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(entries_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_entries_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_set_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(slice_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(subarray_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_subarray_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(int16_array_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let int32_array_descriptors = [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(4)),
            BuiltinAttributes::new(false, false, false),
        )];
        let int32_array_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.int32_array)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
                BuiltinPropertyValueSpec::Data(Value::from_smi(4)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(buffer_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_buffer_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_length_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_offset_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_offset_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::length.id()),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(values_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(keys_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_keys_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(entries_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_entries_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_set_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(slice_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(subarray_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_subarray_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(int32_array_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let float32_array_descriptors = [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(4)),
            BuiltinAttributes::new(false, false, false),
        )];
        let float32_array_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.float32_array)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
                BuiltinPropertyValueSpec::Data(Value::from_smi(4)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(buffer_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_buffer_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_length_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_offset_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_offset_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::length.id()),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(values_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(keys_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_keys_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(entries_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_entries_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_set_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(slice_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(subarray_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_subarray_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(float32_array_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let float64_array_descriptors = [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(8)),
            BuiltinAttributes::new(false, false, false),
        )];
        let float64_array_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.float64_array)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
                BuiltinPropertyValueSpec::Data(Value::from_smi(8)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(buffer_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_buffer_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_length_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_offset_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_offset_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::length.id()),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(values_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(keys_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_keys_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(entries_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_entries_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_set_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(slice_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(subarray_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_subarray_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(float64_array_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let big_int64_array_descriptors = [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(8)),
            BuiltinAttributes::new(false, false, false),
        )];
        let big_int64_array_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.big_int64_array)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
                BuiltinPropertyValueSpec::Data(Value::from_smi(8)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(buffer_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_buffer_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_length_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_offset_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_offset_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::length.id()),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(values_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(keys_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_keys_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(entries_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_entries_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_set_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(slice_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(subarray_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_subarray_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(big_int64_array_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let big_uint64_array_descriptors = [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(8)),
            BuiltinAttributes::new(false, false, false),
        )];
        let big_uint64_array_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.big_uint64_array)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
                BuiltinPropertyValueSpec::Data(Value::from_smi(8)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(buffer_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_buffer_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_length_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_offset_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_offset_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::length.id()),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(values_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(keys_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_keys_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(entries_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_entries_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_set_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(slice_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(subarray_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_subarray_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(big_uint64_array_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let uint32_array_descriptors = [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(4)),
            BuiltinAttributes::new(false, false, false),
        )];
        let uint32_array_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.uint32_array)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
                BuiltinPropertyValueSpec::Data(Value::from_smi(4)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(buffer_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_buffer_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_length_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_offset_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_offset_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::length.id()),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(values_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(keys_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_keys_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(entries_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_entries_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_set_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(slice_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(subarray_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_subarray_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(uint32_array_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let uint16_array_descriptors = [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(2)),
            BuiltinAttributes::new(false, false, false),
        )];
        let uint16_array_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.uint16_array)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
                BuiltinPropertyValueSpec::Data(Value::from_smi(2)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(buffer_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_buffer_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_length_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_offset_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_offset_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::length.id()),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(values_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(keys_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_keys_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(entries_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_entries_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_set_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(slice_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(subarray_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_subarray_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(uint16_array_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let uint8_clamped_array_descriptors = [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(1)),
            BuiltinAttributes::new(false, false, false),
        )];
        let uint8_clamped_array_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(
                    builtins.uint8_clamped_array,
                )),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
                BuiltinPropertyValueSpec::Data(Value::from_smi(1)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(buffer_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_buffer_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_length_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_offset_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_offset_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::length.id()),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(values_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(keys_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_keys_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(entries_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_entries_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_set_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(slice_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(subarray_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_subarray_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(uint8_clamped_array_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let uint8_array_descriptors = [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(1)),
            BuiltinAttributes::new(false, false, false),
        )];
        let uint8_array_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.uint8_array)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
                BuiltinPropertyValueSpec::Data(Value::from_smi(1)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(buffer_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_buffer_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_length_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_offset_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_byte_offset_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::length.id()),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_uint8_array_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(values_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(keys_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_keys_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(entries_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_entries_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_set_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(slice_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(subarray_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_subarray_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(uint8_array_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let string_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(from_char_code_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_from_char_code_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(from_code_point_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_from_code_point_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(raw_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_raw_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
        ];
        let string_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.string)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::toString.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_to_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::valueOf.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_value_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(concat_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_concat_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(char_at_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_char_at_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(char_code_at_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_char_code_at_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(at_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_at_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(code_point_at_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_code_point_at_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(ends_with_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_ends_with_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(includes_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_includes_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(index_of_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_index_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(is_well_formed_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_is_well_formed_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(locale_compare_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_locale_compare_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(match_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_match_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(match_all_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_match_all_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(normalize_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_normalize_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(last_index_of_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_last_index_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(pad_end_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_pad_end_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(pad_start_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_pad_start_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(repeat_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_repeat_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(replace_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_replace_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(replace_all_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_replace_all_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(search_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_search_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(split_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_split_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(slice_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(substring_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_substring_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(starts_with_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_starts_with_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_locale_lower_case_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_to_locale_lower_case_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_locale_upper_case_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_to_locale_upper_case_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_lower_case_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_to_lower_case_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_upper_case_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_to_upper_case_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_well_formed_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_to_well_formed_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(trim_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_trim_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(trim_end_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_trim_end_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(trim_start_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_trim_start_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_iterator_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
        ];
        let string_iterator_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(next_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_iterator_next_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(string_iterator_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let regexp_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(escape_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_regexp_escape_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Species),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_regexp_species_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let regexp_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.regexp)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(exec_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_regexp_exec_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(test_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_regexp_test_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::toString.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_regexp_to_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Match),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_regexp_symbol_match_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Replace),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_regexp_symbol_replace_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Search),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_regexp_symbol_search_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Split),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_regexp_symbol_split_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::MatchAll),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_regexp_symbol_match_all_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bootstrap_atoms.source()),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_regexp_source_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bootstrap_atoms.flags()),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_regexp_flags_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(has_indices_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_regexp_has_indices_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(global_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_regexp_global_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(ignore_case_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_regexp_ignore_case_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(multiline_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_regexp_multiline_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(dot_all_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_regexp_dot_all_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(unicode_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_regexp_unicode_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(sticky_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_regexp_sticky_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let date_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(now_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_now_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(parse_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_parse_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(utc_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_utc_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
        ];
        let date_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.date)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::toString.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_to_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_date_string_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_to_date_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_time_string_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_to_time_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_locale_string_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_to_locale_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_locale_date_string_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_to_locale_date_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_locale_time_string_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_to_locale_time_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::valueOf.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_value_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_time_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_time_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_full_year_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_full_year_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_utc_full_year_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_utc_full_year_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_month_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_month_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_utc_month_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_utc_month_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_date_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_date_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_utc_date_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_utc_date_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_day_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_day_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_utc_day_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_utc_day_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_hours_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_hours_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_utc_hours_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_utc_hours_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_minutes_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_minutes_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_utc_minutes_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_utc_minutes_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_seconds_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_seconds_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_utc_seconds_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_utc_seconds_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_milliseconds_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_milliseconds_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_utc_milliseconds_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_utc_milliseconds_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_timezone_offset_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_timezone_offset_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_time_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_set_time_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_milliseconds_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_set_milliseconds_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_utc_milliseconds_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_set_utc_milliseconds_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_seconds_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_set_seconds_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_utc_seconds_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_set_utc_seconds_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_minutes_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_set_minutes_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_utc_minutes_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_set_utc_minutes_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_hours_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_set_hours_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_utc_hours_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_set_utc_hours_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_date_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_set_date_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_utc_date_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_set_utc_date_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_month_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_set_month_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_utc_month_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_set_utc_month_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_full_year_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_set_full_year_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_utc_full_year_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_set_utc_full_year_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_utc_string_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_to_utc_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_iso_string_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_to_iso_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_json_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_to_json_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_temporal_instant_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_to_temporal_instant_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToPrimitive),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_to_primitive_builtin()),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let number_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(is_finite_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_number_is_finite_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(is_integer_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_number_is_integer_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(is_nan_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_number_is_nan_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(is_safe_integer_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_number_is_safe_integer_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(parse_float_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_parse_float_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(parse_int_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_parse_int_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bootstrap_atoms.nan()),
                BuiltinPropertyValueSpec::Data(Value::from_f64(f64::NAN)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(positive_infinity_atom),
                BuiltinPropertyValueSpec::Data(Value::from_f64(f64::INFINITY)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(negative_infinity_atom),
                BuiltinPropertyValueSpec::Data(Value::from_f64(f64::NEG_INFINITY)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(max_value_atom),
                BuiltinPropertyValueSpec::Data(Value::from_f64(f64::MAX)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(min_value_atom),
                BuiltinPropertyValueSpec::Data(Value::from_f64(f64::MIN_POSITIVE)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(max_safe_integer_atom),
                BuiltinPropertyValueSpec::Data(Value::from_f64(9_007_199_254_740_991.0)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(min_safe_integer_atom),
                BuiltinPropertyValueSpec::Data(Value::from_f64(-9_007_199_254_740_991.0)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(epsilon_atom),
                BuiltinPropertyValueSpec::Data(Value::from_f64(f64::EPSILON)),
                BuiltinAttributes::new(false, false, false),
            ),
        ];
        let number_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.number)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_exponential_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_number_to_exponential_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_fixed_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_number_to_fixed_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_locale_string_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_number_to_locale_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_precision_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_number_to_precision_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::toString.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_number_to_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::valueOf.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_number_value_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
        ];
        let math_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(abs_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_abs_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(acos_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_acos_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(acosh_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_acosh_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(asin_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_asin_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(asinh_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_asinh_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atan_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_atan_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atan2_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_atan2_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atanh_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_atanh_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(cbrt_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_cbrt_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(ceil_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_ceil_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(clz32_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_clz32_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(cos_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_cos_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(cosh_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_cosh_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(exp_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_exp_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(expm1_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_expm1_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(f16round_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_f16round_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(floor_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_floor_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(fround_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_fround_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(hypot_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_hypot_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(imul_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_imul_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(log_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_log_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(log10_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_log10_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(log1p_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_log1p_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(log2_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_log2_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(max_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_max_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(min_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_min_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(pow_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_pow_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(random_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_random_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(round_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_round_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(sign_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_sign_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(sin_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_sin_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(sinh_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_sinh_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(sqrt_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_sqrt_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(sum_precise_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_sum_precise_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(tan_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_tan_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(tanh_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_tanh_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(trunc_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_trunc_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(e_atom),
                BuiltinPropertyValueSpec::Data(Value::from_f64(std::f64::consts::E)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(ln10_atom),
                BuiltinPropertyValueSpec::Data(Value::from_f64(std::f64::consts::LN_10)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(ln2_atom),
                BuiltinPropertyValueSpec::Data(Value::from_f64(std::f64::consts::LN_2)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(log10e_atom),
                BuiltinPropertyValueSpec::Data(Value::from_f64(std::f64::consts::LOG10_E)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(log2e_atom),
                BuiltinPropertyValueSpec::Data(Value::from_f64(std::f64::consts::LOG2_E)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(pi_atom),
                BuiltinPropertyValueSpec::Data(Value::from_f64(std::f64::consts::PI)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(sqrt1_2_atom),
                BuiltinPropertyValueSpec::Data(Value::from_f64(std::f64::consts::FRAC_1_SQRT_2)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(sqrt2_atom),
                BuiltinPropertyValueSpec::Data(Value::from_f64(std::f64::consts::SQRT_2)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(math_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let bigint_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(as_int_n_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_bigint_as_int_n_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(as_uint_n_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_bigint_as_uint_n_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
        ];
        let bigint_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.bigint)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::toString.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_bigint_to_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::valueOf.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_bigint_value_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(bigint_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let boolean_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.boolean)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::toString.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_boolean_to_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::valueOf.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_boolean_value_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
        ];
        let symbol_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::r#for.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_symbol_for_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(agent.bootstrap_atoms().key_for()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_symbol_key_for_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(agent.bootstrap_atoms().has_instance()),
                BuiltinPropertyValueSpec::Data(Value::from_symbol_ref(
                    agent.well_known_symbol(WellKnownSymbolId::HasInstance)?,
                )),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(agent.bootstrap_atoms().is_concat_spreadable()),
                BuiltinPropertyValueSpec::Data(Value::from_symbol_ref(
                    agent.well_known_symbol(WellKnownSymbolId::IsConcatSpreadable)?,
                )),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(agent.bootstrap_atoms().iterator()),
                BuiltinPropertyValueSpec::Data(Value::from_symbol_ref(
                    agent.well_known_symbol(WellKnownSymbolId::Iterator)?,
                )),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(agent.bootstrap_atoms().async_iterator()),
                BuiltinPropertyValueSpec::Data(Value::from_symbol_ref(
                    agent.well_known_symbol(WellKnownSymbolId::AsyncIterator)?,
                )),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(agent.bootstrap_atoms().match_()),
                BuiltinPropertyValueSpec::Data(Value::from_symbol_ref(
                    agent.well_known_symbol(WellKnownSymbolId::Match)?,
                )),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(agent.bootstrap_atoms().match_all()),
                BuiltinPropertyValueSpec::Data(Value::from_symbol_ref(
                    agent.well_known_symbol(WellKnownSymbolId::MatchAll)?,
                )),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(agent.bootstrap_atoms().replace()),
                BuiltinPropertyValueSpec::Data(Value::from_symbol_ref(
                    agent.well_known_symbol(WellKnownSymbolId::Replace)?,
                )),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(agent.bootstrap_atoms().search()),
                BuiltinPropertyValueSpec::Data(Value::from_symbol_ref(
                    agent.well_known_symbol(WellKnownSymbolId::Search)?,
                )),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(agent.bootstrap_atoms().species()),
                BuiltinPropertyValueSpec::Data(Value::from_symbol_ref(
                    agent.well_known_symbol(WellKnownSymbolId::Species)?,
                )),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(agent.bootstrap_atoms().split()),
                BuiltinPropertyValueSpec::Data(Value::from_symbol_ref(
                    agent.well_known_symbol(WellKnownSymbolId::Split)?,
                )),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(agent.bootstrap_atoms().to_primitive()),
                BuiltinPropertyValueSpec::Data(Value::from_symbol_ref(
                    agent.well_known_symbol(WellKnownSymbolId::ToPrimitive)?,
                )),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(agent.bootstrap_atoms().to_string_tag()),
                BuiltinPropertyValueSpec::Data(Value::from_symbol_ref(
                    agent.well_known_symbol(WellKnownSymbolId::ToStringTag)?,
                )),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(agent.bootstrap_atoms().unscopables()),
                BuiltinPropertyValueSpec::Data(Value::from_symbol_ref(
                    agent.well_known_symbol(WellKnownSymbolId::Unscopables)?,
                )),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(agent.bootstrap_atoms().dispose()),
                BuiltinPropertyValueSpec::Data(Value::from_symbol_ref(
                    agent.well_known_symbol(WellKnownSymbolId::Dispose)?,
                )),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(agent.bootstrap_atoms().async_dispose()),
                BuiltinPropertyValueSpec::Data(Value::from_symbol_ref(
                    agent.well_known_symbol(WellKnownSymbolId::AsyncDispose)?,
                )),
                BuiltinAttributes::new(false, false, false),
            ),
        ];
        let description_atom = agent.atoms_mut().intern_collectible("description");
        let symbol_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.symbol)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::toString.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_symbol_to_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::valueOf.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_symbol_value_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToPrimitive),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_symbol_to_primitive_builtin()),
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(symbol_tag),
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(description_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_symbol_description_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let stringify_atom = agent.atoms_mut().intern_collectible("stringify");
        let raw_json_atom = agent.atoms_mut().intern_collectible("rawJSON");
        let is_raw_json_atom = agent.atoms_mut().intern_collectible("isRawJSON");
        let apply_atom = agent.atoms_mut().intern_collectible("apply");
        let construct_atom = agent.atoms_mut().intern_collectible("construct");
        let define_property_atom = agent.atoms_mut().intern_collectible("defineProperty");
        let delete_property_atom = agent.atoms_mut().intern_collectible("deleteProperty");
        let get_atom = agent.atoms_mut().intern_collectible("get");
        let get_own_property_descriptor_atom = agent
            .atoms_mut()
            .intern_collectible("getOwnPropertyDescriptor");
        let get_prototype_of_atom = agent.atoms_mut().intern_collectible("getPrototypeOf");
        let has_atom = agent.atoms_mut().intern_collectible("has");
        let is_extensible_atom = agent.atoms_mut().intern_collectible("isExtensible");
        let own_keys_atom = agent.atoms_mut().intern_collectible("ownKeys");
        let prevent_extensions_atom = agent.atoms_mut().intern_collectible("preventExtensions");
        let revocable_atom = agent.atoms_mut().intern_collectible("revocable");
        let set_atom = agent.atoms_mut().intern_collectible("set");
        let set_prototype_of_atom = agent.atoms_mut().intern_collectible("setPrototypeOf");
        let json_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "JSON",
            Some(bootstrap_atoms.json()),
            AllocationLifetime::Default,
        ));
        let reflect_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Reflect",
            None,
            AllocationLifetime::Default,
        ));
        let json_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(parse_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_json_parse_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(stringify_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_json_stringify_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(raw_json_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_json_raw_json_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(is_raw_json_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_json_is_raw_json_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(json_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let reflect_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(apply_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_reflect_apply_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(construct_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_reflect_construct_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(define_property_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_reflect_define_property_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(delete_property_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_reflect_delete_property_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(lyng_js_types::js3_reflect_get_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_own_property_descriptor_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_reflect_get_own_property_descriptor_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_prototype_of_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_reflect_get_prototype_of_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(has_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(lyng_js_types::js3_reflect_has_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(is_extensible_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_reflect_is_extensible_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(own_keys_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_reflect_own_keys_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(prevent_extensions_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_reflect_prevent_extensions_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(lyng_js_types::js3_reflect_set_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_prototype_of_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_reflect_set_prototype_of_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(reflect_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let proxy_descriptors = [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(revocable_atom),
            BuiltinPropertyValueSpec::BuiltinFunction(lyng_js_types::js3_proxy_revocable_builtin()),
            BuiltinAttributes::new(true, false, true),
        )];
        let error_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.error)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::toString.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_error_to_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::name.id()),
                BuiltinPropertyValueSpec::Data(error_name),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bootstrap_atoms.message()),
                BuiltinPropertyValueSpec::Data(empty_string),
                BuiltinAttributes::new(true, false, true),
            ),
        ];
        let eval_error_prototype_descriptors = native_error_prototype_descriptors(
            builtins.eval_error,
            bootstrap_atoms.message(),
            empty_string,
            eval_error_name,
        );
        let range_error_prototype_descriptors = native_error_prototype_descriptors(
            builtins.range_error,
            bootstrap_atoms.message(),
            empty_string,
            range_error_name,
        );
        let reference_error_prototype_descriptors = native_error_prototype_descriptors(
            builtins.reference_error,
            bootstrap_atoms.message(),
            empty_string,
            reference_error_name,
        );
        let syntax_error_prototype_descriptors = native_error_prototype_descriptors(
            builtins.syntax_error,
            bootstrap_atoms.message(),
            empty_string,
            syntax_error_name,
        );
        let type_error_prototype_descriptors = native_error_prototype_descriptors(
            builtins.type_error,
            bootstrap_atoms.message(),
            empty_string,
            type_error_name,
        );
        let uri_error_prototype_descriptors = native_error_prototype_descriptors(
            builtins.uri_error,
            bootstrap_atoms.message(),
            empty_string,
            uri_error_name,
        );
        let aggregate_error_prototype_descriptors = native_error_prototype_descriptors(
            builtins.aggregate_error,
            bootstrap_atoms.message(),
            empty_string,
            aggregate_error_name,
        );
        let suppressed_error_prototype_descriptors = native_error_prototype_descriptors(
            builtins.suppressed_error,
            bootstrap_atoms.message(),
            empty_string,
            suppressed_error_name,
        );
        let promise_tag_atom = agent.atoms_mut().intern_collectible("Promise");
        let promise_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Promise",
            Some(promise_tag_atom),
            AllocationLifetime::Default,
        ));
        let promise_all_atom = agent.atoms_mut().intern_collectible("all");
        let promise_all_settled_atom = agent.atoms_mut().intern_collectible("allSettled");
        let promise_any_atom = agent.atoms_mut().intern_collectible("any");
        let promise_resolve_atom = agent.atoms_mut().intern_collectible("resolve");
        let promise_reject_atom = agent.atoms_mut().intern_collectible("reject");
        let promise_then_atom = agent.atoms_mut().intern_collectible("then");
        let promise_catch_atom = agent.atoms_mut().intern_collectible("catch");
        let promise_race_atom = agent.atoms_mut().intern_collectible("race");
        let promise_finally_atom = agent.atoms_mut().intern_collectible("finally");
        let adopt_atom = agent.atoms_mut().intern_collectible("adopt");
        let defer_atom = agent.atoms_mut().intern_collectible("defer");
        let dispose_atom = agent.atoms_mut().intern_collectible("dispose");
        let dispose_async_atom = agent.atoms_mut().intern_collectible("disposeAsync");
        let disposed_atom = agent.atoms_mut().intern_collectible("disposed");
        let move_atom = agent.atoms_mut().intern_collectible("move");
        let use_atom = agent.atoms_mut().intern_collectible("use");
        let promise_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(promise_resolve_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_promise_resolve_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(promise_reject_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_promise_reject_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(promise_all_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_promise_all_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(promise_all_settled_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_promise_all_settled_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(promise_race_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_promise_race_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(promise_any_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_promise_any_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Species),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_promise_species_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let promise_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.promise)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(promise_then_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_promise_then_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(promise_catch_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_promise_catch_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(promise_finally_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_promise_finally_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(promise_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let suppressed_error_descriptors = [];
        let suppressed_error_prototype_descriptors = [
            suppressed_error_prototype_descriptors[0],
            suppressed_error_prototype_descriptors[1],
            suppressed_error_prototype_descriptors[2],
        ];
        let disposable_stack_descriptors = [];
        let disposable_stack_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.disposable_stack)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(use_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_disposable_stack_use_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(adopt_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_disposable_stack_adopt_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(defer_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_disposable_stack_defer_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(move_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_disposable_stack_move_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(disposed_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(lyng_js_types::js3_disposable_stack_disposed_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(dispose_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_disposable_stack_dispose_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Dispose),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_disposable_stack_dispose_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(disposable_stack_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let async_disposable_stack_descriptors = [];
        let async_disposable_stack_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(
                    builtins.async_disposable_stack,
                )),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(use_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_async_disposable_stack_use_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(adopt_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_async_disposable_stack_adopt_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(defer_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_async_disposable_stack_defer_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(move_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_async_disposable_stack_move_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(disposed_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(lyng_js_types::js3_async_disposable_stack_disposed_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(dispose_async_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_async_disposable_stack_dispose_async_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::AsyncDispose),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    lyng_js_types::js3_async_disposable_stack_dispose_async_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(async_disposable_stack_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let tables = [
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::ArrayBuffer),
                &array_buffer_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::ArrayBufferPrototype),
                &array_buffer_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::SharedArrayBuffer),
                &shared_array_buffer_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::SharedArrayBufferPrototype),
                &shared_array_buffer_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Atomics),
                &atomics_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::DataView),
                &data_view_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::DataViewPrototype),
                &data_view_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::TypedArray),
                &typed_array_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::TypedArrayPrototype),
                &typed_array_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Int8Array),
                &int8_array_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Int8ArrayPrototype),
                &int8_array_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Int16Array),
                &int16_array_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Int16ArrayPrototype),
                &int16_array_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Int32Array),
                &int32_array_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Int32ArrayPrototype),
                &int32_array_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Float32Array),
                &float32_array_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Float32ArrayPrototype),
                &float32_array_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Float64Array),
                &float64_array_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Float64ArrayPrototype),
                &float64_array_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::BigInt64Array),
                &big_int64_array_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::BigInt64ArrayPrototype),
                &big_int64_array_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::BigUint64Array),
                &big_uint64_array_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::BigUint64ArrayPrototype),
                &big_uint64_array_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Uint32Array),
                &uint32_array_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Uint32ArrayPrototype),
                &uint32_array_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Uint16Array),
                &uint16_array_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Uint16ArrayPrototype),
                &uint16_array_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Uint8ClampedArray),
                &uint8_clamped_array_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Uint8ClampedArrayPrototype),
                &uint8_clamped_array_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Uint8Array),
                &uint8_array_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Uint8ArrayPrototype),
                &uint8_array_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::String),
                &string_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::StringPrototype),
                &string_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::StringIteratorPrototype),
                &string_iterator_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::RegExp),
                &regexp_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::RegExpPrototype),
                &regexp_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Date),
                &date_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::DatePrototype),
                &date_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Number),
                &number_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::NumberPrototype),
                &number_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Math),
                &math_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::BigInt),
                &bigint_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::BigIntPrototype),
                &bigint_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::BooleanPrototype),
                &boolean_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Symbol),
                &symbol_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::SymbolPrototype),
                &symbol_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Json),
                &json_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Reflect),
                &reflect_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Proxy),
                &proxy_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::ErrorPrototype),
                &error_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::EvalErrorPrototype),
                &eval_error_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::RangeErrorPrototype),
                &range_error_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::ReferenceErrorPrototype),
                &reference_error_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::SyntaxErrorPrototype),
                &syntax_error_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::TypeErrorPrototype),
                &type_error_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::UriErrorPrototype),
                &uri_error_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::AggregateErrorPrototype),
                &aggregate_error_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::SuppressedError),
                &suppressed_error_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::SuppressedErrorPrototype),
                &suppressed_error_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Promise),
                &promise_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::PromisePrototype),
                &promise_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::DisposableStack),
                &disposable_stack_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::DisposableStackPrototype),
                &disposable_stack_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::AsyncDisposableStack),
                &async_disposable_stack_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::AsyncDisposableStackPrototype),
                &async_disposable_stack_prototype_descriptors,
            ),
        ];
        self.public.insert(realm, builtins);
        if families::install_object_family_descriptors(agent, self, realm, &builtins).is_err() {
            self.public.remove(&realm);
            return None;
        }
        if families::install_function_family_descriptors(agent, self, realm, &builtins).is_err() {
            self.public.remove(&realm);
            return None;
        }
        if families::install_array_family_descriptors(agent, self, realm, &builtins).is_err() {
            self.public.remove(&realm);
            return None;
        }
        if families::install_collection_family_descriptors(agent, self, realm, &builtins).is_err() {
            self.public.remove(&realm);
            return None;
        }
        if families::install_iterator_family_descriptors(agent, self, realm, &builtins).is_err() {
            self.public.remove(&realm);
            return None;
        }
        if install_descriptor_tables(agent, self, realm, &tables).is_err() {
            self.public.remove(&realm);
            return None;
        }

        Some(builtins)
    }
}

#[inline]
pub fn builtin_metadata(entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
    public_builtin_metadata(entry).or_else(|| internal_builtin_metadata(entry))
}

pub(crate) fn allocate_builtin_ordinary_object(
    agent: &mut Agent,
    root_shape: ShapeId,
    prototype: Option<ObjectRef>,
) -> ObjectRef {
    agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(prototype),
            AllocationLifetime::Default,
        )
    })
}

pub(crate) fn allocate_builtin_primitive_wrapper_object(
    agent: &mut Agent,
    root_shape: ShapeId,
    prototype: Option<ObjectRef>,
    wrapper_kind: PrimitiveWrapperKind,
    value: Value,
) -> ObjectRef {
    lyng_js_ops::object::allocate_primitive_wrapper_object(
        agent,
        root_shape,
        prototype,
        wrapper_kind,
        value,
        AllocationLifetime::Default,
    )
    .expect("builtin primitive wrapper allocation should succeed")
}

pub(in crate::public) fn reparent_builtin_object(
    agent: &mut Agent,
    object: ObjectRef,
    prototype: Option<ObjectRef>,
) {
    let _ = agent.with_heap_and_objects(|heap, objects| {
        objects.set_prototype(&mut heap.mutator(), object, prototype)
    });
}

fn native_error_prototype_descriptors(
    constructor: ObjectRef,
    message_atom: AtomId,
    empty_string: Value,
    name: Value,
) -> [BuiltinPropertyDescriptor; 3] {
    [
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
            BuiltinPropertyValueSpec::Data(Value::from_object_ref(constructor)),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(WellKnownAtom::name.id()),
            BuiltinPropertyValueSpec::Data(name),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(message_atom),
            BuiltinPropertyValueSpec::Data(empty_string),
            BuiltinAttributes::new(true, false, true),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_metadata_is_reexported_from_metadata_module() {
        let entry = js3_eval_builtin();

        assert_eq!(
            metadata::public_builtin_metadata(entry),
            public_builtin_metadata(entry)
        );
    }
}

pub(crate) fn allocate_builtin_function_object(
    agent: &mut Agent,
    realm: RealmRef,
    global_env: EnvironmentRef,
    root_shape: ShapeId,
    callable_prototype: ObjectRef,
    prototype_parent: ObjectRef,
    entry: BuiltinFunctionId,
    metadata: BuiltinEntryMetadata,
    prototype_object: Option<ObjectRef>,
) -> ObjectRef {
    let function_data = FunctionObjectData::native(realm, global_env, entry)
        .with_this_mode(FunctionThisMode::Strict)
        .with_has_prototype_property(metadata.has_prototype_property())
        .with_constructor_flags(if metadata.constructible() {
            FunctionConstructorFlags::constructible()
        } else {
            FunctionConstructorFlags::empty()
        });
    let function = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::function(root_shape)
                .with_prototype(Some(callable_prototype))
                .with_cold_data(ObjectColdData::Function(function_data)),
            AllocationLifetime::Default,
        )
    });
    let display_name_atom = agent
        .atoms_mut()
        .intern_collectible(metadata.display_name());
    let display_name = Value::from_string_ref(agent.alloc_runtime_string(
        metadata.display_name(),
        Some(display_name_atom),
        AllocationLifetime::Default,
    ));

    define_builtin_data_property(
        agent,
        function,
        PropertyKey::from_atom(WellKnownAtom::length.id()),
        Value::from_smi(i32::from(metadata.length())),
        false,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        function,
        PropertyKey::from_atom(WellKnownAtom::name.id()),
        display_name,
        false,
        false,
        true,
    );

    if metadata.has_prototype_property() {
        if let Some(prototype_object) = prototype_object {
            define_builtin_data_property(
                agent,
                function,
                PropertyKey::from_atom(WellKnownAtom::prototype.id()),
                Value::from_object_ref(prototype_object),
                false,
                false,
                false,
            );
            if prototype_object != prototype_parent
                && agent
                    .objects()
                    .object_header(agent.heap().view(), prototype_object)
                    .and_then(|header| header.prototype())
                    .is_none()
            {
                let _ = agent.with_heap_and_objects(|heap, objects| {
                    objects.set_prototype(
                        &mut heap.mutator(),
                        prototype_object,
                        Some(prototype_parent),
                    )
                });
            }
        }
    }

    function
}

pub(in crate::public) fn define_builtin_data_property(
    agent: &mut Agent,
    object: ObjectRef,
    key: PropertyKey,
    value: Value,
    writable: bool,
    enumerable: bool,
    configurable: bool,
) {
    let mut descriptor = lyng_js_types::PropertyDescriptor::new();
    descriptor.set_value(value);
    descriptor.set_writable(writable);
    descriptor.set_enumerable(enumerable);
    descriptor.set_configurable(configurable);
    let defined = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.define_own_property(
            &mut mutator,
            object,
            key,
            descriptor,
            AllocationLifetime::Default,
        )
    });
    assert!(
        matches!(defined, Ok(true)),
        "builtin property installation should succeed"
    );
}

pub(in crate::public) fn define_builtin_accessor_property(
    agent: &mut Agent,
    object: ObjectRef,
    key: PropertyKey,
    getter: Option<ObjectRef>,
    setter: Option<ObjectRef>,
    enumerable: bool,
    configurable: bool,
) {
    let mut descriptor = lyng_js_types::PropertyDescriptor::new();
    descriptor.set_getter(
        getter
            .map(Value::from_object_ref)
            .unwrap_or_else(Value::undefined),
    );
    descriptor.set_setter(
        setter
            .map(Value::from_object_ref)
            .unwrap_or_else(Value::undefined),
    );
    descriptor.set_enumerable(enumerable);
    descriptor.set_configurable(configurable);
    let defined = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.define_own_property(
            &mut mutator,
            object,
            key,
            descriptor,
            AllocationLifetime::Default,
        )
    });
    assert!(
        matches!(defined, Ok(true)),
        "builtin accessor installation should succeed"
    );
}

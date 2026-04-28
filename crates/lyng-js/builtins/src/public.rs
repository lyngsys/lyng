mod dispatch;
mod families;
mod metadata;
mod temporal;

pub use metadata::{builtin_metadata, public_builtin_metadata};

use crate::internal::{InternalBuiltinCache, InternalRealmBuiltins};
use crate::BuiltinEntryMetadata;
use lyng_js_common::WellKnownAtom;
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::{
    FunctionConstructorFlags, FunctionObjectData, FunctionThisMode, ObjectAllocation,
    ObjectColdData, PrimitiveWrapperKind,
};
use lyng_js_types::{
    abstract_module_source_builtin, abstract_module_source_to_string_tag_getter_builtin,
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
    array_with_builtin, async_function_builtin, async_generator_function_builtin,
    async_generator_next_builtin, async_generator_return_builtin, async_generator_throw_builtin,
    atomics_add_builtin, atomics_and_builtin, atomics_compare_exchange_builtin,
    atomics_exchange_builtin, atomics_is_lock_free_builtin, atomics_load_builtin,
    atomics_notify_builtin, atomics_or_builtin, atomics_pause_builtin, atomics_store_builtin,
    atomics_sub_builtin, atomics_wait_async_builtin, atomics_wait_builtin, atomics_xor_builtin,
    big_int64_array_builtin, big_uint64_array_builtin, bigint_as_int_n_builtin,
    bigint_as_uint_n_builtin, bigint_builtin, bigint_to_string_builtin, bigint_value_of_builtin,
    boolean_builtin, boolean_to_string_builtin, boolean_value_of_builtin,
    data_view_buffer_getter_builtin, data_view_builtin, data_view_byte_length_getter_builtin,
    data_view_byte_offset_getter_builtin, data_view_get_big_int64_builtin,
    data_view_get_big_uint64_builtin, data_view_get_float32_builtin, data_view_get_float64_builtin,
    data_view_get_int16_builtin, data_view_get_int32_builtin, data_view_get_int8_builtin,
    data_view_get_uint16_builtin, data_view_get_uint32_builtin, data_view_get_uint8_builtin,
    data_view_set_big_int64_builtin, data_view_set_big_uint64_builtin,
    data_view_set_float32_builtin, data_view_set_float64_builtin, data_view_set_int16_builtin,
    data_view_set_int32_builtin, data_view_set_int8_builtin, data_view_set_uint16_builtin,
    data_view_set_uint32_builtin, data_view_set_uint8_builtin, date_builtin, date_get_date_builtin,
    date_get_day_builtin, date_get_full_year_builtin, date_get_hours_builtin,
    date_get_milliseconds_builtin, date_get_minutes_builtin, date_get_month_builtin,
    date_get_seconds_builtin, date_get_time_builtin, date_get_timezone_offset_builtin,
    date_get_utc_date_builtin, date_get_utc_day_builtin, date_get_utc_full_year_builtin,
    date_get_utc_hours_builtin, date_get_utc_milliseconds_builtin, date_get_utc_minutes_builtin,
    date_get_utc_month_builtin, date_get_utc_seconds_builtin, date_now_builtin, date_parse_builtin,
    date_set_date_builtin, date_set_full_year_builtin, date_set_hours_builtin,
    date_set_milliseconds_builtin, date_set_minutes_builtin, date_set_month_builtin,
    date_set_seconds_builtin, date_set_time_builtin, date_set_utc_date_builtin,
    date_set_utc_full_year_builtin, date_set_utc_hours_builtin, date_set_utc_milliseconds_builtin,
    date_set_utc_minutes_builtin, date_set_utc_month_builtin, date_set_utc_seconds_builtin,
    date_to_date_string_builtin, date_to_iso_string_builtin, date_to_json_builtin,
    date_to_locale_date_string_builtin, date_to_locale_string_builtin,
    date_to_locale_time_string_builtin, date_to_primitive_builtin, date_to_string_builtin,
    date_to_temporal_instant_builtin, date_to_time_string_builtin, date_to_utc_string_builtin,
    date_utc_builtin, date_value_of_builtin, decode_uri_builtin, decode_uri_component_builtin,
    encode_uri_builtin, encode_uri_component_builtin, error_builtin, error_is_error_builtin,
    error_to_string_builtin, eval_builtin, eval_error_builtin, finalization_registry_builtin,
    finalization_registry_register_builtin, finalization_registry_unregister_builtin,
    float32_array_builtin, float64_array_builtin, function_apply_builtin, function_bind_builtin,
    function_builtin, function_call_builtin, function_prototype_builtin,
    function_symbol_has_instance_builtin, function_to_string_builtin, generator_function_builtin,
    generator_next_builtin, generator_return_builtin, generator_throw_builtin, int16_array_builtin,
    int32_array_builtin, int8_array_builtin, is_finite_builtin, is_nan_builtin, iterator_builtin,
    iterator_constructor_getter_builtin, iterator_constructor_setter_builtin,
    iterator_every_builtin, iterator_find_builtin, iterator_for_each_builtin,
    iterator_from_builtin, iterator_prototype_iterator_builtin, iterator_reduce_builtin,
    iterator_some_builtin, iterator_to_array_builtin, iterator_to_string_tag_getter_builtin,
    iterator_to_string_tag_setter_builtin, json_is_raw_json_builtin, json_parse_builtin,
    json_raw_json_builtin, json_stringify_builtin, map_builtin, map_clear_builtin,
    map_delete_builtin, map_entries_builtin, map_for_each_builtin, map_get_builtin,
    map_get_or_insert_builtin, map_get_or_insert_computed_builtin, map_has_builtin,
    map_iterator_next_builtin, map_keys_builtin, map_set_builtin, map_size_getter_builtin,
    map_values_builtin, math_abs_builtin, math_acos_builtin, math_acosh_builtin, math_asin_builtin,
    math_asinh_builtin, math_atan2_builtin, math_atan_builtin, math_atanh_builtin,
    math_cbrt_builtin, math_ceil_builtin, math_clz32_builtin, math_cos_builtin, math_cosh_builtin,
    math_exp_builtin, math_expm1_builtin, math_f16round_builtin, math_floor_builtin,
    math_fround_builtin, math_hypot_builtin, math_imul_builtin, math_log10_builtin,
    math_log1p_builtin, math_log2_builtin, math_log_builtin, math_max_builtin, math_min_builtin,
    math_pow_builtin, math_random_builtin, math_round_builtin, math_sign_builtin, math_sin_builtin,
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
    promise_finally_function_builtin, promise_race_builtin, promise_reject_builtin,
    promise_reject_function_builtin, promise_resolve_builtin, promise_resolve_function_builtin,
    promise_species_getter_builtin, promise_then_builtin, promise_try_builtin,
    promise_with_resolvers_builtin, range_error_builtin, reference_error_builtin, regexp_builtin,
    regexp_dot_all_getter_builtin, regexp_escape_builtin, regexp_exec_builtin,
    regexp_flags_getter_builtin, regexp_global_getter_builtin, regexp_has_indices_getter_builtin,
    regexp_ignore_case_getter_builtin, regexp_multiline_getter_builtin,
    regexp_source_getter_builtin, regexp_species_getter_builtin, regexp_sticky_getter_builtin,
    regexp_symbol_match_all_builtin, regexp_symbol_match_builtin, regexp_symbol_replace_builtin,
    regexp_symbol_search_builtin, regexp_symbol_split_builtin, regexp_test_builtin,
    regexp_to_string_builtin, regexp_unicode_getter_builtin, set_add_builtin, set_builtin,
    set_clear_builtin, set_delete_builtin, set_difference_builtin, set_entries_builtin,
    set_for_each_builtin, set_has_builtin, set_intersection_builtin, set_is_disjoint_from_builtin,
    set_is_subset_of_builtin, set_is_superset_of_builtin, set_iterator_next_builtin,
    set_keys_builtin, set_size_getter_builtin, set_symmetric_difference_builtin, set_union_builtin,
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
    string_trim_end_builtin, string_trim_start_builtin, string_value_of_builtin, symbol_builtin,
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
    uint8_array_entries_builtin, uint8_array_keys_builtin, uint8_array_length_getter_builtin,
    uint8_array_set_builtin, uint8_array_slice_builtin, uint8_array_subarray_builtin,
    uint8_array_values_builtin, uint8_clamped_array_builtin, uri_error_builtin, weak_map_builtin,
    weak_map_delete_builtin, weak_map_get_builtin, weak_map_get_or_insert_builtin,
    weak_map_get_or_insert_computed_builtin, weak_map_has_builtin, weak_map_set_builtin,
    weak_ref_builtin, weak_ref_deref_builtin, weak_set_add_builtin, weak_set_builtin,
    weak_set_delete_builtin, weak_set_has_builtin, BuiltinFunctionId, EnvironmentRef, ObjectRef,
    PropertyKey, RealmRef, ShapeId, Value,
};
use std::collections::HashMap;

pub(crate) use dispatch::dispatch_internal_spec_like_builtin;
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
    map_get_or_insert: ObjectRef,
    map_get_or_insert_computed: ObjectRef,
    set_add: ObjectRef,
    set_has: ObjectRef,
    set_delete: ObjectRef,
    set_clear: ObjectRef,
    set_entries: ObjectRef,
    set_values: ObjectRef,
    set_keys: ObjectRef,
    set_for_each: ObjectRef,
    set_size_getter: ObjectRef,
    set_union: ObjectRef,
    set_intersection: ObjectRef,
    set_difference: ObjectRef,
    set_symmetric_difference: ObjectRef,
    set_is_subset_of: ObjectRef,
    set_is_superset_of: ObjectRef,
    set_is_disjoint_from: ObjectRef,
    weak_map_get: ObjectRef,
    weak_map_set: ObjectRef,
    weak_map_has: ObjectRef,
    weak_map_delete: ObjectRef,
    weak_map_get_or_insert: ObjectRef,
    weak_map_get_or_insert_computed: ObjectRef,
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
    atomics_pause: ObjectRef,
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
    data_view_get_big_int64: ObjectRef,
    data_view_get_big_uint64: ObjectRef,
    data_view_set_big_int64: ObjectRef,
    data_view_set_big_uint64: ObjectRef,
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
    iterator: ObjectRef,
    iterator_from: ObjectRef,
    iterator_reduce: ObjectRef,
    iterator_for_each: ObjectRef,
    iterator_some: ObjectRef,
    iterator_every: ObjectRef,
    iterator_find: ObjectRef,
    iterator_to_array: ObjectRef,
    iterator_to_string_tag_getter: ObjectRef,
    iterator_to_string_tag_setter: ObjectRef,
    iterator_constructor_getter: ObjectRef,
    iterator_constructor_setter: ObjectRef,
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
    error_is_error: ObjectRef,
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
    promise_try: ObjectRef,
    promise_with_resolvers: ObjectRef,
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
        let builtins = families::install_public_builtin_families(agent, &scaffolding);
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
        temporal::install_temporal_public_objects(
            agent,
            realm,
            global_env,
            root_shape,
            builtins.function_prototype,
            builtins.object_prototype,
            realm_record.global_object(),
        )?;
        self.public.insert(realm, builtins);
        if families::install_public_family_descriptors(agent, self, realm, &builtins).is_err() {
            self.public.remove(&realm);
            return None;
        }

        Some(builtins)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_metadata_is_reexported_from_metadata_module() {
        let entry = eval_builtin();

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
    let display_name_atom = agent.atoms_mut().intern(metadata.display_name());
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
                    .and_then(lyng_js_objects::ObjectHeader::prototype)
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
    descriptor.set_getter(getter.map_or_else(Value::undefined, Value::from_object_ref));
    descriptor.set_setter(setter.map_or_else(Value::undefined, Value::from_object_ref));
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

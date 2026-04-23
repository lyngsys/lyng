mod dispatch;
mod temporal;

use crate::bootstrap::install_descriptor_tables;
use crate::internal::{internal_builtin_metadata, InternalBuiltinCache, InternalRealmBuiltins};
use crate::{
    BuiltinAttributes, BuiltinDescriptorTable, BuiltinEntryMetadata, BuiltinInstallTarget,
    BuiltinIntrinsic, BuiltinPropertyDescriptor, BuiltinPropertyKeySpec, BuiltinPropertyValueSpec,
};
use lyng_js_common::{AtomId, WellKnownAtom};
use lyng_js_env::Agent;
use lyng_js_gc::{AllocationLifetime, SymbolFlags};
use lyng_js_objects::{
    FunctionConstructorFlags, FunctionObjectData, FunctionThisMode, ObjectAllocation,
    ObjectColdData, OrdinaryObjectData, PrimitiveWrapperKind,
};
use lyng_js_types::{
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
    js3_big_int64_array_builtin, js3_big_uint64_array_builtin, js3_bigint_builtin,
    js3_bigint_to_string_builtin, js3_bigint_value_of_builtin, js3_boolean_builtin,
    js3_boolean_to_string_builtin, js3_boolean_value_of_builtin,
    js3_data_view_buffer_getter_builtin, js3_data_view_builtin,
    js3_data_view_byte_length_getter_builtin, js3_data_view_byte_offset_getter_builtin,
    js3_data_view_get_float32_builtin, js3_data_view_get_float64_builtin,
    js3_data_view_get_int16_builtin, js3_data_view_get_int32_builtin,
    js3_data_view_get_int8_builtin, js3_data_view_get_uint16_builtin,
    js3_data_view_get_uint32_builtin, js3_data_view_get_uint8_builtin,
    js3_data_view_set_float32_builtin, js3_data_view_set_float64_builtin,
    js3_data_view_set_int16_builtin, js3_data_view_set_int32_builtin,
    js3_data_view_set_int8_builtin, js3_data_view_set_uint16_builtin,
    js3_data_view_set_uint32_builtin, js3_data_view_set_uint8_builtin, js3_date_builtin,
    js3_date_get_timezone_offset_builtin, js3_date_now_builtin, js3_date_to_string_builtin,
    js3_date_value_of_builtin, js3_decode_uri_builtin, js3_decode_uri_component_builtin,
    js3_encode_uri_builtin, js3_encode_uri_component_builtin, js3_error_builtin,
    js3_error_to_string_builtin, js3_eval_builtin, js3_eval_error_builtin,
    js3_finalization_registry_builtin, js3_finalization_registry_register_builtin,
    js3_finalization_registry_unregister_builtin, js3_float32_array_builtin,
    js3_float64_array_builtin, js3_function_apply_builtin, js3_function_bind_builtin,
    js3_function_builtin, js3_function_call_builtin, js3_function_prototype_builtin,
    js3_function_to_string_builtin, js3_generator_function_builtin, js3_generator_next_builtin,
    js3_generator_return_builtin, js3_generator_throw_builtin, js3_int16_array_builtin,
    js3_int32_array_builtin, js3_int8_array_builtin, js3_internal_throw_type_error_builtin,
    js3_is_finite_builtin, js3_is_nan_builtin, js3_iterator_prototype_iterator_builtin,
    js3_json_is_raw_json_builtin, js3_json_parse_builtin, js3_json_raw_json_builtin,
    js3_json_stringify_builtin, js3_map_builtin, js3_map_clear_builtin, js3_map_delete_builtin,
    js3_map_entries_builtin, js3_map_for_each_builtin, js3_map_get_builtin, js3_map_has_builtin,
    js3_map_iterator_next_builtin, js3_map_keys_builtin, js3_map_set_builtin,
    js3_map_size_getter_builtin, js3_map_values_builtin, js3_math_abs_builtin,
    js3_math_floor_builtin, js3_math_max_builtin, js3_math_min_builtin, js3_math_pow_builtin,
    js3_math_round_builtin, js3_math_sign_builtin, js3_math_sqrt_builtin, js3_math_trunc_builtin,
    js3_number_builtin, js3_number_is_finite_builtin, js3_number_is_integer_builtin,
    js3_number_is_nan_builtin, js3_number_is_safe_integer_builtin,
    js3_number_to_exponential_builtin, js3_number_to_string_builtin, js3_number_value_of_builtin,
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
    js3_string_iterator_builtin, js3_string_iterator_next_builtin,
    js3_string_last_index_of_builtin, js3_string_match_builtin, js3_string_pad_end_builtin,
    js3_string_pad_start_builtin, js3_string_repeat_builtin, js3_string_replace_builtin,
    js3_string_slice_builtin, js3_string_split_builtin, js3_string_starts_with_builtin,
    js3_string_substring_builtin, js3_string_to_string_builtin, js3_string_value_of_builtin,
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
    array_concat: ObjectRef,
    array_copy_within: ObjectRef,
    array_fill: ObjectRef,
    array_join: ObjectRef,
    array_shift: ObjectRef,
    array_unshift: ObjectRef,
    array_filter: ObjectRef,
    array_for_each: ObjectRef,
    array_map: ObjectRef,
    array_reverse: ObjectRef,
    array_slice: ObjectRef,
    array_sort: ObjectRef,
    array_splice: ObjectRef,
    array_to_string: ObjectRef,
    array_to_locale_string: ObjectRef,
    array_values: ObjectRef,
    array_keys: ObjectRef,
    array_entries: ObjectRef,
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
    string_char_at: ObjectRef,
    string_char_code_at: ObjectRef,
    string_match: ObjectRef,
    string_last_index_of: ObjectRef,
    string_pad_end: ObjectRef,
    string_pad_start: ObjectRef,
    string_repeat: ObjectRef,
    string_replace: ObjectRef,
    string_split: ObjectRef,
    string_slice: ObjectRef,
    string_substring: ObjectRef,
    string_starts_with: ObjectRef,
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
    date: ObjectRef,
    date_prototype: ObjectRef,
    date_now: ObjectRef,
    date_to_string: ObjectRef,
    date_value_of: ObjectRef,
    date_get_timezone_offset: ObjectRef,
    number: ObjectRef,
    number_prototype: ObjectRef,
    number_is_finite: ObjectRef,
    number_is_integer: ObjectRef,
    number_is_nan: ObjectRef,
    number_is_safe_integer: ObjectRef,
    number_to_exponential: ObjectRef,
    number_to_string: ObjectRef,
    number_value_of: ObjectRef,
    math: ObjectRef,
    math_abs: ObjectRef,
    math_floor: ObjectRef,
    math_max: ObjectRef,
    math_min: ObjectRef,
    math_pow: ObjectRef,
    math_round: ObjectRef,
    math_sign: ObjectRef,
    math_sqrt: ObjectRef,
    math_trunc: ObjectRef,
    bigint: ObjectRef,
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
        if entry == js3_object_builtin() {
            return Some(self.object);
        }
        if entry == js3_object_create_builtin() {
            return Some(self.object_create);
        }
        if entry == js3_object_get_prototype_of_builtin() {
            return Some(self.object_get_prototype_of);
        }
        if entry == js3_object_set_prototype_of_builtin() {
            return Some(self.object_set_prototype_of);
        }
        if entry == js3_object_get_own_property_descriptor_builtin() {
            return Some(self.object_get_own_property_descriptor);
        }
        if entry == js3_object_get_own_property_descriptors_builtin() {
            return Some(self.object_get_own_property_descriptors);
        }
        if entry == js3_object_get_own_property_names_builtin() {
            return Some(self.object_get_own_property_names);
        }
        if entry == js3_object_get_own_property_symbols_builtin() {
            return Some(self.object_get_own_property_symbols);
        }
        if entry == js3_object_define_properties_builtin() {
            return Some(self.object_define_properties);
        }
        if entry == js3_object_define_property_builtin() {
            return Some(self.object_define_property);
        }
        if entry == js3_object_prevent_extensions_builtin() {
            return Some(self.object_prevent_extensions);
        }
        if entry == js3_object_is_extensible_builtin() {
            return Some(self.object_is_extensible);
        }
        if entry == js3_object_is_builtin() {
            return Some(self.object_is);
        }
        if entry == js3_object_seal_builtin() {
            return Some(self.object_seal);
        }
        if entry == js3_object_freeze_builtin() {
            return Some(self.object_freeze);
        }
        if entry == js3_object_is_sealed_builtin() {
            return Some(self.object_is_sealed);
        }
        if entry == js3_object_is_frozen_builtin() {
            return Some(self.object_is_frozen);
        }
        if entry == js3_object_to_locale_string_builtin() {
            return Some(self.object_to_locale_string);
        }
        if entry == js3_object_to_string_builtin() {
            return Some(self.object_to_string);
        }
        if entry == js3_object_value_of_builtin() {
            return Some(self.object_value_of);
        }
        if entry == js3_object_has_own_property_builtin() {
            return Some(self.object_has_own_property);
        }
        if entry == js3_object_is_prototype_of_builtin() {
            return Some(self.object_is_prototype_of);
        }
        if entry == js3_object_property_is_enumerable_builtin() {
            return Some(self.object_property_is_enumerable);
        }
        if entry == js3_object_keys_builtin() {
            return Some(self.object_keys);
        }
        if entry == js3_object_entries_builtin() {
            return Some(self.object_entries);
        }
        if entry == js3_object_values_builtin() {
            return Some(self.object_values);
        }
        if entry == js3_object_has_own_builtin() {
            return Some(self.object_has_own);
        }
        if entry == js3_function_builtin() {
            return Some(self.function);
        }
        if entry == js3_function_prototype_builtin() {
            return Some(self.function_prototype);
        }
        if entry == js3_function_call_builtin() {
            return Some(self.function_call);
        }
        if entry == js3_function_apply_builtin() {
            return Some(self.function_apply);
        }
        if entry == js3_function_bind_builtin() {
            return Some(self.function_bind);
        }
        if entry == js3_function_to_string_builtin() {
            return Some(self.function_to_string);
        }
        if entry == js3_async_function_builtin() {
            return Some(self.async_function);
        }
        if entry == js3_async_generator_function_builtin() {
            return Some(self.async_generator_function);
        }
        if entry == js3_async_generator_next_builtin() {
            return Some(self.async_generator_next);
        }
        if entry == js3_async_generator_return_builtin() {
            return Some(self.async_generator_return);
        }
        if entry == js3_async_generator_throw_builtin() {
            return Some(self.async_generator_throw);
        }
        if entry == js3_generator_function_builtin() {
            return Some(self.generator_function);
        }
        if entry == js3_generator_next_builtin() {
            return Some(self.generator_next);
        }
        if entry == js3_generator_return_builtin() {
            return Some(self.generator_return);
        }
        if entry == js3_generator_throw_builtin() {
            return Some(self.generator_throw);
        }
        if entry == js3_array_builtin() {
            return Some(self.array);
        }
        if entry == js3_array_from_builtin() {
            return Some(self.array_from);
        }
        if entry == js3_map_builtin() {
            return Some(self.map);
        }
        if entry == js3_set_builtin() {
            return Some(self.set);
        }
        if entry == js3_weak_map_builtin() {
            return Some(self.weak_map);
        }
        if entry == js3_weak_set_builtin() {
            return Some(self.weak_set);
        }
        if entry == js3_weak_ref_builtin() {
            return Some(self.weak_ref);
        }
        if entry == js3_finalization_registry_builtin() {
            return Some(self.finalization_registry);
        }
        if entry == js3_array_buffer_builtin() {
            return Some(self.array_buffer);
        }
        if entry == js3_array_buffer_is_view_builtin() {
            return Some(self.array_buffer_is_view);
        }
        if entry == js3_shared_array_buffer_builtin() {
            return Some(self.shared_array_buffer);
        }
        if entry == js3_data_view_builtin() {
            return Some(self.data_view);
        }
        if entry == js3_typed_array_builtin() {
            return Some(self.typed_array);
        }
        if entry == js3_typed_array_from_builtin() {
            return Some(self.typed_array_from);
        }
        if entry == js3_typed_array_of_builtin() {
            return Some(self.typed_array_of);
        }
        if entry == js3_int8_array_builtin() {
            return Some(self.int8_array);
        }
        if entry == js3_int16_array_builtin() {
            return Some(self.int16_array);
        }
        if entry == js3_int32_array_builtin() {
            return Some(self.int32_array);
        }
        if entry == js3_float32_array_builtin() {
            return Some(self.float32_array);
        }
        if entry == js3_float64_array_builtin() {
            return Some(self.float64_array);
        }
        if entry == js3_big_int64_array_builtin() {
            return Some(self.big_int64_array);
        }
        if entry == js3_big_uint64_array_builtin() {
            return Some(self.big_uint64_array);
        }
        if entry == js3_uint32_array_builtin() {
            return Some(self.uint32_array);
        }
        if entry == js3_uint16_array_builtin() {
            return Some(self.uint16_array);
        }
        if entry == js3_uint8_clamped_array_builtin() {
            return Some(self.uint8_clamped_array);
        }
        if entry == js3_uint8_array_builtin() {
            return Some(self.uint8_array);
        }
        if entry == js3_array_is_array_builtin() {
            return Some(self.array_is_array);
        }
        if entry == js3_array_concat_builtin() {
            return Some(self.array_concat);
        }
        if entry == js3_array_copy_within_builtin() {
            return Some(self.array_copy_within);
        }
        if entry == js3_array_fill_builtin() {
            return Some(self.array_fill);
        }
        if entry == js3_array_join_builtin() {
            return Some(self.array_join);
        }
        if entry == js3_array_shift_builtin() {
            return Some(self.array_shift);
        }
        if entry == js3_array_unshift_builtin() {
            return Some(self.array_unshift);
        }
        if entry == js3_array_filter_builtin() {
            return Some(self.array_filter);
        }
        if entry == js3_array_for_each_builtin() {
            return Some(self.array_for_each);
        }
        if entry == js3_array_map_builtin() {
            return Some(self.array_map);
        }
        if entry == js3_array_reverse_builtin() {
            return Some(self.array_reverse);
        }
        if entry == js3_array_slice_builtin() {
            return Some(self.array_slice);
        }
        if entry == js3_array_sort_builtin() {
            return Some(self.array_sort);
        }
        if entry == js3_array_splice_builtin() {
            return Some(self.array_splice);
        }
        if entry == js3_array_to_string_builtin() {
            return Some(self.array_to_string);
        }
        if entry == js3_array_to_locale_string_builtin() {
            return Some(self.array_to_locale_string);
        }
        if entry == js3_array_values_builtin() {
            return Some(self.array_values);
        }
        if entry == js3_array_keys_builtin() {
            return Some(self.array_keys);
        }
        if entry == js3_array_entries_builtin() {
            return Some(self.array_entries);
        }
        if entry == js3_map_get_builtin() {
            return Some(self.map_get);
        }
        if entry == js3_map_set_builtin() {
            return Some(self.map_set);
        }
        if entry == js3_map_has_builtin() {
            return Some(self.map_has);
        }
        if entry == js3_map_delete_builtin() {
            return Some(self.map_delete);
        }
        if entry == js3_map_clear_builtin() {
            return Some(self.map_clear);
        }
        if entry == js3_map_entries_builtin() {
            return Some(self.map_entries);
        }
        if entry == js3_map_values_builtin() {
            return Some(self.map_values);
        }
        if entry == js3_map_keys_builtin() {
            return Some(self.map_keys);
        }
        if entry == js3_map_for_each_builtin() {
            return Some(self.map_for_each);
        }
        if entry == js3_map_size_getter_builtin() {
            return Some(self.map_size_getter);
        }
        if entry == js3_set_add_builtin() {
            return Some(self.set_add);
        }
        if entry == js3_set_has_builtin() {
            return Some(self.set_has);
        }
        if entry == js3_set_delete_builtin() {
            return Some(self.set_delete);
        }
        if entry == js3_set_clear_builtin() {
            return Some(self.set_clear);
        }
        if entry == js3_set_entries_builtin() {
            return Some(self.set_entries);
        }
        if entry == js3_set_values_builtin() {
            return Some(self.set_values);
        }
        if entry == js3_set_keys_builtin() {
            return Some(self.set_keys);
        }
        if entry == js3_set_for_each_builtin() {
            return Some(self.set_for_each);
        }
        if entry == js3_set_size_getter_builtin() {
            return Some(self.set_size_getter);
        }
        if entry == js3_weak_map_get_builtin() {
            return Some(self.weak_map_get);
        }
        if entry == js3_weak_map_set_builtin() {
            return Some(self.weak_map_set);
        }
        if entry == js3_weak_map_has_builtin() {
            return Some(self.weak_map_has);
        }
        if entry == js3_weak_map_delete_builtin() {
            return Some(self.weak_map_delete);
        }
        if entry == js3_weak_set_add_builtin() {
            return Some(self.weak_set_add);
        }
        if entry == js3_weak_set_has_builtin() {
            return Some(self.weak_set_has);
        }
        if entry == js3_weak_set_delete_builtin() {
            return Some(self.weak_set_delete);
        }
        if entry == js3_weak_ref_deref_builtin() {
            return Some(self.weak_ref_deref);
        }
        if entry == js3_finalization_registry_register_builtin() {
            return Some(self.finalization_registry_register);
        }
        if entry == js3_finalization_registry_unregister_builtin() {
            return Some(self.finalization_registry_unregister);
        }
        if entry == js3_array_buffer_byte_length_getter_builtin() {
            return Some(self.array_buffer_byte_length_getter);
        }
        if entry == js3_array_buffer_slice_builtin() {
            return Some(self.array_buffer_slice);
        }
        if entry == js3_shared_array_buffer_byte_length_getter_builtin() {
            return Some(self.shared_array_buffer_byte_length_getter);
        }
        if entry == js3_shared_array_buffer_slice_builtin() {
            return Some(self.shared_array_buffer_slice);
        }
        if entry == js3_atomics_load_builtin() {
            return Some(self.atomics_load);
        }
        if entry == js3_atomics_store_builtin() {
            return Some(self.atomics_store);
        }
        if entry == js3_atomics_add_builtin() {
            return Some(self.atomics_add);
        }
        if entry == js3_atomics_sub_builtin() {
            return Some(self.atomics_sub);
        }
        if entry == js3_atomics_and_builtin() {
            return Some(self.atomics_and);
        }
        if entry == js3_atomics_or_builtin() {
            return Some(self.atomics_or);
        }
        if entry == js3_atomics_xor_builtin() {
            return Some(self.atomics_xor);
        }
        if entry == js3_atomics_exchange_builtin() {
            return Some(self.atomics_exchange);
        }
        if entry == js3_atomics_compare_exchange_builtin() {
            return Some(self.atomics_compare_exchange);
        }
        if entry == js3_atomics_notify_builtin() {
            return Some(self.atomics_notify);
        }
        if entry == js3_atomics_wait_builtin() {
            return Some(self.atomics_wait);
        }
        if entry == js3_atomics_wait_async_builtin() {
            return Some(self.atomics_wait_async);
        }
        if entry == js3_atomics_is_lock_free_builtin() {
            return Some(self.atomics_is_lock_free);
        }
        if entry == js3_data_view_buffer_getter_builtin() {
            return Some(self.data_view_buffer_getter);
        }
        if entry == js3_data_view_byte_length_getter_builtin() {
            return Some(self.data_view_byte_length_getter);
        }
        if entry == js3_data_view_byte_offset_getter_builtin() {
            return Some(self.data_view_byte_offset_getter);
        }
        if entry == js3_data_view_get_float32_builtin() {
            return Some(self.data_view_get_float32);
        }
        if entry == js3_data_view_get_float64_builtin() {
            return Some(self.data_view_get_float64);
        }
        if entry == js3_data_view_get_int16_builtin() {
            return Some(self.data_view_get_int16);
        }
        if entry == js3_data_view_get_int32_builtin() {
            return Some(self.data_view_get_int32);
        }
        if entry == js3_data_view_get_int8_builtin() {
            return Some(self.data_view_get_int8);
        }
        if entry == js3_data_view_get_uint16_builtin() {
            return Some(self.data_view_get_uint16);
        }
        if entry == js3_data_view_get_uint32_builtin() {
            return Some(self.data_view_get_uint32);
        }
        if entry == js3_data_view_get_uint8_builtin() {
            return Some(self.data_view_get_uint8);
        }
        if entry == js3_data_view_set_float32_builtin() {
            return Some(self.data_view_set_float32);
        }
        if entry == js3_data_view_set_float64_builtin() {
            return Some(self.data_view_set_float64);
        }
        if entry == js3_data_view_set_int16_builtin() {
            return Some(self.data_view_set_int16);
        }
        if entry == js3_data_view_set_int32_builtin() {
            return Some(self.data_view_set_int32);
        }
        if entry == js3_data_view_set_int8_builtin() {
            return Some(self.data_view_set_int8);
        }
        if entry == js3_data_view_set_uint16_builtin() {
            return Some(self.data_view_set_uint16);
        }
        if entry == js3_data_view_set_uint32_builtin() {
            return Some(self.data_view_set_uint32);
        }
        if entry == js3_data_view_set_uint8_builtin() {
            return Some(self.data_view_set_uint8);
        }
        if entry == js3_uint8_array_buffer_getter_builtin() {
            return Some(self.uint8_array_buffer_getter);
        }
        if entry == js3_uint8_array_byte_length_getter_builtin() {
            return Some(self.uint8_array_byte_length_getter);
        }
        if entry == js3_uint8_array_byte_offset_getter_builtin() {
            return Some(self.uint8_array_byte_offset_getter);
        }
        if entry == js3_uint8_array_length_getter_builtin() {
            return Some(self.uint8_array_length_getter);
        }
        if entry == js3_uint8_array_values_builtin() {
            return Some(self.uint8_array_values);
        }
        if entry == js3_uint8_array_keys_builtin() {
            return Some(self.uint8_array_keys);
        }
        if entry == js3_uint8_array_entries_builtin() {
            return Some(self.uint8_array_entries);
        }
        if entry == js3_uint8_array_set_builtin() {
            return Some(self.uint8_array_set);
        }
        if entry == js3_uint8_array_slice_builtin() {
            return Some(self.uint8_array_slice);
        }
        if entry == js3_uint8_array_subarray_builtin() {
            return Some(self.uint8_array_subarray);
        }
        if entry == js3_typed_array_every_builtin() {
            return Some(self.typed_array_every);
        }
        if entry == js3_typed_array_some_builtin() {
            return Some(self.typed_array_some);
        }
        if entry == js3_typed_array_find_builtin() {
            return Some(self.typed_array_find);
        }
        if entry == js3_typed_array_find_index_builtin() {
            return Some(self.typed_array_find_index);
        }
        if entry == js3_typed_array_find_last_builtin() {
            return Some(self.typed_array_find_last);
        }
        if entry == js3_typed_array_find_last_index_builtin() {
            return Some(self.typed_array_find_last_index);
        }
        if entry == js3_typed_array_fill_builtin() {
            return Some(self.typed_array_fill);
        }
        if entry == js3_typed_array_copy_within_builtin() {
            return Some(self.typed_array_copy_within);
        }
        if entry == js3_typed_array_filter_builtin() {
            return Some(self.typed_array_filter);
        }
        if entry == js3_typed_array_for_each_builtin() {
            return Some(self.typed_array_for_each);
        }
        if entry == js3_typed_array_includes_builtin() {
            return Some(self.typed_array_includes);
        }
        if entry == js3_typed_array_index_of_builtin() {
            return Some(self.typed_array_index_of);
        }
        if entry == js3_typed_array_join_builtin() {
            return Some(self.typed_array_join);
        }
        if entry == js3_typed_array_last_index_of_builtin() {
            return Some(self.typed_array_last_index_of);
        }
        if entry == js3_typed_array_map_builtin() {
            return Some(self.typed_array_map);
        }
        if entry == js3_typed_array_reduce_builtin() {
            return Some(self.typed_array_reduce);
        }
        if entry == js3_typed_array_reduce_right_builtin() {
            return Some(self.typed_array_reduce_right);
        }
        if entry == js3_typed_array_reverse_builtin() {
            return Some(self.typed_array_reverse);
        }
        if entry == js3_typed_array_sort_builtin() {
            return Some(self.typed_array_sort);
        }
        if entry == js3_typed_array_to_locale_string_builtin() {
            return Some(self.typed_array_to_locale_string);
        }
        if entry == js3_typed_array_to_string_builtin() {
            return Some(self.typed_array_to_string);
        }
        if entry == js3_typed_array_to_reversed_builtin() {
            return Some(self.typed_array_to_reversed);
        }
        if entry == js3_typed_array_to_sorted_builtin() {
            return Some(self.typed_array_to_sorted);
        }
        if entry == js3_typed_array_with_builtin() {
            return Some(self.typed_array_with);
        }
        if entry == js3_typed_array_at_builtin() {
            return Some(self.typed_array_at);
        }
        if entry == js3_typed_array_to_string_tag_getter_builtin() {
            return Some(self.typed_array_to_string_tag_getter);
        }
        if entry == js3_iterator_prototype_iterator_builtin() {
            return Some(self.iterator_prototype_iterator);
        }
        if entry == js3_array_iterator_next_builtin() {
            return Some(self.array_iterator_next);
        }
        if entry == js3_map_iterator_next_builtin() {
            return Some(self.map_iterator_next);
        }
        if entry == js3_set_iterator_next_builtin() {
            return Some(self.set_iterator_next);
        }
        if entry == js3_string_builtin() {
            return Some(self.string);
        }
        if entry == js3_string_iterator_builtin() {
            return Some(self.string_iterator);
        }
        if entry == js3_string_iterator_next_builtin() {
            return Some(self.string_iterator_next);
        }
        if entry == js3_string_to_string_builtin() {
            return Some(self.string_to_string);
        }
        if entry == js3_string_value_of_builtin() {
            return Some(self.string_value_of);
        }
        if entry == js3_string_char_at_builtin() {
            return Some(self.string_char_at);
        }
        if entry == js3_string_char_code_at_builtin() {
            return Some(self.string_char_code_at);
        }
        if entry == js3_string_match_builtin() {
            return Some(self.string_match);
        }
        if entry == js3_string_last_index_of_builtin() {
            return Some(self.string_last_index_of);
        }
        if entry == js3_string_pad_end_builtin() {
            return Some(self.string_pad_end);
        }
        if entry == js3_string_pad_start_builtin() {
            return Some(self.string_pad_start);
        }
        if entry == js3_string_repeat_builtin() {
            return Some(self.string_repeat);
        }
        if entry == js3_string_replace_builtin() {
            return Some(self.string_replace);
        }
        if entry == js3_string_split_builtin() {
            return Some(self.string_split);
        }
        if entry == js3_string_slice_builtin() {
            return Some(self.string_slice);
        }
        if entry == js3_string_substring_builtin() {
            return Some(self.string_substring);
        }
        if entry == js3_string_starts_with_builtin() {
            return Some(self.string_starts_with);
        }
        if entry == js3_regexp_builtin() {
            return Some(self.regexp);
        }
        if entry == js3_regexp_escape_builtin() {
            return Some(self.regexp_escape);
        }
        if entry == js3_regexp_to_string_builtin() {
            return Some(self.regexp_to_string);
        }
        if entry == js3_regexp_exec_builtin() {
            return Some(self.regexp_exec);
        }
        if entry == js3_regexp_test_builtin() {
            return Some(self.regexp_test);
        }
        if entry == js3_regexp_global_getter_builtin() {
            return Some(self.regexp_global_getter);
        }
        if entry == js3_regexp_ignore_case_getter_builtin() {
            return Some(self.regexp_ignore_case_getter);
        }
        if entry == js3_regexp_multiline_getter_builtin() {
            return Some(self.regexp_multiline_getter);
        }
        if entry == js3_regexp_dot_all_getter_builtin() {
            return Some(self.regexp_dot_all_getter);
        }
        if entry == js3_regexp_unicode_getter_builtin() {
            return Some(self.regexp_unicode_getter);
        }
        if entry == js3_regexp_sticky_getter_builtin() {
            return Some(self.regexp_sticky_getter);
        }
        if entry == js3_regexp_source_getter_builtin() {
            return Some(self.regexp_source_getter);
        }
        if entry == js3_regexp_flags_getter_builtin() {
            return Some(self.regexp_flags_getter);
        }
        if entry == js3_regexp_has_indices_getter_builtin() {
            return Some(self.regexp_has_indices_getter);
        }
        if entry == js3_date_builtin() {
            return Some(self.date);
        }
        if entry == js3_date_now_builtin() {
            return Some(self.date_now);
        }
        if entry == js3_date_to_string_builtin() {
            return Some(self.date_to_string);
        }
        if entry == js3_date_value_of_builtin() {
            return Some(self.date_value_of);
        }
        if entry == js3_date_get_timezone_offset_builtin() {
            return Some(self.date_get_timezone_offset);
        }
        if entry == js3_number_builtin() {
            return Some(self.number);
        }
        if entry == js3_number_is_finite_builtin() {
            return Some(self.number_is_finite);
        }
        if entry == js3_number_is_integer_builtin() {
            return Some(self.number_is_integer);
        }
        if entry == js3_number_is_nan_builtin() {
            return Some(self.number_is_nan);
        }
        if entry == js3_number_is_safe_integer_builtin() {
            return Some(self.number_is_safe_integer);
        }
        if entry == js3_number_to_exponential_builtin() {
            return Some(self.number_to_exponential);
        }
        if entry == js3_number_to_string_builtin() {
            return Some(self.number_to_string);
        }
        if entry == js3_number_value_of_builtin() {
            return Some(self.number_value_of);
        }
        if entry == js3_math_abs_builtin() {
            return Some(self.math_abs);
        }
        if entry == js3_math_floor_builtin() {
            return Some(self.math_floor);
        }
        if entry == js3_math_max_builtin() {
            return Some(self.math_max);
        }
        if entry == js3_math_min_builtin() {
            return Some(self.math_min);
        }
        if entry == js3_math_pow_builtin() {
            return Some(self.math_pow);
        }
        if entry == js3_math_round_builtin() {
            return Some(self.math_round);
        }
        if entry == js3_math_sign_builtin() {
            return Some(self.math_sign);
        }
        if entry == js3_math_sqrt_builtin() {
            return Some(self.math_sqrt);
        }
        if entry == js3_math_trunc_builtin() {
            return Some(self.math_trunc);
        }
        if entry == js3_bigint_builtin() {
            return Some(self.bigint);
        }
        if entry == js3_bigint_to_string_builtin() {
            return Some(self.bigint_to_string);
        }
        if entry == js3_bigint_value_of_builtin() {
            return Some(self.bigint_value_of);
        }
        if entry == js3_boolean_builtin() {
            return Some(self.boolean);
        }
        if entry == js3_boolean_to_string_builtin() {
            return Some(self.boolean_to_string);
        }
        if entry == js3_boolean_value_of_builtin() {
            return Some(self.boolean_value_of);
        }
        if entry == js3_symbol_builtin() {
            return Some(self.symbol);
        }
        if entry == js3_symbol_for_builtin() {
            return Some(self.symbol_for);
        }
        if entry == js3_symbol_key_for_builtin() {
            return Some(self.symbol_key_for);
        }
        if entry == js3_symbol_to_string_builtin() {
            return Some(self.symbol_to_string);
        }
        if entry == js3_symbol_value_of_builtin() {
            return Some(self.symbol_value_of);
        }
        if entry == js3_symbol_to_primitive_builtin() {
            return Some(self.symbol_to_primitive);
        }
        if entry == js3_array_species_getter_builtin() {
            return Some(self.array_species_getter);
        }
        if entry == js3_symbol_description_getter_builtin() {
            return Some(self.symbol_description_getter);
        }
        if entry == js3_json_parse_builtin() {
            return Some(self.json_parse);
        }
        if entry == js3_json_stringify_builtin() {
            return Some(self.json_stringify);
        }
        if entry == js3_json_raw_json_builtin() {
            return Some(self.json_raw_json);
        }
        if entry == js3_json_is_raw_json_builtin() {
            return Some(self.json_is_raw_json);
        }
        if entry == lyng_js_types::js3_reflect_apply_builtin() {
            return Some(self.reflect_apply);
        }
        if entry == lyng_js_types::js3_reflect_construct_builtin() {
            return Some(self.reflect_construct);
        }
        if entry == lyng_js_types::js3_reflect_define_property_builtin() {
            return Some(self.reflect_define_property);
        }
        if entry == lyng_js_types::js3_reflect_delete_property_builtin() {
            return Some(self.reflect_delete_property);
        }
        if entry == lyng_js_types::js3_reflect_get_builtin() {
            return Some(self.reflect_get);
        }
        if entry == lyng_js_types::js3_reflect_get_own_property_descriptor_builtin() {
            return Some(self.reflect_get_own_property_descriptor);
        }
        if entry == lyng_js_types::js3_reflect_get_prototype_of_builtin() {
            return Some(self.reflect_get_prototype_of);
        }
        if entry == lyng_js_types::js3_reflect_has_builtin() {
            return Some(self.reflect_has);
        }
        if entry == lyng_js_types::js3_reflect_is_extensible_builtin() {
            return Some(self.reflect_is_extensible);
        }
        if entry == lyng_js_types::js3_reflect_own_keys_builtin() {
            return Some(self.reflect_own_keys);
        }
        if entry == lyng_js_types::js3_reflect_prevent_extensions_builtin() {
            return Some(self.reflect_prevent_extensions);
        }
        if entry == lyng_js_types::js3_reflect_set_builtin() {
            return Some(self.reflect_set);
        }
        if entry == lyng_js_types::js3_reflect_set_prototype_of_builtin() {
            return Some(self.reflect_set_prototype_of);
        }
        if entry == lyng_js_types::js3_proxy_builtin() {
            return Some(self.proxy);
        }
        if entry == lyng_js_types::js3_proxy_revocable_builtin() {
            return Some(self.proxy_revocable);
        }
        if entry == js3_error_builtin() {
            return Some(self.error);
        }
        if entry == js3_error_to_string_builtin() {
            return Some(self.error_to_string);
        }
        if entry == js3_eval_error_builtin() {
            return Some(self.eval_error);
        }
        if entry == js3_range_error_builtin() {
            return Some(self.range_error);
        }
        if entry == js3_reference_error_builtin() {
            return Some(self.reference_error);
        }
        if entry == js3_syntax_error_builtin() {
            return Some(self.syntax_error);
        }
        if entry == js3_type_error_builtin() {
            return Some(self.type_error);
        }
        if entry == js3_uri_error_builtin() {
            return Some(self.uri_error);
        }
        if entry == js3_aggregate_error_builtin() {
            return Some(self.aggregate_error);
        }
        if entry == lyng_js_types::js3_suppressed_error_builtin() {
            return Some(self.suppressed_error);
        }
        if entry == js3_promise_builtin() {
            return Some(self.promise);
        }
        if entry == lyng_js_types::js3_disposable_stack_builtin() {
            return Some(self.disposable_stack);
        }
        if entry == lyng_js_types::js3_async_disposable_stack_builtin() {
            return Some(self.async_disposable_stack);
        }
        if entry == lyng_js_types::js3_disposable_stack_use_builtin() {
            return Some(self.disposable_stack_use);
        }
        if entry == lyng_js_types::js3_disposable_stack_adopt_builtin() {
            return Some(self.disposable_stack_adopt);
        }
        if entry == lyng_js_types::js3_disposable_stack_defer_builtin() {
            return Some(self.disposable_stack_defer);
        }
        if entry == lyng_js_types::js3_disposable_stack_move_builtin() {
            return Some(self.disposable_stack_move);
        }
        if entry == lyng_js_types::js3_disposable_stack_disposed_getter_builtin() {
            return Some(self.disposable_stack_disposed_getter);
        }
        if entry == lyng_js_types::js3_disposable_stack_dispose_builtin() {
            return Some(self.disposable_stack_dispose);
        }
        if entry == lyng_js_types::js3_async_disposable_stack_use_builtin() {
            return Some(self.async_disposable_stack_use);
        }
        if entry == lyng_js_types::js3_async_disposable_stack_adopt_builtin() {
            return Some(self.async_disposable_stack_adopt);
        }
        if entry == lyng_js_types::js3_async_disposable_stack_defer_builtin() {
            return Some(self.async_disposable_stack_defer);
        }
        if entry == lyng_js_types::js3_async_disposable_stack_move_builtin() {
            return Some(self.async_disposable_stack_move);
        }
        if entry == lyng_js_types::js3_async_disposable_stack_disposed_getter_builtin() {
            return Some(self.async_disposable_stack_disposed_getter);
        }
        if entry == lyng_js_types::js3_async_disposable_stack_dispose_async_builtin() {
            return Some(self.async_disposable_stack_dispose_async);
        }
        if entry == lyng_js_types::js3_create_sync_disposal_scope_builtin() {
            return Some(self.create_sync_disposal_scope);
        }
        if entry == lyng_js_types::js3_create_async_disposal_scope_builtin() {
            return Some(self.create_async_disposal_scope);
        }
        if entry == lyng_js_types::js3_add_sync_disposable_resource_builtin() {
            return Some(self.add_sync_disposable_resource);
        }
        if entry == lyng_js_types::js3_add_async_disposable_resource_builtin() {
            return Some(self.add_async_disposable_resource);
        }
        if entry == lyng_js_types::js3_dispose_scope_builtin() {
            return Some(self.dispose_scope);
        }
        if entry == lyng_js_types::js3_dispose_scope_async_builtin() {
            return Some(self.dispose_scope_async);
        }
        if entry == js3_promise_then_builtin() {
            return Some(self.promise_then);
        }
        if entry == js3_promise_catch_builtin() {
            return Some(self.promise_catch);
        }
        if entry == js3_promise_finally_builtin() {
            return Some(self.promise_finally);
        }
        if entry == js3_promise_resolve_builtin() {
            return Some(self.promise_resolve);
        }
        if entry == js3_promise_reject_builtin() {
            return Some(self.promise_reject);
        }
        if entry == js3_promise_all_builtin() {
            return Some(self.promise_all);
        }
        if entry == js3_promise_all_settled_builtin() {
            return Some(self.promise_all_settled);
        }
        if entry == js3_promise_race_builtin() {
            return Some(self.promise_race);
        }
        if entry == js3_promise_any_builtin() {
            return Some(self.promise_any);
        }
        if entry == js3_promise_species_getter_builtin() {
            return Some(self.promise_species_getter);
        }
        if entry == js3_eval_builtin() {
            return Some(self.eval);
        }
        if entry == js3_parse_int_builtin() {
            return Some(self.parse_int);
        }
        if entry == js3_parse_float_builtin() {
            return Some(self.parse_float);
        }
        if entry == js3_is_nan_builtin() {
            return Some(self.is_nan);
        }
        if entry == js3_is_finite_builtin() {
            return Some(self.is_finite);
        }
        if entry == js3_encode_uri_builtin() {
            return Some(self.encode_uri);
        }
        if entry == js3_encode_uri_component_builtin() {
            return Some(self.encode_uri_component);
        }
        if entry == js3_decode_uri_builtin() {
            return Some(self.decode_uri);
        }
        if entry == js3_decode_uri_component_builtin() {
            return Some(self.decode_uri_component);
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
        let object_prototype = allocate_builtin_ordinary_object(agent, root_shape, None);
        let function_prototype = allocate_builtin_function_object(
            agent,
            realm,
            global_env,
            root_shape,
            object_prototype,
            object_prototype,
            js3_function_prototype_builtin(),
            public_builtin_metadata(js3_function_prototype_builtin()).unwrap(),
            None,
        );
        let async_function_prototype = existing_intrinsics
            .async_function_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(function_prototype))
            });
        reparent_builtin_object(agent, async_function_prototype, Some(function_prototype));
        let async_generator_function_prototype = existing_intrinsics
            .async_generator_function_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(function_prototype))
            });
        reparent_builtin_object(
            agent,
            async_generator_function_prototype,
            Some(function_prototype),
        );
        let generator_function_prototype = existing_intrinsics
            .generator_function_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(function_prototype))
            });
        reparent_builtin_object(
            agent,
            generator_function_prototype,
            Some(function_prototype),
        );
        let boolean_prototype = existing_intrinsics
            .boolean_prototype()
            .filter(|object| {
                agent.objects().primitive_wrapper_kind(*object)
                    == Some(PrimitiveWrapperKind::Boolean)
            })
            .unwrap_or_else(|| {
                allocate_builtin_primitive_wrapper_object(
                    agent,
                    root_shape,
                    Some(object_prototype),
                    PrimitiveWrapperKind::Boolean,
                    Value::from_bool(false),
                )
            });
        reparent_builtin_object(agent, boolean_prototype, Some(object_prototype));
        let symbol_prototype = existing_intrinsics
            .symbol_prototype()
            .filter(|object| {
                agent.objects().primitive_wrapper_kind(*object)
                    == Some(PrimitiveWrapperKind::Symbol)
            })
            .unwrap_or_else(|| {
                let symbol = agent.heap_mut().mutator().alloc_symbol(
                    None,
                    SymbolFlags::ordinary(),
                    AllocationLifetime::Default,
                );
                allocate_builtin_primitive_wrapper_object(
                    agent,
                    root_shape,
                    Some(object_prototype),
                    PrimitiveWrapperKind::Symbol,
                    Value::from_symbol_ref(symbol),
                )
            });
        reparent_builtin_object(agent, symbol_prototype, Some(object_prototype));
        let array_prototype = internal.array_prototype();
        reparent_builtin_object(agent, array_prototype, Some(object_prototype));
        let map_prototype = existing_intrinsics.map_prototype().unwrap_or_else(|| {
            allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
        });
        reparent_builtin_object(agent, map_prototype, Some(object_prototype));
        let set_prototype = existing_intrinsics.set_prototype().unwrap_or_else(|| {
            allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
        });
        reparent_builtin_object(agent, set_prototype, Some(object_prototype));
        let weak_map_prototype = existing_intrinsics.weak_map_prototype().unwrap_or_else(|| {
            allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
        });
        reparent_builtin_object(agent, weak_map_prototype, Some(object_prototype));
        let weak_set_prototype = existing_intrinsics.weak_set_prototype().unwrap_or_else(|| {
            allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
        });
        reparent_builtin_object(agent, weak_set_prototype, Some(object_prototype));
        let weak_ref_prototype = existing_intrinsics.weak_ref_prototype().unwrap_or_else(|| {
            allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
        });
        reparent_builtin_object(agent, weak_ref_prototype, Some(object_prototype));
        let finalization_registry_prototype = existing_intrinsics
            .finalization_registry_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
            });
        reparent_builtin_object(
            agent,
            finalization_registry_prototype,
            Some(object_prototype),
        );
        let array_buffer_prototype =
            existing_intrinsics
                .array_buffer_prototype()
                .unwrap_or_else(|| {
                    allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
                });
        reparent_builtin_object(agent, array_buffer_prototype, Some(object_prototype));
        let shared_array_buffer_prototype = existing_intrinsics
            .shared_array_buffer_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
            });
        reparent_builtin_object(agent, shared_array_buffer_prototype, Some(object_prototype));
        let data_view_prototype = existing_intrinsics
            .data_view_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
            });
        reparent_builtin_object(agent, data_view_prototype, Some(object_prototype));
        let typed_array_prototype =
            existing_intrinsics
                .typed_array_prototype()
                .unwrap_or_else(|| {
                    allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
                });
        reparent_builtin_object(agent, typed_array_prototype, Some(object_prototype));
        let int8_array_prototype =
            existing_intrinsics
                .int8_array_prototype()
                .unwrap_or_else(|| {
                    allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
                });
        reparent_builtin_object(agent, int8_array_prototype, Some(typed_array_prototype));
        let int16_array_prototype =
            existing_intrinsics
                .int16_array_prototype()
                .unwrap_or_else(|| {
                    allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
                });
        reparent_builtin_object(agent, int16_array_prototype, Some(typed_array_prototype));
        let int32_array_prototype =
            existing_intrinsics
                .int32_array_prototype()
                .unwrap_or_else(|| {
                    allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
                });
        reparent_builtin_object(agent, int32_array_prototype, Some(typed_array_prototype));
        let float32_array_prototype = existing_intrinsics
            .float32_array_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
            });
        reparent_builtin_object(agent, float32_array_prototype, Some(typed_array_prototype));
        let float64_array_prototype = existing_intrinsics
            .float64_array_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
            });
        reparent_builtin_object(agent, float64_array_prototype, Some(typed_array_prototype));
        let big_int64_array_prototype = existing_intrinsics
            .big_int64_array_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
            });
        reparent_builtin_object(
            agent,
            big_int64_array_prototype,
            Some(typed_array_prototype),
        );
        let big_uint64_array_prototype = existing_intrinsics
            .big_uint64_array_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
            });
        reparent_builtin_object(
            agent,
            big_uint64_array_prototype,
            Some(typed_array_prototype),
        );
        let uint32_array_prototype =
            existing_intrinsics
                .uint32_array_prototype()
                .unwrap_or_else(|| {
                    allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
                });
        reparent_builtin_object(agent, uint32_array_prototype, Some(typed_array_prototype));
        let uint16_array_prototype =
            existing_intrinsics
                .uint16_array_prototype()
                .unwrap_or_else(|| {
                    allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
                });
        reparent_builtin_object(agent, uint16_array_prototype, Some(typed_array_prototype));
        let uint8_clamped_array_prototype = existing_intrinsics
            .uint8_clamped_array_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
            });
        reparent_builtin_object(
            agent,
            uint8_clamped_array_prototype,
            Some(typed_array_prototype),
        );
        let uint8_array_prototype =
            existing_intrinsics
                .uint8_array_prototype()
                .unwrap_or_else(|| {
                    allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
                });
        reparent_builtin_object(agent, uint8_array_prototype, Some(typed_array_prototype));
        let iterator_prototype = existing_intrinsics.iterator_prototype().unwrap_or_else(|| {
            allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
        });
        reparent_builtin_object(agent, iterator_prototype, Some(object_prototype));
        let async_iterator_prototype = existing_intrinsics
            .async_iterator_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
            });
        reparent_builtin_object(agent, async_iterator_prototype, Some(object_prototype));
        let generator_prototype = existing_intrinsics
            .generator_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(iterator_prototype))
            });
        reparent_builtin_object(agent, generator_prototype, Some(iterator_prototype));
        let async_generator_prototype = existing_intrinsics
            .async_generator_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(async_iterator_prototype))
            });
        reparent_builtin_object(
            agent,
            async_generator_prototype,
            Some(async_iterator_prototype),
        );
        let async_from_sync_iterator_prototype = existing_intrinsics
            .async_from_sync_iterator_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(async_iterator_prototype))
            });
        reparent_builtin_object(
            agent,
            async_from_sync_iterator_prototype,
            Some(async_iterator_prototype),
        );
        let array_iterator_prototype = existing_intrinsics
            .array_iterator_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(iterator_prototype))
            });
        reparent_builtin_object(agent, array_iterator_prototype, Some(iterator_prototype));
        let map_iterator_prototype =
            existing_intrinsics
                .map_iterator_prototype()
                .unwrap_or_else(|| {
                    allocate_builtin_ordinary_object(agent, root_shape, Some(iterator_prototype))
                });
        reparent_builtin_object(agent, map_iterator_prototype, Some(iterator_prototype));
        let set_iterator_prototype =
            existing_intrinsics
                .set_iterator_prototype()
                .unwrap_or_else(|| {
                    allocate_builtin_ordinary_object(agent, root_shape, Some(iterator_prototype))
                });
        reparent_builtin_object(agent, set_iterator_prototype, Some(iterator_prototype));
        let string_prototype = internal.string_prototype();
        reparent_builtin_object(agent, string_prototype, Some(object_prototype));
        let string_iterator_prototype = existing_intrinsics
            .string_iterator_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(iterator_prototype))
            });
        reparent_builtin_object(agent, string_iterator_prototype, Some(iterator_prototype));
        let regexp_prototype = existing_intrinsics.regexp_prototype().unwrap_or_else(|| {
            allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
        });
        reparent_builtin_object(agent, regexp_prototype, Some(object_prototype));
        let date_prototype = existing_intrinsics.date_prototype().unwrap_or_else(|| {
            allocate_builtin_date_object(
                agent,
                root_shape,
                Some(object_prototype),
                Value::from_f64(f64::NAN),
            )
        });
        reparent_builtin_object(agent, date_prototype, Some(object_prototype));
        let number_prototype = internal.number_prototype();
        let bigint_prototype = internal.bigint_prototype();
        reparent_builtin_object(agent, number_prototype, Some(object_prototype));
        reparent_builtin_object(agent, bigint_prototype, Some(object_prototype));
        let math = allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype));
        let json = allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype));
        let reflect = allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype));

        let error_prototype =
            allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype));
        let eval_error_prototype =
            allocate_builtin_ordinary_object(agent, root_shape, Some(error_prototype));
        let range_error_prototype =
            allocate_builtin_ordinary_object(agent, root_shape, Some(error_prototype));
        let reference_error_prototype =
            allocate_builtin_ordinary_object(agent, root_shape, Some(error_prototype));
        let syntax_error_prototype =
            allocate_builtin_ordinary_object(agent, root_shape, Some(error_prototype));
        let type_error_prototype =
            allocate_builtin_ordinary_object(agent, root_shape, Some(error_prototype));
        let uri_error_prototype =
            allocate_builtin_ordinary_object(agent, root_shape, Some(error_prototype));
        let aggregate_error_prototype =
            allocate_builtin_ordinary_object(agent, root_shape, Some(error_prototype));
        let suppressed_error_prototype =
            allocate_builtin_ordinary_object(agent, root_shape, Some(error_prototype));
        let promise_prototype = existing_intrinsics.promise_prototype().unwrap_or_else(|| {
            allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
        });
        reparent_builtin_object(agent, promise_prototype, Some(object_prototype));
        let disposable_stack_prototype = existing_intrinsics
            .disposable_stack_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
            });
        reparent_builtin_object(agent, disposable_stack_prototype, Some(object_prototype));
        let async_disposable_stack_prototype = existing_intrinsics
            .async_disposable_stack_prototype()
            .unwrap_or_else(|| {
                allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype))
            });
        reparent_builtin_object(
            agent,
            async_disposable_stack_prototype,
            Some(object_prototype),
        );

        let error = allocate_builtin_function_object(
            agent,
            realm,
            global_env,
            root_shape,
            function_prototype,
            object_prototype,
            js3_error_builtin(),
            public_builtin_metadata(js3_error_builtin()).unwrap(),
            Some(error_prototype),
        );

        let builtins = PublicRealmBuiltins {
            object: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_builtin(),
                public_builtin_metadata(js3_object_builtin()).unwrap(),
                Some(object_prototype),
            ),
            object_prototype,
            object_create: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_create_builtin(),
                public_builtin_metadata(js3_object_create_builtin()).unwrap(),
                None,
            ),
            object_get_prototype_of: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_get_prototype_of_builtin(),
                public_builtin_metadata(js3_object_get_prototype_of_builtin()).unwrap(),
                None,
            ),
            object_set_prototype_of: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_set_prototype_of_builtin(),
                public_builtin_metadata(js3_object_set_prototype_of_builtin()).unwrap(),
                None,
            ),
            object_get_own_property_descriptor: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_get_own_property_descriptor_builtin(),
                public_builtin_metadata(js3_object_get_own_property_descriptor_builtin()).unwrap(),
                None,
            ),
            object_get_own_property_descriptors: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_get_own_property_descriptors_builtin(),
                public_builtin_metadata(js3_object_get_own_property_descriptors_builtin()).unwrap(),
                None,
            ),
            object_get_own_property_names: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_get_own_property_names_builtin(),
                public_builtin_metadata(js3_object_get_own_property_names_builtin()).unwrap(),
                None,
            ),
            object_get_own_property_symbols: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_get_own_property_symbols_builtin(),
                public_builtin_metadata(js3_object_get_own_property_symbols_builtin()).unwrap(),
                None,
            ),
            object_define_properties: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_define_properties_builtin(),
                public_builtin_metadata(js3_object_define_properties_builtin()).unwrap(),
                None,
            ),
            object_define_property: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_define_property_builtin(),
                public_builtin_metadata(js3_object_define_property_builtin()).unwrap(),
                None,
            ),
            object_prevent_extensions: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_prevent_extensions_builtin(),
                public_builtin_metadata(js3_object_prevent_extensions_builtin()).unwrap(),
                None,
            ),
            object_is_extensible: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_is_extensible_builtin(),
                public_builtin_metadata(js3_object_is_extensible_builtin()).unwrap(),
                None,
            ),
            object_is: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_is_builtin(),
                public_builtin_metadata(js3_object_is_builtin()).unwrap(),
                None,
            ),
            object_seal: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_seal_builtin(),
                public_builtin_metadata(js3_object_seal_builtin()).unwrap(),
                None,
            ),
            object_freeze: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_freeze_builtin(),
                public_builtin_metadata(js3_object_freeze_builtin()).unwrap(),
                None,
            ),
            object_is_sealed: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_is_sealed_builtin(),
                public_builtin_metadata(js3_object_is_sealed_builtin()).unwrap(),
                None,
            ),
            object_is_frozen: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_is_frozen_builtin(),
                public_builtin_metadata(js3_object_is_frozen_builtin()).unwrap(),
                None,
            ),
            object_to_locale_string: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_to_locale_string_builtin(),
                public_builtin_metadata(js3_object_to_locale_string_builtin()).unwrap(),
                None,
            ),
            object_to_string: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_to_string_builtin(),
                public_builtin_metadata(js3_object_to_string_builtin()).unwrap(),
                None,
            ),
            object_value_of: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_value_of_builtin(),
                public_builtin_metadata(js3_object_value_of_builtin()).unwrap(),
                None,
            ),
            object_has_own_property: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_has_own_property_builtin(),
                public_builtin_metadata(js3_object_has_own_property_builtin()).unwrap(),
                None,
            ),
            object_is_prototype_of: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_is_prototype_of_builtin(),
                public_builtin_metadata(js3_object_is_prototype_of_builtin()).unwrap(),
                None,
            ),
            object_property_is_enumerable: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_property_is_enumerable_builtin(),
                public_builtin_metadata(js3_object_property_is_enumerable_builtin()).unwrap(),
                None,
            ),
            object_keys: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_keys_builtin(),
                public_builtin_metadata(js3_object_keys_builtin()).unwrap(),
                None,
            ),
            object_entries: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_entries_builtin(),
                public_builtin_metadata(js3_object_entries_builtin()).unwrap(),
                None,
            ),
            object_values: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_values_builtin(),
                public_builtin_metadata(js3_object_values_builtin()).unwrap(),
                None,
            ),
            object_has_own: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_object_has_own_builtin(),
                public_builtin_metadata(js3_object_has_own_builtin()).unwrap(),
                None,
            ),
            function: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_function_builtin(),
                public_builtin_metadata(js3_function_builtin()).unwrap(),
                Some(function_prototype),
            ),
            function_prototype,
            function_call: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_function_call_builtin(),
                public_builtin_metadata(js3_function_call_builtin()).unwrap(),
                None,
            ),
            function_apply: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_function_apply_builtin(),
                public_builtin_metadata(js3_function_apply_builtin()).unwrap(),
                None,
            ),
            function_bind: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_function_bind_builtin(),
                public_builtin_metadata(js3_function_bind_builtin()).unwrap(),
                None,
            ),
            function_to_string: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_function_to_string_builtin(),
                public_builtin_metadata(js3_function_to_string_builtin()).unwrap(),
                None,
            ),
            async_function: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_async_function_builtin(),
                public_builtin_metadata(js3_async_function_builtin()).unwrap(),
                Some(async_function_prototype),
            ),
            async_function_prototype,
            async_generator_function: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_async_generator_function_builtin(),
                public_builtin_metadata(js3_async_generator_function_builtin()).unwrap(),
                Some(async_generator_function_prototype),
            ),
            async_generator_function_prototype,
            async_generator_prototype,
            async_generator_next: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_async_generator_next_builtin(),
                public_builtin_metadata(js3_async_generator_next_builtin()).unwrap(),
                None,
            ),
            async_generator_return: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_async_generator_return_builtin(),
                public_builtin_metadata(js3_async_generator_return_builtin()).unwrap(),
                None,
            ),
            async_generator_throw: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_async_generator_throw_builtin(),
                public_builtin_metadata(js3_async_generator_throw_builtin()).unwrap(),
                None,
            ),
            generator_function: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_generator_function_builtin(),
                public_builtin_metadata(js3_generator_function_builtin()).unwrap(),
                Some(generator_function_prototype),
            ),
            generator_function_prototype,
            generator_prototype,
            generator_next: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_generator_next_builtin(),
                public_builtin_metadata(js3_generator_next_builtin()).unwrap(),
                None,
            ),
            generator_return: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_generator_return_builtin(),
                public_builtin_metadata(js3_generator_return_builtin()).unwrap(),
                None,
            ),
            generator_throw: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_generator_throw_builtin(),
                public_builtin_metadata(js3_generator_throw_builtin()).unwrap(),
                None,
            ),
            async_iterator_prototype,
            array: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_builtin(),
                public_builtin_metadata(js3_array_builtin()).unwrap(),
                Some(array_prototype),
            ),
            array_from: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_from_builtin(),
                public_builtin_metadata(js3_array_from_builtin()).unwrap(),
                None,
            ),
            map: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_map_builtin(),
                public_builtin_metadata(js3_map_builtin()).unwrap(),
                Some(map_prototype),
            ),
            set: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_set_builtin(),
                public_builtin_metadata(js3_set_builtin()).unwrap(),
                Some(set_prototype),
            ),
            weak_map: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_weak_map_builtin(),
                public_builtin_metadata(js3_weak_map_builtin()).unwrap(),
                Some(weak_map_prototype),
            ),
            weak_set: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_weak_set_builtin(),
                public_builtin_metadata(js3_weak_set_builtin()).unwrap(),
                Some(weak_set_prototype),
            ),
            weak_ref: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_weak_ref_builtin(),
                public_builtin_metadata(js3_weak_ref_builtin()).unwrap(),
                Some(weak_ref_prototype),
            ),
            finalization_registry: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_finalization_registry_builtin(),
                public_builtin_metadata(js3_finalization_registry_builtin()).unwrap(),
                Some(finalization_registry_prototype),
            ),
            array_buffer: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_buffer_builtin(),
                public_builtin_metadata(js3_array_buffer_builtin()).unwrap(),
                Some(array_buffer_prototype),
            ),
            shared_array_buffer: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_shared_array_buffer_builtin(),
                public_builtin_metadata(js3_shared_array_buffer_builtin()).unwrap(),
                Some(shared_array_buffer_prototype),
            ),
            atomics: allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype)),
            array_buffer_is_view: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_buffer_is_view_builtin(),
                public_builtin_metadata(js3_array_buffer_is_view_builtin()).unwrap(),
                None,
            ),
            data_view: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_builtin(),
                public_builtin_metadata(js3_data_view_builtin()).unwrap(),
                Some(data_view_prototype),
            ),
            typed_array: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_builtin(),
                public_builtin_metadata(js3_typed_array_builtin()).unwrap(),
                Some(typed_array_prototype),
            ),
            typed_array_from: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_from_builtin(),
                public_builtin_metadata(js3_typed_array_from_builtin()).unwrap(),
                None,
            ),
            typed_array_of: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_of_builtin(),
                public_builtin_metadata(js3_typed_array_of_builtin()).unwrap(),
                None,
            ),
            int8_array: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_int8_array_builtin(),
                public_builtin_metadata(js3_int8_array_builtin()).unwrap(),
                Some(int8_array_prototype),
            ),
            int16_array: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_int16_array_builtin(),
                public_builtin_metadata(js3_int16_array_builtin()).unwrap(),
                Some(int16_array_prototype),
            ),
            int32_array: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_int32_array_builtin(),
                public_builtin_metadata(js3_int32_array_builtin()).unwrap(),
                Some(int32_array_prototype),
            ),
            float32_array: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_float32_array_builtin(),
                public_builtin_metadata(js3_float32_array_builtin()).unwrap(),
                Some(float32_array_prototype),
            ),
            float64_array: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_float64_array_builtin(),
                public_builtin_metadata(js3_float64_array_builtin()).unwrap(),
                Some(float64_array_prototype),
            ),
            big_int64_array: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_big_int64_array_builtin(),
                public_builtin_metadata(js3_big_int64_array_builtin()).unwrap(),
                Some(big_int64_array_prototype),
            ),
            big_uint64_array: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_big_uint64_array_builtin(),
                public_builtin_metadata(js3_big_uint64_array_builtin()).unwrap(),
                Some(big_uint64_array_prototype),
            ),
            uint32_array: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_uint32_array_builtin(),
                public_builtin_metadata(js3_uint32_array_builtin()).unwrap(),
                Some(uint32_array_prototype),
            ),
            uint16_array: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_uint16_array_builtin(),
                public_builtin_metadata(js3_uint16_array_builtin()).unwrap(),
                Some(uint16_array_prototype),
            ),
            uint8_clamped_array: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_uint8_clamped_array_builtin(),
                public_builtin_metadata(js3_uint8_clamped_array_builtin()).unwrap(),
                Some(uint8_clamped_array_prototype),
            ),
            uint8_array: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_uint8_array_builtin(),
                public_builtin_metadata(js3_uint8_array_builtin()).unwrap(),
                Some(uint8_array_prototype),
            ),
            array_is_array: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_is_array_builtin(),
                public_builtin_metadata(js3_array_is_array_builtin()).unwrap(),
                None,
            ),
            array_concat: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_concat_builtin(),
                public_builtin_metadata(js3_array_concat_builtin()).unwrap(),
                None,
            ),
            array_copy_within: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_copy_within_builtin(),
                public_builtin_metadata(js3_array_copy_within_builtin()).unwrap(),
                None,
            ),
            array_fill: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_fill_builtin(),
                public_builtin_metadata(js3_array_fill_builtin()).unwrap(),
                None,
            ),
            array_join: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_join_builtin(),
                public_builtin_metadata(js3_array_join_builtin()).unwrap(),
                None,
            ),
            array_unshift: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_unshift_builtin(),
                public_builtin_metadata(js3_array_unshift_builtin()).unwrap(),
                None,
            ),
            array_shift: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_shift_builtin(),
                public_builtin_metadata(js3_array_shift_builtin()).unwrap(),
                None,
            ),
            array_filter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_filter_builtin(),
                public_builtin_metadata(js3_array_filter_builtin()).unwrap(),
                None,
            ),
            array_for_each: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_for_each_builtin(),
                public_builtin_metadata(js3_array_for_each_builtin()).unwrap(),
                None,
            ),
            array_map: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_map_builtin(),
                public_builtin_metadata(js3_array_map_builtin()).unwrap(),
                None,
            ),
            array_reverse: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_reverse_builtin(),
                public_builtin_metadata(js3_array_reverse_builtin()).unwrap(),
                None,
            ),
            array_slice: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_slice_builtin(),
                public_builtin_metadata(js3_array_slice_builtin()).unwrap(),
                None,
            ),
            array_sort: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_sort_builtin(),
                public_builtin_metadata(js3_array_sort_builtin()).unwrap(),
                None,
            ),
            array_splice: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_splice_builtin(),
                public_builtin_metadata(js3_array_splice_builtin()).unwrap(),
                None,
            ),
            array_to_string: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_to_string_builtin(),
                public_builtin_metadata(js3_array_to_string_builtin()).unwrap(),
                None,
            ),
            array_to_locale_string: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_to_locale_string_builtin(),
                public_builtin_metadata(js3_array_to_locale_string_builtin()).unwrap(),
                None,
            ),
            array_values: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_values_builtin(),
                public_builtin_metadata(js3_array_values_builtin()).unwrap(),
                None,
            ),
            array_keys: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_keys_builtin(),
                public_builtin_metadata(js3_array_keys_builtin()).unwrap(),
                None,
            ),
            array_entries: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_entries_builtin(),
                public_builtin_metadata(js3_array_entries_builtin()).unwrap(),
                None,
            ),
            map_get: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_map_get_builtin(),
                public_builtin_metadata(js3_map_get_builtin()).unwrap(),
                None,
            ),
            map_set: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_map_set_builtin(),
                public_builtin_metadata(js3_map_set_builtin()).unwrap(),
                None,
            ),
            map_has: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_map_has_builtin(),
                public_builtin_metadata(js3_map_has_builtin()).unwrap(),
                None,
            ),
            map_delete: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_map_delete_builtin(),
                public_builtin_metadata(js3_map_delete_builtin()).unwrap(),
                None,
            ),
            map_clear: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_map_clear_builtin(),
                public_builtin_metadata(js3_map_clear_builtin()).unwrap(),
                None,
            ),
            map_entries: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_map_entries_builtin(),
                public_builtin_metadata(js3_map_entries_builtin()).unwrap(),
                None,
            ),
            map_values: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_map_values_builtin(),
                public_builtin_metadata(js3_map_values_builtin()).unwrap(),
                None,
            ),
            map_keys: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_map_keys_builtin(),
                public_builtin_metadata(js3_map_keys_builtin()).unwrap(),
                None,
            ),
            map_for_each: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_map_for_each_builtin(),
                public_builtin_metadata(js3_map_for_each_builtin()).unwrap(),
                None,
            ),
            map_size_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_map_size_getter_builtin(),
                public_builtin_metadata(js3_map_size_getter_builtin()).unwrap(),
                None,
            ),
            set_add: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_set_add_builtin(),
                public_builtin_metadata(js3_set_add_builtin()).unwrap(),
                None,
            ),
            set_has: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_set_has_builtin(),
                public_builtin_metadata(js3_set_has_builtin()).unwrap(),
                None,
            ),
            set_delete: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_set_delete_builtin(),
                public_builtin_metadata(js3_set_delete_builtin()).unwrap(),
                None,
            ),
            set_clear: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_set_clear_builtin(),
                public_builtin_metadata(js3_set_clear_builtin()).unwrap(),
                None,
            ),
            set_entries: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_set_entries_builtin(),
                public_builtin_metadata(js3_set_entries_builtin()).unwrap(),
                None,
            ),
            set_values: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_set_values_builtin(),
                public_builtin_metadata(js3_set_values_builtin()).unwrap(),
                None,
            ),
            set_keys: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_set_keys_builtin(),
                public_builtin_metadata(js3_set_keys_builtin()).unwrap(),
                None,
            ),
            set_for_each: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_set_for_each_builtin(),
                public_builtin_metadata(js3_set_for_each_builtin()).unwrap(),
                None,
            ),
            set_size_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_set_size_getter_builtin(),
                public_builtin_metadata(js3_set_size_getter_builtin()).unwrap(),
                None,
            ),
            weak_map_get: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_weak_map_get_builtin(),
                public_builtin_metadata(js3_weak_map_get_builtin()).unwrap(),
                None,
            ),
            weak_map_set: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_weak_map_set_builtin(),
                public_builtin_metadata(js3_weak_map_set_builtin()).unwrap(),
                None,
            ),
            weak_map_has: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_weak_map_has_builtin(),
                public_builtin_metadata(js3_weak_map_has_builtin()).unwrap(),
                None,
            ),
            weak_map_delete: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_weak_map_delete_builtin(),
                public_builtin_metadata(js3_weak_map_delete_builtin()).unwrap(),
                None,
            ),
            weak_set_add: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_weak_set_add_builtin(),
                public_builtin_metadata(js3_weak_set_add_builtin()).unwrap(),
                None,
            ),
            weak_set_has: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_weak_set_has_builtin(),
                public_builtin_metadata(js3_weak_set_has_builtin()).unwrap(),
                None,
            ),
            weak_set_delete: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_weak_set_delete_builtin(),
                public_builtin_metadata(js3_weak_set_delete_builtin()).unwrap(),
                None,
            ),
            weak_ref_deref: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_weak_ref_deref_builtin(),
                public_builtin_metadata(js3_weak_ref_deref_builtin()).unwrap(),
                None,
            ),
            finalization_registry_register: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_finalization_registry_register_builtin(),
                public_builtin_metadata(js3_finalization_registry_register_builtin()).unwrap(),
                None,
            ),
            finalization_registry_unregister: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_finalization_registry_unregister_builtin(),
                public_builtin_metadata(js3_finalization_registry_unregister_builtin()).unwrap(),
                None,
            ),
            map_prototype,
            set_prototype,
            weak_map_prototype,
            weak_set_prototype,
            weak_ref_prototype,
            finalization_registry_prototype,
            array_buffer_prototype,
            shared_array_buffer_prototype,
            array_buffer_byte_length_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_buffer_byte_length_getter_builtin(),
                public_builtin_metadata(js3_array_buffer_byte_length_getter_builtin()).unwrap(),
                None,
            ),
            array_buffer_slice: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_buffer_slice_builtin(),
                public_builtin_metadata(js3_array_buffer_slice_builtin()).unwrap(),
                None,
            ),
            shared_array_buffer_byte_length_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_shared_array_buffer_byte_length_getter_builtin(),
                public_builtin_metadata(js3_shared_array_buffer_byte_length_getter_builtin())
                    .unwrap(),
                None,
            ),
            shared_array_buffer_slice: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_shared_array_buffer_slice_builtin(),
                public_builtin_metadata(js3_shared_array_buffer_slice_builtin()).unwrap(),
                None,
            ),
            atomics_load: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_atomics_load_builtin(),
                public_builtin_metadata(js3_atomics_load_builtin()).unwrap(),
                None,
            ),
            atomics_store: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_atomics_store_builtin(),
                public_builtin_metadata(js3_atomics_store_builtin()).unwrap(),
                None,
            ),
            atomics_add: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_atomics_add_builtin(),
                public_builtin_metadata(js3_atomics_add_builtin()).unwrap(),
                None,
            ),
            atomics_sub: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_atomics_sub_builtin(),
                public_builtin_metadata(js3_atomics_sub_builtin()).unwrap(),
                None,
            ),
            atomics_and: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_atomics_and_builtin(),
                public_builtin_metadata(js3_atomics_and_builtin()).unwrap(),
                None,
            ),
            atomics_or: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_atomics_or_builtin(),
                public_builtin_metadata(js3_atomics_or_builtin()).unwrap(),
                None,
            ),
            atomics_xor: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_atomics_xor_builtin(),
                public_builtin_metadata(js3_atomics_xor_builtin()).unwrap(),
                None,
            ),
            atomics_exchange: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_atomics_exchange_builtin(),
                public_builtin_metadata(js3_atomics_exchange_builtin()).unwrap(),
                None,
            ),
            atomics_compare_exchange: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_atomics_compare_exchange_builtin(),
                public_builtin_metadata(js3_atomics_compare_exchange_builtin()).unwrap(),
                None,
            ),
            atomics_notify: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_atomics_notify_builtin(),
                public_builtin_metadata(js3_atomics_notify_builtin()).unwrap(),
                None,
            ),
            atomics_wait: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_atomics_wait_builtin(),
                public_builtin_metadata(js3_atomics_wait_builtin()).unwrap(),
                None,
            ),
            atomics_wait_async: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_atomics_wait_async_builtin(),
                public_builtin_metadata(js3_atomics_wait_async_builtin()).unwrap(),
                None,
            ),
            atomics_is_lock_free: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_atomics_is_lock_free_builtin(),
                public_builtin_metadata(js3_atomics_is_lock_free_builtin()).unwrap(),
                None,
            ),
            data_view_prototype,
            data_view_buffer_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_buffer_getter_builtin(),
                public_builtin_metadata(js3_data_view_buffer_getter_builtin()).unwrap(),
                None,
            ),
            data_view_byte_length_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_byte_length_getter_builtin(),
                public_builtin_metadata(js3_data_view_byte_length_getter_builtin()).unwrap(),
                None,
            ),
            data_view_byte_offset_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_byte_offset_getter_builtin(),
                public_builtin_metadata(js3_data_view_byte_offset_getter_builtin()).unwrap(),
                None,
            ),
            data_view_get_float32: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_get_float32_builtin(),
                public_builtin_metadata(js3_data_view_get_float32_builtin()).unwrap(),
                None,
            ),
            data_view_get_float64: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_get_float64_builtin(),
                public_builtin_metadata(js3_data_view_get_float64_builtin()).unwrap(),
                None,
            ),
            data_view_get_int16: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_get_int16_builtin(),
                public_builtin_metadata(js3_data_view_get_int16_builtin()).unwrap(),
                None,
            ),
            data_view_get_int32: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_get_int32_builtin(),
                public_builtin_metadata(js3_data_view_get_int32_builtin()).unwrap(),
                None,
            ),
            data_view_get_int8: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_get_int8_builtin(),
                public_builtin_metadata(js3_data_view_get_int8_builtin()).unwrap(),
                None,
            ),
            data_view_get_uint16: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_get_uint16_builtin(),
                public_builtin_metadata(js3_data_view_get_uint16_builtin()).unwrap(),
                None,
            ),
            data_view_get_uint32: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_get_uint32_builtin(),
                public_builtin_metadata(js3_data_view_get_uint32_builtin()).unwrap(),
                None,
            ),
            data_view_get_uint8: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_get_uint8_builtin(),
                public_builtin_metadata(js3_data_view_get_uint8_builtin()).unwrap(),
                None,
            ),
            data_view_set_float32: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_set_float32_builtin(),
                public_builtin_metadata(js3_data_view_set_float32_builtin()).unwrap(),
                None,
            ),
            data_view_set_float64: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_set_float64_builtin(),
                public_builtin_metadata(js3_data_view_set_float64_builtin()).unwrap(),
                None,
            ),
            data_view_set_int16: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_set_int16_builtin(),
                public_builtin_metadata(js3_data_view_set_int16_builtin()).unwrap(),
                None,
            ),
            data_view_set_int32: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_set_int32_builtin(),
                public_builtin_metadata(js3_data_view_set_int32_builtin()).unwrap(),
                None,
            ),
            data_view_set_int8: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_set_int8_builtin(),
                public_builtin_metadata(js3_data_view_set_int8_builtin()).unwrap(),
                None,
            ),
            data_view_set_uint16: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_set_uint16_builtin(),
                public_builtin_metadata(js3_data_view_set_uint16_builtin()).unwrap(),
                None,
            ),
            data_view_set_uint32: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_set_uint32_builtin(),
                public_builtin_metadata(js3_data_view_set_uint32_builtin()).unwrap(),
                None,
            ),
            data_view_set_uint8: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_data_view_set_uint8_builtin(),
                public_builtin_metadata(js3_data_view_set_uint8_builtin()).unwrap(),
                None,
            ),
            typed_array_prototype,
            int8_array_prototype,
            int16_array_prototype,
            int32_array_prototype,
            float32_array_prototype,
            float64_array_prototype,
            big_int64_array_prototype,
            big_uint64_array_prototype,
            uint32_array_prototype,
            uint16_array_prototype,
            uint8_clamped_array_prototype,
            uint8_array_prototype,
            uint8_array_buffer_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_uint8_array_buffer_getter_builtin(),
                public_builtin_metadata(js3_uint8_array_buffer_getter_builtin()).unwrap(),
                None,
            ),
            uint8_array_byte_length_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_uint8_array_byte_length_getter_builtin(),
                public_builtin_metadata(js3_uint8_array_byte_length_getter_builtin()).unwrap(),
                None,
            ),
            uint8_array_byte_offset_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_uint8_array_byte_offset_getter_builtin(),
                public_builtin_metadata(js3_uint8_array_byte_offset_getter_builtin()).unwrap(),
                None,
            ),
            uint8_array_length_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_uint8_array_length_getter_builtin(),
                public_builtin_metadata(js3_uint8_array_length_getter_builtin()).unwrap(),
                None,
            ),
            uint8_array_values: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_uint8_array_values_builtin(),
                public_builtin_metadata(js3_uint8_array_values_builtin()).unwrap(),
                None,
            ),
            uint8_array_keys: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_uint8_array_keys_builtin(),
                public_builtin_metadata(js3_uint8_array_keys_builtin()).unwrap(),
                None,
            ),
            uint8_array_entries: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_uint8_array_entries_builtin(),
                public_builtin_metadata(js3_uint8_array_entries_builtin()).unwrap(),
                None,
            ),
            uint8_array_set: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_uint8_array_set_builtin(),
                public_builtin_metadata(js3_uint8_array_set_builtin()).unwrap(),
                None,
            ),
            uint8_array_slice: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_uint8_array_slice_builtin(),
                public_builtin_metadata(js3_uint8_array_slice_builtin()).unwrap(),
                None,
            ),
            uint8_array_subarray: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_uint8_array_subarray_builtin(),
                public_builtin_metadata(js3_uint8_array_subarray_builtin()).unwrap(),
                None,
            ),
            typed_array_every: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_every_builtin(),
                public_builtin_metadata(js3_typed_array_every_builtin()).unwrap(),
                None,
            ),
            typed_array_some: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_some_builtin(),
                public_builtin_metadata(js3_typed_array_some_builtin()).unwrap(),
                None,
            ),
            typed_array_find: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_find_builtin(),
                public_builtin_metadata(js3_typed_array_find_builtin()).unwrap(),
                None,
            ),
            typed_array_find_index: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_find_index_builtin(),
                public_builtin_metadata(js3_typed_array_find_index_builtin()).unwrap(),
                None,
            ),
            typed_array_find_last: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_find_last_builtin(),
                public_builtin_metadata(js3_typed_array_find_last_builtin()).unwrap(),
                None,
            ),
            typed_array_find_last_index: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_find_last_index_builtin(),
                public_builtin_metadata(js3_typed_array_find_last_index_builtin()).unwrap(),
                None,
            ),
            typed_array_fill: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_fill_builtin(),
                public_builtin_metadata(js3_typed_array_fill_builtin()).unwrap(),
                None,
            ),
            typed_array_copy_within: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_copy_within_builtin(),
                public_builtin_metadata(js3_typed_array_copy_within_builtin()).unwrap(),
                None,
            ),
            typed_array_filter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_filter_builtin(),
                public_builtin_metadata(js3_typed_array_filter_builtin()).unwrap(),
                None,
            ),
            typed_array_for_each: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_for_each_builtin(),
                public_builtin_metadata(js3_typed_array_for_each_builtin()).unwrap(),
                None,
            ),
            typed_array_includes: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_includes_builtin(),
                public_builtin_metadata(js3_typed_array_includes_builtin()).unwrap(),
                None,
            ),
            typed_array_index_of: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_index_of_builtin(),
                public_builtin_metadata(js3_typed_array_index_of_builtin()).unwrap(),
                None,
            ),
            typed_array_join: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_join_builtin(),
                public_builtin_metadata(js3_typed_array_join_builtin()).unwrap(),
                None,
            ),
            typed_array_last_index_of: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_last_index_of_builtin(),
                public_builtin_metadata(js3_typed_array_last_index_of_builtin()).unwrap(),
                None,
            ),
            typed_array_map: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_map_builtin(),
                public_builtin_metadata(js3_typed_array_map_builtin()).unwrap(),
                None,
            ),
            typed_array_reduce: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_reduce_builtin(),
                public_builtin_metadata(js3_typed_array_reduce_builtin()).unwrap(),
                None,
            ),
            typed_array_reduce_right: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_reduce_right_builtin(),
                public_builtin_metadata(js3_typed_array_reduce_right_builtin()).unwrap(),
                None,
            ),
            typed_array_reverse: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_reverse_builtin(),
                public_builtin_metadata(js3_typed_array_reverse_builtin()).unwrap(),
                None,
            ),
            typed_array_sort: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_sort_builtin(),
                public_builtin_metadata(js3_typed_array_sort_builtin()).unwrap(),
                None,
            ),
            typed_array_to_locale_string: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_to_locale_string_builtin(),
                public_builtin_metadata(js3_typed_array_to_locale_string_builtin()).unwrap(),
                None,
            ),
            typed_array_to_string: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_to_string_builtin(),
                public_builtin_metadata(js3_typed_array_to_string_builtin()).unwrap(),
                None,
            ),
            typed_array_to_reversed: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_to_reversed_builtin(),
                public_builtin_metadata(js3_typed_array_to_reversed_builtin()).unwrap(),
                None,
            ),
            typed_array_to_sorted: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_to_sorted_builtin(),
                public_builtin_metadata(js3_typed_array_to_sorted_builtin()).unwrap(),
                None,
            ),
            typed_array_with: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_with_builtin(),
                public_builtin_metadata(js3_typed_array_with_builtin()).unwrap(),
                None,
            ),
            typed_array_at: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_at_builtin(),
                public_builtin_metadata(js3_typed_array_at_builtin()).unwrap(),
                None,
            ),
            typed_array_to_string_tag_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_typed_array_to_string_tag_getter_builtin(),
                public_builtin_metadata(js3_typed_array_to_string_tag_getter_builtin()).unwrap(),
                None,
            ),
            iterator_prototype_iterator: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_iterator_prototype_iterator_builtin(),
                public_builtin_metadata(js3_iterator_prototype_iterator_builtin()).unwrap(),
                None,
            ),
            async_iterator_method: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_iterator_prototype_iterator_builtin(),
                BuiltinEntryMetadata::new("[Symbol.asyncIterator]", 0, false, false),
                None,
            ),
            array_iterator_next: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_iterator_next_builtin(),
                public_builtin_metadata(js3_array_iterator_next_builtin()).unwrap(),
                None,
            ),
            map_iterator_next: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_map_iterator_next_builtin(),
                public_builtin_metadata(js3_map_iterator_next_builtin()).unwrap(),
                None,
            ),
            set_iterator_next: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_set_iterator_next_builtin(),
                public_builtin_metadata(js3_set_iterator_next_builtin()).unwrap(),
                None,
            ),
            string: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_string_builtin(),
                public_builtin_metadata(js3_string_builtin()).unwrap(),
                Some(string_prototype),
            ),
            string_prototype,
            string_iterator: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_string_iterator_builtin(),
                public_builtin_metadata(js3_string_iterator_builtin()).unwrap(),
                None,
            ),
            string_iterator_next: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_string_iterator_next_builtin(),
                public_builtin_metadata(js3_string_iterator_next_builtin()).unwrap(),
                None,
            ),
            string_to_string: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_string_to_string_builtin(),
                public_builtin_metadata(js3_string_to_string_builtin()).unwrap(),
                None,
            ),
            string_value_of: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_string_value_of_builtin(),
                public_builtin_metadata(js3_string_value_of_builtin()).unwrap(),
                None,
            ),
            string_char_at: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_string_char_at_builtin(),
                public_builtin_metadata(js3_string_char_at_builtin()).unwrap(),
                None,
            ),
            string_char_code_at: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_string_char_code_at_builtin(),
                public_builtin_metadata(js3_string_char_code_at_builtin()).unwrap(),
                None,
            ),
            string_match: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_string_match_builtin(),
                public_builtin_metadata(js3_string_match_builtin()).unwrap(),
                None,
            ),
            string_last_index_of: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_string_last_index_of_builtin(),
                public_builtin_metadata(js3_string_last_index_of_builtin()).unwrap(),
                None,
            ),
            string_pad_end: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_string_pad_end_builtin(),
                public_builtin_metadata(js3_string_pad_end_builtin()).unwrap(),
                None,
            ),
            string_pad_start: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_string_pad_start_builtin(),
                public_builtin_metadata(js3_string_pad_start_builtin()).unwrap(),
                None,
            ),
            string_repeat: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_string_repeat_builtin(),
                public_builtin_metadata(js3_string_repeat_builtin()).unwrap(),
                None,
            ),
            string_replace: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_string_replace_builtin(),
                public_builtin_metadata(js3_string_replace_builtin()).unwrap(),
                None,
            ),
            string_split: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_string_split_builtin(),
                public_builtin_metadata(js3_string_split_builtin()).unwrap(),
                None,
            ),
            string_slice: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_string_slice_builtin(),
                public_builtin_metadata(js3_string_slice_builtin()).unwrap(),
                None,
            ),
            string_substring: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_string_substring_builtin(),
                public_builtin_metadata(js3_string_substring_builtin()).unwrap(),
                None,
            ),
            string_starts_with: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_string_starts_with_builtin(),
                public_builtin_metadata(js3_string_starts_with_builtin()).unwrap(),
                None,
            ),
            regexp: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_regexp_builtin(),
                public_builtin_metadata(js3_regexp_builtin()).unwrap(),
                Some(regexp_prototype),
            ),
            regexp_escape: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_regexp_escape_builtin(),
                public_builtin_metadata(js3_regexp_escape_builtin()).unwrap(),
                None,
            ),
            regexp_prototype,
            regexp_to_string: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_regexp_to_string_builtin(),
                public_builtin_metadata(js3_regexp_to_string_builtin()).unwrap(),
                None,
            ),
            regexp_exec: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_regexp_exec_builtin(),
                public_builtin_metadata(js3_regexp_exec_builtin()).unwrap(),
                None,
            ),
            regexp_test: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_regexp_test_builtin(),
                public_builtin_metadata(js3_regexp_test_builtin()).unwrap(),
                None,
            ),
            regexp_global_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_regexp_global_getter_builtin(),
                public_builtin_metadata(js3_regexp_global_getter_builtin()).unwrap(),
                None,
            ),
            regexp_ignore_case_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_regexp_ignore_case_getter_builtin(),
                public_builtin_metadata(js3_regexp_ignore_case_getter_builtin()).unwrap(),
                None,
            ),
            regexp_multiline_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_regexp_multiline_getter_builtin(),
                public_builtin_metadata(js3_regexp_multiline_getter_builtin()).unwrap(),
                None,
            ),
            regexp_dot_all_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_regexp_dot_all_getter_builtin(),
                public_builtin_metadata(js3_regexp_dot_all_getter_builtin()).unwrap(),
                None,
            ),
            regexp_unicode_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_regexp_unicode_getter_builtin(),
                public_builtin_metadata(js3_regexp_unicode_getter_builtin()).unwrap(),
                None,
            ),
            regexp_sticky_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_regexp_sticky_getter_builtin(),
                public_builtin_metadata(js3_regexp_sticky_getter_builtin()).unwrap(),
                None,
            ),
            regexp_source_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_regexp_source_getter_builtin(),
                public_builtin_metadata(js3_regexp_source_getter_builtin()).unwrap(),
                None,
            ),
            regexp_flags_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_regexp_flags_getter_builtin(),
                public_builtin_metadata(js3_regexp_flags_getter_builtin()).unwrap(),
                None,
            ),
            regexp_has_indices_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_regexp_has_indices_getter_builtin(),
                public_builtin_metadata(js3_regexp_has_indices_getter_builtin()).unwrap(),
                None,
            ),
            date: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_date_builtin(),
                public_builtin_metadata(js3_date_builtin()).unwrap(),
                Some(date_prototype),
            ),
            date_prototype,
            date_now: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_date_now_builtin(),
                public_builtin_metadata(js3_date_now_builtin()).unwrap(),
                None,
            ),
            date_to_string: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_date_to_string_builtin(),
                public_builtin_metadata(js3_date_to_string_builtin()).unwrap(),
                None,
            ),
            date_value_of: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_date_value_of_builtin(),
                public_builtin_metadata(js3_date_value_of_builtin()).unwrap(),
                None,
            ),
            date_get_timezone_offset: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_date_get_timezone_offset_builtin(),
                public_builtin_metadata(js3_date_get_timezone_offset_builtin()).unwrap(),
                None,
            ),
            number: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_number_builtin(),
                public_builtin_metadata(js3_number_builtin()).unwrap(),
                Some(number_prototype),
            ),
            number_prototype,
            number_is_finite: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_number_is_finite_builtin(),
                public_builtin_metadata(js3_number_is_finite_builtin()).unwrap(),
                None,
            ),
            number_is_integer: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_number_is_integer_builtin(),
                public_builtin_metadata(js3_number_is_integer_builtin()).unwrap(),
                None,
            ),
            number_is_nan: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_number_is_nan_builtin(),
                public_builtin_metadata(js3_number_is_nan_builtin()).unwrap(),
                None,
            ),
            number_is_safe_integer: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_number_is_safe_integer_builtin(),
                public_builtin_metadata(js3_number_is_safe_integer_builtin()).unwrap(),
                None,
            ),
            number_to_exponential: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_number_to_exponential_builtin(),
                public_builtin_metadata(js3_number_to_exponential_builtin()).unwrap(),
                None,
            ),
            number_to_string: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_number_to_string_builtin(),
                public_builtin_metadata(js3_number_to_string_builtin()).unwrap(),
                None,
            ),
            number_value_of: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_number_value_of_builtin(),
                public_builtin_metadata(js3_number_value_of_builtin()).unwrap(),
                None,
            ),
            math,
            math_abs: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_math_abs_builtin(),
                public_builtin_metadata(js3_math_abs_builtin()).unwrap(),
                None,
            ),
            math_floor: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_math_floor_builtin(),
                public_builtin_metadata(js3_math_floor_builtin()).unwrap(),
                None,
            ),
            math_max: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_math_max_builtin(),
                public_builtin_metadata(js3_math_max_builtin()).unwrap(),
                None,
            ),
            math_min: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_math_min_builtin(),
                public_builtin_metadata(js3_math_min_builtin()).unwrap(),
                None,
            ),
            math_pow: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_math_pow_builtin(),
                public_builtin_metadata(js3_math_pow_builtin()).unwrap(),
                None,
            ),
            math_round: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_math_round_builtin(),
                public_builtin_metadata(js3_math_round_builtin()).unwrap(),
                None,
            ),
            math_sign: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_math_sign_builtin(),
                public_builtin_metadata(js3_math_sign_builtin()).unwrap(),
                None,
            ),
            math_sqrt: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_math_sqrt_builtin(),
                public_builtin_metadata(js3_math_sqrt_builtin()).unwrap(),
                None,
            ),
            math_trunc: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_math_trunc_builtin(),
                public_builtin_metadata(js3_math_trunc_builtin()).unwrap(),
                None,
            ),
            bigint: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_bigint_builtin(),
                public_builtin_metadata(js3_bigint_builtin()).unwrap(),
                Some(bigint_prototype),
            ),
            bigint_prototype,
            bigint_to_string: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_bigint_to_string_builtin(),
                public_builtin_metadata(js3_bigint_to_string_builtin()).unwrap(),
                None,
            ),
            bigint_value_of: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_bigint_value_of_builtin(),
                public_builtin_metadata(js3_bigint_value_of_builtin()).unwrap(),
                None,
            ),
            boolean: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_boolean_builtin(),
                public_builtin_metadata(js3_boolean_builtin()).unwrap(),
                Some(boolean_prototype),
            ),
            boolean_prototype,
            boolean_to_string: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_boolean_to_string_builtin(),
                public_builtin_metadata(js3_boolean_to_string_builtin()).unwrap(),
                None,
            ),
            boolean_value_of: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_boolean_value_of_builtin(),
                public_builtin_metadata(js3_boolean_value_of_builtin()).unwrap(),
                None,
            ),
            symbol: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_symbol_builtin(),
                public_builtin_metadata(js3_symbol_builtin()).unwrap(),
                Some(symbol_prototype),
            ),
            symbol_prototype,
            symbol_for: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_symbol_for_builtin(),
                public_builtin_metadata(js3_symbol_for_builtin()).unwrap(),
                None,
            ),
            symbol_key_for: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_symbol_key_for_builtin(),
                public_builtin_metadata(js3_symbol_key_for_builtin()).unwrap(),
                None,
            ),
            symbol_to_string: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_symbol_to_string_builtin(),
                public_builtin_metadata(js3_symbol_to_string_builtin()).unwrap(),
                None,
            ),
            symbol_value_of: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_symbol_value_of_builtin(),
                public_builtin_metadata(js3_symbol_value_of_builtin()).unwrap(),
                None,
            ),
            symbol_to_primitive: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_symbol_to_primitive_builtin(),
                public_builtin_metadata(js3_symbol_to_primitive_builtin()).unwrap(),
                None,
            ),
            array_species_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_array_species_getter_builtin(),
                public_builtin_metadata(js3_array_species_getter_builtin()).unwrap(),
                None,
            ),
            symbol_description_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_symbol_description_getter_builtin(),
                public_builtin_metadata(js3_symbol_description_getter_builtin()).unwrap(),
                None,
            ),
            json,
            json_parse: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_json_parse_builtin(),
                public_builtin_metadata(js3_json_parse_builtin()).unwrap(),
                None,
            ),
            json_stringify: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_json_stringify_builtin(),
                public_builtin_metadata(js3_json_stringify_builtin()).unwrap(),
                None,
            ),
            json_raw_json: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_json_raw_json_builtin(),
                public_builtin_metadata(js3_json_raw_json_builtin()).unwrap(),
                None,
            ),
            json_is_raw_json: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_json_is_raw_json_builtin(),
                public_builtin_metadata(js3_json_is_raw_json_builtin()).unwrap(),
                None,
            ),
            reflect,
            reflect_apply: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_reflect_apply_builtin(),
                public_builtin_metadata(lyng_js_types::js3_reflect_apply_builtin()).unwrap(),
                None,
            ),
            reflect_construct: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_reflect_construct_builtin(),
                public_builtin_metadata(lyng_js_types::js3_reflect_construct_builtin()).unwrap(),
                None,
            ),
            reflect_define_property: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_reflect_define_property_builtin(),
                public_builtin_metadata(lyng_js_types::js3_reflect_define_property_builtin())
                    .unwrap(),
                None,
            ),
            reflect_delete_property: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_reflect_delete_property_builtin(),
                public_builtin_metadata(lyng_js_types::js3_reflect_delete_property_builtin())
                    .unwrap(),
                None,
            ),
            reflect_get: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_reflect_get_builtin(),
                public_builtin_metadata(lyng_js_types::js3_reflect_get_builtin()).unwrap(),
                None,
            ),
            reflect_get_own_property_descriptor: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_reflect_get_own_property_descriptor_builtin(),
                public_builtin_metadata(
                    lyng_js_types::js3_reflect_get_own_property_descriptor_builtin(),
                )
                .unwrap(),
                None,
            ),
            reflect_get_prototype_of: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_reflect_get_prototype_of_builtin(),
                public_builtin_metadata(lyng_js_types::js3_reflect_get_prototype_of_builtin())
                    .unwrap(),
                None,
            ),
            reflect_has: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_reflect_has_builtin(),
                public_builtin_metadata(lyng_js_types::js3_reflect_has_builtin()).unwrap(),
                None,
            ),
            reflect_is_extensible: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_reflect_is_extensible_builtin(),
                public_builtin_metadata(lyng_js_types::js3_reflect_is_extensible_builtin()).unwrap(),
                None,
            ),
            reflect_own_keys: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_reflect_own_keys_builtin(),
                public_builtin_metadata(lyng_js_types::js3_reflect_own_keys_builtin()).unwrap(),
                None,
            ),
            reflect_prevent_extensions: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_reflect_prevent_extensions_builtin(),
                public_builtin_metadata(lyng_js_types::js3_reflect_prevent_extensions_builtin())
                    .unwrap(),
                None,
            ),
            reflect_set: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_reflect_set_builtin(),
                public_builtin_metadata(lyng_js_types::js3_reflect_set_builtin()).unwrap(),
                None,
            ),
            reflect_set_prototype_of: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_reflect_set_prototype_of_builtin(),
                public_builtin_metadata(lyng_js_types::js3_reflect_set_prototype_of_builtin())
                    .unwrap(),
                None,
            ),
            proxy: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_proxy_builtin(),
                public_builtin_metadata(lyng_js_types::js3_proxy_builtin()).unwrap(),
                None,
            ),
            proxy_revocable: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_proxy_revocable_builtin(),
                public_builtin_metadata(lyng_js_types::js3_proxy_revocable_builtin()).unwrap(),
                None,
            ),
            error,
            error_prototype,
            error_to_string: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_error_to_string_builtin(),
                public_builtin_metadata(js3_error_to_string_builtin()).unwrap(),
                None,
            ),
            eval_error: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                error,
                object_prototype,
                js3_eval_error_builtin(),
                public_builtin_metadata(js3_eval_error_builtin()).unwrap(),
                Some(eval_error_prototype),
            ),
            eval_error_prototype,
            range_error: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                error,
                object_prototype,
                js3_range_error_builtin(),
                public_builtin_metadata(js3_range_error_builtin()).unwrap(),
                Some(range_error_prototype),
            ),
            range_error_prototype,
            reference_error: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                error,
                object_prototype,
                js3_reference_error_builtin(),
                public_builtin_metadata(js3_reference_error_builtin()).unwrap(),
                Some(reference_error_prototype),
            ),
            reference_error_prototype,
            syntax_error: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                error,
                object_prototype,
                js3_syntax_error_builtin(),
                public_builtin_metadata(js3_syntax_error_builtin()).unwrap(),
                Some(syntax_error_prototype),
            ),
            syntax_error_prototype,
            type_error: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                error,
                object_prototype,
                js3_type_error_builtin(),
                public_builtin_metadata(js3_type_error_builtin()).unwrap(),
                Some(type_error_prototype),
            ),
            type_error_prototype,
            uri_error: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                error,
                object_prototype,
                js3_uri_error_builtin(),
                public_builtin_metadata(js3_uri_error_builtin()).unwrap(),
                Some(uri_error_prototype),
            ),
            uri_error_prototype,
            aggregate_error: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                error,
                object_prototype,
                js3_aggregate_error_builtin(),
                public_builtin_metadata(js3_aggregate_error_builtin()).unwrap(),
                Some(aggregate_error_prototype),
            ),
            aggregate_error_prototype,
            suppressed_error: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                error,
                object_prototype,
                lyng_js_types::js3_suppressed_error_builtin(),
                public_builtin_metadata(lyng_js_types::js3_suppressed_error_builtin()).unwrap(),
                Some(suppressed_error_prototype),
            ),
            suppressed_error_prototype,
            promise: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_promise_builtin(),
                public_builtin_metadata(js3_promise_builtin()).unwrap(),
                Some(promise_prototype),
            ),
            promise_prototype,
            disposable_stack: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_disposable_stack_builtin(),
                public_builtin_metadata(lyng_js_types::js3_disposable_stack_builtin()).unwrap(),
                Some(disposable_stack_prototype),
            ),
            disposable_stack_prototype,
            async_disposable_stack: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_async_disposable_stack_builtin(),
                public_builtin_metadata(lyng_js_types::js3_async_disposable_stack_builtin())
                    .unwrap(),
                Some(async_disposable_stack_prototype),
            ),
            async_disposable_stack_prototype,
            disposable_stack_use: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_disposable_stack_use_builtin(),
                public_builtin_metadata(lyng_js_types::js3_disposable_stack_use_builtin()).unwrap(),
                None,
            ),
            disposable_stack_adopt: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_disposable_stack_adopt_builtin(),
                public_builtin_metadata(lyng_js_types::js3_disposable_stack_adopt_builtin())
                    .unwrap(),
                None,
            ),
            disposable_stack_defer: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_disposable_stack_defer_builtin(),
                public_builtin_metadata(lyng_js_types::js3_disposable_stack_defer_builtin())
                    .unwrap(),
                None,
            ),
            disposable_stack_move: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_disposable_stack_move_builtin(),
                public_builtin_metadata(lyng_js_types::js3_disposable_stack_move_builtin()).unwrap(),
                None,
            ),
            disposable_stack_disposed_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_disposable_stack_disposed_getter_builtin(),
                public_builtin_metadata(
                    lyng_js_types::js3_disposable_stack_disposed_getter_builtin(),
                )
                .unwrap(),
                None,
            ),
            disposable_stack_dispose: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_disposable_stack_dispose_builtin(),
                public_builtin_metadata(lyng_js_types::js3_disposable_stack_dispose_builtin())
                    .unwrap(),
                None,
            ),
            async_disposable_stack_use: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_async_disposable_stack_use_builtin(),
                public_builtin_metadata(lyng_js_types::js3_async_disposable_stack_use_builtin())
                    .unwrap(),
                None,
            ),
            async_disposable_stack_adopt: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_async_disposable_stack_adopt_builtin(),
                public_builtin_metadata(lyng_js_types::js3_async_disposable_stack_adopt_builtin())
                    .unwrap(),
                None,
            ),
            async_disposable_stack_defer: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_async_disposable_stack_defer_builtin(),
                public_builtin_metadata(lyng_js_types::js3_async_disposable_stack_defer_builtin())
                    .unwrap(),
                None,
            ),
            async_disposable_stack_move: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_async_disposable_stack_move_builtin(),
                public_builtin_metadata(lyng_js_types::js3_async_disposable_stack_move_builtin())
                    .unwrap(),
                None,
            ),
            async_disposable_stack_disposed_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_async_disposable_stack_disposed_getter_builtin(),
                public_builtin_metadata(
                    lyng_js_types::js3_async_disposable_stack_disposed_getter_builtin(),
                )
                .unwrap(),
                None,
            ),
            async_disposable_stack_dispose_async: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_async_disposable_stack_dispose_async_builtin(),
                public_builtin_metadata(
                    lyng_js_types::js3_async_disposable_stack_dispose_async_builtin(),
                )
                .unwrap(),
                None,
            ),
            create_sync_disposal_scope: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_create_sync_disposal_scope_builtin(),
                public_builtin_metadata(lyng_js_types::js3_create_sync_disposal_scope_builtin())
                    .unwrap(),
                None,
            ),
            create_async_disposal_scope: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_create_async_disposal_scope_builtin(),
                public_builtin_metadata(lyng_js_types::js3_create_async_disposal_scope_builtin())
                    .unwrap(),
                None,
            ),
            add_sync_disposable_resource: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_add_sync_disposable_resource_builtin(),
                public_builtin_metadata(lyng_js_types::js3_add_sync_disposable_resource_builtin())
                    .unwrap(),
                None,
            ),
            add_async_disposable_resource: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_add_async_disposable_resource_builtin(),
                public_builtin_metadata(lyng_js_types::js3_add_async_disposable_resource_builtin())
                    .unwrap(),
                None,
            ),
            dispose_scope: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_dispose_scope_builtin(),
                public_builtin_metadata(lyng_js_types::js3_dispose_scope_builtin()).unwrap(),
                None,
            ),
            dispose_scope_async: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                lyng_js_types::js3_dispose_scope_async_builtin(),
                public_builtin_metadata(lyng_js_types::js3_dispose_scope_async_builtin()).unwrap(),
                None,
            ),
            promise_then: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_promise_then_builtin(),
                public_builtin_metadata(js3_promise_then_builtin()).unwrap(),
                None,
            ),
            promise_catch: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_promise_catch_builtin(),
                public_builtin_metadata(js3_promise_catch_builtin()).unwrap(),
                None,
            ),
            promise_finally: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_promise_finally_builtin(),
                public_builtin_metadata(js3_promise_finally_builtin()).unwrap(),
                None,
            ),
            promise_resolve: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_promise_resolve_builtin(),
                public_builtin_metadata(js3_promise_resolve_builtin()).unwrap(),
                None,
            ),
            promise_reject: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_promise_reject_builtin(),
                public_builtin_metadata(js3_promise_reject_builtin()).unwrap(),
                None,
            ),
            promise_all: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_promise_all_builtin(),
                public_builtin_metadata(js3_promise_all_builtin()).unwrap(),
                None,
            ),
            promise_all_settled: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_promise_all_settled_builtin(),
                public_builtin_metadata(js3_promise_all_settled_builtin()).unwrap(),
                None,
            ),
            promise_race: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_promise_race_builtin(),
                public_builtin_metadata(js3_promise_race_builtin()).unwrap(),
                None,
            ),
            promise_any: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_promise_any_builtin(),
                public_builtin_metadata(js3_promise_any_builtin()).unwrap(),
                None,
            ),
            promise_species_getter: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_promise_species_getter_builtin(),
                public_builtin_metadata(js3_promise_species_getter_builtin()).unwrap(),
                None,
            ),
            eval: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_eval_builtin(),
                public_builtin_metadata(js3_eval_builtin()).unwrap(),
                None,
            ),
            parse_int: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_parse_int_builtin(),
                public_builtin_metadata(js3_parse_int_builtin()).unwrap(),
                None,
            ),
            parse_float: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_parse_float_builtin(),
                public_builtin_metadata(js3_parse_float_builtin()).unwrap(),
                None,
            ),
            is_nan: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_is_nan_builtin(),
                public_builtin_metadata(js3_is_nan_builtin()).unwrap(),
                None,
            ),
            is_finite: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_is_finite_builtin(),
                public_builtin_metadata(js3_is_finite_builtin()).unwrap(),
                None,
            ),
            encode_uri: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_encode_uri_builtin(),
                public_builtin_metadata(js3_encode_uri_builtin()).unwrap(),
                None,
            ),
            encode_uri_component: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_encode_uri_component_builtin(),
                public_builtin_metadata(js3_encode_uri_component_builtin()).unwrap(),
                None,
            ),
            decode_uri: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_decode_uri_builtin(),
                public_builtin_metadata(js3_decode_uri_builtin()).unwrap(),
                None,
            ),
            decode_uri_component: allocate_builtin_function_object(
                agent,
                realm,
                global_env,
                root_shape,
                function_prototype,
                object_prototype,
                js3_decode_uri_component_builtin(),
                public_builtin_metadata(js3_decode_uri_component_builtin()).unwrap(),
                None,
            ),
        };
        reparent_builtin_object(agent, builtins.async_function, Some(builtins.function));
        reparent_builtin_object(agent, builtins.generator_function, Some(builtins.function));
        reparent_builtin_object(
            agent,
            builtins.async_generator_function,
            Some(builtins.function),
        );
        reparent_builtin_object(agent, builtins.int8_array, Some(builtins.typed_array));
        reparent_builtin_object(agent, builtins.int16_array, Some(builtins.typed_array));
        reparent_builtin_object(agent, builtins.int32_array, Some(builtins.typed_array));
        reparent_builtin_object(agent, builtins.float32_array, Some(builtins.typed_array));
        reparent_builtin_object(agent, builtins.float64_array, Some(builtins.typed_array));
        reparent_builtin_object(agent, builtins.big_int64_array, Some(builtins.typed_array));
        reparent_builtin_object(agent, builtins.big_uint64_array, Some(builtins.typed_array));
        reparent_builtin_object(agent, builtins.uint32_array, Some(builtins.typed_array));
        reparent_builtin_object(agent, builtins.uint16_array, Some(builtins.typed_array));
        reparent_builtin_object(
            agent,
            builtins.uint8_clamped_array,
            Some(builtins.typed_array),
        );
        reparent_builtin_object(agent, builtins.uint8_array, Some(builtins.typed_array));

        let updated_intrinsics = existing_intrinsics
            .with_object(Some(builtins.object))
            .with_object_prototype(Some(builtins.object_prototype))
            .with_function(Some(builtins.function))
            .with_function_prototype(Some(builtins.function_prototype))
            .with_async_function(Some(builtins.async_function))
            .with_async_function_prototype(Some(builtins.async_function_prototype))
            .with_async_generator_function(Some(builtins.async_generator_function))
            .with_async_generator_function_prototype(Some(
                builtins.async_generator_function_prototype,
            ))
            .with_async_generator_prototype(Some(builtins.async_generator_prototype))
            .with_generator_function(Some(builtins.generator_function))
            .with_generator_function_prototype(Some(builtins.generator_function_prototype))
            .with_generator_prototype(Some(builtins.generator_prototype))
            .with_array(Some(builtins.array))
            .with_array_prototype(Some(array_prototype))
            .with_map(Some(builtins.map))
            .with_map_prototype(Some(map_prototype))
            .with_map_iterator_prototype(Some(map_iterator_prototype))
            .with_set(Some(builtins.set))
            .with_set_prototype(Some(set_prototype))
            .with_set_iterator_prototype(Some(set_iterator_prototype))
            .with_weak_map(Some(builtins.weak_map))
            .with_weak_map_prototype(Some(weak_map_prototype))
            .with_weak_set(Some(builtins.weak_set))
            .with_weak_set_prototype(Some(weak_set_prototype))
            .with_weak_ref(Some(builtins.weak_ref))
            .with_weak_ref_prototype(Some(weak_ref_prototype))
            .with_finalization_registry(Some(builtins.finalization_registry))
            .with_finalization_registry_prototype(Some(finalization_registry_prototype))
            .with_array_buffer(Some(builtins.array_buffer))
            .with_array_buffer_prototype(Some(array_buffer_prototype))
            .with_shared_array_buffer(Some(builtins.shared_array_buffer))
            .with_shared_array_buffer_prototype(Some(shared_array_buffer_prototype))
            .with_data_view(Some(builtins.data_view))
            .with_data_view_prototype(Some(data_view_prototype))
            .with_atomics(Some(builtins.atomics))
            .with_typed_array(Some(builtins.typed_array))
            .with_typed_array_prototype(Some(typed_array_prototype))
            .with_int8_array(Some(builtins.int8_array))
            .with_int8_array_prototype(Some(int8_array_prototype))
            .with_int16_array(Some(builtins.int16_array))
            .with_int16_array_prototype(Some(int16_array_prototype))
            .with_int32_array(Some(builtins.int32_array))
            .with_int32_array_prototype(Some(int32_array_prototype))
            .with_float32_array(Some(builtins.float32_array))
            .with_float32_array_prototype(Some(float32_array_prototype))
            .with_float64_array(Some(builtins.float64_array))
            .with_float64_array_prototype(Some(float64_array_prototype))
            .with_big_int64_array(Some(builtins.big_int64_array))
            .with_big_int64_array_prototype(Some(big_int64_array_prototype))
            .with_big_uint64_array(Some(builtins.big_uint64_array))
            .with_big_uint64_array_prototype(Some(big_uint64_array_prototype))
            .with_uint32_array(Some(builtins.uint32_array))
            .with_uint32_array_prototype(Some(uint32_array_prototype))
            .with_uint16_array(Some(builtins.uint16_array))
            .with_uint16_array_prototype(Some(uint16_array_prototype))
            .with_uint8_clamped_array(Some(builtins.uint8_clamped_array))
            .with_uint8_clamped_array_prototype(Some(uint8_clamped_array_prototype))
            .with_uint8_array(Some(builtins.uint8_array))
            .with_uint8_array_prototype(Some(uint8_array_prototype))
            .with_iterator_prototype(Some(iterator_prototype))
            .with_async_iterator_prototype(Some(builtins.async_iterator_prototype))
            .with_async_from_sync_iterator_prototype(Some(async_from_sync_iterator_prototype))
            .with_array_iterator_prototype(Some(array_iterator_prototype))
            .with_string(Some(builtins.string))
            .with_string_prototype(Some(builtins.string_prototype))
            .with_string_iterator_prototype(Some(string_iterator_prototype))
            .with_regexp(Some(builtins.regexp))
            .with_regexp_prototype(Some(builtins.regexp_prototype))
            .with_date(Some(builtins.date))
            .with_date_prototype(Some(builtins.date_prototype))
            .with_number(Some(builtins.number))
            .with_number_prototype(Some(builtins.number_prototype))
            .with_math(Some(builtins.math))
            .with_bigint(Some(builtins.bigint))
            .with_bigint_prototype(Some(builtins.bigint_prototype))
            .with_boolean(Some(builtins.boolean))
            .with_boolean_prototype(Some(builtins.boolean_prototype))
            .with_symbol(Some(builtins.symbol))
            .with_symbol_prototype(Some(builtins.symbol_prototype))
            .with_json(Some(builtins.json))
            .with_reflect(Some(builtins.reflect))
            .with_proxy(Some(builtins.proxy))
            .with_error(Some(builtins.error))
            .with_error_prototype(Some(builtins.error_prototype))
            .with_eval_error(Some(builtins.eval_error))
            .with_eval_error_prototype(Some(builtins.eval_error_prototype))
            .with_range_error(Some(builtins.range_error))
            .with_range_error_prototype(Some(builtins.range_error_prototype))
            .with_reference_error(Some(builtins.reference_error))
            .with_reference_error_prototype(Some(builtins.reference_error_prototype))
            .with_syntax_error(Some(builtins.syntax_error))
            .with_syntax_error_prototype(Some(builtins.syntax_error_prototype))
            .with_type_error(Some(builtins.type_error))
            .with_type_error_prototype(Some(builtins.type_error_prototype))
            .with_uri_error(Some(builtins.uri_error))
            .with_uri_error_prototype(Some(builtins.uri_error_prototype))
            .with_aggregate_error(Some(builtins.aggregate_error))
            .with_aggregate_error_prototype(Some(builtins.aggregate_error_prototype))
            .with_suppressed_error(Some(builtins.suppressed_error))
            .with_suppressed_error_prototype(Some(builtins.suppressed_error_prototype))
            .with_promise(Some(builtins.promise))
            .with_promise_prototype(Some(builtins.promise_prototype))
            .with_disposable_stack(Some(builtins.disposable_stack))
            .with_disposable_stack_prototype(Some(builtins.disposable_stack_prototype))
            .with_async_disposable_stack(Some(builtins.async_disposable_stack))
            .with_async_disposable_stack_prototype(Some(builtins.async_disposable_stack_prototype));
        if !agent.set_realm_intrinsics(realm, updated_intrinsics) {
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

        let bootstrap_atoms = agent.bootstrap_atoms();
        let empty_string = Value::from_string_ref(agent.alloc_runtime_string(
            "",
            Some(WellKnownAtom::Empty.id()),
            AllocationLifetime::Default,
        ));
        let boolean_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Boolean",
            Some(bootstrap_atoms.boolean()),
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
        let number_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Number",
            Some(bootstrap_atoms.number()),
            AllocationLifetime::Default,
        ));
        let array_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Array",
            Some(bootstrap_atoms.array()),
            AllocationLifetime::Default,
        ));
        let map_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Map",
            Some(bootstrap_atoms.map()),
            AllocationLifetime::Default,
        ));
        let set_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Set",
            Some(bootstrap_atoms.set()),
            AllocationLifetime::Default,
        ));
        let weak_map_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "WeakMap",
            Some(bootstrap_atoms.weak_map()),
            AllocationLifetime::Default,
        ));
        let weak_set_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "WeakSet",
            Some(bootstrap_atoms.weak_set()),
            AllocationLifetime::Default,
        ));
        let weak_ref_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "WeakRef",
            Some(bootstrap_atoms.weak_ref()),
            AllocationLifetime::Default,
        ));
        let finalization_registry_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "FinalizationRegistry",
            Some(bootstrap_atoms.finalization_registry()),
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
        let generator_function_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "GeneratorFunction",
            None,
            AllocationLifetime::Default,
        ));
        let async_function_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "AsyncFunction",
            None,
            AllocationLifetime::Default,
        ));
        let async_generator_function_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "AsyncGeneratorFunction",
            None,
            AllocationLifetime::Default,
        ));
        let generator_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Generator",
            None,
            AllocationLifetime::Default,
        ));
        let async_generator_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "AsyncGenerator",
            None,
            AllocationLifetime::Default,
        ));
        let async_iterator_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "AsyncIterator",
            None,
            AllocationLifetime::Default,
        ));
        let string_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "String",
            Some(bootstrap_atoms.string()),
            AllocationLifetime::Default,
        ));
        let array_iterator_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Array Iterator",
            None,
            AllocationLifetime::Default,
        ));
        let map_iterator_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Map Iterator",
            None,
            AllocationLifetime::Default,
        ));
        let set_iterator_tag = Value::from_string_ref(agent.alloc_runtime_string(
            "Set Iterator",
            None,
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
        let caller_atom = agent.atoms_mut().intern_collectible("caller");
        let escape_atom = agent.atoms_mut().intern_collectible("escape");
        let epsilon_atom = agent.atoms_mut().intern_collectible("EPSILON");
        let e_atom = agent.atoms_mut().intern_collectible("E");
        let floor_atom = agent.atoms_mut().intern_collectible("floor");
        let global_atom = agent.atoms_mut().intern_collectible("global");
        let get_timezone_offset_atom = agent.atoms_mut().intern_collectible("getTimezoneOffset");
        let is_atom = agent.atoms_mut().intern_collectible("is");
        let is_finite_atom = agent.atoms_mut().intern_collectible("isFinite");
        let is_integer_atom = agent.atoms_mut().intern_collectible("isInteger");
        let is_nan_atom = agent.atoms_mut().intern_collectible("isNaN");
        let is_safe_integer_atom = agent.atoms_mut().intern_collectible("isSafeInteger");
        let ignore_case_atom = agent.atoms_mut().intern_collectible("ignoreCase");
        let last_index_of_atom = agent.atoms_mut().intern_collectible("lastIndexOf");
        let match_atom = agent.atoms_mut().intern_collectible("match");
        let multiline_atom = agent.atoms_mut().intern_collectible("multiline");
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
        let pi_atom = agent.atoms_mut().intern_collectible("PI");
        let positive_infinity_atom = agent.atoms_mut().intern_collectible("POSITIVE_INFINITY");
        let pow_atom = agent.atoms_mut().intern_collectible("pow");
        let repeat_atom = agent.atoms_mut().intern_collectible("repeat");
        let replace_atom = agent.atoms_mut().intern_collectible("replace");
        let round_atom = agent.atoms_mut().intern_collectible("round");
        let sign_atom = agent.atoms_mut().intern_collectible("sign");
        let split_atom = agent.atoms_mut().intern_collectible("split");
        let starts_with_atom = agent.atoms_mut().intern_collectible("startsWith");
        let sticky_atom = agent.atoms_mut().intern_collectible("sticky");
        let substring_atom = agent.atoms_mut().intern_collectible("substring");
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
        let is_array_atom = agent.atoms_mut().intern_collectible("isArray");
        let has_own_atom = agent.atoms_mut().intern_collectible("hasOwn");
        let join_atom = agent.atoms_mut().intern_collectible("join");
        let keys_atom = agent.atoms_mut().intern_collectible("keys");
        let map_atom = agent.atoms_mut().intern_collectible("map");
        let next_atom = agent.atoms_mut().intern_collectible("next");
        let of_atom = agent.atoms_mut().intern_collectible("of");
        let reduce_atom = agent.atoms_mut().intern_collectible("reduce");
        let reduce_right_atom = agent.atoms_mut().intern_collectible("reduceRight");
        let reverse_atom = agent.atoms_mut().intern_collectible("reverse");
        let some_atom = agent.atoms_mut().intern_collectible("some");
        let throw_atom = agent.atoms_mut().intern_collectible("throw");
        let at_atom = agent.atoms_mut().intern_collectible("at");
        let slice_atom = agent.atoms_mut().intern_collectible("slice");
        let buffer_atom = agent.atoms_mut().intern_collectible("buffer");
        let byte_length_atom = agent.atoms_mut().intern_collectible("byteLength");
        let byte_offset_atom = agent.atoms_mut().intern_collectible("byteOffset");
        let bytes_per_element_atom = agent.atoms_mut().intern_collectible("BYTES_PER_ELEMENT");
        let is_view_atom = agent.atoms_mut().intern_collectible("isView");
        let sort_atom = agent.atoms_mut().intern_collectible("sort");
        let splice_atom = agent.atoms_mut().intern_collectible("splice");
        let to_locale_string_atom = agent.atoms_mut().intern_collectible("toLocaleString");
        let to_reversed_atom = agent.atoms_mut().intern_collectible("toReversed");
        let to_sorted_atom = agent.atoms_mut().intern_collectible("toSorted");
        let shift_atom = agent.atoms_mut().intern_collectible("shift");
        let unshift_atom = agent.atoms_mut().intern_collectible("unshift");
        let values_atom = agent.atoms_mut().intern_collectible("values");
        let with_atom = agent.atoms_mut().intern_collectible("with");
        let sqrt1_2_atom = agent.atoms_mut().intern_collectible("SQRT1_2");
        let sqrt2_atom = agent.atoms_mut().intern_collectible("SQRT2");
        let sqrt_atom = agent.atoms_mut().intern_collectible("sqrt");
        let char_at_atom = agent.atoms_mut().intern_collectible("charAt");
        let char_code_at_atom = agent.atoms_mut().intern_collectible("charCodeAt");
        let dot_all_atom = agent.atoms_mut().intern_collectible("dotAll");
        let exec_atom = agent.atoms_mut().intern_collectible("exec");
        let get_own_property_names_atom =
            agent.atoms_mut().intern_collectible("getOwnPropertyNames");
        let get_own_property_descriptors_atom = agent
            .atoms_mut()
            .intern_collectible("getOwnPropertyDescriptors");
        let get_own_property_symbols_atom = agent
            .atoms_mut()
            .intern_collectible("getOwnPropertySymbols");
        let define_properties_atom = agent.atoms_mut().intern_collectible("defineProperties");
        let get_float32_atom = agent.atoms_mut().intern_collectible("getFloat32");
        let get_float64_atom = agent.atoms_mut().intern_collectible("getFloat64");
        let get_int16_atom = agent.atoms_mut().intern_collectible("getInt16");
        let get_int32_atom = agent.atoms_mut().intern_collectible("getInt32");
        let get_int8_atom = agent.atoms_mut().intern_collectible("getInt8");
        let get_atom = agent.atoms_mut().intern_collectible("get");
        let get_uint16_atom = agent.atoms_mut().intern_collectible("getUint16");
        let get_uint32_atom = agent.atoms_mut().intern_collectible("getUint32");
        let get_uint8_atom = agent.atoms_mut().intern_collectible("getUint8");
        let has_atom = agent.atoms_mut().intern_collectible("has");
        let has_indices_atom = bootstrap_atoms.has_indices();
        let add_atom = agent.atoms_mut().intern_collectible("add");
        let and_atom = agent.atoms_mut().intern_collectible("and");
        let clear_atom = agent.atoms_mut().intern_collectible("clear");
        let compare_exchange_atom = agent.atoms_mut().intern_collectible("compareExchange");
        let delete_atom = agent.atoms_mut().intern_collectible("delete");
        let deref_atom = agent.atoms_mut().intern_collectible("deref");
        let exchange_atom = agent.atoms_mut().intern_collectible("exchange");
        let is_lock_free_atom = agent.atoms_mut().intern_collectible("isLockFree");
        let load_atom = agent.atoms_mut().intern_collectible("load");
        let notify_atom = agent.atoms_mut().intern_collectible("notify");
        let now_atom = agent.atoms_mut().intern_collectible("now");
        let or_atom = agent.atoms_mut().intern_collectible("or");
        let register_atom = agent.atoms_mut().intern_collectible("register");
        let set_atom = agent.atoms_mut().intern_collectible("set");
        let set_float32_atom = agent.atoms_mut().intern_collectible("setFloat32");
        let set_float64_atom = agent.atoms_mut().intern_collectible("setFloat64");
        let set_int16_atom = agent.atoms_mut().intern_collectible("setInt16");
        let set_int32_atom = agent.atoms_mut().intern_collectible("setInt32");
        let set_int8_atom = agent.atoms_mut().intern_collectible("setInt8");
        let set_uint16_atom = agent.atoms_mut().intern_collectible("setUint16");
        let set_uint32_atom = agent.atoms_mut().intern_collectible("setUint32");
        let set_uint8_atom = agent.atoms_mut().intern_collectible("setUint8");
        let size_atom = agent.atoms_mut().intern_collectible("size");
        let store_atom = agent.atoms_mut().intern_collectible("store");
        let sub_atom = agent.atoms_mut().intern_collectible("sub");
        let subarray_atom = agent.atoms_mut().intern_collectible("subarray");
        let unregister_atom = agent.atoms_mut().intern_collectible("unregister");
        let test_atom = agent.atoms_mut().intern_collectible("test");
        let trunc_atom = agent.atoms_mut().intern_collectible("trunc");
        let to_exponential_atom = agent.atoms_mut().intern_collectible("toExponential");
        let unicode_atom = agent.atoms_mut().intern_collectible("unicode");
        let arguments_atom = agent.atoms_mut().intern_collectible("arguments");
        let wait_atom = agent.atoms_mut().intern_collectible("wait");
        let wait_async_atom = agent.atoms_mut().intern_collectible("waitAsync");
        let xor_atom = agent.atoms_mut().intern_collectible("xor");
        let object_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bootstrap_atoms.create()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_create_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bootstrap_atoms.get_prototype_of()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_get_prototype_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bootstrap_atoms.set_prototype_of()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_set_prototype_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bootstrap_atoms.get_own_property_descriptor()),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    js3_object_get_own_property_descriptor_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_own_property_descriptors_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    js3_object_get_own_property_descriptors_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_own_property_names_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    js3_object_get_own_property_names_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_own_property_symbols_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    js3_object_get_own_property_symbols_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(define_properties_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_define_properties_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bootstrap_atoms.define_property()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_define_property_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bootstrap_atoms.prevent_extensions()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_prevent_extensions_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bootstrap_atoms.is_extensible()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_is_extensible_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(is_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_is_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bootstrap_atoms.seal()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_seal_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bootstrap_atoms.freeze()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_freeze_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bootstrap_atoms.is_sealed()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_is_sealed_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bootstrap_atoms.is_frozen()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_is_frozen_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(keys_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_keys_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(entries_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_entries_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(values_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(has_own_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_has_own_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
        ];
        let object_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.object)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_locale_string_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_to_locale_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::toString.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_to_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::valueOf.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_value_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bootstrap_atoms.has_own_property()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_has_own_property_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bootstrap_atoms.is_prototype_of()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_object_is_prototype_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bootstrap_atoms.property_is_enumerable()),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    js3_object_property_is_enumerable_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
        ];
        let function_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.function)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::call.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_function_call_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::apply.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_function_apply_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::bind.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_function_bind_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::toString.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_function_to_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(caller_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_internal_throw_type_error_builtin()),
                    set: Some(js3_internal_throw_type_error_builtin()),
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(arguments_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_internal_throw_type_error_builtin()),
                    set: Some(js3_internal_throw_type_error_builtin()),
                },
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let generator_function_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.generator_function)),
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::prototype.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(
                    builtins.generator_prototype,
                )),
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(generator_function_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let async_function_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.async_function)),
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(async_function_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let async_generator_function_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(
                    builtins.async_generator_function,
                )),
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::prototype.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(
                    builtins.async_generator_prototype,
                )),
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(async_generator_function_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let generator_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(
                    builtins.generator_function_prototype,
                )),
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(next_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_generator_next_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::r#return.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_generator_return_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(throw_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_generator_throw_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_iterator_prototype_iterator_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(generator_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let async_generator_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(
                    builtins.async_generator_function_prototype,
                )),
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(next_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_async_generator_next_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::r#return.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_async_generator_return_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(throw_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_async_generator_throw_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::AsyncIterator),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(
                    builtins.async_iterator_method,
                )),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(async_generator_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let array_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(from_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_from_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(is_array_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_is_array_builtin()),
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
        let map_descriptors = [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Species),
            BuiltinPropertyValueSpec::Accessor {
                get: Some(js3_array_species_getter_builtin()),
                set: None,
            },
            BuiltinAttributes::new(false, false, true),
        )];
        let map_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.map)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_map_get_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_map_set_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(has_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_map_has_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(delete_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_map_delete_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(clear_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_map_clear_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(entries_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_map_entries_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(values_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_map_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(keys_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_map_keys_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(for_each_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_map_for_each_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(size_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_map_size_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_map_entries_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(map_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let set_descriptors = [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Species),
            BuiltinPropertyValueSpec::Accessor {
                get: Some(js3_array_species_getter_builtin()),
                set: None,
            },
            BuiltinAttributes::new(false, false, true),
        )];
        let set_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.set)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(add_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_set_add_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(has_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_set_has_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(delete_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_set_delete_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(clear_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_set_clear_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(entries_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_set_entries_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(values_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_set_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(keys_atom),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.set_values)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(for_each_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_set_for_each_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(size_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(js3_set_size_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_set_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(set_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let weak_map_descriptors: [BuiltinPropertyDescriptor; 0] = [];
        let weak_map_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.weak_map)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_weak_map_get_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_weak_map_set_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(has_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_weak_map_has_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(delete_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_weak_map_delete_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(weak_map_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let weak_set_descriptors: [BuiltinPropertyDescriptor; 0] = [];
        let weak_set_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.weak_set)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(add_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_weak_set_add_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(has_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_weak_set_has_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(delete_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_weak_set_delete_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(weak_set_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let weak_ref_descriptors: [BuiltinPropertyDescriptor; 0] = [];
        let weak_ref_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.weak_ref)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(deref_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_weak_ref_deref_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(weak_ref_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let finalization_registry_descriptors: [BuiltinPropertyDescriptor; 0] = [];
        let finalization_registry_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(
                    builtins.finalization_registry,
                )),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(register_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    js3_finalization_registry_register_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(unregister_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    js3_finalization_registry_unregister_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(finalization_registry_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
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
        let array_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.array)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::toString.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_to_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::length.id()),
                BuiltinPropertyValueSpec::Data(Value::from_smi(0)),
                BuiltinAttributes::new(true, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(concat_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_concat_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(copy_within_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_copy_within_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(fill_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_fill_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(join_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_join_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(shift_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_shift_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(unshift_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_unshift_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(filter_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_filter_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(for_each_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_for_each_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(map_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_map_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(reverse_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_reverse_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(slice_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(sort_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_sort_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(splice_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_splice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_locale_string_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_to_locale_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(values_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(keys_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_keys_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(entries_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_entries_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(array_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let iterator_prototype_descriptors = [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_iterator_prototype_iterator_builtin()),
            BuiltinAttributes::new(true, false, true),
        )];
        let async_iterator_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::AsyncIterator),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(
                    builtins.async_iterator_method,
                )),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(async_iterator_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let array_iterator_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(next_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_array_iterator_next_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(array_iterator_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let map_iterator_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(next_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_map_iterator_next_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(map_iterator_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let set_iterator_prototype_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(next_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_set_iterator_next_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(set_iterator_tag),
                BuiltinAttributes::new(false, false, true),
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
                BuiltinPropertyKeySpec::from_atom(match_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_string_match_builtin()),
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
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(string_tag),
                BuiltinAttributes::new(false, false, true),
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
        let regexp_descriptors = [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(escape_atom),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_regexp_escape_builtin()),
            BuiltinAttributes::new(true, false, true),
        )];
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
        let date_descriptors = [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(now_atom),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_date_now_builtin()),
            BuiltinAttributes::new(true, false, true),
        )];
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
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::valueOf.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_value_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(get_timezone_offset_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_date_get_timezone_offset_builtin()),
                BuiltinAttributes::new(true, false, true),
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
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::toString.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_number_to_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::valueOf.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_number_value_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(number_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ];
        let math_descriptors = [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(abs_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_abs_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(floor_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_floor_builtin()),
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
                BuiltinPropertyKeySpec::from_atom(sqrt_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(js3_math_sqrt_builtin()),
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
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(boolean_tag),
                BuiltinAttributes::new(false, false, true),
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
                BuiltinPropertyKeySpec::from_atom(agent.bootstrap_atoms().species()),
                BuiltinPropertyValueSpec::Data(Value::from_symbol_ref(
                    agent.well_known_symbol(WellKnownSymbolId::Species)?,
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
        let parse_atom = agent.atoms_mut().intern_collectible("parse");
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
                BuiltinPropertyValueSpec::BuiltinFunction(lyng_js_types::js3_reflect_apply_builtin()),
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
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Object),
                &object_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::ObjectPrototype),
                &object_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::FunctionPrototype),
                &function_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::AsyncFunctionPrototype),
                &async_function_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::AsyncGeneratorFunctionPrototype),
                &async_generator_function_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::GeneratorFunctionPrototype),
                &generator_function_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::GeneratorPrototype),
                &generator_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::AsyncGeneratorPrototype),
                &async_generator_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Array),
                &array_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Map),
                &map_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::MapPrototype),
                &map_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Set),
                &set_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::SetPrototype),
                &set_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::WeakMap),
                &weak_map_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::WeakMapPrototype),
                &weak_map_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::WeakSet),
                &weak_set_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::WeakSetPrototype),
                &weak_set_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::WeakRef),
                &weak_ref_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::WeakRefPrototype),
                &weak_ref_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::FinalizationRegistry),
                &finalization_registry_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::FinalizationRegistryPrototype),
                &finalization_registry_prototype_descriptors,
            ),
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
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::ArrayPrototype),
                &array_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::IteratorPrototype),
                &iterator_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::AsyncIteratorPrototype),
                &async_iterator_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::ArrayIteratorPrototype),
                &array_iterator_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::MapIteratorPrototype),
                &map_iterator_prototype_descriptors,
            ),
            BuiltinDescriptorTable::new(
                BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::SetIteratorPrototype),
                &set_iterator_prototype_descriptors,
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
        if install_descriptor_tables(agent, self, realm, &tables).is_err() {
            self.public.remove(&realm);
            return None;
        }

        Some(builtins)
    }
}

/// Compatibility metadata for the public core builtin namespace.
#[inline]
pub fn public_builtin_metadata(entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
    if entry == js3_object_builtin() {
        return Some(BuiltinEntryMetadata::new("Object", 1, true, true));
    }
    if entry == js3_object_create_builtin() {
        return Some(BuiltinEntryMetadata::new("create", 2, false, false));
    }
    if entry == js3_object_get_prototype_of_builtin() {
        return Some(BuiltinEntryMetadata::new("getPrototypeOf", 1, false, false));
    }
    if entry == js3_object_set_prototype_of_builtin() {
        return Some(BuiltinEntryMetadata::new("setPrototypeOf", 2, false, false));
    }
    if entry == js3_object_get_own_property_descriptor_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "getOwnPropertyDescriptor",
            2,
            false,
            false,
        ));
    }
    if entry == js3_object_get_own_property_descriptors_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "getOwnPropertyDescriptors",
            1,
            false,
            false,
        ));
    }
    if entry == js3_object_get_own_property_names_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "getOwnPropertyNames",
            1,
            false,
            false,
        ));
    }
    if entry == js3_object_get_own_property_symbols_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "getOwnPropertySymbols",
            1,
            false,
            false,
        ));
    }
    if entry == js3_object_define_properties_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "defineProperties",
            2,
            false,
            false,
        ));
    }
    if entry == js3_object_define_property_builtin() {
        return Some(BuiltinEntryMetadata::new("defineProperty", 3, false, false));
    }
    if entry == js3_object_prevent_extensions_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "preventExtensions",
            1,
            false,
            false,
        ));
    }
    if entry == js3_object_is_extensible_builtin() {
        return Some(BuiltinEntryMetadata::new("isExtensible", 1, false, false));
    }
    if entry == js3_object_is_builtin() {
        return Some(BuiltinEntryMetadata::new("is", 2, false, false));
    }
    if entry == js3_object_seal_builtin() {
        return Some(BuiltinEntryMetadata::new("seal", 1, false, false));
    }
    if entry == js3_object_freeze_builtin() {
        return Some(BuiltinEntryMetadata::new("freeze", 1, false, false));
    }
    if entry == js3_object_is_sealed_builtin() {
        return Some(BuiltinEntryMetadata::new("isSealed", 1, false, false));
    }
    if entry == js3_object_is_frozen_builtin() {
        return Some(BuiltinEntryMetadata::new("isFrozen", 1, false, false));
    }
    if entry == js3_object_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == js3_object_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == js3_object_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == js3_object_has_own_property_builtin() {
        return Some(BuiltinEntryMetadata::new("hasOwnProperty", 1, false, false));
    }
    if entry == js3_object_is_prototype_of_builtin() {
        return Some(BuiltinEntryMetadata::new("isPrototypeOf", 1, false, false));
    }
    if entry == js3_object_property_is_enumerable_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "propertyIsEnumerable",
            1,
            false,
            false,
        ));
    }
    if entry == js3_object_keys_builtin() {
        return Some(BuiltinEntryMetadata::new("keys", 1, false, false));
    }
    if entry == js3_object_entries_builtin() {
        return Some(BuiltinEntryMetadata::new("entries", 1, false, false));
    }
    if entry == js3_object_values_builtin() {
        return Some(BuiltinEntryMetadata::new("values", 1, false, false));
    }
    if entry == js3_object_has_own_builtin() {
        return Some(BuiltinEntryMetadata::new("hasOwn", 2, false, false));
    }
    if entry == js3_function_builtin() {
        return Some(BuiltinEntryMetadata::new("Function", 1, true, true));
    }
    if entry == js3_function_prototype_builtin() {
        return Some(BuiltinEntryMetadata::new("", 0, false, false));
    }
    if entry == js3_function_call_builtin() {
        return Some(BuiltinEntryMetadata::new("call", 1, false, false));
    }
    if entry == js3_function_apply_builtin() {
        return Some(BuiltinEntryMetadata::new("apply", 2, false, false));
    }
    if entry == js3_function_bind_builtin() {
        return Some(BuiltinEntryMetadata::new("bind", 1, false, false));
    }
    if entry == js3_function_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == js3_async_function_builtin() {
        return Some(BuiltinEntryMetadata::new("AsyncFunction", 1, true, true));
    }
    if entry == js3_async_generator_function_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "AsyncGeneratorFunction",
            1,
            true,
            true,
        ));
    }
    if entry == js3_async_generator_next_builtin() {
        return Some(BuiltinEntryMetadata::new("next", 1, false, false));
    }
    if entry == js3_async_generator_return_builtin() {
        return Some(BuiltinEntryMetadata::new("return", 1, false, false));
    }
    if entry == js3_async_generator_throw_builtin() {
        return Some(BuiltinEntryMetadata::new("throw", 1, false, false));
    }
    if entry == js3_generator_function_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "GeneratorFunction",
            1,
            true,
            true,
        ));
    }
    if entry == js3_generator_next_builtin() {
        return Some(BuiltinEntryMetadata::new("next", 1, false, false));
    }
    if entry == js3_generator_return_builtin() {
        return Some(BuiltinEntryMetadata::new("return", 1, false, false));
    }
    if entry == js3_generator_throw_builtin() {
        return Some(BuiltinEntryMetadata::new("throw", 1, false, false));
    }
    if entry == js3_array_builtin() {
        return Some(BuiltinEntryMetadata::new("Array", 1, true, true));
    }
    if entry == js3_array_from_builtin() {
        return Some(BuiltinEntryMetadata::new("from", 1, false, false));
    }
    if entry == js3_map_builtin() {
        return Some(BuiltinEntryMetadata::new("Map", 0, true, true));
    }
    if entry == js3_set_builtin() {
        return Some(BuiltinEntryMetadata::new("Set", 0, true, true));
    }
    if entry == js3_weak_map_builtin() {
        return Some(BuiltinEntryMetadata::new("WeakMap", 0, true, true));
    }
    if entry == js3_weak_set_builtin() {
        return Some(BuiltinEntryMetadata::new("WeakSet", 0, true, true));
    }
    if entry == js3_weak_ref_builtin() {
        return Some(BuiltinEntryMetadata::new("WeakRef", 1, true, true));
    }
    if entry == js3_finalization_registry_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "FinalizationRegistry",
            1,
            true,
            true,
        ));
    }
    if entry == js3_array_buffer_builtin() {
        return Some(BuiltinEntryMetadata::new("ArrayBuffer", 1, true, true));
    }
    if entry == js3_array_buffer_is_view_builtin() {
        return Some(BuiltinEntryMetadata::new("isView", 1, false, false));
    }
    if entry == js3_shared_array_buffer_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "SharedArrayBuffer",
            1,
            true,
            true,
        ));
    }
    if entry == js3_data_view_builtin() {
        return Some(BuiltinEntryMetadata::new("DataView", 1, true, true));
    }
    if entry == js3_typed_array_builtin() {
        return Some(BuiltinEntryMetadata::new("TypedArray", 0, true, true));
    }
    if entry == js3_typed_array_from_builtin() {
        return Some(BuiltinEntryMetadata::new("from", 1, false, false));
    }
    if entry == js3_typed_array_of_builtin() {
        return Some(BuiltinEntryMetadata::new("of", 0, false, false));
    }
    if entry == js3_typed_array_every_builtin() {
        return Some(BuiltinEntryMetadata::new("every", 1, false, false));
    }
    if entry == js3_typed_array_some_builtin() {
        return Some(BuiltinEntryMetadata::new("some", 1, false, false));
    }
    if entry == js3_typed_array_find_builtin() {
        return Some(BuiltinEntryMetadata::new("find", 1, false, false));
    }
    if entry == js3_typed_array_find_index_builtin() {
        return Some(BuiltinEntryMetadata::new("findIndex", 1, false, false));
    }
    if entry == js3_typed_array_find_last_builtin() {
        return Some(BuiltinEntryMetadata::new("findLast", 1, false, false));
    }
    if entry == js3_typed_array_find_last_index_builtin() {
        return Some(BuiltinEntryMetadata::new("findLastIndex", 1, false, false));
    }
    if entry == js3_typed_array_fill_builtin() {
        return Some(BuiltinEntryMetadata::new("fill", 1, false, false));
    }
    if entry == js3_typed_array_copy_within_builtin() {
        return Some(BuiltinEntryMetadata::new("copyWithin", 2, false, false));
    }
    if entry == js3_typed_array_filter_builtin() {
        return Some(BuiltinEntryMetadata::new("filter", 1, false, false));
    }
    if entry == js3_typed_array_for_each_builtin() {
        return Some(BuiltinEntryMetadata::new("forEach", 1, false, false));
    }
    if entry == js3_typed_array_includes_builtin() {
        return Some(BuiltinEntryMetadata::new("includes", 1, false, false));
    }
    if entry == js3_typed_array_index_of_builtin() {
        return Some(BuiltinEntryMetadata::new("indexOf", 1, false, false));
    }
    if entry == js3_typed_array_join_builtin() {
        return Some(BuiltinEntryMetadata::new("join", 1, false, false));
    }
    if entry == js3_typed_array_last_index_of_builtin() {
        return Some(BuiltinEntryMetadata::new("lastIndexOf", 1, false, false));
    }
    if entry == js3_typed_array_map_builtin() {
        return Some(BuiltinEntryMetadata::new("map", 1, false, false));
    }
    if entry == js3_typed_array_reduce_builtin() {
        return Some(BuiltinEntryMetadata::new("reduce", 1, false, false));
    }
    if entry == js3_typed_array_reduce_right_builtin() {
        return Some(BuiltinEntryMetadata::new("reduceRight", 1, false, false));
    }
    if entry == js3_typed_array_reverse_builtin() {
        return Some(BuiltinEntryMetadata::new("reverse", 0, false, false));
    }
    if entry == js3_typed_array_sort_builtin() {
        return Some(BuiltinEntryMetadata::new("sort", 1, false, false));
    }
    if entry == js3_typed_array_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == js3_typed_array_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == js3_typed_array_to_reversed_builtin() {
        return Some(BuiltinEntryMetadata::new("toReversed", 0, false, false));
    }
    if entry == js3_typed_array_to_sorted_builtin() {
        return Some(BuiltinEntryMetadata::new("toSorted", 1, false, false));
    }
    if entry == js3_typed_array_with_builtin() {
        return Some(BuiltinEntryMetadata::new("with", 2, false, false));
    }
    if entry == js3_int8_array_builtin() {
        return Some(BuiltinEntryMetadata::new("Int8Array", 3, true, true));
    }
    if entry == js3_int16_array_builtin() {
        return Some(BuiltinEntryMetadata::new("Int16Array", 3, true, true));
    }
    if entry == js3_int32_array_builtin() {
        return Some(BuiltinEntryMetadata::new("Int32Array", 3, true, true));
    }
    if entry == js3_float32_array_builtin() {
        return Some(BuiltinEntryMetadata::new("Float32Array", 3, true, true));
    }
    if entry == js3_float64_array_builtin() {
        return Some(BuiltinEntryMetadata::new("Float64Array", 3, true, true));
    }
    if entry == js3_big_int64_array_builtin() {
        return Some(BuiltinEntryMetadata::new("BigInt64Array", 3, true, true));
    }
    if entry == js3_big_uint64_array_builtin() {
        return Some(BuiltinEntryMetadata::new("BigUint64Array", 3, true, true));
    }
    if entry == js3_uint32_array_builtin() {
        return Some(BuiltinEntryMetadata::new("Uint32Array", 3, true, true));
    }
    if entry == js3_uint16_array_builtin() {
        return Some(BuiltinEntryMetadata::new("Uint16Array", 3, true, true));
    }
    if entry == js3_uint8_clamped_array_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "Uint8ClampedArray",
            3,
            true,
            true,
        ));
    }
    if entry == js3_uint8_array_builtin() {
        return Some(BuiltinEntryMetadata::new("Uint8Array", 3, true, true));
    }
    if entry == js3_array_is_array_builtin() {
        return Some(BuiltinEntryMetadata::new("isArray", 1, false, false));
    }
    if entry == js3_array_concat_builtin() {
        return Some(BuiltinEntryMetadata::new("concat", 1, false, false));
    }
    if entry == js3_array_copy_within_builtin() {
        return Some(BuiltinEntryMetadata::new("copyWithin", 2, false, false));
    }
    if entry == js3_array_fill_builtin() {
        return Some(BuiltinEntryMetadata::new("fill", 1, false, false));
    }
    if entry == js3_array_join_builtin() {
        return Some(BuiltinEntryMetadata::new("join", 1, false, false));
    }
    if entry == js3_array_shift_builtin() {
        return Some(BuiltinEntryMetadata::new("shift", 0, false, false));
    }
    if entry == js3_array_unshift_builtin() {
        return Some(BuiltinEntryMetadata::new("unshift", 1, false, false));
    }
    if entry == js3_array_filter_builtin() {
        return Some(BuiltinEntryMetadata::new("filter", 1, false, false));
    }
    if entry == js3_array_for_each_builtin() {
        return Some(BuiltinEntryMetadata::new("forEach", 1, false, false));
    }
    if entry == js3_array_map_builtin() {
        return Some(BuiltinEntryMetadata::new("map", 1, false, false));
    }
    if entry == js3_array_reverse_builtin() {
        return Some(BuiltinEntryMetadata::new("reverse", 0, false, false));
    }
    if entry == js3_array_slice_builtin() {
        return Some(BuiltinEntryMetadata::new("slice", 2, false, false));
    }
    if entry == js3_array_sort_builtin() {
        return Some(BuiltinEntryMetadata::new("sort", 1, false, false));
    }
    if entry == js3_array_splice_builtin() {
        return Some(BuiltinEntryMetadata::new("splice", 2, false, false));
    }
    if entry == js3_array_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == js3_array_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == js3_array_values_builtin() {
        return Some(BuiltinEntryMetadata::new("values", 0, false, false));
    }
    if entry == js3_array_keys_builtin() {
        return Some(BuiltinEntryMetadata::new("keys", 0, false, false));
    }
    if entry == js3_array_entries_builtin() {
        return Some(BuiltinEntryMetadata::new("entries", 0, false, false));
    }
    if entry == js3_map_get_builtin() {
        return Some(BuiltinEntryMetadata::new("get", 1, false, false));
    }
    if entry == js3_map_set_builtin() {
        return Some(BuiltinEntryMetadata::new("set", 2, false, false));
    }
    if entry == js3_map_has_builtin() {
        return Some(BuiltinEntryMetadata::new("has", 1, false, false));
    }
    if entry == js3_map_delete_builtin() {
        return Some(BuiltinEntryMetadata::new("delete", 1, false, false));
    }
    if entry == js3_map_clear_builtin() {
        return Some(BuiltinEntryMetadata::new("clear", 0, false, false));
    }
    if entry == js3_map_entries_builtin() {
        return Some(BuiltinEntryMetadata::new("entries", 0, false, false));
    }
    if entry == js3_map_values_builtin() {
        return Some(BuiltinEntryMetadata::new("values", 0, false, false));
    }
    if entry == js3_map_keys_builtin() {
        return Some(BuiltinEntryMetadata::new("keys", 0, false, false));
    }
    if entry == js3_map_for_each_builtin() {
        return Some(BuiltinEntryMetadata::new("forEach", 1, false, false));
    }
    if entry == js3_map_size_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get size", 0, false, false));
    }
    if entry == js3_set_add_builtin() {
        return Some(BuiltinEntryMetadata::new("add", 1, false, false));
    }
    if entry == js3_set_has_builtin() {
        return Some(BuiltinEntryMetadata::new("has", 1, false, false));
    }
    if entry == js3_set_delete_builtin() {
        return Some(BuiltinEntryMetadata::new("delete", 1, false, false));
    }
    if entry == js3_set_clear_builtin() {
        return Some(BuiltinEntryMetadata::new("clear", 0, false, false));
    }
    if entry == js3_set_entries_builtin() {
        return Some(BuiltinEntryMetadata::new("entries", 0, false, false));
    }
    if entry == js3_set_values_builtin() {
        return Some(BuiltinEntryMetadata::new("values", 0, false, false));
    }
    if entry == js3_set_keys_builtin() {
        return Some(BuiltinEntryMetadata::new("keys", 0, false, false));
    }
    if entry == js3_set_for_each_builtin() {
        return Some(BuiltinEntryMetadata::new("forEach", 1, false, false));
    }
    if entry == js3_set_size_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get size", 0, false, false));
    }
    if entry == js3_weak_map_get_builtin() {
        return Some(BuiltinEntryMetadata::new("get", 1, false, false));
    }
    if entry == js3_weak_map_set_builtin() {
        return Some(BuiltinEntryMetadata::new("set", 2, false, false));
    }
    if entry == js3_weak_map_has_builtin() {
        return Some(BuiltinEntryMetadata::new("has", 1, false, false));
    }
    if entry == js3_weak_map_delete_builtin() {
        return Some(BuiltinEntryMetadata::new("delete", 1, false, false));
    }
    if entry == js3_weak_set_add_builtin() {
        return Some(BuiltinEntryMetadata::new("add", 1, false, false));
    }
    if entry == js3_weak_set_has_builtin() {
        return Some(BuiltinEntryMetadata::new("has", 1, false, false));
    }
    if entry == js3_weak_set_delete_builtin() {
        return Some(BuiltinEntryMetadata::new("delete", 1, false, false));
    }
    if entry == js3_weak_ref_deref_builtin() {
        return Some(BuiltinEntryMetadata::new("deref", 0, false, false));
    }
    if entry == js3_finalization_registry_register_builtin() {
        return Some(BuiltinEntryMetadata::new("register", 2, false, false));
    }
    if entry == js3_finalization_registry_unregister_builtin() {
        return Some(BuiltinEntryMetadata::new("unregister", 1, false, false));
    }
    if entry == js3_array_buffer_byte_length_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get byteLength", 0, false, false));
    }
    if entry == js3_array_buffer_slice_builtin() {
        return Some(BuiltinEntryMetadata::new("slice", 2, false, false));
    }
    if entry == js3_shared_array_buffer_byte_length_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get byteLength", 0, false, false));
    }
    if entry == js3_shared_array_buffer_slice_builtin() {
        return Some(BuiltinEntryMetadata::new("slice", 2, false, false));
    }
    if entry == js3_atomics_load_builtin() {
        return Some(BuiltinEntryMetadata::new("load", 2, false, false));
    }
    if entry == js3_atomics_store_builtin() {
        return Some(BuiltinEntryMetadata::new("store", 3, false, false));
    }
    if entry == js3_atomics_add_builtin() {
        return Some(BuiltinEntryMetadata::new("add", 3, false, false));
    }
    if entry == js3_atomics_sub_builtin() {
        return Some(BuiltinEntryMetadata::new("sub", 3, false, false));
    }
    if entry == js3_atomics_and_builtin() {
        return Some(BuiltinEntryMetadata::new("and", 3, false, false));
    }
    if entry == js3_atomics_or_builtin() {
        return Some(BuiltinEntryMetadata::new("or", 3, false, false));
    }
    if entry == js3_atomics_xor_builtin() {
        return Some(BuiltinEntryMetadata::new("xor", 3, false, false));
    }
    if entry == js3_atomics_exchange_builtin() {
        return Some(BuiltinEntryMetadata::new("exchange", 3, false, false));
    }
    if entry == js3_atomics_compare_exchange_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "compareExchange",
            4,
            false,
            false,
        ));
    }
    if entry == js3_atomics_notify_builtin() {
        return Some(BuiltinEntryMetadata::new("notify", 3, false, false));
    }
    if entry == js3_atomics_wait_builtin() {
        return Some(BuiltinEntryMetadata::new("wait", 4, false, false));
    }
    if entry == js3_atomics_wait_async_builtin() {
        return Some(BuiltinEntryMetadata::new("waitAsync", 4, false, false));
    }
    if entry == js3_atomics_is_lock_free_builtin() {
        return Some(BuiltinEntryMetadata::new("isLockFree", 1, false, false));
    }
    if entry == js3_data_view_buffer_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get buffer", 0, false, false));
    }
    if entry == js3_data_view_byte_length_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get byteLength", 0, false, false));
    }
    if entry == js3_data_view_byte_offset_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get byteOffset", 0, false, false));
    }
    if entry == js3_data_view_get_float32_builtin() {
        return Some(BuiltinEntryMetadata::new("getFloat32", 1, false, false));
    }
    if entry == js3_data_view_get_float64_builtin() {
        return Some(BuiltinEntryMetadata::new("getFloat64", 1, false, false));
    }
    if entry == js3_data_view_get_int16_builtin() {
        return Some(BuiltinEntryMetadata::new("getInt16", 1, false, false));
    }
    if entry == js3_data_view_get_int32_builtin() {
        return Some(BuiltinEntryMetadata::new("getInt32", 1, false, false));
    }
    if entry == js3_data_view_get_int8_builtin() {
        return Some(BuiltinEntryMetadata::new("getInt8", 1, false, false));
    }
    if entry == js3_data_view_get_uint16_builtin() {
        return Some(BuiltinEntryMetadata::new("getUint16", 1, false, false));
    }
    if entry == js3_data_view_get_uint32_builtin() {
        return Some(BuiltinEntryMetadata::new("getUint32", 1, false, false));
    }
    if entry == js3_data_view_get_uint8_builtin() {
        return Some(BuiltinEntryMetadata::new("getUint8", 1, false, false));
    }
    if entry == js3_data_view_set_float32_builtin() {
        return Some(BuiltinEntryMetadata::new("setFloat32", 2, false, false));
    }
    if entry == js3_data_view_set_float64_builtin() {
        return Some(BuiltinEntryMetadata::new("setFloat64", 2, false, false));
    }
    if entry == js3_data_view_set_int16_builtin() {
        return Some(BuiltinEntryMetadata::new("setInt16", 2, false, false));
    }
    if entry == js3_data_view_set_int32_builtin() {
        return Some(BuiltinEntryMetadata::new("setInt32", 2, false, false));
    }
    if entry == js3_data_view_set_int8_builtin() {
        return Some(BuiltinEntryMetadata::new("setInt8", 2, false, false));
    }
    if entry == js3_data_view_set_uint16_builtin() {
        return Some(BuiltinEntryMetadata::new("setUint16", 2, false, false));
    }
    if entry == js3_data_view_set_uint32_builtin() {
        return Some(BuiltinEntryMetadata::new("setUint32", 2, false, false));
    }
    if entry == js3_data_view_set_uint8_builtin() {
        return Some(BuiltinEntryMetadata::new("setUint8", 2, false, false));
    }
    if entry == js3_uint8_array_buffer_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get buffer", 0, false, false));
    }
    if entry == js3_uint8_array_byte_length_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get byteLength", 0, false, false));
    }
    if entry == js3_uint8_array_byte_offset_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get byteOffset", 0, false, false));
    }
    if entry == js3_uint8_array_length_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get length", 0, false, false));
    }
    if entry == js3_uint8_array_values_builtin() {
        return Some(BuiltinEntryMetadata::new("values", 0, false, false));
    }
    if entry == js3_uint8_array_keys_builtin() {
        return Some(BuiltinEntryMetadata::new("keys", 0, false, false));
    }
    if entry == js3_uint8_array_entries_builtin() {
        return Some(BuiltinEntryMetadata::new("entries", 0, false, false));
    }
    if entry == js3_uint8_array_set_builtin() {
        return Some(BuiltinEntryMetadata::new("set", 1, false, false));
    }
    if entry == js3_uint8_array_slice_builtin() {
        return Some(BuiltinEntryMetadata::new("slice", 2, false, false));
    }
    if entry == js3_uint8_array_subarray_builtin() {
        return Some(BuiltinEntryMetadata::new("subarray", 2, false, false));
    }
    if entry == js3_typed_array_at_builtin() {
        return Some(BuiltinEntryMetadata::new("at", 1, false, false));
    }
    if entry == js3_typed_array_to_string_tag_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get [Symbol.toStringTag]",
            0,
            false,
            false,
        ));
    }
    if entry == js3_json_parse_builtin() {
        return Some(BuiltinEntryMetadata::new("parse", 2, false, false));
    }
    if entry == js3_json_stringify_builtin() {
        return Some(BuiltinEntryMetadata::new("stringify", 3, false, false));
    }
    if entry == js3_json_raw_json_builtin() {
        return Some(BuiltinEntryMetadata::new("rawJSON", 1, false, false));
    }
    if entry == js3_json_is_raw_json_builtin() {
        return Some(BuiltinEntryMetadata::new("isRawJSON", 1, false, false));
    }
    if entry == lyng_js_types::js3_reflect_apply_builtin() {
        return Some(BuiltinEntryMetadata::new("apply", 3, false, false));
    }
    if entry == lyng_js_types::js3_reflect_construct_builtin() {
        return Some(BuiltinEntryMetadata::new("construct", 2, false, false));
    }
    if entry == lyng_js_types::js3_reflect_define_property_builtin() {
        return Some(BuiltinEntryMetadata::new("defineProperty", 3, false, false));
    }
    if entry == lyng_js_types::js3_reflect_delete_property_builtin() {
        return Some(BuiltinEntryMetadata::new("deleteProperty", 2, false, false));
    }
    if entry == lyng_js_types::js3_reflect_get_builtin() {
        return Some(BuiltinEntryMetadata::new("get", 2, false, false));
    }
    if entry == lyng_js_types::js3_reflect_get_own_property_descriptor_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "getOwnPropertyDescriptor",
            2,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_reflect_get_prototype_of_builtin() {
        return Some(BuiltinEntryMetadata::new("getPrototypeOf", 1, false, false));
    }
    if entry == lyng_js_types::js3_reflect_has_builtin() {
        return Some(BuiltinEntryMetadata::new("has", 2, false, false));
    }
    if entry == lyng_js_types::js3_reflect_is_extensible_builtin() {
        return Some(BuiltinEntryMetadata::new("isExtensible", 1, false, false));
    }
    if entry == lyng_js_types::js3_reflect_own_keys_builtin() {
        return Some(BuiltinEntryMetadata::new("ownKeys", 1, false, false));
    }
    if entry == lyng_js_types::js3_reflect_prevent_extensions_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "preventExtensions",
            1,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_reflect_set_builtin() {
        return Some(BuiltinEntryMetadata::new("set", 3, false, false));
    }
    if entry == lyng_js_types::js3_reflect_set_prototype_of_builtin() {
        return Some(BuiltinEntryMetadata::new("setPrototypeOf", 2, false, false));
    }
    if entry == lyng_js_types::js3_proxy_builtin() {
        return Some(BuiltinEntryMetadata::new("Proxy", 2, true, false));
    }
    if entry == lyng_js_types::js3_proxy_revocable_builtin() {
        return Some(BuiltinEntryMetadata::new("revocable", 2, false, false));
    }
    if entry == lyng_js_types::js3_proxy_revoke_builtin() {
        return Some(BuiltinEntryMetadata::new("", 0, false, false));
    }
    if entry == js3_iterator_prototype_iterator_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "[Symbol.iterator]",
            0,
            false,
            false,
        ));
    }
    if entry == js3_array_iterator_next_builtin() {
        return Some(BuiltinEntryMetadata::new("next", 0, false, false));
    }
    if entry == js3_map_iterator_next_builtin() {
        return Some(BuiltinEntryMetadata::new("next", 0, false, false));
    }
    if entry == js3_set_iterator_next_builtin() {
        return Some(BuiltinEntryMetadata::new("next", 0, false, false));
    }
    if entry == js3_string_builtin() {
        return Some(BuiltinEntryMetadata::new("String", 1, true, true));
    }
    if entry == js3_string_iterator_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "[Symbol.iterator]",
            0,
            false,
            false,
        ));
    }
    if entry == js3_string_iterator_next_builtin() {
        return Some(BuiltinEntryMetadata::new("next", 0, false, false));
    }
    if entry == js3_string_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == js3_string_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == js3_string_char_at_builtin() {
        return Some(BuiltinEntryMetadata::new("charAt", 1, false, false));
    }
    if entry == js3_string_char_code_at_builtin() {
        return Some(BuiltinEntryMetadata::new("charCodeAt", 1, false, false));
    }
    if entry == js3_string_match_builtin() {
        return Some(BuiltinEntryMetadata::new("match", 1, false, false));
    }
    if entry == js3_string_last_index_of_builtin() {
        return Some(BuiltinEntryMetadata::new("lastIndexOf", 1, false, false));
    }
    if entry == js3_string_pad_end_builtin() {
        return Some(BuiltinEntryMetadata::new("padEnd", 1, false, false));
    }
    if entry == js3_string_pad_start_builtin() {
        return Some(BuiltinEntryMetadata::new("padStart", 1, false, false));
    }
    if entry == js3_string_repeat_builtin() {
        return Some(BuiltinEntryMetadata::new("repeat", 1, false, false));
    }
    if entry == js3_string_replace_builtin() {
        return Some(BuiltinEntryMetadata::new("replace", 2, false, false));
    }
    if entry == js3_string_split_builtin() {
        return Some(BuiltinEntryMetadata::new("split", 2, false, false));
    }
    if entry == js3_string_slice_builtin() {
        return Some(BuiltinEntryMetadata::new("slice", 2, false, false));
    }
    if entry == js3_string_substring_builtin() {
        return Some(BuiltinEntryMetadata::new("substring", 2, false, false));
    }
    if entry == js3_string_starts_with_builtin() {
        return Some(BuiltinEntryMetadata::new("startsWith", 1, false, false));
    }
    if entry == js3_regexp_builtin() {
        return Some(BuiltinEntryMetadata::new("RegExp", 2, true, true));
    }
    if entry == js3_regexp_escape_builtin() {
        return Some(BuiltinEntryMetadata::new("escape", 1, false, false));
    }
    if entry == js3_regexp_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == js3_regexp_exec_builtin() {
        return Some(BuiltinEntryMetadata::new("exec", 1, false, false));
    }
    if entry == js3_regexp_test_builtin() {
        return Some(BuiltinEntryMetadata::new("test", 1, false, false));
    }
    if entry == js3_regexp_global_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get global", 0, false, false));
    }
    if entry == js3_regexp_ignore_case_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get ignoreCase", 0, false, false));
    }
    if entry == js3_regexp_multiline_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get multiline", 0, false, false));
    }
    if entry == js3_regexp_dot_all_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get dotAll", 0, false, false));
    }
    if entry == js3_regexp_unicode_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get unicode", 0, false, false));
    }
    if entry == js3_regexp_sticky_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get sticky", 0, false, false));
    }
    if entry == js3_regexp_source_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get source", 0, false, false));
    }
    if entry == js3_regexp_flags_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get flags", 0, false, false));
    }
    if entry == js3_regexp_has_indices_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get hasIndices", 0, false, false));
    }
    if entry == js3_date_builtin() {
        return Some(BuiltinEntryMetadata::new("Date", 7, true, true));
    }
    if entry == js3_date_now_builtin() {
        return Some(BuiltinEntryMetadata::new("now", 0, false, false));
    }
    if entry == js3_date_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == js3_date_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == js3_date_get_timezone_offset_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "getTimezoneOffset",
            0,
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
    if entry == js3_number_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == js3_number_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == js3_math_abs_builtin() {
        return Some(BuiltinEntryMetadata::new("abs", 1, false, false));
    }
    if entry == js3_math_floor_builtin() {
        return Some(BuiltinEntryMetadata::new("floor", 1, false, false));
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
    if entry == js3_math_round_builtin() {
        return Some(BuiltinEntryMetadata::new("round", 1, false, false));
    }
    if entry == js3_math_sign_builtin() {
        return Some(BuiltinEntryMetadata::new("sign", 1, false, false));
    }
    if entry == js3_math_sqrt_builtin() {
        return Some(BuiltinEntryMetadata::new("sqrt", 1, false, false));
    }
    if entry == js3_math_trunc_builtin() {
        return Some(BuiltinEntryMetadata::new("trunc", 1, false, false));
    }
    if entry == js3_bigint_builtin() {
        return Some(BuiltinEntryMetadata::new("BigInt", 1, false, true));
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
    if entry == lyng_js_types::js3_suppressed_error_builtin() {
        return Some(BuiltinEntryMetadata::new("SuppressedError", 3, true, true));
    }
    if entry == js3_eval_builtin() {
        return Some(BuiltinEntryMetadata::new("eval", 1, false, false));
    }
    if entry == js3_promise_builtin() {
        return Some(BuiltinEntryMetadata::new("Promise", 1, true, true));
    }
    if entry == lyng_js_types::js3_disposable_stack_builtin() {
        return Some(BuiltinEntryMetadata::new("DisposableStack", 0, true, true));
    }
    if entry == lyng_js_types::js3_disposable_stack_use_builtin() {
        return Some(BuiltinEntryMetadata::new("use", 1, false, false));
    }
    if entry == lyng_js_types::js3_disposable_stack_adopt_builtin() {
        return Some(BuiltinEntryMetadata::new("adopt", 2, false, false));
    }
    if entry == lyng_js_types::js3_disposable_stack_defer_builtin() {
        return Some(BuiltinEntryMetadata::new("defer", 1, false, false));
    }
    if entry == lyng_js_types::js3_disposable_stack_move_builtin() {
        return Some(BuiltinEntryMetadata::new("move", 0, false, false));
    }
    if entry == lyng_js_types::js3_disposable_stack_disposed_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get disposed", 0, false, false));
    }
    if entry == lyng_js_types::js3_disposable_stack_dispose_builtin() {
        return Some(BuiltinEntryMetadata::new("dispose", 0, false, false));
    }
    if entry == lyng_js_types::js3_async_disposable_stack_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "AsyncDisposableStack",
            0,
            true,
            true,
        ));
    }
    if entry == lyng_js_types::js3_async_disposable_stack_use_builtin() {
        return Some(BuiltinEntryMetadata::new("use", 1, false, false));
    }
    if entry == lyng_js_types::js3_async_disposable_stack_adopt_builtin() {
        return Some(BuiltinEntryMetadata::new("adopt", 2, false, false));
    }
    if entry == lyng_js_types::js3_async_disposable_stack_defer_builtin() {
        return Some(BuiltinEntryMetadata::new("defer", 1, false, false));
    }
    if entry == lyng_js_types::js3_async_disposable_stack_move_builtin() {
        return Some(BuiltinEntryMetadata::new("move", 0, false, false));
    }
    if entry == lyng_js_types::js3_async_disposable_stack_disposed_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get disposed", 0, false, false));
    }
    if entry == lyng_js_types::js3_async_disposable_stack_dispose_async_builtin() {
        return Some(BuiltinEntryMetadata::new("disposeAsync", 0, false, false));
    }
    if entry == lyng_js_types::js3_async_disposal_resume_builtin() {
        return Some(BuiltinEntryMetadata::new("", 1, false, false));
    }
    if entry == lyng_js_types::js3_create_sync_disposal_scope_builtin() {
        return Some(BuiltinEntryMetadata::new("", 0, false, false));
    }
    if entry == lyng_js_types::js3_create_async_disposal_scope_builtin() {
        return Some(BuiltinEntryMetadata::new("", 0, false, false));
    }
    if entry == lyng_js_types::js3_add_sync_disposable_resource_builtin() {
        return Some(BuiltinEntryMetadata::new("", 2, false, false));
    }
    if entry == lyng_js_types::js3_add_async_disposable_resource_builtin() {
        return Some(BuiltinEntryMetadata::new("", 2, false, false));
    }
    if entry == lyng_js_types::js3_dispose_scope_builtin() {
        return Some(BuiltinEntryMetadata::new("", 1, false, false));
    }
    if entry == lyng_js_types::js3_dispose_scope_async_builtin() {
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

pub(crate) fn allocate_builtin_date_object(
    agent: &mut Agent,
    root_shape: ShapeId,
    prototype: Option<ObjectRef>,
    value: Value,
) -> ObjectRef {
    agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_prototype(prototype)
                .with_date_value(value)
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::Date)),
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

fn reparent_builtin_object(agent: &mut Agent, object: ObjectRef, prototype: Option<ObjectRef>) {
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

fn define_builtin_data_property(
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

fn define_builtin_accessor_property(
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

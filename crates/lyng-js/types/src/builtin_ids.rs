use crate::ids::BuiltinFunctionId;

#[inline]
const fn builtin_id(raw: u32) -> BuiltinFunctionId {
    let Some(id) = BuiltinFunctionId::from_raw(raw) else {
        unreachable!();
    };
    id
}

macro_rules! builtin_id_accessors {
    ($($accessor:ident => $raw:path;)+) => {
        $(
            #[inline]
            pub const fn $accessor() -> BuiltinFunctionId {
                builtin_id($raw)
            }
        )+
    };
}

/// Reserved builtin ID namespace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinIdNamespace {
    /// VM/compiler helper builtin namespace.
    Internal,
    /// Public core ECMAScript builtin namespace.
    Core,
    /// Public completion-oriented builtin namespace.
    Completion,
}

/// One auditable builtin ID registry row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltinIdRegistryEntry {
    accessor_name: &'static str,
    id: BuiltinFunctionId,
    namespace: BuiltinIdNamespace,
}

impl BuiltinIdRegistryEntry {
    #[inline]
    const fn new(accessor_name: &'static str, raw: u32, namespace: BuiltinIdNamespace) -> Self {
        Self {
            accessor_name,
            id: builtin_id(raw),
            namespace,
        }
    }

    #[inline]
    pub const fn accessor_name(self) -> &'static str {
        self.accessor_name
    }

    #[inline]
    pub const fn id(self) -> BuiltinFunctionId {
        self.id
    }

    #[inline]
    pub const fn raw(self) -> u32 {
        self.id.get()
    }

    #[inline]
    pub const fn namespace(self) -> BuiltinIdNamespace {
        self.namespace
    }
}

macro_rules! define_builtin_id_registry {
    ($($raw:ident = $value:literal => $accessor:ident in $namespace:ident;)+) => {
        $(const $raw: u32 = $value;)+

        /// Stable registry for every reserved Lyng JS builtin function ID.
        pub const BUILTIN_ID_REGISTRY: &[BuiltinIdRegistryEntry] = &[
            $(BuiltinIdRegistryEntry::new(
                stringify!($accessor),
                $raw,
                BuiltinIdNamespace::$namespace,
            ),)+
        ];
    };
}

mod binary_data;
mod collections;
mod core_ids;
mod disposal;
mod internal;
mod promises;
mod temporal;
mod typed_arrays;

pub use binary_data::*;
pub use collections::*;
pub use core_ids::*;
pub use disposal::*;
pub use internal::*;
pub use promises::*;
pub use temporal::*;
pub use typed_arrays::*;

define_builtin_id_registry! {
    INTERNAL_FUNCTION_CALL_RAW = 1_001 => internal_function_call_builtin in Internal;
    INTERNAL_STRING_REPLACE_RAW = 1_002 => internal_string_replace_builtin in Internal;
    INTERNAL_STRING_INDEX_OF_RAW = 1_003 => internal_string_index_of_builtin in Internal;
    INTERNAL_ARRAY_INDEX_OF_RAW = 1_004 => internal_array_index_of_builtin in Internal;
    INTERNAL_ARRAY_PUSH_RAW = 1_005 => internal_array_push_builtin in Internal;
    INTERNAL_OBJECT_TO_STRING_RAW = 1_006 => internal_object_to_string_builtin in Internal;
    INTERNAL_TEMPLATE_TO_STRING_RAW = 1_007 => internal_template_to_string_builtin in Internal;
    INTERNAL_GET_TEMPLATE_OBJECT_RAW = 1_008 => internal_get_template_object_builtin in Internal;
    INTERNAL_INSTANCE_OF_RAW = 1_009 => internal_instance_of_builtin in Internal;
    INTERNAL_DEFINE_GETTER_PROPERTY_RAW = 1_010 => internal_define_getter_property_builtin in Internal;
    INTERNAL_DEFINE_SETTER_PROPERTY_RAW = 1_011 => internal_define_setter_property_builtin in Internal;
    INTERNAL_OBJECT_HAS_OWN_PROPERTY_RAW = 1_012 => internal_object_has_own_property_builtin in Internal;
    INTERNAL_THROW_TYPE_ERROR_RAW = 1_013 => internal_throw_type_error_builtin in Internal;
    INTERNAL_ARRAY_POP_RAW = 1_014 => internal_array_pop_builtin in Internal;
    INTERNAL_DEFINE_METHOD_PROPERTY_RAW = 1_015 => internal_define_method_property_builtin in Internal;
    INTERNAL_DEFINE_CLASS_GETTER_PROPERTY_RAW = 1_016 => internal_define_class_getter_property_builtin in Internal;
    INTERNAL_DEFINE_CLASS_SETTER_PROPERTY_RAW = 1_017 => internal_define_class_setter_property_builtin in Internal;
    INTERNAL_SET_FUNCTION_HOME_OBJECT_RAW = 1_018 => internal_set_function_home_object_builtin in Internal;
    INTERNAL_DEFINE_PRIVATE_FIELD_RAW = 1_019 => internal_define_private_field_builtin in Internal;
    INTERNAL_PRIVATE_FIELD_INIT_RAW = 1_020 => internal_private_field_init_builtin in Internal;
    INTERNAL_PRIVATE_FIELD_GET_RAW = 1_021 => internal_private_field_get_builtin in Internal;
    INTERNAL_PRIVATE_FIELD_SET_RAW = 1_022 => internal_private_field_set_builtin in Internal;
    INTERNAL_PRIVATE_HAS_RAW = 1_023 => internal_private_has_builtin in Internal;
    INTERNAL_SUPER_PROPERTY_GET_RAW = 1_024 => internal_super_property_get_builtin in Internal;
    INTERNAL_SUPER_PROPERTY_SET_RAW = 1_025 => internal_super_property_set_builtin in Internal;
    INTERNAL_SUPER_BASE_RAW = 1_026 => internal_super_base_builtin in Internal;
    INTERNAL_CONSTRUCT_SUPER_RAW = 1_027 => internal_construct_super_builtin in Internal;
    INTERNAL_OBJECT_LITERAL_SET_PROTOTYPE_RAW = 1_028 => internal_object_literal_set_prototype_builtin in Internal;
    INTERNAL_BIND_FUNCTION_PRIVATE_ENV_RAW = 1_029 => internal_bind_function_private_env_builtin in Internal;
    INTERNAL_INSTALL_INSTANCE_FIELD_KEY_RAW = 1_030 => internal_install_instance_field_key_builtin in Internal;
    INTERNAL_GET_INSTANCE_FIELD_KEY_RAW = 1_031 => internal_get_instance_field_key_builtin in Internal;
    INTERNAL_CONSTRUCT_SUPER_SPREAD_RAW = 1_032 => internal_construct_super_spread_builtin in Internal;
    INTERNAL_IMPORT_META_RAW = 1_033 => internal_import_meta_builtin in Internal;
    INTERNAL_CAPTURE_ARROW_CONTEXT_RAW = 1_034 => internal_capture_arrow_context_builtin in Internal;
    INTERNAL_DYNAMIC_IMPORT_RAW = 1_035 => internal_dynamic_import_builtin in Internal;
    INTERNAL_FINALIZATION_REGISTRY_CLEANUP_JOB_RAW = 1_036 => internal_finalization_registry_cleanup_job_builtin in Internal;
    INTERNAL_DIRECT_EVAL_RAW = 1_037 => internal_direct_eval_builtin in Internal;
    INTERNAL_REGEXP_LITERAL_RAW = 1_038 => internal_regexp_literal_builtin in Internal;
    INTERNAL_REQUIRE_CONSTRUCTOR_RAW = 1_039 => internal_require_constructor_builtin in Internal;
    INTERNAL_CONSTRUCT_SUPER_ARRAY_LIKE_RAW = 1_040 => internal_construct_super_array_like_builtin in Internal;
    INTERNAL_SUPER_CONSTRUCTOR_RAW = 1_041 => internal_super_constructor_builtin in Internal;
    BOOLEAN_RAW = 2_001 => boolean_builtin in Core;
    BOOLEAN_TO_STRING_RAW = 2_002 => boolean_to_string_builtin in Core;
    BOOLEAN_VALUE_OF_RAW = 2_003 => boolean_value_of_builtin in Core;
    SYMBOL_RAW = 2_004 => symbol_builtin in Core;
    SYMBOL_FOR_RAW = 2_005 => symbol_for_builtin in Core;
    SYMBOL_KEY_FOR_RAW = 2_006 => symbol_key_for_builtin in Core;
    SYMBOL_TO_STRING_RAW = 2_007 => symbol_to_string_builtin in Core;
    SYMBOL_VALUE_OF_RAW = 2_008 => symbol_value_of_builtin in Core;
    SYMBOL_TO_PRIMITIVE_RAW = 2_009 => symbol_to_primitive_builtin in Core;
    SYMBOL_DESCRIPTION_GETTER_RAW = 2_010 => symbol_description_getter_builtin in Core;
    FUNCTION_RAW = 2_011 => function_builtin in Core;
    FUNCTION_PROTOTYPE_RAW = 2_012 => function_prototype_builtin in Core;
    FUNCTION_CALL_RAW = 2_013 => function_call_builtin in Core;
    FUNCTION_APPLY_RAW = 2_014 => function_apply_builtin in Core;
    FUNCTION_BIND_RAW = 2_015 => function_bind_builtin in Core;
    FUNCTION_TO_STRING_RAW = 2_016 => function_to_string_builtin in Core;
    OBJECT_RAW = 2_017 => object_builtin in Core;
    OBJECT_CREATE_RAW = 2_018 => object_create_builtin in Core;
    OBJECT_GET_PROTOTYPE_OF_RAW = 2_019 => object_get_prototype_of_builtin in Core;
    OBJECT_SET_PROTOTYPE_OF_RAW = 2_020 => object_set_prototype_of_builtin in Core;
    OBJECT_GET_OWN_PROPERTY_DESCRIPTOR_RAW = 2_021 => object_get_own_property_descriptor_builtin in Core;
    OBJECT_DEFINE_PROPERTY_RAW = 2_022 => object_define_property_builtin in Core;
    OBJECT_PREVENT_EXTENSIONS_RAW = 2_023 => object_prevent_extensions_builtin in Core;
    OBJECT_IS_EXTENSIBLE_RAW = 2_024 => object_is_extensible_builtin in Core;
    OBJECT_SEAL_RAW = 2_025 => object_seal_builtin in Core;
    OBJECT_FREEZE_RAW = 2_026 => object_freeze_builtin in Core;
    OBJECT_IS_SEALED_RAW = 2_027 => object_is_sealed_builtin in Core;
    OBJECT_IS_FROZEN_RAW = 2_028 => object_is_frozen_builtin in Core;
    OBJECT_TO_STRING_RAW = 2_029 => object_to_string_builtin in Core;
    OBJECT_VALUE_OF_RAW = 2_030 => object_value_of_builtin in Core;
    OBJECT_HAS_OWN_PROPERTY_RAW = 2_031 => object_has_own_property_builtin in Core;
    OBJECT_IS_PROTOTYPE_OF_RAW = 2_032 => object_is_prototype_of_builtin in Core;
    OBJECT_PROPERTY_IS_ENUMERABLE_RAW = 2_033 => object_property_is_enumerable_builtin in Core;
    ERROR_RAW = 2_034 => error_builtin in Core;
    ERROR_TO_STRING_RAW = 2_035 => error_to_string_builtin in Core;
    EVAL_ERROR_RAW = 2_036 => eval_error_builtin in Core;
    RANGE_ERROR_RAW = 2_037 => range_error_builtin in Core;
    REFERENCE_ERROR_RAW = 2_038 => reference_error_builtin in Core;
    SYNTAX_ERROR_RAW = 2_039 => syntax_error_builtin in Core;
    TYPE_ERROR_RAW = 2_040 => type_error_builtin in Core;
    URI_ERROR_RAW = 2_041 => uri_error_builtin in Core;
    ERROR_IS_ERROR_RAW = 2_285 => error_is_error_builtin in Core;
    NUMBER_RAW = 2_042 => number_builtin in Core;
    NUMBER_IS_FINITE_RAW = 2_043 => number_is_finite_builtin in Core;
    NUMBER_IS_INTEGER_RAW = 2_044 => number_is_integer_builtin in Core;
    NUMBER_IS_NAN_RAW = 2_045 => number_is_nan_builtin in Core;
    NUMBER_IS_SAFE_INTEGER_RAW = 2_046 => number_is_safe_integer_builtin in Core;
    NUMBER_TO_STRING_RAW = 2_047 => number_to_string_builtin in Core;
    NUMBER_VALUE_OF_RAW = 2_048 => number_value_of_builtin in Core;
    BIGINT_RAW = 2_049 => bigint_builtin in Core;
    BIGINT_TO_STRING_RAW = 2_050 => bigint_to_string_builtin in Core;
    BIGINT_VALUE_OF_RAW = 2_051 => bigint_value_of_builtin in Core;
    BIGINT_AS_INT_N_RAW = 2_148 => bigint_as_int_n_builtin in Core;
    BIGINT_AS_UINT_N_RAW = 2_149 => bigint_as_uint_n_builtin in Core;
    MATH_ABS_RAW = 2_052 => math_abs_builtin in Core;
    MATH_FLOOR_RAW = 2_053 => math_floor_builtin in Core;
    MATH_MAX_RAW = 2_054 => math_max_builtin in Core;
    MATH_MIN_RAW = 2_055 => math_min_builtin in Core;
    MATH_POW_RAW = 2_056 => math_pow_builtin in Core;
    MATH_ROUND_RAW = 2_057 => math_round_builtin in Core;
    MATH_SIGN_RAW = 2_058 => math_sign_builtin in Core;
    MATH_TRUNC_RAW = 2_059 => math_trunc_builtin in Core;
    MATH_SQRT_RAW = 2_060 => math_sqrt_builtin in Core;
    MATH_ACOS_RAW = 2_151 => math_acos_builtin in Core;
    MATH_ACOSH_RAW = 2_152 => math_acosh_builtin in Core;
    MATH_ASIN_RAW = 2_153 => math_asin_builtin in Core;
    MATH_ASINH_RAW = 2_154 => math_asinh_builtin in Core;
    MATH_ATAN_RAW = 2_155 => math_atan_builtin in Core;
    MATH_ATAN2_RAW = 2_156 => math_atan2_builtin in Core;
    MATH_ATANH_RAW = 2_157 => math_atanh_builtin in Core;
    MATH_CBRT_RAW = 2_158 => math_cbrt_builtin in Core;
    MATH_CEIL_RAW = 2_159 => math_ceil_builtin in Core;
    MATH_CLZ32_RAW = 2_160 => math_clz32_builtin in Core;
    MATH_COS_RAW = 2_161 => math_cos_builtin in Core;
    MATH_COSH_RAW = 2_162 => math_cosh_builtin in Core;
    MATH_EXP_RAW = 2_163 => math_exp_builtin in Core;
    MATH_EXPM1_RAW = 2_164 => math_expm1_builtin in Core;
    MATH_F16ROUND_RAW = 2_165 => math_f16round_builtin in Core;
    MATH_FROUND_RAW = 2_166 => math_fround_builtin in Core;
    MATH_HYPOT_RAW = 2_167 => math_hypot_builtin in Core;
    MATH_IMUL_RAW = 2_168 => math_imul_builtin in Core;
    MATH_LOG_RAW = 2_169 => math_log_builtin in Core;
    MATH_LOG10_RAW = 2_170 => math_log10_builtin in Core;
    MATH_LOG1P_RAW = 2_171 => math_log1p_builtin in Core;
    MATH_LOG2_RAW = 2_172 => math_log2_builtin in Core;
    MATH_RANDOM_RAW = 2_173 => math_random_builtin in Core;
    MATH_SIN_RAW = 2_174 => math_sin_builtin in Core;
    MATH_SINH_RAW = 2_175 => math_sinh_builtin in Core;
    MATH_SUM_PRECISE_RAW = 2_176 => math_sum_precise_builtin in Core;
    MATH_TAN_RAW = 2_177 => math_tan_builtin in Core;
    MATH_TANH_RAW = 2_178 => math_tanh_builtin in Core;
    NUMBER_TO_FIXED_RAW = 2_179 => number_to_fixed_builtin in Core;
    NUMBER_TO_PRECISION_RAW = 2_180 => number_to_precision_builtin in Core;
    NUMBER_TO_LOCALE_STRING_RAW = 2_181 => number_to_locale_string_builtin in Core;
    NUMBER_TO_EXPONENTIAL_RAW = 2_061 => number_to_exponential_builtin in Core;
    STRING_RAW = 2_062 => string_builtin in Core;
    STRING_TO_STRING_RAW = 2_063 => string_to_string_builtin in Core;
    STRING_VALUE_OF_RAW = 2_064 => string_value_of_builtin in Core;
    STRING_CONCAT_RAW = 2_150 => string_concat_builtin in Core;
    REGEXP_RAW = 2_065 => regexp_builtin in Core;
    REGEXP_TO_STRING_RAW = 2_066 => regexp_to_string_builtin in Core;
    DATE_RAW = 2_067 => date_builtin in Core;
    DATE_NOW_RAW = 2_068 => date_now_builtin in Core;
    DATE_PARSE_RAW = 2_069 => date_parse_builtin in Core;
    DATE_TO_STRING_RAW = 2_070 => date_to_string_builtin in Core;
    DATE_VALUE_OF_RAW = 2_071 => date_value_of_builtin in Core;
    PARSE_INT_RAW = 2_072 => parse_int_builtin in Core;
    PARSE_FLOAT_RAW = 2_073 => parse_float_builtin in Core;
    IS_NAN_RAW = 2_074 => is_nan_builtin in Core;
    IS_FINITE_RAW = 2_075 => is_finite_builtin in Core;
    ENCODE_URI_RAW = 2_076 => encode_uri_builtin in Core;
    ENCODE_URI_COMPONENT_RAW = 2_077 => encode_uri_component_builtin in Core;
    DECODE_URI_RAW = 2_078 => decode_uri_builtin in Core;
    DECODE_URI_COMPONENT_RAW = 2_079 => decode_uri_component_builtin in Core;
    STRING_CHAR_AT_RAW = 2_080 => string_char_at_builtin in Core;
    STRING_CHAR_CODE_AT_RAW = 2_081 => string_char_code_at_builtin in Core;
    STRING_MATCH_RAW = 2_082 => string_match_builtin in Core;
    STRING_PAD_END_RAW = 2_083 => string_pad_end_builtin in Core;
    STRING_PAD_START_RAW = 2_084 => string_pad_start_builtin in Core;
    STRING_REPLACE_RAW = 2_085 => string_replace_builtin in Core;
    STRING_SPLIT_RAW = 2_086 => string_split_builtin in Core;
    STRING_LAST_INDEX_OF_RAW = 2_087 => string_last_index_of_builtin in Core;
    STRING_SUBSTRING_RAW = 2_088 => string_substring_builtin in Core;
    REGEXP_EXEC_RAW = 2_089 => regexp_exec_builtin in Core;
    REGEXP_TEST_RAW = 2_090 => regexp_test_builtin in Core;
    REGEXP_GLOBAL_GETTER_RAW = 2_091 => regexp_global_getter_builtin in Core;
    REGEXP_IGNORE_CASE_GETTER_RAW = 2_092 => regexp_ignore_case_getter_builtin in Core;
    REGEXP_MULTILINE_GETTER_RAW = 2_093 => regexp_multiline_getter_builtin in Core;
    REGEXP_DOT_ALL_GETTER_RAW = 2_094 => regexp_dot_all_getter_builtin in Core;
    REGEXP_UNICODE_GETTER_RAW = 2_095 => regexp_unicode_getter_builtin in Core;
    REGEXP_STICKY_GETTER_RAW = 2_096 => regexp_sticky_getter_builtin in Core;
    ARRAY_RAW = 2_097 => array_builtin in Core;
    ARRAY_CONCAT_RAW = 2_098 => array_concat_builtin in Core;
    ARRAY_COPY_WITHIN_RAW = 2_099 => array_copy_within_builtin in Core;
    ARRAY_FILTER_RAW = 2_100 => array_filter_builtin in Core;
    ARRAY_MAP_RAW = 2_101 => array_map_builtin in Core;
    ARRAY_REVERSE_RAW = 2_102 => array_reverse_builtin in Core;
    ARRAY_SLICE_RAW = 2_103 => array_slice_builtin in Core;
    ARRAY_SORT_RAW = 2_104 => array_sort_builtin in Core;
    ARRAY_SPLICE_RAW = 2_105 => array_splice_builtin in Core;
    ARRAY_TO_LOCALE_STRING_RAW = 2_106 => array_to_locale_string_builtin in Core;
    ARRAY_VALUES_RAW = 2_107 => array_values_builtin in Core;
    ARRAY_KEYS_RAW = 2_108 => array_keys_builtin in Core;
    ARRAY_ENTRIES_RAW = 2_109 => array_entries_builtin in Core;
    ITERATOR_PROTOTYPE_ITERATOR_RAW = 2_110 => iterator_prototype_iterator_builtin in Core;
    ARRAY_ITERATOR_NEXT_RAW = 2_111 => array_iterator_next_builtin in Core;
    STRING_ITERATOR_NEXT_RAW = 2_112 => string_iterator_next_builtin in Core;
    STRING_ITERATOR_RAW = 2_113 => string_iterator_builtin in Core;
    ARRAY_SPECIES_GETTER_RAW = 2_114 => array_species_getter_builtin in Core;
    REGEXP_SOURCE_GETTER_RAW = 2_115 => regexp_source_getter_builtin in Core;
    REGEXP_FLAGS_GETTER_RAW = 2_116 => regexp_flags_getter_builtin in Core;
    REGEXP_HAS_INDICES_GETTER_RAW = 2_117 => regexp_has_indices_getter_builtin in Core;
    REGEXP_ESCAPE_RAW = 2_118 => regexp_escape_builtin in Core;
    STRING_SLICE_RAW = 2_119 => string_slice_builtin in Core;
    ARRAY_TO_STRING_RAW = 2_120 => array_to_string_builtin in Core;
    ARRAY_FOR_EACH_RAW = 2_121 => array_for_each_builtin in Core;
    OBJECT_KEYS_RAW = 2_122 => object_keys_builtin in Core;
    OBJECT_GET_OWN_PROPERTY_NAMES_RAW = 2_123 => object_get_own_property_names_builtin in Core;
    OBJECT_GET_OWN_PROPERTY_SYMBOLS_RAW = 2_124 => object_get_own_property_symbols_builtin in Core;
    GENERATOR_FUNCTION_RAW = 2_125 => generator_function_builtin in Core;
    GENERATOR_NEXT_RAW = 2_126 => generator_next_builtin in Core;
    GENERATOR_RETURN_RAW = 2_127 => generator_return_builtin in Core;
    GENERATOR_THROW_RAW = 2_128 => generator_throw_builtin in Core;
    ARRAY_IS_ARRAY_RAW = 2_129 => array_is_array_builtin in Core;
    OBJECT_DEFINE_PROPERTIES_RAW = 2_130 => object_define_properties_builtin in Core;
    EVAL_RAW = 2_131 => eval_builtin in Core;
    ARRAY_FROM_RAW = 2_132 => array_from_builtin in Core;
    ARRAY_FILL_RAW = 2_133 => array_fill_builtin in Core;
    OBJECT_TO_LOCALE_STRING_RAW = 2_134 => object_to_locale_string_builtin in Core;
    ARRAY_JOIN_RAW = 2_135 => array_join_builtin in Core;
    OBJECT_ENTRIES_RAW = 2_136 => object_entries_builtin in Core;
    ARRAY_UNSHIFT_RAW = 2_137 => array_unshift_builtin in Core;
    STRING_STARTS_WITH_RAW = 2_138 => string_starts_with_builtin in Core;
    OBJECT_HAS_OWN_RAW = 2_139 => object_has_own_builtin in Core;
    ARRAY_SHIFT_RAW = 2_140 => array_shift_builtin in Core;
    OBJECT_VALUES_RAW = 2_141 => object_values_builtin in Core;
    STRING_REPEAT_RAW = 2_142 => string_repeat_builtin in Core;
    OBJECT_GET_OWN_PROPERTY_DESCRIPTORS_RAW = 2_143 => object_get_own_property_descriptors_builtin in Core;
    OBJECT_IS_RAW = 2_144 => object_is_builtin in Core;
    DATE_GET_TIMEZONE_OFFSET_RAW = 2_145 => date_get_timezone_offset_builtin in Core;
    STRING_FROM_CHAR_CODE_RAW = 2_146 => string_from_char_code_builtin in Core;
    STRING_SEARCH_RAW = 2_147 => string_search_builtin in Core;
    DATE_UTC_RAW = 2_182 => date_utc_builtin in Core;
    DATE_TO_DATE_STRING_RAW = 2_183 => date_to_date_string_builtin in Core;
    DATE_TO_TIME_STRING_RAW = 2_184 => date_to_time_string_builtin in Core;
    DATE_TO_LOCALE_STRING_RAW = 2_185 => date_to_locale_string_builtin in Core;
    DATE_TO_LOCALE_DATE_STRING_RAW = 2_186 => date_to_locale_date_string_builtin in Core;
    DATE_TO_LOCALE_TIME_STRING_RAW = 2_187 => date_to_locale_time_string_builtin in Core;
    DATE_GET_TIME_RAW = 2_188 => date_get_time_builtin in Core;
    DATE_GET_FULL_YEAR_RAW = 2_189 => date_get_full_year_builtin in Core;
    DATE_GET_UTC_FULL_YEAR_RAW = 2_190 => date_get_utc_full_year_builtin in Core;
    DATE_GET_MONTH_RAW = 2_191 => date_get_month_builtin in Core;
    DATE_GET_UTC_MONTH_RAW = 2_192 => date_get_utc_month_builtin in Core;
    DATE_GET_DATE_RAW = 2_193 => date_get_date_builtin in Core;
    DATE_GET_UTC_DATE_RAW = 2_194 => date_get_utc_date_builtin in Core;
    DATE_GET_DAY_RAW = 2_195 => date_get_day_builtin in Core;
    DATE_GET_UTC_DAY_RAW = 2_196 => date_get_utc_day_builtin in Core;
    DATE_GET_HOURS_RAW = 2_197 => date_get_hours_builtin in Core;
    DATE_GET_UTC_HOURS_RAW = 2_198 => date_get_utc_hours_builtin in Core;
    DATE_GET_MINUTES_RAW = 2_199 => date_get_minutes_builtin in Core;
    DATE_GET_UTC_MINUTES_RAW = 2_200 => date_get_utc_minutes_builtin in Core;
    DATE_GET_SECONDS_RAW = 2_201 => date_get_seconds_builtin in Core;
    DATE_GET_UTC_SECONDS_RAW = 2_202 => date_get_utc_seconds_builtin in Core;
    DATE_GET_MILLISECONDS_RAW = 2_203 => date_get_milliseconds_builtin in Core;
    DATE_GET_UTC_MILLISECONDS_RAW = 2_204 => date_get_utc_milliseconds_builtin in Core;
    DATE_SET_TIME_RAW = 2_205 => date_set_time_builtin in Core;
    DATE_SET_MILLISECONDS_RAW = 2_206 => date_set_milliseconds_builtin in Core;
    DATE_SET_UTC_MILLISECONDS_RAW = 2_207 => date_set_utc_milliseconds_builtin in Core;
    DATE_SET_SECONDS_RAW = 2_208 => date_set_seconds_builtin in Core;
    DATE_SET_UTC_SECONDS_RAW = 2_209 => date_set_utc_seconds_builtin in Core;
    DATE_SET_MINUTES_RAW = 2_210 => date_set_minutes_builtin in Core;
    DATE_SET_UTC_MINUTES_RAW = 2_211 => date_set_utc_minutes_builtin in Core;
    DATE_SET_HOURS_RAW = 2_212 => date_set_hours_builtin in Core;
    DATE_SET_UTC_HOURS_RAW = 2_213 => date_set_utc_hours_builtin in Core;
    DATE_SET_DATE_RAW = 2_214 => date_set_date_builtin in Core;
    DATE_SET_UTC_DATE_RAW = 2_215 => date_set_utc_date_builtin in Core;
    DATE_SET_MONTH_RAW = 2_216 => date_set_month_builtin in Core;
    DATE_SET_UTC_MONTH_RAW = 2_217 => date_set_utc_month_builtin in Core;
    DATE_SET_FULL_YEAR_RAW = 2_218 => date_set_full_year_builtin in Core;
    DATE_SET_UTC_FULL_YEAR_RAW = 2_219 => date_set_utc_full_year_builtin in Core;
    DATE_TO_UTC_STRING_RAW = 2_220 => date_to_utc_string_builtin in Core;
    DATE_TO_ISO_STRING_RAW = 2_221 => date_to_iso_string_builtin in Core;
    DATE_TO_JSON_RAW = 2_222 => date_to_json_builtin in Core;
    DATE_TO_PRIMITIVE_RAW = 2_223 => date_to_primitive_builtin in Core;
    DATE_TO_TEMPORAL_INSTANT_RAW = 2_224 => date_to_temporal_instant_builtin in Core;
    STRING_FROM_CODE_POINT_RAW = 2_225 => string_from_code_point_builtin in Core;
    STRING_RAW_RAW = 2_226 => string_raw_builtin in Core;
    STRING_AT_RAW = 2_227 => string_at_builtin in Core;
    STRING_CODE_POINT_AT_RAW = 2_228 => string_code_point_at_builtin in Core;
    STRING_ENDS_WITH_RAW = 2_229 => string_ends_with_builtin in Core;
    STRING_INCLUDES_RAW = 2_230 => string_includes_builtin in Core;
    STRING_INDEX_OF_RAW = 2_231 => string_index_of_builtin in Core;
    STRING_IS_WELL_FORMED_RAW = 2_232 => string_is_well_formed_builtin in Core;
    STRING_LOCALE_COMPARE_RAW = 2_233 => string_locale_compare_builtin in Core;
    STRING_NORMALIZE_RAW = 2_234 => string_normalize_builtin in Core;
    STRING_REPLACE_ALL_RAW = 2_235 => string_replace_all_builtin in Core;
    STRING_TO_LOCALE_LOWER_CASE_RAW = 2_236 => string_to_locale_lower_case_builtin in Core;
    STRING_TO_LOCALE_UPPER_CASE_RAW = 2_237 => string_to_locale_upper_case_builtin in Core;
    STRING_TO_LOWER_CASE_RAW = 2_238 => string_to_lower_case_builtin in Core;
    STRING_TO_UPPER_CASE_RAW = 2_239 => string_to_upper_case_builtin in Core;
    STRING_TO_WELL_FORMED_RAW = 2_240 => string_to_well_formed_builtin in Core;
    STRING_TRIM_RAW = 2_241 => string_trim_builtin in Core;
    STRING_TRIM_END_RAW = 2_242 => string_trim_end_builtin in Core;
    STRING_TRIM_START_RAW = 2_243 => string_trim_start_builtin in Core;
    REGEXP_SYMBOL_MATCH_RAW = 2_244 => regexp_symbol_match_builtin in Core;
    REGEXP_SYMBOL_REPLACE_RAW = 2_245 => regexp_symbol_replace_builtin in Core;
    REGEXP_SYMBOL_SEARCH_RAW = 2_246 => regexp_symbol_search_builtin in Core;
    REGEXP_SYMBOL_SPLIT_RAW = 2_247 => regexp_symbol_split_builtin in Core;
    REGEXP_SYMBOL_MATCH_ALL_RAW = 2_248 => regexp_symbol_match_all_builtin in Core;
    STRING_MATCH_ALL_RAW = 2_249 => string_match_all_builtin in Core;
    ARRAY_LAST_INDEX_OF_RAW = 2_250 => array_last_index_of_builtin in Core;
    ARRAY_EVERY_RAW = 2_251 => array_every_builtin in Core;
    ARRAY_SOME_RAW = 2_252 => array_some_builtin in Core;
    ARRAY_INCLUDES_RAW = 2_253 => array_includes_builtin in Core;
    ARRAY_INDEX_OF_RAW = 2_254 => array_index_of_builtin in Core;
    ARRAY_REDUCE_RAW = 2_255 => array_reduce_builtin in Core;
    ARRAY_REDUCE_RIGHT_RAW = 2_256 => array_reduce_right_builtin in Core;
    ARRAY_FIND_RAW = 2_257 => array_find_builtin in Core;
    ARRAY_FIND_INDEX_RAW = 2_258 => array_find_index_builtin in Core;
    ARRAY_FIND_LAST_RAW = 2_259 => array_find_last_builtin in Core;
    ARRAY_FIND_LAST_INDEX_RAW = 2_260 => array_find_last_index_builtin in Core;
    ARRAY_TO_REVERSED_RAW = 2_261 => array_to_reversed_builtin in Core;
    ARRAY_TO_SORTED_RAW = 2_262 => array_to_sorted_builtin in Core;
    ARRAY_TO_SPLICED_RAW = 2_263 => array_to_spliced_builtin in Core;
    ARRAY_WITH_RAW = 2_264 => array_with_builtin in Core;
    ARRAY_AT_RAW = 2_265 => array_at_builtin in Core;
    ARRAY_OF_RAW = 2_266 => array_of_builtin in Core;
    ARRAY_FLAT_RAW = 2_267 => array_flat_builtin in Core;
    ARRAY_FLAT_MAP_RAW = 2_268 => array_flat_map_builtin in Core;
    ARRAY_POP_RAW = 2_269 => array_pop_builtin in Core;
    ARRAY_PUSH_RAW = 2_270 => array_push_builtin in Core;
    ARRAY_FROM_ASYNC_RAW = 2_271 => array_from_async_builtin in Core;
    OBJECT_ASSIGN_RAW = 2_272 => object_assign_builtin in Core;
    OBJECT_FROM_ENTRIES_RAW = 2_273 => object_from_entries_builtin in Core;
    OBJECT_DEFINE_GETTER_RAW = 2_274 => object_define_getter_builtin in Core;
    OBJECT_DEFINE_SETTER_RAW = 2_275 => object_define_setter_builtin in Core;
    OBJECT_LOOKUP_GETTER_RAW = 2_276 => object_lookup_getter_builtin in Core;
    OBJECT_LOOKUP_SETTER_RAW = 2_277 => object_lookup_setter_builtin in Core;
    OBJECT_PROTO_GETTER_RAW = 2_278 => object_proto_getter_builtin in Core;
    OBJECT_PROTO_SETTER_RAW = 2_279 => object_proto_setter_builtin in Core;
    OBJECT_GROUP_BY_RAW = 2_280 => object_group_by_builtin in Core;
    ABSTRACT_MODULE_SOURCE_RAW = 2_281 => abstract_module_source_builtin in Core;
    ABSTRACT_MODULE_SOURCE_TO_STRING_TAG_GETTER_RAW = 2_282 => abstract_module_source_to_string_tag_getter_builtin in Core;
    FUNCTION_SYMBOL_HAS_INSTANCE_RAW = 2_283 => function_symbol_has_instance_builtin in Core;
    REGEXP_SPECIES_GETTER_RAW = 2_284 => regexp_species_getter_builtin in Core;
    REGEXP_STRING_ITERATOR_NEXT_RAW = 2_286 => regexp_string_iterator_next_builtin in Core;
    REGEXP_UNICODE_SETS_GETTER_RAW = 2_287 => regexp_unicode_sets_getter_builtin in Core;
    STRING_SUBSTR_RAW = 2_288 => string_substr_builtin in Core;
    ESCAPE_RAW = 2_289 => escape_builtin in Core;
    UNESCAPE_RAW = 2_290 => unescape_builtin in Core;
    STRING_ANCHOR_RAW = 2_291 => string_anchor_builtin in Core;
    STRING_BIG_RAW = 2_292 => string_big_builtin in Core;
    STRING_BLINK_RAW = 2_293 => string_blink_builtin in Core;
    STRING_BOLD_RAW = 2_294 => string_bold_builtin in Core;
    STRING_FIXED_RAW = 2_295 => string_fixed_builtin in Core;
    STRING_FONTCOLOR_RAW = 2_296 => string_fontcolor_builtin in Core;
    STRING_FONTSIZE_RAW = 2_297 => string_fontsize_builtin in Core;
    STRING_ITALICS_RAW = 2_298 => string_italics_builtin in Core;
    STRING_LINK_RAW = 2_299 => string_link_builtin in Core;
    STRING_SMALL_RAW = 2_300 => string_small_builtin in Core;
    STRING_STRIKE_RAW = 2_301 => string_strike_builtin in Core;
    STRING_SUB_RAW = 2_302 => string_sub_builtin in Core;
    STRING_SUP_RAW = 2_303 => string_sup_builtin in Core;
    DATE_GET_YEAR_RAW = 2_304 => date_get_year_builtin in Core;
    DATE_SET_YEAR_RAW = 2_305 => date_set_year_builtin in Core;
    REGEXP_COMPILE_RAW = 2_306 => regexp_compile_builtin in Core;
    REGEXP_LEGACY_INPUT_GETTER_RAW = 2_307 => regexp_legacy_input_getter_builtin in Core;
    REGEXP_LEGACY_INPUT_SETTER_RAW = 2_308 => regexp_legacy_input_setter_builtin in Core;
    REGEXP_LEGACY_LAST_MATCH_GETTER_RAW = 2_309 => regexp_legacy_last_match_getter_builtin in Core;
    REGEXP_LEGACY_LAST_PAREN_GETTER_RAW = 2_310 => regexp_legacy_last_paren_getter_builtin in Core;
    REGEXP_LEGACY_LEFT_CONTEXT_GETTER_RAW = 2_311 => regexp_legacy_left_context_getter_builtin in Core;
    REGEXP_LEGACY_RIGHT_CONTEXT_GETTER_RAW = 2_312 => regexp_legacy_right_context_getter_builtin in Core;
    REGEXP_LEGACY_PAREN1_GETTER_RAW = 2_313 => regexp_legacy_paren1_getter_builtin in Core;
    REGEXP_LEGACY_PAREN2_GETTER_RAW = 2_314 => regexp_legacy_paren2_getter_builtin in Core;
    REGEXP_LEGACY_PAREN3_GETTER_RAW = 2_315 => regexp_legacy_paren3_getter_builtin in Core;
    REGEXP_LEGACY_PAREN4_GETTER_RAW = 2_316 => regexp_legacy_paren4_getter_builtin in Core;
    REGEXP_LEGACY_PAREN5_GETTER_RAW = 2_317 => regexp_legacy_paren5_getter_builtin in Core;
    REGEXP_LEGACY_PAREN6_GETTER_RAW = 2_318 => regexp_legacy_paren6_getter_builtin in Core;
    REGEXP_LEGACY_PAREN7_GETTER_RAW = 2_319 => regexp_legacy_paren7_getter_builtin in Core;
    REGEXP_LEGACY_PAREN8_GETTER_RAW = 2_320 => regexp_legacy_paren8_getter_builtin in Core;
    REGEXP_LEGACY_PAREN9_GETTER_RAW = 2_321 => regexp_legacy_paren9_getter_builtin in Core;
    PROMISE_RAW = 3_101 => promise_builtin in Completion;
    PROMISE_THEN_RAW = 3_102 => promise_then_builtin in Completion;
    PROMISE_CATCH_RAW = 3_103 => promise_catch_builtin in Completion;
    PROMISE_FINALLY_RAW = 3_104 => promise_finally_builtin in Completion;
    PROMISE_RESOLVE_RAW = 3_105 => promise_resolve_builtin in Completion;
    PROMISE_REJECT_RAW = 3_106 => promise_reject_builtin in Completion;
    PROMISE_ALL_RAW = 3_107 => promise_all_builtin in Completion;
    PROMISE_ALL_SETTLED_RAW = 3_108 => promise_all_settled_builtin in Completion;
    PROMISE_RACE_RAW = 3_109 => promise_race_builtin in Completion;
    PROMISE_ANY_RAW = 3_110 => promise_any_builtin in Completion;
    PROMISE_SPECIES_GETTER_RAW = 3_111 => promise_species_getter_builtin in Completion;
    PROMISE_CAPABILITY_EXECUTOR_RAW = 3_112 => promise_capability_executor_builtin in Completion;
    PROMISE_RESOLVE_FUNCTION_RAW = 3_113 => promise_resolve_function_builtin in Completion;
    PROMISE_REJECT_FUNCTION_RAW = 3_114 => promise_reject_function_builtin in Completion;
    PROMISE_ALL_RESOLVE_ELEMENT_RAW = 3_115 => promise_all_resolve_element_builtin in Completion;
    PROMISE_ALL_SETTLED_RESOLVE_ELEMENT_RAW = 3_116 => promise_all_settled_resolve_element_builtin in Completion;
    PROMISE_ALL_SETTLED_REJECT_ELEMENT_RAW = 3_117 => promise_all_settled_reject_element_builtin in Completion;
    PROMISE_ANY_REJECT_ELEMENT_RAW = 3_118 => promise_any_reject_element_builtin in Completion;
    AGGREGATE_ERROR_RAW = 3_119 => aggregate_error_builtin in Completion;
    PROMISE_FINALLY_FUNCTION_RAW = 3_120 => promise_finally_function_builtin in Completion;
    PROMISE_FINALLY_CONTINUATION_RAW = 3_588 => promise_finally_continuation_builtin in Completion;
    ASYNC_FUNCTION_RAW = 3_121 => async_function_builtin in Completion;
    ASYNC_GENERATOR_FUNCTION_RAW = 3_122 => async_generator_function_builtin in Completion;
    ASYNC_GENERATOR_NEXT_RAW = 3_123 => async_generator_next_builtin in Completion;
    ASYNC_GENERATOR_RETURN_RAW = 3_124 => async_generator_return_builtin in Completion;
    ASYNC_GENERATOR_THROW_RAW = 3_125 => async_generator_throw_builtin in Completion;
    ARRAY_BUFFER_RAW = 3_126 => array_buffer_builtin in Completion;
    ARRAY_BUFFER_IS_VIEW_RAW = 3_127 => array_buffer_is_view_builtin in Completion;
    ARRAY_BUFFER_BYTE_LENGTH_GETTER_RAW = 3_128 => array_buffer_byte_length_getter_builtin in Completion;
    ARRAY_BUFFER_SLICE_RAW = 3_129 => array_buffer_slice_builtin in Completion;
    ARRAY_BUFFER_RESIZE_RAW = 3_508 => array_buffer_resize_builtin in Completion;
    ARRAY_BUFFER_DETACHED_GETTER_RAW = 3_578 => array_buffer_detached_getter_builtin in Completion;
    ARRAY_BUFFER_MAX_BYTE_LENGTH_GETTER_RAW = 3_579 => array_buffer_max_byte_length_getter_builtin in Completion;
    ARRAY_BUFFER_RESIZABLE_GETTER_RAW = 3_580 => array_buffer_resizable_getter_builtin in Completion;
    ARRAY_BUFFER_TRANSFER_RAW = 3_581 => array_buffer_transfer_builtin in Completion;
    ARRAY_BUFFER_TRANSFER_TO_FIXED_LENGTH_RAW = 3_582 => array_buffer_transfer_to_fixed_length_builtin in Completion;
    JSON_PARSE_RAW = 3_130 => json_parse_builtin in Completion;
    JSON_STRINGIFY_RAW = 3_131 => json_stringify_builtin in Completion;
    DATA_VIEW_RAW = 3_132 => data_view_builtin in Completion;
    DATA_VIEW_BUFFER_GETTER_RAW = 3_133 => data_view_buffer_getter_builtin in Completion;
    DATA_VIEW_BYTE_LENGTH_GETTER_RAW = 3_134 => data_view_byte_length_getter_builtin in Completion;
    DATA_VIEW_BYTE_OFFSET_GETTER_RAW = 3_135 => data_view_byte_offset_getter_builtin in Completion;
    DATA_VIEW_GET_UINT8_RAW = 3_136 => data_view_get_uint8_builtin in Completion;
    DATA_VIEW_SET_UINT8_RAW = 3_137 => data_view_set_uint8_builtin in Completion;
    UINT8_ARRAY_RAW = 3_138 => uint8_array_builtin in Completion;
    UINT8_ARRAY_BUFFER_GETTER_RAW = 3_139 => uint8_array_buffer_getter_builtin in Completion;
    UINT8_ARRAY_BYTE_LENGTH_GETTER_RAW = 3_140 => uint8_array_byte_length_getter_builtin in Completion;
    UINT8_ARRAY_BYTE_OFFSET_GETTER_RAW = 3_141 => uint8_array_byte_offset_getter_builtin in Completion;
    UINT8_ARRAY_LENGTH_GETTER_RAW = 3_142 => uint8_array_length_getter_builtin in Completion;
    UINT8_ARRAY_VALUES_RAW = 3_143 => uint8_array_values_builtin in Completion;
    UINT8_ARRAY_KEYS_RAW = 3_144 => uint8_array_keys_builtin in Completion;
    UINT8_ARRAY_ENTRIES_RAW = 3_145 => uint8_array_entries_builtin in Completion;
    UINT8_ARRAY_SET_RAW = 3_146 => uint8_array_set_builtin in Completion;
    UINT8_ARRAY_SUBARRAY_RAW = 3_147 => uint8_array_subarray_builtin in Completion;
    DATA_VIEW_GET_INT8_RAW = 3_148 => data_view_get_int8_builtin in Completion;
    DATA_VIEW_SET_INT8_RAW = 3_149 => data_view_set_int8_builtin in Completion;
    UINT8_ARRAY_SLICE_RAW = 3_150 => uint8_array_slice_builtin in Completion;
    DATA_VIEW_GET_UINT16_RAW = 3_151 => data_view_get_uint16_builtin in Completion;
    DATA_VIEW_SET_UINT16_RAW = 3_152 => data_view_set_uint16_builtin in Completion;
    DATA_VIEW_GET_INT16_RAW = 3_153 => data_view_get_int16_builtin in Completion;
    DATA_VIEW_SET_INT16_RAW = 3_154 => data_view_set_int16_builtin in Completion;
    DATA_VIEW_GET_UINT32_RAW = 3_155 => data_view_get_uint32_builtin in Completion;
    DATA_VIEW_SET_UINT32_RAW = 3_156 => data_view_set_uint32_builtin in Completion;
    DATA_VIEW_GET_INT32_RAW = 3_157 => data_view_get_int32_builtin in Completion;
    DATA_VIEW_SET_INT32_RAW = 3_158 => data_view_set_int32_builtin in Completion;
    DATA_VIEW_GET_FLOAT32_RAW = 3_159 => data_view_get_float32_builtin in Completion;
    DATA_VIEW_SET_FLOAT32_RAW = 3_160 => data_view_set_float32_builtin in Completion;
    DATA_VIEW_GET_FLOAT64_RAW = 3_161 => data_view_get_float64_builtin in Completion;
    DATA_VIEW_SET_FLOAT64_RAW = 3_162 => data_view_set_float64_builtin in Completion;
    INT8_ARRAY_RAW = 3_163 => int8_array_builtin in Completion;
    UINT16_ARRAY_RAW = 3_164 => uint16_array_builtin in Completion;
    INT16_ARRAY_RAW = 3_165 => int16_array_builtin in Completion;
    INT32_ARRAY_RAW = 3_166 => int32_array_builtin in Completion;
    UINT32_ARRAY_RAW = 3_167 => uint32_array_builtin in Completion;
    FLOAT32_ARRAY_RAW = 3_168 => float32_array_builtin in Completion;
    FLOAT64_ARRAY_RAW = 3_169 => float64_array_builtin in Completion;
    UINT8_CLAMPED_ARRAY_RAW = 3_170 => uint8_clamped_array_builtin in Completion;
    BIG_UINT64_ARRAY_RAW = 3_171 => big_uint64_array_builtin in Completion;
    BIG_INT64_ARRAY_RAW = 3_172 => big_int64_array_builtin in Completion;
    FLOAT16_ARRAY_RAW = 3_590 => float16_array_builtin in Completion;
    MAP_RAW = 3_173 => map_builtin in Completion;
    MAP_GET_RAW = 3_174 => map_get_builtin in Completion;
    MAP_SET_RAW = 3_175 => map_set_builtin in Completion;
    MAP_HAS_RAW = 3_176 => map_has_builtin in Completion;
    MAP_DELETE_RAW = 3_177 => map_delete_builtin in Completion;
    MAP_CLEAR_RAW = 3_178 => map_clear_builtin in Completion;
    MAP_ENTRIES_RAW = 3_179 => map_entries_builtin in Completion;
    MAP_VALUES_RAW = 3_180 => map_values_builtin in Completion;
    MAP_KEYS_RAW = 3_181 => map_keys_builtin in Completion;
    MAP_SIZE_GETTER_RAW = 3_182 => map_size_getter_builtin in Completion;
    MAP_ITERATOR_NEXT_RAW = 3_183 => map_iterator_next_builtin in Completion;
    SET_RAW = 3_184 => set_builtin in Completion;
    SET_ADD_RAW = 3_185 => set_add_builtin in Completion;
    SET_HAS_RAW = 3_186 => set_has_builtin in Completion;
    SET_DELETE_RAW = 3_187 => set_delete_builtin in Completion;
    SET_CLEAR_RAW = 3_188 => set_clear_builtin in Completion;
    SET_ENTRIES_RAW = 3_189 => set_entries_builtin in Completion;
    SET_VALUES_RAW = 3_190 => set_values_builtin in Completion;
    SET_KEYS_RAW = 3_191 => set_keys_builtin in Completion;
    SET_SIZE_GETTER_RAW = 3_192 => set_size_getter_builtin in Completion;
    SET_ITERATOR_NEXT_RAW = 3_193 => set_iterator_next_builtin in Completion;
    MAP_FOR_EACH_RAW = 3_194 => map_for_each_builtin in Completion;
    SET_FOR_EACH_RAW = 3_195 => set_for_each_builtin in Completion;
    TYPED_ARRAY_RAW = 3_196 => typed_array_builtin in Completion;
    TYPED_ARRAY_AT_RAW = 3_197 => typed_array_at_builtin in Completion;
    TYPED_ARRAY_TO_STRING_TAG_GETTER_RAW = 3_198 => typed_array_to_string_tag_getter_builtin in Completion;
    TYPED_ARRAY_FROM_RAW = 3_199 => typed_array_from_builtin in Completion;
    TYPED_ARRAY_OF_RAW = 3_200 => typed_array_of_builtin in Completion;
    TYPED_ARRAY_EVERY_RAW = 3_201 => typed_array_every_builtin in Completion;
    TYPED_ARRAY_SOME_RAW = 3_202 => typed_array_some_builtin in Completion;
    TYPED_ARRAY_FIND_RAW = 3_203 => typed_array_find_builtin in Completion;
    TYPED_ARRAY_FIND_INDEX_RAW = 3_204 => typed_array_find_index_builtin in Completion;
    TYPED_ARRAY_FIND_LAST_RAW = 3_205 => typed_array_find_last_builtin in Completion;
    TYPED_ARRAY_FIND_LAST_INDEX_RAW = 3_206 => typed_array_find_last_index_builtin in Completion;
    TYPED_ARRAY_FILL_RAW = 3_207 => typed_array_fill_builtin in Completion;
    TYPED_ARRAY_COPY_WITHIN_RAW = 3_208 => typed_array_copy_within_builtin in Completion;
    TYPED_ARRAY_INCLUDES_RAW = 3_209 => typed_array_includes_builtin in Completion;
    TYPED_ARRAY_INDEX_OF_RAW = 3_210 => typed_array_index_of_builtin in Completion;
    TYPED_ARRAY_LAST_INDEX_OF_RAW = 3_211 => typed_array_last_index_of_builtin in Completion;
    TYPED_ARRAY_FILTER_RAW = 3_212 => typed_array_filter_builtin in Completion;
    TYPED_ARRAY_FOR_EACH_RAW = 3_213 => typed_array_for_each_builtin in Completion;
    TYPED_ARRAY_JOIN_RAW = 3_214 => typed_array_join_builtin in Completion;
    TYPED_ARRAY_MAP_RAW = 3_215 => typed_array_map_builtin in Completion;
    TYPED_ARRAY_REDUCE_RAW = 3_216 => typed_array_reduce_builtin in Completion;
    TYPED_ARRAY_REDUCE_RIGHT_RAW = 3_217 => typed_array_reduce_right_builtin in Completion;
    TYPED_ARRAY_REVERSE_RAW = 3_218 => typed_array_reverse_builtin in Completion;
    TYPED_ARRAY_TO_REVERSED_RAW = 3_219 => typed_array_to_reversed_builtin in Completion;
    TYPED_ARRAY_TO_SORTED_RAW = 3_220 => typed_array_to_sorted_builtin in Completion;
    TYPED_ARRAY_WITH_RAW = 3_221 => typed_array_with_builtin in Completion;
    TYPED_ARRAY_SORT_RAW = 3_222 => typed_array_sort_builtin in Completion;
    TYPED_ARRAY_TO_LOCALE_STRING_RAW = 3_223 => typed_array_to_locale_string_builtin in Completion;
    TYPED_ARRAY_TO_STRING_RAW = 3_224 => typed_array_to_string_builtin in Completion;
    JSON_RAW_JSON_RAW = 3_225 => json_raw_json_builtin in Completion;
    JSON_IS_RAW_JSON_RAW = 3_226 => json_is_raw_json_builtin in Completion;
    PROXY_RAW = 3_227 => proxy_builtin in Completion;
    PROXY_REVOCABLE_RAW = 3_228 => proxy_revocable_builtin in Completion;
    PROXY_REVOKE_RAW = 3_229 => proxy_revoke_builtin in Completion;
    REFLECT_APPLY_RAW = 3_230 => reflect_apply_builtin in Completion;
    REFLECT_CONSTRUCT_RAW = 3_231 => reflect_construct_builtin in Completion;
    REFLECT_DEFINE_PROPERTY_RAW = 3_232 => reflect_define_property_builtin in Completion;
    REFLECT_DELETE_PROPERTY_RAW = 3_233 => reflect_delete_property_builtin in Completion;
    REFLECT_GET_RAW = 3_234 => reflect_get_builtin in Completion;
    REFLECT_GET_OWN_PROPERTY_DESCRIPTOR_RAW = 3_235 => reflect_get_own_property_descriptor_builtin in Completion;
    REFLECT_GET_PROTOTYPE_OF_RAW = 3_236 => reflect_get_prototype_of_builtin in Completion;
    REFLECT_HAS_RAW = 3_237 => reflect_has_builtin in Completion;
    REFLECT_IS_EXTENSIBLE_RAW = 3_238 => reflect_is_extensible_builtin in Completion;
    REFLECT_OWN_KEYS_RAW = 3_239 => reflect_own_keys_builtin in Completion;
    REFLECT_PREVENT_EXTENSIONS_RAW = 3_240 => reflect_prevent_extensions_builtin in Completion;
    REFLECT_SET_RAW = 3_241 => reflect_set_builtin in Completion;
    REFLECT_SET_PROTOTYPE_OF_RAW = 3_242 => reflect_set_prototype_of_builtin in Completion;
    WEAK_MAP_RAW = 3_243 => weak_map_builtin in Completion;
    WEAK_MAP_GET_RAW = 3_244 => weak_map_get_builtin in Completion;
    WEAK_MAP_SET_RAW = 3_245 => weak_map_set_builtin in Completion;
    WEAK_MAP_HAS_RAW = 3_246 => weak_map_has_builtin in Completion;
    WEAK_MAP_DELETE_RAW = 3_247 => weak_map_delete_builtin in Completion;
    WEAK_SET_RAW = 3_248 => weak_set_builtin in Completion;
    WEAK_SET_ADD_RAW = 3_249 => weak_set_add_builtin in Completion;
    WEAK_SET_HAS_RAW = 3_250 => weak_set_has_builtin in Completion;
    WEAK_SET_DELETE_RAW = 3_251 => weak_set_delete_builtin in Completion;
    WEAK_REF_RAW = 3_252 => weak_ref_builtin in Completion;
    WEAK_REF_DEREF_RAW = 3_253 => weak_ref_deref_builtin in Completion;
    FINALIZATION_REGISTRY_RAW = 3_254 => finalization_registry_builtin in Completion;
    FINALIZATION_REGISTRY_REGISTER_RAW = 3_255 => finalization_registry_register_builtin in Completion;
    FINALIZATION_REGISTRY_UNREGISTER_RAW = 3_256 => finalization_registry_unregister_builtin in Completion;
    SHARED_ARRAY_BUFFER_RAW = 3_257 => shared_array_buffer_builtin in Completion;
    SHARED_ARRAY_BUFFER_BYTE_LENGTH_GETTER_RAW = 3_258 => shared_array_buffer_byte_length_getter_builtin in Completion;
    SHARED_ARRAY_BUFFER_SLICE_RAW = 3_259 => shared_array_buffer_slice_builtin in Completion;
    SHARED_ARRAY_BUFFER_GROW_RAW = 3_583 => shared_array_buffer_grow_builtin in Completion;
    SHARED_ARRAY_BUFFER_GROWABLE_GETTER_RAW = 3_584 => shared_array_buffer_growable_getter_builtin in Completion;
    SHARED_ARRAY_BUFFER_MAX_BYTE_LENGTH_GETTER_RAW = 3_585 => shared_array_buffer_max_byte_length_getter_builtin in Completion;
    ATOMICS_LOAD_RAW = 3_260 => atomics_load_builtin in Completion;
    ATOMICS_STORE_RAW = 3_261 => atomics_store_builtin in Completion;
    ATOMICS_ADD_RAW = 3_262 => atomics_add_builtin in Completion;
    ATOMICS_SUB_RAW = 3_263 => atomics_sub_builtin in Completion;
    ATOMICS_AND_RAW = 3_264 => atomics_and_builtin in Completion;
    ATOMICS_OR_RAW = 3_265 => atomics_or_builtin in Completion;
    ATOMICS_XOR_RAW = 3_266 => atomics_xor_builtin in Completion;
    ATOMICS_EXCHANGE_RAW = 3_267 => atomics_exchange_builtin in Completion;
    ATOMICS_COMPARE_EXCHANGE_RAW = 3_268 => atomics_compare_exchange_builtin in Completion;
    ATOMICS_NOTIFY_RAW = 3_269 => atomics_notify_builtin in Completion;
    ATOMICS_WAIT_RAW = 3_270 => atomics_wait_builtin in Completion;
    ATOMICS_WAIT_ASYNC_RAW = 3_271 => atomics_wait_async_builtin in Completion;
    ATOMICS_IS_LOCK_FREE_RAW = 3_272 => atomics_is_lock_free_builtin in Completion;
    SUPPRESSED_ERROR_RAW = 3_273 => suppressed_error_builtin in Completion;
    DISPOSABLE_STACK_RAW = 3_274 => disposable_stack_builtin in Completion;
    DISPOSABLE_STACK_USE_RAW = 3_275 => disposable_stack_use_builtin in Completion;
    DISPOSABLE_STACK_ADOPT_RAW = 3_276 => disposable_stack_adopt_builtin in Completion;
    DISPOSABLE_STACK_DEFER_RAW = 3_277 => disposable_stack_defer_builtin in Completion;
    DISPOSABLE_STACK_MOVE_RAW = 3_278 => disposable_stack_move_builtin in Completion;
    DISPOSABLE_STACK_DISPOSED_GETTER_RAW = 3_279 => disposable_stack_disposed_getter_builtin in Completion;
    DISPOSABLE_STACK_DISPOSE_RAW = 3_280 => disposable_stack_dispose_builtin in Completion;
    ASYNC_DISPOSABLE_STACK_RAW = 3_281 => async_disposable_stack_builtin in Completion;
    ASYNC_DISPOSABLE_STACK_USE_RAW = 3_282 => async_disposable_stack_use_builtin in Completion;
    ASYNC_DISPOSABLE_STACK_ADOPT_RAW = 3_283 => async_disposable_stack_adopt_builtin in Completion;
    ASYNC_DISPOSABLE_STACK_DEFER_RAW = 3_284 => async_disposable_stack_defer_builtin in Completion;
    ASYNC_DISPOSABLE_STACK_MOVE_RAW = 3_285 => async_disposable_stack_move_builtin in Completion;
    ASYNC_DISPOSABLE_STACK_DISPOSED_GETTER_RAW = 3_286 => async_disposable_stack_disposed_getter_builtin in Completion;
    ASYNC_DISPOSABLE_STACK_DISPOSE_ASYNC_RAW = 3_287 => async_disposable_stack_dispose_async_builtin in Completion;
    ASYNC_DISPOSAL_RESUME_RAW = 3_288 => async_disposal_resume_builtin in Completion;
    CREATE_SYNC_DISPOSAL_SCOPE_RAW = 3_289 => create_sync_disposal_scope_builtin in Completion;
    CREATE_ASYNC_DISPOSAL_SCOPE_RAW = 3_290 => create_async_disposal_scope_builtin in Completion;
    ADD_SYNC_DISPOSABLE_RESOURCE_RAW = 3_291 => add_sync_disposable_resource_builtin in Completion;
    ADD_ASYNC_DISPOSABLE_RESOURCE_RAW = 3_292 => add_async_disposable_resource_builtin in Completion;
    DISPOSE_SCOPE_RAW = 3_293 => dispose_scope_builtin in Completion;
    DISPOSE_SCOPE_ASYNC_RAW = 3_294 => dispose_scope_async_builtin in Completion;
    TEMPORAL_INSTANT_RAW = 3_295 => temporal_instant_builtin in Completion;
    TEMPORAL_NOW_INSTANT_RAW = 3_296 => temporal_now_instant_builtin in Completion;
    TEMPORAL_NOW_TIME_ZONE_ID_RAW = 3_297 => temporal_now_time_zone_id_builtin in Completion;
    TEMPORAL_INSTANT_FROM_RAW = 3_298 => temporal_instant_from_builtin in Completion;
    TEMPORAL_INSTANT_FROM_EPOCH_NANOSECONDS_RAW = 3_299 => temporal_instant_from_epoch_nanoseconds_builtin in Completion;
    TEMPORAL_INSTANT_COMPARE_RAW = 3_300 => temporal_instant_compare_builtin in Completion;
    TEMPORAL_INSTANT_EPOCH_NANOSECONDS_GETTER_RAW = 3_301 => temporal_instant_epoch_nanoseconds_getter_builtin in Completion;
    TEMPORAL_INSTANT_EPOCH_MILLISECONDS_GETTER_RAW = 3_302 => temporal_instant_epoch_milliseconds_getter_builtin in Completion;
    TEMPORAL_INSTANT_EPOCH_SECONDS_GETTER_RAW = 3_303 => temporal_instant_epoch_seconds_getter_builtin in Completion;
    TEMPORAL_INSTANT_TO_STRING_RAW = 3_304 => temporal_instant_to_string_builtin in Completion;
    TEMPORAL_INSTANT_TO_JSON_RAW = 3_305 => temporal_instant_to_json_builtin in Completion;
    TEMPORAL_INSTANT_VALUE_OF_RAW = 3_306 => temporal_instant_value_of_builtin in Completion;
    TEMPORAL_DURATION_RAW = 3_307 => temporal_duration_builtin in Completion;
    TEMPORAL_DURATION_YEARS_GETTER_RAW = 3_308 => temporal_duration_years_getter_builtin in Completion;
    TEMPORAL_DURATION_MONTHS_GETTER_RAW = 3_309 => temporal_duration_months_getter_builtin in Completion;
    TEMPORAL_DURATION_WEEKS_GETTER_RAW = 3_310 => temporal_duration_weeks_getter_builtin in Completion;
    TEMPORAL_DURATION_DAYS_GETTER_RAW = 3_311 => temporal_duration_days_getter_builtin in Completion;
    TEMPORAL_DURATION_HOURS_GETTER_RAW = 3_312 => temporal_duration_hours_getter_builtin in Completion;
    TEMPORAL_DURATION_MINUTES_GETTER_RAW = 3_313 => temporal_duration_minutes_getter_builtin in Completion;
    TEMPORAL_DURATION_SECONDS_GETTER_RAW = 3_314 => temporal_duration_seconds_getter_builtin in Completion;
    TEMPORAL_DURATION_MILLISECONDS_GETTER_RAW = 3_315 => temporal_duration_milliseconds_getter_builtin in Completion;
    TEMPORAL_DURATION_MICROSECONDS_GETTER_RAW = 3_316 => temporal_duration_microseconds_getter_builtin in Completion;
    TEMPORAL_DURATION_NANOSECONDS_GETTER_RAW = 3_317 => temporal_duration_nanoseconds_getter_builtin in Completion;
    TEMPORAL_DURATION_SIGN_GETTER_RAW = 3_318 => temporal_duration_sign_getter_builtin in Completion;
    TEMPORAL_DURATION_BLANK_GETTER_RAW = 3_319 => temporal_duration_blank_getter_builtin in Completion;
    TEMPORAL_DURATION_TO_STRING_RAW = 3_320 => temporal_duration_to_string_builtin in Completion;
    TEMPORAL_DURATION_VALUE_OF_RAW = 3_321 => temporal_duration_value_of_builtin in Completion;
    TEMPORAL_DURATION_FROM_RAW = 3_322 => temporal_duration_from_builtin in Completion;
    TEMPORAL_PLAIN_DATE_RAW = 3_323 => temporal_plain_date_builtin in Completion;
    TEMPORAL_PLAIN_DATE_YEAR_GETTER_RAW = 3_324 => temporal_plain_date_year_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_MONTH_GETTER_RAW = 3_325 => temporal_plain_date_month_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_MONTH_CODE_GETTER_RAW = 3_326 => temporal_plain_date_month_code_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_DAY_GETTER_RAW = 3_327 => temporal_plain_date_day_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_CALENDAR_ID_GETTER_RAW = 3_328 => temporal_plain_date_calendar_id_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TO_STRING_RAW = 3_329 => temporal_plain_date_to_string_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TO_JSON_RAW = 3_330 => temporal_plain_date_to_json_builtin in Completion;
    TEMPORAL_PLAIN_DATE_VALUE_OF_RAW = 3_331 => temporal_plain_date_value_of_builtin in Completion;
    TEMPORAL_PLAIN_DATE_FROM_RAW = 3_332 => temporal_plain_date_from_builtin in Completion;
    TEMPORAL_PLAIN_TIME_RAW = 3_333 => temporal_plain_time_builtin in Completion;
    TEMPORAL_PLAIN_TIME_HOUR_GETTER_RAW = 3_334 => temporal_plain_time_hour_getter_builtin in Completion;
    TEMPORAL_PLAIN_TIME_MINUTE_GETTER_RAW = 3_335 => temporal_plain_time_minute_getter_builtin in Completion;
    TEMPORAL_PLAIN_TIME_SECOND_GETTER_RAW = 3_336 => temporal_plain_time_second_getter_builtin in Completion;
    TEMPORAL_PLAIN_TIME_MILLISECOND_GETTER_RAW = 3_337 => temporal_plain_time_millisecond_getter_builtin in Completion;
    TEMPORAL_PLAIN_TIME_MICROSECOND_GETTER_RAW = 3_338 => temporal_plain_time_microsecond_getter_builtin in Completion;
    TEMPORAL_PLAIN_TIME_NANOSECOND_GETTER_RAW = 3_339 => temporal_plain_time_nanosecond_getter_builtin in Completion;
    TEMPORAL_PLAIN_TIME_TO_STRING_RAW = 3_340 => temporal_plain_time_to_string_builtin in Completion;
    TEMPORAL_PLAIN_TIME_TO_JSON_RAW = 3_341 => temporal_plain_time_to_json_builtin in Completion;
    TEMPORAL_PLAIN_TIME_VALUE_OF_RAW = 3_342 => temporal_plain_time_value_of_builtin in Completion;
    TEMPORAL_PLAIN_TIME_FROM_RAW = 3_343 => temporal_plain_time_from_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_RAW = 3_344 => temporal_plain_date_time_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_YEAR_GETTER_RAW = 3_345 => temporal_plain_date_time_year_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_MONTH_GETTER_RAW = 3_346 => temporal_plain_date_time_month_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_MONTH_CODE_GETTER_RAW = 3_347 => temporal_plain_date_time_month_code_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_DAY_GETTER_RAW = 3_348 => temporal_plain_date_time_day_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_HOUR_GETTER_RAW = 3_349 => temporal_plain_date_time_hour_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_MINUTE_GETTER_RAW = 3_350 => temporal_plain_date_time_minute_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_SECOND_GETTER_RAW = 3_351 => temporal_plain_date_time_second_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_MILLISECOND_GETTER_RAW = 3_352 => temporal_plain_date_time_millisecond_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_MICROSECOND_GETTER_RAW = 3_353 => temporal_plain_date_time_microsecond_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_NANOSECOND_GETTER_RAW = 3_354 => temporal_plain_date_time_nanosecond_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_CALENDAR_ID_GETTER_RAW = 3_355 => temporal_plain_date_time_calendar_id_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_TO_STRING_RAW = 3_356 => temporal_plain_date_time_to_string_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_TO_JSON_RAW = 3_357 => temporal_plain_date_time_to_json_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_VALUE_OF_RAW = 3_358 => temporal_plain_date_time_value_of_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_FROM_RAW = 3_359 => temporal_plain_date_time_from_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_RAW = 3_360 => temporal_plain_year_month_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_YEAR_GETTER_RAW = 3_361 => temporal_plain_year_month_year_getter_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_MONTH_GETTER_RAW = 3_362 => temporal_plain_year_month_month_getter_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_MONTH_CODE_GETTER_RAW = 3_363 => temporal_plain_year_month_month_code_getter_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_CALENDAR_ID_GETTER_RAW = 3_364 => temporal_plain_year_month_calendar_id_getter_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_TO_STRING_RAW = 3_365 => temporal_plain_year_month_to_string_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_TO_JSON_RAW = 3_366 => temporal_plain_year_month_to_json_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_VALUE_OF_RAW = 3_367 => temporal_plain_year_month_value_of_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_FROM_RAW = 3_368 => temporal_plain_year_month_from_builtin in Completion;
    TEMPORAL_PLAIN_MONTH_DAY_RAW = 3_369 => temporal_plain_month_day_builtin in Completion;
    TEMPORAL_PLAIN_MONTH_DAY_MONTH_CODE_GETTER_RAW = 3_370 => temporal_plain_month_day_month_code_getter_builtin in Completion;
    TEMPORAL_PLAIN_MONTH_DAY_DAY_GETTER_RAW = 3_371 => temporal_plain_month_day_day_getter_builtin in Completion;
    TEMPORAL_PLAIN_MONTH_DAY_CALENDAR_ID_GETTER_RAW = 3_372 => temporal_plain_month_day_calendar_id_getter_builtin in Completion;
    TEMPORAL_PLAIN_MONTH_DAY_TO_STRING_RAW = 3_373 => temporal_plain_month_day_to_string_builtin in Completion;
    TEMPORAL_PLAIN_MONTH_DAY_TO_JSON_RAW = 3_374 => temporal_plain_month_day_to_json_builtin in Completion;
    TEMPORAL_PLAIN_MONTH_DAY_VALUE_OF_RAW = 3_375 => temporal_plain_month_day_value_of_builtin in Completion;
    TEMPORAL_PLAIN_MONTH_DAY_FROM_RAW = 3_376 => temporal_plain_month_day_from_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_RAW = 3_377 => temporal_zoned_date_time_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_YEAR_GETTER_RAW = 3_378 => temporal_zoned_date_time_year_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_MONTH_GETTER_RAW = 3_379 => temporal_zoned_date_time_month_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_MONTH_CODE_GETTER_RAW = 3_380 => temporal_zoned_date_time_month_code_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_DAY_GETTER_RAW = 3_381 => temporal_zoned_date_time_day_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_HOUR_GETTER_RAW = 3_382 => temporal_zoned_date_time_hour_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_MINUTE_GETTER_RAW = 3_383 => temporal_zoned_date_time_minute_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_SECOND_GETTER_RAW = 3_384 => temporal_zoned_date_time_second_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_MILLISECOND_GETTER_RAW = 3_385 => temporal_zoned_date_time_millisecond_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_MICROSECOND_GETTER_RAW = 3_386 => temporal_zoned_date_time_microsecond_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_NANOSECOND_GETTER_RAW = 3_387 => temporal_zoned_date_time_nanosecond_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_GETTER_RAW = 3_388 => temporal_zoned_date_time_epoch_nanoseconds_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_EPOCH_MILLISECONDS_GETTER_RAW = 3_389 => temporal_zoned_date_time_epoch_milliseconds_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_ID_GETTER_RAW = 3_390 => temporal_zoned_date_time_time_zone_id_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_CALENDAR_ID_GETTER_RAW = 3_391 => temporal_zoned_date_time_calendar_id_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_OFFSET_GETTER_RAW = 3_392 => temporal_zoned_date_time_offset_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_OFFSET_NANOSECONDS_GETTER_RAW = 3_393 => temporal_zoned_date_time_offset_nanoseconds_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_TO_STRING_RAW = 3_394 => temporal_zoned_date_time_to_string_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_TO_JSON_RAW = 3_395 => temporal_zoned_date_time_to_json_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_VALUE_OF_RAW = 3_396 => temporal_zoned_date_time_value_of_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_FROM_RAW = 3_397 => temporal_zoned_date_time_from_builtin in Completion;
    TEMPORAL_NOW_PLAIN_DATE_ISO_RAW = 3_398 => temporal_now_plain_date_iso_builtin in Completion;
    TEMPORAL_NOW_PLAIN_TIME_ISO_RAW = 3_399 => temporal_now_plain_time_iso_builtin in Completion;
    TEMPORAL_NOW_PLAIN_DATE_TIME_ISO_RAW = 3_400 => temporal_now_plain_date_time_iso_builtin in Completion;
    TEMPORAL_NOW_ZONED_DATE_TIME_ISO_RAW = 3_401 => temporal_now_zoned_date_time_iso_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_TO_INSTANT_RAW = 3_402 => temporal_zoned_date_time_to_instant_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_TO_PLAIN_DATE_TIME_RAW = 3_403 => temporal_zoned_date_time_to_plain_date_time_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_TO_PLAIN_DATE_RAW = 3_404 => temporal_zoned_date_time_to_plain_date_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_TO_PLAIN_TIME_RAW = 3_405 => temporal_zoned_date_time_to_plain_time_builtin in Completion;
    TEMPORAL_INSTANT_TO_ZONED_DATE_TIME_ISO_RAW = 3_406 => temporal_instant_to_zoned_date_time_iso_builtin in Completion;
    TEMPORAL_PLAIN_DATE_COMPARE_RAW = 3_407 => temporal_plain_date_compare_builtin in Completion;
    TEMPORAL_PLAIN_TIME_COMPARE_RAW = 3_408 => temporal_plain_time_compare_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_COMPARE_RAW = 3_409 => temporal_plain_date_time_compare_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_COMPARE_RAW = 3_410 => temporal_plain_year_month_compare_builtin in Completion;
    TEMPORAL_PLAIN_DATE_EQUALS_RAW = 3_411 => temporal_plain_date_equals_builtin in Completion;
    TEMPORAL_PLAIN_TIME_EQUALS_RAW = 3_412 => temporal_plain_time_equals_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_EQUALS_RAW = 3_413 => temporal_plain_date_time_equals_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_EQUALS_RAW = 3_414 => temporal_plain_year_month_equals_builtin in Completion;
    TEMPORAL_PLAIN_MONTH_DAY_EQUALS_RAW = 3_415 => temporal_plain_month_day_equals_builtin in Completion;
    TEMPORAL_INSTANT_EQUALS_RAW = 3_416 => temporal_instant_equals_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_EQUALS_RAW = 3_417 => temporal_zoned_date_time_equals_builtin in Completion;
    TEMPORAL_DURATION_TO_JSON_RAW = 3_418 => temporal_duration_to_json_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_TO_PLAIN_DATE_RAW = 3_419 => temporal_plain_date_time_to_plain_date_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_TO_PLAIN_TIME_RAW = 3_420 => temporal_plain_date_time_to_plain_time_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TO_PLAIN_DATE_TIME_RAW = 3_421 => temporal_plain_date_to_plain_date_time_builtin in Completion;
    TEMPORAL_PLAIN_TIME_TO_PLAIN_DATE_TIME_RAW = 3_422 => temporal_plain_time_to_plain_date_time_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_TO_PLAIN_DATE_RAW = 3_423 => temporal_plain_year_month_to_plain_date_builtin in Completion;
    TEMPORAL_PLAIN_MONTH_DAY_TO_PLAIN_DATE_RAW = 3_424 => temporal_plain_month_day_to_plain_date_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TO_PLAIN_YEAR_MONTH_RAW = 3_425 => temporal_plain_date_to_plain_year_month_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TO_PLAIN_MONTH_DAY_RAW = 3_426 => temporal_plain_date_to_plain_month_day_builtin in Completion;
    TEMPORAL_DURATION_NEGATED_RAW = 3_427 => temporal_duration_negated_builtin in Completion;
    TEMPORAL_DURATION_ABS_RAW = 3_428 => temporal_duration_abs_builtin in Completion;
    TEMPORAL_PLAIN_DATE_DAY_OF_WEEK_GETTER_RAW = 3_429 => temporal_plain_date_day_of_week_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_DAY_OF_YEAR_GETTER_RAW = 3_430 => temporal_plain_date_day_of_year_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_DAYS_IN_MONTH_GETTER_RAW = 3_431 => temporal_plain_date_days_in_month_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_DAYS_IN_YEAR_GETTER_RAW = 3_432 => temporal_plain_date_days_in_year_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_MONTHS_IN_YEAR_GETTER_RAW = 3_433 => temporal_plain_date_months_in_year_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_IN_LEAP_YEAR_GETTER_RAW = 3_434 => temporal_plain_date_in_leap_year_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_DAY_OF_WEEK_GETTER_RAW = 3_435 => temporal_plain_date_time_day_of_week_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_DAY_OF_YEAR_GETTER_RAW = 3_436 => temporal_plain_date_time_day_of_year_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_DAYS_IN_MONTH_GETTER_RAW = 3_437 => temporal_plain_date_time_days_in_month_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_DAYS_IN_YEAR_GETTER_RAW = 3_438 => temporal_plain_date_time_days_in_year_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_MONTHS_IN_YEAR_GETTER_RAW = 3_439 => temporal_plain_date_time_months_in_year_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_IN_LEAP_YEAR_GETTER_RAW = 3_440 => temporal_plain_date_time_in_leap_year_getter_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_DAYS_IN_MONTH_GETTER_RAW = 3_441 => temporal_plain_year_month_days_in_month_getter_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_DAYS_IN_YEAR_GETTER_RAW = 3_442 => temporal_plain_year_month_days_in_year_getter_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_MONTHS_IN_YEAR_GETTER_RAW = 3_443 => temporal_plain_year_month_months_in_year_getter_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_IN_LEAP_YEAR_GETTER_RAW = 3_444 => temporal_plain_year_month_in_leap_year_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_DAY_OF_WEEK_GETTER_RAW = 3_445 => temporal_zoned_date_time_day_of_week_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_DAY_OF_YEAR_GETTER_RAW = 3_446 => temporal_zoned_date_time_day_of_year_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_DAYS_IN_MONTH_GETTER_RAW = 3_447 => temporal_zoned_date_time_days_in_month_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_DAYS_IN_YEAR_GETTER_RAW = 3_448 => temporal_zoned_date_time_days_in_year_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_MONTHS_IN_YEAR_GETTER_RAW = 3_449 => temporal_zoned_date_time_months_in_year_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_IN_LEAP_YEAR_GETTER_RAW = 3_450 => temporal_zoned_date_time_in_leap_year_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_WITH_RAW = 3_451 => temporal_plain_date_with_builtin in Completion;
    TEMPORAL_PLAIN_TIME_WITH_RAW = 3_452 => temporal_plain_time_with_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_WITH_RAW = 3_453 => temporal_plain_date_time_with_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_WITH_PLAIN_TIME_RAW = 3_589 => temporal_plain_date_time_with_plain_time_builtin in Completion;
    TEMPORAL_PLAIN_DATE_WITH_CALENDAR_RAW = 3_509 => temporal_plain_date_with_calendar_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_WITH_CALENDAR_RAW = 3_510 => temporal_plain_date_time_with_calendar_builtin in Completion;
    TEMPORAL_PLAIN_DATE_DAYS_IN_WEEK_GETTER_RAW = 3_511 => temporal_plain_date_days_in_week_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_WEEK_OF_YEAR_GETTER_RAW = 3_512 => temporal_plain_date_week_of_year_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_YEAR_OF_WEEK_GETTER_RAW = 3_513 => temporal_plain_date_year_of_week_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_ERA_GETTER_RAW = 3_514 => temporal_plain_date_era_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_ERA_YEAR_GETTER_RAW = 3_515 => temporal_plain_date_era_year_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_DAYS_IN_WEEK_GETTER_RAW = 3_516 => temporal_plain_date_time_days_in_week_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_WEEK_OF_YEAR_GETTER_RAW = 3_517 => temporal_plain_date_time_week_of_year_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_YEAR_OF_WEEK_GETTER_RAW = 3_518 => temporal_plain_date_time_year_of_week_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_ERA_GETTER_RAW = 3_519 => temporal_plain_date_time_era_getter_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_ERA_YEAR_GETTER_RAW = 3_520 => temporal_plain_date_time_era_year_getter_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_ERA_GETTER_RAW = 3_521 => temporal_plain_year_month_era_getter_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_ERA_YEAR_GETTER_RAW = 3_522 => temporal_plain_year_month_era_year_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_DAYS_IN_WEEK_GETTER_RAW = 3_523 => temporal_zoned_date_time_days_in_week_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_WEEK_OF_YEAR_GETTER_RAW = 3_524 => temporal_zoned_date_time_week_of_year_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_YEAR_OF_WEEK_GETTER_RAW = 3_525 => temporal_zoned_date_time_year_of_week_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_ERA_GETTER_RAW = 3_526 => temporal_zoned_date_time_era_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_ERA_YEAR_GETTER_RAW = 3_527 => temporal_zoned_date_time_era_year_getter_builtin in Completion;
    PROMISE_TRY_RAW = 3_528 => promise_try_builtin in Completion;
    PROMISE_WITH_RESOLVERS_RAW = 3_529 => promise_with_resolvers_builtin in Completion;
    MAP_GET_OR_INSERT_RAW = 3_530 => map_get_or_insert_builtin in Completion;
    MAP_GET_OR_INSERT_COMPUTED_RAW = 3_531 => map_get_or_insert_computed_builtin in Completion;
    WEAK_MAP_GET_OR_INSERT_RAW = 3_532 => weak_map_get_or_insert_builtin in Completion;
    WEAK_MAP_GET_OR_INSERT_COMPUTED_RAW = 3_533 => weak_map_get_or_insert_computed_builtin in Completion;
    SET_UNION_RAW = 3_534 => set_union_builtin in Completion;
    SET_INTERSECTION_RAW = 3_535 => set_intersection_builtin in Completion;
    SET_DIFFERENCE_RAW = 3_536 => set_difference_builtin in Completion;
    SET_SYMMETRIC_DIFFERENCE_RAW = 3_537 => set_symmetric_difference_builtin in Completion;
    SET_IS_SUBSET_OF_RAW = 3_538 => set_is_subset_of_builtin in Completion;
    SET_IS_SUPERSET_OF_RAW = 3_539 => set_is_superset_of_builtin in Completion;
    SET_IS_DISJOINT_FROM_RAW = 3_540 => set_is_disjoint_from_builtin in Completion;
    ATOMICS_PAUSE_RAW = 3_541 => atomics_pause_builtin in Completion;
    ITERATOR_RAW = 3_542 => iterator_builtin in Completion;
    ITERATOR_FROM_RAW = 3_543 => iterator_from_builtin in Completion;
    ITERATOR_REDUCE_RAW = 3_544 => iterator_reduce_builtin in Completion;
    ITERATOR_FOR_EACH_RAW = 3_545 => iterator_for_each_builtin in Completion;
    ITERATOR_SOME_RAW = 3_546 => iterator_some_builtin in Completion;
    ITERATOR_EVERY_RAW = 3_547 => iterator_every_builtin in Completion;
    ITERATOR_FIND_RAW = 3_548 => iterator_find_builtin in Completion;
    ITERATOR_TO_ARRAY_RAW = 3_549 => iterator_to_array_builtin in Completion;
    ITERATOR_TO_STRING_TAG_GETTER_RAW = 3_550 => iterator_to_string_tag_getter_builtin in Completion;
    ITERATOR_TO_STRING_TAG_SETTER_RAW = 3_551 => iterator_to_string_tag_setter_builtin in Completion;
    ITERATOR_CONSTRUCTOR_GETTER_RAW = 3_552 => iterator_constructor_getter_builtin in Completion;
    ITERATOR_CONSTRUCTOR_SETTER_RAW = 3_553 => iterator_constructor_setter_builtin in Completion;
    ITERATOR_MAP_RAW = 3_567 => iterator_map_builtin in Completion;
    ITERATOR_HELPER_NEXT_RAW = 3_568 => iterator_helper_next_builtin in Completion;
    ITERATOR_HELPER_RETURN_RAW = 3_569 => iterator_helper_return_builtin in Completion;
    ITERATOR_FILTER_RAW = 3_570 => iterator_filter_builtin in Completion;
    ITERATOR_TAKE_RAW = 3_571 => iterator_take_builtin in Completion;
    ITERATOR_DROP_RAW = 3_572 => iterator_drop_builtin in Completion;
    ITERATOR_DISPOSE_RAW = 3_573 => iterator_dispose_builtin in Completion;
    ITERATOR_FLAT_MAP_RAW = 3_574 => iterator_flat_map_builtin in Completion;
    ITERATOR_CONCAT_RAW = 3_575 => iterator_concat_builtin in Completion;
    ITERATOR_ZIP_RAW = 3_576 => iterator_zip_builtin in Completion;
    ITERATOR_ZIP_KEYED_RAW = 3_577 => iterator_zip_keyed_builtin in Completion;
    MAP_GROUP_BY_RAW = 3_586 => map_group_by_builtin in Completion;
    ASYNC_ITERATOR_DISPOSE_RAW = 3_587 => async_iterator_dispose_builtin in Completion;
    DATA_VIEW_GET_BIG_INT64_RAW = 3_554 => data_view_get_big_int64_builtin in Completion;
    DATA_VIEW_GET_BIG_UINT64_RAW = 3_555 => data_view_get_big_uint64_builtin in Completion;
    DATA_VIEW_SET_BIG_INT64_RAW = 3_556 => data_view_set_big_int64_builtin in Completion;
    DATA_VIEW_SET_BIG_UINT64_RAW = 3_557 => data_view_set_big_uint64_builtin in Completion;
    UINT8_ARRAY_FROM_BASE64_RAW = 3_558 => uint8_array_from_base64_builtin in Completion;
    UINT8_ARRAY_FROM_HEX_RAW = 3_559 => uint8_array_from_hex_builtin in Completion;
    UINT8_ARRAY_SET_FROM_BASE64_RAW = 3_560 => uint8_array_set_from_base64_builtin in Completion;
    UINT8_ARRAY_SET_FROM_HEX_RAW = 3_561 => uint8_array_set_from_hex_builtin in Completion;
    UINT8_ARRAY_TO_BASE64_RAW = 3_562 => uint8_array_to_base64_builtin in Completion;
    UINT8_ARRAY_TO_HEX_RAW = 3_563 => uint8_array_to_hex_builtin in Completion;
    DATA_VIEW_GET_FLOAT16_RAW = 3_564 => data_view_get_float16_builtin in Completion;
    DATA_VIEW_SET_FLOAT16_RAW = 3_565 => data_view_set_float16_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_WITH_RAW = 3_454 => temporal_plain_year_month_with_builtin in Completion;
    TEMPORAL_PLAIN_MONTH_DAY_WITH_RAW = 3_455 => temporal_plain_month_day_with_builtin in Completion;
    TEMPORAL_PLAIN_DATE_ADD_RAW = 3_456 => temporal_plain_date_add_builtin in Completion;
    TEMPORAL_PLAIN_DATE_SUBTRACT_RAW = 3_457 => temporal_plain_date_subtract_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_ADD_RAW = 3_458 => temporal_plain_year_month_add_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_SUBTRACT_RAW = 3_459 => temporal_plain_year_month_subtract_builtin in Completion;
    TEMPORAL_PLAIN_TIME_ADD_RAW = 3_460 => temporal_plain_time_add_builtin in Completion;
    TEMPORAL_DURATION_COMPARE_RAW = 3_461 => temporal_duration_compare_builtin in Completion;
    TEMPORAL_DURATION_ADD_RAW = 3_462 => temporal_duration_add_builtin in Completion;
    TEMPORAL_DURATION_SUBTRACT_RAW = 3_463 => temporal_duration_subtract_builtin in Completion;
    TEMPORAL_DURATION_WITH_RAW = 3_464 => temporal_duration_with_builtin in Completion;
    TEMPORAL_INSTANT_ADD_RAW = 3_465 => temporal_instant_add_builtin in Completion;
    TEMPORAL_INSTANT_SUBTRACT_RAW = 3_466 => temporal_instant_subtract_builtin in Completion;
    TEMPORAL_INSTANT_FROM_EPOCH_MILLISECONDS_RAW = 3_467 => temporal_instant_from_epoch_milliseconds_builtin in Completion;
    TEMPORAL_DURATION_ROUND_RAW = 3_468 => temporal_duration_round_builtin in Completion;
    TEMPORAL_DURATION_TOTAL_RAW = 3_469 => temporal_duration_total_builtin in Completion;
    TEMPORAL_DURATION_TO_LOCALE_STRING_RAW = 3_470 => temporal_duration_to_locale_string_builtin in Completion;
    TEMPORAL_INSTANT_ROUND_RAW = 3_471 => temporal_instant_round_builtin in Completion;
    TEMPORAL_INSTANT_SINCE_RAW = 3_472 => temporal_instant_since_builtin in Completion;
    TEMPORAL_INSTANT_UNTIL_RAW = 3_473 => temporal_instant_until_builtin in Completion;
    TEMPORAL_PLAIN_TIME_SUBTRACT_RAW = 3_474 => temporal_plain_time_subtract_builtin in Completion;
    TEMPORAL_PLAIN_TIME_ROUND_RAW = 3_475 => temporal_plain_time_round_builtin in Completion;
    TEMPORAL_PLAIN_TIME_SINCE_RAW = 3_476 => temporal_plain_time_since_builtin in Completion;
    TEMPORAL_PLAIN_TIME_UNTIL_RAW = 3_477 => temporal_plain_time_until_builtin in Completion;
    TEMPORAL_PLAIN_DATE_SINCE_RAW = 3_478 => temporal_plain_date_since_builtin in Completion;
    TEMPORAL_PLAIN_DATE_UNTIL_RAW = 3_479 => temporal_plain_date_until_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_SINCE_RAW = 3_480 => temporal_plain_year_month_since_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_UNTIL_RAW = 3_481 => temporal_plain_year_month_until_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_ADD_RAW = 3_482 => temporal_plain_date_time_add_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_SUBTRACT_RAW = 3_483 => temporal_plain_date_time_subtract_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_ROUND_RAW = 3_484 => temporal_plain_date_time_round_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_SINCE_RAW = 3_485 => temporal_plain_date_time_since_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_UNTIL_RAW = 3_486 => temporal_plain_date_time_until_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_COMPARE_RAW = 3_487 => temporal_zoned_date_time_compare_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_ADD_RAW = 3_488 => temporal_zoned_date_time_add_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_SINCE_RAW = 3_489 => temporal_zoned_date_time_since_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_UNTIL_RAW = 3_490 => temporal_zoned_date_time_until_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_SUBTRACT_RAW = 3_491 => temporal_zoned_date_time_subtract_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_WITH_TIME_ZONE_RAW = 3_492 => temporal_zoned_date_time_with_time_zone_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_WITH_CALENDAR_RAW = 3_493 => temporal_zoned_date_time_with_calendar_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_WITH_PLAIN_TIME_RAW = 3_494 => temporal_zoned_date_time_with_plain_time_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_START_OF_DAY_RAW = 3_495 => temporal_zoned_date_time_start_of_day_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_HOURS_IN_DAY_GETTER_RAW = 3_496 => temporal_zoned_date_time_hours_in_day_getter_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_ROUND_RAW = 3_497 => temporal_zoned_date_time_round_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_WITH_RAW = 3_498 => temporal_zoned_date_time_with_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_TO_ZONED_DATE_TIME_RAW = 3_499 => temporal_plain_date_time_to_zoned_date_time_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TO_ZONED_DATE_TIME_RAW = 3_500 => temporal_plain_date_to_zoned_date_time_builtin in Completion;
    TEMPORAL_PLAIN_TIME_TO_LOCALE_STRING_RAW = 3_501 => temporal_plain_time_to_locale_string_builtin in Completion;
    TEMPORAL_INSTANT_TO_LOCALE_STRING_RAW = 3_502 => temporal_instant_to_locale_string_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TO_LOCALE_STRING_RAW = 3_503 => temporal_plain_date_to_locale_string_builtin in Completion;
    TEMPORAL_PLAIN_DATE_TIME_TO_LOCALE_STRING_RAW = 3_504 => temporal_plain_date_time_to_locale_string_builtin in Completion;
    TEMPORAL_PLAIN_YEAR_MONTH_TO_LOCALE_STRING_RAW = 3_505 => temporal_plain_year_month_to_locale_string_builtin in Completion;
    TEMPORAL_PLAIN_MONTH_DAY_TO_LOCALE_STRING_RAW = 3_506 => temporal_plain_month_day_to_locale_string_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_TO_LOCALE_STRING_RAW = 3_507 => temporal_zoned_date_time_to_locale_string_builtin in Completion;
    TEMPORAL_ZONED_DATE_TIME_GET_TIME_ZONE_TRANSITION_RAW = 3_566 => temporal_zoned_date_time_get_time_zone_transition_builtin in Completion;
}
/// First reserved builtin-entry payload for the internal helper namespace.
pub const INTERNAL_BUILTIN_NAMESPACE_START: u32 = INTERNAL_FUNCTION_CALL_RAW;

/// Last reserved builtin-entry payload for the internal helper namespace.
pub const INTERNAL_BUILTIN_NAMESPACE_END: u32 = INTERNAL_SUPER_CONSTRUCTOR_RAW;

/// First reserved builtin-entry payload for the public core builtin namespace.
pub const CORE_BUILTIN_NAMESPACE_START: u32 = BOOLEAN_RAW;

/// Last reserved builtin-entry payload for the public core builtin namespace.
pub const CORE_BUILTIN_NAMESPACE_END: u32 = REGEXP_LEGACY_PAREN9_GETTER_RAW;

/// First reserved builtin-entry payload for the public completion builtin namespace.
pub const COMPLETION_BUILTIN_NAMESPACE_START: u32 = PROMISE_RAW;

/// Last reserved builtin-entry payload for the public completion builtin namespace.
pub const COMPLETION_BUILTIN_NAMESPACE_END: u32 = FLOAT16_ARRAY_RAW;

/// Returns the registry row for a reserved builtin function ID.
#[inline]
pub fn builtin_id_registry_entry(id: BuiltinFunctionId) -> Option<BuiltinIdRegistryEntry> {
    BUILTIN_ID_REGISTRY
        .iter()
        .copied()
        .find(|entry| entry.id() == id)
}

/// Returns whether a builtin ID falls inside the reserved internal helper namespace.
#[inline]
pub const fn is_internal_builtin(id: BuiltinFunctionId) -> bool {
    let raw = id.get();
    raw >= INTERNAL_BUILTIN_NAMESPACE_START && raw <= INTERNAL_BUILTIN_NAMESPACE_END
}

/// Returns whether a builtin ID falls inside the public core builtin namespace.
#[inline]
pub const fn is_core_builtin(id: BuiltinFunctionId) -> bool {
    let raw = id.get();
    raw >= CORE_BUILTIN_NAMESPACE_START && raw <= CORE_BUILTIN_NAMESPACE_END
}

/// Returns whether a builtin ID belongs to the public Date constructor/prototype family.
#[inline]
pub const fn is_date_builtin(id: BuiltinFunctionId) -> bool {
    matches!(
        id.get(),
        DATE_RAW..=DATE_VALUE_OF_RAW
            | DATE_GET_TIMEZONE_OFFSET_RAW
            | DATE_UTC_RAW..=DATE_TO_TEMPORAL_INSTANT_RAW
            | DATE_GET_YEAR_RAW..=DATE_SET_YEAR_RAW
    )
}

/// Returns whether a builtin ID falls inside the public completion builtin namespace.
#[inline]
pub const fn is_completion_builtin(id: BuiltinFunctionId) -> bool {
    let raw = id.get();
    raw >= COMPLETION_BUILTIN_NAMESPACE_START && raw <= COMPLETION_BUILTIN_NAMESPACE_END
}

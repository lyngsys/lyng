use super::super::*;

pub(in crate::public::metadata) const PUBLIC_PRIMITIVE_BUILTIN_METADATA:
    &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        number_builtin,
        BuiltinEntryMetadata::new("Number", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        number_is_finite_builtin,
        BuiltinEntryMetadata::new("isFinite", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        number_is_integer_builtin,
        BuiltinEntryMetadata::new("isInteger", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        number_is_nan_builtin,
        BuiltinEntryMetadata::new("isNaN", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        number_is_safe_integer_builtin,
        BuiltinEntryMetadata::new("isSafeInteger", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        number_to_exponential_builtin,
        BuiltinEntryMetadata::new("toExponential", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        number_to_fixed_builtin,
        BuiltinEntryMetadata::new("toFixed", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        number_to_locale_string_builtin,
        BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        number_to_precision_builtin,
        BuiltinEntryMetadata::new("toPrecision", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        number_to_string_builtin,
        BuiltinEntryMetadata::new("toString", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        number_value_of_builtin,
        BuiltinEntryMetadata::new("valueOf", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_abs_builtin,
        BuiltinEntryMetadata::new("abs", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_acos_builtin,
        BuiltinEntryMetadata::new("acos", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_acosh_builtin,
        BuiltinEntryMetadata::new("acosh", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_asin_builtin,
        BuiltinEntryMetadata::new("asin", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_asinh_builtin,
        BuiltinEntryMetadata::new("asinh", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_atan_builtin,
        BuiltinEntryMetadata::new("atan", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_atan2_builtin,
        BuiltinEntryMetadata::new("atan2", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_atanh_builtin,
        BuiltinEntryMetadata::new("atanh", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_cbrt_builtin,
        BuiltinEntryMetadata::new("cbrt", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_ceil_builtin,
        BuiltinEntryMetadata::new("ceil", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_clz32_builtin,
        BuiltinEntryMetadata::new("clz32", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_cos_builtin,
        BuiltinEntryMetadata::new("cos", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_cosh_builtin,
        BuiltinEntryMetadata::new("cosh", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_exp_builtin,
        BuiltinEntryMetadata::new("exp", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_expm1_builtin,
        BuiltinEntryMetadata::new("expm1", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_f16round_builtin,
        BuiltinEntryMetadata::new("f16round", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_floor_builtin,
        BuiltinEntryMetadata::new("floor", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_fround_builtin,
        BuiltinEntryMetadata::new("fround", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_hypot_builtin,
        BuiltinEntryMetadata::new("hypot", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_imul_builtin,
        BuiltinEntryMetadata::new("imul", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_log_builtin,
        BuiltinEntryMetadata::new("log", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_log10_builtin,
        BuiltinEntryMetadata::new("log10", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_log1p_builtin,
        BuiltinEntryMetadata::new("log1p", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_log2_builtin,
        BuiltinEntryMetadata::new("log2", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_max_builtin,
        BuiltinEntryMetadata::new("max", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_min_builtin,
        BuiltinEntryMetadata::new("min", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_pow_builtin,
        BuiltinEntryMetadata::new("pow", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_random_builtin,
        BuiltinEntryMetadata::new("random", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_round_builtin,
        BuiltinEntryMetadata::new("round", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_sign_builtin,
        BuiltinEntryMetadata::new("sign", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_sin_builtin,
        BuiltinEntryMetadata::new("sin", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_sinh_builtin,
        BuiltinEntryMetadata::new("sinh", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_sqrt_builtin,
        BuiltinEntryMetadata::new("sqrt", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_sum_precise_builtin,
        BuiltinEntryMetadata::new("sumPrecise", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_tan_builtin,
        BuiltinEntryMetadata::new("tan", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_tanh_builtin,
        BuiltinEntryMetadata::new("tanh", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        math_trunc_builtin,
        BuiltinEntryMetadata::new("trunc", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        bigint_builtin,
        BuiltinEntryMetadata::new("BigInt", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        bigint_as_int_n_builtin,
        BuiltinEntryMetadata::new("asIntN", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        bigint_as_uint_n_builtin,
        BuiltinEntryMetadata::new("asUintN", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        bigint_to_string_builtin,
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        bigint_value_of_builtin,
        BuiltinEntryMetadata::new("valueOf", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        boolean_builtin,
        BuiltinEntryMetadata::new("Boolean", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        boolean_to_string_builtin,
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        boolean_value_of_builtin,
        BuiltinEntryMetadata::new("valueOf", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        symbol_builtin,
        BuiltinEntryMetadata::new("Symbol", 0, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        symbol_for_builtin,
        BuiltinEntryMetadata::new("for", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        symbol_key_for_builtin,
        BuiltinEntryMetadata::new("keyFor", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        symbol_to_string_builtin,
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        symbol_value_of_builtin,
        BuiltinEntryMetadata::new("valueOf", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        symbol_to_primitive_builtin,
        BuiltinEntryMetadata::new("[Symbol.toPrimitive]", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        array_species_getter_builtin,
        BuiltinEntryMetadata::new("get [Symbol.species]", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        symbol_description_getter_builtin,
        BuiltinEntryMetadata::new("get description", 0, false, false),
    ),
];

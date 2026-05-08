use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    boolean_builtin => super::super::BOOLEAN_RAW;
    boolean_to_string_builtin => super::super::BOOLEAN_TO_STRING_RAW;
    boolean_value_of_builtin => super::super::BOOLEAN_VALUE_OF_RAW;
    symbol_builtin => super::super::SYMBOL_RAW;
    symbol_for_builtin => super::super::SYMBOL_FOR_RAW;
    symbol_key_for_builtin => super::super::SYMBOL_KEY_FOR_RAW;
    symbol_to_string_builtin => super::super::SYMBOL_TO_STRING_RAW;
    symbol_value_of_builtin => super::super::SYMBOL_VALUE_OF_RAW;
    symbol_to_primitive_builtin => super::super::SYMBOL_TO_PRIMITIVE_RAW;
    symbol_description_getter_builtin => super::super::SYMBOL_DESCRIPTION_GETTER_RAW;
    number_builtin => super::super::NUMBER_RAW;
    number_is_finite_builtin => super::super::NUMBER_IS_FINITE_RAW;
    number_is_integer_builtin => super::super::NUMBER_IS_INTEGER_RAW;
    number_is_nan_builtin => super::super::NUMBER_IS_NAN_RAW;
    number_is_safe_integer_builtin => super::super::NUMBER_IS_SAFE_INTEGER_RAW;
    number_to_string_builtin => super::super::NUMBER_TO_STRING_RAW;
    number_value_of_builtin => super::super::NUMBER_VALUE_OF_RAW;
    number_to_exponential_builtin => super::super::NUMBER_TO_EXPONENTIAL_RAW;
    number_to_fixed_builtin => super::super::NUMBER_TO_FIXED_RAW;
    number_to_locale_string_builtin => super::super::NUMBER_TO_LOCALE_STRING_RAW;
    number_to_precision_builtin => super::super::NUMBER_TO_PRECISION_RAW;
    bigint_builtin => super::super::BIGINT_RAW;
    bigint_to_string_builtin => super::super::BIGINT_TO_STRING_RAW;
    bigint_value_of_builtin => super::super::BIGINT_VALUE_OF_RAW;
    bigint_as_int_n_builtin => super::super::BIGINT_AS_INT_N_RAW;
    bigint_as_uint_n_builtin => super::super::BIGINT_AS_UINT_N_RAW;
}

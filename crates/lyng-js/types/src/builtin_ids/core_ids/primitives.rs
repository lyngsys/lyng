use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn boolean_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::BOOLEAN_RAW)
}

#[inline]
pub const fn boolean_to_string_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::BOOLEAN_TO_STRING_RAW)
}

#[inline]
pub const fn boolean_value_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::BOOLEAN_VALUE_OF_RAW)
}

#[inline]
pub const fn symbol_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SYMBOL_RAW)
}

#[inline]
pub const fn symbol_for_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SYMBOL_FOR_RAW)
}

#[inline]
pub const fn symbol_key_for_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SYMBOL_KEY_FOR_RAW)
}

#[inline]
pub const fn symbol_to_string_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SYMBOL_TO_STRING_RAW)
}

#[inline]
pub const fn symbol_value_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SYMBOL_VALUE_OF_RAW)
}

#[inline]
pub const fn symbol_to_primitive_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SYMBOL_TO_PRIMITIVE_RAW)
}

#[inline]
pub const fn symbol_description_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SYMBOL_DESCRIPTION_GETTER_RAW)
}

#[inline]
pub const fn number_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::NUMBER_RAW)
}

#[inline]
pub const fn number_is_finite_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::NUMBER_IS_FINITE_RAW)
}

#[inline]
pub const fn number_is_integer_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::NUMBER_IS_INTEGER_RAW)
}

#[inline]
pub const fn number_is_nan_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::NUMBER_IS_NAN_RAW)
}

#[inline]
pub const fn number_is_safe_integer_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::NUMBER_IS_SAFE_INTEGER_RAW)
}

#[inline]
pub const fn number_to_string_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::NUMBER_TO_STRING_RAW)
}

#[inline]
pub const fn number_value_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::NUMBER_VALUE_OF_RAW)
}

#[inline]
pub const fn number_to_exponential_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::NUMBER_TO_EXPONENTIAL_RAW)
}

#[inline]
pub const fn number_to_fixed_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::NUMBER_TO_FIXED_RAW)
}

#[inline]
pub const fn number_to_locale_string_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::NUMBER_TO_LOCALE_STRING_RAW)
}

#[inline]
pub const fn number_to_precision_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::NUMBER_TO_PRECISION_RAW)
}

#[inline]
pub const fn bigint_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::BIGINT_RAW)
}

#[inline]
pub const fn bigint_to_string_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::BIGINT_TO_STRING_RAW)
}

#[inline]
pub const fn bigint_value_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::BIGINT_VALUE_OF_RAW)
}

#[inline]
pub const fn bigint_as_int_n_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::BIGINT_AS_INT_N_RAW)
}

#[inline]
pub const fn bigint_as_uint_n_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::BIGINT_AS_UINT_N_RAW)
}

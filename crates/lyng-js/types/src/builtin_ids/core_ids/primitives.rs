use super::super::*;

#[inline]
pub fn boolean_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(BOOLEAN_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn boolean_to_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(BOOLEAN_TO_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn boolean_value_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(BOOLEAN_VALUE_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn symbol_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SYMBOL_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn symbol_for_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SYMBOL_FOR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn symbol_key_for_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SYMBOL_KEY_FOR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn symbol_to_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SYMBOL_TO_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn symbol_value_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SYMBOL_VALUE_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn symbol_to_primitive_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SYMBOL_TO_PRIMITIVE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn symbol_description_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SYMBOL_DESCRIPTION_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn number_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(NUMBER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn number_is_finite_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(NUMBER_IS_FINITE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn number_is_integer_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(NUMBER_IS_INTEGER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn number_is_nan_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(NUMBER_IS_NAN_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn number_is_safe_integer_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(NUMBER_IS_SAFE_INTEGER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn number_to_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(NUMBER_TO_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn number_value_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(NUMBER_VALUE_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn number_to_exponential_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(NUMBER_TO_EXPONENTIAL_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn number_to_fixed_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(NUMBER_TO_FIXED_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn number_to_locale_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(NUMBER_TO_LOCALE_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn number_to_precision_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(NUMBER_TO_PRECISION_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn bigint_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(BIGINT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn bigint_to_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(BIGINT_TO_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn bigint_value_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(BIGINT_VALUE_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn bigint_as_int_n_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(BIGINT_AS_INT_N_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn bigint_as_uint_n_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(BIGINT_AS_UINT_N_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

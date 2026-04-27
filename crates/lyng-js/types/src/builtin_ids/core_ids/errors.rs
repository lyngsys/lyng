use super::super::*;

#[inline]
pub fn error_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ERROR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn error_to_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ERROR_TO_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn eval_error_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(EVAL_ERROR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn range_error_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(RANGE_ERROR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn reference_error_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REFERENCE_ERROR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn syntax_error_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SYNTAX_ERROR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn type_error_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPE_ERROR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn uri_error_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(URI_ERROR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn error_is_error_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ERROR_IS_ERROR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

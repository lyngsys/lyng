use super::super::*;

#[inline]
pub fn eval_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(EVAL_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn parse_int_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PARSE_INT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn parse_float_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PARSE_FLOAT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn is_nan_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(IS_NAN_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn is_finite_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(IS_FINITE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn encode_uri_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ENCODE_URI_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn encode_uri_component_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ENCODE_URI_COMPONENT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn decode_uri_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DECODE_URI_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn decode_uri_component_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DECODE_URI_COMPONENT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn escape_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ESCAPE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn unescape_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(UNESCAPE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

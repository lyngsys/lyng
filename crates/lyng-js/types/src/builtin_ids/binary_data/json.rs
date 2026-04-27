use super::super::*;

#[inline]
pub fn json_parse_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(JSON_PARSE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn json_stringify_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(JSON_STRINGIFY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn json_raw_json_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(JSON_RAW_JSON_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn json_is_raw_json_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(JSON_IS_RAW_JSON_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

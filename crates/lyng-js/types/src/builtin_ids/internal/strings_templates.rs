use super::super::*;

#[inline]
pub fn internal_string_replace_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_STRING_REPLACE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_string_index_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_STRING_INDEX_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_object_to_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_OBJECT_TO_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_template_to_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_TEMPLATE_TO_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_get_template_object_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_GET_TEMPLATE_OBJECT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

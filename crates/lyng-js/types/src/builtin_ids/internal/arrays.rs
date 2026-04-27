use super::super::*;

#[inline]
pub fn internal_array_index_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_ARRAY_INDEX_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_array_push_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_ARRAY_PUSH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_array_pop_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_ARRAY_POP_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

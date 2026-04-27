use super::super::*;

#[inline]
pub fn iterator_prototype_iterator_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_PROTOTYPE_ITERATOR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_iterator_next_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_ITERATOR_NEXT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_iterator_next_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_ITERATOR_NEXT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_iterator_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_ITERATOR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

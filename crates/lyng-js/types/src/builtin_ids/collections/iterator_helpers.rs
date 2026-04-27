use super::super::*;

#[inline]
pub fn iterator_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn iterator_from_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_FROM_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn iterator_reduce_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_REDUCE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn iterator_for_each_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_FOR_EACH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn iterator_some_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_SOME_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn iterator_every_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_EVERY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn iterator_find_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_FIND_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn iterator_to_array_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_TO_ARRAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn iterator_to_string_tag_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_TO_STRING_TAG_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn iterator_to_string_tag_setter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_TO_STRING_TAG_SETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn iterator_constructor_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_CONSTRUCTOR_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn iterator_constructor_setter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_CONSTRUCTOR_SETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

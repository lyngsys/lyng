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
pub fn iterator_concat_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_CONCAT_RAW) {
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
pub fn iterator_map_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_MAP_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn iterator_filter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_FILTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn iterator_take_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_TAKE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn iterator_drop_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_DROP_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn iterator_dispose_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_DISPOSE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn iterator_flat_map_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_FLAT_MAP_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn iterator_helper_next_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_HELPER_NEXT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn iterator_helper_return_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ITERATOR_HELPER_RETURN_RAW) {
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

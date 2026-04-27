use super::super::*;

#[inline]
pub fn typed_array_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_at_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_AT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_to_string_tag_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_TO_STRING_TAG_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_from_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_FROM_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_every_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_EVERY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_some_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_SOME_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_find_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_FIND_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_find_index_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_FIND_INDEX_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_find_last_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_FIND_LAST_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_find_last_index_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_FIND_LAST_INDEX_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_fill_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_FILL_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_copy_within_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_COPY_WITHIN_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_includes_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_INCLUDES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_index_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_INDEX_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_last_index_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_LAST_INDEX_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_filter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_FILTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_for_each_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_FOR_EACH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_join_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_JOIN_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_map_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_MAP_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_reduce_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_REDUCE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_reduce_right_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_REDUCE_RIGHT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_reverse_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_REVERSE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_to_reversed_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_TO_REVERSED_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_to_sorted_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_TO_SORTED_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_with_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_WITH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_sort_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_SORT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_to_locale_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_TO_LOCALE_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn typed_array_to_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TYPED_ARRAY_TO_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

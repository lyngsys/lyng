use super::super::*;

#[inline]
pub fn array_to_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_TO_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_for_each_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_FOR_EACH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_from_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_FROM_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_from_async_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_FROM_ASYNC_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_concat_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_CONCAT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_copy_within_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_COPY_WITHIN_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_fill_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_FILL_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_join_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_JOIN_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_unshift_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_UNSHIFT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_shift_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_SHIFT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_filter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_FILTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_map_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_MAP_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_reverse_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_REVERSE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_slice_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_SLICE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_last_index_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_LAST_INDEX_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_every_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_EVERY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_some_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_SOME_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_includes_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_INCLUDES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_index_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_INDEX_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_reduce_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_REDUCE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_reduce_right_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_REDUCE_RIGHT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_find_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_FIND_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_find_index_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_FIND_INDEX_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_find_last_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_FIND_LAST_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_find_last_index_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_FIND_LAST_INDEX_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_to_reversed_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_TO_REVERSED_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_to_sorted_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_TO_SORTED_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_to_spliced_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_TO_SPLICED_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_with_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_WITH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_at_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_AT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_flat_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_FLAT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_flat_map_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_FLAT_MAP_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_pop_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_POP_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_push_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_PUSH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_sort_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_SORT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_splice_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_SPLICE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_to_locale_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_TO_LOCALE_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_values_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_VALUES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_keys_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_KEYS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_entries_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_ENTRIES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_is_array_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_IS_ARRAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn array_species_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ARRAY_SPECIES_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

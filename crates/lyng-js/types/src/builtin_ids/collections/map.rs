use super::super::*;

#[inline]
pub fn map_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(MAP_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn map_get_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(MAP_GET_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn map_set_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(MAP_SET_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn map_has_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(MAP_HAS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn map_delete_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(MAP_DELETE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn map_clear_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(MAP_CLEAR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn map_entries_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(MAP_ENTRIES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn map_values_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(MAP_VALUES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn map_keys_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(MAP_KEYS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn map_size_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(MAP_SIZE_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn map_iterator_next_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(MAP_ITERATOR_NEXT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn map_for_each_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(MAP_FOR_EACH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn map_get_or_insert_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(MAP_GET_OR_INSERT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn map_get_or_insert_computed_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(MAP_GET_OR_INSERT_COMPUTED_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

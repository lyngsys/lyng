use super::super::*;

#[inline]
pub fn weak_map_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(WEAK_MAP_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn weak_map_get_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(WEAK_MAP_GET_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn weak_map_set_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(WEAK_MAP_SET_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn weak_map_has_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(WEAK_MAP_HAS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn weak_map_delete_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(WEAK_MAP_DELETE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn weak_map_get_or_insert_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(WEAK_MAP_GET_OR_INSERT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn weak_map_get_or_insert_computed_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(WEAK_MAP_GET_OR_INSERT_COMPUTED_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

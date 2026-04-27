use super::super::*;

#[inline]
pub fn set_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SET_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn set_add_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SET_ADD_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn set_has_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SET_HAS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn set_delete_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SET_DELETE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn set_clear_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SET_CLEAR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn set_entries_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SET_ENTRIES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn set_values_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SET_VALUES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn set_keys_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SET_KEYS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn set_size_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SET_SIZE_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn set_iterator_next_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SET_ITERATOR_NEXT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn set_for_each_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SET_FOR_EACH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

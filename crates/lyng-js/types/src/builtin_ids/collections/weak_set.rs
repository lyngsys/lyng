use super::super::*;

#[inline]
pub fn weak_set_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(WEAK_SET_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn weak_set_add_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(WEAK_SET_ADD_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn weak_set_has_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(WEAK_SET_HAS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn weak_set_delete_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(WEAK_SET_DELETE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

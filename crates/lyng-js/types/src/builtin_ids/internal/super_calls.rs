use super::super::*;

#[inline]
pub fn internal_super_property_get_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_SUPER_PROPERTY_GET_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_super_property_set_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_SUPER_PROPERTY_SET_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_super_base_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_SUPER_BASE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_super_constructor_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_SUPER_CONSTRUCTOR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_construct_super_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_CONSTRUCT_SUPER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_construct_super_spread_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_CONSTRUCT_SUPER_SPREAD_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_construct_super_array_like_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_CONSTRUCT_SUPER_ARRAY_LIKE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

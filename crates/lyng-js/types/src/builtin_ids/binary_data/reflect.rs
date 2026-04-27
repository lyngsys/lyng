use super::super::*;

#[inline]
pub fn reflect_apply_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REFLECT_APPLY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn reflect_construct_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REFLECT_CONSTRUCT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn reflect_define_property_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REFLECT_DEFINE_PROPERTY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn reflect_delete_property_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REFLECT_DELETE_PROPERTY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn reflect_get_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REFLECT_GET_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn reflect_get_own_property_descriptor_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REFLECT_GET_OWN_PROPERTY_DESCRIPTOR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn reflect_get_prototype_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REFLECT_GET_PROTOTYPE_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn reflect_has_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REFLECT_HAS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn reflect_is_extensible_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REFLECT_IS_EXTENSIBLE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn reflect_own_keys_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REFLECT_OWN_KEYS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn reflect_prevent_extensions_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REFLECT_PREVENT_EXTENSIONS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn reflect_set_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REFLECT_SET_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn reflect_set_prototype_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REFLECT_SET_PROTOTYPE_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

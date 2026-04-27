use super::super::*;

#[inline]
pub fn finalization_registry_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(FINALIZATION_REGISTRY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn finalization_registry_register_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(FINALIZATION_REGISTRY_REGISTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn finalization_registry_unregister_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(FINALIZATION_REGISTRY_UNREGISTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

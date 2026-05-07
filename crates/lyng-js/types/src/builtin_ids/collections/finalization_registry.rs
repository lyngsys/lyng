use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn finalization_registry_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::FINALIZATION_REGISTRY_RAW)
}

#[inline]
pub const fn finalization_registry_register_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::FINALIZATION_REGISTRY_REGISTER_RAW)
}

#[inline]
pub const fn finalization_registry_unregister_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::FINALIZATION_REGISTRY_UNREGISTER_RAW)
}

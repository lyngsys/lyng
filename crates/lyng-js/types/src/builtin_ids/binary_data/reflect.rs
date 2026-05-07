use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn reflect_apply_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::REFLECT_APPLY_RAW)
}

#[inline]
pub const fn reflect_construct_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::REFLECT_CONSTRUCT_RAW)
}

#[inline]
pub const fn reflect_define_property_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::REFLECT_DEFINE_PROPERTY_RAW)
}

#[inline]
pub const fn reflect_delete_property_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::REFLECT_DELETE_PROPERTY_RAW)
}

#[inline]
pub const fn reflect_get_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::REFLECT_GET_RAW)
}

#[inline]
pub const fn reflect_get_own_property_descriptor_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::REFLECT_GET_OWN_PROPERTY_DESCRIPTOR_RAW)
}

#[inline]
pub const fn reflect_get_prototype_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::REFLECT_GET_PROTOTYPE_OF_RAW)
}

#[inline]
pub const fn reflect_has_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::REFLECT_HAS_RAW)
}

#[inline]
pub const fn reflect_is_extensible_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::REFLECT_IS_EXTENSIBLE_RAW)
}

#[inline]
pub const fn reflect_own_keys_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::REFLECT_OWN_KEYS_RAW)
}

#[inline]
pub const fn reflect_prevent_extensions_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::REFLECT_PREVENT_EXTENSIONS_RAW)
}

#[inline]
pub const fn reflect_set_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::REFLECT_SET_RAW)
}

#[inline]
pub const fn reflect_set_prototype_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::REFLECT_SET_PROTOTYPE_OF_RAW)
}

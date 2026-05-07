use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn internal_super_property_get_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_SUPER_PROPERTY_GET_RAW)
}

#[inline]
pub const fn internal_super_property_set_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_SUPER_PROPERTY_SET_RAW)
}

#[inline]
pub const fn internal_super_base_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_SUPER_BASE_RAW)
}

#[inline]
pub const fn internal_super_constructor_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_SUPER_CONSTRUCTOR_RAW)
}

#[inline]
pub const fn internal_construct_super_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_CONSTRUCT_SUPER_RAW)
}

#[inline]
pub const fn internal_construct_super_spread_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_CONSTRUCT_SUPER_SPREAD_RAW)
}

#[inline]
pub const fn internal_construct_super_array_like_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_CONSTRUCT_SUPER_ARRAY_LIKE_RAW)
}

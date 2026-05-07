use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn internal_define_class_getter_property_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_DEFINE_CLASS_GETTER_PROPERTY_RAW)
}

#[inline]
pub const fn internal_define_class_setter_property_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_DEFINE_CLASS_SETTER_PROPERTY_RAW)
}

#[inline]
pub const fn internal_define_private_field_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_DEFINE_PRIVATE_FIELD_RAW)
}

#[inline]
pub const fn internal_private_field_init_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_PRIVATE_FIELD_INIT_RAW)
}

#[inline]
pub const fn internal_private_field_get_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_PRIVATE_FIELD_GET_RAW)
}

#[inline]
pub const fn internal_private_field_set_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_PRIVATE_FIELD_SET_RAW)
}

#[inline]
pub const fn internal_private_has_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_PRIVATE_HAS_RAW)
}

#[inline]
pub const fn internal_bind_function_private_env_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_BIND_FUNCTION_PRIVATE_ENV_RAW)
}

#[inline]
pub const fn internal_install_instance_field_key_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_INSTALL_INSTANCE_FIELD_KEY_RAW)
}

#[inline]
pub const fn internal_get_instance_field_key_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_GET_INSTANCE_FIELD_KEY_RAW)
}

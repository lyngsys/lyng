use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn internal_instance_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_INSTANCE_OF_RAW)
}

#[inline]
pub const fn internal_define_getter_property_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_DEFINE_GETTER_PROPERTY_RAW)
}

#[inline]
pub const fn internal_define_setter_property_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_DEFINE_SETTER_PROPERTY_RAW)
}

#[inline]
pub const fn internal_object_has_own_property_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_OBJECT_HAS_OWN_PROPERTY_RAW)
}

#[inline]
pub const fn internal_throw_type_error_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_THROW_TYPE_ERROR_RAW)
}

#[inline]
pub const fn internal_require_constructor_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_REQUIRE_CONSTRUCTOR_RAW)
}

#[inline]
pub const fn internal_define_method_property_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_DEFINE_METHOD_PROPERTY_RAW)
}

#[inline]
pub const fn internal_object_literal_set_prototype_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_OBJECT_LITERAL_SET_PROTOTYPE_RAW)
}

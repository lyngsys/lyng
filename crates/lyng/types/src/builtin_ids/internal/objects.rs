use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    internal_instance_of_builtin => super::super::INTERNAL_INSTANCE_OF_RAW;
    internal_define_getter_property_builtin => super::super::INTERNAL_DEFINE_GETTER_PROPERTY_RAW;
    internal_define_setter_property_builtin => super::super::INTERNAL_DEFINE_SETTER_PROPERTY_RAW;
    internal_object_has_own_property_builtin => super::super::INTERNAL_OBJECT_HAS_OWN_PROPERTY_RAW;
    internal_throw_type_error_builtin => super::super::INTERNAL_THROW_TYPE_ERROR_RAW;
    internal_require_constructor_builtin => super::super::INTERNAL_REQUIRE_CONSTRUCTOR_RAW;
    internal_define_method_property_builtin => super::super::INTERNAL_DEFINE_METHOD_PROPERTY_RAW;
    internal_object_literal_set_prototype_builtin => super::super::INTERNAL_OBJECT_LITERAL_SET_PROTOTYPE_RAW;
}

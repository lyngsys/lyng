use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    internal_define_class_getter_property_builtin => super::super::INTERNAL_DEFINE_CLASS_GETTER_PROPERTY_RAW;
    internal_define_class_setter_property_builtin => super::super::INTERNAL_DEFINE_CLASS_SETTER_PROPERTY_RAW;
    internal_define_private_field_builtin => super::super::INTERNAL_DEFINE_PRIVATE_FIELD_RAW;
    internal_private_field_init_builtin => super::super::INTERNAL_PRIVATE_FIELD_INIT_RAW;
    internal_private_field_get_builtin => super::super::INTERNAL_PRIVATE_FIELD_GET_RAW;
    internal_private_field_set_builtin => super::super::INTERNAL_PRIVATE_FIELD_SET_RAW;
    internal_private_has_builtin => super::super::INTERNAL_PRIVATE_HAS_RAW;
    internal_bind_function_private_env_builtin => super::super::INTERNAL_BIND_FUNCTION_PRIVATE_ENV_RAW;
    internal_install_instance_field_key_builtin => super::super::INTERNAL_INSTALL_INSTANCE_FIELD_KEY_RAW;
    internal_get_instance_field_key_builtin => super::super::INTERNAL_GET_INSTANCE_FIELD_KEY_RAW;
}

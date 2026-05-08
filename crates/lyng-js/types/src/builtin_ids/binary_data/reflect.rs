use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    reflect_apply_builtin => super::super::REFLECT_APPLY_RAW;
    reflect_construct_builtin => super::super::REFLECT_CONSTRUCT_RAW;
    reflect_define_property_builtin => super::super::REFLECT_DEFINE_PROPERTY_RAW;
    reflect_delete_property_builtin => super::super::REFLECT_DELETE_PROPERTY_RAW;
    reflect_get_builtin => super::super::REFLECT_GET_RAW;
    reflect_get_own_property_descriptor_builtin => super::super::REFLECT_GET_OWN_PROPERTY_DESCRIPTOR_RAW;
    reflect_get_prototype_of_builtin => super::super::REFLECT_GET_PROTOTYPE_OF_RAW;
    reflect_has_builtin => super::super::REFLECT_HAS_RAW;
    reflect_is_extensible_builtin => super::super::REFLECT_IS_EXTENSIBLE_RAW;
    reflect_own_keys_builtin => super::super::REFLECT_OWN_KEYS_RAW;
    reflect_prevent_extensions_builtin => super::super::REFLECT_PREVENT_EXTENSIONS_RAW;
    reflect_set_builtin => super::super::REFLECT_SET_RAW;
    reflect_set_prototype_of_builtin => super::super::REFLECT_SET_PROTOTYPE_OF_RAW;
}

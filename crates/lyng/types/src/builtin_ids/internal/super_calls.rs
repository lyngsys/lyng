use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    internal_super_property_get_builtin => super::super::INTERNAL_SUPER_PROPERTY_GET_RAW;
    internal_super_property_set_builtin => super::super::INTERNAL_SUPER_PROPERTY_SET_RAW;
    internal_super_base_builtin => super::super::INTERNAL_SUPER_BASE_RAW;
    internal_super_constructor_builtin => super::super::INTERNAL_SUPER_CONSTRUCTOR_RAW;
    internal_construct_super_builtin => super::super::INTERNAL_CONSTRUCT_SUPER_RAW;
    internal_construct_super_spread_builtin => super::super::INTERNAL_CONSTRUCT_SUPER_SPREAD_RAW;
    internal_construct_super_array_like_builtin => super::super::INTERNAL_CONSTRUCT_SUPER_ARRAY_LIKE_RAW;
}

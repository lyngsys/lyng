use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn object_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_RAW)
}

#[inline]
pub const fn object_create_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_CREATE_RAW)
}

#[inline]
pub const fn object_get_prototype_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_GET_PROTOTYPE_OF_RAW)
}

#[inline]
pub const fn object_set_prototype_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_SET_PROTOTYPE_OF_RAW)
}

#[inline]
pub const fn object_get_own_property_descriptor_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_GET_OWN_PROPERTY_DESCRIPTOR_RAW)
}

#[inline]
pub const fn object_get_own_property_descriptors_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_GET_OWN_PROPERTY_DESCRIPTORS_RAW)
}

#[inline]
pub const fn object_define_property_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_DEFINE_PROPERTY_RAW)
}

#[inline]
pub const fn object_define_properties_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_DEFINE_PROPERTIES_RAW)
}

#[inline]
pub const fn object_prevent_extensions_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_PREVENT_EXTENSIONS_RAW)
}

#[inline]
pub const fn object_is_extensible_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_IS_EXTENSIBLE_RAW)
}

#[inline]
pub const fn object_is_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_IS_RAW)
}

#[inline]
pub const fn object_seal_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_SEAL_RAW)
}

#[inline]
pub const fn object_freeze_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_FREEZE_RAW)
}

#[inline]
pub const fn object_is_sealed_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_IS_SEALED_RAW)
}

#[inline]
pub const fn object_is_frozen_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_IS_FROZEN_RAW)
}

#[inline]
pub const fn object_to_string_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_TO_STRING_RAW)
}

#[inline]
pub const fn object_to_locale_string_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_TO_LOCALE_STRING_RAW)
}

#[inline]
pub const fn object_value_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_VALUE_OF_RAW)
}

#[inline]
pub const fn object_has_own_property_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_HAS_OWN_PROPERTY_RAW)
}

#[inline]
pub const fn object_is_prototype_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_IS_PROTOTYPE_OF_RAW)
}

#[inline]
pub const fn object_property_is_enumerable_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_PROPERTY_IS_ENUMERABLE_RAW)
}

#[inline]
pub const fn object_assign_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_ASSIGN_RAW)
}

#[inline]
pub const fn object_keys_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_KEYS_RAW)
}

#[inline]
pub const fn object_entries_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_ENTRIES_RAW)
}

#[inline]
pub const fn object_values_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_VALUES_RAW)
}

#[inline]
pub const fn object_has_own_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_HAS_OWN_RAW)
}

#[inline]
pub const fn object_from_entries_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_FROM_ENTRIES_RAW)
}

#[inline]
pub const fn object_group_by_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_GROUP_BY_RAW)
}

#[inline]
pub const fn object_define_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_DEFINE_GETTER_RAW)
}

#[inline]
pub const fn object_define_setter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_DEFINE_SETTER_RAW)
}

#[inline]
pub const fn object_lookup_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_LOOKUP_GETTER_RAW)
}

#[inline]
pub const fn object_lookup_setter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_LOOKUP_SETTER_RAW)
}

#[inline]
pub const fn object_proto_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_PROTO_GETTER_RAW)
}

#[inline]
pub const fn object_proto_setter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_PROTO_SETTER_RAW)
}

#[inline]
pub const fn object_get_own_property_names_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_GET_OWN_PROPERTY_NAMES_RAW)
}

#[inline]
pub const fn object_get_own_property_symbols_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::OBJECT_GET_OWN_PROPERTY_SYMBOLS_RAW)
}

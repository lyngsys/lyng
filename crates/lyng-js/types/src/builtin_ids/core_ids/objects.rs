use super::super::*;

#[inline]
pub fn object_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_create_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_CREATE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_get_prototype_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_GET_PROTOTYPE_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_set_prototype_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_SET_PROTOTYPE_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_get_own_property_descriptor_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_GET_OWN_PROPERTY_DESCRIPTOR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_get_own_property_descriptors_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_GET_OWN_PROPERTY_DESCRIPTORS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_define_property_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_DEFINE_PROPERTY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_define_properties_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_DEFINE_PROPERTIES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_prevent_extensions_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_PREVENT_EXTENSIONS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_is_extensible_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_IS_EXTENSIBLE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_is_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_IS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_seal_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_SEAL_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_freeze_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_FREEZE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_is_sealed_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_IS_SEALED_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_is_frozen_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_IS_FROZEN_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_to_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_TO_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_to_locale_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_TO_LOCALE_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_value_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_VALUE_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_has_own_property_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_HAS_OWN_PROPERTY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_is_prototype_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_IS_PROTOTYPE_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_property_is_enumerable_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_PROPERTY_IS_ENUMERABLE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_assign_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_ASSIGN_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_keys_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_KEYS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_entries_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_ENTRIES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_values_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_VALUES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_has_own_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_HAS_OWN_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_from_entries_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_FROM_ENTRIES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_group_by_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_GROUP_BY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_define_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_DEFINE_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_define_setter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_DEFINE_SETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_lookup_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_LOOKUP_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_lookup_setter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_LOOKUP_SETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_proto_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_PROTO_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_proto_setter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_PROTO_SETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_get_own_property_names_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_GET_OWN_PROPERTY_NAMES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn object_get_own_property_symbols_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(OBJECT_GET_OWN_PROPERTY_SYMBOLS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

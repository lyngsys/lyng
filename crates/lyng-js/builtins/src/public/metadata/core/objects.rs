use super::super::{
    object_assign_builtin, object_builtin, object_create_builtin, object_define_getter_builtin,
    object_define_properties_builtin, object_define_property_builtin, object_define_setter_builtin,
    object_entries_builtin, object_freeze_builtin, object_from_entries_builtin,
    object_get_own_property_descriptor_builtin, object_get_own_property_descriptors_builtin,
    object_get_own_property_names_builtin, object_get_own_property_symbols_builtin,
    object_get_prototype_of_builtin, object_group_by_builtin, object_has_own_builtin,
    object_has_own_property_builtin, object_is_builtin, object_is_extensible_builtin,
    object_is_frozen_builtin, object_is_prototype_of_builtin, object_is_sealed_builtin,
    object_keys_builtin, object_lookup_getter_builtin, object_lookup_setter_builtin,
    object_prevent_extensions_builtin, object_property_is_enumerable_builtin,
    object_proto_getter_builtin, object_proto_setter_builtin, object_seal_builtin,
    object_set_prototype_of_builtin, object_to_locale_string_builtin, object_to_string_builtin,
    object_value_of_builtin, object_values_builtin, BuiltinEntryMetadata, PublicBuiltinMetadataRow,
};

pub(in crate::public::metadata) const PUBLIC_OBJECT_BUILTIN_METADATA:
    &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        object_builtin,
        BuiltinEntryMetadata::new("Object", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        object_create_builtin,
        BuiltinEntryMetadata::new("create", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_get_prototype_of_builtin,
        BuiltinEntryMetadata::new("getPrototypeOf", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_set_prototype_of_builtin,
        BuiltinEntryMetadata::new("setPrototypeOf", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_get_own_property_descriptor_builtin,
        BuiltinEntryMetadata::new("getOwnPropertyDescriptor", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_get_own_property_descriptors_builtin,
        BuiltinEntryMetadata::new("getOwnPropertyDescriptors", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_get_own_property_names_builtin,
        BuiltinEntryMetadata::new("getOwnPropertyNames", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_get_own_property_symbols_builtin,
        BuiltinEntryMetadata::new("getOwnPropertySymbols", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_define_properties_builtin,
        BuiltinEntryMetadata::new("defineProperties", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_define_property_builtin,
        BuiltinEntryMetadata::new("defineProperty", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_assign_builtin,
        BuiltinEntryMetadata::new("assign", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_from_entries_builtin,
        BuiltinEntryMetadata::new("fromEntries", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_group_by_builtin,
        BuiltinEntryMetadata::new("groupBy", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_prevent_extensions_builtin,
        BuiltinEntryMetadata::new("preventExtensions", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_is_extensible_builtin,
        BuiltinEntryMetadata::new("isExtensible", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_is_builtin,
        BuiltinEntryMetadata::new("is", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_seal_builtin,
        BuiltinEntryMetadata::new("seal", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_freeze_builtin,
        BuiltinEntryMetadata::new("freeze", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_is_sealed_builtin,
        BuiltinEntryMetadata::new("isSealed", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_is_frozen_builtin,
        BuiltinEntryMetadata::new("isFrozen", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_to_locale_string_builtin,
        BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_to_string_builtin,
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_value_of_builtin,
        BuiltinEntryMetadata::new("valueOf", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_has_own_property_builtin,
        BuiltinEntryMetadata::new("hasOwnProperty", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_is_prototype_of_builtin,
        BuiltinEntryMetadata::new("isPrototypeOf", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_property_is_enumerable_builtin,
        BuiltinEntryMetadata::new("propertyIsEnumerable", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_define_getter_builtin,
        BuiltinEntryMetadata::new("__defineGetter__", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_define_setter_builtin,
        BuiltinEntryMetadata::new("__defineSetter__", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_lookup_getter_builtin,
        BuiltinEntryMetadata::new("__lookupGetter__", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_lookup_setter_builtin,
        BuiltinEntryMetadata::new("__lookupSetter__", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_proto_getter_builtin,
        BuiltinEntryMetadata::new("get __proto__", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_proto_setter_builtin,
        BuiltinEntryMetadata::new("set __proto__", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_keys_builtin,
        BuiltinEntryMetadata::new("keys", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_entries_builtin,
        BuiltinEntryMetadata::new("entries", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_values_builtin,
        BuiltinEntryMetadata::new("values", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        object_has_own_builtin,
        BuiltinEntryMetadata::new("hasOwn", 2, false, false),
    ),
];

use super::super::{BuiltinEntryMetadata, PublicBuiltinMetadataRow};
use lyng_js_types::{
    json_is_raw_json_builtin, json_parse_builtin, json_raw_json_builtin, json_stringify_builtin,
    proxy_builtin, proxy_revocable_builtin, proxy_revoke_builtin, reflect_apply_builtin,
    reflect_construct_builtin, reflect_define_property_builtin, reflect_delete_property_builtin,
    reflect_get_builtin, reflect_get_own_property_descriptor_builtin,
    reflect_get_prototype_of_builtin, reflect_has_builtin, reflect_is_extensible_builtin,
    reflect_own_keys_builtin, reflect_prevent_extensions_builtin, reflect_set_builtin,
    reflect_set_prototype_of_builtin,
};
pub(in crate::public::metadata) const PUBLIC_OBJECT_REFLECTION_BUILTIN_METADATA:
    &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        json_parse_builtin(),
        BuiltinEntryMetadata::new("parse", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        json_stringify_builtin(),
        BuiltinEntryMetadata::new("stringify", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        json_raw_json_builtin(),
        BuiltinEntryMetadata::new("rawJSON", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        json_is_raw_json_builtin(),
        BuiltinEntryMetadata::new("isRawJSON", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        reflect_apply_builtin(),
        BuiltinEntryMetadata::new("apply", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        reflect_construct_builtin(),
        BuiltinEntryMetadata::new("construct", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        reflect_define_property_builtin(),
        BuiltinEntryMetadata::new("defineProperty", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        reflect_delete_property_builtin(),
        BuiltinEntryMetadata::new("deleteProperty", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        reflect_get_builtin(),
        BuiltinEntryMetadata::new("get", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        reflect_get_own_property_descriptor_builtin(),
        BuiltinEntryMetadata::new("getOwnPropertyDescriptor", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        reflect_get_prototype_of_builtin(),
        BuiltinEntryMetadata::new("getPrototypeOf", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        reflect_has_builtin(),
        BuiltinEntryMetadata::new("has", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        reflect_is_extensible_builtin(),
        BuiltinEntryMetadata::new("isExtensible", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        reflect_own_keys_builtin(),
        BuiltinEntryMetadata::new("ownKeys", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        reflect_prevent_extensions_builtin(),
        BuiltinEntryMetadata::new("preventExtensions", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        reflect_set_builtin(),
        BuiltinEntryMetadata::new("set", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        reflect_set_prototype_of_builtin(),
        BuiltinEntryMetadata::new("setPrototypeOf", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        proxy_builtin(),
        BuiltinEntryMetadata::new("Proxy", 2, true, false),
    ),
    PublicBuiltinMetadataRow::new(
        proxy_revocable_builtin(),
        BuiltinEntryMetadata::new("revocable", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        proxy_revoke_builtin(),
        BuiltinEntryMetadata::new("", 0, false, false),
    ),
];

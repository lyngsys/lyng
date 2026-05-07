use super::super::{
    finalization_registry_builtin, finalization_registry_register_builtin,
    finalization_registry_unregister_builtin, map_builtin, map_clear_builtin, map_delete_builtin,
    map_entries_builtin, map_for_each_builtin, map_get_builtin, map_get_or_insert_builtin,
    map_get_or_insert_computed_builtin, map_group_by_builtin, map_has_builtin, map_keys_builtin,
    map_set_builtin, map_size_getter_builtin, map_values_builtin, set_add_builtin, set_builtin,
    set_clear_builtin, set_delete_builtin, set_difference_builtin, set_entries_builtin,
    set_for_each_builtin, set_has_builtin, set_intersection_builtin, set_is_disjoint_from_builtin,
    set_is_subset_of_builtin, set_is_superset_of_builtin, set_keys_builtin,
    set_size_getter_builtin, set_symmetric_difference_builtin, set_union_builtin,
    set_values_builtin, weak_map_builtin, weak_map_delete_builtin, weak_map_get_builtin,
    weak_map_get_or_insert_builtin, weak_map_get_or_insert_computed_builtin, weak_map_has_builtin,
    weak_map_set_builtin, weak_ref_builtin, weak_ref_deref_builtin, weak_set_add_builtin,
    weak_set_builtin, weak_set_delete_builtin, weak_set_has_builtin, BuiltinEntryMetadata,
    PublicBuiltinMetadataRow,
};

pub(in crate::public::metadata) const PUBLIC_KEYED_COLLECTION_BUILTIN_METADATA:
    &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(map_builtin, BuiltinEntryMetadata::new("Map", 0, true, true)),
    PublicBuiltinMetadataRow::new(
        map_group_by_builtin,
        BuiltinEntryMetadata::new("groupBy", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(set_builtin, BuiltinEntryMetadata::new("Set", 0, true, true)),
    PublicBuiltinMetadataRow::new(
        weak_map_builtin,
        BuiltinEntryMetadata::new("WeakMap", 0, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        weak_set_builtin,
        BuiltinEntryMetadata::new("WeakSet", 0, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        map_get_builtin,
        BuiltinEntryMetadata::new("get", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        map_set_builtin,
        BuiltinEntryMetadata::new("set", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        map_has_builtin,
        BuiltinEntryMetadata::new("has", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        map_delete_builtin,
        BuiltinEntryMetadata::new("delete", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        map_clear_builtin,
        BuiltinEntryMetadata::new("clear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        map_entries_builtin,
        BuiltinEntryMetadata::new("entries", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        map_values_builtin,
        BuiltinEntryMetadata::new("values", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        map_keys_builtin,
        BuiltinEntryMetadata::new("keys", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        map_for_each_builtin,
        BuiltinEntryMetadata::new("forEach", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        map_size_getter_builtin,
        BuiltinEntryMetadata::new("get size", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        map_get_or_insert_builtin,
        BuiltinEntryMetadata::new("getOrInsert", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        map_get_or_insert_computed_builtin,
        BuiltinEntryMetadata::new("getOrInsertComputed", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        set_add_builtin,
        BuiltinEntryMetadata::new("add", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        set_has_builtin,
        BuiltinEntryMetadata::new("has", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        set_delete_builtin,
        BuiltinEntryMetadata::new("delete", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        set_clear_builtin,
        BuiltinEntryMetadata::new("clear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        set_entries_builtin,
        BuiltinEntryMetadata::new("entries", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        set_values_builtin,
        BuiltinEntryMetadata::new("values", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        set_keys_builtin,
        BuiltinEntryMetadata::new("keys", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        set_for_each_builtin,
        BuiltinEntryMetadata::new("forEach", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        set_size_getter_builtin,
        BuiltinEntryMetadata::new("get size", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        set_union_builtin,
        BuiltinEntryMetadata::new("union", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        set_intersection_builtin,
        BuiltinEntryMetadata::new("intersection", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        set_difference_builtin,
        BuiltinEntryMetadata::new("difference", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        set_symmetric_difference_builtin,
        BuiltinEntryMetadata::new("symmetricDifference", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        set_is_subset_of_builtin,
        BuiltinEntryMetadata::new("isSubsetOf", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        set_is_superset_of_builtin,
        BuiltinEntryMetadata::new("isSupersetOf", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        set_is_disjoint_from_builtin,
        BuiltinEntryMetadata::new("isDisjointFrom", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        weak_map_get_builtin,
        BuiltinEntryMetadata::new("get", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        weak_map_set_builtin,
        BuiltinEntryMetadata::new("set", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        weak_map_has_builtin,
        BuiltinEntryMetadata::new("has", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        weak_map_delete_builtin,
        BuiltinEntryMetadata::new("delete", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        weak_map_get_or_insert_builtin,
        BuiltinEntryMetadata::new("getOrInsert", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        weak_map_get_or_insert_computed_builtin,
        BuiltinEntryMetadata::new("getOrInsertComputed", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        weak_set_add_builtin,
        BuiltinEntryMetadata::new("add", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        weak_set_has_builtin,
        BuiltinEntryMetadata::new("has", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        weak_set_delete_builtin,
        BuiltinEntryMetadata::new("delete", 1, false, false),
    ),
];

pub(in crate::public::metadata) const PUBLIC_WEAK_REF_BUILTIN_METADATA:
    &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        weak_ref_builtin,
        BuiltinEntryMetadata::new("WeakRef", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        finalization_registry_builtin,
        BuiltinEntryMetadata::new("FinalizationRegistry", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        weak_ref_deref_builtin,
        BuiltinEntryMetadata::new("deref", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        finalization_registry_register_builtin,
        BuiltinEntryMetadata::new("register", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        finalization_registry_unregister_builtin,
        BuiltinEntryMetadata::new("unregister", 1, false, false),
    ),
];

use super::super::*;

pub(in crate::public::metadata) const PUBLIC_REGEXP_BUILTIN_METADATA:
    &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        regexp_builtin,
        BuiltinEntryMetadata::new("RegExp", 2, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_escape_builtin,
        BuiltinEntryMetadata::new("escape", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_to_string_builtin,
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_exec_builtin,
        BuiltinEntryMetadata::new("exec", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_test_builtin,
        BuiltinEntryMetadata::new("test", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_global_getter_builtin,
        BuiltinEntryMetadata::new("get global", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_ignore_case_getter_builtin,
        BuiltinEntryMetadata::new("get ignoreCase", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_multiline_getter_builtin,
        BuiltinEntryMetadata::new("get multiline", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_dot_all_getter_builtin,
        BuiltinEntryMetadata::new("get dotAll", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_unicode_getter_builtin,
        BuiltinEntryMetadata::new("get unicode", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_unicode_sets_getter_builtin,
        BuiltinEntryMetadata::new("get unicodeSets", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_sticky_getter_builtin,
        BuiltinEntryMetadata::new("get sticky", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_source_getter_builtin,
        BuiltinEntryMetadata::new("get source", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_flags_getter_builtin,
        BuiltinEntryMetadata::new("get flags", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_has_indices_getter_builtin,
        BuiltinEntryMetadata::new("get hasIndices", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_species_getter_builtin,
        BuiltinEntryMetadata::new("get [Symbol.species]", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_symbol_match_builtin,
        BuiltinEntryMetadata::new("[Symbol.match]", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_symbol_replace_builtin,
        BuiltinEntryMetadata::new("[Symbol.replace]", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_symbol_search_builtin,
        BuiltinEntryMetadata::new("[Symbol.search]", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_symbol_split_builtin,
        BuiltinEntryMetadata::new("[Symbol.split]", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_symbol_match_all_builtin,
        BuiltinEntryMetadata::new("[Symbol.matchAll]", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_string_iterator_next_builtin,
        BuiltinEntryMetadata::new("next", 0, false, false),
    ),
];

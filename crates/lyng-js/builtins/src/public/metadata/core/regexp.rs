use super::super::{
    regexp_builtin, regexp_compile_builtin, regexp_dot_all_getter_builtin, regexp_escape_builtin,
    regexp_exec_builtin, regexp_flags_getter_builtin, regexp_global_getter_builtin,
    regexp_has_indices_getter_builtin, regexp_ignore_case_getter_builtin,
    regexp_legacy_input_getter_builtin, regexp_legacy_input_setter_builtin,
    regexp_legacy_last_match_getter_builtin, regexp_legacy_last_paren_getter_builtin,
    regexp_legacy_left_context_getter_builtin, regexp_legacy_paren1_getter_builtin,
    regexp_legacy_paren2_getter_builtin, regexp_legacy_paren3_getter_builtin,
    regexp_legacy_paren4_getter_builtin, regexp_legacy_paren5_getter_builtin,
    regexp_legacy_paren6_getter_builtin, regexp_legacy_paren7_getter_builtin,
    regexp_legacy_paren8_getter_builtin, regexp_legacy_paren9_getter_builtin,
    regexp_legacy_right_context_getter_builtin, regexp_multiline_getter_builtin,
    regexp_source_getter_builtin, regexp_species_getter_builtin, regexp_sticky_getter_builtin,
    regexp_string_iterator_next_builtin, regexp_symbol_match_all_builtin,
    regexp_symbol_match_builtin, regexp_symbol_replace_builtin, regexp_symbol_search_builtin,
    regexp_symbol_split_builtin, regexp_test_builtin, regexp_to_string_builtin,
    regexp_unicode_getter_builtin, regexp_unicode_sets_getter_builtin, BuiltinEntryMetadata,
    PublicBuiltinMetadataRow,
};

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
        regexp_compile_builtin,
        BuiltinEntryMetadata::new("compile", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_legacy_input_getter_builtin,
        BuiltinEntryMetadata::new("get input", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_legacy_input_setter_builtin,
        BuiltinEntryMetadata::new("set input", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_legacy_last_match_getter_builtin,
        BuiltinEntryMetadata::new("get lastMatch", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_legacy_last_paren_getter_builtin,
        BuiltinEntryMetadata::new("get lastParen", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_legacy_left_context_getter_builtin,
        BuiltinEntryMetadata::new("get leftContext", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_legacy_right_context_getter_builtin,
        BuiltinEntryMetadata::new("get rightContext", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_legacy_paren1_getter_builtin,
        BuiltinEntryMetadata::new("get $1", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_legacy_paren2_getter_builtin,
        BuiltinEntryMetadata::new("get $2", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_legacy_paren3_getter_builtin,
        BuiltinEntryMetadata::new("get $3", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_legacy_paren4_getter_builtin,
        BuiltinEntryMetadata::new("get $4", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_legacy_paren5_getter_builtin,
        BuiltinEntryMetadata::new("get $5", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_legacy_paren6_getter_builtin,
        BuiltinEntryMetadata::new("get $6", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_legacy_paren7_getter_builtin,
        BuiltinEntryMetadata::new("get $7", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_legacy_paren8_getter_builtin,
        BuiltinEntryMetadata::new("get $8", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        regexp_legacy_paren9_getter_builtin,
        BuiltinEntryMetadata::new("get $9", 0, false, false),
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

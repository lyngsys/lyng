use super::super::{BuiltinEntryMetadata, PublicBuiltinMetadataRow};
use lyng_js_types::{
    array_iterator_next_builtin, async_iterator_dispose_builtin, iterator_builtin,
    iterator_concat_builtin, iterator_constructor_getter_builtin,
    iterator_constructor_setter_builtin, iterator_dispose_builtin, iterator_drop_builtin,
    iterator_every_builtin, iterator_filter_builtin, iterator_find_builtin,
    iterator_flat_map_builtin, iterator_for_each_builtin, iterator_from_builtin,
    iterator_helper_next_builtin, iterator_helper_return_builtin, iterator_map_builtin,
    iterator_prototype_iterator_builtin, iterator_reduce_builtin, iterator_some_builtin,
    iterator_take_builtin, iterator_to_array_builtin, iterator_to_string_tag_getter_builtin,
    iterator_to_string_tag_setter_builtin, iterator_zip_builtin, iterator_zip_keyed_builtin,
    map_iterator_next_builtin, set_iterator_next_builtin, string_anchor_builtin, string_at_builtin,
    string_big_builtin, string_blink_builtin, string_bold_builtin, string_builtin,
    string_char_at_builtin, string_char_code_at_builtin, string_code_point_at_builtin,
    string_concat_builtin, string_ends_with_builtin, string_fixed_builtin,
    string_fontcolor_builtin, string_fontsize_builtin, string_from_char_code_builtin,
    string_from_code_point_builtin, string_includes_builtin, string_index_of_builtin,
    string_is_well_formed_builtin, string_italics_builtin, string_iterator_builtin,
    string_iterator_next_builtin, string_last_index_of_builtin, string_link_builtin,
    string_locale_compare_builtin, string_match_all_builtin, string_match_builtin,
    string_normalize_builtin, string_pad_end_builtin, string_pad_start_builtin, string_raw_builtin,
    string_repeat_builtin, string_replace_all_builtin, string_replace_builtin,
    string_search_builtin, string_slice_builtin, string_small_builtin, string_split_builtin,
    string_starts_with_builtin, string_strike_builtin, string_sub_builtin, string_substr_builtin,
    string_substring_builtin, string_sup_builtin, string_to_locale_lower_case_builtin,
    string_to_locale_upper_case_builtin, string_to_lower_case_builtin, string_to_string_builtin,
    string_to_upper_case_builtin, string_to_well_formed_builtin, string_trim_builtin,
    string_trim_end_builtin, string_trim_start_builtin, string_value_of_builtin,
};
pub(in crate::public::metadata) const PUBLIC_TEXT_BUILTIN_METADATA: &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        iterator_prototype_iterator_builtin(),
        BuiltinEntryMetadata::new("[Symbol.iterator]", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        array_iterator_next_builtin(),
        BuiltinEntryMetadata::new("next", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        map_iterator_next_builtin(),
        BuiltinEntryMetadata::new("next", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        set_iterator_next_builtin(),
        BuiltinEntryMetadata::new("next", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_builtin(),
        BuiltinEntryMetadata::new("Iterator", 0, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_from_builtin(),
        BuiltinEntryMetadata::new("from", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_concat_builtin(),
        BuiltinEntryMetadata::new("concat", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_zip_builtin(),
        BuiltinEntryMetadata::new("zip", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_zip_keyed_builtin(),
        BuiltinEntryMetadata::new("zipKeyed", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_reduce_builtin(),
        BuiltinEntryMetadata::new("reduce", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_for_each_builtin(),
        BuiltinEntryMetadata::new("forEach", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_some_builtin(),
        BuiltinEntryMetadata::new("some", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_every_builtin(),
        BuiltinEntryMetadata::new("every", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_find_builtin(),
        BuiltinEntryMetadata::new("find", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_to_array_builtin(),
        BuiltinEntryMetadata::new("toArray", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_map_builtin(),
        BuiltinEntryMetadata::new("map", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_filter_builtin(),
        BuiltinEntryMetadata::new("filter", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_take_builtin(),
        BuiltinEntryMetadata::new("take", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_drop_builtin(),
        BuiltinEntryMetadata::new("drop", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_dispose_builtin(),
        BuiltinEntryMetadata::new("[Symbol.dispose]", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        async_iterator_dispose_builtin(),
        BuiltinEntryMetadata::new("[Symbol.asyncDispose]", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_flat_map_builtin(),
        BuiltinEntryMetadata::new("flatMap", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_helper_next_builtin(),
        BuiltinEntryMetadata::new("next", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_helper_return_builtin(),
        BuiltinEntryMetadata::new("return", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_to_string_tag_getter_builtin(),
        BuiltinEntryMetadata::new("get [Symbol.toStringTag]", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_to_string_tag_setter_builtin(),
        BuiltinEntryMetadata::new("set [Symbol.toStringTag]", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_constructor_getter_builtin(),
        BuiltinEntryMetadata::new("get constructor", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        iterator_constructor_setter_builtin(),
        BuiltinEntryMetadata::new("set constructor", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_builtin(),
        BuiltinEntryMetadata::new("String", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        string_iterator_builtin(),
        BuiltinEntryMetadata::new("[Symbol.iterator]", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_iterator_next_builtin(),
        BuiltinEntryMetadata::new("next", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_to_string_builtin(),
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_value_of_builtin(),
        BuiltinEntryMetadata::new("valueOf", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_concat_builtin(),
        BuiltinEntryMetadata::new("concat", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_char_at_builtin(),
        BuiltinEntryMetadata::new("charAt", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_char_code_at_builtin(),
        BuiltinEntryMetadata::new("charCodeAt", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_from_char_code_builtin(),
        BuiltinEntryMetadata::new("fromCharCode", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_from_code_point_builtin(),
        BuiltinEntryMetadata::new("fromCodePoint", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_raw_builtin(),
        BuiltinEntryMetadata::new("raw", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_at_builtin(),
        BuiltinEntryMetadata::new("at", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_code_point_at_builtin(),
        BuiltinEntryMetadata::new("codePointAt", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_ends_with_builtin(),
        BuiltinEntryMetadata::new("endsWith", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_includes_builtin(),
        BuiltinEntryMetadata::new("includes", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_index_of_builtin(),
        BuiltinEntryMetadata::new("indexOf", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_is_well_formed_builtin(),
        BuiltinEntryMetadata::new("isWellFormed", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_locale_compare_builtin(),
        BuiltinEntryMetadata::new("localeCompare", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_match_builtin(),
        BuiltinEntryMetadata::new("match", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_match_all_builtin(),
        BuiltinEntryMetadata::new("matchAll", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_normalize_builtin(),
        BuiltinEntryMetadata::new("normalize", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_last_index_of_builtin(),
        BuiltinEntryMetadata::new("lastIndexOf", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_pad_end_builtin(),
        BuiltinEntryMetadata::new("padEnd", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_pad_start_builtin(),
        BuiltinEntryMetadata::new("padStart", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_repeat_builtin(),
        BuiltinEntryMetadata::new("repeat", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_replace_builtin(),
        BuiltinEntryMetadata::new("replace", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_replace_all_builtin(),
        BuiltinEntryMetadata::new("replaceAll", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_search_builtin(),
        BuiltinEntryMetadata::new("search", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_split_builtin(),
        BuiltinEntryMetadata::new("split", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_slice_builtin(),
        BuiltinEntryMetadata::new("slice", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_substr_builtin(),
        BuiltinEntryMetadata::new("substr", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_substring_builtin(),
        BuiltinEntryMetadata::new("substring", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_starts_with_builtin(),
        BuiltinEntryMetadata::new("startsWith", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_to_locale_lower_case_builtin(),
        BuiltinEntryMetadata::new("toLocaleLowerCase", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_to_locale_upper_case_builtin(),
        BuiltinEntryMetadata::new("toLocaleUpperCase", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_to_lower_case_builtin(),
        BuiltinEntryMetadata::new("toLowerCase", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_to_upper_case_builtin(),
        BuiltinEntryMetadata::new("toUpperCase", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_to_well_formed_builtin(),
        BuiltinEntryMetadata::new("toWellFormed", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_trim_builtin(),
        BuiltinEntryMetadata::new("trim", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_trim_end_builtin(),
        BuiltinEntryMetadata::new("trimEnd", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_trim_start_builtin(),
        BuiltinEntryMetadata::new("trimStart", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_anchor_builtin(),
        BuiltinEntryMetadata::new("anchor", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_big_builtin(),
        BuiltinEntryMetadata::new("big", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_blink_builtin(),
        BuiltinEntryMetadata::new("blink", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_bold_builtin(),
        BuiltinEntryMetadata::new("bold", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_fixed_builtin(),
        BuiltinEntryMetadata::new("fixed", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_fontcolor_builtin(),
        BuiltinEntryMetadata::new("fontcolor", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_fontsize_builtin(),
        BuiltinEntryMetadata::new("fontsize", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_italics_builtin(),
        BuiltinEntryMetadata::new("italics", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_link_builtin(),
        BuiltinEntryMetadata::new("link", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_small_builtin(),
        BuiltinEntryMetadata::new("small", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_strike_builtin(),
        BuiltinEntryMetadata::new("strike", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_sub_builtin(),
        BuiltinEntryMetadata::new("sub", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        string_sup_builtin(),
        BuiltinEntryMetadata::new("sup", 0, false, false),
    ),
];

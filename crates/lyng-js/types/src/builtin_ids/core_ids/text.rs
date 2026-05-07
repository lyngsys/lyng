use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn string_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_RAW)
}

#[inline]
pub const fn string_to_string_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_TO_STRING_RAW)
}

#[inline]
pub const fn string_value_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_VALUE_OF_RAW)
}

#[inline]
pub const fn string_concat_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_CONCAT_RAW)
}

#[inline]
pub const fn string_char_at_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_CHAR_AT_RAW)
}

#[inline]
pub const fn string_char_code_at_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_CHAR_CODE_AT_RAW)
}

#[inline]
pub const fn string_from_char_code_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_FROM_CHAR_CODE_RAW)
}

#[inline]
pub const fn string_from_code_point_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_FROM_CODE_POINT_RAW)
}

#[inline]
pub const fn string_raw_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_RAW_RAW)
}

#[inline]
pub const fn string_at_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_AT_RAW)
}

#[inline]
pub const fn string_code_point_at_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_CODE_POINT_AT_RAW)
}

#[inline]
pub const fn string_ends_with_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_ENDS_WITH_RAW)
}

#[inline]
pub const fn string_includes_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_INCLUDES_RAW)
}

#[inline]
pub const fn string_index_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_INDEX_OF_RAW)
}

#[inline]
pub const fn string_is_well_formed_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_IS_WELL_FORMED_RAW)
}

#[inline]
pub const fn string_locale_compare_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_LOCALE_COMPARE_RAW)
}

#[inline]
pub const fn string_normalize_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_NORMALIZE_RAW)
}

#[inline]
pub const fn string_replace_all_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_REPLACE_ALL_RAW)
}

#[inline]
pub const fn string_to_locale_lower_case_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_TO_LOCALE_LOWER_CASE_RAW)
}

#[inline]
pub const fn string_to_locale_upper_case_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_TO_LOCALE_UPPER_CASE_RAW)
}

#[inline]
pub const fn string_to_lower_case_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_TO_LOWER_CASE_RAW)
}

#[inline]
pub const fn string_to_upper_case_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_TO_UPPER_CASE_RAW)
}

#[inline]
pub const fn string_to_well_formed_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_TO_WELL_FORMED_RAW)
}

#[inline]
pub const fn string_anchor_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_ANCHOR_RAW)
}

#[inline]
pub const fn string_big_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_BIG_RAW)
}

#[inline]
pub const fn string_blink_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_BLINK_RAW)
}

#[inline]
pub const fn string_bold_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_BOLD_RAW)
}

#[inline]
pub const fn string_fixed_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_FIXED_RAW)
}

#[inline]
pub const fn string_fontcolor_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_FONTCOLOR_RAW)
}

#[inline]
pub const fn string_fontsize_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_FONTSIZE_RAW)
}

#[inline]
pub const fn string_italics_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_ITALICS_RAW)
}

#[inline]
pub const fn string_link_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_LINK_RAW)
}

#[inline]
pub const fn string_small_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_SMALL_RAW)
}

#[inline]
pub const fn string_strike_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_STRIKE_RAW)
}

#[inline]
pub const fn string_sub_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_SUB_RAW)
}

#[inline]
pub const fn string_sup_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_SUP_RAW)
}

#[inline]
pub const fn string_trim_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_TRIM_RAW)
}

#[inline]
pub const fn string_trim_end_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_TRIM_END_RAW)
}

#[inline]
pub const fn string_trim_start_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_TRIM_START_RAW)
}

#[inline]
pub const fn string_search_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_SEARCH_RAW)
}

#[inline]
pub const fn string_match_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_MATCH_RAW)
}

#[inline]
pub const fn string_pad_end_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_PAD_END_RAW)
}

#[inline]
pub const fn string_pad_start_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_PAD_START_RAW)
}

#[inline]
pub const fn string_replace_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_REPLACE_RAW)
}

#[inline]
pub const fn string_split_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_SPLIT_RAW)
}

#[inline]
pub const fn string_last_index_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_LAST_INDEX_OF_RAW)
}

#[inline]
pub const fn string_substring_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_SUBSTRING_RAW)
}

#[inline]
pub const fn string_substr_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_SUBSTR_RAW)
}

#[inline]
pub const fn string_starts_with_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_STARTS_WITH_RAW)
}

#[inline]
pub const fn string_repeat_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_REPEAT_RAW)
}

#[inline]
pub const fn string_match_all_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_MATCH_ALL_RAW)
}

#[inline]
pub const fn string_slice_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_SLICE_RAW)
}

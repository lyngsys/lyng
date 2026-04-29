use super::super::*;

#[inline]
pub fn string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_to_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_TO_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_value_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_VALUE_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_concat_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_CONCAT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_char_at_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_CHAR_AT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_char_code_at_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_CHAR_CODE_AT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_from_char_code_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_FROM_CHAR_CODE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_from_code_point_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_FROM_CODE_POINT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_raw_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_RAW_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_at_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_AT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_code_point_at_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_CODE_POINT_AT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_ends_with_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_ENDS_WITH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_includes_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_INCLUDES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_index_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_INDEX_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_is_well_formed_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_IS_WELL_FORMED_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_locale_compare_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_LOCALE_COMPARE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_normalize_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_NORMALIZE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_replace_all_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_REPLACE_ALL_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_to_locale_lower_case_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_TO_LOCALE_LOWER_CASE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_to_locale_upper_case_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_TO_LOCALE_UPPER_CASE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_to_lower_case_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_TO_LOWER_CASE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_to_upper_case_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_TO_UPPER_CASE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_to_well_formed_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_TO_WELL_FORMED_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_trim_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_TRIM_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_trim_end_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_TRIM_END_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_trim_start_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_TRIM_START_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_search_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_SEARCH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_match_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_MATCH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_pad_end_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_PAD_END_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_pad_start_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_PAD_START_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_replace_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_REPLACE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_split_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_SPLIT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_last_index_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_LAST_INDEX_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_substring_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_SUBSTRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_substr_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_SUBSTR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_starts_with_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_STARTS_WITH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_repeat_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_REPEAT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_match_all_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_MATCH_ALL_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn string_slice_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(STRING_SLICE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

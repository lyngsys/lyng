use super::super::*;

#[inline]
pub fn regexp_symbol_match_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_SYMBOL_MATCH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_symbol_replace_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_SYMBOL_REPLACE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_symbol_search_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_SYMBOL_SEARCH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_symbol_split_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_SYMBOL_SPLIT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_symbol_match_all_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_SYMBOL_MATCH_ALL_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_string_iterator_next_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_STRING_ITERATOR_NEXT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_to_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_TO_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_exec_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_EXEC_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_test_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_TEST_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_global_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_GLOBAL_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_ignore_case_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_IGNORE_CASE_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_multiline_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_MULTILINE_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_dot_all_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_DOT_ALL_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_unicode_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_UNICODE_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_unicode_sets_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_UNICODE_SETS_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_sticky_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_STICKY_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_source_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_SOURCE_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_flags_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_FLAGS_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_has_indices_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_HAS_INDICES_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_species_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_SPECIES_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn regexp_escape_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(REGEXP_ESCAPE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

use super::super::*;

#[inline]
pub fn abstract_module_source_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ABSTRACT_MODULE_SOURCE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn abstract_module_source_to_string_tag_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ABSTRACT_MODULE_SOURCE_TO_STRING_TAG_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

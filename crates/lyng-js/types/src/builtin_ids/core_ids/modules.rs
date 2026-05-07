use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn abstract_module_source_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ABSTRACT_MODULE_SOURCE_RAW)
}

#[inline]
pub const fn abstract_module_source_to_string_tag_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ABSTRACT_MODULE_SOURCE_TO_STRING_TAG_GETTER_RAW)
}

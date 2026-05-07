use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn internal_string_replace_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_STRING_REPLACE_RAW)
}

#[inline]
pub const fn internal_string_index_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_STRING_INDEX_OF_RAW)
}

#[inline]
pub const fn internal_object_to_string_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_OBJECT_TO_STRING_RAW)
}

#[inline]
pub const fn internal_template_to_string_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_TEMPLATE_TO_STRING_RAW)
}

#[inline]
pub const fn internal_get_template_object_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_GET_TEMPLATE_OBJECT_RAW)
}

use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn error_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ERROR_RAW)
}

#[inline]
pub const fn error_to_string_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ERROR_TO_STRING_RAW)
}

#[inline]
pub const fn eval_error_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::EVAL_ERROR_RAW)
}

#[inline]
pub const fn range_error_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::RANGE_ERROR_RAW)
}

#[inline]
pub const fn reference_error_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::REFERENCE_ERROR_RAW)
}

#[inline]
pub const fn syntax_error_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SYNTAX_ERROR_RAW)
}

#[inline]
pub const fn type_error_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TYPE_ERROR_RAW)
}

#[inline]
pub const fn uri_error_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::URI_ERROR_RAW)
}

#[inline]
pub const fn error_is_error_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ERROR_IS_ERROR_RAW)
}

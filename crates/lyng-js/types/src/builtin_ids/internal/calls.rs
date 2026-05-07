use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn internal_function_call_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_FUNCTION_CALL_RAW)
}

#[inline]
pub const fn internal_set_function_home_object_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_SET_FUNCTION_HOME_OBJECT_RAW)
}

#[inline]
pub const fn internal_capture_arrow_context_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_CAPTURE_ARROW_CONTEXT_RAW)
}

use super::super::*;

#[inline]
pub fn internal_function_call_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_FUNCTION_CALL_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_set_function_home_object_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_SET_FUNCTION_HOME_OBJECT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_capture_arrow_context_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_CAPTURE_ARROW_CONTEXT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

use super::super::*;

#[inline]
pub fn async_function_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ASYNC_FUNCTION_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

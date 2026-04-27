use super::super::*;

#[inline]
pub fn async_generator_function_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ASYNC_GENERATOR_FUNCTION_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn async_generator_next_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ASYNC_GENERATOR_NEXT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn async_generator_return_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ASYNC_GENERATOR_RETURN_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn async_generator_throw_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ASYNC_GENERATOR_THROW_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

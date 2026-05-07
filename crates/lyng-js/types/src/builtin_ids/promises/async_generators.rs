use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn async_generator_function_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ASYNC_GENERATOR_FUNCTION_RAW)
}

#[inline]
pub const fn async_generator_next_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ASYNC_GENERATOR_NEXT_RAW)
}

#[inline]
pub const fn async_generator_return_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ASYNC_GENERATOR_RETURN_RAW)
}

#[inline]
pub const fn async_generator_throw_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ASYNC_GENERATOR_THROW_RAW)
}

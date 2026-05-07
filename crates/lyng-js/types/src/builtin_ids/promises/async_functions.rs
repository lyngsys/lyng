use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn async_function_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ASYNC_FUNCTION_RAW)
}

use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn function_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::FUNCTION_RAW)
}

#[inline]
pub const fn function_prototype_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::FUNCTION_PROTOTYPE_RAW)
}

#[inline]
pub const fn function_call_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::FUNCTION_CALL_RAW)
}

#[inline]
pub const fn function_apply_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::FUNCTION_APPLY_RAW)
}

#[inline]
pub const fn function_bind_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::FUNCTION_BIND_RAW)
}

#[inline]
pub const fn function_to_string_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::FUNCTION_TO_STRING_RAW)
}

#[inline]
pub const fn generator_function_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::GENERATOR_FUNCTION_RAW)
}

#[inline]
pub const fn generator_next_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::GENERATOR_NEXT_RAW)
}

#[inline]
pub const fn generator_return_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::GENERATOR_RETURN_RAW)
}

#[inline]
pub const fn generator_throw_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::GENERATOR_THROW_RAW)
}

#[inline]
pub const fn function_symbol_has_instance_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::FUNCTION_SYMBOL_HAS_INSTANCE_RAW)
}

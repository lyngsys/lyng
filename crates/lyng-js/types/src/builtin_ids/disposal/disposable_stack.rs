use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn disposable_stack_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::DISPOSABLE_STACK_RAW)
}

#[inline]
pub const fn disposable_stack_use_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::DISPOSABLE_STACK_USE_RAW)
}

#[inline]
pub const fn disposable_stack_adopt_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::DISPOSABLE_STACK_ADOPT_RAW)
}

#[inline]
pub const fn disposable_stack_defer_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::DISPOSABLE_STACK_DEFER_RAW)
}

#[inline]
pub const fn disposable_stack_move_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::DISPOSABLE_STACK_MOVE_RAW)
}

#[inline]
pub const fn disposable_stack_disposed_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::DISPOSABLE_STACK_DISPOSED_GETTER_RAW)
}

#[inline]
pub const fn disposable_stack_dispose_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::DISPOSABLE_STACK_DISPOSE_RAW)
}

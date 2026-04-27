use super::super::*;

#[inline]
pub fn async_disposable_stack_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ASYNC_DISPOSABLE_STACK_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn async_disposable_stack_use_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ASYNC_DISPOSABLE_STACK_USE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn async_disposable_stack_adopt_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ASYNC_DISPOSABLE_STACK_ADOPT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn async_disposable_stack_defer_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ASYNC_DISPOSABLE_STACK_DEFER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn async_disposable_stack_move_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ASYNC_DISPOSABLE_STACK_MOVE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn async_disposable_stack_disposed_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ASYNC_DISPOSABLE_STACK_DISPOSED_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn async_disposable_stack_dispose_async_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ASYNC_DISPOSABLE_STACK_DISPOSE_ASYNC_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn async_disposal_resume_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ASYNC_DISPOSAL_RESUME_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

use super::super::*;

#[inline]
pub fn disposable_stack_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DISPOSABLE_STACK_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn disposable_stack_use_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DISPOSABLE_STACK_USE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn disposable_stack_adopt_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DISPOSABLE_STACK_ADOPT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn disposable_stack_defer_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DISPOSABLE_STACK_DEFER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn disposable_stack_move_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DISPOSABLE_STACK_MOVE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn disposable_stack_disposed_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DISPOSABLE_STACK_DISPOSED_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn disposable_stack_dispose_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DISPOSABLE_STACK_DISPOSE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

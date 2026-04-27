use super::super::*;

#[inline]
pub fn promise_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn promise_then_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_THEN_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn promise_catch_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_CATCH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn promise_finally_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_FINALLY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn promise_resolve_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_RESOLVE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn promise_reject_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_REJECT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn promise_all_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_ALL_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn promise_all_settled_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_ALL_SETTLED_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn promise_race_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_RACE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn promise_any_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_ANY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn promise_species_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_SPECIES_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn promise_capability_executor_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_CAPABILITY_EXECUTOR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn promise_resolve_function_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_RESOLVE_FUNCTION_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn promise_reject_function_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_REJECT_FUNCTION_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn promise_all_resolve_element_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_ALL_RESOLVE_ELEMENT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn promise_all_settled_resolve_element_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_ALL_SETTLED_RESOLVE_ELEMENT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn promise_all_settled_reject_element_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_ALL_SETTLED_REJECT_ELEMENT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn promise_any_reject_element_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_ANY_REJECT_ELEMENT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn promise_finally_function_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROMISE_FINALLY_FUNCTION_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

use super::super::*;

#[inline]
pub fn function_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(FUNCTION_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn function_prototype_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(FUNCTION_PROTOTYPE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn function_call_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(FUNCTION_CALL_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn function_apply_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(FUNCTION_APPLY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn function_bind_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(FUNCTION_BIND_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn function_to_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(FUNCTION_TO_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn generator_function_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(GENERATOR_FUNCTION_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn generator_next_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(GENERATOR_NEXT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn generator_return_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(GENERATOR_RETURN_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn generator_throw_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(GENERATOR_THROW_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn function_symbol_has_instance_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(FUNCTION_SYMBOL_HAS_INSTANCE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

use super::super::*;

#[inline]
pub fn atomics_load_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ATOMICS_LOAD_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn atomics_store_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ATOMICS_STORE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn atomics_add_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ATOMICS_ADD_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn atomics_sub_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ATOMICS_SUB_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn atomics_and_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ATOMICS_AND_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn atomics_or_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ATOMICS_OR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn atomics_xor_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ATOMICS_XOR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn atomics_exchange_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ATOMICS_EXCHANGE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn atomics_compare_exchange_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ATOMICS_COMPARE_EXCHANGE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn atomics_notify_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ATOMICS_NOTIFY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn atomics_wait_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ATOMICS_WAIT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn atomics_wait_async_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ATOMICS_WAIT_ASYNC_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn atomics_is_lock_free_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ATOMICS_IS_LOCK_FREE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

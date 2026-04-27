use super::super::*;

#[inline]
pub fn create_sync_disposal_scope_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(CREATE_SYNC_DISPOSAL_SCOPE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn create_async_disposal_scope_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(CREATE_ASYNC_DISPOSAL_SCOPE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn add_sync_disposable_resource_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ADD_SYNC_DISPOSABLE_RESOURCE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn add_async_disposable_resource_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(ADD_ASYNC_DISPOSABLE_RESOURCE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn dispose_scope_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DISPOSE_SCOPE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn dispose_scope_async_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DISPOSE_SCOPE_ASYNC_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

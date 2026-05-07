use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn create_sync_disposal_scope_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::CREATE_SYNC_DISPOSAL_SCOPE_RAW)
}

#[inline]
pub const fn create_async_disposal_scope_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::CREATE_ASYNC_DISPOSAL_SCOPE_RAW)
}

#[inline]
pub const fn add_sync_disposable_resource_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ADD_SYNC_DISPOSABLE_RESOURCE_RAW)
}

#[inline]
pub const fn add_async_disposable_resource_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ADD_ASYNC_DISPOSABLE_RESOURCE_RAW)
}

#[inline]
pub const fn dispose_scope_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::DISPOSE_SCOPE_RAW)
}

#[inline]
pub const fn dispose_scope_async_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::DISPOSE_SCOPE_ASYNC_RAW)
}

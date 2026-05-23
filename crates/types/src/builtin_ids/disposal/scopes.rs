use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    create_sync_disposal_scope_builtin => super::super::CREATE_SYNC_DISPOSAL_SCOPE_RAW;
    create_async_disposal_scope_builtin => super::super::CREATE_ASYNC_DISPOSAL_SCOPE_RAW;
    add_sync_disposable_resource_builtin => super::super::ADD_SYNC_DISPOSABLE_RESOURCE_RAW;
    add_async_disposable_resource_builtin => super::super::ADD_ASYNC_DISPOSABLE_RESOURCE_RAW;
    dispose_scope_builtin => super::super::DISPOSE_SCOPE_RAW;
    dispose_scope_async_builtin => super::super::DISPOSE_SCOPE_ASYNC_RAW;
}

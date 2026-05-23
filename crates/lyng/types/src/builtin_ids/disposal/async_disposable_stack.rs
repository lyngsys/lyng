use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    async_disposable_stack_builtin => super::super::ASYNC_DISPOSABLE_STACK_RAW;
    async_disposable_stack_use_builtin => super::super::ASYNC_DISPOSABLE_STACK_USE_RAW;
    async_disposable_stack_adopt_builtin => super::super::ASYNC_DISPOSABLE_STACK_ADOPT_RAW;
    async_disposable_stack_defer_builtin => super::super::ASYNC_DISPOSABLE_STACK_DEFER_RAW;
    async_disposable_stack_move_builtin => super::super::ASYNC_DISPOSABLE_STACK_MOVE_RAW;
    async_disposable_stack_disposed_getter_builtin => super::super::ASYNC_DISPOSABLE_STACK_DISPOSED_GETTER_RAW;
    async_disposable_stack_dispose_async_builtin => super::super::ASYNC_DISPOSABLE_STACK_DISPOSE_ASYNC_RAW;
    async_disposal_resume_builtin => super::super::ASYNC_DISPOSAL_RESUME_RAW;
}

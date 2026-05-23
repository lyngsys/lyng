use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    disposable_stack_builtin => super::super::DISPOSABLE_STACK_RAW;
    disposable_stack_use_builtin => super::super::DISPOSABLE_STACK_USE_RAW;
    disposable_stack_adopt_builtin => super::super::DISPOSABLE_STACK_ADOPT_RAW;
    disposable_stack_defer_builtin => super::super::DISPOSABLE_STACK_DEFER_RAW;
    disposable_stack_move_builtin => super::super::DISPOSABLE_STACK_MOVE_RAW;
    disposable_stack_disposed_getter_builtin => super::super::DISPOSABLE_STACK_DISPOSED_GETTER_RAW;
    disposable_stack_dispose_builtin => super::super::DISPOSABLE_STACK_DISPOSE_RAW;
}

use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn promise_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_RAW)
}

#[inline]
pub const fn promise_then_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_THEN_RAW)
}

#[inline]
pub const fn promise_catch_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_CATCH_RAW)
}

#[inline]
pub const fn promise_finally_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_FINALLY_RAW)
}

#[inline]
pub const fn promise_resolve_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_RESOLVE_RAW)
}

#[inline]
pub const fn promise_reject_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_REJECT_RAW)
}

#[inline]
pub const fn promise_all_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_ALL_RAW)
}

#[inline]
pub const fn promise_all_settled_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_ALL_SETTLED_RAW)
}

#[inline]
pub const fn promise_race_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_RACE_RAW)
}

#[inline]
pub const fn promise_any_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_ANY_RAW)
}

#[inline]
pub const fn promise_species_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_SPECIES_GETTER_RAW)
}

#[inline]
pub const fn promise_capability_executor_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_CAPABILITY_EXECUTOR_RAW)
}

#[inline]
pub const fn promise_resolve_function_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_RESOLVE_FUNCTION_RAW)
}

#[inline]
pub const fn promise_reject_function_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_REJECT_FUNCTION_RAW)
}

#[inline]
pub const fn promise_all_resolve_element_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_ALL_RESOLVE_ELEMENT_RAW)
}

#[inline]
pub const fn promise_all_settled_resolve_element_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_ALL_SETTLED_RESOLVE_ELEMENT_RAW)
}

#[inline]
pub const fn promise_all_settled_reject_element_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_ALL_SETTLED_REJECT_ELEMENT_RAW)
}

#[inline]
pub const fn promise_any_reject_element_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_ANY_REJECT_ELEMENT_RAW)
}

#[inline]
pub const fn promise_finally_function_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_FINALLY_FUNCTION_RAW)
}

#[inline]
pub const fn promise_finally_continuation_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_FINALLY_CONTINUATION_RAW)
}

#[inline]
pub const fn promise_try_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_TRY_RAW)
}

#[inline]
pub const fn promise_with_resolvers_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROMISE_WITH_RESOLVERS_RAW)
}

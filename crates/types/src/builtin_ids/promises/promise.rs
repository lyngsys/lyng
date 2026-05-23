use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    promise_builtin => super::super::PROMISE_RAW;
    promise_then_builtin => super::super::PROMISE_THEN_RAW;
    promise_catch_builtin => super::super::PROMISE_CATCH_RAW;
    promise_finally_builtin => super::super::PROMISE_FINALLY_RAW;
    promise_resolve_builtin => super::super::PROMISE_RESOLVE_RAW;
    promise_reject_builtin => super::super::PROMISE_REJECT_RAW;
    promise_all_builtin => super::super::PROMISE_ALL_RAW;
    promise_all_settled_builtin => super::super::PROMISE_ALL_SETTLED_RAW;
    promise_race_builtin => super::super::PROMISE_RACE_RAW;
    promise_any_builtin => super::super::PROMISE_ANY_RAW;
    promise_species_getter_builtin => super::super::PROMISE_SPECIES_GETTER_RAW;
    promise_capability_executor_builtin => super::super::PROMISE_CAPABILITY_EXECUTOR_RAW;
    promise_resolve_function_builtin => super::super::PROMISE_RESOLVE_FUNCTION_RAW;
    promise_reject_function_builtin => super::super::PROMISE_REJECT_FUNCTION_RAW;
    promise_all_resolve_element_builtin => super::super::PROMISE_ALL_RESOLVE_ELEMENT_RAW;
    promise_all_settled_resolve_element_builtin => super::super::PROMISE_ALL_SETTLED_RESOLVE_ELEMENT_RAW;
    promise_all_settled_reject_element_builtin => super::super::PROMISE_ALL_SETTLED_REJECT_ELEMENT_RAW;
    promise_any_reject_element_builtin => super::super::PROMISE_ANY_REJECT_ELEMENT_RAW;
    promise_finally_function_builtin => super::super::PROMISE_FINALLY_FUNCTION_RAW;
    promise_finally_continuation_builtin => super::super::PROMISE_FINALLY_CONTINUATION_RAW;
    promise_try_builtin => super::super::PROMISE_TRY_RAW;
    promise_with_resolvers_builtin => super::super::PROMISE_WITH_RESOLVERS_RAW;
}

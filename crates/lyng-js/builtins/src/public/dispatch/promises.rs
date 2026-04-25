use super::PublicBuiltinDispatchContext;
use crate::BuiltinInvocation;
use lyng_js_types::{BuiltinFunctionId, Value};

pub(super) fn dispatch_promise_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_promise_constructor_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_promise_prototype_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_promise_internal_builtin(context, entry, invocation)
}

fn dispatch_promise_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_promise_builtin() {
        return super::promise_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_promise_resolve_builtin() {
        return super::promise_resolve_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_promise_reject_builtin() {
        return super::promise_reject_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_promise_all_builtin() {
        return super::promise_all_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_promise_all_settled_builtin() {
        return super::promise_all_settled_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_promise_race_builtin() {
        return super::promise_race_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_promise_any_builtin() {
        return super::promise_any_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_promise_species_getter_builtin() {
        return super::promise_species_getter_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_promise_prototype_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_promise_then_builtin() {
        return super::promise_then_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_promise_catch_builtin() {
        return super::promise_catch_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_promise_finally_builtin() {
        return super::promise_finally_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_promise_internal_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_promise_capability_executor_builtin() {
        return super::promise_capability_executor_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_promise_resolve_function_builtin() {
        return super::promise_resolve_function_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_promise_reject_function_builtin() {
        return super::promise_reject_function_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_promise_finally_function_builtin() {
        return super::promise_finally_function_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_promise_all_resolve_element_builtin() {
        return super::promise_combinator_element_builtin(
            context,
            invocation,
            super::PromiseCombinatorElementKind::AllResolve,
        )
        .map(Some);
    }
    if entry == super::js3_promise_all_settled_resolve_element_builtin() {
        return super::promise_combinator_element_builtin(
            context,
            invocation,
            super::PromiseCombinatorElementKind::AllSettledResolve,
        )
        .map(Some);
    }
    if entry == super::js3_promise_all_settled_reject_element_builtin() {
        return super::promise_combinator_element_builtin(
            context,
            invocation,
            super::PromiseCombinatorElementKind::AllSettledReject,
        )
        .map(Some);
    }
    if entry == super::js3_promise_any_reject_element_builtin() {
        return super::promise_combinator_element_builtin(
            context,
            invocation,
            super::PromiseCombinatorElementKind::AnyReject,
        )
        .map(Some);
    }
    Ok(None)
}

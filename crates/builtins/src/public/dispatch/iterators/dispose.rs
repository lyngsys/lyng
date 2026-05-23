use super::{
    errors, new_promise_capability, perform_promise_then_with_capability,
    promise_capability_promise, promise_capability_reject, promise_capability_resolve,
    promise_default_constructor, promise_resolve_method, property_key_from_text, BuiltinInvocation,
    PromiseCapabilityId, PromiseReactionHandler, PublicBuiltinDispatchContext, Value,
};

pub(super) fn iterator_dispose_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let this = invocation.this_value();
    let return_key = property_key_from_text(cx, "return");
    let return_value = cx.get_property_value(this, return_key)?;
    if return_value.is_undefined() || return_value.is_null() {
        return Ok(Value::undefined());
    }
    let return_method = cx.require_callable_object(return_value)?;
    cx.call_to_completion(return_method, this, &[])?;
    Ok(Value::undefined())
}

fn reject_async_iterator_dispose_error<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    capability: PromiseCapabilityId,
    error: Cx::Error,
) -> Result<Value, Cx::Error> {
    let Some(thrown) = cx.extract_thrown_value(error)? else {
        unreachable!("non-abrupt builtin error should propagate")
    };
    reject_async_iterator_dispose_value(cx, capability, thrown)
}

fn reject_async_iterator_dispose_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    capability: PromiseCapabilityId,
    reason: Value,
) -> Result<Value, Cx::Error> {
    let reject = promise_capability_reject(cx, capability)?;
    let _ = cx.call_to_completion(reject, Value::undefined(), &[reason])?;
    let promise = promise_capability_promise(cx, capability)?;
    Ok(Value::from_object_ref(promise))
}

pub(super) fn async_iterator_dispose_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let promise_constructor = promise_default_constructor(cx)?;
    let capability = new_promise_capability(cx, promise_constructor)?;
    let promise = promise_capability_promise(cx, capability)?;
    let receiver = invocation.this_value();
    let return_key = property_key_from_text(cx, "return");
    let return_method = match cx.get_property_value(receiver, return_key) {
        Ok(method) if method.is_undefined() || method.is_null() => None,
        Ok(method) => match cx.require_callable_object(method) {
            Ok(method) => Some(method),
            Err(error) => return reject_async_iterator_dispose_error(cx, capability, error),
        },
        Err(error) => return reject_async_iterator_dispose_error(cx, capability, error),
    };
    let Some(return_method) = return_method else {
        let resolve = promise_capability_resolve(cx, capability)?;
        let _ = cx.call_to_completion(resolve, Value::undefined(), &[Value::undefined()])?;
        return Ok(Value::from_object_ref(promise));
    };
    let result = match cx.call_to_completion(return_method, receiver, &[Value::undefined()]) {
        Ok(result) => result,
        Err(error) => return reject_async_iterator_dispose_error(cx, capability, error),
    };
    let promise_resolve = match promise_resolve_method(cx, promise_constructor) {
        Ok(resolve) => resolve,
        Err(error) => return reject_async_iterator_dispose_error(cx, capability, error),
    };
    let result_wrapper = match cx.call_to_completion(
        promise_resolve,
        Value::from_object_ref(promise_constructor),
        &[result],
    ) {
        Ok(result_wrapper) => result_wrapper,
        Err(error) => return reject_async_iterator_dispose_error(cx, capability, error),
    };
    let Some(result_wrapper) = result_wrapper
        .as_object_ref()
        .filter(|object| cx.agent().promise_record(*object).is_some())
    else {
        let reason = errors::throw_type_error(cx.agent())
            .thrown_value()
            .unwrap_or(Value::undefined());
        return reject_async_iterator_dispose_value(cx, capability, reason);
    };
    perform_promise_then_with_capability(
        cx,
        result_wrapper,
        PromiseReactionHandler::PassThrough(Value::undefined()),
        PromiseReactionHandler::Thrower,
        Some(capability),
    )?;
    Ok(Value::from_object_ref(promise))
}

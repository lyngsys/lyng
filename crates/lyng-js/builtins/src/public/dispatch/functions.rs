use super::PublicBuiltinDispatchContext;
use crate::BuiltinInvocation;
use lyng_js_types::{BuiltinFunctionId, Value};

pub(super) fn dispatch_function_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_function_constructor_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_function_prototype_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_generator_builtin(context, entry, invocation)
}

fn dispatch_function_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_function_builtin() {
        return super::function_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_async_function_builtin() {
        return super::async_function_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_generator_function_builtin() {
        return super::generator_function_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_async_generator_function_builtin() {
        return super::async_generator_function_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_function_prototype_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_function_prototype_builtin() {
        return super::function_prototype_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_function_call_builtin() {
        return super::function_call_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_function_apply_builtin() {
        return super::function_apply_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_function_bind_builtin() {
        return super::function_bind_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_function_to_string_builtin() {
        return super::function_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_function_symbol_has_instance_builtin() {
        return super::function_symbol_has_instance_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_generator_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_async_generator_next_builtin() {
        return super::async_generator_next_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_async_generator_return_builtin() {
        return super::async_generator_return_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_async_generator_throw_builtin() {
        return super::async_generator_throw_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_generator_next_builtin() {
        return super::generator_next_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_generator_return_builtin() {
        return super::generator_return_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_generator_throw_builtin() {
        return super::generator_throw_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

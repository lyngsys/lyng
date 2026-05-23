use super::{string_value, type_error, BuiltinProxyBridge, PublicBuiltinDispatchContext};
use crate::{BuiltinInvocation, DynamicFunctionKind};
use lyng_js_common::WellKnownAtom;
use lyng_js_env::Agent;
use lyng_js_objects::FunctionEntryIdentity;
use lyng_js_ops::object;
use lyng_js_types::{BuiltinFunctionId, ObjectRef, PropertyKey, Value};

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
    if entry == super::function_builtin() {
        return function_builtin(context, invocation).map(Some);
    }
    if entry == super::async_function_builtin() {
        return async_function_builtin(context, invocation).map(Some);
    }
    if entry == super::generator_function_builtin() {
        return generator_function_builtin(context, invocation).map(Some);
    }
    if entry == super::async_generator_function_builtin() {
        return async_generator_function_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_function_prototype_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::function_prototype_builtin() {
        return function_prototype_builtin(context, invocation).map(Some);
    }
    if entry == super::function_call_builtin() {
        return function_call_builtin(context, invocation).map(Some);
    }
    if entry == super::function_apply_builtin() {
        return function_apply_builtin(context, invocation).map(Some);
    }
    if entry == super::function_bind_builtin() {
        return function_bind_builtin(context, invocation).map(Some);
    }
    if entry == super::function_to_string_builtin() {
        return function_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::function_symbol_has_instance_builtin() {
        return function_symbol_has_instance_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_generator_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::async_generator_next_builtin() {
        return async_generator_next_builtin(context, invocation).map(Some);
    }
    if entry == super::async_generator_return_builtin() {
        return async_generator_return_builtin(context, invocation).map(Some);
    }
    if entry == super::async_generator_throw_builtin() {
        return async_generator_throw_builtin(context, invocation).map(Some);
    }
    if entry == super::generator_next_builtin() {
        return generator_next_builtin(context, invocation).map(Some);
    }
    if entry == super::generator_return_builtin() {
        return generator_return_builtin(context, invocation).map(Some);
    }
    if entry == super::generator_throw_builtin() {
        return generator_throw_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn function_constructor_source<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    arguments: &[Value],
) -> Result<(String, String), Cx::Error> {
    if arguments.is_empty() {
        return Ok((String::new(), String::new()));
    }
    let body_index = arguments.len().saturating_sub(1);
    let mut parameters = String::new();
    for (index, value) in arguments[..body_index].iter().copied().enumerate() {
        if index != 0 {
            parameters.push(',');
        }
        parameters.push_str(&cx.value_to_string_text(value)?);
    }
    let body = cx.value_to_string_text(arguments[body_index])?;
    Ok((parameters, body))
}

fn function_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let (parameters_source, body_source) = function_constructor_source(cx, invocation.arguments())?;
    let function = cx.create_dynamic_function(
        realm,
        &parameters_source,
        &body_source,
        cx.caller_is_strict(),
        DynamicFunctionKind::Ordinary,
        invocation.new_target(),
    )?;
    Ok(Value::from_object_ref(function))
}

fn generator_function_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let (parameters_source, body_source) = function_constructor_source(cx, invocation.arguments())?;
    let function = cx.create_dynamic_function(
        realm,
        &parameters_source,
        &body_source,
        cx.caller_is_strict(),
        DynamicFunctionKind::Generator,
        invocation.new_target(),
    )?;
    Ok(Value::from_object_ref(function))
}

fn async_function_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let (parameters_source, body_source) = function_constructor_source(cx, invocation.arguments())?;
    let function = cx.create_dynamic_function(
        realm,
        &parameters_source,
        &body_source,
        cx.caller_is_strict(),
        DynamicFunctionKind::Async,
        invocation.new_target(),
    )?;
    Ok(Value::from_object_ref(function))
}

fn async_generator_function_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let (parameters_source, body_source) = function_constructor_source(cx, invocation.arguments())?;
    let function = cx.create_dynamic_function(
        realm,
        &parameters_source,
        &body_source,
        cx.caller_is_strict(),
        DynamicFunctionKind::AsyncGenerator,
        invocation.new_target(),
    )?;
    Ok(Value::from_object_ref(function))
}

fn function_prototype_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.new_target().is_some() {
        return Err(type_error(cx));
    }
    Ok(Value::undefined())
}

fn function_call_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target = cx.require_callable_object(invocation.this_value())?;
    let rebound_this = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    cx.call_to_completion(
        target,
        rebound_this,
        invocation.arguments().get(1..).unwrap_or(&[]),
    )
}

fn function_apply_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target = cx.require_callable_object(invocation.this_value())?;
    let rebound_this = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let arguments_value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    if let Some(result) = cx.try_fast_apply_builtin(target, rebound_this, arguments_value)? {
        return Ok(result);
    }
    let apply_arguments = cx.collect_array_like_arguments(cx.builtin_realm(), arguments_value)?;
    cx.call_to_completion(target, rebound_this, &apply_arguments)
}

fn function_bind_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target = cx.require_callable_object(invocation.this_value())?;
    let bound_this = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let function = cx.create_bound_function(
        target,
        bound_this,
        invocation.arguments().get(1..).unwrap_or(&[]),
    )?;
    Ok(Value::from_object_ref(function))
}

fn function_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let function = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let text = cx.function_to_string_text(function)?;
    Ok(string_value(cx, &text))
}

fn function_symbol_has_instance_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    Ok(Value::from_bool(ordinary_has_instance(
        cx,
        invocation.this_value(),
        value,
    )?))
}

fn ordinary_has_instance<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor: Value,
    value: Value,
) -> Result<bool, Cx::Error> {
    let Some(constructor) = constructor.as_object_ref() else {
        return Ok(false);
    };
    let callable = {
        let agent = cx.agent();
        agent.objects().is_callable(constructor)
    };
    if !callable {
        return Ok(false);
    }
    if let Some(target) = {
        let agent = cx.agent();
        bound_function_target(agent, constructor)
    } {
        return ordinary_has_instance(cx, Value::from_object_ref(target), value);
    }
    let Some(object) = value.as_object_ref() else {
        return Ok(false);
    };

    let prototype = {
        let mut bridge = BuiltinProxyBridge { cx };
        object::get_with_receiver_in_context(
            &mut bridge,
            constructor,
            PropertyKey::from_atom(WellKnownAtom::prototype.id()),
            Value::from_object_ref(constructor),
        )?
    }
    .as_object_ref()
    .ok_or_else(|| type_error(cx))?;

    let mut current = {
        let mut bridge = BuiltinProxyBridge { cx };
        object::get_prototype_of_in_context(&mut bridge, object)?
    };
    while let Some(candidate) = current {
        if candidate == prototype {
            return Ok(true);
        }
        current = {
            let mut bridge = BuiltinProxyBridge { cx };
            object::get_prototype_of_in_context(&mut bridge, candidate)?
        };
    }
    Ok(false)
}

fn bound_function_target(agent: &Agent, function: ObjectRef) -> Option<ObjectRef> {
    let data = agent.objects().function_data(function)?;
    if data.entry()? != FunctionEntryIdentity::Bound {
        return None;
    }
    let payload = data.gc_payload()?;
    agent
        .heap()
        .view()
        .function_payload(payload)?
        .bound()
        .map(lyng_js_gc::RuntimeBoundFunctionRecord::target)
}

fn async_generator_next_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    cx.async_generator_next(
        invocation.this_value(),
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )
}

fn async_generator_return_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    cx.async_generator_return(
        invocation.this_value(),
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )
}

fn async_generator_throw_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    cx.async_generator_throw(
        invocation.this_value(),
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )
}

fn generator_next_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let generator = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    cx.generator_next(
        generator,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )
}

fn generator_return_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let generator = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    cx.generator_return(
        generator,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )
}

fn generator_throw_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let generator = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    cx.generator_throw(
        generator,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )
}

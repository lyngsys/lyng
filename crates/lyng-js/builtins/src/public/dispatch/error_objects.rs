use super::{
    create_array_from_values, define_data_property_with_attrs, has_property_on_object,
    iterable_to_values_list, map_completion, property_key_from_text, string_value, type_error,
    PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_common::WellKnownAtom;
use lyng_js_ops::errors;
use lyng_js_types::{BuiltinFunctionId, ObjectRef, PropertyKey, Value};

pub(super) fn dispatch_error_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::error_builtin() {
        return error_builtin(context, invocation).map(Some);
    }
    if entry == super::error_to_string_builtin() {
        return error_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::eval_error_builtin() {
        return eval_error_builtin(context, invocation).map(Some);
    }
    if entry == super::range_error_builtin() {
        return range_error_builtin(context, invocation).map(Some);
    }
    if entry == super::reference_error_builtin() {
        return reference_error_builtin(context, invocation).map(Some);
    }
    if entry == super::syntax_error_builtin() {
        return syntax_error_builtin(context, invocation).map(Some);
    }
    if entry == super::type_error_builtin() {
        return type_error_builtin(context, invocation).map(Some);
    }
    if entry == super::uri_error_builtin() {
        return uri_error_builtin(context, invocation).map(Some);
    }
    if entry == super::aggregate_error_builtin() {
        return aggregate_error_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::suppressed_error_builtin() {
        return suppressed_error_builtin(context, invocation).map(Some);
    }
    if entry == super::error_is_error_builtin() {
        return Ok(Some(error_is_error_value(context, invocation)));
    }
    Ok(None)
}

fn error_is_error_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Value {
    let argument = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(object_ref) = argument.as_object_ref() else {
        return Value::from_bool(false);
    };
    let agent = cx.agent();
    let is_error = agent
        .objects()
        .object_header(agent.heap().view(), object_ref)
        .is_some_and(|header| header.flags().is_error_object());
    Value::from_bool(is_error)
}

fn error_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: errors::ErrorKind,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        errors::intrinsic_error_prototype_for_realm(agent, realm, kind)
    }
    .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?;
    let message = optional_error_message_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let options = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let error_object = {
        let agent = cx.agent();
        errors::create_error_object(agent, realm, Some(prototype), message)
    };
    let error_object = map_completion(cx, error_object)?;
    install_error_cause(cx, error_object, options)?;
    Ok(Value::from_object_ref(error_object))
}

fn optional_error_message_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Option<Value>, Cx::Error> {
    if value.is_undefined() {
        return Ok(None);
    }
    let text = cx.value_to_string_text(value)?;
    Ok(Some(string_value(cx, &text)))
}

fn install_error_cause<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    error: ObjectRef,
    options: Value,
) -> Result<(), Cx::Error> {
    let Some(options_object) = options.as_object_ref() else {
        return Ok(());
    };
    let cause_key = property_key_from_text(cx, "cause");
    if !has_property_on_object(cx, options_object, cause_key)? {
        return Ok(());
    }
    let cause = cx.get_property_value(Value::from_object_ref(options_object), cause_key)?;
    define_data_property_with_attrs(cx, error, cause_key, cause, true, false, true)
}

fn error_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    error_constructor_builtin(cx, invocation, errors::ErrorKind::Error)
}

fn eval_error_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    error_constructor_builtin(cx, invocation, errors::ErrorKind::Eval)
}

fn range_error_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    error_constructor_builtin(cx, invocation, errors::ErrorKind::Range)
}

fn reference_error_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    error_constructor_builtin(cx, invocation, errors::ErrorKind::Reference)
}

fn syntax_error_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    error_constructor_builtin(cx, invocation, errors::ErrorKind::Syntax)
}

fn type_error_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    error_constructor_builtin(cx, invocation, errors::ErrorKind::Type)
}

fn uri_error_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    error_constructor_builtin(cx, invocation, errors::ErrorKind::Uri)
}

fn aggregate_error_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let default_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().aggregate_error_prototype())
        .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?;
    let errors_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let message = optional_error_message_value(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let options = invocation
        .arguments()
        .get(2)
        .copied()
        .unwrap_or(Value::undefined());
    let error = {
        let agent = cx.agent();
        errors::create_error_object(agent, realm, Some(prototype), message)
    };
    let error = map_completion(cx, error)?;
    install_error_cause(cx, error, options)?;
    let values = iterable_to_values_list(cx, errors_value)?;
    let errors_array = create_array_from_values(cx, &values)?;
    let errors_key = property_key_from_text(cx, "errors");
    define_data_property_with_attrs(
        cx,
        error,
        errors_key,
        Value::from_object_ref(errors_array),
        true,
        false,
        true,
    )?;
    Ok(Value::from_object_ref(error))
}

fn suppressed_error_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let default_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().suppressed_error_prototype())
        .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?;
    let error_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let suppressed_value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let message = optional_error_message_value(
        cx,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let options = invocation
        .arguments()
        .get(3)
        .copied()
        .unwrap_or(Value::undefined());
    let error = create_suppressed_error_with_prototype(
        cx,
        prototype,
        error_value,
        suppressed_value,
        message,
        options,
    )?;
    Ok(Value::from_object_ref(error))
}

fn create_suppressed_error_with_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: ObjectRef,
    error_value: Value,
    suppressed_value: Value,
    message: Option<Value>,
    options: Value,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    let error = {
        let agent = cx.agent();
        errors::create_error_object(agent, realm, Some(prototype), message)
    };
    let error = map_completion(cx, error)?;
    install_error_cause(cx, error, options)?;
    let error_key = property_key_from_text(cx, "error");
    define_data_property_with_attrs(cx, error, error_key, error_value, true, false, true)?;
    let suppressed_key = property_key_from_text(cx, "suppressed");
    define_data_property_with_attrs(
        cx,
        error,
        suppressed_key,
        suppressed_value,
        true,
        false,
        true,
    )?;
    Ok(error)
}

pub(super) fn create_suppressed_error_from_values<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    error_value: Value,
    suppressed_value: Value,
    message: Option<Value>,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().suppressed_error_prototype())
        .ok_or_else(|| type_error(cx))?;
    create_suppressed_error_with_prototype(
        cx,
        prototype,
        error_value,
        suppressed_value,
        message,
        Value::undefined(),
    )
}

fn error_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let name = cx.get_property_value(
        Value::from_object_ref(object_ref),
        PropertyKey::from_atom(WellKnownAtom::name.id()),
    )?;
    let message = {
        let message_atom = {
            let agent = cx.agent();
            agent.bootstrap_atoms().message()
        };
        cx.get_property_value(
            Value::from_object_ref(object_ref),
            PropertyKey::from_atom(message_atom),
        )?
    };
    let name_text = if name.is_undefined() {
        "Error".to_owned()
    } else {
        cx.value_to_string_text(name)?
    };
    let message_text = if message.is_undefined() {
        String::new()
    } else {
        cx.value_to_string_text(message)?
    };
    let text = if name_text.is_empty() {
        message_text
    } else if message_text.is_empty() {
        name_text
    } else {
        format!("{name_text}: {message_text}")
    };
    Ok(string_value(cx, &text))
}

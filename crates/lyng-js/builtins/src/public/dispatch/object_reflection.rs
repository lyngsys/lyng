use super::{
    allocate_proxy_object, create_array_result, create_data_property_or_throw,
    get_property_from_object_with_receiver, has_property_on_object, property_key_from_text,
    property_key_value, proxy_define_property, proxy_delete_property, proxy_get_own_property,
    proxy_get_prototype_of, proxy_is_extensible, proxy_own_property_keys, proxy_prevent_extensions,
    proxy_set_prototype_of, require_constructor_object, require_object_argument,
    require_proxy_argument_object, set_property_on_object, set_property_on_object_with_receiver,
    type_error, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::FunctionObjectData;
use lyng_js_types::{BuiltinFunctionId, PropertyKey, Value};

pub(super) fn dispatch_object_reflection_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_reflect_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_proxy_builtin(context, entry, invocation)
}

fn dispatch_reflect_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::js3_reflect_apply_builtin() {
        return reflect_apply_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_construct_builtin() {
        return reflect_construct_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_define_property_builtin() {
        return reflect_define_property_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_delete_property_builtin() {
        return reflect_delete_property_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_get_builtin() {
        return reflect_get_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_get_own_property_descriptor_builtin() {
        return reflect_get_own_property_descriptor_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_get_prototype_of_builtin() {
        return reflect_get_prototype_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_has_builtin() {
        return reflect_has_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_is_extensible_builtin() {
        return reflect_is_extensible_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_own_keys_builtin() {
        return reflect_own_keys_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_prevent_extensions_builtin() {
        return reflect_prevent_extensions_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_set_builtin() {
        return reflect_set_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_set_prototype_of_builtin() {
        return reflect_set_prototype_of_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_proxy_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::js3_proxy_builtin() {
        return proxy_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_proxy_revocable_builtin() {
        return proxy_revocable_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_proxy_revoke_builtin() {
        return proxy_revoke_builtin(context).map(Some);
    }
    Ok(None)
}

fn proxy_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let target = require_proxy_argument_object(cx, invocation, 0)?;
    let handler = require_proxy_argument_object(cx, invocation, 1)?;
    let proxy = allocate_proxy_object(cx, cx.builtin_realm(), target, handler)?;
    Ok(Value::from_object_ref(proxy))
}

fn proxy_revocable_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target = require_proxy_argument_object(cx, invocation, 0)?;
    let handler = require_proxy_argument_object(cx, invocation, 1)?;
    let realm = cx.builtin_realm();
    let proxy = allocate_proxy_object(cx, realm, target, handler)?;
    let revoke = cx.allocate_builtin_function(lyng_js_types::js3_proxy_revoke_builtin())?;
    let object_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().object_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let pair = cx.allocate_ordinary_object_with_prototype(realm, Some(object_prototype))?;
    let installed = cx.agent().with_heap_and_objects(|heap, objects| {
        objects.set_function_home_object(&mut heap.mutator(), revoke, Some(proxy))
    });
    if !installed {
        return Err(type_error(cx));
    }
    let proxy_key = property_key_from_text(cx, "proxy");
    create_data_property_or_throw(cx, pair, proxy_key, Value::from_object_ref(proxy))?;
    let revoke_key = property_key_from_text(cx, "revoke");
    create_data_property_or_throw(cx, pair, revoke_key, Value::from_object_ref(revoke))?;
    Ok(Value::from_object_ref(pair))
}

fn proxy_revoke_builtin<Cx: PublicBuiltinDispatchContext>(cx: &mut Cx) -> Result<Value, Cx::Error> {
    let callee = cx.callee_object();
    let proxy = {
        let agent = cx.agent();
        agent
            .objects()
            .function_data(callee)
            .and_then(FunctionObjectData::home_object)
    };
    let Some(proxy) = proxy else {
        return Ok(Value::undefined());
    };
    let (revoked, cleared) = cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let revoked = objects.revoke_proxy(&mut mutator, proxy);
        let cleared = objects.set_function_home_object(&mut mutator, callee, None);
        (revoked, cleared)
    });
    if !revoked || !cleared {
        return Err(type_error(cx));
    }
    Ok(Value::undefined())
}

fn reflect_apply_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target = cx.require_callable_object(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let this_arg = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let arguments_list = require_object_argument(cx, invocation, 2)?;
    let arguments = cx
        .collect_array_like_arguments(cx.builtin_realm(), Value::from_object_ref(arguments_list))?;
    cx.call_to_completion(target, this_arg, &arguments)
}

fn reflect_construct_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target = require_constructor_object(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let arguments_list = require_object_argument(cx, invocation, 1)?;
    let arguments = cx
        .collect_array_like_arguments(cx.builtin_realm(), Value::from_object_ref(arguments_list))?;
    let new_target = invocation
        .arguments()
        .get(2)
        .copied()
        .map(|value| require_constructor_object(cx, value))
        .transpose()?;
    let object = cx.construct_to_completion(target, &arguments, new_target)?;
    Ok(Value::from_object_ref(object))
}

fn reflect_define_property_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let descriptor_object = require_object_argument(cx, invocation, 2)?;
    let descriptor = cx.to_property_descriptor(descriptor_object)?;
    let defined =
        proxy_define_property(cx, object_ref, key, descriptor, AllocationLifetime::Default)?;
    Ok(Value::from_bool(defined))
}

fn reflect_delete_property_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(proxy_delete_property(
        cx, object_ref, key,
    )?))
}

fn reflect_get_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let receiver = invocation
        .arguments()
        .get(2)
        .copied()
        .unwrap_or_else(|| Value::from_object_ref(object_ref));
    get_property_from_object_with_receiver(cx, object_ref, key, receiver)
}

fn reflect_get_own_property_descriptor_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let Some(descriptor) = proxy_get_own_property(cx, object_ref, key)? else {
        return Ok(Value::undefined());
    };
    cx.descriptor_object_from_descriptor(cx.builtin_realm(), descriptor)
}

fn reflect_get_prototype_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    Ok(proxy_get_prototype_of(cx, object_ref)?.map_or(Value::null(), Value::from_object_ref))
}

fn reflect_has_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(has_property_on_object(
        cx, object_ref, key,
    )?))
}

fn reflect_is_extensible_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    Ok(Value::from_bool(proxy_is_extensible(cx, object_ref)?))
}

fn reflect_own_keys_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    let keys = proxy_own_property_keys(cx, object_ref)?;
    let result = create_array_result(cx, keys.len())?;
    for (index, key) in keys.into_iter().enumerate() {
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        let value = property_key_value(cx, key);
        set_property_on_object(cx, result, PropertyKey::Index(index), value)?;
    }
    Ok(Value::from_object_ref(result))
}

fn reflect_prevent_extensions_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    Ok(Value::from_bool(proxy_prevent_extensions(cx, object_ref)?))
}

fn reflect_set_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let value = invocation
        .arguments()
        .get(2)
        .copied()
        .unwrap_or(Value::undefined());
    let receiver = invocation
        .arguments()
        .get(3)
        .copied()
        .unwrap_or_else(|| Value::from_object_ref(object_ref));
    Ok(Value::from_bool(set_property_on_object_with_receiver(
        cx, object_ref, key, value, receiver,
    )?))
}

fn reflect_set_prototype_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = require_object_argument(cx, invocation, 0)?;
    let prototype_value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let prototype = if prototype_value.is_null() {
        None
    } else if let Some(object_ref) = prototype_value.as_object_ref() {
        Some(object_ref)
    } else {
        return Err(type_error(cx));
    };
    Ok(Value::from_bool(proxy_set_prototype_of(
        cx, object_ref, prototype,
    )?))
}

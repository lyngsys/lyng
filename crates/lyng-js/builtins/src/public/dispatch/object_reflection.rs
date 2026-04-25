use super::PublicBuiltinDispatchContext;
use crate::BuiltinInvocation;
use lyng_js_types::{BuiltinFunctionId, Value};

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
        return super::reflect_apply_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_construct_builtin() {
        return super::reflect_construct_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_define_property_builtin() {
        return super::reflect_define_property_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_delete_property_builtin() {
        return super::reflect_delete_property_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_get_builtin() {
        return super::reflect_get_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_get_own_property_descriptor_builtin() {
        return super::reflect_get_own_property_descriptor_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_get_prototype_of_builtin() {
        return super::reflect_get_prototype_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_has_builtin() {
        return super::reflect_has_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_is_extensible_builtin() {
        return super::reflect_is_extensible_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_own_keys_builtin() {
        return super::reflect_own_keys_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_prevent_extensions_builtin() {
        return super::reflect_prevent_extensions_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_set_builtin() {
        return super::reflect_set_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_reflect_set_prototype_of_builtin() {
        return super::reflect_set_prototype_of_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_proxy_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::js3_proxy_builtin() {
        return super::proxy_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_proxy_revocable_builtin() {
        return super::proxy_revocable_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_proxy_revoke_builtin() {
        return super::proxy_revoke_builtin(context).map(Some);
    }
    Ok(None)
}

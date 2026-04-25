use super::PublicBuiltinDispatchContext;
use crate::BuiltinInvocation;
use lyng_js_types::{BuiltinFunctionId, Value};

pub(super) fn dispatch_language_support_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_module_source_builtin(context, entry)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_error_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_disposal_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_global_builtin(context, entry, invocation)
}

fn dispatch_module_source_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_abstract_module_source_builtin() {
        return Err(super::type_error(context));
    }
    if entry == super::js3_abstract_module_source_to_string_tag_getter_builtin() {
        return Ok(Some(Value::undefined()));
    }
    Ok(None)
}

fn dispatch_error_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_error_builtin() {
        return super::error_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_error_to_string_builtin() {
        return super::error_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_eval_error_builtin() {
        return super::eval_error_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_range_error_builtin() {
        return super::range_error_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_reference_error_builtin() {
        return super::reference_error_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_syntax_error_builtin() {
        return super::syntax_error_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_type_error_builtin() {
        return super::type_error_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_uri_error_builtin() {
        return super::uri_error_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_aggregate_error_builtin() {
        return super::aggregate_error_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_suppressed_error_builtin() {
        return super::suppressed_error_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_disposal_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_disposal_stack_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_async_disposal_stack_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_disposal_scope_builtin(context, entry, invocation)
}

fn dispatch_disposal_stack_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::js3_disposable_stack_builtin() {
        return super::disposable_stack_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_disposable_stack_use_builtin() {
        return super::disposable_stack_use_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_disposable_stack_adopt_builtin() {
        return super::disposable_stack_adopt_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_disposable_stack_defer_builtin() {
        return super::disposable_stack_defer_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_disposable_stack_move_builtin() {
        return super::disposable_stack_move_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_disposable_stack_disposed_getter_builtin() {
        return super::disposable_stack_disposed_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_disposable_stack_dispose_builtin() {
        return super::disposal_stack_dispose_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_async_disposal_stack_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::js3_async_disposable_stack_builtin() {
        return super::async_disposable_stack_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_async_disposable_stack_use_builtin() {
        return super::async_disposable_stack_use_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_async_disposable_stack_adopt_builtin() {
        return super::async_disposable_stack_adopt_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_async_disposable_stack_defer_builtin() {
        return super::async_disposable_stack_defer_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_async_disposable_stack_move_builtin() {
        return super::async_disposable_stack_move_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_async_disposable_stack_disposed_getter_builtin() {
        return super::async_disposable_stack_disposed_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::js3_async_disposable_stack_dispose_async_builtin() {
        return super::async_disposable_stack_dispose_async_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_async_disposal_resume_builtin() {
        return super::async_disposal_resume_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_disposal_scope_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::js3_create_sync_disposal_scope_builtin() {
        return super::create_disposal_scope_builtin(
            context,
            lyng_js_env::DisposalCapabilityKind::Sync,
        )
        .map(Some);
    }
    if entry == lyng_js_types::js3_create_async_disposal_scope_builtin() {
        return super::create_disposal_scope_builtin(
            context,
            lyng_js_env::DisposalCapabilityKind::Async,
        )
        .map(Some);
    }
    if entry == lyng_js_types::js3_add_sync_disposable_resource_builtin() {
        return super::add_disposal_scope_resource_builtin(context, invocation, false).map(Some);
    }
    if entry == lyng_js_types::js3_add_async_disposable_resource_builtin() {
        return super::add_disposal_scope_resource_builtin(context, invocation, true).map(Some);
    }
    if entry == lyng_js_types::js3_dispose_scope_builtin() {
        return super::dispose_scope_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_dispose_scope_async_builtin() {
        return super::dispose_scope_async_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_global_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_eval_builtin() {
        return super::eval_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_parse_int_builtin() {
        return super::parse_int_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_parse_float_builtin() {
        return super::parse_float_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_is_nan_builtin() {
        return super::is_nan_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_is_finite_builtin() {
        return super::is_finite_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_encode_uri_builtin() {
        return super::encode_uri_builtin(context, invocation, false).map(Some);
    }
    if entry == super::js3_encode_uri_component_builtin() {
        return super::encode_uri_builtin(context, invocation, true).map(Some);
    }
    if entry == super::js3_decode_uri_builtin() {
        return super::decode_uri_builtin(context, invocation, false).map(Some);
    }
    if entry == super::js3_decode_uri_component_builtin() {
        return super::decode_uri_builtin(context, invocation, true).map(Some);
    }
    Ok(None)
}

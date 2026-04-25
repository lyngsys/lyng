use super::PublicBuiltinDispatchContext;
use crate::BuiltinInvocation;
use lyng_js_types::{BuiltinFunctionId, Value};

pub(super) fn dispatch_json_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_json_parse_builtin() {
        return super::json_parse_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_json_stringify_builtin() {
        return super::json_stringify_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_json_raw_json_builtin() {
        return super::json_raw_json_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_json_is_raw_json_builtin() {
        return super::json_is_raw_json_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

use super::PublicBuiltinDispatchContext;
use crate::BuiltinInvocation;
use lyng_js_types::{BuiltinFunctionId, Value};

pub(super) fn dispatch_regexp_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_regexp_constructor_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_regexp_prototype_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_regexp_symbol_builtin(context, entry, invocation)
}

fn dispatch_regexp_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_regexp_builtin() {
        return super::regexp_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_regexp_escape_builtin() {
        return super::regexp_escape_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_regexp_species_getter_builtin() {
        return super::regexp_species_getter_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_regexp_prototype_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_regexp_to_string_builtin() {
        return super::regexp_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_regexp_exec_builtin() {
        return super::regexp_exec_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_regexp_test_builtin() {
        return super::regexp_test_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_regexp_global_getter_builtin() {
        return super::regexp_flag_getter_builtin(context, invocation, 'g').map(Some);
    }
    if entry == super::js3_regexp_ignore_case_getter_builtin() {
        return super::regexp_flag_getter_builtin(context, invocation, 'i').map(Some);
    }
    if entry == super::js3_regexp_multiline_getter_builtin() {
        return super::regexp_flag_getter_builtin(context, invocation, 'm').map(Some);
    }
    if entry == super::js3_regexp_dot_all_getter_builtin() {
        return super::regexp_flag_getter_builtin(context, invocation, 's').map(Some);
    }
    if entry == super::js3_regexp_unicode_getter_builtin() {
        return super::regexp_flag_getter_builtin(context, invocation, 'u').map(Some);
    }
    if entry == super::js3_regexp_sticky_getter_builtin() {
        return super::regexp_flag_getter_builtin(context, invocation, 'y').map(Some);
    }
    if entry == super::js3_regexp_source_getter_builtin() {
        return super::regexp_source_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_regexp_flags_getter_builtin() {
        return super::regexp_flags_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_regexp_has_indices_getter_builtin() {
        return super::regexp_has_indices_getter_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_regexp_symbol_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_regexp_symbol_match_builtin() {
        return super::regexp_symbol_match_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_regexp_symbol_replace_builtin() {
        return super::regexp_symbol_replace_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_regexp_symbol_search_builtin() {
        return super::regexp_symbol_search_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_regexp_symbol_split_builtin() {
        return super::regexp_symbol_split_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_regexp_symbol_match_all_builtin() {
        return super::regexp_symbol_match_all_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

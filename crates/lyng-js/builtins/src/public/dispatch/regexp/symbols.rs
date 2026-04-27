use super::super::{
    allocate_array_like_result, code_unit_range_value, define_data_property_with_attrs,
    string_ref_code_units, to_string_string_ref, to_uint32_for_builtin, type_error,
    PublicBuiltinDispatchContext,
};
use super::{
    advance_string_index, regexp_match_all_with_string, regexp_match_with_string,
    regexp_matcher_this_object, regexp_object_flags, regexp_replace_with_string,
    regexp_search_with_string,
};
use crate::BuiltinInvocation;
use lyng_js_types::{BuiltinFunctionId, PropertyKey, Value};

pub(super) fn dispatch_regexp_symbol_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::super::regexp_symbol_match_builtin() {
        return regexp_symbol_match_builtin(context, invocation).map(Some);
    }
    if entry == super::super::regexp_symbol_replace_builtin() {
        return regexp_symbol_replace_builtin(context, invocation).map(Some);
    }
    if entry == super::super::regexp_symbol_search_builtin() {
        return regexp_symbol_search_builtin(context, invocation).map(Some);
    }
    if entry == super::super::regexp_symbol_split_builtin() {
        return regexp_symbol_split_builtin(context, invocation).map(Some);
    }
    if entry == super::super::regexp_symbol_match_all_builtin() {
        return regexp_symbol_match_all_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn regexp_symbol_match_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = regexp_matcher_this_object(cx, invocation.this_value())?;
    let input_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    regexp_match_with_string(cx, object_ref, input_ref)
}

fn regexp_symbol_replace_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = regexp_matcher_this_object(cx, invocation.this_value())?;
    let input_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let replacement = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    regexp_replace_with_string(cx, object_ref, input_ref, replacement)
}

fn regexp_symbol_search_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = regexp_matcher_this_object(cx, invocation.this_value())?;
    let input_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    regexp_search_with_string(cx, object_ref, input_ref)
}

fn regexp_symbol_split_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = regexp_matcher_this_object(cx, invocation.this_value())?;
    let input_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let limit = if let Some(value) = invocation.arguments().get(1).copied() {
        if value.is_undefined() {
            u32::MAX
        } else {
            to_uint32_for_builtin(cx, value)?
        }
    } else {
        u32::MAX
    };
    if limit == 0 {
        return Ok(Value::from_object_ref(allocate_array_like_result(cx, 0)?));
    }

    let source_units = string_ref_code_units(cx, input_ref)?;
    let flags = regexp_object_flags(cx, object_ref)?;
    let payload = {
        let agent = cx.agent();
        agent.objects().regexp_payload(object_ref).cloned()
    }
    .ok_or_else(|| type_error(cx))?;

    if payload.source().is_empty() {
        let part_count = source_units
            .len()
            .min(usize::try_from(limit).unwrap_or(usize::MAX));
        let object = allocate_array_like_result(cx, u32::try_from(part_count).unwrap_or(u32::MAX))?;
        for index in 0..part_count {
            let value = code_unit_range_value(cx, &source_units, index..index + 1);
            define_data_property_with_attrs(
                cx,
                object,
                PropertyKey::Index(u32::try_from(index).unwrap_or(u32::MAX)),
                value,
                true,
                true,
                true,
            )?;
        }
        return Ok(Value::from_object_ref(object));
    }

    let mut parts = Vec::new();
    let mut last_end = 0;
    let mut search_start = 0;
    let mut suppress_trailing_empty = false;
    let limit_len = usize::try_from(limit).unwrap_or(usize::MAX);
    while search_start <= source_units.len() && parts.len() < limit_len {
        let Some(matched) = payload.find_from_code_units(&source_units, search_start) else {
            break;
        };
        if matched.start() < last_end {
            search_start = advance_string_index(&source_units, search_start, flags.unicode_aware());
            continue;
        }
        if matched.start() == matched.end() && matched.start() == search_start {
            search_start = advance_string_index(&source_units, search_start, flags.unicode_aware());
            continue;
        }

        parts.push(Some(last_end..matched.start()));
        if parts.len() >= limit_len {
            break;
        }
        for capture in matched.captures() {
            parts.push(capture.clone());
            if parts.len() >= limit_len {
                break;
            }
        }
        last_end = matched.end();
        search_start = matched.end();
        suppress_trailing_empty =
            matched.start() == matched.end() && matched.end() == source_units.len();
    }
    if parts.len() < limit_len && !suppress_trailing_empty {
        parts.push(Some(last_end..source_units.len()));
    }

    let object = allocate_array_like_result(cx, u32::try_from(parts.len()).unwrap_or(u32::MAX))?;
    for (index, part) in parts.into_iter().enumerate() {
        let value = part.map_or(Value::undefined(), |range| {
            code_unit_range_value(cx, &source_units, range)
        });
        define_data_property_with_attrs(
            cx,
            object,
            PropertyKey::Index(u32::try_from(index).unwrap_or(u32::MAX)),
            value,
            true,
            true,
            true,
        )?;
    }
    Ok(Value::from_object_ref(object))
}

fn regexp_symbol_match_all_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = regexp_matcher_this_object(cx, invocation.this_value())?;
    let input_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    regexp_match_all_with_string(cx, object_ref, input_ref)
}

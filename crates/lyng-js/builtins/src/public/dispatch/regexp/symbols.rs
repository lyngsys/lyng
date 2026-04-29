use super::super::{
    code_unit_range_value, create_array_from_values, map_completion, string_ref_code_units,
    string_value, to_length_for_builtin, to_string_string_ref, to_uint32_for_builtin, type_error,
    usize_index_value, PublicBuiltinDispatchContext,
};
use super::{
    advance_string_index, regexp_exec, regexp_match_all_with_string, regexp_replace_with_string,
    regexp_result_capture_count, regexp_species_constructor, set_property_on_object_or_throw,
};
use crate::BuiltinInvocation;
use lyng_js_ops::read;
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
    let receiver = invocation.this_value();
    let object_ref = receiver.as_object_ref().ok_or_else(|| type_error(cx))?;
    let input_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let input_units = string_ref_code_units(cx, input_ref)?;
    let flags_key = {
        let agent = cx.agent();
        PropertyKey::from_atom(agent.bootstrap_atoms().flags())
    };
    let flags_value = cx.get_property_value(receiver, flags_key)?;
    let flags_text = cx.value_to_string_text(flags_value)?;
    if !flags_text.contains('g') {
        return regexp_exec(cx, object_ref, input_ref, &input_units);
    }

    let full_unicode = flags_text.contains('u') || flags_text.contains('v');
    let last_index_key = {
        let agent = cx.agent();
        PropertyKey::from_atom(agent.bootstrap_atoms().last_index())
    };
    set_property_on_object_or_throw(cx, object_ref, last_index_key, Value::from_smi(0))?;

    let mut matches = Vec::new();
    loop {
        let result = regexp_exec(cx, object_ref, input_ref, &input_units)?;
        if result.is_null() {
            break;
        }
        let matched = cx.get_property_value(result, PropertyKey::Index(0))?;
        let matched_ref = to_string_string_ref(cx, matched)?;
        let matched_units = string_ref_code_units(cx, matched_ref)?;
        matches.push(Value::from_string_ref(matched_ref));
        if matched_units.is_empty() {
            let this_index = cx.get_property_value(receiver, last_index_key)?;
            let this_index = to_length_for_builtin(cx, this_index)?;
            let next_index = advance_string_index(&input_units, this_index, full_unicode);
            set_property_on_object_or_throw(
                cx,
                object_ref,
                last_index_key,
                usize_index_value(next_index),
            )?;
        }
    }
    if matches.is_empty() {
        return Ok(Value::null());
    }
    let array = create_array_from_values(cx, &matches)?;
    Ok(Value::from_object_ref(array))
}

fn regexp_symbol_replace_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let receiver = invocation.this_value();
    receiver.as_object_ref().ok_or_else(|| type_error(cx))?;
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
    regexp_replace_with_string(cx, receiver, input_ref, replacement)
}

fn regexp_symbol_search_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let receiver = invocation.this_value();
    let object_ref = receiver.as_object_ref().ok_or_else(|| type_error(cx))?;
    let input_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let last_index_key = {
        let agent = cx.agent();
        PropertyKey::from_atom(agent.bootstrap_atoms().last_index())
    };
    let previous_last_index = cx.get_property_value(receiver, last_index_key)?;
    if !same_value_for_builtin(cx, previous_last_index, Value::from_smi(0))? {
        set_property_on_object_or_throw(cx, object_ref, last_index_key, Value::from_smi(0))?;
    }

    let input_units = string_ref_code_units(cx, input_ref)?;
    let result = regexp_exec(cx, object_ref, input_ref, &input_units)?;
    let current_last_index = cx.get_property_value(receiver, last_index_key)?;
    if !same_value_for_builtin(cx, current_last_index, previous_last_index)? {
        set_property_on_object_or_throw(cx, object_ref, last_index_key, previous_last_index)?;
    }
    if result.is_null() {
        return Ok(Value::from_smi(-1));
    }
    let index_key = {
        let agent = cx.agent();
        PropertyKey::from_atom(agent.atoms_mut().intern_collectible("index"))
    };
    cx.get_property_value(result, index_key)
}

fn same_value_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    left: Value,
    right: Value,
) -> Result<bool, Cx::Error> {
    let same = {
        let agent = cx.agent();
        read::same_value(agent.heap().view(), left, right)
    };
    map_completion(cx, same)
}

fn regexp_symbol_split_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let receiver = invocation.this_value();
    let object_ref = receiver.as_object_ref().ok_or_else(|| type_error(cx))?;
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
        let empty = create_array_from_values(cx, &[])?;
        return Ok(Value::from_object_ref(empty));
    }

    let source_units = string_ref_code_units(cx, input_ref)?;
    let flags_key = {
        let agent = cx.agent();
        PropertyKey::from_atom(agent.bootstrap_atoms().flags())
    };
    let flags_value = cx.get_property_value(receiver, flags_key)?;
    let flags_text = cx.value_to_string_text(flags_value)?;
    let unicode_matching = flags_text.contains('u') || flags_text.contains('v');
    let new_flags = if flags_text.contains('y') {
        flags_text
    } else {
        format!("{flags_text}y")
    };
    let constructor = regexp_species_constructor(cx, object_ref)?;
    let flags_arg = string_value(cx, &new_flags);
    let splitter =
        cx.construct_to_completion(constructor, &[receiver, flags_arg], Some(constructor))?;
    let last_index_key = {
        let agent = cx.agent();
        PropertyKey::from_atom(agent.bootstrap_atoms().last_index())
    };

    if source_units.is_empty() {
        let matched = regexp_exec(cx, splitter, input_ref, &source_units)?;
        if !matched.is_null() {
            let empty = create_array_from_values(cx, &[])?;
            return Ok(Value::from_object_ref(empty));
        }
        let parts = [Value::from_string_ref(input_ref)];
        let array = create_array_from_values(cx, &parts)?;
        return Ok(Value::from_object_ref(array));
    }

    let limit_len = usize::try_from(limit).unwrap_or(usize::MAX);
    let mut parts = Vec::new();
    let mut p = 0;
    let mut q = 0;
    while q < source_units.len() {
        set_property_on_object_or_throw(cx, splitter, last_index_key, usize_index_value(q))?;
        let matched = regexp_exec(cx, splitter, input_ref, &source_units)?;
        if matched.is_null() {
            q = advance_string_index(&source_units, q, unicode_matching);
            continue;
        }

        let e = cx.get_property_value(Value::from_object_ref(splitter), last_index_key)?;
        let e = to_length_for_builtin(cx, e)?.min(source_units.len());
        if e == p {
            q = advance_string_index(&source_units, q, unicode_matching);
            continue;
        }

        parts.push(code_unit_range_value(cx, &source_units, p..q));
        if parts.len() == limit_len {
            let array = create_array_from_values(cx, &parts)?;
            return Ok(Value::from_object_ref(array));
        }
        p = e;

        let capture_count = regexp_result_capture_count(cx, matched)?;
        for index in 1..=capture_count {
            let capture = cx.get_property_value(
                matched,
                PropertyKey::Index(u32::try_from(index).unwrap_or(u32::MAX)),
            )?;
            parts.push(capture);
            if parts.len() == limit_len {
                let array = create_array_from_values(cx, &parts)?;
                return Ok(Value::from_object_ref(array));
            }
        }
        q = p;
    }
    parts.push(code_unit_range_value(
        cx,
        &source_units,
        p..source_units.len(),
    ));
    let array = create_array_from_values(cx, &parts)?;
    Ok(Value::from_object_ref(array))
}

fn regexp_symbol_match_all_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let receiver = invocation.this_value();
    let input_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    regexp_match_all_with_string(cx, receiver, input_ref)
}

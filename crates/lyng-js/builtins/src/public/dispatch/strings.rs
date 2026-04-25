use super::{
    allocate_array_like_result, array_like_index_property_key, callable_object_from_value,
    define_data_property_with_attrs,
    iterators::{string_iterator_builtin, string_iterator_next_builtin},
    map_completion, primitive_wrapper_constructor, property_key_from_text, range_error, regexp,
    string_from_code_units, string_ref_code_units, string_ref_text, string_this_ref, string_value,
    symbol_descriptive_string, to_integer_or_infinity_for_builtin, to_length_for_builtin,
    to_number_for_builtin, to_string_string_ref, to_uint32_for_builtin, type_error,
    usize_index_value, BuiltinToPrimitiveBridge, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_common::WellKnownAtom;
use lyng_js_objects::PrimitiveWrapperKind;
use lyng_js_ops::object;
use lyng_js_types::{BuiltinFunctionId, PropertyKey, Value, WellKnownSymbolId};

pub(super) fn dispatch_string_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_string_constructor_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_string_iterator_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_string_basic_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_string_search_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_string_transform_builtin(context, entry, invocation)
}

fn dispatch_string_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::string_builtin() {
        return string_builtin(context, invocation).map(Some);
    }
    if entry == super::string_from_char_code_builtin() {
        return string_from_char_code_builtin(context, invocation).map(Some);
    }
    if entry == super::string_from_code_point_builtin() {
        return string_from_code_point_builtin(context, invocation).map(Some);
    }
    if entry == super::string_raw_builtin() {
        return string_raw_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_string_iterator_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::string_iterator_builtin() {
        return string_iterator_builtin(context, invocation).map(Some);
    }
    if entry == super::string_iterator_next_builtin() {
        return string_iterator_next_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_string_basic_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::string_to_string_builtin() {
        return string_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::string_value_of_builtin() {
        return string_value_of_builtin(context, invocation).map(Some);
    }
    if entry == super::string_concat_builtin() {
        return string_concat_builtin(context, invocation).map(Some);
    }
    if entry == super::string_char_at_builtin() {
        return string_char_at_builtin(context, invocation).map(Some);
    }
    if entry == super::string_char_code_at_builtin() {
        return string_char_code_at_builtin(context, invocation).map(Some);
    }
    if entry == super::string_at_builtin() {
        return string_at_builtin(context, invocation).map(Some);
    }
    if entry == super::string_code_point_at_builtin() {
        return string_code_point_at_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_string_search_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::string_ends_with_builtin() {
        return string_ends_with_builtin(context, invocation).map(Some);
    }
    if entry == super::string_includes_builtin() {
        return string_includes_builtin(context, invocation).map(Some);
    }
    if entry == super::string_index_of_builtin() {
        return string_index_of_builtin(context, invocation).map(Some);
    }
    if entry == super::string_locale_compare_builtin() {
        return string_locale_compare_builtin(context, invocation).map(Some);
    }
    if entry == super::string_match_builtin() {
        return string_match_builtin(context, invocation).map(Some);
    }
    if entry == super::string_match_all_builtin() {
        return string_match_all_builtin(context, invocation).map(Some);
    }
    if entry == super::string_last_index_of_builtin() {
        return string_last_index_of_builtin(context, invocation).map(Some);
    }
    if entry == super::string_replace_builtin() {
        return string_replace_builtin(context, invocation).map(Some);
    }
    if entry == super::string_replace_all_builtin() {
        return string_replace_all_builtin(context, invocation).map(Some);
    }
    if entry == super::string_search_builtin() {
        return string_search_builtin(context, invocation).map(Some);
    }
    if entry == super::string_split_builtin() {
        return string_split_builtin(context, invocation).map(Some);
    }
    if entry == super::string_starts_with_builtin() {
        return string_starts_with_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_string_transform_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::string_is_well_formed_builtin() {
        return string_is_well_formed_builtin(context, invocation).map(Some);
    }
    if entry == super::string_normalize_builtin() {
        return string_normalize_builtin(context, invocation).map(Some);
    }
    if entry == super::string_pad_end_builtin() {
        return string_pad_end_builtin(context, invocation).map(Some);
    }
    if entry == super::string_pad_start_builtin() {
        return string_pad_start_builtin(context, invocation).map(Some);
    }
    if entry == super::string_repeat_builtin() {
        return string_repeat_builtin(context, invocation).map(Some);
    }
    if entry == super::string_slice_builtin() {
        return string_slice_builtin(context, invocation).map(Some);
    }
    if entry == super::string_substring_builtin() {
        return string_substring_builtin(context, invocation).map(Some);
    }
    if entry == super::string_to_locale_lower_case_builtin() {
        return string_case_mapping_builtin(context, invocation, StringCaseMapping::Lower)
            .map(Some);
    }
    if entry == super::string_to_locale_upper_case_builtin() {
        return string_case_mapping_builtin(context, invocation, StringCaseMapping::Upper)
            .map(Some);
    }
    if entry == super::string_to_lower_case_builtin() {
        return string_case_mapping_builtin(context, invocation, StringCaseMapping::Lower)
            .map(Some);
    }
    if entry == super::string_to_upper_case_builtin() {
        return string_case_mapping_builtin(context, invocation, StringCaseMapping::Upper)
            .map(Some);
    }
    if entry == super::string_to_well_formed_builtin() {
        return string_to_well_formed_builtin(context, invocation).map(Some);
    }
    if entry == super::string_trim_builtin() {
        return string_trim_builtin(context, invocation, true, true).map(Some);
    }
    if entry == super::string_trim_end_builtin() {
        return string_trim_builtin(context, invocation, false, true).map(Some);
    }
    if entry == super::string_trim_start_builtin() {
        return string_trim_builtin(context, invocation, true, false).map(Some);
    }
    Ok(None)
}

fn string_value_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Value, Cx::Error> {
    let primitive = if value.is_object() {
        let mut bridge = BuiltinToPrimitiveBridge { cx };
        object::to_primitive(&mut bridge, value, object::ToPrimitiveHint::String)?
    } else {
        value
    };

    if let Some(string) = primitive.as_string_ref() {
        return Ok(Value::from_string_ref(string));
    }
    if let Some(symbol) = primitive.as_symbol_ref() {
        let text = symbol_descriptive_string(cx, symbol)?;
        return Ok(string_value(cx, &text));
    }
    if primitive.is_bigint() {
        let text = {
            let agent = cx.agent();
            object::bigint_to_string(agent, primitive, 10)
        };
        let text = map_completion(cx, text)?;
        return Ok(string_value(cx, &text));
    }

    let text = cx.value_to_string_text(primitive)?;
    Ok(string_value(cx, &text))
}

fn string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.new_target().is_none() {
        let value = if invocation.arguments().is_empty() {
            string_value(cx, "")
        } else {
            string_value_from_value(cx, invocation.arguments()[0])?
        };
        return Ok(value);
    }
    let value = if invocation.arguments().is_empty() {
        string_value(cx, "")
    } else {
        Value::from_string_ref(to_string_string_ref(cx, invocation.arguments()[0])?)
    };
    let realm = cx.builtin_realm();
    let default_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().string_prototype())
        .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?;
    primitive_wrapper_constructor(cx, realm, prototype, PrimitiveWrapperKind::String, value)
}

fn string_wrapper_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Value, Cx::Error> {
    if value.as_string_ref().is_some() {
        return Ok(value);
    }
    let Some(object_ref) = value.as_object_ref() else {
        return Err(type_error(cx));
    };
    let payload = {
        let agent = cx.agent();
        if agent.objects().primitive_wrapper_kind(object_ref) == Some(PrimitiveWrapperKind::String)
        {
            agent
                .objects()
                .primitive_wrapper_value(agent.heap().view(), object_ref)
        } else {
            None
        }
    };
    payload.ok_or_else(|| type_error(cx))
}

fn string_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    string_wrapper_value(cx, invocation.this_value())
}

fn string_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    string_wrapper_value(cx, invocation.this_value())
}

fn string_concat_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let this_string = string_this_ref(cx, invocation.this_value())?;
    let mut text = string_ref_text(cx, this_string)?;
    for argument in invocation.arguments() {
        let argument_string = to_string_string_ref(cx, *argument)?;
        text.push_str(&string_ref_text(cx, argument_string)?);
    }
    Ok(string_value(cx, &text))
}

fn string_position_index(position: f64, length: usize) -> Option<usize> {
    if position == 0.0 {
        return (length > 0).then_some(0);
    }
    if !position.is_finite() || position < 0.0 {
        return None;
    }
    let index = position as usize;
    (index < length).then_some(index)
}

fn find_subsequence(haystack: &[u16], needle: &[u16], start: usize) -> Option<usize> {
    if needle.is_empty() {
        return Some(start.min(haystack.len()));
    }
    if needle.len() > haystack.len() || start > haystack.len().saturating_sub(needle.len()) {
        return None;
    }
    haystack[start..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|offset| offset + start)
}

fn string_char_at_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let position = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let Some(index) = string_position_index(position, units.len()) else {
        return Ok(string_from_code_units(cx, &[]));
    };
    Ok(string_from_code_units(cx, &units[index..index + 1]))
}

fn string_char_code_at_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let position = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let Some(index) = string_position_index(position, units.len()) else {
        return Ok(Value::from_f64(f64::NAN));
    };
    Ok(Value::from_smi(i32::from(units[index])))
}

fn string_from_char_code_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let mut units = Vec::with_capacity(invocation.arguments().len());
    for value in invocation.arguments().iter().copied() {
        units.push((to_uint32_for_builtin(cx, value)? & 0xffff) as u16);
    }
    Ok(string_from_code_units(cx, &units))
}

fn append_code_point_units(units: &mut Vec<u16>, code_point: u32) {
    if code_point <= 0xFFFF {
        units.push(code_point as u16);
        return;
    }

    let adjusted = code_point - 0x1_0000;
    units.push(0xD800 | ((adjusted >> 10) as u16));
    units.push(0xDC00 | ((adjusted as u16) & 0x03FF));
}

fn string_from_code_point_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let mut units = Vec::with_capacity(invocation.arguments().len());
    for value in invocation.arguments().iter().copied() {
        let number = to_number_for_builtin(cx, value)?;
        if !number.is_finite() || number.trunc() != number || !(0.0..=1_114_111.0).contains(&number)
        {
            return Err(range_error(cx));
        }
        append_code_point_units(&mut units, number as u32);
    }
    Ok(string_from_code_units(cx, &units))
}

fn string_raw_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let template_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let template = cx.to_object_for_builtin_value(cx.builtin_realm(), template_value)?;
    let raw_key = property_key_from_text(cx, "raw");
    let raw_value = cx.get_property_value(Value::from_object_ref(template), raw_key)?;
    let raw = cx.to_object_for_builtin_value(cx.builtin_realm(), raw_value)?;
    let length_value = cx.get_property_value(
        Value::from_object_ref(raw),
        PropertyKey::from_atom(WellKnownAtom::length.id()),
    )?;
    let literal_segments = to_length_for_builtin(cx, length_value)?;
    if literal_segments == 0 {
        return Ok(string_from_code_units(cx, &[]));
    }

    let mut result = Vec::new();
    for index in 0..literal_segments {
        let key = array_like_index_property_key(
            cx,
            u64::try_from(index).expect("raw template index must fit into u64"),
        );
        let segment = cx.get_property_value(Value::from_object_ref(raw), key)?;
        let segment = to_string_string_ref(cx, segment)?;
        result.extend_from_slice(&string_ref_code_units(cx, segment)?);

        if index + 1 == literal_segments {
            break;
        }
        if let Some(substitution) = invocation.arguments().get(index + 1).copied() {
            let substitution = to_string_string_ref(cx, substitution)?;
            result.extend_from_slice(&string_ref_code_units(cx, substitution)?);
        }
    }

    Ok(string_from_code_units(cx, &result))
}

fn string_at_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let relative_index = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let index = if relative_index < 0.0 {
        units.len() as f64 + relative_index
    } else {
        relative_index
    };
    if !index.is_finite() || index < 0.0 || index >= units.len() as f64 {
        return Ok(Value::undefined());
    }
    let index = index as usize;
    Ok(string_from_code_units(cx, &units[index..index + 1]))
}

fn string_code_point_at_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let position = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let Some(index) = string_position_index(position, units.len()) else {
        return Ok(Value::undefined());
    };
    let first = units[index];
    let code_point = if (0xD800..=0xDBFF).contains(&first) {
        if let Some(second) = units.get(index + 1).copied() {
            if (0xDC00..=0xDFFF).contains(&second) {
                0x1_0000 + ((u32::from(first - 0xD800)) << 10) + u32::from(second - 0xDC00)
            } else {
                u32::from(first)
            }
        } else {
            u32::from(first)
        }
    } else {
        u32::from(first)
    };
    Ok(Value::from_smi(
        i32::try_from(code_point).expect("Unicode code points fit into i32"),
    ))
}

fn string_index_of_units(source: &[u16], search: &[u16], position: usize) -> i32 {
    find_subsequence(source, search, position)
        .and_then(|index| i32::try_from(index).ok())
        .unwrap_or(-1)
}

fn string_index_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let search_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let search_units = string_ref_code_units(cx, search_ref)?;
    let position = if let Some(value) = invocation.arguments().get(1).copied() {
        to_integer_or_infinity_for_builtin(cx, value)?
    } else {
        0.0
    };
    let start = if position.is_nan() || position <= 0.0 {
        0
    } else if !position.is_finite() {
        source_units.len()
    } else {
        (position as usize).min(source_units.len())
    };
    Ok(Value::from_smi(string_index_of_units(
        &source_units,
        &search_units,
        start,
    )))
}

fn string_includes_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let search_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if regexp::regexp_search_value_is_rejected(cx, search_value)? {
        return Err(type_error(cx));
    }
    let search_ref = to_string_string_ref(cx, search_value)?;
    let search_units = string_ref_code_units(cx, search_ref)?;
    let position = if let Some(value) = invocation.arguments().get(1).copied() {
        to_integer_or_infinity_for_builtin(cx, value)?
    } else {
        0.0
    };
    let start = if position.is_nan() || position <= 0.0 {
        0
    } else if !position.is_finite() {
        source_units.len()
    } else {
        (position as usize).min(source_units.len())
    };
    Ok(Value::from_bool(
        find_subsequence(&source_units, &search_units, start).is_some(),
    ))
}

fn string_ends_with_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let search_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if regexp::regexp_search_value_is_rejected(cx, search_value)? {
        return Err(type_error(cx));
    }
    let search_ref = to_string_string_ref(cx, search_value)?;
    let search_units = string_ref_code_units(cx, search_ref)?;
    let end_position = if let Some(value) = invocation.arguments().get(1).copied() {
        to_integer_or_infinity_for_builtin(cx, value)?
    } else {
        source_units.len() as f64
    };
    let end = if end_position.is_nan() || end_position <= 0.0 {
        0
    } else if !end_position.is_finite() {
        source_units.len()
    } else {
        (end_position as usize).min(source_units.len())
    };
    let Some(start) = end.checked_sub(search_units.len()) else {
        return Ok(Value::from_bool(false));
    };
    Ok(Value::from_bool(
        source_units[start..end] == search_units[..],
    ))
}

fn string_locale_compare_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_text = string_ref_text(cx, source_ref)?;
    let that_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let that_text = string_ref_text(cx, that_ref)?;
    let source_key = normalize_text_for_form(&source_text, "NFD").ok_or_else(|| range_error(cx))?;
    let that_key = normalize_text_for_form(&that_text, "NFD").ok_or_else(|| range_error(cx))?;
    let result = match source_key.cmp(&that_key) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    };
    Ok(Value::from_smi(result))
}

fn is_well_formed_utf16(units: &[u16]) -> bool {
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit) {
            if !matches!(units.get(index + 1), Some(next) if (0xDC00..=0xDFFF).contains(next)) {
                return false;
            }
            index += 2;
            continue;
        }
        if (0xDC00..=0xDFFF).contains(&unit) {
            return false;
        }
        index += 1;
    }
    true
}

fn string_is_well_formed_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    Ok(Value::from_bool(is_well_formed_utf16(&units)))
}

fn to_well_formed_utf16(units: &[u16]) -> Vec<u16> {
    let mut result = Vec::with_capacity(units.len());
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit) {
            if matches!(units.get(index + 1), Some(next) if (0xDC00..=0xDFFF).contains(next)) {
                result.push(unit);
                result.push(units[index + 1]);
                index += 2;
                continue;
            }
            result.push(0xFFFD);
            index += 1;
            continue;
        }
        if (0xDC00..=0xDFFF).contains(&unit) {
            result.push(0xFFFD);
        } else {
            result.push(unit);
        }
        index += 1;
    }
    result
}

fn string_to_well_formed_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    Ok(string_from_code_units(cx, &to_well_formed_utf16(&units)))
}

fn is_ecmascript_trim_unit(unit: u16) -> bool {
    matches!(
        unit,
        0x0009 | 0x000A | 0x000B | 0x000C | 0x000D | 0x0020 | 0x00A0 | 0x1680 | 0x2000
            ..=0x200A | 0x2028 | 0x2029 | 0x202F | 0x205F | 0x3000 | 0xFEFF
    )
}

fn string_trim_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    trim_start: bool,
    trim_end: bool,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let mut start = 0;
    let mut end = units.len();
    if trim_start {
        while start < end && is_ecmascript_trim_unit(units[start]) {
            start += 1;
        }
    }
    if trim_end {
        while end > start && is_ecmascript_trim_unit(units[end - 1]) {
            end -= 1;
        }
    }
    Ok(string_from_code_units(cx, &units[start..end]))
}

enum StringCaseMapping {
    Lower,
    Upper,
}

fn push_char_units(output: &mut Vec<u16>, ch: char) {
    let mut buffer = [0_u16; 2];
    output.extend_from_slice(ch.encode_utf16(&mut buffer));
}

pub(super) fn push_code_point_units(output: &mut Vec<u16>, code_point: u32) {
    if let Some(ch) = char::from_u32(code_point) {
        push_char_units(output, ch);
    } else if let Ok(unit) = u16::try_from(code_point) {
        output.push(unit);
    }
}

fn utf16_code_points(units: &[u16]) -> Vec<u32> {
    let mut points = Vec::with_capacity(units.len());
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit)
            && matches!(units.get(index + 1), Some(next) if (0xDC00..=0xDFFF).contains(next))
        {
            let trailing = units[index + 1];
            points
                .push(0x1_0000 + ((u32::from(unit - 0xD800)) << 10) + u32::from(trailing - 0xDC00));
            index += 2;
        } else {
            points.push(u32::from(unit));
            index += 1;
        }
    }
    points
}

fn is_case_ignorable_code_point(code_point: u32) -> bool {
    matches!(
        code_point,
        0x00AD | 0x0345 | 0x180E | 0x0300..=0x036F | 0x1D242
    )
}

fn is_cased_code_point(code_point: u32) -> bool {
    if is_case_ignorable_code_point(code_point) {
        return false;
    }
    if matches!(
        code_point,
        0x0041..=0x005A
            | 0x0061..=0x007A
            | 0x00C0..=0x024F
            | 0x0391..=0x03A9
            | 0x03B1..=0x03C9
            | 0x1D4A2
    ) {
        return true;
    }
    let Some(ch) = char::from_u32(code_point) else {
        return false;
    };
    let lower: String = ch.to_lowercase().collect();
    let upper: String = ch.to_uppercase().collect();
    lower != ch.to_string() || upper != ch.to_string()
}

fn is_final_sigma_context(points: &[u32], index: usize) -> bool {
    points[..index].iter().copied().any(is_cased_code_point)
        && !points[index + 1..].iter().copied().any(is_cased_code_point)
}

fn string_case_mapping_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    mapping: StringCaseMapping,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let mut mapped = Vec::with_capacity(units.len());
    let points = utf16_code_points(&units);
    for (index, code_point) in points.iter().copied().enumerate() {
        if matches!(mapping, StringCaseMapping::Lower) && code_point == 0x03A3 {
            push_code_point_units(
                &mut mapped,
                if is_final_sigma_context(&points, index) {
                    0x03C2
                } else {
                    0x03C3
                },
            );
            continue;
        }
        let Some(ch) = char::from_u32(code_point) else {
            push_code_point_units(&mut mapped, code_point);
            continue;
        };
        match mapping {
            StringCaseMapping::Lower => {
                for ch in ch.to_lowercase() {
                    push_char_units(&mut mapped, ch);
                }
            }
            StringCaseMapping::Upper => {
                for ch in ch.to_uppercase() {
                    push_char_units(&mut mapped, ch);
                }
            }
        }
    }
    Ok(string_from_code_units(cx, &mapped))
}

fn canonical_combining_class(code_point: u32) -> u8 {
    match code_point {
        0x093C => 7,
        0x031B => 216,
        0x0323 => 220,
        0x0327 => 202,
        0x0301 | 0x0302 | 0x0306 | 0x0307 | 0x0308 | 0x030A => 230,
        _ => 0,
    }
}

fn decompose_hangul(code_point: u32, output: &mut Vec<u32>) -> bool {
    const S_BASE: u32 = 0xAC00;
    const L_BASE: u32 = 0x1100;
    const V_BASE: u32 = 0x1161;
    const T_BASE: u32 = 0x11A7;
    const L_COUNT: u32 = 19;
    const V_COUNT: u32 = 21;
    const T_COUNT: u32 = 28;
    const N_COUNT: u32 = V_COUNT * T_COUNT;
    const S_COUNT: u32 = L_COUNT * N_COUNT;

    if !(S_BASE..S_BASE + S_COUNT).contains(&code_point) {
        return false;
    }
    let s_index = code_point - S_BASE;
    output.push(L_BASE + s_index / N_COUNT);
    output.push(V_BASE + (s_index % N_COUNT) / T_COUNT);
    let trailing = s_index % T_COUNT;
    if trailing != 0 {
        output.push(T_BASE + trailing);
    }
    true
}

fn decompose_code_point(code_point: u32, compatibility: bool, output: &mut Vec<u32>) {
    if decompose_hangul(code_point, output) {
        return;
    }
    let decomposition: Option<&'static [u32]> = match code_point {
        0x00C5 => Some(&[0x0041, 0x030A]),
        0x00C7 => Some(&[0x0043, 0x0327]),
        0x00C9 => Some(&[0x0045, 0x0301]),
        0x00E4 => Some(&[0x0061, 0x0308]),
        0x00E9 => Some(&[0x0065, 0x0301]),
        0x00F4 => Some(&[0x006F, 0x0302]),
        0x00F6 => Some(&[0x006F, 0x0308]),
        0x0103 => Some(&[0x0061, 0x0306]),
        0x01B0 => Some(&[0x0075, 0x031B]),
        0x0344 => Some(&[0x0308, 0x0301]),
        0x0958 => Some(&[0x0915, 0x093C]),
        0x1E0B => Some(&[0x0064, 0x0307]),
        0x1E0D => Some(&[0x0064, 0x0323]),
        0x1E63 => Some(&[0x0073, 0x0323]),
        0x1E69 => Some(&[0x0073, 0x0323, 0x0307]),
        0x1E9B => Some(&[0x017F, 0x0307]),
        0x1EA1 => Some(&[0x0061, 0x0323]),
        0x1EE5 => Some(&[0x0075, 0x0323]),
        0x1EF1 => Some(&[0x0075, 0x031B, 0x0323]),
        0x2126 => Some(&[0x03A9]),
        0x212B => Some(&[0x00C5]),
        0x2ADC => Some(&[0x2ADD, 0x0338]),
        0x017F if compatibility => Some(&[0x0073]),
        _ => None,
    };
    if let Some(decomposition) = decomposition {
        for point in decomposition {
            decompose_code_point(*point, compatibility, output);
        }
    } else {
        output.push(code_point);
    }
}

fn reorder_combining_marks(points: &mut [u32]) {
    let mut index = 1;
    while index < points.len() {
        let class = canonical_combining_class(points[index]);
        if class == 0 {
            index += 1;
            continue;
        }
        let mut scan = index;
        while scan > 0 {
            let previous = canonical_combining_class(points[scan - 1]);
            if previous == 0 || previous <= class {
                break;
            }
            points.swap(scan - 1, scan);
            scan -= 1;
        }
        index += 1;
    }
}

fn compose_hangul_pair(left: u32, right: u32) -> Option<u32> {
    const S_BASE: u32 = 0xAC00;
    const L_BASE: u32 = 0x1100;
    const V_BASE: u32 = 0x1161;
    const T_BASE: u32 = 0x11A7;
    const L_COUNT: u32 = 19;
    const V_COUNT: u32 = 21;
    const T_COUNT: u32 = 28;
    const N_COUNT: u32 = V_COUNT * T_COUNT;
    const S_COUNT: u32 = L_COUNT * N_COUNT;

    if (L_BASE..L_BASE + L_COUNT).contains(&left) && (V_BASE..V_BASE + V_COUNT).contains(&right) {
        return Some(S_BASE + (left - L_BASE) * N_COUNT + (right - V_BASE) * T_COUNT);
    }
    if (S_BASE..S_BASE + S_COUNT).contains(&left)
        && (left - S_BASE) % T_COUNT == 0
        && (T_BASE + 1..T_BASE + T_COUNT).contains(&right)
    {
        return Some(left + (right - T_BASE));
    }
    None
}

fn compose_pair(left: u32, right: u32) -> Option<u32> {
    if let Some(hangul) = compose_hangul_pair(left, right) {
        return Some(hangul);
    }
    match (left, right) {
        (0x0041, 0x030A) => Some(0x00C5),
        (0x0043, 0x0327) => Some(0x00C7),
        (0x0045, 0x0301) => Some(0x00C9),
        (0x0061, 0x0306) => Some(0x0103),
        (0x0061, 0x0308) => Some(0x00E4),
        (0x0061, 0x0323) => Some(0x1EA1),
        (0x0064, 0x0307) => Some(0x1E0B),
        (0x0064, 0x0323) => Some(0x1E0D),
        (0x0065, 0x0301) => Some(0x00E9),
        (0x006F, 0x0302) => Some(0x00F4),
        (0x006F, 0x0308) => Some(0x00F6),
        (0x0073, 0x0323) => Some(0x1E63),
        (0x0075, 0x031B) => Some(0x01B0),
        (0x0075, 0x0323) => Some(0x1EE5),
        (0x017F, 0x0307) => Some(0x1E9B),
        (0x01B0, 0x0323) => Some(0x1EF1),
        (0x1E63, 0x0307) => Some(0x1E69),
        _ => None,
    }
}

fn compose_normalized_code_points(points: &[u32]) -> Vec<u32> {
    let mut result: Vec<u32> = Vec::with_capacity(points.len());
    let mut starter_index: Option<usize> = None;
    let mut previous_class = 0;
    for point in points {
        let class = canonical_combining_class(*point);
        if let Some(starter) = starter_index {
            if previous_class == 0 || previous_class < class {
                if let Some(composed) = compose_pair(result[starter], *point) {
                    result[starter] = composed;
                    continue;
                }
            }
        }
        if class == 0 {
            starter_index = Some(result.len());
        }
        previous_class = class;
        result.push(*point);
    }
    result
}

fn code_points_to_string(points: &[u32]) -> String {
    let mut text = String::new();
    for point in points {
        if let Some(ch) = char::from_u32(*point) {
            text.push(ch);
        } else {
            text.push('\u{FFFD}');
        }
    }
    text
}

fn normalize_text_for_form(text: &str, form: &str) -> Option<String> {
    let (compatibility, compose) = match form {
        "NFC" => (false, true),
        "NFD" => (false, false),
        "NFKC" => (true, true),
        "NFKD" => (true, false),
        _ => return None,
    };
    let mut points = Vec::with_capacity(text.chars().count());
    for ch in text.chars() {
        decompose_code_point(ch as u32, compatibility, &mut points);
    }
    reorder_combining_marks(&mut points);
    let normalized = if compose {
        compose_normalized_code_points(&points)
    } else {
        points
    };
    Some(code_points_to_string(&normalized))
}

fn string_normalize_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let text = string_ref_text(cx, string)?;
    let form = if let Some(value) = invocation.arguments().first().copied() {
        if value.is_undefined() {
            "NFC".to_owned()
        } else {
            let form_ref = to_string_string_ref(cx, value)?;
            string_ref_text(cx, form_ref)?
        }
    } else {
        "NFC".to_owned()
    };
    let normalized = normalize_text_for_form(&text, &form).ok_or_else(|| range_error(cx))?;
    Ok(string_value(cx, &normalized))
}

fn expand_string_replacement_template(
    template_units: &[u16],
    source_units: &[u16],
    matched: std::ops::Range<usize>,
) -> Vec<u16> {
    let mut result = Vec::with_capacity(template_units.len());
    let mut index = 0;
    while index < template_units.len() {
        if template_units[index] != u16::from(b'$') {
            result.push(template_units[index]);
            index += 1;
            continue;
        }
        let Some(next) = template_units.get(index + 1).copied() else {
            result.push(u16::from(b'$'));
            index += 1;
            continue;
        };
        match regexp::code_unit_ascii(next).map(char::from) {
            Some('$') => {
                result.push(u16::from(b'$'));
                index += 2;
            }
            Some('&') => {
                result.extend_from_slice(&source_units[matched.clone()]);
                index += 2;
            }
            Some('`') => {
                result.extend_from_slice(&source_units[..matched.start]);
                index += 2;
            }
            Some('\'') => {
                result.extend_from_slice(&source_units[matched.end..]);
                index += 2;
            }
            _ => {
                result.push(u16::from(b'$'));
                index += 1;
            }
        }
    }
    result
}

fn string_replace_all_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let search_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let replacement = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());

    if search_value.as_object_ref().is_some() {
        if regexp::is_regexp_value(cx, search_value)? {
            let flags_key = {
                let agent = cx.agent();
                PropertyKey::from_atom(agent.bootstrap_atoms().flags())
            };
            let flags_value = cx.get_property_value(search_value, flags_key)?;
            let flags = cx.value_to_string_text(flags_value)?;
            if !flags.contains('g') {
                return Err(type_error(cx));
            }
        }
        if let Some(replacer) =
            regexp::get_method_for_well_known_symbol(cx, search_value, WellKnownSymbolId::Replace)?
        {
            return cx.call_to_completion(
                replacer,
                search_value,
                &[invocation.this_value(), replacement],
            );
        }
    }

    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let source_value = Value::from_string_ref(source_ref);
    let search_ref = to_string_string_ref(cx, search_value)?;
    let search_units = string_ref_code_units(cx, search_ref)?;
    let callable_replacement = callable_object_from_value(cx, replacement);
    let replacement_template = if callable_replacement.is_none() {
        let replacement_ref = to_string_string_ref(cx, replacement)?;
        Some(string_ref_code_units(cx, replacement_ref)?)
    } else {
        None
    };

    let mut positions = Vec::new();
    if search_units.is_empty() {
        positions.extend(0..=source_units.len());
    } else {
        let mut search_start = 0;
        while let Some(position) = find_subsequence(&source_units, &search_units, search_start) {
            positions.push(position);
            search_start = position + search_units.len();
            if search_start > source_units.len() {
                break;
            }
        }
    }

    if positions.is_empty() {
        return Ok(Value::from_string_ref(source_ref));
    }

    let mut result = Vec::with_capacity(source_units.len());
    let mut cursor = 0;
    for position in positions {
        result.extend_from_slice(&source_units[cursor..position]);
        let end = position + search_units.len();
        let replacement_units = if let Some(callee) = callable_replacement {
            let matched = string_from_code_units(cx, &source_units[position..end]);
            let arguments = [matched, usize_index_value(position), source_value];
            let replaced = cx.call_to_completion(callee, Value::undefined(), &arguments)?;
            let replaced = to_string_string_ref(cx, replaced)?;
            string_ref_code_units(cx, replaced)?
        } else {
            expand_string_replacement_template(
                replacement_template
                    .as_deref()
                    .expect("replacement template should exist for string replacements"),
                &source_units,
                position..end,
            )
        };
        result.extend_from_slice(&replacement_units);
        cursor = end;
    }
    result.extend_from_slice(&source_units[cursor..]);
    Ok(string_from_code_units(cx, &result))
}

fn string_match_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let pattern_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if pattern_value.as_object_ref().is_some() {
        if let Some(matcher) =
            regexp::get_method_for_well_known_symbol(cx, pattern_value, WellKnownSymbolId::Match)?
        {
            let source_ref = string_this_ref(cx, invocation.this_value())?;
            let source_value = Value::from_string_ref(source_ref);
            return cx.call_to_completion(matcher, pattern_value, &[source_value]);
        }
    }

    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let pattern_text = if pattern_value.is_undefined() {
        String::new()
    } else {
        cx.value_to_string_text(pattern_value)?
    };
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().regexp_prototype())
    };
    let default_prototype = default_prototype.ok_or_else(|| type_error(cx))?;
    let regexp_object =
        regexp::allocate_regexp_object(cx, realm, default_prototype, &pattern_text, "")?;
    let matcher = regexp::get_method_for_well_known_symbol(
        cx,
        Value::from_object_ref(regexp_object),
        WellKnownSymbolId::Match,
    )?
    .ok_or_else(|| type_error(cx))?;
    cx.call_to_completion(
        matcher,
        Value::from_object_ref(regexp_object),
        &[Value::from_string_ref(source_ref)],
    )
}

fn string_match_all_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let pattern_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if pattern_value.as_object_ref().is_some() {
        if regexp::is_regexp_value(cx, pattern_value)? {
            let flags_key = {
                let agent = cx.agent();
                PropertyKey::from_atom(agent.bootstrap_atoms().flags())
            };
            let flags_value = cx.get_property_value(pattern_value, flags_key)?;
            let flags = cx.value_to_string_text(flags_value)?;
            if !flags.contains('g') {
                return Err(type_error(cx));
            }
        }
        if let Some(matcher) = regexp::get_method_for_well_known_symbol(
            cx,
            pattern_value,
            WellKnownSymbolId::MatchAll,
        )? {
            let source_ref = string_this_ref(cx, invocation.this_value())?;
            return cx.call_to_completion(
                matcher,
                pattern_value,
                &[Value::from_string_ref(source_ref)],
            );
        }
    }

    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let pattern_text = if let Some(object_ref) = pattern_value.as_object_ref() {
        if regexp::is_regexp_object(cx, object_ref) {
            regexp::regexp_object_source_text(cx, object_ref)?
        } else {
            cx.value_to_string_text(pattern_value)?
        }
    } else if pattern_value.is_undefined() {
        String::new()
    } else {
        cx.value_to_string_text(pattern_value)?
    };
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().regexp_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let regexp_object =
        regexp::allocate_regexp_object(cx, realm, default_prototype, &pattern_text, "g")?;
    let matcher = regexp::get_method_for_well_known_symbol(
        cx,
        Value::from_object_ref(regexp_object),
        WellKnownSymbolId::MatchAll,
    )?
    .ok_or_else(|| type_error(cx))?;
    cx.call_to_completion(
        matcher,
        Value::from_object_ref(regexp_object),
        &[Value::from_string_ref(source_ref)],
    )
}

fn string_search_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let pattern_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if pattern_value.as_object_ref().is_some() {
        if let Some(searcher) =
            regexp::get_method_for_well_known_symbol(cx, pattern_value, WellKnownSymbolId::Search)?
        {
            let source_ref = string_this_ref(cx, invocation.this_value())?;
            return cx.call_to_completion(
                searcher,
                pattern_value,
                &[Value::from_string_ref(source_ref)],
            );
        }
    }

    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let pattern_text = if pattern_value.is_undefined() {
        String::new()
    } else {
        cx.value_to_string_text(pattern_value)?
    };
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().regexp_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let regexp_object =
        regexp::allocate_regexp_object(cx, realm, default_prototype, &pattern_text, "")?;
    let searcher = regexp::get_method_for_well_known_symbol(
        cx,
        Value::from_object_ref(regexp_object),
        WellKnownSymbolId::Search,
    )?
    .ok_or_else(|| type_error(cx))?;
    cx.call_to_completion(
        searcher,
        Value::from_object_ref(regexp_object),
        &[Value::from_string_ref(source_ref)],
    )
}

fn string_last_index_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let search_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let search_units = string_ref_code_units(cx, search_ref)?;

    let position = if let Some(value) = invocation.arguments().get(1).copied() {
        let number = to_number_for_builtin(cx, value)?;
        if number.is_nan() {
            f64::INFINITY
        } else if number == 0.0 {
            0.0
        } else if !number.is_finite() {
            number
        } else {
            number.trunc()
        }
    } else {
        f64::INFINITY
    };

    let source_len = source_units.len();
    let search_len = search_units.len();
    let start = if position.is_nan() || position == f64::INFINITY {
        source_len
    } else if position <= 0.0 {
        0
    } else {
        (position as usize).min(source_len)
    };

    if search_units.is_empty() {
        return Ok(Value::from_smi(i32::try_from(start).unwrap_or(i32::MAX)));
    }
    if search_len > source_len {
        return Ok(Value::from_smi(-1));
    }

    let max_index = start.min(source_len - search_len);
    for index in (0..=max_index).rev() {
        if source_units[index..index + search_len] == search_units[..] {
            return Ok(Value::from_smi(i32::try_from(index).unwrap_or(i32::MAX)));
        }
    }
    Ok(Value::from_smi(-1))
}

fn string_pad_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    at_start: bool,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, string)?;
    let max_length = to_length_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if max_length <= source_units.len() {
        return Ok(Value::from_string_ref(string));
    }

    let fill_units = if let Some(value) = invocation.arguments().get(1).copied() {
        if value.is_undefined() {
            vec![u16::from(b' ')]
        } else {
            let fill = to_string_string_ref(cx, value)?;
            string_ref_code_units(cx, fill)?
        }
    } else {
        vec![u16::from(b' ')]
    };
    if fill_units.is_empty() {
        return Ok(Value::from_string_ref(string));
    }

    let fill_len = max_length - source_units.len();
    let mut padding = Vec::with_capacity(fill_len);
    while padding.len() < fill_len {
        let remaining = fill_len - padding.len();
        if remaining >= fill_units.len() {
            padding.extend_from_slice(&fill_units);
        } else {
            padding.extend_from_slice(&fill_units[..remaining]);
        }
    }

    let mut result = Vec::with_capacity(max_length);
    if at_start {
        result.extend_from_slice(&padding);
        result.extend_from_slice(&source_units);
    } else {
        result.extend_from_slice(&source_units);
        result.extend_from_slice(&padding);
    }
    Ok(string_from_code_units(cx, &result))
}

fn string_pad_end_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    string_pad_builtin(cx, invocation, false)
}

fn string_pad_start_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    string_pad_builtin(cx, invocation, true)
}

fn string_repeat_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let count = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if count < 0.0 || !count.is_finite() {
        return Err(range_error(cx));
    }

    let repeat_count = count as usize;
    if repeat_count == 0 || units.is_empty() {
        return Ok(string_from_code_units(cx, &[]));
    }
    let result_len = units
        .len()
        .checked_mul(repeat_count)
        .ok_or_else(|| range_error(cx))?;
    if u32::try_from(result_len).is_err() {
        return Err(range_error(cx));
    }

    let mut result = Vec::with_capacity(result_len);
    for _ in 0..repeat_count {
        result.extend_from_slice(&units);
    }
    Ok(string_from_code_units(cx, &result))
}

fn string_replace_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let search_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let replacement = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    if search_value.as_object_ref().is_some() {
        if let Some(replacer) =
            regexp::get_method_for_well_known_symbol(cx, search_value, WellKnownSymbolId::Replace)?
        {
            return cx.call_to_completion(
                replacer,
                search_value,
                &[invocation.this_value(), replacement],
            );
        }
    }

    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let source_value = Value::from_string_ref(source_ref);
    let callable_replacement = callable_object_from_value(cx, replacement);
    let pattern_ref = to_string_string_ref(cx, search_value)?;
    let pattern_units = string_ref_code_units(cx, pattern_ref)?;
    let replacement_template = if callable_replacement.is_none() {
        let replacement_ref = to_string_string_ref(cx, replacement)?;
        Some(string_ref_code_units(cx, replacement_ref)?)
    } else {
        None
    };

    let Some(start) = find_subsequence(&source_units, &pattern_units, 0) else {
        return Ok(Value::from_string_ref(source_ref));
    };
    let end = start + pattern_units.len();
    let replacement_units = if let Some(callee) = callable_replacement {
        let matched_value = string_from_code_units(cx, &source_units[start..end]);
        let arguments = [matched_value, usize_index_value(start), source_value];
        let replaced = cx.call_to_completion(callee, Value::undefined(), &arguments)?;
        let replaced_ref = to_string_string_ref(cx, replaced)?;
        string_ref_code_units(cx, replaced_ref)?
    } else {
        expand_string_replacement_template(
            replacement_template
                .as_deref()
                .expect("template units should exist for non-callable string replacement"),
            &source_units,
            start..end,
        )
    };

    let mut result = Vec::with_capacity(source_units.len() + replacement_units.len());
    result.extend_from_slice(&source_units[..start]);
    result.extend_from_slice(&replacement_units);
    result.extend_from_slice(&source_units[end..]);
    Ok(string_from_code_units(cx, &result))
}

fn string_split_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let separator_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let limit_value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    if separator_value.as_object_ref().is_some() {
        if let Some(splitter) =
            regexp::get_method_for_well_known_symbol(cx, separator_value, WellKnownSymbolId::Split)?
        {
            return cx.call_to_completion(
                splitter,
                separator_value,
                &[invocation.this_value(), limit_value],
            );
        }
    }

    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let limit = if let Some(value) = invocation.arguments().get(1).copied() {
        if value.is_undefined() {
            u32::MAX
        } else {
            to_uint32_for_builtin(cx, value)?
        }
    } else {
        u32::MAX
    };
    let separator_units = if separator_value.is_undefined() {
        None
    } else {
        let separator_ref = to_string_string_ref(cx, separator_value)?;
        Some(string_ref_code_units(cx, separator_ref)?)
    };

    let mut parts: Vec<Vec<u16>> = Vec::new();
    if limit == 0 {
        return Ok(Value::from_object_ref(allocate_array_like_result(cx, 0)?));
    }

    match separator_units {
        None => parts.push(source_units.clone()),
        Some(ref separator) if separator.is_empty() => {
            for unit in &source_units {
                if parts.len() >= usize::try_from(limit).unwrap_or(usize::MAX) {
                    break;
                }
                parts.push(vec![*unit]);
            }
        }
        Some(separator) => {
            let mut start = 0;
            loop {
                if parts.len() >= usize::try_from(limit).unwrap_or(usize::MAX) {
                    break;
                }
                let Some(index) = find_subsequence(&source_units, &separator, start) else {
                    parts.push(source_units[start..].to_vec());
                    break;
                };
                parts.push(source_units[start..index].to_vec());
                start = index + separator.len();
                if start > source_units.len() {
                    if parts.len() < usize::try_from(limit).unwrap_or(usize::MAX) {
                        parts.push(Vec::new());
                    }
                    break;
                }
            }
        }
    }

    let object = allocate_array_like_result(cx, u32::try_from(parts.len()).unwrap_or(u32::MAX))?;
    for (index, part) in parts.iter().enumerate() {
        let part_value = string_from_code_units(cx, part);
        define_data_property_with_attrs(
            cx,
            object,
            PropertyKey::Index(u32::try_from(index).unwrap_or(u32::MAX)),
            part_value,
            true,
            true,
            true,
        )?;
    }
    Ok(Value::from_object_ref(object))
}

fn string_slice_index(value: f64, length: usize) -> usize {
    if value.is_nan() || value == f64::NEG_INFINITY {
        return 0;
    }
    if value < 0.0 {
        let offset = (-value).min(length as f64) as usize;
        return length.saturating_sub(offset);
    }
    if !value.is_finite() {
        return length;
    }
    (value as usize).min(length)
}

fn string_slice_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let length = units.len();
    let start = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let end = if let Some(value) = invocation.arguments().get(1).copied() {
        if value.is_undefined() {
            length as f64
        } else {
            to_integer_or_infinity_for_builtin(cx, value)?
        }
    } else {
        length as f64
    };
    let from = string_slice_index(start, length);
    let to = string_slice_index(end, length);
    if to <= from {
        return Ok(string_from_code_units(cx, &[]));
    }
    Ok(string_from_code_units(cx, &units[from..to]))
}

fn string_substring_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let length = units.len();
    let start = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let end = if let Some(value) = invocation.arguments().get(1).copied() {
        if value.is_undefined() {
            length as f64
        } else {
            to_integer_or_infinity_for_builtin(cx, value)?
        }
    } else {
        length as f64
    };

    let clamp = |value: f64| -> usize {
        if value.is_nan() || value <= 0.0 {
            0
        } else if !value.is_finite() {
            length
        } else {
            (value as usize).min(length)
        }
    };

    let start_index = clamp(start);
    let end_index = clamp(end);
    let (from, to) = if start_index <= end_index {
        (start_index, end_index)
    } else {
        (end_index, start_index)
    };
    Ok(string_from_code_units(cx, &units[from..to]))
}

fn string_starts_with_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_units = string_ref_code_units(cx, source_ref)?;
    let search_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if regexp::regexp_search_value_is_rejected(cx, search_value)? {
        return Err(type_error(cx));
    }
    let search_ref = to_string_string_ref(cx, search_value)?;
    let search_units = string_ref_code_units(cx, search_ref)?;
    let position = if let Some(value) = invocation.arguments().get(1).copied() {
        to_integer_or_infinity_for_builtin(cx, value)?
    } else {
        0.0
    };
    let start = if position.is_nan() || position <= 0.0 {
        0
    } else if !position.is_finite() {
        source_units.len()
    } else {
        (position as usize).min(source_units.len())
    };
    let end = start.saturating_add(search_units.len());
    let matches = end <= source_units.len() && source_units[start..end] == search_units[..];
    Ok(Value::from_bool(matches))
}

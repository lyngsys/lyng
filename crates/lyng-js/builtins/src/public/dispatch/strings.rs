mod basic;
mod normalization;

use super::{
    allocate_array_like_result, callable_object_from_value, define_data_property_with_attrs,
    range_error, regexp, string_from_code_units, string_ref_code_units, string_this_ref,
    to_integer_or_infinity_for_builtin, to_length_for_builtin, to_number_for_builtin,
    to_string_string_ref, to_uint32_for_builtin, type_error, usize_index_value,
    PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use basic::{
    dispatch_string_basic_builtin, dispatch_string_constructor_builtin,
    dispatch_string_iterator_builtin,
};
use lyng_js_types::{BuiltinFunctionId, PropertyKey, Value, WellKnownSymbolId};
use normalization::{string_locale_compare_builtin, string_normalize_builtin};

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
    if entry == super::string_substr_builtin() {
        return string_substr_builtin(context, invocation).map(Some);
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
    if let Some(spec) = string_html_method_spec(entry) {
        return string_html_builtin(context, invocation, spec).map(Some);
    }
    Ok(None)
}

#[derive(Clone, Copy)]
struct StringHtmlMethodSpec {
    tag: &'static str,
    attribute: Option<&'static str>,
}

fn string_html_method_spec(entry: BuiltinFunctionId) -> Option<StringHtmlMethodSpec> {
    let spec = if entry == super::string_anchor_builtin() {
        StringHtmlMethodSpec {
            tag: "a",
            attribute: Some("name"),
        }
    } else if entry == super::string_big_builtin() {
        StringHtmlMethodSpec {
            tag: "big",
            attribute: None,
        }
    } else if entry == super::string_blink_builtin() {
        StringHtmlMethodSpec {
            tag: "blink",
            attribute: None,
        }
    } else if entry == super::string_bold_builtin() {
        StringHtmlMethodSpec {
            tag: "b",
            attribute: None,
        }
    } else if entry == super::string_fixed_builtin() {
        StringHtmlMethodSpec {
            tag: "tt",
            attribute: None,
        }
    } else if entry == super::string_fontcolor_builtin() {
        StringHtmlMethodSpec {
            tag: "font",
            attribute: Some("color"),
        }
    } else if entry == super::string_fontsize_builtin() {
        StringHtmlMethodSpec {
            tag: "font",
            attribute: Some("size"),
        }
    } else if entry == super::string_italics_builtin() {
        StringHtmlMethodSpec {
            tag: "i",
            attribute: None,
        }
    } else if entry == super::string_link_builtin() {
        StringHtmlMethodSpec {
            tag: "a",
            attribute: Some("href"),
        }
    } else if entry == super::string_small_builtin() {
        StringHtmlMethodSpec {
            tag: "small",
            attribute: None,
        }
    } else if entry == super::string_strike_builtin() {
        StringHtmlMethodSpec {
            tag: "strike",
            attribute: None,
        }
    } else if entry == super::string_sub_builtin() {
        StringHtmlMethodSpec {
            tag: "sub",
            attribute: None,
        }
    } else if entry == super::string_sup_builtin() {
        StringHtmlMethodSpec {
            tag: "sup",
            attribute: None,
        }
    } else {
        return None;
    };
    Some(spec)
}

fn append_ascii_units(output: &mut Vec<u16>, text: &str) {
    output.extend(text.bytes().map(u16::from));
}

fn append_html_attribute_value_units(output: &mut Vec<u16>, input: &[u16]) {
    for unit in input {
        if *unit == u16::from(b'"') {
            append_ascii_units(output, "&quot;");
        } else {
            output.push(*unit);
        }
    }
}

fn string_html_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    spec: StringHtmlMethodSpec,
) -> Result<Value, Cx::Error> {
    let text_ref = string_this_ref(cx, invocation.this_value())?;
    let text_units = string_ref_code_units(cx, text_ref)?;
    let attribute_units = if let Some(attribute) = spec.attribute {
        let value_ref = to_string_string_ref(
            cx,
            invocation
                .arguments()
                .first()
                .copied()
                .unwrap_or(Value::undefined()),
        )?;
        Some((attribute, string_ref_code_units(cx, value_ref)?))
    } else {
        None
    };

    let mut html = Vec::with_capacity(text_units.len() + 8 + spec.tag.len() * 2);
    html.push(u16::from(b'<'));
    append_ascii_units(&mut html, spec.tag);
    if let Some((attribute, value_units)) = attribute_units {
        html.push(u16::from(b' '));
        append_ascii_units(&mut html, attribute);
        append_ascii_units(&mut html, "=\"");
        append_html_attribute_value_units(&mut html, &value_units);
        html.push(u16::from(b'"'));
    }
    html.push(u16::from(b'>'));
    html.extend_from_slice(&text_units);
    append_ascii_units(&mut html, "</");
    append_ascii_units(&mut html, spec.tag);
    html.push(u16::from(b'>'));
    Ok(string_from_code_units(cx, &html))
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

fn string_index_of_units(source: &[u16], search: &[u16], position: usize) -> i32 {
    find_subsequence(source, search, position)
        .and_then(|index| i32::try_from(index).ok())
        .unwrap_or(-1)
}

pub(super) fn string_index_of_builtin<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn string_replace_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn string_substr_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let units = string_ref_code_units(cx, string)?;
    let size = units.len();
    let start = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let start_index = if start == f64::NEG_INFINITY {
        0
    } else if start < 0.0 {
        size.saturating_sub((-start).min(size as f64) as usize)
    } else if !start.is_finite() {
        size
    } else {
        (start as usize).min(size)
    };

    let substring_length = if let Some(value) = invocation.arguments().get(1).copied() {
        if value.is_undefined() {
            size as f64
        } else {
            to_integer_or_infinity_for_builtin(cx, value)?
        }
    } else {
        size as f64
    };
    let remaining = size.saturating_sub(start_index);
    let count = if substring_length <= 0.0 {
        0
    } else if !substring_length.is_finite() {
        remaining
    } else {
        (substring_length as usize).min(remaining)
    };
    let end = start_index + count;
    Ok(string_from_code_units(cx, &units[start_index..end]))
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

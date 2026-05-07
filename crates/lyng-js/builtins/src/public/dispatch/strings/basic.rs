use super::super::{
    append_string_ref_code_units, array_like_index_property_key,
    iterators::{string_iterator_builtin, string_iterator_next_builtin},
    map_completion, number_to_u32_after_range_check, number_to_usize_after_range_check,
    numbers_are_equal, primitive_wrapper_constructor, property_key_from_text, range_error,
    string_from_code_units, string_ref_code_unit_len, string_ref_code_units, string_this_ref,
    string_value, symbol_descriptive_string, to_integer_or_infinity_for_builtin,
    to_length_for_builtin, to_number_for_builtin, to_string_string_ref, to_uint32_for_builtin,
    type_error, usize_index_as_number, BuiltinToPrimitiveBridge, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_common::WellKnownAtom;
use lyng_js_objects::PrimitiveWrapperKind;
use lyng_js_ops::object;
use lyng_js_types::{BuiltinFunctionId, PropertyKey, Value};

pub(super) fn dispatch_string_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::super::string_builtin() {
        return string_builtin(context, invocation).map(Some);
    }
    if entry == super::super::string_from_char_code_builtin() {
        return string_from_char_code_builtin(context, invocation).map(Some);
    }
    if entry == super::super::string_from_code_point_builtin() {
        return string_from_code_point_builtin(context, invocation).map(Some);
    }
    if entry == super::super::string_raw_builtin() {
        return string_raw_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

pub(super) fn dispatch_string_iterator_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::super::string_iterator_builtin() {
        return string_iterator_builtin(context, invocation).map(Some);
    }
    if entry == super::super::string_iterator_next_builtin() {
        return string_iterator_next_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

pub(super) fn dispatch_string_basic_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::super::string_to_string_builtin() {
        return string_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::super::string_value_of_builtin() {
        return string_value_of_builtin(context, invocation).map(Some);
    }
    if entry == super::super::string_concat_builtin() {
        return string_concat_builtin(context, invocation).map(Some);
    }
    if entry == super::super::string_char_at_builtin() {
        return string_char_at_builtin(context, invocation).map(Some);
    }
    if entry == super::super::string_char_code_at_builtin() {
        return string_char_code_at_builtin(context, invocation).map(Some);
    }
    if entry == super::super::string_at_builtin() {
        return string_at_builtin(context, invocation).map(Some);
    }
    if entry == super::super::string_code_point_at_builtin() {
        return string_code_point_at_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn string_value_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Value, Cx::Error> {
    if let Some(string) = value.as_string_ref() {
        return Ok(Value::from_string_ref(string));
    }
    if let Some(symbol) = value.as_symbol_ref() {
        let text = symbol_descriptive_string(cx, symbol)?;
        return Ok(string_value(cx, &text));
    }
    if value.is_object() {
        let primitive = {
            let mut bridge = BuiltinToPrimitiveBridge { cx };
            object::to_primitive(&mut bridge, value, object::ToPrimitiveHint::String)?
        };
        let string = to_string_string_ref(cx, primitive)?;
        return Ok(Value::from_string_ref(string));
    }
    if value.is_bigint() {
        let text = {
            let agent = cx.agent();
            object::bigint_to_string(agent, value, 10)
        };
        let text = map_completion(cx, text)?;
        return Ok(string_value(cx, &text));
    }

    let text = cx.value_to_string_text(value)?;
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
    let mut strings = Vec::with_capacity(invocation.arguments().len() + 1);
    strings.push(this_string);
    for argument in invocation.arguments() {
        strings.push(to_string_string_ref(cx, *argument)?);
    }

    let mut total_len = 0_usize;
    for string in strings.iter().copied() {
        total_len = total_len
            .checked_add(string_ref_code_unit_len(cx, string)?)
            .expect("string concat length must fit into usize");
    }

    let mut units = Vec::with_capacity(total_len);
    for string in strings {
        append_string_ref_code_units(cx, string, &mut units)?;
    }
    Ok(string_from_code_units(cx, &units))
}

fn string_position_index(position: f64, length: usize) -> Option<usize> {
    if position == 0.0 {
        return (length > 0).then_some(0);
    }
    if !position.is_finite() || position < 0.0 {
        return None;
    }
    let index = number_to_usize_after_range_check(position);
    (index < length).then_some(index)
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
    Ok(string_from_code_units(cx, &units[index..=index]))
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
        let unit = if let Some(value) = value.as_smi() {
            u16::try_from(value.cast_unsigned() & 0xffff)
                .expect("masked UTF-16 code unit should fit into u16")
        } else {
            u16::try_from(to_uint32_for_builtin(cx, value)? & 0xffff)
                .expect("masked UTF-16 code unit should fit into u16")
        };
        units.push(unit);
    }
    Ok(string_from_code_units(cx, &units))
}

fn append_code_point_units(units: &mut Vec<u16>, code_point: u32) {
    if code_point <= 0xFFFF {
        units.push(u16::try_from(code_point).expect("BMP code point should fit into u16"));
        return;
    }

    let adjusted = code_point - 0x1_0000;
    let high = u16::try_from(adjusted >> 10).expect("high surrogate payload should fit into u16");
    let low = u16::try_from(adjusted & 0x03FF).expect("low surrogate payload should fit into u16");
    units.push(0xD800 | high);
    units.push(0xDC00 | low);
}

fn string_from_code_point_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let mut units = Vec::with_capacity(invocation.arguments().len());
    for value in invocation.arguments().iter().copied() {
        if let Some(value) = value.as_smi() {
            let Ok(code_point) = u32::try_from(value) else {
                return Err(range_error(cx));
            };
            if code_point > 0x0010_FFFF {
                return Err(range_error(cx));
            }
            append_code_point_units(&mut units, code_point);
        } else {
            let number = to_number_for_builtin(cx, value)?;
            if !number.is_finite()
                || !numbers_are_equal(number, number.trunc())
                || !(0.0..=1_114_111.0).contains(&number)
            {
                return Err(range_error(cx));
            }
            append_code_point_units(&mut units, number_to_u32_after_range_check(number));
        }
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
        append_string_ref_code_units(cx, segment, &mut result)?;

        if index + 1 == literal_segments {
            break;
        }
        if let Some(substitution) = invocation.arguments().get(index + 1).copied() {
            let substitution = to_string_string_ref(cx, substitution)?;
            append_string_ref_code_units(cx, substitution, &mut result)?;
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
        usize_index_as_number(units.len()) + relative_index
    } else {
        relative_index
    };
    if !index.is_finite() || index < 0.0 || index >= usize_index_as_number(units.len()) {
        return Ok(Value::undefined());
    }
    let index = number_to_usize_after_range_check(index);
    Ok(string_from_code_units(cx, &units[index..=index]))
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
        units.get(index + 1).copied().map_or_else(
            || u32::from(first),
            |second| {
                if (0xDC00..=0xDFFF).contains(&second) {
                    0x1_0000 + ((u32::from(first - 0xD800)) << 10) + u32::from(second - 0xDC00)
                } else {
                    u32::from(first)
                }
            },
        )
    } else {
        u32::from(first)
    };
    Ok(Value::from_smi(
        i32::try_from(code_point).expect("Unicode code points fit into i32"),
    ))
}

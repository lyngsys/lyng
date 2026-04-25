use super::strings::push_code_point_units;
use super::{
    number_value, string_from_code_units, string_ref_code_units, string_value,
    to_number_for_builtin, to_number_value_for_builtin, to_string_string_ref, uri_error,
    PublicBuiltinDispatchContext,
};
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

fn dispatch_global_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_eval_builtin() {
        return eval_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_parse_int_builtin() {
        return parse_int_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_parse_float_builtin() {
        return parse_float_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_is_nan_builtin() {
        return is_nan_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_is_finite_builtin() {
        return is_finite_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_encode_uri_builtin() {
        return encode_uri_builtin(context, invocation, false).map(Some);
    }
    if entry == super::js3_encode_uri_component_builtin() {
        return encode_uri_builtin(context, invocation, true).map(Some);
    }
    if entry == super::js3_decode_uri_builtin() {
        return decode_uri_builtin(context, invocation, false).map(Some);
    }
    if entry == super::js3_decode_uri_component_builtin() {
        return decode_uri_builtin(context, invocation, true).map(Some);
    }
    Ok(None)
}

fn to_int32(number: f64) -> i32 {
    if !number.is_finite() || number == 0.0 {
        return 0;
    }
    let truncated = number.trunc();
    let modulo = truncated.rem_euclid(4_294_967_296.0);
    if modulo >= 2_147_483_648.0 {
        (modulo - 4_294_967_296.0) as i32
    } else {
        modulo as i32
    }
}

fn is_ecmascript_whitespace(ch: char) -> bool {
    matches!(
        ch,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{0020}' | '\u{00A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
                | '\n'
                | '\r'
    )
}

fn parse_ascii_digit(byte: u8, radix: u32) -> Option<u32> {
    let digit = match byte {
        b'0'..=b'9' => u32::from(byte - b'0'),
        b'a'..=b'z' => u32::from(byte - b'a') + 10,
        b'A'..=b'Z' => u32::from(byte - b'A') + 10,
        _ => return None,
    };
    (digit < radix).then_some(digit)
}

fn parse_int_string(text: &str, radix_number: f64) -> f64 {
    let mut input = text.trim_start_matches(is_ecmascript_whitespace);
    let mut sign: f64 = 1.0;
    if let Some(rest) = input.strip_prefix('-') {
        sign = -1.0;
        input = rest;
    } else if let Some(rest) = input.strip_prefix('+') {
        input = rest;
    }

    let mut radix = to_int32(radix_number);
    let mut strip_prefix = true;
    if radix != 0 {
        if !(2..=36).contains(&radix) {
            return f64::NAN;
        }
        if radix != 16 {
            strip_prefix = false;
        }
    } else {
        radix = 10;
    }

    if strip_prefix && (input.starts_with("0x") || input.starts_with("0X")) {
        input = &input[2..];
        radix = 16;
    }

    let mut value: f64 = 0.0;
    let mut consumed = 0_usize;
    for byte in input.bytes() {
        let Some(digit) = parse_ascii_digit(byte, radix as u32) else {
            break;
        };
        value = value.mul_add(f64::from(radix), f64::from(digit));
        consumed += 1;
    }
    if consumed == 0 {
        return f64::NAN;
    }
    let result = sign * value;
    if result == 0.0 && sign.is_sign_negative() {
        -0.0
    } else {
        result
    }
}

fn parse_float_string(text: &str) -> f64 {
    let input = text.trim_start_matches(is_ecmascript_whitespace);
    if input.is_empty() {
        return f64::NAN;
    }
    if let Some(rest) = input.strip_prefix("+Infinity") {
        if rest.is_empty() || !rest.starts_with(['n', 'N']) {
            return f64::INFINITY;
        }
    }
    if let Some(rest) = input.strip_prefix("-Infinity") {
        if rest.is_empty() || !rest.starts_with(['n', 'N']) {
            return f64::NEG_INFINITY;
        }
    }
    if let Some(rest) = input.strip_prefix("Infinity") {
        if rest.is_empty() || !rest.starts_with(['n', 'N']) {
            return f64::INFINITY;
        }
    }

    let bytes = input.as_bytes();
    let mut index = 0_usize;
    if matches!(bytes.first(), Some(b'+' | b'-')) {
        index += 1;
    }
    let mut seen_digit = false;
    while bytes
        .get(index)
        .copied()
        .is_some_and(|byte| byte.is_ascii_digit())
    {
        index += 1;
        seen_digit = true;
    }
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        while bytes
            .get(index)
            .copied()
            .is_some_and(|byte| byte.is_ascii_digit())
        {
            index += 1;
            seen_digit = true;
        }
    }
    if !seen_digit {
        return f64::NAN;
    }

    let exponent_start = index;
    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        let mut exponent_index = index + 1;
        if matches!(bytes.get(exponent_index), Some(b'+' | b'-')) {
            exponent_index += 1;
        }
        let exponent_digits_start = exponent_index;
        while bytes
            .get(exponent_index)
            .copied()
            .is_some_and(|byte| byte.is_ascii_digit())
        {
            exponent_index += 1;
        }
        if exponent_index > exponent_digits_start {
            index = exponent_index;
        } else {
            index = exponent_start;
        }
    }

    input[..index].parse::<f64>().unwrap_or(f64::NAN)
}

fn is_uri_unescaped(component: bool, ch: char) -> bool {
    matches!(ch, 'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '!' | '~' | '*' | '\'' | '(' | ')')
        || (!component
            && matches!(
                ch,
                ';' | ',' | '/' | '?' | ':' | '@' | '&' | '=' | '+' | '$' | '#'
            ))
}

fn is_uri_reserved(ch: char) -> bool {
    matches!(
        ch,
        ';' | ',' | '/' | '?' | ':' | '@' | '&' | '=' | '+' | '$' | '#'
    )
}

fn uri_hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn uri_hex_value_unit(unit: u16) -> Option<u8> {
    u8::try_from(unit).ok().and_then(uri_hex_value)
}

fn push_percent_byte(output: &mut String, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    output.push('%');
    output.push(char::from(HEX[usize::from(byte >> 4)]));
    output.push(char::from(HEX[usize::from(byte & 0x0F)]));
}

fn percent_encode_code_point(output: &mut String, code_point: u32) -> Result<(), ()> {
    if code_point <= 0x7F {
        push_percent_byte(output, u8::try_from(code_point).map_err(|_| ())?);
    } else if code_point <= 0x07FF {
        push_percent_byte(
            output,
            0xC0 | u8::try_from(code_point >> 6).map_err(|_| ())?,
        );
        push_percent_byte(
            output,
            0x80 | u8::try_from(code_point & 0x3F).map_err(|_| ())?,
        );
    } else if code_point <= 0xFFFF {
        push_percent_byte(
            output,
            0xE0 | u8::try_from(code_point >> 12).map_err(|_| ())?,
        );
        push_percent_byte(
            output,
            0x80 | u8::try_from((code_point >> 6) & 0x3F).map_err(|_| ())?,
        );
        push_percent_byte(
            output,
            0x80 | u8::try_from(code_point & 0x3F).map_err(|_| ())?,
        );
    } else if code_point <= 0x0010_FFFF {
        push_percent_byte(
            output,
            0xF0 | u8::try_from(code_point >> 18).map_err(|_| ())?,
        );
        push_percent_byte(
            output,
            0x80 | u8::try_from((code_point >> 12) & 0x3F).map_err(|_| ())?,
        );
        push_percent_byte(
            output,
            0x80 | u8::try_from((code_point >> 6) & 0x3F).map_err(|_| ())?,
        );
        push_percent_byte(
            output,
            0x80 | u8::try_from(code_point & 0x3F).map_err(|_| ())?,
        );
    } else {
        return Err(());
    }
    Ok(())
}

fn encode_uri_units(units: &[u16], component: bool) -> Result<String, ()> {
    let mut encoded = String::new();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        let code_point = if (0xD800..=0xDBFF).contains(&unit) {
            let Some(trailing) = units.get(index + 1).copied() else {
                return Err(());
            };
            if !(0xDC00..=0xDFFF).contains(&trailing) {
                return Err(());
            }
            index += 2;
            0x1_0000 + ((u32::from(unit - 0xD800)) << 10) + u32::from(trailing - 0xDC00)
        } else if (0xDC00..=0xDFFF).contains(&unit) {
            return Err(());
        } else {
            index += 1;
            u32::from(unit)
        };
        let Some(ch) = char::from_u32(code_point) else {
            return Err(());
        };
        if is_uri_unescaped(component, ch) {
            encoded.push(ch);
            continue;
        }
        percent_encode_code_point(&mut encoded, code_point)?;
    }
    Ok(encoded)
}

fn decode_percent_byte(units: &[u16], index: usize) -> Result<u8, ()> {
    if index + 2 >= units.len() || units[index] != u16::from(b'%') {
        return Err(());
    }
    let high = uri_hex_value_unit(units[index + 1]).ok_or(())?;
    let low = uri_hex_value_unit(units[index + 2]).ok_or(())?;
    Ok((high << 4) | low)
}

fn is_utf8_continuation(byte: u8) -> bool {
    (0x80..=0xBF).contains(&byte)
}

fn decode_utf8_percent_sequence(
    units: &[u16],
    index: usize,
    first: u8,
) -> Result<(u32, usize), ()> {
    if first < 0x80 {
        return Ok((u32::from(first), index + 3));
    }
    let length = match first {
        0xC2..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF4 => 4,
        _ => return Err(()),
    };
    let end = index + length * 3;
    if end > units.len() {
        return Err(());
    }
    let mut bytes = [0_u8; 4];
    bytes[0] = first;
    for offset in 1..length {
        let byte = decode_percent_byte(units, index + offset * 3)?;
        if !is_utf8_continuation(byte) {
            return Err(());
        }
        bytes[offset] = byte;
    }
    let code_point = match length {
        2 => u32::from(first & 0x1F) << 6 | u32::from(bytes[1] & 0x3F),
        3 => {
            if (first == 0xE0 && bytes[1] < 0xA0) || (first == 0xED && bytes[1] > 0x9F) {
                return Err(());
            }
            u32::from(first & 0x0F) << 12
                | u32::from(bytes[1] & 0x3F) << 6
                | u32::from(bytes[2] & 0x3F)
        }
        4 => {
            if (first == 0xF0 && bytes[1] < 0x90) || (first == 0xF4 && bytes[1] > 0x8F) {
                return Err(());
            }
            u32::from(first & 0x07) << 18
                | u32::from(bytes[1] & 0x3F) << 12
                | u32::from(bytes[2] & 0x3F) << 6
                | u32::from(bytes[3] & 0x3F)
        }
        _ => return Err(()),
    };
    if code_point > 0x0010_FFFF || (0xD800..=0xDFFF).contains(&code_point) {
        return Err(());
    }
    Ok((code_point, end))
}

fn decode_uri_units(units: &[u16], component: bool) -> Result<Vec<u16>, ()> {
    let mut index = 0_usize;
    let mut decoded = Vec::with_capacity(units.len());
    while index < units.len() {
        if units[index] != u16::from(b'%') {
            decoded.push(units[index]);
            index += 1;
            continue;
        }

        let first = decode_percent_byte(units, index)?;
        if first < 0x80 {
            let ch = char::from(first);
            if !component && is_uri_reserved(ch) {
                decoded.extend_from_slice(&units[index..index + 3]);
            } else {
                decoded.push(u16::from(first));
            }
            index += 3;
            continue;
        }

        let (code_point, end) = decode_utf8_percent_sequence(units, index, first)?;
        push_code_point_units(&mut decoded, code_point);
        index = end;
    }
    Ok(decoded)
}

fn eval_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(source_ref) = source.as_string_ref() else {
        return Ok(source);
    };
    let source_text = cx.value_to_string_text(Value::from_string_ref(source_ref))?;
    cx.evaluate_script_in_realm(cx.builtin_realm(), &source_text)
}

fn parse_int_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let input = cx.value_to_string_text(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let radix = to_number_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(number_value(parse_int_string(&input, radix)))
}

fn parse_float_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let input = cx.value_to_string_text(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(number_value(parse_float_string(&input)))
}

fn is_nan_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let numeric = to_number_value_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(numeric.as_f64().is_some_and(f64::is_nan)))
}

fn is_finite_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let numeric = to_number_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(numeric.is_finite()))
}

fn encode_uri_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    component: bool,
) -> Result<Value, Cx::Error> {
    let input_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let units = string_ref_code_units(cx, input_ref)?;
    let encoded = encode_uri_units(&units, component).map_err(|()| uri_error(cx))?;
    Ok(string_value(cx, &encoded))
}

fn decode_uri_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    component: bool,
) -> Result<Value, Cx::Error> {
    let input_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let input_units = string_ref_code_units(cx, input_ref)?;
    let decoded = decode_uri_units(&input_units, component).map_err(|()| uri_error(cx))?;
    Ok(string_from_code_units(cx, &decoded))
}

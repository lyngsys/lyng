use super::strings::push_code_point_units;
use super::{
    number_to_i32_after_range_check, number_value, string_from_code_units, string_ref_code_units,
    string_value, to_number_for_builtin, to_number_value_for_builtin, to_string_string_ref,
    type_error, uri_error, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_types::{BuiltinFunctionId, Value};

pub(super) fn dispatch_language_support_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::decode_uri_builtin() {
        return decode_uri_builtin(context, invocation, false).map(Some);
    }
    if entry == super::decode_uri_component_builtin() {
        return decode_uri_builtin(context, invocation, true).map(Some);
    }
    if let Some(result) = dispatch_module_source_builtin(context, entry)? {
        return Ok(Some(result));
    }
    dispatch_global_builtin(context, entry, invocation)
}

fn dispatch_module_source_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::abstract_module_source_builtin() {
        return Err(super::type_error(context));
    }
    if entry == super::abstract_module_source_to_string_tag_getter_builtin() {
        return Ok(Some(Value::undefined()));
    }
    Ok(None)
}

fn dispatch_global_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::eval_builtin() {
        return eval_builtin(context, invocation).map(Some);
    }
    if entry == super::parse_int_builtin() {
        return parse_int_builtin(context, invocation).map(Some);
    }
    if entry == super::parse_float_builtin() {
        return parse_float_builtin(context, invocation).map(Some);
    }
    if entry == super::is_nan_builtin() {
        return is_nan_builtin(context, invocation).map(Some);
    }
    if entry == super::is_finite_builtin() {
        return is_finite_builtin(context, invocation).map(Some);
    }
    if entry == super::encode_uri_builtin() {
        return encode_uri_builtin(context, invocation, false).map(Some);
    }
    if entry == super::encode_uri_component_builtin() {
        return encode_uri_builtin(context, invocation, true).map(Some);
    }
    if entry == super::decode_uri_builtin() {
        return decode_uri_builtin(context, invocation, false).map(Some);
    }
    if entry == super::decode_uri_component_builtin() {
        return decode_uri_builtin(context, invocation, true).map(Some);
    }
    if entry == super::escape_builtin() {
        return escape_builtin(context, invocation).map(Some);
    }
    if entry == super::unescape_builtin() {
        return unescape_builtin(context, invocation).map(Some);
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
        number_to_i32_after_range_check(modulo - 4_294_967_296.0)
    } else {
        number_to_i32_after_range_check(modulo)
    }
}

const fn is_ecmascript_whitespace(ch: char) -> bool {
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
        let Some(digit) = parse_ascii_digit(byte, radix.cast_unsigned()) else {
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
    if let Some(rest) = input.strip_prefix("+Infinity")
        && (rest.is_empty() || !rest.starts_with(['n', 'N']))
    {
        return f64::INFINITY;
    }
    if let Some(rest) = input.strip_prefix("-Infinity")
        && (rest.is_empty() || !rest.starts_with(['n', 'N']))
    {
        return f64::NEG_INFINITY;
    }
    if let Some(rest) = input.strip_prefix("Infinity")
        && (rest.is_empty() || !rest.starts_with(['n', 'N']))
    {
        return f64::INFINITY;
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

const fn is_uri_unescaped(component: bool, ch: char) -> bool {
    matches!(ch, 'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '!' | '~' | '*' | '\'' | '(' | ')')
        || (!component
            && matches!(
                ch,
                ';' | ',' | '/' | '?' | ':' | '@' | '&' | '=' | '+' | '$' | '#'
            ))
}

const fn is_uri_reserved(ch: char) -> bool {
    matches!(
        ch,
        ';' | ',' | '/' | '?' | ':' | '@' | '&' | '=' | '+' | '$' | '#'
    )
}

const fn uri_hex_value(byte: u8) -> Option<u8> {
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

fn push_legacy_escape_unit(output: &mut String, unit: u16) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    output.push('%');
    if unit <= 0x00FF {
        output.push(char::from(HEX[usize::from(unit >> 4)]));
    } else {
        output.push('u');
        output.push(char::from(HEX[usize::from((unit >> 12) & 0x000F)]));
        output.push(char::from(HEX[usize::from((unit >> 8) & 0x000F)]));
        output.push(char::from(HEX[usize::from((unit >> 4) & 0x000F)]));
    }
    output.push(char::from(HEX[usize::from(unit & 0x000F)]));
}

const fn is_legacy_escape_unescaped(unit: u16) -> bool {
    matches!(
        unit,
        0x0041..=0x005A
            | 0x0061..=0x007A
            | 0x0030..=0x0039
            | 0x0040
            | 0x002A
            | 0x005F
            | 0x002B
            | 0x002D
            | 0x002E
            | 0x002F
    )
}

fn legacy_escape_units(units: &[u16]) -> String {
    let mut output = String::new();
    for unit in units.iter().copied() {
        if is_legacy_escape_unescaped(unit) {
            output.push(char::from(
                u8::try_from(unit).expect("unescaped escape() unit should be ASCII"),
            ));
        } else {
            push_legacy_escape_unit(&mut output, unit);
        }
    }
    output
}

fn legacy_unescape_hex_unit(units: &[u16], start: usize, len: usize) -> Option<u16> {
    let mut value = 0_u16;
    for offset in 0..len {
        let digit = u16::from(uri_hex_value_unit(*units.get(start + offset)?)?);
        value = (value << 4) | digit;
    }
    Some(value)
}

fn legacy_unescape_units(units: &[u16]) -> Vec<u16> {
    let mut output = Vec::with_capacity(units.len());
    let mut index = 0_usize;
    while index < units.len() {
        if units[index] == u16::from(b'%') {
            if index + 5 < units.len()
                && units[index + 1] == u16::from(b'u')
                && let Some(unit) = legacy_unescape_hex_unit(units, index + 2, 4)
            {
                output.push(unit);
                index += 6;
                continue;
            }
            if index + 2 < units.len()
                && let Some(unit) = legacy_unescape_hex_unit(units, index + 1, 2)
            {
                output.push(unit);
                index += 3;
                continue;
            }
        }
        output.push(units[index]);
        index += 1;
    }
    output
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

#[derive(Clone, Copy)]
enum UriCodeUnits<'a> {
    Latin1(&'a [u8]),
    Utf16(&'a [u8]),
}

impl UriCodeUnits<'_> {
    const fn len(self) -> usize {
        match self {
            Self::Latin1(bytes) => bytes.len(),
            Self::Utf16(bytes) => bytes.len() / 2,
        }
    }

    fn unit(self, index: usize) -> Option<u16> {
        match self {
            Self::Latin1(bytes) => bytes.get(index).copied().map(u16::from),
            Self::Utf16(bytes) => {
                let offset = index.checked_mul(2)?;
                let chunk = bytes.get(offset..offset + 2)?;
                Some(u16::from_le_bytes([chunk[0], chunk[1]]))
            }
        }
    }

    fn extend_units(self, output: &mut Vec<u16>, start: usize, end: usize) -> Result<(), ()> {
        for index in start..end {
            output.push(self.unit(index).ok_or(())?);
        }
        Ok(())
    }
}

fn decode_percent_byte(units: UriCodeUnits<'_>, index: usize) -> Result<u8, ()> {
    if index + 2 >= units.len() || units.unit(index) != Some(u16::from(b'%')) {
        return Err(());
    }
    let high = uri_hex_value_unit(units.unit(index + 1).ok_or(())?).ok_or(())?;
    let low = uri_hex_value_unit(units.unit(index + 2).ok_or(())?).ok_or(())?;
    Ok((high << 4) | low)
}

fn is_utf8_continuation(byte: u8) -> bool {
    (0x80..=0xBF).contains(&byte)
}

fn decode_utf8_percent_sequence(
    units: UriCodeUnits<'_>,
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
    for (offset, slot) in bytes.iter_mut().enumerate().take(length).skip(1) {
        let byte = decode_percent_byte(units, index + offset * 3)?;
        if !is_utf8_continuation(byte) {
            return Err(());
        }
        *slot = byte;
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

fn decode_latin1_percent_hex_byte(bytes: &[u8], index: usize) -> Result<u8, ()> {
    let high = uri_hex_value(bytes[index + 1]).ok_or(())?;
    let low = uri_hex_value(bytes[index + 2]).ok_or(())?;
    Ok((high << 4) | low)
}

fn decode_exact_four_byte_percent_sequence(bytes: &[u8]) -> Option<Result<[u16; 2], ()>> {
    if bytes.len() != 12
        || bytes[0] != b'%'
        || bytes[3] != b'%'
        || bytes[6] != b'%'
        || bytes[9] != b'%'
    {
        return None;
    }

    let first = match decode_latin1_percent_hex_byte(bytes, 0) {
        Ok(first) if (0xF0..=0xF4).contains(&first) => first,
        Ok(_) => return None,
        Err(()) => return Some(Err(())),
    };
    let second = match decode_latin1_percent_hex_byte(bytes, 3) {
        Ok(second) if is_utf8_continuation(second) => second,
        Ok(_) | Err(()) => return Some(Err(())),
    };
    let third = match decode_latin1_percent_hex_byte(bytes, 6) {
        Ok(third) if is_utf8_continuation(third) => third,
        Ok(_) | Err(()) => return Some(Err(())),
    };
    let fourth = match decode_latin1_percent_hex_byte(bytes, 9) {
        Ok(fourth) if is_utf8_continuation(fourth) => fourth,
        Ok(_) | Err(()) => return Some(Err(())),
    };
    if (first == 0xF0 && second < 0x90) || (first == 0xF4 && second > 0x8F) {
        return Some(Err(()));
    }

    let code_point = u32::from(first & 0x07) << 18
        | u32::from(second & 0x3F) << 12
        | u32::from(third & 0x3F) << 6
        | u32::from(fourth & 0x3F);
    let adjusted = code_point - 0x1_0000;
    let high =
        0xD800 | u16::try_from(adjusted >> 10).expect("high surrogate payload should fit into u16");
    let low = 0xDC00
        | u16::try_from(adjusted & 0x03FF).expect("low surrogate payload should fit into u16");
    Some(Ok([high, low]))
}

fn decode_uri_code_units_to_units(
    units: UriCodeUnits<'_>,
    component: bool,
    decoded: &mut Vec<u16>,
) -> Result<(), ()> {
    let mut index = 0_usize;
    decoded.clear();
    decoded.reserve(units.len());
    while index < units.len() {
        if units.unit(index) != Some(u16::from(b'%')) {
            decoded.push(units.unit(index).ok_or(())?);
            index += 1;
            continue;
        }

        let first = decode_percent_byte(units, index)?;
        if first < 0x80 {
            let ch = char::from(first);
            if !component && is_uri_reserved(ch) {
                units.extend_units(decoded, index, index + 3)?;
            } else {
                decoded.push(u16::from(first));
            }
            index += 3;
            continue;
        }

        let (code_point, end) = decode_utf8_percent_sequence(units, index, first)?;
        push_code_point_units(decoded, code_point);
        index = end;
    }
    Ok(())
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
    let exact_four_byte_result = {
        let Some(view) = ({
            let agent = cx.agent();
            agent.heap().view().string_view(input_ref)
        }) else {
            return Err(type_error(cx));
        };
        view.latin1_bytes()
            .and_then(decode_exact_four_byte_percent_sequence)
    };
    match exact_four_byte_result {
        Some(Ok(units)) => return Ok(string_from_code_units(cx, &units)),
        Some(Err(())) => return Err(uri_error(cx)),
        None => {}
    }

    let mut decoded = cx.take_string_code_units_scratch();
    let decode_result = {
        let Some(view) = ({
            let agent = cx.agent();
            agent.heap().view().string_view(input_ref)
        }) else {
            cx.recycle_string_code_units_scratch(decoded);
            return Err(type_error(cx));
        };
        let units = view.latin1_bytes().map_or_else(
            || {
                UriCodeUnits::Utf16(
                    view.utf16_bytes()
                        .expect("runtime string view should expose payload bytes"),
                )
            },
            UriCodeUnits::Latin1,
        );
        decode_uri_code_units_to_units(units, component, &mut decoded)
    };
    if decode_result.is_err() {
        cx.recycle_string_code_units_scratch(decoded);
        return Err(uri_error(cx));
    }
    let value = string_from_code_units(cx, &decoded);
    cx.recycle_string_code_units_scratch(decoded);
    Ok(value)
}

fn escape_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
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
    let escaped = legacy_escape_units(&input_units);
    Ok(string_value(cx, &escaped))
}

fn unescape_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
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
    let unescaped = legacy_unescape_units(&input_units);
    Ok(string_from_code_units(cx, &unescaped))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_uri_code_units_preserves_reserved_and_decodes_four_byte_utf8() {
        let mut decoded = Vec::new();

        decode_uri_code_units_to_units(
            UriCodeUnits::Latin1(b"%2F%F0%9F%98%80"),
            false,
            &mut decoded,
        )
        .expect("valid URI escape sequence should decode");

        assert_eq!(
            decoded,
            vec![
                u16::from(b'%'),
                u16::from(b'2'),
                u16::from(b'F'),
                0xD83D,
                0xDE00
            ]
        );
    }

    #[test]
    fn decode_uri_component_code_units_decodes_reserved_escapes() {
        let mut decoded = Vec::new();

        decode_uri_code_units_to_units(UriCodeUnits::Latin1(b"%2F"), true, &mut decoded)
            .expect("valid URI component escape sequence should decode");

        assert_eq!(decoded, vec![u16::from(b'/')]);
    }

    #[test]
    fn exact_four_byte_percent_sequence_decodes_to_surrogate_pair() {
        let decoded = decode_exact_four_byte_percent_sequence(b"%F0%9F%98%80")
            .expect("exact four-byte sequence should use the fast path")
            .expect("valid four-byte UTF-8 sequence should decode");

        assert_eq!(decoded, [0xD83D, 0xDE00]);
    }

    #[test]
    fn exact_four_byte_percent_sequence_rejects_overlong_encoding() {
        let decoded = decode_exact_four_byte_percent_sequence(b"%F0%80%80%80")
            .expect("exact malformed four-byte sequence should use the fast path");

        assert_eq!(decoded, Err(()));
    }
}

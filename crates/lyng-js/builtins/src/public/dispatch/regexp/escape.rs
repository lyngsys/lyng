use super::super::{string_ref_code_units, string_value, type_error, PublicBuiltinDispatchContext};
use crate::BuiltinInvocation;
use lyng_js_types::Value;
use std::fmt::Write as _;

fn regexp_escape_push_hex(output: &mut String, unit: u16) {
    let _ = write!(output, "\\x{unit:02x}");
}

fn regexp_escape_push_unicode(output: &mut String, unit: u16) {
    let _ = write!(output, "\\u{unit:04x}");
}

fn regexp_escape_is_ascii_letter_or_digit(unit: u16) -> bool {
    (u16::from(b'0')..=u16::from(b'9')).contains(&unit)
        || (u16::from(b'A')..=u16::from(b'Z')).contains(&unit)
        || (u16::from(b'a')..=u16::from(b'z')).contains(&unit)
}

fn regexp_escape_is_other_punctuator(unit: u16) -> bool {
    [
        u16::from(b','),
        u16::from(b'-'),
        u16::from(b'='),
        u16::from(b'<'),
        u16::from(b'>'),
        u16::from(b'#'),
        u16::from(b'&'),
        u16::from(b'!'),
        u16::from(b'%'),
        u16::from(b':'),
        u16::from(b';'),
        u16::from(b'@'),
        u16::from(b'~'),
        u16::from(b'\''),
        u16::from(b'`'),
        u16::from(b'"'),
    ]
    .contains(&unit)
}

fn regexp_escape_is_syntax_character(unit: u16) -> bool {
    [
        u16::from(b'^'),
        u16::from(b'$'),
        u16::from(b'\\'),
        u16::from(b'.'),
        u16::from(b'*'),
        u16::from(b'+'),
        u16::from(b'?'),
        u16::from(b'('),
        u16::from(b')'),
        u16::from(b'['),
        u16::from(b']'),
        u16::from(b'{'),
        u16::from(b'}'),
        u16::from(b'|'),
        u16::from(b'/'),
    ]
    .contains(&unit)
}

fn regexp_escape_is_whitespace_or_line_terminator(unit: u16) -> bool {
    matches!(
        unit,
        0x0009
            | 0x000A
            | 0x000B
            | 0x000C
            | 0x000D
            | 0x0020
            | 0x00A0
            | 0x1680
            | 0x2000
            | 0x2001
            | 0x2002
            | 0x2003
            | 0x2004
            | 0x2005
            | 0x2006
            | 0x2007
            | 0x2008
            | 0x2009
            | 0x200A
            | 0x2028
            | 0x2029
            | 0x202F
            | 0x205F
            | 0x3000
            | 0xFEFF
    )
}

fn regexp_escape_append_encoded_unit(output: &mut String, unit: u16) {
    match unit {
        0x0009 => output.push_str("\\t"),
        0x000A => output.push_str("\\n"),
        0x000B => output.push_str("\\v"),
        0x000C => output.push_str("\\f"),
        0x000D => output.push_str("\\r"),
        _ if regexp_escape_is_syntax_character(unit) => {
            output.push('\\');
            output.push(char::from(
                u8::try_from(unit).expect("syntax characters stay ASCII"),
            ));
        }
        _ if regexp_escape_is_other_punctuator(unit)
            || regexp_escape_is_whitespace_or_line_terminator(unit) =>
        {
            if unit <= 0x00FF {
                regexp_escape_push_hex(output, unit);
            } else {
                regexp_escape_push_unicode(output, unit);
            }
        }
        _ if (0xD800..=0xDFFF).contains(&unit) => regexp_escape_push_unicode(output, unit),
        _ => {
            output.push(
                char::from_u32(u32::from(unit))
                    .expect("non-surrogate UTF-16 code unit should convert to Unicode scalar"),
            );
        }
    }
}

pub(super) fn regexp_escape_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let input = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let string_ref = input.as_string_ref().ok_or_else(|| type_error(cx))?;
    let units = string_ref_code_units(cx, string_ref)?;
    let mut escaped = String::with_capacity(units.len() * 2);
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if index == 0 && regexp_escape_is_ascii_letter_or_digit(unit) {
            regexp_escape_push_hex(&mut escaped, unit);
            index += 1;
            continue;
        }
        if (0xD800..=0xDBFF).contains(&unit)
            && matches!(units.get(index + 1), Some(next) if (0xDC00..=0xDFFF).contains(next))
        {
            let high = u32::from(unit - 0xD800);
            let low = u32::from(units[index + 1] - 0xDC00);
            let code_point = 0x1_0000 + ((high << 10) | low);
            escaped.push(
                char::from_u32(code_point)
                    .expect("valid surrogate pair should convert to Unicode scalar"),
            );
            index += 2;
            continue;
        }
        regexp_escape_append_encoded_unit(&mut escaped, unit);
        index += 1;
    }
    Ok(string_value(cx, &escaped))
}

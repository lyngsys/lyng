use lyng_js_gc::{BigIntSign, PrimitiveBigIntView, PrimitiveStringView};
use lyng_js_types::{AbruptCompletion, Value};
use std::fmt::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LogicalType {
    Undefined,
    Null,
    Boolean,
    Number,
    Object,
    String,
    Symbol,
    BigInt,
    Sentinel,
}

pub(crate) fn logical_type(value: Value) -> LogicalType {
    if value.is_undefined() {
        LogicalType::Undefined
    } else if value.is_null() {
        LogicalType::Null
    } else if value.is_bool() {
        LogicalType::Boolean
    } else if value.is_number() {
        LogicalType::Number
    } else if value.is_object() {
        LogicalType::Object
    } else if value.is_string() {
        LogicalType::String
    } else if value.is_symbol() {
        LogicalType::Symbol
    } else if value.is_bigint() {
        LogicalType::BigInt
    } else {
        LogicalType::Sentinel
    }
}

#[inline]
pub(crate) fn same_logical_type(left: Value, right: Value) -> bool {
    logical_type(left) == logical_type(right)
}

#[inline]
pub(crate) fn phase2_type_error() -> AbruptCompletion {
    // Phase 2 keeps the stable Throw(Value) shape without realm-aware error allocation yet.
    AbruptCompletion::throw(Value::undefined())
}

#[allow(clippy::cast_possible_truncation, clippy::float_cmp)]
pub(crate) fn encode_number(number: f64) -> Value {
    if !number.is_nan()
        && number.is_finite()
        && number == number.trunc()
        && number != 0.0
        && number >= f64::from(i32::MIN)
        && number <= f64::from(i32::MAX)
    {
        Value::from_smi(number as i32)
    } else if number == 0.0 && !number.is_sign_negative() {
        Value::from_smi(0)
    } else {
        Value::from_f64(number)
    }
}

/// Converts one IEEE-754 number into the current ECMAScript number string form.
///
/// # Panics
/// Panics only if the intermediate shortest-decimal digit count stops fitting in `i32`.
pub fn number_to_string(number: f64) -> String {
    if number.is_nan() {
        return "NaN".to_string();
    }
    if number == 0.0 {
        return "0".to_string();
    }
    if number.is_infinite() {
        return if number.is_sign_positive() {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        };
    }

    let negative = number.is_sign_negative();
    let raw = number.abs().to_string();
    let (digits, n) = shortest_decimal_digits_and_exponent(&raw);
    let k = i32::try_from(digits.len()).expect("digit count must fit into i32");
    let mut text = String::new();

    if negative {
        text.push('-');
    }

    if k <= n && n <= 21 {
        text.push_str(&digits);
        for _ in 0..(n - k) {
            text.push('0');
        }
        return text;
    }

    if 0 < n && n <= 21 {
        let split = usize::try_from(n).expect("positive exponent split must fit into usize");
        text.push_str(&digits[..split]);
        text.push('.');
        text.push_str(&digits[split..]);
        return text;
    }

    if -6 < n && n <= 0 {
        text.push_str("0.");
        for _ in 0..(-n) {
            text.push('0');
        }
        text.push_str(&digits);
        return text;
    }

    text.push(digits.as_bytes()[0] as char);
    if k > 1 {
        text.push('.');
        text.push_str(&digits[1..]);
    }
    text.push('e');
    let exponent = n - 1;
    if exponent >= 0 {
        text.push('+');
    } else {
        text.push('-');
    }
    let _ = write!(text, "{}", exponent.abs());
    text
}

pub(crate) fn lossy_string_from_view(view: PrimitiveStringView<'_>) -> String {
    if let Some(bytes) = view.latin1_bytes() {
        let mut text = String::with_capacity(bytes.len());
        for byte in bytes {
            text.push(char::from(*byte));
        }
        return text;
    }

    let bytes = view
        .utf16_bytes()
        .expect("string view must be Latin1 or UTF-16");
    let mut units = Vec::with_capacity(view.code_unit_len() as usize);
    for chunk in bytes.chunks_exact(2) {
        units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    String::from_utf16_lossy(&units)
}

pub(crate) fn string_to_number(text: &str) -> f64 {
    let trimmed = text.trim_matches(is_ecmascript_trim_whitespace);

    if trimmed.is_empty() {
        return 0.0;
    }
    if trimmed == "Infinity" || trimmed == "+Infinity" {
        return f64::INFINITY;
    }
    if trimmed == "-Infinity" {
        return f64::NEG_INFINITY;
    }
    if trimmed.len() > 2 && (trimmed.starts_with("0x") || trimmed.starts_with("0X")) {
        return parse_non_decimal_to_f64(&trimmed[2..], 16);
    }
    if trimmed.len() > 2 && (trimmed.starts_with("0o") || trimmed.starts_with("0O")) {
        return parse_non_decimal_to_f64(&trimmed[2..], 8);
    }
    if trimmed.len() > 2 && (trimmed.starts_with("0b") || trimmed.starts_with("0B")) {
        return parse_non_decimal_to_f64(&trimmed[2..], 2);
    }
    if is_non_ecmascript_infinity_literal(trimmed) {
        return f64::NAN;
    }

    trimmed.parse::<f64>().unwrap_or(f64::NAN)
}

pub(crate) fn parse_string_to_bigint(text: &str) -> Option<(BigIntSign, Vec<u64>)> {
    let trimmed = text.trim_matches(is_ecmascript_trim_whitespace);
    if trimmed.is_empty() {
        return Some((BigIntSign::NonNegative, Vec::new()));
    }

    let (sign, mut digits, signed) = match trimmed.as_bytes()[0] {
        b'+' => (BigIntSign::NonNegative, &trimmed[1..], true),
        b'-' => (BigIntSign::Negative, &trimmed[1..], true),
        _ => (BigIntSign::NonNegative, trimmed, false),
    };
    if digits.is_empty() {
        return None;
    }

    let radix = if digits.len() > 2 && (digits.starts_with("0x") || digits.starts_with("0X")) {
        if signed {
            return None;
        }
        digits = &digits[2..];
        16
    } else if digits.len() > 2 && (digits.starts_with("0o") || digits.starts_with("0O")) {
        if signed {
            return None;
        }
        digits = &digits[2..];
        8
    } else if digits.len() > 2 && (digits.starts_with("0b") || digits.starts_with("0B")) {
        if signed {
            return None;
        }
        digits = &digits[2..];
        2
    } else {
        10
    };
    if digits.is_empty() {
        return None;
    }

    let mut limbs = Vec::new();
    for digit in digits
        .bytes()
        .map(|byte| ascii_digit_value(byte, radix))
        .collect::<Option<Vec<_>>>()?
    {
        mul_add_small(&mut limbs, radix, digit);
    }
    normalize_limbs(&mut limbs);

    let sign = if limbs.is_empty() {
        BigIntSign::NonNegative
    } else {
        sign
    };
    Some((sign, limbs))
}

pub(crate) fn bigint_view_equals_parts(
    view: PrimitiveBigIntView<'_>,
    sign: BigIntSign,
    limbs: &[u64],
) -> bool {
    let mut normalized = limbs.to_vec();
    normalize_limbs(&mut normalized);
    let expected_sign = if normalized.is_empty() {
        BigIntSign::NonNegative
    } else {
        sign
    };
    if view.sign() != expected_sign || view.limb_count() as usize != normalized.len() {
        return false;
    }

    normalized
        .iter()
        .enumerate()
        .all(|(index, limb)| view.limb_at(index) == Some(*limb))
}

pub(crate) fn bigint_view_to_string(view: PrimitiveBigIntView<'_>) -> String {
    format_bigint_decimal_owned(view.sign(), view.to_limbs())
}

pub(crate) fn bigint_view_to_radix_string(view: PrimitiveBigIntView<'_>, radix: u32) -> String {
    format_bigint_radix_owned(view.sign(), view.to_limbs(), radix)
}

pub(crate) fn bigint_parts_to_radix_string(sign: BigIntSign, limbs: &[u64], radix: u32) -> String {
    format_bigint_radix_owned(sign, limbs.to_vec(), radix)
}

pub(crate) fn bigint_equals_integral_number(view: PrimitiveBigIntView<'_>, number: f64) -> bool {
    let Some((sign, limbs)) = integral_number_to_bigint(number) else {
        return false;
    };
    bigint_view_equals_parts(view, sign, &limbs)
}

#[cfg(test)]
fn format_bigint_decimal(sign: BigIntSign, limbs: &[u64]) -> String {
    format_bigint_decimal_owned(sign, limbs.to_vec())
}

fn format_bigint_decimal_owned(sign: BigIntSign, mut magnitude: Vec<u64>) -> String {
    normalize_limbs(&mut magnitude);
    if magnitude.is_empty() {
        return "0".to_string();
    }

    let mut groups = Vec::new();
    while !magnitude.is_empty() {
        let remainder = div_rem_small(&mut magnitude, 1_000_000_000);
        groups.push(remainder);
        normalize_limbs(&mut magnitude);
    }

    let mut text = String::new();
    if sign == BigIntSign::Negative {
        text.push('-');
    }

    let first = groups
        .pop()
        .expect("non-zero bigint formatting must produce at least one group");
    let _ = write!(text, "{first}");
    while let Some(group) = groups.pop() {
        let _ = write!(text, "{group:09}");
    }
    text
}

fn format_bigint_radix_owned(sign: BigIntSign, mut magnitude: Vec<u64>, radix: u32) -> String {
    debug_assert!((2..=36).contains(&radix));
    normalize_limbs(&mut magnitude);
    if magnitude.is_empty() {
        return "0".to_string();
    }

    let mut digits = Vec::new();
    while !magnitude.is_empty() {
        let remainder = div_rem_small(&mut magnitude, radix);
        digits.push(
            b"0123456789abcdefghijklmnopqrstuvwxyz"
                [usize::try_from(remainder).expect("remainder must fit into usize")]
                as char,
        );
        normalize_limbs(&mut magnitude);
    }

    let mut text = String::new();
    if sign == BigIntSign::Negative {
        text.push('-');
    }
    while let Some(digit) = digits.pop() {
        text.push(digit);
    }
    text
}

#[allow(clippy::float_cmp)]
pub(crate) fn integral_number_to_bigint(number: f64) -> Option<(BigIntSign, Vec<u64>)> {
    if !number.is_finite() || number != number.trunc() {
        return None;
    }
    if number == 0.0 {
        return Some((BigIntSign::NonNegative, Vec::new()));
    }

    let bits = number.to_bits();
    let sign = if bits >> 63 == 0 {
        BigIntSign::NonNegative
    } else {
        BigIntSign::Negative
    };
    let exponent_bits = ((bits >> 52) & 0x7ff) as i32;
    if exponent_bits == 0 {
        return None;
    }

    let exponent = exponent_bits - 1023;
    let significand = (1_u64 << 52) | (bits & ((1_u64 << 52) - 1));
    let shift = exponent - 52;

    let mut limbs = if shift >= 0 {
        shift_left_word(significand, shift.cast_unsigned())
    } else {
        let right_shift = (-shift).cast_unsigned();
        if right_shift >= 64 {
            return None;
        }
        let mask = (1_u64 << right_shift) - 1;
        if significand & mask != 0 {
            return None;
        }
        let truncated = significand >> right_shift;
        if truncated == 0 {
            Vec::new()
        } else {
            vec![truncated]
        }
    };
    normalize_limbs(&mut limbs);
    Some((sign, limbs))
}

fn shift_left_word(word: u64, shift: u32) -> Vec<u64> {
    let whole_words = usize::try_from(shift / 64).expect("whole-word shift must fit into usize");
    let bit_shift = shift % 64;
    let mut limbs = vec![0; whole_words];

    if bit_shift == 0 {
        limbs.push(word);
        return limbs;
    }

    limbs.push(word << bit_shift);
    let carry = word >> (64 - bit_shift);
    if carry != 0 {
        limbs.push(carry);
    }
    limbs
}

fn mul_add_small(limbs: &mut Vec<u64>, multiplier: u32, addend: u32) {
    let multiplier = u128::from(multiplier);
    let mut carry = u128::from(addend);

    for limb in limbs.iter_mut() {
        let product = u128::from(*limb) * multiplier + carry;
        *limb = u64::try_from(product & u128::from(u64::MAX))
            .expect("low 64 bits of product must fit into u64");
        carry = product >> 64;
    }

    if carry != 0 {
        limbs.push(u64::try_from(carry).expect("carry limb must fit into u64"));
    }
}

fn ascii_digit_value(byte: u8, radix: u32) -> Option<u32> {
    let digit = match byte {
        b'0'..=b'9' => u32::from(byte - b'0'),
        b'a'..=b'z' => u32::from(byte - b'a') + 10,
        b'A'..=b'Z' => u32::from(byte - b'A') + 10,
        _ => return None,
    };
    (digit < radix).then_some(digit)
}

fn div_rem_small(limbs: &mut [u64], divisor: u32) -> u32 {
    let divisor = u128::from(divisor);
    let mut remainder = 0_u128;

    for limb in limbs.iter_mut().rev() {
        let value = (remainder << 64) | u128::from(*limb);
        *limb = u64::try_from(value / divisor).expect("quotient limb must fit into u64");
        remainder = value % divisor;
    }

    u32::try_from(remainder).expect("small-division remainder must fit into u32")
}

fn normalize_limbs(limbs: &mut Vec<u64>) {
    while limbs.last().copied() == Some(0) {
        limbs.pop();
    }
}

fn shortest_decimal_digits_and_exponent(raw: &str) -> (String, i32) {
    if let Some((mantissa, exponent)) = raw.split_once(['e', 'E']) {
        let exponent = exponent.parse::<i32>().unwrap_or(0);
        let mut digits = mantissa.chars().filter(|ch| *ch != '.').collect::<String>();
        while digits.len() > 1 && digits.ends_with('0') {
            digits.pop();
        }
        debug_assert!(!digits.is_empty(), "finite non-zero numbers need digits");
        return (digits, exponent + 1);
    }

    if let Some((integer, fraction)) = raw.split_once('.') {
        if integer != "0" {
            let mut digits = String::with_capacity(integer.len() + fraction.len());
            digits.push_str(integer);
            digits.push_str(fraction);
            while digits.len() > 1 && digits.ends_with('0') {
                digits.pop();
            }
            return (
                digits,
                i32::try_from(integer.len()).expect("integer digit count must fit into i32"),
            );
        }

        let first_non_zero = fraction
            .bytes()
            .position(|byte| byte != b'0')
            .expect("finite non-zero fractional strings need a non-zero digit");
        let mut digits = fraction[first_non_zero..].to_string();
        while digits.len() > 1 && digits.ends_with('0') {
            digits.pop();
        }
        return (
            digits,
            -i32::try_from(first_non_zero).expect("fractional zero run must fit into i32"),
        );
    }

    let mut digits = raw.to_string();
    while digits.len() > 1 && digits.ends_with('0') {
        digits.pop();
    }
    (
        digits,
        i32::try_from(raw.len()).expect("raw decimal digit count must fit into i32"),
    )
}

fn parse_non_decimal_to_f64(digits: &str, radix: u32) -> f64 {
    if digits.is_empty() {
        return f64::NAN;
    }

    let mut value: f64 = 0.0;
    for byte in digits.bytes() {
        let Some(digit) = ascii_digit_value(byte, radix) else {
            return f64::NAN;
        };
        value = value.mul_add(f64::from(radix), f64::from(digit));
    }
    value
}

fn is_non_ecmascript_infinity_literal(text: &str) -> bool {
    let unsigned = if let Some(rest) = text.strip_prefix('+') {
        rest
    } else if let Some(rest) = text.strip_prefix('-') {
        rest
    } else {
        text
    };

    unsigned.eq_ignore_ascii_case("inf")
        || (unsigned.eq_ignore_ascii_case("infinity")
            && text != "Infinity"
            && text != "+Infinity"
            && text != "-Infinity")
}

fn is_ecmascript_trim_whitespace(ch: char) -> bool {
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

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use lyng_js_gc::{AllocationLifetime, PrimitiveHeap, StringEncoding};

    #[test]
    fn string_to_number_handles_trim_prefixes_and_invalid_input() {
        assert_eq!(string_to_number(""), 0.0);
        assert_eq!(string_to_number("\u{00A0}42\u{FEFF}"), 42.0);
        assert_eq!(string_to_number("0x10"), 16.0);
        assert_eq!(string_to_number("0o10"), 8.0);
        assert_eq!(string_to_number("0b10"), 2.0);
        assert_eq!(
            string_to_number("0x10000000000000000"),
            18_446_744_073_709_552_000.0
        );
        assert_eq!(string_to_number("Infinity"), f64::INFINITY);
        assert_eq!(string_to_number("-Infinity"), f64::NEG_INFINITY);
        assert!(string_to_number("foo").is_nan());
        assert!(string_to_number("+0x10").is_nan());
        assert!(string_to_number("inf").is_nan());
        assert!(string_to_number("-inf").is_nan());
        assert!(string_to_number("+inf").is_nan());
        assert!(string_to_number("infinity").is_nan());
        assert!(string_to_number("INFINITY").is_nan());
    }

    #[test]
    fn bigint_decimal_parse_and_format_round_trip() {
        let parsed = parse_string_to_bigint("  -18446744073709551617 ").unwrap();
        assert_eq!(parsed.0, BigIntSign::Negative);
        assert_eq!(
            format_bigint_decimal(parsed.0, &parsed.1),
            "-18446744073709551617"
        );
    }

    #[test]
    fn bigint_parse_accepts_unsigned_radix_prefixes_only() {
        let hex = parse_string_to_bigint("0x10").unwrap();
        let binary = parse_string_to_bigint("0b101").unwrap();
        let octal = parse_string_to_bigint("0o10").unwrap();

        assert_eq!(format_bigint_decimal(hex.0, &hex.1), "16");
        assert_eq!(format_bigint_decimal(binary.0, &binary.1), "5");
        assert_eq!(format_bigint_decimal(octal.0, &octal.1), "8");
        assert_eq!(parse_string_to_bigint("+0x10"), None);
        assert_eq!(parse_string_to_bigint("-0x10"), None);
    }

    #[test]
    fn number_encoding_preserves_negative_zero_and_uses_smis_for_small_integers() {
        assert_eq!(encode_number(5.0), Value::from_smi(5));
        assert_eq!(encode_number(0.0), Value::from_smi(0));
        assert_eq!(encode_number(-0.0).as_f64(), Some(-0.0));
        assert!(encode_number(f64::NAN).is_nan());
    }

    #[test]
    fn number_to_string_matches_ecmascript_threshold_formatting() {
        assert_eq!(number_to_string(1e21), "1e+21");
        assert_eq!(number_to_string(1e20), "100000000000000000000");
        assert_eq!(number_to_string(1e-7), "1e-7");
        assert_eq!(number_to_string(1e-6), "0.000001");
        assert_eq!(number_to_string(-0.0), "0");
        assert_eq!(
            number_to_string(1_000_000_000_000_000_128.0),
            "1000000000000000100"
        );
    }

    #[test]
    fn lossy_string_materialization_preserves_parse_relevant_content() {
        let mut heap = PrimitiveHeap::new();
        let (latin1, utf16) = {
            let mut mutator = heap.mutator();
            let latin1 = mutator.alloc_string(
                StringEncoding::Latin1,
                4,
                &[0x63, 0x61, 0x66, 0xE9],
                None,
                AllocationLifetime::Default,
            );
            let utf16 = mutator.alloc_string(
                StringEncoding::Utf16,
                1,
                &[0x00, 0xD8],
                None,
                AllocationLifetime::Default,
            );
            (latin1, utf16)
        };
        let view = heap.view();

        assert_eq!(
            lossy_string_from_view(view.string_view(latin1).unwrap()),
            "café"
        );
        assert_eq!(
            lossy_string_from_view(view.string_view(utf16).unwrap()),
            "\u{FFFD}"
        );
    }

    #[test]
    fn bigint_number_comparison_handles_large_exact_integers() {
        let mut heap = PrimitiveHeap::new();
        let bigint = {
            let mut mutator = heap.mutator();
            mutator.alloc_bigint(
                BigIntSign::NonNegative,
                &[0x0010_0000_0000_0000],
                AllocationLifetime::Default,
            )
        };
        let view = heap.view().bigint_view(bigint).unwrap();

        assert!(bigint_equals_integral_number(view, 4_503_599_627_370_496.0));
        assert!(!bigint_equals_integral_number(
            view,
            4_503_599_627_370_497.0
        ));
        assert!(!bigint_equals_integral_number(view, 1.5));
    }
}

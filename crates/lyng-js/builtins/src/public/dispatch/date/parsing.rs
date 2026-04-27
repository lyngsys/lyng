use super::super::PublicBuiltinDispatchContext;
use super::{
    date_days_in_month, date_make_local_value, date_make_utc_value, date_time_clip_value,
    DATE_MONTH_NAMES, DATE_MS_PER_MINUTE,
};
use lyng_js_types::Value;

fn date_parse_two_digits(bytes: &[u8], index: usize) -> Option<u32> {
    let tens = *bytes.get(index)?;
    let ones = *bytes.get(index + 1)?;
    if !tens.is_ascii_digit() || !ones.is_ascii_digit() {
        return None;
    }
    Some(u32::from(tens - b'0') * 10 + u32::from(ones - b'0'))
}

fn date_parse_fixed_digits(bytes: &[u8], index: usize, len: usize) -> Option<i32> {
    let mut value = 0_i32;
    for offset in 0..len {
        let byte = *bytes.get(index + offset)?;
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(i32::from(byte - b'0'))?;
    }
    Some(value)
}

fn date_month_name_index(name: &str) -> Option<u8> {
    DATE_MONTH_NAMES
        .iter()
        .position(|candidate| *candidate == name)
        .and_then(|index| u8::try_from(index + 1).ok())
}

fn date_parse_time(text: &str) -> Option<(u32, u32, u32)> {
    let bytes = text.as_bytes();
    if bytes.len() != 8 || bytes.get(2) != Some(&b':') || bytes.get(5) != Some(&b':') {
        return None;
    }
    Some((
        date_parse_two_digits(bytes, 0)?,
        date_parse_two_digits(bytes, 3)?,
        date_parse_two_digits(bytes, 6)?,
    ))
}

fn date_parse_timezone_offset_colon(text: &str) -> Option<i32> {
    let bytes = text.as_bytes();
    if bytes.len() != 6 || bytes.get(3) != Some(&b':') {
        return None;
    }
    let sign = match bytes[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let hour = i32::try_from(date_parse_two_digits(bytes, 1)?).ok()?;
    let minute = i32::try_from(date_parse_two_digits(bytes, 4)?).ok()?;
    if hour > 23 || minute > 59 {
        return None;
    }
    Some(sign * (hour * 60 + minute))
}

fn date_parse_timezone_offset_compact(text: &str) -> Option<i32> {
    let bytes = text.as_bytes();
    if bytes.len() != 5 {
        return None;
    }
    let sign = match bytes[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let hour = i32::try_from(date_parse_two_digits(bytes, 1)?).ok()?;
    let minute = i32::try_from(date_parse_two_digits(bytes, 3)?).ok()?;
    if hour > 23 || minute > 59 {
        return None;
    }
    Some(sign * (hour * 60 + minute))
}

fn date_validate_iso_date(year: i32, month: u32, day: u32) -> bool {
    if !(1..=12).contains(&month) {
        return false;
    }
    let Ok(month_u8) = u8::try_from(month) else {
        return false;
    };
    (1..=u32::from(date_days_in_month(year, month_u8))).contains(&day)
}

fn date_parse_iso_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    text: &str,
) -> Result<Option<Value>, Cx::Error> {
    let bytes = text.as_bytes();
    let mut index = 0;
    let mut sign = 1_i32;
    let year_digits = match bytes.first().copied() {
        Some(b'+') => {
            index = 1;
            6
        }
        Some(b'-') => {
            index = 1;
            sign = -1;
            6
        }
        _ => 4,
    };
    let Some(mut year) = date_parse_fixed_digits(bytes, index, year_digits) else {
        return Ok(None);
    };
    if sign == -1 && year == 0 && year_digits == 6 {
        return Ok(None);
    }
    year *= sign;
    index += year_digits;

    let mut month = 1_u32;
    let mut day = 1_u32;
    let mut date_only = true;
    if bytes.get(index) == Some(&b'-') {
        index += 1;
        month = date_parse_two_digits(bytes, index).unwrap_or(0);
        index += 2;
        if bytes.get(index) == Some(&b'-') {
            index += 1;
            day = date_parse_two_digits(bytes, index).unwrap_or(0);
            index += 2;
        }
    }
    if !date_validate_iso_date(year, month, day) {
        return Ok(None);
    }

    let mut hour = 0_u32;
    let mut minute = 0_u32;
    let mut second = 0_u32;
    let mut millisecond = 0_u32;
    let mut offset_minutes: Option<i32> = None;

    if bytes.get(index) == Some(&b'T') {
        date_only = false;
        index += 1;
        hour = date_parse_two_digits(bytes, index).unwrap_or(u32::MAX);
        index += 2;
        if bytes.get(index) != Some(&b':') {
            return Ok(None);
        }
        index += 1;
        minute = date_parse_two_digits(bytes, index).unwrap_or(u32::MAX);
        index += 2;
        if bytes.get(index) == Some(&b':') {
            index += 1;
            second = date_parse_two_digits(bytes, index).unwrap_or(u32::MAX);
            index += 2;
            if bytes.get(index) == Some(&b'.') {
                index += 1;
                let mut scale = 100;
                while let Some(byte) = bytes.get(index).copied() {
                    if !byte.is_ascii_digit() {
                        break;
                    }
                    if scale > 0 {
                        millisecond += u32::from(byte - b'0') * scale;
                        scale /= 10;
                    }
                    index += 1;
                }
            }
        }
        if hour > 24
            || minute > 59
            || second > 59
            || (hour == 24 && (minute != 0 || second != 0 || millisecond != 0))
        {
            return Ok(None);
        }
        match bytes.get(index).copied() {
            Some(b'Z') => {
                offset_minutes = Some(0);
                index += 1;
            }
            Some(b'+' | b'-') => {
                let offset_text = &text[index..];
                offset_minutes = date_parse_timezone_offset_colon(offset_text);
                if offset_minutes.is_none() {
                    return Ok(None);
                }
                index = text.len();
            }
            _ => {}
        }
    }
    if index != text.len() {
        return Ok(None);
    }

    let value = if let Some(offset) = offset_minutes {
        let utc = date_make_utc_value(
            f64::from(year),
            f64::from(month - 1),
            f64::from(day),
            f64::from(hour),
            f64::from(minute),
            f64::from(second),
            f64::from(millisecond),
        );
        let Some(millis) = utc.as_f64().filter(|millis| millis.is_finite()) else {
            return Ok(Some(Value::from_f64(f64::NAN)));
        };
        date_time_clip_value(millis - f64::from(offset) * DATE_MS_PER_MINUTE as f64)
    } else if date_only {
        date_make_utc_value(
            f64::from(year),
            f64::from(month - 1),
            f64::from(day),
            0.0,
            0.0,
            0.0,
            0.0,
        )
    } else {
        date_make_local_value(
            cx,
            f64::from(year),
            f64::from(month - 1),
            f64::from(day),
            f64::from(hour),
            f64::from(minute),
            f64::from(second),
            f64::from(millisecond),
        )?
    };
    Ok(Some(value))
}

fn date_parse_utc_string(text: &str) -> Option<Value> {
    let parts: Vec<_> = text.split_whitespace().collect();
    if parts.len() != 6 || parts[5] != "GMT" {
        return None;
    }
    let day = parts[1].parse::<u32>().ok()?;
    let month = date_month_name_index(parts[2])?;
    let year = parts[3].parse::<i32>().ok()?;
    let (hour, minute, second) = date_parse_time(parts[4])?;
    Some(date_make_utc_value(
        f64::from(year),
        f64::from(month - 1),
        f64::from(day),
        f64::from(hour),
        f64::from(minute),
        f64::from(second),
        0.0,
    ))
}

fn date_parse_local_string(text: &str) -> Option<Value> {
    let parts: Vec<_> = text.split_whitespace().collect();
    if parts.len() < 6 || !parts[5].starts_with("GMT") {
        return None;
    }
    let month = date_month_name_index(parts[1])?;
    let day = parts[2].parse::<u32>().ok()?;
    let year = parts[3].parse::<i32>().ok()?;
    let (hour, minute, second) = date_parse_time(parts[4])?;
    let offset = date_parse_timezone_offset_compact(&parts[5][3..])?;
    let utc = date_make_utc_value(
        f64::from(year),
        f64::from(month - 1),
        f64::from(day),
        f64::from(hour),
        f64::from(minute),
        f64::from(second),
        0.0,
    );
    let millis = utc.as_f64().filter(|millis| millis.is_finite())?;
    Some(date_time_clip_value(
        millis - f64::from(offset) * DATE_MS_PER_MINUTE as f64,
    ))
}

pub(super) fn date_parse_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    text: &str,
) -> Result<Value, Cx::Error> {
    if let Some(value) = date_parse_iso_text(cx, text)? {
        return Ok(value);
    }
    if let Some(value) = date_parse_utc_string(text) {
        return Ok(value);
    }
    if let Some(value) = date_parse_local_string(text) {
        return Ok(value);
    }
    Ok(Value::from_f64(f64::NAN))
}

use super::{
    instant_epoch_nanoseconds_is_valid, iso_day_of_year, iso_days_before_year, iso_days_in_month,
    NANOS_PER_DAY, NANOS_PER_HOUR, NANOS_PER_MINUTE, NANOS_PER_SECOND,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedPlainDateTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub fraction_nanoseconds: i128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CalendarAnnotationPolicy {
    RequireIso8601,
    IgnoreNonCritical,
}

pub fn parse_instant(text: &str) -> Option<i128> {
    let text = strip_temporal_annotations(text, CalendarAnnotationPolicy::IgnoreNonCritical)?;
    let bytes = text.as_bytes();
    let mut index = 0;
    let year = parse_iso_year(text, &mut index)?;
    let month = if matches!(bytes.get(index).copied(), Some(b'-')) {
        index += 1;
        let month = parse_fixed_u8(text, &mut index, 2)?;
        expect_byte(bytes, &mut index, b'-')?;
        month
    } else {
        parse_fixed_u8(text, &mut index, 2)?
    };
    let day = parse_fixed_u8(text, &mut index, 2)?;
    if day == 0 || day > iso_days_in_month(year, month) {
        return None;
    }

    match bytes.get(index).copied()? {
        b'T' | b't' | b' ' => index += 1,
        _ => return None,
    }
    let (hour, minute, second, fraction_nanoseconds) = parse_instant_time(text, &mut index)?;

    let offset_nanoseconds = parse_instant_offset_nanoseconds(text, &mut index)?;
    if index != bytes.len() {
        return None;
    }

    let days = i128::from(iso_days_before_year(year))
        .checked_add(i128::from(iso_day_of_year(year, month, day)))?
        .checked_sub(1)?
        .checked_sub(i128::from(iso_days_before_year(1970)))?;
    let date_nanoseconds = days.checked_mul(NANOS_PER_DAY)?;
    let time_nanoseconds = i128::from(hour)
        .checked_mul(NANOS_PER_HOUR)?
        .checked_add(i128::from(minute).checked_mul(NANOS_PER_MINUTE)?)?
        .checked_add(i128::from(second).checked_mul(NANOS_PER_SECOND)?)?
        .checked_add(fraction_nanoseconds)?;
    let epoch_nanoseconds = date_nanoseconds
        .checked_add(time_nanoseconds)?
        .checked_sub(offset_nanoseconds)?;
    instant_epoch_nanoseconds_is_valid(epoch_nanoseconds).then_some(epoch_nanoseconds)
}

pub fn parse_plain_date(text: &str) -> Option<(i32, u8, u8)> {
    let date_time = parse_plain_date_time(text)?;
    Some((date_time.year, date_time.month, date_time.day))
}

pub fn parse_plain_date_time(text: &str) -> Option<ParsedPlainDateTime> {
    let text = strip_temporal_annotations(text, CalendarAnnotationPolicy::RequireIso8601)?;
    let bytes = text.as_bytes();
    let mut index = 0;
    let (year, month, day) = parse_iso_date(text, bytes, &mut index)?;
    let (hour, minute, second, fraction_nanoseconds) = match bytes.get(index).copied() {
        None => (0, 0, 0, 0),
        Some(b'T' | b't' | b' ') => {
            index += 1;
            parse_instant_time(text, &mut index)?
        }
        _ => return None,
    };
    parse_plain_offset(text, bytes, &mut index)?;
    if index != bytes.len() {
        return None;
    }
    Some(ParsedPlainDateTime {
        year,
        month,
        day,
        hour,
        minute,
        second,
        fraction_nanoseconds,
    })
}

pub fn parse_plain_time(text: &str) -> Option<(u8, u8, u8, i128)> {
    let text = strip_temporal_annotations(text, CalendarAnnotationPolicy::RequireIso8601)?;
    let bytes = text.as_bytes();
    let mut index = 0;

    if matches!(bytes.get(index).copied(), Some(b'T' | b't')) {
        index += 1;
    } else if parse_prefixed_plain_time_date(text, bytes, &mut index).is_none() {
        if plain_time_requires_designator(text) {
            return None;
        }
        index = 0;
    }

    let time = parse_instant_time(text, &mut index)?;
    parse_plain_offset(text, bytes, &mut index)?;
    if index != bytes.len() {
        return None;
    }
    Some(time)
}

fn plain_time_requires_designator(text: &str) -> bool {
    let bytes = text.as_bytes();
    match bytes.len() {
        4 if ascii_digits(bytes) => valid_iso_month_day(bytes[0], bytes[1], bytes[2], bytes[3]),
        5 if bytes.get(2).copied() == Some(b'-') => {
            valid_iso_month_day(bytes[0], bytes[1], bytes[3], bytes[4])
        }
        6 if ascii_digits(bytes) => {
            valid_iso_year_month(bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5])
        }
        7 if bytes.get(4).copied() == Some(b'-') => {
            valid_iso_year_month(bytes[0], bytes[1], bytes[2], bytes[3], bytes[5], bytes[6])
        }
        _ => false,
    }
}

fn ascii_digits(bytes: &[u8]) -> bool {
    bytes.iter().all(std::primitive::u8::is_ascii_digit)
}

fn decimal_digit(byte: u8) -> Option<u8> {
    byte.is_ascii_digit().then_some(byte - b'0')
}

fn valid_iso_month_day(month_tens: u8, month_ones: u8, day_tens: u8, day_ones: u8) -> bool {
    let Some(month) = decimal_digit(month_tens)
        .and_then(|tens| decimal_digit(month_ones).map(|ones| tens * 10 + ones))
    else {
        return false;
    };
    let Some(day) = decimal_digit(day_tens)
        .and_then(|tens| decimal_digit(day_ones).map(|ones| tens * 10 + ones))
    else {
        return false;
    };
    month != 0 && day != 0 && day <= iso_days_in_month(1972, month)
}

fn valid_iso_year_month(
    _year_thousands: u8,
    _year_hundreds: u8,
    _year_tens: u8,
    _year_ones: u8,
    month_tens: u8,
    month_ones: u8,
) -> bool {
    let Some(month) = decimal_digit(month_tens)
        .and_then(|tens| decimal_digit(month_ones).map(|ones| tens * 10 + ones))
    else {
        return false;
    };
    (1..=12).contains(&month)
}

pub fn parse_plain_year_month(text: &str) -> Option<(i32, u8, u8)> {
    let text = strip_temporal_annotations(text, CalendarAnnotationPolicy::RequireIso8601)?;
    let bytes = text.as_bytes();
    let mut index = 0;
    let mut saw_time = false;
    let year = parse_iso_year(text, &mut index)?;
    let month = if matches!(bytes.get(index).copied(), Some(b'-')) {
        index += 1;
        parse_fixed_u8(text, &mut index, 2)?
    } else {
        parse_fixed_u8(text, &mut index, 2)?
    };
    let day = if matches!(bytes.get(index).copied(), Some(b'-')) {
        index += 1;
        let day = parse_fixed_u8(text, &mut index, 2)?;
        if day == 0 || day > iso_days_in_month(year, month) {
            return None;
        }
        day
    } else {
        1
    };
    if matches!(bytes.get(index).copied(), Some(b'T' | b't' | b' ')) {
        index += 1;
        let _ = parse_instant_time(text, &mut index)?;
        saw_time = true;
    }
    if !saw_time && matches!(bytes.get(index).copied(), Some(b'+' | b'-' | b'Z' | b'z')) {
        return None;
    }
    parse_plain_offset(text, bytes, &mut index)?;
    if index != bytes.len() || month == 0 || month > 12 {
        return None;
    }
    Some((year, month, day))
}

pub fn parse_plain_month_day(text: &str) -> Option<(u8, u8, i32)> {
    let text = strip_temporal_annotations(text, CalendarAnnotationPolicy::RequireIso8601)?;
    let bytes = text.as_bytes();
    let mut index = 0;
    let mut reference_year = 1972_i32;
    let mut saw_time = false;
    let (month, day) = if matches!(bytes.get(index).copied(), Some(b'-'))
        && matches!(bytes.get(index + 1).copied(), Some(b'-'))
    {
        index += 2;
        let month = parse_fixed_u8(text, &mut index, 2)?;
        if matches!(bytes.get(index).copied(), Some(b'-')) {
            index += 1;
        }
        let day = parse_fixed_u8(text, &mut index, 2)?;
        (month, day)
    } else {
        let full_date_saved = index;
        if let Some((year, month, day)) = parse_iso_date(text, bytes, &mut index) {
            reference_year = year;
            (month, day)
        } else {
            index = full_date_saved;
            let month = parse_fixed_u8(text, &mut index, 2)?;
            if matches!(bytes.get(index).copied(), Some(b'-')) {
                index += 1;
            }
            let day = parse_fixed_u8(text, &mut index, 2)?;
            (month, day)
        }
    };
    if matches!(bytes.get(index).copied(), Some(b'T' | b't' | b' ')) {
        index += 1;
        let _ = parse_instant_time(text, &mut index)?;
        saw_time = true;
    }
    if !saw_time && matches!(bytes.get(index).copied(), Some(b'+' | b'-' | b'Z' | b'z')) {
        return None;
    }
    parse_plain_offset(text, bytes, &mut index)?;
    if index != bytes.len()
        || month == 0
        || month > 12
        || day == 0
        || day > iso_days_in_month(reference_year, month)
    {
        return None;
    }
    Some((month, day, reference_year))
}

fn parse_iso_date(text: &str, bytes: &[u8], index: &mut usize) -> Option<(i32, u8, u8)> {
    let year = parse_iso_year(text, index)?;
    let month = if matches!(bytes.get(*index).copied(), Some(b'-')) {
        *index += 1;
        let month = parse_fixed_u8(text, index, 2)?;
        expect_byte(bytes, index, b'-')?;
        month
    } else {
        parse_fixed_u8(text, index, 2)?
    };
    let day = parse_fixed_u8(text, index, 2)?;
    if day == 0 || day > iso_days_in_month(year, month) {
        return None;
    }
    Some((year, month, day))
}

fn parse_plain_offset(text: &str, bytes: &[u8], index: &mut usize) -> Option<()> {
    match bytes.get(*index).copied() {
        Some(b'+' | b'-') => {
            let _ = parse_instant_offset_nanoseconds(text, index)?;
            Some(())
        }
        Some(b'Z' | b'z') => None,
        _ => Some(()),
    }
}

fn parse_prefixed_plain_time_date(text: &str, bytes: &[u8], index: &mut usize) -> Option<()> {
    let saved = *index;
    let _ = parse_iso_date(text, bytes, index)?;
    if !matches!(bytes.get(*index).copied(), Some(b'T' | b't' | b' ')) {
        *index = saved;
        return None;
    }
    *index += 1;
    Some(())
}

fn strip_temporal_annotations(
    text: &str,
    calendar_policy: CalendarAnnotationPolicy,
) -> Option<&str> {
    let Some(annotation_start) = text.find('[') else {
        return Some(text);
    };
    let mut index = annotation_start;
    let bytes = text.as_bytes();
    let mut seen_time_zone_annotation = false;
    let mut calendar_annotation_count = 0_u8;
    let mut critical_calendar_annotation_seen = false;
    while index < bytes.len() {
        if bytes[index] != b'[' {
            return None;
        }
        index += 1;
        let item_start = index;
        while index < bytes.len() && bytes[index] != b']' {
            if bytes[index] == b'[' {
                return None;
            }
            index += 1;
        }
        if index == bytes.len() || index == item_start {
            return None;
        }
        validate_temporal_annotation(
            &text[item_start..index],
            calendar_policy,
            &mut seen_time_zone_annotation,
            &mut calendar_annotation_count,
            &mut critical_calendar_annotation_seen,
        )?;
        index += 1;
    }
    Some(&text[..annotation_start])
}

fn validate_temporal_annotation(
    item: &str,
    calendar_policy: CalendarAnnotationPolicy,
    seen_time_zone_annotation: &mut bool,
    calendar_annotation_count: &mut u8,
    critical_calendar_annotation_seen: &mut bool,
) -> Option<()> {
    let (critical, body) = if let Some(body) = item.strip_prefix('!') {
        (true, body)
    } else {
        (false, item)
    };
    if body.is_empty() {
        return None;
    }
    let Some(key_end) = body.find('=') else {
        if *seen_time_zone_annotation {
            return None;
        }
        validate_time_zone_annotation(body)?;
        *seen_time_zone_annotation = true;
        return Some(());
    };
    let key = &body[..key_end];
    let value = &body[key_end + 1..];
    if key.is_empty()
        || value.is_empty()
        || !key.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
    {
        return None;
    }
    if key != "u-ca" {
        return (!critical).then_some(());
    }
    if *calendar_annotation_count > 0 {
        if critical || *critical_calendar_annotation_seen {
            return None;
        }
        *calendar_annotation_count = calendar_annotation_count.checked_add(1)?;
        return Some(());
    }
    let accepts_value = match calendar_policy {
        CalendarAnnotationPolicy::RequireIso8601 => value.eq_ignore_ascii_case("iso8601"),
        CalendarAnnotationPolicy::IgnoreNonCritical => {
            !critical || value.eq_ignore_ascii_case("iso8601")
        }
    };
    if !accepts_value {
        return None;
    }
    *calendar_annotation_count = calendar_annotation_count.checked_add(1)?;
    *critical_calendar_annotation_seen |= critical;
    Some(())
}

fn validate_time_zone_annotation(body: &str) -> Option<()> {
    if !matches!(body.as_bytes().first().copied(), Some(b'+' | b'-')) {
        return Some(());
    }
    let bytes = body.as_bytes();
    let mut index = 1;
    let hour = parse_fixed_u8(body, &mut index, 2)?;
    let mut minute = 0_u8;
    let mut second_present = false;
    if matches!(bytes.get(index).copied(), Some(b':')) {
        index += 1;
        minute = parse_fixed_u8(body, &mut index, 2)?;
        if matches!(bytes.get(index).copied(), Some(b':')) {
            second_present = true;
        }
    } else if bytes
        .get(index)
        .is_some_and(std::primitive::u8::is_ascii_digit)
    {
        minute = parse_fixed_u8(body, &mut index, 2)?;
        if bytes
            .get(index)
            .is_some_and(std::primitive::u8::is_ascii_digit)
        {
            second_present = true;
        }
    }
    if hour > 23 || minute > 59 || second_present || index != bytes.len() {
        return None;
    }
    Some(())
}

fn parse_iso_year(text: &str, index: &mut usize) -> Option<i32> {
    let bytes = text.as_bytes();
    let (sign, has_explicit_sign) = match bytes.get(*index).copied() {
        Some(b'-') => {
            *index += 1;
            (-1_i64, true)
        }
        Some(b'+') => {
            *index += 1;
            (1_i64, true)
        }
        _ => (1_i64, false),
    };
    let digits = if has_explicit_sign { 6 } else { 4 };
    let unsigned_year = parse_fixed_i64(text, index, digits)?;
    if sign < 0 && unsigned_year == 0 {
        return None;
    }
    let year = unsigned_year.checked_mul(sign)?;
    i32::try_from(year).ok()
}

fn parse_fixed_u8(text: &str, index: &mut usize, digits: usize) -> Option<u8> {
    u8::try_from(parse_fixed_i64(text, index, digits)?).ok()
}

fn parse_fixed_i64(text: &str, index: &mut usize, digits: usize) -> Option<i64> {
    let bytes = text.as_bytes();
    let end = index.checked_add(digits)?;
    if end > bytes.len() {
        return None;
    }
    let mut value = 0_i64;
    for digit in &bytes[*index..end] {
        let digit = i64::from(digit.checked_sub(b'0')?);
        if digit > 9 {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(digit)?;
    }
    *index = end;
    Some(value)
}

fn expect_byte(bytes: &[u8], index: &mut usize, expected: u8) -> Option<()> {
    if bytes.get(*index).copied()? != expected {
        return None;
    }
    *index += 1;
    Some(())
}

fn parse_nanosecond_fraction(text: &str, index: &mut usize) -> Option<i128> {
    let bytes = text.as_bytes();
    let start = *index;
    let mut digits = 0_usize;
    let mut nanoseconds = 0_i128;
    while *index < bytes.len() && bytes[*index].is_ascii_digit() {
        if digits < 9 {
            nanoseconds = nanoseconds
                .checked_mul(10)?
                .checked_add(i128::from(bytes[*index] - b'0'))?;
        }
        digits += 1;
        *index += 1;
    }
    if *index == start || digits > 9 {
        return None;
    }
    for _ in digits..9 {
        nanoseconds = nanoseconds.checked_mul(10)?;
    }
    Some(nanoseconds)
}

fn parse_instant_time(text: &str, index: &mut usize) -> Option<(u8, u8, u8, i128)> {
    let bytes = text.as_bytes();
    let hour = parse_fixed_u8(text, index, 2)?;
    let mut minute = 0_u8;
    let mut second = 0_u8;
    let mut fraction_nanoseconds = 0_i128;
    let mut second_present = false;

    if matches!(bytes.get(*index).copied(), Some(b':')) {
        *index += 1;
        minute = parse_fixed_u8(text, index, 2)?;
        if matches!(bytes.get(*index).copied(), Some(b':')) {
            *index += 1;
            second = parse_fixed_u8(text, index, 2)?;
            second_present = true;
        }
    } else if bytes
        .get(*index)
        .is_some_and(std::primitive::u8::is_ascii_digit)
    {
        minute = parse_fixed_u8(text, index, 2)?;
        if bytes
            .get(*index)
            .is_some_and(std::primitive::u8::is_ascii_digit)
        {
            second = parse_fixed_u8(text, index, 2)?;
            second_present = true;
        }
    }

    if matches!(bytes.get(*index).copied(), Some(b'.' | b',')) {
        if !second_present {
            return None;
        }
        *index += 1;
        fraction_nanoseconds = parse_nanosecond_fraction(text, index)?;
    }
    if hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    if second == 60 {
        second = 59;
    }
    Some((hour, minute, second, fraction_nanoseconds))
}

fn parse_instant_offset_nanoseconds(text: &str, index: &mut usize) -> Option<i128> {
    let bytes = text.as_bytes();
    match bytes.get(*index).copied()? {
        b'Z' | b'z' => {
            *index += 1;
            return Some(0);
        }
        _ => {}
    }
    let sign = match bytes.get(*index).copied()? {
        b'+' => 1_i128,
        b'-' => -1_i128,
        _ => return None,
    };
    *index += 1;
    let hour = parse_fixed_u8(text, index, 2)?;
    let mut minute = 0_u8;
    let mut second = 0_u8;
    let mut fraction_nanoseconds = 0_i128;
    let mut second_present = false;
    if matches!(bytes.get(*index).copied(), Some(b':')) {
        *index += 1;
        minute = parse_fixed_u8(text, index, 2)?;
        if matches!(bytes.get(*index).copied(), Some(b':')) {
            *index += 1;
            second = parse_fixed_u8(text, index, 2)?;
            second_present = true;
        }
    } else if bytes
        .get(*index)
        .is_some_and(std::primitive::u8::is_ascii_digit)
    {
        minute = parse_fixed_u8(text, index, 2)?;
        if bytes
            .get(*index)
            .is_some_and(std::primitive::u8::is_ascii_digit)
        {
            second = parse_fixed_u8(text, index, 2)?;
            second_present = true;
        }
    }
    if matches!(bytes.get(*index).copied(), Some(b'.' | b',')) {
        if !second_present {
            return None;
        }
        *index += 1;
        fraction_nanoseconds = parse_nanosecond_fraction(text, index)?;
    }
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let offset = i128::from(hour)
        .checked_mul(NANOS_PER_HOUR)?
        .checked_add(i128::from(minute).checked_mul(NANOS_PER_MINUTE)?)?
        .checked_add(i128::from(second).checked_mul(NANOS_PER_SECOND)?)?
        .checked_add(fraction_nanoseconds)?;
    offset.checked_mul(sign)
}

#[cfg(test)]
mod tests {
    use super::{parse_instant, parse_plain_date};

    #[test]
    fn parse_instant_ignores_noncritical_calendar_annotations() {
        assert_eq!(
            parse_instant("1976-11-18T15:23:30.123456789Z[u-ca=discord]"),
            Some(217_178_610_123_456_789)
        );
    }

    #[test]
    fn parse_plain_date_rejects_invalid_calendar_annotations() {
        assert_eq!(parse_plain_date("2020-01-01[u-ca=notexist]"), None);
    }
}

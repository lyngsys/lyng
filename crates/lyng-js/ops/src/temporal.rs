//! Engine-owned Temporal semantic helpers.
//!
//! This module intentionally stays below the builtin dispatch layer: callers
//! perform JS conversion, host interaction, and error allocation, while these
//! helpers own reusable Temporal arithmetic and formatting rules.

use lyng_js_objects::{
    TemporalDurationObjectData, TemporalPlainDateObjectData, TemporalPlainDateTimeObjectData,
    TemporalPlainMonthDayObjectData, TemporalPlainTimeObjectData, TemporalPlainYearMonthObjectData,
};
use std::fmt::Write as _;

mod parse;

pub use parse::{
    parse_instant, parse_plain_date, parse_plain_date_time, parse_plain_month_day,
    parse_plain_time, parse_plain_year_month, ParsedPlainDateTime,
};

pub const SAFE_INTEGER_MAX: i128 = 9_007_199_254_740_991;
pub const NANOS_PER_SECOND: i128 = 1_000_000_000;
pub const NANOS_PER_MILLISECOND: i128 = 1_000_000;
pub const NANOS_PER_MICROSECOND: i128 = 1_000;
pub const NANOS_PER_MINUTE: i128 = 60 * NANOS_PER_SECOND;
pub const NANOS_PER_HOUR: i128 = 60 * NANOS_PER_MINUTE;
pub const NANOS_PER_DAY: i128 = 24 * NANOS_PER_HOUR;
pub const INSTANT_EPOCH_NANOSECONDS_MAX: i128 = 8_640_000_000_000_000_000_000;
pub const INSTANT_EPOCH_MILLISECONDS_MAX: i128 =
    INSTANT_EPOCH_NANOSECONDS_MAX / NANOS_PER_MILLISECOND;
pub const UTC_TIME_ZONE_ID: &str = "UTC";
pub const DEFAULT_PLAIN_MONTH_DAY_REFERENCE_YEAR: i64 = 1972;
const DURATION_DATE_UNIT_MAX: i128 = 4_294_967_295;
const DURATION_TIME_NANOS_MAX: i128 = SAFE_INTEGER_MAX * NANOS_PER_SECOND + NANOS_PER_SECOND - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalRoundingMode {
    Ceil,
    Floor,
    Expand,
    Trunc,
    HalfCeil,
    HalfFloor,
    HalfExpand,
    HalfTrunc,
    HalfEven,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalDurationExactUnit {
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
}

impl TemporalDurationExactUnit {
    const fn order(self) -> u8 {
        match self {
            Self::Day => 0,
            Self::Hour => 1,
            Self::Minute => 2,
            Self::Second => 3,
            Self::Millisecond => 4,
            Self::Microsecond => 5,
            Self::Nanosecond => 6,
        }
    }

    const fn nanoseconds(self) -> i128 {
        match self {
            Self::Day => NANOS_PER_DAY,
            Self::Hour => NANOS_PER_HOUR,
            Self::Minute => NANOS_PER_MINUTE,
            Self::Second => NANOS_PER_SECOND,
            Self::Millisecond => NANOS_PER_MILLISECOND,
            Self::Microsecond => NANOS_PER_MICROSECOND,
            Self::Nanosecond => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DurationLargestTimeUnit {
    Day,
    Hour,
    Minute,
    Second,
}

pub const fn duration_components(data: TemporalDurationObjectData) -> [i128; 10] {
    [
        data.years(),
        data.months(),
        data.weeks(),
        data.days(),
        data.hours(),
        data.minutes(),
        data.seconds(),
        data.milliseconds(),
        data.microseconds(),
        data.nanoseconds(),
    ]
}

pub fn instant_epoch_nanoseconds_is_valid(epoch_nanoseconds: i128) -> bool {
    (-INSTANT_EPOCH_NANOSECONDS_MAX..=INSTANT_EPOCH_NANOSECONDS_MAX).contains(&epoch_nanoseconds)
}

pub fn duration_sign(data: TemporalDurationObjectData) -> i32 {
    for component in duration_components(data) {
        match component.cmp(&0) {
            std::cmp::Ordering::Less => return -1,
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => return 1,
        }
    }
    0
}

pub fn duration_signs_are_balanced(data: TemporalDurationObjectData) -> bool {
    let mut sign = 0_i32;
    for component in duration_components(data) {
        let component_sign = component.signum() as i32;
        if component_sign == 0 {
            continue;
        }
        if sign == 0 {
            sign = component_sign;
            continue;
        }
        if sign != component_sign {
            return false;
        }
    }
    true
}

pub fn duration_is_within_limits(data: TemporalDurationObjectData) -> bool {
    for component in [data.years(), data.months(), data.weeks()] {
        if component.abs() > DURATION_DATE_UNIT_MAX {
            return false;
        }
    }
    let Some(time_nanoseconds) = duration_total_time_nanoseconds(data) else {
        return false;
    };
    time_nanoseconds.abs() <= DURATION_TIME_NANOS_MAX
}

pub fn durations_are_equal(
    left: TemporalDurationObjectData,
    right: TemporalDurationObjectData,
) -> bool {
    duration_components(left) == duration_components(right)
}

pub const fn duration_has_calendar_relative_units(data: TemporalDurationObjectData) -> bool {
    data.years() != 0 || data.months() != 0 || data.weeks() != 0
}

pub const fn duration_has_date_units(data: TemporalDurationObjectData) -> bool {
    data.years() != 0 || data.months() != 0 || data.weeks() != 0 || data.days() != 0
}

pub const fn duration_calendar_relative_components_are_equal(
    left: TemporalDurationObjectData,
    right: TemporalDurationObjectData,
) -> bool {
    left.years() == right.years()
        && left.months() == right.months()
        && left.weeks() == right.weeks()
}

pub fn compare_time_duration(
    left: TemporalDurationObjectData,
    right: TemporalDurationObjectData,
) -> Option<std::cmp::Ordering> {
    let left = duration_total_time_nanoseconds(left)?;
    let right = duration_total_time_nanoseconds(right)?;
    Some(left.cmp(&right))
}

pub const fn duration_default_largest_exact_unit(
    data: TemporalDurationObjectData,
    smallest_unit: TemporalDurationExactUnit,
) -> TemporalDurationExactUnit {
    let largest_present = if data.days() != 0 {
        TemporalDurationExactUnit::Day
    } else if data.hours() != 0 {
        TemporalDurationExactUnit::Hour
    } else if data.minutes() != 0 {
        TemporalDurationExactUnit::Minute
    } else if data.seconds() != 0 {
        TemporalDurationExactUnit::Second
    } else if data.milliseconds() != 0 {
        TemporalDurationExactUnit::Millisecond
    } else if data.microseconds() != 0 {
        TemporalDurationExactUnit::Microsecond
    } else if data.nanoseconds() != 0 {
        TemporalDurationExactUnit::Nanosecond
    } else {
        smallest_unit
    };
    if largest_present.order() <= smallest_unit.order() {
        largest_present
    } else {
        smallest_unit
    }
}

pub const fn duration_exact_unit_allows_largest_smallest(
    largest_unit: TemporalDurationExactUnit,
    smallest_unit: TemporalDurationExactUnit,
) -> bool {
    largest_unit.order() <= smallest_unit.order()
}

fn duration_total_time_nanoseconds(data: TemporalDurationObjectData) -> Option<i128> {
    let days = data.days().checked_mul(NANOS_PER_DAY)?;
    let hours = data.hours().checked_mul(NANOS_PER_HOUR)?;
    let minutes = data.minutes().checked_mul(NANOS_PER_MINUTE)?;
    let seconds = data.seconds().checked_mul(NANOS_PER_SECOND)?;
    let milliseconds = data.milliseconds().checked_mul(NANOS_PER_MILLISECOND)?;
    let microseconds = data.microseconds().checked_mul(NANOS_PER_MICROSECOND)?;
    days.checked_add(hours)?
        .checked_add(minutes)?
        .checked_add(seconds)?
        .checked_add(milliseconds)?
        .checked_add(microseconds)?
        .checked_add(data.nanoseconds())
}

pub fn round_duration_exact(
    data: TemporalDurationObjectData,
    largest_unit: TemporalDurationExactUnit,
    smallest_unit: TemporalDurationExactUnit,
    rounding_increment: i128,
    rounding_mode: TemporalRoundingMode,
) -> Option<TemporalDurationObjectData> {
    if duration_has_calendar_relative_units(data)
        || !duration_exact_unit_allows_largest_smallest(largest_unit, smallest_unit)
        || rounding_increment <= 0
    {
        return None;
    }
    let increment = smallest_unit
        .nanoseconds()
        .checked_mul(rounding_increment)?;
    let rounded = round_i128_to_increment(
        duration_total_time_nanoseconds(data)?,
        increment,
        rounding_mode,
    )?;
    let [days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds] =
        distribute_duration_nanoseconds_for_exact_largest_unit(rounded, largest_unit)?;
    Some(TemporalDurationObjectData::new(
        0,
        0,
        0,
        days,
        hours,
        minutes,
        seconds,
        milliseconds,
        microseconds,
        nanoseconds,
    ))
}

pub fn total_duration_exact(
    data: TemporalDurationObjectData,
    unit: TemporalDurationExactUnit,
) -> Option<f64> {
    if duration_has_calendar_relative_units(data) {
        return None;
    }
    total_nanoseconds_as_unit(duration_total_time_nanoseconds(data)?, unit.nanoseconds())
}

pub fn total_nanoseconds_as_unit(total_nanoseconds: i128, unit_nanoseconds: i128) -> Option<f64> {
    if unit_nanoseconds <= 0 {
        return None;
    }
    rational_i128_to_f64(total_nanoseconds, unit_nanoseconds)
}

fn rational_i128_to_f64(numerator: i128, denominator: i128) -> Option<f64> {
    if denominator <= 0 {
        return None;
    }
    let negative = numerator < 0;
    let numerator = numerator.unsigned_abs();
    let denominator = u128::try_from(denominator).ok()?;
    let whole = numerator / denominator;
    let mut remainder = numerator % denominator;
    if remainder == 0 {
        let value = whole.to_string().parse::<f64>().ok()?;
        return Some(if negative { -value } else { value });
    }

    let mut text = whole.to_string();
    text.push('.');
    for _ in 0..80 {
        remainder = remainder.checked_mul(10)?;
        let digit = remainder / denominator;
        let digit = u8::try_from(digit).ok()?;
        text.push(char::from(b'0' + digit));
        remainder %= denominator;
        if remainder == 0 {
            break;
        }
    }
    let value = text.parse::<f64>().ok()?;
    Some(if negative { -value } else { value })
}

pub fn add_durations(
    left: TemporalDurationObjectData,
    right: TemporalDurationObjectData,
) -> Option<TemporalDurationObjectData> {
    add_durations_with_largest_unit(
        left,
        right,
        duration_largest_exact_unit_for_addition(left, duration_largest_exact_unit(right)),
    )
}

pub fn subtract_durations(
    left: TemporalDurationObjectData,
    right: TemporalDurationObjectData,
) -> Option<TemporalDurationObjectData> {
    subtract_durations_with_largest_unit(
        left,
        right,
        duration_largest_exact_unit_for_addition(left, duration_largest_exact_unit(right)),
    )
}

pub fn add_durations_with_largest_unit(
    left: TemporalDurationObjectData,
    right: TemporalDurationObjectData,
    largest_unit: TemporalDurationExactUnit,
) -> Option<TemporalDurationObjectData> {
    combine_durations(left, right, 1, largest_unit)
}

pub fn subtract_durations_with_largest_unit(
    left: TemporalDurationObjectData,
    right: TemporalDurationObjectData,
    largest_unit: TemporalDurationExactUnit,
) -> Option<TemporalDurationObjectData> {
    combine_durations(left, right, -1, largest_unit)
}

pub fn balance_duration_subsecond_fields(
    seconds: i128,
    milliseconds: i128,
    microseconds: i128,
    nanoseconds: i128,
) -> Option<[i64; 4]> {
    let total_nanoseconds = seconds
        .checked_mul(NANOS_PER_SECOND)?
        .checked_add(milliseconds.checked_mul(NANOS_PER_MILLISECOND)?)?
        .checked_add(microseconds.checked_mul(NANOS_PER_MICROSECOND)?)?
        .checked_add(nanoseconds)?;
    let seconds = total_nanoseconds / NANOS_PER_SECOND;
    let remainder = total_nanoseconds % NANOS_PER_SECOND;
    let milliseconds = remainder / NANOS_PER_MILLISECOND;
    let remainder = remainder % NANOS_PER_MILLISECOND;
    let microseconds = remainder / NANOS_PER_MICROSECOND;
    let nanoseconds = remainder % NANOS_PER_MICROSECOND;
    Some([
        i64::try_from(seconds).ok()?,
        i64::try_from(milliseconds).ok()?,
        i64::try_from(microseconds).ok()?,
        i64::try_from(nanoseconds).ok()?,
    ])
}

fn combine_durations(
    left: TemporalDurationObjectData,
    right: TemporalDurationObjectData,
    right_sign: i128,
    largest_unit: TemporalDurationExactUnit,
) -> Option<TemporalDurationObjectData> {
    let years = combine_component(left.years(), right.years(), right_sign)?;
    let months = combine_component(left.months(), right.months(), right_sign)?;
    let weeks = combine_component(left.weeks(), right.weeks(), right_sign)?;
    let day_time_nanoseconds = combine_component(left.days(), right.days(), right_sign)?
        .checked_mul(NANOS_PER_DAY)?
        .checked_add(duration_time_nanoseconds(left))?
        .checked_add(duration_time_nanoseconds(right).checked_mul(right_sign)?)?;
    let [days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds] =
        distribute_additive_duration_nanoseconds(day_time_nanoseconds, largest_unit)?;
    Some(TemporalDurationObjectData::new(
        years,
        months,
        weeks,
        days,
        hours,
        minutes,
        seconds,
        milliseconds,
        microseconds,
        nanoseconds,
    ))
}

fn combine_component(left: i128, right: i128, right_sign: i128) -> Option<i128> {
    let value = left.checked_add(right.checked_mul(right_sign)?)?;
    duration_component_from_integer(value)
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "Temporal Duration fields are stored as float64-representable integer values."
)]
const fn duration_component_from_integer(value: i128) -> Option<i128> {
    Some((value as f64) as i128)
}

pub fn parse_duration(text: &str) -> Option<TemporalDurationObjectData> {
    let bytes = text.as_bytes();
    let mut index = 0;
    let sign = match bytes.first().copied() {
        Some(b'-') => {
            index += 1;
            -1_i64
        }
        Some(b'+') => {
            index += 1;
            1_i64
        }
        _ => 1_i64,
    };

    if !matches!(bytes.get(index).copied(), Some(b'P' | b'p')) {
        return None;
    }
    index += 1;
    if index == bytes.len() {
        return None;
    }

    let mut years = 0_i64;
    let mut months = 0_i64;
    let mut weeks = 0_i64;
    let mut days = 0_i64;
    let mut hours = 0_i64;
    let mut minutes = 0_i64;
    let mut seconds = 0_i64;
    let mut milliseconds = 0_i64;
    let mut microseconds = 0_i64;
    let mut nanoseconds = 0_i64;
    let mut in_time = false;
    let mut seen = false;
    let mut last_date_order = 0_u8;
    let mut last_time_order = 0_u8;

    while index < bytes.len() {
        if matches!(bytes[index], b'T' | b't') && !in_time {
            in_time = true;
            index += 1;
            if index == bytes.len() {
                return None;
            }
            continue;
        }

        let start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if start == index {
            return None;
        }
        let whole = parse_decimal_i64(&text[start..index])?;

        let fraction_start = if index < bytes.len() && matches!(bytes[index], b'.' | b',') {
            index += 1;
            let fraction_start = index;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }
            if fraction_start == index || index - fraction_start > 9 {
                return None;
            }
            Some(fraction_start)
        } else {
            None
        };

        let unit = bytes.get(index).copied()?.to_ascii_uppercase();
        index += 1;

        if in_time {
            let order = match unit {
                b'H' => 1,
                b'M' => 2,
                b'S' => 3,
                _ => return None,
            };
            if order <= last_time_order {
                return None;
            }
            last_time_order = order;
            if let Some(fraction_start) = fraction_start {
                if index != bytes.len() {
                    return None;
                }
                let fraction = fraction_nanoseconds(
                    &text[fraction_start..index - 1],
                    match unit {
                        b'H' => NANOS_PER_HOUR,
                        b'M' => NANOS_PER_MINUTE,
                        b'S' => NANOS_PER_SECOND,
                        _ => return None,
                    },
                )?;
                match unit {
                    b'H' => {
                        hours = whole;
                        let remainder = distribute_nanoseconds(fraction)?;
                        minutes = remainder[0];
                        seconds = remainder[1];
                        milliseconds = remainder[2];
                        microseconds = remainder[3];
                        nanoseconds = remainder[4];
                    }
                    b'M' => {
                        minutes = whole;
                        let remainder = distribute_nanoseconds(fraction)?;
                        seconds = remainder[1];
                        milliseconds = remainder[2];
                        microseconds = remainder[3];
                        nanoseconds = remainder[4];
                    }
                    b'S' => {
                        seconds = whole;
                        let remainder = distribute_nanoseconds(fraction)?;
                        milliseconds = remainder[2];
                        microseconds = remainder[3];
                        nanoseconds = remainder[4];
                    }
                    _ => return None,
                }
            } else {
                match unit {
                    b'H' => hours = whole,
                    b'M' => minutes = whole,
                    b'S' => seconds = whole,
                    _ => return None,
                }
            }
        } else {
            if fraction_start.is_some() {
                return None;
            }
            let order = match unit {
                b'Y' => 1,
                b'M' => 2,
                b'W' => 3,
                b'D' => 4,
                _ => return None,
            };
            if order <= last_date_order {
                return None;
            }
            last_date_order = order;
            match unit {
                b'Y' => years = whole,
                b'M' => months = whole,
                b'W' => weeks = whole,
                b'D' => days = whole,
                _ => return None,
            }
        }
        seen = true;
    }

    if !seen {
        return None;
    }

    Some(TemporalDurationObjectData::new(
        years.checked_mul(sign)?,
        months.checked_mul(sign)?,
        weeks.checked_mul(sign)?,
        days.checked_mul(sign)?,
        hours.checked_mul(sign)?,
        minutes.checked_mul(sign)?,
        seconds.checked_mul(sign)?,
        milliseconds.checked_mul(sign)?,
        microseconds.checked_mul(sign)?,
        nanoseconds.checked_mul(sign)?,
    ))
}

fn parse_decimal_i64(text: &str) -> Option<i64> {
    let mut value = 0_i64;
    for digit in text.bytes() {
        let digit = i64::from(digit.checked_sub(b'0')?);
        if digit > 9 {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(digit)?;
    }
    Some(value)
}

fn fraction_nanoseconds(text: &str, unit_nanoseconds: i128) -> Option<i128> {
    let fraction = i128::from(parse_decimal_i64(text)?);
    let scale = 10_i128.checked_pow(u32::try_from(text.len()).ok()?)?;
    fraction.checked_mul(unit_nanoseconds)?.checked_div(scale)
}

fn distribute_nanoseconds(nanoseconds: i128) -> Option<[i64; 5]> {
    let minutes = nanoseconds / NANOS_PER_MINUTE;
    let nanoseconds = nanoseconds % NANOS_PER_MINUTE;
    let seconds = nanoseconds / NANOS_PER_SECOND;
    let nanoseconds = nanoseconds % NANOS_PER_SECOND;
    let milliseconds = nanoseconds / NANOS_PER_MILLISECOND;
    let nanoseconds = nanoseconds % NANOS_PER_MILLISECOND;
    let microseconds = nanoseconds / NANOS_PER_MICROSECOND;
    let nanoseconds = nanoseconds % NANOS_PER_MICROSECOND;
    Some([
        i64::try_from(minutes).ok()?,
        i64::try_from(seconds).ok()?,
        i64::try_from(milliseconds).ok()?,
        i64::try_from(microseconds).ok()?,
        i64::try_from(nanoseconds).ok()?,
    ])
}

pub fn format_duration(data: TemporalDurationObjectData) -> String {
    format_duration_parts(data, None)
}

pub fn format_duration_with_seconds_precision(
    data: TemporalDurationObjectData,
    fractional_digits: u8,
    rounding_mode: TemporalRoundingMode,
) -> Option<String> {
    let data = round_duration_to_fractional_digits(data, fractional_digits, rounding_mode)?;
    if !duration_is_within_limits(data) {
        return None;
    }
    Some(format_duration_parts(data, Some(fractional_digits)))
}

fn round_duration_to_fractional_digits(
    data: TemporalDurationObjectData,
    fractional_digits: u8,
    rounding_mode: TemporalRoundingMode,
) -> Option<TemporalDurationObjectData> {
    let increment = 10_i128.pow(u32::from(9 - fractional_digits));
    let day_time_nanoseconds = data
        .days()
        .checked_mul(NANOS_PER_DAY)?
        .checked_add(duration_time_nanoseconds(data))?;
    let rounded = round_i128_to_increment(day_time_nanoseconds, increment, rounding_mode)?;
    let [days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds] =
        distribute_duration_nanoseconds_for_largest_unit(
            rounded,
            duration_largest_time_unit(data),
        )?;
    Some(TemporalDurationObjectData::new(
        data.years(),
        data.months(),
        data.weeks(),
        days,
        hours,
        minutes,
        seconds,
        milliseconds,
        microseconds,
        nanoseconds,
    ))
}

pub fn round_epoch_nanoseconds_to_fractional_digits(
    epoch_nanoseconds: i128,
    fractional_digits: u8,
    rounding_mode: TemporalRoundingMode,
) -> Option<i128> {
    let increment = 10_i128.pow(u32::from(9 - fractional_digits));
    round_epoch_nanoseconds_to_increment(epoch_nanoseconds, increment, rounding_mode)
}

pub fn round_epoch_nanoseconds_to_increment(
    epoch_nanoseconds: i128,
    increment: i128,
    rounding_mode: TemporalRoundingMode,
) -> Option<i128> {
    round_i128_to_increment_as_if_positive(epoch_nanoseconds, increment, rounding_mode)
}

pub fn round_duration_nanoseconds_to_increment(
    duration_nanoseconds: i128,
    increment: i128,
    rounding_mode: TemporalRoundingMode,
) -> Option<i128> {
    round_i128_to_increment(duration_nanoseconds, increment, rounding_mode)
}

fn round_i128_to_increment(
    value: i128,
    increment: i128,
    rounding_mode: TemporalRoundingMode,
) -> Option<i128> {
    let quotient = value.div_euclid(increment);
    let remainder = value.rem_euclid(increment);
    let lower = quotient.checked_mul(increment)?;
    if remainder == 0 {
        return Some(lower);
    }
    let upper = quotient.checked_add(1)?.checked_mul(increment)?;
    let rounded = match rounding_mode {
        TemporalRoundingMode::Ceil => upper,
        TemporalRoundingMode::Floor => lower,
        TemporalRoundingMode::Expand => {
            if value >= 0 {
                upper
            } else {
                lower
            }
        }
        TemporalRoundingMode::Trunc => {
            if value >= 0 {
                lower
            } else {
                upper
            }
        }
        TemporalRoundingMode::HalfCeil
        | TemporalRoundingMode::HalfFloor
        | TemporalRoundingMode::HalfExpand
        | TemporalRoundingMode::HalfTrunc
        | TemporalRoundingMode::HalfEven => {
            let twice = remainder.checked_mul(2)?;
            match twice.cmp(&increment) {
                std::cmp::Ordering::Less => lower,
                std::cmp::Ordering::Greater => upper,
                std::cmp::Ordering::Equal => match rounding_mode {
                    TemporalRoundingMode::HalfCeil => upper,
                    TemporalRoundingMode::HalfFloor => lower,
                    TemporalRoundingMode::HalfExpand => {
                        if value >= 0 {
                            upper
                        } else {
                            lower
                        }
                    }
                    TemporalRoundingMode::HalfTrunc => {
                        if value >= 0 {
                            lower
                        } else {
                            upper
                        }
                    }
                    TemporalRoundingMode::HalfEven => {
                        if quotient.rem_euclid(2) == 0 {
                            lower
                        } else {
                            upper
                        }
                    }
                    TemporalRoundingMode::Ceil
                    | TemporalRoundingMode::Floor
                    | TemporalRoundingMode::Expand
                    | TemporalRoundingMode::Trunc => unreachable!("filtered to half modes"),
                },
            }
        }
    };
    Some(rounded)
}

fn round_i128_to_increment_as_if_positive(
    value: i128,
    increment: i128,
    rounding_mode: TemporalRoundingMode,
) -> Option<i128> {
    let quotient = value.div_euclid(increment);
    let remainder = value.rem_euclid(increment);
    let lower = quotient.checked_mul(increment)?;
    if remainder == 0 {
        return Some(lower);
    }
    let upper = quotient.checked_add(1)?.checked_mul(increment)?;
    let rounded = match rounding_mode {
        TemporalRoundingMode::Ceil | TemporalRoundingMode::Expand => upper,
        TemporalRoundingMode::Floor | TemporalRoundingMode::Trunc => lower,
        TemporalRoundingMode::HalfCeil
        | TemporalRoundingMode::HalfFloor
        | TemporalRoundingMode::HalfExpand
        | TemporalRoundingMode::HalfTrunc
        | TemporalRoundingMode::HalfEven => {
            let twice = remainder.checked_mul(2)?;
            match twice.cmp(&increment) {
                std::cmp::Ordering::Less => lower,
                std::cmp::Ordering::Greater => upper,
                std::cmp::Ordering::Equal => match rounding_mode {
                    TemporalRoundingMode::HalfCeil | TemporalRoundingMode::HalfExpand => upper,
                    TemporalRoundingMode::HalfFloor | TemporalRoundingMode::HalfTrunc => lower,
                    TemporalRoundingMode::HalfEven => {
                        if quotient.rem_euclid(2) == 0 {
                            lower
                        } else {
                            upper
                        }
                    }
                    TemporalRoundingMode::Ceil
                    | TemporalRoundingMode::Floor
                    | TemporalRoundingMode::Expand
                    | TemporalRoundingMode::Trunc => unreachable!("filtered to half modes"),
                },
            }
        }
    };
    Some(rounded)
}

const fn duration_largest_time_unit(data: TemporalDurationObjectData) -> DurationLargestTimeUnit {
    if data.days() != 0 {
        DurationLargestTimeUnit::Day
    } else if data.hours() != 0 {
        DurationLargestTimeUnit::Hour
    } else if data.minutes() != 0 {
        DurationLargestTimeUnit::Minute
    } else {
        DurationLargestTimeUnit::Second
    }
}

pub const fn duration_largest_exact_unit(
    data: TemporalDurationObjectData,
) -> TemporalDurationExactUnit {
    if data.days() != 0 {
        TemporalDurationExactUnit::Day
    } else if data.hours() != 0 {
        TemporalDurationExactUnit::Hour
    } else if data.minutes() != 0 {
        TemporalDurationExactUnit::Minute
    } else if data.seconds() != 0 {
        TemporalDurationExactUnit::Second
    } else if data.milliseconds() != 0 {
        TemporalDurationExactUnit::Millisecond
    } else if data.microseconds() != 0 {
        TemporalDurationExactUnit::Microsecond
    } else {
        TemporalDurationExactUnit::Nanosecond
    }
}

pub fn duration_largest_exact_unit_for_addition(
    left: TemporalDurationObjectData,
    right: TemporalDurationExactUnit,
) -> TemporalDurationExactUnit {
    if left.days() != 0 || right == TemporalDurationExactUnit::Day {
        TemporalDurationExactUnit::Day
    } else if left.hours() != 0 || right == TemporalDurationExactUnit::Hour {
        TemporalDurationExactUnit::Hour
    } else if left.minutes() != 0 || right == TemporalDurationExactUnit::Minute {
        TemporalDurationExactUnit::Minute
    } else if left.seconds() != 0 || right == TemporalDurationExactUnit::Second {
        TemporalDurationExactUnit::Second
    } else if left.milliseconds() != 0 || right == TemporalDurationExactUnit::Millisecond {
        TemporalDurationExactUnit::Millisecond
    } else if left.microseconds() != 0 || right == TemporalDurationExactUnit::Microsecond {
        TemporalDurationExactUnit::Microsecond
    } else {
        TemporalDurationExactUnit::Nanosecond
    }
}

fn distribute_duration_nanoseconds_for_largest_unit(
    nanoseconds: i128,
    largest_unit: DurationLargestTimeUnit,
) -> Option<[i128; 7]> {
    let units: &[(usize, i128)] = match largest_unit {
        DurationLargestTimeUnit::Day => &[
            (0, NANOS_PER_DAY),
            (1, NANOS_PER_HOUR),
            (2, NANOS_PER_MINUTE),
            (3, NANOS_PER_SECOND),
            (4, NANOS_PER_MILLISECOND),
            (5, NANOS_PER_MICROSECOND),
            (6, 1),
        ],
        DurationLargestTimeUnit::Hour => &[
            (1, NANOS_PER_HOUR),
            (2, NANOS_PER_MINUTE),
            (3, NANOS_PER_SECOND),
            (4, NANOS_PER_MILLISECOND),
            (5, NANOS_PER_MICROSECOND),
            (6, 1),
        ],
        DurationLargestTimeUnit::Minute => &[
            (2, NANOS_PER_MINUTE),
            (3, NANOS_PER_SECOND),
            (4, NANOS_PER_MILLISECOND),
            (5, NANOS_PER_MICROSECOND),
            (6, 1),
        ],
        DurationLargestTimeUnit::Second => &[
            (3, NANOS_PER_SECOND),
            (4, NANOS_PER_MILLISECOND),
            (5, NANOS_PER_MICROSECOND),
            (6, 1),
        ],
    };
    let mut parts = [0_i128; 7];
    let mut remainder = nanoseconds;
    for (index, unit) in units {
        parts[*index] = duration_component_from_integer(remainder / *unit)?;
        remainder %= *unit;
    }
    Some(parts)
}

fn distribute_additive_duration_nanoseconds(
    nanoseconds: i128,
    largest_unit: TemporalDurationExactUnit,
) -> Option<[i128; 7]> {
    const UNITS: [(TemporalDurationExactUnit, usize, i128); 7] = [
        (TemporalDurationExactUnit::Day, 0, NANOS_PER_DAY),
        (TemporalDurationExactUnit::Hour, 1, NANOS_PER_HOUR),
        (TemporalDurationExactUnit::Minute, 2, NANOS_PER_MINUTE),
        (TemporalDurationExactUnit::Second, 3, NANOS_PER_SECOND),
        (
            TemporalDurationExactUnit::Millisecond,
            4,
            NANOS_PER_MILLISECOND,
        ),
        (
            TemporalDurationExactUnit::Microsecond,
            5,
            NANOS_PER_MICROSECOND,
        ),
        (TemporalDurationExactUnit::Nanosecond, 6, 1),
    ];
    let mut parts = [0_i128; 7];
    let mut remainder = nanoseconds;
    let start = usize::from(largest_unit.order());
    for (_, index, unit) in &UNITS[start..] {
        parts[*index] = duration_component_from_integer(remainder / *unit)?;
        remainder %= *unit;
    }
    Some(parts)
}

fn distribute_duration_nanoseconds_for_exact_largest_unit(
    nanoseconds: i128,
    largest_unit: TemporalDurationExactUnit,
) -> Option<[i128; 7]> {
    const UNITS: [(TemporalDurationExactUnit, usize, i128); 7] = [
        (TemporalDurationExactUnit::Day, 0, NANOS_PER_DAY),
        (TemporalDurationExactUnit::Hour, 1, NANOS_PER_HOUR),
        (TemporalDurationExactUnit::Minute, 2, NANOS_PER_MINUTE),
        (TemporalDurationExactUnit::Second, 3, NANOS_PER_SECOND),
        (
            TemporalDurationExactUnit::Millisecond,
            4,
            NANOS_PER_MILLISECOND,
        ),
        (
            TemporalDurationExactUnit::Microsecond,
            5,
            NANOS_PER_MICROSECOND,
        ),
        (TemporalDurationExactUnit::Nanosecond, 6, 1),
    ];
    let mut parts = [0_i128; 7];
    let mut remainder = nanoseconds;
    let start = usize::from(largest_unit.order());
    for (_, index, unit) in &UNITS[start..] {
        parts[*index] = duration_component_from_integer(remainder / *unit)?;
        remainder %= *unit;
    }
    Some(parts)
}

fn format_duration_parts(
    data: TemporalDurationObjectData,
    fixed_fractional_digits: Option<u8>,
) -> String {
    let sign = duration_sign(data);
    if sign == 0 {
        return match fixed_fractional_digits {
            Some(0) | None => "PT0S".to_string(),
            Some(digits) => format!("PT0.{:0<width$}S", "", width = usize::from(digits)),
        };
    }

    let mut text = String::new();
    if sign < 0 {
        text.push('-');
    }
    text.push('P');

    let years = data.years().unsigned_abs();
    let months = data.months().unsigned_abs();
    let weeks = data.weeks().unsigned_abs();
    let days = data.days().unsigned_abs();
    let hours = data.hours().unsigned_abs();
    let minutes = data.minutes().unsigned_abs();
    let seconds = data.seconds().unsigned_abs();
    let total_subsecond_nanoseconds = seconds
        .saturating_mul(1_000_000_000)
        .saturating_add(data.milliseconds().unsigned_abs().saturating_mul(1_000_000))
        .saturating_add(data.microseconds().unsigned_abs().saturating_mul(1_000))
        .saturating_add(data.nanoseconds().unsigned_abs());
    let whole_seconds = total_subsecond_nanoseconds / 1_000_000_000;
    let fractional_nanoseconds = total_subsecond_nanoseconds % 1_000_000_000;

    if years != 0 {
        let _ = write!(text, "{years}Y");
    }
    if months != 0 {
        let _ = write!(text, "{months}M");
    }
    if weeks != 0 {
        let _ = write!(text, "{weeks}W");
    }
    if days != 0 {
        let _ = write!(text, "{days}D");
    }

    if hours != 0
        || minutes != 0
        || whole_seconds != 0
        || fractional_nanoseconds != 0
        || fixed_fractional_digits.is_some()
    {
        text.push('T');
        if hours != 0 {
            let _ = write!(text, "{hours}H");
        }
        if minutes != 0 {
            let _ = write!(text, "{minutes}M");
        }
        if whole_seconds != 0 || fractional_nanoseconds != 0 || fixed_fractional_digits.is_some() {
            let _ = write!(text, "{whole_seconds}");
            match fixed_fractional_digits {
                Some(0) => {}
                Some(digits) => {
                    let mut fraction = format!("{fractional_nanoseconds:09}");
                    fraction.truncate(usize::from(digits));
                    text.push('.');
                    text.push_str(&fraction);
                }
                None if fractional_nanoseconds != 0 => {
                    let mut fraction = format!("{fractional_nanoseconds:09}");
                    while fraction.ends_with('0') {
                        fraction.pop();
                    }
                    text.push('.');
                    text.push_str(&fraction);
                }
                None => {}
            }
            text.push('S');
        }
    }

    text
}

pub fn negate_duration(data: TemporalDurationObjectData) -> TemporalDurationObjectData {
    TemporalDurationObjectData::new(
        -data.years(),
        -data.months(),
        -data.weeks(),
        -data.days(),
        -data.hours(),
        -data.minutes(),
        -data.seconds(),
        -data.milliseconds(),
        -data.microseconds(),
        -data.nanoseconds(),
    )
}

pub const fn duration_has_lower_than_month_units(data: TemporalDurationObjectData) -> bool {
    data.weeks() != 0
        || data.days() != 0
        || data.hours() != 0
        || data.minutes() != 0
        || data.seconds() != 0
        || data.milliseconds() != 0
        || data.microseconds() != 0
        || data.nanoseconds() != 0
}

pub const fn duration_time_nanoseconds(data: TemporalDurationObjectData) -> i128 {
    data.hours() * NANOS_PER_HOUR
        + data.minutes() * NANOS_PER_MINUTE
        + data.seconds() * NANOS_PER_SECOND
        + data.milliseconds() * NANOS_PER_MILLISECOND
        + data.microseconds() * NANOS_PER_MICROSECOND
        + data.nanoseconds()
}

pub const fn duration_whole_days_from_time(data: TemporalDurationObjectData) -> i128 {
    duration_time_nanoseconds(data) / NANOS_PER_DAY
}

pub const fn is_iso_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

pub const fn iso_days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_iso_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

pub fn iso_day_of_year(year: i32, month: u8, day: u8) -> i32 {
    const COMMON_DAYS_BEFORE_MONTH: [i32; 12] =
        [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let month_index = usize::from(month.saturating_sub(1));
    let leap_adjustment = i32::from(month > 2 && is_iso_leap_year(year));
    COMMON_DAYS_BEFORE_MONTH[month_index] + leap_adjustment + i32::from(day)
}

pub fn iso_days_before_year(year: i32) -> i64 {
    let year = i64::from(year) - 1;
    year * 365 + year.div_euclid(4) - year.div_euclid(100) + year.div_euclid(400)
}

pub fn iso_day_of_week(year: i32, month: u8, day: u8) -> i32 {
    let days_since_iso_epoch =
        iso_days_before_year(year) + i64::from(iso_day_of_year(year, month, day)) - 1;
    i32::try_from(days_since_iso_epoch.rem_euclid(7) + 1)
        .expect("ISO day-of-week should stay in 1..=7")
}

pub fn iso_week_of_year(year: i32, month: u8, day: u8) -> (i32, i32) {
    let day_of_year = iso_day_of_year(year, month, day);
    let day_of_week = iso_day_of_week(year, month, day);
    let week = (day_of_year - day_of_week + 10) / 7;
    if week < 1 {
        let previous_year = year - 1;
        return (iso_weeks_in_year(previous_year), previous_year);
    }
    let weeks_in_year = iso_weeks_in_year(year);
    if week > weeks_in_year {
        return (1, year + 1);
    }
    (week, year)
}

fn iso_weeks_in_year(year: i32) -> i32 {
    let jan_1 = iso_day_of_week(year, 1, 1);
    if jan_1 == 4 || (jan_1 == 3 && is_iso_leap_year(year)) {
        53
    } else {
        52
    }
}

pub const fn iso_days_in_year(year: i32) -> i32 {
    if is_iso_leap_year(year) {
        366
    } else {
        365
    }
}

pub fn plain_date_ordinal_day(data: TemporalPlainDateObjectData) -> i128 {
    i128::from(iso_days_before_year(data.year()))
        + i128::from(iso_day_of_year(data.year(), data.month(), data.day()))
        - 1
}

pub fn plain_time_nanoseconds(data: TemporalPlainTimeObjectData) -> i128 {
    i128::from(data.hour()) * NANOS_PER_HOUR
        + i128::from(data.minute()) * NANOS_PER_MINUTE
        + i128::from(data.second()) * NANOS_PER_SECOND
        + i128::from(data.millisecond()) * NANOS_PER_MILLISECOND
        + i128::from(data.microsecond()) * NANOS_PER_MICROSECOND
        + i128::from(data.nanosecond())
}

pub fn plain_time_from_nanoseconds(nanoseconds: i128) -> Option<TemporalPlainTimeObjectData> {
    let mut remaining = nanoseconds.rem_euclid(NANOS_PER_DAY);
    let hour = remaining / NANOS_PER_HOUR;
    remaining %= NANOS_PER_HOUR;
    let minute = remaining / NANOS_PER_MINUTE;
    remaining %= NANOS_PER_MINUTE;
    let second = remaining / NANOS_PER_SECOND;
    remaining %= NANOS_PER_SECOND;
    let millisecond = remaining / NANOS_PER_MILLISECOND;
    remaining %= NANOS_PER_MILLISECOND;
    let microsecond = remaining / NANOS_PER_MICROSECOND;
    let nanosecond = remaining % NANOS_PER_MICROSECOND;

    Some(TemporalPlainTimeObjectData::new(
        u8::try_from(hour).ok()?,
        u8::try_from(minute).ok()?,
        u8::try_from(second).ok()?,
        u16::try_from(millisecond).ok()?,
        u16::try_from(microsecond).ok()?,
        u16::try_from(nanosecond).ok()?,
    ))
}

pub fn format_iso_year(year: i32) -> String {
    if (0..=9999).contains(&year) {
        format!("{year:04}")
    } else if year < 0 {
        format!("-{:06}", -year)
    } else {
        format!("+{year:06}")
    }
}

pub fn format_plain_date(data: TemporalPlainDateObjectData) -> String {
    let year_text = format_iso_year(data.year());
    format!("{year_text}-{:02}-{:02}", data.month(), data.day())
}

pub fn format_plain_time(data: TemporalPlainTimeObjectData) -> String {
    let mut text = format!(
        "{:02}:{:02}:{:02}",
        data.hour(),
        data.minute(),
        data.second()
    );
    let fraction = u32::from(data.millisecond()) * 1_000_000
        + u32::from(data.microsecond()) * 1_000
        + u32::from(data.nanosecond());
    if fraction != 0 {
        let mut fraction_text = format!("{fraction:09}");
        while fraction_text.ends_with('0') {
            fraction_text.pop();
        }
        text.push('.');
        text.push_str(&fraction_text);
    }
    text
}

pub fn format_plain_date_time(data: TemporalPlainDateTimeObjectData) -> String {
    let date =
        TemporalPlainDateObjectData::new(data.year(), data.month(), data.day(), data.calendar());
    let time = TemporalPlainTimeObjectData::new(
        data.hour(),
        data.minute(),
        data.second(),
        data.millisecond(),
        data.microsecond(),
        data.nanosecond(),
    );
    format!("{}T{}", format_plain_date(date), format_plain_time(time))
}

pub fn format_plain_year_month(data: TemporalPlainYearMonthObjectData) -> String {
    let year_text = format_iso_year(data.year());
    format!("{year_text}-{:02}", data.month())
}

pub fn format_plain_month_day(data: TemporalPlainMonthDayObjectData) -> String {
    format!("{:02}-{:02}", data.month(), data.day())
}

pub fn format_offset(offset_nanoseconds: i64) -> String {
    let sign = if offset_nanoseconds < 0 { '-' } else { '+' };
    let offset = offset_nanoseconds.unsigned_abs();
    let total_seconds = offset / 1_000_000_000;
    let subsecond = offset % 1_000_000_000;
    let hours = total_seconds / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;
    if subsecond == 0 && seconds == 0 {
        return format!("{sign}{hours:02}:{minutes:02}");
    }
    if subsecond == 0 {
        return format!("{sign}{hours:02}:{minutes:02}:{seconds:02}");
    }
    let mut fraction = format!("{subsecond:09}");
    while fraction.ends_with('0') {
        fraction.pop();
    }
    format!("{sign}{hours:02}:{minutes:02}:{seconds:02}.{fraction}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_common::AtomId;

    #[test]
    fn duration_sign_rejects_mixed_sign_components() {
        let duration = TemporalDurationObjectData::new(1, 0, 0, -1, 0, 0, 0, 0, 0, 0);
        assert_eq!(duration_sign(duration), 1);
        assert!(!duration_signs_are_balanced(duration));
    }

    #[test]
    fn duration_limit_check_balances_day_time_units() {
        assert!(duration_is_within_limits(TemporalDurationObjectData::new(
            DURATION_DATE_UNIT_MAX as i64,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        )));
        assert!(!duration_is_within_limits(TemporalDurationObjectData::new(
            DURATION_DATE_UNIT_MAX as i64 + 1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        )));
        assert!(duration_is_within_limits(TemporalDurationObjectData::new(
            0,
            0,
            0,
            104_249_991_374_i128,
            0,
            0,
            0,
            0,
            0,
            27_391_999_999_999_i128,
        )));
        assert!(!duration_is_within_limits(TemporalDurationObjectData::new(
            0,
            0,
            0,
            104_249_991_375_i128,
            0,
            0,
            0,
            0,
            0,
            0,
        )));
        assert!(duration_is_within_limits(TemporalDurationObjectData::new(
            0,
            0,
            0,
            0,
            0,
            0,
            SAFE_INTEGER_MAX as i64,
            999,
            999,
            999,
        )));
        assert!(!duration_is_within_limits(TemporalDurationObjectData::new(
            0,
            0,
            0,
            0,
            0,
            0,
            SAFE_INTEGER_MAX as i64,
            1_000,
            0,
            0,
        )));
        assert!(!duration_is_within_limits(TemporalDurationObjectData::new(
            0,
            0,
            0,
            -104_249_991_375_i128,
            0,
            0,
            0,
            0,
            0,
            0,
        )));
    }

    #[test]
    fn parse_plain_time_requires_designator_for_ambiguous_date_like_strings() {
        assert!(parse_plain_time("1214").is_none());
        assert!(parse_plain_time("12-14").is_none());
        assert!(parse_plain_time("202112").is_none());
        assert!(parse_plain_time("2021-12[u-ca=iso8601]").is_none());
        assert_eq!(parse_plain_time("T1214"), Some((12, 14, 0, 0)));
        assert_eq!(parse_plain_time("202113"), Some((20, 21, 13, 0)));
        assert_eq!(parse_plain_time("13-14"), Some((13, 0, 0, 0)));
    }

    #[test]
    fn duration_format_trims_fractional_seconds() {
        let duration = TemporalDurationObjectData::new(0, 0, 0, 0, 1, 2, 3, 400, 500, 600);
        assert_eq!(format_duration(duration), "PT1H2M3.4005006S");
    }

    #[test]
    fn duration_subsecond_balancing_preserves_exact_integer_totals() {
        assert_eq!(
            balance_duration_subsecond_fields(
                0,
                4_503_599_627_370_497_024,
                4_503_599_627_370_494_951_424,
                0,
            ),
            Some([9_007_199_254_740_991, 975, 424, 0])
        );
        assert_eq!(
            balance_duration_subsecond_fields(
                0,
                -4_503_599_627_370_497_024,
                -4_503_599_627_370_494_951_424,
                0,
            ),
            Some([-9_007_199_254_740_991, -975, -424, 0])
        );
    }

    #[test]
    fn epoch_nanosecond_rounding_uses_as_if_positive_modes_for_negative_values() {
        let half_second_before_epoch_year = -65_261_246_399_500_000_000_i128;
        assert_eq!(
            round_epoch_nanoseconds_to_increment(
                half_second_before_epoch_year,
                NANOS_PER_SECOND,
                TemporalRoundingMode::Floor,
            ),
            Some(-65_261_246_400_000_000_000)
        );
        assert_eq!(
            round_epoch_nanoseconds_to_increment(
                half_second_before_epoch_year,
                NANOS_PER_SECOND,
                TemporalRoundingMode::Trunc,
            ),
            Some(-65_261_246_400_000_000_000)
        );
        assert_eq!(
            round_epoch_nanoseconds_to_increment(
                half_second_before_epoch_year,
                NANOS_PER_SECOND,
                TemporalRoundingMode::Ceil,
            ),
            Some(-65_261_246_399_000_000_000)
        );
        assert_eq!(
            round_epoch_nanoseconds_to_increment(
                half_second_before_epoch_year,
                NANOS_PER_SECOND,
                TemporalRoundingMode::HalfExpand,
            ),
            Some(-65_261_246_399_000_000_000)
        );

        let instant = -1_000_000_000_000_000_000_i128;
        assert_eq!(
            round_epoch_nanoseconds_to_increment(
                instant,
                NANOS_PER_HOUR,
                TemporalRoundingMode::HalfExpand,
            ),
            Some(-1_000_000_800_000_000_000)
        );
        assert_eq!(
            round_epoch_nanoseconds_to_increment(
                instant,
                NANOS_PER_HOUR,
                TemporalRoundingMode::Expand,
            ),
            Some(-999_997_200_000_000_000)
        );
    }

    #[test]
    fn duration_nanosecond_rounding_keeps_signed_modes_for_negative_values() {
        let negative_duration = -1_500_000_000_i128;
        assert_eq!(
            round_duration_nanoseconds_to_increment(
                negative_duration,
                NANOS_PER_SECOND,
                TemporalRoundingMode::Floor,
            ),
            Some(-2_000_000_000)
        );
        assert_eq!(
            round_duration_nanoseconds_to_increment(
                negative_duration,
                NANOS_PER_SECOND,
                TemporalRoundingMode::Trunc,
            ),
            Some(-1_000_000_000)
        );
        assert_eq!(
            round_duration_nanoseconds_to_increment(
                negative_duration,
                NANOS_PER_SECOND,
                TemporalRoundingMode::Ceil,
            ),
            Some(-1_000_000_000)
        );
        assert_eq!(
            round_duration_nanoseconds_to_increment(
                negative_duration,
                NANOS_PER_SECOND,
                TemporalRoundingMode::Expand,
            ),
            Some(-2_000_000_000)
        );
        assert_eq!(
            round_duration_nanoseconds_to_increment(
                negative_duration,
                NANOS_PER_SECOND,
                TemporalRoundingMode::HalfExpand,
            ),
            Some(-2_000_000_000)
        );
        assert_eq!(
            round_duration_nanoseconds_to_increment(
                negative_duration,
                NANOS_PER_SECOND,
                TemporalRoundingMode::HalfTrunc,
            ),
            Some(-1_000_000_000)
        );
    }

    #[test]
    fn duration_format_with_seconds_precision_rounds_and_pads() {
        let duration = TemporalDurationObjectData::new(0, 0, 0, 0, 1, 59, 59, 900, 0, 0);
        assert_eq!(
            format_duration_with_seconds_precision(duration, 0, TemporalRoundingMode::Expand),
            Some("PT2H0S".to_string())
        );
        let duration = TemporalDurationObjectData::new(0, 0, 0, 0, 0, 0, 59, 900, 0, 0);
        assert_eq!(
            format_duration_with_seconds_precision(duration, 0, TemporalRoundingMode::Expand),
            Some("PT60S".to_string())
        );
        let duration = TemporalDurationObjectData::new(1, 2, 3, 4, 5, 6, 7, 987, 650, 0);
        assert_eq!(
            format_duration_with_seconds_precision(duration, 6, TemporalRoundingMode::Trunc),
            Some("P1Y2M3W4DT5H6M7.987650S".to_string())
        );
    }

    #[test]
    fn duration_exact_rounding_balances_to_requested_largest_unit() {
        let duration = TemporalDurationObjectData::new(0, 0, 0, 5, 5, 5, 5, 5, 5, 5);
        assert_eq!(
            duration_components(
                round_duration_exact(
                    duration,
                    TemporalDurationExactUnit::Second,
                    TemporalDurationExactUnit::Nanosecond,
                    1,
                    TemporalRoundingMode::HalfExpand,
                )
                .unwrap()
            ),
            [0, 0, 0, 0, 0, 0, 450_305, 5, 5, 5]
        );

        let rounded = TemporalDurationObjectData::new(0, 0, 0, 0, 1, 59, 59, 900, 0, 0);
        assert_eq!(
            duration_components(
                round_duration_exact(
                    rounded,
                    TemporalDurationExactUnit::Hour,
                    TemporalDurationExactUnit::Minute,
                    1,
                    TemporalRoundingMode::Ceil,
                )
                .unwrap()
            ),
            [0, 0, 0, 0, 2, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn duration_exact_total_divides_by_requested_unit() {
        let duration = TemporalDurationObjectData::new(0, 0, 0, 5, 5, 5, 5, 5, 5, 5);
        assert_eq!(
            total_duration_exact(duration, TemporalDurationExactUnit::Second),
            Some(450_305.005_005_005)
        );
        assert_eq!(
            total_duration_exact(duration, TemporalDurationExactUnit::Millisecond),
            Some(450_305_005.005_005)
        );
        assert_eq!(
            total_duration_exact(duration, TemporalDurationExactUnit::Nanosecond),
            Some(450_305_005_005_005.0)
        );
    }

    #[test]
    fn duration_parse_accepts_iso_duration_strings() {
        let duration = parse_duration("P1Y2M3W4DT5H6M7.00800901S").unwrap();
        assert_eq!(
            duration_components(duration),
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        );
        assert_eq!(
            duration_components(parse_duration("-PT1.03125H").unwrap()),
            [0, 0, 0, 0, -1, -1, -52, -500, 0, 0]
        );
        assert_eq!(
            duration_components(parse_duration("p1dt0,5h").unwrap()),
            [0, 0, 0, 1, 0, 30, 0, 0, 0, 0]
        );
    }

    #[test]
    fn duration_parse_rejects_invalid_iso_duration_strings() {
        for text in ["", "P", "PT", "P2H", "P0.5Y", "PT0.5H5S", "PT1.1234567891S"] {
            assert!(parse_duration(text).is_none(), "{text}");
        }
    }

    #[test]
    fn instant_parser_accepts_utc_offsets_fraction_and_leap_second() {
        assert_eq!(parse_instant("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_instant("1970-01-01T01:00:00+01:00"), Some(0));
        assert_eq!(
            parse_instant("1970-01-01T00:00:00.123456789Z"),
            Some(123_456_789)
        );
        assert_eq!(
            parse_instant("1970-01-01T00:00:60Z"),
            Some(59 * NANOS_PER_SECOND)
        );
        assert_eq!(parse_instant("1970-01-01T00:00:00Z[UTC]"), Some(0));
        assert_eq!(parse_instant("1970-01-01T00:00:00Z[!foo=bar]"), None);
        assert_eq!(parse_instant("1970-01-01T00:00Z[!UTC]"), Some(0));
        assert_eq!(parse_instant("1970-01-01T00:00Z[!u-ca=discord]"), Some(0));
        assert_eq!(parse_instant("1970-01-01T00:00Z[U-CA=iso8601]"), None);
        assert_eq!(parse_instant("1970-01-01T00:00Z[UTC][UTC]"), None);
        assert_eq!(
            parse_instant("1970-01-01T00:00Z[u-ca=iso8601][!u-ca=iso8601]"),
            None
        );
        assert_eq!(
            parse_instant("+275760-09-13T00:00:00Z"),
            Some(INSTANT_EPOCH_NANOSECONDS_MAX)
        );
        assert_eq!(parse_instant("+275760-09-13T00:00:00.000000001Z"), None);
        assert_eq!(
            parse_instant("19761118T152330.1+0000"),
            Some(217_178_610_100_000_000)
        );
        assert_eq!(
            parse_instant("+0019761118T152330.1+0000"),
            Some(217_178_610_100_000_000)
        );
        assert_eq!(
            parse_instant("1976-11-18T15Z"),
            Some(217_177_200_000_000_000)
        );
        assert_eq!(parse_instant("1970-01-01T00+000000.000000000"), Some(0));
        assert_eq!(parse_instant("1970-01-01T00:19:32.37+00:19:32.37"), Some(0));
        assert_eq!(parse_instant("2021-08-19T17:30-07:00:00[-07:00:00]"), None);
        assert_eq!(
            parse_instant("2021-08-19T17:30-07:00:00.1[-070000.0]"),
            None
        );
        assert_eq!(parse_instant("-000000-03-30T00:45Z"), None);
        assert_eq!(parse_instant("1970-01-01T00:00:00"), None);
        assert_eq!(parse_instant("1970-02-30T00:00:00Z"), None);
    }

    #[test]
    fn plain_date_parser_accepts_iso_calendar_dates() {
        assert_eq!(parse_plain_date("2017-01-01"), Some((2017, 1, 1)));
        assert_eq!(parse_plain_date("20170101"), Some((2017, 1, 1)));
        assert_eq!(
            parse_plain_date("2017-01-01[u-ca=iso8601]"),
            Some((2017, 1, 1))
        );
        assert_eq!(parse_plain_date("2017-02-29"), None);
        assert_eq!(parse_plain_date("2017-13-01"), None);
    }

    #[test]
    fn duration_time_compare_orders_time_units() {
        let left = TemporalDurationObjectData::new(0, 0, 0, 0, 5, 5, 5, 5, 5, 5);
        let right = TemporalDurationObjectData::new(0, 0, 0, 0, 5, 4, 5, 5, 5, 5);
        assert_eq!(
            compare_time_duration(left, right),
            Some(std::cmp::Ordering::Greater)
        );
        assert_eq!(
            compare_time_duration(right, left),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            compare_time_duration(left, left),
            Some(std::cmp::Ordering::Equal)
        );
    }

    #[test]
    fn duration_time_compare_treats_days_as_exact_24_hour_units() {
        let left = TemporalDurationObjectData::new(0, 0, 0, 200, 0, 0, 0, 0, 0, 0);
        let right = TemporalDurationObjectData::new(0, 0, 0, 200, 0, 0, 0, 0, 0, 1);
        assert_eq!(
            compare_time_duration(left, right),
            Some(std::cmp::Ordering::Less)
        );
    }

    #[test]
    fn duration_calendar_relative_unit_detection_excludes_days() {
        assert!(duration_has_calendar_relative_units(
            TemporalDurationObjectData::new(0, 0, 1, 0, 0, 0, 0, 0, 0, 0)
        ));
        assert!(!duration_has_calendar_relative_units(
            TemporalDurationObjectData::new(0, 0, 0, 1, 0, 0, 0, 0, 0, 0)
        ));
        assert!(duration_has_date_units(TemporalDurationObjectData::new(
            0, 0, 0, 1, 0, 0, 0, 0, 0, 0
        )));
        assert!(!duration_has_date_units(TemporalDurationObjectData::new(
            0, 0, 0, 0, 1, 0, 0, 0, 0, 0
        )));
    }

    #[test]
    fn duration_calendar_relative_component_equality_ignores_exact_units() {
        let left = TemporalDurationObjectData::new(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
        let right = TemporalDurationObjectData::new(1, 2, 3, 40, 50, 60, 70, 80, 90, 100);
        let different = TemporalDurationObjectData::new(1, 2, 4, 4, 5, 6, 7, 8, 9, 10);
        assert!(duration_calendar_relative_components_are_equal(left, right));
        assert!(!duration_calendar_relative_components_are_equal(
            left, different
        ));
    }

    #[test]
    fn duration_add_and_subtract_balance_day_time_units() {
        let duration = TemporalDurationObjectData::new(0, 0, 0, 50, 50, 50, 50, 500, 500, 500);
        assert_eq!(
            duration_components(add_durations(duration, duration).unwrap()),
            [0, 0, 0, 104, 5, 41, 41, 1, 1, 0]
        );
        let left = TemporalDurationObjectData::new(0, 0, 0, 0, 1, 0, 3721, 0, 0, 0);
        let right =
            TemporalDurationObjectData::new(0, 0, 0, 0, 0, 61, 0, 0, 0, 3_722_000_000_001_i128);
        assert_eq!(
            duration_components(subtract_durations(left, right).unwrap()),
            [0, 0, 0, 0, 0, -1, -1, 0, 0, -1]
        );
    }

    #[test]
    fn iso_calendar_helpers_cover_leap_years() {
        assert_eq!(iso_days_in_month(2024, 2), 29);
        assert_eq!(iso_days_in_month(2023, 2), 28);
        assert_eq!(iso_day_of_year(2024, 3, 1), 61);
        assert_eq!(iso_week_of_year(2021, 1, 1), (53, 2020));
        assert_eq!(iso_week_of_year(2021, 1, 4), (1, 2021));
        assert_eq!(iso_week_of_year(1970, 1, 1), (1, 1970));
    }

    #[test]
    fn plain_time_nanoseconds_wraps_by_day() {
        let time = plain_time_from_nanoseconds(NANOS_PER_DAY + 3 * NANOS_PER_HOUR + 5).unwrap();
        assert_eq!(time.hour(), 3);
        assert_eq!(time.minute(), 0);
        assert_eq!(time.nanosecond(), 5);
    }

    #[test]
    fn plain_date_time_format_uses_temporal_iso_shape() {
        let calendar = AtomId::from_raw(1);
        let data = TemporalPlainDateTimeObjectData::new(2026, 4, 21, 9, 8, 7, 6, 5, 4, calendar);
        assert_eq!(
            format_plain_date_time(data),
            "2026-04-21T09:08:07.006005004"
        );
    }
}

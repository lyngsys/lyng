use super::{
    range_error, string_ref_text, temporal_number_to_i128_after_range_check, to_number_for_builtin,
    to_string_string_ref, PublicBuiltinDispatchContext, TemporalBuiltinDurationExactUnit,
    TemporalDurationCalendarUnit, TemporalDurationParsedLargestUnit, TemporalDurationParsedUnit,
    Value,
};

pub(in crate::public::dispatch::temporal) fn temporal_duration_largest_unit_option<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationParsedLargestUnit, Cx::Error> {
    if value.is_undefined() {
        return Ok(TemporalDurationParsedLargestUnit::Missing);
    }
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    if text == "auto" {
        return Ok(TemporalDurationParsedLargestUnit::Auto);
    }
    Ok(match temporal_duration_unit_from_text(&text) {
        Some(TemporalDurationParsedUnit::Exact(unit)) => {
            TemporalDurationParsedLargestUnit::Exact(unit)
        }
        Some(TemporalDurationParsedUnit::CalendarRelative(unit)) => {
            TemporalDurationParsedLargestUnit::CalendarRelative(unit)
        }
        None => return Err(range_error(cx)),
    })
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_parsed_unit<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationParsedUnit, Cx::Error> {
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    temporal_duration_unit_from_text(&text).ok_or_else(|| range_error(cx))
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_unit_from_text(
    text: &str,
) -> Option<TemporalDurationParsedUnit> {
    match text {
        "year" | "years" => Some(TemporalDurationParsedUnit::CalendarRelative(
            TemporalDurationCalendarUnit::Year,
        )),
        "month" | "months" => Some(TemporalDurationParsedUnit::CalendarRelative(
            TemporalDurationCalendarUnit::Month,
        )),
        "week" | "weeks" => Some(TemporalDurationParsedUnit::CalendarRelative(
            TemporalDurationCalendarUnit::Week,
        )),
        "day" | "days" => Some(TemporalDurationParsedUnit::Exact(
            TemporalBuiltinDurationExactUnit::Day,
        )),
        "hour" | "hours" => Some(TemporalDurationParsedUnit::Exact(
            TemporalBuiltinDurationExactUnit::Hour,
        )),
        "minute" | "minutes" => Some(TemporalDurationParsedUnit::Exact(
            TemporalBuiltinDurationExactUnit::Minute,
        )),
        "second" | "seconds" => Some(TemporalDurationParsedUnit::Exact(
            TemporalBuiltinDurationExactUnit::Second,
        )),
        "millisecond" | "milliseconds" => Some(TemporalDurationParsedUnit::Exact(
            TemporalBuiltinDurationExactUnit::Millisecond,
        )),
        "microsecond" | "microseconds" => Some(TemporalDurationParsedUnit::Exact(
            TemporalBuiltinDurationExactUnit::Microsecond,
        )),
        "nanosecond" | "nanoseconds" => Some(TemporalDurationParsedUnit::Exact(
            TemporalBuiltinDurationExactUnit::Nanosecond,
        )),
        _ => None,
    }
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_rounding_increment_option<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
) -> Result<i128, Cx::Error> {
    if value.is_undefined() {
        return Ok(1);
    }
    let number = to_number_for_builtin(cx, value)?;
    if !number.is_finite() || number.is_nan() {
        return Err(range_error(cx));
    }
    let increment = number.floor();
    if !(1.0..=1_000_000_000.0).contains(&increment) {
        return Err(range_error(cx));
    }
    Ok(temporal_number_to_i128_after_range_check(increment))
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_rounding_increment_is_valid(
    smallest_unit: TemporalBuiltinDurationExactUnit,
    rounding_increment: i128,
) -> bool {
    let maximum = match smallest_unit {
        TemporalBuiltinDurationExactUnit::Day => {
            return (1..=1_000_000_000).contains(&rounding_increment);
        }
        TemporalBuiltinDurationExactUnit::Hour => 24,
        TemporalBuiltinDurationExactUnit::Minute | TemporalBuiltinDurationExactUnit::Second => 60,
        TemporalBuiltinDurationExactUnit::Millisecond
        | TemporalBuiltinDurationExactUnit::Microsecond
        | TemporalBuiltinDurationExactUnit::Nanosecond => 1000,
    };
    rounding_increment > 0 && rounding_increment < maximum && maximum % rounding_increment == 0
}

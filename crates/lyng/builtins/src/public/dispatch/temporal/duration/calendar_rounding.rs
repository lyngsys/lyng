use super::{
    range_error, temporal_date_difference_unit_order, temporal_duration_exact_unit_nanoseconds,
    temporal_duration_from_date_units, temporal_duration_from_nanoseconds_with_largest_unit,
    temporal_duration_time_nanoseconds, temporal_ops, temporal_plain_date_add_duration,
    temporal_plain_date_difference_trunc, temporal_plain_date_ordinal_day,
    temporal_plain_date_time_add_duration, temporal_plain_date_time_date,
    temporal_plain_date_time_from_total_nanoseconds, temporal_plain_date_time_time,
    temporal_plain_date_time_total_nanoseconds, temporal_plain_time_nanoseconds,
    temporal_round_duration_nanoseconds_to_increment, temporal_round_i128_to_increment,
    temporal_zoned_date_time_add_duration, temporal_zoned_date_time_civil,
    temporal_zoned_date_time_data, PublicBuiltinDispatchContext, TemporalBuiltinDurationExactUnit,
    TemporalBuiltinRoundingMode, TemporalDateDifferenceUnit, TemporalDurationCalendarUnit,
    TemporalDurationObjectData, TemporalDurationParsedLargestUnit, TemporalDurationParsedUnit,
    TemporalDurationRelativeTo, TemporalOverflow, TemporalPlainDateObjectData,
    TemporalPlainDateTimeObjectData, TEMPORAL_NANOS_PER_DAY,
};

pub(in crate::public::dispatch::temporal) fn temporal_duration_relative_total_nanoseconds<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    duration: TemporalDurationObjectData,
    relative_to: TemporalDurationRelativeTo,
) -> Result<i128, Cx::Error> {
    match relative_to {
        TemporalDurationRelativeTo::PlainDate(start) => {
            let date_duration = TemporalDurationObjectData::new(
                duration.years(),
                duration.months(),
                duration.weeks(),
                duration.days(),
                0,
                0,
                0,
                0,
                0,
                0,
            );
            let end = temporal_plain_date_add_duration(
                cx,
                start,
                date_duration,
                TemporalOverflow::Constrain,
            )?;
            let days = temporal_plain_date_ordinal_day(end)
                .checked_sub(temporal_plain_date_ordinal_day(start))
                .ok_or_else(|| range_error(cx))?;
            days.checked_mul(TEMPORAL_NANOS_PER_DAY)
                .and_then(|date_nanoseconds| {
                    date_nanoseconds.checked_add(temporal_duration_time_nanoseconds(duration))
                })
                .ok_or_else(|| range_error(cx))
        }
        TemporalDurationRelativeTo::PlainDateTime(start) => {
            let end = temporal_plain_date_time_add_duration(
                cx,
                start,
                duration,
                TemporalOverflow::Constrain,
            )?;
            temporal_plain_date_time_total_nanoseconds(end)
                .and_then(|end_nanoseconds| {
                    let start_nanoseconds = temporal_plain_date_time_total_nanoseconds(start)?;
                    end_nanoseconds.checked_sub(start_nanoseconds)
                })
                .ok_or_else(|| range_error(cx))
        }
        TemporalDurationRelativeTo::ZonedDateTime(start) => {
            let end_value = temporal_zoned_date_time_add_duration(
                cx,
                start,
                duration,
                TemporalOverflow::Constrain,
            )?;
            let end = temporal_zoned_date_time_data(cx, end_value)?;
            end.epoch_nanoseconds()
                .checked_sub(start.epoch_nanoseconds())
                .ok_or_else(|| range_error(cx))
        }
    }
}

pub(in crate::public::dispatch::temporal) const fn temporal_duration_round_calendar_largest_unit(
    duration: TemporalDurationObjectData,
    largest_unit: TemporalDurationParsedLargestUnit,
    smallest_unit: Option<TemporalDurationParsedUnit>,
) -> Option<TemporalDateDifferenceUnit> {
    match largest_unit {
        TemporalDurationParsedLargestUnit::CalendarRelative(unit) => Some(
            temporal_duration_calendar_unit_to_date_difference_unit(unit),
        ),
        TemporalDurationParsedLargestUnit::Missing | TemporalDurationParsedLargestUnit::Auto => {
            match smallest_unit {
                Some(TemporalDurationParsedUnit::CalendarRelative(unit)) => {
                    let smallest = temporal_duration_calendar_unit_to_date_difference_unit(unit);
                    Some(temporal_duration_default_largest_date_unit(
                        duration, smallest,
                    ))
                }
                Some(TemporalDurationParsedUnit::Exact(_))
                    if temporal_ops::duration_has_calendar_relative_units(duration) =>
                {
                    Some(temporal_duration_default_largest_date_unit(
                        duration,
                        TemporalDateDifferenceUnit::Day,
                    ))
                }
                Some(TemporalDurationParsedUnit::Exact(_)) | None => None,
            }
        }
        TemporalDurationParsedLargestUnit::Exact(_) => None,
    }
}

pub(in crate::public::dispatch::temporal) const fn temporal_duration_default_largest_date_unit(
    duration: TemporalDurationObjectData,
    smallest_unit: TemporalDateDifferenceUnit,
) -> TemporalDateDifferenceUnit {
    let largest_present = if duration.years() != 0 {
        TemporalDateDifferenceUnit::Year
    } else if duration.months() != 0 {
        TemporalDateDifferenceUnit::Month
    } else if duration.weeks() != 0 {
        TemporalDateDifferenceUnit::Week
    } else {
        TemporalDateDifferenceUnit::Day
    };
    if temporal_date_difference_unit_order(largest_present)
        <= temporal_date_difference_unit_order(smallest_unit)
    {
        largest_present
    } else {
        smallest_unit
    }
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_round_calendar_smallest_unit<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    smallest_unit: Option<TemporalDurationParsedUnit>,
    _largest_unit: TemporalDateDifferenceUnit,
) -> Result<TemporalDateDifferenceUnit, Cx::Error> {
    match smallest_unit {
        Some(TemporalDurationParsedUnit::CalendarRelative(unit)) => Ok(
            temporal_duration_calendar_unit_to_date_difference_unit(unit),
        ),
        Some(TemporalDurationParsedUnit::Exact(TemporalBuiltinDurationExactUnit::Day)) => {
            Ok(TemporalDateDifferenceUnit::Day)
        }
        Some(TemporalDurationParsedUnit::Exact(_)) => Err(range_error(cx)),
        None => Ok(TemporalDateDifferenceUnit::Day),
    }
}

pub(in crate::public::dispatch::temporal) const fn temporal_duration_calendar_unit_to_date_difference_unit(
    unit: TemporalDurationCalendarUnit,
) -> TemporalDateDifferenceUnit {
    match unit {
        TemporalDurationCalendarUnit::Year => TemporalDateDifferenceUnit::Year,
        TemporalDurationCalendarUnit::Month => TemporalDateDifferenceUnit::Month,
        TemporalDurationCalendarUnit::Week => TemporalDateDifferenceUnit::Week,
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "calendar-relative duration rounding keeps the balancing and remainder steps in one algorithm"
)]
pub(in crate::public::dispatch::temporal) fn temporal_duration_round_calendar_relative<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    duration: TemporalDurationObjectData,
    relative_to: TemporalDurationRelativeTo,
    largest_unit: TemporalDateDifferenceUnit,
    smallest_unit: TemporalDateDifferenceUnit,
    rounding_increment: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
) -> Result<TemporalDurationObjectData, Cx::Error> {
    match smallest_unit {
        TemporalDateDifferenceUnit::Week | TemporalDateDifferenceUnit::Day => {
            if smallest_unit == TemporalDateDifferenceUnit::Week
                && matches!(
                    largest_unit,
                    TemporalDateDifferenceUnit::Year | TemporalDateDifferenceUnit::Month
                )
            {
                return temporal_duration_round_calendar_relative_weeks(
                    cx,
                    duration,
                    relative_to,
                    largest_unit,
                    rounding_increment,
                    rounding_mode,
                );
            }
            let unit_days = if smallest_unit == TemporalDateDifferenceUnit::Week {
                7
            } else {
                1
            };
            let increment = rounding_increment
                .checked_mul(unit_days)
                .and_then(|days| days.checked_mul(TEMPORAL_NANOS_PER_DAY))
                .ok_or_else(|| range_error(cx))?;
            let total_nanoseconds =
                temporal_duration_relative_total_nanoseconds(cx, duration, relative_to)?;
            let rounded = temporal_round_duration_nanoseconds_to_increment(
                total_nanoseconds,
                increment,
                rounding_mode,
            )
            .ok_or_else(|| range_error(cx))?;
            let days = rounded
                .checked_div(TEMPORAL_NANOS_PER_DAY)
                .ok_or_else(|| range_error(cx))?;
            temporal_duration_validate_day_rounding_boundary(cx, relative_to, days)?;
            let days_increment = rounding_increment
                .checked_mul(unit_days)
                .ok_or_else(|| range_error(cx))?;
            if days == 0 && total_nanoseconds != 0 && rounding_increment != 1 {
                let adjacent = if total_nanoseconds < 0 {
                    -days_increment
                } else {
                    days_increment
                };
                temporal_duration_validate_day_rounding_boundary(cx, relative_to, adjacent)?;
            }
            if matches!(
                largest_unit,
                TemporalDateDifferenceUnit::Year | TemporalDateDifferenceUnit::Month
            ) {
                let start = temporal_duration_relative_start_plain_date(cx, relative_to)?;
                let rounded_days = i64::try_from(days).map_err(|_| range_error(cx))?;
                let end = temporal_plain_date_add_duration(
                    cx,
                    start,
                    TemporalDurationObjectData::new(0, 0, 0, rounded_days, 0, 0, 0, 0, 0, 0),
                    TemporalOverflow::Constrain,
                )?;
                return temporal_plain_date_difference_trunc(
                    cx,
                    start,
                    end,
                    largest_unit,
                    smallest_unit,
                );
            }
            temporal_duration_from_date_units(
                cx,
                days,
                largest_unit,
                TemporalDateDifferenceUnit::Day,
            )
        }
        TemporalDateDifferenceUnit::Year | TemporalDateDifferenceUnit::Month => {
            if smallest_unit == TemporalDateDifferenceUnit::Year {
                let raw_years = temporal_duration_round_years_between_dates(
                    cx,
                    duration,
                    relative_to,
                    rounding_increment,
                    rounding_mode,
                )?;
                return temporal_duration_from_date_units(
                    cx,
                    raw_years,
                    largest_unit,
                    TemporalDateDifferenceUnit::Year,
                );
            }
            let raw_months = temporal_duration_round_months_between_dates(
                cx,
                duration,
                relative_to,
                largest_unit,
                rounding_increment,
                rounding_mode,
            )?;
            temporal_duration_from_date_units(
                cx,
                raw_months,
                largest_unit,
                TemporalDateDifferenceUnit::Month,
            )
        }
    }
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_round_calendar_relative_weeks<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    duration: TemporalDurationObjectData,
    relative_to: TemporalDurationRelativeTo,
    largest_unit: TemporalDateDifferenceUnit,
    rounding_increment: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
) -> Result<TemporalDurationObjectData, Cx::Error> {
    let total_nanoseconds =
        temporal_duration_relative_total_nanoseconds(cx, duration, relative_to)?;
    let (date, time_nanoseconds) = temporal_duration_calendar_day_time_difference(
        cx,
        relative_to,
        total_nanoseconds,
        largest_unit,
    )?;
    let remainder = date
        .days()
        .checked_mul(TEMPORAL_NANOS_PER_DAY)
        .and_then(|days| days.checked_add(time_nanoseconds))
        .ok_or_else(|| range_error(cx))?;
    let increment = rounding_increment
        .checked_mul(7)
        .and_then(|weeks| weeks.checked_mul(TEMPORAL_NANOS_PER_DAY))
        .ok_or_else(|| range_error(cx))?;
    let rounded =
        temporal_round_duration_nanoseconds_to_increment(remainder, increment, rounding_mode)
            .ok_or_else(|| range_error(cx))?;
    let weeks = rounded
        .checked_div(7 * TEMPORAL_NANOS_PER_DAY)
        .ok_or_else(|| range_error(cx))?;
    Ok(TemporalDurationObjectData::new(
        date.years(),
        date.months(),
        i64::try_from(weeks).map_err(|_| range_error(cx))?,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ))
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_round_calendar_relative_exact<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    duration: TemporalDurationObjectData,
    relative_to: TemporalDurationRelativeTo,
    largest_unit: TemporalDateDifferenceUnit,
    smallest_unit: TemporalBuiltinDurationExactUnit,
    rounding_increment: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
) -> Result<TemporalDurationObjectData, Cx::Error> {
    let total_nanoseconds =
        temporal_duration_relative_total_nanoseconds(cx, duration, relative_to)?;
    let increment = temporal_duration_exact_unit_nanoseconds(smallest_unit)
        .checked_mul(rounding_increment)
        .ok_or_else(|| range_error(cx))?;
    let rounded = temporal_round_duration_nanoseconds_to_increment(
        total_nanoseconds,
        increment,
        rounding_mode,
    )
    .ok_or_else(|| range_error(cx))?;
    let (date, time_nanoseconds) =
        temporal_duration_calendar_day_time_difference(cx, relative_to, rounded, largest_unit)?;
    let time = temporal_duration_from_nanoseconds_with_largest_unit(
        cx,
        time_nanoseconds,
        TemporalBuiltinDurationExactUnit::Hour,
    )?;
    Ok(TemporalDurationObjectData::new(
        date.years(),
        date.months(),
        date.weeks(),
        date.days(),
        time.hours(),
        time.minutes(),
        time.seconds(),
        time.milliseconds(),
        time.microseconds(),
        time.nanoseconds(),
    ))
}

pub(in crate::public::dispatch::temporal) const fn temporal_rounding_mode_for_negated_duration(
    rounding_mode: TemporalBuiltinRoundingMode,
) -> TemporalBuiltinRoundingMode {
    match rounding_mode {
        TemporalBuiltinRoundingMode::Ceil => TemporalBuiltinRoundingMode::Floor,
        TemporalBuiltinRoundingMode::Floor => TemporalBuiltinRoundingMode::Ceil,
        TemporalBuiltinRoundingMode::HalfCeil => TemporalBuiltinRoundingMode::HalfFloor,
        TemporalBuiltinRoundingMode::HalfFloor => TemporalBuiltinRoundingMode::HalfCeil,
        TemporalBuiltinRoundingMode::Expand
        | TemporalBuiltinRoundingMode::Trunc
        | TemporalBuiltinRoundingMode::HalfExpand
        | TemporalBuiltinRoundingMode::HalfTrunc
        | TemporalBuiltinRoundingMode::HalfEven => rounding_mode,
    }
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_calendar_day_time_difference<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    relative_to: TemporalDurationRelativeTo,
    total_nanoseconds: i128,
    largest_unit: TemporalDateDifferenceUnit,
) -> Result<(TemporalDurationObjectData, i128), Cx::Error> {
    let start = temporal_duration_relative_start_plain_date_time(cx, relative_to)?;
    let start_total =
        temporal_plain_date_time_total_nanoseconds(start).ok_or_else(|| range_error(cx))?;
    let end_total = start_total
        .checked_add(total_nanoseconds)
        .ok_or_else(|| range_error(cx))?;
    let end = temporal_plain_date_time_from_total_nanoseconds(cx, end_total)?;
    let start_date = temporal_plain_date_time_date(start);
    let mut end_date = temporal_plain_date_time_date(end);
    let start_time = temporal_plain_time_nanoseconds(temporal_plain_date_time_time(start));
    let end_time = temporal_plain_time_nanoseconds(temporal_plain_date_time_time(end));
    let mut time_nanoseconds = end_time
        .checked_sub(start_time)
        .ok_or_else(|| range_error(cx))?;
    if total_nanoseconds >= 0 && time_nanoseconds < 0 {
        end_date = temporal_plain_date_add_duration(
            cx,
            end_date,
            TemporalDurationObjectData::new(0, 0, 0, -1, 0, 0, 0, 0, 0, 0),
            TemporalOverflow::Constrain,
        )?;
        time_nanoseconds = time_nanoseconds
            .checked_add(TEMPORAL_NANOS_PER_DAY)
            .ok_or_else(|| range_error(cx))?;
    } else if total_nanoseconds < 0 && time_nanoseconds > 0 {
        end_date = temporal_plain_date_add_duration(
            cx,
            end_date,
            TemporalDurationObjectData::new(0, 0, 0, 1, 0, 0, 0, 0, 0, 0),
            TemporalOverflow::Constrain,
        )?;
        time_nanoseconds = time_nanoseconds
            .checked_sub(TEMPORAL_NANOS_PER_DAY)
            .ok_or_else(|| range_error(cx))?;
    }
    let date = temporal_plain_date_difference_trunc(
        cx,
        start_date,
        end_date,
        largest_unit,
        TemporalDateDifferenceUnit::Day,
    )?;
    Ok((date, time_nanoseconds))
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_round_years_between_dates<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    duration: TemporalDurationObjectData,
    relative_to: TemporalDurationRelativeTo,
    increment: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
) -> Result<i128, Cx::Error> {
    let total_nanoseconds =
        temporal_duration_relative_total_nanoseconds(cx, duration, relative_to)?;
    let (date, _) = temporal_duration_calendar_day_time_difference(
        cx,
        relative_to,
        total_nanoseconds,
        TemporalDateDifferenceUnit::Year,
    )?;
    let start = temporal_duration_relative_start_plain_date_time(cx, relative_to)?;
    let start_date = temporal_plain_date_time_date(start);
    let start_time = temporal_plain_time_nanoseconds(temporal_plain_date_time_time(start));
    let start_total =
        temporal_plain_date_time_total_nanoseconds(start).ok_or_else(|| range_error(cx))?;
    let end_total = start_total
        .checked_add(total_nanoseconds)
        .ok_or_else(|| range_error(cx))?;
    let whole_years = date.years();
    let whole_years_i64 = i64::try_from(whole_years).map_err(|_| range_error(cx))?;
    let after_years = temporal_plain_date_add_duration(
        cx,
        start_date,
        TemporalDurationObjectData::new(whole_years_i64, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        TemporalOverflow::Constrain,
    )?;
    let after_years_total =
        temporal_duration_date_with_start_time_total_nanoseconds(cx, after_years, start_time)?;

    if whole_years < 0 || end_total < after_years_total {
        let previous_years =
            i64::try_from(whole_years.checked_sub(1).ok_or_else(|| range_error(cx))?)
                .map_err(|_| range_error(cx))?;
        let previous_year = temporal_plain_date_add_duration(
            cx,
            start_date,
            TemporalDurationObjectData::new(previous_years, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            TemporalOverflow::Constrain,
        )?;
        let previous_year_total = temporal_duration_date_with_start_time_total_nanoseconds(
            cx,
            previous_year,
            start_time,
        )?;
        let remainder = after_years_total
            .checked_sub(end_total)
            .ok_or_else(|| range_error(cx))?;
        let unit = after_years_total
            .checked_sub(previous_year_total)
            .ok_or_else(|| range_error(cx))?;
        let magnitude = -whole_years;
        let rounded_abs = if temporal_duration_should_round_negative_remainder_away(
            magnitude,
            remainder,
            unit,
            rounding_mode,
        ) {
            magnitude.checked_add(1).ok_or_else(|| range_error(cx))?
        } else {
            magnitude
        };
        return temporal_round_i128_to_increment(cx, -rounded_abs, increment, rounding_mode);
    }

    let next_years = i64::try_from(whole_years.checked_add(1).ok_or_else(|| range_error(cx))?)
        .map_err(|_| range_error(cx))?;
    let next_year = temporal_plain_date_add_duration(
        cx,
        start_date,
        TemporalDurationObjectData::new(next_years, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        TemporalOverflow::Constrain,
    )?;
    let next_year_total =
        temporal_duration_date_with_start_time_total_nanoseconds(cx, next_year, start_time)?;
    let remainder = end_total
        .checked_sub(after_years_total)
        .ok_or_else(|| range_error(cx))?;
    let unit = next_year_total
        .checked_sub(after_years_total)
        .ok_or_else(|| range_error(cx))?;
    let rounded_years = if temporal_duration_should_round_positive_remainder_up(
        whole_years,
        remainder,
        unit,
        rounding_mode,
    ) {
        whole_years.checked_add(1).ok_or_else(|| range_error(cx))?
    } else {
        whole_years
    };
    temporal_round_i128_to_increment(cx, rounded_years, increment, rounding_mode)
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_date_with_start_time_total_nanoseconds<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    date: TemporalPlainDateObjectData,
    start_time: i128,
) -> Result<i128, Cx::Error> {
    temporal_plain_date_ordinal_day(date)
        .checked_mul(TEMPORAL_NANOS_PER_DAY)
        .and_then(|days| days.checked_add(start_time))
        .ok_or_else(|| range_error(cx))
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_round_months_between_dates<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    duration: TemporalDurationObjectData,
    relative_to: TemporalDurationRelativeTo,
    largest_unit: TemporalDateDifferenceUnit,
    increment: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
) -> Result<i128, Cx::Error> {
    let total_nanoseconds =
        temporal_duration_relative_total_nanoseconds(cx, duration, relative_to)?;
    let (date, time_nanoseconds) = temporal_duration_calendar_day_time_difference(
        cx,
        relative_to,
        total_nanoseconds,
        largest_unit,
    )?;
    let start = temporal_duration_relative_start_plain_date(cx, relative_to)?;
    let whole_months = date
        .years()
        .checked_mul(12)
        .and_then(|years| years.checked_add(date.months()))
        .ok_or_else(|| range_error(cx))?;
    let remainder = date
        .days()
        .checked_mul(TEMPORAL_NANOS_PER_DAY)
        .and_then(|days| days.checked_add(time_nanoseconds))
        .ok_or_else(|| range_error(cx))?;
    if whole_months < 0 || remainder < 0 {
        let magnitude = -whole_months;
        let whole_months_i64 = i64::try_from(whole_months).map_err(|_| range_error(cx))?;
        let after_months = temporal_plain_date_add_duration(
            cx,
            start,
            TemporalDurationObjectData::new(0, whole_months_i64, 0, 0, 0, 0, 0, 0, 0, 0),
            TemporalOverflow::Constrain,
        )?;
        let previous_months =
            i64::try_from(whole_months.checked_sub(1).ok_or_else(|| range_error(cx))?)
                .map_err(|_| range_error(cx))?;
        let previous_month = temporal_plain_date_add_duration(
            cx,
            start,
            TemporalDurationObjectData::new(0, previous_months, 0, 0, 0, 0, 0, 0, 0, 0),
            TemporalOverflow::Constrain,
        )?;
        let unit = temporal_plain_date_ordinal_day(after_months)
            .checked_sub(temporal_plain_date_ordinal_day(previous_month))
            .and_then(|days| days.checked_mul(TEMPORAL_NANOS_PER_DAY))
            .ok_or_else(|| range_error(cx))?;
        let rounded_abs = if temporal_duration_should_round_negative_remainder_away(
            magnitude,
            -remainder,
            unit,
            rounding_mode,
        ) {
            magnitude.checked_add(1).ok_or_else(|| range_error(cx))?
        } else {
            magnitude
        };
        let rounded = temporal_round_i128_to_increment(cx, -rounded_abs, increment, rounding_mode)?;
        temporal_duration_validate_month_rounding_boundary(cx, start, rounded)?;
        if increment != 1 {
            let adjacent = rounded
                .checked_sub(increment)
                .ok_or_else(|| range_error(cx))?;
            temporal_duration_validate_month_rounding_boundary(cx, start, adjacent)?;
        }
        return Ok(rounded);
    }
    let after_months_count = i64::try_from(whole_months).map_err(|_| range_error(cx))?;
    let after_months = temporal_plain_date_add_duration(
        cx,
        start,
        TemporalDurationObjectData::new(0, after_months_count, 0, 0, 0, 0, 0, 0, 0, 0),
        TemporalOverflow::Constrain,
    )?;
    let next_months = i64::try_from(whole_months.checked_add(1).ok_or_else(|| range_error(cx))?)
        .map_err(|_| range_error(cx))?;
    let next_month = temporal_plain_date_add_duration(
        cx,
        start,
        TemporalDurationObjectData::new(0, next_months, 0, 0, 0, 0, 0, 0, 0, 0),
        TemporalOverflow::Constrain,
    )?;
    let unit = temporal_plain_date_ordinal_day(next_month)
        .checked_sub(temporal_plain_date_ordinal_day(after_months))
        .and_then(|days| days.checked_mul(TEMPORAL_NANOS_PER_DAY))
        .ok_or_else(|| range_error(cx))?;
    let rounded_months = if temporal_duration_should_round_positive_remainder_up(
        whole_months,
        remainder,
        unit,
        rounding_mode,
    ) {
        whole_months.checked_add(1).ok_or_else(|| range_error(cx))?
    } else {
        whole_months
    };
    let rounded = temporal_round_i128_to_increment(cx, rounded_months, increment, rounding_mode)?;
    temporal_duration_validate_month_rounding_boundary(cx, start, rounded)?;
    if increment != 1 {
        let adjacent = rounded
            .checked_add(increment)
            .ok_or_else(|| range_error(cx))?;
        temporal_duration_validate_month_rounding_boundary(cx, start, adjacent)?;
    }
    Ok(rounded)
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_validate_month_rounding_boundary<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    start: TemporalPlainDateObjectData,
    months: i128,
) -> Result<(), Cx::Error> {
    let months = i64::try_from(months).map_err(|_| range_error(cx))?;
    let _ = temporal_plain_date_add_duration(
        cx,
        start,
        TemporalDurationObjectData::new(0, months, 0, 0, 0, 0, 0, 0, 0, 0),
        TemporalOverflow::Constrain,
    )?;
    Ok(())
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_validate_day_rounding_boundary<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    relative_to: TemporalDurationRelativeTo,
    days: i128,
) -> Result<(), Cx::Error> {
    let days = i64::try_from(days).map_err(|_| range_error(cx))?;
    let duration = TemporalDurationObjectData::new(0, 0, 0, days, 0, 0, 0, 0, 0, 0);
    match relative_to {
        TemporalDurationRelativeTo::PlainDate(_) | TemporalDurationRelativeTo::PlainDateTime(_) => {
        }
        TemporalDurationRelativeTo::ZonedDateTime(start) => {
            let value = temporal_zoned_date_time_add_duration(
                cx,
                start,
                duration,
                TemporalOverflow::Constrain,
            )?;
            let _ = temporal_zoned_date_time_data(cx, value)?;
        }
    }
    Ok(())
}

pub(in crate::public::dispatch::temporal) const fn temporal_duration_should_round_positive_remainder_up(
    lower: i128,
    remainder: i128,
    unit: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
) -> bool {
    if remainder == 0 {
        return false;
    }
    match rounding_mode {
        TemporalBuiltinRoundingMode::Ceil | TemporalBuiltinRoundingMode::Expand => true,
        TemporalBuiltinRoundingMode::Floor | TemporalBuiltinRoundingMode::Trunc => false,
        TemporalBuiltinRoundingMode::HalfCeil | TemporalBuiltinRoundingMode::HalfExpand => {
            remainder * 2 >= unit
        }
        TemporalBuiltinRoundingMode::HalfFloor | TemporalBuiltinRoundingMode::HalfTrunc => {
            remainder * 2 > unit
        }
        TemporalBuiltinRoundingMode::HalfEven => {
            let doubled = remainder * 2;
            doubled > unit || (doubled == unit && lower.rem_euclid(2) != 0)
        }
    }
}

pub(in crate::public::dispatch::temporal) const fn temporal_duration_should_round_negative_remainder_away(
    lower_magnitude: i128,
    remainder: i128,
    unit: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
) -> bool {
    if remainder == 0 {
        return false;
    }
    match rounding_mode {
        TemporalBuiltinRoundingMode::Floor | TemporalBuiltinRoundingMode::Expand => true,
        TemporalBuiltinRoundingMode::Ceil | TemporalBuiltinRoundingMode::Trunc => false,
        TemporalBuiltinRoundingMode::HalfFloor | TemporalBuiltinRoundingMode::HalfExpand => {
            remainder * 2 >= unit
        }
        TemporalBuiltinRoundingMode::HalfCeil | TemporalBuiltinRoundingMode::HalfTrunc => {
            remainder * 2 > unit
        }
        TemporalBuiltinRoundingMode::HalfEven => {
            let doubled = remainder * 2;
            doubled > unit || (doubled == unit && (lower_magnitude + 1).rem_euclid(2) == 0)
        }
    }
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_relative_start_plain_date<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    relative_to: TemporalDurationRelativeTo,
) -> Result<TemporalPlainDateObjectData, Cx::Error> {
    match relative_to {
        TemporalDurationRelativeTo::PlainDate(date) => Ok(date),
        TemporalDurationRelativeTo::PlainDateTime(date_time) => {
            Ok(TemporalPlainDateObjectData::new(
                date_time.year(),
                date_time.month(),
                date_time.day(),
                date_time.calendar(),
            ))
        }
        TemporalDurationRelativeTo::ZonedDateTime(zoned) => {
            let civil = temporal_zoned_date_time_civil(cx, zoned)?.date_time;
            Ok(TemporalPlainDateObjectData::new(
                civil.year,
                civil.month,
                civil.day,
                zoned.calendar(),
            ))
        }
    }
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_relative_start_plain_date_time<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    relative_to: TemporalDurationRelativeTo,
) -> Result<TemporalPlainDateTimeObjectData, Cx::Error> {
    match relative_to {
        TemporalDurationRelativeTo::PlainDate(date) => Ok(TemporalPlainDateTimeObjectData::new(
            date.year(),
            date.month(),
            date.day(),
            0,
            0,
            0,
            0,
            0,
            0,
            date.calendar(),
        )),
        TemporalDurationRelativeTo::PlainDateTime(date_time) => Ok(date_time),
        TemporalDurationRelativeTo::ZonedDateTime(zoned) => {
            let civil = temporal_zoned_date_time_civil(cx, zoned)?.date_time;
            Ok(TemporalPlainDateTimeObjectData::new(
                civil.year,
                civil.month,
                civil.day,
                civil.hour,
                civil.minute,
                civil.second,
                civil.millisecond,
                civil.microsecond,
                civil.nanosecond,
                zoned.calendar(),
            ))
        }
    }
}

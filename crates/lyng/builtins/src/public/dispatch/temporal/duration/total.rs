use super::{
    range_error, temporal_duration_calendar_day_time_difference,
    temporal_duration_calendar_unit_to_date_difference_unit, temporal_duration_data,
    temporal_duration_date_with_start_time_total_nanoseconds,
    temporal_duration_exact_unit_nanoseconds, temporal_duration_relative_start_plain_date_time,
    temporal_duration_relative_total_nanoseconds, temporal_duration_sign,
    temporal_duration_total_options, temporal_duration_validate_exact_relative_to_range,
    temporal_i128_as_number, temporal_ops, temporal_plain_date_add_duration,
    temporal_plain_date_time_date, temporal_plain_date_time_time, temporal_plain_time_nanoseconds,
    temporal_total_duration_exact, temporal_total_nanoseconds_as_unit, BuiltinInvocation,
    PublicBuiltinDispatchContext, TemporalBuiltinDurationExactUnit, TemporalDurationCalendarUnit,
    TemporalDurationObjectData, TemporalDurationParsedUnit, TemporalDurationRelativeTo,
    TemporalOverflow, TemporalPlainDateObjectData, TemporalZonedDateTimeObjectData, Value,
    TEMPORAL_NANOS_PER_DAY,
};

pub(in crate::public::dispatch::temporal) fn temporal_duration_total_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    let options = temporal_duration_total_options(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let total = match options.unit {
        TemporalDurationParsedUnit::Exact(unit) => {
            if unit == TemporalBuiltinDurationExactUnit::Day
                && let Some(TemporalDurationRelativeTo::ZonedDateTime(relative_to)) =
                    options.relative_to
            {
                temporal_duration_validate_zoned_relative_total_day_boundary(cx, relative_to)?;
            }
            if let Some(relative_to) = options.relative_to {
                temporal_duration_validate_exact_relative_to_range(cx, data, relative_to)?;
            }
            if temporal_ops::duration_has_calendar_relative_units(data) {
                let Some(relative_to) = options.relative_to else {
                    return Err(range_error(cx));
                };
                let total_nanoseconds =
                    temporal_duration_relative_total_nanoseconds(cx, data, relative_to)?;
                temporal_total_nanoseconds_as_unit(
                    total_nanoseconds,
                    temporal_duration_exact_unit_nanoseconds(unit),
                )
                .ok_or_else(|| range_error(cx))?
            } else {
                temporal_total_duration_exact(data, unit)
                    .filter(|value| value.is_finite())
                    .ok_or_else(|| range_error(cx))?
            }
        }
        TemporalDurationParsedUnit::CalendarRelative(unit) => {
            if temporal_duration_sign(data) == 0 && options.relative_to.is_some() {
                0.0
            } else {
                let Some(relative_to) = options.relative_to else {
                    return Err(range_error(cx));
                };
                temporal_duration_total_calendar_relative(cx, data, relative_to, unit)?
            }
        }
    };
    Ok(Value::from_f64(total))
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_validate_zoned_relative_total_day_boundary<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    relative_to: TemporalZonedDateTimeObjectData,
) -> Result<(), Cx::Error> {
    let latest_start = temporal_ops::INSTANT_EPOCH_NANOSECONDS_MAX
        .checked_sub(TEMPORAL_NANOS_PER_DAY)
        .ok_or_else(|| range_error(cx))?;
    if relative_to.epoch_nanoseconds() > latest_start {
        return Err(range_error(cx));
    }
    Ok(())
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_total_calendar_relative<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    duration: TemporalDurationObjectData,
    relative_to: TemporalDurationRelativeTo,
    unit: TemporalDurationCalendarUnit,
) -> Result<f64, Cx::Error> {
    let largest_unit = temporal_duration_calendar_unit_to_date_difference_unit(unit);
    let total_nanoseconds =
        temporal_duration_relative_total_nanoseconds(cx, duration, relative_to)?;
    let (date, _) = temporal_duration_calendar_day_time_difference(
        cx,
        relative_to,
        total_nanoseconds,
        largest_unit,
    )?;
    let start = temporal_duration_relative_start_plain_date_time(cx, relative_to)?;
    let start_date = temporal_plain_date_time_date(start);
    let start_time = temporal_plain_time_nanoseconds(temporal_plain_date_time_time(start));
    let start_total =
        temporal_duration_date_with_start_time_total_nanoseconds(cx, start_date, start_time)?;
    let end_total = start_total
        .checked_add(total_nanoseconds)
        .ok_or_else(|| range_error(cx))?;

    let whole_units = match unit {
        TemporalDurationCalendarUnit::Year => date.years(),
        TemporalDurationCalendarUnit::Month => date
            .years()
            .checked_mul(12)
            .and_then(|years| years.checked_add(date.months()))
            .ok_or_else(|| range_error(cx))?,
        TemporalDurationCalendarUnit::Week => date.weeks(),
    };
    let boundary = temporal_duration_calendar_total_boundary_nanoseconds(
        cx,
        start_date,
        start_time,
        unit,
        whole_units,
    )?;
    let remainder = end_total
        .checked_sub(boundary)
        .ok_or_else(|| range_error(cx))?;
    if remainder == 0 {
        return Ok(temporal_i128_as_number(whole_units));
    }
    let adjacent_units = if remainder < 0 {
        whole_units.checked_sub(1)
    } else {
        whole_units.checked_add(1)
    }
    .ok_or_else(|| range_error(cx))?;
    let adjacent = temporal_duration_calendar_total_boundary_nanoseconds(
        cx,
        start_date,
        start_time,
        unit,
        adjacent_units,
    )?;
    let unit_nanoseconds = adjacent
        .checked_sub(boundary)
        .map(i128::abs)
        .ok_or_else(|| range_error(cx))?;
    let total_units = whole_units
        .checked_mul(unit_nanoseconds)
        .and_then(|whole| whole.checked_add(remainder))
        .ok_or_else(|| range_error(cx))?;
    temporal_total_nanoseconds_as_unit(total_units, unit_nanoseconds).ok_or_else(|| range_error(cx))
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_calendar_total_boundary_nanoseconds<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    start_date: TemporalPlainDateObjectData,
    start_time: i128,
    unit: TemporalDurationCalendarUnit,
    units: i128,
) -> Result<i128, Cx::Error> {
    let units = i64::try_from(units).map_err(|_| range_error(cx))?;
    let duration = match unit {
        TemporalDurationCalendarUnit::Year => {
            TemporalDurationObjectData::new(units, 0, 0, 0, 0, 0, 0, 0, 0, 0)
        }
        TemporalDurationCalendarUnit::Month => {
            TemporalDurationObjectData::new(0, units, 0, 0, 0, 0, 0, 0, 0, 0)
        }
        TemporalDurationCalendarUnit::Week => {
            TemporalDurationObjectData::new(0, 0, units, 0, 0, 0, 0, 0, 0, 0)
        }
    };
    let boundary =
        temporal_plain_date_add_duration(cx, start_date, duration, TemporalOverflow::Constrain)?;
    temporal_duration_date_with_start_time_total_nanoseconds(cx, boundary, start_time)
}

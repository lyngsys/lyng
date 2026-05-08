use super::{
    allocate_current_temporal_blank_duration_object, allocate_temporal_duration_object,
    current_temporal_duration_prototype, range_error, temporal_date_difference_unit_order,
    temporal_date_time_difference_unit_from_duration_exact, temporal_duration_data,
    temporal_duration_default_largest_exact_unit,
    temporal_duration_exact_unit_allows_largest_smallest, temporal_duration_exact_unit_nanoseconds,
    temporal_duration_from_date_time_nanoseconds, temporal_duration_relative_total_nanoseconds,
    temporal_duration_round_calendar_largest_unit, temporal_duration_round_calendar_relative,
    temporal_duration_round_calendar_relative_exact,
    temporal_duration_round_calendar_smallest_unit, temporal_duration_round_options,
    temporal_duration_rounding_increment_is_valid, temporal_duration_sign,
    temporal_duration_time_nanoseconds, temporal_ops, temporal_plain_date_add_duration,
    temporal_plain_date_from_parts, temporal_plain_date_time_add_duration,
    temporal_plain_date_time_date, temporal_plain_date_time_is_within_limits,
    temporal_plain_date_time_total_nanoseconds, temporal_round_duration_exact,
    temporal_round_duration_nanoseconds_to_increment, temporal_zoned_date_time_add_duration,
    temporal_zoned_date_time_civil, validate_temporal_duration, BuiltinInvocation,
    PublicBuiltinDispatchContext, TemporalBuiltinDurationExactUnit, TemporalBuiltinRoundingMode,
    TemporalDateTimeDifferenceUnit, TemporalDurationObjectData, TemporalDurationParsedLargestUnit,
    TemporalDurationParsedUnit, TemporalDurationRelativeTo, TemporalOverflow,
    TemporalPlainDateObjectData, TemporalPlainDateTimeObjectData, TemporalZonedDateTimeObjectData,
    Value,
};

#[allow(
    clippy::too_many_lines,
    reason = "Temporal.Duration round follows the option parsing and balancing steps in spec order"
)]
pub(in crate::public::dispatch::temporal) fn temporal_duration_round_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    let options = temporal_duration_round_options(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;

    if temporal_ops::duration_has_calendar_relative_units(data)
        && matches!(
            options.largest_unit,
            TemporalDurationParsedLargestUnit::Missing
        )
        && options.smallest_unit.is_none()
    {
        return Err(range_error(cx));
    }

    if let Some(largest_unit) = temporal_duration_round_calendar_largest_unit(
        data,
        options.largest_unit,
        options.smallest_unit,
    ) {
        if temporal_duration_sign(data) == 0 {
            return allocate_current_temporal_blank_duration_object(cx);
        }
        let Some(relative_to) = options.relative_to else {
            return Err(range_error(cx));
        };
        if let Some(TemporalDurationParsedUnit::Exact(smallest_exact_unit)) = options.smallest_unit
            && smallest_exact_unit != TemporalBuiltinDurationExactUnit::Day
        {
            if !temporal_duration_rounding_increment_is_valid(
                smallest_exact_unit,
                options.rounding_increment,
            ) {
                return Err(range_error(cx));
            }
            let rounded = temporal_duration_round_calendar_relative_exact(
                cx,
                data,
                relative_to,
                largest_unit,
                smallest_exact_unit,
                options.rounding_increment,
                options.rounding_mode,
            )?;
            validate_temporal_duration(cx, rounded)?;
            let prototype = current_temporal_duration_prototype(cx)?;
            return allocate_temporal_duration_object(cx, prototype, rounded);
        }
        let smallest_unit = temporal_duration_round_calendar_smallest_unit(
            cx,
            options.smallest_unit,
            largest_unit,
        )?;
        if temporal_date_difference_unit_order(largest_unit)
            > temporal_date_difference_unit_order(smallest_unit)
        {
            return Err(range_error(cx));
        }
        if options.rounding_increment != 1
            && temporal_date_difference_unit_order(largest_unit)
                < temporal_date_difference_unit_order(smallest_unit)
        {
            return Err(range_error(cx));
        }
        let rounded = temporal_duration_round_calendar_relative(
            cx,
            data,
            relative_to,
            largest_unit,
            smallest_unit,
            options.rounding_increment,
            options.rounding_mode,
        )?;
        validate_temporal_duration(cx, rounded)?;
        let prototype = current_temporal_duration_prototype(cx)?;
        return allocate_temporal_duration_object(cx, prototype, rounded);
    }

    let smallest_unit = match options.smallest_unit {
        Some(TemporalDurationParsedUnit::Exact(unit)) => unit,
        Some(TemporalDurationParsedUnit::CalendarRelative(_)) => return Err(range_error(cx)),
        None => TemporalBuiltinDurationExactUnit::Nanosecond,
    };
    let largest_unit = match options.largest_unit {
        TemporalDurationParsedLargestUnit::Missing | TemporalDurationParsedLargestUnit::Auto => {
            temporal_duration_default_largest_exact_unit(data, smallest_unit)
        }
        TemporalDurationParsedLargestUnit::Exact(unit) => unit,
        TemporalDurationParsedLargestUnit::CalendarRelative(_) => {
            if temporal_duration_sign(data) == 0 && options.relative_to.is_some() {
                return allocate_current_temporal_blank_duration_object(cx);
            }
            return Err(range_error(cx));
        }
    };

    if !temporal_duration_exact_unit_allows_largest_smallest(largest_unit, smallest_unit)
        || !temporal_duration_rounding_increment_is_valid(smallest_unit, options.rounding_increment)
    {
        return Err(range_error(cx));
    }
    if matches!(
        options.relative_to,
        Some(TemporalDurationRelativeTo::ZonedDateTime(_))
    ) {
        let Some(TemporalDurationRelativeTo::ZonedDateTime(relative_to)) = options.relative_to
        else {
            unreachable!("checked above")
        };
        temporal_duration_validate_zoned_relative_day_rounding_boundary(
            cx,
            relative_to,
            largest_unit,
            smallest_unit,
        )?;
        if largest_unit == TemporalBuiltinDurationExactUnit::Day
            && smallest_unit == TemporalBuiltinDurationExactUnit::Day
        {
            let _ = temporal_zoned_date_time_add_duration(
                cx,
                relative_to,
                data,
                TemporalOverflow::Constrain,
            )?;
        }
        if let Some(data) = temporal_duration_round_zoned_relative_exact_remainder(
            cx,
            data,
            largest_unit,
            smallest_unit,
            options.rounding_increment,
            options.rounding_mode,
        )? {
            validate_temporal_duration(cx, data)?;
            let prototype = current_temporal_duration_prototype(cx)?;
            return allocate_temporal_duration_object(cx, prototype, data);
        }
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
        let increment = temporal_duration_exact_unit_nanoseconds(smallest_unit)
            .checked_mul(options.rounding_increment)
            .ok_or_else(|| range_error(cx))?;
        let rounded = temporal_round_duration_nanoseconds_to_increment(
            total_nanoseconds,
            increment,
            options.rounding_mode,
        )
        .ok_or_else(|| range_error(cx))?;
        let data = temporal_duration_from_date_time_nanoseconds(
            cx,
            rounded,
            temporal_date_time_difference_unit_from_duration_exact(largest_unit),
        )?;
        validate_temporal_duration(cx, data)?;
        let prototype = current_temporal_duration_prototype(cx)?;
        return allocate_temporal_duration_object(cx, prototype, data);
    }
    let data = temporal_round_duration_exact(
        data,
        largest_unit,
        smallest_unit,
        options.rounding_increment,
        options.rounding_mode,
    )
    .ok_or_else(|| range_error(cx))?;
    validate_temporal_duration(cx, data)?;
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(cx, prototype, data)
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_validate_exact_relative_to_range<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    duration: TemporalDurationObjectData,
    relative_to: TemporalDurationRelativeTo,
) -> Result<(), Cx::Error> {
    let sign = temporal_duration_sign(duration);
    if sign == 0 {
        return Ok(());
    }
    match relative_to {
        TemporalDurationRelativeTo::ZonedDateTime(relative_to) => {
            let _ = temporal_zoned_date_time_add_duration(
                cx,
                relative_to,
                duration,
                TemporalOverflow::Constrain,
            )?;
        }
        TemporalDurationRelativeTo::PlainDate(date) => {
            if sign > 0 && temporal_duration_is_min_plain_date(date) {
                return Err(range_error(cx));
            }
            let start = TemporalPlainDateTimeObjectData::new(
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
            );
            temporal_duration_validate_plain_date_time_limit(cx, start)?;
            let end = temporal_plain_date_time_add_duration(
                cx,
                start,
                duration,
                TemporalOverflow::Constrain,
            )?;
            temporal_duration_validate_plain_date_time_limit(cx, end)?;
        }
        TemporalDurationRelativeTo::PlainDateTime(start) => {
            if sign > 0 && temporal_duration_is_min_plain_date(temporal_plain_date_time_date(start))
            {
                return Err(range_error(cx));
            }
            temporal_duration_validate_plain_date_time_limit(cx, start)?;
            let end = temporal_plain_date_time_add_duration(
                cx,
                start,
                duration,
                TemporalOverflow::Constrain,
            )?;
            temporal_duration_validate_plain_date_time_limit(cx, end)?;
        }
    }
    Ok(())
}

pub(in crate::public::dispatch::temporal) const fn temporal_duration_is_min_plain_date(
    date: TemporalPlainDateObjectData,
) -> bool {
    date.year() == -271_821 && date.month() == 4 && date.day() == 19
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_validate_plain_date_time_limit<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    date_time: TemporalPlainDateTimeObjectData,
) -> Result<(), Cx::Error> {
    let total_nanoseconds =
        temporal_plain_date_time_total_nanoseconds(date_time).ok_or_else(|| range_error(cx))?;
    if !temporal_plain_date_time_is_within_limits(date_time.calendar(), total_nanoseconds) {
        return Err(range_error(cx));
    }
    Ok(())
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_validate_zoned_relative_day_rounding_boundary<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    relative_to: TemporalZonedDateTimeObjectData,
    largest_unit: TemporalBuiltinDurationExactUnit,
    smallest_unit: TemporalBuiltinDurationExactUnit,
) -> Result<(), Cx::Error> {
    if largest_unit != TemporalBuiltinDurationExactUnit::Day
        || smallest_unit == TemporalBuiltinDurationExactUnit::Day
    {
        return Ok(());
    }

    let civil = temporal_zoned_date_time_civil(cx, relative_to)?.date_time;
    let date = temporal_plain_date_from_parts(
        cx,
        i64::from(civil.year),
        i64::from(civil.month),
        i64::from(civil.day),
    )?;
    let _ = temporal_plain_date_add_duration(
        cx,
        date,
        TemporalDurationObjectData::new(0, 0, 0, 1, 0, 0, 0, 0, 0, 0),
        TemporalOverflow::Constrain,
    )?;
    Ok(())
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_round_zoned_relative_exact_remainder<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    data: TemporalDurationObjectData,
    largest_unit: TemporalBuiltinDurationExactUnit,
    smallest_unit: TemporalBuiltinDurationExactUnit,
    rounding_increment: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
) -> Result<Option<TemporalDurationObjectData>, Cx::Error> {
    if temporal_ops::duration_has_calendar_relative_units(data)
        || data.days() == 0
        || largest_unit != TemporalBuiltinDurationExactUnit::Day
        || smallest_unit == TemporalBuiltinDurationExactUnit::Day
    {
        return Ok(None);
    }
    let increment = temporal_duration_exact_unit_nanoseconds(smallest_unit)
        .checked_mul(rounding_increment)
        .ok_or_else(|| range_error(cx))?;
    let rounded_time = temporal_round_duration_nanoseconds_to_increment(
        temporal_duration_time_nanoseconds(data),
        increment,
        rounding_mode,
    )
    .ok_or_else(|| range_error(cx))?;
    let time = temporal_duration_from_date_time_nanoseconds(
        cx,
        rounded_time,
        TemporalDateTimeDifferenceUnit::Day,
    )?;
    let days = data
        .days()
        .checked_add(time.days())
        .ok_or_else(|| range_error(cx))?;
    Ok(Some(TemporalDurationObjectData::new(
        0,
        0,
        0,
        i64::try_from(days).map_err(|_| range_error(cx))?,
        time.hours(),
        time.minutes(),
        time.seconds(),
        time.milliseconds(),
        time.microseconds(),
        time.nanoseconds(),
    )))
}

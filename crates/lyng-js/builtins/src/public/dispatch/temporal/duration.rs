use super::{
    current_temporal_duration_prototype, format_temporal_duration,
    format_temporal_duration_with_seconds_precision, map_completion, negate_temporal_duration,
    object, parse_temporal_duration, range_error, string_ref_text, string_value,
    temporal_compare_ordering, temporal_constructor_prototype, temporal_date_difference_unit_order,
    temporal_duration_default_largest_exact_unit,
    temporal_duration_exact_unit_allows_largest_smallest,
    temporal_duration_from_date_time_nanoseconds, temporal_duration_from_date_units,
    temporal_duration_from_nanoseconds_with_largest_unit, temporal_duration_sign,
    temporal_duration_time_nanoseconds, temporal_i128_as_number, temporal_integer_part_from_value,
    temporal_month_from_month_code_value, temporal_number_to_i128_after_range_check,
    temporal_number_to_u8_after_range_check, temporal_ops, temporal_parse_offset_string,
    temporal_plain_date_add_duration, temporal_plain_date_difference_trunc,
    temporal_plain_date_from_parts, temporal_plain_date_from_value,
    temporal_plain_date_ordinal_day, temporal_plain_date_time_add_duration,
    temporal_plain_date_time_date, temporal_plain_date_time_from_parts_with_overflow,
    temporal_plain_date_time_from_total_nanoseconds, temporal_plain_date_time_from_value,
    temporal_plain_date_time_is_within_limits, temporal_plain_date_time_time,
    temporal_plain_date_time_total_nanoseconds, temporal_plain_time_nanoseconds,
    temporal_property_value, temporal_round_duration_exact,
    temporal_round_duration_nanoseconds_to_increment, temporal_round_i128_to_increment,
    temporal_time_part_from_value, temporal_time_zone_id_from_value, temporal_total_duration_exact,
    temporal_total_nanoseconds_as_unit, temporal_validate_iso_calendar_value,
    temporal_zoned_date_time_add_duration, temporal_zoned_date_time_civil,
    temporal_zoned_date_time_data, temporal_zoned_date_time_explicit_offset,
    temporal_zoned_date_time_from_parts, temporal_zoned_date_time_from_value,
    temporal_zoned_date_time_zone_annotation, to_number_for_builtin, to_string_string_ref,
    type_error, AllocationLifetime, BuiltinFunctionId, BuiltinInvocation, ObjectAllocation,
    ObjectColdData, ObjectRef, OrdinaryObjectData, PublicBuiltinDispatchContext,
    TemporalBuiltinDurationExactUnit, TemporalBuiltinRoundingMode, TemporalCivilDateTime,
    TemporalCivilToInstantRequest, TemporalDateDifferenceUnit, TemporalDateTimeDifferenceUnit,
    TemporalDisambiguation, TemporalDurationObjectData, TemporalObjectData, TemporalObjectKind,
    TemporalOverflow, TemporalPlainDateObjectData, TemporalPlainDateTimeObjectData,
    TemporalZonedDateTimeObjectData, Value, TEMPORAL_NANOS_PER_DAY, TEMPORAL_NANOS_PER_HOUR,
    TEMPORAL_NANOS_PER_MICROSECOND, TEMPORAL_NANOS_PER_MILLISECOND, TEMPORAL_NANOS_PER_MINUTE,
    TEMPORAL_NANOS_PER_SECOND, TEMPORAL_SAFE_INTEGER_MAX,
};

pub(super) fn dispatch_temporal_duration_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::temporal_duration_builtin() {
        return temporal_duration_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_years_getter_builtin() {
        return temporal_duration_years_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_months_getter_builtin() {
        return temporal_duration_months_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_weeks_getter_builtin() {
        return temporal_duration_weeks_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_days_getter_builtin() {
        return temporal_duration_days_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_hours_getter_builtin() {
        return temporal_duration_hours_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_minutes_getter_builtin() {
        return temporal_duration_minutes_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_seconds_getter_builtin() {
        return temporal_duration_seconds_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_milliseconds_getter_builtin() {
        return temporal_duration_milliseconds_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_microseconds_getter_builtin() {
        return temporal_duration_microseconds_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_nanoseconds_getter_builtin() {
        return temporal_duration_nanoseconds_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_sign_getter_builtin() {
        return temporal_duration_sign_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_blank_getter_builtin() {
        return temporal_duration_blank_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_to_string_builtin() {
        return temporal_duration_to_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_to_json_builtin() {
        return temporal_duration_to_json_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_to_locale_string_builtin() {
        return temporal_duration_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_negated_builtin() {
        return temporal_duration_negated_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_abs_builtin() {
        return temporal_duration_abs_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_with_builtin() {
        return temporal_duration_with_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_round_builtin() {
        return temporal_duration_round_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_total_builtin() {
        return temporal_duration_total_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_add_builtin() {
        return temporal_duration_add_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_subtract_builtin() {
        return temporal_duration_subtract_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_value_of_builtin() {
        return temporal_duration_value_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_from_builtin() {
        return temporal_duration_from_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_compare_builtin() {
        return temporal_duration_compare_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn allocate_current_temporal_blank_duration_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<Value, Cx::Error> {
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(
        cx,
        prototype,
        TemporalDurationObjectData::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
    )
}

fn temporal_duration_data<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    this_value: Value,
) -> Result<TemporalDurationObjectData, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, this_value, TemporalObjectKind::Duration)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::Duration(data) = payload else {
        return Err(type_error(cx));
    };
    Ok(data)
}

fn temporal_duration_component_getter<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    component: fn(TemporalDurationObjectData) -> i128,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    Ok(temporal_i128_to_number_value(component(data)))
}

fn temporal_duration_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let Some(new_target) = invocation.new_target() else {
        return Err(type_error(cx));
    };
    let data = TemporalDurationObjectData::new(
        temporal_duration_part_from_argument(cx, invocation, 0)?,
        temporal_duration_part_from_argument(cx, invocation, 1)?,
        temporal_duration_part_from_argument(cx, invocation, 2)?,
        temporal_duration_part_from_argument(cx, invocation, 3)?,
        temporal_duration_part_from_argument(cx, invocation, 4)?,
        temporal_duration_part_from_argument(cx, invocation, 5)?,
        temporal_duration_part_from_argument(cx, invocation, 6)?,
        temporal_duration_part_from_argument(cx, invocation, 7)?,
        temporal_duration_part_from_argument(cx, invocation, 8)?,
        temporal_duration_part_from_argument(cx, invocation, 9)?,
    );
    validate_temporal_duration(cx, data)?;
    let prototype = temporal_constructor_prototype(cx, new_target)?;
    allocate_temporal_duration_object(cx, prototype, data)
}

fn temporal_duration_years_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::years)
}

fn temporal_duration_months_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::months)
}

fn temporal_duration_weeks_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::weeks)
}

fn temporal_duration_days_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::days)
}

fn temporal_duration_hours_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::hours)
}

fn temporal_duration_minutes_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::minutes)
}

fn temporal_duration_seconds_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::seconds)
}

fn temporal_duration_milliseconds_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::milliseconds)
}

fn temporal_duration_microseconds_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::microseconds)
}

fn temporal_duration_nanoseconds_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::nanoseconds)
}

fn temporal_duration_sign_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(temporal_duration_sign(data)))
}

fn temporal_duration_blank_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    Ok(Value::from_bool(temporal_duration_sign(data) == 0))
}

fn temporal_duration_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    let (fractional_digits, rounding_mode) = temporal_duration_to_string_options(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let text = if let Some(digits) = fractional_digits {
        format_temporal_duration_with_seconds_precision(data, digits, rounding_mode)
            .ok_or_else(|| range_error(cx))?
    } else {
        format_temporal_duration(data)
    };
    Ok(string_value(cx, &text))
}

fn temporal_duration_to_json_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format_temporal_duration(data)))
}

fn temporal_duration_to_locale_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format_temporal_duration(data)))
}

fn temporal_duration_negated_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = negate_temporal_duration(temporal_duration_data(cx, invocation.this_value())?);
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(cx, prototype, data)
}

fn temporal_duration_abs_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    let data = if temporal_duration_sign(data) < 0 {
        negate_temporal_duration(data)
    } else {
        data
    };
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(cx, prototype, data)
}

fn temporal_duration_with_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let base = temporal_duration_data(cx, invocation.this_value())?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(object_ref) = value.as_object_ref() else {
        return Err(type_error(cx));
    };

    let days = temporal_optional_duration_part_from_property(cx, object_ref, "days")?;
    let hours = temporal_optional_duration_part_from_property(cx, object_ref, "hours")?;
    let microseconds =
        temporal_optional_duration_part_from_property(cx, object_ref, "microseconds")?;
    let milliseconds =
        temporal_optional_duration_part_from_property(cx, object_ref, "milliseconds")?;
    let minutes = temporal_optional_duration_part_from_property(cx, object_ref, "minutes")?;
    let months = temporal_optional_duration_part_from_property(cx, object_ref, "months")?;
    let nanoseconds = temporal_optional_duration_part_from_property(cx, object_ref, "nanoseconds")?;
    let seconds = temporal_optional_duration_part_from_property(cx, object_ref, "seconds")?;
    let weeks = temporal_optional_duration_part_from_property(cx, object_ref, "weeks")?;
    let years = temporal_optional_duration_part_from_property(cx, object_ref, "years")?;

    if [
        days,
        hours,
        microseconds,
        milliseconds,
        minutes,
        months,
        nanoseconds,
        seconds,
        weeks,
        years,
    ]
    .iter()
    .all(Option::is_none)
    {
        return Err(type_error(cx));
    }

    let data = TemporalDurationObjectData::new(
        years.unwrap_or_else(|| base.years()),
        months.unwrap_or_else(|| base.months()),
        weeks.unwrap_or_else(|| base.weeks()),
        days.unwrap_or_else(|| base.days()),
        hours.unwrap_or_else(|| base.hours()),
        minutes.unwrap_or_else(|| base.minutes()),
        seconds.unwrap_or_else(|| base.seconds()),
        milliseconds.unwrap_or_else(|| base.milliseconds()),
        microseconds.unwrap_or_else(|| base.microseconds()),
        nanoseconds.unwrap_or_else(|| base.nanoseconds()),
    );
    validate_temporal_duration(cx, data)?;
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(cx, prototype, data)
}

#[allow(
    clippy::too_many_lines,
    reason = "Temporal.Duration round follows the option parsing and balancing steps in spec order"
)]
fn temporal_duration_round_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_duration_validate_exact_relative_to_range<Cx: PublicBuiltinDispatchContext>(
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

const fn temporal_duration_is_min_plain_date(date: TemporalPlainDateObjectData) -> bool {
    date.year() == -271_821 && date.month() == 4 && date.day() == 19
}

fn temporal_duration_validate_plain_date_time_limit<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_duration_validate_zoned_relative_day_rounding_boundary<
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

fn temporal_duration_round_zoned_relative_exact_remainder<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_duration_total_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_duration_validate_zoned_relative_total_day_boundary<
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

fn temporal_duration_total_calendar_relative<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_duration_calendar_total_boundary_nanoseconds<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_duration_add_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_additive_builtin(
        cx,
        invocation,
        temporal_ops::add_durations_with_largest_unit,
    )
}

fn temporal_duration_subtract_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_additive_builtin(
        cx,
        invocation,
        temporal_ops::subtract_durations_with_largest_unit,
    )
}

fn temporal_duration_additive_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    operation: fn(
        TemporalDurationObjectData,
        TemporalDurationObjectData,
        temporal_ops::TemporalDurationExactUnit,
    ) -> Option<TemporalDurationObjectData>,
) -> Result<Value, Cx::Error> {
    let base = temporal_duration_data(cx, invocation.this_value())?;
    let other = temporal_duration_from_additive_argument_with_largest_unit(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if temporal_ops::duration_has_calendar_relative_units(base)
        || temporal_ops::duration_has_calendar_relative_units(other.data)
    {
        return Err(range_error(cx));
    }
    let largest_unit =
        temporal_ops::duration_largest_exact_unit_for_addition(base, other.largest_unit);
    let data = operation(base, other.data, largest_unit).ok_or_else(|| range_error(cx))?;
    validate_temporal_duration(cx, data)?;
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(cx, prototype, data)
}

fn temporal_duration_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Err(type_error(cx))
}

fn temporal_duration_from_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(cx, prototype, data)
}

fn temporal_duration_compare_relative_to_option<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Option<TemporalDurationRelativeTo>, Cx::Error> {
    if value.is_undefined() {
        return Ok(None);
    }
    let Some(object_ref) = value.as_object_ref() else {
        return Err(type_error(cx));
    };
    let relative_to = temporal_property_value(cx, object_ref, "relativeTo")?;
    temporal_duration_relative_to_option(cx, relative_to)
}

fn temporal_duration_compare_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = temporal_duration_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let right = temporal_duration_from_value(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let relative_to = temporal_duration_compare_relative_to_option(
        cx,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if temporal_ops::durations_are_equal(left, right) {
        return Ok(Value::from_smi(0));
    }
    let has_calendar_relative_units = temporal_ops::duration_has_calendar_relative_units(left)
        || temporal_ops::duration_has_calendar_relative_units(right);
    let has_date_units =
        temporal_ops::duration_has_date_units(left) || temporal_ops::duration_has_date_units(right);
    if has_date_units {
        let Some(relative_to) = relative_to else {
            if has_calendar_relative_units {
                return Err(range_error(cx));
            }
            let ordering =
                temporal_ops::compare_time_duration(left, right).ok_or_else(|| range_error(cx))?;
            return Ok(temporal_compare_ordering(ordering));
        };
        temporal_duration_validate_exact_relative_to_range(cx, left, relative_to)?;
        temporal_duration_validate_exact_relative_to_range(cx, right, relative_to)?;
        let left_total = temporal_duration_relative_total_nanoseconds(cx, left, relative_to)?;
        let right_total = temporal_duration_relative_total_nanoseconds(cx, right, relative_to)?;
        return Ok(temporal_compare_ordering(left_total.cmp(&right_total)));
    }
    let ordering =
        temporal_ops::compare_time_duration(left, right).ok_or_else(|| range_error(cx))?;
    Ok(temporal_compare_ordering(ordering))
}

pub(super) fn temporal_i128_to_number_value(value: i128) -> Value {
    i32::try_from(value).map_or_else(
        |_| Value::from_f64(temporal_i128_as_number(value)),
        Value::from_smi,
    )
}

#[allow(
    clippy::float_cmp,
    reason = "Temporal numeric conversion requires exact integral-number validation."
)]
pub(super) fn temporal_duration_part_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<i128, Cx::Error> {
    if value.is_undefined() {
        return Ok(0);
    }
    let number = to_number_for_builtin(cx, value)?;
    if !number.is_finite() || number.trunc() != number {
        return Err(range_error(cx));
    }
    let max = temporal_i128_as_number(TEMPORAL_SAFE_INTEGER_MAX);
    if !(-max..=max).contains(&number) {
        return Err(range_error(cx));
    }
    Ok(temporal_number_to_i128_after_range_check(number))
}

#[allow(
    clippy::float_cmp,
    reason = "Temporal numeric conversion requires exact integral-number validation."
)]
pub(super) fn temporal_duration_part_i128_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<i128, Cx::Error> {
    if value.is_undefined() {
        return Ok(0);
    }
    let number = to_number_for_builtin(cx, value)?;
    if !number.is_finite() || number.trunc() != number {
        return Err(range_error(cx));
    }
    if number < temporal_i128_as_number(i128::MIN) || number > temporal_i128_as_number(i128::MAX) {
        return Err(range_error(cx));
    }
    Ok(temporal_number_to_i128_after_range_check(number))
}

pub(super) fn temporal_duration_part_from_argument<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    index: usize,
) -> Result<i128, Cx::Error> {
    temporal_duration_part_from_value(
        cx,
        invocation
            .arguments()
            .get(index)
            .copied()
            .unwrap_or(Value::undefined()),
    )
}

pub(super) fn temporal_optional_duration_part_from_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    property_name: &str,
) -> Result<Option<i128>, Cx::Error> {
    let value = temporal_property_value(cx, object_ref, property_name)?;
    if value.is_undefined() {
        return Ok(None);
    }
    temporal_duration_part_from_value(cx, value).map(Some)
}

pub(super) fn temporal_duration_part_i128_from_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    property_name: &str,
) -> Result<(bool, i128), Cx::Error> {
    let value = temporal_property_value(cx, object_ref, property_name)?;
    if value.is_undefined() {
        return Ok((false, 0));
    }
    Ok((true, temporal_duration_part_i128_from_value(cx, value)?))
}

pub(super) fn validate_temporal_duration<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    data: TemporalDurationObjectData,
) -> Result<(), Cx::Error> {
    if !temporal_ops::duration_signs_are_balanced(data)
        || !temporal_ops::duration_is_within_limits(data)
    {
        return Err(range_error(cx));
    }
    Ok(())
}

pub(super) fn allocate_temporal_duration_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: ObjectRef,
    data: TemporalDurationObjectData,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let root_shape = cx
        .agent()
        .realm(realm)
        .and_then(|realm| realm.root_shape())
        .ok_or_else(|| type_error(cx))?;
    let object = {
        let agent = cx.agent();
        agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape)
                    .with_prototype(Some(prototype))
                    .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::Temporal(
                        TemporalObjectKind::Duration,
                    ))),
                AllocationLifetime::Default,
            )
        })
    };
    let installed = cx
        .agent()
        .objects_mut()
        .install_temporal_object(object, TemporalObjectData::Duration(data));
    if !installed {
        return Err(type_error(cx));
    }
    Ok(Value::from_object_ref(object))
}

#[derive(Debug, Clone, Copy)]
struct TemporalDurationWithLargestUnit {
    data: TemporalDurationObjectData,
    largest_unit: temporal_ops::TemporalDurationExactUnit,
}

pub(super) fn temporal_duration_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationObjectData, Cx::Error> {
    Ok(temporal_duration_from_value_with_largest_unit(cx, value)?.data)
}

fn temporal_duration_from_value_with_largest_unit<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationWithLargestUnit, Cx::Error> {
    let Some(object_ref) = value.as_object_ref() else {
        if !value.is_string() {
            return Err(type_error(cx));
        }
        let string_ref = to_string_string_ref(cx, value)?;
        let text = string_ref_text(cx, string_ref)?;
        let data = parse_temporal_duration(&text).ok_or_else(|| range_error(cx))?;
        validate_temporal_duration(cx, data)?;
        return Ok(TemporalDurationWithLargestUnit {
            data,
            largest_unit: temporal_ops::duration_largest_exact_unit(data),
        });
    };
    let existing = {
        let agent = cx.agent();
        agent.objects().temporal_object(object_ref).copied()
    };
    if let Some(TemporalObjectData::Duration(data)) = existing {
        return Ok(TemporalDurationWithLargestUnit {
            data,
            largest_unit: temporal_ops::duration_largest_exact_unit(data),
        });
    }

    let (has_days, days) = temporal_duration_part_i128_from_property(cx, object_ref, "days")?;
    let (has_hours, hours) = temporal_duration_part_i128_from_property(cx, object_ref, "hours")?;
    let (has_microseconds, microseconds) =
        temporal_duration_part_i128_from_property(cx, object_ref, "microseconds")?;
    let (has_milliseconds, milliseconds) =
        temporal_duration_part_i128_from_property(cx, object_ref, "milliseconds")?;
    let (has_minutes, minutes) =
        temporal_duration_part_i128_from_property(cx, object_ref, "minutes")?;
    let (has_months, months) = temporal_duration_part_i128_from_property(cx, object_ref, "months")?;
    let (has_nanoseconds, nanoseconds) =
        temporal_duration_part_i128_from_property(cx, object_ref, "nanoseconds")?;
    let (has_seconds, seconds) =
        temporal_duration_part_i128_from_property(cx, object_ref, "seconds")?;
    let (has_weeks, weeks) = temporal_duration_part_i128_from_property(cx, object_ref, "weeks")?;
    let (has_years, years) = temporal_duration_part_i128_from_property(cx, object_ref, "years")?;
    if ![
        has_days,
        has_hours,
        has_microseconds,
        has_milliseconds,
        has_minutes,
        has_months,
        has_nanoseconds,
        has_seconds,
        has_weeks,
        has_years,
    ]
    .iter()
    .any(|present| *present)
    {
        return Err(type_error(cx));
    }
    let largest_unit = temporal_duration_largest_exact_unit_from_raw_parts(
        days,
        hours,
        minutes,
        seconds,
        milliseconds,
        microseconds,
        nanoseconds,
    );

    let data = TemporalDurationObjectData::new(
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
    );
    validate_temporal_duration(cx, data)?;
    Ok(TemporalDurationWithLargestUnit { data, largest_unit })
}

pub(super) fn temporal_duration_from_additive_argument<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationObjectData, Cx::Error> {
    if value.as_object_ref().is_none() && !value.is_string() {
        return Err(type_error(cx));
    }
    temporal_duration_from_value(cx, value)
}

fn temporal_duration_from_additive_argument_with_largest_unit<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationWithLargestUnit, Cx::Error> {
    if value.as_object_ref().is_none() && !value.is_string() {
        return Err(type_error(cx));
    }
    temporal_duration_from_value_with_largest_unit(cx, value)
}

const fn temporal_duration_largest_exact_unit_from_raw_parts(
    days: i128,
    hours: i128,
    minutes: i128,
    seconds: i128,
    milliseconds: i128,
    microseconds: i128,
    _nanoseconds: i128,
) -> temporal_ops::TemporalDurationExactUnit {
    if days != 0 {
        temporal_ops::TemporalDurationExactUnit::Day
    } else if hours != 0 {
        temporal_ops::TemporalDurationExactUnit::Hour
    } else if minutes != 0 {
        temporal_ops::TemporalDurationExactUnit::Minute
    } else if seconds != 0 {
        temporal_ops::TemporalDurationExactUnit::Second
    } else if milliseconds != 0 {
        temporal_ops::TemporalDurationExactUnit::Millisecond
    } else if microseconds != 0 {
        temporal_ops::TemporalDurationExactUnit::Microsecond
    } else {
        temporal_ops::TemporalDurationExactUnit::Nanosecond
    }
}

pub(super) fn temporal_duration_to_string_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<(Option<u8>, TemporalBuiltinRoundingMode), Cx::Error> {
    if value.is_undefined() {
        return Ok((None, TemporalBuiltinRoundingMode::Trunc));
    }
    let Some(object_ref) = value.as_object_ref() else {
        return Err(type_error(cx));
    };

    let fractional_second_digits =
        temporal_property_value(cx, object_ref, "fractionalSecondDigits")?;
    let fractional_digits =
        temporal_duration_fractional_second_digits_option(cx, fractional_second_digits)?;
    let rounding_mode = temporal_property_value(cx, object_ref, "roundingMode")?;
    let rounding_mode = temporal_duration_rounding_mode_option(cx, rounding_mode)?;
    let smallest_unit = temporal_property_value(cx, object_ref, "smallestUnit")?;
    if !smallest_unit.is_undefined() {
        return Ok((
            Some(temporal_duration_smallest_unit_digits(cx, smallest_unit)?),
            rounding_mode,
        ));
    }
    Ok((fractional_digits, rounding_mode))
}

pub(super) fn temporal_duration_smallest_unit_digits<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<u8, Cx::Error> {
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    match text.as_str() {
        "second" | "seconds" => Ok(0),
        "millisecond" | "milliseconds" => Ok(3),
        "microsecond" | "microseconds" => Ok(6),
        "nanosecond" | "nanoseconds" => Ok(9),
        _ => Err(range_error(cx)),
    }
}

pub(super) fn temporal_duration_fractional_second_digits_option<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
) -> Result<Option<u8>, Cx::Error> {
    if value.is_undefined() {
        return Ok(None);
    }
    if let Some(number) = value.as_f64() {
        if !number.is_finite() {
            return Err(range_error(cx));
        }
        let digits = number.floor();
        if !(0.0..=9.0).contains(&digits) {
            return Err(range_error(cx));
        }
        return Ok(Some(temporal_number_to_u8_after_range_check(digits)));
    }
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    if text == "auto" {
        return Ok(None);
    }
    Err(range_error(cx))
}

pub(super) fn temporal_duration_rounding_mode_option<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalBuiltinRoundingMode, Cx::Error> {
    temporal_duration_rounding_mode_option_with_default(
        cx,
        value,
        TemporalBuiltinRoundingMode::Trunc,
    )
}

pub(super) fn temporal_duration_rounding_mode_option_with_default<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
    default: TemporalBuiltinRoundingMode,
) -> Result<TemporalBuiltinRoundingMode, Cx::Error> {
    if value.is_undefined() {
        return Ok(default);
    }
    temporal_duration_rounding_mode_from_value(cx, value)
}

pub(super) fn temporal_duration_rounding_mode_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalBuiltinRoundingMode, Cx::Error> {
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    match text.as_str() {
        "ceil" => Ok(TemporalBuiltinRoundingMode::Ceil),
        "floor" => Ok(TemporalBuiltinRoundingMode::Floor),
        "expand" => Ok(TemporalBuiltinRoundingMode::Expand),
        "trunc" => Ok(TemporalBuiltinRoundingMode::Trunc),
        "halfCeil" => Ok(TemporalBuiltinRoundingMode::HalfCeil),
        "halfFloor" => Ok(TemporalBuiltinRoundingMode::HalfFloor),
        "halfExpand" => Ok(TemporalBuiltinRoundingMode::HalfExpand),
        "halfTrunc" => Ok(TemporalBuiltinRoundingMode::HalfTrunc),
        "halfEven" => Ok(TemporalBuiltinRoundingMode::HalfEven),
        _ => Err(range_error(cx)),
    }
}

pub(super) fn temporal_option_string_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Option<String>, Cx::Error> {
    if value.is_undefined() {
        return Ok(None);
    }
    let string_ref = to_string_string_ref(cx, value)?;
    Ok(Some(string_ref_text(cx, string_ref)?))
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TemporalDurationCalendarUnit {
    Year,
    Month,
    Week,
}

#[derive(Clone, Copy)]
pub(super) enum TemporalDurationParsedUnit {
    CalendarRelative(TemporalDurationCalendarUnit),
    Exact(TemporalBuiltinDurationExactUnit),
}

#[derive(Clone, Copy)]
pub(super) enum TemporalDurationParsedLargestUnit {
    Missing,
    Auto,
    CalendarRelative(TemporalDurationCalendarUnit),
    Exact(TemporalBuiltinDurationExactUnit),
}

pub(super) struct TemporalDurationRoundOptions {
    largest_unit: TemporalDurationParsedLargestUnit,
    smallest_unit: Option<TemporalDurationParsedUnit>,
    rounding_increment: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
    relative_to: Option<TemporalDurationRelativeTo>,
}

pub(super) struct TemporalDurationTotalOptions {
    unit: TemporalDurationParsedUnit,
    relative_to: Option<TemporalDurationRelativeTo>,
}

#[derive(Clone, Copy)]
pub(super) enum TemporalDurationRelativeTo {
    PlainDate(TemporalPlainDateObjectData),
    PlainDateTime(TemporalPlainDateTimeObjectData),
    ZonedDateTime(TemporalZonedDateTimeObjectData),
}

pub(super) fn temporal_duration_round_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationRoundOptions, Cx::Error> {
    if value.is_undefined() || value.is_null() {
        return Err(type_error(cx));
    }
    if value.is_string() {
        return Ok(TemporalDurationRoundOptions {
            largest_unit: TemporalDurationParsedLargestUnit::Missing,
            smallest_unit: Some(temporal_duration_parsed_unit(cx, value)?),
            rounding_increment: 1,
            rounding_mode: TemporalBuiltinRoundingMode::HalfExpand,
            relative_to: None,
        });
    }
    let Some(object_ref) = value.as_object_ref() else {
        return Err(type_error(cx));
    };

    let largest_unit_value = temporal_property_value(cx, object_ref, "largestUnit")?;
    let largest_unit = temporal_duration_largest_unit_option(cx, largest_unit_value)?;
    let relative_to = temporal_property_value(cx, object_ref, "relativeTo")?;
    let relative_to = temporal_duration_relative_to_option(cx, relative_to)?;
    let rounding_increment_value = temporal_property_value(cx, object_ref, "roundingIncrement")?;
    let rounding_increment =
        temporal_duration_rounding_increment_option(cx, rounding_increment_value)?;
    let rounding_mode_value = temporal_property_value(cx, object_ref, "roundingMode")?;
    let rounding_mode = temporal_duration_rounding_mode_option_with_default(
        cx,
        rounding_mode_value,
        TemporalBuiltinRoundingMode::HalfExpand,
    )?;
    let smallest_unit_value = temporal_property_value(cx, object_ref, "smallestUnit")?;
    let smallest_unit = if smallest_unit_value.is_undefined() {
        None
    } else {
        Some(temporal_duration_parsed_unit(cx, smallest_unit_value)?)
    };

    if matches!(largest_unit, TemporalDurationParsedLargestUnit::Missing) && smallest_unit.is_none()
    {
        return Err(range_error(cx));
    }

    Ok(TemporalDurationRoundOptions {
        largest_unit,
        smallest_unit,
        rounding_increment,
        rounding_mode,
        relative_to,
    })
}

pub(super) fn temporal_duration_total_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationTotalOptions, Cx::Error> {
    if value.is_undefined() || value.is_null() {
        return Err(type_error(cx));
    }
    if value.is_string() {
        return Ok(TemporalDurationTotalOptions {
            unit: temporal_duration_parsed_unit(cx, value)?,
            relative_to: None,
        });
    }
    let Some(object_ref) = value.as_object_ref() else {
        return Err(type_error(cx));
    };
    let relative_to = temporal_property_value(cx, object_ref, "relativeTo")?;
    let relative_to = temporal_duration_relative_to_option(cx, relative_to)?;
    let unit_value = temporal_property_value(cx, object_ref, "unit")?;
    if unit_value.is_undefined() {
        return Err(range_error(cx));
    }
    Ok(TemporalDurationTotalOptions {
        unit: temporal_duration_parsed_unit(cx, unit_value)?,
        relative_to,
    })
}

pub(super) fn temporal_duration_relative_to_option<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Option<TemporalDurationRelativeTo>, Cx::Error> {
    if value.is_undefined() {
        return Ok(None);
    }
    if value.is_null() {
        return Err(type_error(cx));
    }
    if let Some(string_ref) = value.as_string_ref() {
        let text = string_ref_text(cx, string_ref)?;
        if temporal_zoned_date_time_zone_annotation(&text).is_some() {
            let zoned = temporal_zoned_date_time_from_value(cx, value)?;
            temporal_duration_validate_relative_zoned_string_limits(cx, &text, zoned)?;
            return Ok(Some(TemporalDurationRelativeTo::ZonedDateTime(zoned)));
        }
        if text.contains('T') || text.contains('t') {
            return temporal_plain_date_time_from_value(cx, value)
                .map(TemporalDurationRelativeTo::PlainDateTime)
                .map(Some);
        }
        return temporal_plain_date_from_value(cx, value)
            .map(TemporalDurationRelativeTo::PlainDate)
            .map(Some);
    }
    let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let existing = {
        let agent = cx.agent();
        agent.objects().temporal_object(object_ref).copied()
    };
    match existing {
        Some(TemporalObjectData::PlainDate(data)) => {
            return Ok(Some(TemporalDurationRelativeTo::PlainDate(data)));
        }
        Some(TemporalObjectData::PlainDateTime(data)) => {
            return Ok(Some(TemporalDurationRelativeTo::PlainDateTime(data)));
        }
        Some(TemporalObjectData::ZonedDateTime(data)) => {
            return Ok(Some(TemporalDurationRelativeTo::ZonedDateTime(data)));
        }
        _ => {}
    }

    temporal_duration_relative_to_from_property_bag(cx, object_ref).map(Some)
}

fn temporal_duration_validate_relative_zoned_string_limits<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    text: &str,
    zoned: TemporalZonedDateTimeObjectData,
) -> Result<(), Cx::Error> {
    if zoned.epoch_nanoseconds() == -temporal_ops::INSTANT_EPOCH_NANOSECONDS_MAX
        && matches!(
            temporal_zoned_date_time_explicit_offset(text),
            Some(offset) if offset < 0
        )
    {
        return Err(range_error(cx));
    }
    Ok(())
}

#[allow(
    clippy::too_many_lines,
    reason = "relativeTo property-bag parsing keeps calendar, time-zone, and date-time branches together"
)]
fn temporal_duration_relative_to_from_property_bag<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
) -> Result<TemporalDurationRelativeTo, Cx::Error> {
    let calendar = temporal_property_value(cx, object_ref, "calendar")?;
    if !calendar.is_undefined() {
        temporal_validate_iso_calendar_value(cx, calendar)?;
    }
    let day_value = temporal_property_value(cx, object_ref, "day")?;
    if day_value.is_undefined() {
        return Err(type_error(cx));
    }
    let day = temporal_integer_part_from_value(cx, day_value)?;
    let hour_value = temporal_property_value(cx, object_ref, "hour")?;
    let hour = if hour_value.is_undefined() {
        None
    } else {
        Some(temporal_time_part_from_value(cx, hour_value)?)
    };
    let microsecond_value = temporal_property_value(cx, object_ref, "microsecond")?;
    let microsecond = if microsecond_value.is_undefined() {
        None
    } else {
        Some(temporal_time_part_from_value(cx, microsecond_value)?)
    };
    let millisecond_value = temporal_property_value(cx, object_ref, "millisecond")?;
    let millisecond = if millisecond_value.is_undefined() {
        None
    } else {
        Some(temporal_time_part_from_value(cx, millisecond_value)?)
    };
    let minute_value = temporal_property_value(cx, object_ref, "minute")?;
    let minute = if minute_value.is_undefined() {
        None
    } else {
        Some(temporal_time_part_from_value(cx, minute_value)?)
    };
    let month_value = temporal_property_value(cx, object_ref, "month")?;
    let month = if month_value.is_undefined() {
        None
    } else {
        Some(temporal_integer_part_from_value(cx, month_value)?)
    };
    let month_code_value = temporal_property_value(cx, object_ref, "monthCode")?;
    let month = if let Some(month) = month {
        if !month_code_value.is_undefined() {
            let month_code = temporal_month_from_month_code_value(cx, month_code_value)?;
            if month != month_code {
                return Err(range_error(cx));
            }
        }
        month
    } else if !month_code_value.is_undefined() {
        temporal_month_from_month_code_value(cx, month_code_value)?
    } else {
        return Err(type_error(cx));
    };
    let nanosecond_value = temporal_property_value(cx, object_ref, "nanosecond")?;
    let nanosecond = if nanosecond_value.is_undefined() {
        None
    } else {
        Some(temporal_time_part_from_value(cx, nanosecond_value)?)
    };
    let offset = temporal_property_value(cx, object_ref, "offset")?;
    let offset_text = if offset.is_undefined() {
        None
    } else if let Some(offset_ref) = offset.as_string_ref() {
        Some(string_ref_text(cx, offset_ref)?)
    } else {
        if offset.as_object_ref().is_none() {
            return Err(type_error(cx));
        }
        let offset_ref = to_string_string_ref(cx, offset)?;
        Some(string_ref_text(cx, offset_ref)?)
    };
    if let Some(offset_text) = offset_text.as_ref()
        && temporal_parse_offset_string(offset_text, true).is_none()
    {
        return Err(range_error(cx));
    }
    let second_value = temporal_property_value(cx, object_ref, "second")?;
    let second = if second_value.is_undefined() {
        None
    } else {
        Some(temporal_time_part_from_value(cx, second_value)?)
    };
    let time_zone = temporal_property_value(cx, object_ref, "timeZone")?;
    let year_value = temporal_property_value(cx, object_ref, "year")?;
    if year_value.is_undefined() {
        return Err(type_error(cx));
    }
    let year = temporal_integer_part_from_value(cx, year_value)?;

    let date = temporal_plain_date_from_parts(cx, year, month, day)?;
    let hour = hour.unwrap_or(0);
    let minute = minute.unwrap_or(0);
    let second = second.unwrap_or(0);
    let millisecond = millisecond.unwrap_or(0);
    let microsecond = microsecond.unwrap_or(0);
    let nanosecond = nanosecond.unwrap_or(0);

    if !time_zone.is_undefined() {
        let time_zone_id = temporal_time_zone_id_from_value(cx, time_zone)?;
        let Ok(hour) = u8::try_from(hour) else {
            return Err(range_error(cx));
        };
        let Ok(minute) = u8::try_from(minute) else {
            return Err(range_error(cx));
        };
        let Ok(mut second) = u8::try_from(second) else {
            return Err(range_error(cx));
        };
        if second > 60 {
            return Err(range_error(cx));
        }
        second = second.min(59);
        let Ok(millisecond) = u16::try_from(millisecond) else {
            return Err(range_error(cx));
        };
        let Ok(microsecond) = u16::try_from(microsecond) else {
            return Err(range_error(cx));
        };
        let Ok(nanosecond) = u16::try_from(nanosecond) else {
            return Err(range_error(cx));
        };
        let date_time = TemporalCivilDateTime::new(
            date.year(),
            date.month(),
            date.day(),
            hour,
            minute,
            second,
            millisecond,
            microsecond,
            nanosecond,
        );
        let instant = cx.temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
            time_zone_id: time_zone_id.clone(),
            date_time,
            disambiguation: TemporalDisambiguation::Compatible,
        })?;
        let zoned =
            temporal_zoned_date_time_from_parts(cx, instant.epoch_nanoseconds, &time_zone_id)?;
        return Ok(TemporalDurationRelativeTo::ZonedDateTime(zoned));
    }

    if hour_value.is_undefined()
        && minute_value.is_undefined()
        && second_value.is_undefined()
        && millisecond_value.is_undefined()
        && microsecond_value.is_undefined()
        && nanosecond_value.is_undefined()
    {
        return Ok(TemporalDurationRelativeTo::PlainDate(date));
    }

    let date_time = temporal_plain_date_time_from_parts_with_overflow(
        cx,
        i64::from(date.year()),
        i64::from(date.month()),
        i64::from(date.day()),
        hour,
        minute,
        second,
        millisecond,
        microsecond,
        nanosecond,
        TemporalOverflow::Constrain,
    )?;
    Ok(TemporalDurationRelativeTo::PlainDateTime(date_time))
}

pub(super) const fn temporal_duration_exact_unit_nanoseconds(
    unit: TemporalBuiltinDurationExactUnit,
) -> i128 {
    match unit {
        TemporalBuiltinDurationExactUnit::Day => TEMPORAL_NANOS_PER_DAY,
        TemporalBuiltinDurationExactUnit::Hour => TEMPORAL_NANOS_PER_HOUR,
        TemporalBuiltinDurationExactUnit::Minute => TEMPORAL_NANOS_PER_MINUTE,
        TemporalBuiltinDurationExactUnit::Second => TEMPORAL_NANOS_PER_SECOND,
        TemporalBuiltinDurationExactUnit::Millisecond => TEMPORAL_NANOS_PER_MILLISECOND,
        TemporalBuiltinDurationExactUnit::Microsecond => TEMPORAL_NANOS_PER_MICROSECOND,
        TemporalBuiltinDurationExactUnit::Nanosecond => 1,
    }
}

pub(super) const fn temporal_date_time_difference_unit_from_duration_exact(
    unit: TemporalBuiltinDurationExactUnit,
) -> TemporalDateTimeDifferenceUnit {
    match unit {
        TemporalBuiltinDurationExactUnit::Day => TemporalDateTimeDifferenceUnit::Day,
        TemporalBuiltinDurationExactUnit::Hour => TemporalDateTimeDifferenceUnit::Hour,
        TemporalBuiltinDurationExactUnit::Minute => TemporalDateTimeDifferenceUnit::Minute,
        TemporalBuiltinDurationExactUnit::Second => TemporalDateTimeDifferenceUnit::Second,
        TemporalBuiltinDurationExactUnit::Millisecond => {
            TemporalDateTimeDifferenceUnit::Millisecond
        }
        TemporalBuiltinDurationExactUnit::Microsecond => {
            TemporalDateTimeDifferenceUnit::Microsecond
        }
        TemporalBuiltinDurationExactUnit::Nanosecond => TemporalDateTimeDifferenceUnit::Nanosecond,
    }
}

pub(super) fn temporal_duration_relative_total_nanoseconds<Cx: PublicBuiltinDispatchContext>(
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

const fn temporal_duration_round_calendar_largest_unit(
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

const fn temporal_duration_default_largest_date_unit(
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

fn temporal_duration_round_calendar_smallest_unit<Cx: PublicBuiltinDispatchContext>(
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

const fn temporal_duration_calendar_unit_to_date_difference_unit(
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
pub(super) fn temporal_duration_round_calendar_relative<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_duration_round_calendar_relative_weeks<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn temporal_duration_round_calendar_relative_exact<Cx: PublicBuiltinDispatchContext>(
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

pub(super) const fn temporal_rounding_mode_for_negated_duration(
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

fn temporal_duration_calendar_day_time_difference<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_duration_round_years_between_dates<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_duration_date_with_start_time_total_nanoseconds<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    date: TemporalPlainDateObjectData,
    start_time: i128,
) -> Result<i128, Cx::Error> {
    temporal_plain_date_ordinal_day(date)
        .checked_mul(TEMPORAL_NANOS_PER_DAY)
        .and_then(|days| days.checked_add(start_time))
        .ok_or_else(|| range_error(cx))
}

fn temporal_duration_round_months_between_dates<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn temporal_duration_validate_month_rounding_boundary<
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

fn temporal_duration_validate_day_rounding_boundary<Cx: PublicBuiltinDispatchContext>(
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

const fn temporal_duration_should_round_positive_remainder_up(
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

const fn temporal_duration_should_round_negative_remainder_away(
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

fn temporal_duration_relative_start_plain_date<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_duration_relative_start_plain_date_time<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn temporal_duration_largest_unit_option<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn temporal_duration_parsed_unit<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationParsedUnit, Cx::Error> {
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    temporal_duration_unit_from_text(&text).ok_or_else(|| range_error(cx))
}

pub(super) fn temporal_duration_unit_from_text(text: &str) -> Option<TemporalDurationParsedUnit> {
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

pub(super) fn temporal_duration_rounding_increment_option<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn temporal_duration_rounding_increment_is_valid(
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

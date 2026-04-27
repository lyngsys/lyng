use super::*;

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
    component: fn(TemporalDurationObjectData) -> i64,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    Ok(temporal_i64_to_number_value(component(data)))
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

    let smallest_unit = match options.smallest_unit {
        Some(TemporalDurationParsedUnit::Exact(unit)) => unit,
        Some(TemporalDurationParsedUnit::CalendarRelative) => {
            if temporal_duration_sign(data) == 0 && options.relative_to.is_some() {
                return allocate_current_temporal_blank_duration_object(cx);
            }
            return Err(range_error(cx));
        }
        None => TemporalBuiltinDurationExactUnit::Nanosecond,
    };
    let largest_unit = match options.largest_unit {
        TemporalDurationParsedLargestUnit::Missing | TemporalDurationParsedLargestUnit::Auto => {
            temporal_duration_default_largest_exact_unit(data, smallest_unit)
        }
        TemporalDurationParsedLargestUnit::Exact(unit) => unit,
        TemporalDurationParsedLargestUnit::CalendarRelative => {
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
    if temporal_ops::duration_has_calendar_relative_units(data) {
        let Some(relative_to) = options.relative_to else {
            return Err(range_error(cx));
        };
        let total_nanoseconds =
            temporal_duration_relative_total_nanoseconds(cx, data, relative_to)?;
        let increment = temporal_duration_exact_unit_nanoseconds(smallest_unit)
            .checked_mul(options.rounding_increment)
            .ok_or_else(|| range_error(cx))?;
        let rounded = temporal_round_epoch_nanoseconds_to_increment(
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
            if temporal_ops::duration_has_calendar_relative_units(data) {
                let Some(relative_to) = options.relative_to else {
                    return Err(range_error(cx));
                };
                let total_nanoseconds =
                    temporal_duration_relative_total_nanoseconds(cx, data, relative_to)?;
                total_nanoseconds as f64 / temporal_duration_exact_unit_nanoseconds(unit) as f64
            } else {
                temporal_total_duration_exact(data, unit)
                    .filter(|value| value.is_finite())
                    .ok_or_else(|| range_error(cx))?
            }
        }
        TemporalDurationParsedUnit::CalendarRelative => {
            if temporal_duration_sign(data) == 0 && options.relative_to.is_some() {
                0.0
            } else {
                return Err(range_error(cx));
            }
        }
    };
    Ok(Value::from_f64(total))
}

fn temporal_duration_add_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_additive_builtin(cx, invocation, temporal_ops::add_durations)
}

fn temporal_duration_subtract_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_additive_builtin(cx, invocation, temporal_ops::subtract_durations)
}

fn temporal_duration_additive_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    operation: fn(
        TemporalDurationObjectData,
        TemporalDurationObjectData,
    ) -> Option<TemporalDurationObjectData>,
) -> Result<Value, Cx::Error> {
    let base = temporal_duration_data(cx, invocation.this_value())?;
    let other = temporal_duration_from_additive_argument(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let data = operation(base, other).ok_or_else(|| range_error(cx))?;
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
    if temporal_ops::duration_has_calendar_relative_units(left)
        || temporal_ops::duration_has_calendar_relative_units(right)
    {
        let Some(relative_to) = relative_to else {
            return Err(range_error(cx));
        };
        let left_total = temporal_duration_relative_total_nanoseconds(cx, left, relative_to)?;
        let right_total = temporal_duration_relative_total_nanoseconds(cx, right, relative_to)?;
        return Ok(temporal_compare_ordering(left_total.cmp(&right_total)));
    }
    let ordering =
        temporal_ops::compare_time_duration(left, right).ok_or_else(|| range_error(cx))?;
    Ok(temporal_compare_ordering(ordering))
}

pub(super) fn temporal_i64_to_number_value(value: i64) -> Value {
    i32::try_from(value).map_or_else(|_| Value::from_f64(value as f64), Value::from_smi)
}

#[allow(
    clippy::float_cmp,
    reason = "Temporal numeric conversion requires exact integral-number validation."
)]
pub(super) fn temporal_duration_part_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<i64, Cx::Error> {
    if value.is_undefined() {
        return Ok(0);
    }
    let number = to_number_for_builtin(cx, value)?;
    if !number.is_finite() || number.trunc() != number {
        return Err(range_error(cx));
    }
    if !(-(TEMPORAL_SAFE_INTEGER_MAX as f64)..=TEMPORAL_SAFE_INTEGER_MAX as f64).contains(&number) {
        return Err(range_error(cx));
    }
    Ok(number as i64)
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
    if number < i128::MIN as f64 || number > i128::MAX as f64 {
        return Err(range_error(cx));
    }
    Ok(number as i128)
}

pub(super) fn temporal_duration_part_i128_to_i64<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: i128,
) -> Result<i64, Cx::Error> {
    i64::try_from(value).map_err(|_| range_error(cx))
}

pub(super) fn temporal_duration_part_from_argument<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    index: usize,
) -> Result<i64, Cx::Error> {
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
) -> Result<Option<i64>, Cx::Error> {
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
        .and_then(RealmRecord::root_shape)
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

pub(super) fn temporal_duration_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationObjectData, Cx::Error> {
    let Some(object_ref) = value.as_object_ref() else {
        if !value.is_string() {
            return Err(type_error(cx));
        }
        let string_ref = to_string_string_ref(cx, value)?;
        let text = string_ref_text(cx, string_ref)?;
        let data = parse_temporal_duration(&text).ok_or_else(|| range_error(cx))?;
        validate_temporal_duration(cx, data)?;
        return Ok(data);
    };
    let existing = {
        let agent = cx.agent();
        agent.objects().temporal_object(object_ref).copied()
    };
    if let Some(TemporalObjectData::Duration(data)) = existing {
        return Ok(data);
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
    let [seconds, milliseconds, microseconds, nanoseconds] = match [
        i64::try_from(seconds),
        i64::try_from(milliseconds),
        i64::try_from(microseconds),
        i64::try_from(nanoseconds),
    ] {
        [Ok(seconds), Ok(milliseconds), Ok(microseconds), Ok(nanoseconds)] => {
            [seconds, milliseconds, microseconds, nanoseconds]
        }
        _ => temporal_ops::balance_duration_subsecond_fields(
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
        )
        .ok_or_else(|| range_error(cx))?,
    };

    let data = TemporalDurationObjectData::new(
        temporal_duration_part_i128_to_i64(cx, years)?,
        temporal_duration_part_i128_to_i64(cx, months)?,
        temporal_duration_part_i128_to_i64(cx, weeks)?,
        temporal_duration_part_i128_to_i64(cx, days)?,
        temporal_duration_part_i128_to_i64(cx, hours)?,
        temporal_duration_part_i128_to_i64(cx, minutes)?,
        seconds,
        milliseconds,
        microseconds,
        nanoseconds,
    );
    validate_temporal_duration(cx, data)?;
    Ok(data)
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
        return Ok(Some(digits as u8));
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

#[derive(Clone, Copy)]
pub(super) enum TemporalDurationParsedUnit {
    CalendarRelative,
    Exact(TemporalBuiltinDurationExactUnit),
}

#[derive(Clone, Copy)]
pub(super) enum TemporalDurationParsedLargestUnit {
    Missing,
    Auto,
    CalendarRelative,
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
    if value.is_undefined() || value.is_null() {
        return Ok(None);
    }
    if let Some(string_ref) = value.as_string_ref() {
        let text = string_ref_text(cx, string_ref)?;
        if temporal_zoned_date_time_zone_annotation(&text).is_some() {
            return temporal_zoned_date_time_from_value(cx, value)
                .map(TemporalDurationRelativeTo::ZonedDateTime)
                .map(Some);
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

    let time_zone = temporal_property_value(cx, object_ref, "timeZone")?;
    if !time_zone.is_undefined() {
        return temporal_zoned_date_time_from_value(cx, value)
            .map(TemporalDurationRelativeTo::ZonedDateTime)
            .map(Some);
    }

    let date = temporal_plain_date_from_value(cx, value)?;
    let has_time = [
        "hour",
        "minute",
        "second",
        "millisecond",
        "microsecond",
        "nanosecond",
    ]
    .iter()
    .try_fold(false, |has_time, property| {
        let property_value = temporal_property_value(cx, object_ref, property)?;
        Ok::<_, Cx::Error>(has_time || !property_value.is_undefined())
    })?;
    if !has_time {
        return Ok(Some(TemporalDurationRelativeTo::PlainDate(date)));
    }
    let hour = temporal_optional_time_part_from_property(cx, object_ref, "hour")?.unwrap_or(0);
    let minute = temporal_optional_time_part_from_property(cx, object_ref, "minute")?.unwrap_or(0);
    let second = temporal_optional_time_part_from_property(cx, object_ref, "second")?.unwrap_or(0);
    let millisecond =
        temporal_optional_time_part_from_property(cx, object_ref, "millisecond")?.unwrap_or(0);
    let microsecond =
        temporal_optional_time_part_from_property(cx, object_ref, "microsecond")?.unwrap_or(0);
    let nanosecond =
        temporal_optional_time_part_from_property(cx, object_ref, "nanosecond")?.unwrap_or(0);
    let date_time = temporal_plain_date_time_from_parts(
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
    )?;
    Ok(Some(TemporalDurationRelativeTo::PlainDateTime(date_time)))
}

pub(super) fn temporal_duration_exact_unit_nanoseconds(
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

pub(super) fn temporal_date_time_difference_unit_from_duration_exact(
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
            let end_value = temporal_zoned_date_time_add_duration(cx, start, duration)?;
            let end = temporal_zoned_date_time_data(cx, end_value)?;
            end.epoch_nanoseconds()
                .checked_sub(start.epoch_nanoseconds())
                .ok_or_else(|| range_error(cx))
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
        Some(TemporalDurationParsedUnit::CalendarRelative) => {
            TemporalDurationParsedLargestUnit::CalendarRelative
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
        "year" | "years" | "month" | "months" | "week" | "weeks" => {
            Some(TemporalDurationParsedUnit::CalendarRelative)
        }
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
    Ok(increment as i128)
}

pub(super) fn temporal_duration_rounding_increment_is_valid(
    smallest_unit: TemporalBuiltinDurationExactUnit,
    rounding_increment: i128,
) -> bool {
    let maximum = match smallest_unit {
        TemporalBuiltinDurationExactUnit::Day => {
            return (1..=1_000_000_000).contains(&rounding_increment)
        }
        TemporalBuiltinDurationExactUnit::Hour => 24,
        TemporalBuiltinDurationExactUnit::Minute | TemporalBuiltinDurationExactUnit::Second => 60,
        TemporalBuiltinDurationExactUnit::Millisecond
        | TemporalBuiltinDurationExactUnit::Microsecond
        | TemporalBuiltinDurationExactUnit::Nanosecond => 1000,
    };
    rounding_increment > 0 && rounding_increment < maximum && maximum % rounding_increment == 0
}

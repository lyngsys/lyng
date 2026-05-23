use super::{
    allocate_temporal_duration_object, current_temporal_duration_prototype,
    format_temporal_duration, format_temporal_duration_with_seconds_precision, map_completion,
    negate_temporal_duration, object, range_error, string_value, temporal_constructor_prototype,
    temporal_duration_part_from_argument, temporal_duration_sign,
    temporal_duration_to_string_options, temporal_i128_to_number_value,
    temporal_optional_duration_part_from_property, type_error, validate_temporal_duration,
    BuiltinInvocation, PublicBuiltinDispatchContext, TemporalDurationObjectData,
    TemporalObjectData, TemporalObjectKind, Value,
};

pub(in crate::public::dispatch::temporal) fn allocate_current_temporal_blank_duration_object<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
) -> Result<Value, Cx::Error> {
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(
        cx,
        prototype,
        TemporalDurationObjectData::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
    )
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_data<
    Cx: PublicBuiltinDispatchContext,
>(
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

pub(in crate::public::dispatch::temporal) fn temporal_duration_component_getter<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    component: fn(TemporalDurationObjectData) -> i128,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    Ok(temporal_i128_to_number_value(component(data)))
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
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

pub(in crate::public::dispatch::temporal) fn temporal_duration_years_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::years)
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_months_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::months)
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_weeks_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::weeks)
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_days_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::days)
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_hours_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::hours)
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_minutes_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::minutes)
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_seconds_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::seconds)
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_milliseconds_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::milliseconds)
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_microseconds_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::microseconds)
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_nanoseconds_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::nanoseconds)
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_sign_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(temporal_duration_sign(data)))
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_blank_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    Ok(Value::from_bool(temporal_duration_sign(data) == 0))
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_to_string_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
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

pub(in crate::public::dispatch::temporal) fn temporal_duration_to_json_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format_temporal_duration(data)))
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_to_locale_string_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format_temporal_duration(data)))
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_negated_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = negate_temporal_duration(temporal_duration_data(cx, invocation.this_value())?);
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(cx, prototype, data)
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_abs_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
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

pub(in crate::public::dispatch::temporal) fn temporal_duration_with_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
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

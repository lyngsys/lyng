use super::{
    allocate_temporal_duration_object, current_temporal_duration_prototype, range_error,
    temporal_compare_ordering, temporal_duration_data,
    temporal_duration_from_additive_argument_with_largest_unit, temporal_duration_from_value,
    temporal_duration_relative_to_option, temporal_duration_relative_total_nanoseconds,
    temporal_duration_validate_exact_relative_to_range, temporal_ops, temporal_property_value,
    type_error, validate_temporal_duration, BuiltinInvocation, PublicBuiltinDispatchContext,
    TemporalDurationObjectData, TemporalDurationRelativeTo, Value,
};

pub(in crate::public::dispatch::temporal) fn temporal_duration_add_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_additive_builtin(
        cx,
        invocation,
        temporal_ops::add_durations_with_largest_unit,
    )
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_subtract_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_additive_builtin(
        cx,
        invocation,
        temporal_ops::subtract_durations_with_largest_unit,
    )
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_additive_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
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

pub(in crate::public::dispatch::temporal) fn temporal_duration_value_of_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Err(type_error(cx))
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_from_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
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

pub(in crate::public::dispatch::temporal) fn temporal_duration_compare_relative_to_option<
    Cx: PublicBuiltinDispatchContext,
>(
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

pub(in crate::public::dispatch::temporal) fn temporal_duration_compare_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
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

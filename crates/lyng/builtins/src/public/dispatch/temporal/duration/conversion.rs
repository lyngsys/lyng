use super::{
    parse_temporal_duration, range_error, string_ref_text, temporal_i128_as_number,
    temporal_number_to_i128_after_range_check, temporal_ops, temporal_property_value,
    to_number_for_builtin, to_string_string_ref, type_error, AllocationLifetime, BuiltinInvocation,
    ObjectAllocation, ObjectColdData, ObjectRef, OrdinaryObjectData, PublicBuiltinDispatchContext,
    TemporalDurationObjectData, TemporalObjectData, TemporalObjectKind, Value,
    TEMPORAL_SAFE_INTEGER_MAX,
};

pub(in crate::public::dispatch::temporal) fn temporal_i128_to_number_value(value: i128) -> Value {
    i32::try_from(value).map_or_else(
        |_| Value::from_f64(temporal_i128_as_number(value)),
        Value::from_smi,
    )
}

#[allow(
    clippy::float_cmp,
    reason = "Temporal numeric conversion requires exact integral-number validation."
)]
pub(in crate::public::dispatch::temporal) fn temporal_duration_part_from_value<
    Cx: PublicBuiltinDispatchContext,
>(
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
pub(in crate::public::dispatch::temporal) fn temporal_duration_part_i128_from_value<
    Cx: PublicBuiltinDispatchContext,
>(
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

pub(in crate::public::dispatch::temporal) fn temporal_duration_part_from_argument<
    Cx: PublicBuiltinDispatchContext,
>(
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

pub(in crate::public::dispatch::temporal) fn temporal_optional_duration_part_from_property<
    Cx: PublicBuiltinDispatchContext,
>(
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

pub(in crate::public::dispatch::temporal) fn temporal_duration_part_i128_from_property<
    Cx: PublicBuiltinDispatchContext,
>(
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

pub(in crate::public::dispatch::temporal) fn validate_temporal_duration<
    Cx: PublicBuiltinDispatchContext,
>(
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

pub(in crate::public::dispatch::temporal) fn allocate_temporal_duration_object<
    Cx: PublicBuiltinDispatchContext,
>(
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
pub(in crate::public::dispatch::temporal) struct TemporalDurationWithLargestUnit {
    pub(super) data: TemporalDurationObjectData,
    pub(super) largest_unit: temporal_ops::TemporalDurationExactUnit,
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_from_value<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationObjectData, Cx::Error> {
    Ok(temporal_duration_from_value_with_largest_unit(cx, value)?.data)
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_from_value_with_largest_unit<
    Cx: PublicBuiltinDispatchContext,
>(
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

pub(in crate::public::dispatch::temporal) fn temporal_duration_from_additive_argument<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationObjectData, Cx::Error> {
    if value.as_object_ref().is_none() && !value.is_string() {
        return Err(type_error(cx));
    }
    temporal_duration_from_value(cx, value)
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_from_additive_argument_with_largest_unit<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationWithLargestUnit, Cx::Error> {
    if value.as_object_ref().is_none() && !value.is_string() {
        return Err(type_error(cx));
    }
    temporal_duration_from_value_with_largest_unit(cx, value)
}

pub(in crate::public::dispatch::temporal) const fn temporal_duration_largest_exact_unit_from_raw_parts(
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

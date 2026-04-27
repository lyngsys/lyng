use super::*;

pub(super) fn dispatch_temporal_plain_time_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::temporal_plain_time_builtin() {
        return temporal_plain_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_hour_getter_builtin() {
        return temporal_plain_time_hour_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_minute_getter_builtin() {
        return temporal_plain_time_minute_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_second_getter_builtin() {
        return temporal_plain_time_second_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_millisecond_getter_builtin() {
        return temporal_plain_time_millisecond_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_microsecond_getter_builtin() {
        return temporal_plain_time_microsecond_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_nanosecond_getter_builtin() {
        return temporal_plain_time_nanosecond_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_to_string_builtin() {
        return temporal_plain_time_to_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_to_json_builtin() {
        return temporal_plain_time_to_json_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_to_locale_string_builtin() {
        return temporal_plain_time_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_value_of_builtin() {
        return temporal_plain_time_value_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_equals_builtin() {
        return temporal_plain_time_equals_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_with_builtin() {
        return temporal_plain_time_with_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_add_builtin() {
        return temporal_plain_time_add_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_subtract_builtin() {
        return temporal_plain_time_subtract_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_round_builtin() {
        return temporal_plain_time_round_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_since_builtin() {
        return temporal_plain_time_since_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_until_builtin() {
        return temporal_plain_time_until_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_to_plain_date_time_builtin() {
        return temporal_plain_time_to_plain_date_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_from_builtin() {
        return temporal_plain_time_from_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_time_compare_builtin() {
        return temporal_plain_time_compare_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

pub(super) fn allocate_temporal_plain_time_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: ObjectRef,
    data: TemporalPlainTimeObjectData,
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
                        TemporalObjectKind::PlainTime,
                    ))),
                AllocationLifetime::Default,
            )
        })
    };
    let installed = cx
        .agent()
        .objects_mut()
        .install_temporal_object(object, TemporalObjectData::PlainTime(data));
    if !installed {
        return Err(type_error(cx));
    }
    Ok(Value::from_object_ref(object))
}

fn temporal_plain_time_data<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    this_value: Value,
) -> Result<TemporalPlainTimeObjectData, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, this_value, TemporalObjectKind::PlainTime)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::PlainTime(data) = payload else {
        return Err(type_error(cx));
    };
    Ok(data)
}

fn temporal_plain_time_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let Some(new_target) = invocation.new_target() else {
        return Err(type_error(cx));
    };
    let hour = temporal_time_part_from_argument(cx, invocation, 0)?;
    let minute = temporal_time_part_from_argument(cx, invocation, 1)?;
    let second = temporal_time_part_from_argument(cx, invocation, 2)?;
    let millisecond = temporal_time_part_from_argument(cx, invocation, 3)?;
    let microsecond = temporal_time_part_from_argument(cx, invocation, 4)?;
    let nanosecond = temporal_time_part_from_argument(cx, invocation, 5)?;
    let data = temporal_plain_time_from_parts(
        cx,
        hour,
        minute,
        second,
        millisecond,
        microsecond,
        nanosecond,
    )?;
    let prototype = temporal_constructor_prototype(cx, new_target)?;
    allocate_temporal_plain_time_object(cx, prototype, data)
}

fn temporal_plain_time_hour_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.hour())))
}

fn temporal_plain_time_minute_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.minute())))
}

fn temporal_plain_time_second_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.second())))
}

fn temporal_plain_time_millisecond_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.millisecond())))
}

fn temporal_plain_time_microsecond_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.microsecond())))
}

fn temporal_plain_time_nanosecond_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.nanosecond())))
}

fn temporal_plain_time_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_time_data(cx, invocation.this_value())?;
    let (precision, rounding_mode) = temporal_instant_to_string_options(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let data = temporal_plain_time_for_string_precision(cx, data, precision, rounding_mode)?;
    Ok(string_value(
        cx,
        &format_temporal_plain_time_with_precision(data, precision),
    ))
}

fn temporal_plain_time_to_json_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_time_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format_temporal_plain_time(data)))
}

fn temporal_plain_time_to_locale_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_time_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format_temporal_plain_time(data)))
}

fn temporal_plain_time_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Err(type_error(cx))
}

fn temporal_plain_time_equals_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = temporal_plain_time_data(cx, invocation.this_value())?;
    let right = temporal_plain_time_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(left == right))
}

fn temporal_plain_time_with_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let time = temporal_plain_time_data(cx, invocation.this_value())?;
    let object_ref = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined())
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    {
        let temporal = {
            let agent = cx.agent();
            agent.objects().temporal_object(object_ref).copied()
        };
        if temporal.is_some() {
            return Err(type_error(cx));
        }
    }
    temporal_reject_calendar_or_time_zone_properties(cx, object_ref)?;
    let hour = temporal_optional_time_part_from_property(cx, object_ref, "hour")?;
    let microsecond = temporal_optional_time_part_from_property(cx, object_ref, "microsecond")?;
    let millisecond = temporal_optional_time_part_from_property(cx, object_ref, "millisecond")?;
    let minute = temporal_optional_time_part_from_property(cx, object_ref, "minute")?;
    let nanosecond = temporal_optional_time_part_from_property(cx, object_ref, "nanosecond")?;
    let second = temporal_optional_time_part_from_property(cx, object_ref, "second")?;
    if hour.is_none()
        && microsecond.is_none()
        && millisecond.is_none()
        && minute.is_none()
        && nanosecond.is_none()
        && second.is_none()
    {
        return Err(type_error(cx));
    }
    let overflow = temporal_overflow_from_options(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let data = temporal_plain_time_from_parts_with_overflow(
        cx,
        hour.unwrap_or(i64::from(time.hour())),
        minute.unwrap_or(i64::from(time.minute())),
        second.unwrap_or(i64::from(time.second())),
        millisecond.unwrap_or(i64::from(time.millisecond())),
        microsecond.unwrap_or(i64::from(time.microsecond())),
        nanosecond.unwrap_or(i64::from(time.nanosecond())),
        overflow,
    )?;
    let prototype = current_temporal_plain_time_prototype(cx)?;
    allocate_temporal_plain_time_object(cx, prototype, data)
}

fn temporal_plain_time_add_duration<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    time: TemporalPlainTimeObjectData,
    duration: TemporalDurationObjectData,
) -> Result<TemporalPlainTimeObjectData, Cx::Error> {
    temporal_plain_time_from_nanoseconds(
        cx,
        temporal_plain_time_nanoseconds(time) + temporal_duration_time_nanoseconds(duration),
    )
}

fn temporal_plain_time_add_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let time = temporal_plain_time_data(cx, invocation.this_value())?;
    let duration = temporal_duration_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let data = temporal_plain_time_add_duration(cx, time, duration)?;
    let prototype = current_temporal_plain_time_prototype(cx)?;
    allocate_temporal_plain_time_object(cx, prototype, data)
}

fn temporal_plain_time_subtract_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let time = temporal_plain_time_data(cx, invocation.this_value())?;
    let duration = temporal_duration_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let data = temporal_plain_time_add_duration(cx, time, negate_temporal_duration(duration))?;
    let prototype = current_temporal_plain_time_prototype(cx)?;
    allocate_temporal_plain_time_object(cx, prototype, data)
}

fn temporal_plain_time_round_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let time = temporal_plain_time_data(cx, invocation.this_value())?;
    let options = temporal_exact_time_round_options(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let increment = temporal_exact_time_unit_nanoseconds(options.smallest_unit)
        .checked_mul(options.rounding_increment)
        .ok_or_else(|| range_error(cx))?;
    let rounded = temporal_round_epoch_nanoseconds_to_increment(
        temporal_plain_time_nanoseconds(time),
        increment,
        options.rounding_mode,
    )
    .ok_or_else(|| range_error(cx))?
    .rem_euclid(TEMPORAL_NANOS_PER_DAY);
    let data = temporal_plain_time_from_nanoseconds(cx, rounded)?;
    let prototype = current_temporal_plain_time_prototype(cx)?;
    allocate_temporal_plain_time_object(cx, prototype, data)
}

struct TemporalPlainTimeDifferenceOptions {
    largest_unit: TemporalBuiltinDurationExactUnit,
    smallest_unit: TemporalBuiltinDurationExactUnit,
    rounding_increment: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
}

fn temporal_plain_time_difference_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalPlainTimeDifferenceOptions, Cx::Error> {
    if value.is_undefined() {
        return Ok(TemporalPlainTimeDifferenceOptions {
            largest_unit: TemporalBuiltinDurationExactUnit::Hour,
            smallest_unit: TemporalBuiltinDurationExactUnit::Nanosecond,
            rounding_increment: 1,
            rounding_mode: TemporalBuiltinRoundingMode::Trunc,
        });
    }
    let Some(object_ref) = value.as_object_ref() else {
        return Err(type_error(cx));
    };

    let largest_unit_value = temporal_property_value(cx, object_ref, "largestUnit")?;
    let largest_unit_text = temporal_option_string_text(cx, largest_unit_value)?;
    let rounding_increment_value = temporal_property_value(cx, object_ref, "roundingIncrement")?;
    let rounding_increment =
        temporal_duration_rounding_increment_option(cx, rounding_increment_value)?;
    let rounding_mode_value = temporal_property_value(cx, object_ref, "roundingMode")?;
    let rounding_mode = temporal_duration_rounding_mode_option(cx, rounding_mode_value)?;
    let smallest_unit_value = temporal_property_value(cx, object_ref, "smallestUnit")?;
    let smallest_unit_text = temporal_option_string_text(cx, smallest_unit_value)?;
    let smallest_unit = if let Some(text) = smallest_unit_text.as_deref() {
        temporal_exact_time_unit_from_text(text).ok_or_else(|| range_error(cx))?
    } else {
        TemporalBuiltinDurationExactUnit::Nanosecond
    };
    let largest_unit = if let Some(text) = largest_unit_text.as_deref() {
        if text == "auto" {
            TemporalBuiltinDurationExactUnit::Hour
        } else {
            temporal_exact_time_unit_from_text(text).ok_or_else(|| range_error(cx))?
        }
    } else {
        TemporalBuiltinDurationExactUnit::Hour
    };
    if temporal_exact_time_unit_order(largest_unit) > temporal_exact_time_unit_order(smallest_unit)
        || !temporal_exact_time_rounding_increment_is_valid(smallest_unit, rounding_increment)
    {
        return Err(range_error(cx));
    }

    Ok(TemporalPlainTimeDifferenceOptions {
        largest_unit,
        smallest_unit,
        rounding_increment,
        rounding_mode,
    })
}

fn temporal_plain_time_difference_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    sign: i128,
) -> Result<Value, Cx::Error> {
    let time = temporal_plain_time_data(cx, invocation.this_value())?;
    let other = temporal_plain_time_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let options = temporal_plain_time_difference_options(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let raw_difference = temporal_plain_time_nanoseconds(time)
        .checked_sub(temporal_plain_time_nanoseconds(other))
        .and_then(|difference| difference.checked_mul(sign))
        .ok_or_else(|| range_error(cx))?;
    let increment = temporal_exact_time_unit_nanoseconds(options.smallest_unit)
        .checked_mul(options.rounding_increment)
        .ok_or_else(|| range_error(cx))?;
    let rounded = temporal_round_epoch_nanoseconds_to_increment(
        raw_difference,
        increment,
        options.rounding_mode,
    )
    .ok_or_else(|| range_error(cx))?;
    let duration =
        temporal_duration_from_nanoseconds_with_largest_unit(cx, rounded, options.largest_unit)?;
    validate_temporal_duration(cx, duration)?;
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(cx, prototype, duration)
}

fn temporal_plain_time_since_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_plain_time_difference_builtin(cx, invocation, 1)
}

fn temporal_plain_time_until_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_plain_time_difference_builtin(cx, invocation, -1)
}

fn temporal_plain_time_to_plain_date_time_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let time = temporal_plain_time_data(cx, invocation.this_value())?;
    let date = temporal_plain_date_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let data = TemporalPlainDateTimeObjectData::new(
        date.year(),
        date.month(),
        date.day(),
        time.hour(),
        time.minute(),
        time.second(),
        time.millisecond(),
        time.microsecond(),
        time.nanosecond(),
        date.calendar(),
    );
    let prototype = current_temporal_plain_date_time_prototype(cx)?;
    allocate_temporal_plain_date_time_object(cx, prototype, data)
}

fn temporal_plain_time_from_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let options = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let data = if value.is_string() {
        let data =
            temporal_plain_time_from_value_with_overflow(cx, value, TemporalOverflow::Constrain)?;
        let _overflow = temporal_overflow_from_options(cx, options)?;
        data
    } else if let Some(object_ref) = value.as_object_ref() {
        let temporal = {
            let agent = cx.agent();
            agent.objects().temporal_object(object_ref).copied()
        };
        if temporal.is_none() {
            let parts = temporal_plain_time_parts_from_property_bag(cx, object_ref)?;
            let overflow = temporal_overflow_from_options(cx, options)?;
            temporal_plain_time_from_parts_with_overflow(
                cx,
                parts.hour,
                parts.minute,
                parts.second,
                parts.millisecond,
                parts.microsecond,
                parts.nanosecond,
                overflow,
            )?
        } else {
            let data = temporal_plain_time_from_value_with_overflow(
                cx,
                value,
                TemporalOverflow::Constrain,
            )?;
            let _overflow = temporal_overflow_from_options(cx, options)?;
            data
        }
    } else {
        let data =
            temporal_plain_time_from_value_with_overflow(cx, value, TemporalOverflow::Constrain)?;
        let _overflow = temporal_overflow_from_options(cx, options)?;
        data
    };
    let prototype = current_temporal_plain_time_prototype(cx)?;
    allocate_temporal_plain_time_object(cx, prototype, data)
}

fn temporal_plain_time_compare_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = temporal_plain_time_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let right = temporal_plain_time_from_value(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(temporal_compare_ordering(
        (
            left.hour(),
            left.minute(),
            left.second(),
            left.millisecond(),
            left.microsecond(),
            left.nanosecond(),
        )
            .cmp(&(
                right.hour(),
                right.minute(),
                right.second(),
                right.millisecond(),
                right.microsecond(),
                right.nanosecond(),
            )),
    ))
}

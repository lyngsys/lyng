use super::{
    allocate_temporal_duration_object, allocate_temporal_zoned_date_time_object,
    current_temporal_duration_prototype, current_temporal_instant_prototype,
    current_temporal_zoned_date_time_prototype, format_temporal_civil_date_time_with_precision,
    format_temporal_offset, map_completion, object, parse_temporal_instant, range_error,
    string_ref_text, string_value, temporal_bigint_to_i128, temporal_compare_ordering,
    temporal_constructor_prototype, temporal_duration_from_additive_argument,
    temporal_duration_from_nanoseconds_with_largest_unit,
    temporal_duration_rounding_increment_option, temporal_duration_rounding_mode_option,
    temporal_duration_time_nanoseconds, temporal_exact_time_rounding_increment_is_valid,
    temporal_exact_time_unit_from_text, temporal_exact_time_unit_nanoseconds,
    temporal_exact_time_unit_order, temporal_i128_as_number, temporal_i128_to_bigint_value,
    temporal_instant_epoch_nanoseconds_is_valid, temporal_instant_fractional_second_digits_option,
    temporal_instant_largest_unit_default, temporal_instant_round_options,
    temporal_instant_smallest_unit_precision_from_text, temporal_number_to_i128_after_range_check,
    temporal_ops, temporal_option_string_text, temporal_property_value,
    temporal_round_duration_nanoseconds_to_increment,
    temporal_round_epoch_nanoseconds_to_fractional_digits,
    temporal_round_epoch_nanoseconds_to_increment, temporal_safe_integer_number,
    temporal_time_zone_id_from_value, temporal_zoned_date_time_from_parts, to_bigint_for_builtin,
    to_number_for_builtin, to_string_string_ref, type_error, validate_temporal_duration,
    AllocationLifetime, BuiltinFunctionId, BuiltinInvocation, ObjectAllocation, ObjectColdData,
    ObjectRef, OrdinaryObjectData, PropertyKey, PublicBuiltinDispatchContext,
    TemporalBuiltinDurationExactUnit, TemporalBuiltinRoundingMode, TemporalInstantObjectData,
    TemporalInstantStringPrecision, TemporalInstantToCivilRequest, TemporalObjectData,
    TemporalObjectKind, Value, TEMPORAL_INSTANT_EPOCH_MILLISECONDS_MAX,
    TEMPORAL_NANOS_PER_MILLISECOND, TEMPORAL_NANOS_PER_MINUTE, TEMPORAL_NANOS_PER_SECOND,
    TEMPORAL_UTC_TIME_ZONE_ID,
};

pub(super) fn dispatch_temporal_instant_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::temporal_instant_builtin() {
        return temporal_instant_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_instant_from_builtin() {
        return temporal_instant_from_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_instant_from_epoch_nanoseconds_builtin() {
        return temporal_instant_from_epoch_nanoseconds_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_instant_from_epoch_milliseconds_builtin() {
        return temporal_instant_from_epoch_milliseconds_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_instant_compare_builtin() {
        return temporal_instant_compare_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_instant_epoch_nanoseconds_getter_builtin() {
        return temporal_instant_epoch_nanoseconds_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_instant_epoch_milliseconds_getter_builtin() {
        return temporal_instant_epoch_milliseconds_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_instant_epoch_seconds_getter_builtin() {
        return temporal_instant_epoch_seconds_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_instant_to_string_builtin() {
        return temporal_instant_to_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_instant_to_json_builtin() {
        return temporal_instant_to_json_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_instant_to_locale_string_builtin() {
        return temporal_instant_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_instant_value_of_builtin() {
        return temporal_instant_value_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_instant_equals_builtin() {
        return temporal_instant_equals_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_instant_add_builtin() {
        return temporal_instant_add_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_instant_subtract_builtin() {
        return temporal_instant_subtract_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_instant_round_builtin() {
        return temporal_instant_round_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_instant_since_builtin() {
        return temporal_instant_since_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_instant_until_builtin() {
        return temporal_instant_until_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_instant_to_zoned_date_time_iso_builtin() {
        return temporal_instant_to_zoned_date_time_iso_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

pub(super) fn allocate_temporal_instant_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: ObjectRef,
    epoch_nanoseconds: i128,
) -> Result<Value, Cx::Error> {
    if !temporal_instant_epoch_nanoseconds_is_valid(epoch_nanoseconds) {
        return Err(range_error(cx));
    }
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
                        TemporalObjectKind::Instant,
                    ))),
                AllocationLifetime::Default,
            )
        })
    };
    let installed = cx.agent().objects_mut().install_temporal_object(
        object,
        TemporalObjectData::Instant(TemporalInstantObjectData::new(epoch_nanoseconds)),
    );
    if !installed {
        return Err(type_error(cx));
    }
    Ok(Value::from_object_ref(object))
}

pub(super) fn create_temporal_instant_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    epoch_nanoseconds: i128,
) -> Result<Value, Cx::Error> {
    let prototype = current_temporal_instant_prototype(cx)?;
    allocate_temporal_instant_object(cx, prototype, epoch_nanoseconds)
}

fn temporal_instant_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<i128, Cx::Error> {
    if let Some(object_ref) = value.as_object_ref() {
        let payload = {
            let agent = cx.agent();
            agent.objects().temporal_object(object_ref).copied()
        };
        match payload {
            Some(TemporalObjectData::Instant(data)) => return Ok(data.epoch_nanoseconds()),
            Some(TemporalObjectData::ZonedDateTime(data)) => return Ok(data.epoch_nanoseconds()),
            _ => {}
        }
        let key = {
            let agent = cx.agent();
            PropertyKey::from_atom(agent.atoms_mut().intern_collectible("epochNanoseconds"))
        };
        let epoch_nanoseconds = cx.get_property_value(value, key)?;
        if !epoch_nanoseconds.is_undefined() {
            let value = {
                let agent = cx.agent();
                temporal_bigint_to_i128(agent, epoch_nanoseconds)
            };
            return value.ok_or_else(|| range_error(cx));
        }
        let string_ref = to_string_string_ref(cx, value)?;
        let text = string_ref_text(cx, string_ref)?;
        return parse_temporal_instant(&text).ok_or_else(|| range_error(cx));
    }

    if let Some(string_ref) = value.as_string_ref() {
        let text = string_ref_text(cx, string_ref)?;
        return parse_temporal_instant(&text).ok_or_else(|| range_error(cx));
    }

    Err(type_error(cx))
}

fn temporal_instant_constructor_epoch_nanoseconds_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<i128, Cx::Error> {
    let bigint = to_bigint_for_builtin(cx, value)?;
    let value = {
        let agent = cx.agent();
        temporal_bigint_to_i128(agent, bigint)
    };
    value.ok_or_else(|| range_error(cx))
}

struct TemporalInstantToStringOptions {
    precision: TemporalInstantStringPrecision,
    rounding_mode: TemporalBuiltinRoundingMode,
    time_zone_id: Option<String>,
}

fn format_temporal_instant_utc<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    epoch_nanoseconds: i128,
    precision: TemporalInstantStringPrecision,
) -> Result<String, Cx::Error> {
    let civil = cx.temporal_instant_to_civil_time(&TemporalInstantToCivilRequest {
        time_zone_id: TEMPORAL_UTC_TIME_ZONE_ID.to_string(),
        epoch_nanoseconds,
    })?;
    let calendar = {
        let agent = cx.agent();
        agent.atoms_mut().intern_collectible("iso8601")
    };
    let mut text =
        format_temporal_civil_date_time_with_precision(civil.date_time, calendar, precision);
    text.push('Z');
    Ok(text)
}

fn format_temporal_instant_with_time_zone<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    epoch_nanoseconds: i128,
    precision: TemporalInstantStringPrecision,
    time_zone_id: &str,
) -> Result<String, Cx::Error> {
    let civil = cx.temporal_instant_to_civil_time(&TemporalInstantToCivilRequest {
        time_zone_id: time_zone_id.to_string(),
        epoch_nanoseconds,
    })?;
    let calendar = {
        let agent = cx.agent();
        agent.atoms_mut().intern_collectible("iso8601")
    };
    let mut text =
        format_temporal_civil_date_time_with_precision(civil.date_time, calendar, precision);
    text.push_str(&format_temporal_offset(civil.offset_nanoseconds));
    Ok(text)
}

fn temporal_instant_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let Some(new_target) = invocation.new_target() else {
        return Err(type_error(cx));
    };
    let epoch_nanoseconds = temporal_instant_constructor_epoch_nanoseconds_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let prototype = temporal_constructor_prototype(cx, new_target)?;
    allocate_temporal_instant_object(cx, prototype, epoch_nanoseconds)
}

fn temporal_instant_from_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let epoch_nanoseconds = temporal_instant_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let prototype = current_temporal_instant_prototype(cx)?;
    allocate_temporal_instant_object(cx, prototype, epoch_nanoseconds)
}

fn temporal_instant_from_epoch_nanoseconds_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let bigint = to_bigint_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let epoch_nanoseconds = {
        let agent = cx.agent();
        temporal_bigint_to_i128(agent, bigint)
    }
    .ok_or_else(|| range_error(cx))?;
    let prototype = current_temporal_instant_prototype(cx)?;
    allocate_temporal_instant_object(cx, prototype, epoch_nanoseconds)
}

fn temporal_instant_from_epoch_milliseconds_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let epoch_milliseconds = temporal_epoch_milliseconds_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let epoch_nanoseconds = epoch_milliseconds
        .checked_mul(TEMPORAL_NANOS_PER_MILLISECOND)
        .ok_or_else(|| range_error(cx))?;
    let prototype = current_temporal_instant_prototype(cx)?;
    allocate_temporal_instant_object(cx, prototype, epoch_nanoseconds)
}

#[allow(
    clippy::float_cmp,
    reason = "Temporal numeric conversion requires exact integral-number validation."
)]
fn temporal_epoch_milliseconds_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<i128, Cx::Error> {
    let number = to_number_for_builtin(cx, value)?;
    if !number.is_finite() || number.trunc() != number {
        return Err(range_error(cx));
    }
    let max = temporal_i128_as_number(TEMPORAL_INSTANT_EPOCH_MILLISECONDS_MAX);
    if !(-max..=max).contains(&number) {
        return Err(range_error(cx));
    }
    Ok(temporal_number_to_i128_after_range_check(number))
}

fn temporal_instant_compare_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = temporal_instant_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let right = temporal_instant_from_value(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(temporal_compare_ordering(left.cmp(&right)))
}

fn temporal_instant_epoch_nanoseconds_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, invocation.this_value(), TemporalObjectKind::Instant)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::Instant(data) = payload else {
        return Err(type_error(cx));
    };
    Ok(temporal_i128_to_bigint_value(
        cx.agent(),
        data.epoch_nanoseconds(),
    ))
}

fn temporal_instant_epoch_milliseconds_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, invocation.this_value(), TemporalObjectKind::Instant)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::Instant(data) = payload else {
        return Err(type_error(cx));
    };
    temporal_safe_integer_number(
        cx,
        data.epoch_nanoseconds()
            .div_euclid(TEMPORAL_NANOS_PER_MILLISECOND),
    )
}

fn temporal_instant_epoch_seconds_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, invocation.this_value(), TemporalObjectKind::Instant)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::Instant(data) = payload else {
        return Err(type_error(cx));
    };
    temporal_safe_integer_number(
        cx,
        data.epoch_nanoseconds()
            .div_euclid(TEMPORAL_NANOS_PER_SECOND),
    )
}

fn temporal_instant_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, invocation.this_value(), TemporalObjectKind::Instant)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::Instant(data) = payload else {
        return Err(type_error(cx));
    };
    let options = temporal_instant_to_string_with_time_zone_options(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let epoch_nanoseconds = match options.precision {
        TemporalInstantStringPrecision::Auto => data.epoch_nanoseconds(),
        TemporalInstantStringPrecision::Minute => temporal_round_epoch_nanoseconds_to_increment(
            data.epoch_nanoseconds(),
            TEMPORAL_NANOS_PER_MINUTE,
            options.rounding_mode,
        )
        .ok_or_else(|| range_error(cx))?,
        TemporalInstantStringPrecision::FractionalSecond(digits) => {
            temporal_round_epoch_nanoseconds_to_fractional_digits(
                data.epoch_nanoseconds(),
                digits,
                options.rounding_mode,
            )
            .ok_or_else(|| range_error(cx))?
        }
    };
    if !temporal_instant_epoch_nanoseconds_is_valid(epoch_nanoseconds) {
        return Err(range_error(cx));
    }
    let text = if let Some(time_zone_id) = options.time_zone_id.as_deref() {
        format_temporal_instant_with_time_zone(
            cx,
            epoch_nanoseconds,
            options.precision,
            time_zone_id,
        )?
    } else {
        format_temporal_instant_utc(cx, epoch_nanoseconds, options.precision)?
    };
    Ok(string_value(cx, &text))
}

fn temporal_instant_to_json_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, invocation.this_value(), TemporalObjectKind::Instant)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::Instant(data) = payload else {
        return Err(type_error(cx));
    };
    let text = format_temporal_instant_utc(
        cx,
        data.epoch_nanoseconds(),
        TemporalInstantStringPrecision::Auto,
    )?;
    Ok(string_value(cx, &text))
}

fn temporal_instant_to_locale_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, invocation.this_value(), TemporalObjectKind::Instant)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::Instant(data) = payload else {
        return Err(type_error(cx));
    };
    let text = format_temporal_instant_utc(
        cx,
        data.epoch_nanoseconds(),
        TemporalInstantStringPrecision::Auto,
    )?;
    Ok(string_value(cx, &text))
}

fn temporal_instant_to_string_with_time_zone_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalInstantToStringOptions, Cx::Error> {
    if value.is_undefined() {
        return Ok(TemporalInstantToStringOptions {
            precision: TemporalInstantStringPrecision::Auto,
            rounding_mode: TemporalBuiltinRoundingMode::Trunc,
            time_zone_id: None,
        });
    }
    let Some(object_ref) = value.as_object_ref() else {
        return Err(type_error(cx));
    };

    let fractional_second_digits =
        temporal_property_value(cx, object_ref, "fractionalSecondDigits")?;
    let fractional_second_precision =
        temporal_instant_fractional_second_digits_option(cx, fractional_second_digits)?;
    let rounding_mode = temporal_property_value(cx, object_ref, "roundingMode")?;
    let rounding_mode = temporal_duration_rounding_mode_option(cx, rounding_mode)?;
    let smallest_unit = temporal_property_value(cx, object_ref, "smallestUnit")?;
    let smallest_unit = if smallest_unit.is_undefined() {
        None
    } else {
        let string_ref = to_string_string_ref(cx, smallest_unit)?;
        Some(string_ref_text(cx, string_ref)?)
    };
    let time_zone = temporal_property_value(cx, object_ref, "timeZone")?;
    let precision = if let Some(unit) = smallest_unit.as_deref() {
        temporal_instant_smallest_unit_precision_from_text(cx, unit)?
    } else {
        fractional_second_precision
    };
    let time_zone_id = if time_zone.is_undefined() {
        None
    } else {
        Some(temporal_time_zone_id_from_value(cx, time_zone)?)
    };
    Ok(TemporalInstantToStringOptions {
        precision,
        rounding_mode,
        time_zone_id,
    })
}

fn temporal_instant_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Err(type_error(cx))
}

fn temporal_instant_equals_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, invocation.this_value(), TemporalObjectKind::Instant)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::Instant(data) = payload else {
        return Err(type_error(cx));
    };
    let other = temporal_instant_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(data.epoch_nanoseconds() == other))
}

fn temporal_instant_add_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_instant_additive_builtin(cx, invocation, 1)
}

fn temporal_instant_subtract_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_instant_additive_builtin(cx, invocation, -1)
}

fn temporal_instant_additive_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    sign: i128,
) -> Result<Value, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, invocation.this_value(), TemporalObjectKind::Instant)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::Instant(data) = payload else {
        return Err(type_error(cx));
    };
    let duration = temporal_duration_from_additive_argument(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if temporal_ops::duration_has_calendar_relative_units(duration) || duration.days() != 0 {
        return Err(range_error(cx));
    }
    let delta = temporal_duration_time_nanoseconds(duration)
        .checked_mul(sign)
        .ok_or_else(|| range_error(cx))?;
    let epoch_nanoseconds = data
        .epoch_nanoseconds()
        .checked_add(delta)
        .ok_or_else(|| range_error(cx))?;
    let prototype = current_temporal_instant_prototype(cx)?;
    allocate_temporal_instant_object(cx, prototype, epoch_nanoseconds)
}

fn temporal_instant_round_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, invocation.this_value(), TemporalObjectKind::Instant)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::Instant(data) = payload else {
        return Err(type_error(cx));
    };
    let options = temporal_instant_round_options(
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
    let epoch_nanoseconds = temporal_round_epoch_nanoseconds_to_increment(
        data.epoch_nanoseconds(),
        increment,
        options.rounding_mode,
    )
    .ok_or_else(|| range_error(cx))?;
    let prototype = current_temporal_instant_prototype(cx)?;
    allocate_temporal_instant_object(cx, prototype, epoch_nanoseconds)
}

struct TemporalInstantDifferenceOptions {
    largest_unit: TemporalBuiltinDurationExactUnit,
    smallest_unit: TemporalBuiltinDurationExactUnit,
    rounding_increment: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
}

fn temporal_instant_difference_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalInstantDifferenceOptions, Cx::Error> {
    if !value.is_undefined() {
        let Some(object_ref) = value.as_object_ref() else {
            return Err(type_error(cx));
        };
        let largest_unit_value = temporal_property_value(cx, object_ref, "largestUnit")?;
        let largest_unit_text = temporal_option_string_text(cx, largest_unit_value)?;
        let rounding_increment_value =
            temporal_property_value(cx, object_ref, "roundingIncrement")?;
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
                temporal_instant_largest_unit_default(smallest_unit)
            } else {
                temporal_exact_time_unit_from_text(text).ok_or_else(|| range_error(cx))?
            }
        } else {
            temporal_instant_largest_unit_default(smallest_unit)
        };
        if temporal_exact_time_unit_order(largest_unit)
            > temporal_exact_time_unit_order(smallest_unit)
            || !temporal_exact_time_rounding_increment_is_valid(smallest_unit, rounding_increment)
        {
            return Err(range_error(cx));
        }
        return Ok(TemporalInstantDifferenceOptions {
            largest_unit,
            smallest_unit,
            rounding_increment,
            rounding_mode,
        });
    }

    Ok(TemporalInstantDifferenceOptions {
        largest_unit: TemporalBuiltinDurationExactUnit::Second,
        smallest_unit: TemporalBuiltinDurationExactUnit::Nanosecond,
        rounding_increment: 1,
        rounding_mode: TemporalBuiltinRoundingMode::Trunc,
    })
}

fn temporal_instant_difference_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    sign: i128,
) -> Result<Value, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, invocation.this_value(), TemporalObjectKind::Instant)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::Instant(data) = payload else {
        return Err(type_error(cx));
    };
    let other = temporal_instant_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let options = temporal_instant_difference_options(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let raw_difference = data
        .epoch_nanoseconds()
        .checked_sub(other)
        .and_then(|difference| difference.checked_mul(sign))
        .ok_or_else(|| range_error(cx))?;
    let increment = temporal_exact_time_unit_nanoseconds(options.smallest_unit)
        .checked_mul(options.rounding_increment)
        .ok_or_else(|| range_error(cx))?;
    let rounded = temporal_round_duration_nanoseconds_to_increment(
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

fn temporal_instant_since_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_instant_difference_builtin(cx, invocation, 1)
}

fn temporal_instant_until_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_instant_difference_builtin(cx, invocation, -1)
}

fn temporal_instant_to_zoned_date_time_iso_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, invocation.this_value(), TemporalObjectKind::Instant)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::Instant(data) = payload else {
        return Err(type_error(cx));
    };
    let time_zone_id = temporal_time_zone_id_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let zoned = temporal_zoned_date_time_from_parts(cx, data.epoch_nanoseconds(), &time_zone_id)?;
    let prototype = current_temporal_zoned_date_time_prototype(cx)?;
    allocate_temporal_zoned_date_time_object(cx, prototype, zoned)
}

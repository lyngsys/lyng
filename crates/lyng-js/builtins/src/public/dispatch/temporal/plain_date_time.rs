use super::{
    allocate_temporal_duration_object, allocate_temporal_plain_date_object,
    allocate_temporal_zoned_date_time_object, current_temporal_duration_prototype,
    current_temporal_plain_date_prototype, current_temporal_plain_date_time_prototype,
    current_temporal_plain_time_prototype, current_temporal_zoned_date_time_prototype,
    format_temporal_plain_date, format_temporal_plain_time_with_precision, map_completion,
    negate_temporal_duration, object, parse_temporal_plain_date_time, plain_time, range_error,
    string_ref_text, string_value, temporal_civil_date_time_from_plain_date_time,
    temporal_compare_ordering, temporal_constructor_prototype,
    temporal_duration_from_nanoseconds_with_largest_unit, temporal_duration_from_value,
    temporal_duration_round_calendar_relative, temporal_duration_round_calendar_relative_exact,
    temporal_duration_rounding_increment_option, temporal_duration_rounding_mode_option,
    temporal_duration_rounding_mode_option_with_default, temporal_duration_time_nanoseconds,
    temporal_exact_time_rounding_increment_is_valid,
    temporal_instant_fractional_second_digits_option, temporal_instant_smallest_unit_precision,
    temporal_integer_part_from_argument, temporal_is_iso_leap_year, temporal_iso_day_of_week,
    temporal_iso_day_of_year, temporal_iso_days_in_month, temporal_iso_days_in_year,
    temporal_iso_week_of_year, temporal_month_from_month_code_text,
    temporal_month_from_property_bag, temporal_optional_integer_part_from_property,
    temporal_optional_time_part_from_property, temporal_overflow_from_options,
    temporal_plain_date_add_duration, temporal_plain_date_difference_trunc,
    temporal_plain_date_from_ordinal_day, temporal_plain_date_from_parts_with_overflow,
    temporal_plain_date_ordinal_day, temporal_plain_date_time_bag_fields,
    temporal_plain_date_time_is_within_limits, temporal_plain_time_from_nanoseconds,
    temporal_plain_time_from_parts_with_overflow, temporal_plain_time_from_value,
    temporal_plain_time_nanoseconds, temporal_property_value,
    temporal_reject_calendar_or_time_zone_properties, temporal_required_integer_part_from_property,
    temporal_resolve_month_from_fields, temporal_round_duration_nanoseconds_to_increment,
    temporal_round_epoch_nanoseconds_to_fractional_digits,
    temporal_round_epoch_nanoseconds_to_increment, temporal_rounding_mode_for_negated_duration,
    temporal_string_option, temporal_subsecond_parts_from_nanoseconds,
    temporal_time_part_from_argument, temporal_time_part_from_property,
    temporal_time_zone_id_from_value, temporal_validate_iso_calendar_value,
    temporal_validate_optional_iso_calendar_identifier_argument,
    temporal_validate_optional_iso_calendar_property, temporal_validate_options_object,
    temporal_zoned_date_time_civil, temporal_zoned_date_time_from_parts, to_string_string_ref,
    type_error, validate_temporal_duration, AllocationLifetime, BuiltinFunctionId,
    BuiltinInvocation, ObjectAllocation, ObjectColdData, ObjectRef, OrdinaryObjectData,
    PublicBuiltinDispatchContext, TemporalBuiltinDurationExactUnit, TemporalBuiltinRoundingMode,
    TemporalCivilToInstantRequest, TemporalDateDifferenceUnit, TemporalDisambiguation,
    TemporalDurationObjectData, TemporalDurationRelativeTo, TemporalInstantStringPrecision,
    TemporalObjectData, TemporalObjectKind, TemporalOverflow, TemporalPlainDateObjectData,
    TemporalPlainDateTimeBagFields, TemporalPlainDateTimeObjectData, TemporalPlainTimeObjectData,
    TemporalZonedDateTimeCalendarNameOption, Value, TEMPORAL_NANOS_PER_DAY,
    TEMPORAL_NANOS_PER_HOUR, TEMPORAL_NANOS_PER_MICROSECOND, TEMPORAL_NANOS_PER_MILLISECOND,
    TEMPORAL_NANOS_PER_MINUTE, TEMPORAL_NANOS_PER_SECOND,
};

#[allow(
    clippy::too_many_lines,
    reason = "Temporal PlainDateTime dispatch is the builtin ID switchboard for this spec domain"
)]
pub(super) fn dispatch_temporal_plain_date_time_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::temporal_plain_date_time_builtin() {
        return temporal_plain_date_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_year_getter_builtin() {
        return temporal_plain_date_time_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_month_getter_builtin() {
        return temporal_plain_date_time_month_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_month_code_getter_builtin() {
        return temporal_plain_date_time_month_code_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_day_getter_builtin() {
        return temporal_plain_date_time_day_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_day_of_week_getter_builtin() {
        return temporal_plain_date_time_day_of_week_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_day_of_year_getter_builtin() {
        return temporal_plain_date_time_day_of_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_days_in_month_getter_builtin() {
        return temporal_plain_date_time_days_in_month_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_days_in_year_getter_builtin() {
        return temporal_plain_date_time_days_in_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_months_in_year_getter_builtin() {
        return temporal_plain_date_time_months_in_year_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_in_leap_year_getter_builtin() {
        return temporal_plain_date_time_in_leap_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_days_in_week_getter_builtin() {
        return temporal_plain_date_time_days_in_week_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_week_of_year_getter_builtin() {
        return temporal_plain_date_time_week_of_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_year_of_week_getter_builtin() {
        return temporal_plain_date_time_year_of_week_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_era_getter_builtin() {
        return temporal_plain_date_time_era_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_era_year_getter_builtin() {
        return temporal_plain_date_time_era_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_hour_getter_builtin() {
        return temporal_plain_date_time_hour_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_minute_getter_builtin() {
        return temporal_plain_date_time_minute_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_second_getter_builtin() {
        return temporal_plain_date_time_second_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_millisecond_getter_builtin() {
        return temporal_plain_date_time_millisecond_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_microsecond_getter_builtin() {
        return temporal_plain_date_time_microsecond_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_nanosecond_getter_builtin() {
        return temporal_plain_date_time_nanosecond_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_calendar_id_getter_builtin() {
        return temporal_plain_date_time_calendar_id_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_to_string_builtin() {
        return temporal_plain_date_time_to_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_to_json_builtin() {
        return temporal_plain_date_time_to_json_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_to_locale_string_builtin() {
        return temporal_plain_date_time_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_value_of_builtin() {
        return temporal_plain_date_time_value_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_equals_builtin() {
        return temporal_plain_date_time_equals_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_with_builtin() {
        return temporal_plain_date_time_with_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_with_plain_time_builtin() {
        return temporal_plain_date_time_with_plain_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_with_calendar_builtin() {
        return temporal_plain_date_time_with_calendar_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_add_builtin() {
        return temporal_plain_date_time_add_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_subtract_builtin() {
        return temporal_plain_date_time_subtract_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_round_builtin() {
        return temporal_plain_date_time_round_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_since_builtin() {
        return temporal_plain_date_time_since_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_until_builtin() {
        return temporal_plain_date_time_until_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_to_plain_date_builtin() {
        return temporal_plain_date_time_to_plain_date_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_to_plain_time_builtin() {
        return temporal_plain_date_time_to_plain_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_to_zoned_date_time_builtin() {
        return temporal_plain_date_time_to_zoned_date_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_from_builtin() {
        return temporal_plain_date_time_from_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_time_compare_builtin() {
        return temporal_plain_date_time_compare_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

#[allow(
    clippy::too_many_arguments,
    reason = "PlainDateTime construction takes the explicit ECMA date and time fields"
)]
pub(super) fn temporal_plain_date_time_from_parts<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
    millisecond: i64,
    microsecond: i64,
    nanosecond: i64,
) -> Result<TemporalPlainDateTimeObjectData, Cx::Error> {
    temporal_plain_date_time_from_parts_with_overflow(
        cx,
        year,
        month,
        day,
        hour,
        minute,
        second,
        millisecond,
        microsecond,
        nanosecond,
        TemporalOverflow::Reject,
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "PlainDateTime construction takes the explicit ECMA date and time fields plus overflow"
)]
pub(super) fn temporal_plain_date_time_from_parts_with_overflow<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
    millisecond: i64,
    microsecond: i64,
    nanosecond: i64,
    overflow: TemporalOverflow,
) -> Result<TemporalPlainDateTimeObjectData, Cx::Error> {
    let date = temporal_plain_date_from_parts_with_overflow(cx, year, month, day, overflow)?;
    let time = temporal_plain_time_from_parts_with_overflow(
        cx,
        hour,
        minute,
        second,
        millisecond,
        microsecond,
        nanosecond,
        overflow,
    )?;
    let date_time_data = TemporalPlainDateTimeObjectData::new(
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
    let total_nanoseconds = temporal_plain_date_time_total_nanoseconds(date_time_data)
        .ok_or_else(|| range_error(cx))?;
    if !temporal_plain_date_time_is_within_limits(date_time_data.calendar(), total_nanoseconds) {
        return Err(range_error(cx));
    }
    Ok(date_time_data)
}

pub(super) fn allocate_temporal_plain_date_time_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: ObjectRef,
    data: TemporalPlainDateTimeObjectData,
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
                        TemporalObjectKind::PlainDateTime,
                    ))),
                AllocationLifetime::Default,
            )
        })
    };
    let installed = cx
        .agent()
        .objects_mut()
        .install_temporal_object(object, TemporalObjectData::PlainDateTime(data));
    if !installed {
        return Err(type_error(cx));
    }
    Ok(Value::from_object_ref(object))
}

pub(super) fn temporal_plain_date_time_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalPlainDateTimeObjectData, Cx::Error> {
    if let Some(string_ref) = value.as_string_ref() {
        let text = string_ref_text(cx, string_ref)?;
        let parsed = parse_temporal_plain_date_time(&text).ok_or_else(|| range_error(cx))?;
        let (millisecond, microsecond, nanosecond) =
            temporal_subsecond_parts_from_nanoseconds(cx, parsed.fraction_nanoseconds)?;
        return temporal_plain_date_time_from_parts(
            cx,
            i64::from(parsed.year),
            i64::from(parsed.month),
            i64::from(parsed.day),
            i64::from(parsed.hour),
            i64::from(parsed.minute),
            i64::from(parsed.second),
            millisecond,
            microsecond,
            nanosecond,
        );
    }

    let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let existing = {
        let agent = cx.agent();
        agent.objects().temporal_object(object_ref).copied()
    };
    match existing {
        Some(TemporalObjectData::PlainDateTime(data)) => return Ok(data),
        Some(TemporalObjectData::PlainDate(data)) => {
            return Ok(TemporalPlainDateTimeObjectData::new(
                data.year(),
                data.month(),
                data.day(),
                0,
                0,
                0,
                0,
                0,
                0,
                data.calendar(),
            ));
        }
        Some(TemporalObjectData::ZonedDateTime(data)) => {
            let civil = temporal_zoned_date_time_civil(cx, data)?;
            let date_time = civil.date_time;
            return temporal_plain_date_time_from_parts(
                cx,
                i64::from(date_time.year),
                i64::from(date_time.month),
                i64::from(date_time.day),
                i64::from(date_time.hour),
                i64::from(date_time.minute),
                i64::from(date_time.second),
                i64::from(date_time.millisecond),
                i64::from(date_time.microsecond),
                i64::from(date_time.nanosecond),
            );
        }
        _ => {}
    }

    temporal_validate_optional_iso_calendar_property(cx, object_ref)?;
    let day = temporal_required_integer_part_from_property(cx, object_ref, "day")?;
    let hour = temporal_time_part_from_property(cx, object_ref, "hour")?;
    let microsecond = temporal_time_part_from_property(cx, object_ref, "microsecond")?;
    let millisecond = temporal_time_part_from_property(cx, object_ref, "millisecond")?;
    let minute = temporal_time_part_from_property(cx, object_ref, "minute")?;
    let month = temporal_month_from_property_bag(cx, object_ref, None)?;
    let nanosecond = temporal_time_part_from_property(cx, object_ref, "nanosecond")?;
    let second = temporal_time_part_from_property(cx, object_ref, "second")?;
    let second = if second == 60 { 59 } else { second };
    let year = temporal_required_integer_part_from_property(cx, object_ref, "year")?;
    temporal_plain_date_time_from_parts(
        cx,
        year,
        month,
        day,
        hour,
        minute,
        second,
        millisecond,
        microsecond,
        nanosecond,
    )
}

pub(super) fn temporal_plain_date_time_from_bag_fields<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    fields: &TemporalPlainDateTimeBagFields,
    overflow: TemporalOverflow,
) -> Result<TemporalPlainDateTimeObjectData, Cx::Error> {
    let year = fields.year.ok_or_else(|| type_error(cx))?;
    let day = fields.day.ok_or_else(|| type_error(cx))?;
    let month = temporal_resolve_month_from_fields(
        cx,
        fields.month,
        fields.month_code_text.as_deref(),
        None,
    )?;
    temporal_plain_date_time_from_parts_with_overflow(
        cx,
        year,
        month,
        day,
        fields.hour,
        fields.minute,
        fields.second,
        fields.millisecond,
        fields.microsecond,
        fields.nanosecond,
        overflow,
    )
}

pub(super) fn temporal_plain_date_time_data<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    this_value: Value,
) -> Result<TemporalPlainDateTimeObjectData, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, this_value, TemporalObjectKind::PlainDateTime)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::PlainDateTime(data) = payload else {
        return Err(type_error(cx));
    };
    Ok(data)
}

pub(super) fn temporal_plain_date_time_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let Some(new_target) = invocation.new_target() else {
        return Err(type_error(cx));
    };
    let year = temporal_integer_part_from_argument(cx, invocation, 0)?;
    let month = temporal_integer_part_from_argument(cx, invocation, 1)?;
    let day = temporal_integer_part_from_argument(cx, invocation, 2)?;
    let hour = temporal_time_part_from_argument(cx, invocation, 3)?;
    let minute = temporal_time_part_from_argument(cx, invocation, 4)?;
    let second = temporal_time_part_from_argument(cx, invocation, 5)?;
    let millisecond = temporal_time_part_from_argument(cx, invocation, 6)?;
    let microsecond = temporal_time_part_from_argument(cx, invocation, 7)?;
    let nanosecond = temporal_time_part_from_argument(cx, invocation, 8)?;
    temporal_validate_optional_iso_calendar_identifier_argument(cx, invocation, 9)?;
    let data = temporal_plain_date_time_from_parts(
        cx,
        year,
        month,
        day,
        hour,
        minute,
        second,
        millisecond,
        microsecond,
        nanosecond,
    )?;
    let prototype = temporal_constructor_prototype(cx, new_target)?;
    allocate_temporal_plain_date_time_object(cx, prototype, data)
}

pub(super) fn temporal_plain_date_time_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(data.year()))
}

pub(super) fn temporal_plain_date_time_month_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.month())))
}

pub(super) fn temporal_plain_date_time_month_code_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format!("M{:02}", data.month())))
}

pub(super) fn temporal_plain_date_time_day_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.day())))
}

pub(super) fn temporal_plain_date_time_day_of_week_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(temporal_iso_day_of_week(
        data.year(),
        data.month(),
        data.day(),
    )))
}

pub(super) fn temporal_plain_date_time_day_of_year_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(temporal_iso_day_of_year(
        data.year(),
        data.month(),
        data.day(),
    )))
}

pub(super) fn temporal_plain_date_time_days_in_month_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(temporal_iso_days_in_month(
        data.year(),
        data.month(),
    ))))
}

pub(super) fn temporal_plain_date_time_days_in_year_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(temporal_iso_days_in_year(data.year())))
}

pub(super) fn temporal_plain_date_time_months_in_year_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(12))
}

pub(super) fn temporal_plain_date_time_in_leap_year_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_bool(temporal_is_iso_leap_year(data.year())))
}

pub(super) fn temporal_plain_date_time_days_in_week_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(7))
}

pub(super) fn temporal_plain_date_time_week_of_year_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    let (week, _) = temporal_iso_week_of_year(data.year(), data.month(), data.day());
    Ok(Value::from_smi(week))
}

pub(super) fn temporal_plain_date_time_year_of_week_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    let (_, year) = temporal_iso_week_of_year(data.year(), data.month(), data.day());
    Ok(Value::from_smi(year))
}

pub(super) fn temporal_plain_date_time_era_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::undefined())
}

pub(super) fn temporal_plain_date_time_era_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::undefined())
}

pub(super) fn temporal_plain_date_time_hour_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.hour())))
}

pub(super) fn temporal_plain_date_time_minute_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.minute())))
}

pub(super) fn temporal_plain_date_time_second_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.second())))
}

pub(super) fn temporal_plain_date_time_millisecond_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.millisecond())))
}

pub(super) fn temporal_plain_date_time_microsecond_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.microsecond())))
}

pub(super) fn temporal_plain_date_time_nanosecond_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.nanosecond())))
}

pub(super) fn temporal_plain_date_time_calendar_id_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(string_value(cx, "iso8601"))
}

pub(super) fn temporal_plain_date_time_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    let options = temporal_plain_date_time_to_string_options(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let data = temporal_plain_date_time_for_string_precision(
        cx,
        data,
        options.precision,
        options.rounding_mode,
    )?;
    Ok(string_value(
        cx,
        &format_temporal_plain_date_time_with_options(data, options),
    ))
}

#[derive(Clone, Copy)]
pub(super) struct TemporalPlainDateTimeToStringOptions {
    pub(super) precision: TemporalInstantStringPrecision,
    pub(super) rounding_mode: TemporalBuiltinRoundingMode,
    pub(super) calendar_name: TemporalZonedDateTimeCalendarNameOption,
}

pub(super) fn temporal_plain_date_time_to_string_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalPlainDateTimeToStringOptions, Cx::Error> {
    if value.is_undefined() {
        return Ok(TemporalPlainDateTimeToStringOptions {
            precision: TemporalInstantStringPrecision::Auto,
            rounding_mode: TemporalBuiltinRoundingMode::Trunc,
            calendar_name: TemporalZonedDateTimeCalendarNameOption::Auto,
        });
    }
    let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let calendar_name_value = temporal_property_value(cx, object_ref, "calendarName")?;
    let calendar_name = match temporal_string_option(
        cx,
        calendar_name_value,
        &["auto", "always", "never", "critical"],
        "auto",
    )?
    .as_str()
    {
        "auto" => TemporalZonedDateTimeCalendarNameOption::Auto,
        "always" => TemporalZonedDateTimeCalendarNameOption::Always,
        "never" => TemporalZonedDateTimeCalendarNameOption::Never,
        "critical" => TemporalZonedDateTimeCalendarNameOption::Critical,
        _ => unreachable!("temporal_string_option constrained calendarName"),
    };
    let fractional_second_digits =
        temporal_property_value(cx, object_ref, "fractionalSecondDigits")?;
    let fractional_second_precision =
        temporal_instant_fractional_second_digits_option(cx, fractional_second_digits)?;
    let rounding_mode_value = temporal_property_value(cx, object_ref, "roundingMode")?;
    let rounding_mode = temporal_duration_rounding_mode_option(cx, rounding_mode_value)?;
    let smallest_unit = temporal_property_value(cx, object_ref, "smallestUnit")?;
    let precision = if smallest_unit.is_undefined() {
        fractional_second_precision
    } else {
        temporal_instant_smallest_unit_precision(cx, smallest_unit)?
    };
    Ok(TemporalPlainDateTimeToStringOptions {
        precision,
        rounding_mode,
        calendar_name,
    })
}

pub(super) fn temporal_plain_date_time_for_string_precision<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    data: TemporalPlainDateTimeObjectData,
    precision: TemporalInstantStringPrecision,
    rounding_mode: TemporalBuiltinRoundingMode,
) -> Result<TemporalPlainDateTimeObjectData, Cx::Error> {
    let total_nanoseconds = match precision {
        TemporalInstantStringPrecision::Auto => return Ok(data),
        TemporalInstantStringPrecision::Minute => temporal_round_epoch_nanoseconds_to_increment(
            temporal_plain_date_time_total_nanoseconds(data).ok_or_else(|| range_error(cx))?,
            TEMPORAL_NANOS_PER_MINUTE,
            rounding_mode,
        )
        .ok_or_else(|| range_error(cx))?,
        TemporalInstantStringPrecision::FractionalSecond(digits) => {
            temporal_round_epoch_nanoseconds_to_fractional_digits(
                temporal_plain_date_time_total_nanoseconds(data).ok_or_else(|| range_error(cx))?,
                digits,
                rounding_mode,
            )
            .ok_or_else(|| range_error(cx))?
        }
    };
    if !temporal_plain_date_time_is_within_limits(data.calendar(), total_nanoseconds) {
        return Err(range_error(cx));
    }
    temporal_plain_date_time_from_total_nanoseconds(cx, total_nanoseconds)
}

pub(super) fn format_temporal_plain_date_time_with_options(
    data: TemporalPlainDateTimeObjectData,
    options: TemporalPlainDateTimeToStringOptions,
) -> String {
    let mut text = {
        let date_part = temporal_plain_date_time_date(data);
        let time_part = temporal_plain_date_time_time(data);
        format!(
            "{}T{}",
            format_temporal_plain_date(date_part),
            format_temporal_plain_time_with_precision(time_part, options.precision)
        )
    };
    match options.calendar_name {
        TemporalZonedDateTimeCalendarNameOption::Always => {
            text.push_str("[u-ca=iso8601]");
        }
        TemporalZonedDateTimeCalendarNameOption::Critical => {
            text.push_str("[!u-ca=iso8601]");
        }
        TemporalZonedDateTimeCalendarNameOption::Auto
        | TemporalZonedDateTimeCalendarNameOption::Never => {}
    }
    text
}

pub(super) fn temporal_plain_date_time_to_json_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(string_value(
        cx,
        &format_temporal_plain_date_time_with_options(
            data,
            TemporalPlainDateTimeToStringOptions {
                precision: TemporalInstantStringPrecision::Auto,
                rounding_mode: TemporalBuiltinRoundingMode::Trunc,
                calendar_name: TemporalZonedDateTimeCalendarNameOption::Auto,
            },
        ),
    ))
}

pub(super) fn temporal_plain_date_time_to_locale_string_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_plain_date_time_to_json_builtin(cx, invocation)
}

pub(super) fn temporal_plain_date_time_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Err(type_error(cx))
}

pub(super) fn temporal_plain_date_time_equals_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = temporal_plain_date_time_data(cx, invocation.this_value())?;
    let right = temporal_plain_date_time_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(left == right))
}

pub(super) fn temporal_plain_date_time_with_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let date_time = temporal_plain_date_time_data(cx, invocation.this_value())?;
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
    let day_value = temporal_optional_integer_part_from_property(cx, object_ref, "day")?;
    let day = day_value.unwrap_or_else(|| i64::from(date_time.day()));
    let hour_value = temporal_optional_time_part_from_property(cx, object_ref, "hour")?;
    let hour = hour_value.unwrap_or_else(|| i64::from(date_time.hour()));
    let microsecond_value =
        temporal_optional_time_part_from_property(cx, object_ref, "microsecond")?;
    let microsecond = microsecond_value.unwrap_or_else(|| i64::from(date_time.microsecond()));
    let millisecond_value =
        temporal_optional_time_part_from_property(cx, object_ref, "millisecond")?;
    let millisecond = millisecond_value.unwrap_or_else(|| i64::from(date_time.millisecond()));
    let minute_value = temporal_optional_time_part_from_property(cx, object_ref, "minute")?;
    let minute = minute_value.unwrap_or_else(|| i64::from(date_time.minute()));
    let month_value = temporal_optional_integer_part_from_property(cx, object_ref, "month")?;
    let month_code_value = temporal_property_value(cx, object_ref, "monthCode")?;
    let month_code_text = if month_code_value.is_undefined() {
        None
    } else {
        let string_ref = to_string_string_ref(cx, month_code_value)?;
        Some(string_ref_text(cx, string_ref)?)
    };
    let nanosecond_value = temporal_optional_time_part_from_property(cx, object_ref, "nanosecond")?;
    let nanosecond = nanosecond_value.unwrap_or_else(|| i64::from(date_time.nanosecond()));
    let second_value = temporal_optional_time_part_from_property(cx, object_ref, "second")?;
    let second = second_value.unwrap_or_else(|| i64::from(date_time.second()));
    let year = temporal_optional_integer_part_from_property(cx, object_ref, "year")?;
    if year.is_none()
        && month_value.is_none()
        && month_code_value.is_undefined()
        && day_value.is_none()
        && hour_value.is_none()
        && microsecond_value.is_none()
        && millisecond_value.is_none()
        && minute_value.is_none()
        && nanosecond_value.is_none()
        && second_value.is_none()
    {
        return Err(type_error(cx));
    }
    let options = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let overflow = if options.is_undefined() || options.as_object_ref().is_some() {
        temporal_overflow_from_options(cx, options)?
    } else {
        TemporalOverflow::Constrain
    };
    let month = if let Some(month) = month_value {
        if let Some(text) = month_code_text.as_deref() {
            let month_code =
                temporal_month_from_month_code_text(text).ok_or_else(|| range_error(cx))?;
            if month != month_code {
                return Err(range_error(cx));
            }
        }
        month
    } else if let Some(text) = month_code_text.as_deref() {
        temporal_month_from_month_code_text(text).ok_or_else(|| range_error(cx))?
    } else {
        i64::from(date_time.month())
    };
    let data = temporal_plain_date_time_from_parts_with_overflow(
        cx,
        year.unwrap_or_else(|| i64::from(date_time.year())),
        month,
        day,
        hour,
        minute,
        second,
        millisecond,
        microsecond,
        nanosecond,
        overflow,
    )?;
    if !options.is_undefined() && options.as_object_ref().is_none() {
        return Err(type_error(cx));
    }
    let prototype = current_temporal_plain_date_time_prototype(cx)?;
    allocate_temporal_plain_date_time_object(cx, prototype, data)
}

pub(super) fn temporal_plain_date_time_with_plain_time_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let date_time = temporal_plain_date_time_data(cx, invocation.this_value())?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let time = if value.is_undefined() {
        TemporalPlainTimeObjectData::new(0, 0, 0, 0, 0, 0)
    } else {
        temporal_plain_time_from_value(cx, value)?
    };
    let checked = temporal_plain_date_time_from_parts(
        cx,
        i64::from(date_time.year()),
        i64::from(date_time.month()),
        i64::from(date_time.day()),
        i64::from(time.hour()),
        i64::from(time.minute()),
        i64::from(time.second()),
        i64::from(time.millisecond()),
        i64::from(time.microsecond()),
        i64::from(time.nanosecond()),
    )?;
    let data = TemporalPlainDateTimeObjectData::new(
        checked.year(),
        checked.month(),
        checked.day(),
        checked.hour(),
        checked.minute(),
        checked.second(),
        checked.millisecond(),
        checked.microsecond(),
        checked.nanosecond(),
        date_time.calendar(),
    );
    let prototype = current_temporal_plain_date_time_prototype(cx)?;
    allocate_temporal_plain_date_time_object(cx, prototype, data)
}

pub(super) fn temporal_plain_date_time_with_calendar_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let date_time = temporal_plain_date_time_data(cx, invocation.this_value())?;
    temporal_validate_iso_calendar_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let data = TemporalPlainDateTimeObjectData::new(
        date_time.year(),
        date_time.month(),
        date_time.day(),
        date_time.hour(),
        date_time.minute(),
        date_time.second(),
        date_time.millisecond(),
        date_time.microsecond(),
        date_time.nanosecond(),
        date_time.calendar(),
    );
    let prototype = current_temporal_plain_date_time_prototype(cx)?;
    allocate_temporal_plain_date_time_object(cx, prototype, data)
}

pub(super) const fn temporal_plain_date_time_date(
    data: TemporalPlainDateTimeObjectData,
) -> TemporalPlainDateObjectData {
    TemporalPlainDateObjectData::new(data.year(), data.month(), data.day(), data.calendar())
}

pub(super) const fn temporal_plain_date_time_time(
    data: TemporalPlainDateTimeObjectData,
) -> TemporalPlainTimeObjectData {
    TemporalPlainTimeObjectData::new(
        data.hour(),
        data.minute(),
        data.second(),
        data.millisecond(),
        data.microsecond(),
        data.nanosecond(),
    )
}

pub(super) fn temporal_plain_date_time_total_nanoseconds(
    data: TemporalPlainDateTimeObjectData,
) -> Option<i128> {
    temporal_plain_date_ordinal_day(temporal_plain_date_time_date(data))
        .checked_mul(TEMPORAL_NANOS_PER_DAY)?
        .checked_add(temporal_plain_time_nanoseconds(
            temporal_plain_date_time_time(data),
        ))
}

pub(super) fn temporal_plain_date_time_from_total_nanoseconds<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    total_nanoseconds: i128,
) -> Result<TemporalPlainDateTimeObjectData, Cx::Error> {
    let ordinal_day = total_nanoseconds.div_euclid(TEMPORAL_NANOS_PER_DAY);
    let time_nanoseconds = total_nanoseconds.rem_euclid(TEMPORAL_NANOS_PER_DAY);
    let date = temporal_plain_date_from_ordinal_day(cx, ordinal_day)?;
    let time = temporal_plain_time_from_nanoseconds(cx, time_nanoseconds)?;
    let date_time_data = TemporalPlainDateTimeObjectData::new(
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
    let total_nanoseconds = temporal_plain_date_time_total_nanoseconds(date_time_data)
        .ok_or_else(|| range_error(cx))?;
    if !temporal_plain_date_time_is_within_limits(date_time_data.calendar(), total_nanoseconds) {
        return Err(range_error(cx));
    }
    Ok(date_time_data)
}

pub(super) fn temporal_plain_date_time_add_duration<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    data: TemporalPlainDateTimeObjectData,
    duration: TemporalDurationObjectData,
    overflow: TemporalOverflow,
) -> Result<TemporalPlainDateTimeObjectData, Cx::Error> {
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
    let date_part = temporal_plain_date_add_duration(
        cx,
        temporal_plain_date_time_date(data),
        date_duration,
        overflow,
    )?;
    let time_nanoseconds = temporal_plain_time_nanoseconds(temporal_plain_date_time_time(data))
        .checked_add(temporal_duration_time_nanoseconds(duration))
        .ok_or_else(|| range_error(cx))?;
    let day_carry = time_nanoseconds.div_euclid(TEMPORAL_NANOS_PER_DAY);
    let time = temporal_plain_time_from_nanoseconds(
        cx,
        time_nanoseconds.rem_euclid(TEMPORAL_NANOS_PER_DAY),
    )?;
    let Some(ordinal_day) = temporal_plain_date_ordinal_day(date_part).checked_add(day_carry)
    else {
        return Err(range_error(cx));
    };
    let adjusted_date = temporal_plain_date_from_ordinal_day(cx, ordinal_day)?;
    let date_time_data = TemporalPlainDateTimeObjectData::new(
        adjusted_date.year(),
        adjusted_date.month(),
        adjusted_date.day(),
        time.hour(),
        time.minute(),
        time.second(),
        time.millisecond(),
        time.microsecond(),
        time.nanosecond(),
        adjusted_date.calendar(),
    );
    let total_nanoseconds = temporal_plain_date_time_total_nanoseconds(date_time_data)
        .ok_or_else(|| range_error(cx))?;
    if !temporal_plain_date_time_is_within_limits(date_time_data.calendar(), total_nanoseconds) {
        return Err(range_error(cx));
    }
    Ok(date_time_data)
}

pub(super) fn temporal_plain_date_time_add_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let date_time = temporal_plain_date_time_data(cx, invocation.this_value())?;
    let duration = temporal_duration_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let overflow = temporal_overflow_from_options(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let data = temporal_plain_date_time_add_duration(cx, date_time, duration, overflow)?;
    let prototype = current_temporal_plain_date_time_prototype(cx)?;
    allocate_temporal_plain_date_time_object(cx, prototype, data)
}

pub(super) fn temporal_plain_date_time_subtract_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let date_time = temporal_plain_date_time_data(cx, invocation.this_value())?;
    let duration = temporal_duration_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let overflow = temporal_overflow_from_options(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let data = temporal_plain_date_time_add_duration(
        cx,
        date_time,
        negate_temporal_duration(duration),
        overflow,
    )?;
    let prototype = current_temporal_plain_date_time_prototype(cx)?;
    allocate_temporal_plain_date_time_object(cx, prototype, data)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TemporalDateTimeDifferenceUnit {
    Year,
    Month,
    Week,
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
}

pub(super) fn temporal_date_time_difference_unit_from_text(
    text: &str,
) -> Option<TemporalDateTimeDifferenceUnit> {
    match text {
        "year" | "years" => Some(TemporalDateTimeDifferenceUnit::Year),
        "month" | "months" => Some(TemporalDateTimeDifferenceUnit::Month),
        "week" | "weeks" => Some(TemporalDateTimeDifferenceUnit::Week),
        "day" | "days" => Some(TemporalDateTimeDifferenceUnit::Day),
        "hour" | "hours" => Some(TemporalDateTimeDifferenceUnit::Hour),
        "minute" | "minutes" => Some(TemporalDateTimeDifferenceUnit::Minute),
        "second" | "seconds" => Some(TemporalDateTimeDifferenceUnit::Second),
        "millisecond" | "milliseconds" => Some(TemporalDateTimeDifferenceUnit::Millisecond),
        "microsecond" | "microseconds" => Some(TemporalDateTimeDifferenceUnit::Microsecond),
        "nanosecond" | "nanoseconds" => Some(TemporalDateTimeDifferenceUnit::Nanosecond),
        _ => None,
    }
}

pub(super) fn temporal_date_time_difference_unit_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDateTimeDifferenceUnit, Cx::Error> {
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    temporal_date_time_difference_unit_from_text(&text).ok_or_else(|| range_error(cx))
}

pub(super) const fn temporal_date_time_difference_unit_order(
    unit: TemporalDateTimeDifferenceUnit,
) -> u8 {
    match unit {
        TemporalDateTimeDifferenceUnit::Year => 0,
        TemporalDateTimeDifferenceUnit::Month => 1,
        TemporalDateTimeDifferenceUnit::Week => 2,
        TemporalDateTimeDifferenceUnit::Day => 3,
        TemporalDateTimeDifferenceUnit::Hour => 4,
        TemporalDateTimeDifferenceUnit::Minute => 5,
        TemporalDateTimeDifferenceUnit::Second => 6,
        TemporalDateTimeDifferenceUnit::Millisecond => 7,
        TemporalDateTimeDifferenceUnit::Microsecond => 8,
        TemporalDateTimeDifferenceUnit::Nanosecond => 9,
    }
}

pub(super) const fn temporal_date_time_difference_unit_nanoseconds(
    unit: TemporalDateTimeDifferenceUnit,
) -> Option<i128> {
    match unit {
        TemporalDateTimeDifferenceUnit::Week => Some(TEMPORAL_NANOS_PER_DAY * 7),
        TemporalDateTimeDifferenceUnit::Day => Some(TEMPORAL_NANOS_PER_DAY),
        TemporalDateTimeDifferenceUnit::Hour => Some(TEMPORAL_NANOS_PER_HOUR),
        TemporalDateTimeDifferenceUnit::Minute => Some(TEMPORAL_NANOS_PER_MINUTE),
        TemporalDateTimeDifferenceUnit::Second => Some(TEMPORAL_NANOS_PER_SECOND),
        TemporalDateTimeDifferenceUnit::Millisecond => Some(TEMPORAL_NANOS_PER_MILLISECOND),
        TemporalDateTimeDifferenceUnit::Microsecond => Some(TEMPORAL_NANOS_PER_MICROSECOND),
        TemporalDateTimeDifferenceUnit::Nanosecond => Some(1),
        TemporalDateTimeDifferenceUnit::Year | TemporalDateTimeDifferenceUnit::Month => None,
    }
}

pub(super) const fn temporal_date_time_date_difference_unit(
    unit: TemporalDateTimeDifferenceUnit,
) -> Option<TemporalDateDifferenceUnit> {
    match unit {
        TemporalDateTimeDifferenceUnit::Year => Some(TemporalDateDifferenceUnit::Year),
        TemporalDateTimeDifferenceUnit::Month => Some(TemporalDateDifferenceUnit::Month),
        TemporalDateTimeDifferenceUnit::Week => Some(TemporalDateDifferenceUnit::Week),
        TemporalDateTimeDifferenceUnit::Day => Some(TemporalDateDifferenceUnit::Day),
        TemporalDateTimeDifferenceUnit::Hour
        | TemporalDateTimeDifferenceUnit::Minute
        | TemporalDateTimeDifferenceUnit::Second
        | TemporalDateTimeDifferenceUnit::Millisecond
        | TemporalDateTimeDifferenceUnit::Microsecond
        | TemporalDateTimeDifferenceUnit::Nanosecond => None,
    }
}

pub(super) const fn temporal_date_time_exact_unit(
    unit: TemporalDateTimeDifferenceUnit,
) -> Option<TemporalBuiltinDurationExactUnit> {
    match unit {
        TemporalDateTimeDifferenceUnit::Hour => Some(TemporalBuiltinDurationExactUnit::Hour),
        TemporalDateTimeDifferenceUnit::Minute => Some(TemporalBuiltinDurationExactUnit::Minute),
        TemporalDateTimeDifferenceUnit::Second => Some(TemporalBuiltinDurationExactUnit::Second),
        TemporalDateTimeDifferenceUnit::Millisecond => {
            Some(TemporalBuiltinDurationExactUnit::Millisecond)
        }
        TemporalDateTimeDifferenceUnit::Microsecond => {
            Some(TemporalBuiltinDurationExactUnit::Microsecond)
        }
        TemporalDateTimeDifferenceUnit::Nanosecond => {
            Some(TemporalBuiltinDurationExactUnit::Nanosecond)
        }
        TemporalDateTimeDifferenceUnit::Year
        | TemporalDateTimeDifferenceUnit::Month
        | TemporalDateTimeDifferenceUnit::Week
        | TemporalDateTimeDifferenceUnit::Day => None,
    }
}

pub(super) const fn temporal_date_time_rounding_increment_is_valid(
    smallest_unit: TemporalDateTimeDifferenceUnit,
    rounding_increment: i128,
) -> bool {
    match smallest_unit {
        TemporalDateTimeDifferenceUnit::Year
        | TemporalDateTimeDifferenceUnit::Month
        | TemporalDateTimeDifferenceUnit::Week
        | TemporalDateTimeDifferenceUnit::Day => rounding_increment > 0,
        TemporalDateTimeDifferenceUnit::Hour
        | TemporalDateTimeDifferenceUnit::Minute
        | TemporalDateTimeDifferenceUnit::Second
        | TemporalDateTimeDifferenceUnit::Millisecond
        | TemporalDateTimeDifferenceUnit::Microsecond
        | TemporalDateTimeDifferenceUnit::Nanosecond => {
            temporal_exact_time_rounding_increment_is_valid(
                temporal_date_time_exact_unit(smallest_unit).expect("exact unit"),
                rounding_increment,
            )
        }
    }
}

const fn temporal_plain_date_time_rounding_increment_is_valid(
    smallest_unit: TemporalDateTimeDifferenceUnit,
    rounding_increment: i128,
) -> bool {
    match smallest_unit {
        TemporalDateTimeDifferenceUnit::Year
        | TemporalDateTimeDifferenceUnit::Month
        | TemporalDateTimeDifferenceUnit::Week => false,
        TemporalDateTimeDifferenceUnit::Day => rounding_increment == 1,
        TemporalDateTimeDifferenceUnit::Hour
        | TemporalDateTimeDifferenceUnit::Minute
        | TemporalDateTimeDifferenceUnit::Second
        | TemporalDateTimeDifferenceUnit::Millisecond
        | TemporalDateTimeDifferenceUnit::Microsecond
        | TemporalDateTimeDifferenceUnit::Nanosecond => {
            temporal_exact_time_rounding_increment_is_valid(
                temporal_date_time_exact_unit(smallest_unit).expect("exact unit"),
                rounding_increment,
            )
        }
    }
}

pub(super) struct TemporalPlainDateTimeRoundOptions {
    pub(super) smallest_unit: TemporalDateTimeDifferenceUnit,
    pub(super) rounding_increment: i128,
    pub(super) rounding_mode: TemporalBuiltinRoundingMode,
}

pub(super) fn temporal_plain_date_time_round_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalPlainDateTimeRoundOptions, Cx::Error> {
    if value.is_string() {
        let smallest_unit = temporal_date_time_difference_unit_from_value(cx, value)?;
        if !temporal_plain_date_time_rounding_increment_is_valid(smallest_unit, 1) {
            return Err(range_error(cx));
        }
        return Ok(TemporalPlainDateTimeRoundOptions {
            smallest_unit,
            rounding_increment: 1,
            rounding_mode: TemporalBuiltinRoundingMode::HalfExpand,
        });
    }
    let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
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
    if smallest_unit_value.is_undefined() {
        return Err(range_error(cx));
    }
    let smallest_unit = temporal_date_time_difference_unit_from_value(cx, smallest_unit_value)?;
    if !temporal_plain_date_time_rounding_increment_is_valid(smallest_unit, rounding_increment) {
        return Err(range_error(cx));
    }
    Ok(TemporalPlainDateTimeRoundOptions {
        smallest_unit,
        rounding_increment,
        rounding_mode,
    })
}

pub(super) fn temporal_plain_date_time_round_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let date_time = temporal_plain_date_time_data(cx, invocation.this_value())?;
    let options = temporal_plain_date_time_round_options(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let unit_nanoseconds = temporal_date_time_difference_unit_nanoseconds(options.smallest_unit)
        .ok_or_else(|| range_error(cx))?;
    let increment = unit_nanoseconds
        .checked_mul(options.rounding_increment)
        .ok_or_else(|| range_error(cx))?;
    let rounded = temporal_round_epoch_nanoseconds_to_increment(
        temporal_plain_date_time_total_nanoseconds(date_time).ok_or_else(|| range_error(cx))?,
        increment,
        options.rounding_mode,
    )
    .ok_or_else(|| range_error(cx))?;
    if !temporal_plain_date_time_is_within_limits(date_time.calendar(), rounded) {
        return Err(range_error(cx));
    }
    let data = temporal_plain_date_time_from_total_nanoseconds(cx, rounded)?;
    let prototype = current_temporal_plain_date_time_prototype(cx)?;
    allocate_temporal_plain_date_time_object(cx, prototype, data)
}

pub(super) struct TemporalPlainDateTimeDifferenceOptions {
    pub(super) largest_unit: TemporalDateTimeDifferenceUnit,
    pub(super) smallest_unit: TemporalDateTimeDifferenceUnit,
    pub(super) rounding_increment: i128,
    pub(super) rounding_mode: TemporalBuiltinRoundingMode,
}

pub(super) fn temporal_plain_date_time_difference_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    default_largest_unit: TemporalDateTimeDifferenceUnit,
) -> Result<TemporalPlainDateTimeDifferenceOptions, Cx::Error> {
    if value.is_undefined() {
        return Ok(TemporalPlainDateTimeDifferenceOptions {
            largest_unit: default_largest_unit,
            smallest_unit: TemporalDateTimeDifferenceUnit::Nanosecond,
            rounding_increment: 1,
            rounding_mode: TemporalBuiltinRoundingMode::Trunc,
        });
    }
    let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let largest_unit_value = temporal_property_value(cx, object_ref, "largestUnit")?;
    let largest_unit_option = if largest_unit_value.is_undefined() {
        None
    } else {
        let string_ref = to_string_string_ref(cx, largest_unit_value)?;
        let text = string_ref_text(cx, string_ref)?;
        if text == "auto" {
            None
        } else {
            Some(
                temporal_date_time_difference_unit_from_text(&text)
                    .ok_or_else(|| range_error(cx))?,
            )
        }
    };
    let rounding_increment_value = temporal_property_value(cx, object_ref, "roundingIncrement")?;
    let rounding_increment =
        temporal_duration_rounding_increment_option(cx, rounding_increment_value)?;
    let rounding_mode_value = temporal_property_value(cx, object_ref, "roundingMode")?;
    let rounding_mode = temporal_duration_rounding_mode_option(cx, rounding_mode_value)?;
    let smallest_unit_value = temporal_property_value(cx, object_ref, "smallestUnit")?;
    let smallest_unit = if smallest_unit_value.is_undefined() {
        TemporalDateTimeDifferenceUnit::Nanosecond
    } else {
        temporal_date_time_difference_unit_from_value(cx, smallest_unit_value)?
    };
    let default_largest_unit = if temporal_date_time_difference_unit_order(default_largest_unit)
        > temporal_date_time_difference_unit_order(smallest_unit)
    {
        smallest_unit
    } else {
        default_largest_unit
    };
    let largest_unit = largest_unit_option.unwrap_or(default_largest_unit);
    if temporal_date_time_difference_unit_order(largest_unit)
        > temporal_date_time_difference_unit_order(smallest_unit)
        || !temporal_date_time_rounding_increment_is_valid(smallest_unit, rounding_increment)
    {
        return Err(range_error(cx));
    }
    Ok(TemporalPlainDateTimeDifferenceOptions {
        largest_unit,
        smallest_unit,
        rounding_increment,
        rounding_mode,
    })
}

pub(super) fn temporal_duration_from_date_time_nanoseconds<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    nanoseconds: i128,
    largest_unit: TemporalDateTimeDifferenceUnit,
) -> Result<TemporalDurationObjectData, Cx::Error> {
    if let Some(exact_largest_unit) = temporal_date_time_exact_unit(largest_unit) {
        return temporal_duration_from_nanoseconds_with_largest_unit(
            cx,
            nanoseconds,
            exact_largest_unit,
        );
    }

    let mut remainder = nanoseconds;
    let weeks = if largest_unit == TemporalDateTimeDifferenceUnit::Week {
        let value = remainder / (TEMPORAL_NANOS_PER_DAY * 7);
        remainder %= TEMPORAL_NANOS_PER_DAY * 7;
        i64::try_from(value).map_err(|_| range_error(cx))?
    } else {
        0
    };
    let days = if matches!(
        largest_unit,
        TemporalDateTimeDifferenceUnit::Week | TemporalDateTimeDifferenceUnit::Day
    ) {
        let value = remainder / TEMPORAL_NANOS_PER_DAY;
        remainder %= TEMPORAL_NANOS_PER_DAY;
        i64::try_from(value).map_err(|_| range_error(cx))?
    } else {
        0
    };
    let time = temporal_duration_from_nanoseconds_with_largest_unit(
        cx,
        remainder,
        TemporalBuiltinDurationExactUnit::Hour,
    )?;
    Ok(TemporalDurationObjectData::new(
        0,
        0,
        weeks,
        days,
        time.hours(),
        time.minutes(),
        time.seconds(),
        time.milliseconds(),
        time.microseconds(),
        time.nanoseconds(),
    ))
}

pub(super) fn temporal_plain_date_time_calendar_difference_duration<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    start: TemporalPlainDateTimeObjectData,
    end: TemporalPlainDateTimeObjectData,
    largest_unit: TemporalDateTimeDifferenceUnit,
) -> Result<TemporalDurationObjectData, Cx::Error> {
    let total_nanoseconds = temporal_plain_date_time_total_nanoseconds(end)
        .and_then(|end| {
            temporal_plain_date_time_total_nanoseconds(start)
                .and_then(|start| end.checked_sub(start))
        })
        .ok_or_else(|| range_error(cx))?;
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
        temporal_date_time_date_difference_unit(largest_unit)
            .expect("calendar difference requires a date largest unit"),
        TemporalDateDifferenceUnit::Day,
    )?;
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

#[allow(
    clippy::too_many_lines,
    reason = "PlainDateTime difference keeps the calendar/time balancing algorithm in order"
)]
pub(super) fn temporal_plain_date_time_difference_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    sign: i128,
) -> Result<Value, Cx::Error> {
    let date_time = temporal_plain_date_time_data(cx, invocation.this_value())?;
    let other = temporal_plain_date_time_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let options = temporal_plain_date_time_difference_options(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
        TemporalDateTimeDifferenceUnit::Day,
    )?;
    if let Some(largest_date_unit) = temporal_date_time_date_difference_unit(options.largest_unit) {
        let (start, end) = if sign > 0 {
            (other, date_time)
        } else {
            (date_time, other)
        };
        let unrounded = temporal_plain_date_time_calendar_difference_duration(
            cx,
            start,
            end,
            options.largest_unit,
        )?;
        let duration = if options.rounding_increment == 1
            && options.rounding_mode == TemporalBuiltinRoundingMode::Trunc
            && options.smallest_unit == TemporalDateTimeDifferenceUnit::Nanosecond
        {
            unrounded
        } else {
            let round_input = if sign > 0 {
                negate_temporal_duration(unrounded)
            } else {
                unrounded
            };
            let rounding_mode = if sign > 0 {
                temporal_rounding_mode_for_negated_duration(options.rounding_mode)
            } else {
                options.rounding_mode
            };
            let rounded =
                if let Some(exact_unit) = temporal_date_time_exact_unit(options.smallest_unit) {
                    temporal_duration_round_calendar_relative_exact(
                        cx,
                        round_input,
                        TemporalDurationRelativeTo::PlainDateTime(date_time),
                        largest_date_unit,
                        exact_unit,
                        options.rounding_increment,
                        rounding_mode,
                    )?
                } else {
                    let smallest_date_unit =
                        temporal_date_time_date_difference_unit(options.smallest_unit)
                            .expect("validated date-time difference date unit");
                    temporal_duration_round_calendar_relative(
                        cx,
                        round_input,
                        TemporalDurationRelativeTo::PlainDateTime(date_time),
                        largest_date_unit,
                        smallest_date_unit,
                        options.rounding_increment,
                        rounding_mode,
                    )?
                };
            if sign > 0 {
                negate_temporal_duration(rounded)
            } else {
                rounded
            }
        };
        validate_temporal_duration(cx, duration)?;
        let prototype = current_temporal_duration_prototype(cx)?;
        return allocate_temporal_duration_object(cx, prototype, duration);
    }
    let left =
        temporal_plain_date_time_total_nanoseconds(date_time).ok_or_else(|| range_error(cx))?;
    let right = temporal_plain_date_time_total_nanoseconds(other).ok_or_else(|| range_error(cx))?;
    let raw_difference = left
        .checked_sub(right)
        .and_then(|difference| difference.checked_mul(sign))
        .ok_or_else(|| range_error(cx))?;
    let unit_nanoseconds = temporal_date_time_difference_unit_nanoseconds(options.smallest_unit)
        .ok_or_else(|| range_error(cx))?;
    let increment = unit_nanoseconds
        .checked_mul(options.rounding_increment)
        .ok_or_else(|| range_error(cx))?;
    let rounded = temporal_round_duration_nanoseconds_to_increment(
        raw_difference,
        increment,
        options.rounding_mode,
    )
    .ok_or_else(|| range_error(cx))?;
    let duration = temporal_duration_from_date_time_nanoseconds(cx, rounded, options.largest_unit)?;
    validate_temporal_duration(cx, duration)?;
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(cx, prototype, duration)
}

pub(super) fn temporal_plain_date_time_since_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_plain_date_time_difference_builtin(cx, invocation, 1)
}

pub(super) fn temporal_plain_date_time_until_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_plain_date_time_difference_builtin(cx, invocation, -1)
}

pub(super) fn temporal_plain_date_time_to_plain_date_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    let plain_date_data =
        TemporalPlainDateObjectData::new(data.year(), data.month(), data.day(), data.calendar());
    let prototype = current_temporal_plain_date_prototype(cx)?;
    allocate_temporal_plain_date_object(cx, prototype, plain_date_data)
}

pub(super) fn temporal_plain_date_time_to_plain_time_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    let time = TemporalPlainTimeObjectData::new(
        data.hour(),
        data.minute(),
        data.second(),
        data.millisecond(),
        data.microsecond(),
        data.nanosecond(),
    );
    let prototype = current_temporal_plain_time_prototype(cx)?;
    plain_time::allocate_temporal_plain_time_object(cx, prototype, time)
}

pub(super) fn temporal_plain_date_time_to_zoned_date_time_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let date_time = temporal_plain_date_time_data(cx, invocation.this_value())?;
    let time_zone_id = temporal_time_zone_id_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let disambiguation = temporal_disambiguation_from_options(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let instant = cx.temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
        time_zone_id: time_zone_id.clone(),
        date_time: temporal_civil_date_time_from_plain_date_time(date_time),
        disambiguation,
    })?;
    let data = temporal_zoned_date_time_from_parts(cx, instant.epoch_nanoseconds, &time_zone_id)?;
    let prototype = current_temporal_zoned_date_time_prototype(cx)?;
    allocate_temporal_zoned_date_time_object(cx, prototype, data)
}

pub(super) fn temporal_disambiguation_from_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    options: Value,
) -> Result<TemporalDisambiguation, Cx::Error> {
    temporal_validate_options_object(cx, options)?;
    if options.is_undefined() {
        return Ok(TemporalDisambiguation::Compatible);
    }
    let object_ref = options.as_object_ref().ok_or_else(|| type_error(cx))?;
    let value = temporal_property_value(cx, object_ref, "disambiguation")?;
    let disambiguation = temporal_string_option(
        cx,
        value,
        &["compatible", "earlier", "later", "reject"],
        "compatible",
    )?;
    match disambiguation.as_str() {
        "compatible" => Ok(TemporalDisambiguation::Compatible),
        "earlier" => Ok(TemporalDisambiguation::Earlier),
        "later" => Ok(TemporalDisambiguation::Later),
        "reject" => Ok(TemporalDisambiguation::Reject),
        _ => unreachable!("temporal_string_option constrained disambiguation"),
    }
}

pub(super) fn temporal_plain_date_time_from_builtin<Cx: PublicBuiltinDispatchContext>(
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
        let data = temporal_plain_date_time_from_value(cx, value)?;
        let _overflow = temporal_overflow_from_options(cx, options)?;
        data
    } else {
        let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
        let existing = {
            let agent = cx.agent();
            agent.objects().temporal_object(object_ref).copied()
        };
        match existing {
            Some(TemporalObjectData::PlainDateTime(data)) => {
                let _overflow = temporal_overflow_from_options(cx, options)?;
                data
            }
            Some(TemporalObjectData::PlainDate(data)) => {
                let _overflow = temporal_overflow_from_options(cx, options)?;
                TemporalPlainDateTimeObjectData::new(
                    data.year(),
                    data.month(),
                    data.day(),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    data.calendar(),
                )
            }
            Some(TemporalObjectData::ZonedDateTime(data)) => {
                let _overflow = temporal_overflow_from_options(cx, options)?;
                let civil = temporal_zoned_date_time_civil(cx, data)?;
                temporal_plain_date_time_from_parts(
                    cx,
                    i64::from(civil.date_time.year),
                    i64::from(civil.date_time.month),
                    i64::from(civil.date_time.day),
                    i64::from(civil.date_time.hour),
                    i64::from(civil.date_time.minute),
                    i64::from(civil.date_time.second),
                    i64::from(civil.date_time.millisecond),
                    i64::from(civil.date_time.microsecond),
                    i64::from(civil.date_time.nanosecond),
                )?
            }
            _ => {
                let fields = temporal_plain_date_time_bag_fields(cx, object_ref)?;
                if fields.year.is_none() || fields.day.is_none() {
                    return Err(type_error(cx));
                }
                if fields.month.is_none() && fields.month_code_text.is_none() {
                    return Err(type_error(cx));
                }
                let overflow = temporal_overflow_from_options(cx, options)?;
                temporal_plain_date_time_from_bag_fields(cx, &fields, overflow)?
            }
        }
    };
    let prototype = current_temporal_plain_date_time_prototype(cx)?;
    allocate_temporal_plain_date_time_object(cx, prototype, data)
}

pub(super) fn temporal_plain_date_time_compare_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = temporal_plain_date_time_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let right = temporal_plain_date_time_from_value(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(temporal_compare_ordering(
        (
            left.year(),
            left.month(),
            left.day(),
            left.hour(),
            left.minute(),
            left.second(),
            left.millisecond(),
            left.microsecond(),
            left.nanosecond(),
        )
            .cmp(&(
                right.year(),
                right.month(),
                right.day(),
                right.hour(),
                right.minute(),
                right.second(),
                right.millisecond(),
                right.microsecond(),
                right.nanosecond(),
            )),
    ))
}

use super::{
    allocate_temporal_duration_object, allocate_temporal_plain_date_time_object,
    allocate_temporal_zoned_date_time_object, current_temporal_duration_prototype,
    current_temporal_plain_date_prototype, current_temporal_plain_date_time_prototype,
    current_temporal_plain_month_day_prototype, current_temporal_plain_year_month_prototype,
    current_temporal_zoned_date_time_prototype, format_temporal_plain_date, map_completion,
    negate_temporal_duration, object, plain_month_day, plain_year_month, range_error,
    string_ref_text, string_value, temporal_compare_ordering, temporal_constructor_prototype,
    temporal_duration_from_value, temporal_duration_round_calendar_relative,
    temporal_duration_rounding_increment_option, temporal_duration_rounding_mode_option,
    temporal_duration_whole_days_from_time, temporal_integer_part_from_argument,
    temporal_is_iso_leap_year, temporal_iso_day_of_week, temporal_iso_day_of_year,
    temporal_iso_days_before_year, temporal_iso_days_in_month, temporal_iso_days_in_year,
    temporal_iso_week_of_year, temporal_month_from_month_code_text, temporal_ops,
    temporal_optional_integer_part_from_property, temporal_optional_month_code_text_from_property,
    temporal_overflow_from_options, temporal_plain_date_ordinal_day,
    temporal_plain_date_time_total_nanoseconds, temporal_plain_time_from_value,
    temporal_property_value, temporal_resolve_month_from_fields,
    temporal_round_duration_nanoseconds_to_increment, temporal_rounding_mode_for_negated_duration,
    temporal_string_option, temporal_time_part_from_property, temporal_time_zone_id_from_value,
    temporal_validate_iso_calendar_value,
    temporal_validate_optional_iso_calendar_identifier_argument,
    temporal_validate_optional_iso_calendar_property, temporal_zoned_date_time_civil,
    temporal_zoned_date_time_from_parts, to_string_string_ref, type_error,
    validate_temporal_duration, AllocationLifetime, AtomId, BuiltinFunctionId, BuiltinInvocation,
    ObjectAllocation, ObjectColdData, ObjectRef, OrdinaryObjectData, PublicBuiltinDispatchContext,
    TemporalBuiltinRoundingMode, TemporalCivilDateTime, TemporalCivilToInstantRequest,
    TemporalDisambiguation, TemporalDurationObjectData, TemporalDurationRelativeTo,
    TemporalObjectData, TemporalObjectKind, TemporalOverflow, TemporalPlainDateObjectData,
    TemporalPlainDateTimeObjectData, TemporalPlainMonthDayObjectData, TemporalPlainTimeObjectData,
    TemporalPlainYearMonthObjectData, TemporalZonedDateTimeCalendarNameOption, Value,
    TEMPORAL_NANOS_PER_DAY,
};

#[allow(
    clippy::too_many_lines,
    reason = "Temporal PlainDate dispatch is the builtin ID switchboard for this spec domain"
)]
pub(super) fn dispatch_temporal_plain_date_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::temporal_plain_date_builtin() {
        return temporal_plain_date_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_year_getter_builtin() {
        return temporal_plain_date_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_month_getter_builtin() {
        return temporal_plain_date_month_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_month_code_getter_builtin() {
        return temporal_plain_date_month_code_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_day_getter_builtin() {
        return temporal_plain_date_day_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_day_of_week_getter_builtin() {
        return temporal_plain_date_day_of_week_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_day_of_year_getter_builtin() {
        return temporal_plain_date_day_of_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_days_in_month_getter_builtin() {
        return temporal_plain_date_days_in_month_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_days_in_year_getter_builtin() {
        return temporal_plain_date_days_in_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_months_in_year_getter_builtin() {
        return temporal_plain_date_months_in_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_in_leap_year_getter_builtin() {
        return temporal_plain_date_in_leap_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_days_in_week_getter_builtin() {
        return temporal_plain_date_days_in_week_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_week_of_year_getter_builtin() {
        return temporal_plain_date_week_of_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_year_of_week_getter_builtin() {
        return temporal_plain_date_year_of_week_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_era_getter_builtin() {
        return temporal_plain_date_era_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_era_year_getter_builtin() {
        return temporal_plain_date_era_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_calendar_id_getter_builtin() {
        return temporal_plain_date_calendar_id_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_to_string_builtin() {
        return temporal_plain_date_to_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_to_json_builtin() {
        return temporal_plain_date_to_json_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_to_locale_string_builtin() {
        return temporal_plain_date_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_value_of_builtin() {
        return temporal_plain_date_value_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_equals_builtin() {
        return temporal_plain_date_equals_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_with_builtin() {
        return temporal_plain_date_with_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_with_calendar_builtin() {
        return temporal_plain_date_with_calendar_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_add_builtin() {
        return temporal_plain_date_add_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_subtract_builtin() {
        return temporal_plain_date_subtract_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_since_builtin() {
        return temporal_plain_date_since_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_until_builtin() {
        return temporal_plain_date_until_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_to_plain_date_time_builtin() {
        return temporal_plain_date_to_plain_date_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_to_zoned_date_time_builtin() {
        return temporal_plain_date_to_zoned_date_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_to_plain_year_month_builtin() {
        return temporal_plain_date_to_plain_year_month_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_to_plain_month_day_builtin() {
        return temporal_plain_date_to_plain_month_day_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_from_builtin() {
        return temporal_plain_date_from_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_date_compare_builtin() {
        return temporal_plain_date_compare_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

pub(super) fn temporal_plain_date_from_ordinal_day<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    ordinal_day: i128,
) -> Result<TemporalPlainDateObjectData, Cx::Error> {
    let min_ordinal = i128::from(temporal_iso_days_before_year(-271_821));
    let max_ordinal = i128::from(temporal_iso_days_before_year(275_760))
        + i128::from(temporal_iso_days_in_year(275_760))
        - 1;
    if !(min_ordinal..=max_ordinal).contains(&ordinal_day) {
        return Err(range_error(cx));
    }

    let mut low = -271_821;
    let mut high = 275_761;
    while low + 1 < high {
        let mid = low + (high - low) / 2;
        if i128::from(temporal_iso_days_before_year(mid)) <= ordinal_day {
            low = mid;
        } else {
            high = mid;
        }
    }

    let year = low;
    let mut remaining =
        i32::try_from(ordinal_day - i128::from(temporal_iso_days_before_year(year)))
            .map_err(|_| range_error(cx))?;
    let mut month = 1_u8;
    loop {
        let days_in_month = i32::from(temporal_iso_days_in_month(year, month));
        if remaining < days_in_month {
            let day = remaining + 1;
            return temporal_plain_date_from_parts(cx, i64::from(year), month.into(), day.into());
        }
        remaining -= days_in_month;
        month = month.checked_add(1).ok_or_else(|| range_error(cx))?;
    }
}

pub(super) fn temporal_plain_date_from_parts<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    year: i64,
    month: i64,
    day: i64,
) -> Result<TemporalPlainDateObjectData, Cx::Error> {
    temporal_plain_date_from_parts_with_overflow(cx, year, month, day, TemporalOverflow::Reject)
}

pub(super) fn temporal_plain_date_from_parts_with_overflow<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    year: i64,
    month: i64,
    day: i64,
    overflow: TemporalOverflow,
) -> Result<TemporalPlainDateObjectData, Cx::Error> {
    if !(-271_821..=275_760).contains(&year) {
        return Err(range_error(cx));
    }
    let year = i32::try_from(year).map_err(|_| range_error(cx))?;
    if month < 1 || day < 1 {
        return Err(range_error(cx));
    }
    let month = match overflow {
        TemporalOverflow::Constrain => month.min(12),
        TemporalOverflow::Reject => {
            if month > 12 {
                return Err(range_error(cx));
            }
            month
        }
    };
    let month = u8::try_from(month).map_err(|_| range_error(cx))?;
    let max_day = i64::from(temporal_iso_days_in_month(year, month));
    let day = match overflow {
        TemporalOverflow::Constrain => day.min(max_day),
        TemporalOverflow::Reject => {
            if day > max_day {
                return Err(range_error(cx));
            }
            day
        }
    };
    let day = u8::try_from(day).map_err(|_| range_error(cx))?;
    if (year, month, day) < (-271_821, 4, 19) || (year, month, day) > (275_760, 9, 13) {
        return Err(range_error(cx));
    }
    let calendar = {
        let agent = cx.agent();
        agent.atoms_mut().intern_collectible("iso8601")
    };
    Ok(TemporalPlainDateObjectData::new(year, month, day, calendar))
}

pub(super) fn allocate_temporal_plain_date_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: ObjectRef,
    data: TemporalPlainDateObjectData,
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
                        TemporalObjectKind::PlainDate,
                    ))),
                AllocationLifetime::Default,
            )
        })
    };
    let installed = cx
        .agent()
        .objects_mut()
        .install_temporal_object(object, TemporalObjectData::PlainDate(data));
    if !installed {
        return Err(type_error(cx));
    }
    Ok(Value::from_object_ref(object))
}

pub(super) fn temporal_plain_date_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalPlainDateObjectData, Cx::Error> {
    if value.is_string() {
        let string_ref = to_string_string_ref(cx, value)?;
        let text = string_ref_text(cx, string_ref)?;
        let (year, month, day) =
            temporal_ops::parse_plain_date(&text).ok_or_else(|| range_error(cx))?;
        return temporal_plain_date_from_parts(cx, year.into(), month.into(), day.into());
    }
    let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let existing = {
        let agent = cx.agent();
        agent.objects().temporal_object(object_ref).copied()
    };
    match existing {
        Some(TemporalObjectData::PlainDate(data)) => return Ok(data),
        Some(TemporalObjectData::PlainDateTime(data)) => {
            return temporal_plain_date_from_parts(
                cx,
                i64::from(data.year()),
                i64::from(data.month()),
                i64::from(data.day()),
            );
        }
        Some(TemporalObjectData::ZonedDateTime(data)) => {
            let civil = temporal_zoned_date_time_civil(cx, data)?;
            return temporal_plain_date_from_parts(
                cx,
                i64::from(civil.date_time.year),
                i64::from(civil.date_time.month),
                i64::from(civil.date_time.day),
            );
        }
        _ => {}
    }

    temporal_plain_date_from_property_bag(cx, object_ref, true)
}

pub(super) fn temporal_plain_date_from_property_bag<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    validate_calendar: bool,
) -> Result<TemporalPlainDateObjectData, Cx::Error> {
    if validate_calendar {
        temporal_validate_optional_iso_calendar_property(cx, object_ref)?;
    }
    let fields = TemporalPlainDateBagFields {
        day: temporal_optional_integer_part_from_property(cx, object_ref, "day")?,
        month: temporal_optional_integer_part_from_property(cx, object_ref, "month")?,
        month_code_text: temporal_optional_month_code_text_from_property(cx, object_ref)?,
        year: temporal_optional_integer_part_from_property(cx, object_ref, "year")?,
    };
    temporal_plain_date_from_bag_fields(cx, &fields, TemporalOverflow::Reject)
}

pub(super) struct TemporalPlainDateBagFields {
    pub(super) year: Option<i64>,
    pub(super) month: Option<i64>,
    pub(super) month_code_text: Option<String>,
    pub(super) day: Option<i64>,
}

pub(super) struct TemporalPlainDateTimeBagFields {
    pub(super) year: Option<i64>,
    pub(super) month: Option<i64>,
    pub(super) month_code_text: Option<String>,
    pub(super) day: Option<i64>,
    pub(super) hour: i64,
    pub(super) minute: i64,
    pub(super) second: i64,
    pub(super) millisecond: i64,
    pub(super) microsecond: i64,
    pub(super) nanosecond: i64,
}

pub(super) fn temporal_plain_date_bag_fields<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
) -> Result<TemporalPlainDateBagFields, Cx::Error> {
    temporal_validate_optional_iso_calendar_property(cx, object_ref)?;
    Ok(TemporalPlainDateBagFields {
        day: temporal_optional_integer_part_from_property(cx, object_ref, "day")?,
        month: temporal_optional_integer_part_from_property(cx, object_ref, "month")?,
        month_code_text: temporal_optional_month_code_text_from_property(cx, object_ref)?,
        year: temporal_optional_integer_part_from_property(cx, object_ref, "year")?,
    })
}

pub(super) fn temporal_plain_date_time_bag_fields<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
) -> Result<TemporalPlainDateTimeBagFields, Cx::Error> {
    temporal_validate_optional_iso_calendar_property(cx, object_ref)?;
    Ok(TemporalPlainDateTimeBagFields {
        day: temporal_optional_integer_part_from_property(cx, object_ref, "day")?,
        hour: temporal_time_part_from_property(cx, object_ref, "hour")?,
        microsecond: temporal_time_part_from_property(cx, object_ref, "microsecond")?,
        millisecond: temporal_time_part_from_property(cx, object_ref, "millisecond")?,
        minute: temporal_time_part_from_property(cx, object_ref, "minute")?,
        month: temporal_optional_integer_part_from_property(cx, object_ref, "month")?,
        month_code_text: temporal_optional_month_code_text_from_property(cx, object_ref)?,
        nanosecond: temporal_time_part_from_property(cx, object_ref, "nanosecond")?,
        second: temporal_time_part_from_property(cx, object_ref, "second")?,
        year: temporal_optional_integer_part_from_property(cx, object_ref, "year")?,
    })
}

pub(super) fn temporal_plain_date_from_bag_fields<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    fields: &TemporalPlainDateBagFields,
    overflow: TemporalOverflow,
) -> Result<TemporalPlainDateObjectData, Cx::Error> {
    let year = fields.year.ok_or_else(|| type_error(cx))?;
    let day = fields.day.ok_or_else(|| type_error(cx))?;
    let month = temporal_resolve_month_from_fields(
        cx,
        fields.month,
        fields.month_code_text.as_deref(),
        None,
    )?;
    temporal_plain_date_from_parts_with_overflow(cx, year, month, day, overflow)
}

pub(super) fn temporal_plain_date_data<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    this_value: Value,
) -> Result<TemporalPlainDateObjectData, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, this_value, TemporalObjectKind::PlainDate)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::PlainDate(data) = payload else {
        return Err(type_error(cx));
    };
    Ok(data)
}

pub(super) fn temporal_plain_date_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let Some(new_target) = invocation.new_target() else {
        return Err(type_error(cx));
    };
    let year = temporal_integer_part_from_argument(cx, invocation, 0)?;
    let month = temporal_integer_part_from_argument(cx, invocation, 1)?;
    let day = temporal_integer_part_from_argument(cx, invocation, 2)?;
    temporal_validate_optional_iso_calendar_identifier_argument(cx, invocation, 3)?;
    let data = temporal_plain_date_from_parts(cx, year, month, day)?;
    let prototype = temporal_constructor_prototype(cx, new_target)?;
    allocate_temporal_plain_date_object(cx, prototype, data)
}

pub(super) fn temporal_plain_date_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(data.year()))
}

pub(super) fn temporal_plain_date_month_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.month())))
}

pub(super) fn temporal_plain_date_month_code_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format!("M{:02}", data.month())))
}

pub(super) fn temporal_plain_date_day_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.day())))
}

pub(super) fn temporal_plain_date_day_of_week_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(temporal_iso_day_of_week(
        data.year(),
        data.month(),
        data.day(),
    )))
}

pub(super) fn temporal_plain_date_day_of_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(temporal_iso_day_of_year(
        data.year(),
        data.month(),
        data.day(),
    )))
}

pub(super) fn temporal_plain_date_days_in_month_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(temporal_iso_days_in_month(
        data.year(),
        data.month(),
    ))))
}

pub(super) fn temporal_plain_date_days_in_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(temporal_iso_days_in_year(data.year())))
}

pub(super) fn temporal_plain_date_months_in_year_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(12))
}

pub(super) fn temporal_plain_date_in_leap_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::from_bool(temporal_is_iso_leap_year(data.year())))
}

pub(super) fn temporal_plain_date_days_in_week_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(7))
}

pub(super) fn temporal_plain_date_week_of_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    let (week, _) = temporal_iso_week_of_year(data.year(), data.month(), data.day());
    Ok(Value::from_smi(week))
}

pub(super) fn temporal_plain_date_year_of_week_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    let (_, year) = temporal_iso_week_of_year(data.year(), data.month(), data.day());
    Ok(Value::from_smi(year))
}

pub(super) fn temporal_plain_date_era_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::undefined())
}

pub(super) fn temporal_plain_date_era_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::undefined())
}

pub(super) fn temporal_plain_date_calendar_id_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(string_value(cx, "iso8601"))
}

pub(super) fn temporal_plain_date_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    let calendar_name = temporal_plain_date_to_string_calendar_name(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let mut text = format_temporal_plain_date(data);
    match calendar_name {
        TemporalZonedDateTimeCalendarNameOption::Always => {
            text.push_str("[u-ca=iso8601]");
        }
        TemporalZonedDateTimeCalendarNameOption::Critical => {
            text.push_str("[!u-ca=iso8601]");
        }
        TemporalZonedDateTimeCalendarNameOption::Auto
        | TemporalZonedDateTimeCalendarNameOption::Never => {}
    }
    Ok(string_value(cx, &text))
}

pub(super) fn temporal_plain_date_to_json_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format_temporal_plain_date(data)))
}

pub(super) fn temporal_plain_date_to_locale_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format_temporal_plain_date(data)))
}

pub(super) fn temporal_plain_date_to_string_calendar_name<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalZonedDateTimeCalendarNameOption, Cx::Error> {
    if value.is_undefined() {
        return Ok(TemporalZonedDateTimeCalendarNameOption::Auto);
    }
    let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let calendar_name_value = temporal_property_value(cx, object_ref, "calendarName")?;
    match temporal_string_option(
        cx,
        calendar_name_value,
        &["auto", "always", "never", "critical"],
        "auto",
    )?
    .as_str()
    {
        "auto" => Ok(TemporalZonedDateTimeCalendarNameOption::Auto),
        "always" => Ok(TemporalZonedDateTimeCalendarNameOption::Always),
        "never" => Ok(TemporalZonedDateTimeCalendarNameOption::Never),
        "critical" => Ok(TemporalZonedDateTimeCalendarNameOption::Critical),
        _ => unreachable!("temporal_string_option constrained calendarName"),
    }
}

pub(super) fn temporal_plain_date_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Err(type_error(cx))
}

pub(super) fn temporal_plain_date_equals_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = temporal_plain_date_data(cx, invocation.this_value())?;
    let right = temporal_plain_date_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(left == right))
}

pub(super) fn temporal_plain_date_with_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let date = temporal_plain_date_data(cx, invocation.this_value())?;
    let fields = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined())
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    {
        let temporal = {
            let agent = cx.agent();
            agent.objects().temporal_object(fields).copied()
        };
        if temporal.is_some() {
            return Err(type_error(cx));
        }
    }
    temporal_reject_calendar_or_time_zone_properties(cx, fields)?;
    let day = temporal_optional_integer_part_from_property(cx, fields, "day")?;
    let month_value = temporal_optional_integer_part_from_property(cx, fields, "month")?;
    let month_code_value = temporal_property_value(cx, fields, "monthCode")?;
    let month_code_text = if month_code_value.is_undefined() {
        None
    } else {
        let string_ref = to_string_string_ref(cx, month_code_value)?;
        Some(string_ref_text(cx, string_ref)?)
    };
    let year = temporal_optional_integer_part_from_property(cx, fields, "year")?;
    if year.is_none() && day.is_none() && month_value.is_none() && month_code_value.is_undefined() {
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
        i64::from(date.month())
    };
    let result = temporal_plain_date_from_parts_with_overflow(
        cx,
        year.unwrap_or_else(|| i64::from(date.year())),
        month,
        day.unwrap_or_else(|| i64::from(date.day())),
        overflow,
    )?;
    if !options.is_undefined() && options.as_object_ref().is_none() {
        return Err(type_error(cx));
    }
    let prototype = current_temporal_plain_date_prototype(cx)?;
    allocate_temporal_plain_date_object(cx, prototype, result)
}

pub(super) fn temporal_plain_date_with_calendar_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let date = temporal_plain_date_data(cx, invocation.this_value())?;
    temporal_validate_iso_calendar_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let result =
        TemporalPlainDateObjectData::new(date.year(), date.month(), date.day(), date.calendar());
    let prototype = current_temporal_plain_date_prototype(cx)?;
    allocate_temporal_plain_date_object(cx, prototype, result)
}

pub(super) fn temporal_plain_date_add_duration<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    date: TemporalPlainDateObjectData,
    duration: TemporalDurationObjectData,
    overflow: TemporalOverflow,
) -> Result<TemporalPlainDateObjectData, Cx::Error> {
    let total_months = i128::from(date.year()) * 12
        + i128::from(date.month() - 1)
        + duration.years() * 12
        + duration.months();
    let year = total_months.div_euclid(12);
    let month = total_months.rem_euclid(12) + 1;
    if !(-271_821..=275_760).contains(&year) {
        return Err(range_error(cx));
    }

    let year = i32::try_from(year).map_err(|_| range_error(cx))?;
    let month = u8::try_from(month).map_err(|_| range_error(cx))?;
    let max_day = temporal_iso_days_in_month(year, month);
    if matches!(overflow, TemporalOverflow::Reject) && date.day() > max_day {
        return Err(range_error(cx));
    }
    let day = date.day().min(max_day);
    let constrained = temporal_plain_date_from_parts_with_overflow(
        cx,
        i64::from(year),
        i64::from(month),
        i64::from(day),
        overflow,
    )?;

    let day_delta =
        duration.weeks() * 7 + duration.days() + temporal_duration_whole_days_from_time(duration);
    let ordinal_day = temporal_plain_date_ordinal_day(constrained)
        .checked_add(day_delta)
        .ok_or_else(|| range_error(cx))?;
    temporal_plain_date_from_ordinal_day(cx, ordinal_day)
}

pub(super) fn temporal_plain_date_add_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let date = temporal_plain_date_data(cx, invocation.this_value())?;
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
    let result = temporal_plain_date_add_duration(cx, date, duration, overflow)?;
    let prototype = current_temporal_plain_date_prototype(cx)?;
    allocate_temporal_plain_date_object(cx, prototype, result)
}

pub(super) fn temporal_plain_date_subtract_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let date = temporal_plain_date_data(cx, invocation.this_value())?;
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
    let result =
        temporal_plain_date_add_duration(cx, date, negate_temporal_duration(duration), overflow)?;
    let prototype = current_temporal_plain_date_prototype(cx)?;
    allocate_temporal_plain_date_object(cx, prototype, result)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TemporalDateDifferenceUnit {
    Year,
    Month,
    Week,
    Day,
}

pub(super) fn temporal_date_difference_unit_from_text(
    text: &str,
) -> Option<TemporalDateDifferenceUnit> {
    match text {
        "year" | "years" => Some(TemporalDateDifferenceUnit::Year),
        "month" | "months" => Some(TemporalDateDifferenceUnit::Month),
        "week" | "weeks" => Some(TemporalDateDifferenceUnit::Week),
        "day" | "days" => Some(TemporalDateDifferenceUnit::Day),
        _ => None,
    }
}

pub(super) fn temporal_date_difference_unit_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDateDifferenceUnit, Cx::Error> {
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    temporal_date_difference_unit_from_text(&text).ok_or_else(|| range_error(cx))
}

pub(super) const fn temporal_date_difference_unit_order(unit: TemporalDateDifferenceUnit) -> u8 {
    match unit {
        TemporalDateDifferenceUnit::Year => 0,
        TemporalDateDifferenceUnit::Month => 1,
        TemporalDateDifferenceUnit::Week => 2,
        TemporalDateDifferenceUnit::Day => 3,
    }
}

pub(super) struct TemporalDateDifferenceOptions {
    pub(super) largest_unit: TemporalDateDifferenceUnit,
    pub(super) smallest_unit: TemporalDateDifferenceUnit,
    pub(super) rounding_increment: i128,
    pub(super) rounding_mode: TemporalBuiltinRoundingMode,
}

pub(super) fn temporal_date_difference_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    default_largest_unit: TemporalDateDifferenceUnit,
    default_smallest_unit: TemporalDateDifferenceUnit,
) -> Result<TemporalDateDifferenceOptions, Cx::Error> {
    if value.is_undefined() {
        return Ok(TemporalDateDifferenceOptions {
            largest_unit: default_largest_unit,
            smallest_unit: default_smallest_unit,
            rounding_increment: 1,
            rounding_mode: TemporalBuiltinRoundingMode::Trunc,
        });
    }
    let Some(object_ref) = value.as_object_ref() else {
        return Err(type_error(cx));
    };
    let largest_unit_value = temporal_property_value(cx, object_ref, "largestUnit")?;
    let largest_unit_text = if largest_unit_value.is_undefined() {
        None
    } else {
        let string_ref = to_string_string_ref(cx, largest_unit_value)?;
        Some(string_ref_text(cx, string_ref)?)
    };
    let rounding_increment_value = temporal_property_value(cx, object_ref, "roundingIncrement")?;
    let rounding_increment =
        temporal_duration_rounding_increment_option(cx, rounding_increment_value)?;
    let rounding_mode_value = temporal_property_value(cx, object_ref, "roundingMode")?;
    let rounding_mode = temporal_duration_rounding_mode_option(cx, rounding_mode_value)?;
    let smallest_unit_value = temporal_property_value(cx, object_ref, "smallestUnit")?;
    let smallest_unit = if smallest_unit_value.is_undefined() {
        default_smallest_unit
    } else {
        temporal_date_difference_unit_from_value(cx, smallest_unit_value)?
    };
    let default_largest_unit = if temporal_date_difference_unit_order(default_largest_unit)
        > temporal_date_difference_unit_order(smallest_unit)
    {
        smallest_unit
    } else {
        default_largest_unit
    };
    let largest_unit = if let Some(text) = largest_unit_text.as_deref() {
        if text == "auto" {
            default_largest_unit
        } else {
            temporal_date_difference_unit_from_text(text).ok_or_else(|| range_error(cx))?
        }
    } else {
        default_largest_unit
    };
    if temporal_date_difference_unit_order(largest_unit)
        > temporal_date_difference_unit_order(smallest_unit)
    {
        return Err(range_error(cx));
    }
    Ok(TemporalDateDifferenceOptions {
        largest_unit,
        smallest_unit,
        rounding_increment,
        rounding_mode,
    })
}

pub(super) fn temporal_round_i128_to_increment<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: i128,
    increment: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
) -> Result<i128, Cx::Error> {
    temporal_round_duration_nanoseconds_to_increment(value, increment, rounding_mode)
        .ok_or_else(|| range_error(cx))
}

pub(super) fn temporal_plain_date_ordering(
    left: TemporalPlainDateObjectData,
    right: TemporalPlainDateObjectData,
) -> std::cmp::Ordering {
    (left.year(), left.month(), left.day()).cmp(&(right.year(), right.month(), right.day()))
}

pub(super) fn temporal_plain_date_balanced_positive_months_until<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    start: TemporalPlainDateObjectData,
    end: TemporalPlainDateObjectData,
) -> Result<(i128, TemporalPlainDateObjectData), Cx::Error> {
    let mut months = (i128::from(end.year()) - i128::from(start.year()))
        .checked_mul(12)
        .and_then(|difference| {
            difference.checked_add(i128::from(end.month()) - i128::from(start.month()))
        })
        .ok_or_else(|| range_error(cx))?;
    let initial_months = i64::try_from(months).map_err(|_| range_error(cx))?;
    let mut candidate = temporal_plain_date_add_duration(
        cx,
        start,
        TemporalDurationObjectData::new(0, initial_months, 0, 0, 0, 0, 0, 0, 0, 0),
        TemporalOverflow::Constrain,
    )?;
    if temporal_plain_date_ordering(candidate, end).is_gt() {
        months = months.checked_sub(1).ok_or_else(|| range_error(cx))?;
        let adjusted_months = i64::try_from(months).map_err(|_| range_error(cx))?;
        candidate = temporal_plain_date_add_duration(
            cx,
            start,
            TemporalDurationObjectData::new(0, adjusted_months, 0, 0, 0, 0, 0, 0, 0, 0),
            TemporalOverflow::Constrain,
        )?;
    }
    Ok((months, candidate))
}

fn temporal_plain_date_balanced_negative_months_until<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    start: TemporalPlainDateObjectData,
    end: TemporalPlainDateObjectData,
) -> Result<(i128, TemporalPlainDateObjectData), Cx::Error> {
    let mut months = (i128::from(start.year()) - i128::from(end.year()))
        .checked_mul(12)
        .and_then(|difference| {
            difference.checked_add(i128::from(start.month()) - i128::from(end.month()))
        })
        .ok_or_else(|| range_error(cx))?;
    let initial_months = i64::try_from(-months).map_err(|_| range_error(cx))?;
    let mut candidate = temporal_plain_date_add_duration(
        cx,
        start,
        TemporalDurationObjectData::new(0, initial_months, 0, 0, 0, 0, 0, 0, 0, 0),
        TemporalOverflow::Constrain,
    )?;
    if temporal_plain_date_ordering(candidate, end).is_lt() {
        months = months.checked_sub(1).ok_or_else(|| range_error(cx))?;
        let adjusted_months = i64::try_from(-months).map_err(|_| range_error(cx))?;
        candidate = temporal_plain_date_add_duration(
            cx,
            start,
            TemporalDurationObjectData::new(0, adjusted_months, 0, 0, 0, 0, 0, 0, 0, 0),
            TemporalOverflow::Constrain,
        )?;
    }
    Ok((months, candidate))
}

pub(super) fn temporal_plain_date_difference_trunc<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    start: TemporalPlainDateObjectData,
    end: TemporalPlainDateObjectData,
    largest_unit: TemporalDateDifferenceUnit,
    smallest_unit: TemporalDateDifferenceUnit,
) -> Result<TemporalDurationObjectData, Cx::Error> {
    if temporal_plain_date_ordering(start, end).is_gt()
        && matches!(
            largest_unit,
            TemporalDateDifferenceUnit::Year | TemporalDateDifferenceUnit::Month
        )
    {
        let (total_months, after_months) =
            temporal_plain_date_balanced_negative_months_until(cx, start, end)?;
        let remainder_days = temporal_plain_date_ordinal_day(after_months)
            .checked_sub(temporal_plain_date_ordinal_day(end))
            .ok_or_else(|| range_error(cx))?;
        let (years, months) = if largest_unit == TemporalDateDifferenceUnit::Year {
            (total_months.div_euclid(12), total_months.rem_euclid(12))
        } else {
            (0, total_months)
        };
        let (years, months, weeks, days) = match smallest_unit {
            TemporalDateDifferenceUnit::Year => (years, 0, 0, 0),
            TemporalDateDifferenceUnit::Month => (years, months, 0, 0),
            TemporalDateDifferenceUnit::Week => (years, months, remainder_days / 7, 0),
            TemporalDateDifferenceUnit::Day => (years, months, 0, remainder_days),
        };
        return Ok(TemporalDurationObjectData::new(
            i64::try_from(-years).map_err(|_| range_error(cx))?,
            i64::try_from(-months).map_err(|_| range_error(cx))?,
            i64::try_from(-weeks).map_err(|_| range_error(cx))?,
            i64::try_from(-days).map_err(|_| range_error(cx))?,
            0,
            0,
            0,
            0,
            0,
            0,
        ));
    }

    let (sign, start, end) = match temporal_plain_date_ordering(start, end) {
        std::cmp::Ordering::Less | std::cmp::Ordering::Equal => (1_i64, start, end),
        std::cmp::Ordering::Greater => (-1_i64, end, start),
    };

    let (years, months, weeks, days) = match largest_unit {
        TemporalDateDifferenceUnit::Year | TemporalDateDifferenceUnit::Month => {
            let (total_months, after_months) =
                temporal_plain_date_balanced_positive_months_until(cx, start, end)?;
            let remainder_days = temporal_plain_date_ordinal_day(end)
                .checked_sub(temporal_plain_date_ordinal_day(after_months))
                .ok_or_else(|| range_error(cx))?;

            let (years, months) = if largest_unit == TemporalDateDifferenceUnit::Year {
                (total_months.div_euclid(12), total_months.rem_euclid(12))
            } else {
                (0, total_months)
            };
            match smallest_unit {
                TemporalDateDifferenceUnit::Year => (years, 0, 0, 0),
                TemporalDateDifferenceUnit::Month => (years, months, 0, 0),
                TemporalDateDifferenceUnit::Week => (years, months, remainder_days / 7, 0),
                TemporalDateDifferenceUnit::Day => (years, months, 0, remainder_days),
            }
        }
        TemporalDateDifferenceUnit::Week | TemporalDateDifferenceUnit::Day => {
            let total_days = temporal_plain_date_ordinal_day(end)
                .checked_sub(temporal_plain_date_ordinal_day(start))
                .ok_or_else(|| range_error(cx))?;
            match smallest_unit {
                TemporalDateDifferenceUnit::Week => (0, 0, total_days / 7, 0),
                TemporalDateDifferenceUnit::Day => {
                    if largest_unit == TemporalDateDifferenceUnit::Week {
                        (0, 0, total_days / 7, total_days % 7)
                    } else {
                        (0, 0, 0, total_days)
                    }
                }
                TemporalDateDifferenceUnit::Year | TemporalDateDifferenceUnit::Month => {
                    unreachable!("validated largest/smallest ordering")
                }
            }
        }
    };

    let sign = i128::from(sign);
    Ok(TemporalDurationObjectData::new(
        i64::try_from(years.checked_mul(sign).ok_or_else(|| range_error(cx))?)
            .map_err(|_| range_error(cx))?,
        i64::try_from(months.checked_mul(sign).ok_or_else(|| range_error(cx))?)
            .map_err(|_| range_error(cx))?,
        i64::try_from(weeks.checked_mul(sign).ok_or_else(|| range_error(cx))?)
            .map_err(|_| range_error(cx))?,
        i64::try_from(days.checked_mul(sign).ok_or_else(|| range_error(cx))?)
            .map_err(|_| range_error(cx))?,
        0,
        0,
        0,
        0,
        0,
        0,
    ))
}

pub(super) fn temporal_duration_from_date_units<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    units: i128,
    largest_unit: TemporalDateDifferenceUnit,
    unit_kind: TemporalDateDifferenceUnit,
) -> Result<TemporalDurationObjectData, Cx::Error> {
    let (years, months, weeks, days) = match (unit_kind, largest_unit) {
        (TemporalDateDifferenceUnit::Day, TemporalDateDifferenceUnit::Week) => {
            (0, 0, units / 7, units % 7)
        }
        (TemporalDateDifferenceUnit::Day, _) => (0, 0, 0, units),
        (TemporalDateDifferenceUnit::Week, _) => (0, 0, units, 0),
        (TemporalDateDifferenceUnit::Month, TemporalDateDifferenceUnit::Year) => {
            (units / 12, units % 12, 0, 0)
        }
        (TemporalDateDifferenceUnit::Month, _) => (0, units, 0, 0),
        (TemporalDateDifferenceUnit::Year, _) => (units, 0, 0, 0),
    };
    Ok(TemporalDurationObjectData::new(
        i64::try_from(years).map_err(|_| range_error(cx))?,
        i64::try_from(months).map_err(|_| range_error(cx))?,
        i64::try_from(weeks).map_err(|_| range_error(cx))?,
        i64::try_from(days).map_err(|_| range_error(cx))?,
        0,
        0,
        0,
        0,
        0,
        0,
    ))
}

pub(super) fn temporal_plain_date_difference_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    sign: i128,
) -> Result<Value, Cx::Error> {
    let date = temporal_plain_date_data(cx, invocation.this_value())?;
    let other = temporal_plain_date_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let options = temporal_date_difference_options(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
        TemporalDateDifferenceUnit::Day,
        TemporalDateDifferenceUnit::Day,
    )?;

    let relative_duration = temporal_plain_date_difference_trunc(
        cx,
        date,
        other,
        options.largest_unit,
        TemporalDateDifferenceUnit::Day,
    )?;
    let duration = if options.rounding_increment == 1
        && options.rounding_mode == TemporalBuiltinRoundingMode::Trunc
        && options.smallest_unit == TemporalDateDifferenceUnit::Day
    {
        if sign > 0 {
            negate_temporal_duration(relative_duration)
        } else {
            relative_duration
        }
    } else {
        let rounding_mode = if sign > 0 {
            temporal_rounding_mode_for_negated_duration(options.rounding_mode)
        } else {
            options.rounding_mode
        };
        let rounded = temporal_duration_round_calendar_relative(
            cx,
            relative_duration,
            TemporalDurationRelativeTo::PlainDate(date),
            options.largest_unit,
            options.smallest_unit,
            options.rounding_increment,
            rounding_mode,
        )?;
        if sign > 0 {
            negate_temporal_duration(rounded)
        } else {
            rounded
        }
    };
    validate_temporal_duration(cx, duration)?;
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(cx, prototype, duration)
}

pub(super) fn temporal_plain_date_since_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_plain_date_difference_builtin(cx, invocation, 1)
}

pub(super) fn temporal_plain_date_until_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_plain_date_difference_builtin(cx, invocation, -1)
}

pub(super) fn temporal_plain_date_to_plain_date_time_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let date = temporal_plain_date_data(cx, invocation.this_value())?;
    let time = match invocation.arguments().first().copied() {
        Some(value) if !value.is_undefined() => temporal_plain_time_from_value(cx, value)?,
        _ => TemporalPlainTimeObjectData::new(0, 0, 0, 0, 0, 0),
    };
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
    if !temporal_plain_date_time_is_within_limits(date.calendar(), total_nanoseconds) {
        return Err(range_error(cx));
    }
    let prototype = current_temporal_plain_date_time_prototype(cx)?;
    allocate_temporal_plain_date_time_object(cx, prototype, date_time_data)
}

pub(super) fn temporal_reject_calendar_or_time_zone_properties<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
) -> Result<(), Cx::Error> {
    let calendar = temporal_property_value(cx, object_ref, "calendar")?;
    if !calendar.is_undefined() {
        return Err(type_error(cx));
    }
    let time_zone = temporal_property_value(cx, object_ref, "timeZone")?;
    if !time_zone.is_undefined() {
        return Err(type_error(cx));
    }
    Ok(())
}

pub(super) fn temporal_plain_date_time_is_within_limits(
    calendar: AtomId,
    total_nanoseconds: i128,
) -> bool {
    let min = temporal_plain_date_ordinal_day(TemporalPlainDateObjectData::new(
        -271_821, 4, 19, calendar,
    )) * TEMPORAL_NANOS_PER_DAY;
    let max =
        temporal_plain_date_ordinal_day(TemporalPlainDateObjectData::new(275_760, 9, 13, calendar))
            * TEMPORAL_NANOS_PER_DAY
            + (TEMPORAL_NANOS_PER_DAY - 1);
    total_nanoseconds > min && total_nanoseconds <= max
}

pub(super) fn temporal_plain_date_to_zoned_date_time_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let date = temporal_plain_date_data(cx, invocation.this_value())?;
    let argument = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let midnight = TemporalPlainTimeObjectData::new(0, 0, 0, 0, 0, 0);
    let (time_zone_id, time) = if let Some(object_ref) = argument.as_object_ref() {
        let time_zone = temporal_property_value(cx, object_ref, "timeZone")?;
        if time_zone.is_undefined() {
            return Err(type_error(cx));
        }
        let plain_time = temporal_property_value(cx, object_ref, "plainTime")?;
        let time = if plain_time.is_undefined() {
            midnight
        } else {
            temporal_plain_time_from_value(cx, plain_time)?
        };
        (temporal_time_zone_id_from_value(cx, time_zone)?, time)
    } else {
        (temporal_time_zone_id_from_value(cx, argument)?, midnight)
    };
    let date_time = TemporalCivilDateTime::new(
        date.year(),
        date.month(),
        date.day(),
        time.hour(),
        time.minute(),
        time.second(),
        time.millisecond(),
        time.microsecond(),
        time.nanosecond(),
    );
    let instant = cx.temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
        time_zone_id: time_zone_id.clone(),
        date_time,
        disambiguation: TemporalDisambiguation::Compatible,
    })?;
    let zoned_date_time_data =
        temporal_zoned_date_time_from_parts(cx, instant.epoch_nanoseconds, &time_zone_id)?;
    let prototype = current_temporal_zoned_date_time_prototype(cx)?;
    allocate_temporal_zoned_date_time_object(cx, prototype, zoned_date_time_data)
}

pub(super) fn temporal_plain_date_to_plain_year_month_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let date = temporal_plain_date_data(cx, invocation.this_value())?;
    let year_month_data = TemporalPlainYearMonthObjectData::new(
        date.year(),
        date.month(),
        date.day(),
        date.calendar(),
    );
    let prototype = current_temporal_plain_year_month_prototype(cx)?;
    plain_year_month::allocate_temporal_plain_year_month_object(cx, prototype, year_month_data)
}

pub(super) fn temporal_plain_date_to_plain_month_day_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let date = temporal_plain_date_data(cx, invocation.this_value())?;
    let month_day_data = TemporalPlainMonthDayObjectData::new(
        date.month(),
        date.day(),
        date.year(),
        date.calendar(),
    );
    let prototype = current_temporal_plain_month_day_prototype(cx)?;
    plain_month_day::allocate_temporal_plain_month_day_object(cx, prototype, month_day_data)
}

pub(super) fn temporal_plain_date_from_builtin<Cx: PublicBuiltinDispatchContext>(
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
        let data = temporal_plain_date_from_value(cx, value)?;
        let _overflow = temporal_overflow_from_options(cx, options)?;
        data
    } else {
        let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
        let existing = {
            let agent = cx.agent();
            agent.objects().temporal_object(object_ref).copied()
        };
        match existing {
            Some(TemporalObjectData::PlainDate(data)) => {
                let _overflow = temporal_overflow_from_options(cx, options)?;
                data
            }
            Some(TemporalObjectData::PlainDateTime(data)) => {
                let _overflow = temporal_overflow_from_options(cx, options)?;
                temporal_plain_date_from_parts(
                    cx,
                    i64::from(data.year()),
                    i64::from(data.month()),
                    i64::from(data.day()),
                )?
            }
            Some(TemporalObjectData::ZonedDateTime(data)) => {
                let _overflow = temporal_overflow_from_options(cx, options)?;
                let civil = temporal_zoned_date_time_civil(cx, data)?;
                temporal_plain_date_from_parts(
                    cx,
                    i64::from(civil.date_time.year),
                    i64::from(civil.date_time.month),
                    i64::from(civil.date_time.day),
                )?
            }
            _ => {
                let fields = temporal_plain_date_bag_fields(cx, object_ref)?;
                if fields.year.is_none() || fields.day.is_none() {
                    return Err(type_error(cx));
                }
                if fields.month.is_none() && fields.month_code_text.is_none() {
                    return Err(type_error(cx));
                }
                let overflow = temporal_overflow_from_options(cx, options)?;
                temporal_plain_date_from_bag_fields(cx, &fields, overflow)?
            }
        }
    };
    let prototype = current_temporal_plain_date_prototype(cx)?;
    allocate_temporal_plain_date_object(cx, prototype, data)
}

pub(super) fn temporal_plain_date_compare_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = temporal_plain_date_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let right = temporal_plain_date_from_value(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(temporal_compare_ordering(
        (left.year(), left.month(), left.day()).cmp(&(right.year(), right.month(), right.day())),
    ))
}

use super::{
    allocate_temporal_plain_date_object, current_temporal_plain_date_prototype,
    current_temporal_plain_month_day_prototype, format_temporal_plain_date,
    format_temporal_plain_month_day, map_completion, object, parse_temporal_plain_month_day,
    range_error, string_ref_text, string_value, temporal_constructor_prototype,
    temporal_integer_part_from_argument, temporal_integer_part_from_value,
    temporal_iso_days_in_month, temporal_optional_integer_part_from_property,
    temporal_optional_string_text_from_property, temporal_overflow_from_options,
    temporal_parse_month_code_syntax, temporal_plain_date_from_parts_with_overflow,
    temporal_plain_date_to_string_calendar_name, temporal_reject_calendar_or_time_zone_properties,
    temporal_required_integer_part_from_property, temporal_resolve_month_from_fields,
    temporal_validate_optional_iso_calendar_identifier_argument,
    temporal_validate_optional_iso_calendar_property, type_error, AllocationLifetime,
    BuiltinFunctionId, BuiltinInvocation, ObjectAllocation, ObjectColdData, ObjectRef,
    OrdinaryObjectData, PublicBuiltinDispatchContext, RealmRecord, TemporalObjectData,
    TemporalObjectKind, TemporalOverflow, TemporalPlainDateObjectData,
    TemporalPlainMonthDayObjectData, TemporalZonedDateTimeCalendarNameOption, Value,
    TEMPORAL_DEFAULT_PLAIN_MONTH_DAY_REFERENCE_YEAR,
};

pub(super) fn dispatch_temporal_plain_month_day_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::temporal_plain_month_day_builtin() {
        return temporal_plain_month_day_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_month_day_month_code_getter_builtin() {
        return temporal_plain_month_day_month_code_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_month_day_day_getter_builtin() {
        return temporal_plain_month_day_day_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_month_day_calendar_id_getter_builtin() {
        return temporal_plain_month_day_calendar_id_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_month_day_to_string_builtin() {
        return temporal_plain_month_day_to_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_month_day_to_json_builtin() {
        return temporal_plain_month_day_to_json_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_month_day_to_locale_string_builtin() {
        return temporal_plain_month_day_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_month_day_value_of_builtin() {
        return temporal_plain_month_day_value_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_month_day_equals_builtin() {
        return temporal_plain_month_day_equals_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_month_day_with_builtin() {
        return temporal_plain_month_day_with_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_month_day_to_plain_date_builtin() {
        return temporal_plain_month_day_to_plain_date_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_month_day_from_builtin() {
        return temporal_plain_month_day_from_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

struct TemporalPlainMonthDayBagFields {
    year: Option<i64>,
    month: Option<i64>,
    month_code_text: Option<String>,
    day: Option<i64>,
}

fn temporal_plain_month_day_bag_fields<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
) -> Result<TemporalPlainMonthDayBagFields, Cx::Error> {
    let day = temporal_optional_integer_part_from_property(cx, object_ref, "day")?;
    let month = temporal_optional_integer_part_from_property(cx, object_ref, "month")?;
    let month_code_text = temporal_optional_string_text_from_property(cx, object_ref, "monthCode")?;
    if month_code_text
        .as_deref()
        .is_some_and(|text| temporal_parse_month_code_syntax(text).is_none())
    {
        return Err(range_error(cx));
    }
    Ok(TemporalPlainMonthDayBagFields {
        day,
        month,
        month_code_text,
        year: temporal_optional_integer_part_from_property(cx, object_ref, "year")?,
    })
}

fn temporal_plain_month_day_from_parts<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    month: i64,
    day: i64,
    reference_year: i64,
) -> Result<TemporalPlainMonthDayObjectData, Cx::Error> {
    temporal_plain_month_day_from_parts_with_overflow(
        cx,
        reference_year,
        month,
        day,
        reference_year,
        TemporalOverflow::Reject,
    )
}

fn temporal_plain_month_day_from_parts_with_overflow<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    overflow_year: i64,
    month: i64,
    day: i64,
    reference_year: i64,
    overflow: TemporalOverflow,
) -> Result<TemporalPlainMonthDayObjectData, Cx::Error> {
    if month < 1 || day < 1 {
        return Err(range_error(cx));
    }
    let overflow_year = i32::try_from(overflow_year).map_err(|_| range_error(cx))?;
    let reference_year = i32::try_from(reference_year).map_err(|_| range_error(cx))?;
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
    let max_day = i64::from(temporal_iso_days_in_month(overflow_year, month));
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
    if (reference_year, month, day) < (-271_821, 4, 19)
        || (reference_year, month, day) > (275_760, 9, 13)
    {
        return Err(range_error(cx));
    }
    let calendar = {
        let agent = cx.agent();
        agent.atoms_mut().intern_collectible("iso8601")
    };
    Ok(TemporalPlainMonthDayObjectData::new(
        month,
        day,
        reference_year,
        calendar,
    ))
}

fn temporal_plain_month_day_from_bag_fields<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    fields: TemporalPlainMonthDayBagFields,
    overflow: TemporalOverflow,
    reference_year: i64,
) -> Result<TemporalPlainMonthDayObjectData, Cx::Error> {
    let day = fields.day.ok_or_else(|| type_error(cx))?;
    let month = temporal_resolve_month_from_fields(
        cx,
        fields.month,
        fields.month_code_text.as_deref(),
        None,
    )?;
    temporal_plain_month_day_from_parts_with_overflow(
        cx,
        fields.year.unwrap_or(reference_year),
        month,
        day,
        reference_year,
        overflow,
    )
}

pub(super) fn allocate_temporal_plain_month_day_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: ObjectRef,
    data: TemporalPlainMonthDayObjectData,
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
                        TemporalObjectKind::PlainMonthDay,
                    ))),
                AllocationLifetime::Default,
            )
        })
    };
    let installed = cx
        .agent()
        .objects_mut()
        .install_temporal_object(object, TemporalObjectData::PlainMonthDay(data));
    if !installed {
        return Err(type_error(cx));
    }
    Ok(Value::from_object_ref(object))
}

fn temporal_plain_month_day_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalPlainMonthDayObjectData, Cx::Error> {
    if let Some(string_ref) = value.as_string_ref() {
        let text = string_ref_text(cx, string_ref)?;
        let (month, day, _overflow_year) =
            parse_temporal_plain_month_day(&text).ok_or_else(|| range_error(cx))?;
        return temporal_plain_month_day_from_parts(
            cx,
            i64::from(month),
            i64::from(day),
            TEMPORAL_DEFAULT_PLAIN_MONTH_DAY_REFERENCE_YEAR,
        );
    }

    let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let existing = {
        let agent = cx.agent();
        agent.objects().temporal_object(object_ref).copied()
    };
    match existing {
        Some(TemporalObjectData::PlainMonthDay(data)) => return Ok(data),
        Some(TemporalObjectData::PlainDate(data)) => {
            return temporal_plain_month_day_from_parts(
                cx,
                i64::from(data.month()),
                i64::from(data.day()),
                i64::from(data.year()),
            );
        }
        Some(TemporalObjectData::PlainDateTime(data)) => {
            return temporal_plain_month_day_from_parts(
                cx,
                i64::from(data.month()),
                i64::from(data.day()),
                i64::from(data.year()),
            );
        }
        _ => {}
    }

    temporal_validate_optional_iso_calendar_property(cx, object_ref)?;
    let fields = temporal_plain_month_day_bag_fields(cx, object_ref)?;
    temporal_plain_month_day_from_bag_fields(
        cx,
        fields,
        TemporalOverflow::Constrain,
        TEMPORAL_DEFAULT_PLAIN_MONTH_DAY_REFERENCE_YEAR,
    )
}

fn temporal_plain_month_day_data<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    this_value: Value,
) -> Result<TemporalPlainMonthDayObjectData, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, this_value, TemporalObjectKind::PlainMonthDay)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::PlainMonthDay(data) = payload else {
        return Err(type_error(cx));
    };
    Ok(data)
}

fn temporal_plain_month_day_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let Some(new_target) = invocation.new_target() else {
        return Err(type_error(cx));
    };
    let month = temporal_integer_part_from_argument(cx, invocation, 0)?;
    let day = temporal_integer_part_from_argument(cx, invocation, 1)?;
    temporal_validate_optional_iso_calendar_identifier_argument(cx, invocation, 2)?;
    let reference_year = match invocation.arguments().get(3).copied() {
        Some(value) if !value.is_undefined() => temporal_integer_part_from_value(cx, value)?,
        Some(_) | None => TEMPORAL_DEFAULT_PLAIN_MONTH_DAY_REFERENCE_YEAR,
    };
    let data = temporal_plain_month_day_from_parts(cx, month, day, reference_year)?;
    let prototype = temporal_constructor_prototype(cx, new_target)?;
    allocate_temporal_plain_month_day_object(cx, prototype, data)
}

fn temporal_plain_month_day_month_code_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_month_day_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format!("M{:02}", data.month())))
}

fn temporal_plain_month_day_day_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_month_day_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.day())))
}

fn temporal_plain_month_day_calendar_id_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_month_day_data(cx, invocation.this_value())?;
    Ok(string_value(cx, "iso8601"))
}

fn temporal_plain_month_day_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_month_day_data(cx, invocation.this_value())?;
    let calendar_name = temporal_plain_date_to_string_calendar_name(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let mut text = if matches!(
        calendar_name,
        TemporalZonedDateTimeCalendarNameOption::Always
            | TemporalZonedDateTimeCalendarNameOption::Critical
    ) {
        format_temporal_plain_date(TemporalPlainDateObjectData::new(
            data.reference_year(),
            data.month(),
            data.day(),
            data.calendar(),
        ))
    } else {
        format_temporal_plain_month_day(data)
    };
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

fn temporal_plain_month_day_to_json_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_month_day_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format_temporal_plain_month_day(data)))
}

fn temporal_plain_month_day_to_locale_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_plain_month_day_to_string_builtin(cx, invocation)
}

fn temporal_plain_month_day_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Err(type_error(cx))
}

fn temporal_plain_month_day_equals_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = temporal_plain_month_day_data(cx, invocation.this_value())?;
    let right = temporal_plain_month_day_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(left == right))
}

fn temporal_plain_month_day_with_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let month_day = temporal_plain_month_day_data(cx, invocation.this_value())?;
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
    let fields = temporal_plain_month_day_bag_fields(cx, object_ref)?;
    if fields.year.is_none()
        && fields.month.is_none()
        && fields.month_code_text.is_none()
        && fields.day.is_none()
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
    let month = temporal_resolve_month_from_fields(
        cx,
        fields.month,
        fields.month_code_text.as_deref(),
        Some(i64::from(month_day.month())),
    )?;
    let data = temporal_plain_month_day_from_parts_with_overflow(
        cx,
        fields.year.unwrap_or(i64::from(month_day.reference_year())),
        month,
        fields.day.unwrap_or(i64::from(month_day.day())),
        i64::from(month_day.reference_year()),
        overflow,
    )?;
    if !options.is_undefined() && options.as_object_ref().is_none() {
        return Err(type_error(cx));
    }
    let prototype = current_temporal_plain_month_day_prototype(cx)?;
    allocate_temporal_plain_month_day_object(cx, prototype, data)
}

fn temporal_plain_month_day_to_plain_date_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_month_day_data(cx, invocation.this_value())?;
    let item = invocation
        .arguments()
        .first()
        .and_then(|value| value.as_object_ref())
        .ok_or_else(|| type_error(cx))?;
    let year = temporal_required_integer_part_from_property(cx, item, "year")?;
    let date = temporal_plain_date_from_parts_with_overflow(
        cx,
        year,
        data.month().into(),
        data.day().into(),
        TemporalOverflow::Constrain,
    )?;
    let prototype = current_temporal_plain_date_prototype(cx)?;
    allocate_temporal_plain_date_object(cx, prototype, date)
}

fn temporal_plain_month_day_from_builtin<Cx: PublicBuiltinDispatchContext>(
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
        let data = temporal_plain_month_day_from_value(cx, value)?;
        let _overflow = temporal_overflow_from_options(cx, options)?;
        data
    } else {
        let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
        let existing = {
            let agent = cx.agent();
            agent.objects().temporal_object(object_ref).copied()
        };
        match existing {
            Some(TemporalObjectData::PlainMonthDay(data)) => {
                let _overflow = temporal_overflow_from_options(cx, options)?;
                data
            }
            Some(TemporalObjectData::PlainDate(data)) => {
                let _overflow = temporal_overflow_from_options(cx, options)?;
                temporal_plain_month_day_from_parts(
                    cx,
                    i64::from(data.month()),
                    i64::from(data.day()),
                    i64::from(data.year()),
                )?
            }
            Some(TemporalObjectData::PlainDateTime(data)) => {
                let _overflow = temporal_overflow_from_options(cx, options)?;
                temporal_plain_month_day_from_parts(
                    cx,
                    i64::from(data.month()),
                    i64::from(data.day()),
                    i64::from(data.year()),
                )?
            }
            _ => {
                temporal_validate_optional_iso_calendar_property(cx, object_ref)?;
                let fields = temporal_plain_month_day_bag_fields(cx, object_ref)?;
                let overflow = temporal_overflow_from_options(cx, options)?;
                temporal_plain_month_day_from_bag_fields(
                    cx,
                    fields,
                    overflow,
                    TEMPORAL_DEFAULT_PLAIN_MONTH_DAY_REFERENCE_YEAR,
                )?
            }
        }
    };
    let prototype = current_temporal_plain_month_day_prototype(cx)?;
    allocate_temporal_plain_month_day_object(cx, prototype, data)
}

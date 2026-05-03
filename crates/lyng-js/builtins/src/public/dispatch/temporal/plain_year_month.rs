use super::*;

pub(super) fn dispatch_temporal_plain_year_month_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::temporal_plain_year_month_builtin() {
        return temporal_plain_year_month_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_year_getter_builtin() {
        return temporal_plain_year_month_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_month_getter_builtin() {
        return temporal_plain_year_month_month_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_month_code_getter_builtin() {
        return temporal_plain_year_month_month_code_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_days_in_month_getter_builtin() {
        return temporal_plain_year_month_days_in_month_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_days_in_year_getter_builtin() {
        return temporal_plain_year_month_days_in_year_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_months_in_year_getter_builtin() {
        return temporal_plain_year_month_months_in_year_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_in_leap_year_getter_builtin() {
        return temporal_plain_year_month_in_leap_year_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_era_getter_builtin() {
        return temporal_plain_year_month_era_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_era_year_getter_builtin() {
        return temporal_plain_year_month_era_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_calendar_id_getter_builtin() {
        return temporal_plain_year_month_calendar_id_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_to_string_builtin() {
        return temporal_plain_year_month_to_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_to_json_builtin() {
        return temporal_plain_year_month_to_json_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_to_locale_string_builtin() {
        return temporal_plain_year_month_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_value_of_builtin() {
        return temporal_plain_year_month_value_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_equals_builtin() {
        return temporal_plain_year_month_equals_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_with_builtin() {
        return temporal_plain_year_month_with_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_add_builtin() {
        return temporal_plain_year_month_add_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_subtract_builtin() {
        return temporal_plain_year_month_subtract_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_since_builtin() {
        return temporal_plain_year_month_since_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_until_builtin() {
        return temporal_plain_year_month_until_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_to_plain_date_builtin() {
        return temporal_plain_year_month_to_plain_date_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_from_builtin() {
        return temporal_plain_year_month_from_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_plain_year_month_compare_builtin() {
        return temporal_plain_year_month_compare_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

struct TemporalPlainYearMonthBagFields {
    year: Option<i64>,
    month: Option<i64>,
    month_code_text: Option<String>,
}

fn temporal_plain_year_month_bag_fields<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
) -> Result<TemporalPlainYearMonthBagFields, Cx::Error> {
    temporal_validate_optional_iso_calendar_property(cx, object_ref)?;
    Ok(TemporalPlainYearMonthBagFields {
        month: temporal_optional_integer_part_from_property(cx, object_ref, "month")?,
        month_code_text: temporal_optional_month_code_text_from_property(cx, object_ref)?,
        year: temporal_optional_integer_part_from_property(cx, object_ref, "year")?,
    })
}

fn temporal_plain_year_month_from_parts<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    year: i64,
    month: i64,
    reference_day: i64,
) -> Result<TemporalPlainYearMonthObjectData, Cx::Error> {
    if !(-271_821..=275_760).contains(&year) {
        return Err(range_error(cx));
    }
    let year = i32::try_from(year).map_err(|_| range_error(cx))?;
    if !(1..=12).contains(&month) {
        return Err(range_error(cx));
    }
    let month = u8::try_from(month).map_err(|_| range_error(cx))?;
    let max_reference_day = i64::from(temporal_iso_days_in_month(year, month));
    if !(1..=max_reference_day).contains(&reference_day) {
        return Err(range_error(cx));
    }
    let reference_day = u8::try_from(reference_day).map_err(|_| range_error(cx))?;
    if (year, month) < (-271_821, 4) || (year, month) > (275_760, 9) {
        return Err(range_error(cx));
    }
    let calendar = {
        let agent = cx.agent();
        agent.atoms_mut().intern_collectible("iso8601")
    };
    Ok(TemporalPlainYearMonthObjectData::new(
        year,
        month,
        reference_day,
        calendar,
    ))
}

fn temporal_plain_year_month_from_parts_with_overflow<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    year: i64,
    month: i64,
    reference_day: i64,
    overflow: TemporalOverflow,
) -> Result<TemporalPlainYearMonthObjectData, Cx::Error> {
    if !(-271_821..=275_760).contains(&year) {
        return Err(range_error(cx));
    }
    let year = i32::try_from(year).map_err(|_| range_error(cx))?;
    let month = match overflow {
        TemporalOverflow::Constrain => {
            if month < 1 {
                return Err(range_error(cx));
            }
            month.min(12)
        }
        TemporalOverflow::Reject => {
            if !(1..=12).contains(&month) {
                return Err(range_error(cx));
            }
            month
        }
    };
    let month = u8::try_from(month).map_err(|_| range_error(cx))?;
    let max_reference_day = i64::from(temporal_iso_days_in_month(year, month));
    let reference_day = match overflow {
        TemporalOverflow::Constrain => reference_day.clamp(1, max_reference_day),
        TemporalOverflow::Reject => {
            if !(1..=max_reference_day).contains(&reference_day) {
                return Err(range_error(cx));
            }
            reference_day
        }
    };
    let reference_day = u8::try_from(reference_day).map_err(|_| range_error(cx))?;
    if (year, month) < (-271_821, 4) || (year, month) > (275_760, 9) {
        return Err(range_error(cx));
    }
    let calendar = {
        let agent = cx.agent();
        agent.atoms_mut().intern_collectible("iso8601")
    };
    Ok(TemporalPlainYearMonthObjectData::new(
        year,
        month,
        reference_day,
        calendar,
    ))
}

pub(super) fn allocate_temporal_plain_year_month_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: ObjectRef,
    data: TemporalPlainYearMonthObjectData,
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
                        TemporalObjectKind::PlainYearMonth,
                    ))),
                AllocationLifetime::Default,
            )
        })
    };
    let installed = cx
        .agent()
        .objects_mut()
        .install_temporal_object(object, TemporalObjectData::PlainYearMonth(data));
    if !installed {
        return Err(type_error(cx));
    }
    Ok(Value::from_object_ref(object))
}

fn temporal_plain_year_month_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalPlainYearMonthObjectData, Cx::Error> {
    if let Some(string_ref) = value.as_string_ref() {
        let text = string_ref_text(cx, string_ref)?;
        let (year, month, _reference_day) =
            parse_temporal_plain_year_month(&text).ok_or_else(|| range_error(cx))?;
        return temporal_plain_year_month_from_parts(cx, i64::from(year), i64::from(month), 1);
    }

    let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let existing = {
        let agent = cx.agent();
        agent.objects().temporal_object(object_ref).copied()
    };
    match existing {
        Some(TemporalObjectData::PlainYearMonth(data)) => return Ok(data),
        Some(TemporalObjectData::PlainDate(data)) => {
            return temporal_plain_year_month_from_parts(
                cx,
                i64::from(data.year()),
                i64::from(data.month()),
                1,
            );
        }
        Some(TemporalObjectData::PlainDateTime(data)) => {
            return temporal_plain_year_month_from_parts(
                cx,
                i64::from(data.year()),
                i64::from(data.month()),
                1,
            );
        }
        _ => {}
    }

    let fields = temporal_plain_year_month_bag_fields(cx, object_ref)?;
    let year = fields.year.ok_or_else(|| type_error(cx))?;
    let month = temporal_resolve_month_from_fields(
        cx,
        fields.month,
        fields.month_code_text.as_deref(),
        None,
    )?;
    temporal_plain_year_month_from_parts(cx, year, month, 1)
}

fn temporal_plain_year_month_data<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    this_value: Value,
) -> Result<TemporalPlainYearMonthObjectData, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, this_value, TemporalObjectKind::PlainYearMonth)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::PlainYearMonth(data) = payload else {
        return Err(type_error(cx));
    };
    Ok(data)
}

fn temporal_plain_year_month_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let Some(new_target) = invocation.new_target() else {
        return Err(type_error(cx));
    };
    let year = temporal_integer_part_from_argument(cx, invocation, 0)?;
    let month = temporal_integer_part_from_argument(cx, invocation, 1)?;
    temporal_validate_optional_iso_calendar_identifier_argument(cx, invocation, 2)?;
    let reference_day = match invocation.arguments().get(3).copied() {
        Some(value) => temporal_integer_part_from_value(cx, value)?,
        None => 1,
    };
    let data = temporal_plain_year_month_from_parts(cx, year, month, reference_day)?;
    let prototype = temporal_constructor_prototype(cx, new_target)?;
    allocate_temporal_plain_year_month_object(cx, prototype, data)
}

fn temporal_plain_year_month_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_year_month_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(data.year()))
}

fn temporal_plain_year_month_month_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_year_month_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.month())))
}

fn temporal_plain_year_month_month_code_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_year_month_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format!("M{:02}", data.month())))
}

fn temporal_plain_year_month_days_in_month_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_year_month_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(temporal_iso_days_in_month(
        data.year(),
        data.month(),
    ))))
}

fn temporal_plain_year_month_days_in_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_year_month_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(temporal_iso_days_in_year(data.year())))
}

fn temporal_plain_year_month_months_in_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_year_month_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(12))
}

fn temporal_plain_year_month_in_leap_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_year_month_data(cx, invocation.this_value())?;
    Ok(Value::from_bool(temporal_is_iso_leap_year(data.year())))
}

fn temporal_plain_year_month_era_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_year_month_data(cx, invocation.this_value())?;
    Ok(Value::undefined())
}

fn temporal_plain_year_month_era_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_year_month_data(cx, invocation.this_value())?;
    Ok(Value::undefined())
}

fn temporal_plain_year_month_calendar_id_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_year_month_data(cx, invocation.this_value())?;
    Ok(string_value(cx, "iso8601"))
}

fn temporal_plain_year_month_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_year_month_data(cx, invocation.this_value())?;
    let calendar_name = temporal_plain_date_to_string_calendar_name(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let year_text = temporal_ops::format_iso_year(data.year());
    let text = match calendar_name {
        TemporalZonedDateTimeCalendarNameOption::Auto
        | TemporalZonedDateTimeCalendarNameOption::Never => {
            format!("{year_text}-{:02}", data.month())
        }
        TemporalZonedDateTimeCalendarNameOption::Always => format!(
            "{year_text}-{:02}-{:02}[u-ca=iso8601]",
            data.month(),
            data.reference_day()
        ),
        TemporalZonedDateTimeCalendarNameOption::Critical => format!(
            "{year_text}-{:02}-{:02}[!u-ca=iso8601]",
            data.month(),
            data.reference_day()
        ),
    };
    Ok(string_value(cx, &text))
}

fn temporal_plain_year_month_to_json_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_year_month_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format_temporal_plain_year_month(data)))
}

fn temporal_plain_year_month_to_locale_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_plain_year_month_to_string_builtin(cx, invocation)
}

fn temporal_plain_year_month_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Err(type_error(cx))
}

fn temporal_plain_year_month_equals_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = temporal_plain_year_month_data(cx, invocation.this_value())?;
    let right = temporal_plain_year_month_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(left == right))
}

fn temporal_plain_year_month_with_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let year_month = temporal_plain_year_month_data(cx, invocation.this_value())?;
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
    let month = temporal_optional_integer_part_from_property(cx, object_ref, "month")?;
    let month_code_text = temporal_optional_month_code_text_from_property(cx, object_ref)?;
    let year = temporal_optional_integer_part_from_property(cx, object_ref, "year")?;
    if year.is_none() && month.is_none() && month_code_text.is_none() {
        return Err(type_error(cx));
    }
    let options = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    if !options.is_undefined() && options.as_object_ref().is_none() {
        let month = temporal_resolve_month_from_fields(
            cx,
            month,
            month_code_text.as_deref(),
            Some(i64::from(year_month.month())),
        )?;
        let _ = temporal_plain_year_month_from_parts(
            cx,
            year.unwrap_or(i64::from(year_month.year())),
            month,
            i64::from(year_month.reference_day()),
        )?;
    }
    let overflow = temporal_overflow_from_options(cx, options)?;
    let month = temporal_resolve_month_from_fields(
        cx,
        month,
        month_code_text.as_deref(),
        Some(i64::from(year_month.month())),
    )?;
    let data = temporal_plain_year_month_from_parts_with_overflow(
        cx,
        year.unwrap_or(i64::from(year_month.year())),
        month,
        i64::from(year_month.reference_day()),
        overflow,
    )?;
    let prototype = current_temporal_plain_year_month_prototype(cx)?;
    allocate_temporal_plain_year_month_object(cx, prototype, data)
}

fn temporal_plain_year_month_add_duration<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    year_month: TemporalPlainYearMonthObjectData,
    duration: TemporalDurationObjectData,
    overflow: TemporalOverflow,
) -> Result<TemporalPlainYearMonthObjectData, Cx::Error> {
    let _ = temporal_plain_date_from_parts(
        cx,
        i64::from(year_month.year()),
        i64::from(year_month.month()),
        i64::from(year_month.reference_day()),
    )?;
    if temporal_duration_has_lower_than_month_units(duration) {
        return Err(range_error(cx));
    }

    let total_months = i128::from(year_month.year()) * 12
        + i128::from(year_month.month() - 1)
        + i128::from(duration.years()) * 12
        + i128::from(duration.months());
    let year = total_months.div_euclid(12);
    let month = total_months.rem_euclid(12) + 1;
    if !(-271_821..=275_760).contains(&year) {
        return Err(range_error(cx));
    }

    let year = i32::try_from(year).map_err(|_| range_error(cx))?;
    let month = u8::try_from(month).map_err(|_| range_error(cx))?;
    let reference_day = i64::from(year_month.reference_day());
    temporal_plain_year_month_from_parts_with_overflow(
        cx,
        i64::from(year),
        i64::from(month),
        reference_day,
        overflow,
    )
}

fn temporal_plain_year_month_round_relative_months<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    start: TemporalPlainDateObjectData,
    months: i128,
    largest_unit: TemporalDateDifferenceUnit,
    smallest_unit: TemporalDateDifferenceUnit,
    rounding_increment: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
) -> Result<i128, Cx::Error> {
    let rounded = match smallest_unit {
        TemporalDateDifferenceUnit::Year => {
            let increment = rounding_increment
                .checked_mul(12)
                .ok_or_else(|| range_error(cx))?;
            temporal_round_i128_to_increment(cx, months, increment, rounding_mode)?
        }
        TemporalDateDifferenceUnit::Month if largest_unit == TemporalDateDifferenceUnit::Year => {
            let years = months / 12;
            let month_remainder = months % 12;
            let rounded_remainder = temporal_round_i128_to_increment(
                cx,
                month_remainder,
                rounding_increment,
                rounding_mode,
            )?;
            years
                .checked_mul(12)
                .and_then(|year_months| year_months.checked_add(rounded_remainder))
                .ok_or_else(|| range_error(cx))?
        }
        TemporalDateDifferenceUnit::Month => {
            temporal_round_i128_to_increment(cx, months, rounding_increment, rounding_mode)?
        }
        TemporalDateDifferenceUnit::Week | TemporalDateDifferenceUnit::Day => {
            unreachable!("PlainYearMonth filters lower date units")
        }
    };

    temporal_duration_validate_month_rounding_boundary(cx, start, rounded)?;
    if rounding_increment != 1 {
        let adjacent = match smallest_unit {
            TemporalDateDifferenceUnit::Year => {
                rounding_increment.checked_mul(12).and_then(|increment| {
                    if months < 0 {
                        rounded.checked_sub(increment)
                    } else {
                        rounded.checked_add(increment)
                    }
                })
            }
            TemporalDateDifferenceUnit::Month => {
                if months < 0 {
                    rounded.checked_sub(rounding_increment)
                } else {
                    rounded.checked_add(rounding_increment)
                }
            }
            TemporalDateDifferenceUnit::Week | TemporalDateDifferenceUnit::Day => {
                unreachable!("PlainYearMonth filters lower date units")
            }
        }
        .ok_or_else(|| range_error(cx))?;
        temporal_duration_validate_month_rounding_boundary(cx, start, adjacent)?;
    }

    Ok(rounded)
}

fn temporal_plain_year_month_add_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let year_month = temporal_plain_year_month_data(cx, invocation.this_value())?;
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
    let data = temporal_plain_year_month_add_duration(cx, year_month, duration, overflow)?;
    let prototype = current_temporal_plain_year_month_prototype(cx)?;
    allocate_temporal_plain_year_month_object(cx, prototype, data)
}

fn temporal_plain_year_month_subtract_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let year_month = temporal_plain_year_month_data(cx, invocation.this_value())?;
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
    let data = temporal_plain_year_month_add_duration(
        cx,
        year_month,
        negate_temporal_duration(duration),
        overflow,
    )?;
    let prototype = current_temporal_plain_year_month_prototype(cx)?;
    allocate_temporal_plain_year_month_object(cx, prototype, data)
}

fn temporal_plain_year_month_difference_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    sign: i128,
) -> Result<Value, Cx::Error> {
    let year_month = temporal_plain_year_month_data(cx, invocation.this_value())?;
    let other = temporal_plain_year_month_from_value(
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
        TemporalDateDifferenceUnit::Year,
        TemporalDateDifferenceUnit::Month,
    )?;
    if matches!(
        options.largest_unit,
        TemporalDateDifferenceUnit::Week | TemporalDateDifferenceUnit::Day
    ) || matches!(
        options.smallest_unit,
        TemporalDateDifferenceUnit::Week | TemporalDateDifferenceUnit::Day
    ) {
        return Err(range_error(cx));
    }

    let start_date = if year_month != other {
        let start_date = temporal_plain_date_from_parts(
            cx,
            i64::from(year_month.year()),
            i64::from(year_month.month()),
            1,
        )?;
        let _ = temporal_plain_date_from_parts(
            cx,
            i64::from(other.year()),
            i64::from(other.month()),
            1,
        )?;
        Some(start_date)
    } else {
        None
    };

    let left = i128::from(year_month.year()) * 12 + i128::from(year_month.month() - 1);
    let right = i128::from(other.year()) * 12 + i128::from(other.month() - 1);
    let relative_months = right.checked_sub(left).ok_or_else(|| range_error(cx))?;
    let result_months = if options.rounding_increment == 1
        && options.rounding_mode == TemporalBuiltinRoundingMode::Trunc
        && options.smallest_unit == TemporalDateDifferenceUnit::Month
    {
        if sign > 0 {
            relative_months
                .checked_neg()
                .ok_or_else(|| range_error(cx))?
        } else {
            relative_months
        }
    } else {
        let rounding_mode = if sign > 0 {
            temporal_rounding_mode_for_negated_duration(options.rounding_mode)
        } else {
            options.rounding_mode
        };
        let start_date = match start_date {
            Some(start_date) => start_date,
            None => temporal_plain_date_from_parts(
                cx,
                i64::from(year_month.year()),
                i64::from(year_month.month()),
                1,
            )?,
        };
        let rounded = temporal_plain_year_month_round_relative_months(
            cx,
            start_date,
            relative_months,
            options.largest_unit,
            options.smallest_unit,
            options.rounding_increment,
            rounding_mode,
        )?;
        if sign > 0 {
            rounded.checked_neg().ok_or_else(|| range_error(cx))?
        } else {
            rounded
        }
    };
    let duration = temporal_duration_from_date_units(
        cx,
        result_months,
        options.largest_unit,
        TemporalDateDifferenceUnit::Month,
    )?;
    validate_temporal_duration(cx, duration)?;
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(cx, prototype, duration)
}

fn temporal_plain_year_month_since_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_plain_year_month_difference_builtin(cx, invocation, 1)
}

fn temporal_plain_year_month_until_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_plain_year_month_difference_builtin(cx, invocation, -1)
}

fn temporal_plain_year_month_to_plain_date_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_year_month_data(cx, invocation.this_value())?;
    let item = invocation
        .arguments()
        .first()
        .and_then(|value| value.as_object_ref())
        .ok_or_else(|| type_error(cx))?;
    let day = temporal_required_integer_part_from_property(cx, item, "day")?;
    let date = temporal_plain_date_from_parts_with_overflow(
        cx,
        i64::from(data.year()),
        data.month().into(),
        day,
        TemporalOverflow::Constrain,
    )?;
    let prototype = current_temporal_plain_date_prototype(cx)?;
    allocate_temporal_plain_date_object(cx, prototype, date)
}

fn temporal_plain_year_month_from_builtin<Cx: PublicBuiltinDispatchContext>(
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
        let data = temporal_plain_year_month_from_value(cx, value)?;
        let _overflow = temporal_overflow_from_options(cx, options)?;
        data
    } else {
        let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
        let existing = {
            let agent = cx.agent();
            agent.objects().temporal_object(object_ref).copied()
        };
        match existing {
            Some(TemporalObjectData::PlainYearMonth(data)) => {
                let _overflow = temporal_overflow_from_options(cx, options)?;
                data
            }
            Some(TemporalObjectData::PlainDate(data)) => {
                let _overflow = temporal_overflow_from_options(cx, options)?;
                temporal_plain_year_month_from_parts(
                    cx,
                    i64::from(data.year()),
                    i64::from(data.month()),
                    1,
                )?
            }
            Some(TemporalObjectData::PlainDateTime(data)) => {
                let _overflow = temporal_overflow_from_options(cx, options)?;
                temporal_plain_year_month_from_parts(
                    cx,
                    i64::from(data.year()),
                    i64::from(data.month()),
                    1,
                )?
            }
            _ => {
                let fields = temporal_plain_year_month_bag_fields(cx, object_ref)?;
                if fields.year.is_none() {
                    return Err(type_error(cx));
                }
                if fields.month.is_none() && fields.month_code_text.is_none() {
                    return Err(type_error(cx));
                }
                let overflow = temporal_overflow_from_options(cx, options)?;
                let month = temporal_resolve_month_from_fields(
                    cx,
                    fields.month,
                    fields.month_code_text.as_deref(),
                    None,
                )?;
                temporal_plain_year_month_from_parts_with_overflow(
                    cx,
                    fields.year.expect("checked above"),
                    month,
                    1,
                    overflow,
                )?
            }
        }
    };
    let prototype = current_temporal_plain_year_month_prototype(cx)?;
    allocate_temporal_plain_year_month_object(cx, prototype, data)
}

fn temporal_plain_year_month_compare_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = temporal_plain_year_month_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let right = temporal_plain_year_month_from_value(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(temporal_compare_ordering(
        (left.year(), left.month(), left.reference_day()).cmp(&(
            right.year(),
            right.month(),
            right.reference_day(),
        )),
    ))
}

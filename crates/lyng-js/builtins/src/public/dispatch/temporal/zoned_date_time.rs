use super::*;

pub(super) fn dispatch_temporal_zoned_date_time_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::temporal_zoned_date_time_builtin() {
        return temporal_zoned_date_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_year_getter_builtin() {
        return temporal_zoned_date_time_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_month_getter_builtin() {
        return temporal_zoned_date_time_month_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_month_code_getter_builtin() {
        return temporal_zoned_date_time_month_code_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_day_getter_builtin() {
        return temporal_zoned_date_time_day_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_day_of_week_getter_builtin() {
        return temporal_zoned_date_time_day_of_week_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_day_of_year_getter_builtin() {
        return temporal_zoned_date_time_day_of_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_days_in_month_getter_builtin() {
        return temporal_zoned_date_time_days_in_month_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_days_in_year_getter_builtin() {
        return temporal_zoned_date_time_days_in_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_months_in_year_getter_builtin() {
        return temporal_zoned_date_time_months_in_year_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_in_leap_year_getter_builtin() {
        return temporal_zoned_date_time_in_leap_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_days_in_week_getter_builtin() {
        return temporal_zoned_date_time_days_in_week_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_week_of_year_getter_builtin() {
        return temporal_zoned_date_time_week_of_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_year_of_week_getter_builtin() {
        return temporal_zoned_date_time_year_of_week_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_era_getter_builtin() {
        return temporal_zoned_date_time_era_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_era_year_getter_builtin() {
        return temporal_zoned_date_time_era_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_hour_getter_builtin() {
        return temporal_zoned_date_time_hour_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_minute_getter_builtin() {
        return temporal_zoned_date_time_minute_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_second_getter_builtin() {
        return temporal_zoned_date_time_second_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_millisecond_getter_builtin() {
        return temporal_zoned_date_time_millisecond_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_microsecond_getter_builtin() {
        return temporal_zoned_date_time_microsecond_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_nanosecond_getter_builtin() {
        return temporal_zoned_date_time_nanosecond_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_epoch_nanoseconds_getter_builtin() {
        return temporal_zoned_date_time_epoch_nanoseconds_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_epoch_milliseconds_getter_builtin() {
        return temporal_zoned_date_time_epoch_milliseconds_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_time_zone_id_getter_builtin() {
        return temporal_zoned_date_time_time_zone_id_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_calendar_id_getter_builtin() {
        return temporal_zoned_date_time_calendar_id_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_offset_getter_builtin() {
        return temporal_zoned_date_time_offset_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_offset_nanoseconds_getter_builtin() {
        return temporal_zoned_date_time_offset_nanoseconds_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_to_string_builtin() {
        return temporal_zoned_date_time_to_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_to_json_builtin() {
        return temporal_zoned_date_time_to_json_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_to_locale_string_builtin() {
        return temporal_zoned_date_time_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_value_of_builtin() {
        return temporal_zoned_date_time_value_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_equals_builtin() {
        return temporal_zoned_date_time_equals_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_add_builtin() {
        return temporal_zoned_date_time_add_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_round_builtin() {
        return temporal_zoned_date_time_round_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_with_builtin() {
        return temporal_zoned_date_time_with_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_subtract_builtin() {
        return temporal_zoned_date_time_subtract_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_with_time_zone_builtin() {
        return temporal_zoned_date_time_with_time_zone_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_with_calendar_builtin() {
        return temporal_zoned_date_time_with_calendar_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_with_plain_time_builtin() {
        return temporal_zoned_date_time_with_plain_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_start_of_day_builtin() {
        return temporal_zoned_date_time_start_of_day_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_get_time_zone_transition_builtin() {
        return temporal_zoned_date_time_get_time_zone_transition_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_hours_in_day_getter_builtin() {
        return temporal_zoned_date_time_hours_in_day_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_since_builtin() {
        return temporal_zoned_date_time_since_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_until_builtin() {
        return temporal_zoned_date_time_until_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_from_builtin() {
        return temporal_zoned_date_time_from_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_compare_builtin() {
        return temporal_zoned_date_time_compare_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_to_instant_builtin() {
        return temporal_zoned_date_time_to_instant_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_to_plain_date_time_builtin() {
        return temporal_zoned_date_time_to_plain_date_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_to_plain_date_builtin() {
        return temporal_zoned_date_time_to_plain_date_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_zoned_date_time_to_plain_time_builtin() {
        return temporal_zoned_date_time_to_plain_time_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

pub(super) fn temporal_time_zone_id_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<String, Cx::Error> {
    if let Some(object_ref) = value.as_object_ref() {
        let temporal = {
            let agent = cx.agent();
            agent.objects().temporal_object(object_ref).copied()
        };
        if let Some(TemporalObjectData::ZonedDateTime(data)) = temporal {
            return temporal_atom_text(cx, data.time_zone());
        }
        return Err(type_error(cx));
    }
    let Some(string_ref) = value.as_string_ref() else {
        return Err(type_error(cx));
    };
    let text = string_ref_text(cx, string_ref)?;
    temporal_time_zone_id_from_string(&text).ok_or_else(|| range_error(cx))
}

pub(super) fn temporal_time_zone_id_from_string(text: &str) -> Option<String> {
    if text.eq_ignore_ascii_case(TEMPORAL_UTC_TIME_ZONE_ID) {
        return Some(TEMPORAL_UTC_TIME_ZONE_ID.to_string());
    }
    if let Some(offset_nanoseconds) = temporal_parse_fixed_offset_time_zone_id(text) {
        return Some(format_temporal_offset(offset_nanoseconds));
    }
    if text.contains('[') {
        let time_zone_id = temporal_zoned_date_time_zone_annotation(text)?;
        let prefix = text.split_once('[').map_or(text, |(prefix, _)| prefix);
        if !prefix.is_empty()
            && !prefix.eq_ignore_ascii_case(TEMPORAL_UTC_TIME_ZONE_ID)
            && temporal_parse_fixed_offset_time_zone_id(prefix).is_none()
            && parse_temporal_instant(prefix).is_none()
            && parse_temporal_plain_date_time(prefix).is_none()
        {
            return None;
        }
        return Some(time_zone_id);
    }
    temporal_time_zone_id_from_iso_date_time_offset(text)
}

pub(super) fn temporal_parse_fixed_offset_time_zone_id(text: &str) -> Option<i64> {
    temporal_parse_offset_string(text, false)
}

pub(super) fn temporal_parse_offset_string(text: &str, allow_subminute: bool) -> Option<i64> {
    fn parse_two_digits(bytes: &[u8], index: &mut usize) -> Option<i128> {
        let tens = *bytes.get(*index)?;
        let ones = *bytes.get(*index + 1)?;
        if !tens.is_ascii_digit() || !ones.is_ascii_digit() {
            return None;
        }
        *index += 2;
        Some(i128::from(tens - b'0') * 10 + i128::from(ones - b'0'))
    }

    let bytes = text.as_bytes();
    let sign = match bytes.first().copied()? {
        b'+' => 1_i128,
        b'-' => -1_i128,
        _ => return None,
    };
    let mut index = 1;
    let hours = parse_two_digits(bytes, &mut index)?;
    let mut minutes = 0_i128;
    let mut seconds = 0_i128;
    let mut fraction = 0_i128;
    let mut has_subminute_syntax = false;
    let separated = matches!(bytes.get(index).copied(), Some(b':'));
    if index < bytes.len() {
        if separated {
            index += 1;
        }
        minutes = parse_two_digits(bytes, &mut index)?;
    }
    if index < bytes.len() {
        has_subminute_syntax = true;
        if separated {
            if !matches!(bytes.get(index).copied(), Some(b':')) {
                return None;
            }
            index += 1;
        }
        seconds = parse_two_digits(bytes, &mut index)?;
        if matches!(bytes.get(index).copied(), Some(b'.')) {
            index += 1;
            let start = index;
            while index < bytes.len() && index - start < 9 && bytes[index].is_ascii_digit() {
                fraction = fraction
                    .checked_mul(10)?
                    .checked_add(i128::from(bytes[index] - b'0'))?;
                index += 1;
            }
            if index == start {
                return None;
            }
            for _ in (index - start)..9 {
                fraction = fraction.checked_mul(10)?;
            }
        }
    }
    if index != bytes.len() || hours > 23 || minutes > 59 || seconds > 59 {
        return None;
    }
    if !allow_subminute && has_subminute_syntax {
        return None;
    }
    let total = hours
        .checked_mul(60)?
        .checked_add(minutes)?
        .checked_mul(60)?
        .checked_add(seconds)?
        .checked_mul(TEMPORAL_NANOS_PER_SECOND)?
        .checked_add(fraction)?
        .checked_mul(sign)?;
    i64::try_from(total).ok()
}

pub(super) fn temporal_time_zone_id_from_iso_date_time_offset(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let time_separator = bytes
        .iter()
        .position(|byte| matches!(byte, b'T' | b't' | b' '))?;
    if matches!(bytes.last().copied(), Some(b'Z' | b'z')) {
        let prefix = &text[..text.len() - 1];
        parse_temporal_plain_date_time(prefix)?;
        return Some(TEMPORAL_UTC_TIME_ZONE_ID.to_string());
    }
    let offset_start = bytes
        .iter()
        .enumerate()
        .skip(time_separator + 1)
        .rev()
        .find_map(|(index, byte)| matches!(byte, b'+' | b'-').then_some(index))?;
    let prefix = &text[..offset_start];
    parse_temporal_plain_date_time(prefix)?;
    let offset_nanoseconds = temporal_parse_fixed_offset_time_zone_id(&text[offset_start..])?;
    Some(format_temporal_offset(offset_nanoseconds))
}

fn temporal_time_zone_string_has_iso_date_time_prefix(text: &str) -> bool {
    let prefix = text.split_once('[').map_or(text, |(prefix, _)| prefix);
    parse_temporal_instant(prefix).is_some()
        || parse_temporal_plain_date_time(prefix).is_some()
        || temporal_time_zone_id_from_iso_date_time_offset(text).is_some()
}

fn temporal_time_zone_id_from_constructor_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<String, Cx::Error> {
    if let Some(string_ref) = value.as_string_ref() {
        let text = string_ref_text(cx, string_ref)?;
        if temporal_time_zone_string_has_iso_date_time_prefix(&text) {
            return Err(range_error(cx));
        }
        return temporal_time_zone_id_from_string(&text).ok_or_else(|| range_error(cx));
    }
    temporal_time_zone_id_from_value(cx, value)
}

pub(super) fn temporal_time_zone_id_from_optional_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<String, Cx::Error> {
    if value.is_undefined() {
        let zone = cx.temporal_default_time_zone(&TemporalDefaultTimeZoneRequest {})?;
        return Ok(zone.time_zone_id);
    }
    temporal_time_zone_id_from_value(cx, value)
}

pub(super) fn temporal_now_instant_and_civil<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    time_zone_id: &str,
) -> Result<(i128, TemporalCivilTime), Cx::Error> {
    let instant = cx.temporal_current_instant(&TemporalCurrentInstantRequest {})?;
    let civil = cx.temporal_instant_to_civil_time(&TemporalInstantToCivilRequest {
        time_zone_id: time_zone_id.to_string(),
        epoch_nanoseconds: instant.epoch_nanoseconds,
    })?;
    Ok((instant.epoch_nanoseconds, civil))
}

pub(super) fn temporal_atom_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    atom: lyng_js_common::AtomId,
) -> Result<String, Cx::Error> {
    cx.agent()
        .atoms()
        .get(atom)
        .map(str::to_string)
        .ok_or_else(|| type_error(cx))
}

pub(super) fn temporal_zoned_date_time_from_parts<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    epoch_nanoseconds: i128,
    time_zone_id: &str,
) -> Result<TemporalZonedDateTimeObjectData, Cx::Error> {
    if !temporal_instant_epoch_nanoseconds_is_valid(epoch_nanoseconds) {
        return Err(range_error(cx));
    }
    let time_zone = {
        let agent = cx.agent();
        agent.atoms_mut().intern_collectible(time_zone_id)
    };
    let calendar = {
        let agent = cx.agent();
        agent.atoms_mut().intern_collectible("iso8601")
    };
    Ok(TemporalZonedDateTimeObjectData::new(
        epoch_nanoseconds,
        time_zone,
        calendar,
    ))
}

pub(super) fn allocate_temporal_zoned_date_time_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: ObjectRef,
    data: TemporalZonedDateTimeObjectData,
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
                        TemporalObjectKind::ZonedDateTime,
                    ))),
                AllocationLifetime::Default,
            )
        })
    };
    let installed = cx
        .agent()
        .objects_mut()
        .install_temporal_object(object, TemporalObjectData::ZonedDateTime(data));
    if !installed {
        return Err(type_error(cx));
    }
    Ok(Value::from_object_ref(object))
}

pub(super) fn temporal_zoned_date_time_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalZonedDateTimeObjectData, Cx::Error> {
    temporal_zoned_date_time_from_value_with_options(cx, value, Value::undefined())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TemporalZonedDateTimeOffsetBehavior {
    Prefer,
    Use,
    Ignore,
    Reject,
}

struct TemporalZonedDateTimeFromOptions {
    disambiguation: TemporalDisambiguation,
    offset: TemporalZonedDateTimeOffsetBehavior,
    overflow: TemporalOverflow,
}

fn temporal_zoned_date_time_offset_behavior_from_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    options: Value,
) -> Result<TemporalZonedDateTimeOffsetBehavior, Cx::Error> {
    temporal_zoned_date_time_offset_behavior_from_options_with_default(
        cx,
        options,
        TemporalZonedDateTimeOffsetBehavior::Reject,
    )
}

fn temporal_zoned_date_time_offset_behavior_from_options_with_default<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    options: Value,
    default: TemporalZonedDateTimeOffsetBehavior,
) -> Result<TemporalZonedDateTimeOffsetBehavior, Cx::Error> {
    temporal_validate_options_object(cx, options)?;
    let default_text = match default {
        TemporalZonedDateTimeOffsetBehavior::Prefer => "prefer",
        TemporalZonedDateTimeOffsetBehavior::Use => "use",
        TemporalZonedDateTimeOffsetBehavior::Ignore => "ignore",
        TemporalZonedDateTimeOffsetBehavior::Reject => "reject",
    };
    if options.is_undefined() {
        return Ok(default);
    }
    let object_ref = options.as_object_ref().ok_or_else(|| type_error(cx))?;
    let value = temporal_property_value(cx, object_ref, "offset")?;
    match temporal_string_option(
        cx,
        value,
        &["prefer", "use", "ignore", "reject"],
        default_text,
    )?
    .as_str()
    {
        "prefer" => Ok(TemporalZonedDateTimeOffsetBehavior::Prefer),
        "use" => Ok(TemporalZonedDateTimeOffsetBehavior::Use),
        "ignore" => Ok(TemporalZonedDateTimeOffsetBehavior::Ignore),
        "reject" => Ok(TemporalZonedDateTimeOffsetBehavior::Reject),
        _ => unreachable!("temporal_string_option constrained offset"),
    }
}

fn temporal_zoned_date_time_from_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    options: Value,
) -> Result<TemporalZonedDateTimeFromOptions, Cx::Error> {
    Ok(TemporalZonedDateTimeFromOptions {
        disambiguation: temporal_disambiguation_from_options(cx, options)?,
        offset: temporal_zoned_date_time_offset_behavior_from_options(cx, options)?,
        overflow: temporal_overflow_from_options(cx, options)?,
    })
}

fn temporal_zoned_date_time_with_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    options: Value,
) -> Result<TemporalZonedDateTimeFromOptions, Cx::Error> {
    Ok(TemporalZonedDateTimeFromOptions {
        disambiguation: temporal_disambiguation_from_options(cx, options)?,
        offset: temporal_zoned_date_time_offset_behavior_from_options_with_default(
            cx,
            options,
            TemporalZonedDateTimeOffsetBehavior::Prefer,
        )?,
        overflow: temporal_overflow_from_options(cx, options)?,
    })
}

fn temporal_zoned_date_time_offset_text_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Option<String>, Cx::Error> {
    if value.is_undefined() {
        return Ok(None);
    }
    let offset_ref = if let Some(offset_ref) = value.as_string_ref() {
        offset_ref
    } else {
        if value.as_object_ref().is_none() {
            return Err(type_error(cx));
        }
        to_string_string_ref(cx, value)?
    };
    Ok(Some(string_ref_text(cx, offset_ref)?))
}

fn temporal_zoned_date_time_instant_from_explicit_offset<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    date_time: TemporalCivilDateTime,
    offset_nanoseconds: i64,
) -> Result<i128, Cx::Error> {
    let offset_time_zone = format_temporal_offset(offset_nanoseconds);
    let instant = cx.temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
        time_zone_id: offset_time_zone,
        date_time,
        disambiguation: TemporalDisambiguation::Compatible,
    })?;
    Ok(instant.epoch_nanoseconds)
}

fn temporal_zoned_date_time_civil_from_string<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    text: &str,
) -> Result<TemporalCivilDateTime, Cx::Error> {
    let parsed = parse_temporal_plain_date_time(text).ok_or_else(|| range_error(cx))?;
    let (millisecond, microsecond, nanosecond) =
        temporal_subsecond_parts_from_nanoseconds(cx, parsed.fraction_nanoseconds)?;
    let data = temporal_plain_date_time_from_parts(
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
    )?;
    Ok(TemporalCivilDateTime::new(
        data.year(),
        data.month(),
        data.day(),
        data.hour(),
        data.minute(),
        data.second(),
        data.millisecond(),
        data.microsecond(),
        data.nanosecond(),
    ))
}

fn temporal_zoned_date_time_civil_within_wall_clock_limits(
    date_time: TemporalCivilDateTime,
) -> bool {
    (
        date_time.year,
        date_time.month,
        date_time.day,
        date_time.hour,
        date_time.minute,
        date_time.second,
        date_time.millisecond,
        date_time.microsecond,
        date_time.nanosecond,
    ) >= (-271_821, 4, 20, 0, 0, 0, 0, 0, 0)
        && (
            date_time.year,
            date_time.month,
            date_time.day,
            date_time.hour,
            date_time.minute,
            date_time.second,
            date_time.millisecond,
            date_time.microsecond,
            date_time.nanosecond,
        ) <= (275_760, 9, 13, 23, 59, 59, 999, 999, 999)
}

pub(super) fn temporal_zoned_date_time_from_value_with_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    options: Value,
) -> Result<TemporalZonedDateTimeObjectData, Cx::Error> {
    if let Some(string_ref) = value.as_string_ref() {
        let text = string_ref_text(cx, string_ref)?;
        let mut time_zone_id =
            temporal_zoned_date_time_zone_annotation(&text).ok_or_else(|| range_error(cx))?;
        let explicit_offset = temporal_zoned_date_time_explicit_offset(&text);
        let parsed_instant = parse_temporal_instant(&text);
        let parsed_date_time = if parsed_instant.is_some() && explicit_offset.is_none() {
            None
        } else {
            Some(temporal_zoned_date_time_civil_from_string(cx, &text)?)
        };
        let options = temporal_zoned_date_time_from_options(cx, options)?;
        if let Some(explicit_offset) = explicit_offset {
            let actual_offset = if time_zone_id == TEMPORAL_UTC_TIME_ZONE_ID {
                Some(0)
            } else {
                temporal_parse_fixed_offset_time_zone_id(&time_zone_id)
            };
            if options.offset == TemporalZonedDateTimeOffsetBehavior::Reject
                && matches!(actual_offset, Some(actual_offset) if actual_offset != explicit_offset)
            {
                return Err(range_error(cx));
            }
            if actual_offset.is_none()
                && options.offset != TemporalZonedDateTimeOffsetBehavior::Ignore
            {
                time_zone_id = format_temporal_offset(explicit_offset);
            }
        }
        let epoch_nanoseconds = if explicit_offset.is_none() {
            if let Some(epoch_nanoseconds) = parsed_instant {
                epoch_nanoseconds
            } else {
                let date_time = parsed_date_time.expect("date-time parsed above");
                let instant =
                    cx.temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
                        time_zone_id: time_zone_id.clone(),
                        date_time,
                        disambiguation: options.disambiguation,
                    })?;
                instant.epoch_nanoseconds
            }
        } else if let Some(explicit_offset) = explicit_offset {
            let date_time = parsed_date_time.expect("explicit-offset strings parse civil fields");
            if matches!(
                options.offset,
                TemporalZonedDateTimeOffsetBehavior::Use
                    | TemporalZonedDateTimeOffsetBehavior::Prefer
            ) {
                if options.offset == TemporalZonedDateTimeOffsetBehavior::Prefer
                    && !temporal_zoned_date_time_civil_within_wall_clock_limits(date_time)
                {
                    return Err(range_error(cx));
                }
                temporal_zoned_date_time_instant_from_explicit_offset(
                    cx,
                    date_time,
                    explicit_offset,
                )?
            } else {
                if options.offset == TemporalZonedDateTimeOffsetBehavior::Reject
                    && !temporal_zoned_date_time_civil_within_wall_clock_limits(date_time)
                {
                    return Err(range_error(cx));
                }
                let instant =
                    cx.temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
                        time_zone_id: time_zone_id.clone(),
                        date_time,
                        disambiguation: options.disambiguation,
                    })?;
                instant.epoch_nanoseconds
            }
        } else if let Some(epoch_nanoseconds) = parsed_instant {
            epoch_nanoseconds
        } else {
            unreachable!("string path parsed either an instant or a plain date-time above")
        };
        return temporal_zoned_date_time_from_parts(cx, epoch_nanoseconds, &time_zone_id);
    }

    let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let existing = {
        let agent = cx.agent();
        agent.objects().temporal_object(object_ref).copied()
    };
    if let Some(TemporalObjectData::ZonedDateTime(data)) = existing {
        let _ = temporal_zoned_date_time_from_options(cx, options)?;
        return Ok(data);
    }
    temporal_validate_optional_iso_calendar_property(cx, object_ref)?;
    let day = temporal_optional_integer_part_from_property(cx, object_ref, "day")?;
    let hour = temporal_optional_time_part_from_property(cx, object_ref, "hour")?.unwrap_or(0);
    let microsecond =
        temporal_optional_time_part_from_property(cx, object_ref, "microsecond")?.unwrap_or(0);
    let millisecond =
        temporal_optional_time_part_from_property(cx, object_ref, "millisecond")?.unwrap_or(0);
    let minute = temporal_optional_time_part_from_property(cx, object_ref, "minute")?.unwrap_or(0);
    let month = temporal_optional_integer_part_from_property(cx, object_ref, "month")?;
    let month_code_text = temporal_optional_month_code_text_from_property(cx, object_ref)?;
    let nanosecond =
        temporal_optional_time_part_from_property(cx, object_ref, "nanosecond")?.unwrap_or(0);
    let offset = temporal_property_value(cx, object_ref, "offset")?;
    let offset_text = temporal_zoned_date_time_offset_text_from_value(cx, offset)?;
    let parsed_offset = if let Some(offset_text) = offset_text.as_ref() {
        Some(temporal_parse_offset_string(offset_text, true).ok_or_else(|| range_error(cx))?)
    } else {
        None
    };
    let second = temporal_optional_time_part_from_property(cx, object_ref, "second")?.unwrap_or(0);
    let time_zone = temporal_property_value(cx, object_ref, "timeZone")?;
    let year = temporal_optional_integer_part_from_property(cx, object_ref, "year")?;
    let options = temporal_zoned_date_time_from_options(cx, options)?;
    if time_zone.is_undefined() {
        return Err(type_error(cx));
    }
    let time_zone_id = temporal_time_zone_id_from_value(cx, time_zone)?;
    let year = year.ok_or_else(|| type_error(cx))?;
    let day = day.ok_or_else(|| type_error(cx))?;
    let month = temporal_resolve_month_from_fields(cx, month, month_code_text.as_deref(), None)?;
    let Ok(hour) = u8::try_from(hour) else {
        return Err(range_error(cx));
    };
    let Ok(minute) = u8::try_from(minute) else {
        return Err(range_error(cx));
    };
    let Ok(mut second) = u8::try_from(second) else {
        return Err(range_error(cx));
    };
    if second > 60 {
        return Err(range_error(cx));
    }
    second = second.min(59);
    let Ok(millisecond) = u16::try_from(millisecond) else {
        return Err(range_error(cx));
    };
    let Ok(microsecond) = u16::try_from(microsecond) else {
        return Err(range_error(cx));
    };
    let Ok(nanosecond) = u16::try_from(nanosecond) else {
        return Err(range_error(cx));
    };
    let date_time_data = temporal_plain_date_time_from_parts_with_overflow(
        cx,
        year,
        month,
        day,
        i64::from(hour),
        i64::from(minute),
        i64::from(second),
        i64::from(millisecond),
        i64::from(microsecond),
        i64::from(nanosecond),
        options.overflow,
    );
    let date_time_data = date_time_data?;
    let date_time = TemporalCivilDateTime::new(
        date_time_data.year(),
        date_time_data.month(),
        date_time_data.day(),
        date_time_data.hour(),
        date_time_data.minute(),
        date_time_data.second(),
        date_time_data.millisecond(),
        date_time_data.microsecond(),
        date_time_data.nanosecond(),
    );
    let instant = cx.temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
        time_zone_id: time_zone_id.clone(),
        date_time,
        disambiguation: options.disambiguation,
    })?;
    let epoch_nanoseconds = match (parsed_offset, options.offset) {
        (Some(offset), TemporalZonedDateTimeOffsetBehavior::Reject)
            if offset != instant.offset_nanoseconds =>
        {
            return Err(range_error(cx));
        }
        (
            Some(offset),
            TemporalZonedDateTimeOffsetBehavior::Use | TemporalZonedDateTimeOffsetBehavior::Prefer,
        ) => temporal_zoned_date_time_instant_from_explicit_offset(cx, date_time, offset)?,
        _ => instant.epoch_nanoseconds,
    };
    temporal_zoned_date_time_from_parts(cx, epoch_nanoseconds, &time_zone_id)
}

pub(super) fn temporal_zoned_date_time_explicit_offset(text: &str) -> Option<i64> {
    let prefix = text.split_once('[').map_or(text, |(prefix, _)| prefix);
    let bytes = prefix.as_bytes();
    let time_separator = bytes
        .iter()
        .position(|byte| matches!(byte, b'T' | b't' | b' '))?;
    let offset_start = bytes
        .iter()
        .enumerate()
        .skip(time_separator + 1)
        .rev()
        .find_map(|(index, byte)| matches!(byte, b'+' | b'-').then_some(index))?;
    parse_temporal_plain_date_time(&prefix[..offset_start])?;
    temporal_parse_offset_string(&prefix[offset_start..], true)
}

pub(super) fn temporal_zoned_date_time_zone_annotation(text: &str) -> Option<String> {
    let mut remaining = text;
    while let Some(start) = remaining.find('[') {
        let after_start = &remaining[start + 1..];
        let end = after_start.find(']')?;
        let body = after_start[..end].trim_start_matches('!');
        if body.eq_ignore_ascii_case(TEMPORAL_UTC_TIME_ZONE_ID) {
            return Some(TEMPORAL_UTC_TIME_ZONE_ID.to_string());
        }
        if body.contains('\u{2212}') {
            return None;
        }
        if let Some(offset_nanoseconds) = temporal_parse_fixed_offset_time_zone_id(body) {
            return Some(format_temporal_offset(offset_nanoseconds));
        }
        if matches!(body.as_bytes().first(), Some(b'+' | b'-')) {
            return None;
        }
        if !body.is_empty() && !body.contains('=') {
            return Some(body.to_string());
        }
        remaining = &after_start[end + 1..];
    }
    None
}

pub(super) fn temporal_zoned_date_time_data<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    this_value: Value,
) -> Result<TemporalZonedDateTimeObjectData, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, this_value, TemporalObjectKind::ZonedDateTime)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::ZonedDateTime(data) = payload else {
        return Err(type_error(cx));
    };
    Ok(data)
}

pub(super) fn temporal_zoned_date_time_civil<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    data: TemporalZonedDateTimeObjectData,
) -> Result<TemporalCivilTime, Cx::Error> {
    let time_zone_id = temporal_atom_text(cx, data.time_zone())?;
    cx.temporal_instant_to_civil_time(&TemporalInstantToCivilRequest {
        time_zone_id,
        epoch_nanoseconds: data.epoch_nanoseconds(),
    })
}

pub(super) fn format_temporal_zoned_date_time<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    data: TemporalZonedDateTimeObjectData,
) -> Result<String, Cx::Error> {
    format_temporal_zoned_date_time_with_options(
        cx,
        data,
        TemporalZonedDateTimeToStringOptions::default(),
    )
}

#[derive(Clone, Copy)]
pub(super) enum TemporalZonedDateTimeOffsetOption {
    Auto,
    Never,
}

#[derive(Clone, Copy)]
pub(super) enum TemporalZonedDateTimeTimeZoneNameOption {
    Auto,
    Never,
    Critical,
}

#[derive(Clone, Copy)]
pub(super) enum TemporalZonedDateTimeCalendarNameOption {
    Auto,
    Always,
    Never,
    Critical,
}

#[derive(Clone, Copy)]
pub(super) struct TemporalZonedDateTimeToStringOptions {
    precision: TemporalInstantStringPrecision,
    rounding_mode: TemporalBuiltinRoundingMode,
    offset: TemporalZonedDateTimeOffsetOption,
    time_zone_name: TemporalZonedDateTimeTimeZoneNameOption,
    calendar_name: TemporalZonedDateTimeCalendarNameOption,
}

impl Default for TemporalZonedDateTimeToStringOptions {
    fn default() -> Self {
        Self {
            precision: TemporalInstantStringPrecision::Auto,
            rounding_mode: TemporalBuiltinRoundingMode::Trunc,
            offset: TemporalZonedDateTimeOffsetOption::Auto,
            time_zone_name: TemporalZonedDateTimeTimeZoneNameOption::Auto,
            calendar_name: TemporalZonedDateTimeCalendarNameOption::Auto,
        }
    }
}

pub(super) fn temporal_zoned_date_time_to_string_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalZonedDateTimeToStringOptions, Cx::Error> {
    if value.is_undefined() {
        return Ok(TemporalZonedDateTimeToStringOptions::default());
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
    let offset_value = temporal_property_value(cx, object_ref, "offset")?;
    let offset =
        match temporal_string_option(cx, offset_value, &["auto", "never"], "auto")?.as_str() {
            "auto" => TemporalZonedDateTimeOffsetOption::Auto,
            "never" => TemporalZonedDateTimeOffsetOption::Never,
            _ => unreachable!("temporal_string_option constrained offset"),
        };
    let rounding_mode = temporal_property_value(cx, object_ref, "roundingMode")?;
    let rounding_mode = temporal_duration_rounding_mode_option(cx, rounding_mode)?;
    let smallest_unit = temporal_property_value(cx, object_ref, "smallestUnit")?;
    let smallest_unit_text = if smallest_unit.is_undefined() {
        None
    } else {
        let string_ref = to_string_string_ref(cx, smallest_unit)?;
        Some(string_ref_text(cx, string_ref)?)
    };
    let time_zone_name_value = temporal_property_value(cx, object_ref, "timeZoneName")?;
    let time_zone_name = match temporal_string_option(
        cx,
        time_zone_name_value,
        &["auto", "never", "critical"],
        "auto",
    )?
    .as_str()
    {
        "auto" => TemporalZonedDateTimeTimeZoneNameOption::Auto,
        "never" => TemporalZonedDateTimeTimeZoneNameOption::Never,
        "critical" => TemporalZonedDateTimeTimeZoneNameOption::Critical,
        _ => unreachable!("temporal_string_option constrained timeZoneName"),
    };
    let precision = if let Some(text) = smallest_unit_text {
        temporal_instant_smallest_unit_precision_from_text(cx, &text)?
    } else {
        fractional_second_precision
    };
    Ok(TemporalZonedDateTimeToStringOptions {
        precision,
        rounding_mode,
        offset,
        time_zone_name,
        calendar_name,
    })
}

pub(super) fn temporal_zoned_date_time_epoch_for_string_precision<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    epoch_nanoseconds: i128,
    options: TemporalZonedDateTimeToStringOptions,
) -> Result<i128, Cx::Error> {
    let rounded = match options.precision {
        TemporalInstantStringPrecision::Auto => epoch_nanoseconds,
        TemporalInstantStringPrecision::Minute => temporal_round_epoch_nanoseconds_to_increment(
            epoch_nanoseconds,
            TEMPORAL_NANOS_PER_MINUTE,
            options.rounding_mode,
        )
        .ok_or_else(|| range_error(cx))?,
        TemporalInstantStringPrecision::FractionalSecond(digits) => {
            temporal_round_epoch_nanoseconds_to_fractional_digits(
                epoch_nanoseconds,
                digits,
                options.rounding_mode,
            )
            .ok_or_else(|| range_error(cx))?
        }
    };
    if !temporal_instant_epoch_nanoseconds_is_valid(rounded) {
        return Err(range_error(cx));
    }
    Ok(rounded)
}

pub(super) fn format_temporal_zoned_date_time_with_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    data: TemporalZonedDateTimeObjectData,
    options: TemporalZonedDateTimeToStringOptions,
) -> Result<String, Cx::Error> {
    let epoch_nanoseconds =
        temporal_zoned_date_time_epoch_for_string_precision(cx, data.epoch_nanoseconds(), options)?;
    let time_zone_id = temporal_atom_text(cx, data.time_zone())?;
    let civil = cx.temporal_instant_to_civil_time(&TemporalInstantToCivilRequest {
        time_zone_id: time_zone_id.clone(),
        epoch_nanoseconds,
    })?;
    let date_time = civil.date_time;
    let mut text = format_temporal_civil_date_time_with_precision(
        date_time,
        data.calendar(),
        options.precision,
    );
    if matches!(options.offset, TemporalZonedDateTimeOffsetOption::Auto) {
        text.push_str(&format_temporal_offset(civil.offset_nanoseconds));
    }
    match options.time_zone_name {
        TemporalZonedDateTimeTimeZoneNameOption::Auto => {
            let _ = write!(&mut text, "[{time_zone_id}]");
        }
        TemporalZonedDateTimeTimeZoneNameOption::Critical => {
            let _ = write!(&mut text, "[!{time_zone_id}]");
        }
        TemporalZonedDateTimeTimeZoneNameOption::Never => {}
    }
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
    Ok(text)
}

pub(super) fn temporal_zoned_date_time_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let Some(new_target) = invocation.new_target() else {
        return Err(type_error(cx));
    };
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
    let time_zone_id = temporal_time_zone_id_from_constructor_value(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    temporal_validate_optional_iso_calendar_identifier_argument(cx, invocation, 2)?;
    let data = temporal_zoned_date_time_from_parts(cx, epoch_nanoseconds, &time_zone_id)?;
    let prototype = temporal_constructor_prototype(cx, new_target)?;
    allocate_temporal_zoned_date_time_object(cx, prototype, data)
}

pub(super) fn temporal_zoned_date_time_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(civil.date_time.year))
}

pub(super) fn temporal_zoned_date_time_month_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(i32::from(civil.date_time.month)))
}

pub(super) fn temporal_zoned_date_time_month_code_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(string_value(cx, &format!("M{:02}", civil.date_time.month)))
}

pub(super) fn temporal_zoned_date_time_day_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(i32::from(civil.date_time.day)))
}

pub(super) fn temporal_zoned_date_time_day_of_week_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    let date_time = civil.date_time;
    Ok(Value::from_smi(temporal_iso_day_of_week(
        date_time.year,
        date_time.month,
        date_time.day,
    )))
}

pub(super) fn temporal_zoned_date_time_day_of_year_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    let date_time = civil.date_time;
    Ok(Value::from_smi(temporal_iso_day_of_year(
        date_time.year,
        date_time.month,
        date_time.day,
    )))
}

pub(super) fn temporal_zoned_date_time_days_in_month_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    let date_time = civil.date_time;
    Ok(Value::from_smi(i32::from(temporal_iso_days_in_month(
        date_time.year,
        date_time.month,
    ))))
}

pub(super) fn temporal_zoned_date_time_days_in_year_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(temporal_iso_days_in_year(
        civil.date_time.year,
    )))
}

pub(super) fn temporal_zoned_date_time_months_in_year_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(12))
}

pub(super) fn temporal_zoned_date_time_in_leap_year_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_bool(temporal_is_iso_leap_year(
        civil.date_time.year,
    )))
}

pub(super) fn temporal_zoned_date_time_days_in_week_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(7))
}

pub(super) fn temporal_zoned_date_time_week_of_year_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    let date_time = civil.date_time;
    let (week, _) = temporal_iso_week_of_year(date_time.year, date_time.month, date_time.day);
    Ok(Value::from_smi(week))
}

pub(super) fn temporal_zoned_date_time_year_of_week_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    let date_time = civil.date_time;
    let (_, year) = temporal_iso_week_of_year(date_time.year, date_time.month, date_time.day);
    Ok(Value::from_smi(year))
}

pub(super) fn temporal_zoned_date_time_era_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    Ok(Value::undefined())
}

pub(super) fn temporal_zoned_date_time_era_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    Ok(Value::undefined())
}

pub(super) fn temporal_zoned_date_time_hour_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(i32::from(civil.date_time.hour)))
}

pub(super) fn temporal_zoned_date_time_minute_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(i32::from(civil.date_time.minute)))
}

pub(super) fn temporal_zoned_date_time_second_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(i32::from(civil.date_time.second)))
}

pub(super) fn temporal_zoned_date_time_millisecond_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(i32::from(civil.date_time.millisecond)))
}

pub(super) fn temporal_zoned_date_time_microsecond_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(i32::from(civil.date_time.microsecond)))
}

pub(super) fn temporal_zoned_date_time_nanosecond_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(i32::from(civil.date_time.nanosecond)))
}

pub(super) fn temporal_zoned_date_time_epoch_nanoseconds_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    Ok(temporal_i128_to_bigint_value(
        cx.agent(),
        data.epoch_nanoseconds(),
    ))
}

pub(super) fn temporal_zoned_date_time_epoch_milliseconds_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    temporal_safe_integer_number(
        cx,
        data.epoch_nanoseconds()
            .div_euclid(TEMPORAL_NANOS_PER_MILLISECOND),
    )
}

pub(super) fn temporal_zoned_date_time_time_zone_id_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let time_zone_id = temporal_atom_text(cx, data.time_zone())?;
    Ok(string_value(cx, &time_zone_id))
}

pub(super) fn temporal_zoned_date_time_calendar_id_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    Ok(string_value(cx, "iso8601"))
}

pub(super) fn temporal_zoned_date_time_offset_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(string_value(
        cx,
        &format_temporal_offset(civil.offset_nanoseconds),
    ))
}

pub(super) fn temporal_zoned_date_time_offset_nanoseconds_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_f64(civil.offset_nanoseconds as f64))
}

pub(super) fn temporal_zoned_date_time_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let options = temporal_zoned_date_time_to_string_options(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let text = format_temporal_zoned_date_time_with_options(cx, data, options)?;
    Ok(string_value(cx, &text))
}

pub(super) fn temporal_zoned_date_time_to_json_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let text = format_temporal_zoned_date_time(cx, data)?;
    Ok(string_value(cx, &text))
}

pub(super) fn temporal_zoned_date_time_to_locale_string_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_zoned_date_time_to_json_builtin(cx, invocation)
}

pub(super) fn temporal_zoned_date_time_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Err(type_error(cx))
}

pub(super) fn temporal_zoned_date_time_equals_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let right = temporal_zoned_date_time_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(left == right))
}

pub(super) fn temporal_civil_date_time_from_plain_date_time(
    date_time: TemporalPlainDateTimeObjectData,
) -> TemporalCivilDateTime {
    TemporalCivilDateTime::new(
        date_time.year(),
        date_time.month(),
        date_time.day(),
        date_time.hour(),
        date_time.minute(),
        date_time.second(),
        date_time.millisecond(),
        date_time.microsecond(),
        date_time.nanosecond(),
    )
}

pub(super) fn temporal_zoned_date_time_add_duration<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    zoned: TemporalZonedDateTimeObjectData,
    duration: TemporalDurationObjectData,
    overflow: TemporalOverflow,
) -> Result<Value, Cx::Error> {
    let civil = temporal_zoned_date_time_civil(cx, zoned)?.date_time;
    let plain_date_time = temporal_plain_date_time_from_parts(
        cx,
        i64::from(civil.year),
        i64::from(civil.month),
        i64::from(civil.day),
        i64::from(civil.hour),
        i64::from(civil.minute),
        i64::from(civil.second),
        i64::from(civil.millisecond),
        i64::from(civil.microsecond),
        i64::from(civil.nanosecond),
    )?;
    let added = temporal_plain_date_time_add_duration(cx, plain_date_time, duration, overflow)?;
    let time_zone_id = temporal_atom_text(cx, zoned.time_zone())?;
    let instant = cx.temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
        time_zone_id: time_zone_id.clone(),
        date_time: temporal_civil_date_time_from_plain_date_time(added),
        disambiguation: TemporalDisambiguation::Compatible,
    })?;
    let data = temporal_zoned_date_time_from_parts(cx, instant.epoch_nanoseconds, &time_zone_id)?;
    let prototype = current_temporal_zoned_date_time_prototype(cx)?;
    allocate_temporal_zoned_date_time_object(cx, prototype, data)
}

pub(super) fn temporal_zoned_date_time_add_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let zoned = temporal_zoned_date_time_data(cx, invocation.this_value())?;
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
    temporal_zoned_date_time_add_duration(cx, zoned, duration, overflow)
}

pub(super) fn temporal_zoned_date_time_subtract_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let zoned = temporal_zoned_date_time_data(cx, invocation.this_value())?;
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
    temporal_zoned_date_time_add_duration(cx, zoned, negate_temporal_duration(duration), overflow)
}

pub(super) fn temporal_zoned_date_time_round_to_day<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    zoned: TemporalZonedDateTimeObjectData,
    rounding_mode: TemporalBuiltinRoundingMode,
) -> Result<i128, Cx::Error> {
    let civil = temporal_zoned_date_time_civil(cx, zoned)?.date_time;
    let time_zone_id = temporal_atom_text(cx, zoned.time_zone())?;
    let start = temporal_zoned_date_time_midnight_epoch_nanoseconds(
        cx,
        &time_zone_id,
        civil.year,
        civil.month,
        civil.day,
    )?;
    let date = temporal_plain_date_from_parts(
        cx,
        i64::from(civil.year),
        i64::from(civil.month),
        i64::from(civil.day),
    )?;
    let next_date = temporal_plain_date_add_duration(
        cx,
        date,
        TemporalDurationObjectData::new(0, 0, 0, 1, 0, 0, 0, 0, 0, 0),
        TemporalOverflow::Constrain,
    )?;
    let end = temporal_zoned_date_time_midnight_epoch_nanoseconds(
        cx,
        &time_zone_id,
        next_date.year(),
        next_date.month(),
        next_date.day(),
    )?;
    let day_length = end.checked_sub(start).ok_or_else(|| range_error(cx))?;
    let elapsed = zoned
        .epoch_nanoseconds()
        .checked_sub(start)
        .ok_or_else(|| range_error(cx))?;
    let rounded = temporal_round_epoch_nanoseconds_to_increment(elapsed, day_length, rounding_mode)
        .ok_or_else(|| range_error(cx))?;
    if rounded == 0 {
        Ok(start)
    } else if rounded == day_length {
        Ok(end)
    } else {
        Err(range_error(cx))
    }
}

pub(super) fn temporal_zoned_date_time_round_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let zoned = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let options = temporal_plain_date_time_round_options(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let epoch_nanoseconds = if options.smallest_unit == TemporalDateTimeDifferenceUnit::Day {
        if options.rounding_increment != 1 {
            return Err(range_error(cx));
        }
        temporal_zoned_date_time_round_to_day(cx, zoned, options.rounding_mode)?
    } else {
        let unit_nanoseconds =
            temporal_date_time_difference_unit_nanoseconds(options.smallest_unit)
                .ok_or_else(|| range_error(cx))?;
        let increment = unit_nanoseconds
            .checked_mul(options.rounding_increment)
            .ok_or_else(|| range_error(cx))?;
        let civil = temporal_zoned_date_time_civil(cx, zoned)?.date_time;
        let date_time = temporal_plain_date_time_from_parts(
            cx,
            i64::from(civil.year),
            i64::from(civil.month),
            i64::from(civil.day),
            i64::from(civil.hour),
            i64::from(civil.minute),
            i64::from(civil.second),
            i64::from(civil.millisecond),
            i64::from(civil.microsecond),
            i64::from(civil.nanosecond),
        )?;
        let rounded = temporal_round_epoch_nanoseconds_to_increment(
            temporal_plain_date_time_total_nanoseconds(date_time).ok_or_else(|| range_error(cx))?,
            increment,
            options.rounding_mode,
        )
        .ok_or_else(|| range_error(cx))?;
        let rounded = temporal_plain_date_time_from_total_nanoseconds(cx, rounded)?;
        let time_zone_id = temporal_atom_text(cx, zoned.time_zone())?;
        let instant = cx.temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
            time_zone_id,
            date_time: temporal_civil_date_time_from_plain_date_time(rounded),
            disambiguation: TemporalDisambiguation::Compatible,
        })?;
        instant.epoch_nanoseconds
    };
    if !temporal_instant_epoch_nanoseconds_is_valid(epoch_nanoseconds) {
        return Err(range_error(cx));
    }
    let data = TemporalZonedDateTimeObjectData::new(
        epoch_nanoseconds,
        zoned.time_zone(),
        zoned.calendar(),
    );
    let prototype = current_temporal_zoned_date_time_prototype(cx)?;
    allocate_temporal_zoned_date_time_object(cx, prototype, data)
}

pub(super) fn temporal_zoned_date_time_with_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let zoned = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let object_ref = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined())
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let temporal = {
        let agent = cx.agent();
        agent.objects().temporal_object(object_ref).copied()
    };
    if temporal.is_some() {
        return Err(type_error(cx));
    }
    let calendar = temporal_property_value(cx, object_ref, "calendar")?;
    if !calendar.is_undefined() {
        return Err(type_error(cx));
    }
    let time_zone = temporal_property_value(cx, object_ref, "timeZone")?;
    if !time_zone.is_undefined() {
        return Err(type_error(cx));
    }
    let civil = temporal_zoned_date_time_civil(cx, zoned)?.date_time;
    let day = temporal_optional_integer_part_from_property(cx, object_ref, "day")?;
    let hour = temporal_optional_time_part_from_property(cx, object_ref, "hour")?;
    let microsecond = temporal_optional_time_part_from_property(cx, object_ref, "microsecond")?;
    let millisecond = temporal_optional_time_part_from_property(cx, object_ref, "millisecond")?;
    let minute = temporal_optional_time_part_from_property(cx, object_ref, "minute")?;
    let month_value = temporal_optional_integer_part_from_property(cx, object_ref, "month")?;
    let month_code_text = temporal_optional_string_text_from_property(cx, object_ref, "monthCode")?;
    let nanosecond = temporal_optional_time_part_from_property(cx, object_ref, "nanosecond")?;
    let offset = temporal_property_value(cx, object_ref, "offset")?;
    let offset_text = temporal_zoned_date_time_offset_text_from_value(cx, offset)?;
    let second = temporal_optional_time_part_from_property(cx, object_ref, "second")?;
    let year = temporal_optional_integer_part_from_property(cx, object_ref, "year")?;
    if day.is_some_and(|day| day < 1) || month_value.is_some_and(|month| month < 1) {
        return Err(range_error(cx));
    }
    let options = temporal_zoned_date_time_with_options(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if day.is_none()
        && hour.is_none()
        && microsecond.is_none()
        && millisecond.is_none()
        && minute.is_none()
        && month_value.is_none()
        && month_code_text.is_none()
        && nanosecond.is_none()
        && offset_text.is_none()
        && second.is_none()
        && year.is_none()
    {
        return Err(type_error(cx));
    }
    let parsed_offset = if let Some(offset_text) = offset_text.as_ref() {
        Some(temporal_parse_offset_string(offset_text, true).ok_or_else(|| range_error(cx))?)
    } else {
        None
    };
    let year = year.unwrap_or(i64::from(civil.year));
    let month = temporal_resolve_month_from_fields(
        cx,
        month_value,
        month_code_text.as_deref(),
        Some(i64::from(civil.month)),
    )?;
    let day = day.unwrap_or(i64::from(civil.day));
    let hour = hour.unwrap_or(i64::from(civil.hour));
    let minute = minute.unwrap_or(i64::from(civil.minute));
    let second = second.unwrap_or(i64::from(civil.second));
    let millisecond = millisecond.unwrap_or(i64::from(civil.millisecond));
    let microsecond = microsecond.unwrap_or(i64::from(civil.microsecond));
    let nanosecond = nanosecond.unwrap_or(i64::from(civil.nanosecond));
    let date_time = temporal_plain_date_time_from_parts_with_overflow(
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
        options.overflow,
    )?;
    let time_zone_id = temporal_atom_text(cx, zoned.time_zone())?;
    let date_time = temporal_civil_date_time_from_plain_date_time(date_time);
    let instant = cx.temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
        time_zone_id: time_zone_id.clone(),
        date_time,
        disambiguation: options.disambiguation,
    })?;
    let epoch_nanoseconds = match (parsed_offset, options.offset) {
        (Some(offset), TemporalZonedDateTimeOffsetBehavior::Reject)
            if offset != instant.offset_nanoseconds =>
        {
            return Err(range_error(cx));
        }
        (
            Some(offset),
            TemporalZonedDateTimeOffsetBehavior::Reject
            | TemporalZonedDateTimeOffsetBehavior::Prefer,
        ) if !temporal_zoned_date_time_civil_within_wall_clock_limits(date_time) => {
            return Err(range_error(cx));
        }
        (Some(offset), TemporalZonedDateTimeOffsetBehavior::Use) => {
            temporal_zoned_date_time_instant_from_explicit_offset(cx, date_time, offset)?
        }
        (Some(offset), TemporalZonedDateTimeOffsetBehavior::Prefer)
            if offset == instant.offset_nanoseconds =>
        {
            temporal_zoned_date_time_instant_from_explicit_offset(cx, date_time, offset)?
        }
        _ => instant.epoch_nanoseconds,
    };
    if !temporal_instant_epoch_nanoseconds_is_valid(epoch_nanoseconds) {
        return Err(range_error(cx));
    }
    let data = TemporalZonedDateTimeObjectData::new(
        epoch_nanoseconds,
        zoned.time_zone(),
        zoned.calendar(),
    );
    let prototype = current_temporal_zoned_date_time_prototype(cx)?;
    allocate_temporal_zoned_date_time_object(cx, prototype, data)
}

pub(super) fn temporal_zoned_date_time_with_time_zone_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let zoned = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let time_zone_id = temporal_time_zone_id_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let data = temporal_zoned_date_time_from_parts(cx, zoned.epoch_nanoseconds(), &time_zone_id)?;
    let prototype = current_temporal_zoned_date_time_prototype(cx)?;
    allocate_temporal_zoned_date_time_object(cx, prototype, data)
}

pub(super) fn temporal_zoned_date_time_with_calendar_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let zoned = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    temporal_validate_iso_calendar_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let time_zone_id = temporal_atom_text(cx, zoned.time_zone())?;
    let data = temporal_zoned_date_time_from_parts(cx, zoned.epoch_nanoseconds(), &time_zone_id)?;
    let prototype = current_temporal_zoned_date_time_prototype(cx)?;
    allocate_temporal_zoned_date_time_object(cx, prototype, data)
}

pub(super) fn temporal_zoned_date_time_with_plain_time_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let zoned = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let replacement = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let time = if replacement.is_undefined() {
        TemporalPlainTimeObjectData::new(0, 0, 0, 0, 0, 0)
    } else {
        temporal_plain_time_from_value(cx, replacement)?
    };
    let civil = temporal_zoned_date_time_civil(cx, zoned)?.date_time;
    let date_time = TemporalCivilDateTime::new(
        civil.year,
        civil.month,
        civil.day,
        time.hour(),
        time.minute(),
        time.second(),
        time.millisecond(),
        time.microsecond(),
        time.nanosecond(),
    );
    let time_zone_id = temporal_atom_text(cx, zoned.time_zone())?;
    let instant = cx.temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
        time_zone_id: time_zone_id.clone(),
        date_time,
        disambiguation: TemporalDisambiguation::Compatible,
    })?;
    let data = temporal_zoned_date_time_from_parts(cx, instant.epoch_nanoseconds, &time_zone_id)?;
    let prototype = current_temporal_zoned_date_time_prototype(cx)?;
    allocate_temporal_zoned_date_time_object(cx, prototype, data)
}

pub(super) fn temporal_zoned_date_time_midnight_epoch_nanoseconds<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    time_zone_id: &str,
    year: i32,
    month: u8,
    day: u8,
) -> Result<i128, Cx::Error> {
    let instant = cx.temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
        time_zone_id: time_zone_id.to_string(),
        date_time: TemporalCivilDateTime::new(year, month, day, 0, 0, 0, 0, 0, 0),
        disambiguation: TemporalDisambiguation::Compatible,
    })?;
    if !temporal_instant_epoch_nanoseconds_is_valid(instant.epoch_nanoseconds) {
        return Err(range_error(cx));
    }
    Ok(instant.epoch_nanoseconds)
}

pub(super) fn temporal_zoned_date_time_start_of_day_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let zoned = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, zoned)?.date_time;
    let time_zone_id = temporal_atom_text(cx, zoned.time_zone())?;
    let epoch_nanoseconds = temporal_zoned_date_time_midnight_epoch_nanoseconds(
        cx,
        &time_zone_id,
        civil.year,
        civil.month,
        civil.day,
    )?;
    let data = temporal_zoned_date_time_from_parts(cx, epoch_nanoseconds, &time_zone_id)?;
    let prototype = current_temporal_zoned_date_time_prototype(cx)?;
    allocate_temporal_zoned_date_time_object(cx, prototype, data)
}

fn temporal_validate_time_zone_transition_direction<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    direction_param: Value,
) -> Result<(), Cx::Error> {
    if direction_param.is_undefined() {
        return Err(type_error(cx));
    }
    let direction_value = if direction_param.is_string() {
        direction_param
    } else {
        let object_ref = direction_param
            .as_object_ref()
            .ok_or_else(|| type_error(cx))?;
        temporal_property_value(cx, object_ref, "direction")?
    };
    if direction_value.is_undefined() {
        return Err(range_error(cx));
    }
    let string_ref = to_string_string_ref(cx, direction_value)?;
    match string_ref_text(cx, string_ref)?.as_str() {
        "next" | "previous" => Ok(()),
        _ => Err(range_error(cx)),
    }
}

pub(super) fn temporal_zoned_date_time_get_time_zone_transition_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let zoned = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    temporal_validate_time_zone_transition_direction(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;

    let time_zone_id = temporal_atom_text(cx, zoned.time_zone())?;
    if time_zone_id == TEMPORAL_UTC_TIME_ZONE_ID
        || temporal_parse_fixed_offset_time_zone_id(&time_zone_id).is_some()
    {
        return Ok(Value::null());
    }

    Err(type_error(cx))
}

pub(super) fn temporal_zoned_date_time_hours_in_day_getter_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let zoned = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, zoned)?.date_time;
    let time_zone_id = temporal_atom_text(cx, zoned.time_zone())?;
    let start = temporal_zoned_date_time_midnight_epoch_nanoseconds(
        cx,
        &time_zone_id,
        civil.year,
        civil.month,
        civil.day,
    )?;
    let date = temporal_plain_date_from_parts(
        cx,
        i64::from(civil.year),
        i64::from(civil.month),
        i64::from(civil.day),
    )?;
    let next_date = temporal_plain_date_add_duration(
        cx,
        date,
        TemporalDurationObjectData::new(0, 0, 0, 1, 0, 0, 0, 0, 0, 0),
        TemporalOverflow::Constrain,
    )?;
    let next = temporal_zoned_date_time_midnight_epoch_nanoseconds(
        cx,
        &time_zone_id,
        next_date.year(),
        next_date.month(),
        next_date.day(),
    )?;
    Ok(Value::from_f64(
        (next - start) as f64 / TEMPORAL_NANOS_PER_HOUR as f64,
    ))
}

pub(super) fn temporal_zoned_date_time_difference_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    sign: i128,
) -> Result<Value, Cx::Error> {
    let zoned = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let other = temporal_zoned_date_time_from_value(
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
        TemporalDateTimeDifferenceUnit::Hour,
    )?;
    let raw_difference = zoned
        .epoch_nanoseconds()
        .checked_sub(other.epoch_nanoseconds())
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

pub(super) fn temporal_zoned_date_time_since_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_zoned_date_time_difference_builtin(cx, invocation, 1)
}

pub(super) fn temporal_zoned_date_time_until_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_zoned_date_time_difference_builtin(cx, invocation, -1)
}

pub(super) fn temporal_zoned_date_time_from_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_from_value_with_options(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let prototype = current_temporal_zoned_date_time_prototype(cx)?;
    allocate_temporal_zoned_date_time_object(cx, prototype, data)
}

pub(super) fn temporal_zoned_date_time_compare_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = temporal_zoned_date_time_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let right = temporal_zoned_date_time_from_value(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(temporal_compare_ordering(
        left.epoch_nanoseconds().cmp(&right.epoch_nanoseconds()),
    ))
}

pub(super) fn temporal_zoned_date_time_to_instant_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let prototype = current_temporal_instant_prototype(cx)?;
    instant::allocate_temporal_instant_object(cx, prototype, data.epoch_nanoseconds())
}

pub(super) fn temporal_zoned_date_time_to_plain_date_time_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    let date_time = civil.date_time;
    let data = temporal_plain_date_time_from_parts(
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
    )?;
    let prototype = current_temporal_plain_date_time_prototype(cx)?;
    allocate_temporal_plain_date_time_object(cx, prototype, data)
}

pub(super) fn temporal_zoned_date_time_to_plain_date_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    let date_time = civil.date_time;
    let data = temporal_plain_date_from_parts(
        cx,
        i64::from(date_time.year),
        i64::from(date_time.month),
        i64::from(date_time.day),
    )?;
    let prototype = current_temporal_plain_date_prototype(cx)?;
    allocate_temporal_plain_date_object(cx, prototype, data)
}

pub(super) fn temporal_zoned_date_time_to_plain_time_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    let date_time = civil.date_time;
    let data = TemporalPlainTimeObjectData::new(
        date_time.hour,
        date_time.minute,
        date_time.second,
        date_time.millisecond,
        date_time.microsecond,
        date_time.nanosecond,
    );
    let prototype = current_temporal_plain_time_prototype(cx)?;
    plain_time::allocate_temporal_plain_time_object(cx, prototype, data)
}

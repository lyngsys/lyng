use super::{
    range_error, string_ref_text, temporal_duration_largest_unit_option,
    temporal_duration_parsed_unit, temporal_duration_rounding_increment_option,
    temporal_integer_part_from_value, temporal_month_from_month_code_value,
    temporal_number_to_u8_after_range_check, temporal_ops, temporal_parse_offset_string,
    temporal_plain_date_from_parts, temporal_plain_date_from_value,
    temporal_plain_date_time_from_parts_with_overflow, temporal_plain_date_time_from_value,
    temporal_property_value, temporal_time_part_from_value, temporal_time_zone_id_from_value,
    temporal_validate_iso_calendar_value, temporal_zoned_date_time_explicit_offset,
    temporal_zoned_date_time_from_parts, temporal_zoned_date_time_from_value,
    temporal_zoned_date_time_zone_annotation, to_string_string_ref, type_error, ObjectRef,
    PublicBuiltinDispatchContext, TemporalBuiltinDurationExactUnit, TemporalBuiltinRoundingMode,
    TemporalCivilDateTime, TemporalCivilToInstantRequest, TemporalDateTimeDifferenceUnit,
    TemporalDisambiguation, TemporalObjectData, TemporalOverflow, TemporalPlainDateObjectData,
    TemporalPlainDateTimeObjectData, TemporalZonedDateTimeObjectData, Value,
    TEMPORAL_NANOS_PER_DAY, TEMPORAL_NANOS_PER_HOUR, TEMPORAL_NANOS_PER_MICROSECOND,
    TEMPORAL_NANOS_PER_MILLISECOND, TEMPORAL_NANOS_PER_MINUTE, TEMPORAL_NANOS_PER_SECOND,
};

pub(in crate::public::dispatch::temporal) fn temporal_duration_to_string_options<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
) -> Result<(Option<u8>, TemporalBuiltinRoundingMode), Cx::Error> {
    if value.is_undefined() {
        return Ok((None, TemporalBuiltinRoundingMode::Trunc));
    }
    let Some(object_ref) = value.as_object_ref() else {
        return Err(type_error(cx));
    };

    let fractional_second_digits =
        temporal_property_value(cx, object_ref, "fractionalSecondDigits")?;
    let fractional_digits =
        temporal_duration_fractional_second_digits_option(cx, fractional_second_digits)?;
    let rounding_mode = temporal_property_value(cx, object_ref, "roundingMode")?;
    let rounding_mode = temporal_duration_rounding_mode_option(cx, rounding_mode)?;
    let smallest_unit = temporal_property_value(cx, object_ref, "smallestUnit")?;
    if !smallest_unit.is_undefined() {
        return Ok((
            Some(temporal_duration_smallest_unit_digits(cx, smallest_unit)?),
            rounding_mode,
        ));
    }
    Ok((fractional_digits, rounding_mode))
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_smallest_unit_digits<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
) -> Result<u8, Cx::Error> {
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    match text.as_str() {
        "second" | "seconds" => Ok(0),
        "millisecond" | "milliseconds" => Ok(3),
        "microsecond" | "microseconds" => Ok(6),
        "nanosecond" | "nanoseconds" => Ok(9),
        _ => Err(range_error(cx)),
    }
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_fractional_second_digits_option<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
) -> Result<Option<u8>, Cx::Error> {
    if value.is_undefined() {
        return Ok(None);
    }
    if let Some(number) = value.as_f64() {
        if !number.is_finite() {
            return Err(range_error(cx));
        }
        let digits = number.floor();
        if !(0.0..=9.0).contains(&digits) {
            return Err(range_error(cx));
        }
        return Ok(Some(temporal_number_to_u8_after_range_check(digits)));
    }
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    if text == "auto" {
        return Ok(None);
    }
    Err(range_error(cx))
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_rounding_mode_option<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalBuiltinRoundingMode, Cx::Error> {
    temporal_duration_rounding_mode_option_with_default(
        cx,
        value,
        TemporalBuiltinRoundingMode::Trunc,
    )
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_rounding_mode_option_with_default<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
    default: TemporalBuiltinRoundingMode,
) -> Result<TemporalBuiltinRoundingMode, Cx::Error> {
    if value.is_undefined() {
        return Ok(default);
    }
    temporal_duration_rounding_mode_from_value(cx, value)
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_rounding_mode_from_value<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalBuiltinRoundingMode, Cx::Error> {
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    match text.as_str() {
        "ceil" => Ok(TemporalBuiltinRoundingMode::Ceil),
        "floor" => Ok(TemporalBuiltinRoundingMode::Floor),
        "expand" => Ok(TemporalBuiltinRoundingMode::Expand),
        "trunc" => Ok(TemporalBuiltinRoundingMode::Trunc),
        "halfCeil" => Ok(TemporalBuiltinRoundingMode::HalfCeil),
        "halfFloor" => Ok(TemporalBuiltinRoundingMode::HalfFloor),
        "halfExpand" => Ok(TemporalBuiltinRoundingMode::HalfExpand),
        "halfTrunc" => Ok(TemporalBuiltinRoundingMode::HalfTrunc),
        "halfEven" => Ok(TemporalBuiltinRoundingMode::HalfEven),
        _ => Err(range_error(cx)),
    }
}

pub(in crate::public::dispatch::temporal) fn temporal_option_string_text<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
) -> Result<Option<String>, Cx::Error> {
    if value.is_undefined() {
        return Ok(None);
    }
    let string_ref = to_string_string_ref(cx, value)?;
    Ok(Some(string_ref_text(cx, string_ref)?))
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::public::dispatch::temporal) enum TemporalDurationCalendarUnit {
    Year,
    Month,
    Week,
}

#[derive(Clone, Copy)]
pub(in crate::public::dispatch::temporal) enum TemporalDurationParsedUnit {
    CalendarRelative(TemporalDurationCalendarUnit),
    Exact(TemporalBuiltinDurationExactUnit),
}

#[derive(Clone, Copy)]
pub(in crate::public::dispatch::temporal) enum TemporalDurationParsedLargestUnit {
    Missing,
    Auto,
    CalendarRelative(TemporalDurationCalendarUnit),
    Exact(TemporalBuiltinDurationExactUnit),
}

pub(in crate::public::dispatch::temporal) struct TemporalDurationRoundOptions {
    pub(super) largest_unit: TemporalDurationParsedLargestUnit,
    pub(super) smallest_unit: Option<TemporalDurationParsedUnit>,
    pub(super) rounding_increment: i128,
    pub(super) rounding_mode: TemporalBuiltinRoundingMode,
    pub(super) relative_to: Option<TemporalDurationRelativeTo>,
}

pub(in crate::public::dispatch::temporal) struct TemporalDurationTotalOptions {
    pub(super) unit: TemporalDurationParsedUnit,
    pub(super) relative_to: Option<TemporalDurationRelativeTo>,
}

#[derive(Clone, Copy)]
pub(in crate::public::dispatch::temporal) enum TemporalDurationRelativeTo {
    PlainDate(TemporalPlainDateObjectData),
    PlainDateTime(TemporalPlainDateTimeObjectData),
    ZonedDateTime(TemporalZonedDateTimeObjectData),
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_round_options<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationRoundOptions, Cx::Error> {
    if value.is_undefined() || value.is_null() {
        return Err(type_error(cx));
    }
    if value.is_string() {
        return Ok(TemporalDurationRoundOptions {
            largest_unit: TemporalDurationParsedLargestUnit::Missing,
            smallest_unit: Some(temporal_duration_parsed_unit(cx, value)?),
            rounding_increment: 1,
            rounding_mode: TemporalBuiltinRoundingMode::HalfExpand,
            relative_to: None,
        });
    }
    let Some(object_ref) = value.as_object_ref() else {
        return Err(type_error(cx));
    };

    let largest_unit_value = temporal_property_value(cx, object_ref, "largestUnit")?;
    let largest_unit = temporal_duration_largest_unit_option(cx, largest_unit_value)?;
    let relative_to = temporal_property_value(cx, object_ref, "relativeTo")?;
    let relative_to = temporal_duration_relative_to_option(cx, relative_to)?;
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
    let smallest_unit = if smallest_unit_value.is_undefined() {
        None
    } else {
        Some(temporal_duration_parsed_unit(cx, smallest_unit_value)?)
    };

    if matches!(largest_unit, TemporalDurationParsedLargestUnit::Missing) && smallest_unit.is_none()
    {
        return Err(range_error(cx));
    }

    Ok(TemporalDurationRoundOptions {
        largest_unit,
        smallest_unit,
        rounding_increment,
        rounding_mode,
        relative_to,
    })
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_total_options<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationTotalOptions, Cx::Error> {
    if value.is_undefined() || value.is_null() {
        return Err(type_error(cx));
    }
    if value.is_string() {
        return Ok(TemporalDurationTotalOptions {
            unit: temporal_duration_parsed_unit(cx, value)?,
            relative_to: None,
        });
    }
    let Some(object_ref) = value.as_object_ref() else {
        return Err(type_error(cx));
    };
    let relative_to = temporal_property_value(cx, object_ref, "relativeTo")?;
    let relative_to = temporal_duration_relative_to_option(cx, relative_to)?;
    let unit_value = temporal_property_value(cx, object_ref, "unit")?;
    if unit_value.is_undefined() {
        return Err(range_error(cx));
    }
    Ok(TemporalDurationTotalOptions {
        unit: temporal_duration_parsed_unit(cx, unit_value)?,
        relative_to,
    })
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_relative_to_option<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
) -> Result<Option<TemporalDurationRelativeTo>, Cx::Error> {
    if value.is_undefined() {
        return Ok(None);
    }
    if value.is_null() {
        return Err(type_error(cx));
    }
    if let Some(string_ref) = value.as_string_ref() {
        let text = string_ref_text(cx, string_ref)?;
        if temporal_zoned_date_time_zone_annotation(&text).is_some() {
            let zoned = temporal_zoned_date_time_from_value(cx, value)?;
            temporal_duration_validate_relative_zoned_string_limits(cx, &text, zoned)?;
            return Ok(Some(TemporalDurationRelativeTo::ZonedDateTime(zoned)));
        }
        if text.contains('T') || text.contains('t') {
            return temporal_plain_date_time_from_value(cx, value)
                .map(TemporalDurationRelativeTo::PlainDateTime)
                .map(Some);
        }
        return temporal_plain_date_from_value(cx, value)
            .map(TemporalDurationRelativeTo::PlainDate)
            .map(Some);
    }
    let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let existing = {
        let agent = cx.agent();
        agent.objects().temporal_object(object_ref).copied()
    };
    match existing {
        Some(TemporalObjectData::PlainDate(data)) => {
            return Ok(Some(TemporalDurationRelativeTo::PlainDate(data)));
        }
        Some(TemporalObjectData::PlainDateTime(data)) => {
            return Ok(Some(TemporalDurationRelativeTo::PlainDateTime(data)));
        }
        Some(TemporalObjectData::ZonedDateTime(data)) => {
            return Ok(Some(TemporalDurationRelativeTo::ZonedDateTime(data)));
        }
        _ => {}
    }

    temporal_duration_relative_to_from_property_bag(cx, object_ref).map(Some)
}

pub(in crate::public::dispatch::temporal) fn temporal_duration_validate_relative_zoned_string_limits<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    text: &str,
    zoned: TemporalZonedDateTimeObjectData,
) -> Result<(), Cx::Error> {
    if zoned.epoch_nanoseconds() == -temporal_ops::INSTANT_EPOCH_NANOSECONDS_MAX
        && matches!(
            temporal_zoned_date_time_explicit_offset(text),
            Some(offset) if offset < 0
        )
    {
        return Err(range_error(cx));
    }
    Ok(())
}

#[allow(
    clippy::too_many_lines,
    reason = "relativeTo property-bag parsing keeps calendar, time-zone, and date-time branches together"
)]
pub(in crate::public::dispatch::temporal) fn temporal_duration_relative_to_from_property_bag<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    object_ref: ObjectRef,
) -> Result<TemporalDurationRelativeTo, Cx::Error> {
    let calendar = temporal_property_value(cx, object_ref, "calendar")?;
    if !calendar.is_undefined() {
        temporal_validate_iso_calendar_value(cx, calendar)?;
    }
    let day_value = temporal_property_value(cx, object_ref, "day")?;
    if day_value.is_undefined() {
        return Err(type_error(cx));
    }
    let day = temporal_integer_part_from_value(cx, day_value)?;
    let hour_value = temporal_property_value(cx, object_ref, "hour")?;
    let hour = if hour_value.is_undefined() {
        None
    } else {
        Some(temporal_time_part_from_value(cx, hour_value)?)
    };
    let microsecond_value = temporal_property_value(cx, object_ref, "microsecond")?;
    let microsecond = if microsecond_value.is_undefined() {
        None
    } else {
        Some(temporal_time_part_from_value(cx, microsecond_value)?)
    };
    let millisecond_value = temporal_property_value(cx, object_ref, "millisecond")?;
    let millisecond = if millisecond_value.is_undefined() {
        None
    } else {
        Some(temporal_time_part_from_value(cx, millisecond_value)?)
    };
    let minute_value = temporal_property_value(cx, object_ref, "minute")?;
    let minute = if minute_value.is_undefined() {
        None
    } else {
        Some(temporal_time_part_from_value(cx, minute_value)?)
    };
    let month_value = temporal_property_value(cx, object_ref, "month")?;
    let month = if month_value.is_undefined() {
        None
    } else {
        Some(temporal_integer_part_from_value(cx, month_value)?)
    };
    let month_code_value = temporal_property_value(cx, object_ref, "monthCode")?;
    let month = if let Some(month) = month {
        if !month_code_value.is_undefined() {
            let month_code = temporal_month_from_month_code_value(cx, month_code_value)?;
            if month != month_code {
                return Err(range_error(cx));
            }
        }
        month
    } else if !month_code_value.is_undefined() {
        temporal_month_from_month_code_value(cx, month_code_value)?
    } else {
        return Err(type_error(cx));
    };
    let nanosecond_value = temporal_property_value(cx, object_ref, "nanosecond")?;
    let nanosecond = if nanosecond_value.is_undefined() {
        None
    } else {
        Some(temporal_time_part_from_value(cx, nanosecond_value)?)
    };
    let offset = temporal_property_value(cx, object_ref, "offset")?;
    let offset_text = if offset.is_undefined() {
        None
    } else if let Some(offset_ref) = offset.as_string_ref() {
        Some(string_ref_text(cx, offset_ref)?)
    } else {
        if offset.as_object_ref().is_none() {
            return Err(type_error(cx));
        }
        let offset_ref = to_string_string_ref(cx, offset)?;
        Some(string_ref_text(cx, offset_ref)?)
    };
    if let Some(offset_text) = offset_text.as_ref()
        && temporal_parse_offset_string(offset_text, true).is_none()
    {
        return Err(range_error(cx));
    }
    let second_value = temporal_property_value(cx, object_ref, "second")?;
    let second = if second_value.is_undefined() {
        None
    } else {
        Some(temporal_time_part_from_value(cx, second_value)?)
    };
    let time_zone = temporal_property_value(cx, object_ref, "timeZone")?;
    let year_value = temporal_property_value(cx, object_ref, "year")?;
    if year_value.is_undefined() {
        return Err(type_error(cx));
    }
    let year = temporal_integer_part_from_value(cx, year_value)?;

    let date = temporal_plain_date_from_parts(cx, year, month, day)?;
    let hour = hour.unwrap_or(0);
    let minute = minute.unwrap_or(0);
    let second = second.unwrap_or(0);
    let millisecond = millisecond.unwrap_or(0);
    let microsecond = microsecond.unwrap_or(0);
    let nanosecond = nanosecond.unwrap_or(0);

    if !time_zone.is_undefined() {
        let time_zone_id = temporal_time_zone_id_from_value(cx, time_zone)?;
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
        let date_time = TemporalCivilDateTime::new(
            date.year(),
            date.month(),
            date.day(),
            hour,
            minute,
            second,
            millisecond,
            microsecond,
            nanosecond,
        );
        let instant = cx.temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
            time_zone_id: time_zone_id.clone(),
            date_time,
            disambiguation: TemporalDisambiguation::Compatible,
        })?;
        let zoned =
            temporal_zoned_date_time_from_parts(cx, instant.epoch_nanoseconds, &time_zone_id)?;
        return Ok(TemporalDurationRelativeTo::ZonedDateTime(zoned));
    }

    if hour_value.is_undefined()
        && minute_value.is_undefined()
        && second_value.is_undefined()
        && millisecond_value.is_undefined()
        && microsecond_value.is_undefined()
        && nanosecond_value.is_undefined()
    {
        return Ok(TemporalDurationRelativeTo::PlainDate(date));
    }

    let date_time = temporal_plain_date_time_from_parts_with_overflow(
        cx,
        i64::from(date.year()),
        i64::from(date.month()),
        i64::from(date.day()),
        hour,
        minute,
        second,
        millisecond,
        microsecond,
        nanosecond,
        TemporalOverflow::Constrain,
    )?;
    Ok(TemporalDurationRelativeTo::PlainDateTime(date_time))
}

pub(in crate::public::dispatch::temporal) const fn temporal_duration_exact_unit_nanoseconds(
    unit: TemporalBuiltinDurationExactUnit,
) -> i128 {
    match unit {
        TemporalBuiltinDurationExactUnit::Day => TEMPORAL_NANOS_PER_DAY,
        TemporalBuiltinDurationExactUnit::Hour => TEMPORAL_NANOS_PER_HOUR,
        TemporalBuiltinDurationExactUnit::Minute => TEMPORAL_NANOS_PER_MINUTE,
        TemporalBuiltinDurationExactUnit::Second => TEMPORAL_NANOS_PER_SECOND,
        TemporalBuiltinDurationExactUnit::Millisecond => TEMPORAL_NANOS_PER_MILLISECOND,
        TemporalBuiltinDurationExactUnit::Microsecond => TEMPORAL_NANOS_PER_MICROSECOND,
        TemporalBuiltinDurationExactUnit::Nanosecond => 1,
    }
}

pub(in crate::public::dispatch::temporal) const fn temporal_date_time_difference_unit_from_duration_exact(
    unit: TemporalBuiltinDurationExactUnit,
) -> TemporalDateTimeDifferenceUnit {
    match unit {
        TemporalBuiltinDurationExactUnit::Day => TemporalDateTimeDifferenceUnit::Day,
        TemporalBuiltinDurationExactUnit::Hour => TemporalDateTimeDifferenceUnit::Hour,
        TemporalBuiltinDurationExactUnit::Minute => TemporalDateTimeDifferenceUnit::Minute,
        TemporalBuiltinDurationExactUnit::Second => TemporalDateTimeDifferenceUnit::Second,
        TemporalBuiltinDurationExactUnit::Millisecond => {
            TemporalDateTimeDifferenceUnit::Millisecond
        }
        TemporalBuiltinDurationExactUnit::Microsecond => {
            TemporalDateTimeDifferenceUnit::Microsecond
        }
        TemporalBuiltinDurationExactUnit::Nanosecond => TemporalDateTimeDifferenceUnit::Nanosecond,
    }
}

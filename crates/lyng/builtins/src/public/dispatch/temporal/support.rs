use super::*;
use std::fmt::Write as _;

pub(super) const fn temporal_i64_as_number(value: i64) -> f64 {
    #[allow(
        clippy::cast_precision_loss,
        reason = "Temporal host offsets are exposed through ECMAScript Number values"
    )]
    let number = value as f64;
    number
}

pub(super) const fn temporal_i128_as_number(value: i128) -> f64 {
    #[allow(
        clippy::cast_precision_loss,
        reason = "Temporal exact nanosecond values are exposed through ECMAScript Number operations"
    )]
    let number = value as f64;
    number
}

pub(super) const fn temporal_number_to_i64_after_range_check(number: f64) -> i64 {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "caller validates Temporal safe-integer range before narrowing to i64"
    )]
    let integer = number as i64;
    integer
}

pub(super) const fn temporal_number_to_i128_after_range_check(number: f64) -> i128 {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "caller validates Temporal integer range before narrowing to i128"
    )]
    let integer = number as i128;
    integer
}

pub(super) const fn temporal_number_to_u8_after_range_check(number: f64) -> u8 {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "caller validates Temporal fractional-second digit range before narrowing"
    )]
    let integer = number as u8;
    integer
}

pub(super) fn temporal_constructor_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor: ObjectRef,
) -> Result<ObjectRef, Cx::Error> {
    let prototype = cx.get_property_from_object_with_receiver(
        constructor,
        PropertyKey::from_atom(WellKnownAtom::prototype.id()),
        Value::from_object_ref(constructor),
    )?;
    prototype.as_object_ref().ok_or_else(|| type_error(cx))
}

pub(super) fn current_temporal_constructor_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor_name: &str,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    let global_object = cx
        .agent()
        .realm(realm)
        .map(|realm| realm.global_object())
        .ok_or_else(|| type_error(cx))?;
    let (temporal_key, instant_key) = {
        let agent = cx.agent();
        (
            PropertyKey::from_atom(agent.atoms_mut().intern_collectible("Temporal")),
            PropertyKey::from_atom(agent.atoms_mut().intern_collectible(constructor_name)),
        )
    };
    let temporal = cx.get_property_from_object_with_receiver(
        global_object,
        temporal_key,
        Value::from_object_ref(global_object),
    )?;
    let temporal_object = temporal.as_object_ref().ok_or_else(|| type_error(cx))?;
    let constructor = cx.get_property_from_object_with_receiver(
        temporal_object,
        instant_key,
        Value::from_object_ref(temporal_object),
    )?;
    let constructor = constructor.as_object_ref().ok_or_else(|| type_error(cx))?;
    temporal_constructor_prototype(cx, constructor)
}

pub(super) fn current_temporal_instant_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    current_temporal_constructor_prototype(cx, "Instant")
}

pub(super) fn current_temporal_duration_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    current_temporal_constructor_prototype(cx, "Duration")
}

pub(super) fn current_temporal_plain_date_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    current_temporal_constructor_prototype(cx, "PlainDate")
}

pub(super) fn current_temporal_plain_time_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    current_temporal_constructor_prototype(cx, "PlainTime")
}

pub(super) fn current_temporal_plain_date_time_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    current_temporal_constructor_prototype(cx, "PlainDateTime")
}

pub(super) fn current_temporal_plain_year_month_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    current_temporal_constructor_prototype(cx, "PlainYearMonth")
}

pub(super) fn current_temporal_plain_month_day_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    current_temporal_constructor_prototype(cx, "PlainMonthDay")
}

pub(super) fn current_temporal_zoned_date_time_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    current_temporal_constructor_prototype(cx, "ZonedDateTime")
}

pub(super) fn temporal_integer_part_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<i64, Cx::Error> {
    let number = to_number_for_builtin(cx, value)?;
    if !number.is_finite() {
        return Err(range_error(cx));
    }
    let number = number.trunc();
    let max = temporal_i128_as_number(TEMPORAL_SAFE_INTEGER_MAX);
    if !(-max..=max).contains(&number) {
        return Err(range_error(cx));
    }
    Ok(temporal_number_to_i64_after_range_check(number))
}

pub(super) fn temporal_integer_part_from_argument<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    index: usize,
) -> Result<i64, Cx::Error> {
    temporal_integer_part_from_value(
        cx,
        invocation
            .arguments()
            .get(index)
            .copied()
            .unwrap_or(Value::undefined()),
    )
}

pub(super) fn temporal_property_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    property_name: &str,
) -> Result<Value, Cx::Error> {
    let key = {
        let agent = cx.agent();
        PropertyKey::from_atom(agent.atoms_mut().intern_collectible(property_name))
    };
    cx.get_property_value(Value::from_object_ref(object_ref), key)
}

pub(super) fn temporal_validate_options_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<(), Cx::Error> {
    if value.is_undefined() || value.as_object_ref().is_some() {
        return Ok(());
    }
    Err(type_error(cx))
}

pub(super) fn temporal_required_integer_part_from_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    property_name: &str,
) -> Result<i64, Cx::Error> {
    let value = temporal_property_value(cx, object_ref, property_name)?;
    if value.is_undefined() {
        return Err(type_error(cx));
    }
    temporal_integer_part_from_value(cx, value)
}

pub(super) fn temporal_optional_integer_part_from_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    property_name: &str,
) -> Result<Option<i64>, Cx::Error> {
    let value = temporal_property_value(cx, object_ref, property_name)?;
    if value.is_undefined() {
        return Ok(None);
    }
    temporal_integer_part_from_value(cx, value).map(Some)
}

pub(super) fn temporal_string_text_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<String, Cx::Error> {
    let primitive = if value.is_object() {
        let mut bridge = BuiltinToPrimitiveBridge { cx };
        object::to_primitive(&mut bridge, value, object::ToPrimitiveHint::String)?
    } else {
        value
    };
    let string_ref = primitive.as_string_ref().ok_or_else(|| type_error(cx))?;
    string_ref_text(cx, string_ref)
}

pub(super) fn temporal_optional_string_text_from_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    property_name: &str,
) -> Result<Option<String>, Cx::Error> {
    let value = temporal_property_value(cx, object_ref, property_name)?;
    if value.is_undefined() {
        return Ok(None);
    }
    temporal_string_text_from_value(cx, value).map(Some)
}

pub(super) fn temporal_optional_month_code_text_from_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
) -> Result<Option<String>, Cx::Error> {
    let text = temporal_optional_string_text_from_property(cx, object_ref, "monthCode")?;
    if let Some(text) = text {
        if temporal_parse_month_code_syntax(&text).is_none() {
            return Err(range_error(cx));
        }
        return Ok(Some(text));
    }
    Ok(None)
}

pub(super) fn temporal_resolve_month_from_fields<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    month: Option<i64>,
    month_code_text: Option<&str>,
    default_month: Option<i64>,
) -> Result<i64, Cx::Error> {
    if let Some(month) = month {
        if let Some(text) = month_code_text {
            let month_code =
                temporal_month_from_month_code_text(text).ok_or_else(|| range_error(cx))?;
            if month != month_code {
                return Err(range_error(cx));
            }
        }
        return Ok(month);
    }
    if let Some(text) = month_code_text {
        return temporal_month_from_month_code_text(text).ok_or_else(|| range_error(cx));
    }
    default_month.ok_or_else(|| type_error(cx))
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TemporalOverflow {
    Constrain,
    Reject,
}

pub(super) fn temporal_overflow_from_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    options: Value,
) -> Result<TemporalOverflow, Cx::Error> {
    temporal_validate_options_object(cx, options)?;
    if options.is_undefined() {
        return Ok(TemporalOverflow::Constrain);
    }
    let object_ref = options.as_object_ref().ok_or_else(|| type_error(cx))?;
    let overflow_value = temporal_property_value(cx, object_ref, "overflow")?;
    match temporal_string_option(cx, overflow_value, &["constrain", "reject"], "constrain")?
        .as_str()
    {
        "constrain" => Ok(TemporalOverflow::Constrain),
        "reject" => Ok(TemporalOverflow::Reject),
        _ => unreachable!("temporal_string_option constrained overflow"),
    }
}

pub(super) fn temporal_month_from_month_code_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<i64, Cx::Error> {
    let text = temporal_string_text_from_value(cx, value)?;
    temporal_month_from_month_code_text(&text).ok_or_else(|| range_error(cx))
}

pub(super) enum TemporalMonthCodeSyntax {
    Standard(i64),
    Leap,
}

pub(super) fn temporal_parse_month_code_syntax(text: &str) -> Option<TemporalMonthCodeSyntax> {
    let bytes = text.as_bytes();
    if !(bytes.len() == 3 || bytes.len() == 4) || bytes[0] != b'M' {
        return None;
    }
    if !bytes[1].is_ascii_digit() || !bytes[2].is_ascii_digit() {
        return None;
    }
    let month = i64::from((bytes[1] - b'0') * 10 + (bytes[2] - b'0'));
    if bytes.len() == 4 {
        (bytes[3] == b'L').then_some(TemporalMonthCodeSyntax::Leap)
    } else {
        Some(TemporalMonthCodeSyntax::Standard(month))
    }
}

pub(super) fn temporal_month_from_month_code_text(text: &str) -> Option<i64> {
    match temporal_parse_month_code_syntax(text)? {
        TemporalMonthCodeSyntax::Standard(month) if (1..=12).contains(&month) => Some(month),
        TemporalMonthCodeSyntax::Standard(_) | TemporalMonthCodeSyntax::Leap => None,
    }
}

pub(super) fn temporal_month_from_property_bag<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    default_month: Option<i64>,
) -> Result<i64, Cx::Error> {
    let month_value = temporal_optional_integer_part_from_property(cx, object_ref, "month")?;
    let month_code_value = temporal_property_value(cx, object_ref, "monthCode")?;
    if let Some(month) = month_value {
        if !month_code_value.is_undefined() {
            let month_code = temporal_month_from_month_code_value(cx, month_code_value)?;
            if month != month_code {
                return Err(range_error(cx));
            }
        }
        return Ok(month);
    }
    if !month_code_value.is_undefined() {
        return temporal_month_from_month_code_value(cx, month_code_value);
    }
    default_month.ok_or_else(|| type_error(cx))
}

pub(super) fn temporal_validate_iso_calendar_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<(), Cx::Error> {
    if value.is_undefined() {
        return Err(type_error(cx));
    }
    if let Some(object_ref) = value.as_object_ref() {
        let temporal = {
            let agent = cx.agent();
            agent.objects().temporal_object(object_ref).copied()
        };
        return match temporal {
            Some(
                TemporalObjectData::PlainDate(_)
                | TemporalObjectData::PlainDateTime(_)
                | TemporalObjectData::PlainMonthDay(_)
                | TemporalObjectData::PlainYearMonth(_)
                | TemporalObjectData::ZonedDateTime(_),
            ) => Ok(()),
            _ => Err(type_error(cx)),
        };
    }
    let Some(string_ref) = value.as_string_ref() else {
        return Err(type_error(cx));
    };
    let text = string_ref_text(cx, string_ref)?;
    if temporal_is_valid_iso_calendar_string(&text) {
        return Ok(());
    }
    Err(range_error(cx))
}

pub(super) fn temporal_validate_iso_calendar_identifier_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<(), Cx::Error> {
    let Some(string_ref) = value.as_string_ref() else {
        return Err(type_error(cx));
    };
    let text = string_ref_text(cx, string_ref)?;
    if text.eq_ignore_ascii_case("iso8601") {
        return Ok(());
    }
    Err(range_error(cx))
}

pub(super) fn temporal_validate_optional_iso_calendar_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
) -> Result<(), Cx::Error> {
    let value = temporal_property_value(cx, object_ref, "calendar")?;
    if value.is_undefined() {
        return Ok(());
    }
    temporal_validate_iso_calendar_value(cx, value)
}

pub(super) fn temporal_validate_optional_iso_calendar_identifier_argument<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    index: usize,
) -> Result<(), Cx::Error> {
    let value = invocation
        .arguments()
        .get(index)
        .copied()
        .unwrap_or(Value::undefined());
    if value.is_undefined() {
        return Ok(());
    }
    temporal_validate_iso_calendar_identifier_value(cx, value)
}

pub(super) fn temporal_is_valid_iso_calendar_string(text: &str) -> bool {
    if text.eq_ignore_ascii_case("iso8601") {
        return true;
    }
    temporal_ops::parse_plain_date(text).is_some()
        || temporal_ops::parse_plain_date_time(text).is_some()
        || parse_temporal_plain_time(text).is_some()
        || temporal_ops::parse_plain_year_month(text).is_some()
        || temporal_ops::parse_plain_month_day(text).is_some()
        || parse_temporal_instant(text).is_some()
}

pub(super) fn temporal_time_part_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<i64, Cx::Error> {
    if value.is_undefined() {
        return Ok(0);
    }
    temporal_integer_part_from_value(cx, value)
}

pub(super) fn temporal_time_part_from_argument<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    index: usize,
) -> Result<i64, Cx::Error> {
    temporal_time_part_from_value(
        cx,
        invocation
            .arguments()
            .get(index)
            .copied()
            .unwrap_or(Value::undefined()),
    )
}

pub(super) fn temporal_time_part_from_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    property_name: &str,
) -> Result<i64, Cx::Error> {
    let key = {
        let agent = cx.agent();
        PropertyKey::from_atom(agent.atoms_mut().intern_collectible(property_name))
    };
    let value = cx.get_property_value(Value::from_object_ref(object_ref), key)?;
    temporal_time_part_from_value(cx, value)
}

pub(super) fn temporal_optional_time_part_from_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    property_name: &str,
) -> Result<Option<i64>, Cx::Error> {
    let value = temporal_property_value(cx, object_ref, property_name)?;
    if value.is_undefined() {
        return Ok(None);
    }
    temporal_time_part_from_value(cx, value).map(Some)
}

pub(super) fn temporal_plain_time_from_parts<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    hour: i64,
    minute: i64,
    second: i64,
    millisecond: i64,
    microsecond: i64,
    nanosecond: i64,
) -> Result<TemporalPlainTimeObjectData, Cx::Error> {
    temporal_plain_time_from_parts_with_overflow(
        cx,
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
    reason = "PlainTime construction takes the explicit ECMA time fields plus overflow"
)]
pub(super) fn temporal_plain_time_from_parts_with_overflow<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    hour: i64,
    minute: i64,
    second: i64,
    millisecond: i64,
    microsecond: i64,
    nanosecond: i64,
    overflow: TemporalOverflow,
) -> Result<TemporalPlainTimeObjectData, Cx::Error> {
    let (hour, minute, second, millisecond, microsecond, nanosecond) = match overflow {
        TemporalOverflow::Constrain => (
            hour.clamp(0, 23),
            minute.clamp(0, 59),
            second.clamp(0, 59),
            millisecond.clamp(0, 999),
            microsecond.clamp(0, 999),
            nanosecond.clamp(0, 999),
        ),
        TemporalOverflow::Reject => (hour, minute, second, millisecond, microsecond, nanosecond),
    };
    let hour = u8::try_from(hour).map_err(|_| range_error(cx))?;
    if hour > 23 {
        return Err(range_error(cx));
    }
    let minute = u8::try_from(minute).map_err(|_| range_error(cx))?;
    if minute > 59 {
        return Err(range_error(cx));
    }
    let second = u8::try_from(second).map_err(|_| range_error(cx))?;
    if second > 59 {
        return Err(range_error(cx));
    }
    let millisecond = u16::try_from(millisecond).map_err(|_| range_error(cx))?;
    if millisecond > 999 {
        return Err(range_error(cx));
    }
    let microsecond = u16::try_from(microsecond).map_err(|_| range_error(cx))?;
    if microsecond > 999 {
        return Err(range_error(cx));
    }
    let nanosecond = u16::try_from(nanosecond).map_err(|_| range_error(cx))?;
    if nanosecond > 999 {
        return Err(range_error(cx));
    }

    Ok(TemporalPlainTimeObjectData::new(
        hour,
        minute,
        second,
        millisecond,
        microsecond,
        nanosecond,
    ))
}

pub(super) fn temporal_subsecond_parts_from_nanoseconds<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    fraction_nanoseconds: i128,
) -> Result<(i64, i64, i64), Cx::Error> {
    let millisecond = fraction_nanoseconds / TEMPORAL_NANOS_PER_MILLISECOND;
    let remainder = fraction_nanoseconds % TEMPORAL_NANOS_PER_MILLISECOND;
    let microsecond = remainder / TEMPORAL_NANOS_PER_MICROSECOND;
    let nanosecond = remainder % TEMPORAL_NANOS_PER_MICROSECOND;
    Ok((
        i64::try_from(millisecond).map_err(|_| range_error(cx))?,
        i64::try_from(microsecond).map_err(|_| range_error(cx))?,
        i64::try_from(nanosecond).map_err(|_| range_error(cx))?,
    ))
}

pub(super) fn temporal_plain_time_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalPlainTimeObjectData, Cx::Error> {
    temporal_plain_time_from_value_with_overflow(cx, value, TemporalOverflow::Constrain)
}

pub(super) fn temporal_plain_time_from_value_with_overflow<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    overflow: TemporalOverflow,
) -> Result<TemporalPlainTimeObjectData, Cx::Error> {
    if let Some(string_ref) = value.as_string_ref() {
        let text = string_ref_text(cx, string_ref)?;
        let (hour, minute, second, fraction_nanoseconds) =
            parse_temporal_plain_time(&text).ok_or_else(|| range_error(cx))?;
        let (millisecond, microsecond, nanosecond) =
            temporal_subsecond_parts_from_nanoseconds(cx, fraction_nanoseconds)?;
        return temporal_plain_time_from_parts(
            cx,
            i64::from(hour),
            i64::from(minute),
            i64::from(second),
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
    if let Some(TemporalObjectData::PlainTime(data)) = existing {
        return Ok(data);
    }
    if let Some(TemporalObjectData::PlainDateTime(data)) = existing {
        return Ok(temporal_plain_date_time_time(data));
    }
    if let Some(TemporalObjectData::ZonedDateTime(data)) = existing {
        let civil = temporal_zoned_date_time_civil(cx, data)?;
        let date_time = civil.date_time;
        return Ok(TemporalPlainTimeObjectData::new(
            date_time.hour,
            date_time.minute,
            date_time.second,
            date_time.millisecond,
            date_time.microsecond,
            date_time.nanosecond,
        ));
    }

    let parts = temporal_plain_time_parts_from_property_bag(cx, object_ref)?;
    temporal_plain_time_from_parts_with_overflow(
        cx,
        parts.hour,
        parts.minute,
        parts.second,
        parts.millisecond,
        parts.microsecond,
        parts.nanosecond,
        overflow,
    )
}

pub(super) struct TemporalPlainTimeParts {
    pub(super) hour: i64,
    pub(super) minute: i64,
    pub(super) second: i64,
    pub(super) millisecond: i64,
    pub(super) microsecond: i64,
    pub(super) nanosecond: i64,
}

pub(super) fn temporal_plain_time_parts_from_property_bag<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
) -> Result<TemporalPlainTimeParts, Cx::Error> {
    let hour_value = temporal_property_value(cx, object_ref, "hour")?;
    let hour = temporal_time_part_from_value(cx, hour_value)?;
    let microsecond_value = temporal_property_value(cx, object_ref, "microsecond")?;
    let microsecond = temporal_time_part_from_value(cx, microsecond_value)?;
    let millisecond_value = temporal_property_value(cx, object_ref, "millisecond")?;
    let millisecond = temporal_time_part_from_value(cx, millisecond_value)?;
    let minute_value = temporal_property_value(cx, object_ref, "minute")?;
    let minute = temporal_time_part_from_value(cx, minute_value)?;
    let nanosecond_value = temporal_property_value(cx, object_ref, "nanosecond")?;
    let nanosecond = temporal_time_part_from_value(cx, nanosecond_value)?;
    let second_value = temporal_property_value(cx, object_ref, "second")?;
    let second = temporal_time_part_from_value(cx, second_value)?;
    if hour_value.is_undefined()
        && microsecond_value.is_undefined()
        && millisecond_value.is_undefined()
        && minute_value.is_undefined()
        && nanosecond_value.is_undefined()
        && second_value.is_undefined()
    {
        return Err(type_error(cx));
    }
    Ok(TemporalPlainTimeParts {
        hour,
        minute,
        second,
        millisecond,
        microsecond,
        nanosecond,
    })
}

pub(super) fn temporal_plain_time_for_string_precision<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    data: TemporalPlainTimeObjectData,
    precision: TemporalInstantStringPrecision,
    rounding_mode: TemporalBuiltinRoundingMode,
) -> Result<TemporalPlainTimeObjectData, Cx::Error> {
    let nanoseconds = match precision {
        TemporalInstantStringPrecision::Auto => return Ok(data),
        TemporalInstantStringPrecision::Minute => temporal_round_epoch_nanoseconds_to_increment(
            temporal_plain_time_nanoseconds(data),
            TEMPORAL_NANOS_PER_MINUTE,
            rounding_mode,
        )
        .ok_or_else(|| range_error(cx))?,
        TemporalInstantStringPrecision::FractionalSecond(digits) => {
            temporal_round_epoch_nanoseconds_to_fractional_digits(
                temporal_plain_time_nanoseconds(data),
                digits,
                rounding_mode,
            )
            .ok_or_else(|| range_error(cx))?
        }
    };
    temporal_plain_time_from_nanoseconds(cx, nanoseconds.rem_euclid(TEMPORAL_NANOS_PER_DAY))
}

pub(super) fn format_temporal_plain_time_with_precision(
    data: TemporalPlainTimeObjectData,
    precision: TemporalInstantStringPrecision,
) -> String {
    if precision == TemporalInstantStringPrecision::Auto {
        return format_temporal_plain_time(data);
    }

    let mut text = format!("{:02}:{:02}", data.hour(), data.minute());
    if precision == TemporalInstantStringPrecision::Minute {
        return text;
    }
    let _ = write!(&mut text, ":{:02}", data.second());
    let fraction = u32::from(data.millisecond()) * 1_000_000
        + u32::from(data.microsecond()) * 1_000
        + u32::from(data.nanosecond());
    match precision {
        TemporalInstantStringPrecision::FractionalSecond(0) => {}
        TemporalInstantStringPrecision::FractionalSecond(digits) => {
            let fraction_text = format!("{fraction:09}");
            text.push('.');
            text.push_str(&fraction_text[..usize::from(digits)]);
        }
        TemporalInstantStringPrecision::Auto | TemporalInstantStringPrecision::Minute => {
            unreachable!("handled before seconds formatting")
        }
    }
    text
}

pub(super) fn temporal_plain_time_from_nanoseconds<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    nanoseconds: i128,
) -> Result<TemporalPlainTimeObjectData, Cx::Error> {
    temporal_ops::plain_time_from_nanoseconds(nanoseconds).ok_or_else(|| range_error(cx))
}

pub(super) fn temporal_safe_integer_number<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: i128,
) -> Result<Value, Cx::Error> {
    if !(-TEMPORAL_SAFE_INTEGER_MAX..=TEMPORAL_SAFE_INTEGER_MAX).contains(&value) {
        return Err(range_error(cx));
    }
    Ok(Value::from_f64(temporal_i128_as_number(value)))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TemporalInstantStringPrecision {
    Auto,
    Minute,
    FractionalSecond(u8),
}

pub(super) fn temporal_instant_to_string_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<(TemporalInstantStringPrecision, TemporalBuiltinRoundingMode), Cx::Error> {
    if value.is_undefined() {
        return Ok((
            TemporalInstantStringPrecision::Auto,
            TemporalBuiltinRoundingMode::Trunc,
        ));
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
    if !smallest_unit.is_undefined() {
        return Ok((
            temporal_instant_smallest_unit_precision(cx, smallest_unit)?,
            rounding_mode,
        ));
    }
    Ok((fractional_second_precision, rounding_mode))
}

pub(super) fn temporal_instant_smallest_unit_precision<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalInstantStringPrecision, Cx::Error> {
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    temporal_instant_smallest_unit_precision_from_text(cx, &text)
}

pub(super) fn temporal_instant_smallest_unit_precision_from_text<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    text: &str,
) -> Result<TemporalInstantStringPrecision, Cx::Error> {
    match text {
        "minute" | "minutes" => Ok(TemporalInstantStringPrecision::Minute),
        "second" | "seconds" => Ok(TemporalInstantStringPrecision::FractionalSecond(0)),
        "millisecond" | "milliseconds" => Ok(TemporalInstantStringPrecision::FractionalSecond(3)),
        "microsecond" | "microseconds" => Ok(TemporalInstantStringPrecision::FractionalSecond(6)),
        "nanosecond" | "nanoseconds" => Ok(TemporalInstantStringPrecision::FractionalSecond(9)),
        _ => Err(range_error(cx)),
    }
}

pub(super) fn temporal_instant_fractional_second_digits_option<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalInstantStringPrecision, Cx::Error> {
    if value.is_undefined() {
        return Ok(TemporalInstantStringPrecision::Auto);
    }
    if let Some(number) = value.as_f64() {
        if !number.is_finite() {
            return Err(range_error(cx));
        }
        let digits = number.floor();
        if !(0.0..=9.0).contains(&digits) {
            return Err(range_error(cx));
        }
        return Ok(TemporalInstantStringPrecision::FractionalSecond(
            temporal_number_to_u8_after_range_check(digits),
        ));
    }
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    if text == "auto" {
        return Ok(TemporalInstantStringPrecision::Auto);
    }
    Err(range_error(cx))
}

pub(super) fn temporal_string_option<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    allowed: &[&str],
    default: &str,
) -> Result<String, Cx::Error> {
    if value.is_undefined() {
        return Ok(default.to_string());
    }
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    if allowed.iter().any(|allowed| *allowed == text) {
        return Ok(text);
    }
    Err(range_error(cx))
}

pub(super) const fn temporal_compare_ordering(ordering: std::cmp::Ordering) -> Value {
    Value::from_smi(match ordering {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    })
}

pub(super) fn temporal_exact_time_unit_from_text(
    text: &str,
) -> Option<TemporalBuiltinDurationExactUnit> {
    match text {
        "hour" | "hours" => Some(TemporalBuiltinDurationExactUnit::Hour),
        "minute" | "minutes" => Some(TemporalBuiltinDurationExactUnit::Minute),
        "second" | "seconds" => Some(TemporalBuiltinDurationExactUnit::Second),
        "millisecond" | "milliseconds" => Some(TemporalBuiltinDurationExactUnit::Millisecond),
        "microsecond" | "microseconds" => Some(TemporalBuiltinDurationExactUnit::Microsecond),
        "nanosecond" | "nanoseconds" => Some(TemporalBuiltinDurationExactUnit::Nanosecond),
        _ => None,
    }
}

pub(super) fn temporal_exact_time_unit_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalBuiltinDurationExactUnit, Cx::Error> {
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    temporal_exact_time_unit_from_text(&text).ok_or_else(|| range_error(cx))
}

pub(super) fn temporal_exact_time_unit_order(unit: TemporalBuiltinDurationExactUnit) -> u8 {
    match unit {
        TemporalBuiltinDurationExactUnit::Hour => 0,
        TemporalBuiltinDurationExactUnit::Minute => 1,
        TemporalBuiltinDurationExactUnit::Second => 2,
        TemporalBuiltinDurationExactUnit::Millisecond => 3,
        TemporalBuiltinDurationExactUnit::Microsecond => 4,
        TemporalBuiltinDurationExactUnit::Nanosecond => 5,
        TemporalBuiltinDurationExactUnit::Day => {
            unreachable!("exact time helpers do not accept days")
        }
    }
}

pub(super) fn temporal_exact_time_largest_unit_includes(
    largest_unit: TemporalBuiltinDurationExactUnit,
    component_unit: TemporalBuiltinDurationExactUnit,
) -> bool {
    temporal_exact_time_unit_order(largest_unit) <= temporal_exact_time_unit_order(component_unit)
}

pub(super) fn temporal_exact_time_unit_nanoseconds(unit: TemporalBuiltinDurationExactUnit) -> i128 {
    match unit {
        TemporalBuiltinDurationExactUnit::Hour => TEMPORAL_NANOS_PER_HOUR,
        TemporalBuiltinDurationExactUnit::Minute => TEMPORAL_NANOS_PER_MINUTE,
        TemporalBuiltinDurationExactUnit::Second => TEMPORAL_NANOS_PER_SECOND,
        TemporalBuiltinDurationExactUnit::Millisecond => TEMPORAL_NANOS_PER_MILLISECOND,
        TemporalBuiltinDurationExactUnit::Microsecond => TEMPORAL_NANOS_PER_MICROSECOND,
        TemporalBuiltinDurationExactUnit::Nanosecond => 1,
        TemporalBuiltinDurationExactUnit::Day => {
            unreachable!("exact time helpers do not accept days")
        }
    }
}

pub(super) fn temporal_instant_largest_unit_default(
    smallest_unit: TemporalBuiltinDurationExactUnit,
) -> TemporalBuiltinDurationExactUnit {
    match smallest_unit {
        TemporalBuiltinDurationExactUnit::Hour | TemporalBuiltinDurationExactUnit::Minute => {
            smallest_unit
        }
        TemporalBuiltinDurationExactUnit::Second
        | TemporalBuiltinDurationExactUnit::Millisecond
        | TemporalBuiltinDurationExactUnit::Microsecond
        | TemporalBuiltinDurationExactUnit::Nanosecond => TemporalBuiltinDurationExactUnit::Second,
        TemporalBuiltinDurationExactUnit::Day => unreachable!("Instant does not accept days"),
    }
}

pub(super) const fn temporal_exact_time_rounding_increment_is_valid(
    smallest_unit: TemporalBuiltinDurationExactUnit,
    rounding_increment: i128,
) -> bool {
    let maximum = temporal_exact_time_rounding_increment_maximum(smallest_unit);
    rounding_increment > 0 && rounding_increment < maximum && maximum % rounding_increment == 0
}

pub(super) const fn temporal_exact_time_rounding_increment_maximum(
    smallest_unit: TemporalBuiltinDurationExactUnit,
) -> i128 {
    match smallest_unit {
        TemporalBuiltinDurationExactUnit::Hour => 24,
        TemporalBuiltinDurationExactUnit::Minute | TemporalBuiltinDurationExactUnit::Second => 60,
        TemporalBuiltinDurationExactUnit::Millisecond
        | TemporalBuiltinDurationExactUnit::Microsecond
        | TemporalBuiltinDurationExactUnit::Nanosecond => 1000,
        TemporalBuiltinDurationExactUnit::Day => 0,
    }
}

pub(super) fn temporal_instant_rounding_increment_is_valid(
    smallest_unit: TemporalBuiltinDurationExactUnit,
    rounding_increment: i128,
) -> bool {
    if smallest_unit == TemporalBuiltinDurationExactUnit::Hour {
        return rounding_increment > 0 && rounding_increment <= 24 && 24 % rounding_increment == 0;
    }
    let day_maximum = match smallest_unit {
        TemporalBuiltinDurationExactUnit::Minute => 24 * 60,
        TemporalBuiltinDurationExactUnit::Second => 24 * 60 * 60,
        TemporalBuiltinDurationExactUnit::Millisecond => 24 * 60 * 60 * 1_000,
        TemporalBuiltinDurationExactUnit::Microsecond => 24 * 60 * 60 * 1_000_000,
        TemporalBuiltinDurationExactUnit::Nanosecond => 24 * 60 * 60 * 1_000_000_000,
        TemporalBuiltinDurationExactUnit::Hour | TemporalBuiltinDurationExactUnit::Day => {
            return false;
        }
    };
    rounding_increment > 0
        && rounding_increment <= day_maximum
        && day_maximum % rounding_increment == 0
}

#[derive(Clone, Copy)]
pub(super) struct TemporalExactTimeRoundOptions {
    pub(super) smallest_unit: TemporalBuiltinDurationExactUnit,
    pub(super) rounding_increment: i128,
    pub(super) rounding_mode: TemporalBuiltinRoundingMode,
}

pub(super) fn temporal_exact_time_round_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalExactTimeRoundOptions, Cx::Error> {
    temporal_exact_time_round_options_with_validator(
        cx,
        value,
        temporal_exact_time_rounding_increment_is_valid,
    )
}

pub(super) fn temporal_instant_round_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalExactTimeRoundOptions, Cx::Error> {
    temporal_exact_time_round_options_with_validator(
        cx,
        value,
        temporal_instant_rounding_increment_is_valid,
    )
}

pub(super) fn temporal_exact_time_round_options_with_validator<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    validator: fn(TemporalBuiltinDurationExactUnit, i128) -> bool,
) -> Result<TemporalExactTimeRoundOptions, Cx::Error> {
    if value.is_string() {
        return Ok(TemporalExactTimeRoundOptions {
            smallest_unit: temporal_exact_time_unit_from_value(cx, value)?,
            rounding_increment: 1,
            rounding_mode: TemporalBuiltinRoundingMode::HalfExpand,
        });
    }
    let Some(object_ref) = value.as_object_ref() else {
        return Err(type_error(cx));
    };

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
    let smallest_unit = temporal_exact_time_unit_from_value(cx, smallest_unit_value)?;
    if !validator(smallest_unit, rounding_increment) {
        return Err(range_error(cx));
    }

    Ok(TemporalExactTimeRoundOptions {
        smallest_unit,
        rounding_increment,
        rounding_mode,
    })
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "Temporal Duration fields are stored as float64-representable integer values."
)]
pub(super) fn temporal_duration_from_nanoseconds_with_largest_unit<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    nanoseconds: i128,
    largest_unit: TemporalBuiltinDurationExactUnit,
) -> Result<TemporalDurationObjectData, Cx::Error> {
    let mut remainder = nanoseconds;
    let mut part = |unit: i128| -> Result<i64, Cx::Error> {
        let value = remainder / unit;
        remainder %= unit;
        let value = i64::try_from(value).map_err(|_| range_error(cx))?;
        Ok((value as f64) as i64)
    };
    let hours = if temporal_exact_time_largest_unit_includes(
        largest_unit,
        TemporalBuiltinDurationExactUnit::Hour,
    ) {
        part(TEMPORAL_NANOS_PER_HOUR)?
    } else {
        0
    };
    let minutes = if temporal_exact_time_largest_unit_includes(
        largest_unit,
        TemporalBuiltinDurationExactUnit::Minute,
    ) {
        part(TEMPORAL_NANOS_PER_MINUTE)?
    } else {
        0
    };
    let seconds = if temporal_exact_time_largest_unit_includes(
        largest_unit,
        TemporalBuiltinDurationExactUnit::Second,
    ) {
        part(TEMPORAL_NANOS_PER_SECOND)?
    } else {
        0
    };
    let milliseconds = if temporal_exact_time_largest_unit_includes(
        largest_unit,
        TemporalBuiltinDurationExactUnit::Millisecond,
    ) {
        part(TEMPORAL_NANOS_PER_MILLISECOND)?
    } else {
        0
    };
    let microseconds = if temporal_exact_time_largest_unit_includes(
        largest_unit,
        TemporalBuiltinDurationExactUnit::Microsecond,
    ) {
        part(TEMPORAL_NANOS_PER_MICROSECOND)?
    } else {
        0
    };
    let nanoseconds = i64::try_from(remainder).map_err(|_| range_error(cx))?;
    Ok(TemporalDurationObjectData::new(
        0,
        0,
        0,
        0,
        hours,
        minutes,
        seconds,
        milliseconds,
        microseconds,
        nanoseconds,
    ))
}

pub(super) fn format_temporal_civil_date_time_with_precision(
    date_time: TemporalCivilDateTime,
    calendar: AtomId,
    precision: TemporalInstantStringPrecision,
) -> String {
    let date =
        TemporalPlainDateObjectData::new(date_time.year, date_time.month, date_time.day, calendar);
    let time = TemporalPlainTimeObjectData::new(
        date_time.hour,
        date_time.minute,
        date_time.second,
        date_time.millisecond,
        date_time.microsecond,
        date_time.nanosecond,
    );
    format!(
        "{}T{}",
        format_temporal_plain_date(date),
        format_temporal_plain_time_with_precision(time, precision)
    )
}

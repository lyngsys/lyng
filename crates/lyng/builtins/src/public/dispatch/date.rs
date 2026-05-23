mod parsing;

use super::{
    map_completion, range_error, string_ref_text, string_value, temporal, to_number_for_builtin,
    type_error, BuiltinToPrimitiveBridge, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_gc::AllocationLifetime;
use lyng_js_host::{
    TemporalCivilDateTime, TemporalCivilToInstantRequest, TemporalDefaultTimeZoneRequest,
    TemporalDisambiguation, TemporalInstantToCivilRequest,
};
use lyng_js_ops::{object, read};
use lyng_js_types::{BuiltinFunctionId, ObjectRef, PropertyKey, RealmRef, Value};
use parsing::date_parse_text;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn dispatch_date_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_date_constructor_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_date_format_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_date_getter_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_date_setter_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_date_conversion_builtin(context, entry, invocation)
}

fn dispatch_date_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::date_builtin() {
        return date_builtin(context, invocation).map(Some);
    }
    if entry == super::date_now_builtin() {
        return Ok(Some(date_now_value()));
    }
    if entry == super::date_parse_builtin() {
        return date_parse_builtin(context, invocation).map(Some);
    }
    if entry == super::date_utc_builtin() {
        return date_utc_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_date_format_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::date_to_string_builtin() {
        return date_to_string_builtin(context, invocation, DateStringKind::Full).map(Some);
    }
    if entry == super::date_to_date_string_builtin() {
        return date_to_string_builtin(context, invocation, DateStringKind::Date).map(Some);
    }
    if entry == super::date_to_time_string_builtin() {
        return date_to_string_builtin(context, invocation, DateStringKind::Time).map(Some);
    }
    if entry == super::date_to_locale_string_builtin() {
        return date_to_string_builtin(context, invocation, DateStringKind::Full).map(Some);
    }
    if entry == super::date_to_locale_date_string_builtin() {
        return date_to_string_builtin(context, invocation, DateStringKind::Date).map(Some);
    }
    if entry == super::date_to_locale_time_string_builtin() {
        return date_to_string_builtin(context, invocation, DateStringKind::Time).map(Some);
    }
    Ok(None)
}

fn dispatch_date_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::date_value_of_builtin() || entry == super::date_get_time_builtin() {
        return date_value_of_builtin(context, invocation).map(Some);
    }
    if entry == super::date_get_full_year_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::FullYear, false)
            .map(Some);
    }
    if entry == super::date_get_year_builtin() {
        return date_get_year_builtin(context, invocation).map(Some);
    }
    if entry == super::date_get_utc_full_year_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::FullYear, true)
            .map(Some);
    }
    if entry == super::date_get_month_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Month, false)
            .map(Some);
    }
    if entry == super::date_get_utc_month_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Month, true)
            .map(Some);
    }
    dispatch_date_getter_part_two(context, entry, invocation)
}

fn dispatch_date_getter_part_two<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::date_get_date_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Date, false)
            .map(Some);
    }
    if entry == super::date_get_utc_date_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Date, true)
            .map(Some);
    }
    if entry == super::date_get_day_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Day, false)
            .map(Some);
    }
    if entry == super::date_get_utc_day_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Day, true).map(Some);
    }
    if entry == super::date_get_hours_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Hours, false)
            .map(Some);
    }
    if entry == super::date_get_utc_hours_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Hours, true)
            .map(Some);
    }
    dispatch_date_getter_part_three(context, entry, invocation)
}

fn dispatch_date_getter_part_three<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::date_get_minutes_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Minutes, false)
            .map(Some);
    }
    if entry == super::date_get_utc_minutes_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Minutes, true)
            .map(Some);
    }
    if entry == super::date_get_seconds_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Seconds, false)
            .map(Some);
    }
    if entry == super::date_get_utc_seconds_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Seconds, true)
            .map(Some);
    }
    if entry == super::date_get_milliseconds_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Milliseconds, false)
            .map(Some);
    }
    if entry == super::date_get_utc_milliseconds_builtin() {
        return date_get_component_builtin(context, invocation, DateComponent::Milliseconds, true)
            .map(Some);
    }
    if entry == super::date_get_timezone_offset_builtin() {
        return date_get_timezone_offset_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_date_setter_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::date_set_time_builtin() {
        return date_set_time_builtin(context, invocation).map(Some);
    }
    if entry == super::date_set_milliseconds_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Milliseconds, false)
            .map(Some);
    }
    if entry == super::date_set_utc_milliseconds_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Milliseconds, true)
            .map(Some);
    }
    if entry == super::date_set_seconds_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Seconds, false)
            .map(Some);
    }
    if entry == super::date_set_utc_seconds_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Seconds, true)
            .map(Some);
    }
    dispatch_date_setter_part_two(context, entry, invocation)
}

fn dispatch_date_setter_part_two<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::date_set_minutes_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Minutes, false)
            .map(Some);
    }
    if entry == super::date_set_utc_minutes_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Minutes, true)
            .map(Some);
    }
    if entry == super::date_set_hours_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Hours, false)
            .map(Some);
    }
    if entry == super::date_set_utc_hours_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Hours, true).map(Some);
    }
    if entry == super::date_set_date_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Date, false).map(Some);
    }
    if entry == super::date_set_utc_date_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Date, true).map(Some);
    }
    dispatch_date_setter_part_three(context, entry, invocation)
}

fn dispatch_date_setter_part_three<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::date_set_month_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Month, false)
            .map(Some);
    }
    if entry == super::date_set_utc_month_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::Month, true).map(Some);
    }
    if entry == super::date_set_full_year_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::FullYear, false)
            .map(Some);
    }
    if entry == super::date_set_year_builtin() {
        return date_set_year_builtin(context, invocation).map(Some);
    }
    if entry == super::date_set_utc_full_year_builtin() {
        return date_set_component_builtin(context, invocation, DateSetKind::FullYear, true)
            .map(Some);
    }
    Ok(None)
}

fn dispatch_date_conversion_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::date_to_utc_string_builtin() {
        return date_to_utc_string_builtin(context, invocation).map(Some);
    }
    if entry == super::date_to_iso_string_builtin() {
        return date_to_iso_string_builtin(context, invocation).map(Some);
    }
    if entry == super::date_to_json_builtin() {
        return date_to_json_builtin(context, invocation).map(Some);
    }
    if entry == super::date_to_temporal_instant_builtin() {
        return date_to_temporal_instant_builtin(context, invocation).map(Some);
    }
    if entry == super::date_to_primitive_builtin() {
        return date_to_primitive_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn allocate_date_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
    value: Value,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let root_shape = {
        let agent = cx.agent();
        agent.realm_root_shape(realm)
    }
    .ok_or_else(|| type_error(cx))?;
    Ok(cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_date_object(
            &mut mutator,
            root_shape,
            Some(prototype),
            value,
            AllocationLifetime::Default,
        )
    }))
}

fn current_time_value() -> Value {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map_or(f64::NAN, |duration| {
            date_u128_as_number(duration.as_millis())
        });
    Value::from_f64(millis)
}

fn date_display_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<String, Cx::Error> {
    date_format_local(cx, value, DateStringKind::Full)
}

const DATE_MS_PER_SECOND: i64 = 1_000;
const DATE_MS_PER_MINUTE: i64 = 60 * DATE_MS_PER_SECOND;
const DATE_MS_PER_HOUR: i64 = 60 * DATE_MS_PER_MINUTE;
const DATE_MS_PER_DAY: i64 = 24 * DATE_MS_PER_HOUR;
const DATE_NANOS_PER_MILLISECOND: i128 = 1_000_000;
const DATE_NANOS_PER_MINUTE: i64 = 60 * 1_000_000_000;
const DATE_CLIP_LIMIT_MS: f64 = 8_640_000_000_000_000.0;
const DATE_WEEKDAY_NAMES: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const DATE_MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

const fn date_i64_as_number(value: i64) -> f64 {
    #[allow(
        clippy::cast_precision_loss,
        reason = "ECMAScript Date stores time values in the Number type"
    )]
    let number = value as f64;
    number
}

const fn date_i128_as_number(value: i128) -> f64 {
    #[allow(
        clippy::cast_precision_loss,
        reason = "ECMAScript Date exposes host nanosecond calculations as Number milliseconds"
    )]
    let number = value as f64;
    number
}

const fn date_u128_as_number(value: u128) -> f64 {
    #[allow(
        clippy::cast_precision_loss,
        reason = "SystemTime milliseconds are exposed through ECMAScript Date Number values"
    )]
    let number = value as f64;
    number
}

const fn date_number_to_i64_after_range_check(value: f64) -> i64 {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "caller validates the finite Date Number range before narrowing"
    )]
    let integer = value as i64;
    integer
}

const fn date_number_to_i128_after_range_check(value: f64) -> i128 {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "caller validates the finite Date Number range before converting to nanoseconds"
    )]
    let integer = value as i128;
    integer
}

#[derive(Clone, Copy, Debug)]
struct DateParts {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    millisecond: u16,
    weekday: u8,
    offset_minutes: i32,
}

#[derive(Clone, Copy, Debug)]
enum DateStringKind {
    Full,
    Date,
    Time,
}

#[derive(Clone, Copy, Debug)]
enum DateComponent {
    FullYear,
    Month,
    Date,
    Day,
    Hours,
    Minutes,
    Seconds,
    Milliseconds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DateSetKind {
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Date,
    Month,
    FullYear,
}

fn date_number_argument<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Option<Value>,
    default: f64,
) -> Result<f64, Cx::Error> {
    value.map_or(Ok(default), |value| to_number_for_builtin(cx, value))
}

fn date_finite_integer(value: f64) -> Option<i64> {
    if !value.is_finite()
        || value < date_i64_as_number(i64::MIN)
        || value > date_i64_as_number(i64::MAX)
    {
        return None;
    }
    Some(date_number_to_i64_after_range_check(value.trunc()))
}

fn date_time_clip_value(time: f64) -> Value {
    if !time.is_finite() || time.abs() > DATE_CLIP_LIMIT_MS {
        return Value::from_f64(f64::NAN);
    }
    Value::from_f64(time.trunc() + 0.0)
}

fn date_apply_legacy_year_offset(year: f64) -> f64 {
    if !year.is_finite() {
        return year;
    }
    let integer = year.trunc();
    if (0.0..=99.0).contains(&integer) {
        1900.0 + integer
    } else {
        year
    }
}

fn date_balance_year_month(year: i64, month: i64) -> Option<(i32, u8)> {
    let balanced_year = year.checked_add(month.div_euclid(12))?;
    let balanced_month = month.rem_euclid(12) + 1;
    Some((
        i32::try_from(balanced_year).ok()?,
        u8::try_from(balanced_month).ok()?,
    ))
}

const fn date_is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

const fn date_days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if date_is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn date_days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let year = i64::from(year) - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month_prime = i64::from(month) + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + i64::from(day) - 1;
    let day_of_era = yoe * 365 + yoe / 4 - yoe / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn date_civil_from_days(days_since_epoch: i64) -> Option<(i32, u8, u8)> {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + i64::from(month <= 2);
    Some((
        i32::try_from(year).ok()?,
        u8::try_from(month).ok()?,
        u8::try_from(day).ok()?,
    ))
}

fn date_split_time_millis(total_millis: i64) -> (u8, u8, u8, u16) {
    let hour = total_millis / DATE_MS_PER_HOUR;
    let minute = (total_millis % DATE_MS_PER_HOUR) / DATE_MS_PER_MINUTE;
    let second = (total_millis % DATE_MS_PER_MINUTE) / DATE_MS_PER_SECOND;
    let millisecond = total_millis % DATE_MS_PER_SECOND;
    (
        u8::try_from(hour).unwrap(),
        u8::try_from(minute).unwrap(),
        u8::try_from(second).unwrap(),
        u16::try_from(millisecond).unwrap(),
    )
}

fn date_weekday_from_days(days_since_epoch: i64) -> u8 {
    u8::try_from((days_since_epoch + 4).rem_euclid(7)).unwrap()
}

fn date_value_epoch_nanoseconds(value: Value) -> Option<i128> {
    let millis = value.as_f64()?;
    if !millis.is_finite() {
        return None;
    }
    Some(date_number_to_i128_after_range_check(millis.trunc()) * DATE_NANOS_PER_MILLISECOND)
}

fn date_make_day(year: f64, month: f64, date: f64) -> Option<i64> {
    let year = date_finite_integer(year)?;
    let month = date_finite_integer(month)?;
    let date = date_finite_integer(date)?;
    let (year, month) = date_balance_year_month(year, month)?;
    date_days_from_civil(year, month, 1).checked_add(date - 1)
}

#[allow(
    clippy::suboptimal_flops,
    reason = "ECMA-262 MakeTime requires ordinary Number multiplication and addition order, not fused arithmetic"
)]
fn date_make_time(hour: f64, minute: f64, second: f64, millisecond: f64) -> Option<f64> {
    if !hour.is_finite() || !minute.is_finite() || !second.is_finite() || !millisecond.is_finite() {
        return None;
    }
    let hour_millis = hour.trunc() * date_i64_as_number(DATE_MS_PER_HOUR);
    let minute_millis = minute.trunc() * date_i64_as_number(DATE_MS_PER_MINUTE);
    let second_millis = second.trunc() * date_i64_as_number(DATE_MS_PER_SECOND);
    Some(((hour_millis + minute_millis) + second_millis) + millisecond.trunc())
}

#[allow(
    clippy::suboptimal_flops,
    reason = "ECMA-262 MakeDate requires ordinary Number multiplication and addition order, not fused arithmetic"
)]
fn date_make_utc_value(
    year: f64,
    month: f64,
    date: f64,
    hour: f64,
    minute: f64,
    second: f64,
    millisecond: f64,
) -> Value {
    let Some(day) = date_make_day(year, month, date) else {
        return Value::from_f64(f64::NAN);
    };
    let Some(time) = date_make_time(hour, minute, second, millisecond) else {
        return Value::from_f64(f64::NAN);
    };
    date_time_clip_value(date_i64_as_number(day) * date_i64_as_number(DATE_MS_PER_DAY) + time)
}

#[allow(
    clippy::too_many_arguments,
    reason = "Date construction receives the explicit ECMA MakeDate/MakeTime fields"
)]
fn date_make_local_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    year: f64,
    month: f64,
    date: f64,
    hour: f64,
    minute: f64,
    second: f64,
    millisecond: f64,
) -> Result<Value, Cx::Error> {
    let Some(base_day) = date_make_day(year, month, date) else {
        return Ok(Value::from_f64(f64::NAN));
    };
    let Some(time) = date_make_time(hour, minute, second, millisecond) else {
        return Ok(Value::from_f64(f64::NAN));
    };
    let Some(time_millis) = date_finite_integer(time) else {
        return Ok(Value::from_f64(f64::NAN));
    };
    let Some(day) = base_day.checked_add(time_millis.div_euclid(DATE_MS_PER_DAY)) else {
        return Ok(Value::from_f64(f64::NAN));
    };
    let Some((year, month, day_of_month)) = date_civil_from_days(day) else {
        return Ok(Value::from_f64(f64::NAN));
    };
    let (hour, minute, second, millisecond) =
        date_split_time_millis(time_millis.rem_euclid(DATE_MS_PER_DAY));
    let time_zone = cx.temporal_default_time_zone(&TemporalDefaultTimeZoneRequest {})?;
    let instant = cx.temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
        time_zone_id: time_zone.time_zone_id,
        date_time: TemporalCivilDateTime::new(
            year,
            month,
            day_of_month,
            hour,
            minute,
            second,
            millisecond,
            0,
            0,
        ),
        disambiguation: TemporalDisambiguation::Compatible,
    })?;
    Ok(date_time_clip_value(date_i128_as_number(
        instant.epoch_nanoseconds / DATE_NANOS_PER_MILLISECOND,
    )))
}

fn date_local_time_value_from_arguments<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    arguments: &[Value],
) -> Result<Value, Cx::Error> {
    let year = date_apply_legacy_year_offset(date_number_argument(
        cx,
        arguments.first().copied(),
        f64::NAN,
    )?);
    let month = date_number_argument(cx, arguments.get(1).copied(), f64::NAN)?;
    let date = date_number_argument(cx, arguments.get(2).copied(), 1.0)?;
    let hour = date_number_argument(cx, arguments.get(3).copied(), 0.0)?;
    let minute = date_number_argument(cx, arguments.get(4).copied(), 0.0)?;
    let second = date_number_argument(cx, arguments.get(5).copied(), 0.0)?;
    let millisecond = date_number_argument(cx, arguments.get(6).copied(), 0.0)?;
    date_make_local_value(cx, year, month, date, hour, minute, second, millisecond)
}

fn date_utc_time_value_from_arguments<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    arguments: &[Value],
) -> Result<Value, Cx::Error> {
    let year = date_apply_legacy_year_offset(date_number_argument(
        cx,
        arguments.first().copied(),
        f64::NAN,
    )?);
    let month = date_number_argument(cx, arguments.get(1).copied(), 0.0)?;
    let date = date_number_argument(cx, arguments.get(2).copied(), 1.0)?;
    let hour = date_number_argument(cx, arguments.get(3).copied(), 0.0)?;
    let minute = date_number_argument(cx, arguments.get(4).copied(), 0.0)?;
    let second = date_number_argument(cx, arguments.get(5).copied(), 0.0)?;
    let millisecond = date_number_argument(cx, arguments.get(6).copied(), 0.0)?;
    Ok(date_make_utc_value(
        year,
        month,
        date,
        hour,
        minute,
        second,
        millisecond,
    ))
}

fn date_utc_parts_from_millis(millis: f64) -> Option<DateParts> {
    let millis = date_finite_integer(millis)?;
    let day = millis.div_euclid(DATE_MS_PER_DAY);
    let time = millis.rem_euclid(DATE_MS_PER_DAY);
    let (year, month, date) = date_civil_from_days(day)?;
    let (hour, minute, second, millisecond) = date_split_time_millis(time);
    Some(DateParts {
        year,
        month,
        day: date,
        hour,
        minute,
        second,
        millisecond,
        weekday: date_weekday_from_days(day),
        offset_minutes: 0,
    })
}

fn date_local_parts_from_millis<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    millis: f64,
) -> Result<Option<DateParts>, Cx::Error> {
    let Some(epoch_nanoseconds) = date_value_epoch_nanoseconds(Value::from_f64(millis)) else {
        return Ok(None);
    };
    let time_zone = cx.temporal_default_time_zone(&TemporalDefaultTimeZoneRequest {})?;
    let civil_time = cx.temporal_instant_to_civil_time(&TemporalInstantToCivilRequest {
        time_zone_id: time_zone.time_zone_id,
        epoch_nanoseconds,
    })?;
    let date_time = civil_time.date_time;
    let day = date_days_from_civil(date_time.year, date_time.month, date_time.day);
    Ok(Some(DateParts {
        year: date_time.year,
        month: date_time.month,
        day: date_time.day,
        hour: date_time.hour,
        minute: date_time.minute,
        second: date_time.second,
        millisecond: date_time.millisecond,
        weekday: date_weekday_from_days(day),
        offset_minutes: i32::try_from(civil_time.offset_nanoseconds / DATE_NANOS_PER_MINUTE)
            .unwrap_or(0),
    }))
}

fn date_parts_for_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    utc: bool,
) -> Result<Option<DateParts>, Cx::Error> {
    let Some(millis) = value.as_f64().filter(|millis| millis.is_finite()) else {
        return Ok(None);
    };
    if utc {
        Ok(date_utc_parts_from_millis(millis))
    } else {
        date_local_parts_from_millis(cx, millis)
    }
}

fn date_format_year_for_date_string(year: i32) -> String {
    if (0..=9999).contains(&year) {
        format!("{year:04}")
    } else if year < 0 && year > -10_000 {
        format!("-{:04}", year.abs())
    } else {
        year.to_string()
    }
}

fn date_format_year_for_iso(year: i32) -> String {
    if (0..=9999).contains(&year) {
        format!("{year:04}")
    } else if year < 0 {
        format!("-{:06}", year.abs())
    } else {
        format!("+{year:06}")
    }
}

fn date_format_date(parts: DateParts) -> String {
    format!(
        "{} {} {:02} {}",
        DATE_WEEKDAY_NAMES[usize::from(parts.weekday)],
        DATE_MONTH_NAMES[usize::from(parts.month - 1)],
        parts.day,
        date_format_year_for_date_string(parts.year)
    )
}

fn date_format_time(parts: DateParts) -> String {
    let offset = parts.offset_minutes;
    let sign = if offset < 0 { '-' } else { '+' };
    let abs_offset = offset.abs();
    format!(
        "{:02}:{:02}:{:02} GMT{}{:02}{:02}",
        parts.hour,
        parts.minute,
        parts.second,
        sign,
        abs_offset / 60,
        abs_offset % 60
    )
}

fn date_format_local<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    kind: DateStringKind,
) -> Result<String, Cx::Error> {
    let Some(parts) = date_parts_for_value(cx, value, false)? else {
        return Ok("Invalid Date".to_owned());
    };
    Ok(match kind {
        DateStringKind::Full => format!("{} {}", date_format_date(parts), date_format_time(parts)),
        DateStringKind::Date => date_format_date(parts),
        DateStringKind::Time => date_format_time(parts),
    })
}

fn date_format_utc(value: Value) -> String {
    let Some(millis) = value.as_f64().filter(|millis| millis.is_finite()) else {
        return "Invalid Date".to_owned();
    };
    let Some(parts) = date_utc_parts_from_millis(millis) else {
        return "Invalid Date".to_owned();
    };
    format!(
        "{}, {:02} {} {} {:02}:{:02}:{:02} GMT",
        DATE_WEEKDAY_NAMES[usize::from(parts.weekday)],
        parts.day,
        DATE_MONTH_NAMES[usize::from(parts.month - 1)],
        date_format_year_for_date_string(parts.year),
        parts.hour,
        parts.minute,
        parts.second
    )
}

fn date_format_iso(value: Value) -> Option<String> {
    let millis = value.as_f64().filter(|millis| millis.is_finite())?;
    let parts = date_utc_parts_from_millis(millis)?;
    Some(format!(
        "{}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        date_format_year_for_iso(parts.year),
        parts.month,
        parts.day,
        parts.hour,
        parts.minute,
        parts.second,
        parts.millisecond
    ))
}

fn date_this_object_and_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<(ObjectRef, Value), Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let date_value = {
        let agent = cx.agent();
        if !agent.objects().is_date_object(object) {
            return Err(type_error(cx));
        }
        agent.objects().date_value(agent.heap().view(), object)
    };
    Ok((object, date_value.ok_or_else(|| type_error(cx))?))
}

fn date_store_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    value: Value,
) -> Result<(), Cx::Error> {
    let stored = cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.set_date_value(&mut mutator, object, value)
    });
    if stored {
        Ok(())
    } else {
        Err(type_error(cx))
    }
}

fn date_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.new_target().is_none() {
        let text = date_display_text(cx, current_time_value())?;
        return Ok(string_value(cx, &text));
    }

    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm_intrinsics(realm)
            .and_then(lyng_js_env::Intrinsics::date_prototype)
    }
    .ok_or_else(|| type_error(cx))?;

    let time_value = if invocation.arguments().is_empty() {
        current_time_value()
    } else if invocation.arguments().len() == 1 {
        let argument = invocation.arguments()[0];
        if let Some(object) = argument.as_object_ref() {
            let date_value = {
                let agent = cx.agent();
                agent.objects().date_value(agent.heap().view(), object)
            };
            if let Some(date_value) = date_value {
                date_value
            } else {
                let primitive = {
                    let mut bridge = BuiltinToPrimitiveBridge { cx };
                    object::to_primitive(&mut bridge, argument, object::ToPrimitiveHint::Default)?
                };
                if primitive.is_string() {
                    let text = cx.value_to_string_text(primitive)?;
                    date_parse_text(cx, &text)?
                } else {
                    let number = {
                        let agent = cx.agent();
                        read::to_number(agent.heap().view(), primitive)
                    };
                    match number {
                        Ok(number) => date_time_clip_value(number.as_f64().unwrap_or(f64::NAN)),
                        Err(_) => return Err(type_error(cx)),
                    }
                }
            }
        } else if argument.is_string() {
            let text = cx.value_to_string_text(argument)?;
            date_parse_text(cx, &text)?
        } else {
            let number = to_number_for_builtin(cx, argument)?;
            date_time_clip_value(number)
        }
    } else {
        date_local_time_value_from_arguments(cx, invocation.arguments())?
    };

    let prototype =
        cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?;
    Ok(Value::from_object_ref(allocate_date_object(
        cx, realm, prototype, time_value,
    )?))
}

fn date_now_value() -> Value {
    current_time_value()
}

fn date_parse_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let text = cx.value_to_string_text(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    date_parse_text(cx, &text)
}

fn date_utc_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    date_utc_time_value_from_arguments(cx, invocation.arguments())
}

fn date_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: DateStringKind,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_date_value(agent, invocation.this_value())
    };
    let value = map_completion(cx, value)?;
    let text = date_format_local(cx, value, kind)?;
    Ok(string_value(cx, &text))
}

fn date_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_date_value(agent, invocation.this_value())
    };
    map_completion(cx, value)
}

fn date_get_component_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    component: DateComponent,
    utc: bool,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_date_value(agent, invocation.this_value())
    };
    let value = map_completion(cx, value)?;
    let Some(parts) = date_parts_for_value(cx, value, utc)? else {
        return Ok(Value::from_f64(f64::NAN));
    };
    let value = match component {
        DateComponent::FullYear => parts.year,
        DateComponent::Month => i32::from(parts.month - 1),
        DateComponent::Date => i32::from(parts.day),
        DateComponent::Day => i32::from(parts.weekday),
        DateComponent::Hours => i32::from(parts.hour),
        DateComponent::Minutes => i32::from(parts.minute),
        DateComponent::Seconds => i32::from(parts.second),
        DateComponent::Milliseconds => i32::from(parts.millisecond),
    };
    Ok(Value::from_smi(value))
}

fn date_get_year_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_date_value(agent, invocation.this_value())
    };
    let value = map_completion(cx, value)?;
    let Some(parts) = date_parts_for_value(cx, value, false)? else {
        return Ok(Value::from_f64(f64::NAN));
    };
    Ok(Value::from_smi(parts.year - 1900))
}

fn date_get_timezone_offset_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_date_value(agent, invocation.this_value())
    };
    let value = map_completion(cx, value)?;
    let Some(epoch_nanoseconds) = date_value_epoch_nanoseconds(value) else {
        return Ok(Value::from_f64(f64::NAN));
    };
    let time_zone_request = TemporalDefaultTimeZoneRequest {};
    if cx.temporal_default_time_zone_is_utc(&time_zone_request)? {
        return Ok(Value::from_smi(0));
    }
    let time_zone = cx.temporal_default_time_zone(&time_zone_request)?;
    let civil_time = cx.temporal_instant_to_civil_time(&TemporalInstantToCivilRequest {
        time_zone_id: time_zone.time_zone_id,
        epoch_nanoseconds,
    })?;
    let offset_minutes = -date_i64_as_number(civil_time.offset_nanoseconds / DATE_NANOS_PER_MINUTE);
    Ok(Value::from_f64(offset_minutes + 0.0))
}

fn date_set_time_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (object, _) = date_this_object_and_value(cx, invocation.this_value())?;
    let time = to_number_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let value = date_time_clip_value(time);
    date_store_value(cx, object, value)?;
    Ok(value)
}

fn date_set_component_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: DateSetKind,
    utc: bool,
) -> Result<Value, Cx::Error> {
    let (object, old_value) = date_this_object_and_value(cx, invocation.this_value())?;
    let old_millis = old_value.as_f64().unwrap_or(f64::NAN);
    let first = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let first_number = to_number_for_builtin(cx, first)?;
    let second_number = match invocation.arguments().get(1).copied() {
        Some(value) => Some(to_number_for_builtin(cx, value)?),
        None => None,
    };
    let third_number = match invocation.arguments().get(2).copied() {
        Some(value) => Some(to_number_for_builtin(cx, value)?),
        None => None,
    };
    let fourth_number = match invocation.arguments().get(3).copied() {
        Some(value) => Some(to_number_for_builtin(cx, value)?),
        None => None,
    };

    let base_millis = if kind == DateSetKind::FullYear && old_millis.is_nan() {
        0.0
    } else {
        old_millis
    };
    if kind != DateSetKind::FullYear && !base_millis.is_finite() {
        return Ok(Value::from_f64(f64::NAN));
    }
    let parts = if utc {
        date_utc_parts_from_millis(base_millis)
    } else {
        date_local_parts_from_millis(cx, base_millis)?
    };
    let Some(parts) = parts else {
        return Ok(Value::from_f64(f64::NAN));
    };

    let mut year = f64::from(parts.year);
    let mut month = f64::from(parts.month - 1);
    let mut date = f64::from(parts.day);
    let mut hour = f64::from(parts.hour);
    let mut minute = f64::from(parts.minute);
    let mut second = f64::from(parts.second);
    let mut millisecond = f64::from(parts.millisecond);

    match kind {
        DateSetKind::Milliseconds => {
            millisecond = first_number;
        }
        DateSetKind::Seconds => {
            second = first_number;
            millisecond = second_number.unwrap_or(millisecond);
        }
        DateSetKind::Minutes => {
            minute = first_number;
            second = second_number.unwrap_or(second);
            millisecond = third_number.unwrap_or(millisecond);
        }
        DateSetKind::Hours => {
            hour = first_number;
            minute = second_number.unwrap_or(minute);
            second = third_number.unwrap_or(second);
            millisecond = fourth_number.unwrap_or(millisecond);
        }
        DateSetKind::Date => {
            date = first_number;
        }
        DateSetKind::Month => {
            month = first_number;
            date = second_number.unwrap_or(date);
        }
        DateSetKind::FullYear => {
            year = first_number;
            month = second_number.unwrap_or(month);
            date = third_number.unwrap_or(date);
        }
    }

    let value = if utc {
        date_make_utc_value(year, month, date, hour, minute, second, millisecond)
    } else {
        date_make_local_value(cx, year, month, date, hour, minute, second, millisecond)?
    };
    date_store_value(cx, object, value)?;
    Ok(value)
}

fn date_set_year_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (object, old_value) = date_this_object_and_value(cx, invocation.this_value())?;
    let year = to_number_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if year.is_nan() {
        let value = Value::from_f64(f64::NAN);
        date_store_value(cx, object, value)?;
        return Ok(value);
    }

    let parts = if old_value.as_f64().is_some_and(f64::is_nan) {
        date_utc_parts_from_millis(0.0)
    } else {
        let old_millis = old_value.as_f64().unwrap_or(f64::NAN);
        date_local_parts_from_millis(cx, old_millis)?
    };
    let Some(parts) = parts else {
        let value = Value::from_f64(f64::NAN);
        date_store_value(cx, object, value)?;
        return Ok(value);
    };

    let integer_year = year.trunc();
    let year = if (0.0..=99.0).contains(&integer_year) {
        1900.0 + integer_year
    } else {
        year
    };
    let value = date_make_local_value(
        cx,
        year,
        f64::from(parts.month - 1),
        f64::from(parts.day),
        f64::from(parts.hour),
        f64::from(parts.minute),
        f64::from(parts.second),
        f64::from(parts.millisecond),
    )?;
    date_store_value(cx, object, value)?;
    Ok(value)
}

fn date_to_utc_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_date_value(agent, invocation.this_value())
    };
    let value = map_completion(cx, value)?;
    Ok(string_value(cx, &date_format_utc(value)))
}

fn date_to_iso_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_date_value(agent, invocation.this_value())
    };
    let value = map_completion(cx, value)?;
    let Some(text) = date_format_iso(value) else {
        return Err(range_error(cx));
    };
    Ok(string_value(cx, &text))
}

fn date_to_json_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = {
        let realm = cx.builtin_realm();
        let agent = cx.agent();
        object::to_object(agent, realm, invocation.this_value())
    };
    let object = map_completion(cx, object)?;
    let primitive = {
        let mut bridge = BuiltinToPrimitiveBridge { cx };
        object::to_primitive(
            &mut bridge,
            Value::from_object_ref(object),
            object::ToPrimitiveHint::Number,
        )?
    };
    if primitive.as_f64().is_some_and(|number| !number.is_finite()) {
        return Ok(Value::null());
    }
    let key = {
        let agent = cx.agent();
        PropertyKey::from_atom(agent.atoms_mut().intern_collectible("toISOString"))
    };
    let method = cx.get_property_value(Value::from_object_ref(object), key)?;
    let method = cx.require_callable_object(method)?;
    cx.call_to_completion(method, Value::from_object_ref(object), &[])
}

fn date_to_temporal_instant_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_date_value(agent, invocation.this_value())
    };
    let value = map_completion(cx, value)?;
    let Some(epoch_nanoseconds) = date_value_epoch_nanoseconds(value) else {
        return Err(range_error(cx));
    };
    temporal::create_temporal_instant_object(cx, epoch_nanoseconds)
}

fn date_to_primitive_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let hint_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let hint = hint_value.as_string_ref().ok_or_else(|| type_error(cx))?;
    let hint_text = string_ref_text(cx, hint)?;
    let hint = match hint_text.as_str() {
        "string" | "default" => object::ToPrimitiveHint::String,
        "number" => object::ToPrimitiveHint::Number,
        _ => return Err(type_error(cx)),
    };
    object::ordinary_to_primitive(&mut BuiltinToPrimitiveBridge { cx }, object, hint)
}

mod duration;
mod instant;
mod now;
mod plain_date;
mod plain_date_time;
mod plain_month_day;
mod plain_time;
mod plain_year_month;
mod support;
mod zoned_date_time;

use duration::*;
use plain_date::*;
use plain_date_time::*;
use support::*;
use zoned_date_time::*;

use super::{
    map_completion, range_error, string_ref_text, string_value, to_bigint_for_builtin,
    to_number_for_builtin, to_string_string_ref, type_error, BuiltinToPrimitiveBridge,
    PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_common::{AtomId, WellKnownAtom};
use lyng_js_env::{Agent, RealmRecord};
use lyng_js_gc::{AllocationLifetime, BigIntSign};
use lyng_js_host::{
    TemporalCivilDateTime, TemporalCivilTime, TemporalCivilToInstantRequest,
    TemporalCurrentInstantRequest, TemporalDefaultTimeZoneRequest, TemporalDisambiguation,
    TemporalInstantToCivilRequest,
};
use lyng_js_objects::{
    ObjectAllocation, ObjectColdData, OrdinaryObjectData, TemporalDurationObjectData,
    TemporalInstantObjectData, TemporalObjectData, TemporalObjectKind, TemporalPlainDateObjectData,
    TemporalPlainDateTimeObjectData, TemporalPlainMonthDayObjectData, TemporalPlainTimeObjectData,
    TemporalPlainYearMonthObjectData, TemporalZonedDateTimeObjectData,
};
use lyng_js_ops::temporal::{
    duration_default_largest_exact_unit as temporal_duration_default_largest_exact_unit,
    duration_exact_unit_allows_largest_smallest as temporal_duration_exact_unit_allows_largest_smallest,
    duration_has_lower_than_month_units as temporal_duration_has_lower_than_month_units,
    duration_sign as temporal_duration_sign,
    duration_time_nanoseconds as temporal_duration_time_nanoseconds,
    duration_whole_days_from_time as temporal_duration_whole_days_from_time,
    format_duration as format_temporal_duration,
    format_duration_with_seconds_precision as format_temporal_duration_with_seconds_precision,
    format_offset as format_temporal_offset, format_plain_date as format_temporal_plain_date,
    format_plain_month_day as format_temporal_plain_month_day,
    format_plain_time as format_temporal_plain_time,
    format_plain_year_month as format_temporal_plain_year_month,
    instant_epoch_nanoseconds_is_valid as temporal_instant_epoch_nanoseconds_is_valid,
    is_iso_leap_year as temporal_is_iso_leap_year, iso_day_of_week as temporal_iso_day_of_week,
    iso_day_of_year as temporal_iso_day_of_year,
    iso_days_before_year as temporal_iso_days_before_year,
    iso_days_in_month as temporal_iso_days_in_month, iso_days_in_year as temporal_iso_days_in_year,
    iso_week_of_year as temporal_iso_week_of_year, negate_duration as negate_temporal_duration,
    parse_duration as parse_temporal_duration, parse_instant as parse_temporal_instant,
    parse_plain_date_time as parse_temporal_plain_date_time,
    parse_plain_month_day as parse_temporal_plain_month_day,
    parse_plain_time as parse_temporal_plain_time,
    parse_plain_year_month as parse_temporal_plain_year_month,
    plain_date_ordinal_day as temporal_plain_date_ordinal_day,
    plain_time_nanoseconds as temporal_plain_time_nanoseconds,
    round_duration_exact as temporal_round_duration_exact,
    round_duration_nanoseconds_to_increment as temporal_round_duration_nanoseconds_to_increment,
    round_epoch_nanoseconds_to_fractional_digits as temporal_round_epoch_nanoseconds_to_fractional_digits,
    round_epoch_nanoseconds_to_increment as temporal_round_epoch_nanoseconds_to_increment,
    total_duration_exact as temporal_total_duration_exact,
    total_nanoseconds_as_unit as temporal_total_nanoseconds_as_unit,
    TemporalDurationExactUnit as TemporalBuiltinDurationExactUnit,
    TemporalRoundingMode as TemporalBuiltinRoundingMode,
    DEFAULT_PLAIN_MONTH_DAY_REFERENCE_YEAR as TEMPORAL_DEFAULT_PLAIN_MONTH_DAY_REFERENCE_YEAR,
    INSTANT_EPOCH_MILLISECONDS_MAX as TEMPORAL_INSTANT_EPOCH_MILLISECONDS_MAX,
    NANOS_PER_DAY as TEMPORAL_NANOS_PER_DAY, NANOS_PER_HOUR as TEMPORAL_NANOS_PER_HOUR,
    NANOS_PER_MICROSECOND as TEMPORAL_NANOS_PER_MICROSECOND,
    NANOS_PER_MILLISECOND as TEMPORAL_NANOS_PER_MILLISECOND,
    NANOS_PER_MINUTE as TEMPORAL_NANOS_PER_MINUTE, NANOS_PER_SECOND as TEMPORAL_NANOS_PER_SECOND,
    SAFE_INTEGER_MAX as TEMPORAL_SAFE_INTEGER_MAX, UTC_TIME_ZONE_ID as TEMPORAL_UTC_TIME_ZONE_ID,
};
use lyng_js_ops::{object, temporal as temporal_ops};
use lyng_js_types::{BuiltinFunctionId, ObjectRef, PropertyKey, Value};
use std::fmt::Write as _;

pub(super) fn create_temporal_instant_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    epoch_nanoseconds: i128,
) -> Result<Value, Cx::Error> {
    instant::create_temporal_instant_object(cx, epoch_nanoseconds)
}

pub(super) fn dispatch_temporal_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = instant::dispatch_temporal_instant_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = now::dispatch_temporal_now_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = duration::dispatch_temporal_duration_builtin(context, entry, invocation)?
    {
        return Ok(Some(result));
    }
    if let Some(result) =
        plain_date::dispatch_temporal_plain_date_builtin(context, entry, invocation)?
    {
        return Ok(Some(result));
    }
    if let Some(result) =
        plain_time::dispatch_temporal_plain_time_builtin(context, entry, invocation)?
    {
        return Ok(Some(result));
    }
    if let Some(result) =
        plain_date_time::dispatch_temporal_plain_date_time_builtin(context, entry, invocation)?
    {
        return Ok(Some(result));
    }
    if let Some(result) =
        plain_year_month::dispatch_temporal_plain_year_month_builtin(context, entry, invocation)?
    {
        return Ok(Some(result));
    }
    if let Some(result) =
        plain_month_day::dispatch_temporal_plain_month_day_builtin(context, entry, invocation)?
    {
        return Ok(Some(result));
    }
    if let Some(result) =
        zoned_date_time::dispatch_temporal_zoned_date_time_builtin(context, entry, invocation)?
    {
        return Ok(Some(result));
    }
    Ok(None)
}

fn temporal_bigint_to_i128(agent: &Agent, value: Value) -> Option<i128> {
    let bigint = value.as_bigint_ref()?;
    let view = agent.heap().view().bigint_view(bigint)?;
    if view.limb_count() > 2 {
        return None;
    }
    let low = view.limb_at(0).unwrap_or(0);
    let high = view.limb_at(1).unwrap_or(0);
    let magnitude = u128::from(low) | (u128::from(high) << 64);
    match view.sign() {
        BigIntSign::NonNegative => i128::try_from(magnitude).ok(),
        BigIntSign::Negative => {
            if magnitude == (1_u128 << 127) {
                Some(i128::MIN)
            } else {
                i128::try_from(magnitude).ok().map(|value| -value)
            }
        }
    }
}

fn temporal_i128_to_bigint_value(agent: &mut Agent, value: i128) -> Value {
    let magnitude = value.unsigned_abs();
    let low = magnitude as u64;
    let high = (magnitude >> 64) as u64;
    let limbs = if high == 0 {
        vec![low]
    } else {
        vec![low, high]
    };
    let sign = if value < 0 {
        BigIntSign::Negative
    } else {
        BigIntSign::NonNegative
    };
    let bigint = agent
        .heap_mut()
        .mutator()
        .alloc_bigint(sign, &limbs, AllocationLifetime::Default);
    Value::from_bigint_ref(bigint)
}

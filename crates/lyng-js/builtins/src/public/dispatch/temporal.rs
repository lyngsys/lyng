use super::{
    map_completion, range_error, string_ref_text, string_value, to_bigint_for_builtin,
    to_number_for_builtin, to_string_string_ref, type_error, BuiltinToPrimitiveBridge,
    PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_common::{AtomId, WellKnownAtom};
use lyng_js_env::Agent;
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
    round_epoch_nanoseconds_to_fractional_digits as temporal_round_epoch_nanoseconds_to_fractional_digits,
    round_epoch_nanoseconds_to_increment as temporal_round_epoch_nanoseconds_to_increment,
    total_duration_exact as temporal_total_duration_exact,
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

pub(super) fn dispatch_temporal_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::js3_temporal_instant_builtin() {
        return temporal_instant_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_now_instant_builtin() {
        return temporal_now_instant_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_now_time_zone_id_builtin() {
        return temporal_now_time_zone_id_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_now_plain_date_iso_builtin() {
        return temporal_now_plain_date_iso_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_now_plain_time_iso_builtin() {
        return temporal_now_plain_time_iso_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_now_plain_date_time_iso_builtin() {
        return temporal_now_plain_date_time_iso_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_now_zoned_date_time_iso_builtin() {
        return temporal_now_zoned_date_time_iso_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_instant_from_builtin() {
        return temporal_instant_from_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_instant_from_epoch_nanoseconds_builtin() {
        return temporal_instant_from_epoch_nanoseconds_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_instant_from_epoch_milliseconds_builtin() {
        return temporal_instant_from_epoch_milliseconds_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_instant_compare_builtin() {
        return temporal_instant_compare_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_instant_epoch_nanoseconds_getter_builtin() {
        return temporal_instant_epoch_nanoseconds_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_instant_epoch_milliseconds_getter_builtin() {
        return temporal_instant_epoch_milliseconds_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_instant_epoch_seconds_getter_builtin() {
        return temporal_instant_epoch_seconds_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_instant_to_string_builtin() {
        return temporal_instant_to_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_instant_to_json_builtin() {
        return temporal_instant_to_json_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_instant_to_locale_string_builtin() {
        return temporal_instant_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_instant_value_of_builtin() {
        return temporal_instant_value_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_instant_equals_builtin() {
        return temporal_instant_equals_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_instant_add_builtin() {
        return temporal_instant_add_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_instant_subtract_builtin() {
        return temporal_instant_subtract_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_instant_round_builtin() {
        return temporal_instant_round_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_instant_since_builtin() {
        return temporal_instant_since_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_instant_until_builtin() {
        return temporal_instant_until_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_instant_to_zoned_date_time_iso_builtin() {
        return temporal_instant_to_zoned_date_time_iso_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_builtin() {
        return temporal_duration_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_years_getter_builtin() {
        return temporal_duration_years_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_months_getter_builtin() {
        return temporal_duration_months_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_weeks_getter_builtin() {
        return temporal_duration_weeks_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_days_getter_builtin() {
        return temporal_duration_days_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_hours_getter_builtin() {
        return temporal_duration_hours_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_minutes_getter_builtin() {
        return temporal_duration_minutes_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_seconds_getter_builtin() {
        return temporal_duration_seconds_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_milliseconds_getter_builtin() {
        return temporal_duration_milliseconds_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_microseconds_getter_builtin() {
        return temporal_duration_microseconds_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_nanoseconds_getter_builtin() {
        return temporal_duration_nanoseconds_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_sign_getter_builtin() {
        return temporal_duration_sign_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_blank_getter_builtin() {
        return temporal_duration_blank_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_to_string_builtin() {
        return temporal_duration_to_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_to_json_builtin() {
        return temporal_duration_to_json_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_to_locale_string_builtin() {
        return temporal_duration_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_negated_builtin() {
        return temporal_duration_negated_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_abs_builtin() {
        return temporal_duration_abs_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_with_builtin() {
        return temporal_duration_with_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_round_builtin() {
        return temporal_duration_round_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_total_builtin() {
        return temporal_duration_total_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_add_builtin() {
        return temporal_duration_add_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_subtract_builtin() {
        return temporal_duration_subtract_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_value_of_builtin() {
        return temporal_duration_value_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_from_builtin() {
        return temporal_duration_from_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_duration_compare_builtin() {
        return temporal_duration_compare_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_builtin() {
        return temporal_plain_date_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_year_getter_builtin() {
        return temporal_plain_date_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_month_getter_builtin() {
        return temporal_plain_date_month_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_month_code_getter_builtin() {
        return temporal_plain_date_month_code_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_day_getter_builtin() {
        return temporal_plain_date_day_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_day_of_week_getter_builtin() {
        return temporal_plain_date_day_of_week_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_day_of_year_getter_builtin() {
        return temporal_plain_date_day_of_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_days_in_month_getter_builtin() {
        return temporal_plain_date_days_in_month_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_days_in_year_getter_builtin() {
        return temporal_plain_date_days_in_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_months_in_year_getter_builtin() {
        return temporal_plain_date_months_in_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_in_leap_year_getter_builtin() {
        return temporal_plain_date_in_leap_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_days_in_week_getter_builtin() {
        return temporal_plain_date_days_in_week_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_week_of_year_getter_builtin() {
        return temporal_plain_date_week_of_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_year_of_week_getter_builtin() {
        return temporal_plain_date_year_of_week_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_era_getter_builtin() {
        return temporal_plain_date_era_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_era_year_getter_builtin() {
        return temporal_plain_date_era_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_calendar_id_getter_builtin() {
        return temporal_plain_date_calendar_id_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_to_string_builtin() {
        return temporal_plain_date_to_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_to_json_builtin() {
        return temporal_plain_date_to_json_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_to_locale_string_builtin() {
        return temporal_plain_date_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_value_of_builtin() {
        return temporal_plain_date_value_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_equals_builtin() {
        return temporal_plain_date_equals_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_with_builtin() {
        return temporal_plain_date_with_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_with_calendar_builtin() {
        return temporal_plain_date_with_calendar_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_add_builtin() {
        return temporal_plain_date_add_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_subtract_builtin() {
        return temporal_plain_date_subtract_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_since_builtin() {
        return temporal_plain_date_since_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_until_builtin() {
        return temporal_plain_date_until_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_to_plain_date_time_builtin() {
        return temporal_plain_date_to_plain_date_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_to_zoned_date_time_builtin() {
        return temporal_plain_date_to_zoned_date_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_to_plain_year_month_builtin() {
        return temporal_plain_date_to_plain_year_month_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_to_plain_month_day_builtin() {
        return temporal_plain_date_to_plain_month_day_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_from_builtin() {
        return temporal_plain_date_from_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_compare_builtin() {
        return temporal_plain_date_compare_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_builtin() {
        return temporal_plain_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_hour_getter_builtin() {
        return temporal_plain_time_hour_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_minute_getter_builtin() {
        return temporal_plain_time_minute_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_second_getter_builtin() {
        return temporal_plain_time_second_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_millisecond_getter_builtin() {
        return temporal_plain_time_millisecond_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_microsecond_getter_builtin() {
        return temporal_plain_time_microsecond_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_nanosecond_getter_builtin() {
        return temporal_plain_time_nanosecond_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_to_string_builtin() {
        return temporal_plain_time_to_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_to_json_builtin() {
        return temporal_plain_time_to_json_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_to_locale_string_builtin() {
        return temporal_plain_time_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_value_of_builtin() {
        return temporal_plain_time_value_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_equals_builtin() {
        return temporal_plain_time_equals_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_with_builtin() {
        return temporal_plain_time_with_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_add_builtin() {
        return temporal_plain_time_add_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_subtract_builtin() {
        return temporal_plain_time_subtract_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_round_builtin() {
        return temporal_plain_time_round_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_since_builtin() {
        return temporal_plain_time_since_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_until_builtin() {
        return temporal_plain_time_until_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_to_plain_date_time_builtin() {
        return temporal_plain_time_to_plain_date_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_from_builtin() {
        return temporal_plain_time_from_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_time_compare_builtin() {
        return temporal_plain_time_compare_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_builtin() {
        return temporal_plain_date_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_year_getter_builtin() {
        return temporal_plain_date_time_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_month_getter_builtin() {
        return temporal_plain_date_time_month_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_month_code_getter_builtin() {
        return temporal_plain_date_time_month_code_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_day_getter_builtin() {
        return temporal_plain_date_time_day_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_day_of_week_getter_builtin() {
        return temporal_plain_date_time_day_of_week_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_day_of_year_getter_builtin() {
        return temporal_plain_date_time_day_of_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_days_in_month_getter_builtin() {
        return temporal_plain_date_time_days_in_month_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_days_in_year_getter_builtin() {
        return temporal_plain_date_time_days_in_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_months_in_year_getter_builtin() {
        return temporal_plain_date_time_months_in_year_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_in_leap_year_getter_builtin() {
        return temporal_plain_date_time_in_leap_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_days_in_week_getter_builtin() {
        return temporal_plain_date_time_days_in_week_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_week_of_year_getter_builtin() {
        return temporal_plain_date_time_week_of_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_year_of_week_getter_builtin() {
        return temporal_plain_date_time_year_of_week_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_era_getter_builtin() {
        return temporal_plain_date_time_era_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_era_year_getter_builtin() {
        return temporal_plain_date_time_era_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_hour_getter_builtin() {
        return temporal_plain_date_time_hour_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_minute_getter_builtin() {
        return temporal_plain_date_time_minute_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_second_getter_builtin() {
        return temporal_plain_date_time_second_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_millisecond_getter_builtin() {
        return temporal_plain_date_time_millisecond_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_microsecond_getter_builtin() {
        return temporal_plain_date_time_microsecond_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_nanosecond_getter_builtin() {
        return temporal_plain_date_time_nanosecond_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_calendar_id_getter_builtin() {
        return temporal_plain_date_time_calendar_id_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_to_string_builtin() {
        return temporal_plain_date_time_to_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_to_json_builtin() {
        return temporal_plain_date_time_to_json_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_to_locale_string_builtin() {
        return temporal_plain_date_time_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_value_of_builtin() {
        return temporal_plain_date_time_value_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_equals_builtin() {
        return temporal_plain_date_time_equals_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_with_builtin() {
        return temporal_plain_date_time_with_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_with_plain_time_builtin() {
        return temporal_plain_date_time_with_plain_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_with_calendar_builtin() {
        return temporal_plain_date_time_with_calendar_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_add_builtin() {
        return temporal_plain_date_time_add_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_subtract_builtin() {
        return temporal_plain_date_time_subtract_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_round_builtin() {
        return temporal_plain_date_time_round_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_since_builtin() {
        return temporal_plain_date_time_since_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_until_builtin() {
        return temporal_plain_date_time_until_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_to_plain_date_builtin() {
        return temporal_plain_date_time_to_plain_date_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_to_plain_time_builtin() {
        return temporal_plain_date_time_to_plain_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_to_zoned_date_time_builtin() {
        return temporal_plain_date_time_to_zoned_date_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_from_builtin() {
        return temporal_plain_date_time_from_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_compare_builtin() {
        return temporal_plain_date_time_compare_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_builtin() {
        return temporal_plain_year_month_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_year_getter_builtin() {
        return temporal_plain_year_month_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_month_getter_builtin() {
        return temporal_plain_year_month_month_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_month_code_getter_builtin() {
        return temporal_plain_year_month_month_code_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_days_in_month_getter_builtin() {
        return temporal_plain_year_month_days_in_month_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_days_in_year_getter_builtin() {
        return temporal_plain_year_month_days_in_year_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_months_in_year_getter_builtin() {
        return temporal_plain_year_month_months_in_year_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_in_leap_year_getter_builtin() {
        return temporal_plain_year_month_in_leap_year_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_era_getter_builtin() {
        return temporal_plain_year_month_era_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_era_year_getter_builtin() {
        return temporal_plain_year_month_era_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_calendar_id_getter_builtin() {
        return temporal_plain_year_month_calendar_id_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_to_string_builtin() {
        return temporal_plain_year_month_to_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_to_json_builtin() {
        return temporal_plain_year_month_to_json_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_to_locale_string_builtin() {
        return temporal_plain_year_month_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_value_of_builtin() {
        return temporal_plain_year_month_value_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_equals_builtin() {
        return temporal_plain_year_month_equals_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_with_builtin() {
        return temporal_plain_year_month_with_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_add_builtin() {
        return temporal_plain_year_month_add_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_subtract_builtin() {
        return temporal_plain_year_month_subtract_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_since_builtin() {
        return temporal_plain_year_month_since_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_until_builtin() {
        return temporal_plain_year_month_until_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_to_plain_date_builtin() {
        return temporal_plain_year_month_to_plain_date_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_from_builtin() {
        return temporal_plain_year_month_from_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_compare_builtin() {
        return temporal_plain_year_month_compare_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_builtin() {
        return temporal_plain_month_day_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_month_code_getter_builtin() {
        return temporal_plain_month_day_month_code_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_day_getter_builtin() {
        return temporal_plain_month_day_day_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_calendar_id_getter_builtin() {
        return temporal_plain_month_day_calendar_id_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_to_string_builtin() {
        return temporal_plain_month_day_to_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_to_json_builtin() {
        return temporal_plain_month_day_to_json_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_to_locale_string_builtin() {
        return temporal_plain_month_day_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_value_of_builtin() {
        return temporal_plain_month_day_value_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_equals_builtin() {
        return temporal_plain_month_day_equals_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_with_builtin() {
        return temporal_plain_month_day_with_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_to_plain_date_builtin() {
        return temporal_plain_month_day_to_plain_date_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_from_builtin() {
        return temporal_plain_month_day_from_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_builtin() {
        return temporal_zoned_date_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_year_getter_builtin() {
        return temporal_zoned_date_time_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_month_getter_builtin() {
        return temporal_zoned_date_time_month_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_month_code_getter_builtin() {
        return temporal_zoned_date_time_month_code_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_day_getter_builtin() {
        return temporal_zoned_date_time_day_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_day_of_week_getter_builtin() {
        return temporal_zoned_date_time_day_of_week_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_day_of_year_getter_builtin() {
        return temporal_zoned_date_time_day_of_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_days_in_month_getter_builtin() {
        return temporal_zoned_date_time_days_in_month_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_days_in_year_getter_builtin() {
        return temporal_zoned_date_time_days_in_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_months_in_year_getter_builtin() {
        return temporal_zoned_date_time_months_in_year_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_in_leap_year_getter_builtin() {
        return temporal_zoned_date_time_in_leap_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_days_in_week_getter_builtin() {
        return temporal_zoned_date_time_days_in_week_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_week_of_year_getter_builtin() {
        return temporal_zoned_date_time_week_of_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_year_of_week_getter_builtin() {
        return temporal_zoned_date_time_year_of_week_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_era_getter_builtin() {
        return temporal_zoned_date_time_era_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_era_year_getter_builtin() {
        return temporal_zoned_date_time_era_year_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_hour_getter_builtin() {
        return temporal_zoned_date_time_hour_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_minute_getter_builtin() {
        return temporal_zoned_date_time_minute_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_second_getter_builtin() {
        return temporal_zoned_date_time_second_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_millisecond_getter_builtin() {
        return temporal_zoned_date_time_millisecond_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_microsecond_getter_builtin() {
        return temporal_zoned_date_time_microsecond_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_nanosecond_getter_builtin() {
        return temporal_zoned_date_time_nanosecond_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_epoch_nanoseconds_getter_builtin() {
        return temporal_zoned_date_time_epoch_nanoseconds_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_epoch_milliseconds_getter_builtin() {
        return temporal_zoned_date_time_epoch_milliseconds_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_time_zone_id_getter_builtin() {
        return temporal_zoned_date_time_time_zone_id_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_calendar_id_getter_builtin() {
        return temporal_zoned_date_time_calendar_id_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_offset_getter_builtin() {
        return temporal_zoned_date_time_offset_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_offset_nanoseconds_getter_builtin() {
        return temporal_zoned_date_time_offset_nanoseconds_getter_builtin(context, invocation)
            .map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_to_string_builtin() {
        return temporal_zoned_date_time_to_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_to_json_builtin() {
        return temporal_zoned_date_time_to_json_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_to_locale_string_builtin() {
        return temporal_zoned_date_time_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_value_of_builtin() {
        return temporal_zoned_date_time_value_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_equals_builtin() {
        return temporal_zoned_date_time_equals_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_add_builtin() {
        return temporal_zoned_date_time_add_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_round_builtin() {
        return temporal_zoned_date_time_round_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_with_builtin() {
        return temporal_zoned_date_time_with_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_subtract_builtin() {
        return temporal_zoned_date_time_subtract_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_with_time_zone_builtin() {
        return temporal_zoned_date_time_with_time_zone_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_with_calendar_builtin() {
        return temporal_zoned_date_time_with_calendar_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_with_plain_time_builtin() {
        return temporal_zoned_date_time_with_plain_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_start_of_day_builtin() {
        return temporal_zoned_date_time_start_of_day_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_hours_in_day_getter_builtin() {
        return temporal_zoned_date_time_hours_in_day_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_since_builtin() {
        return temporal_zoned_date_time_since_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_until_builtin() {
        return temporal_zoned_date_time_until_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_from_builtin() {
        return temporal_zoned_date_time_from_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_compare_builtin() {
        return temporal_zoned_date_time_compare_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_to_instant_builtin() {
        return temporal_zoned_date_time_to_instant_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_to_plain_date_time_builtin() {
        return temporal_zoned_date_time_to_plain_date_time_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_to_plain_date_builtin() {
        return temporal_zoned_date_time_to_plain_date_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_to_plain_time_builtin() {
        return temporal_zoned_date_time_to_plain_time_builtin(context, invocation).map(Some);
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

fn temporal_constructor_prototype<Cx: PublicBuiltinDispatchContext>(
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

fn current_temporal_constructor_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor_name: &str,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    let global_object = cx
        .agent()
        .realm(realm)
        .map(|record| record.global_object())
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

fn current_temporal_instant_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    current_temporal_constructor_prototype(cx, "Instant")
}

fn current_temporal_duration_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    current_temporal_constructor_prototype(cx, "Duration")
}

fn current_temporal_plain_date_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    current_temporal_constructor_prototype(cx, "PlainDate")
}

fn current_temporal_plain_time_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    current_temporal_constructor_prototype(cx, "PlainTime")
}

fn current_temporal_plain_date_time_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    current_temporal_constructor_prototype(cx, "PlainDateTime")
}

fn current_temporal_plain_year_month_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    current_temporal_constructor_prototype(cx, "PlainYearMonth")
}

fn current_temporal_plain_month_day_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    current_temporal_constructor_prototype(cx, "PlainMonthDay")
}

fn current_temporal_zoned_date_time_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    current_temporal_constructor_prototype(cx, "ZonedDateTime")
}

fn allocate_temporal_instant_object<Cx: PublicBuiltinDispatchContext>(
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
        .and_then(|record| record.root_shape())
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

fn temporal_safe_integer_number<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: i128,
) -> Result<Value, Cx::Error> {
    if !(-TEMPORAL_SAFE_INTEGER_MAX..=TEMPORAL_SAFE_INTEGER_MAX).contains(&value) {
        return Err(range_error(cx));
    }
    Ok(Value::from_f64(value as f64))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TemporalInstantStringPrecision {
    Auto,
    Minute,
    FractionalSecond(u8),
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

fn temporal_now_instant_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let instant = cx.temporal_current_instant(&TemporalCurrentInstantRequest {})?;
    let prototype = current_temporal_instant_prototype(cx)?;
    allocate_temporal_instant_object(cx, prototype, instant.epoch_nanoseconds)
}

fn temporal_now_time_zone_id_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let zone = cx.temporal_default_time_zone(&TemporalDefaultTimeZoneRequest {})?;
    Ok(string_value(cx, &zone.time_zone_id))
}

fn temporal_now_plain_date_iso_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let time_zone_id = temporal_time_zone_id_from_optional_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let (_, civil) = temporal_now_instant_and_civil(cx, &time_zone_id)?;
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

fn temporal_now_plain_time_iso_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let time_zone_id = temporal_time_zone_id_from_optional_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let (_, civil) = temporal_now_instant_and_civil(cx, &time_zone_id)?;
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
    allocate_temporal_plain_time_object(cx, prototype, data)
}

fn temporal_now_plain_date_time_iso_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let time_zone_id = temporal_time_zone_id_from_optional_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let (_, civil) = temporal_now_instant_and_civil(cx, &time_zone_id)?;
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

fn temporal_now_zoned_date_time_iso_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let time_zone_id = temporal_time_zone_id_from_optional_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let (epoch_nanoseconds, _) = temporal_now_instant_and_civil(cx, &time_zone_id)?;
    let data = temporal_zoned_date_time_from_parts(cx, epoch_nanoseconds, &time_zone_id)?;
    let prototype = current_temporal_zoned_date_time_prototype(cx)?;
    allocate_temporal_zoned_date_time_object(cx, prototype, data)
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

fn temporal_epoch_milliseconds_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<i128, Cx::Error> {
    let number = to_number_for_builtin(cx, value)?;
    if !number.is_finite() || number.trunc() != number {
        return Err(range_error(cx));
    }
    let max = TEMPORAL_INSTANT_EPOCH_MILLISECONDS_MAX as f64;
    if !(-max..=max).contains(&number) {
        return Err(range_error(cx));
    }
    Ok(number as i128)
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

fn temporal_compare_ordering(ordering: std::cmp::Ordering) -> Value {
    Value::from_smi(match ordering {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    })
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

fn temporal_instant_to_string_options<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_instant_smallest_unit_precision<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalInstantStringPrecision, Cx::Error> {
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    temporal_instant_smallest_unit_precision_from_text(cx, &text)
}

fn temporal_instant_smallest_unit_precision_from_text<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_instant_fractional_second_digits_option<Cx: PublicBuiltinDispatchContext>(
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
            digits as u8,
        ));
    }
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    if text == "auto" {
        return Ok(TemporalInstantStringPrecision::Auto);
    }
    Err(range_error(cx))
}

fn temporal_string_option<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_exact_time_unit_from_text(text: &str) -> Option<TemporalBuiltinDurationExactUnit> {
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

fn temporal_exact_time_unit_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalBuiltinDurationExactUnit, Cx::Error> {
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    temporal_exact_time_unit_from_text(&text).ok_or_else(|| range_error(cx))
}

fn temporal_exact_time_unit_order(unit: TemporalBuiltinDurationExactUnit) -> u8 {
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

fn temporal_exact_time_largest_unit_includes(
    largest_unit: TemporalBuiltinDurationExactUnit,
    component_unit: TemporalBuiltinDurationExactUnit,
) -> bool {
    temporal_exact_time_unit_order(largest_unit) <= temporal_exact_time_unit_order(component_unit)
}

fn temporal_exact_time_unit_nanoseconds(unit: TemporalBuiltinDurationExactUnit) -> i128 {
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

fn temporal_instant_largest_unit_default(
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

fn temporal_exact_time_rounding_increment_is_valid(
    smallest_unit: TemporalBuiltinDurationExactUnit,
    rounding_increment: i128,
) -> bool {
    let maximum = temporal_exact_time_rounding_increment_maximum(smallest_unit);
    rounding_increment > 0 && rounding_increment < maximum && maximum % rounding_increment == 0
}

fn temporal_exact_time_rounding_increment_maximum(
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

fn temporal_instant_rounding_increment_is_valid(
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
struct TemporalExactTimeRoundOptions {
    smallest_unit: TemporalBuiltinDurationExactUnit,
    rounding_increment: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
}

fn temporal_exact_time_round_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalExactTimeRoundOptions, Cx::Error> {
    temporal_exact_time_round_options_with_validator(
        cx,
        value,
        temporal_exact_time_rounding_increment_is_valid,
    )
}

fn temporal_instant_round_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalExactTimeRoundOptions, Cx::Error> {
    temporal_exact_time_round_options_with_validator(
        cx,
        value,
        temporal_instant_rounding_increment_is_valid,
    )
}

fn temporal_exact_time_round_options_with_validator<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_duration_from_nanoseconds_with_largest_unit<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    nanoseconds: i128,
    largest_unit: TemporalBuiltinDurationExactUnit,
) -> Result<TemporalDurationObjectData, Cx::Error> {
    let mut remainder = nanoseconds;
    let mut part = |unit: i128| -> Result<i64, Cx::Error> {
        let value = remainder / unit;
        remainder %= unit;
        i64::try_from(value).map_err(|_| range_error(cx))
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
    let rounded = temporal_round_epoch_nanoseconds_to_increment(
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

fn temporal_i64_to_number_value(value: i64) -> Value {
    i32::try_from(value)
        .map(Value::from_smi)
        .unwrap_or_else(|_| Value::from_f64(value as f64))
}

fn temporal_duration_part_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<i64, Cx::Error> {
    if value.is_undefined() {
        return Ok(0);
    }
    let number = to_number_for_builtin(cx, value)?;
    if !number.is_finite() || number.trunc() != number {
        return Err(range_error(cx));
    }
    if !(-(TEMPORAL_SAFE_INTEGER_MAX as f64)..=TEMPORAL_SAFE_INTEGER_MAX as f64).contains(&number) {
        return Err(range_error(cx));
    }
    Ok(number as i64)
}

fn temporal_duration_part_i128_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<i128, Cx::Error> {
    if value.is_undefined() {
        return Ok(0);
    }
    let number = to_number_for_builtin(cx, value)?;
    if !number.is_finite() || number.trunc() != number {
        return Err(range_error(cx));
    }
    if number < i128::MIN as f64 || number > i128::MAX as f64 {
        return Err(range_error(cx));
    }
    Ok(number as i128)
}

fn temporal_duration_part_i128_to_i64<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: i128,
) -> Result<i64, Cx::Error> {
    i64::try_from(value).map_err(|_| range_error(cx))
}

fn temporal_duration_part_from_argument<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    index: usize,
) -> Result<i64, Cx::Error> {
    temporal_duration_part_from_value(
        cx,
        invocation
            .arguments()
            .get(index)
            .copied()
            .unwrap_or(Value::undefined()),
    )
}

fn temporal_optional_duration_part_from_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    property_name: &str,
) -> Result<Option<i64>, Cx::Error> {
    let value = temporal_property_value(cx, object_ref, property_name)?;
    if value.is_undefined() {
        return Ok(None);
    }
    temporal_duration_part_from_value(cx, value).map(Some)
}

fn temporal_duration_part_i128_from_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    property_name: &str,
) -> Result<(bool, i128), Cx::Error> {
    let value = temporal_property_value(cx, object_ref, property_name)?;
    if value.is_undefined() {
        return Ok((false, 0));
    }
    Ok((true, temporal_duration_part_i128_from_value(cx, value)?))
}

fn validate_temporal_duration<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    data: TemporalDurationObjectData,
) -> Result<(), Cx::Error> {
    if !temporal_ops::duration_signs_are_balanced(data)
        || !temporal_ops::duration_is_within_limits(data)
    {
        return Err(range_error(cx));
    }
    Ok(())
}

fn allocate_temporal_duration_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: ObjectRef,
    data: TemporalDurationObjectData,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let root_shape = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.root_shape())
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
                        TemporalObjectKind::Duration,
                    ))),
                AllocationLifetime::Default,
            )
        })
    };
    let installed = cx
        .agent()
        .objects_mut()
        .install_temporal_object(object, TemporalObjectData::Duration(data));
    if !installed {
        return Err(type_error(cx));
    }
    Ok(Value::from_object_ref(object))
}

fn allocate_current_temporal_blank_duration_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<Value, Cx::Error> {
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(
        cx,
        prototype,
        TemporalDurationObjectData::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
    )
}

fn temporal_duration_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationObjectData, Cx::Error> {
    let Some(object_ref) = value.as_object_ref() else {
        if !value.is_string() {
            return Err(type_error(cx));
        }
        let string_ref = to_string_string_ref(cx, value)?;
        let text = string_ref_text(cx, string_ref)?;
        let data = parse_temporal_duration(&text).ok_or_else(|| range_error(cx))?;
        validate_temporal_duration(cx, data)?;
        return Ok(data);
    };
    let existing = {
        let agent = cx.agent();
        agent.objects().temporal_object(object_ref).copied()
    };
    if let Some(TemporalObjectData::Duration(data)) = existing {
        return Ok(data);
    }

    let (has_days, days) = temporal_duration_part_i128_from_property(cx, object_ref, "days")?;
    let (has_hours, hours) = temporal_duration_part_i128_from_property(cx, object_ref, "hours")?;
    let (has_microseconds, microseconds) =
        temporal_duration_part_i128_from_property(cx, object_ref, "microseconds")?;
    let (has_milliseconds, milliseconds) =
        temporal_duration_part_i128_from_property(cx, object_ref, "milliseconds")?;
    let (has_minutes, minutes) =
        temporal_duration_part_i128_from_property(cx, object_ref, "minutes")?;
    let (has_months, months) = temporal_duration_part_i128_from_property(cx, object_ref, "months")?;
    let (has_nanoseconds, nanoseconds) =
        temporal_duration_part_i128_from_property(cx, object_ref, "nanoseconds")?;
    let (has_seconds, seconds) =
        temporal_duration_part_i128_from_property(cx, object_ref, "seconds")?;
    let (has_weeks, weeks) = temporal_duration_part_i128_from_property(cx, object_ref, "weeks")?;
    let (has_years, years) = temporal_duration_part_i128_from_property(cx, object_ref, "years")?;
    if ![
        has_days,
        has_hours,
        has_microseconds,
        has_milliseconds,
        has_minutes,
        has_months,
        has_nanoseconds,
        has_seconds,
        has_weeks,
        has_years,
    ]
    .iter()
    .any(|present| *present)
    {
        return Err(type_error(cx));
    }
    let [seconds, milliseconds, microseconds, nanoseconds] = match [
        i64::try_from(seconds),
        i64::try_from(milliseconds),
        i64::try_from(microseconds),
        i64::try_from(nanoseconds),
    ] {
        [Ok(seconds), Ok(milliseconds), Ok(microseconds), Ok(nanoseconds)] => {
            [seconds, milliseconds, microseconds, nanoseconds]
        }
        _ => temporal_ops::balance_duration_subsecond_fields(
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
        )
        .ok_or_else(|| range_error(cx))?,
    };

    let data = TemporalDurationObjectData::new(
        temporal_duration_part_i128_to_i64(cx, years)?,
        temporal_duration_part_i128_to_i64(cx, months)?,
        temporal_duration_part_i128_to_i64(cx, weeks)?,
        temporal_duration_part_i128_to_i64(cx, days)?,
        temporal_duration_part_i128_to_i64(cx, hours)?,
        temporal_duration_part_i128_to_i64(cx, minutes)?,
        seconds,
        milliseconds,
        microseconds,
        nanoseconds,
    );
    validate_temporal_duration(cx, data)?;
    Ok(data)
}

fn temporal_duration_from_additive_argument<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationObjectData, Cx::Error> {
    if value.as_object_ref().is_none() && !value.is_string() {
        return Err(type_error(cx));
    }
    temporal_duration_from_value(cx, value)
}

fn temporal_duration_data<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    this_value: Value,
) -> Result<TemporalDurationObjectData, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, this_value, TemporalObjectKind::Duration)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::Duration(data) = payload else {
        return Err(type_error(cx));
    };
    Ok(data)
}

fn temporal_duration_component_getter<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    component: fn(TemporalDurationObjectData) -> i64,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    Ok(temporal_i64_to_number_value(component(data)))
}

fn temporal_duration_to_string_options<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_duration_smallest_unit_digits<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_duration_fractional_second_digits_option<Cx: PublicBuiltinDispatchContext>(
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
        return Ok(Some(digits as u8));
    }
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    if text == "auto" {
        return Ok(None);
    }
    Err(range_error(cx))
}

fn temporal_duration_rounding_mode_option<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalBuiltinRoundingMode, Cx::Error> {
    temporal_duration_rounding_mode_option_with_default(
        cx,
        value,
        TemporalBuiltinRoundingMode::Trunc,
    )
}

fn temporal_duration_rounding_mode_option_with_default<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    default: TemporalBuiltinRoundingMode,
) -> Result<TemporalBuiltinRoundingMode, Cx::Error> {
    if value.is_undefined() {
        return Ok(default);
    }
    temporal_duration_rounding_mode_from_value(cx, value)
}

fn temporal_duration_rounding_mode_from_value<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_option_string_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Option<String>, Cx::Error> {
    if value.is_undefined() {
        return Ok(None);
    }
    let string_ref = to_string_string_ref(cx, value)?;
    Ok(Some(string_ref_text(cx, string_ref)?))
}

#[derive(Clone, Copy)]
enum TemporalDurationParsedUnit {
    CalendarRelative,
    Exact(TemporalBuiltinDurationExactUnit),
}

#[derive(Clone, Copy)]
enum TemporalDurationParsedLargestUnit {
    Missing,
    Auto,
    CalendarRelative,
    Exact(TemporalBuiltinDurationExactUnit),
}

struct TemporalDurationRoundOptions {
    largest_unit: TemporalDurationParsedLargestUnit,
    smallest_unit: Option<TemporalDurationParsedUnit>,
    rounding_increment: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
    relative_to: Option<TemporalDurationRelativeTo>,
}

struct TemporalDurationTotalOptions {
    unit: TemporalDurationParsedUnit,
    relative_to: Option<TemporalDurationRelativeTo>,
}

#[derive(Clone, Copy)]
enum TemporalDurationRelativeTo {
    PlainDate(TemporalPlainDateObjectData),
    PlainDateTime(TemporalPlainDateTimeObjectData),
    ZonedDateTime(TemporalZonedDateTimeObjectData),
}

fn temporal_duration_round_options<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_duration_total_options<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_duration_relative_to_option<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Option<TemporalDurationRelativeTo>, Cx::Error> {
    if value.is_undefined() || value.is_null() {
        return Ok(None);
    }
    if let Some(string_ref) = value.as_string_ref() {
        let text = string_ref_text(cx, string_ref)?;
        if temporal_zoned_date_time_zone_annotation(&text).is_some() {
            return temporal_zoned_date_time_from_value(cx, value)
                .map(TemporalDurationRelativeTo::ZonedDateTime)
                .map(Some);
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

    let time_zone = temporal_property_value(cx, object_ref, "timeZone")?;
    if !time_zone.is_undefined() {
        return temporal_zoned_date_time_from_value(cx, value)
            .map(TemporalDurationRelativeTo::ZonedDateTime)
            .map(Some);
    }

    let date = temporal_plain_date_from_value(cx, value)?;
    let has_time = [
        "hour",
        "minute",
        "second",
        "millisecond",
        "microsecond",
        "nanosecond",
    ]
    .iter()
    .try_fold(false, |has_time, property| {
        let property_value = temporal_property_value(cx, object_ref, property)?;
        Ok::<_, Cx::Error>(has_time || !property_value.is_undefined())
    })?;
    if !has_time {
        return Ok(Some(TemporalDurationRelativeTo::PlainDate(date)));
    }
    let hour = temporal_optional_time_part_from_property(cx, object_ref, "hour")?.unwrap_or(0);
    let minute = temporal_optional_time_part_from_property(cx, object_ref, "minute")?.unwrap_or(0);
    let second = temporal_optional_time_part_from_property(cx, object_ref, "second")?.unwrap_or(0);
    let millisecond =
        temporal_optional_time_part_from_property(cx, object_ref, "millisecond")?.unwrap_or(0);
    let microsecond =
        temporal_optional_time_part_from_property(cx, object_ref, "microsecond")?.unwrap_or(0);
    let nanosecond =
        temporal_optional_time_part_from_property(cx, object_ref, "nanosecond")?.unwrap_or(0);
    let date_time = temporal_plain_date_time_from_parts(
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
    )?;
    Ok(Some(TemporalDurationRelativeTo::PlainDateTime(date_time)))
}

fn temporal_duration_exact_unit_nanoseconds(unit: TemporalBuiltinDurationExactUnit) -> i128 {
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

fn temporal_date_time_difference_unit_from_duration_exact(
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

fn temporal_duration_relative_total_nanoseconds<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    duration: TemporalDurationObjectData,
    relative_to: TemporalDurationRelativeTo,
) -> Result<i128, Cx::Error> {
    match relative_to {
        TemporalDurationRelativeTo::PlainDate(start) => {
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
            let end = temporal_plain_date_add_duration(
                cx,
                start,
                date_duration,
                TemporalOverflow::Constrain,
            )?;
            let days = temporal_plain_date_ordinal_day(end)
                .checked_sub(temporal_plain_date_ordinal_day(start))
                .ok_or_else(|| range_error(cx))?;
            days.checked_mul(TEMPORAL_NANOS_PER_DAY)
                .and_then(|date_nanoseconds| {
                    date_nanoseconds.checked_add(temporal_duration_time_nanoseconds(duration))
                })
                .ok_or_else(|| range_error(cx))
        }
        TemporalDurationRelativeTo::PlainDateTime(start) => {
            let end = temporal_plain_date_time_add_duration(
                cx,
                start,
                duration,
                TemporalOverflow::Constrain,
            )?;
            temporal_plain_date_time_total_nanoseconds(end)
                .and_then(|end_nanoseconds| {
                    let start_nanoseconds = temporal_plain_date_time_total_nanoseconds(start)?;
                    end_nanoseconds.checked_sub(start_nanoseconds)
                })
                .ok_or_else(|| range_error(cx))
        }
        TemporalDurationRelativeTo::ZonedDateTime(start) => {
            let end_value = temporal_zoned_date_time_add_duration(cx, start, duration)?;
            let end = temporal_zoned_date_time_data(cx, end_value)?;
            end.epoch_nanoseconds()
                .checked_sub(start.epoch_nanoseconds())
                .ok_or_else(|| range_error(cx))
        }
    }
}

fn temporal_duration_largest_unit_option<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationParsedLargestUnit, Cx::Error> {
    if value.is_undefined() {
        return Ok(TemporalDurationParsedLargestUnit::Missing);
    }
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    if text == "auto" {
        return Ok(TemporalDurationParsedLargestUnit::Auto);
    }
    Ok(match temporal_duration_unit_from_text(&text) {
        Some(TemporalDurationParsedUnit::Exact(unit)) => {
            TemporalDurationParsedLargestUnit::Exact(unit)
        }
        Some(TemporalDurationParsedUnit::CalendarRelative) => {
            TemporalDurationParsedLargestUnit::CalendarRelative
        }
        None => return Err(range_error(cx)),
    })
}

fn temporal_duration_parsed_unit<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDurationParsedUnit, Cx::Error> {
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    temporal_duration_unit_from_text(&text).ok_or_else(|| range_error(cx))
}

fn temporal_duration_unit_from_text(text: &str) -> Option<TemporalDurationParsedUnit> {
    match text {
        "year" | "years" | "month" | "months" | "week" | "weeks" => {
            Some(TemporalDurationParsedUnit::CalendarRelative)
        }
        "day" | "days" => Some(TemporalDurationParsedUnit::Exact(
            TemporalBuiltinDurationExactUnit::Day,
        )),
        "hour" | "hours" => Some(TemporalDurationParsedUnit::Exact(
            TemporalBuiltinDurationExactUnit::Hour,
        )),
        "minute" | "minutes" => Some(TemporalDurationParsedUnit::Exact(
            TemporalBuiltinDurationExactUnit::Minute,
        )),
        "second" | "seconds" => Some(TemporalDurationParsedUnit::Exact(
            TemporalBuiltinDurationExactUnit::Second,
        )),
        "millisecond" | "milliseconds" => Some(TemporalDurationParsedUnit::Exact(
            TemporalBuiltinDurationExactUnit::Millisecond,
        )),
        "microsecond" | "microseconds" => Some(TemporalDurationParsedUnit::Exact(
            TemporalBuiltinDurationExactUnit::Microsecond,
        )),
        "nanosecond" | "nanoseconds" => Some(TemporalDurationParsedUnit::Exact(
            TemporalBuiltinDurationExactUnit::Nanosecond,
        )),
        _ => None,
    }
}

fn temporal_duration_rounding_increment_option<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<i128, Cx::Error> {
    if value.is_undefined() {
        return Ok(1);
    }
    let number = to_number_for_builtin(cx, value)?;
    if !number.is_finite() || number.is_nan() {
        return Err(range_error(cx));
    }
    let increment = number.floor();
    if !(1.0..=1_000_000_000.0).contains(&increment) {
        return Err(range_error(cx));
    }
    Ok(increment as i128)
}

fn temporal_duration_rounding_increment_is_valid(
    smallest_unit: TemporalBuiltinDurationExactUnit,
    rounding_increment: i128,
) -> bool {
    let maximum = match smallest_unit {
        TemporalBuiltinDurationExactUnit::Day => {
            return (1..=1_000_000_000).contains(&rounding_increment)
        }
        TemporalBuiltinDurationExactUnit::Hour => 24,
        TemporalBuiltinDurationExactUnit::Minute | TemporalBuiltinDurationExactUnit::Second => 60,
        TemporalBuiltinDurationExactUnit::Millisecond
        | TemporalBuiltinDurationExactUnit::Microsecond
        | TemporalBuiltinDurationExactUnit::Nanosecond => 1000,
    };
    rounding_increment > 0 && rounding_increment < maximum && maximum % rounding_increment == 0
}

fn temporal_duration_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let Some(new_target) = invocation.new_target() else {
        return Err(type_error(cx));
    };
    let data = TemporalDurationObjectData::new(
        temporal_duration_part_from_argument(cx, invocation, 0)?,
        temporal_duration_part_from_argument(cx, invocation, 1)?,
        temporal_duration_part_from_argument(cx, invocation, 2)?,
        temporal_duration_part_from_argument(cx, invocation, 3)?,
        temporal_duration_part_from_argument(cx, invocation, 4)?,
        temporal_duration_part_from_argument(cx, invocation, 5)?,
        temporal_duration_part_from_argument(cx, invocation, 6)?,
        temporal_duration_part_from_argument(cx, invocation, 7)?,
        temporal_duration_part_from_argument(cx, invocation, 8)?,
        temporal_duration_part_from_argument(cx, invocation, 9)?,
    );
    validate_temporal_duration(cx, data)?;
    let prototype = temporal_constructor_prototype(cx, new_target)?;
    allocate_temporal_duration_object(cx, prototype, data)
}

fn temporal_duration_years_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::years)
}

fn temporal_duration_months_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::months)
}

fn temporal_duration_weeks_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::weeks)
}

fn temporal_duration_days_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::days)
}

fn temporal_duration_hours_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::hours)
}

fn temporal_duration_minutes_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::minutes)
}

fn temporal_duration_seconds_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::seconds)
}

fn temporal_duration_milliseconds_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::milliseconds)
}

fn temporal_duration_microseconds_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::microseconds)
}

fn temporal_duration_nanoseconds_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_component_getter(cx, invocation, TemporalDurationObjectData::nanoseconds)
}

fn temporal_duration_sign_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(temporal_duration_sign(data)))
}

fn temporal_duration_blank_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    Ok(Value::from_bool(temporal_duration_sign(data) == 0))
}

fn temporal_duration_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    let (fractional_digits, rounding_mode) = temporal_duration_to_string_options(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let text = if let Some(digits) = fractional_digits {
        format_temporal_duration_with_seconds_precision(data, digits, rounding_mode)
            .ok_or_else(|| range_error(cx))?
    } else {
        format_temporal_duration(data)
    };
    Ok(string_value(cx, &text))
}

fn temporal_duration_to_json_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format_temporal_duration(data)))
}

fn temporal_duration_to_locale_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format_temporal_duration(data)))
}

fn temporal_duration_negated_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = negate_temporal_duration(temporal_duration_data(cx, invocation.this_value())?);
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(cx, prototype, data)
}

fn temporal_duration_abs_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    let data = if temporal_duration_sign(data) < 0 {
        negate_temporal_duration(data)
    } else {
        data
    };
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(cx, prototype, data)
}

fn temporal_duration_with_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let base = temporal_duration_data(cx, invocation.this_value())?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(object_ref) = value.as_object_ref() else {
        return Err(type_error(cx));
    };

    let days = temporal_optional_duration_part_from_property(cx, object_ref, "days")?;
    let hours = temporal_optional_duration_part_from_property(cx, object_ref, "hours")?;
    let microseconds =
        temporal_optional_duration_part_from_property(cx, object_ref, "microseconds")?;
    let milliseconds =
        temporal_optional_duration_part_from_property(cx, object_ref, "milliseconds")?;
    let minutes = temporal_optional_duration_part_from_property(cx, object_ref, "minutes")?;
    let months = temporal_optional_duration_part_from_property(cx, object_ref, "months")?;
    let nanoseconds = temporal_optional_duration_part_from_property(cx, object_ref, "nanoseconds")?;
    let seconds = temporal_optional_duration_part_from_property(cx, object_ref, "seconds")?;
    let weeks = temporal_optional_duration_part_from_property(cx, object_ref, "weeks")?;
    let years = temporal_optional_duration_part_from_property(cx, object_ref, "years")?;

    if [
        days,
        hours,
        microseconds,
        milliseconds,
        minutes,
        months,
        nanoseconds,
        seconds,
        weeks,
        years,
    ]
    .iter()
    .all(Option::is_none)
    {
        return Err(type_error(cx));
    }

    let data = TemporalDurationObjectData::new(
        years.unwrap_or_else(|| base.years()),
        months.unwrap_or_else(|| base.months()),
        weeks.unwrap_or_else(|| base.weeks()),
        days.unwrap_or_else(|| base.days()),
        hours.unwrap_or_else(|| base.hours()),
        minutes.unwrap_or_else(|| base.minutes()),
        seconds.unwrap_or_else(|| base.seconds()),
        milliseconds.unwrap_or_else(|| base.milliseconds()),
        microseconds.unwrap_or_else(|| base.microseconds()),
        nanoseconds.unwrap_or_else(|| base.nanoseconds()),
    );
    validate_temporal_duration(cx, data)?;
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(cx, prototype, data)
}

fn temporal_duration_round_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    let options = temporal_duration_round_options(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;

    let smallest_unit = match options.smallest_unit {
        Some(TemporalDurationParsedUnit::Exact(unit)) => unit,
        Some(TemporalDurationParsedUnit::CalendarRelative) => {
            if temporal_duration_sign(data) == 0 && options.relative_to.is_some() {
                return allocate_current_temporal_blank_duration_object(cx);
            }
            return Err(range_error(cx));
        }
        None => TemporalBuiltinDurationExactUnit::Nanosecond,
    };
    let largest_unit = match options.largest_unit {
        TemporalDurationParsedLargestUnit::Missing | TemporalDurationParsedLargestUnit::Auto => {
            temporal_duration_default_largest_exact_unit(data, smallest_unit)
        }
        TemporalDurationParsedLargestUnit::Exact(unit) => unit,
        TemporalDurationParsedLargestUnit::CalendarRelative => {
            if temporal_duration_sign(data) == 0 && options.relative_to.is_some() {
                return allocate_current_temporal_blank_duration_object(cx);
            }
            return Err(range_error(cx));
        }
    };

    if !temporal_duration_exact_unit_allows_largest_smallest(largest_unit, smallest_unit)
        || !temporal_duration_rounding_increment_is_valid(smallest_unit, options.rounding_increment)
    {
        return Err(range_error(cx));
    }
    if temporal_ops::duration_has_calendar_relative_units(data) {
        let Some(relative_to) = options.relative_to else {
            return Err(range_error(cx));
        };
        let total_nanoseconds =
            temporal_duration_relative_total_nanoseconds(cx, data, relative_to)?;
        let increment = temporal_duration_exact_unit_nanoseconds(smallest_unit)
            .checked_mul(options.rounding_increment)
            .ok_or_else(|| range_error(cx))?;
        let rounded = temporal_round_epoch_nanoseconds_to_increment(
            total_nanoseconds,
            increment,
            options.rounding_mode,
        )
        .ok_or_else(|| range_error(cx))?;
        let data = temporal_duration_from_date_time_nanoseconds(
            cx,
            rounded,
            temporal_date_time_difference_unit_from_duration_exact(largest_unit),
        )?;
        validate_temporal_duration(cx, data)?;
        let prototype = current_temporal_duration_prototype(cx)?;
        return allocate_temporal_duration_object(cx, prototype, data);
    }
    let data = temporal_round_duration_exact(
        data,
        largest_unit,
        smallest_unit,
        options.rounding_increment,
        options.rounding_mode,
    )
    .ok_or_else(|| range_error(cx))?;
    validate_temporal_duration(cx, data)?;
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(cx, prototype, data)
}

fn temporal_duration_total_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_data(cx, invocation.this_value())?;
    let options = temporal_duration_total_options(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let total = match options.unit {
        TemporalDurationParsedUnit::Exact(unit) => {
            if temporal_ops::duration_has_calendar_relative_units(data) {
                let Some(relative_to) = options.relative_to else {
                    return Err(range_error(cx));
                };
                let total_nanoseconds =
                    temporal_duration_relative_total_nanoseconds(cx, data, relative_to)?;
                total_nanoseconds as f64 / temporal_duration_exact_unit_nanoseconds(unit) as f64
            } else {
                temporal_total_duration_exact(data, unit)
                    .filter(|value| value.is_finite())
                    .ok_or_else(|| range_error(cx))?
            }
        }
        TemporalDurationParsedUnit::CalendarRelative => {
            if temporal_duration_sign(data) == 0 && options.relative_to.is_some() {
                0.0
            } else {
                return Err(range_error(cx));
            }
        }
    };
    Ok(Value::from_f64(total))
}

fn temporal_duration_add_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_additive_builtin(cx, invocation, temporal_ops::add_durations)
}

fn temporal_duration_subtract_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_duration_additive_builtin(cx, invocation, temporal_ops::subtract_durations)
}

fn temporal_duration_additive_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    operation: fn(
        TemporalDurationObjectData,
        TemporalDurationObjectData,
    ) -> Option<TemporalDurationObjectData>,
) -> Result<Value, Cx::Error> {
    let base = temporal_duration_data(cx, invocation.this_value())?;
    let other = temporal_duration_from_additive_argument(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let data = operation(base, other).ok_or_else(|| range_error(cx))?;
    validate_temporal_duration(cx, data)?;
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(cx, prototype, data)
}

fn temporal_duration_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Err(type_error(cx))
}

fn temporal_duration_from_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_duration_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(cx, prototype, data)
}

fn temporal_duration_compare_relative_to_option<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Option<TemporalDurationRelativeTo>, Cx::Error> {
    if value.is_undefined() {
        return Ok(None);
    }
    let Some(object_ref) = value.as_object_ref() else {
        return Err(type_error(cx));
    };
    let relative_to = temporal_property_value(cx, object_ref, "relativeTo")?;
    temporal_duration_relative_to_option(cx, relative_to)
}

fn temporal_duration_compare_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = temporal_duration_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let right = temporal_duration_from_value(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let relative_to = temporal_duration_compare_relative_to_option(
        cx,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if temporal_ops::durations_are_equal(left, right) {
        return Ok(Value::from_smi(0));
    }
    if temporal_ops::duration_has_calendar_relative_units(left)
        || temporal_ops::duration_has_calendar_relative_units(right)
    {
        let Some(relative_to) = relative_to else {
            return Err(range_error(cx));
        };
        let left_total = temporal_duration_relative_total_nanoseconds(cx, left, relative_to)?;
        let right_total = temporal_duration_relative_total_nanoseconds(cx, right, relative_to)?;
        return Ok(temporal_compare_ordering(left_total.cmp(&right_total)));
    }
    let ordering =
        temporal_ops::compare_time_duration(left, right).ok_or_else(|| range_error(cx))?;
    Ok(temporal_compare_ordering(ordering))
}

fn temporal_integer_part_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<i64, Cx::Error> {
    let number = to_number_for_builtin(cx, value)?;
    if !number.is_finite() {
        return Err(range_error(cx));
    }
    let number = number.trunc();
    if !(-(TEMPORAL_SAFE_INTEGER_MAX as f64)..=TEMPORAL_SAFE_INTEGER_MAX as f64).contains(&number) {
        return Err(range_error(cx));
    }
    Ok(number as i64)
}

fn temporal_integer_part_from_argument<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_property_value<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_validate_options_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<(), Cx::Error> {
    if value.is_undefined() || value.as_object_ref().is_some() {
        return Ok(());
    }
    Err(type_error(cx))
}

fn temporal_required_integer_part_from_property<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_optional_integer_part_from_property<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_from_ordinal_day<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_from_parts<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    year: i64,
    month: i64,
    day: i64,
) -> Result<TemporalPlainDateObjectData, Cx::Error> {
    temporal_plain_date_from_parts_with_overflow(cx, year, month, day, TemporalOverflow::Reject)
}

fn temporal_plain_date_from_parts_with_overflow<Cx: PublicBuiltinDispatchContext>(
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

fn allocate_temporal_plain_date_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: ObjectRef,
    data: TemporalPlainDateObjectData,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let root_shape = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.root_shape())
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

fn temporal_plain_date_from_value<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_from_property_bag<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    validate_calendar: bool,
) -> Result<TemporalPlainDateObjectData, Cx::Error> {
    if validate_calendar {
        temporal_validate_optional_iso_calendar_property(cx, object_ref)?;
    }
    let year = temporal_required_integer_part_from_property(cx, object_ref, "year")?;
    let month = temporal_month_from_property_bag(cx, object_ref, None)?;
    let day = temporal_required_integer_part_from_property(cx, object_ref, "day")?;
    temporal_plain_date_from_parts(cx, year, month, day)
}

struct TemporalPlainDateBagFields {
    year: Option<i64>,
    month: Option<i64>,
    month_code_text: Option<String>,
    day: Option<i64>,
}

struct TemporalPlainMonthDayBagFields {
    year: Option<i64>,
    month: Option<i64>,
    month_code_text: Option<String>,
    day: Option<i64>,
}

struct TemporalPlainDateTimeBagFields {
    year: Option<i64>,
    month: Option<i64>,
    month_code_text: Option<String>,
    day: Option<i64>,
    hour: i64,
    minute: i64,
    second: i64,
    millisecond: i64,
    microsecond: i64,
    nanosecond: i64,
}

struct TemporalPlainYearMonthBagFields {
    year: Option<i64>,
    month: Option<i64>,
    month_code_text: Option<String>,
}

fn temporal_string_text_from_value<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_optional_string_text_from_property<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_optional_month_code_text_from_property<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_resolve_month_from_fields<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_bag_fields<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_time_bag_fields<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_from_bag_fields<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    fields: TemporalPlainDateBagFields,
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

fn temporal_plain_date_data<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(data.year()))
}

fn temporal_plain_date_month_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.month())))
}

fn temporal_plain_date_month_code_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format!("M{:02}", data.month())))
}

fn temporal_plain_date_day_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.day())))
}

fn temporal_plain_date_day_of_week_getter_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_day_of_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_days_in_month_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(temporal_iso_days_in_month(
        data.year(),
        data.month(),
    ))))
}

fn temporal_plain_date_days_in_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(temporal_iso_days_in_year(data.year())))
}

fn temporal_plain_date_months_in_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(12))
}

fn temporal_plain_date_in_leap_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::from_bool(temporal_is_iso_leap_year(data.year())))
}

fn temporal_plain_date_days_in_week_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(7))
}

fn temporal_plain_date_week_of_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    let (week, _) = temporal_iso_week_of_year(data.year(), data.month(), data.day());
    Ok(Value::from_smi(week))
}

fn temporal_plain_date_year_of_week_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    let (_, year) = temporal_iso_week_of_year(data.year(), data.month(), data.day());
    Ok(Value::from_smi(year))
}

fn temporal_plain_date_era_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::undefined())
}

fn temporal_plain_date_era_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(Value::undefined())
}

fn temporal_plain_date_calendar_id_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(string_value(cx, "iso8601"))
}

fn temporal_plain_date_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_to_json_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format_temporal_plain_date(data)))
}

fn temporal_plain_date_to_locale_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format_temporal_plain_date(data)))
}

fn temporal_plain_date_to_string_calendar_name<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Err(type_error(cx))
}

fn temporal_plain_date_equals_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_with_builtin<Cx: PublicBuiltinDispatchContext>(
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
    let data = temporal_plain_date_from_parts_with_overflow(
        cx,
        year.unwrap_or(i64::from(date.year())),
        month,
        day.unwrap_or(i64::from(date.day())),
        overflow,
    )?;
    if !options.is_undefined() && options.as_object_ref().is_none() {
        return Err(type_error(cx));
    }
    let prototype = current_temporal_plain_date_prototype(cx)?;
    allocate_temporal_plain_date_object(cx, prototype, data)
}

fn temporal_plain_date_with_calendar_builtin<Cx: PublicBuiltinDispatchContext>(
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
    let data =
        TemporalPlainDateObjectData::new(date.year(), date.month(), date.day(), date.calendar());
    let prototype = current_temporal_plain_date_prototype(cx)?;
    allocate_temporal_plain_date_object(cx, prototype, data)
}

fn temporal_plain_date_add_duration<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    date: TemporalPlainDateObjectData,
    duration: TemporalDurationObjectData,
    overflow: TemporalOverflow,
) -> Result<TemporalPlainDateObjectData, Cx::Error> {
    let total_months = i128::from(date.year()) * 12
        + i128::from(date.month() - 1)
        + i128::from(duration.years()) * 12
        + i128::from(duration.months());
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

    let day_delta = i128::from(duration.weeks()) * 7
        + i128::from(duration.days())
        + temporal_duration_whole_days_from_time(duration);
    let ordinal_day = temporal_plain_date_ordinal_day(constrained)
        .checked_add(day_delta)
        .ok_or_else(|| range_error(cx))?;
    temporal_plain_date_from_ordinal_day(cx, ordinal_day)
}

fn temporal_plain_date_add_builtin<Cx: PublicBuiltinDispatchContext>(
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
    let data = temporal_plain_date_add_duration(cx, date, duration, overflow)?;
    let prototype = current_temporal_plain_date_prototype(cx)?;
    allocate_temporal_plain_date_object(cx, prototype, data)
}

fn temporal_plain_date_subtract_builtin<Cx: PublicBuiltinDispatchContext>(
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
    let data =
        temporal_plain_date_add_duration(cx, date, negate_temporal_duration(duration), overflow)?;
    let prototype = current_temporal_plain_date_prototype(cx)?;
    allocate_temporal_plain_date_object(cx, prototype, data)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TemporalDateDifferenceUnit {
    Year,
    Month,
    Week,
    Day,
}

fn temporal_date_difference_unit_from_text(text: &str) -> Option<TemporalDateDifferenceUnit> {
    match text {
        "year" | "years" => Some(TemporalDateDifferenceUnit::Year),
        "month" | "months" => Some(TemporalDateDifferenceUnit::Month),
        "week" | "weeks" => Some(TemporalDateDifferenceUnit::Week),
        "day" | "days" => Some(TemporalDateDifferenceUnit::Day),
        _ => None,
    }
}

fn temporal_date_difference_unit_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDateDifferenceUnit, Cx::Error> {
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    temporal_date_difference_unit_from_text(&text).ok_or_else(|| range_error(cx))
}

fn temporal_date_difference_unit_order(unit: TemporalDateDifferenceUnit) -> u8 {
    match unit {
        TemporalDateDifferenceUnit::Year => 0,
        TemporalDateDifferenceUnit::Month => 1,
        TemporalDateDifferenceUnit::Week => 2,
        TemporalDateDifferenceUnit::Day => 3,
    }
}

struct TemporalDateDifferenceOptions {
    largest_unit: TemporalDateDifferenceUnit,
    smallest_unit: TemporalDateDifferenceUnit,
    rounding_increment: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
}

fn temporal_date_difference_options<Cx: PublicBuiltinDispatchContext>(
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
    let largest_unit = if largest_unit_value.is_undefined() {
        default_largest_unit
    } else {
        let string_ref = to_string_string_ref(cx, largest_unit_value)?;
        let text = string_ref_text(cx, string_ref)?;
        if text == "auto" {
            default_largest_unit
        } else {
            temporal_date_difference_unit_from_text(&text).ok_or_else(|| range_error(cx))?
        }
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

fn temporal_round_i128_to_increment<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: i128,
    increment: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
) -> Result<i128, Cx::Error> {
    temporal_round_epoch_nanoseconds_to_increment(value, increment, rounding_mode)
        .ok_or_else(|| range_error(cx))
}

fn temporal_plain_date_ordering(
    left: TemporalPlainDateObjectData,
    right: TemporalPlainDateObjectData,
) -> std::cmp::Ordering {
    (left.year(), left.month(), left.day()).cmp(&(right.year(), right.month(), right.day()))
}

fn temporal_plain_date_balanced_positive_months_until<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    start: TemporalPlainDateObjectData,
    end: TemporalPlainDateObjectData,
) -> Result<(i128, TemporalPlainDateObjectData), Cx::Error> {
    let mut months = (i128::from(end.year()) - i128::from(start.year()))
        .checked_mul(12)
        .and_then(|difference| difference.checked_add(i128::from(end.month() - start.month())))
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

fn temporal_plain_date_difference_trunc<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    start: TemporalPlainDateObjectData,
    end: TemporalPlainDateObjectData,
    largest_unit: TemporalDateDifferenceUnit,
    smallest_unit: TemporalDateDifferenceUnit,
) -> Result<TemporalDurationObjectData, Cx::Error> {
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

fn temporal_duration_from_date_units<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_difference_builtin<Cx: PublicBuiltinDispatchContext>(
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

    if options.rounding_increment == 1
        && options.rounding_mode == TemporalBuiltinRoundingMode::Trunc
        && !matches!(
            (options.largest_unit, options.smallest_unit),
            (
                TemporalDateDifferenceUnit::Day,
                TemporalDateDifferenceUnit::Week | TemporalDateDifferenceUnit::Day
            ) | (
                TemporalDateDifferenceUnit::Week,
                TemporalDateDifferenceUnit::Week | TemporalDateDifferenceUnit::Day
            )
        )
    {
        let (start, end) = if sign > 0 {
            (other, date)
        } else {
            (date, other)
        };
        let duration = temporal_plain_date_difference_trunc(
            cx,
            start,
            end,
            options.largest_unit,
            options.smallest_unit,
        )?;
        validate_temporal_duration(cx, duration)?;
        let prototype = current_temporal_duration_prototype(cx)?;
        return allocate_temporal_duration_object(cx, prototype, duration);
    }

    let (raw_units, unit_kind) = match options.smallest_unit {
        TemporalDateDifferenceUnit::Year | TemporalDateDifferenceUnit::Month => {
            let left = i128::from(date.year()) * 12 + i128::from(date.month() - 1);
            let right = i128::from(other.year()) * 12 + i128::from(other.month() - 1);
            (
                left.checked_sub(right)
                    .and_then(|difference| difference.checked_mul(sign))
                    .ok_or_else(|| range_error(cx))?,
                TemporalDateDifferenceUnit::Month,
            )
        }
        TemporalDateDifferenceUnit::Week | TemporalDateDifferenceUnit::Day => (
            temporal_plain_date_ordinal_day(date)
                .checked_sub(temporal_plain_date_ordinal_day(other))
                .and_then(|difference| difference.checked_mul(sign))
                .ok_or_else(|| range_error(cx))?,
            TemporalDateDifferenceUnit::Day,
        ),
    };
    let increment = match options.smallest_unit {
        TemporalDateDifferenceUnit::Year => options
            .rounding_increment
            .checked_mul(12)
            .ok_or_else(|| range_error(cx))?,
        TemporalDateDifferenceUnit::Month | TemporalDateDifferenceUnit::Day => {
            options.rounding_increment
        }
        TemporalDateDifferenceUnit::Week => options
            .rounding_increment
            .checked_mul(7)
            .ok_or_else(|| range_error(cx))?,
    };
    let rounded =
        temporal_round_i128_to_increment(cx, raw_units, increment, options.rounding_mode)?;
    let duration = temporal_duration_from_date_units(cx, rounded, options.largest_unit, unit_kind)?;
    validate_temporal_duration(cx, duration)?;
    let prototype = current_temporal_duration_prototype(cx)?;
    allocate_temporal_duration_object(cx, prototype, duration)
}

fn temporal_plain_date_since_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_plain_date_difference_builtin(cx, invocation, 1)
}

fn temporal_plain_date_until_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_plain_date_difference_builtin(cx, invocation, -1)
}

fn temporal_plain_date_to_plain_date_time_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let date = temporal_plain_date_data(cx, invocation.this_value())?;
    let time = match invocation.arguments().first().copied() {
        Some(value) if !value.is_undefined() => temporal_plain_time_from_value(cx, value)?,
        _ => TemporalPlainTimeObjectData::new(0, 0, 0, 0, 0, 0),
    };
    let data = TemporalPlainDateTimeObjectData::new(
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
    let total_nanoseconds =
        temporal_plain_date_time_total_nanoseconds(data).ok_or_else(|| range_error(cx))?;
    if !temporal_plain_date_time_is_within_limits(date.calendar(), total_nanoseconds) {
        return Err(range_error(cx));
    }
    let prototype = current_temporal_plain_date_time_prototype(cx)?;
    allocate_temporal_plain_date_time_object(cx, prototype, data)
}

fn temporal_reject_calendar_or_time_zone_properties<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_time_is_within_limits(calendar: AtomId, total_nanoseconds: i128) -> bool {
    let min = temporal_plain_date_ordinal_day(TemporalPlainDateObjectData::new(
        -271_821, 4, 19, calendar,
    )) * TEMPORAL_NANOS_PER_DAY;
    let max =
        temporal_plain_date_ordinal_day(TemporalPlainDateObjectData::new(275_760, 9, 13, calendar))
            * TEMPORAL_NANOS_PER_DAY
            + (TEMPORAL_NANOS_PER_DAY - 1);
    total_nanoseconds > min && total_nanoseconds <= max
}

fn temporal_plain_date_to_zoned_date_time_builtin<Cx: PublicBuiltinDispatchContext>(
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
    let data = temporal_zoned_date_time_from_parts(cx, instant.epoch_nanoseconds, &time_zone_id)?;
    let prototype = current_temporal_zoned_date_time_prototype(cx)?;
    allocate_temporal_zoned_date_time_object(cx, prototype, data)
}

fn temporal_plain_date_to_plain_year_month_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let date = temporal_plain_date_data(cx, invocation.this_value())?;
    let data = TemporalPlainYearMonthObjectData::new(
        date.year(),
        date.month(),
        date.day(),
        date.calendar(),
    );
    let prototype = current_temporal_plain_year_month_prototype(cx)?;
    allocate_temporal_plain_year_month_object(cx, prototype, data)
}

fn temporal_plain_date_to_plain_month_day_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let date = temporal_plain_date_data(cx, invocation.this_value())?;
    let data = TemporalPlainMonthDayObjectData::new(
        date.month(),
        date.day(),
        date.year(),
        date.calendar(),
    );
    let prototype = current_temporal_plain_month_day_prototype(cx)?;
    allocate_temporal_plain_month_day_object(cx, prototype, data)
}

fn temporal_plain_date_from_builtin<Cx: PublicBuiltinDispatchContext>(
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
                temporal_plain_date_from_bag_fields(cx, fields, overflow)?
            }
        }
    };
    let prototype = current_temporal_plain_date_prototype(cx)?;
    allocate_temporal_plain_date_object(cx, prototype, data)
}

fn temporal_plain_date_compare_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_time_part_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<i64, Cx::Error> {
    if value.is_undefined() {
        return Ok(0);
    }
    temporal_integer_part_from_value(cx, value)
}

fn temporal_time_part_from_argument<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_time_part_from_property<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_optional_time_part_from_property<Cx: PublicBuiltinDispatchContext>(
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum TemporalOverflow {
    Constrain,
    Reject,
}

fn temporal_overflow_from_options<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_time_from_parts<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_time_from_parts_with_overflow<Cx: PublicBuiltinDispatchContext>(
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

fn allocate_temporal_plain_time_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: ObjectRef,
    data: TemporalPlainTimeObjectData,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let root_shape = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.root_shape())
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
                        TemporalObjectKind::PlainTime,
                    ))),
                AllocationLifetime::Default,
            )
        })
    };
    let installed = cx
        .agent()
        .objects_mut()
        .install_temporal_object(object, TemporalObjectData::PlainTime(data));
    if !installed {
        return Err(type_error(cx));
    }
    Ok(Value::from_object_ref(object))
}

fn temporal_subsecond_parts_from_nanoseconds<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_time_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalPlainTimeObjectData, Cx::Error> {
    temporal_plain_time_from_value_with_overflow(cx, value, TemporalOverflow::Constrain)
}

fn temporal_plain_time_from_value_with_overflow<Cx: PublicBuiltinDispatchContext>(
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

struct TemporalPlainTimeParts {
    hour: i64,
    minute: i64,
    second: i64,
    millisecond: i64,
    microsecond: i64,
    nanosecond: i64,
}

fn temporal_plain_time_parts_from_property_bag<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_time_data<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    this_value: Value,
) -> Result<TemporalPlainTimeObjectData, Cx::Error> {
    let payload = {
        let agent = cx.agent();
        object::require_temporal_object(agent, this_value, TemporalObjectKind::PlainTime)
    };
    let payload = map_completion(cx, payload)?;
    let TemporalObjectData::PlainTime(data) = payload else {
        return Err(type_error(cx));
    };
    Ok(data)
}

fn temporal_plain_time_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let Some(new_target) = invocation.new_target() else {
        return Err(type_error(cx));
    };
    let hour = temporal_time_part_from_argument(cx, invocation, 0)?;
    let minute = temporal_time_part_from_argument(cx, invocation, 1)?;
    let second = temporal_time_part_from_argument(cx, invocation, 2)?;
    let millisecond = temporal_time_part_from_argument(cx, invocation, 3)?;
    let microsecond = temporal_time_part_from_argument(cx, invocation, 4)?;
    let nanosecond = temporal_time_part_from_argument(cx, invocation, 5)?;
    let data = temporal_plain_time_from_parts(
        cx,
        hour,
        minute,
        second,
        millisecond,
        microsecond,
        nanosecond,
    )?;
    let prototype = temporal_constructor_prototype(cx, new_target)?;
    allocate_temporal_plain_time_object(cx, prototype, data)
}

fn temporal_plain_time_hour_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.hour())))
}

fn temporal_plain_time_minute_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.minute())))
}

fn temporal_plain_time_second_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.second())))
}

fn temporal_plain_time_millisecond_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.millisecond())))
}

fn temporal_plain_time_microsecond_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.microsecond())))
}

fn temporal_plain_time_nanosecond_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.nanosecond())))
}

fn temporal_plain_time_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_time_data(cx, invocation.this_value())?;
    let (precision, rounding_mode) = temporal_instant_to_string_options(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let data = temporal_plain_time_for_string_precision(cx, data, precision, rounding_mode)?;
    Ok(string_value(
        cx,
        &format_temporal_plain_time_with_precision(data, precision),
    ))
}

fn temporal_plain_time_to_json_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_time_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format_temporal_plain_time(data)))
}

fn temporal_plain_time_to_locale_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_time_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format_temporal_plain_time(data)))
}

fn temporal_plain_time_for_string_precision<Cx: PublicBuiltinDispatchContext>(
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

fn format_temporal_plain_time_with_precision(
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
    text.push_str(&format!(":{:02}", data.second()));
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

fn temporal_plain_time_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Err(type_error(cx))
}

fn temporal_plain_time_equals_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = temporal_plain_time_data(cx, invocation.this_value())?;
    let right = temporal_plain_time_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(Value::from_bool(left == right))
}

fn temporal_plain_time_with_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let time = temporal_plain_time_data(cx, invocation.this_value())?;
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
    let hour = temporal_optional_time_part_from_property(cx, object_ref, "hour")?;
    let microsecond = temporal_optional_time_part_from_property(cx, object_ref, "microsecond")?;
    let millisecond = temporal_optional_time_part_from_property(cx, object_ref, "millisecond")?;
    let minute = temporal_optional_time_part_from_property(cx, object_ref, "minute")?;
    let nanosecond = temporal_optional_time_part_from_property(cx, object_ref, "nanosecond")?;
    let second = temporal_optional_time_part_from_property(cx, object_ref, "second")?;
    if hour.is_none()
        && microsecond.is_none()
        && millisecond.is_none()
        && minute.is_none()
        && nanosecond.is_none()
        && second.is_none()
    {
        return Err(type_error(cx));
    }
    let overflow = temporal_overflow_from_options(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let data = temporal_plain_time_from_parts_with_overflow(
        cx,
        hour.unwrap_or(i64::from(time.hour())),
        minute.unwrap_or(i64::from(time.minute())),
        second.unwrap_or(i64::from(time.second())),
        millisecond.unwrap_or(i64::from(time.millisecond())),
        microsecond.unwrap_or(i64::from(time.microsecond())),
        nanosecond.unwrap_or(i64::from(time.nanosecond())),
        overflow,
    )?;
    let prototype = current_temporal_plain_time_prototype(cx)?;
    allocate_temporal_plain_time_object(cx, prototype, data)
}

fn temporal_plain_time_from_nanoseconds<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    nanoseconds: i128,
) -> Result<TemporalPlainTimeObjectData, Cx::Error> {
    temporal_ops::plain_time_from_nanoseconds(nanoseconds).ok_or_else(|| range_error(cx))
}

fn temporal_plain_time_add_duration<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    time: TemporalPlainTimeObjectData,
    duration: TemporalDurationObjectData,
) -> Result<TemporalPlainTimeObjectData, Cx::Error> {
    temporal_plain_time_from_nanoseconds(
        cx,
        temporal_plain_time_nanoseconds(time) + temporal_duration_time_nanoseconds(duration),
    )
}

fn temporal_plain_time_add_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let time = temporal_plain_time_data(cx, invocation.this_value())?;
    let duration = temporal_duration_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let data = temporal_plain_time_add_duration(cx, time, duration)?;
    let prototype = current_temporal_plain_time_prototype(cx)?;
    allocate_temporal_plain_time_object(cx, prototype, data)
}

fn temporal_plain_time_subtract_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let time = temporal_plain_time_data(cx, invocation.this_value())?;
    let duration = temporal_duration_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let data = temporal_plain_time_add_duration(cx, time, negate_temporal_duration(duration))?;
    let prototype = current_temporal_plain_time_prototype(cx)?;
    allocate_temporal_plain_time_object(cx, prototype, data)
}

fn temporal_plain_time_round_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let time = temporal_plain_time_data(cx, invocation.this_value())?;
    let options = temporal_exact_time_round_options(
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
    let rounded = temporal_round_epoch_nanoseconds_to_increment(
        temporal_plain_time_nanoseconds(time),
        increment,
        options.rounding_mode,
    )
    .ok_or_else(|| range_error(cx))?
    .rem_euclid(TEMPORAL_NANOS_PER_DAY);
    let data = temporal_plain_time_from_nanoseconds(cx, rounded)?;
    let prototype = current_temporal_plain_time_prototype(cx)?;
    allocate_temporal_plain_time_object(cx, prototype, data)
}

struct TemporalPlainTimeDifferenceOptions {
    largest_unit: TemporalBuiltinDurationExactUnit,
    smallest_unit: TemporalBuiltinDurationExactUnit,
    rounding_increment: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
}

fn temporal_plain_time_difference_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalPlainTimeDifferenceOptions, Cx::Error> {
    if value.is_undefined() {
        return Ok(TemporalPlainTimeDifferenceOptions {
            largest_unit: TemporalBuiltinDurationExactUnit::Hour,
            smallest_unit: TemporalBuiltinDurationExactUnit::Nanosecond,
            rounding_increment: 1,
            rounding_mode: TemporalBuiltinRoundingMode::Trunc,
        });
    }
    let Some(object_ref) = value.as_object_ref() else {
        return Err(type_error(cx));
    };

    let largest_unit_value = temporal_property_value(cx, object_ref, "largestUnit")?;
    let largest_unit_text = temporal_option_string_text(cx, largest_unit_value)?;
    let rounding_increment_value = temporal_property_value(cx, object_ref, "roundingIncrement")?;
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
            TemporalBuiltinDurationExactUnit::Hour
        } else {
            temporal_exact_time_unit_from_text(text).ok_or_else(|| range_error(cx))?
        }
    } else {
        TemporalBuiltinDurationExactUnit::Hour
    };
    if temporal_exact_time_unit_order(largest_unit) > temporal_exact_time_unit_order(smallest_unit)
        || !temporal_exact_time_rounding_increment_is_valid(smallest_unit, rounding_increment)
    {
        return Err(range_error(cx));
    }

    Ok(TemporalPlainTimeDifferenceOptions {
        largest_unit,
        smallest_unit,
        rounding_increment,
        rounding_mode,
    })
}

fn temporal_plain_time_difference_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    sign: i128,
) -> Result<Value, Cx::Error> {
    let time = temporal_plain_time_data(cx, invocation.this_value())?;
    let other = temporal_plain_time_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let options = temporal_plain_time_difference_options(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let raw_difference = temporal_plain_time_nanoseconds(time)
        .checked_sub(temporal_plain_time_nanoseconds(other))
        .and_then(|difference| difference.checked_mul(sign))
        .ok_or_else(|| range_error(cx))?;
    let increment = temporal_exact_time_unit_nanoseconds(options.smallest_unit)
        .checked_mul(options.rounding_increment)
        .ok_or_else(|| range_error(cx))?;
    let rounded = temporal_round_epoch_nanoseconds_to_increment(
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

fn temporal_plain_time_since_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_plain_time_difference_builtin(cx, invocation, 1)
}

fn temporal_plain_time_until_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_plain_time_difference_builtin(cx, invocation, -1)
}

fn temporal_plain_time_to_plain_date_time_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let time = temporal_plain_time_data(cx, invocation.this_value())?;
    let date = temporal_plain_date_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let data = TemporalPlainDateTimeObjectData::new(
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
    let prototype = current_temporal_plain_date_time_prototype(cx)?;
    allocate_temporal_plain_date_time_object(cx, prototype, data)
}

fn temporal_plain_time_from_builtin<Cx: PublicBuiltinDispatchContext>(
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
        let data =
            temporal_plain_time_from_value_with_overflow(cx, value, TemporalOverflow::Constrain)?;
        let _overflow = temporal_overflow_from_options(cx, options)?;
        data
    } else if let Some(object_ref) = value.as_object_ref() {
        let temporal = {
            let agent = cx.agent();
            agent.objects().temporal_object(object_ref).copied()
        };
        if temporal.is_none() {
            let parts = temporal_plain_time_parts_from_property_bag(cx, object_ref)?;
            let overflow = temporal_overflow_from_options(cx, options)?;
            temporal_plain_time_from_parts_with_overflow(
                cx,
                parts.hour,
                parts.minute,
                parts.second,
                parts.millisecond,
                parts.microsecond,
                parts.nanosecond,
                overflow,
            )?
        } else {
            let data = temporal_plain_time_from_value_with_overflow(
                cx,
                value,
                TemporalOverflow::Constrain,
            )?;
            let _overflow = temporal_overflow_from_options(cx, options)?;
            data
        }
    } else {
        let data =
            temporal_plain_time_from_value_with_overflow(cx, value, TemporalOverflow::Constrain)?;
        let _overflow = temporal_overflow_from_options(cx, options)?;
        data
    };
    let prototype = current_temporal_plain_time_prototype(cx)?;
    allocate_temporal_plain_time_object(cx, prototype, data)
}

fn temporal_plain_time_compare_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = temporal_plain_time_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let right = temporal_plain_time_from_value(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    Ok(temporal_compare_ordering(
        (
            left.hour(),
            left.minute(),
            left.second(),
            left.millisecond(),
            left.microsecond(),
            left.nanosecond(),
        )
            .cmp(&(
                right.hour(),
                right.minute(),
                right.second(),
                right.millisecond(),
                right.microsecond(),
                right.nanosecond(),
            )),
    ))
}

fn temporal_plain_date_time_from_parts<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_time_from_parts_with_overflow<Cx: PublicBuiltinDispatchContext>(
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
    Ok(TemporalPlainDateTimeObjectData::new(
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
    ))
}

fn allocate_temporal_plain_date_time_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: ObjectRef,
    data: TemporalPlainDateTimeObjectData,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let root_shape = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.root_shape())
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

fn temporal_plain_date_time_from_value<Cx: PublicBuiltinDispatchContext>(
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
    let year = temporal_required_integer_part_from_property(cx, object_ref, "year")?;
    let month = temporal_month_from_property_bag(cx, object_ref, None)?;
    let day = temporal_required_integer_part_from_property(cx, object_ref, "day")?;
    let hour = temporal_time_part_from_property(cx, object_ref, "hour")?;
    let minute = temporal_time_part_from_property(cx, object_ref, "minute")?;
    let second = temporal_time_part_from_property(cx, object_ref, "second")?;
    let millisecond = temporal_time_part_from_property(cx, object_ref, "millisecond")?;
    let microsecond = temporal_time_part_from_property(cx, object_ref, "microsecond")?;
    let nanosecond = temporal_time_part_from_property(cx, object_ref, "nanosecond")?;
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

fn temporal_plain_date_time_from_bag_fields<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    fields: TemporalPlainDateTimeBagFields,
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

fn temporal_plain_date_time_data<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_time_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_time_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(data.year()))
}

fn temporal_plain_date_time_month_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.month())))
}

fn temporal_plain_date_time_month_code_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(string_value(cx, &format!("M{:02}", data.month())))
}

fn temporal_plain_date_time_day_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.day())))
}

fn temporal_plain_date_time_day_of_week_getter_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_time_day_of_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_time_days_in_month_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(temporal_iso_days_in_month(
        data.year(),
        data.month(),
    ))))
}

fn temporal_plain_date_time_days_in_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(temporal_iso_days_in_year(data.year())))
}

fn temporal_plain_date_time_months_in_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(12))
}

fn temporal_plain_date_time_in_leap_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_bool(temporal_is_iso_leap_year(data.year())))
}

fn temporal_plain_date_time_days_in_week_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(7))
}

fn temporal_plain_date_time_week_of_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    let (week, _) = temporal_iso_week_of_year(data.year(), data.month(), data.day());
    Ok(Value::from_smi(week))
}

fn temporal_plain_date_time_year_of_week_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    let (_, year) = temporal_iso_week_of_year(data.year(), data.month(), data.day());
    Ok(Value::from_smi(year))
}

fn temporal_plain_date_time_era_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::undefined())
}

fn temporal_plain_date_time_era_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::undefined())
}

fn temporal_plain_date_time_hour_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.hour())))
}

fn temporal_plain_date_time_minute_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.minute())))
}

fn temporal_plain_date_time_second_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.second())))
}

fn temporal_plain_date_time_millisecond_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.millisecond())))
}

fn temporal_plain_date_time_microsecond_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.microsecond())))
}

fn temporal_plain_date_time_nanosecond_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(i32::from(data.nanosecond())))
}

fn temporal_plain_date_time_calendar_id_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    Ok(string_value(cx, "iso8601"))
}

fn temporal_plain_date_time_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
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
struct TemporalPlainDateTimeToStringOptions {
    precision: TemporalInstantStringPrecision,
    rounding_mode: TemporalBuiltinRoundingMode,
    calendar_name: TemporalZonedDateTimeCalendarNameOption,
}

fn temporal_plain_date_time_to_string_options<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_time_for_string_precision<Cx: PublicBuiltinDispatchContext>(
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
    temporal_plain_date_time_from_total_nanoseconds(cx, total_nanoseconds)
}

fn format_temporal_plain_date_time_with_options(
    data: TemporalPlainDateTimeObjectData,
    options: TemporalPlainDateTimeToStringOptions,
) -> String {
    let mut text = {
        let date = temporal_plain_date_time_date(data);
        let time = temporal_plain_date_time_time(data);
        format!(
            "{}T{}",
            format_temporal_plain_date(date),
            format_temporal_plain_time_with_precision(time, options.precision)
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

fn temporal_plain_date_time_to_json_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_time_to_locale_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_plain_date_time_to_json_builtin(cx, invocation)
}

fn temporal_plain_date_time_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Err(type_error(cx))
}

fn temporal_plain_date_time_equals_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_time_with_builtin<Cx: PublicBuiltinDispatchContext>(
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
    temporal_reject_calendar_or_time_zone_properties(cx, object_ref)?;
    let overflow = temporal_overflow_from_options(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let year = temporal_optional_integer_part_from_property(cx, object_ref, "year")?;
    let month =
        temporal_month_from_property_bag(cx, object_ref, Some(i64::from(date_time.month())))?;
    let month_value_missing =
        temporal_optional_integer_part_from_property(cx, object_ref, "month")?.is_none();
    let month_code_missing = temporal_property_value(cx, object_ref, "monthCode")?.is_undefined();
    let day = temporal_optional_integer_part_from_property(cx, object_ref, "day")?
        .unwrap_or(i64::from(date_time.day()));
    let hour = temporal_optional_time_part_from_property(cx, object_ref, "hour")?
        .unwrap_or(i64::from(date_time.hour()));
    let minute = temporal_optional_time_part_from_property(cx, object_ref, "minute")?
        .unwrap_or(i64::from(date_time.minute()));
    let second = temporal_optional_time_part_from_property(cx, object_ref, "second")?
        .unwrap_or(i64::from(date_time.second()));
    let millisecond = temporal_optional_time_part_from_property(cx, object_ref, "millisecond")?
        .unwrap_or(i64::from(date_time.millisecond()));
    let microsecond = temporal_optional_time_part_from_property(cx, object_ref, "microsecond")?
        .unwrap_or(i64::from(date_time.microsecond()));
    let nanosecond = temporal_optional_time_part_from_property(cx, object_ref, "nanosecond")?
        .unwrap_or(i64::from(date_time.nanosecond()));
    if year.is_none()
        && month_value_missing
        && month_code_missing
        && day == i64::from(date_time.day())
        && hour == i64::from(date_time.hour())
        && minute == i64::from(date_time.minute())
        && second == i64::from(date_time.second())
        && millisecond == i64::from(date_time.millisecond())
        && microsecond == i64::from(date_time.microsecond())
        && nanosecond == i64::from(date_time.nanosecond())
    {
        return Err(type_error(cx));
    }
    let data = temporal_plain_date_time_from_parts_with_overflow(
        cx,
        year.unwrap_or(i64::from(date_time.year())),
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
    let prototype = current_temporal_plain_date_time_prototype(cx)?;
    allocate_temporal_plain_date_time_object(cx, prototype, data)
}

fn temporal_plain_date_time_with_plain_time_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_time_with_calendar_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_time_date(
    data: TemporalPlainDateTimeObjectData,
) -> TemporalPlainDateObjectData {
    TemporalPlainDateObjectData::new(data.year(), data.month(), data.day(), data.calendar())
}

fn temporal_plain_date_time_time(
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

fn temporal_plain_date_time_total_nanoseconds(
    data: TemporalPlainDateTimeObjectData,
) -> Option<i128> {
    temporal_plain_date_ordinal_day(temporal_plain_date_time_date(data))
        .checked_mul(TEMPORAL_NANOS_PER_DAY)?
        .checked_add(temporal_plain_time_nanoseconds(
            temporal_plain_date_time_time(data),
        ))
}

fn temporal_plain_date_time_from_total_nanoseconds<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    total_nanoseconds: i128,
) -> Result<TemporalPlainDateTimeObjectData, Cx::Error> {
    let ordinal_day = total_nanoseconds.div_euclid(TEMPORAL_NANOS_PER_DAY);
    let time_nanoseconds = total_nanoseconds.rem_euclid(TEMPORAL_NANOS_PER_DAY);
    let date = temporal_plain_date_from_ordinal_day(cx, ordinal_day)?;
    let time = temporal_plain_time_from_nanoseconds(cx, time_nanoseconds)?;
    Ok(TemporalPlainDateTimeObjectData::new(
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
    ))
}

fn temporal_plain_date_time_add_duration<Cx: PublicBuiltinDispatchContext>(
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
    let date = temporal_plain_date_add_duration(
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
    let ordinal_day = match temporal_plain_date_ordinal_day(date).checked_add(day_carry) {
        Some(ordinal_day) => ordinal_day,
        None => return Err(range_error(cx)),
    };
    let date = temporal_plain_date_from_ordinal_day(cx, ordinal_day)?;
    Ok(TemporalPlainDateTimeObjectData::new(
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
    ))
}

fn temporal_plain_date_time_add_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_time_subtract_builtin<Cx: PublicBuiltinDispatchContext>(
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
enum TemporalDateTimeDifferenceUnit {
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

fn temporal_date_time_difference_unit_from_text(
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

fn temporal_date_time_difference_unit_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalDateTimeDifferenceUnit, Cx::Error> {
    let string_ref = to_string_string_ref(cx, value)?;
    let text = string_ref_text(cx, string_ref)?;
    temporal_date_time_difference_unit_from_text(&text).ok_or_else(|| range_error(cx))
}

fn temporal_date_time_difference_unit_order(unit: TemporalDateTimeDifferenceUnit) -> u8 {
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

fn temporal_date_time_difference_unit_nanoseconds(
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

fn temporal_date_time_exact_unit(
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

fn temporal_date_time_rounding_increment_is_valid(
    smallest_unit: TemporalDateTimeDifferenceUnit,
    rounding_increment: i128,
) -> bool {
    match smallest_unit {
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
        TemporalDateTimeDifferenceUnit::Year
        | TemporalDateTimeDifferenceUnit::Month
        | TemporalDateTimeDifferenceUnit::Week => false,
    }
}

struct TemporalPlainDateTimeRoundOptions {
    smallest_unit: TemporalDateTimeDifferenceUnit,
    rounding_increment: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
}

fn temporal_plain_date_time_round_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalPlainDateTimeRoundOptions, Cx::Error> {
    if value.is_string() {
        let smallest_unit = temporal_date_time_difference_unit_from_value(cx, value)?;
        if !temporal_date_time_rounding_increment_is_valid(smallest_unit, 1) {
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
    if !temporal_date_time_rounding_increment_is_valid(smallest_unit, rounding_increment) {
        return Err(range_error(cx));
    }
    Ok(TemporalPlainDateTimeRoundOptions {
        smallest_unit,
        rounding_increment,
        rounding_mode,
    })
}

fn temporal_plain_date_time_round_builtin<Cx: PublicBuiltinDispatchContext>(
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
    let data = temporal_plain_date_time_from_total_nanoseconds(cx, rounded)?;
    let prototype = current_temporal_plain_date_time_prototype(cx)?;
    allocate_temporal_plain_date_time_object(cx, prototype, data)
}

struct TemporalPlainDateTimeDifferenceOptions {
    largest_unit: TemporalDateTimeDifferenceUnit,
    smallest_unit: TemporalDateTimeDifferenceUnit,
    rounding_increment: i128,
    rounding_mode: TemporalBuiltinRoundingMode,
}

fn temporal_plain_date_time_difference_options<Cx: PublicBuiltinDispatchContext>(
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
    let largest_unit = if largest_unit_value.is_undefined() {
        default_largest_unit
    } else {
        let string_ref = to_string_string_ref(cx, largest_unit_value)?;
        let text = string_ref_text(cx, string_ref)?;
        if text == "auto" {
            default_largest_unit
        } else {
            temporal_date_time_difference_unit_from_text(&text).ok_or_else(|| range_error(cx))?
        }
    };
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

fn temporal_duration_from_date_time_nanoseconds<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_time_difference_builtin<Cx: PublicBuiltinDispatchContext>(
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
    let rounded = temporal_round_epoch_nanoseconds_to_increment(
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

fn temporal_plain_date_time_since_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_plain_date_time_difference_builtin(cx, invocation, 1)
}

fn temporal_plain_date_time_until_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_plain_date_time_difference_builtin(cx, invocation, -1)
}

fn temporal_plain_date_time_to_plain_date_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_plain_date_time_data(cx, invocation.this_value())?;
    let date =
        TemporalPlainDateObjectData::new(data.year(), data.month(), data.day(), data.calendar());
    let prototype = current_temporal_plain_date_prototype(cx)?;
    allocate_temporal_plain_date_object(cx, prototype, date)
}

fn temporal_plain_date_time_to_plain_time_builtin<Cx: PublicBuiltinDispatchContext>(
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
    allocate_temporal_plain_time_object(cx, prototype, time)
}

fn temporal_plain_date_time_to_zoned_date_time_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_disambiguation_from_options<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_plain_date_time_from_builtin<Cx: PublicBuiltinDispatchContext>(
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
                temporal_plain_date_time_from_bag_fields(cx, fields, overflow)?
            }
        }
    };
    let prototype = current_temporal_plain_date_time_prototype(cx)?;
    allocate_temporal_plain_date_time_object(cx, prototype, data)
}

fn temporal_plain_date_time_compare_builtin<Cx: PublicBuiltinDispatchContext>(
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
        TemporalOverflow::Constrain => month.clamp(1, 12),
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

fn allocate_temporal_plain_year_month_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: ObjectRef,
    data: TemporalPlainYearMonthObjectData,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let root_shape = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.root_shape())
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

    temporal_validate_optional_iso_calendar_property(cx, object_ref)?;
    let year = temporal_required_integer_part_from_property(cx, object_ref, "year")?;
    let month = temporal_month_from_property_bag(cx, object_ref, None)?;
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

    let left = i128::from(year_month.year()) * 12 + i128::from(year_month.month() - 1);
    let right = i128::from(other.year()) * 12 + i128::from(other.month() - 1);
    let raw_months = left
        .checked_sub(right)
        .and_then(|difference| difference.checked_mul(sign))
        .ok_or_else(|| range_error(cx))?;
    let increment = match options.smallest_unit {
        TemporalDateDifferenceUnit::Year => options
            .rounding_increment
            .checked_mul(12)
            .ok_or_else(|| range_error(cx))?,
        TemporalDateDifferenceUnit::Month => options.rounding_increment,
        TemporalDateDifferenceUnit::Week | TemporalDateDifferenceUnit::Day => {
            unreachable!("filtered lower units")
        }
    };
    let rounded =
        temporal_round_i128_to_increment(cx, raw_months, increment, options.rounding_mode)?;
    let duration = temporal_duration_from_date_units(
        cx,
        rounded,
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
                let date = temporal_plain_date_from_parts_with_overflow(
                    cx,
                    fields.year.expect("checked above"),
                    month,
                    1,
                    overflow,
                )?;
                TemporalPlainYearMonthObjectData::new(
                    date.year(),
                    date.month(),
                    date.day(),
                    date.calendar(),
                )
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

fn temporal_month_from_month_code_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<i64, Cx::Error> {
    let text = temporal_string_text_from_value(cx, value)?;
    temporal_month_from_month_code_text(&text).ok_or_else(|| range_error(cx))
}

enum TemporalMonthCodeSyntax {
    Standard(i64),
    Leap,
}

fn temporal_parse_month_code_syntax(text: &str) -> Option<TemporalMonthCodeSyntax> {
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

fn temporal_month_from_month_code_text(text: &str) -> Option<i64> {
    match temporal_parse_month_code_syntax(text)? {
        TemporalMonthCodeSyntax::Standard(month) if (1..=12).contains(&month) => Some(month),
        TemporalMonthCodeSyntax::Standard(_) | TemporalMonthCodeSyntax::Leap => None,
    }
}

fn temporal_month_from_property_bag<Cx: PublicBuiltinDispatchContext>(
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

fn allocate_temporal_plain_month_day_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: ObjectRef,
    data: TemporalPlainMonthDayObjectData,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let root_shape = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.root_shape())
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
            i64::from(TEMPORAL_DEFAULT_PLAIN_MONTH_DAY_REFERENCE_YEAR),
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
        i64::from(TEMPORAL_DEFAULT_PLAIN_MONTH_DAY_REFERENCE_YEAR),
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
        Some(_) => TEMPORAL_DEFAULT_PLAIN_MONTH_DAY_REFERENCE_YEAR,
        None => TEMPORAL_DEFAULT_PLAIN_MONTH_DAY_REFERENCE_YEAR,
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
                    i64::from(TEMPORAL_DEFAULT_PLAIN_MONTH_DAY_REFERENCE_YEAR),
                )?
            }
        }
    };
    let prototype = current_temporal_plain_month_day_prototype(cx)?;
    allocate_temporal_plain_month_day_object(cx, prototype, data)
}

fn temporal_time_zone_id_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<String, Cx::Error> {
    let string_ref = value.as_string_ref().ok_or_else(|| type_error(cx))?;
    let text = string_ref_text(cx, string_ref)?;
    temporal_time_zone_id_from_string(&text).ok_or_else(|| range_error(cx))
}

fn temporal_time_zone_id_from_string(text: &str) -> Option<String> {
    if text.eq_ignore_ascii_case(TEMPORAL_UTC_TIME_ZONE_ID) {
        return Some(TEMPORAL_UTC_TIME_ZONE_ID.to_string());
    }
    if let Some(offset_nanoseconds) = temporal_parse_fixed_offset_time_zone_id(&text) {
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

fn temporal_parse_fixed_offset_time_zone_id(text: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    let sign = match bytes.first().copied()? {
        b'+' => 1_i128,
        b'-' => -1_i128,
        _ => return None,
    };
    fn parse_two_digits(bytes: &[u8], index: &mut usize) -> Option<i128> {
        let tens = *bytes.get(*index)?;
        let ones = *bytes.get(*index + 1)?;
        if !tens.is_ascii_digit() || !ones.is_ascii_digit() {
            return None;
        }
        *index += 2;
        Some(i128::from(tens - b'0') * 10 + i128::from(ones - b'0'))
    }

    let mut index = 1;
    let hours = parse_two_digits(bytes, &mut index)?;
    let mut minutes = 0_i128;
    if index < bytes.len() {
        if bytes[index] == b':' {
            index += 1;
        }
        minutes = parse_two_digits(bytes, &mut index)?;
    }
    if index != bytes.len() || hours > 23 || minutes > 59 {
        return None;
    }
    let total = (hours * 60 + minutes)
        .checked_mul(TEMPORAL_NANOS_PER_MINUTE)?
        .checked_mul(sign)?;
    i64::try_from(total).ok()
}

fn temporal_time_zone_id_from_iso_date_time_offset(text: &str) -> Option<String> {
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

fn temporal_validate_iso_calendar_value<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_validate_iso_calendar_identifier_value<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_validate_optional_iso_calendar_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
) -> Result<(), Cx::Error> {
    let value = temporal_property_value(cx, object_ref, "calendar")?;
    if value.is_undefined() {
        return Ok(());
    }
    temporal_validate_iso_calendar_value(cx, value)
}

fn temporal_validate_optional_iso_calendar_identifier_argument<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_is_valid_iso_calendar_string(text: &str) -> bool {
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

fn temporal_time_zone_id_from_optional_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<String, Cx::Error> {
    if value.is_undefined() {
        let zone = cx.temporal_default_time_zone(&TemporalDefaultTimeZoneRequest {})?;
        return Ok(zone.time_zone_id);
    }
    temporal_time_zone_id_from_value(cx, value)
}

fn temporal_now_instant_and_civil<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_atom_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    atom: lyng_js_common::AtomId,
) -> Result<String, Cx::Error> {
    cx.agent()
        .atoms()
        .get(atom)
        .map(str::to_string)
        .ok_or_else(|| type_error(cx))
}

fn temporal_zoned_date_time_from_parts<Cx: PublicBuiltinDispatchContext>(
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

fn allocate_temporal_zoned_date_time_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: ObjectRef,
    data: TemporalZonedDateTimeObjectData,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let root_shape = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.root_shape())
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

fn temporal_zoned_date_time_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalZonedDateTimeObjectData, Cx::Error> {
    if let Some(string_ref) = value.as_string_ref() {
        let text = string_ref_text(cx, string_ref)?;
        let time_zone_id =
            temporal_zoned_date_time_zone_annotation(&text).ok_or_else(|| range_error(cx))?;
        let epoch_nanoseconds = if let Some(epoch_nanoseconds) = parse_temporal_instant(&text) {
            epoch_nanoseconds
        } else {
            let parsed = parse_temporal_plain_date_time(&text).ok_or_else(|| range_error(cx))?;
            let (millisecond, microsecond, nanosecond) =
                temporal_subsecond_parts_from_nanoseconds(cx, parsed.fraction_nanoseconds)?;
            let millisecond = match u16::try_from(millisecond) {
                Ok(value) => value,
                Err(_) => return Err(range_error(cx)),
            };
            let microsecond = match u16::try_from(microsecond) {
                Ok(value) => value,
                Err(_) => return Err(range_error(cx)),
            };
            let nanosecond = match u16::try_from(nanosecond) {
                Ok(value) => value,
                Err(_) => return Err(range_error(cx)),
            };
            let date_time = TemporalCivilDateTime::new(
                parsed.year,
                parsed.month,
                parsed.day,
                parsed.hour,
                parsed.minute,
                parsed.second,
                millisecond,
                microsecond,
                nanosecond,
            );
            let instant = cx.temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
                time_zone_id: time_zone_id.clone(),
                date_time,
                disambiguation: TemporalDisambiguation::Compatible,
            })?;
            instant.epoch_nanoseconds
        };
        return temporal_zoned_date_time_from_parts(cx, epoch_nanoseconds, &time_zone_id);
    }

    let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let existing = {
        let agent = cx.agent();
        agent.objects().temporal_object(object_ref).copied()
    };
    if let Some(TemporalObjectData::ZonedDateTime(data)) = existing {
        return Ok(data);
    }
    temporal_validate_optional_iso_calendar_property(cx, object_ref)?;
    let time_zone = temporal_property_value(cx, object_ref, "timeZone")?;
    if time_zone.is_undefined() {
        return Err(type_error(cx));
    }
    let time_zone_id = temporal_time_zone_id_from_value(cx, time_zone)?;
    let date = temporal_plain_date_from_property_bag(cx, object_ref, false)?;
    let hour = temporal_optional_time_part_from_property(cx, object_ref, "hour")?.unwrap_or(0);
    let minute = temporal_optional_time_part_from_property(cx, object_ref, "minute")?.unwrap_or(0);
    let second = temporal_optional_time_part_from_property(cx, object_ref, "second")?.unwrap_or(0);
    let millisecond =
        temporal_optional_time_part_from_property(cx, object_ref, "millisecond")?.unwrap_or(0);
    let microsecond =
        temporal_optional_time_part_from_property(cx, object_ref, "microsecond")?.unwrap_or(0);
    let nanosecond =
        temporal_optional_time_part_from_property(cx, object_ref, "nanosecond")?.unwrap_or(0);
    let hour = match u8::try_from(hour) {
        Ok(value) => value,
        Err(_) => return Err(range_error(cx)),
    };
    let minute = match u8::try_from(minute) {
        Ok(value) => value,
        Err(_) => return Err(range_error(cx)),
    };
    let second = match u8::try_from(second) {
        Ok(value) => value,
        Err(_) => return Err(range_error(cx)),
    };
    let millisecond = match u16::try_from(millisecond) {
        Ok(value) => value,
        Err(_) => return Err(range_error(cx)),
    };
    let microsecond = match u16::try_from(microsecond) {
        Ok(value) => value,
        Err(_) => return Err(range_error(cx)),
    };
    let nanosecond = match u16::try_from(nanosecond) {
        Ok(value) => value,
        Err(_) => return Err(range_error(cx)),
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
    temporal_zoned_date_time_from_parts(cx, instant.epoch_nanoseconds, &time_zone_id)
}

fn temporal_zoned_date_time_zone_annotation(text: &str) -> Option<String> {
    let mut remaining = text;
    while let Some(start) = remaining.find('[') {
        let after_start = &remaining[start + 1..];
        let end = after_start.find(']')?;
        let body = after_start[..end].trim_start_matches('!');
        if body.eq_ignore_ascii_case(TEMPORAL_UTC_TIME_ZONE_ID) {
            return Some(TEMPORAL_UTC_TIME_ZONE_ID.to_string());
        }
        if let Some(offset_nanoseconds) = temporal_parse_fixed_offset_time_zone_id(body) {
            return Some(format_temporal_offset(offset_nanoseconds));
        }
        remaining = &after_start[end + 1..];
    }
    None
}

fn temporal_zoned_date_time_data<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_zoned_date_time_civil<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    data: TemporalZonedDateTimeObjectData,
) -> Result<TemporalCivilTime, Cx::Error> {
    let time_zone_id = temporal_atom_text(cx, data.time_zone())?;
    cx.temporal_instant_to_civil_time(&TemporalInstantToCivilRequest {
        time_zone_id,
        epoch_nanoseconds: data.epoch_nanoseconds(),
    })
}

fn format_temporal_zoned_date_time<Cx: PublicBuiltinDispatchContext>(
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
enum TemporalZonedDateTimeOffsetOption {
    Auto,
    Never,
}

#[derive(Clone, Copy)]
enum TemporalZonedDateTimeTimeZoneNameOption {
    Auto,
    Never,
    Critical,
}

#[derive(Clone, Copy)]
enum TemporalZonedDateTimeCalendarNameOption {
    Auto,
    Always,
    Never,
    Critical,
}

#[derive(Clone, Copy)]
struct TemporalZonedDateTimeToStringOptions {
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

fn temporal_zoned_date_time_to_string_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TemporalZonedDateTimeToStringOptions, Cx::Error> {
    if value.is_undefined() {
        return Ok(TemporalZonedDateTimeToStringOptions::default());
    }
    let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let (precision, rounding_mode) = temporal_instant_to_string_options(cx, value)?;
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
    let offset_value = temporal_property_value(cx, object_ref, "offset")?;
    let offset =
        match temporal_string_option(cx, offset_value, &["auto", "never"], "auto")?.as_str() {
            "auto" => TemporalZonedDateTimeOffsetOption::Auto,
            "never" => TemporalZonedDateTimeOffsetOption::Never,
            _ => unreachable!("temporal_string_option constrained offset"),
        };
    Ok(TemporalZonedDateTimeToStringOptions {
        precision,
        rounding_mode,
        offset,
        time_zone_name,
        calendar_name,
    })
}

fn temporal_zoned_date_time_epoch_for_string_precision<Cx: PublicBuiltinDispatchContext>(
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

fn format_temporal_civil_date_time_with_precision(
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

fn format_temporal_zoned_date_time_with_options<Cx: PublicBuiltinDispatchContext>(
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
            text.push_str(&format!("[{time_zone_id}]"));
        }
        TemporalZonedDateTimeTimeZoneNameOption::Critical => {
            text.push_str(&format!("[!{time_zone_id}]"));
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

fn temporal_zoned_date_time_builtin<Cx: PublicBuiltinDispatchContext>(
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
    let time_zone_id = temporal_time_zone_id_from_value(
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

fn temporal_zoned_date_time_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(civil.date_time.year))
}

fn temporal_zoned_date_time_month_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(i32::from(civil.date_time.month)))
}

fn temporal_zoned_date_time_month_code_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(string_value(cx, &format!("M{:02}", civil.date_time.month)))
}

fn temporal_zoned_date_time_day_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(i32::from(civil.date_time.day)))
}

fn temporal_zoned_date_time_day_of_week_getter_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_zoned_date_time_day_of_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_zoned_date_time_days_in_month_getter_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_zoned_date_time_days_in_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(temporal_iso_days_in_year(
        civil.date_time.year,
    )))
}

fn temporal_zoned_date_time_months_in_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(12))
}

fn temporal_zoned_date_time_in_leap_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_bool(temporal_is_iso_leap_year(
        civil.date_time.year,
    )))
}

fn temporal_zoned_date_time_days_in_week_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    Ok(Value::from_smi(7))
}

fn temporal_zoned_date_time_week_of_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    let date_time = civil.date_time;
    let (week, _) = temporal_iso_week_of_year(date_time.year, date_time.month, date_time.day);
    Ok(Value::from_smi(week))
}

fn temporal_zoned_date_time_year_of_week_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    let date_time = civil.date_time;
    let (_, year) = temporal_iso_week_of_year(date_time.year, date_time.month, date_time.day);
    Ok(Value::from_smi(year))
}

fn temporal_zoned_date_time_era_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    Ok(Value::undefined())
}

fn temporal_zoned_date_time_era_year_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    Ok(Value::undefined())
}

fn temporal_zoned_date_time_hour_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(i32::from(civil.date_time.hour)))
}

fn temporal_zoned_date_time_minute_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(i32::from(civil.date_time.minute)))
}

fn temporal_zoned_date_time_second_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(i32::from(civil.date_time.second)))
}

fn temporal_zoned_date_time_millisecond_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(i32::from(civil.date_time.millisecond)))
}

fn temporal_zoned_date_time_microsecond_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(i32::from(civil.date_time.microsecond)))
}

fn temporal_zoned_date_time_nanosecond_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_smi(i32::from(civil.date_time.nanosecond)))
}

fn temporal_zoned_date_time_epoch_nanoseconds_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    Ok(temporal_i128_to_bigint_value(
        cx.agent(),
        data.epoch_nanoseconds(),
    ))
}

fn temporal_zoned_date_time_epoch_milliseconds_getter_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_zoned_date_time_time_zone_id_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let time_zone_id = temporal_atom_text(cx, data.time_zone())?;
    Ok(string_value(cx, &time_zone_id))
}

fn temporal_zoned_date_time_calendar_id_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let _data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    Ok(string_value(cx, "iso8601"))
}

fn temporal_zoned_date_time_offset_getter_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_zoned_date_time_offset_nanoseconds_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let civil = temporal_zoned_date_time_civil(cx, data)?;
    Ok(Value::from_f64(civil.offset_nanoseconds as f64))
}

fn temporal_zoned_date_time_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_zoned_date_time_to_json_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let text = format_temporal_zoned_date_time(cx, data)?;
    Ok(string_value(cx, &text))
}

fn temporal_zoned_date_time_to_locale_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_zoned_date_time_to_json_builtin(cx, invocation)
}

fn temporal_zoned_date_time_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Err(type_error(cx))
}

fn temporal_zoned_date_time_equals_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_civil_date_time_from_plain_date_time(
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

fn temporal_zoned_date_time_add_duration<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    zoned: TemporalZonedDateTimeObjectData,
    duration: TemporalDurationObjectData,
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
    let added = temporal_plain_date_time_add_duration(
        cx,
        plain_date_time,
        duration,
        TemporalOverflow::Constrain,
    )?;
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

fn temporal_zoned_date_time_add_builtin<Cx: PublicBuiltinDispatchContext>(
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
    temporal_zoned_date_time_add_duration(cx, zoned, duration)
}

fn temporal_zoned_date_time_subtract_builtin<Cx: PublicBuiltinDispatchContext>(
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
    temporal_zoned_date_time_add_duration(cx, zoned, negate_temporal_duration(duration))
}

fn temporal_zoned_date_time_round_to_day<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    zoned: TemporalZonedDateTimeObjectData,
    rounding_mode: TemporalBuiltinRoundingMode,
) -> Result<i128, Cx::Error> {
    let civil = temporal_zoned_date_time_civil(cx, zoned)?.date_time;
    let local_nanoseconds = i128::from(civil.hour) * TEMPORAL_NANOS_PER_HOUR
        + i128::from(civil.minute) * TEMPORAL_NANOS_PER_MINUTE
        + i128::from(civil.second) * TEMPORAL_NANOS_PER_SECOND
        + i128::from(civil.millisecond) * TEMPORAL_NANOS_PER_MILLISECOND
        + i128::from(civil.microsecond) * TEMPORAL_NANOS_PER_MICROSECOND
        + i128::from(civil.nanosecond);
    let rounded = temporal_round_epoch_nanoseconds_to_increment(
        local_nanoseconds,
        TEMPORAL_NANOS_PER_DAY,
        rounding_mode,
    )
    .ok_or_else(|| range_error(cx))?;
    let (year, month, day) = if rounded == TEMPORAL_NANOS_PER_DAY {
        let date =
            TemporalPlainDateObjectData::new(civil.year, civil.month, civil.day, zoned.calendar());
        let next = temporal_plain_date_add_duration(
            cx,
            date,
            TemporalDurationObjectData::new(0, 0, 0, 1, 0, 0, 0, 0, 0, 0),
            TemporalOverflow::Constrain,
        )?;
        (next.year(), next.month(), next.day())
    } else if rounded == 0 {
        (civil.year, civil.month, civil.day)
    } else {
        return Err(range_error(cx));
    };
    let time_zone_id = temporal_atom_text(cx, zoned.time_zone())?;
    temporal_zoned_date_time_midnight_epoch_nanoseconds(cx, &time_zone_id, year, month, day)
}

fn temporal_zoned_date_time_round_builtin<Cx: PublicBuiltinDispatchContext>(
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
        temporal_round_epoch_nanoseconds_to_increment(
            zoned.epoch_nanoseconds(),
            increment,
            options.rounding_mode,
        )
        .ok_or_else(|| range_error(cx))?
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

fn temporal_zoned_date_time_with_builtin<Cx: PublicBuiltinDispatchContext>(
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
    let civil = temporal_zoned_date_time_civil(cx, zoned)?.date_time;
    let year = temporal_optional_integer_part_from_property(cx, object_ref, "year")?
        .unwrap_or(i64::from(civil.year));
    let month_value = temporal_optional_integer_part_from_property(cx, object_ref, "month")?;
    let month_code_value = temporal_property_value(cx, object_ref, "monthCode")?;
    let month = if let Some(month) = month_value {
        if !month_code_value.is_undefined() {
            let month_code = temporal_month_from_month_code_value(cx, month_code_value)?;
            if month != month_code {
                return Err(range_error(cx));
            }
        }
        month
    } else if month_code_value.is_undefined() {
        i64::from(civil.month)
    } else {
        temporal_month_from_month_code_value(cx, month_code_value)?
    };
    let day = temporal_optional_integer_part_from_property(cx, object_ref, "day")?
        .unwrap_or(i64::from(civil.day));
    let hour = temporal_optional_time_part_from_property(cx, object_ref, "hour")?
        .unwrap_or(i64::from(civil.hour));
    let minute = temporal_optional_time_part_from_property(cx, object_ref, "minute")?
        .unwrap_or(i64::from(civil.minute));
    let second = temporal_optional_time_part_from_property(cx, object_ref, "second")?
        .unwrap_or(i64::from(civil.second));
    let millisecond = temporal_optional_time_part_from_property(cx, object_ref, "millisecond")?
        .unwrap_or(i64::from(civil.millisecond));
    let microsecond = temporal_optional_time_part_from_property(cx, object_ref, "microsecond")?
        .unwrap_or(i64::from(civil.microsecond));
    let nanosecond = temporal_optional_time_part_from_property(cx, object_ref, "nanosecond")?
        .unwrap_or(i64::from(civil.nanosecond));
    let date_time = temporal_plain_date_time_from_parts(
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
    let time_zone_id = temporal_atom_text(cx, zoned.time_zone())?;
    let instant = cx.temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
        time_zone_id,
        date_time: temporal_civil_date_time_from_plain_date_time(date_time),
        disambiguation: TemporalDisambiguation::Compatible,
    })?;
    let data = TemporalZonedDateTimeObjectData::new(
        instant.epoch_nanoseconds,
        zoned.time_zone(),
        zoned.calendar(),
    );
    let prototype = current_temporal_zoned_date_time_prototype(cx)?;
    allocate_temporal_zoned_date_time_object(cx, prototype, data)
}

fn temporal_zoned_date_time_with_time_zone_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_zoned_date_time_with_calendar_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_zoned_date_time_with_plain_time_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_zoned_date_time_midnight_epoch_nanoseconds<Cx: PublicBuiltinDispatchContext>(
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
    Ok(instant.epoch_nanoseconds)
}

fn temporal_zoned_date_time_start_of_day_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_zoned_date_time_hours_in_day_getter_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_zoned_date_time_difference_builtin<Cx: PublicBuiltinDispatchContext>(
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
    let rounded = temporal_round_epoch_nanoseconds_to_increment(
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

fn temporal_zoned_date_time_since_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_zoned_date_time_difference_builtin(cx, invocation, 1)
}

fn temporal_zoned_date_time_until_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    temporal_zoned_date_time_difference_builtin(cx, invocation, -1)
}

fn temporal_zoned_date_time_from_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_from_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let prototype = current_temporal_zoned_date_time_prototype(cx)?;
    allocate_temporal_zoned_date_time_object(cx, prototype, data)
}

fn temporal_zoned_date_time_compare_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_zoned_date_time_to_instant_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let data = temporal_zoned_date_time_data(cx, invocation.this_value())?;
    let prototype = current_temporal_instant_prototype(cx)?;
    allocate_temporal_instant_object(cx, prototype, data.epoch_nanoseconds())
}

fn temporal_zoned_date_time_to_plain_date_time_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_zoned_date_time_to_plain_date_builtin<Cx: PublicBuiltinDispatchContext>(
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

fn temporal_zoned_date_time_to_plain_time_builtin<Cx: PublicBuiltinDispatchContext>(
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
    allocate_temporal_plain_time_object(cx, prototype, data)
}

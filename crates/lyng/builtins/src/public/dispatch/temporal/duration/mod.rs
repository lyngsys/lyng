use super::{
    current_temporal_duration_prototype, format_temporal_duration,
    format_temporal_duration_with_seconds_precision, map_completion, negate_temporal_duration,
    object, parse_temporal_duration, range_error, string_ref_text, string_value,
    temporal_compare_ordering, temporal_constructor_prototype, temporal_date_difference_unit_order,
    temporal_duration_default_largest_exact_unit,
    temporal_duration_exact_unit_allows_largest_smallest,
    temporal_duration_from_date_time_nanoseconds, temporal_duration_from_date_units,
    temporal_duration_from_nanoseconds_with_largest_unit, temporal_duration_sign,
    temporal_duration_time_nanoseconds, temporal_i128_as_number, temporal_integer_part_from_value,
    temporal_month_from_month_code_value, temporal_number_to_i128_after_range_check,
    temporal_number_to_u8_after_range_check, temporal_ops, temporal_parse_offset_string,
    temporal_plain_date_add_duration, temporal_plain_date_difference_trunc,
    temporal_plain_date_from_parts, temporal_plain_date_from_value,
    temporal_plain_date_ordinal_day, temporal_plain_date_time_add_duration,
    temporal_plain_date_time_date, temporal_plain_date_time_from_parts_with_overflow,
    temporal_plain_date_time_from_total_nanoseconds, temporal_plain_date_time_from_value,
    temporal_plain_date_time_is_within_limits, temporal_plain_date_time_time,
    temporal_plain_date_time_total_nanoseconds, temporal_plain_time_nanoseconds,
    temporal_property_value, temporal_round_duration_exact,
    temporal_round_duration_nanoseconds_to_increment, temporal_round_i128_to_increment,
    temporal_time_part_from_value, temporal_time_zone_id_from_value, temporal_total_duration_exact,
    temporal_total_nanoseconds_as_unit, temporal_validate_iso_calendar_value,
    temporal_zoned_date_time_add_duration, temporal_zoned_date_time_civil,
    temporal_zoned_date_time_data, temporal_zoned_date_time_explicit_offset,
    temporal_zoned_date_time_from_parts, temporal_zoned_date_time_from_value,
    temporal_zoned_date_time_zone_annotation, to_number_for_builtin, to_string_string_ref,
    type_error, AllocationLifetime, BuiltinFunctionId, BuiltinInvocation, ObjectAllocation,
    ObjectColdData, ObjectRef, OrdinaryObjectData, PublicBuiltinDispatchContext,
    TemporalBuiltinDurationExactUnit, TemporalBuiltinRoundingMode, TemporalCivilDateTime,
    TemporalCivilToInstantRequest, TemporalDateDifferenceUnit, TemporalDateTimeDifferenceUnit,
    TemporalDisambiguation, TemporalDurationObjectData, TemporalObjectData, TemporalObjectKind,
    TemporalOverflow, TemporalPlainDateObjectData, TemporalPlainDateTimeObjectData,
    TemporalZonedDateTimeObjectData, Value, TEMPORAL_NANOS_PER_DAY, TEMPORAL_NANOS_PER_HOUR,
    TEMPORAL_NANOS_PER_MICROSECOND, TEMPORAL_NANOS_PER_MILLISECOND, TEMPORAL_NANOS_PER_MINUTE,
    TEMPORAL_NANOS_PER_SECOND, TEMPORAL_SAFE_INTEGER_MAX,
};

pub(super) fn dispatch_temporal_duration_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::temporal_duration_builtin() {
        return temporal_duration_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_years_getter_builtin() {
        return temporal_duration_years_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_months_getter_builtin() {
        return temporal_duration_months_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_weeks_getter_builtin() {
        return temporal_duration_weeks_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_days_getter_builtin() {
        return temporal_duration_days_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_hours_getter_builtin() {
        return temporal_duration_hours_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_minutes_getter_builtin() {
        return temporal_duration_minutes_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_seconds_getter_builtin() {
        return temporal_duration_seconds_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_milliseconds_getter_builtin() {
        return temporal_duration_milliseconds_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_microseconds_getter_builtin() {
        return temporal_duration_microseconds_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_nanoseconds_getter_builtin() {
        return temporal_duration_nanoseconds_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_sign_getter_builtin() {
        return temporal_duration_sign_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_blank_getter_builtin() {
        return temporal_duration_blank_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_to_string_builtin() {
        return temporal_duration_to_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_to_json_builtin() {
        return temporal_duration_to_json_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_to_locale_string_builtin() {
        return temporal_duration_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_negated_builtin() {
        return temporal_duration_negated_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_abs_builtin() {
        return temporal_duration_abs_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_with_builtin() {
        return temporal_duration_with_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_round_builtin() {
        return temporal_duration_round_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_total_builtin() {
        return temporal_duration_total_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_add_builtin() {
        return temporal_duration_add_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_subtract_builtin() {
        return temporal_duration_subtract_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_value_of_builtin() {
        return temporal_duration_value_of_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_from_builtin() {
        return temporal_duration_from_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::temporal_duration_compare_builtin() {
        return temporal_duration_compare_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

mod arithmetic;
mod calendar_rounding;
mod construction;
mod conversion;
mod options;
mod round;
mod total;
mod units;

use arithmetic::{
    temporal_duration_add_builtin, temporal_duration_compare_builtin,
    temporal_duration_from_builtin, temporal_duration_subtract_builtin,
    temporal_duration_value_of_builtin,
};
use calendar_rounding::{
    temporal_duration_calendar_day_time_difference,
    temporal_duration_calendar_unit_to_date_difference_unit,
    temporal_duration_date_with_start_time_total_nanoseconds,
    temporal_duration_relative_start_plain_date_time, temporal_duration_relative_total_nanoseconds,
    temporal_duration_round_calendar_largest_unit, temporal_duration_round_calendar_smallest_unit,
};
use construction::{
    allocate_current_temporal_blank_duration_object, temporal_duration_abs_builtin,
    temporal_duration_blank_getter_builtin, temporal_duration_builtin, temporal_duration_data,
    temporal_duration_days_getter_builtin, temporal_duration_hours_getter_builtin,
    temporal_duration_microseconds_getter_builtin, temporal_duration_milliseconds_getter_builtin,
    temporal_duration_minutes_getter_builtin, temporal_duration_months_getter_builtin,
    temporal_duration_nanoseconds_getter_builtin, temporal_duration_negated_builtin,
    temporal_duration_seconds_getter_builtin, temporal_duration_sign_getter_builtin,
    temporal_duration_to_json_builtin, temporal_duration_to_locale_string_builtin,
    temporal_duration_to_string_builtin, temporal_duration_weeks_getter_builtin,
    temporal_duration_with_builtin, temporal_duration_years_getter_builtin,
};
use conversion::{
    temporal_duration_from_additive_argument_with_largest_unit,
    temporal_duration_part_from_argument, temporal_i128_to_number_value,
    temporal_optional_duration_part_from_property,
};
use options::{
    temporal_date_time_difference_unit_from_duration_exact,
    temporal_duration_exact_unit_nanoseconds, temporal_duration_relative_to_option,
    temporal_duration_round_options, temporal_duration_to_string_options,
    temporal_duration_total_options, TemporalDurationCalendarUnit,
    TemporalDurationParsedLargestUnit, TemporalDurationParsedUnit,
};
use round::{temporal_duration_round_builtin, temporal_duration_validate_exact_relative_to_range};
use total::temporal_duration_total_builtin;
use units::{
    temporal_duration_largest_unit_option, temporal_duration_parsed_unit,
    temporal_duration_rounding_increment_is_valid,
};

pub(super) use calendar_rounding::{
    temporal_duration_round_calendar_relative, temporal_duration_round_calendar_relative_exact,
    temporal_duration_validate_month_rounding_boundary,
    temporal_rounding_mode_for_negated_duration,
};
pub(super) use conversion::{
    allocate_temporal_duration_object, temporal_duration_from_additive_argument,
    temporal_duration_from_value, validate_temporal_duration,
};
pub(super) use options::{
    temporal_duration_rounding_mode_option, temporal_duration_rounding_mode_option_with_default,
    temporal_option_string_text, TemporalDurationRelativeTo,
};
pub(super) use units::temporal_duration_rounding_increment_option;

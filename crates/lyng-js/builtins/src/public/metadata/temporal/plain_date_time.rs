use crate::BuiltinEntryMetadata;
use lyng_js_types::BuiltinFunctionId;

#[allow(
    clippy::too_many_lines,
    reason = "PlainDateTime metadata mirrors the ordered Temporal builtin ID table"
)]
pub(super) fn plain_date_time_public_builtin_metadata(
    entry: BuiltinFunctionId,
) -> Option<BuiltinEntryMetadata> {
    if entry == lyng_js_types::temporal_plain_date_time_builtin() {
        return Some(BuiltinEntryMetadata::new("PlainDateTime", 3, true, true));
    }
    if entry == lyng_js_types::temporal_plain_date_time_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get year", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_month_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get month", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_month_code_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get monthCode", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_day_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get day", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_day_of_week_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get dayOfWeek", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_day_of_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get dayOfYear", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_days_in_month_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get daysInMonth",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_plain_date_time_days_in_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get daysInYear", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_months_in_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get monthsInYear",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_plain_date_time_in_leap_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get inLeapYear", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_days_in_week_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get daysInWeek", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_week_of_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get weekOfYear", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_year_of_week_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get yearOfWeek", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_era_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get era", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_era_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get eraYear", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_hour_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get hour", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_minute_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get minute", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_second_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get second", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_millisecond_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get millisecond",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_plain_date_time_microsecond_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get microsecond",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_plain_date_time_nanosecond_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get nanosecond", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_calendar_id_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get calendarId", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_to_json_builtin() {
        return Some(BuiltinEntryMetadata::new("toJSON", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_equals_builtin() {
        return Some(BuiltinEntryMetadata::new("equals", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_with_builtin() {
        return Some(BuiltinEntryMetadata::new("with", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_with_plain_time_builtin() {
        return Some(BuiltinEntryMetadata::new("withPlainTime", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_with_calendar_builtin() {
        return Some(BuiltinEntryMetadata::new("withCalendar", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_add_builtin() {
        return Some(BuiltinEntryMetadata::new("add", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_subtract_builtin() {
        return Some(BuiltinEntryMetadata::new("subtract", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_round_builtin() {
        return Some(BuiltinEntryMetadata::new("round", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_since_builtin() {
        return Some(BuiltinEntryMetadata::new("since", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_until_builtin() {
        return Some(BuiltinEntryMetadata::new("until", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_to_plain_date_builtin() {
        return Some(BuiltinEntryMetadata::new("toPlainDate", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_to_plain_time_builtin() {
        return Some(BuiltinEntryMetadata::new("toPlainTime", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_to_zoned_date_time_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "toZonedDateTime",
            1,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_plain_date_time_from_builtin() {
        return Some(BuiltinEntryMetadata::new("from", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_time_compare_builtin() {
        return Some(BuiltinEntryMetadata::new("compare", 2, false, false));
    }
    None
}

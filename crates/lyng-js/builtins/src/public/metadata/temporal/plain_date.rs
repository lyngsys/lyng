use crate::BuiltinEntryMetadata;
use lyng_js_types::BuiltinFunctionId;

#[allow(
    clippy::too_many_lines,
    reason = "PlainDate metadata mirrors the ordered Temporal builtin ID table"
)]
pub(super) fn plain_date_public_builtin_metadata(
    entry: BuiltinFunctionId,
) -> Option<BuiltinEntryMetadata> {
    if entry == lyng_js_types::temporal_plain_date_builtin() {
        return Some(BuiltinEntryMetadata::new("PlainDate", 3, true, true));
    }
    if entry == lyng_js_types::temporal_plain_date_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get year", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_month_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get month", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_month_code_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get monthCode", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_day_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get day", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_day_of_week_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get dayOfWeek", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_day_of_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get dayOfYear", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_days_in_month_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get daysInMonth",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_plain_date_days_in_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get daysInYear", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_months_in_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get monthsInYear",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_plain_date_in_leap_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get inLeapYear", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_days_in_week_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get daysInWeek", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_week_of_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get weekOfYear", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_year_of_week_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get yearOfWeek", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_era_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get era", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_era_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get eraYear", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_calendar_id_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get calendarId", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_to_json_builtin() {
        return Some(BuiltinEntryMetadata::new("toJSON", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_equals_builtin() {
        return Some(BuiltinEntryMetadata::new("equals", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_with_builtin() {
        return Some(BuiltinEntryMetadata::new("with", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_with_calendar_builtin() {
        return Some(BuiltinEntryMetadata::new("withCalendar", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_add_builtin() {
        return Some(BuiltinEntryMetadata::new("add", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_subtract_builtin() {
        return Some(BuiltinEntryMetadata::new("subtract", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_since_builtin() {
        return Some(BuiltinEntryMetadata::new("since", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_until_builtin() {
        return Some(BuiltinEntryMetadata::new("until", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_to_plain_date_time_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "toPlainDateTime",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_plain_date_to_zoned_date_time_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "toZonedDateTime",
            1,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_plain_date_to_plain_year_month_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "toPlainYearMonth",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_plain_date_to_plain_month_day_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "toPlainMonthDay",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_plain_date_from_builtin() {
        return Some(BuiltinEntryMetadata::new("from", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_date_compare_builtin() {
        return Some(BuiltinEntryMetadata::new("compare", 2, false, false));
    }
    None
}

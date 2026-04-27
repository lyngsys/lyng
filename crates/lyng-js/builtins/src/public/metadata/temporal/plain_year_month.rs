use crate::BuiltinEntryMetadata;
use lyng_js_types::BuiltinFunctionId;

pub(super) fn plain_year_month_public_builtin_metadata(
    entry: BuiltinFunctionId,
) -> Option<BuiltinEntryMetadata> {
    if entry == lyng_js_types::temporal_plain_year_month_builtin() {
        return Some(BuiltinEntryMetadata::new("PlainYearMonth", 2, true, true));
    }
    if entry == lyng_js_types::temporal_plain_year_month_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get year", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_month_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get month", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_month_code_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get monthCode", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_days_in_month_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get daysInMonth",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_plain_year_month_days_in_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get daysInYear", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_months_in_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get monthsInYear",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_plain_year_month_in_leap_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get inLeapYear", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_era_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get era", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_era_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get eraYear", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_calendar_id_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get calendarId", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_to_json_builtin() {
        return Some(BuiltinEntryMetadata::new("toJSON", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_equals_builtin() {
        return Some(BuiltinEntryMetadata::new("equals", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_with_builtin() {
        return Some(BuiltinEntryMetadata::new("with", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_add_builtin() {
        return Some(BuiltinEntryMetadata::new("add", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_subtract_builtin() {
        return Some(BuiltinEntryMetadata::new("subtract", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_since_builtin() {
        return Some(BuiltinEntryMetadata::new("since", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_until_builtin() {
        return Some(BuiltinEntryMetadata::new("until", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_to_plain_date_builtin() {
        return Some(BuiltinEntryMetadata::new("toPlainDate", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_from_builtin() {
        return Some(BuiltinEntryMetadata::new("from", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_year_month_compare_builtin() {
        return Some(BuiltinEntryMetadata::new("compare", 2, false, false));
    }
    None
}

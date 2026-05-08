use super::super::PublicBuiltinMetadataRow;
use crate::BuiltinEntryMetadata;

pub(in crate::public::metadata) const PUBLIC_TEMPORAL_PLAIN_DATE_BUILTIN_METADATA:
    &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_builtin(),
        BuiltinEntryMetadata::new("PlainDate", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_year_getter_builtin(),
        BuiltinEntryMetadata::new("get year", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_month_getter_builtin(),
        BuiltinEntryMetadata::new("get month", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_month_code_getter_builtin(),
        BuiltinEntryMetadata::new("get monthCode", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_day_getter_builtin(),
        BuiltinEntryMetadata::new("get day", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_day_of_week_getter_builtin(),
        BuiltinEntryMetadata::new("get dayOfWeek", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_day_of_year_getter_builtin(),
        BuiltinEntryMetadata::new("get dayOfYear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_days_in_month_getter_builtin(),
        BuiltinEntryMetadata::new("get daysInMonth", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_days_in_year_getter_builtin(),
        BuiltinEntryMetadata::new("get daysInYear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_months_in_year_getter_builtin(),
        BuiltinEntryMetadata::new("get monthsInYear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_in_leap_year_getter_builtin(),
        BuiltinEntryMetadata::new("get inLeapYear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_days_in_week_getter_builtin(),
        BuiltinEntryMetadata::new("get daysInWeek", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_week_of_year_getter_builtin(),
        BuiltinEntryMetadata::new("get weekOfYear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_year_of_week_getter_builtin(),
        BuiltinEntryMetadata::new("get yearOfWeek", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_era_getter_builtin(),
        BuiltinEntryMetadata::new("get era", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_era_year_getter_builtin(),
        BuiltinEntryMetadata::new("get eraYear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_calendar_id_getter_builtin(),
        BuiltinEntryMetadata::new("get calendarId", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_to_string_builtin(),
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_to_json_builtin(),
        BuiltinEntryMetadata::new("toJSON", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_to_locale_string_builtin(),
        BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_value_of_builtin(),
        BuiltinEntryMetadata::new("valueOf", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_equals_builtin(),
        BuiltinEntryMetadata::new("equals", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_with_builtin(),
        BuiltinEntryMetadata::new("with", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_with_calendar_builtin(),
        BuiltinEntryMetadata::new("withCalendar", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_add_builtin(),
        BuiltinEntryMetadata::new("add", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_subtract_builtin(),
        BuiltinEntryMetadata::new("subtract", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_since_builtin(),
        BuiltinEntryMetadata::new("since", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_until_builtin(),
        BuiltinEntryMetadata::new("until", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_to_plain_date_time_builtin(),
        BuiltinEntryMetadata::new("toPlainDateTime", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_to_zoned_date_time_builtin(),
        BuiltinEntryMetadata::new("toZonedDateTime", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_to_plain_year_month_builtin(),
        BuiltinEntryMetadata::new("toPlainYearMonth", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_to_plain_month_day_builtin(),
        BuiltinEntryMetadata::new("toPlainMonthDay", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_from_builtin(),
        BuiltinEntryMetadata::new("from", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_compare_builtin(),
        BuiltinEntryMetadata::new("compare", 2, false, false),
    ),
];

use super::super::PublicBuiltinMetadataRow;
use crate::BuiltinEntryMetadata;

pub(in crate::public::metadata) const PUBLIC_TEMPORAL_PLAIN_DATE_TIME_BUILTIN_METADATA:
    &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_builtin(),
        BuiltinEntryMetadata::new("PlainDateTime", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_year_getter_builtin(),
        BuiltinEntryMetadata::new("get year", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_month_getter_builtin(),
        BuiltinEntryMetadata::new("get month", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_month_code_getter_builtin(),
        BuiltinEntryMetadata::new("get monthCode", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_day_getter_builtin(),
        BuiltinEntryMetadata::new("get day", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_day_of_week_getter_builtin(),
        BuiltinEntryMetadata::new("get dayOfWeek", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_day_of_year_getter_builtin(),
        BuiltinEntryMetadata::new("get dayOfYear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_days_in_month_getter_builtin(),
        BuiltinEntryMetadata::new("get daysInMonth", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_days_in_year_getter_builtin(),
        BuiltinEntryMetadata::new("get daysInYear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_months_in_year_getter_builtin(),
        BuiltinEntryMetadata::new("get monthsInYear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_in_leap_year_getter_builtin(),
        BuiltinEntryMetadata::new("get inLeapYear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_days_in_week_getter_builtin(),
        BuiltinEntryMetadata::new("get daysInWeek", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_week_of_year_getter_builtin(),
        BuiltinEntryMetadata::new("get weekOfYear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_year_of_week_getter_builtin(),
        BuiltinEntryMetadata::new("get yearOfWeek", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_era_getter_builtin(),
        BuiltinEntryMetadata::new("get era", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_era_year_getter_builtin(),
        BuiltinEntryMetadata::new("get eraYear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_hour_getter_builtin(),
        BuiltinEntryMetadata::new("get hour", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_minute_getter_builtin(),
        BuiltinEntryMetadata::new("get minute", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_second_getter_builtin(),
        BuiltinEntryMetadata::new("get second", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_millisecond_getter_builtin(),
        BuiltinEntryMetadata::new("get millisecond", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_microsecond_getter_builtin(),
        BuiltinEntryMetadata::new("get microsecond", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_nanosecond_getter_builtin(),
        BuiltinEntryMetadata::new("get nanosecond", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_calendar_id_getter_builtin(),
        BuiltinEntryMetadata::new("get calendarId", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_to_string_builtin(),
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_to_json_builtin(),
        BuiltinEntryMetadata::new("toJSON", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_to_locale_string_builtin(),
        BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_value_of_builtin(),
        BuiltinEntryMetadata::new("valueOf", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_equals_builtin(),
        BuiltinEntryMetadata::new("equals", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_with_builtin(),
        BuiltinEntryMetadata::new("with", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_with_plain_time_builtin(),
        BuiltinEntryMetadata::new("withPlainTime", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_with_calendar_builtin(),
        BuiltinEntryMetadata::new("withCalendar", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_add_builtin(),
        BuiltinEntryMetadata::new("add", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_subtract_builtin(),
        BuiltinEntryMetadata::new("subtract", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_round_builtin(),
        BuiltinEntryMetadata::new("round", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_since_builtin(),
        BuiltinEntryMetadata::new("since", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_until_builtin(),
        BuiltinEntryMetadata::new("until", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_to_plain_date_builtin(),
        BuiltinEntryMetadata::new("toPlainDate", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_to_plain_time_builtin(),
        BuiltinEntryMetadata::new("toPlainTime", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_to_zoned_date_time_builtin(),
        BuiltinEntryMetadata::new("toZonedDateTime", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_from_builtin(),
        BuiltinEntryMetadata::new("from", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_date_time_compare_builtin(),
        BuiltinEntryMetadata::new("compare", 2, false, false),
    ),
];

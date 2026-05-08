use super::super::PublicBuiltinMetadataRow;
use crate::BuiltinEntryMetadata;

pub(in crate::public::metadata) const PUBLIC_TEMPORAL_PLAIN_YEAR_MONTH_BUILTIN_METADATA:
    &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_builtin(),
        BuiltinEntryMetadata::new("PlainYearMonth", 2, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_year_getter_builtin(),
        BuiltinEntryMetadata::new("get year", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_month_getter_builtin(),
        BuiltinEntryMetadata::new("get month", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_month_code_getter_builtin(),
        BuiltinEntryMetadata::new("get monthCode", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_days_in_month_getter_builtin(),
        BuiltinEntryMetadata::new("get daysInMonth", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_days_in_year_getter_builtin(),
        BuiltinEntryMetadata::new("get daysInYear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_months_in_year_getter_builtin(),
        BuiltinEntryMetadata::new("get monthsInYear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_in_leap_year_getter_builtin(),
        BuiltinEntryMetadata::new("get inLeapYear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_era_getter_builtin(),
        BuiltinEntryMetadata::new("get era", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_era_year_getter_builtin(),
        BuiltinEntryMetadata::new("get eraYear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_calendar_id_getter_builtin(),
        BuiltinEntryMetadata::new("get calendarId", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_to_string_builtin(),
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_to_json_builtin(),
        BuiltinEntryMetadata::new("toJSON", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_to_locale_string_builtin(),
        BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_value_of_builtin(),
        BuiltinEntryMetadata::new("valueOf", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_equals_builtin(),
        BuiltinEntryMetadata::new("equals", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_with_builtin(),
        BuiltinEntryMetadata::new("with", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_add_builtin(),
        BuiltinEntryMetadata::new("add", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_subtract_builtin(),
        BuiltinEntryMetadata::new("subtract", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_since_builtin(),
        BuiltinEntryMetadata::new("since", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_until_builtin(),
        BuiltinEntryMetadata::new("until", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_to_plain_date_builtin(),
        BuiltinEntryMetadata::new("toPlainDate", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_from_builtin(),
        BuiltinEntryMetadata::new("from", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_year_month_compare_builtin(),
        BuiltinEntryMetadata::new("compare", 2, false, false),
    ),
];

use super::super::PublicBuiltinMetadataRow;
use crate::BuiltinEntryMetadata;

pub(in crate::public::metadata) const PUBLIC_TEMPORAL_PLAIN_MONTH_DAY_BUILTIN_METADATA:
    &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_month_day_builtin(),
        BuiltinEntryMetadata::new("PlainMonthDay", 2, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_month_day_month_code_getter_builtin(),
        BuiltinEntryMetadata::new("get monthCode", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_month_day_day_getter_builtin(),
        BuiltinEntryMetadata::new("get day", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_month_day_calendar_id_getter_builtin(),
        BuiltinEntryMetadata::new("get calendarId", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_month_day_to_string_builtin(),
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_month_day_to_json_builtin(),
        BuiltinEntryMetadata::new("toJSON", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_month_day_to_locale_string_builtin(),
        BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_month_day_value_of_builtin(),
        BuiltinEntryMetadata::new("valueOf", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_month_day_equals_builtin(),
        BuiltinEntryMetadata::new("equals", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_month_day_with_builtin(),
        BuiltinEntryMetadata::new("with", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_month_day_to_plain_date_builtin(),
        BuiltinEntryMetadata::new("toPlainDate", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_month_day_from_builtin(),
        BuiltinEntryMetadata::new("from", 1, false, false),
    ),
];

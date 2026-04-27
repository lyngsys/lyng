use super::super::*;

pub(in crate::public::metadata) const PUBLIC_DATE_BUILTIN_METADATA: &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        date_builtin,
        BuiltinEntryMetadata::new("Date", 7, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        date_now_builtin,
        BuiltinEntryMetadata::new("now", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_parse_builtin,
        BuiltinEntryMetadata::new("parse", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_utc_builtin,
        BuiltinEntryMetadata::new("UTC", 7, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_to_string_builtin,
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_to_date_string_builtin,
        BuiltinEntryMetadata::new("toDateString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_to_time_string_builtin,
        BuiltinEntryMetadata::new("toTimeString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_to_locale_string_builtin,
        BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_to_locale_date_string_builtin,
        BuiltinEntryMetadata::new("toLocaleDateString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_to_locale_time_string_builtin,
        BuiltinEntryMetadata::new("toLocaleTimeString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_value_of_builtin,
        BuiltinEntryMetadata::new("valueOf", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_get_time_builtin,
        BuiltinEntryMetadata::new("getTime", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_get_full_year_builtin,
        BuiltinEntryMetadata::new("getFullYear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_get_utc_full_year_builtin,
        BuiltinEntryMetadata::new("getUTCFullYear", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_get_month_builtin,
        BuiltinEntryMetadata::new("getMonth", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_get_utc_month_builtin,
        BuiltinEntryMetadata::new("getUTCMonth", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_get_date_builtin,
        BuiltinEntryMetadata::new("getDate", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_get_utc_date_builtin,
        BuiltinEntryMetadata::new("getUTCDate", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_get_day_builtin,
        BuiltinEntryMetadata::new("getDay", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_get_utc_day_builtin,
        BuiltinEntryMetadata::new("getUTCDay", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_get_hours_builtin,
        BuiltinEntryMetadata::new("getHours", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_get_utc_hours_builtin,
        BuiltinEntryMetadata::new("getUTCHours", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_get_minutes_builtin,
        BuiltinEntryMetadata::new("getMinutes", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_get_utc_minutes_builtin,
        BuiltinEntryMetadata::new("getUTCMinutes", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_get_seconds_builtin,
        BuiltinEntryMetadata::new("getSeconds", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_get_utc_seconds_builtin,
        BuiltinEntryMetadata::new("getUTCSeconds", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_get_milliseconds_builtin,
        BuiltinEntryMetadata::new("getMilliseconds", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_get_utc_milliseconds_builtin,
        BuiltinEntryMetadata::new("getUTCMilliseconds", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_get_timezone_offset_builtin,
        BuiltinEntryMetadata::new("getTimezoneOffset", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_set_time_builtin,
        BuiltinEntryMetadata::new("setTime", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_set_milliseconds_builtin,
        BuiltinEntryMetadata::new("setMilliseconds", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_set_utc_milliseconds_builtin,
        BuiltinEntryMetadata::new("setUTCMilliseconds", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_set_seconds_builtin,
        BuiltinEntryMetadata::new("setSeconds", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_set_utc_seconds_builtin,
        BuiltinEntryMetadata::new("setUTCSeconds", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_set_minutes_builtin,
        BuiltinEntryMetadata::new("setMinutes", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_set_utc_minutes_builtin,
        BuiltinEntryMetadata::new("setUTCMinutes", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_set_hours_builtin,
        BuiltinEntryMetadata::new("setHours", 4, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_set_utc_hours_builtin,
        BuiltinEntryMetadata::new("setUTCHours", 4, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_set_date_builtin,
        BuiltinEntryMetadata::new("setDate", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_set_utc_date_builtin,
        BuiltinEntryMetadata::new("setUTCDate", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_set_month_builtin,
        BuiltinEntryMetadata::new("setMonth", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_set_utc_month_builtin,
        BuiltinEntryMetadata::new("setUTCMonth", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_set_full_year_builtin,
        BuiltinEntryMetadata::new("setFullYear", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_set_utc_full_year_builtin,
        BuiltinEntryMetadata::new("setUTCFullYear", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_to_utc_string_builtin,
        BuiltinEntryMetadata::new("toUTCString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_to_iso_string_builtin,
        BuiltinEntryMetadata::new("toISOString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_to_json_builtin,
        BuiltinEntryMetadata::new("toJSON", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_to_temporal_instant_builtin,
        BuiltinEntryMetadata::new("toTemporalInstant", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        date_to_primitive_builtin,
        BuiltinEntryMetadata::new("[Symbol.toPrimitive]", 1, false, false),
    ),
];

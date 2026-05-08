use super::super::PublicBuiltinMetadataRow;
use crate::BuiltinEntryMetadata;

pub(in crate::public::metadata) const PUBLIC_TEMPORAL_PLAIN_TIME_BUILTIN_METADATA:
    &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_builtin(),
        BuiltinEntryMetadata::new("PlainTime", 0, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_hour_getter_builtin(),
        BuiltinEntryMetadata::new("get hour", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_minute_getter_builtin(),
        BuiltinEntryMetadata::new("get minute", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_second_getter_builtin(),
        BuiltinEntryMetadata::new("get second", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_millisecond_getter_builtin(),
        BuiltinEntryMetadata::new("get millisecond", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_microsecond_getter_builtin(),
        BuiltinEntryMetadata::new("get microsecond", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_nanosecond_getter_builtin(),
        BuiltinEntryMetadata::new("get nanosecond", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_to_string_builtin(),
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_to_json_builtin(),
        BuiltinEntryMetadata::new("toJSON", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_to_locale_string_builtin(),
        BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_value_of_builtin(),
        BuiltinEntryMetadata::new("valueOf", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_equals_builtin(),
        BuiltinEntryMetadata::new("equals", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_with_builtin(),
        BuiltinEntryMetadata::new("with", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_add_builtin(),
        BuiltinEntryMetadata::new("add", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_subtract_builtin(),
        BuiltinEntryMetadata::new("subtract", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_round_builtin(),
        BuiltinEntryMetadata::new("round", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_since_builtin(),
        BuiltinEntryMetadata::new("since", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_until_builtin(),
        BuiltinEntryMetadata::new("until", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_from_builtin(),
        BuiltinEntryMetadata::new("from", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_plain_time_compare_builtin(),
        BuiltinEntryMetadata::new("compare", 2, false, false),
    ),
];

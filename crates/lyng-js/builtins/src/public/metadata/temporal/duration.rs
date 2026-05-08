use super::super::PublicBuiltinMetadataRow;
use crate::BuiltinEntryMetadata;

pub(in crate::public::metadata) const PUBLIC_TEMPORAL_DURATION_BUILTIN_METADATA:
    &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_builtin(),
        BuiltinEntryMetadata::new("Duration", 0, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_years_getter_builtin(),
        BuiltinEntryMetadata::new("get years", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_months_getter_builtin(),
        BuiltinEntryMetadata::new("get months", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_weeks_getter_builtin(),
        BuiltinEntryMetadata::new("get weeks", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_days_getter_builtin(),
        BuiltinEntryMetadata::new("get days", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_hours_getter_builtin(),
        BuiltinEntryMetadata::new("get hours", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_minutes_getter_builtin(),
        BuiltinEntryMetadata::new("get minutes", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_seconds_getter_builtin(),
        BuiltinEntryMetadata::new("get seconds", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_milliseconds_getter_builtin(),
        BuiltinEntryMetadata::new("get milliseconds", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_microseconds_getter_builtin(),
        BuiltinEntryMetadata::new("get microseconds", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_nanoseconds_getter_builtin(),
        BuiltinEntryMetadata::new("get nanoseconds", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_sign_getter_builtin(),
        BuiltinEntryMetadata::new("get sign", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_blank_getter_builtin(),
        BuiltinEntryMetadata::new("get blank", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_to_string_builtin(),
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_to_json_builtin(),
        BuiltinEntryMetadata::new("toJSON", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_to_locale_string_builtin(),
        BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_negated_builtin(),
        BuiltinEntryMetadata::new("negated", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_abs_builtin(),
        BuiltinEntryMetadata::new("abs", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_with_builtin(),
        BuiltinEntryMetadata::new("with", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_round_builtin(),
        BuiltinEntryMetadata::new("round", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_total_builtin(),
        BuiltinEntryMetadata::new("total", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_add_builtin(),
        BuiltinEntryMetadata::new("add", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_subtract_builtin(),
        BuiltinEntryMetadata::new("subtract", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_value_of_builtin(),
        BuiltinEntryMetadata::new("valueOf", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_from_builtin(),
        BuiltinEntryMetadata::new("from", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_duration_compare_builtin(),
        BuiltinEntryMetadata::new("compare", 2, false, false),
    ),
];

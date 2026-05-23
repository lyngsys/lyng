use super::super::PublicBuiltinMetadataRow;
use crate::BuiltinEntryMetadata;

pub(in crate::public::metadata) const PUBLIC_TEMPORAL_INSTANT_BUILTIN_METADATA:
    &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_instant_builtin(),
        BuiltinEntryMetadata::new("Instant", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_instant_from_builtin(),
        BuiltinEntryMetadata::new("from", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_instant_from_epoch_nanoseconds_builtin(),
        BuiltinEntryMetadata::new("fromEpochNanoseconds", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_instant_from_epoch_milliseconds_builtin(),
        BuiltinEntryMetadata::new("fromEpochMilliseconds", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_instant_compare_builtin(),
        BuiltinEntryMetadata::new("compare", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_instant_epoch_nanoseconds_getter_builtin(),
        BuiltinEntryMetadata::new("get epochNanoseconds", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_instant_epoch_milliseconds_getter_builtin(),
        BuiltinEntryMetadata::new("get epochMilliseconds", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_instant_to_string_builtin(),
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_instant_to_json_builtin(),
        BuiltinEntryMetadata::new("toJSON", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_instant_to_locale_string_builtin(),
        BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_instant_value_of_builtin(),
        BuiltinEntryMetadata::new("valueOf", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_instant_equals_builtin(),
        BuiltinEntryMetadata::new("equals", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_instant_add_builtin(),
        BuiltinEntryMetadata::new("add", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_instant_subtract_builtin(),
        BuiltinEntryMetadata::new("subtract", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_instant_round_builtin(),
        BuiltinEntryMetadata::new("round", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_instant_since_builtin(),
        BuiltinEntryMetadata::new("since", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_instant_until_builtin(),
        BuiltinEntryMetadata::new("until", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_instant_to_zoned_date_time_iso_builtin(),
        BuiltinEntryMetadata::new("toZonedDateTimeISO", 1, false, false),
    ),
];

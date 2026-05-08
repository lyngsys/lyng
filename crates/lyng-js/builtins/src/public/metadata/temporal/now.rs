use super::super::PublicBuiltinMetadataRow;
use crate::BuiltinEntryMetadata;

pub(in crate::public::metadata) const PUBLIC_TEMPORAL_NOW_BUILTIN_METADATA:
    &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_now_instant_builtin(),
        BuiltinEntryMetadata::new("instant", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_now_time_zone_id_builtin(),
        BuiltinEntryMetadata::new("timeZoneId", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_now_plain_date_iso_builtin(),
        BuiltinEntryMetadata::new("plainDateISO", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_now_plain_time_iso_builtin(),
        BuiltinEntryMetadata::new("plainTimeISO", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_now_plain_date_time_iso_builtin(),
        BuiltinEntryMetadata::new("plainDateTimeISO", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        lyng_js_types::temporal_now_zoned_date_time_iso_builtin(),
        BuiltinEntryMetadata::new("zonedDateTimeISO", 0, false, false),
    ),
];

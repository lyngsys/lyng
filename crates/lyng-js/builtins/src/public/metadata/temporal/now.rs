use crate::BuiltinEntryMetadata;
use lyng_js_types::BuiltinFunctionId;

pub(super) fn now_public_builtin_metadata(
    entry: BuiltinFunctionId,
) -> Option<BuiltinEntryMetadata> {
    if entry == lyng_js_types::temporal_now_instant_builtin() {
        return Some(BuiltinEntryMetadata::new("instant", 0, false, false));
    }
    if entry == lyng_js_types::temporal_now_time_zone_id_builtin() {
        return Some(BuiltinEntryMetadata::new("timeZoneId", 0, false, false));
    }
    if entry == lyng_js_types::temporal_now_plain_date_iso_builtin() {
        return Some(BuiltinEntryMetadata::new("plainDateISO", 0, false, false));
    }
    if entry == lyng_js_types::temporal_now_plain_time_iso_builtin() {
        return Some(BuiltinEntryMetadata::new("plainTimeISO", 0, false, false));
    }
    if entry == lyng_js_types::temporal_now_plain_date_time_iso_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "plainDateTimeISO",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_now_zoned_date_time_iso_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "zonedDateTimeISO",
            0,
            false,
            false,
        ));
    }
    None
}

use crate::BuiltinEntryMetadata;
use lyng_js_types::BuiltinFunctionId;

pub(super) fn instant_public_builtin_metadata(
    entry: BuiltinFunctionId,
) -> Option<BuiltinEntryMetadata> {
    if entry == lyng_js_types::temporal_instant_builtin() {
        return Some(BuiltinEntryMetadata::new("Instant", 1, true, true));
    }
    if entry == lyng_js_types::temporal_instant_from_builtin() {
        return Some(BuiltinEntryMetadata::new("from", 1, false, false));
    }
    if entry == lyng_js_types::temporal_instant_from_epoch_nanoseconds_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "fromEpochNanoseconds",
            1,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_instant_from_epoch_milliseconds_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "fromEpochMilliseconds",
            1,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_instant_compare_builtin() {
        return Some(BuiltinEntryMetadata::new("compare", 2, false, false));
    }
    if entry == lyng_js_types::temporal_instant_epoch_nanoseconds_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get epochNanoseconds",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_instant_epoch_milliseconds_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get epochMilliseconds",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_instant_epoch_seconds_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get epochSeconds",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_instant_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == lyng_js_types::temporal_instant_to_json_builtin() {
        return Some(BuiltinEntryMetadata::new("toJSON", 0, false, false));
    }
    if entry == lyng_js_types::temporal_instant_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == lyng_js_types::temporal_instant_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == lyng_js_types::temporal_instant_equals_builtin() {
        return Some(BuiltinEntryMetadata::new("equals", 1, false, false));
    }
    if entry == lyng_js_types::temporal_instant_add_builtin() {
        return Some(BuiltinEntryMetadata::new("add", 1, false, false));
    }
    if entry == lyng_js_types::temporal_instant_subtract_builtin() {
        return Some(BuiltinEntryMetadata::new("subtract", 1, false, false));
    }
    if entry == lyng_js_types::temporal_instant_round_builtin() {
        return Some(BuiltinEntryMetadata::new("round", 1, false, false));
    }
    if entry == lyng_js_types::temporal_instant_since_builtin() {
        return Some(BuiltinEntryMetadata::new("since", 1, false, false));
    }
    if entry == lyng_js_types::temporal_instant_until_builtin() {
        return Some(BuiltinEntryMetadata::new("until", 1, false, false));
    }
    if entry == lyng_js_types::temporal_instant_to_zoned_date_time_iso_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "toZonedDateTimeISO",
            1,
            false,
            false,
        ));
    }
    None
}

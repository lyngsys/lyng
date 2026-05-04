use crate::BuiltinEntryMetadata;
use lyng_js_types::BuiltinFunctionId;

pub(super) fn plain_time_public_builtin_metadata(
    entry: BuiltinFunctionId,
) -> Option<BuiltinEntryMetadata> {
    if entry == lyng_js_types::temporal_plain_time_builtin() {
        return Some(BuiltinEntryMetadata::new("PlainTime", 0, true, true));
    }
    if entry == lyng_js_types::temporal_plain_time_hour_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get hour", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_time_minute_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get minute", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_time_second_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get second", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_time_millisecond_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get millisecond",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_plain_time_microsecond_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get microsecond",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_plain_time_nanosecond_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get nanosecond", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_time_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_time_to_json_builtin() {
        return Some(BuiltinEntryMetadata::new("toJSON", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_time_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_time_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_time_equals_builtin() {
        return Some(BuiltinEntryMetadata::new("equals", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_time_with_builtin() {
        return Some(BuiltinEntryMetadata::new("with", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_time_add_builtin() {
        return Some(BuiltinEntryMetadata::new("add", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_time_subtract_builtin() {
        return Some(BuiltinEntryMetadata::new("subtract", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_time_round_builtin() {
        return Some(BuiltinEntryMetadata::new("round", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_time_since_builtin() {
        return Some(BuiltinEntryMetadata::new("since", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_time_until_builtin() {
        return Some(BuiltinEntryMetadata::new("until", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_time_from_builtin() {
        return Some(BuiltinEntryMetadata::new("from", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_time_compare_builtin() {
        return Some(BuiltinEntryMetadata::new("compare", 2, false, false));
    }
    None
}

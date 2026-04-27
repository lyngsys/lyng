use crate::BuiltinEntryMetadata;
use lyng_js_types::BuiltinFunctionId;

pub(super) fn duration_public_builtin_metadata(
    entry: BuiltinFunctionId,
) -> Option<BuiltinEntryMetadata> {
    if entry == lyng_js_types::temporal_duration_builtin() {
        return Some(BuiltinEntryMetadata::new("Duration", 0, true, true));
    }
    if entry == lyng_js_types::temporal_duration_years_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get years", 0, false, false));
    }
    if entry == lyng_js_types::temporal_duration_months_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get months", 0, false, false));
    }
    if entry == lyng_js_types::temporal_duration_weeks_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get weeks", 0, false, false));
    }
    if entry == lyng_js_types::temporal_duration_days_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get days", 0, false, false));
    }
    if entry == lyng_js_types::temporal_duration_hours_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get hours", 0, false, false));
    }
    if entry == lyng_js_types::temporal_duration_minutes_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get minutes", 0, false, false));
    }
    if entry == lyng_js_types::temporal_duration_seconds_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get seconds", 0, false, false));
    }
    if entry == lyng_js_types::temporal_duration_milliseconds_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get milliseconds",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_duration_microseconds_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get microseconds",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_duration_nanoseconds_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get nanoseconds",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::temporal_duration_sign_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get sign", 0, false, false));
    }
    if entry == lyng_js_types::temporal_duration_blank_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get blank", 0, false, false));
    }
    if entry == lyng_js_types::temporal_duration_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == lyng_js_types::temporal_duration_to_json_builtin() {
        return Some(BuiltinEntryMetadata::new("toJSON", 0, false, false));
    }
    if entry == lyng_js_types::temporal_duration_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == lyng_js_types::temporal_duration_negated_builtin() {
        return Some(BuiltinEntryMetadata::new("negated", 0, false, false));
    }
    if entry == lyng_js_types::temporal_duration_abs_builtin() {
        return Some(BuiltinEntryMetadata::new("abs", 0, false, false));
    }
    if entry == lyng_js_types::temporal_duration_with_builtin() {
        return Some(BuiltinEntryMetadata::new("with", 1, false, false));
    }
    if entry == lyng_js_types::temporal_duration_round_builtin() {
        return Some(BuiltinEntryMetadata::new("round", 1, false, false));
    }
    if entry == lyng_js_types::temporal_duration_total_builtin() {
        return Some(BuiltinEntryMetadata::new("total", 1, false, false));
    }
    if entry == lyng_js_types::temporal_duration_add_builtin() {
        return Some(BuiltinEntryMetadata::new("add", 1, false, false));
    }
    if entry == lyng_js_types::temporal_duration_subtract_builtin() {
        return Some(BuiltinEntryMetadata::new("subtract", 1, false, false));
    }
    if entry == lyng_js_types::temporal_duration_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == lyng_js_types::temporal_duration_from_builtin() {
        return Some(BuiltinEntryMetadata::new("from", 1, false, false));
    }
    if entry == lyng_js_types::temporal_duration_compare_builtin() {
        return Some(BuiltinEntryMetadata::new("compare", 2, false, false));
    }
    None
}

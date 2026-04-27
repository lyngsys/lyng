use crate::BuiltinEntryMetadata;
use lyng_js_types::BuiltinFunctionId;

pub(super) fn plain_month_day_public_builtin_metadata(
    entry: BuiltinFunctionId,
) -> Option<BuiltinEntryMetadata> {
    if entry == lyng_js_types::temporal_plain_month_day_builtin() {
        return Some(BuiltinEntryMetadata::new("PlainMonthDay", 2, true, true));
    }
    if entry == lyng_js_types::temporal_plain_month_day_month_code_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get monthCode", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_month_day_day_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get day", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_month_day_calendar_id_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get calendarId", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_month_day_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_month_day_to_json_builtin() {
        return Some(BuiltinEntryMetadata::new("toJSON", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_month_day_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_month_day_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == lyng_js_types::temporal_plain_month_day_equals_builtin() {
        return Some(BuiltinEntryMetadata::new("equals", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_month_day_with_builtin() {
        return Some(BuiltinEntryMetadata::new("with", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_month_day_to_plain_date_builtin() {
        return Some(BuiltinEntryMetadata::new("toPlainDate", 1, false, false));
    }
    if entry == lyng_js_types::temporal_plain_month_day_from_builtin() {
        return Some(BuiltinEntryMetadata::new("from", 1, false, false));
    }
    None
}

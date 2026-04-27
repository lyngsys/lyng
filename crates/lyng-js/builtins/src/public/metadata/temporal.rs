mod duration;
mod instant;
mod now;
mod plain_date;
mod plain_date_time;
mod plain_month_day;
mod plain_time;
mod plain_year_month;
mod zoned_date_time;

use crate::BuiltinEntryMetadata;
use lyng_js_types::BuiltinFunctionId;

pub(super) fn temporal_public_builtin_metadata(
    entry: BuiltinFunctionId,
) -> Option<BuiltinEntryMetadata> {
    if let Some(metadata) = instant::instant_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if let Some(metadata) = now::now_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if let Some(metadata) = duration::duration_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if let Some(metadata) = plain_date::plain_date_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if let Some(metadata) = plain_time::plain_time_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if let Some(metadata) = plain_date_time::plain_date_time_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if let Some(metadata) = plain_year_month::plain_year_month_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if let Some(metadata) = plain_month_day::plain_month_day_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    if let Some(metadata) = zoned_date_time::zoned_date_time_public_builtin_metadata(entry) {
        return Some(metadata);
    }
    None
}

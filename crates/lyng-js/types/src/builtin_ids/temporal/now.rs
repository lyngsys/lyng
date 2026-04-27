use super::super::*;

#[inline]
pub fn temporal_now_instant_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_NOW_INSTANT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_now_time_zone_id_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_NOW_TIME_ZONE_ID_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_now_plain_date_iso_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_NOW_PLAIN_DATE_ISO_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_now_plain_time_iso_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_NOW_PLAIN_TIME_ISO_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_now_plain_date_time_iso_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_NOW_PLAIN_DATE_TIME_ISO_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_now_zoned_date_time_iso_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_NOW_ZONED_DATE_TIME_ISO_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn temporal_now_instant_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_NOW_INSTANT_RAW)
}

#[inline]
pub const fn temporal_now_time_zone_id_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_NOW_TIME_ZONE_ID_RAW)
}

#[inline]
pub const fn temporal_now_plain_date_iso_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_NOW_PLAIN_DATE_ISO_RAW)
}

#[inline]
pub const fn temporal_now_plain_time_iso_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_NOW_PLAIN_TIME_ISO_RAW)
}

#[inline]
pub const fn temporal_now_plain_date_time_iso_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_NOW_PLAIN_DATE_TIME_ISO_RAW)
}

#[inline]
pub const fn temporal_now_zoned_date_time_iso_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_NOW_ZONED_DATE_TIME_ISO_RAW)
}

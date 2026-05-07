use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn temporal_instant_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_RAW)
}

#[inline]
pub const fn temporal_instant_from_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_FROM_RAW)
}

#[inline]
pub const fn temporal_instant_from_epoch_nanoseconds_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_FROM_EPOCH_NANOSECONDS_RAW)
}

#[inline]
pub const fn temporal_instant_from_epoch_milliseconds_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_FROM_EPOCH_MILLISECONDS_RAW)
}

#[inline]
pub const fn temporal_instant_compare_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_COMPARE_RAW)
}

#[inline]
pub const fn temporal_instant_epoch_nanoseconds_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_EPOCH_NANOSECONDS_GETTER_RAW)
}

#[inline]
pub const fn temporal_instant_epoch_milliseconds_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_EPOCH_MILLISECONDS_GETTER_RAW)
}

#[inline]
pub const fn temporal_instant_epoch_seconds_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_EPOCH_SECONDS_GETTER_RAW)
}

#[inline]
pub const fn temporal_instant_to_string_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_TO_STRING_RAW)
}

#[inline]
pub const fn temporal_instant_to_json_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_TO_JSON_RAW)
}

#[inline]
pub const fn temporal_instant_to_locale_string_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_TO_LOCALE_STRING_RAW)
}

#[inline]
pub const fn temporal_instant_value_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_VALUE_OF_RAW)
}

#[inline]
pub const fn temporal_instant_equals_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_EQUALS_RAW)
}

#[inline]
pub const fn temporal_instant_add_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_ADD_RAW)
}

#[inline]
pub const fn temporal_instant_subtract_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_SUBTRACT_RAW)
}

#[inline]
pub const fn temporal_instant_round_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_ROUND_RAW)
}

#[inline]
pub const fn temporal_instant_since_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_SINCE_RAW)
}

#[inline]
pub const fn temporal_instant_until_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_UNTIL_RAW)
}

#[inline]
pub const fn temporal_instant_to_zoned_date_time_iso_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::TEMPORAL_INSTANT_TO_ZONED_DATE_TIME_ISO_RAW)
}

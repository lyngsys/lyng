use super::super::*;

#[inline]
pub fn temporal_instant_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_instant_from_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_FROM_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_instant_from_epoch_nanoseconds_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_FROM_EPOCH_NANOSECONDS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_instant_from_epoch_milliseconds_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_FROM_EPOCH_MILLISECONDS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_instant_compare_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_COMPARE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_instant_epoch_nanoseconds_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_EPOCH_NANOSECONDS_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_instant_epoch_milliseconds_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_EPOCH_MILLISECONDS_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_instant_epoch_seconds_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_EPOCH_SECONDS_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_instant_to_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_TO_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_instant_to_json_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_TO_JSON_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_instant_to_locale_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_TO_LOCALE_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_instant_value_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_VALUE_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_instant_equals_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_EQUALS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_instant_add_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_ADD_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_instant_subtract_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_SUBTRACT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_instant_round_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_ROUND_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_instant_since_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_SINCE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_instant_until_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_UNTIL_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_instant_to_zoned_date_time_iso_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_INSTANT_TO_ZONED_DATE_TIME_ISO_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

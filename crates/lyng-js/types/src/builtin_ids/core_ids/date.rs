use super::super::*;

#[inline]
pub fn date_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_now_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_NOW_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_parse_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_PARSE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_to_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_TO_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_value_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_VALUE_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_timezone_offset_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_TIMEZONE_OFFSET_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_utc_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_UTC_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_to_date_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_TO_DATE_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_to_time_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_TO_TIME_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_to_locale_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_TO_LOCALE_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_to_locale_date_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_TO_LOCALE_DATE_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_to_locale_time_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_TO_LOCALE_TIME_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_time_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_TIME_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_full_year_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_FULL_YEAR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_year_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_YEAR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_utc_full_year_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_UTC_FULL_YEAR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_month_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_MONTH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_utc_month_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_UTC_MONTH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_date_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_DATE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_utc_date_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_UTC_DATE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_day_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_DAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_utc_day_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_UTC_DAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_hours_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_HOURS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_utc_hours_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_UTC_HOURS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_minutes_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_MINUTES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_utc_minutes_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_UTC_MINUTES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_seconds_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_SECONDS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_utc_seconds_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_UTC_SECONDS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_milliseconds_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_MILLISECONDS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_get_utc_milliseconds_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_GET_UTC_MILLISECONDS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_set_time_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_SET_TIME_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_set_milliseconds_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_SET_MILLISECONDS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_set_utc_milliseconds_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_SET_UTC_MILLISECONDS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_set_seconds_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_SET_SECONDS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_set_utc_seconds_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_SET_UTC_SECONDS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_set_minutes_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_SET_MINUTES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_set_utc_minutes_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_SET_UTC_MINUTES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_set_hours_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_SET_HOURS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_set_utc_hours_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_SET_UTC_HOURS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_set_date_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_SET_DATE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_set_utc_date_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_SET_UTC_DATE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_set_month_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_SET_MONTH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_set_utc_month_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_SET_UTC_MONTH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_set_full_year_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_SET_FULL_YEAR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_set_year_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_SET_YEAR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_set_utc_full_year_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_SET_UTC_FULL_YEAR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_to_utc_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_TO_UTC_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_to_iso_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_TO_ISO_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_to_json_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_TO_JSON_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_to_primitive_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_TO_PRIMITIVE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn date_to_temporal_instant_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATE_TO_TEMPORAL_INSTANT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

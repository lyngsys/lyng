use super::super::*;

#[inline]
pub fn temporal_plain_date_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_year_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_YEAR_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_month_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_MONTH_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_month_code_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_MONTH_CODE_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_day_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_DAY_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_day_of_week_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_DAY_OF_WEEK_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_day_of_year_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_DAY_OF_YEAR_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_days_in_month_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_DAYS_IN_MONTH_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_days_in_year_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_DAYS_IN_YEAR_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_months_in_year_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_MONTHS_IN_YEAR_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_in_leap_year_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_IN_LEAP_YEAR_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_days_in_week_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_DAYS_IN_WEEK_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_week_of_year_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_WEEK_OF_YEAR_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_year_of_week_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_YEAR_OF_WEEK_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_era_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_ERA_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_era_year_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_ERA_YEAR_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_calendar_id_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_CALENDAR_ID_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_to_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_TO_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_to_json_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_TO_JSON_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_to_locale_string_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_TO_LOCALE_STRING_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_value_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_VALUE_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_from_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_FROM_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_compare_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_COMPARE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_equals_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_EQUALS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_with_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_WITH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_add_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_ADD_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_subtract_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_SUBTRACT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_since_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_SINCE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_until_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_UNTIL_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_to_plain_date_time_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_TO_PLAIN_DATE_TIME_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_to_plain_year_month_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_TO_PLAIN_YEAR_MONTH_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_to_plain_month_day_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_TO_PLAIN_MONTH_DAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_with_calendar_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_WITH_CALENDAR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn temporal_plain_date_to_zoned_date_time_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(TEMPORAL_PLAIN_DATE_TO_ZONED_DATE_TIME_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

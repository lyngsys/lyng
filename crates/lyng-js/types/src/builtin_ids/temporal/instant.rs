use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    temporal_instant_builtin => super::super::TEMPORAL_INSTANT_RAW;
    temporal_instant_from_builtin => super::super::TEMPORAL_INSTANT_FROM_RAW;
    temporal_instant_from_epoch_nanoseconds_builtin => super::super::TEMPORAL_INSTANT_FROM_EPOCH_NANOSECONDS_RAW;
    temporal_instant_from_epoch_milliseconds_builtin => super::super::TEMPORAL_INSTANT_FROM_EPOCH_MILLISECONDS_RAW;
    temporal_instant_compare_builtin => super::super::TEMPORAL_INSTANT_COMPARE_RAW;
    temporal_instant_epoch_nanoseconds_getter_builtin => super::super::TEMPORAL_INSTANT_EPOCH_NANOSECONDS_GETTER_RAW;
    temporal_instant_epoch_milliseconds_getter_builtin => super::super::TEMPORAL_INSTANT_EPOCH_MILLISECONDS_GETTER_RAW;
    temporal_instant_epoch_seconds_getter_builtin => super::super::TEMPORAL_INSTANT_EPOCH_SECONDS_GETTER_RAW;
    temporal_instant_to_string_builtin => super::super::TEMPORAL_INSTANT_TO_STRING_RAW;
    temporal_instant_to_json_builtin => super::super::TEMPORAL_INSTANT_TO_JSON_RAW;
    temporal_instant_to_locale_string_builtin => super::super::TEMPORAL_INSTANT_TO_LOCALE_STRING_RAW;
    temporal_instant_value_of_builtin => super::super::TEMPORAL_INSTANT_VALUE_OF_RAW;
    temporal_instant_equals_builtin => super::super::TEMPORAL_INSTANT_EQUALS_RAW;
    temporal_instant_add_builtin => super::super::TEMPORAL_INSTANT_ADD_RAW;
    temporal_instant_subtract_builtin => super::super::TEMPORAL_INSTANT_SUBTRACT_RAW;
    temporal_instant_round_builtin => super::super::TEMPORAL_INSTANT_ROUND_RAW;
    temporal_instant_since_builtin => super::super::TEMPORAL_INSTANT_SINCE_RAW;
    temporal_instant_until_builtin => super::super::TEMPORAL_INSTANT_UNTIL_RAW;
    temporal_instant_to_zoned_date_time_iso_builtin => super::super::TEMPORAL_INSTANT_TO_ZONED_DATE_TIME_ISO_RAW;
}

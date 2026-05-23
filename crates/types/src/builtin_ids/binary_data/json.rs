use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    json_parse_builtin => super::super::JSON_PARSE_RAW;
    json_stringify_builtin => super::super::JSON_STRINGIFY_RAW;
    json_raw_json_builtin => super::super::JSON_RAW_JSON_RAW;
    json_is_raw_json_builtin => super::super::JSON_IS_RAW_JSON_RAW;
}

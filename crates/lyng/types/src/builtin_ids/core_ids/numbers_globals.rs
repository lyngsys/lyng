use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    eval_builtin => super::super::EVAL_RAW;
    parse_int_builtin => super::super::PARSE_INT_RAW;
    parse_float_builtin => super::super::PARSE_FLOAT_RAW;
    is_nan_builtin => super::super::IS_NAN_RAW;
    is_finite_builtin => super::super::IS_FINITE_RAW;
    encode_uri_builtin => super::super::ENCODE_URI_RAW;
    encode_uri_component_builtin => super::super::ENCODE_URI_COMPONENT_RAW;
    decode_uri_builtin => super::super::DECODE_URI_RAW;
    decode_uri_component_builtin => super::super::DECODE_URI_COMPONENT_RAW;
    escape_builtin => super::super::ESCAPE_RAW;
    unescape_builtin => super::super::UNESCAPE_RAW;
}

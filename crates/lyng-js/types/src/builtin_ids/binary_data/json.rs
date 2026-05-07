use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn json_parse_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::JSON_PARSE_RAW)
}

#[inline]
pub const fn json_stringify_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::JSON_STRINGIFY_RAW)
}

#[inline]
pub const fn json_raw_json_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::JSON_RAW_JSON_RAW)
}

#[inline]
pub const fn json_is_raw_json_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::JSON_IS_RAW_JSON_RAW)
}

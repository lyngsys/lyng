use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn eval_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::EVAL_RAW)
}

#[inline]
pub const fn parse_int_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PARSE_INT_RAW)
}

#[inline]
pub const fn parse_float_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PARSE_FLOAT_RAW)
}

#[inline]
pub const fn is_nan_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::IS_NAN_RAW)
}

#[inline]
pub const fn is_finite_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::IS_FINITE_RAW)
}

#[inline]
pub const fn encode_uri_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ENCODE_URI_RAW)
}

#[inline]
pub const fn encode_uri_component_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ENCODE_URI_COMPONENT_RAW)
}

#[inline]
pub const fn decode_uri_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::DECODE_URI_RAW)
}

#[inline]
pub const fn decode_uri_component_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::DECODE_URI_COMPONENT_RAW)
}

#[inline]
pub const fn escape_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ESCAPE_RAW)
}

#[inline]
pub const fn unescape_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::UNESCAPE_RAW)
}

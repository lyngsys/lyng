use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn int8_array_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INT8_ARRAY_RAW)
}

#[inline]
pub const fn int16_array_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INT16_ARRAY_RAW)
}

#[inline]
pub const fn int32_array_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INT32_ARRAY_RAW)
}

#[inline]
pub const fn uint32_array_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::UINT32_ARRAY_RAW)
}

#[inline]
pub const fn float32_array_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::FLOAT32_ARRAY_RAW)
}

#[inline]
pub const fn float16_array_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::FLOAT16_ARRAY_RAW)
}

#[inline]
pub const fn float64_array_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::FLOAT64_ARRAY_RAW)
}

#[inline]
pub const fn uint8_clamped_array_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::UINT8_CLAMPED_ARRAY_RAW)
}

#[inline]
pub const fn big_uint64_array_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::BIG_UINT64_ARRAY_RAW)
}

#[inline]
pub const fn big_int64_array_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::BIG_INT64_ARRAY_RAW)
}

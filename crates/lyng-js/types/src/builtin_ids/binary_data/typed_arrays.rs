use super::super::*;

#[inline]
pub fn int8_array_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INT8_ARRAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn int16_array_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INT16_ARRAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn int32_array_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INT32_ARRAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn uint32_array_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(UINT32_ARRAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn float32_array_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(FLOAT32_ARRAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn float64_array_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(FLOAT64_ARRAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn uint8_clamped_array_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(UINT8_CLAMPED_ARRAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn big_uint64_array_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(BIG_UINT64_ARRAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn big_int64_array_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(BIG_INT64_ARRAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

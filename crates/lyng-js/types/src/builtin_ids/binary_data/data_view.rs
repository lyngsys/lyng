use super::super::*;

#[inline]
pub fn data_view_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_buffer_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_BUFFER_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_byte_length_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_BYTE_LENGTH_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_byte_offset_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_BYTE_OFFSET_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_get_uint8_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_GET_UINT8_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_set_uint8_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_SET_UINT8_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_get_int8_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_GET_INT8_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_set_int8_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_SET_INT8_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_get_uint16_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_GET_UINT16_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_set_uint16_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_SET_UINT16_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_get_int16_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_GET_INT16_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_set_int16_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_SET_INT16_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_get_uint32_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_GET_UINT32_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_set_uint32_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_SET_UINT32_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_get_int32_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_GET_INT32_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_set_int32_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_SET_INT32_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_get_float32_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_GET_FLOAT32_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_set_float32_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_SET_FLOAT32_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_get_float64_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_GET_FLOAT64_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_set_float64_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_SET_FLOAT64_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_get_big_int64_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_GET_BIG_INT64_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_get_big_uint64_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_GET_BIG_UINT64_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_set_big_int64_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_SET_BIG_INT64_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_set_big_uint64_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_SET_BIG_UINT64_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_get_float16_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_GET_FLOAT16_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn data_view_set_float16_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(DATA_VIEW_SET_FLOAT16_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

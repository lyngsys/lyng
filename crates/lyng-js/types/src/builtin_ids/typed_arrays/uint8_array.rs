use super::super::*;

#[inline]
pub fn uint16_array_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(UINT16_ARRAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn uint8_array_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(UINT8_ARRAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn uint8_array_buffer_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(UINT8_ARRAY_BUFFER_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn uint8_array_byte_length_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(UINT8_ARRAY_BYTE_LENGTH_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn uint8_array_byte_offset_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(UINT8_ARRAY_BYTE_OFFSET_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn uint8_array_length_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(UINT8_ARRAY_LENGTH_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn uint8_array_values_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(UINT8_ARRAY_VALUES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn uint8_array_keys_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(UINT8_ARRAY_KEYS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn uint8_array_entries_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(UINT8_ARRAY_ENTRIES_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn uint8_array_set_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(UINT8_ARRAY_SET_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn uint8_array_subarray_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(UINT8_ARRAY_SUBARRAY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn uint8_array_slice_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(UINT8_ARRAY_SLICE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

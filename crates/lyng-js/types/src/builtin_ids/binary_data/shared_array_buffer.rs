use super::super::*;

#[inline]
pub fn shared_array_buffer_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SHARED_ARRAY_BUFFER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn shared_array_buffer_byte_length_getter_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SHARED_ARRAY_BUFFER_BYTE_LENGTH_GETTER_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn shared_array_buffer_slice_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SHARED_ARRAY_BUFFER_SLICE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn shared_array_buffer_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SHARED_ARRAY_BUFFER_RAW)
}

#[inline]
pub const fn shared_array_buffer_byte_length_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SHARED_ARRAY_BUFFER_BYTE_LENGTH_GETTER_RAW)
}

#[inline]
pub const fn shared_array_buffer_grow_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SHARED_ARRAY_BUFFER_GROW_RAW)
}

#[inline]
pub const fn shared_array_buffer_growable_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SHARED_ARRAY_BUFFER_GROWABLE_GETTER_RAW)
}

#[inline]
pub const fn shared_array_buffer_max_byte_length_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SHARED_ARRAY_BUFFER_MAX_BYTE_LENGTH_GETTER_RAW)
}

#[inline]
pub const fn shared_array_buffer_slice_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SHARED_ARRAY_BUFFER_SLICE_RAW)
}

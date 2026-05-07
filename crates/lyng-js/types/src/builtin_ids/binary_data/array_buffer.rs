use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn array_buffer_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ARRAY_BUFFER_RAW)
}

#[inline]
pub const fn array_buffer_is_view_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ARRAY_BUFFER_IS_VIEW_RAW)
}

#[inline]
pub const fn array_buffer_byte_length_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ARRAY_BUFFER_BYTE_LENGTH_GETTER_RAW)
}

#[inline]
pub const fn array_buffer_detached_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ARRAY_BUFFER_DETACHED_GETTER_RAW)
}

#[inline]
pub const fn array_buffer_max_byte_length_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ARRAY_BUFFER_MAX_BYTE_LENGTH_GETTER_RAW)
}

#[inline]
pub const fn array_buffer_resizable_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ARRAY_BUFFER_RESIZABLE_GETTER_RAW)
}

#[inline]
pub const fn array_buffer_slice_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ARRAY_BUFFER_SLICE_RAW)
}

#[inline]
pub const fn array_buffer_resize_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ARRAY_BUFFER_RESIZE_RAW)
}

#[inline]
pub const fn array_buffer_transfer_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ARRAY_BUFFER_TRANSFER_RAW)
}

#[inline]
pub const fn array_buffer_transfer_to_fixed_length_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ARRAY_BUFFER_TRANSFER_TO_FIXED_LENGTH_RAW)
}

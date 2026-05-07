use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn internal_array_index_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_ARRAY_INDEX_OF_RAW)
}

#[inline]
pub const fn internal_array_push_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_ARRAY_PUSH_RAW)
}

#[inline]
pub const fn internal_array_pop_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_ARRAY_POP_RAW)
}

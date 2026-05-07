use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn iterator_prototype_iterator_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_PROTOTYPE_ITERATOR_RAW)
}

#[inline]
pub const fn array_iterator_next_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ARRAY_ITERATOR_NEXT_RAW)
}

#[inline]
pub const fn string_iterator_next_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_ITERATOR_NEXT_RAW)
}

#[inline]
pub const fn string_iterator_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::STRING_ITERATOR_RAW)
}

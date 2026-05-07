use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn iterator_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_RAW)
}

#[inline]
pub const fn iterator_from_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_FROM_RAW)
}

#[inline]
pub const fn iterator_concat_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_CONCAT_RAW)
}

#[inline]
pub const fn iterator_zip_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_ZIP_RAW)
}

#[inline]
pub const fn iterator_zip_keyed_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_ZIP_KEYED_RAW)
}

#[inline]
pub const fn iterator_reduce_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_REDUCE_RAW)
}

#[inline]
pub const fn iterator_for_each_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_FOR_EACH_RAW)
}

#[inline]
pub const fn iterator_some_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_SOME_RAW)
}

#[inline]
pub const fn iterator_every_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_EVERY_RAW)
}

#[inline]
pub const fn iterator_find_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_FIND_RAW)
}

#[inline]
pub const fn iterator_to_array_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_TO_ARRAY_RAW)
}

#[inline]
pub const fn iterator_map_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_MAP_RAW)
}

#[inline]
pub const fn iterator_filter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_FILTER_RAW)
}

#[inline]
pub const fn iterator_take_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_TAKE_RAW)
}

#[inline]
pub const fn iterator_drop_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_DROP_RAW)
}

#[inline]
pub const fn iterator_dispose_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_DISPOSE_RAW)
}

#[inline]
pub const fn async_iterator_dispose_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ASYNC_ITERATOR_DISPOSE_RAW)
}

#[inline]
pub const fn iterator_flat_map_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_FLAT_MAP_RAW)
}

#[inline]
pub const fn iterator_helper_next_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_HELPER_NEXT_RAW)
}

#[inline]
pub const fn iterator_helper_return_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_HELPER_RETURN_RAW)
}

#[inline]
pub const fn iterator_to_string_tag_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_TO_STRING_TAG_GETTER_RAW)
}

#[inline]
pub const fn iterator_to_string_tag_setter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_TO_STRING_TAG_SETTER_RAW)
}

#[inline]
pub const fn iterator_constructor_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_CONSTRUCTOR_GETTER_RAW)
}

#[inline]
pub const fn iterator_constructor_setter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ITERATOR_CONSTRUCTOR_SETTER_RAW)
}

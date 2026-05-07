use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn map_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::MAP_RAW)
}

#[inline]
pub const fn map_group_by_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::MAP_GROUP_BY_RAW)
}

#[inline]
pub const fn map_get_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::MAP_GET_RAW)
}

#[inline]
pub const fn map_set_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::MAP_SET_RAW)
}

#[inline]
pub const fn map_has_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::MAP_HAS_RAW)
}

#[inline]
pub const fn map_delete_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::MAP_DELETE_RAW)
}

#[inline]
pub const fn map_clear_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::MAP_CLEAR_RAW)
}

#[inline]
pub const fn map_entries_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::MAP_ENTRIES_RAW)
}

#[inline]
pub const fn map_values_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::MAP_VALUES_RAW)
}

#[inline]
pub const fn map_keys_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::MAP_KEYS_RAW)
}

#[inline]
pub const fn map_size_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::MAP_SIZE_GETTER_RAW)
}

#[inline]
pub const fn map_iterator_next_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::MAP_ITERATOR_NEXT_RAW)
}

#[inline]
pub const fn map_for_each_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::MAP_FOR_EACH_RAW)
}

#[inline]
pub const fn map_get_or_insert_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::MAP_GET_OR_INSERT_RAW)
}

#[inline]
pub const fn map_get_or_insert_computed_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::MAP_GET_OR_INSERT_COMPUTED_RAW)
}

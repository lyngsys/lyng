use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn weak_map_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::WEAK_MAP_RAW)
}

#[inline]
pub const fn weak_map_get_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::WEAK_MAP_GET_RAW)
}

#[inline]
pub const fn weak_map_set_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::WEAK_MAP_SET_RAW)
}

#[inline]
pub const fn weak_map_has_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::WEAK_MAP_HAS_RAW)
}

#[inline]
pub const fn weak_map_delete_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::WEAK_MAP_DELETE_RAW)
}

#[inline]
pub const fn weak_map_get_or_insert_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::WEAK_MAP_GET_OR_INSERT_RAW)
}

#[inline]
pub const fn weak_map_get_or_insert_computed_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::WEAK_MAP_GET_OR_INSERT_COMPUTED_RAW)
}

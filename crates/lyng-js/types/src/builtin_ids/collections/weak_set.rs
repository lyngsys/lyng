use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn weak_set_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::WEAK_SET_RAW)
}

#[inline]
pub const fn weak_set_add_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::WEAK_SET_ADD_RAW)
}

#[inline]
pub const fn weak_set_has_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::WEAK_SET_HAS_RAW)
}

#[inline]
pub const fn weak_set_delete_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::WEAK_SET_DELETE_RAW)
}

use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn set_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SET_RAW)
}

#[inline]
pub const fn set_add_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SET_ADD_RAW)
}

#[inline]
pub const fn set_has_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SET_HAS_RAW)
}

#[inline]
pub const fn set_delete_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SET_DELETE_RAW)
}

#[inline]
pub const fn set_clear_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SET_CLEAR_RAW)
}

#[inline]
pub const fn set_entries_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SET_ENTRIES_RAW)
}

#[inline]
pub const fn set_values_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SET_VALUES_RAW)
}

#[inline]
pub const fn set_keys_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SET_KEYS_RAW)
}

#[inline]
pub const fn set_size_getter_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SET_SIZE_GETTER_RAW)
}

#[inline]
pub const fn set_iterator_next_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SET_ITERATOR_NEXT_RAW)
}

#[inline]
pub const fn set_for_each_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SET_FOR_EACH_RAW)
}

#[inline]
pub const fn set_union_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SET_UNION_RAW)
}

#[inline]
pub const fn set_intersection_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SET_INTERSECTION_RAW)
}

#[inline]
pub const fn set_difference_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SET_DIFFERENCE_RAW)
}

#[inline]
pub const fn set_symmetric_difference_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SET_SYMMETRIC_DIFFERENCE_RAW)
}

#[inline]
pub const fn set_is_subset_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SET_IS_SUBSET_OF_RAW)
}

#[inline]
pub const fn set_is_superset_of_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SET_IS_SUPERSET_OF_RAW)
}

#[inline]
pub const fn set_is_disjoint_from_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SET_IS_DISJOINT_FROM_RAW)
}

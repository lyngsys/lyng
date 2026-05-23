use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    set_builtin => super::super::SET_RAW;
    set_add_builtin => super::super::SET_ADD_RAW;
    set_has_builtin => super::super::SET_HAS_RAW;
    set_delete_builtin => super::super::SET_DELETE_RAW;
    set_clear_builtin => super::super::SET_CLEAR_RAW;
    set_entries_builtin => super::super::SET_ENTRIES_RAW;
    set_values_builtin => super::super::SET_VALUES_RAW;
    set_keys_builtin => super::super::SET_KEYS_RAW;
    set_size_getter_builtin => super::super::SET_SIZE_GETTER_RAW;
    set_iterator_next_builtin => super::super::SET_ITERATOR_NEXT_RAW;
    set_for_each_builtin => super::super::SET_FOR_EACH_RAW;
    set_union_builtin => super::super::SET_UNION_RAW;
    set_intersection_builtin => super::super::SET_INTERSECTION_RAW;
    set_difference_builtin => super::super::SET_DIFFERENCE_RAW;
    set_symmetric_difference_builtin => super::super::SET_SYMMETRIC_DIFFERENCE_RAW;
    set_is_subset_of_builtin => super::super::SET_IS_SUBSET_OF_RAW;
    set_is_superset_of_builtin => super::super::SET_IS_SUPERSET_OF_RAW;
    set_is_disjoint_from_builtin => super::super::SET_IS_DISJOINT_FROM_RAW;
}

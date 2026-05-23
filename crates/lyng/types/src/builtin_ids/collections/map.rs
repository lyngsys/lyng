use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    map_builtin => super::super::MAP_RAW;
    map_group_by_builtin => super::super::MAP_GROUP_BY_RAW;
    map_get_builtin => super::super::MAP_GET_RAW;
    map_set_builtin => super::super::MAP_SET_RAW;
    map_has_builtin => super::super::MAP_HAS_RAW;
    map_delete_builtin => super::super::MAP_DELETE_RAW;
    map_clear_builtin => super::super::MAP_CLEAR_RAW;
    map_entries_builtin => super::super::MAP_ENTRIES_RAW;
    map_values_builtin => super::super::MAP_VALUES_RAW;
    map_keys_builtin => super::super::MAP_KEYS_RAW;
    map_size_getter_builtin => super::super::MAP_SIZE_GETTER_RAW;
    map_iterator_next_builtin => super::super::MAP_ITERATOR_NEXT_RAW;
    map_for_each_builtin => super::super::MAP_FOR_EACH_RAW;
    map_get_or_insert_builtin => super::super::MAP_GET_OR_INSERT_RAW;
    map_get_or_insert_computed_builtin => super::super::MAP_GET_OR_INSERT_COMPUTED_RAW;
}

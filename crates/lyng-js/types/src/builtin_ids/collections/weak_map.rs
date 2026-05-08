use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    weak_map_builtin => super::super::WEAK_MAP_RAW;
    weak_map_get_builtin => super::super::WEAK_MAP_GET_RAW;
    weak_map_set_builtin => super::super::WEAK_MAP_SET_RAW;
    weak_map_has_builtin => super::super::WEAK_MAP_HAS_RAW;
    weak_map_delete_builtin => super::super::WEAK_MAP_DELETE_RAW;
    weak_map_get_or_insert_builtin => super::super::WEAK_MAP_GET_OR_INSERT_RAW;
    weak_map_get_or_insert_computed_builtin => super::super::WEAK_MAP_GET_OR_INSERT_COMPUTED_RAW;
}

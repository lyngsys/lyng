use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    weak_set_builtin => super::super::WEAK_SET_RAW;
    weak_set_add_builtin => super::super::WEAK_SET_ADD_RAW;
    weak_set_has_builtin => super::super::WEAK_SET_HAS_RAW;
    weak_set_delete_builtin => super::super::WEAK_SET_DELETE_RAW;
}

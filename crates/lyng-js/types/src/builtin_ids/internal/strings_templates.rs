use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    internal_string_replace_builtin => super::super::INTERNAL_STRING_REPLACE_RAW;
    internal_string_index_of_builtin => super::super::INTERNAL_STRING_INDEX_OF_RAW;
    internal_object_to_string_builtin => super::super::INTERNAL_OBJECT_TO_STRING_RAW;
    internal_template_to_string_builtin => super::super::INTERNAL_TEMPLATE_TO_STRING_RAW;
    internal_get_template_object_builtin => super::super::INTERNAL_GET_TEMPLATE_OBJECT_RAW;
}

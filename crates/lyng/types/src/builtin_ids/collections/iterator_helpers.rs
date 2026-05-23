use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    iterator_builtin => super::super::ITERATOR_RAW;
    iterator_from_builtin => super::super::ITERATOR_FROM_RAW;
    iterator_concat_builtin => super::super::ITERATOR_CONCAT_RAW;
    iterator_zip_builtin => super::super::ITERATOR_ZIP_RAW;
    iterator_zip_keyed_builtin => super::super::ITERATOR_ZIP_KEYED_RAW;
    iterator_reduce_builtin => super::super::ITERATOR_REDUCE_RAW;
    iterator_for_each_builtin => super::super::ITERATOR_FOR_EACH_RAW;
    iterator_some_builtin => super::super::ITERATOR_SOME_RAW;
    iterator_every_builtin => super::super::ITERATOR_EVERY_RAW;
    iterator_find_builtin => super::super::ITERATOR_FIND_RAW;
    iterator_to_array_builtin => super::super::ITERATOR_TO_ARRAY_RAW;
    iterator_map_builtin => super::super::ITERATOR_MAP_RAW;
    iterator_filter_builtin => super::super::ITERATOR_FILTER_RAW;
    iterator_take_builtin => super::super::ITERATOR_TAKE_RAW;
    iterator_drop_builtin => super::super::ITERATOR_DROP_RAW;
    iterator_dispose_builtin => super::super::ITERATOR_DISPOSE_RAW;
    async_iterator_dispose_builtin => super::super::ASYNC_ITERATOR_DISPOSE_RAW;
    iterator_flat_map_builtin => super::super::ITERATOR_FLAT_MAP_RAW;
    iterator_helper_next_builtin => super::super::ITERATOR_HELPER_NEXT_RAW;
    iterator_helper_return_builtin => super::super::ITERATOR_HELPER_RETURN_RAW;
    iterator_to_string_tag_getter_builtin => super::super::ITERATOR_TO_STRING_TAG_GETTER_RAW;
    iterator_to_string_tag_setter_builtin => super::super::ITERATOR_TO_STRING_TAG_SETTER_RAW;
    iterator_constructor_getter_builtin => super::super::ITERATOR_CONSTRUCTOR_GETTER_RAW;
    iterator_constructor_setter_builtin => super::super::ITERATOR_CONSTRUCTOR_SETTER_RAW;
}

use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    iterator_prototype_iterator_builtin => super::super::ITERATOR_PROTOTYPE_ITERATOR_RAW;
    array_iterator_next_builtin => super::super::ARRAY_ITERATOR_NEXT_RAW;
    string_iterator_next_builtin => super::super::STRING_ITERATOR_NEXT_RAW;
    string_iterator_builtin => super::super::STRING_ITERATOR_RAW;
}

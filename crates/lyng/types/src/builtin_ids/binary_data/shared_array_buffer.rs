use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    shared_array_buffer_builtin => super::super::SHARED_ARRAY_BUFFER_RAW;
    shared_array_buffer_byte_length_getter_builtin => super::super::SHARED_ARRAY_BUFFER_BYTE_LENGTH_GETTER_RAW;
    shared_array_buffer_grow_builtin => super::super::SHARED_ARRAY_BUFFER_GROW_RAW;
    shared_array_buffer_growable_getter_builtin => super::super::SHARED_ARRAY_BUFFER_GROWABLE_GETTER_RAW;
    shared_array_buffer_max_byte_length_getter_builtin => super::super::SHARED_ARRAY_BUFFER_MAX_BYTE_LENGTH_GETTER_RAW;
    shared_array_buffer_slice_builtin => super::super::SHARED_ARRAY_BUFFER_SLICE_RAW;
}

use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    internal_function_call_builtin => super::super::INTERNAL_FUNCTION_CALL_RAW;
    internal_set_function_home_object_builtin => super::super::INTERNAL_SET_FUNCTION_HOME_OBJECT_RAW;
    internal_capture_arrow_context_builtin => super::super::INTERNAL_CAPTURE_ARROW_CONTEXT_RAW;
}

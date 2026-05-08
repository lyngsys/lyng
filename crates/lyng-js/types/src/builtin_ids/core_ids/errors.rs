use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    error_builtin => super::super::ERROR_RAW;
    error_to_string_builtin => super::super::ERROR_TO_STRING_RAW;
    eval_error_builtin => super::super::EVAL_ERROR_RAW;
    range_error_builtin => super::super::RANGE_ERROR_RAW;
    reference_error_builtin => super::super::REFERENCE_ERROR_RAW;
    syntax_error_builtin => super::super::SYNTAX_ERROR_RAW;
    type_error_builtin => super::super::TYPE_ERROR_RAW;
    uri_error_builtin => super::super::URI_ERROR_RAW;
    error_is_error_builtin => super::super::ERROR_IS_ERROR_RAW;
}

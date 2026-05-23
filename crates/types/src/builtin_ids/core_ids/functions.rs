use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    function_builtin => super::super::FUNCTION_RAW;
    function_prototype_builtin => super::super::FUNCTION_PROTOTYPE_RAW;
    function_call_builtin => super::super::FUNCTION_CALL_RAW;
    function_apply_builtin => super::super::FUNCTION_APPLY_RAW;
    function_bind_builtin => super::super::FUNCTION_BIND_RAW;
    function_to_string_builtin => super::super::FUNCTION_TO_STRING_RAW;
    generator_function_builtin => super::super::GENERATOR_FUNCTION_RAW;
    generator_next_builtin => super::super::GENERATOR_NEXT_RAW;
    generator_return_builtin => super::super::GENERATOR_RETURN_RAW;
    generator_throw_builtin => super::super::GENERATOR_THROW_RAW;
    function_symbol_has_instance_builtin => super::super::FUNCTION_SYMBOL_HAS_INSTANCE_RAW;
}

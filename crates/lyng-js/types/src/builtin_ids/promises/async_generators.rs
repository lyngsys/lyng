use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    async_generator_function_builtin => super::super::ASYNC_GENERATOR_FUNCTION_RAW;
    async_generator_next_builtin => super::super::ASYNC_GENERATOR_NEXT_RAW;
    async_generator_return_builtin => super::super::ASYNC_GENERATOR_RETURN_RAW;
    async_generator_throw_builtin => super::super::ASYNC_GENERATOR_THROW_RAW;
}

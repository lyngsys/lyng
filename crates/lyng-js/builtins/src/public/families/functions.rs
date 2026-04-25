use super::{
    install_public_builtin_function, FamilyInstallContext, FunctionFamilyBuiltins,
    FunctionFamilyPrototypes,
};
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_async_function_builtin, js3_async_generator_function_builtin,
    js3_async_generator_next_builtin, js3_async_generator_return_builtin,
    js3_async_generator_throw_builtin, js3_function_apply_builtin, js3_function_bind_builtin,
    js3_function_builtin, js3_function_call_builtin, js3_function_symbol_has_instance_builtin,
    js3_function_to_string_builtin, js3_generator_function_builtin, js3_generator_next_builtin,
    js3_generator_return_builtin, js3_generator_throw_builtin,
};

#[allow(clippy::too_many_lines)]
pub(in crate::public) fn install_function_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototypes: FunctionFamilyPrototypes,
) -> FunctionFamilyBuiltins {
    FunctionFamilyBuiltins {
        function: install_public_builtin_function(
            agent,
            cx,
            js3_function_builtin(),
            Some(cx.function_prototype),
        ),
        function_prototype: cx.function_prototype,
        function_call: install_public_builtin_function(
            agent,
            cx,
            js3_function_call_builtin(),
            None,
        ),
        function_apply: install_public_builtin_function(
            agent,
            cx,
            js3_function_apply_builtin(),
            None,
        ),
        function_bind: install_public_builtin_function(
            agent,
            cx,
            js3_function_bind_builtin(),
            None,
        ),
        function_to_string: install_public_builtin_function(
            agent,
            cx,
            js3_function_to_string_builtin(),
            None,
        ),
        function_symbol_has_instance: install_public_builtin_function(
            agent,
            cx,
            js3_function_symbol_has_instance_builtin(),
            None,
        ),
        async_function: install_public_builtin_function(
            agent,
            cx,
            js3_async_function_builtin(),
            Some(prototypes.async_function_prototype),
        ),
        async_function_prototype: prototypes.async_function_prototype,
        async_generator_function: install_public_builtin_function(
            agent,
            cx,
            js3_async_generator_function_builtin(),
            Some(prototypes.async_generator_function_prototype),
        ),
        async_generator_function_prototype: prototypes.async_generator_function_prototype,
        async_generator_prototype: prototypes.async_generator_prototype,
        async_generator_next: install_public_builtin_function(
            agent,
            cx,
            js3_async_generator_next_builtin(),
            None,
        ),
        async_generator_return: install_public_builtin_function(
            agent,
            cx,
            js3_async_generator_return_builtin(),
            None,
        ),
        async_generator_throw: install_public_builtin_function(
            agent,
            cx,
            js3_async_generator_throw_builtin(),
            None,
        ),
        generator_function: install_public_builtin_function(
            agent,
            cx,
            js3_generator_function_builtin(),
            Some(prototypes.generator_function_prototype),
        ),
        generator_function_prototype: prototypes.generator_function_prototype,
        generator_prototype: prototypes.generator_prototype,
        generator_next: install_public_builtin_function(
            agent,
            cx,
            js3_generator_next_builtin(),
            None,
        ),
        generator_return: install_public_builtin_function(
            agent,
            cx,
            js3_generator_return_builtin(),
            None,
        ),
        generator_throw: install_public_builtin_function(
            agent,
            cx,
            js3_generator_throw_builtin(),
            None,
        ),
    }
}

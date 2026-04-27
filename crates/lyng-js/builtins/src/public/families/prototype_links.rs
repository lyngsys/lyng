use crate::public::{reparent_builtin_object, PublicRealmBuiltins};
use lyng_js_env::Agent;

pub(in crate::public) fn link_installed_family_prototypes(
    agent: &mut Agent,
    builtins: &PublicRealmBuiltins,
) {
    reparent_builtin_object(agent, builtins.async_function, Some(builtins.function));
    reparent_builtin_object(agent, builtins.generator_function, Some(builtins.function));
    reparent_builtin_object(
        agent,
        builtins.async_generator_function,
        Some(builtins.function),
    );
    reparent_builtin_object(agent, builtins.int8_array, Some(builtins.typed_array));
    reparent_builtin_object(agent, builtins.int16_array, Some(builtins.typed_array));
    reparent_builtin_object(agent, builtins.int32_array, Some(builtins.typed_array));
    reparent_builtin_object(agent, builtins.float32_array, Some(builtins.typed_array));
    reparent_builtin_object(agent, builtins.float64_array, Some(builtins.typed_array));
    reparent_builtin_object(agent, builtins.big_int64_array, Some(builtins.typed_array));
    reparent_builtin_object(agent, builtins.big_uint64_array, Some(builtins.typed_array));
    reparent_builtin_object(agent, builtins.uint32_array, Some(builtins.typed_array));
    reparent_builtin_object(agent, builtins.uint16_array, Some(builtins.typed_array));
    reparent_builtin_object(
        agent,
        builtins.uint8_clamped_array,
        Some(builtins.typed_array),
    );
    reparent_builtin_object(agent, builtins.uint8_array, Some(builtins.typed_array));
}

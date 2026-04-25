use super::{install_public_builtin_function, FamilyInstallContext, GlobalFunctionFamilyBuiltins};
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_decode_uri_builtin, js3_decode_uri_component_builtin, js3_encode_uri_builtin,
    js3_encode_uri_component_builtin, js3_eval_builtin, js3_is_finite_builtin, js3_is_nan_builtin,
    js3_parse_float_builtin, js3_parse_int_builtin,
};

pub(in crate::public) fn install_global_function_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
) -> GlobalFunctionFamilyBuiltins {
    GlobalFunctionFamilyBuiltins {
        eval: install_public_builtin_function(agent, cx, js3_eval_builtin(), None),
        parse_int: install_public_builtin_function(agent, cx, js3_parse_int_builtin(), None),
        parse_float: install_public_builtin_function(agent, cx, js3_parse_float_builtin(), None),
        is_nan: install_public_builtin_function(agent, cx, js3_is_nan_builtin(), None),
        is_finite: install_public_builtin_function(agent, cx, js3_is_finite_builtin(), None),
        encode_uri: install_public_builtin_function(agent, cx, js3_encode_uri_builtin(), None),
        encode_uri_component: install_public_builtin_function(
            agent,
            cx,
            js3_encode_uri_component_builtin(),
            None,
        ),
        decode_uri: install_public_builtin_function(agent, cx, js3_decode_uri_builtin(), None),
        decode_uri_component: install_public_builtin_function(
            agent,
            cx,
            js3_decode_uri_component_builtin(),
            None,
        ),
    }
}

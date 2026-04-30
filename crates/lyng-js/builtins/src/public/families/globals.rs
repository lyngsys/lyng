use super::{install_public_builtin_function, FamilyInstallContext, GlobalFunctionFamilyBuiltins};
use crate::public::PublicRealmBuiltins;
use lyng_js_env::Agent;
use lyng_js_types::{
    decode_uri_builtin, decode_uri_component_builtin, encode_uri_builtin,
    encode_uri_component_builtin, escape_builtin, eval_builtin, is_finite_builtin, is_nan_builtin,
    parse_float_builtin, parse_int_builtin, unescape_builtin, BuiltinFunctionId, ObjectRef,
};

pub(in crate::public) fn install_global_function_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
) -> GlobalFunctionFamilyBuiltins {
    GlobalFunctionFamilyBuiltins {
        eval: install_public_builtin_function(agent, cx, eval_builtin(), None),
        parse_int: install_public_builtin_function(agent, cx, parse_int_builtin(), None),
        parse_float: install_public_builtin_function(agent, cx, parse_float_builtin(), None),
        is_nan: install_public_builtin_function(agent, cx, is_nan_builtin(), None),
        is_finite: install_public_builtin_function(agent, cx, is_finite_builtin(), None),
        encode_uri: install_public_builtin_function(agent, cx, encode_uri_builtin(), None),
        encode_uri_component: install_public_builtin_function(
            agent,
            cx,
            encode_uri_component_builtin(),
            None,
        ),
        decode_uri: install_public_builtin_function(agent, cx, decode_uri_builtin(), None),
        decode_uri_component: install_public_builtin_function(
            agent,
            cx,
            decode_uri_component_builtin(),
            None,
        ),
        escape: install_public_builtin_function(agent, cx, escape_builtin(), None),
        unescape: install_public_builtin_function(agent, cx, unescape_builtin(), None),
    }
}

pub(in crate::public) fn global_function_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (eval_builtin(), builtins.eval),
        (parse_int_builtin(), builtins.parse_int),
        (parse_float_builtin(), builtins.parse_float),
        (is_nan_builtin(), builtins.is_nan),
        (is_finite_builtin(), builtins.is_finite),
        (encode_uri_builtin(), builtins.encode_uri),
        (
            encode_uri_component_builtin(),
            builtins.encode_uri_component,
        ),
        (decode_uri_builtin(), builtins.decode_uri),
        (
            decode_uri_component_builtin(),
            builtins.decode_uri_component,
        ),
        (escape_builtin(), builtins.escape),
        (unescape_builtin(), builtins.unescape),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

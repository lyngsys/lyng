use super::{
    install_public_builtin_function, FamilyInstallContext, JsonFamilyBuiltins, JsonFamilyObjects,
};
use crate::public::PublicRealmBuiltins;
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_json_is_raw_json_builtin, js3_json_parse_builtin, js3_json_raw_json_builtin,
    js3_json_stringify_builtin, BuiltinFunctionId, ObjectRef,
};

pub(in crate::public) fn install_json_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    objects: JsonFamilyObjects,
) -> JsonFamilyBuiltins {
    JsonFamilyBuiltins {
        json: objects.json,
        json_parse: install_public_builtin_function(agent, cx, js3_json_parse_builtin(), None),
        json_stringify: install_public_builtin_function(
            agent,
            cx,
            js3_json_stringify_builtin(),
            None,
        ),
        json_raw_json: install_public_builtin_function(
            agent,
            cx,
            js3_json_raw_json_builtin(),
            None,
        ),
        json_is_raw_json: install_public_builtin_function(
            agent,
            cx,
            js3_json_is_raw_json_builtin(),
            None,
        ),
    }
}

pub(in crate::public) fn json_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (js3_json_parse_builtin(), builtins.json_parse),
        (js3_json_stringify_builtin(), builtins.json_stringify),
        (js3_json_raw_json_builtin(), builtins.json_raw_json),
        (js3_json_is_raw_json_builtin(), builtins.json_is_raw_json),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

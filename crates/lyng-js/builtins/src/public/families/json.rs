use super::{
    install_public_builtin_function, FamilyInstallContext, JsonFamilyBuiltins, JsonFamilyObjects,
};
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_json_is_raw_json_builtin, js3_json_parse_builtin, js3_json_raw_json_builtin,
    js3_json_stringify_builtin,
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

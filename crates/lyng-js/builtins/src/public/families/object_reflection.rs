use super::{
    install_public_builtin_function, FamilyInstallContext, ObjectReflectionFamilyBuiltins,
    ObjectReflectionFamilyObjects,
};
use crate::public::PublicRealmBuiltins;
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_proxy_builtin, js3_proxy_revocable_builtin, js3_reflect_apply_builtin,
    js3_reflect_construct_builtin, js3_reflect_define_property_builtin,
    js3_reflect_delete_property_builtin, js3_reflect_get_builtin,
    js3_reflect_get_own_property_descriptor_builtin, js3_reflect_get_prototype_of_builtin,
    js3_reflect_has_builtin, js3_reflect_is_extensible_builtin, js3_reflect_own_keys_builtin,
    js3_reflect_prevent_extensions_builtin, js3_reflect_set_builtin,
    js3_reflect_set_prototype_of_builtin, BuiltinFunctionId, ObjectRef,
};

pub(in crate::public) fn install_object_reflection_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    objects: ObjectReflectionFamilyObjects,
) -> ObjectReflectionFamilyBuiltins {
    ObjectReflectionFamilyBuiltins {
        reflect: objects.reflect,
        reflect_apply: install_public_builtin_function(
            agent,
            cx,
            js3_reflect_apply_builtin(),
            None,
        ),
        reflect_construct: install_public_builtin_function(
            agent,
            cx,
            js3_reflect_construct_builtin(),
            None,
        ),
        reflect_define_property: install_public_builtin_function(
            agent,
            cx,
            js3_reflect_define_property_builtin(),
            None,
        ),
        reflect_delete_property: install_public_builtin_function(
            agent,
            cx,
            js3_reflect_delete_property_builtin(),
            None,
        ),
        reflect_get: install_public_builtin_function(agent, cx, js3_reflect_get_builtin(), None),
        reflect_get_own_property_descriptor: install_public_builtin_function(
            agent,
            cx,
            js3_reflect_get_own_property_descriptor_builtin(),
            None,
        ),
        reflect_get_prototype_of: install_public_builtin_function(
            agent,
            cx,
            js3_reflect_get_prototype_of_builtin(),
            None,
        ),
        reflect_has: install_public_builtin_function(agent, cx, js3_reflect_has_builtin(), None),
        reflect_is_extensible: install_public_builtin_function(
            agent,
            cx,
            js3_reflect_is_extensible_builtin(),
            None,
        ),
        reflect_own_keys: install_public_builtin_function(
            agent,
            cx,
            js3_reflect_own_keys_builtin(),
            None,
        ),
        reflect_prevent_extensions: install_public_builtin_function(
            agent,
            cx,
            js3_reflect_prevent_extensions_builtin(),
            None,
        ),
        reflect_set: install_public_builtin_function(agent, cx, js3_reflect_set_builtin(), None),
        reflect_set_prototype_of: install_public_builtin_function(
            agent,
            cx,
            js3_reflect_set_prototype_of_builtin(),
            None,
        ),
        proxy: install_public_builtin_function(agent, cx, js3_proxy_builtin(), None),
        proxy_revocable: install_public_builtin_function(
            agent,
            cx,
            js3_proxy_revocable_builtin(),
            None,
        ),
    }
}

pub(in crate::public) fn object_reflection_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (js3_reflect_apply_builtin(), builtins.reflect_apply),
        (js3_reflect_construct_builtin(), builtins.reflect_construct),
        (
            js3_reflect_define_property_builtin(),
            builtins.reflect_define_property,
        ),
        (
            js3_reflect_delete_property_builtin(),
            builtins.reflect_delete_property,
        ),
        (js3_reflect_get_builtin(), builtins.reflect_get),
        (
            js3_reflect_get_own_property_descriptor_builtin(),
            builtins.reflect_get_own_property_descriptor,
        ),
        (
            js3_reflect_get_prototype_of_builtin(),
            builtins.reflect_get_prototype_of,
        ),
        (js3_reflect_has_builtin(), builtins.reflect_has),
        (
            js3_reflect_is_extensible_builtin(),
            builtins.reflect_is_extensible,
        ),
        (js3_reflect_own_keys_builtin(), builtins.reflect_own_keys),
        (
            js3_reflect_prevent_extensions_builtin(),
            builtins.reflect_prevent_extensions,
        ),
        (js3_reflect_set_builtin(), builtins.reflect_set),
        (
            js3_reflect_set_prototype_of_builtin(),
            builtins.reflect_set_prototype_of,
        ),
        (js3_proxy_builtin(), builtins.proxy),
        (js3_proxy_revocable_builtin(), builtins.proxy_revocable),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

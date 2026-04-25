use super::{install_public_builtin_function, FamilyInstallContext, ObjectFamilyBuiltins};
use crate::public::PublicRealmBuiltins;
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_object_assign_builtin, js3_object_builtin, js3_object_create_builtin,
    js3_object_define_getter_builtin, js3_object_define_properties_builtin,
    js3_object_define_property_builtin, js3_object_define_setter_builtin,
    js3_object_entries_builtin, js3_object_freeze_builtin, js3_object_from_entries_builtin,
    js3_object_get_own_property_descriptor_builtin,
    js3_object_get_own_property_descriptors_builtin, js3_object_get_own_property_names_builtin,
    js3_object_get_own_property_symbols_builtin, js3_object_get_prototype_of_builtin,
    js3_object_group_by_builtin, js3_object_has_own_builtin, js3_object_has_own_property_builtin,
    js3_object_is_builtin, js3_object_is_extensible_builtin, js3_object_is_frozen_builtin,
    js3_object_is_prototype_of_builtin, js3_object_is_sealed_builtin, js3_object_keys_builtin,
    js3_object_lookup_getter_builtin, js3_object_lookup_setter_builtin,
    js3_object_prevent_extensions_builtin, js3_object_property_is_enumerable_builtin,
    js3_object_proto_getter_builtin, js3_object_proto_setter_builtin, js3_object_seal_builtin,
    js3_object_set_prototype_of_builtin, js3_object_to_locale_string_builtin,
    js3_object_to_string_builtin, js3_object_value_of_builtin, js3_object_values_builtin,
    BuiltinFunctionId, ObjectRef,
};

#[allow(clippy::too_many_lines)]
pub(in crate::public) fn install_object_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
) -> ObjectFamilyBuiltins {
    ObjectFamilyBuiltins {
        object: install_public_builtin_function(
            agent,
            cx,
            js3_object_builtin(),
            Some(cx.object_prototype),
        ),
        object_prototype: cx.object_prototype,
        object_create: install_public_builtin_function(
            agent,
            cx,
            js3_object_create_builtin(),
            None,
        ),
        object_get_prototype_of: install_public_builtin_function(
            agent,
            cx,
            js3_object_get_prototype_of_builtin(),
            None,
        ),
        object_set_prototype_of: install_public_builtin_function(
            agent,
            cx,
            js3_object_set_prototype_of_builtin(),
            None,
        ),
        object_get_own_property_descriptor: install_public_builtin_function(
            agent,
            cx,
            js3_object_get_own_property_descriptor_builtin(),
            None,
        ),
        object_get_own_property_descriptors: install_public_builtin_function(
            agent,
            cx,
            js3_object_get_own_property_descriptors_builtin(),
            None,
        ),
        object_get_own_property_names: install_public_builtin_function(
            agent,
            cx,
            js3_object_get_own_property_names_builtin(),
            None,
        ),
        object_get_own_property_symbols: install_public_builtin_function(
            agent,
            cx,
            js3_object_get_own_property_symbols_builtin(),
            None,
        ),
        object_define_properties: install_public_builtin_function(
            agent,
            cx,
            js3_object_define_properties_builtin(),
            None,
        ),
        object_define_property: install_public_builtin_function(
            agent,
            cx,
            js3_object_define_property_builtin(),
            None,
        ),
        object_assign: install_public_builtin_function(
            agent,
            cx,
            js3_object_assign_builtin(),
            None,
        ),
        object_from_entries: install_public_builtin_function(
            agent,
            cx,
            js3_object_from_entries_builtin(),
            None,
        ),
        object_group_by: install_public_builtin_function(
            agent,
            cx,
            js3_object_group_by_builtin(),
            None,
        ),
        object_prevent_extensions: install_public_builtin_function(
            agent,
            cx,
            js3_object_prevent_extensions_builtin(),
            None,
        ),
        object_is_extensible: install_public_builtin_function(
            agent,
            cx,
            js3_object_is_extensible_builtin(),
            None,
        ),
        object_is: install_public_builtin_function(agent, cx, js3_object_is_builtin(), None),
        object_seal: install_public_builtin_function(agent, cx, js3_object_seal_builtin(), None),
        object_freeze: install_public_builtin_function(
            agent,
            cx,
            js3_object_freeze_builtin(),
            None,
        ),
        object_is_sealed: install_public_builtin_function(
            agent,
            cx,
            js3_object_is_sealed_builtin(),
            None,
        ),
        object_is_frozen: install_public_builtin_function(
            agent,
            cx,
            js3_object_is_frozen_builtin(),
            None,
        ),
        object_to_locale_string: install_public_builtin_function(
            agent,
            cx,
            js3_object_to_locale_string_builtin(),
            None,
        ),
        object_to_string: install_public_builtin_function(
            agent,
            cx,
            js3_object_to_string_builtin(),
            None,
        ),
        object_value_of: install_public_builtin_function(
            agent,
            cx,
            js3_object_value_of_builtin(),
            None,
        ),
        object_has_own_property: install_public_builtin_function(
            agent,
            cx,
            js3_object_has_own_property_builtin(),
            None,
        ),
        object_is_prototype_of: install_public_builtin_function(
            agent,
            cx,
            js3_object_is_prototype_of_builtin(),
            None,
        ),
        object_property_is_enumerable: install_public_builtin_function(
            agent,
            cx,
            js3_object_property_is_enumerable_builtin(),
            None,
        ),
        object_define_getter: install_public_builtin_function(
            agent,
            cx,
            js3_object_define_getter_builtin(),
            None,
        ),
        object_define_setter: install_public_builtin_function(
            agent,
            cx,
            js3_object_define_setter_builtin(),
            None,
        ),
        object_lookup_getter: install_public_builtin_function(
            agent,
            cx,
            js3_object_lookup_getter_builtin(),
            None,
        ),
        object_lookup_setter: install_public_builtin_function(
            agent,
            cx,
            js3_object_lookup_setter_builtin(),
            None,
        ),
        object_proto_getter: install_public_builtin_function(
            agent,
            cx,
            js3_object_proto_getter_builtin(),
            None,
        ),
        object_proto_setter: install_public_builtin_function(
            agent,
            cx,
            js3_object_proto_setter_builtin(),
            None,
        ),
        object_keys: install_public_builtin_function(agent, cx, js3_object_keys_builtin(), None),
        object_entries: install_public_builtin_function(
            agent,
            cx,
            js3_object_entries_builtin(),
            None,
        ),
        object_values: install_public_builtin_function(
            agent,
            cx,
            js3_object_values_builtin(),
            None,
        ),
        object_has_own: install_public_builtin_function(
            agent,
            cx,
            js3_object_has_own_builtin(),
            None,
        ),
    }
}

pub(in crate::public) fn object_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    object_static_builtin_object(builtins, entry)
        .or_else(|| object_prototype_builtin_object(builtins, entry))
}

fn object_static_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (js3_object_builtin(), builtins.object),
        (js3_object_create_builtin(), builtins.object_create),
        (
            js3_object_get_prototype_of_builtin(),
            builtins.object_get_prototype_of,
        ),
        (
            js3_object_set_prototype_of_builtin(),
            builtins.object_set_prototype_of,
        ),
        (
            js3_object_get_own_property_descriptor_builtin(),
            builtins.object_get_own_property_descriptor,
        ),
        (
            js3_object_get_own_property_descriptors_builtin(),
            builtins.object_get_own_property_descriptors,
        ),
        (
            js3_object_get_own_property_names_builtin(),
            builtins.object_get_own_property_names,
        ),
        (
            js3_object_get_own_property_symbols_builtin(),
            builtins.object_get_own_property_symbols,
        ),
        (
            js3_object_define_properties_builtin(),
            builtins.object_define_properties,
        ),
        (
            js3_object_define_property_builtin(),
            builtins.object_define_property,
        ),
        (js3_object_assign_builtin(), builtins.object_assign),
        (
            js3_object_from_entries_builtin(),
            builtins.object_from_entries,
        ),
        (js3_object_group_by_builtin(), builtins.object_group_by),
        (
            js3_object_prevent_extensions_builtin(),
            builtins.object_prevent_extensions,
        ),
        (
            js3_object_is_extensible_builtin(),
            builtins.object_is_extensible,
        ),
        (js3_object_is_builtin(), builtins.object_is),
        (js3_object_seal_builtin(), builtins.object_seal),
        (js3_object_freeze_builtin(), builtins.object_freeze),
        (js3_object_is_sealed_builtin(), builtins.object_is_sealed),
        (js3_object_is_frozen_builtin(), builtins.object_is_frozen),
        (js3_object_keys_builtin(), builtins.object_keys),
        (js3_object_entries_builtin(), builtins.object_entries),
        (js3_object_values_builtin(), builtins.object_values),
        (js3_object_has_own_builtin(), builtins.object_has_own),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

fn object_prototype_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (
            js3_object_to_locale_string_builtin(),
            builtins.object_to_locale_string,
        ),
        (js3_object_to_string_builtin(), builtins.object_to_string),
        (js3_object_value_of_builtin(), builtins.object_value_of),
        (
            js3_object_has_own_property_builtin(),
            builtins.object_has_own_property,
        ),
        (
            js3_object_is_prototype_of_builtin(),
            builtins.object_is_prototype_of,
        ),
        (
            js3_object_property_is_enumerable_builtin(),
            builtins.object_property_is_enumerable,
        ),
        (
            js3_object_define_getter_builtin(),
            builtins.object_define_getter,
        ),
        (
            js3_object_define_setter_builtin(),
            builtins.object_define_setter,
        ),
        (
            js3_object_lookup_getter_builtin(),
            builtins.object_lookup_getter,
        ),
        (
            js3_object_lookup_setter_builtin(),
            builtins.object_lookup_setter,
        ),
        (
            js3_object_proto_getter_builtin(),
            builtins.object_proto_getter,
        ),
        (
            js3_object_proto_setter_builtin(),
            builtins.object_proto_setter,
        ),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

use super::{
    install_public_builtin_function, FamilyInstallContext, ModuleFamilyBuiltins,
    ModuleFamilyPrototypes,
};
use crate::public::{
    define_builtin_accessor_property, define_builtin_data_property, PublicRealmBuiltins,
};
use lyng_js_common::WellKnownAtom;
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_abstract_module_source_builtin, js3_abstract_module_source_to_string_tag_getter_builtin,
    BuiltinFunctionId, ObjectRef, PropertyKey, Value, WellKnownSymbolId,
};

pub(in crate::public) fn install_module_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototypes: ModuleFamilyPrototypes,
) -> ModuleFamilyBuiltins {
    ModuleFamilyBuiltins {
        abstract_module_source: install_public_builtin_function(
            agent,
            cx,
            js3_abstract_module_source_builtin(),
            Some(prototypes.abstract_module_source_prototype),
        ),
        abstract_module_source_prototype: prototypes.abstract_module_source_prototype,
        abstract_module_source_to_string_tag_getter: install_public_builtin_function(
            agent,
            cx,
            js3_abstract_module_source_to_string_tag_getter_builtin(),
            None,
        ),
    }
}

pub(in crate::public) fn install_module_family_descriptors(
    agent: &mut Agent,
    builtins: &PublicRealmBuiltins,
) {
    define_builtin_data_property(
        agent,
        builtins.abstract_module_source_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        Value::from_object_ref(builtins.abstract_module_source),
        true,
        false,
        true,
    );
    if let Some(to_string_tag) = agent.well_known_symbol(WellKnownSymbolId::ToStringTag) {
        define_builtin_accessor_property(
            agent,
            builtins.abstract_module_source_prototype,
            PropertyKey::from_symbol(to_string_tag),
            Some(builtins.abstract_module_source_to_string_tag_getter),
            None,
            false,
            true,
        );
    }
}

pub(in crate::public) fn module_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (
            js3_abstract_module_source_builtin(),
            builtins.abstract_module_source,
        ),
        (
            js3_abstract_module_source_to_string_tag_getter_builtin(),
            builtins.abstract_module_source_to_string_tag_getter,
        ),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

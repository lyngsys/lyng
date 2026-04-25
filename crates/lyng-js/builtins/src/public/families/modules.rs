use super::{
    install_public_builtin_function, FamilyInstallContext, ModuleFamilyBuiltins,
    ModuleFamilyPrototypes,
};
use crate::public::PublicRealmBuiltins;
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_abstract_module_source_builtin, js3_abstract_module_source_to_string_tag_getter_builtin,
    BuiltinFunctionId, ObjectRef,
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

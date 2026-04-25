use super::descriptors::{
    builtin_function_atom_property, data_symbol_property, descriptor_tag,
    readonly_builtin_attributes,
};
use super::{
    install_public_builtin_function, FamilyInstallContext, ObjectReflectionFamilyBuiltins,
    ObjectReflectionFamilyObjects,
};
use crate::bootstrap::{install_descriptor_tables, BuiltinBootstrapError};
use crate::public::{BuiltinCache, PublicRealmBuiltins};
use crate::{BuiltinDescriptorTable, BuiltinInstallTarget, BuiltinIntrinsic};
use lyng_js_common::AtomId;
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_proxy_builtin, js3_proxy_revocable_builtin, js3_reflect_apply_builtin,
    js3_reflect_construct_builtin, js3_reflect_define_property_builtin,
    js3_reflect_delete_property_builtin, js3_reflect_get_builtin,
    js3_reflect_get_own_property_descriptor_builtin, js3_reflect_get_prototype_of_builtin,
    js3_reflect_has_builtin, js3_reflect_is_extensible_builtin, js3_reflect_own_keys_builtin,
    js3_reflect_prevent_extensions_builtin, js3_reflect_set_builtin,
    js3_reflect_set_prototype_of_builtin, BuiltinFunctionId, ObjectRef, RealmRef,
    WellKnownSymbolId,
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

pub(in crate::public) fn install_object_reflection_family_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
) -> Result<(), BuiltinBootstrapError> {
    let atoms = ObjectReflectionDescriptorAtoms::new(agent);
    let reflect_tag = descriptor_tag(agent, "Reflect");

    let reflect_descriptors = [
        builtin_function_atom_property(atoms.apply, js3_reflect_apply_builtin()),
        builtin_function_atom_property(atoms.construct, js3_reflect_construct_builtin()),
        builtin_function_atom_property(
            atoms.define_property,
            js3_reflect_define_property_builtin(),
        ),
        builtin_function_atom_property(
            atoms.delete_property,
            js3_reflect_delete_property_builtin(),
        ),
        builtin_function_atom_property(atoms.get, js3_reflect_get_builtin()),
        builtin_function_atom_property(
            atoms.get_own_property_descriptor,
            js3_reflect_get_own_property_descriptor_builtin(),
        ),
        builtin_function_atom_property(
            atoms.get_prototype_of,
            js3_reflect_get_prototype_of_builtin(),
        ),
        builtin_function_atom_property(atoms.has, js3_reflect_has_builtin()),
        builtin_function_atom_property(atoms.is_extensible, js3_reflect_is_extensible_builtin()),
        builtin_function_atom_property(atoms.own_keys, js3_reflect_own_keys_builtin()),
        builtin_function_atom_property(
            atoms.prevent_extensions,
            js3_reflect_prevent_extensions_builtin(),
        ),
        builtin_function_atom_property(atoms.set, js3_reflect_set_builtin()),
        builtin_function_atom_property(
            atoms.set_prototype_of,
            js3_reflect_set_prototype_of_builtin(),
        ),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            reflect_tag,
            readonly_builtin_attributes(),
        ),
    ];
    let proxy_descriptors = [builtin_function_atom_property(
        atoms.revocable,
        js3_proxy_revocable_builtin(),
    )];
    let tables = [
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Reflect),
            &reflect_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Proxy),
            &proxy_descriptors,
        ),
    ];
    install_descriptor_tables(agent, cache, realm, &tables)
}

#[derive(Clone, Copy)]
struct ObjectReflectionDescriptorAtoms {
    apply: AtomId,
    construct: AtomId,
    define_property: AtomId,
    delete_property: AtomId,
    get: AtomId,
    get_own_property_descriptor: AtomId,
    get_prototype_of: AtomId,
    has: AtomId,
    is_extensible: AtomId,
    own_keys: AtomId,
    prevent_extensions: AtomId,
    revocable: AtomId,
    set: AtomId,
    set_prototype_of: AtomId,
}

impl ObjectReflectionDescriptorAtoms {
    fn new(agent: &mut Agent) -> Self {
        Self {
            apply: agent.atoms_mut().intern_collectible("apply"),
            construct: agent.atoms_mut().intern_collectible("construct"),
            define_property: agent.atoms_mut().intern_collectible("defineProperty"),
            delete_property: agent.atoms_mut().intern_collectible("deleteProperty"),
            get: agent.atoms_mut().intern_collectible("get"),
            get_own_property_descriptor: agent
                .atoms_mut()
                .intern_collectible("getOwnPropertyDescriptor"),
            get_prototype_of: agent.atoms_mut().intern_collectible("getPrototypeOf"),
            has: agent.atoms_mut().intern_collectible("has"),
            is_extensible: agent.atoms_mut().intern_collectible("isExtensible"),
            own_keys: agent.atoms_mut().intern_collectible("ownKeys"),
            prevent_extensions: agent.atoms_mut().intern_collectible("preventExtensions"),
            revocable: agent.atoms_mut().intern_collectible("revocable"),
            set: agent.atoms_mut().intern_collectible("set"),
            set_prototype_of: agent.atoms_mut().intern_collectible("setPrototypeOf"),
        }
    }
}

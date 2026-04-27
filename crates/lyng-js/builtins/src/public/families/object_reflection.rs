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
    proxy_builtin, proxy_revocable_builtin, reflect_apply_builtin, reflect_construct_builtin,
    reflect_define_property_builtin, reflect_delete_property_builtin, reflect_get_builtin,
    reflect_get_own_property_descriptor_builtin, reflect_get_prototype_of_builtin,
    reflect_has_builtin, reflect_is_extensible_builtin, reflect_own_keys_builtin,
    reflect_prevent_extensions_builtin, reflect_set_builtin, reflect_set_prototype_of_builtin,
    BuiltinFunctionId, ObjectRef, RealmRef, WellKnownSymbolId,
};

pub(in crate::public) fn install_object_reflection_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    objects: ObjectReflectionFamilyObjects,
) -> ObjectReflectionFamilyBuiltins {
    ObjectReflectionFamilyBuiltins {
        reflect: objects.reflect,
        reflect_apply: install_public_builtin_function(agent, cx, reflect_apply_builtin(), None),
        reflect_construct: install_public_builtin_function(
            agent,
            cx,
            reflect_construct_builtin(),
            None,
        ),
        reflect_define_property: install_public_builtin_function(
            agent,
            cx,
            reflect_define_property_builtin(),
            None,
        ),
        reflect_delete_property: install_public_builtin_function(
            agent,
            cx,
            reflect_delete_property_builtin(),
            None,
        ),
        reflect_get: install_public_builtin_function(agent, cx, reflect_get_builtin(), None),
        reflect_get_own_property_descriptor: install_public_builtin_function(
            agent,
            cx,
            reflect_get_own_property_descriptor_builtin(),
            None,
        ),
        reflect_get_prototype_of: install_public_builtin_function(
            agent,
            cx,
            reflect_get_prototype_of_builtin(),
            None,
        ),
        reflect_has: install_public_builtin_function(agent, cx, reflect_has_builtin(), None),
        reflect_is_extensible: install_public_builtin_function(
            agent,
            cx,
            reflect_is_extensible_builtin(),
            None,
        ),
        reflect_own_keys: install_public_builtin_function(
            agent,
            cx,
            reflect_own_keys_builtin(),
            None,
        ),
        reflect_prevent_extensions: install_public_builtin_function(
            agent,
            cx,
            reflect_prevent_extensions_builtin(),
            None,
        ),
        reflect_set: install_public_builtin_function(agent, cx, reflect_set_builtin(), None),
        reflect_set_prototype_of: install_public_builtin_function(
            agent,
            cx,
            reflect_set_prototype_of_builtin(),
            None,
        ),
        proxy: install_public_builtin_function(agent, cx, proxy_builtin(), None),
        proxy_revocable: install_public_builtin_function(
            agent,
            cx,
            proxy_revocable_builtin(),
            None,
        ),
    }
}

pub(in crate::public) fn object_reflection_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (reflect_apply_builtin(), builtins.reflect_apply),
        (reflect_construct_builtin(), builtins.reflect_construct),
        (
            reflect_define_property_builtin(),
            builtins.reflect_define_property,
        ),
        (
            reflect_delete_property_builtin(),
            builtins.reflect_delete_property,
        ),
        (reflect_get_builtin(), builtins.reflect_get),
        (
            reflect_get_own_property_descriptor_builtin(),
            builtins.reflect_get_own_property_descriptor,
        ),
        (
            reflect_get_prototype_of_builtin(),
            builtins.reflect_get_prototype_of,
        ),
        (reflect_has_builtin(), builtins.reflect_has),
        (
            reflect_is_extensible_builtin(),
            builtins.reflect_is_extensible,
        ),
        (reflect_own_keys_builtin(), builtins.reflect_own_keys),
        (
            reflect_prevent_extensions_builtin(),
            builtins.reflect_prevent_extensions,
        ),
        (reflect_set_builtin(), builtins.reflect_set),
        (
            reflect_set_prototype_of_builtin(),
            builtins.reflect_set_prototype_of,
        ),
        (proxy_builtin(), builtins.proxy),
        (proxy_revocable_builtin(), builtins.proxy_revocable),
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
        builtin_function_atom_property(atoms.apply, reflect_apply_builtin()),
        builtin_function_atom_property(atoms.construct, reflect_construct_builtin()),
        builtin_function_atom_property(atoms.define_property, reflect_define_property_builtin()),
        builtin_function_atom_property(atoms.delete_property, reflect_delete_property_builtin()),
        builtin_function_atom_property(atoms.get, reflect_get_builtin()),
        builtin_function_atom_property(
            atoms.get_own_property_descriptor,
            reflect_get_own_property_descriptor_builtin(),
        ),
        builtin_function_atom_property(atoms.get_prototype_of, reflect_get_prototype_of_builtin()),
        builtin_function_atom_property(atoms.has, reflect_has_builtin()),
        builtin_function_atom_property(atoms.is_extensible, reflect_is_extensible_builtin()),
        builtin_function_atom_property(atoms.own_keys, reflect_own_keys_builtin()),
        builtin_function_atom_property(
            atoms.prevent_extensions,
            reflect_prevent_extensions_builtin(),
        ),
        builtin_function_atom_property(atoms.set, reflect_set_builtin()),
        builtin_function_atom_property(atoms.set_prototype_of, reflect_set_prototype_of_builtin()),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            reflect_tag,
            readonly_builtin_attributes(),
        ),
    ];
    let proxy_descriptors = [builtin_function_atom_property(
        atoms.revocable,
        proxy_revocable_builtin(),
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
            apply: agent.atoms_mut().intern("apply"),
            construct: agent.atoms_mut().intern("construct"),
            define_property: agent.atoms_mut().intern("defineProperty"),
            delete_property: agent.atoms_mut().intern("deleteProperty"),
            get: agent.atoms_mut().intern("get"),
            get_own_property_descriptor: agent.atoms_mut().intern("getOwnPropertyDescriptor"),
            get_prototype_of: agent.atoms_mut().intern("getPrototypeOf"),
            has: agent.atoms_mut().intern("has"),
            is_extensible: agent.atoms_mut().intern("isExtensible"),
            own_keys: agent.atoms_mut().intern("ownKeys"),
            prevent_extensions: agent.atoms_mut().intern("preventExtensions"),
            revocable: agent.atoms_mut().intern("revocable"),
            set: agent.atoms_mut().intern("set"),
            set_prototype_of: agent.atoms_mut().intern("setPrototypeOf"),
        }
    }
}

use super::descriptors::{
    accessor_atom_property, accessor_symbol_property, builtin_function_atom_property,
    builtin_function_symbol_property, data_atom_property, data_symbol_property,
    descriptor_tag_with_atom, readonly_builtin_attributes, writable_builtin_attributes,
};
use super::{
    install_public_builtin_function, CollectionFamilyBuiltins, CollectionFamilyPrototypes,
    FamilyInstallContext,
};
use crate::bootstrap::{install_descriptor_tables, BuiltinBootstrapError};
use crate::public::{BuiltinCache, PublicRealmBuiltins};
use crate::{BuiltinDescriptorTable, BuiltinInstallTarget, BuiltinIntrinsic};
use lyng_js_common::{AtomId, WellKnownAtom};
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_array_species_getter_builtin, js3_finalization_registry_builtin,
    js3_finalization_registry_register_builtin, js3_finalization_registry_unregister_builtin,
    js3_map_builtin, js3_map_clear_builtin, js3_map_delete_builtin, js3_map_entries_builtin,
    js3_map_for_each_builtin, js3_map_get_builtin, js3_map_has_builtin, js3_map_keys_builtin,
    js3_map_set_builtin, js3_map_size_getter_builtin, js3_map_values_builtin, js3_set_add_builtin,
    js3_set_builtin, js3_set_clear_builtin, js3_set_delete_builtin, js3_set_entries_builtin,
    js3_set_for_each_builtin, js3_set_has_builtin, js3_set_keys_builtin,
    js3_set_size_getter_builtin, js3_set_values_builtin, js3_weak_map_builtin,
    js3_weak_map_delete_builtin, js3_weak_map_get_builtin, js3_weak_map_has_builtin,
    js3_weak_map_set_builtin, js3_weak_ref_builtin, js3_weak_ref_deref_builtin,
    js3_weak_set_add_builtin, js3_weak_set_builtin, js3_weak_set_delete_builtin,
    js3_weak_set_has_builtin, BuiltinFunctionId, ObjectRef, RealmRef, Value, WellKnownSymbolId,
};

#[allow(clippy::too_many_lines)]
pub(in crate::public) fn install_collection_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototypes: CollectionFamilyPrototypes,
) -> CollectionFamilyBuiltins {
    CollectionFamilyBuiltins {
        map: install_public_builtin_function(
            agent,
            cx,
            js3_map_builtin(),
            Some(prototypes.map_prototype),
        ),
        set: install_public_builtin_function(
            agent,
            cx,
            js3_set_builtin(),
            Some(prototypes.set_prototype),
        ),
        weak_map: install_public_builtin_function(
            agent,
            cx,
            js3_weak_map_builtin(),
            Some(prototypes.weak_map_prototype),
        ),
        weak_set: install_public_builtin_function(
            agent,
            cx,
            js3_weak_set_builtin(),
            Some(prototypes.weak_set_prototype),
        ),
        weak_ref: install_public_builtin_function(
            agent,
            cx,
            js3_weak_ref_builtin(),
            Some(prototypes.weak_ref_prototype),
        ),
        finalization_registry: install_public_builtin_function(
            agent,
            cx,
            js3_finalization_registry_builtin(),
            Some(prototypes.finalization_registry_prototype),
        ),
        map_get: install_public_builtin_function(agent, cx, js3_map_get_builtin(), None),
        map_set: install_public_builtin_function(agent, cx, js3_map_set_builtin(), None),
        map_has: install_public_builtin_function(agent, cx, js3_map_has_builtin(), None),
        map_delete: install_public_builtin_function(agent, cx, js3_map_delete_builtin(), None),
        map_clear: install_public_builtin_function(agent, cx, js3_map_clear_builtin(), None),
        map_entries: install_public_builtin_function(agent, cx, js3_map_entries_builtin(), None),
        map_values: install_public_builtin_function(agent, cx, js3_map_values_builtin(), None),
        map_keys: install_public_builtin_function(agent, cx, js3_map_keys_builtin(), None),
        map_for_each: install_public_builtin_function(agent, cx, js3_map_for_each_builtin(), None),
        map_size_getter: install_public_builtin_function(
            agent,
            cx,
            js3_map_size_getter_builtin(),
            None,
        ),
        set_add: install_public_builtin_function(agent, cx, js3_set_add_builtin(), None),
        set_has: install_public_builtin_function(agent, cx, js3_set_has_builtin(), None),
        set_delete: install_public_builtin_function(agent, cx, js3_set_delete_builtin(), None),
        set_clear: install_public_builtin_function(agent, cx, js3_set_clear_builtin(), None),
        set_entries: install_public_builtin_function(agent, cx, js3_set_entries_builtin(), None),
        set_values: install_public_builtin_function(agent, cx, js3_set_values_builtin(), None),
        set_keys: install_public_builtin_function(agent, cx, js3_set_keys_builtin(), None),
        set_for_each: install_public_builtin_function(agent, cx, js3_set_for_each_builtin(), None),
        set_size_getter: install_public_builtin_function(
            agent,
            cx,
            js3_set_size_getter_builtin(),
            None,
        ),
        weak_map_get: install_public_builtin_function(agent, cx, js3_weak_map_get_builtin(), None),
        weak_map_set: install_public_builtin_function(agent, cx, js3_weak_map_set_builtin(), None),
        weak_map_has: install_public_builtin_function(agent, cx, js3_weak_map_has_builtin(), None),
        weak_map_delete: install_public_builtin_function(
            agent,
            cx,
            js3_weak_map_delete_builtin(),
            None,
        ),
        weak_set_add: install_public_builtin_function(agent, cx, js3_weak_set_add_builtin(), None),
        weak_set_has: install_public_builtin_function(agent, cx, js3_weak_set_has_builtin(), None),
        weak_set_delete: install_public_builtin_function(
            agent,
            cx,
            js3_weak_set_delete_builtin(),
            None,
        ),
        weak_ref_deref: install_public_builtin_function(
            agent,
            cx,
            js3_weak_ref_deref_builtin(),
            None,
        ),
        finalization_registry_register: install_public_builtin_function(
            agent,
            cx,
            js3_finalization_registry_register_builtin(),
            None,
        ),
        finalization_registry_unregister: install_public_builtin_function(
            agent,
            cx,
            js3_finalization_registry_unregister_builtin(),
            None,
        ),
        map_prototype: prototypes.map_prototype,
        set_prototype: prototypes.set_prototype,
        weak_map_prototype: prototypes.weak_map_prototype,
        weak_set_prototype: prototypes.weak_set_prototype,
        weak_ref_prototype: prototypes.weak_ref_prototype,
        finalization_registry_prototype: prototypes.finalization_registry_prototype,
    }
}

pub(in crate::public) fn install_collection_family_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
) -> Result<(), BuiltinBootstrapError> {
    let atoms = CollectionDescriptorAtoms::new(agent);
    let tags = CollectionDescriptorTags::new(agent);
    install_map_constructor_descriptors(agent, cache, realm)?;
    install_map_prototype_descriptors(agent, cache, realm, builtins, &atoms, &tags)?;
    install_set_constructor_descriptors(agent, cache, realm)?;
    install_set_prototype_descriptors(agent, cache, realm, builtins, &atoms, &tags)?;
    install_weak_map_prototype_descriptors(agent, cache, realm, builtins, &atoms, &tags)?;
    install_weak_set_prototype_descriptors(agent, cache, realm, builtins, &atoms, &tags)?;
    install_weak_ref_prototype_descriptors(agent, cache, realm, builtins, &atoms, &tags)?;
    install_finalization_registry_prototype_descriptors(
        agent, cache, realm, builtins, &atoms, &tags,
    )
}

fn install_map_constructor_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [accessor_symbol_property(
        WellKnownSymbolId::Species,
        Some(js3_array_species_getter_builtin()),
        None,
        readonly_builtin_attributes(),
    )];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Map),
            &descriptors,
        )],
    )
}

fn install_map_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
    atoms: &CollectionDescriptorAtoms,
    tags: &CollectionDescriptorTags,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(builtins.map),
            writable_builtin_attributes(),
        ),
        builtin_function_atom_property(atoms.get, js3_map_get_builtin()),
        builtin_function_atom_property(atoms.set, js3_map_set_builtin()),
        builtin_function_atom_property(atoms.has, js3_map_has_builtin()),
        builtin_function_atom_property(atoms.delete, js3_map_delete_builtin()),
        builtin_function_atom_property(atoms.clear, js3_map_clear_builtin()),
        builtin_function_atom_property(atoms.entries, js3_map_entries_builtin()),
        builtin_function_atom_property(atoms.values, js3_map_values_builtin()),
        builtin_function_atom_property(atoms.keys, js3_map_keys_builtin()),
        builtin_function_atom_property(atoms.for_each, js3_map_for_each_builtin()),
        accessor_atom_property(
            atoms.size,
            Some(js3_map_size_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        builtin_function_symbol_property(
            WellKnownSymbolId::Iterator,
            js3_map_entries_builtin(),
            writable_builtin_attributes(),
        ),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            tags.map,
            readonly_builtin_attributes(),
        ),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::MapPrototype),
            &descriptors,
        )],
    )
}

fn install_set_constructor_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [accessor_symbol_property(
        WellKnownSymbolId::Species,
        Some(js3_array_species_getter_builtin()),
        None,
        readonly_builtin_attributes(),
    )];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Set),
            &descriptors,
        )],
    )
}

fn install_set_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
    atoms: &CollectionDescriptorAtoms,
    tags: &CollectionDescriptorTags,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(builtins.set),
            writable_builtin_attributes(),
        ),
        builtin_function_atom_property(atoms.add, js3_set_add_builtin()),
        builtin_function_atom_property(atoms.has, js3_set_has_builtin()),
        builtin_function_atom_property(atoms.delete, js3_set_delete_builtin()),
        builtin_function_atom_property(atoms.clear, js3_set_clear_builtin()),
        builtin_function_atom_property(atoms.entries, js3_set_entries_builtin()),
        builtin_function_atom_property(atoms.values, js3_set_values_builtin()),
        data_atom_property(
            atoms.keys,
            Value::from_object_ref(builtins.set_values),
            writable_builtin_attributes(),
        ),
        builtin_function_atom_property(atoms.for_each, js3_set_for_each_builtin()),
        accessor_atom_property(
            atoms.size,
            Some(js3_set_size_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        builtin_function_symbol_property(
            WellKnownSymbolId::Iterator,
            js3_set_values_builtin(),
            writable_builtin_attributes(),
        ),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            tags.set,
            readonly_builtin_attributes(),
        ),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::SetPrototype),
            &descriptors,
        )],
    )
}

fn install_weak_map_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
    atoms: &CollectionDescriptorAtoms,
    tags: &CollectionDescriptorTags,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(builtins.weak_map),
            writable_builtin_attributes(),
        ),
        builtin_function_atom_property(atoms.get, js3_weak_map_get_builtin()),
        builtin_function_atom_property(atoms.set, js3_weak_map_set_builtin()),
        builtin_function_atom_property(atoms.has, js3_weak_map_has_builtin()),
        builtin_function_atom_property(atoms.delete, js3_weak_map_delete_builtin()),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            tags.weak_map,
            readonly_builtin_attributes(),
        ),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::WeakMapPrototype),
            &descriptors,
        )],
    )
}

fn install_weak_set_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
    atoms: &CollectionDescriptorAtoms,
    tags: &CollectionDescriptorTags,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(builtins.weak_set),
            writable_builtin_attributes(),
        ),
        builtin_function_atom_property(atoms.add, js3_weak_set_add_builtin()),
        builtin_function_atom_property(atoms.has, js3_weak_set_has_builtin()),
        builtin_function_atom_property(atoms.delete, js3_weak_set_delete_builtin()),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            tags.weak_set,
            readonly_builtin_attributes(),
        ),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::WeakSetPrototype),
            &descriptors,
        )],
    )
}

fn install_weak_ref_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
    atoms: &CollectionDescriptorAtoms,
    tags: &CollectionDescriptorTags,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(builtins.weak_ref),
            writable_builtin_attributes(),
        ),
        builtin_function_atom_property(atoms.deref, js3_weak_ref_deref_builtin()),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            tags.weak_ref,
            readonly_builtin_attributes(),
        ),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::WeakRefPrototype),
            &descriptors,
        )],
    )
}

fn install_finalization_registry_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
    atoms: &CollectionDescriptorAtoms,
    tags: &CollectionDescriptorTags,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(builtins.finalization_registry),
            writable_builtin_attributes(),
        ),
        builtin_function_atom_property(
            atoms.register,
            js3_finalization_registry_register_builtin(),
        ),
        builtin_function_atom_property(
            atoms.unregister,
            js3_finalization_registry_unregister_builtin(),
        ),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            tags.finalization_registry,
            readonly_builtin_attributes(),
        ),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::FinalizationRegistryPrototype),
            &descriptors,
        )],
    )
}

#[derive(Clone, Copy, Debug)]
struct CollectionDescriptorAtoms {
    add: AtomId,
    clear: AtomId,
    delete: AtomId,
    deref: AtomId,
    entries: AtomId,
    for_each: AtomId,
    get: AtomId,
    has: AtomId,
    keys: AtomId,
    register: AtomId,
    set: AtomId,
    size: AtomId,
    unregister: AtomId,
    values: AtomId,
}

impl CollectionDescriptorAtoms {
    fn new(agent: &mut Agent) -> Self {
        let atoms = agent.atoms_mut();
        Self {
            add: atoms.intern_collectible("add"),
            clear: atoms.intern_collectible("clear"),
            delete: atoms.intern_collectible("delete"),
            deref: atoms.intern_collectible("deref"),
            entries: atoms.intern_collectible("entries"),
            for_each: atoms.intern_collectible("forEach"),
            get: atoms.intern_collectible("get"),
            has: atoms.intern_collectible("has"),
            keys: atoms.intern_collectible("keys"),
            register: atoms.intern_collectible("register"),
            set: atoms.intern_collectible("set"),
            size: atoms.intern_collectible("size"),
            unregister: atoms.intern_collectible("unregister"),
            values: atoms.intern_collectible("values"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct CollectionDescriptorTags {
    finalization_registry: Value,
    map: Value,
    set: Value,
    weak_map: Value,
    weak_ref: Value,
    weak_set: Value,
}

impl CollectionDescriptorTags {
    fn new(agent: &mut Agent) -> Self {
        let bootstrap_atoms = agent.bootstrap_atoms();
        Self {
            finalization_registry: descriptor_tag_with_atom(
                agent,
                "FinalizationRegistry",
                bootstrap_atoms.finalization_registry(),
            ),
            map: descriptor_tag_with_atom(agent, "Map", bootstrap_atoms.map()),
            set: descriptor_tag_with_atom(agent, "Set", bootstrap_atoms.set()),
            weak_map: descriptor_tag_with_atom(agent, "WeakMap", bootstrap_atoms.weak_map()),
            weak_ref: descriptor_tag_with_atom(agent, "WeakRef", bootstrap_atoms.weak_ref()),
            weak_set: descriptor_tag_with_atom(agent, "WeakSet", bootstrap_atoms.weak_set()),
        }
    }
}

pub(in crate::public) fn collection_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (js3_map_builtin(), builtins.map),
        (js3_set_builtin(), builtins.set),
        (js3_weak_map_builtin(), builtins.weak_map),
        (js3_weak_set_builtin(), builtins.weak_set),
        (js3_weak_ref_builtin(), builtins.weak_ref),
        (
            js3_finalization_registry_builtin(),
            builtins.finalization_registry,
        ),
        (js3_map_get_builtin(), builtins.map_get),
        (js3_map_set_builtin(), builtins.map_set),
        (js3_map_has_builtin(), builtins.map_has),
        (js3_map_delete_builtin(), builtins.map_delete),
        (js3_map_clear_builtin(), builtins.map_clear),
        (js3_map_entries_builtin(), builtins.map_entries),
        (js3_map_values_builtin(), builtins.map_values),
        (js3_map_keys_builtin(), builtins.map_keys),
        (js3_map_for_each_builtin(), builtins.map_for_each),
        (js3_map_size_getter_builtin(), builtins.map_size_getter),
        (js3_set_add_builtin(), builtins.set_add),
        (js3_set_has_builtin(), builtins.set_has),
        (js3_set_delete_builtin(), builtins.set_delete),
        (js3_set_clear_builtin(), builtins.set_clear),
        (js3_set_entries_builtin(), builtins.set_entries),
        (js3_set_values_builtin(), builtins.set_values),
        (js3_set_keys_builtin(), builtins.set_keys),
        (js3_set_for_each_builtin(), builtins.set_for_each),
        (js3_set_size_getter_builtin(), builtins.set_size_getter),
        (js3_weak_map_get_builtin(), builtins.weak_map_get),
        (js3_weak_map_set_builtin(), builtins.weak_map_set),
        (js3_weak_map_has_builtin(), builtins.weak_map_has),
        (js3_weak_map_delete_builtin(), builtins.weak_map_delete),
        (js3_weak_set_add_builtin(), builtins.weak_set_add),
        (js3_weak_set_has_builtin(), builtins.weak_set_has),
        (js3_weak_set_delete_builtin(), builtins.weak_set_delete),
        (js3_weak_ref_deref_builtin(), builtins.weak_ref_deref),
        (
            js3_finalization_registry_register_builtin(),
            builtins.finalization_registry_register,
        ),
        (
            js3_finalization_registry_unregister_builtin(),
            builtins.finalization_registry_unregister,
        ),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

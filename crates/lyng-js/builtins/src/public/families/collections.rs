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
    array_species_getter_builtin, finalization_registry_builtin,
    finalization_registry_register_builtin, finalization_registry_unregister_builtin, map_builtin,
    map_clear_builtin, map_delete_builtin, map_entries_builtin, map_for_each_builtin,
    map_get_builtin, map_get_or_insert_builtin, map_get_or_insert_computed_builtin,
    map_has_builtin, map_keys_builtin, map_set_builtin, map_size_getter_builtin,
    map_values_builtin, set_add_builtin, set_builtin, set_clear_builtin, set_delete_builtin,
    set_difference_builtin, set_entries_builtin, set_for_each_builtin, set_has_builtin,
    set_intersection_builtin, set_is_disjoint_from_builtin, set_is_subset_of_builtin,
    set_is_superset_of_builtin, set_keys_builtin, set_size_getter_builtin,
    set_symmetric_difference_builtin, set_union_builtin, set_values_builtin, weak_map_builtin,
    weak_map_delete_builtin, weak_map_get_builtin, weak_map_get_or_insert_builtin,
    weak_map_get_or_insert_computed_builtin, weak_map_has_builtin, weak_map_set_builtin,
    weak_ref_builtin, weak_ref_deref_builtin, weak_set_add_builtin, weak_set_builtin,
    weak_set_delete_builtin, weak_set_has_builtin, BuiltinFunctionId, ObjectRef, RealmRef, Value,
    WellKnownSymbolId,
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
            map_builtin(),
            Some(prototypes.map_prototype),
        ),
        set: install_public_builtin_function(
            agent,
            cx,
            set_builtin(),
            Some(prototypes.set_prototype),
        ),
        weak_map: install_public_builtin_function(
            agent,
            cx,
            weak_map_builtin(),
            Some(prototypes.weak_map_prototype),
        ),
        weak_set: install_public_builtin_function(
            agent,
            cx,
            weak_set_builtin(),
            Some(prototypes.weak_set_prototype),
        ),
        weak_ref: install_public_builtin_function(
            agent,
            cx,
            weak_ref_builtin(),
            Some(prototypes.weak_ref_prototype),
        ),
        finalization_registry: install_public_builtin_function(
            agent,
            cx,
            finalization_registry_builtin(),
            Some(prototypes.finalization_registry_prototype),
        ),
        map_get: install_public_builtin_function(agent, cx, map_get_builtin(), None),
        map_set: install_public_builtin_function(agent, cx, map_set_builtin(), None),
        map_has: install_public_builtin_function(agent, cx, map_has_builtin(), None),
        map_delete: install_public_builtin_function(agent, cx, map_delete_builtin(), None),
        map_clear: install_public_builtin_function(agent, cx, map_clear_builtin(), None),
        map_entries: install_public_builtin_function(agent, cx, map_entries_builtin(), None),
        map_values: install_public_builtin_function(agent, cx, map_values_builtin(), None),
        map_keys: install_public_builtin_function(agent, cx, map_keys_builtin(), None),
        map_for_each: install_public_builtin_function(agent, cx, map_for_each_builtin(), None),
        map_size_getter: install_public_builtin_function(
            agent,
            cx,
            map_size_getter_builtin(),
            None,
        ),
        map_get_or_insert: install_public_builtin_function(
            agent,
            cx,
            map_get_or_insert_builtin(),
            None,
        ),
        map_get_or_insert_computed: install_public_builtin_function(
            agent,
            cx,
            map_get_or_insert_computed_builtin(),
            None,
        ),
        set_add: install_public_builtin_function(agent, cx, set_add_builtin(), None),
        set_has: install_public_builtin_function(agent, cx, set_has_builtin(), None),
        set_delete: install_public_builtin_function(agent, cx, set_delete_builtin(), None),
        set_clear: install_public_builtin_function(agent, cx, set_clear_builtin(), None),
        set_entries: install_public_builtin_function(agent, cx, set_entries_builtin(), None),
        set_values: install_public_builtin_function(agent, cx, set_values_builtin(), None),
        set_keys: install_public_builtin_function(agent, cx, set_keys_builtin(), None),
        set_for_each: install_public_builtin_function(agent, cx, set_for_each_builtin(), None),
        set_size_getter: install_public_builtin_function(
            agent,
            cx,
            set_size_getter_builtin(),
            None,
        ),
        set_union: install_public_builtin_function(agent, cx, set_union_builtin(), None),
        set_intersection: install_public_builtin_function(
            agent,
            cx,
            set_intersection_builtin(),
            None,
        ),
        set_difference: install_public_builtin_function(agent, cx, set_difference_builtin(), None),
        set_symmetric_difference: install_public_builtin_function(
            agent,
            cx,
            set_symmetric_difference_builtin(),
            None,
        ),
        set_is_subset_of: install_public_builtin_function(
            agent,
            cx,
            set_is_subset_of_builtin(),
            None,
        ),
        set_is_superset_of: install_public_builtin_function(
            agent,
            cx,
            set_is_superset_of_builtin(),
            None,
        ),
        set_is_disjoint_from: install_public_builtin_function(
            agent,
            cx,
            set_is_disjoint_from_builtin(),
            None,
        ),
        weak_map_get: install_public_builtin_function(agent, cx, weak_map_get_builtin(), None),
        weak_map_set: install_public_builtin_function(agent, cx, weak_map_set_builtin(), None),
        weak_map_has: install_public_builtin_function(agent, cx, weak_map_has_builtin(), None),
        weak_map_delete: install_public_builtin_function(
            agent,
            cx,
            weak_map_delete_builtin(),
            None,
        ),
        weak_map_get_or_insert: install_public_builtin_function(
            agent,
            cx,
            weak_map_get_or_insert_builtin(),
            None,
        ),
        weak_map_get_or_insert_computed: install_public_builtin_function(
            agent,
            cx,
            weak_map_get_or_insert_computed_builtin(),
            None,
        ),
        weak_set_add: install_public_builtin_function(agent, cx, weak_set_add_builtin(), None),
        weak_set_has: install_public_builtin_function(agent, cx, weak_set_has_builtin(), None),
        weak_set_delete: install_public_builtin_function(
            agent,
            cx,
            weak_set_delete_builtin(),
            None,
        ),
        weak_ref_deref: install_public_builtin_function(agent, cx, weak_ref_deref_builtin(), None),
        finalization_registry_register: install_public_builtin_function(
            agent,
            cx,
            finalization_registry_register_builtin(),
            None,
        ),
        finalization_registry_unregister: install_public_builtin_function(
            agent,
            cx,
            finalization_registry_unregister_builtin(),
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
        Some(array_species_getter_builtin()),
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
        builtin_function_atom_property(atoms.get, map_get_builtin()),
        builtin_function_atom_property(atoms.set, map_set_builtin()),
        builtin_function_atom_property(atoms.has, map_has_builtin()),
        builtin_function_atom_property(atoms.delete, map_delete_builtin()),
        builtin_function_atom_property(atoms.clear, map_clear_builtin()),
        builtin_function_atom_property(atoms.entries, map_entries_builtin()),
        builtin_function_atom_property(atoms.values, map_values_builtin()),
        builtin_function_atom_property(atoms.keys, map_keys_builtin()),
        builtin_function_atom_property(atoms.for_each, map_for_each_builtin()),
        builtin_function_atom_property(atoms.get_or_insert, map_get_or_insert_builtin()),
        builtin_function_atom_property(
            atoms.get_or_insert_computed,
            map_get_or_insert_computed_builtin(),
        ),
        accessor_atom_property(
            atoms.size,
            Some(map_size_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        builtin_function_symbol_property(
            WellKnownSymbolId::Iterator,
            map_entries_builtin(),
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
        Some(array_species_getter_builtin()),
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
        builtin_function_atom_property(atoms.add, set_add_builtin()),
        builtin_function_atom_property(atoms.has, set_has_builtin()),
        builtin_function_atom_property(atoms.delete, set_delete_builtin()),
        builtin_function_atom_property(atoms.clear, set_clear_builtin()),
        builtin_function_atom_property(atoms.entries, set_entries_builtin()),
        builtin_function_atom_property(atoms.values, set_values_builtin()),
        builtin_function_atom_property(atoms.union, set_union_builtin()),
        builtin_function_atom_property(atoms.intersection, set_intersection_builtin()),
        builtin_function_atom_property(atoms.difference, set_difference_builtin()),
        builtin_function_atom_property(
            atoms.symmetric_difference,
            set_symmetric_difference_builtin(),
        ),
        builtin_function_atom_property(atoms.is_subset_of, set_is_subset_of_builtin()),
        builtin_function_atom_property(atoms.is_superset_of, set_is_superset_of_builtin()),
        builtin_function_atom_property(atoms.is_disjoint_from, set_is_disjoint_from_builtin()),
        data_atom_property(
            atoms.keys,
            Value::from_object_ref(builtins.set_values),
            writable_builtin_attributes(),
        ),
        builtin_function_atom_property(atoms.for_each, set_for_each_builtin()),
        accessor_atom_property(
            atoms.size,
            Some(set_size_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        builtin_function_symbol_property(
            WellKnownSymbolId::Iterator,
            set_values_builtin(),
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
        builtin_function_atom_property(atoms.get, weak_map_get_builtin()),
        builtin_function_atom_property(atoms.set, weak_map_set_builtin()),
        builtin_function_atom_property(atoms.has, weak_map_has_builtin()),
        builtin_function_atom_property(atoms.delete, weak_map_delete_builtin()),
        builtin_function_atom_property(atoms.get_or_insert, weak_map_get_or_insert_builtin()),
        builtin_function_atom_property(
            atoms.get_or_insert_computed,
            weak_map_get_or_insert_computed_builtin(),
        ),
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
        builtin_function_atom_property(atoms.add, weak_set_add_builtin()),
        builtin_function_atom_property(atoms.has, weak_set_has_builtin()),
        builtin_function_atom_property(atoms.delete, weak_set_delete_builtin()),
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
        builtin_function_atom_property(atoms.deref, weak_ref_deref_builtin()),
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
        builtin_function_atom_property(atoms.register, finalization_registry_register_builtin()),
        builtin_function_atom_property(
            atoms.unregister,
            finalization_registry_unregister_builtin(),
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
    difference: AtomId,
    entries: AtomId,
    for_each: AtomId,
    get: AtomId,
    get_or_insert: AtomId,
    get_or_insert_computed: AtomId,
    has: AtomId,
    intersection: AtomId,
    is_disjoint_from: AtomId,
    is_subset_of: AtomId,
    is_superset_of: AtomId,
    keys: AtomId,
    register: AtomId,
    set: AtomId,
    size: AtomId,
    symmetric_difference: AtomId,
    union: AtomId,
    unregister: AtomId,
    values: AtomId,
}

impl CollectionDescriptorAtoms {
    fn new(agent: &mut Agent) -> Self {
        let atoms = agent.atoms_mut();
        Self {
            add: atoms.intern("add"),
            clear: atoms.intern("clear"),
            delete: atoms.intern("delete"),
            deref: atoms.intern("deref"),
            difference: atoms.intern("difference"),
            entries: atoms.intern("entries"),
            for_each: atoms.intern("forEach"),
            get: atoms.intern("get"),
            get_or_insert: atoms.intern("getOrInsert"),
            get_or_insert_computed: atoms.intern("getOrInsertComputed"),
            has: atoms.intern("has"),
            intersection: atoms.intern("intersection"),
            is_disjoint_from: atoms.intern("isDisjointFrom"),
            is_subset_of: atoms.intern("isSubsetOf"),
            is_superset_of: atoms.intern("isSupersetOf"),
            keys: atoms.intern("keys"),
            register: atoms.intern("register"),
            set: atoms.intern("set"),
            size: atoms.intern("size"),
            symmetric_difference: atoms.intern("symmetricDifference"),
            union: atoms.intern("union"),
            unregister: atoms.intern("unregister"),
            values: atoms.intern("values"),
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
        (map_builtin(), builtins.map),
        (set_builtin(), builtins.set),
        (weak_map_builtin(), builtins.weak_map),
        (weak_set_builtin(), builtins.weak_set),
        (weak_ref_builtin(), builtins.weak_ref),
        (
            finalization_registry_builtin(),
            builtins.finalization_registry,
        ),
        (map_get_builtin(), builtins.map_get),
        (map_set_builtin(), builtins.map_set),
        (map_has_builtin(), builtins.map_has),
        (map_delete_builtin(), builtins.map_delete),
        (map_clear_builtin(), builtins.map_clear),
        (map_entries_builtin(), builtins.map_entries),
        (map_values_builtin(), builtins.map_values),
        (map_keys_builtin(), builtins.map_keys),
        (map_for_each_builtin(), builtins.map_for_each),
        (map_size_getter_builtin(), builtins.map_size_getter),
        (map_get_or_insert_builtin(), builtins.map_get_or_insert),
        (
            map_get_or_insert_computed_builtin(),
            builtins.map_get_or_insert_computed,
        ),
        (set_add_builtin(), builtins.set_add),
        (set_has_builtin(), builtins.set_has),
        (set_delete_builtin(), builtins.set_delete),
        (set_clear_builtin(), builtins.set_clear),
        (set_entries_builtin(), builtins.set_entries),
        (set_values_builtin(), builtins.set_values),
        (set_keys_builtin(), builtins.set_keys),
        (set_for_each_builtin(), builtins.set_for_each),
        (set_size_getter_builtin(), builtins.set_size_getter),
        (set_union_builtin(), builtins.set_union),
        (set_intersection_builtin(), builtins.set_intersection),
        (set_difference_builtin(), builtins.set_difference),
        (
            set_symmetric_difference_builtin(),
            builtins.set_symmetric_difference,
        ),
        (set_is_subset_of_builtin(), builtins.set_is_subset_of),
        (set_is_superset_of_builtin(), builtins.set_is_superset_of),
        (
            set_is_disjoint_from_builtin(),
            builtins.set_is_disjoint_from,
        ),
        (weak_map_get_builtin(), builtins.weak_map_get),
        (weak_map_set_builtin(), builtins.weak_map_set),
        (weak_map_has_builtin(), builtins.weak_map_has),
        (weak_map_delete_builtin(), builtins.weak_map_delete),
        (
            weak_map_get_or_insert_builtin(),
            builtins.weak_map_get_or_insert,
        ),
        (
            weak_map_get_or_insert_computed_builtin(),
            builtins.weak_map_get_or_insert_computed,
        ),
        (weak_set_add_builtin(), builtins.weak_set_add),
        (weak_set_has_builtin(), builtins.weak_set_has),
        (weak_set_delete_builtin(), builtins.weak_set_delete),
        (weak_ref_deref_builtin(), builtins.weak_ref_deref),
        (
            finalization_registry_register_builtin(),
            builtins.finalization_registry_register,
        ),
        (
            finalization_registry_unregister_builtin(),
            builtins.finalization_registry_unregister,
        ),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

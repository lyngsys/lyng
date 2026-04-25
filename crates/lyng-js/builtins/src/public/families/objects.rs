use super::descriptors::{
    accessor_atom_property, builtin_function_atom_property, data_atom_property,
    readonly_builtin_attributes, writable_builtin_attributes,
};
use super::{install_public_builtin_function, FamilyInstallContext, ObjectFamilyBuiltins};
use crate::bootstrap::{install_descriptor_tables, BuiltinBootstrapError};
use crate::public::{BuiltinCache, PublicRealmBuiltins};
use crate::{BuiltinDescriptorTable, BuiltinInstallTarget, BuiltinIntrinsic};
use lyng_js_common::{AtomId, WellKnownAtom};
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
    BuiltinFunctionId, ObjectRef, RealmRef, Value,
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

pub(in crate::public) fn install_object_family_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
) -> Result<(), BuiltinBootstrapError> {
    let atoms = ObjectDescriptorAtoms::new(agent);
    install_object_constructor_descriptors(agent, cache, realm, &atoms)?;
    install_object_prototype_descriptors(agent, cache, realm, builtins, &atoms)
}

fn install_object_constructor_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    atoms: &ObjectDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    let bootstrap_atoms = agent.bootstrap_atoms();
    let descriptors = [
        builtin_function_atom_property(atoms.assign, js3_object_assign_builtin()),
        builtin_function_atom_property(bootstrap_atoms.create(), js3_object_create_builtin()),
        builtin_function_atom_property(
            bootstrap_atoms.get_prototype_of(),
            js3_object_get_prototype_of_builtin(),
        ),
        builtin_function_atom_property(
            bootstrap_atoms.set_prototype_of(),
            js3_object_set_prototype_of_builtin(),
        ),
        builtin_function_atom_property(
            bootstrap_atoms.get_own_property_descriptor(),
            js3_object_get_own_property_descriptor_builtin(),
        ),
        builtin_function_atom_property(
            atoms.get_own_property_descriptors,
            js3_object_get_own_property_descriptors_builtin(),
        ),
        builtin_function_atom_property(
            atoms.get_own_property_names,
            js3_object_get_own_property_names_builtin(),
        ),
        builtin_function_atom_property(
            atoms.get_own_property_symbols,
            js3_object_get_own_property_symbols_builtin(),
        ),
        builtin_function_atom_property(
            atoms.define_properties,
            js3_object_define_properties_builtin(),
        ),
        builtin_function_atom_property(
            bootstrap_atoms.define_property(),
            js3_object_define_property_builtin(),
        ),
        builtin_function_atom_property(atoms.from_entries, js3_object_from_entries_builtin()),
        builtin_function_atom_property(atoms.group_by, js3_object_group_by_builtin()),
        builtin_function_atom_property(
            bootstrap_atoms.prevent_extensions(),
            js3_object_prevent_extensions_builtin(),
        ),
        builtin_function_atom_property(
            bootstrap_atoms.is_extensible(),
            js3_object_is_extensible_builtin(),
        ),
        builtin_function_atom_property(atoms.is, js3_object_is_builtin()),
        builtin_function_atom_property(bootstrap_atoms.seal(), js3_object_seal_builtin()),
        builtin_function_atom_property(bootstrap_atoms.freeze(), js3_object_freeze_builtin()),
        builtin_function_atom_property(bootstrap_atoms.is_sealed(), js3_object_is_sealed_builtin()),
        builtin_function_atom_property(bootstrap_atoms.is_frozen(), js3_object_is_frozen_builtin()),
        builtin_function_atom_property(atoms.keys, js3_object_keys_builtin()),
        builtin_function_atom_property(atoms.entries, js3_object_entries_builtin()),
        builtin_function_atom_property(atoms.values, js3_object_values_builtin()),
        builtin_function_atom_property(atoms.has_own, js3_object_has_own_builtin()),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Object),
            &descriptors,
        )],
    )
}

fn install_object_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
    atoms: &ObjectDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    let bootstrap_atoms = agent.bootstrap_atoms();
    let descriptors = [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(builtins.object),
            writable_builtin_attributes(),
        ),
        builtin_function_atom_property(atoms.define_getter, js3_object_define_getter_builtin()),
        builtin_function_atom_property(atoms.define_setter, js3_object_define_setter_builtin()),
        builtin_function_atom_property(
            atoms.to_locale_string,
            js3_object_to_locale_string_builtin(),
        ),
        builtin_function_atom_property(
            WellKnownAtom::toString.id(),
            js3_object_to_string_builtin(),
        ),
        builtin_function_atom_property(WellKnownAtom::valueOf.id(), js3_object_value_of_builtin()),
        builtin_function_atom_property(
            bootstrap_atoms.has_own_property(),
            js3_object_has_own_property_builtin(),
        ),
        builtin_function_atom_property(atoms.lookup_getter, js3_object_lookup_getter_builtin()),
        builtin_function_atom_property(atoms.lookup_setter, js3_object_lookup_setter_builtin()),
        builtin_function_atom_property(
            bootstrap_atoms.is_prototype_of(),
            js3_object_is_prototype_of_builtin(),
        ),
        builtin_function_atom_property(
            bootstrap_atoms.property_is_enumerable(),
            js3_object_property_is_enumerable_builtin(),
        ),
        accessor_atom_property(
            WellKnownAtom::__proto__.id(),
            Some(js3_object_proto_getter_builtin()),
            Some(js3_object_proto_setter_builtin()),
            readonly_builtin_attributes(),
        ),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::ObjectPrototype),
            &descriptors,
        )],
    )
}

#[derive(Clone, Copy, Debug)]
struct ObjectDescriptorAtoms {
    assign: AtomId,
    define_getter: AtomId,
    define_properties: AtomId,
    define_setter: AtomId,
    entries: AtomId,
    from_entries: AtomId,
    get_own_property_descriptors: AtomId,
    get_own_property_names: AtomId,
    get_own_property_symbols: AtomId,
    group_by: AtomId,
    has_own: AtomId,
    is: AtomId,
    keys: AtomId,
    lookup_getter: AtomId,
    lookup_setter: AtomId,
    to_locale_string: AtomId,
    values: AtomId,
}

impl ObjectDescriptorAtoms {
    fn new(agent: &mut Agent) -> Self {
        let atoms = agent.atoms_mut();
        Self {
            assign: atoms.intern_collectible("assign"),
            define_getter: atoms.intern_collectible("__defineGetter__"),
            define_properties: atoms.intern_collectible("defineProperties"),
            define_setter: atoms.intern_collectible("__defineSetter__"),
            entries: atoms.intern_collectible("entries"),
            from_entries: atoms.intern_collectible("fromEntries"),
            get_own_property_descriptors: atoms.intern_collectible("getOwnPropertyDescriptors"),
            get_own_property_names: atoms.intern_collectible("getOwnPropertyNames"),
            get_own_property_symbols: atoms.intern_collectible("getOwnPropertySymbols"),
            group_by: atoms.intern_collectible("groupBy"),
            has_own: atoms.intern_collectible("hasOwn"),
            is: atoms.intern_collectible("is"),
            keys: atoms.intern_collectible("keys"),
            lookup_getter: atoms.intern_collectible("__lookupGetter__"),
            lookup_setter: atoms.intern_collectible("__lookupSetter__"),
            to_locale_string: atoms.intern_collectible("toLocaleString"),
            values: atoms.intern_collectible("values"),
        }
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

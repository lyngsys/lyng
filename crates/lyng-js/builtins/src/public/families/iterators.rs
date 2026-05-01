use super::descriptors::{
    accessor_atom_property, accessor_symbol_property, builtin_function_atom_property,
    builtin_function_symbol_property, data_atom_property, data_symbol_property, descriptor_tag,
    readonly_builtin_attributes, writable_builtin_attributes,
};
use super::{
    install_public_builtin_function, install_public_builtin_function_with_metadata,
    FamilyInstallContext, IteratorFamilyBuiltins, IteratorFamilyPrototypes,
};
use crate::bootstrap::{install_descriptor_tables, BuiltinBootstrapError};
use crate::public::{BuiltinCache, PublicRealmBuiltins};
use crate::{BuiltinDescriptorTable, BuiltinEntryMetadata, BuiltinInstallTarget, BuiltinIntrinsic};
use lyng_js_common::AtomId;
use lyng_js_env::Agent;
use lyng_js_types::{
    iterator_builtin, iterator_concat_builtin, iterator_constructor_getter_builtin,
    iterator_constructor_setter_builtin, iterator_dispose_builtin, iterator_drop_builtin,
    iterator_every_builtin, iterator_filter_builtin, iterator_find_builtin,
    iterator_flat_map_builtin, iterator_for_each_builtin, iterator_from_builtin,
    iterator_helper_next_builtin, iterator_helper_return_builtin, iterator_map_builtin,
    iterator_prototype_iterator_builtin, iterator_reduce_builtin, iterator_some_builtin,
    iterator_take_builtin, iterator_to_array_builtin, iterator_to_string_tag_getter_builtin,
    iterator_to_string_tag_setter_builtin, iterator_zip_builtin, iterator_zip_keyed_builtin,
    map_iterator_next_builtin, set_iterator_next_builtin, BuiltinFunctionId, ObjectRef, RealmRef,
    Value, WellKnownSymbolId,
};

pub(in crate::public) fn install_iterator_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototypes: IteratorFamilyPrototypes,
) -> IteratorFamilyBuiltins {
    IteratorFamilyBuiltins {
        async_iterator_prototype: prototypes.async_iterator_prototype,
        iterator_prototype_iterator: install_public_builtin_function(
            agent,
            cx,
            iterator_prototype_iterator_builtin(),
            None,
        ),
        async_iterator_method: install_public_builtin_function_with_metadata(
            agent,
            cx,
            iterator_prototype_iterator_builtin(),
            BuiltinEntryMetadata::new("[Symbol.asyncIterator]", 0, false, false),
            None,
        ),
        map_iterator_next: install_public_builtin_function(
            agent,
            cx,
            map_iterator_next_builtin(),
            None,
        ),
        set_iterator_next: install_public_builtin_function(
            agent,
            cx,
            set_iterator_next_builtin(),
            None,
        ),
        iterator: install_public_builtin_function(
            agent,
            cx,
            iterator_builtin(),
            Some(prototypes.iterator_prototype),
        ),
        iterator_from: install_public_builtin_function(agent, cx, iterator_from_builtin(), None),
        iterator_concat: install_public_builtin_function(
            agent,
            cx,
            iterator_concat_builtin(),
            None,
        ),
        iterator_zip: install_public_builtin_function(agent, cx, iterator_zip_builtin(), None),
        iterator_zip_keyed: install_public_builtin_function(
            agent,
            cx,
            iterator_zip_keyed_builtin(),
            None,
        ),
        iterator_reduce: install_public_builtin_function(
            agent,
            cx,
            iterator_reduce_builtin(),
            None,
        ),
        iterator_for_each: install_public_builtin_function(
            agent,
            cx,
            iterator_for_each_builtin(),
            None,
        ),
        iterator_some: install_public_builtin_function(agent, cx, iterator_some_builtin(), None),
        iterator_every: install_public_builtin_function(agent, cx, iterator_every_builtin(), None),
        iterator_find: install_public_builtin_function(agent, cx, iterator_find_builtin(), None),
        iterator_to_array: install_public_builtin_function(
            agent,
            cx,
            iterator_to_array_builtin(),
            None,
        ),
        iterator_map: install_public_builtin_function(agent, cx, iterator_map_builtin(), None),
        iterator_filter: install_public_builtin_function(
            agent,
            cx,
            iterator_filter_builtin(),
            None,
        ),
        iterator_take: install_public_builtin_function(agent, cx, iterator_take_builtin(), None),
        iterator_drop: install_public_builtin_function(agent, cx, iterator_drop_builtin(), None),
        iterator_dispose: install_public_builtin_function(
            agent,
            cx,
            iterator_dispose_builtin(),
            None,
        ),
        iterator_flat_map: install_public_builtin_function(
            agent,
            cx,
            iterator_flat_map_builtin(),
            None,
        ),
        iterator_helper_next: install_public_builtin_function(
            agent,
            cx,
            iterator_helper_next_builtin(),
            None,
        ),
        iterator_helper_return: install_public_builtin_function(
            agent,
            cx,
            iterator_helper_return_builtin(),
            None,
        ),
        iterator_to_string_tag_getter: install_public_builtin_function(
            agent,
            cx,
            iterator_to_string_tag_getter_builtin(),
            None,
        ),
        iterator_to_string_tag_setter: install_public_builtin_function(
            agent,
            cx,
            iterator_to_string_tag_setter_builtin(),
            None,
        ),
        iterator_constructor_getter: install_public_builtin_function(
            agent,
            cx,
            iterator_constructor_getter_builtin(),
            None,
        ),
        iterator_constructor_setter: install_public_builtin_function(
            agent,
            cx,
            iterator_constructor_setter_builtin(),
            None,
        ),
    }
}

pub(in crate::public) fn iterator_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (
            iterator_prototype_iterator_builtin(),
            builtins.iterator_prototype_iterator,
        ),
        (map_iterator_next_builtin(), builtins.map_iterator_next),
        (set_iterator_next_builtin(), builtins.set_iterator_next),
        (iterator_builtin(), builtins.iterator),
        (iterator_from_builtin(), builtins.iterator_from),
        (iterator_concat_builtin(), builtins.iterator_concat),
        (iterator_zip_builtin(), builtins.iterator_zip),
        (iterator_zip_keyed_builtin(), builtins.iterator_zip_keyed),
        (iterator_reduce_builtin(), builtins.iterator_reduce),
        (iterator_for_each_builtin(), builtins.iterator_for_each),
        (iterator_some_builtin(), builtins.iterator_some),
        (iterator_every_builtin(), builtins.iterator_every),
        (iterator_find_builtin(), builtins.iterator_find),
        (iterator_to_array_builtin(), builtins.iterator_to_array),
        (iterator_map_builtin(), builtins.iterator_map),
        (iterator_filter_builtin(), builtins.iterator_filter),
        (iterator_take_builtin(), builtins.iterator_take),
        (iterator_drop_builtin(), builtins.iterator_drop),
        (iterator_dispose_builtin(), builtins.iterator_dispose),
        (iterator_flat_map_builtin(), builtins.iterator_flat_map),
        (
            iterator_helper_next_builtin(),
            builtins.iterator_helper_next,
        ),
        (
            iterator_helper_return_builtin(),
            builtins.iterator_helper_return,
        ),
        (
            iterator_to_string_tag_getter_builtin(),
            builtins.iterator_to_string_tag_getter,
        ),
        (
            iterator_to_string_tag_setter_builtin(),
            builtins.iterator_to_string_tag_setter,
        ),
        (
            iterator_constructor_getter_builtin(),
            builtins.iterator_constructor_getter,
        ),
        (
            iterator_constructor_setter_builtin(),
            builtins.iterator_constructor_setter,
        ),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

pub(in crate::public) fn install_iterator_family_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
) -> Result<(), BuiltinBootstrapError> {
    let atoms = IteratorDescriptorAtoms::new(agent);
    let async_iterator_tag = descriptor_tag(agent, "AsyncIterator");
    let map_iterator_tag = descriptor_tag(agent, "Map Iterator");
    let set_iterator_tag = descriptor_tag(agent, "Set Iterator");

    let iterator_constructor_descriptors = [
        data_atom_property(
            atoms.from,
            Value::from_object_ref(builtins.iterator_from),
            writable_builtin_attributes(),
        ),
        data_atom_property(
            atoms.concat,
            Value::from_object_ref(builtins.iterator_concat),
            writable_builtin_attributes(),
        ),
        data_atom_property(
            atoms.zip,
            Value::from_object_ref(builtins.iterator_zip),
            writable_builtin_attributes(),
        ),
        data_atom_property(
            atoms.zip_keyed,
            Value::from_object_ref(builtins.iterator_zip_keyed),
            writable_builtin_attributes(),
        ),
    ];
    let iterator_prototype_descriptors = [
        builtin_function_symbol_property(
            WellKnownSymbolId::Iterator,
            iterator_prototype_iterator_builtin(),
            writable_builtin_attributes(),
        ),
        builtin_function_atom_property(atoms.reduce, iterator_reduce_builtin()),
        builtin_function_atom_property(atoms.for_each, iterator_for_each_builtin()),
        builtin_function_atom_property(atoms.some, iterator_some_builtin()),
        builtin_function_atom_property(atoms.every, iterator_every_builtin()),
        builtin_function_atom_property(atoms.find, iterator_find_builtin()),
        builtin_function_atom_property(atoms.to_array, iterator_to_array_builtin()),
        builtin_function_atom_property(atoms.map, iterator_map_builtin()),
        builtin_function_atom_property(atoms.filter, iterator_filter_builtin()),
        builtin_function_atom_property(atoms.take, iterator_take_builtin()),
        builtin_function_atom_property(atoms.drop, iterator_drop_builtin()),
        builtin_function_atom_property(atoms.flat_map, iterator_flat_map_builtin()),
        builtin_function_symbol_property(
            WellKnownSymbolId::Dispose,
            iterator_dispose_builtin(),
            writable_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.constructor,
            Some(iterator_constructor_getter_builtin()),
            Some(iterator_constructor_setter_builtin()),
            writable_builtin_attributes(),
        ),
        accessor_symbol_property(
            WellKnownSymbolId::ToStringTag,
            Some(iterator_to_string_tag_getter_builtin()),
            Some(iterator_to_string_tag_setter_builtin()),
            writable_builtin_attributes(),
        ),
    ];
    let iterator_helper_prototype_descriptors = [
        builtin_function_atom_property(atoms.next, iterator_helper_next_builtin()),
        builtin_function_atom_property(atoms.return_, iterator_helper_return_builtin()),
    ];
    let async_iterator_prototype_descriptors = [
        data_symbol_property(
            WellKnownSymbolId::AsyncIterator,
            Value::from_object_ref(builtins.async_iterator_method),
            writable_builtin_attributes(),
        ),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            async_iterator_tag,
            readonly_builtin_attributes(),
        ),
    ];
    let map_iterator_prototype_descriptors = [
        builtin_function_atom_property(atoms.next, map_iterator_next_builtin()),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            map_iterator_tag,
            readonly_builtin_attributes(),
        ),
    ];
    let set_iterator_prototype_descriptors = [
        builtin_function_atom_property(atoms.next, set_iterator_next_builtin()),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            set_iterator_tag,
            readonly_builtin_attributes(),
        ),
    ];
    let tables = [
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Iterator),
            &iterator_constructor_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::IteratorPrototype),
            &iterator_prototype_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::IteratorHelperPrototype),
            &iterator_helper_prototype_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::AsyncIteratorPrototype),
            &async_iterator_prototype_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::MapIteratorPrototype),
            &map_iterator_prototype_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::SetIteratorPrototype),
            &set_iterator_prototype_descriptors,
        ),
    ];
    install_descriptor_tables(agent, cache, realm, &tables)
}

#[derive(Clone, Copy)]
struct IteratorDescriptorAtoms {
    next: AtomId,
    constructor: AtomId,
    from: AtomId,
    concat: AtomId,
    zip: AtomId,
    zip_keyed: AtomId,
    reduce: AtomId,
    for_each: AtomId,
    some: AtomId,
    every: AtomId,
    find: AtomId,
    to_array: AtomId,
    map: AtomId,
    filter: AtomId,
    take: AtomId,
    drop: AtomId,
    flat_map: AtomId,
    return_: AtomId,
}

impl IteratorDescriptorAtoms {
    fn new(agent: &mut Agent) -> Self {
        let atoms = agent.atoms_mut();
        Self {
            next: atoms.intern("next"),
            constructor: lyng_js_common::WellKnownAtom::constructor.id(),
            from: atoms.intern("from"),
            concat: atoms.intern("concat"),
            zip: atoms.intern("zip"),
            zip_keyed: atoms.intern("zipKeyed"),
            reduce: atoms.intern("reduce"),
            for_each: atoms.intern("forEach"),
            some: atoms.intern("some"),
            every: atoms.intern("every"),
            find: atoms.intern("find"),
            to_array: atoms.intern("toArray"),
            map: atoms.intern("map"),
            filter: atoms.intern("filter"),
            take: atoms.intern("take"),
            drop: atoms.intern("drop"),
            flat_map: atoms.intern("flatMap"),
            return_: atoms.intern("return"),
        }
    }
}

use super::descriptors::{
    builtin_function_atom_property, builtin_function_symbol_property, data_symbol_property,
    descriptor_tag, readonly_builtin_attributes, writable_builtin_attributes,
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
    iterator_prototype_iterator_builtin, map_iterator_next_builtin, set_iterator_next_builtin,
    BuiltinFunctionId, ObjectRef, RealmRef, Value, WellKnownSymbolId,
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

    let iterator_prototype_descriptors = [builtin_function_symbol_property(
        WellKnownSymbolId::Iterator,
        iterator_prototype_iterator_builtin(),
        writable_builtin_attributes(),
    )];
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
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::IteratorPrototype),
            &iterator_prototype_descriptors,
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
}

impl IteratorDescriptorAtoms {
    fn new(agent: &mut Agent) -> Self {
        Self {
            next: agent.atoms_mut().intern_collectible("next"),
        }
    }
}

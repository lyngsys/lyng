use super::descriptors::{
    accessor_atom_property, builtin_function_atom_property, builtin_function_symbol_property,
    data_atom_property, data_symbol_property, descriptor_tag, hidden_builtin_attributes,
    readonly_builtin_attributes, writable_builtin_attributes,
};
use super::{
    install_public_builtin_function, FamilyInstallContext, FunctionFamilyBuiltins,
    FunctionFamilyPrototypes,
};
use crate::bootstrap::{install_descriptor_tables, BuiltinBootstrapError};
use crate::public::{BuiltinCache, PublicRealmBuiltins};
use crate::{BuiltinDescriptorTable, BuiltinInstallTarget, BuiltinIntrinsic};
use lyng_js_common::{AtomId, WellKnownAtom};
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_async_function_builtin, js3_async_generator_function_builtin,
    js3_async_generator_next_builtin, js3_async_generator_return_builtin,
    js3_async_generator_throw_builtin, js3_function_apply_builtin, js3_function_bind_builtin,
    js3_function_builtin, js3_function_call_builtin, js3_function_prototype_builtin,
    js3_function_symbol_has_instance_builtin, js3_function_to_string_builtin,
    js3_generator_function_builtin, js3_generator_next_builtin, js3_generator_return_builtin,
    js3_generator_throw_builtin, js3_internal_throw_type_error_builtin,
    js3_iterator_prototype_iterator_builtin, BuiltinFunctionId, ObjectRef, RealmRef, Value,
    WellKnownSymbolId,
};

#[allow(clippy::too_many_lines)]
pub(in crate::public) fn install_function_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototypes: FunctionFamilyPrototypes,
) -> FunctionFamilyBuiltins {
    FunctionFamilyBuiltins {
        function: install_public_builtin_function(
            agent,
            cx,
            js3_function_builtin(),
            Some(cx.function_prototype),
        ),
        function_prototype: cx.function_prototype,
        function_call: install_public_builtin_function(
            agent,
            cx,
            js3_function_call_builtin(),
            None,
        ),
        function_apply: install_public_builtin_function(
            agent,
            cx,
            js3_function_apply_builtin(),
            None,
        ),
        function_bind: install_public_builtin_function(
            agent,
            cx,
            js3_function_bind_builtin(),
            None,
        ),
        function_to_string: install_public_builtin_function(
            agent,
            cx,
            js3_function_to_string_builtin(),
            None,
        ),
        function_symbol_has_instance: install_public_builtin_function(
            agent,
            cx,
            js3_function_symbol_has_instance_builtin(),
            None,
        ),
        async_function: install_public_builtin_function(
            agent,
            cx,
            js3_async_function_builtin(),
            Some(prototypes.async_function_prototype),
        ),
        async_function_prototype: prototypes.async_function_prototype,
        async_generator_function: install_public_builtin_function(
            agent,
            cx,
            js3_async_generator_function_builtin(),
            Some(prototypes.async_generator_function_prototype),
        ),
        async_generator_function_prototype: prototypes.async_generator_function_prototype,
        async_generator_prototype: prototypes.async_generator_prototype,
        async_generator_next: install_public_builtin_function(
            agent,
            cx,
            js3_async_generator_next_builtin(),
            None,
        ),
        async_generator_return: install_public_builtin_function(
            agent,
            cx,
            js3_async_generator_return_builtin(),
            None,
        ),
        async_generator_throw: install_public_builtin_function(
            agent,
            cx,
            js3_async_generator_throw_builtin(),
            None,
        ),
        generator_function: install_public_builtin_function(
            agent,
            cx,
            js3_generator_function_builtin(),
            Some(prototypes.generator_function_prototype),
        ),
        generator_function_prototype: prototypes.generator_function_prototype,
        generator_prototype: prototypes.generator_prototype,
        generator_next: install_public_builtin_function(
            agent,
            cx,
            js3_generator_next_builtin(),
            None,
        ),
        generator_return: install_public_builtin_function(
            agent,
            cx,
            js3_generator_return_builtin(),
            None,
        ),
        generator_throw: install_public_builtin_function(
            agent,
            cx,
            js3_generator_throw_builtin(),
            None,
        ),
    }
}

pub(in crate::public) fn install_function_family_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
) -> Result<(), BuiltinBootstrapError> {
    let atoms = FunctionDescriptorAtoms::new(agent);
    let tags = FunctionDescriptorTags::new(agent);
    install_function_prototype_descriptors(agent, cache, realm, builtins, &atoms)?;
    install_async_function_prototype_descriptors(agent, cache, realm, builtins, &tags)?;
    install_async_generator_function_prototype_descriptors(agent, cache, realm, builtins, &tags)?;
    install_generator_function_prototype_descriptors(agent, cache, realm, builtins, &tags)?;
    install_generator_prototype_descriptors(agent, cache, realm, builtins, &atoms, &tags)?;
    install_async_generator_prototype_descriptors(agent, cache, realm, builtins, &atoms, &tags)
}

fn install_function_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
    atoms: &FunctionDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(builtins.function),
            writable_builtin_attributes(),
        ),
        builtin_function_atom_property(WellKnownAtom::call.id(), js3_function_call_builtin()),
        builtin_function_atom_property(WellKnownAtom::apply.id(), js3_function_apply_builtin()),
        builtin_function_atom_property(WellKnownAtom::bind.id(), js3_function_bind_builtin()),
        builtin_function_atom_property(
            WellKnownAtom::toString.id(),
            js3_function_to_string_builtin(),
        ),
        builtin_function_symbol_property(
            WellKnownSymbolId::HasInstance,
            js3_function_symbol_has_instance_builtin(),
            hidden_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.caller,
            Some(js3_internal_throw_type_error_builtin()),
            Some(js3_internal_throw_type_error_builtin()),
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.arguments,
            Some(js3_internal_throw_type_error_builtin()),
            Some(js3_internal_throw_type_error_builtin()),
            readonly_builtin_attributes(),
        ),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::FunctionPrototype),
            &descriptors,
        )],
    )
}

fn install_generator_function_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
    tags: &FunctionDescriptorTags,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(builtins.generator_function),
            readonly_builtin_attributes(),
        ),
        data_atom_property(
            WellKnownAtom::prototype.id(),
            Value::from_object_ref(builtins.generator_prototype),
            readonly_builtin_attributes(),
        ),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            tags.generator_function,
            readonly_builtin_attributes(),
        ),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::GeneratorFunctionPrototype),
            &descriptors,
        )],
    )
}

fn install_async_function_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
    tags: &FunctionDescriptorTags,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(builtins.async_function),
            readonly_builtin_attributes(),
        ),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            tags.async_function,
            readonly_builtin_attributes(),
        ),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::AsyncFunctionPrototype),
            &descriptors,
        )],
    )
}

fn install_async_generator_function_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
    tags: &FunctionDescriptorTags,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(builtins.async_generator_function),
            readonly_builtin_attributes(),
        ),
        data_atom_property(
            WellKnownAtom::prototype.id(),
            Value::from_object_ref(builtins.async_generator_prototype),
            readonly_builtin_attributes(),
        ),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            tags.async_generator_function,
            readonly_builtin_attributes(),
        ),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::AsyncGeneratorFunctionPrototype),
            &descriptors,
        )],
    )
}

fn install_generator_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
    atoms: &FunctionDescriptorAtoms,
    tags: &FunctionDescriptorTags,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(builtins.generator_function_prototype),
            readonly_builtin_attributes(),
        ),
        builtin_function_atom_property(atoms.next, js3_generator_next_builtin()),
        builtin_function_atom_property(
            WellKnownAtom::r#return.id(),
            js3_generator_return_builtin(),
        ),
        builtin_function_atom_property(atoms.throw, js3_generator_throw_builtin()),
        builtin_function_symbol_property(
            WellKnownSymbolId::Iterator,
            js3_iterator_prototype_iterator_builtin(),
            writable_builtin_attributes(),
        ),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            tags.generator,
            readonly_builtin_attributes(),
        ),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::GeneratorPrototype),
            &descriptors,
        )],
    )
}

fn install_async_generator_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
    atoms: &FunctionDescriptorAtoms,
    tags: &FunctionDescriptorTags,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(builtins.async_generator_function_prototype),
            readonly_builtin_attributes(),
        ),
        builtin_function_atom_property(atoms.next, js3_async_generator_next_builtin()),
        builtin_function_atom_property(
            WellKnownAtom::r#return.id(),
            js3_async_generator_return_builtin(),
        ),
        builtin_function_atom_property(atoms.throw, js3_async_generator_throw_builtin()),
        data_symbol_property(
            WellKnownSymbolId::AsyncIterator,
            Value::from_object_ref(builtins.async_iterator_method),
            writable_builtin_attributes(),
        ),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            tags.async_generator,
            readonly_builtin_attributes(),
        ),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::AsyncGeneratorPrototype),
            &descriptors,
        )],
    )
}

#[derive(Clone, Copy, Debug)]
struct FunctionDescriptorAtoms {
    arguments: AtomId,
    caller: AtomId,
    next: AtomId,
    throw: AtomId,
}

impl FunctionDescriptorAtoms {
    fn new(agent: &mut Agent) -> Self {
        let atoms = agent.atoms_mut();
        Self {
            arguments: atoms.intern_collectible("arguments"),
            caller: atoms.intern_collectible("caller"),
            next: atoms.intern_collectible("next"),
            throw: atoms.intern_collectible("throw"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct FunctionDescriptorTags {
    async_function: Value,
    async_generator: Value,
    async_generator_function: Value,
    generator: Value,
    generator_function: Value,
}

impl FunctionDescriptorTags {
    fn new(agent: &mut Agent) -> Self {
        Self {
            async_function: descriptor_tag(agent, "AsyncFunction"),
            async_generator: descriptor_tag(agent, "AsyncGenerator"),
            async_generator_function: descriptor_tag(agent, "AsyncGeneratorFunction"),
            generator: descriptor_tag(agent, "Generator"),
            generator_function: descriptor_tag(agent, "GeneratorFunction"),
        }
    }
}

pub(in crate::public) fn function_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (js3_function_builtin(), builtins.function),
        (
            js3_function_prototype_builtin(),
            builtins.function_prototype,
        ),
        (js3_function_call_builtin(), builtins.function_call),
        (js3_function_apply_builtin(), builtins.function_apply),
        (js3_function_bind_builtin(), builtins.function_bind),
        (
            js3_function_to_string_builtin(),
            builtins.function_to_string,
        ),
        (
            js3_function_symbol_has_instance_builtin(),
            builtins.function_symbol_has_instance,
        ),
        (js3_async_function_builtin(), builtins.async_function),
        (
            js3_async_generator_function_builtin(),
            builtins.async_generator_function,
        ),
        (
            js3_async_generator_next_builtin(),
            builtins.async_generator_next,
        ),
        (
            js3_async_generator_return_builtin(),
            builtins.async_generator_return,
        ),
        (
            js3_async_generator_throw_builtin(),
            builtins.async_generator_throw,
        ),
        (
            js3_generator_function_builtin(),
            builtins.generator_function,
        ),
        (js3_generator_next_builtin(), builtins.generator_next),
        (js3_generator_return_builtin(), builtins.generator_return),
        (js3_generator_throw_builtin(), builtins.generator_throw),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

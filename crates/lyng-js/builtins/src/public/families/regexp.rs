use super::descriptors::{
    accessor_atom_property, accessor_symbol_property, builtin_function_atom_property,
    builtin_function_symbol_property, data_atom_property, readonly_builtin_attributes,
    writable_builtin_attributes,
};
use super::{
    install_public_builtin_function, FamilyInstallContext, RegExpFamilyBuiltins,
    RegExpFamilyPrototypes,
};
use crate::bootstrap::{install_descriptor_tables, BuiltinBootstrapError};
use crate::public::{BuiltinCache, PublicRealmBuiltins};
use crate::{
    BuiltinDescriptorTable, BuiltinInstallTarget, BuiltinIntrinsic, BuiltinPropertyDescriptor,
};
use lyng_js_common::{AtomId, WellKnownAtom};
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_regexp_builtin, js3_regexp_dot_all_getter_builtin, js3_regexp_escape_builtin,
    js3_regexp_exec_builtin, js3_regexp_flags_getter_builtin, js3_regexp_global_getter_builtin,
    js3_regexp_has_indices_getter_builtin, js3_regexp_ignore_case_getter_builtin,
    js3_regexp_multiline_getter_builtin, js3_regexp_source_getter_builtin,
    js3_regexp_species_getter_builtin, js3_regexp_sticky_getter_builtin,
    js3_regexp_symbol_match_all_builtin, js3_regexp_symbol_match_builtin,
    js3_regexp_symbol_replace_builtin, js3_regexp_symbol_search_builtin,
    js3_regexp_symbol_split_builtin, js3_regexp_test_builtin, js3_regexp_to_string_builtin,
    js3_regexp_unicode_getter_builtin, BuiltinFunctionId, ObjectRef, RealmRef, Value,
    WellKnownSymbolId,
};

pub(in crate::public) fn install_regexp_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototypes: RegExpFamilyPrototypes,
) -> RegExpFamilyBuiltins {
    let flag_accessors = install_regexp_flag_accessors(agent, cx);
    let symbol_methods = install_regexp_symbol_methods(agent, cx);

    RegExpFamilyBuiltins {
        regexp: install_public_builtin_function(
            agent,
            cx,
            js3_regexp_builtin(),
            Some(prototypes.regexp_prototype),
        ),
        regexp_escape: install_public_builtin_function(
            agent,
            cx,
            js3_regexp_escape_builtin(),
            None,
        ),
        regexp_prototype: prototypes.regexp_prototype,
        regexp_to_string: install_public_builtin_function(
            agent,
            cx,
            js3_regexp_to_string_builtin(),
            None,
        ),
        regexp_exec: install_public_builtin_function(agent, cx, js3_regexp_exec_builtin(), None),
        regexp_test: install_public_builtin_function(agent, cx, js3_regexp_test_builtin(), None),
        regexp_global_getter: flag_accessors.global,
        regexp_ignore_case_getter: flag_accessors.ignore_case,
        regexp_multiline_getter: flag_accessors.multiline,
        regexp_dot_all_getter: flag_accessors.dot_all,
        regexp_unicode_getter: flag_accessors.unicode,
        regexp_sticky_getter: flag_accessors.sticky,
        regexp_source_getter: flag_accessors.source,
        regexp_flags_getter: flag_accessors.flags,
        regexp_has_indices_getter: flag_accessors.has_indices,
        regexp_species_getter: install_public_builtin_function(
            agent,
            cx,
            js3_regexp_species_getter_builtin(),
            None,
        ),
        regexp_symbol_match: symbol_methods.match_method,
        regexp_symbol_replace: symbol_methods.replace,
        regexp_symbol_search: symbol_methods.search,
        regexp_symbol_split: symbol_methods.split,
        regexp_symbol_match_all: symbol_methods.match_all,
    }
}

pub(in crate::public) fn regexp_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (js3_regexp_builtin(), builtins.regexp),
        (js3_regexp_escape_builtin(), builtins.regexp_escape),
        (js3_regexp_to_string_builtin(), builtins.regexp_to_string),
        (js3_regexp_exec_builtin(), builtins.regexp_exec),
        (js3_regexp_test_builtin(), builtins.regexp_test),
        (
            js3_regexp_global_getter_builtin(),
            builtins.regexp_global_getter,
        ),
        (
            js3_regexp_ignore_case_getter_builtin(),
            builtins.regexp_ignore_case_getter,
        ),
        (
            js3_regexp_multiline_getter_builtin(),
            builtins.regexp_multiline_getter,
        ),
        (
            js3_regexp_dot_all_getter_builtin(),
            builtins.regexp_dot_all_getter,
        ),
        (
            js3_regexp_unicode_getter_builtin(),
            builtins.regexp_unicode_getter,
        ),
        (
            js3_regexp_sticky_getter_builtin(),
            builtins.regexp_sticky_getter,
        ),
        (
            js3_regexp_source_getter_builtin(),
            builtins.regexp_source_getter,
        ),
        (
            js3_regexp_flags_getter_builtin(),
            builtins.regexp_flags_getter,
        ),
        (
            js3_regexp_has_indices_getter_builtin(),
            builtins.regexp_has_indices_getter,
        ),
        (
            js3_regexp_species_getter_builtin(),
            builtins.regexp_species_getter,
        ),
        (
            js3_regexp_symbol_match_builtin(),
            builtins.regexp_symbol_match,
        ),
        (
            js3_regexp_symbol_replace_builtin(),
            builtins.regexp_symbol_replace,
        ),
        (
            js3_regexp_symbol_search_builtin(),
            builtins.regexp_symbol_search,
        ),
        (
            js3_regexp_symbol_split_builtin(),
            builtins.regexp_symbol_split,
        ),
        (
            js3_regexp_symbol_match_all_builtin(),
            builtins.regexp_symbol_match_all,
        ),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

#[derive(Clone, Copy, Debug)]
struct RegExpFlagAccessors {
    global: ObjectRef,
    ignore_case: ObjectRef,
    multiline: ObjectRef,
    dot_all: ObjectRef,
    unicode: ObjectRef,
    sticky: ObjectRef,
    source: ObjectRef,
    flags: ObjectRef,
    has_indices: ObjectRef,
}

fn install_regexp_flag_accessors(
    agent: &mut Agent,
    cx: FamilyInstallContext,
) -> RegExpFlagAccessors {
    RegExpFlagAccessors {
        global: install_public_builtin_function(
            agent,
            cx,
            js3_regexp_global_getter_builtin(),
            None,
        ),
        ignore_case: install_public_builtin_function(
            agent,
            cx,
            js3_regexp_ignore_case_getter_builtin(),
            None,
        ),
        multiline: install_public_builtin_function(
            agent,
            cx,
            js3_regexp_multiline_getter_builtin(),
            None,
        ),
        dot_all: install_public_builtin_function(
            agent,
            cx,
            js3_regexp_dot_all_getter_builtin(),
            None,
        ),
        unicode: install_public_builtin_function(
            agent,
            cx,
            js3_regexp_unicode_getter_builtin(),
            None,
        ),
        sticky: install_public_builtin_function(
            agent,
            cx,
            js3_regexp_sticky_getter_builtin(),
            None,
        ),
        source: install_public_builtin_function(
            agent,
            cx,
            js3_regexp_source_getter_builtin(),
            None,
        ),
        flags: install_public_builtin_function(agent, cx, js3_regexp_flags_getter_builtin(), None),
        has_indices: install_public_builtin_function(
            agent,
            cx,
            js3_regexp_has_indices_getter_builtin(),
            None,
        ),
    }
}

#[derive(Clone, Copy, Debug)]
struct RegExpSymbolMethods {
    match_method: ObjectRef,
    replace: ObjectRef,
    search: ObjectRef,
    split: ObjectRef,
    match_all: ObjectRef,
}

fn install_regexp_symbol_methods(
    agent: &mut Agent,
    cx: FamilyInstallContext,
) -> RegExpSymbolMethods {
    RegExpSymbolMethods {
        match_method: install_public_builtin_function(
            agent,
            cx,
            js3_regexp_symbol_match_builtin(),
            None,
        ),
        replace: install_public_builtin_function(
            agent,
            cx,
            js3_regexp_symbol_replace_builtin(),
            None,
        ),
        search: install_public_builtin_function(
            agent,
            cx,
            js3_regexp_symbol_search_builtin(),
            None,
        ),
        split: install_public_builtin_function(agent, cx, js3_regexp_symbol_split_builtin(), None),
        match_all: install_public_builtin_function(
            agent,
            cx,
            js3_regexp_symbol_match_all_builtin(),
            None,
        ),
    }
}

pub(in crate::public) fn install_regexp_family_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
) -> Result<(), BuiltinBootstrapError> {
    let atoms = RegExpDescriptorAtoms::new(agent);
    install_regexp_constructor_descriptors(agent, cache, realm, atoms)?;
    install_regexp_prototype_descriptors(agent, cache, realm, builtins.regexp, atoms)
}

fn install_regexp_constructor_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    atoms: RegExpDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        builtin_function_atom_property(atoms.escape, js3_regexp_escape_builtin()),
        accessor_symbol_property(
            WellKnownSymbolId::Species,
            Some(js3_regexp_species_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
    ];
    install_intrinsic_descriptor_table(agent, cache, realm, BuiltinIntrinsic::RegExp, &descriptors)
}

fn install_regexp_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    regexp: ObjectRef,
    atoms: RegExpDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    let constructor = [data_atom_property(
        WellKnownAtom::constructor.id(),
        Value::from_object_ref(regexp),
        writable_builtin_attributes(),
    )];
    install_intrinsic_descriptor_table(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::RegExpPrototype,
        &constructor,
    )?;

    let atom_methods = regexp_prototype_atom_method_specs(atoms)
        .map(|(atom, entry)| builtin_function_atom_property(atom, entry));
    install_intrinsic_descriptor_table(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::RegExpPrototype,
        &atom_methods,
    )?;

    let symbol_methods = regexp_prototype_symbol_method_specs().map(|(symbol, entry)| {
        builtin_function_symbol_property(symbol, entry, writable_builtin_attributes())
    });
    install_intrinsic_descriptor_table(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::RegExpPrototype,
        &symbol_methods,
    )?;

    let accessors = regexp_prototype_accessor_specs(atoms).map(|(atom, get)| {
        accessor_atom_property(atom, Some(get), None, readonly_builtin_attributes())
    });
    install_intrinsic_descriptor_table(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::RegExpPrototype,
        &accessors,
    )
}

fn install_intrinsic_descriptor_table(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    target: BuiltinIntrinsic,
    descriptors: &[BuiltinPropertyDescriptor],
) -> Result<(), BuiltinBootstrapError> {
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(target),
            descriptors,
        )],
    )
}

#[derive(Clone, Copy)]
struct RegExpDescriptorAtoms {
    escape: AtomId,
    exec: AtomId,
    test: AtomId,
    source: AtomId,
    flags: AtomId,
    has_indices: AtomId,
    global: AtomId,
    ignore_case: AtomId,
    multiline: AtomId,
    dot_all: AtomId,
    unicode: AtomId,
    sticky: AtomId,
}

impl RegExpDescriptorAtoms {
    fn new(agent: &mut Agent) -> Self {
        let bootstrap_atoms = agent.bootstrap_atoms();
        Self {
            escape: agent.atoms_mut().intern_collectible("escape"),
            exec: agent.atoms_mut().intern_collectible("exec"),
            test: agent.atoms_mut().intern_collectible("test"),
            source: bootstrap_atoms.source(),
            flags: bootstrap_atoms.flags(),
            has_indices: bootstrap_atoms.has_indices(),
            global: agent.atoms_mut().intern_collectible("global"),
            ignore_case: agent.atoms_mut().intern_collectible("ignoreCase"),
            multiline: agent.atoms_mut().intern_collectible("multiline"),
            dot_all: agent.atoms_mut().intern_collectible("dotAll"),
            unicode: agent.atoms_mut().intern_collectible("unicode"),
            sticky: agent.atoms_mut().intern_collectible("sticky"),
        }
    }
}

fn regexp_prototype_atom_method_specs(
    atoms: RegExpDescriptorAtoms,
) -> [(AtomId, BuiltinFunctionId); 3] {
    [
        (atoms.exec, js3_regexp_exec_builtin()),
        (atoms.test, js3_regexp_test_builtin()),
        (WellKnownAtom::toString.id(), js3_regexp_to_string_builtin()),
    ]
}

fn regexp_prototype_symbol_method_specs() -> [(WellKnownSymbolId, BuiltinFunctionId); 5] {
    [
        (WellKnownSymbolId::Match, js3_regexp_symbol_match_builtin()),
        (
            WellKnownSymbolId::Replace,
            js3_regexp_symbol_replace_builtin(),
        ),
        (
            WellKnownSymbolId::Search,
            js3_regexp_symbol_search_builtin(),
        ),
        (WellKnownSymbolId::Split, js3_regexp_symbol_split_builtin()),
        (
            WellKnownSymbolId::MatchAll,
            js3_regexp_symbol_match_all_builtin(),
        ),
    ]
}

fn regexp_prototype_accessor_specs(
    atoms: RegExpDescriptorAtoms,
) -> [(AtomId, BuiltinFunctionId); 9] {
    [
        (atoms.source, js3_regexp_source_getter_builtin()),
        (atoms.flags, js3_regexp_flags_getter_builtin()),
        (atoms.has_indices, js3_regexp_has_indices_getter_builtin()),
        (atoms.global, js3_regexp_global_getter_builtin()),
        (atoms.ignore_case, js3_regexp_ignore_case_getter_builtin()),
        (atoms.multiline, js3_regexp_multiline_getter_builtin()),
        (atoms.dot_all, js3_regexp_dot_all_getter_builtin()),
        (atoms.unicode, js3_regexp_unicode_getter_builtin()),
        (atoms.sticky, js3_regexp_sticky_getter_builtin()),
    ]
}

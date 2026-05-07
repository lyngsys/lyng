use super::descriptors::{
    accessor_atom_property, accessor_symbol_property, builtin_function_atom_property,
    builtin_function_symbol_property, data_atom_property, data_symbol_property, descriptor_tag,
    readonly_builtin_attributes, writable_builtin_attributes,
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
    regexp_builtin, regexp_compile_builtin, regexp_dot_all_getter_builtin, regexp_escape_builtin,
    regexp_exec_builtin, regexp_flags_getter_builtin, regexp_global_getter_builtin,
    regexp_has_indices_getter_builtin, regexp_ignore_case_getter_builtin,
    regexp_legacy_input_getter_builtin, regexp_legacy_input_setter_builtin,
    regexp_legacy_last_match_getter_builtin, regexp_legacy_last_paren_getter_builtin,
    regexp_legacy_left_context_getter_builtin, regexp_legacy_paren1_getter_builtin,
    regexp_legacy_paren2_getter_builtin, regexp_legacy_paren3_getter_builtin,
    regexp_legacy_paren4_getter_builtin, regexp_legacy_paren5_getter_builtin,
    regexp_legacy_paren6_getter_builtin, regexp_legacy_paren7_getter_builtin,
    regexp_legacy_paren8_getter_builtin, regexp_legacy_paren9_getter_builtin,
    regexp_legacy_right_context_getter_builtin, regexp_multiline_getter_builtin,
    regexp_source_getter_builtin, regexp_species_getter_builtin, regexp_sticky_getter_builtin,
    regexp_string_iterator_next_builtin, regexp_symbol_match_all_builtin,
    regexp_symbol_match_builtin, regexp_symbol_replace_builtin, regexp_symbol_search_builtin,
    regexp_symbol_split_builtin, regexp_test_builtin, regexp_to_string_builtin,
    regexp_unicode_getter_builtin, regexp_unicode_sets_getter_builtin, BuiltinFunctionId,
    ObjectRef, RealmRef, Value, WellKnownSymbolId,
};

pub(in crate::public) fn install_regexp_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototypes: RegExpFamilyPrototypes,
) -> RegExpFamilyBuiltins {
    let flag_accessors = install_regexp_flag_accessors(agent, cx);
    let legacy_accessors = install_regexp_legacy_accessors(agent, cx);
    let symbol_methods = install_regexp_symbol_methods(agent, cx);

    RegExpFamilyBuiltins {
        regexp: install_public_builtin_function(
            agent,
            cx,
            regexp_builtin(),
            Some(prototypes.regexp_prototype),
        ),
        regexp_escape: install_public_builtin_function(agent, cx, regexp_escape_builtin(), None),
        regexp_prototype: prototypes.regexp_prototype,
        regexp_compile: install_public_builtin_function(agent, cx, regexp_compile_builtin(), None),
        regexp_legacy_input_getter: legacy_accessors.input_getter,
        regexp_legacy_input_setter: legacy_accessors.input_setter,
        regexp_legacy_last_match_getter: legacy_accessors.last_match_getter,
        regexp_legacy_last_paren_getter: legacy_accessors.last_paren_getter,
        regexp_legacy_left_context_getter: legacy_accessors.left_context_getter,
        regexp_legacy_right_context_getter: legacy_accessors.right_context_getter,
        regexp_legacy_paren1_getter: legacy_accessors.paren1_getter,
        regexp_legacy_paren2_getter: legacy_accessors.paren2_getter,
        regexp_legacy_paren3_getter: legacy_accessors.paren3_getter,
        regexp_legacy_paren4_getter: legacy_accessors.paren4_getter,
        regexp_legacy_paren5_getter: legacy_accessors.paren5_getter,
        regexp_legacy_paren6_getter: legacy_accessors.paren6_getter,
        regexp_legacy_paren7_getter: legacy_accessors.paren7_getter,
        regexp_legacy_paren8_getter: legacy_accessors.paren8_getter,
        regexp_legacy_paren9_getter: legacy_accessors.paren9_getter,
        regexp_to_string: install_public_builtin_function(
            agent,
            cx,
            regexp_to_string_builtin(),
            None,
        ),
        regexp_exec: install_public_builtin_function(agent, cx, regexp_exec_builtin(), None),
        regexp_test: install_public_builtin_function(agent, cx, regexp_test_builtin(), None),
        regexp_global_getter: flag_accessors.global,
        regexp_ignore_case_getter: flag_accessors.ignore_case,
        regexp_multiline_getter: flag_accessors.multiline,
        regexp_dot_all_getter: flag_accessors.dot_all,
        regexp_unicode_getter: flag_accessors.unicode,
        regexp_unicode_sets_getter: flag_accessors.unicode_sets,
        regexp_sticky_getter: flag_accessors.sticky,
        regexp_source_getter: flag_accessors.source,
        regexp_flags_getter: flag_accessors.flags,
        regexp_has_indices_getter: flag_accessors.has_indices,
        regexp_species_getter: install_public_builtin_function(
            agent,
            cx,
            regexp_species_getter_builtin(),
            None,
        ),
        regexp_symbol_match: symbol_methods.match_method,
        regexp_symbol_replace: symbol_methods.replace,
        regexp_symbol_search: symbol_methods.search,
        regexp_symbol_split: symbol_methods.split,
        regexp_symbol_match_all: symbol_methods.match_all,
        regexp_string_iterator_next: install_public_builtin_function(
            agent,
            cx,
            regexp_string_iterator_next_builtin(),
            None,
        ),
    }
}

pub(in crate::public) fn regexp_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (regexp_builtin(), builtins.regexp),
        (regexp_escape_builtin(), builtins.regexp_escape),
        (regexp_compile_builtin(), builtins.regexp_compile),
        (
            regexp_legacy_input_getter_builtin(),
            builtins.regexp_legacy_input_getter,
        ),
        (
            regexp_legacy_input_setter_builtin(),
            builtins.regexp_legacy_input_setter,
        ),
        (
            regexp_legacy_last_match_getter_builtin(),
            builtins.regexp_legacy_last_match_getter,
        ),
        (
            regexp_legacy_last_paren_getter_builtin(),
            builtins.regexp_legacy_last_paren_getter,
        ),
        (
            regexp_legacy_left_context_getter_builtin(),
            builtins.regexp_legacy_left_context_getter,
        ),
        (
            regexp_legacy_right_context_getter_builtin(),
            builtins.regexp_legacy_right_context_getter,
        ),
        (
            regexp_legacy_paren1_getter_builtin(),
            builtins.regexp_legacy_paren1_getter,
        ),
        (
            regexp_legacy_paren2_getter_builtin(),
            builtins.regexp_legacy_paren2_getter,
        ),
        (
            regexp_legacy_paren3_getter_builtin(),
            builtins.regexp_legacy_paren3_getter,
        ),
        (
            regexp_legacy_paren4_getter_builtin(),
            builtins.regexp_legacy_paren4_getter,
        ),
        (
            regexp_legacy_paren5_getter_builtin(),
            builtins.regexp_legacy_paren5_getter,
        ),
        (
            regexp_legacy_paren6_getter_builtin(),
            builtins.regexp_legacy_paren6_getter,
        ),
        (
            regexp_legacy_paren7_getter_builtin(),
            builtins.regexp_legacy_paren7_getter,
        ),
        (
            regexp_legacy_paren8_getter_builtin(),
            builtins.regexp_legacy_paren8_getter,
        ),
        (
            regexp_legacy_paren9_getter_builtin(),
            builtins.regexp_legacy_paren9_getter,
        ),
        (regexp_to_string_builtin(), builtins.regexp_to_string),
        (regexp_exec_builtin(), builtins.regexp_exec),
        (regexp_test_builtin(), builtins.regexp_test),
        (
            regexp_global_getter_builtin(),
            builtins.regexp_global_getter,
        ),
        (
            regexp_ignore_case_getter_builtin(),
            builtins.regexp_ignore_case_getter,
        ),
        (
            regexp_multiline_getter_builtin(),
            builtins.regexp_multiline_getter,
        ),
        (
            regexp_dot_all_getter_builtin(),
            builtins.regexp_dot_all_getter,
        ),
        (
            regexp_unicode_getter_builtin(),
            builtins.regexp_unicode_getter,
        ),
        (
            regexp_unicode_sets_getter_builtin(),
            builtins.regexp_unicode_sets_getter,
        ),
        (
            regexp_sticky_getter_builtin(),
            builtins.regexp_sticky_getter,
        ),
        (
            regexp_source_getter_builtin(),
            builtins.regexp_source_getter,
        ),
        (regexp_flags_getter_builtin(), builtins.regexp_flags_getter),
        (
            regexp_has_indices_getter_builtin(),
            builtins.regexp_has_indices_getter,
        ),
        (
            regexp_species_getter_builtin(),
            builtins.regexp_species_getter,
        ),
        (regexp_symbol_match_builtin(), builtins.regexp_symbol_match),
        (
            regexp_symbol_replace_builtin(),
            builtins.regexp_symbol_replace,
        ),
        (
            regexp_symbol_search_builtin(),
            builtins.regexp_symbol_search,
        ),
        (regexp_symbol_split_builtin(), builtins.regexp_symbol_split),
        (
            regexp_symbol_match_all_builtin(),
            builtins.regexp_symbol_match_all,
        ),
        (
            regexp_string_iterator_next_builtin(),
            builtins.regexp_string_iterator_next,
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
    unicode_sets: ObjectRef,
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
        global: install_public_builtin_function(agent, cx, regexp_global_getter_builtin(), None),
        ignore_case: install_public_builtin_function(
            agent,
            cx,
            regexp_ignore_case_getter_builtin(),
            None,
        ),
        multiline: install_public_builtin_function(
            agent,
            cx,
            regexp_multiline_getter_builtin(),
            None,
        ),
        dot_all: install_public_builtin_function(agent, cx, regexp_dot_all_getter_builtin(), None),
        unicode: install_public_builtin_function(agent, cx, regexp_unicode_getter_builtin(), None),
        unicode_sets: install_public_builtin_function(
            agent,
            cx,
            regexp_unicode_sets_getter_builtin(),
            None,
        ),
        sticky: install_public_builtin_function(agent, cx, regexp_sticky_getter_builtin(), None),
        source: install_public_builtin_function(agent, cx, regexp_source_getter_builtin(), None),
        flags: install_public_builtin_function(agent, cx, regexp_flags_getter_builtin(), None),
        has_indices: install_public_builtin_function(
            agent,
            cx,
            regexp_has_indices_getter_builtin(),
            None,
        ),
    }
}

#[derive(Clone, Copy, Debug)]
struct RegExpLegacyAccessors {
    input_getter: ObjectRef,
    input_setter: ObjectRef,
    last_match_getter: ObjectRef,
    last_paren_getter: ObjectRef,
    left_context_getter: ObjectRef,
    right_context_getter: ObjectRef,
    paren1_getter: ObjectRef,
    paren2_getter: ObjectRef,
    paren3_getter: ObjectRef,
    paren4_getter: ObjectRef,
    paren5_getter: ObjectRef,
    paren6_getter: ObjectRef,
    paren7_getter: ObjectRef,
    paren8_getter: ObjectRef,
    paren9_getter: ObjectRef,
}

fn install_regexp_legacy_accessors(
    agent: &mut Agent,
    cx: FamilyInstallContext,
) -> RegExpLegacyAccessors {
    RegExpLegacyAccessors {
        input_getter: install_public_builtin_function(
            agent,
            cx,
            regexp_legacy_input_getter_builtin(),
            None,
        ),
        input_setter: install_public_builtin_function(
            agent,
            cx,
            regexp_legacy_input_setter_builtin(),
            None,
        ),
        last_match_getter: install_public_builtin_function(
            agent,
            cx,
            regexp_legacy_last_match_getter_builtin(),
            None,
        ),
        last_paren_getter: install_public_builtin_function(
            agent,
            cx,
            regexp_legacy_last_paren_getter_builtin(),
            None,
        ),
        left_context_getter: install_public_builtin_function(
            agent,
            cx,
            regexp_legacy_left_context_getter_builtin(),
            None,
        ),
        right_context_getter: install_public_builtin_function(
            agent,
            cx,
            regexp_legacy_right_context_getter_builtin(),
            None,
        ),
        paren1_getter: install_public_builtin_function(
            agent,
            cx,
            regexp_legacy_paren1_getter_builtin(),
            None,
        ),
        paren2_getter: install_public_builtin_function(
            agent,
            cx,
            regexp_legacy_paren2_getter_builtin(),
            None,
        ),
        paren3_getter: install_public_builtin_function(
            agent,
            cx,
            regexp_legacy_paren3_getter_builtin(),
            None,
        ),
        paren4_getter: install_public_builtin_function(
            agent,
            cx,
            regexp_legacy_paren4_getter_builtin(),
            None,
        ),
        paren5_getter: install_public_builtin_function(
            agent,
            cx,
            regexp_legacy_paren5_getter_builtin(),
            None,
        ),
        paren6_getter: install_public_builtin_function(
            agent,
            cx,
            regexp_legacy_paren6_getter_builtin(),
            None,
        ),
        paren7_getter: install_public_builtin_function(
            agent,
            cx,
            regexp_legacy_paren7_getter_builtin(),
            None,
        ),
        paren8_getter: install_public_builtin_function(
            agent,
            cx,
            regexp_legacy_paren8_getter_builtin(),
            None,
        ),
        paren9_getter: install_public_builtin_function(
            agent,
            cx,
            regexp_legacy_paren9_getter_builtin(),
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
            regexp_symbol_match_builtin(),
            None,
        ),
        replace: install_public_builtin_function(agent, cx, regexp_symbol_replace_builtin(), None),
        search: install_public_builtin_function(agent, cx, regexp_symbol_search_builtin(), None),
        split: install_public_builtin_function(agent, cx, regexp_symbol_split_builtin(), None),
        match_all: install_public_builtin_function(
            agent,
            cx,
            regexp_symbol_match_all_builtin(),
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
    let regexp_string_iterator_tag = descriptor_tag(agent, "RegExp String Iterator");
    install_regexp_constructor_descriptors(agent, cache, realm, atoms)?;
    install_regexp_prototype_descriptors(agent, cache, realm, builtins.regexp, atoms)?;
    install_regexp_string_iterator_prototype_descriptors(
        agent,
        cache,
        realm,
        atoms,
        regexp_string_iterator_tag,
    )
}

fn install_regexp_constructor_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    atoms: RegExpDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        builtin_function_atom_property(atoms.escape, regexp_escape_builtin()),
        accessor_symbol_property(
            WellKnownSymbolId::Species,
            Some(regexp_species_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.input,
            Some(regexp_legacy_input_getter_builtin()),
            Some(regexp_legacy_input_setter_builtin()),
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.input_alias,
            Some(regexp_legacy_input_getter_builtin()),
            Some(regexp_legacy_input_setter_builtin()),
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.last_match,
            Some(regexp_legacy_last_match_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.last_match_alias,
            Some(regexp_legacy_last_match_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.last_paren,
            Some(regexp_legacy_last_paren_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.last_paren_alias,
            Some(regexp_legacy_last_paren_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.left_context,
            Some(regexp_legacy_left_context_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.left_context_alias,
            Some(regexp_legacy_left_context_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.right_context,
            Some(regexp_legacy_right_context_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.right_context_alias,
            Some(regexp_legacy_right_context_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.paren1,
            Some(regexp_legacy_paren1_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.paren2,
            Some(regexp_legacy_paren2_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.paren3,
            Some(regexp_legacy_paren3_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.paren4,
            Some(regexp_legacy_paren4_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.paren5,
            Some(regexp_legacy_paren5_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.paren6,
            Some(regexp_legacy_paren6_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.paren7,
            Some(regexp_legacy_paren7_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.paren8,
            Some(regexp_legacy_paren8_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.paren9,
            Some(regexp_legacy_paren9_getter_builtin()),
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

fn install_regexp_string_iterator_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    atoms: RegExpDescriptorAtoms,
    regexp_string_iterator_tag: Value,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        builtin_function_atom_property(atoms.next, regexp_string_iterator_next_builtin()),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            regexp_string_iterator_tag,
            readonly_builtin_attributes(),
        ),
    ];
    install_intrinsic_descriptor_table(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::RegExpStringIteratorPrototype,
        &descriptors,
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
    next: AtomId,
    compile: AtomId,
    input: AtomId,
    input_alias: AtomId,
    last_match: AtomId,
    last_match_alias: AtomId,
    last_paren: AtomId,
    last_paren_alias: AtomId,
    left_context: AtomId,
    left_context_alias: AtomId,
    right_context: AtomId,
    right_context_alias: AtomId,
    paren1: AtomId,
    paren2: AtomId,
    paren3: AtomId,
    paren4: AtomId,
    paren5: AtomId,
    paren6: AtomId,
    paren7: AtomId,
    paren8: AtomId,
    paren9: AtomId,
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
    unicode_sets: AtomId,
    sticky: AtomId,
}

impl RegExpDescriptorAtoms {
    fn new(agent: &mut Agent) -> Self {
        let bootstrap_atoms = agent.bootstrap_atoms();
        Self {
            escape: agent.atoms_mut().intern("escape"),
            next: agent.atoms_mut().intern("next"),
            compile: agent.atoms_mut().intern("compile"),
            input: agent.atoms_mut().intern("input"),
            input_alias: agent.atoms_mut().intern("$_"),
            last_match: agent.atoms_mut().intern("lastMatch"),
            last_match_alias: agent.atoms_mut().intern("$&"),
            last_paren: agent.atoms_mut().intern("lastParen"),
            last_paren_alias: agent.atoms_mut().intern("$+"),
            left_context: agent.atoms_mut().intern("leftContext"),
            left_context_alias: agent.atoms_mut().intern("$`"),
            right_context: agent.atoms_mut().intern("rightContext"),
            right_context_alias: agent.atoms_mut().intern("$'"),
            paren1: agent.atoms_mut().intern("$1"),
            paren2: agent.atoms_mut().intern("$2"),
            paren3: agent.atoms_mut().intern("$3"),
            paren4: agent.atoms_mut().intern("$4"),
            paren5: agent.atoms_mut().intern("$5"),
            paren6: agent.atoms_mut().intern("$6"),
            paren7: agent.atoms_mut().intern("$7"),
            paren8: agent.atoms_mut().intern("$8"),
            paren9: agent.atoms_mut().intern("$9"),
            exec: agent.atoms_mut().intern("exec"),
            test: agent.atoms_mut().intern("test"),
            source: bootstrap_atoms.source(),
            flags: bootstrap_atoms.flags(),
            has_indices: bootstrap_atoms.has_indices(),
            global: agent.atoms_mut().intern("global"),
            ignore_case: agent.atoms_mut().intern("ignoreCase"),
            multiline: agent.atoms_mut().intern("multiline"),
            dot_all: agent.atoms_mut().intern("dotAll"),
            unicode: agent.atoms_mut().intern("unicode"),
            unicode_sets: agent.atoms_mut().intern("unicodeSets"),
            sticky: agent.atoms_mut().intern("sticky"),
        }
    }
}

const fn regexp_prototype_atom_method_specs(
    atoms: RegExpDescriptorAtoms,
) -> [(AtomId, BuiltinFunctionId); 4] {
    [
        (atoms.compile, regexp_compile_builtin()),
        (atoms.exec, regexp_exec_builtin()),
        (atoms.test, regexp_test_builtin()),
        (WellKnownAtom::toString.id(), regexp_to_string_builtin()),
    ]
}

const fn regexp_prototype_symbol_method_specs() -> [(WellKnownSymbolId, BuiltinFunctionId); 5] {
    [
        (WellKnownSymbolId::Match, regexp_symbol_match_builtin()),
        (WellKnownSymbolId::Replace, regexp_symbol_replace_builtin()),
        (WellKnownSymbolId::Search, regexp_symbol_search_builtin()),
        (WellKnownSymbolId::Split, regexp_symbol_split_builtin()),
        (
            WellKnownSymbolId::MatchAll,
            regexp_symbol_match_all_builtin(),
        ),
    ]
}

const fn regexp_prototype_accessor_specs(
    atoms: RegExpDescriptorAtoms,
) -> [(AtomId, BuiltinFunctionId); 10] {
    [
        (atoms.source, regexp_source_getter_builtin()),
        (atoms.flags, regexp_flags_getter_builtin()),
        (atoms.has_indices, regexp_has_indices_getter_builtin()),
        (atoms.global, regexp_global_getter_builtin()),
        (atoms.ignore_case, regexp_ignore_case_getter_builtin()),
        (atoms.multiline, regexp_multiline_getter_builtin()),
        (atoms.dot_all, regexp_dot_all_getter_builtin()),
        (atoms.unicode, regexp_unicode_getter_builtin()),
        (atoms.unicode_sets, regexp_unicode_sets_getter_builtin()),
        (atoms.sticky, regexp_sticky_getter_builtin()),
    ]
}

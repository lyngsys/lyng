use super::descriptors::{
    builtin_function_atom_property, builtin_function_symbol_property, data_atom_property,
    data_symbol_property, descriptor_tag, readonly_builtin_attributes, writable_builtin_attributes,
};
use super::{
    install_public_builtin_function, FamilyInstallContext, StringFamilyBuiltins,
    StringFamilyPrototypes,
};
use crate::bootstrap::{install_descriptor_tables, BuiltinBootstrapError};
use crate::public::{BuiltinCache, PublicRealmBuiltins};
use crate::{
    BuiltinDescriptorTable, BuiltinInstallTarget, BuiltinIntrinsic, BuiltinPropertyDescriptor,
};
use lyng_js_common::{AtomId, WellKnownAtom};
use lyng_js_env::Agent;
use lyng_js_types::{
    string_at_builtin, string_builtin, string_char_at_builtin, string_char_code_at_builtin,
    string_code_point_at_builtin, string_concat_builtin, string_ends_with_builtin,
    string_from_char_code_builtin, string_from_code_point_builtin, string_includes_builtin,
    string_index_of_builtin, string_is_well_formed_builtin, string_iterator_builtin,
    string_iterator_next_builtin, string_last_index_of_builtin, string_locale_compare_builtin,
    string_match_all_builtin, string_match_builtin, string_normalize_builtin,
    string_pad_end_builtin, string_pad_start_builtin, string_raw_builtin, string_repeat_builtin,
    string_replace_all_builtin, string_replace_builtin, string_search_builtin,
    string_slice_builtin, string_split_builtin, string_starts_with_builtin,
    string_substring_builtin, string_to_locale_lower_case_builtin,
    string_to_locale_upper_case_builtin, string_to_lower_case_builtin, string_to_string_builtin,
    string_to_upper_case_builtin, string_to_well_formed_builtin, string_trim_builtin,
    string_trim_end_builtin, string_trim_start_builtin, string_value_of_builtin, BuiltinFunctionId,
    ObjectRef, RealmRef, Value, WellKnownSymbolId,
};

#[allow(clippy::too_many_lines)]
pub(in crate::public) fn install_string_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototypes: StringFamilyPrototypes,
) -> StringFamilyBuiltins {
    StringFamilyBuiltins {
        string: install_public_builtin_function(
            agent,
            cx,
            string_builtin(),
            Some(prototypes.string_prototype),
        ),
        string_prototype: prototypes.string_prototype,
        string_iterator: install_public_builtin_function(
            agent,
            cx,
            string_iterator_builtin(),
            None,
        ),
        string_iterator_next: install_public_builtin_function(
            agent,
            cx,
            string_iterator_next_builtin(),
            None,
        ),
        string_to_string: install_public_builtin_function(
            agent,
            cx,
            string_to_string_builtin(),
            None,
        ),
        string_value_of: install_public_builtin_function(
            agent,
            cx,
            string_value_of_builtin(),
            None,
        ),
        string_concat: install_public_builtin_function(agent, cx, string_concat_builtin(), None),
        string_char_at: install_public_builtin_function(agent, cx, string_char_at_builtin(), None),
        string_char_code_at: install_public_builtin_function(
            agent,
            cx,
            string_char_code_at_builtin(),
            None,
        ),
        string_from_char_code: install_public_builtin_function(
            agent,
            cx,
            string_from_char_code_builtin(),
            None,
        ),
        string_from_code_point: install_public_builtin_function(
            agent,
            cx,
            string_from_code_point_builtin(),
            None,
        ),
        string_raw: install_public_builtin_function(agent, cx, string_raw_builtin(), None),
        string_at: install_public_builtin_function(agent, cx, string_at_builtin(), None),
        string_code_point_at: install_public_builtin_function(
            agent,
            cx,
            string_code_point_at_builtin(),
            None,
        ),
        string_ends_with: install_public_builtin_function(
            agent,
            cx,
            string_ends_with_builtin(),
            None,
        ),
        string_includes: install_public_builtin_function(
            agent,
            cx,
            string_includes_builtin(),
            None,
        ),
        string_index_of: install_public_builtin_function(
            agent,
            cx,
            string_index_of_builtin(),
            None,
        ),
        string_is_well_formed: install_public_builtin_function(
            agent,
            cx,
            string_is_well_formed_builtin(),
            None,
        ),
        string_locale_compare: install_public_builtin_function(
            agent,
            cx,
            string_locale_compare_builtin(),
            None,
        ),
        string_match: install_public_builtin_function(agent, cx, string_match_builtin(), None),
        string_match_all: install_public_builtin_function(
            agent,
            cx,
            string_match_all_builtin(),
            None,
        ),
        string_normalize: install_public_builtin_function(
            agent,
            cx,
            string_normalize_builtin(),
            None,
        ),
        string_last_index_of: install_public_builtin_function(
            agent,
            cx,
            string_last_index_of_builtin(),
            None,
        ),
        string_pad_end: install_public_builtin_function(agent, cx, string_pad_end_builtin(), None),
        string_pad_start: install_public_builtin_function(
            agent,
            cx,
            string_pad_start_builtin(),
            None,
        ),
        string_repeat: install_public_builtin_function(agent, cx, string_repeat_builtin(), None),
        string_replace: install_public_builtin_function(agent, cx, string_replace_builtin(), None),
        string_replace_all: install_public_builtin_function(
            agent,
            cx,
            string_replace_all_builtin(),
            None,
        ),
        string_search: install_public_builtin_function(agent, cx, string_search_builtin(), None),
        string_split: install_public_builtin_function(agent, cx, string_split_builtin(), None),
        string_slice: install_public_builtin_function(agent, cx, string_slice_builtin(), None),
        string_substring: install_public_builtin_function(
            agent,
            cx,
            string_substring_builtin(),
            None,
        ),
        string_starts_with: install_public_builtin_function(
            agent,
            cx,
            string_starts_with_builtin(),
            None,
        ),
        string_to_locale_lower_case: install_public_builtin_function(
            agent,
            cx,
            string_to_locale_lower_case_builtin(),
            None,
        ),
        string_to_locale_upper_case: install_public_builtin_function(
            agent,
            cx,
            string_to_locale_upper_case_builtin(),
            None,
        ),
        string_to_lower_case: install_public_builtin_function(
            agent,
            cx,
            string_to_lower_case_builtin(),
            None,
        ),
        string_to_upper_case: install_public_builtin_function(
            agent,
            cx,
            string_to_upper_case_builtin(),
            None,
        ),
        string_to_well_formed: install_public_builtin_function(
            agent,
            cx,
            string_to_well_formed_builtin(),
            None,
        ),
        string_trim: install_public_builtin_function(agent, cx, string_trim_builtin(), None),
        string_trim_end: install_public_builtin_function(
            agent,
            cx,
            string_trim_end_builtin(),
            None,
        ),
        string_trim_start: install_public_builtin_function(
            agent,
            cx,
            string_trim_start_builtin(),
            None,
        ),
    }
}

pub(in crate::public) fn string_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (string_builtin(), builtins.string),
        (string_iterator_builtin(), builtins.string_iterator),
        (
            string_iterator_next_builtin(),
            builtins.string_iterator_next,
        ),
        (string_to_string_builtin(), builtins.string_to_string),
        (string_value_of_builtin(), builtins.string_value_of),
        (string_concat_builtin(), builtins.string_concat),
        (string_char_at_builtin(), builtins.string_char_at),
        (string_char_code_at_builtin(), builtins.string_char_code_at),
        (
            string_from_char_code_builtin(),
            builtins.string_from_char_code,
        ),
        (
            string_from_code_point_builtin(),
            builtins.string_from_code_point,
        ),
        (string_raw_builtin(), builtins.string_raw),
        (string_at_builtin(), builtins.string_at),
        (
            string_code_point_at_builtin(),
            builtins.string_code_point_at,
        ),
        (string_ends_with_builtin(), builtins.string_ends_with),
        (string_includes_builtin(), builtins.string_includes),
        (string_index_of_builtin(), builtins.string_index_of),
        (
            string_is_well_formed_builtin(),
            builtins.string_is_well_formed,
        ),
        (
            string_locale_compare_builtin(),
            builtins.string_locale_compare,
        ),
        (string_match_builtin(), builtins.string_match),
        (string_match_all_builtin(), builtins.string_match_all),
        (string_normalize_builtin(), builtins.string_normalize),
        (
            string_last_index_of_builtin(),
            builtins.string_last_index_of,
        ),
        (string_pad_end_builtin(), builtins.string_pad_end),
        (string_pad_start_builtin(), builtins.string_pad_start),
        (string_repeat_builtin(), builtins.string_repeat),
        (string_replace_builtin(), builtins.string_replace),
        (string_replace_all_builtin(), builtins.string_replace_all),
        (string_search_builtin(), builtins.string_search),
        (string_split_builtin(), builtins.string_split),
        (string_slice_builtin(), builtins.string_slice),
        (string_substring_builtin(), builtins.string_substring),
        (string_starts_with_builtin(), builtins.string_starts_with),
        (
            string_to_locale_lower_case_builtin(),
            builtins.string_to_locale_lower_case,
        ),
        (
            string_to_locale_upper_case_builtin(),
            builtins.string_to_locale_upper_case,
        ),
        (
            string_to_lower_case_builtin(),
            builtins.string_to_lower_case,
        ),
        (
            string_to_upper_case_builtin(),
            builtins.string_to_upper_case,
        ),
        (
            string_to_well_formed_builtin(),
            builtins.string_to_well_formed,
        ),
        (string_trim_builtin(), builtins.string_trim),
        (string_trim_end_builtin(), builtins.string_trim_end),
        (string_trim_start_builtin(), builtins.string_trim_start),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

pub(in crate::public) fn install_string_family_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
) -> Result<(), BuiltinBootstrapError> {
    let atoms = StringDescriptorAtoms::new(agent);
    install_string_constructor_descriptors(agent, cache, realm, atoms)?;
    install_string_prototype_descriptors(agent, cache, realm, builtins.string, atoms)?;
    install_string_iterator_prototype_descriptors(agent, cache, realm, atoms)
}

fn install_string_constructor_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    atoms: StringDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = string_constructor_method_specs(atoms)
        .map(|(atom, entry)| builtin_function_atom_property(atom, entry));
    install_intrinsic_descriptor_table(agent, cache, realm, BuiltinIntrinsic::String, &descriptors)
}

fn install_string_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    string: ObjectRef,
    atoms: StringDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    let constructor = [data_atom_property(
        WellKnownAtom::constructor.id(),
        Value::from_object_ref(string),
        writable_builtin_attributes(),
    )];
    install_intrinsic_descriptor_table(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::StringPrototype,
        &constructor,
    )?;

    let methods = string_prototype_method_specs(atoms)
        .map(|(atom, entry)| builtin_function_atom_property(atom, entry));
    install_intrinsic_descriptor_table(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::StringPrototype,
        &methods,
    )?;

    let iterator = [builtin_function_symbol_property(
        WellKnownSymbolId::Iterator,
        string_iterator_builtin(),
        writable_builtin_attributes(),
    )];
    install_intrinsic_descriptor_table(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::StringPrototype,
        &iterator,
    )
}

fn install_string_iterator_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    atoms: StringDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        builtin_function_atom_property(atoms.next, string_iterator_next_builtin()),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            atoms.string_iterator_tag,
            readonly_builtin_attributes(),
        ),
    ];
    install_intrinsic_descriptor_table(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::StringIteratorPrototype,
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
struct StringDescriptorAtoms {
    from_char_code: AtomId,
    from_code_point: AtomId,
    raw: AtomId,
    concat: AtomId,
    char_at: AtomId,
    char_code_at: AtomId,
    at: AtomId,
    code_point_at: AtomId,
    ends_with: AtomId,
    includes: AtomId,
    index_of: AtomId,
    is_well_formed: AtomId,
    locale_compare: AtomId,
    match_: AtomId,
    match_all: AtomId,
    normalize: AtomId,
    last_index_of: AtomId,
    pad_end: AtomId,
    pad_start: AtomId,
    repeat: AtomId,
    replace: AtomId,
    replace_all: AtomId,
    search: AtomId,
    split: AtomId,
    slice: AtomId,
    substring: AtomId,
    starts_with: AtomId,
    to_locale_lower_case: AtomId,
    to_locale_upper_case: AtomId,
    to_lower_case: AtomId,
    to_upper_case: AtomId,
    to_well_formed: AtomId,
    trim: AtomId,
    trim_end: AtomId,
    trim_start: AtomId,
    next: AtomId,
    string_iterator_tag: Value,
}

impl StringDescriptorAtoms {
    fn new(agent: &mut Agent) -> Self {
        Self {
            from_char_code: agent.atoms_mut().intern("fromCharCode"),
            from_code_point: agent.atoms_mut().intern("fromCodePoint"),
            raw: agent.atoms_mut().intern("raw"),
            concat: agent.atoms_mut().intern("concat"),
            char_at: agent.atoms_mut().intern("charAt"),
            char_code_at: agent.atoms_mut().intern("charCodeAt"),
            at: agent.atoms_mut().intern("at"),
            code_point_at: agent.atoms_mut().intern("codePointAt"),
            ends_with: agent.atoms_mut().intern("endsWith"),
            includes: agent.atoms_mut().intern("includes"),
            index_of: agent.atoms_mut().intern("indexOf"),
            is_well_formed: agent.atoms_mut().intern("isWellFormed"),
            locale_compare: agent.atoms_mut().intern("localeCompare"),
            match_: agent.atoms_mut().intern("match"),
            match_all: agent.atoms_mut().intern("matchAll"),
            normalize: agent.atoms_mut().intern("normalize"),
            last_index_of: agent.atoms_mut().intern("lastIndexOf"),
            pad_end: agent.atoms_mut().intern("padEnd"),
            pad_start: agent.atoms_mut().intern("padStart"),
            repeat: agent.atoms_mut().intern("repeat"),
            replace: agent.atoms_mut().intern("replace"),
            replace_all: agent.atoms_mut().intern("replaceAll"),
            search: agent.atoms_mut().intern("search"),
            split: agent.atoms_mut().intern("split"),
            slice: agent.atoms_mut().intern("slice"),
            substring: agent.atoms_mut().intern("substring"),
            starts_with: agent.atoms_mut().intern("startsWith"),
            to_locale_lower_case: agent.atoms_mut().intern("toLocaleLowerCase"),
            to_locale_upper_case: agent.atoms_mut().intern("toLocaleUpperCase"),
            to_lower_case: agent.atoms_mut().intern("toLowerCase"),
            to_upper_case: agent.atoms_mut().intern("toUpperCase"),
            to_well_formed: agent.atoms_mut().intern("toWellFormed"),
            trim: agent.atoms_mut().intern("trim"),
            trim_end: agent.atoms_mut().intern("trimEnd"),
            trim_start: agent.atoms_mut().intern("trimStart"),
            next: agent.atoms_mut().intern("next"),
            string_iterator_tag: descriptor_tag(agent, "String Iterator"),
        }
    }
}

fn string_constructor_method_specs(
    atoms: StringDescriptorAtoms,
) -> [(AtomId, BuiltinFunctionId); 3] {
    [
        (atoms.from_char_code, string_from_char_code_builtin()),
        (atoms.from_code_point, string_from_code_point_builtin()),
        (atoms.raw, string_raw_builtin()),
    ]
}

fn string_prototype_method_specs(
    atoms: StringDescriptorAtoms,
) -> [(AtomId, BuiltinFunctionId); 34] {
    [
        (WellKnownAtom::toString.id(), string_to_string_builtin()),
        (WellKnownAtom::valueOf.id(), string_value_of_builtin()),
        (atoms.concat, string_concat_builtin()),
        (atoms.char_at, string_char_at_builtin()),
        (atoms.char_code_at, string_char_code_at_builtin()),
        (atoms.at, string_at_builtin()),
        (atoms.code_point_at, string_code_point_at_builtin()),
        (atoms.ends_with, string_ends_with_builtin()),
        (atoms.includes, string_includes_builtin()),
        (atoms.index_of, string_index_of_builtin()),
        (atoms.is_well_formed, string_is_well_formed_builtin()),
        (atoms.locale_compare, string_locale_compare_builtin()),
        (atoms.match_, string_match_builtin()),
        (atoms.match_all, string_match_all_builtin()),
        (atoms.normalize, string_normalize_builtin()),
        (atoms.last_index_of, string_last_index_of_builtin()),
        (atoms.pad_end, string_pad_end_builtin()),
        (atoms.pad_start, string_pad_start_builtin()),
        (atoms.repeat, string_repeat_builtin()),
        (atoms.replace, string_replace_builtin()),
        (atoms.replace_all, string_replace_all_builtin()),
        (atoms.search, string_search_builtin()),
        (atoms.split, string_split_builtin()),
        (atoms.slice, string_slice_builtin()),
        (atoms.substring, string_substring_builtin()),
        (atoms.starts_with, string_starts_with_builtin()),
        (
            atoms.to_locale_lower_case,
            string_to_locale_lower_case_builtin(),
        ),
        (
            atoms.to_locale_upper_case,
            string_to_locale_upper_case_builtin(),
        ),
        (atoms.to_lower_case, string_to_lower_case_builtin()),
        (atoms.to_upper_case, string_to_upper_case_builtin()),
        (atoms.to_well_formed, string_to_well_formed_builtin()),
        (atoms.trim, string_trim_builtin()),
        (atoms.trim_end, string_trim_end_builtin()),
        (atoms.trim_start, string_trim_start_builtin()),
    ]
}

use super::{
    install_public_builtin_function, FamilyInstallContext, RegExpFamilyBuiltins,
    RegExpFamilyPrototypes,
};
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
    js3_regexp_unicode_getter_builtin, BuiltinFunctionId, ObjectRef,
};

use crate::public::PublicRealmBuiltins;

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

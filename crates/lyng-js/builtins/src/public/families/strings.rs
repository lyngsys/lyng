use super::{
    install_public_builtin_function, FamilyInstallContext, StringFamilyBuiltins,
    StringFamilyPrototypes,
};
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_string_at_builtin, js3_string_builtin, js3_string_char_at_builtin,
    js3_string_char_code_at_builtin, js3_string_code_point_at_builtin, js3_string_concat_builtin,
    js3_string_ends_with_builtin, js3_string_from_char_code_builtin,
    js3_string_from_code_point_builtin, js3_string_includes_builtin, js3_string_index_of_builtin,
    js3_string_is_well_formed_builtin, js3_string_iterator_builtin,
    js3_string_iterator_next_builtin, js3_string_last_index_of_builtin,
    js3_string_locale_compare_builtin, js3_string_match_all_builtin, js3_string_match_builtin,
    js3_string_normalize_builtin, js3_string_pad_end_builtin, js3_string_pad_start_builtin,
    js3_string_raw_builtin, js3_string_repeat_builtin, js3_string_replace_all_builtin,
    js3_string_replace_builtin, js3_string_search_builtin, js3_string_slice_builtin,
    js3_string_split_builtin, js3_string_starts_with_builtin, js3_string_substring_builtin,
    js3_string_to_locale_lower_case_builtin, js3_string_to_locale_upper_case_builtin,
    js3_string_to_lower_case_builtin, js3_string_to_string_builtin,
    js3_string_to_upper_case_builtin, js3_string_to_well_formed_builtin, js3_string_trim_builtin,
    js3_string_trim_end_builtin, js3_string_trim_start_builtin, js3_string_value_of_builtin,
    BuiltinFunctionId, ObjectRef,
};

use crate::public::PublicRealmBuiltins;

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
            js3_string_builtin(),
            Some(prototypes.string_prototype),
        ),
        string_prototype: prototypes.string_prototype,
        string_iterator: install_public_builtin_function(
            agent,
            cx,
            js3_string_iterator_builtin(),
            None,
        ),
        string_iterator_next: install_public_builtin_function(
            agent,
            cx,
            js3_string_iterator_next_builtin(),
            None,
        ),
        string_to_string: install_public_builtin_function(
            agent,
            cx,
            js3_string_to_string_builtin(),
            None,
        ),
        string_value_of: install_public_builtin_function(
            agent,
            cx,
            js3_string_value_of_builtin(),
            None,
        ),
        string_concat: install_public_builtin_function(
            agent,
            cx,
            js3_string_concat_builtin(),
            None,
        ),
        string_char_at: install_public_builtin_function(
            agent,
            cx,
            js3_string_char_at_builtin(),
            None,
        ),
        string_char_code_at: install_public_builtin_function(
            agent,
            cx,
            js3_string_char_code_at_builtin(),
            None,
        ),
        string_from_char_code: install_public_builtin_function(
            agent,
            cx,
            js3_string_from_char_code_builtin(),
            None,
        ),
        string_from_code_point: install_public_builtin_function(
            agent,
            cx,
            js3_string_from_code_point_builtin(),
            None,
        ),
        string_raw: install_public_builtin_function(agent, cx, js3_string_raw_builtin(), None),
        string_at: install_public_builtin_function(agent, cx, js3_string_at_builtin(), None),
        string_code_point_at: install_public_builtin_function(
            agent,
            cx,
            js3_string_code_point_at_builtin(),
            None,
        ),
        string_ends_with: install_public_builtin_function(
            agent,
            cx,
            js3_string_ends_with_builtin(),
            None,
        ),
        string_includes: install_public_builtin_function(
            agent,
            cx,
            js3_string_includes_builtin(),
            None,
        ),
        string_index_of: install_public_builtin_function(
            agent,
            cx,
            js3_string_index_of_builtin(),
            None,
        ),
        string_is_well_formed: install_public_builtin_function(
            agent,
            cx,
            js3_string_is_well_formed_builtin(),
            None,
        ),
        string_locale_compare: install_public_builtin_function(
            agent,
            cx,
            js3_string_locale_compare_builtin(),
            None,
        ),
        string_match: install_public_builtin_function(agent, cx, js3_string_match_builtin(), None),
        string_match_all: install_public_builtin_function(
            agent,
            cx,
            js3_string_match_all_builtin(),
            None,
        ),
        string_normalize: install_public_builtin_function(
            agent,
            cx,
            js3_string_normalize_builtin(),
            None,
        ),
        string_last_index_of: install_public_builtin_function(
            agent,
            cx,
            js3_string_last_index_of_builtin(),
            None,
        ),
        string_pad_end: install_public_builtin_function(
            agent,
            cx,
            js3_string_pad_end_builtin(),
            None,
        ),
        string_pad_start: install_public_builtin_function(
            agent,
            cx,
            js3_string_pad_start_builtin(),
            None,
        ),
        string_repeat: install_public_builtin_function(
            agent,
            cx,
            js3_string_repeat_builtin(),
            None,
        ),
        string_replace: install_public_builtin_function(
            agent,
            cx,
            js3_string_replace_builtin(),
            None,
        ),
        string_replace_all: install_public_builtin_function(
            agent,
            cx,
            js3_string_replace_all_builtin(),
            None,
        ),
        string_search: install_public_builtin_function(
            agent,
            cx,
            js3_string_search_builtin(),
            None,
        ),
        string_split: install_public_builtin_function(agent, cx, js3_string_split_builtin(), None),
        string_slice: install_public_builtin_function(agent, cx, js3_string_slice_builtin(), None),
        string_substring: install_public_builtin_function(
            agent,
            cx,
            js3_string_substring_builtin(),
            None,
        ),
        string_starts_with: install_public_builtin_function(
            agent,
            cx,
            js3_string_starts_with_builtin(),
            None,
        ),
        string_to_locale_lower_case: install_public_builtin_function(
            agent,
            cx,
            js3_string_to_locale_lower_case_builtin(),
            None,
        ),
        string_to_locale_upper_case: install_public_builtin_function(
            agent,
            cx,
            js3_string_to_locale_upper_case_builtin(),
            None,
        ),
        string_to_lower_case: install_public_builtin_function(
            agent,
            cx,
            js3_string_to_lower_case_builtin(),
            None,
        ),
        string_to_upper_case: install_public_builtin_function(
            agent,
            cx,
            js3_string_to_upper_case_builtin(),
            None,
        ),
        string_to_well_formed: install_public_builtin_function(
            agent,
            cx,
            js3_string_to_well_formed_builtin(),
            None,
        ),
        string_trim: install_public_builtin_function(agent, cx, js3_string_trim_builtin(), None),
        string_trim_end: install_public_builtin_function(
            agent,
            cx,
            js3_string_trim_end_builtin(),
            None,
        ),
        string_trim_start: install_public_builtin_function(
            agent,
            cx,
            js3_string_trim_start_builtin(),
            None,
        ),
    }
}

pub(in crate::public) fn string_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (js3_string_builtin(), builtins.string),
        (js3_string_iterator_builtin(), builtins.string_iterator),
        (
            js3_string_iterator_next_builtin(),
            builtins.string_iterator_next,
        ),
        (js3_string_to_string_builtin(), builtins.string_to_string),
        (js3_string_value_of_builtin(), builtins.string_value_of),
        (js3_string_concat_builtin(), builtins.string_concat),
        (js3_string_char_at_builtin(), builtins.string_char_at),
        (
            js3_string_char_code_at_builtin(),
            builtins.string_char_code_at,
        ),
        (
            js3_string_from_char_code_builtin(),
            builtins.string_from_char_code,
        ),
        (
            js3_string_from_code_point_builtin(),
            builtins.string_from_code_point,
        ),
        (js3_string_raw_builtin(), builtins.string_raw),
        (js3_string_at_builtin(), builtins.string_at),
        (
            js3_string_code_point_at_builtin(),
            builtins.string_code_point_at,
        ),
        (js3_string_ends_with_builtin(), builtins.string_ends_with),
        (js3_string_includes_builtin(), builtins.string_includes),
        (js3_string_index_of_builtin(), builtins.string_index_of),
        (
            js3_string_is_well_formed_builtin(),
            builtins.string_is_well_formed,
        ),
        (
            js3_string_locale_compare_builtin(),
            builtins.string_locale_compare,
        ),
        (js3_string_match_builtin(), builtins.string_match),
        (js3_string_match_all_builtin(), builtins.string_match_all),
        (js3_string_normalize_builtin(), builtins.string_normalize),
        (
            js3_string_last_index_of_builtin(),
            builtins.string_last_index_of,
        ),
        (js3_string_pad_end_builtin(), builtins.string_pad_end),
        (js3_string_pad_start_builtin(), builtins.string_pad_start),
        (js3_string_repeat_builtin(), builtins.string_repeat),
        (js3_string_replace_builtin(), builtins.string_replace),
        (
            js3_string_replace_all_builtin(),
            builtins.string_replace_all,
        ),
        (js3_string_search_builtin(), builtins.string_search),
        (js3_string_split_builtin(), builtins.string_split),
        (js3_string_slice_builtin(), builtins.string_slice),
        (js3_string_substring_builtin(), builtins.string_substring),
        (
            js3_string_starts_with_builtin(),
            builtins.string_starts_with,
        ),
        (
            js3_string_to_locale_lower_case_builtin(),
            builtins.string_to_locale_lower_case,
        ),
        (
            js3_string_to_locale_upper_case_builtin(),
            builtins.string_to_locale_upper_case,
        ),
        (
            js3_string_to_lower_case_builtin(),
            builtins.string_to_lower_case,
        ),
        (
            js3_string_to_upper_case_builtin(),
            builtins.string_to_upper_case,
        ),
        (
            js3_string_to_well_formed_builtin(),
            builtins.string_to_well_formed,
        ),
        (js3_string_trim_builtin(), builtins.string_trim),
        (js3_string_trim_end_builtin(), builtins.string_trim_end),
        (js3_string_trim_start_builtin(), builtins.string_trim_start),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

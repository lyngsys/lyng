use super::{
    iterators::{string_iterator_builtin, string_iterator_next_builtin},
    PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_types::{BuiltinFunctionId, Value};

pub(super) fn dispatch_string_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_string_constructor_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_string_iterator_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_string_basic_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_string_search_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_string_transform_builtin(context, entry, invocation)
}

fn dispatch_string_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_string_builtin() {
        return super::string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_from_char_code_builtin() {
        return super::string_from_char_code_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_from_code_point_builtin() {
        return super::string_from_code_point_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_raw_builtin() {
        return super::string_raw_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_string_iterator_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_string_iterator_builtin() {
        return string_iterator_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_iterator_next_builtin() {
        return string_iterator_next_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_string_basic_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_string_to_string_builtin() {
        return super::string_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_value_of_builtin() {
        return super::string_value_of_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_concat_builtin() {
        return super::string_concat_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_char_at_builtin() {
        return super::string_char_at_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_char_code_at_builtin() {
        return super::string_char_code_at_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_at_builtin() {
        return super::string_at_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_code_point_at_builtin() {
        return super::string_code_point_at_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_string_search_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_string_ends_with_builtin() {
        return super::string_ends_with_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_includes_builtin() {
        return super::string_includes_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_index_of_builtin() {
        return super::string_index_of_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_locale_compare_builtin() {
        return super::string_locale_compare_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_match_builtin() {
        return super::string_match_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_match_all_builtin() {
        return super::string_match_all_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_last_index_of_builtin() {
        return super::string_last_index_of_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_replace_builtin() {
        return super::string_replace_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_replace_all_builtin() {
        return super::string_replace_all_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_search_builtin() {
        return super::string_search_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_split_builtin() {
        return super::string_split_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_starts_with_builtin() {
        return super::string_starts_with_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_string_transform_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_string_is_well_formed_builtin() {
        return super::string_is_well_formed_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_normalize_builtin() {
        return super::string_normalize_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_pad_end_builtin() {
        return super::string_pad_end_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_pad_start_builtin() {
        return super::string_pad_start_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_repeat_builtin() {
        return super::string_repeat_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_slice_builtin() {
        return super::string_slice_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_substring_builtin() {
        return super::string_substring_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_to_locale_lower_case_builtin() {
        return super::string_case_mapping_builtin(
            context,
            invocation,
            super::StringCaseMapping::Lower,
        )
        .map(Some);
    }
    if entry == super::js3_string_to_locale_upper_case_builtin() {
        return super::string_case_mapping_builtin(
            context,
            invocation,
            super::StringCaseMapping::Upper,
        )
        .map(Some);
    }
    if entry == super::js3_string_to_lower_case_builtin() {
        return super::string_case_mapping_builtin(
            context,
            invocation,
            super::StringCaseMapping::Lower,
        )
        .map(Some);
    }
    if entry == super::js3_string_to_upper_case_builtin() {
        return super::string_case_mapping_builtin(
            context,
            invocation,
            super::StringCaseMapping::Upper,
        )
        .map(Some);
    }
    if entry == super::js3_string_to_well_formed_builtin() {
        return super::string_to_well_formed_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_string_trim_builtin() {
        return super::string_trim_builtin(context, invocation, true, true).map(Some);
    }
    if entry == super::js3_string_trim_end_builtin() {
        return super::string_trim_builtin(context, invocation, false, true).map(Some);
    }
    if entry == super::js3_string_trim_start_builtin() {
        return super::string_trim_builtin(context, invocation, true, false).map(Some);
    }
    Ok(None)
}

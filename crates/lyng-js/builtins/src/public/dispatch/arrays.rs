use super::{
    iterators::{array_iterator_factory_builtin, array_iterator_next_builtin, ArrayIterationKind},
    PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_types::{BuiltinFunctionId, Value};

pub(super) fn dispatch_array_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_array_constructor_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_array_indexed_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_array_iteration_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_array_copying_builtin(context, entry, invocation)
}

fn dispatch_array_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_array_builtin() {
        return super::array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_from_builtin() {
        return super::array_from_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_from_async_builtin() {
        return super::array_from_async_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_of_builtin() {
        return super::array_of_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_is_array_builtin() {
        return super::array_is_array_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_species_getter_builtin() {
        return super::array_species_getter_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_array_indexed_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_array_at_builtin() {
        return super::array_at_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_concat_builtin() {
        return super::array_concat_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_copy_within_builtin() {
        return super::array_copy_within_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_fill_builtin() {
        return super::array_fill_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_join_builtin() {
        return super::array_join_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_pop_builtin() {
        return super::array_pop_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_push_builtin() {
        return super::array_push_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_shift_builtin() {
        return super::array_shift_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_unshift_builtin() {
        return super::array_unshift_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_reverse_builtin() {
        return super::array_reverse_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_slice_builtin() {
        return super::array_slice_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_sort_builtin() {
        return super::array_sort_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_splice_builtin() {
        return super::array_splice_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_array_iteration_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_array_every_builtin() {
        return super::array_every_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_filter_builtin() {
        return super::array_filter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_flat_builtin() {
        return super::array_flat_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_flat_map_builtin() {
        return super::array_flat_map_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_find_builtin() {
        return super::array_find_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_find_index_builtin() {
        return super::array_find_index_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_find_last_builtin() {
        return super::array_find_last_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_find_last_index_builtin() {
        return super::array_find_last_index_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_for_each_builtin() {
        return super::array_for_each_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_includes_builtin() {
        return super::array_includes_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_index_of_builtin() {
        return super::array_index_of_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_last_index_of_builtin() {
        return super::array_last_index_of_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_map_builtin() {
        return super::array_map_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_reduce_builtin() {
        return super::array_reduce_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_reduce_right_builtin() {
        return super::array_reduce_right_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_some_builtin() {
        return super::array_some_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_array_copying_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_array_to_reversed_builtin() {
        return super::array_to_reversed_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_to_sorted_builtin() {
        return super::array_to_sorted_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_to_spliced_builtin() {
        return super::array_to_spliced_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_to_string_builtin() {
        return super::array_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_to_locale_string_builtin() {
        return super::array_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_values_builtin() {
        return array_iterator_factory_builtin(context, invocation, ArrayIterationKind::Value)
            .map(Some);
    }
    if entry == super::js3_array_keys_builtin() {
        return array_iterator_factory_builtin(context, invocation, ArrayIterationKind::Key)
            .map(Some);
    }
    if entry == super::js3_array_entries_builtin() {
        return array_iterator_factory_builtin(context, invocation, ArrayIterationKind::Entry)
            .map(Some);
    }
    if entry == super::js3_array_iterator_next_builtin() {
        return array_iterator_next_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_array_with_builtin() {
        return super::array_with_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

use super::{
    array_like_length,
    binary_data::{typed_array_is_out_of_bounds, typed_array_validated_object_and_record},
    close_iterator_after_error, create_array_from_values, create_array_result,
    create_data_property_or_throw, define_data_property_with_attrs, get_property_from_object,
    length_value, length_value_u64, map_completion, number_to_u32_after_range_check, number_value,
    numbers_are_equal,
    promises::{
        new_promise_capability, perform_promise_then_with_capability, promise_capability_promise,
        promise_capability_reject, promise_capability_resolve, promise_default_constructor,
        promise_resolve_method,
    },
    property_key_from_text, proxy_get_own_property, proxy_get_prototype_of,
    proxy_own_property_keys, range_error, set_property_on_object, string_from_code_units,
    string_ref_code_units, string_ref_text, string_this_ref, string_value, to_number_for_builtin,
    type_error, BuiltinIteratorBridge, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_common::{AtomId, WellKnownAtom};
use lyng_js_env::{Agent, PromiseCapabilityId, PromiseReactionHandler};
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::{ObjectAllocation, ObjectColdData, OrdinaryObjectData};
use lyng_js_ops::{errors, iterator, read};
use lyng_js_types::{
    AbruptCompletion, BuiltinFunctionId, ObjectRef, PropertyKey, Value, WellKnownSymbolId,
};

pub(super) fn dispatch_iterator_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::iterator_prototype_iterator_builtin() {
        return Ok(Some(iterator_prototype_iterator_value(invocation)));
    }
    if entry == super::iterator_builtin() {
        return iterator_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_from_builtin() {
        return iterator_from_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_concat_builtin() {
        return iterator_concat_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_zip_builtin() {
        return iterator_zip_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_zip_keyed_builtin() {
        return iterator_zip_keyed_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_reduce_builtin() {
        return iterator_reduce_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_for_each_builtin() {
        return iterator_for_each_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_some_builtin() {
        return iterator_some_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_every_builtin() {
        return iterator_every_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_find_builtin() {
        return iterator_find_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_to_array_builtin() {
        return iterator_to_array_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_map_builtin() {
        return iterator_map_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_filter_builtin() {
        return iterator_filter_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_flat_map_builtin() {
        return iterator_flat_map_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_take_builtin() {
        return iterator_take_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_drop_builtin() {
        return iterator_drop_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_dispose_builtin() {
        return iterator_dispose_builtin(context, invocation).map(Some);
    }
    if entry == super::async_iterator_dispose_builtin() {
        return async_iterator_dispose_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_helper_next_builtin() {
        return iterator_helper_next_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_helper_return_builtin() {
        return iterator_helper_return_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_to_string_tag_getter_builtin() {
        return Ok(Some(iterator_to_string_tag_value(context)));
    }
    if entry == super::iterator_to_string_tag_setter_builtin() {
        return iterator_to_string_tag_setter_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_constructor_getter_builtin() {
        return iterator_constructor_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_constructor_setter_builtin() {
        return iterator_constructor_setter_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

mod dispose;
mod eager;
mod helper_state;
mod indexed;
mod lazy;
mod protocol;
mod sequence;

use dispose::{async_iterator_dispose_builtin, iterator_dispose_builtin};
use eager::{
    iterator_every_builtin, iterator_find_builtin, iterator_for_each_builtin,
    iterator_reduce_builtin, iterator_some_builtin, iterator_to_array_builtin,
};
use helper_state::{
    clear_iterator_helper_inner, get_iterator_flattenable, iterator_helper_active_record,
    iterator_helper_counter, iterator_helper_done, iterator_helper_inner_record,
    iterator_helper_iterated_object, iterator_helper_limit, iterator_helper_record,
    iterator_helper_running, iterator_helper_sequence_count, iterator_helper_this_object,
    set_iterator_helper_done, set_iterator_helper_limit, set_iterator_helper_running, u64_to_value,
    IteratorHelperKind, IteratorZipCollectedRecord, IteratorZipKey, IteratorZipMode,
    ITERATOR_HELPER_COUNTER_SLOT, ITERATOR_HELPER_INNER_ITERATED_SLOT,
    ITERATOR_HELPER_INNER_NEXT_METHOD_SLOT, ITERATOR_HELPER_ITERATED_SLOT,
    ITERATOR_HELPER_KIND_SLOT, ITERATOR_HELPER_NEXT_METHOD_SLOT, ITERATOR_HELPER_PARAM_SLOT,
    ITERATOR_HELPER_SEQUENCE_BASE_SLOT, ITERATOR_ZIP_ALIVE_OFFSET, ITERATOR_ZIP_ITERATED_OFFSET,
    ITERATOR_ZIP_KEY_KIND_OFFSET, ITERATOR_ZIP_KEY_PAYLOAD_OFFSET, ITERATOR_ZIP_NEXT_METHOD_OFFSET,
    ITERATOR_ZIP_PADDING_OFFSET, ITERATOR_ZIP_RECORD_WIDTH,
};
use indexed::iterator_prototype_iterator_value;
pub(super) use indexed::{
    allocate_iterator_object, array_iterator_factory_builtin, array_iterator_next_builtin,
    create_iterator_result_value, iterator_slot_value_for_builtin,
    set_iterator_slot_value_for_builtin, string_iterator_builtin, string_iterator_next_builtin,
    typed_array_iterator_factory_builtin, ArrayIterationKind, MAP_ITERATOR_INDEX_SLOT,
    MAP_ITERATOR_KIND_SLOT, MAP_ITERATOR_TARGET_SLOT, SET_ITERATOR_INDEX_SLOT,
    SET_ITERATOR_KIND_SLOT, SET_ITERATOR_TARGET_SLOT,
};
use lazy::{
    iterator_drop_builtin, iterator_filter_builtin, iterator_flat_map_builtin,
    iterator_helper_next_builtin, iterator_helper_return_builtin, iterator_map_builtin,
    iterator_take_builtin,
};
use protocol::{
    iterator_builtin, iterator_close_for_validation_failure, iterator_constructor_getter_builtin,
    iterator_constructor_setter_builtin, iterator_direct_record, iterator_this_object,
    iterator_to_string_tag_setter_builtin, iterator_to_string_tag_value,
};
use sequence::{
    iterator_concat_builtin, iterator_from_builtin, iterator_helper_concat_next,
    iterator_helper_concat_return, iterator_helper_zip_next, iterator_helper_zip_return,
    iterator_helper_zip_started, iterator_zip_builtin, iterator_zip_keyed_builtin,
};

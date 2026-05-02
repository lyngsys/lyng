mod atomics;
mod buffers;
mod data_view;
mod typed_arrays;

use atomics::dispatch_atomics_builtin;
use buffers::dispatch_buffer_builtin;
use data_view::dispatch_data_view_builtin;
use typed_arrays::{
    dispatch_typed_array_access_builtin, dispatch_typed_array_constructor_builtin,
    dispatch_typed_array_iteration_builtin, dispatch_typed_array_mutation_builtin,
    dispatch_typed_array_search_builtin, dispatch_uint8_array_base64_hex_builtin,
};
pub(super) use typed_arrays::{
    typed_array_is_out_of_bounds, typed_array_storage_bits_from_builtin_value,
    typed_array_validated_object_and_record, typed_array_write_storage_bits,
};

use super::{
    array_like_index_property_key, array_like_length_u64, arrays,
    collect_array_like_values_for_from_builtin, create_data_property_or_throw,
    get_property_from_object, iterable_to_values_list, iterators, length_value_u64, map_completion,
    normalize_relative_index_u64, promises, property_key_from_text, range_error,
    string_from_code_units, string_ref_code_units, string_value, syntax_error,
    to_bigint_for_builtin, to_boolean_for_builtin, to_index_for_builtin,
    to_integer_or_infinity_for_builtin, to_number_for_builtin, to_string_string_ref,
    to_uint32_for_builtin, to_uint8_clamp_for_builtin, to_uint8_for_builtin, type_error,
    PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_types::{BuiltinFunctionId, Value};

pub(super) fn dispatch_binary_data_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_buffer_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_data_view_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_atomics_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_uint8_array_base64_hex_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_typed_array_constructor_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_typed_array_prototype_builtin(context, entry, invocation)
}

fn dispatch_typed_array_prototype_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_typed_array_access_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_typed_array_iteration_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_typed_array_mutation_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_typed_array_search_builtin(context, entry, invocation)
}

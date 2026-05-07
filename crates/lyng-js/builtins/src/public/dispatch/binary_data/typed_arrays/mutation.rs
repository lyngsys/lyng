use super::super::super::{
    set_property_on_object, typed_array_copy_within_builtin, typed_array_fill_builtin,
    typed_array_reverse_builtin, typed_array_sort_builtin, typed_array_to_reversed_builtin,
    typed_array_to_sorted_builtin, typed_array_with_builtin, uint8_array_set_builtin,
    uint8_array_slice_builtin, uint8_array_subarray_builtin,
};
use super::super::{
    array_like_index_property_key, array_like_length_u64, arrays, get_property_from_object,
    length_value_u64, normalize_relative_index_u64, range_error, to_index_for_builtin,
    to_integer_or_infinity_for_builtin, type_error, PublicBuiltinDispatchContext,
};
use super::{
    typed_array_current_length, typed_array_read_element_value, typed_array_read_storage_bits,
    typed_array_same_kind_create, typed_array_snapshot_storage_bits, typed_array_species_create,
    typed_array_species_create_with_arguments, typed_array_storage_bits_from_builtin_value,
    typed_array_storage_bits_to_value, typed_array_storage_u16_bits, typed_array_storage_u32_bits,
    typed_array_storage_u8_bits, typed_array_this_object, typed_array_this_record,
    typed_array_validated_object_record_and_length, typed_array_validated_record,
    typed_array_validated_record_and_length, typed_array_write_storage_bits,
};
use crate::BuiltinInvocation;
use lyng_js_objects::{float16_bits_to_f64, TypedArrayElementKind};
use lyng_js_types::{BuiltinFunctionId, ObjectRef, Value};

pub(in crate::public::dispatch::binary_data) fn dispatch_typed_array_mutation_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == uint8_array_set_builtin() {
        return uint8_array_set_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == uint8_array_slice_builtin() {
        return uint8_array_slice_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == uint8_array_subarray_builtin() {
        return uint8_array_subarray_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_fill_builtin() {
        return typed_array_fill_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_copy_within_builtin() {
        return typed_array_copy_within_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_reverse_builtin() {
        return typed_array_reverse_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_sort_builtin() {
        return typed_array_sort_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_to_reversed_builtin() {
        return typed_array_to_reversed_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_to_sorted_builtin() {
        return typed_array_to_sorted_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_with_builtin() {
        return typed_array_with_builtin_dispatch(context, invocation).map(Some);
    }
    Ok(None)
}

fn compare_typed_array_float_values(left: f64, right: f64) -> std::cmp::Ordering {
    if left.is_nan() {
        return if right.is_nan() {
            std::cmp::Ordering::Equal
        } else {
            std::cmp::Ordering::Greater
        };
    }
    if right.is_nan() {
        return std::cmp::Ordering::Less;
    }
    if left < right {
        return std::cmp::Ordering::Less;
    }
    if left > right {
        return std::cmp::Ordering::Greater;
    }
    if left == 0.0 && right == 0.0 {
        return match (left.is_sign_negative(), right.is_sign_negative()) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        };
    }
    std::cmp::Ordering::Equal
}

fn compare_typed_array_default_elements(
    kind: TypedArrayElementKind,
    left_bits: u64,
    right_bits: u64,
) -> std::cmp::Ordering {
    match kind {
        TypedArrayElementKind::BigInt64 => left_bits.cast_signed().cmp(&right_bits.cast_signed()),
        TypedArrayElementKind::BigUint64 => left_bits.cmp(&right_bits),
        TypedArrayElementKind::Int8 => typed_array_storage_u8_bits(left_bits)
            .cast_signed()
            .cmp(&typed_array_storage_u8_bits(right_bits).cast_signed()),
        TypedArrayElementKind::Int16 => typed_array_storage_u16_bits(left_bits)
            .cast_signed()
            .cmp(&typed_array_storage_u16_bits(right_bits).cast_signed()),
        TypedArrayElementKind::Int32 => typed_array_storage_u32_bits(left_bits)
            .cast_signed()
            .cmp(&typed_array_storage_u32_bits(right_bits).cast_signed()),
        TypedArrayElementKind::Uint8 | TypedArrayElementKind::Uint8Clamped => {
            typed_array_storage_u8_bits(left_bits).cmp(&typed_array_storage_u8_bits(right_bits))
        }
        TypedArrayElementKind::Uint16 => {
            typed_array_storage_u16_bits(left_bits).cmp(&typed_array_storage_u16_bits(right_bits))
        }
        TypedArrayElementKind::Uint32 => {
            typed_array_storage_u32_bits(left_bits).cmp(&typed_array_storage_u32_bits(right_bits))
        }
        TypedArrayElementKind::Float16 => compare_typed_array_float_values(
            float16_bits_to_f64(typed_array_storage_u16_bits(left_bits)),
            float16_bits_to_f64(typed_array_storage_u16_bits(right_bits)),
        ),
        TypedArrayElementKind::Float32 => compare_typed_array_float_values(
            f64::from(f32::from_bits(typed_array_storage_u32_bits(left_bits))),
            f64::from(f32::from_bits(typed_array_storage_u32_bits(right_bits))),
        ),
        TypedArrayElementKind::Float64 => {
            compare_typed_array_float_values(f64::from_bits(left_bits), f64::from_bits(right_bits))
        }
    }
}

fn compare_typed_array_sort_elements<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: TypedArrayElementKind,
    compare_fn: Option<ObjectRef>,
    left_bits: u64,
    right_bits: u64,
) -> Result<std::cmp::Ordering, Cx::Error> {
    if let Some(compare_fn) = compare_fn {
        let left = typed_array_storage_bits_to_value(cx.agent(), kind, left_bits);
        let right = typed_array_storage_bits_to_value(cx.agent(), kind, right_bits);
        return arrays::compare_array_sort_values(cx, Some(compare_fn), left, right);
    }
    Ok(compare_typed_array_default_elements(
        kind, left_bits, right_bits,
    ))
}

fn sort_typed_array_default_elements(kind: TypedArrayElementKind, elements: &mut [u64]) {
    if counting_sort_typed_array_default_elements(kind, elements) {
        return;
    }
    elements.sort_by(|left, right| compare_typed_array_default_elements(kind, *left, *right));
}

fn counting_sort_typed_array_default_elements(
    kind: TypedArrayElementKind,
    elements: &mut [u64],
) -> bool {
    let range = match kind {
        TypedArrayElementKind::Int16 | TypedArrayElementKind::Uint16 => 1_usize << 16,
        _ => return false,
    };
    let mut counts = vec![0_usize; range];
    for bits in elements.iter().copied() {
        counts[usize::from(typed_array_storage_u16_bits(bits))] += 1;
    }
    let mut index = 0;
    match kind {
        TypedArrayElementKind::Int16 => {
            for (key, count) in counts.iter().copied().enumerate().skip(1_usize << 15) {
                for _ in 0..count {
                    elements[index] =
                        u64::try_from(key).expect("16-bit counting-sort key should fit u64");
                    index += 1;
                }
            }
            for (key, count) in counts.iter().copied().enumerate().take(1_usize << 15) {
                for _ in 0..count {
                    elements[index] =
                        u64::try_from(key).expect("16-bit counting-sort key should fit u64");
                    index += 1;
                }
            }
        }
        TypedArrayElementKind::Uint16 => {
            for (key, count) in counts.iter().copied().enumerate() {
                for _ in 0..count {
                    elements[index] =
                        u64::try_from(key).expect("16-bit counting-sort key should fit u64");
                    index += 1;
                }
            }
        }
        _ => unreachable!("counting sort range should only be selected for 16-bit integer arrays"),
    }
    true
}

fn sort_typed_array_compare_elements<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: TypedArrayElementKind,
    compare_fn: ObjectRef,
    elements: &mut [u64],
) -> Result<(), Cx::Error> {
    if elements.len() <= 1 {
        return Ok(());
    }
    let mut scratch = elements.to_vec();
    merge_sort_typed_array_compare_elements(
        cx,
        kind,
        compare_fn,
        elements,
        &mut scratch,
        0,
        elements.len(),
    )
}

fn merge_sort_typed_array_compare_elements<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: TypedArrayElementKind,
    compare_fn: ObjectRef,
    elements: &mut [u64],
    scratch: &mut [u64],
    start: usize,
    end: usize,
) -> Result<(), Cx::Error> {
    let len = end - start;
    if len <= 1 {
        return Ok(());
    }
    let mid = start + len / 2;
    merge_sort_typed_array_compare_elements(cx, kind, compare_fn, elements, scratch, start, mid)?;
    merge_sort_typed_array_compare_elements(cx, kind, compare_fn, elements, scratch, mid, end)?;

    let mut left = start;
    let mut right = mid;
    for slot in scratch.iter_mut().take(end).skip(start) {
        if left == mid {
            *slot = elements[right];
            right += 1;
        } else if right == end {
            *slot = elements[left];
            left += 1;
        } else if compare_typed_array_sort_elements(
            cx,
            kind,
            Some(compare_fn),
            elements[left],
            elements[right],
        )? == std::cmp::Ordering::Greater
        {
            *slot = elements[right];
            right += 1;
        } else {
            *slot = elements[left];
            left += 1;
        }
    }
    elements[start..end].copy_from_slice(&scratch[start..end]);
    Ok(())
}

fn typed_array_reverse_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (record, length) = typed_array_validated_record_and_length(cx, invocation.this_value())?;
    let half_len = length / 2;
    let last_index = length.saturating_sub(1);
    for lower in 0..half_len {
        let upper = last_index - lower;
        let lower_bits = typed_array_read_storage_bits(cx.agent(), record, lower)
            .ok_or_else(|| type_error(cx))?;
        let upper_bits = typed_array_read_storage_bits(cx.agent(), record, upper)
            .ok_or_else(|| type_error(cx))?;
        typed_array_write_storage_bits(cx, record, lower, upper_bits)?;
        typed_array_write_storage_bits(cx, record, upper, lower_bits)?;
    }
    Ok(invocation.this_value())
}

fn typed_array_sort_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_validated_record(cx, invocation.this_value())?;
    let compare_fn = match invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined())
    {
        value if value.is_undefined() => None,
        value => Some(cx.require_callable_object(value)?),
    };
    let mut elements = typed_array_snapshot_storage_bits(cx.agent(), record);
    if let Some(compare_fn) = compare_fn {
        sort_typed_array_compare_elements(cx, record.kind(), compare_fn, &mut elements)?;
    } else {
        sort_typed_array_default_elements(record.kind(), &mut elements);
    }
    let Some(current_length) = typed_array_current_length(cx.agent(), record) else {
        return Ok(invocation.this_value());
    };
    for (index, bits) in elements.into_iter().take(current_length).enumerate() {
        typed_array_write_storage_bits(cx, record, index, bits)?;
    }
    Ok(invocation.this_value())
}

fn typed_array_to_reversed_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_validated_record(cx, invocation.this_value())?;
    let length = record.length();
    let (result_object, result_record) = typed_array_same_kind_create(cx, record.kind(), length)?;
    let source = typed_array_snapshot_storage_bits(cx.agent(), record);
    for (index, bits) in source.into_iter().rev().enumerate() {
        typed_array_write_storage_bits(cx, result_record, index, bits)?;
    }
    Ok(Value::from_object_ref(result_object))
}

fn typed_array_to_sorted_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_validated_record(cx, invocation.this_value())?;
    let compare_fn = match invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined())
    {
        value if value.is_undefined() => None,
        value => Some(cx.require_callable_object(value)?),
    };
    let length = record.length();
    let (result_object, result_record) = typed_array_same_kind_create(cx, record.kind(), length)?;
    let mut elements = typed_array_snapshot_storage_bits(cx.agent(), record);
    if let Some(compare_fn) = compare_fn {
        sort_typed_array_compare_elements(cx, record.kind(), compare_fn, &mut elements)?;
    } else {
        sort_typed_array_default_elements(record.kind(), &mut elements);
    }
    for (index, bits) in elements.into_iter().enumerate() {
        typed_array_write_storage_bits(cx, result_record, index, bits)?;
    }
    Ok(Value::from_object_ref(result_object))
}

fn typed_array_with_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (record, length) = typed_array_validated_record_and_length(cx, invocation.this_value())?;
    let relative_index = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let replacement_bits = typed_array_storage_bits_from_builtin_value(
        cx,
        record.kind(),
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let actual_index = if relative_index < 0.0 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "ECMAScript relative typed-array indices are represented as Number values"
        )]
        let length_number = length as f64;
        length_number + relative_index
    } else {
        relative_index
    };
    if !actual_index.is_finite() || actual_index < 0.0 {
        return Err(range_error(cx));
    }
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "TypedArray.prototype.with converts a checked finite non-negative integer index"
    )]
    let actual_index = actual_index as usize;
    if typed_array_current_length(cx.agent(), record).is_none_or(|length| actual_index >= length) {
        return Err(range_error(cx));
    }
    let (result_object, result_record) = typed_array_same_kind_create(cx, record.kind(), length)?;
    for index in 0..length {
        let bits = if index == actual_index {
            replacement_bits
        } else {
            let value = typed_array_read_element_value(cx.agent(), record, index);
            typed_array_storage_bits_from_builtin_value(cx, result_record.kind(), value)?
        };
        typed_array_write_storage_bits(cx, result_record, index, bits)?;
    }
    Ok(Value::from_object_ref(result_object))
}

fn typed_array_fill_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (record, length) = typed_array_validated_record_and_length(cx, invocation.this_value())?;
    let length = u64::try_from(length).unwrap_or(u64::MAX);
    let relative_start = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let start = normalize_relative_index_u64(length, relative_start);
    let end = match invocation.arguments().get(2).copied() {
        Some(value) if !value.is_undefined() => {
            let relative_end = to_integer_or_infinity_for_builtin(cx, value)?;
            normalize_relative_index_u64(length, relative_end)
        }
        _ => length,
    };
    let fill_bits = typed_array_storage_bits_from_builtin_value(
        cx,
        record.kind(),
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let current_length =
        typed_array_current_length(cx.agent(), record).ok_or_else(|| type_error(cx))?;
    let end = end.min(u64::try_from(current_length).unwrap_or(u64::MAX));
    for index in start..end {
        let index = usize::try_from(index).map_err(|_| range_error(cx))?;
        typed_array_write_storage_bits(cx, record, index, fill_bits)?;
    }
    Ok(invocation.this_value())
}

fn typed_array_copy_within_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (record, length) = typed_array_validated_record_and_length(cx, invocation.this_value())?;
    let length = u64::try_from(length).unwrap_or(u64::MAX);
    let relative_target = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let to = normalize_relative_index_u64(length, relative_target);
    let relative_start = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let from = normalize_relative_index_u64(length, relative_start);
    let final_index = match invocation.arguments().get(2).copied() {
        Some(value) if !value.is_undefined() => {
            let relative_end = to_integer_or_infinity_for_builtin(cx, value)?;
            normalize_relative_index_u64(length, relative_end)
        }
        _ => length,
    };
    let count = final_index
        .saturating_sub(from)
        .min(length.saturating_sub(to));
    let current_length =
        typed_array_current_length(cx.agent(), record).ok_or_else(|| type_error(cx))?;
    if count == 0 {
        return Ok(invocation.this_value());
    }
    let from_usize = usize::try_from(from).map_err(|_| range_error(cx))?;
    let to_usize = usize::try_from(to).map_err(|_| range_error(cx))?;
    let count_usize = usize::try_from(count).map_err(|_| range_error(cx))?;
    let mut copied_bits = Vec::with_capacity(count_usize);
    for offset in 0..count_usize {
        let index = from_usize
            .checked_add(offset)
            .ok_or_else(|| range_error(cx))?;
        let bits = if index < current_length {
            typed_array_read_storage_bits(cx.agent(), record, index)
        } else {
            None
        };
        copied_bits.push(bits);
    }
    for (offset, bits) in copied_bits.into_iter().enumerate() {
        let index = to_usize
            .checked_add(offset)
            .ok_or_else(|| range_error(cx))?;
        if index >= current_length {
            continue;
        }
        if let Some(bits) = bits {
            typed_array_write_storage_bits(cx, record, index, bits)?;
        }
    }
    Ok(invocation.this_value())
}

fn uint8_array_set_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = typed_array_this_object(cx, invocation.this_value())?;
    let source = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let offset = to_index_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let offset = usize::try_from(offset).map_err(|_| range_error(cx))?;
    let (_record, target_length) =
        typed_array_validated_record_and_length(cx, Value::from_object_ref(object))?;

    if let Some(source_object) = source
        .as_object_ref()
        .filter(|object| cx.agent().objects().typed_array(*object).is_some())
    {
        let (source_record, source_length) =
            typed_array_validated_record_and_length(cx, Value::from_object_ref(source_object))?;
        if offset > target_length || source_length > target_length.saturating_sub(offset) {
            return Err(range_error(cx));
        }
        let mut values = Vec::with_capacity(source_length);
        for index in 0..source_length {
            values.push(typed_array_read_element_value(
                cx.agent(),
                source_record,
                index,
            ));
        }
        for (index, value) in values.into_iter().enumerate() {
            let target_index = offset.checked_add(index).ok_or_else(|| range_error(cx))?;
            let key =
                array_like_index_property_key(cx, u64::try_from(target_index).unwrap_or(u64::MAX));
            set_property_on_object(cx, object, key, value)?;
        }
        return Ok(Value::undefined());
    }

    let source_object = cx.to_object_for_builtin_value(cx.builtin_realm(), source)?;
    let source_length = array_like_length_u64(cx, source_object)?;
    let source_length = usize::try_from(source_length).map_err(|_| range_error(cx))?;
    if offset > target_length || source_length > target_length.saturating_sub(offset) {
        return Err(range_error(cx));
    }
    for index in 0..source_length {
        let key = array_like_index_property_key(cx, u64::try_from(index).unwrap_or(u64::MAX));
        let value = get_property_from_object(cx, source_object, key)?;
        let target_index = offset.checked_add(index).ok_or_else(|| range_error(cx))?;
        let key =
            array_like_index_property_key(cx, u64::try_from(target_index).unwrap_or(u64::MAX));
        set_property_on_object(cx, object, key, value)?;
    }
    Ok(Value::undefined())
}

fn uint8_array_slice_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (object, record, source_length) =
        typed_array_validated_object_record_and_length(cx, invocation.this_value())?;
    let source_length = u64::try_from(source_length).unwrap_or(u64::MAX);
    let start = normalize_relative_index_u64(
        source_length,
        to_integer_or_infinity_for_builtin(
            cx,
            invocation
                .arguments()
                .first()
                .copied()
                .unwrap_or(Value::undefined()),
        )?,
    );
    let end = match invocation.arguments().get(1).copied() {
        Some(value) if value.is_undefined() => source_length,
        Some(value) => normalize_relative_index_u64(
            source_length,
            to_integer_or_infinity_for_builtin(cx, value)?,
        ),
        None => source_length,
    };
    let new_end = end.max(start);
    let length = usize::try_from(new_end.saturating_sub(start)).map_err(|_| range_error(cx))?;
    let start_index = usize::try_from(start).map_err(|_| range_error(cx))?;
    let (result_object, result_record) =
        typed_array_species_create(cx, object, record.kind(), length)?;
    let copy_length = if length == 0 {
        0
    } else {
        let current_length =
            typed_array_current_length(cx.agent(), record).ok_or_else(|| type_error(cx))?;
        let end_index = start_index
            .checked_add(length)
            .ok_or_else(|| range_error(cx))?
            .min(current_length);
        end_index.saturating_sub(start_index)
    };
    for offset in 0..copy_length {
        let source_index = start_index
            .checked_add(offset)
            .ok_or_else(|| range_error(cx))?;
        let bits = if result_record.kind() == record.kind() {
            typed_array_read_storage_bits(cx.agent(), record, source_index)
                .ok_or_else(|| type_error(cx))?
        } else {
            let value = typed_array_read_element_value(cx.agent(), record, source_index);
            typed_array_storage_bits_from_builtin_value(cx, result_record.kind(), value)?
        };
        typed_array_write_storage_bits(cx, result_record, offset, bits)?;
    }
    Ok(Value::from_object_ref(result_object))
}

fn uint8_array_subarray_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = typed_array_this_object(cx, invocation.this_value())?;
    let record = typed_array_this_record(cx, invocation.this_value())?;
    let source_length = u64::try_from(typed_array_current_length(cx.agent(), record).unwrap_or(0))
        .unwrap_or(u64::MAX);
    let start = normalize_relative_index_u64(
        source_length,
        to_integer_or_infinity_for_builtin(
            cx,
            invocation
                .arguments()
                .first()
                .copied()
                .unwrap_or(Value::undefined()),
        )?,
    );
    let end_argument = invocation.arguments().get(1).copied();
    let end_is_undefined = end_argument.is_none_or(lyng_js_types::Value::is_undefined);
    let end = match end_argument {
        Some(value) if value.is_undefined() => source_length,
        Some(value) => normalize_relative_index_u64(
            source_length,
            to_integer_or_infinity_for_builtin(cx, value)?,
        ),
        None => source_length,
    };
    let new_end = end.max(start);
    let byte_offset = record
        .byte_offset()
        .checked_add(
            usize::try_from(start)
                .map_err(|_| range_error(cx))?
                .checked_mul(record.kind().bytes_per_element())
                .ok_or_else(|| range_error(cx))?,
        )
        .ok_or_else(|| range_error(cx))?;
    let length = usize::try_from(new_end.saturating_sub(start)).map_err(|_| range_error(cx))?;
    let buffer = Value::from_object_ref(record.viewed_array_buffer());
    let byte_offset = length_value_u64(u64::try_from(byte_offset).unwrap_or(u64::MAX));
    let (result_object, _) = if record.is_length_tracking() && end_is_undefined {
        let arguments = [buffer, byte_offset];
        typed_array_species_create_with_arguments(cx, object, record.kind(), &arguments, None)?
    } else {
        let arguments = [
            buffer,
            byte_offset,
            length_value_u64(u64::try_from(length).unwrap_or(u64::MAX)),
        ];
        typed_array_species_create_with_arguments(cx, object, record.kind(), &arguments, None)?
    };
    Ok(Value::from_object_ref(result_object))
}

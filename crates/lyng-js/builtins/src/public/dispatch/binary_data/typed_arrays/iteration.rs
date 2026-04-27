use super::super::super::{
    typed_array_every_builtin, typed_array_filter_builtin, typed_array_find_builtin,
    typed_array_find_index_builtin, typed_array_find_last_builtin,
    typed_array_find_last_index_builtin, typed_array_for_each_builtin, typed_array_map_builtin,
    typed_array_reduce_builtin, typed_array_reduce_right_builtin, typed_array_some_builtin,
};
use super::super::{
    length_value_u64, to_boolean_for_builtin, type_error, PublicBuiltinDispatchContext,
};
use super::{
    typed_array_read_element_value, typed_array_species_create,
    typed_array_storage_bits_from_builtin_value, typed_array_validated_object_and_record,
    typed_array_validated_record, typed_array_write_storage_bits,
};
use crate::BuiltinInvocation;
use lyng_js_types::{BuiltinFunctionId, Value};

pub(in crate::public::dispatch::binary_data) fn dispatch_typed_array_iteration_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == typed_array_every_builtin() {
        return typed_array_every_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_some_builtin() {
        return typed_array_some_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_find_builtin() {
        return typed_array_find_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_find_index_builtin() {
        return typed_array_find_index_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_find_last_builtin() {
        return typed_array_find_last_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_find_last_index_builtin() {
        return typed_array_find_last_index_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_filter_builtin() {
        return typed_array_filter_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_for_each_builtin() {
        return typed_array_for_each_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_map_builtin() {
        return typed_array_map_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_reduce_builtin() {
        return typed_array_reduce_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_reduce_right_builtin() {
        return typed_array_reduce_right_builtin_dispatch(context, invocation).map(Some);
    }
    Ok(None)
}

#[derive(Clone, Copy)]
enum TypedArrayPredicateKind {
    Every,
    Some,
    Find,
    FindIndex,
    FindLast,
    FindLastIndex,
}

fn typed_array_predicate_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: TypedArrayPredicateKind,
) -> Result<Value, Cx::Error> {
    let this_value = invocation.this_value();
    let record = typed_array_validated_record(cx, this_value)?;
    let callback = cx.require_callable_object(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let this_arg = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let mut indices: Box<dyn Iterator<Item = usize>> = match kind {
        TypedArrayPredicateKind::FindLast | TypedArrayPredicateKind::FindLastIndex => {
            Box::new((0..record.length()).rev())
        }
        _ => Box::new(0..record.length()),
    };
    for index in indices.by_ref() {
        let element = typed_array_read_element_value(cx.agent(), record, index);
        let selected = cx.call_to_completion(
            callback,
            this_arg,
            &[
                element,
                length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)),
                this_value,
            ],
        )?;
        let selected = to_boolean_for_builtin(cx, selected)?;
        match kind {
            TypedArrayPredicateKind::Every => {
                if !selected {
                    return Ok(Value::from_bool(false));
                }
            }
            TypedArrayPredicateKind::Some => {
                if selected {
                    return Ok(Value::from_bool(true));
                }
            }
            TypedArrayPredicateKind::Find | TypedArrayPredicateKind::FindLast => {
                if selected {
                    return Ok(element);
                }
            }
            TypedArrayPredicateKind::FindIndex | TypedArrayPredicateKind::FindLastIndex => {
                if selected {
                    return Ok(length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)));
                }
            }
        }
    }
    Ok(match kind {
        TypedArrayPredicateKind::Every => Value::from_bool(true),
        TypedArrayPredicateKind::Some => Value::from_bool(false),
        TypedArrayPredicateKind::Find | TypedArrayPredicateKind::FindLast => Value::undefined(),
        TypedArrayPredicateKind::FindIndex | TypedArrayPredicateKind::FindLastIndex => {
            Value::from_smi(-1)
        }
    })
}

fn typed_array_every_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_predicate_builtin(cx, invocation, TypedArrayPredicateKind::Every)
}

fn typed_array_some_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_predicate_builtin(cx, invocation, TypedArrayPredicateKind::Some)
}

fn typed_array_find_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_predicate_builtin(cx, invocation, TypedArrayPredicateKind::Find)
}

fn typed_array_find_index_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_predicate_builtin(cx, invocation, TypedArrayPredicateKind::FindIndex)
}

fn typed_array_find_last_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_predicate_builtin(cx, invocation, TypedArrayPredicateKind::FindLast)
}

fn typed_array_find_last_index_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_predicate_builtin(cx, invocation, TypedArrayPredicateKind::FindLastIndex)
}

fn typed_array_filter_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (object, record) = typed_array_validated_object_and_record(cx, invocation.this_value())?;
    let callback = cx.require_callable_object(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let this_arg = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let this_value = invocation.this_value();
    let mut kept = Vec::with_capacity(record.length());
    for index in 0..record.length() {
        let value = typed_array_read_element_value(cx.agent(), record, index);
        let selected = cx.call_to_completion(
            callback,
            this_arg,
            &[
                value,
                length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)),
                this_value,
            ],
        )?;
        if to_boolean_for_builtin(cx, selected)? {
            kept.push(value);
        }
    }
    let (result_object, result_record) =
        typed_array_species_create(cx, object, record.kind(), kept.len())?;
    for (index, value) in kept.into_iter().enumerate() {
        let bits = typed_array_storage_bits_from_builtin_value(cx, result_record.kind(), value)?;
        typed_array_write_storage_bits(cx, result_record, index, bits)?;
    }
    Ok(Value::from_object_ref(result_object))
}

fn typed_array_for_each_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_validated_record(cx, invocation.this_value())?;
    let callback = cx.require_callable_object(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let this_arg = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let this_value = invocation.this_value();
    for index in 0..record.length() {
        let value = typed_array_read_element_value(cx.agent(), record, index);
        let _ = cx.call_to_completion(
            callback,
            this_arg,
            &[
                value,
                length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)),
                this_value,
            ],
        )?;
    }
    Ok(Value::undefined())
}

fn typed_array_map_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (object, record) = typed_array_validated_object_and_record(cx, invocation.this_value())?;
    let callback = cx.require_callable_object(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let this_arg = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let (result_object, result_record) =
        typed_array_species_create(cx, object, record.kind(), record.length())?;
    let this_value = invocation.this_value();
    for index in 0..record.length() {
        let value = typed_array_read_element_value(cx.agent(), record, index);
        let mapped = cx.call_to_completion(
            callback,
            this_arg,
            &[
                value,
                length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)),
                this_value,
            ],
        )?;
        let bits = typed_array_storage_bits_from_builtin_value(cx, result_record.kind(), mapped)?;
        typed_array_write_storage_bits(cx, result_record, index, bits)?;
    }
    Ok(Value::from_object_ref(result_object))
}

#[derive(Clone, Copy)]
enum TypedArrayReduceDirection {
    Forward,
    Reverse,
}

fn typed_array_reduce_common<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    direction: TypedArrayReduceDirection,
) -> Result<Value, Cx::Error> {
    let record = typed_array_validated_record(cx, invocation.this_value())?;
    let callback = cx.require_callable_object(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let this_value = invocation.this_value();
    let len = record.length();
    let mut accumulator;
    let mut next_index;
    match invocation.arguments().get(1).copied() {
        Some(initial_value) => {
            accumulator = initial_value;
            next_index = match direction {
                TypedArrayReduceDirection::Forward => Some(0),
                TypedArrayReduceDirection::Reverse => len.checked_sub(1),
            };
        }
        None => {
            if len == 0 {
                return Err(type_error(cx));
            }
            let initial_index = match direction {
                TypedArrayReduceDirection::Forward => 0,
                TypedArrayReduceDirection::Reverse => len - 1,
            };
            accumulator = typed_array_read_element_value(cx.agent(), record, initial_index);
            next_index = match direction {
                TypedArrayReduceDirection::Forward => initial_index.checked_add(1),
                TypedArrayReduceDirection::Reverse => initial_index.checked_sub(1),
            };
        }
    }

    match direction {
        TypedArrayReduceDirection::Forward => {
            while let Some(index) = next_index {
                if index >= len {
                    break;
                }
                let value = typed_array_read_element_value(cx.agent(), record, index);
                accumulator = cx.call_to_completion(
                    callback,
                    Value::undefined(),
                    &[
                        accumulator,
                        value,
                        length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)),
                        this_value,
                    ],
                )?;
                next_index = index.checked_add(1);
            }
        }
        TypedArrayReduceDirection::Reverse => {
            while let Some(index) = next_index {
                let value = typed_array_read_element_value(cx.agent(), record, index);
                accumulator = cx.call_to_completion(
                    callback,
                    Value::undefined(),
                    &[
                        accumulator,
                        value,
                        length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)),
                        this_value,
                    ],
                )?;
                next_index = index.checked_sub(1);
            }
        }
    }

    Ok(accumulator)
}

fn typed_array_reduce_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_reduce_common(cx, invocation, TypedArrayReduceDirection::Forward)
}

fn typed_array_reduce_right_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_reduce_common(cx, invocation, TypedArrayReduceDirection::Reverse)
}

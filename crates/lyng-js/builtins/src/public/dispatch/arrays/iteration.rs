use super::super::{
    array_like_index_property_key, array_like_length, array_like_length_u64,
    array_species_create_for_length, create_data_property_or_throw, get_property_from_object,
    has_property_on_object, is_array_for_species, length_value, length_value_u64, map_completion,
    normalize_relative_index_u64, to_boolean_for_builtin, to_integer_or_infinity_for_builtin,
    type_error, PublicBuiltinDispatchContext, MAX_SAFE_INTEGER_U64,
};
use super::{array_index_from_number, array_length_as_number};
use crate::BuiltinInvocation;
use lyng_js_ops::read;
use lyng_js_types::{BuiltinFunctionId, ObjectRef, PropertyKey, Value};

pub(super) fn dispatch_array_iteration_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::super::array_every_builtin() {
        return array_every_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_filter_builtin() {
        return array_filter_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_flat_builtin() {
        return array_flat_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_flat_map_builtin() {
        return array_flat_map_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_find_builtin() {
        return array_find_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_find_index_builtin() {
        return array_find_index_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_find_last_builtin() {
        return array_find_last_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_find_last_index_builtin() {
        return array_find_last_index_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_for_each_builtin() {
        return array_for_each_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_includes_builtin() {
        return array_includes_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_index_of_builtin() {
        return array_index_of_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_last_index_of_builtin() {
        return array_last_index_of_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_map_builtin() {
        return array_map_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_reduce_builtin() {
        return array_reduce_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_reduce_right_builtin() {
        return array_reduce_right_builtin(context, invocation).map(Some);
    }
    if entry == super::super::array_some_builtin() {
        return array_some_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

#[derive(Clone, Copy)]
enum ArrayPredicateKind {
    Every,
    Some,
}

fn array_predicate_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: ArrayPredicateKind,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
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
    let receiver = Value::from_object_ref(object_ref);
    for index in 0..length {
        let key = array_like_index_property_key(cx, index);
        if !has_property_on_object(cx, object_ref, key)? {
            continue;
        }
        let value = get_property_from_object(cx, object_ref, key)?;
        let selected = cx.call_to_completion(
            callback,
            this_arg,
            &[value, length_value_u64(index), receiver],
        )?;
        let selected = to_boolean_for_builtin(cx, selected)?;
        match kind {
            ArrayPredicateKind::Every if !selected => return Ok(Value::from_bool(false)),
            ArrayPredicateKind::Some if selected => return Ok(Value::from_bool(true)),
            ArrayPredicateKind::Every | ArrayPredicateKind::Some => {}
        }
    }
    Ok(match kind {
        ArrayPredicateKind::Every => Value::from_bool(true),
        ArrayPredicateKind::Some => Value::from_bool(false),
    })
}

fn array_every_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    array_predicate_builtin(cx, invocation, ArrayPredicateKind::Every)
}

fn array_some_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    array_predicate_builtin(cx, invocation, ArrayPredicateKind::Some)
}

#[derive(Clone, Copy)]
enum ArrayFindKind {
    Find,
    FindIndex,
    FindLast,
    FindLastIndex,
}

fn array_find_builtin_common<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: ArrayFindKind,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
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
    let receiver = Value::from_object_ref(object_ref);

    match kind {
        ArrayFindKind::Find | ArrayFindKind::FindIndex => {
            for index in 0..length {
                let key = array_like_index_property_key(cx, index);
                let value = get_property_from_object(cx, object_ref, key)?;
                let selected = cx.call_to_completion(
                    callback,
                    this_arg,
                    &[value, length_value_u64(index), receiver],
                )?;
                if to_boolean_for_builtin(cx, selected)? {
                    return Ok(match kind {
                        ArrayFindKind::Find => value,
                        ArrayFindKind::FindIndex => length_value_u64(index),
                        ArrayFindKind::FindLast | ArrayFindKind::FindLastIndex => unreachable!(),
                    });
                }
            }
        }
        ArrayFindKind::FindLast | ArrayFindKind::FindLastIndex => {
            if length == 0 {
                return Ok(match kind {
                    ArrayFindKind::FindLast => Value::undefined(),
                    ArrayFindKind::FindLastIndex => Value::from_smi(-1),
                    ArrayFindKind::Find | ArrayFindKind::FindIndex => unreachable!(),
                });
            }
            let mut index = length - 1;
            loop {
                let key = array_like_index_property_key(cx, index);
                let value = get_property_from_object(cx, object_ref, key)?;
                let selected = cx.call_to_completion(
                    callback,
                    this_arg,
                    &[value, length_value_u64(index), receiver],
                )?;
                if to_boolean_for_builtin(cx, selected)? {
                    return Ok(match kind {
                        ArrayFindKind::FindLast => value,
                        ArrayFindKind::FindLastIndex => length_value_u64(index),
                        ArrayFindKind::Find | ArrayFindKind::FindIndex => unreachable!(),
                    });
                }
                if index == 0 {
                    break;
                }
                index -= 1;
            }
        }
    }

    Ok(match kind {
        ArrayFindKind::Find | ArrayFindKind::FindLast => Value::undefined(),
        ArrayFindKind::FindIndex | ArrayFindKind::FindLastIndex => Value::from_smi(-1),
    })
}

fn array_find_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    array_find_builtin_common(cx, invocation, ArrayFindKind::Find)
}

fn array_find_index_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    array_find_builtin_common(cx, invocation, ArrayFindKind::FindIndex)
}

fn array_find_last_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    array_find_builtin_common(cx, invocation, ArrayFindKind::FindLast)
}

fn array_find_last_index_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    array_find_builtin_common(cx, invocation, ArrayFindKind::FindLastIndex)
}

#[derive(Clone, Copy)]
enum ArraySearchKind {
    Includes,
    IndexOf,
}

fn array_search_matches<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: ArraySearchKind,
    search_element: Value,
    element: Value,
) -> Result<bool, Cx::Error> {
    let same = {
        let heap_view = cx.agent().heap().view();
        match kind {
            ArraySearchKind::Includes => read::same_value_zero(heap_view, search_element, element),
            ArraySearchKind::IndexOf => read::is_strictly_equal(heap_view, search_element, element),
        }
    };
    map_completion(cx, same)
}

fn array_search_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: ArraySearchKind,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    let search_element = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if length == 0 {
        return Ok(match kind {
            ArraySearchKind::Includes => Value::from_bool(false),
            ArraySearchKind::IndexOf => Value::from_smi(-1),
        });
    }

    let relative_index = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if relative_index == f64::INFINITY {
        return Ok(match kind {
            ArraySearchKind::Includes => Value::from_bool(false),
            ArraySearchKind::IndexOf => Value::from_smi(-1),
        });
    }
    let start = if relative_index == f64::NEG_INFINITY {
        0
    } else {
        normalize_relative_index_u64(length, relative_index)
    };
    if start >= length {
        return Ok(match kind {
            ArraySearchKind::Includes => Value::from_bool(false),
            ArraySearchKind::IndexOf => Value::from_smi(-1),
        });
    }

    for index in start..length {
        let key = array_like_index_property_key(cx, index);
        let element = match kind {
            ArraySearchKind::Includes => get_property_from_object(cx, object_ref, key)?,
            ArraySearchKind::IndexOf => {
                if !has_property_on_object(cx, object_ref, key)? {
                    continue;
                }
                get_property_from_object(cx, object_ref, key)?
            }
        };
        if array_search_matches(cx, kind, search_element, element)? {
            return Ok(match kind {
                ArraySearchKind::Includes => Value::from_bool(true),
                ArraySearchKind::IndexOf => length_value_u64(index),
            });
        }
    }

    Ok(match kind {
        ArraySearchKind::Includes => Value::from_bool(false),
        ArraySearchKind::IndexOf => Value::from_smi(-1),
    })
}

fn array_includes_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    array_search_builtin(cx, invocation, ArraySearchKind::Includes)
}

pub(in crate::public::dispatch) fn array_index_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    array_search_builtin(cx, invocation, ArraySearchKind::IndexOf)
}

fn array_last_index_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    if length == 0 {
        return Ok(Value::from_smi(-1));
    }
    let from_index = match invocation.arguments().get(1).copied() {
        Some(value) => to_integer_or_infinity_for_builtin(cx, value)?,
        None => array_length_as_number(length - 1),
    };
    if from_index == f64::NEG_INFINITY {
        return Ok(Value::from_smi(-1));
    }
    let mut index = if from_index >= 0.0 {
        if from_index.is_finite() {
            array_index_from_number(from_index).min(length - 1)
        } else {
            length - 1
        }
    } else {
        let computed = array_length_as_number(length) + from_index;
        if computed < 0.0 {
            return Ok(Value::from_smi(-1));
        }
        array_index_from_number(computed)
    };
    let search_element = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    loop {
        let key = array_like_index_property_key(cx, index);
        if has_property_on_object(cx, object_ref, key)? {
            let element = get_property_from_object(cx, object_ref, key)?;
            let equal = {
                let agent = cx.agent();
                read::is_strictly_equal(agent.heap().view(), search_element, element)
            };
            if map_completion(cx, equal)? {
                return Ok(length_value_u64(index));
            }
        }
        if index == 0 {
            break;
        }
        index -= 1;
    }
    Ok(Value::from_smi(-1))
}

#[derive(Clone, Copy)]
enum ArrayReduceDirection {
    Forward,
    Reverse,
}

fn array_reduce_common<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    direction: ArrayReduceDirection,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    let callback = cx.require_callable_object(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let receiver = Value::from_object_ref(object_ref);

    let mut accumulator;
    let mut next_index;
    if let Some(initial_value) = invocation.arguments().get(1).copied() {
        accumulator = initial_value;
        next_index = match direction {
            ArrayReduceDirection::Forward => Some(0),
            ArrayReduceDirection::Reverse => length.checked_sub(1),
        };
    } else {
        if length == 0 {
            return Err(type_error(cx));
        }
        match direction {
            ArrayReduceDirection::Forward => {
                let mut index = 0_u64;
                loop {
                    let key = array_like_index_property_key(cx, index);
                    if has_property_on_object(cx, object_ref, key)? {
                        accumulator = get_property_from_object(cx, object_ref, key)?;
                        next_index = index.checked_add(1);
                        break;
                    }
                    index += 1;
                    if index >= length {
                        return Err(type_error(cx));
                    }
                }
            }
            ArrayReduceDirection::Reverse => {
                let mut index = length - 1;
                loop {
                    let key = array_like_index_property_key(cx, index);
                    if has_property_on_object(cx, object_ref, key)? {
                        accumulator = get_property_from_object(cx, object_ref, key)?;
                        next_index = index.checked_sub(1);
                        break;
                    }
                    if index == 0 {
                        return Err(type_error(cx));
                    }
                    index -= 1;
                }
            }
        }
    }

    match direction {
        ArrayReduceDirection::Forward => {
            while let Some(index) = next_index {
                if index >= length {
                    break;
                }
                let key = array_like_index_property_key(cx, index);
                if has_property_on_object(cx, object_ref, key)? {
                    let value = get_property_from_object(cx, object_ref, key)?;
                    accumulator = cx.call_to_completion(
                        callback,
                        Value::undefined(),
                        &[accumulator, value, length_value_u64(index), receiver],
                    )?;
                }
                next_index = index.checked_add(1);
            }
        }
        ArrayReduceDirection::Reverse => {
            while let Some(index) = next_index {
                let key = array_like_index_property_key(cx, index);
                if has_property_on_object(cx, object_ref, key)? {
                    let value = get_property_from_object(cx, object_ref, key)?;
                    accumulator = cx.call_to_completion(
                        callback,
                        Value::undefined(),
                        &[accumulator, value, length_value_u64(index), receiver],
                    )?;
                }
                next_index = index.checked_sub(1);
            }
        }
    }

    Ok(accumulator)
}

fn array_reduce_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    array_reduce_common(cx, invocation, ArrayReduceDirection::Forward)
}

fn array_reduce_right_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    array_reduce_common(cx, invocation, ArrayReduceDirection::Reverse)
}

fn array_filter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length(cx, object_ref)?;
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
    let result = array_species_create_for_length(cx, object_ref, 0)?;
    let mut to = 0_u32;
    for index in 0..length {
        let key = PropertyKey::Index(index);
        if !has_property_on_object(cx, object_ref, key)? {
            continue;
        }
        let value = get_property_from_object(cx, object_ref, key)?;
        let selected = cx.call_to_completion(
            callback,
            this_arg,
            &[
                value,
                length_value(index),
                Value::from_object_ref(object_ref),
            ],
        )?;
        if to_boolean_for_builtin(cx, selected)? {
            create_data_property_or_throw(cx, result, PropertyKey::Index(to), value)?;
            to = to.saturating_add(1);
        }
    }
    Ok(Value::from_object_ref(result))
}

fn array_flatten_into_array<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    target: ObjectRef,
    source: ObjectRef,
    source_len: u64,
    start: u64,
    depth: u64,
    mapper: Option<(ObjectRef, Value)>,
) -> Result<u64, Cx::Error> {
    let mut target_index = start;
    for source_index in 0..source_len {
        let source_key = array_like_index_property_key(cx, source_index);
        if !has_property_on_object(cx, source, source_key)? {
            continue;
        }
        let mut element = get_property_from_object(cx, source, source_key)?;
        if let Some((mapper, this_arg)) = mapper {
            element = cx.call_to_completion(
                mapper,
                this_arg,
                &[
                    element,
                    length_value_u64(source_index),
                    Value::from_object_ref(source),
                ],
            )?;
        }

        let should_flatten = if depth > 0 {
            if let Some(element_object) = element.as_object_ref() {
                is_array_for_species(cx, element_object)?
            } else {
                false
            }
        } else {
            false
        };

        if should_flatten {
            let element_object = element
                .as_object_ref()
                .expect("flattenable element should be an object");
            let element_len = array_like_length_u64(cx, element_object)?;
            target_index = array_flatten_into_array(
                cx,
                target,
                element_object,
                element_len,
                target_index,
                depth.saturating_sub(1),
                None,
            )?;
        } else {
            if target_index >= MAX_SAFE_INTEGER_U64 {
                return Err(type_error(cx));
            }
            let target_key = array_like_index_property_key(cx, target_index);
            create_data_property_or_throw(cx, target, target_key, element)?;
            target_index += 1;
        }
    }
    Ok(target_index)
}

fn array_flat_depth<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    depth_value: Value,
) -> Result<u64, Cx::Error> {
    if depth_value.is_undefined() {
        return Ok(1);
    }
    let depth = to_integer_or_infinity_for_builtin(cx, depth_value)?;
    if depth <= 0.0 || depth.is_nan() {
        Ok(0)
    } else if depth.is_infinite() {
        Ok(u64::MAX)
    } else {
        Ok(array_index_from_number(depth))
    }
}

fn array_flat_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let source_len = array_like_length_u64(cx, object_ref)?;
    let depth = array_flat_depth(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let result = array_species_create_for_length(cx, object_ref, 0)?;
    array_flatten_into_array(cx, result, object_ref, source_len, 0, depth, None)?;
    Ok(Value::from_object_ref(result))
}

fn array_flat_map_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let source_len = array_like_length_u64(cx, object_ref)?;
    let mapper = cx.require_callable_object(
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
    let result = array_species_create_for_length(cx, object_ref, 0)?;
    array_flatten_into_array(
        cx,
        result,
        object_ref,
        source_len,
        0,
        1,
        Some((mapper, this_arg)),
    )?;
    Ok(Value::from_object_ref(result))
}

fn array_for_each_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length(cx, object_ref)?;
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
    for index in 0..length {
        let key = PropertyKey::Index(index);
        if !has_property_on_object(cx, object_ref, key)? {
            continue;
        }
        let value = get_property_from_object(cx, object_ref, key)?;
        let _ = cx.call_to_completion(
            callback,
            this_arg,
            &[
                value,
                length_value(index),
                Value::from_object_ref(object_ref),
            ],
        )?;
    }
    Ok(Value::undefined())
}

fn array_map_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
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
    let result = array_species_create_for_length(cx, object_ref, length)?;
    for index in 0..length {
        let key = array_like_index_property_key(cx, index);
        if !has_property_on_object(cx, object_ref, key)? {
            continue;
        }
        let value = get_property_from_object(cx, object_ref, key)?;
        let mapped = cx.call_to_completion(
            callback,
            this_arg,
            &[
                value,
                length_value_u64(index),
                Value::from_object_ref(object_ref),
            ],
        )?;
        create_data_property_or_throw(cx, result, key, mapped)?;
    }
    Ok(Value::from_object_ref(result))
}

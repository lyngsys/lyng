use super::super::super::{
    typed_array_includes_builtin, typed_array_index_of_builtin, typed_array_join_builtin,
    typed_array_last_index_of_builtin,
};
use super::super::{
    length_value_u64, map_completion, normalize_relative_index_u64, range_error, string_value,
    to_integer_or_infinity_for_builtin, type_error, PublicBuiltinDispatchContext,
};
use super::{
    typed_array_read_element_value, typed_array_read_storage_bits,
    typed_array_storage_bits_to_value, typed_array_this_record, typed_array_validated_record,
};
use crate::BuiltinInvocation;
use lyng_js_ops::read;
use lyng_js_types::{BuiltinFunctionId, Value};

pub(in crate::public::dispatch::binary_data) fn dispatch_typed_array_search_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == typed_array_includes_builtin() {
        return typed_array_includes_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_index_of_builtin() {
        return typed_array_index_of_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_join_builtin() {
        return typed_array_join_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == typed_array_last_index_of_builtin() {
        return typed_array_last_index_of_builtin_dispatch(context, invocation).map(Some);
    }
    Ok(None)
}

fn typed_array_join_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = typed_array_validated_record(cx, invocation.this_value())?;
    let separator = match invocation.arguments().first().copied() {
        Some(value) if !value.is_undefined() => cx.value_to_string_text(value)?,
        _ => ",".to_owned(),
    };
    let mut text = String::new();
    for index in 0..record.length() {
        if index != 0 {
            text.push_str(&separator);
        }
        let value = typed_array_read_element_value(cx.agent(), record, index);
        if value.is_undefined() || value.is_null() {
            continue;
        }
        text.push_str(&cx.value_to_string_text(value)?);
    }
    Ok(string_value(cx, &text))
}

#[derive(Clone, Copy)]
enum TypedArraySearchKind {
    Includes,
    IndexOf,
    LastIndexOf,
}

fn typed_array_search_matches<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: TypedArraySearchKind,
    search_element: Value,
    element: Value,
) -> Result<bool, Cx::Error> {
    let heap_view = cx.agent().heap().view();
    let same = match kind {
        TypedArraySearchKind::Includes => read::same_value_zero(heap_view, search_element, element),
        TypedArraySearchKind::IndexOf | TypedArraySearchKind::LastIndexOf => {
            read::is_strictly_equal(heap_view, search_element, element)
        }
    };
    map_completion(cx, same)
}

fn typed_array_search_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: TypedArraySearchKind,
) -> Result<Value, Cx::Error> {
    let record = typed_array_this_record(cx, invocation.this_value())?;
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    let length = u64::try_from(record.length()).unwrap_or(u64::MAX);
    let search_element = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if length == 0 {
        return Ok(match kind {
            TypedArraySearchKind::Includes => Value::from_bool(false),
            TypedArraySearchKind::IndexOf | TypedArraySearchKind::LastIndexOf => {
                Value::from_smi(-1)
            }
        });
    }

    match kind {
        TypedArraySearchKind::Includes | TypedArraySearchKind::IndexOf => {
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
                    TypedArraySearchKind::Includes => Value::from_bool(false),
                    TypedArraySearchKind::IndexOf => Value::from_smi(-1),
                    TypedArraySearchKind::LastIndexOf => unreachable!(),
                });
            }
            let start = if relative_index == f64::NEG_INFINITY {
                0
            } else {
                normalize_relative_index_u64(length, relative_index)
            };
            if start >= length {
                return Ok(match kind {
                    TypedArraySearchKind::Includes => Value::from_bool(false),
                    TypedArraySearchKind::IndexOf => Value::from_smi(-1),
                    TypedArraySearchKind::LastIndexOf => unreachable!(),
                });
            }
            if cx
                .agent()
                .backing_store_is_detached(record.backing_store())
                .ok_or_else(|| type_error(cx))?
            {
                return Ok(match kind {
                    TypedArraySearchKind::Includes => {
                        Value::from_bool(search_element.is_undefined())
                    }
                    TypedArraySearchKind::IndexOf => Value::from_smi(-1),
                    TypedArraySearchKind::LastIndexOf => unreachable!(),
                });
            }
            for index in start..length {
                let index = usize::try_from(index).map_err(|_| range_error(cx))?;
                let bits = typed_array_read_storage_bits(cx.agent(), record, index)
                    .ok_or_else(|| type_error(cx))?;
                let element = typed_array_storage_bits_to_value(cx.agent(), record.kind(), bits);
                if typed_array_search_matches(cx, kind, search_element, element)? {
                    return Ok(match kind {
                        TypedArraySearchKind::Includes => Value::from_bool(true),
                        TypedArraySearchKind::IndexOf => {
                            length_value_u64(u64::try_from(index).unwrap_or(u64::MAX))
                        }
                        TypedArraySearchKind::LastIndexOf => unreachable!(),
                    });
                }
            }
            Ok(match kind {
                TypedArraySearchKind::Includes => Value::from_bool(false),
                TypedArraySearchKind::IndexOf => Value::from_smi(-1),
                TypedArraySearchKind::LastIndexOf => unreachable!(),
            })
        }
        TypedArraySearchKind::LastIndexOf => {
            let relative_index = match invocation.arguments().get(1).copied() {
                Some(value) => to_integer_or_infinity_for_builtin(cx, value)?,
                None => (length.saturating_sub(1)) as f64,
            };
            if relative_index == f64::NEG_INFINITY {
                return Ok(Value::from_smi(-1));
            }
            let start = if relative_index == f64::INFINITY {
                length.saturating_sub(1)
            } else if relative_index >= 0.0 {
                (relative_index.min((length.saturating_sub(1)) as f64)) as u64
            } else {
                let computed = (length as f64) + relative_index;
                if computed < 0.0 {
                    return Ok(Value::from_smi(-1));
                }
                computed as u64
            };
            if cx
                .agent()
                .backing_store_is_detached(record.backing_store())
                .ok_or_else(|| type_error(cx))?
            {
                return Ok(Value::from_smi(-1));
            }
            let mut index = usize::try_from(start).map_err(|_| range_error(cx))?;
            loop {
                let bits = typed_array_read_storage_bits(cx.agent(), record, index)
                    .ok_or_else(|| type_error(cx))?;
                let element = typed_array_storage_bits_to_value(cx.agent(), record.kind(), bits);
                if typed_array_search_matches(cx, kind, search_element, element)? {
                    return Ok(length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)));
                }
                if index == 0 {
                    break;
                }
                index -= 1;
            }
            Ok(Value::from_smi(-1))
        }
    }
}

fn typed_array_includes_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_search_builtin_dispatch(cx, invocation, TypedArraySearchKind::Includes)
}

fn typed_array_index_of_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_search_builtin_dispatch(cx, invocation, TypedArraySearchKind::IndexOf)
}

fn typed_array_last_index_of_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    typed_array_search_builtin_dispatch(cx, invocation, TypedArraySearchKind::LastIndexOf)
}

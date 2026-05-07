use super::super::super::{
    typed_array_includes_builtin, typed_array_index_of_builtin, typed_array_join_builtin,
    typed_array_last_index_of_builtin,
};
use super::super::{
    length_value_u64, map_completion, normalize_relative_index_u64, range_error,
    string_from_code_units, string_ref_code_units, to_integer_or_infinity_for_builtin,
    to_string_string_ref, PublicBuiltinDispatchContext,
};
use super::{
    typed_array_read_element_value, typed_array_read_storage_bits,
    typed_array_storage_bits_to_value, typed_array_validated_record_and_length,
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
    let (record, length) = typed_array_validated_record_and_length(cx, invocation.this_value())?;
    let separator = match invocation.arguments().first().copied() {
        Some(value) if !value.is_undefined() => {
            let separator_ref = to_string_string_ref(cx, value)?;
            Some(string_ref_code_units(cx, separator_ref)?)
        }
        _ => None,
    };
    let mut units = Vec::new();
    for index in 0..length {
        if index != 0 {
            if let Some(separator) = separator.as_deref() {
                units.extend_from_slice(separator);
            } else {
                units.push(u16::from(b','));
            }
        }
        let value = typed_array_read_element_value(cx.agent(), record, index);
        if value.is_undefined() || value.is_null() {
            continue;
        }
        units.extend(cx.value_to_string_text(value)?.encode_utf16());
    }
    Ok(string_from_code_units(cx, &units))
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

fn typed_array_last_index_of_start<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    arguments: &[Value],
    length: u64,
) -> Result<Option<u64>, Cx::Error> {
    let Some(value) = arguments.get(1).copied() else {
        return Ok(Some(length.saturating_sub(1)));
    };
    let relative_index = to_integer_or_infinity_for_builtin(cx, value)?;
    if relative_index == f64::NEG_INFINITY {
        return Ok(None);
    }
    if relative_index == f64::INFINITY {
        return Ok(Some(length.saturating_sub(1)));
    }
    if relative_index < 0.0 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "TypedArray.prototype.lastIndexOf compares Number indices with array length"
        )]
        let length_number = length as f64;
        if relative_index.abs() > length_number {
            return Ok(None);
        }
    }
    Ok(Some(
        normalize_relative_index_u64(length, relative_index).min(length.saturating_sub(1)),
    ))
}

fn typed_array_search_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: TypedArraySearchKind,
) -> Result<Value, Cx::Error> {
    let (record, length) = typed_array_validated_record_and_length(cx, invocation.this_value())?;
    let length = u64::try_from(length).unwrap_or(u64::MAX);
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
            for index in start..length {
                let index = usize::try_from(index).map_err(|_| range_error(cx))?;
                let Some(bits) = typed_array_read_storage_bits(cx.agent(), record, index) else {
                    if matches!(kind, TypedArraySearchKind::Includes)
                        && search_element.is_undefined()
                    {
                        return Ok(Value::from_bool(true));
                    }
                    continue;
                };
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
            let Some(start) = typed_array_last_index_of_start(cx, invocation.arguments(), length)?
            else {
                return Ok(Value::from_smi(-1));
            };
            let mut index = usize::try_from(start).map_err(|_| range_error(cx))?;
            loop {
                if let Some(bits) = typed_array_read_storage_bits(cx.agent(), record, index) {
                    let element =
                        typed_array_storage_bits_to_value(cx.agent(), record.kind(), bits);
                    if typed_array_search_matches(cx, kind, search_element, element)? {
                        return Ok(length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)));
                    }
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

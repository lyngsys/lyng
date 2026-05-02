mod iteration;

pub(super) use iteration::array_index_of_builtin;
use iteration::dispatch_array_iteration_builtin;

use super::{
    array_like_index_property_key, array_like_join_text_for_length, array_like_length,
    array_like_length_u64, array_result_capacity_hint, array_species_create_for_length,
    close_iterator_after_error, collect_array_like_values_for_from_builtin, create_array_result,
    create_array_result_for_length, create_array_result_with_prototype,
    create_data_property_or_throw, define_array_length, delete_property_from_object,
    get_property_from_object, has_property_on_object, is_array_for_species, is_concat_spreadable,
    iterators::{array_iterator_factory_builtin, array_iterator_next_builtin, ArrayIterationKind},
    length_value_u64, normalize_relative_index_u64, objects, property_key_from_text, range_error,
    set_length_property, set_property_on_object, string_value, to_integer_or_infinity_for_builtin,
    to_number_for_builtin, type_error, valid_array_length, BuiltinIteratorBridge,
    PublicBuiltinDispatchContext, MAX_SAFE_INTEGER_U64,
};
use crate::{BuiltinInvocation, DynamicFunctionKind};
use lyng_js_common::WellKnownAtom;
use lyng_js_ops::iterator;
use lyng_js_types::{BuiltinFunctionId, ObjectRef, PropertyKey, Value, WellKnownSymbolId};

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
    if entry == super::array_builtin() {
        return array_builtin(context, invocation).map(Some);
    }
    if entry == super::array_from_builtin() {
        return array_from_builtin(context, invocation).map(Some);
    }
    if entry == super::array_from_async_builtin() {
        return array_from_async_builtin(context, invocation).map(Some);
    }
    if entry == super::array_of_builtin() {
        return array_of_builtin(context, invocation).map(Some);
    }
    if entry == super::array_is_array_builtin() {
        return array_is_array_builtin(context, invocation).map(Some);
    }
    if entry == super::array_species_getter_builtin() {
        return array_species_getter_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_array_indexed_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::array_at_builtin() {
        return array_at_builtin(context, invocation).map(Some);
    }
    if entry == super::array_concat_builtin() {
        return array_concat_builtin(context, invocation).map(Some);
    }
    if entry == super::array_copy_within_builtin() {
        return array_copy_within_builtin(context, invocation).map(Some);
    }
    if entry == super::array_fill_builtin() {
        return array_fill_builtin(context, invocation).map(Some);
    }
    if entry == super::array_join_builtin() {
        return array_join_builtin(context, invocation).map(Some);
    }
    if entry == super::array_pop_builtin() {
        return array_pop_builtin(context, invocation).map(Some);
    }
    if entry == super::array_push_builtin() {
        return array_push_builtin(context, invocation).map(Some);
    }
    if entry == super::array_shift_builtin() {
        return array_shift_builtin(context, invocation).map(Some);
    }
    if entry == super::array_unshift_builtin() {
        return array_unshift_builtin(context, invocation).map(Some);
    }
    if entry == super::array_reverse_builtin() {
        return array_reverse_builtin(context, invocation).map(Some);
    }
    if entry == super::array_slice_builtin() {
        return array_slice_builtin(context, invocation).map(Some);
    }
    if entry == super::array_sort_builtin() {
        return array_sort_builtin(context, invocation).map(Some);
    }
    if entry == super::array_splice_builtin() {
        return array_splice_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_array_copying_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::array_to_reversed_builtin() {
        return array_to_reversed_builtin(context, invocation).map(Some);
    }
    if entry == super::array_to_sorted_builtin() {
        return array_to_sorted_builtin(context, invocation).map(Some);
    }
    if entry == super::array_to_spliced_builtin() {
        return array_to_spliced_builtin(context, invocation).map(Some);
    }
    if entry == super::array_to_string_builtin() {
        return array_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::array_to_locale_string_builtin() {
        return array_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == super::array_values_builtin() {
        return array_iterator_factory_builtin(context, invocation, ArrayIterationKind::Value)
            .map(Some);
    }
    if entry == super::array_keys_builtin() {
        return array_iterator_factory_builtin(context, invocation, ArrayIterationKind::Key)
            .map(Some);
    }
    if entry == super::array_entries_builtin() {
        return array_iterator_factory_builtin(context, invocation, ArrayIterationKind::Entry)
            .map(Some);
    }
    if entry == super::array_iterator_next_builtin() {
        return array_iterator_next_builtin(context, invocation).map(Some);
    }
    if entry == super::array_with_builtin() {
        return array_with_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let arguments = invocation.arguments();
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().array_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let new_target = invocation
        .new_target()
        .unwrap_or_else(|| cx.callee_object());
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let array = create_array_result_with_prototype(cx, realm, prototype, arguments.len())?;
    if arguments.is_empty() {
        return Ok(Value::from_object_ref(array));
    }

    if arguments.len() == 1 {
        if arguments[0].as_smi().is_some() || arguments[0].as_f64().is_some() {
            let number = to_number_for_builtin(cx, arguments[0])?;
            let Some(length) = valid_array_length(number) else {
                return Err(range_error(cx));
            };
            define_array_length(cx, array, length)?;
            return Ok(Value::from_object_ref(array));
        }
    }

    for (index, value) in arguments.iter().copied().enumerate() {
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        set_property_on_object(cx, array, PropertyKey::Index(index), value)?;
    }
    Ok(Value::from_object_ref(array))
}

fn array_is_array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let is_array = match value.as_object_ref() {
        Some(object) => is_array_for_species(cx, object)?,
        None => false,
    };
    Ok(Value::from_bool(is_array))
}

fn get_sync_iterator_from_method<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    iterable: Value,
    iterator_method: ObjectRef,
) -> Result<iterator::IteratorRecord, Cx::Error> {
    let iterable_object = cx.to_object_for_builtin_value(cx.caller_realm(), iterable)?;
    let iterator = cx.call_to_completion(
        iterator_method,
        Value::from_object_ref(iterable_object),
        &[],
    )?;
    let iterator_object = iterator.as_object_ref().ok_or_else(|| type_error(cx))?;
    let next_key = property_key_from_text(cx, "next");
    let next_value = cx.get_property_value(Value::from_object_ref(iterator_object), next_key)?;
    let next_method = cx.require_callable_object(next_value)?;
    Ok(iterator::IteratorRecord::new(iterator_object, next_method))
}

fn array_from_iterable_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor_receiver: Value,
    iterable: Value,
    iterator_method: ObjectRef,
    mapper: Option<ObjectRef>,
    this_arg: Value,
) -> Result<Value, Cx::Error> {
    let array = array_from_result_object(cx, constructor_receiver, 0, true)?;
    let mut iterator_record = get_sync_iterator_from_method(cx, iterable, iterator_method)?;
    let mut index = 0_u64;

    loop {
        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut iterator_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => {
                iterator_record.set_done(true);
                return Err(error);
            }
        };
        let Some(next) = next else {
            set_length_property(cx, array, index)?;
            return Ok(Value::from_object_ref(array));
        };
        let next_value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let next_value = match next_value {
            Ok(next_value) => next_value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        if index >= MAX_SAFE_INTEGER_U64 {
            let error = type_error(cx);
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
        let mapped = if let Some(mapper) = mapper {
            match cx.call_to_completion(mapper, this_arg, &[next_value, length_value_u64(index)]) {
                Ok(mapped) => mapped,
                Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
            }
        } else {
            next_value
        };
        let key = array_like_index_property_key(cx, index);
        if let Err(error) = create_data_property_or_throw(cx, array, key, mapped) {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
        index += 1;
    }
}

fn array_from_result_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor_receiver: Value,
    source_len: usize,
    used_iterator: bool,
) -> Result<ObjectRef, Cx::Error> {
    let constructor = constructor_receiver
        .as_object_ref()
        .filter(|object| cx.agent().objects().is_constructor(*object));
    match constructor {
        Some(constructor) if used_iterator => cx.construct_to_completion(constructor, &[], None),
        Some(constructor) => cx.construct_to_completion(
            constructor,
            &[length_value_u64(
                u64::try_from(source_len).unwrap_or(u64::MAX),
            )],
            None,
        ),
        None => create_array_result(cx, source_len),
    }
}

fn array_from_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let mapper = match invocation.arguments().get(1).copied() {
        Some(mapper) if !mapper.is_undefined() => Some(cx.require_callable_object(mapper)?),
        _ => None,
    };
    let this_arg = invocation
        .arguments()
        .get(2)
        .copied()
        .unwrap_or(Value::undefined());
    if let Some(iterator_symbol) = cx.agent().well_known_symbol(WellKnownSymbolId::Iterator) {
        let iterator_method =
            cx.get_property_value(source, PropertyKey::from_symbol(iterator_symbol))?;
        if !(iterator_method.is_undefined() || iterator_method.is_null()) {
            let iterator_method = cx.require_callable_object(iterator_method)?;
            return array_from_iterable_builtin(
                cx,
                invocation.this_value(),
                source,
                iterator_method,
                mapper,
                this_arg,
            );
        }
    }
    let values = collect_array_like_values_for_from_builtin(cx, source)?;
    let array = array_from_result_object(cx, invocation.this_value(), values.len(), false)?;
    for (index, value) in values.iter().copied().enumerate() {
        let mapped = if let Some(mapper) = mapper {
            cx.call_to_completion(
                mapper,
                this_arg,
                &[
                    value,
                    length_value_u64(u64::try_from(index).unwrap_or(u64::MAX)),
                ],
            )?
        } else {
            value
        };
        let key = array_like_index_property_key(cx, u64::try_from(index).unwrap_or(u64::MAX));
        create_data_property_or_throw(cx, array, key, mapped)?;
    }
    set_length_property(cx, array, u64::try_from(values.len()).unwrap_or(u64::MAX))?;
    Ok(Value::from_object_ref(array))
}

const ARRAY_FROM_ASYNC_DYNAMIC_PARAMETERS: &str =
    "asyncItems, mapfn, thisArg, iteratorSymbol, asyncIteratorSymbol";

const ARRAY_FROM_ASYNC_DYNAMIC_BODY: &str = r#"
"use strict";

const MAX_SAFE_LENGTH = 9007199254740991;

function isObject(value) {
    return value !== null && (typeof value === "object" || typeof value === "function");
}

function isConstructor(value) {
    if (!isObject(value)) {
        return false;
    }
    try {
        Reflect.construct(function() {}, [], value);
        return true;
    } catch (error) {
        return false;
    }
}

function getMethod(value, key) {
    const method = value[key];
    if (method === undefined || method === null) {
        return undefined;
    }
    if (typeof method !== "function") {
        throw new TypeError();
    }
    return method;
}

function toLength(value) {
    if (typeof value === "bigint") {
        throw new TypeError();
    }
    let length = Number(value);
    if (length !== length || length <= 0) {
        return 0;
    }
    if (length === Infinity) {
        return MAX_SAFE_LENGTH;
    }
    length = Math.floor(length);
    if (length > MAX_SAFE_LENGTH) {
        return MAX_SAFE_LENGTH;
    }
    return length;
}

function createDataProperty(object, index, value) {
    Object.defineProperty(object, index, {
        value,
        writable: true,
        enumerable: true,
        configurable: true
    });
}

async function closeIterator(iterator, completion) {
    const returnMethod = getMethod(iterator, "return");
    if (returnMethod !== undefined) {
        let innerResult = returnMethod.call(iterator);
        if (isObject(innerResult)) {
            innerResult = await innerResult;
        }
        if (!isObject(innerResult)) {
            throw new TypeError();
        }
    }
    throw completion;
}

async function collectIterator(iterator, nextMethod, array, mapping, mapfn, thisArg, syncIterator) {
    let index = 0;
    while (true) {
        let next = nextMethod.call(iterator);
        if (!syncIterator && isObject(next)) {
            next = await next;
        }
        if (!isObject(next)) {
            throw new TypeError();
        }
        if (next.done) {
            array.length = index;
            return array;
        }

        let value = next.value;
        if (syncIterator || mapping) {
            try {
                value = await value;
            } catch (error) {
                await closeIterator(iterator, error);
            }
        }

        if (index >= MAX_SAFE_LENGTH) {
            await closeIterator(iterator, new TypeError());
        }

        let mapped = value;
        if (mapping) {
            try {
                mapped = mapfn.call(thisArg, value, index);
            } catch (error) {
                await closeIterator(iterator, error);
            }
            try {
                mapped = await mapped;
            } catch (error) {
                await closeIterator(iterator, error);
            }
        }

        try {
            createDataProperty(array, index, mapped);
        } catch (error) {
            await closeIterator(iterator, error);
        }
        index += 1;
    }
}

const mapping = mapfn !== undefined;
if (mapping && typeof mapfn !== "function") {
    throw new TypeError();
}
if (asyncItems === null || asyncItems === undefined) {
    throw new TypeError();
}

const constructor = isConstructor(this);
let iteratorMethod = getMethod(asyncItems, asyncIteratorSymbol);
let syncIterator = false;
if (iteratorMethod === undefined) {
    iteratorMethod = getMethod(asyncItems, iteratorSymbol);
    syncIterator = iteratorMethod !== undefined;
}

if (iteratorMethod !== undefined) {
    const iterator = iteratorMethod.call(asyncItems);
    if (!isObject(iterator)) {
        throw new TypeError();
    }
    const nextMethod = getMethod(iterator, "next");
    if (nextMethod === undefined) {
        throw new TypeError();
    }
    const array = constructor ? Reflect.construct(this, []) : [];
    return collectIterator(iterator, nextMethod, array, mapping, mapfn, thisArg, syncIterator);
}

const arrayLike = Object(asyncItems);
const length = toLength(arrayLike.length);
const array = constructor ? Reflect.construct(this, [length]) : new Array(length);
for (let index = 0; index < length; index += 1) {
    let value = arrayLike[index];
    value = await value;
    if (mapping) {
        value = mapfn.call(thisArg, value, index);
        value = await value;
    }
    createDataProperty(array, index, value);
}
array.length = length;
return array;
"#;

fn array_from_async_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let function = cx.create_dynamic_function(
        realm,
        ARRAY_FROM_ASYNC_DYNAMIC_PARAMETERS,
        ARRAY_FROM_ASYNC_DYNAMIC_BODY,
        true,
        DynamicFunctionKind::Async,
        None,
    )?;
    let iterator_symbol = cx
        .agent()
        .well_known_symbol(WellKnownSymbolId::Iterator)
        .ok_or_else(|| type_error(cx))?;
    let async_iterator_symbol = cx
        .agent()
        .well_known_symbol(WellKnownSymbolId::AsyncIterator)
        .ok_or_else(|| type_error(cx))?;
    let mut arguments = Vec::with_capacity(5);
    arguments.push(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    );
    arguments.push(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    );
    arguments.push(
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    );
    arguments.push(Value::from_symbol_ref(iterator_symbol));
    arguments.push(Value::from_symbol_ref(async_iterator_symbol));
    cx.call_to_completion(function, invocation.this_value(), &arguments)
}

fn array_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let length = invocation.arguments().len();
    let array = array_from_result_object(cx, invocation.this_value(), length, false)?;
    for (index, value) in invocation.arguments().iter().copied().enumerate() {
        let key = array_like_index_property_key(cx, u64::try_from(index).unwrap_or(u64::MAX));
        create_data_property_or_throw(cx, array, key, value)?;
    }
    set_length_property(cx, array, u64::try_from(length).unwrap_or(u64::MAX))?;
    Ok(Value::from_object_ref(array))
}

fn array_species_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    _cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Ok(invocation.this_value())
}

fn array_at_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    let relative_index = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let actual_index = if relative_index < 0.0 {
        length as f64 + relative_index
    } else {
        relative_index
    };
    if !actual_index.is_finite() || actual_index < 0.0 || actual_index >= length as f64 {
        return Ok(Value::undefined());
    }
    let key = array_like_index_property_key(cx, actual_index as u64);
    get_property_from_object(cx, object_ref, key)
}

fn array_concat_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let result = array_species_create_for_length(cx, object_ref, 0)?;
    let mut next_index = 0_u64;
    for value in std::iter::once(Value::from_object_ref(object_ref))
        .chain(invocation.arguments().iter().copied())
    {
        if let Some(source_object) = value.as_object_ref() {
            if is_concat_spreadable(cx, value)? {
                let length = array_like_length_u64(cx, source_object)?;
                let Some(limit) = next_index.checked_add(length) else {
                    return Err(type_error(cx));
                };
                if limit > MAX_SAFE_INTEGER_U64 {
                    return Err(type_error(cx));
                }
                for index in 0..length {
                    let source_key = array_like_index_property_key(cx, index);
                    if has_property_on_object(cx, source_object, source_key)? {
                        let item = get_property_from_object(cx, source_object, source_key)?;
                        let target_key = array_like_index_property_key(cx, next_index);
                        create_data_property_or_throw(cx, result, target_key, item)?;
                    }
                    next_index = next_index.saturating_add(1);
                }
                continue;
            }
        }
        if next_index >= MAX_SAFE_INTEGER_U64 {
            return Err(type_error(cx));
        }
        let target_key = array_like_index_property_key(cx, next_index);
        create_data_property_or_throw(cx, result, target_key, value)?;
        next_index = next_index.saturating_add(1);
    }
    set_length_property(cx, result, next_index)?;
    Ok(Value::from_object_ref(result))
}

fn array_copy_within_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    let target = normalize_relative_index_u64(
        length,
        to_integer_or_infinity_for_builtin(
            cx,
            invocation
                .arguments()
                .first()
                .copied()
                .unwrap_or(Value::undefined()),
        )?,
    );
    let start = normalize_relative_index_u64(
        length,
        to_integer_or_infinity_for_builtin(
            cx,
            invocation
                .arguments()
                .get(1)
                .copied()
                .unwrap_or(Value::undefined()),
        )?,
    );
    let end = normalize_relative_index_u64(
        length,
        match invocation.arguments().get(2).copied() {
            Some(value) if value.is_undefined() => length as f64,
            Some(value) => to_integer_or_infinity_for_builtin(cx, value)?,
            None => length as f64,
        },
    );
    let count = end.saturating_sub(start).min(length.saturating_sub(target));
    if count == 0 {
        return Ok(Value::from_object_ref(object_ref));
    }

    let (mut from, mut to, forward) = if start < target && target < start.saturating_add(count) {
        (start + count - 1, target + count - 1, false)
    } else {
        (start, target, true)
    };
    let mut remaining = count;
    while remaining > 0 {
        let from_key = array_like_index_property_key(cx, from);
        let to_key = array_like_index_property_key(cx, to);
        if has_property_on_object(cx, object_ref, from_key)? {
            let value = get_property_from_object(cx, object_ref, from_key)?;
            set_property_on_object(cx, object_ref, to_key, value)?;
        } else {
            delete_property_from_object(cx, object_ref, to_key)?;
        }

        remaining -= 1;
        if remaining == 0 {
            break;
        }
        if forward {
            from += 1;
            to += 1;
        } else {
            from -= 1;
            to -= 1;
        }
    }
    Ok(Value::from_object_ref(object_ref))
}

fn array_fill_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    let start = normalize_relative_index_u64(
        length,
        to_integer_or_infinity_for_builtin(
            cx,
            invocation
                .arguments()
                .get(1)
                .copied()
                .unwrap_or(Value::undefined()),
        )?,
    );
    let end = match invocation.arguments().get(2).copied() {
        Some(value) if !value.is_undefined() => {
            normalize_relative_index_u64(length, to_integer_or_infinity_for_builtin(cx, value)?)
        }
        _ => length,
    };
    let fill_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    for index in start..end {
        let key = array_like_index_property_key(cx, index);
        set_property_on_object(cx, object_ref, key, fill_value)?;
    }
    Ok(Value::from_object_ref(object_ref))
}

fn array_reverse_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    let middle = length / 2;
    let mut lower = 0_u64;
    while lower < middle {
        let upper = length - lower - 1;
        let lower_key = array_like_index_property_key(cx, lower);
        let upper_key = array_like_index_property_key(cx, upper);
        let lower_present = has_property_on_object(cx, object_ref, lower_key)?;
        let lower_value = if lower_present {
            Some(get_property_from_object(cx, object_ref, lower_key)?)
        } else {
            None
        };
        let upper_present = has_property_on_object(cx, object_ref, upper_key)?;
        let upper_value = if upper_present {
            Some(get_property_from_object(cx, object_ref, upper_key)?)
        } else {
            None
        };

        match (lower_value, upper_value) {
            (Some(lower_value), Some(upper_value)) => {
                set_property_on_object(cx, object_ref, lower_key, upper_value)?;
                set_property_on_object(cx, object_ref, upper_key, lower_value)?;
            }
            (None, Some(upper_value)) => {
                set_property_on_object(cx, object_ref, lower_key, upper_value)?;
                delete_property_from_object(cx, object_ref, upper_key)?;
            }
            (Some(lower_value), None) => {
                delete_property_from_object(cx, object_ref, lower_key)?;
                set_property_on_object(cx, object_ref, upper_key, lower_value)?;
            }
            (None, None) => {}
        }

        lower += 1;
    }
    Ok(Value::from_object_ref(object_ref))
}

fn array_slice_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    let start = normalize_relative_index_u64(
        length,
        to_integer_or_infinity_for_builtin(
            cx,
            invocation
                .arguments()
                .first()
                .copied()
                .unwrap_or(Value::undefined()),
        )?,
    );
    let end = normalize_relative_index_u64(
        length,
        match invocation.arguments().get(1).copied() {
            Some(value) if value.is_undefined() => length as f64,
            Some(value) => to_integer_or_infinity_for_builtin(cx, value)?,
            None => length as f64,
        },
    );
    let count = end.saturating_sub(start);
    if count > u64::from(u32::MAX) {
        return Err(range_error(cx));
    }
    let result = array_species_create_for_length(cx, object_ref, count)?;
    for offset in 0..count {
        let source_key = array_like_index_property_key(cx, start.saturating_add(offset));
        if !has_property_on_object(cx, object_ref, source_key)? {
            continue;
        }
        let value = get_property_from_object(cx, object_ref, source_key)?;
        let target_index = u32::try_from(offset).expect("slice result length already validated");
        create_data_property_or_throw(cx, result, PropertyKey::Index(target_index), value)?;
    }
    define_array_length(
        cx,
        result,
        u32::try_from(count).expect("slice result length already validated"),
    )?;
    Ok(Value::from_object_ref(result))
}

pub(super) fn compare_array_sort_values<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    compare_fn: Option<lyng_js_types::ObjectRef>,
    left: Value,
    right: Value,
) -> Result<std::cmp::Ordering, Cx::Error> {
    if left.is_undefined() && right.is_undefined() {
        return Ok(std::cmp::Ordering::Equal);
    }
    if left.is_undefined() {
        return Ok(std::cmp::Ordering::Greater);
    }
    if right.is_undefined() {
        return Ok(std::cmp::Ordering::Less);
    }
    if let Some(compare_fn) = compare_fn {
        let compared = cx.call_to_completion(compare_fn, Value::undefined(), &[left, right])?;
        let number = to_number_for_builtin(cx, compared)?;
        return Ok(if number.is_nan() || number == 0.0 {
            std::cmp::Ordering::Equal
        } else if number < 0.0 {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        });
    }
    let left_text = cx.value_to_string_text(left)?;
    let right_text = cx.value_to_string_text(right)?;
    Ok(left_text.cmp(&right_text))
}

fn array_sort_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let compare_fn = match invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined())
    {
        value if value.is_undefined() => None,
        value => Some(cx.require_callable_object(value)?),
    };
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length(cx, object_ref)?;
    let mut items = Vec::with_capacity(usize::try_from(length).unwrap_or(usize::MAX));
    let mut undefined_count = 0_u32;
    for index in 0..length {
        let key = PropertyKey::Index(index);
        if !has_property_on_object(cx, object_ref, key)? {
            continue;
        }
        let value = get_property_from_object(cx, object_ref, key)?;
        if value.is_undefined() {
            undefined_count = undefined_count.saturating_add(1);
        } else {
            items.push(value);
        }
    }

    for i in 1..items.len() {
        let mut j = i;
        while j > 0
            && compare_array_sort_values(cx, compare_fn, items[j - 1], items[j])?
                == std::cmp::Ordering::Greater
        {
            items.swap(j - 1, j);
            j -= 1;
        }
    }

    let mut index = 0_u32;
    for value in items {
        set_property_on_object(cx, object_ref, PropertyKey::Index(index), value)?;
        index = index.saturating_add(1);
    }
    for _ in 0..undefined_count {
        set_property_on_object(
            cx,
            object_ref,
            PropertyKey::Index(index),
            Value::undefined(),
        )?;
        index = index.saturating_add(1);
    }
    while index < length {
        delete_property_from_object(cx, object_ref, PropertyKey::Index(index))?;
        index = index.saturating_add(1);
    }
    Ok(Value::from_object_ref(object_ref))
}

fn array_splice_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let arguments = invocation.arguments();
    let length = array_like_length_u64(cx, object_ref)?;
    let start = normalize_relative_index_u64(
        length,
        to_integer_or_infinity_for_builtin(
            cx,
            arguments.first().copied().unwrap_or(Value::undefined()),
        )?,
    );
    let insert_count = u64::try_from(arguments.len().saturating_sub(2)).unwrap_or(u64::MAX);
    let delete_count = if arguments.is_empty() {
        0
    } else if arguments.len() == 1 {
        length.saturating_sub(start)
    } else {
        let requested = to_integer_or_infinity_for_builtin(
            cx,
            arguments.get(1).copied().unwrap_or(Value::undefined()),
        )?;
        if requested <= 0.0 {
            0
        } else {
            requested.min(length.saturating_sub(start) as f64) as u64
        }
    };
    let items = if arguments.len() > 2 {
        &arguments[2..]
    } else {
        &[]
    };
    let Some(new_length) = length
        .checked_add(insert_count)
        .and_then(|value| value.checked_sub(delete_count))
    else {
        return Err(type_error(cx));
    };
    if new_length > MAX_SAFE_INTEGER_U64 {
        return Err(type_error(cx));
    }

    let removed = array_species_create_for_length(cx, object_ref, delete_count)?;
    for offset in 0..delete_count {
        let from_key = array_like_index_property_key(cx, start.saturating_add(offset));
        if !has_property_on_object(cx, object_ref, from_key)? {
            continue;
        }
        let value = get_property_from_object(cx, object_ref, from_key)?;
        let to_key = array_like_index_property_key(cx, offset);
        create_data_property_or_throw(cx, removed, to_key, value)?;
    }
    set_length_property(cx, removed, delete_count)?;

    if insert_count < delete_count {
        let mut index = start;
        let shift_limit = length - delete_count;
        while index < shift_limit {
            let from_key = array_like_index_property_key(cx, index + delete_count);
            let to_key = array_like_index_property_key(cx, index + insert_count);
            if has_property_on_object(cx, object_ref, from_key)? {
                let value = get_property_from_object(cx, object_ref, from_key)?;
                set_property_on_object(cx, object_ref, to_key, value)?;
            } else {
                delete_property_from_object(cx, object_ref, to_key)?;
            }
            index += 1;
        }

        let mut index = length;
        let delete_from = length - delete_count + insert_count;
        while index > delete_from {
            let key = array_like_index_property_key(cx, index - 1);
            delete_property_from_object(cx, object_ref, key)?;
            index -= 1;
        }
    } else if insert_count > delete_count {
        let mut index = length - delete_count;
        while index > start {
            let from_key = array_like_index_property_key(cx, index + delete_count - 1);
            let to_key = array_like_index_property_key(cx, index + insert_count - 1);
            if has_property_on_object(cx, object_ref, from_key)? {
                let value = get_property_from_object(cx, object_ref, from_key)?;
                set_property_on_object(cx, object_ref, to_key, value)?;
            } else {
                delete_property_from_object(cx, object_ref, to_key)?;
            }
            index -= 1;
        }
    }

    for (offset, value) in items.iter().copied().enumerate() {
        let key = array_like_index_property_key(
            cx,
            start.saturating_add(u64::try_from(offset).unwrap_or(u64::MAX)),
        );
        set_property_on_object(cx, object_ref, key, value)?;
    }
    set_length_property(cx, object_ref, new_length)?;
    Ok(Value::from_object_ref(removed))
}

fn array_to_reversed_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    let result = create_array_result_for_length(cx, length)?;
    for index in 0..length {
        let from_key = array_like_index_property_key(cx, length - index - 1);
        let value = get_property_from_object(cx, object_ref, from_key)?;
        let to_key = array_like_index_property_key(cx, index);
        create_data_property_or_throw(cx, result, to_key, value)?;
    }
    Ok(Value::from_object_ref(result))
}

fn array_to_sorted_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let compare_fn = match invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined())
    {
        value if value.is_undefined() => None,
        value => Some(cx.require_callable_object(value)?),
    };
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    let result = create_array_result_for_length(cx, length)?;
    let mut elements = Vec::with_capacity(array_result_capacity_hint(length));
    for index in 0..length {
        let key = array_like_index_property_key(cx, index);
        elements.push(get_property_from_object(cx, object_ref, key)?);
    }
    for i in 1..elements.len() {
        let mut j = i;
        while j > 0
            && compare_array_sort_values(cx, compare_fn, elements[j - 1], elements[j])?
                == std::cmp::Ordering::Greater
        {
            elements.swap(j - 1, j);
            j -= 1;
        }
    }
    for (index, value) in elements.into_iter().enumerate() {
        let key = array_like_index_property_key(cx, u64::try_from(index).unwrap_or(u64::MAX));
        create_data_property_or_throw(cx, result, key, value)?;
    }
    Ok(Value::from_object_ref(result))
}

fn array_to_spliced_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let arguments = invocation.arguments();
    let length = array_like_length_u64(cx, object_ref)?;
    let actual_start = normalize_relative_index_u64(
        length,
        to_integer_or_infinity_for_builtin(
            cx,
            arguments.first().copied().unwrap_or(Value::undefined()),
        )?,
    );
    let actual_delete_count = if arguments.is_empty() {
        0
    } else if arguments.len() == 1 {
        length.saturating_sub(actual_start)
    } else {
        let delete_count = to_integer_or_infinity_for_builtin(
            cx,
            arguments.get(1).copied().unwrap_or(Value::undefined()),
        )?;
        if delete_count <= 0.0 || delete_count.is_nan() {
            0
        } else {
            (delete_count as u64).min(length.saturating_sub(actual_start))
        }
    };
    let items = if arguments.len() > 2 {
        &arguments[2..]
    } else {
        &[]
    };
    let insert_count = u64::try_from(items.len()).unwrap_or(u64::MAX);
    let Some(new_length) = length
        .checked_add(insert_count)
        .and_then(|value| value.checked_sub(actual_delete_count))
    else {
        return Err(type_error(cx));
    };
    if new_length > MAX_SAFE_INTEGER_U64 {
        return Err(type_error(cx));
    }

    let result = create_array_result_for_length(cx, new_length)?;
    let mut to_index = 0_u64;
    for from_index in 0..actual_start {
        let from_key = array_like_index_property_key(cx, from_index);
        let value = get_property_from_object(cx, object_ref, from_key)?;
        let to_key = array_like_index_property_key(cx, to_index);
        create_data_property_or_throw(cx, result, to_key, value)?;
        to_index += 1;
    }
    for value in items.iter().copied() {
        let to_key = array_like_index_property_key(cx, to_index);
        create_data_property_or_throw(cx, result, to_key, value)?;
        to_index += 1;
    }
    let tail_start = actual_start.saturating_add(actual_delete_count);
    for from_index in tail_start..length {
        let from_key = array_like_index_property_key(cx, from_index);
        let value = get_property_from_object(cx, object_ref, from_key)?;
        let to_key = array_like_index_property_key(cx, to_index);
        create_data_property_or_throw(cx, result, to_key, value)?;
        to_index += 1;
    }
    Ok(Value::from_object_ref(result))
}

fn array_with_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    let relative_index = to_integer_or_infinity_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let actual_index = if relative_index < 0.0 {
        length as f64 + relative_index
    } else {
        relative_index
    };
    if !actual_index.is_finite() || actual_index < 0.0 || actual_index >= length as f64 {
        return Err(range_error(cx));
    }
    let actual_index = actual_index as u64;
    let replacement = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let result = create_array_result_for_length(cx, length)?;
    for index in 0..length {
        let value = if index == actual_index {
            replacement
        } else {
            let key = array_like_index_property_key(cx, index);
            get_property_from_object(cx, object_ref, key)?
        };
        let key = array_like_index_property_key(cx, index);
        create_data_property_or_throw(cx, result, key, value)?;
    }
    Ok(Value::from_object_ref(result))
}

fn array_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let join_key = property_key_from_text(cx, "join");
    let join_value = cx.get_property_value(invocation.this_value(), join_key)?;
    let join = if let Some(object) = join_value.as_object_ref() {
        let is_callable = {
            let agent = cx.agent();
            agent.objects().is_callable(object)
        };
        is_callable.then_some(object)
    } else {
        None
    };
    if let Some(join) = join {
        return cx.call_to_completion(join, invocation.this_value(), &[]);
    }
    objects::object_to_string_builtin(cx, invocation)
}

fn array_join_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length(cx, object_ref)?;
    let separator_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let separator = if separator_value.is_undefined() {
        ",".to_owned()
    } else {
        cx.value_to_string_text(separator_value)?
    };
    let text = array_like_join_text_for_length(cx, object_ref, length, &separator)?;
    Ok(string_value(cx, &text))
}

pub(super) fn array_pop_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    if length == 0 {
        set_length_property(cx, object_ref, 0)?;
        return Ok(Value::undefined());
    }

    let new_length = length - 1;
    let key = array_like_index_property_key(cx, new_length);
    let element = get_property_from_object(cx, object_ref, key)?;
    delete_property_from_object(cx, object_ref, key)?;
    set_length_property(cx, object_ref, new_length)?;
    Ok(element)
}

pub(super) fn array_push_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let mut length = array_like_length_u64(cx, object_ref)?;
    let item_count = u64::try_from(invocation.arguments().len()).unwrap_or(u64::MAX);
    if item_count > MAX_SAFE_INTEGER_U64.saturating_sub(length) {
        return Err(type_error(cx));
    }

    for argument in invocation.arguments() {
        let key = array_like_index_property_key(cx, length);
        set_property_on_object(cx, object_ref, key, *argument)?;
        length += 1;
    }
    set_length_property(cx, object_ref, length)?;
    Ok(length_value_u64(length))
}

fn array_shift_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    if length == 0 {
        set_property_on_object(
            cx,
            object_ref,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
            Value::from_smi(0),
        )?;
        return Ok(Value::undefined());
    }

    let first_key = array_like_index_property_key(cx, 0);
    let first = get_property_from_object(cx, object_ref, first_key)?;
    for index in 1..length {
        let from = array_like_index_property_key(cx, index);
        let to = array_like_index_property_key(cx, index - 1);
        if has_property_on_object(cx, object_ref, from)? {
            let value = get_property_from_object(cx, object_ref, from)?;
            set_property_on_object(cx, object_ref, to, value)?;
        } else {
            delete_property_from_object(cx, object_ref, to)?;
        }
    }

    let last = array_like_index_property_key(cx, length - 1);
    delete_property_from_object(cx, object_ref, last)?;
    let new_length = length - 1;
    let length_value = if new_length <= u64::from(i32::MAX as u32) {
        Value::from_smi(i32::try_from(new_length).unwrap_or(i32::MAX))
    } else {
        Value::from_f64(new_length as f64)
    };
    set_property_on_object(
        cx,
        object_ref,
        PropertyKey::from_atom(WellKnownAtom::length.id()),
        length_value,
    )?;
    Ok(first)
}

fn array_unshift_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length_u64(cx, object_ref)?;
    let item_count = u64::try_from(invocation.arguments().len()).unwrap_or(u64::MAX);
    let Some(new_length) = length.checked_add(item_count) else {
        return Err(type_error(cx));
    };
    if new_length > MAX_SAFE_INTEGER_U64 {
        return Err(type_error(cx));
    }

    if item_count > 0 {
        let mut index = length;
        while index > 0 {
            let from_index = index - 1;
            let from = array_like_index_property_key(cx, from_index);
            let to = array_like_index_property_key(cx, from_index + item_count);
            if has_property_on_object(cx, object_ref, from)? {
                let value = get_property_from_object(cx, object_ref, from)?;
                set_property_on_object(cx, object_ref, to, value)?;
            } else {
                delete_property_from_object(cx, object_ref, to)?;
            }
            index -= 1;
        }

        for (index, value) in invocation.arguments().iter().copied().enumerate() {
            let key = array_like_index_property_key(cx, u64::try_from(index).unwrap_or(u64::MAX));
            set_property_on_object(cx, object_ref, key, value)?;
        }
    }

    set_length_property(cx, object_ref, new_length)?;
    Ok(length_value_u64(new_length))
}

fn array_to_locale_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let length = array_like_length(cx, object_ref)?;
    let to_locale_string_key = property_key_from_text(cx, "toLocaleString");
    let mut parts = Vec::with_capacity(usize::try_from(length).unwrap_or(usize::MAX));
    for index in 0..length {
        let key = PropertyKey::Index(index);
        let text = if !has_property_on_object(cx, object_ref, key)? {
            String::new()
        } else {
            let value = get_property_from_object(cx, object_ref, key)?;
            if value.is_undefined() || value.is_null() {
                String::new()
            } else {
                let method_value = cx.get_property_value(value, to_locale_string_key)?;
                let method = cx.require_callable_object(method_value)?;
                let result = cx.call_to_completion(method, value, &[])?;
                cx.value_to_string_text(result)?
            }
        };
        parts.push(text);
    }
    Ok(string_value(cx, &parts.join(",")))
}

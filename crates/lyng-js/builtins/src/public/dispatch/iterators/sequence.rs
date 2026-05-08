use super::{
    allocate_iterator_object, create_data_property_or_throw, create_iterator_result_value,
    get_iterator_flattenable, iterator, iterator_helper_active_record, iterator_helper_counter,
    iterator_helper_sequence_count, iterator_slot_value_for_builtin,
    number_to_u32_after_range_check, numbers_are_equal, property_key_from_text,
    proxy_get_own_property, proxy_get_prototype_of, proxy_own_property_keys,
    set_iterator_helper_done, set_iterator_slot_value_for_builtin, string_ref_text, type_error,
    u64_to_value, AbruptCompletion, AtomId, BuiltinInvocation, BuiltinIteratorBridge,
    IteratorHelperKind, IteratorZipCollectedRecord, IteratorZipKey, IteratorZipMode, ObjectRef,
    OrdinaryObjectData, PropertyKey, PublicBuiltinDispatchContext, Value, WellKnownSymbolId,
    ITERATOR_HELPER_COUNTER_SLOT, ITERATOR_HELPER_ITERATED_SLOT, ITERATOR_HELPER_NEXT_METHOD_SLOT,
    ITERATOR_HELPER_SEQUENCE_BASE_SLOT, ITERATOR_ZIP_ALIVE_OFFSET, ITERATOR_ZIP_ITERATED_OFFSET,
    ITERATOR_ZIP_KEY_KIND_OFFSET, ITERATOR_ZIP_KEY_PAYLOAD_OFFSET, ITERATOR_ZIP_NEXT_METHOD_OFFSET,
    ITERATOR_ZIP_PADDING_OFFSET, ITERATOR_ZIP_RECORD_WIDTH,
};

pub(super) fn iterator_concat_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let arguments = invocation.arguments();
    let iterator_symbol = cx
        .agent()
        .well_known_symbol(WellKnownSymbolId::Iterator)
        .ok_or_else(|| type_error(cx))?;
    let count = i32::try_from(arguments.len()).map_err(|_| type_error(cx))?;
    let mut slot_values = Vec::with_capacity(ITERATOR_HELPER_SEQUENCE_BASE_SLOT as usize + 2);
    slot_values.extend_from_slice(&[
        Value::undefined(),
        Value::undefined(),
        Value::from_bool(false),
        Value::from_bool(false),
        IteratorHelperKind::Concat.into_value(),
        Value::from_smi(count),
        Value::from_smi(0),
    ]);
    for item in arguments.iter().copied() {
        let iterable = item.as_object_ref().ok_or_else(|| type_error(cx))?;
        let method = cx.get_property_value(
            Value::from_object_ref(iterable),
            PropertyKey::from_symbol(iterator_symbol),
        )?;
        if method.is_undefined() || method.is_null() {
            return Err(type_error(cx));
        }
        let method = cx.require_callable_object(method)?;
        slot_values.push(Value::from_object_ref(iterable));
        slot_values.push(Value::from_object_ref(method));
    }
    let realm = cx.builtin_realm();
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator_helper_prototype())
        .ok_or_else(|| type_error(cx))?;
    let helper = allocate_iterator_object(
        cx,
        prototype,
        OrdinaryObjectData::IteratorHelper,
        slot_values.as_slice(),
    )?;
    Ok(Value::from_object_ref(helper))
}

pub(super) fn iterator_zip_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let arguments = invocation.arguments();
    let iterables = arguments.first().copied().unwrap_or(Value::undefined());
    iterables.as_object_ref().ok_or_else(|| type_error(cx))?;
    let (mode, padding) = iterator_zip_options(cx, arguments)?;
    let mut input_record = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::get_iterator(&mut bridge, iterables)?
    };
    let mut records = Vec::new();
    loop {
        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut input_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => return iterator_zip_close_collected_after_error(cx, &records, error),
        };
        let Some(next) = next else {
            break;
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => return iterator_zip_close_collected_after_error(cx, &records, error),
        };
        let (iterator, next_method) = match get_iterator_flattenable(cx, value) {
            Ok(record) => record,
            Err(error) => {
                return iterator_zip_close_collected_and_input_after_error(
                    cx,
                    &records,
                    &mut input_record,
                    error,
                );
            }
        };
        records.push(IteratorZipCollectedRecord {
            key: IteratorZipKey::None,
            iterator,
            next_method,
            padding: Value::undefined(),
        });
    }
    iterator_zip_finish(cx, records, mode, padding, IteratorHelperKind::Zip)
}

pub(super) fn iterator_zip_keyed_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let arguments = invocation.arguments();
    let iterables = arguments.first().copied().unwrap_or(Value::undefined());
    let iterables = iterables.as_object_ref().ok_or_else(|| type_error(cx))?;
    let (mode, padding) = iterator_zip_options(cx, arguments)?;
    let keys = proxy_own_property_keys(cx, iterables)?;
    let mut records = Vec::new();
    for key in keys {
        let descriptor = match proxy_get_own_property(cx, iterables, key) {
            Ok(descriptor) => descriptor,
            Err(error) => return iterator_zip_close_collected_after_error(cx, &records, error),
        };
        let Some(descriptor) = descriptor else {
            continue;
        };
        if descriptor.enumerable() != Some(true) {
            continue;
        }
        let value = match cx.get_property_value(Value::from_object_ref(iterables), key) {
            Ok(value) => value,
            Err(error) => return iterator_zip_close_collected_after_error(cx, &records, error),
        };
        if value.is_undefined() {
            continue;
        }
        let (iterator, next_method) = match get_iterator_flattenable(cx, value) {
            Ok(record) => record,
            Err(error) => return iterator_zip_close_collected_after_error(cx, &records, error),
        };
        records.push(IteratorZipCollectedRecord {
            key: iterator_zip_key_from_property_key(key),
            iterator,
            next_method,
            padding: Value::undefined(),
        });
    }
    iterator_zip_finish(cx, records, mode, padding, IteratorHelperKind::ZipKeyed)
}

fn iterator_zip_finish<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    mut records: Vec<IteratorZipCollectedRecord>,
    mode: IteratorZipMode,
    padding: Option<ObjectRef>,
    kind: IteratorHelperKind,
) -> Result<Value, Cx::Error> {
    if mode == IteratorZipMode::Longest {
        let padding_values = if kind == IteratorHelperKind::ZipKeyed {
            iterator_zip_keyed_padding_values(cx, padding, &records)?
        } else {
            iterator_zip_padding_values(cx, padding, records.len(), &records)?
        };
        for (record, padding_value) in records.iter_mut().zip(padding_values) {
            record.padding = padding_value;
        }
    }
    let count = i32::try_from(records.len()).map_err(|_| type_error(cx))?;
    let mut slot_values = Vec::with_capacity(
        ITERATOR_HELPER_SEQUENCE_BASE_SLOT as usize
            + records.len() * ITERATOR_ZIP_RECORD_WIDTH as usize,
    );
    slot_values.extend_from_slice(&[
        Value::from_bool(false),
        Value::undefined(),
        Value::from_bool(false),
        Value::from_bool(false),
        kind.into_value(),
        Value::from_smi(count),
        mode.into_value(),
    ]);
    for record in records {
        let (key_kind, key_payload) = iterator_zip_key_to_slot_values(record.key);
        slot_values.push(key_kind);
        slot_values.push(key_payload);
        slot_values.push(Value::from_object_ref(record.iterator));
        slot_values.push(record.next_method);
        slot_values.push(Value::from_bool(true));
        slot_values.push(record.padding);
    }
    let realm = cx.builtin_realm();
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator_helper_prototype())
        .ok_or_else(|| type_error(cx))?;
    let helper = allocate_iterator_object(
        cx,
        prototype,
        OrdinaryObjectData::IteratorHelper,
        slot_values.as_slice(),
    )?;
    Ok(Value::from_object_ref(helper))
}

fn iterator_zip_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    arguments: &[Value],
) -> Result<(IteratorZipMode, Option<ObjectRef>), Cx::Error> {
    let options = arguments.get(1).copied().unwrap_or(Value::undefined());
    let Some(options) = options.as_object_ref() else {
        if options.is_undefined() {
            return Ok((IteratorZipMode::Shortest, None));
        }
        return Err(type_error(cx));
    };
    let mode_key = property_key_from_text(cx, "mode");
    let mode_value = cx.get_property_value(Value::from_object_ref(options), mode_key)?;
    let mode = if mode_value.is_undefined() {
        IteratorZipMode::Shortest
    } else if let Some(string) = mode_value.as_string_ref() {
        match string_ref_text(cx, string)?.as_str() {
            "shortest" => IteratorZipMode::Shortest,
            "longest" => IteratorZipMode::Longest,
            "strict" => IteratorZipMode::Strict,
            _ => return Err(type_error(cx)),
        }
    } else {
        return Err(type_error(cx));
    };
    let padding = if mode == IteratorZipMode::Longest {
        let padding_key = property_key_from_text(cx, "padding");
        let padding = cx.get_property_value(Value::from_object_ref(options), padding_key)?;
        if padding.is_undefined() {
            None
        } else {
            Some(padding.as_object_ref().ok_or_else(|| type_error(cx))?)
        }
    } else {
        None
    };
    Ok((mode, padding))
}
pub(super) fn iterator_helper_concat_next<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    loop {
        if let Some(mut iterator_record) = iterator_helper_active_record(cx, helper)? {
            let next = {
                let mut bridge = BuiltinIteratorBridge { cx };
                iterator::iterator_step(&mut bridge, &mut iterator_record)
            };
            let next = match next {
                Ok(next) => next,
                Err(error) => {
                    set_iterator_helper_done(cx, helper)?;
                    return Err(error);
                }
            };
            let Some(next) = next else {
                clear_iterator_helper_current(cx, helper)?;
                continue;
            };
            let value = {
                let mut bridge = BuiltinIteratorBridge { cx };
                iterator::iterator_value(&mut bridge, next)
            };
            let value = match value {
                Ok(value) => value,
                Err(error) => {
                    set_iterator_helper_done(cx, helper)?;
                    return Err(error);
                }
            };
            return create_iterator_result_value(cx, value, false);
        }

        let index = iterator_helper_counter(cx, helper)?;
        let count = iterator_helper_sequence_count(cx, helper)?;
        if index >= count {
            set_iterator_helper_done(cx, helper)?;
            return create_iterator_result_value(cx, Value::undefined(), true);
        }
        if let Err(error) = iterator_helper_concat_open_current(cx, helper, index) {
            set_iterator_helper_done(cx, helper)?;
            return Err(error);
        }
        set_iterator_slot_value_for_builtin(
            cx,
            helper,
            OrdinaryObjectData::IteratorHelper,
            ITERATOR_HELPER_COUNTER_SLOT,
            u64_to_value(index.saturating_add(1)),
        )?;
    }
}

pub(super) fn iterator_helper_concat_return<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    if let Some(mut iterator_record) = iterator_helper_active_record(cx, helper)? {
        clear_iterator_helper_current(cx, helper)?;
        let close_result = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_close(
                &mut bridge,
                &mut iterator_record,
                Ok::<(), lyng_js_types::AbruptCompletion>(()),
            )
        };
        close_result?;
    }
    create_iterator_result_value(cx, Value::undefined(), true)
}

pub(super) fn iterator_helper_zip_next<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    kind: IteratorHelperKind,
) -> Result<Value, Cx::Error> {
    let count = iterator_helper_sequence_count(cx, helper)?;
    if count == 0 {
        set_iterator_helper_done(cx, helper)?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    }
    let mode = iterator_helper_zip_mode(cx, helper)?;
    let mut values = Vec::with_capacity(usize::try_from(count).map_err(|_| type_error(cx))?);
    let mut any_done = false;
    let mut any_value = false;

    for index in 0..count {
        if !iterator_helper_zip_alive(cx, helper, index)? {
            values.push(iterator_helper_zip_padding(cx, helper, index)?);
            continue;
        }
        let mut iterator_record = match iterator_helper_zip_step_record(cx, helper, index) {
            Ok(record) => record,
            Err(error) => {
                set_iterator_helper_done(cx, helper)?;
                iterator_helper_zip_mark_dead(cx, helper, index)?;
                return iterator_helper_zip_close_all_after_error(cx, helper, error);
            }
        };
        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut iterator_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => {
                set_iterator_helper_done(cx, helper)?;
                iterator_helper_zip_mark_dead(cx, helper, index)?;
                return iterator_helper_zip_close_all_after_error(cx, helper, error);
            }
        };
        let Some(next) = next else {
            iterator_helper_zip_mark_dead(cx, helper, index)?;
            match mode {
                IteratorZipMode::Shortest => {
                    set_iterator_helper_done(cx, helper)?;
                    iterator_helper_zip_close_all(cx, helper, None)?;
                    return create_iterator_result_value(cx, Value::undefined(), true);
                }
                IteratorZipMode::Longest => {
                    any_done = true;
                    values.push(iterator_helper_zip_padding(cx, helper, index)?);
                    continue;
                }
                IteratorZipMode::Strict => {
                    any_done = true;
                    if any_value {
                        set_iterator_helper_done(cx, helper)?;
                        let error = type_error(cx);
                        return iterator_helper_zip_close_all_after_error(cx, helper, error);
                    }
                    values.push(Value::undefined());
                    continue;
                }
            }
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => {
                set_iterator_helper_done(cx, helper)?;
                iterator_helper_zip_mark_dead(cx, helper, index)?;
                return iterator_helper_zip_close_all_after_error(cx, helper, error);
            }
        };
        if mode == IteratorZipMode::Strict && any_done {
            set_iterator_helper_done(cx, helper)?;
            let error = type_error(cx);
            return iterator_helper_zip_close_all_after_error(cx, helper, error);
        }
        any_value = true;
        values.push(value);
    }

    if mode == IteratorZipMode::Longest && !any_value {
        set_iterator_helper_done(cx, helper)?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    }
    if mode == IteratorZipMode::Strict && any_done {
        set_iterator_helper_done(cx, helper)?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    }

    let value = if kind == IteratorHelperKind::Zip {
        Value::from_object_ref(super::create_array_from_values(cx, &values)?)
    } else {
        Value::from_object_ref(iterator_helper_zip_keyed_result(cx, helper, &values)?)
    };
    iterator_helper_zip_set_started(cx, helper)?;
    create_iterator_result_value(cx, value, false)
}

pub(super) fn iterator_helper_zip_return<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    iterator_helper_zip_close_all(cx, helper, None)?;
    create_iterator_result_value(cx, Value::undefined(), true)
}

fn iterator_zip_padding_values<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    padding: Option<ObjectRef>,
    count: usize,
    records: &[IteratorZipCollectedRecord],
) -> Result<Vec<Value>, Cx::Error> {
    let Some(padding) = padding else {
        return Ok(vec![Value::undefined(); count]);
    };
    let mut padding_record = {
        let mut bridge = BuiltinIteratorBridge { cx };
        match iterator::get_iterator(&mut bridge, Value::from_object_ref(padding)) {
            Ok(record) => record,
            Err(error) => return iterator_zip_close_collected_after_error(cx, records, error),
        }
    };
    let mut using_iterator = true;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        if !using_iterator {
            values.push(Value::undefined());
            continue;
        }
        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut padding_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => return iterator_zip_close_collected_after_error(cx, records, error),
        };
        let Some(next) = next else {
            using_iterator = false;
            values.push(Value::undefined());
            continue;
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        match value {
            Ok(value) => values.push(value),
            Err(error) => return iterator_zip_close_collected_after_error(cx, records, error),
        }
    }
    if using_iterator {
        let thrown = iterator_zip_close_iterator_record(cx, &mut padding_record, None)?;
        if let Some(thrown) = thrown {
            let thrown = iterator_zip_close_collected_with_thrown(cx, records, Some(thrown))?
                .expect("throw completion should be preserved");
            return Err(cx.abrupt(AbruptCompletion::throw(thrown)));
        }
    }
    Ok(values)
}

fn iterator_zip_keyed_padding_values<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    padding: Option<ObjectRef>,
    records: &[IteratorZipCollectedRecord],
) -> Result<Vec<Value>, Cx::Error> {
    let Some(padding) = padding else {
        return Ok(vec![Value::undefined(); records.len()]);
    };
    let mut values = Vec::with_capacity(records.len());
    for record in records {
        let key = iterator_zip_collected_key_to_property_key(cx, record.key)?;
        let value = match cx.get_property_value(Value::from_object_ref(padding), key) {
            Ok(value) => value,
            Err(error) => return iterator_zip_close_collected_after_error(cx, records, error),
        };
        values.push(value);
    }
    Ok(values)
}

fn iterator_zip_close_collected_after_error<Cx: PublicBuiltinDispatchContext, T>(
    cx: &mut Cx,
    records: &[IteratorZipCollectedRecord],
    error: Cx::Error,
) -> Result<T, Cx::Error> {
    let thrown = iterator_zip_thrown_value(cx, error)?;
    let thrown = iterator_zip_close_collected_with_thrown(cx, records, Some(thrown))?
        .expect("throw completion should be preserved");
    Err(cx.abrupt(AbruptCompletion::throw(thrown)))
}

fn iterator_zip_close_collected_and_input_after_error<Cx: PublicBuiltinDispatchContext, T>(
    cx: &mut Cx,
    records: &[IteratorZipCollectedRecord],
    input_record: &mut iterator::IteratorRecord,
    error: Cx::Error,
) -> Result<T, Cx::Error> {
    let thrown = iterator_zip_thrown_value(cx, error)?;
    let thrown = iterator_zip_close_collected_with_thrown(cx, records, Some(thrown))?
        .expect("throw completion should be preserved");
    let thrown = iterator_zip_close_iterator_record(cx, input_record, Some(thrown))?
        .expect("throw completion should be preserved");
    Err(cx.abrupt(AbruptCompletion::throw(thrown)))
}

fn iterator_zip_close_collected_with_thrown<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    records: &[IteratorZipCollectedRecord],
    mut thrown: Option<Value>,
) -> Result<Option<Value>, Cx::Error> {
    for record in records.iter().rev() {
        thrown =
            iterator_zip_close_iterator_value(cx, record.iterator, record.next_method, thrown)?;
    }
    Ok(thrown)
}

fn iterator_helper_zip_close_all_after_error<Cx: PublicBuiltinDispatchContext, T>(
    cx: &mut Cx,
    helper: ObjectRef,
    error: Cx::Error,
) -> Result<T, Cx::Error> {
    let thrown = iterator_zip_thrown_value(cx, error)?;
    let thrown = iterator_helper_zip_close_all_with_thrown(cx, helper, Some(thrown))?
        .expect("throw completion should be preserved");
    Err(cx.abrupt(AbruptCompletion::throw(thrown)))
}

fn iterator_helper_zip_close_all<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    thrown: Option<Value>,
) -> Result<(), Cx::Error> {
    if let Some(thrown) = iterator_helper_zip_close_all_with_thrown(cx, helper, thrown)? {
        return Err(cx.abrupt(AbruptCompletion::throw(thrown)));
    }
    Ok(())
}

fn iterator_helper_zip_close_all_with_thrown<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    mut thrown: Option<Value>,
) -> Result<Option<Value>, Cx::Error> {
    let count = iterator_helper_sequence_count(cx, helper)?;
    for index in (0..count).rev() {
        if !iterator_helper_zip_alive(cx, helper, index)? {
            continue;
        }
        let iterator = iterator_helper_zip_iterator(cx, helper, index)?;
        let next_method = iterator_helper_zip_next_method(cx, helper, index)?;
        iterator_helper_zip_mark_dead(cx, helper, index)?;
        thrown = iterator_zip_close_iterator_value(cx, iterator, next_method, thrown)?;
    }
    Ok(thrown)
}

fn iterator_zip_close_iterator_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    iterator: ObjectRef,
    next_method: Value,
    thrown: Option<Value>,
) -> Result<Option<Value>, Cx::Error> {
    let next_method = next_method
        .as_object_ref()
        .unwrap_or_else(|| cx.callee_object());
    let mut record = iterator::IteratorRecord::new(iterator, next_method);
    iterator_zip_close_iterator_record(cx, &mut record, thrown)
}

fn iterator_zip_close_iterator_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    record: &mut iterator::IteratorRecord,
    thrown: Option<Value>,
) -> Result<Option<Value>, Cx::Error> {
    let completion = thrown.map_or(Ok(()), |thrown| Err(AbruptCompletion::throw(thrown)));
    let close = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_close(&mut bridge, record, completion)
    };
    match close {
        Ok(()) => Ok(None),
        Err(error) => iterator_zip_thrown_value(cx, error).map(Some),
    }
}

fn iterator_zip_thrown_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    error: Cx::Error,
) -> Result<Value, Cx::Error> {
    let Some(thrown) = cx.extract_thrown_value(error)? else {
        unreachable!("non-abrupt builtin error should propagate");
    };
    Ok(thrown)
}

fn iterator_helper_zip_mode<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<IteratorZipMode, Cx::Error> {
    IteratorZipMode::from_value(iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_COUNTER_SLOT,
    )?)
    .ok_or_else(|| type_error(cx))
}

pub(super) fn iterator_helper_zip_started<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<bool, Cx::Error> {
    iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_ITERATED_SLOT,
    )?
    .as_bool()
    .ok_or_else(|| type_error(cx))
}

fn iterator_helper_zip_set_started<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<(), Cx::Error> {
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_ITERATED_SLOT,
        Value::from_bool(true),
    )
}

fn iterator_helper_zip_record_slot<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    index: u64,
    offset: u32,
) -> Result<u32, Cx::Error> {
    ITERATOR_HELPER_SEQUENCE_BASE_SLOT
        .checked_add(
            u32::try_from(index.saturating_mul(u64::from(ITERATOR_ZIP_RECORD_WIDTH)))
                .map_err(|_| type_error(cx))?,
        )
        .and_then(|base| base.checked_add(offset))
        .ok_or_else(|| type_error(cx))
}

fn iterator_helper_zip_slot_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
    offset: u32,
) -> Result<Value, Cx::Error> {
    let slot = iterator_helper_zip_record_slot(cx, index, offset)?;
    iterator_slot_value_for_builtin(cx, helper, OrdinaryObjectData::IteratorHelper, slot)
}

fn iterator_helper_zip_set_slot_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
    offset: u32,
    value: Value,
) -> Result<(), Cx::Error> {
    let slot = iterator_helper_zip_record_slot(cx, index, offset)?;
    set_iterator_slot_value_for_builtin(cx, helper, OrdinaryObjectData::IteratorHelper, slot, value)
}

fn iterator_helper_zip_alive<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
) -> Result<bool, Cx::Error> {
    iterator_helper_zip_slot_value(cx, helper, index, ITERATOR_ZIP_ALIVE_OFFSET)?
        .as_bool()
        .ok_or_else(|| type_error(cx))
}

fn iterator_helper_zip_mark_dead<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
) -> Result<(), Cx::Error> {
    iterator_helper_zip_set_slot_value(
        cx,
        helper,
        index,
        ITERATOR_ZIP_ALIVE_OFFSET,
        Value::from_bool(false),
    )
}

fn iterator_helper_zip_iterator<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
) -> Result<ObjectRef, Cx::Error> {
    iterator_helper_zip_slot_value(cx, helper, index, ITERATOR_ZIP_ITERATED_OFFSET)?
        .as_object_ref()
        .ok_or_else(|| type_error(cx))
}

pub(super) fn iterator_helper_zip_next_method<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
) -> Result<Value, Cx::Error> {
    iterator_helper_zip_slot_value(cx, helper, index, ITERATOR_ZIP_NEXT_METHOD_OFFSET)
}

fn iterator_helper_zip_step_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
) -> Result<iterator::IteratorRecord, Cx::Error> {
    let iterator = iterator_helper_zip_iterator(cx, helper, index)?;
    let next_method = iterator_helper_zip_next_method(cx, helper, index)?;
    let next_method = cx.require_callable_object(next_method)?;
    Ok(iterator::IteratorRecord::new(iterator, next_method))
}

fn iterator_helper_zip_padding<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
) -> Result<Value, Cx::Error> {
    iterator_helper_zip_slot_value(cx, helper, index, ITERATOR_ZIP_PADDING_OFFSET)
}

fn iterator_helper_zip_keyed_result<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    values: &[Value],
) -> Result<ObjectRef, Cx::Error> {
    let object = cx.allocate_ordinary_object_with_prototype(cx.builtin_realm(), None)?;
    for (index, value) in values.iter().copied().enumerate() {
        let key = iterator_helper_zip_key(cx, helper, index as u64)?;
        create_data_property_or_throw(cx, object, key, value)?;
    }
    Ok(object)
}

fn iterator_helper_zip_key<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
) -> Result<PropertyKey, Cx::Error> {
    let kind = iterator_helper_zip_slot_value(cx, helper, index, ITERATOR_ZIP_KEY_KIND_OFFSET)?;
    let payload =
        iterator_helper_zip_slot_value(cx, helper, index, ITERATOR_ZIP_KEY_PAYLOAD_OFFSET)?;
    iterator_zip_key_from_slot_values(cx, kind, payload)
}

const fn iterator_zip_key_from_property_key(key: PropertyKey) -> IteratorZipKey {
    match key {
        PropertyKey::Index(index) => IteratorZipKey::Index(index),
        PropertyKey::Atom(atom) => IteratorZipKey::Atom(atom),
        PropertyKey::Symbol(symbol) => IteratorZipKey::Symbol(symbol),
    }
}

fn iterator_zip_collected_key_to_property_key<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    key: IteratorZipKey,
) -> Result<PropertyKey, Cx::Error> {
    match key {
        IteratorZipKey::None => Err(type_error(cx)),
        IteratorZipKey::Index(index) => Ok(PropertyKey::Index(index)),
        IteratorZipKey::Atom(atom) => Ok(PropertyKey::Atom(atom)),
        IteratorZipKey::Symbol(symbol) => Ok(PropertyKey::Symbol(symbol)),
    }
}

fn iterator_zip_key_to_slot_values(key: IteratorZipKey) -> (Value, Value) {
    match key {
        IteratorZipKey::None => (Value::from_smi(0), Value::undefined()),
        IteratorZipKey::Index(index) => (Value::from_smi(1), u64_to_value(u64::from(index))),
        IteratorZipKey::Atom(atom) => (Value::from_smi(2), u64_to_value(u64::from(atom.raw()))),
        IteratorZipKey::Symbol(symbol) => (Value::from_smi(3), Value::from_symbol_ref(symbol)),
    }
}

fn iterator_zip_key_from_slot_values<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: Value,
    payload: Value,
) -> Result<PropertyKey, Cx::Error> {
    match kind.as_smi() {
        Some(1) => Ok(PropertyKey::Index(iterator_zip_u32_payload(cx, payload)?)),
        Some(2) => Ok(PropertyKey::Atom(AtomId::from_raw(
            iterator_zip_u32_payload(cx, payload)?,
        ))),
        Some(3) => payload
            .as_symbol_ref()
            .map(PropertyKey::from_symbol)
            .ok_or_else(|| type_error(cx)),
        _ => Err(type_error(cx)),
    }
}

fn iterator_zip_u32_payload<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<u32, Cx::Error> {
    if let Some(value) = value.as_smi() {
        return u32::try_from(value).map_err(|_| type_error(cx));
    }
    let number = value.as_f64().ok_or_else(|| type_error(cx))?;
    if !number.is_finite()
        || number < 0.0
        || !numbers_are_equal(number, number.trunc())
        || number > f64::from(u32::MAX)
    {
        return Err(type_error(cx));
    }
    Ok(number_to_u32_after_range_check(number))
}

fn iterator_helper_concat_open_current<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
) -> Result<(), Cx::Error> {
    let slot_base = ITERATOR_HELPER_SEQUENCE_BASE_SLOT
        .checked_add(u32::try_from(index.saturating_mul(2)).map_err(|_| type_error(cx))?)
        .ok_or_else(|| type_error(cx))?;
    let iterable =
        iterator_slot_value_for_builtin(cx, helper, OrdinaryObjectData::IteratorHelper, slot_base)?
            .as_object_ref()
            .ok_or_else(|| type_error(cx))?;
    let method = iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        slot_base.saturating_add(1),
    )?
    .as_object_ref()
    .ok_or_else(|| type_error(cx))?;
    let iterator = cx.call_to_completion(method, Value::from_object_ref(iterable), &[])?;
    let iterator = iterator.as_object_ref().ok_or_else(|| type_error(cx))?;
    let next_key = property_key_from_text(cx, "next");
    let next = cx.get_property_value(Value::from_object_ref(iterator), next_key)?;
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_ITERATED_SLOT,
        Value::from_object_ref(iterator),
    )?;
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_NEXT_METHOD_SLOT,
        next,
    )
}

fn clear_iterator_helper_current<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<(), Cx::Error> {
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_ITERATED_SLOT,
        Value::undefined(),
    )?;
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_NEXT_METHOD_SLOT,
        Value::undefined(),
    )
}

pub(super) fn iterator_from_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let argument = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let realm = cx.builtin_realm();
    let iterator_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator_prototype())
        .ok_or_else(|| type_error(cx))?;
    let (iterator, next_method) = get_iterator_flattenable_for_iterator_from(cx, argument)?;
    if iterator_prototype_in_chain(cx, iterator, iterator_prototype)? {
        return Ok(Value::from_object_ref(iterator));
    }
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator_helper_prototype())
        .ok_or_else(|| type_error(cx))?;
    let slot_values = [
        Value::from_object_ref(iterator),
        next_method,
        Value::from_bool(false),
        Value::from_bool(false),
        IteratorHelperKind::Wrap.into_value(),
        Value::undefined(),
        Value::from_smi(0),
    ];
    let wrapper = allocate_iterator_object(
        cx,
        prototype,
        OrdinaryObjectData::IteratorHelper,
        &slot_values,
    )?;
    Ok(Value::from_object_ref(wrapper))
}

fn get_iterator_flattenable_for_iterator_from<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<(ObjectRef, Value), Cx::Error> {
    if value.as_object_ref().is_none() && !value.is_string() {
        return Err(type_error(cx));
    }
    let iterator_symbol = cx
        .agent()
        .well_known_symbol(WellKnownSymbolId::Iterator)
        .ok_or_else(|| type_error(cx))?;
    let method = cx.get_property_value(value, PropertyKey::from_symbol(iterator_symbol))?;
    let iterator = if method.is_undefined() || method.is_null() {
        value.as_object_ref().ok_or_else(|| type_error(cx))?
    } else {
        let method = cx.require_callable_object(method)?;
        let iterator = cx.call_to_completion(method, value, &[])?;
        iterator.as_object_ref().ok_or_else(|| type_error(cx))?
    };
    let next_key = property_key_from_text(cx, "next");
    let next = cx.get_property_value(Value::from_object_ref(iterator), next_key)?;
    Ok((iterator, next))
}

fn iterator_prototype_in_chain<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    start: lyng_js_types::ObjectRef,
    target_prototype: lyng_js_types::ObjectRef,
) -> Result<bool, Cx::Error> {
    let mut current = Some(start);
    let mut steps = 0_u32;
    while let Some(object) = current {
        if object == target_prototype {
            return Ok(true);
        }
        // Cap traversal to prevent runaway proxy traps from misbehaving;
        // the spec never requires more than a finite chain.
        steps = steps.saturating_add(1);
        if steps > 1024 {
            break;
        }
        current = proxy_get_prototype_of(cx, object)?;
    }
    Ok(false)
}

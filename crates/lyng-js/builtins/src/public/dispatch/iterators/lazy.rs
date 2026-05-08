use super::{
    allocate_iterator_object, clear_iterator_helper_inner, close_iterator_after_error,
    create_iterator_result_value, get_iterator_flattenable, iterator,
    iterator_close_for_validation_failure, iterator_helper_concat_next,
    iterator_helper_concat_return, iterator_helper_counter, iterator_helper_done,
    iterator_helper_inner_record, iterator_helper_iterated_object, iterator_helper_limit,
    iterator_helper_record, iterator_helper_running, iterator_helper_this_object,
    iterator_helper_zip_next, iterator_helper_zip_return, iterator_helper_zip_started,
    iterator_slot_value_for_builtin, iterator_this_object, map_completion, number_value,
    property_key_from_text, range_error, read, set_iterator_helper_done, set_iterator_helper_limit,
    set_iterator_helper_running, set_iterator_slot_value_for_builtin, to_number_for_builtin,
    type_error, u64_to_value, BuiltinInvocation, BuiltinIteratorBridge, IteratorHelperKind,
    ObjectRef, OrdinaryObjectData, PublicBuiltinDispatchContext, Value,
    ITERATOR_HELPER_COUNTER_SLOT, ITERATOR_HELPER_INNER_ITERATED_SLOT,
    ITERATOR_HELPER_INNER_NEXT_METHOD_SLOT, ITERATOR_HELPER_KIND_SLOT,
    ITERATOR_HELPER_NEXT_METHOD_SLOT, ITERATOR_HELPER_PARAM_SLOT,
};

pub(super) fn iterator_map_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    iterator_lazy_callback_helper(cx, invocation, IteratorHelperKind::Map)
}

pub(super) fn iterator_filter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    iterator_lazy_callback_helper(cx, invocation, IteratorHelperKind::Filter)
}

pub(super) fn iterator_flat_map_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    iterator_lazy_callback_helper(cx, invocation, IteratorHelperKind::FlatMap)
}

pub(super) fn iterator_take_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    iterator_limit_helper(cx, invocation, IteratorHelperKind::Take)
}

pub(super) fn iterator_drop_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    iterator_limit_helper(cx, invocation, IteratorHelperKind::Drop)
}
fn iterator_lazy_callback_helper<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: IteratorHelperKind,
) -> Result<Value, Cx::Error> {
    let object = iterator_this_object(cx, invocation.this_value())?;
    let callback_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let callback = match cx.require_callable_object(callback_value) {
        Ok(callback) => callback,
        Err(error) => {
            iterator_close_for_validation_failure(cx, object);
            return Err(error);
        }
    };
    let next_key = property_key_from_text(cx, "next");
    let next_method = cx.get_property_value(Value::from_object_ref(object), next_key)?;
    let realm = cx.builtin_realm();
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator_helper_prototype())
        .ok_or_else(|| type_error(cx))?;
    let mut slot_values = vec![
        Value::from_object_ref(object),
        next_method,
        Value::from_bool(false),
        Value::from_bool(false),
        kind.into_value(),
        Value::from_object_ref(callback),
        Value::from_smi(0),
    ];
    if kind == IteratorHelperKind::FlatMap {
        slot_values.push(Value::undefined());
        slot_values.push(Value::undefined());
    }
    let helper = allocate_iterator_object(
        cx,
        prototype,
        OrdinaryObjectData::IteratorHelper,
        slot_values.as_slice(),
    )?;
    Ok(Value::from_object_ref(helper))
}

fn iterator_limit_helper<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: IteratorHelperKind,
) -> Result<Value, Cx::Error> {
    let object = iterator_this_object(cx, invocation.this_value())?;
    let limit_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let limit = match iterator_limit_value(cx, limit_value) {
        Ok(limit) => limit,
        Err(error) => {
            iterator_close_for_validation_failure(cx, object);
            return Err(error);
        }
    };
    let next_key = property_key_from_text(cx, "next");
    let next_method = cx.get_property_value(Value::from_object_ref(object), next_key)?;
    let realm = cx.builtin_realm();
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator_helper_prototype())
        .ok_or_else(|| type_error(cx))?;
    let slot_values = [
        Value::from_object_ref(object),
        next_method,
        Value::from_bool(false),
        Value::from_bool(false),
        kind.into_value(),
        number_value(limit),
        Value::from_smi(0),
    ];
    let helper = allocate_iterator_object(
        cx,
        prototype,
        OrdinaryObjectData::IteratorHelper,
        &slot_values,
    )?;
    Ok(Value::from_object_ref(helper))
}

fn iterator_limit_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<f64, Cx::Error> {
    let number = to_number_for_builtin(cx, value)?;
    if number.is_nan() {
        return Err(range_error(cx));
    }
    let integer = if number == 0.0 {
        0.0
    } else if number.is_finite() {
        number.trunc()
    } else {
        number
    };
    if integer < 0.0 {
        return Err(range_error(cx));
    }
    Ok(integer)
}

pub(super) fn iterator_helper_next_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let helper = iterator_helper_this_object(cx, invocation.this_value())?;
    if iterator_helper_running(cx, helper)? {
        return Err(type_error(cx));
    }
    if iterator_helper_done(cx, helper)? {
        return create_iterator_result_value(cx, Value::undefined(), true);
    }
    let kind = IteratorHelperKind::from_value(iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_KIND_SLOT,
    )?)
    .ok_or_else(|| type_error(cx))?;
    set_iterator_helper_running(cx, helper, true)?;
    let result = match kind {
        IteratorHelperKind::Map => iterator_helper_map_next(cx, helper),
        IteratorHelperKind::Filter => iterator_helper_filter_next(cx, helper),
        IteratorHelperKind::Take => iterator_helper_take_next(cx, helper),
        IteratorHelperKind::Drop => iterator_helper_drop_next(cx, helper),
        IteratorHelperKind::FlatMap => iterator_helper_flat_map_next(cx, helper),
        IteratorHelperKind::Wrap => iterator_helper_wrap_next(cx, helper),
        IteratorHelperKind::Concat => iterator_helper_concat_next(cx, helper),
        IteratorHelperKind::Zip | IteratorHelperKind::ZipKeyed => {
            iterator_helper_zip_next(cx, helper, kind)
        }
    };
    set_iterator_helper_running(cx, helper, false)?;
    result
}

pub(super) fn iterator_helper_return_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let helper = iterator_helper_this_object(cx, invocation.this_value())?;
    if iterator_helper_running(cx, helper)? {
        return Err(type_error(cx));
    }
    let kind = IteratorHelperKind::from_value(iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_KIND_SLOT,
    )?)
    .ok_or_else(|| type_error(cx))?;
    if kind == IteratorHelperKind::Wrap {
        set_iterator_helper_running(cx, helper, true)?;
        let result = iterator_helper_wrap_return(cx, helper);
        set_iterator_helper_running(cx, helper, false)?;
        return result;
    }
    if iterator_helper_done(cx, helper)? {
        return create_iterator_result_value(cx, Value::undefined(), true);
    }
    if matches!(kind, IteratorHelperKind::Zip | IteratorHelperKind::ZipKeyed) {
        set_iterator_helper_done(cx, helper)?;
        let started = iterator_helper_zip_started(cx, helper)?;
        if started {
            set_iterator_helper_running(cx, helper, true)?;
        }
        let result = iterator_helper_zip_return(cx, helper);
        if started {
            set_iterator_helper_running(cx, helper, false)?;
        }
        return result;
    }
    set_iterator_helper_running(cx, helper, true)?;
    set_iterator_helper_done(cx, helper)?;
    if kind == IteratorHelperKind::Concat {
        let result = iterator_helper_concat_return(cx, helper);
        set_iterator_helper_running(cx, helper, false)?;
        return result;
    }
    if kind == IteratorHelperKind::FlatMap
        && let Some(mut inner_record) = iterator_helper_inner_record(cx, helper)?
    {
        clear_iterator_helper_inner(cx, helper)?;
        let inner_close = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_close(
                &mut bridge,
                &mut inner_record,
                Ok::<(), lyng_js_types::AbruptCompletion>(()),
            )
        };
        if let Err(error) = inner_close {
            set_iterator_helper_running(cx, helper, false)?;
            return Err(error);
        }
    }
    let mut iterator_record = iterator_helper_record(cx, helper)?;
    let close_result = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_close(
            &mut bridge,
            &mut iterator_record,
            Ok::<(), lyng_js_types::AbruptCompletion>(()),
        )
    };
    let result = match close_result {
        Ok(()) => create_iterator_result_value(cx, Value::undefined(), true),
        Err(error) => Err(error),
    };
    set_iterator_helper_running(cx, helper, false)?;
    result
}

fn iterator_helper_wrap_next<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    let iterated = iterator_helper_iterated_object(cx, helper)?;
    let next_method = iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_NEXT_METHOD_SLOT,
    )?;
    let next_method = cx.require_callable_object(next_method)?;
    cx.call_to_completion(next_method, Value::from_object_ref(iterated), &[])
}

fn iterator_helper_wrap_return<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    let iterated = iterator_helper_iterated_object(cx, helper)?;
    let return_key = property_key_from_text(cx, "return");
    let return_value = cx.get_property_value(Value::from_object_ref(iterated), return_key)?;
    if return_value.is_undefined() || return_value.is_null() {
        return create_iterator_result_value(cx, Value::undefined(), true);
    }
    let return_method = cx.require_callable_object(return_value)?;
    cx.call_to_completion(return_method, Value::from_object_ref(iterated), &[])
}
fn iterator_helper_source_error<Cx: PublicBuiltinDispatchContext, T>(
    cx: &mut Cx,
    helper: ObjectRef,
    error: Cx::Error,
) -> Result<T, Cx::Error> {
    set_iterator_helper_done(cx, helper)?;
    Err(error)
}

fn iterator_helper_map_next<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    let mut iterator_record = iterator_helper_record(cx, helper)?;
    let next = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_step(&mut bridge, &mut iterator_record)
    };
    let next = match next {
        Ok(next) => next,
        Err(error) => {
            return iterator_helper_source_error(cx, helper, error);
        }
    };
    let Some(next) = next else {
        set_iterator_helper_done(cx, helper)?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    };
    let value = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_value(&mut bridge, next)
    };
    let value = match value {
        Ok(value) => value,
        Err(error) => {
            return iterator_helper_source_error(cx, helper, error);
        }
    };
    let mapper = iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_PARAM_SLOT,
    )?
    .as_object_ref()
    .ok_or_else(|| type_error(cx))?;
    let counter = iterator_helper_counter(cx, helper)?;
    let mapped_value =
        match cx.call_to_completion(mapper, Value::undefined(), &[value, u64_to_value(counter)]) {
            Ok(value) => value,
            Err(error) => {
                set_iterator_helper_done(cx, helper)?;
                return close_iterator_after_error(cx, &mut iterator_record, error);
            }
        };
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_COUNTER_SLOT,
        u64_to_value(counter.saturating_add(1)),
    )?;
    create_iterator_result_value(cx, mapped_value, false)
}

fn iterator_helper_take_next<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    let mut iterator_record = iterator_helper_record(cx, helper)?;
    let remaining = iterator_helper_limit(cx, helper)?;
    if remaining == 0.0 {
        set_iterator_helper_done(cx, helper)?;
        let close_result = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_close(
                &mut bridge,
                &mut iterator_record,
                Ok::<(), lyng_js_types::AbruptCompletion>(()),
            )
        };
        close_result?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    }
    if remaining.is_finite() {
        set_iterator_helper_limit(cx, helper, remaining - 1.0)?;
    }
    let next = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_step(&mut bridge, &mut iterator_record)
    };
    let next = match next {
        Ok(next) => next,
        Err(error) => {
            return iterator_helper_source_error(cx, helper, error);
        }
    };
    let Some(next) = next else {
        set_iterator_helper_done(cx, helper)?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    };
    let value = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_value(&mut bridge, next)
    };
    let value = match value {
        Ok(value) => value,
        Err(error) => {
            return iterator_helper_source_error(cx, helper, error);
        }
    };
    create_iterator_result_value(cx, value, false)
}

fn iterator_helper_drop_next<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    let mut iterator_record = iterator_helper_record(cx, helper)?;
    let mut remaining = iterator_helper_limit(cx, helper)?;
    #[allow(
        clippy::while_float,
        reason = "Iterator helper drop counts are represented as ECMAScript Number limits"
    )]
    while remaining > 0.0 {
        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut iterator_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => {
                return iterator_helper_source_error(cx, helper, error);
            }
        };
        if next.is_none() {
            set_iterator_helper_done(cx, helper)?;
            return create_iterator_result_value(cx, Value::undefined(), true);
        }
        if remaining.is_finite() {
            remaining -= 1.0;
            set_iterator_helper_limit(cx, helper, remaining)?;
        }
    }
    let next = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_step(&mut bridge, &mut iterator_record)
    };
    let next = match next {
        Ok(next) => next,
        Err(error) => {
            return iterator_helper_source_error(cx, helper, error);
        }
    };
    let Some(next) = next else {
        set_iterator_helper_done(cx, helper)?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    };
    let value = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_value(&mut bridge, next)
    };
    let value = match value {
        Ok(value) => value,
        Err(error) => {
            return iterator_helper_source_error(cx, helper, error);
        }
    };
    create_iterator_result_value(cx, value, false)
}

fn iterator_helper_flat_map_next<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    let mut outer_record = iterator_helper_record(cx, helper)?;
    loop {
        if let Some(value) = iterator_helper_flat_map_inner_next(cx, helper, &mut outer_record)? {
            return create_iterator_result_value(cx, value, false);
        }
        if !iterator_helper_flat_map_start_inner(cx, helper, &mut outer_record)? {
            return create_iterator_result_value(cx, Value::undefined(), true);
        }
    }
}

fn iterator_helper_flat_map_inner_next<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    outer_record: &mut iterator::IteratorRecord,
) -> Result<Option<Value>, Cx::Error> {
    let Some(mut inner_record) = (match iterator_helper_inner_record(cx, helper) {
        Ok(record) => record,
        Err(error) => {
            clear_iterator_helper_inner(cx, helper)?;
            set_iterator_helper_done(cx, helper)?;
            return close_iterator_after_error(cx, outer_record, error);
        }
    }) else {
        return Ok(None);
    };
    let next = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_step(&mut bridge, &mut inner_record)
    };
    let next = match next {
        Ok(next) => next,
        Err(error) => {
            clear_iterator_helper_inner(cx, helper)?;
            set_iterator_helper_done(cx, helper)?;
            return close_iterator_after_error(cx, outer_record, error);
        }
    };
    let Some(next) = next else {
        clear_iterator_helper_inner(cx, helper)?;
        return Ok(None);
    };
    let value = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_value(&mut bridge, next)
    };
    match value {
        Ok(value) => Ok(Some(value)),
        Err(error) => {
            clear_iterator_helper_inner(cx, helper)?;
            set_iterator_helper_done(cx, helper)?;
            close_iterator_after_error(cx, outer_record, error)
        }
    }
}

fn iterator_helper_flat_map_start_inner<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    outer_record: &mut iterator::IteratorRecord,
) -> Result<bool, Cx::Error> {
    let next = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_step(&mut bridge, outer_record)
    };
    let next = match next {
        Ok(next) => next,
        Err(error) => return iterator_helper_source_error(cx, helper, error),
    };
    let Some(next) = next else {
        set_iterator_helper_done(cx, helper)?;
        return Ok(false);
    };
    let value = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_value(&mut bridge, next)
    };
    let value = match value {
        Ok(value) => value,
        Err(error) => return iterator_helper_source_error(cx, helper, error),
    };
    let mapper = iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_PARAM_SLOT,
    )?
    .as_object_ref()
    .ok_or_else(|| type_error(cx))?;
    let counter = iterator_helper_counter(cx, helper)?;
    let mapped_value =
        match cx.call_to_completion(mapper, Value::undefined(), &[value, u64_to_value(counter)]) {
            Ok(value) => value,
            Err(error) => {
                set_iterator_helper_done(cx, helper)?;
                return close_iterator_after_error(cx, outer_record, error);
            }
        };
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_COUNTER_SLOT,
        u64_to_value(counter.saturating_add(1)),
    )?;
    let (inner, inner_next) = match get_iterator_flattenable(cx, mapped_value) {
        Ok(record) => record,
        Err(error) => {
            set_iterator_helper_done(cx, helper)?;
            return close_iterator_after_error(cx, outer_record, error);
        }
    };
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_INNER_ITERATED_SLOT,
        Value::from_object_ref(inner),
    )?;
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_INNER_NEXT_METHOD_SLOT,
        inner_next,
    )?;
    Ok(true)
}

fn iterator_helper_filter_next<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    let mut iterator_record = iterator_helper_record(cx, helper)?;
    loop {
        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut iterator_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => {
                return iterator_helper_source_error(cx, helper, error);
            }
        };
        let Some(next) = next else {
            set_iterator_helper_done(cx, helper)?;
            return create_iterator_result_value(cx, Value::undefined(), true);
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => {
                return iterator_helper_source_error(cx, helper, error);
            }
        };
        let predicate = iterator_slot_value_for_builtin(
            cx,
            helper,
            OrdinaryObjectData::IteratorHelper,
            ITERATOR_HELPER_PARAM_SLOT,
        )?
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
        let counter = iterator_helper_counter(cx, helper)?;
        let selected = match cx.call_to_completion(
            predicate,
            Value::undefined(),
            &[value, u64_to_value(counter)],
        ) {
            Ok(selected) => selected,
            Err(error) => {
                set_iterator_helper_done(cx, helper)?;
                return close_iterator_after_error(cx, &mut iterator_record, error);
            }
        };
        set_iterator_slot_value_for_builtin(
            cx,
            helper,
            OrdinaryObjectData::IteratorHelper,
            ITERATOR_HELPER_COUNTER_SLOT,
            u64_to_value(counter.saturating_add(1)),
        )?;
        let selected = {
            let completion = {
                let agent = cx.agent();
                read::to_boolean_agent(agent, selected)
            };
            map_completion(cx, completion)?
        };
        if selected {
            return create_iterator_result_value(cx, value, false);
        }
    }
}

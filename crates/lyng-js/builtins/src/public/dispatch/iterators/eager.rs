use super::{
    close_iterator_after_error, iterator, iterator_close_for_validation_failure,
    iterator_direct_record, iterator_this_object, map_completion, read, type_error, u64_to_value,
    BuiltinInvocation, BuiltinIteratorBridge, PublicBuiltinDispatchContext, Value,
};

pub(super) fn iterator_reduce_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = iterator_this_object(cx, invocation.this_value())?;
    let reducer_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let reducer = match cx.require_callable_object(reducer_value) {
        Ok(reducer) => reducer,
        Err(error) => {
            iterator_close_for_validation_failure(cx, object);
            return Err(error);
        }
    };
    let mut iterator_record = iterator_direct_record(cx, object)?;
    let initial = invocation.arguments().get(1).copied();
    let (mut accumulator, mut counter): (Value, u64) = if let Some(value) = initial {
        (value, 0)
    } else {
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
            return Err(type_error(cx));
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        (value, 1)
    };
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
            return Ok(accumulator);
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let counter_value = u64_to_value(counter);
        match cx.call_to_completion(
            reducer,
            Value::undefined(),
            &[accumulator, value, counter_value],
        ) {
            Ok(result) => {
                accumulator = result;
                counter = counter.saturating_add(1);
            }
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        }
    }
}

pub(super) fn iterator_for_each_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
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
    let mut iterator_record = iterator_direct_record(cx, object)?;
    let mut counter: u64 = 0;
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
            return Ok(Value::undefined());
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let counter_value = u64_to_value(counter);
        if let Err(error) =
            cx.call_to_completion(callback, Value::undefined(), &[value, counter_value])
        {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
        counter = counter.saturating_add(1);
    }
}

pub(super) fn iterator_some_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    iterator_predicate_helper(cx, invocation, true)
}

pub(super) fn iterator_every_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    iterator_predicate_helper(cx, invocation, false)
}

// some: returns true on first truthy → true; default false.
// every: returns false on first falsy → false; default true.
fn iterator_predicate_helper<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    is_some: bool,
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
    let mut iterator_record = iterator_direct_record(cx, object)?;
    let mut counter: u64 = 0;
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
            return Ok(Value::from_bool(!is_some));
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let counter_value = u64_to_value(counter);
        let result =
            match cx.call_to_completion(callback, Value::undefined(), &[value, counter_value]) {
                Ok(result) => result,
                Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
            };
        let truthy = {
            let completion = {
                let agent = cx.agent();
                read::to_boolean_agent(agent, result)
            };
            map_completion(cx, completion)?
        };
        if is_some && truthy {
            // some short-circuits to true
            let close_result = {
                let mut bridge = BuiltinIteratorBridge { cx };
                iterator::iterator_close(
                    &mut bridge,
                    &mut iterator_record,
                    Ok::<(), lyng_js_types::AbruptCompletion>(()),
                )
            };
            close_result?;
            return Ok(Value::from_bool(true));
        }
        if !is_some && !truthy {
            // every short-circuits to false
            let close_result = {
                let mut bridge = BuiltinIteratorBridge { cx };
                iterator::iterator_close(
                    &mut bridge,
                    &mut iterator_record,
                    Ok::<(), lyng_js_types::AbruptCompletion>(()),
                )
            };
            close_result?;
            return Ok(Value::from_bool(false));
        }
        counter = counter.saturating_add(1);
    }
}

pub(super) fn iterator_find_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
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
    let mut iterator_record = iterator_direct_record(cx, object)?;
    let mut counter: u64 = 0;
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
            return Ok(Value::undefined());
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let counter_value = u64_to_value(counter);
        let result =
            match cx.call_to_completion(callback, Value::undefined(), &[value, counter_value]) {
                Ok(result) => result,
                Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
            };
        let truthy = {
            let completion = {
                let agent = cx.agent();
                read::to_boolean_agent(agent, result)
            };
            map_completion(cx, completion)?
        };
        if truthy {
            let close_result = {
                let mut bridge = BuiltinIteratorBridge { cx };
                iterator::iterator_close(
                    &mut bridge,
                    &mut iterator_record,
                    Ok::<(), lyng_js_types::AbruptCompletion>(()),
                )
            };
            close_result?;
            return Ok(value);
        }
        counter = counter.saturating_add(1);
    }
}

pub(super) fn iterator_to_array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = iterator_this_object(cx, invocation.this_value())?;
    let mut iterator_record = iterator_direct_record(cx, object)?;
    let mut values: Vec<Value> = Vec::new();
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
            break;
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        values.push(value);
    }
    let array = super::create_array_from_values(cx, &values)?;
    Ok(Value::from_object_ref(array))
}

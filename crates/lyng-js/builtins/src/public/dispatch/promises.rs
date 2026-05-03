use super::{
    close_iterator_after_error, create_array_from_values, create_data_property_or_throw,
    define_data_property_with_attrs, map_completion, promise_all_resolve_element_builtin,
    promise_all_settled_reject_element_builtin, promise_all_settled_resolve_element_builtin,
    promise_any_reject_element_builtin, property_key_from_text, require_constructor_object,
    string_value, type_error, BuiltinIteratorBridge, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_common::WellKnownAtom;
use lyng_js_env::{
    Agent, PromiseCombinatorElementKind, PromiseCombinatorElementRecord, PromiseCombinatorKind,
    PromiseFinallyFunctionKind, PromiseFinallyFunctionRecord, PromiseReactionHandler,
    PromiseReactionKind, PromiseResolvingFunctionKind, PromiseState,
};
use lyng_js_ops::{errors, iterator, promise};
use lyng_js_types::{
    AbruptCompletion, BuiltinFunctionId, ObjectRef, PropertyKey, RealmRef, Value, WellKnownSymbolId,
};

pub(super) fn dispatch_promise_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_promise_constructor_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_promise_prototype_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_promise_internal_builtin(context, entry, invocation)
}

fn dispatch_promise_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::promise_builtin() {
        return promise_builtin(context, invocation).map(Some);
    }
    if entry == super::promise_resolve_builtin() {
        return promise_resolve_builtin(context, invocation).map(Some);
    }
    if entry == super::promise_reject_builtin() {
        return promise_reject_builtin(context, invocation).map(Some);
    }
    if entry == super::promise_all_builtin() {
        return promise_all_builtin(context, invocation).map(Some);
    }
    if entry == super::promise_all_settled_builtin() {
        return promise_all_settled_builtin(context, invocation).map(Some);
    }
    if entry == super::promise_race_builtin() {
        return promise_race_builtin(context, invocation).map(Some);
    }
    if entry == super::promise_any_builtin() {
        return promise_any_builtin(context, invocation).map(Some);
    }
    if entry == super::promise_try_builtin() {
        return promise_try_builtin(context, invocation).map(Some);
    }
    if entry == super::promise_with_resolvers_builtin() {
        return promise_with_resolvers_builtin(context, invocation).map(Some);
    }
    if entry == super::promise_species_getter_builtin() {
        return promise_species_getter_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_promise_prototype_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::promise_then_builtin() {
        return promise_then_builtin(context, invocation).map(Some);
    }
    if entry == super::promise_catch_builtin() {
        return promise_catch_builtin(context, invocation).map(Some);
    }
    if entry == super::promise_finally_builtin() {
        return promise_finally_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_promise_internal_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::promise_capability_executor_builtin() {
        return promise_capability_executor_builtin(context, invocation).map(Some);
    }
    if entry == super::promise_resolve_function_builtin() {
        return promise_resolve_function_builtin(context, invocation).map(Some);
    }
    if entry == super::promise_reject_function_builtin() {
        return promise_reject_function_builtin(context, invocation).map(Some);
    }
    if entry == super::promise_finally_function_builtin() {
        return promise_finally_function_builtin(context, invocation).map(Some);
    }
    if entry == super::promise_finally_continuation_builtin() {
        return promise_finally_function_builtin(context, invocation).map(Some);
    }
    if entry == super::promise_all_resolve_element_builtin() {
        return promise_combinator_element_builtin(
            context,
            invocation,
            PromiseCombinatorElementKind::AllResolve,
        )
        .map(Some);
    }
    if entry == super::promise_all_settled_resolve_element_builtin() {
        return promise_combinator_element_builtin(
            context,
            invocation,
            PromiseCombinatorElementKind::AllSettledResolve,
        )
        .map(Some);
    }
    if entry == super::promise_all_settled_reject_element_builtin() {
        return promise_combinator_element_builtin(
            context,
            invocation,
            PromiseCombinatorElementKind::AllSettledReject,
        )
        .map(Some);
    }
    if entry == super::promise_any_reject_element_builtin() {
        return promise_combinator_element_builtin(
            context,
            invocation,
            PromiseCombinatorElementKind::AnyReject,
        )
        .map(Some);
    }
    Ok(None)
}

fn promise_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let executor = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let executor = cx.require_callable_object(executor)?;
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let realm = cx.builtin_realm();
    let default_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|realm| realm.intrinsics().promise_prototype())
        .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let promise_object = cx.allocate_ordinary_object_with_prototype(realm, Some(prototype))?;
    let _ = cx.agent().alloc_promise(promise_object, realm);
    let capability = cx.agent().alloc_promise_capability();
    let _ = cx
        .agent()
        .set_promise_capability_promise(capability, promise_object);
    let resolve = cx.allocate_builtin_function(super::promise_resolve_function_builtin())?;
    let reject = cx.allocate_builtin_function(super::promise_reject_function_builtin())?;
    let _ = cx
        .agent()
        .set_promise_capability_resolve(capability, resolve);
    let _ = cx.agent().set_promise_capability_reject(capability, reject);
    let _ = cx.agent().alloc_promise_resolving_function(
        resolve,
        lyng_js_env::PromiseResolvingFunctionRecord::new(
            PromiseResolvingFunctionKind::Resolve,
            capability,
        ),
    );
    let _ = cx.agent().alloc_promise_resolving_function(
        reject,
        lyng_js_env::PromiseResolvingFunctionRecord::new(
            PromiseResolvingFunctionKind::Reject,
            capability,
        ),
    );
    if let Err(error) = cx.call_to_completion(
        executor,
        Value::undefined(),
        &[
            Value::from_object_ref(resolve),
            Value::from_object_ref(reject),
        ],
    ) {
        if let Some(thrown) = cx.extract_thrown_value(error)? {
            let _ = cx.call_to_completion(reject, Value::undefined(), &[thrown])?;
        }
    }
    Ok(Value::from_object_ref(promise_object))
}

fn promise_then_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let promise = require_promise_receiver(cx, invocation.this_value())?;
    let on_fulfilled = reaction_handler_for_value(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
        PromiseReactionHandler::Identity,
    );
    let on_rejected = reaction_handler_for_value(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
        PromiseReactionHandler::Thrower,
    );
    perform_promise_then(cx, promise, on_fulfilled, on_rejected)
}

fn promise_catch_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let promise = invocation.this_value();
    let on_rejected = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    invoke_then_method(cx, promise, Value::undefined(), on_rejected)
}

fn promise_finally_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let promise = invocation.this_value();
    let on_finally = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(on_finally_object) = on_finally
        .as_object_ref()
        .filter(|object| cx.agent().objects().is_callable(*object))
    else {
        return invoke_then_method(cx, promise, on_finally, on_finally);
    };
    let promise_object = promise.as_object_ref().ok_or_else(|| type_error(cx))?;
    let constructor = promise_species_constructor(cx, promise_object)?;
    let then_finally = allocate_promise_finally_function(
        cx,
        PromiseFinallyFunctionKind::Then,
        on_finally_object,
        constructor,
    )?;
    let catch_finally = allocate_promise_finally_function(
        cx,
        PromiseFinallyFunctionKind::Catch,
        on_finally_object,
        constructor,
    )?;
    invoke_then_method(
        cx,
        promise,
        Value::from_object_ref(then_finally),
        Value::from_object_ref(catch_finally),
    )
}

fn promise_resolve_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let constructor = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let resolution = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if let Some(promise_object) = resolution
        .as_object_ref()
        .filter(|object| cx.agent().promise_record(*object).is_some())
    {
        let constructor_value = cx.get_property_value(
            Value::from_object_ref(promise_object),
            PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        )?;
        if constructor_value.as_object_ref() == Some(constructor) {
            return Ok(resolution);
        }
    }

    let capability = new_promise_capability(cx, constructor)?;
    let promise_object = promise_capability_promise(cx, capability)?;
    let resolve = promise_capability_resolve(cx, capability)?;
    let _ = cx.call_to_completion(resolve, Value::undefined(), &[resolution])?;
    Ok(Value::from_object_ref(promise_object))
}

fn promise_reject_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let constructor = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let reason = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let capability = new_promise_capability(cx, constructor)?;
    let promise_object = promise_capability_promise(cx, capability)?;
    let reject = promise_capability_reject(cx, capability)?;
    let _ = cx.call_to_completion(reject, Value::undefined(), &[reason])?;
    Ok(Value::from_object_ref(promise_object))
}

fn promise_try_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let constructor = require_constructor_object(cx, invocation.this_value())?;
    let capability = new_promise_capability(cx, constructor)?;
    let promise_object = promise_capability_promise(cx, capability)?;
    let arguments = invocation.arguments();
    let callback = arguments.first().copied().unwrap_or(Value::undefined());
    let callback_object = cx.require_callable_object(callback)?;
    let extra_args: Vec<Value> = arguments.iter().skip(1).copied().collect();
    match cx.call_to_completion(callback_object, Value::undefined(), &extra_args) {
        Ok(value) => {
            let resolve = promise_capability_resolve(cx, capability)?;
            let _ = cx.call_to_completion(resolve, Value::undefined(), &[value])?;
        }
        Err(error) => {
            reject_promise_capability_error(cx, capability, error)?;
        }
    }
    Ok(Value::from_object_ref(promise_object))
}

fn promise_with_resolvers_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let constructor = require_constructor_object(cx, invocation.this_value())?;
    let capability = new_promise_capability(cx, constructor)?;
    let promise_object = promise_capability_promise(cx, capability)?;
    let resolve = promise_capability_resolve(cx, capability)?;
    let reject = promise_capability_reject(cx, capability)?;
    let realm = cx.builtin_realm();
    let object_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|realm| realm.intrinsics().object_prototype())
        .ok_or_else(|| type_error(cx))?;
    let result = cx.allocate_ordinary_object_with_prototype(realm, Some(object_prototype))?;
    let promise_key = property_key_from_text(cx, "promise");
    let resolve_key = property_key_from_text(cx, "resolve");
    let reject_key = property_key_from_text(cx, "reject");
    create_data_property_or_throw(
        cx,
        result,
        promise_key,
        Value::from_object_ref(promise_object),
    )?;
    create_data_property_or_throw(cx, result, resolve_key, Value::from_object_ref(resolve))?;
    create_data_property_or_throw(cx, result, reject_key, Value::from_object_ref(reject))?;
    Ok(Value::from_object_ref(result))
}

fn promise_all_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    promise_collecting_combinator_builtin(cx, invocation, PromiseCombinatorKind::All)
}

fn promise_all_settled_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    promise_collecting_combinator_builtin(cx, invocation, PromiseCombinatorKind::AllSettled)
}

fn promise_race_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let constructor = require_constructor_object(cx, invocation.this_value())?;
    let capability = new_promise_capability(cx, constructor)?;
    let promise = Value::from_object_ref(promise_capability_promise(cx, capability)?);
    let iterable = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if let Err(error) = perform_promise_race(cx, constructor, capability, iterable) {
        reject_promise_capability_error(cx, capability, error)?;
    }
    Ok(promise)
}

fn promise_any_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    promise_collecting_combinator_builtin(cx, invocation, PromiseCombinatorKind::Any)
}

fn promise_species_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    _cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Ok(invocation.this_value())
}

fn promise_capability_executor_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let function = cx.callee_object();
    let record = cx
        .agent()
        .promise_resolving_function(function)
        .ok_or_else(|| type_error(cx))?;
    if record.kind() != PromiseResolvingFunctionKind::CapabilityExecutor {
        return Err(type_error(cx));
    }
    let capability = record.capability();
    if cx
        .agent()
        .promise_capability(capability)
        .is_some_and(|record| {
            record
                .resolve_value()
                .is_some_and(|value| !value.is_undefined())
                || record
                    .reject_value()
                    .is_some_and(|value| !value.is_undefined())
        })
    {
        return Err(type_error(cx));
    }
    let resolve = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let reject = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let _ = cx
        .agent()
        .set_promise_capability_resolve_value(capability, resolve);
    let _ = cx
        .agent()
        .set_promise_capability_reject_value(capability, reject);
    Ok(Value::undefined())
}

fn promise_resolve_function_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    promise_resolving_function_builtin(cx, invocation, PromiseResolvingFunctionKind::Resolve)
}

fn promise_reject_function_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    promise_resolving_function_builtin(cx, invocation, PromiseResolvingFunctionKind::Reject)
}

fn promise_combinator_element_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    expected_kind: PromiseCombinatorElementKind,
) -> Result<Value, Cx::Error> {
    let function = cx.callee_object();
    let record = cx
        .agent()
        .promise_combinator_element(function)
        .ok_or_else(|| type_error(cx))?;
    if record.kind() != expected_kind {
        return Err(type_error(cx));
    }
    let combinator_id = record.combinator();
    if cx
        .agent()
        .promise_combinator_already_called(combinator_id, record.index())
        .ok_or_else(|| type_error(cx))?
    {
        return Ok(Value::undefined());
    }
    let _ = cx
        .agent()
        .set_promise_combinator_already_called(combinator_id, record.index(), true);
    let capability = cx
        .agent()
        .promise_combinator(combinator_id)
        .map(lyng_js_env::PromiseCombinatorRecord::capability)
        .ok_or_else(|| type_error(cx))?;
    let argument = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    match expected_kind {
        PromiseCombinatorElementKind::AllResolve => {
            if !cx
                .agent()
                .set_promise_combinator_value(combinator_id, record.index(), argument)
            {
                return Err(type_error(cx));
            }
            let remaining = cx
                .agent()
                .decrement_promise_combinator_remaining(combinator_id)
                .ok_or_else(|| type_error(cx))?;
            if remaining == 0 {
                let values = promise_combinator_values_array(cx, combinator_id)?;
                let resolve = promise_capability_resolve(cx, capability)?;
                let _ = cx.call_to_completion(
                    resolve,
                    Value::undefined(),
                    &[Value::from_object_ref(values)],
                )?;
            }
        }
        PromiseCombinatorElementKind::AllSettledResolve
        | PromiseCombinatorElementKind::AllSettledReject => {
            let settled = promise_all_settled_result_object(cx, expected_kind, argument)?;
            if !cx.agent().set_promise_combinator_value(
                combinator_id,
                record.index(),
                Value::from_object_ref(settled),
            ) {
                return Err(type_error(cx));
            }
            let remaining = cx
                .agent()
                .decrement_promise_combinator_remaining(combinator_id)
                .ok_or_else(|| type_error(cx))?;
            if remaining == 0 {
                let values = promise_combinator_values_array(cx, combinator_id)?;
                let resolve = promise_capability_resolve(cx, capability)?;
                let _ = cx.call_to_completion(
                    resolve,
                    Value::undefined(),
                    &[Value::from_object_ref(values)],
                )?;
            }
        }
        PromiseCombinatorElementKind::AnyReject => {
            if !cx
                .agent()
                .set_promise_combinator_value(combinator_id, record.index(), argument)
            {
                return Err(type_error(cx));
            }
            let remaining = cx
                .agent()
                .decrement_promise_combinator_remaining(combinator_id)
                .ok_or_else(|| type_error(cx))?;
            if remaining == 0 {
                reject_promise_any_errors(cx, capability, combinator_id)?;
            }
        }
    }
    Ok(Value::undefined())
}

fn promise_collecting_combinator_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: PromiseCombinatorKind,
) -> Result<Value, Cx::Error> {
    let constructor = require_constructor_object(cx, invocation.this_value())?;
    let capability = new_promise_capability(cx, constructor)?;
    let promise = Value::from_object_ref(promise_capability_promise(cx, capability)?);
    let iterable = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let combinator = cx.agent().alloc_promise_combinator(kind, capability);
    let result = match kind {
        PromiseCombinatorKind::All => {
            perform_promise_all(cx, constructor, capability, combinator, iterable)
        }
        PromiseCombinatorKind::AllSettled => {
            perform_promise_all_settled(cx, constructor, capability, combinator, iterable)
        }
        PromiseCombinatorKind::Any => {
            perform_promise_any(cx, constructor, capability, combinator, iterable)
        }
    };
    if let Err(error) = result {
        reject_promise_capability_error(cx, capability, error)?;
    }
    Ok(promise)
}

fn perform_promise_all<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor: ObjectRef,
    capability: lyng_js_env::PromiseCapabilityId,
    combinator: lyng_js_env::PromiseCombinatorId,
    iterable: Value,
) -> Result<(), Cx::Error> {
    let promise_resolve = promise_resolve_method(cx, constructor)?;
    let reject = promise_capability_reject(cx, capability)?;
    let mut iterator_record = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::get_iterator(&mut bridge, iterable)?
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
            let remaining = cx
                .agent()
                .decrement_promise_combinator_remaining(combinator)
                .ok_or_else(|| type_error(cx))?;
            if remaining == 0 {
                let values = promise_combinator_values_array(cx, combinator)?;
                let resolve = promise_capability_resolve(cx, capability)?;
                let _ = cx.call_to_completion(
                    resolve,
                    Value::undefined(),
                    &[Value::from_object_ref(values)],
                )?;
            }
            return Ok(());
        };
        let next_value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let next_value = match next_value {
            Ok(next_value) => next_value,
            Err(error) => {
                iterator_record.set_done(true);
                return Err(error);
            }
        };
        let index = cx
            .agent()
            .push_promise_combinator_placeholder(combinator)
            .ok_or_else(|| type_error(cx))?;
        let next_promise = match cx.call_to_completion(
            promise_resolve,
            Value::from_object_ref(constructor),
            &[next_value],
        ) {
            Ok(next_promise) => next_promise,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let resolve_element = allocate_promise_combinator_element(
            cx,
            combinator,
            index,
            PromiseCombinatorElementKind::AllResolve,
        )?;
        if let Err(error) = invoke_then_method(
            cx,
            next_promise,
            Value::from_object_ref(resolve_element),
            Value::from_object_ref(reject),
        ) {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    }
}

fn perform_promise_all_settled<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor: ObjectRef,
    capability: lyng_js_env::PromiseCapabilityId,
    combinator: lyng_js_env::PromiseCombinatorId,
    iterable: Value,
) -> Result<(), Cx::Error> {
    let promise_resolve = promise_resolve_method(cx, constructor)?;
    let mut iterator_record = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::get_iterator(&mut bridge, iterable)?
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
            let remaining = cx
                .agent()
                .decrement_promise_combinator_remaining(combinator)
                .ok_or_else(|| type_error(cx))?;
            if remaining == 0 {
                let values = promise_combinator_values_array(cx, combinator)?;
                let resolve = promise_capability_resolve(cx, capability)?;
                let _ = cx.call_to_completion(
                    resolve,
                    Value::undefined(),
                    &[Value::from_object_ref(values)],
                )?;
            }
            return Ok(());
        };
        let next_value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let next_value = match next_value {
            Ok(next_value) => next_value,
            Err(error) => {
                iterator_record.set_done(true);
                return Err(error);
            }
        };
        let index = cx
            .agent()
            .push_promise_combinator_placeholder(combinator)
            .ok_or_else(|| type_error(cx))?;
        let next_promise = match cx.call_to_completion(
            promise_resolve,
            Value::from_object_ref(constructor),
            &[next_value],
        ) {
            Ok(next_promise) => next_promise,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let resolve_element = allocate_promise_combinator_element(
            cx,
            combinator,
            index,
            PromiseCombinatorElementKind::AllSettledResolve,
        )?;
        let reject_element = allocate_promise_combinator_element(
            cx,
            combinator,
            index,
            PromiseCombinatorElementKind::AllSettledReject,
        )?;
        if let Err(error) = invoke_then_method(
            cx,
            next_promise,
            Value::from_object_ref(resolve_element),
            Value::from_object_ref(reject_element),
        ) {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    }
}

fn perform_promise_any<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor: ObjectRef,
    capability: lyng_js_env::PromiseCapabilityId,
    combinator: lyng_js_env::PromiseCombinatorId,
    iterable: Value,
) -> Result<(), Cx::Error> {
    let promise_resolve = promise_resolve_method(cx, constructor)?;
    let resolve = promise_capability_resolve(cx, capability)?;
    let mut iterator_record = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::get_iterator(&mut bridge, iterable)?
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
            let remaining = cx
                .agent()
                .decrement_promise_combinator_remaining(combinator)
                .ok_or_else(|| type_error(cx))?;
            if remaining == 0 {
                reject_promise_any_errors(cx, capability, combinator)?;
            }
            return Ok(());
        };
        let next_value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let next_value = match next_value {
            Ok(next_value) => next_value,
            Err(error) => {
                iterator_record.set_done(true);
                return Err(error);
            }
        };
        let index = cx
            .agent()
            .push_promise_combinator_placeholder(combinator)
            .ok_or_else(|| type_error(cx))?;
        let next_promise = match cx.call_to_completion(
            promise_resolve,
            Value::from_object_ref(constructor),
            &[next_value],
        ) {
            Ok(next_promise) => next_promise,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let reject_element = allocate_promise_combinator_element(
            cx,
            combinator,
            index,
            PromiseCombinatorElementKind::AnyReject,
        )?;
        if let Err(error) = invoke_then_method(
            cx,
            next_promise,
            Value::from_object_ref(resolve),
            Value::from_object_ref(reject_element),
        ) {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    }
}

fn perform_promise_race<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor: ObjectRef,
    capability: lyng_js_env::PromiseCapabilityId,
    iterable: Value,
) -> Result<(), Cx::Error> {
    let promise_resolve = promise_resolve_method(cx, constructor)?;
    let resolve = promise_capability_resolve(cx, capability)?;
    let reject = promise_capability_reject(cx, capability)?;
    let mut iterator_record = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::get_iterator(&mut bridge, iterable)?
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
            return Ok(());
        };
        let next_value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let next_value = match next_value {
            Ok(next_value) => next_value,
            Err(error) => {
                iterator_record.set_done(true);
                return Err(error);
            }
        };
        let next_promise = match cx.call_to_completion(
            promise_resolve,
            Value::from_object_ref(constructor),
            &[next_value],
        ) {
            Ok(next_promise) => next_promise,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        if let Err(error) = invoke_then_method(
            cx,
            next_promise,
            Value::from_object_ref(resolve),
            Value::from_object_ref(reject),
        ) {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    }
}

fn reject_promise_any_errors<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    capability: lyng_js_env::PromiseCapabilityId,
    combinator: lyng_js_env::PromiseCombinatorId,
) -> Result<(), Cx::Error> {
    let reasons = cx
        .agent()
        .promise_combinator(combinator)
        .map(lyng_js_env::PromiseCombinatorRecord::values)
        .map(<[Value]>::to_vec)
        .ok_or_else(|| type_error(cx))?;
    let aggregate_error = create_aggregate_error_from_values(cx, &reasons, None)?;
    let reject = promise_capability_reject(cx, capability)?;
    let _ = cx.call_to_completion(
        reject,
        Value::undefined(),
        &[Value::from_object_ref(aggregate_error)],
    )?;
    Ok(())
}

pub(super) fn promise_resolve_method<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor: ObjectRef,
) -> Result<ObjectRef, Cx::Error> {
    let resolve_key = property_key_from_text(cx, "resolve");
    let resolve = cx.get_property_value(Value::from_object_ref(constructor), resolve_key)?;
    cx.require_callable_object(resolve)
}

pub(super) fn invoke_then_method<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    promise: Value,
    on_fulfilled: Value,
    on_rejected: Value,
) -> Result<Value, Cx::Error> {
    invoke_then_method_with_args(cx, promise, &[on_fulfilled, on_rejected])
}

fn invoke_then_method_with_args<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    promise: Value,
    arguments: &[Value],
) -> Result<Value, Cx::Error> {
    let then_key = property_key_from_text(cx, "then");
    let then = cx.get_property_value(promise, then_key)?;
    let then = cx.require_callable_object(then)?;
    cx.call_to_completion(then, promise, arguments)
}

fn allocate_promise_finally_function<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: PromiseFinallyFunctionKind,
    on_finally: ObjectRef,
    constructor: ObjectRef,
) -> Result<ObjectRef, Cx::Error> {
    let function = cx.allocate_builtin_function(super::promise_finally_function_builtin())?;
    let _ = cx.agent().alloc_promise_finally_function(
        function,
        PromiseFinallyFunctionRecord::new(kind, on_finally, constructor),
    );
    Ok(function)
}

fn allocate_promise_finally_continuation<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: PromiseFinallyFunctionKind,
    argument: Value,
) -> Result<ObjectRef, Cx::Error> {
    let function = cx.allocate_builtin_function(super::promise_finally_continuation_builtin())?;
    let _ = cx.agent().alloc_promise_finally_function(
        function,
        PromiseFinallyFunctionRecord::continuation(kind, argument),
    );
    Ok(function)
}

fn allocate_promise_combinator_element<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    combinator: lyng_js_env::PromiseCombinatorId,
    index: usize,
    kind: PromiseCombinatorElementKind,
) -> Result<ObjectRef, Cx::Error> {
    let entry = promise_combinator_element_entry(kind);
    let function = cx.allocate_builtin_function(entry)?;
    let _ = cx.agent().alloc_promise_combinator_element(
        function,
        PromiseCombinatorElementRecord::new(kind, combinator, index),
    );
    Ok(function)
}

fn promise_combinator_element_entry(kind: PromiseCombinatorElementKind) -> BuiltinFunctionId {
    match kind {
        PromiseCombinatorElementKind::AllResolve => promise_all_resolve_element_builtin(),
        PromiseCombinatorElementKind::AllSettledResolve => {
            promise_all_settled_resolve_element_builtin()
        }
        PromiseCombinatorElementKind::AllSettledReject => {
            promise_all_settled_reject_element_builtin()
        }
        PromiseCombinatorElementKind::AnyReject => promise_any_reject_element_builtin(),
    }
}

fn reject_promise_capability_error<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    capability: lyng_js_env::PromiseCapabilityId,
    error: Cx::Error,
) -> Result<(), Cx::Error> {
    let Some(thrown) = cx.extract_thrown_value(error)? else {
        unreachable!("non-abrupt builtin error should propagate")
    };
    let reject = promise_capability_reject(cx, capability)?;
    let _ = cx.call_to_completion(reject, Value::undefined(), &[thrown])?;
    Ok(())
}

fn promise_combinator_values_array<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    combinator: lyng_js_env::PromiseCombinatorId,
) -> Result<ObjectRef, Cx::Error> {
    let values = cx
        .agent()
        .promise_combinator(combinator)
        .map(lyng_js_env::PromiseCombinatorRecord::values)
        .map(<[Value]>::to_vec)
        .ok_or_else(|| type_error(cx))?;
    create_array_from_values(cx, &values)
}

fn create_aggregate_error_from_values<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    values: &[Value],
    message: Option<Value>,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().aggregate_error_prototype())
        .ok_or_else(|| type_error(cx))?;
    let error = {
        let agent = cx.agent();
        errors::create_error_object(agent, realm, Some(prototype), message)
    };
    let error = map_completion(cx, error)?;
    let errors_array = create_array_from_values(cx, values)?;
    let errors_key = property_key_from_text(cx, "errors");
    define_data_property_with_attrs(
        cx,
        error,
        errors_key,
        Value::from_object_ref(errors_array),
        true,
        false,
        true,
    )?;
    Ok(error)
}

fn promise_all_settled_result_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: PromiseCombinatorElementKind,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    let object_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|realm| realm.intrinsics().object_prototype())
        .ok_or_else(|| type_error(cx))?;
    let result = cx.allocate_ordinary_object_with_prototype(realm, Some(object_prototype))?;
    let status = match kind {
        PromiseCombinatorElementKind::AllSettledResolve => string_value(cx, "fulfilled"),
        PromiseCombinatorElementKind::AllSettledReject => string_value(cx, "rejected"),
        _ => return Err(type_error(cx)),
    };
    let status_key = property_key_from_text(cx, "status");
    create_data_property_or_throw(cx, result, status_key, status)?;
    let key = match kind {
        PromiseCombinatorElementKind::AllSettledResolve => property_key_from_text(cx, "value"),
        PromiseCombinatorElementKind::AllSettledReject => property_key_from_text(cx, "reason"),
        _ => return Err(type_error(cx)),
    };
    create_data_property_or_throw(cx, result, key, value)?;
    Ok(result)
}

fn require_promise_receiver<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if cx.agent().promise_record(object).is_none() {
        return Err(type_error(cx));
    }
    Ok(object)
}

fn reaction_handler_for_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    fallback: PromiseReactionHandler,
) -> PromiseReactionHandler {
    value
        .as_object_ref()
        .filter(|object| cx.agent().objects().is_callable(*object))
        .map_or(fallback, PromiseReactionHandler::Callable)
}

pub(super) fn promise_default_constructor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    cx.agent()
        .realm(realm)
        .and_then(|realm| realm.intrinsics().promise())
        .ok_or_else(|| type_error(cx))
}

fn promise_species_constructor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    promise_object: ObjectRef,
) -> Result<ObjectRef, Cx::Error> {
    let default_constructor = promise_default_constructor(cx)?;
    let constructor = cx.get_property_value(
        Value::from_object_ref(promise_object),
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
    )?;
    if constructor.is_undefined() {
        return Ok(default_constructor);
    }
    let constructor = constructor.as_object_ref().ok_or_else(|| type_error(cx))?;
    let species_symbol = cx
        .agent()
        .well_known_symbol(WellKnownSymbolId::Species)
        .ok_or_else(|| type_error(cx))?;
    let species = cx.get_property_value(
        Value::from_object_ref(constructor),
        PropertyKey::from_symbol(species_symbol),
    )?;
    if species.is_undefined() || species.is_null() {
        return Ok(default_constructor);
    }
    let species = species.as_object_ref().ok_or_else(|| type_error(cx))?;
    if !cx.agent().objects().is_constructor(species) {
        return Err(type_error(cx));
    }
    Ok(species)
}

pub(super) fn new_promise_capability<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    constructor: ObjectRef,
) -> Result<lyng_js_env::PromiseCapabilityId, Cx::Error> {
    let capability = cx.agent().alloc_promise_capability();
    let executor = cx.allocate_builtin_function(super::promise_capability_executor_builtin())?;
    let _ = cx.agent().alloc_promise_resolving_function(
        executor,
        lyng_js_env::PromiseResolvingFunctionRecord::new(
            PromiseResolvingFunctionKind::CapabilityExecutor,
            capability,
        ),
    );
    let promise = cx.construct_to_completion(
        constructor,
        &[Value::from_object_ref(executor)],
        Some(constructor),
    )?;
    let _ = cx
        .agent()
        .set_promise_capability_promise(capability, promise);
    let (resolve, reject) = cx
        .agent()
        .promise_capability(capability)
        .map(|record| (record.resolve_value(), record.reject_value()))
        .ok_or_else(|| type_error(cx))?;
    let resolve = resolve.ok_or_else(|| type_error(cx))?;
    let reject = reject.ok_or_else(|| type_error(cx))?;
    let resolve = cx.require_callable_object(resolve)?;
    let reject = cx.require_callable_object(reject)?;
    let _ = cx
        .agent()
        .set_promise_capability_resolve(capability, resolve);
    let _ = cx.agent().set_promise_capability_reject(capability, reject);
    Ok(capability)
}

pub(super) fn promise_capability_promise<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    capability: lyng_js_env::PromiseCapabilityId,
) -> Result<ObjectRef, Cx::Error> {
    cx.agent()
        .promise_capability(capability)
        .and_then(lyng_js_env::PromiseCapabilityRecord::promise)
        .ok_or_else(|| type_error(cx))
}

pub(super) fn promise_capability_resolve<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    capability: lyng_js_env::PromiseCapabilityId,
) -> Result<ObjectRef, Cx::Error> {
    cx.agent()
        .promise_capability(capability)
        .and_then(lyng_js_env::PromiseCapabilityRecord::resolve)
        .ok_or_else(|| type_error(cx))
}

pub(super) fn promise_capability_reject<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    capability: lyng_js_env::PromiseCapabilityId,
) -> Result<ObjectRef, Cx::Error> {
    cx.agent()
        .promise_capability(capability)
        .and_then(lyng_js_env::PromiseCapabilityRecord::reject)
        .ok_or_else(|| type_error(cx))
}

pub(super) fn perform_promise_then_with_capability<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    promise_object: ObjectRef,
    on_fulfilled: PromiseReactionHandler,
    on_rejected: PromiseReactionHandler,
    capability: Option<lyng_js_env::PromiseCapabilityId>,
) -> Result<(), Cx::Error> {
    let fulfill_reaction = promise::create_promise_reaction(
        cx.agent(),
        PromiseReactionKind::Fulfill,
        on_fulfilled,
        capability,
    );
    let reject_reaction = promise::create_promise_reaction(
        cx.agent(),
        PromiseReactionKind::Reject,
        on_rejected,
        capability,
    );
    let record = cx
        .agent()
        .promise_record(promise_object)
        .cloned()
        .ok_or_else(|| type_error(cx))?;
    let _ = cx.agent().set_promise_handled(promise_object, true);
    match record.state() {
        PromiseState::Pending => {
            let _ = cx.agent().push_promise_reaction(
                promise_object,
                PromiseReactionKind::Fulfill,
                fulfill_reaction,
            );
            let _ = cx.agent().push_promise_reaction(
                promise_object,
                PromiseReactionKind::Reject,
                reject_reaction,
            );
        }
        PromiseState::Fulfilled => {
            enqueue_promise_reaction_job(
                cx.agent(),
                record.realm(),
                fulfill_reaction,
                record.result(),
            );
        }
        PromiseState::Rejected => {
            enqueue_promise_reaction_job(
                cx.agent(),
                record.realm(),
                reject_reaction,
                record.result(),
            );
        }
    }
    Ok(())
}

fn perform_promise_then<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    promise_object: ObjectRef,
    on_fulfilled: PromiseReactionHandler,
    on_rejected: PromiseReactionHandler,
) -> Result<Value, Cx::Error> {
    let constructor = promise_species_constructor(cx, promise_object)?;
    let capability = new_promise_capability(cx, constructor)?;
    perform_promise_then_with_capability(
        cx,
        promise_object,
        on_fulfilled,
        on_rejected,
        Some(capability),
    )?;
    Ok(Value::from_object_ref(promise_capability_promise(
        cx, capability,
    )?))
}

fn enqueue_promise_reaction_job(
    agent: &mut Agent,
    realm: RealmRef,
    reaction: lyng_js_env::PromiseReactionId,
    argument: Value,
) {
    let _ = agent.enqueue_job_with_payload(
        lyng_js_host::HostJobKind::Promise,
        lyng_js_env::ExecutableId::Builtin,
        lyng_js_env::RuntimeJobPayload::PromiseReaction { reaction, argument },
        Some(realm),
        Some("PromiseReaction".into()),
    );
}

fn promise_resolving_function_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    expected_kind: PromiseResolvingFunctionKind,
) -> Result<Value, Cx::Error> {
    let function = cx.callee_object();
    let record = cx
        .agent()
        .promise_resolving_function(function)
        .ok_or_else(|| type_error(cx))?;
    if record.kind() != expected_kind {
        return Err(type_error(cx));
    }
    let capability = record.capability();
    if cx
        .agent()
        .promise_capability(capability)
        .is_some_and(lyng_js_env::PromiseCapabilityRecord::already_resolved)
    {
        return Ok(Value::undefined());
    }
    let _ = cx
        .agent()
        .set_promise_capability_already_resolved(capability, true);
    let promise_object = promise_capability_promise(cx, capability)?;
    let resolution = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if expected_kind == PromiseResolvingFunctionKind::Reject {
        promise::reject_promise(cx.agent(), promise_object, resolution)
            .map_err(|abrupt| cx.abrupt(abrupt))?;
        return Ok(Value::undefined());
    }
    if resolution.as_object_ref() == Some(promise_object) {
        let reason = errors::throw_type_error(cx.agent())
            .thrown_value()
            .unwrap_or(Value::undefined());
        promise::reject_promise(cx.agent(), promise_object, reason)
            .map_err(|abrupt| cx.abrupt(abrupt))?;
        return Ok(Value::undefined());
    }
    let Some(thenable) = resolution.as_object_ref() else {
        promise::fulfill_promise(cx.agent(), promise_object, resolution)
            .map_err(|abrupt| cx.abrupt(abrupt))?;
        return Ok(Value::undefined());
    };
    let then_key = PropertyKey::from_atom(cx.agent().atoms_mut().intern_collectible("then"));
    let then = match cx.get_property_value(Value::from_object_ref(thenable), then_key) {
        Ok(then) => then,
        Err(error) => {
            if let Some(thrown) = cx.extract_thrown_value(error)? {
                promise::reject_promise(cx.agent(), promise_object, thrown)
                    .map_err(|abrupt| cx.abrupt(abrupt))?;
                return Ok(Value::undefined());
            }
            unreachable!("non-abrupt builtin error should propagate")
        }
    };
    let Some(then) = then
        .as_object_ref()
        .filter(|object| cx.agent().objects().is_callable(*object))
    else {
        promise::fulfill_promise(cx.agent(), promise_object, resolution)
            .map_err(|abrupt| cx.abrupt(abrupt))?;
        return Ok(Value::undefined());
    };
    let fallback_realm = cx.builtin_realm();
    let realm = cx
        .agent()
        .promise_record(promise_object)
        .map_or(fallback_realm, lyng_js_env::PromiseRecord::realm);
    promise::enqueue_thenable_job(cx.agent(), realm, promise_object, thenable, then);
    Ok(Value::undefined())
}

fn promise_finally_function_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let function = cx.callee_object();
    let record = cx
        .agent()
        .promise_finally_function(function)
        .ok_or_else(|| type_error(cx))?;
    let argument = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if record.kind() == PromiseFinallyFunctionKind::ValueThunk {
        return Ok(record.argument());
    }
    if record.kind() == PromiseFinallyFunctionKind::Thrower {
        return Err(cx.abrupt(AbruptCompletion::throw(record.argument())));
    }
    let on_finally = record.on_finally().ok_or_else(|| type_error(cx))?;
    let constructor = record.constructor().ok_or_else(|| type_error(cx))?;
    let result = cx.call_to_completion(on_finally, Value::undefined(), &[])?;
    let resolve = promise_resolve_method(cx, constructor)?;
    let promise = cx.call_to_completion(resolve, Value::from_object_ref(constructor), &[result])?;
    let promise_object = promise
        .as_object_ref()
        .filter(|object| cx.agent().promise_record(*object).is_some())
        .ok_or_else(|| type_error(cx))?;
    let continuation_kind = match record.kind() {
        PromiseFinallyFunctionKind::Then => PromiseFinallyFunctionKind::ValueThunk,
        PromiseFinallyFunctionKind::Catch => PromiseFinallyFunctionKind::Thrower,
        PromiseFinallyFunctionKind::ValueThunk | PromiseFinallyFunctionKind::Thrower => {
            unreachable!("continuation kinds returned above")
        }
    };
    let continuation = allocate_promise_finally_continuation(cx, continuation_kind, argument)?;
    invoke_then_method_with_args(
        cx,
        Value::from_object_ref(promise_object),
        &[Value::from_object_ref(continuation)],
    )
}

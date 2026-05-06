use super::{
    error_objects,
    promises::{
        invoke_then_method, new_promise_capability, promise_capability_promise,
        promise_capability_reject, promise_capability_resolve, promise_default_constructor,
        promise_resolve_method,
    },
    reference_error, type_error, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_env::{DisposalCapabilityKind, RealmRecord};
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::ObjectAllocation;
use lyng_js_ops::errors;
use lyng_js_types::{
    AbruptCompletion, BuiltinFunctionId, ObjectRef, PropertyKey, RealmRef, Value, WellKnownSymbolId,
};

pub(super) fn dispatch_disposal_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_disposal_stack_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_async_disposal_stack_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_disposal_scope_builtin(context, entry, invocation)
}

fn dispatch_disposal_stack_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::disposable_stack_builtin() {
        return disposable_stack_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::disposable_stack_use_builtin() {
        return disposable_stack_use_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::disposable_stack_adopt_builtin() {
        return disposable_stack_adopt_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::disposable_stack_defer_builtin() {
        return disposable_stack_defer_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::disposable_stack_move_builtin() {
        return disposable_stack_move_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::disposable_stack_disposed_getter_builtin() {
        return disposable_stack_disposed_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::disposable_stack_dispose_builtin() {
        return disposal_stack_dispose_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_async_disposal_stack_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::async_disposable_stack_builtin() {
        return async_disposable_stack_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::async_disposable_stack_use_builtin() {
        return async_disposable_stack_use_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::async_disposable_stack_adopt_builtin() {
        return async_disposable_stack_adopt_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::async_disposable_stack_defer_builtin() {
        return async_disposable_stack_defer_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::async_disposable_stack_move_builtin() {
        return async_disposable_stack_move_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::async_disposable_stack_disposed_getter_builtin() {
        return async_disposable_stack_disposed_getter_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::async_disposable_stack_dispose_async_builtin() {
        return async_disposable_stack_dispose_async_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::async_disposal_resume_builtin() {
        return async_disposal_resume_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_disposal_scope_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == lyng_js_types::create_sync_disposal_scope_builtin() {
        return create_disposal_scope_builtin(context, DisposalCapabilityKind::Sync).map(Some);
    }
    if entry == lyng_js_types::create_async_disposal_scope_builtin() {
        return create_disposal_scope_builtin(context, DisposalCapabilityKind::Async).map(Some);
    }
    if entry == lyng_js_types::add_sync_disposable_resource_builtin() {
        return add_disposal_scope_resource_builtin(context, invocation, false).map(Some);
    }
    if entry == lyng_js_types::add_async_disposable_resource_builtin() {
        return add_disposal_scope_resource_builtin(context, invocation, true).map(Some);
    }
    if entry == lyng_js_types::dispose_scope_builtin() {
        return dispose_scope_builtin(context, invocation).map(Some);
    }
    if entry == lyng_js_types::dispose_scope_async_builtin() {
        return dispose_scope_async_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn disposal_capability_payload_value(id: lyng_js_env::DisposalCapabilityId) -> Value {
    i32::try_from(id.get()).map_or_else(|_| Value::from_f64(f64::from(id.get())), Value::from_smi)
}

fn disposal_stack_default_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<ObjectRef, Cx::Error> {
    let intrinsics = cx
        .agent()
        .realm(realm)
        .map(lyng_js_env::RealmRecord::intrinsics);
    let prototype = match kind {
        lyng_js_env::DisposalCapabilityKind::Sync => {
            intrinsics.and_then(lyng_js_env::Intrinsics::disposable_stack_prototype)
        }
        lyng_js_env::DisposalCapabilityKind::Async => {
            intrinsics.and_then(lyng_js_env::Intrinsics::async_disposable_stack_prototype)
        }
    };
    prototype.ok_or_else(|| type_error(cx))
}

fn create_disposal_stack_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: ObjectRef,
    capability: lyng_js_env::DisposalCapabilityId,
) -> Result<ObjectRef, Cx::Error> {
    let root_shape = cx
        .agent()
        .realm(realm)
        .and_then(RealmRecord::root_shape)
        .ok_or_else(|| type_error(cx))?;
    let payload = disposal_capability_payload_value(capability);
    let object = cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_prototype(Some(prototype))
                .with_ordinary_payload_value(payload),
            AllocationLifetime::Default,
        )
    });
    let _ = cx
        .agent()
        .bind_disposal_capability_object(object, capability);
    Ok(object)
}

fn create_disposal_scope_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    capability: lyng_js_env::DisposalCapabilityId,
) -> Result<ObjectRef, Cx::Error> {
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().object_prototype())
        .ok_or_else(|| type_error(cx))?;
    create_disposal_stack_object(cx, realm, prototype, capability)
}

fn require_disposal_stack_receiver<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<
    (
        ObjectRef,
        lyng_js_env::DisposalCapabilityId,
        lyng_js_env::DisposalCapabilityState,
    ),
    Cx::Error,
> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let capability = cx
        .agent()
        .disposal_capability_id_for_object(object)
        .ok_or_else(|| type_error(cx))?;
    let Some(record) = cx.agent().disposal_capability(capability) else {
        return Err(type_error(cx));
    };
    if record.kind() != kind {
        return Err(type_error(cx));
    }
    Ok((object, capability, record.state()))
}

fn require_pending_disposal_stack<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<(ObjectRef, lyng_js_env::DisposalCapabilityId), Cx::Error> {
    let (object, capability, state) = require_disposal_stack_receiver(cx, value, kind)?;
    if matches!(state, lyng_js_env::DisposalCapabilityState::Disposed) {
        return Err(reference_error(cx));
    }
    Ok((object, capability))
}

fn require_disposal_scope_receiver<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<
    (
        ObjectRef,
        lyng_js_env::DisposalCapabilityId,
        lyng_js_env::DisposalCapabilityRecord,
    ),
    Cx::Error,
> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let capability = cx
        .agent()
        .disposal_capability_id_for_object(object)
        .ok_or_else(|| type_error(cx))?;
    let Some(record) = cx.agent().disposal_capability(capability).cloned() else {
        return Err(type_error(cx));
    };
    Ok((object, capability, record))
}

fn require_pending_disposal_scope<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<
    (
        ObjectRef,
        lyng_js_env::DisposalCapabilityId,
        lyng_js_env::DisposalCapabilityRecord,
    ),
    Cx::Error,
> {
    let (object, capability, record) = require_disposal_scope_receiver(cx, value)?;
    if record.is_disposed() {
        return Err(reference_error(cx));
    }
    Ok((object, capability, record))
}

fn dispose_method_for_hint<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    async_hint: bool,
) -> Result<Option<(ObjectRef, lyng_js_env::DisposalMethodKind)>, Cx::Error> {
    if value.is_undefined() || value.is_null() {
        return Ok(None);
    }
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if async_hint {
        let async_dispose = cx
            .agent()
            .well_known_symbol(WellKnownSymbolId::AsyncDispose)
            .ok_or_else(|| type_error(cx))?;
        let method = cx.get_property_value(
            Value::from_object_ref(object),
            PropertyKey::from_symbol(async_dispose),
        )?;
        if !(method.is_undefined() || method.is_null()) {
            return Ok(Some((
                cx.require_callable_object(method)?,
                lyng_js_env::DisposalMethodKind::Async,
            )));
        }
    }
    let dispose = cx
        .agent()
        .well_known_symbol(WellKnownSymbolId::Dispose)
        .ok_or_else(|| type_error(cx))?;
    let method = cx.get_property_value(
        Value::from_object_ref(object),
        PropertyKey::from_symbol(dispose),
    )?;
    if method.is_undefined() || method.is_null() {
        return Err(type_error(cx));
    }
    Ok(Some((
        cx.require_callable_object(method)?,
        if async_hint {
            lyng_js_env::DisposalMethodKind::AsyncFromSync
        } else {
            lyng_js_env::DisposalMethodKind::Sync
        },
    )))
}

fn dispose_method_for_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<Option<(ObjectRef, lyng_js_env::DisposalMethodKind)>, Cx::Error> {
    dispose_method_for_hint(
        cx,
        value,
        matches!(kind, lyng_js_env::DisposalCapabilityKind::Async),
    )
}

fn append_disposal_error<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    existing: Option<Value>,
    new_error: Value,
) -> Result<Value, Cx::Error> {
    let Some(existing) = existing else {
        return Ok(new_error);
    };
    let error = error_objects::create_suppressed_error_from_values(cx, new_error, existing, None)?;
    Ok(Value::from_object_ref(error))
}

fn call_disposal_resource<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    resource: lyng_js_env::DisposableResourceRecord,
) -> Result<Value, Cx::Error> {
    match resource.kind() {
        lyng_js_env::DisposableResourceKind::NoMethod => Ok(Value::undefined()),
        lyng_js_env::DisposableResourceKind::UseMethod => {
            let callable = resource.callable().ok_or_else(|| type_error(cx))?;
            cx.call_to_completion(callable, resource.value(), &[])
        }
        lyng_js_env::DisposableResourceKind::CallbackWithValue => {
            let callable = resource.callable().ok_or_else(|| type_error(cx))?;
            cx.call_to_completion(callable, Value::undefined(), &[resource.value()])
        }
        lyng_js_env::DisposableResourceKind::CallbackWithoutValue => {
            let callable = resource.callable().ok_or_else(|| type_error(cx))?;
            cx.call_to_completion(callable, Value::undefined(), &[])
        }
    }
}

fn promise_for_async_disposal_result<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let constructor = promise_default_constructor(cx)?;
    let resolve = promise_resolve_method(cx, constructor)?;
    let promise = cx.call_to_completion(resolve, Value::from_object_ref(constructor), &[value])?;
    promise
        .as_object_ref()
        .filter(|object| cx.agent().promise_record(*object).is_some())
        .ok_or_else(|| type_error(cx))
}

fn allocate_async_disposal_resume_function<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    operation: lyng_js_env::AsyncDisposalOperationId,
    reject: bool,
) -> Result<ObjectRef, Cx::Error> {
    let function = cx.allocate_builtin_function(lyng_js_types::async_disposal_resume_builtin())?;
    let _ = cx.agent().alloc_async_disposal_resume(
        function,
        lyng_js_env::AsyncDisposalResumeRecord::new(operation, reject),
    );
    Ok(function)
}

fn continue_async_disposal<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    operation_id: lyng_js_env::AsyncDisposalOperationId,
) -> Result<(), Cx::Error> {
    loop {
        let operation = cx
            .agent()
            .async_disposal_operation(operation_id)
            .ok_or_else(|| type_error(cx))?;
        if operation.completed() {
            return Ok(());
        }
        let capability = operation.capability();
        let Some(resource) = cx.agent().pop_disposal_resource(capability) else {
            let _ = cx
                .agent()
                .set_async_disposal_operation_completed(operation_id, true);
            if operation.has_disposal_error() {
                let error = operation
                    .pending_error()
                    .expect("disposal failures should seed a pending error");
                let reject = promise_capability_reject(cx, operation.promise_capability())?;
                let _ = cx.call_to_completion(reject, Value::undefined(), &[error])?;
            } else {
                let resolve = promise_capability_resolve(cx, operation.promise_capability())?;
                let _ =
                    cx.call_to_completion(resolve, Value::undefined(), &[Value::undefined()])?;
            }
            return Ok(());
        };

        let method_kind = resource.method_kind();
        let result = match call_disposal_resource(cx, resource) {
            Ok(result) => result,
            Err(error) => {
                let Some(thrown) = cx.extract_thrown_value(error)? else {
                    unreachable!("non-abrupt builtin error should propagate")
                };
                let pending = append_disposal_error(cx, operation.pending_error(), thrown)?;
                let _ = cx
                    .agent()
                    .set_async_disposal_operation_pending_error(operation_id, Some(pending));
                let _ = cx
                    .agent()
                    .set_async_disposal_operation_has_disposal_error(operation_id, true);
                continue;
            }
        };
        if matches!(method_kind, lyng_js_env::DisposalMethodKind::AsyncFromSync) {
            continue;
        }

        let promise = match promise_for_async_disposal_result(cx, result) {
            Ok(promise) => promise,
            Err(error) => {
                let Some(thrown) = cx.extract_thrown_value(error)? else {
                    unreachable!("non-abrupt builtin error should propagate")
                };
                let pending = append_disposal_error(cx, operation.pending_error(), thrown)?;
                let _ = cx
                    .agent()
                    .set_async_disposal_operation_pending_error(operation_id, Some(pending));
                let _ = cx
                    .agent()
                    .set_async_disposal_operation_has_disposal_error(operation_id, true);
                continue;
            }
        };

        let on_fulfilled = allocate_async_disposal_resume_function(cx, operation_id, false)?;
        let on_rejected = allocate_async_disposal_resume_function(cx, operation_id, true)?;
        let _ = cx
            .agent()
            .set_async_disposal_operation_waiting(operation_id, true);
        match invoke_then_method(
            cx,
            Value::from_object_ref(promise),
            Value::from_object_ref(on_fulfilled),
            Value::from_object_ref(on_rejected),
        ) {
            Ok(_) => return Ok(()),
            Err(error) => {
                let _ = cx
                    .agent()
                    .set_async_disposal_operation_waiting(operation_id, false);
                let Some(thrown) = cx.extract_thrown_value(error)? else {
                    unreachable!("non-abrupt builtin error should propagate")
                };
                let pending = append_disposal_error(cx, operation.pending_error(), thrown)?;
                let _ = cx
                    .agent()
                    .set_async_disposal_operation_pending_error(operation_id, Some(pending));
                let _ = cx
                    .agent()
                    .set_async_disposal_operation_has_disposal_error(operation_id, true);
            }
        }
    }
}

fn create_disposal_scope_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let capability = cx.agent().alloc_disposal_capability(kind);
    let object = create_disposal_scope_object(cx, realm, capability)?;
    Ok(Value::from_object_ref(object))
}

fn add_disposal_scope_resource_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    async_hint: bool,
) -> Result<Value, Cx::Error> {
    let scope = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let (_, capability, record) = require_pending_disposal_scope(cx, scope)?;
    if async_hint && record.kind() != lyng_js_env::DisposalCapabilityKind::Async {
        return Err(type_error(cx));
    }
    if async_hint && (value.is_undefined() || value.is_null()) {
        let _ = cx.agent().push_disposal_resource(
            capability,
            lyng_js_env::DisposableResourceRecord::no_method(
                lyng_js_env::DisposalMethodKind::Async,
            ),
        );
        return Ok(value);
    }
    let Some((callable, method_kind)) = dispose_method_for_hint(cx, value, async_hint)? else {
        return Ok(value);
    };
    let _ = cx.agent().push_disposal_resource(
        capability,
        lyng_js_env::DisposableResourceRecord::use_method(value, callable, method_kind),
    );
    Ok(value)
}

fn dispose_scope_capability<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    capability: lyng_js_env::DisposalCapabilityId,
    prior_error: Option<Value>,
) -> Result<Value, Cx::Error> {
    let _ = cx
        .agent()
        .set_disposal_capability_state(capability, lyng_js_env::DisposalCapabilityState::Disposed);
    let mut pending_error = prior_error;
    let mut saw_disposal_error = false;
    while let Some(resource) = cx.agent().pop_disposal_resource(capability) {
        match call_disposal_resource(cx, resource) {
            Ok(_) => {}
            Err(error) => {
                let Some(thrown) = cx.extract_thrown_value(error)? else {
                    unreachable!("non-abrupt builtin error should propagate")
                };
                saw_disposal_error = true;
                pending_error = Some(append_disposal_error(cx, pending_error, thrown)?);
            }
        }
    }
    if saw_disposal_error {
        let thrown = pending_error.expect("disposal errors should seed a pending error");
        return Err(cx.abrupt(AbruptCompletion::throw(thrown)));
    }
    Ok(Value::undefined())
}

fn dispose_scope_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let scope = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let (_, capability, record) = require_disposal_scope_receiver(cx, scope)?;
    if record.is_disposed() {
        return Ok(Value::undefined());
    }
    if record.kind() != lyng_js_env::DisposalCapabilityKind::Sync {
        return Err(type_error(cx));
    }
    dispose_scope_capability(cx, capability, invocation.arguments().get(1).copied())
}

fn dispose_scope_async_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let scope = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Ok((_, capability, record)) = require_disposal_scope_receiver(cx, scope) else {
        let promise_constructor = promise_default_constructor(cx)?;
        let promise_capability = new_promise_capability(cx, promise_constructor)?;
        let promise = promise_capability_promise(cx, promise_capability)?;
        let reject = promise_capability_reject(cx, promise_capability)?;
        let reason = errors::throw_type_error(cx.agent())
            .thrown_value()
            .unwrap_or(Value::undefined());
        let _ = cx.call_to_completion(reject, Value::undefined(), &[reason])?;
        return Ok(Value::from_object_ref(promise));
    };
    if record.is_disposed() {
        return Ok(Value::undefined());
    }
    if record.resources().is_empty() {
        let _ = cx.agent().set_disposal_capability_state(
            capability,
            lyng_js_env::DisposalCapabilityState::Disposed,
        );
        return Ok(Value::undefined());
    }
    let promise_constructor = promise_default_constructor(cx)?;
    let promise_capability = new_promise_capability(cx, promise_constructor)?;
    let promise = promise_capability_promise(cx, promise_capability)?;
    let _ = cx
        .agent()
        .set_disposal_capability_state(capability, lyng_js_env::DisposalCapabilityState::Disposed);
    let operation = cx
        .agent()
        .alloc_async_disposal_operation(capability, promise_capability);
    let prior_error = invocation.arguments().get(1).copied();
    let _ = cx
        .agent()
        .set_async_disposal_operation_pending_error(operation, prior_error);
    let _ = cx
        .agent()
        .set_async_disposal_operation_has_disposal_error(operation, false);
    continue_async_disposal(cx, operation)?;
    Ok(Value::from_object_ref(promise))
}

fn disposal_stack_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let default_prototype = disposal_stack_default_prototype(cx, realm, kind)?;
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let capability = cx.agent().alloc_disposal_capability(kind);
    let object = create_disposal_stack_object(cx, realm, prototype, capability)?;
    Ok(Value::from_object_ref(object))
}

fn disposable_stack_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_constructor_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Sync)
}

fn async_disposable_stack_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_constructor_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Async)
}

fn disposable_stack_disposed_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (_, capability, state) = require_disposal_stack_receiver(
        cx,
        invocation.this_value(),
        lyng_js_env::DisposalCapabilityKind::Sync,
    )?;
    let disposed = matches!(state, lyng_js_env::DisposalCapabilityState::Disposed)
        && cx.agent().disposal_capability(capability).is_some();
    Ok(Value::from_bool(disposed))
}

fn async_disposable_stack_disposed_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (_, capability, state) = require_disposal_stack_receiver(
        cx,
        invocation.this_value(),
        lyng_js_env::DisposalCapabilityKind::Async,
    )?;
    let disposed = matches!(state, lyng_js_env::DisposalCapabilityState::Disposed)
        && cx.agent().disposal_capability(capability).is_some();
    Ok(Value::from_bool(disposed))
}

fn disposal_stack_use_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<Value, Cx::Error> {
    let (_, capability) = require_pending_disposal_stack(cx, invocation.this_value(), kind)?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if matches!(kind, lyng_js_env::DisposalCapabilityKind::Async)
        && (value.is_undefined() || value.is_null())
    {
        let _ = cx.agent().push_disposal_resource(
            capability,
            lyng_js_env::DisposableResourceRecord::no_method(
                lyng_js_env::DisposalMethodKind::Async,
            ),
        );
        return Ok(value);
    }
    let Some((callable, method_kind)) = dispose_method_for_value(cx, value, kind)? else {
        return Ok(value);
    };
    let _ = cx.agent().push_disposal_resource(
        capability,
        lyng_js_env::DisposableResourceRecord::use_method(value, callable, method_kind),
    );
    Ok(value)
}

fn disposable_stack_use_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_use_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Sync)
}

fn async_disposable_stack_use_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_use_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Async)
}

fn disposal_stack_adopt_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<Value, Cx::Error> {
    let (_, capability) = require_pending_disposal_stack(cx, invocation.this_value(), kind)?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let on_dispose = cx.require_callable_object(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let method_kind = match kind {
        lyng_js_env::DisposalCapabilityKind::Sync => lyng_js_env::DisposalMethodKind::Sync,
        lyng_js_env::DisposalCapabilityKind::Async => lyng_js_env::DisposalMethodKind::Async,
    };
    let _ = cx.agent().push_disposal_resource(
        capability,
        lyng_js_env::DisposableResourceRecord::callback_with_value(value, on_dispose, method_kind),
    );
    Ok(value)
}

fn disposable_stack_adopt_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_adopt_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Sync)
}

fn async_disposable_stack_adopt_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_adopt_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Async)
}

fn disposal_stack_defer_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<Value, Cx::Error> {
    let (_, capability) = require_pending_disposal_stack(cx, invocation.this_value(), kind)?;
    let on_dispose = cx.require_callable_object(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let method_kind = match kind {
        lyng_js_env::DisposalCapabilityKind::Sync => lyng_js_env::DisposalMethodKind::Sync,
        lyng_js_env::DisposalCapabilityKind::Async => lyng_js_env::DisposalMethodKind::Async,
    };
    let _ = cx.agent().push_disposal_resource(
        capability,
        lyng_js_env::DisposableResourceRecord::callback_without_value(on_dispose, method_kind),
    );
    Ok(Value::undefined())
}

fn disposable_stack_defer_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_defer_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Sync)
}

fn async_disposable_stack_defer_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_defer_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Async)
}

fn disposal_stack_move_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: lyng_js_env::DisposalCapabilityKind,
) -> Result<Value, Cx::Error> {
    let (_, capability) = require_pending_disposal_stack(cx, invocation.this_value(), kind)?;
    let realm = cx.builtin_realm();
    let prototype = disposal_stack_default_prototype(cx, realm, kind)?;
    let resources = cx
        .agent()
        .take_disposal_resources(capability)
        .ok_or_else(|| type_error(cx))?;
    let _ = cx
        .agent()
        .set_disposal_capability_state(capability, lyng_js_env::DisposalCapabilityState::Disposed);
    let new_capability = cx.agent().alloc_disposal_capability(kind);
    let _ = cx
        .agent()
        .replace_disposal_resources(new_capability, resources);
    let object = create_disposal_stack_object(cx, realm, prototype, new_capability)?;
    Ok(Value::from_object_ref(object))
}

fn disposable_stack_move_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_move_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Sync)
}

fn async_disposable_stack_move_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    disposal_stack_move_builtin(cx, invocation, lyng_js_env::DisposalCapabilityKind::Async)
}

fn disposal_stack_dispose_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (_, capability, state) = require_disposal_stack_receiver(
        cx,
        invocation.this_value(),
        lyng_js_env::DisposalCapabilityKind::Sync,
    )?;
    if matches!(state, lyng_js_env::DisposalCapabilityState::Disposed) {
        return Ok(Value::undefined());
    }
    let _ = cx
        .agent()
        .set_disposal_capability_state(capability, lyng_js_env::DisposalCapabilityState::Disposed);
    let mut pending_error = None;
    while let Some(resource) = cx.agent().pop_disposal_resource(capability) {
        match call_disposal_resource(cx, resource) {
            Ok(_) => {}
            Err(error) => {
                let Some(thrown) = cx.extract_thrown_value(error)? else {
                    unreachable!("non-abrupt builtin error should propagate")
                };
                pending_error = Some(append_disposal_error(cx, pending_error, thrown)?);
            }
        }
    }
    if let Some(thrown) = pending_error {
        return Err(cx.abrupt(AbruptCompletion::throw(thrown)));
    }
    Ok(Value::undefined())
}

fn async_disposable_stack_dispose_async_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let promise_constructor = promise_default_constructor(cx)?;
    let promise_capability = new_promise_capability(cx, promise_constructor)?;
    let promise = promise_capability_promise(cx, promise_capability)?;
    let Some(receiver) = invocation.this_value().as_object_ref() else {
        let reject = promise_capability_reject(cx, promise_capability)?;
        let reason = errors::throw_type_error(cx.agent())
            .thrown_value()
            .unwrap_or(Value::undefined());
        let _ = cx.call_to_completion(reject, Value::undefined(), &[reason])?;
        return Ok(Value::from_object_ref(promise));
    };
    let Some(capability) = cx.agent().disposal_capability_id_for_object(receiver) else {
        let reject = promise_capability_reject(cx, promise_capability)?;
        let reason = errors::throw_type_error(cx.agent())
            .thrown_value()
            .unwrap_or(Value::undefined());
        let _ = cx.call_to_completion(reject, Value::undefined(), &[reason])?;
        return Ok(Value::from_object_ref(promise));
    };
    let Some(record) = cx.agent().disposal_capability(capability) else {
        let reject = promise_capability_reject(cx, promise_capability)?;
        let reason = errors::throw_type_error(cx.agent())
            .thrown_value()
            .unwrap_or(Value::undefined());
        let _ = cx.call_to_completion(reject, Value::undefined(), &[reason])?;
        return Ok(Value::from_object_ref(promise));
    };
    if !matches!(record.kind(), lyng_js_env::DisposalCapabilityKind::Async) {
        let reject = promise_capability_reject(cx, promise_capability)?;
        let reason = errors::throw_type_error(cx.agent())
            .thrown_value()
            .unwrap_or(Value::undefined());
        let _ = cx.call_to_completion(reject, Value::undefined(), &[reason])?;
        return Ok(Value::from_object_ref(promise));
    }
    if record.is_disposed() {
        let resolve = promise_capability_resolve(cx, promise_capability)?;
        let _ = cx.call_to_completion(resolve, Value::undefined(), &[Value::undefined()])?;
        return Ok(Value::from_object_ref(promise));
    }
    let _ = cx
        .agent()
        .set_disposal_capability_state(capability, lyng_js_env::DisposalCapabilityState::Disposed);
    let operation = cx
        .agent()
        .alloc_async_disposal_operation(capability, promise_capability);
    continue_async_disposal(cx, operation)?;
    Ok(Value::from_object_ref(promise))
}

fn async_disposal_resume_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let callee = cx.callee_object();
    let Some(record) = cx.agent().async_disposal_resume(callee) else {
        return Err(type_error(cx));
    };
    let Some(operation) = cx.agent().async_disposal_operation(record.operation()) else {
        return Ok(Value::undefined());
    };
    if operation.completed() || !operation.waiting() {
        return Ok(Value::undefined());
    }
    let _ = cx
        .agent()
        .set_async_disposal_operation_waiting(record.operation(), false);
    if record.reject() {
        let argument = invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined());
        let pending = append_disposal_error(cx, operation.pending_error(), argument)?;
        let _ = cx
            .agent()
            .set_async_disposal_operation_pending_error(record.operation(), Some(pending));
        let _ = cx
            .agent()
            .set_async_disposal_operation_has_disposal_error(record.operation(), true);
    }
    continue_async_disposal(cx, record.operation())?;
    Ok(Value::undefined())
}

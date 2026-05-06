use super::{
    create_data_property_or_throw, length_value_u64, map_completion, promises,
    property_key_from_text, range_error, string_value, to_bigint_for_builtin, to_index_for_builtin,
    to_integer_or_infinity_for_builtin, to_number_for_builtin, type_error,
    typed_array_storage_bits_from_builtin_value, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_env::{
    AsyncWaiterRecord, ExecutableId, ParkedAgentRecord, RuntimeJobPayload, WaiterKind,
};
use lyng_js_host::{HostJobKind, ParkAgentRequest, ParkAgentStatus, UnparkAgentRequest};
use lyng_js_objects::{TypedArrayElementKind, TypedArrayObjectData};
use lyng_js_ops::{promise, shared_memory as shared_memory_ops};
use lyng_js_types::{BuiltinFunctionId, ObjectRef, Value};

pub(super) fn dispatch_atomics_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::super::atomics_load_builtin() {
        return atomics_load_builtin(context, invocation).map(Some);
    }
    if entry == super::super::atomics_store_builtin() {
        return atomics_store_builtin(context, invocation).map(Some);
    }
    if entry == super::super::atomics_add_builtin() {
        return atomics_add_builtin(context, invocation).map(Some);
    }
    if entry == super::super::atomics_sub_builtin() {
        return atomics_sub_builtin(context, invocation).map(Some);
    }
    if entry == super::super::atomics_and_builtin() {
        return atomics_and_builtin(context, invocation).map(Some);
    }
    if entry == super::super::atomics_or_builtin() {
        return atomics_or_builtin(context, invocation).map(Some);
    }
    if entry == super::super::atomics_xor_builtin() {
        return atomics_xor_builtin(context, invocation).map(Some);
    }
    if entry == super::super::atomics_exchange_builtin() {
        return atomics_exchange_builtin(context, invocation).map(Some);
    }
    if entry == super::super::atomics_compare_exchange_builtin() {
        return atomics_compare_exchange_builtin(context, invocation).map(Some);
    }
    if entry == super::super::atomics_notify_builtin() {
        return atomics_notify_builtin(context, invocation).map(Some);
    }
    if entry == super::super::atomics_wait_builtin() {
        return atomics_wait_builtin(context, invocation).map(Some);
    }
    if entry == super::super::atomics_wait_async_builtin() {
        return atomics_wait_async_builtin(context, invocation).map(Some);
    }
    if entry == super::super::atomics_pause_builtin() {
        return atomics_pause_builtin(context, invocation).map(Some);
    }
    if entry == super::super::atomics_is_lock_free_builtin() {
        return atomics_is_lock_free_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn atomics_pause_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if let Some(argument) = invocation.arguments().first().copied() {
        if !argument.is_undefined() {
            if !argument.is_number() {
                return Err(type_error(cx));
            }
            let number = argument.as_f64().ok_or_else(|| type_error(cx))?;
            if !number.is_finite() || number.fract() != 0.0 {
                return Err(type_error(cx));
            }
        }
    }
    Ok(Value::undefined())
}

fn atomics_typed_array<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    waitable: bool,
    require_shared: bool,
) -> Result<TypedArrayObjectData, Cx::Error> {
    let typed_array = invocation
        .arguments()
        .first()
        .and_then(|value| value.as_object_ref())
        .ok_or_else(|| type_error(cx))?;
    shared_memory_ops::validate_atomic_typed_array(
        cx.agent(),
        typed_array,
        waitable,
        require_shared,
    )
    .map_err(|error| match error {
        shared_memory_ops::AtomicAccessError::Type => type_error(cx),
        shared_memory_ops::AtomicAccessError::Range => range_error(cx),
    })
}

fn atomics_access_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    waitable: bool,
    require_shared: bool,
) -> Result<shared_memory_ops::AtomicAccessRecord, Cx::Error> {
    let typed_array = atomics_typed_array(cx, invocation, waitable, require_shared)?;
    let index = to_index_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let element_index = shared_memory_ops::validate_atomic_index(typed_array, index).map_err(
        |error| match error {
            shared_memory_ops::AtomicAccessError::Type => type_error(cx),
            shared_memory_ops::AtomicAccessError::Range => range_error(cx),
        },
    )?;
    Ok(shared_memory_ops::atomic_access_record(
        typed_array,
        element_index,
    ))
}

fn atomics_value_argument<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    record: shared_memory_ops::AtomicAccessRecord,
    value: Value,
) -> Result<u64, Cx::Error> {
    typed_array_storage_bits_from_builtin_value(cx, record.typed_array().kind(), value)
}

fn integer_storage_bits(integer: f64, width: u32) -> u64 {
    if integer == 0.0 || !integer.is_finite() {
        return 0;
    }
    let modulus = 2_f64.powi(i32::try_from(width).expect("integer width should fit"));
    let mut wrapped = integer % modulus;
    if wrapped < 0.0 {
        wrapped += modulus;
    }
    wrapped as u64
}

fn integer_value(integer: f64) -> Value {
    if integer.is_finite() && integer >= f64::from(i32::MIN) && integer <= f64::from(i32::MAX) {
        Value::from_smi(integer as i32)
    } else {
        Value::from_f64(integer)
    }
}

fn atomics_store_value_argument<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    record: shared_memory_ops::AtomicAccessRecord,
    value: Value,
) -> Result<(u64, Value), Cx::Error> {
    match record.typed_array().kind() {
        TypedArrayElementKind::BigInt64 | TypedArrayElementKind::BigUint64 => {
            let bigint = to_bigint_for_builtin(cx, value)?;
            let bits = typed_array_storage_bits_from_builtin_value(
                cx,
                record.typed_array().kind(),
                bigint,
            )?;
            Ok((bits, bigint))
        }
        TypedArrayElementKind::Int8 | TypedArrayElementKind::Uint8 => {
            let integer = to_integer_or_infinity_for_builtin(cx, value)?;
            Ok((integer_storage_bits(integer, 8), integer_value(integer)))
        }
        TypedArrayElementKind::Int16 | TypedArrayElementKind::Uint16 => {
            let integer = to_integer_or_infinity_for_builtin(cx, value)?;
            Ok((integer_storage_bits(integer, 16), integer_value(integer)))
        }
        TypedArrayElementKind::Int32 | TypedArrayElementKind::Uint32 => {
            let integer = to_integer_or_infinity_for_builtin(cx, value)?;
            Ok((integer_storage_bits(integer, 32), integer_value(integer)))
        }
        TypedArrayElementKind::Float32
        | TypedArrayElementKind::Float16
        | TypedArrayElementKind::Float64
        | TypedArrayElementKind::Uint8Clamped => Err(type_error(cx)),
    }
}

fn atomics_load_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = atomics_access_record(cx, invocation, false, false)?;
    let bits =
        shared_memory_ops::read_atomic_bits(cx.agent(), record).ok_or_else(|| type_error(cx))?;
    Ok(shared_memory_ops::atomic_value_from_bits(
        cx.agent(),
        record.typed_array().kind(),
        bits,
    ))
}

fn atomics_store_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = atomics_access_record(cx, invocation, false, false)?;
    let (bits, converted_value) = atomics_store_value_argument(
        cx,
        record,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let _ = shared_memory_ops::atomic_store_bits(cx.agent(), record, bits)
        .ok_or_else(|| type_error(cx))?;
    Ok(converted_value)
}

fn atomics_rmw_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    op: shared_memory_ops::AtomicRmwOp,
) -> Result<Value, Cx::Error> {
    let record = atomics_access_record(cx, invocation, false, false)?;
    let value = atomics_value_argument(
        cx,
        record,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let bits = shared_memory_ops::atomic_rmw_bits(cx.agent(), record, value, op)
        .ok_or_else(|| type_error(cx))?;
    Ok(shared_memory_ops::atomic_value_from_bits(
        cx.agent(),
        record.typed_array().kind(),
        bits,
    ))
}

fn atomics_add_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    atomics_rmw_builtin(cx, invocation, shared_memory_ops::AtomicRmwOp::Add)
}

fn atomics_sub_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    atomics_rmw_builtin(cx, invocation, shared_memory_ops::AtomicRmwOp::Sub)
}

fn atomics_and_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    atomics_rmw_builtin(cx, invocation, shared_memory_ops::AtomicRmwOp::And)
}

fn atomics_or_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    atomics_rmw_builtin(cx, invocation, shared_memory_ops::AtomicRmwOp::Or)
}

fn atomics_xor_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    atomics_rmw_builtin(cx, invocation, shared_memory_ops::AtomicRmwOp::Xor)
}

fn atomics_exchange_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    atomics_rmw_builtin(cx, invocation, shared_memory_ops::AtomicRmwOp::Exchange)
}

fn atomics_compare_exchange_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = atomics_access_record(cx, invocation, false, false)?;
    let expected = atomics_value_argument(
        cx,
        record,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let replacement = atomics_value_argument(
        cx,
        record,
        invocation
            .arguments()
            .get(3)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let bits =
        shared_memory_ops::atomic_compare_exchange_bits(cx.agent(), record, expected, replacement)
            .ok_or_else(|| type_error(cx))?;
    Ok(shared_memory_ops::atomic_value_from_bits(
        cx.agent(),
        record.typed_array().kind(),
        bits,
    ))
}

fn atomics_notify_count<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    count: Option<Value>,
) -> Result<u32, Cx::Error> {
    let Some(count) = count.filter(|value| !value.is_undefined()) else {
        return Ok(u32::MAX);
    };
    let integer = to_integer_or_infinity_for_builtin(cx, count)?;
    if !integer.is_finite() {
        return Ok(u32::MAX);
    }
    if integer <= 0.0 {
        return Ok(0);
    }
    Ok(integer.min(f64::from(u32::MAX)) as u32)
}

fn atomics_notify_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let typed_array = atomics_typed_array(cx, invocation, true, false)?;
    let index = to_index_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let element_index = shared_memory_ops::validate_atomic_index(typed_array, index).map_err(
        |error| match error {
            shared_memory_ops::AtomicAccessError::Type => type_error(cx),
            shared_memory_ops::AtomicAccessError::Range => range_error(cx),
        },
    )?;
    let record = shared_memory_ops::atomic_access_record(typed_array, element_index);
    let count = atomics_notify_count(cx, invocation.arguments().get(2).copied())?;
    if !cx
        .agent()
        .backing_store_is_shared(record.typed_array().backing_store())
        .unwrap_or(false)
    {
        return Ok(length_value_u64(0));
    }
    if count == 0 {
        return Ok(length_value_u64(0));
    }
    let location = shared_memory_ops::wait_location(record);
    let waiters = cx.agent().wake_shared_memory_waiters(location, count);
    let mut blocking_count = 0_u32;
    for waiter in &waiters {
        match waiter.kind() {
            WaiterKind::Blocking(_) => {
                blocking_count = blocking_count.saturating_add(1);
            }
            WaiterKind::Async(record) => {
                fulfill_wait_async_promise(cx, record.promise(), "ok")?;
            }
        }
    }
    if blocking_count > 0 {
        let _ = cx.unpark_agent(&UnparkAgentRequest {
            location,
            max_count: blocking_count,
        })?;
    }
    Ok(length_value_u64(
        u64::try_from(waiters.len()).unwrap_or(u64::MAX),
    ))
}

fn atomics_wait_timeout_ns<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    timeout: Option<Value>,
) -> Result<Option<u64>, Cx::Error> {
    let Some(timeout) = timeout.filter(|value| !value.is_undefined()) else {
        return Ok(None);
    };
    let timeout_ms = to_number_for_builtin(cx, timeout)?;
    if timeout_ms.is_nan() || timeout_ms.is_infinite() && timeout_ms.is_sign_positive() {
        return Ok(None);
    }
    if timeout_ms <= 0.0 || timeout_ms.is_sign_negative() {
        return Ok(Some(0));
    }
    let timeout_ns = (timeout_ms * 1_000_000.0).min(u64::MAX as f64);
    Ok(Some(timeout_ns as u64))
}

fn wait_async_result_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    is_async: bool,
    value: Value,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().object_prototype())
        .ok_or_else(|| type_error(cx))?;
    let object = cx.allocate_ordinary_object_with_prototype(realm, Some(prototype))?;
    let async_key = property_key_from_text(cx, "async");
    let value_key = property_key_from_text(cx, "value");
    create_data_property_or_throw(cx, object, async_key, Value::from_bool(is_async))?;
    create_data_property_or_throw(cx, object, value_key, value)?;
    Ok(Value::from_object_ref(object))
}

fn fulfill_wait_async_promise<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    promise_object: ObjectRef,
    result: &str,
) -> Result<(), Cx::Error> {
    let value = string_value(cx, result);
    let completion = promise::fulfill_promise(cx.agent(), promise_object, value);
    map_completion(cx, completion)
}

fn atomics_wait_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = atomics_access_record(cx, invocation, true, true)?;
    let expected = atomics_value_argument(
        cx,
        record,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let current =
        shared_memory_ops::read_atomic_bits(cx.agent(), record).ok_or_else(|| type_error(cx))?;
    if current != expected {
        return Ok(string_value(cx, "not-equal"));
    }
    let timeout_ns = atomics_wait_timeout_ns(cx, invocation.arguments().get(3).copied())?;
    if timeout_ns == Some(0) {
        return Ok(string_value(cx, "timed-out"));
    }
    let Some(host_id) = cx.agent().host_id() else {
        return if timeout_ns.is_some() {
            Ok(string_value(cx, "timed-out"))
        } else {
            Err(type_error(cx))
        };
    };
    let location = shared_memory_ops::wait_location(record);
    let agent_id = cx.agent().id();
    let thread_id = cx.agent().bound_thread();
    let token = cx
        .agent()
        .park_shared_memory_waiter(location, ParkedAgentRecord::new(agent_id, thread_id, false))
        .ok_or_else(|| type_error(cx))?;
    let result = cx.park_agent(&ParkAgentRequest {
        agent_id: host_id,
        thread_id,
        location,
        timeout_ns,
        allow_async: false,
    })?;
    let _ = cx.agent().remove_shared_memory_waiter(location, token);
    Ok(match result.status {
        ParkAgentStatus::Parked => string_value(cx, "ok"),
        ParkAgentStatus::TimedOut | ParkAgentStatus::Interrupted => string_value(cx, "timed-out"),
    })
}

fn atomics_wait_async_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = atomics_access_record(cx, invocation, true, true)?;
    let expected = atomics_value_argument(
        cx,
        record,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let current =
        shared_memory_ops::read_atomic_bits(cx.agent(), record).ok_or_else(|| type_error(cx))?;
    if current != expected {
        let value = string_value(cx, "not-equal");
        return wait_async_result_object(cx, false, value);
    }
    let timeout_ns = atomics_wait_timeout_ns(cx, invocation.arguments().get(3).copied())?;
    if timeout_ns == Some(0) {
        let value = string_value(cx, "timed-out");
        return wait_async_result_object(cx, false, value);
    }
    let promise_constructor = promises::promise_default_constructor(cx)?;
    let capability = promises::new_promise_capability(cx, promise_constructor)?;
    let promise_object = promises::promise_capability_promise(cx, capability)?;
    let location = shared_memory_ops::wait_location(record);
    let agent_id = cx.agent().id();
    let token = cx.agent().park_async_shared_memory_waiter(
        location,
        AsyncWaiterRecord::new(agent_id, promise_object),
    );
    if timeout_ns.is_some() {
        let realm = cx.builtin_realm();
        let _ = cx.agent().enqueue_job_with_payload(
            HostJobKind::Harness,
            ExecutableId::Builtin,
            RuntimeJobPayload::AtomicsWaitAsyncTimeout {
                location,
                token,
                promise: promise_object,
            },
            Some(realm),
            Some("AtomicsWaitAsyncTimeout".into()),
        );
    }
    wait_async_result_object(cx, true, Value::from_object_ref(promise_object))
}

fn atomics_is_lock_free_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let size = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let integer = to_integer_or_infinity_for_builtin(cx, size)?;
    if !integer.is_finite() || integer <= 0.0 {
        return Ok(Value::from_bool(false));
    }
    Ok(Value::from_bool(shared_memory_ops::atomics_is_lock_free(
        integer as u64,
    )))
}

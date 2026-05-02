use super::typed_array_indices::{
    typed_array_index_descriptor, typed_array_numeric_key, TypedArrayNumericKey,
};
use crate::errors::{internal_method_error, throw_type_error};
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::NativeFunctionRegistry;
use lyng_js_types::{Completion, ObjectRef, PropertyDescriptor, PropertyKey, Value};

/// Ordinary-only `HasProperty` over the object substrate.
///
/// This helper bypasses proxy traps by going directly through `ObjectRuntime`
/// internal methods. Guest-observable code should use
/// [`super::has_property_in_context`] unless the algorithm is explicitly
/// operating on an ordinary/bootstrap object.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn ordinary_has_property(
    agent: &mut Agent,
    object: ObjectRef,
    key: PropertyKey,
) -> Completion<bool> {
    if let Some(numeric_key) = typed_array_numeric_key(agent, object, key) {
        return Ok(matches!(numeric_key, TypedArrayNumericKey::Valid(_)));
    }
    agent
        .objects()
        .has_property(agent.heap().view(), object, key)
        .map_err(|error| internal_method_error(agent, error))
}

/// Ordinary-only `Get` over the object substrate.
///
/// This helper bypasses proxy traps by going directly through `ObjectRuntime`
/// internal methods. Guest-observable code should use [`super::get_in_context`]
/// unless the algorithm is explicitly operating on an ordinary/bootstrap object.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn ordinary_get(agent: &mut Agent, object: ObjectRef, key: PropertyKey) -> Completion<Value> {
    ordinary_get_with_receiver(agent, object, key, Value::from_object_ref(object))
}

/// Ordinary-only `Get` over the object substrate with an explicit receiver.
///
/// This helper bypasses proxy traps by going directly through `ObjectRuntime`
/// internal methods. Guest-observable code should use
/// [`super::get_with_receiver_in_context`] unless the algorithm is explicitly
/// operating on an ordinary/bootstrap object.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn ordinary_get_with_receiver(
    agent: &mut Agent,
    object: ObjectRef,
    key: PropertyKey,
    receiver: Value,
) -> Completion<Value> {
    if let Some(numeric_key) = typed_array_numeric_key(agent, object, key) {
        return match numeric_key {
            TypedArrayNumericKey::Valid(index) => {
                Ok(typed_array_index_descriptor(agent, object, index)?
                    .and_then(|descriptor| descriptor.value())
                    .unwrap_or(Value::undefined()))
            }
            TypedArrayNumericKey::Invalid => Ok(Value::undefined()),
        };
    }
    agent
        .objects()
        .get(agent.heap().view(), object, key, receiver)
        .map_err(|error| internal_method_error(agent, error))
}

/// Resolves the `super` base object from one `[[HomeObject]]`.
///
/// # Errors
/// Returns an abrupt completion if the home object has a null prototype or the
/// underlying object internal methods fail.
pub fn super_base(agent: &mut Agent, home_object: ObjectRef) -> Completion<ObjectRef> {
    agent
        .objects()
        .get_prototype_of(agent.heap().view(), home_object)
        .map_err(|error| internal_method_error(agent, error))?
        .ok_or_else(|| throw_type_error(agent))
}

/// ECMAScript `GetSuper`-style helper using a pre-resolved home object and receiver.
///
/// # Errors
/// Returns an abrupt completion if the home object has a null prototype or the
/// underlying object internal methods fail.
pub fn super_get(
    agent: &mut Agent,
    home_object: ObjectRef,
    receiver: Value,
    key: PropertyKey,
) -> Completion<Value> {
    let base = super_base(agent, home_object)?;
    ordinary_get_with_receiver(agent, base, key, receiver)
}

/// Ordinary-only `[[GetPrototypeOf]]` over the object substrate.
///
/// This helper bypasses proxy traps by going directly through `ObjectRuntime`
/// internal methods. Guest-observable code should use
/// [`super::get_prototype_of_in_context`] unless the algorithm is explicitly
/// operating on an ordinary/bootstrap object.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn ordinary_get_prototype_of(
    agent: &mut Agent,
    object: ObjectRef,
) -> Completion<Option<ObjectRef>> {
    agent
        .objects()
        .get_prototype_of(agent.heap().view(), object)
        .map_err(|error| internal_method_error(agent, error))
}

/// Ordinary-only `[[SetPrototypeOf]]` over the object substrate.
///
/// This helper bypasses proxy traps by going directly through `ObjectRuntime`
/// internal methods. Guest-observable code should use
/// [`super::set_prototype_of_in_context`] unless the algorithm is explicitly
/// operating on an ordinary/bootstrap object.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn ordinary_set_prototype_of(
    agent: &mut Agent,
    object: ObjectRef,
    prototype: Option<ObjectRef>,
) -> Completion<bool> {
    let result = agent.with_heap_and_objects(|heap, objects| {
        objects.set_prototype_of(&mut heap.mutator(), object, prototype)
    });
    result.map_err(|error| internal_method_error(agent, error))
}

/// Ordinary-only `[[GetOwnProperty]]` over the object substrate.
///
/// This helper bypasses proxy traps by going directly through `ObjectRuntime`
/// internal methods. Guest-observable code should use
/// [`super::get_own_property_in_context`] unless the algorithm is explicitly
/// operating on an ordinary/bootstrap object.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn ordinary_get_own_property(
    agent: &mut Agent,
    object: ObjectRef,
    key: PropertyKey,
) -> Completion<Option<PropertyDescriptor>> {
    if let Some(numeric_key) = typed_array_numeric_key(agent, object, key) {
        return match numeric_key {
            TypedArrayNumericKey::Valid(index) => {
                typed_array_index_descriptor(agent, object, index)
            }
            TypedArrayNumericKey::Invalid => Ok(None),
        };
    }
    agent
        .objects()
        .get_own_property(agent.heap().view(), object, key)
        .map_err(|error| internal_method_error(agent, error))
}

/// Ordinary-only `[[DefineOwnProperty]]` over the object substrate.
///
/// This helper bypasses proxy traps by going directly through `ObjectRuntime`
/// internal methods. Guest-observable code should use
/// [`super::define_property_in_context`] unless the algorithm is explicitly
/// operating on an ordinary/bootstrap object.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn ordinary_define_property(
    agent: &mut Agent,
    object: ObjectRef,
    key: PropertyKey,
    descriptor: PropertyDescriptor,
    lifetime: AllocationLifetime,
) -> Completion<bool> {
    if let Some(numeric_key) = typed_array_numeric_key(agent, object, key) {
        let TypedArrayNumericKey::Valid(index) = numeric_key else {
            return Ok(false);
        };
        let result = agent.with_heap_and_objects(|heap, objects| {
            objects.define_own_property(
                &mut heap.mutator(),
                object,
                PropertyKey::Index(index),
                descriptor,
                lifetime,
            )
        });
        return result.map_err(|error| internal_method_error(agent, error));
    }
    let result = agent.with_heap_and_objects(|heap, objects| {
        objects.define_own_property(&mut heap.mutator(), object, key, descriptor, lifetime)
    });
    result.map_err(|error| internal_method_error(agent, error))
}

/// Ordinary-only `[[IsExtensible]]` over the object substrate.
///
/// This helper bypasses proxy traps by going directly through `ObjectRuntime`
/// internal methods. Guest-observable code should use `proxy::is_extensible`
/// with an object-operation context unless the algorithm is explicitly
/// operating on an ordinary/bootstrap object.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn ordinary_is_extensible(agent: &mut Agent, object: ObjectRef) -> Completion<bool> {
    agent
        .objects()
        .is_extensible(object)
        .map_err(|error| internal_method_error(agent, error))
}

/// Ordinary-only `[[PreventExtensions]]` over the object substrate.
///
/// This helper bypasses proxy traps by going directly through `ObjectRuntime`
/// internal methods. Guest-observable code should use `proxy::prevent_extensions`
/// with an object-operation context unless the algorithm is explicitly
/// operating on an ordinary/bootstrap object.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn ordinary_prevent_extensions(agent: &mut Agent, object: ObjectRef) -> Completion<bool> {
    let result = agent
        .with_heap_and_objects(|heap, objects| objects.prevent_extensions(heap.view(), object));
    result.map_err(|error| internal_method_error(agent, error))
}

/// Ordinary-only `[[OwnPropertyKeys]]` over the object substrate.
///
/// This helper bypasses proxy traps by going directly through `ObjectRuntime`
/// internal methods. Guest-observable code should use
/// [`super::own_property_keys_in_context`] unless the algorithm is explicitly
/// operating on an ordinary/bootstrap object.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn ordinary_own_property_keys(
    agent: &mut Agent,
    object: ObjectRef,
) -> Completion<Vec<PropertyKey>> {
    agent
        .objects()
        .own_property_keys(agent.heap().view(), object)
        .map_err(|error| internal_method_error(agent, error))
}

/// Ordinary-only `Set` over the object substrate.
///
/// This helper bypasses proxy traps by going directly through `ObjectRuntime`
/// internal methods. Guest-observable code should use [`super::set_in_context`]
/// unless the algorithm is explicitly operating on an ordinary/bootstrap object.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn ordinary_set(
    agent: &mut Agent,
    object: ObjectRef,
    key: PropertyKey,
    value: Value,
    lifetime: AllocationLifetime,
) -> Completion<bool> {
    ordinary_set_with_receiver(
        agent,
        object,
        key,
        value,
        Value::from_object_ref(object),
        lifetime,
    )
}

/// Ordinary-only `Set` over the object substrate with an explicit receiver.
///
/// This helper bypasses proxy traps by going directly through `ObjectRuntime`
/// internal methods. Guest-observable code should use
/// [`super::set_with_receiver_in_context`] unless the algorithm is explicitly
/// operating on an ordinary/bootstrap object.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn ordinary_set_with_receiver(
    agent: &mut Agent,
    object: ObjectRef,
    key: PropertyKey,
    value: Value,
    receiver: Value,
    lifetime: AllocationLifetime,
) -> Completion<bool> {
    let result = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.set(&mut mutator, object, key, value, receiver, lifetime)
    });
    result.map_err(|error| internal_method_error(agent, error))
}

/// ECMAScript `SetSuper`-style helper using a pre-resolved home object and receiver.
///
/// # Errors
/// Returns an abrupt completion if the home object has a null prototype or the
/// underlying object internal methods fail.
pub fn super_set(
    agent: &mut Agent,
    home_object: ObjectRef,
    receiver: Value,
    key: PropertyKey,
    value: Value,
    lifetime: AllocationLifetime,
) -> Completion<bool> {
    let base = super_base(agent, home_object)?;
    ordinary_set_with_receiver(agent, base, key, value, receiver, lifetime)
}

/// Ordinary-only `DeletePropertyOrThrow`-style primitive over the object substrate.
///
/// This helper bypasses proxy traps by going directly through `ObjectRuntime`
/// internal methods. Guest-observable code should use
/// [`super::delete_property_in_context`] unless the algorithm is explicitly
/// operating on an ordinary/bootstrap object.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn ordinary_delete_property(
    agent: &mut Agent,
    object: ObjectRef,
    key: PropertyKey,
) -> Completion<bool> {
    if let Some(numeric_key) = typed_array_numeric_key(agent, object, key) {
        return Ok(!matches!(numeric_key, TypedArrayNumericKey::Valid(_)));
    }
    let result = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.delete(&mut mutator, object, key)
    });
    result.map_err(|error| internal_method_error(agent, error))
}

/// Ordinary-only `CreateDataProperty` over the object substrate.
///
/// This helper bypasses proxy traps by going directly through `ObjectRuntime`
/// internal methods. Guest-observable code should use a context operation that
/// defines the property through proxy-aware object semantics unless the
/// algorithm is explicitly operating on an ordinary/bootstrap object.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn ordinary_create_data_property(
    agent: &mut Agent,
    object: ObjectRef,
    key: PropertyKey,
    value: Value,
    lifetime: AllocationLifetime,
) -> Completion<bool> {
    let mut descriptor = PropertyDescriptor::new();
    descriptor.set_value(value);
    descriptor.set_writable(true);
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);

    let result = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.define_own_property(&mut mutator, object, key, descriptor, lifetime)
    });
    result.map_err(|error| internal_method_error(agent, error))
}

/// ECMAScript `Call` over the public object substrate.
///
/// # Errors
/// Returns an abrupt completion if the callee is not an object or if the
/// underlying call internal methods fail.
pub fn call(
    agent: &mut Agent,
    callee: ObjectRef,
    this_value: Value,
    arguments: &[Value],
    registry: &mut dyn NativeFunctionRegistry,
) -> Completion<Value> {
    let result = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.call(&mut mutator, callee, this_value, arguments, registry)
    });
    result.map_err(|error| internal_method_error(agent, error))
}

/// ECMAScript `Construct` over the public object substrate.
///
/// # Errors
/// Returns an abrupt completion if the callee is not an object or if the
/// underlying construct internal methods fail.
pub fn construct(
    agent: &mut Agent,
    callee: ObjectRef,
    arguments: &[Value],
    new_target: Option<ObjectRef>,
    registry: &mut dyn NativeFunctionRegistry,
) -> Completion<ObjectRef> {
    let result = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.construct(&mut mutator, callee, arguments, new_target, registry)
    });
    result.map_err(|error| internal_method_error(agent, error))
}

use crate::errors::{internal_method_error, throw_type_error};
use lyng_js_common::AtomId;
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::ClassPrivateElementKind;
use lyng_js_types::{Completion, ObjectRef, Value};

/// Defines a private field descriptor layout for a class.
///
/// # Errors
/// Returns a type-error completion when the class/prototype layout cannot be recorded.
pub fn define_private_field_layout(
    agent: &mut Agent,
    class_object: ObjectRef,
    prototype: ObjectRef,
    name: AtomId,
    is_static: bool,
) -> Completion<u32> {
    agent
        .with_heap_and_objects(|_heap, objects| {
            objects.define_private_field_layout(class_object, prototype, name, is_static)
        })
        .ok_or_else(|| throw_type_error(agent))
}

/// Defines a private method/accessor/field descriptor layout for a class.
///
/// # Errors
/// Returns a type-error completion when the class/prototype layout cannot be recorded.
pub fn define_private_element_layout(
    agent: &mut Agent,
    class_object: ObjectRef,
    prototype: ObjectRef,
    name: AtomId,
    is_static: bool,
    kind: ClassPrivateElementKind,
) -> Completion<u32> {
    agent
        .with_heap_and_objects(|_heap, objects| {
            objects.define_private_element_layout(class_object, prototype, name, is_static, kind)
        })
        .ok_or_else(|| throw_type_error(agent))
}

/// Installs a private method or accessor value.
///
/// # Errors
/// Returns an abrupt completion when object private storage rejects the install.
pub fn install_private_element_value(
    agent: &mut Agent,
    class_key: ObjectRef,
    descriptor_index: u32,
    value: Value,
) -> Completion<Value> {
    let installed = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.install_private_element_value(
            &mut mutator,
            class_key,
            descriptor_index,
            value,
            AllocationLifetime::Default,
        )
    });
    installed
        .map_err(|error| internal_method_error(agent, error))
        .map(|()| value)
}

/// Installs the runtime key value for an instance public field.
///
/// # Errors
/// Returns an abrupt completion when class private storage cannot record the key.
pub fn install_instance_public_field_key(
    agent: &mut Agent,
    class_object: ObjectRef,
    field_index: u32,
    key_value: Value,
) -> Completion<Value> {
    let installed = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.install_instance_public_field_key(
            &mut mutator,
            class_object,
            field_index,
            key_value,
            AllocationLifetime::Default,
        )
    });
    installed
        .map_err(|error| internal_method_error(agent, error))
        .map(|()| key_value)
}

/// Reads the runtime key value for an instance public field.
///
/// # Errors
/// Returns an abrupt completion when the class record or backing slot is missing.
pub fn instance_public_field_key(
    agent: &mut Agent,
    class_object: ObjectRef,
    field_index: u32,
) -> Completion<Value> {
    agent
        .objects()
        .instance_public_field_key(agent.heap().view(), class_object, field_index)
        .map_err(|error| internal_method_error(agent, error))
}

/// Reads the kind of a private element descriptor.
///
/// # Errors
/// Returns an abrupt completion when the class record or descriptor is missing.
pub fn private_element_kind(
    agent: &mut Agent,
    class_key: ObjectRef,
    descriptor_index: u32,
) -> Completion<ClassPrivateElementKind> {
    agent
        .objects()
        .private_element_kind(class_key, descriptor_index)
        .map_err(|error| internal_method_error(agent, error))
}

/// Reads a shared private method or accessor value.
///
/// # Errors
/// Returns an abrupt completion when the descriptor or storage is invalid.
pub fn private_shared_element_value(
    agent: &mut Agent,
    class_key: ObjectRef,
    descriptor_index: u32,
) -> Completion<Value> {
    agent
        .objects()
        .private_shared_element_value(agent.heap().view(), class_key, descriptor_index)
        .map_err(|error| internal_method_error(agent, error))
}

/// Initializes a private field on a receiver.
///
/// # Errors
/// Returns an abrupt completion when the brand is invalid, already initialized, or storage is
/// corrupt.
pub fn private_field_init(
    agent: &mut Agent,
    receiver: ObjectRef,
    class_key: ObjectRef,
    descriptor_index: u32,
    value: Value,
) -> Completion<Value> {
    let initialized = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.private_field_init(
            &mut mutator,
            receiver,
            class_key,
            descriptor_index,
            value,
            AllocationLifetime::Default,
        )
    });
    initialized
        .map_err(|error| internal_method_error(agent, error))
        .map(|()| value)
}

/// Reads a private field from a receiver.
///
/// # Errors
/// Returns an abrupt completion when the brand is invalid or storage is corrupt.
pub fn private_field_get(
    agent: &mut Agent,
    receiver: ObjectRef,
    class_key: ObjectRef,
    descriptor_index: u32,
) -> Completion<Value> {
    agent
        .objects()
        .private_field_get(agent.heap().view(), receiver, class_key, descriptor_index)
        .map_err(|error| internal_method_error(agent, error))
}

/// Stores a private field on a receiver.
///
/// # Errors
/// Returns an abrupt completion when the brand is invalid or storage update fails.
pub fn private_field_set(
    agent: &mut Agent,
    receiver: ObjectRef,
    class_key: ObjectRef,
    descriptor_index: u32,
    value: Value,
) -> Completion<Value> {
    let updated = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.private_field_set(&mut mutator, receiver, class_key, descriptor_index, value)
    });
    updated
        .map_err(|error| internal_method_error(agent, error))
        .map(|()| value)
}

/// Tests whether a receiver has a private brand.
///
/// # Errors
/// Returns an abrupt completion when the class record or descriptor is missing.
pub fn private_has(
    agent: &mut Agent,
    receiver: ObjectRef,
    class_key: ObjectRef,
    descriptor_index: u32,
) -> Completion<bool> {
    agent
        .objects()
        .private_has(receiver, class_key, descriptor_index)
        .map_err(|error| internal_method_error(agent, error))
}

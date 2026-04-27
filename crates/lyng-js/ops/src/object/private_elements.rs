use crate::errors::{internal_method_error, throw_type_error};
use lyng_js_common::AtomId;
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::ClassPrivateElementKind;
use lyng_js_types::{Completion, ObjectRef, Value};

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

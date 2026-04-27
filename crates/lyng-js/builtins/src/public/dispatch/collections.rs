mod iteration;

use super::{
    close_iterator_after_error, get_property_from_object, iterators::ArrayIterationKind,
    length_value_u64, map_completion, type_error, BuiltinIteratorBridge,
    PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use iteration::{
    map_for_each_builtin, map_iterator_factory_builtin, map_iterator_next_builtin,
    set_for_each_builtin, set_iterator_factory_builtin, set_iterator_next_builtin,
};
use lyng_js_gc::{AllocationLifetime, WeakHeapRef};
use lyng_js_objects::{
    MapEntry, MapObjectData, ObjectAllocation, ObjectColdData, OrdinaryObjectData, SetObjectData,
};
use lyng_js_ops::{iterator, read};
use lyng_js_types::{BuiltinFunctionId, ObjectRef, PropertyKey, RealmRef, Value};

pub(super) fn dispatch_collection_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_collection_constructor_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_map_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_set_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_weak_collection_builtin(context, entry, invocation)
}

fn dispatch_collection_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::map_builtin() {
        return map_builtin(context, invocation).map(Some);
    }
    if entry == super::set_builtin() {
        return set_builtin(context, invocation).map(Some);
    }
    if entry == super::weak_map_builtin() {
        return weak_map_builtin(context, invocation).map(Some);
    }
    if entry == super::weak_set_builtin() {
        return weak_set_builtin(context, invocation).map(Some);
    }
    if entry == super::weak_ref_builtin() {
        return weak_ref_builtin(context, invocation).map(Some);
    }
    if entry == super::finalization_registry_builtin() {
        return finalization_registry_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_map_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::map_get_builtin() {
        return map_get_builtin(context, invocation).map(Some);
    }
    if entry == super::map_set_builtin() {
        return map_set_builtin(context, invocation).map(Some);
    }
    if entry == super::map_has_builtin() {
        return map_has_builtin(context, invocation).map(Some);
    }
    if entry == super::map_delete_builtin() {
        return map_delete_builtin(context, invocation).map(Some);
    }
    if entry == super::map_clear_builtin() {
        return map_clear_builtin(context, invocation).map(Some);
    }
    if entry == super::map_entries_builtin() {
        return map_iterator_factory_builtin(context, invocation, ArrayIterationKind::Entry)
            .map(Some);
    }
    if entry == super::map_values_builtin() {
        return map_iterator_factory_builtin(context, invocation, ArrayIterationKind::Value)
            .map(Some);
    }
    if entry == super::map_keys_builtin() {
        return map_iterator_factory_builtin(context, invocation, ArrayIterationKind::Key)
            .map(Some);
    }
    if entry == super::map_for_each_builtin() {
        return map_for_each_builtin(context, invocation).map(Some);
    }
    if entry == super::map_size_getter_builtin() {
        return map_size_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::map_iterator_next_builtin() {
        return map_iterator_next_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_set_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::set_add_builtin() {
        return set_add_builtin(context, invocation).map(Some);
    }
    if entry == super::set_has_builtin() {
        return set_has_builtin(context, invocation).map(Some);
    }
    if entry == super::set_delete_builtin() {
        return set_delete_builtin(context, invocation).map(Some);
    }
    if entry == super::set_clear_builtin() {
        return set_clear_builtin(context, invocation).map(Some);
    }
    if entry == super::set_entries_builtin() {
        return set_iterator_factory_builtin(context, invocation, ArrayIterationKind::Entry)
            .map(Some);
    }
    if entry == super::set_values_builtin() {
        return set_iterator_factory_builtin(context, invocation, ArrayIterationKind::Value)
            .map(Some);
    }
    if entry == super::set_keys_builtin() {
        return set_iterator_factory_builtin(context, invocation, ArrayIterationKind::Key)
            .map(Some);
    }
    if entry == super::set_for_each_builtin() {
        return set_for_each_builtin(context, invocation).map(Some);
    }
    if entry == super::set_size_getter_builtin() {
        return set_size_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::set_iterator_next_builtin() {
        return set_iterator_next_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_weak_collection_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::weak_map_get_builtin() {
        return weak_map_get_builtin(context, invocation).map(Some);
    }
    if entry == super::weak_map_set_builtin() {
        return weak_map_set_builtin(context, invocation).map(Some);
    }
    if entry == super::weak_map_has_builtin() {
        return weak_map_has_builtin(context, invocation).map(Some);
    }
    if entry == super::weak_map_delete_builtin() {
        return weak_map_delete_builtin(context, invocation).map(Some);
    }
    if entry == super::weak_set_add_builtin() {
        return weak_set_add_builtin(context, invocation).map(Some);
    }
    if entry == super::weak_set_has_builtin() {
        return weak_set_has_builtin(context, invocation).map(Some);
    }
    if entry == super::weak_set_delete_builtin() {
        return weak_set_delete_builtin(context, invocation).map(Some);
    }
    if entry == super::weak_ref_deref_builtin() {
        return weak_ref_deref_builtin(context, invocation).map(Some);
    }
    if entry == super::finalization_registry_register_builtin() {
        return finalization_registry_register_builtin(context, invocation).map(Some);
    }
    if entry == super::finalization_registry_unregister_builtin() {
        return finalization_registry_unregister_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn allocate_map_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let root_shape = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(lyng_js_env::RealmRecord::root_shape)
    }
    .ok_or_else(|| type_error(cx))?;
    Ok(cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_prototype(Some(prototype))
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::Map)),
            AllocationLifetime::Default,
        );
        let installed = objects.install_map_object(object, MapObjectData::new());
        debug_assert!(installed, "fresh Map object should install ordered storage");
        object
    }))
}

fn allocate_set_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let root_shape = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(lyng_js_env::RealmRecord::root_shape)
    }
    .ok_or_else(|| type_error(cx))?;
    Ok(cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_prototype(Some(prototype))
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::Set)),
            AllocationLifetime::Default,
        );
        let installed = objects.install_set_object(object, SetObjectData::new());
        debug_assert!(installed, "fresh Set object should install ordered storage");
        object
    }))
}

fn allocate_weak_map_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let root_shape = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(lyng_js_env::RealmRecord::root_shape)
    }
    .ok_or_else(|| type_error(cx))?;
    Ok(cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_prototype(Some(prototype))
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::WeakMap)),
            AllocationLifetime::Default,
        );
        let initialized = mutator.init_weak_map(object);
        debug_assert!(
            initialized,
            "fresh WeakMap object should install weak state"
        );
        object
    }))
}

fn allocate_weak_set_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let root_shape = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(lyng_js_env::RealmRecord::root_shape)
    }
    .ok_or_else(|| type_error(cx))?;
    Ok(cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_prototype(Some(prototype))
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::WeakSet)),
            AllocationLifetime::Default,
        );
        let initialized = mutator.init_weak_set(object);
        debug_assert!(
            initialized,
            "fresh WeakSet object should install weak state"
        );
        object
    }))
}

fn allocate_weak_ref_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
    target: WeakHeapRef,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let root_shape = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(lyng_js_env::RealmRecord::root_shape)
    }
    .ok_or_else(|| type_error(cx))?;
    Ok(cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_prototype(Some(prototype))
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::WeakRef)),
            AllocationLifetime::Default,
        );
        let initialized = mutator.init_weak_ref(object, target);
        debug_assert!(
            initialized,
            "fresh WeakRef object should install weak state"
        );
        object
    }))
}

fn allocate_finalization_registry_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
    callback: ObjectRef,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let root_shape = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(lyng_js_env::RealmRecord::root_shape)
    }
    .ok_or_else(|| type_error(cx))?;
    Ok(cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_prototype(Some(prototype))
                .with_cold_data(ObjectColdData::Ordinary(
                    OrdinaryObjectData::FinalizationRegistry,
                ))
                .with_ordinary_payload_value(Value::from_object_ref(callback)),
            AllocationLifetime::Default,
        );
        let initialized = mutator.init_finalization_registry(object);
        debug_assert!(
            initialized,
            "fresh FinalizationRegistry object should install weak state"
        );
        object
    }))
}

fn map_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if cx.agent().objects().is_map_object(object) {
        Ok(object)
    } else {
        Err(type_error(cx))
    }
}

fn set_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if cx.agent().objects().is_set_object(object) {
        Ok(object)
    } else {
        Err(type_error(cx))
    }
}

fn weak_map_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if cx.agent().objects().is_weak_map_object(object) {
        Ok(object)
    } else {
        Err(type_error(cx))
    }
}

fn weak_set_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if cx.agent().objects().is_weak_set_object(object) {
        Ok(object)
    } else {
        Err(type_error(cx))
    }
}

fn weak_ref_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if cx.agent().objects().is_weak_ref_object(object) {
        Ok(object)
    } else {
        Err(type_error(cx))
    }
}

fn finalization_registry_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if cx.agent().objects().is_finalization_registry_object(object) {
        Ok(object)
    } else {
        Err(type_error(cx))
    }
}

#[inline]
fn weak_heap_ref_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Option<WeakHeapRef> {
    match WeakHeapRef::from_value(value) {
        Some(WeakHeapRef::Symbol(symbol)) if cx.agent().global_symbol_key_for(symbol).is_some() => {
            None
        }
        other => other,
    }
}

#[inline]
fn same_weak_heap_ref_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    left: WeakHeapRef,
    right: Value,
) -> bool {
    weak_heap_ref_from_value(cx, right) == Some(left)
}

fn map_entry_index<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    key: Value,
) -> Result<Option<usize>, Cx::Error> {
    let entries = cx
        .agent()
        .objects()
        .map(object)
        .map(|map| map.entries().to_vec())
        .ok_or_else(|| type_error(cx))?;
    for (index, entry) in entries.iter().copied().enumerate() {
        let Some(entry) = entry else {
            continue;
        };
        let heap_view = cx.agent().heap().view();
        let same = read::same_value_zero(heap_view, entry.key(), key);
        if map_completion(cx, same)? {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

fn set_entry_index<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    value: Value,
) -> Result<Option<usize>, Cx::Error> {
    let entries = cx
        .agent()
        .objects()
        .set_object_data(object)
        .map(|set| set.entries().to_vec())
        .ok_or_else(|| type_error(cx))?;
    for (index, entry) in entries.iter().copied().enumerate() {
        let Some(entry) = entry else {
            continue;
        };
        let heap_view = cx.agent().heap().view();
        let same = read::same_value_zero(heap_view, entry, value);
        if map_completion(cx, same)? {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

fn weak_map_store_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    key: Value,
    value: Value,
) -> Result<(), Cx::Error> {
    let key = weak_heap_ref_from_value(cx, key).ok_or_else(|| type_error(cx))?;
    let stored = cx
        .agent()
        .with_heap_and_objects(|heap, _| heap.mutator().weak_map_set(object, key, value));
    if stored {
        Ok(())
    } else {
        Err(type_error(cx))
    }
}

fn weak_set_add_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    value: Value,
) -> Result<(), Cx::Error> {
    let value = weak_heap_ref_from_value(cx, value).ok_or_else(|| type_error(cx))?;
    let inserted = cx
        .agent()
        .with_heap_and_objects(|heap, _| heap.mutator().weak_set_insert(object, value));
    if inserted {
        Ok(())
    } else {
        Err(type_error(cx))
    }
}

fn perform_weak_map_constructor_entries<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    iterable: Value,
) -> Result<(), Cx::Error> {
    let set_atom = cx.agent().atoms_mut().intern_collectible("set");
    let adder = cx.get_property_value(
        Value::from_object_ref(object),
        PropertyKey::from_atom(set_atom),
    )?;
    let adder = cx.require_callable_object(adder)?;
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
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let Some(entry) = next_value.as_object_ref() else {
            let error = type_error(cx);
            return close_iterator_after_error(cx, &mut iterator_record, error);
        };
        let key = match get_property_from_object(cx, entry, PropertyKey::Index(0)) {
            Ok(key) => key,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let value = match get_property_from_object(cx, entry, PropertyKey::Index(1)) {
            Ok(value) => value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        if let Err(error) =
            cx.call_to_completion(adder, Value::from_object_ref(object), &[key, value])
        {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    }
}

fn perform_weak_set_constructor_values<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    iterable: Value,
) -> Result<(), Cx::Error> {
    let add_atom = cx.agent().atoms_mut().intern_collectible("add");
    let adder = cx.get_property_value(
        Value::from_object_ref(object),
        PropertyKey::from_atom(add_atom),
    )?;
    let adder = cx.require_callable_object(adder)?;
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
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        if let Err(error) =
            cx.call_to_completion(adder, Value::from_object_ref(object), &[next_value])
        {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    }
}

fn map_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().map_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let object = allocate_map_object(cx, realm, prototype)?;
    let iterable = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if !(iterable.is_undefined() || iterable.is_null()) {
        perform_weak_map_constructor_entries(cx, object, iterable)?;
    }
    Ok(Value::from_object_ref(object))
}

fn set_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().set_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let object = allocate_set_object(cx, realm, prototype)?;
    let iterable = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if !(iterable.is_undefined() || iterable.is_null()) {
        perform_weak_set_constructor_values(cx, object, iterable)?;
    }
    Ok(Value::from_object_ref(object))
}

fn weak_map_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().weak_map_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let object = allocate_weak_map_object(cx, realm, prototype)?;
    let iterable = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if !(iterable.is_undefined() || iterable.is_null()) {
        perform_weak_map_constructor_entries(cx, object, iterable)?;
    }
    Ok(Value::from_object_ref(object))
}

fn weak_set_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().weak_set_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let object = allocate_weak_set_object(cx, realm, prototype)?;
    let iterable = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if !(iterable.is_undefined() || iterable.is_null()) {
        perform_weak_set_constructor_values(cx, object, iterable)?;
    }
    Ok(Value::from_object_ref(object))
}

fn weak_ref_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let target = invocation
        .arguments()
        .first()
        .copied()
        .and_then(|value| weak_heap_ref_from_value(cx, value))
        .ok_or_else(|| type_error(cx))?;
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().weak_ref_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let object = allocate_weak_ref_object(cx, realm, prototype, target)?;
    Ok(Value::from_object_ref(object))
}

fn finalization_registry_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    let cleanup_callback = invocation
        .arguments()
        .first()
        .copied()
        .ok_or_else(|| type_error(cx))
        .and_then(|value| cx.require_callable_object(value))?;
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().finalization_registry_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let object = allocate_finalization_registry_object(cx, realm, prototype, cleanup_callback)?;
    Ok(Value::from_object_ref(object))
}

fn map_get_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = map_this_object(cx, invocation.this_value())?;
    let key = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(index) = map_entry_index(cx, object, key)? else {
        return Ok(Value::undefined());
    };
    cx.agent()
        .objects()
        .map(object)
        .and_then(|map| map.entries().get(index).copied().flatten())
        .map(MapEntry::value)
        .ok_or_else(|| type_error(cx))
}

fn map_set_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = map_this_object(cx, invocation.this_value())?;
    let key = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let existing = map_entry_index(cx, object, key)?;
    let updated = cx.agent().with_heap_and_objects(|_, objects| {
        objects.with_map_mut(object, |map| {
            if let Some(index) = existing {
                if let Some(Some(entry)) = map.entries_mut().get_mut(index) {
                    entry.set_value(value);
                    true
                } else {
                    false
                }
            } else {
                map.push(MapEntry::new(key, value));
                true
            }
        })
    });
    if updated == Some(true) {
        Ok(invocation.this_value())
    } else {
        Err(type_error(cx))
    }
}

fn map_has_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = map_this_object(cx, invocation.this_value())?;
    let key = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    Ok(Value::from_bool(
        map_entry_index(cx, object, key)?.is_some(),
    ))
}

fn map_delete_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = map_this_object(cx, invocation.this_value())?;
    let key = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(index) = map_entry_index(cx, object, key)? else {
        return Ok(Value::from_bool(false));
    };
    let deleted = cx.agent().with_heap_and_objects(|_, objects| {
        objects.with_map_mut(object, |map| map.delete_index(index))
    });
    match deleted {
        Some(true) => Ok(Value::from_bool(true)),
        Some(false) => Ok(Value::from_bool(false)),
        None => Err(type_error(cx)),
    }
}

fn map_clear_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = map_this_object(cx, invocation.this_value())?;
    let cleared = cx
        .agent()
        .with_heap_and_objects(|_, objects| objects.with_map_mut(object, MapObjectData::clear));
    if cleared.is_some() {
        Ok(Value::undefined())
    } else {
        Err(type_error(cx))
    }
}

fn map_size_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = map_this_object(cx, invocation.this_value())?;
    let size = cx
        .agent()
        .objects()
        .map(object)
        .map(MapObjectData::len)
        .ok_or_else(|| type_error(cx))?;
    Ok(length_value_u64(u64::try_from(size).unwrap_or(u64::MAX)))
}

fn set_add_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if set_entry_index(cx, object, value)?.is_none() {
        let inserted = cx.agent().with_heap_and_objects(|_, objects| {
            objects.with_set_mut(object, |set| {
                set.push(value);
                true
            })
        });
        if inserted != Some(true) {
            return Err(type_error(cx));
        }
    }
    Ok(invocation.this_value())
}

fn set_has_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    Ok(Value::from_bool(
        set_entry_index(cx, object, value)?.is_some(),
    ))
}

fn set_delete_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(index) = set_entry_index(cx, object, value)? else {
        return Ok(Value::from_bool(false));
    };
    let deleted = cx.agent().with_heap_and_objects(|_, objects| {
        objects.with_set_mut(object, |set| set.delete_index(index))
    });
    match deleted {
        Some(true) => Ok(Value::from_bool(true)),
        Some(false) => Ok(Value::from_bool(false)),
        None => Err(type_error(cx)),
    }
}

fn set_clear_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
    let cleared = cx
        .agent()
        .with_heap_and_objects(|_, objects| objects.with_set_mut(object, SetObjectData::clear));
    if cleared.is_some() {
        Ok(Value::undefined())
    } else {
        Err(type_error(cx))
    }
}

fn set_size_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
    let size = cx
        .agent()
        .objects()
        .set_object_data(object)
        .map(SetObjectData::len)
        .ok_or_else(|| type_error(cx))?;
    Ok(length_value_u64(u64::try_from(size).unwrap_or(u64::MAX)))
}

fn weak_map_get_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = weak_map_this_object(cx, invocation.this_value())?;
    let key = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(key) = weak_heap_ref_from_value(cx, key) else {
        return Ok(Value::undefined());
    };
    let value = cx
        .agent()
        .heap()
        .view()
        .weak_map_get(object, key)
        .ok_or_else(|| type_error(cx))?
        .unwrap_or(Value::undefined());
    Ok(value)
}

fn weak_map_set_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = weak_map_this_object(cx, invocation.this_value())?;
    let key = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    weak_map_store_value(cx, object, key, value)?;
    Ok(invocation.this_value())
}

fn weak_map_has_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = weak_map_this_object(cx, invocation.this_value())?;
    let key = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(key) = weak_heap_ref_from_value(cx, key) else {
        return Ok(Value::from_bool(false));
    };
    let has = cx
        .agent()
        .heap()
        .view()
        .weak_map_get(object, key)
        .ok_or_else(|| type_error(cx))?
        .is_some();
    Ok(Value::from_bool(has))
}

fn weak_map_delete_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = weak_map_this_object(cx, invocation.this_value())?;
    let key = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(key) = weak_heap_ref_from_value(cx, key) else {
        return Ok(Value::from_bool(false));
    };
    let deleted = cx
        .agent()
        .with_heap_and_objects(|heap, _| heap.mutator().weak_map_delete(object, key));
    deleted.map(Value::from_bool).ok_or_else(|| type_error(cx))
}

fn weak_set_add_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = weak_set_this_object(cx, invocation.this_value())?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    weak_set_add_value(cx, object, value)?;
    Ok(invocation.this_value())
}

fn weak_set_has_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = weak_set_this_object(cx, invocation.this_value())?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(value) = weak_heap_ref_from_value(cx, value) else {
        return Ok(Value::from_bool(false));
    };
    let has = cx
        .agent()
        .heap()
        .view()
        .weak_set_contains(object, value)
        .ok_or_else(|| type_error(cx))?;
    Ok(Value::from_bool(has))
}

fn weak_set_delete_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = weak_set_this_object(cx, invocation.this_value())?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(value) = weak_heap_ref_from_value(cx, value) else {
        return Ok(Value::from_bool(false));
    };
    let deleted = cx
        .agent()
        .with_heap_and_objects(|heap, _| heap.mutator().weak_set_delete(object, value));
    deleted.map(Value::from_bool).ok_or_else(|| type_error(cx))
}

fn weak_ref_deref_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = weak_ref_this_object(cx, invocation.this_value())?;
    let target = cx
        .agent()
        .weak_ref_target(object)
        .ok_or_else(|| type_error(cx))?
        .map_or(Value::undefined(), |target| {
            cx.agent().keep_weak_target_alive(target);
            match target {
                WeakHeapRef::Object(object) => Value::from_object_ref(object),
                WeakHeapRef::Symbol(symbol) => Value::from_symbol_ref(symbol),
            }
        });
    Ok(target)
}

fn finalization_registry_register_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let registry = finalization_registry_this_object(cx, invocation.this_value())?;
    let target = invocation
        .arguments()
        .first()
        .copied()
        .and_then(|value| weak_heap_ref_from_value(cx, value))
        .ok_or_else(|| type_error(cx))?;
    let holdings = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    if same_weak_heap_ref_value(cx, target, holdings) {
        return Err(type_error(cx));
    }
    let unregister_token = match invocation.arguments().get(2).copied() {
        Some(value) if value.is_undefined() => None,
        Some(value) => Some(weak_heap_ref_from_value(cx, value).ok_or_else(|| type_error(cx))?),
        None => None,
    };

    let registered = cx.agent().with_heap_and_objects(|heap, _| {
        heap.mutator()
            .finalization_registry_register(registry, target, holdings, unregister_token)
    });
    if !registered {
        return Err(type_error(cx));
    }
    Ok(Value::undefined())
}

fn finalization_registry_unregister_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let registry = finalization_registry_this_object(cx, invocation.this_value())?;
    let unregister_token = invocation
        .arguments()
        .first()
        .copied()
        .and_then(|value| weak_heap_ref_from_value(cx, value))
        .ok_or_else(|| type_error(cx))?;
    let unregistered = cx
        .agent()
        .with_heap_and_objects(|heap, _| {
            heap.mutator()
                .finalization_registry_unregister(registry, unregister_token)
        })
        .ok_or_else(|| type_error(cx))?;
    Ok(Value::from_bool(unregistered))
}

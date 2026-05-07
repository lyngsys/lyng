mod iteration;

use super::{
    close_iterator_after_error, create_array_from_values, get_property_from_object,
    iterators::ArrayIterationKind, length_u64_as_number, length_value_u64, map_completion,
    property_key_from_text, to_number_for_builtin, type_error, BuiltinIteratorBridge,
    PublicBuiltinDispatchContext, MAX_SAFE_INTEGER_U64,
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
use lyng_js_ops::{iterator, pure, read};
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
    if entry == super::map_group_by_builtin() {
        return map_group_by_builtin(context, invocation).map(Some);
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
    if entry == super::map_get_or_insert_builtin() {
        return map_get_or_insert_builtin(context, invocation).map(Some);
    }
    if entry == super::map_get_or_insert_computed_builtin() {
        return map_get_or_insert_computed_builtin(context, invocation).map(Some);
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
    if entry == super::set_union_builtin() {
        return set_union_builtin(context, invocation).map(Some);
    }
    if entry == super::set_intersection_builtin() {
        return set_intersection_builtin(context, invocation).map(Some);
    }
    if entry == super::set_difference_builtin() {
        return set_difference_builtin(context, invocation).map(Some);
    }
    if entry == super::set_symmetric_difference_builtin() {
        return set_symmetric_difference_builtin(context, invocation).map(Some);
    }
    if entry == super::set_is_subset_of_builtin() {
        return set_is_subset_of_builtin(context, invocation).map(Some);
    }
    if entry == super::set_is_superset_of_builtin() {
        return set_is_superset_of_builtin(context, invocation).map(Some);
    }
    if entry == super::set_is_disjoint_from_builtin() {
        return set_is_disjoint_from_builtin(context, invocation).map(Some);
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
    if entry == super::weak_map_get_or_insert_builtin() {
        return weak_map_get_or_insert_builtin(context, invocation).map(Some);
    }
    if entry == super::weak_map_get_or_insert_computed_builtin() {
        return weak_map_get_or_insert_computed_builtin(context, invocation).map(Some);
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
        let next_value = next_value?;
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
        let next_value = next_value?;
        if let Err(error) =
            cx.call_to_completion(adder, Value::from_object_ref(object), &[next_value])
        {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    }
}

fn add_value_to_map_group<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    groups: &mut Vec<(Value, Vec<Value>)>,
    key: Value,
    value: Value,
) -> Result<(), Cx::Error> {
    for (existing_key, values) in groups.iter_mut() {
        let same = {
            let heap_view = cx.agent().heap().view();
            read::same_value(heap_view, *existing_key, key)
        };
        if map_completion(cx, same)? {
            values.push(value);
            return Ok(());
        }
    }
    groups.push((key, vec![value]));
    Ok(())
}

fn map_group_by_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let items = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let callback = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let callback = cx.require_callable_object(callback)?;
    let mut iterator_record = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::get_iterator(&mut bridge, items)?
    };
    let mut groups = Vec::new();
    let mut index = 0_u64;

    loop {
        if index >= MAX_SAFE_INTEGER_U64 {
            let error = type_error(cx);
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }

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
        let mut key = match cx.call_to_completion(
            callback,
            Value::undefined(),
            &[value, length_value_u64(index)],
        ) {
            Ok(key) => key,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        if pure::is_negative_zero(key) {
            key = Value::from_smi(0);
        }
        add_value_to_map_group(cx, &mut groups, key, value)?;
        index += 1;
    }

    let realm = cx.builtin_realm();
    let map_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().map_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let map = allocate_map_object(cx, realm, map_prototype)?;
    let mut entries = Vec::with_capacity(groups.len());
    for (key, values) in groups {
        let array = create_array_from_values(cx, &values)?;
        entries.push(MapEntry::new(key, Value::from_object_ref(array)));
    }
    let installed = cx.agent().with_heap_and_objects(|_, objects| {
        objects.with_map_mut(map, |map_data| {
            for entry in entries {
                map_data.push(entry);
            }
            true
        })
    });
    match installed {
        Some(true) => Ok(Value::from_object_ref(map)),
        Some(false) | None => Err(type_error(cx)),
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

fn map_get_or_insert_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = map_this_object(cx, invocation.this_value())?;
    let key = canonicalize_keyed_collection_key(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    );
    let value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    if let Some(index) = map_entry_index(cx, object, key)? {
        return cx
            .agent()
            .objects()
            .map(object)
            .and_then(|map| map.entries().get(index).copied().flatten())
            .map(MapEntry::value)
            .ok_or_else(|| type_error(cx));
    }
    let inserted = cx.agent().with_heap_and_objects(|_, objects| {
        objects.with_map_mut(object, |map| {
            map.push(MapEntry::new(key, value));
            true
        })
    });
    if inserted != Some(true) {
        return Err(type_error(cx));
    }
    Ok(value)
}

fn map_get_or_insert_computed_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = map_this_object(cx, invocation.this_value())?;
    let key = canonicalize_keyed_collection_key(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    );
    let callback = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let callback_object = cx.require_callable_object(callback)?;
    if let Some(index) = map_entry_index(cx, object, key)? {
        return cx
            .agent()
            .objects()
            .map(object)
            .and_then(|map| map.entries().get(index).copied().flatten())
            .map(MapEntry::value)
            .ok_or_else(|| type_error(cx));
    }
    let value = cx.call_to_completion(callback_object, Value::undefined(), &[key])?;
    let existing_after = map_entry_index(cx, object, key)?;
    let inserted = cx.agent().with_heap_and_objects(|_, objects| {
        objects.with_map_mut(object, |map| {
            if let Some(index) = existing_after {
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
    if inserted != Some(true) {
        return Err(type_error(cx));
    }
    Ok(value)
}

#[inline]
fn canonicalize_keyed_collection_key(key: Value) -> Value {
    if let Some(number) = key.as_f64()
        && number == 0.0
        && number.is_sign_negative()
    {
        return Value::from_smi(0);
    }
    key
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

struct SetRecord {
    object: ObjectRef,
    size: f64,
    has: ObjectRef,
    keys: ObjectRef,
}

enum SetEntryState {
    Occupied(Value),
    Empty,
    End,
}

fn get_set_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<SetRecord, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let size_key = property_key_from_text(cx, "size");
    let size_value = cx.get_property_value(value, size_key)?;
    let raw_size = to_number_for_builtin(cx, size_value)?;
    if raw_size.is_nan() {
        return Err(type_error(cx));
    }
    let int_size = if raw_size == 0.0 {
        0.0
    } else if !raw_size.is_finite() {
        raw_size
    } else {
        raw_size.trunc()
    };
    if int_size < 0.0 {
        return Err(type_error(cx));
    }
    let has_key = property_key_from_text(cx, "has");
    let has_value = cx.get_property_value(value, has_key)?;
    let has = cx.require_callable_object(has_value)?;
    let keys_key = property_key_from_text(cx, "keys");
    let keys_value = cx.get_property_value(value, keys_key)?;
    let keys = cx.require_callable_object(keys_value)?;
    Ok(SetRecord {
        object,
        size: int_size,
        has,
        keys,
    })
}

fn set_record_iterator<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    record: &SetRecord,
) -> Result<lyng_js_ops::iterator::IteratorRecord, Cx::Error> {
    let iterator_value =
        cx.call_to_completion(record.keys, Value::from_object_ref(record.object), &[])?;
    let iterator_object = iterator_value
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let next_key = property_key_from_text(cx, "next");
    let next_value = cx.get_property_value(Value::from_object_ref(iterator_object), next_key)?;
    let next_method = cx.require_callable_object(next_value)?;
    Ok(lyng_js_ops::iterator::IteratorRecord::new(
        iterator_object,
        next_method,
    ))
}

fn set_record_has<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    record: &SetRecord,
    value: Value,
) -> Result<bool, Cx::Error> {
    let result =
        cx.call_to_completion(record.has, Value::from_object_ref(record.object), &[value])?;
    let agent = cx.agent();
    let to_bool = read::to_boolean_agent(agent, result);
    map_completion(cx, to_bool)
}

fn collect_set_values<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
) -> Result<Vec<Value>, Cx::Error> {
    let values = cx
        .agent()
        .objects()
        .set_object_data(object)
        .map(|set| {
            set.entries()
                .iter()
                .filter_map(|entry| *entry)
                .collect::<Vec<Value>>()
        })
        .ok_or_else(|| type_error(cx))?;
    Ok(values)
}

fn set_entry_at<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    index: usize,
) -> Result<SetEntryState, Cx::Error> {
    let entry = {
        let agent = cx.agent();
        let Some(set) = agent.objects().set_object_data(object) else {
            return Err(type_error(cx));
        };
        set.entries().get(index).copied()
    };
    Ok(match entry {
        Some(Some(value)) => SetEntryState::Occupied(value),
        Some(None) => SetEntryState::Empty,
        None => SetEntryState::End,
    })
}

fn allocate_result_set<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().set_prototype())
        .ok_or_else(|| type_error(cx))?;
    allocate_set_object(cx, realm, prototype)
}

fn set_push_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    target: ObjectRef,
    value: Value,
) -> Result<(), Cx::Error> {
    let inserted = cx.agent().with_heap_and_objects(|_, objects| {
        objects.with_set_mut(target, |set| {
            set.push(value);
            true
        })
    });
    if inserted == Some(true) {
        Ok(())
    } else {
        Err(type_error(cx))
    }
}

fn set_contains_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    target: ObjectRef,
    value: Value,
) -> Result<bool, Cx::Error> {
    Ok(set_entry_index(cx, target, value)?.is_some())
}

fn canonical_set_value(value: Value) -> Value {
    canonicalize_keyed_collection_key(value)
}

fn set_union_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
    let other = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let record = get_set_record(cx, other)?;
    let result = allocate_result_set(cx)?;
    let mut iterator_record = set_record_iterator(cx, &record)?;
    for value in collect_set_values(cx, object)? {
        set_push_value(cx, result, value)?;
    }
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
        let next_value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let next_value = match next_value {
            Ok(value) => canonical_set_value(value),
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        if !set_contains_value(cx, result, next_value)?
            && let Err(error) = set_push_value(cx, result, next_value)
        {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    }
    Ok(Value::from_object_ref(result))
}

fn set_intersection_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
    let other = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let record = get_set_record(cx, other)?;
    let result = allocate_result_set(cx)?;
    let this_size = u64::try_from(
        cx.agent()
            .objects()
            .set_object_data(object)
            .map(SetObjectData::len)
            .ok_or_else(|| type_error(cx))?,
    )
    .unwrap_or(u64::MAX);
    let this_size_f = length_u64_as_number(this_size);
    if this_size_f <= record.size {
        let mut index = 0_usize;
        loop {
            let entry = set_entry_at(cx, object, index)?;
            index = index.saturating_add(1);
            let value = match entry {
                SetEntryState::Occupied(value) => value,
                SetEntryState::Empty => continue,
                SetEntryState::End => break,
            };
            if set_record_has(cx, &record, value)? {
                let canonical = canonical_set_value(value);
                if !set_contains_value(cx, result, canonical)? {
                    set_push_value(cx, result, canonical)?;
                }
            }
        }
    } else {
        let mut iterator_record = set_record_iterator(cx, &record)?;
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
            let next_value = {
                let mut bridge = BuiltinIteratorBridge { cx };
                iterator::iterator_value(&mut bridge, next)
            };
            let next_value = match next_value {
                Ok(value) => canonical_set_value(value),
                Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
            };
            let in_this = match set_contains_value(cx, object, next_value) {
                Ok(b) => b,
                Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
            };
            if in_this
                && !set_contains_value(cx, result, next_value)?
                && let Err(error) = set_push_value(cx, result, next_value)
            {
                return close_iterator_after_error(cx, &mut iterator_record, error);
            }
        }
    }
    Ok(Value::from_object_ref(result))
}

fn set_difference_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
    let other = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let record = get_set_record(cx, other)?;
    let result = allocate_result_set(cx)?;
    let this_size = u64::try_from(
        cx.agent()
            .objects()
            .set_object_data(object)
            .map(SetObjectData::len)
            .ok_or_else(|| type_error(cx))?,
    )
    .unwrap_or(u64::MAX);
    let this_size_f = length_u64_as_number(this_size);
    let this_values = collect_set_values(cx, object)?;
    if this_size_f <= record.size {
        for value in this_values {
            if !set_record_has(cx, &record, value)? {
                set_push_value(cx, result, value)?;
            }
        }
    } else {
        for value in &this_values {
            set_push_value(cx, result, *value)?;
        }
        let mut iterator_record = set_record_iterator(cx, &record)?;
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
            let next_value = {
                let mut bridge = BuiltinIteratorBridge { cx };
                iterator::iterator_value(&mut bridge, next)
            };
            let next_value = match next_value {
                Ok(value) => canonical_set_value(value),
                Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
            };
            let index = match set_entry_index(cx, result, next_value) {
                Ok(idx) => idx,
                Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
            };
            if let Some(idx) = index {
                let deleted = cx.agent().with_heap_and_objects(|_, objects| {
                    objects.with_set_mut(result, |set| set.delete_index(idx))
                });
                if deleted != Some(true) {
                    let error = type_error(cx);
                    return close_iterator_after_error(cx, &mut iterator_record, error);
                }
            }
        }
    }
    Ok(Value::from_object_ref(result))
}

fn set_symmetric_difference_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
    let other = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let record = get_set_record(cx, other)?;
    let result = allocate_result_set(cx)?;
    let mut iterator_record = set_record_iterator(cx, &record)?;
    for value in collect_set_values(cx, object)? {
        set_push_value(cx, result, value)?;
    }
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
        let next_value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let next_value = match next_value {
            Ok(value) => canonical_set_value(value),
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let result_index = match set_entry_index(cx, result, next_value) {
            Ok(idx) => idx,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let in_original = match set_contains_value(cx, object, next_value) {
            Ok(b) => b,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        if in_original {
            if let Some(idx) = result_index {
                let deleted = cx.agent().with_heap_and_objects(|_, objects| {
                    objects.with_set_mut(result, |set| set.delete_index(idx))
                });
                if deleted != Some(true) {
                    let error = type_error(cx);
                    return close_iterator_after_error(cx, &mut iterator_record, error);
                }
            }
        } else if result_index.is_none()
            && let Err(error) = set_push_value(cx, result, next_value)
        {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    }
    Ok(Value::from_object_ref(result))
}

fn set_is_subset_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
    let other = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let record = get_set_record(cx, other)?;
    let this_size = u64::try_from(
        cx.agent()
            .objects()
            .set_object_data(object)
            .map(SetObjectData::len)
            .ok_or_else(|| type_error(cx))?,
    )
    .unwrap_or(u64::MAX);
    if length_u64_as_number(this_size) > record.size {
        return Ok(Value::from_bool(false));
    }
    let mut index = 0_usize;
    loop {
        let entry = {
            let agent = cx.agent();
            let Some(set) = agent.objects().set_object_data(object) else {
                return Err(type_error(cx));
            };
            set.entries().get(index).copied()
        };
        let Some(entry) = entry else {
            break;
        };
        index = index.saturating_add(1);
        let Some(value) = entry else {
            continue;
        };
        if !set_record_has(cx, &record, value)? {
            return Ok(Value::from_bool(false));
        }
    }
    Ok(Value::from_bool(true))
}

fn set_is_superset_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
    let other = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let record = get_set_record(cx, other)?;
    let this_size = u64::try_from(
        cx.agent()
            .objects()
            .set_object_data(object)
            .map(SetObjectData::len)
            .ok_or_else(|| type_error(cx))?,
    )
    .unwrap_or(u64::MAX);
    if length_u64_as_number(this_size) < record.size {
        return Ok(Value::from_bool(false));
    }
    let mut iterator_record = set_record_iterator(cx, &record)?;
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
        let next_value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let next_value = match next_value {
            Ok(value) => canonical_set_value(value),
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let in_this = match set_contains_value(cx, object, next_value) {
            Ok(b) => b,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        if !in_this {
            let close_result = {
                let mut bridge = BuiltinIteratorBridge { cx };
                lyng_js_ops::iterator::iterator_close(
                    &mut bridge,
                    &mut iterator_record,
                    Ok::<(), lyng_js_types::AbruptCompletion>(()),
                )
            };
            close_result?;
            return Ok(Value::from_bool(false));
        }
    }
    Ok(Value::from_bool(true))
}

fn set_is_disjoint_from_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
    let other = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let record = get_set_record(cx, other)?;
    let this_size = u64::try_from(
        cx.agent()
            .objects()
            .set_object_data(object)
            .map(SetObjectData::len)
            .ok_or_else(|| type_error(cx))?,
    )
    .unwrap_or(u64::MAX);
    let this_size_f = length_u64_as_number(this_size);
    if this_size_f <= record.size {
        let mut index = 0_usize;
        loop {
            let entry = {
                let agent = cx.agent();
                let Some(set) = agent.objects().set_object_data(object) else {
                    return Err(type_error(cx));
                };
                set.entries().get(index).copied()
            };
            let Some(entry) = entry else {
                break;
            };
            index = index.saturating_add(1);
            let Some(value) = entry else {
                continue;
            };
            if set_record_has(cx, &record, value)? {
                return Ok(Value::from_bool(false));
            }
        }
    } else {
        let mut iterator_record = set_record_iterator(cx, &record)?;
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
            let next_value = {
                let mut bridge = BuiltinIteratorBridge { cx };
                iterator::iterator_value(&mut bridge, next)
            };
            let next_value = match next_value {
                Ok(value) => canonical_set_value(value),
                Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
            };
            let in_this = match set_contains_value(cx, object, next_value) {
                Ok(b) => b,
                Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
            };
            if in_this {
                let close_result = {
                    let mut bridge = BuiltinIteratorBridge { cx };
                    lyng_js_ops::iterator::iterator_close(
                        &mut bridge,
                        &mut iterator_record,
                        Ok::<(), lyng_js_types::AbruptCompletion>(()),
                    )
                };
                close_result?;
                return Ok(Value::from_bool(false));
            }
        }
    }
    Ok(Value::from_bool(true))
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

fn weak_map_get_or_insert_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = weak_map_this_object(cx, invocation.this_value())?;
    let key_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let key = weak_heap_ref_from_value(cx, key_value).ok_or_else(|| type_error(cx))?;
    if let Some(existing) = cx
        .agent()
        .heap()
        .view()
        .weak_map_get(object, key)
        .ok_or_else(|| type_error(cx))?
    {
        return Ok(existing);
    }
    let stored = cx
        .agent()
        .with_heap_and_objects(|heap, _| heap.mutator().weak_map_set(object, key, value));
    if !stored {
        return Err(type_error(cx));
    }
    Ok(value)
}

fn weak_map_get_or_insert_computed_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = weak_map_this_object(cx, invocation.this_value())?;
    let key_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let callback = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let callback_object = cx.require_callable_object(callback)?;
    let key = weak_heap_ref_from_value(cx, key_value).ok_or_else(|| type_error(cx))?;
    if let Some(existing) = cx
        .agent()
        .heap()
        .view()
        .weak_map_get(object, key)
        .ok_or_else(|| type_error(cx))?
    {
        return Ok(existing);
    }
    let value = cx.call_to_completion(callback_object, Value::undefined(), &[key_value])?;
    let stored = cx
        .agent()
        .with_heap_and_objects(|heap, _| heap.mutator().weak_map_set(object, key, value));
    if !stored {
        return Err(type_error(cx));
    }
    Ok(value)
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

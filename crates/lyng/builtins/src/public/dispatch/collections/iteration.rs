use super::super::{
    create_array_result,
    iterators::{
        allocate_iterator_object, create_iterator_result_value, iterator_slot_value_for_builtin,
        set_iterator_slot_value_for_builtin, ArrayIterationKind, MAP_ITERATOR_INDEX_SLOT,
        MAP_ITERATOR_KIND_SLOT, MAP_ITERATOR_TARGET_SLOT, SET_ITERATOR_INDEX_SLOT,
        SET_ITERATOR_KIND_SLOT, SET_ITERATOR_TARGET_SLOT,
    },
    length_value, set_property_on_object, type_error, PublicBuiltinDispatchContext,
};
use super::{map_this_object, set_this_object};
use crate::BuiltinInvocation;
use lyng_js_objects::OrdinaryObjectData;
use lyng_js_types::{PropertyKey, Value};

pub(super) fn map_for_each_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = map_this_object(cx, invocation.this_value())?;
    let callback = cx.require_callable_object(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let this_arg = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let self_value = Value::from_object_ref(object);
    let mut index = 0_usize;
    loop {
        let next = {
            let agent = cx.agent();
            let Some(map) = agent.objects().map(object) else {
                return Err(type_error(cx));
            };
            map.entries().get(index).copied()
        };
        let Some(next) = next else {
            break;
        };
        index = index.saturating_add(1);
        let Some(entry) = next else {
            continue;
        };
        let arguments = [entry.value(), entry.key(), self_value];
        let _ = cx.call_to_completion(callback, this_arg, &arguments)?;
    }
    Ok(Value::undefined())
}

pub(super) fn set_for_each_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = set_this_object(cx, invocation.this_value())?;
    let callback = cx.require_callable_object(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let this_arg = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let self_value = Value::from_object_ref(object);
    let mut index = 0_usize;
    loop {
        let next = {
            let agent = cx.agent();
            let Some(set) = agent.objects().set_object_data(object) else {
                return Err(type_error(cx));
            };
            set.entries().get(index).copied()
        };
        let Some(next) = next else {
            break;
        };
        index = index.saturating_add(1);
        let Some(entry) = next else {
            continue;
        };
        let arguments = [entry, entry, self_value];
        let _ = cx.call_to_completion(callback, this_arg, &arguments)?;
    }
    Ok(Value::undefined())
}

pub(super) fn map_iterator_factory_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: ArrayIterationKind,
) -> Result<Value, Cx::Error> {
    let object_ref = map_this_object(cx, invocation.this_value())?;
    let prototype = {
        let realm = cx.builtin_realm();
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().map_iterator_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let slot_values = [
        Value::from_object_ref(object_ref),
        Value::from_smi(0),
        kind.into_value(),
    ];
    let iterator_object =
        allocate_iterator_object(cx, prototype, OrdinaryObjectData::MapIterator, &slot_values)?;
    Ok(Value::from_object_ref(iterator_object))
}

pub(super) fn set_iterator_factory_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: ArrayIterationKind,
) -> Result<Value, Cx::Error> {
    let object_ref = set_this_object(cx, invocation.this_value())?;
    let prototype = {
        let realm = cx.builtin_realm();
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().set_iterator_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let slot_values = [
        Value::from_object_ref(object_ref),
        Value::from_smi(0),
        kind.into_value(),
    ];
    let iterator_object =
        allocate_iterator_object(cx, prototype, OrdinaryObjectData::SetIterator, &slot_values)?;
    Ok(Value::from_object_ref(iterator_object))
}

pub(super) fn map_iterator_next_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let iterator_object = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let target = iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::MapIterator,
        MAP_ITERATOR_TARGET_SLOT,
    )?;
    let Some(target_object) = target.as_object_ref() else {
        return create_iterator_result_value(cx, Value::undefined(), true);
    };
    let index = iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::MapIterator,
        MAP_ITERATOR_INDEX_SLOT,
    )?
    .as_smi()
    .and_then(|value| usize::try_from(value).ok())
    .ok_or_else(|| type_error(cx))?;
    let kind = ArrayIterationKind::from_value(iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::MapIterator,
        MAP_ITERATOR_KIND_SLOT,
    )?)
    .ok_or_else(|| type_error(cx))?;
    let next_entry = {
        let agent = cx.agent();
        let Some(map) = agent.objects().map(target_object) else {
            return Err(type_error(cx));
        };
        map.entries()
            .iter()
            .enumerate()
            .skip(index)
            .find_map(|(entry_index, entry)| entry.map(|entry| (entry_index, entry)))
    };
    let Some((entry_index, entry)) = next_entry else {
        set_iterator_slot_value_for_builtin(
            cx,
            iterator_object,
            OrdinaryObjectData::MapIterator,
            MAP_ITERATOR_TARGET_SLOT,
            Value::undefined(),
        )?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    };
    set_iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::MapIterator,
        MAP_ITERATOR_INDEX_SLOT,
        length_value(u32::try_from(entry_index.saturating_add(1)).unwrap_or(u32::MAX)),
    )?;
    let value = match kind {
        ArrayIterationKind::Key => entry.key(),
        ArrayIterationKind::Value => entry.value(),
        ArrayIterationKind::Entry => {
            let pair = create_array_result(cx, 2)?;
            set_property_on_object(cx, pair, PropertyKey::Index(0), entry.key())?;
            set_property_on_object(cx, pair, PropertyKey::Index(1), entry.value())?;
            Value::from_object_ref(pair)
        }
    };
    create_iterator_result_value(cx, value, false)
}

pub(super) fn set_iterator_next_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let iterator_object = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let target = iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::SetIterator,
        SET_ITERATOR_TARGET_SLOT,
    )?;
    let Some(target_object) = target.as_object_ref() else {
        return create_iterator_result_value(cx, Value::undefined(), true);
    };
    let index = iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::SetIterator,
        SET_ITERATOR_INDEX_SLOT,
    )?
    .as_smi()
    .and_then(|value| usize::try_from(value).ok())
    .ok_or_else(|| type_error(cx))?;
    let kind = ArrayIterationKind::from_value(iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::SetIterator,
        SET_ITERATOR_KIND_SLOT,
    )?)
    .ok_or_else(|| type_error(cx))?;
    let next_entry = {
        let agent = cx.agent();
        let Some(set) = agent.objects().set_object_data(target_object) else {
            return Err(type_error(cx));
        };
        set.entries()
            .iter()
            .enumerate()
            .skip(index)
            .find_map(|(entry_index, entry)| entry.map(|entry| (entry_index, entry)))
    };
    let Some((entry_index, entry)) = next_entry else {
        set_iterator_slot_value_for_builtin(
            cx,
            iterator_object,
            OrdinaryObjectData::SetIterator,
            SET_ITERATOR_TARGET_SLOT,
            Value::undefined(),
        )?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    };
    set_iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::SetIterator,
        SET_ITERATOR_INDEX_SLOT,
        length_value(u32::try_from(entry_index.saturating_add(1)).unwrap_or(u32::MAX)),
    )?;
    let value = match kind {
        ArrayIterationKind::Key | ArrayIterationKind::Value => entry,
        ArrayIterationKind::Entry => {
            let pair = create_array_result(cx, 2)?;
            set_property_on_object(cx, pair, PropertyKey::Index(0), entry)?;
            set_property_on_object(cx, pair, PropertyKey::Index(1), entry)?;
            Value::from_object_ref(pair)
        }
    };
    create_iterator_result_value(cx, value, false)
}

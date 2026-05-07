use super::{
    binary_data::{typed_array_storage_bits_from_builtin_value, typed_array_write_storage_bits},
    close_iterator_after_error, create_array_from_values, create_array_result,
    create_data_property_or_throw, get_property_from_object, is_array_for_species,
    length_value_u64, map_completion, property_key_string_value, proxy_define_property,
    proxy_get_own_property, proxy_get_prototype_of, proxy_is_extensible, proxy_own_property_keys,
    proxy_prevent_extensions, proxy_set_prototype_of, set_property_on_object_with_receiver,
    string_value, type_error, typed_array_index_is_valid, BuiltinIteratorBridge,
    PublicBuiltinDispatchContext, MAX_SAFE_INTEGER_U64,
};
use crate::BuiltinInvocation;
use lyng_js_common::WellKnownAtom;
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::PrimitiveWrapperKind;
use lyng_js_ops::{iterator, read};
use lyng_js_types::{
    BuiltinFunctionId, ObjectRef, PropertyDescriptor, PropertyKey, Value, WellKnownSymbolId,
};

pub(super) fn dispatch_object_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_object_constructor_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_object_prototype_builtin(context, entry, invocation)
}

fn dispatch_object_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::object_builtin() {
        return object_builtin(context, invocation).map(Some);
    }
    if entry == super::object_create_builtin() {
        return object_create_builtin(context, invocation).map(Some);
    }
    if entry == super::object_get_prototype_of_builtin() {
        return object_get_prototype_of_builtin(context, invocation).map(Some);
    }
    if entry == super::object_set_prototype_of_builtin() {
        return object_set_prototype_of_builtin(context, invocation).map(Some);
    }
    if entry == super::object_get_own_property_descriptor_builtin() {
        return object_get_own_property_descriptor_builtin(context, invocation).map(Some);
    }
    if entry == super::object_get_own_property_descriptors_builtin() {
        return object_get_own_property_descriptors_builtin(context, invocation).map(Some);
    }
    if entry == super::object_get_own_property_names_builtin() {
        return object_get_own_property_names_builtin(context, invocation).map(Some);
    }
    if entry == super::object_get_own_property_symbols_builtin() {
        return object_get_own_property_symbols_builtin(context, invocation).map(Some);
    }
    if entry == super::object_define_properties_builtin() {
        return object_define_properties_builtin(context, invocation).map(Some);
    }
    if entry == super::object_define_property_builtin() {
        return object_define_property_builtin(context, invocation).map(Some);
    }
    if entry == super::object_assign_builtin() {
        return object_assign_builtin(context, invocation).map(Some);
    }
    if entry == super::object_from_entries_builtin() {
        return object_from_entries_builtin(context, invocation).map(Some);
    }
    if entry == super::object_group_by_builtin() {
        return object_group_by_builtin(context, invocation).map(Some);
    }
    if entry == super::object_prevent_extensions_builtin() {
        return object_prevent_extensions_builtin(context, invocation).map(Some);
    }
    if entry == super::object_is_extensible_builtin() {
        return object_is_extensible_builtin(context, invocation).map(Some);
    }
    if entry == super::object_is_builtin() {
        return object_is_builtin(context, invocation).map(Some);
    }
    if entry == super::object_seal_builtin() {
        return object_seal_builtin(context, invocation).map(Some);
    }
    if entry == super::object_freeze_builtin() {
        return object_freeze_builtin(context, invocation).map(Some);
    }
    if entry == super::object_is_sealed_builtin() {
        return object_is_sealed_builtin(context, invocation).map(Some);
    }
    if entry == super::object_is_frozen_builtin() {
        return object_is_frozen_builtin(context, invocation).map(Some);
    }
    if entry == super::object_keys_builtin() {
        return object_keys_builtin(context, invocation).map(Some);
    }
    if entry == super::object_entries_builtin() {
        return object_entries_builtin(context, invocation).map(Some);
    }
    if entry == super::object_values_builtin() {
        return object_values_builtin(context, invocation).map(Some);
    }
    if entry == super::object_has_own_builtin() {
        return object_has_own_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_object_prototype_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::object_to_locale_string_builtin() {
        return object_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == super::object_to_string_builtin() {
        return object_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::object_value_of_builtin() {
        return object_value_of_builtin(context, invocation).map(Some);
    }
    if entry == super::object_has_own_property_builtin() {
        return object_has_own_property_builtin(context, invocation).map(Some);
    }
    if entry == super::object_is_prototype_of_builtin() {
        return object_is_prototype_of_builtin(context, invocation).map(Some);
    }
    if entry == super::object_property_is_enumerable_builtin() {
        return object_property_is_enumerable_builtin(context, invocation).map(Some);
    }
    if entry == super::object_define_getter_builtin() {
        return object_define_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::object_define_setter_builtin() {
        return object_define_setter_builtin(context, invocation).map(Some);
    }
    if entry == super::object_lookup_getter_builtin() {
        return object_lookup_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::object_lookup_setter_builtin() {
        return object_lookup_setter_builtin(context, invocation).map(Some);
    }
    if entry == super::object_proto_getter_builtin() {
        return object_proto_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::object_proto_setter_builtin() {
        return object_proto_setter_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn is_error_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
) -> bool {
    let agent = cx.agent();
    agent
        .objects()
        .object_header(agent.heap().view(), object_ref)
        .is_some_and(|header| header.flags().is_error_object())
}

fn object_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let argument = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let realm = cx.builtin_realm();
    if let Some(new_target) = invocation.new_target()
        && new_target != cx.callee_object()
    {
        let default_prototype = {
            let agent = cx.agent();
            agent
                .realm(realm)
                .and_then(|record| record.intrinsics().object_prototype())
        }
        .ok_or_else(|| type_error(cx))?;
        let prototype =
            cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
        let object = cx.allocate_ordinary_object_with_prototype(realm, Some(prototype))?;
        return Ok(Value::from_object_ref(object));
    }
    if let Some(object) = argument.as_object_ref() {
        return Ok(Value::from_object_ref(object));
    }
    if argument.is_null() || argument.is_undefined() {
        let default_prototype = {
            let agent = cx.agent();
            agent
                .realm(realm)
                .and_then(|record| record.intrinsics().object_prototype())
        }
        .ok_or_else(|| type_error(cx))?;
        let prototype =
            cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?;
        let object = cx.allocate_ordinary_object_with_prototype(realm, Some(prototype))?;
        return Ok(Value::from_object_ref(object));
    }
    Ok(Value::from_object_ref(
        cx.to_object_for_builtin_value(realm, argument)?,
    ))
}

fn object_create_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let prototype_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let prototype = if prototype_value.is_null() {
        None
    } else if let Some(object) = prototype_value.as_object_ref() {
        Some(object)
    } else {
        return Err(type_error(cx));
    };
    let object = cx.allocate_ordinary_object_with_prototype(cx.builtin_realm(), prototype)?;
    if let Some(properties) = invocation.arguments().get(1).copied()
        && !properties.is_undefined()
    {
        define_properties_from_source(cx, object, properties)?;
    }
    Ok(Value::from_object_ref(object))
}

fn define_properties_from_source<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    target: lyng_js_types::ObjectRef,
    properties: Value,
) -> Result<(), Cx::Error> {
    let props = cx.to_object_for_builtin_value(cx.builtin_realm(), properties)?;
    let keys = { proxy_own_property_keys(cx, props) };
    let keys = keys?;
    let mut descriptors = Vec::with_capacity(keys.len());

    for key in keys {
        let property = { proxy_get_own_property(cx, props, key) };
        let Some(property) = property? else {
            continue;
        };
        if property.enumerable() != Some(true) {
            continue;
        }

        let descriptor_value = cx.get_property_value(Value::from_object_ref(props), key)?;
        let descriptor_object = descriptor_value
            .as_object_ref()
            .ok_or_else(|| type_error(cx))?;
        let descriptor = cx.to_property_descriptor(descriptor_object)?;
        descriptors.push((key, descriptor));
    }

    for (key, descriptor) in descriptors {
        define_property_or_throw_builtin(cx, target, key, descriptor)?;
    }

    Ok(())
}

fn define_property_or_throw_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    target: lyng_js_types::ObjectRef,
    key: PropertyKey,
    descriptor: PropertyDescriptor,
) -> Result<(), Cx::Error> {
    if let Some(index) = key.as_index() {
        let typed_array = cx.agent().objects().typed_array(target);
        if let Some(record) = typed_array {
            let element_index = usize::try_from(index).unwrap_or(usize::MAX);
            if !typed_array_index_is_valid(cx, record, element_index)?
                || descriptor.has_get()
                || descriptor.has_set()
                || descriptor.configurable() == Some(false)
                || descriptor.enumerable() == Some(false)
                || descriptor.writable() == Some(false)
            {
                return Err(type_error(cx));
            }
            if let Some(value) = descriptor.value() {
                let bits = typed_array_storage_bits_from_builtin_value(cx, record.kind(), value)?;
                let record = cx
                    .agent()
                    .objects()
                    .typed_array(target)
                    .ok_or_else(|| type_error(cx))?;
                if !typed_array_index_is_valid(cx, record, element_index)? {
                    return Err(type_error(cx));
                }
                typed_array_write_storage_bits(cx, record, element_index, bits)?;
            }
            return Ok(());
        }
    }
    let defined =
        { proxy_define_property(cx, target, key, descriptor, AllocationLifetime::Default) };
    if !defined? {
        return Err(type_error(cx));
    }
    Ok(())
}

fn object_get_prototype_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let object = cx.to_object_for_builtin_value(cx.builtin_realm(), value)?;
    Ok(proxy_get_prototype_of(cx, object)?.map_or(Value::null(), Value::from_object_ref))
}

fn object_set_prototype_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    if value.is_undefined() || value.is_null() {
        return Err(type_error(cx));
    }
    let prototype_value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let prototype = if prototype_value.is_null() {
        None
    } else if let Some(object) = prototype_value.as_object_ref() {
        Some(object)
    } else {
        return Err(type_error(cx));
    };
    let Some(object) = value.as_object_ref() else {
        return Ok(value);
    };
    if !proxy_set_prototype_of(cx, object, prototype)? {
        return Err(type_error(cx));
    }
    Ok(Value::from_object_ref(object))
}

fn object_get_own_property_descriptor_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), target_value)?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let Some(descriptor) = proxy_get_own_property(cx, object_ref, key)? else {
        return Ok(Value::undefined());
    };
    cx.descriptor_object_from_descriptor(cx.builtin_realm(), descriptor)
}

fn object_get_own_property_descriptors_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let realm = cx.builtin_realm();
    let object_ref = cx.to_object_for_builtin_value(realm, target_value)?;
    let object_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|realm| realm.intrinsics().object_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let result = cx.allocate_ordinary_object_with_prototype(realm, Some(object_prototype))?;
    let keys = proxy_own_property_keys(cx, object_ref)?;

    for key in keys {
        let Some(descriptor) = proxy_get_own_property(cx, object_ref, key)? else {
            continue;
        };
        let descriptor_value = cx.descriptor_object_from_descriptor(realm, descriptor)?;
        create_data_property_or_throw(cx, result, key, descriptor_value)?;
    }

    Ok(Value::from_object_ref(result))
}

fn own_property_name_list_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    enumerable_only: bool,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), value)?;
    let keys = proxy_own_property_keys(cx, object_ref)?;
    let mut names = Vec::with_capacity(keys.len());

    for key in keys {
        if key.is_symbol() {
            continue;
        }
        if enumerable_only {
            let Some(descriptor) = proxy_get_own_property(cx, object_ref, key)? else {
                continue;
            };
            if descriptor.enumerable() != Some(true) {
                continue;
            }
        }
        names.push(key);
    }

    let result = create_array_result(cx, names.len())?;
    for (index, key) in names.into_iter().enumerate() {
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        let value = property_key_string_value(cx, key);
        create_data_property_or_throw(cx, result, PropertyKey::Index(index), value)?;
    }
    Ok(Value::from_object_ref(result))
}

fn object_get_own_property_names_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    own_property_name_list_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
        false,
    )
}

fn object_get_own_property_symbols_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(
        cx.builtin_realm(),
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let keys = proxy_own_property_keys(cx, object_ref)?;
    let symbols: Vec<_> = keys
        .into_iter()
        .filter_map(PropertyKey::as_symbol)
        .collect();

    let result = create_array_result(cx, symbols.len())?;
    for (index, symbol) in symbols.into_iter().enumerate() {
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        create_data_property_or_throw(
            cx,
            result,
            PropertyKey::Index(index),
            Value::from_symbol_ref(symbol),
        )?;
    }
    Ok(Value::from_object_ref(result))
}

fn object_define_property_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let object_ref = target_value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let descriptor_object = invocation
        .arguments()
        .get(2)
        .copied()
        .and_then(Value::as_object_ref)
        .ok_or_else(|| type_error(cx))?;
    let descriptor = cx.to_property_descriptor(descriptor_object)?;
    define_property_or_throw_builtin(cx, object_ref, key, descriptor)?;
    Ok(Value::from_object_ref(object_ref))
}

fn object_define_properties_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let object_ref = target_value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let properties = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    define_properties_from_source(cx, object_ref, properties)?;
    Ok(Value::from_object_ref(object_ref))
}

fn object_assign_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let target_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let target = cx.to_object_for_builtin_value(cx.builtin_realm(), target_value)?;
    let target_receiver = Value::from_object_ref(target);

    for source in invocation.arguments().iter().copied().skip(1) {
        if source.is_undefined() || source.is_null() {
            continue;
        }
        let source = cx.to_object_for_builtin_value(cx.builtin_realm(), source)?;
        let keys = proxy_own_property_keys(cx, source)?;

        for key in keys {
            let Some(descriptor) = proxy_get_own_property(cx, source, key)? else {
                continue;
            };
            if descriptor.enumerable() != Some(true) {
                continue;
            }
            let value = cx.get_property_value(Value::from_object_ref(source), key)?;
            if !set_property_on_object_with_receiver(cx, target, key, value, target_receiver)? {
                return Err(type_error(cx));
            }
        }
    }

    Ok(Value::from_object_ref(target))
}

fn add_entries_from_iterable_to_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    iterable: Value,
) -> Result<(), Cx::Error> {
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
        let key = match cx.to_property_key(key) {
            Ok(key) => key,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        if let Err(error) = create_data_property_or_throw(cx, object, key, value) {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    }
}

fn object_from_entries_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let object_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().object_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let object = cx.allocate_ordinary_object_with_prototype(realm, Some(object_prototype))?;
    let iterable = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    add_entries_from_iterable_to_object(cx, object, iterable)?;
    Ok(Value::from_object_ref(object))
}

fn add_value_to_keyed_group(
    groups: &mut Vec<(PropertyKey, Vec<Value>)>,
    key: PropertyKey,
    value: Value,
) {
    if let Some((_, values)) = groups
        .iter_mut()
        .find(|(existing_key, _)| *existing_key == key)
    {
        values.push(value);
        return;
    }
    groups.push((key, vec![value]));
}

fn object_group_by_builtin<Cx: PublicBuiltinDispatchContext>(
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
        let key = match cx.call_to_completion(
            callback,
            Value::undefined(),
            &[value, length_value_u64(index)],
        ) {
            Ok(key) => key,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let key = match cx.to_property_key(key) {
            Ok(key) => key,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        add_value_to_keyed_group(&mut groups, key, value);
        index += 1;
    }

    let result = cx.allocate_ordinary_object_with_prototype(cx.builtin_realm(), None)?;
    for (key, values) in groups {
        let array = create_array_from_values(cx, &values)?;
        create_data_property_or_throw(cx, result, key, Value::from_object_ref(array))?;
    }
    Ok(Value::from_object_ref(result))
}

fn object_prevent_extensions_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(object_ref) = value.as_object_ref() else {
        return Ok(value);
    };
    if !proxy_prevent_extensions(cx, object_ref)? {
        return Err(type_error(cx));
    }
    Ok(Value::from_object_ref(object_ref))
}

fn object_is_extensible_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(object_ref) = value.as_object_ref() else {
        return Ok(Value::from_bool(false));
    };
    Ok(Value::from_bool(proxy_is_extensible(cx, object_ref)?))
}

fn object_is_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let right = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let same = {
        let agent = cx.agent();
        read::same_value(agent.heap().view(), left, right)
    };
    Ok(Value::from_bool(map_completion(cx, same)?))
}

fn object_seal_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(object_ref) = value.as_object_ref() else {
        return Ok(value);
    };
    if !cx.set_integrity_level(object_ref, false)? {
        return Err(type_error(cx));
    }
    Ok(Value::from_object_ref(object_ref))
}

fn object_freeze_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(object_ref) = value.as_object_ref() else {
        return Ok(value);
    };
    if !cx.set_integrity_level(object_ref, true)? {
        return Err(type_error(cx));
    }
    Ok(Value::from_object_ref(object_ref))
}

fn object_is_sealed_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(object_ref) = value.as_object_ref() else {
        return Ok(Value::from_bool(true));
    };
    Ok(Value::from_bool(
        cx.test_integrity_level(object_ref, false)?,
    ))
}

fn object_is_frozen_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(object_ref) = value.as_object_ref() else {
        return Ok(Value::from_bool(true));
    };
    Ok(Value::from_bool(cx.test_integrity_level(object_ref, true)?))
}

fn object_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Ok(Value::from_object_ref(cx.to_object_for_builtin_value(
        cx.builtin_realm(),
        invocation.this_value(),
    )?))
}

fn object_to_locale_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.this_value().is_undefined() || invocation.this_value().is_null() {
        return Err(type_error(cx));
    }
    let key = PropertyKey::from_atom(WellKnownAtom::toString.id());
    let method_value = cx.get_property_value(invocation.this_value(), key)?;
    let method = cx.require_callable_object(method_value)?;
    cx.call_to_completion(method, invocation.this_value(), &[])
}

fn object_is_prototype_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let Some(mut current) = invocation
        .arguments()
        .first()
        .copied()
        .and_then(Value::as_object_ref)
    else {
        return Ok(Value::from_bool(false));
    };
    let prototype_object =
        cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    loop {
        let Some(next) = proxy_get_prototype_of(cx, current)? else {
            return Ok(Value::from_bool(false));
        };
        if next == prototype_object {
            return Ok(Value::from_bool(true));
        }
        current = next;
    }
}

fn object_property_is_enumerable_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let key = cx.to_property_key(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    Ok(Value::from_bool(
        proxy_get_own_property(cx, object_ref, key)?
            .is_some_and(|descriptor| descriptor.enumerable() == Some(true)),
    ))
}

fn object_define_accessor_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    define_getter: bool,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let callable = invocation
        .arguments()
        .get(1)
        .copied()
        .and_then(Value::as_object_ref)
        .filter(|object| cx.agent().objects().is_callable(*object))
        .ok_or_else(|| type_error(cx))?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;

    let mut descriptor = PropertyDescriptor::new();
    if define_getter {
        descriptor.set_getter(Value::from_object_ref(callable));
    } else {
        descriptor.set_setter(Value::from_object_ref(callable));
    }
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    define_property_or_throw_builtin(cx, object_ref, key, descriptor)?;
    Ok(Value::undefined())
}

fn object_define_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    object_define_accessor_builtin(cx, invocation, true)
}

fn object_define_setter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    object_define_accessor_builtin(cx, invocation, false)
}

fn object_lookup_accessor_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    lookup_getter: bool,
) -> Result<Value, Cx::Error> {
    let mut object_ref =
        cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;

    loop {
        if let Some(descriptor) = proxy_get_own_property(cx, object_ref, key)? {
            if descriptor.has_get() || descriptor.has_set() {
                let accessor = if lookup_getter {
                    descriptor.getter()
                } else {
                    descriptor.setter()
                };
                return Ok(accessor.unwrap_or(Value::undefined()));
            }
            return Ok(Value::undefined());
        }

        let Some(prototype) = proxy_get_prototype_of(cx, object_ref)? else {
            return Ok(Value::undefined());
        };
        object_ref = prototype;
    }
}

fn object_lookup_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    object_lookup_accessor_builtin(cx, invocation, true)
}

fn object_lookup_setter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    object_lookup_accessor_builtin(cx, invocation, false)
}

fn object_proto_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    Ok(proxy_get_prototype_of(cx, object_ref)?.map_or(Value::null(), Value::from_object_ref))
}

fn object_proto_setter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let this_value = invocation.this_value();
    if this_value.is_undefined() || this_value.is_null() {
        return Err(type_error(cx));
    }

    let prototype_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let prototype = if prototype_value.is_null() {
        None
    } else if let Some(object) = prototype_value.as_object_ref() {
        Some(object)
    } else {
        return Ok(Value::undefined());
    };

    let Some(object_ref) = this_value.as_object_ref() else {
        return Ok(Value::undefined());
    };
    if !proxy_set_prototype_of(cx, object_ref, prototype)? {
        return Err(type_error(cx));
    }
    Ok(Value::undefined())
}

fn object_keys_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    own_property_name_list_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
        true,
    )
}

fn object_entries_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(
        cx.builtin_realm(),
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let keys = proxy_own_property_keys(cx, object_ref)?;
    let result = create_array_result(cx, keys.len())?;
    let mut index = 0_u32;

    for key in keys {
        if key.is_symbol() {
            continue;
        }
        let Some(descriptor) = proxy_get_own_property(cx, object_ref, key)? else {
            continue;
        };
        if descriptor.enumerable() != Some(true) {
            continue;
        }
        let entry = create_array_result(cx, 2)?;
        let key_value = property_key_string_value(cx, key);
        let value = get_property_from_object(cx, object_ref, key)?;
        create_data_property_or_throw(cx, entry, PropertyKey::Index(0), key_value)?;
        create_data_property_or_throw(cx, entry, PropertyKey::Index(1), value)?;
        create_data_property_or_throw(
            cx,
            result,
            PropertyKey::Index(index),
            Value::from_object_ref(entry),
        )?;
        index = index.saturating_add(1);
    }
    Ok(Value::from_object_ref(result))
}

fn object_values_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(
        cx.builtin_realm(),
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let keys = proxy_own_property_keys(cx, object_ref)?;
    let result = create_array_result(cx, keys.len())?;
    let mut index = 0_u32;

    for key in keys {
        if key.is_symbol() {
            continue;
        }
        let Some(descriptor) = proxy_get_own_property(cx, object_ref, key)? else {
            continue;
        };
        if descriptor.enumerable() != Some(true) {
            continue;
        }
        let value = get_property_from_object(cx, object_ref, key)?;
        create_data_property_or_throw(cx, result, PropertyKey::Index(index), value)?;
        index = index.saturating_add(1);
    }
    Ok(Value::from_object_ref(result))
}

fn object_has_own_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(
        cx.builtin_realm(),
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let key = cx.to_property_key(
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if let Some(index) = key.as_index()
        && let Some(has_own) = cx.try_fast_has_own_index_property(object_ref, index)?
    {
        return Ok(Value::from_bool(has_own));
    }
    Ok(Value::from_bool(
        proxy_get_own_property(cx, object_ref, key)?.is_some(),
    ))
}

pub(super) fn object_has_own_property_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let key = cx.to_property_key(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    if let Some(index) = key.as_index()
        && let Some(has_own) = cx.try_fast_has_own_index_property(object_ref, index)?
    {
        return Ok(Value::from_bool(has_own));
    }
    Ok(Value::from_bool(
        proxy_get_own_property(cx, object_ref, key)?.is_some(),
    ))
}

pub(super) fn object_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.this_value().is_undefined() {
        return Ok(string_value(cx, "[object Undefined]"));
    }
    if invocation.this_value().is_null() {
        return Ok(string_value(cx, "[object Null]"));
    }
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let default_tag = {
        let is_function = {
            let agent = cx.agent();
            agent.objects().is_callable(object_ref)
        };
        if is_array_for_species(cx, object_ref)? {
            "Array"
        } else if is_function {
            "Function"
        } else {
            let is_date = {
                let agent = cx.agent();
                agent.objects().is_date_object(object_ref)
            };
            let is_regexp = {
                let agent = cx.agent();
                agent.objects().is_regexp_object(object_ref)
            };
            let primitive_wrapper_kind = {
                let agent = cx.agent();
                agent.objects().primitive_wrapper_kind(object_ref)
            };
            let is_arguments = {
                let agent = cx.agent();
                agent
                    .objects()
                    .object_header(agent.heap().view(), object_ref)
                    .is_some_and(|header| header.flags().is_arguments_object())
            };
            if is_date {
                "Date"
            } else if is_regexp {
                "RegExp"
            } else if let Some(kind) = primitive_wrapper_kind {
                match kind {
                    PrimitiveWrapperKind::String => "String",
                    PrimitiveWrapperKind::Number => "Number",
                    PrimitiveWrapperKind::Boolean => "Boolean",
                    PrimitiveWrapperKind::Symbol | PrimitiveWrapperKind::BigInt => "Object",
                }
            } else if is_arguments {
                "Arguments"
            } else if is_error_object(cx, object_ref) {
                "Error"
            } else {
                "Object"
            }
        }
    };
    let to_string_tag = {
        let key = {
            let agent = cx.agent();
            agent
                .well_known_symbol(WellKnownSymbolId::ToStringTag)
                .map(PropertyKey::from_symbol)
        };
        if let Some(key) = key {
            let value = cx.get_property_value(Value::from_object_ref(object_ref), key)?;
            if value.is_string() {
                Some(cx.value_to_string_text(value)?)
            } else {
                None
            }
        } else {
            None
        }
    };
    Ok(string_value(
        cx,
        &format!(
            "[object {}]",
            to_string_tag.as_deref().unwrap_or(default_tag)
        ),
    ))
}

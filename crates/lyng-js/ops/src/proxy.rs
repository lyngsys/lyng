use crate::{object as ordinary_object, read};
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_types::{
    AbruptCompletion, Completion, ObjectRef, PropertyDescriptor, PropertyKey, Value,
};
use std::collections::HashSet;

pub trait ProxyTrapContext {
    type Error;

    fn agent(&mut self) -> &mut Agent;

    fn abrupt(&mut self, completion: AbruptCompletion) -> Self::Error;

    fn type_error(&mut self) -> Self::Error;

    fn get_property_value(
        &mut self,
        receiver: Value,
        key: PropertyKey,
    ) -> Result<Value, Self::Error>;

    fn get_property_from_object_with_receiver(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
        receiver: Value,
    ) -> Result<Value, Self::Error>;

    fn get_own_property_from_object(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> Result<Option<PropertyDescriptor>, Self::Error>;

    fn set_property_on_object_with_receiver(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
        value: Value,
        receiver: Value,
        lifetime: AllocationLifetime,
    ) -> Result<bool, Self::Error>;

    fn define_property_on_object(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
    ) -> Result<bool, Self::Error>;

    fn delete_property_from_object(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> Result<bool, Self::Error>;

    fn call_to_completion(
        &mut self,
        callee_object: ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> Result<Value, Self::Error>;

    fn construct_to_completion(
        &mut self,
        callee_object: ObjectRef,
        arguments: &[Value],
        new_target: Option<ObjectRef>,
    ) -> Result<ObjectRef, Self::Error>;

    fn to_property_key(&mut self, value: Value) -> Result<PropertyKey, Self::Error>;

    fn to_property_descriptor(
        &mut self,
        descriptor_object: ObjectRef,
    ) -> Result<PropertyDescriptor, Self::Error>;

    fn descriptor_object_from_descriptor(
        &mut self,
        descriptor: PropertyDescriptor,
    ) -> Result<Value, Self::Error>;

    fn create_array_from_values(&mut self, values: &[Value]) -> Result<ObjectRef, Self::Error>;
}

pub fn has_property<Cx: ProxyTrapContext>(
    cx: &mut Cx,
    object: ObjectRef,
    key: PropertyKey,
) -> Result<bool, Cx::Error> {
    if !is_proxy(cx, object) {
        if get_own_property(cx, object, key)?.is_some() {
            return Ok(true);
        }
        let Some(prototype) = get_prototype_of(cx, object)? else {
            return Ok(false);
        };
        return has_property(cx, prototype, key);
    }

    let (target, handler) = proxy_target_and_handler(cx, object)?;
    let Some(trap) = trap_method(cx, handler, "has")? else {
        return has_property(cx, target, key);
    };
    let key_value = property_key_value(cx, key);
    let trap_result = call_trap(
        cx,
        trap,
        handler,
        &[Value::from_object_ref(target), key_value],
    )?;
    let trap_result = to_boolean(cx, trap_result)?;
    if !trap_result {
        if let Some(target_descriptor) = get_own_property(cx, target, key)? {
            if target_descriptor.configurable() == Some(false) || !is_extensible(cx, target)? {
                return Err(cx.type_error());
            }
        }
    }
    Ok(trap_result)
}

pub fn get<Cx: ProxyTrapContext>(
    cx: &mut Cx,
    object: ObjectRef,
    key: PropertyKey,
    receiver: Value,
) -> Result<Value, Cx::Error> {
    if !is_proxy(cx, object) {
        return cx.get_property_from_object_with_receiver(object, key, receiver);
    }

    let (target, handler) = proxy_target_and_handler(cx, object)?;
    let Some(trap) = trap_method(cx, handler, "get")? else {
        return get(cx, target, key, receiver);
    };
    let key_value = property_key_value(cx, key);
    let trap_result = call_trap(
        cx,
        trap,
        handler,
        &[Value::from_object_ref(target), key_value, receiver],
    )?;
    if let Some(target_descriptor) = get_own_property(cx, target, key)? {
        if target_descriptor.configurable() == Some(false) {
            if is_data_descriptor(target_descriptor)
                && target_descriptor.writable() == Some(false)
                && !same_value(
                    cx,
                    trap_result,
                    target_descriptor.value().unwrap_or(Value::undefined()),
                )?
            {
                return Err(cx.type_error());
            }
            if is_accessor_descriptor(target_descriptor)
                && target_descriptor.getter() == Some(Value::undefined())
                && !trap_result.is_undefined()
            {
                return Err(cx.type_error());
            }
        }
    }
    Ok(trap_result)
}

pub fn get_prototype_of<Cx: ProxyTrapContext>(
    cx: &mut Cx,
    object: ObjectRef,
) -> Result<Option<ObjectRef>, Cx::Error> {
    if !is_proxy(cx, object) {
        let result = {
            let agent = cx.agent();
            ordinary_object::ordinary_get_prototype_of(agent, object)
        };
        return map_completion(cx, result);
    }

    let (target, handler) = proxy_target_and_handler(cx, object)?;
    let Some(trap) = trap_method(cx, handler, "getPrototypeOf")? else {
        return get_prototype_of(cx, target);
    };
    let trap_result = call_trap(cx, trap, handler, &[Value::from_object_ref(target)])?;
    let prototype = if trap_result.is_null() {
        None
    } else {
        Some(trap_result.as_object_ref().ok_or_else(|| cx.type_error())?)
    };
    let target_prototype = get_prototype_of(cx, target)?;
    if is_extensible(cx, target)? || prototype == target_prototype {
        Ok(prototype)
    } else {
        Err(cx.type_error())
    }
}

pub fn set_prototype_of<Cx: ProxyTrapContext>(
    cx: &mut Cx,
    object: ObjectRef,
    prototype: Option<ObjectRef>,
) -> Result<bool, Cx::Error> {
    if !is_proxy(cx, object) {
        let result = {
            let agent = cx.agent();
            ordinary_object::ordinary_set_prototype_of(agent, object, prototype)
        };
        return map_completion(cx, result);
    }

    let (target, handler) = proxy_target_and_handler(cx, object)?;
    let Some(trap) = trap_method(cx, handler, "setPrototypeOf")? else {
        return set_prototype_of(cx, target, prototype);
    };
    let trap_result = call_trap(
        cx,
        trap,
        handler,
        &[
            Value::from_object_ref(target),
            prototype.map_or(Value::null(), Value::from_object_ref),
        ],
    )?;
    if !to_boolean(cx, trap_result)? {
        return Ok(false);
    }
    if is_extensible(cx, target)? || get_prototype_of(cx, target)? == prototype {
        Ok(true)
    } else {
        Err(cx.type_error())
    }
}

pub fn get_own_property<Cx: ProxyTrapContext>(
    cx: &mut Cx,
    object: ObjectRef,
    key: PropertyKey,
) -> Result<Option<PropertyDescriptor>, Cx::Error> {
    if !is_proxy(cx, object) {
        return cx.get_own_property_from_object(object, key);
    }

    let (target, handler) = proxy_target_and_handler(cx, object)?;
    let Some(trap) = trap_method(cx, handler, "getOwnPropertyDescriptor")? else {
        return get_own_property(cx, target, key);
    };
    let key_value = property_key_value(cx, key);
    let trap_result = call_trap(
        cx,
        trap,
        handler,
        &[Value::from_object_ref(target), key_value],
    )?;
    let target_descriptor = get_own_property(cx, target, key)?;
    if trap_result.is_undefined() {
        if let Some(target_descriptor) = target_descriptor {
            if target_descriptor.configurable() == Some(false) || !is_extensible(cx, target)? {
                return Err(cx.type_error());
            }
        }
        return Ok(None);
    }

    let descriptor_object = trap_result.as_object_ref().ok_or_else(|| cx.type_error())?;
    let result_descriptor =
        complete_property_descriptor(cx.to_property_descriptor(descriptor_object)?)
            .map_err(|()| cx.type_error())?;
    let extensible_target = is_extensible(cx, target)?;
    if !is_compatible_property_descriptor(extensible_target, result_descriptor, target_descriptor)
        .map_err(|()| cx.type_error())?
    {
        return Err(cx.type_error());
    }
    if result_descriptor.configurable() == Some(false) {
        let Some(target_descriptor) = target_descriptor else {
            return Err(cx.type_error());
        };
        if target_descriptor.configurable() != Some(false) {
            return Err(cx.type_error());
        }
        if is_data_descriptor(result_descriptor)
            && is_data_descriptor(target_descriptor)
            && result_descriptor.writable() == Some(false)
            && target_descriptor.writable() == Some(true)
        {
            return Err(cx.type_error());
        }
    }
    Ok(Some(result_descriptor))
}

pub fn define_property<Cx: ProxyTrapContext>(
    cx: &mut Cx,
    object: ObjectRef,
    key: PropertyKey,
    descriptor: PropertyDescriptor,
    lifetime: AllocationLifetime,
) -> Result<bool, Cx::Error> {
    if !is_proxy(cx, object) {
        return cx.define_property_on_object(object, key, descriptor, lifetime);
    }

    let (target, handler) = proxy_target_and_handler(cx, object)?;
    let Some(trap) = trap_method(cx, handler, "defineProperty")? else {
        return define_property(cx, target, key, descriptor, lifetime);
    };
    let key_value = property_key_value(cx, key);
    let descriptor_object = cx
        .descriptor_object_from_descriptor(descriptor)?
        .as_object_ref()
        .ok_or_else(|| cx.type_error())?;
    let trap_result = call_trap(
        cx,
        trap,
        handler,
        &[
            Value::from_object_ref(target),
            key_value,
            Value::from_object_ref(descriptor_object),
        ],
    )?;
    if !to_boolean(cx, trap_result)? {
        return Ok(false);
    }

    let target_descriptor = get_own_property(cx, target, key)?;
    let extensible_target = is_extensible(cx, target)?;
    if !is_compatible_property_descriptor(extensible_target, descriptor, target_descriptor)
        .map_err(|()| cx.type_error())?
    {
        return Err(cx.type_error());
    }
    if let Some(target_descriptor) = target_descriptor {
        if is_data_descriptor(target_descriptor)
            && target_descriptor.configurable() == Some(false)
            && target_descriptor.writable() == Some(true)
            && descriptor.writable() == Some(false)
        {
            return Err(cx.type_error());
        }
    }
    if descriptor.configurable() == Some(false) {
        let Some(target_descriptor) = target_descriptor else {
            return Err(cx.type_error());
        };
        if target_descriptor.configurable() != Some(false) {
            return Err(cx.type_error());
        }
        if is_data_descriptor(descriptor)
            && descriptor.writable() == Some(false)
            && is_data_descriptor(target_descriptor)
            && target_descriptor.writable() == Some(true)
        {
            return Err(cx.type_error());
        }
    }
    Ok(true)
}

pub fn is_extensible<Cx: ProxyTrapContext>(
    cx: &mut Cx,
    object: ObjectRef,
) -> Result<bool, Cx::Error> {
    if !is_proxy(cx, object) {
        let result = {
            let agent = cx.agent();
            ordinary_object::ordinary_is_extensible(agent, object)
        };
        return map_completion(cx, result);
    }

    let (target, handler) = proxy_target_and_handler(cx, object)?;
    let Some(trap) = trap_method(cx, handler, "isExtensible")? else {
        return is_extensible(cx, target);
    };
    let trap_result = call_trap(cx, trap, handler, &[Value::from_object_ref(target)])?;
    let trap_result = to_boolean(cx, trap_result)?;
    if trap_result == is_extensible(cx, target)? {
        Ok(trap_result)
    } else {
        Err(cx.type_error())
    }
}

pub fn prevent_extensions<Cx: ProxyTrapContext>(
    cx: &mut Cx,
    object: ObjectRef,
) -> Result<bool, Cx::Error> {
    if !is_proxy(cx, object) {
        let result = {
            let agent = cx.agent();
            ordinary_object::ordinary_prevent_extensions(agent, object)
        };
        return map_completion(cx, result);
    }

    let (target, handler) = proxy_target_and_handler(cx, object)?;
    let Some(trap) = trap_method(cx, handler, "preventExtensions")? else {
        return prevent_extensions(cx, target);
    };
    let trap_result = call_trap(cx, trap, handler, &[Value::from_object_ref(target)])?;
    let trap_result = to_boolean(cx, trap_result)?;
    if !trap_result {
        return Ok(false);
    }
    if is_extensible(cx, target)? {
        Err(cx.type_error())
    } else {
        Ok(true)
    }
}

pub fn own_property_keys<Cx: ProxyTrapContext>(
    cx: &mut Cx,
    object: ObjectRef,
) -> Result<Vec<PropertyKey>, Cx::Error> {
    if !is_proxy(cx, object) {
        let result = {
            let agent = cx.agent();
            ordinary_object::ordinary_own_property_keys(agent, object)
        };
        return map_completion(cx, result);
    }

    let (target, handler) = proxy_target_and_handler(cx, object)?;
    let Some(trap) = trap_method(cx, handler, "ownKeys")? else {
        return own_property_keys(cx, target);
    };
    let trap_result = call_trap(cx, trap, handler, &[Value::from_object_ref(target)])?;
    let trap_result = trap_result.as_object_ref().ok_or_else(|| cx.type_error())?;
    let trap_keys = create_list_from_array_like_keys(cx, trap_result)?;
    let target_keys = own_property_keys(cx, target)?;
    let mut non_configurable = Vec::new();
    let mut configurable = Vec::new();
    for key in &target_keys {
        let descriptor = get_own_property(cx, target, *key)?.ok_or_else(|| cx.type_error())?;
        if descriptor.configurable() == Some(false) {
            non_configurable.push(*key);
        } else {
            configurable.push(*key);
        }
    }
    if is_extensible(cx, target)? && non_configurable.is_empty() {
        return Ok(trap_keys);
    }

    let mut remaining: HashSet<PropertyKey> = trap_keys.iter().copied().collect();
    for key in non_configurable {
        if !remaining.remove(&key) {
            return Err(cx.type_error());
        }
    }
    if is_extensible(cx, target)? {
        return Ok(trap_keys);
    }
    for key in configurable {
        if !remaining.remove(&key) {
            return Err(cx.type_error());
        }
    }
    if remaining.is_empty() {
        Ok(trap_keys)
    } else {
        Err(cx.type_error())
    }
}

pub fn set<Cx: ProxyTrapContext>(
    cx: &mut Cx,
    object: ObjectRef,
    key: PropertyKey,
    value: Value,
    receiver: Value,
    lifetime: AllocationLifetime,
) -> Result<bool, Cx::Error> {
    if !is_proxy(cx, object) {
        return cx.set_property_on_object_with_receiver(object, key, value, receiver, lifetime);
    }

    let (target, handler) = proxy_target_and_handler(cx, object)?;
    let Some(trap) = trap_method(cx, handler, "set")? else {
        return set(cx, target, key, value, receiver, lifetime);
    };
    let key_value = property_key_value(cx, key);
    let trap_result = call_trap(
        cx,
        trap,
        handler,
        &[Value::from_object_ref(target), key_value, value, receiver],
    )?;
    if !to_boolean(cx, trap_result)? {
        return Ok(false);
    }

    if let Some(target_descriptor) = get_own_property(cx, target, key)? {
        if target_descriptor.configurable() == Some(false) {
            if is_data_descriptor(target_descriptor)
                && target_descriptor.writable() == Some(false)
                && !same_value(
                    cx,
                    value,
                    target_descriptor.value().unwrap_or(Value::undefined()),
                )?
            {
                return Err(cx.type_error());
            }
            if is_accessor_descriptor(target_descriptor)
                && target_descriptor.setter() == Some(Value::undefined())
            {
                return Err(cx.type_error());
            }
        }
    }
    Ok(true)
}

pub fn delete_property<Cx: ProxyTrapContext>(
    cx: &mut Cx,
    object: ObjectRef,
    key: PropertyKey,
) -> Result<bool, Cx::Error> {
    if !is_proxy(cx, object) {
        return cx.delete_property_from_object(object, key);
    }

    let (target, handler) = proxy_target_and_handler(cx, object)?;
    let Some(trap) = trap_method(cx, handler, "deleteProperty")? else {
        return delete_property(cx, target, key);
    };
    let key_value = property_key_value(cx, key);
    let trap_result = call_trap(
        cx,
        trap,
        handler,
        &[Value::from_object_ref(target), key_value],
    )?;
    if !to_boolean(cx, trap_result)? {
        return Ok(false);
    }
    if let Some(target_descriptor) = get_own_property(cx, target, key)? {
        if target_descriptor.configurable() == Some(false) {
            return Err(cx.type_error());
        }
        if !is_extensible(cx, target)? {
            return Err(cx.type_error());
        }
    }
    Ok(true)
}

pub fn call<Cx: ProxyTrapContext>(
    cx: &mut Cx,
    callee: ObjectRef,
    this_value: Value,
    arguments: &[Value],
) -> Result<Value, Cx::Error> {
    if !is_proxy(cx, callee) {
        return cx.call_to_completion(callee, this_value, arguments);
    }

    let (target, handler) = proxy_target_and_handler(cx, callee)?;
    let Some(trap) = trap_method(cx, handler, "apply")? else {
        return call(cx, target, this_value, arguments);
    };
    let arg_array = cx.create_array_from_values(arguments)?;
    call_trap(
        cx,
        trap,
        handler,
        &[
            Value::from_object_ref(target),
            this_value,
            Value::from_object_ref(arg_array),
        ],
    )
}

pub fn construct<Cx: ProxyTrapContext>(
    cx: &mut Cx,
    callee: ObjectRef,
    arguments: &[Value],
    new_target: Option<ObjectRef>,
) -> Result<ObjectRef, Cx::Error> {
    if !is_proxy(cx, callee) {
        return cx.construct_to_completion(callee, arguments, new_target);
    }

    let (target, handler) = proxy_target_and_handler(cx, callee)?;
    let Some(trap) = trap_method(cx, handler, "construct")? else {
        return construct(cx, target, arguments, new_target);
    };
    let arg_array = cx.create_array_from_values(arguments)?;
    let result = call_trap(
        cx,
        trap,
        handler,
        &[
            Value::from_object_ref(target),
            Value::from_object_ref(arg_array),
            Value::from_object_ref(new_target.unwrap_or(callee)),
        ],
    )?;
    result.as_object_ref().ok_or_else(|| cx.type_error())
}

fn create_list_from_array_like_keys<Cx: ProxyTrapContext>(
    cx: &mut Cx,
    object: ObjectRef,
) -> Result<Vec<PropertyKey>, Cx::Error> {
    let length_value = cx.get_property_value(
        Value::from_object_ref(object),
        PropertyKey::from_atom(lyng_js_common::WellKnownAtom::length.id()),
    )?;
    let length_result = {
        let view = cx.agent().heap().view();
        read::to_number(view, length_value)
    };
    let length_number = map_completion(cx, length_result)?;
    let length = to_length(length_number.as_f64().unwrap_or(0.0));
    let mut seen = HashSet::new();
    let mut keys = Vec::with_capacity(usize::try_from(length).unwrap_or(usize::MAX));
    for index in 0..length {
        let value =
            cx.get_property_value(Value::from_object_ref(object), array_like_index_key(index))?;
        let key = if value.as_string_ref().is_some() || value.as_symbol_ref().is_some() {
            cx.to_property_key(value)?
        } else {
            return Err(cx.type_error());
        };
        if !seen.insert(key) {
            return Err(cx.type_error());
        }
        keys.push(key);
    }
    Ok(keys)
}

fn array_like_index_key(index: u64) -> PropertyKey {
    PropertyKey::from_array_index(index).unwrap_or_else(|| {
        let atom = lyng_js_common::AtomId::from_raw(0);
        let _ = atom;
        panic!("proxy ownKeys trap index should stay within u32")
    })
}

fn property_key_value<Cx: ProxyTrapContext>(cx: &mut Cx, key: PropertyKey) -> Value {
    match key {
        PropertyKey::Index(index) => {
            let text = index.to_string();
            let string = cx
                .agent()
                .alloc_runtime_string(&text, None, AllocationLifetime::Default);
            Value::from_string_ref(string)
        }
        PropertyKey::Atom(atom) => {
            let string =
                cx.agent()
                    .alloc_runtime_string("", Some(atom), AllocationLifetime::Default);
            Value::from_string_ref(string)
        }
        PropertyKey::Symbol(symbol) => Value::from_symbol_ref(symbol),
    }
}

fn proxy_target_and_handler<Cx: ProxyTrapContext>(
    cx: &mut Cx,
    proxy: ObjectRef,
) -> Result<(ObjectRef, ObjectRef), Cx::Error> {
    let Some(data) = cx.agent().objects().proxy_data(proxy) else {
        return Err(cx.type_error());
    };
    if data.revoked() {
        return Err(cx.type_error());
    }
    let Some(handler) = data.handler() else {
        return Err(cx.type_error());
    };
    Ok((data.target(), handler))
}

fn trap_method<Cx: ProxyTrapContext>(
    cx: &mut Cx,
    handler: ObjectRef,
    name: &str,
) -> Result<Option<ObjectRef>, Cx::Error> {
    let trap_key = {
        let atom = cx.agent().atoms_mut().intern_collectible(name);
        PropertyKey::from_atom(atom)
    };
    let trap = cx.get_property_value(Value::from_object_ref(handler), trap_key)?;
    if trap.is_undefined() || trap.is_null() {
        return Ok(None);
    }
    let trap = trap.as_object_ref().ok_or_else(|| cx.type_error())?;
    if !cx.agent().objects().is_callable(trap) {
        return Err(cx.type_error());
    }
    Ok(Some(trap))
}

fn call_trap<Cx: ProxyTrapContext>(
    cx: &mut Cx,
    trap: ObjectRef,
    handler: ObjectRef,
    arguments: &[Value],
) -> Result<Value, Cx::Error> {
    cx.call_to_completion(trap, Value::from_object_ref(handler), arguments)
}

fn to_boolean<Cx: ProxyTrapContext>(cx: &mut Cx, value: Value) -> Result<bool, Cx::Error> {
    let result = {
        let view = cx.agent().heap().view();
        read::to_boolean(view, value)
    };
    map_completion(cx, result)
}

fn same_value<Cx: ProxyTrapContext>(
    cx: &mut Cx,
    left: Value,
    right: Value,
) -> Result<bool, Cx::Error> {
    let result = {
        let view = cx.agent().heap().view();
        read::same_value(view, left, right)
    };
    map_completion(cx, result)
}

fn map_completion<Cx: ProxyTrapContext, T>(
    cx: &mut Cx,
    result: Completion<T>,
) -> Result<T, Cx::Error> {
    result.map_err(|completion| cx.abrupt(completion))
}

fn is_proxy<Cx: ProxyTrapContext>(cx: &mut Cx, object: ObjectRef) -> bool {
    cx.agent().objects().is_proxy_object(object)
}

fn descriptor_kind(descriptor: PropertyDescriptor) -> Result<DescriptorKind, ()> {
    let is_data = descriptor.has_value() || descriptor.has_writable();
    let is_accessor = descriptor.has_get() || descriptor.has_set();
    match (is_data, is_accessor) {
        (true, true) => Err(()),
        (true, false) => Ok(DescriptorKind::Data),
        (false, true) => Ok(DescriptorKind::Accessor),
        (false, false) => Ok(DescriptorKind::Generic),
    }
}

fn is_data_descriptor(descriptor: PropertyDescriptor) -> bool {
    matches!(descriptor_kind(descriptor), Ok(DescriptorKind::Data))
}

fn is_accessor_descriptor(descriptor: PropertyDescriptor) -> bool {
    matches!(descriptor_kind(descriptor), Ok(DescriptorKind::Accessor))
}

fn complete_property_descriptor(
    mut descriptor: PropertyDescriptor,
) -> Result<PropertyDescriptor, ()> {
    let kind = descriptor_kind(descriptor)?;
    match kind {
        DescriptorKind::Generic | DescriptorKind::Data => {
            if !descriptor.has_value() {
                descriptor.set_value(Value::undefined());
            }
            if !descriptor.has_writable() {
                descriptor.set_writable(false);
            }
        }
        DescriptorKind::Accessor => {
            if !descriptor.has_get() {
                descriptor.set_getter(Value::undefined());
            }
            if !descriptor.has_set() {
                descriptor.set_setter(Value::undefined());
            }
        }
    }
    if !descriptor.has_enumerable() {
        descriptor.set_enumerable(false);
    }
    if !descriptor.has_configurable() {
        descriptor.set_configurable(false);
    }
    Ok(descriptor)
}

fn is_compatible_property_descriptor(
    extensible: bool,
    descriptor: PropertyDescriptor,
    current: Option<PropertyDescriptor>,
) -> Result<bool, ()> {
    let Some(current) = current else {
        return Ok(extensible);
    };
    if descriptor.present().is_empty() {
        return Ok(true);
    }

    let current_kind = descriptor_kind(current)?;
    let descriptor_kind = descriptor_kind(descriptor)?;
    if current.configurable() == Some(false) {
        if descriptor.configurable() == Some(true) {
            return Ok(false);
        }
        if let Some(enumerable) = descriptor.enumerable() {
            if Some(enumerable) != current.enumerable() {
                return Ok(false);
            }
        }
    }
    if descriptor_kind == DescriptorKind::Generic {
        return Ok(true);
    }
    if descriptor_kind != current_kind {
        return Ok(current.configurable() == Some(true));
    }
    match descriptor_kind {
        DescriptorKind::Generic => Ok(true),
        DescriptorKind::Data => {
            if current.configurable() == Some(false) && current.writable() == Some(false) {
                if descriptor.writable() == Some(true) {
                    return Ok(false);
                }
                if let Some(value) = descriptor.value() {
                    if current.value() != Some(value) {
                        return Ok(false);
                    }
                }
            }
            Ok(true)
        }
        DescriptorKind::Accessor => {
            if current.configurable() == Some(false) {
                if let Some(getter) = descriptor.getter() {
                    if current.getter() != Some(getter) {
                        return Ok(false);
                    }
                }
                if let Some(setter) = descriptor.setter() {
                    if current.setter() != Some(setter) {
                        return Ok(false);
                    }
                }
            }
            Ok(true)
        }
    }
}

fn to_length(number: f64) -> u64 {
    if !number.is_finite() || number <= 0.0 {
        0
    } else {
        number.min(9_007_199_254_740_991.0) as u64
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DescriptorKind {
    Generic,
    Data,
    Accessor,
}

use super::{
    errors, object, proxy, read, Agent, AllocationLifetime, FrameRecord, FunctionEntryIdentity,
    HostHooks, NativeFunctionRegistry, ObjectAllocation, ObjectRef, PropertyDescriptor,
    PropertyKey, RealmRef, Value, Vm, VmError, VmProxyBridge, VmResult, WellKnownAtom,
    WellKnownSymbolId,
};
use lyng_js_types::{function_symbol_has_instance_builtin, NativeFunctionId};

impl Vm {
    pub(in crate::vm) fn allocate_ordinary_object_with_prototype(
        agent: &mut Agent,
        realm: RealmRef,
        prototype: Option<ObjectRef>,
    ) -> VmResult<ObjectRef> {
        let root_shape = agent
            .realm(realm)
            .and_then(|realm| realm.root_shape())
            .ok_or(VmError::MissingRootShape(realm))?;
        Ok(agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape).with_prototype(prototype),
                AllocationLifetime::Default,
            )
        }))
    }

    pub(in crate::vm) fn descriptor_object_from_descriptor(
        agent: &mut Agent,
        realm: RealmRef,
        descriptor: PropertyDescriptor,
    ) -> VmResult<Value> {
        let object = Self::allocate_ordinary_object_with_prototype(
            agent,
            realm,
            agent
                .realm(realm)
                .and_then(|record| record.intrinsics().object_prototype()),
        )?;
        if let Some(value) = descriptor.value() {
            Self::define_data_property_with_attrs(
                agent,
                object,
                PropertyKey::from_atom(WellKnownAtom::value.id()),
                value,
                true,
                true,
                true,
            )?;
        }
        if let Some(getter) = descriptor.getter() {
            Self::define_data_property_with_attrs(
                agent,
                object,
                PropertyKey::from_atom(WellKnownAtom::get.id()),
                getter,
                true,
                true,
                true,
            )?;
        }
        if let Some(setter) = descriptor.setter() {
            Self::define_data_property_with_attrs(
                agent,
                object,
                PropertyKey::from_atom(WellKnownAtom::set.id()),
                setter,
                true,
                true,
                true,
            )?;
        }
        if let Some(writable) = descriptor.writable() {
            Self::define_data_property_with_attrs(
                agent,
                object,
                PropertyKey::from_atom(WellKnownAtom::writable.id()),
                Value::from_bool(writable),
                true,
                true,
                true,
            )?;
        }
        if let Some(enumerable) = descriptor.enumerable() {
            Self::define_data_property_with_attrs(
                agent,
                object,
                PropertyKey::from_atom(WellKnownAtom::enumerable.id()),
                Value::from_bool(enumerable),
                true,
                true,
                true,
            )?;
        }
        if let Some(configurable) = descriptor.configurable() {
            Self::define_data_property_with_attrs(
                agent,
                object,
                PropertyKey::from_atom(WellKnownAtom::configurable.id()),
                Value::from_bool(configurable),
                true,
                true,
                true,
            )?;
        }
        Ok(Value::from_object_ref(object))
    }

    pub(super) fn set_integrity_level(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        object_ref: ObjectRef,
        freeze: bool,
    ) -> VmResult<bool> {
        let mut bridge = VmProxyBridge {
            vm: self,
            agent,
            host,
            registry,
            frame,
        };
        if !proxy::prevent_extensions(&mut bridge, object_ref)? {
            return Ok(false);
        }
        let keys = object::own_property_keys_in_context(&mut bridge, object_ref)?;
        for key in keys {
            let Some(current) = object::get_own_property_in_context(&mut bridge, object_ref, key)?
            else {
                continue;
            };
            let mut descriptor = PropertyDescriptor::new();
            descriptor.set_configurable(false);
            let is_data_descriptor = (current.has_value() || current.has_writable())
                && !(current.has_get() || current.has_set());
            if freeze && is_data_descriptor {
                descriptor.set_writable(false);
            }
            if !object::define_property_in_context(
                &mut bridge,
                object_ref,
                key,
                descriptor,
                AllocationLifetime::Default,
            )? {
                return Err(proxy::ProxyTrapContext::type_error(&mut bridge));
            }
        }
        Ok(true)
    }

    pub(super) fn test_integrity_level(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        object_ref: ObjectRef,
        frozen: bool,
    ) -> VmResult<bool> {
        let mut bridge = VmProxyBridge {
            vm: self,
            agent,
            host,
            registry,
            frame,
        };
        if proxy::is_extensible(&mut bridge, object_ref)? {
            return Ok(false);
        }
        let keys = object::own_property_keys_in_context(&mut bridge, object_ref)?;
        for key in keys {
            let Some(descriptor) =
                object::get_own_property_in_context(&mut bridge, object_ref, key)?
            else {
                continue;
            };
            if descriptor.configurable() != Some(false) {
                return Ok(false);
            }
            let is_data_descriptor = (descriptor.has_value() || descriptor.has_writable())
                && !(descriptor.has_get() || descriptor.has_set());
            if frozen && is_data_descriptor && descriptor.writable() != Some(false) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(super) fn instance_of_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: &FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let value = arguments.first().copied().unwrap_or(Value::undefined());
        let constructor_value = arguments.get(1).copied().unwrap_or(Value::undefined());
        let constructor_object = constructor_value
            .as_object_ref()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let has_instance_key = agent
            .well_known_symbol(WellKnownSymbolId::HasInstance)
            .map(PropertyKey::from_symbol)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let has_instance = {
            let mut bridge = VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame: caller,
            };
            object::get_with_receiver_in_context(
                &mut bridge,
                constructor_object,
                has_instance_key,
                constructor_value,
            )?
        };
        if !has_instance.is_undefined() && !has_instance.is_null() {
            if Self::is_default_function_has_instance(agent, has_instance) {
                let is_instance = self.ordinary_has_instance_with_context(
                    agent,
                    host,
                    registry,
                    caller,
                    constructor_value,
                    value,
                )?;
                return Ok(Value::from_bool(is_instance));
            }
            let has_instance = Self::require_callable_object(agent, *caller, has_instance)?;
            let result = self.call_to_completion(
                agent,
                host,
                registry,
                caller,
                has_instance,
                constructor_value,
                &[value],
            )?;
            let is_instance = read::to_boolean_agent(agent, result).map_err(VmError::Abrupt)?;
            return Ok(Value::from_bool(is_instance));
        }

        let constructor = Self::require_callable_object(agent, *caller, constructor_value)?;
        let Some(object) = value.as_object_ref() else {
            return Ok(Value::from_bool(false));
        };
        let mut bridge = VmProxyBridge {
            vm: self,
            agent,
            host,
            registry,
            frame: caller,
        };
        let prototype = object::get_with_receiver_in_context(
            &mut bridge,
            constructor,
            PropertyKey::from_atom(WellKnownAtom::prototype.id()),
            Value::from_object_ref(constructor),
        )?
        .as_object_ref()
        .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(bridge.agent)))?;

        let mut current = object::get_prototype_of_in_context(&mut bridge, object)?;
        while let Some(candidate) = current {
            if candidate == prototype {
                return Ok(Value::from_bool(true));
            }
            current = object::get_prototype_of_in_context(&mut bridge, candidate)?;
        }

        Ok(Value::from_bool(false))
    }

    fn is_default_function_has_instance(agent: &Agent, value: Value) -> bool {
        let Some(object) = value.as_object_ref() else {
            return false;
        };
        matches!(
            agent
                .objects()
                .function_data(object)
                .and_then(lyng_js_objects::FunctionObjectData::entry),
            Some(FunctionEntryIdentity::Native(NativeFunctionId::Builtin(entry)))
                if entry == function_symbol_has_instance_builtin()
        )
    }

    fn ordinary_has_instance_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: &FrameRecord,
        constructor: Value,
        value: Value,
    ) -> VmResult<bool> {
        let Some(constructor) = constructor.as_object_ref() else {
            return Ok(false);
        };
        if !agent.objects().is_callable(constructor) {
            return Ok(false);
        }
        if let Some(bound) = Self::bound_function_record(agent, constructor) {
            return self.ordinary_has_instance_with_context(
                agent,
                host,
                registry,
                caller,
                Value::from_object_ref(bound.target()),
                value,
            );
        }
        let Some(object) = value.as_object_ref() else {
            return Ok(false);
        };

        let mut bridge = VmProxyBridge {
            vm: self,
            agent,
            host,
            registry,
            frame: caller,
        };
        let prototype = object::get_with_receiver_in_context(
            &mut bridge,
            constructor,
            PropertyKey::from_atom(WellKnownAtom::prototype.id()),
            Value::from_object_ref(constructor),
        )?
        .as_object_ref()
        .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(bridge.agent)))?;

        let mut current = object::get_prototype_of_in_context(&mut bridge, object)?;
        while let Some(candidate) = current {
            if candidate == prototype {
                return Ok(true);
            }
            current = object::get_prototype_of_in_context(&mut bridge, candidate)?;
        }

        Ok(false)
    }
}

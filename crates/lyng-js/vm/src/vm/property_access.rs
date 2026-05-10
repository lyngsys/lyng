use super::{
    AbruptCompletion, Agent, AllocationLifetime, ArgumentsMode, FrameRecord, HostHooks, ModuleKey,
    ModuleRecord, ModuleStatus, NativeFunctionRegistry, ObjectRef, Value, Vm, VmError, VmResult,
    WellKnownAtom,
};
use crate::vm::values::alloc_code_unit_string;
use crate::vm::values::encode_number;
use lyng_js_ops::{
    errors,
    object::{self, ToPrimitiveContext},
    proxy, read, typed_array,
};
use lyng_js_types::{PropertyDescriptor, PropertyKey, StringRef};
use std::collections::HashSet;

pub(super) use lyng_js_ops::object::ToPrimitiveHint;

fn array_length_to_uint32(number: f64) -> u32 {
    const TWO_32: f64 = 4_294_967_296.0;
    if !number.is_finite() || number == 0.0 {
        return 0;
    }
    number_to_u32_after_range_check(number.trunc().rem_euclid(TWO_32))
}

fn primitive_string_code_unit_len(agent: &mut Agent, string: StringRef) -> VmResult<u32> {
    if let Some(length) = agent
        .heap()
        .view()
        .string_view(string)
        .map(lyng_js_gc::PrimitiveStringView::code_unit_len)
    {
        return Ok(length);
    }
    Err(VmError::Abrupt(errors::throw_type_error(agent)))
}

fn primitive_string_code_unit(
    agent: &mut Agent,
    string: StringRef,
    index: u32,
) -> VmResult<Option<u16>> {
    let Some(unit) = ({
        agent
            .heap()
            .view()
            .string_view(string)
            .map(|view| view.code_unit_at(index as usize))
    }) else {
        return Err(VmError::Abrupt(errors::throw_type_error(agent)));
    };
    Ok(unit)
}

struct VmToPrimitiveBridge<'a> {
    vm: &'a mut Vm,
    agent: &'a mut Agent,
    host: &'a dyn HostHooks,
    registry: &'a mut dyn NativeFunctionRegistry,
    frame: FrameRecord,
}

pub(super) struct VmProxyBridge<'a> {
    pub(super) vm: &'a mut Vm,
    pub(super) agent: &'a mut Agent,
    pub(super) host: &'a dyn HostHooks,
    pub(super) registry: &'a mut dyn NativeFunctionRegistry,
    pub(super) frame: FrameRecord,
}

impl proxy::ProxyTrapContext for VmProxyBridge<'_> {
    type Error = VmError;

    fn agent(&mut self) -> &mut Agent {
        self.agent
    }

    fn abrupt(&mut self, completion: lyng_js_types::AbruptCompletion) -> Self::Error {
        VmError::Abrupt(completion)
    }

    fn type_error(&mut self) -> Self::Error {
        VmError::Abrupt(errors::throw_type_error(self.agent))
    }

    fn get_property_value(
        &mut self,
        receiver: Value,
        key: PropertyKey,
    ) -> Result<Value, Self::Error> {
        self.vm.get_property_from_value(
            self.agent,
            self.host,
            self.registry,
            self.frame,
            receiver,
            key,
        )
    }

    fn get_property_from_object_with_receiver(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
        receiver: Value,
    ) -> Result<Value, Self::Error> {
        self.vm.get_property_from_object_ordinary(
            self.agent,
            self.host,
            self.registry,
            self.frame,
            object,
            receiver,
            key,
        )
    }

    fn get_own_property_from_object(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> Result<Option<PropertyDescriptor>, Self::Error> {
        self.vm.get_own_property_from_object(
            self.agent,
            self.host,
            self.registry,
            self.frame,
            object,
            key,
        )
    }

    fn set_property_on_object_with_receiver(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
        value: Value,
        receiver: Value,
        _lifetime: AllocationLifetime,
    ) -> Result<bool, Self::Error> {
        self.vm.set_property_on_object_ordinary(
            self.agent,
            self.host,
            self.registry,
            self.frame,
            object,
            receiver,
            key,
            value,
        )
    }

    fn define_property_on_object(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
    ) -> Result<bool, Self::Error> {
        self.vm.evaluate_deferred_module_namespace(
            self.agent,
            self.host,
            self.registry,
            self.frame,
            object,
            key,
        )?;
        if let Some(result) = self.vm.define_typed_array_numeric_property(
            self.agent,
            self.host,
            self.registry,
            self.frame,
            object,
            key,
            descriptor,
        )? {
            return Ok(result);
        }
        self.vm
            .define_property_on_object(self.agent, object, key, descriptor, lifetime)
    }

    fn delete_property_from_object(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> Result<bool, Self::Error> {
        self.vm.evaluate_deferred_module_namespace(
            self.agent,
            self.host,
            self.registry,
            self.frame,
            object,
            key,
        )?;
        self.vm.delete_property_from_object(self.agent, object, key)
    }

    fn prepare_own_property_keys_from_object(
        &mut self,
        object: ObjectRef,
    ) -> Result<(), Self::Error> {
        self.vm.evaluate_deferred_module_namespace_for_own_keys(
            self.agent,
            self.host,
            self.registry,
            self.frame,
            object,
        )
    }

    fn prepare_has_property_from_object(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> Result<(), Self::Error> {
        self.vm.evaluate_deferred_module_namespace(
            self.agent,
            self.host,
            self.registry,
            self.frame,
            object,
            key,
        )
    }

    fn call_to_completion(
        &mut self,
        callee_object: ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> Result<Value, Self::Error> {
        self.vm.call_to_completion(
            self.agent,
            self.host,
            self.registry,
            self.frame,
            callee_object,
            this_value,
            arguments,
        )
    }

    fn construct_to_completion(
        &mut self,
        callee_object: ObjectRef,
        arguments: &[Value],
        new_target: Option<ObjectRef>,
    ) -> Result<ObjectRef, Self::Error> {
        self.vm.construct_to_completion(
            self.agent,
            self.host,
            self.registry,
            self.frame,
            callee_object,
            arguments,
            new_target,
        )
    }

    fn to_property_key(&mut self, value: Value) -> Result<PropertyKey, Self::Error> {
        self.vm
            .property_key_from_value(self.agent, self.host, self.registry, self.frame, value)
    }

    fn to_property_descriptor(
        &mut self,
        descriptor_object: ObjectRef,
    ) -> Result<PropertyDescriptor, Self::Error> {
        let mut descriptor = PropertyDescriptor::new();

        if object::has_property_in_context(
            self,
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::enumerable.id()),
        )? {
            let enumerable = self.get_property_value(
                Value::from_object_ref(descriptor_object),
                PropertyKey::from_atom(WellKnownAtom::enumerable.id()),
            )?;
            descriptor.set_enumerable(
                read::to_boolean_agent(self.agent, enumerable).map_err(VmError::Abrupt)?,
            );
        }
        if object::has_property_in_context(
            self,
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::configurable.id()),
        )? {
            let configurable = self.get_property_value(
                Value::from_object_ref(descriptor_object),
                PropertyKey::from_atom(WellKnownAtom::configurable.id()),
            )?;
            descriptor.set_configurable(
                read::to_boolean_agent(self.agent, configurable).map_err(VmError::Abrupt)?,
            );
        }
        if object::has_property_in_context(
            self,
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::value.id()),
        )? {
            let value = self.get_property_value(
                Value::from_object_ref(descriptor_object),
                PropertyKey::from_atom(WellKnownAtom::value.id()),
            )?;
            descriptor.set_value(value);
        }
        if object::has_property_in_context(
            self,
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::writable.id()),
        )? {
            let writable = self.get_property_value(
                Value::from_object_ref(descriptor_object),
                PropertyKey::from_atom(WellKnownAtom::writable.id()),
            )?;
            descriptor.set_writable(
                read::to_boolean_agent(self.agent, writable).map_err(VmError::Abrupt)?,
            );
        }
        if object::has_property_in_context(
            self,
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::get.id()),
        )? {
            let getter = self.get_property_value(
                Value::from_object_ref(descriptor_object),
                PropertyKey::from_atom(WellKnownAtom::get.id()),
            )?;
            if !(getter.is_undefined()
                || getter
                    .as_object_ref()
                    .is_some_and(|object| self.agent.objects().is_callable(object)))
            {
                return Err(VmError::Abrupt(errors::throw_type_error(self.agent)));
            }
            descriptor.set_getter(getter);
        }
        if object::has_property_in_context(
            self,
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::set.id()),
        )? {
            let setter = self.get_property_value(
                Value::from_object_ref(descriptor_object),
                PropertyKey::from_atom(WellKnownAtom::set.id()),
            )?;
            if !(setter.is_undefined()
                || setter
                    .as_object_ref()
                    .is_some_and(|object| self.agent.objects().is_callable(object)))
            {
                return Err(VmError::Abrupt(errors::throw_type_error(self.agent)));
            }
            descriptor.set_setter(setter);
        }

        if (descriptor.has_get() || descriptor.has_set())
            && (descriptor.has_value() || descriptor.has_writable())
        {
            return Err(VmError::Abrupt(errors::throw_type_error(self.agent)));
        }

        Ok(descriptor)
    }

    fn descriptor_object_from_descriptor(
        &mut self,
        descriptor: PropertyDescriptor,
    ) -> Result<Value, Self::Error> {
        Vm::descriptor_object_from_descriptor(self.agent, self.frame.realm(), descriptor)
    }

    fn create_array_from_values(&mut self, values: &[Value]) -> Result<ObjectRef, Self::Error> {
        let array = Vm::create_array(self.agent, self.frame.realm(), values.len())?;
        for (index, value) in values.iter().copied().enumerate() {
            let key = PropertyKey::Index(u32::try_from(index).unwrap_or(u32::MAX));
            let created = object::ordinary_create_data_property(
                self.agent,
                array,
                key,
                value,
                AllocationLifetime::Default,
            )
            .map_err(VmError::Abrupt)?;
            if !created {
                return Err(VmError::Abrupt(errors::throw_type_error(self.agent)));
            }
        }
        Ok(array)
    }
}

impl ToPrimitiveContext for VmToPrimitiveBridge<'_> {
    type Error = VmError;

    fn agent(&mut self) -> &mut Agent {
        self.agent
    }

    fn abrupt(&mut self, completion: lyng_js_types::AbruptCompletion) -> Self::Error {
        VmError::Abrupt(completion)
    }

    fn type_error(&mut self) -> Self::Error {
        VmError::Abrupt(errors::throw_type_error(self.agent))
    }

    fn get_property_value(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> Result<Value, Self::Error> {
        self.vm.get_property_from_object(
            self.agent,
            self.host,
            self.registry,
            self.frame,
            object,
            Value::from_object_ref(object),
            key,
        )
    }

    fn require_callable_object(&mut self, value: Value) -> Result<ObjectRef, Self::Error> {
        Vm::require_callable_object(self.agent, self.frame, value)
    }

    fn call_to_completion(
        &mut self,
        callee_object: ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> Result<Value, Self::Error> {
        self.vm.call_to_completion(
            self.agent,
            self.host,
            self.registry,
            self.frame,
            callee_object,
            this_value,
            arguments,
        )
    }
}

impl Vm {
    pub(super) fn property_key_from_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        value: Value,
    ) -> VmResult<PropertyKey> {
        if let Some(symbol) = value.as_symbol_ref() {
            return Ok(PropertyKey::from_symbol(symbol));
        }
        let primitive =
            self.to_primitive(agent, host, registry, frame, value, ToPrimitiveHint::String)?;
        self.value_to_property_key(
            agent,
            frame,
            frame.code(),
            frame.instruction_offset(),
            primitive,
        )
    }

    pub(super) fn get_property_from_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        receiver: Value,
        key: PropertyKey,
    ) -> VmResult<Value> {
        if let Some(string) = receiver.as_string_ref() {
            return self.get_property_from_string_primitive(
                agent, host, registry, frame, string, receiver, key,
            );
        }
        let object = Self::to_object_for_value(agent, frame.realm(), receiver)?;
        self.get_property_from_object(agent, host, registry, frame, object, receiver, key)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn get_property_from_string_primitive(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        string: StringRef,
        receiver: Value,
        key: PropertyKey,
    ) -> VmResult<Value> {
        if let Some(index) = key.as_index() {
            if let Some(unit) = primitive_string_code_unit(agent, string, index)? {
                let value = Value::from_string_ref(alloc_code_unit_string(agent, &[unit], None));
                return Ok(value);
            }
        } else if key.as_atom() == Some(WellKnownAtom::length.id()) {
            let length = primitive_string_code_unit_len(agent, string)?;
            return Ok(i32::try_from(length)
                .map_or_else(|_| Value::from_f64(f64::from(length)), Value::from_smi));
        }

        let prototype = agent
            .realm(frame.realm())
            .and_then(|record| record.intrinsics().string_prototype())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        self.get_property_from_object(agent, host, registry, frame, prototype, receiver, key)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn set_property_on_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        receiver: Value,
        key: PropertyKey,
        value: Value,
    ) -> VmResult<bool> {
        if receiver.is_null() || receiver.is_undefined() {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        let object = Self::to_object_for_value(agent, frame.realm(), receiver)?;
        if let Some(index) = key.as_index()
            && let Some(result) = self.mapped_arguments_set(agent, object, index, value)
        {
            result?;
            return Ok(true);
        }
        self.set_property_on_object(agent, host, registry, frame, object, receiver, key, value)
    }

    pub(super) fn try_fast_set_engine_array_index(
        agent: &mut Agent,
        object: ObjectRef,
        index: u32,
        value: Value,
    ) -> VmResult<Option<bool>> {
        if let Some(updated) = agent
            .with_heap_and_objects(|heap, objects| {
                let mut mutator = heap.mutator();
                objects.fast_update_engine_array_existing_index(&mut mutator, object, index, value)
            })
            .map_err(|_error| VmError::Abrupt(errors::throw_type_error(agent)))?
        {
            return Ok(Some(updated));
        }
        if !Self::engine_array_index_prototype_chain_is_clear(agent, object) {
            return Ok(None);
        }
        agent
            .with_heap_and_objects(|heap, objects| {
                let mut mutator = heap.mutator();
                objects.fast_set_engine_array_index(
                    &mut mutator,
                    object,
                    index,
                    value,
                    AllocationLifetime::Default,
                )
            })
            .map_err(|_error| VmError::Abrupt(errors::throw_type_error(agent)))
    }

    pub(super) fn try_fast_set_ordinary_index_data_property(
        agent: &mut Agent,
        object: ObjectRef,
        index: u32,
        value: Value,
    ) -> VmResult<Option<bool>> {
        if agent
            .objects()
            .object_header(agent.heap().view(), object)
            .is_some_and(|header| header.flags().is_engine_array())
        {
            return Ok(None);
        }
        if !Self::engine_array_index_prototype_chain_is_clear(agent, object) {
            return Ok(None);
        }
        agent
            .with_heap_and_objects(|heap, objects| {
                let mut mutator = heap.mutator();
                objects.fast_set_ordinary_index_data_property(
                    &mut mutator,
                    object,
                    index,
                    value,
                    AllocationLifetime::Default,
                )
            })
            .map_err(|_error| VmError::Abrupt(errors::throw_type_error(agent)))
    }

    pub(super) fn engine_array_index_prototype_chain_is_clear(
        agent: &Agent,
        object: ObjectRef,
    ) -> bool {
        let mut current = agent
            .objects()
            .object_header(agent.heap().view(), object)
            .and_then(lyng_js_objects::ObjectHeader::prototype);
        while let Some(prototype) = current {
            if agent.objects().is_proxy_object(prototype)
                || agent.objects().is_module_namespace_object(prototype)
                || agent.objects().primitive_wrapper_kind(prototype)
                    == Some(lyng_js_objects::PrimitiveWrapperKind::String)
                || agent.objects().is_typed_array_object(prototype)
                || agent.objects().element_logical_len(prototype).unwrap_or(0) != 0
            {
                return false;
            }
            current = agent
                .objects()
                .object_header(agent.heap().view(), prototype)
                .and_then(lyng_js_objects::ObjectHeader::prototype);
        }
        true
    }

    pub(super) fn prototype_chain_has_proxy(agent: &Agent, object: ObjectRef) -> bool {
        let mut current = Some(object);
        while let Some(object) = current {
            if agent.objects().is_proxy_object(object) {
                return true;
            }
            current = agent
                .objects()
                .object_header(agent.heap().view(), object)
                .and_then(lyng_js_objects::ObjectHeader::prototype);
        }
        false
    }

    fn legacy_function_caller(
        &self,
        agent: &Agent,
        object: ObjectRef,
        key: PropertyKey,
    ) -> Option<Value> {
        let PropertyKey::Atom(atom) = key else {
            return None;
        };
        if agent.atoms().resolve(atom) != "caller" {
            return None;
        }
        let code = Self::bytecode_entry(agent, object)?;
        let function = self.installed_function(code)?;
        if !Self::legacy_function_allows_caller_arguments(function) {
            return None;
        }

        let Some(active_index) = self
            .frames
            .iter()
            .rposition(|frame| frame.callee() == Some(object))
        else {
            return Some(Value::null());
        };
        let Some(active_frame) = self.frames.get(active_index).copied() else {
            return Some(Value::null());
        };
        if let Some(caller) = active_frame.tail_caller() {
            if active_frame.tail_caller_strict()
                || self.legacy_function_caller_is_restricted(agent, caller)
            {
                return Some(Value::null());
            }
            return Some(Value::from_object_ref(caller));
        }
        let Some(caller) = self.frames[..active_index]
            .iter()
            .rev()
            .find_map(|frame| frame.callee())
        else {
            return Some(Value::null());
        };
        if self.legacy_function_caller_is_restricted(agent, caller) {
            return Some(Value::null());
        }
        Some(Value::from_object_ref(caller))
    }

    fn legacy_function_allows_caller_arguments(
        function: &lyng_js_bytecode::BytecodeFunction,
    ) -> bool {
        function.kind() == lyng_js_bytecode::BytecodeFunctionKind::Function
            && !function.flags().strict()
            && !function.flags().generator()
            && !function.flags().async_function()
            && !function.flags().class_constructor()
            && function.flags().has_prototype_property()
            && function.flags().constructible()
    }

    fn legacy_function_caller_is_restricted(&self, agent: &Agent, function: ObjectRef) -> bool {
        Self::bytecode_entry(agent, function).is_some_and(|code| {
            self.installed_function(code)
                .is_some_and(|function| !Self::legacy_function_allows_caller_arguments(function))
        })
    }

    fn legacy_function_arguments(
        &self,
        agent: &Agent,
        object: ObjectRef,
        key: PropertyKey,
    ) -> VmResult<Option<Value>> {
        let PropertyKey::Atom(atom) = key else {
            return Ok(None);
        };
        if agent.atoms().resolve(atom) != "arguments" {
            return Ok(None);
        }
        let Some(code) = Self::bytecode_entry(agent, object) else {
            return Ok(None);
        };
        let Some(function) = self.installed_function(code) else {
            return Ok(None);
        };
        if !Self::legacy_function_allows_caller_arguments(function) {
            return Ok(None);
        }

        let Some(active_frame) = self
            .frames
            .iter()
            .rposition(|frame| frame.callee() == Some(object))
            .and_then(|index| self.frames.get(index))
            .copied()
        else {
            return Ok(Some(Value::null()));
        };
        let Some(arguments_slot) = legacy_function_arguments_slot(
            function.parameter_count(),
            function.arguments_mode(),
            function.has_rest_parameter(),
        ) else {
            return Ok(None);
        };
        Ok(Some(Self::read_environment_slot_raw(
            agent,
            active_frame.lexical_env(),
            arguments_slot,
        )?))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn copy_data_properties(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        target: ObjectRef,
        source: Value,
        excluded_keys: Value,
    ) -> VmResult<()> {
        if source.is_null() || source.is_undefined() {
            return Ok(());
        }
        let source = Self::to_object_for_value(agent, frame.realm(), source)?;
        let mut excluded = HashSet::new();

        if !excluded_keys.is_undefined() {
            let excluded_object = Self::to_object_for_value(agent, frame.realm(), excluded_keys)?;
            let excluded_values = object::own_property_keys_in_context(
                &mut VmProxyBridge {
                    vm: self,
                    agent,
                    host,
                    registry,
                    frame,
                },
                excluded_object,
            )?;
            for excluded_index in excluded_values {
                if excluded_index.as_index().is_none() {
                    continue;
                }
                let excluded_value = self.get_property_from_object(
                    agent,
                    host,
                    registry,
                    frame,
                    excluded_object,
                    Value::from_object_ref(excluded_object),
                    excluded_index,
                )?;
                excluded.insert(self.property_key_from_value(
                    agent,
                    host,
                    registry,
                    frame,
                    excluded_value,
                )?);
            }
        }

        let keys = object::own_property_keys_in_context(
            &mut VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame,
            },
            source,
        )?;
        for key in keys {
            if excluded.contains(&key) {
                continue;
            }
            let Some(descriptor) = object::get_own_property_in_context(
                &mut VmProxyBridge {
                    vm: self,
                    agent,
                    host,
                    registry,
                    frame,
                },
                source,
                key,
            )?
            else {
                continue;
            };
            if descriptor.enumerable() != Some(true) {
                continue;
            }

            let value = self.get_property_from_object(
                agent,
                host,
                registry,
                frame,
                source,
                Value::from_object_ref(source),
                key,
            )?;
            let created = object::ordinary_create_data_property(
                agent,
                target,
                key,
                value,
                AllocationLifetime::Default,
            )
            .map_err(VmError::Abrupt)?;
            if !created {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
        }

        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn get_property_from_object(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        object: ObjectRef,
        receiver: Value,
        key: PropertyKey,
    ) -> VmResult<Value> {
        self.evaluate_deferred_module_namespace(agent, host, registry, caller, object, key)?;
        object::get_with_receiver_in_context(
            &mut VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame: caller,
            },
            object,
            key,
            receiver,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn get_property_from_object_ordinary(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        object: ObjectRef,
        receiver: Value,
        key: PropertyKey,
    ) -> VmResult<Value> {
        self.evaluate_deferred_module_namespace(agent, host, registry, caller, object, key)?;
        if let Some(index) = key.as_index()
            && let Some(result) = self.mapped_arguments_get(agent, object, index)
        {
            return result;
        }
        if typed_array::is_numeric_key(agent, object, key) {
            return object::ordinary_get_with_receiver(agent, object, key, receiver)
                .map_err(VmError::Abrupt);
        }
        let descriptor =
            object::ordinary_get_own_property(agent, object, key).map_err(VmError::Abrupt)?;
        if let Some(descriptor) = descriptor {
            if let Some(value) = descriptor.value() {
                return Ok(value);
            }
            if let Some(value) =
                self.call_property_getter(agent, host, registry, caller, descriptor, receiver)?
            {
                return Ok(value);
            }
            return Ok(Value::undefined());
        }
        if let Some(value) = self.legacy_function_caller(agent, object, key) {
            return Ok(value);
        }
        if let Some(value) = self.legacy_function_arguments(agent, object, key)? {
            return Ok(value);
        }

        let prototype = agent
            .objects()
            .get_prototype_of(agent.heap().view(), object)
            .map_err(|_error| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let Some(prototype) = prototype else {
            return Ok(Value::undefined());
        };
        self.get_property_from_object(agent, host, registry, caller, prototype, receiver, key)
    }

    pub(in crate::vm) fn evaluate_deferred_module_namespace(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        object: ObjectRef,
        key: PropertyKey,
    ) -> VmResult<()> {
        if key
            .as_atom()
            .is_some_and(|atom| agent.atoms().resolve(atom) == "then")
        {
            return Ok(());
        }
        if key.is_symbol() {
            return Ok(());
        }
        self.evaluate_deferred_module_namespace_object(agent, host, registry, caller, object)
    }

    pub(in crate::vm) fn evaluate_deferred_module_namespace_for_own_keys(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        object: ObjectRef,
    ) -> VmResult<()> {
        self.evaluate_deferred_module_namespace_object(agent, host, registry, caller, object)
    }

    fn evaluate_deferred_module_namespace_object(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        object: ObjectRef,
    ) -> VmResult<()> {
        let Some(key) = self.deferred_module_namespaces.get(&object).cloned() else {
            return Ok(());
        };
        let status = agent
            .module_record(&key)
            .ok_or(VmError::MissingModuleRecord)?
            .status();
        match status {
            ModuleStatus::Evaluated => {
                self.deferred_module_namespaces.remove(&object);
                Ok(())
            }
            ModuleStatus::Errored => {
                let thrown = agent
                    .module_record(&key)
                    .and_then(ModuleRecord::evaluation_error)
                    .unwrap_or(Value::undefined());
                Err(VmError::Abrupt(AbruptCompletion::throw(thrown)))
            }
            ModuleStatus::Evaluating => Err(VmError::Abrupt(errors::throw_type_error(agent))),
            ModuleStatus::New
            | ModuleStatus::Unlinked
            | ModuleStatus::Linking
            | ModuleStatus::Linked => {
                let realm = agent
                    .realm(caller.realm())
                    .ok_or_else(|| VmError::MissingRootShape(caller.realm()))?;
                let module_env = self.link_module_graph(agent, &realm, &key)?;
                if !self.ready_for_sync_module_execution(agent, &key, &mut Vec::new())? {
                    return Err(VmError::Abrupt(errors::throw_type_error(agent)));
                }
                let result = self.evaluate_module_graph(
                    agent, &realm, &key, module_env, host, registry, None, true,
                );
                if agent
                    .module_record(&key)
                    .is_some_and(|record| matches!(record.status(), ModuleStatus::Evaluated))
                {
                    self.deferred_module_namespaces.remove(&object);
                }
                result.map(|_| ())
            }
        }
    }

    fn ready_for_sync_module_execution(
        &self,
        agent: &Agent,
        key: &ModuleKey,
        seen: &mut Vec<ModuleKey>,
    ) -> VmResult<bool> {
        if seen.iter().any(|candidate| candidate == key) {
            return Ok(true);
        }
        seen.push(key.clone());
        let (status, requested_modules) = {
            let record = agent
                .module_record(key)
                .ok_or(VmError::MissingModuleRecord)?;
            (record.status(), record.requested_modules().to_vec())
        };
        match status {
            ModuleStatus::Evaluated | ModuleStatus::Errored => return Ok(true),
            ModuleStatus::Evaluating
            | ModuleStatus::New
            | ModuleStatus::Unlinked
            | ModuleStatus::Linking => return Ok(false),
            ModuleStatus::Linked => {}
        }
        if self.module_has_top_level_await(agent, key)? {
            return Ok(false);
        }
        for request in requested_modules {
            let Some(resolved_key) = request.resolved_key().cloned() else {
                return Err(VmError::MissingModuleResolution);
            };
            if !self.ready_for_sync_module_execution(agent, &resolved_key, seen)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(super) fn get_own_property_from_object(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        object: ObjectRef,
        key: PropertyKey,
    ) -> VmResult<Option<PropertyDescriptor>> {
        self.evaluate_deferred_module_namespace(agent, host, registry, caller, object, key)?;
        let mut descriptor =
            object::ordinary_get_own_property(agent, object, key).map_err(VmError::Abrupt)?;
        let Some(index) = key.as_index() else {
            return Ok(descriptor);
        };
        let Some((environment, slot)) = self.activation_tables.mapped_argument_slot(object, index)
        else {
            return Ok(descriptor);
        };
        let Some(mapped_descriptor) = descriptor.as_mut() else {
            return Ok(None);
        };
        if !mapped_descriptor.has_get() && !mapped_descriptor.has_set() {
            mapped_descriptor.set_value(Self::read_environment_slot(agent, environment, slot)?);
        }
        Ok(descriptor)
    }

    pub(super) fn try_fast_own_index_value(
        agent: &mut Agent,
        object: ObjectRef,
        index: u32,
    ) -> VmResult<Option<Value>> {
        agent
            .objects()
            .fast_own_index_data_value(agent.heap().view(), object, index)
            .map_err(|_| VmError::Abrupt(errors::throw_type_error(agent)))
    }

    pub(super) fn try_fast_typed_array_index_value(
        agent: &mut Agent,
        object: ObjectRef,
        index: u32,
    ) -> Option<Value> {
        let record = agent.objects().typed_array(object)?;
        let index = usize::try_from(index).expect("u32 index should fit into usize");
        Some(typed_array::read_element_value(agent, record, index))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn try_fast_set_typed_array_index(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        object: ObjectRef,
        index: u32,
        value: Value,
    ) -> VmResult<Option<bool>> {
        if agent.objects().typed_array(object).is_none() {
            return Ok(None);
        }
        self.set_typed_array_numeric_index(
            agent,
            host,
            registry,
            caller,
            object,
            f64::from(index),
            value,
        )
        .map(Some)
    }

    pub(super) fn define_property_on_object(
        &mut self,
        agent: &mut Agent,
        object_ref: ObjectRef,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
    ) -> VmResult<bool> {
        let Some(index) = key.as_index() else {
            return object::ordinary_define_property(agent, object_ref, key, descriptor, lifetime)
                .map_err(VmError::Abrupt);
        };
        let Some((environment, slot)) = self
            .activation_tables
            .mapped_argument_slot(object_ref, index)
        else {
            return object::ordinary_define_property(agent, object_ref, key, descriptor, lifetime)
                .map_err(VmError::Abrupt);
        };

        let mut define_descriptor = descriptor;
        let is_accessor_descriptor = descriptor.has_get() || descriptor.has_set();
        if !is_accessor_descriptor
            && !descriptor.has_value()
            && descriptor.writable() == Some(false)
        {
            define_descriptor.set_value(Self::read_environment_slot(agent, environment, slot)?);
        }

        let defined =
            object::ordinary_define_property(agent, object_ref, key, define_descriptor, lifetime)
                .map_err(VmError::Abrupt)?;
        if !defined {
            return Ok(false);
        }

        if is_accessor_descriptor {
            let _ = self
                .activation_tables
                .detach_mapped_argument(object_ref, index);
            return Ok(true);
        }

        if let Some(value) = descriptor.value() {
            Self::set_environment_slot_raw(agent, environment, slot, value)?;
        }
        if descriptor.writable() == Some(false) {
            let _ = self
                .activation_tables
                .detach_mapped_argument(object_ref, index);
        }
        Ok(true)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn define_typed_array_numeric_property(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        object: ObjectRef,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
    ) -> VmResult<Option<bool>> {
        if agent.objects().typed_array(object).is_none() {
            return Ok(None);
        }
        let Some(numeric_index) = typed_array::numeric_property_index(agent, key) else {
            return Ok(None);
        };
        let Some(numeric_key) = typed_array::numeric_key(agent, object, key) else {
            return Ok(None);
        };
        if descriptor.has_get()
            || descriptor.has_set()
            || descriptor.configurable() == Some(false)
            || descriptor.enumerable() == Some(false)
            || descriptor.writable() == Some(false)
        {
            return Ok(Some(false));
        }
        let typed_array::NumericKey::Valid(_) = numeric_key else {
            return Ok(Some(false));
        };
        if let Some(value) = descriptor.value() {
            return self
                .set_typed_array_numeric_index(
                    agent,
                    host,
                    registry,
                    caller,
                    object,
                    numeric_index,
                    value,
                )
                .map(Some);
        }
        Ok(Some(true))
    }

    pub(super) fn delete_property_from_object(
        &mut self,
        agent: &mut Agent,
        object: ObjectRef,
        key: PropertyKey,
    ) -> VmResult<bool> {
        let deleted =
            object::ordinary_delete_property(agent, object, key).map_err(VmError::Abrupt)?;
        if deleted && let Some(index) = key.as_index() {
            let _ = self.activation_tables.detach_mapped_argument(object, index);
        }
        Ok(deleted)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn set_property_on_object(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        object: ObjectRef,
        receiver: Value,
        key: PropertyKey,
        value: Value,
    ) -> VmResult<bool> {
        object::set_with_receiver_in_context(
            &mut VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame: caller,
            },
            object,
            key,
            value,
            receiver,
            AllocationLifetime::Default,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn set_property_on_object_ordinary(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        object: ObjectRef,
        receiver: Value,
        key: PropertyKey,
        mut value: Value,
    ) -> VmResult<bool> {
        if agent.objects().is_module_namespace_object(object) {
            return Ok(false);
        }
        if agent.objects().typed_array(object).is_some()
            && let Some(index) = typed_array::numeric_property_index(agent, key)
        {
            if receiver.as_object_ref() == Some(object) {
                return self.set_typed_array_numeric_index(
                    agent, host, registry, caller, object, index, value,
                );
            }
            if !matches!(
                typed_array::numeric_key(agent, object, key),
                Some(typed_array::NumericKey::Valid(_))
            ) {
                return Ok(true);
            }
        }
        if Self::is_engine_array_length_property(agent, object, key) {
            value = self.normalize_array_length_set_value(agent, host, registry, caller, value)?;
        }
        let own_descriptor =
            object::ordinary_get_own_property(agent, object, key).map_err(VmError::Abrupt)?;
        if let Some(descriptor) = own_descriptor {
            return self.set_property_from_descriptor(
                agent, host, registry, caller, descriptor, receiver, key, value,
            );
        }

        let prototype = agent
            .objects()
            .get_prototype_of(agent.heap().view(), object)
            .map_err(|_error| VmError::Abrupt(errors::throw_type_error(agent)))?;
        if let Some(prototype) = prototype {
            return self.set_property_on_object(
                agent, host, registry, caller, prototype, receiver, key, value,
            );
        }

        self.create_or_update_receiver_data_property(
            agent, host, registry, caller, receiver, key, value,
        )
    }

    fn is_engine_array_length_property(agent: &Agent, object: ObjectRef, key: PropertyKey) -> bool {
        key.as_atom() == Some(WellKnownAtom::length.id())
            && agent
                .objects()
                .object_header(agent.heap().view(), object)
                .is_some_and(|header| header.flags().is_engine_array())
    }

    fn normalize_array_length_set_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        value: Value,
    ) -> VmResult<Value> {
        let primitive = self.to_primitive(
            agent,
            host,
            registry,
            caller,
            value,
            ToPrimitiveHint::Number,
        )?;
        let _ = read::to_number(agent.heap().view(), primitive).map_err(VmError::Abrupt)?;
        let primitive = self.to_primitive(
            agent,
            host,
            registry,
            caller,
            value,
            ToPrimitiveHint::Number,
        )?;
        let number = read::to_number(agent.heap().view(), primitive).map_err(VmError::Abrupt)?;
        let number = number
            .as_f64()
            .expect("ToNumber must always produce a numeric Value");
        let length = array_length_to_uint32(number);
        #[allow(
            clippy::float_cmp,
            reason = "array length assignment requires exact ECMA-262 ToUint32 equality"
        )]
        if number != f64::from(length) {
            return Err(VmError::Abrupt(errors::throw_range_error(agent)));
        }
        Ok(encode_number(f64::from(length)))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn set_property_from_descriptor(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        descriptor: PropertyDescriptor,
        receiver: Value,
        key: PropertyKey,
        value: Value,
    ) -> VmResult<bool> {
        if descriptor.has_get() || descriptor.has_set() {
            return self
                .call_property_setter(agent, host, registry, caller, descriptor, receiver, value);
        }

        if !descriptor.writable().unwrap_or(false) {
            return Ok(false);
        }
        self.create_or_update_receiver_data_property(
            agent, host, registry, caller, receiver, key, value,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn create_or_update_receiver_data_property(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        receiver: Value,
        key: PropertyKey,
        mut value: Value,
    ) -> VmResult<bool> {
        let Some(receiver) = receiver.as_object_ref() else {
            return Ok(false);
        };
        if agent.objects().is_module_namespace_object(receiver) {
            return Ok(false);
        }
        if Self::is_engine_array_length_property(agent, receiver, key) {
            value = self.normalize_array_length_set_value(agent, host, registry, caller, value)?;
        }
        let receiver_descriptor = object::get_own_property_in_context(
            &mut VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame: caller,
            },
            receiver,
            key,
        )?;
        if let Some(receiver_descriptor) = receiver_descriptor {
            if receiver_descriptor.has_get() || receiver_descriptor.has_set() {
                return Ok(false);
            }
            if !receiver_descriptor.writable().unwrap_or(false) {
                return Ok(false);
            }
            let mut update = PropertyDescriptor::new();
            update.set_value(value);
            let defined = object::define_property_in_context(
                &mut VmProxyBridge {
                    vm: self,
                    agent,
                    host,
                    registry,
                    frame: caller,
                },
                receiver,
                key,
                update,
                AllocationLifetime::Default,
            )?;
            return Ok(defined);
        }

        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_value(value);
        descriptor.set_writable(true);
        descriptor.set_enumerable(true);
        descriptor.set_configurable(true);
        object::define_property_in_context(
            &mut VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame: caller,
            },
            receiver,
            key,
            descriptor,
            AllocationLifetime::Default,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn set_typed_array_numeric_index(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        object: ObjectRef,
        numeric_index: f64,
        value: Value,
    ) -> VmResult<bool> {
        let Some(typed_array) = agent.objects().typed_array(object) else {
            return Ok(false);
        };
        let bits = {
            let mut bridge = VmToPrimitiveBridge {
                vm: self,
                agent,
                host,
                registry,
                frame: caller,
            };
            typed_array::storage_bits_from_value(&mut bridge, typed_array.kind(), value)?
        };
        let Some(typed_array) = agent.objects().typed_array(object) else {
            return Ok(false);
        };
        let Some(element_index) = typed_array::element_index_from_numeric_index(numeric_index)
        else {
            return Ok(true);
        };
        let Some(start) =
            typed_array::valid_integer_index_byte_start(agent, typed_array, element_index)
        else {
            return Ok(true);
        };
        if !agent.backing_store_store_bits(
            typed_array.backing_store(),
            start,
            typed_array.kind().bytes_per_element(),
            bits,
        ) {
            return Ok(false);
        }
        Ok(true)
    }

    #[allow(
        clippy::wrong_self_convention,
        reason = "method name intentionally mirrors the ECMA-262 ToPrimitive abstract operation"
    )]
    pub(super) fn to_primitive(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        value: Value,
        hint: ToPrimitiveHint,
    ) -> VmResult<Value> {
        let mut bridge = VmToPrimitiveBridge {
            vm: self,
            agent,
            host,
            registry,
            frame,
        };
        lyng_js_ops::object::to_primitive(&mut bridge, value, hint)
    }
}

fn legacy_function_arguments_slot(
    parameter_count: u16,
    arguments_mode: ArgumentsMode,
    has_rest_parameter: bool,
) -> Option<u32> {
    match arguments_mode {
        ArgumentsMode::None => None,
        ArgumentsMode::Mapped => Some(u32::from(parameter_count)),
        ArgumentsMode::Unmapped => Some(u32::from(has_rest_parameter)),
    }
}

const fn number_to_u32_after_range_check(number: f64) -> u32 {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "caller applies the ECMAScript modulo/range rules before narrowing to u32"
    )]
    let integer = number as u32;
    integer
}

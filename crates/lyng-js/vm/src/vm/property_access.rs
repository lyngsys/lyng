use super::*;
use crate::vm::values::encode_number;
use lyng_js_objects::{InternalMethodError, TypedArrayElementKind};
use lyng_js_ops::{
    errors, number_to_string,
    object::{self, ToPrimitiveContext},
    proxy, read,
};
use lyng_js_types::{PropertyDescriptor, PropertyKey};
use std::collections::HashSet;

pub(super) use lyng_js_ops::object::ToPrimitiveHint;

fn bigint_to_uint64_bits(agent: &Agent, value: Value) -> Option<u64> {
    let bigint = value.as_bigint_ref()?;
    let view = agent.heap().view().bigint_view(bigint)?;
    let low = view.limb_at(0).unwrap_or(0);
    Some(match view.sign() {
        lyng_js_gc::BigIntSign::NonNegative => low,
        lyng_js_gc::BigIntSign::Negative => 0_u64.wrapping_sub(low),
    })
}

fn array_length_to_uint32(number: f64) -> u32 {
    if !number.is_finite() || number == 0.0 {
        return 0;
    }
    const TWO_32: f64 = 4_294_967_296.0;
    number.trunc().rem_euclid(TWO_32) as u32
}

fn vm_typed_array_storage_bits(kind: TypedArrayElementKind, number: f64) -> u64 {
    match kind {
        TypedArrayElementKind::BigInt64 => number as u64,
        TypedArrayElementKind::BigUint64 => number as u64,
        TypedArrayElementKind::Int8 | TypedArrayElementKind::Uint8 => {
            u64::from(vm_to_uint8(number))
        }
        TypedArrayElementKind::Uint8Clamped => u64::from(vm_to_uint8_clamp(number)),
        TypedArrayElementKind::Int16 | TypedArrayElementKind::Uint16 => {
            u64::from(vm_to_uint16(number))
        }
        TypedArrayElementKind::Float32 => u64::from(f32::to_bits(number as f32)),
        TypedArrayElementKind::Float64 => number.to_bits(),
        TypedArrayElementKind::Int32 | TypedArrayElementKind::Uint32 => {
            u64::from(vm_to_uint32(number))
        }
    }
}

fn canonical_numeric_index_string(text: &str) -> Option<f64> {
    if text == "-0" {
        return Some(-0.0);
    }
    let number = match text {
        "NaN" => f64::NAN,
        "Infinity" => f64::INFINITY,
        "-Infinity" => f64::NEG_INFINITY,
        _ => text.parse::<f64>().ok()?,
    };
    (number_to_string(number) == text).then_some(number)
}

fn typed_array_numeric_atom_index(agent: &Agent, key: PropertyKey) -> Option<f64> {
    canonical_numeric_index_string(agent.atoms().resolve(key.as_atom()?))
}

fn map_internal_method_error(agent: &mut Agent, error: InternalMethodError) -> VmError {
    let abrupt = match error {
        InternalMethodError::RangeError => errors::throw_range_error(agent),
        InternalMethodError::ReferenceError => errors::throw_reference_error(agent),
        _ => errors::throw_type_error(agent),
    };
    VmError::Abrupt(abrupt)
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
        self.vm.get_property_from_object(
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
        self.vm
            .get_own_property_from_object(self.agent, object, key)
    }

    fn set_property_on_object_with_receiver(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
        value: Value,
        receiver: Value,
        _lifetime: AllocationLifetime,
    ) -> Result<bool, Self::Error> {
        self.vm.set_property_on_object(
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
        self.vm
            .define_property_on_object(self.agent, object, key, descriptor, lifetime)
    }

    fn delete_property_from_object(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> Result<bool, Self::Error> {
        self.vm.delete_property_from_object(self.agent, object, key)
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
            .to_property_key_from_value(self.agent, self.host, self.registry, self.frame, value)
    }

    fn to_property_descriptor(
        &mut self,
        descriptor_object: ObjectRef,
    ) -> Result<PropertyDescriptor, Self::Error> {
        let mut descriptor = PropertyDescriptor::new();

        if proxy::has_property(
            self,
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::enumerable.id()),
        )? {
            let enumerable = self.get_property_value(
                Value::from_object_ref(descriptor_object),
                PropertyKey::from_atom(WellKnownAtom::enumerable.id()),
            )?;
            descriptor.set_enumerable(
                read::to_boolean(self.agent.heap().view(), enumerable).map_err(VmError::Abrupt)?,
            );
        }
        if proxy::has_property(
            self,
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::configurable.id()),
        )? {
            let configurable = self.get_property_value(
                Value::from_object_ref(descriptor_object),
                PropertyKey::from_atom(WellKnownAtom::configurable.id()),
            )?;
            descriptor.set_configurable(
                read::to_boolean(self.agent.heap().view(), configurable)
                    .map_err(VmError::Abrupt)?,
            );
        }
        if proxy::has_property(
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
        if proxy::has_property(
            self,
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::writable.id()),
        )? {
            let writable = self.get_property_value(
                Value::from_object_ref(descriptor_object),
                PropertyKey::from_atom(WellKnownAtom::writable.id()),
            )?;
            descriptor.set_writable(
                read::to_boolean(self.agent.heap().view(), writable).map_err(VmError::Abrupt)?,
            );
        }
        if proxy::has_property(
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
        if proxy::has_property(
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
        self.vm
            .descriptor_object_from_descriptor(self.agent, self.frame.realm(), descriptor)
    }

    fn create_array_from_values(&mut self, values: &[Value]) -> Result<ObjectRef, Self::Error> {
        let array = self
            .vm
            .create_array(self.agent, self.frame.realm(), values.len())?;
        for (index, value) in values.iter().copied().enumerate() {
            let key = PropertyKey::Index(u32::try_from(index).unwrap_or(u32::MAX));
            let created = object::create_data_property(
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
    pub(super) fn to_property_key_from_value(
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
        let object = self.to_object_for_value(agent, frame.realm(), receiver)?;
        self.get_property_from_object(agent, host, registry, frame, object, receiver, key)
    }

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
        let object = self.to_object_for_value(agent, frame.realm(), receiver)?;
        if let Some(index) = key.as_index() {
            if let Some(result) = self.mapped_arguments_set(agent, object, index, value) {
                result?;
                return Ok(true);
            }
        }
        self.set_property_on_object(agent, host, registry, frame, object, receiver, key, value)
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
                .and_then(|header| header.prototype());
        }
        false
    }

    fn legacy_function_caller(
        &self,
        agent: &mut Agent,
        object: ObjectRef,
        key: PropertyKey,
    ) -> VmResult<Option<Value>> {
        let PropertyKey::Atom(atom) = key else {
            return Ok(None);
        };
        if agent.atoms().resolve(atom) != "caller" {
            return Ok(None);
        }
        let Some(code) = Self::bytecode_entry(agent, object) else {
            return Ok(None);
        };
        let Some(function) = self.installed_function(code) else {
            return Ok(None);
        };
        if function.kind() != lyng_js_bytecode::BytecodeFunctionKind::Function
            || function.flags().strict()
        {
            return Ok(None);
        }

        let Some(active_index) = self
            .frames
            .iter()
            .rposition(|frame| frame.callee() == Some(object))
        else {
            return Ok(Some(Value::null()));
        };
        let Some(active_frame) = self.frames.get(active_index).copied() else {
            return Ok(Some(Value::null()));
        };
        if let Some(caller) = active_frame.tail_caller() {
            if active_frame.tail_caller_strict() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return Ok(Some(Value::from_object_ref(caller)));
        }
        let Some(caller_frame) = active_index
            .checked_sub(1)
            .and_then(|index| self.frames.get(index))
            .copied()
        else {
            return Ok(Some(Value::null()));
        };
        let Some(caller) = caller_frame.callee() else {
            return Ok(Some(Value::null()));
        };
        if Self::bytecode_entry(agent, caller).is_some_and(|caller_code| {
            self.installed_function(caller_code)
                .is_some_and(|function| function.flags().strict())
        }) {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(Some(Value::from_object_ref(caller)))
    }

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
        let source = self.to_object_for_value(agent, frame.realm(), source)?;
        let mut excluded = HashSet::new();

        if !excluded_keys.is_undefined() {
            let excluded_object = self.to_object_for_value(agent, frame.realm(), excluded_keys)?;
            let excluded_values =
                object::own_property_keys(agent, excluded_object).map_err(VmError::Abrupt)?;
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
                excluded.insert(self.to_property_key_from_value(
                    agent,
                    host,
                    registry,
                    frame,
                    excluded_value,
                )?);
            }
        }

        let keys = object::own_property_keys(agent, source).map_err(VmError::Abrupt)?;
        for key in keys {
            if excluded.contains(&key) {
                continue;
            }
            let Some(descriptor) =
                object::get_own_property(agent, source, key).map_err(VmError::Abrupt)?
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
            let created = object::create_data_property(
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
        if agent.objects().is_proxy_object(object) {
            let mut bridge = VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame: caller,
            };
            return proxy::get(&mut bridge, object, key, receiver);
        }
        if let Some(index) = key.as_index() {
            if let Some(result) = self.mapped_arguments_get(agent, object, index) {
                return result;
            }
            if agent.objects().typed_array(object).is_some() {
                return object::get_with_receiver(agent, object, key, receiver)
                    .map_err(VmError::Abrupt);
            }
        }
        let descriptor = object::get_own_property(agent, object, key).map_err(VmError::Abrupt)?;
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
        if let Some(value) = self.legacy_function_caller(agent, object, key)? {
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

    pub(super) fn get_own_property_from_object(
        &self,
        agent: &mut Agent,
        object: ObjectRef,
        key: PropertyKey,
    ) -> VmResult<Option<PropertyDescriptor>> {
        let mut descriptor =
            object::get_own_property(agent, object, key).map_err(VmError::Abrupt)?;
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
            mapped_descriptor.set_value(self.read_environment_slot(agent, environment, slot)?);
        }
        Ok(descriptor)
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
            return object::define_property(agent, object_ref, key, descriptor, lifetime)
                .map_err(VmError::Abrupt);
        };
        let Some((environment, slot)) = self
            .activation_tables
            .mapped_argument_slot(object_ref, index)
        else {
            return object::define_property(agent, object_ref, key, descriptor, lifetime)
                .map_err(VmError::Abrupt);
        };

        let mut define_descriptor = descriptor;
        let is_accessor_descriptor = descriptor.has_get() || descriptor.has_set();
        if !is_accessor_descriptor
            && !descriptor.has_value()
            && descriptor.writable() == Some(false)
        {
            define_descriptor.set_value(self.read_environment_slot(agent, environment, slot)?);
        }

        let defined = object::define_property(agent, object_ref, key, define_descriptor, lifetime)
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

    pub(super) fn delete_property_from_object(
        &mut self,
        agent: &mut Agent,
        object: ObjectRef,
        key: PropertyKey,
    ) -> VmResult<bool> {
        let deleted = object::delete_property(agent, object, key).map_err(VmError::Abrupt)?;
        if deleted {
            if let Some(index) = key.as_index() {
                let _ = self.activation_tables.detach_mapped_argument(object, index);
            }
        }
        Ok(deleted)
    }

    pub(super) fn set_property_on_object(
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
        if agent.objects().is_proxy_object(object) {
            let mut bridge = VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame: caller,
            };
            return proxy::set(
                &mut bridge,
                object,
                key,
                value,
                receiver,
                AllocationLifetime::Default,
            );
        }
        if agent.objects().typed_array(object).is_some() {
            if key.as_index().is_some() {
                return self
                    .set_typed_array_index(agent, host, registry, caller, object, key, value);
            }
            if let Some(index) = typed_array_numeric_atom_index(agent, key) {
                if index.is_finite()
                    && !(index == 0.0 && index.is_sign_negative())
                    && index.fract() == 0.0
                    && index >= 0.0
                {
                    if let Some(index_key) = PropertyKey::from_array_index(index as u64) {
                        return self.set_typed_array_index(
                            agent, host, registry, caller, object, index_key, value,
                        );
                    }
                }
                return Ok(false);
            }
        }
        if Self::is_engine_array_length_property(agent, object, key) {
            value = self.normalize_array_length_set_value(agent, host, registry, caller, value)?;
        }
        let own_descriptor =
            object::get_own_property(agent, object, key).map_err(VmError::Abrupt)?;
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
        if number != f64::from(length) {
            return Err(VmError::Abrupt(errors::throw_range_error(agent)));
        }
        Ok(encode_number(f64::from(length)))
    }

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
        if Self::is_engine_array_length_property(agent, receiver, key) {
            value = self.normalize_array_length_set_value(agent, host, registry, caller, value)?;
        }
        if key.as_index().is_some() && agent.objects().typed_array(receiver).is_some() {
            return self.set_typed_array_index(agent, host, registry, caller, receiver, key, value);
        }
        let receiver_descriptor = if agent.objects().is_proxy_object(receiver) {
            proxy::get_own_property(
                &mut VmProxyBridge {
                    vm: self,
                    agent,
                    host,
                    registry,
                    frame: caller,
                },
                receiver,
                key,
            )?
        } else {
            object::get_own_property(agent, receiver, key).map_err(VmError::Abrupt)?
        };
        if let Some(receiver_descriptor) = receiver_descriptor {
            if receiver_descriptor.has_get() || receiver_descriptor.has_set() {
                return Ok(false);
            }
            if !receiver_descriptor.writable().unwrap_or(false) {
                return Ok(false);
            }
            let mut update = PropertyDescriptor::new();
            update.set_value(value);
            if agent.objects().is_proxy_object(receiver) {
                return proxy::define_property(
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
                );
            }
            let defined = agent.with_heap_and_objects(|heap, objects| {
                let mut mutator = heap.mutator();
                objects.define_own_property(
                    &mut mutator,
                    receiver,
                    key,
                    update,
                    AllocationLifetime::Default,
                )
            });
            let _ = defined.map_err(|error| map_internal_method_error(agent, error))?;
            return Ok(true);
        }

        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_value(value);
        descriptor.set_writable(true);
        descriptor.set_enumerable(true);
        descriptor.set_configurable(true);
        if agent.objects().is_proxy_object(receiver) {
            return proxy::define_property(
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
            );
        }
        let defined = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.define_own_property(
                &mut mutator,
                receiver,
                key,
                descriptor,
                AllocationLifetime::Default,
            )
        });
        let defined = defined.map_err(|error| map_internal_method_error(agent, error))?;
        Ok(defined)
    }

    fn set_typed_array_index(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        object: ObjectRef,
        key: PropertyKey,
        value: Value,
    ) -> VmResult<bool> {
        let Some(index) = key.as_index() else {
            return Ok(false);
        };
        let Some(typed_array) = agent.objects().typed_array(object) else {
            return Ok(false);
        };
        let index = usize::try_from(index).unwrap_or(usize::MAX);
        if index >= typed_array.length() {
            return Ok(false);
        }
        if agent
            .backing_store_is_detached(typed_array.backing_store())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?
        {
            return Ok(false);
        }

        let primitive = self.to_primitive(
            agent,
            host,
            registry,
            caller,
            value,
            ToPrimitiveHint::Number,
        )?;
        if agent
            .backing_store_is_detached(typed_array.backing_store())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?
        {
            return Ok(false);
        }
        let bits = if matches!(
            typed_array.kind(),
            TypedArrayElementKind::BigInt64 | TypedArrayElementKind::BigUint64
        ) {
            let bigint = object::primitive_to_bigint(agent, primitive).map_err(VmError::Abrupt)?;
            bigint_to_uint64_bits(agent, bigint)
                .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?
        } else {
            let number =
                read::to_number(agent.heap().view(), primitive).map_err(VmError::Abrupt)?;
            vm_typed_array_storage_bits(
                typed_array.kind(),
                number
                    .as_f64()
                    .expect("ToNumber must always produce a numeric Value"),
            )
        };
        let element_size = typed_array.kind().bytes_per_element();
        let absolute_index = typed_array
            .byte_offset()
            .checked_add(index.checked_mul(element_size).unwrap_or(usize::MAX))
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        for offset in 0..element_size {
            let byte_index = absolute_index
                .checked_add(offset)
                .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
            let shift = offset * 8;
            let byte = u8::try_from((bits >> shift) & 0xff).expect("element byte should fit");
            if !agent.backing_store_set_byte(typed_array.backing_store(), byte_index, byte) {
                return Ok(false);
            }
        }
        Ok(true)
    }

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

fn vm_to_uint8(number: f64) -> u8 {
    if number.is_nan() || number == 0.0 || !number.is_finite() {
        return 0;
    }
    let integer = number.trunc();
    let mut modulo = integer % 256.0;
    if modulo < 0.0 {
        modulo += 256.0;
    }
    modulo as u8
}

fn vm_to_uint8_clamp(number: f64) -> u8 {
    if number.is_nan() || number <= 0.0 {
        return 0;
    }
    if number >= 255.0 {
        return 255;
    }
    let floor = number.floor();
    if floor + 0.5 < number {
        return (floor as u8).saturating_add(1);
    }
    if number < floor + 0.5 {
        return floor as u8;
    }
    let floor_u8 = floor as u8;
    if floor_u8 % 2 == 1 {
        floor_u8.saturating_add(1)
    } else {
        floor_u8
    }
}

fn vm_to_uint16(number: f64) -> u16 {
    if number.is_nan() || number == 0.0 || !number.is_finite() {
        return 0;
    }
    let integer = number.trunc();
    let mut modulo = integer % 65_536.0;
    if modulo < 0.0 {
        modulo += 65_536.0;
    }
    modulo as u16
}

fn vm_to_uint32(number: f64) -> u32 {
    if number.is_nan() || number == 0.0 || !number.is_finite() {
        return 0;
    }
    let integer = number.trunc();
    let mut modulo = integer % 4_294_967_296.0;
    if modulo < 0.0 {
        modulo += 4_294_967_296.0;
    }
    modulo as u32
}

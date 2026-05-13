use super::{
    alloc_code_unit_string, errors, object, object_to_string_builtin, read, to_f64_number,
    AbruptCompletion, Agent, HostErrorKind, ObjectRef, PropertyDescriptor, PropertyKey, RealmRef,
    Value, Vm, VmBuiltinDispatch, VmError, VmProxyBridge, VmResult, WellKnownAtom,
};

impl object::ToPrimitiveContext for VmBuiltinDispatch<'_, '_, '_> {
    type Error = VmError;

    fn agent(&mut self) -> &mut Agent {
        self.agent
    }

    fn abrupt(&mut self, completion: AbruptCompletion) -> Self::Error {
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
        self.builtin_get_property_value_from_object(object, key)
    }

    fn require_callable_object(&mut self, value: Value) -> Result<ObjectRef, Self::Error> {
        Vm::require_callable_object(self.agent, *self.caller_frame, value)
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
            self.caller_frame,
            callee_object,
            this_value,
            arguments,
        )
    }

    fn default_to_primitive_result(
        &mut self,
        object: ObjectRef,
        method_name: lyng_js_common::AtomId,
        method_object: ObjectRef,
    ) -> Result<Option<Value>, Self::Error> {
        let Some(entry) = Vm::builtin_entry(self.agent, method_object) else {
            return Ok(None);
        };
        if method_name != WellKnownAtom::toString.id() || entry != object_to_string_builtin() {
            return Ok(None);
        }

        let is_engine_array = self
            .agent
            .objects()
            .object_header(self.agent.heap().view(), object)
            .is_some_and(|header| header.flags().is_engine_array());
        if !is_engine_array {
            return Ok(None);
        }

        self.engine_array_to_string_fallback_value(object).map(Some)
    }
}

impl VmBuiltinDispatch<'_, '_, '_> {
    pub(super) fn builtin_get_property_value(
        &mut self,
        receiver: Value,
        key: PropertyKey,
    ) -> VmResult<Value> {
        self.vm.get_property_from_value(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            receiver,
            key,
        )
    }

    fn builtin_get_property_value_from_object(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> VmResult<Value> {
        self.builtin_get_property_value(Value::from_object_ref(object), key)
    }

    fn builtin_has_property_on_object(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> VmResult<bool> {
        object::has_property_in_context(
            &mut VmProxyBridge {
                vm: self.vm,
                agent: self.agent,
                host: self.host,
                registry: self.registry,
                frame: self.caller_frame,
            },
            object,
            key,
        )
    }

    pub(super) fn builtin_constructor_prototype(
        &mut self,
        source_realm: RealmRef,
        default_prototype: ObjectRef,
        new_target: Option<ObjectRef>,
    ) -> VmResult<ObjectRef> {
        let Some(new_target) = new_target else {
            return Ok(default_prototype);
        };
        if let Some(prototype) =
            self.fast_ordinary_constructor_prototype(source_realm, default_prototype, new_target)?
        {
            return Ok(prototype);
        }
        let prototype = self.builtin_get_property_value_from_object(
            new_target,
            PropertyKey::from_atom(WellKnownAtom::prototype.id()),
        )?;
        if let Some(prototype) = prototype.as_object_ref() {
            return Ok(prototype);
        }
        let function_realm = Vm::function_realm(self.agent, new_target)?;
        if function_realm == source_realm {
            return Ok(default_prototype);
        }
        Ok(self
            .remap_constructor_default_prototype(source_realm, function_realm, default_prototype)
            .unwrap_or(default_prototype))
    }

    fn fast_ordinary_constructor_prototype(
        &mut self,
        source_realm: RealmRef,
        default_prototype: ObjectRef,
        new_target: ObjectRef,
    ) -> VmResult<Option<ObjectRef>> {
        let is_ordinary_target = self
            .agent
            .objects()
            .object_header(self.agent.heap().view(), new_target)
            .is_some_and(|header| header.kind() != lyng_js_objects::ObjectKind::Proxy);
        if !is_ordinary_target {
            return Ok(None);
        }
        let key = PropertyKey::from_atom(WellKnownAtom::prototype.id());
        let Some(descriptor) = object::ordinary_get_own_property(self.agent, new_target, key)
            .map_err(VmError::Abrupt)?
        else {
            return Ok(None);
        };
        let Some(value) = descriptor.value() else {
            return Ok(None);
        };
        if let Some(prototype) = value.as_object_ref() {
            return Ok(Some(prototype));
        }
        let function_realm = Vm::function_realm(self.agent, new_target)?;
        if function_realm == source_realm {
            return Ok(Some(default_prototype));
        }
        Ok(Some(
            self.remap_constructor_default_prototype(
                source_realm,
                function_realm,
                default_prototype,
            )
            .unwrap_or(default_prototype),
        ))
    }

    fn remap_constructor_default_prototype(
        &self,
        source_realm: RealmRef,
        target_realm: RealmRef,
        default_prototype: ObjectRef,
    ) -> Option<ObjectRef> {
        let source_intrinsics = self.agent.realm(source_realm)?.intrinsics();
        let target_intrinsics = self.agent.realm(target_realm)?.intrinsics();

        macro_rules! remap_intrinsic_prototype {
            ($getter:ident) => {
                if source_intrinsics.$getter() == Some(default_prototype) {
                    return target_intrinsics.$getter();
                }
            };
        }

        remap_intrinsic_prototype!(object_prototype);
        remap_intrinsic_prototype!(function_prototype);
        remap_intrinsic_prototype!(async_function_prototype);
        remap_intrinsic_prototype!(async_generator_function_prototype);
        remap_intrinsic_prototype!(async_generator_prototype);
        remap_intrinsic_prototype!(generator_function_prototype);
        remap_intrinsic_prototype!(generator_prototype);
        remap_intrinsic_prototype!(array_prototype);
        remap_intrinsic_prototype!(map_prototype);
        remap_intrinsic_prototype!(map_iterator_prototype);
        remap_intrinsic_prototype!(set_prototype);
        remap_intrinsic_prototype!(set_iterator_prototype);
        remap_intrinsic_prototype!(weak_map_prototype);
        remap_intrinsic_prototype!(weak_set_prototype);
        remap_intrinsic_prototype!(weak_ref_prototype);
        remap_intrinsic_prototype!(finalization_registry_prototype);
        remap_intrinsic_prototype!(array_buffer_prototype);
        remap_intrinsic_prototype!(shared_array_buffer_prototype);
        remap_intrinsic_prototype!(data_view_prototype);
        remap_intrinsic_prototype!(typed_array_prototype);
        remap_intrinsic_prototype!(int8_array_prototype);
        remap_intrinsic_prototype!(int16_array_prototype);
        remap_intrinsic_prototype!(int32_array_prototype);
        remap_intrinsic_prototype!(float16_array_prototype);
        remap_intrinsic_prototype!(float32_array_prototype);
        remap_intrinsic_prototype!(float64_array_prototype);
        remap_intrinsic_prototype!(big_int64_array_prototype);
        remap_intrinsic_prototype!(big_uint64_array_prototype);
        remap_intrinsic_prototype!(uint32_array_prototype);
        remap_intrinsic_prototype!(uint16_array_prototype);
        remap_intrinsic_prototype!(uint8_clamped_array_prototype);
        remap_intrinsic_prototype!(uint8_array_prototype);
        remap_intrinsic_prototype!(iterator_prototype);
        remap_intrinsic_prototype!(async_iterator_prototype);
        remap_intrinsic_prototype!(async_from_sync_iterator_prototype);
        remap_intrinsic_prototype!(array_iterator_prototype);
        remap_intrinsic_prototype!(string_prototype);
        remap_intrinsic_prototype!(string_iterator_prototype);
        remap_intrinsic_prototype!(regexp_prototype);
        remap_intrinsic_prototype!(date_prototype);
        remap_intrinsic_prototype!(number_prototype);
        remap_intrinsic_prototype!(bigint_prototype);
        remap_intrinsic_prototype!(boolean_prototype);
        remap_intrinsic_prototype!(symbol_prototype);
        remap_intrinsic_prototype!(error_prototype);
        remap_intrinsic_prototype!(eval_error_prototype);
        remap_intrinsic_prototype!(range_error_prototype);
        remap_intrinsic_prototype!(reference_error_prototype);
        remap_intrinsic_prototype!(syntax_error_prototype);
        remap_intrinsic_prototype!(type_error_prototype);
        remap_intrinsic_prototype!(uri_error_prototype);
        remap_intrinsic_prototype!(aggregate_error_prototype);
        remap_intrinsic_prototype!(suppressed_error_prototype);
        remap_intrinsic_prototype!(promise_prototype);
        remap_intrinsic_prototype!(disposable_stack_prototype);
        remap_intrinsic_prototype!(async_disposable_stack_prototype);

        None
    }

    pub(super) fn map_temporal_host_result<T>(
        &mut self,
        result: Result<T, lyng_js_host::HostError>,
    ) -> Result<T, VmError> {
        result.map_err(|error| match error.kind() {
            HostErrorKind::InvalidRequest => VmError::Abrupt(errors::throw_range_error(self.agent)),
            _ => VmError::Host(error),
        })
    }

    pub(super) fn builtin_to_property_descriptor(
        &mut self,
        descriptor_object: ObjectRef,
    ) -> VmResult<PropertyDescriptor> {
        let mut descriptor = PropertyDescriptor::new();

        if self.builtin_has_property_on_object(
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::enumerable.id()),
        )? {
            let enumerable = self.builtin_get_property_value_from_object(
                descriptor_object,
                PropertyKey::from_atom(WellKnownAtom::enumerable.id()),
            )?;
            descriptor.set_enumerable(
                read::to_boolean_agent(self.agent, enumerable).map_err(VmError::Abrupt)?,
            );
        }
        if self.builtin_has_property_on_object(
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::configurable.id()),
        )? {
            let configurable = self.builtin_get_property_value_from_object(
                descriptor_object,
                PropertyKey::from_atom(WellKnownAtom::configurable.id()),
            )?;
            descriptor.set_configurable(
                read::to_boolean_agent(self.agent, configurable).map_err(VmError::Abrupt)?,
            );
        }
        if self.builtin_has_property_on_object(
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::value.id()),
        )? {
            let value = self.builtin_get_property_value_from_object(
                descriptor_object,
                PropertyKey::from_atom(WellKnownAtom::value.id()),
            )?;
            descriptor.set_value(value);
        }
        if self.builtin_has_property_on_object(
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::writable.id()),
        )? {
            let writable = self.builtin_get_property_value_from_object(
                descriptor_object,
                PropertyKey::from_atom(WellKnownAtom::writable.id()),
            )?;
            descriptor.set_writable(
                read::to_boolean_agent(self.agent, writable).map_err(VmError::Abrupt)?,
            );
        }
        if self.builtin_has_property_on_object(
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::get.id()),
        )? {
            let getter = self.builtin_get_property_value_from_object(
                descriptor_object,
                PropertyKey::from_atom(WellKnownAtom::get.id()),
            )?;
            if !(getter.is_undefined()
                || getter
                    .as_object_ref()
                    .and_then(|object| self.agent.objects().function_data(object))
                    .is_some())
            {
                return Err(VmError::Abrupt(errors::throw_type_error(self.agent)));
            }
            descriptor.set_getter(getter);
        }
        if self.builtin_has_property_on_object(
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::set.id()),
        )? {
            let setter = self.builtin_get_property_value_from_object(
                descriptor_object,
                PropertyKey::from_atom(WellKnownAtom::set.id()),
            )?;
            if !(setter.is_undefined()
                || setter
                    .as_object_ref()
                    .and_then(|object| self.agent.objects().function_data(object))
                    .is_some())
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

    pub(super) fn builtin_value_to_string_text(&mut self, value: Value) -> VmResult<String> {
        let primitive = object::to_primitive(self, value, object::ToPrimitiveHint::String)?;
        Vm::value_to_string_text(self.agent, primitive)
    }

    pub(super) fn builtin_to_property_key(&mut self, value: Value) -> VmResult<PropertyKey> {
        if let Some(symbol) = value.as_symbol_ref() {
            return Ok(PropertyKey::from_symbol(symbol));
        }
        let primitive = object::to_primitive(self, value, object::ToPrimitiveHint::String)?;
        self.vm.value_to_property_key(
            self.agent,
            self.caller_frame,
            self.caller_frame.code(),
            self.caller_frame.instruction_offset(),
            primitive,
        )
    }

    fn engine_array_to_string_fallback_value(&mut self, object: ObjectRef) -> VmResult<Value> {
        let length = self.builtin_get_property_value_from_object(
            object,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
        )?;
        let length = nonnegative_number_to_u32_length(to_f64_number(self.agent, length)?);
        let mut units = Vec::new();
        for index in 0..length {
            if index != 0 {
                units.push(u16::from(b','));
            }
            let element =
                self.builtin_get_property_value_from_object(object, PropertyKey::Index(index))?;
            if element.is_undefined() || element.is_null() {
                continue;
            }
            Vm::append_value_string_code_units(self.agent, element, &mut units)?;
        }
        Ok(Value::from_string_ref(alloc_code_unit_string(
            self.agent, &units, None,
        )))
    }
}

const fn nonnegative_number_to_u32_length(number: f64) -> u32 {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "engine array fallback length is clamped to the non-negative u32 element range"
    )]
    let length = number.max(0.0) as u32;
    length
}

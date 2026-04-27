use super::*;

impl PublicBuiltinDispatchContext for VmBuiltinDispatch<'_, '_, '_> {
    fn agent(&mut self) -> &mut Agent {
        self.agent
    }

    fn callee_object(&self) -> ObjectRef {
        self.callee_object
    }

    fn builtin_realm(&self) -> RealmRef {
        Vm::builtin_realm(self.agent, self.callee_object, self.caller_frame)
    }

    fn caller_realm(&self) -> RealmRef {
        self.caller_frame.realm()
    }

    fn caller_is_strict(&self) -> bool {
        self.vm.caller_is_strict(self.caller_frame)
    }

    fn abrupt(&mut self, completion: AbruptCompletion) -> Self::Error {
        VmError::Abrupt(completion)
    }

    fn extract_thrown_value(&mut self, error: Self::Error) -> Result<Option<Value>, Self::Error> {
        match error {
            VmError::Abrupt(completion) => Ok(completion.thrown_value()),
            other => Err(other),
        }
    }

    fn value_to_string_text(&mut self, value: Value) -> Result<String, Self::Error> {
        self.builtin_value_to_string_text(value)
    }

    fn to_property_key(&mut self, value: Value) -> Result<PropertyKey, Self::Error> {
        self.builtin_to_property_key(value)
    }

    fn get_property_value(
        &mut self,
        receiver: Value,
        key: PropertyKey,
    ) -> Result<Value, Self::Error> {
        self.builtin_get_property_value(receiver, key)
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
            self.caller_frame,
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
    ) -> Result<bool, Self::Error> {
        self.vm.set_property_on_object(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
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

    fn to_object_for_builtin_value(
        &mut self,
        realm: RealmRef,
        value: Value,
    ) -> Result<ObjectRef, Self::Error> {
        self.vm.to_object_for_value(self.agent, realm, value)
    }

    fn allocate_ordinary_object_with_prototype(
        &mut self,
        realm: RealmRef,
        prototype: Option<ObjectRef>,
    ) -> Result<ObjectRef, Self::Error> {
        self.vm
            .allocate_ordinary_object_with_prototype(self.agent, realm, prototype)
    }

    fn allocate_builtin_function(
        &mut self,
        entry: BuiltinFunctionId,
    ) -> Result<ObjectRef, Self::Error> {
        self.vm
            .allocate_builtin_function_object(self.agent, self.builtin_realm(), entry)
    }

    fn create_array_object(
        &mut self,
        realm: RealmRef,
        element_capacity: usize,
    ) -> Result<ObjectRef, Self::Error> {
        self.vm.create_array(self.agent, realm, element_capacity)
    }

    fn ordinary_constructor_prototype(
        &mut self,
        realm: RealmRef,
        new_target: Option<ObjectRef>,
        default_prototype: ObjectRef,
    ) -> Result<ObjectRef, Self::Error> {
        self.builtin_constructor_prototype(realm, default_prototype, new_target)
    }

    fn descriptor_object_from_descriptor(
        &mut self,
        realm: RealmRef,
        descriptor: PropertyDescriptor,
    ) -> Result<Value, Self::Error> {
        self.vm
            .descriptor_object_from_descriptor(self.agent, realm, descriptor)
    }

    fn to_property_descriptor(
        &mut self,
        descriptor_object: ObjectRef,
    ) -> Result<PropertyDescriptor, Self::Error> {
        self.builtin_to_property_descriptor(descriptor_object)
    }

    fn set_integrity_level(
        &mut self,
        object: ObjectRef,
        freeze: bool,
    ) -> Result<bool, Self::Error> {
        self.vm.set_integrity_level(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            object,
            freeze,
        )
    }

    fn test_integrity_level(
        &mut self,
        object: ObjectRef,
        frozen: bool,
    ) -> Result<bool, Self::Error> {
        self.vm.test_integrity_level(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            object,
            frozen,
        )
    }

    fn park_agent(
        &mut self,
        request: &lyng_js_host::ParkAgentRequest,
    ) -> Result<lyng_js_host::ParkAgentResult, Self::Error> {
        self.host.park_agent(request).map_err(VmError::Host)
    }

    fn unpark_agent(
        &mut self,
        request: &lyng_js_host::UnparkAgentRequest,
    ) -> Result<lyng_js_host::UnparkAgentResult, Self::Error> {
        self.host.unpark_agent(request).map_err(VmError::Host)
    }

    fn temporal_current_instant(
        &mut self,
        request: &TemporalCurrentInstantRequest,
    ) -> Result<TemporalInstant, Self::Error> {
        self.map_temporal_host_result(self.host.temporal_current_instant(request))
    }

    fn temporal_default_time_zone(
        &mut self,
        request: &TemporalDefaultTimeZoneRequest,
    ) -> Result<TemporalDefaultTimeZone, Self::Error> {
        self.map_temporal_host_result(self.host.temporal_default_time_zone(request))
    }

    fn temporal_instant_to_civil_time(
        &mut self,
        request: &TemporalInstantToCivilRequest,
    ) -> Result<TemporalCivilTime, Self::Error> {
        self.map_temporal_host_result(self.host.temporal_instant_to_civil_time(request))
    }

    fn temporal_civil_time_to_instant(
        &mut self,
        request: &TemporalCivilToInstantRequest,
    ) -> Result<TemporalInstantWithOffset, Self::Error> {
        self.map_temporal_host_result(self.host.temporal_civil_time_to_instant(request))
    }

    fn require_callable_object(&mut self, value: Value) -> Result<ObjectRef, Self::Error> {
        let realm = self.builtin_realm();
        let object = value.as_object_ref().ok_or_else(|| {
            Vm::abrupt_intrinsic_error(self.agent, realm, errors::ErrorKind::Type)
        })?;
        if !self.agent.objects().is_callable(object) {
            return Err(Vm::abrupt_intrinsic_error(
                self.agent,
                realm,
                errors::ErrorKind::Type,
            ));
        }
        Ok(object)
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
            self.caller_frame,
            callee_object,
            arguments,
            new_target,
        )
    }

    fn collect_array_like_arguments(
        &mut self,
        realm: RealmRef,
        value: Value,
    ) -> Result<Vec<Value>, Self::Error> {
        self.vm.collect_array_like_arguments(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            realm,
            value,
        )
    }

    fn create_bound_function(
        &mut self,
        target: ObjectRef,
        bound_this: Value,
        bound_arguments: &[Value],
    ) -> Result<ObjectRef, Self::Error> {
        self.vm.create_bound_function(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            target,
            bound_this,
            bound_arguments,
        )
    }

    fn create_dynamic_function(
        &mut self,
        realm: RealmRef,
        parameters_source: &str,
        body_source: &str,
        strict_caller: bool,
        kind: DynamicFunctionKind,
        new_target: Option<ObjectRef>,
    ) -> Result<ObjectRef, Self::Error> {
        let default_prototype = self
            .agent
            .realm(realm)
            .and_then(|record| match kind {
                DynamicFunctionKind::Ordinary => record.intrinsics().function_prototype(),
                DynamicFunctionKind::Generator => {
                    record.intrinsics().generator_function_prototype()
                }
                DynamicFunctionKind::Async => record.intrinsics().async_function_prototype(),
                DynamicFunctionKind::AsyncGenerator => {
                    record.intrinsics().async_generator_function_prototype()
                }
            })
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(self.agent)))?;
        let prototype = self.builtin_constructor_prototype(realm, default_prototype, new_target)?;
        self.vm.create_dynamic_function(
            self.agent,
            realm,
            parameters_source,
            body_source,
            strict_caller,
            kind,
            prototype,
        )
    }

    fn generator_next(&mut self, generator: ObjectRef, value: Value) -> Result<Value, Self::Error> {
        if self.vm.is_async_generator_object(generator) {
            return self.vm.resume_async_generator(
                self.agent,
                self.host,
                self.registry,
                self.caller_frame,
                generator,
                GeneratorResumeKind::Next,
                value,
            );
        }
        self.vm.resume_generator(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            generator,
            GeneratorResumeKind::Next,
            value,
        )
    }

    fn generator_return(
        &mut self,
        generator: ObjectRef,
        value: Value,
    ) -> Result<Value, Self::Error> {
        if self.vm.is_async_generator_object(generator) {
            return self.vm.resume_async_generator(
                self.agent,
                self.host,
                self.registry,
                self.caller_frame,
                generator,
                GeneratorResumeKind::Return,
                value,
            );
        }
        self.vm.resume_generator(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            generator,
            GeneratorResumeKind::Return,
            value,
        )
    }

    fn generator_throw(
        &mut self,
        generator: ObjectRef,
        value: Value,
    ) -> Result<Value, Self::Error> {
        if self.vm.is_async_generator_object(generator) {
            return self.vm.resume_async_generator(
                self.agent,
                self.host,
                self.registry,
                self.caller_frame,
                generator,
                GeneratorResumeKind::Throw,
                value,
            );
        }
        self.vm.resume_generator(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            generator,
            GeneratorResumeKind::Throw,
            value,
        )
    }

    fn async_generator_next(
        &mut self,
        this_value: Value,
        value: Value,
    ) -> Result<Value, Self::Error> {
        self.vm.resume_async_generator_from_value(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            this_value,
            GeneratorResumeKind::Next,
            value,
        )
    }

    fn async_generator_return(
        &mut self,
        this_value: Value,
        value: Value,
    ) -> Result<Value, Self::Error> {
        self.vm.resume_async_generator_from_value(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            this_value,
            GeneratorResumeKind::Return,
            value,
        )
    }

    fn async_generator_throw(
        &mut self,
        this_value: Value,
        value: Value,
    ) -> Result<Value, Self::Error> {
        self.vm.resume_async_generator_from_value(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            this_value,
            GeneratorResumeKind::Throw,
            value,
        )
    }

    fn evaluate_script_in_realm(
        &mut self,
        realm: RealmRef,
        source_text: &str,
    ) -> Result<Value, Self::Error> {
        self.vm.evaluate_indirect_eval_source(
            self.agent,
            self.host,
            self.registry,
            realm,
            source_text,
        )
    }

    fn function_to_string_text(&mut self, function: ObjectRef) -> Result<String, Self::Error> {
        if !self.agent.objects().is_callable(function) {
            return Err(VmError::Abrupt(errors::throw_type_error(self.agent)));
        }
        let Some(entry) = self
            .agent
            .objects()
            .function_data(function)
            .and_then(|data| data.entry())
        else {
            return self.vm.native_function_source_text(self.agent, function);
        };
        match entry {
            lyng_js_objects::FunctionEntryIdentity::Bytecode(code) => self
                .vm
                .source_function_source_text(self.agent, code, function),
            lyng_js_objects::FunctionEntryIdentity::Native(_)
            | lyng_js_objects::FunctionEntryIdentity::Bound => {
                self.vm.native_function_source_text(self.agent, function)
            }
        }
    }
}

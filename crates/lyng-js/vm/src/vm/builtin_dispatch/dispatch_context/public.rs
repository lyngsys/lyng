use super::{
    errors, AbruptCompletion, Agent, AllocationLifetime, BuiltinFunctionId, DynamicFunctionKind,
    GeneratorResumeKind, ObjectRef, PropertyDescriptor, PropertyKey, PublicBuiltinDispatchContext,
    RealmRef, TemporalCivilTime, TemporalCivilToInstantRequest, TemporalCurrentInstantRequest,
    TemporalDefaultTimeZone, TemporalDefaultTimeZoneRequest, TemporalInstant,
    TemporalInstantToCivilRequest, TemporalInstantWithOffset, Value, Vm, VmBuiltinDispatch,
    VmError,
};

const MAX_REUSABLE_STRING_CODE_UNITS: usize = 1 << 20;

impl PublicBuiltinDispatchContext for VmBuiltinDispatch<'_, '_, '_> {
    fn agent(&mut self) -> &mut Agent {
        self.agent
    }

    fn take_string_code_units_scratch(&mut self) -> Vec<u16> {
        std::mem::take(&mut self.vm.string_code_units_scratch)
    }

    fn recycle_string_code_units_scratch(&mut self, mut units: Vec<u16>) {
        if units.capacity() > MAX_REUSABLE_STRING_CODE_UNITS {
            return;
        }
        units.clear();
        if units.capacity() > self.vm.string_code_units_scratch.capacity() {
            self.vm.string_code_units_scratch = units;
        }
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
        self.vm.get_own_property_from_object(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
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
        self.vm.evaluate_deferred_module_namespace(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            object,
            key,
        )?;
        if let Some(result) = self.vm.define_typed_array_numeric_property(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            object,
            key,
            descriptor,
        )? {
            return Ok(result);
        }
        self.vm
            .define_property_on_object(self.agent, object, key, descriptor, lifetime)
    }

    fn try_fast_create_data_property(
        &mut self,
        object: ObjectRef,
        index: u32,
        value: Value,
    ) -> Result<bool, Self::Error> {
        let result = self.agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.fast_set_engine_array_index(
                &mut mutator,
                object,
                index,
                value,
                AllocationLifetime::Default,
            )
        });
        match result {
            Ok(Some(true)) => Ok(true),
            Ok(Some(false) | None) => Ok(false),
            Err(_) => Err(VmError::Abrupt(errors::throw_type_error(self.agent))),
        }
    }

    fn try_fast_has_own_index_property(
        &mut self,
        object: ObjectRef,
        index: u32,
    ) -> Result<Option<bool>, Self::Error> {
        self.agent
            .objects()
            .fast_has_own_index_property(self.agent.heap().view(), object, index)
            .map_err(|_| VmError::Abrupt(errors::throw_type_error(self.agent)))
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
            self.caller_frame,
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
            self.caller_frame,
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
            self.caller_frame,
            object,
            key,
        )
    }

    fn to_object_for_builtin_value(
        &mut self,
        realm: RealmRef,
        value: Value,
    ) -> Result<ObjectRef, Self::Error> {
        Vm::to_object_for_value(self.agent, realm, value)
    }

    fn allocate_ordinary_object_with_prototype(
        &mut self,
        realm: RealmRef,
        prototype: Option<ObjectRef>,
    ) -> Result<ObjectRef, Self::Error> {
        Vm::allocate_ordinary_object_with_prototype(self.agent, realm, prototype)
    }

    fn allocate_builtin_function(
        &mut self,
        entry: BuiltinFunctionId,
    ) -> Result<ObjectRef, Self::Error> {
        Vm::allocate_builtin_function_object(self.agent, self.builtin_realm(), entry)
    }

    fn create_array_object(
        &mut self,
        realm: RealmRef,
        element_capacity: usize,
    ) -> Result<ObjectRef, Self::Error> {
        Vm::create_array(self.agent, realm, element_capacity)
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
        Vm::descriptor_object_from_descriptor(self.agent, realm, descriptor)
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

    fn temporal_default_time_zone_is_utc(
        &mut self,
        request: &TemporalDefaultTimeZoneRequest,
    ) -> Result<bool, Self::Error> {
        self.map_temporal_host_result(self.host.temporal_default_time_zone_is_utc(request))
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

    fn try_fast_apply_builtin(
        &mut self,
        target: ObjectRef,
        this_value: Value,
        arguments: Value,
    ) -> Result<Option<Value>, Self::Error> {
        self.vm
            .try_fast_apply_builtin(self.agent, target, this_value, arguments)
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
        let installed = self.vm.install_dynamic_function(
            self.agent,
            realm,
            parameters_source,
            body_source,
            strict_caller,
            kind,
        )?;
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
        self.vm.instantiate_dynamic_function(
            self.agent,
            self.host,
            self.registry,
            realm,
            installed,
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
            .and_then(lyng_js_objects::FunctionObjectData::entry)
        else {
            return Vm::native_function_source_text(self.agent, function);
        };
        match entry {
            lyng_js_objects::FunctionEntryIdentity::Bytecode(code) => self
                .vm
                .source_function_source_text(self.agent, code, function),
            lyng_js_objects::FunctionEntryIdentity::Native(_) => {
                Vm::native_function_source_text(self.agent, function)
            }
            lyng_js_objects::FunctionEntryIdentity::Bound => {
                Ok("function () { [native code] }".to_owned())
            }
        }
    }
}

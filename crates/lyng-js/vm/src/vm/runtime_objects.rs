use super::property_access::VmProxyBridge;
use super::{
    Agent, AllocationLifetime, BytecodeFunction, FrameRecord, HostHooks, NativeFunctionRegistry,
    ObjectAllocation, ObjectRef, RealmRef, Value, Vm, VmError, VmResult, WellKnownAtom,
};
use lyng_js_objects::{
    FunctionConstructorFlags, FunctionKindFlags, FunctionObjectData, FunctionThisMode,
    InternalMethodError, ObjectColdData, ObjectFlags, OrdinaryObjectData, RegExpPayload,
};
use lyng_js_ops::{enumeration::ForInEnumerator, errors, iterator, object, proxy};
use lyng_js_types::{PropertyDescriptor, PropertyKey};

fn map_internal_method_error(agent: &mut Agent, error: InternalMethodError) -> VmError {
    let abrupt = match error {
        InternalMethodError::RangeError => errors::throw_range_error(agent),
        InternalMethodError::ReferenceError => errors::throw_reference_error(agent),
        _ => errors::throw_type_error(agent),
    };
    VmError::Abrupt(abrupt)
}

pub(super) struct VmIteratorBridge<'a> {
    pub(super) vm: &'a mut Vm,
    pub(super) agent: &'a mut Agent,
    pub(super) host: &'a dyn HostHooks,
    pub(super) registry: &'a mut dyn NativeFunctionRegistry,
    pub(super) frame: &'a FrameRecord,
}

impl iterator::IteratorOpsContext for VmIteratorBridge<'_> {
    type Error = VmError;

    fn agent(&mut self) -> &mut Agent {
        self.agent
    }

    fn realm(&self) -> RealmRef {
        self.frame.realm()
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

    fn require_callable_object(&mut self, value: Value) -> Result<ObjectRef, Self::Error> {
        Vm::require_callable_object(self.agent, *self.frame, value)
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
    pub(super) fn create_object(
        agent: &mut Agent,
        realm: RealmRef,
        _named_slot_count: usize,
    ) -> VmResult<ObjectRef> {
        let root_shape = agent
            .realm(realm)
            .and_then(|realm| realm.root_shape())
            .ok_or(VmError::MissingRootShape(realm))?;
        let prototype = agent
            .realm(realm)
            .and_then(|realm| realm.intrinsics().object_prototype());
        Ok(agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape).with_prototype(prototype),
                AllocationLifetime::Default,
            )
        }))
    }

    pub(super) fn create_array(
        agent: &mut Agent,
        realm: RealmRef,
        element_capacity: usize,
    ) -> VmResult<ObjectRef> {
        let realm_record = agent.realm(realm).ok_or(VmError::MissingRootShape(realm))?;
        let root_shape = realm_record
            .root_shape()
            .ok_or(VmError::MissingRootShape(realm))?;
        let prototype = realm_record.intrinsics().array_prototype();
        let object = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape)
                    .with_flags(ObjectFlags::extensible().union(ObjectFlags::ENGINE_ARRAY))
                    .with_prototype(prototype)
                    .with_element_capacity(element_capacity),
                AllocationLifetime::Default,
            )
        });
        Self::define_length_property(agent, object, 0, false)?;
        Ok(object)
    }

    pub(super) fn create_closure(
        &self,
        agent: &mut Agent,
        frame: &FrameRecord,
        child_index: u32,
    ) -> VmResult<ObjectRef> {
        let child_code = self
            .installed_child_code(frame.code(), child_index)
            .ok_or_else(|| VmError::MissingInstalledCode(frame.code()))?;
        let child = self
            .installed_function(child_code)
            .ok_or(VmError::MissingInstalledCode(child_code))?;
        let constructor_flags = bytecode_constructor_flags(child);
        let kind_flags = bytecode_kind_flags(child);
        let root_shape = agent
            .realm(frame.realm())
            .and_then(|realm| realm.root_shape())
            .ok_or_else(|| VmError::MissingRootShape(frame.realm()))?;
        let home_object = if child.flags().has_prototype_property() {
            Some(Self::create_function_prototype(
                agent,
                frame.realm(),
                prototype_parent_for_function(agent, frame.realm(), kind_flags)?,
            )?)
        } else {
            None
        };
        let private_env = agent
            .current_execution_context()
            .and_then(lyng_js_env::ExecutionContext::private_env);
        let environment = if child.captures().is_empty() {
            let lexical_env = frame.lexical_env();
            if matches!(
                agent.environment(lexical_env),
                Some(lyng_js_env::EnvironmentRecord::Object(_))
            ) {
                lexical_env
            } else {
                self.active_direct_eval_environment(self.frames.len())
                    .or_else(|| {
                        self.active_loop_iteration_environment_for_captures(child.captures())
                    })
                    .unwrap_or(lexical_env)
            }
        } else {
            self.active_loop_iteration_environment_for_captures(child.captures())
                .unwrap_or_else(|| frame.lexical_env())
        };
        let function_data = FunctionObjectData::bytecode(frame.realm(), environment, child_code)
            .with_private_env(private_env)
            .with_this_mode(bytecode_this_mode(child))
            .with_has_prototype_property(child.flags().has_prototype_property())
            .with_constructor_flags(constructor_flags)
            .with_kind_flags(kind_flags);
        let function_prototype = callable_prototype_for_function(agent, frame.realm(), kind_flags)?;
        let function = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::function(root_shape)
                    .with_prototype(function_prototype)
                    .with_cold_data(ObjectColdData::Function(function_data)),
                AllocationLifetime::Default,
            )
        });

        if let Some(prototype) = home_object
            && !kind_flags.is_generator()
        {
            Self::define_data_property_with_attrs(
                agent,
                prototype,
                PropertyKey::from_atom(WellKnownAtom::constructor.id()),
                Value::from_object_ref(function),
                true,
                false,
                true,
            )?;
        }

        Self::define_data_property_with_attrs(
            agent,
            function,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
            length_value(u32::from(child.minimum_argument_count())),
            false,
            false,
            true,
        )?;

        if let Some(name) = child.name() {
            let name = self.canonical_atom_for_code(child_code, name);
            let name_text = agent.atoms().resolve(name).to_owned();
            let name_value =
                Value::from_string_ref(super::values::alloc_atom_string(agent, name, &name_text));
            Self::set_function_name(agent, function, name_value)?;
        }

        if let Some(prototype) = home_object {
            Self::define_data_property_with_attrs(
                agent,
                function,
                PropertyKey::from_atom(WellKnownAtom::prototype.id()),
                Value::from_object_ref(prototype),
                !kind_flags.is_class_constructor(),
                false,
                false,
            )?;
        }

        Ok(function)
    }

    pub(super) fn create_function_prototype(
        agent: &mut Agent,
        realm: RealmRef,
        prototype_parent: ObjectRef,
    ) -> VmResult<ObjectRef> {
        let root_shape = agent
            .realm(realm)
            .and_then(|realm| realm.root_shape())
            .ok_or(VmError::MissingRootShape(realm))?;
        Ok(agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape).with_prototype(Some(prototype_parent)),
                AllocationLifetime::Default,
            )
        }))
    }

    pub(super) fn allocate_regexp_object_with_payload(
        agent: &mut Agent,
        realm: RealmRef,
        payload: RegExpPayload,
    ) -> VmResult<ObjectRef> {
        let realm_record = agent.realm(realm).ok_or(VmError::MissingRootShape(realm))?;
        let root_shape = realm_record
            .root_shape()
            .ok_or(VmError::MissingRootShape(realm))?;
        let prototype = realm_record
            .intrinsics()
            .regexp_prototype()
            .ok_or(VmError::MissingRootShape(realm))?;
        let object = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let object = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape)
                    .with_prototype(Some(prototype))
                    .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::RegExp)),
                AllocationLifetime::Default,
            );
            let stored = objects.store_regexp_payload(object, payload);
            debug_assert!(stored, "fresh RegExp objects should accept payload storage");
            object
        });
        let key = PropertyKey::from_atom(agent.bootstrap_atoms().last_index());
        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_value(Value::from_smi(0));
        descriptor.set_writable(true);
        descriptor.set_enumerable(false);
        descriptor.set_configurable(false);
        let defined = object::ordinary_define_property(
            agent,
            object,
            key,
            descriptor,
            AllocationLifetime::Default,
        )
        .map_err(VmError::Abrupt)?;
        if defined {
            Ok(object)
        } else {
            Err(VmError::Abrupt(errors::throw_type_error(agent)))
        }
    }

    pub(super) fn define_data_property_with_attrs(
        agent: &mut Agent,
        object: ObjectRef,
        key: PropertyKey,
        value: Value,
        writable: bool,
        enumerable: bool,
        configurable: bool,
    ) -> VmResult<()> {
        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_value(value);
        descriptor.set_writable(writable);
        descriptor.set_enumerable(enumerable);
        descriptor.set_configurable(configurable);
        let defined = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.define_own_property(
                &mut mutator,
                object,
                key,
                descriptor,
                AllocationLifetime::Default,
            )
        });
        let _ = defined.map_err(|error| map_internal_method_error(agent, error))?;
        Ok(())
    }

    pub(super) fn set_function_name(
        agent: &mut Agent,
        function: ObjectRef,
        name_value: Value,
    ) -> VmResult<()> {
        let name_value = name_value.as_symbol_ref().map_or(name_value, |symbol| {
            let name =
                function_name_text_from_property_key(agent, PropertyKey::from_symbol(symbol), None);
            Value::from_string_ref(agent.alloc_runtime_string(
                &name,
                None,
                AllocationLifetime::Default,
            ))
        });
        let key = PropertyKey::from_atom(WellKnownAtom::name.id());
        let existing =
            object::ordinary_get_own_property(agent, function, key).map_err(VmError::Abrupt)?;
        if let Some(existing) = existing {
            let can_overwrite = existing
                .value()
                .and_then(Value::as_string_ref)
                .is_some_and(|value| string_ref_is_empty(agent, value))
                && name_value
                    .as_string_ref()
                    .is_some_and(|value| !string_ref_is_empty(agent, value));
            if !can_overwrite {
                return Ok(());
            }
        }
        Self::define_data_property_with_attrs(agent, function, key, name_value, false, false, true)
    }

    pub(super) fn set_function_name_from_property_key(
        agent: &mut Agent,
        function: ObjectRef,
        key: PropertyKey,
        prefix: Option<&str>,
    ) -> VmResult<()> {
        let name_value = if let (None, PropertyKey::Atom(atom)) = (prefix, key) {
            let name = agent.atoms().resolve(atom).to_owned();
            Value::from_string_ref(agent.alloc_runtime_string(
                &name,
                Some(atom),
                AllocationLifetime::Default,
            ))
        } else {
            let name = function_name_text_from_property_key(agent, key, prefix);
            Value::from_string_ref(agent.alloc_runtime_string(
                &name,
                None,
                AllocationLifetime::Default,
            ))
        };
        Self::set_function_name(agent, function, name_value)
    }

    pub(super) fn to_object_for_value(
        agent: &mut Agent,
        realm: RealmRef,
        value: Value,
    ) -> VmResult<ObjectRef> {
        object::to_object(agent, realm, value).map_err(VmError::Abrupt)
    }

    pub(super) fn check_object_coercible(agent: &mut Agent, value: Value) -> VmResult<()> {
        if value.is_null() || value.is_undefined() {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(())
    }

    pub(super) fn delete_property_from_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        receiver: Value,
        key: PropertyKey,
    ) -> VmResult<bool> {
        let object = Self::to_object_for_value(agent, frame.realm(), receiver)?;
        if agent.objects().is_proxy_object(object) {
            let mut bridge = VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame,
            };
            return proxy::delete_property(&mut bridge, object, key);
        }
        self.evaluate_deferred_module_namespace(agent, host, registry, frame, object, key)?;
        self.delete_property_from_object(agent, object, key)
    }

    pub(super) fn create_for_in_enumerator_for_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        receiver: Value,
    ) -> VmResult<ForInEnumerator> {
        if receiver.is_null() || receiver.is_undefined() {
            return Ok(ForInEnumerator::new(Vec::new()));
        }
        let object = Self::to_object_for_value(agent, frame.realm(), receiver)?;
        let mut visited = std::collections::HashSet::new();
        let mut keys = Vec::new();
        let mut current = Some(object);
        let mut bridge = VmProxyBridge {
            vm: self,
            agent,
            host,
            registry,
            frame,
        };

        while let Some(object) = current {
            let own_keys = object::own_property_keys_in_context(&mut bridge, object)?;
            for key in own_keys {
                if key.is_symbol() || !visited.insert(key) {
                    continue;
                }
                let descriptor = object::get_own_property_in_context(&mut bridge, object, key)?;
                if descriptor.is_some_and(|descriptor| descriptor.enumerable() == Some(true)) {
                    keys.push((object, key));
                }
            }
            current = object::get_prototype_of_in_context(&mut bridge, object)?;
        }

        Ok(ForInEnumerator::new(keys))
    }

    pub(super) fn create_iterator_for_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        value: Value,
        async_iteration: bool,
    ) -> VmResult<iterator::IteratorRecord> {
        let mut bridge = VmIteratorBridge {
            vm: self,
            agent,
            host,
            registry,
            frame,
        };
        if async_iteration {
            iterator::get_async_iterator(&mut bridge, value)
        } else {
            iterator::get_iterator(&mut bridge, value)
        }
    }

    pub(super) fn append_iterator_values(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        value: Value,
        values: &mut Vec<Value>,
    ) -> VmResult<()> {
        let mut bridge = VmIteratorBridge {
            vm: self,
            agent,
            host,
            registry,
            frame,
        };
        let mut iterator = iterator::get_iterator(&mut bridge, value)?;
        loop {
            let Some(result) = iterator::iterator_step(&mut bridge, &mut iterator)? else {
                return Ok(());
            };
            match iterator::iterator_value(&mut bridge, result) {
                Ok(value) => values.push(value),
                Err(VmError::Abrupt(abrupt)) => {
                    let _: () = iterator::iterator_close(&mut bridge, &mut iterator, Err(abrupt))?;
                    return Ok(());
                }
                Err(error) => return Err(error),
            }
        }
    }

    pub(super) fn advance_iterator_state(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        iterator_register: u16,
    ) -> VmResult<Option<Value>> {
        let mut record = self
            .iterator_states
            .remove(frame.registers().base(), iterator_register)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        if record.is_async() {
            return self.advance_async_iterator_state(
                agent,
                host,
                registry,
                frame,
                iterator_register,
                record,
            );
        }
        let mut bridge = VmIteratorBridge {
            vm: self,
            agent,
            host,
            registry,
            frame,
        };
        let Some(result) = iterator::iterator_step(&mut bridge, &mut record)? else {
            self.iterator_states
                .insert(frame.registers().base(), iterator_register, record);
            return Ok(None);
        };
        let value = iterator::iterator_value(&mut bridge, result)?;
        self.iterator_states
            .insert(frame.registers().base(), iterator_register, record);
        Ok(Some(value))
    }

    pub(super) fn close_iterator_state(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        iterator_register: u16,
        preserve_completion: bool,
    ) -> VmResult<()> {
        let Some(mut record) = self
            .iterator_states
            .remove(frame.registers().base(), iterator_register)
        else {
            return Ok(());
        };
        if record.is_async() {
            return self.close_async_iterator_state(
                agent,
                host,
                registry,
                frame,
                iterator_register,
                record,
                preserve_completion,
            );
        }
        if preserve_completion {
            self.close_iterator_state_preserving_completion(agent, host, registry, frame, record);
            return Ok(());
        }
        let mut bridge = VmIteratorBridge {
            vm: self,
            agent,
            host,
            registry,
            frame,
        };
        let _: () = iterator::iterator_close(&mut bridge, &mut record, Ok(()))?;
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn async_from_sync_iterator_continuation(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        value: Value,
        done: bool,
        close_on_rejection: Option<(ObjectRef, ObjectRef)>,
    ) -> VmResult<ObjectRef> {
        let capability = Self::create_intrinsic_promise_capability(agent, frame.realm())?;
        let promise = Self::promise_capability_promise(agent, capability)?;
        let value_wrapper =
            match self.promise_resolve_in_realm(agent, host, registry, frame, frame.realm(), value)
            {
                Ok(value_wrapper) => value_wrapper,
                Err(VmError::Abrupt(completion)) => {
                    let completion = if done {
                        completion
                    } else if let Some((iterator, next_method)) = close_on_rejection {
                        let mut record =
                            iterator::IteratorRecord::new_async_from_sync(iterator, next_method);
                        let mut bridge = VmIteratorBridge {
                            vm: self,
                            agent,
                            host,
                            registry,
                            frame,
                        };
                        match iterator::iterator_close::<_, ()>(
                            &mut bridge,
                            &mut record,
                            Err(completion),
                        ) {
                            Ok(()) => completion,
                            Err(VmError::Abrupt(completion)) => completion,
                            Err(error) => return Err(error),
                        }
                    } else {
                        completion
                    };
                    let realm = agent
                        .realm(frame.realm())
                        .ok_or_else(|| VmError::MissingRootShape(frame.realm()))?;
                    self.settle_promise_capability(
                        agent,
                        host,
                        registry,
                        &realm,
                        capability,
                        true,
                        completion.thrown_value().unwrap_or(Value::undefined()),
                    )?;
                    return Ok(promise);
                }
                Err(error) => return Err(error),
            };
        let realm = agent
            .realm(frame.realm())
            .ok_or_else(|| VmError::MissingRootShape(frame.realm()))?;
        Self::enqueue_promise_then(
            agent,
            &realm,
            value_wrapper,
            lyng_js_env::PromiseReactionHandler::AsyncFromSyncIteratorValue { done },
            if done {
                lyng_js_env::PromiseReactionHandler::Thrower
            } else {
                close_on_rejection.map_or(
                    lyng_js_env::PromiseReactionHandler::Thrower,
                    |(iterator, next_method)| {
                        lyng_js_env::PromiseReactionHandler::AsyncFromSyncIteratorReject {
                            iterator,
                            next_method,
                        }
                    },
                )
            },
            Some(capability),
        )?;
        Ok(promise)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "spec-shaped VM routine stays contiguous to preserve completion ordering and cleanup invariants"
    )]
    fn advance_async_iterator_state(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        iterator_register: u16,
        mut record: iterator::IteratorRecord,
    ) -> VmResult<Option<Value>> {
        if frame.resume_active() {
            let resume_kind = frame.resume_kind();
            let resume_value = frame.resume_value();
            self.clear_active_resume();
            if resume_kind == crate::frame::GeneratorResumeKind::Throw {
                return Err(VmError::Abrupt(lyng_js_types::AbruptCompletion::Throw(
                    resume_value,
                )));
            }

            let value = match record.kind() {
                iterator::IteratorKind::Async => {
                    let iter_result = resume_value
                        .as_object_ref()
                        .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
                    let mut bridge = VmIteratorBridge {
                        vm: self,
                        agent,
                        host,
                        registry,
                        frame,
                    };
                    if iterator::iterator_complete(&mut bridge, iter_result)? {
                        record.set_done(true);
                        self.iterator_states.insert(
                            frame.registers().base(),
                            iterator_register,
                            record,
                        );
                        return Ok(None);
                    }
                    iterator::iterator_value(&mut bridge, iter_result)?
                }
                iterator::IteratorKind::AsyncFromSync => {
                    let state = record.async_from_sync_state();
                    record.set_async_from_sync_state(iterator::AsyncFromSyncState::None);
                    match state {
                        iterator::AsyncFromSyncState::Next { .. } => {
                            let iter_result = resume_value
                                .as_object_ref()
                                .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
                            let mut bridge = VmIteratorBridge {
                                vm: self,
                                agent,
                                host,
                                registry,
                                frame,
                            };
                            if iterator::iterator_complete(&mut bridge, iter_result)? {
                                record.set_done(true);
                                self.iterator_states.insert(
                                    frame.registers().base(),
                                    iterator_register,
                                    record,
                                );
                                return Ok(None);
                            }
                            iterator::iterator_value(&mut bridge, iter_result)?
                        }
                        iterator::AsyncFromSyncState::None
                        | iterator::AsyncFromSyncState::Return => {
                            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
                        }
                    }
                }
                iterator::IteratorKind::Sync => {
                    return Err(VmError::Abrupt(errors::throw_type_error(agent)));
                }
            };
            self.iterator_states
                .insert(frame.registers().base(), iterator_register, record);
            return Ok(Some(value));
        }

        let receiver = Value::from_object_ref(record.iterator());
        let next_method = Self::require_callable_object(agent, *frame, record.next_method())?;
        let result =
            self.call_to_completion(agent, host, registry, frame, next_method, receiver, &[])?;
        match record.kind() {
            iterator::IteratorKind::Async => {
                let promise = self.promise_resolve_in_realm(
                    agent,
                    host,
                    registry,
                    frame,
                    frame.realm(),
                    result,
                )?;
                self.iterator_states
                    .insert(frame.registers().base(), iterator_register, record);
                self.suspend_for_await_promise(agent, frame, promise)?;
                Ok(None)
            }
            iterator::IteratorKind::AsyncFromSync => {
                let iter_result = result
                    .as_object_ref()
                    .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
                let (done, value) = {
                    let mut bridge = VmIteratorBridge {
                        vm: self,
                        agent,
                        host,
                        registry,
                        frame,
                    };
                    (
                        iterator::iterator_complete(&mut bridge, iter_result)?,
                        iterator::iterator_value(&mut bridge, iter_result)?,
                    )
                };
                let promise = self.async_from_sync_iterator_continuation(
                    agent,
                    host,
                    registry,
                    frame,
                    value,
                    done,
                    record
                        .next_method()
                        .as_object_ref()
                        .map(|next_method| (record.iterator(), next_method)),
                )?;
                let promise = self.promise_resolve_in_realm(
                    agent,
                    host,
                    registry,
                    frame,
                    frame.realm(),
                    Value::from_object_ref(promise),
                )?;
                record.set_async_from_sync_state(iterator::AsyncFromSyncState::Next { done });
                self.iterator_states
                    .insert(frame.registers().base(), iterator_register, record);
                self.suspend_for_await_promise(agent, frame, promise)?;
                Ok(None)
            }
            iterator::IteratorKind::Sync => Err(VmError::Abrupt(errors::throw_type_error(agent))),
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "spec-shaped VM routine stays contiguous to preserve completion ordering and cleanup invariants"
    )]
    fn close_async_iterator_state(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        iterator_register: u16,
        mut record: iterator::IteratorRecord,
        preserve_completion: bool,
    ) -> VmResult<()> {
        if frame.resume_active() {
            let resume_kind = frame.resume_kind();
            let resume_value = frame.resume_value();
            let preserve_completion = record.preserve_completion_on_close();
            record.set_preserve_completion_on_close(false);
            self.clear_active_resume();
            if resume_kind == crate::frame::GeneratorResumeKind::Throw {
                if preserve_completion {
                    record.set_done(true);
                    return Ok(());
                }
                return Err(VmError::Abrupt(lyng_js_types::AbruptCompletion::Throw(
                    resume_value,
                )));
            }
            match record.kind() {
                iterator::IteratorKind::Async => {
                    if resume_value.as_object_ref().is_none() {
                        if preserve_completion {
                            record.set_done(true);
                            return Ok(());
                        }
                        return Err(VmError::Abrupt(errors::throw_type_error(agent)));
                    }
                }
                iterator::IteratorKind::AsyncFromSync => {
                    let state = record.async_from_sync_state();
                    record.set_async_from_sync_state(iterator::AsyncFromSyncState::None);
                    if state != iterator::AsyncFromSyncState::Return {
                        if preserve_completion {
                            record.set_done(true);
                            return Ok(());
                        }
                        return Err(VmError::Abrupt(errors::throw_type_error(agent)));
                    }
                }
                iterator::IteratorKind::Sync => {
                    if preserve_completion {
                        record.set_done(true);
                        return Ok(());
                    }
                    return Err(VmError::Abrupt(errors::throw_type_error(agent)));
                }
            }
            record.set_done(true);
            return Ok(());
        }

        if record.done() {
            return Ok(());
        }
        let receiver = Value::from_object_ref(record.iterator());
        let return_value = self.get_property_from_value(
            agent,
            host,
            registry,
            frame,
            receiver,
            PropertyKey::from_atom(WellKnownAtom::r#return.id()),
        );
        let return_value = match return_value {
            Ok(return_value) => return_value,
            Err(_) if preserve_completion => {
                record.set_done(true);
                return Ok(());
            }
            Err(error) => return Err(error),
        };
        if return_value.is_undefined() || return_value.is_null() {
            record.set_done(true);
            return Ok(());
        }
        let return_method = match Self::require_callable_object(agent, *frame, return_value) {
            Ok(return_method) => return_method,
            Err(_) if preserve_completion => {
                record.set_done(true);
                return Ok(());
            }
            Err(error) => return Err(error),
        };
        let result =
            self.call_to_completion(agent, host, registry, frame, return_method, receiver, &[]);
        let result = match result {
            Ok(result) => result,
            Err(_) if preserve_completion => {
                record.set_done(true);
                return Ok(());
            }
            Err(error) => return Err(error),
        };
        match record.kind() {
            iterator::IteratorKind::Async => {
                let promise = match self.promise_resolve_in_realm(
                    agent,
                    host,
                    registry,
                    frame,
                    frame.realm(),
                    result,
                ) {
                    Ok(promise) => promise,
                    Err(_) if preserve_completion => {
                        record.set_done(true);
                        return Ok(());
                    }
                    Err(error) => return Err(error),
                };
                record.set_preserve_completion_on_close(preserve_completion);
                self.iterator_states
                    .insert(frame.registers().base(), iterator_register, record);
                self.suspend_for_await_promise(agent, frame, promise)
            }
            iterator::IteratorKind::AsyncFromSync => {
                let iter_result = match result.as_object_ref() {
                    Some(iter_result) => iter_result,
                    None if preserve_completion => {
                        record.set_done(true);
                        return Ok(());
                    }
                    None => return Err(VmError::Abrupt(errors::throw_type_error(agent))),
                };
                let value = {
                    let mut bridge = VmIteratorBridge {
                        vm: self,
                        agent,
                        host,
                        registry,
                        frame,
                    };
                    match iterator::iterator_value(&mut bridge, iter_result) {
                        Ok(value) => value,
                        Err(_) if preserve_completion => {
                            record.set_done(true);
                            return Ok(());
                        }
                        Err(error) => return Err(error),
                    }
                };
                let promise = match self.async_from_sync_iterator_continuation(
                    agent, host, registry, frame, value, true, None,
                ) {
                    Ok(promise) => promise,
                    Err(_) if preserve_completion => {
                        record.set_done(true);
                        return Ok(());
                    }
                    Err(error) => return Err(error),
                };
                let promise = match self.promise_resolve_in_realm(
                    agent,
                    host,
                    registry,
                    frame,
                    frame.realm(),
                    Value::from_object_ref(promise),
                ) {
                    Ok(promise) => promise,
                    Err(_) if preserve_completion => {
                        record.set_done(true);
                        return Ok(());
                    }
                    Err(error) => return Err(error),
                };
                record.set_async_from_sync_state(iterator::AsyncFromSyncState::Return);
                record.set_preserve_completion_on_close(preserve_completion);
                self.iterator_states
                    .insert(frame.registers().base(), iterator_register, record);
                self.suspend_for_await_promise(agent, frame, promise)
            }
            iterator::IteratorKind::Sync => Err(VmError::Abrupt(errors::throw_type_error(agent))),
        }
    }

    fn close_iterator_state_preserving_completion(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        mut record: iterator::IteratorRecord,
    ) {
        if record.done() {
            return;
        }
        record.set_done(true);
        let receiver = Value::from_object_ref(record.iterator());
        let Ok(return_value) = self.get_property_from_value(
            agent,
            host,
            registry,
            frame,
            receiver,
            PropertyKey::from_atom(WellKnownAtom::r#return.id()),
        ) else {
            return;
        };
        if return_value.is_undefined() || return_value.is_null() {
            return;
        }
        let Ok(return_method) = Self::require_callable_object(agent, *frame, return_value) else {
            return;
        };
        let Ok(result) =
            self.call_to_completion(agent, host, registry, frame, return_method, receiver, &[])
        else {
            return;
        };
        let _ = result.as_object_ref();
    }
}

const fn bytecode_this_mode(function: &BytecodeFunction) -> lyng_js_objects::FunctionThisMode {
    match function.this_mode() {
        lyng_js_bytecode::ThisMode::Lexical => FunctionThisMode::Lexical,
        lyng_js_bytecode::ThisMode::Strict => FunctionThisMode::Strict,
        lyng_js_bytecode::ThisMode::Global => FunctionThisMode::Global,
    }
}

const fn bytecode_constructor_flags(function: &BytecodeFunction) -> FunctionConstructorFlags {
    match function.kind() {
        lyng_js_bytecode::BytecodeFunctionKind::Function => {
            FunctionConstructorFlags::empty().with_constructible(function.flags().constructible())
        }
        lyng_js_bytecode::BytecodeFunctionKind::Arrow
        | lyng_js_bytecode::BytecodeFunctionKind::Module
        | lyng_js_bytecode::BytecodeFunctionKind::Script
        | lyng_js_bytecode::BytecodeFunctionKind::Builtin => FunctionConstructorFlags::empty(),
    }
}

const fn bytecode_kind_flags(function: &BytecodeFunction) -> FunctionKindFlags {
    let mut flags = match function.kind() {
        lyng_js_bytecode::BytecodeFunctionKind::Arrow => FunctionKindFlags::ARROW,
        _ => FunctionKindFlags::empty(),
    };
    if function.flags().class_constructor() {
        flags = flags.union(FunctionKindFlags::CLASS_CONSTRUCTOR);
    }
    if function.flags().generator() {
        flags = flags.union(FunctionKindFlags::GENERATOR);
    }
    if function.flags().async_function() {
        flags = flags.union(FunctionKindFlags::ASYNC);
    }
    if function.flags().generator() && function.flags().async_function() {
        flags = flags.union(FunctionKindFlags::ASYNC_GENERATOR);
    }
    flags
}

fn callable_prototype_for_function(
    agent: &Agent,
    realm: RealmRef,
    kind_flags: FunctionKindFlags,
) -> VmResult<Option<ObjectRef>> {
    let intrinsics = agent
        .realm(realm)
        .map(|realm| realm.intrinsics())
        .ok_or(VmError::MissingRootShape(realm))?;
    Ok(if kind_flags.is_async_generator() {
        intrinsics.async_generator_function_prototype()
    } else if kind_flags.is_generator() {
        intrinsics.generator_function_prototype()
    } else if kind_flags.is_async() {
        intrinsics.async_function_prototype()
    } else {
        intrinsics.function_prototype()
    })
}

fn prototype_parent_for_function(
    agent: &Agent,
    realm: RealmRef,
    kind_flags: FunctionKindFlags,
) -> VmResult<ObjectRef> {
    let intrinsics = agent
        .realm(realm)
        .map(|realm| realm.intrinsics())
        .ok_or(VmError::MissingRootShape(realm))?;
    if kind_flags.is_async_generator() {
        intrinsics
            .async_generator_prototype()
            .ok_or(VmError::MissingRootShape(realm))
    } else if kind_flags.is_generator() {
        intrinsics
            .generator_prototype()
            .ok_or(VmError::MissingRootShape(realm))
    } else {
        intrinsics
            .object_prototype()
            .ok_or(VmError::MissingRootShape(realm))
    }
}

pub(super) fn length_value(length: u32) -> Value {
    i32::try_from(length).map_or_else(|_| Value::from_f64(f64::from(length)), Value::from_smi)
}

fn function_name_text_from_property_key(
    agent: &Agent,
    key: PropertyKey,
    prefix: Option<&str>,
) -> String {
    let base = match key {
        PropertyKey::Index(index) => index.to_string(),
        PropertyKey::Atom(atom) => agent.atoms().resolve(atom).to_owned(),
        PropertyKey::Symbol(symbol) => agent
            .heap()
            .view()
            .symbol_view(symbol)
            .and_then(|view| view.description_view().and_then(decode_function_name_text))
            .map_or_else(String::new, |description| format!("[{description}]")),
    };
    match prefix {
        Some(prefix) => format!("{prefix} {base}"),
        None => base,
    }
}

fn decode_function_name_text(view: &lyng_js_gc::PrimitiveStringView<'_>) -> Option<String> {
    if let Some(bytes) = view.latin1_bytes() {
        return Some(bytes.iter().map(|byte| char::from(*byte)).collect());
    }
    let bytes = view.utf16_bytes()?;
    let mut units = Vec::with_capacity(view.code_unit_len() as usize);
    for chunk in bytes.chunks_exact(2) {
        units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    String::from_utf16(&units).ok()
}

fn string_ref_is_empty(agent: &Agent, string: lyng_js_types::StringRef) -> bool {
    agent
        .heap()
        .view()
        .string_view(string)
        .is_some_and(|view| view.code_unit_len() == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::RegisterWindow;
    use lyng_js_bytecode::{
        BytecodeBuilder, BytecodeFunctionFlags, BytecodeFunctionId, BytecodeFunctionKind,
        CompiledFunctionUnit,
    };
    use lyng_js_common::SourceId;
    use lyng_js_env::{
        EnvironmentLayout, EnvironmentLayoutKind, ExecutionContext, ExecutionContextKind, Runtime,
    };
    use lyng_js_host::NoopHostHooks;
    use lyng_js_ops::object;
    use lyng_js_types::PropertyKey;

    #[test]
    fn create_closure_captures_current_private_environment() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist after boot");
        let global_env = realm.global_env();
        let private_layout = agent.alloc_environment_layout(EnvironmentLayout::empty(
            EnvironmentLayoutKind::Private,
            true,
        ));
        let private_env = agent
            .alloc_private_environment(
                Some(global_env),
                private_layout,
                AllocationLifetime::Default,
            )
            .expect("private environment should allocate");

        let parent_id = BytecodeFunctionId::from_raw(1).expect("bytecode id should allocate");
        let child_id = BytecodeFunctionId::from_raw(2).expect("bytecode id should allocate");
        let mut parent = BytecodeBuilder::new(parent_id, BytecodeFunctionKind::Function);
        parent
            .add_child_function(child_id)
            .expect("test bytecode child should build");
        let parent = parent.finish().expect("test bytecode should build");
        let child = BytecodeBuilder::new(child_id, BytecodeFunctionKind::Arrow)
            .finish()
            .expect("test bytecode should build");
        let unit = CompiledFunctionUnit::new(SourceId::new(90), parent.id(), vec![parent, child]);

        let mut vm = Vm::new();
        let _ = vm
            .bootstrap_realm(agent, realm.id(), lyng_js_builtins::BootstrapMode::SpecOnly)
            .expect("bootstrap should succeed");
        let installed = vm
            .install_function(agent, realm.id(), &unit)
            .expect("function unit should install");
        let frame = FrameRecord::new(
            installed.code(),
            0,
            RegisterWindow::new(0, 0),
            None,
            realm.id(),
            global_env,
            global_env,
            ExecutionContextKind::Function,
        );
        agent.push_execution_context(
            ExecutionContext::bytecode(realm.id(), installed.code(), global_env, global_env)
                .with_private_env(Some(private_env)),
        );

        let closure = vm
            .create_closure(agent, &frame, 0)
            .expect("closure creation should succeed");
        let function_data = agent
            .objects()
            .function_data(closure)
            .expect("created closure should carry function payload metadata");
        assert_eq!(function_data.private_env(), Some(private_env));
    }

    #[test]
    fn create_closure_keeps_class_constructor_prototype_non_writable() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist after boot");
        let global_env = realm.global_env();

        let parent_id = BytecodeFunctionId::from_raw(1).expect("bytecode id should allocate");
        let child_id = BytecodeFunctionId::from_raw(2).expect("bytecode id should allocate");
        let mut parent = BytecodeBuilder::new(parent_id, BytecodeFunctionKind::Function);
        parent
            .add_child_function(child_id)
            .expect("test bytecode child should build");
        let parent = parent.finish().expect("test bytecode should build");
        let mut child = BytecodeBuilder::new(child_id, BytecodeFunctionKind::Function);
        child.set_flags(
            BytecodeFunctionFlags::new(true, false)
                .with_class_constructor(true)
                .with_constructible(true),
        );
        let child = child.finish().expect("test bytecode should build");
        let unit = CompiledFunctionUnit::new(SourceId::new(91), parent.id(), vec![parent, child]);

        let mut vm = Vm::new();
        let _ = vm
            .bootstrap_realm(agent, realm.id(), lyng_js_builtins::BootstrapMode::SpecOnly)
            .expect("bootstrap should succeed");
        let installed = vm
            .install_function(agent, realm.id(), &unit)
            .expect("function unit should install");
        let frame = FrameRecord::new(
            installed.code(),
            0,
            RegisterWindow::new(0, 0),
            None,
            realm.id(),
            global_env,
            global_env,
            ExecutionContextKind::Function,
        );

        let closure = vm
            .create_closure(agent, &frame, 0)
            .expect("closure creation should succeed");
        let descriptor = object::ordinary_get_own_property(
            agent,
            closure,
            PropertyKey::from_atom(WellKnownAtom::prototype.id()),
        )
        .expect("prototype descriptor lookup should succeed")
        .expect("class constructor should expose a prototype property");
        assert_eq!(descriptor.configurable(), Some(false));
        assert_eq!(descriptor.enumerable(), Some(false));
        assert_eq!(descriptor.writable(), Some(false));
    }

    #[test]
    fn setting_function_name_keeps_existing_class_constructor_prototype_attrs() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist after boot");
        let global_env = realm.global_env();

        let parent_id = BytecodeFunctionId::from_raw(3).expect("bytecode id should allocate");
        let child_id = BytecodeFunctionId::from_raw(4).expect("bytecode id should allocate");
        let mut parent = BytecodeBuilder::new(parent_id, BytecodeFunctionKind::Function);
        parent
            .add_child_function(child_id)
            .expect("test bytecode child should build");
        let parent = parent.finish().expect("test bytecode should build");
        let mut child = BytecodeBuilder::new(child_id, BytecodeFunctionKind::Function);
        child.set_flags(
            BytecodeFunctionFlags::new(true, false)
                .with_class_constructor(true)
                .with_constructible(true),
        );
        let child = child.finish().expect("test bytecode should build");
        let unit = CompiledFunctionUnit::new(SourceId::new(92), parent.id(), vec![parent, child]);

        let mut vm = Vm::new();
        let _ = vm
            .bootstrap_realm(agent, realm.id(), lyng_js_builtins::BootstrapMode::SpecOnly)
            .expect("bootstrap should succeed");
        let installed = vm
            .install_function(agent, realm.id(), &unit)
            .expect("function unit should install");
        let frame = FrameRecord::new(
            installed.code(),
            0,
            RegisterWindow::new(0, 0),
            None,
            realm.id(),
            global_env,
            global_env,
            ExecutionContextKind::Function,
        );

        let closure = vm
            .create_closure(agent, &frame, 0)
            .expect("closure creation should succeed");
        let name_atom = agent.atoms_mut().intern_collectible("C");
        let function_name = Value::from_string_ref(agent.alloc_runtime_string(
            "C",
            Some(name_atom),
            AllocationLifetime::Default,
        ));
        Vm::set_function_name(agent, closure, function_name)
            .expect("setting function name should succeed");

        let prototype_descriptor = object::ordinary_get_own_property(
            agent,
            closure,
            PropertyKey::from_atom(WellKnownAtom::prototype.id()),
        )
        .expect("prototype descriptor lookup should succeed")
        .expect("class constructor should expose a prototype property");
        assert_eq!(prototype_descriptor.configurable(), Some(false));
        assert_eq!(prototype_descriptor.enumerable(), Some(false));
        assert_eq!(prototype_descriptor.writable(), Some(false));

        let name_descriptor = object::ordinary_get_own_property(
            agent,
            closure,
            PropertyKey::from_atom(WellKnownAtom::name.id()),
        )
        .expect("name descriptor lookup should succeed")
        .expect("function should expose a name property");
        assert_eq!(name_descriptor.configurable(), Some(true));
        assert_eq!(name_descriptor.enumerable(), Some(false));
        assert_eq!(name_descriptor.writable(), Some(false));
    }

    #[test]
    fn binding_home_object_and_private_env_keeps_class_constructor_prototype_attrs() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist after boot");
        let global_env = realm.global_env();
        let private_layout = agent.alloc_environment_layout(EnvironmentLayout::empty(
            EnvironmentLayoutKind::Private,
            true,
        ));
        let private_env = agent
            .alloc_private_environment(
                Some(global_env),
                private_layout,
                AllocationLifetime::Default,
            )
            .expect("private environment should allocate");

        let parent_id = BytecodeFunctionId::from_raw(5).expect("bytecode id should allocate");
        let child_id = BytecodeFunctionId::from_raw(6).expect("bytecode id should allocate");
        let mut parent = BytecodeBuilder::new(parent_id, BytecodeFunctionKind::Function);
        parent
            .add_child_function(child_id)
            .expect("test bytecode child should build");
        let parent = parent.finish().expect("test bytecode should build");
        let mut child = BytecodeBuilder::new(child_id, BytecodeFunctionKind::Function);
        child.set_flags(
            BytecodeFunctionFlags::new(true, false)
                .with_class_constructor(true)
                .with_constructible(true),
        );
        let child = child.finish().expect("test bytecode should build");
        let unit = CompiledFunctionUnit::new(SourceId::new(93), parent.id(), vec![parent, child]);

        let mut vm = Vm::new();
        let _ = vm
            .bootstrap_realm(agent, realm.id(), lyng_js_builtins::BootstrapMode::SpecOnly)
            .expect("bootstrap should succeed");
        let installed = vm
            .install_function(agent, realm.id(), &unit)
            .expect("function unit should install");
        let frame = FrameRecord::new(
            installed.code(),
            0,
            RegisterWindow::new(0, 0),
            None,
            realm.id(),
            global_env,
            global_env,
            ExecutionContextKind::Function,
        );

        let closure = vm
            .create_closure(agent, &frame, 0)
            .expect("closure creation should succeed");
        let prototype = object::ordinary_get_own_property(
            agent,
            closure,
            PropertyKey::from_atom(WellKnownAtom::prototype.id()),
        )
        .expect("prototype descriptor lookup should succeed")
        .expect("class constructor should expose a prototype property")
        .value()
        .and_then(Value::as_object_ref)
        .expect("prototype descriptor should expose the created prototype object");
        let updated_home_object = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.set_function_home_object(&mut mutator, closure, Some(prototype))
        });
        assert!(updated_home_object);
        let updated_private_env = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.set_function_private_env(&mut mutator, closure, Some(private_env))
        });
        assert!(updated_private_env);

        let prototype_descriptor = object::ordinary_get_own_property(
            agent,
            closure,
            PropertyKey::from_atom(WellKnownAtom::prototype.id()),
        )
        .expect("prototype descriptor lookup should succeed")
        .expect("class constructor should expose a prototype property");
        assert_eq!(prototype_descriptor.configurable(), Some(false));
        assert_eq!(prototype_descriptor.enumerable(), Some(false));
        assert_eq!(prototype_descriptor.writable(), Some(false));
    }
}

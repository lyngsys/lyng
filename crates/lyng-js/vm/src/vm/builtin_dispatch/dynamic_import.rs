use super::{
    alloc_string, errors, object, promise_capability_executor_builtin, Agent, AllocationLifetime,
    FrameRecord, HostHooks, ImportMetaValue, ModuleImportAttribute, ModuleKey, ModuleSourceRequest,
    NativeFunctionRegistry, ObjectAllocation, ObjectRef, PromiseResolvingFunctionKind, PropertyKey,
    RealmRef, ToPrimitiveHint, Value, Vm, VmError, VmProxyBridge, VmResult,
};
use crate::vm::DynamicImportPhase;
use crate::vm::{DynamicImportRequest, PendingDynamicImport};
use lyng_js_bytecode::Opcode;
use lyng_js_env::{ModuleStatus, RealmRecord};

enum DynamicImportEvaluationOutcome {
    Fulfilled {
        key: ModuleKey,
        value: Value,
    },
    Rejected {
        key: Option<ModuleKey>,
        reason: Value,
    },
    Pending,
}

impl Vm {
    pub(super) fn import_meta_builtin(
        agent: &mut Agent,
        caller_frame: &FrameRecord,
    ) -> VmResult<Value> {
        let module_key = agent
            .module_key_for_environment(caller_frame.lexical_env())
            .ok_or(VmError::MissingModuleRecord)?;
        let (cached_object, host_properties) = {
            let record = agent
                .module_record(&module_key)
                .ok_or(VmError::MissingModuleRecord)?;
            (
                record.import_meta_object(),
                record.import_meta_properties().cloned(),
            )
        };
        if let Some(import_meta) = cached_object {
            return Ok(Value::from_object_ref(import_meta));
        }

        let realm = agent
            .realm(caller_frame.realm())
            .ok_or_else(|| VmError::MissingRootShape(caller_frame.realm()))?;
        let root_shape = realm
            .root_shape()
            .ok_or_else(|| VmError::MissingRootShape(caller_frame.realm()))?;
        let import_meta = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                AllocationLifetime::Default,
            )
        });
        if let Some(host_properties) = host_properties {
            for property in host_properties.properties {
                let key = agent.atoms_mut().intern_collectible(&property.key);
                let value = match property.value {
                    ImportMetaValue::String(text) => {
                        Value::from_string_ref(alloc_string(agent, &text, None))
                    }
                    ImportMetaValue::Boolean(value) => Value::from_bool(value),
                    ImportMetaValue::Smi(value) => Value::from_smi(value),
                    ImportMetaValue::Null => Value::null(),
                };
                object::ordinary_create_data_property(
                    agent,
                    import_meta,
                    PropertyKey::from_atom(key),
                    value,
                    AllocationLifetime::Default,
                )
                .map_err(VmError::Abrupt)?;
            }
        } else {
            let url = Value::from_string_ref(alloc_string(agent, module_key.as_str(), None));
            let url_atom = agent.atoms_mut().intern_collectible("url");
            object::ordinary_create_data_property(
                agent,
                import_meta,
                PropertyKey::from_atom(url_atom),
                url,
                AllocationLifetime::Default,
            )
            .map_err(VmError::Abrupt)?;
        }
        let _ = agent.set_module_record_import_meta_object(&module_key, Some(import_meta));
        Ok(Value::from_object_ref(import_meta))
    }

    pub(super) fn dynamic_import_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: &FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let realm = caller_frame.realm();
        let constructor = agent
            .realm(realm)
            .and_then(|realm| realm.intrinsics().promise())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let capability = self.create_dynamic_import_capability(
            agent,
            host,
            registry,
            caller_frame,
            constructor,
        )?;
        let promise = agent
            .promise_capability(capability)
            .and_then(lyng_js_env::PromiseCapabilityRecord::promise)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let specifier = arguments.first().copied().unwrap_or(Value::undefined());
        let options = arguments.get(1).copied().unwrap_or(Value::undefined());
        let phase = DynamicImportPhase::from_value(arguments.get(2).copied());

        let outcome = (|| -> Result<ModuleSourceRequest, Value> {
            let specifier = self
                .to_primitive(
                    agent,
                    host,
                    registry,
                    *caller_frame,
                    specifier,
                    ToPrimitiveHint::String,
                )
                .map_err(|error| Self::dynamic_import_error_value(agent, error))?;
            let specifier = Self::value_to_string_text(agent, specifier)
                .map_err(|error| Self::dynamic_import_error_value(agent, error))?;
            let attributes = self
                .normalize_dynamic_import_attributes(agent, host, registry, caller_frame, options)
                .map_err(|error| Self::dynamic_import_error_value(agent, error))?;
            if phase == DynamicImportPhase::Source {
                return Err(errors::syntax_error_value(agent));
            }
            let request = ModuleSourceRequest {
                specifier,
                referrer: Self::active_script_or_module_referrer(agent),
                attributes,
            };
            Ok(request)
        })();

        match outcome {
            Ok(request) => {
                self.enqueue_dynamic_import_evaluate_job(agent, realm, capability, request, phase);
            }
            Err(reason) => {
                Self::enqueue_dynamic_import_settle_job(agent, realm, capability, reason, true);
            }
        }
        Ok(Value::from_object_ref(promise))
    }

    fn create_dynamic_import_capability(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: &FrameRecord,
        constructor: ObjectRef,
    ) -> VmResult<lyng_js_env::PromiseCapabilityId> {
        let capability = agent.alloc_promise_capability();
        let executor = Self::allocate_builtin_function_object(
            agent,
            caller_frame.realm(),
            promise_capability_executor_builtin(),
        )?;
        let _ = agent.alloc_promise_resolving_function(
            executor,
            lyng_js_env::PromiseResolvingFunctionRecord::new(
                PromiseResolvingFunctionKind::CapabilityExecutor,
                capability,
            ),
        );
        let promise = self.construct_to_completion(
            agent,
            host,
            registry,
            *caller_frame,
            constructor,
            &[Value::from_object_ref(executor)],
            Some(constructor),
        )?;
        let _ = agent.set_promise_capability_promise(capability, promise);
        if agent
            .promise_capability(capability)
            .is_none_or(|record| record.resolve().is_none() || record.reject().is_none())
        {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(capability)
    }

    fn normalize_dynamic_import_attributes(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: &FrameRecord,
        options: Value,
    ) -> VmResult<Vec<ModuleImportAttribute>> {
        if options.is_undefined() {
            return Ok(Vec::new());
        }
        let Some(options_object) = options.as_object_ref() else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        let with_key = PropertyKey::from_atom(agent.atoms_mut().intern_collectible("with"));
        let with_value = self.get_property_from_object(
            agent,
            host,
            registry,
            *caller_frame,
            options_object,
            Value::from_object_ref(options_object),
            with_key,
        )?;
        if with_value.is_undefined() {
            return Ok(Vec::new());
        }
        let Some(attributes_object) = with_value.as_object_ref() else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        let keys = object::own_property_keys_in_context(
            &mut VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame: caller_frame,
            },
            attributes_object,
        )?;
        let mut attributes = Vec::new();
        for key in keys {
            let enumerable = object::get_own_property_in_context(
                &mut VmProxyBridge {
                    vm: self,
                    agent,
                    host,
                    registry,
                    frame: caller_frame,
                },
                attributes_object,
                key,
            )?
            .is_some_and(|descriptor| descriptor.enumerable() == Some(true));
            if !enumerable {
                continue;
            }
            let Some(attribute_key) = Self::dynamic_import_attribute_key(agent, key) else {
                continue;
            };
            let attribute_value = self.get_property_from_object(
                agent,
                host,
                registry,
                *caller_frame,
                attributes_object,
                Value::from_object_ref(attributes_object),
                key,
            )?;
            if attribute_value.as_string_ref().is_none() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            let attribute_value = Self::value_to_string_text(agent, attribute_value)?;
            attributes.push(ModuleImportAttribute {
                key: attribute_key,
                value: attribute_value,
            });
        }
        Ok(attributes)
    }

    fn dynamic_import_attribute_key(agent: &Agent, key: PropertyKey) -> Option<String> {
        if let Some(index) = key.as_index() {
            return Some(index.to_string());
        }
        key.as_atom()
            .map(|atom| agent.atoms().resolve(atom).to_owned())
    }

    pub(crate) fn active_script_or_module_referrer(agent: &Agent) -> Option<ModuleKey> {
        agent
            .current_execution_context()
            .and_then(lyng_js_env::ExecutionContext::script_or_module_referrer)
            .map(|atom| ModuleKey::new(agent.atoms().resolve(atom).to_owned().into_boxed_str()))
    }

    fn enqueue_dynamic_import_settle_job(
        agent: &mut Agent,
        realm: RealmRef,
        capability: lyng_js_env::PromiseCapabilityId,
        value: Value,
        rejected: bool,
    ) {
        let script_or_module_referrer = agent
            .current_execution_context()
            .and_then(lyng_js_env::ExecutionContext::script_or_module_referrer);
        let _ = agent.enqueue_job_with_payload(
            lyng_js_host::HostJobKind::Promise,
            lyng_js_env::ExecutableId::Builtin,
            lyng_js_env::RuntimeJobPayload::DynamicImportSettle {
                capability,
                value,
                rejected,
                script_or_module_referrer,
            },
            Some(realm),
            Some("DynamicImportSettle".into()),
        );
    }

    fn enqueue_dynamic_import_evaluate_job(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        capability: lyng_js_env::PromiseCapabilityId,
        request: ModuleSourceRequest,
        phase: DynamicImportPhase,
    ) {
        let script_or_module_referrer = request
            .referrer
            .as_ref()
            .map(|key| agent.atoms_mut().intern_collectible(key.as_str()));
        let request_id = self.alloc_dynamic_import_request(DynamicImportRequest {
            capability,
            request,
            phase,
        });
        let _ = agent.enqueue_job_with_payload(
            lyng_js_host::HostJobKind::Promise,
            lyng_js_env::ExecutableId::Builtin,
            lyng_js_env::RuntimeJobPayload::DynamicImportEvaluate {
                request: request_id,
                script_or_module_referrer,
            },
            Some(realm),
            Some("DynamicImportEvaluate".into()),
        );
    }

    fn alloc_dynamic_import_request(&mut self, request: DynamicImportRequest) -> u32 {
        let index = self.dynamic_import_requests.len();
        let id = u32::try_from(index).expect("dynamic import request id should fit into u32");
        self.dynamic_import_requests.push(Some(request));
        id
    }

    pub(in crate::vm) fn execute_dynamic_import_evaluate_job(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        realm_record: &RealmRecord,
        request_id: u32,
    ) -> VmResult<()> {
        let Some(request) = self
            .dynamic_import_requests
            .get_mut(usize::try_from(request_id).expect("request id should fit usize"))
            .and_then(Option::take)
        else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        self.dynamic_import_evaluate_depth = self.dynamic_import_evaluate_depth.saturating_add(1);
        let outcome =
            self.evaluate_dynamic_import_request(agent, realm_record, host, registry, &request);
        self.dynamic_import_evaluate_depth = self.dynamic_import_evaluate_depth.saturating_sub(1);
        match outcome {
            DynamicImportEvaluationOutcome::Fulfilled { key, value } => {
                self.settle_promise_capability(
                    agent,
                    host,
                    registry,
                    realm_record,
                    request.capability,
                    false,
                    value,
                )?;
                self.settle_waiting_dynamic_imports_for_module(agent, host, registry, &key)?;
                self.settle_ready_dynamic_imports(agent, host, registry)
            }
            DynamicImportEvaluationOutcome::Rejected { key, reason } => {
                self.settle_promise_capability(
                    agent,
                    host,
                    registry,
                    realm_record,
                    request.capability,
                    true,
                    reason,
                )?;
                if let Some(key) = key {
                    self.settle_waiting_dynamic_imports_for_module(agent, host, registry, &key)?;
                }
                self.settle_ready_dynamic_imports(agent, host, registry)
            }
            DynamicImportEvaluationOutcome::Pending => Ok(()),
        }
    }

    fn evaluate_dynamic_import_request(
        &mut self,
        agent: &mut Agent,
        realm_record: &RealmRecord,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        request: &DynamicImportRequest,
    ) -> DynamicImportEvaluationOutcome {
        let loaded =
            match self.load_module_graph_from_host(agent, realm_record, host, &request.request) {
                Ok(loaded) => loaded,
                Err(error) => {
                    return DynamicImportEvaluationOutcome::Rejected {
                        key: None,
                        reason: Self::dynamic_import_module_error_value(agent, error),
                    };
                }
            };
        let key = loaded.key().clone();
        let module_env = match self.link_module_graph(agent, realm_record, &key) {
            Ok(module_env) => module_env,
            Err(error) => {
                return DynamicImportEvaluationOutcome::Rejected {
                    key: Some(key.clone()),
                    reason: Self::dynamic_import_error_value(agent, error),
                };
            }
        };
        if request.phase == DynamicImportPhase::Defer {
            return self.evaluate_deferred_dynamic_import_request(
                agent,
                realm_record,
                host,
                registry,
                key,
            );
        }
        if agent
            .module_record(&key)
            .is_some_and(|record| record.status() == ModuleStatus::Evaluating)
        {
            self.dynamic_import_waiting_modules
                .entry(key)
                .or_default()
                .push(PendingDynamicImport {
                    capability: request.capability,
                    realm: realm_record.id(),
                });
            return DynamicImportEvaluationOutcome::Pending;
        }
        if let Err(error) = self.evaluate_module_graph(
            agent,
            realm_record,
            &key,
            module_env,
            host,
            registry,
            Some(&key),
            true,
        ) {
            if self.dynamic_import_evaluate_depth > 1
                && self.async_dependency_completed_modules.remove(&key)
            {
                self.dynamic_import_waiting_modules
                    .entry(key)
                    .or_default()
                    .push(PendingDynamicImport {
                        capability: request.capability,
                        realm: realm_record.id(),
                    });
                return DynamicImportEvaluationOutcome::Pending;
            }
            self.async_dependency_completed_modules.remove(&key);
            return DynamicImportEvaluationOutcome::Rejected {
                key: Some(key.clone()),
                reason: Self::dynamic_import_error_value(agent, error),
            };
        }
        let namespace = match self.module_namespace_object(agent, realm_record, &key) {
            Ok(namespace) => namespace,
            Err(error) => {
                return DynamicImportEvaluationOutcome::Rejected {
                    key: Some(key.clone()),
                    reason: Self::dynamic_import_error_value(agent, error),
                };
            }
        };
        if self.dynamic_import_evaluate_depth > 1
            && self.async_dependency_completed_modules.remove(&key)
        {
            self.dynamic_import_waiting_modules
                .entry(key)
                .or_default()
                .push(PendingDynamicImport {
                    capability: request.capability,
                    realm: realm_record.id(),
                });
            return DynamicImportEvaluationOutcome::Pending;
        }
        self.async_dependency_completed_modules.remove(&key);
        DynamicImportEvaluationOutcome::Fulfilled {
            key,
            value: Value::from_object_ref(namespace),
        }
    }

    fn evaluate_deferred_dynamic_import_request(
        &mut self,
        agent: &mut Agent,
        realm_record: &RealmRecord,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        key: ModuleKey,
    ) -> DynamicImportEvaluationOutcome {
        let dependencies = match self.gather_asynchronous_transitive_dependencies(agent, &key) {
            Ok(dependencies) => dependencies,
            Err(error) => {
                return DynamicImportEvaluationOutcome::Rejected {
                    key: Some(key),
                    reason: Self::dynamic_import_error_value(agent, error),
                };
            }
        };
        for dependency_key in dependencies {
            let dependency_env = match self.link_module_graph(agent, realm_record, &dependency_key)
            {
                Ok(module_env) => module_env,
                Err(error) => {
                    return DynamicImportEvaluationOutcome::Rejected {
                        key: Some(key),
                        reason: Self::dynamic_import_error_value(agent, error),
                    };
                }
            };
            if let Err(error) = self.evaluate_module_graph(
                agent,
                realm_record,
                &dependency_key,
                dependency_env,
                host,
                registry,
                None,
                true,
            ) {
                return DynamicImportEvaluationOutcome::Rejected {
                    key: Some(key),
                    reason: Self::dynamic_import_error_value(agent, error),
                };
            }
        }
        let namespace =
            match self.module_namespace_object_with_phase(agent, realm_record, &key, true) {
                Ok(namespace) => namespace,
                Err(error) => {
                    return DynamicImportEvaluationOutcome::Rejected {
                        key: Some(key),
                        reason: Self::dynamic_import_error_value(agent, error),
                    };
                }
            };
        DynamicImportEvaluationOutcome::Fulfilled {
            key,
            value: Value::from_object_ref(namespace),
        }
    }

    pub(in crate::vm) fn gather_asynchronous_transitive_dependencies(
        &self,
        agent: &Agent,
        key: &ModuleKey,
    ) -> VmResult<Vec<ModuleKey>> {
        let mut seen = Vec::new();
        let mut result = Vec::new();
        self.gather_asynchronous_transitive_dependencies_inner(agent, key, &mut seen, &mut result)?;
        Ok(result)
    }

    fn gather_asynchronous_transitive_dependencies_inner(
        &self,
        agent: &Agent,
        key: &ModuleKey,
        seen: &mut Vec<ModuleKey>,
        result: &mut Vec<ModuleKey>,
    ) -> VmResult<()> {
        if seen.iter().any(|candidate| candidate == key) {
            return Ok(());
        }
        seen.push(key.clone());
        let (status, requested_modules, has_top_level_await) = {
            let record = agent
                .module_record(key)
                .ok_or(VmError::MissingModuleRecord)?;
            (
                record.status(),
                record.requested_modules().to_vec(),
                self.module_has_top_level_await(agent, key)?,
            )
        };
        if matches!(status, ModuleStatus::Evaluating | ModuleStatus::Evaluated) {
            return Ok(());
        }
        if has_top_level_await {
            if !result.iter().any(|candidate| candidate == key) {
                result.push(key.clone());
            }
            return Ok(());
        }
        for request in requested_modules {
            let Some(resolved_key) = request.resolved_key().cloned() else {
                return Err(VmError::MissingModuleResolution);
            };
            self.gather_asynchronous_transitive_dependencies_inner(
                agent,
                &resolved_key,
                seen,
                result,
            )?;
        }
        Ok(())
    }

    pub(in crate::vm) fn module_has_top_level_await(
        &self,
        agent: &Agent,
        key: &ModuleKey,
    ) -> VmResult<bool> {
        let code = agent
            .module_record(key)
            .ok_or(VmError::MissingModuleRecord)?
            .code()
            .ok_or(VmError::MissingModuleCode)?;
        let function = self
            .installed_function(code)
            .ok_or(VmError::MissingInstalledCode(code))?;
        Ok(function
            .instructions()
            .iter()
            .any(|instruction| instruction.opcode() == Opcode::Await))
    }

    pub(in crate::vm) fn settle_waiting_dynamic_imports_for_module(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        key: &ModuleKey,
    ) -> VmResult<()> {
        let Some(waiters) = self.dynamic_import_waiting_modules.remove(key) else {
            return Ok(());
        };
        if waiters.is_empty() {
            return Ok(());
        }

        let (status, evaluation_error) = {
            let record = agent
                .module_record(key)
                .ok_or(VmError::MissingModuleRecord)?;
            (record.status(), record.evaluation_error())
        };
        match status {
            ModuleStatus::Evaluated => {
                for waiter in waiters {
                    let realm = agent
                        .realm(waiter.realm)
                        .ok_or(VmError::MissingRootShape(waiter.realm))?;
                    let namespace = self.module_namespace_object(agent, &realm, key)?;
                    self.settle_promise_capability(
                        agent,
                        host,
                        registry,
                        &realm,
                        waiter.capability,
                        false,
                        Value::from_object_ref(namespace),
                    )?;
                }
                Ok(())
            }
            ModuleStatus::Errored => {
                let reason = evaluation_error.unwrap_or(Value::undefined());
                for waiter in waiters {
                    let realm = agent
                        .realm(waiter.realm)
                        .ok_or(VmError::MissingRootShape(waiter.realm))?;
                    self.settle_promise_capability(
                        agent,
                        host,
                        registry,
                        &realm,
                        waiter.capability,
                        true,
                        reason,
                    )?;
                }
                Ok(())
            }
            ModuleStatus::Evaluating
            | ModuleStatus::New
            | ModuleStatus::Unlinked
            | ModuleStatus::Linking
            | ModuleStatus::Linked => {
                self.dynamic_import_waiting_modules
                    .entry(key.clone())
                    .or_default()
                    .extend(waiters);
                Ok(())
            }
        }
    }

    fn settle_ready_dynamic_imports(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<()> {
        let ready = self
            .dynamic_import_waiting_modules
            .keys()
            .filter(|key| {
                agent.module_record(key).is_some_and(|record| {
                    matches!(
                        record.status(),
                        ModuleStatus::Evaluated | ModuleStatus::Errored
                    )
                })
            })
            .cloned()
            .collect::<Vec<_>>();
        for key in ready {
            self.settle_waiting_dynamic_imports_for_module(agent, host, registry, &key)?;
        }
        Ok(())
    }

    fn dynamic_import_module_error_value(
        agent: &mut Agent,
        error: crate::error::ModuleLoadError,
    ) -> Value {
        match error {
            crate::error::ModuleLoadError::Vm(error) => {
                Self::dynamic_import_error_value(agent, error)
            }
            crate::error::ModuleLoadError::Host(error) => {
                Self::dynamic_import_host_error_value(agent, &error)
            }
            crate::error::ModuleLoadError::Parse | crate::error::ModuleLoadError::Sema => {
                errors::syntax_error_value(agent)
            }
            crate::error::ModuleLoadError::Lowering => {
                Value::from_string_ref(alloc_string(agent, "dynamic import lowering failure", None))
            }
        }
    }

    fn dynamic_import_error_value(agent: &mut Agent, error: VmError) -> Value {
        match error {
            VmError::Abrupt(completion) => completion.thrown_value().unwrap_or(Value::undefined()),
            VmError::Host(error) => Self::dynamic_import_host_error_value(agent, &error),
            VmError::AmbiguousModuleExport | VmError::MissingModuleResolution => {
                errors::syntax_error_value(agent)
            }
            other => Value::from_string_ref(alloc_string(agent, &format!("{other:?}"), None)),
        }
    }

    fn dynamic_import_host_error_value(
        agent: &mut Agent,
        error: &lyng_js_host::HostError,
    ) -> Value {
        Value::from_string_ref(alloc_string(agent, &error.to_string(), None))
    }
}

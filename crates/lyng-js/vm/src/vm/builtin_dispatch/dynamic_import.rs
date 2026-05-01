use super::*;
use crate::vm::{DynamicImportRequest, PendingDynamicImport};
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
        &mut self,
        agent: &mut Agent,
        caller_frame: FrameRecord,
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
            .ok_or(VmError::MissingRootShape(caller_frame.realm()))?;
        let root_shape = realm
            .root_shape()
            .ok_or(VmError::MissingRootShape(caller_frame.realm()))?;
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
        caller_frame: FrameRecord,
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
            .and_then(|record| record.promise())
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
                    caller_frame,
                    specifier,
                    ToPrimitiveHint::String,
                )
                .map_err(|error| self.dynamic_import_error_value(agent, error))?;
            let specifier = self
                .value_to_string_text(agent, specifier)
                .map_err(|error| self.dynamic_import_error_value(agent, error))?;
            let attributes = self
                .normalize_dynamic_import_attributes(agent, host, registry, caller_frame, options)
                .map_err(|error| self.dynamic_import_error_value(agent, error))?;
            if phase == DynamicImportPhase::Source {
                return Err(errors::syntax_error_value(agent));
            }
            let request = ModuleSourceRequest {
                specifier,
                referrer: self.active_script_or_module_referrer(agent),
                attributes,
            };
            Ok(request)
        })();

        match outcome {
            Ok(request) => {
                self.enqueue_dynamic_import_evaluate_job(agent, realm, capability, request)
            }
            Err(reason) => {
                self.enqueue_dynamic_import_settle_job(agent, realm, capability, reason, true)
            }
        }
        Ok(Value::from_object_ref(promise))
    }

    fn create_dynamic_import_capability(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        constructor: ObjectRef,
    ) -> VmResult<lyng_js_env::PromiseCapabilityId> {
        let capability = agent.alloc_promise_capability();
        let executor = self.allocate_builtin_function_object(
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
            caller_frame,
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
        caller_frame: FrameRecord,
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
            caller_frame,
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
            let Some(attribute_key) = self.dynamic_import_attribute_key(agent, key) else {
                continue;
            };
            let attribute_value = self.get_property_from_object(
                agent,
                host,
                registry,
                caller_frame,
                attributes_object,
                Value::from_object_ref(attributes_object),
                key,
            )?;
            if attribute_value.as_string_ref().is_none() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            let attribute_value = self.value_to_string_text(agent, attribute_value)?;
            attributes.push(ModuleImportAttribute {
                key: attribute_key,
                value: attribute_value,
            });
        }
        Ok(attributes)
    }

    fn dynamic_import_attribute_key(&self, agent: &Agent, key: PropertyKey) -> Option<String> {
        if let Some(index) = key.as_index() {
            return Some(index.to_string());
        }
        key.as_atom()
            .map(|atom| agent.atoms().resolve(atom).to_owned())
    }

    pub(crate) fn active_script_or_module_referrer(&self, agent: &Agent) -> Option<ModuleKey> {
        agent
            .current_execution_context()
            .and_then(|context| context.script_or_module_referrer())
            .map(|atom| ModuleKey::new(agent.atoms().resolve(atom).to_owned().into_boxed_str()))
    }

    fn enqueue_dynamic_import_settle_job(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        capability: lyng_js_env::PromiseCapabilityId,
        value: Value,
        rejected: bool,
    ) {
        let script_or_module_referrer = agent
            .current_execution_context()
            .and_then(|context| context.script_or_module_referrer());
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
    ) {
        let script_or_module_referrer = request
            .referrer
            .as_ref()
            .map(|key| agent.atoms_mut().intern_collectible(key.as_str()));
        let request_id = self.alloc_dynamic_import_request(DynamicImportRequest {
            capability,
            request,
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
        realm_record: RealmRecord,
        request_id: u32,
    ) -> VmResult<()> {
        let Some(request) = self
            .dynamic_import_requests
            .get_mut(usize::try_from(request_id).expect("request id should fit usize"))
            .and_then(Option::take)
        else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        let outcome =
            self.evaluate_dynamic_import_request(agent, realm_record, host, registry, &request)?;
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
                self.settle_waiting_dynamic_imports_for_module(agent, host, registry, &key)
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
                Ok(())
            }
            DynamicImportEvaluationOutcome::Pending => Ok(()),
        }
    }

    fn evaluate_dynamic_import_request(
        &mut self,
        agent: &mut Agent,
        realm_record: RealmRecord,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        request: &DynamicImportRequest,
    ) -> VmResult<DynamicImportEvaluationOutcome> {
        let loaded =
            match self.load_module_graph_from_host(agent, realm_record, host, &request.request) {
                Ok(loaded) => loaded,
                Err(error) => {
                    return Ok(DynamicImportEvaluationOutcome::Rejected {
                        key: None,
                        reason: self.dynamic_import_module_error_value(agent, error),
                    })
                }
            };
        let key = loaded.key().clone();
        let module_env = match self.link_module_graph(agent, realm_record, &key) {
            Ok(module_env) => module_env,
            Err(error) => {
                return Ok(DynamicImportEvaluationOutcome::Rejected {
                    key: Some(key.clone()),
                    reason: self.dynamic_import_error_value(agent, error),
                })
            }
        };
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
            return Ok(DynamicImportEvaluationOutcome::Pending);
        }
        if let Err(error) = self.evaluate_module_graph(
            agent,
            realm_record,
            &key,
            module_env,
            host,
            registry,
            Some(&key),
        ) {
            return Ok(DynamicImportEvaluationOutcome::Rejected {
                key: Some(key.clone()),
                reason: self.dynamic_import_error_value(agent, error),
            });
        }
        let namespace = match self.module_namespace_object(agent, realm_record, &key) {
            Ok(namespace) => namespace,
            Err(error) => {
                return Ok(DynamicImportEvaluationOutcome::Rejected {
                    key: Some(key.clone()),
                    reason: self.dynamic_import_error_value(agent, error),
                })
            }
        };
        Ok(DynamicImportEvaluationOutcome::Fulfilled {
            key,
            value: Value::from_object_ref(namespace),
        })
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
            ModuleStatus::Evaluating => {
                self.dynamic_import_waiting_modules
                    .entry(key.clone())
                    .or_default()
                    .extend(waiters);
                Ok(())
            }
            ModuleStatus::Evaluated => {
                for waiter in waiters {
                    let realm = agent
                        .realm(waiter.realm)
                        .ok_or(VmError::MissingRootShape(waiter.realm))?;
                    let namespace = self.module_namespace_object(agent, realm, key)?;
                    self.settle_promise_capability(
                        agent,
                        host,
                        registry,
                        realm,
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
                        realm,
                        waiter.capability,
                        true,
                        reason,
                    )?;
                }
                Ok(())
            }
            ModuleStatus::New
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

    fn dynamic_import_module_error_value(
        &self,
        agent: &mut Agent,
        error: crate::error::ModuleLoadError,
    ) -> Value {
        match error {
            crate::error::ModuleLoadError::Vm(error) => {
                self.dynamic_import_error_value(agent, error)
            }
            crate::error::ModuleLoadError::Host(error) => {
                self.dynamic_import_host_error_value(agent, error)
            }
            crate::error::ModuleLoadError::Parse | crate::error::ModuleLoadError::Sema => {
                errors::syntax_error_value(agent)
            }
            crate::error::ModuleLoadError::Lowering => {
                Value::from_string_ref(alloc_string(agent, "dynamic import lowering failure", None))
            }
        }
    }

    fn dynamic_import_error_value(&self, agent: &mut Agent, error: VmError) -> Value {
        match error {
            VmError::Abrupt(completion) => completion.thrown_value().unwrap_or(Value::undefined()),
            VmError::Host(error) => self.dynamic_import_host_error_value(agent, error),
            VmError::AmbiguousModuleExport | VmError::MissingModuleResolution => {
                errors::syntax_error_value(agent)
            }
            other => Value::from_string_ref(alloc_string(agent, &format!("{other:?}"), None)),
        }
    }

    fn dynamic_import_host_error_value(
        &self,
        agent: &mut Agent,
        error: lyng_js_host::HostError,
    ) -> Value {
        Value::from_string_ref(alloc_string(agent, &error.to_string(), None))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DynamicImportPhase {
    Evaluation,
    Source,
    Defer,
}

impl DynamicImportPhase {
    fn from_value(value: Option<Value>) -> Self {
        match value.and_then(Value::as_smi) {
            Some(1) => Self::Source,
            Some(2) => Self::Defer,
            _ => Self::Evaluation,
        }
    }
}

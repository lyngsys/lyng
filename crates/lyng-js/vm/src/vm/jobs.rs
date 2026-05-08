use super::runtime_objects::VmIteratorBridge;
use super::{
    Agent, CodeRef, FrameFlags, FrameRecord, InstalledCode, NativeFunctionRegistry, RealmRecord,
    RegisterWindow, Vm, VmError, VmResult,
};
use lyng_js_env::{
    ExecutableId, JobQueueKind, PromiseCapabilityId, PromiseReactionHandler, PromiseReactionKind,
    PromiseReactionRecord, PromiseState, RuntimeJob, RuntimeJobPayload, WaiterToken,
};
use lyng_js_host::{HostError, HostHooks, HostJobPhase, JobObservation, WaitLocation};
use lyng_js_ops::{errors, iterator, object, promise as promise_ops};
use lyng_js_types::{
    promise_reject_function_builtin, promise_resolve_function_builtin, AbruptCompletion, ObjectRef,
    PropertyKey, Value,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PromiseReactionOutcome {
    Fulfill(Value),
    Reject(Value),
    Deferred,
}

impl Vm {
    /// # Errors
    ///
    /// Returns a VM error if a promise or harness job throws, host observation fails, or a queued
    /// runtime job cannot be executed.
    pub fn checkpoint_promise_jobs(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<()> {
        loop {
            self.drain_job_queue_with_registry(agent, host, registry, JobQueueKind::Promise)?;
            if !self.run_next_job_from_queue_with_registry(
                agent,
                host,
                registry,
                JobQueueKind::Harness,
            )? {
                return Ok(());
            }
        }
    }

    /// # Errors
    ///
    /// Returns a VM error if a queued job throws, host observation fails, or runtime job execution
    /// fails.
    pub fn drain_job_queue_with_registry(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        queue_kind: JobQueueKind,
    ) -> VmResult<()> {
        while self.run_next_job_from_queue_with_registry(agent, host, registry, queue_kind)? {}
        Ok(())
    }

    fn run_next_job_from_queue_with_registry(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        queue_kind: JobQueueKind,
    ) -> VmResult<bool> {
        let Some(job) = agent.dequeue_job(queue_kind) else {
            return Ok(false);
        };
        Self::observe_job_phase(agent, host, &job, HostJobPhase::Enqueued)?;
        Self::observe_job_phase(agent, host, &job, HostJobPhase::Started)?;
        let result = self.execute_runtime_job(agent, host, registry, &job);
        match result {
            Ok(()) => Self::observe_job_phase(agent, host, &job, HostJobPhase::Completed)?,
            Err(error) => {
                Self::observe_job_phase(agent, host, &job, HostJobPhase::Failed)?;
                return Err(error);
            }
        }
        Ok(true)
    }

    fn observe_job_phase(
        agent: &Agent,
        host: &dyn HostHooks,
        job: &RuntimeJob,
        phase: HostJobPhase,
    ) -> VmResult<()> {
        host.observe_job(&JobObservation {
            agent: agent.host_id(),
            job_id: job.host_job_id(),
            phase,
            kind: job.kind(),
        })
        .map_err(VmError::Host)
    }

    fn execute_runtime_job(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        job: &RuntimeJob,
    ) -> VmResult<()> {
        let realm = job
            .realm()
            .or_else(|| agent.default_realm_id())
            .ok_or(VmError::MissingDefaultRealm)?;
        let realm_record = agent.realm(realm).ok_or(VmError::MissingRootShape(realm))?;
        let lexical_env = realm_record.global_env();
        let variable_env = realm_record.global_env();
        let script_or_module_referrer = match job.payload() {
            RuntimeJobPayload::PromiseReaction { reaction, .. } => agent
                .promise_reaction(reaction)
                .and_then(lyng_js_env::PromiseReactionRecord::script_or_module_referrer),
            RuntimeJobPayload::DynamicImportEvaluate {
                script_or_module_referrer,
                ..
            }
            | RuntimeJobPayload::DynamicImportSettle {
                script_or_module_referrer,
                ..
            } => script_or_module_referrer,
            RuntimeJobPayload::Executable
            | RuntimeJobPayload::PromiseThenableResolve { .. }
            | RuntimeJobPayload::AtomicsWaitAsyncTimeout { .. }
            | RuntimeJobPayload::FinalizationCleanup { .. } => None,
        };
        agent.push_execution_context(
            lyng_js_env::ExecutionContext::job(realm, job.executable(), lexical_env, variable_env)
                .with_script_or_module_referrer(script_or_module_referrer),
        );
        let result = match job.payload() {
            RuntimeJobPayload::Executable => {
                self.execute_executable_job(agent, host, registry, job, &realm_record)
            }
            RuntimeJobPayload::PromiseReaction { reaction, argument } => self
                .execute_promise_reaction_job(
                    agent,
                    host,
                    registry,
                    &realm_record,
                    reaction,
                    argument,
                ),
            RuntimeJobPayload::PromiseThenableResolve {
                promise,
                thenable,
                then,
            } => self.execute_promise_thenable_job(
                agent,
                host,
                registry,
                &realm_record,
                promise,
                thenable,
                then,
            ),
            RuntimeJobPayload::DynamicImportEvaluate { request, .. } => self
                .execute_dynamic_import_evaluate_job(agent, host, registry, &realm_record, request),
            RuntimeJobPayload::DynamicImportSettle {
                capability,
                value,
                rejected,
                ..
            } => self.execute_dynamic_import_settle_job(
                agent,
                host,
                registry,
                &realm_record,
                capability,
                value,
                rejected,
            ),
            RuntimeJobPayload::AtomicsWaitAsyncTimeout {
                location,
                token,
                promise,
            } => Self::execute_atomics_wait_async_timeout_job(agent, location, token, promise),
            RuntimeJobPayload::FinalizationCleanup {
                registry: registry_object,
            } => self.execute_finalization_cleanup_job(
                agent,
                host,
                registry,
                &realm_record,
                registry_object,
            ),
        };
        let _ = agent.pop_execution_context();
        agent.clear_kept_objects();
        result
    }

    fn execute_atomics_wait_async_timeout_job(
        agent: &mut Agent,
        location: WaitLocation,
        token: WaiterToken,
        promise: ObjectRef,
    ) -> VmResult<()> {
        if promise_ops::promise_state(agent, promise) != Some(PromiseState::Pending) {
            return Ok(());
        }
        if !agent.remove_shared_memory_waiter(location, token) {
            return Ok(());
        }
        let value = Value::from_string_ref(agent.alloc_runtime_string(
            "timed-out",
            None,
            lyng_js_gc::AllocationLifetime::Default,
        ));
        promise_ops::fulfill_promise(agent, promise, value).map_err(VmError::Abrupt)
    }

    fn execute_executable_job(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        job: &RuntimeJob,
        realm: &RealmRecord,
    ) -> VmResult<()> {
        let ExecutableId::Bytecode(code) = job.executable() else {
            return Err(VmError::Host(HostError::unsupported(
                "drain_job_queue",
                format!("unsupported runtime job executable {:?}", job.executable()),
            )));
        };
        let installed = InstalledCode::new(
            code,
            self.installed_function(code)
                .ok_or(VmError::MissingInstalledCode(code))?
                .id(),
        );
        let _ = self.evaluate_entry_with_registry(
            agent,
            installed,
            realm.global_env(),
            realm.global_env(),
            None,
            host,
            registry,
            None,
            None,
        )?;
        Ok(())
    }

    fn execute_promise_reaction_job(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        realm: &RealmRecord,
        reaction_id: lyng_js_env::PromiseReactionId,
        argument: Value,
    ) -> VmResult<()> {
        let reaction = agent
            .promise_reaction(reaction_id)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let async_generator_return = match reaction.handler() {
            PromiseReactionHandler::AsyncGeneratorReturn { generator, .. } => Some(generator),
            _ => None,
        };
        let outcome =
            self.execute_promise_reaction(agent, host, registry, realm, reaction, argument)?;
        let result = match (reaction.capability(), outcome) {
            (_, PromiseReactionOutcome::Deferred) | (None, PromiseReactionOutcome::Fulfill(_)) => {
                Ok(())
            }
            (Some(capability), PromiseReactionOutcome::Fulfill(value)) => self
                .settle_promise_capability(agent, host, registry, realm, capability, false, value),
            (Some(capability), PromiseReactionOutcome::Reject(reason)) => self
                .settle_promise_capability(agent, host, registry, realm, capability, true, reason),
            (None, PromiseReactionOutcome::Reject(reason)) => {
                Err(VmError::Abrupt(AbruptCompletion::throw(reason)))
            }
        };
        if result.is_ok()
            && let Some(generator) = async_generator_return
        {
            self.pop_async_generator_front_request(generator);
            self.drain_async_generator_queue(agent, host, registry, generator)?;
        }
        result
    }

    #[expect(
        clippy::too_many_lines,
        reason = "spec-shaped VM algorithm stays contiguous until the VM module split issue extracts it"
    )]
    fn execute_promise_reaction(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        realm: &RealmRecord,
        reaction: lyng_js_env::PromiseReactionRecord,
        argument: Value,
    ) -> VmResult<PromiseReactionOutcome> {
        match reaction.handler() {
            PromiseReactionHandler::Identity => Ok(PromiseReactionOutcome::Fulfill(argument)),
            PromiseReactionHandler::Thrower => Ok(PromiseReactionOutcome::Reject(argument)),
            PromiseReactionHandler::PassThrough(value) => {
                Ok(PromiseReactionOutcome::Fulfill(value))
            }
            PromiseReactionHandler::ThrowWith(value) => Ok(PromiseReactionOutcome::Reject(value)),
            PromiseReactionHandler::AsyncFromSyncIteratorValue { done } => {
                let result =
                    iterator::create_iterator_result_object(agent, realm.id(), argument, done)
                        .map_err(VmError::Abrupt)?;
                Ok(PromiseReactionOutcome::Fulfill(Value::from_object_ref(
                    result,
                )))
            }
            PromiseReactionHandler::AsyncFromSyncIteratorReject {
                iterator,
                next_method,
            } => {
                let caller = self.synthetic_job_caller_frame(realm);
                let mut record =
                    iterator::IteratorRecord::new_async_from_sync(iterator, next_method);
                let mut bridge = VmIteratorBridge {
                    vm: self,
                    agent,
                    host,
                    registry,
                    frame: caller,
                };
                let completion = AbruptCompletion::throw(argument);
                match iterator::iterator_close::<_, ()>(&mut bridge, &mut record, Err(completion)) {
                    Ok(()) => Ok(PromiseReactionOutcome::Reject(argument)),
                    Err(VmError::Abrupt(completion)) => Ok(PromiseReactionOutcome::Reject(
                        completion.thrown_value().unwrap_or(argument),
                    )),
                    Err(error) => Err(error),
                }
            }
            PromiseReactionHandler::AsyncResume { suspended, reject } => {
                let suspended_state = self
                    .suspended_side_states
                    .get(&suspended)
                    .cloned()
                    .unwrap_or_default();
                if suspended_state.async_frame_state.is_some() {
                    self.resume_async_function(
                        agent, host, registry, realm, suspended, reject, argument,
                    )?;
                } else if suspended_state.async_generator_frame_state.is_some() {
                    self.resume_async_generator_request(
                        agent, host, registry, suspended, reject, argument,
                    )?;
                } else {
                    self.resume_suspended_execution_job(
                        agent, host, registry, suspended, reject, argument,
                    )?;
                }
                Ok(PromiseReactionOutcome::Fulfill(Value::undefined()))
            }
            PromiseReactionHandler::AsyncGeneratorReturnResume { suspended, reject } => {
                self.resume_async_generator_return_from_suspended_yield(
                    agent, host, registry, suspended, reject, argument,
                )?;
                Ok(PromiseReactionOutcome::Fulfill(Value::undefined()))
            }
            PromiseReactionHandler::AsyncGeneratorReturn { reject, .. } => {
                if reject {
                    return Ok(PromiseReactionOutcome::Reject(argument));
                }
                let result =
                    iterator::create_iterator_result_object(agent, realm.id(), argument, true)
                        .map_err(VmError::Abrupt)?;
                Ok(PromiseReactionOutcome::Fulfill(Value::from_object_ref(
                    result,
                )))
            }
            PromiseReactionHandler::Callable(handler) => {
                let caller = self.synthetic_job_caller_frame(realm);
                match self.call_to_completion(
                    agent,
                    host,
                    registry,
                    caller,
                    handler,
                    Value::undefined(),
                    &[argument],
                ) {
                    Ok(value) => Ok(PromiseReactionOutcome::Fulfill(value)),
                    Err(VmError::Abrupt(completion)) => Ok(PromiseReactionOutcome::Reject(
                        completion.thrown_value().unwrap_or(Value::undefined()),
                    )),
                    Err(error) => Err(error),
                }
            }
            PromiseReactionHandler::Finally {
                on_finally,
                constructor,
                reject,
            } => self.execute_promise_finally_reaction(
                agent,
                host,
                registry,
                realm,
                reaction.capability(),
                on_finally,
                constructor,
                reject,
                argument,
            ),
        }
    }

    fn execute_finalization_cleanup_job(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        realm: &RealmRecord,
        registry_object: ObjectRef,
    ) -> VmResult<()> {
        let cleanup_callback = agent
            .finalization_cleanup_callback(registry_object)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let holdings = agent.take_finalization_cleanup_holdings(registry_object);
        let caller = self.synthetic_job_caller_frame(realm);
        let mut result = Ok(());
        for holding in holdings {
            if let Err(error) = self.call_to_completion(
                agent,
                host,
                registry,
                caller,
                cleanup_callback,
                Value::undefined(),
                &[holding],
            ) {
                result = Err(error);
                break;
            }
        }
        let _ = agent.set_finalization_cleanup_active(registry_object, false);
        if agent.finalization_cleanup_pending(registry_object) {
            let _ = agent.enqueue_finalization_cleanup_job(registry_object);
        }
        result
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn execute_promise_finally_reaction(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        realm: &RealmRecord,
        capability: Option<PromiseCapabilityId>,
        on_finally: ObjectRef,
        constructor: ObjectRef,
        reject: bool,
        argument: Value,
    ) -> VmResult<PromiseReactionOutcome> {
        let caller = self.synthetic_job_caller_frame(realm);
        let result = match self.call_to_completion(
            agent,
            host,
            registry,
            caller,
            on_finally,
            Value::undefined(),
            &[],
        ) {
            Ok(value) => value,
            Err(VmError::Abrupt(completion)) => {
                return Ok(PromiseReactionOutcome::Reject(
                    completion.thrown_value().unwrap_or(Value::undefined()),
                ));
            }
            Err(error) => return Err(error),
        };
        let resolve_key = PropertyKey::from_atom(agent.atoms_mut().intern_collectible("resolve"));
        let resolve =
            object::ordinary_get(agent, constructor, resolve_key).map_err(VmError::Abrupt)?;
        let resolve = resolve
            .as_object_ref()
            .filter(|object| agent.objects().is_callable(*object))
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let caller = self.synthetic_job_caller_frame(realm);
        let promise = match self.call_to_completion(
            agent,
            host,
            registry,
            caller,
            resolve,
            Value::from_object_ref(constructor),
            &[result],
        ) {
            Ok(value) => value,
            Err(VmError::Abrupt(completion)) => {
                return Ok(PromiseReactionOutcome::Reject(
                    completion.thrown_value().unwrap_or(Value::undefined()),
                ));
            }
            Err(error) => return Err(error),
        };
        let promise = promise
            .as_object_ref()
            .filter(|object| agent.promise_record(*object).is_some())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let on_fulfilled = if reject {
            PromiseReactionHandler::ThrowWith(argument)
        } else {
            PromiseReactionHandler::PassThrough(argument)
        };
        Self::enqueue_promise_then(
            agent,
            realm,
            promise,
            on_fulfilled,
            PromiseReactionHandler::Thrower,
            capability,
        )?;
        Ok(PromiseReactionOutcome::Deferred)
    }

    pub(in crate::vm) fn enqueue_promise_then(
        agent: &mut Agent,
        realm: &RealmRecord,
        promise: ObjectRef,
        on_fulfilled: PromiseReactionHandler,
        on_rejected: PromiseReactionHandler,
        capability: Option<PromiseCapabilityId>,
    ) -> VmResult<()> {
        let script_or_module_referrer = agent
            .current_execution_context()
            .and_then(lyng_js_env::ExecutionContext::script_or_module_referrer);
        let fulfill_reaction = agent.alloc_promise_reaction(
            PromiseReactionRecord::new(PromiseReactionKind::Fulfill, on_fulfilled, capability)
                .with_script_or_module_referrer(script_or_module_referrer),
        );
        let reject_reaction = agent.alloc_promise_reaction(
            PromiseReactionRecord::new(PromiseReactionKind::Reject, on_rejected, capability)
                .with_script_or_module_referrer(script_or_module_referrer),
        );
        let record = agent
            .promise_record(promise)
            .cloned()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let _ = agent.set_promise_handled(promise, true);
        match record.state() {
            lyng_js_env::PromiseState::Pending => {
                let _ = agent.push_promise_reaction(
                    promise,
                    PromiseReactionKind::Fulfill,
                    fulfill_reaction,
                );
                let _ = agent.push_promise_reaction(
                    promise,
                    PromiseReactionKind::Reject,
                    reject_reaction,
                );
            }
            lyng_js_env::PromiseState::Fulfilled => {
                Self::enqueue_promise_reaction_job(agent, realm, fulfill_reaction, record.result());
            }
            lyng_js_env::PromiseState::Rejected => {
                Self::enqueue_promise_reaction_job(agent, realm, reject_reaction, record.result());
            }
        }
        Ok(())
    }

    pub(super) fn enqueue_promise_reaction_job(
        agent: &mut Agent,
        realm: &RealmRecord,
        reaction: lyng_js_env::PromiseReactionId,
        argument: Value,
    ) {
        let _ = agent.enqueue_job_with_payload(
            lyng_js_host::HostJobKind::Promise,
            ExecutableId::Builtin,
            RuntimeJobPayload::PromiseReaction { reaction, argument },
            Some(realm.id()),
            Some("PromiseReaction".into()),
        );
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(in crate::vm) fn settle_promise_capability(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        realm: &RealmRecord,
        capability: PromiseCapabilityId,
        rejected: bool,
        value: Value,
    ) -> VmResult<()> {
        let record = agent
            .promise_capability(capability)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let function = if rejected {
            record.reject()
        } else {
            record.resolve()
        }
        .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let caller = self.synthetic_job_caller_frame(realm);
        let _ = self.call_to_completion(
            agent,
            host,
            registry,
            caller,
            function,
            Value::undefined(),
            &[value],
        )?;
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn execute_promise_thenable_job(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        realm: &RealmRecord,
        promise: ObjectRef,
        thenable: ObjectRef,
        then: ObjectRef,
    ) -> VmResult<()> {
        let (resolve, reject) =
            Self::create_promise_job_resolving_functions(agent, realm, promise)?;
        let caller = self.synthetic_job_caller_frame(realm);
        match self.call_to_completion(
            agent,
            host,
            registry,
            caller,
            then,
            Value::from_object_ref(thenable),
            &[
                Value::from_object_ref(resolve),
                Value::from_object_ref(reject),
            ],
        ) {
            Ok(_) => Ok(()),
            Err(VmError::Abrupt(completion)) => {
                let caller = self.synthetic_job_caller_frame(realm);
                let _ = self.call_to_completion(
                    agent,
                    host,
                    registry,
                    caller,
                    reject,
                    Value::undefined(),
                    &[completion.thrown_value().unwrap_or(Value::undefined())],
                )?;
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    fn create_promise_job_resolving_functions(
        agent: &mut Agent,
        realm: &RealmRecord,
        promise: ObjectRef,
    ) -> VmResult<(ObjectRef, ObjectRef)> {
        let capability = agent.alloc_promise_capability();
        let _ = agent.set_promise_capability_promise(capability, promise);
        let resolve = Self::allocate_builtin_function_object(
            agent,
            realm.id(),
            promise_resolve_function_builtin(),
        )?;
        let reject = Self::allocate_builtin_function_object(
            agent,
            realm.id(),
            promise_reject_function_builtin(),
        )?;
        let _ = agent.alloc_promise_resolving_function(
            resolve,
            lyng_js_env::PromiseResolvingFunctionRecord::new(
                lyng_js_env::PromiseResolvingFunctionKind::Resolve,
                capability,
            ),
        );
        let _ = agent.alloc_promise_resolving_function(
            reject,
            lyng_js_env::PromiseResolvingFunctionRecord::new(
                lyng_js_env::PromiseResolvingFunctionKind::Reject,
                capability,
            ),
        );
        let _ = agent.set_promise_capability_resolve(capability, resolve);
        let _ = agent.set_promise_capability_reject(capability, reject);
        Ok((resolve, reject))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn execute_dynamic_import_settle_job(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        realm: &RealmRecord,
        capability: PromiseCapabilityId,
        value: Value,
        rejected: bool,
    ) -> VmResult<()> {
        self.settle_promise_capability(agent, host, registry, realm, capability, rejected, value)
    }

    pub(super) fn synthetic_job_caller_frame(&self, realm: &RealmRecord) -> FrameRecord {
        FrameRecord::new(
            self.job_caller_code(),
            0,
            RegisterWindow::new(0, 0),
            None,
            realm.id(),
            realm.global_env(),
            realm.global_env(),
            lyng_js_env::ExecutionContextKind::Job,
        )
        .with_flags(FrameFlags::entry())
    }

    fn job_caller_code(&self) -> CodeRef {
        self.frame()
            .map(FrameRecord::code)
            .or_else(|| {
                self.installed
                    .iter()
                    .position(Option::is_some)
                    .and_then(|index| {
                        CodeRef::from_raw(
                            u32::try_from(index + 1)
                                .expect("installed code index should fit into u32"),
                        )
                    })
            })
            .unwrap_or_else(|| CodeRef::from_raw(1).expect("synthetic job code should be non-zero"))
    }
}

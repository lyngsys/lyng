use super::bytecode_calls::PreparedBytecodeCall;
use super::runtime_objects::VmIteratorBridge;
use super::*;
use crate::frame::GeneratorResumeKind;
use lyng_js_common::WellKnownAtom;
use lyng_js_env::{ExecutableId, ExecutionContextKind, ThisState};
use lyng_js_gc::{AllocationLifetime, RuntimeSuspendedExecutionRecord};
use lyng_js_objects::{GeneratorState, ObjectAllocation, ObjectColdData, OrdinaryObjectData};
use lyng_js_ops::{errors, iterator, iterator::IteratorOpsContext, object};
use lyng_js_types::{PropertyKey, SuspendedExecutionRef, Value};

const THIS_STATE_LEXICAL_RAW: u8 = 0;
const THIS_STATE_UNINITIALIZED_RAW: u8 = 1;
const THIS_STATE_VALUE_RAW: u8 = 2;

enum GeneratorExecutionOutcome {
    Complete(Value),
    Yield {
        value: Value,
        raw_iterator_result: bool,
    },
    Throw(Value),
    AsyncSuspend,
}

enum DelegateYieldOutcome {
    Suspend {
        value: Value,
        record: iterator::IteratorRecord,
        raw_iterator_result: bool,
    },
    Complete {
        value: Value,
    },
}

impl Vm {
    pub(super) fn instantiate_generator_call(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        prepared: PreparedBytecodeCall,
        arguments: &[Value],
    ) -> VmResult<ObjectRef> {
        let is_async_generator = self
            .installed_function(prepared.code)
            .is_some_and(|function| function.flags().async_function());
        let prior_frame_depth = self.frames.len();
        let prior_context_depth = agent.execution_contexts().len();
        let prior_register_len = self.register_stack.len();
        self.install_prepared_bytecode_call(
            agent,
            prepared,
            arguments,
            u32::try_from(prior_register_len).expect("register stack length should fit u32"),
            None,
            None,
            false,
        )?;
        self.internal_completion_targets.push(prior_frame_depth);
        let result = self.run(agent, host, registry);
        if self.internal_completion_targets.last().copied() == Some(prior_frame_depth) {
            let _ = self.internal_completion_targets.pop();
        }
        self.cleanup_internal_completion(
            agent,
            prior_frame_depth,
            prior_context_depth,
            prior_register_len,
        )?;

        match result {
            Err(VmError::GeneratorStart { suspended }) => {
                self.create_generator_object(agent, prepared.realm, suspended, is_async_generator)
            }
            Ok(_) => Err(VmError::Abrupt(errors::throw_type_error(agent))),
            Err(error) => Err(error),
        }
    }

    pub(super) fn resume_generator(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        generator: ObjectRef,
        resume_kind: GeneratorResumeKind,
        value: Value,
    ) -> VmResult<Value> {
        let (state, suspended) = {
            let objects = agent.objects();
            if !objects.is_generator_object(generator) {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            (
                objects.generator_state(generator),
                objects.generator_suspended(agent.heap().view(), generator),
            )
        };
        let state = state.ok_or(VmError::Abrupt(errors::throw_type_error(agent)))?;

        match state {
            GeneratorState::Executing => Err(VmError::Abrupt(errors::throw_type_error(agent))),
            GeneratorState::Completed => {
                self.generator_completion_result(agent, caller_frame.realm(), resume_kind, value)
            }
            GeneratorState::SuspendedStart if resume_kind == GeneratorResumeKind::Throw => {
                self.complete_generator_object(agent, generator)?;
                Err(VmError::Abrupt(lyng_js_types::AbruptCompletion::throw(
                    value,
                )))
            }
            GeneratorState::SuspendedStart if resume_kind == GeneratorResumeKind::Return => {
                self.complete_generator_object(agent, generator)?;
                self.generator_result_object(agent, caller_frame.realm(), value, true)
            }
            GeneratorState::SuspendedStart | GeneratorState::SuspendedYield => {
                let suspended =
                    suspended.ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
                self.set_generator_state(agent, generator, GeneratorState::Executing, None)?;
                let effective_value = if state == GeneratorState::SuspendedStart
                    && resume_kind == GeneratorResumeKind::Next
                {
                    Value::undefined()
                } else {
                    value
                };
                if let Err(error) =
                    self.restore_suspended_execution(agent, suspended, resume_kind, effective_value)
                {
                    let _ = self.complete_generator_object(agent, generator);
                    return Err(error);
                }
                if state == GeneratorState::SuspendedStart
                    && resume_kind == GeneratorResumeKind::Next
                {
                    self.clear_active_resume();
                }
                match self.run_generator_frame(agent, host, registry, generator, None)? {
                    GeneratorExecutionOutcome::Complete(value) => {
                        self.generator_result_object(agent, caller_frame.realm(), value, true)
                    }
                    GeneratorExecutionOutcome::Yield {
                        value,
                        raw_iterator_result,
                    } => {
                        if raw_iterator_result {
                            Ok(value)
                        } else {
                            self.generator_result_object(agent, caller_frame.realm(), value, false)
                        }
                    }
                    GeneratorExecutionOutcome::Throw(thrown) => Err(VmError::Abrupt(
                        lyng_js_types::AbruptCompletion::throw(thrown),
                    )),
                    GeneratorExecutionOutcome::AsyncSuspend => {
                        let _ = self.complete_generator_object(agent, generator);
                        Err(VmError::AsyncSuspend)
                    }
                }
            }
        }
    }

    pub(super) fn resume_async_generator(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        generator: ObjectRef,
        resume_kind: GeneratorResumeKind,
        value: Value,
    ) -> VmResult<Value> {
        if !self.is_async_generator_object(generator) {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        let capability = self.create_intrinsic_promise_capability(agent, caller_frame.realm())?;
        let promise = self.promise_capability_promise(agent, capability)?;
        self.async_generator_queues
            .entry(generator)
            .or_default()
            .push_back(AsyncGeneratorRequest {
                kind: resume_kind,
                value,
                capability,
                realm: caller_frame.realm(),
            });
        self.drain_async_generator_queue(agent, host, registry, generator)?;
        Ok(Value::from_object_ref(promise))
    }

    pub(super) fn resume_async_generator_from_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        this_value: Value,
        resume_kind: GeneratorResumeKind,
        value: Value,
    ) -> VmResult<Value> {
        let capability = self.create_intrinsic_promise_capability(agent, caller_frame.realm())?;
        let promise = self.promise_capability_promise(agent, capability)?;
        let realm = agent
            .realm(caller_frame.realm())
            .ok_or(VmError::MissingRootShape(caller_frame.realm()))?;
        let Some(generator) = this_value.as_object_ref() else {
            let type_error_value = errors::throw_type_error(agent)
                .thrown_value()
                .unwrap_or(Value::undefined());
            self.settle_promise_capability(
                agent,
                host,
                registry,
                realm,
                capability,
                true,
                type_error_value,
            )?;
            return Ok(Value::from_object_ref(promise));
        };
        if !self.is_async_generator_object(generator) {
            let type_error_value = errors::throw_type_error(agent)
                .thrown_value()
                .unwrap_or(Value::undefined());
            self.settle_promise_capability(
                agent,
                host,
                registry,
                realm,
                capability,
                true,
                type_error_value,
            )?;
            return Ok(Value::from_object_ref(promise));
        }
        self.async_generator_queues
            .entry(generator)
            .or_default()
            .push_back(AsyncGeneratorRequest {
                kind: resume_kind,
                value,
                capability,
                realm: caller_frame.realm(),
            });
        self.drain_async_generator_queue(agent, host, registry, generator)?;
        Ok(Value::from_object_ref(promise))
    }

    pub(super) fn is_async_generator_object(&self, generator: ObjectRef) -> bool {
        self.async_generator_objects.contains(&generator)
    }

    pub(super) fn resume_async_generator_request(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        suspended: SuspendedExecutionRef,
        rejected: bool,
        argument: Value,
    ) -> VmResult<()> {
        let resume_kind = if rejected {
            GeneratorResumeKind::Throw
        } else {
            GeneratorResumeKind::Next
        };
        self.resume_async_generator_request_with_kind(
            agent,
            host,
            registry,
            suspended,
            resume_kind,
            argument,
        )
    }

    pub(super) fn resume_async_generator_request_with_kind(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        suspended: SuspendedExecutionRef,
        resume_kind: GeneratorResumeKind,
        argument: Value,
    ) -> VmResult<()> {
        let frame_state = self
            .suspended_side_states
            .get(&suspended)
            .and_then(|state| state.async_generator_frame_state)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let front_request = self
            .async_generator_queues
            .get(&frame_state.generator)
            .and_then(|queue| queue.front().copied())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        if front_request.capability != frame_state.capability {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }

        self.restore_suspended_execution(agent, suspended, resume_kind, argument)?;
        let outcome =
            self.run_generator_frame(agent, host, registry, frame_state.generator, None)?;
        if matches!(outcome, GeneratorExecutionOutcome::AsyncSuspend) {
            return Ok(());
        }
        self.finish_async_generator_front_request(
            agent,
            host,
            registry,
            frame_state.generator,
            outcome,
        )?;
        self.drain_async_generator_queue(agent, host, registry, frame_state.generator)
    }

    pub(super) fn resume_async_generator_return_from_suspended_yield(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        suspended: SuspendedExecutionRef,
        rejected: bool,
        argument: Value,
    ) -> VmResult<()> {
        let generator = self
            .suspended_side_states
            .get(&suspended)
            .and_then(|state| state.async_generator_frame_state)
            .map(|state| state.generator)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let front_request = self
            .async_generator_queues
            .get(&generator)
            .and_then(|queue| queue.front().copied())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        if front_request.kind != GeneratorResumeKind::Return {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }

        let resume_kind = if rejected {
            GeneratorResumeKind::Throw
        } else {
            GeneratorResumeKind::Return
        };
        self.restore_suspended_execution(agent, suspended, resume_kind, argument)?;
        let frame_state = AsyncGeneratorFrameState {
            generator,
            capability: front_request.capability,
            realm: front_request.realm,
        };
        let outcome =
            self.run_generator_frame(agent, host, registry, generator, Some(frame_state))?;
        if matches!(outcome, GeneratorExecutionOutcome::AsyncSuspend) {
            return Ok(());
        }
        self.finish_async_generator_front_request(agent, host, registry, generator, outcome)?;
        self.drain_async_generator_queue(agent, host, registry, generator)
    }

    pub(super) fn drain_async_generator_queue(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        generator: ObjectRef,
    ) -> VmResult<()> {
        loop {
            let request = match self
                .async_generator_queues
                .get(&generator)
                .and_then(|queue| queue.front().copied())
            {
                Some(request) => request,
                None => return Ok(()),
            };
            let (state, suspended) = {
                let objects = agent.objects();
                if !objects.is_generator_object(generator) {
                    return Err(VmError::Abrupt(errors::throw_type_error(agent)));
                }
                (
                    objects.generator_state(generator),
                    objects.generator_suspended(agent.heap().view(), generator),
                )
            };
            let state = state.ok_or(VmError::Abrupt(errors::throw_type_error(agent)))?;

            match state {
                GeneratorState::Executing => return Ok(()),
                GeneratorState::Completed => {
                    if request.kind == GeneratorResumeKind::Return {
                        self.await_async_generator_return_completion(
                            agent, host, registry, generator, request,
                        )?;
                        return Ok(());
                    }
                    let completion = self.generator_completion_result(
                        agent,
                        request.realm,
                        request.kind,
                        request.value,
                    );
                    self.settle_async_generator_request_completion(
                        agent, host, registry, generator, request, completion,
                    )?;
                }
                GeneratorState::SuspendedStart if request.kind != GeneratorResumeKind::Next => {
                    self.complete_generator_object(agent, generator)?;
                    if request.kind == GeneratorResumeKind::Return {
                        self.await_async_generator_return_completion(
                            agent, host, registry, generator, request,
                        )?;
                        return Ok(());
                    }
                    let completion = self.generator_completion_result(
                        agent,
                        request.realm,
                        request.kind,
                        request.value,
                    );
                    self.settle_async_generator_request_completion(
                        agent, host, registry, generator, request, completion,
                    )?;
                }
                GeneratorState::SuspendedYield if request.kind == GeneratorResumeKind::Return => {
                    let suspended = suspended
                        .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
                    self.set_generator_state(agent, generator, GeneratorState::Executing, None)?;
                    self.await_suspended_async_generator_return_completion(
                        agent, host, registry, suspended, request,
                    )?;
                    return Ok(());
                }
                GeneratorState::SuspendedStart | GeneratorState::SuspendedYield => {
                    let suspended = suspended
                        .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
                    self.set_generator_state(agent, generator, GeneratorState::Executing, None)?;
                    let effective_value = if state == GeneratorState::SuspendedStart
                        && request.kind == GeneratorResumeKind::Next
                    {
                        Value::undefined()
                    } else {
                        request.value
                    };
                    if let Err(error) = self.restore_suspended_execution(
                        agent,
                        suspended,
                        request.kind,
                        effective_value,
                    ) {
                        let _ = self.complete_generator_object(agent, generator);
                        return Err(error);
                    }
                    if state == GeneratorState::SuspendedStart
                        && request.kind == GeneratorResumeKind::Next
                    {
                        self.clear_active_resume();
                    }
                    let frame_state = AsyncGeneratorFrameState {
                        generator,
                        capability: request.capability,
                        realm: request.realm,
                    };
                    let outcome = self.run_generator_frame(
                        agent,
                        host,
                        registry,
                        generator,
                        Some(frame_state),
                    )?;
                    if matches!(outcome, GeneratorExecutionOutcome::AsyncSuspend) {
                        return Ok(());
                    }
                    self.finish_async_generator_front_request(
                        agent, host, registry, generator, outcome,
                    )?;
                }
            }
        }
    }

    fn finish_async_generator_front_request(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        generator: ObjectRef,
        outcome: GeneratorExecutionOutcome,
    ) -> VmResult<()> {
        let request = self
            .async_generator_queues
            .get(&generator)
            .and_then(|queue| queue.front().copied())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        match outcome {
            GeneratorExecutionOutcome::Complete(value) => {
                let result = self.generator_result_object(agent, request.realm, value, true);
                self.settle_async_generator_request_completion(
                    agent, host, registry, generator, request, result,
                )
            }
            GeneratorExecutionOutcome::Yield {
                value,
                raw_iterator_result,
            } => {
                let result = if raw_iterator_result {
                    Ok(value)
                } else {
                    self.generator_result_object(agent, request.realm, value, false)
                };
                self.settle_async_generator_request_completion(
                    agent, host, registry, generator, request, result,
                )
            }
            GeneratorExecutionOutcome::Throw(thrown) => {
                let result = Err(VmError::Abrupt(lyng_js_types::AbruptCompletion::throw(
                    thrown,
                )));
                self.settle_async_generator_request_completion(
                    agent, host, registry, generator, request, result,
                )
            }
            GeneratorExecutionOutcome::AsyncSuspend => Ok(()),
        }
    }

    fn settle_async_generator_request_completion(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        generator: ObjectRef,
        request: AsyncGeneratorRequest,
        completion: VmResult<Value>,
    ) -> VmResult<()> {
        let realm = agent
            .realm(request.realm)
            .ok_or(VmError::MissingRootShape(request.realm))?;
        match completion {
            Ok(value) => {
                self.settle_promise_capability(
                    agent,
                    host,
                    registry,
                    realm,
                    request.capability,
                    false,
                    value,
                )?;
            }
            Err(VmError::Abrupt(lyng_js_types::AbruptCompletion::Throw(thrown))) => {
                self.settle_promise_capability(
                    agent,
                    host,
                    registry,
                    realm,
                    request.capability,
                    true,
                    thrown,
                )?;
            }
            Err(error) => return Err(error),
        }
        self.pop_async_generator_front_request(generator);
        Ok(())
    }

    fn await_async_generator_return_completion(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        generator: ObjectRef,
        request: AsyncGeneratorRequest,
    ) -> VmResult<()> {
        let realm = agent
            .realm(request.realm)
            .ok_or(VmError::MissingRootShape(request.realm))?;
        let caller = self.synthetic_job_caller_frame(realm);
        let promise = match self.promise_resolve_in_realm(
            agent,
            host,
            registry,
            caller,
            request.realm,
            request.value,
        ) {
            Ok(promise) => promise,
            Err(VmError::Abrupt(completion)) => {
                self.settle_async_generator_request_completion(
                    agent,
                    host,
                    registry,
                    generator,
                    request,
                    Err(VmError::Abrupt(completion)),
                )?;
                return Ok(());
            }
            Err(error) => return Err(error),
        };
        self.enqueue_promise_then(
            agent,
            realm,
            promise,
            lyng_js_env::PromiseReactionHandler::AsyncGeneratorReturn {
                generator,
                reject: false,
            },
            lyng_js_env::PromiseReactionHandler::AsyncGeneratorReturn {
                generator,
                reject: true,
            },
            Some(request.capability),
        )
    }

    fn await_suspended_async_generator_return_completion(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        suspended: SuspendedExecutionRef,
        request: AsyncGeneratorRequest,
    ) -> VmResult<()> {
        let realm = agent
            .realm(request.realm)
            .ok_or(VmError::MissingRootShape(request.realm))?;
        let caller = self.synthetic_job_caller_frame(realm);
        let promise = match self.promise_resolve_in_realm(
            agent,
            host,
            registry,
            caller,
            request.realm,
            request.value,
        ) {
            Ok(promise) => promise,
            Err(VmError::Abrupt(completion)) => {
                let thrown = completion.thrown_value().unwrap_or(Value::undefined());
                self.resume_async_generator_return_from_suspended_yield(
                    agent, host, registry, suspended, true, thrown,
                )?;
                return Ok(());
            }
            Err(error) => return Err(error),
        };
        self.enqueue_promise_then(
            agent,
            realm,
            promise,
            lyng_js_env::PromiseReactionHandler::AsyncGeneratorReturnResume {
                suspended,
                reject: false,
            },
            lyng_js_env::PromiseReactionHandler::AsyncGeneratorReturnResume {
                suspended,
                reject: true,
            },
            None,
        )
    }

    pub(super) fn pop_async_generator_front_request(&mut self, generator: ObjectRef) {
        let remove_queue = if let Some(queue) = self.async_generator_queues.get_mut(&generator) {
            let _ = queue.pop_front();
            queue.is_empty()
        } else {
            false
        };
        if remove_queue {
            let _ = self.async_generator_queues.remove(&generator);
        }
    }

    fn run_generator_frame(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        generator: ObjectRef,
        async_generator_state: Option<AsyncGeneratorFrameState>,
    ) -> VmResult<GeneratorExecutionOutcome> {
        let frame_base = self
            .frame()
            .map(|frame| frame.registers().base())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        if let Some(state) = async_generator_state {
            self.async_generator_frame_states.insert(frame_base, state);
        }
        let prior_frame_depth = self.frames.len().saturating_sub(1);
        let prior_context_depth = agent.execution_contexts().len().saturating_sub(1);
        let prior_register_len = usize::try_from(
            self.frames
                .get(prior_frame_depth)
                .map(|frame| frame.registers().end())
                .unwrap_or(0),
        )
        .expect("prior register length should fit usize");
        self.internal_completion_targets.push(prior_frame_depth);

        let result = self.run(agent, host, registry);
        if self.internal_completion_targets.last().copied() == Some(prior_frame_depth) {
            let _ = self.internal_completion_targets.pop();
        }
        self.cleanup_internal_completion(
            agent,
            prior_frame_depth,
            prior_context_depth,
            prior_register_len,
        )?;
        let _ = self.async_generator_frame_states.remove(&frame_base);

        match result {
            Ok(value) => {
                self.complete_generator_object(agent, generator)?;
                Ok(GeneratorExecutionOutcome::Complete(value))
            }
            Err(VmError::GeneratorYield {
                value,
                suspended,
                raw_iterator_result,
            }) => {
                self.set_generator_state(
                    agent,
                    generator,
                    GeneratorState::SuspendedYield,
                    Some(suspended),
                )?;
                Ok(GeneratorExecutionOutcome::Yield {
                    value,
                    raw_iterator_result,
                })
            }
            Err(VmError::AsyncSuspend) => Ok(GeneratorExecutionOutcome::AsyncSuspend),
            Err(VmError::Abrupt(lyng_js_types::AbruptCompletion::Throw(thrown))) => {
                self.complete_generator_object(agent, generator)?;
                Ok(GeneratorExecutionOutcome::Throw(thrown))
            }
            Err(error) => {
                let _ = self.complete_generator_object(agent, generator);
                Err(error)
            }
        }
    }

    pub(super) fn suspend_current_generator_frame(
        &mut self,
        agent: &mut Agent,
        frame: FrameRecord,
        yielded_value: Value,
        resume_instruction_offset: u32,
        raw_iterator_result: bool,
    ) -> VmResult<()> {
        let suspended =
            self.snapshot_suspended_execution(agent, frame, resume_instruction_offset)?;
        let active = self
            .frames
            .pop()
            .expect("generator suspension requires one active frame");
        debug_assert_eq!(active, frame);
        self.register_stack.truncate(
            usize::try_from(frame.registers().base()).expect("base should fit into usize"),
        );
        let _ = self.current_exception.take();
        let _ = agent.pop_execution_context();
        Err(VmError::GeneratorYield {
            value: yielded_value,
            suspended,
            raw_iterator_result,
        })
    }

    pub(super) fn suspend_generator_start(
        &mut self,
        agent: &mut Agent,
        frame: FrameRecord,
        resume_instruction_offset: u32,
    ) -> VmResult<()> {
        let suspended =
            self.snapshot_suspended_execution(agent, frame, resume_instruction_offset)?;
        let active = self
            .frames
            .pop()
            .expect("generator start suspension requires one active frame");
        debug_assert_eq!(active, frame);
        self.register_stack.truncate(
            usize::try_from(frame.registers().base()).expect("base should fit into usize"),
        );
        let _ = self.current_exception.take();
        let _ = agent.pop_execution_context();
        Err(VmError::GeneratorStart { suspended })
    }

    pub(super) fn delegate_yield(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        iterator_register: u16,
        result_register: u16,
        done_register: u16,
    ) -> VmResult<()> {
        let register_base = frame.registers().base();
        let mut record = self
            .iterator_states
            .remove(register_base, iterator_register)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;

        if record.delegate_yield_await_state() != iterator::DelegateYieldAwaitState::None {
            return self.resume_async_delegate_yield(
                agent,
                host,
                registry,
                frame,
                iterator_register,
                result_register,
                done_register,
                record,
            );
        }

        let outcome = {
            let mut bridge = VmIteratorBridge {
                vm: self,
                agent,
                host,
                registry,
                frame,
            };
            let receiver = Value::from_object_ref(record.iterator());
            let resume_kind = frame.resume_kind();
            let resume_value = frame.resume_value();

            match resume_kind {
                GeneratorResumeKind::Next => {
                    let argument = Some(if record.delegate_started() {
                        resume_value
                    } else {
                        Value::undefined()
                    });
                    record.set_delegate_started(true);
                    if record.is_async() {
                        return self.start_async_delegate_next(
                            agent,
                            host,
                            registry,
                            frame,
                            iterator_register,
                            record,
                            argument,
                        );
                    }
                    let iter_result = iterator::iterator_next(&mut bridge, &record, argument)?;
                    let done = iterator::iterator_complete(&mut bridge, iter_result)?;
                    if done {
                        let value = iterator::iterator_value(&mut bridge, iter_result)?;
                        record.set_done(true);
                        DelegateYieldOutcome::Complete { value }
                    } else {
                        DelegateYieldOutcome::Suspend {
                            value: Value::from_object_ref(iter_result),
                            record,
                            raw_iterator_result: true,
                        }
                    }
                }
                GeneratorResumeKind::Throw => {
                    let throw_method = bridge.get_property_value(
                        receiver,
                        PropertyKey::from_atom(WellKnownAtom::throw.id()),
                    )?;
                    if throw_method.is_undefined() || throw_method.is_null() {
                        let return_method = bridge.get_property_value(
                            receiver,
                            PropertyKey::from_atom(WellKnownAtom::r#return.id()),
                        )?;
                        if !return_method.is_undefined() && !return_method.is_null() {
                            let return_method =
                                Self::require_callable_object(bridge.agent, frame, return_method)?;
                            let close_result = bridge.vm.call_to_completion(
                                bridge.agent,
                                bridge.host,
                                bridge.registry,
                                frame,
                                return_method,
                                receiver,
                                &[],
                            )?;
                            let _ = close_result.as_object_ref().ok_or_else(|| {
                                VmError::Abrupt(errors::throw_type_error(bridge.agent))
                            })?;
                        }
                        return Err(VmError::Abrupt(errors::throw_type_error(bridge.agent)));
                    }
                    let throw_method =
                        Self::require_callable_object(bridge.agent, frame, throw_method)?;
                    let result = bridge.vm.call_to_completion(
                        bridge.agent,
                        bridge.host,
                        bridge.registry,
                        frame,
                        throw_method,
                        receiver,
                        &[resume_value],
                    )?;
                    record.set_delegate_started(true);
                    if record.is_async() {
                        return self.start_async_delegate_iterator_result_await(
                            agent,
                            host,
                            registry,
                            frame,
                            iterator_register,
                            record,
                            result,
                            false,
                        );
                    }
                    let iter_result = result
                        .as_object_ref()
                        .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(bridge.agent)))?;
                    let done = iterator::iterator_complete(&mut bridge, iter_result)?;
                    if done {
                        let value = iterator::iterator_value(&mut bridge, iter_result)?;
                        record.set_done(true);
                        DelegateYieldOutcome::Complete { value }
                    } else {
                        DelegateYieldOutcome::Suspend {
                            value: Value::from_object_ref(iter_result),
                            record,
                            raw_iterator_result: true,
                        }
                    }
                }
                GeneratorResumeKind::Return => {
                    if record.is_async() {
                        return self.finish_delegate_return_resume(
                            agent,
                            host,
                            registry,
                            frame,
                            iterator_register,
                            result_register,
                            done_register,
                            record,
                            resume_value,
                        );
                    }
                    let return_method = bridge.get_property_value(
                        receiver,
                        PropertyKey::from_atom(WellKnownAtom::r#return.id()),
                    )?;
                    if return_method.is_undefined() || return_method.is_null() {
                        DelegateYieldOutcome::Complete {
                            value: resume_value,
                        }
                    } else {
                        let return_method =
                            Self::require_callable_object(bridge.agent, frame, return_method)?;
                        let result = bridge.vm.call_to_completion(
                            bridge.agent,
                            bridge.host,
                            bridge.registry,
                            frame,
                            return_method,
                            receiver,
                            &[resume_value],
                        )?;
                        let iter_result = result.as_object_ref().ok_or_else(|| {
                            VmError::Abrupt(errors::throw_type_error(bridge.agent))
                        })?;
                        let done = iterator::iterator_complete(&mut bridge, iter_result)?;
                        if done {
                            let value = iterator::iterator_value(&mut bridge, iter_result)?;
                            record.set_done(true);
                            DelegateYieldOutcome::Complete { value }
                        } else {
                            DelegateYieldOutcome::Suspend {
                                value: Value::from_object_ref(iter_result),
                                record,
                                raw_iterator_result: true,
                            }
                        }
                    }
                }
            }
        };

        self.finish_delegate_yield_outcome(
            agent,
            frame,
            result_register,
            done_register,
            register_base,
            iterator_register,
            outcome,
        )
    }

    fn finish_delegate_return_resume(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        iterator_register: u16,
        result_register: u16,
        done_register: u16,
        mut record: iterator::IteratorRecord,
        resume_value: Value,
    ) -> VmResult<()> {
        let receiver = Value::from_object_ref(record.iterator());
        let outcome = {
            let mut bridge = VmIteratorBridge {
                vm: self,
                agent,
                host,
                registry,
                frame,
            };
            let return_method = bridge.get_property_value(
                receiver,
                PropertyKey::from_atom(WellKnownAtom::r#return.id()),
            )?;
            if return_method.is_undefined() || return_method.is_null() {
                if record.is_async() {
                    return self.start_async_delegate_value_await(
                        agent,
                        host,
                        registry,
                        frame,
                        iterator_register,
                        record,
                        resume_value,
                        true,
                        true,
                    );
                }
                DelegateYieldOutcome::Complete {
                    value: resume_value,
                }
            } else {
                let return_method =
                    Self::require_callable_object(bridge.agent, frame, return_method)?;
                let result = bridge.vm.call_to_completion(
                    bridge.agent,
                    bridge.host,
                    bridge.registry,
                    frame,
                    return_method,
                    receiver,
                    &[resume_value],
                )?;
                record.set_delegate_started(true);
                if record.is_async() {
                    return self.start_async_delegate_iterator_result_await(
                        agent,
                        host,
                        registry,
                        frame,
                        iterator_register,
                        record,
                        result,
                        true,
                    );
                }
                let iter_result = result
                    .as_object_ref()
                    .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(bridge.agent)))?;
                let done = iterator::iterator_complete(&mut bridge, iter_result)?;
                if done {
                    let value = iterator::iterator_value(&mut bridge, iter_result)?;
                    record.set_done(true);
                    DelegateYieldOutcome::Complete { value }
                } else {
                    DelegateYieldOutcome::Suspend {
                        value: Value::from_object_ref(iter_result),
                        record,
                        raw_iterator_result: true,
                    }
                }
            }
        };

        self.finish_delegate_yield_outcome(
            agent,
            frame,
            result_register,
            done_register,
            frame.registers().base(),
            iterator_register,
            outcome,
        )
    }

    fn finish_delegate_yield_outcome(
        &mut self,
        agent: &mut Agent,
        frame: FrameRecord,
        result_register: u16,
        done_register: u16,
        register_base: u32,
        iterator_register: u16,
        outcome: DelegateYieldOutcome,
    ) -> VmResult<()> {
        match outcome {
            DelegateYieldOutcome::Suspend {
                value,
                record,
                raw_iterator_result,
            } => {
                self.iterator_states
                    .insert(register_base, iterator_register, record);
                self.write_register(frame, result_register, value)?;
                self.write_register(frame, done_register, Value::from_bool(false))?;
                self.suspend_current_generator_frame(
                    agent,
                    frame,
                    value,
                    frame.instruction_offset(),
                    raw_iterator_result,
                )
            }
            DelegateYieldOutcome::Complete { value } => {
                self.write_register(frame, result_register, value)?;
                self.write_register(frame, done_register, Value::from_bool(true))?;
                self.advance_instruction()?;
                Ok(())
            }
        }
    }

    fn start_async_delegate_next(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        iterator_register: u16,
        record: iterator::IteratorRecord,
        argument: Option<Value>,
    ) -> VmResult<()> {
        match record.kind() {
            iterator::IteratorKind::Async => {
                let receiver = Value::from_object_ref(record.iterator());
                let next_method =
                    Self::require_callable_object(agent, frame, record.next_method())?;
                let mut arguments = [Value::undefined(); 1];
                let arguments = if let Some(argument) = argument {
                    arguments[0] = argument;
                    &arguments[..1]
                } else {
                    &arguments[..0]
                };
                let result = self.call_to_completion(
                    agent,
                    host,
                    registry,
                    frame,
                    next_method,
                    receiver,
                    arguments,
                )?;
                self.start_async_delegate_iterator_result_await(
                    agent,
                    host,
                    registry,
                    frame,
                    iterator_register,
                    record,
                    result,
                    false,
                )
            }
            iterator::IteratorKind::AsyncFromSync => {
                let iter_result = {
                    let mut bridge = VmIteratorBridge {
                        vm: self,
                        agent,
                        host,
                        registry,
                        frame,
                    };
                    iterator::iterator_next(&mut bridge, &record, argument)?
                };
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
                self.start_async_delegate_value_await(
                    agent,
                    host,
                    registry,
                    frame,
                    iterator_register,
                    record,
                    value,
                    done,
                    false,
                )
            }
            iterator::IteratorKind::Sync => Err(VmError::Abrupt(errors::throw_type_error(agent))),
        }
    }

    fn start_async_delegate_iterator_result_await(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        iterator_register: u16,
        mut record: iterator::IteratorRecord,
        result: Value,
        return_completion: bool,
    ) -> VmResult<()> {
        let promise =
            self.promise_resolve_in_realm(agent, host, registry, frame, frame.realm(), result)?;
        record.set_delegate_yield_await_state(iterator::DelegateYieldAwaitState::IteratorResult {
            return_completion,
        });
        self.iterator_states
            .insert(frame.registers().base(), iterator_register, record);
        self.suspend_for_await_promise(agent, frame, promise)
    }

    fn start_async_delegate_value_await(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        iterator_register: u16,
        mut record: iterator::IteratorRecord,
        value: Value,
        done: bool,
        return_completion: bool,
    ) -> VmResult<()> {
        let promise =
            match self.promise_resolve_in_realm(agent, host, registry, frame, frame.realm(), value)
            {
                Ok(promise) => promise,
                Err(VmError::Abrupt(completion)) if record.is_async_from_sync() && !done => {
                    let mut record = record;
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
                        Ok(()) => return Err(VmError::Abrupt(completion)),
                        Err(error) => return Err(error),
                    }
                }
                Err(error) => return Err(error),
            };
        record.set_delegate_yield_await_state(iterator::DelegateYieldAwaitState::Value {
            done,
            return_completion,
        });
        self.iterator_states
            .insert(frame.registers().base(), iterator_register, record);
        self.suspend_for_await_promise(agent, frame, promise)
    }

    fn resume_async_delegate_yield(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        iterator_register: u16,
        result_register: u16,
        done_register: u16,
        mut record: iterator::IteratorRecord,
    ) -> VmResult<()> {
        let await_state = record.delegate_yield_await_state();
        record.set_delegate_yield_await_state(iterator::DelegateYieldAwaitState::None);
        let resume_kind = frame.resume_kind();
        let resume_value = frame.resume_value();
        self.clear_active_resume();
        let frame = self
            .frame()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        if resume_kind == GeneratorResumeKind::Throw {
            if let iterator::DelegateYieldAwaitState::Value { done: false, .. } = await_state {
                if record.is_async_from_sync() {
                    let mut bridge = VmIteratorBridge {
                        vm: self,
                        agent,
                        host,
                        registry,
                        frame,
                    };
                    let _: () = iterator::iterator_close(
                        &mut bridge,
                        &mut record,
                        Err(lyng_js_types::AbruptCompletion::Throw(resume_value)),
                    )?;
                }
            }
            return Err(VmError::Abrupt(lyng_js_types::AbruptCompletion::Throw(
                resume_value,
            )));
        }

        match await_state {
            iterator::DelegateYieldAwaitState::IteratorResult { return_completion } => {
                let iter_result = resume_value
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
                if record.is_async_from_sync() || (done && return_completion) {
                    return self.start_async_delegate_value_await(
                        agent,
                        host,
                        registry,
                        frame,
                        iterator_register,
                        record,
                        value,
                        done,
                        return_completion && done,
                    );
                }
                let outcome = if done {
                    record.set_done(true);
                    DelegateYieldOutcome::Complete { value }
                } else {
                    DelegateYieldOutcome::Suspend {
                        value,
                        record,
                        raw_iterator_result: false,
                    }
                };
                self.finish_delegate_yield_outcome(
                    agent,
                    frame,
                    result_register,
                    done_register,
                    frame.registers().base(),
                    iterator_register,
                    outcome,
                )
            }
            iterator::DelegateYieldAwaitState::Value {
                done,
                return_completion,
            } => {
                if done {
                    record.set_done(true);
                    if return_completion {
                        self.reactivate_delegate_return_completion(agent, resume_value)?;
                    }
                    self.finish_delegate_yield_outcome(
                        agent,
                        frame,
                        result_register,
                        done_register,
                        frame.registers().base(),
                        iterator_register,
                        DelegateYieldOutcome::Complete {
                            value: resume_value,
                        },
                    )
                } else {
                    self.finish_delegate_yield_outcome(
                        agent,
                        frame,
                        result_register,
                        done_register,
                        frame.registers().base(),
                        iterator_register,
                        DelegateYieldOutcome::Suspend {
                            value: resume_value,
                            record,
                            raw_iterator_result: false,
                        },
                    )
                }
            }
            iterator::DelegateYieldAwaitState::None => {
                Err(VmError::Abrupt(errors::throw_type_error(agent)))
            }
        }
    }

    fn reactivate_delegate_return_completion(
        &mut self,
        agent: &mut Agent,
        value: Value,
    ) -> VmResult<()> {
        let frame = self
            .frames
            .last_mut()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        *frame = frame.with_resume(GeneratorResumeKind::Return, value);
        Ok(())
    }

    pub(super) fn restore_suspended_execution(
        &mut self,
        agent: &mut Agent,
        suspended: SuspendedExecutionRef,
        resume_kind: GeneratorResumeKind,
        resume_value: Value,
    ) -> VmResult<()> {
        let (record, saved_registers) = {
            let view = agent.heap().view();
            let record = view.suspended_execution(suspended);
            let saved_registers = record
                .and_then(|record| {
                    record
                        .registers()
                        .and_then(|registers| view.suspended_registers(registers))
                        .map(<[Value]>::to_vec)
                })
                .unwrap_or_default();
            (record, saved_registers)
        };
        let record = record.ok_or(VmError::Abrupt(errors::throw_type_error(agent)))?;
        {
            let _ = agent
                .heap_mut()
                .mutator()
                .free_suspended_execution(suspended);
        }

        let register_base =
            u32::try_from(self.register_stack.len()).expect("register stack length should fit u32");
        let register_len = u16::try_from(saved_registers.len())
            .expect("suspended register window length should fit u16");
        self.reserve_register_window(register_base, register_len);
        let start =
            usize::try_from(register_base).expect("suspended register base should fit usize");
        for (index, value) in saved_registers.into_iter().enumerate() {
            self.register_stack[start + index] = value;
        }

        let side_state = self.suspended_side_states.remove(&suspended);
        let script_or_module_referrer = side_state
            .as_ref()
            .and_then(|state| state.script_or_module_referrer);

        let context_kind = decode_execution_context_kind(record.context_kind_raw())
            .unwrap_or(ExecutionContextKind::Function);
        let context = ExecutionContext::new(
            context_kind,
            record.realm(),
            ExecutableId::Bytecode(record.code()),
            record.lexical_env(),
            record.variable_env(),
        )
        .with_private_env(record.private_env())
        .with_this_state(decode_this_state(
            record.this_state_kind(),
            record.this_value(),
        ))
        .with_script_or_module_referrer(script_or_module_referrer)
        .with_new_target(record.new_target());
        let mut frame = FrameRecord::new(
            record.code(),
            0,
            RegisterWindow::new(register_base, register_len),
            None,
            record.realm(),
            record.lexical_env(),
            record.variable_env(),
            context_kind,
        )
        .with_this_value(record.this_value())
        .with_construct_this(record.construct_this())
        .with_new_target(record.new_target())
        .with_callee(record.callee())
        .with_handler_cursor(record.handler_cursor())
        .with_flags(FrameFlags::from_raw(record.frame_flags_raw()))
        .with_resume(resume_kind, resume_value);
        frame.set_instruction_offset(record.instruction_offset());

        agent.push_execution_context(context);
        self.frames.push(frame);
        self.note_frame_depth();

        if let Some(side_state) = side_state {
            self.iterator_states
                .restore_window(frame.registers(), side_state.iterator_states);
            self.for_in_states
                .restore_window(frame.registers(), side_state.for_in_states);
            self.captured_name_references
                .restore_window(frame.registers(), side_state.captured_name_references);
            self.restore_loop_iteration_state(self.frames.len(), side_state.loop_iteration_envs);
            self.restore_with_environment_state(
                self.frames.len(),
                side_state.with_environment_states,
            );
            self.restore_direct_eval_environment_state(
                self.frames.len(),
                side_state.direct_eval_environment_states,
            );
            if let Some(async_state) = side_state.async_frame_state {
                self.async_frame_states
                    .insert(frame.registers().base(), async_state);
            }
            if let Some(async_generator_state) = side_state.async_generator_frame_state {
                self.async_generator_frame_states
                    .insert(frame.registers().base(), async_generator_state);
            }
        }
        Ok(())
    }

    pub(super) fn snapshot_suspended_execution(
        &mut self,
        agent: &mut Agent,
        frame: FrameRecord,
        instruction_offset: u32,
    ) -> VmResult<SuspendedExecutionRef> {
        let context = agent
            .current_execution_context()
            .ok_or(VmError::MissingEnvironment(frame.lexical_env()))?;
        let register_base =
            usize::try_from(frame.registers().base()).expect("register base should fit usize");
        let register_end =
            usize::try_from(frame.registers().end()).expect("register end should fit usize");
        let register_values = self.register_stack[register_base..register_end].to_vec();

        let registers = {
            let mut mutator = agent.heap_mut().mutator();
            if register_values.is_empty() {
                None
            } else {
                let registers = mutator.alloc_suspended_registers(
                    register_values.len(),
                    Value::undefined(),
                    AllocationLifetime::Default,
                );
                for (index, value) in register_values.into_iter().enumerate() {
                    assert!(
                        mutator.write_suspended_register(
                            registers,
                            u32::try_from(index).expect("suspended register index should fit u32"),
                            value,
                        ),
                        "suspended register slot should remain writable"
                    );
                }
                Some(registers)
            }
        };

        let suspended = {
            let mut mutator = agent.heap_mut().mutator();
            mutator.alloc_suspended_execution(
                RuntimeSuspendedExecutionRecord::new(
                    frame.realm(),
                    frame.code(),
                    instruction_offset,
                    frame.lexical_env(),
                    frame.variable_env(),
                    context.private_env(),
                    frame.this_value(),
                    encode_this_state_kind(context.this_state()),
                    frame.construct_this(),
                    frame.new_target(),
                    frame.callee(),
                    frame.handler_cursor(),
                    frame.flags().raw(),
                    encode_execution_context_kind(frame.kind()),
                    registers,
                ),
                AllocationLifetime::Default,
            )
        };

        let frame_depth = self.frames.len();
        let side_state = SuspendedExecutionSideState {
            iterator_states: self.iterator_states.drain_window(frame.registers()),
            for_in_states: self.for_in_states.drain_window(frame.registers()),
            captured_name_references: self
                .captured_name_references
                .drain_window(frame.registers()),
            loop_iteration_envs: self.drain_loop_iteration_state(frame_depth),
            with_environment_states: self.drain_with_environment_state(frame_depth),
            direct_eval_environment_states: self.drain_direct_eval_environment_state(frame_depth),
            async_frame_state: self.async_frame_states.remove(&frame.registers().base()),
            async_generator_frame_state: self
                .async_generator_frame_states
                .remove(&frame.registers().base()),
            script_or_module_referrer: context.script_or_module_referrer(),
        };
        if !side_state.iterator_states.is_empty()
            || !side_state.for_in_states.is_empty()
            || !side_state.captured_name_references.is_empty()
            || !side_state.loop_iteration_envs.is_empty()
            || !side_state.with_environment_states.is_empty()
            || !side_state.direct_eval_environment_states.is_empty()
            || side_state.async_frame_state.is_some()
            || side_state.async_generator_frame_state.is_some()
            || side_state.script_or_module_referrer.is_some()
        {
            self.suspended_side_states.insert(suspended, side_state);
        }

        Ok(suspended)
    }

    fn create_generator_object(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        suspended: SuspendedExecutionRef,
        async_generator: bool,
    ) -> VmResult<ObjectRef> {
        let default_prototype = agent
            .realm(realm)
            .and_then(|record| {
                if async_generator {
                    record.intrinsics().async_generator_prototype()
                } else {
                    record.intrinsics().generator_prototype()
                }
            })
            .ok_or(VmError::MissingRootShape(realm))?;
        let prototype = {
            let view = agent.heap().view();
            let record = view
                .suspended_execution(suspended)
                .ok_or(VmError::Abrupt(errors::throw_type_error(agent)))?;
            match record.callee() {
                Some(callee) => object::ordinary_get(
                    agent,
                    callee,
                    PropertyKey::from_atom(WellKnownAtom::prototype.id()),
                )
                .map_err(VmError::Abrupt)?
                .as_object_ref()
                .unwrap_or(default_prototype),
                None => default_prototype,
            }
        };
        let realm_record = agent.realm(realm).ok_or(VmError::MissingRootShape(realm))?;
        let root_shape = realm_record
            .root_shape()
            .ok_or(VmError::MissingRootShape(realm))?;
        Ok(agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let object = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape)
                    .with_prototype(Some(prototype))
                    .with_ordinary_payload_value(Value::from_suspended_execution_ref(suspended))
                    .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::Generator)),
                AllocationLifetime::Default,
            );
            let _ = objects.install_generator_object(object, GeneratorState::SuspendedStart);
            if async_generator {
                let _ = self.async_generator_objects.insert(object);
            }
            object
        }))
    }

    fn set_generator_state(
        &mut self,
        agent: &mut Agent,
        generator: ObjectRef,
        state: GeneratorState,
        suspended: Option<SuspendedExecutionRef>,
    ) -> VmResult<()> {
        let updated = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let state_updated = objects.set_generator_state(generator, state);
            let suspended_updated =
                objects.set_generator_suspended(&mut mutator, generator, suspended);
            state_updated && suspended_updated
        });
        if updated {
            Ok(())
        } else {
            Err(VmError::Abrupt(errors::throw_type_error(agent)))
        }
    }

    fn complete_generator_object(
        &mut self,
        agent: &mut Agent,
        generator: ObjectRef,
    ) -> VmResult<()> {
        agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            if let Some(suspended) = objects.generator_suspended(mutator.view(), generator) {
                let _ = mutator.free_suspended_execution(suspended);
            }
            let _ = objects.set_generator_suspended(&mut mutator, generator, None);
            let _ = objects.set_generator_state(generator, GeneratorState::Completed);
        });
        Ok(())
    }

    fn generator_result_object(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        value: Value,
        done: bool,
    ) -> VmResult<Value> {
        iterator::create_iterator_result_object(agent, realm, value, done)
            .map(Value::from_object_ref)
            .map_err(VmError::Abrupt)
    }

    fn generator_completion_result(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        resume_kind: GeneratorResumeKind,
        value: Value,
    ) -> VmResult<Value> {
        match resume_kind {
            GeneratorResumeKind::Next => {
                self.generator_result_object(agent, realm, Value::undefined(), true)
            }
            GeneratorResumeKind::Return => self.generator_result_object(agent, realm, value, true),
            GeneratorResumeKind::Throw => Err(VmError::Abrupt(
                lyng_js_types::AbruptCompletion::throw(value),
            )),
        }
    }
}

fn encode_execution_context_kind(kind: ExecutionContextKind) -> u8 {
    match kind {
        ExecutionContextKind::Script => 0,
        ExecutionContextKind::Module => 1,
        ExecutionContextKind::Builtin => 2,
        ExecutionContextKind::Function => 3,
        ExecutionContextKind::Eval => 4,
        ExecutionContextKind::Job => 5,
    }
}

fn decode_execution_context_kind(raw: u8) -> Option<ExecutionContextKind> {
    match raw {
        0 => Some(ExecutionContextKind::Script),
        1 => Some(ExecutionContextKind::Module),
        2 => Some(ExecutionContextKind::Builtin),
        3 => Some(ExecutionContextKind::Function),
        4 => Some(ExecutionContextKind::Eval),
        5 => Some(ExecutionContextKind::Job),
        _ => None,
    }
}

fn encode_this_state_kind(this_state: ThisState) -> u8 {
    match this_state {
        ThisState::Lexical => THIS_STATE_LEXICAL_RAW,
        ThisState::Uninitialized => THIS_STATE_UNINITIALIZED_RAW,
        ThisState::Value(_) => THIS_STATE_VALUE_RAW,
    }
}

fn decode_this_state(kind: u8, value: Value) -> ThisState {
    match kind {
        THIS_STATE_LEXICAL_RAW => ThisState::Lexical,
        THIS_STATE_UNINITIALIZED_RAW => ThisState::Uninitialized,
        THIS_STATE_VALUE_RAW => ThisState::Value(value),
        _ => ThisState::Value(value),
    }
}

use super::bytecode_calls::PreparedBytecodeCall;
use super::*;
use crate::frame::GeneratorResumeKind;
use lyng_js_env::{
    PromiseCapabilityId, PromiseReactionHandler, PromiseReactionKind, PromiseReactionRecord,
    PromiseResolvingFunctionKind, PromiseResolvingFunctionRecord, RealmRecord,
};
use lyng_js_ops::{errors, object};
use lyng_js_types::{
    promise_reject_function_builtin, promise_resolve_function_builtin, AbruptCompletion,
    PropertyKey, SuspendedExecutionRef, Value,
};

impl Vm {
    pub(super) fn resume_suspended_execution_job(
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
        self.restore_suspended_execution(agent, suspended, resume_kind, argument)?;
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

        match result {
            Ok(_) | Err(VmError::AsyncSuspend) => Ok(()),
            Err(error) => Err(error),
        }
    }

    pub(super) fn instantiate_async_function_call(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        prepared: PreparedBytecodeCall,
        arguments: &[Value],
    ) -> VmResult<ObjectRef> {
        let capability = self.create_intrinsic_promise_capability(agent, prepared.realm)?;
        let promise = self.promise_capability_promise(agent, capability)?;
        let prior_frame_depth = self.frames.len();
        let prior_context_depth = agent.execution_contexts().len();
        let prior_register_len = self.register_stack.len();
        let register_base =
            u32::try_from(prior_register_len).expect("register stack length should fit u32");

        self.install_prepared_bytecode_call(
            agent,
            prepared,
            arguments,
            register_base,
            None,
            None,
            false,
        )?;
        self.async_frame_states
            .insert(register_base, AsyncFrameState { capability });
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
        let _ = self.async_frame_states.remove(&register_base);

        let realm_record = agent
            .realm(prepared.realm)
            .ok_or(VmError::MissingRootShape(prepared.realm))?;
        match result {
            Ok(value) => {
                self.settle_promise_capability(
                    agent,
                    host,
                    registry,
                    realm_record,
                    capability,
                    false,
                    value,
                )?;
            }
            Err(VmError::AsyncSuspend) => {}
            Err(VmError::Abrupt(AbruptCompletion::Throw(thrown))) => {
                self.settle_promise_capability(
                    agent,
                    host,
                    registry,
                    realm_record,
                    capability,
                    true,
                    thrown,
                )?;
            }
            Err(error) => return Err(error),
        }

        Ok(promise)
    }

    pub(super) fn await_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        register: u16,
    ) -> VmResult<()> {
        if frame.resume_active() {
            let resume_value = frame.resume_value();
            let resume_kind = frame.resume_kind();
            self.clear_active_resume();
            if resume_kind == GeneratorResumeKind::Throw {
                if self.transfer_to_exception_handler(agent, resume_value)? {
                    return Ok(());
                }
                return Err(VmError::Abrupt(AbruptCompletion::Throw(resume_value)));
            }
            self.write_register(frame, register, resume_value)?;
            self.advance_instruction()?;
            return Ok(());
        }

        let value = self.read_register(frame, register)?;
        let promise = self.promise_resolve_in_realm(agent, host, registry, frame.realm(), value)?;
        self.suspend_for_await_promise(agent, frame, promise)
    }

    pub(super) fn resume_async_function(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        realm: RealmRecord,
        suspended: SuspendedExecutionRef,
        rejected: bool,
        argument: Value,
    ) -> VmResult<()> {
        let capability = self
            .suspended_side_states
            .get(&suspended)
            .and_then(|state| state.async_frame_state)
            .map(|state| state.capability)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let resume_kind = if rejected {
            GeneratorResumeKind::Throw
        } else {
            GeneratorResumeKind::Next
        };
        self.restore_suspended_execution(agent, suspended, resume_kind, argument)?;
        let async_frame_base = self
            .frame()
            .map(|frame| frame.registers().base())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
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
        let _ = self.async_frame_states.remove(&async_frame_base);

        match result {
            Ok(value) => {
                self.settle_promise_capability(
                    agent, host, registry, realm, capability, false, value,
                )?;
            }
            Err(VmError::AsyncSuspend) => {}
            Err(VmError::Abrupt(AbruptCompletion::Throw(thrown))) => {
                self.settle_promise_capability(
                    agent, host, registry, realm, capability, true, thrown,
                )?;
            }
            Err(error) => return Err(error),
        }
        Ok(())
    }

    pub(super) fn promise_resolve_in_realm(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        realm: RealmRef,
        value: Value,
    ) -> VmResult<ObjectRef> {
        let promise_constructor = agent
            .realm(realm)
            .and_then(|record| record.intrinsics().promise())
            .ok_or(VmError::MissingRootShape(realm))?;
        if let Some(promise) = value
            .as_object_ref()
            .filter(|object| agent.promise_record(*object).is_some())
        {
            let constructor = object::ordinary_get(
                agent,
                promise,
                PropertyKey::from_atom(WellKnownAtom::constructor.id()),
            )
            .map_err(VmError::Abrupt)?;
            if constructor == Value::from_object_ref(promise_constructor) {
                return Ok(promise);
            }
        }

        let capability = self.create_intrinsic_promise_capability(agent, realm)?;
        let promise = self.promise_capability_promise(agent, capability)?;
        let realm_record = agent.realm(realm).ok_or(VmError::MissingRootShape(realm))?;
        self.settle_promise_capability(
            agent,
            host,
            registry,
            realm_record,
            capability,
            false,
            value,
        )?;
        Ok(promise)
    }

    pub(super) fn create_intrinsic_promise_capability(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
    ) -> VmResult<PromiseCapabilityId> {
        let promise_prototype = agent
            .realm(realm)
            .and_then(|record| record.intrinsics().promise_prototype())
            .ok_or(VmError::MissingRootShape(realm))?;
        let promise =
            self.allocate_ordinary_object_with_prototype(agent, realm, Some(promise_prototype))?;
        let _ = agent.alloc_promise(promise, realm);
        let capability = agent.alloc_promise_capability();
        let _ = agent.set_promise_capability_promise(capability, promise);
        let resolve = self.allocate_builtin_function_object(
            agent,
            realm,
            promise_resolve_function_builtin(),
        )?;
        let reject =
            self.allocate_builtin_function_object(agent, realm, promise_reject_function_builtin())?;
        let _ = agent.set_promise_capability_resolve(capability, resolve);
        let _ = agent.set_promise_capability_reject(capability, reject);
        let _ = agent.alloc_promise_resolving_function(
            resolve,
            PromiseResolvingFunctionRecord::new(PromiseResolvingFunctionKind::Resolve, capability),
        );
        let _ = agent.alloc_promise_resolving_function(
            reject,
            PromiseResolvingFunctionRecord::new(PromiseResolvingFunctionKind::Reject, capability),
        );
        Ok(capability)
    }

    pub(super) fn promise_capability_promise(
        &self,
        agent: &mut Agent,
        capability: PromiseCapabilityId,
    ) -> VmResult<ObjectRef> {
        agent
            .promise_capability(capability)
            .and_then(|record| record.promise())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))
    }

    pub(super) fn suspend_for_await_promise(
        &mut self,
        agent: &mut Agent,
        frame: FrameRecord,
        promise: ObjectRef,
    ) -> VmResult<()> {
        let suspended =
            self.snapshot_suspended_execution(agent, frame, frame.instruction_offset())?;
        let active = self
            .frames
            .pop()
            .expect("await suspension requires one active frame");
        debug_assert_eq!(active, frame);
        self.register_stack.truncate(
            usize::try_from(frame.registers().base()).expect("base should fit into usize"),
        );
        let _ = self.current_exception.take();
        let _ = agent.pop_execution_context();
        self.enqueue_await_resume(agent, promise, suspended)?;
        Err(VmError::AsyncSuspend)
    }

    fn enqueue_await_resume(
        &mut self,
        agent: &mut Agent,
        promise: ObjectRef,
        suspended: SuspendedExecutionRef,
    ) -> VmResult<()> {
        let record = agent
            .promise_record(promise)
            .cloned()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let _ = agent.set_promise_handled(promise, true);
        let fulfill_reaction = agent.alloc_promise_reaction(PromiseReactionRecord::new(
            PromiseReactionKind::Fulfill,
            PromiseReactionHandler::AsyncResume {
                suspended,
                reject: false,
            },
            None,
        ));
        let reject_reaction = agent.alloc_promise_reaction(PromiseReactionRecord::new(
            PromiseReactionKind::Reject,
            PromiseReactionHandler::AsyncResume {
                suspended,
                reject: true,
            },
            None,
        ));
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
                let realm = agent
                    .realm(record.realm())
                    .ok_or(VmError::MissingRootShape(record.realm()))?;
                self.enqueue_promise_reaction_job(agent, realm, fulfill_reaction, record.result());
            }
            lyng_js_env::PromiseState::Rejected => {
                let realm = agent
                    .realm(record.realm())
                    .ok_or(VmError::MissingRootShape(record.realm()))?;
                self.enqueue_promise_reaction_job(agent, realm, reject_reaction, record.result());
            }
        }
        Ok(())
    }
}

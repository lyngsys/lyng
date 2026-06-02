use super::{
    Agent, CallerContext, FrameRecord, HostHooks, NativeFunctionRegistry, ObjectRef, Value, Vm,
    VmError, VmResult,
};
use crate::vm::property_access::VmProxyBridge;
use lyng_ops::{errors, object, proxy};
use lyng_types::{CodeRef, EnvironmentRef, PropertyDescriptor, RealmRef};

impl Vm {
    fn callback_object(agent: &Agent, value: Value) -> Option<ObjectRef> {
        value
            .as_object_ref()
            .filter(|object| agent.objects().is_callable(*object))
    }

    pub(super) fn cleanup_internal_completion(
        &mut self,
        agent: &mut Agent,
        prior_frame_depth: usize,
        prior_register_len: usize,
    ) -> VmResult<()> {
        while self.frame_depth() > prior_frame_depth {
            let leaked = self.pop_current_frame();
            self.close_loop_iteration_frames(self.frame_depth());
            self.close_direct_eval_frames(self.frame_depth());
            self.for_in_states.clear_window(leaked.registers());
            self.iterator_states.clear_window(leaked.registers());
            self.captured_name_references
                .clear_window(leaked.registers());
            let _ = self.async_frame_states.remove(&leaked.registers().base());
            let _ = self
                .async_generator_frame_states
                .remove(&leaked.registers().base());
            let lexical_env = self.frame_header(Self::cfr_of(&leaked)).lexical_env();
            self.finalize_mapped_arguments(agent, lexical_env)?;
            self.release_frame_to_caller(Self::cfr_of(&leaked));
        }
        // Internal calls inherit the referrer rather than establishing one, so
        // no scope is pushed here. Unwind any referrer scopes established by
        // frames above `prior_frame_depth` (e.g. a re-entrant entry frame); no-op
        // only when none were pushed.
        self.unwind_referrer_scopes_to(prior_frame_depth);
        self.release_register_stack_to(prior_register_len);
        self.refresh_running_context(agent);
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn call_to_completion(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: CallerContext,
        callee_object: ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> VmResult<Value> {
        if let Some(bound) = Self::bound_function_record(agent, callee_object) {
            let mut combined_arguments = arguments.to_vec();
            Self::prepend_bound_arguments(agent, bound, &mut combined_arguments)?;
            return self.call_to_completion(
                agent,
                host,
                registry,
                caller,
                bound.target(),
                bound.this_value(),
                &combined_arguments,
            );
        }
        let caller_realm = caller.realm;
        Self::reject_class_constructor_call(agent, callee_object, caller_realm)?;
        if let Some(result) = self.call_builtin(
            agent,
            host,
            registry,
            caller,
            callee_object,
            this_value,
            arguments,
            None,
        )? {
            return Ok(result);
        }
        if agent.objects().is_proxy_object(callee_object) {
            return proxy::call(
                &mut VmProxyBridge {
                    vm: self,
                    agent,
                    host,
                    registry,
                    caller_realm,
                    caller_lexical_env: caller.lexical_env,
                    caller_code: caller.code,
                    caller_pc: caller.pc,
                },
                callee_object,
                this_value,
                arguments,
            );
        }
        if Self::bytecode_entry(agent, callee_object).is_none() {
            return object::call(agent, callee_object, this_value, arguments, registry)
                .map_err(VmError::Abrupt);
        }

        let prior_frame_depth = self.frame_depth();
        let prior_register_len = self.register_stack_top();
        let prepared =
            self.prepare_bytecode_call(agent, caller.lexical_env, callee_object, this_value, None)?;
        if self
            .installed_function(prepared.code)
            .is_some_and(|function| function.flags().generator())
        {
            let generator =
                self.instantiate_generator_call(agent, host, registry, prepared, arguments)?;
            return Ok(Value::from_object_ref(generator));
        }
        if self
            .installed_function(prepared.code)
            .is_some_and(|function| function.flags().async_function())
        {
            let promise =
                self.instantiate_async_function_call(agent, host, registry, prepared, arguments)?;
            return Ok(Value::from_object_ref(promise));
        }
        self.install_prepared_bytecode_call(agent, prepared, arguments, None, None, false)?;
        self.internal_completion_targets.push(prior_frame_depth);

        let result = self.run(agent, host, registry);
        if self.internal_completion_targets.last().copied() == Some(prior_frame_depth) {
            let _ = self.internal_completion_targets.pop();
        }
        self.cleanup_internal_completion(agent, prior_frame_depth, prior_register_len)?;

        result
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn construct_to_completion(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: CallerContext,
        callee_object: ObjectRef,
        arguments: &[Value],
        new_target: Option<ObjectRef>,
    ) -> VmResult<ObjectRef> {
        let mut callee = callee_object;
        let mut effective_new_target = new_target.unwrap_or(callee_object);
        let mut combined_arguments = arguments.to_vec();
        Self::resolve_bound_construct_chain(
            agent,
            &mut callee,
            &mut effective_new_target,
            &mut combined_arguments,
        )?;
        let caller_realm = caller.realm;
        if agent.objects().is_proxy_object(callee) {
            return proxy::construct(
                &mut VmProxyBridge {
                    vm: self,
                    agent,
                    host,
                    registry,
                    caller_realm,
                    caller_lexical_env: caller.lexical_env,
                    caller_code: caller.code,
                    caller_pc: caller.pc,
                },
                callee,
                &combined_arguments,
                Some(effective_new_target),
            );
        }
        if !agent.objects().is_constructor(callee) {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        if Self::bytecode_entry(agent, callee).is_none() {
            if let Some(result) = self.call_builtin(
                agent,
                host,
                registry,
                caller,
                callee,
                Value::undefined(),
                &combined_arguments,
                Some(effective_new_target),
            )? {
                return result
                    .as_object_ref()
                    .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return object::construct(
                agent,
                callee,
                &combined_arguments,
                Some(effective_new_target),
                registry,
            )
            .map_err(VmError::Abrupt);
        }

        let prior_frame_depth = self.frame_depth();
        let prior_register_len = self.register_stack_top();
        let derived_construct = Self::bytecode_entry(agent, callee)
            .and_then(|code| self.installed_function(code))
            .is_some_and(|function| function.flags().derived_class_constructor());
        let construct_this = if derived_construct {
            None
        } else {
            Some(self.create_construct_this(
                agent,
                host,
                registry,
                caller,
                caller_realm,
                effective_new_target,
            )?)
        };
        let prepared = self.prepare_bytecode_call(
            agent,
            caller.lexical_env,
            callee,
            construct_this.map_or(Value::undefined(), Value::from_object_ref),
            Some(effective_new_target),
        )?;
        self.install_prepared_bytecode_call(
            agent,
            prepared,
            &combined_arguments,
            None,
            construct_this,
            true,
        )?;
        self.internal_completion_targets.push(prior_frame_depth);

        let result = self.run(agent, host, registry);
        if self.internal_completion_targets.last().copied() == Some(prior_frame_depth) {
            let _ = self.internal_completion_targets.pop();
        }
        self.cleanup_internal_completion(agent, prior_frame_depth, prior_register_len)?;

        result?
            .as_object_ref()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn call_optional_callback(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: CallerContext,
        callback: Value,
        this_value: Value,
        arguments: &[Value],
    ) -> VmResult<Option<Value>> {
        if callback == Value::undefined() {
            return Ok(None);
        }
        let callback = Self::require_callable_object(agent, callback)?;
        self.call_to_completion(
            agent, host, registry, caller, callback, this_value, arguments,
        )
        .map(Some)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn call_if_callable_object(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: CallerContext,
        callback: Value,
        this_value: Value,
        arguments: &[Value],
    ) -> VmResult<Option<Value>> {
        let Some(callback) = Self::callback_object(agent, callback) else {
            return Ok(None);
        };
        self.call_to_completion(
            agent, host, registry, caller, callback, this_value, arguments,
        )
        .map(Some)
    }

    pub(super) fn call_property_getter(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_realm: RealmRef,
        caller_lexical_env: EnvironmentRef,
        caller_code: CodeRef,
        caller_pc: u32,
        descriptor: PropertyDescriptor,
        receiver: Value,
    ) -> VmResult<Option<Value>> {
        let getter = descriptor.getter().unwrap_or(Value::undefined());
        if getter == Value::undefined() {
            return Ok(None);
        }
        let getter_object = Self::require_callable_object(agent, getter)?;
        let caller = CallerContext {
            realm: caller_realm,
            lexical_env: caller_lexical_env,
            code: caller_code,
            pc: caller_pc,
        };
        self.call_to_completion(agent, host, registry, caller, getter_object, receiver, &[])
            .map(Some)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn call_property_setter(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_realm: RealmRef,
        caller_lexical_env: EnvironmentRef,
        caller_code: CodeRef,
        caller_pc: u32,
        descriptor: PropertyDescriptor,
        receiver: Value,
        value: Value,
    ) -> VmResult<bool> {
        let setter = descriptor.setter().unwrap_or(Value::undefined());
        if setter == Value::undefined() {
            return Ok(false);
        }
        let setter_object = Self::require_callable_object(agent, setter)?;
        let arguments = [value];
        let caller = CallerContext {
            realm: caller_realm,
            lexical_env: caller_lexical_env,
            code: caller_code,
            pc: caller_pc,
        };
        self.call_to_completion(
            agent,
            host,
            registry,
            caller,
            setter_object,
            receiver,
            &arguments,
        )
        .map(|_| true)
    }

    /// Build a minimal `FrameRecord` from a [`CallerContext`] for the embedding
    /// `force_collect` GC-root path (the embedding function context no longer
    /// holds a `&FrameRecord`). The synthetic frame carries no registers (window
    /// `[0, 0)`), no callee (`None`), and the caller's `lexical_env`/`code`.
    ///
    /// Heap-edge-equivalent to the former caller-frame trace: for a live caller
    /// the arena cfr-walk already covers the real roots (this is redundant
    /// over-tracing); for a synthetic caller it reproduces exactly the
    /// `lexical_env`/`code` edges (the realm is not traced off the record). See
    /// `ActiveVmRoots::trace_heap_edges`.
    pub(crate) const fn synthetic_caller_frame(caller: CallerContext) -> FrameRecord {
        use crate::frame::RegisterWindow;
        use lyng_env::ExecutionContextKind;
        FrameRecord::new(
            caller.code,
            0,
            RegisterWindow::new(0, 0),
            None,
            caller.lexical_env,
            caller.lexical_env,
            ExecutionContextKind::Function,
        )
    }
}

use super::*;
use crate::vm::property_access::VmProxyBridge;
use lyng_js_ops::{errors, object, proxy};
use lyng_js_types::PropertyDescriptor;

impl Vm {
    fn callback_object(&self, agent: &Agent, value: Value) -> Option<ObjectRef> {
        value
            .as_object_ref()
            .filter(|object| agent.objects().is_callable(*object))
    }

    pub(super) fn cleanup_internal_completion(
        &mut self,
        agent: &mut Agent,
        prior_frame_depth: usize,
        prior_context_depth: usize,
        prior_register_len: usize,
    ) -> VmResult<()> {
        while self.frames.len() > prior_frame_depth {
            let leaked = self
                .frames
                .pop()
                .expect("frame count should be greater than baseline");
            self.close_loop_iteration_frames(self.frames.len());
            self.close_direct_eval_frames(self.frames.len());
            self.for_in_states.clear_window(leaked.registers());
            self.iterator_states.clear_window(leaked.registers());
            self.captured_name_references
                .clear_window(leaked.registers());
            let _ = self.async_frame_states.remove(&leaked.registers().base());
            let _ = self
                .async_generator_frame_states
                .remove(&leaked.registers().base());
            self.finalize_mapped_arguments(agent, leaked.lexical_env())?;
            self.register_stack.truncate(
                usize::try_from(leaked.registers().base()).expect("base should fit usize"),
            );
        }
        self.register_stack.truncate(prior_register_len);
        while agent.execution_contexts().len() > prior_context_depth {
            let _ = agent.pop_execution_context();
        }
        Ok(())
    }

    pub(super) fn call_to_completion(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
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
                caller_frame,
                bound.target(),
                bound.this_value(),
                &combined_arguments,
            );
        }
        Self::reject_class_constructor_call(agent, callee_object)?;
        if let Some(result) = self.call_builtin(
            agent,
            host,
            registry,
            caller_frame,
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
                    frame: caller_frame,
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

        let prior_frame_depth = self.frames.len();
        let prior_context_depth = agent.execution_contexts().len();
        let prior_register_len = self.register_stack.len();
        let prepared =
            self.prepare_bytecode_call(agent, caller_frame, callee_object, this_value, None)?;
        let register_base =
            u32::try_from(prior_register_len).expect("register stack length should fit u32");
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
        self.install_prepared_bytecode_call(
            agent,
            prepared,
            arguments,
            register_base,
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

        result
    }

    pub(super) fn construct_to_completion(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        callee_object: ObjectRef,
        arguments: &[Value],
        new_target: Option<ObjectRef>,
    ) -> VmResult<ObjectRef> {
        let mut callee = callee_object;
        let mut effective_new_target = new_target.unwrap_or(callee_object);
        let mut combined_arguments = arguments.to_vec();
        self.resolve_bound_construct_chain(
            agent,
            &mut callee,
            &mut effective_new_target,
            &mut combined_arguments,
        )?;
        if agent.objects().is_proxy_object(callee) {
            return proxy::construct(
                &mut VmProxyBridge {
                    vm: self,
                    agent,
                    host,
                    registry,
                    frame: caller_frame,
                },
                callee,
                &combined_arguments,
                Some(effective_new_target),
            );
        }
        if Self::bytecode_entry(agent, callee).is_none() {
            if Self::builtin_entry(agent, callee).is_some()
                && !agent.objects().is_constructor(callee)
            {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            if let Some(result) = self.call_builtin(
                agent,
                host,
                registry,
                caller_frame,
                callee,
                Value::undefined(),
                &combined_arguments,
                Some(effective_new_target),
            )? {
                return result
                    .as_object_ref()
                    .ok_or(VmError::Abrupt(errors::throw_type_error(agent)));
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

        let prior_frame_depth = self.frames.len();
        let prior_context_depth = agent.execution_contexts().len();
        let prior_register_len = self.register_stack.len();
        let derived_construct = Self::bytecode_entry(agent, callee)
            .and_then(|code| self.installed_function(code))
            .map(|function| function.flags().derived_class_constructor())
            .unwrap_or(false);
        let construct_this = if derived_construct {
            None
        } else {
            Some(Self::create_construct_this(
                agent,
                caller_frame.realm(),
                effective_new_target,
            )?)
        };
        let prepared = self.prepare_bytecode_call(
            agent,
            caller_frame,
            callee,
            construct_this.map_or(Value::undefined(), Value::from_object_ref),
            Some(effective_new_target),
        )?;
        let register_base =
            u32::try_from(prior_register_len).expect("register stack length should fit u32");
        self.install_prepared_bytecode_call(
            agent,
            prepared,
            &combined_arguments,
            register_base,
            None,
            construct_this,
            true,
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

        result?
            .as_object_ref()
            .ok_or(VmError::Abrupt(errors::throw_type_error(agent)))
    }

    pub(super) fn call_optional_callback(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        callback: Value,
        this_value: Value,
        arguments: &[Value],
    ) -> VmResult<Option<Value>> {
        if callback == Value::undefined() {
            return Ok(None);
        }
        let callback = Self::require_callable_object(agent, caller_frame, callback)?;
        self.call_to_completion(
            agent,
            host,
            registry,
            caller_frame,
            callback,
            this_value,
            arguments,
        )
        .map(Some)
    }

    pub(super) fn call_if_callable_object(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        callback: Value,
        this_value: Value,
        arguments: &[Value],
    ) -> VmResult<Option<Value>> {
        let Some(callback) = self.callback_object(agent, callback) else {
            return Ok(None);
        };
        self.call_to_completion(
            agent,
            host,
            registry,
            caller_frame,
            callback,
            this_value,
            arguments,
        )
        .map(Some)
    }

    pub(super) fn call_property_getter(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        descriptor: PropertyDescriptor,
        receiver: Value,
    ) -> VmResult<Option<Value>> {
        let getter = descriptor.getter().unwrap_or(Value::undefined());
        self.call_optional_callback(agent, host, registry, caller_frame, getter, receiver, &[])
    }

    pub(super) fn call_property_setter(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        descriptor: PropertyDescriptor,
        receiver: Value,
        value: Value,
    ) -> VmResult<bool> {
        let setter = descriptor.setter().unwrap_or(Value::undefined());
        let arguments = [value];
        self.call_optional_callback(
            agent,
            host,
            registry,
            caller_frame,
            setter,
            receiver,
            &arguments,
        )
        .map(|result| result.is_some())
    }
}

use super::*;

impl Vm {
    pub(in crate::vm::builtin_dispatch) fn super_property_get_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let receiver = arguments.first().copied().unwrap_or(Value::undefined());
        let home_object = arguments
            .get(2)
            .and_then(|value| value.as_object_ref())
            .map(Ok)
            .unwrap_or_else(|| {
                Self::resolve_super_home_object(agent, caller.lexical_env(), caller)
            })?;
        let base = object::super_base(agent, home_object).map_err(VmError::Abrupt)?;
        let key_value = arguments.get(1).copied().unwrap_or(Value::undefined());
        let key = self.to_property_key_from_value(agent, host, registry, caller, key_value)?;
        self.get_property_from_object(agent, host, registry, caller, base, receiver, key)
    }

    pub(in crate::vm::builtin_dispatch) fn super_property_set_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let receiver = arguments.first().copied().unwrap_or(Value::undefined());
        let value = arguments.get(2).copied().unwrap_or(Value::undefined());
        let home_object = arguments
            .get(3)
            .and_then(|value| value.as_object_ref())
            .map(Ok)
            .unwrap_or_else(|| {
                Self::resolve_super_home_object(agent, caller.lexical_env(), caller)
            })?;
        let base = object::super_base(agent, home_object).map_err(VmError::Abrupt)?;
        let key_value = arguments.get(1).copied().unwrap_or(Value::undefined());
        let key = self.to_property_key_from_value(agent, host, registry, caller, key_value)?;
        let updated =
            self.set_property_on_object(agent, host, registry, caller, base, receiver, key, value)?;
        if !updated && self.caller_is_strict(caller) {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(value)
    }

    pub(in crate::vm::builtin_dispatch) fn construct_super_with_arguments(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let record = Self::this_environment_record(agent, caller.lexical_env())?;
        let function_env = record.map(|record| record.declarative().id());
        let active_function = record
            .map(|record| record.function_object())
            .or_else(|| caller.callee())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let derived_constructor = {
            agent
                .objects()
                .function_data(active_function)
                .and_then(|data| match data.entry() {
                    Some(FunctionEntryIdentity::Bytecode(code)) => Some(code),
                    _ => None,
                })
                .and_then(|code| self.installed_function(code))
                .map(|function| function.flags().derived_class_constructor())
                .unwrap_or(false)
        } || caller
            .flags()
            .contains(crate::FrameFlags::derived_construct());
        if !derived_constructor {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        let binding_status = record.map_or_else(
            || {
                if caller.construct_this().is_some()
                    || agent
                        .current_execution_context()
                        .is_some_and(|context| context.this_state() != ThisState::Uninitialized)
                {
                    lyng_js_env::ThisBindingStatus::Initialized
                } else {
                    lyng_js_env::ThisBindingStatus::Uninitialized
                }
            },
            |record| record.this_binding_status(),
        );
        let super_constructor = object::ordinary_get_prototype_of(agent, active_function)
            .map_err(VmError::Abrupt)?
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let new_target = record
            .and_then(|record| record.new_target())
            .or_else(|| caller.new_target())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let this_object = self.construct_to_completion(
            agent,
            host,
            registry,
            caller,
            super_constructor,
            arguments,
            Some(new_target),
        )?;
        let this_value = Value::from_object_ref(this_object);
        if binding_status != lyng_js_env::ThisBindingStatus::Uninitialized {
            return Err(VmError::Abrupt(errors::throw_reference_error(agent)));
        }
        if let Some(function_env) = function_env {
            let _ = agent.set_function_this_binding(
                function_env,
                lyng_js_env::ThisBindingStatus::Initialized,
                this_value,
            );
            if !agent.set_execution_context_this_state_for_lexical_env(
                function_env,
                ThisState::Value(this_value),
            ) {
                let _ =
                    agent.set_current_execution_context_this_state(ThisState::Value(this_value));
            }
        } else {
            let _ = agent.set_current_execution_context_this_state(ThisState::Value(this_value));
        }
        let frame_index = self
            .frames
            .iter()
            .rposition(|frame| frame.callee() == Some(active_function))
            .or_else(|| {
                function_env.and_then(|function_env| {
                    self.frames.iter().rposition(|frame| {
                        frame.lexical_env() == function_env || frame.variable_env() == function_env
                    })
                })
            })
            .or_else(|| {
                self.frames.iter().rposition(|frame| {
                    frame.code() == caller.code()
                        && frame.registers() == caller.registers()
                        && frame.callee() == caller.callee()
                })
            })
            .or_else(|| self.frames.len().checked_sub(1));
        let Some(frame_index) = frame_index else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        let Some(frame) = self.frames.get_mut(frame_index) else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        frame.set_this_value(this_value);
        frame.set_construct_this(Some(this_object));
        Ok(this_value)
    }

    pub(in crate::vm::builtin_dispatch) fn construct_super_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        self.construct_super_with_arguments(agent, host, registry, caller, arguments)
    }

    pub(in crate::vm::builtin_dispatch) fn construct_super_spread_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let spread_source = arguments.first().copied().unwrap_or(Value::undefined());
        let mut spread_arguments = Vec::new();
        self.append_spread_argument(
            agent,
            host,
            registry,
            caller,
            spread_source,
            &mut spread_arguments,
        )?;
        self.construct_super_with_arguments(agent, host, registry, caller, &spread_arguments)
    }
}

use super::{
    errors, object, Agent, FrameRecord, FunctionEntryIdentity, FunctionThisMode, HostHooks,
    NativeFunctionRegistry, ObjectRef, ThisBindingStatus, ThisState, Value, Vm, VmError, VmResult,
};

#[derive(Clone, Copy, Debug)]
struct SuperConstructContext {
    function_env: Option<lyng_js_types::EnvironmentRef>,
    active_function: ObjectRef,
    binding_status: ThisBindingStatus,
    new_target: ObjectRef,
}

impl Vm {
    fn super_constructor_this_environment_record(
        agent: &Agent,
        start: lyng_js_types::EnvironmentRef,
    ) -> VmResult<Option<lyng_js_env::FunctionEnvironmentRecord>> {
        let mut current = Some(start);
        while let Some(environment) = current {
            match agent
                .environment(environment)
                .ok_or(VmError::MissingEnvironment(environment))?
            {
                lyng_js_env::EnvironmentRecord::Function(record) => {
                    let function_is_lexical = agent
                        .objects()
                        .function_data(record.function_object())
                        .is_some_and(|data| data.this_mode() == FunctionThisMode::Lexical);
                    if record.this_binding_status() == ThisBindingStatus::Lexical
                        || function_is_lexical
                    {
                        current = record.declarative().outer();
                        continue;
                    }
                    return Ok(Some(record));
                }
                lyng_js_env::EnvironmentRecord::Declarative(record) => current = record.outer(),
                lyng_js_env::EnvironmentRecord::Private(record) => current = record.outer(),
                lyng_js_env::EnvironmentRecord::Module(record) => current = record.outer(),
                lyng_js_env::EnvironmentRecord::Global(record) => current = record.outer(),
                lyng_js_env::EnvironmentRecord::Object(record) => current = record.outer(),
            }
        }
        Ok(None)
    }

    pub(in crate::vm::builtin_dispatch) fn super_property_get_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let receiver = arguments.first().copied().unwrap_or(Value::undefined());
        let base = if arguments.get(3).and_then(|value| value.as_bool()) == Some(true) {
            let base_value = arguments.get(2).copied().unwrap_or(Value::undefined());
            self.to_object_for_value(agent, caller.realm(), base_value)?
        } else {
            let home_object = arguments
                .get(2)
                .and_then(|value| value.as_object_ref())
                .map_or_else(
                    || Self::resolve_super_home_object(agent, caller.lexical_env(), caller),
                    Ok,
                )?;
            object::super_base(agent, home_object).map_err(VmError::Abrupt)?
        };
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
        let base = if arguments.get(4).and_then(|value| value.as_bool()) == Some(true) {
            let base_value = arguments.get(3).copied().unwrap_or(Value::undefined());
            self.to_object_for_value(agent, caller.realm(), base_value)?
        } else {
            let home_object = arguments
                .get(3)
                .and_then(|value| value.as_object_ref())
                .map_or_else(
                    || Self::resolve_super_home_object(agent, caller.lexical_env(), caller),
                    Ok,
                )?;
            object::super_base(agent, home_object).map_err(VmError::Abrupt)?
        };
        let key_value = arguments.get(1).copied().unwrap_or(Value::undefined());
        let key = self.to_property_key_from_value(agent, host, registry, caller, key_value)?;
        let updated =
            self.set_property_on_object(agent, host, registry, caller, base, receiver, key, value)?;
        if !updated && self.caller_is_strict(caller) {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(value)
    }

    pub(in crate::vm::builtin_dispatch) fn super_base_builtin(
        agent: &mut Agent,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let home_object = arguments
            .first()
            .and_then(|value| value.as_object_ref())
            .map_or_else(
                || Self::resolve_super_home_object(agent, caller.lexical_env(), caller),
                Ok,
            )?;
        let base = object::ordinary_get_prototype_of(agent, home_object)
            .map_err(VmError::Abrupt)?
            .map_or_else(Value::null, Value::from_object_ref);
        Ok(base)
    }

    pub(in crate::vm::builtin_dispatch) fn super_constructor_builtin(
        &self,
        agent: &mut Agent,
        caller: FrameRecord,
    ) -> VmResult<Value> {
        let context = self.super_construct_context(agent, caller)?;
        let super_constructor = object::ordinary_get_prototype_of(agent, context.active_function)
            .map_err(VmError::Abrupt)?
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        Ok(Value::from_object_ref(super_constructor))
    }

    fn super_construct_context(
        &self,
        agent: &mut Agent,
        caller: FrameRecord,
    ) -> VmResult<SuperConstructContext> {
        let record = Self::super_constructor_this_environment_record(agent, caller.lexical_env())?;
        let function_env = record.map(|record| record.declarative().id());
        let active_function = record
            .map(lyng_js_env::FunctionEnvironmentRecord::function_object)
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
                .is_some_and(|function| function.flags().derived_class_constructor())
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
            lyng_js_env::FunctionEnvironmentRecord::this_binding_status,
        );
        let new_target = record
            .and_then(lyng_js_env::FunctionEnvironmentRecord::new_target)
            .or_else(|| caller.new_target())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        Ok(SuperConstructContext {
            function_env,
            active_function,
            binding_status,
            new_target,
        })
    }

    fn construct_super_with_constructor(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        super_constructor: ObjectRef,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let context = self.super_construct_context(agent, caller)?;
        let this_object = self.construct_to_completion(
            agent,
            host,
            registry,
            caller,
            super_constructor,
            arguments,
            Some(context.new_target),
        )?;
        let this_value = Value::from_object_ref(this_object);
        if context.binding_status != lyng_js_env::ThisBindingStatus::Uninitialized {
            return Err(VmError::Abrupt(errors::throw_reference_error(agent)));
        }
        if let Some(function_env) = context.function_env {
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
            .rposition(|frame| frame.callee() == Some(context.active_function))
            .or_else(|| {
                context.function_env.and_then(|function_env| {
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

    pub(in crate::vm::builtin_dispatch) fn construct_super_with_arguments(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let context = self.super_construct_context(agent, caller)?;
        let super_constructor = object::ordinary_get_prototype_of(agent, context.active_function)
            .map_err(VmError::Abrupt)?
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        self.construct_super_with_constructor(
            agent,
            host,
            registry,
            caller,
            super_constructor,
            arguments,
        )
    }

    pub(in crate::vm::builtin_dispatch) fn construct_super_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let Some(super_constructor_value) = arguments.first().copied() else {
            return self.construct_super_with_arguments(agent, host, registry, caller, arguments);
        };
        let super_constructor = super_constructor_value
            .as_object_ref()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        self.construct_super_with_constructor(
            agent,
            host,
            registry,
            caller,
            super_constructor,
            &arguments[1..],
        )
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

    pub(in crate::vm::builtin_dispatch) fn construct_super_array_like_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let array_like = arguments.first().copied().unwrap_or(Value::undefined());
        let super_arguments = self.collect_array_like_arguments(
            agent,
            host,
            registry,
            caller,
            caller.realm(),
            array_like,
        )?;
        self.construct_super_with_arguments(agent, host, registry, caller, &super_arguments)
    }
}

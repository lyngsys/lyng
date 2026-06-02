use super::{
    Agent, FunctionEntryIdentity, FunctionThisMode, HostHooks, NativeFunctionRegistry, ObjectRef,
    ThisBindingStatus, ThisState, Value, Vm, VmError, VmResult, errors, object,
};
use crate::frame::{CallerContext, FrameView};

#[derive(Clone, Copy, Debug)]
struct SuperConstructContext {
    function_env: Option<lyng_types::EnvironmentRef>,
    active_function: ObjectRef,
    binding_status: ThisBindingStatus,
    new_target: ObjectRef,
}

impl Vm {
    fn super_constructor_this_environment_record(
        agent: &Agent,
        start: lyng_types::EnvironmentRef,
    ) -> VmResult<Option<lyng_env::FunctionEnvironmentRecord>> {
        let mut current = Some(start);
        while let Some(environment) = current {
            match agent
                .environment(environment)
                .ok_or(VmError::MissingEnvironment(environment))?
            {
                lyng_env::EnvironmentRecord::Function(record) => {
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
                lyng_env::EnvironmentRecord::Declarative(record) => current = record.outer(),
                lyng_env::EnvironmentRecord::Private(record) => current = record.outer(),
                lyng_env::EnvironmentRecord::Module(record) => current = record.outer(),
                lyng_env::EnvironmentRecord::Global(record) => current = record.outer(),
                lyng_env::EnvironmentRecord::Object(record) => current = record.outer(),
            }
        }
        Ok(None)
    }

    pub(in crate::vm::builtin_dispatch) fn super_property_get_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameView,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let receiver = arguments.first().copied().unwrap_or(Value::undefined());
        let caller_realm = self.realm_of(agent, caller.cfr());
        let caller_lexical_env = self.frame_header(caller.cfr()).lexical_env();
        let caller_callee = self.frame_header(caller.cfr()).callee();
        let base = if arguments.get(3).and_then(|value| value.as_bool()) == Some(true) {
            let base_value = arguments.get(2).copied().unwrap_or(Value::undefined());
            Self::to_object_for_value(agent, caller_realm, base_value)?
        } else {
            let home_object = arguments
                .get(2)
                .and_then(|value| value.as_object_ref())
                .map_or_else(
                    || Self::resolve_super_home_object(agent, caller_lexical_env, caller_callee),
                    Ok,
                )?;
            object::super_base(agent, home_object).map_err(VmError::Abrupt)?
        };
        let key_value = arguments.get(1).copied().unwrap_or(Value::undefined());
        let key = self.property_key_from_value(
            agent,
            host,
            registry,
            caller_realm,
            caller_lexical_env,
            caller.code(),
            caller.instruction_offset(),
            key_value,
        )?;
        self.get_property_from_object(
            agent,
            host,
            registry,
            caller_realm,
            caller_lexical_env,
            caller.code(),
            caller.instruction_offset(),
            base,
            receiver,
            key,
        )
    }

    pub(in crate::vm::builtin_dispatch) fn super_property_set_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameView,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let receiver = arguments.first().copied().unwrap_or(Value::undefined());
        let value = arguments.get(2).copied().unwrap_or(Value::undefined());
        let caller_realm = self.realm_of(agent, caller.cfr());
        let caller_lexical_env = self.frame_header(caller.cfr()).lexical_env();
        let caller_callee = self.frame_header(caller.cfr()).callee();
        let base = if arguments.get(4).and_then(|value| value.as_bool()) == Some(true) {
            let base_value = arguments.get(3).copied().unwrap_or(Value::undefined());
            Self::to_object_for_value(agent, caller_realm, base_value)?
        } else {
            let home_object = arguments
                .get(3)
                .and_then(|value| value.as_object_ref())
                .map_or_else(
                    || Self::resolve_super_home_object(agent, caller_lexical_env, caller_callee),
                    Ok,
                )?;
            object::super_base(agent, home_object).map_err(VmError::Abrupt)?
        };
        let key_value = arguments.get(1).copied().unwrap_or(Value::undefined());
        let key = self.property_key_from_value(
            agent,
            host,
            registry,
            caller_realm,
            caller_lexical_env,
            caller.code(),
            caller.instruction_offset(),
            key_value,
        )?;
        let updated = self.set_property_on_object(
            agent,
            host,
            registry,
            caller_realm,
            caller_lexical_env,
            caller.code(),
            caller.instruction_offset(),
            base,
            receiver,
            key,
            value,
        )?;
        if !updated && self.caller_is_strict(caller.code()) {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(value)
    }

    pub(in crate::vm::builtin_dispatch) fn super_base_builtin(
        &self,
        agent: &mut Agent,
        caller: FrameView,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let caller_lexical_env = self.frame_header(caller.cfr()).lexical_env();
        let caller_callee = self.frame_header(caller.cfr()).callee();
        let home_object = arguments
            .first()
            .and_then(|value| value.as_object_ref())
            .map_or_else(
                || Self::resolve_super_home_object(agent, caller_lexical_env, caller_callee),
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
        caller: FrameView,
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
        caller: FrameView,
    ) -> VmResult<SuperConstructContext> {
        let cfr = caller.cfr();
        let header = self.frame_header(cfr);
        let caller_lexical_env = header.lexical_env();
        let record = Self::super_constructor_this_environment_record(agent, caller_lexical_env)?;
        let function_env = record.map(|record| record.declarative().id());
        let active_function = record
            .map(lyng_env::FunctionEnvironmentRecord::function_object)
            .or_else(|| self.frame_header(cfr).callee())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let derived_constructor =
            {
                agent
                    .objects()
                    .function_data(active_function)
                    .and_then(|data| match data.entry() {
                        Some(FunctionEntryIdentity::Bytecode(code)) => Some(code),
                        _ => None,
                    })
                    .and_then(|code| self.installed_function(code))
                    .is_some_and(|function| function.flags().derived_class_constructor())
            } || crate::frame::FrameFlags::from_raw(self.frame_header(cfr).flags_bits())
                .contains(crate::FrameFlags::derived_construct());
        if !derived_constructor {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        let binding_status = record.map_or_else(
            || {
                if self.frame_header(cfr).construct_this().is_some()
                    || self.frame_header(cfr).this_state() != ThisState::Uninitialized
                {
                    lyng_env::ThisBindingStatus::Initialized
                } else {
                    lyng_env::ThisBindingStatus::Uninitialized
                }
            },
            lyng_env::FunctionEnvironmentRecord::this_binding_status,
        );
        let new_target = record
            .and_then(lyng_env::FunctionEnvironmentRecord::new_target)
            .or_else(|| self.frame_header(cfr).new_target())
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
        caller: FrameView,
        super_constructor: ObjectRef,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let context = self.super_construct_context(agent, caller)?;
        let caller_record = self.frame_record_for_view(caller);
        let caller_ctx = Self::caller_context_from_record(agent, &caller_record);
        let this_object = self.construct_to_completion(
            agent,
            host,
            registry,
            caller_ctx,
            super_constructor,
            arguments,
            Some(context.new_target),
        )?;
        let this_value = Value::from_object_ref(this_object);
        if context.binding_status != lyng_env::ThisBindingStatus::Uninitialized {
            return Err(VmError::Abrupt(errors::throw_reference_error(agent)));
        }
        if let Some(function_env) = context.function_env {
            let _ = agent.set_function_this_binding(
                function_env,
                lyng_env::ThisBindingStatus::Initialized,
                this_value,
            );
            // Whole-stack scan: update `this_state`/`this_value` on every frame
            // bound to `function_env`. The overlay is the source of truth for both
            // `this_state` and `lexical_env` now, so the scan walks cfrs and reads
            // the overlay `lexical_env`. (Collect cfrs first to avoid a borrow
            // conflict with the `&mut` overlay writes.)
            let matching_cfrs: Vec<u32> = self
                .frame_cfrs()
                .filter(|&cfr| self.frame_header(cfr).lexical_env() == function_env)
                .collect();
            if matching_cfrs.is_empty() {
                if let Some(cfr) = self.current_cfr_opt() {
                    self.frame_header_mut(cfr)
                        .set_this(ThisState::Value(this_value), this_value);
                }
            } else {
                for cfr in matching_cfrs {
                    self.frame_header_mut(cfr)
                        .set_this(ThisState::Value(this_value), this_value);
                }
            }
        } else if let Some(cfr) = self.current_cfr_opt() {
            self.frame_header_mut(cfr)
                .set_this(ThisState::Value(this_value), this_value);
        }
        // Find the target frame's cfr, innermost-first (matches the old
        // `rposition` over the deleted `frames` Vec). `callee`/`variable_env`/`code` are
        // immutable metadata (overlay == record); `lexical_env` is read from the
        // overlay (authoritative now). `registers().base()` derives from the cfr.
        let target_cfr = self
            .frame_cfrs()
            .find(|&cfr| self.frame_header(cfr).callee() == Some(context.active_function))
            .or_else(|| {
                context.function_env.and_then(|function_env| {
                    self.frame_cfrs().find(|&cfr| {
                        let header = self.frame_header(cfr);
                        header.lexical_env() == function_env
                            || header.variable_env() == function_env
                    })
                })
            })
            .or_else(|| {
                let caller_cfr = caller.cfr();
                self.frame_cfrs().find(|&cfr| {
                    let header = self.frame_header(cfr);
                    header.code() == caller.code()
                        && cfr == caller_cfr
                        && header.callee() == self.frame_header(caller_cfr).callee()
                })
            })
            .or_else(|| self.current_cfr_opt());
        let Some(target_cfr) = target_cfr else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        let header = self.frame_header_mut(target_cfr);
        header.set_construct_this(Some(this_object));
        header.set_this(ThisState::Value(this_value), this_value);
        Ok(this_value)
    }

    pub(in crate::vm::builtin_dispatch) fn construct_super_with_arguments(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameView,
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
        caller: FrameView,
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
        caller: FrameView,
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
        caller: FrameView,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let array_like = arguments.first().copied().unwrap_or(Value::undefined());
        let caller_realm = self.realm_of(agent, caller.cfr());
        let caller_ctx = CallerContext {
            realm: caller_realm,
            lexical_env: self.frame_header(caller.cfr()).lexical_env(),
            code: caller.code(),
            pc: caller.instruction_offset(),
        };
        let super_arguments = self.collect_array_like_arguments(
            agent,
            host,
            registry,
            caller_ctx,
            caller_realm,
            array_like,
        )?;
        self.construct_super_with_arguments(agent, host, registry, caller, &super_arguments)
    }
}

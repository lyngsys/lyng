use super::activation_objects::ActivationObjectInit;
use super::{
    Agent, AllocationLifetime, ArgumentsMode, CodeRef, EnvironmentRef, ExecutionContext,
    FrameFlags, FrameRecord, HostHooks, ObjectAllocation, ObjectRef, RealmRef, RegisterWindow,
    ThisBindingStatus, ThisState, Value, Vm, VmError, VmResult, WellKnownAtom,
};
use lyng_js_objects::{FunctionEntryIdentity, FunctionThisMode, NativeFunctionRegistry};
use lyng_js_ops::errors;
use lyng_js_types::PropertyKey;

impl Vm {
    pub(super) fn enter_bytecode_call(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        result_register: u16,
        callee_object: ObjectRef,
        this_value: Value,
        arguments: &[Value],
        new_target: Option<ObjectRef>,
        construct_call: bool,
    ) -> VmResult<()> {
        let prepared =
            self.prepare_bytecode_call(agent, caller_frame, callee_object, this_value, new_target)?;
        let register_base =
            u32::try_from(self.register_stack.len()).expect("register stack length should fit u32");
        if self
            .installed_function(prepared.code)
            .is_some_and(|function| function.flags().generator())
        {
            let generator =
                self.instantiate_generator_call(agent, host, registry, prepared, arguments)?;
            self.write_register(
                caller_frame,
                result_register,
                Value::from_object_ref(generator),
            )?;
            return Ok(());
        }
        if self
            .installed_function(prepared.code)
            .is_some_and(|function| function.flags().async_function())
        {
            let promise =
                self.instantiate_async_function_call(agent, host, registry, prepared, arguments)?;
            self.write_register(
                caller_frame,
                result_register,
                Value::from_object_ref(promise),
            )?;
            return Ok(());
        }
        let construct_this = construct_call
            .then_some(())
            .and_then(|_| this_value.as_object_ref());
        self.install_prepared_bytecode_call(
            agent,
            prepared,
            arguments,
            register_base,
            Some(result_register),
            construct_this,
            construct_call,
        )
    }

    pub(super) fn recycle_tail_bytecode_call(
        &mut self,
        agent: &mut Agent,
        caller_frame: FrameRecord,
        callee_object: ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> VmResult<()> {
        let tail_caller = caller_frame.callee();
        let tail_caller_strict = self.frame_is_strict(caller_frame);
        let prepared =
            self.prepare_bytecode_call(agent, caller_frame, callee_object, this_value, None)?;
        let register_base = caller_frame.registers().base();
        let construct_this = caller_frame.construct_this().or_else(|| {
            caller_frame
                .flags()
                .contains(FrameFlags::construct())
                .then_some(())
                .and_then(|_| caller_frame.this_value().as_object_ref())
        });
        self.teardown_tail_frame(agent, caller_frame)?;
        self.install_prepared_bytecode_call(
            agent,
            prepared,
            arguments,
            register_base,
            caller_frame.return_register(),
            construct_this,
            caller_frame.flags().contains(FrameFlags::construct()),
        )?;
        if let Some(frame) = self.frames.last_mut() {
            frame.set_tail_caller(tail_caller, tail_caller_strict);
        }
        Ok(())
    }

    pub(super) fn prepare_bytecode_call(
        &mut self,
        agent: &mut Agent,
        caller_frame: FrameRecord,
        callee_object: ObjectRef,
        this_value: Value,
        new_target: Option<ObjectRef>,
    ) -> VmResult<PreparedBytecodeCall> {
        let function_data = agent
            .objects()
            .function_data(callee_object)
            .cloned()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let FunctionEntryIdentity::Bytecode(code) = function_data
            .entry()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?
        else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        let (
            needs_environment,
            environment_layout,
            register_count,
            hidden_register_count,
            parameter_count,
            parameter_initializer_end_offset,
            arguments_mode,
            has_rest_parameter,
            derived_class_constructor,
        ) = {
            let function = self
                .installed_function(code)
                .ok_or(VmError::MissingInstalledCode(code))?;
            (
                function.needs_environment(),
                function.environment_layout(),
                function.register_count(),
                function.hidden_register_count(),
                function.parameter_count(),
                function.parameter_initializer_end_offset(),
                function.arguments_mode(),
                function.has_rest_parameter(),
                function.flags().derived_class_constructor(),
            )
        };

        let outer_environment = function_data
            .environment()
            .ok_or(VmError::MissingEnvironment(caller_frame.lexical_env()))?;
        let realm = function_data.realm().unwrap_or(caller_frame.realm());
        let derived_construct_call = new_target.is_some() && derived_class_constructor;
        let (effective_this, execution_this_state, env_this_status, effective_new_target) =
            match function_data.this_mode() {
                FunctionThisMode::Lexical => {
                    let (lexical_this, lexical_new_target) =
                        Self::lexical_call_state(agent, outer_environment, caller_frame)?;
                    (
                        lexical_this,
                        ThisState::Lexical,
                        ThisBindingStatus::Lexical,
                        lexical_new_target,
                    )
                }
                FunctionThisMode::Strict if derived_construct_call => (
                    Value::undefined(),
                    ThisState::Uninitialized,
                    ThisBindingStatus::Uninitialized,
                    new_target,
                ),
                FunctionThisMode::Strict => (
                    this_value,
                    ThisState::Value(this_value),
                    ThisBindingStatus::Initialized,
                    new_target,
                ),
                FunctionThisMode::Global => {
                    let resolved = self.resolve_global_this(agent, realm, this_value)?;
                    (
                        resolved,
                        ThisState::Value(resolved),
                        ThisBindingStatus::Initialized,
                        new_target,
                    )
                }
            };

        let (lexical_env, variable_env) = if needs_environment {
            let layout = environment_layout
                .and_then(|layout| lyng_js_env::EnvironmentLayoutId::from_raw(layout.get()))
                .ok_or(VmError::MissingEnvironmentLayout(code))?;
            let env = agent
                .alloc_function_environment(
                    Some(outer_environment),
                    layout,
                    callee_object,
                    env_this_status,
                    effective_this,
                    effective_new_target,
                    function_data.home_object(),
                    AllocationLifetime::Default,
                )
                .ok_or(VmError::MissingEnvironmentLayout(code))?;
            (env, env)
        } else {
            (outer_environment, outer_environment)
        };

        Ok(PreparedBytecodeCall {
            code,
            realm,
            lexical_env,
            variable_env,
            private_env: function_data.private_env(),
            this_value: effective_this,
            execution_this_state,
            new_target: effective_new_target,
            callee: callee_object,
            derived_class_constructor,
            parameter_count,
            parameter_initializer_end_offset,
            register_count,
            hidden_register_count,
            arguments_mode,
            has_rest_parameter,
        })
    }

    pub(super) fn install_prepared_bytecode_call(
        &mut self,
        agent: &mut Agent,
        prepared: PreparedBytecodeCall,
        arguments: &[Value],
        register_base: u32,
        return_register: Option<u16>,
        construct_this: Option<ObjectRef>,
        construct_call: bool,
    ) -> VmResult<()> {
        let register_len = prepared
            .register_count
            .checked_add(prepared.hidden_register_count)
            .expect("frame register span should fit within u16");
        self.reserve_register_window(register_base, register_len);
        self.copy_arguments_into_frame(register_base, prepared.parameter_count, arguments);

        let script_or_module_referrer = agent
            .current_execution_context()
            .and_then(|context| context.script_or_module_referrer());
        let context = ExecutionContext::bytecode(
            prepared.realm,
            prepared.code,
            prepared.lexical_env,
            prepared.variable_env,
        )
        .with_private_env(prepared.private_env)
        .with_this_state(prepared.execution_this_state)
        .with_script_or_module_referrer(script_or_module_referrer)
        .with_new_target(prepared.new_target);
        if let Err(error) = self.initialize_activation_objects(
            agent,
            ActivationObjectInit {
                realm: prepared.realm,
                parameter_count: prepared.parameter_count,
                arguments_mode: prepared.arguments_mode,
                has_rest_parameter: prepared.has_rest_parameter,
                lexical_env: prepared.lexical_env,
                arguments,
                callee: Value::from_object_ref(prepared.callee),
            },
        ) {
            self.register_stack.truncate(
                usize::try_from(register_base).expect("register base should fit into usize"),
            );
            return Err(error);
        }
        let frame = FrameRecord::new(
            prepared.code,
            0,
            RegisterWindow::new(register_base, register_len),
            return_register,
            prepared.realm,
            prepared.lexical_env,
            prepared.variable_env,
            context.kind(),
        )
        .with_this_value(prepared.this_value)
        .with_parameter_initializer_end_offset(prepared.parameter_initializer_end_offset)
        .with_construct_this(construct_this)
        .with_new_target(prepared.new_target)
        .with_callee(Some(prepared.callee))
        .with_flags(
            FrameFlags::entry()
                .with_flag(FrameFlags::suspendable(), true)
                .with_flag(FrameFlags::construct(), construct_call)
                .with_flag(
                    FrameFlags::derived_construct(),
                    construct_call && prepared.derived_class_constructor,
                ),
        );
        agent.push_execution_context(context);
        self.frames.push(frame);
        self.note_frame_depth();
        Ok(())
    }

    fn teardown_tail_frame(&mut self, agent: &mut Agent, frame: FrameRecord) -> VmResult<()> {
        let active = self
            .frames
            .pop()
            .expect("tail-call recycling requires one active frame");
        debug_assert_eq!(active, frame);
        self.close_loop_iteration_frames(self.frames.len());
        self.for_in_states.clear_window(frame.registers());
        self.iterator_states.clear_window(frame.registers());
        self.captured_name_references
            .clear_window(frame.registers());
        self.finalize_mapped_arguments(agent, frame.lexical_env())?;
        self.register_stack.truncate(
            usize::try_from(frame.registers().base()).expect("base should fit into usize"),
        );
        let _ = self.current_exception.take();
        let _ = agent.pop_execution_context();
        Ok(())
    }

    pub(super) fn copy_arguments_into_frame(
        &mut self,
        register_base: u32,
        parameter_count: u16,
        arguments: &[Value],
    ) {
        for index in 0..usize::from(parameter_count) {
            let absolute =
                usize::try_from(register_base).expect("register base should fit usize") + index;
            if let Some(slot) = self.register_stack.get_mut(absolute) {
                *slot = arguments.get(index).copied().unwrap_or(Value::undefined());
            }
        }
    }

    pub(super) fn bytecode_entry(agent: &Agent, callee_object: ObjectRef) -> Option<CodeRef> {
        let data = agent.objects().function_data(callee_object)?;
        match data.entry()? {
            FunctionEntryIdentity::Bytecode(code) => Some(code),
            FunctionEntryIdentity::Native(_) => None,
            FunctionEntryIdentity::Bound => None,
        }
    }

    pub(super) fn require_callable_object(
        agent: &mut Agent,
        _frame: FrameRecord,
        value: Value,
    ) -> VmResult<ObjectRef> {
        let object = value
            .as_object_ref()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        if !agent.objects().is_callable(object) {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(object)
    }

    pub(super) fn reject_class_constructor_call(
        agent: &mut Agent,
        callee: ObjectRef,
        fallback_realm: RealmRef,
    ) -> VmResult<()> {
        let Some(data) = agent.objects().function_data(callee) else {
            return Ok(());
        };
        if data.kind_flags().is_class_constructor() {
            let realm = data.realm().unwrap_or(fallback_realm);
            return Err(Self::abrupt_intrinsic_error(
                agent,
                realm,
                errors::ErrorKind::Type,
            ));
        }
        Ok(())
    }

    pub(super) fn lexical_call_state(
        agent: &Agent,
        start: EnvironmentRef,
        caller_frame: FrameRecord,
    ) -> VmResult<(Value, Option<ObjectRef>)> {
        if let Some(record) = Self::this_environment_record(agent, start)? {
            return Ok((record.this_value(), record.new_target()));
        }
        Ok((caller_frame.this_value(), caller_frame.new_target()))
    }

    pub(super) fn resolve_this_binding(
        agent: &mut Agent,
        start: EnvironmentRef,
        caller_frame: FrameRecord,
    ) -> VmResult<Value> {
        let Some(record) = Self::this_environment_record(agent, start)? else {
            return Ok(caller_frame.this_value());
        };
        match record.this_binding_status() {
            ThisBindingStatus::Initialized => Ok(record.this_value()),
            ThisBindingStatus::Uninitialized => {
                Err(VmError::Abrupt(errors::throw_reference_error(agent)))
            }
            ThisBindingStatus::Lexical => unreachable!("lexical this environments are skipped"),
        }
    }

    pub(super) fn resolve_super_home_object(
        agent: &mut Agent,
        start: EnvironmentRef,
        caller_frame: FrameRecord,
    ) -> VmResult<ObjectRef> {
        if let Some(record) = Self::this_environment_record(agent, start)? {
            if let Some(home_object) = record.home_object() {
                return Ok(home_object);
            }
            if let Some(home_object) = agent
                .objects()
                .function_data(record.function_object())
                .and_then(|data| data.home_object())
            {
                return Ok(home_object);
            }
        }
        caller_frame
            .callee()
            .and_then(|callee| {
                agent
                    .objects()
                    .function_data(callee)
                    .and_then(|data| data.home_object())
            })
            .ok_or_else(|| VmError::Abrupt(errors::throw_reference_error(agent)))
    }

    pub(super) fn this_environment_record(
        agent: &Agent,
        start: EnvironmentRef,
    ) -> VmResult<Option<lyng_js_env::FunctionEnvironmentRecord>> {
        let mut current = Some(start);
        while let Some(environment) = current {
            match agent
                .environment(environment)
                .ok_or(VmError::MissingEnvironment(environment))?
            {
                lyng_js_env::EnvironmentRecord::Function(record) => {
                    if record.this_binding_status() == ThisBindingStatus::Lexical {
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

    pub(super) fn resolve_global_this(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        this_value: Value,
    ) -> VmResult<Value> {
        if this_value.is_null() || this_value.is_undefined() {
            let global = agent
                .realm(realm)
                .ok_or(VmError::MissingRootShape(realm))?
                .global_object();
            return Ok(Value::from_object_ref(global));
        }
        if this_value.as_object_ref().is_none() {
            let object = self.to_object_for_value(agent, realm, this_value)?;
            return Ok(Value::from_object_ref(object));
        }
        Ok(this_value)
    }

    pub(super) fn create_construct_this(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        realm: RealmRef,
        new_target: ObjectRef,
    ) -> VmResult<ObjectRef> {
        let prototype = self.get_property_from_object(
            agent,
            host,
            registry,
            frame,
            new_target,
            Value::from_object_ref(new_target),
            PropertyKey::from_atom(WellKnownAtom::prototype.id()),
        )?;
        let prototype = if let Some(prototype) = prototype.as_object_ref() {
            Some(prototype)
        } else {
            let function_realm = Self::function_realm(agent, new_target)?;
            agent
                .realm(function_realm)
                .and_then(|record| record.intrinsics().object_prototype())
        };
        let root_shape = agent
            .realm(realm)
            .and_then(|realm| realm.root_shape())
            .ok_or(VmError::MissingRootShape(realm))?;
        Ok(agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape).with_prototype(prototype),
                AllocationLifetime::Default,
            )
        }))
    }
}

#[derive(Clone, Copy)]
pub(super) struct PreparedBytecodeCall {
    pub(super) code: CodeRef,
    pub(super) realm: RealmRef,
    pub(super) lexical_env: EnvironmentRef,
    pub(super) variable_env: EnvironmentRef,
    pub(super) private_env: Option<EnvironmentRef>,
    pub(super) this_value: Value,
    pub(super) execution_this_state: ThisState,
    pub(super) new_target: Option<ObjectRef>,
    pub(super) callee: ObjectRef,
    pub(super) derived_class_constructor: bool,
    pub(super) parameter_count: u16,
    pub(super) parameter_initializer_end_offset: u32,
    pub(super) register_count: u16,
    pub(super) hidden_register_count: u16,
    pub(super) arguments_mode: ArgumentsMode,
    pub(super) has_rest_parameter: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::call::RejectingNativeRegistry;
    use lyng_js_bytecode::{
        BytecodeBuilder, BytecodeFunctionId, BytecodeFunctionKind, CompiledFunctionUnit,
        CompiledScriptUnit,
    };
    use lyng_js_common::SourceId;
    use lyng_js_compiler::compile_script;
    use lyng_js_env::{
        EnvironmentLayout, EnvironmentLayoutKind, ExecutionContextKind, Runtime, ThisBindingStatus,
    };
    use lyng_js_host::NoopHostHooks;
    use lyng_js_objects::{FunctionObjectData, ObjectColdData};
    use lyng_js_parser::parse_script;
    use lyng_js_sema::analyze_script;
    use lyng_js_types::js3_eval_builtin;

    #[test]
    fn prepared_bytecode_call_threads_private_env_into_execution_context() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist after boot");
        let global_env = realm.global_env();
        let root_shape = realm
            .root_shape()
            .expect("default realm should expose a root shape");
        let private_layout = agent.alloc_environment_layout(EnvironmentLayout::empty(
            EnvironmentLayoutKind::Private,
            true,
        ));
        let private_env = agent
            .alloc_private_environment(
                Some(global_env),
                private_layout,
                AllocationLifetime::Default,
            )
            .expect("private environment should allocate");

        let function = BytecodeBuilder::new(
            BytecodeFunctionId::from_raw(11).unwrap(),
            BytecodeFunctionKind::Function,
        )
        .finish();
        let unit = CompiledFunctionUnit::new(SourceId::new(91), function.id(), vec![function]);
        let mut vm = Vm::new();
        let installed = vm
            .install_function(agent, realm.id(), &unit)
            .expect("function unit should install");
        let callee = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::function(root_shape).with_cold_data(ObjectColdData::Function(
                    FunctionObjectData::bytecode(realm.id(), global_env, installed.code())
                        .with_private_env(Some(private_env)),
                )),
                AllocationLifetime::Default,
            )
        });
        let caller = FrameRecord::new(
            installed.code(),
            0,
            RegisterWindow::new(0, 0),
            None,
            realm.id(),
            global_env,
            global_env,
            ExecutionContextKind::Function,
        );

        let prepared = vm
            .prepare_bytecode_call(agent, caller, callee, Value::undefined(), None)
            .expect("bytecode call should prepare");
        assert_eq!(prepared.private_env, Some(private_env));
        vm.install_prepared_bytecode_call(agent, prepared, &[], 0, None, None, false)
            .expect("bytecode call should install");

        assert_eq!(
            agent
                .current_execution_context()
                .expect("installed call should push an execution context")
                .private_env(),
            Some(private_env)
        );
    }

    #[test]
    fn prepared_runtime_closure_call_frame_resolves_global_eval_through_load_name() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist after boot");
        let global_env = realm.global_env();
        let global_object = realm.global_object();
        let root_shape = realm
            .root_shape()
            .expect("default realm should expose a root shape");
        let empty_layout = agent.alloc_environment_layout(EnvironmentLayout::empty(
            EnvironmentLayoutKind::Function,
            true,
        ));
        let closure_outer = agent
            .alloc_function_environment(
                Some(global_env),
                empty_layout,
                global_object,
                ThisBindingStatus::Initialized,
                Value::from_object_ref(global_object),
                None,
                None,
                AllocationLifetime::Default,
            )
            .expect("root closure environment should allocate");

        let mut function = BytecodeBuilder::new(
            BytecodeFunctionId::from_raw(12).unwrap(),
            BytecodeFunctionKind::Function,
        );
        function.set_needs_environment(true);
        let function = function.finish();
        let unit = CompiledFunctionUnit::new(SourceId::new(92), function.id(), vec![function]);
        let mut vm = Vm::new();
        let _ = vm
            .bootstrap_realm(agent, realm.id(), lyng_js_builtins::BootstrapMode::SpecOnly)
            .expect("bootstrap should succeed");
        let installed = vm
            .install_function(agent, realm.id(), &unit)
            .expect("function unit should install");
        let callee = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::function(root_shape).with_cold_data(ObjectColdData::Function(
                    FunctionObjectData::bytecode(realm.id(), closure_outer, installed.code()),
                )),
                AllocationLifetime::Default,
            )
        });
        let caller = FrameRecord::new(
            installed.code(),
            0,
            RegisterWindow::new(0, 0),
            None,
            realm.id(),
            global_env,
            global_env,
            ExecutionContextKind::Function,
        );

        let prepared = vm
            .prepare_bytecode_call(agent, caller, callee, Value::undefined(), None)
            .expect("bytecode call should prepare");
        vm.install_prepared_bytecode_call(agent, prepared, &[], 0, None, None, false)
            .expect("bytecode call should install");

        let frame = *vm
            .frames
            .last()
            .expect("prepared call should leave one active frame");
        let eval_atom = agent.atoms_mut().intern_collectible("eval");
        let eval_value = vm
            .load_name(agent, frame, eval_atom)
            .expect("prepared runtime closure frame should resolve eval");
        let builtin_eval = vm
            .builtin_cache
            .builtin_constant(agent, realm.id(), js3_eval_builtin())
            .expect("eval builtin should be installed");

        assert_eq!(eval_value, builtin_eval);
    }

    fn compile_test_script(source: &str) -> CompiledScriptUnit {
        let mut atoms = lyng_js_common::AtomTable::new();
        let parsed = parse_script(&mut atoms, SourceId::new(93), source);
        assert!(!parsed.diagnostics.has_errors());
        let sema = analyze_script(&parsed, &atoms);
        assert!(!sema.diagnostics.has_errors());
        compile_script(&parsed, &sema, &mut atoms).expect("script should lower")
    }

    #[test]
    fn prepared_actual_function_expression_call_frame_resolves_global_math_through_load_name() {
        let unit = compile_test_script(
            r#"
                (function() {
                    if (false) eval("1");
                    return Math;
                });
            "#,
        );
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist after boot");
        let mut vm = Vm::new();
        let result = vm
            .evaluate_script(agent, realm, &unit)
            .expect("script should execute and return a function object");
        let function_object = result
            .as_object_ref()
            .expect("script should return the function expression object");
        let Some(FunctionEntryIdentity::Bytecode(code)) = agent
            .objects()
            .function_data(function_object)
            .and_then(|data| data.entry())
        else {
            panic!("function expression should remain backed by installed bytecode");
        };
        let caller = FrameRecord::new(
            code,
            0,
            RegisterWindow::new(0, 0),
            None,
            realm.id(),
            realm.global_env(),
            realm.global_env(),
            ExecutionContextKind::Script,
        );

        let prepared = vm
            .prepare_bytecode_call(agent, caller, function_object, Value::undefined(), None)
            .expect("bytecode call should prepare");
        vm.install_prepared_bytecode_call(agent, prepared, &[], 0, None, None, false)
            .expect("bytecode call should install");

        let frame = *vm
            .frames
            .last()
            .expect("prepared call should leave one active frame");
        let math_atom = agent.atoms_mut().intern_collectible("Math");
        let math_value = vm
            .load_name(agent, frame, math_atom)
            .expect("prepared actual closure frame should resolve Math");

        assert!(math_value.as_object_ref().is_some());
    }

    #[test]
    fn prepared_actual_function_expression_call_runs_dead_eval_branch_without_throwing() {
        let unit = compile_test_script(
            r#"
                (function() {
                    if (false) eval("1");
                });
            "#,
        );
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist after boot");
        let mut vm = Vm::new();
        let result = vm
            .evaluate_script(agent, realm, &unit)
            .expect("script should execute and return a function object");
        let function_object = result
            .as_object_ref()
            .expect("script should return the function expression object");
        let Some(FunctionEntryIdentity::Bytecode(code)) = agent
            .objects()
            .function_data(function_object)
            .and_then(|data| data.entry())
        else {
            panic!("function expression should remain backed by installed bytecode");
        };
        let caller = FrameRecord::new(
            code,
            0,
            RegisterWindow::new(0, 0),
            None,
            realm.id(),
            realm.global_env(),
            realm.global_env(),
            ExecutionContextKind::Script,
        );

        let prepared = vm
            .prepare_bytecode_call(agent, caller, function_object, Value::undefined(), None)
            .expect("bytecode call should prepare");
        vm.install_prepared_bytecode_call(agent, prepared, &[], 0, None, None, false)
            .expect("bytecode call should install");

        let mut registry = RejectingNativeRegistry;
        let result = vm.run(agent, &NoopHostHooks, &mut registry);

        assert_eq!(result, Ok(Value::undefined()));
    }

    #[test]
    fn nested_prepared_actual_function_expression_call_runs_dead_eval_branch_without_throwing() {
        let unit = compile_test_script(
            r#"
                (function() {
                    if (false) eval("1");
                });
            "#,
        );
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist after boot");
        let mut vm = Vm::new();
        let result = vm
            .evaluate_script(agent, realm, &unit)
            .expect("script should execute and return a function object");
        let function_object = result
            .as_object_ref()
            .expect("script should return the function expression object");
        let Some(FunctionEntryIdentity::Bytecode(code)) = agent
            .objects()
            .function_data(function_object)
            .and_then(|data| data.entry())
        else {
            panic!("function expression should remain backed by installed bytecode");
        };
        let caller = FrameRecord::new(
            code,
            0,
            RegisterWindow::new(0, 4),
            None,
            realm.id(),
            realm.global_env(),
            realm.global_env(),
            ExecutionContextKind::Script,
        );
        vm.register_stack.resize(4, Value::undefined());
        agent.push_execution_context(ExecutionContext::script(
            realm.id(),
            realm.global_env(),
            realm.global_env(),
        ));
        vm.frames.push(caller);

        let prepared = vm
            .prepare_bytecode_call(agent, caller, function_object, Value::undefined(), None)
            .expect("bytecode call should prepare");
        vm.install_prepared_bytecode_call(agent, prepared, &[], 4, Some(0), None, false)
            .expect("bytecode call should install");

        let mut registry = RejectingNativeRegistry;
        let result = vm.run(agent, &NoopHostHooks, &mut registry);

        assert_eq!(result, Ok(Value::undefined()));
    }
}

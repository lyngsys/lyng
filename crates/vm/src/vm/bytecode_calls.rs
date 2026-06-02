use super::activation_objects::ActivationObjectInit;
use super::{
    Agent, AllocationLifetime, ArgumentsMode, CodeRef, EnvironmentRef, ExecutionContextKind,
    FrameFlags, FrameRecord, HostHooks, ObjectAllocation, ObjectRef, RealmRef, RegisterWindow,
    ThisBindingStatus, ThisState, Value, Vm, VmDebugSafepointKind, VmError, VmResult,
    WellKnownAtom,
};
use crate::frame::{CallerContext, FrameView};
use lyng_objects::{FunctionEntryIdentity, FunctionThisMode, NativeFunctionRegistry};
use lyng_ops::errors;
use lyng_types::PropertyKey;

const MAX_BYTECODE_CALL_DEPTH: usize = 8_192;

impl Vm {
    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn enter_bytecode_call(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameView,
        result_register: u16,
        callee_object: ObjectRef,
        this_value: Value,
        arguments: &[Value],
        new_target: Option<ObjectRef>,
        construct_call: bool,
    ) -> VmResult<()> {
        let caller_cfr = caller_frame.cfr();
        let prepared = self.prepare_bytecode_call(
            agent,
            self.frame_header(caller_cfr).lexical_env(),
            callee_object,
            this_value,
            new_target,
        )?;
        if self
            .installed_function(prepared.code)
            .is_some_and(|function| function.flags().generator())
        {
            let generator =
                self.instantiate_generator_call(agent, host, registry, prepared, arguments)?;
            self.write_register(
                caller_frame.registers(),
                result_register,
                Value::from_object_ref(generator),
            );
            return Ok(());
        }
        if self
            .installed_function(prepared.code)
            .is_some_and(|function| function.flags().async_function())
        {
            let promise =
                self.instantiate_async_function_call(agent, host, registry, prepared, arguments)?;
            self.write_register(
                caller_frame.registers(),
                result_register,
                Value::from_object_ref(promise),
            );
            return Ok(());
        }
        let construct_this = construct_call
            .then_some(())
            .and_then(|()| this_value.as_object_ref());
        self.install_prepared_bytecode_call(
            agent,
            prepared,
            arguments,
            Some(result_register),
            construct_this,
            construct_call,
        )
        .map(|_register_base| ())
    }

    pub(super) fn recycle_tail_bytecode_call(
        &mut self,
        agent: &mut Agent,
        caller_frame: FrameView,
        callee_object: ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> VmResult<()> {
        let caller_cfr = caller_frame.cfr();
        let tail_caller = self.frame_header(caller_cfr).callee();
        let tail_caller_strict = self.frame_is_strict(caller_frame);
        let prepared = self.prepare_bytecode_call(
            agent,
            self.frame_header(caller_cfr).lexical_env(),
            callee_object,
            this_value,
            None,
        )?;
        let caller_flags =
            crate::frame::FrameFlags::from_raw(self.frame_header(caller_cfr).flags_bits());
        let construct_this = self.frame_header(caller_cfr).construct_this().or_else(|| {
            caller_flags
                .contains(FrameFlags::construct())
                .then_some(())
                .and_then(|()| self.frame_header(caller_cfr).this_value().as_object_ref())
        });
        let caller_return_register = self.frame_header(caller_cfr).return_register();
        // Tear the caller frame down first: `teardown_tail_frame` releases its
        // `[header][window]` run back to the caller's cfr and restores
        // `current_cfr` to the caller's caller. The install below then reserves a
        // fresh run starting at that same cfr — recycling the caller's slot for
        // the tail callee, as the old register-base reuse did.
        self.teardown_tail_frame(agent, caller_frame)?;
        self.install_prepared_bytecode_call(
            agent,
            prepared,
            arguments,
            caller_return_register,
            construct_this,
            caller_flags.contains(FrameFlags::construct()),
        )?;
        if let Some(cold) = self.current_cold_mut() {
            cold.tail_caller = tail_caller;
            cold.tail_caller_strict = tail_caller_strict;
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_lines,
        reason = "spec-shaped VM routine stays contiguous to preserve completion ordering and cleanup invariants"
    )]
    pub(super) fn prepare_bytecode_call(
        &self,
        agent: &mut Agent,
        caller_lexical_env: EnvironmentRef,
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
            .ok_or(VmError::MissingEnvironment(caller_lexical_env))?;
        // Every real bytecode callee carries `Some(realm)` (the `bytecode()`
        // constructor required by the `entry == Bytecode` match above sets it), so
        // the fallback is unreachable in practice. Derive the caller's realm from the
        // active arena frame (the caller IS the current frame at every call site).
        let realm = function_data.realm().unwrap_or_else(|| {
            self.current_realm_of(agent)
                .expect("active frame realm for bytecode call")
        });
        let derived_construct_call = new_target.is_some() && derived_class_constructor;
        // For the Lexical this-mode fallback, derive caller this/new_target from the
        // current arena frame; default to undefined/None when no frame is active
        // (synthetic-frame callers that reach the Lexical branch always have a live
        // environment record, so the fallback is unreachable for synthetic frames).
        let (caller_this_value, caller_new_target) =
            self.frame().map_or((Value::undefined(), None), |f| {
                (f.this_value(), f.new_target())
            });
        let (effective_this, execution_this_state, env_this_status, effective_new_target) =
            match function_data.this_mode() {
                FunctionThisMode::Lexical => {
                    let (lexical_this, lexical_new_target) = Self::lexical_call_state(
                        agent,
                        outer_environment,
                        caller_this_value,
                        caller_new_target,
                    )?;
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
                    let resolved = Self::resolve_global_this(agent, realm, this_value)?;
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
                .and_then(|layout| lyng_env::EnvironmentLayoutId::from_raw(layout.get()))
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

    /// Reserves the callee frame, copies arguments, pushes it, and returns the
    /// callee window base (`register_base`) so callers that key side-tables on
    /// the frame's window (e.g. `async_frame_states`) match the value
    /// `frame.registers().base()` reports.
    pub(super) fn install_prepared_bytecode_call(
        &mut self,
        agent: &mut Agent,
        prepared: PreparedBytecodeCall,
        arguments: &[Value],
        return_register: Option<u16>,
        construct_this: Option<ObjectRef>,
        construct_call: bool,
    ) -> VmResult<u32> {
        // Frame-count guard: cheap O(1) pre-check. The arena soft-limit in
        // `reserve_frame` is the byte-budget backstop that also covers entry
        // and generator-resume paths where no depth counter is maintained.
        if self.frame_depth() >= MAX_BYTECODE_CALL_DEPTH {
            return Err(VmError::Abrupt(errors::throw_range_error(agent)));
        }
        let register_len = prepared
            .register_count
            .checked_add(prepared.hidden_register_count)
            .ok_or_else(|| VmError::Abrupt(errors::throw_range_error(agent)))?;
        let (cfr, register_base) = self.reserve_frame(agent, register_len)?;
        self.copy_arguments_into_frame(register_base, prepared.parameter_count, arguments);

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
            // Reclaim the just-reserved `[header][window]` run; no frame was
            // pushed, so `current_cfr` is unchanged.
            self.arena.release_to(cfr);
            return Err(error);
        }
        let frame = FrameRecord::new(
            prepared.code,
            0,
            RegisterWindow::new(register_base, register_len),
            return_register,
            prepared.lexical_env,
            prepared.variable_env,
            ExecutionContextKind::Function,
        )
        .with_this_value(prepared.this_value)
        .with_this_state(prepared.execution_this_state)
        .with_private_env(prepared.private_env)
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
        self.note_executed_code(frame.code());
        self.push_frame_with_header(cfr, frame);
        self.refresh_running_context(agent);
        self.note_frame_depth();
        self.poll_debug_safepoint(agent, VmDebugSafepointKind::FunctionEntry);
        self.request_dispatch_frame_check();
        Ok(register_base)
    }

    fn teardown_tail_frame(&mut self, agent: &mut Agent, frame: FrameView) -> VmResult<()> {
        if self.current_cfr_opt().is_none() {
            debug_assert!(false, "tail-call recycling requires one active frame");
            return Err(VmError::MissingActiveFrame);
        }
        debug_assert_eq!(self.current_cfr, frame.cfr());
        self.pop_frame_depth();
        self.close_loop_iteration_frames(self.frame_depth());
        self.close_env_scope_frames(self.frame_depth());
        self.for_in_states.clear_window(frame.registers());
        self.iterator_states.clear_window(frame.registers());
        self.captured_name_references
            .clear_window(frame.registers());
        self.finalize_mapped_arguments(agent, self.frame_header(frame.cfr()).lexical_env())?;
        // Release the caller's `[header][window]` run (to its cfr) and restore
        // `current_cfr` to its caller. The recycled tail-callee then re-reserves
        // from this same cfr.
        self.release_frame_to_caller(frame.cfr());
        let _ = self.current_exception.take();
        self.refresh_running_context(agent);
        Ok(())
    }

    pub(super) fn copy_arguments_into_frame(
        &mut self,
        register_base: u32,
        parameter_count: u16,
        arguments: &[Value],
    ) {
        let Ok(register_base) = usize::try_from(register_base) else {
            debug_assert!(false, "register base should fit usize");
            return;
        };
        for index in 0..usize::from(parameter_count) {
            let absolute = register_base + index;
            if let Some(slot) = self.arena.slots_mut().get_mut(absolute) {
                *slot = arguments.get(index).copied().unwrap_or(Value::undefined());
            }
        }
        self.record_argument_frame_copies(u64::from(parameter_count));
    }

    /// Bytecode-to-bytecode call entry that consumes the caller's
    /// contiguous argument window directly. Gated on the caller having
    /// verified eligibility via
    /// [`Vm::ordinary_bytecode_call_eligibility`] — generator/async/
    /// class-constructor/bound/arguments-object/rest-parameter callees
    /// must take [`enter_bytecode_call`] instead.
    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter and call-site state explicitly to mirror the slow-path entry; construct params added for direct register-window construct entry"
    )]
    pub(super) fn enter_bytecode_call_from_caller_registers(
        &mut self,
        agent: &mut Agent,
        caller_frame: FrameView,
        result_register: u16,
        callee_object: ObjectRef,
        this_value: Value,
        caller_arg_base: u32,
        arg_count: u16,
        new_target: Option<ObjectRef>,
        construct_this: Option<ObjectRef>,
        construct_call: bool,
    ) -> VmResult<()> {
        let prepared = self.prepare_bytecode_call(
            agent,
            self.frame_header(caller_frame.cfr()).lexical_env(),
            callee_object,
            this_value,
            new_target,
        )?;
        debug_assert_eq!(prepared.arguments_mode, ArgumentsMode::None);
        debug_assert!(!prepared.has_rest_parameter);
        self.install_prepared_bytecode_call_from_registers(
            agent,
            prepared,
            caller_arg_base,
            arg_count,
            Some(result_register),
            construct_this,
            construct_call,
        )
    }

    /// Mirror of [`Self::install_prepared_bytecode_call`] for the cache
    /// path: copies arguments directly from caller register slots into
    /// the callee frame instead of consuming a `&[Value]` slice. Only
    /// safe when the prepared call has `arguments_mode == None` and no
    /// rest parameter, since `initialize_activation_objects` would
    /// otherwise need a materialized slice.
    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter and call-site state explicitly to mirror the slow-path entry; construct params added for direct register-window construct entry"
    )]
    fn install_prepared_bytecode_call_from_registers(
        &mut self,
        agent: &mut Agent,
        prepared: PreparedBytecodeCall,
        caller_arg_base: u32,
        arg_count: u16,
        return_register: Option<u16>,
        construct_this: Option<ObjectRef>,
        construct_call: bool,
    ) -> VmResult<()> {
        // Frame-count guard: cheap O(1) pre-check. The arena soft-limit in
        // `reserve_frame` is the byte-budget backstop that also covers entry
        // and generator-resume paths where no depth counter is maintained.
        if self.frame_depth() >= MAX_BYTECODE_CALL_DEPTH {
            return Err(VmError::Abrupt(errors::throw_range_error(agent)));
        }
        let register_len = prepared
            .register_count
            .checked_add(prepared.hidden_register_count)
            .ok_or_else(|| VmError::Abrupt(errors::throw_range_error(agent)))?;
        // Reserve first: the new window base sits HEADER_SLOTS above the current
        // arena top, which already lies at or above `caller_arg_base + arg_count`,
        // so the caller-register copy below still satisfies `dest >= src_end`.
        let (cfr, register_base) = self.reserve_frame(agent, register_len)?;
        self.copy_arguments_from_caller_registers(
            register_base,
            prepared.parameter_count,
            caller_arg_base,
            arg_count,
        );

        let frame = FrameRecord::new(
            prepared.code,
            0,
            RegisterWindow::new(register_base, register_len),
            return_register,
            prepared.lexical_env,
            prepared.variable_env,
            ExecutionContextKind::Function,
        )
        .with_this_value(prepared.this_value)
        .with_this_state(prepared.execution_this_state)
        .with_private_env(prepared.private_env)
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
        self.note_executed_code(frame.code());
        self.push_frame_with_header(cfr, frame);
        self.refresh_running_context(agent);
        self.note_frame_depth();
        self.poll_debug_safepoint(agent, VmDebugSafepointKind::FunctionEntry);
        self.request_dispatch_frame_check();
        Ok(())
    }

    fn copy_arguments_from_caller_registers(
        &mut self,
        register_base: u32,
        parameter_count: u16,
        caller_arg_base: u32,
        arg_count: u16,
    ) {
        let Ok(dest_start) = usize::try_from(register_base) else {
            debug_assert!(false, "register base should fit usize");
            return;
        };
        let Ok(src_start) = usize::try_from(caller_arg_base) else {
            debug_assert!(false, "caller arg base should fit usize");
            return;
        };
        let copy_count = usize::from(parameter_count.min(arg_count));
        if copy_count == 0 {
            return;
        }
        let Some(src_end) = src_start.checked_add(copy_count) else {
            debug_assert!(false, "caller arg range should fit usize");
            return;
        };
        debug_assert!(
            dest_start >= src_end,
            "direct caller arg window must sit entirely before the callee frame"
        );
        self.arena
            .slots_mut()
            .copy_within(src_start..src_end, dest_start);
        let copy_count_u64 = u64::try_from(copy_count).unwrap_or(u64::MAX);
        self.record_argument_frame_copies(copy_count_u64);
    }

    pub(super) fn bytecode_entry(agent: &Agent, callee_object: ObjectRef) -> Option<CodeRef> {
        let data = agent.objects().function_data(callee_object)?;
        match data.entry()? {
            FunctionEntryIdentity::Bytecode(code) => Some(code),
            FunctionEntryIdentity::Native(_) | FunctionEntryIdentity::Bound => None,
        }
    }

    pub(super) fn require_callable_object(agent: &mut Agent, value: Value) -> VmResult<ObjectRef> {
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
        caller_this_value: Value,
        caller_new_target: Option<ObjectRef>,
    ) -> VmResult<(Value, Option<ObjectRef>)> {
        if let Some(record) = Self::this_environment_record(agent, start)? {
            return Ok((record.this_value(), record.new_target()));
        }
        Ok((caller_this_value, caller_new_target))
    }

    pub(in crate::vm) fn resolve_this_binding(
        agent: &mut Agent,
        start: EnvironmentRef,
        caller_this_value: Value,
    ) -> VmResult<Value> {
        let Some(record) = Self::this_environment_record(agent, start)? else {
            return Ok(caller_this_value);
        };
        match record.this_binding_status() {
            ThisBindingStatus::Initialized => Ok(record.this_value()),
            ThisBindingStatus::Uninitialized => {
                Err(VmError::Abrupt(errors::throw_reference_error(agent)))
            }
            ThisBindingStatus::Lexical => {
                debug_assert!(false, "lexical this environments are skipped");
                Ok(caller_this_value)
            }
        }
    }

    pub(super) fn resolve_super_home_object(
        agent: &mut Agent,
        start: EnvironmentRef,
        caller_callee: Option<ObjectRef>,
    ) -> VmResult<ObjectRef> {
        if let Some(record) = Self::this_environment_record(agent, start)? {
            if let Some(home_object) = record.home_object() {
                return Ok(home_object);
            }
            if let Some(home_object) = agent
                .objects()
                .function_data(record.function_object())
                .and_then(lyng_objects::FunctionObjectData::home_object)
            {
                return Ok(home_object);
            }
        }
        caller_callee
            .and_then(|callee| {
                agent
                    .objects()
                    .function_data(callee)
                    .and_then(lyng_objects::FunctionObjectData::home_object)
            })
            .ok_or_else(|| VmError::Abrupt(errors::throw_reference_error(agent)))
    }

    pub(super) fn this_environment_record(
        agent: &Agent,
        start: EnvironmentRef,
    ) -> VmResult<Option<lyng_env::FunctionEnvironmentRecord>> {
        let mut current = Some(start);
        while let Some(environment) = current {
            match agent
                .environment(environment)
                .ok_or(VmError::MissingEnvironment(environment))?
            {
                lyng_env::EnvironmentRecord::Function(record) => {
                    if record.this_binding_status() == ThisBindingStatus::Lexical {
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

    pub(super) fn resolve_global_this(
        agent: &mut Agent,
        realm: RealmRef,
        this_value: Value,
    ) -> VmResult<Value> {
        if this_value.is_null() || this_value.is_undefined() {
            let global = agent
                .realm_global_object(realm)
                .ok_or(VmError::MissingRootShape(realm))?;
            return Ok(Value::from_object_ref(global));
        }
        if this_value.as_object_ref().is_none() {
            let object = Self::to_object_for_value(agent, realm, this_value)?;
            return Ok(Value::from_object_ref(object));
        }
        Ok(this_value)
    }

    pub(super) fn create_construct_this(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: CallerContext,
        realm: RealmRef,
        new_target: ObjectRef,
    ) -> VmResult<ObjectRef> {
        let prototype = self.get_property_from_object(
            agent,
            host,
            registry,
            caller.realm,
            caller.lexical_env,
            caller.code,
            caller.pc,
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
    use lyng_bytecode::{
        BytecodeBuilder, BytecodeFunctionId, BytecodeFunctionKind, CompiledFunctionUnit,
        CompiledScriptUnit,
    };
    use lyng_common::SourceId;
    use lyng_compiler::compile_script;
    use lyng_env::{
        EnvironmentLayout, EnvironmentLayoutKind, ExecutionContextKind, Runtime, ThisBindingStatus,
    };
    use lyng_host::NoopHostHooks;
    use lyng_objects::{FunctionObjectData, ObjectColdData};
    use lyng_parser::parse_script;
    use lyng_sema::analyze_script;
    use lyng_types::eval_builtin;

    #[test]
    fn prepared_bytecode_call_threads_private_env_into_frame() {
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
        .finish()
        .expect("test bytecode should build");
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
        let prepared = vm
            .prepare_bytecode_call(agent, global_env, callee, Value::undefined(), None)
            .expect("bytecode call should prepare");
        assert_eq!(prepared.private_env, Some(private_env));
        vm.install_prepared_bytecode_call(agent, prepared, &[], None, None, false)
            .expect("bytecode call should install");

        let frame = vm
            .frame()
            .expect("installed call should leave one active frame");
        assert_eq!(frame.private_env(), Some(private_env));
    }

    #[test]
    fn prepared_bytecode_call_frame_carries_prepared_this_state_and_private_env() {
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
            BytecodeFunctionId::from_raw(13).unwrap(),
            BytecodeFunctionKind::Function,
        )
        .finish()
        .expect("test bytecode should build");
        let unit = CompiledFunctionUnit::new(SourceId::new(94), function.id(), vec![function]);
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
        let prepared = vm
            .prepare_bytecode_call(agent, global_env, callee, Value::undefined(), None)
            .expect("bytecode call should prepare");
        // Capture the prepared values before install; the installed frame must
        // carry these directly (the frame is now the single source of truth).
        let expected_this_state = prepared.execution_this_state;
        let expected_private_env = prepared.private_env;
        vm.install_prepared_bytecode_call(agent, prepared, &[], None, None, false)
            .expect("bytecode call should install");

        let frame = vm
            .frame()
            .expect("prepared call should leave one active frame");

        // The frame is authoritative; it must carry the prepared this_state /
        // private_env directly.
        assert_eq!(frame.this_state(), expected_this_state);
        assert_eq!(frame.private_env(), expected_private_env);
        assert_eq!(frame.private_env(), Some(private_env));
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
        let function = function.finish().expect("test bytecode should build");
        let unit = CompiledFunctionUnit::new(SourceId::new(92), function.id(), vec![function]);
        let mut vm = Vm::new();
        let _ = vm
            .bootstrap_realm(agent, realm.id(), lyng_builtins::BootstrapMode::SpecOnly)
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
        let prepared = vm
            .prepare_bytecode_call(agent, global_env, callee, Value::undefined(), None)
            .expect("bytecode call should prepare");
        vm.install_prepared_bytecode_call(agent, prepared, &[], None, None, false)
            .expect("bytecode call should install");

        let frame = vm
            .frame()
            .expect("prepared call should leave one active frame");
        let eval_atom = agent.atoms_mut().intern_collectible("eval");
        let eval_value = vm
            .load_name(agent, &frame, eval_atom)
            .expect("prepared runtime closure frame should resolve eval");
        let builtin_eval = vm
            .builtin_cache
            .builtin_constant(agent, realm.id(), eval_builtin())
            .expect("eval builtin should be installed");

        assert_eq!(eval_value, builtin_eval);
    }

    fn compile_test_script(source: &str) -> CompiledScriptUnit {
        let mut atoms = lyng_common::AtomTable::new();
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
            .run()
            .expect("script should execute and return a function object");
        let function_object = result
            .as_object_ref()
            .expect("script should return the function expression object");
        let Some(FunctionEntryIdentity::Bytecode(code)) = agent
            .objects()
            .function_data(function_object)
            .and_then(lyng_objects::FunctionObjectData::entry)
        else {
            panic!("function expression should remain backed by installed bytecode");
        };
        let caller = FrameRecord::new(
            code,
            0,
            RegisterWindow::new(0, 0),
            None,
            realm.global_env(),
            realm.global_env(),
            ExecutionContextKind::Script,
        );

        let prepared = vm
            .prepare_bytecode_call(
                agent,
                caller.lexical_env(),
                function_object,
                Value::undefined(),
                None,
            )
            .expect("bytecode call should prepare");
        vm.install_prepared_bytecode_call(agent, prepared, &[], None, None, false)
            .expect("bytecode call should install");

        let frame = vm
            .frame()
            .expect("prepared call should leave one active frame");
        let math_atom = agent.atoms_mut().intern_collectible("Math");
        let math_value = vm
            .load_name(agent, &frame, math_atom)
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
            .run()
            .expect("script should execute and return a function object");
        let function_object = result
            .as_object_ref()
            .expect("script should return the function expression object");
        let Some(FunctionEntryIdentity::Bytecode(code)) = agent
            .objects()
            .function_data(function_object)
            .and_then(lyng_objects::FunctionObjectData::entry)
        else {
            panic!("function expression should remain backed by installed bytecode");
        };
        let caller = FrameRecord::new(
            code,
            0,
            RegisterWindow::new(0, 0),
            None,
            realm.global_env(),
            realm.global_env(),
            ExecutionContextKind::Script,
        );

        let prepared = vm
            .prepare_bytecode_call(
                agent,
                caller.lexical_env(),
                function_object,
                Value::undefined(),
                None,
            )
            .expect("bytecode call should prepare");
        vm.install_prepared_bytecode_call(agent, prepared, &[], None, None, false)
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
            .run()
            .expect("script should execute and return a function object");
        let function_object = result
            .as_object_ref()
            .expect("script should return the function expression object");
        let Some(FunctionEntryIdentity::Bytecode(code)) = agent
            .objects()
            .function_data(function_object)
            .and_then(lyng_objects::FunctionObjectData::entry)
        else {
            panic!("function expression should remain backed by installed bytecode");
        };
        vm.push_test_root_frame(agent, 4, &[Value::undefined(); 4], |window| {
            FrameRecord::new(
                code,
                0,
                window,
                None,
                realm.global_env(),
                realm.global_env(),
                ExecutionContextKind::Script,
            )
        });
        let caller = vm.frame().expect("test root frame should be active");

        let prepared = vm
            .prepare_bytecode_call(
                agent,
                caller.lexical_env(),
                function_object,
                Value::undefined(),
                None,
            )
            .expect("bytecode call should prepare");
        vm.install_prepared_bytecode_call(agent, prepared, &[], Some(0), None, false)
            .expect("bytecode call should install");

        let mut registry = RejectingNativeRegistry;
        let result = vm.run(agent, &NoopHostHooks, &mut registry);

        assert_eq!(result, Ok(Value::undefined()));
    }

    #[test]
    fn register_window_entry_threads_construct_params_into_frame() {
        // Build a minimal bytecode function that does nothing (no arguments,
        // no environment) — we only care that the pushed FrameRecord has the
        // correct this_value, new_target, and construct flag.
        let function = BytecodeBuilder::new(
            BytecodeFunctionId::from_raw(21).unwrap(),
            BytecodeFunctionKind::Function,
        )
        .finish()
        .expect("test bytecode should build");
        let unit = CompiledFunctionUnit::new(SourceId::new(94), function.id(), vec![function]);

        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist after boot");
        let global_env = realm.global_env();
        let root_shape = realm
            .root_shape()
            .expect("default realm should expose a root shape");

        let mut vm = Vm::new();
        let installed = vm
            .install_function(agent, realm.id(), &unit)
            .expect("function unit should install");

        // The callee doubles as new_target (matching real construct semantics).
        let callee = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::function(root_shape).with_cold_data(ObjectColdData::Function(
                    FunctionObjectData::bytecode(realm.id(), global_env, installed.code()),
                )),
                AllocationLifetime::Default,
            )
        });

        // A plain object that represents the freshly-allocated construct_this.
        let construct_this_obj = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                AllocationLifetime::Default,
            )
        });

        // Seed a minimal caller frame with one register slot so
        // caller_arg_base is valid and lies before the callee frame. The frame
        // is reserved through the real path, so its window base sits HEADER_SLOTS
        // above its cfr (not at arena base 0).
        let caller_arg_base =
            vm.push_test_root_frame(agent, 1, &[Value::undefined(); 1], |window| {
                FrameRecord::new(
                    installed.code(),
                    0,
                    window,
                    None,
                    global_env,
                    global_env,
                    ExecutionContextKind::Function,
                )
            }) + 1; // caller_arg_base points past the single register in the caller window
        let caller_frame = vm.frame().expect("test root frame should be active");

        vm.enter_bytecode_call_from_caller_registers(
            agent,
            FrameView::from_record(&caller_frame),
            0,
            callee,
            Value::from_object_ref(construct_this_obj),
            caller_arg_base,
            0,
            Some(callee),
            Some(construct_this_obj),
            true,
        )
        .expect("register-window construct entry should succeed");

        let frame = vm.frame().expect("entry should push a callee frame");

        assert_eq!(
            frame.this_value(),
            Value::from_object_ref(construct_this_obj),
            "callee frame this_value must equal construct_this object"
        );
        assert_eq!(
            frame.new_target(),
            Some(callee),
            "callee frame new_target must equal the callee (new_target param)"
        );
        assert!(
            frame.flags().contains(FrameFlags::construct()),
            "callee frame must have the construct flag set"
        );
        assert_eq!(
            frame.construct_this(),
            Some(construct_this_obj),
            "callee frame construct_this must equal the allocated object"
        );
    }
}

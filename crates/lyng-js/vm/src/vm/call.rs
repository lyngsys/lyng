use super::{
    Agent, CallRange, FrameFlags, FrameRecord, HostHooks, NativeFunctionRegistry, ObjectRef,
    RealmRef, Value, Vm, VmResult,
};
use crate::vm::property_access::VmProxyBridge;
use crate::VmError;
use lyng_js_gc::{PrimitiveMutator, RuntimeBoundFunctionRecord};
use lyng_js_objects::{
    FunctionEntryIdentity, InternalMethodError, NativeCallRequest, NativeConstructRequest,
    ObjectRuntime,
};
use lyng_js_ops::{errors, object, proxy};
use lyng_js_types::{function_call_builtin, FeedbackSlotId};

impl Vm {
    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn invoke_call_target(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        feedback_slot: Option<FeedbackSlotId>,
        result_register: u16,
        callee: ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> VmResult<()> {
        if self.invoke_function_call_builtin_target(
            agent,
            host,
            registry,
            frame,
            feedback_slot,
            result_register,
            callee,
            this_value,
            arguments,
        )? == Some(())
        {
            return Ok(());
        }
        if Self::bytecode_entry(agent, callee).is_some() {
            self.advance_instruction();
            return self.enter_bytecode_call(
                agent,
                host,
                registry,
                frame,
                result_register,
                callee,
                this_value,
                arguments,
                None,
                false,
            );
        }

        if self.try_invoke_cached_builtin_call(
            agent,
            host,
            registry,
            frame,
            feedback_slot,
            result_register,
            callee,
            this_value,
            arguments,
        )? {
            return Ok(());
        }
        let result = if let Some(result) = self.call_builtin(
            agent, host, registry, &frame, callee, this_value, arguments, None,
        )? {
            result
        } else if agent.objects().is_proxy_object(callee) {
            proxy::call(
                &mut VmProxyBridge {
                    vm: self,
                    agent,
                    host,
                    registry,
                    frame: &frame,
                },
                callee,
                this_value,
                arguments,
            )?
        } else {
            object::call(agent, callee, this_value, arguments, registry).map_err(VmError::Abrupt)?
        };
        self.write_register(frame.registers(), result_register, result);
        self.advance_instruction();
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn invoke_collected_call_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        feedback_slot: Option<FeedbackSlotId>,
        result_register: u16,
        callee_value: Value,
        this_value: Value,
        collected_arguments: &mut Vec<Value>,
    ) -> VmResult<()> {
        let mut callee = Self::require_callable_object(agent, frame, callee_value)?;
        let mut effective_this = this_value;
        Self::resolve_bound_call_chain(
            agent,
            &mut callee,
            &mut effective_this,
            collected_arguments,
        )?;
        Self::reject_class_constructor_call(agent, callee, frame.realm())?;

        self.invoke_call_target(
            agent,
            host,
            registry,
            frame,
            feedback_slot,
            result_register,
            callee,
            effective_this,
            collected_arguments,
        )?;
        self.observe_call_target(agent, frame.code(), feedback_slot, callee);
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn try_invoke_cached_builtin_call(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        feedback_slot: Option<FeedbackSlotId>,
        result_register: u16,
        callee: ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> VmResult<bool> {
        let Some(entry) =
            self.cached_frame_safe_builtin_call_target(frame.code(), feedback_slot, callee)
        else {
            return Ok(false);
        };
        let Some(result) = self.call_frame_safe_builtin(
            agent, host, registry, &frame, callee, entry, this_value, arguments,
        )?
        else {
            return Ok(false);
        };
        self.write_register(frame.registers(), result_register, result);
        self.advance_instruction();
        Ok(true)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn invoke_function_call_builtin_target(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        feedback_slot: Option<FeedbackSlotId>,
        result_register: u16,
        callee: ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> VmResult<Option<()>> {
        if Self::builtin_entry(agent, callee) != Some(function_call_builtin()) {
            return Ok(None);
        }

        let mut target = Self::require_callable_object(agent, frame, this_value)?;
        let mut effective_this = arguments.first().copied().unwrap_or(Value::undefined());
        let call_arguments = arguments.get(1..).unwrap_or(&[]);
        if Self::bound_function_record(agent, target).is_some() {
            let mut rebound_arguments = call_arguments.to_vec();
            Self::resolve_bound_call_chain(
                agent,
                &mut target,
                &mut effective_this,
                &mut rebound_arguments,
            )?;
            Self::reject_class_constructor_call(agent, target, frame.realm())?;
            self.invoke_call_target(
                agent,
                host,
                registry,
                frame,
                feedback_slot,
                result_register,
                target,
                effective_this,
                &rebound_arguments,
            )?;
            return Ok(Some(()));
        }

        Self::reject_class_constructor_call(agent, target, frame.realm())?;
        self.invoke_call_target(
            agent,
            host,
            registry,
            frame,
            feedback_slot,
            result_register,
            target,
            effective_this,
            call_arguments,
        )?;
        Ok(Some(()))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn invoke_tail_call_target(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        callee: ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> VmResult<Option<Value>> {
        if let Some(code) = Self::bytecode_entry(agent, callee) {
            if self
                .installed_function(code)
                .is_some_and(|function| function.flags().generator())
            {
                let prepared =
                    self.prepare_bytecode_call(agent, frame, callee, this_value, None)?;
                let generator =
                    self.instantiate_generator_call(agent, host, registry, prepared, arguments)?;
                let _ = agent.pop_execution_context();
                return self.finish_frame(agent, Value::from_object_ref(generator));
            }
            if self
                .installed_function(code)
                .is_some_and(|function| function.flags().async_function())
            {
                let prepared =
                    self.prepare_bytecode_call(agent, frame, callee, this_value, None)?;
                let promise = self
                    .instantiate_async_function_call(agent, host, registry, prepared, arguments)?;
                let _ = agent.pop_execution_context();
                return self.finish_frame(agent, Value::from_object_ref(promise));
            }
            self.recycle_tail_bytecode_call(agent, frame, callee, this_value, arguments)?;
            return Ok(None);
        }

        let result = if let Some(result) = self.call_builtin(
            agent, host, registry, &frame, callee, this_value, arguments, None,
        )? {
            result
        } else if agent.objects().is_proxy_object(callee) {
            proxy::call(
                &mut VmProxyBridge {
                    vm: self,
                    agent,
                    host,
                    registry,
                    frame: &frame,
                },
                callee,
                this_value,
                arguments,
            )?
        } else {
            object::call(agent, callee, this_value, arguments, registry).map_err(VmError::Abrupt)?
        };
        let _ = agent.pop_execution_context();
        self.finish_frame(agent, result)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn call_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        feedback_slot: Option<FeedbackSlotId>,
        result_register: u16,
        callee_register: u16,
        this_register: u16,
        arguments: CallRange,
        spread_mask: Option<u64>,
    ) -> VmResult<()> {
        let callee_value = self.read_register(frame.registers(), callee_register);
        let this_value = self.read_register(frame.registers(), this_register);
        let mut collected_arguments = std::mem::take(&mut self.argument_scratch);
        collected_arguments.clear();
        let result = (|| {
            self.collect_arguments_into(
                agent,
                host,
                registry,
                frame,
                arguments,
                spread_mask,
                &mut collected_arguments,
            )?;
            self.invoke_collected_call_value(
                agent,
                host,
                registry,
                frame,
                feedback_slot,
                result_register,
                callee_value,
                this_value,
                &mut collected_arguments,
            )
        })();
        self.argument_scratch = collected_arguments;
        result
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn call_value_small(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        feedback_slot: Option<FeedbackSlotId>,
        result_register: u16,
        callee_register: u16,
        call_base_register: u16,
        argument_count: u8,
    ) -> VmResult<()> {
        let callee_value = self.read_register(frame.registers(), callee_register);
        let this_value = self.read_register(frame.registers(), call_base_register);
        let mut collected_arguments = std::mem::take(&mut self.argument_scratch);
        collected_arguments.clear();
        collected_arguments.reserve(usize::from(argument_count));
        for offset in 0..argument_count {
            collected_arguments
                .push(self.read_register(frame.registers(), call_base_register + 1 + u16::from(offset)));
        }
        let result = self.invoke_collected_call_value(
            agent,
            host,
            registry,
            frame,
            feedback_slot,
            result_register,
            callee_value,
            this_value,
            &mut collected_arguments,
        );
        self.argument_scratch = collected_arguments;
        result
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn tail_call_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        _feedback_slot: Option<FeedbackSlotId>,
        callee_register: u16,
        this_register: u16,
        arguments: CallRange,
        spread_mask: Option<u64>,
    ) -> VmResult<Option<Value>> {
        let callee_value = self.read_register(frame.registers(), callee_register);
        let this_value = self.read_register(frame.registers(), this_register);
        let mut collected_arguments = std::mem::take(&mut self.argument_scratch);
        collected_arguments.clear();
        let result = (|| {
            self.collect_arguments_into(
                agent,
                host,
                registry,
                frame,
                arguments,
                spread_mask,
                &mut collected_arguments,
            )?;
            let mut callee = Self::require_callable_object(agent, frame, callee_value)?;
            let mut effective_this = this_value;
            Self::resolve_bound_call_chain(
                agent,
                &mut callee,
                &mut effective_this,
                &mut collected_arguments,
            )?;
            Self::reject_class_constructor_call(agent, callee, frame.realm())?;

            self.invoke_tail_call_target(
                agent,
                host,
                registry,
                frame,
                callee,
                effective_this,
                &collected_arguments,
            )
        })();
        self.argument_scratch = collected_arguments;
        result
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "spec-shaped VM routine stays contiguous to preserve completion ordering and cleanup invariants"
    )]
    pub(super) fn construct_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        feedback_slot: Option<FeedbackSlotId>,
        result_register: u16,
        callee_register: u16,
        arguments: CallRange,
        spread_mask: Option<u64>,
    ) -> VmResult<()> {
        let callee_value = self.read_register(frame.registers(), callee_register);
        let mut collected_arguments = std::mem::take(&mut self.argument_scratch);
        collected_arguments.clear();
        let result = (|| {
            self.collect_arguments_into(
                agent,
                host,
                registry,
                frame,
                arguments,
                spread_mask,
                &mut collected_arguments,
            )?;
            let mut callee = callee_value
                .as_object_ref()
                .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
            let mut new_target = callee;
            Self::resolve_bound_construct_chain(
                agent,
                &mut callee,
                &mut new_target,
                &mut collected_arguments,
            )?;
            if !agent.objects().is_constructor(callee) {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }

            if agent.objects().is_proxy_object(callee) {
                let result = proxy::construct(
                    &mut VmProxyBridge {
                        vm: self,
                        agent,
                        host,
                        registry,
                        frame: &frame,
                    },
                    callee,
                    &collected_arguments,
                    Some(new_target),
                )?;
                self.observe_construct_target(
                    agent,
                    frame.code(),
                    feedback_slot,
                    callee,
                    Some(result),
                );
                self.write_register(frame.registers(), result_register, Value::from_object_ref(result));
                self.advance_instruction();
                return Ok(());
            }

            if let Some(code) = Self::bytecode_entry(agent, callee) {
                let derived_construct = self
                    .installed_function(code)
                    .is_some_and(|function| function.flags().derived_class_constructor());
                let construct_this = if derived_construct {
                    None
                } else {
                    Some(self.create_construct_this(
                        agent,
                        host,
                        registry,
                        frame,
                        frame.realm(),
                        new_target,
                    )?)
                };
                let this_value = construct_this.map_or(Value::undefined(), Value::from_object_ref);
                self.observe_construct_target(
                    agent,
                    frame.code(),
                    feedback_slot,
                    callee,
                    construct_this,
                );
                self.advance_instruction();
                return self.enter_bytecode_call(
                    agent,
                    host,
                    registry,
                    frame,
                    result_register,
                    callee,
                    this_value,
                    &collected_arguments,
                    Some(new_target),
                    true,
                );
            }

            let result = if Self::builtin_entry(agent, callee).is_some()
                && !agent.objects().is_constructor(callee)
            {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            } else if let Some(result) = self.call_builtin(
                agent,
                host,
                registry,
                &frame,
                callee,
                Value::undefined(),
                &collected_arguments,
                Some(new_target),
            )? {
                result
                    .as_object_ref()
                    .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?
            } else {
                object::construct(
                    agent,
                    callee,
                    &collected_arguments,
                    Some(new_target),
                    registry,
                )
                .map_err(VmError::Abrupt)?
            };
            self.observe_construct_target(agent, frame.code(), feedback_slot, callee, Some(result));
            self.write_register(frame.registers(), result_register, Value::from_object_ref(result));
            self.advance_instruction();
            Ok(())
        })();
        self.argument_scratch = collected_arguments;
        result
    }

    pub(super) fn bound_function_record(
        agent: &Agent,
        callee_object: ObjectRef,
    ) -> Option<RuntimeBoundFunctionRecord> {
        let data = agent.objects().function_data(callee_object)?;
        if data.entry()? != FunctionEntryIdentity::Bound {
            return None;
        }
        let payload = data.gc_payload()?;
        agent.heap().view().function_payload(payload)?.bound()
    }

    pub(super) fn function_realm(agent: &mut Agent, function: ObjectRef) -> VmResult<RealmRef> {
        if let Some(bound) = Self::bound_function_record(agent, function) {
            return Self::function_realm(agent, bound.target());
        }
        if agent.objects().is_proxy_object(function) {
            let data = agent
                .objects()
                .proxy_data(function)
                .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
            if data.revoked() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return Self::function_realm(agent, data.target());
        }
        agent
            .objects()
            .function_data(function)
            .and_then(lyng_js_objects::FunctionObjectData::realm)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))
    }

    pub(super) fn prepend_bound_arguments(
        agent: &mut Agent,
        bound: RuntimeBoundFunctionRecord,
        arguments: &mut Vec<Value>,
    ) -> VmResult<()> {
        let mut combined = Vec::new();
        if let Some(bound_arguments) = bound.arguments() {
            let heap_view = agent.heap().view();
            let Some(slots) = heap_view.object_slots(bound_arguments) else {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            };
            combined.extend_from_slice(slots);
        }
        combined.extend_from_slice(arguments);
        *arguments = combined;
        Ok(())
    }

    pub(super) fn resolve_bound_call_chain(
        agent: &mut Agent,
        callee: &mut ObjectRef,
        this_value: &mut Value,
        arguments: &mut Vec<Value>,
    ) -> VmResult<()> {
        while let Some(bound) = Self::bound_function_record(agent, *callee) {
            Self::prepend_bound_arguments(agent, bound, arguments)?;
            *this_value = bound.this_value();
            *callee = bound.target();
        }
        Ok(())
    }

    pub(super) fn resolve_bound_construct_chain(
        agent: &mut Agent,
        callee: &mut ObjectRef,
        new_target: &mut ObjectRef,
        arguments: &mut Vec<Value>,
    ) -> VmResult<()> {
        while let Some(bound) = Self::bound_function_record(agent, *callee) {
            Self::prepend_bound_arguments(agent, bound, arguments)?;
            if *new_target == *callee {
                *new_target = bound.target();
            }
            *callee = bound.target();
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn collect_arguments_into(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        range: CallRange,
        spread_mask: Option<u64>,
        arguments: &mut Vec<Value>,
    ) -> VmResult<()> {
        arguments.clear();
        arguments.reserve(usize::from(range.argument_count()));
        let spread_mask = spread_mask.unwrap_or(0);
        for offset in 0..range.argument_count() {
            let value = self.read_register(frame.registers(), range.argument_base() + offset);
            let is_spread = usize::from(offset) < u64::BITS as usize
                && (spread_mask & (1_u64 << u32::from(offset))) != 0;
            if !is_spread {
                arguments.push(value);
                continue;
            }
            self.append_spread_argument(agent, host, registry, frame, value, arguments)?;
        }
        Ok(())
    }

    pub(super) fn append_spread_argument(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        value: Value,
        arguments: &mut Vec<Value>,
    ) -> VmResult<()> {
        self.append_iterator_values(agent, host, registry, &frame, value, arguments)
    }
}

pub(super) fn finalize_frame_result(
    agent: &mut Agent,
    frame: FrameRecord,
    result: Value,
) -> VmResult<Value> {
    if frame.flags().contains(FrameFlags::construct()) && result.as_object_ref().is_none() {
        if frame.flags().contains(FrameFlags::derived_construct()) {
            if !result.is_undefined() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            if frame.construct_this().is_none() {
                return Err(VmError::Abrupt(errors::throw_reference_error(agent)));
            }
        }
        return Ok(frame
            .construct_this()
            .map_or_else(|| frame.this_value(), Value::from_object_ref));
    }
    Ok(result)
}

#[derive(Default)]
pub(super) struct RejectingNativeRegistry;

impl NativeFunctionRegistry for RejectingNativeRegistry {
    fn call(
        &mut self,
        _runtime: &mut ObjectRuntime,
        _heap: &mut PrimitiveMutator<'_>,
        _request: NativeCallRequest<'_>,
    ) -> Result<Value, InternalMethodError> {
        Err(InternalMethodError::MissingFunctionPayload)
    }

    fn construct(
        &mut self,
        _runtime: &mut ObjectRuntime,
        _heap: &mut PrimitiveMutator<'_>,
        _request: NativeConstructRequest<'_>,
    ) -> Result<ObjectRef, InternalMethodError> {
        Err(InternalMethodError::MissingFunctionPayload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FrameRecord, InstalledCode, RegisterWindow};
    use lyng_js_bytecode::CompiledScriptUnit;
    use lyng_js_common::{AtomId, AtomTable, SourceId};
    use lyng_js_compiler::compile_script;
    use lyng_js_env::{ExecutionContextKind, Runtime};
    use lyng_js_host::NoopHostHooks;
    use lyng_js_objects::FunctionEntryIdentity;
    use lyng_js_ops::object::ordinary_get;
    use lyng_js_parser::parse_script;
    use lyng_js_sema::analyze_script;
    use lyng_js_types::PropertyKey;

    fn compile_test_unit(source_id: u32, source: &str) -> CompiledScriptUnit {
        let mut atoms = AtomTable::new();
        let parsed = parse_script(&mut atoms, SourceId::new(source_id), source);
        assert!(!parsed.diagnostics.has_errors());
        let sema = analyze_script(&parsed, &atoms);
        assert!(!sema.diagnostics.has_errors());
        compile_script(&parsed, &sema, &mut atoms).unwrap()
    }

    fn unit_atom(unit: &CompiledScriptUnit, text: &str) -> AtomId {
        unit.atoms()
            .iter()
            .find_map(|(atom, candidate)| (candidate.as_str() == Some(text)).then_some(*atom))
            .unwrap_or_else(|| panic!("compiled unit should intern atom {text:?}"))
    }

    fn unit_runtime_atom(agent: &mut Agent, unit: &CompiledScriptUnit, atom: AtomId) -> AtomId {
        if let Some(text) = unit.atom_text(atom) {
            return agent.atoms_mut().intern_collectible(text);
        }
        let units = unit
            .atom_utf16(atom)
            .expect("compiled unit atom should resolve to UTF-8 or UTF-16 data");
        agent.atoms_mut().intern_collectible_utf16(units)
    }

    #[test]
    fn indexed_accessor_gets_match_direct_getter_invocation() {
        let unit = compile_test_unit(
            71,
            r"
            var object = {
                get [1]() { return 10; },
                set [1](_) {}
            };
            ",
        );
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent.default_realm().expect("default realm should exist");
        let mut vm = Vm::new();
        let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
        let _ = vm
            .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap();

        let object_atom = unit_atom(&unit, "object");
        let runtime_atom = unit_runtime_atom(agent, &unit, object_atom);
        let object_value = ordinary_get(
            agent,
            realm.global_object(),
            PropertyKey::from_atom(runtime_atom),
        )
        .unwrap();
        let object = object_value
            .as_object_ref()
            .expect("global object binding should store an object literal");
        let descriptor = agent
            .objects()
            .get_own_property(agent.heap().view(), object, PropertyKey::Index(1))
            .unwrap()
            .expect("index accessor should be installed");
        let getter = descriptor
            .getter()
            .and_then(Value::as_object_ref)
            .expect("index descriptor should carry a getter closure");
        let Some(FunctionEntryIdentity::Bytecode(getter_code)) = agent
            .objects()
            .function_data(getter)
            .and_then(lyng_js_objects::FunctionObjectData::entry)
        else {
            panic!("getter descriptor should reference bytecode");
        };
        let getter_environment = agent
            .objects()
            .function_data(getter)
            .and_then(lyng_js_objects::FunctionObjectData::environment)
            .expect("getter closure should preserve its outer environment");
        let getter_entry = vm
            .installed_function(getter_code)
            .expect("getter bytecode should stay installed")
            .id();
        let frame = FrameRecord::new(
            installed.code(),
            0,
            RegisterWindow::new(0, 0),
            None,
            realm.id(),
            realm.global_env(),
            realm.global_env(),
            ExecutionContextKind::Script,
        );
        let mut registry = RejectingNativeRegistry;

        let direct = vm
            .evaluate_installed(
                agent,
                InstalledCode::new(getter_code, getter_entry),
                getter_environment,
                getter_environment,
            )
            .expect("getter should execute directly");
        assert_eq!(direct, Value::from_smi(10));

        let via_property = vm
            .get_property_from_value(
                agent,
                &lyng_js_host::NoopHostHooks,
                &mut registry,
                &frame,
                Value::from_object_ref(object),
                PropertyKey::Index(1),
            )
            .expect("indexed accessor should resolve through the property helper");
        assert_eq!(via_property, Value::from_smi(10));
    }
}

use super::support::*;

#[test]
fn installed_code_reports_bytecode_executable_identity() {
    let installed = InstalledCode::new(
        CodeRef::from_raw(9).unwrap(),
        BytecodeFunctionId::from_raw(2).unwrap(),
    );

    assert_eq!(installed.code(), CodeRef::from_raw(9).unwrap());
    assert_eq!(installed.entry(), BytecodeFunctionId::from_raw(2).unwrap());
    assert_eq!(
        installed.executable(),
        ExecutableId::Bytecode(CodeRef::from_raw(9).unwrap())
    );
}
#[test]
fn seed_registers_uses_window_length() {
    let registers = seed_registers(RegisterWindow::new(10, 3));

    assert_eq!(registers.len(), 3);
    assert!(registers.iter().all(|value| *value == Value::undefined()));
}

#[test]
fn frame_record_carries_bytecode_execution_state() {
    let frame = FrameRecord::new(
        CodeRef::from_raw(2).unwrap(),
        4,
        RegisterWindow::new(8, 2),
        Some(1),
        RealmRef::from_raw(1).unwrap(),
        EnvironmentRef::from_raw(3).unwrap(),
        EnvironmentRef::from_raw(4).unwrap(),
        ExecutionContextKind::Function,
    )
    .with_this_value(Value::from_smi(9))
    .with_handler_cursor(2)
    .with_flags(FrameFlags::entry().with_flag(FrameFlags::suspendable(), true));

    assert_eq!(size_of::<FrameFlags>(), size_of::<u8>());
    assert_eq!(frame.instruction_offset(), 4);
    assert_eq!(frame.realm(), RealmRef::from_raw(1).unwrap());
    assert_eq!(frame.lexical_env(), EnvironmentRef::from_raw(3).unwrap());
    assert_eq!(frame.variable_env(), EnvironmentRef::from_raw(4).unwrap());
    assert_eq!(frame.this_value(), Value::from_smi(9));
    assert_eq!(frame.handler_cursor(), 2);
    assert!(frame.flags().contains(FrameFlags::entry()));
    assert!(frame.flags().contains(FrameFlags::suspendable()));
}

#[test]
fn vm_installs_script_units_into_code_storage_and_executes_basic_dispatch() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder.set_name(Some(AtomId::from_raw(17)));
    builder
        .alloc_registers(2)
        .expect("test bytecode registers should allocate");
    let constant = builder
        .add_constant(ConstantValue::Smi(41))
        .expect("test bytecode constant should build");
    builder
        .emit_abx(Opcode::LoadConst, 0, constant)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::Move, 1, 0, 0)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 1)
        .expect("test bytecode should build");
    builder
        .add_feedback_site(
            0,
            FeedbackSiteKind::Arithmetic,
            lyng_js_bytecode::FeedbackSiteMetadata::None,
        )
        .expect("test bytecode feedback site should build");
    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(9), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let code_record = agent
        .heap()
        .view()
        .code(installed.code())
        .expect("installed code record should exist");
    let code_slots = code_record
        .constants()
        .and_then(|slots| agent.heap().view().code_slots(slots))
        .expect("constant slots should exist");

    assert_eq!(code_record.realm(), Some(realm.id()));
    assert_eq!(code_slots, &[Value::from_smi(41)]);

    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(41));
    assert!(vm.frames().is_empty());
    assert!(vm.register_stack().is_empty());
    assert!(agent.current_execution_context().is_none());
}

#[test]
fn vm_function_table_dispatch_mode_executes_installed_bytecode() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(12).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(3)
        .expect("test bytecode registers should allocate");
    builder
        .emit_abx(Opcode::LoadOne, 0, 0)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::AddSmi, 1, 0, 41)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::Move, 2, 1, 0)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 2)
        .expect("test bytecode should build");
    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(12), function.id(), vec![function]);

    let mut vm = Vm::new();
    vm.set_dispatch_mode(VmDispatchMode::FunctionTable);
    assert_eq!(vm.dispatch_mode(), VmDispatchMode::FunctionTable);

    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(42));
    assert!(vm.frames().is_empty());
}

#[test]
fn vm_executes_specialized_smi_opcodes_and_fallback_paths() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(11).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(13)
        .expect("test bytecode registers should allocate");
    builder
        .emit_abx(Opcode::LoadOne, 0, 0)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::AddSmi, 1, 0, 13)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::SubSmi, 2, 1, 5)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::MulSmi, 3, 2, 7)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::DivSmi, 4, 3, 2)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::ModSmi, 4, 4, 5)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::BitAndSmi, 4, 4, 3)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::EqualZero, 4, 3, 0)
        .expect("test bytecode should build");
    builder
        .emit_abx(Opcode::LoadZero, 5, 0)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::EqualZero, 6, 5, 0)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::AddSmi, 7, 6, 1)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::SubSmi, 8, 6, 1)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::MulSmi, 9, 6, 7)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::Add, 10, 3, 7)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::Add, 11, 10, 8)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::Add, 12, 11, 9)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 12)
        .expect("test bytecode should build");
    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(22), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(72));
}

#[test]
fn vm_rejects_register_operands_outside_installed_frame() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        None,
        ArgumentsMode::None,
    )
    .with_kind(BytecodeFunctionKind::Script)
    .with_register_counts(1, 0)
    .with_instructions(vec![
        Instruction::abc(Opcode::Move, 1, 0, 0),
        Instruction::ax(Opcode::ReturnUndefined, 0),
    ]);
    let unit = CompiledScriptUnit::new(SourceId::new(19), function.id(), vec![function]);

    let mut vm = Vm::new();
    let error = vm
        .install_script(agent, realm.id(), &unit)
        .expect_err("invalid register operands should be rejected at install");

    assert!(matches!(
        error,
        VmError::RegisterOutOfBounds {
            code,
            register: 1
        } if code == CodeRef::from_raw(1).unwrap()
    ));
}

#[test]
fn vm_installs_callable_index_accessors_from_object_literals() {
    let unit = compile_test_unit(
        41,
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
    let _ = vm.evaluate_script(agent, realm, &unit).unwrap();

    let object_atom = unit_atom(&unit, "object");
    let runtime_atom = unit_runtime_atom(agent, &unit, object_atom);
    let global_object = realm.global_object();
    let object_value =
        ordinary_get(agent, global_object, PropertyKey::from_atom(runtime_atom)).unwrap();
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
        .expect("getter should be present on the index descriptor");
    let setter = descriptor
        .setter()
        .expect("setter should be present on the index descriptor");
    let getter_object = getter
        .as_object_ref()
        .expect("getter should be represented as an object reference");
    let setter_object = setter
        .as_object_ref()
        .expect("setter should be represented as an object reference");

    assert!(
        agent.objects().function_data(getter_object).is_some(),
        "getter slot should contain a callable function object"
    );
    assert!(
        agent.objects().function_data(setter_object).is_some(),
        "setter slot should contain a callable function object"
    );
    assert!(matches!(
        agent
            .objects()
            .function_data(getter_object)
            .and_then(lyng_js_objects::FunctionObjectData::entry),
        Some(FunctionEntryIdentity::Bytecode(_))
    ));
    assert!(matches!(
        agent
            .objects()
            .function_data(setter_object)
            .and_then(lyng_js_objects::FunctionObjectData::entry),
        Some(FunctionEntryIdentity::Bytecode(_))
    ));

    let Some(FunctionEntryIdentity::Bytecode(getter_code)) = agent
        .objects()
        .function_data(getter_object)
        .and_then(lyng_js_objects::FunctionObjectData::entry)
    else {
        panic!("getter should remain backed by installed bytecode");
    };
    let getter_function = vm
        .installed_function(getter_code)
        .expect("getter bytecode should stay installed");
    let getter_environment = agent
        .objects()
        .function_data(getter_object)
        .and_then(lyng_js_objects::FunctionObjectData::environment)
        .expect("getter closure should preserve its outer environment");
    let getter_result = vm
        .evaluate_installed(
            agent,
            InstalledCode::new(getter_code, getter_function.id()),
            getter_environment,
            getter_environment,
        )
        .expect("getter bytecode should execute as a standalone entry");
    assert_eq!(getter_result, Value::from_smi(10));
}

#[test]
fn vm_bootstraps_phase5_default_global_bindings_before_script_entry() {
    let unit = compile_test_unit(
        52,
        r"
        (globalThis === this ? 1 : 0)
            + (Infinity === 1 / 0 ? 2 : 0)
            + (NaN !== NaN ? 4 : 0)
            + (undefined === undefined ? 8 : 0);
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let global_this = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            realm.global_object(),
            PropertyKey::from_atom(agent.bootstrap_atoms().global_this()),
        )
        .unwrap()
        .expect("globalThis should be installed during entry bootstrap");

    assert_eq!(result, Value::from_smi(15));
    assert_eq!(
        agent
            .realm(realm.id())
            .expect("default realm should remain queryable")
            .bootstrap_state(),
        RealmBootstrapState::new().with_spec_ready(true)
    );
    assert_eq!(
        global_this.value(),
        Some(Value::from_object_ref(realm.global_object()))
    );
}

#[test]
fn bootstrap_installs_phase6_wrapper_prototypes_for_to_object() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let string = agent.alloc_runtime_string("abc", None, AllocationLifetime::Default);
    let bigint = agent.heap_mut().mutator().alloc_bigint(
        BigIntSign::NonNegative,
        &[23],
        AllocationLifetime::Default,
    );

    let _ = vm
        .bootstrap_realm(agent, realm.id(), lyng_js_builtins::BootstrapMode::SpecOnly)
        .expect("bootstrap should succeed");

    assert!(lyng_js_ops::object::to_object(agent, realm.id(), Value::from_smi(7)).is_ok());
    assert!(
        lyng_js_ops::object::to_object(agent, realm.id(), Value::from_string_ref(string)).is_ok()
    );
    assert!(
        lyng_js_ops::object::to_object(agent, realm.id(), Value::from_bigint_ref(bigint)).is_ok()
    );
}

struct WrapperPrimitiveProbe<'a> {
    agent: &'a mut lyng_js_env::Agent,
    called: bool,
}

impl lyng_js_ops::object::ToPrimitiveContext for WrapperPrimitiveProbe<'_> {
    type Error = lyng_js_types::AbruptCompletion;

    fn agent(&mut self) -> &mut lyng_js_env::Agent {
        self.agent
    }

    fn abrupt(&mut self, completion: lyng_js_types::AbruptCompletion) -> Self::Error {
        completion
    }

    fn type_error(&mut self) -> Self::Error {
        lyng_js_ops::errors::throw_type_error(self.agent)
    }

    fn get_property_value(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> Result<Value, Self::Error> {
        ordinary_get(self.agent, object, key)
    }

    fn require_callable_object(&mut self, value: Value) -> Result<ObjectRef, Self::Error> {
        let Some(object) = value.as_object_ref() else {
            return Err(lyng_js_ops::errors::throw_type_error(self.agent));
        };
        if self.agent.objects().function_data(object).is_some() {
            Ok(object)
        } else {
            Err(lyng_js_ops::errors::throw_type_error(self.agent))
        }
    }

    fn call_to_completion(
        &mut self,
        _callee_object: ObjectRef,
        this_value: Value,
        _arguments: &[Value],
    ) -> Result<Value, Self::Error> {
        self.called = true;
        let Some(object) = this_value.as_object_ref() else {
            return Err(lyng_js_ops::errors::throw_type_error(self.agent));
        };
        self.agent
            .objects()
            .primitive_wrapper_value(self.agent.heap().view(), object)
            .ok_or_else(|| lyng_js_ops::errors::throw_type_error(self.agent))
    }
}

#[test]
fn bootstrap_string_wrapper_uses_bootstrapped_string_prototype_methods() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let string = agent.alloc_runtime_string("abc", None, AllocationLifetime::Default);

    let _ = vm
        .bootstrap_realm(agent, realm.id(), lyng_js_builtins::BootstrapMode::SpecOnly)
        .expect("bootstrap should succeed");

    let string_wrapper =
        lyng_js_ops::object::to_object(agent, realm.id(), Value::from_string_ref(string)).unwrap();
    let mut probe = WrapperPrimitiveProbe {
        agent,
        called: false,
    };

    assert_eq!(
        lyng_js_ops::object::to_primitive(
            &mut probe,
            Value::from_object_ref(string_wrapper),
            lyng_js_ops::object::ToPrimitiveHint::Number,
        ),
        Ok(Value::from_string_ref(string))
    );
    assert!(probe.called);
}

#[test]
fn global_script_instantiation_precreates_non_configurable_var_bindings() {
    let unit = compile_test_unit(53, "var x = 1;");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let _ = vm
        .bootstrap_realm(agent, realm.id(), lyng_js_builtins::BootstrapMode::SpecOnly)
        .expect("bootstrap should succeed");
    let _ = vm.install_script(agent, realm.id(), &unit).unwrap();
    Vm::instantiate_global_script(agent, &realm, unit.instantiation_plan()).unwrap();

    let x_atom = unit_runtime_atom(agent, &unit, unit_atom(&unit, "x"));
    let descriptor = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            realm.global_object(),
            PropertyKey::from_atom(x_atom),
        )
        .unwrap()
        .expect("instantiation should precreate a global property");

    assert_eq!(descriptor.value(), Some(Value::undefined()));
    assert_eq!(descriptor.writable(), Some(true));
    assert_eq!(descriptor.enumerable(), Some(true));
    assert_eq!(descriptor.configurable(), Some(false));
}

#[test]
fn global_script_instantiation_uses_dictionary_storage_for_bulk_var_bindings() {
    let mut source = String::new();
    for index in 0..96 {
        writeln!(&mut source, "var binding_{index}").expect("writing to String should not fail");
    }
    let unit = compile_test_unit(5_301, &source);
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let _ = vm
        .bootstrap_realm(agent, realm.id(), lyng_js_builtins::BootstrapMode::SpecOnly)
        .expect("bootstrap should succeed");
    let _ = vm.install_script(agent, realm.id(), &unit).unwrap();
    Vm::instantiate_global_script(agent, &realm, unit.instantiation_plan()).unwrap();

    assert_eq!(
        agent
            .objects()
            .named_property_storage_mode(realm.global_object()),
        Some(NamedPropertyStorageMode::Dictionary)
    );
    let last_atom = agent.atoms_mut().intern_collectible("binding_95");
    assert!(agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            realm.global_object(),
            PropertyKey::from_atom(last_atom),
        )
        .unwrap()
        .is_some());
}

#[test]
fn vm_executes_wide_register_and_constant_operands() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(7).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(300)
        .expect("test bytecode registers should allocate");
    let mut last_constant = 0;
    for index in 0..70_000u32 {
        last_constant = builder
            .add_constant(ConstantValue::Smi(index.cast_signed()))
            .expect("test bytecode constant should build");
    }
    builder
        .emit_abx(Opcode::LoadConst, 299u16, last_constant)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 299)
        .expect("test bytecode should build");

    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(17), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(69_999));
}

#[test]
fn vm_executes_wide_conditional_jumps() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(8).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(300)
        .expect("test bytecode registers should allocate");
    builder
        .emit_abx(Opcode::LoadTrue, 299u16, 0)
        .expect("test bytecode should build");
    let jump = builder
        .emit_cond_jump_placeholder(Opcode::JumpIfTrue, 299u16)
        .expect("test bytecode should build");
    builder
        .emit_abx(Opcode::LoadSmi, 0u16, 1u16)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 0)
        .expect("test bytecode should build");
    for _ in 0..40_000 {
        builder
            .emit_ax(Opcode::Nop, 0)
            .expect("test bytecode should build");
    }
    let target = builder
        .current_offset()
        .expect("test bytecode offset should build");
    builder
        .emit_abx(Opcode::LoadSmi, 0u16, 7u16)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 0)
        .expect("test bytecode should build");
    builder
        .patch_jump_to(jump, target)
        .expect("test bytecode jump should patch");

    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(18), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn global_load_matches_runtime_atom_text_when_ids_differ() {
    let unit = compile_test_unit(19, "runtimeOnly;");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let _ = agent.atoms_mut().intern_collectible("padding");
    let runtime_name = agent.atoms_mut().intern_collectible("runtimeOnly");
    install_global_value(agent, &realm, runtime_name, Value::from_smi(13));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(13));
}

#[test]
fn typeof_name_resolution_matches_runtime_atom_text_when_ids_differ() {
    let unit = compile_test_unit(20, "typeof runtimeOnly;");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let _ = agent.atoms_mut().intern_collectible("padding");
    let runtime_name = agent.atoms_mut().intern_collectible("runtimeOnly");
    install_global_value(agent, &realm, runtime_name, Value::from_smi(13));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    let string = result
        .as_string_ref()
        .expect("typeof should return a string");
    let view = agent
        .heap()
        .view()
        .string_view(string)
        .expect("string should exist in the heap");

    assert_eq!(decode_string(&view), "number");
}

#[test]
fn concatenated_strings_feed_char_access_and_slice_consumers() {
    let unit = compile_test_unit(
        2_050,
        r#"
        let value = "ab" + "cd";
        String.fromCharCode(value.charCodeAt(2)) + value.slice(1, 3);
        "#,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let string = result
        .as_string_ref()
        .expect("consumer result should be a string");
    let view = agent
        .heap()
        .view()
        .string_view(string)
        .expect("string should exist in the heap");

    assert_eq!(decode_string(&view), "cbc");
}

#[test]
fn direct_named_property_definitions_preserve_all_named_slots() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });

    for (name, value) in [
        ("1.2", 1),
        ("1e+55", 2),
        ("0.000001", 3),
        ("Infinity", 5),
        ("-Infinity", 6),
        ("NaN", 7),
    ] {
        let atom = agent.atoms_mut().intern_collectible(name);
        assert!(ordinary_create_data_property(
            agent,
            object,
            PropertyKey::from_atom(atom),
            Value::from_smi(value),
            AllocationLifetime::Default,
        )
        .unwrap());
    }

    for (name, value) in [
        ("1.2", 1),
        ("1e+55", 2),
        ("0.000001", 3),
        ("Infinity", 5),
        ("-Infinity", 6),
        ("NaN", 7),
    ] {
        let atom = agent.atoms_mut().intern_collectible(name);
        assert_eq!(
            ordinary_get(agent, object, PropertyKey::from_atom(atom)).unwrap(),
            Value::from_smi(value)
        );
    }
}

#[test]
fn vm_tracks_child_parent_links_and_unconditional_jumps() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let child = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(2).unwrap(),
        Some(AtomId::from_raw(33)),
        ArgumentsMode::None,
    )
    .with_register_counts(1, 0)
    .with_instructions(vec![Instruction::ax(Opcode::ReturnUndefined, 0)]);

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        BytecodeFunctionKind::Function,
    );
    builder
        .alloc_registers(1)
        .expect("test bytecode registers should allocate");
    let jump = builder
        .emit_jump_placeholder(Opcode::Jump)
        .expect("test bytecode should build");
    builder
        .emit_abx(Opcode::LoadSmi, 0, 99)
        .expect("test bytecode should build");
    let ret = builder
        .emit_ax(Opcode::ReturnUndefined, 0)
        .expect("test bytecode should build");
    builder
        .patch_jump_to(jump, ret)
        .expect("test bytecode jump should patch");
    let parent = builder
        .finish()
        .expect("test bytecode should build")
        .with_child_functions(vec![child.id()]);
    let unit = CompiledFunctionUnit::new(SourceId::new(11), parent.id(), vec![parent, child]);

    let mut vm = Vm::new();
    let installed = vm.install_function(agent, realm.id(), &unit).unwrap();
    let child_code = vm
        .installed_child_code(installed.code(), 0)
        .expect("installed child code should exist");

    assert_eq!(
        agent.heap().view().code(child_code).unwrap().parent(),
        Some(installed.code())
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::undefined()
    );
}

#[test]
fn load_const_supports_atom_backed_string_constants() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        None,
        ArgumentsMode::None,
    )
    .with_kind(BytecodeFunctionKind::Script)
    .with_register_counts(1, 0)
    .with_constants(vec![ConstantValue::Atom(AtomId::from_raw(9))])
    .with_instructions(vec![
        Instruction::abx(Opcode::LoadConst, 0, 0),
        Instruction::ax(Opcode::Return, 0),
    ]);
    let unit =
        CompiledScriptUnit::new(SourceId::new(13), function.id(), vec![function]).with_atoms(vec![
            (AtomId::from_raw(9), CompiledAtom::from("loaded-atom")),
        ]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    let string = result
        .as_string_ref()
        .expect("atom constant should load a string");
    let view = agent
        .heap()
        .view()
        .string_view(string)
        .expect("loaded string should exist in the heap");
    let cached_atom = view.cached_atom();
    let expected_atom = agent.atoms_mut().intern_collectible("loaded-atom");

    assert_eq!(cached_atom, Some(expected_atom));
}

#[test]
fn load_const_supports_utf16_only_atom_backed_string_constants() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        None,
        ArgumentsMode::None,
    )
    .with_kind(BytecodeFunctionKind::Script)
    .with_register_counts(1, 0)
    .with_constants(vec![ConstantValue::Atom(AtomId::from_raw(9))])
    .with_instructions(vec![
        Instruction::abx(Opcode::LoadConst, 0, 0),
        Instruction::ax(Opcode::Return, 0),
    ]);
    let unit =
        CompiledScriptUnit::new(SourceId::new(14), function.id(), vec![function]).with_atoms(vec![
            (AtomId::from_raw(9), CompiledAtom::from(vec![0xD83D])),
        ]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    let string = result
        .as_string_ref()
        .expect("atom constant should load a string");
    let view = agent
        .heap()
        .view()
        .string_view(string)
        .expect("loaded string should exist in the heap");
    let bytes = view
        .utf16_bytes()
        .expect("UTF-16-only atom constant should materialize as a UTF-16 string");
    let units = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    let cached_atom = view.cached_atom();
    let expected_atom = agent.atoms_mut().intern_collectible_utf16(&[0xD83D]);

    assert_eq!(units, vec![0xD83D]);
    assert_eq!(cached_atom, Some(expected_atom));
}

#[test]
fn load_const_still_rejects_builtin_constants_without_runtime_support() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        None,
        ArgumentsMode::None,
    )
    .with_kind(BytecodeFunctionKind::Script)
    .with_register_counts(1, 0)
    .with_constants(vec![ConstantValue::Builtin(
        lyng_js_types::BuiltinFunctionId::from_raw(9).unwrap(),
    )])
    .with_instructions(vec![
        Instruction::abx(Opcode::LoadConst, 0, 0),
        Instruction::ax(Opcode::Return, 0),
    ]);
    let unit = CompiledScriptUnit::new(SourceId::new(14), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env()),
        Err(VmError::UnsupportedConstant {
            code: installed.code(),
            index: 0,
            constant: ConstantValue::Builtin(
                lyng_js_types::BuiltinFunctionId::from_raw(9).unwrap()
            ),
        })
    );
}

#[test]
fn load_const_supports_reserved_internal_builtin_constants() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        None,
        ArgumentsMode::None,
    )
    .with_kind(BytecodeFunctionKind::Script)
    .with_register_counts(1, 0)
    .with_constants(vec![ConstantValue::Builtin(
        internal_function_call_builtin(),
    )])
    .with_instructions(vec![
        Instruction::abx(Opcode::LoadConst, 0, 0),
        Instruction::ax(Opcode::Return, 0),
    ]);
    let unit = CompiledScriptUnit::new(SourceId::new(141), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    let builtin = result
        .as_object_ref()
        .expect("reserved internal builtin constants should resolve to builtin objects");
    let function_data = agent
        .objects()
        .function_data(builtin)
        .expect("builtin constant should resolve to a callable object");

    assert_eq!(
        function_data.entry(),
        Some(FunctionEntryIdentity::Native(NativeFunctionId::builtin(
            internal_function_call_builtin()
        )))
    );
}

#[test]
fn load_const_supports_phase5_public_builtin_constants() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        None,
        ArgumentsMode::None,
    )
    .with_kind(BytecodeFunctionKind::Script)
    .with_register_counts(1, 0)
    .with_constants(vec![ConstantValue::Builtin(symbol_builtin())])
    .with_instructions(vec![
        Instruction::abx(Opcode::LoadConst, 0, 0),
        Instruction::ax(Opcode::Return, 0),
    ]);
    let unit = CompiledScriptUnit::new(SourceId::new(142), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    let builtin = result
        .as_object_ref()
        .expect("public builtin constants should resolve to builtin objects");
    let function_data = agent
        .objects()
        .function_data(builtin)
        .expect("builtin constant should resolve to a callable object");

    assert_eq!(
        function_data.entry(),
        Some(FunctionEntryIdentity::Native(NativeFunctionId::builtin(
            symbol_builtin(),
        )))
    );
}

#[test]
fn load_const_supports_phase5_function_builtin_constants() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        None,
        ArgumentsMode::None,
    )
    .with_kind(BytecodeFunctionKind::Script)
    .with_register_counts(1, 0)
    .with_constants(vec![ConstantValue::Builtin(function_builtin())])
    .with_instructions(vec![
        Instruction::abx(Opcode::LoadConst, 0, 0),
        Instruction::ax(Opcode::Return, 0),
    ]);
    let unit = CompiledScriptUnit::new(SourceId::new(143), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    let builtin = result
        .as_object_ref()
        .expect("Function builtin constants should resolve to callable objects");
    let function_data = agent
        .objects()
        .function_data(builtin)
        .expect("Function builtin constant should resolve to a function object");

    assert_eq!(
        function_data.entry(),
        Some(FunctionEntryIdentity::Native(NativeFunctionId::builtin(
            function_builtin(),
        )))
    );
}

#[test]
fn symbol_global_dispatches_through_the_shared_builtins_bridge() {
    let unit = compile_test_unit(
        144,
        r#"
            Symbol("dispatch-bridge");
        "#,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    let symbol = result
        .as_symbol_ref()
        .expect("Symbol() should keep returning runtime symbols through the dispatch bridge");
    let description = agent
        .heap()
        .view()
        .symbol_view(symbol)
        .expect("symbol result should be live")
        .description()
        .expect("symbol description should be stored");

    assert_eq!(
        decode_string(&agent.heap().view().string_view(description).unwrap()),
        "dispatch-bridge"
    );
}

#[test]
fn symbol_constructor_exposes_disposal_well_known_symbols() {
    let unit = compile_test_unit(
        145,
        r#"
            let dispose = Object.getOwnPropertyDescriptor(Symbol, "dispose");
            let asyncDispose = Object.getOwnPropertyDescriptor(Symbol, "asyncDispose");
            (typeof Symbol.dispose === "symbol" ? 1 : 0)
                + (typeof Symbol.asyncDispose === "symbol" ? 2 : 0)
                + (Symbol.dispose !== Symbol.asyncDispose ? 4 : 0)
                + (dispose && !dispose.writable && !dispose.enumerable && !dispose.configurable ? 8 : 0)
                + (asyncDispose && !asyncDispose.writable && !asyncDispose.enumerable && !asyncDispose.configurable ? 16 : 0)
                + (String(Symbol.dispose) === "Symbol(Symbol.dispose)" ? 32 : 0)
                + (String(Symbol.asyncDispose) === "Symbol(Symbol.asyncDispose)" ? 64 : 0);
        "#,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn function_builtins_dispatch_through_the_shared_builtins_bridge() {
    let unit = compile_test_unit(
        146,
        r#"
            Function("return 9;").call(undefined);
        "#,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(9));
}

#[test]
fn function_call_builtin_rebinds_nested_targets_without_frame_leaks() {
    let unit = compile_test_unit(
        147,
        r"
            function Base(left, right) {
                this.total = left + right;
            }

            function Sub(left, right) {
                Base.call(this, left, right);
            }

            var object = new Sub(3, 4);
            var nested = Function.prototype.call.call(
                function (value) { return this.total + value; },
                object,
                5
            );
            object.total + nested;
        ",
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(19));
    assert!(vm.frames().is_empty());
    assert!(vm.register_stack().is_empty());
    assert!(agent.current_execution_context().is_none());
}

#[test]
fn array_push_preserves_index_setter_observability() {
    let unit = compile_test_unit(
        148,
        r#"
            var observed = 0;
            Object.defineProperty(Array.prototype, "0", {
                set: function (value) { observed = value; },
                configurable: true
            });
            var array = [];
            var length = array.push(7);
            var hasOwn = Object.prototype.hasOwnProperty.call(array, "0");
            delete Array.prototype[0];
            observed === 7 && length === 1 && array.length === 1 && !hasOwn;
        "#,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn for_in_state_is_cleared_when_return_exits_loop_body() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = AtomId::from_raw(71);
    let value_name = AtomId::from_raw(72);

    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    assert!(ordinary_create_data_property(
        agent,
        object,
        PropertyKey::from_atom(value_name),
        Value::from_smi(1),
        AllocationLifetime::Default,
    )
    .unwrap());
    assert!(ordinary_create_data_property(
        agent,
        realm.global_object(),
        PropertyKey::from_atom(source_name),
        Value::from_object_ref(object),
        AllocationLifetime::Default,
    )
    .unwrap());

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(4)
        .expect("test bytecode registers should allocate");
    let object_name = builder
        .add_constant(ConstantValue::Atom(source_name))
        .expect("test bytecode constant should build");
    builder
        .emit_abx(Opcode::LoadGlobal, 0, object_name)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::CreateForIn, 1, 0, 0)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::AdvanceForIn, 1, 2, 3)
        .expect("test bytecode should build");
    let done = builder
        .emit_cond_jump_placeholder(Opcode::JumpIfTrue, 3)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::ReturnUndefined, 0)
        .expect("test bytecode should build");
    let close_offset = builder
        .current_offset()
        .expect("test bytecode offset should build");
    builder
        .patch_jump_to(done, close_offset)
        .expect("test bytecode jump should patch");
    builder
        .emit_abx(Opcode::CloseForIn, 1, 0)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::ReturnUndefined, 0)
        .expect("test bytecode should build");

    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(15), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::undefined()
    );
    assert_eq!(vm.active_for_in_enumerators(), 0);
}

#[test]
fn throw_transfers_control_to_matching_catch_handler() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(2)
        .expect("test bytecode registers should allocate");
    builder
        .emit_abx(Opcode::LoadSmi, 0, 13)
        .expect("test bytecode should build");
    let protected_end = builder
        .current_offset()
        .expect("test bytecode offset should build")
        + 1;
    builder
        .emit_ax(Opcode::Throw, 0)
        .expect("test bytecode should build");
    let catch_entry = builder
        .current_offset()
        .expect("test bytecode offset should build");
    builder
        .emit_ax(Opcode::EnterHandler, 0)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::LoadException, 1)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::LeaveHandler, 0)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 1)
        .expect("test bytecode should build");
    builder
        .add_exception_handler(ExceptionHandler::new(
            0,
            protected_end,
            catch_entry,
            ExceptionHandlerKind::Catch,
            builder.header().register_count(),
            Some(1),
        ))
        .expect("test bytecode handler should build");

    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(16), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(13));
}

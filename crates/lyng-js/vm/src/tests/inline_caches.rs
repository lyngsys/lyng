use super::support::*;

#[test]
fn load_env_slot_throws_for_uninitialized_lexicals() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let layout = agent.alloc_environment_layout(EnvironmentLayout::new(
        EnvironmentLayoutKind::Declarative,
        [EnvironmentBindingLayout::new(
            Some(AtomId::from_raw(91)),
            EnvironmentSlotFlags::mutable_lexical(),
        )],
        true,
    ));
    let lexical_env = agent
        .alloc_declarative_environment(None, layout, AllocationLifetime::Default)
        .expect("declarative environment should allocate");

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(30).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(1)
        .expect("test bytecode registers should allocate");
    builder
        .emit_abx(Opcode::LoadEnvSlot, 0, 0)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 0)
        .expect("test bytecode should build");
    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(24), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm.evaluate_installed(agent, installed, lexical_env, lexical_env);

    assert!(matches!(result, Err(VmError::Abrupt(_))));
    assert_eq!(
        agent.environment_slot(lexical_env, 0),
        Some(Value::uninitialized_lexical())
    );
}

#[test]
fn store_env_slot_rejects_reassigning_initialized_const_bindings() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let layout = agent.alloc_environment_layout(EnvironmentLayout::new(
        EnvironmentLayoutKind::Declarative,
        [EnvironmentBindingLayout::new(
            Some(AtomId::from_raw(92)),
            EnvironmentSlotFlags::immutable_lexical(),
        )],
        true,
    ));
    let lexical_env = agent
        .alloc_declarative_environment(None, layout, AllocationLifetime::Default)
        .expect("declarative environment should allocate");
    assert!(agent.set_environment_slot(lexical_env, 0, Value::from_smi(1)));

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(31).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(1)
        .expect("test bytecode registers should allocate");
    let constant = builder
        .add_constant(ConstantValue::Smi(2))
        .expect("test bytecode constant should build");
    builder
        .emit_abx(Opcode::LoadConst, 0, constant)
        .expect("test bytecode should build");
    builder
        .emit_abx(Opcode::StoreEnvSlot, 0, 0)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 0)
        .expect("test bytecode should build");
    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(25), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm.evaluate_installed(agent, installed, lexical_env, lexical_env);

    assert!(matches!(result, Err(VmError::Abrupt(_))));
    assert_eq!(
        agent.environment_slot(lexical_env, 0),
        Some(Value::from_smi(1))
    );
}

#[test]
fn named_property_load_ic_becomes_monomorphic_for_one_shape() {
    let unit = compile_test_unit(30, "source.value;");
    let entry = unit.function(unit.entry()).unwrap();
    let slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::NamedPropertyLoad)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a named-load site");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let value_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "value"));
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
        Value::from_smi(7),
        AllocationLifetime::Default,
    )
    .unwrap());
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(7)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(7)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_js_objects::NamedPropertyCachePath::OwnData)
        ))
    );
}

#[test]
fn global_property_load_ic_becomes_monomorphic_for_global_object_data_property() {
    let unit = compile_test_unit(36, "globalValue;");
    let entry = unit.function(unit.entry()).unwrap();
    let slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::NamedPropertyLoad)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a named-load site for the global access");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let global_value_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "globalValue"));
    install_global_value(agent, &realm, global_value_name, Value::from_smi(11));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(11)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(11)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_js_objects::NamedPropertyCachePath::OwnData)
        ))
    );
}

#[test]
fn global_property_store_ic_caches_global_object_data_property() {
    let unit = compile_test_unit(
        37,
        "var globalValue; globalValue = globalValue + 1; globalValue;",
    );
    let entry = unit.function(unit.entry()).unwrap();
    let slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::NamedPropertyStore)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a named-store site for the global access");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let global_value_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "globalValue"));
    install_global_value(agent, &realm, global_value_name, Value::from_smi(0));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(1)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(2)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_js_objects::NamedPropertyCachePath::OwnData)
        ))
    );
}

#[test]
fn named_property_load_ic_grows_polymorphic_and_then_megamorphic() {
    let unit = compile_test_unit(31, "source.value;");
    let entry = unit.function(unit.entry()).unwrap();
    let value_atom = unit_atom(&unit, "value");
    let slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| {
            descriptor.kind() == FeedbackSiteKind::NamedPropertyLoad
                && descriptor.metadata() == FeedbackSiteMetadata::NamedProperty(value_atom)
        })
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a named-load site for source.value");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let value_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "value"));

    let mut sources = Vec::new();
    for index in 0..6 {
        let object = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                AllocationLifetime::Default,
            )
        });
        for extra in 0..index {
            assert!(ordinary_create_data_property(
                agent,
                object,
                PropertyKey::from_atom(AtomId::from_raw(20_000 + extra)),
                Value::from_smi(extra.cast_signed()),
                AllocationLifetime::Default,
            )
            .unwrap());
        }
        assert!(ordinary_create_data_property(
            agent,
            object,
            PropertyKey::from_atom(value_name),
            Value::from_smi(index.cast_signed()),
            AllocationLifetime::Default,
        )
        .unwrap());
        sources.push(object);
    }

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    for (index, object) in sources.into_iter().enumerate() {
        install_global_value(agent, &realm, source_name, Value::from_object_ref(object));
        assert_eq!(
            vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
                .unwrap(),
            Value::from_smi(i32::try_from(index).expect("test source index should fit i32"))
        );
    }

    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some(("Megamorphic", 0, None))
    );
}

#[test]
fn named_property_store_ic_caches_own_data_paths() {
    let unit = compile_test_unit(32, "source.value = 9; source.value;");
    let entry = unit.function(unit.entry()).unwrap();
    let slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::NamedPropertyStore)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a named-store site");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let value_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "value"));
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
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(9)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(9)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_js_objects::NamedPropertyCachePath::OwnData)
        ))
    );
}

#[test]
fn keyed_named_atom_ic_becomes_monomorphic() {
    let unit = compile_test_unit(33, "source[\"value\"];");
    let entry = unit.function(unit.entry()).unwrap();
    let slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::KeyedPropertyAccess)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a keyed-access site");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let value_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "value"));
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
        Value::from_smi(4),
        AllocationLifetime::Default,
    )
    .unwrap());
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(4)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(4)
    );
    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Monomorphic", Some("NamedAtom"), 1))
    );
}

#[test]
fn keyed_dense_index_sites_fall_back_to_megamorphic_classification() {
    let unit = compile_test_unit(34, "let index = 0; source[index];");
    let entry = unit.function(unit.entry()).unwrap();
    let slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::KeyedPropertyAccess)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a keyed-access site");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_element_capacity(1),
            AllocationLifetime::Default,
        );
        assert!(objects.set_element(
            &mut mutator,
            object,
            0,
            Value::from_smi(12),
            AllocationLifetime::Default,
        ));
        object
    });
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(12)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(12)
    );
    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Megamorphic", Some("DenseIndex"), 0))
    );
}

#[test]
fn ordinary_object_dense_index_store_uses_fast_path_without_feedback_slow_path() {
    let unit = compile_test_unit(44, "source[0] = 9;");
    let entry = unit.function(unit.entry()).unwrap();
    let slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::KeyedPropertyAccess)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a keyed-store site");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    for _ in 0..2 {
        assert_eq!(
            vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
                .unwrap(),
            Value::from_smi(9)
        );
    }

    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Uninitialized", None, 0))
    );
}

#[test]
fn ordinary_object_index_store_observes_inherited_index_setter() {
    let unit = compile_test_unit(
        45,
        r#"
        var hit = 0;
        var proto = {};
        Object.defineProperty(proto, "0", {
            set: function(value) {
                hit = value;
            }
        });
        var source = Object.create(proto);
        source[0] = 9;
        hit;
        "#,
    );
    let entry = unit.function(unit.entry()).unwrap();
    let slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::KeyedPropertyAccess)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a keyed-store site");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(9)
    );

    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Megamorphic", Some("DenseIndex"), 0))
    );
}

#[test]
fn engine_array_sparse_index_store_uses_fast_path_without_feedback_slow_path() {
    let unit = compile_test_unit(
        46,
        r"
        var source = [];
        source[32] = 7;
        source[31] = 9;
        source.length;
        ",
    );
    let entry = unit.function(unit.entry()).unwrap();
    let slots: Vec<_> = entry
        .feedback_sites()
        .iter()
        .filter(|descriptor| descriptor.kind() == FeedbackSiteKind::KeyedPropertyAccess)
        .map(|descriptor| descriptor.slot())
        .collect();
    assert_eq!(
        slots.len(),
        2,
        "entry script should contain two keyed-store sites"
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(33)
    );

    for slot in slots {
        assert_eq!(
            vm.keyed_property_cache_snapshot(installed.code(), slot),
            Some(("Uninitialized", None, 0))
        );
    }
}

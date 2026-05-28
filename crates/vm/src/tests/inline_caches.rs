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
    let result = vm
        .evaluate_installed(agent, installed, lexical_env, lexical_env)
        .run();

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
    let result = vm
        .evaluate_installed(agent, installed, lexical_env, lexical_env)
        .run();

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
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(7)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(7)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );
}

#[test]
fn named_property_load_ic_caches_prototype_data_one_hop() {
    let unit = compile_test_unit(150, "source.value;");
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
        .expect("entry script should contain a named-load site for .value");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

    let prototype = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    assert!(ordinary_create_data_property(
        agent,
        prototype,
        PropertyKey::from_atom(value_name),
        Value::from_smi(42),
        AllocationLifetime::Default,
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(prototype)),
            AllocationLifetime::Default,
        )
    });
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(42)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(42)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(NamedPropertyCachePath::PrototypeData)
        ))
    );
}

#[test]
fn named_property_load_ic_invalidates_proto_cache_on_prototype_swap() {
    let unit = compile_test_unit(151, "source.value;");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let value_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "value"));

    let prototype_a = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    assert!(ordinary_create_data_property(
        agent,
        prototype_a,
        PropertyKey::from_atom(value_name),
        Value::from_smi(11),
        AllocationLifetime::Default,
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());
    let prototype_b = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    assert!(ordinary_create_data_property(
        agent,
        prototype_b,
        PropertyKey::from_atom(value_name),
        Value::from_smi(22),
        AllocationLifetime::Default,
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(prototype_a)),
            AllocationLifetime::Default,
        )
    });
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(11)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(11)
    );

    // Capture shape before prototype swap.
    let shape_before =
        agent.with_heap_and_objects(|heap, _objects| heap.view().object(object).unwrap().shape());

    // Swap the prototype to one with a different value at the same shape.
    // The receiver epoch bump (cause = PrototypeMutation) must invalidate
    // the proto shortcut so the next access observes the new value.
    // Use Agent::set_prototype_of to trigger shape transition (PR 3).
    agent
        .set_prototype_of(
            object,
            Some(prototype_b),
            &mut NoopAdaptiveProtoLoadDispatch,
        )
        .unwrap();

    // PR 3: Verify that the shape transitioned on prototype swap.
    let shape_after =
        agent.with_heap_and_objects(|heap, _objects| heap.view().object(object).unwrap().shape());
    assert_ne!(
        shape_before, shape_after,
        "PR 3: proto swap should transition the shape"
    );

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(22)
    );
}

#[test]
fn keyed_named_property_load_ic_caches_prototype_data_one_hop() {
    let unit = compile_test_unit(152, "var k = \"value\"; source[k];");
    let entry = unit.function(unit.entry()).unwrap();
    let slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::KeyedPropertyAccess)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a keyed-load site");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let value_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "value"));

    let prototype = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    assert!(ordinary_create_data_property(
        agent,
        prototype,
        PropertyKey::from_atom(value_name),
        Value::from_smi(99),
        AllocationLifetime::Default,
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(prototype)),
            AllocationLifetime::Default,
        )
    });
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(99)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(99)
    );
    let snapshot = vm
        .keyed_property_cache_snapshot(installed.code(), slot)
        .expect("keyed cache snapshot should be populated");
    assert_eq!(snapshot.0, "Monomorphic");
    assert_eq!(snapshot.1, Some("NamedAtom"));
}

#[test]
fn named_property_load_ic_does_not_engage_proto_specialized_path_for_three_hop_chain() {
    let unit = compile_test_unit(153, "source.value;");
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
        .expect("entry script should contain a named-load site for .value");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

    let great_grandparent = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    assert!(ordinary_create_data_property(
        agent,
        great_grandparent,
        PropertyKey::from_atom(value_name),
        Value::from_smi(77),
        AllocationLifetime::Default,
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());
    let grandparent = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(great_grandparent)),
            AllocationLifetime::Default,
        )
    });
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(grandparent)),
            AllocationLifetime::Default,
        )
    });
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(77)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(77)
    );
    // The IC still records the entry as PrototypeData — but with three
    // dependencies, the proto cache handler stays NONE and the slow chain
    // services the access.
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(NamedPropertyCachePath::PrototypeData)
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
            .run()
            .unwrap(),
        Value::from_smi(11)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(11)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn global_property_load_ic_hit_avoids_semantic_slow_path() {
    let unit = compile_test_unit(544, "globalValue;");
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
            .run()
            .unwrap(),
        Value::from_smi(11)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(11)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );

    let counters = vm.opcode_counters_mut();
    counters.enable_slow_path();
    counters.reset();

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(11)
    );

    let counters = vm.opcode_counters();
    let dispatch = counters.dispatch_counts();
    let slow_path = counters
        .slow_path_counts()
        .expect("slow-path counters should be enabled");
    assert_eq!(dispatch.count(Opcode::LoadGlobal), 1);
    assert_eq!(
        slow_path.semantic(Opcode::LoadGlobal),
        0,
        "cached global-object load IC hit should avoid the semantic slow bridge"
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
            .run()
            .unwrap(),
        Value::from_smi(1)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(2)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );
}

#[test]
fn named_property_load_ic_keeps_six_shape_polymorphic_cache() {
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
                &mut NoopAdaptiveProtoLoadDispatch,
            )
            .unwrap());
        }
        assert!(ordinary_create_data_property(
            agent,
            object,
            PropertyKey::from_atom(value_name),
            Value::from_smi(index.cast_signed()),
            AllocationLifetime::Default,
            &mut NoopAdaptiveProtoLoadDispatch,
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
                .run()
                .unwrap(),
            Value::from_smi(i32::try_from(index).expect("test source index should fit i32"))
        );
    }

    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Polymorphic",
            6,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );
}

#[test]
fn named_property_load_ic_orders_polymorphic_entries_by_shape() {
    let unit = compile_test_unit(52, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

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
                PropertyKey::from_atom(AtomId::from_raw(24_000 + extra)),
                Value::from_smi(extra.cast_signed()),
                AllocationLifetime::Default,
                &mut NoopAdaptiveProtoLoadDispatch,
            )
            .unwrap());
        }
        assert!(ordinary_create_data_property(
            agent,
            object,
            PropertyKey::from_atom(value_name),
            Value::from_smi(index.cast_signed()),
            AllocationLifetime::Default,
            &mut NoopAdaptiveProtoLoadDispatch,
        )
        .unwrap());
        let shape = agent
            .objects()
            .object_header(agent.heap().view(), object)
            .expect("test object should have a header")
            .shape()
            .get();
        sources.push((shape, object, index));
    }
    sources.sort_by_key(|(shape, _, _)| std::cmp::Reverse(*shape));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    for (_, object, index) in sources {
        install_global_value(agent, &realm, source_name, Value::from_object_ref(object));
        assert_eq!(
            vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
                .run()
                .unwrap(),
            Value::from_smi(i32::try_from(index).expect("test source index should fit i32"))
        );
    }

    let status = vm
        .named_property_status(installed.code(), slot)
        .expect("source.value should expose named-property status");
    let actual_shapes = status
        .entries
        .iter()
        .map(|entry| entry.receiver_shape().get())
        .collect::<Vec<_>>();
    let mut sorted_shapes = actual_shapes.clone();
    sorted_shapes.sort_unstable();

    assert_eq!(status.state(), FeedbackInlineCacheState::Polymorphic);
    assert_eq!(actual_shapes, sorted_shapes);
}

#[test]
fn named_property_load_ic_promotes_to_megamorphic_beyond_polymorphic_capacity() {
    let unit = compile_test_unit(47, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

    let mut sources = Vec::new();
    for index in 0..10 {
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
                PropertyKey::from_atom(AtomId::from_raw(22_000 + extra)),
                Value::from_smi(extra.cast_signed()),
                AllocationLifetime::Default,
                &mut NoopAdaptiveProtoLoadDispatch,
            )
            .unwrap());
        }
        assert!(ordinary_create_data_property(
            agent,
            object,
            PropertyKey::from_atom(value_name),
            Value::from_smi(index.cast_signed()),
            AllocationLifetime::Default,
            &mut NoopAdaptiveProtoLoadDispatch,
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
                .run()
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
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(9)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(9)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn named_property_store_ic_hit_avoids_semantic_slow_path() {
    let unit = compile_test_unit(543, "source.value = 9; source.value;");
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
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(9)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );

    let counters = vm.opcode_counters_mut();
    counters.enable_slow_path();
    counters.reset();

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(9)
    );

    let counters = vm.opcode_counters();
    let dispatch = counters.dispatch_counts();
    let slow_path = counters
        .slow_path_counts()
        .expect("slow-path counters should be enabled");
    assert_eq!(dispatch.count(Opcode::AssignNamedProperty), 1);
    assert_eq!(
        slow_path.semantic(Opcode::AssignNamedProperty),
        0,
        "cached named-property store IC hit should avoid the semantic slow bridge"
    );
}

#[test]
fn named_property_store_ic_caches_absent_own_data_transitions() {
    let unit = compile_test_unit(
        154,
        r"
        function Box(value) {
            this.value = value;
        }
        var first = new Box(1);
        var second = new Box(2);
        var third = new Box(3);
        third.value;
        ",
    );
    let entry = unit.function(unit.entry()).unwrap();
    let constructor = unit
        .functions()
        .iter()
        .find(|function| function.name() == Some(unit_atom(&unit, "Box")))
        .expect("Box constructor should be lowered as a child function");
    let store_slot = constructor
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::NamedPropertyStore)
        .map(|descriptor| descriptor.slot())
        .expect("constructor should contain a named-store site");
    let constructor_child_index = entry
        .child_functions()
        .iter()
        .position(|child| *child == constructor.id())
        .and_then(|index| u32::try_from(index).ok())
        .expect("entry script should install Box as a direct child");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let constructor_code = vm
        .installed_child_code(installed.code(), constructor_child_index)
        .expect("Box constructor should have installed code");

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(3)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(constructor_code, store_slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_objects::NamedPropertyCachePath::OwnDataTransition)
        ))
    );
}

#[test]
fn absent_named_property_load_records_without_megamorphic_feedback() {
    let unit = compile_test_unit(155, "source.missing;");
    let entry = unit.function(unit.entry()).unwrap();
    let missing_atom = unit_atom(&unit, "missing");
    let slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| {
            descriptor.kind() == FeedbackSiteKind::NamedPropertyLoad
                && descriptor.metadata() == FeedbackSiteMetadata::NamedProperty(missing_atom)
        })
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a named-load site for source.missing");

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
                .run()
                .unwrap(),
            Value::undefined()
        );
    }

    // Phase D.4.2: the absent path now uses the generic `record_feedback_slot`
    // warmup ping — no PropertyIcState is lazily created. The IC slot stays
    // absent (and therefore cannot be promoted to Megamorphic, which is the
    // contract this test protects).
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        None
    );
    assert_eq!(vm.feedback_execution_count(installed.code(), slot), None);
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
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(4)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(4)
    );
    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Monomorphic", Some("NamedAtom"), 1))
    );
}

#[test]
fn keyed_named_atom_ic_keeps_six_shape_polymorphic_cache() {
    let unit = compile_test_unit(48, "source[\"value\"];");
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
                PropertyKey::from_atom(AtomId::from_raw(23_000 + extra)),
                Value::from_smi(extra.cast_signed()),
                AllocationLifetime::Default,
                &mut NoopAdaptiveProtoLoadDispatch,
            )
            .unwrap());
        }
        assert!(ordinary_create_data_property(
            agent,
            object,
            PropertyKey::from_atom(value_name),
            Value::from_smi(index.cast_signed()),
            AllocationLifetime::Default,
            &mut NoopAdaptiveProtoLoadDispatch,
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
                .run()
                .unwrap(),
            Value::from_smi(i32::try_from(index).expect("test source index should fit i32"))
        );
    }

    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Polymorphic", Some("NamedAtom"), 6))
    );
}

#[test]
fn keyed_named_atom_ic_orders_polymorphic_entries_by_shape() {
    let unit = compile_test_unit(53, "source[\"value\"];");
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
                PropertyKey::from_atom(AtomId::from_raw(25_000 + extra)),
                Value::from_smi(extra.cast_signed()),
                AllocationLifetime::Default,
                &mut NoopAdaptiveProtoLoadDispatch,
            )
            .unwrap());
        }
        assert!(ordinary_create_data_property(
            agent,
            object,
            PropertyKey::from_atom(value_name),
            Value::from_smi(index.cast_signed()),
            AllocationLifetime::Default,
            &mut NoopAdaptiveProtoLoadDispatch,
        )
        .unwrap());
        let shape = agent
            .objects()
            .object_header(agent.heap().view(), object)
            .expect("test object should have a header")
            .shape()
            .get();
        sources.push((shape, object, index));
    }
    sources.sort_by_key(|(shape, _, _)| std::cmp::Reverse(*shape));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    for (_, object, index) in sources {
        install_global_value(agent, &realm, source_name, Value::from_object_ref(object));
        assert_eq!(
            vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
                .run()
                .unwrap(),
            Value::from_smi(i32::try_from(index).expect("test source index should fit i32"))
        );
    }

    let status = vm
        .keyed_property_status(installed.code(), slot)
        .expect("source[\"value\"] should expose keyed-property status");
    let actual_shapes = status
        .named_entries
        .iter()
        .map(|entry| entry.receiver_shape().get())
        .collect::<Vec<_>>();
    let mut sorted_shapes = actual_shapes.clone();
    sorted_shapes.sort_unstable();

    assert_eq!(status.state(), FeedbackInlineCacheState::Polymorphic);
    assert_eq!(
        status.family(),
        Some(FeedbackKeyedPropertyFamily::NamedAtom)
    );
    assert_eq!(actual_shapes, sorted_shapes);
}

#[test]
fn keyed_dense_index_load_site_caches_dense_shape() {
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
            .run()
            .unwrap(),
        Value::from_smi(12)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(12)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(12)
    );
    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Monomorphic", Some("DenseIndex"), 1))
    );
}

#[test]
fn keyed_dense_index_load_cache_tracks_shape_changes_polymorphically() {
    let unit = compile_test_unit(49, "let index = 0; source[index];");
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
    let extra_name = agent.atoms_mut().intern_collectible("extra");
    let first = agent.with_heap_and_objects(|heap, objects| {
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
            Value::from_smi(3),
            AllocationLifetime::Default,
        ));
        object
    });
    let second = agent.with_heap_and_objects(|heap, objects| {
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
            Value::from_smi(5),
            AllocationLifetime::Default,
        ));
        object
    });
    assert!(ordinary_create_data_property(
        agent,
        second,
        PropertyKey::from_atom(extra_name),
        Value::from_smi(1),
        AllocationLifetime::Default,
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    for (object, expected) in [
        (first, Value::from_smi(3)),
        (second, Value::from_smi(5)),
        (first, Value::from_smi(3)),
        (second, Value::from_smi(5)),
    ] {
        install_global_value(agent, &realm, source_name, Value::from_object_ref(object));
        assert_eq!(
            vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
                .run()
                .unwrap(),
            expected
        );
    }

    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Polymorphic", Some("DenseIndex"), 2))
    );
}

#[test]
fn keyed_dense_index_cache_falls_back_after_sparse_transition() {
    let unit = compile_test_unit(50, "let index = 0; source[index];");
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
    for _ in 0..3 {
        assert_eq!(
            vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
                .run()
                .unwrap(),
            Value::from_smi(12)
        );
    }
    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Monomorphic", Some("DenseIndex"), 1))
    );

    agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        assert!(objects.set_element(
            &mut mutator,
            object,
            32,
            Value::from_smi(32),
            AllocationLifetime::Default,
        ));
    });
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(12)
    );
    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Megamorphic", Some("DenseIndex"), 0))
    );
}

#[test]
fn mixed_named_and_dense_index_keyed_site_promotes_to_generic() {
    let unit = compile_test_unit(51, "source[key];");
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
    let key_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "key"));
    let value_name = agent.atoms_mut().intern_collectible("value");
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
    assert!(ordinary_create_data_property(
        agent,
        object,
        PropertyKey::from_atom(value_name),
        Value::from_smi(44),
        AllocationLifetime::Default,
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));
    install_global_value(agent, &realm, key_name, Value::from_smi(0));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    for _ in 0..3 {
        assert_eq!(
            vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
                .run()
                .unwrap(),
            Value::from_smi(12)
        );
    }
    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Monomorphic", Some("DenseIndex"), 1))
    );

    let key_string = agent.alloc_runtime_string("value", None, AllocationLifetime::Default);
    install_global_value(agent, &realm, key_name, Value::from_string_ref(key_string));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(44)
    );
    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Megamorphic", Some("Generic"), 0))
    );
}

#[test]
fn ordinary_object_dense_index_store_uses_specialized_path_without_feedback_slow_path() {
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
                .run()
                .unwrap(),
            Value::from_smi(9)
        );
    }

    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Monomorphic", Some("DenseIndex"), 1))
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
            .run()
            .unwrap(),
        Value::from_smi(9)
    );

    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Megamorphic", Some("DenseIndex"), 0))
    );
}

#[test]
fn engine_array_existing_index_store_skips_prototype_setter_scan() {
    let unit = compile_test_unit(
        48,
        r#"
        var hit = 0;
        var proto = {};
        Object.defineProperty(proto, "0", {
            set: function(value) {
                hit = value;
            }
        });
        var source = [1];
        Object.setPrototypeOf(source, proto);
        source[0] = 9;
        hit + source[0];
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
            .run()
            .unwrap(),
        Value::from_smi(9)
    );

    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Monomorphic", Some("DenseIndex"), 1))
    );
}

#[test]
fn engine_array_sparse_index_store_uses_specialized_path_without_feedback_slow_path() {
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
            .run()
            .unwrap(),
        Value::from_smi(33)
    );

    for slot in slots {
        assert_eq!(
            vm.keyed_property_cache_snapshot(installed.code(), slot),
            Some(("Megamorphic", Some("DenseIndex"), 0))
        );
    }
}

// -----------------------------------------------------------------------------
// Phase 3f: polymorphic-OwnData inline IC shortcut
// -----------------------------------------------------------------------------

fn make_object_with_value(
    agent: &mut lyng_env::Agent,
    root_shape: lyng_types::ShapeId,
    extra_atoms: &[u32],
    value_atom: AtomId,
    value: Value,
) -> ObjectRef {
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    for &raw_atom in extra_atoms {
        assert!(ordinary_create_data_property(
            agent,
            object,
            PropertyKey::from_atom(AtomId::from_raw(raw_atom)),
            Value::from_smi(0),
            AllocationLifetime::Default,
            &mut NoopAdaptiveProtoLoadDispatch,
        )
        .unwrap());
    }
    assert!(ordinary_create_data_property(
        agent,
        object,
        PropertyKey::from_atom(value_atom),
        value,
        AllocationLifetime::Default,
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());
    object
}

#[test]
fn named_property_load_ic_polymorphic_own_data_handlers_load_returns_value_for_two_shapes() {
    // After the polymorphic transition the inline shortcut walks the
    // `polymorphic_own_data_handlers` sidecar (Phase 3f). Two distinct shapes both
    // resolve to `.value`; each evaluation must return its receiver's
    // value, not the other shape's.
    let unit = compile_test_unit(540, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

    let shape_a = make_object_with_value(agent, root_shape, &[], value_name, Value::from_smi(10));
    let shape_b = make_object_with_value(
        agent,
        root_shape,
        &[26_000],
        value_name,
        Value::from_smi(20),
    );

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    // Prime: install A then B to bring the IC to Polymorphic-2.
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_a));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(10)
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_b));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(20)
    );
    // Re-evaluate both shapes after the polymorphic transition: each
    // must come out of the inline sidecar with the right value, not the
    // other shape's value.
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_a));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(10)
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_b));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(20)
    );

    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Polymorphic",
            2,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn named_property_load_ic_hit_avoids_semantic_slow_path() {
    let unit = compile_test_unit(542, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);
    let object = make_object_with_value(agent, root_shape, &[], value_name, Value::from_smi(33));
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(33)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );

    let counters = vm.opcode_counters_mut();
    counters.enable_slow_path();
    counters.reset();

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(33)
    );

    let counters = vm.opcode_counters();
    let dispatch = counters.dispatch_counts();
    let slow_path = counters
        .slow_path_counts()
        .expect("slow-path counters should be enabled");
    assert_eq!(dispatch.count(Opcode::GetNamedProperty), 1);
    assert_eq!(
        slow_path.semantic(Opcode::GetNamedProperty),
        0,
        "cached named-property IC hit should avoid the semantic slow bridge"
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn named_property_load_polymorphic_ic_hits_avoid_semantic_slow_path() {
    // Polymorphic OwnData inline-slot two-shape walk. After priming the IC
    // with two distinct receiver shapes, both should resolve through the
    // asm-DSL `.try_poly` label (FeedbackEntry mode = 4) and avoid the
    // semantic slow bridge.
    let unit = compile_test_unit(549, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);
    let shape_a = make_object_with_value(agent, root_shape, &[], value_name, Value::from_smi(10));
    let shape_b = make_object_with_value(
        agent,
        root_shape,
        &[26_000],
        value_name,
        Value::from_smi(20),
    );

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    // Prime the IC: shape A then shape B → Polymorphic-2.
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_a));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(10)
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_b));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(20)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Polymorphic",
            2,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );

    let counters = vm.opcode_counters_mut();
    counters.enable_slow_path();
    counters.reset();

    // Slot 0 path: receiver shape matches the first cached entry.
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_a));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(10)
    );
    // Slot 1 path: receiver shape matches the second cached entry.
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_b));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(20)
    );

    let counters = vm.opcode_counters();
    let dispatch = counters.dispatch_counts();
    let slow_path = counters
        .slow_path_counts()
        .expect("slow-path counters should be enabled");
    assert_eq!(dispatch.count(Opcode::GetNamedProperty), 2);
    assert_eq!(
        slow_path.semantic(Opcode::GetNamedProperty),
        0,
        "cached polymorphic named-property IC hits should avoid the semantic slow bridge"
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn named_property_load_outline_ic_hit_avoids_semantic_slow_path() {
    let unit = compile_test_unit(548, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);
    let object = make_object_with_value(
        agent,
        root_shape,
        &[1001, 1002, 1003, 1004],
        value_name,
        Value::from_smi(34),
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(34)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );

    let counters = vm.opcode_counters_mut();
    counters.enable_slow_path();
    counters.reset();

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(34)
    );

    let counters = vm.opcode_counters();
    let dispatch = counters.dispatch_counts();
    let slow_path = counters
        .slow_path_counts()
        .expect("slow-path counters should be enabled");
    assert_eq!(dispatch.count(Opcode::GetNamedProperty), 1);
    assert_eq!(
        slow_path.semantic(Opcode::GetNamedProperty),
        0,
        "cached out-of-line named-property IC hit should avoid the semantic slow bridge"
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn named_property_load_proto_ic_hit_avoids_semantic_slow_path() {
    let unit = compile_test_unit(543, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

    let prototype = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    assert!(ordinary_create_data_property(
        agent,
        prototype,
        PropertyKey::from_atom(value_name),
        Value::from_smi(42),
        AllocationLifetime::Default,
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(prototype)),
            AllocationLifetime::Default,
        )
    });
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(42)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_objects::NamedPropertyCachePath::PrototypeData)
        ))
    );

    let counters = vm.opcode_counters_mut();
    counters.enable_slow_path();
    counters.reset();

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(42)
    );

    let counters = vm.opcode_counters();
    let dispatch = counters.dispatch_counts();
    let slow_path = counters
        .slow_path_counts()
        .expect("slow-path counters should be enabled");
    assert_eq!(dispatch.count(Opcode::GetNamedProperty), 1);
    assert_eq!(
        slow_path.semantic(Opcode::GetNamedProperty),
        0,
        "cached prototype-data named-property IC hit should avoid the semantic slow bridge"
    );
}

#[test]
fn named_property_load_ic_polymorphic_own_data_handlers_load_falls_through_beyond_poly_limit() {
    // With POLY_LIMIT=4, six distinct shapes leave entries 4..6 reachable
    // only via the slow chain. Semantic correctness must hold regardless
    // of which path serves each evaluation. This is a tighter version of
    // `named_property_load_ic_keeps_six_shape_polymorphic_cache` that
    // also probes each shape after the cache reaches its final state.
    let unit = compile_test_unit(541, "source.value;");
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
        .expect("entry script should contain a named-load site");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

    let sources: Vec<_> = (0..6)
        .map(|index| {
            let extras: Vec<u32> = (0..index).map(|extra| 26_500 + extra).collect();
            let object = make_object_with_value(
                agent,
                root_shape,
                &extras,
                value_name,
                Value::from_smi(i32::try_from(index).expect("test index fits i32") + 100),
            );
            (object, index)
        })
        .collect();

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    // Prime the cache with all six shapes.
    for (object, index) in &sources {
        install_global_value(agent, &realm, source_name, Value::from_object_ref(*object));
        assert_eq!(
            vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
                .run()
                .unwrap(),
            Value::from_smi(i32::try_from(*index).expect("test index fits i32") + 100)
        );
    }
    // Re-evaluate in the same order: half come through the inline
    // sidecar, half through the slow-chain binary search. Both must
    // return the correct value.
    for (object, index) in &sources {
        install_global_value(agent, &realm, source_name, Value::from_object_ref(*object));
        assert_eq!(
            vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
                .run()
                .unwrap(),
            Value::from_smi(i32::try_from(*index).expect("test index fits i32") + 100)
        );
    }

    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Polymorphic",
            6,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );
}

#[test]
fn named_property_store_ic_polymorphic_own_data_handlers_store_writes_correct_slot() {
    // Phase 3f store-side polymorphic shortcut: two distinct shapes
    // both have a writable `.value` slot. After the polymorphic
    // transition, writes through each shape must land in that shape's
    // slot (not the other shape's). The load that follows must read the
    // freshly written value, not a stale cached one.
    let unit = compile_test_unit(542, "source.value = 99; source.value;");
    let entry = unit.function(unit.entry()).unwrap();
    let store_slot = entry
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

    let shape_a = make_object_with_value(agent, root_shape, &[], value_name, Value::from_smi(1));
    let shape_b =
        make_object_with_value(agent, root_shape, &[27_000], value_name, Value::from_smi(2));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    // Prime polymorphic-2.
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_a));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(99)
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_b));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(99)
    );

    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), store_slot),
        Some((
            "Polymorphic",
            2,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );

    // Both shapes still hit their own slot after the polymorphic
    // transition. Each shape's `.value` should reflect the most recent
    // store performed against that shape.
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_a));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(99)
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_b));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(99)
    );
}

#[test]
fn named_property_load_ic_polymorphic_own_data_handlers_load_invalidates_on_prototype_swap() {
    // After the polymorphic transition, mutating a cached receiver's
    // prototype bumps that receiver's invalidation_epoch even though
    // the shape ID stays the same. The polymorphic_own_data_handlers hit must miss
    // on the affected receiver and fall through to the slow chain,
    // which re-resolves the own-data slot (still correct, just no
    // longer eligible for the cache hit until the cache refreshes).
    let unit = compile_test_unit(543, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

    let shape_a = make_object_with_value(agent, root_shape, &[], value_name, Value::from_smi(10));
    let shape_b = make_object_with_value(
        agent,
        root_shape,
        &[27_500],
        value_name,
        Value::from_smi(20),
    );
    let new_prototype = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_a));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(10)
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_b));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(20)
    );

    // Swap shape_a's prototype. This bumps shape_a's invalidation
    // epoch with cause = PrototypeMutation. The polymorphic_own_data_handlers hit
    // for shape_a must now miss the epoch check and fall through.
    agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects
            .set_prototype_of(&mut mutator, shape_a, Some(new_prototype))
            .unwrap()
    });

    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_a));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(10),
        "own-data slot still resolves correctly through slow chain after epoch bump"
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_b));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(20),
        "shape_b's polymorphic_own_data_handlers hit is unaffected by shape_a's epoch bump"
    );

    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Polymorphic",
            2,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );
}

#[test]
fn keyed_named_property_load_ic_polymorphic_own_data_handlers_load_returns_value_for_two_shapes() {
    // Phase 3f keyed-named polymorphic shortcut: a keyed access
    // `source[key]` (with key = "value") on two distinct receiver
    // shapes. After the polymorphic transition the keyed sidecar walk
    // must match both atom AND receiver shape per entry.
    let unit = compile_test_unit(544, "source['value'];");
    let entry = unit.function(unit.entry()).unwrap();
    let slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::KeyedPropertyAccess)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a keyed-property access site");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let value_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "value"));

    let shape_a = make_object_with_value(agent, root_shape, &[], value_name, Value::from_smi(70));
    let shape_b = make_object_with_value(
        agent,
        root_shape,
        &[28_000],
        value_name,
        Value::from_smi(80),
    );

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_a));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(70)
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_b));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(80)
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_a));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(70)
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_b));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(80)
    );

    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Polymorphic", Some("NamedAtom"), 2))
    );
}

// Spec 2 Phase A — A.4: an orphan `AdaptiveProtoLoad` watchpoint from a
// prior install no-ops when the IC slot's current generation no longer
// matches the watchpoint's recorded generation. The dispatch path inside
// `Agent::fire_watchpoints_for_shape` routes the fire to
// `Vm::clear_ic_slot_if_generation_matches`, which is the guard.
#[test]
fn adaptive_proto_load_orphan_watchpoint_noops_on_generation_mismatch() {
    let unit = compile_test_unit(20_400, "source.value;");
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
        .expect("entry script should contain a named-load site for .value");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

    let prototype = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    assert!(ordinary_create_data_property(
        agent,
        prototype,
        PropertyKey::from_atom(value_name),
        Value::from_smi(42),
        AllocationLifetime::Default,
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(prototype)),
            AllocationLifetime::Default,
        )
    });
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    // Capture the prototype's shape now — before any cache install. The
    // slow path will register an `AdaptiveProtoLoad` watchpoint on this
    // shape during the first IC install.
    let proto_shape = agent
        .with_heap_and_objects(|heap, _objects| heap.view().object(prototype).unwrap().shape())
        .expect("prototype object should have a shape");

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    // Two evaluations — the first walks up the feedback warmup counter
    // (FEEDBACK_ALLOCATION_THRESHOLD); the second allocates the sites
    // array and runs the slow path, which installs the proto-cache IC
    // entry. Slow path bumps the IC slot's generation 0 → 1 and
    // registers a watchpoint on `proto_shape` with `generation = 1`.
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(42)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(42)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(NamedPropertyCachePath::PrototypeData)
        ))
    );
    assert_eq!(
        vm.named_property_generation_snapshot(installed.code(), slot),
        Some(("Monomorphic", 1))
    );

    // Bump the IC slot's generation manually — simulates any path that
    // re-installs (e.g. a second polymorphic chain-shape registration)
    // without firing the original watchpoint. The orphan watchpoint
    // registered on `proto_shape` still carries `generation = 1`, but
    // the slot now carries `generation = 2`.
    let bumped = <Vm as AdaptiveProtoLoadDispatch>::bump_generation_for_install(
        &mut vm,
        installed.code(),
        slot,
    );
    assert_eq!(bumped, 2);
    assert_eq!(
        vm.named_property_generation_snapshot(installed.code(), slot),
        Some(("Monomorphic", 2))
    );

    // Fire the watchpoint on `proto_shape`. The orphan fires with
    // `generation = 1`; `clear_ic_slot_if_generation_matches` sees the
    // slot currently at `generation = 2` and no-ops.
    agent.fire_watchpoints_for_shape(proto_shape, &mut vm);

    // The slot is still present and still at generation 2 — the orphan
    // watchpoint did not clear it.
    assert!(vm.named_property_slot_is_present(installed.code(), slot));
    assert_eq!(
        vm.named_property_generation_snapshot(installed.code(), slot),
        Some(("Monomorphic", 2))
    );
}

// Spec 2 Phase A — A.5: when the slow path attempts to install a
// proto-cache IC entry but some chain shape is already `Invalidated`,
// the install is abandoned: the IC slot is not committed (the
// `observe_slow_path` call is skipped), and a subsequent re-evaluation
// retries the install. The chain shape's `WatchpointSet` cannot accept
// new watchpoints once invalidated, so the next install registers on
// the *post-transition* prototype shape — not the dead one.
#[test]
fn adaptive_proto_load_register_on_invalidated_chain_abandons_install() {
    let unit = compile_test_unit(20_500, "source.value;");
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
        .expect("entry script should contain a named-load site for .value");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

    let prototype = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    assert!(ordinary_create_data_property(
        agent,
        prototype,
        PropertyKey::from_atom(value_name),
        Value::from_smi(123),
        AllocationLifetime::Default,
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(prototype)),
            AllocationLifetime::Default,
        )
    });
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    // Force the prototype's current shape into the `Invalidated` watchpoint
    // state *before* the IC install. The slow path will then attempt
    // `register` on this set, get `Err(Invalidated)`, and abandon the
    // install.
    let proto_shape = agent
        .with_heap_and_objects(|heap, _objects| heap.view().object(prototype).unwrap().shape())
        .expect("prototype object should have a shape");
    agent
        .objects_mut()
        .watchpoint_set_mut(proto_shape)
        .register(Watchpoint::ShapeInvalidation {
            observer: ShapeInvalidationObserver::Recording { token: 0xdead },
        })
        .unwrap();
    agent.fire_watchpoints_for_shape(proto_shape, &mut NoopAdaptiveProtoLoadDispatch);
    assert_eq!(
        agent
            .objects()
            .watchpoint_sets_inspect(proto_shape)
            .map(|set| set.state()),
        Some(WatchpointState::Invalidated),
        "test precondition: proto shape's watchpoint set must be invalidated"
    );

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    // Two evaluations: the first warms the slot, the second runs the
    // slow path. The IC entry is a PrototypeData plan;
    // `register_adaptive_proto_load_for_chain` walks the chain, hits the
    // Invalidated set for `proto_shape`, and returns `Err`.
    // `record_named_property_cache_entry` skips `observe_slow_path`, so
    // the IC slot is never committed. The script still returns the
    // correct value (the slow load itself succeeded).
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(123)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(123)
    );

    // The IC slot stays absent — confirming the abandon. (Generation is
    // not observable here because the slot was never committed; the
    // `named_property_slot_is_present` probe is the load-bearing
    // assertion.)
    assert!(
        !vm.named_property_slot_is_present(installed.code(), slot),
        "abandon-on-invalidated must leave the IC slot uncommitted"
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        None
    );
}

// Spec 2 Phase A — A.2: mutating the holder of a proto-cached property
// fires the `AdaptiveProtoLoad` watchpoint registered at IC install time,
// which clears the IC slot. The next read re-caches against the new holder
// shape and still returns the correct value.
//
// Chain: obj → proto. `value` lives on `proto`.
// After the IC installs as Monomorphic PrototypeData, adding a property
// to `proto` fires the watchpoint on proto's shape, clearing the slot.
#[test]
fn proto_chain_holder_mutation_clears_ic_slot() {
    let unit = compile_test_unit(20_600, "source.value;");
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
        .expect("entry script should contain a named-load site for .value");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let value_name = unit_runtime_atom(agent, &unit, value_atom);
    let extra_name = agent.atoms_mut().intern_collectible("_extra");

    // Build obj → proto. `value` lives on `proto`.
    let proto = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    assert!(ordinary_create_data_property(
        agent,
        proto,
        PropertyKey::from_atom(value_name),
        Value::from_smi(42),
        AllocationLifetime::Default,
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());
    let obj = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(proto)),
            AllocationLifetime::Default,
        )
    });
    install_global_value(agent, &realm, source_name, Value::from_object_ref(obj));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    // Two evaluations to warm the counter and install the IC entry.
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(42)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(42)
    );

    // IC should be installed as Monomorphic PrototypeData.
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_objects::NamedPropertyCachePath::PrototypeData)
        ))
    );
    assert!(vm.named_property_slot_is_present(installed.code(), slot));

    // Mutate the holder: add a new property to `proto`. This fires the
    // `AdaptiveProtoLoad` watchpoint registered on proto's current shape,
    // clearing the IC slot.
    assert!(ordinary_create_data_property(
        agent,
        proto,
        PropertyKey::from_atom(extra_name),
        Value::from_smi(0),
        AllocationLifetime::Default,
        &mut vm,
    )
    .unwrap());

    // The IC slot must have been cleared by the watchpoint fire.
    assert!(
        !vm.named_property_slot_is_present(installed.code(), slot),
        "A.2: holder mutation must clear the IC slot via AdaptiveProtoLoad watchpoint"
    );

    // Re-evaluation re-installs the IC and still returns the correct value.
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(42),
        "A.2: value must remain correct after IC re-install"
    );
    assert!(
        vm.named_property_slot_is_present(installed.code(), slot),
        "A.2: IC slot must be re-installed after re-evaluation"
    );
}

// Spec 2 Phase A — A.3: mutating an intermediate prototype in a two-hop
// chain fires the `AdaptiveProtoLoad` watchpoint registered on mid's shape
// at IC install time, clearing the IC slot. The next read re-installs.
//
// Chain: obj → mid → root. `value` lives on `root`.
// The IC is serviced by the slow chain (3 dependencies). At install time
// watchpoints are registered on both `mid`'s and `root`'s shapes.
// Adding a property to `mid` transitions its shape, firing the watchpoint
// and clearing the IC slot.
#[test]
fn two_hop_chain_middle_proto_mutation_clears_ic() {
    let unit = compile_test_unit(20_700, "source.value;");
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
        .expect("entry script should contain a named-load site for .value");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let value_name = unit_runtime_atom(agent, &unit, value_atom);
    let extra_name = agent.atoms_mut().intern_collectible("_extra2");

    // Build obj → mid → root. `value` lives on `root`.
    let root = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    assert!(ordinary_create_data_property(
        agent,
        root,
        PropertyKey::from_atom(value_name),
        Value::from_smi(99),
        AllocationLifetime::Default,
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());
    let mid = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(root)),
            AllocationLifetime::Default,
        )
    });
    let obj = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(mid)),
            AllocationLifetime::Default,
        )
    });
    install_global_value(agent, &realm, source_name, Value::from_object_ref(obj));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    // Two evaluations to warm the counter and install the IC entry.
    // The 3-hop chain is serviced by the slow chain, not the proto fast path.
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(99)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(99)
    );

    // IC should be installed as Monomorphic PrototypeData (slow chain).
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_objects::NamedPropertyCachePath::PrototypeData)
        ))
    );
    assert!(vm.named_property_slot_is_present(installed.code(), slot));

    // Mutate `mid`: add a new property. This transitions mid's shape,
    // firing the `AdaptiveProtoLoad` watchpoint registered on mid's old
    // shape at IC install time and clearing the IC slot.
    assert!(ordinary_create_data_property(
        agent,
        mid,
        PropertyKey::from_atom(extra_name),
        Value::from_smi(0),
        AllocationLifetime::Default,
        &mut vm,
    )
    .unwrap());

    // The IC slot must have been cleared.
    assert!(
        !vm.named_property_slot_is_present(installed.code(), slot),
        "A.3: middle-proto mutation must clear the IC slot via AdaptiveProtoLoad watchpoint"
    );

    // Re-evaluation re-installs the IC and still returns the correct value.
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(99),
        "A.3: value must remain correct after IC re-install"
    );
    assert!(
        vm.named_property_slot_is_present(installed.code(), slot),
        "A.3: IC slot must be re-installed after re-evaluation"
    );
}

// -----------------------------------------------------------------------------
// Spec 2 Phase B — polymorphic out-of-line chain tests (B.1.6)
// -----------------------------------------------------------------------------
//
// `NamedPropertyFeedback.entries` holds at most `POLY_LIMIT` (= 2) entries
// inline. Entries beyond POLY_LIMIT (logical positions 2..8) live out-of-line
// in `Vm::polymorphic_chains`, keyed by `(CodeRef, FeedbackSlotId)`. Once a
// 9th shape is observed the IC transitions to Megamorphic and the chain is
// dropped. AdaptiveProtoLoad fires must clear both the inline slot and the
// chain entry.

// B1: After exactly two distinct receiver shapes the IC is Polymorphic with
// `entry_count == 2`. The inline POLY_LIMIT array is full — no chain entry
// should exist yet.
#[test]
fn b1_polymorphic_two_entries_stay_inline_no_map() {
    let unit = compile_test_unit(30_100, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

    let mut sources = Vec::new();
    for index in 0..2 {
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
                PropertyKey::from_atom(AtomId::from_raw(30_100 + extra)),
                Value::from_smi(extra.cast_signed()),
                AllocationLifetime::Default,
                &mut NoopAdaptiveProtoLoadDispatch,
            )
            .unwrap());
        }
        assert!(ordinary_create_data_property(
            agent,
            object,
            PropertyKey::from_atom(value_name),
            Value::from_smi(index.cast_signed()),
            AllocationLifetime::Default,
            &mut NoopAdaptiveProtoLoadDispatch,
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
                .run()
                .unwrap(),
            Value::from_smi(i32::try_from(index).expect("test source index should fit i32"))
        );
    }

    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Polymorphic",
            2,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );
    assert!(
        vm.polymorphic_chain(installed.code(), slot).is_none(),
        "B1: 2-entry polymorphic IC must keep both entries inline; no chain map entry"
    );
}

// B2: A third distinct shape spills one entry into the out-of-line chain.
// Aggregate `entry_count == 3`; chain length is exactly 1.
#[test]
fn b2_polymorphic_third_entry_creates_chain_entry() {
    let unit = compile_test_unit(30_200, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

    let mut sources = Vec::new();
    for index in 0..3 {
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
                PropertyKey::from_atom(AtomId::from_raw(30_200 + extra)),
                Value::from_smi(extra.cast_signed()),
                AllocationLifetime::Default,
                &mut NoopAdaptiveProtoLoadDispatch,
            )
            .unwrap());
        }
        assert!(ordinary_create_data_property(
            agent,
            object,
            PropertyKey::from_atom(value_name),
            Value::from_smi(index.cast_signed()),
            AllocationLifetime::Default,
            &mut NoopAdaptiveProtoLoadDispatch,
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
                .run()
                .unwrap(),
            Value::from_smi(i32::try_from(index).expect("test source index should fit i32"))
        );
    }

    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Polymorphic",
            3,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );
    let chain = vm
        .polymorphic_chain(installed.code(), slot)
        .expect("B2: third entry must create an out-of-line chain entry");
    assert_eq!(chain.len(), 1, "B2: chain holds exactly one entry");
}

// B3: A ninth distinct shape transitions the IC to Megamorphic and the chain
// entry is dropped.
#[test]
fn b3_polymorphic_ninth_entry_transitions_to_mega_and_drops_chain() {
    let unit = compile_test_unit(30_300, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

    let mut sources = Vec::new();
    for index in 0..9 {
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
                PropertyKey::from_atom(AtomId::from_raw(30_300 + extra)),
                Value::from_smi(extra.cast_signed()),
                AllocationLifetime::Default,
                &mut NoopAdaptiveProtoLoadDispatch,
            )
            .unwrap());
        }
        assert!(ordinary_create_data_property(
            agent,
            object,
            PropertyKey::from_atom(value_name),
            Value::from_smi(index.cast_signed()),
            AllocationLifetime::Default,
            &mut NoopAdaptiveProtoLoadDispatch,
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
                .run()
                .unwrap(),
            Value::from_smi(i32::try_from(index).expect("test source index should fit i32"))
        );
    }

    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some(("Megamorphic", 0, None))
    );
    assert!(
        vm.polymorphic_chain(installed.code(), slot).is_none(),
        "B3: Megamorphic transition must drop the chain map entry"
    );
}

// B4: Walk order — drive 4 distinct receiver shapes (2 inline + 2 in chain),
// each carrying a different value at the cached property; re-running the IC
// on each shape must return that shape's value, exercising both the inline
// fast path (entries 0..POLY_LIMIT) and the chain walk (POLY_LIMIT..N).
#[test]
fn b4_polymorphic_walk_returns_correct_value_for_each_shape() {
    let unit = compile_test_unit(30_400, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

    // Build 4 receivers with distinct shapes and distinct values at `.value`.
    // The `index`-shaped receiver carries `Value::from_smi(1000 + index)`.
    let mut sources = Vec::new();
    for index in 0..4 {
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
                PropertyKey::from_atom(AtomId::from_raw(30_400 + extra)),
                Value::from_smi(extra.cast_signed()),
                AllocationLifetime::Default,
                &mut NoopAdaptiveProtoLoadDispatch,
            )
            .unwrap());
        }
        assert!(ordinary_create_data_property(
            agent,
            object,
            PropertyKey::from_atom(value_name),
            Value::from_smi(1000 + i32::try_from(index).expect("index fits i32")),
            AllocationLifetime::Default,
            &mut NoopAdaptiveProtoLoadDispatch,
        )
        .unwrap());
        sources.push(object);
    }

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    // First pass: install all four shapes into the IC (2 inline + 2 chain).
    for (index, object) in sources.iter().enumerate() {
        install_global_value(agent, &realm, source_name, Value::from_object_ref(*object));
        assert_eq!(
            vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
                .run()
                .unwrap(),
            Value::from_smi(1000 + i32::try_from(index).expect("index fits i32"))
        );
    }

    // Confirm Poly state + chain population (2 inline + 2 in chain).
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Polymorphic",
            4,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );
    assert_eq!(
        vm.polymorphic_chain(installed.code(), slot)
            .map(|chain| chain.len()),
        Some(2),
        "B4: 4 entries means 2 inline + 2 chain"
    );

    // Second pass: hit each shape again, in reverse order. Each load must
    // return the correct cached value, demonstrating that both inline and
    // chain walks resolve to the right entry.
    for (index, object) in sources.iter().enumerate().rev() {
        install_global_value(agent, &realm, source_name, Value::from_object_ref(*object));
        assert_eq!(
            vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
                .run()
                .unwrap(),
            Value::from_smi(1000 + i32::try_from(index).expect("index fits i32")),
            "B4: walk must return the correct value for shape index {index}"
        );
    }
}

// B5: AdaptiveProtoLoad fire clears both inline + chain. Build a 3-receiver
// polymorphic IC where all receivers share the same prototype holding the
// cached property. Each receiver has a distinct local shape, so the IC ends
// up Polymorphic with 1 chain entry, but every entry's `holder` is the same
// shared proto. Mutating the proto fires the AdaptiveProtoLoad watchpoint on
// the proto's shape, which must clear both the inline slot *and* the chain
// map entry.
#[test]
fn b5_adaptive_proto_load_fire_clears_inline_and_chain() {
    let unit = compile_test_unit(30_500, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);
    let extra_name = agent.atoms_mut().intern_collectible("_extra_b5");

    // Single shared prototype that holds `.value`.
    let proto = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    assert!(ordinary_create_data_property(
        agent,
        proto,
        PropertyKey::from_atom(value_name),
        Value::from_smi(42),
        AllocationLifetime::Default,
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());

    // Three receivers, each with a distinct *own* shape (via differing
    // padding properties) but all sharing the same proto.
    let mut sources = Vec::new();
    for index in 0..3 {
        let object = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape).with_prototype(Some(proto)),
                AllocationLifetime::Default,
            )
        });
        for extra in 0..index {
            assert!(ordinary_create_data_property(
                agent,
                object,
                PropertyKey::from_atom(AtomId::from_raw(30_500 + extra)),
                Value::from_smi(extra.cast_signed()),
                AllocationLifetime::Default,
                &mut NoopAdaptiveProtoLoadDispatch,
            )
            .unwrap());
        }
        sources.push(object);
    }

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    // Warm + drive each receiver. The first receiver needs two evaluations
    // (warmup + slow-path install); the next two only need one each to push
    // the IC into Polymorphic with 2 inline + 1 chain entry. We just drive
    // every receiver twice — extra hits on already-cached shapes are no-ops
    // for the IC state.
    for object in &sources {
        install_global_value(agent, &realm, source_name, Value::from_object_ref(*object));
        for _ in 0..2 {
            assert_eq!(
                vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
                    .run()
                    .unwrap(),
                Value::from_smi(42)
            );
        }
    }

    // Pre-mutation: Poly with 2 inline + 1 chain entry, PrototypeData path.
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Polymorphic",
            3,
            Some(lyng_objects::NamedPropertyCachePath::PrototypeData)
        ))
    );
    assert_eq!(
        vm.polymorphic_chain(installed.code(), slot)
            .map(|chain| chain.len()),
        Some(1),
        "B5: 3-entry polymorphic IC must have 1 chain entry"
    );
    assert!(vm.named_property_slot_is_present(installed.code(), slot));

    // Mutate the shared proto: add a new property. The shape transition
    // fires the AdaptiveProtoLoad watchpoints registered on the proto's
    // pre-mutation shape — one watchpoint per cache entry (inline + chain),
    // all routed to `clear_ic_slot_if_generation_matches` which drops both
    // the inline slot and the chain map entry.
    assert!(ordinary_create_data_property(
        agent,
        proto,
        PropertyKey::from_atom(extra_name),
        Value::from_smi(0),
        AllocationLifetime::Default,
        &mut vm,
    )
    .unwrap());

    // Both the inline slot and the chain map entry must be gone.
    assert!(
        !vm.named_property_slot_is_present(installed.code(), slot),
        "B5: AdaptiveProtoLoad fire must clear the inline IC slot"
    );
    assert!(
        vm.polymorphic_chain(installed.code(), slot).is_none(),
        "B5: AdaptiveProtoLoad fire must drop the chain map entry"
    );
}

// B6: GC sweep prunes chain entries for code that is no longer live.
//
// Note: there is no public "uninstall" hook on Vm, so we test
// `prune_dead_code_polymorphic_chains` directly by passing an `is_live`
// closure that treats the installed code as dead. This is the right level to
// test: the GC call-site (`force_collect_with_active_roots`) just calls the
// same retain with the same liveness predicate.
#[test]
fn b6_polymorphic_chain_pruned_when_code_dies() {
    let unit = compile_test_unit(30_600, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

    // Build 3 receivers with distinct shapes to push the IC into Polymorphic
    // with 2 inline entries + 1 chain entry.
    let mut sources = Vec::new();
    for index in 0..3 {
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
                PropertyKey::from_atom(AtomId::from_raw(30_600 + extra)),
                Value::from_smi(extra.cast_signed()),
                AllocationLifetime::Default,
                &mut NoopAdaptiveProtoLoadDispatch,
            )
            .unwrap());
        }
        assert!(ordinary_create_data_property(
            agent,
            object,
            PropertyKey::from_atom(value_name),
            Value::from_smi(index.cast_signed()),
            AllocationLifetime::Default,
            &mut NoopAdaptiveProtoLoadDispatch,
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
                .run()
                .unwrap(),
            Value::from_smi(i32::try_from(index).expect("test source index should fit i32"))
        );
    }

    // Pre-condition: chain entry exists with 1 out-of-line entry.
    assert!(
        vm.polymorphic_chain(installed.code(), slot).is_some(),
        "B6: chain entry should exist before prune"
    );

    // Simulate code death: tell prune_dead_code_polymorphic_chains that no
    // code is live. The chain entry for `installed.code()` should be removed.
    vm.prune_dead_code_polymorphic_chains(|_code| false);

    assert!(
        vm.polymorphic_chain(installed.code(), slot).is_none(),
        "B6: chain entry must be pruned when code is dead"
    );
}

// B7: GC sweep retains chain entries for code that remains live.
//
// Mirror of B6: `prune_dead_code_polymorphic_chains` is called with an
// `is_live` predicate that keeps the installed code alive. The chain entry
// must survive.
#[test]
fn b7_polymorphic_chain_retained_when_code_lives() {
    let unit = compile_test_unit(30_700, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

    // Build 3 receivers with distinct shapes to push the IC into Polymorphic
    // with 2 inline entries + 1 chain entry.
    let mut sources = Vec::new();
    for index in 0..3 {
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
                PropertyKey::from_atom(AtomId::from_raw(30_700 + extra)),
                Value::from_smi(extra.cast_signed()),
                AllocationLifetime::Default,
                &mut NoopAdaptiveProtoLoadDispatch,
            )
            .unwrap());
        }
        assert!(ordinary_create_data_property(
            agent,
            object,
            PropertyKey::from_atom(value_name),
            Value::from_smi(index.cast_signed()),
            AllocationLifetime::Default,
            &mut NoopAdaptiveProtoLoadDispatch,
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
                .run()
                .unwrap(),
            Value::from_smi(i32::try_from(index).expect("test source index should fit i32"))
        );
    }

    // Pre-condition: chain entry exists with 1 out-of-line entry.
    assert!(
        vm.polymorphic_chain(installed.code(), slot).is_some(),
        "B7: chain entry should exist before prune"
    );

    // Simulate a GC sweep where this code is still live.
    let live_code = installed.code();
    vm.prune_dead_code_polymorphic_chains(|code| code == live_code);

    assert!(
        vm.polymorphic_chain(installed.code(), slot).is_some(),
        "B7: chain entry must be retained when code is live"
    );
}

// C4: asm IC fast path reads from MetadataTable.
//
// Strategy: warm the IC for `source.value` so the PropertyMetadata slot has
// mode=1 (OwnData-inline). Then corrupt the mode to 0 (Uninit), forcing the
// asm fast path to miss and fall to the slow path. The slow path re-observes
// the real receiver shape, writes a fresh mode=1, and returns the correct
// value. After this, mode must no longer be 0.
//
// The test is gated on `diagnostic-counters` so we can confirm that the slow path
// was triggered: exactly one `GetNamedProperty` semantic is expected after the
// corruption run.
#[cfg(feature = "diagnostic-counters")]
#[test]
fn c4_asm_ic_fast_path_reads_from_metadata_table() {
    use crate::vm::metadata_table::MetadataKind;

    let unit = compile_test_unit(45_001, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);
    let object = make_object_with_value(agent, root_shape, &[], value_name, Value::from_smi(42));
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    // First run: IC is Uninitialized; slow path fires, writes mode=1.
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(42)
    );
    // Second run: IC is Monomorphic/OwnData; asm fast path should hit.
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(42)
    );
    // Confirm the IC is now Monomorphic.
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );

    // Read the PropertyMetadata mode; it should be 1 (OwnData-inline).
    let mode_before = vm
        .metadata_table(installed.code())
        .expect("MetadataTable should exist after install")
        .property(slot.get())
        .mode;
    assert_ne!(
        mode_before, 0,
        "IC should be warm (mode != 0) before corruption"
    );

    // Corrupt the mode to 0 (Uninit). The asm fast path checks the mode byte
    // first; a 0 means "uninitialized" — no known fast-path handler — so it
    // must branch to the slow path.
    vm.metadata_table_mut(installed.code())
        .expect("MetadataTable should be mutable")
        .property_mut(slot.get())
        .mode = 0;

    // Enable slow-path counters and reset.
    let counters = vm.opcode_counters_mut();
    counters.enable_slow_path();
    counters.reset();

    // Third run: asm sees mode=0, misses, slow path fires.
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(42),
        "slow path must still return the correct value after IC corruption"
    );

    // The slow path must have been invoked exactly once.
    let slow_path = vm
        .opcode_counters()
        .slow_path_counts()
        .expect("slow-path counters should be enabled");
    assert_eq!(
        slow_path.semantic(Opcode::GetNamedProperty),
        1,
        "C4: corrupted mode=0 must cause exactly one slow-path semantic invocation"
    );

    // After the slow path, mode must be refreshed to a non-zero sane value.
    let mode_after = vm
        .metadata_table(installed.code())
        .expect("MetadataTable should still exist")
        .property(slot.get())
        .mode;
    assert_ne!(
        mode_after, 0,
        "C4: slow path must refresh PropertyMetadata.mode to a sane non-zero value"
    );

    let _ = MetadataKind::Property; // import guard — confirms Property kind is used
}

// -----------------------------------------------------------------------------
// Spec 2 Phase D.1.1 — PropertyIcState side-table tests (D1-D4)
// -----------------------------------------------------------------------------
//
// These tests verify the new `Vm::property_ic_states` side-table directly,
// independent of the legacy `FeedbackSiteState::NamedProperty` read path used
// by the snapshot API. Each test drives the IC through a specific transition
// and asserts on `vm.property_ic_state(code, slot)`.

use crate::vm::ic_state::InlineCacheState;

// D1: Uninit → Monomorphic. After installing one shape, cache_state must be
// Monomorphic and entry_count must be 1.
#[test]
fn d1_property_ic_state_uninit_to_monomorphic() {
    let unit = compile_test_unit(50_001, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

    let object = make_object_with_value(agent, root_shape, &[], value_name, Value::from_smi(1));
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    let ic_state = vm
        .property_ic_state(installed.code(), slot)
        .expect("D1: PropertyIcState should exist after first slow-path install");
    assert_eq!(
        ic_state.cache_state,
        InlineCacheState::Monomorphic,
        "D1: cache_state must be Monomorphic after one shape"
    );
    assert_eq!(
        ic_state.entry_count, 1,
        "D1: entry_count must be 1 after installing one shape"
    );
}

// D2: Monomorphic → Polymorphic. After installing two distinct shapes,
// cache_state must be Polymorphic and entry_count must be 2.
#[test]
fn d2_property_ic_state_mono_to_polymorphic() {
    let unit = compile_test_unit(50_002, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    for index in 0..2u32 {
        // Each object gets a unique extra property to get a distinct shape.
        let extra_atoms: Vec<u32> = (0..index).map(|i| 50_002 + i).collect();
        let object = make_object_with_value(
            agent,
            root_shape,
            &extra_atoms,
            value_name,
            Value::from_smi(index.cast_signed()),
        );
        install_global_value(agent, &realm, source_name, Value::from_object_ref(object));
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap();
    }

    let ic_state = vm
        .property_ic_state(installed.code(), slot)
        .expect("D2: PropertyIcState should exist after two slow-path installs");
    assert_eq!(
        ic_state.cache_state,
        InlineCacheState::Polymorphic,
        "D2: cache_state must be Polymorphic after two distinct shapes"
    );
    assert_eq!(
        ic_state.entry_count, 2,
        "D2: entry_count must be 2 after installing two shapes"
    );
}

// D3: Polymorphic → Megamorphic. Drive 9 distinct shapes (POLY_LIMIT=2 inline +
// chain capacity 6 + one more) → cache_state must be Megamorphic and chain
// entry must be dropped.
#[test]
fn d3_property_ic_state_poly_to_megamorphic() {
    let unit = compile_test_unit(50_003, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    // 9 shapes: at shape 9 the IC must transition to Megamorphic.
    for index in 0..9u32 {
        let extra_atoms: Vec<u32> = (0..index).map(|i| 50_003 + i).collect();
        let object = make_object_with_value(
            agent,
            root_shape,
            &extra_atoms,
            value_name,
            Value::from_smi(index.cast_signed()),
        );
        install_global_value(agent, &realm, source_name, Value::from_object_ref(object));
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap();
    }

    let ic_state = vm
        .property_ic_state(installed.code(), slot)
        .expect("D3: PropertyIcState should exist after mega transition");
    assert_eq!(
        ic_state.cache_state,
        InlineCacheState::Megamorphic,
        "D3: cache_state must be Megamorphic after 9 distinct shapes"
    );
    assert_eq!(
        ic_state.entry_count, 0,
        "D3: entry_count must be 0 in Megamorphic state"
    );
    assert!(
        vm.polymorphic_chain(installed.code(), slot).is_none(),
        "D3: polymorphic chain must be dropped on Mega transition"
    );
}

// D4: Clear → Monomorphic. Install a PrototypeData IC entry to get Monomorphic,
// then mutate the prototype shape to fire the AdaptiveProtoLoad watchpoint,
// which clears the IC slot (via `clear_ic_slot_if_generation_matches`) and
// removes the `PropertyIcState` entry. After re-running, the slow path must
// reinstall and return to Monomorphic.
//
// This mirrors the B5 / A.3 AdaptiveProtoLoad tests but asserts on
// `vm.property_ic_state` instead of (or in addition to) the legacy snapshot.
#[test]
fn d4_property_ic_state_clear_and_reinstall() {
    // Source reads `source.value` where `value` is on a shared prototype.
    let unit = compile_test_unit(50_004, "source.value;");
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
    let value_name = unit_runtime_atom(agent, &unit, value_atom);
    let extra_name = agent.atoms_mut().intern_collectible("_extra_d4");

    // Build a prototype that carries `value`, and a receiver object that
    // delegates to it (PrototypeData IC path).
    let proto = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    assert!(ordinary_create_data_property(
        agent,
        proto,
        PropertyKey::from_atom(value_name),
        Value::from_smi(99),
        AllocationLifetime::Default,
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());
    let proto_shape = agent
        .objects()
        .object_header(agent.heap().view(), proto)
        .expect("proto must have header")
        .shape();

    // Receiver has `proto` as its prototype.
    let receiver = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(proto)),
            AllocationLifetime::Default,
        )
    });
    install_global_value(agent, &realm, source_name, Value::from_object_ref(receiver));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    // First two runs: warmup + IC install (PrototypeData path → Monomorphic).
    for _ in 0..2 {
        assert_eq!(
            vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
                .run()
                .unwrap(),
            Value::from_smi(99),
            "D4: initial runs must return the proto value"
        );
    }

    // Confirm Monomorphic state in PropertyIcState.
    {
        let ic_state = vm
            .property_ic_state(installed.code(), slot)
            .expect("D4: PropertyIcState should exist after first install");
        assert_eq!(
            ic_state.cache_state,
            InlineCacheState::Monomorphic,
            "D4: must be Monomorphic before watchpoint fire"
        );
    }

    // Mutate the prototype (add a new property). This fires the
    // AdaptiveProtoLoad watchpoint on `proto_shape`, which calls
    // `clear_ic_slot_if_generation_matches`. That clears the
    // `FeedbackSiteState` slot to `None` AND removes the `PropertyIcState`.
    assert!(ordinary_create_data_property(
        agent,
        proto,
        PropertyKey::from_atom(extra_name),
        Value::from_smi(0),
        AllocationLifetime::Default,
        &mut vm,
    )
    .unwrap());

    // Both feedback slot and PropertyIcState must be gone.
    assert!(
        !vm.named_property_slot_is_present(installed.code(), slot),
        "D4: watchpoint fire must clear the IC slot"
    );
    assert!(
        vm.property_ic_state(installed.code(), slot).is_none(),
        "D4: watchpoint fire must remove the PropertyIcState entry"
    );

    // After the proto mutation, `value` is still accessible (now on the
    // post-transition proto shape).
    // Re-run: slow path reinstalls against the new prototype shape.
    let new_proto_shape = agent
        .objects()
        .object_header(agent.heap().view(), proto)
        .expect("proto must still have header after mutation")
        .shape();
    assert_ne!(
        proto_shape, new_proto_shape,
        "D4: proto shape must have changed after property addition"
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(99),
        "D4: value must still be accessible after watchpoint fire + reinstall"
    );

    // PropertyIcState must be back to Monomorphic.
    let after = vm
        .property_ic_state(installed.code(), slot)
        .expect("D4: PropertyIcState should exist after re-install");
    assert_eq!(
        after.cache_state,
        InlineCacheState::Monomorphic,
        "D4: must return to Monomorphic after re-install"
    );
    assert_eq!(
        after.entry_count, 1,
        "D4: entry_count must be 1 after re-install"
    );
}

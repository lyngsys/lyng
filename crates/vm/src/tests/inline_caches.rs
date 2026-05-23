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
            .unwrap(),
        Value::from_smi(42)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
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
            .unwrap(),
        Value::from_smi(11)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(11)
    );

    // Swap the prototype to one with a different value at the same shape.
    // The receiver epoch bump (cause = PrototypeMutation) must invalidate
    // the proto shortcut so the next access observes the new value.
    agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects
            .set_prototype_of(&mut mutator, object, Some(prototype_b))
            .unwrap()
    });
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
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
            .unwrap(),
        Value::from_smi(99)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
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
            .unwrap(),
        Value::from_smi(77)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
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
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );
}

#[cfg(feature = "opcode-counters")]
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
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );

    vm.enable_opcode_dispatch_counts();
    vm.enable_slow_path_counts();
    vm.reset_opcode_dispatch_counts();
    vm.reset_slow_path_counts();

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(11)
    );

    let dispatch = vm
        .opcode_dispatch_counts()
        .expect("opcode counters should be enabled");
    let slow_path = vm
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
                .unwrap(),
            Value::from_smi(i32::try_from(index).expect("test source index should fit i32"))
        );
    }

    let snapshot = vm
        .feedback_vector_snapshot(installed.code())
        .expect("entry code should expose a feedback snapshot");
    let FeedbackSiteDetail::NamedProperty(named) = snapshot
        .sites()
        .iter()
        .find(|site| site.slot() == slot)
        .expect("named load site should be present")
        .detail()
    else {
        panic!("source.value should expose named-property feedback");
    };
    let actual_shapes = named
        .entries()
        .iter()
        .map(|entry| entry.receiver_shape().get())
        .collect::<Vec<_>>();
    let mut sorted_shapes = actual_shapes.clone();
    sorted_shapes.sort_unstable();

    assert_eq!(named.state(), FeedbackInlineCacheState::Polymorphic);
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
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );
}

#[cfg(feature = "opcode-counters")]
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
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_objects::NamedPropertyCachePath::OwnData)
        ))
    );

    vm.enable_opcode_dispatch_counts();
    vm.enable_slow_path_counts();
    vm.reset_opcode_dispatch_counts();
    vm.reset_slow_path_counts();

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(9)
    );

    let dispatch = vm
        .opcode_dispatch_counts()
        .expect("opcode counters should be enabled");
    let slow_path = vm
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
                .unwrap(),
            Value::undefined()
        );
    }

    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some(("Uninitialized", 0, None))
    );
    assert_eq!(vm.feedback_execution_count(installed.code(), slot), Some(2));
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
                .unwrap(),
            Value::from_smi(i32::try_from(index).expect("test source index should fit i32"))
        );
    }

    let snapshot = vm
        .feedback_vector_snapshot(installed.code())
        .expect("entry code should expose a feedback snapshot");
    let FeedbackSiteDetail::KeyedProperty(keyed) = snapshot
        .sites()
        .iter()
        .find(|site| site.slot() == slot)
        .expect("keyed access site should be present")
        .detail()
    else {
        panic!("source[\"value\"] should expose keyed-property feedback");
    };
    let actual_shapes = keyed
        .entries()
        .iter()
        .map(|entry| entry.entry().receiver_shape().get())
        .collect::<Vec<_>>();
    let mut sorted_shapes = actual_shapes.clone();
    sorted_shapes.sort_unstable();

    assert_eq!(keyed.state(), FeedbackInlineCacheState::Polymorphic);
    assert_eq!(keyed.family(), Some(FeedbackKeyedPropertyFamily::NamedAtom));
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
            .unwrap(),
        Value::from_smi(12)
    );
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
    )
    .unwrap());
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));
    install_global_value(agent, &realm, key_name, Value::from_smi(0));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    for _ in 0..3 {
        assert_eq!(
            vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
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
        )
        .unwrap());
    }
    assert!(ordinary_create_data_property(
        agent,
        object,
        PropertyKey::from_atom(value_atom),
        value,
        AllocationLifetime::Default,
    )
    .unwrap());
    object
}

#[test]
fn flat_named_property_header_tracks_monomorphic_inline_load() {
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
    let object = make_object_with_value(agent, root_shape, &[], value_name, Value::from_smi(33));
    install_global_value(agent, &realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
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

    let (mode, handler_bits, epoch) = vm
        .flat_named_property_header_snapshot(installed.code(), slot)
        .expect("flat header should exist for named-load slot");
    assert_eq!(
        mode,
        crate::dsl::feedback_flat::LLINT_IC_MODE_NAMED_OWN_INLINE_LOAD
    );
    assert_ne!(handler_bits, 0);
    assert_eq!(epoch, 0);
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
            .unwrap(),
        Value::from_smi(10)
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_b));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(20)
    );
    // Re-evaluate both shapes after the polymorphic transition: each
    // must come out of the inline sidecar with the right value, not the
    // other shape's value.
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_a));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(10)
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_b));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
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

#[cfg(feature = "opcode-counters")]
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

    vm.enable_opcode_dispatch_counts();
    vm.enable_slow_path_counts();
    vm.reset_opcode_dispatch_counts();
    vm.reset_slow_path_counts();

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(33)
    );

    let dispatch = vm
        .opcode_dispatch_counts()
        .expect("opcode counters should be enabled");
    let slow_path = vm
        .slow_path_counts()
        .expect("slow-path counters should be enabled");
    assert_eq!(dispatch.count(Opcode::GetNamedProperty), 1);
    assert_eq!(
        slow_path.semantic(Opcode::GetNamedProperty),
        0,
        "cached named-property IC hit should avoid the semantic slow bridge"
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
            .unwrap(),
        Value::from_smi(99)
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_b));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
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
            .unwrap(),
        Value::from_smi(99)
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_b));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
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
            .unwrap(),
        Value::from_smi(10)
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_b));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
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
            .unwrap(),
        Value::from_smi(10),
        "own-data slot still resolves correctly through slow chain after epoch bump"
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_b));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
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
            .unwrap(),
        Value::from_smi(70)
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_b));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(80)
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_a));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(70)
    );
    install_global_value(agent, &realm, source_name, Value::from_object_ref(shape_b));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(80)
    );

    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Polymorphic", Some("NamedAtom"), 2))
    );
}

use super::support::*;

#[test]
fn feedback_vectors_allocate_lazily_without_changing_entry_script_result() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        SourceId::new(21),
        r"
            (function add(left, right) {
                return left + right;
            })(1, 2);
        ",
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit.function(unit.entry()).unwrap();
    let call_slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::Call)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain one call site");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    let first = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    assert_eq!(first, Value::from_smi(3));
    assert_eq!(vm.feedback_warmup_counter(installed.code()), Some(1));
    assert!(!vm.has_feedback_vector(installed.code()));

    let second = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    assert_eq!(second, Value::from_smi(3));
    assert_eq!(vm.feedback_warmup_counter(installed.code()), Some(2));
    assert!(vm.has_feedback_vector(installed.code()));
    assert_eq!(
        vm.feedback_execution_count(installed.code(), call_slot),
        Some(1)
    );
}

fn feedback_site(
    snapshot: &FeedbackVectorSnapshot,
    slot: FeedbackSlotId,
) -> &crate::FeedbackSiteSnapshot {
    snapshot
        .sites()
        .iter()
        .find(|site| site.slot() == slot)
        .expect("feedback snapshot should include the requested slot")
}

#[test]
fn feedback_vector_snapshot_reports_scalar_sites_for_tier_decisions() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        SourceId::new(39),
        r"
            function C(value) { this.value = value; }
            function add(left, right) { return left + right; }
            add(1, 2) < 5;
            new C(9);
        ",
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let add_name = unit_atom(&unit, "add");
    let add_function = unit
        .functions()
        .iter()
        .find(|function| function.name() == Some(add_name))
        .expect("add function should be lowered");
    let entry = unit.function(unit.entry()).unwrap();
    let call_slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::Call)
        .map(|descriptor| descriptor.slot())
        .expect("entry should contain a call site");
    let construct_slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::Construct)
        .map(|descriptor| descriptor.slot())
        .expect("entry should contain a construct site");
    let comparison_slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::Comparison)
        .map(|descriptor| descriptor.slot())
        .expect("entry should contain a comparison site");
    let arithmetic_slot = add_function
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::Arithmetic)
        .map(|descriptor| descriptor.slot())
        .expect("add should contain an arithmetic site");
    let add_child_index = entry
        .child_functions()
        .iter()
        .position(|child| *child == add_function.id())
        .and_then(|index| u32::try_from(index).ok())
        .expect("entry should install add as a direct child");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let add_code = vm
        .installed_child_code(installed.code(), add_child_index)
        .expect("add should have installed code");

    for _ in 0..2 {
        assert!(vm
            .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap()
            .is_object());
    }

    let entry_snapshot = vm
        .feedback_vector_snapshot(installed.code())
        .expect("entry code should expose a feedback snapshot");
    assert!(entry_snapshot.allocated());
    assert!(feedback_site(&entry_snapshot, comparison_slot).execution_count() >= 1);
    assert_eq!(
        feedback_site(&entry_snapshot, call_slot).detail(),
        FeedbackSiteDetail::Call {
            expected_arity: Some(2)
        }
    );
    assert_eq!(
        feedback_site(&entry_snapshot, construct_slot).detail(),
        FeedbackSiteDetail::Construct {
            expected_arity: Some(1)
        }
    );

    let add_snapshot = vm
        .feedback_vector_snapshot(add_code)
        .expect("add code should expose a feedback snapshot");
    assert_eq!(
        feedback_site(&add_snapshot, arithmetic_slot).execution_count(),
        1
    );
    assert!(matches!(
        feedback_site(&add_snapshot, arithmetic_slot).detail(),
        FeedbackSiteDetail::Arithmetic
    ));
}

#[test]
fn feedback_vector_snapshot_reports_property_cache_state_without_mutable_entries() {
    let unit = compile_test_unit(40, "source.value;");
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
                PropertyKey::from_atom(AtomId::from_raw(21_000 + extra)),
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

        let snapshot = vm
            .feedback_vector_snapshot(installed.code())
            .expect("entry code should expose a feedback snapshot");
        let FeedbackSiteDetail::NamedProperty(named) = feedback_site(&snapshot, slot).detail()
        else {
            panic!("source.value should expose named-property feedback");
        };
        match index {
            0 => {
                assert_eq!(named.state(), FeedbackInlineCacheState::Monomorphic);
                assert_eq!(named.entries().len(), 1);
                assert_eq!(named.entries()[0].path(), NamedPropertyCachePath::OwnData);
            }
            1 => {
                assert_eq!(named.state(), FeedbackInlineCacheState::Polymorphic);
                assert_eq!(named.entries().len(), 2);
            }
            5 => {
                assert_eq!(named.state(), FeedbackInlineCacheState::Megamorphic);
                assert!(named.entries().is_empty());
            }
            _ => {}
        }
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "semantic regression scenario stays contiguous within its domain-focused test module"
)]
fn feedback_vector_snapshot_reports_keyed_property_classifiers() {
    let named_unit = compile_test_unit(41, "source[\"value\"];");
    let named_entry = named_unit.function(named_unit.entry()).unwrap();
    let named_slot = named_entry
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
    let source_name = unit_runtime_atom(agent, &named_unit, unit_atom(&named_unit, "source"));
    let value_name = unit_runtime_atom(agent, &named_unit, unit_atom(&named_unit, "value"));
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
    let named_installed = vm.install_script(agent, realm.id(), &named_unit).unwrap();
    for _ in 0..2 {
        assert_eq!(
            vm.evaluate_installed(
                agent,
                named_installed,
                realm.global_env(),
                realm.global_env()
            )
            .unwrap(),
            Value::from_smi(4)
        );
    }
    let named_snapshot = vm
        .feedback_vector_snapshot(named_installed.code())
        .expect("named keyed access should expose feedback");
    let FeedbackSiteDetail::KeyedProperty(named_keyed) =
        feedback_site(&named_snapshot, named_slot).detail()
    else {
        panic!("source[\"value\"] should expose keyed-property feedback");
    };
    assert_eq!(named_keyed.state(), FeedbackInlineCacheState::Monomorphic);
    assert_eq!(
        named_keyed.family(),
        Some(FeedbackKeyedPropertyFamily::NamedAtom)
    );
    assert_eq!(named_keyed.entries().len(), 1);

    let dense_unit = compile_test_unit(42, "let index = 0; source[index];");
    let dense_entry = dense_unit.function(dense_unit.entry()).unwrap();
    let dense_slot = dense_entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::KeyedPropertyAccess)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a dense keyed-access site");
    let dense_source_name = unit_runtime_atom(agent, &dense_unit, unit_atom(&dense_unit, "source"));
    let dense_object = agent.with_heap_and_objects(|heap, objects| {
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
    install_global_value(
        agent,
        &realm,
        dense_source_name,
        Value::from_object_ref(dense_object),
    );
    let dense_installed = vm.install_script(agent, realm.id(), &dense_unit).unwrap();
    for _ in 0..2 {
        assert_eq!(
            vm.evaluate_installed(
                agent,
                dense_installed,
                realm.global_env(),
                realm.global_env()
            )
            .unwrap(),
            Value::from_smi(12)
        );
    }
    let dense_snapshot = vm
        .feedback_vector_snapshot(dense_installed.code())
        .expect("dense keyed access should expose feedback");
    let FeedbackSiteDetail::KeyedProperty(dense_keyed) =
        feedback_site(&dense_snapshot, dense_slot).detail()
    else {
        panic!("source[index] should expose keyed-property feedback");
    };
    assert_eq!(dense_keyed.state(), FeedbackInlineCacheState::Megamorphic);
    assert_eq!(
        dense_keyed.family(),
        Some(FeedbackKeyedPropertyFamily::DenseIndex)
    );
    assert!(dense_keyed.entries().is_empty());
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "semantic regression scenario stays contiguous within its domain-focused test module"
)]
fn prototype_cache_snapshots_replan_after_object_owned_invalidation() {
    let unit = compile_test_unit(43, "source.value;");
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
    let (receiver, replacement) = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let prototype = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        );
        let receiver = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(prototype)),
            AllocationLifetime::Default,
        );
        let replacement = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        );
        (receiver, replacement)
    });
    assert!(ordinary_create_data_property(
        agent,
        replacement,
        PropertyKey::from_atom(value_name),
        Value::from_smi(13),
        AllocationLifetime::Default,
    )
    .unwrap());
    install_global_value(agent, &realm, source_name, Value::from_object_ref(receiver));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let original_prototype = agent
        .with_heap_and_objects(|heap, _| heap.view().object(receiver).unwrap().prototype())
        .expect("receiver should keep its original prototype");
    assert!(ordinary_create_data_property(
        agent,
        original_prototype,
        PropertyKey::from_atom(value_name),
        Value::from_smi(7),
        AllocationLifetime::Default,
    )
    .unwrap());

    for _ in 0..2 {
        assert_eq!(
            vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
                .unwrap(),
            Value::from_smi(7)
        );
    }
    let before = vm
        .feedback_vector_snapshot(installed.code())
        .expect("prototype load should expose feedback");
    let FeedbackSiteDetail::NamedProperty(before_named) = feedback_site(&before, slot).detail()
    else {
        panic!("source.value should expose named-property feedback");
    };
    assert_eq!(
        before_named.entries()[0].path(),
        NamedPropertyCachePath::PrototypeData
    );
    assert_eq!(before_named.entries()[0].dependencies().len(), 2);
    let old_holder = before_named.entries()[0].holder();

    assert!(agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects
            .set_prototype_of(&mut mutator, receiver, Some(replacement))
            .unwrap()
    }));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(13)
    );
    let after = vm
        .feedback_vector_snapshot(installed.code())
        .expect("prototype load should expose feedback after invalidation");
    let FeedbackSiteDetail::NamedProperty(after_named) = feedback_site(&after, slot).detail()
    else {
        panic!("source.value should expose named-property feedback");
    };
    assert_eq!(after_named.state(), FeedbackInlineCacheState::Monomorphic);
    assert_eq!(
        after_named.entries()[0].path(),
        NamedPropertyCachePath::PrototypeData
    );
    assert_ne!(after_named.entries()[0].holder(), old_holder);
}

#[test]
fn tiering_hotness_is_opt_in_and_independent_of_lazy_feedback_allocation() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        SourceId::new(25),
        r"
            (function add(left, right) {
                return left + right;
            })(1, 2);
        ",
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit.function(unit.entry()).unwrap();
    let call_slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::Call)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain one call site");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    let initial = vm
        .tiering_snapshot(installed.code())
        .expect("installed code should expose tiering state");
    assert!(!initial.is_eligible());
    assert_eq!(initial.status(), TierStatus::InterpreterOnly);
    assert_eq!(initial.hotness(), 0);

    let first = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    assert_eq!(first, Value::from_smi(3));
    assert_eq!(vm.feedback_warmup_counter(installed.code()), Some(1));
    assert!(!vm.has_feedback_vector(installed.code()));
    assert_eq!(
        vm.tiering_snapshot(installed.code())
            .expect("installed code should expose tiering state")
            .hotness(),
        0
    );

    assert!(vm.set_tier_eligible(installed.code(), true));
    let eligible = vm
        .tiering_snapshot(installed.code())
        .expect("installed code should expose tiering state");
    assert!(eligible.is_eligible());
    assert_eq!(eligible.status(), TierStatus::Collecting);

    let second = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    assert_eq!(second, Value::from_smi(3));
    assert_eq!(vm.feedback_warmup_counter(installed.code()), Some(2));
    assert!(vm.has_feedback_vector(installed.code()));
    assert_eq!(
        vm.feedback_execution_count(installed.code(), call_slot),
        Some(1)
    );
    let warmed = vm
        .tiering_snapshot(installed.code())
        .expect("installed code should expose tiering state");
    assert_eq!(warmed.status(), TierStatus::Collecting);
    assert_eq!(warmed.hotness(), 1);
    assert_eq!(warmed.feedback_events(), 1);
    assert_eq!(warmed.backedge_events(), 0);
}

#[test]
fn closures_sharing_one_code_ref_share_feedback_warmup_and_vector_state() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        SourceId::new(22),
        r"
            function makeAdder(base) {
                return function(delta) {
                    return base + delta;
                };
            }
            let first = makeAdder(1);
            let second = makeAdder(2);
            first(3);
            second(4);
        ",
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let outer = unit
        .functions()
        .iter()
        .find(|function| function.name().is_some())
        .expect("named outer function should be lowered");
    let inner = unit
        .functions()
        .iter()
        .find(|function| function.name().is_none() && !function.captures().is_empty())
        .expect("capturing inner closure should be lowered");
    let arithmetic_slot = inner
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::Arithmetic)
        .map(|descriptor| descriptor.slot())
        .expect("inner closure should contain one arithmetic site");
    let outer_child_index = unit
        .function(unit.entry())
        .expect("entry function should exist")
        .child_functions()
        .iter()
        .position(|child| *child == outer.id())
        .and_then(|index| u32::try_from(index).ok())
        .expect("script should install the outer function as a direct child");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let outer_code = vm
        .installed_child_code(installed.code(), outer_child_index)
        .expect("outer function should have one installed code record");
    let inner_code = vm
        .installed_child_code(outer_code, 0)
        .expect("inner closure template should install under the outer function");

    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(6));
    assert_eq!(vm.feedback_warmup_counter(inner_code), Some(2));
    assert!(vm.has_feedback_vector(inner_code));
    assert_eq!(
        vm.feedback_execution_count(inner_code, arithmetic_slot),
        Some(1)
    );
}

#[test]
fn closures_sharing_one_code_ref_share_tiering_hotness() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        SourceId::new(26),
        r"
            function makeAdder(base) {
                return function(delta) {
                    return base + delta;
                };
            }
            let first = makeAdder(1);
            let second = makeAdder(2);
            first(3);
            second(4);
        ",
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let outer = unit
        .functions()
        .iter()
        .find(|function| function.name().is_some())
        .expect("named outer function should be lowered");
    let outer_child_index = unit
        .function(unit.entry())
        .expect("entry function should exist")
        .child_functions()
        .iter()
        .position(|child| *child == outer.id())
        .and_then(|index| u32::try_from(index).ok())
        .expect("script should install the outer function as a direct child");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let outer_code = vm
        .installed_child_code(installed.code(), outer_child_index)
        .expect("outer function should have one installed code record");
    let inner_code = vm
        .installed_child_code(outer_code, 0)
        .expect("inner closure template should install under the outer function");

    assert!(vm.set_tier_eligible(inner_code, true));
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(6));
    assert!(vm.has_feedback_vector(inner_code));
    let snapshot = vm
        .tiering_snapshot(inner_code)
        .expect("inner code should expose tiering state");
    assert_eq!(snapshot.status(), TierStatus::Collecting);
    assert_eq!(snapshot.hotness(), 2);
    assert_eq!(snapshot.feedback_events(), 2);
}

#[test]
fn loop_backedges_make_eligible_code_ready_and_invalidation_resets_hotness() {
    let unit = compile_test_unit(
        27,
        r"
            let total = 0;
            for (let i = 0; i < 16; i = i + 1) {
                total = total + i;
            }
            total;
        ",
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    assert!(vm.set_tier_eligible(installed.code(), true));
    let first = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    assert_eq!(first, Value::from_smi(120));

    let ready = vm
        .tiering_snapshot(installed.code())
        .expect("installed code should expose tiering state");
    assert_eq!(ready.status(), TierStatus::ReadyForNative);
    assert!(ready.hotness() >= 8);
    assert!(ready.backedge_events() > 0);
    assert_eq!(ready.invalidation_epoch(), 0);
    assert_eq!(ready.native_generation(), None);

    assert!(vm.invalidate_tier_state(installed.code()));
    let invalidated = vm
        .tiering_snapshot(installed.code())
        .expect("installed code should expose tiering state");
    assert_eq!(invalidated.status(), TierStatus::Invalidated);
    assert_eq!(invalidated.hotness(), 0);
    assert_eq!(invalidated.invalidation_epoch(), 1);
    assert_eq!(invalidated.native_generation(), None);

    let second = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    assert_eq!(second, Value::from_smi(120));
    let rewarmed = vm
        .tiering_snapshot(installed.code())
        .expect("installed code should expose tiering state");
    assert_eq!(rewarmed.status(), TierStatus::ReadyForNative);
    assert_eq!(rewarmed.invalidation_epoch(), 1);
}

#[test]
fn internal_bytecode_callbacks_share_feedback_state_with_the_parent_vm() {
    let unit = compile_test_unit(
        23,
        r#"
            function callback() {
                return 1 + 2;
            }
            "ab".replace("b", callback);
            0;
        "#,
    );
    let callback_name = unit_atom(&unit, "callback");
    let callback = unit
        .functions()
        .iter()
        .find(|function| function.name() == Some(callback_name))
        .expect("callback function should be lowered");
    let arithmetic_slot = callback
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::Arithmetic)
        .map(|descriptor| descriptor.slot())
        .expect("callback should contain one arithmetic site");
    let callback_child_index = unit
        .function(unit.entry())
        .expect("entry function should exist")
        .child_functions()
        .iter()
        .position(|child| *child == callback.id())
        .and_then(|index| u32::try_from(index).ok())
        .expect("script should install the callback as a direct child");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let callback_code = vm
        .installed_child_code(installed.code(), callback_child_index)
        .expect("callback function should have installed code");

    let first = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    assert_eq!(first, Value::from_smi(0));
    assert_eq!(vm.feedback_warmup_counter(callback_code), Some(1));
    assert!(!vm.has_feedback_vector(callback_code));

    let second = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    assert_eq!(second, Value::from_smi(0));
    assert_eq!(vm.feedback_warmup_counter(callback_code), Some(2));
    assert!(vm.has_feedback_vector(callback_code));
    assert_eq!(
        vm.feedback_execution_count(callback_code, arithmetic_slot),
        Some(1)
    );
}

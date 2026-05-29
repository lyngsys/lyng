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
        .run()
        .unwrap();
    assert_eq!(first, Value::from_smi(3));
    assert_eq!(vm.feedback_warmup_counter(installed.code()), Some(1));
    assert!(!vm.has_feedback_vector(installed.code()));

    let second = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();
    assert_eq!(second, Value::from_smi(3));
    assert_eq!(vm.feedback_warmup_counter(installed.code()), Some(2));
    assert!(vm.has_feedback_vector(installed.code()));
    assert_eq!(
        vm.feedback_execution_count(installed.code(), call_slot),
        Some(1)
    );
}

fn first_call_slot(unit: &CompiledScriptUnit) -> FeedbackSlotId {
    unit.function(unit.entry())
        .expect("test unit should have an entry function")
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::Call)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a call site")
}

fn first_construct_slot(unit: &CompiledScriptUnit) -> FeedbackSlotId {
    unit.function(unit.entry())
        .expect("test unit should have an entry function")
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::Construct)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a construct site")
}

fn evaluated_call_status(source_id: u32, source: &str, expected: Value) -> crate::CallStatus {
    let unit = compile_test_unit(source_id, source);
    let call_slot = first_call_slot(&unit);
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();
    assert_eq!(result, expected);

    vm.call_status(installed.code(), call_slot)
        .expect("entry code should expose a call status")
}

fn evaluated_construct_status(
    source_id: u32,
    source: &str,
    expected: Value,
) -> crate::ConstructStatus {
    let unit = compile_test_unit(source_id, source);
    let construct_slot = first_construct_slot(&unit);
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();
    assert_eq!(result, expected);

    vm.construct_status(installed.code(), construct_slot)
        .expect("entry code should expose a construct status")
}

#[test]
fn call_status_records_monomorphic_target_identity() {
    let status = evaluated_call_status(
        601,
        r"
            function target(value) { return value + 1; }
            var total = 0;
            for (var i = 0; i < 4; i = i + 1) {
                total = total + target(i);
            }
            total;
        ",
        Value::from_smi(10),
    );

    assert_eq!(status.expected_arity(), Some(1));
    assert_eq!(status.state(), FeedbackInlineCacheState::Monomorphic);
    assert_eq!(status.entries.len(), 1);
    assert!(status.entries[0].realm.is_some());
}

#[test]
fn call_status_records_builtin_target_identity() {
    let status = evaluated_call_status(
        607,
        r#"
            var text = "abc";
            var total = 0;
            for (var i = 0; i < 4; i = i + 1) {
                total = total + text.charCodeAt(1);
            }
            total;
        "#,
        Value::from_smi(392),
    );

    assert_eq!(status.expected_arity(), Some(1));
    assert_eq!(status.state(), FeedbackInlineCacheState::Monomorphic);
    assert_eq!(status.entries.len(), 1);
    assert_eq!(
        status.entries[0].builtin,
        Some(lyng_types::string_char_code_at_builtin())
    );
}

#[test]
fn call_status_records_polymorphic_target_identities() {
    let status = evaluated_call_status(
        602,
        r"
            function a(value) { return value + 1; }
            function b(value) { return value + 2; }
            var total = 0;
            for (var i = 0; i < 6; i = i + 1) {
                var fn = a;
                if (i >= 3) {
                    fn = b;
                }
                total = total + fn(i);
            }
            total;
        ",
        Value::from_smi(24),
    );

    assert_eq!(status.expected_arity(), Some(1));
    assert_eq!(status.state(), FeedbackInlineCacheState::Polymorphic);
    assert_eq!(status.entries.len(), 2);
    assert_ne!(status.entries[0].function, status.entries[1].function);
    assert!(status.entries.iter().all(|entry| entry.realm.is_some()));
}

#[test]
fn call_status_promotes_to_megamorphic_after_cache_limit() {
    let status = evaluated_call_status(
        603,
        r"
            function f0(value) { return value; }
            function f1(value) { return value; }
            function f2(value) { return value; }
            function f3(value) { return value; }
            function f4(value) { return value; }
            function f5(value) { return value; }
            function f6(value) { return value; }
            function f7(value) { return value; }
            function f8(value) { return value; }
            function f9(value) { return value; }
            var total = 0;
            for (var i = 0; i < 10; i = i + 1) {
                var fn = f0;
                if (i === 1) { fn = f1; }
                if (i === 2) { fn = f2; }
                if (i === 3) { fn = f3; }
                if (i === 4) { fn = f4; }
                if (i === 5) { fn = f5; }
                if (i === 6) { fn = f6; }
                if (i === 7) { fn = f7; }
                if (i === 8) { fn = f8; }
                if (i === 9) { fn = f9; }
                total = total + fn(i);
            }
            total;
        ",
        Value::from_smi(45),
    );

    assert_eq!(status.expected_arity(), Some(1));
    assert_eq!(status.state(), FeedbackInlineCacheState::Megamorphic);
    assert!(status.entries.is_empty());
}

#[test]
fn construct_status_records_monomorphic_target_and_created_shape() {
    let status = evaluated_construct_status(
        604,
        r"
            function Target(value) { this.value = value; }
            var total = 0;
            for (var i = 0; i < 4; i = i + 1) {
                total = total + new Target(i).value;
            }
            total;
        ",
        Value::from_smi(6),
    );

    assert_eq!(status.expected_arity(), Some(1));
    assert_eq!(status.state(), FeedbackInlineCacheState::Monomorphic);
    assert_eq!(status.entries.len(), 1);
    assert!(status.entries[0].realm.is_some());
    assert!(status.entries[0].created_shape.is_some());
}

#[test]
fn construct_status_records_polymorphic_targets() {
    let status = evaluated_construct_status(
        605,
        r"
            function A(value) { this.value = value + 1; }
            function B(value) { this.value = value + 2; }
            var total = 0;
            for (var i = 0; i < 6; i = i + 1) {
                var Ctor = A;
                if (i >= 3) {
                    Ctor = B;
                }
                total = total + new Ctor(i).value;
            }
            total;
        ",
        Value::from_smi(24),
    );

    assert_eq!(status.expected_arity(), Some(1));
    assert_eq!(status.state(), FeedbackInlineCacheState::Polymorphic);
    assert_eq!(status.entries.len(), 2);
    assert_ne!(status.entries[0].function, status.entries[1].function);
    assert!(status
        .entries
        .iter()
        .all(|entry| entry.created_shape.is_some()));
}

#[test]
fn construct_status_promotes_to_megamorphic_after_cache_limit() {
    let status = evaluated_construct_status(
        606,
        r"
            function C0(value) { this.value = value; }
            function C1(value) { this.value = value; }
            function C2(value) { this.value = value; }
            function C3(value) { this.value = value; }
            function C4(value) { this.value = value; }
            function C5(value) { this.value = value; }
            function C6(value) { this.value = value; }
            function C7(value) { this.value = value; }
            function C8(value) { this.value = value; }
            function C9(value) { this.value = value; }
            var total = 0;
            for (var i = 0; i < 10; i = i + 1) {
                var Ctor = C0;
                if (i === 1) { Ctor = C1; }
                if (i === 2) { Ctor = C2; }
                if (i === 3) { Ctor = C3; }
                if (i === 4) { Ctor = C4; }
                if (i === 5) { Ctor = C5; }
                if (i === 6) { Ctor = C6; }
                if (i === 7) { Ctor = C7; }
                if (i === 8) { Ctor = C8; }
                if (i === 9) { Ctor = C9; }
                total = total + new Ctor(i).value;
            }
            total;
        ",
        Value::from_smi(45),
    );

    assert_eq!(status.expected_arity(), Some(1));
    assert_eq!(status.state(), FeedbackInlineCacheState::Megamorphic);
    assert!(status.entries.is_empty());
}

#[test]
fn metadata_table_footprint_reports_scalar_sites_for_tier_decisions() {
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
            .run()
            .unwrap()
            .is_object());
    }

    let footprint = vm
        .metadata_table_footprint(installed.code())
        .expect("entry code should expose a metadata footprint");
    assert!(footprint.allocated());
    // Phase D.1.0: Comparison feedback is MetadataTable-owned (no asm callers
    // yet); execution_count is always 0 for Comparison slots.
    let comparison_status = vm
        .comparison_status(installed.code(), comparison_slot)
        .expect("comparison site should expose comparison status");
    assert_eq!(
        comparison_status.execution_count, 0,
        "Comparison execution_count comes from MetadataTable (no asm callers); must be 0"
    );
    let call_status = vm
        .call_status(installed.code(), call_slot)
        .expect("entry call site should expose call status");
    assert_eq!(call_status.expected_arity(), Some(2));
    let construct_status = vm
        .construct_status(installed.code(), construct_slot)
        .expect("entry construct site should expose construct status");
    assert_eq!(construct_status.expected_arity(), Some(1));

    // Phase D.1.0: Arithmetic execution_count is MetadataTable-owned (asm-written).
    // drain_llint_scalar_feedback zeroes MetadataTable after each run(); the
    // status reports execution_count 0.
    let arith_status = vm
        .arith_status(add_code, arithmetic_slot)
        .expect("add code should expose an arith status");
    assert_eq!(
        arith_status.execution_count, 0,
        "Arithmetic execution_count is MetadataTable-owned after Phase D.1.0; drained on each run"
    );
}

#[test]
fn llint_scalar_feedback_batch_drain_preserves_warmup_execution_counts() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        SourceId::new(57),
        r"
            function four(x) {
                x = x + 1;
                x = x + 1;
                x = x + 1;
                x = x + 1;
                return x;
            }
            four(0);
        ",
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let four_name = unit_atom(&unit, "four");
    let four_function = unit
        .functions()
        .iter()
        .find(|function| function.name() == Some(four_name))
        .expect("four function should be lowered");
    let arithmetic_slots: Vec<_> = four_function
        .feedback_sites()
        .iter()
        .filter(|descriptor| descriptor.kind() == FeedbackSiteKind::Arithmetic)
        .map(|descriptor| descriptor.slot())
        .collect();
    assert_eq!(
        arithmetic_slots.len(),
        4,
        "straight-line test shape should produce four arithmetic feedback sites"
    );
    let four_child_index = unit
        .function(unit.entry())
        .expect("entry function should exist")
        .child_functions()
        .iter()
        .position(|child| *child == four_function.id())
        .and_then(|index| u32::try_from(index).ok())
        .expect("script should install four as a direct child");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let four_code = vm
        .installed_child_code(installed.code(), four_child_index)
        .expect("four function should have installed code");

    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(4));
    assert_eq!(vm.feedback_warmup_counter(four_code), Some(2));
    assert!(vm.has_feedback_vector(four_code));
    // Phase D.1.0: Arithmetic slots have no Rust-side FeedbackSiteState variant.
    // feedback_execution_count returns None for Arithmetic slots.
    // Per-slot execution counts live in MetadataTable (zeroed by drain_llint_scalar_feedback
    // after each run). The warmup counter (above) is the durable record of drain activity.
    for slot in &arithmetic_slots {
        assert_eq!(
            vm.feedback_execution_count(four_code, *slot),
            None,
            "Arithmetic slots have no Rust-side state after Phase D.1.0"
        );
    }
}

#[test]
fn drain_only_scans_executed_code() {
    // A script with two functions: `runs` (called, does arithmetic) and
    // `never` (installed, has an arithmetic site, but never invoked). The
    // drain after each `run` must only scan executed code: `never`'s feedback
    // is left untouched, and `executed_codes` is cleared post-drain.
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        SourceId::new(58),
        r"
            function runs(x) {
                x = x + 1;
                return x;
            }
            function never(y) {
                y = y + 1;
                return y;
            }
            runs(0);
        ",
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();

    let runs_name = unit_atom(&unit, "runs");
    let never_name = unit_atom(&unit, "never");
    let entry = unit.function(unit.entry()).expect("entry function exists");
    let child_index = |target: &BytecodeFunction| {
        entry
            .child_functions()
            .iter()
            .position(|child| *child == target.id())
            .and_then(|index| u32::try_from(index).ok())
            .expect("function should be a direct child of entry")
    };
    let runs_function = unit
        .functions()
        .iter()
        .find(|function| function.name() == Some(runs_name))
        .expect("runs function should be lowered");
    let never_function = unit
        .functions()
        .iter()
        .find(|function| function.name() == Some(never_name))
        .expect("never function should be lowered");
    let runs_child = child_index(runs_function);
    let never_child = child_index(never_function);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let runs_code = vm
        .installed_child_code(installed.code(), runs_child)
        .expect("runs function should have installed code");
    let never_code = vm
        .installed_child_code(installed.code(), never_child)
        .expect("never function should have installed code");

    // Before any run, nothing has executed.
    assert!(vm.executed_codes_for_test().is_empty());

    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();
    assert_eq!(result, Value::from_smi(1));

    // After the run + drain, `executed_codes` is cleared for the next cycle.
    assert!(
        vm.executed_codes_for_test().is_empty(),
        "drain must clear executed_codes"
    );

    // The executed function tiered up (its arith feedback was drained).
    assert!(vm.feedback_warmup_counter(runs_code).is_some());

    // The never-executed function was never queued for draining: no feedback
    // vector was allocated and its warmup counter never advanced.
    assert!(
        !vm.has_feedback_vector(never_code),
        "never-executed code must not be drained"
    );
    assert_eq!(vm.feedback_warmup_counter(never_code), Some(0));
}

#[test]
fn named_property_status_reports_cache_state_without_mutable_entries() {
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

        let status = vm
            .named_property_status(installed.code(), slot)
            .expect("source.value should expose named-property status");
        match index {
            0 => {
                assert_eq!(status.state(), FeedbackInlineCacheState::Monomorphic);
                assert_eq!(status.entries.len(), 1);
                assert_eq!(status.entries[0].path(), NamedPropertyCachePath::OwnData);
            }
            1 => {
                assert_eq!(status.state(), FeedbackInlineCacheState::Polymorphic);
                assert_eq!(status.entries.len(), 2);
            }
            5 => {
                assert_eq!(status.state(), FeedbackInlineCacheState::Polymorphic);
                assert_eq!(status.entries.len(), 6);
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
fn keyed_property_status_reports_classifiers() {
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
        &mut NoopAdaptiveProtoLoadDispatch,
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
            .run()
            .unwrap(),
            Value::from_smi(4)
        );
    }
    let named_status = vm
        .keyed_property_status(named_installed.code(), named_slot)
        .expect("source[\"value\"] should expose keyed-property status");
    assert_eq!(named_status.state(), FeedbackInlineCacheState::Monomorphic);
    assert_eq!(
        named_status.family(),
        Some(FeedbackKeyedPropertyFamily::NamedAtom)
    );
    assert_eq!(named_status.named_entries.len(), 1);

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
            .run()
            .unwrap(),
            Value::from_smi(12)
        );
    }
    let dense_status = vm
        .keyed_property_status(dense_installed.code(), dense_slot)
        .expect("source[index] should expose keyed-property status");
    assert_eq!(dense_status.state(), FeedbackInlineCacheState::Monomorphic);
    assert_eq!(
        dense_status.family(),
        Some(FeedbackKeyedPropertyFamily::DenseIndex)
    );
    assert_eq!(dense_status.dense_entries.len(), 1);
    assert!(dense_status.named_entries.is_empty());
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "semantic regression scenario stays contiguous within its domain-focused test module"
)]
fn prototype_cache_status_replan_after_object_owned_invalidation() {
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
        &mut NoopAdaptiveProtoLoadDispatch,
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
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());

    for _ in 0..2 {
        assert_eq!(
            vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
                .run()
                .unwrap(),
            Value::from_smi(7)
        );
    }
    let before_status = vm
        .named_property_status(installed.code(), slot)
        .expect("source.value should expose named-property status");
    assert_eq!(
        before_status.entries[0].path(),
        NamedPropertyCachePath::PrototypeData
    );
    assert_eq!(before_status.entries[0].dependencies().len(), 2);
    let old_holder = before_status.entries[0].holder();

    // Swap receiver's prototype. `objects.set_prototype_of` does NOT perform
    // a shape transition on `receiver` (it only bumps the invalidation epoch);
    // nor does it fire AdaptiveProtoLoad watchpoints (no VM dispatch path).
    // Both `original_prototype` and `replacement` share the same post-property-
    // addition shape (shape transitions are shared). After Phase A.2's epoch-
    // check removal: the IC fast path sees matching receiver and prototype
    // shapes, reads the value from the CURRENT prototype object (`replacement`,
    // value=13) and returns the correct value without replanning.
    assert!(agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects
            .set_prototype_of(&mut mutator, receiver, Some(replacement))
            .unwrap()
    }));
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_smi(13),
        "value must be read from the new prototype even without IC replan"
    );
    let after_status = vm
        .named_property_status(installed.code(), slot)
        .expect("source.value should expose named-property status after prototype swap");
    // The IC stays Monomorphic PrototypeData. Phase A.2 removed the epoch
    // check; a shape-compare match against the shared prototype shape means
    // the fast path hit succeeds and no replan is needed. The `old_holder`
    // variable (used for the pre-Phase-A.2 `assert_ne` check) is kept to
    // document the before/after contrast.
    let _ = old_holder;
    assert_eq!(after_status.state(), FeedbackInlineCacheState::Monomorphic);
    assert_eq!(
        after_status.entries[0].path(),
        NamedPropertyCachePath::PrototypeData
    );
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
    let mut tiering = Tiering::new();
    tiering.ensure_slot(installed.code());

    let initial = tiering
        .snapshot(installed.code())
        .expect("installed code should expose tiering state");
    assert!(!initial.is_eligible());
    assert_eq!(initial.status(), TierStatus::InterpreterOnly);
    assert_eq!(initial.hotness(), 0);

    let first = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .with_tiering(&mut tiering)
        .run()
        .unwrap();
    assert_eq!(first, Value::from_smi(3));
    assert_eq!(
        tiering
            .snapshot(installed.code())
            .expect("installed code should expose tiering state after first run")
            .warmup_counter(),
        1,
        "warmup_counter now lives on TieringState; first execution should bump it to 1"
    );
    assert_eq!(
        tiering
            .snapshot(installed.code())
            .expect("installed code should expose tiering state")
            .hotness(),
        0
    );

    assert!(tiering.set_eligible(installed.code(), true));
    let eligible = tiering
        .snapshot(installed.code())
        .expect("installed code should expose tiering state");
    assert!(eligible.is_eligible());
    assert_eq!(eligible.status(), TierStatus::Collecting);

    let second = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .with_tiering(&mut tiering)
        .run()
        .unwrap();
    assert_eq!(second, Value::from_smi(3));
    assert_eq!(
        tiering
            .snapshot(installed.code())
            .expect("installed code should expose tiering state after second run")
            .warmup_counter(),
        2,
        "warmup_counter now lives on TieringState; second execution should bump it to 2 (= allocation threshold)"
    );
    // The MetadataTable footprint reports the per-kind site counts even
    // before allocation; the per-slot status APIs are the source of truth
    // for IC state after Phase E.
    let call_status = vm
        .call_status(installed.code(), call_slot)
        .expect("call slot should expose call status");
    assert_eq!(call_status.execution_count, 1);
    let warmed = tiering
        .snapshot(installed.code())
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
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(6));
    assert_eq!(vm.feedback_warmup_counter(inner_code), Some(2));
    assert!(vm.has_feedback_vector(inner_code));
    // Phase D.1.0: Arithmetic slots have no Rust-side FeedbackSiteState variant.
    // feedback_execution_count returns None; warmup counter is the durable record.
    assert_eq!(
        vm.feedback_execution_count(inner_code, arithmetic_slot),
        None,
        "Arithmetic slots have no Rust-side state after Phase D.1.0"
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

    let mut tiering = Tiering::new();
    tiering.ensure_slot(inner_code);
    assert!(tiering.set_eligible(inner_code, true));
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .with_tiering(&mut tiering)
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(6));
    let snapshot = tiering
        .snapshot(inner_code)
        .expect("inner code should expose tiering state");
    assert_eq!(snapshot.status(), TierStatus::Collecting);
    assert_eq!(snapshot.hotness(), 2);
    assert_eq!(snapshot.feedback_events(), 2);
    // Two executions cross the allocation threshold (= 2), so the external
    // tiering carries the allocated marker.
    assert!(snapshot.warmup_counter() >= 2);
}

#[test]
fn loop_execution_preserves_tier_state_invalidation_resets_hotness() {
    // DSL-0c C6: tier-accounting on backedges deleted with the α path.
    // After DSL-0c the interpreter has no tier-up accounting on backedges —
    // intentional per design §6 + §10 (JIT is out of scope, §2). A loop
    // no longer bumps hotness; only feedback-site events do. This test
    // exercises the same workload as the pre-DSL-0c
    // `loop_backedges_make_eligible_code_ready_and_invalidation_resets_hotness`
    // test, but only checks invariants that survive the backedge-deletion:
    // the tier state remains Collecting (since no backedge events fire),
    // invalidation still resets hotness, and reruns still execute cleanly.
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
    let mut tiering = Tiering::new();
    tiering.ensure_slot(installed.code());

    assert!(tiering.set_eligible(installed.code(), true));
    let first = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .with_tiering(&mut tiering)
        .run()
        .unwrap();
    assert_eq!(first, Value::from_smi(120));

    let after_first = tiering
        .snapshot(installed.code())
        .expect("installed code should expose tiering state");
    // Without backedge accounting the loop alone never reaches
    // ReadyForNative — feedback-site events are the only hotness source.
    assert_eq!(after_first.backedge_events(), 0);
    assert_eq!(after_first.invalidation_epoch(), 0);
    assert_eq!(after_first.native_generation(), None);

    assert!(tiering.invalidate(installed.code()));
    let invalidated = tiering
        .snapshot(installed.code())
        .expect("installed code should expose tiering state");
    assert_eq!(invalidated.status(), TierStatus::Invalidated);
    assert_eq!(invalidated.hotness(), 0);
    assert_eq!(invalidated.invalidation_epoch(), 1);
    assert_eq!(invalidated.native_generation(), None);

    let second = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .with_tiering(&mut tiering)
        .run()
        .unwrap();
    assert_eq!(second, Value::from_smi(120));
    let rewarmed = tiering
        .snapshot(installed.code())
        .expect("installed code should expose tiering state");
    assert_eq!(rewarmed.invalidation_epoch(), 1);
    assert_eq!(rewarmed.backedge_events(), 0);
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
        .run()
        .unwrap();
    assert_eq!(first, Value::from_smi(0));
    assert_eq!(vm.feedback_warmup_counter(callback_code), Some(1));
    assert!(!vm.has_feedback_vector(callback_code));

    let second = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();
    assert_eq!(second, Value::from_smi(0));
    assert_eq!(vm.feedback_warmup_counter(callback_code), Some(2));
    assert!(vm.has_feedback_vector(callback_code));
    // Phase D.1.0: Arithmetic slots have no Rust-side FeedbackSiteState variant.
    // feedback_execution_count returns None; warmup counter is the durable record.
    assert_eq!(
        vm.feedback_execution_count(callback_code, arithmetic_slot),
        None,
        "Arithmetic slots have no Rust-side state after Phase D.1.0"
    );
}

#[test]
fn metadata_table_allocated_at_install_with_correct_per_kind_counts() {
    use crate::vm::metadata_table::MetadataKind;

    let src = r"
        function f(v) { return v; }
        var source = { x: 1, y: 2 };
        source.x + source.y + f(source.x);
    ";
    let unit = compile_test_unit(101, src);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    let table = vm
        .metadata_table(installed.code())
        .expect("MetadataTable should be allocated at install time");

    let entry_fn = vm
        .installed_function(installed.code())
        .expect("installed script should expose its template");
    let mut expected_property: u32 = 0;
    let mut expected_call: u32 = 0;
    let mut expected_arith: u32 = 0;
    let mut expected_comparison: u32 = 0;
    let mut expected_keyed: u32 = 0;
    for descriptor in entry_fn.feedback_sites() {
        match MetadataKind::from_site_kind(descriptor.kind()) {
            MetadataKind::Property => expected_property += 1,
            MetadataKind::Call => expected_call += 1,
            MetadataKind::Arith => expected_arith += 1,
            MetadataKind::Comparison => expected_comparison += 1,
            MetadataKind::KeyedProperty => expected_keyed += 1,
        }
    }

    // Guard: the script must exercise these kinds for the test to be meaningful.
    assert!(
        expected_property >= 2,
        "expected at least 2 property loads; got {expected_property}"
    );
    assert!(
        expected_call >= 1,
        "expected at least 1 call; got {expected_call}"
    );
    assert!(
        expected_arith >= 1,
        "expected at least 1 arithmetic op; got {expected_arith}"
    );

    assert_eq!(
        table.run_len_for_kind(MetadataKind::Property),
        expected_property,
        "Property run length mismatch"
    );
    assert_eq!(
        table.run_len_for_kind(MetadataKind::Call),
        expected_call,
        "Call run length mismatch"
    );
    assert_eq!(
        table.run_len_for_kind(MetadataKind::Arith),
        expected_arith,
        "Arith run length mismatch"
    );
    assert_eq!(
        table.run_len_for_kind(MetadataKind::Comparison),
        expected_comparison,
        "Comparison run length mismatch"
    );
    assert_eq!(
        table.run_len_for_kind(MetadataKind::KeyedProperty),
        expected_keyed,
        "KeyedProperty run length mismatch"
    );
}

#[test]
fn metadata_table_kind_offsets_partition_buffer() {
    use crate::vm::metadata_table::MetadataKind;

    let src = r"
        function f(v) { return v; }
        var source = { a: 1, b: 2, c: 3 };
        source.a + source.b + source.c;
    ";
    let unit = compile_test_unit(102, src);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    let table = vm
        .metadata_table(installed.code())
        .expect("MetadataTable should be allocated at install time");

    let property_off = table.kind_offset(MetadataKind::Property) as usize;
    assert!(
        property_off.is_multiple_of(8),
        "Property run start ({property_off}) is not 8-aligned"
    );

    let mut prev_end = 0usize;
    for kind in [
        MetadataKind::Property,
        MetadataKind::Call,
        MetadataKind::Arith,
        MetadataKind::Comparison,
        MetadataKind::KeyedProperty,
    ] {
        let off = table.kind_offset(kind) as usize;
        assert!(
            off >= prev_end,
            "{kind:?} offset {off} overlaps previous kind run end {prev_end}"
        );
        prev_end = off + (table.run_len_for_kind(kind) as usize) * kind.stride_bytes();
    }
    assert!(
        prev_end <= table.buffer().len(),
        "kind runs extend past buffer end (prev_end={prev_end}, buf_len={})",
        table.buffer().len()
    );
}

#[test]
fn metadata_table_in_kind_indices_are_monotone_per_kind() {
    use crate::vm::metadata_table::{MetadataKind, METADATA_KIND_COUNT};

    let src = r"
        var source = { x: 1, y: 2 };
        source.x + source.y;
    ";
    let unit = compile_test_unit(103, src);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    let table = vm
        .metadata_table(installed.code())
        .expect("MetadataTable should be allocated at install time");

    let entry_fn = vm
        .installed_function(installed.code())
        .expect("installed script should expose its template");

    let mut seen_per_kind = [0u32; METADATA_KIND_COUNT];
    for descriptor in entry_fn.feedback_sites() {
        let mk = MetadataKind::from_site_kind(descriptor.kind());
        let expected = seen_per_kind[mk.index()];
        let actual = table.in_kind_index_for_slot_with_kind(descriptor.slot().get(), mk);
        assert_eq!(
            actual,
            expected,
            "slot {} kind {:?}: expected in-kind index {expected} but got {actual}",
            descriptor.slot().get(),
            descriptor.kind(),
        );
        seen_per_kind[mk.index()] += 1;
    }

    // Guard: the script must produce ≥2 Property slots to make monotonicity meaningful.
    assert!(
        seen_per_kind[MetadataKind::Property.index()] >= 2,
        "expected at least 2 Property slots to verify monotone ordering; got {}",
        seen_per_kind[MetadataKind::Property.index()]
    );
}

// C6: GC sweep releases MetadataTable entries for dead code objects.
//
// Strategy: install a script so a MetadataTable is allocated, confirm it
// exists, then call `prune_dead_code_metadata_tables` with an `is_live`
// predicate that reports the code as dead. The table entry must become `None`.
//
// This mirrors the B6 test pattern (`prune_dead_code_polymorphic_chains`),
// using a direct-unit call instead of a full GC cycle to keep the test
// isolated from GC timing.
#[test]
fn c6_metadata_table_released_when_code_is_pruned_dead() {
    let unit = compile_test_unit(45_002, "var source = { x: 1 }; source.x;");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    // Run once so execution metadata is exercised.
    let _ = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    let code = installed.code();

    // Pre-condition: MetadataTable exists.
    assert!(
        vm.metadata_table(code).is_some(),
        "C6: MetadataTable should be present after install and run"
    );

    // Simulate code death: prune_dead_code_metadata_tables treats all code as dead.
    vm.prune_dead_code_metadata_tables(|_code| false);

    // Post-condition: the table slot must be cleared.
    assert!(
        vm.metadata_table(code).is_none(),
        "C6: MetadataTable must be released when code is pruned dead"
    );
}

// C6b: GC sweep retains MetadataTable entries for live code objects.
//
// Mirror of C6: `prune_dead_code_metadata_tables` with an `is_live` predicate
// that keeps the installed code alive — the table must survive.
#[test]
fn c6b_metadata_table_retained_when_code_is_live() {
    let unit = compile_test_unit(45_003, "var source = { x: 1 }; source.x;");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    let _ = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    let code = installed.code();

    assert!(
        vm.metadata_table(code).is_some(),
        "C6b: MetadataTable should be present before prune"
    );

    // Simulate a GC sweep where this code is still live.
    vm.prune_dead_code_metadata_tables(|c| c == code);

    assert!(
        vm.metadata_table(code).is_some(),
        "C6b: MetadataTable must be retained when code is live"
    );
}

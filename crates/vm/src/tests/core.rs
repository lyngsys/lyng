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
fn completed_bytecode_calls_keep_register_storage_inactive_for_reuse() {
    let unit = compile_test_unit(
        149,
        r"
        function add(left, right) {
            return left + right;
        }

        var total = 0;
        for (var i = 0; i < 4; i = i + 1) {
            total = add(total, i);
        }
        total;
        ",
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(6));
    assert!(vm.register_stack().is_empty());
    assert!(vm.register_stack_storage_len_for_tests() > 0);
}

#[test]
fn small_arity_calls_preserve_call_semantics() {
    let unit = compile_test_unit(
        150,
        r#"
        function score(a, b, c, d) {
            return arguments.length * 1000
                + (a || 0)
                + (b || 0) * 10
                + (c || 0) * 100
                + (d || 0) * 1000;
        }

        var receiver = {
            base: 50,
            plus(a, b, c) {
                return this.base + a + b + c;
            }
        };
        var bound = receiver.plus.bind({ base: 70 }, 1);
        var proxy = new Proxy(function(a, b) {
            return this.base + a + b;
        }, {
            apply(target, thisArg, args) {
                return Reflect.apply(target, thisArg, args) + args.length;
            }
        });
        function Constructor(a, b, c) {
            this.total = a + b + c;
        }
        function tail(n, acc) {
            "use strict";
            if (n === 0) {
                return acc;
            }
            return tail(n - 1, acc + n);
        }

        score() === 0
            && score(4) === 1004
            && score(4, 5) === 2054
            && score(4, 5, 6) === 3654
            && score(4, 5, 6, 7) === 11654
            && score(...[4]) === 1004
            && receiver.plus(1, 2, 3) === 56
            && bound(2, 3) === 76
            && receiver.plus.call(receiver, 4, 5, 6) === 65
            && proxy.call(receiver, 8, 9) === 69
            && new Constructor(1, 2, 3).total === 6
            && tail(3, 0) === 6;
        "#,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).run().unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_builder_runs_default_case() {
    let unit = compile_test_unit(151, "2 + 3;");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).run().unwrap();

    assert_eq!(result, Value::from_smi(5));
}

#[test]
fn evaluate_installed_builder_runs_default_case() {
    let unit = compile_test_unit(162, "7 * 6;");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(42));
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
            lyng_bytecode::FeedbackSiteMetadata::None,
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
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(41));
    assert!(vm.frames().is_empty());
    assert!(vm.register_stack().is_empty());
    assert!(agent.current_execution_context().is_none());
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn vm_opcode_dispatch_counters_are_opt_in_and_record_executed_opcodes() {
    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(13).unwrap(),
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
    let unit = CompiledScriptUnit::new(SourceId::new(13), function.id(), vec![function]);

    {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent.default_realm().expect("default realm should exist");
        let mut vm = Vm::new();
        let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

        // DSL-1 Phase 1.B.0 Task 1: counters are now always allocated;
        // `opcode_dispatch_counts()` returns `Some` even before the
        // first dispatch. Total starts at 0.
        assert_eq!(vm.opcode_counters().dispatch_counts().total(), 0);
        let result = vm
            .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap();
        assert_eq!(result, Value::from_smi(42));
        // Until Task 4 wires the asm-side increment, the dispatch
        // bank stays at zero across runs. Reset to be explicit before
        // the enable/run round.
        vm.opcode_counters_mut().reset_dispatch_counts();

        let result = vm
            .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap();
        assert_eq!(result, Value::from_smi(42));

        let counts = vm.opcode_counters().dispatch_counts();
        assert_eq!(counts.total(), 4);
        assert_eq!(counts.count(Opcode::LdaOne), 1);
        assert_eq!(counts.count(Opcode::AddSmi), 1);
        assert_eq!(counts.count(Opcode::StoreLocal2), 1);
        assert_eq!(counts.count(Opcode::Return), 1);
        assert_eq!(counts.top(2)[0].opcode(), Opcode::AddSmi);

        vm.opcode_counters_mut().reset_dispatch_counts();
        assert_eq!(vm.opcode_counters().dispatch_counts().total(), 0);
    }
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn current_opcode_cell_is_published_after_a_run() {
    use std::sync::atomic::Ordering;
    // Build + run a tiny bytecode unit, then assert the asm dispatch prologue
    // published at least one opcode (cell moved off the idle sentinel). This
    // proves the `str [x9, #6144]` fired.
    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(17).unwrap(),
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
    let unit = CompiledScriptUnit::new(SourceId::new(17), function.id(), vec![function]);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();
    assert_eq!(result, Value::from_smi(42));

    let raw = vm
        .opcode_counters()
        .dispatch_banks()
        .current_opcode_cell()
        .load(Ordering::Relaxed);
    assert_ne!(
        raw,
        crate::opcode_counts::CURRENT_OPCODE_IDLE,
        "asm prologue should have published an opcode"
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn evaluate_builder_with_opcode_counters_redirects_asm_writes_to_external_store() {
    use lyng_vm::OpcodeCounters;

    let mut function_builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(91).unwrap(),
        BytecodeFunctionKind::Script,
    );
    function_builder
        .alloc_registers(2)
        .expect("test bytecode registers should allocate");
    function_builder
        .emit_abx(Opcode::LoadOne, 0, 0)
        .expect("test bytecode should build");
    function_builder
        .emit_abc(Opcode::AddSmi, 1, 0, 41)
        .expect("test bytecode should build");
    function_builder
        .emit_ax(Opcode::Return, 1)
        .expect("test bytecode should build");
    let function = function_builder
        .finish()
        .expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(91), function.id(), vec![function]);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    // Prime the VM's internal counters so we can prove the external
    // store, not the VM's, accumulates the asm-driven writes during
    // the run with the builder hook installed.
    vm.opcode_counters_mut().reset_dispatch_counts();
    assert_eq!(vm.opcode_counters().dispatch_counts().total(), 0);

    let mut external = OpcodeCounters::new();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .with_opcode_counters(&mut external)
        .run()
        .unwrap();
    assert_eq!(result, Value::from_smi(42));

    assert_eq!(
        external.dispatch_counts().count(Opcode::AddSmi),
        1,
        "asm-driven dispatch writes during .run() must land in the external OpcodeCounters"
    );
    assert_eq!(
        vm.opcode_counters().dispatch_counts().total(),
        0,
        "VM's internal counters must be restored (zero) after the run completes"
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn add_smi_hit_avoids_semantic_slow_path() {
    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(18).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(2)
        .expect("test bytecode registers should allocate");
    builder
        .emit_abx(Opcode::LoadOne, 0, 0)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::AddSmi, 1, 0, 41)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 1)
        .expect("test bytecode should build");
    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(18), function.id(), vec![function]);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let counters = vm.opcode_counters_mut();
    counters.enable_slow_path();
    counters.reset();

    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();
    assert_eq!(result, Value::from_smi(42));

    let counters = vm.opcode_counters();
    let dispatch = counters.dispatch_counts();
    let slow_path = counters
        .slow_path_counts()
        .expect("slow-path counters should be enabled");
    assert_eq!(dispatch.count(Opcode::AddSmi), 1);
    assert_eq!(
        slow_path.semantic(Opcode::AddSmi),
        0,
        "AddSmi LLInt SMI hit should avoid the semantic slow bridge"
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn jump_i24_forward_hit_avoids_semantic_slow_path() {
    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(18).unwrap(),
        None,
        ArgumentsMode::None,
    )
    .with_register_counts(1, 0)
    .with_instructions(vec![
        Instruction::ax(Opcode::Jump, 4),
        Instruction::abx(Opcode::LoadSmi, 0, 1),
        Instruction::abx(Opcode::LoadSmi, 0, 42),
        Instruction::ax(Opcode::Return, 0),
    ]);
    let unit = CompiledScriptUnit::new(SourceId::new(18), function.id(), vec![function]);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let counters = vm.opcode_counters_mut();
    counters.enable_slow_path();
    counters.reset();

    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();
    assert_eq!(result, Value::from_smi(42));

    let counters = vm.opcode_counters();
    let dispatch = counters.dispatch_counts();
    let slow_path = counters
        .slow_path_counts()
        .expect("slow-path counters should be enabled");
    assert_eq!(dispatch.count(Opcode::Jump), 1);
    assert_eq!(
        slow_path.semantic(Opcode::Jump),
        0,
        "Jump i24 forward hit should avoid the semantic slow bridge"
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn jump_i24_backward_hit_without_pending_poll_avoids_semantic_slow_path() {
    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(21).unwrap(),
        None,
        ArgumentsMode::None,
    )
    .with_register_counts(1, 0)
    .with_instructions(vec![
        Instruction::ax(Opcode::Jump, 8),
        Instruction::abx(Opcode::LoadSmi, 0, 42),
        Instruction::ax(Opcode::Return, 0),
        Instruction::ax(Opcode::Jump, -12),
    ]);
    let unit = CompiledScriptUnit::new(SourceId::new(21), function.id(), vec![function]);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let counters = vm.opcode_counters_mut();
    counters.enable_slow_path();
    counters.reset();

    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();
    assert_eq!(result, Value::from_smi(42));

    let counters = vm.opcode_counters();
    let dispatch = counters.dispatch_counts();
    let slow_path = counters
        .slow_path_counts()
        .expect("slow-path counters should be enabled");
    assert_eq!(dispatch.count(Opcode::Jump), 2);
    assert_eq!(
        slow_path.semantic(Opcode::Jump),
        0,
        "Jump i24 backward hit without a pending poll should avoid the semantic slow bridge"
    );
    assert_eq!(
        slow_path.safepoint(Opcode::Jump),
        0,
        "Jump i24 backward hit should poll without taking the safepoint slow path when clear"
    );
}

#[cfg(feature = "diagnostic-counters")]
struct ResumeDebugHook;

#[cfg(feature = "diagnostic-counters")]
impl VmDebugHook for ResumeDebugHook {
    fn on_pause(&mut self, _context: VmDebugPauseContext<'_>) -> VmDebugCommand {
        VmDebugCommand::Resume
    }
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn jump_i24_backward_pending_debug_uses_safepoint_not_semantic_slow_path() {
    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(24).unwrap(),
        None,
        ArgumentsMode::None,
    )
    .with_register_counts(1, 0)
    .with_instructions(vec![
        Instruction::ax(Opcode::Jump, 8),
        Instruction::abx(Opcode::LoadSmi, 0, 7),
        Instruction::ax(Opcode::Return, 0),
        Instruction::ax(Opcode::Jump, -12),
    ]);
    let unit = CompiledScriptUnit::new(SourceId::new(24), function.id(), vec![function]);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let mut debugger = VmDebugger::new(ResumeDebugHook);
    debugger.request_pause_at(installed.code(), 12);

    let counters = vm.opcode_counters_mut();
    counters.enable_slow_path();
    counters.reset();

    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .with_debugger(&mut debugger)
        .run()
        .unwrap();
    assert_eq!(result, Value::from_smi(7));

    let slow_path = vm
        .opcode_counters()
        .slow_path_counts()
        .expect("slow-path counters should be enabled");
    assert_eq!(
        slow_path.safepoint(Opcode::Jump),
        1,
        "pending debugger work should enter the Jump safepoint slow path"
    );
    assert_eq!(
        slow_path.semantic(Opcode::Jump),
        0,
        "pending debugger work should not force the Jump semantic slow bridge"
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn jump_if_false8_bool_hit_avoids_semantic_slow_path() {
    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(19).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(2)
        .expect("test bytecode registers should allocate");
    builder
        .emit_abx(Opcode::LoadFalse, 0, 0)
        .expect("test bytecode should build");
    let jump = builder
        .emit_cond_jump_placeholder(Opcode::JumpIfFalse, 0)
        .expect("test bytecode should build");
    builder
        .emit_abx(Opcode::LoadOne, 1, 0)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 1)
        .expect("test bytecode should build");
    let target = builder
        .current_offset()
        .expect("test bytecode offset should build");
    builder
        .emit_abx(Opcode::LoadSmi, 1, 42)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 1)
        .expect("test bytecode should build");
    builder
        .patch_jump_to(jump, target)
        .expect("test bytecode jump should patch");

    let function = builder.finish().expect("test bytecode should build");
    assert!(
        function
            .instructions()
            .iter()
            .any(|instruction| instruction.opcode() == Opcode::JumpIfFalse8),
        "short boolean branch test should encode a JumpIfFalse8"
    );
    let unit = CompiledScriptUnit::new(SourceId::new(19), function.id(), vec![function]);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let counters = vm.opcode_counters_mut();
    counters.enable_slow_path();
    counters.reset();

    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();
    assert_eq!(result, Value::from_smi(42));

    let counters = vm.opcode_counters();
    let dispatch = counters.dispatch_counts();
    let slow_path = counters
        .slow_path_counts()
        .expect("slow-path counters should be enabled");
    assert_eq!(dispatch.count(Opcode::JumpIfFalse8), 1);
    assert_eq!(
        slow_path.semantic(Opcode::JumpIfFalse8),
        0,
        "JumpIfFalse8 boolean hit should avoid the semantic slow bridge"
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn vm_lda_star_pair_dispatches_each_handler_under_dsl() {
    // Originally Phase 4b's `vm_star_fusion_elides_star_dispatch_after_lda`
    // regression: the α dispatch loop's `dispatch_next_with_value!` peephole
    // fused `LdaX; StarN` so the pair produced a single dispatch (LdaOne)
    // with Star2 elided. DSL-0c deleted the α loop in favor of the
    // standalone asm-DSL handlers, and the DSL has no equivalent fusion
    // peephole — each opcode runs its own handler and bumps its own
    // dispatch counter. DSL-1 Phase 1.B.0 Task 4 wired the asm-side
    // increment, so the LdaOne / Star2 split is now directly observable
    // here. Reintroducing fusion is a future optimisation; for now the
    // test pins the current expected counts (each handler fires once) so
    // a regression would be caught.
    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(17).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(3)
        .expect("test bytecode registers should allocate");
    // `LoadOne r0` compacts to `LdaOne` (1-byte, accumulator target).
    builder
        .emit_abx(Opcode::LoadOne, 0, 0)
        .expect("test bytecode should build");
    // `Move r2 ← r0` compacts to `Star2` (1-byte, accumulator → r2).
    builder
        .emit_abc(Opcode::Move, 2, 0, 0)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 2)
        .expect("test bytecode should build");
    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(17), function.id(), vec![function]);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();
    assert_eq!(result, Value::from_smi(1));

    let counts = vm.opcode_counters().dispatch_counts();
    assert_eq!(
        counts.count(Opcode::LdaOne),
        1,
        "LdaOne should dispatch once"
    );
    assert_eq!(
        counts.count(Opcode::Star2),
        1,
        "Star2 dispatches under the DSL (no Lda;Star fusion peephole)"
    );
    assert_eq!(
        counts.count(Opcode::Return),
        1,
        "Return should still dispatch"
    );
    assert_eq!(
        counts.total(),
        3,
        "DSL dispatches LdaOne, Star2, and Return individually"
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn smi_equal_hit_avoids_semantic_slow_path() {
    let unit = compile_test_unit(545, "left == right;");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let left_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "left"));
    let right_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "right"));
    install_global_value(agent, &realm, left_name, Value::from_smi(7));
    install_global_value(agent, &realm, right_name, Value::from_smi(7));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let counters = vm.opcode_counters_mut();
    counters.enable_slow_path();
    counters.reset();

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_bool(true)
    );

    let counters = vm.opcode_counters();
    let dispatch = counters.dispatch_counts();
    let slow_path = counters
        .slow_path_counts()
        .expect("slow-path counters should be enabled");
    assert_eq!(dispatch.count(Opcode::Equal), 1);
    assert_eq!(
        slow_path.semantic(Opcode::Equal),
        0,
        "SMI Equal LLInt hit should avoid the semantic slow bridge"
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn nullish_equal_hit_avoids_semantic_slow_path() {
    let unit = compile_test_unit(547, "left == right;");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let left_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "left"));
    let right_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "right"));
    install_global_value(agent, &realm, left_name, Value::null());
    install_global_value(agent, &realm, right_name, Value::undefined());

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let counters = vm.opcode_counters_mut();
    counters.enable_slow_path();
    counters.reset();

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_bool(true)
    );

    let counters = vm.opcode_counters();
    let dispatch = counters.dispatch_counts();
    let slow_path = counters
        .slow_path_counts()
        .expect("slow-path counters should be enabled");
    assert_eq!(dispatch.count(Opcode::Equal), 1);
    assert_eq!(
        slow_path.semantic(Opcode::Equal),
        0,
        "nullish Equal LLInt hit should avoid the semantic slow bridge"
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn primitive_strict_equal_hit_avoids_semantic_slow_path() {
    let unit = compile_test_unit(546, "left === right;");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let left_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "left"));
    let right_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "right"));
    install_global_value(agent, &realm, left_name, Value::from_bool(true));
    install_global_value(agent, &realm, right_name, Value::from_bool(true));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let counters = vm.opcode_counters_mut();
    counters.enable_slow_path();
    counters.reset();

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap(),
        Value::from_bool(true)
    );

    let counters = vm.opcode_counters();
    let dispatch = counters.dispatch_counts();
    let slow_path = counters
        .slow_path_counts()
        .expect("slow-path counters should be enabled");
    assert_eq!(dispatch.count(Opcode::StrictEqual), 1);
    assert_eq!(
        slow_path.semantic(Opcode::StrictEqual),
        0,
        "primitive StrictEqual LLInt hit should avoid the semantic slow bridge"
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn generic_call_with_more_than_three_args_also_avoids_scratch_pushes() {
    let unit = compile_test_unit(
        153,
        r"
        var add5 = (a, b, c, d, e) => a + b + c + d + e;
        var total = 0;
        for (var i = 0; i < 8; i = i + 1) {
            total = total + add5(i, i, i, i, i);
        }
        total;
        ",
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    vm.opcode_counters_mut().enable_call_argument_copy();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    // 5 * sum(0..7) = 5 * 28 = 140
    assert_eq!(result, Value::from_smi(140));

    let counts = vm
        .opcode_counters()
        .call_argument_copy_counts()
        .expect("enabled call argument copy counters should produce a snapshot");
    assert_eq!(
        counts.scratch_pushes(),
        0,
        "ordinary non-spread bytecode-to-bytecode `Call` should not push into argument_scratch \
         even when argument_count > 3"
    );
    assert_eq!(
        counts.frame_copies(),
        40,
        "each ordinary Call5 should copy exactly its 5 arguments into the callee frame"
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn spread_call_still_materializes_into_argument_scratch() {
    let unit = compile_test_unit(
        154,
        r"
        var add3 = (a, b, c) => a + b + c;
        var args = [1, 2, 3];
        add3(...args);
        ",
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    vm.opcode_counters_mut().enable_call_argument_copy();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(6));

    let counts = vm
        .opcode_counters()
        .call_argument_copy_counts()
        .expect("enabled call argument copy counters should produce a snapshot");
    assert!(
        counts.scratch_pushes() > 0,
        "spread calls must materialize into argument_scratch; the shortcut does not handle spread"
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn bound_function_call_still_materializes_into_argument_scratch() {
    let unit = compile_test_unit(
        155,
        r"
        function plus(a, b, c) { return this.base + a + b + c; }
        var bound = plus.bind({ base: 10 }, 1);
        bound(2, 3);
        ",
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    vm.opcode_counters_mut().enable_call_argument_copy();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(16));

    let counts = vm
        .opcode_counters()
        .call_argument_copy_counts()
        .expect("enabled call argument copy counters should produce a snapshot");
    assert!(
        counts.scratch_pushes() > 0,
        "bound function calls need argument prepending and must stay on the Vec path"
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn nonstrict_function_referencing_arguments_object_stays_on_slow_path() {
    let unit = compile_test_unit(
        156,
        r"
        function variadic(a, b, c) {
            // Reference `arguments` to force ArgumentsMode != None
            return arguments.length * 100 + a + b + c;
        }
        variadic(1, 2, 3);
        ",
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    vm.opcode_counters_mut().enable_call_argument_copy();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(306));

    let counts = vm
        .opcode_counters()
        .call_argument_copy_counts()
        .expect("enabled call argument copy counters should produce a snapshot");
    assert!(
        counts.scratch_pushes() > 0,
        "functions that materialize an `arguments` object must stay on the slow path"
    );
}

#[test]
fn specialized_path_handles_iife_closure_helpers_calling_each_other() {
    // Repro for Test262 harness deepEqual failure: tight pattern of small
    // arrow + named helpers calling each other through `||` short-circuit
    // chains. All eligible for the call-arg direct path.
    let unit = compile_test_unit(
        158,
        r"
        (function() {
          var EQUAL = 1;
          var NOT_EQUAL = -1;
          var UNKNOWN = 0;

          function compareEquality(a, b, cache) {
            return compareIf(a, b, isOptional, compareOptionality)
              || compareIf(a, b, isPrimitiveEquatable, comparePrimitiveEquality)
              || NOT_EQUAL;
          }

          function compareIf(a, b, test, compare, cache) {
            return !test(a)
              ? !test(b) ? UNKNOWN : NOT_EQUAL
              : !test(b) ? NOT_EQUAL : compare(a, b, cache);
          }

          function tryCompareStrictEquality(a, b) {
            return a === b ? EQUAL : UNKNOWN;
          }

          function isOptional(value) {
            return value === undefined || value === null;
          }

          function compareOptionality(a, b) {
            return tryCompareStrictEquality(a, b) || NOT_EQUAL;
          }

          function isPrimitiveEquatable(value) {
            return typeof value === 'number' || typeof value === 'string';
          }

          function comparePrimitiveEquality(a, b) {
            return tryCompareStrictEquality(a, b) || NOT_EQUAL;
          }

          var results = [
            compareEquality(1, 1),
            compareEquality(1, 2),
            compareEquality('a', 'a'),
            compareEquality('a', 'b'),
            compareEquality(undefined, undefined),
          ];
          // r1=1, r2=-1, r3=1, r4=-1, r5=1 → total = 1
          return results.reduce(function (acc, x) { return acc + x; }, 0);
        })();
        ",
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).run().unwrap();
    assert_eq!(result, Value::from_smi(1));
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn rest_parameter_function_stays_on_slow_path() {
    let unit = compile_test_unit(
        157,
        r"
        var sumRest = (head, ...tail) => {
            var sum = head;
            for (var i = 0; i < tail.length; i = i + 1) {
                sum = sum + tail[i];
            }
            return sum;
        };
        sumRest(1, 2, 3, 4);
        ",
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    vm.opcode_counters_mut().enable_call_argument_copy();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(10));

    let counts = vm
        .opcode_counters()
        .call_argument_copy_counts()
        .expect("enabled call argument copy counters should produce a snapshot");
    assert!(
        counts.scratch_pushes() > 0,
        "rest-parameter functions need a materialized argument slice"
    );
}

#[cfg(feature = "diagnostic-counters")]
#[test]
fn ordinary_bytecode_calls_avoid_argument_scratch_pushes() {
    let unit = compile_test_unit(
        152,
        r"
        var add3 = (a, b, c) => a + b + c;
        var total = 0;
        for (var i = 0; i < 10; i = i + 1) {
            total = total + add3(i, i + 1, i + 2);
        }
        total;
        ",
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    vm.opcode_counters_mut().enable_call_argument_copy();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    // sum over i in 0..10 of (i + (i+1) + (i+2)) = 3*(0+1+...+9) + 30 = 135 + 30 = 165
    assert_eq!(result, Value::from_smi(165));

    let counts = vm
        .opcode_counters()
        .call_argument_copy_counts()
        .expect("enabled call argument copy counters should produce a snapshot");
    assert_eq!(
        counts.scratch_pushes(),
        0,
        "ordinary bytecode-to-bytecode Call3 (arrow, non-spread, non-bound, no `arguments`) \
         should not push into argument_scratch"
    );
    // Each iteration: 3 args copied into callee frame, 10 iterations = 30.
    assert_eq!(
        counts.frame_copies(),
        30,
        "each ordinary Call3 should copy exactly its 3 arguments into the callee frame"
    );
}

#[test]
fn vm_loop_backedges_poll_active_incremental_major_mark() {
    let unit = compile_test_unit(
        151,
        r"
        var i = 0;
        while (i < 3) {
            i = i + 1;
        }
        i;
        ",
    );

    {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent.default_realm().expect("default realm should exist");
        let mut vm = Vm::new();
        let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

        let roots = PrimitiveRoots::new();
        let live = agent.heap_mut().mutator().alloc_string(
            StringEncoding::Latin1,
            4,
            b"live",
            None,
            AllocationLifetime::Default,
        );
        let _rooted = roots.root_string(live);
        agent.heap_mut().set_major_mark_slice_budget(1);
        assert!(agent.heap_mut().begin_incremental_mark(&roots));
        assert_eq!(
            agent.heap().active_incremental_mark_pending_work_items(),
            Some(1)
        );

        let result = vm
            .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run()
            .unwrap();

        assert_eq!(result, Value::from_smi(3));
        assert_eq!(
            agent.heap().active_incremental_mark_pending_work_items(),
            Some(0)
        );
    }
}

#[test]
fn vm_full_jump_backedges_poll_active_incremental_major_mark() {
    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(23).unwrap(),
        None,
        ArgumentsMode::None,
    )
    .with_kind(BytecodeFunctionKind::Script)
    .with_register_counts(1, 0)
    .with_instructions(vec![
        Instruction::ax(Opcode::Jump, 8),
        Instruction::abx(Opcode::LoadSmi, 0, 7),
        Instruction::ax(Opcode::Return, 0),
        Instruction::ax(Opcode::Jump, -12),
    ]);
    let unit = CompiledScriptUnit::new(SourceId::new(23), function.id(), vec![function]);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    let roots = PrimitiveRoots::new();
    let live = agent.heap_mut().mutator().alloc_string(
        StringEncoding::Latin1,
        4,
        b"live",
        None,
        AllocationLifetime::Default,
    );
    let _rooted = roots.root_string(live);
    agent.heap_mut().set_major_mark_slice_budget(1);
    assert!(agent.heap_mut().begin_incremental_mark(&roots));
    assert_eq!(
        agent.heap().active_incremental_mark_pending_work_items(),
        Some(1)
    );

    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(7));
    assert_eq!(
        agent.heap().active_incremental_mark_pending_work_items(),
        Some(0)
    );
}

#[test]
fn vm_dsl_poll_pending_tracks_active_incremental_major_mark() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let mut vm = Vm::new();

    let roots = PrimitiveRoots::new();
    let live = agent.heap_mut().mutator().alloc_string(
        StringEncoding::Latin1,
        4,
        b"live",
        None,
        AllocationLifetime::Default,
    );
    let _rooted = roots.root_string(live);
    agent.heap_mut().set_major_mark_slice_budget(1);
    assert!(agent.heap_mut().begin_incremental_mark(&roots));

    vm.refresh_dsl_poll_pending_for_agent(agent);
    assert_eq!(vm.dsl_poll_pending, 1);

    Vm::poll_incremental_mark_safepoint(agent);
    vm.refresh_dsl_poll_pending_for_agent(agent);
    assert_eq!(vm.dsl_poll_pending, 0);
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
        .run()
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
fn vm_rejects_jump_targets_outside_instruction_stream() {
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
        Instruction::ax(Opcode::Jump, 4),
        Instruction::ax(Opcode::ReturnUndefined, 0),
    ]);
    let unit = CompiledScriptUnit::new(SourceId::new(20), function.id(), vec![function]);

    let mut vm = Vm::new();
    let error = vm
        .install_script(agent, realm.id(), &unit)
        .expect_err("invalid jump targets should be rejected at install");

    assert!(matches!(
        error,
        VmError::InvalidJumpTarget {
            code,
            instruction_offset: 0,
            target_offset: 8
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
    let _ = vm.evaluate_script(agent, realm, &unit).run().unwrap();

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
            .and_then(lyng_objects::FunctionObjectData::entry),
        Some(FunctionEntryIdentity::Bytecode(_))
    ));
    assert!(matches!(
        agent
            .objects()
            .function_data(setter_object)
            .and_then(lyng_objects::FunctionObjectData::entry),
        Some(FunctionEntryIdentity::Bytecode(_))
    ));

    let Some(FunctionEntryIdentity::Bytecode(getter_code)) = agent
        .objects()
        .function_data(getter_object)
        .and_then(lyng_objects::FunctionObjectData::entry)
    else {
        panic!("getter should remain backed by installed bytecode");
    };
    let getter_function = vm
        .installed_function(getter_code)
        .expect("getter bytecode should stay installed");
    let getter_environment = agent
        .objects()
        .function_data(getter_object)
        .and_then(lyng_objects::FunctionObjectData::environment)
        .expect("getter closure should preserve its outer environment");
    let getter_result = vm
        .evaluate_installed(
            agent,
            InstalledCode::new(getter_code, getter_function.id()),
            getter_environment,
            getter_environment,
        )
        .run()
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

    let result = vm.evaluate_script(agent, realm, &unit).run().unwrap();
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
        .bootstrap_realm(agent, realm.id(), lyng_builtins::BootstrapMode::SpecOnly)
        .expect("bootstrap should succeed");

    assert!(lyng_ops::object::to_object(agent, realm.id(), Value::from_smi(7)).is_ok());
    assert!(lyng_ops::object::to_object(agent, realm.id(), Value::from_string_ref(string)).is_ok());
    assert!(lyng_ops::object::to_object(agent, realm.id(), Value::from_bigint_ref(bigint)).is_ok());
}

struct WrapperPrimitiveProbe<'a> {
    agent: &'a mut lyng_env::Agent,
    called: bool,
}

impl lyng_ops::object::ToPrimitiveContext for WrapperPrimitiveProbe<'_> {
    type Error = lyng_types::AbruptCompletion;

    fn agent(&mut self) -> &mut lyng_env::Agent {
        self.agent
    }

    fn abrupt(&mut self, completion: lyng_types::AbruptCompletion) -> Self::Error {
        completion
    }

    fn type_error(&mut self) -> Self::Error {
        lyng_ops::errors::throw_type_error(self.agent)
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
            return Err(lyng_ops::errors::throw_type_error(self.agent));
        };
        if self.agent.objects().function_data(object).is_some() {
            Ok(object)
        } else {
            Err(lyng_ops::errors::throw_type_error(self.agent))
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
            return Err(lyng_ops::errors::throw_type_error(self.agent));
        };
        self.agent
            .objects()
            .primitive_wrapper_value(self.agent.heap().view(), object)
            .ok_or_else(|| lyng_ops::errors::throw_type_error(self.agent))
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
        .bootstrap_realm(agent, realm.id(), lyng_builtins::BootstrapMode::SpecOnly)
        .expect("bootstrap should succeed");

    let string_wrapper =
        lyng_ops::object::to_object(agent, realm.id(), Value::from_string_ref(string)).unwrap();
    let mut probe = WrapperPrimitiveProbe {
        agent,
        called: false,
    };

    assert_eq!(
        lyng_ops::object::to_primitive(
            &mut probe,
            Value::from_object_ref(string_wrapper),
            lyng_ops::object::ToPrimitiveHint::Number,
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
        .bootstrap_realm(agent, realm.id(), lyng_builtins::BootstrapMode::SpecOnly)
        .expect("bootstrap should succeed");
    let _ = vm.install_script(agent, realm.id(), &unit).unwrap();
    vm.instantiate_global_script(agent, &realm, unit.instantiation_plan())
        .unwrap();

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
        .bootstrap_realm(agent, realm.id(), lyng_builtins::BootstrapMode::SpecOnly)
        .expect("bootstrap should succeed");
    let _ = vm.install_script(agent, realm.id(), &unit).unwrap();
    vm.instantiate_global_script(agent, &realm, unit.instantiation_plan())
        .unwrap();

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
        .run()
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
        .run()
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
        .run()
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
        .run()
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
    let result = vm.evaluate_script(agent, realm, &unit).run().unwrap();
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
            &mut NoopAdaptiveProtoLoadDispatch,
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
            .run()
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
        .run()
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
        .run()
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
        lyng_types::BuiltinFunctionId::from_raw(9).unwrap(),
    )])
    .with_instructions(vec![
        Instruction::abx(Opcode::LoadConst, 0, 0),
        Instruction::ax(Opcode::Return, 0),
    ]);
    let unit = CompiledScriptUnit::new(SourceId::new(14), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .run(),
        Err(VmError::UnsupportedConstant {
            code: installed.code(),
            index: 0,
            constant: ConstantValue::Builtin(lyng_types::BuiltinFunctionId::from_raw(9).unwrap()),
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
        .run()
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
        .run()
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
        .run()
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
        .run()
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

    let result = vm.evaluate_script(agent, realm, &unit).run().unwrap();

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
        .run()
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
        .run()
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
        .run()
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
        &mut NoopAdaptiveProtoLoadDispatch,
    )
    .unwrap());
    assert!(ordinary_create_data_property(
        agent,
        realm.global_object(),
        PropertyKey::from_atom(source_name),
        Value::from_object_ref(object),
        AllocationLifetime::Default,
        &mut NoopAdaptiveProtoLoadDispatch,
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
            .run()
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
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(13));
}

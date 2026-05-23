//! Cross-crate coverage for the public compile, install, and evaluate pipeline.

use lyng_js_bytecode::{
    disassemble, BytecodeBuilder, BytecodeFunctionId, BytecodeFunctionKind, CompiledScriptUnit,
    DeoptFrameValue, DeoptValueSource, FeedbackSiteKind, FeedbackSiteMetadata, Opcode,
    SafepointKind,
};
use lyng_js_common::{AtomTable, SourceId, WellKnownAtom};
use lyng_js_compiler::{compile_module, compile_script};
use lyng_js_env::{ExecutionContext, ExecutionContextKind, ModuleStatus, Runtime};
use lyng_js_host::{ModuleKey, NoopHostHooks};
use lyng_js_parser::{parse_module, parse_script};
use lyng_js_sema::{analyze_module, analyze_script};
use lyng_js_types::{CodeRef, EnvironmentRef, RealmRef, Value};
use lyng_js_vm::{seed_registers, FrameRecord, RegisterWindow, Vm};

#[test]
fn runtime_context_and_frame_records_seed_register_windows() {
    let context = ExecutionContext::bytecode(
        RealmRef::from_raw(1).unwrap(),
        CodeRef::from_raw(2).unwrap(),
        EnvironmentRef::from_raw(3).unwrap(),
        EnvironmentRef::from_raw(3).unwrap(),
    );
    let frame = FrameRecord::new(
        CodeRef::from_raw(2).unwrap(),
        4,
        RegisterWindow::new(8, 2),
        Some(1),
        RealmRef::from_raw(1).unwrap(),
        EnvironmentRef::from_raw(3).unwrap(),
        EnvironmentRef::from_raw(3).unwrap(),
        ExecutionContextKind::Function,
    );
    let registers = seed_registers(frame.registers());

    assert_eq!(context.kind(), ExecutionContextKind::Function);
    assert_eq!(frame.instruction_offset(), 4);
    assert_eq!(registers.len(), 2);
}

#[test]
fn vm_installs_and_executes_hand_authored_bytecode() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent
        .default_realm()
        .expect("runtime should expose a default realm");

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(1).expect("non-zero bytecode id"),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(2)
        .expect("test bytecode registers should allocate");
    let constant = builder
        .add_constant(lyng_js_bytecode::ConstantValue::Smi(12))
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
        .add_feedback_site(0, FeedbackSiteKind::Arithmetic, FeedbackSiteMetadata::None)
        .expect("test bytecode feedback site should build");
    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(19), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(installed.entry(), BytecodeFunctionId::from_raw(1).unwrap());
    assert_eq!(result, Value::from_smi(12));
    assert!(vm.frames().is_empty());
    assert!(vm.register_stack().is_empty());
    assert!(agent.current_execution_context().is_none());
}

#[test]
fn public_compile_and_evaluate_script_entrypoints_execute_end_to_end() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        SourceId::new(23),
        r"
        let base = 40;
        base + 2;
        ",
    );
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors: {:?}",
        parsed.diagnostics.as_slice()
    );
    let sema = analyze_script(&parsed, &atoms);
    assert!(
        !sema.diagnostics.has_errors(),
        "unexpected sema errors: {:?}",
        sema.diagnostics.as_slice()
    );

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let text = disassemble(unit.function(unit.entry()).unwrap());

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(unit.source(), SourceId::new(23));
    assert_eq!(result, Value::from_smi(42));
    assert!(text.contains("LoadConst") || text.contains("LoadSmi"));
    assert!(text.contains("Return"));
}

#[test]
fn public_compile_module_evaluates_default_export() {
    let mut atoms = AtomTable::new();
    let parsed = parse_module(&mut atoms, SourceId::new(29), "export default 42;");
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors: {:?}",
        parsed.diagnostics.as_slice()
    );
    let sema = analyze_module(&parsed, &atoms);
    assert!(
        !sema.diagnostics.has_errors(),
        "unexpected sema errors: {:?}",
        sema.diagnostics.as_slice()
    );

    let unit = compile_module(&parsed, &sema, &mut atoms).unwrap();
    let key = ModuleKey::new("/tmp/pipeline-default.mjs");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm
        .evaluate_module(agent, realm, &key, "/tmp/pipeline-default.mjs", &unit)
        .unwrap();

    let record = agent
        .module_record(&key)
        .expect("evaluated module should stay cached on the agent");
    let module_env = record
        .environment()
        .expect("module evaluation should materialize a module environment");
    let default_slot = unit
        .local_exports()
        .iter()
        .find(|entry| entry.export_name() == WellKnownAtom::default.id())
        .expect("module should expose a default export")
        .local_slot();

    assert_eq!(result, Value::undefined());
    assert_eq!(record.status(), ModuleStatus::Evaluated);
    assert_eq!(
        agent.environment_slot(module_env, default_slot),
        Some(Value::from_smi(42))
    );
}

#[test]
fn public_vm_metadata_accessors_resolve_installed_template_records() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        SourceId::new(31),
        r"
        let make = function(value) { return value; };
        let count = 0;
        while (count < 1) {
            count = count + 1;
        }
        try {
            make({ value: count });
        } catch (err) {
            err;
        }
        ",
    );
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors: {:?}",
        parsed.diagnostics.as_slice()
    );
    let sema = analyze_script(&parsed, &atoms);
    assert!(
        !sema.diagnostics.has_errors(),
        "unexpected sema errors: {:?}",
        sema.diagnostics.as_slice()
    );

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let function = unit
        .function(unit.entry())
        .expect("entry script should exist");
    let exception = function
        .safepoints()
        .iter()
        .find(|descriptor| descriptor.kind() == SafepointKind::ExceptionEdge)
        .copied()
        .expect("script should expose an exception-edge safepoint");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    assert!(vm
        .source_map_entry(installed.code(), exception.instruction_offset())
        .is_some());
    assert_eq!(
        vm.safepoint_by_id(installed.code(), exception.id())
            .expect("installed code should expose the exception safepoint")
            .kind(),
        SafepointKind::ExceptionEdge
    );
    assert!(vm
        .deopt_snapshot(installed.code(), exception.id())
        .expect("installed code should expose the exception deopt snapshot")
        .values()
        .contains(&DeoptValueSource::FrameValue(
            DeoptFrameValue::ExceptionValue,
        )));
}

#[test]
fn public_vm_feedback_footprint_reports_allocated_vector_bytes() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        SourceId::new(37),
        r"
        var total = 0;
        var i = 0;
        while (i < 4) {
            total = total + i;
            i = i + 1;
        }
        total;
        ",
    );
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors: {:?}",
        parsed.diagnostics.as_slice()
    );
    let sema = analyze_script(&parsed, &atoms);
    assert!(
        !sema.diagnostics.has_errors(),
        "unexpected sema errors: {:?}",
        sema.diagnostics.as_slice()
    );

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    let before = vm
        .feedback_vector_footprint(installed.code())
        .expect("installed code should expose feedback footprint details");
    assert!(before.slot_count() > 0);
    assert_eq!(before.live_site_count(), before.slot_count());
    assert!(!before.allocated());
    assert_eq!(before.allocated_bytes(), 0);

    vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    let after = vm
        .feedback_vector_footprint(installed.code())
        .expect("installed code should expose feedback footprint after execution");
    assert!(after.allocated());
    assert!(after.allocated_bytes() > 0);
    assert!(after.warmup_counter() >= 2);
}

//! Compile-smoke coverage for the execution-semantics crate DAG.

use lyng_js_ast::FunctionId;
use lyng_js_bytecode::{
    disassemble, ArgumentsMode, BytecodeBuilder, BytecodeFunction, BytecodeFunctionId,
    BytecodeFunctionKind, BytecodeMarker, CompiledScriptUnit, DeoptFrameValue, DeoptValueSource,
    FeedbackSiteDescriptor, FeedbackSiteKind, FeedbackSiteMetadata, Instruction, Opcode,
    SafepointKind,
};
use lyng_js_common::{AtomId, AtomTable, SourceId};
use lyng_js_compiler::{
    compile_module, compile_script, installable_module_unit, installable_script_unit,
    ActivationMetadata, CompilerMarker, LoweredFunctionPlan, LoweringContext,
};
use lyng_js_env::{ExecutableId, ExecutionContext, ExecutionContextKind, Runtime};
use lyng_js_host::NoopHostHooks;
use lyng_js_parser::{parse_module, parse_script};
use lyng_js_sema::{analyze_module, analyze_script, FunctionSemaId, ScopeId};
use lyng_js_types::{CodeRef, EnvironmentRef, FeedbackSlotId, RealmRef, Value};
use lyng_js_vm::{
    seed_registers, FrameFlags, FrameRecord, InstalledCode, RegisterWindow, Vm, VmMarker,
};
use std::mem::size_of;
use std::num::NonZeroU32;

#[test]
fn phase4_scaffold_crates_form_expected_dependency_chain() {
    let entry = BytecodeFunctionId::from_raw(1).expect("non-zero bytecode id");
    let feedback_slot = FeedbackSlotId::from_raw(2).expect("non-zero feedback slot");
    let marker = BytecodeMarker::new(SourceId::new(5), entry, feedback_slot);
    let compiler = CompilerMarker::new(marker, ScopeId::new(3), FunctionSemaId::new(7));
    let runtime = Runtime::new(NoopHostHooks);
    let realm = runtime
        .root_agent()
        .default_realm()
        .expect("runtime should expose a default realm");
    let context = ExecutionContext::bytecode(
        realm.id(),
        CodeRef::from_raw(11).unwrap(),
        realm.global_env(),
        realm.global_env(),
    );
    let vm = VmMarker::new(
        marker,
        context,
        FrameRecord::new(
            CodeRef::from_raw(11).unwrap(),
            0,
            RegisterWindow::new(0, 4),
            Some(1),
            realm.id(),
            realm.global_env(),
            realm.global_env(),
            ExecutionContextKind::Function,
        )
        .with_flags(FrameFlags::entry()),
    );

    assert_eq!(compiler.bytecode(), marker);
    assert_eq!(compiler.scope_root(), ScopeId::new(3));
    assert_eq!(compiler.function(), FunctionSemaId::new(7));
    assert_eq!(vm.bytecode(), marker);
    assert_eq!(
        vm.context().executable(),
        ExecutableId::Bytecode(CodeRef::from_raw(11).unwrap())
    );
    assert_eq!(vm.frame().registers(), RegisterWindow::new(0, 4));
    assert_eq!(size_of::<BytecodeFunctionId>(), size_of::<u32>());
    assert_eq!(size_of::<Option<BytecodeFunctionId>>(), size_of::<u32>());
}

#[test]
fn phase4_scaffold_types_support_compile_smoke_instantiation() {
    let source = SourceId::new(9);
    let entry = BytecodeFunctionId::new(NonZeroU32::new(1).unwrap());
    let function =
        BytecodeFunction::new(entry, Some(AtomId::from_raw(17)), ArgumentsMode::Unmapped)
            .with_register_counts(3, 1)
            .with_instructions(vec![
                Instruction::Abc {
                    opcode: Opcode::Move,
                    a: 0,
                    b: 1,
                    c: 0,
                },
                Instruction::Ax {
                    opcode: Opcode::Return,
                    ax: 0,
                },
            ])
            .with_feedback_sites(vec![FeedbackSiteDescriptor::new(
                FeedbackSlotId::new(NonZeroU32::new(4).unwrap()),
                0,
                FeedbackSiteKind::Arithmetic,
            )]);
    let unit = CompiledScriptUnit::new(source, entry, vec![function.clone()]);
    let (unit_source, installed_unit) = installable_script_unit(source, unit.clone());
    let activation = ActivationMetadata::new(false, ArgumentsMode::Unmapped, true, None, false);
    let plan = LoweredFunctionPlan::new(
        FunctionId::new(0),
        FunctionSemaId::new(0),
        ScopeId::new(0),
        activation,
        entry,
    );
    let lowering = LoweringContext::new(FunctionSemaId::new(0));
    let installed = InstalledCode::new(CodeRef::from_raw(7).unwrap(), entry);
    let text = disassemble(&function);

    assert_eq!(unit_source, source);
    assert_eq!(installed_unit, unit);
    assert_eq!(plan.ast_function(), FunctionId::new(0));
    assert_eq!(plan.sema_function(), FunctionSemaId::new(0));
    assert_eq!(plan.bytecode(), entry);
    assert_eq!(activation, plan.activation());
    assert_eq!(lowering.function(), FunctionSemaId::new(0));
    assert_eq!(
        installed.executable(),
        ExecutableId::Bytecode(CodeRef::from_raw(7).unwrap())
    );
    assert!(text.contains("Move"));
    assert!(text.contains("Return"));
}

#[test]
fn phase4_vm_scaffold_uses_real_execution_context_types() {
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
fn phase4_vm_installs_and_executes_hand_authored_bytecode() {
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
fn phase4_public_compile_and_evaluate_script_entrypoints_execute_end_to_end() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        SourceId::new(23),
        r#"
        let base = 40;
        base + 2;
        "#,
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
fn phase4_public_compile_module_exposes_real_module_artifact() {
    let mut atoms = AtomTable::new();
    let parsed = parse_module(&mut atoms, SourceId::new(29), "export const value = 1;");
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
    let (unit_source, installed_unit) = installable_module_unit(SourceId::new(29), unit.clone());

    assert_eq!(unit_source, SourceId::new(29));
    assert_eq!(installed_unit, unit);
    assert_eq!(unit.source(), SourceId::new(29));
    assert!(unit.function(unit.entry()).is_some());
    assert_eq!(unit.local_exports().len(), 1);
}

#[test]
fn phase4_public_vm_metadata_accessors_resolve_installed_template_records() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        SourceId::new(31),
        r#"
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
        "#,
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
fn phase4_public_vm_feedback_footprint_reports_allocated_vector_bytes() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        SourceId::new(37),
        r#"
        var total = 0;
        var i = 0;
        while (i < 4) {
            total = total + i;
            i = i + 1;
        }
        total;
        "#,
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

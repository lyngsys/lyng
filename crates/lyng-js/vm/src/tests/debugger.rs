use super::support::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
struct PauseRecord {
    reason: VmDebugPauseReason,
    kind: VmDebugSafepointKind,
    code: CodeRef,
    instruction_offset: u32,
    frame_count: usize,
    register_zero: Option<Value>,
    environment_slot_zero: Option<Value>,
}

#[derive(Clone)]
struct RecordingDebugHook {
    pauses: Rc<RefCell<Vec<PauseRecord>>>,
    commands: Rc<RefCell<VecDeque<VmDebugCommand>>>,
}

impl VmDebugHook for RecordingDebugHook {
    fn on_pause(&mut self, context: VmDebugPauseContext<'_>) -> VmDebugCommand {
        let safepoint = context.safepoint();
        self.pauses.borrow_mut().push(PauseRecord {
            reason: context.reason(),
            kind: safepoint.kind(),
            code: safepoint.code(),
            instruction_offset: safepoint.instruction_offset(),
            frame_count: context.frames().len(),
            register_zero: context.read_register(0, 0),
            environment_slot_zero: context.read_env_slot(0, 0, 0),
        });
        self.commands
            .borrow_mut()
            .pop_front()
            .unwrap_or(VmDebugCommand::Resume)
    }
}

#[test]
fn debugger_pauses_at_requested_loop_header_and_reads_frame_state() {
    let (unit, loop_offset) = inspector_fixture_unit();
    let pauses = Rc::new(RefCell::new(Vec::new()));

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    vm.set_debug_hook(RecordingDebugHook {
        pauses: Rc::clone(&pauses),
        commands: Rc::new(RefCell::new(VecDeque::new())),
    });
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    vm.request_debug_pause_at(installed.code(), loop_offset);

    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(41));
    assert_eq!(
        pauses.borrow().as_slice(),
        &[PauseRecord {
            reason: VmDebugPauseReason::Requested,
            kind: VmDebugSafepointKind::LoopHeader,
            code: installed.code(),
            instruction_offset: loop_offset,
            frame_count: 1,
            register_zero: Some(Value::from_smi(41)),
            environment_slot_zero: Some(Value::from_smi(41)),
        }]
    );
}

#[test]
fn debugger_step_commands_pause_at_frame_depth_boundaries() {
    assert_step_command(
        VmDebugCommand::StepIn,
        VmDebugStepMode::In,
        ExpectedStepPause::InnerEntry,
    );
    assert_step_command(
        VmDebugCommand::StepOver,
        VmDebugStepMode::Over,
        ExpectedStepPause::OuterLoop,
    );
    assert_step_command(
        VmDebugCommand::StepOut,
        VmDebugStepMode::Out,
        ExpectedStepPause::ScriptLoop,
    );
}

fn assert_step_command(
    command: VmDebugCommand,
    step_mode: VmDebugStepMode,
    expected: ExpectedStepPause,
) {
    let (unit, offsets) = stepping_fixture_unit();
    let pauses = Rc::new(RefCell::new(Vec::new()));
    let commands = Rc::new(RefCell::new(VecDeque::from([command])));

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    vm.set_debug_hook(RecordingDebugHook {
        pauses: Rc::clone(&pauses),
        commands,
    });
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let script_code = installed.code();
    let outer_code = vm
        .installed_child_code(script_code, 0)
        .expect("script should install outer child code");
    let inner_code = vm
        .installed_child_code(outer_code, 0)
        .expect("outer should install inner child code");
    vm.request_debug_pause_at(outer_code, offsets.outer_entry);

    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(7));
    let pauses = pauses.borrow();
    assert_eq!(pauses.len(), 2);
    assert_eq!(
        pauses[0],
        PauseRecord {
            reason: VmDebugPauseReason::Requested,
            kind: VmDebugSafepointKind::LoopHeader,
            code: outer_code,
            instruction_offset: offsets.outer_entry,
            frame_count: 2,
            register_zero: Some(Value::undefined()),
            environment_slot_zero: None,
        }
    );
    expected.assert_matches(
        &pauses[1],
        script_code,
        outer_code,
        inner_code,
        offsets,
        step_mode,
    );
}

fn inspector_fixture_unit() -> (CompiledScriptUnit, u32) {
    let env_operand = encode_env_operand(0, 0);
    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(15).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(2)
        .expect("test bytecode registers should allocate");
    builder.set_needs_environment(true);
    builder.set_environment_bindings(vec![BytecodeEnvironmentBinding::new(
        None,
        BytecodeEnvironmentSlotFlags::var_like(),
    )]);
    builder
        .emit_abx(Opcode::LoadSmi, 0, 41)
        .expect("test bytecode should build");
    builder
        .emit_abx(Opcode::StoreEnvSlot, 0, env_operand)
        .expect("test bytecode should build");
    let loop_offset = builder
        .emit_ax(Opcode::LoopHeader, 0)
        .expect("test bytecode should build");
    builder
        .emit_abx(Opcode::LoadEnvSlot, 1, env_operand)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 1)
        .expect("test bytecode should build");
    let function = builder.finish().expect("test bytecode should build");
    (
        CompiledScriptUnit::new(SourceId::new(15), function.id(), vec![function]),
        loop_offset,
    )
}

#[derive(Clone, Copy)]
struct StepFixtureOffsets {
    script_after_outer: u32,
    outer_entry: u32,
    outer_after_inner: u32,
}

#[derive(Clone, Copy)]
enum ExpectedStepPause {
    InnerEntry,
    OuterLoop,
    ScriptLoop,
}

impl ExpectedStepPause {
    fn assert_matches(
        self,
        record: &PauseRecord,
        script_code: CodeRef,
        outer_code: CodeRef,
        inner_code: CodeRef,
        offsets: StepFixtureOffsets,
        step_mode: VmDebugStepMode,
    ) {
        let (kind, code, instruction_offset, frame_count) = match self {
            Self::InnerEntry => (VmDebugSafepointKind::FunctionEntry, inner_code, 0, 3),
            Self::OuterLoop => (
                VmDebugSafepointKind::LoopHeader,
                outer_code,
                offsets.outer_after_inner,
                2,
            ),
            Self::ScriptLoop => (
                VmDebugSafepointKind::LoopHeader,
                script_code,
                offsets.script_after_outer,
                1,
            ),
        };
        assert_eq!(record.reason, VmDebugPauseReason::Step(step_mode));
        assert_eq!(record.kind, kind);
        assert_eq!(record.code, code);
        assert_eq!(record.instruction_offset, instruction_offset);
        assert_eq!(record.frame_count, frame_count);
        assert_eq!(record.environment_slot_zero, None);
    }
}

fn stepping_fixture_unit() -> (CompiledScriptUnit, StepFixtureOffsets) {
    let inner_id = BytecodeFunctionId::from_raw(23).unwrap();
    let outer_id = BytecodeFunctionId::from_raw(22).unwrap();
    let script_id = BytecodeFunctionId::from_raw(21).unwrap();

    let mut inner = BytecodeBuilder::new(inner_id, BytecodeFunctionKind::Function);
    inner
        .alloc_registers(1)
        .expect("test bytecode registers should allocate");
    inner
        .emit_ax(Opcode::LoopHeader, 0)
        .expect("test bytecode should build");
    inner
        .emit_abx(Opcode::LoadSmi, 0, 7)
        .expect("test bytecode should build");
    inner
        .emit_ax(Opcode::Return, 0)
        .expect("test bytecode should build");
    let inner = inner.finish().expect("test bytecode should build");

    let mut outer = BytecodeBuilder::new(outer_id, BytecodeFunctionKind::Function);
    outer
        .alloc_registers(3)
        .expect("test bytecode registers should allocate");
    let inner_child = outer
        .add_child_function(inner_id)
        .expect("test bytecode should build");
    outer
        .emit_ax(Opcode::Nop, 0)
        .expect("test bytecode should build");
    let outer_entry = outer
        .emit_ax(Opcode::LoopHeader, 0)
        .expect("test bytecode should build");
    outer
        .emit_abx(Opcode::CreateClosure, 0, inner_child)
        .expect("test bytecode should build");
    outer
        .emit_abx(Opcode::LoadUndefined, 1, 0)
        .expect("test bytecode should build");
    outer
        .emit_abc(Opcode::Call0, 2, 0, 1)
        .expect("test bytecode should build");
    let outer_after_inner = outer
        .emit_ax(Opcode::LoopHeader, 0)
        .expect("test bytecode should build");
    outer
        .emit_ax(Opcode::Return, 2)
        .expect("test bytecode should build");
    let outer = outer.finish().expect("test bytecode should build");

    let mut script = BytecodeBuilder::new(script_id, BytecodeFunctionKind::Script);
    script
        .alloc_registers(3)
        .expect("test bytecode registers should allocate");
    let outer_child = script
        .add_child_function(outer_id)
        .expect("test bytecode should build");
    script
        .emit_abx(Opcode::CreateClosure, 0, outer_child)
        .expect("test bytecode should build");
    script
        .emit_abx(Opcode::LoadUndefined, 1, 0)
        .expect("test bytecode should build");
    script
        .emit_abc(Opcode::Call0, 2, 0, 1)
        .expect("test bytecode should build");
    let script_after_outer = script
        .emit_ax(Opcode::LoopHeader, 0)
        .expect("test bytecode should build");
    script
        .emit_ax(Opcode::Return, 2)
        .expect("test bytecode should build");
    let script = script.finish().expect("test bytecode should build");

    (
        CompiledScriptUnit::new(SourceId::new(16), script.id(), vec![script, outer, inner]),
        StepFixtureOffsets {
            script_after_outer,
            outer_entry,
            outer_after_inner,
        },
    )
}

const fn encode_env_operand(depth: u8, slot: u32) -> u32 {
    ((depth as u32) << 24) | (slot & 0x00ff_ffff)
}

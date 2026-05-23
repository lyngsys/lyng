use lyng_js_env::Agent;
use lyng_js_types::{CodeRef, EnvironmentRef, Value};

use crate::{FrameRecord, Vm};

pub trait VmDebugHook {
    fn on_pause(&mut self, context: VmDebugPauseContext<'_>) -> VmDebugCommand;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VmDebugCommand {
    Resume,
    StepIn,
    StepOver,
    StepOut,
}

impl VmDebugCommand {
    const fn step_mode(self) -> Option<VmDebugStepMode> {
        match self {
            Self::Resume => None,
            Self::StepIn => Some(VmDebugStepMode::In),
            Self::StepOver => Some(VmDebugStepMode::Over),
            Self::StepOut => Some(VmDebugStepMode::Out),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VmDebugStepMode {
    In,
    Over,
    Out,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VmDebugPauseReason {
    Requested,
    Step(VmDebugStepMode),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VmDebugSafepointKind {
    FunctionEntry,
    LoopHeader,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VmDebugSafepoint {
    kind: VmDebugSafepointKind,
    code: CodeRef,
    instruction_offset: u32,
    frame_depth: usize,
}

impl VmDebugSafepoint {
    pub(super) const fn new(
        kind: VmDebugSafepointKind,
        frame: &FrameRecord,
        frame_depth: usize,
    ) -> Self {
        Self {
            kind,
            code: frame.code(),
            instruction_offset: frame.instruction_offset(),
            frame_depth,
        }
    }

    #[inline]
    pub const fn kind(self) -> VmDebugSafepointKind {
        self.kind
    }

    #[inline]
    pub const fn code(self) -> CodeRef {
        self.code
    }

    #[inline]
    pub const fn instruction_offset(self) -> u32 {
        self.instruction_offset
    }

    #[inline]
    pub const fn frame_depth(self) -> usize {
        self.frame_depth
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VmDebugFrame {
    index: usize,
    code: CodeRef,
    instruction_offset: u32,
    register_count: u16,
    lexical_env: EnvironmentRef,
    variable_env: EnvironmentRef,
}

impl VmDebugFrame {
    #[inline]
    pub const fn index(self) -> usize {
        self.index
    }

    #[inline]
    pub const fn code(self) -> CodeRef {
        self.code
    }

    #[inline]
    pub const fn instruction_offset(self) -> u32 {
        self.instruction_offset
    }

    #[inline]
    pub const fn register_count(self) -> u16 {
        self.register_count
    }

    #[inline]
    pub const fn lexical_env(self) -> EnvironmentRef {
        self.lexical_env
    }

    #[inline]
    pub const fn variable_env(self) -> EnvironmentRef {
        self.variable_env
    }
}

pub struct VmDebugPauseContext<'a> {
    vm: &'a Vm,
    agent: &'a Agent,
    safepoint: VmDebugSafepoint,
    reason: VmDebugPauseReason,
}

impl<'a> VmDebugPauseContext<'a> {
    pub(super) const fn new(
        vm: &'a Vm,
        agent: &'a Agent,
        safepoint: VmDebugSafepoint,
        reason: VmDebugPauseReason,
    ) -> Self {
        Self {
            vm,
            agent,
            safepoint,
            reason,
        }
    }

    #[inline]
    pub const fn safepoint(&self) -> VmDebugSafepoint {
        self.safepoint
    }

    #[inline]
    pub const fn reason(&self) -> VmDebugPauseReason {
        self.reason
    }

    pub fn frames(&self) -> Vec<VmDebugFrame> {
        self.vm
            .frames()
            .iter()
            .rev()
            .enumerate()
            .map(|(index, frame)| VmDebugFrame {
                index,
                code: frame.code(),
                instruction_offset: frame.instruction_offset(),
                register_count: frame.registers().len(),
                lexical_env: frame.lexical_env(),
                variable_env: frame.variable_env(),
            })
            .collect()
    }

    pub fn read_register(&self, frame_index: usize, register: u16) -> Option<Value> {
        let frame = self.frame_at(frame_index)?;
        (register < frame.registers().len())
            .then(|| self.vm.read_register(frame.registers(), register))
    }

    pub fn read_env_slot(&self, frame_index: usize, depth: u8, slot: u32) -> Option<Value> {
        let frame = self.frame_at(frame_index)?;
        let environment = self
            .vm
            .environment_for_slot_access(self.agent, frame.lexical_env(), depth, slot)
            .ok()?;
        self.agent.environment_slot(environment, slot)
    }

    fn frame_at(&self, frame_index: usize) -> Option<FrameRecord> {
        self.vm.frames().iter().rev().nth(frame_index).copied()
    }
}

#[derive(Default)]
pub(super) struct VmDebugState {
    pause_request: Option<VmDebugPauseRequest>,
    step: Option<VmDebugStep>,
}

impl VmDebugState {
    pub(super) const fn should_poll(&self) -> bool {
        self.pause_request.is_some() || self.step.is_some()
    }

    pub(super) const fn request_pause(&mut self, request: VmDebugPauseRequest) {
        self.pause_request = Some(request);
    }

    pub(super) const fn clear_pause_request(&mut self) {
        self.pause_request = None;
    }

    pub(super) const fn clear(&mut self) {
        self.pause_request = None;
        self.step = None;
    }

    pub(super) fn consume_pause(
        &mut self,
        safepoint: VmDebugSafepoint,
    ) -> Option<VmDebugPauseReason> {
        if self
            .pause_request
            .is_some_and(|request| request.matches(safepoint))
        {
            self.pause_request = None;
            return Some(VmDebugPauseReason::Requested);
        }

        let step = self.step?;
        if step.should_pause(safepoint.frame_depth()) {
            self.step = None;
            return Some(VmDebugPauseReason::Step(step.mode));
        }
        None
    }

    pub(super) const fn apply_command(
        &mut self,
        command: VmDebugCommand,
        origin_frame_depth: usize,
    ) {
        self.step = match command.step_mode() {
            Some(mode) => Some(VmDebugStep {
                mode,
                origin_frame_depth,
            }),
            None => None,
        };
    }
}

#[derive(Clone, Copy)]
pub(super) struct VmDebugPauseRequest {
    code: Option<CodeRef>,
    instruction_offset: Option<u32>,
}

impl VmDebugPauseRequest {
    pub(super) const fn any() -> Self {
        Self {
            code: None,
            instruction_offset: None,
        }
    }

    pub(super) const fn at(code: CodeRef, instruction_offset: u32) -> Self {
        Self {
            code: Some(code),
            instruction_offset: Some(instruction_offset),
        }
    }

    fn matches(self, safepoint: VmDebugSafepoint) -> bool {
        let code_matches = self.code.is_none_or(|code| code == safepoint.code());
        let offset_matches = self
            .instruction_offset
            .is_none_or(|instruction_offset| instruction_offset == safepoint.instruction_offset());
        code_matches && offset_matches
    }
}

#[derive(Clone, Copy)]
struct VmDebugStep {
    mode: VmDebugStepMode,
    origin_frame_depth: usize,
}

impl VmDebugStep {
    const fn should_pause(self, frame_depth: usize) -> bool {
        match self.mode {
            VmDebugStepMode::In => true,
            VmDebugStepMode::Over => frame_depth <= self.origin_frame_depth,
            VmDebugStepMode::Out => frame_depth < self.origin_frame_depth,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{VmDebugStep, VmDebugStepMode};

    #[test]
    fn step_modes_are_defined_by_observed_frame_depth() {
        let step_in = VmDebugStep {
            mode: VmDebugStepMode::In,
            origin_frame_depth: 2,
        };
        assert!(step_in.should_pause(3));
        assert!(step_in.should_pause(2));
        assert!(step_in.should_pause(1));

        let step_over = VmDebugStep {
            mode: VmDebugStepMode::Over,
            origin_frame_depth: 2,
        };
        assert!(!step_over.should_pause(3));
        assert!(step_over.should_pause(2));
        assert!(step_over.should_pause(1));

        let step_out = VmDebugStep {
            mode: VmDebugStepMode::Out,
            origin_frame_depth: 2,
        };
        assert!(!step_out.should_pause(3));
        assert!(!step_out.should_pause(2));
        assert!(step_out.should_pause(1));
    }
}

use lyng_env::Agent;
use lyng_types::{CodeRef, EnvironmentRef, Value};

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

    /// The referrer reported by the parallel `Vm` side-stack at the current live
    /// pause. The SP-0a referrer migration made this side-stack the single
    /// source of truth for the establishment referrer.
    #[cfg(test)]
    pub(crate) fn current_referrer(&self) -> Option<lyng_common::AtomId> {
        self.vm.current_referrer()
    }

    /// The Agent's ambient `running_context` snapshot alongside the values the
    /// VM would recompute from the active frame. SP-0a requires the scalar to
    /// track the frame at every live point; this exposes both for a test.
    ///
    /// Test-only: exposes parity between the `running_context` scalar and the
    /// frame-derived values for regression coverage.
    #[cfg(test)]
    pub(crate) fn running_context_parity(&self) -> RunningContextParity {
        RunningContextParity {
            scalar: self.agent.running_context(),
            frame_realm: self.vm.frame().map(|frame| frame.realm()),
            frame_referrer: self.vm.current_referrer(),
        }
    }
}

/// The Agent `running_context` scalar paired with the values the VM recomputes
/// from the active frame, captured at a single live pause for a parity test.
#[cfg(test)]
pub(crate) struct RunningContextParity {
    pub(crate) scalar: Option<lyng_env::RunningContext>,
    pub(crate) frame_realm: Option<lyng_types::RealmRef>,
    pub(crate) frame_referrer: Option<lyng_common::AtomId>,
}

/// Caller-owned bundle of `(hook, state)` that drives the VM debug poll.
///
/// Constructed externally, then passed by mutable reference to
/// [`EvaluateScript::with_debugger`] / [`EvaluateInstalled::with_debugger`]
/// for a single `.run()`. The builder swaps it into the VM's internal slot
/// at run entry and swaps it back at run exit — so pause-control mutations
/// the caller makes between runs persist on the externally-owned struct.
///
/// Mirrors [`OpcodeCounters`] for the debug side of the VM extension surface.
#[derive(Default)]
pub struct VmDebugger {
    hook: Option<Box<dyn VmDebugHook>>,
    state: VmDebugState,
}

impl VmDebugger {
    #[must_use]
    pub fn new(hook: impl VmDebugHook + 'static) -> Self {
        Self {
            hook: Some(Box::new(hook)),
            state: VmDebugState::default(),
        }
    }

    /// Drop the installed hook and clear all pause/step state.
    pub fn clear(&mut self) {
        self.hook = None;
        self.state.clear();
    }

    /// Request a pause at the next safepoint of any kind.
    pub const fn request_pause(&mut self) {
        self.state.request_pause(VmDebugPauseRequest::any());
    }

    /// Request a pause at the next safepoint matching `code` and `instruction_offset`.
    pub const fn request_pause_at(&mut self, code: CodeRef, instruction_offset: u32) {
        self.state
            .request_pause(VmDebugPauseRequest::at(code, instruction_offset));
    }

    /// Drop any pending pause request without affecting the active step.
    pub const fn clear_pause_request(&mut self) {
        self.state.clear_pause_request();
    }

    /// True when the swapped-in debugger should drive the asm fast-path
    /// safepoint byte. Empty debugger (no hook, no requests, no step)
    /// returns false — fast path stays at 2 instructions.
    #[inline]
    pub(super) const fn poll_enabled(&self) -> bool {
        self.hook.is_some() && self.state.should_poll()
    }

    #[inline]
    pub(super) fn consume_pause(
        &mut self,
        safepoint: VmDebugSafepoint,
    ) -> Option<VmDebugPauseReason> {
        self.state.consume_pause(safepoint)
    }

    #[inline]
    pub(super) const fn apply_command(
        &mut self,
        command: VmDebugCommand,
        origin_frame_depth: usize,
    ) {
        self.state.apply_command(command, origin_frame_depth);
    }

    /// Temporarily detach the hook so the VM can invoke it while holding
    /// `&mut Vm` (the hook callback receives a `VmDebugPauseContext` that
    /// borrows the VM). Caller MUST follow with [`Self::restore_hook`].
    #[inline]
    pub(super) fn take_hook(&mut self) -> Option<Box<dyn VmDebugHook>> {
        self.hook.take()
    }

    #[inline]
    pub(super) fn restore_hook(&mut self, hook: Box<dyn VmDebugHook>) {
        self.hook = Some(hook);
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

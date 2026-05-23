use lyng_js_env::ExecutionContextKind;
use lyng_js_types::{CodeRef, EnvironmentRef, ObjectRef, RealmRef, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct FrameFlags(u8);

impl FrameFlags {
    const ENTRY: u8 = 1 << 0;
    const SUSPENDABLE: u8 = 1 << 1;
    const CONSTRUCT: u8 = 1 << 2;
    const DERIVED_CONSTRUCT: u8 = 1 << 3;

    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn entry() -> Self {
        Self(Self::ENTRY)
    }

    #[inline]
    pub const fn suspendable() -> Self {
        Self(Self::SUSPENDABLE)
    }

    #[inline]
    pub const fn construct() -> Self {
        Self(Self::CONSTRUCT)
    }

    #[inline]
    pub const fn derived_construct() -> Self {
        Self(Self::DERIVED_CONSTRUCT)
    }

    #[inline]
    pub const fn contains(self, flags: Self) -> bool {
        self.0 & flags.0 == flags.0
    }

    #[inline]
    pub const fn from_raw(raw: u8) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn raw(self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn with_flag(mut self, flags: Self, enabled: bool) -> Self {
        if enabled {
            self.0 |= flags.0;
        } else {
            self.0 &= !flags.0;
        }
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum GeneratorResumeKind {
    #[default]
    Next = 0,
    Throw = 1,
    Return = 2,
}

impl GeneratorResumeKind {
    #[inline]
    pub const fn raw(self) -> u8 {
        self as u8
    }

    #[inline]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        match raw {
            0 => Some(Self::Next),
            1 => Some(Self::Throw),
            2 => Some(Self::Return),
            _ => None,
        }
    }
}

/// Shared register-stack window reserved for one active frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RegisterWindow {
    base: u32,
    len: u16,
}

impl RegisterWindow {
    #[inline]
    pub const fn new(base: u32, len: u16) -> Self {
        Self { base, len }
    }

    #[inline]
    pub const fn base(self) -> u32 {
        self.base
    }

    #[inline]
    pub const fn len(self) -> u16 {
        self.len
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }

    #[inline]
    pub const fn end(self) -> u32 {
        self.base + self.len as u32
    }
}

/// Stable per-activation half of [`FrameRecord`].
///
/// Hoisted once by the outer dispatch loop and reused across every opcode in the
/// activation. Fields here are immutable once the frame is installed — no opcode
/// helper writes to them mid-dispatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FrameMetadata {
    code: CodeRef,
    parameter_initializer_end_offset: u32,
    registers: RegisterWindow,
    return_register: Option<u16>,
    realm: RealmRef,
    variable_env: EnvironmentRef,
    new_target: Option<ObjectRef>,
    callee: Option<ObjectRef>,
    kind: ExecutionContextKind,
}

impl FrameMetadata {
    #[inline]
    pub const fn code(&self) -> CodeRef {
        self.code
    }

    #[inline]
    pub const fn parameter_initializer_end_offset(&self) -> u32 {
        self.parameter_initializer_end_offset
    }

    #[inline]
    pub const fn registers(&self) -> RegisterWindow {
        self.registers
    }

    #[inline]
    pub const fn return_register(&self) -> Option<u16> {
        self.return_register
    }

    #[inline]
    pub const fn realm(&self) -> RealmRef {
        self.realm
    }

    #[inline]
    pub const fn variable_env(&self) -> EnvironmentRef {
        self.variable_env
    }

    #[inline]
    pub const fn new_target(&self) -> Option<ObjectRef> {
        self.new_target
    }

    #[inline]
    pub const fn callee(&self) -> Option<ObjectRef> {
        self.callee
    }

    #[inline]
    pub const fn kind(&self) -> ExecutionContextKind {
        self.kind
    }
}

/// Mutable per-opcode half of [`FrameRecord`].
///
/// Hoisted into local dispatch state for the active loop and synced back to the
/// frame stack at observable boundaries. Fields here are written by local PC
/// advancement, environment changes, the exception handler, generator resume
/// helpers, super-ops (`set_this_value`/`set_construct_this` after `super()`
/// returns into a still-dispatching derived constructor frame), and tail-call
/// installation (`set_tail_caller` writes the brand-new same-depth activation's
/// tail-caller record without changing `code`, so the outer-loop `code`+`depth`
/// invariant alone cannot detect it).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FrameState {
    instruction_offset: u32,
    lexical_env: EnvironmentRef,
    this_value: Value,
    construct_this: Option<ObjectRef>,
    tail_caller: Option<ObjectRef>,
    tail_caller_strict: bool,
    handler_cursor: u16,
    flags: FrameFlags,
    resume_kind: GeneratorResumeKind,
    resume_value: Value,
    resume_active: bool,
}

impl FrameState {
    #[inline]
    pub const fn instruction_offset(&self) -> u32 {
        self.instruction_offset
    }

    #[inline]
    pub const fn lexical_env(&self) -> EnvironmentRef {
        self.lexical_env
    }

    #[inline]
    pub const fn this_value(&self) -> Value {
        self.this_value
    }

    #[inline]
    pub const fn construct_this(&self) -> Option<ObjectRef> {
        self.construct_this
    }

    #[inline]
    pub const fn tail_caller(&self) -> Option<ObjectRef> {
        self.tail_caller
    }

    #[inline]
    pub const fn tail_caller_strict(&self) -> bool {
        self.tail_caller_strict
    }

    #[inline]
    pub const fn handler_cursor(&self) -> u16 {
        self.handler_cursor
    }

    #[inline]
    pub const fn flags(&self) -> FrameFlags {
        self.flags
    }

    #[inline]
    pub const fn resume_active(&self) -> bool {
        self.resume_active
    }

    #[inline]
    pub const fn resume_kind(&self) -> GeneratorResumeKind {
        self.resume_kind
    }

    #[inline]
    pub const fn resume_value(&self) -> Value {
        self.resume_value
    }
}

/// Frame record for the active bytecode call-frame contract.
///
/// Composed of [`FrameMetadata`] (stable across the activation, hoisted once by the
/// outer dispatch loop) and [`FrameState`] (mutated by opcode handlers, read live
/// each iteration). The split keeps the per-opcode snapshot small while letting hot
/// stable fields (`code`, `realm`, `registers`, …) stay in registers across the
/// inner loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FrameRecord {
    metadata: FrameMetadata,
    state: FrameState,
}

impl FrameRecord {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        code: CodeRef,
        instruction_offset: u32,
        registers: RegisterWindow,
        return_register: Option<u16>,
        realm: RealmRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
        kind: ExecutionContextKind,
    ) -> Self {
        Self {
            metadata: FrameMetadata {
                code,
                parameter_initializer_end_offset: 0,
                registers,
                return_register,
                realm,
                variable_env,
                new_target: None,
                callee: None,
                kind,
            },
            state: FrameState {
                instruction_offset,
                lexical_env,
                this_value: Value::undefined(),
                construct_this: None,
                tail_caller: None,
                tail_caller_strict: false,
                handler_cursor: 0,
                flags: FrameFlags::empty(),
                resume_kind: GeneratorResumeKind::Next,
                resume_value: Value::undefined(),
                resume_active: false,
            },
        }
    }

    #[inline]
    pub const fn with_this_value(mut self, this_value: Value) -> Self {
        self.state.this_value = this_value;
        self
    }

    #[inline]
    pub const fn with_new_target(mut self, new_target: Option<ObjectRef>) -> Self {
        self.metadata.new_target = new_target;
        self
    }

    #[inline]
    pub const fn with_construct_this(mut self, construct_this: Option<ObjectRef>) -> Self {
        self.state.construct_this = construct_this;
        self
    }

    #[inline]
    pub const fn with_callee(mut self, callee: Option<ObjectRef>) -> Self {
        self.metadata.callee = callee;
        self
    }

    #[inline]
    pub const fn with_tail_caller(
        mut self,
        tail_caller: Option<ObjectRef>,
        tail_caller_strict: bool,
    ) -> Self {
        self.state.tail_caller = tail_caller;
        self.state.tail_caller_strict = tail_caller_strict;
        self
    }

    #[inline]
    pub(crate) const fn set_tail_caller(
        &mut self,
        tail_caller: Option<ObjectRef>,
        tail_caller_strict: bool,
    ) {
        self.state.tail_caller = tail_caller;
        self.state.tail_caller_strict = tail_caller_strict;
    }

    #[inline]
    pub const fn with_handler_cursor(mut self, handler_cursor: u16) -> Self {
        self.state.handler_cursor = handler_cursor;
        self
    }

    #[inline]
    pub(crate) const fn set_handler_cursor(&mut self, handler_cursor: u16) {
        self.state.handler_cursor = handler_cursor;
    }

    #[inline]
    pub const fn with_flags(mut self, flags: FrameFlags) -> Self {
        self.state.flags = flags;
        self
    }

    #[inline]
    pub const fn with_parameter_initializer_end_offset(
        mut self,
        parameter_initializer_end_offset: u32,
    ) -> Self {
        self.metadata.parameter_initializer_end_offset = parameter_initializer_end_offset;
        self
    }

    #[inline]
    pub const fn with_resume(
        mut self,
        resume_kind: GeneratorResumeKind,
        resume_value: Value,
    ) -> Self {
        self.state.resume_kind = resume_kind;
        self.state.resume_value = resume_value;
        self.state.resume_active = true;
        self
    }

    #[inline]
    pub const fn metadata(&self) -> FrameMetadata {
        self.metadata
    }

    #[inline]
    pub const fn state(&self) -> FrameState {
        self.state
    }

    #[inline]
    pub const fn code(&self) -> CodeRef {
        self.metadata.code
    }

    #[inline]
    pub const fn instruction_offset(&self) -> u32 {
        self.state.instruction_offset
    }

    #[inline]
    pub const fn parameter_initializer_end_offset(&self) -> u32 {
        self.metadata.parameter_initializer_end_offset
    }

    #[inline]
    pub(crate) const fn set_instruction_offset(&mut self, instruction_offset: u32) {
        self.state.instruction_offset = instruction_offset;
    }

    #[inline]
    pub const fn registers(&self) -> RegisterWindow {
        self.metadata.registers
    }

    #[inline]
    pub const fn return_register(&self) -> Option<u16> {
        self.metadata.return_register
    }

    #[inline]
    pub const fn realm(&self) -> RealmRef {
        self.metadata.realm
    }

    #[inline]
    pub const fn lexical_env(&self) -> EnvironmentRef {
        self.state.lexical_env
    }

    #[inline]
    pub(crate) const fn set_lexical_env(&mut self, lexical_env: EnvironmentRef) {
        self.state.lexical_env = lexical_env;
    }

    #[inline]
    pub const fn variable_env(&self) -> EnvironmentRef {
        self.metadata.variable_env
    }

    #[inline]
    pub const fn this_value(&self) -> Value {
        self.state.this_value
    }

    #[inline]
    pub(crate) const fn set_this_value(&mut self, this_value: Value) {
        self.state.this_value = this_value;
    }

    #[inline]
    pub const fn construct_this(&self) -> Option<ObjectRef> {
        self.state.construct_this
    }

    #[inline]
    pub(crate) const fn set_construct_this(&mut self, construct_this: Option<ObjectRef>) {
        self.state.construct_this = construct_this;
    }

    #[inline]
    pub const fn new_target(&self) -> Option<ObjectRef> {
        self.metadata.new_target
    }

    #[inline]
    pub const fn callee(&self) -> Option<ObjectRef> {
        self.metadata.callee
    }

    #[inline]
    pub const fn tail_caller(&self) -> Option<ObjectRef> {
        self.state.tail_caller
    }

    #[inline]
    pub const fn tail_caller_strict(&self) -> bool {
        self.state.tail_caller_strict
    }

    #[inline]
    pub const fn handler_cursor(&self) -> u16 {
        self.state.handler_cursor
    }

    #[inline]
    pub const fn flags(&self) -> FrameFlags {
        self.state.flags
    }

    #[inline]
    pub const fn kind(&self) -> ExecutionContextKind {
        self.metadata.kind
    }

    #[inline]
    pub const fn resume_active(&self) -> bool {
        self.state.resume_active
    }

    #[inline]
    pub const fn resume_kind(&self) -> GeneratorResumeKind {
        self.state.resume_kind
    }

    #[inline]
    pub const fn resume_value(&self) -> Value {
        self.state.resume_value
    }

    #[inline]
    pub(crate) const fn clear_resume(&mut self) {
        self.state.resume_active = false;
    }
}

/// Reserve a VM-facing evaluation shell without claiming a working interpreter yet.
#[inline]
pub fn seed_registers(window: RegisterWindow) -> Vec<Value> {
    vec![Value::undefined(); usize::from(window.len())]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU32;

    const fn id(raw: u32) -> NonZeroU32 {
        match NonZeroU32::new(raw) {
            Some(id) => id,
            None => panic!("test ids must be non-zero"),
        }
    }

    #[test]
    fn metadata_and_state_round_trip_through_frame_record() {
        let code = CodeRef::new(id(1));
        let lexical_env = EnvironmentRef::new(id(2));
        let variable_env = EnvironmentRef::new(id(3));
        let replacement_env = EnvironmentRef::new(id(4));
        let realm = RealmRef::new(id(5));
        let registers = RegisterWindow::new(11, 7);
        let mut frame = FrameRecord::new(
            code,
            17,
            registers,
            Some(3),
            realm,
            lexical_env,
            variable_env,
            ExecutionContextKind::Function,
        )
        .with_handler_cursor(2)
        .with_flags(FrameFlags::entry())
        .with_resume(GeneratorResumeKind::Throw, Value::from_smi(42));

        let metadata = frame.metadata();
        assert_eq!(metadata.code(), code);
        assert_eq!(metadata.registers(), registers);
        assert_eq!(metadata.return_register(), Some(3));
        assert_eq!(metadata.realm(), realm);
        assert_eq!(metadata.variable_env(), variable_env);
        assert_eq!(metadata.kind(), ExecutionContextKind::Function);

        frame.set_instruction_offset(41);
        frame.set_handler_cursor(9);
        frame.set_lexical_env(replacement_env);
        frame.clear_resume();

        assert_eq!(
            frame.metadata(),
            metadata,
            "metadata is untouched by state edits"
        );
        assert_eq!(frame.instruction_offset(), 41);
        assert_eq!(frame.handler_cursor(), 9);
        assert_eq!(frame.lexical_env(), replacement_env);
        assert_eq!(frame.flags(), FrameFlags::entry());
        assert!(!frame.resume_active());
    }

    #[test]
    fn this_value_and_construct_this_live_in_state() {
        let code = CodeRef::new(id(1));
        let lexical_env = EnvironmentRef::new(id(2));
        let variable_env = EnvironmentRef::new(id(3));
        let realm = RealmRef::new(id(5));
        let registers = RegisterWindow::new(0, 1);
        let mut frame = FrameRecord::new(
            code,
            0,
            registers,
            None,
            realm,
            lexical_env,
            variable_env,
            ExecutionContextKind::Function,
        );
        let metadata_before = frame.metadata();

        frame.set_this_value(Value::from_smi(7));
        frame.set_construct_this(None);

        assert_eq!(frame.this_value(), Value::from_smi(7));
        assert_eq!(
            frame.metadata(),
            metadata_before,
            "metadata must not change when this_value/construct_this are mutated mid-activation",
        );
    }
}

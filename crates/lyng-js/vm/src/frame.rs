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

/// Frame record for the active bytecode call-frame contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FrameRecord {
    code: CodeRef,
    instruction_offset: u32,
    parameter_initializer_end_offset: u32,
    registers: RegisterWindow,
    return_register: Option<u16>,
    realm: RealmRef,
    lexical_env: EnvironmentRef,
    variable_env: EnvironmentRef,
    this_value: Value,
    construct_this: Option<ObjectRef>,
    new_target: Option<ObjectRef>,
    callee: Option<ObjectRef>,
    tail_caller: Option<ObjectRef>,
    tail_caller_strict: bool,
    handler_cursor: u16,
    flags: FrameFlags,
    kind: ExecutionContextKind,
    resume_kind: GeneratorResumeKind,
    resume_value: Value,
    resume_active: bool,
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
            code,
            instruction_offset,
            parameter_initializer_end_offset: 0,
            registers,
            return_register,
            realm,
            lexical_env,
            variable_env,
            this_value: Value::undefined(),
            construct_this: None,
            new_target: None,
            callee: None,
            tail_caller: None,
            tail_caller_strict: false,
            handler_cursor: 0,
            flags: FrameFlags::empty(),
            kind,
            resume_kind: GeneratorResumeKind::Next,
            resume_value: Value::undefined(),
            resume_active: false,
        }
    }

    #[inline]
    pub const fn with_this_value(mut self, this_value: Value) -> Self {
        self.this_value = this_value;
        self
    }

    #[inline]
    pub const fn with_new_target(mut self, new_target: Option<ObjectRef>) -> Self {
        self.new_target = new_target;
        self
    }

    #[inline]
    pub const fn with_construct_this(mut self, construct_this: Option<ObjectRef>) -> Self {
        self.construct_this = construct_this;
        self
    }

    #[inline]
    pub const fn with_callee(mut self, callee: Option<ObjectRef>) -> Self {
        self.callee = callee;
        self
    }

    #[inline]
    pub const fn with_tail_caller(
        mut self,
        tail_caller: Option<ObjectRef>,
        tail_caller_strict: bool,
    ) -> Self {
        self.tail_caller = tail_caller;
        self.tail_caller_strict = tail_caller_strict;
        self
    }

    #[inline]
    pub(crate) const fn set_tail_caller(
        &mut self,
        tail_caller: Option<ObjectRef>,
        tail_caller_strict: bool,
    ) {
        self.tail_caller = tail_caller;
        self.tail_caller_strict = tail_caller_strict;
    }

    #[inline]
    pub const fn with_handler_cursor(mut self, handler_cursor: u16) -> Self {
        self.handler_cursor = handler_cursor;
        self
    }

    #[inline]
    pub(crate) const fn set_handler_cursor(&mut self, handler_cursor: u16) {
        self.handler_cursor = handler_cursor;
    }

    #[inline]
    pub const fn with_flags(mut self, flags: FrameFlags) -> Self {
        self.flags = flags;
        self
    }

    #[inline]
    pub const fn with_parameter_initializer_end_offset(
        mut self,
        parameter_initializer_end_offset: u32,
    ) -> Self {
        self.parameter_initializer_end_offset = parameter_initializer_end_offset;
        self
    }

    #[inline]
    pub const fn with_resume(
        mut self,
        resume_kind: GeneratorResumeKind,
        resume_value: Value,
    ) -> Self {
        self.resume_kind = resume_kind;
        self.resume_value = resume_value;
        self.resume_active = true;
        self
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
    pub const fn parameter_initializer_end_offset(self) -> u32 {
        self.parameter_initializer_end_offset
    }

    #[inline]
    pub(crate) const fn set_instruction_offset(&mut self, instruction_offset: u32) {
        self.instruction_offset = instruction_offset;
    }

    #[inline]
    pub const fn registers(self) -> RegisterWindow {
        self.registers
    }

    #[inline]
    pub const fn return_register(self) -> Option<u16> {
        self.return_register
    }

    #[inline]
    pub const fn realm(self) -> RealmRef {
        self.realm
    }

    #[inline]
    pub const fn lexical_env(self) -> EnvironmentRef {
        self.lexical_env
    }

    #[inline]
    pub(crate) const fn set_lexical_env(&mut self, lexical_env: EnvironmentRef) {
        self.lexical_env = lexical_env;
    }

    #[inline]
    pub const fn variable_env(self) -> EnvironmentRef {
        self.variable_env
    }

    #[inline]
    pub const fn this_value(self) -> Value {
        self.this_value
    }

    #[inline]
    pub(crate) const fn set_this_value(&mut self, this_value: Value) {
        self.this_value = this_value;
    }

    #[inline]
    pub const fn construct_this(self) -> Option<ObjectRef> {
        self.construct_this
    }

    #[inline]
    pub(crate) const fn set_construct_this(&mut self, construct_this: Option<ObjectRef>) {
        self.construct_this = construct_this;
    }

    #[inline]
    pub const fn new_target(self) -> Option<ObjectRef> {
        self.new_target
    }

    #[inline]
    pub const fn callee(self) -> Option<ObjectRef> {
        self.callee
    }

    #[inline]
    pub const fn tail_caller(self) -> Option<ObjectRef> {
        self.tail_caller
    }

    #[inline]
    pub const fn tail_caller_strict(self) -> bool {
        self.tail_caller_strict
    }

    #[inline]
    pub const fn handler_cursor(self) -> u16 {
        self.handler_cursor
    }

    #[inline]
    pub const fn flags(self) -> FrameFlags {
        self.flags
    }

    #[inline]
    pub const fn kind(self) -> ExecutionContextKind {
        self.kind
    }

    #[inline]
    pub const fn resume_active(self) -> bool {
        self.resume_active
    }

    #[inline]
    pub const fn resume_kind(self) -> GeneratorResumeKind {
        self.resume_kind
    }

    #[inline]
    pub const fn resume_value(self) -> Value {
        self.resume_value
    }

    #[inline]
    pub(crate) const fn clear_resume(&mut self) {
        self.resume_active = false;
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct FrameMetadata {
    code: CodeRef,
    parameter_initializer_end_offset: u32,
    registers: RegisterWindow,
    return_register: Option<u16>,
    realm: RealmRef,
    variable_env: EnvironmentRef,
    this_value: Value,
    construct_this: Option<ObjectRef>,
    new_target: Option<ObjectRef>,
    callee: Option<ObjectRef>,
    tail_caller: Option<ObjectRef>,
    tail_caller_strict: bool,
    kind: ExecutionContextKind,
}

#[cfg(test)]
impl FrameMetadata {
    #[inline]
    const fn from_frame(frame: &FrameRecord) -> Self {
        Self {
            code: frame.code,
            parameter_initializer_end_offset: frame.parameter_initializer_end_offset,
            registers: frame.registers,
            return_register: frame.return_register,
            realm: frame.realm,
            variable_env: frame.variable_env,
            this_value: frame.this_value,
            construct_this: frame.construct_this,
            new_target: frame.new_target,
            callee: frame.callee,
            tail_caller: frame.tail_caller,
            tail_caller_strict: frame.tail_caller_strict,
            kind: frame.kind,
        }
    }

    #[inline]
    const fn code(&self) -> CodeRef {
        self.code
    }

    #[inline]
    #[cfg(test)]
    const fn registers(&self) -> RegisterWindow {
        self.registers
    }

    #[inline]
    #[cfg(test)]
    const fn return_register(&self) -> Option<u16> {
        self.return_register
    }

    #[inline]
    #[cfg(test)]
    const fn realm(&self) -> RealmRef {
        self.realm
    }

    #[inline]
    #[cfg(test)]
    const fn variable_env(&self) -> EnvironmentRef {
        self.variable_env
    }

    #[inline]
    #[cfg(test)]
    const fn kind(&self) -> ExecutionContextKind {
        self.kind
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct FrameState {
    instruction_offset: u32,
    lexical_env: EnvironmentRef,
    handler_cursor: u16,
    flags: FrameFlags,
    resume_kind: GeneratorResumeKind,
    resume_value: Value,
    resume_active: bool,
}

#[cfg(test)]
impl FrameState {
    #[inline]
    const fn from_frame(frame: &FrameRecord) -> Self {
        Self {
            instruction_offset: frame.instruction_offset,
            lexical_env: frame.lexical_env,
            handler_cursor: frame.handler_cursor,
            flags: frame.flags,
            resume_kind: frame.resume_kind,
            resume_value: frame.resume_value,
            resume_active: frame.resume_active,
        }
    }

    #[inline]
    #[cfg(test)]
    const fn set_instruction_offset(&mut self, instruction_offset: u32) {
        self.instruction_offset = instruction_offset;
    }

    #[inline]
    #[cfg(test)]
    const fn set_lexical_env(&mut self, lexical_env: EnvironmentRef) {
        self.lexical_env = lexical_env;
    }

    #[inline]
    #[cfg(test)]
    const fn set_handler_cursor(&mut self, handler_cursor: u16) {
        self.handler_cursor = handler_cursor;
    }

    #[inline]
    #[cfg(test)]
    const fn clear_resume(&mut self) {
        self.resume_active = false;
    }

    #[inline]
    #[cfg(test)]
    pub(crate) const fn write_back(&self, frame: &mut FrameRecord) {
        frame.instruction_offset = self.instruction_offset;
        frame.lexical_env = self.lexical_env;
        frame.handler_cursor = self.handler_cursor;
        frame.flags = self.flags;
        frame.resume_kind = self.resume_kind;
        frame.resume_value = self.resume_value;
        frame.resume_active = self.resume_active;
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
    fn dispatch_state_writeback_updates_mutable_fields_only() {
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

        let metadata = FrameMetadata::from_frame(&frame);
        let mut state = FrameState::from_frame(&frame);
        state.set_instruction_offset(41);
        state.set_handler_cursor(9);
        state.set_lexical_env(replacement_env);
        state.clear_resume();
        state.write_back(&mut frame);

        assert_eq!(metadata.code(), code);
        assert_eq!(metadata.registers(), registers);
        assert_eq!(metadata.return_register(), Some(3));
        assert_eq!(metadata.realm(), realm);
        assert_eq!(metadata.variable_env(), variable_env);
        assert_eq!(metadata.kind(), ExecutionContextKind::Function);
        assert_eq!(frame.code(), code);
        assert_eq!(frame.registers(), registers);
        assert_eq!(frame.return_register(), Some(3));
        assert_eq!(frame.realm(), realm);
        assert_eq!(frame.variable_env(), variable_env);
        assert_eq!(frame.kind(), ExecutionContextKind::Function);
        assert_eq!(frame.instruction_offset(), 41);
        assert_eq!(frame.handler_cursor(), 9);
        assert_eq!(frame.lexical_env(), replacement_env);
        assert_eq!(frame.flags(), FrameFlags::entry());
        assert!(!frame.resume_active());
    }
}

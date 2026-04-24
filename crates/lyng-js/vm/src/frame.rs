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

/// Frame record scaffold matching the Phase 4 call-frame contract.
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
    pub(crate) fn set_tail_caller(
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
    pub(crate) fn set_handler_cursor(&mut self, handler_cursor: u16) {
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
    pub(crate) fn set_instruction_offset(&mut self, instruction_offset: u32) {
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
    pub(crate) fn set_lexical_env(&mut self, lexical_env: EnvironmentRef) {
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
    pub(crate) fn set_this_value(&mut self, this_value: Value) {
        self.this_value = this_value;
    }

    #[inline]
    pub const fn construct_this(self) -> Option<ObjectRef> {
        self.construct_this
    }

    #[inline]
    pub(crate) fn set_construct_this(&mut self, construct_this: Option<ObjectRef>) {
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
    pub(crate) fn clear_resume(&mut self) {
        self.resume_active = false;
    }
}

/// Reserve a VM-facing evaluation shell without claiming a working interpreter yet.
#[inline]
pub fn seed_registers(window: RegisterWindow) -> Vec<Value> {
    vec![Value::undefined(); usize::from(window.len())]
}

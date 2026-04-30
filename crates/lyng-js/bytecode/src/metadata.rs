use crate::{BytecodeEnvironmentBinding, EnvironmentLayoutRef};
use lyng_js_common::{AtomId, SourceId};
use lyng_js_types::{BuiltinFunctionId, FeedbackSlotId, ShapeId};

/// Activation policy for the `arguments` object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ArgumentsMode {
    None,
    Unmapped,
    Mapped,
}

/// `this` binding strategy recorded in one bytecode header.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ThisMode {
    Lexical,
    Strict,
    Global,
}

/// High-level template classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BytecodeFunctionKind {
    Script,
    Module,
    Function,
    Arrow,
    Builtin,
}

/// Boolean flags that affect bytecode activation or call behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BytecodeFunctionFlags {
    strict: bool,
    tail_call_capable: bool,
    non_constructible: bool,
    has_prototype_property: bool,
    class_constructor: bool,
    derived_class_constructor: bool,
    generator: bool,
    async_function: bool,
}

impl BytecodeFunctionFlags {
    #[inline]
    pub const fn empty() -> Self {
        Self {
            strict: false,
            tail_call_capable: false,
            non_constructible: false,
            has_prototype_property: true,
            class_constructor: false,
            derived_class_constructor: false,
            generator: false,
            async_function: false,
        }
    }

    #[inline]
    pub const fn new(strict: bool, tail_call_capable: bool) -> Self {
        Self {
            strict,
            tail_call_capable,
            non_constructible: false,
            has_prototype_property: true,
            class_constructor: false,
            derived_class_constructor: false,
            generator: false,
            async_function: false,
        }
    }

    #[inline]
    pub const fn strict(self) -> bool {
        self.strict
    }

    #[inline]
    pub const fn tail_call_capable(self) -> bool {
        self.tail_call_capable
    }

    #[inline]
    pub const fn constructible(self) -> bool {
        !self.non_constructible
    }

    #[inline]
    pub const fn class_constructor(self) -> bool {
        self.class_constructor
    }

    #[inline]
    pub const fn derived_class_constructor(self) -> bool {
        self.derived_class_constructor
    }

    #[inline]
    pub const fn has_prototype_property(self) -> bool {
        self.has_prototype_property
    }

    #[inline]
    pub const fn generator(self) -> bool {
        self.generator
    }

    #[inline]
    pub const fn async_function(self) -> bool {
        self.async_function
    }

    #[inline]
    pub const fn with_tail_call_capable(mut self, tail_call_capable: bool) -> Self {
        self.tail_call_capable = tail_call_capable;
        self
    }

    #[inline]
    pub const fn with_constructible(mut self, constructible: bool) -> Self {
        self.non_constructible = !constructible;
        self
    }

    #[inline]
    pub const fn with_has_prototype_property(mut self, has_prototype_property: bool) -> Self {
        self.has_prototype_property = has_prototype_property;
        self
    }

    #[inline]
    pub const fn with_class_constructor(mut self, class_constructor: bool) -> Self {
        self.class_constructor = class_constructor;
        self
    }

    #[inline]
    pub const fn with_derived_class_constructor(mut self, derived_class_constructor: bool) -> Self {
        self.derived_class_constructor = derived_class_constructor;
        self
    }

    #[inline]
    pub const fn with_generator(mut self, generator: bool) -> Self {
        self.generator = generator;
        self
    }

    #[inline]
    pub const fn with_async_function(mut self, async_function: bool) -> Self {
        self.async_function = async_function;
        self
    }
}

/// Typed constant pool entries reserved for Phase 4 bytecode templates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConstantValue {
    Smi(i32),
    Float64Bits(u64),
    Atom(AtomId),
    Builtin(BuiltinFunctionId),
}

/// Describes how a child closure resolves one captured binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CaptureSource {
    EnvironmentSlot { depth: u16, slot: u16 },
    ParentCapture { index: u16 },
}

/// Capture metadata recorded in the owning bytecode template.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CaptureDescriptor {
    name: Option<AtomId>,
    source: CaptureSource,
}

impl CaptureDescriptor {
    #[inline]
    pub const fn new(name: Option<AtomId>, source: CaptureSource) -> Self {
        Self { name, source }
    }

    #[inline]
    pub const fn name(self) -> Option<AtomId> {
        self.name
    }

    #[inline]
    pub const fn source(self) -> CaptureSource {
        self.source
    }
}

/// Compiler-owned metadata for one loop-iteration lexical-environment site.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LoopIterationEnvironmentSite {
    instruction_offset: u32,
    iteration_slots: Vec<u16>,
    shared_slots: Vec<u16>,
    detached_slots: Vec<u16>,
}

impl LoopIterationEnvironmentSite {
    #[inline]
    pub fn new(
        instruction_offset: u32,
        iteration_slots: Vec<u16>,
        shared_slots: Vec<u16>,
        detached_slots: Vec<u16>,
    ) -> Self {
        Self {
            instruction_offset,
            iteration_slots,
            shared_slots,
            detached_slots,
        }
    }

    #[inline]
    pub const fn instruction_offset(&self) -> u32 {
        self.instruction_offset
    }

    #[inline]
    pub fn iteration_slots(&self) -> &[u16] {
        &self.iteration_slots
    }

    #[inline]
    pub fn shared_slots(&self) -> &[u16] {
        &self.shared_slots
    }

    #[inline]
    pub fn detached_slots(&self) -> &[u16] {
        &self.detached_slots
    }
}

/// Compiler-owned metadata for one active lexical scope at a direct-eval call site.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DirectEvalLexicalScope {
    source_base: u32,
    bindings: Vec<BytecodeEnvironmentBinding>,
    annex_b_catch_name: Option<AtomId>,
}

impl DirectEvalLexicalScope {
    #[inline]
    pub fn new(source_base: u32, bindings: Vec<BytecodeEnvironmentBinding>) -> Self {
        Self {
            source_base,
            bindings,
            annex_b_catch_name: None,
        }
    }

    #[inline]
    pub const fn with_annex_b_catch_name(mut self, name: AtomId) -> Self {
        self.annex_b_catch_name = Some(name);
        self
    }

    #[inline]
    pub const fn source_base(&self) -> u32 {
        self.source_base
    }

    #[inline]
    pub fn bindings(&self) -> &[BytecodeEnvironmentBinding] {
        &self.bindings
    }

    #[inline]
    pub const fn annex_b_catch_name(&self) -> Option<AtomId> {
        self.annex_b_catch_name
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DirectEvalSiteFlags(u8);

impl DirectEvalSiteFlags {
    const FORBID_ARGUMENTS_IN_CLASS_INITIALIZER: u8 = 1 << 0;

    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn forbid_arguments_in_class_initializer(self) -> bool {
        self.0 & Self::FORBID_ARGUMENTS_IN_CLASS_INITIALIZER != 0
    }

    #[inline]
    pub const fn with_forbid_arguments_in_class_initializer(mut self, enabled: bool) -> Self {
        if enabled {
            self.0 |= Self::FORBID_ARGUMENTS_IN_CLASS_INITIALIZER;
        }
        self
    }
}

/// Compiler-owned metadata for one direct-eval lexical-heritage site.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DirectEvalLexicalSite {
    instruction_offset: u32,
    scopes: Vec<DirectEvalLexicalScope>,
    flags: DirectEvalSiteFlags,
    annex_b_catch_names: Vec<AtomId>,
    parameter_names: Vec<AtomId>,
}

impl DirectEvalLexicalSite {
    #[inline]
    pub fn new(
        instruction_offset: u32,
        scopes: Vec<DirectEvalLexicalScope>,
        flags: DirectEvalSiteFlags,
        annex_b_catch_names: Vec<AtomId>,
        parameter_names: Vec<AtomId>,
    ) -> Self {
        Self {
            instruction_offset,
            scopes,
            flags,
            annex_b_catch_names,
            parameter_names,
        }
    }

    #[inline]
    pub const fn instruction_offset(&self) -> u32 {
        self.instruction_offset
    }

    #[inline]
    pub fn scopes(&self) -> &[DirectEvalLexicalScope] {
        &self.scopes
    }

    #[inline]
    pub const fn flags(&self) -> DirectEvalSiteFlags {
        self.flags
    }

    #[inline]
    pub fn annex_b_catch_names(&self) -> &[AtomId] {
        &self.annex_b_catch_names
    }

    #[inline]
    pub fn parameter_names(&self) -> &[AtomId] {
        &self.parameter_names
    }
}

/// Exception-handler classification for one protected region.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExceptionHandlerKind {
    Catch,
    Finally,
}

/// Exception-table entry owned by the bytecode layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExceptionHandler {
    protected_start: u32,
    protected_end: u32,
    handler: u32,
    kind: ExceptionHandlerKind,
    stack_depth: u16,
    target_register: Option<u16>,
}

impl ExceptionHandler {
    #[inline]
    pub const fn new(
        protected_start: u32,
        protected_end: u32,
        handler: u32,
        kind: ExceptionHandlerKind,
        stack_depth: u16,
        target_register: Option<u16>,
    ) -> Self {
        Self {
            protected_start,
            protected_end,
            handler,
            kind,
            stack_depth,
            target_register,
        }
    }

    #[inline]
    pub const fn protected_start(self) -> u32 {
        self.protected_start
    }

    #[inline]
    pub const fn protected_end(self) -> u32 {
        self.protected_end
    }

    #[inline]
    pub const fn handler(self) -> u32 {
        self.handler
    }

    #[inline]
    pub const fn kind(self) -> ExceptionHandlerKind {
        self.kind
    }

    #[inline]
    pub const fn stack_depth(self) -> u16 {
        self.stack_depth
    }

    #[inline]
    pub const fn target_register(self) -> Option<u16> {
        self.target_register
    }
}

/// Phase 4 feedback-site kinds owned by the compiler and bytecode layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FeedbackSiteKind {
    Arithmetic,
    Comparison,
    NamedPropertyLoad,
    NamedPropertyStore,
    KeyedPropertyAccess,
    Call,
    Construct,
}

/// Auxiliary descriptor payload for one feedback site.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FeedbackSiteMetadata {
    None,
    ExpectedArity(u16),
    CallArguments {
        expected_arity: u16,
        spread_mask: u64,
    },
    NamedProperty(AtomId),
    KeyedProperty,
}

impl FeedbackSiteMetadata {
    #[inline]
    pub const fn expected_arity(self) -> Option<u16> {
        match self {
            Self::ExpectedArity(arity) => Some(arity),
            Self::CallArguments { expected_arity, .. } => Some(expected_arity),
            _ => None,
        }
    }

    #[inline]
    pub const fn spread_mask(self) -> Option<u64> {
        match self {
            Self::CallArguments { spread_mask, .. } => Some(spread_mask),
            _ => None,
        }
    }
}

/// Compiler-owned feedback descriptor shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FeedbackSiteDescriptor {
    slot: FeedbackSlotId,
    instruction_offset: u32,
    kind: FeedbackSiteKind,
    metadata: FeedbackSiteMetadata,
}

/// Auxiliary call metadata packed into one wide side-table payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CallRange {
    argument_base: u16,
    argument_count: u16,
}

impl CallRange {
    #[inline]
    pub const fn new(argument_base: u16, argument_count: u16) -> Self {
        Self {
            argument_base,
            argument_count,
        }
    }

    #[inline]
    pub const fn argument_base(self) -> u16 {
        self.argument_base
    }

    #[inline]
    pub const fn argument_count(self) -> u16 {
        self.argument_count
    }

    #[inline]
    pub const fn encode(self) -> u32 {
        ((self.argument_base as u32) << 16) | (self.argument_count as u32)
    }

    #[inline]
    pub fn decode(payload: u32) -> Self {
        Self {
            argument_base: masked_u16(payload >> 16),
            argument_count: masked_u16(payload),
        }
    }
}

/// Widened logical operands for one `ABC` instruction family member.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WideAbcOperands {
    a: u16,
    b: u16,
    c: u16,
}

impl WideAbcOperands {
    #[inline]
    pub const fn new(a: u16, b: u16, c: u16) -> Self {
        Self { a, b, c }
    }

    #[inline]
    pub const fn narrow(a: u8, b: u8, c: u8) -> Self {
        Self::new(a as u16, b as u16, c as u16)
    }

    #[inline]
    pub const fn decode(a: u8, b: u8, c: u8, payload: u32) -> Self {
        Self::new(
            (((payload >> 16) & 0xff) as u16) << 8 | (a as u16),
            (((payload >> 8) & 0xff) as u16) << 8 | (b as u16),
            ((payload & 0xff) as u16) << 8 | (c as u16),
        )
    }

    #[inline]
    pub const fn a(self) -> u16 {
        self.a
    }

    #[inline]
    pub const fn b(self) -> u16 {
        self.b
    }

    #[inline]
    pub const fn c(self) -> u16 {
        self.c
    }

    #[inline]
    pub fn narrow_a(self) -> u8 {
        narrow_u8(self.a)
    }

    #[inline]
    pub fn narrow_b(self) -> u8 {
        narrow_u8(self.b)
    }

    #[inline]
    pub fn narrow_c(self) -> u8 {
        narrow_u8(self.c)
    }

    #[inline]
    pub const fn needs_wide(self) -> bool {
        self.a > u8::MAX as u16 || self.b > u8::MAX as u16 || self.c > u8::MAX as u16
    }

    #[inline]
    pub const fn encode_payload(self) -> u32 {
        (((self.a >> 8) as u32) << 16) | (((self.b >> 8) as u32) << 8) | ((self.c >> 8) as u32)
    }
}

/// Widened logical operands for one `ABx` instruction family member.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WideAbxOperands {
    a: u16,
    bx: u32,
}

impl WideAbxOperands {
    #[inline]
    pub const fn new(a: u16, bx: u32) -> Self {
        Self { a, bx }
    }

    #[inline]
    pub const fn narrow(a: u8, bx: u16) -> Self {
        Self::new(a as u16, bx as u32)
    }

    #[inline]
    pub const fn decode(a: u8, bx: u16, payload: u32) -> Self {
        Self::new(
            (((payload >> 24) & 0xff) as u16) << 8 | (a as u16),
            ((payload & 0x0000_ffff) << 16) | (bx as u32),
        )
    }

    #[inline]
    pub const fn a(self) -> u16 {
        self.a
    }

    #[inline]
    pub const fn bx(self) -> u32 {
        self.bx
    }

    #[inline]
    pub fn narrow_a(self) -> u8 {
        narrow_u8(self.a)
    }

    #[inline]
    pub fn narrow_bx(self) -> u16 {
        masked_u16(self.bx)
    }

    #[inline]
    pub const fn needs_wide(self) -> bool {
        self.a > u8::MAX as u16 || self.bx > u16::MAX as u32
    }

    #[inline]
    pub const fn encode_payload(self) -> u32 {
        (((self.a >> 8) as u32) << 24) | ((self.bx >> 16) & 0x0000_ffff)
    }
}

impl FeedbackSiteDescriptor {
    #[inline]
    pub const fn new(
        slot: FeedbackSlotId,
        instruction_offset: u32,
        kind: FeedbackSiteKind,
    ) -> Self {
        Self {
            slot,
            instruction_offset,
            kind,
            metadata: FeedbackSiteMetadata::None,
        }
    }

    #[inline]
    pub const fn with_metadata(mut self, metadata: FeedbackSiteMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    #[inline]
    pub const fn slot(self) -> FeedbackSlotId {
        self.slot
    }

    #[inline]
    pub const fn instruction_offset(self) -> u32 {
        self.instruction_offset
    }

    #[inline]
    pub const fn kind(self) -> FeedbackSiteKind {
        self.kind
    }

    #[inline]
    pub const fn metadata(self) -> FeedbackSiteMetadata {
        self.metadata
    }
}

/// Source mapping owned by one bytecode template.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SourceMapEntry {
    source: SourceId,
    instruction_offset: u32,
    start: u32,
    end: u32,
}

impl SourceMapEntry {
    #[inline]
    pub const fn new(source: SourceId, instruction_offset: u32, start: u32, end: u32) -> Self {
        Self {
            source,
            instruction_offset,
            start,
            end,
        }
    }

    #[inline]
    pub const fn source(self) -> SourceId {
        self.source
    }

    #[inline]
    pub const fn instruction_offset(self) -> u32 {
        self.instruction_offset
    }

    #[inline]
    pub const fn start(self) -> u32 {
        self.start
    }

    #[inline]
    pub const fn end(self) -> u32 {
        self.end
    }
}

/// Interpreter-visible safepoint categories frozen for later optimized tiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SafepointKind {
    Allocation,
    LoopBackedge,
    ExceptionEdge,
}

/// Optional wide-operand side table entry reserved for later large-function support.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WideOperand {
    instruction_offset: u32,
    payload: u32,
}

impl WideOperand {
    #[inline]
    pub const fn new(instruction_offset: u32, payload: u32) -> Self {
        Self {
            instruction_offset,
            payload,
        }
    }

    #[inline]
    pub const fn instruction_offset(self) -> u32 {
        self.instruction_offset
    }

    #[inline]
    pub const fn payload(self) -> u32 {
        self.payload
    }
}

/// Runtime state carried at one safepoint for later GC and deoptimization work.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RuntimeStateCapture {
    flags: u8,
}

impl RuntimeStateCapture {
    const LEXICAL_ENV: u8 = 1 << 0;
    const VARIABLE_ENV: u8 = 1 << 1;
    const THIS_VALUE: u8 = 1 << 2;
    const NEW_TARGET: u8 = 1 << 3;
    const CALLEE: u8 = 1 << 4;
    const EXCEPTION_STATE: u8 = 1 << 5;
    const COMPLETION_STATE: u8 = 1 << 6;

    #[inline]
    pub const fn new() -> Self {
        Self { flags: 0 }
    }

    #[inline]
    pub const fn with_lexical_env(mut self, capture: bool) -> Self {
        self = self.with_flag(Self::LEXICAL_ENV, capture);
        self
    }

    #[inline]
    pub const fn with_variable_env(mut self, capture: bool) -> Self {
        self = self.with_flag(Self::VARIABLE_ENV, capture);
        self
    }

    #[inline]
    pub const fn with_this_value(mut self, capture: bool) -> Self {
        self = self.with_flag(Self::THIS_VALUE, capture);
        self
    }

    #[inline]
    pub const fn with_new_target(mut self, capture: bool) -> Self {
        self = self.with_flag(Self::NEW_TARGET, capture);
        self
    }

    #[inline]
    pub const fn with_callee(mut self, capture: bool) -> Self {
        self = self.with_flag(Self::CALLEE, capture);
        self
    }

    #[inline]
    pub const fn with_exception_state(mut self, capture: bool) -> Self {
        self = self.with_flag(Self::EXCEPTION_STATE, capture);
        self
    }

    #[inline]
    pub const fn with_completion_state(mut self, capture: bool) -> Self {
        self = self.with_flag(Self::COMPLETION_STATE, capture);
        self
    }

    #[inline]
    pub const fn lexical_env(self) -> bool {
        self.has_flag(Self::LEXICAL_ENV)
    }

    #[inline]
    pub const fn variable_env(self) -> bool {
        self.has_flag(Self::VARIABLE_ENV)
    }

    #[inline]
    pub const fn this_value(self) -> bool {
        self.has_flag(Self::THIS_VALUE)
    }

    #[inline]
    pub const fn new_target(self) -> bool {
        self.has_flag(Self::NEW_TARGET)
    }

    #[inline]
    pub const fn callee(self) -> bool {
        self.has_flag(Self::CALLEE)
    }

    #[inline]
    pub const fn exception_state(self) -> bool {
        self.has_flag(Self::EXCEPTION_STATE)
    }

    #[inline]
    pub const fn completion_state(self) -> bool {
        self.has_flag(Self::COMPLETION_STATE)
    }

    #[inline]
    const fn with_flag(mut self, flag: u8, enabled: bool) -> Self {
        if enabled {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
        self
    }

    #[inline]
    const fn has_flag(self, flag: u8) -> bool {
        self.flags & flag != 0
    }
}

/// Safepoint descriptor shell reserved for later precise GC and deoptimization work.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SafepointDescriptor {
    id: u32,
    instruction_offset: u32,
    kind: SafepointKind,
    register_window_len: u16,
    environment_layout: Option<EnvironmentLayoutRef>,
    runtime_state: RuntimeStateCapture,
}

impl SafepointDescriptor {
    #[inline]
    pub const fn new(
        id: u32,
        instruction_offset: u32,
        kind: SafepointKind,
        register_window_len: u16,
    ) -> Self {
        Self {
            id,
            instruction_offset,
            kind,
            register_window_len,
            environment_layout: None,
            runtime_state: RuntimeStateCapture::new(),
        }
    }

    #[inline]
    pub const fn with_environment_layout(
        mut self,
        environment_layout: Option<EnvironmentLayoutRef>,
    ) -> Self {
        self.environment_layout = environment_layout;
        self
    }

    #[inline]
    pub const fn with_register_window_len(mut self, register_window_len: u16) -> Self {
        self.register_window_len = register_window_len;
        self
    }

    #[inline]
    pub const fn with_runtime_state(mut self, runtime_state: RuntimeStateCapture) -> Self {
        self.runtime_state = runtime_state;
        self
    }

    #[inline]
    pub const fn id(self) -> u32 {
        self.id
    }

    #[inline]
    pub const fn instruction_offset(self) -> u32 {
        self.instruction_offset
    }

    #[inline]
    pub const fn kind(self) -> SafepointKind {
        self.kind
    }

    #[inline]
    pub const fn register_window_len(self) -> u16 {
        self.register_window_len
    }

    #[inline]
    pub const fn environment_layout(self) -> Option<EnvironmentLayoutRef> {
        self.environment_layout
    }

    #[inline]
    pub const fn captures_lexical_env(self) -> bool {
        self.runtime_state.lexical_env()
    }

    #[inline]
    pub const fn captures_variable_env(self) -> bool {
        self.runtime_state.variable_env()
    }

    #[inline]
    pub const fn captures_this(self) -> bool {
        self.runtime_state.this_value()
    }

    #[inline]
    pub const fn captures_new_target(self) -> bool {
        self.runtime_state.new_target()
    }

    #[inline]
    pub const fn captures_callee(self) -> bool {
        self.runtime_state.callee()
    }

    #[inline]
    pub const fn captures_exception_state(self) -> bool {
        self.runtime_state.exception_state()
    }

    #[inline]
    pub const fn captures_completion_state(self) -> bool {
        self.runtime_state.completion_state()
    }
}

/// Interpreter-visible location for one deoptimization value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeoptFrameValue {
    ThisValue,
    NewTarget,
    Callee,
    ExceptionValue,
    CompletionKind,
    CompletionValue,
    CompletionTarget,
}

/// Interpreter-visible location for one deoptimization value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeoptValueSource {
    Register(u16),
    EnvironmentSlot { depth: u16, slot: u16 },
    Constant(u32),
    Shape(ShapeId),
    FrameValue(DeoptFrameValue),
}

/// Compiler-owned deoptimization snapshot shell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeoptSnapshot {
    safepoint_id: u32,
    values: Vec<DeoptValueSource>,
}

impl DeoptSnapshot {
    #[inline]
    pub fn new(safepoint_id: u32, values: Vec<DeoptValueSource>) -> Self {
        Self {
            safepoint_id,
            values,
        }
    }

    #[inline]
    pub const fn safepoint_id(&self) -> u32 {
        self.safepoint_id
    }

    #[inline]
    pub fn values(&self) -> &[DeoptValueSource] {
        &self.values
    }
}

#[inline]
fn masked_u16(value: u32) -> u16 {
    u16::try_from(value & u32::from(u16::MAX)).expect("value is masked to 16 bits")
}

#[inline]
fn narrow_u8(value: u16) -> u8 {
    u8::try_from(value & u16::from(u8::MAX)).expect("value is masked to one byte")
}

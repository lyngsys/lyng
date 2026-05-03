use lyng_js_common::AtomId;
use lyng_js_gc::{PrimitiveTracer, TraceHeapEdges};
use lyng_js_types::{CodeRef, EnvironmentRef, ObjectRef, RealmRef, ShapeId, StringRef, Value};
use std::ops::Range;

/// Execution identity categories reserved by the Phase 3 runtime substrate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutableId {
    Script,
    Module,
    Builtin,
    Bytecode(CodeRef),
}

mod intrinsics;

pub use intrinsics::Intrinsics;

/// Stable execution-context classification frozen by Phase 3.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionContextKind {
    Script,
    Module,
    Builtin,
    Function,
    Eval,
    Job,
}

/// Current `this` state tracked by one execution context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThisState {
    Lexical,
    Uninitialized,
    Value(Value),
}

/// Cold execution-context record owned by `lyng_js_env`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExecutionContext {
    realm: RealmRef,
    executable: ExecutableId,
    lexical_env: EnvironmentRef,
    variable_env: EnvironmentRef,
    private_env: Option<EnvironmentRef>,
    script_or_module_referrer: Option<AtomId>,
    this_state: ThisState,
    new_target: Option<ObjectRef>,
    kind: ExecutionContextKind,
}

impl ExecutionContext {
    #[inline]
    pub const fn new(
        kind: ExecutionContextKind,
        realm: RealmRef,
        executable: ExecutableId,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) -> Self {
        Self {
            realm,
            executable,
            lexical_env,
            variable_env,
            private_env: None,
            script_or_module_referrer: None,
            this_state: ThisState::Uninitialized,
            new_target: None,
            kind,
        }
    }

    #[inline]
    pub const fn script(
        realm: RealmRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) -> Self {
        Self::new(
            ExecutionContextKind::Script,
            realm,
            ExecutableId::Script,
            lexical_env,
            variable_env,
        )
    }

    #[inline]
    pub const fn module(
        realm: RealmRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) -> Self {
        Self::new(
            ExecutionContextKind::Module,
            realm,
            ExecutableId::Module,
            lexical_env,
            variable_env,
        )
    }

    #[inline]
    pub const fn builtin(
        realm: RealmRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) -> Self {
        Self::new(
            ExecutionContextKind::Builtin,
            realm,
            ExecutableId::Builtin,
            lexical_env,
            variable_env,
        )
    }

    #[inline]
    pub const fn eval(
        realm: RealmRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) -> Self {
        Self::new(
            ExecutionContextKind::Eval,
            realm,
            ExecutableId::Script,
            lexical_env,
            variable_env,
        )
    }

    #[inline]
    pub const fn job(
        realm: RealmRef,
        executable: ExecutableId,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) -> Self {
        Self::new(
            ExecutionContextKind::Job,
            realm,
            executable,
            lexical_env,
            variable_env,
        )
    }

    #[inline]
    pub const fn bytecode(
        realm: RealmRef,
        code: CodeRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) -> Self {
        Self::new(
            ExecutionContextKind::Function,
            realm,
            ExecutableId::Bytecode(code),
            lexical_env,
            variable_env,
        )
    }

    #[inline]
    pub const fn with_private_env(mut self, private_env: Option<EnvironmentRef>) -> Self {
        self.private_env = private_env;
        self
    }

    #[inline]
    pub const fn with_script_or_module_referrer(mut self, referrer: Option<AtomId>) -> Self {
        self.script_or_module_referrer = referrer;
        self
    }

    #[inline]
    pub const fn with_this_state(mut self, this_state: ThisState) -> Self {
        self.this_state = this_state;
        self
    }

    #[inline]
    pub const fn with_new_target(mut self, new_target: Option<ObjectRef>) -> Self {
        self.new_target = new_target;
        self
    }

    #[inline]
    pub const fn realm(self) -> RealmRef {
        self.realm
    }

    #[inline]
    pub const fn executable(self) -> ExecutableId {
        self.executable
    }

    #[inline]
    pub const fn lexical_env(self) -> EnvironmentRef {
        self.lexical_env
    }

    #[inline]
    pub const fn variable_env(self) -> EnvironmentRef {
        self.variable_env
    }

    #[inline]
    pub const fn private_env(self) -> Option<EnvironmentRef> {
        self.private_env
    }

    #[inline]
    pub const fn script_or_module_referrer(self) -> Option<AtomId> {
        self.script_or_module_referrer
    }

    #[inline]
    pub const fn this_state(self) -> ThisState {
        self.this_state
    }

    #[inline]
    pub const fn new_target(self) -> Option<ObjectRef> {
        self.new_target
    }

    #[inline]
    pub const fn kind(self) -> ExecutionContextKind {
        self.kind
    }
}

/// Read-only view over one realm record and its typed intrinsic table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RealmRecord {
    pub(crate) id: RealmRef,
    pub(crate) global_object: ObjectRef,
    pub(crate) global_env: EnvironmentRef,
    pub(crate) bootstrap_code: Option<CodeRef>,
    pub(crate) root_shape: Option<ShapeId>,
    pub(crate) intrinsics: Intrinsics,
    pub(crate) bootstrap_state: RealmBootstrapState,
    pub(crate) is_default: bool,
}

impl RealmRecord {
    #[inline]
    pub const fn id(self) -> RealmRef {
        self.id
    }

    #[inline]
    pub const fn global_object(self) -> ObjectRef {
        self.global_object
    }

    #[inline]
    pub const fn global_env(self) -> EnvironmentRef {
        self.global_env
    }

    #[inline]
    pub const fn bootstrap_code(self) -> Option<CodeRef> {
        self.bootstrap_code
    }

    #[inline]
    pub const fn root_shape(self) -> Option<ShapeId> {
        self.root_shape
    }

    #[inline]
    pub const fn intrinsics(self) -> Intrinsics {
        self.intrinsics
    }

    #[inline]
    pub const fn bootstrap_state(self) -> RealmBootstrapState {
        self.bootstrap_state
    }

    #[inline]
    pub const fn is_default(self) -> bool {
        self.is_default
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RealmBootstrapState {
    spec_ready: bool,
    embedding_ready: bool,
}

impl RealmBootstrapState {
    #[inline]
    pub const fn new() -> Self {
        Self {
            spec_ready: false,
            embedding_ready: false,
        }
    }

    #[inline]
    pub const fn spec_ready(self) -> bool {
        self.spec_ready
    }

    #[inline]
    pub const fn embedding_ready(self) -> bool {
        self.embedding_ready
    }

    #[inline]
    pub const fn with_spec_ready(mut self, spec_ready: bool) -> Self {
        self.spec_ready = spec_ready;
        self
    }

    #[inline]
    pub const fn with_embedding_ready(mut self, embedding_ready: bool) -> Self {
        self.embedding_ready = embedding_ready;
        self
    }
}

impl TraceHeapEdges for ExecutableId {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        if let Self::Bytecode(code) = self {
            code.trace_heap_edges(tracer);
        }
    }
}

impl TraceHeapEdges for ThisState {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        if let Self::Value(value) = self {
            value.trace_heap_edges(tracer);
        }
    }
}

impl TraceHeapEdges for ExecutionContext {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.realm.trace_heap_edges(tracer);
        self.executable.trace_heap_edges(tracer);
        self.lexical_env.trace_heap_edges(tracer);
        self.variable_env.trace_heap_edges(tracer);
        self.private_env.trace_heap_edges(tracer);
        self.this_state.trace_heap_edges(tracer);
        self.new_target.trace_heap_edges(tracer);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RealmMetadata {
    pub(crate) intrinsics: Intrinsics,
    pub(crate) bootstrap_state: RealmBootstrapState,
    pub(crate) is_default: bool,
    pub(crate) regexp_legacy_static_state: RegExpLegacyStaticState,
}

/// Annex B state exposed by the legacy static accessors on a realm's `%RegExp%`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegExpLegacyStaticState {
    input: RegExpLegacyStaticText,
    last_match: RegExpLegacyStaticText,
    last_paren: RegExpLegacyStaticText,
    left_context: RegExpLegacyStaticText,
    right_context: RegExpLegacyStaticText,
    parens: [RegExpLegacyStaticText; 9],
}

/// Lazily materialized text backing for Annex B RegExp legacy static accessors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegExpLegacyStaticText {
    Empty,
    Owned(Vec<u16>),
    SourceSlice {
        source: StringRef,
        range: Range<usize>,
    },
}

impl Default for RegExpLegacyStaticState {
    fn default() -> Self {
        Self {
            input: RegExpLegacyStaticText::Empty,
            last_match: RegExpLegacyStaticText::Empty,
            last_paren: RegExpLegacyStaticText::Empty,
            left_context: RegExpLegacyStaticText::Empty,
            right_context: RegExpLegacyStaticText::Empty,
            parens: std::array::from_fn(|_| RegExpLegacyStaticText::Empty),
        }
    }
}

impl RegExpLegacyStaticState {
    #[inline]
    pub fn input(&self) -> &RegExpLegacyStaticText {
        &self.input
    }

    #[inline]
    pub fn last_match(&self) -> &RegExpLegacyStaticText {
        &self.last_match
    }

    #[inline]
    pub fn last_paren(&self) -> &RegExpLegacyStaticText {
        &self.last_paren
    }

    #[inline]
    pub fn left_context(&self) -> &RegExpLegacyStaticText {
        &self.left_context
    }

    #[inline]
    pub fn right_context(&self) -> &RegExpLegacyStaticText {
        &self.right_context
    }

    #[inline]
    pub fn paren(&self, one_based_index: usize) -> Option<&RegExpLegacyStaticText> {
        one_based_index
            .checked_sub(1)
            .and_then(|index| self.parens.get(index))
    }

    #[inline]
    pub fn set_input(&mut self, input: Vec<u16>) {
        self.input = if input.is_empty() {
            RegExpLegacyStaticText::Empty
        } else {
            RegExpLegacyStaticText::Owned(input)
        };
    }

    pub fn record_match(
        &mut self,
        source: StringRef,
        input_len: usize,
        matched: Range<usize>,
        captures: &[Option<Range<usize>>],
    ) {
        self.input = RegExpLegacyStaticText::source_slice(source, 0..input_len);
        self.last_match = RegExpLegacyStaticText::source_slice(source, matched.clone());
        self.left_context = RegExpLegacyStaticText::source_slice(source, 0..matched.start);
        self.right_context = RegExpLegacyStaticText::source_slice(source, matched.end..input_len);
        self.last_paren = RegExpLegacyStaticText::Empty;

        for (index, slot) in self.parens.iter_mut().enumerate() {
            let Some(Some(range)) = captures.get(index) else {
                *slot = RegExpLegacyStaticText::Empty;
                continue;
            };
            *slot = RegExpLegacyStaticText::source_slice(source, range.clone());
            self.last_paren = slot.clone();
        }
    }
}

impl RegExpLegacyStaticText {
    #[inline]
    fn source_slice(source: StringRef, range: Range<usize>) -> Self {
        if range.start == range.end {
            Self::Empty
        } else {
            Self::SourceSlice { source, range }
        }
    }
}

impl TraceHeapEdges for RegExpLegacyStaticText {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        if let Self::SourceSlice { source, .. } = self {
            source.trace_heap_edges(tracer);
        }
    }
}

impl TraceHeapEdges for RegExpLegacyStaticState {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.input.trace_heap_edges(tracer);
        self.last_match.trace_heap_edges(tracer);
        self.last_paren.trace_heap_edges(tracer);
        self.left_context.trace_heap_edges(tracer);
        self.right_context.trace_heap_edges(tracer);
        for paren in &self.parens {
            paren.trace_heap_edges(tracer);
        }
    }
}

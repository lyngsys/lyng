use super::binding::BytecodeEnvironmentBinding;
use super::header::BytecodeFunctionHeader;
use crate::ids::{BytecodeFunctionId, EnvironmentLayoutRef};
use crate::instruction::Instruction;
use crate::metadata::{
    ArgumentsMode, BytecodeFunctionFlags, BytecodeFunctionKind, CaptureDescriptor, ConstantValue,
    DeoptSnapshot, DirectEvalLexicalSite, ExceptionHandler, FeedbackSiteDescriptor,
    LoopIterationEnvironmentSite, SafepointDescriptor, SourceMapEntry, ThisMode, WideOperand,
};
use lyng_js_common::{AtomId, Span};

/// Immutable bytecode template shared by the compiler and runtime installation layers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BytecodeFunction {
    header: BytecodeFunctionHeader,
    environment_bindings: Vec<BytecodeEnvironmentBinding>,
    instructions: Vec<Instruction>,
    constants: Vec<ConstantValue>,
    child_functions: Vec<BytecodeFunctionId>,
    captures: Vec<CaptureDescriptor>,
    direct_eval_lexical_sites: Vec<DirectEvalLexicalSite>,
    loop_iteration_environment_sites: Vec<LoopIterationEnvironmentSite>,
    exception_handlers: Vec<ExceptionHandler>,
    feedback_sites: Vec<FeedbackSiteDescriptor>,
    source_map: Vec<SourceMapEntry>,
    wide_operands: Vec<WideOperand>,
    safepoints: Vec<SafepointDescriptor>,
    deopt_snapshots: Vec<DeoptSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BytecodeFunctionBody {
    pub(crate) environment_bindings: Vec<BytecodeEnvironmentBinding>,
    pub(crate) instructions: Vec<Instruction>,
    pub(crate) constants: Vec<ConstantValue>,
    pub(crate) child_functions: Vec<BytecodeFunctionId>,
    pub(crate) captures: Vec<CaptureDescriptor>,
    pub(crate) direct_eval_lexical_sites: Vec<DirectEvalLexicalSite>,
    pub(crate) loop_iteration_environment_sites: Vec<LoopIterationEnvironmentSite>,
    pub(crate) exception_handlers: Vec<ExceptionHandler>,
    pub(crate) feedback_sites: Vec<FeedbackSiteDescriptor>,
    pub(crate) source_map: Vec<SourceMapEntry>,
    pub(crate) wide_operands: Vec<WideOperand>,
    pub(crate) safepoints: Vec<SafepointDescriptor>,
    pub(crate) deopt_snapshots: Vec<DeoptSnapshot>,
}

impl BytecodeFunction {
    #[inline]
    pub const fn new(
        id: BytecodeFunctionId,
        name: Option<AtomId>,
        arguments_mode: ArgumentsMode,
    ) -> Self {
        Self {
            header: BytecodeFunctionHeader::new(
                id,
                BytecodeFunctionKind::Function,
                name,
                ThisMode::Global,
                arguments_mode,
            ),
            environment_bindings: Vec::new(),
            instructions: Vec::new(),
            constants: Vec::new(),
            child_functions: Vec::new(),
            captures: Vec::new(),
            direct_eval_lexical_sites: Vec::new(),
            loop_iteration_environment_sites: Vec::new(),
            exception_handlers: Vec::new(),
            feedback_sites: Vec::new(),
            source_map: Vec::new(),
            wide_operands: Vec::new(),
            safepoints: Vec::new(),
            deopt_snapshots: Vec::new(),
        }
    }

    #[inline]
    pub(crate) fn from_parts(header: BytecodeFunctionHeader, body: BytecodeFunctionBody) -> Self {
        Self {
            header,
            environment_bindings: body.environment_bindings,
            instructions: body.instructions,
            constants: body.constants,
            child_functions: body.child_functions,
            captures: body.captures,
            direct_eval_lexical_sites: body.direct_eval_lexical_sites,
            loop_iteration_environment_sites: body.loop_iteration_environment_sites,
            exception_handlers: body.exception_handlers,
            feedback_sites: body.feedback_sites,
            source_map: body.source_map,
            wide_operands: body.wide_operands,
            safepoints: body.safepoints,
            deopt_snapshots: body.deopt_snapshots,
        }
    }

    #[inline]
    pub const fn header(&self) -> BytecodeFunctionHeader {
        self.header
    }

    #[inline]
    pub const fn id(&self) -> BytecodeFunctionId {
        self.header.id()
    }

    #[inline]
    pub const fn kind(&self) -> BytecodeFunctionKind {
        self.header.kind()
    }

    #[inline]
    pub const fn flags(&self) -> BytecodeFunctionFlags {
        self.header.flags()
    }

    #[inline]
    pub const fn name(&self) -> Option<AtomId> {
        self.header.name()
    }

    #[inline]
    pub const fn this_mode(&self) -> ThisMode {
        self.header.this_mode()
    }

    #[inline]
    pub const fn arguments_mode(&self) -> ArgumentsMode {
        self.header.arguments_mode()
    }

    #[inline]
    pub const fn parameter_count(&self) -> u16 {
        self.header.parameter_count()
    }

    #[inline]
    pub const fn minimum_argument_count(&self) -> u16 {
        self.header.minimum_argument_count()
    }

    #[inline]
    pub const fn parameter_initializer_end_offset(&self) -> u32 {
        self.header.parameter_initializer_end_offset()
    }

    #[inline]
    pub const fn register_count(&self) -> u16 {
        self.header.register_count()
    }

    #[inline]
    pub const fn hidden_register_count(&self) -> u16 {
        self.header.hidden_register_count()
    }

    #[inline]
    pub const fn needs_environment(&self) -> bool {
        self.header.needs_environment()
    }

    #[inline]
    pub const fn environment_slot_count(&self) -> u16 {
        self.header.environment_slot_count()
    }

    #[inline]
    pub const fn environment_layout(&self) -> Option<EnvironmentLayoutRef> {
        self.header.environment_layout()
    }

    #[inline]
    pub const fn source_span(&self) -> Option<Span> {
        self.header.source_span()
    }

    #[inline]
    pub const fn has_rest_parameter(&self) -> bool {
        self.header.has_rest_parameter()
    }

    #[inline]
    pub fn environment_bindings(&self) -> &[BytecodeEnvironmentBinding] {
        &self.environment_bindings
    }

    #[inline]
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    #[inline]
    pub fn constants(&self) -> &[ConstantValue] {
        &self.constants
    }

    #[inline]
    pub fn child_functions(&self) -> &[BytecodeFunctionId] {
        &self.child_functions
    }

    #[inline]
    pub fn captures(&self) -> &[CaptureDescriptor] {
        &self.captures
    }

    #[inline]
    pub fn direct_eval_lexical_sites(&self) -> &[DirectEvalLexicalSite] {
        &self.direct_eval_lexical_sites
    }

    #[inline]
    pub fn loop_iteration_environment_sites(&self) -> &[LoopIterationEnvironmentSite] {
        &self.loop_iteration_environment_sites
    }

    #[inline]
    pub fn exception_handlers(&self) -> &[ExceptionHandler] {
        &self.exception_handlers
    }

    #[inline]
    pub fn feedback_sites(&self) -> &[FeedbackSiteDescriptor] {
        &self.feedback_sites
    }

    #[inline]
    pub fn feedback_slot_count(&self) -> u32 {
        self.feedback_sites
            .iter()
            .map(|descriptor| descriptor.slot().get())
            .max()
            .unwrap_or(0)
    }

    #[inline]
    pub fn source_map(&self) -> &[SourceMapEntry] {
        &self.source_map
    }

    #[inline]
    pub fn source_map_entry_at(&self, instruction_offset: u32) -> Option<SourceMapEntry> {
        self.source_map
            .iter()
            .find(|entry| entry.instruction_offset() == instruction_offset)
            .copied()
    }

    #[inline]
    pub fn wide_operands(&self) -> &[WideOperand] {
        &self.wide_operands
    }

    #[inline]
    pub fn safepoints(&self) -> &[SafepointDescriptor] {
        &self.safepoints
    }

    #[inline]
    pub fn safepoint_at(&self, instruction_offset: u32) -> Option<SafepointDescriptor> {
        self.safepoints
            .iter()
            .find(|descriptor| descriptor.instruction_offset() == instruction_offset)
            .copied()
    }

    #[inline]
    pub fn safepoint_by_id(&self, id: u32) -> Option<SafepointDescriptor> {
        self.safepoints
            .iter()
            .find(|descriptor| descriptor.id() == id)
            .copied()
    }

    #[inline]
    pub fn deopt_snapshots(&self) -> &[DeoptSnapshot] {
        &self.deopt_snapshots
    }

    #[inline]
    pub fn deopt_snapshot_for_safepoint(&self, safepoint_id: u32) -> Option<&DeoptSnapshot> {
        self.deopt_snapshots
            .iter()
            .find(|snapshot| snapshot.safepoint_id() == safepoint_id)
    }

    #[inline]
    pub const fn with_kind(mut self, kind: BytecodeFunctionKind) -> Self {
        self.header = self.header.with_kind(kind);
        self
    }

    #[inline]
    pub const fn with_this_mode(mut self, this_mode: ThisMode) -> Self {
        self.header = self.header.with_this_mode(this_mode);
        self
    }

    #[inline]
    pub const fn with_flags(mut self, flags: BytecodeFunctionFlags) -> Self {
        self.header = self.header.with_flags(flags);
        self
    }

    #[inline]
    pub const fn with_parameter_counts(
        mut self,
        parameter_count: u16,
        minimum_argument_count: u16,
    ) -> Self {
        self.header = self
            .header
            .with_parameter_counts(parameter_count, minimum_argument_count);
        self
    }

    #[inline]
    pub const fn with_register_counts(
        mut self,
        register_count: u16,
        hidden_register_count: u16,
    ) -> Self {
        self.header = self
            .header
            .with_register_counts(register_count, hidden_register_count);
        self
    }

    #[inline]
    pub const fn with_environment_layout(
        mut self,
        environment_layout: Option<EnvironmentLayoutRef>,
    ) -> Self {
        self.header = self.header.with_environment_layout(environment_layout);
        self
    }

    #[inline]
    pub const fn with_needs_environment(mut self, needs_environment: bool) -> Self {
        self.header = self.header.with_needs_environment(needs_environment);
        self
    }

    #[inline]
    pub const fn with_environment_slot_count(mut self, environment_slot_count: u16) -> Self {
        self.header = self
            .header
            .with_environment_slot_count(environment_slot_count);
        self
    }

    #[inline]
    pub fn with_environment_bindings(
        mut self,
        environment_bindings: Vec<BytecodeEnvironmentBinding>,
    ) -> Self {
        self.header = self.header.with_environment_slot_count(
            u16::try_from(environment_bindings.len()).unwrap_or(u16::MAX),
        );
        self.environment_bindings = environment_bindings;
        self
    }

    #[inline]
    pub const fn with_has_rest_parameter(mut self, has_rest_parameter: bool) -> Self {
        self.header = self.header.with_has_rest_parameter(has_rest_parameter);
        self
    }

    #[inline]
    pub fn with_instructions(mut self, instructions: Vec<Instruction>) -> Self {
        self.instructions = instructions;
        self
    }

    #[inline]
    pub fn with_constants(mut self, constants: Vec<ConstantValue>) -> Self {
        self.constants = constants;
        self
    }

    #[inline]
    pub fn with_child_functions(mut self, child_functions: Vec<BytecodeFunctionId>) -> Self {
        self.child_functions = child_functions;
        self
    }

    #[inline]
    pub fn with_captures(mut self, captures: Vec<CaptureDescriptor>) -> Self {
        self.captures = captures;
        self
    }

    #[inline]
    pub fn with_direct_eval_lexical_sites(
        mut self,
        direct_eval_lexical_sites: Vec<DirectEvalLexicalSite>,
    ) -> Self {
        self.direct_eval_lexical_sites = direct_eval_lexical_sites;
        self
    }

    #[inline]
    pub fn with_loop_iteration_environment_sites(
        mut self,
        loop_iteration_environment_sites: Vec<LoopIterationEnvironmentSite>,
    ) -> Self {
        self.loop_iteration_environment_sites = loop_iteration_environment_sites;
        self
    }

    #[inline]
    pub fn with_exception_handlers(mut self, exception_handlers: Vec<ExceptionHandler>) -> Self {
        self.exception_handlers = exception_handlers;
        self
    }

    #[inline]
    pub fn with_feedback_sites(mut self, feedback_sites: Vec<FeedbackSiteDescriptor>) -> Self {
        self.feedback_sites = feedback_sites;
        self
    }

    #[inline]
    pub fn with_source_map(mut self, source_map: Vec<SourceMapEntry>) -> Self {
        self.source_map = source_map;
        self
    }

    #[inline]
    pub fn with_wide_operands(mut self, wide_operands: Vec<WideOperand>) -> Self {
        self.wide_operands = wide_operands;
        self
    }

    #[inline]
    pub fn with_safepoints(mut self, safepoints: Vec<SafepointDescriptor>) -> Self {
        self.safepoints = safepoints;
        self
    }

    #[inline]
    pub fn with_safepoint_environment_layout(
        mut self,
        environment_layout: Option<EnvironmentLayoutRef>,
    ) -> Self {
        self.safepoints = self
            .safepoints
            .into_iter()
            .map(|descriptor| descriptor.with_environment_layout(environment_layout))
            .collect();
        self
    }

    #[inline]
    pub fn with_deopt_snapshots(mut self, deopt_snapshots: Vec<DeoptSnapshot>) -> Self {
        self.deopt_snapshots = deopt_snapshots;
        self
    }
}

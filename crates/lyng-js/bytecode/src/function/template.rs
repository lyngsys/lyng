use super::binding::BytecodeEnvironmentBinding;
use super::header::BytecodeFunctionHeader;
use crate::decoder::decode_instruction_bytes;
use crate::ids::{BytecodeFunctionId, EnvironmentLayoutRef};
use crate::instruction::{Instruction, INSTRUCTION_WIDTH};
use crate::metadata::{
    ArgumentsMode, BytecodeFunctionFlags, BytecodeFunctionKind, CaptureDescriptor, ConstantValue,
    DeoptSnapshot, DirectEvalLexicalSite, ExceptionHandler, FeedbackSiteDescriptor,
    LoopIterationEnvironmentSite, SafepointDescriptor, SourceMapEntry, ThisMode,
};
use lyng_js_common::{AtomId, Span};
use std::fmt;

/// Immutable bytecode template shared by the compiler and runtime installation layers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BytecodeFunction {
    header: BytecodeFunctionHeader,
    environment_bindings: Vec<BytecodeEnvironmentBinding>,
    instructions: Vec<u8>,
    constants: Vec<ConstantValue>,
    child_functions: Vec<BytecodeFunctionId>,
    captures: Vec<CaptureDescriptor>,
    direct_eval_lexical_sites: Vec<DirectEvalLexicalSite>,
    loop_iteration_environment_sites: Vec<LoopIterationEnvironmentSite>,
    exception_handlers: Vec<ExceptionHandler>,
    feedback_sites: Vec<FeedbackSiteDescriptor>,
    source_map: Vec<SourceMapEntry>,
    safepoints: Vec<SafepointDescriptor>,
    deopt_snapshots: Vec<DeoptSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BytecodeFunctionBody {
    pub(crate) environment_bindings: Vec<BytecodeEnvironmentBinding>,
    pub(crate) instructions: Vec<u8>,
    pub(crate) constants: Vec<ConstantValue>,
    pub(crate) child_functions: Vec<BytecodeFunctionId>,
    pub(crate) captures: Vec<CaptureDescriptor>,
    pub(crate) direct_eval_lexical_sites: Vec<DirectEvalLexicalSite>,
    pub(crate) loop_iteration_environment_sites: Vec<LoopIterationEnvironmentSite>,
    pub(crate) exception_handlers: Vec<ExceptionHandler>,
    pub(crate) feedback_sites: Vec<FeedbackSiteDescriptor>,
    pub(crate) source_map: Vec<SourceMapEntry>,
    pub(crate) safepoints: Vec<SafepointDescriptor>,
    pub(crate) deopt_snapshots: Vec<DeoptSnapshot>,
}

#[derive(Clone, Copy)]
pub struct InstructionStream<'a> {
    bytes: &'a [u8],
}

impl<'a> InstructionStream<'a> {
    #[inline]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    #[inline]
    pub fn len(self) -> usize {
        self.iter().count()
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.bytes.is_empty()
    }

    #[inline]
    pub fn get(self, index: usize) -> Option<Instruction> {
        self.iter().nth(index)
    }

    #[inline]
    pub const fn bytes(self) -> &'a [u8] {
        self.bytes
    }

    #[inline]
    pub const fn iter(self) -> InstructionIter<'a> {
        InstructionIter {
            bytes: self.bytes,
            byte_offset: 0,
        }
    }

    #[inline]
    pub const fn byte_offsets(self) -> InstructionByteOffsetIter<'a> {
        InstructionByteOffsetIter {
            bytes: self.bytes,
            byte_offset: 0,
        }
    }

    #[inline]
    pub fn to_vec(self) -> Vec<Instruction> {
        self.iter().collect()
    }
}

impl fmt::Debug for InstructionStream<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_list().entries(self.iter()).finish()
    }
}

impl PartialEq<&[Instruction]> for InstructionStream<'_> {
    fn eq(&self, other: &&[Instruction]) -> bool {
        self.iter().eq(other.iter().copied())
    }
}

impl<const N: usize> PartialEq<&[Instruction; N]> for InstructionStream<'_> {
    fn eq(&self, other: &&[Instruction; N]) -> bool {
        self.iter().eq(other.iter().copied())
    }
}

impl<'a> IntoIterator for InstructionStream<'a> {
    type IntoIter = InstructionIter<'a>;
    type Item = Instruction;

    fn into_iter(self) -> Self::IntoIter {
        InstructionIter {
            bytes: self.bytes,
            byte_offset: 0,
        }
    }
}

pub struct InstructionIter<'a> {
    bytes: &'a [u8],
    byte_offset: usize,
}

impl Iterator for InstructionIter<'_> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        let instruction = decode_instruction_bytes(self.bytes.get(self.byte_offset..)?).ok()?;
        self.byte_offset = self.byte_offset.checked_add(instruction.encoded_len())?;
        Some(instruction)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.bytes.len().saturating_sub(self.byte_offset)))
    }
}

pub struct InstructionByteOffsetIter<'a> {
    bytes: &'a [u8],
    byte_offset: usize,
}

impl Iterator for InstructionByteOffsetIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.byte_offset;
        let instruction = decode_instruction_bytes(self.bytes.get(offset..)?).ok()?;
        self.byte_offset = self.byte_offset.checked_add(instruction.encoded_len())?;
        Some(offset)
    }
}

fn encode_instructions(instructions: impl IntoIterator<Item = Instruction>) -> Vec<u8> {
    let iter = instructions.into_iter();
    let mut bytes = Vec::with_capacity(iter.size_hint().0.saturating_mul(INSTRUCTION_WIDTH));
    for instruction in iter {
        instruction.write_bytes(&mut bytes);
    }
    bytes
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
    pub fn instructions(&self) -> InstructionStream<'_> {
        InstructionStream::new(&self.instructions)
    }

    #[inline]
    pub fn instruction_bytes(&self) -> &[u8] {
        &self.instructions
    }

    #[inline]
    pub fn instruction_count(&self) -> usize {
        self.instructions().len()
    }

    #[inline]
    pub fn instruction_at(&self, instruction_offset: u32) -> Option<Instruction> {
        decode_instruction_bytes(
            self.instructions
                .get(usize::try_from(instruction_offset).ok()?..)?,
        )
        .ok()
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
        self.instructions = encode_instructions(instructions);
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

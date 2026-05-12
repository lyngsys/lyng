use crate::decoder::decode_instruction_bytes;
use crate::function::{
    BytecodeEnvironmentBinding, BytecodeFunction, BytecodeFunctionBody, BytecodeFunctionHeader,
    InstructionStream,
};
use crate::ids::{BytecodeFunctionId, EnvironmentLayoutRef};
use crate::instruction::{Instruction, INSTRUCTION_WIDTH};
use crate::metadata::{
    ArgumentsMode, BytecodeFunctionFlags, BytecodeFunctionKind, CallRange, CaptureDescriptor,
    ConstantValue, DeoptSnapshot, DirectEvalLexicalScope, DirectEvalLexicalSite,
    DirectEvalSiteFlags, ExceptionHandler, FeedbackSiteDescriptor, FeedbackSiteKind,
    FeedbackSiteMetadata, LoopIterationEnvironmentSite, SafepointDescriptor, SafepointKind,
    SourceMapEntry, ThisMode, WideAbcOperands, WideAbxOperands, WideOperand,
};
use crate::Opcode;
use lyng_js_common::{AtomId, SourceId, Span};
use lyng_js_types::FeedbackSlotId;

mod peephole;

pub type BytecodeBuildResult<T> = Result<T, BytecodeBuildError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BytecodeBuildError {
    LimitExceeded {
        kind: BytecodeLimitKind,
    },
    OperandOverflow {
        kind: BytecodeOperandKind,
    },
    InvalidJumpOpcode {
        opcode: Opcode,
    },
    InvalidJumpPatch {
        instruction_offset: u32,
    },
    JumpDeltaOverflow {
        instruction_offset: u32,
        target_offset: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BytecodeLimitKind {
    RegisterWindow,
    InstructionStream,
    ConstantPool,
    ChildFunctionTable,
    CaptureTable,
    ExceptionHandlerTable,
    FeedbackSlot,
    SafepointId,
    FinalRegisterWindow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BytecodeOperandKind {
    A,
    B,
    C,
    Bx,
    CallResult,
    CallCallee,
    CallThis,
    CallBase,
    TailCallCallee,
    TailCallThis,
    ConstructResult,
    ConstructCallee,
    JumpCondition,
}

fn operand_u16<T>(value: T, kind: BytecodeOperandKind) -> BytecodeBuildResult<u16>
where
    T: TryInto<u16>,
{
    value
        .try_into()
        .map_err(|_| BytecodeBuildError::OperandOverflow { kind })
}

fn operand_u32<T>(value: T, kind: BytecodeOperandKind) -> BytecodeBuildResult<u32>
where
    T: TryInto<u32>,
{
    value
        .try_into()
        .map_err(|_| BytecodeBuildError::OperandOverflow { kind })
}

fn narrow_call_operand(register: u16, kind: BytecodeOperandKind) -> BytecodeBuildResult<u8> {
    u8::try_from(register).map_err(|_| BytecodeBuildError::OperandOverflow { kind })
}

const fn signed_i8_fits(value: i32) -> bool {
    value >= i8::MIN as i32 && value <= i8::MAX as i32
}

const fn load_local_opcode(register: u16) -> Option<Opcode> {
    match register {
        0 => Some(Opcode::LoadLocal0),
        1 => Some(Opcode::LoadLocal1),
        2 => Some(Opcode::LoadLocal2),
        3 => Some(Opcode::LoadLocal3),
        _ => None,
    }
}

const fn store_local_opcode(register: u16) -> Option<Opcode> {
    match register {
        0 => Some(Opcode::StoreLocal0),
        1 => Some(Opcode::StoreLocal1),
        2 => Some(Opcode::StoreLocal2),
        3 => Some(Opcode::StoreLocal3),
        _ => None,
    }
}

fn compact_move_instruction(operands: WideAbcOperands) -> Option<Instruction> {
    if operands.needs_wide() || operands.c() != 0 {
        return None;
    }
    if let Some(opcode) = store_local_opcode(operands.a()) {
        return Some(Instruction::abx(opcode, operands.narrow_b(), 0));
    }
    load_local_opcode(operands.b()).map(|opcode| Instruction::abx(opcode, operands.narrow_a(), 0))
}

fn validate_short_abx_operands(
    operands: WideAbxOperands,
    bx_kind: BytecodeOperandKind,
) -> BytecodeBuildResult<(u8, u8)> {
    let a = u8::try_from(operands.a()).map_err(|_| BytecodeBuildError::OperandOverflow {
        kind: BytecodeOperandKind::A,
    })?;
    let bx = u8::try_from(operands.bx())
        .map_err(|_| BytecodeBuildError::OperandOverflow { kind: bx_kind })?;
    Ok((a, bx))
}

fn compact_abx_instruction(
    opcode: Opcode,
    operands: WideAbxOperands,
) -> BytecodeBuildResult<Option<Instruction>> {
    match opcode {
        Opcode::LoadSmi8 | Opcode::LoadConst8 | Opcode::JumpIfTrue8 | Opcode::JumpIfFalse8 => {
            let (a, bx) = validate_short_abx_operands(operands, BytecodeOperandKind::Bx)?;
            Ok(Some(Instruction::abx(opcode, a, u16::from(bx))))
        }
        Opcode::LoadLocal0
        | Opcode::LoadLocal1
        | Opcode::LoadLocal2
        | Opcode::LoadLocal3
        | Opcode::StoreLocal0
        | Opcode::StoreLocal1
        | Opcode::StoreLocal2
        | Opcode::StoreLocal3 => {
            let a =
                u8::try_from(operands.a()).map_err(|_| BytecodeBuildError::OperandOverflow {
                    kind: BytecodeOperandKind::A,
                })?;
            Ok(Some(Instruction::abx(opcode, a, 0)))
        }
        Opcode::LoadConst if !operands.needs_wide() && u8::try_from(operands.bx()).is_ok() => {
            Ok(Some(Instruction::abx(
                Opcode::LoadConst8,
                operands.narrow_a(),
                operands.narrow_bx(),
            )))
        }
        Opcode::LoadSmi if !operands.needs_wide() => {
            let raw = operands.narrow_bx();
            let value = i16::from_le_bytes(raw.to_le_bytes());
            Ok(i8::try_from(value).ok().map(|value| {
                Instruction::abx(
                    Opcode::LoadSmi8,
                    operands.narrow_a(),
                    u16::from(value.cast_unsigned()),
                )
            }))
        }
        _ => Ok(None),
    }
}

const fn short_conditional_jump_opcode(opcode: Opcode) -> Option<Opcode> {
    match opcode {
        Opcode::JumpIfTrue | Opcode::JumpIfTrue8 => Some(Opcode::JumpIfTrue8),
        Opcode::JumpIfFalse | Opcode::JumpIfFalse8 => Some(Opcode::JumpIfFalse8),
        _ => None,
    }
}

const fn full_conditional_jump_opcode(opcode: Opcode) -> Option<Opcode> {
    match opcode {
        Opcode::JumpIfTrue | Opcode::JumpIfTrue8 => Some(Opcode::JumpIfTrue),
        Opcode::JumpIfFalse | Opcode::JumpIfFalse8 => Some(Opcode::JumpIfFalse),
        _ => None,
    }
}

/// Incremental builder for one immutable [`BytecodeFunction`].
pub struct BytecodeBuilder {
    header: BytecodeFunctionHeader,
    environment_bindings: Vec<BytecodeEnvironmentBinding>,
    instructions: Vec<u8>,
    instruction_byte_offsets: Vec<usize>,
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
    next_feedback_slot: u32,
    next_safepoint_id: u32,
}

impl BytecodeBuilder {
    #[inline]
    pub const fn new(id: BytecodeFunctionId, kind: BytecodeFunctionKind) -> Self {
        Self {
            header: BytecodeFunctionHeader::new(
                id,
                kind,
                None,
                ThisMode::Global,
                ArgumentsMode::None,
            ),
            environment_bindings: Vec::new(),
            instructions: Vec::new(),
            instruction_byte_offsets: Vec::new(),
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
            next_feedback_slot: 1,
            next_safepoint_id: 1,
        }
    }

    #[inline]
    pub fn for_function(
        id: BytecodeFunctionId,
        name: Option<AtomId>,
        arguments_mode: ArgumentsMode,
    ) -> Self {
        let mut builder = Self::new(id, BytecodeFunctionKind::Function);
        builder.header = builder.header.with_flags(BytecodeFunctionFlags::default());
        builder.header = BytecodeFunctionHeader::new(
            id,
            BytecodeFunctionKind::Function,
            name,
            ThisMode::Global,
            arguments_mode,
        );
        builder
    }

    #[inline]
    pub const fn header(&self) -> BytecodeFunctionHeader {
        self.header
    }

    #[inline]
    pub const fn set_name(&mut self, name: Option<AtomId>) {
        self.header = BytecodeFunctionHeader::new(
            self.header.id(),
            self.header.kind(),
            name,
            self.header.this_mode(),
            self.header.arguments_mode(),
        )
        .with_flags(self.header.flags())
        .with_parameter_counts(
            self.header.parameter_count(),
            self.header.minimum_argument_count(),
        )
        .with_parameter_initializer_end_offset(self.header.parameter_initializer_end_offset())
        .with_register_counts(
            self.header.register_count(),
            self.header.hidden_register_count(),
        )
        .with_needs_environment(self.header.needs_environment())
        .with_environment_slot_count(self.header.environment_slot_count())
        .with_has_rest_parameter(self.header.has_rest_parameter())
        .with_environment_layout(self.header.environment_layout())
        .with_source_span(self.header.source_span());
    }

    #[inline]
    pub const fn set_flags(&mut self, flags: BytecodeFunctionFlags) {
        self.header = self.header.with_flags(flags);
    }

    #[inline]
    pub const fn set_this_mode(&mut self, this_mode: ThisMode) {
        self.header = BytecodeFunctionHeader::new(
            self.header.id(),
            self.header.kind(),
            self.header.name(),
            this_mode,
            self.header.arguments_mode(),
        )
        .with_flags(self.header.flags())
        .with_parameter_counts(
            self.header.parameter_count(),
            self.header.minimum_argument_count(),
        )
        .with_parameter_initializer_end_offset(self.header.parameter_initializer_end_offset())
        .with_register_counts(
            self.header.register_count(),
            self.header.hidden_register_count(),
        )
        .with_needs_environment(self.header.needs_environment())
        .with_environment_slot_count(self.header.environment_slot_count())
        .with_has_rest_parameter(self.header.has_rest_parameter())
        .with_environment_layout(self.header.environment_layout())
        .with_source_span(self.header.source_span());
    }

    #[inline]
    pub const fn set_arguments_mode(&mut self, arguments_mode: ArgumentsMode) {
        self.header = BytecodeFunctionHeader::new(
            self.header.id(),
            self.header.kind(),
            self.header.name(),
            self.header.this_mode(),
            arguments_mode,
        )
        .with_flags(self.header.flags())
        .with_parameter_counts(
            self.header.parameter_count(),
            self.header.minimum_argument_count(),
        )
        .with_parameter_initializer_end_offset(self.header.parameter_initializer_end_offset())
        .with_register_counts(
            self.header.register_count(),
            self.header.hidden_register_count(),
        )
        .with_needs_environment(self.header.needs_environment())
        .with_environment_slot_count(self.header.environment_slot_count())
        .with_has_rest_parameter(self.header.has_rest_parameter())
        .with_environment_layout(self.header.environment_layout())
        .with_source_span(self.header.source_span());
    }

    #[inline]
    pub const fn set_parameter_counts(
        &mut self,
        parameter_count: u16,
        minimum_argument_count: u16,
    ) {
        self.header = self
            .header
            .with_parameter_counts(parameter_count, minimum_argument_count);
    }

    #[inline]
    pub const fn set_parameter_initializer_end_offset(
        &mut self,
        parameter_initializer_end_offset: u32,
    ) {
        self.header = self
            .header
            .with_parameter_initializer_end_offset(parameter_initializer_end_offset);
    }

    #[inline]
    pub const fn set_environment_layout(
        &mut self,
        environment_layout: Option<EnvironmentLayoutRef>,
    ) {
        self.header = self.header.with_environment_layout(environment_layout);
    }

    #[inline]
    pub const fn set_source_span(&mut self, source_span: Option<Span>) {
        self.header = self.header.with_source_span(source_span);
    }

    #[inline]
    pub fn set_environment_bindings(
        &mut self,
        environment_bindings: Vec<BytecodeEnvironmentBinding>,
    ) {
        self.header = self.header.with_environment_slot_count(
            u16::try_from(environment_bindings.len()).unwrap_or(u16::MAX),
        );
        self.environment_bindings = environment_bindings;
    }

    #[inline]
    pub fn try_alloc_register(&mut self) -> Option<u16> {
        let register = self.header.register_count();
        let next = register.checked_add(1)?;
        self.header = self
            .header
            .with_register_counts(next, self.header.hidden_register_count());
        Some(register)
    }

    #[inline]
    /// Allocate one visible VM register.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::LimitExceeded`] when the register window is full.
    pub fn alloc_register(&mut self) -> BytecodeBuildResult<u16> {
        self.try_alloc_register()
            .ok_or(BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::RegisterWindow,
            })
    }

    #[inline]
    pub fn try_alloc_registers(&mut self, count: u16) -> Option<u16> {
        let first = self.header.register_count();
        let next = self.header.register_count().checked_add(count)?;
        self.header = self
            .header
            .with_register_counts(next, self.header.hidden_register_count());
        Some(first)
    }

    #[inline]
    /// Allocate a contiguous visible register range.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::LimitExceeded`] when the range would overflow the register
    /// window.
    pub fn alloc_registers(&mut self, count: u16) -> BytecodeBuildResult<u16> {
        self.try_alloc_registers(count)
            .ok_or(BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::RegisterWindow,
            })
    }

    #[inline]
    pub const fn set_hidden_register_count(&mut self, hidden_register_count: u16) {
        self.header = self
            .header
            .with_register_counts(self.header.register_count(), hidden_register_count);
    }

    #[inline]
    pub const fn set_environment_slot_count(&mut self, environment_slot_count: u16) {
        self.header = self
            .header
            .with_environment_slot_count(environment_slot_count);
    }

    #[inline]
    pub const fn set_has_rest_parameter(&mut self, has_rest_parameter: bool) {
        self.header = self.header.with_has_rest_parameter(has_rest_parameter);
    }

    #[inline]
    pub const fn set_needs_environment(&mut self, needs_environment: bool) {
        self.header = self.header.with_needs_environment(needs_environment);
    }

    #[inline]
    /// Return the instruction offset that the next emit will use.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::LimitExceeded`] when the instruction stream length no longer
    /// fits in the serialized offset width.
    pub fn current_offset(&self) -> BytecodeBuildResult<u32> {
        u32::try_from(self.instruction_count()).map_err(|_| BytecodeBuildError::LimitExceeded {
            kind: BytecodeLimitKind::InstructionStream,
        })
    }

    #[inline]
    const fn instruction_count(&self) -> usize {
        self.instruction_byte_offsets.len()
    }

    #[inline]
    fn instruction_at(&self, instruction_offset: u32) -> Option<Instruction> {
        let offset = usize::try_from(instruction_offset).ok()?;
        let start = *self.instruction_byte_offsets.get(offset)?;
        decode_instruction_bytes(self.instructions.get(start..)?).ok()
    }

    #[inline]
    fn replace_instruction(
        &mut self,
        instruction_offset: u32,
        instruction: Instruction,
    ) -> BytecodeBuildResult<()> {
        let offset = usize::try_from(instruction_offset)
            .map_err(|_| BytecodeBuildError::InvalidJumpPatch { instruction_offset })?;
        let start = *self
            .instruction_byte_offsets
            .get(offset)
            .ok_or(BytecodeBuildError::InvalidJumpPatch { instruction_offset })?;
        let old_len = self
            .instruction_byte_offsets
            .get(offset.saturating_add(1))
            .copied()
            .unwrap_or(self.instructions.len())
            .checked_sub(start)
            .ok_or(BytecodeBuildError::InvalidJumpPatch { instruction_offset })?;
        let new_bytes = instruction.encode_bytes();
        self.instructions.splice(
            start..start.saturating_add(old_len),
            new_bytes.iter().copied(),
        );
        match new_bytes.len().cmp(&old_len) {
            std::cmp::Ordering::Less => {
                let shrink = old_len - new_bytes.len();
                for byte_offset in self.instruction_byte_offsets.iter_mut().skip(offset + 1) {
                    *byte_offset -= shrink;
                }
            }
            std::cmp::Ordering::Greater => {
                let growth = new_bytes.len() - old_len;
                for byte_offset in self.instruction_byte_offsets.iter_mut().skip(offset + 1) {
                    *byte_offset += growth;
                }
            }
            std::cmp::Ordering::Equal => {}
        }
        Ok(())
    }

    #[inline]
    fn push_instruction(&mut self, instruction: Instruction) {
        self.instruction_byte_offsets.push(self.instructions.len());
        instruction.write_bytes(&mut self.instructions);
    }

    #[inline]
    pub(super) fn decoded_instructions(&self) -> Vec<Instruction> {
        InstructionStream::new(&self.instructions).to_vec()
    }

    #[inline]
    pub(super) fn replace_decoded_instructions(&mut self, instructions: Vec<Instruction>) {
        self.instructions.clear();
        self.instruction_byte_offsets.clear();
        self.instructions
            .reserve(instructions.len().saturating_mul(INSTRUCTION_WIDTH));
        self.instruction_byte_offsets.reserve(instructions.len());
        for instruction in instructions {
            self.push_instruction(instruction);
        }
    }

    #[inline]
    fn wide_payload(&self, instruction_offset: u32) -> Option<u32> {
        self.wide_operands.iter().find_map(|operand| {
            (operand.instruction_offset() == instruction_offset).then_some(operand.payload())
        })
    }

    #[inline]
    fn set_wide_operand(&mut self, instruction_offset: u32, payload: u32) {
        if let Some(operand) = self
            .wide_operands
            .iter_mut()
            .find(|operand| operand.instruction_offset() == instruction_offset)
        {
            *operand = WideOperand::new(instruction_offset, payload);
            return;
        }
        self.wide_operands
            .push(WideOperand::new(instruction_offset, payload));
    }

    #[inline]
    fn remove_wide_operand(&mut self, instruction_offset: u32) {
        self.wide_operands
            .retain(|operand| operand.instruction_offset() != instruction_offset);
    }

    #[inline]
    fn decode_abx_operands(&self, instruction_offset: u32, a: u8, bx: u16) -> WideAbxOperands {
        self.wide_payload(instruction_offset).map_or_else(
            || WideAbxOperands::narrow(a, bx),
            |payload| WideAbxOperands::decode(a, bx, payload),
        )
    }

    #[inline]
    /// Append one fully encoded instruction to the function body.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::LimitExceeded`] when the current instruction offset cannot be
    /// represented.
    pub fn emit(&mut self, instruction: Instruction) -> BytecodeBuildResult<u32> {
        let offset = self.current_offset()?;
        self.push_instruction(instruction);
        Ok(offset)
    }

    #[inline]
    /// Append an ABC-form instruction, recording a wide payload when any operand is out of range.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::OperandOverflow`] when an operand cannot be represented in the
    /// wide operand format, or [`BytecodeBuildError::LimitExceeded`] when the instruction offset is
    /// too large.
    pub fn emit_abc<A, B, C>(
        &mut self,
        opcode: Opcode,
        a: A,
        b: B,
        c: C,
    ) -> BytecodeBuildResult<u32>
    where
        A: TryInto<u16>,
        B: TryInto<u16>,
        C: TryInto<u16>,
    {
        let operands = WideAbcOperands::new(
            operand_u16(a, BytecodeOperandKind::A)?,
            operand_u16(b, BytecodeOperandKind::B)?,
            operand_u16(c, BytecodeOperandKind::C)?,
        );
        if opcode == Opcode::Move
            && let Some(instruction) = compact_move_instruction(operands)
        {
            return self.emit(instruction);
        }
        let offset = self.emit(Instruction::abc(
            opcode,
            operands.narrow_a(),
            operands.narrow_b(),
            operands.narrow_c(),
        ))?;
        if operands.needs_wide() {
            self.add_wide_operand(offset, operands.encode_payload());
        }
        Ok(offset)
    }

    #[inline]
    /// Append an ABx-form instruction, recording a wide payload when either operand is out of range.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::OperandOverflow`] when an operand cannot be represented in the
    /// wide operand format, or [`BytecodeBuildError::LimitExceeded`] when the instruction offset is
    /// too large.
    pub fn emit_abx<A, B>(&mut self, opcode: Opcode, a: A, bx: B) -> BytecodeBuildResult<u32>
    where
        A: TryInto<u16>,
        B: TryInto<u32>,
    {
        let operands = WideAbxOperands::new(
            operand_u16(a, BytecodeOperandKind::A)?,
            operand_u32(bx, BytecodeOperandKind::Bx)?,
        );
        if let Some(instruction) = compact_abx_instruction(opcode, operands)? {
            return self.emit(instruction);
        }
        let offset = self.emit(Instruction::abx(
            opcode,
            operands.narrow_a(),
            operands.narrow_bx(),
        ))?;
        if operands.needs_wide() {
            self.add_wide_operand(offset, operands.encode_payload());
        }
        Ok(offset)
    }

    #[inline]
    /// Append an AX-form instruction.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::LimitExceeded`] when the instruction offset is too large.
    pub fn emit_ax(&mut self, opcode: Opcode, ax: i32) -> BytecodeBuildResult<u32> {
        match opcode {
            Opcode::Jump if signed_i8_fits(ax) => self.emit(Instruction::ax(Opcode::Jump8, ax)),
            Opcode::Jump8 if signed_i8_fits(ax) => self.emit(Instruction::ax(opcode, ax)),
            Opcode::Jump8 => Err(BytecodeBuildError::OperandOverflow {
                kind: BytecodeOperandKind::Bx,
            }),
            _ => self.emit(Instruction::ax(opcode, ax)),
        }
    }

    #[inline]
    /// Append a jump instruction whose target will be patched later.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::InvalidJumpOpcode`] when `opcode` is not a jump opcode, or
    /// [`BytecodeBuildError::LimitExceeded`] when the instruction offset is too large.
    pub fn emit_jump_placeholder(&mut self, opcode: Opcode) -> BytecodeBuildResult<u32> {
        if !opcode.is_jump() {
            return Err(BytecodeBuildError::InvalidJumpOpcode { opcode });
        }
        self.emit_ax(opcode, 0)
    }

    #[inline]
    /// Append a conditional jump instruction whose target will be patched later.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::InvalidJumpOpcode`] when `opcode` is not a supported
    /// conditional jump, [`BytecodeBuildError::OperandOverflow`] when the condition register is too
    /// wide, or [`BytecodeBuildError::LimitExceeded`] when the instruction offset is too large.
    pub fn emit_cond_jump_placeholder<A>(
        &mut self,
        opcode: Opcode,
        condition: A,
    ) -> BytecodeBuildResult<u32>
    where
        A: TryInto<u16>,
    {
        if !matches!(
            opcode,
            Opcode::JumpIfTrue | Opcode::JumpIfFalse | Opcode::JumpIfTrue8 | Opcode::JumpIfFalse8
        ) {
            return Err(BytecodeBuildError::InvalidJumpOpcode { opcode });
        }
        self.emit_abx(
            opcode,
            operand_u16(condition, BytecodeOperandKind::JumpCondition)?,
            0,
        )
    }

    #[inline]
    /// Append a call instruction and its argument range side payload.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::OperandOverflow`] when one of the fixed call registers cannot
    /// fit in the narrow call fields, or [`BytecodeBuildError::LimitExceeded`] when the instruction
    /// offset is too large.
    pub fn emit_call(
        &mut self,
        result: u16,
        callee: u16,
        this_value: u16,
        arguments: CallRange,
    ) -> BytecodeBuildResult<u32> {
        let result = narrow_call_operand(result, BytecodeOperandKind::CallResult)?;
        let callee = narrow_call_operand(callee, BytecodeOperandKind::CallCallee)?;
        let this_value = narrow_call_operand(this_value, BytecodeOperandKind::CallThis)?;
        let instruction_offset =
            self.emit(Instruction::abc(Opcode::Call, result, callee, this_value))?;
        self.add_wide_operand(instruction_offset, arguments.encode());
        Ok(instruction_offset)
    }

    #[inline]
    /// Append a compact non-spread call instruction for zero through three arguments.
    ///
    /// The encoded call base register stores the receiver, followed by the argument registers.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::OperandOverflow`] when one of the fixed call registers cannot
    /// fit in the compact call fields, or [`BytecodeBuildError::LimitExceeded`] when the
    /// instruction offset is too large.
    pub fn emit_small_call(
        &mut self,
        result: u16,
        callee: u16,
        call_base: u16,
        argument_count: u8,
    ) -> BytecodeBuildResult<u32> {
        let opcode = match argument_count {
            0 => Opcode::Call0,
            1 => Opcode::Call1,
            2 => Opcode::Call2,
            3 => Opcode::Call3,
            _ => {
                return Err(BytecodeBuildError::OperandOverflow {
                    kind: BytecodeOperandKind::CallBase,
                });
            }
        };
        let result = narrow_call_operand(result, BytecodeOperandKind::CallResult)?;
        let callee = narrow_call_operand(callee, BytecodeOperandKind::CallCallee)?;
        let call_base = narrow_call_operand(call_base, BytecodeOperandKind::CallBase)?;
        self.emit(Instruction::abc(opcode, result, callee, call_base))
    }

    #[inline]
    /// Append a tail-call instruction and mark the function as tail-call capable.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::OperandOverflow`] when one of the fixed call registers cannot
    /// fit in the narrow call fields, or [`BytecodeBuildError::LimitExceeded`] when the instruction
    /// offset is too large.
    pub fn emit_tail_call(
        &mut self,
        callee: u16,
        this_value: u16,
        arguments: CallRange,
    ) -> BytecodeBuildResult<u32> {
        let callee = narrow_call_operand(callee, BytecodeOperandKind::TailCallCallee)?;
        let this_value = narrow_call_operand(this_value, BytecodeOperandKind::TailCallThis)?;
        self.header = self
            .header
            .with_flags(self.header.flags().with_tail_call_capable(true));
        let instruction_offset =
            self.emit(Instruction::abc(Opcode::TailCall, callee, this_value, 0))?;
        self.add_wide_operand(instruction_offset, arguments.encode());
        Ok(instruction_offset)
    }

    #[inline]
    /// Append a construct instruction and its argument range side payload.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::OperandOverflow`] when one of the fixed construct registers
    /// cannot fit in the narrow fields, or [`BytecodeBuildError::LimitExceeded`] when the
    /// instruction offset is too large.
    pub fn emit_construct(
        &mut self,
        result: u16,
        callee: u16,
        arguments: CallRange,
    ) -> BytecodeBuildResult<u32> {
        let result = narrow_call_operand(result, BytecodeOperandKind::ConstructResult)?;
        let callee = narrow_call_operand(callee, BytecodeOperandKind::ConstructCallee)?;
        let instruction_offset =
            self.emit(Instruction::abc(Opcode::Construct, result, callee, 0))?;
        self.add_wide_operand(instruction_offset, arguments.encode());
        Ok(instruction_offset)
    }

    #[inline]
    /// Patch a placeholder jump to branch to `target_offset`.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::InvalidJumpPatch`] when `instruction_offset` does not point to
    /// a jump instruction, or [`BytecodeBuildError::JumpDeltaOverflow`] when the relative branch
    /// delta cannot be encoded.
    pub fn patch_jump_to(
        &mut self,
        instruction_offset: u32,
        target_offset: u32,
    ) -> BytecodeBuildResult<()> {
        let existing = self
            .instruction_at(instruction_offset)
            .ok_or(BytecodeBuildError::InvalidJumpPatch { instruction_offset })?;
        if !existing.opcode().is_jump() {
            return Err(BytecodeBuildError::InvalidJumpPatch { instruction_offset });
        }
        let delta = i64::from(target_offset) - (i64::from(instruction_offset) + 1);
        let instruction = match existing {
            Instruction::Ax {
                opcode: Opcode::Jump | Opcode::Jump8,
                ..
            } => {
                let delta =
                    i32::try_from(delta).map_err(|_| BytecodeBuildError::JumpDeltaOverflow {
                        instruction_offset,
                        target_offset,
                    })?;
                if signed_i8_fits(delta) {
                    Instruction::ax(Opcode::Jump8, delta)
                } else {
                    Instruction::ax(Opcode::Jump, delta)
                }
            }
            Instruction::Abx { opcode, a, bx } if opcode.is_jump() => {
                let delta =
                    i32::try_from(delta).map_err(|_| BytecodeBuildError::JumpDeltaOverflow {
                        instruction_offset,
                        target_offset,
                    })?;
                let condition = if matches!(opcode, Opcode::JumpIfTrue8 | Opcode::JumpIfFalse8) {
                    u16::from(a)
                } else {
                    self.decode_abx_operands(instruction_offset, a, bx).a()
                };
                if signed_i8_fits(delta)
                    && let (Ok(condition), Ok(delta)) =
                        (u8::try_from(condition), i8::try_from(delta))
                {
                    self.remove_wide_operand(instruction_offset);
                    Instruction::abx(
                        short_conditional_jump_opcode(opcode)
                            .ok_or(BytecodeBuildError::InvalidJumpPatch { instruction_offset })?,
                        condition,
                        u16::from(delta.cast_unsigned()),
                    )
                } else {
                    let opcode = full_conditional_jump_opcode(opcode)
                        .ok_or(BytecodeBuildError::InvalidJumpPatch { instruction_offset })?;
                    let updated =
                        WideAbxOperands::new(condition, u32::from_le_bytes(delta.to_le_bytes()));
                    if updated.needs_wide() {
                        self.set_wide_operand(instruction_offset, updated.encode_payload());
                    } else {
                        self.remove_wide_operand(instruction_offset);
                    }
                    Instruction::abx(opcode, updated.narrow_a(), updated.narrow_bx())
                }
            }
            _ => {
                return Err(BytecodeBuildError::InvalidJumpPatch { instruction_offset });
            }
        };
        self.replace_instruction(instruction_offset, instruction)?;
        Ok(())
    }

    #[inline]
    /// Append a constant-pool entry and return its index.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::LimitExceeded`] when the constant-pool index cannot be encoded.
    pub fn add_constant(&mut self, constant: ConstantValue) -> BytecodeBuildResult<u32> {
        let index =
            u32::try_from(self.constants.len()).map_err(|_| BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::ConstantPool,
            })?;
        self.constants.push(constant);
        Ok(index)
    }

    #[inline]
    /// Append a child function reference and return its table index.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::LimitExceeded`] when the child-function table index cannot be
    /// encoded.
    pub fn add_child_function(&mut self, child: BytecodeFunctionId) -> BytecodeBuildResult<u16> {
        let index = u16::try_from(self.child_functions.len()).map_err(|_| {
            BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::ChildFunctionTable,
            }
        })?;
        self.child_functions.push(child);
        Ok(index)
    }

    #[inline]
    /// Append one capture descriptor and return its table index.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::LimitExceeded`] when the capture table index cannot be encoded.
    pub fn add_capture(&mut self, capture: CaptureDescriptor) -> BytecodeBuildResult<u16> {
        let index =
            u16::try_from(self.captures.len()).map_err(|_| BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::CaptureTable,
            })?;
        self.captures.push(capture);
        Ok(index)
    }

    #[inline]
    /// Append one exception-handler descriptor and return its table index.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::LimitExceeded`] when the exception-handler table index cannot
    /// be encoded.
    pub fn add_exception_handler(&mut self, handler: ExceptionHandler) -> BytecodeBuildResult<u16> {
        let index = u16::try_from(self.exception_handlers.len()).map_err(|_| {
            BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::ExceptionHandlerTable,
            }
        })?;
        self.exception_handlers.push(handler);
        Ok(index)
    }

    #[inline]
    pub fn add_loop_iteration_environment_site(
        &mut self,
        instruction_offset: u32,
        iteration_slots: Vec<u16>,
        shared_slots: Vec<u16>,
        detached_slots: Vec<u16>,
    ) {
        self.loop_iteration_environment_sites
            .push(LoopIterationEnvironmentSite::new(
                instruction_offset,
                iteration_slots,
                shared_slots,
                detached_slots,
            ));
    }

    #[inline]
    pub fn add_direct_eval_lexical_site(
        &mut self,
        instruction_offset: u32,
        scopes: Vec<DirectEvalLexicalScope>,
        flags: DirectEvalSiteFlags,
        annex_b_catch_names: Vec<lyng_js_common::AtomId>,
        parameter_names: Vec<lyng_js_common::AtomId>,
    ) {
        self.direct_eval_lexical_sites
            .push(DirectEvalLexicalSite::new(
                instruction_offset,
                scopes,
                flags,
                annex_b_catch_names,
                parameter_names,
            ));
    }

    #[inline]
    /// Append one feedback-site descriptor and allocate its feedback slot.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::LimitExceeded`] when the feedback slot id overflows.
    pub fn add_feedback_site(
        &mut self,
        instruction_offset: u32,
        kind: FeedbackSiteKind,
        metadata: FeedbackSiteMetadata,
    ) -> BytecodeBuildResult<FeedbackSlotId> {
        let next_feedback_slot =
            self.next_feedback_slot
                .checked_add(1)
                .ok_or(BytecodeBuildError::LimitExceeded {
                    kind: BytecodeLimitKind::FeedbackSlot,
                })?;
        let slot = FeedbackSlotId::from_raw(self.next_feedback_slot).ok_or(
            BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::FeedbackSlot,
            },
        )?;
        if u16::try_from(slot.get()).is_err() {
            return Err(BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::FeedbackSlot,
            });
        }
        self.next_feedback_slot = next_feedback_slot;
        if let Some(profiled_instruction) = self
            .instruction_at(instruction_offset)
            .and_then(|instruction| instruction.with_feedback_slot(slot))
        {
            self.replace_instruction(instruction_offset, profiled_instruction)?;
        }
        self.feedback_sites.push(
            FeedbackSiteDescriptor::new(slot, instruction_offset, kind).with_metadata(metadata),
        );
        Ok(slot)
    }

    #[inline]
    pub fn add_source_map_entry(
        &mut self,
        source: SourceId,
        instruction_offset: u32,
        start: u32,
        end: u32,
    ) {
        self.source_map
            .push(SourceMapEntry::new(source, instruction_offset, start, end));
    }

    #[inline]
    pub fn add_wide_operand(&mut self, instruction_offset: u32, payload: u32) {
        self.wide_operands
            .push(WideOperand::new(instruction_offset, payload));
    }

    #[inline]
    pub fn add_safepoint(&mut self, safepoint: SafepointDescriptor) {
        self.safepoints.push(safepoint);
    }

    #[inline]
    /// Append a safepoint descriptor at an instruction offset and allocate its id.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::LimitExceeded`] when the safepoint id overflows.
    pub fn add_safepoint_at(
        &mut self,
        instruction_offset: u32,
        kind: SafepointKind,
        register_window_len: u16,
    ) -> BytecodeBuildResult<u32> {
        let id = self.alloc_safepoint_id()?;
        self.safepoints.push(SafepointDescriptor::new(
            id,
            instruction_offset,
            kind,
            register_window_len,
        ));
        Ok(id)
    }

    #[inline]
    /// Allocate the next safepoint id without appending a descriptor.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::LimitExceeded`] when the safepoint id overflows.
    pub fn alloc_safepoint_id(&mut self) -> BytecodeBuildResult<u32> {
        let id = self.next_safepoint_id;
        self.next_safepoint_id =
            self.next_safepoint_id
                .checked_add(1)
                .ok_or(BytecodeBuildError::LimitExceeded {
                    kind: BytecodeLimitKind::SafepointId,
                })?;
        Ok(id)
    }

    #[inline]
    pub fn add_deopt_snapshot(&mut self, snapshot: DeoptSnapshot) {
        self.deopt_snapshots.push(snapshot);
    }

    #[inline]
    /// Finalize the immutable bytecode function.
    ///
    /// # Errors
    /// Returns [`BytecodeBuildError::LimitExceeded`] when the visible and hidden register counts do
    /// not fit in the final register window.
    pub fn finish(mut self) -> BytecodeBuildResult<BytecodeFunction> {
        peephole::optimize(&mut self)?;
        let final_register_window = self
            .header
            .register_count()
            .checked_add(self.header.hidden_register_count())
            .ok_or(BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::FinalRegisterWindow,
            })?;
        Ok(BytecodeFunction::from_parts(
            self.header,
            BytecodeFunctionBody {
                environment_bindings: self.environment_bindings,
                instructions: self.instructions,
                constants: self.constants,
                child_functions: self.child_functions,
                captures: self.captures,
                direct_eval_lexical_sites: self.direct_eval_lexical_sites,
                loop_iteration_environment_sites: self.loop_iteration_environment_sites,
                exception_handlers: self.exception_handlers,
                feedback_sites: self.feedback_sites,
                source_map: self.source_map,
                wide_operands: self.wide_operands,
                safepoints: self
                    .safepoints
                    .into_iter()
                    .map(|descriptor| descriptor.with_register_window_len(final_register_window))
                    .collect(),
                deopt_snapshots: self.deopt_snapshots,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{disassemble, CaptureSource, ExceptionHandlerKind};
    use std::num::NonZeroU32;

    #[test]
    fn builder_tracks_constants_jumps_handlers_and_children() -> BytecodeBuildResult<()> {
        let mut builder = BytecodeBuilder::for_function(
            BytecodeFunctionId::new(NonZeroU32::new(1).unwrap()),
            Some(AtomId::from_raw(41)),
            ArgumentsMode::Unmapped,
        );
        builder.set_flags(BytecodeFunctionFlags::new(true, false));
        builder.set_parameter_counts(1, 1);
        builder.alloc_registers(3)?;
        builder.set_hidden_register_count(1);

        let constant = builder.add_constant(ConstantValue::Smi(42))?;
        let load = builder.emit_abx(Opcode::LoadConst, 0, constant)?;
        let jump = builder.emit_jump_placeholder(Opcode::Jump)?;
        let _move_inst = builder.emit_abc(Opcode::Move, 1, 0, 0)?;
        let ret = builder.emit_ax(Opcode::Return, 1)?;
        builder.patch_jump_to(jump, ret)?;
        builder.add_child_function(BytecodeFunctionId::new(NonZeroU32::new(2).unwrap()))?;
        builder.add_exception_handler(ExceptionHandler::new(
            load,
            ret,
            ret,
            ExceptionHandlerKind::Catch,
            1,
            Some(2),
        ))?;

        let function = builder.finish()?;

        assert_eq!(function.constants(), &[ConstantValue::Smi(42)]);
        assert_eq!(
            function.child_functions(),
            &[BytecodeFunctionId::new(NonZeroU32::new(2).unwrap())]
        );
        assert_eq!(function.exception_handlers().len(), 1);
        assert_eq!(function.register_count(), 3);
        assert_eq!(function.hidden_register_count(), 1);
        assert_eq!(function.instructions().len(), 2);
        assert!(matches!(
            function.instructions().to_vec().as_slice(),
            [
                Instruction::Abx {
                    opcode: Opcode::LoadConst8,
                    ..
                },
                Instruction::Ax {
                    opcode: Opcode::Return,
                    ax: 1
                }
            ]
        ));
        assert_eq!(function.exception_handlers()[0].protected_start(), 0);
        assert_eq!(function.exception_handlers()[0].protected_end(), 1);
        assert_eq!(function.exception_handlers()[0].handler(), 1);
        Ok(())
    }

    #[test]
    fn builder_assigns_feedback_sites_and_metadata() -> BytecodeBuildResult<()> {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(9).unwrap()),
            BytecodeFunctionKind::Function,
        );
        let instruction = builder.emit_abc(Opcode::Add, 0, 1, 2)?;
        let slot = builder.add_feedback_site(
            instruction,
            FeedbackSiteKind::Arithmetic,
            FeedbackSiteMetadata::None,
        )?;
        let function = builder.finish()?;

        assert_eq!(slot, FeedbackSlotId::from_raw(1).unwrap());
        assert_eq!(function.feedback_sites().len(), 1);
        assert_eq!(
            function.feedback_sites()[0].instruction_offset(),
            instruction
        );
        Ok(())
    }

    #[test]
    fn builder_inlines_feedback_slot_operand_on_profiled_instructions() -> BytecodeBuildResult<()> {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(90).unwrap()),
            BytecodeFunctionKind::Function,
        );
        builder.alloc_registers(3)?;
        let add = builder.emit_abc(Opcode::Add, 0, 1, 2)?;
        let global = builder.emit_abx(Opcode::LoadGlobal, 0, 7)?;

        let add_slot = builder.add_feedback_site(
            add,
            FeedbackSiteKind::Arithmetic,
            FeedbackSiteMetadata::None,
        )?;
        let global_slot = builder.add_feedback_site(
            global,
            FeedbackSiteKind::NamedPropertyLoad,
            FeedbackSiteMetadata::NamedProperty(AtomId::from_raw(7)),
        )?;

        let function = builder.finish()?;

        assert_eq!(
            function.instructions().get(0),
            Some(Instruction::profiled_abc(Opcode::Add, 0, 1, 2, add_slot))
        );
        assert_eq!(
            function.instructions().get(1),
            Some(Instruction::profiled_abx(
                Opcode::LoadGlobal,
                0,
                7,
                global_slot
            ))
        );
        Ok(())
    }

    #[test]
    fn peephole_threads_jump_to_jump_targets_and_removes_unreachable_jump_chain(
    ) -> BytecodeBuildResult<()> {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(12).unwrap()),
            BytecodeFunctionKind::Function,
        );
        builder.alloc_registers(1)?;
        builder.emit_abx(Opcode::LoadTrue, 0, 0)?;
        let branch = builder.emit_cond_jump_placeholder(Opcode::JumpIfTrue, 0)?;
        builder.emit_ax(Opcode::ReturnUndefined, 0)?;
        let intermediate = builder.emit_jump_placeholder(Opcode::Jump)?;
        builder.emit_abx(Opcode::LoadOne, 0, 0)?;
        let ret = builder.emit_ax(Opcode::Return, 0)?;
        builder.patch_jump_to(branch, intermediate)?;
        builder.patch_jump_to(intermediate, ret)?;

        let function = builder.finish()?;

        assert_eq!(function.instructions().len(), 4);
        assert!(matches!(
            function.instructions().to_vec().as_slice(),
            [
                Instruction::Abx {
                    opcode: Opcode::LoadTrue,
                    ..
                },
                Instruction::Abx {
                    opcode: Opcode::JumpIfTrue8,
                    ..
                },
                Instruction::Ax {
                    opcode: Opcode::ReturnUndefined,
                    ..
                },
                Instruction::Ax {
                    opcode: Opcode::Return,
                    ..
                }
            ]
        ));
        assert_eq!(
            function
                .instructions()
                .get(1)
                .and_then(Instruction::bx_value),
            Some(1)
        );
        Ok(())
    }

    #[test]
    fn peephole_removes_noop_jump_and_remaps_offset_metadata() -> BytecodeBuildResult<()> {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(13).unwrap()),
            BytecodeFunctionKind::Function,
        );
        builder.alloc_registers(1)?;
        let jump = builder.emit_jump_placeholder(Opcode::Jump)?;
        let ret = builder.emit_ax(Opcode::ReturnUndefined, 0)?;
        builder.patch_jump_to(jump, ret)?;
        builder.add_source_map_entry(SourceId::new(1), ret, 10, 20);
        let slot = builder.add_feedback_site(
            ret,
            FeedbackSiteKind::Comparison,
            FeedbackSiteMetadata::None,
        )?;
        let safepoint = builder.add_safepoint_at(ret, SafepointKind::LoopBackedge, 1)?;
        builder.add_deopt_snapshot(DeoptSnapshot::new(
            safepoint,
            vec![crate::DeoptValueSource::Register(0)],
        ));

        let function = builder.finish()?;

        assert_eq!(
            function.instructions(),
            &[Instruction::ax(Opcode::ReturnUndefined, 0)]
        );
        assert_eq!(function.source_map()[0].instruction_offset(), 0);
        assert_eq!(function.feedback_sites()[0].slot(), slot);
        assert_eq!(function.feedback_sites()[0].instruction_offset(), 0);
        assert_eq!(function.safepoints()[0].id(), safepoint);
        assert_eq!(function.safepoints()[0].instruction_offset(), 0);
        assert_eq!(function.deopt_snapshots().len(), 1);
        assert_eq!(function.deopt_snapshots()[0].safepoint_id(), safepoint);
        Ok(())
    }

    #[test]
    fn peephole_removes_dead_code_after_terminal_control_flow() -> BytecodeBuildResult<()> {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(14).unwrap()),
            BytecodeFunctionKind::Function,
        );
        builder.alloc_registers(1)?;
        builder.emit_ax(Opcode::ReturnUndefined, 0)?;
        builder.emit_abx(Opcode::LoadOne, 0, 0)?;
        builder.emit_ax(Opcode::Return, 0)?;

        let function = builder.finish()?;

        assert_eq!(
            function.instructions(),
            &[Instruction::ax(Opcode::ReturnUndefined, 0)]
        );
        Ok(())
    }

    #[test]
    fn builder_records_source_maps_safepoints_and_deopt_snapshots() -> BytecodeBuildResult<()> {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(10).unwrap()),
            BytecodeFunctionKind::Function,
        );
        builder.alloc_registers(2)?;
        let call = builder.emit_tail_call(0, 1, CallRange::new(0, 1))?;
        builder.add_source_map_entry(SourceId::new(9), call, 4, 18);
        let safepoint = builder.add_safepoint_at(call, SafepointKind::Allocation, 2)?;
        builder.add_deopt_snapshot(DeoptSnapshot::new(
            safepoint,
            vec![crate::DeoptValueSource::FrameValue(
                crate::DeoptFrameValue::ThisValue,
            )],
        ));

        let function = builder.finish()?;

        assert_eq!(function.source_map().len(), 1);
        assert_eq!(function.safepoints().len(), 1);
        assert_eq!(function.deopt_snapshots().len(), 1);
        assert_eq!(function.safepoints()[0].id(), safepoint);
        assert_eq!(function.safepoints()[0].kind(), SafepointKind::Allocation);
        assert_eq!(function.deopt_snapshots()[0].safepoint_id(), safepoint);
        Ok(())
    }

    #[test]
    fn disassembly_is_stable_for_builder_output() -> BytecodeBuildResult<()> {
        let mut builder = BytecodeBuilder::for_function(
            BytecodeFunctionId::new(NonZeroU32::new(3).unwrap()),
            Some(AtomId::from_raw(19)),
            ArgumentsMode::None,
        );
        builder.alloc_registers(2)?;
        let constant = builder.add_constant(ConstantValue::Smi(7))?;
        builder.emit_abx(Opcode::LoadConst, 0, constant)?;
        builder.emit_ax(Opcode::Return, 0)?;
        let function = builder.finish()?;

        let text = disassemble(&function);

        assert_eq!(
            text,
            "function BytecodeFunctionId(3) kind=Function this=Global args=None params=0 min_args=0 regs=2 hidden=0 env=false env_slots=0 rest=false\n0000: LoadConst8      r0, const[0] ; Smi(7)\n0001: Return          r0\nconstants:\n  [0] Smi(7)\n"
        );
        Ok(())
    }

    #[test]
    fn disassembly_renders_explicit_tail_call_sites() -> BytecodeBuildResult<()> {
        let mut builder = BytecodeBuilder::for_function(
            BytecodeFunctionId::new(NonZeroU32::new(7).unwrap()),
            Some(AtomId::from_raw(23)),
            ArgumentsMode::None,
        );
        builder.alloc_registers(6)?;
        builder.emit_tail_call(2, 3, CallRange::new(4, 2))?;
        let function = builder.finish()?;

        let text = disassemble(&function);

        assert!(text.contains("TailCall        callee=r2, this=r3, args=[r4..r6)"));
        Ok(())
    }

    #[test]
    fn conditional_jump_patch_uses_signed_i16_payload() -> BytecodeBuildResult<()> {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(4).unwrap()),
            BytecodeFunctionKind::Script,
        );
        builder.alloc_registers(1)?;

        let conditional = builder.emit_cond_jump_placeholder(Opcode::JumpIfFalse, 0)?;
        builder.emit_ax(Opcode::Nop, 0)?;
        let target = builder.emit_ax(Opcode::ReturnUndefined, 0)?;
        builder.patch_jump_to(conditional, target)?;
        let function = builder.finish()?;

        assert_eq!(
            function
                .instructions()
                .get(usize::try_from(conditional).unwrap())
                .and_then(Instruction::bx_value),
            Some(1)
        );
        Ok(())
    }

    #[test]
    fn builder_records_wide_register_and_constant_operands() -> BytecodeBuildResult<()> {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(5).unwrap()),
            BytecodeFunctionKind::Script,
        );
        builder.alloc_registers(300)?;

        let mut last_constant = 0;
        for index in 0..70_000u32 {
            last_constant = builder.add_constant(ConstantValue::Smi(index.cast_signed()))?;
        }
        builder.emit_abx(Opcode::LoadConst, 299u16, last_constant)?;
        builder.emit_abc(Opcode::Move, 298u16, 299u16, 0u16)?;
        builder.emit_ax(Opcode::Return, 299)?;

        let function = builder.finish()?;
        let text = disassemble(&function);

        assert_eq!(function.register_count(), 300);
        assert_eq!(function.constants().len(), 70_000);
        assert_eq!(function.wide_operands().len(), 2);
        assert!(text.contains("LoadConst       r299, const[69999]"));
        assert!(text.contains("Move            r298, r299"));
        Ok(())
    }

    #[test]
    fn builder_emits_variable_width_short_instructions() -> BytecodeBuildResult<()> {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(16).unwrap()),
            BytecodeFunctionKind::Script,
        );

        builder.emit_abx(Opcode::LoadSmi8, 0, 0xfb)?;
        builder.emit_ax(Opcode::Jump8, -1)?;
        let function = builder.finish()?;

        assert_eq!(function.instruction_count(), 2);
        assert_eq!(
            function.instruction_bytes(),
            &[Opcode::LoadSmi8 as u8, 0, 0xfb, Opcode::Jump8 as u8, 0xff]
        );
        assert_eq!(
            function.instructions().to_vec(),
            vec![
                Instruction::abx(Opcode::LoadSmi8, 0, 0xfb),
                Instruction::ax(Opcode::Jump8, -1),
            ]
        );
        Ok(())
    }

    #[test]
    fn builder_prefers_store_local_when_both_move_operands_are_local() -> BytecodeBuildResult<()> {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(17).unwrap()),
            BytecodeFunctionKind::Script,
        );

        builder.emit_abc(Opcode::Move, 2, 1, 0)?;
        let function = builder.finish()?;

        assert_eq!(
            function.instructions().to_vec(),
            vec![Instruction::abx(Opcode::StoreLocal2, 1, 0)]
        );
        assert_eq!(
            function.instruction_bytes(),
            &[Opcode::StoreLocal2 as u8, 1]
        );
        Ok(())
    }

    #[test]
    fn single_register_allocation_reports_overflow_instead_of_panicking() {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(11).unwrap()),
            BytecodeFunctionKind::Script,
        );
        builder.alloc_registers(u16::MAX).unwrap();

        assert_eq!(
            builder.alloc_register(),
            Err(BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::RegisterWindow
            })
        );
    }

    #[test]
    fn narrow_call_operands_report_overflow_instead_of_panicking() {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(12).unwrap()),
            BytecodeFunctionKind::Script,
        );

        assert_eq!(
            builder.emit_call(256, 0, 0, CallRange::new(0, 0)),
            Err(BytecodeBuildError::OperandOverflow {
                kind: BytecodeOperandKind::CallResult
            })
        );
        assert_eq!(
            builder.emit_tail_call(256, 0, CallRange::new(0, 0)),
            Err(BytecodeBuildError::OperandOverflow {
                kind: BytecodeOperandKind::TailCallCallee
            })
        );
        assert_eq!(
            builder.emit_construct(0, 256, CallRange::new(0, 0)),
            Err(BytecodeBuildError::OperandOverflow {
                kind: BytecodeOperandKind::ConstructCallee
            })
        );
    }

    #[test]
    fn invalid_jump_patch_reports_error_instead_of_panicking() {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(13).unwrap()),
            BytecodeFunctionKind::Script,
        );
        let not_jump = builder.emit_ax(Opcode::Nop, 0).unwrap();

        assert_eq!(
            builder.patch_jump_to(not_jump, not_jump),
            Err(BytecodeBuildError::InvalidJumpPatch {
                instruction_offset: not_jump
            })
        );
        assert_eq!(
            builder.patch_jump_to(99, not_jump),
            Err(BytecodeBuildError::InvalidJumpPatch {
                instruction_offset: 99
            })
        );
    }

    #[test]
    fn table_limits_report_overflow_instead_of_panicking() {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(14).unwrap()),
            BytecodeFunctionKind::Script,
        );
        let child = BytecodeFunctionId::new(NonZeroU32::new(15).unwrap());
        for _ in 0..=u16::MAX {
            builder.add_child_function(child).unwrap();
        }
        assert_eq!(
            builder.add_child_function(child),
            Err(BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::ChildFunctionTable
            })
        );

        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(16).unwrap()),
            BytecodeFunctionKind::Script,
        );
        let capture = CaptureDescriptor::new(
            Some(AtomId::from_raw(1)),
            CaptureSource::EnvironmentSlot { depth: 0, slot: 0 },
        );
        for _ in 0..=u16::MAX {
            builder.add_capture(capture).unwrap();
        }
        assert_eq!(
            builder.add_capture(capture),
            Err(BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::CaptureTable
            })
        );

        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(17).unwrap()),
            BytecodeFunctionKind::Script,
        );
        let handler = ExceptionHandler::new(0, 0, 0, ExceptionHandlerKind::Catch, 0, None);
        for _ in 0..=u16::MAX {
            builder.add_exception_handler(handler).unwrap();
        }
        assert_eq!(
            builder.add_exception_handler(handler),
            Err(BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::ExceptionHandlerTable
            })
        );
    }

    #[test]
    fn feedback_and_safepoint_limits_report_overflow_instead_of_panicking() {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(18).unwrap()),
            BytecodeFunctionKind::Script,
        );
        builder.next_feedback_slot = u32::MAX;
        assert_eq!(
            builder.add_feedback_site(0, FeedbackSiteKind::Arithmetic, FeedbackSiteMetadata::None,),
            Err(BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::FeedbackSlot
            })
        );

        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(19).unwrap()),
            BytecodeFunctionKind::Script,
        );
        builder.next_safepoint_id = u32::MAX;
        assert_eq!(
            builder.alloc_safepoint_id(),
            Err(BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::SafepointId
            })
        );
    }

    #[test]
    fn final_register_window_reports_overflow_instead_of_panicking() {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(20).unwrap()),
            BytecodeFunctionKind::Script,
        );
        builder.alloc_registers(u16::MAX).unwrap();
        builder.set_hidden_register_count(1);

        assert_eq!(
            builder.finish(),
            Err(BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::FinalRegisterWindow
            })
        );
    }

    #[test]
    fn conditional_jump_patch_uses_wide_payload_for_large_spans() -> BytecodeBuildResult<()> {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(6).unwrap()),
            BytecodeFunctionKind::Script,
        );
        builder.alloc_registers(300)?;

        let conditional = builder.emit_cond_jump_placeholder(Opcode::JumpIfFalse, 299u16)?;
        for _ in 0..40_000 {
            builder.emit_ax(Opcode::Nop, 0)?;
        }
        let target = builder.emit_ax(Opcode::ReturnUndefined, 0)?;
        builder.patch_jump_to(conditional, target)?;

        let function = builder.finish()?;
        let text = disassemble(&function);

        assert!(function
            .wide_operands()
            .iter()
            .any(|operand| operand.instruction_offset() == conditional));
        assert!(text.contains("JumpIfFalse"));
        assert!(text.contains("r299"));
        assert!(text.contains("+40000"));
        Ok(())
    }
}

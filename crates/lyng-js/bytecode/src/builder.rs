use crate::function::{
    BytecodeEnvironmentBinding, BytecodeFunction, BytecodeFunctionBody, BytecodeFunctionHeader,
};
use crate::ids::{BytecodeFunctionId, EnvironmentLayoutRef};
use crate::instruction::Instruction;
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

/// Incremental builder for one immutable [`BytecodeFunction`].
pub struct BytecodeBuilder {
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
    next_feedback_slot: u32,
    next_safepoint_id: u32,
}

impl BytecodeBuilder {
    #[inline]
    pub fn new(id: BytecodeFunctionId, kind: BytecodeFunctionKind) -> Self {
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
    pub fn set_name(&mut self, name: Option<AtomId>) {
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
    pub fn set_flags(&mut self, flags: BytecodeFunctionFlags) {
        self.header = self.header.with_flags(flags);
    }

    #[inline]
    pub fn set_this_mode(&mut self, this_mode: ThisMode) {
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
    pub fn set_arguments_mode(&mut self, arguments_mode: ArgumentsMode) {
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
    pub fn set_parameter_counts(&mut self, parameter_count: u16, minimum_argument_count: u16) {
        self.header = self
            .header
            .with_parameter_counts(parameter_count, minimum_argument_count);
    }

    #[inline]
    pub fn set_parameter_initializer_end_offset(&mut self, parameter_initializer_end_offset: u32) {
        self.header = self
            .header
            .with_parameter_initializer_end_offset(parameter_initializer_end_offset);
    }

    #[inline]
    pub fn set_environment_layout(&mut self, environment_layout: Option<EnvironmentLayoutRef>) {
        self.header = self.header.with_environment_layout(environment_layout);
    }

    #[inline]
    pub fn set_source_span(&mut self, source_span: Option<Span>) {
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
    /// # Panics
    /// Panics if growing the register window would overflow the `u16` register-count limit.
    pub fn alloc_register(&mut self) -> u16 {
        self.try_alloc_register()
            .expect("register allocation overflow in bytecode builder")
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
    /// # Panics
    /// Panics if growing the register window would overflow the `u16` register-count limit.
    pub fn alloc_registers(&mut self, count: u16) -> u16 {
        self.try_alloc_registers(count)
            .expect("register allocation overflow in bytecode builder")
    }

    #[inline]
    pub fn set_hidden_register_count(&mut self, hidden_register_count: u16) {
        self.header = self
            .header
            .with_register_counts(self.header.register_count(), hidden_register_count);
    }

    #[inline]
    pub fn set_environment_slot_count(&mut self, environment_slot_count: u16) {
        self.header = self
            .header
            .with_environment_slot_count(environment_slot_count);
    }

    #[inline]
    pub fn set_has_rest_parameter(&mut self, has_rest_parameter: bool) {
        self.header = self.header.with_has_rest_parameter(has_rest_parameter);
    }

    #[inline]
    pub fn set_needs_environment(&mut self, needs_environment: bool) {
        self.header = self.header.with_needs_environment(needs_environment);
    }

    #[inline]
    /// # Panics
    /// Panics if the instruction stream length does not fit in `u32`.
    pub fn current_offset(&self) -> u32 {
        u32::try_from(self.instructions.len()).expect("instruction stream should fit in u32")
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
    pub fn emit(&mut self, instruction: Instruction) -> u32 {
        let offset = self.current_offset();
        self.instructions.push(instruction);
        offset
    }

    #[inline]
    /// # Panics
    /// Panics if any operand does not fit in the widened `u16` operand representation.
    pub fn emit_abc<A, B, C>(&mut self, opcode: Opcode, a: A, b: B, c: C) -> u32
    where
        A: TryInto<u16>,
        B: TryInto<u16>,
        C: TryInto<u16>,
        A::Error: std::fmt::Debug,
        B::Error: std::fmt::Debug,
        C::Error: std::fmt::Debug,
    {
        let operands = WideAbcOperands::new(
            a.try_into().expect("instruction operand should fit in u16"),
            b.try_into().expect("instruction operand should fit in u16"),
            c.try_into().expect("instruction operand should fit in u16"),
        );
        let offset = self.emit(Instruction::abc(
            opcode,
            operands.narrow_a(),
            operands.narrow_b(),
            operands.narrow_c(),
        ));
        if operands.needs_wide() {
            self.set_wide_operand(offset, operands.encode_payload());
        } else {
            self.remove_wide_operand(offset);
        }
        offset
    }

    #[inline]
    /// # Panics
    /// Panics if `a` does not fit in `u16` or `bx` does not fit in `u32`.
    pub fn emit_abx<A, B>(&mut self, opcode: Opcode, a: A, bx: B) -> u32
    where
        A: TryInto<u16>,
        B: TryInto<u32>,
        A::Error: std::fmt::Debug,
        B::Error: std::fmt::Debug,
    {
        let operands = WideAbxOperands::new(
            a.try_into().expect("instruction operand should fit in u16"),
            bx.try_into()
                .expect("instruction operand should fit in u32"),
        );
        let offset = self.emit(Instruction::abx(
            opcode,
            operands.narrow_a(),
            operands.narrow_bx(),
        ));
        if operands.needs_wide() {
            self.set_wide_operand(offset, operands.encode_payload());
        } else {
            self.remove_wide_operand(offset);
        }
        offset
    }

    #[inline]
    pub fn emit_ax(&mut self, opcode: Opcode, ax: i32) -> u32 {
        self.emit(Instruction::ax(opcode, ax))
    }

    #[inline]
    /// # Panics
    /// Panics if `opcode` is not part of the jump family.
    pub fn emit_jump_placeholder(&mut self, opcode: Opcode) -> u32 {
        assert!(
            opcode.is_jump(),
            "jump patching requires a jump-family opcode"
        );
        self.emit_ax(opcode, 0)
    }

    #[inline]
    /// # Panics
    /// Panics if `opcode` is not a conditional jump opcode or if `condition` does not fit in
    /// the widened operand representation.
    pub fn emit_cond_jump_placeholder<A>(&mut self, opcode: Opcode, condition: A) -> u32
    where
        A: TryInto<u16>,
        A::Error: std::fmt::Debug,
    {
        assert!(
            matches!(opcode, Opcode::JumpIfTrue | Opcode::JumpIfFalse),
            "conditional jump patching requires a conditional jump opcode"
        );
        self.emit_abx(opcode, condition, 0)
    }

    #[inline]
    /// # Panics
    /// Panics while call operands are still narrow if any register operand does not fit in `u8`.
    pub fn emit_call(
        &mut self,
        result: u16,
        callee: u16,
        this_value: u16,
        arguments: CallRange,
    ) -> u32 {
        let result = u8::try_from(result)
            .expect("call result register must fit into u8 until call operands widen");
        let callee = u8::try_from(callee)
            .expect("call callee register must fit into u8 until call operands widen");
        let this_value = u8::try_from(this_value)
            .expect("call this register must fit into u8 until call operands widen");
        let instruction_offset =
            self.emit(Instruction::abc(Opcode::Call, result, callee, this_value));
        self.add_wide_operand(instruction_offset, arguments.encode());
        instruction_offset
    }

    #[inline]
    /// # Panics
    /// Panics while tail-call operands are still narrow if any register operand does not fit in
    /// `u8`.
    pub fn emit_tail_call(&mut self, callee: u16, this_value: u16, arguments: CallRange) -> u32 {
        let callee = u8::try_from(callee)
            .expect("tail-call callee register must fit into u8 until call operands widen");
        let this_value = u8::try_from(this_value)
            .expect("tail-call this register must fit into u8 until call operands widen");
        self.header = self
            .header
            .with_flags(self.header.flags().with_tail_call_capable(true));
        let instruction_offset =
            self.emit(Instruction::abc(Opcode::TailCall, callee, this_value, 0));
        self.add_wide_operand(instruction_offset, arguments.encode());
        instruction_offset
    }

    #[inline]
    /// # Panics
    /// Panics while construct operands are still narrow if any register operand does not fit in
    /// `u8`.
    pub fn emit_construct(&mut self, result: u16, callee: u16, arguments: CallRange) -> u32 {
        let result = u8::try_from(result)
            .expect("construct result register must fit into u8 until construct operands widen");
        let callee = u8::try_from(callee)
            .expect("construct callee register must fit into u8 until construct operands widen");
        let instruction_offset = self.emit(Instruction::abc(Opcode::Construct, result, callee, 0));
        self.add_wide_operand(instruction_offset, arguments.encode());
        instruction_offset
    }

    #[inline]
    /// # Panics
    /// Panics if `instruction_offset` does not reference a patchable jump instruction or if the
    /// computed relative jump delta does not fit in the encoded operand width.
    pub fn patch_jump_to(&mut self, instruction_offset: u32, target_offset: u32) {
        let existing_abx = self
            .instructions
            .get(usize::try_from(instruction_offset).expect("u32 should fit in usize"));
        let existing_abx = match existing_abx {
            Some(Instruction::Abx { a, bx, .. }) => Some((*a, *bx)),
            _ => None,
        };
        let existing_operands =
            existing_abx.map(|(a, bx)| self.decode_abx_operands(instruction_offset, a, bx));
        let instruction = self
            .instructions
            .get_mut(usize::try_from(instruction_offset).expect("u32 should fit in usize"))
            .expect("patch target should exist");
        let delta = i64::from(target_offset) - (i64::from(instruction_offset) + 1);
        match instruction {
            Instruction::Ax { .. } => {
                let delta = i32::try_from(delta).expect("relative jump delta should fit in i32");
                instruction.patch_ax(delta);
            }
            Instruction::Abx { a, .. } => {
                let delta = i32::try_from(delta).expect("conditional jump delta should fit in i32");
                let operands = existing_operands
                    .expect("existing Abx operands should be captured before patching");
                let updated =
                    WideAbxOperands::new(operands.a(), u32::from_le_bytes(delta.to_le_bytes()));
                *a = updated.narrow_a();
                instruction.patch_bx(updated.narrow_bx());
                if updated.needs_wide() {
                    self.set_wide_operand(instruction_offset, updated.encode_payload());
                } else {
                    self.remove_wide_operand(instruction_offset);
                }
            }
            Instruction::Abc { .. } => {
                panic!("only jump-family instructions can be patched")
            }
        }
    }

    #[inline]
    /// # Panics
    /// Panics if the constant pool length does not fit in `u32`.
    pub fn add_constant(&mut self, constant: ConstantValue) -> u32 {
        let index = u32::try_from(self.constants.len()).expect("constant pool should fit in u32");
        self.constants.push(constant);
        index
    }

    #[inline]
    /// # Panics
    /// Panics if the child-function table length does not fit in `u16`.
    pub fn add_child_function(&mut self, child: BytecodeFunctionId) -> u16 {
        let index = u16::try_from(self.child_functions.len())
            .expect("child-function count should fit in u16");
        self.child_functions.push(child);
        index
    }

    #[inline]
    /// # Panics
    /// Panics if the capture table length does not fit in `u16`.
    pub fn add_capture(&mut self, capture: CaptureDescriptor) -> u16 {
        let index = u16::try_from(self.captures.len()).expect("capture count should fit in u16");
        self.captures.push(capture);
        index
    }

    #[inline]
    /// # Panics
    /// Panics if the exception-handler table length does not fit in `u16`.
    pub fn add_exception_handler(&mut self, handler: ExceptionHandler) -> u16 {
        let index = u16::try_from(self.exception_handlers.len())
            .expect("exception-handler count should fit in u16");
        self.exception_handlers.push(handler);
        index
    }

    #[inline]
    pub fn add_loop_iteration_environment_site(
        &mut self,
        instruction_offset: u32,
        iteration_slots: Vec<u16>,
        shared_slots: Vec<u16>,
    ) {
        self.loop_iteration_environment_sites
            .push(LoopIterationEnvironmentSite::new(
                instruction_offset,
                iteration_slots,
                shared_slots,
            ));
    }

    #[inline]
    pub fn add_direct_eval_lexical_site(
        &mut self,
        instruction_offset: u32,
        scopes: Vec<DirectEvalLexicalScope>,
        flags: DirectEvalSiteFlags,
    ) {
        self.direct_eval_lexical_sites
            .push(DirectEvalLexicalSite::new(
                instruction_offset,
                scopes,
                flags,
            ));
    }

    #[inline]
    /// # Panics
    /// Panics if the next feedback slot id would overflow the supported `u32` range.
    pub fn add_feedback_site(
        &mut self,
        instruction_offset: u32,
        kind: FeedbackSiteKind,
        metadata: FeedbackSiteMetadata,
    ) -> FeedbackSlotId {
        let slot = FeedbackSlotId::from_raw(self.next_feedback_slot)
            .expect("feedback slots should start at one");
        self.next_feedback_slot = self
            .next_feedback_slot
            .checked_add(1)
            .expect("feedback-slot counter should fit in u32");
        self.feedback_sites.push(
            FeedbackSiteDescriptor::new(slot, instruction_offset, kind).with_metadata(metadata),
        );
        slot
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
    pub fn add_safepoint_at(
        &mut self,
        instruction_offset: u32,
        kind: SafepointKind,
        register_window_len: u16,
    ) -> u32 {
        let id = self.alloc_safepoint_id();
        self.safepoints.push(SafepointDescriptor::new(
            id,
            instruction_offset,
            kind,
            register_window_len,
        ));
        id
    }

    #[inline]
    /// # Panics
    /// Panics if the next safepoint id would overflow the supported `u32` range.
    pub fn alloc_safepoint_id(&mut self) -> u32 {
        let id = self.next_safepoint_id;
        self.next_safepoint_id = self
            .next_safepoint_id
            .checked_add(1)
            .expect("safepoint ids should fit in u32");
        id
    }

    #[inline]
    pub fn add_deopt_snapshot(&mut self, snapshot: DeoptSnapshot) {
        self.deopt_snapshots.push(snapshot);
    }

    #[inline]
    /// # Panics
    /// Panics if the combined visible and hidden register window would overflow `u16`.
    pub fn finish(self) -> BytecodeFunction {
        let final_register_window = self
            .header
            .register_count()
            .checked_add(self.header.hidden_register_count())
            .expect("final register window should fit in u16");
        BytecodeFunction::from_parts(
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
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{disassemble, ExceptionHandlerKind};
    use std::num::NonZeroU32;

    #[test]
    fn builder_tracks_constants_jumps_handlers_and_children() {
        let mut builder = BytecodeBuilder::for_function(
            BytecodeFunctionId::new(NonZeroU32::new(1).unwrap()),
            Some(AtomId::from_raw(41)),
            ArgumentsMode::Unmapped,
        );
        builder.set_flags(BytecodeFunctionFlags::new(true, false));
        builder.set_parameter_counts(1, 1);
        builder.alloc_registers(3);
        builder.set_hidden_register_count(1);

        let constant = builder.add_constant(ConstantValue::Smi(42));
        let load = builder.emit_abx(Opcode::LoadConst, 0, constant);
        let jump = builder.emit_jump_placeholder(Opcode::Jump);
        let _move_inst = builder.emit_abc(Opcode::Move, 1, 0, 0);
        let ret = builder.emit_ax(Opcode::Return, 1);
        builder.patch_jump_to(jump, ret);
        builder.add_child_function(BytecodeFunctionId::new(NonZeroU32::new(2).unwrap()));
        builder.add_exception_handler(ExceptionHandler::new(
            load,
            ret,
            ret,
            ExceptionHandlerKind::Catch,
            1,
            Some(2),
        ));

        let function = builder.finish();

        assert_eq!(function.constants(), &[ConstantValue::Smi(42)]);
        assert_eq!(
            function.child_functions(),
            &[BytecodeFunctionId::new(NonZeroU32::new(2).unwrap())]
        );
        assert_eq!(function.exception_handlers().len(), 1);
        assert_eq!(function.register_count(), 3);
        assert_eq!(function.hidden_register_count(), 1);
        assert_eq!(
            function.instructions()[usize::try_from(jump).unwrap()].ax_value(),
            Some(1)
        );
    }

    #[test]
    fn builder_assigns_feedback_sites_and_metadata() {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(9).unwrap()),
            BytecodeFunctionKind::Function,
        );
        let instruction = builder.emit_abc(Opcode::Add, 0, 1, 2);
        let slot = builder.add_feedback_site(
            instruction,
            FeedbackSiteKind::Arithmetic,
            FeedbackSiteMetadata::None,
        );
        let function = builder.finish();

        assert_eq!(slot, FeedbackSlotId::from_raw(1).unwrap());
        assert_eq!(function.feedback_sites().len(), 1);
        assert_eq!(
            function.feedback_sites()[0].instruction_offset(),
            instruction
        );
    }

    #[test]
    fn builder_records_source_maps_safepoints_and_deopt_snapshots() {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(10).unwrap()),
            BytecodeFunctionKind::Function,
        );
        builder.alloc_registers(2);
        let call = builder.emit_tail_call(0, 1, CallRange::new(0, 1));
        builder.add_source_map_entry(SourceId::new(9), call, 4, 18);
        let safepoint = builder.add_safepoint_at(call, SafepointKind::Allocation, 2);
        builder.add_deopt_snapshot(DeoptSnapshot::new(
            safepoint,
            vec![crate::DeoptValueSource::FrameValue(
                crate::DeoptFrameValue::ThisValue,
            )],
        ));

        let function = builder.finish();

        assert_eq!(function.source_map().len(), 1);
        assert_eq!(function.safepoints().len(), 1);
        assert_eq!(function.deopt_snapshots().len(), 1);
        assert_eq!(function.safepoints()[0].id(), safepoint);
        assert_eq!(function.safepoints()[0].kind(), SafepointKind::Allocation);
        assert_eq!(function.deopt_snapshots()[0].safepoint_id(), safepoint);
    }

    #[test]
    fn disassembly_is_stable_for_builder_output() {
        let mut builder = BytecodeBuilder::for_function(
            BytecodeFunctionId::new(NonZeroU32::new(3).unwrap()),
            Some(AtomId::from_raw(19)),
            ArgumentsMode::None,
        );
        builder.alloc_registers(2);
        let constant = builder.add_constant(ConstantValue::Smi(7));
        builder.emit_abx(Opcode::LoadConst, 0, constant);
        builder.emit_ax(Opcode::Return, 0);
        let function = builder.finish();

        let text = disassemble(&function);

        assert_eq!(
            text,
            "function BytecodeFunctionId(3) kind=Function this=Global args=None params=0 min_args=0 regs=2 hidden=0 env=false env_slots=0 rest=false\n0000: LoadConst       r0, const[0] ; Smi(7)\n0001: Return          r0\nconstants:\n  [0] Smi(7)\n"
        );
    }

    #[test]
    fn disassembly_renders_explicit_tail_call_sites() {
        let mut builder = BytecodeBuilder::for_function(
            BytecodeFunctionId::new(NonZeroU32::new(7).unwrap()),
            Some(AtomId::from_raw(23)),
            ArgumentsMode::None,
        );
        builder.alloc_registers(6);
        builder.emit_tail_call(2, 3, CallRange::new(4, 2));
        let function = builder.finish();

        let text = disassemble(&function);

        assert!(text.contains("TailCall        callee=r2, this=r3, args=[r4..r6)"));
    }

    #[test]
    fn conditional_jump_patch_uses_signed_i16_payload() {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(4).unwrap()),
            BytecodeFunctionKind::Script,
        );
        builder.alloc_registers(1);

        let conditional = builder.emit_cond_jump_placeholder(Opcode::JumpIfFalse, 0);
        let target = builder.emit_ax(Opcode::ReturnUndefined, 0);
        builder.patch_jump_to(conditional, target);
        let function = builder.finish();

        assert_eq!(
            function.instructions()[usize::try_from(conditional).unwrap()].bx_value(),
            Some(0)
        );
    }

    #[test]
    fn builder_records_wide_register_and_constant_operands() {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(5).unwrap()),
            BytecodeFunctionKind::Script,
        );
        builder.alloc_registers(300);

        let mut last_constant = 0;
        for index in 0..70_000u32 {
            last_constant = builder.add_constant(ConstantValue::Smi(index.cast_signed()));
        }
        builder.emit_abx(Opcode::LoadConst, 299u16, last_constant);
        builder.emit_abc(Opcode::Move, 298u16, 299u16, 0u16);
        builder.emit_ax(Opcode::Return, 299);

        let function = builder.finish();
        let text = disassemble(&function);

        assert_eq!(function.register_count(), 300);
        assert_eq!(function.constants().len(), 70_000);
        assert_eq!(function.wide_operands().len(), 2);
        assert!(text.contains("LoadConst       r299, const[69999]"));
        assert!(text.contains("Move            r298, r299"));
    }

    #[test]
    #[should_panic(expected = "register allocation overflow in bytecode builder")]
    fn single_register_allocation_panics_on_overflow_instead_of_saturating() {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(11).unwrap()),
            BytecodeFunctionKind::Script,
        );
        builder.alloc_registers(u16::MAX);

        let _ = builder.alloc_register();
    }

    #[test]
    fn conditional_jump_patch_uses_wide_payload_for_large_spans() {
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::new(NonZeroU32::new(6).unwrap()),
            BytecodeFunctionKind::Script,
        );
        builder.alloc_registers(300);

        let conditional = builder.emit_cond_jump_placeholder(Opcode::JumpIfFalse, 299u16);
        for _ in 0..40_000 {
            builder.emit_ax(Opcode::Nop, 0);
        }
        let target = builder.emit_ax(Opcode::ReturnUndefined, 0);
        builder.patch_jump_to(conditional, target);

        let function = builder.finish();
        let text = disassemble(&function);

        assert!(function
            .wide_operands()
            .iter()
            .any(|operand| operand.instruction_offset() == conditional));
        assert!(text.contains("JumpIfFalse"));
        assert!(text.contains("r299"));
        assert!(text.contains("+40000"));
    }
}

use super::state::GeneratorResumeRegisters;
use super::{
    AssignOp, AtomId, BinaryOp, BuiltinFunctionId, BytecodeBuildError, BytecodeLimitKind,
    CompletionKind, ConstantValue, DeoptFrameValue, DeoptSnapshot, DeoptValueSource, Expr, ExprId,
    FeedbackSiteKind, FeedbackSiteMetadata, FunctionCompiler, LoweringError, LoweringResult,
    Opcode, SafepointKind, Span,
};

impl FunctionCompiler<'_, '_> {
    pub(super) fn alloc_temp(&mut self) -> LoweringResult<u16> {
        let register = self
            .builder
            .try_alloc_register()
            .ok_or(LoweringError::RegisterOverflow { register: u16::MAX })?;
        let _ = self.encode_register(register)?;
        Ok(register)
    }

    pub(super) fn ensure_generator_resume_registers(
        &mut self,
    ) -> LoweringResult<GeneratorResumeRegisters> {
        if let Some(registers) = self.generator_resume_registers {
            return Ok(registers);
        }
        let registers = GeneratorResumeRegisters {
            kind: self.alloc_temp()?,
            value: self.alloc_temp()?,
        };
        self.generator_resume_registers = Some(registers);
        Ok(registers)
    }

    pub(super) fn emit_generator_resume_dispatch(&mut self, dest: u16) -> LoweringResult<()> {
        let registers = self.ensure_generator_resume_registers()?;
        self.builder
            .emit_ax(Opcode::LoadResumeKind, i32::from(registers.kind))?;
        self.builder
            .emit_ax(Opcode::LoadResumeValue, i32::from(registers.value))?;

        let next_kind = self.alloc_temp()?;
        self.emit_load_smi(next_kind, 0)?;
        let is_next = self.alloc_temp()?;
        self.builder.emit_abc(
            Opcode::StrictEqual,
            self.encode_register(is_next)?,
            self.encode_register(registers.kind)?,
            self.encode_register(next_kind)?,
        )?;
        let jump_not_next = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfFalse, self.encode_register(is_next)?)?;
        self.emit_move(dest, registers.value)?;
        let jump_end = self.builder.emit_jump_placeholder(Opcode::Jump)?;

        let throw_entry = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_not_next, throw_entry)?;
        let throw_kind = self.alloc_temp()?;
        self.emit_load_smi(throw_kind, 1)?;
        let is_throw = self.alloc_temp()?;
        self.builder.emit_abc(
            Opcode::StrictEqual,
            self.encode_register(is_throw)?,
            self.encode_register(registers.kind)?,
            self.encode_register(throw_kind)?,
        )?;
        let jump_return = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfFalse, self.encode_register(is_throw)?)?;
        self.builder
            .emit_ax(Opcode::Throw, i32::from(registers.value))?;

        let return_entry = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_return, return_entry)?;
        if let Some(finally_index) = self.nearest_active_finally() {
            self.set_completion_state(CompletionKind::Return, Some(registers.value), None)?;
            self.emit_jump_to_finally(finally_index)?;
        } else {
            self.builder
                .emit_ax(Opcode::Return, i32::from(registers.value))?;
        }

        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_end, end)?;
        Ok(())
    }

    pub(super) fn emit_generator_delegate_completion_dispatch(
        &mut self,
        value_register: u16,
    ) -> LoweringResult<()> {
        let registers = self.ensure_generator_resume_registers()?;
        self.builder
            .emit_ax(Opcode::LoadResumeKind, i32::from(registers.kind))?;
        self.builder
            .emit_ax(Opcode::LoadResumeValue, i32::from(registers.value))?;
        let return_kind = self.alloc_temp()?;
        self.emit_load_smi(return_kind, 2)?;
        let is_return = self.alloc_temp()?;
        self.builder.emit_abc(
            Opcode::StrictEqual,
            self.encode_register(is_return)?,
            self.encode_register(registers.kind)?,
            self.encode_register(return_kind)?,
        )?;
        let jump_end = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfFalse, self.encode_register(is_return)?)?;
        if let Some(finally_index) = self.nearest_active_finally() {
            self.set_completion_state(CompletionKind::Return, Some(value_register), None)?;
            self.emit_jump_to_finally(finally_index)?;
        } else {
            self.builder
                .emit_ax(Opcode::Return, i32::from(value_register))?;
        }
        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_end, end)?;
        Ok(())
    }

    pub(super) fn emit_move(&mut self, dest: u16, src: u16) -> LoweringResult<()> {
        if dest == src {
            return Ok(());
        }
        self.builder.emit_abc(
            Opcode::Move,
            self.encode_register(dest)?,
            self.encode_register(src)?,
            0,
        )?;
        Ok(())
    }

    pub(super) fn emit_load_undefined(&mut self, dest: u16) -> LoweringResult<()> {
        self.builder
            .emit_abx(Opcode::LoadUndefined, self.encode_register(dest)?, 0)?;
        Ok(())
    }

    pub(super) fn emit_load_uninitialized_lexical(&mut self, dest: u16) -> LoweringResult<()> {
        self.builder.emit_abx(
            Opcode::LoadUninitializedLexical,
            self.encode_register(dest)?,
            0,
        )?;
        Ok(())
    }

    pub(super) fn emit_throw_if_uninitialized(&mut self, value: u16) -> LoweringResult<()> {
        self.builder.emit_abx(
            Opcode::ThrowIfUninitialized,
            self.encode_register(value)?,
            0,
        )?;
        Ok(())
    }

    pub(super) fn emit_load_null(&mut self, dest: u16) -> LoweringResult<()> {
        self.builder
            .emit_abx(Opcode::LoadNull, self.encode_register(dest)?, 0)?;
        Ok(())
    }

    pub(super) fn emit_load_bool(&mut self, dest: u16, value: bool) -> LoweringResult<()> {
        self.builder.emit_abx(
            if value {
                Opcode::LoadTrue
            } else {
                Opcode::LoadFalse
            },
            self.encode_register(dest)?,
            0,
        )?;
        Ok(())
    }

    pub(super) fn emit_load_smi(&mut self, dest: u16, value: i16) -> LoweringResult<()> {
        self.builder.emit_abx(
            Opcode::LoadSmi,
            self.encode_register(dest)?,
            u16::from_le_bytes(value.to_le_bytes()),
        )?;
        Ok(())
    }

    pub(super) fn emit_load_this(&mut self, dest: u16) -> LoweringResult<()> {
        self.builder
            .emit_abx(Opcode::LoadThis, self.encode_register(dest)?, 0)?;
        Ok(())
    }

    pub(super) fn emit_load_callee(&mut self, dest: u16) -> LoweringResult<()> {
        self.builder
            .emit_abx(Opcode::LoadCallee, self.encode_register(dest)?, 0)?;
        Ok(())
    }

    pub(super) fn emit_profiled_negate(&mut self, dest: u16, argument: u16) -> LoweringResult<()> {
        let instruction_offset = self.builder.emit_abc(
            Opcode::Negate,
            self.encode_register(dest)?,
            self.encode_register(argument)?,
            0,
        )?;
        self.builder.add_feedback_site(
            instruction_offset,
            FeedbackSiteKind::Arithmetic,
            FeedbackSiteMetadata::None,
        )?;
        Ok(())
    }

    pub(super) fn emit_profiled_bit_not(&mut self, dest: u16, argument: u16) -> LoweringResult<()> {
        let instruction_offset = self.builder.emit_abc(
            Opcode::BitNot,
            self.encode_register(dest)?,
            self.encode_register(argument)?,
            0,
        )?;
        self.builder.add_feedback_site(
            instruction_offset,
            FeedbackSiteKind::Arithmetic,
            FeedbackSiteMetadata::None,
        )?;
        Ok(())
    }

    pub(super) fn emit_profiled_update(
        &mut self,
        opcode: Opcode,
        dest: u16,
        argument: u16,
    ) -> LoweringResult<()> {
        debug_assert!(matches!(opcode, Opcode::Increment | Opcode::Decrement));
        let instruction_offset = self.builder.emit_abc(
            opcode,
            self.encode_register(dest)?,
            self.encode_register(argument)?,
            0,
        )?;
        self.builder.add_feedback_site(
            instruction_offset,
            FeedbackSiteKind::Arithmetic,
            FeedbackSiteMetadata::None,
        )?;
        Ok(())
    }

    pub(super) fn emit_load_new_target(&mut self, dest: u16) -> LoweringResult<()> {
        self.builder
            .emit_abx(Opcode::LoadNewTarget, self.encode_register(dest)?, 0)?;
        Ok(())
    }

    pub(super) fn emit_check_object_coercible(&mut self, value: u16) -> LoweringResult<()> {
        self.builder.emit_abx(
            Opcode::CheckObjectCoercible,
            self.encode_register(value)?,
            0,
        )?;
        Ok(())
    }

    pub(super) fn emit_load_numeric(
        &mut self,
        dest: u16,
        value: lyng_js_ast::NumericLiteral,
    ) -> LoweringResult<()> {
        match value {
            lyng_js_ast::NumericLiteral::Int32(number) => {
                if let Ok(number) = i16::try_from(number) {
                    self.emit_load_smi(dest, number)
                } else {
                    let index = self.constant_smi(number)?;
                    self.builder
                        .emit_abx(Opcode::LoadConst, self.encode_register(dest)?, index)?;
                    Ok(())
                }
            }
            lyng_js_ast::NumericLiteral::Number(number) => {
                let index = self.constant_float(number.to_bits())?;
                self.builder
                    .emit_abx(Opcode::LoadConst, self.encode_register(dest)?, index)?;
                Ok(())
            }
        }
    }

    pub(super) fn emit_load_string(
        &mut self,
        dest: u16,
        value: lyng_js_ast::StringLiteralId,
    ) -> LoweringResult<()> {
        let literal = self.ast().literals().get_string_value(value).clone();
        let atom = match literal {
            lyng_js_ast::StringLiteralValue::Utf8(text) => self.state.atoms.intern(&text),
            lyng_js_ast::StringLiteralValue::Utf16(units) => self.state.atoms.intern_utf16(&units),
        };
        let index = self.constant_atom(atom)?;
        self.builder
            .emit_abx(Opcode::LoadConst, self.encode_register(dest)?, index)?;
        Ok(())
    }

    pub(super) fn emit_load_atom_string(&mut self, dest: u16, atom: AtomId) -> LoweringResult<()> {
        let index = self.constant_atom(atom)?;
        self.builder
            .emit_abx(Opcode::LoadConst, self.encode_register(dest)?, index)?;
        Ok(())
    }

    pub(super) fn emit_load_builtin(
        &mut self,
        dest: u16,
        builtin: BuiltinFunctionId,
    ) -> LoweringResult<()> {
        let index = self.constant_builtin(builtin)?;
        self.builder
            .emit_abx(Opcode::LoadConst, self.encode_register(dest)?, index)?;
        Ok(())
    }

    pub(super) fn emit_load_i32_constant(&mut self, dest: u16, value: i32) -> LoweringResult<()> {
        if let Ok(value) = i16::try_from(value) {
            return self.emit_load_smi(dest, value);
        }
        let index = self.constant_smi(value)?;
        self.builder
            .emit_abx(Opcode::LoadConst, self.encode_register(dest)?, index)?;
        Ok(())
    }

    pub(super) fn emit_load_global(&mut self, dest: u16, name: AtomId) -> LoweringResult<()> {
        let index = self.constant_atom(name)?;
        let instruction_offset =
            self.builder
                .emit_abx(Opcode::LoadGlobal, self.encode_register(dest)?, index)?;
        self.builder.add_feedback_site(
            instruction_offset,
            FeedbackSiteKind::NamedPropertyLoad,
            FeedbackSiteMetadata::NamedProperty(name),
        )?;
        Ok(())
    }

    pub(super) fn emit_store_global(&mut self, value: u16, name: AtomId) -> LoweringResult<()> {
        let index = self.constant_atom(name)?;
        let instruction_offset =
            self.builder
                .emit_abx(Opcode::StoreGlobal, self.encode_register(value)?, index)?;
        self.builder.add_feedback_site(
            instruction_offset,
            FeedbackSiteKind::NamedPropertyStore,
            FeedbackSiteMetadata::NamedProperty(name),
        )?;
        Ok(())
    }

    pub(super) fn emit_assign_global(&mut self, value: u16, name: AtomId) -> LoweringResult<()> {
        let index = self.constant_atom(name)?;
        let instruction_offset =
            self.builder
                .emit_abx(Opcode::AssignGlobal, self.encode_register(value)?, index)?;
        self.builder.add_feedback_site(
            instruction_offset,
            FeedbackSiteKind::NamedPropertyStore,
            FeedbackSiteMetadata::NamedProperty(name),
        )?;
        Ok(())
    }

    pub(super) fn emit_load_name(&mut self, dest: u16, name: AtomId) -> LoweringResult<()> {
        let index = self.constant_atom(name)?;
        self.builder
            .emit_abx(Opcode::LoadName, self.encode_register(dest)?, index)?;
        Ok(())
    }

    pub(super) fn emit_assign_name(&mut self, value: u16, name: AtomId) -> LoweringResult<()> {
        let index = self.constant_atom(name)?;
        self.builder
            .emit_abx(Opcode::AssignName, self.encode_register(value)?, index)?;
        Ok(())
    }

    pub(super) fn emit_assign_variable_name(
        &mut self,
        value: u16,
        name: AtomId,
    ) -> LoweringResult<()> {
        let index = self.constant_atom(name)?;
        self.builder.emit_abx(
            Opcode::AssignVariableName,
            self.encode_register(value)?,
            index,
        )?;
        Ok(())
    }

    pub(super) fn emit_capture_name(&mut self, reference: u16, name: AtomId) -> LoweringResult<()> {
        let index = self.constant_atom(name)?;
        self.builder
            .emit_abx(Opcode::CaptureName, self.encode_register(reference)?, index)?;
        Ok(())
    }

    pub(super) fn emit_load_captured_name(
        &mut self,
        dest: u16,
        reference: u16,
    ) -> LoweringResult<()> {
        self.builder.emit_abx(
            Opcode::LoadCapturedName,
            self.encode_register(dest)?,
            u32::from(reference),
        )?;
        Ok(())
    }

    pub(super) fn emit_load_captured_name_this(
        &mut self,
        dest: u16,
        reference: u16,
    ) -> LoweringResult<()> {
        self.builder.emit_abx(
            Opcode::LoadCapturedNameThis,
            self.encode_register(dest)?,
            u32::from(self.encode_register(reference)?),
        )?;
        Ok(())
    }

    pub(super) fn emit_assign_captured_name(
        &mut self,
        value: u16,
        reference: u16,
    ) -> LoweringResult<()> {
        self.builder.emit_abx(
            Opcode::AssignCapturedName,
            self.encode_register(value)?,
            u32::from(reference),
        )?;
        Ok(())
    }

    pub(super) fn emit_delete_name(&mut self, dest: u16, name: AtomId) -> LoweringResult<()> {
        let index = self.constant_atom(name)?;
        self.builder
            .emit_abx(Opcode::DeleteName, self.encode_register(dest)?, index)?;
        Ok(())
    }

    pub(super) fn emit_push_with_env(&mut self, object: u16, span: Span) -> LoweringResult<()> {
        let instruction_offset = self
            .builder
            .emit_ax(Opcode::PushWithEnv, i32::from(object))?;
        self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation)?;
        Ok(())
    }

    pub(super) fn emit_pop_with_env(&mut self) -> LoweringResult<()> {
        self.builder.emit_ax(Opcode::PopWithEnv, 0)?;
        Ok(())
    }

    pub(super) fn emit_enter_env_scope(&mut self, base: u16, count: u32) -> LoweringResult<()> {
        self.builder
            .emit_abx(Opcode::EnterEnvScope, self.encode_register(base)?, count)?;
        Ok(())
    }

    pub(super) fn emit_leave_env_scope(&mut self, base: u16, count: u32) -> LoweringResult<()> {
        self.builder
            .emit_abx(Opcode::LeaveEnvScope, self.encode_register(base)?, count)?;
        Ok(())
    }

    pub(super) fn emit_load_env_slot(
        &mut self,
        dest: u16,
        depth: u8,
        slot: u32,
    ) -> LoweringResult<()> {
        let operand = encode_env_operand(depth, slot);
        self.builder
            .emit_abx(Opcode::LoadEnvSlot, self.encode_register(dest)?, operand)?;
        Ok(())
    }

    pub(super) fn emit_store_env_slot(
        &mut self,
        value: u16,
        depth: u8,
        slot: u32,
    ) -> LoweringResult<()> {
        let operand = encode_env_operand(depth, slot);
        self.builder
            .emit_abx(Opcode::StoreEnvSlot, self.encode_register(value)?, operand)?;
        Ok(())
    }

    pub(super) fn emit_assign_env_slot(
        &mut self,
        value: u16,
        depth: u8,
        slot: u32,
    ) -> LoweringResult<()> {
        let operand = encode_env_operand(depth, slot);
        self.builder
            .emit_abx(Opcode::AssignEnvSlot, self.encode_register(value)?, operand)?;
        Ok(())
    }

    pub(super) fn named_property_atom(&mut self, expr: ExprId) -> Option<AtomId> {
        let expr = self.ast().get_expr(expr).clone();
        match expr {
            Expr::Identifier { name, .. } => Some(name),
            Expr::StringLiteral { value, .. } => {
                let literal = self.ast().literals().get_string_value(value).clone();
                match literal {
                    lyng_js_ast::StringLiteralValue::Utf8(text) => {
                        if is_canonical_array_index_string(&text) {
                            return None;
                        }
                        Some(self.state.atoms.intern(&text))
                    }
                    lyng_js_ast::StringLiteralValue::Utf16(units) => {
                        Some(self.state.atoms.intern_utf16(&units))
                    }
                }
            }
            _ => None,
        }
    }

    pub(super) fn named_atom_operand(&mut self, atom: AtomId) -> LoweringResult<u16> {
        let index = self.constant_atom(atom)?;
        Self::encode_small_index(index)
    }

    pub(super) fn emit_get_property_by_atom(
        &mut self,
        dest: u16,
        object: u16,
        atom: AtomId,
    ) -> LoweringResult<()> {
        match self.named_atom_operand(atom) {
            Ok(index) => {
                let instruction_offset = self.builder.emit_abc(
                    Opcode::GetNamedProperty,
                    self.encode_register(dest)?,
                    self.encode_register(object)?,
                    index,
                )?;
                self.builder.add_feedback_site(
                    instruction_offset,
                    FeedbackSiteKind::NamedPropertyLoad,
                    FeedbackSiteMetadata::NamedProperty(atom),
                )?;
            }
            Err(LoweringError::ConstantIndexOverflow { .. }) => {
                let key_register = self.alloc_temp()?;
                self.emit_load_atom_string(key_register, atom)?;
                self.emit_get_keyed_property(dest, object, key_register)?;
            }
            Err(error) => return Err(error),
        }
        Ok(())
    }

    pub(super) fn emit_set_property_by_atom(
        &mut self,
        object: u16,
        value: u16,
        atom: AtomId,
    ) -> LoweringResult<()> {
        match self.named_atom_operand(atom) {
            Ok(index) => {
                let instruction_offset = self.builder.emit_abc(
                    Opcode::SetNamedProperty,
                    self.encode_register(object)?,
                    self.encode_register(value)?,
                    index,
                )?;
                self.builder.add_feedback_site(
                    instruction_offset,
                    FeedbackSiteKind::NamedPropertyStore,
                    FeedbackSiteMetadata::NamedProperty(atom),
                )?;
            }
            Err(LoweringError::ConstantIndexOverflow { .. }) => {
                let key_register = self.alloc_temp()?;
                self.emit_load_atom_string(key_register, atom)?;
                self.emit_set_keyed_property(object, value, key_register)?;
            }
            Err(error) => return Err(error),
        }
        Ok(())
    }

    pub(super) fn emit_assign_property_by_atom(
        &mut self,
        object: u16,
        value: u16,
        atom: AtomId,
    ) -> LoweringResult<()> {
        match self.named_atom_operand(atom) {
            Ok(index) => {
                let opcode = if self.force_strict_assignment {
                    Opcode::StrictAssignNamedProperty
                } else {
                    Opcode::AssignNamedProperty
                };
                let instruction_offset = self.builder.emit_abc(
                    opcode,
                    self.encode_register(object)?,
                    self.encode_register(value)?,
                    index,
                )?;
                self.builder.add_feedback_site(
                    instruction_offset,
                    FeedbackSiteKind::NamedPropertyStore,
                    FeedbackSiteMetadata::NamedProperty(atom),
                )?;
            }
            Err(LoweringError::ConstantIndexOverflow { .. }) => {
                let key_register = self.alloc_temp()?;
                self.emit_load_atom_string(key_register, atom)?;
                self.emit_assign_keyed_property(object, value, key_register)?;
            }
            Err(error) => return Err(error),
        }
        Ok(())
    }

    pub(super) fn emit_define_property_by_atom(
        &mut self,
        object: u16,
        value: u16,
        atom: AtomId,
    ) -> LoweringResult<()> {
        match self.named_atom_operand(atom) {
            Ok(index) => {
                self.builder.emit_abc(
                    Opcode::DefineNamedProperty,
                    self.encode_register(object)?,
                    self.encode_register(value)?,
                    index,
                )?;
            }
            Err(LoweringError::ConstantIndexOverflow { .. }) => {
                let key_register = self.alloc_temp()?;
                self.emit_load_atom_string(key_register, atom)?;
                self.emit_define_keyed_property(object, value, key_register)?;
            }
            Err(error) => return Err(error),
        }
        Ok(())
    }

    pub(super) fn emit_get_keyed_property(
        &mut self,
        dest: u16,
        object: u16,
        key: u16,
    ) -> LoweringResult<()> {
        let instruction_offset = self.builder.emit_abc(
            Opcode::GetKeyedProperty,
            self.encode_register(dest)?,
            self.encode_register(object)?,
            self.encode_register(key)?,
        )?;
        self.builder.add_feedback_site(
            instruction_offset,
            FeedbackSiteKind::KeyedPropertyAccess,
            FeedbackSiteMetadata::KeyedProperty,
        )?;
        Ok(())
    }

    pub(super) fn emit_set_keyed_property(
        &mut self,
        object: u16,
        value: u16,
        key: u16,
    ) -> LoweringResult<()> {
        let instruction_offset = self.builder.emit_abc(
            Opcode::SetKeyedProperty,
            self.encode_register(object)?,
            self.encode_register(value)?,
            self.encode_register(key)?,
        )?;
        self.builder.add_feedback_site(
            instruction_offset,
            FeedbackSiteKind::KeyedPropertyAccess,
            FeedbackSiteMetadata::KeyedProperty,
        )?;
        Ok(())
    }

    pub(super) fn emit_assign_keyed_property(
        &mut self,
        object: u16,
        value: u16,
        key: u16,
    ) -> LoweringResult<()> {
        let opcode = if self.force_strict_assignment {
            Opcode::StrictAssignKeyedProperty
        } else {
            Opcode::AssignKeyedProperty
        };
        let instruction_offset = self.builder.emit_abc(
            opcode,
            self.encode_register(object)?,
            self.encode_register(value)?,
            self.encode_register(key)?,
        )?;
        self.builder.add_feedback_site(
            instruction_offset,
            FeedbackSiteKind::KeyedPropertyAccess,
            FeedbackSiteMetadata::KeyedProperty,
        )?;
        Ok(())
    }

    pub(super) fn emit_define_keyed_property(
        &mut self,
        object: u16,
        value: u16,
        key: u16,
    ) -> LoweringResult<()> {
        self.builder.emit_abc(
            Opcode::DefineKeyedProperty,
            self.encode_register(object)?,
            self.encode_register(value)?,
            self.encode_register(key)?,
        )?;
        Ok(())
    }

    pub(super) fn emit_copy_data_properties(
        &mut self,
        target: u16,
        source: u16,
        excluded_keys: u16,
    ) -> LoweringResult<()> {
        self.builder.emit_abc(
            Opcode::CopyDataProperties,
            self.encode_register(target)?,
            self.encode_register(source)?,
            self.encode_register(excluded_keys)?,
        )?;
        Ok(())
    }

    pub(super) fn emit_set_function_name(
        &mut self,
        function: u16,
        name: u16,
    ) -> LoweringResult<()> {
        self.builder.emit_abc(
            Opcode::SetFunctionName,
            self.encode_register(function)?,
            self.encode_register(name)?,
            0,
        )?;
        Ok(())
    }

    pub(super) fn emit_to_property_key(&mut self, dest: u16, value: u16) -> LoweringResult<()> {
        self.builder.emit_abc(
            Opcode::ToPropertyKey,
            self.encode_register(dest)?,
            self.encode_register(value)?,
            0,
        )?;
        Ok(())
    }

    pub(super) fn emit_profiled_binary(
        &mut self,
        opcode: Opcode,
        dest: u16,
        left: u16,
        right: u16,
    ) -> LoweringResult<()> {
        let instruction_offset = self.builder.emit_abc(
            opcode,
            self.encode_register(dest)?,
            self.encode_register(left)?,
            self.encode_register(right)?,
        )?;
        let kind = match opcode {
            Opcode::Add
            | Opcode::Sub
            | Opcode::Mul
            | Opcode::Div
            | Opcode::Mod
            | Opcode::Exp
            | Opcode::BitOr
            | Opcode::BitXor
            | Opcode::BitAnd
            | Opcode::BitNot
            | Opcode::ShiftLeft
            | Opcode::ShiftRight
            | Opcode::UnsignedShiftRight => FeedbackSiteKind::Arithmetic,
            Opcode::Equal
            | Opcode::StrictEqual
            | Opcode::LessThan
            | Opcode::LessEqual
            | Opcode::GreaterThan
            | Opcode::GreaterEqual => FeedbackSiteKind::Comparison,
            _ => return Ok(()),
        };
        self.builder
            .add_feedback_site(instruction_offset, kind, FeedbackSiteMetadata::None)?;
        Ok(())
    }

    pub(super) const fn binary_opcode(operator: BinaryOp) -> LoweringResult<Opcode> {
        match operator {
            BinaryOp::Add => Ok(Opcode::Add),
            BinaryOp::Sub => Ok(Opcode::Sub),
            BinaryOp::Mul => Ok(Opcode::Mul),
            BinaryOp::Div => Ok(Opcode::Div),
            BinaryOp::Rem => Ok(Opcode::Mod),
            BinaryOp::Exp => Ok(Opcode::Exp),
            BinaryOp::BitOr => Ok(Opcode::BitOr),
            BinaryOp::BitXor => Ok(Opcode::BitXor),
            BinaryOp::BitAnd => Ok(Opcode::BitAnd),
            BinaryOp::Shl => Ok(Opcode::ShiftLeft),
            BinaryOp::Shr => Ok(Opcode::ShiftRight),
            BinaryOp::UShr => Ok(Opcode::UnsignedShiftRight),
            BinaryOp::Eq => Ok(Opcode::Equal),
            BinaryOp::StrictEq => Ok(Opcode::StrictEqual),
            BinaryOp::Lt => Ok(Opcode::LessThan),
            BinaryOp::LtEq => Ok(Opcode::LessEqual),
            BinaryOp::Gt => Ok(Opcode::GreaterThan),
            BinaryOp::GtEq => Ok(Opcode::GreaterEqual),
            _ => Err(LoweringError::UnsupportedExpression {
                expr: ExprId::new(0),
            }),
        }
    }

    pub(super) const fn assignment_opcode(operator: AssignOp) -> LoweringResult<Opcode> {
        match operator {
            AssignOp::AddAssign => Ok(Opcode::Add),
            AssignOp::SubAssign => Ok(Opcode::Sub),
            AssignOp::MulAssign => Ok(Opcode::Mul),
            AssignOp::DivAssign => Ok(Opcode::Div),
            AssignOp::RemAssign => Ok(Opcode::Mod),
            AssignOp::ExpAssign => Ok(Opcode::Exp),
            AssignOp::BitOrAssign => Ok(Opcode::BitOr),
            AssignOp::BitXorAssign => Ok(Opcode::BitXor),
            AssignOp::BitAndAssign => Ok(Opcode::BitAnd),
            AssignOp::ShlAssign => Ok(Opcode::ShiftLeft),
            AssignOp::ShrAssign => Ok(Opcode::ShiftRight),
            AssignOp::UShrAssign => Ok(Opcode::UnsignedShiftRight),
            _ => Err(LoweringError::UnsupportedExpression {
                expr: ExprId::new(0),
            }),
        }
    }

    pub(super) fn emit_logical_assignment_short_circuit(
        &mut self,
        operator: AssignOp,
        current: u16,
    ) -> LoweringResult<u32> {
        match operator {
            AssignOp::AndAssign => Ok(self
                .builder
                .emit_cond_jump_placeholder(Opcode::JumpIfFalse, self.encode_register(current)?)?),
            AssignOp::OrAssign => Ok(self
                .builder
                .emit_cond_jump_placeholder(Opcode::JumpIfTrue, self.encode_register(current)?)?),
            AssignOp::NullishAssign => {
                let null_value = self.alloc_temp()?;
                self.emit_load_null(null_value)?;
                let is_null = self.alloc_temp()?;
                self.emit_profiled_binary(Opcode::StrictEqual, is_null, current, null_value)?;
                let jump_assign_from_null = self.builder.emit_cond_jump_placeholder(
                    Opcode::JumpIfTrue,
                    self.encode_register(is_null)?,
                )?;

                let undefined_value = self.alloc_temp()?;
                self.emit_load_undefined(undefined_value)?;
                let is_undefined = self.alloc_temp()?;
                self.emit_profiled_binary(
                    Opcode::StrictEqual,
                    is_undefined,
                    current,
                    undefined_value,
                )?;
                let jump_end = self.builder.emit_cond_jump_placeholder(
                    Opcode::JumpIfFalse,
                    self.encode_register(is_undefined)?,
                )?;
                let assign_offset = self.builder.current_offset()?;
                self.builder
                    .patch_jump_to(jump_assign_from_null, assign_offset)?;
                Ok(jump_end)
            }
            _ => Err(LoweringError::UnsupportedExpression {
                expr: ExprId::new(0),
            }),
        }
    }

    pub(super) fn constant_atom(&mut self, atom: AtomId) -> LoweringResult<u32> {
        if let Some(index) = self.atom_constants.get(&atom) {
            return Ok(*index);
        }
        let index = self.builder.add_constant(ConstantValue::Atom(atom))?;
        self.state.record_atom_text(atom);
        self.atom_constants.insert(atom, index);
        Ok(index)
    }

    pub(super) fn constant_smi(&mut self, value: i32) -> LoweringResult<u32> {
        Ok(self.builder.add_constant(ConstantValue::Smi(value))?)
    }

    pub(super) fn constant_float(&mut self, bits: u64) -> LoweringResult<u32> {
        if let Some(index) = self.float_constants.get(&bits) {
            return Ok(*index);
        }
        let index = self
            .builder
            .add_constant(ConstantValue::Float64Bits(bits))?;
        self.float_constants.insert(bits, index);
        Ok(index)
    }

    pub(super) fn constant_builtin(&mut self, builtin: BuiltinFunctionId) -> LoweringResult<u32> {
        if let Some(index) = self.builtin_constants.get(&builtin) {
            return Ok(*index);
        }
        let index = self.builder.add_constant(ConstantValue::Builtin(builtin))?;
        self.builtin_constants.insert(builtin, index);
        Ok(index)
    }

    #[allow(
        clippy::unnecessary_wraps,
        clippy::unused_self,
        reason = "register encoding shares the fallible emission interface used by wide operands"
    )]
    pub(super) const fn encode_register(&self, register: u16) -> LoweringResult<u16> {
        Ok(register)
    }

    pub(super) fn encode_small_index(index: u32) -> LoweringResult<u16> {
        u16::try_from(index).map_err(|_| LoweringError::ConstantIndexOverflow { index })
    }

    pub(super) const fn ast(&self) -> &lyng_js_ast::Ast {
        self.state.program.ast
    }

    pub(super) const fn root_span(&self) -> Span {
        self.state.program.span
    }

    pub(super) fn record_source_map_span(&mut self, instruction_offset: u32, span: Span) {
        self.builder.add_source_map_entry(
            span.source,
            instruction_offset,
            span.range.start.raw(),
            span.range.end.raw(),
        );
    }

    pub(super) fn attach_safepoint(
        &mut self,
        instruction_offset: u32,
        span: Span,
        kind: SafepointKind,
    ) -> LoweringResult<u32> {
        self.record_source_map_span(instruction_offset, span);
        let register_window_len = self
            .builder
            .header()
            .register_count()
            .checked_add(self.builder.header().hidden_register_count())
            .ok_or(BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::FinalRegisterWindow,
            })?;
        let safepoint_id = self.builder.alloc_safepoint_id()?;
        let runtime_state = lyng_js_bytecode::RuntimeStateCapture::new()
            .with_lexical_env(true)
            .with_variable_env(true)
            .with_this_value(true)
            .with_new_target(true)
            .with_callee(true)
            .with_exception_state(matches!(kind, SafepointKind::ExceptionEdge))
            .with_completion_state(
                self.completion_registers.is_some() || matches!(kind, SafepointKind::ExceptionEdge),
            );
        let descriptor = lyng_js_bytecode::SafepointDescriptor::new(
            safepoint_id,
            instruction_offset,
            kind,
            register_window_len,
        )
        .with_runtime_state(runtime_state);
        self.builder.add_safepoint(descriptor);
        let snapshot = DeoptSnapshot::new(safepoint_id, self.default_deopt_values(kind));
        self.builder.add_deopt_snapshot(snapshot);
        Ok(safepoint_id)
    }

    fn default_deopt_values(&self, kind: SafepointKind) -> Vec<DeoptValueSource> {
        let mut values = vec![
            DeoptValueSource::FrameValue(DeoptFrameValue::ThisValue),
            DeoptValueSource::FrameValue(DeoptFrameValue::NewTarget),
            DeoptValueSource::FrameValue(DeoptFrameValue::Callee),
        ];
        if let Some(registers) = self.completion_registers {
            values.push(DeoptValueSource::Register(registers.kind));
            values.push(DeoptValueSource::Register(registers.value));
            values.push(DeoptValueSource::Register(registers.target));
        }
        if matches!(kind, SafepointKind::ExceptionEdge) {
            values.push(DeoptValueSource::FrameValue(
                DeoptFrameValue::ExceptionValue,
            ));
            values.push(DeoptValueSource::FrameValue(
                DeoptFrameValue::CompletionKind,
            ));
            values.push(DeoptValueSource::FrameValue(
                DeoptFrameValue::CompletionValue,
            ));
            values.push(DeoptValueSource::FrameValue(
                DeoptFrameValue::CompletionTarget,
            ));
        }
        values
    }
}

#[inline]
const fn encode_env_operand(depth: u8, slot: u32) -> u32 {
    ((depth as u32) << 24) | (slot & 0x00ff_ffff)
}

fn is_canonical_array_index_string(text: &str) -> bool {
    let Ok(index) = text.parse::<u64>() else {
        return false;
    };
    lyng_js_types::PropertyKey::from_array_index(index).is_some() && index.to_string() == text
}

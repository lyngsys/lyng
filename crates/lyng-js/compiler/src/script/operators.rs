use super::*;

impl<'a, 'b> FunctionCompiler<'a, 'b> {
    pub(super) fn lower_unary_expression(
        &mut self,
        _expr_id: ExprId,
        operator: lyng_js_ast::UnaryOp,
        argument: ExprId,
        dest: u16,
    ) -> LoweringResult<()> {
        match operator {
            lyng_js_ast::UnaryOp::Minus => {
                let argument_register = self.lower_expr_to_temp(argument)?;
                self.emit_profiled_negate(dest, argument_register)
            }
            lyng_js_ast::UnaryOp::Plus => {
                let argument_register = self.lower_expr_to_temp(argument)?;
                let zero = self.alloc_temp()?;
                self.emit_load_smi(zero, 0)?;
                self.emit_profiled_binary(Opcode::Sub, dest, argument_register, zero)
            }
            lyng_js_ast::UnaryOp::Not => {
                let argument_register = self.lower_expr_to_temp(argument)?;
                let jump_false = self.builder.emit_cond_jump_placeholder(
                    Opcode::JumpIfFalse,
                    self.encode_register(argument_register)?,
                )?;
                self.emit_load_bool(dest, false)?;
                let jump_end = self.builder.emit_jump_placeholder(Opcode::Jump)?;
                let true_offset = self.builder.current_offset()?;
                self.builder.patch_jump_to(jump_false, true_offset)?;
                self.emit_load_bool(dest, true)?;
                let end_offset = self.builder.current_offset()?;
                self.builder.patch_jump_to(jump_end, end_offset)?;
                Ok(())
            }
            lyng_js_ast::UnaryOp::BitNot => {
                let argument_register = self.lower_expr_to_temp(argument)?;
                let mask = self.alloc_temp()?;
                self.emit_load_smi(mask, -1)?;
                let int32_value = self.alloc_temp()?;
                self.emit_profiled_binary(Opcode::BitAnd, int32_value, argument_register, mask)?;
                let one = self.alloc_temp()?;
                self.emit_load_smi(one, 1)?;
                let incremented = self.alloc_temp()?;
                self.emit_profiled_binary(Opcode::Add, incremented, int32_value, one)?;
                self.emit_profiled_negate(dest, incremented)
            }
            lyng_js_ast::UnaryOp::Void => {
                let temp = self.alloc_temp()?;
                self.lower_expr_into(argument, temp)?;
                self.emit_load_undefined(dest)
            }
            lyng_js_ast::UnaryOp::TypeOf => {
                let mut current = argument;
                while let Expr::ParenthesizedExpression { expression, .. } =
                    self.ast().get_expr(current)
                {
                    current = *expression;
                }

                if let Expr::Identifier { name, .. } = self.ast().get_expr(current).clone() {
                    let use_site = self.use_site(current)?;
                    if matches!(use_site.resolution_kind, ResolutionKind::Dynamic) {
                        let index = self.constant_atom(name)?;
                        self.builder.emit_abx(
                            Opcode::ResolveName,
                            self.encode_register(dest)?,
                            index,
                        )?;
                        self.builder.emit_ax(Opcode::TypeOf, i32::from(dest))?;
                        return Ok(());
                    }
                    if matches!(
                        use_site.resolution_kind,
                        ResolutionKind::Global | ResolutionKind::Unresolved
                    ) {
                        let index = self.constant_atom(name)?;
                        self.builder.emit_abx(
                            Opcode::ResolveGlobal,
                            self.encode_register(dest)?,
                            index,
                        )?;
                        self.builder.emit_ax(Opcode::TypeOf, i32::from(dest))?;
                        return Ok(());
                    }
                }
                self.lower_expr_into(argument, dest)?;
                self.builder.emit_ax(Opcode::TypeOf, i32::from(dest))?;
                Ok(())
            }
            lyng_js_ast::UnaryOp::Delete => self.lower_delete_expression(argument, dest),
        }
    }

    pub(super) fn lower_binary_expression(
        &mut self,
        operator: BinaryOp,
        left: ExprId,
        right: ExprId,
        dest: u16,
    ) -> LoweringResult<()> {
        match operator {
            BinaryOp::Add
            | BinaryOp::Sub
            | BinaryOp::Mul
            | BinaryOp::Div
            | BinaryOp::Rem
            | BinaryOp::Exp
            | BinaryOp::BitOr
            | BinaryOp::BitXor
            | BinaryOp::BitAnd
            | BinaryOp::Shl
            | BinaryOp::Shr
            | BinaryOp::UShr
            | BinaryOp::Eq
            | BinaryOp::StrictEq
            | BinaryOp::Lt
            | BinaryOp::LtEq
            | BinaryOp::Gt
            | BinaryOp::GtEq => {
                let left_register = self.lower_expr_to_temp(left)?;
                let right_register = self.lower_expr_to_temp(right)?;
                self.emit_profiled_binary(
                    self.binary_opcode(operator)?,
                    dest,
                    left_register,
                    right_register,
                )
            }
            BinaryOp::NotEq => self.lower_inverted_binary(Opcode::Equal, left, right, dest),
            BinaryOp::StrictNotEq => {
                self.lower_inverted_binary(Opcode::StrictEqual, left, right, dest)
            }
            BinaryOp::In => {
                let left_register = self.lower_expr_to_temp(left)?;
                let right_register = self.lower_expr_to_temp(right)?;
                self.emit_profiled_binary(Opcode::In, dest, left_register, right_register)
            }
            BinaryOp::Instanceof => self.lower_internal_binary_builtin(
                internal_instance_of_builtin(),
                left,
                right,
                dest,
            ),
        }
    }

    pub(super) fn lower_inverted_binary(
        &mut self,
        opcode: Opcode,
        left: ExprId,
        right: ExprId,
        dest: u16,
    ) -> LoweringResult<()> {
        let left_register = self.lower_expr_to_temp(left)?;
        let right_register = self.lower_expr_to_temp(right)?;
        self.emit_profiled_binary(opcode, dest, left_register, right_register)?;
        let false_register = self.alloc_temp()?;
        self.emit_load_bool(false_register, false)?;
        self.builder.emit_abc(
            Opcode::StrictEqual,
            self.encode_register(dest)?,
            self.encode_register(dest)?,
            self.encode_register(false_register)?,
        )?;
        Ok(())
    }

    fn lower_internal_binary_builtin(
        &mut self,
        builtin: BuiltinFunctionId,
        left_expr: ExprId,
        right_expr: ExprId,
        dest: u16,
    ) -> LoweringResult<()> {
        let callee = self.alloc_temp()?;
        self.emit_load_builtin(callee, builtin)?;
        let this_value = self.alloc_temp()?;
        self.emit_load_undefined(this_value)?;
        let left = self.lower_expr_to_temp(left_expr)?;
        let right = self.lower_expr_to_temp(right_expr)?;
        let arguments = [left, right];
        let argument_range = self.materialize_argument_block(&arguments)?;
        let (call_result, call_callee, call_this, move_back) =
            self.bridge_call_registers(dest, callee, this_value)?;
        let instruction_offset = self.builder.emit_call(
            self.encode_register(call_result)?,
            self.encode_register(call_callee)?,
            self.encode_register(call_this)?,
            argument_range,
        )?;
        self.attach_safepoint(
            instruction_offset,
            self.ast()
                .get_expr(left_expr)
                .span()
                .cover(self.ast().get_expr(right_expr).span()),
            SafepointKind::Allocation,
        )?;
        if let Some(dest) = move_back {
            self.emit_move(dest, call_result)?;
        }
        Ok(())
    }

    pub(super) fn lower_logical_expression(
        &mut self,
        operator: lyng_js_ast::LogicalOp,
        left: ExprId,
        right: ExprId,
        dest: u16,
    ) -> LoweringResult<()> {
        self.lower_expr_into(left, dest)?;
        match operator {
            lyng_js_ast::LogicalOp::And | lyng_js_ast::LogicalOp::Or => {
                let short_circuit = match operator {
                    lyng_js_ast::LogicalOp::And => Opcode::JumpIfFalse,
                    lyng_js_ast::LogicalOp::Or => Opcode::JumpIfTrue,
                    lyng_js_ast::LogicalOp::NullishCoalescing => unreachable!(),
                };
                let jump_end = self
                    .builder
                    .emit_cond_jump_placeholder(short_circuit, self.encode_register(dest)?)?;
                self.lower_expr_into(right, dest)?;
                let end = self.builder.current_offset()?;
                self.builder.patch_jump_to(jump_end, end)?;
                Ok(())
            }
            lyng_js_ast::LogicalOp::NullishCoalescing => {
                let null_value = self.alloc_temp()?;
                self.emit_load_null(null_value)?;
                let is_null = self.alloc_temp()?;
                self.emit_profiled_binary(Opcode::StrictEqual, is_null, dest, null_value)?;
                let jump_right_from_null = self.builder.emit_cond_jump_placeholder(
                    Opcode::JumpIfTrue,
                    self.encode_register(is_null)?,
                )?;

                let undefined_value = self.alloc_temp()?;
                self.emit_load_undefined(undefined_value)?;
                let is_undefined = self.alloc_temp()?;
                self.emit_profiled_binary(
                    Opcode::StrictEqual,
                    is_undefined,
                    dest,
                    undefined_value,
                )?;
                let jump_end = self.builder.emit_cond_jump_placeholder(
                    Opcode::JumpIfFalse,
                    self.encode_register(is_undefined)?,
                )?;
                let right_offset = self.builder.current_offset()?;
                self.builder
                    .patch_jump_to(jump_right_from_null, right_offset)?;
                self.lower_expr_into(right, dest)?;
                let end = self.builder.current_offset()?;
                self.builder.patch_jump_to(jump_end, end)?;
                Ok(())
            }
        }
    }

    pub(super) fn lower_conditional_expression(
        &mut self,
        test: ExprId,
        consequent: ExprId,
        alternate: ExprId,
        dest: u16,
    ) -> LoweringResult<()> {
        let test_register = self.lower_expr_to_temp(test)?;
        let jump_alternate = self.builder.emit_cond_jump_placeholder(
            Opcode::JumpIfFalse,
            self.encode_register(test_register)?,
        )?;
        self.lower_expr_into(consequent, dest)?;
        let jump_end = self.builder.emit_jump_placeholder(Opcode::Jump)?;
        let alternate_offset = self.builder.current_offset()?;
        self.builder
            .patch_jump_to(jump_alternate, alternate_offset)?;
        self.lower_expr_into(alternate, dest)?;
        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_end, end)?;
        Ok(())
    }
}

use super::*;

struct OptionalNullishGuard {
    jump_end: u32,
}

impl<'a, 'b> FunctionCompiler<'a, 'b> {
    pub(super) fn expr_continues_optional_chain(&self, expr_id: ExprId) -> bool {
        match self.ast().get_expr(expr_id) {
            Expr::OptionalChainExpression { .. } => true,
            Expr::StaticMemberExpression { object, .. }
            | Expr::ComputedMemberExpression { object, .. }
            | Expr::PrivateMemberExpression { object, .. } => {
                self.expr_continues_optional_chain(*object)
            }
            Expr::CallExpression { callee, .. } => self.expr_continues_optional_chain(*callee),
            _ => false,
        }
    }

    pub(super) fn lower_optional_chain_expression(
        &mut self,
        expr_id: ExprId,
        dest: u16,
    ) -> LoweringResult<()> {
        let shorted = self.alloc_temp()?;
        self.emit_load_bool(shorted, false)?;
        self.lower_optional_chain_segment(expr_id, dest, shorted)
    }

    pub(super) fn lower_optional_chain_static_member_continuation(
        &mut self,
        object: ExprId,
        property: AtomId,
        dest: u16,
    ) -> LoweringResult<()> {
        let shorted = self.alloc_temp()?;
        self.emit_load_bool(shorted, false)?;
        self.lower_optional_chain_static_continuation_with_flag(object, property, dest, shorted)
    }

    pub(super) fn lower_optional_chain_computed_member_continuation(
        &mut self,
        object: ExprId,
        property: ExprId,
        dest: u16,
    ) -> LoweringResult<()> {
        let shorted = self.alloc_temp()?;
        self.emit_load_bool(shorted, false)?;
        self.lower_optional_chain_computed_continuation_with_flag(object, property, dest, shorted)
    }

    pub(super) fn lower_optional_chain_private_member_continuation(
        &mut self,
        expr_id: ExprId,
        object: ExprId,
        property: AtomId,
        dest: u16,
    ) -> LoweringResult<()> {
        let shorted = self.alloc_temp()?;
        self.emit_load_bool(shorted, false)?;
        self.lower_optional_chain_private_continuation_with_flag(
            expr_id, object, property, dest, shorted,
        )
    }

    pub(super) fn lower_optional_chain_call_continuation(
        &mut self,
        expr_id: ExprId,
        callee: ExprId,
        arguments: lyng_js_ast::NodeList<ExprId>,
        dest: u16,
    ) -> LoweringResult<()> {
        let shorted = self.alloc_temp()?;
        self.emit_load_bool(shorted, false)?;
        self.emit_load_undefined(dest)?;

        let callee_register = self.alloc_temp()?;
        let this_register = self.alloc_temp()?;
        self.lower_optional_chain_call_target(callee, callee_register, this_register, shorted)?;
        let jump_end = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfTrue, self.encode_register(shorted)?)?;
        self.emit_optional_call(expr_id, callee_register, this_register, arguments, dest)?;
        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_end, end)?;
        Ok(())
    }

    pub(super) fn lower_optional_chain_call_target(
        &mut self,
        callee: ExprId,
        callee_dest: u16,
        this_dest: u16,
        shorted: u16,
    ) -> LoweringResult<()> {
        match self.ast().get_expr(callee).clone() {
            Expr::ParenthesizedExpression { expression, .. } => {
                self.lower_optional_chain_call_target(expression, callee_dest, this_dest, shorted)
            }
            Expr::OptionalChainExpression { base, .. } => self
                .lower_optional_chain_optional_call_target(base, callee_dest, this_dest, shorted),
            Expr::StaticMemberExpression {
                object, property, ..
            } if self.expr_continues_optional_chain(object) => self
                .lower_optional_chain_static_call_target(
                    object,
                    property,
                    callee_dest,
                    this_dest,
                    shorted,
                ),
            Expr::ComputedMemberExpression {
                object, property, ..
            } if self.expr_continues_optional_chain(object) => self
                .lower_optional_chain_computed_call_target(
                    object,
                    property,
                    callee_dest,
                    this_dest,
                    shorted,
                ),
            Expr::PrivateMemberExpression {
                object, property, ..
            } if self.expr_continues_optional_chain(object) => self
                .lower_optional_chain_private_call_target(
                    callee,
                    object,
                    property,
                    callee_dest,
                    this_dest,
                    shorted,
                ),
            _ => {
                let (callee_register, this_register) = self.lower_call_target(callee)?;
                self.emit_move(callee_dest, callee_register)?;
                self.emit_move(this_dest, this_register)
            }
        }
    }

    fn lower_optional_chain_segment(
        &mut self,
        expr_id: ExprId,
        dest: u16,
        shorted: u16,
    ) -> LoweringResult<()> {
        match self.ast().get_expr(expr_id).clone() {
            Expr::OptionalChainExpression { base, .. } => {
                self.lower_optional_chain_optional_hop(base, dest, shorted)
            }
            Expr::StaticMemberExpression {
                object, property, ..
            } if self.expr_continues_optional_chain(object) => self
                .lower_optional_chain_static_continuation_with_flag(
                    object, property, dest, shorted,
                ),
            Expr::ComputedMemberExpression {
                object, property, ..
            } if self.expr_continues_optional_chain(object) => self
                .lower_optional_chain_computed_continuation_with_flag(
                    object, property, dest, shorted,
                ),
            Expr::PrivateMemberExpression {
                object, property, ..
            } if self.expr_continues_optional_chain(object) => self
                .lower_optional_chain_private_continuation_with_flag(
                    expr_id, object, property, dest, shorted,
                ),
            Expr::CallExpression {
                callee, arguments, ..
            } if self.expr_continues_optional_chain(callee) => self
                .lower_optional_chain_call_continuation_with_flag(
                    expr_id, callee, arguments, dest, shorted,
                ),
            _ => self.lower_expr_into(expr_id, dest),
        }
    }

    fn lower_optional_chain_optional_hop(
        &mut self,
        base: ExprId,
        dest: u16,
        shorted: u16,
    ) -> LoweringResult<()> {
        match self.ast().get_expr(base).clone() {
            Expr::StaticMemberExpression {
                object, property, ..
            } => self.lower_optional_static_member_hop(object, property, dest, shorted),
            Expr::ComputedMemberExpression {
                object, property, ..
            } => self.lower_optional_computed_member_hop(object, property, dest, shorted),
            Expr::PrivateMemberExpression {
                object, property, ..
            } => self.lower_optional_private_member_hop(base, object, property, dest, shorted),
            Expr::CallExpression {
                callee, arguments, ..
            } => self.lower_optional_call_hop(base, callee, arguments, dest, shorted),
            _ => self.lower_optional_chain_segment(base, dest, shorted),
        }
    }

    fn lower_optional_static_member_hop(
        &mut self,
        object: ExprId,
        property: AtomId,
        dest: u16,
        shorted: u16,
    ) -> LoweringResult<()> {
        let receiver = self.alloc_temp()?;
        self.lower_optional_chain_operand_value(object, receiver, shorted)?;
        let guard = self.emit_optional_nullish_guard(receiver, dest, shorted)?;
        self.emit_get_property_by_atom(dest, receiver, property)?;
        self.finish_optional_nullish_guard(guard)?;
        Ok(())
    }

    fn lower_optional_computed_member_hop(
        &mut self,
        object: ExprId,
        property: ExprId,
        dest: u16,
        shorted: u16,
    ) -> LoweringResult<()> {
        let receiver = self.alloc_temp()?;
        self.lower_optional_chain_operand_value(object, receiver, shorted)?;
        let guard = self.emit_optional_nullish_guard(receiver, dest, shorted)?;
        let key = self.lower_expr_to_temp(property)?;
        self.emit_get_keyed_property(dest, receiver, key)?;
        self.finish_optional_nullish_guard(guard)?;
        Ok(())
    }

    fn lower_optional_private_member_hop(
        &mut self,
        expr_id: ExprId,
        object: ExprId,
        property: AtomId,
        dest: u16,
        shorted: u16,
    ) -> LoweringResult<()> {
        let receiver = self.alloc_temp()?;
        self.lower_optional_chain_operand_value(object, receiver, shorted)?;
        let guard = self.emit_optional_nullish_guard(receiver, dest, shorted)?;
        let span = self.ast().get_expr(object).span();
        self.emit_private_field_get_from_receiver(expr_id, receiver, property, span, dest)?;
        self.finish_optional_nullish_guard(guard)?;
        Ok(())
    }

    fn lower_optional_call_hop(
        &mut self,
        expr_id: ExprId,
        callee: ExprId,
        arguments: lyng_js_ast::NodeList<ExprId>,
        dest: u16,
        shorted: u16,
    ) -> LoweringResult<()> {
        let callee_register = self.alloc_temp()?;
        let this_register = self.alloc_temp()?;
        self.lower_optional_chain_call_target(callee, callee_register, this_register, shorted)?;
        let guard = self.emit_optional_nullish_guard(callee_register, dest, shorted)?;
        self.emit_optional_call(expr_id, callee_register, this_register, arguments, dest)?;
        self.finish_optional_nullish_guard(guard)?;
        Ok(())
    }

    fn lower_optional_chain_static_continuation_with_flag(
        &mut self,
        object: ExprId,
        property: AtomId,
        dest: u16,
        shorted: u16,
    ) -> LoweringResult<()> {
        self.emit_load_undefined(dest)?;
        let receiver = self.alloc_temp()?;
        self.lower_optional_chain_segment(object, receiver, shorted)?;
        let jump_end = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfTrue, self.encode_register(shorted)?)?;
        self.emit_get_property_by_atom(dest, receiver, property)?;
        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_end, end)?;
        Ok(())
    }

    fn lower_optional_chain_computed_continuation_with_flag(
        &mut self,
        object: ExprId,
        property: ExprId,
        dest: u16,
        shorted: u16,
    ) -> LoweringResult<()> {
        self.emit_load_undefined(dest)?;
        let receiver = self.alloc_temp()?;
        self.lower_optional_chain_segment(object, receiver, shorted)?;
        let jump_end = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfTrue, self.encode_register(shorted)?)?;
        let key = self.lower_expr_to_temp(property)?;
        self.emit_get_keyed_property(dest, receiver, key)?;
        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_end, end)?;
        Ok(())
    }

    fn lower_optional_chain_private_continuation_with_flag(
        &mut self,
        expr_id: ExprId,
        object: ExprId,
        property: AtomId,
        dest: u16,
        shorted: u16,
    ) -> LoweringResult<()> {
        self.emit_load_undefined(dest)?;
        let receiver = self.alloc_temp()?;
        self.lower_optional_chain_segment(object, receiver, shorted)?;
        let jump_end = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfTrue, self.encode_register(shorted)?)?;
        let span = self.ast().get_expr(object).span();
        self.emit_private_field_get_from_receiver(expr_id, receiver, property, span, dest)?;
        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_end, end)?;
        Ok(())
    }

    fn lower_optional_chain_call_continuation_with_flag(
        &mut self,
        expr_id: ExprId,
        callee: ExprId,
        arguments: lyng_js_ast::NodeList<ExprId>,
        dest: u16,
        shorted: u16,
    ) -> LoweringResult<()> {
        self.emit_load_undefined(dest)?;
        let callee_register = self.alloc_temp()?;
        let this_register = self.alloc_temp()?;
        self.lower_optional_chain_call_target(callee, callee_register, this_register, shorted)?;
        let jump_end = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfTrue, self.encode_register(shorted)?)?;
        self.emit_optional_call(expr_id, callee_register, this_register, arguments, dest)?;
        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_end, end)?;
        Ok(())
    }

    fn lower_optional_chain_optional_call_target(
        &mut self,
        base: ExprId,
        callee_dest: u16,
        this_dest: u16,
        shorted: u16,
    ) -> LoweringResult<()> {
        match self.ast().get_expr(base).clone() {
            Expr::StaticMemberExpression {
                object, property, ..
            } => {
                let receiver = self.alloc_temp()?;
                self.lower_optional_chain_operand_value(object, receiver, shorted)?;
                let guard = self.emit_optional_nullish_guard(receiver, callee_dest, shorted)?;
                self.emit_get_property_by_atom(callee_dest, receiver, property)?;
                self.emit_move(this_dest, receiver)?;
                self.finish_optional_nullish_guard(guard)?;
                Ok(())
            }
            Expr::ComputedMemberExpression {
                object, property, ..
            } => {
                let receiver = self.alloc_temp()?;
                self.lower_optional_chain_operand_value(object, receiver, shorted)?;
                let guard = self.emit_optional_nullish_guard(receiver, callee_dest, shorted)?;
                let key = self.lower_expr_to_temp(property)?;
                self.emit_get_keyed_property(callee_dest, receiver, key)?;
                self.emit_move(this_dest, receiver)?;
                self.finish_optional_nullish_guard(guard)?;
                Ok(())
            }
            Expr::PrivateMemberExpression {
                object, property, ..
            } => {
                let receiver = self.alloc_temp()?;
                self.lower_optional_chain_operand_value(object, receiver, shorted)?;
                let guard = self.emit_optional_nullish_guard(receiver, callee_dest, shorted)?;
                let span = self.ast().get_expr(object).span();
                self.emit_private_field_get_from_receiver(
                    base,
                    receiver,
                    property,
                    span,
                    callee_dest,
                )?;
                self.emit_move(this_dest, receiver)?;
                self.finish_optional_nullish_guard(guard)?;
                Ok(())
            }
            Expr::CallExpression {
                callee, arguments, ..
            } => {
                self.lower_optional_call_hop(base, callee, arguments, callee_dest, shorted)?;
                self.emit_load_undefined(this_dest)
            }
            _ => {
                self.lower_optional_chain_segment(base, callee_dest, shorted)?;
                self.emit_load_undefined(this_dest)
            }
        }
    }

    fn lower_optional_chain_static_call_target(
        &mut self,
        object: ExprId,
        property: AtomId,
        callee_dest: u16,
        this_dest: u16,
        shorted: u16,
    ) -> LoweringResult<()> {
        self.emit_load_undefined(callee_dest)?;
        let receiver = self.alloc_temp()?;
        self.lower_optional_chain_segment(object, receiver, shorted)?;
        let jump_end = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfTrue, self.encode_register(shorted)?)?;
        self.emit_get_property_by_atom(callee_dest, receiver, property)?;
        self.emit_move(this_dest, receiver)?;
        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_end, end)?;
        Ok(())
    }

    fn lower_optional_chain_computed_call_target(
        &mut self,
        object: ExprId,
        property: ExprId,
        callee_dest: u16,
        this_dest: u16,
        shorted: u16,
    ) -> LoweringResult<()> {
        self.emit_load_undefined(callee_dest)?;
        let receiver = self.alloc_temp()?;
        self.lower_optional_chain_segment(object, receiver, shorted)?;
        let jump_end = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfTrue, self.encode_register(shorted)?)?;
        let key = self.lower_expr_to_temp(property)?;
        self.emit_get_keyed_property(callee_dest, receiver, key)?;
        self.emit_move(this_dest, receiver)?;
        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_end, end)?;
        Ok(())
    }

    fn lower_optional_chain_private_call_target(
        &mut self,
        expr_id: ExprId,
        object: ExprId,
        property: AtomId,
        callee_dest: u16,
        this_dest: u16,
        shorted: u16,
    ) -> LoweringResult<()> {
        self.emit_load_undefined(callee_dest)?;
        let receiver = self.alloc_temp()?;
        self.lower_optional_chain_segment(object, receiver, shorted)?;
        let jump_end = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfTrue, self.encode_register(shorted)?)?;
        let span = self.ast().get_expr(object).span();
        self.emit_private_field_get_from_receiver(expr_id, receiver, property, span, callee_dest)?;
        self.emit_move(this_dest, receiver)?;
        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_end, end)?;
        Ok(())
    }

    fn lower_optional_chain_operand_value(
        &mut self,
        expr_id: ExprId,
        dest: u16,
        shorted: u16,
    ) -> LoweringResult<()> {
        if self.expr_continues_optional_chain(expr_id) {
            self.lower_optional_chain_segment(expr_id, dest, shorted)
        } else {
            self.lower_expr_into(expr_id, dest)
        }
    }

    fn emit_optional_nullish_guard(
        &mut self,
        value: u16,
        dest: u16,
        shorted: u16,
    ) -> LoweringResult<OptionalNullishGuard> {
        let null_value = self.alloc_temp()?;
        self.emit_load_null(null_value)?;
        let is_null = self.alloc_temp()?;
        self.emit_profiled_binary(Opcode::StrictEqual, is_null, value, null_value)?;
        let jump_short_from_null = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfTrue, self.encode_register(is_null)?)?;

        let undefined_value = self.alloc_temp()?;
        self.emit_load_undefined(undefined_value)?;
        let is_undefined = self.alloc_temp()?;
        self.emit_profiled_binary(Opcode::StrictEqual, is_undefined, value, undefined_value)?;
        let jump_continue = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfFalse, self.encode_register(is_undefined)?)?;

        let short_offset = self.builder.current_offset()?;
        self.builder
            .patch_jump_to(jump_short_from_null, short_offset)?;
        self.emit_load_undefined(dest)?;
        self.emit_load_bool(shorted, true)?;
        let jump_end = self.builder.emit_jump_placeholder(Opcode::Jump)?;

        let continue_offset = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_continue, continue_offset)?;
        Ok(OptionalNullishGuard { jump_end })
    }

    fn finish_optional_nullish_guard(&mut self, guard: OptionalNullishGuard) -> LoweringResult<()> {
        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(guard.jump_end, end)?;
        Ok(())
    }

    fn emit_optional_call(
        &mut self,
        expr_id: ExprId,
        callee_register: u16,
        this_register: u16,
        arguments: lyng_js_ast::NodeList<ExprId>,
        dest: u16,
    ) -> LoweringResult<()> {
        let argument_values = self.lower_call_arguments(arguments)?;
        let argument_range = self.materialize_argument_block(&argument_values.registers)?;
        let (call_result, call_callee, call_this, move_back) =
            self.bridge_call_registers(dest, callee_register, this_register)?;
        let instruction_offset = self.builder.emit_call(
            self.encode_register(call_result)?,
            self.encode_register(call_callee)?,
            self.encode_register(call_this)?,
            argument_range,
        )?;
        let span = self.ast().get_expr(expr_id).span();
        self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation)?;
        self.builder.add_feedback_site(
            instruction_offset,
            FeedbackSiteKind::Call,
            self.call_feedback_metadata(
                argument_range.argument_count(),
                argument_values.spread_mask,
            ),
        )?;
        if let Some(dest) = move_back {
            self.emit_move(dest, call_result)?;
        }
        Ok(())
    }
}

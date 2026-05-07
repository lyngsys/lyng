use super::{
    internal_get_template_object_builtin, internal_template_to_string_builtin, ExprId,
    FeedbackSiteKind, FeedbackSiteMetadata, FunctionCompiler, LoweringError, LoweringResult,
    Opcode, SafepointKind, Span,
};

impl FunctionCompiler<'_, '_> {
    pub(super) fn lower_template_literal(
        &mut self,
        expr_id: ExprId,
        template: lyng_js_ast::TemplateLiteralId,
        dest: u16,
    ) -> LoweringResult<()> {
        let quasis = self.ast().templates().get_quasis(template).to_vec();
        let expressions = self.ast().templates().get_expressions(template).to_vec();
        debug_assert_eq!(quasis.len(), expressions.len() + 1);

        let first = quasis
            .first()
            .and_then(|quasi| quasi.cooked)
            .ok_or(LoweringError::UnsupportedExpression { expr: expr_id })?;
        self.emit_load_string(dest, first)?;

        for (index, expression) in expressions.iter().copied().enumerate() {
            let value = self.lower_expr_to_temp(expression)?;
            let coerced = self.lower_internal_template_to_string(expr_id, value)?;
            self.emit_profiled_binary(Opcode::Add, dest, dest, coerced)?;

            let Some(quasi) = quasis.get(index + 1) else {
                continue;
            };
            let Some(cooked) = quasi.cooked else {
                return Err(LoweringError::UnsupportedExpression { expr: expr_id });
            };
            if self.ast().literals().string_is_empty(cooked) {
                continue;
            }
            let text = self.alloc_temp()?;
            self.emit_load_string(text, cooked)?;
            self.emit_profiled_binary(Opcode::Add, dest, dest, text)?;
        }

        Ok(())
    }

    pub(super) fn lower_tagged_template_expression(
        &mut self,
        expr_id: ExprId,
        tag: ExprId,
        template: lyng_js_ast::TemplateLiteralId,
        dest: u16,
    ) -> LoweringResult<()> {
        self.lower_tagged_template_call(expr_id, tag, template, Some(dest))
    }

    pub(super) fn lower_tail_tagged_template_expression(
        &mut self,
        expr_id: ExprId,
        tag: ExprId,
        template: lyng_js_ast::TemplateLiteralId,
    ) -> LoweringResult<()> {
        self.lower_tagged_template_call(expr_id, tag, template, None)
    }

    fn lower_tagged_template_call(
        &mut self,
        expr_id: ExprId,
        tag: ExprId,
        template: lyng_js_ast::TemplateLiteralId,
        dest: Option<u16>,
    ) -> LoweringResult<()> {
        let span = self.ast().get_expr(expr_id).span();
        let (callee_register, this_register) = self.lower_call_target(tag)?;
        let template_object = self.lower_template_object(span, template)?;
        let expressions = self.ast().templates().get_expressions(template).to_vec();
        let mut argument_values = Vec::with_capacity(expressions.len() + 1);
        argument_values.push(template_object);
        for expression in expressions {
            argument_values.push(self.lower_expr_to_temp(expression)?);
        }
        let argument_range = self.materialize_argument_block(&argument_values)?;

        if let Some(dest) = dest {
            let (call_result, call_callee, call_this, move_back) =
                self.bridge_call_registers(dest, callee_register, this_register)?;
            let instruction_offset = self.builder.emit_call(
                self.encode_register(call_result)?,
                self.encode_register(call_callee)?,
                self.encode_register(call_this)?,
                argument_range,
            )?;
            self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation)?;
            self.builder.add_feedback_site(
                instruction_offset,
                FeedbackSiteKind::Call,
                FeedbackSiteMetadata::ExpectedArity(argument_range.argument_count()),
            )?;
            if let Some(dest) = move_back {
                self.emit_move(dest, call_result)?;
            }
            return Ok(());
        }

        let (tail_callee, tail_this) =
            self.bridge_tail_call_registers(callee_register, this_register)?;
        let instruction_offset = self.builder.emit_tail_call(
            self.encode_register(tail_callee)?,
            self.encode_register(tail_this)?,
            argument_range,
        )?;
        self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation)?;
        self.builder.add_feedback_site(
            instruction_offset,
            FeedbackSiteKind::Call,
            FeedbackSiteMetadata::ExpectedArity(argument_range.argument_count()),
        )?;
        Ok(())
    }

    fn lower_template_object(
        &mut self,
        span: Span,
        template: lyng_js_ast::TemplateLiteralId,
    ) -> LoweringResult<u16> {
        let quasis = self.ast().templates().get_quasis(template).to_vec();
        let callee = self.alloc_temp()?;
        self.emit_load_builtin(callee, internal_get_template_object_builtin())?;
        let this_value = self.alloc_temp()?;
        self.emit_load_undefined(this_value)?;

        let site = self.alloc_temp()?;
        self.emit_load_i32_constant(site, i32::try_from(template.raw()).unwrap_or(i32::MAX))?;

        let mut arguments = Vec::with_capacity(1 + quasis.len() * 2);
        arguments.push(site);
        for quasi in quasis {
            let cooked = self.alloc_temp()?;
            if let Some(cooked_id) = quasi.cooked {
                self.emit_load_string(cooked, cooked_id)?;
            } else {
                self.emit_load_undefined(cooked)?;
            }
            arguments.push(cooked);

            let raw = self.alloc_temp()?;
            self.emit_load_string(raw, quasi.raw)?;
            arguments.push(raw);
        }

        let argument_range = self.materialize_argument_block(&arguments)?;
        let result = self.alloc_temp()?;
        let (call_result, call_callee, call_this, move_back) =
            self.bridge_call_registers(result, callee, this_value)?;
        let instruction_offset = self.builder.emit_call(
            self.encode_register(call_result)?,
            self.encode_register(call_callee)?,
            self.encode_register(call_this)?,
            argument_range,
        )?;
        self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation)?;
        if let Some(dest) = move_back {
            self.emit_move(dest, call_result)?;
        }
        Ok(result)
    }

    fn lower_internal_template_to_string(
        &mut self,
        expr_id: ExprId,
        value: u16,
    ) -> LoweringResult<u16> {
        let callee = self.alloc_temp()?;
        self.emit_load_builtin(callee, internal_template_to_string_builtin())?;
        let this_value = self.alloc_temp()?;
        self.emit_load_undefined(this_value)?;
        let arguments = [value];
        let argument_range = self.materialize_argument_block(&arguments)?;
        let result = self.alloc_temp()?;
        let (call_result, call_callee, call_this, move_back) =
            self.bridge_call_registers(result, callee, this_value)?;
        let instruction_offset = self.builder.emit_call(
            self.encode_register(call_result)?,
            self.encode_register(call_callee)?,
            self.encode_register(call_this)?,
            argument_range,
        )?;
        let span = self.ast().get_expr(expr_id).span();
        self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation)?;
        if let Some(dest) = move_back {
            self.emit_move(dest, call_result)?;
        }
        Ok(result)
    }
}

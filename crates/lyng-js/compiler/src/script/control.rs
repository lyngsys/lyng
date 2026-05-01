use super::*;

impl<'a, 'b> FunctionCompiler<'a, 'b> {
    pub(super) fn lower_return_statement(
        &mut self,
        argument: Option<ExprId>,
    ) -> LoweringResult<()> {
        let is_async_generator = self.current_function.is_some_and(|function| {
            self.state.function_kind(function) == FunctionKind::AsyncGenerator
        });
        if let Some(finally_index) = self.nearest_active_finally() {
            let value = if let Some(argument) = argument {
                let value = self.lower_expr_to_temp(argument)?;
                if is_async_generator {
                    self.builder.emit_ax(Opcode::Await, i32::from(value))?;
                }
                value
            } else {
                let value = self.alloc_temp()?;
                self.emit_load_undefined(value)?;
                value
            };
            self.set_completion_state(CompletionKind::Return, Some(value), None)?;
            self.emit_jump_to_finally(finally_index)?;
            return Ok(());
        }

        let Some(argument) = argument else {
            self.builder.emit_ax(Opcode::ReturnUndefined, 0)?;
            return Ok(());
        };

        if is_async_generator {
            let value = self.lower_expr_to_temp(argument)?;
            self.builder.emit_ax(Opcode::Await, i32::from(value))?;
            self.builder.emit_ax(Opcode::Return, i32::from(value))?;
            return Ok(());
        }

        self.lower_tail_return_expression(argument)
    }

    pub(super) fn lower_tail_return_expression(&mut self, expr: ExprId) -> LoweringResult<()> {
        match self.ast().get_expr(expr).clone() {
            Expr::ParenthesizedExpression { expression, .. } => {
                self.lower_tail_return_expression(expression)
            }
            Expr::CallExpression {
                callee, arguments, ..
            } => self.lower_tail_call_expression(expr, callee, arguments),
            Expr::TaggedTemplateExpression { tag, template, .. } => {
                self.lower_tail_tagged_template_expression(expr, tag, template)
            }
            Expr::ConditionalExpression {
                test,
                consequent,
                alternate,
                ..
            } => self.lower_tail_conditional_expression(test, consequent, alternate),
            Expr::LogicalExpression {
                operator,
                left,
                right,
                ..
            } => self.lower_tail_logical_expression(operator, left, right),
            Expr::SequenceExpression { expressions, .. } => {
                self.lower_tail_sequence_expression(expressions)
            }
            _ => {
                let value = self.lower_expr_to_temp(expr)?;
                self.builder.emit_ax(Opcode::Return, i32::from(value))?;
                Ok(())
            }
        }
    }

    pub(super) fn lower_tail_conditional_expression(
        &mut self,
        test: ExprId,
        consequent: ExprId,
        alternate: ExprId,
    ) -> LoweringResult<()> {
        let test_register = self.lower_expr_to_temp(test)?;
        let jump_alternate = self.builder.emit_cond_jump_placeholder(
            Opcode::JumpIfFalse,
            self.encode_register(test_register)?,
        )?;
        self.lower_tail_return_expression(consequent)?;
        let alternate_offset = self.builder.current_offset()?;
        self.builder
            .patch_jump_to(jump_alternate, alternate_offset)?;
        self.lower_tail_return_expression(alternate)
    }

    pub(super) fn lower_tail_logical_expression(
        &mut self,
        operator: lyng_js_ast::LogicalOp,
        left: ExprId,
        right: ExprId,
    ) -> LoweringResult<()> {
        let left_register = self.lower_expr_to_temp(left)?;
        match operator {
            lyng_js_ast::LogicalOp::And | lyng_js_ast::LogicalOp::Or => {
                let short_circuit = match operator {
                    lyng_js_ast::LogicalOp::And => Opcode::JumpIfFalse,
                    lyng_js_ast::LogicalOp::Or => Opcode::JumpIfTrue,
                    lyng_js_ast::LogicalOp::NullishCoalescing => unreachable!(),
                };
                let jump_short = self.builder.emit_cond_jump_placeholder(
                    short_circuit,
                    self.encode_register(left_register)?,
                )?;
                self.lower_tail_return_expression(right)?;
                let short_offset = self.builder.current_offset()?;
                self.builder.patch_jump_to(jump_short, short_offset)?;
                self.builder
                    .emit_ax(Opcode::Return, i32::from(left_register))?;
                Ok(())
            }
            lyng_js_ast::LogicalOp::NullishCoalescing => {
                let null_value = self.alloc_temp()?;
                self.emit_load_null(null_value)?;
                let is_null = self.alloc_temp()?;
                self.emit_profiled_binary(Opcode::StrictEqual, is_null, left_register, null_value)?;
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
                    left_register,
                    undefined_value,
                )?;
                let jump_return_left = self.builder.emit_cond_jump_placeholder(
                    Opcode::JumpIfFalse,
                    self.encode_register(is_undefined)?,
                )?;
                let right_offset = self.builder.current_offset()?;
                self.builder
                    .patch_jump_to(jump_right_from_null, right_offset)?;
                self.lower_tail_return_expression(right)?;
                let return_left_offset = self.builder.current_offset()?;
                self.builder
                    .patch_jump_to(jump_return_left, return_left_offset)?;
                self.builder
                    .emit_ax(Opcode::Return, i32::from(left_register))?;
                Ok(())
            }
        }
    }

    pub(super) fn lower_tail_sequence_expression(
        &mut self,
        expressions: lyng_js_ast::NodeList<ExprId>,
    ) -> LoweringResult<()> {
        let expressions = self.ast().get_expr_list(expressions).to_vec();
        let Some((last, rest)) = expressions.split_last() else {
            self.builder.emit_ax(Opcode::ReturnUndefined, 0)?;
            return Ok(());
        };
        for expr in rest {
            let temp = self.alloc_temp()?;
            self.lower_expr_into(*expr, temp)?;
        }
        self.lower_tail_return_expression(*last)
    }

    pub(super) fn lower_break_statement(&mut self, label: Option<AtomId>) -> LoweringResult<()> {
        let target =
            self.resolve_break_target(label)
                .ok_or(LoweringError::UnsupportedStatement {
                    stmt: StmtId::new(0),
                })?;
        self.lower_control_transfer(CompletionKind::Break, target)
    }

    pub(super) fn lower_continue_statement(&mut self, label: Option<AtomId>) -> LoweringResult<()> {
        let target =
            self.resolve_continue_target(label)
                .ok_or(LoweringError::UnsupportedStatement {
                    stmt: StmtId::new(0),
                })?;
        self.lower_control_transfer(CompletionKind::Continue, target)
    }

    pub(super) fn lower_control_transfer(
        &mut self,
        kind: CompletionKind,
        target: usize,
    ) -> LoweringResult<()> {
        if let Some(finally_index) = self.nearest_active_finally() {
            let target_id = self.control_targets[target].id;
            self.set_completion_state(kind, self.result_register, Some(target_id))?;
            self.emit_jump_to_finally(finally_index)?;
            return Ok(());
        }

        let placeholder = self.builder.emit_jump_placeholder(Opcode::Jump)?;
        match kind {
            CompletionKind::Break => self.control_targets[target]
                .break_placeholders
                .push(placeholder),
            CompletionKind::Continue => {
                self.control_targets[target]
                    .continue_placeholders
                    .push(placeholder);
            }
            _ => unreachable!("only break and continue use control-transfer targets"),
        }
        Ok(())
    }

    pub(super) fn lower_try_statement(
        &mut self,
        block: StmtId,
        handler: Option<CatchClause>,
        finalizer: Option<StmtId>,
    ) -> LoweringResult<()> {
        self.reset_statement_result()?;
        match (handler, finalizer) {
            (Some(handler), Some(finalizer)) => {
                self.lower_try_catch_finally(block, handler, finalizer)
            }
            (Some(handler), None) => self.lower_try_catch(block, handler),
            (None, Some(finalizer)) => self.lower_try_finally(block, finalizer),
            (None, None) => Err(LoweringError::UnsupportedStatement { stmt: block }),
        }
    }

    pub(super) fn lower_try_catch(
        &mut self,
        block: StmtId,
        handler: CatchClause,
    ) -> LoweringResult<()> {
        let protected_start = self.builder.current_offset()?;
        self.lower_statement(block)?;
        let jump_end = self.builder.emit_jump_placeholder(Opcode::Jump)?;
        let protected_end = self.builder.current_offset()?;
        let catch_entry = self.builder.current_offset()?;
        let enter_handler = self.emit_enter_handler()?;
        let catch_span = self.ast().get_stmt(handler.body).span();
        self.attach_safepoint(enter_handler, catch_span, SafepointKind::ExceptionEdge)?;
        self.lower_catch_clause(handler)?;
        self.emit_leave_handler()?;
        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_end, end)?;
        self.builder.add_exception_handler(ExceptionHandler::new(
            protected_start,
            protected_end,
            catch_entry,
            ExceptionHandlerKind::Catch,
            self.builder.header().register_count(),
            None,
        ))?;
        Ok(())
    }

    pub(super) fn lower_try_finally(
        &mut self,
        block: StmtId,
        finalizer: StmtId,
    ) -> LoweringResult<()> {
        let finally_index = self.push_finally_context();
        let protected_start = self.builder.current_offset()?;
        self.lower_statement(block)?;
        let protected_end = self.builder.current_offset()?;
        self.set_completion_state(CompletionKind::Normal, self.result_register, None)?;
        self.emit_jump_to_finally(finally_index)?;

        let throw_entry = self.builder.current_offset()?;
        let enter_handler = self.emit_enter_handler()?;
        let finalizer_span = self.ast().get_stmt(finalizer).span();
        self.attach_safepoint(enter_handler, finalizer_span, SafepointKind::ExceptionEdge)?;
        self.begin_exception_finally_path()?;
        let normal_entry = self.builder.current_offset()?;
        self.set_finally_normal_entry(finally_index, normal_entry)?;
        self.mark_finally_body(finally_index, true);
        self.reset_statement_result()?;
        self.lower_statement(finalizer)?;
        self.emit_leave_handler()?;
        self.emit_finally_dispatch(finally_index)?;
        self.mark_finally_body(finally_index, false);
        self.pop_finally_context(finally_index);

        self.builder.add_exception_handler(ExceptionHandler::new(
            protected_start,
            protected_end,
            throw_entry,
            ExceptionHandlerKind::Finally,
            self.builder.header().register_count(),
            None,
        ))?;
        Ok(())
    }

    pub(super) fn lower_try_catch_finally(
        &mut self,
        block: StmtId,
        handler: CatchClause,
        finalizer: StmtId,
    ) -> LoweringResult<()> {
        let finally_index = self.push_finally_context();
        let try_start = self.builder.current_offset()?;
        self.lower_statement(block)?;
        let try_end = self.builder.current_offset()?;
        self.set_completion_state(CompletionKind::Normal, self.result_register, None)?;
        self.emit_jump_to_finally(finally_index)?;

        let catch_entry = self.builder.current_offset()?;
        let catch_enter = self.emit_enter_handler()?;
        let catch_span = self.ast().get_stmt(handler.body).span();
        self.attach_safepoint(catch_enter, catch_span, SafepointKind::ExceptionEdge)?;
        self.lower_catch_clause(handler)?;
        self.emit_leave_handler()?;
        let catch_end = self.builder.current_offset()?;
        self.set_completion_state(CompletionKind::Normal, self.result_register, None)?;
        self.emit_jump_to_finally(finally_index)?;

        let throw_entry = self.builder.current_offset()?;
        let finally_enter = self.emit_enter_handler()?;
        let finalizer_span = self.ast().get_stmt(finalizer).span();
        self.attach_safepoint(finally_enter, finalizer_span, SafepointKind::ExceptionEdge)?;
        self.begin_exception_finally_path()?;
        let normal_entry = self.builder.current_offset()?;
        self.set_finally_normal_entry(finally_index, normal_entry)?;
        self.mark_finally_body(finally_index, true);
        self.reset_statement_result()?;
        self.lower_statement(finalizer)?;
        self.emit_leave_handler()?;
        self.emit_finally_dispatch(finally_index)?;
        self.mark_finally_body(finally_index, false);
        self.pop_finally_context(finally_index);

        self.builder.add_exception_handler(ExceptionHandler::new(
            try_start,
            try_end,
            catch_entry,
            ExceptionHandlerKind::Catch,
            self.builder.header().register_count(),
            None,
        ))?;
        self.builder.add_exception_handler(ExceptionHandler::new(
            catch_entry,
            catch_end,
            throw_entry,
            ExceptionHandlerKind::Finally,
            self.builder.header().register_count(),
            None,
        ))?;
        Ok(())
    }

    pub(super) fn lower_catch_clause(&mut self, handler: CatchClause) -> LoweringResult<()> {
        self.with_child_scope(ScopeKind::Catch, true, handler.body, |this| {
            if let Some(pattern) = handler.param {
                let value = this.alloc_temp()?;
                this.builder
                    .emit_ax(Opcode::LoadException, i32::from(value))?;
                this.lower_binding_pattern_initialization(
                    pattern,
                    DeclarationKind::CatchParam,
                    value,
                )?;
            }
            this.reset_statement_result()?;
            this.lower_statement(handler.body)
        })
    }

    pub(super) fn begin_exception_finally_path(&mut self) -> LoweringResult<()> {
        let registers = self.ensure_completion_registers()?;
        self.builder
            .emit_ax(Opcode::LoadException, i32::from(registers.value))?;
        self.emit_load_smi(registers.target, 0)?;
        self.emit_load_smi(registers.kind, CompletionKind::Throw.encoded())?;
        Ok(())
    }

    pub(super) fn emit_finally_dispatch(&mut self, current_finally: usize) -> LoweringResult<()> {
        let registers = self.ensure_completion_registers()?;
        if let Some(result_register) = self.result_register {
            self.emit_move(result_register, registers.value)?;
        }
        let kind_test = self.alloc_temp()?;
        let zero = self.alloc_temp()?;
        self.emit_load_smi(zero, CompletionKind::Normal.encoded())?;
        self.builder.emit_abc(
            Opcode::StrictEqual,
            self.encode_register(kind_test)?,
            self.encode_register(registers.kind)?,
            self.encode_register(zero)?,
        )?;
        let jump_resume = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfFalse, self.encode_register(kind_test)?)?;
        let normal_end = self.builder.emit_jump_placeholder(Opcode::Jump)?;
        let resume_offset = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_resume, resume_offset)?;

        if let Some(outer) = self.outer_active_finally(current_finally) {
            self.emit_jump_to_finally(outer)?;
        } else {
            self.emit_completion_terminal_dispatch()?;
        }

        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(normal_end, end)?;
        Ok(())
    }

    pub(super) fn emit_completion_terminal_dispatch(&mut self) -> LoweringResult<()> {
        let registers = self.ensure_completion_registers()?;
        let kind_test = self.alloc_temp()?;
        let constant = self.alloc_temp()?;

        self.emit_load_smi(constant, CompletionKind::Return.encoded())?;
        self.builder.emit_abc(
            Opcode::StrictEqual,
            self.encode_register(kind_test)?,
            self.encode_register(registers.kind)?,
            self.encode_register(constant)?,
        )?;
        let jump_return = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfFalse, self.encode_register(kind_test)?)?;
        self.builder
            .emit_ax(Opcode::Return, i32::from(registers.value))?;
        let next = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_return, next)?;

        self.emit_load_smi(constant, CompletionKind::Throw.encoded())?;
        self.builder.emit_abc(
            Opcode::StrictEqual,
            self.encode_register(kind_test)?,
            self.encode_register(registers.kind)?,
            self.encode_register(constant)?,
        )?;
        let jump_throw = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfFalse, self.encode_register(kind_test)?)?;
        self.builder
            .emit_ax(Opcode::Throw, i32::from(registers.value))?;
        let next = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_throw, next)?;

        self.emit_target_dispatch(CompletionKind::Break)?;
        self.emit_target_dispatch(CompletionKind::Continue)?;
        Ok(())
    }

    pub(super) fn emit_target_dispatch(&mut self, kind: CompletionKind) -> LoweringResult<()> {
        let registers = self.ensure_completion_registers()?;
        let kind_constant = self.alloc_temp()?;
        let kind_match = self.alloc_temp()?;
        self.emit_load_smi(kind_constant, kind.encoded())?;
        self.builder.emit_abc(
            Opcode::StrictEqual,
            self.encode_register(kind_match)?,
            self.encode_register(registers.kind)?,
            self.encode_register(kind_constant)?,
        )?;
        let jump_end = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfFalse, self.encode_register(kind_match)?)?;

        for index in 0..self.control_targets.len() {
            if kind == CompletionKind::Continue
                && self.control_targets[index].kind != ControlTargetKind::Loop
            {
                continue;
            }

            let target_constant = self.alloc_temp()?;
            let target_match = self.alloc_temp()?;
            self.emit_load_smi(
                target_constant,
                i16::try_from(self.control_targets[index].id).unwrap_or(i16::MAX),
            )?;
            self.builder.emit_abc(
                Opcode::StrictEqual,
                self.encode_register(target_match)?,
                self.encode_register(registers.target)?,
                self.encode_register(target_constant)?,
            )?;
            let next_case = self.builder.emit_cond_jump_placeholder(
                Opcode::JumpIfFalse,
                self.encode_register(target_match)?,
            )?;
            let placeholder = self.builder.emit_jump_placeholder(Opcode::Jump)?;
            match kind {
                CompletionKind::Break => self.control_targets[index]
                    .break_placeholders
                    .push(placeholder),
                CompletionKind::Continue => self.control_targets[index]
                    .continue_placeholders
                    .push(placeholder),
                _ => unreachable!("target dispatch only handles break/continue"),
            }
            let after_case = self.builder.current_offset()?;
            self.builder.patch_jump_to(next_case, after_case)?;
        }

        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_end, end)?;
        Ok(())
    }

    pub(super) fn statement_supports_continue_label(&self, stmt: StmtId) -> bool {
        matches!(
            self.ast().get_stmt(stmt),
            Stmt::DoWhile { .. }
                | Stmt::While { .. }
                | Stmt::For { .. }
                | Stmt::ForIn { .. }
                | Stmt::ForOf { .. }
        )
    }

    pub(super) fn push_control_target(
        &mut self,
        label: Option<AtomId>,
        kind: ControlTargetKind,
    ) -> usize {
        let id = self.next_control_target_id;
        self.next_control_target_id = self.next_control_target_id.saturating_add(1);
        self.control_targets
            .push(ControlTarget::new(id, label, kind));
        self.control_targets.len() - 1
    }

    pub(super) fn pop_control_target(&mut self, index: usize) {
        debug_assert_eq!(index + 1, self.control_targets.len());
        let _ = self.control_targets.pop();
    }

    pub(super) fn patch_break_placeholders(
        &mut self,
        index: usize,
        target_offset: u32,
    ) -> LoweringResult<()> {
        let placeholders = std::mem::take(&mut self.control_targets[index].break_placeholders);
        for jump in placeholders {
            self.builder.patch_jump_to(jump, target_offset)?;
        }
        Ok(())
    }

    pub(super) fn patch_continue_placeholders(
        &mut self,
        index: usize,
        target_offset: u32,
    ) -> LoweringResult<()> {
        let placeholders = std::mem::take(&mut self.control_targets[index].continue_placeholders);
        for jump in placeholders {
            self.builder.patch_jump_to(jump, target_offset)?;
        }
        Ok(())
    }

    pub(super) fn resolve_break_target(&self, label: Option<AtomId>) -> Option<usize> {
        if let Some(label) = label {
            return self
                .control_targets
                .iter()
                .rposition(|target| target.label == Some(label));
        }
        self.control_targets.iter().rposition(|target| {
            matches!(
                target.kind,
                ControlTargetKind::Loop | ControlTargetKind::Switch
            )
        })
    }

    pub(super) fn resolve_continue_target(&self, label: Option<AtomId>) -> Option<usize> {
        self.control_targets.iter().rposition(|target| {
            target.kind == ControlTargetKind::Loop
                && match label {
                    Some(label) => target.label == Some(label),
                    None => true,
                }
        })
    }

    pub(super) fn ensure_completion_registers(&mut self) -> LoweringResult<CompletionRegisters> {
        if let Some(registers) = self.completion_registers {
            return Ok(registers);
        }
        let registers = CompletionRegisters {
            kind: self.alloc_temp()?,
            value: self.alloc_temp()?,
            target: self.alloc_temp()?,
        };
        self.completion_registers = Some(registers);
        Ok(registers)
    }

    pub(super) fn reset_statement_result(&mut self) -> LoweringResult<()> {
        if let Some(result_register) = self.result_register {
            self.emit_load_undefined(result_register)?;
        }
        Ok(())
    }

    pub(super) fn set_completion_state(
        &mut self,
        kind: CompletionKind,
        value: Option<u16>,
        target: Option<u16>,
    ) -> LoweringResult<()> {
        let registers = self.ensure_completion_registers()?;
        self.emit_load_smi(registers.kind, kind.encoded())?;
        if let Some(value) = value {
            self.emit_move(registers.value, value)?;
        } else {
            self.emit_load_undefined(registers.value)?;
        }
        if let Some(target) = target {
            self.emit_load_smi(registers.target, i16::try_from(target).unwrap_or(i16::MAX))?;
        } else {
            self.emit_load_smi(registers.target, 0)?;
        }
        Ok(())
    }

    pub(super) fn push_finally_context(&mut self) -> usize {
        self.finally_stack.push(FinallyContext::default());
        self.finally_stack.len() - 1
    }

    pub(super) fn pop_finally_context(&mut self, index: usize) {
        debug_assert_eq!(index + 1, self.finally_stack.len());
        let _ = self.finally_stack.pop();
    }

    pub(super) fn mark_finally_body(&mut self, index: usize, enabled: bool) {
        if let Some(context) = self.finally_stack.get_mut(index) {
            context.in_finalizer = enabled;
        }
    }

    pub(super) fn set_finally_normal_entry(
        &mut self,
        index: usize,
        entry: u32,
    ) -> LoweringResult<()> {
        let context = &mut self.finally_stack[index];
        context.normal_entry = Some(entry);
        for jump in std::mem::take(&mut context.normal_entry_placeholders) {
            self.builder.patch_jump_to(jump, entry)?;
        }
        Ok(())
    }

    pub(super) fn emit_jump_to_finally(&mut self, index: usize) -> LoweringResult<()> {
        if let Some(entry) = self.finally_stack[index].normal_entry {
            let jump = self.builder.emit_jump_placeholder(Opcode::Jump)?;
            self.builder.patch_jump_to(jump, entry)?;
            return Ok(());
        }

        let jump = self.builder.emit_jump_placeholder(Opcode::Jump)?;
        self.finally_stack[index]
            .normal_entry_placeholders
            .push(jump);
        Ok(())
    }

    pub(super) fn nearest_active_finally(&self) -> Option<usize> {
        self.finally_stack
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, context)| (!context.in_finalizer).then_some(index))
    }

    pub(super) fn outer_active_finally(&self, current: usize) -> Option<usize> {
        self.finally_stack[..current]
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, context)| (!context.in_finalizer).then_some(index))
    }

    pub(super) fn emit_enter_handler(&mut self) -> LoweringResult<u32> {
        Ok(self.builder.emit_ax(Opcode::EnterHandler, 0)?)
    }

    pub(super) fn emit_leave_handler(&mut self) -> LoweringResult<()> {
        self.builder.emit_ax(Opcode::LeaveHandler, 0)?;
        Ok(())
    }
}

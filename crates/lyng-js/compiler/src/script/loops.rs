use super::{
    AtomId, CompletionKind, ControlTargetKind, Decl, DeclId, DeclarationKind, ExceptionHandler,
    ExceptionHandlerKind, Expr, ExprId, ForInOfLeft, ForInit, FunctionCompiler, FunctionId,
    LoweringError, LoweringResult, Opcode, Pattern, SafepointKind, ScopeId, ScopeKind,
    SemanticBindingId, Span, Stmt, StmtId, VariableKind,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct LoopIterationEnvironmentPlan {
    iteration_slots: Vec<u16>,
    shared_slots: Vec<u16>,
    copy_slots: Vec<u16>,
}

impl FunctionCompiler<'_, '_> {
    fn for_init_has_lexical_scope(&self, init: Option<ForInit>) -> bool {
        matches!(init, Some(ForInit::Declaration(decl_id)) if matches!(
            self.ast().get_decl(decl_id),
            Decl::Variable {
                kind:
                    VariableKind::Let
                    | VariableKind::Const
                    | VariableKind::Using
                    | VariableKind::AwaitUsing,
                ..
            }
        ))
    }

    fn for_in_of_has_lexical_scope(&self, left: ForInOfLeft) -> bool {
        matches!(left, ForInOfLeft::Declaration(decl_id) if matches!(
            self.ast().get_decl(decl_id),
            Decl::Variable {
                kind:
                    VariableKind::Let
                    | VariableKind::Const
                    | VariableKind::Using
                    | VariableKind::AwaitUsing,
                ..
            }
        ))
    }

    fn lower_for_init(&mut self, init: ForInit) -> LoweringResult<()> {
        match init {
            ForInit::Declaration(decl) => self.lower_declaration(decl),
            ForInit::Expression(expr) => {
                let temp = self.alloc_temp()?;
                self.lower_expr_into(expr, temp)
            }
        }
    }

    pub(super) fn lower_do_while_statement(
        &mut self,
        label: Option<AtomId>,
        body: StmtId,
        test: ExprId,
        span: Span,
    ) -> LoweringResult<()> {
        self.reset_statement_result()?;
        let target = self.push_control_target(label, ControlTargetKind::Loop);
        let loop_start = self.builder.current_offset()?;
        self.lower_statement(body)?;
        let continue_target = self.builder.current_offset()?;
        self.patch_continue_placeholders(target, continue_target)?;
        let test_register = self.lower_expr_to_temp(test)?;
        let jump_end = self.builder.emit_cond_jump_placeholder(
            Opcode::JumpIfFalse,
            self.encode_register(test_register)?,
        )?;
        let jump_back = self.builder.emit_jump_placeholder(Opcode::Jump)?;
        self.attach_safepoint(jump_back, span, SafepointKind::LoopBackedge)?;
        self.builder.patch_jump_to(jump_back, loop_start)?;
        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_end, end)?;
        self.patch_break_placeholders(target, end)?;
        self.pop_control_target(target);
        Ok(())
    }

    pub(super) fn lower_while_statement(
        &mut self,
        label: Option<AtomId>,
        test: ExprId,
        body: StmtId,
        span: Span,
    ) -> LoweringResult<()> {
        self.reset_statement_result()?;
        let target = self.push_control_target(label, ControlTargetKind::Loop);
        let loop_start = self.builder.current_offset()?;
        self.patch_continue_placeholders(target, loop_start)?;
        let test_register = self.lower_expr_to_temp(test)?;
        let jump_end = self.builder.emit_cond_jump_placeholder(
            Opcode::JumpIfFalse,
            self.encode_register(test_register)?,
        )?;
        self.lower_statement(body)?;
        self.patch_continue_placeholders(target, loop_start)?;
        let jump_back = self.builder.emit_jump_placeholder(Opcode::Jump)?;
        self.attach_safepoint(jump_back, span, SafepointKind::LoopBackedge)?;
        self.builder.patch_jump_to(jump_back, loop_start)?;
        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_end, end)?;
        self.patch_break_placeholders(target, end)?;
        self.pop_control_target(target);
        Ok(())
    }

    pub(super) fn lower_for_statement(
        &mut self,
        label: Option<AtomId>,
        init: Option<ForInit>,
        test: Option<ExprId>,
        update: Option<ExprId>,
        body: StmtId,
        span: Span,
    ) -> LoweringResult<()> {
        if self.for_init_has_lexical_scope(init) {
            return self.with_child_scope(ScopeKind::ForLoop, true, body, |this| {
                if let Some(kind) = this.for_init_disposal_scope_kind(init) {
                    return this.with_disposal_scope(kind, span, move |inner| {
                        inner.lower_for_statement_core(label, init, test, update, body, span)
                    });
                }
                this.lower_for_statement_core(label, init, test, update, body, span)
            });
        }
        if let Some(kind) = self.for_init_disposal_scope_kind(init) {
            return self.with_disposal_scope(kind, span, move |this| {
                this.lower_for_statement_core(label, init, test, update, body, span)
            });
        }
        self.lower_for_statement_core(label, init, test, update, body, span)
    }

    fn lower_for_statement_core(
        &mut self,
        label: Option<AtomId>,
        init: Option<ForInit>,
        test: Option<ExprId>,
        update: Option<ExprId>,
        body: StmtId,
        span: Span,
    ) -> LoweringResult<()> {
        self.reset_statement_result()?;
        let env_scope = self.current_dynamic_env_scope_range()?;
        if let Some((base, count)) = env_scope {
            self.emit_enter_env_scope(base, count)?;
        }
        if self.for_init_has_lexical_scope(init) {
            self.emit_frame_local_tdz_initializers_for_current_scope()?;
        }
        if let Some(init) = init {
            self.lower_for_init(init)?;
        }
        let loop_iteration_plan = self.normal_for_loop_iteration_plan(init, test, update, body)?;
        if let Some(plan) = &loop_iteration_plan {
            self.emit_push_loop_iteration_environment(plan)?;
        }
        let target = self.push_control_target(label, ControlTargetKind::Loop);
        let loop_start = self.builder.current_offset()?;
        let exit_jump = if let Some(test) = test {
            let test_register = self.lower_expr_to_temp(test)?;
            Some(self.builder.emit_cond_jump_placeholder(
                Opcode::JumpIfFalse,
                self.encode_register(test_register)?,
            )?)
        } else {
            None
        };
        self.lower_statement(body)?;
        let continue_target = self.builder.current_offset()?;
        self.patch_continue_placeholders(target, continue_target)?;
        if let Some(plan) = &loop_iteration_plan {
            self.emit_recreate_loop_iteration_environment(plan)?;
        }
        if let Some(update) = update {
            let update_register = self.alloc_temp()?;
            self.lower_expr_into(update, update_register)?;
        }
        let jump_back = self.builder.emit_jump_placeholder(Opcode::Jump)?;
        self.attach_safepoint(jump_back, span, SafepointKind::LoopBackedge)?;
        self.builder.patch_jump_to(jump_back, loop_start)?;
        let break_cleanup = self.builder.current_offset()?;
        if loop_iteration_plan.is_some() {
            self.builder.emit_ax(Opcode::PopClosureEnv, 0)?;
        }
        if let Some((base, count)) = env_scope {
            self.emit_leave_env_scope(base, count)?;
        }
        if let Some(exit_jump) = exit_jump {
            self.builder.patch_jump_to(exit_jump, break_cleanup)?;
        }
        self.patch_break_placeholders(target, break_cleanup)?;
        self.pop_control_target(target);
        Ok(())
    }

    pub(super) fn lower_for_in_statement(
        &mut self,
        label: Option<AtomId>,
        left: ForInOfLeft,
        right: ExprId,
        body: StmtId,
        span: Span,
    ) -> LoweringResult<()> {
        if self.for_in_of_has_lexical_scope(left) {
            return self.with_child_scope(ScopeKind::ForLoop, true, body, |this| {
                this.lower_for_in_statement_core(label, left, right, body, span)
            });
        }
        self.lower_for_in_statement_core(label, left, right, body, span)
    }

    fn lower_for_in_statement_core(
        &mut self,
        label: Option<AtomId>,
        left: ForInOfLeft,
        right: ExprId,
        body: StmtId,
        span: Span,
    ) -> LoweringResult<()> {
        self.reset_statement_result()?;
        let env_scope = self.current_dynamic_env_scope_range()?;
        if let Some((base, count)) = env_scope {
            self.emit_enter_env_scope(base, count)?;
        }
        if self.for_in_of_has_lexical_scope(left) {
            self.emit_frame_local_tdz_initializers_for_current_scope()?;
        }
        let head_tdz_plan = self.for_in_of_head_tdz_plan(left)?;
        let iteration_disposal_kind = self.for_in_of_declaration_disposal_scope_kind(left);
        self.lower_annex_b_for_in_var_initializer(left)?;
        let object_register = if let Some(plan) = &head_tdz_plan {
            let push = self.builder.emit_ax(Opcode::PushClosureEnv, 0)?;
            self.builder.add_loop_iteration_environment_site(
                push,
                plan.iteration_slots.clone(),
                plan.shared_slots.clone(),
                plan.copy_slots.clone(),
            );
            let object_register = self.lower_expr_to_temp(right)?;
            self.builder.emit_ax(Opcode::PopClosureEnv, 0)?;
            object_register
        } else {
            self.lower_expr_to_temp(right)?
        };
        let enumerator_register = self.alloc_temp()?;
        let key_register = self.alloc_temp()?;
        let done_register = self.alloc_temp()?;
        let target = self.push_control_target(label, ControlTargetKind::Loop);
        let loop_iteration_plan = self.for_in_of_loop_iteration_plan(left, body)?;

        self.builder.emit_abc(
            Opcode::CreateForIn,
            self.encode_register(enumerator_register)?,
            self.encode_register(object_register)?,
            0,
        )?;

        let loop_start = self.builder.current_offset()?;
        self.builder.emit_abc(
            Opcode::AdvanceForIn,
            self.encode_register(enumerator_register)?,
            self.encode_register(key_register)?,
            self.encode_register(done_register)?,
        )?;
        let exit_jump = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfTrue, self.encode_register(done_register)?)?;

        if let Some(plan) = &loop_iteration_plan {
            let push = self.builder.emit_ax(Opcode::PushClosureEnv, 0)?;
            self.builder.add_loop_iteration_environment_site(
                push,
                plan.iteration_slots.clone(),
                plan.shared_slots.clone(),
                Vec::new(),
            );
        }
        if let Some(kind) = iteration_disposal_kind {
            self.with_disposal_scope(kind, span, move |this| {
                this.lower_for_in_left_assignment(left, key_register)?;
                this.lower_statement(body)
            })?;
        } else {
            self.lower_for_in_left_assignment(left, key_register)?;
            self.lower_statement(body)?;
        }
        let continue_cleanup = self.builder.current_offset()?;
        self.patch_continue_placeholders(target, continue_cleanup)?;
        if loop_iteration_plan.is_some() {
            self.builder.emit_ax(Opcode::PopClosureEnv, 0)?;
        }

        let jump_back = self.builder.emit_jump_placeholder(Opcode::Jump)?;
        self.attach_safepoint(jump_back, span, SafepointKind::LoopBackedge)?;
        self.builder.patch_jump_to(jump_back, loop_start)?;

        let break_cleanup = self.builder.current_offset()?;
        if loop_iteration_plan.is_some() {
            self.builder.emit_ax(Opcode::PopClosureEnv, 0)?;
        }
        if let Some((base, count)) = env_scope {
            self.emit_leave_env_scope(base, count)?;
        }
        let close_offset = self.builder.current_offset()?;
        self.builder.emit_abx(
            Opcode::CloseForIn,
            self.encode_register(enumerator_register)?,
            0,
        )?;
        let end_offset = self.builder.current_offset()?;
        self.builder.patch_jump_to(exit_jump, close_offset)?;
        self.patch_break_placeholders(target, break_cleanup)?;
        self.pop_control_target(target);
        debug_assert!(end_offset >= close_offset);
        Ok(())
    }

    fn lower_annex_b_for_in_var_initializer(&mut self, left: ForInOfLeft) -> LoweringResult<()> {
        let ForInOfLeft::Declaration(decl_id) = left else {
            return Ok(());
        };
        let Decl::Variable {
            kind, declarators, ..
        } = self.ast().get_decl(decl_id).clone()
        else {
            return Ok(());
        };
        if kind != VariableKind::Var {
            return Ok(());
        }

        let declarators = self.ast().get_var_declarator_list(declarators).to_vec();
        let [declarator] = declarators.as_slice() else {
            return Ok(());
        };
        let Some(init) = declarator.init else {
            return Ok(());
        };

        let value = self.alloc_temp()?;
        self.lower_initializer_with_inferred_name(
            init,
            self.binding_pattern_name(declarator.id),
            value,
        )?;
        self.lower_binding_pattern_initialization(declarator.id, DeclarationKind::Var, value)
    }

    pub(super) fn lower_for_of_statement(
        &mut self,
        label: Option<AtomId>,
        left: ForInOfLeft,
        right: ExprId,
        body: StmtId,
        r#await: bool,
        span: Span,
    ) -> LoweringResult<()> {
        if self.for_in_of_has_lexical_scope(left) {
            return self.with_child_scope(ScopeKind::ForLoop, true, body, |this| {
                this.lower_for_of_statement_core(label, left, right, body, r#await, span)
            });
        }
        self.lower_for_of_statement_core(label, left, right, body, r#await, span)
    }

    fn lower_for_of_statement_core(
        &mut self,
        label: Option<AtomId>,
        left: ForInOfLeft,
        right: ExprId,
        body: StmtId,
        r#await: bool,
        span: Span,
    ) -> LoweringResult<()> {
        self.reset_statement_result()?;
        let env_scope = self.current_dynamic_env_scope_range()?;
        if let Some((base, count)) = env_scope {
            self.emit_enter_env_scope(base, count)?;
        }
        if self.for_in_of_has_lexical_scope(left) {
            self.emit_frame_local_tdz_initializers_for_current_scope()?;
        }
        let head_tdz_plan = self.for_in_of_head_tdz_plan(left)?;
        let iteration_disposal_kind = self.for_in_of_declaration_disposal_scope_kind(left);
        let iterable_register = if let Some(plan) = &head_tdz_plan {
            let push = self.builder.emit_ax(Opcode::PushClosureEnv, 0)?;
            self.builder.add_loop_iteration_environment_site(
                push,
                plan.iteration_slots.clone(),
                plan.shared_slots.clone(),
                Vec::new(),
            );
            let iterable_register = self.lower_expr_to_temp(right)?;
            self.builder.emit_ax(Opcode::PopClosureEnv, 0)?;
            iterable_register
        } else {
            self.lower_expr_to_temp(right)?
        };
        let iterator_register = self.alloc_temp()?;
        let value_register = self.alloc_temp()?;
        let done_register = self.alloc_temp()?;
        let target = self.push_control_target(label, ControlTargetKind::Loop);
        let target_id = self.control_targets[target].id;
        let loop_iteration_plan = self.for_in_of_loop_iteration_plan(left, body)?;
        self.builder.emit_abc(
            Opcode::CreateIterator,
            self.encode_register(iterator_register)?,
            self.encode_register(iterable_register)?,
            u8::from(r#await),
        )?;
        let loop_start = self.builder.current_offset()?;
        self.builder.emit_abc(
            Opcode::AdvanceIterator,
            self.encode_register(iterator_register)?,
            self.encode_register(value_register)?,
            self.encode_register(done_register)?,
        )?;
        let exit_jump = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfTrue, self.encode_register(done_register)?)?;
        if let Some(plan) = &loop_iteration_plan {
            let push = self.builder.emit_ax(Opcode::PushClosureEnv, 0)?;
            self.builder.add_loop_iteration_environment_site(
                push,
                plan.iteration_slots.clone(),
                plan.shared_slots.clone(),
                Vec::new(),
            );
        }
        let finally_index = self.push_finally_context();
        let protected_start = self.builder.current_offset()?;
        if let Some(kind) = iteration_disposal_kind {
            self.with_disposal_scope(kind, span, move |this| {
                this.lower_for_in_left_assignment(left, value_register)?;
                this.lower_statement(body)
            })?;
        } else {
            self.lower_for_in_left_assignment(left, value_register)?;
            self.lower_statement(body)?;
        }
        let protected_end = self.builder.current_offset()?;
        self.set_completion_state(CompletionKind::Normal, self.result_register, None)?;
        self.emit_jump_to_finally(finally_index)?;

        let throw_entry = self.builder.current_offset()?;
        let enter_handler = self.emit_enter_handler()?;
        self.attach_safepoint(enter_handler, span, SafepointKind::ExceptionEdge)?;
        self.begin_exception_finally_path()?;
        let normal_entry = self.builder.current_offset()?;
        self.set_finally_normal_entry(finally_index, normal_entry)?;
        self.mark_finally_body(finally_index, true);
        self.emit_leave_handler()?;

        let resume_jump = self.emit_completion_dispatch_branch(CompletionKind::Normal, None)?;
        let continue_self_jump =
            self.emit_completion_dispatch_branch(CompletionKind::Continue, Some(target_id))?;
        let break_self_jump =
            self.emit_completion_dispatch_branch(CompletionKind::Break, Some(target_id))?;
        let jump_escape = self.builder.emit_jump_placeholder(Opcode::Jump)?;

        let resume_offset = self.builder.current_offset()?;
        self.builder.patch_jump_to(resume_jump, resume_offset)?;
        self.builder
            .patch_jump_to(continue_self_jump, resume_offset)?;
        self.patch_continue_placeholders(target, resume_offset)?;
        if loop_iteration_plan.is_some() {
            self.builder.emit_ax(Opcode::PopClosureEnv, 0)?;
        }
        let jump_back = self.builder.emit_jump_placeholder(Opcode::Jump)?;
        self.attach_safepoint(jump_back, span, SafepointKind::LoopBackedge)?;
        self.builder.patch_jump_to(jump_back, loop_start)?;

        let break_cleanup = self.builder.current_offset()?;
        self.builder.patch_jump_to(break_self_jump, break_cleanup)?;
        if loop_iteration_plan.is_some() {
            self.builder.emit_ax(Opcode::PopClosureEnv, 0)?;
        }
        if let Some((base, count)) = env_scope {
            self.emit_leave_env_scope(base, count)?;
        }
        self.builder.emit_abx(
            Opcode::CloseIterator,
            self.encode_register(iterator_register)?,
            0,
        )?;
        let jump_end = self.builder.emit_jump_placeholder(Opcode::Jump)?;

        let escape_offset = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_escape, escape_offset)?;
        if loop_iteration_plan.is_some() {
            self.builder.emit_ax(Opcode::PopClosureEnv, 0)?;
        }
        if let Some((base, count)) = env_scope {
            self.emit_leave_env_scope(base, count)?;
        }
        self.emit_close_iterator_for_completion(iterator_register)?;
        if let Some(outer) = self.outer_active_finally(finally_index) {
            self.emit_jump_to_finally(outer)?;
        } else {
            self.emit_completion_terminal_dispatch()?;
        }
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

        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_end, end)?;
        self.builder.patch_jump_to(exit_jump, end)?;
        self.patch_break_placeholders(target, break_cleanup)?;
        self.pop_control_target(target);
        Ok(())
    }

    fn emit_completion_dispatch_branch(
        &mut self,
        kind: CompletionKind,
        target: Option<u16>,
    ) -> LoweringResult<u32> {
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
        let jump_next = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfFalse, self.encode_register(kind_match)?)?;

        let branch = if let Some(target) = target {
            let target_constant = self.alloc_temp()?;
            let target_match = self.alloc_temp()?;
            self.emit_load_smi(target_constant, i16::try_from(target).unwrap_or(i16::MAX))?;
            self.builder.emit_abc(
                Opcode::StrictEqual,
                self.encode_register(target_match)?,
                self.encode_register(registers.target)?,
                self.encode_register(target_constant)?,
            )?;
            let jump_target = self.builder.emit_cond_jump_placeholder(
                Opcode::JumpIfFalse,
                self.encode_register(target_match)?,
            )?;
            let branch = self.builder.emit_jump_placeholder(Opcode::Jump)?;
            let next = self.builder.current_offset()?;
            self.builder.patch_jump_to(jump_target, next)?;
            branch
        } else {
            self.builder.emit_jump_placeholder(Opcode::Jump)?
        };

        let next = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_next, next)?;
        Ok(branch)
    }

    fn lower_for_in_left_assignment(
        &mut self,
        left: ForInOfLeft,
        value_register: u16,
    ) -> LoweringResult<()> {
        match left {
            ForInOfLeft::Declaration(decl) => {
                self.lower_for_in_declaration_assignment(decl, value_register)
            }
            ForInOfLeft::Pattern(pattern) => {
                let Pattern::Identifier { name, .. } = self.ast().get_pattern(pattern).clone()
                else {
                    return Err(LoweringError::UnsupportedPattern { pattern });
                };
                let binding_id = self
                    .find_named_binding(name, DeclarationKind::Let)
                    .or_else(|_| self.find_named_binding(name, DeclarationKind::Const))?;
                self.store_binding_value(binding_id, name, value_register)
            }
            ForInOfLeft::Expression(expr) => match self.ast().get_expr(expr) {
                Expr::ArrayExpression { .. }
                | Expr::ObjectExpression { .. }
                | Expr::AssignmentExpression { .. }
                | Expr::ParenthesizedExpression { .. } => {
                    self.lower_destructuring_assignment_from_register(expr, value_register)
                }
                _ => self.lower_assignment_target_from_register(expr, value_register),
            },
        }
    }

    fn lower_for_in_declaration_assignment(
        &mut self,
        decl_id: DeclId,
        value_register: u16,
    ) -> LoweringResult<()> {
        let Decl::Variable {
            kind, declarators, ..
        } = self.ast().get_decl(decl_id).clone()
        else {
            return Err(LoweringError::UnsupportedDeclaration { decl: decl_id });
        };
        let declarators = self.ast().get_var_declarator_list(declarators).to_vec();
        let [declarator] = declarators.as_slice() else {
            return Err(LoweringError::UnsupportedDeclaration { decl: decl_id });
        };
        if kind == VariableKind::Var
            && let Pattern::Identifier { name, .. } = self.ast().get_pattern(declarator.id)
            && let Some(catch_binding) = self.active_simple_catch_binding_for_name(*name)
        {
            return self.assign_binding_value(catch_binding, *name, value_register);
        }
        self.lower_binding_pattern_initialization(
            declarator.id,
            match kind {
                VariableKind::Var => DeclarationKind::Var,
                VariableKind::Let => DeclarationKind::Let,
                VariableKind::Const => DeclarationKind::Const,
                VariableKind::Using => DeclarationKind::Using,
                VariableKind::AwaitUsing => DeclarationKind::AwaitUsing,
            },
            value_register,
        )?;
        if matches!(kind, VariableKind::Using | VariableKind::AwaitUsing) {
            self.lower_disposable_resource_registration(kind, value_register, declarator.span)?;
        }
        Ok(())
    }

    fn for_in_of_loop_iteration_plan(
        &self,
        left: ForInOfLeft,
        body: StmtId,
    ) -> LoweringResult<Option<LoopIterationEnvironmentPlan>> {
        let mut iteration_slots = self.for_in_of_binding_slots(left)?;
        if let Some(body_scope) = self.loop_body_block_scope(body) {
            self.collect_per_iteration_tdz_environment_slots(body_scope, &mut iteration_slots)?;
        }
        let per_iteration_roots = if let Some(binding_id) = self.for_in_of_capture_binding(left)? {
            let loop_scope = self.binding(binding_id)?.scope;
            self.for_in_of_capture_roots(loop_scope)
        } else {
            Vec::new()
        };

        let mut nested_functions = Vec::new();
        self.collect_functions_in_statement(body, &mut nested_functions);

        let mut shared_slots = Vec::new();
        for function in nested_functions {
            let sema_id = self
                .state
                .ast_to_sema
                .get(&function)
                .copied()
                .ok_or(LoweringError::MissingFunctionRecord { function })?;
            for &capture in &self.state.sema.function_table.get(sema_id).captures {
                let capture_scope = self.binding(capture)?.scope;
                if !self.binding_belongs_to_owner(capture_scope, self.current_function) {
                    continue;
                }
                let Some((depth, slot)) = self.binding_env_access(capture)? else {
                    continue;
                };
                if depth != 0 {
                    continue;
                }
                let slot = u16::try_from(slot)
                    .map_err(|_| LoweringError::ConstantIndexOverflow { index: slot })?;
                if per_iteration_roots
                    .iter()
                    .copied()
                    .any(|root| self.scope_is_same_or_descendant(capture_scope, root))
                {
                    iteration_slots.push(slot);
                } else {
                    shared_slots.push(slot);
                }
            }
        }

        iteration_slots.sort_unstable();
        iteration_slots.dedup();
        shared_slots.sort_unstable();
        shared_slots.dedup();

        Ok(
            (!iteration_slots.is_empty() || !shared_slots.is_empty()).then_some(
                LoopIterationEnvironmentPlan {
                    iteration_slots,
                    shared_slots,
                    copy_slots: Vec::new(),
                },
            ),
        )
    }

    fn loop_body_block_scope(&self, body: StmtId) -> Option<ScopeId> {
        matches!(self.ast().get_stmt(body), Stmt::Block { .. })
            .then(|| self.peek_child_scope_with_kind(ScopeKind::Block))
            .flatten()
    }

    fn collect_per_iteration_tdz_environment_slots(
        &self,
        scope: ScopeId,
        slots: &mut Vec<u16>,
    ) -> LoweringResult<()> {
        let record = self.state.sema.scope_table.get(scope);
        if record.owning_function != self.current_function {
            return Ok(());
        }
        for &binding_id in &record.bindings {
            let binding = self.binding(binding_id)?;
            if !binding.has_tdz {
                continue;
            }
            let Some((depth, slot)) = self.binding_env_access(binding_id)? else {
                continue;
            };
            if depth != 0 {
                continue;
            }
            slots.push(
                u16::try_from(slot)
                    .map_err(|_| LoweringError::ConstantIndexOverflow { index: slot })?,
            );
        }
        for &child in &record.children {
            self.collect_per_iteration_tdz_environment_slots(child, slots)?;
        }
        Ok(())
    }

    fn for_in_of_head_tdz_plan(
        &self,
        left: ForInOfLeft,
    ) -> LoweringResult<Option<LoopIterationEnvironmentPlan>> {
        let iteration_slots = self.for_in_of_binding_slots(left)?;
        Ok(
            (!iteration_slots.is_empty()).then_some(LoopIterationEnvironmentPlan {
                iteration_slots,
                shared_slots: Vec::new(),
                copy_slots: Vec::new(),
            }),
        )
    }

    fn normal_for_loop_iteration_plan(
        &self,
        init: Option<ForInit>,
        test: Option<ExprId>,
        update: Option<ExprId>,
        body: StmtId,
    ) -> LoweringResult<Option<LoopIterationEnvironmentPlan>> {
        let copy_slots = self.for_init_let_binding_slots(init)?;
        let mut nested_functions = Vec::new();
        if let Some(init) = init {
            match init {
                ForInit::Declaration(decl) => {
                    self.collect_functions_in_declaration(decl, &mut nested_functions);
                }
                ForInit::Expression(expr) => {
                    self.collect_functions_in_expression(expr, &mut nested_functions);
                }
            }
        }
        if let Some(test) = test {
            self.collect_functions_in_expression(test, &mut nested_functions);
        }
        if let Some(update) = update {
            self.collect_functions_in_expression(update, &mut nested_functions);
        }
        self.collect_functions_in_statement(body, &mut nested_functions);

        let mut iteration_slots = Vec::new();
        if let Some(body_scope) = self.loop_body_block_scope(body) {
            self.collect_per_iteration_tdz_environment_slots(body_scope, &mut iteration_slots)?;
        }
        let mut shared_slots = Vec::new();
        for function in nested_functions {
            let sema_id = self
                .state
                .ast_to_sema
                .get(&function)
                .copied()
                .ok_or(LoweringError::MissingFunctionRecord { function })?;
            for &capture in &self.state.sema.function_table.get(sema_id).captures {
                let capture_scope = self.binding(capture)?.scope;
                if !self.binding_belongs_to_owner(capture_scope, self.current_function) {
                    continue;
                }
                let Some((depth, slot)) = self.binding_env_access(capture)? else {
                    continue;
                };
                if depth != 0 {
                    continue;
                }
                let slot = u16::try_from(slot)
                    .map_err(|_| LoweringError::ConstantIndexOverflow { index: slot })?;
                if copy_slots.contains(&slot)
                    || (capture_scope != self.current_scope
                        && self.scope_is_same_or_descendant(capture_scope, self.current_scope))
                {
                    iteration_slots.push(slot);
                } else {
                    shared_slots.push(slot);
                }
            }
        }

        iteration_slots.sort_unstable();
        iteration_slots.dedup();
        shared_slots.sort_unstable();
        shared_slots.dedup();

        let mut copy_slots = copy_slots
            .into_iter()
            .filter(|slot| iteration_slots.contains(slot))
            .collect::<Vec<_>>();
        copy_slots.sort_unstable();
        copy_slots.dedup();

        Ok(
            (!iteration_slots.is_empty()).then_some(LoopIterationEnvironmentPlan {
                iteration_slots,
                shared_slots,
                copy_slots,
            }),
        )
    }

    fn emit_recreate_loop_iteration_environment(
        &mut self,
        plan: &LoopIterationEnvironmentPlan,
    ) -> LoweringResult<()> {
        let copied_values = self.load_loop_iteration_copy_slots(plan)?;
        self.builder.emit_ax(Opcode::PopClosureEnv, 0)?;
        self.emit_push_loop_iteration_environment_with_copies(plan, copied_values)
    }

    fn emit_push_loop_iteration_environment(
        &mut self,
        plan: &LoopIterationEnvironmentPlan,
    ) -> LoweringResult<()> {
        let copied_values = self.load_loop_iteration_copy_slots(plan)?;
        self.emit_push_loop_iteration_environment_with_copies(plan, copied_values)
    }

    fn load_loop_iteration_copy_slots(
        &mut self,
        plan: &LoopIterationEnvironmentPlan,
    ) -> LoweringResult<Vec<(u16, u16)>> {
        let mut copied_values = Vec::with_capacity(plan.copy_slots.len());
        for &slot in &plan.copy_slots {
            let value = self.alloc_temp()?;
            self.emit_load_env_slot(value, 0, u32::from(slot))?;
            copied_values.push((slot, value));
        }
        Ok(copied_values)
    }

    fn emit_push_loop_iteration_environment_with_copies(
        &mut self,
        plan: &LoopIterationEnvironmentPlan,
        copied_values: Vec<(u16, u16)>,
    ) -> LoweringResult<()> {
        let push = self.builder.emit_ax(Opcode::PushClosureEnv, 0)?;
        self.builder.add_loop_iteration_environment_site(
            push,
            plan.iteration_slots.clone(),
            plan.shared_slots.clone(),
            plan.copy_slots.clone(),
        );
        for (slot, value) in copied_values {
            self.emit_store_env_slot(value, 0, u32::from(slot))?;
        }
        Ok(())
    }

    fn for_in_of_capture_roots(&self, loop_scope: ScopeId) -> Vec<ScopeId> {
        // Ancestor loop bindings stay owned by their active outer iteration environments.
        vec![loop_scope]
    }

    fn scope_is_same_or_descendant(&self, scope: ScopeId, ancestor: ScopeId) -> bool {
        let mut cursor = Some(scope);
        while let Some(current) = cursor {
            if current == ancestor {
                return true;
            }
            cursor = self.state.sema.scope_table.get(current).parent;
        }
        false
    }

    fn for_in_of_capture_binding(
        &self,
        left: ForInOfLeft,
    ) -> LoweringResult<Option<SemanticBindingId>> {
        let ForInOfLeft::Declaration(decl_id) = left else {
            return Ok(None);
        };
        let Decl::Variable {
            kind, declarators, ..
        } = self.ast().get_decl(decl_id).clone()
        else {
            return Ok(None);
        };
        let expected_kind = match kind {
            VariableKind::Let => DeclarationKind::Let,
            VariableKind::Const => DeclarationKind::Const,
            VariableKind::Var => return Ok(None),
            VariableKind::Using => DeclarationKind::Using,
            VariableKind::AwaitUsing => DeclarationKind::AwaitUsing,
        };
        let [declarator] = self.ast().get_var_declarator_list(declarators) else {
            return Ok(None);
        };
        let Some(binding_pattern) = self.first_identifier_in_pattern(declarator.id) else {
            return Ok(None);
        };
        Ok(Some(self.declared_binding_for_pattern(
            binding_pattern,
            expected_kind,
        )?))
    }

    fn for_init_let_binding_slots(&self, init: Option<ForInit>) -> LoweringResult<Vec<u16>> {
        let Some(ForInit::Declaration(decl_id)) = init else {
            return Ok(Vec::new());
        };
        let Decl::Variable {
            kind, declarators, ..
        } = self.ast().get_decl(decl_id).clone()
        else {
            return Ok(Vec::new());
        };
        if kind != VariableKind::Let {
            return Ok(Vec::new());
        }

        let mut slots = Vec::new();
        for declarator in self.ast().get_var_declarator_list(declarators) {
            let mut identifier_patterns = Vec::new();
            self.collect_identifier_patterns(declarator.id, &mut identifier_patterns);
            for pattern in identifier_patterns {
                let binding = self.declared_binding_for_pattern(pattern, DeclarationKind::Let)?;
                let Some((depth, slot)) = self.binding_env_access(binding)? else {
                    continue;
                };
                if depth != 0 {
                    continue;
                }
                let slot = u16::try_from(slot)
                    .map_err(|_| LoweringError::ConstantIndexOverflow { index: slot })?;
                slots.push(slot);
            }
        }

        slots.sort_unstable();
        slots.dedup();
        Ok(slots)
    }

    fn for_in_of_binding_slots(&self, left: ForInOfLeft) -> LoweringResult<Vec<u16>> {
        let ForInOfLeft::Declaration(decl_id) = left else {
            return Ok(Vec::new());
        };
        let Decl::Variable {
            kind, declarators, ..
        } = self.ast().get_decl(decl_id).clone()
        else {
            return Ok(Vec::new());
        };
        let expected_kind = match kind {
            VariableKind::Let => DeclarationKind::Let,
            VariableKind::Const => DeclarationKind::Const,
            VariableKind::Var => return Ok(Vec::new()),
            VariableKind::Using => DeclarationKind::Using,
            VariableKind::AwaitUsing => DeclarationKind::AwaitUsing,
        };
        let [declarator] = self.ast().get_var_declarator_list(declarators) else {
            return Ok(Vec::new());
        };

        let mut identifier_patterns = Vec::new();
        self.collect_identifier_patterns(declarator.id, &mut identifier_patterns);

        let mut slots = Vec::new();
        for pattern in identifier_patterns {
            let binding = self.declared_binding_for_pattern(pattern, expected_kind)?;
            let Some((depth, slot)) = self.binding_env_access(binding)? else {
                continue;
            };
            if depth != 0 {
                continue;
            }
            let slot = u16::try_from(slot)
                .map_err(|_| LoweringError::ConstantIndexOverflow { index: slot })?;
            slots.push(slot);
        }

        slots.sort_unstable();
        slots.dedup();
        Ok(slots)
    }

    fn first_identifier_in_pattern(
        &self,
        pattern_id: lyng_js_ast::PatternId,
    ) -> Option<lyng_js_ast::PatternId> {
        match self.ast().get_pattern(pattern_id) {
            Pattern::Identifier { .. } => Some(pattern_id),
            Pattern::Object {
                properties, rest, ..
            } => self
                .ast()
                .get_obj_pattern_prop_list(*properties)
                .iter()
                .find_map(|property| self.first_identifier_in_pattern(property.value))
                .or_else(|| rest.and_then(|rest| self.first_identifier_in_pattern(rest))),
            Pattern::Array { elements, rest, .. } => self
                .ast()
                .get_opt_pattern_elem_list(*elements)
                .iter()
                .flatten()
                .find_map(|element| self.first_identifier_in_pattern(element.pattern))
                .or_else(|| rest.and_then(|rest| self.first_identifier_in_pattern(rest))),
            Pattern::Assignment { left, .. } => self.first_identifier_in_pattern(*left),
            Pattern::InvalidPattern { .. } => None,
        }
    }

    fn collect_identifier_patterns(
        &self,
        pattern_id: lyng_js_ast::PatternId,
        patterns: &mut Vec<lyng_js_ast::PatternId>,
    ) {
        match self.ast().get_pattern(pattern_id) {
            Pattern::Identifier { .. } => patterns.push(pattern_id),
            Pattern::Object {
                properties, rest, ..
            } => {
                for property in self.ast().get_obj_pattern_prop_list(*properties) {
                    self.collect_identifier_patterns(property.value, patterns);
                }
                if let Some(rest) = rest {
                    self.collect_identifier_patterns(*rest, patterns);
                }
            }
            Pattern::Array { elements, rest, .. } => {
                for element in self
                    .ast()
                    .get_opt_pattern_elem_list(*elements)
                    .iter()
                    .flatten()
                {
                    self.collect_identifier_patterns(element.pattern, patterns);
                }
                if let Some(rest) = rest {
                    self.collect_identifier_patterns(*rest, patterns);
                }
            }
            Pattern::Assignment { left, .. } => {
                self.collect_identifier_patterns(*left, patterns);
            }
            Pattern::InvalidPattern { .. } => {}
        }
    }

    fn collect_functions_in_statement(&self, stmt_id: StmtId, functions: &mut Vec<FunctionId>) {
        match self.ast().get_stmt(stmt_id) {
            Stmt::Block { body, .. } => {
                for &stmt in self.ast().get_stmt_list(*body) {
                    self.collect_functions_in_statement(stmt, functions);
                }
            }
            Stmt::Empty { .. }
            | Stmt::Break { .. }
            | Stmt::Continue { .. }
            | Stmt::Debugger { .. }
            | Stmt::InvalidStatement { .. } => {}
            Stmt::Expression { expression, .. } => {
                self.collect_functions_in_expression(*expression, functions);
            }
            Stmt::If {
                test,
                consequent,
                alternate,
                ..
            } => {
                self.collect_functions_in_expression(*test, functions);
                self.collect_functions_in_statement(*consequent, functions);
                if let Some(alternate) = alternate {
                    self.collect_functions_in_statement(*alternate, functions);
                }
            }
            Stmt::DoWhile { body, test, .. } | Stmt::While { test, body, .. } => {
                self.collect_functions_in_statement(*body, functions);
                self.collect_functions_in_expression(*test, functions);
            }
            Stmt::For {
                init,
                test,
                update,
                body,
                ..
            } => {
                if let Some(init) = init {
                    match init {
                        ForInit::Declaration(decl) => {
                            self.collect_functions_in_declaration(*decl, functions);
                        }
                        ForInit::Expression(expr) => {
                            self.collect_functions_in_expression(*expr, functions);
                        }
                    }
                }
                if let Some(test) = test {
                    self.collect_functions_in_expression(*test, functions);
                }
                if let Some(update) = update {
                    self.collect_functions_in_expression(*update, functions);
                }
                self.collect_functions_in_statement(*body, functions);
            }
            Stmt::ForIn {
                left, right, body, ..
            }
            | Stmt::ForOf {
                left, right, body, ..
            } => {
                self.collect_functions_in_for_in_left(*left, functions);
                self.collect_functions_in_expression(*right, functions);
                self.collect_functions_in_statement(*body, functions);
            }
            Stmt::Return { argument, .. } => {
                if let Some(argument) = argument {
                    self.collect_functions_in_expression(*argument, functions);
                }
            }
            Stmt::With { object, body, .. } => {
                self.collect_functions_in_expression(*object, functions);
                self.collect_functions_in_statement(*body, functions);
            }
            Stmt::Switch {
                discriminant,
                cases,
                ..
            } => {
                self.collect_functions_in_expression(*discriminant, functions);
                for case in self.ast().get_switch_case_list(*cases) {
                    if let Some(test) = case.test {
                        self.collect_functions_in_expression(test, functions);
                    }
                    for &stmt in self.ast().get_stmt_list(case.consequent) {
                        self.collect_functions_in_statement(stmt, functions);
                    }
                }
            }
            Stmt::Labeled { body, .. } => {
                self.collect_functions_in_statement(*body, functions);
            }
            Stmt::Throw { argument, .. } => {
                self.collect_functions_in_expression(*argument, functions);
            }
            Stmt::Try {
                block,
                handler,
                finalizer,
                ..
            } => {
                self.collect_functions_in_statement(*block, functions);
                if let Some(handler) = handler {
                    if let Some(param) = handler.param {
                        self.collect_functions_in_pattern(param, functions);
                    }
                    self.collect_functions_in_statement(handler.body, functions);
                }
                if let Some(finalizer) = finalizer {
                    self.collect_functions_in_statement(*finalizer, functions);
                }
            }
            Stmt::Declaration { decl, .. } => {
                self.collect_functions_in_declaration(*decl, functions);
            }
        }
    }

    fn collect_functions_in_for_in_left(&self, left: ForInOfLeft, functions: &mut Vec<FunctionId>) {
        match left {
            ForInOfLeft::Declaration(decl) => {
                self.collect_functions_in_declaration(decl, functions);
            }
            ForInOfLeft::Pattern(pattern) => self.collect_functions_in_pattern(pattern, functions),
            ForInOfLeft::Expression(expr) => self.collect_functions_in_expression(expr, functions),
        }
    }

    fn collect_functions_in_declaration(&self, decl_id: DeclId, functions: &mut Vec<FunctionId>) {
        match self.ast().get_decl(decl_id) {
            Decl::Variable { declarators, .. } => {
                for declarator in self.ast().get_var_declarator_list(*declarators) {
                    self.collect_functions_in_pattern(declarator.id, functions);
                    if let Some(init) = declarator.init {
                        self.collect_functions_in_expression(init, functions);
                    }
                }
            }
            Decl::Function { function, .. } => {
                self.collect_functions_in_function(*function, functions);
            }
            Decl::Class {
                super_class, body, ..
            } => {
                if let Some(super_class) = super_class {
                    self.collect_functions_in_expression(*super_class, functions);
                }
                self.collect_functions_in_class_body(*body, functions);
            }
            Decl::Export { kind, .. } => match kind {
                lyng_js_ast::ExportKind::Default { declaration } => match declaration {
                    lyng_js_ast::ExportDefaultDecl::Function(function) => {
                        self.collect_functions_in_function(*function, functions);
                    }
                    lyng_js_ast::ExportDefaultDecl::Class(decl) => {
                        self.collect_functions_in_declaration(*decl, functions);
                    }
                    lyng_js_ast::ExportDefaultDecl::Expression(expr) => {
                        self.collect_functions_in_expression(*expr, functions);
                    }
                },
                lyng_js_ast::ExportKind::Declaration { decl } => {
                    self.collect_functions_in_declaration(*decl, functions);
                }
                lyng_js_ast::ExportKind::Named { .. } | lyng_js_ast::ExportKind::All { .. } => {}
            },
            Decl::Import { .. } | Decl::InvalidDeclaration { .. } => {}
        }
    }

    fn collect_functions_in_pattern(
        &self,
        pattern_id: lyng_js_ast::PatternId,
        functions: &mut Vec<FunctionId>,
    ) {
        match self.ast().get_pattern(pattern_id) {
            Pattern::Identifier { .. } | Pattern::InvalidPattern { .. } => {}
            Pattern::Object {
                properties, rest, ..
            } => {
                for property in self.ast().get_obj_pattern_prop_list(*properties) {
                    if property.computed {
                        self.collect_functions_in_expression(property.key, functions);
                    }
                    self.collect_functions_in_pattern(property.value, functions);
                }
                if let Some(rest) = rest {
                    self.collect_functions_in_pattern(*rest, functions);
                }
            }
            Pattern::Array { elements, rest, .. } => {
                for element in self
                    .ast()
                    .get_opt_pattern_elem_list(*elements)
                    .iter()
                    .flatten()
                {
                    self.collect_functions_in_pattern(element.pattern, functions);
                }
                if let Some(rest) = rest {
                    self.collect_functions_in_pattern(*rest, functions);
                }
            }
            Pattern::Assignment { left, right, .. } => {
                self.collect_functions_in_pattern(*left, functions);
                self.collect_functions_in_expression(*right, functions);
            }
        }
    }

    fn collect_functions_in_expression(&self, expr_id: ExprId, functions: &mut Vec<FunctionId>) {
        match self.ast().get_expr(expr_id) {
            Expr::This { .. }
            | Expr::Super { .. }
            | Expr::Identifier { .. }
            | Expr::NullLiteral { .. }
            | Expr::BooleanLiteral { .. }
            | Expr::NumericLiteral { .. }
            | Expr::StringLiteral { .. }
            | Expr::BigIntLiteral { .. }
            | Expr::RegExpLiteral { .. }
            | Expr::MetaProperty { .. }
            | Expr::InvalidExpression { .. } => {}
            Expr::ArrayExpression { elements, .. } => {
                for element in self.ast().get_opt_expr_list(*elements).iter().flatten() {
                    self.collect_functions_in_expression(*element, functions);
                }
            }
            Expr::ObjectExpression { properties, .. } => {
                for property in self.ast().get_property_list(*properties) {
                    if property.computed {
                        self.collect_functions_in_expression(property.key, functions);
                    }
                    self.collect_functions_in_expression(property.value, functions);
                }
            }
            Expr::FunctionExpression { function, .. }
            | Expr::ArrowFunctionExpression { function, .. } => {
                self.collect_functions_in_function(*function, functions);
            }
            Expr::ClassExpression {
                super_class, body, ..
            } => {
                if let Some(super_class) = super_class {
                    self.collect_functions_in_expression(*super_class, functions);
                }
                self.collect_functions_in_class_body(*body, functions);
            }
            Expr::TemplateLiteral { template, .. } => {
                for &expression in self.ast().templates().get_expressions(*template) {
                    self.collect_functions_in_expression(expression, functions);
                }
            }
            Expr::TaggedTemplateExpression { tag, template, .. } => {
                self.collect_functions_in_expression(*tag, functions);
                for &expression in self.ast().templates().get_expressions(*template) {
                    self.collect_functions_in_expression(expression, functions);
                }
            }
            Expr::UnaryExpression { argument, .. }
            | Expr::UpdateExpression { argument, .. }
            | Expr::AwaitExpression { argument, .. }
            | Expr::SpreadElement { argument, .. }
            | Expr::OptionalChainExpression { base: argument, .. }
            | Expr::ParenthesizedExpression {
                expression: argument,
                ..
            } => self.collect_functions_in_expression(*argument, functions),
            Expr::BinaryExpression { left, right, .. }
            | Expr::LogicalExpression { left, right, .. }
            | Expr::AssignmentExpression { left, right, .. } => {
                self.collect_functions_in_expression(*left, functions);
                self.collect_functions_in_expression(*right, functions);
            }
            Expr::ConditionalExpression {
                test,
                consequent,
                alternate,
                ..
            } => {
                self.collect_functions_in_expression(*test, functions);
                self.collect_functions_in_expression(*consequent, functions);
                self.collect_functions_in_expression(*alternate, functions);
            }
            Expr::SequenceExpression { expressions, .. } => {
                for &expression in self.ast().get_expr_list(*expressions) {
                    self.collect_functions_in_expression(expression, functions);
                }
            }
            Expr::CallExpression {
                callee, arguments, ..
            }
            | Expr::NewExpression {
                callee, arguments, ..
            } => {
                self.collect_functions_in_expression(*callee, functions);
                for &argument in self.ast().get_expr_list(*arguments) {
                    self.collect_functions_in_expression(argument, functions);
                }
            }
            Expr::StaticMemberExpression { object, .. }
            | Expr::PrivateMemberExpression { object, .. } => {
                self.collect_functions_in_expression(*object, functions);
            }
            Expr::ComputedMemberExpression {
                object, property, ..
            } => {
                self.collect_functions_in_expression(*object, functions);
                self.collect_functions_in_expression(*property, functions);
            }
            Expr::PrivateInExpression { object, .. } => {
                self.collect_functions_in_expression(*object, functions);
            }
            Expr::YieldExpression { argument, .. } => {
                if let Some(argument) = argument {
                    self.collect_functions_in_expression(*argument, functions);
                }
            }
            Expr::ImportExpression {
                source, options, ..
            } => {
                self.collect_functions_in_expression(*source, functions);
                if let Some(options) = options {
                    self.collect_functions_in_expression(*options, functions);
                }
            }
        }
    }

    fn collect_functions_in_function(
        &self,
        function_id: FunctionId,
        functions: &mut Vec<FunctionId>,
    ) {
        functions.push(function_id);

        let function = self.ast().get_function(function_id);
        for &parameter in self.ast().get_pattern_list(function.params.params) {
            self.collect_functions_in_pattern(parameter, functions);
        }
        if let Some(rest) = function.params.rest {
            self.collect_functions_in_pattern(rest, functions);
        }
        for &stmt in self.ast().get_stmt_list(function.body) {
            self.collect_functions_in_statement(stmt, functions);
        }
        if let Some(expression) = function.expression_body {
            self.collect_functions_in_expression(expression, functions);
        }
    }

    fn collect_functions_in_class_body(
        &self,
        body: lyng_js_ast::NodeList<lyng_js_ast::ClassElementId>,
        functions: &mut Vec<FunctionId>,
    ) {
        for &element in self.ast().get_class_element_list(body) {
            match self.ast().get_class_element(element) {
                lyng_js_ast::ClassElement::Method {
                    key,
                    value,
                    computed,
                    ..
                } => {
                    if *computed {
                        self.collect_functions_in_expression(*key, functions);
                    }
                    self.collect_functions_in_function(*value, functions);
                }
                lyng_js_ast::ClassElement::Property {
                    key,
                    value,
                    computed,
                    ..
                } => {
                    if *computed {
                        self.collect_functions_in_expression(*key, functions);
                    }
                    if let Some(value) = value {
                        self.collect_functions_in_expression(*value, functions);
                    }
                }
                lyng_js_ast::ClassElement::StaticBlock { body, .. } => {
                    for &stmt in self.ast().get_stmt_list(*body) {
                        self.collect_functions_in_statement(stmt, functions);
                    }
                }
                lyng_js_ast::ClassElement::InvalidElement { .. } => {}
            }
        }
    }
}

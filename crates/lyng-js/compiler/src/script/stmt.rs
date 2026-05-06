use super::*;
use lyng_js_bytecode::DirectEvalSiteFlags;

#[derive(Clone, Copy)]
pub(super) enum ObjectRestExcludedKey {
    Atom(AtomId),
    Register(u16),
}

enum ObjectDestructuringAssignmentTarget {
    Prepared {
        target: PreparedReferenceTarget,
        default_initializer: Option<ExprId>,
    },
    Pattern(ExprId),
}

#[derive(Clone, Copy)]
struct PreparedPrivateAssignmentTarget {
    receiver: u16,
    descriptor: u16,
    depth: u16,
    span: Span,
}

impl<'a, 'b> FunctionCompiler<'a, 'b> {
    pub(super) fn next_child_scope_with_kind(&mut self, kind: ScopeKind) -> Option<ScopeId> {
        let children = self
            .state
            .sema
            .scope_table
            .get(self.current_scope)
            .children
            .clone();
        let cursor = self
            .scope_child_cursors
            .get_mut(self.current_scope.raw() as usize)?;
        while *cursor < children.len() {
            let scope_id = children[*cursor];
            *cursor += 1;
            if self.state.sema.scope_table.get(scope_id).kind == kind {
                return Some(scope_id);
            }
        }
        None
    }

    pub(super) fn peek_child_scope_with_kind(&self, kind: ScopeKind) -> Option<ScopeId> {
        let children = self
            .state
            .sema
            .scope_table
            .get(self.current_scope)
            .children
            .clone();
        let mut cursor = *self
            .scope_child_cursors
            .get(self.current_scope.raw() as usize)?;
        while cursor < children.len() {
            let scope_id = children[cursor];
            cursor += 1;
            if self.state.sema.scope_table.get(scope_id).kind == kind {
                return Some(scope_id);
            }
        }
        None
    }

    pub(super) fn active_direct_eval_scope(&self, scope: ScopeId) -> bool {
        self.state.scope_environment_base(scope).is_some()
    }

    pub(super) fn in_with_scope(&self) -> bool {
        let mut current = Some(self.current_scope);
        while let Some(scope_id) = current {
            let scope = self.state.sema.scope_table.get(scope_id);
            if scope.kind == ScopeKind::With {
                return true;
            }
            if matches!(
                scope.kind,
                ScopeKind::Function | ScopeKind::Parameter | ScopeKind::Global | ScopeKind::Module
            ) {
                return false;
            }
            current = scope.parent;
        }
        false
    }

    pub(super) fn with_child_scope<F>(
        &mut self,
        kind: ScopeKind,
        track_direct_eval: bool,
        failure_stmt: StmtId,
        body: F,
    ) -> LoweringResult<()>
    where
        F: FnOnce(&mut Self) -> LoweringResult<()>,
    {
        let Some(scope) = self.next_child_scope_with_kind(kind) else {
            return Err(LoweringError::UnsupportedStatement { stmt: failure_stmt });
        };
        let previous_scope = self.current_scope;
        self.current_scope = scope;
        let tracked = track_direct_eval && self.active_direct_eval_scope(scope);
        if tracked {
            self.active_direct_eval_scopes.push(scope);
        }

        let result = body(self);

        if tracked {
            let _ = self.active_direct_eval_scopes.pop();
        }
        self.current_scope = previous_scope;
        result
    }

    pub(super) fn active_direct_eval_lexical_scopes(&self) -> Vec<DirectEvalLexicalScope> {
        self.active_direct_eval_scopes
            .iter()
            .filter_map(|scope| {
                let base = self.state.scope_environment_base(*scope)?;
                let bindings = self.state.scope_environment_bindings_for(*scope);
                if bindings.is_empty() {
                    return None;
                }
                let mut lexical_scope = DirectEvalLexicalScope::new(base, bindings);
                if let Some(name) = self.annex_b_simple_catch_name_for_scope(*scope) {
                    lexical_scope = lexical_scope.with_annex_b_catch_name(name);
                }
                Some(lexical_scope)
            })
            .collect()
    }

    pub(super) fn active_direct_eval_parameter_names(&self) -> Vec<AtomId> {
        if self.state.sema.scope_table.get(self.current_scope).kind != ScopeKind::Parameter {
            return Vec::new();
        }

        let mut names = Vec::new();
        for binding_id in self
            .state
            .sema
            .scope_table
            .get(self.current_scope)
            .bindings
            .iter()
            .copied()
        {
            let binding = self.state.sema.binding_table.get(binding_id);
            if binding.kind == DeclarationKind::Parameter && !names.contains(&binding.name) {
                names.push(binding.name);
            }
        }
        names
    }

    fn annex_b_simple_catch_name_for_scope(&self, scope: ScopeId) -> Option<AtomId> {
        let names = self
            .state
            .sema
            .scope_table
            .get(scope)
            .bindings
            .iter()
            .copied()
            .filter_map(|binding_id| {
                let binding = self.state.sema.binding_table.get(binding_id);
                (binding.kind == DeclarationKind::CatchParam).then_some(binding.name)
            })
            .collect::<Vec<_>>();
        (names.len() == 1).then(|| names[0])
    }

    pub(super) fn active_direct_eval_site_flags(&self) -> DirectEvalSiteFlags {
        DirectEvalSiteFlags::empty()
            .with_forbid_arguments_in_class_initializer(self.in_class_field_initializer)
            .with_forbid_super_call_in_class_initializer(self.in_class_field_initializer)
            .with_allow_new_target(self.state.sema.direct_eval_allows_new_target)
            .with_allow_super(self.state.sema.direct_eval_allows_super)
    }

    pub(super) fn active_direct_eval_annex_b_catch_names(&self) -> Vec<AtomId> {
        let mut names = Vec::new();
        let mut current = Some(self.current_scope);
        while let Some(scope) = current {
            if let Some(name) = self.annex_b_simple_catch_name_for_scope(scope) {
                if !names.contains(&name) {
                    names.push(name);
                }
            }
            current = self.state.sema.scope_table.get(scope).parent;
        }
        names
    }

    pub(super) fn active_simple_catch_binding_for_name(
        &self,
        name: AtomId,
    ) -> Option<SemanticBindingId> {
        let mut current = Some(self.current_scope);
        while let Some(scope) = current {
            let record = self.state.sema.scope_table.get(scope);
            if record.kind == ScopeKind::Catch {
                let mut matches = record.bindings.iter().copied().filter(|&binding_id| {
                    let binding = self.state.sema.binding_table.get(binding_id);
                    binding.kind == DeclarationKind::CatchParam && binding.name == name
                });
                if let Some(first) = matches.next() {
                    if matches.next().is_none() {
                        return Some(first);
                    }
                    return None;
                }
            }
            if matches!(
                record.kind,
                ScopeKind::Function | ScopeKind::Global | ScopeKind::Module
            ) {
                return None;
            }
            current = record.parent;
        }
        None
    }

    fn merge_disposal_scope_kind(
        current: Option<lyng_js_env::DisposalCapabilityKind>,
        next: Option<lyng_js_env::DisposalCapabilityKind>,
    ) -> Option<lyng_js_env::DisposalCapabilityKind> {
        match (current, next) {
            (Some(lyng_js_env::DisposalCapabilityKind::Async), _)
            | (_, Some(lyng_js_env::DisposalCapabilityKind::Async)) => {
                Some(lyng_js_env::DisposalCapabilityKind::Async)
            }
            (Some(lyng_js_env::DisposalCapabilityKind::Sync), _)
            | (_, Some(lyng_js_env::DisposalCapabilityKind::Sync)) => {
                Some(lyng_js_env::DisposalCapabilityKind::Sync)
            }
            (None, None) => None,
        }
    }

    fn decl_disposal_scope_kind(
        &self,
        decl_id: DeclId,
    ) -> Option<lyng_js_env::DisposalCapabilityKind> {
        match self.ast().get_decl(decl_id) {
            Decl::Variable {
                kind: VariableKind::Using,
                ..
            } => Some(lyng_js_env::DisposalCapabilityKind::Sync),
            Decl::Variable {
                kind: VariableKind::AwaitUsing,
                ..
            } => Some(lyng_js_env::DisposalCapabilityKind::Async),
            Decl::Export { kind, .. } => match kind {
                lyng_js_ast::ExportKind::Declaration { decl } => {
                    self.decl_disposal_scope_kind(*decl)
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn statement_list_disposal_scope_kind(
        &self,
        list: lyng_js_ast::NodeList<StmtId>,
    ) -> Option<lyng_js_env::DisposalCapabilityKind> {
        let mut kind = None;
        for &stmt in self.ast().get_stmt_list(list) {
            if let Stmt::Declaration { decl, .. } = self.ast().get_stmt(stmt) {
                kind = Self::merge_disposal_scope_kind(kind, self.decl_disposal_scope_kind(*decl));
            }
        }
        kind
    }

    fn switch_disposal_scope_kind(
        &self,
        cases: lyng_js_ast::NodeList<SwitchCase>,
    ) -> Option<lyng_js_env::DisposalCapabilityKind> {
        let mut kind = None;
        for case in self.ast().get_switch_case_list(cases) {
            kind = Self::merge_disposal_scope_kind(
                kind,
                self.statement_list_disposal_scope_kind(case.consequent),
            );
        }
        kind
    }

    pub(super) fn for_init_disposal_scope_kind(
        &self,
        init: Option<ForInit>,
    ) -> Option<lyng_js_env::DisposalCapabilityKind> {
        let ForInit::Declaration(decl) = init? else {
            return None;
        };
        self.decl_disposal_scope_kind(decl)
    }

    pub(super) fn for_in_of_declaration_disposal_scope_kind(
        &self,
        left: ForInOfLeft,
    ) -> Option<lyng_js_env::DisposalCapabilityKind> {
        let ForInOfLeft::Declaration(decl) = left else {
            return None;
        };
        self.decl_disposal_scope_kind(decl)
    }

    fn active_disposal_scope_for_kind(
        &self,
        kind: VariableKind,
    ) -> Option<super::state::ActiveDisposalScope> {
        let scope = self.active_disposal_scopes.last().copied()?;
        match kind {
            VariableKind::Using => Some(scope),
            VariableKind::AwaitUsing
                if scope.kind == lyng_js_env::DisposalCapabilityKind::Async =>
            {
                Some(scope)
            }
            _ => None,
        }
    }

    pub(super) fn lower_statement_list_with_disposal(
        &mut self,
        list: lyng_js_ast::NodeList<StmtId>,
        span: Span,
    ) -> LoweringResult<()> {
        let env_scope = self.current_dynamic_env_scope_range()?;
        if let Some((base, count)) = env_scope {
            self.emit_enter_env_scope(base, count)?;
        }
        self.emit_frame_local_tdz_initializers_for_current_scope()?;
        let stmts = self.ast().get_stmt_list(list).to_vec();
        if let Some(kind) = self.statement_list_disposal_scope_kind(list) {
            let result = self.with_disposal_scope(kind, span, move |this| {
                for stmt in stmts {
                    this.lower_statement(stmt)?;
                }
                Ok(())
            });
            if let Some((base, count)) = env_scope {
                self.emit_leave_env_scope(base, count)?;
            }
            return result;
        }
        for stmt in stmts {
            self.lower_statement(stmt)?;
        }
        if let Some((base, count)) = env_scope {
            self.emit_leave_env_scope(base, count)?;
        }
        Ok(())
    }

    pub(super) fn current_dynamic_env_scope_range(&self) -> LoweringResult<Option<(u16, u32)>> {
        let Some(base) = self.state.scope_environment_base(self.current_scope) else {
            return Ok(None);
        };
        let bindings = self
            .state
            .scope_environment_bindings_for(self.current_scope);
        if bindings.is_empty() || !bindings.iter().any(|binding| binding.flags().is_scoped()) {
            return Ok(None);
        }
        let base = u16::try_from(base)
            .map_err(|_| LoweringError::ConstantIndexOverflow { index: base })?;
        Ok(Some((
            base,
            u32::try_from(bindings.len()).unwrap_or(u32::MAX),
        )))
    }

    pub(super) fn emit_frame_local_tdz_initializers_for_current_scope(
        &mut self,
    ) -> LoweringResult<()> {
        let bindings = self
            .state
            .sema
            .scope_table
            .get(self.current_scope)
            .bindings
            .clone();
        for binding_id in bindings {
            let binding = self.binding(binding_id)?;
            if binding.storage_class != StorageClass::FrameLocal || !binding.has_tdz {
                continue;
            }
            let register = self.ensure_local_register(binding_id)?;
            self.emit_load_uninitialized_lexical(register)?;
        }
        Ok(())
    }

    fn emit_disposal_scope_cleanup(
        &mut self,
        scope: super::state::ActiveDisposalScope,
        span: Span,
    ) -> LoweringResult<()> {
        let Some(registers) = self.completion_registers else {
            return self.emit_disposal_scope_cleanup_call(scope, span, None);
        };
        let throw_constant = self.alloc_temp()?;
        let throw_match = self.alloc_temp()?;
        self.emit_load_smi(throw_constant, CompletionKind::Throw.encoded())?;
        self.builder.emit_abc(
            Opcode::StrictEqual,
            self.encode_register(throw_match)?,
            self.encode_register(registers.kind)?,
            self.encode_register(throw_constant)?,
        )?;
        let jump_without_prior = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfFalse, self.encode_register(throw_match)?)?;
        self.emit_disposal_scope_cleanup_call(scope, span, Some(registers.value))?;
        let jump_end = self.builder.emit_jump_placeholder(Opcode::Jump)?;
        let without_prior = self.builder.current_offset()?;
        self.builder
            .patch_jump_to(jump_without_prior, without_prior)?;
        self.emit_disposal_scope_cleanup_call(scope, span, None)?;
        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(jump_end, end)?;
        Ok(())
    }

    fn emit_disposal_scope_cleanup_call(
        &mut self,
        scope: super::state::ActiveDisposalScope,
        span: Span,
        prior_error: Option<u16>,
    ) -> LoweringResult<()> {
        let mut arguments = vec![scope.capability_register];
        if let Some(prior_error) = prior_error {
            arguments.push(prior_error);
        }
        match scope.kind {
            lyng_js_env::DisposalCapabilityKind::Sync => {
                self.emit_internal_builtin_call(dispose_scope_builtin(), &arguments, span)
            }
            lyng_js_env::DisposalCapabilityKind::Async => {
                let promise = self.alloc_temp()?;
                self.emit_internal_builtin_call_into(
                    dispose_scope_async_builtin(),
                    &arguments,
                    span,
                    promise,
                )?;
                let undefined = self.alloc_temp()?;
                self.emit_load_undefined(undefined)?;
                let is_empty = self.alloc_temp()?;
                self.builder.emit_abc(
                    Opcode::StrictEqual,
                    self.encode_register(is_empty)?,
                    self.encode_register(promise)?,
                    self.encode_register(undefined)?,
                )?;
                let skip_await = self.builder.emit_cond_jump_placeholder(
                    Opcode::JumpIfTrue,
                    self.encode_register(is_empty)?,
                )?;
                self.builder.emit_ax(Opcode::Await, i32::from(promise))?;
                let end = self.builder.current_offset()?;
                self.builder.patch_jump_to(skip_await, end)?;
                Ok(())
            }
        }
    }

    pub(super) fn with_disposal_scope<F>(
        &mut self,
        kind: lyng_js_env::DisposalCapabilityKind,
        span: Span,
        body: F,
    ) -> LoweringResult<()>
    where
        F: FnOnce(&mut Self) -> LoweringResult<()>,
    {
        let capability_register = self.alloc_temp()?;
        let create_builtin = match kind {
            lyng_js_env::DisposalCapabilityKind::Sync => create_sync_disposal_scope_builtin(),
            lyng_js_env::DisposalCapabilityKind::Async => create_async_disposal_scope_builtin(),
        };
        self.emit_internal_builtin_call_into(create_builtin, &[], span, capability_register)?;
        let scope = super::state::ActiveDisposalScope {
            capability_register,
            kind,
        };
        self.active_disposal_scopes.push(scope);

        let finally_index = self.push_finally_context();
        let protected_start = self.builder.current_offset()?;
        if let Err(error) = body(self) {
            let _ = self.active_disposal_scopes.pop();
            self.pop_finally_context(finally_index);
            return Err(error);
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
        self.emit_disposal_scope_cleanup(scope, span)?;
        self.emit_leave_handler()?;
        self.emit_finally_dispatch(finally_index)?;
        self.mark_finally_body(finally_index, false);
        self.pop_finally_context(finally_index);
        let _ = self.active_disposal_scopes.pop();

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

    pub(super) fn lower_statement(&mut self, stmt_id: StmtId) -> LoweringResult<()> {
        self.lower_statement_with_label(stmt_id, None)
    }

    pub(super) fn lower_statement_with_label(
        &mut self,
        stmt_id: StmtId,
        pending_label: Option<AtomId>,
    ) -> LoweringResult<()> {
        let stmt = self.ast().get_stmt(stmt_id).clone();
        match stmt {
            Stmt::Block { body, span } => {
                self.with_child_scope(ScopeKind::Block, true, stmt_id, |this| {
                    this.emit_block_function_declaration_instantiations(body)?;
                    this.lower_statement_list_with_disposal(body, span)
                })
            }
            Stmt::Empty { .. } | Stmt::Debugger { .. } => Ok(()),
            Stmt::Expression { expression, .. } => {
                if let Some(result_register) = self.result_register {
                    self.lower_expr_into(expression, result_register)
                } else {
                    let temp = self.alloc_temp()?;
                    self.lower_expr_into(expression, temp)
                }
            }
            Stmt::Labeled { label, body, .. } => {
                if self.statement_supports_continue_label(body) {
                    self.lower_statement_with_label(body, Some(label))
                } else {
                    let target = self.push_control_target(Some(label), ControlTargetKind::Label);
                    self.lower_statement(body)?;
                    let end = self.builder.current_offset()?;
                    self.patch_break_placeholders(target, end)?;
                    self.pop_control_target(target);
                    Ok(())
                }
            }
            Stmt::If {
                test,
                consequent,
                alternate,
                ..
            } => {
                self.reset_statement_result()?;
                let test_register = self.lower_expr_to_temp(test)?;
                let jump_false = self.builder.emit_cond_jump_placeholder(
                    Opcode::JumpIfFalse,
                    self.encode_register(test_register)?,
                )?;
                self.lower_statement(consequent)?;
                if let Some(alternate) = alternate {
                    let jump_end = self.builder.emit_jump_placeholder(Opcode::Jump)?;
                    let alternate_offset = self.builder.current_offset()?;
                    self.builder.patch_jump_to(jump_false, alternate_offset)?;
                    self.lower_statement(alternate)?;
                    let end_offset = self.builder.current_offset()?;
                    self.builder.patch_jump_to(jump_end, end_offset)?;
                } else {
                    let end_offset = self.builder.current_offset()?;
                    self.builder.patch_jump_to(jump_false, end_offset)?;
                }
                Ok(())
            }
            Stmt::DoWhile {
                body, test, span, ..
            } => self.lower_do_while_statement(pending_label, body, test, span),
            Stmt::While {
                test, body, span, ..
            } => self.lower_while_statement(pending_label, test, body, span),
            Stmt::For {
                init,
                test,
                update,
                body,
                span,
                ..
            } => self.lower_for_statement(pending_label, init, test, update, body, span),
            Stmt::ForIn {
                left,
                right,
                body,
                span,
                ..
            } => self.lower_for_in_statement(pending_label, left, right, body, span),
            Stmt::ForOf {
                left,
                right,
                body,
                r#await,
                span,
                ..
            } => self.lower_for_of_statement(pending_label, left, right, body, r#await, span),
            Stmt::Switch {
                discriminant,
                cases,
                span,
                ..
            } => self.with_child_scope(ScopeKind::Switch, true, stmt_id, |this| {
                this.lower_switch_statement(pending_label, discriminant, cases, span)
            }),
            Stmt::Break { label, .. } => self.lower_break_statement(label),
            Stmt::Continue { label, .. } => self.lower_continue_statement(label),
            Stmt::Throw { argument, .. } => {
                let value = self.lower_expr_to_temp(argument)?;
                self.builder.emit_ax(Opcode::Throw, i32::from(value))?;
                Ok(())
            }
            Stmt::Try {
                block,
                handler,
                finalizer,
                ..
            } => self.lower_try_statement(block, handler, finalizer),
            Stmt::With { object, body, span } => {
                self.with_child_scope(ScopeKind::With, false, stmt_id, |this| {
                    this.lower_with_statement(object, body, span)
                })
            }
            Stmt::Return { argument, .. } => {
                if self.current_function.is_none() {
                    return Err(LoweringError::UnsupportedStatement { stmt: stmt_id });
                }
                self.lower_return_statement(argument)
            }
            Stmt::Declaration { decl, .. } => self.lower_declaration(decl),
            _ => Err(LoweringError::UnsupportedStatement { stmt: stmt_id }),
        }
    }

    pub(super) fn lower_with_statement(
        &mut self,
        object: ExprId,
        body: StmtId,
        span: Span,
    ) -> LoweringResult<()> {
        if let Some(result_register) = self.result_register {
            self.emit_load_undefined(result_register)?;
        }
        let object_register = self.lower_expr_to_temp(object)?;
        self.emit_push_with_env(object_register, self.ast().get_expr(object).span())?;

        let finally_index = self.push_finally_context();
        let protected_start = self.builder.current_offset()?;
        self.lower_statement(body)?;
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
        self.emit_pop_with_env()?;
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

    pub(super) fn lower_switch_statement(
        &mut self,
        label: Option<AtomId>,
        discriminant: ExprId,
        cases: lyng_js_ast::NodeList<SwitchCase>,
        span: Span,
    ) -> LoweringResult<()> {
        self.reset_statement_result()?;
        if let Some(kind) = self.switch_disposal_scope_kind(cases) {
            return self.with_disposal_scope(kind, span, move |this| {
                this.lower_switch_statement_core(label, discriminant, cases)
            });
        }
        self.lower_switch_statement_core(label, discriminant, cases)
    }

    fn lower_switch_statement_core(
        &mut self,
        label: Option<AtomId>,
        discriminant: ExprId,
        cases: lyng_js_ast::NodeList<SwitchCase>,
    ) -> LoweringResult<()> {
        let discriminant_register = self.lower_expr_to_temp(discriminant)?;
        self.emit_frame_local_tdz_initializers_for_current_scope()?;
        let cases = self.ast().get_switch_case_list(cases).to_vec();
        self.emit_switch_function_declaration_instantiations(&cases)?;
        let switch_target = self.push_control_target(label, ControlTargetKind::Switch);
        let mut case_body_offsets = vec![None; cases.len()];
        let mut case_match_jumps = Vec::new();
        let mut default_index = None;

        for (index, case) in cases.iter().enumerate() {
            let Some(test) = case.test else {
                default_index = Some(index);
                continue;
            };
            let test_register = self.lower_expr_to_temp(test)?;
            let match_register = self.alloc_temp()?;
            self.builder.emit_abc(
                Opcode::StrictEqual,
                self.encode_register(match_register)?,
                self.encode_register(discriminant_register)?,
                self.encode_register(test_register)?,
            )?;
            let jump = self.builder.emit_cond_jump_placeholder(
                Opcode::JumpIfTrue,
                self.encode_register(match_register)?,
            )?;
            case_match_jumps.push((index, jump));
        }

        let jump_fallback = self.builder.emit_jump_placeholder(Opcode::Jump)?;
        for (index, case) in cases.iter().enumerate() {
            case_body_offsets[index] = Some(self.builder.current_offset()?);
            for stmt in self.ast().get_stmt_list(case.consequent).to_vec() {
                self.lower_statement(stmt)?;
            }
        }

        let end = self.builder.current_offset()?;
        let fallback = default_index
            .and_then(|index| case_body_offsets[index])
            .unwrap_or(end);
        self.builder.patch_jump_to(jump_fallback, fallback)?;
        for (index, jump) in case_match_jumps {
            let target = case_body_offsets[index].unwrap_or(end);
            self.builder.patch_jump_to(jump, target)?;
        }
        self.patch_break_placeholders(switch_target, end)?;
        self.pop_control_target(switch_target);
        Ok(())
    }

    fn emit_switch_function_declaration_instantiations(
        &mut self,
        cases: &[SwitchCase],
    ) -> LoweringResult<()> {
        for case in cases {
            self.emit_block_function_declaration_instantiations(case.consequent)?;
        }
        Ok(())
    }

    pub(super) fn lower_assignment_target_from_register(
        &mut self,
        expr_id: ExprId,
        value_register: u16,
    ) -> LoweringResult<()> {
        if self.lower_annex_b_call_assignment_target_reference_error(expr_id)? {
            return Ok(());
        }

        match self.ast().get_expr(expr_id).clone() {
            Expr::ParenthesizedExpression { expression, .. } => {
                self.lower_assignment_target_from_register(expression, value_register)
            }
            Expr::Identifier { name, .. } => {
                let use_site = self.use_site(expr_id)?;
                match use_site.resolution_kind {
                    ResolutionKind::Local | ResolutionKind::Captured => {
                        let binding_id = use_site.resolved_binding.ok_or(
                            LoweringError::MissingResolvedBinding {
                                expr: expr_id,
                                name,
                            },
                        )?;
                        self.assign_binding_value(binding_id, name, value_register)
                    }
                    ResolutionKind::Global | ResolutionKind::Unresolved => {
                        self.emit_assign_global(value_register, name)
                    }
                    ResolutionKind::Dynamic => self.emit_assign_name(value_register, name),
                }
            }
            Expr::StaticMemberExpression {
                object, property, ..
            } => {
                if matches!(self.ast().get_expr(object), Expr::Super { .. }) {
                    let target = self
                        .prepare_reference_target(expr_id, ReferenceUsage::WriteOnly)?
                        .ok_or(LoweringError::UnsupportedExpression { expr: expr_id })?;
                    return self.assign_prepared_reference(target, value_register);
                }
                let object_register = self.lower_expr_to_temp(object)?;
                self.emit_assign_property_by_atom(object_register, value_register, property)
            }
            Expr::ComputedMemberExpression {
                object, property, ..
            } => {
                if matches!(self.ast().get_expr(object), Expr::Super { .. }) {
                    let target = self
                        .prepare_reference_target(expr_id, ReferenceUsage::WriteOnly)?
                        .ok_or(LoweringError::UnsupportedExpression { expr: expr_id })?;
                    return self.assign_prepared_reference(target, value_register);
                }
                let object_register = self.lower_expr_to_temp(object)?;
                let key_register = self.lower_expr_to_temp(property)?;
                self.emit_assign_keyed_property(object_register, value_register, key_register)?;
                Ok(())
            }
            Expr::PrivateMemberExpression { .. } => {
                let target = self
                    .prepare_private_assignment_target(expr_id)?
                    .ok_or(LoweringError::UnsupportedExpression { expr: expr_id })?;
                self.assign_prepared_private_target(target, value_register)
            }
            _ => Err(LoweringError::UnsupportedExpression { expr: expr_id }),
        }
    }

    pub(super) fn lower_destructuring_assignment_from_register(
        &mut self,
        expr_id: ExprId,
        source_register: u16,
    ) -> LoweringResult<()> {
        match self.ast().get_expr(expr_id).clone() {
            Expr::ParenthesizedExpression { expression, .. } => {
                self.lower_destructuring_assignment_from_register(expression, source_register)
            }
            Expr::Identifier { .. }
            | Expr::StaticMemberExpression { .. }
            | Expr::ComputedMemberExpression { .. }
            | Expr::PrivateMemberExpression { .. } => {
                self.lower_assignment_target_from_register(expr_id, source_register)
            }
            Expr::AssignmentExpression {
                operator: AssignOp::Assign,
                left,
                right,
                ..
            } => {
                let undefined = self.alloc_temp()?;
                self.emit_load_undefined(undefined)?;
                let is_undefined = self.alloc_temp()?;
                self.emit_profiled_binary(
                    Opcode::StrictEqual,
                    is_undefined,
                    source_register,
                    undefined,
                )?;
                let use_source = self.builder.emit_cond_jump_placeholder(
                    Opcode::JumpIfFalse,
                    self.encode_register(is_undefined)?,
                )?;
                let default_value = self.alloc_temp()?;
                self.lower_initializer_with_inferred_name(
                    right,
                    self.assignment_target_name(left),
                    default_value,
                )?;
                self.lower_destructuring_assignment_from_register(left, default_value)?;
                let end = self.builder.emit_jump_placeholder(Opcode::Jump)?;
                let use_source_offset = self.builder.current_offset()?;
                self.builder.patch_jump_to(use_source, use_source_offset)?;
                self.lower_destructuring_assignment_from_register(left, source_register)?;
                let end_offset = self.builder.current_offset()?;
                self.builder.patch_jump_to(end, end_offset)?;
                Ok(())
            }
            Expr::ArrayExpression { elements, .. } => self
                .lower_array_destructuring_assignment_from_iterator(
                    elements,
                    source_register,
                    self.ast().get_expr(expr_id).span(),
                ),
            Expr::ObjectExpression { properties, .. } => {
                self.emit_check_object_coercible(source_register)?;
                let mut rest_target = None;
                let mut excluded_keys = Vec::new();
                for property in self.ast().get_property_list(properties).to_vec() {
                    if property.method {
                        return Err(LoweringError::UnsupportedExpression {
                            expr: property.value,
                        });
                    }
                    if let Expr::SpreadElement { argument, .. } =
                        self.ast().get_expr(property.value).clone()
                    {
                        rest_target = Some(argument);
                        continue;
                    }
                    let value = self.alloc_temp()?;
                    let source_key = if !property.computed {
                        if let Some(atom) = self.named_property_atom(property.key)? {
                            ObjectRestExcludedKey::Atom(atom)
                        } else {
                            self.lower_object_destructuring_property_key(property.key)?
                        }
                    } else {
                        self.lower_object_destructuring_property_key(property.key)?
                    };
                    let target = self.prepare_object_destructuring_assignment_target(
                        property.shorthand,
                        property.key,
                        property.value,
                    )?;
                    match source_key {
                        ObjectRestExcludedKey::Atom(atom) => {
                            self.emit_get_property_by_atom(value, source_register, atom)?;
                        }
                        ObjectRestExcludedKey::Register(key) => {
                            self.emit_get_keyed_property(value, source_register, key)?;
                        }
                    }
                    excluded_keys.push(source_key);
                    match target {
                        ObjectDestructuringAssignmentTarget::Prepared {
                            target,
                            default_initializer,
                        } => self.assign_array_element_value_to_prepared_target(
                            target,
                            value,
                            default_initializer,
                        )?,
                        ObjectDestructuringAssignmentTarget::Pattern(target_expr) => {
                            self.lower_destructuring_assignment_from_register(target_expr, value)?;
                        }
                    }
                }
                if let Some(rest_target) = rest_target {
                    self.lower_object_rest_assignment_from_register(
                        rest_target,
                        source_register,
                        &excluded_keys,
                    )?;
                }
                Ok(())
            }
            _ => Err(LoweringError::UnsupportedExpression { expr: expr_id }),
        }
    }

    fn lower_object_destructuring_property_key(
        &mut self,
        key_expr: ExprId,
    ) -> LoweringResult<ObjectRestExcludedKey> {
        let raw_key = self.lower_expr_to_temp(key_expr)?;
        let key = self.alloc_temp()?;
        self.emit_to_property_key(key, raw_key)?;
        Ok(ObjectRestExcludedKey::Register(key))
    }

    fn prepare_object_destructuring_assignment_target(
        &mut self,
        shorthand: bool,
        property_key: ExprId,
        property_value: ExprId,
    ) -> LoweringResult<ObjectDestructuringAssignmentTarget> {
        if shorthand && property_value != property_key {
            let target = self
                .prepare_reference_target(property_key, ReferenceUsage::WriteOnly)?
                .ok_or(LoweringError::UnsupportedExpression { expr: property_key })?;
            return Ok(ObjectDestructuringAssignmentTarget::Prepared {
                target,
                default_initializer: Some(property_value),
            });
        }

        let target_expr = if shorthand {
            property_key
        } else {
            property_value
        };
        match self.ast().get_expr(target_expr).clone() {
            Expr::AssignmentExpression {
                operator: AssignOp::Assign,
                left,
                right,
                ..
            } if !self.assignment_rest_target_is_pattern(left) => {
                let target = self
                    .prepare_reference_target(left, ReferenceUsage::WriteOnly)?
                    .ok_or(LoweringError::UnsupportedExpression { expr: left })?;
                Ok(ObjectDestructuringAssignmentTarget::Prepared {
                    target,
                    default_initializer: Some(right),
                })
            }
            _ if !self.assignment_rest_target_is_pattern(target_expr) => {
                let target = self
                    .prepare_reference_target(target_expr, ReferenceUsage::WriteOnly)?
                    .ok_or(LoweringError::UnsupportedExpression { expr: target_expr })?;
                Ok(ObjectDestructuringAssignmentTarget::Prepared {
                    target,
                    default_initializer: None,
                })
            }
            _ => Ok(ObjectDestructuringAssignmentTarget::Pattern(target_expr)),
        }
    }

    fn prepare_private_assignment_target(
        &mut self,
        expr_id: ExprId,
    ) -> LoweringResult<Option<PreparedPrivateAssignmentTarget>> {
        match self.ast().get_expr(expr_id).clone() {
            Expr::ParenthesizedExpression { expression, .. } => {
                self.prepare_private_assignment_target(expression)
            }
            Expr::PrivateMemberExpression {
                object, property, ..
            } => {
                let (descriptor_index, class_depth) =
                    self.resolved_private_field_access(expr_id, property, true)?;
                let receiver = self.lower_expr_to_temp(object)?;
                let descriptor = self.alloc_temp()?;
                let descriptor_smi = i16::try_from(descriptor_index)
                    .map_err(|_| LoweringError::UnsupportedExpression { expr: expr_id })?;
                self.emit_load_smi(descriptor, descriptor_smi)?;
                let depth = self.alloc_temp()?;
                let depth_smi = i16::try_from(class_depth)
                    .map_err(|_| LoweringError::UnsupportedExpression { expr: expr_id })?;
                self.emit_load_smi(depth, depth_smi)?;
                Ok(Some(PreparedPrivateAssignmentTarget {
                    receiver,
                    descriptor,
                    depth,
                    span: self.ast().get_expr(expr_id).span(),
                }))
            }
            _ => Ok(None),
        }
    }

    fn assign_prepared_private_target(
        &mut self,
        target: PreparedPrivateAssignmentTarget,
        value_register: u16,
    ) -> LoweringResult<()> {
        let sink = self.alloc_temp()?;
        self.emit_internal_builtin_call_into(
            internal_private_field_set_builtin(),
            &[
                target.receiver,
                target.descriptor,
                value_register,
                target.depth,
            ],
            target.span,
            sink,
        )
    }

    fn lower_array_destructuring_assignment_from_iterator(
        &mut self,
        elements: lyng_js_ast::NodeList<Option<ExprId>>,
        source_register: u16,
        span: Span,
    ) -> LoweringResult<()> {
        let iterator_register = self.alloc_temp()?;
        self.builder.emit_abc(
            Opcode::CreateIterator,
            self.encode_register(iterator_register)?,
            self.encode_register(source_register)?,
            0,
        )?;

        let finally_index = self.push_finally_context();
        let protected_start = self.builder.current_offset()?;
        if let Err(error) =
            self.lower_array_destructuring_assignment_elements(elements, iterator_register)
        {
            self.pop_finally_context(finally_index);
            return Err(error);
        }
        let protected_end = self.builder.current_offset()?;
        self.set_completion_state(CompletionKind::Normal, None, None)?;
        self.emit_jump_to_finally(finally_index)?;

        let throw_entry = self.builder.current_offset()?;
        let enter_handler = self.emit_enter_handler()?;
        self.attach_safepoint(enter_handler, span, SafepointKind::ExceptionEdge)?;
        self.begin_exception_finally_path()?;
        let normal_entry = self.builder.current_offset()?;
        self.set_finally_normal_entry(finally_index, normal_entry)?;
        self.mark_finally_body(finally_index, true);
        self.emit_close_iterator_for_completion(iterator_register)?;
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

    fn lower_array_destructuring_assignment_elements(
        &mut self,
        elements: lyng_js_ast::NodeList<Option<ExprId>>,
        iterator_register: u16,
    ) -> LoweringResult<()> {
        let value_register = self.alloc_temp()?;
        let done_register = self.alloc_temp()?;
        for element in self.ast().get_opt_expr_list(elements).to_vec() {
            let Some(element) = element else {
                self.emit_advance_iterator(iterator_register, value_register, done_register)?;
                continue;
            };
            match self.ast().get_expr(element).clone() {
                Expr::SpreadElement { argument, .. } => {
                    self.lower_array_rest_assignment_from_iterator(argument, iterator_register)?;
                    break;
                }
                _ => self.lower_array_assignment_element_from_iterator(
                    element,
                    iterator_register,
                    value_register,
                    done_register,
                )?,
            }
        }

        Ok(())
    }

    fn emit_advance_iterator(
        &mut self,
        iterator_register: u16,
        value_register: u16,
        done_register: u16,
    ) -> LoweringResult<()> {
        self.builder.emit_abc(
            Opcode::AdvanceIterator,
            self.encode_register(iterator_register)?,
            self.encode_register(value_register)?,
            self.encode_register(done_register)?,
        )?;
        Ok(())
    }

    fn lower_array_assignment_element_from_iterator(
        &mut self,
        element: ExprId,
        iterator_register: u16,
        value_register: u16,
        done_register: u16,
    ) -> LoweringResult<()> {
        match self.ast().get_expr(element).clone() {
            Expr::AssignmentExpression {
                operator: AssignOp::Assign,
                left,
                right,
                ..
            } if !self.assignment_rest_target_is_pattern(left) => {
                let target = self
                    .prepare_reference_target(left, ReferenceUsage::WriteOnly)?
                    .ok_or(LoweringError::UnsupportedExpression { expr: left })?;
                self.emit_advance_iterator(iterator_register, value_register, done_register)?;
                self.assign_array_element_value_to_prepared_target(
                    target,
                    value_register,
                    Some(right),
                )
            }
            Expr::AssignmentExpression {
                operator: AssignOp::Assign,
                ..
            } => {
                self.emit_advance_iterator(iterator_register, value_register, done_register)?;
                self.lower_destructuring_assignment_from_register(element, value_register)
            }
            _ if !self.assignment_rest_target_is_pattern(element) => {
                let target = self
                    .prepare_reference_target(element, ReferenceUsage::WriteOnly)?
                    .ok_or(LoweringError::UnsupportedExpression { expr: element })?;
                self.emit_advance_iterator(iterator_register, value_register, done_register)?;
                self.assign_array_element_value_to_prepared_target(target, value_register, None)
            }
            _ => {
                self.emit_advance_iterator(iterator_register, value_register, done_register)?;
                self.lower_destructuring_assignment_from_register(element, value_register)
            }
        }
    }

    fn assign_array_element_value_to_prepared_target(
        &mut self,
        target: PreparedReferenceTarget,
        value_register: u16,
        default_initializer: Option<ExprId>,
    ) -> LoweringResult<()> {
        let Some(default_initializer) = default_initializer else {
            return self.assign_prepared_reference(target, value_register);
        };

        let undefined = self.alloc_temp()?;
        self.emit_load_undefined(undefined)?;
        let is_undefined = self.alloc_temp()?;
        self.emit_profiled_binary(Opcode::StrictEqual, is_undefined, value_register, undefined)?;
        let use_source = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfFalse, self.encode_register(is_undefined)?)?;
        let default_value = self.alloc_temp()?;
        self.lower_initializer_with_inferred_name(
            default_initializer,
            self.reference_target_inferred_name(target),
            default_value,
        )?;
        self.assign_prepared_reference(target, default_value)?;
        let end = self.builder.emit_jump_placeholder(Opcode::Jump)?;
        let use_source_offset = self.builder.current_offset()?;
        self.builder.patch_jump_to(use_source, use_source_offset)?;
        self.assign_prepared_reference(target, value_register)?;
        let end_offset = self.builder.current_offset()?;
        self.builder.patch_jump_to(end, end_offset)?;
        Ok(())
    }

    pub(super) fn emit_close_iterator_for_completion(
        &mut self,
        iterator_register: u16,
    ) -> LoweringResult<()> {
        let registers = self.ensure_completion_registers()?;
        let throw_kind = self.alloc_temp()?;
        self.emit_load_smi(throw_kind, CompletionKind::Throw.encoded())?;
        let is_throw = self.alloc_temp()?;
        self.builder.emit_abc(
            Opcode::StrictEqual,
            self.encode_register(is_throw)?,
            self.encode_register(registers.kind)?,
            self.encode_register(throw_kind)?,
        )?;
        let close_without_preserving = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfFalse, self.encode_register(is_throw)?)?;
        self.builder.emit_abx(
            Opcode::CloseIterator,
            self.encode_register(iterator_register)?,
            1,
        )?;
        let end = self.builder.emit_jump_placeholder(Opcode::Jump)?;
        let close_without_preserving_offset = self.builder.current_offset()?;
        self.builder
            .patch_jump_to(close_without_preserving, close_without_preserving_offset)?;
        self.builder.emit_abx(
            Opcode::CloseIterator,
            self.encode_register(iterator_register)?,
            0,
        )?;
        let end_offset = self.builder.current_offset()?;
        self.builder.patch_jump_to(end, end_offset)?;
        Ok(())
    }

    fn lower_array_rest_assignment_from_iterator(
        &mut self,
        target_expr: ExprId,
        iterator_register: u16,
    ) -> LoweringResult<()> {
        let prepared_target = if self.assignment_rest_target_is_pattern(target_expr) {
            None
        } else {
            Some(
                self.prepare_reference_target(target_expr, ReferenceUsage::WriteOnly)?
                    .ok_or(LoweringError::UnsupportedExpression { expr: target_expr })?,
            )
        };
        let rest_value = self.alloc_temp()?;
        let instruction_offset =
            self.builder
                .emit_abx(Opcode::CreateArray, self.encode_register(rest_value)?, 0)?;
        self.attach_safepoint(
            instruction_offset,
            self.ast().get_expr(target_expr).span(),
            SafepointKind::Allocation,
        )?;

        let rest_index = self.alloc_temp()?;
        self.emit_load_smi(rest_index, 0)?;
        let one_register = self.alloc_temp()?;
        self.emit_load_smi(one_register, 1)?;
        let element_value = self.alloc_temp()?;
        let done_register = self.alloc_temp()?;

        let loop_start = self.builder.current_offset()?;
        self.builder.emit_abc(
            Opcode::AdvanceIterator,
            self.encode_register(iterator_register)?,
            self.encode_register(element_value)?,
            self.encode_register(done_register)?,
        )?;
        let exit_jump = self
            .builder
            .emit_cond_jump_placeholder(Opcode::JumpIfTrue, self.encode_register(done_register)?)?;
        self.emit_set_keyed_property(rest_value, element_value, rest_index)?;

        let next_rest = self.alloc_temp()?;
        self.emit_profiled_binary(Opcode::Add, next_rest, rest_index, one_register)?;
        self.emit_move(rest_index, next_rest)?;

        let jump_back = self.builder.emit_jump_placeholder(Opcode::Jump)?;
        self.builder.patch_jump_to(jump_back, loop_start)?;
        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(exit_jump, end)?;
        self.emit_set_property_by_atom(rest_value, rest_index, WellKnownAtom::length.id())?;

        if let Some(target) = prepared_target {
            self.assign_prepared_reference(target, rest_value)
        } else {
            self.lower_destructuring_assignment_from_register(target_expr, rest_value)
        }
    }

    fn assignment_rest_target_is_pattern(&self, expr_id: ExprId) -> bool {
        match self.ast().get_expr(expr_id) {
            Expr::ParenthesizedExpression { expression, .. } => {
                self.assignment_rest_target_is_pattern(*expression)
            }
            Expr::ArrayExpression { .. } | Expr::ObjectExpression { .. } => true,
            _ => false,
        }
    }

    fn lower_object_rest_assignment_from_register(
        &mut self,
        target_expr: ExprId,
        source_register: u16,
        excluded_keys: &[ObjectRestExcludedKey],
    ) -> LoweringResult<()> {
        let rest_value = self.create_object_rest_copy_from_register(
            source_register,
            excluded_keys,
            self.ast().get_expr(target_expr).span(),
        )?;

        self.lower_destructuring_assignment_from_register(target_expr, rest_value)
    }

    pub(super) fn create_object_rest_copy_from_register(
        &mut self,
        source_register: u16,
        excluded_keys: &[ObjectRestExcludedKey],
        span: Span,
    ) -> LoweringResult<u16> {
        let rest_value = self.alloc_temp()?;
        let instruction_offset =
            self.builder
                .emit_abx(Opcode::CreateObject, self.encode_register(rest_value)?, 0)?;
        self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation)?;
        let excluded_keys_register = if excluded_keys.is_empty() {
            let excluded = self.alloc_temp()?;
            self.emit_load_undefined(excluded)?;
            excluded
        } else {
            let excluded = self.alloc_temp()?;
            let instruction_offset =
                self.builder
                    .emit_abx(Opcode::CreateArray, self.encode_register(excluded)?, 0)?;
            self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation)?;
            for (index, excluded_key) in excluded_keys.iter().enumerate() {
                let excluded_value = match excluded_key {
                    ObjectRestExcludedKey::Atom(atom) => {
                        let value = self.alloc_temp()?;
                        self.emit_load_atom_string(value, *atom)?;
                        value
                    }
                    ObjectRestExcludedKey::Register(register) => *register,
                };
                let index_register = self.alloc_temp()?;
                let index_value = i32::try_from(index)
                    .map_err(|_| LoweringError::ConstantIndexOverflow { index: u32::MAX })?;
                if let Ok(index_smi) = i16::try_from(index_value) {
                    self.emit_load_smi(index_register, index_smi)?;
                } else {
                    let constant = self.constant_smi(index_value)?;
                    self.builder.emit_abx(
                        Opcode::LoadConst,
                        self.encode_register(index_register)?,
                        constant,
                    )?;
                }
                self.emit_define_keyed_property(excluded, excluded_value, index_register)?;
            }
            excluded
        };
        self.emit_copy_data_properties(rest_value, source_register, excluded_keys_register)?;
        Ok(rest_value)
    }

    fn assignment_target_name(&self, expr_id: ExprId) -> Option<AtomId> {
        match self.ast().get_expr(expr_id) {
            Expr::Identifier { name, .. } => Some(*name),
            Expr::ParenthesizedExpression { expression, .. } => {
                self.assignment_target_name(*expression)
            }
            Expr::AssignmentExpression {
                operator: AssignOp::Assign,
                left,
                ..
            } => self.assignment_target_name(*left),
            _ => None,
        }
    }

    pub(super) fn store_binding_value(
        &mut self,
        binding_id: SemanticBindingId,
        name: AtomId,
        value_register: u16,
    ) -> LoweringResult<()> {
        let binding = self.binding(binding_id)?;
        if binding.storage_class == StorageClass::DynamicLookup {
            return self.emit_assign_name(value_register, name);
        }
        if binding.storage_class == StorageClass::DynamicVariableLookup {
            return self.emit_assign_variable_name(value_register, name);
        }
        if let Some((depth, slot)) = self.binding_env_access(binding_id)? {
            return self.emit_store_env_slot(value_register, depth, slot);
        }
        match binding.storage_class {
            StorageClass::FrameLocal => {
                let register = self.ensure_local_register(binding_id)?;
                self.emit_move(register, value_register)
            }
            StorageClass::GlobalName => self.emit_store_global(value_register, binding.name),
            StorageClass::EnvironmentSlot => unreachable!("env-backed bindings handled above"),
            StorageClass::DynamicLookup | StorageClass::DynamicVariableLookup => {
                unreachable!("dynamic lookup bindings must lower through name ops")
            }
        }
    }

    pub(super) fn load_binding_value(
        &mut self,
        binding_id: SemanticBindingId,
        name: AtomId,
        dest: u16,
    ) -> LoweringResult<()> {
        let binding = self.binding(binding_id)?;
        let has_tdz = binding.has_tdz;
        if matches!(
            binding.storage_class,
            StorageClass::DynamicLookup | StorageClass::DynamicVariableLookup
        ) {
            return self.emit_load_name(dest, name);
        }
        if let Some((depth, slot)) = self.binding_env_access(binding_id)? {
            return self.emit_load_env_slot(dest, depth, slot);
        }
        match binding.storage_class {
            StorageClass::FrameLocal => {
                let register = self.ensure_local_register(binding_id)?;
                self.emit_move(dest, register)?;
                if has_tdz {
                    self.emit_throw_if_uninitialized(dest)?;
                }
                Ok(())
            }
            StorageClass::GlobalName => self.emit_load_global(dest, binding.name),
            StorageClass::EnvironmentSlot => unreachable!("env-backed bindings handled above"),
            StorageClass::DynamicLookup | StorageClass::DynamicVariableLookup => {
                unreachable!("dynamic lookup bindings must lower through LoadName")
            }
        }
    }

    pub(super) fn assign_binding_value(
        &mut self,
        binding_id: SemanticBindingId,
        name: AtomId,
        value_register: u16,
    ) -> LoweringResult<()> {
        let binding = self.binding(binding_id)?;
        let has_tdz = binding.has_tdz;
        if binding.storage_class == StorageClass::DynamicLookup {
            return self.emit_assign_name(value_register, name);
        }
        if binding.storage_class == StorageClass::DynamicVariableLookup {
            return self.emit_assign_variable_name(value_register, name);
        }
        if let Some((depth, slot)) = self.binding_env_access(binding_id)? {
            return self.emit_assign_env_slot(value_register, depth, slot);
        }
        match binding.storage_class {
            StorageClass::FrameLocal => {
                let register = self.ensure_local_register(binding_id)?;
                if has_tdz {
                    self.emit_throw_if_uninitialized(register)?;
                }
                self.emit_move(register, value_register)
            }
            StorageClass::GlobalName => self.emit_assign_global(value_register, binding.name),
            StorageClass::EnvironmentSlot => unreachable!("env-backed bindings handled above"),
            StorageClass::DynamicLookup | StorageClass::DynamicVariableLookup => {
                unreachable!("dynamic lookup bindings must lower through name ops")
            }
        }
    }

    pub(super) fn lower_declaration(&mut self, decl_id: DeclId) -> LoweringResult<()> {
        let decl = self.ast().get_decl(decl_id).clone();
        match decl {
            Decl::Variable {
                kind, declarators, ..
            } => {
                for declarator in self.ast().get_var_declarator_list(declarators).to_vec() {
                    self.lower_variable_declarator(kind, declarator)?;
                }
                Ok(())
            }
            Decl::Function { function, .. } => {
                if self.hoisted_function_decls.contains(&decl_id) {
                    Ok(())
                } else if self.block_instantiated_function_decls.contains(&decl_id) {
                    self.lower_block_instantiated_function_declaration(function)
                } else {
                    self.lower_function_declaration(decl_id, function)
                }
            }
            Decl::Class {
                name,
                super_class,
                body,
                ..
            } => self.lower_class_declaration(decl_id, name, super_class, body),
            Decl::Import { .. } => Ok(()),
            Decl::Export { kind, .. } => self.lower_export_declaration(kind),
            Decl::InvalidDeclaration { .. } => {
                Err(LoweringError::UnsupportedDeclaration { decl: decl_id })
            }
        }
    }

    fn lower_export_declaration(&mut self, kind: lyng_js_ast::ExportKind) -> LoweringResult<()> {
        match kind {
            lyng_js_ast::ExportKind::Declaration { decl } => self.lower_declaration(decl),
            lyng_js_ast::ExportKind::Default { declaration } => match declaration {
                lyng_js_ast::ExportDefaultDecl::Function(function)
                    if self.hoisted_default_export_functions.contains(&function) =>
                {
                    Ok(())
                }
                _ => self.lower_default_export_declaration(declaration),
            },
            lyng_js_ast::ExportKind::Named { .. } | lyng_js_ast::ExportKind::All { .. } => Ok(()),
        }
    }

    pub(super) fn lower_default_export_declaration(
        &mut self,
        declaration: lyng_js_ast::ExportDefaultDecl,
    ) -> LoweringResult<()> {
        let default_name = WellKnownAtom::default.id();
        let slot = self.state.module_default_export_slot().ok_or(
            LoweringError::UnsupportedDeclaration {
                decl: DeclId::new(0),
            },
        )?;
        let value_register = self.alloc_temp()?;
        match declaration {
            lyng_js_ast::ExportDefaultDecl::Function(function) => {
                let function_expr = self.ast().get_function(function).clone();
                let child_index = self.ensure_child_index(function)?;
                self.emit_create_closure(value_register, child_index, function_expr.span)?;
                if function_expr.name.is_none() {
                    let name_value = self.alloc_temp()?;
                    self.emit_load_atom_string(name_value, WellKnownAtom::default.id())?;
                    self.emit_set_function_name(value_register, name_value)?;
                } else if let Some(name) = function_expr.name {
                    let binding_id = self.find_named_binding(name, DeclarationKind::Function)?;
                    self.store_binding_value(binding_id, name, value_register)?;
                }
            }
            lyng_js_ast::ExportDefaultDecl::Class(decl_id) => {
                let Decl::Class {
                    name,
                    super_class,
                    body,
                    ..
                } = self.ast().get_decl(decl_id).clone()
                else {
                    return Err(LoweringError::UnsupportedDeclaration { decl: decl_id });
                };
                let class_span = self.ast().get_decl(decl_id).span();
                self.lower_class_definition(
                    name.or(Some(default_name)),
                    super_class,
                    body,
                    class_span,
                    value_register,
                )?;
                if let Some(name) = name {
                    let binding_id = self.class_declaration_binding(name)?;
                    self.store_binding_value(binding_id, name, value_register)?;
                }
            }
            lyng_js_ast::ExportDefaultDecl::Expression(expr) => {
                self.lower_initializer_with_inferred_name(
                    expr,
                    Some(default_name),
                    value_register,
                )?;
            }
        }
        self.emit_store_env_slot(value_register, 0, slot)
    }

    pub(super) fn emit_block_function_declaration_instantiations(
        &mut self,
        list: lyng_js_ast::NodeList<StmtId>,
    ) -> LoweringResult<()> {
        let stmts = self.ast().get_stmt_list(list).to_vec();
        for stmt in stmts {
            if let Some((decl, function)) = self.block_function_declaration_from_statement(stmt) {
                self.instantiate_block_function_declaration(decl, function)?;
            }
        }
        Ok(())
    }

    fn block_function_declaration_from_statement(
        &self,
        stmt_id: StmtId,
    ) -> Option<(DeclId, FunctionId)> {
        match self.ast().get_stmt(stmt_id) {
            Stmt::Declaration { decl, .. } => match self.ast().get_decl(*decl) {
                Decl::Function { function, .. } => Some((*decl, *function)),
                _ => None,
            },
            Stmt::Labeled { body, .. } => self.block_function_declaration_from_statement(*body),
            _ => None,
        }
    }

    fn instantiate_block_function_declaration(
        &mut self,
        decl_id: DeclId,
        function: FunctionId,
    ) -> LoweringResult<()> {
        let ast_function = self.ast().get_function(function).clone();
        let name = ast_function
            .name
            .ok_or(LoweringError::UnsupportedFunction { function })?;
        let binding_id = self.function_declaration_binding(name)?;
        let child_index = self.ensure_child_index(function)?;
        let value_register = self.alloc_temp()?;

        self.builder.emit_abx(
            Opcode::CreateClosure,
            self.encode_register(value_register)?,
            child_index,
        )?;
        self.store_binding_value(binding_id, name, value_register)?;
        self.block_instantiated_function_decls.insert(decl_id);
        Ok(())
    }

    pub(super) fn lower_function_declaration(
        &mut self,
        _decl_id: DeclId,
        function: FunctionId,
    ) -> LoweringResult<()> {
        let ast_function = self.ast().get_function(function).clone();
        let name = ast_function
            .name
            .ok_or(LoweringError::UnsupportedFunction { function })?;
        let binding_id = self.function_declaration_binding(name)?;
        let child_index = self.ensure_child_index(function)?;
        let value_register = self.alloc_temp()?;

        self.builder.emit_abx(
            Opcode::CreateClosure,
            self.encode_register(value_register)?,
            child_index,
        )?;
        self.store_binding_value(binding_id, name, value_register)?;

        if self.function_declaration_uses_annex_b_var_update(&ast_function) {
            if let Some(var_binding) = self.annex_b_var_binding_for_block_function(name, binding_id)
            {
                self.assign_binding_value(var_binding, name, value_register)?;
            }
        }

        Ok(())
    }

    fn lower_block_instantiated_function_declaration(
        &mut self,
        function: FunctionId,
    ) -> LoweringResult<()> {
        let ast_function = self.ast().get_function(function).clone();
        let name = ast_function
            .name
            .ok_or(LoweringError::UnsupportedFunction { function })?;
        if !self.function_declaration_uses_annex_b_var_update(&ast_function) {
            return Ok(());
        }
        let binding_id = self.function_declaration_binding(name)?;
        if let Some(var_binding) = self.annex_b_var_binding_for_block_function(name, binding_id) {
            let value_register = self.alloc_temp()?;
            self.load_binding_value(binding_id, name, value_register)?;
            self.assign_binding_value(var_binding, name, value_register)?;
        }
        Ok(())
    }

    fn function_declaration_binding(&self, name: AtomId) -> LoweringResult<SemanticBindingId> {
        let scope_kind = self.state.sema.scope_table.get(self.current_scope).kind;
        if matches!(scope_kind, ScopeKind::Block | ScopeKind::Switch) {
            if let Some(binding) = self.find_named_binding_in_scope(
                name,
                DeclarationKind::Function,
                self.current_scope,
            ) {
                return Ok(binding);
            }
        }

        self.find_named_binding_in_scope(name, DeclarationKind::Function, self.nearest_var_scope())
            .or_else(|| {
                self.find_named_binding_in_scope(
                    name,
                    DeclarationKind::Var,
                    self.nearest_var_scope(),
                )
            })
            .ok_or(LoweringError::MissingDeclarationBinding { name })
    }

    fn function_declaration_uses_annex_b_var_update(
        &self,
        ast_function: &lyng_js_ast::Function,
    ) -> bool {
        if ast_function.kind != FunctionKind::Normal {
            return false;
        }
        let scope_kind = self.state.sema.scope_table.get(self.current_scope).kind;
        if !matches!(scope_kind, ScopeKind::Block | ScopeKind::Switch) {
            return false;
        }
        if let Some(function) = self.current_function {
            !self.state.sema.function_table.get(function).strict
        } else {
            !self.state.program.strict
        }
    }

    fn annex_b_var_binding_for_block_function(
        &self,
        name: AtomId,
        lexical_binding: SemanticBindingId,
    ) -> Option<SemanticBindingId> {
        if self.annex_b_var_update_would_conflict(name, lexical_binding) {
            return None;
        }
        let var_scope = self.nearest_var_scope();
        let binding = self
            .find_named_binding_in_scope(name, DeclarationKind::Function, var_scope)
            .or_else(|| self.find_named_binding_in_scope(name, DeclarationKind::Var, var_scope))?;
        if self.binding(binding).ok()?.kind == DeclarationKind::Parameter {
            return None;
        }
        (binding != lexical_binding).then_some(binding)
    }

    fn annex_b_var_update_would_conflict(
        &self,
        name: AtomId,
        lexical_binding: SemanticBindingId,
    ) -> bool {
        let var_scope = self.nearest_var_scope();
        let mut scope_id = self.current_scope;
        loop {
            if let Some(binding) = self.find_any_named_binding_in_scope(name, scope_id) {
                if binding != lexical_binding && self.annex_b_binding_is_lexical(binding, scope_id)
                {
                    return true;
                }
            }
            if scope_id == var_scope {
                return false;
            }
            let Some(parent) = self.state.sema.scope_table.get(scope_id).parent else {
                return false;
            };
            scope_id = parent;
        }
    }

    fn find_any_named_binding_in_scope(
        &self,
        name: AtomId,
        scope: ScopeId,
    ) -> Option<SemanticBindingId> {
        self.state
            .sema
            .binding_table
            .as_slice()
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, binding)| {
                (binding.name == name && binding.scope == scope)
                    .then_some(SemanticBindingId::new(index as u32))
            })
    }

    fn annex_b_binding_is_lexical(&self, binding: SemanticBindingId, scope: ScopeId) -> bool {
        let Some(binding) = self
            .state
            .sema
            .binding_table
            .as_slice()
            .get(binding.raw() as usize)
        else {
            return false;
        };
        binding.kind.is_lexical()
            || (binding.kind == DeclarationKind::Function
                && !matches!(
                    self.state.sema.scope_table.get(scope).kind,
                    ScopeKind::Global | ScopeKind::Function
                ))
    }

    pub(super) fn lower_variable_declarator(
        &mut self,
        kind: VariableKind,
        declarator: lyng_js_ast::VariableDeclarator,
    ) -> LoweringResult<()> {
        let declaration_kind = match kind {
            VariableKind::Var => DeclarationKind::Var,
            VariableKind::Let => DeclarationKind::Let,
            VariableKind::Const => DeclarationKind::Const,
            VariableKind::Using => DeclarationKind::Using,
            VariableKind::AwaitUsing => DeclarationKind::AwaitUsing,
        };
        let pattern = self.ast().get_pattern(declarator.id).clone();
        let Pattern::Identifier { name, .. } = pattern else {
            if matches!(kind, VariableKind::Using | VariableKind::AwaitUsing) {
                return Err(LoweringError::UnsupportedPattern {
                    pattern: declarator.id,
                });
            }
            if declarator.init.is_none() && kind == VariableKind::Var {
                return Ok(());
            }
            let value_register = self.alloc_temp()?;
            if let Some(init) = declarator.init {
                self.lower_expr_into(init, value_register)?;
            } else {
                self.emit_load_undefined(value_register)?;
            }
            return self.lower_binding_pattern_initialization(
                declarator.id,
                declaration_kind,
                value_register,
            );
        };
        if kind == VariableKind::Var {
            if let Some(catch_binding) = self.active_simple_catch_binding_for_name(name) {
                if let Some(init) = declarator.init {
                    let value_register = self.alloc_temp()?;
                    self.lower_initializer_with_inferred_name(init, Some(name), value_register)?;
                    self.assign_binding_value(catch_binding, name, value_register)?;
                }
                return Ok(());
            }
            if self.in_with_scope() {
                if let Some(init) = declarator.init {
                    let reference = self.alloc_temp()?;
                    self.emit_capture_name(reference, name)?;
                    let value_register = self.alloc_temp()?;
                    self.lower_initializer_with_inferred_name(init, Some(name), value_register)?;
                    self.emit_assign_captured_name(value_register, reference)?;
                }
                return Ok(());
            }
        }

        let binding_id = self.declared_binding_for_pattern(declarator.id, declaration_kind)?;
        let (storage_class, binding_name) = {
            let binding = self.binding(binding_id)?;
            (binding.storage_class, binding.name)
        };
        let initialize_var_binding = |this: &mut Self, value_register: u16| {
            if kind == VariableKind::Var {
                if this.in_with_scope() {
                    this.emit_assign_name(value_register, binding_name)
                } else {
                    this.assign_binding_value(binding_id, binding_name, value_register)
                }
            } else {
                this.store_binding_value(binding_id, binding_name, value_register)
            }
        };

        match storage_class {
            StorageClass::FrameLocal => {
                let register = self.ensure_local_register(binding_id)?;
                if let Some(init) = declarator.init {
                    self.lower_initializer_with_inferred_name(init, Some(binding_name), register)?;
                    if kind == VariableKind::Var {
                        initialize_var_binding(self, register)?;
                    }
                } else if kind != VariableKind::Var {
                    self.emit_load_undefined(register)?;
                }
                if matches!(kind, VariableKind::Using | VariableKind::AwaitUsing) {
                    self.lower_disposable_resource_registration(kind, register, declarator.span)?;
                }
            }
            StorageClass::GlobalName => {
                if declarator.init.is_none() && kind == VariableKind::Var {
                    return Ok(());
                }
                let value_register = self.alloc_temp()?;
                if let Some(init) = declarator.init {
                    self.lower_initializer_with_inferred_name(
                        init,
                        Some(binding_name),
                        value_register,
                    )?;
                } else {
                    self.emit_load_undefined(value_register)?;
                }
                initialize_var_binding(self, value_register)?;
                if matches!(kind, VariableKind::Using | VariableKind::AwaitUsing) {
                    self.lower_disposable_resource_registration(
                        kind,
                        value_register,
                        declarator.span,
                    )?;
                }
            }
            StorageClass::EnvironmentSlot => {
                if declarator.init.is_none() && kind == VariableKind::Var {
                    return Ok(());
                }
                let value_register = self.alloc_temp()?;
                if let Some(init) = declarator.init {
                    self.lower_initializer_with_inferred_name(
                        init,
                        Some(binding_name),
                        value_register,
                    )?;
                } else {
                    self.emit_load_undefined(value_register)?;
                }
                if kind == VariableKind::Var {
                    initialize_var_binding(self, value_register)?;
                } else {
                    let (depth, slot) = self.binding_env_access(binding_id)?.ok_or(
                        LoweringError::MissingEnvironmentSlot {
                            binding: binding_id,
                        },
                    )?;
                    self.emit_store_env_slot(value_register, depth, slot)?;
                }
                if matches!(kind, VariableKind::Using | VariableKind::AwaitUsing) {
                    self.lower_disposable_resource_registration(
                        kind,
                        value_register,
                        declarator.span,
                    )?;
                }
            }
            StorageClass::DynamicLookup | StorageClass::DynamicVariableLookup => {
                if declarator.init.is_none() && kind == VariableKind::Var {
                    return Ok(());
                }
                let value_register = self.alloc_temp()?;
                if let Some(init) = declarator.init {
                    self.lower_initializer_with_inferred_name(
                        init,
                        Some(binding_name),
                        value_register,
                    )?;
                } else {
                    self.emit_load_undefined(value_register)?;
                }
                self.assign_binding_value(binding_id, binding_name, value_register)?;
                if matches!(kind, VariableKind::Using | VariableKind::AwaitUsing) {
                    self.lower_disposable_resource_registration(
                        kind,
                        value_register,
                        declarator.span,
                    )?;
                }
            }
        }

        Ok(())
    }

    pub(super) fn lower_disposable_resource_registration(
        &mut self,
        kind: VariableKind,
        value_register: u16,
        span: Span,
    ) -> LoweringResult<()> {
        let scope = self.active_disposal_scope_for_kind(kind).ok_or(
            LoweringError::UnsupportedDeclaration {
                decl: DeclId::new(0),
            },
        )?;
        let builtin = match kind {
            VariableKind::Using => add_sync_disposable_resource_builtin(),
            VariableKind::AwaitUsing => add_async_disposable_resource_builtin(),
            _ => return Ok(()),
        };
        self.emit_internal_builtin_call(builtin, &[scope.capability_register, value_register], span)
    }
}

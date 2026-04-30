use super::*;

impl<'a, 'b> FunctionCompiler<'a, 'b> {
    pub(super) fn assignment_target(&self, expr_id: ExprId) -> ExprId {
        let mut current = expr_id;
        while let Expr::ParenthesizedExpression { expression, .. } = self.ast().get_expr(current) {
            current = *expression;
        }
        current
    }

    pub(super) fn lower_annex_b_call_assignment_target_reference_error(
        &mut self,
        expr_id: ExprId,
    ) -> LoweringResult<bool> {
        let target = self.assignment_target(expr_id);
        if !matches!(self.ast().get_expr(target), Expr::CallExpression { .. }) {
            return Ok(false);
        }

        let value = self.alloc_temp()?;
        self.lower_expr_into(target, value)?;
        self.emit_throw_reference_error(self.ast().get_expr(target).span())?;
        Ok(true)
    }

    pub(super) fn named_function_expression_self_binding(
        &self,
        function: FunctionId,
        name: AtomId,
    ) -> LoweringResult<Option<SemanticBindingId>> {
        let sema_id = self
            .state
            .ast_to_sema
            .get(&function)
            .copied()
            .ok_or(LoweringError::MissingFunctionRecord { function })?;
        Ok(self
            .state
            .sema
            .binding_table
            .as_slice()
            .iter()
            .enumerate()
            .find_map(|(index, binding)| {
                (binding.kind == DeclarationKind::FunctionName
                    && binding.name == name
                    && self.binding_belongs_to_owner(binding.scope, Some(sema_id)))
                .then_some(SemanticBindingId::new(index as u32))
            }))
    }

    pub(super) fn lower_identifier(
        &mut self,
        expr_id: ExprId,
        name: AtomId,
        dest: u16,
    ) -> LoweringResult<()> {
        let use_site = self.use_site(expr_id)?;
        let resolution_kind = use_site.resolution_kind;
        let resolved_binding = use_site.resolved_binding;
        if resolved_binding.is_none() {
            if let Some((depth, slot)) = self.arguments_access(name)? {
                return self.emit_load_env_slot(dest, depth, slot);
            }
        }
        match resolution_kind {
            ResolutionKind::Local | ResolutionKind::Captured => {
                let binding_id = resolved_binding.ok_or(LoweringError::MissingResolvedBinding {
                    expr: expr_id,
                    name,
                })?;
                let (storage_class, binding_name) = {
                    let binding = self.binding(binding_id)?;
                    (binding.storage_class, binding.name)
                };
                if storage_class == StorageClass::DynamicLookup {
                    return self.emit_load_name(dest, binding_name);
                }
                if let Some((depth, slot)) = self.binding_env_access(binding_id)? {
                    self.emit_load_env_slot(dest, depth, slot)
                } else {
                    match storage_class {
                        StorageClass::FrameLocal => {
                            let register = self.ensure_local_register(binding_id)?;
                            self.emit_move(dest, register)
                        }
                        StorageClass::EnvironmentSlot => {
                            unreachable!("env-backed bindings handled above")
                        }
                        StorageClass::GlobalName => self.emit_load_global(dest, binding_name),
                        StorageClass::DynamicLookup => {
                            unreachable!("dynamic lookup bindings must lower through LoadName")
                        }
                    }
                }
            }
            ResolutionKind::Global | ResolutionKind::Unresolved => {
                self.emit_load_global(dest, name)
            }
            ResolutionKind::Dynamic => self.emit_load_name(dest, name),
        }
    }

    pub(super) fn lower_assignment_expression(
        &mut self,
        operator: AssignOp,
        left: ExprId,
        right: ExprId,
        dest: u16,
    ) -> LoweringResult<()> {
        let left = self.assignment_target(left);
        if self.lower_annex_b_call_assignment_target_reference_error(left)? {
            return Ok(());
        }

        match self.ast().get_expr(left).clone() {
            Expr::ArrayExpression { .. } | Expr::ObjectExpression { .. }
                if operator == AssignOp::Assign =>
            {
                let value = self.lower_expr_to_temp(right)?;
                self.lower_destructuring_assignment_from_register(left, value)?;
                self.emit_move(dest, value)
            }
            _ => {
                let usage = if operator == AssignOp::Assign {
                    ReferenceUsage::WriteOnly
                } else {
                    ReferenceUsage::ReadWrite
                };
                let Some(target) = self.prepare_reference_target(left, usage)? else {
                    return Err(LoweringError::UnsupportedExpression { expr: left });
                };
                match operator {
                    AssignOp::Assign => {
                        let value = self.alloc_temp()?;
                        if let Some(name) = self.reference_target_inferred_name(target) {
                            self.lower_initializer_with_inferred_name(right, Some(name), value)?;
                        } else {
                            self.lower_expr_into(right, value)?;
                        }
                        self.assign_prepared_reference(target, value)?;
                        self.emit_move(dest, value)
                    }
                    AssignOp::AndAssign | AssignOp::OrAssign | AssignOp::NullishAssign => {
                        let current = self.alloc_temp()?;
                        self.load_prepared_reference(target, current)?;
                        self.emit_move(dest, current)?;
                        let jump_end =
                            self.emit_logical_assignment_short_circuit(operator, current)?;
                        let value = self.alloc_temp()?;
                        if let Some(name) = self.reference_target_inferred_name(target) {
                            self.lower_initializer_with_inferred_name(right, Some(name), value)?;
                        } else {
                            self.lower_expr_into(right, value)?;
                        }
                        self.assign_prepared_reference(target, value)?;
                        self.emit_move(dest, value)?;
                        let end = self.builder.current_offset()?;
                        self.builder.patch_jump_to(jump_end, end)?;
                        Ok(())
                    }
                    operator => {
                        let current = self.alloc_temp()?;
                        self.load_prepared_reference(target, current)?;
                        let rhs = self.lower_expr_to_temp(right)?;
                        let result = self.alloc_temp()?;
                        self.builder.emit_abc(
                            self.assignment_opcode(operator)?,
                            self.encode_register(result)?,
                            self.encode_register(current)?,
                            self.encode_register(rhs)?,
                        )?;
                        self.assign_prepared_reference(target, result)?;
                        self.emit_move(dest, result)
                    }
                }
            }
        }
    }
}

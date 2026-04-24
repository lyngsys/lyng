use super::*;
use lyng_js_types::js3_internal_import_meta_builtin;

impl<'a, 'b> FunctionCompiler<'a, 'b> {
    pub(super) fn lower_update_expression(
        &mut self,
        expr_id: ExprId,
        operator: lyng_js_ast::UpdateOp,
        argument: ExprId,
        prefix: bool,
        dest: u16,
    ) -> LoweringResult<()> {
        let target = self.delete_target(argument);
        let Some(target) = self.prepare_reference_target(target, ReferenceUsage::ReadWrite)? else {
            return Err(LoweringError::UnsupportedExpression { expr: expr_id });
        };
        let current = self.alloc_temp()?;
        self.load_prepared_reference(target, current)?;
        let result = self.lower_updated_value(current, operator)?;
        self.store_prepared_reference(target, result)?;
        self.emit_move(dest, if prefix { result } else { current })
    }

    fn lower_updated_value(
        &mut self,
        current: u16,
        operator: lyng_js_ast::UpdateOp,
    ) -> LoweringResult<u16> {
        let result = self.alloc_temp()?;
        let opcode = match operator {
            lyng_js_ast::UpdateOp::Increment => Opcode::Increment,
            lyng_js_ast::UpdateOp::Decrement => Opcode::Decrement,
        };
        self.emit_profiled_update(opcode, result, current)?;
        Ok(result)
    }

    pub(super) fn lower_static_member_get(
        &mut self,
        object: ExprId,
        property: AtomId,
        dest: u16,
    ) -> LoweringResult<()> {
        if self.expr_continues_optional_chain(object) {
            return self.lower_optional_chain_static_member_continuation(object, property, dest);
        }
        if matches!(self.ast().get_expr(object), Expr::Super { .. }) {
            let receiver = self.lower_super_receiver()?;
            let key = self.alloc_temp()?;
            self.emit_load_atom_string(key, property)?;
            return self.emit_super_property_get(
                receiver,
                key,
                self.ast().get_expr(object).span(),
                dest,
            );
        }
        let object_register = self.lower_expr_to_temp(object)?;
        self.emit_get_property_by_atom(dest, object_register, property)
    }

    pub(super) fn lower_computed_member_get(
        &mut self,
        object: ExprId,
        property: ExprId,
        dest: u16,
    ) -> LoweringResult<()> {
        if self.expr_continues_optional_chain(object) {
            return self.lower_optional_chain_computed_member_continuation(object, property, dest);
        }
        if matches!(self.ast().get_expr(object), Expr::Super { .. }) {
            let receiver = self.lower_super_receiver()?;
            let key = self.lower_expr_to_temp(property)?;
            return self.emit_super_property_get(
                receiver,
                key,
                self.ast().get_expr(object).span(),
                dest,
            );
        }
        let object_register = self.lower_expr_to_temp(object)?;
        let key_register = self.lower_expr_to_temp(property)?;
        self.emit_get_keyed_property(dest, object_register, key_register)
    }

    pub(super) fn lower_object_expression(
        &mut self,
        expr_id: ExprId,
        properties: lyng_js_ast::NodeList<Property>,
        dest: u16,
    ) -> LoweringResult<()> {
        let properties = self.ast().get_property_list(properties).to_vec();
        let instruction_offset = self.builder.emit_abx(
            Opcode::CreateObject,
            self.encode_register(dest)?,
            u32::try_from(properties.len()).unwrap_or(u32::MAX),
        );
        let span = self.ast().get_expr(expr_id).span();
        self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation);
        for property in properties {
            self.lower_object_property(dest, property)?;
        }
        Ok(())
    }

    pub(super) fn lower_object_property(
        &mut self,
        object: u16,
        property: Property,
    ) -> LoweringResult<()> {
        if let Expr::SpreadElement { argument, .. } = self.ast().get_expr(property.value).clone() {
            let source_register = self.lower_expr_to_temp(argument)?;
            let excluded_keys = self.alloc_temp()?;
            self.emit_load_undefined(excluded_keys)?;
            self.emit_copy_data_properties(object, source_register, excluded_keys)?;
            return Ok(());
        }

        match property.kind {
            PropertyKind::Init => {
                let named_atom = if property.computed {
                    None
                } else {
                    self.named_property_atom(property.key)?
                };
                let key_register = if named_atom.is_none() {
                    let raw_key = self.lower_expr_to_temp(property.key)?;
                    let property_key = self.alloc_temp()?;
                    self.emit_to_property_key(property_key, raw_key)?;
                    Some(property_key)
                } else {
                    None
                };
                let value_register = self.lower_expr_to_temp(property.value)?;
                if !property.computed
                    && !property.method
                    && !property.shorthand
                    && named_atom == Some(WellKnownAtom::__proto__.id())
                {
                    return self.emit_internal_builtin_call(
                        js3_internal_object_literal_set_prototype_builtin(),
                        &[object, value_register],
                        property.span,
                    );
                }
                if property.method {
                    self.bind_function_home_object(value_register, object, property.span)?;
                }
                if let Some(atom) = named_atom {
                    self.emit_define_property_by_atom(object, value_register, atom)?;
                    return Ok(());
                }

                self.emit_define_keyed_property(
                    object,
                    value_register,
                    key_register.expect("non-atom property keys should be lowered"),
                )
            }
            PropertyKind::Get | PropertyKind::Set => {
                let key = self.lower_object_property_key_value(property)?;
                let accessor = self.lower_expr_to_temp(property.value)?;
                self.lower_object_accessor_property(
                    object,
                    key,
                    accessor,
                    property.kind == PropertyKind::Get,
                    property.span,
                )
            }
        }
    }

    fn lower_object_property_key_value(&mut self, property: Property) -> LoweringResult<u16> {
        if !property.computed {
            if let Some(atom) = self.named_property_atom(property.key)? {
                let key = self.alloc_temp()?;
                self.emit_load_atom_string(key, atom)?;
                return Ok(key);
            }
        }
        let raw_key = self.lower_expr_to_temp(property.key)?;
        let key = self.alloc_temp()?;
        self.emit_to_property_key(key, raw_key)?;
        Ok(key)
    }

    fn lower_object_accessor_property(
        &mut self,
        object: u16,
        key: u16,
        accessor: u16,
        is_getter: bool,
        span: Span,
    ) -> LoweringResult<()> {
        self.bind_function_home_object(accessor, object, span)?;
        let callee = self.alloc_temp()?;
        self.emit_load_builtin(
            callee,
            if is_getter {
                js3_internal_define_getter_property_builtin()
            } else {
                js3_internal_define_setter_property_builtin()
            },
        )?;
        let this_value = self.alloc_temp()?;
        self.emit_load_undefined(this_value)?;
        let arguments = [object, key, accessor];
        let argument_range = self.materialize_argument_block(&arguments)?;
        let result = self.alloc_temp()?;
        let (call_result, call_callee, call_this, move_back) =
            self.bridge_call_registers(result, callee, this_value)?;
        let instruction_offset = self.builder.emit_call(
            self.encode_register(call_result)?,
            self.encode_register(call_callee)?,
            self.encode_register(call_this)?,
            argument_range,
        );
        self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation);
        if let Some(dest) = move_back {
            self.emit_move(dest, call_result)?;
        }
        Ok(())
    }

    pub(super) fn lower_meta_property(
        &mut self,
        meta: AtomId,
        property: AtomId,
        dest: u16,
        expr_id: ExprId,
    ) -> LoweringResult<()> {
        if meta == WellKnownAtom::new.id() && property == WellKnownAtom::target.id() {
            return self.emit_load_new_target(dest);
        }
        if meta == WellKnownAtom::import.id() && property == WellKnownAtom::meta.id() {
            return self.emit_internal_builtin_call_into(
                js3_internal_import_meta_builtin(),
                &[],
                self.ast().get_expr(expr_id).span(),
                dest,
            );
        }
        Err(LoweringError::UnsupportedExpression { expr: expr_id })
    }

    pub(super) fn lower_super_receiver(&mut self) -> LoweringResult<u16> {
        let receiver = self.alloc_temp()?;
        if let Some(this_override) = self.this_override_register {
            self.emit_move(receiver, this_override)?;
        } else {
            self.emit_load_this(receiver)?;
        }
        Ok(receiver)
    }

    pub(super) fn emit_super_property_get(
        &mut self,
        receiver: u16,
        key: u16,
        span: Span,
        dest: u16,
    ) -> LoweringResult<()> {
        let arguments = if let Some(home_object) = self.super_home_object_override {
            vec![receiver, key, home_object]
        } else {
            vec![receiver, key]
        };
        self.emit_internal_builtin_call_into(
            js3_internal_super_property_get_builtin(),
            &arguments,
            span,
            dest,
        )
    }

    pub(super) fn emit_super_property_set(
        &mut self,
        receiver: u16,
        key: u16,
        value: u16,
        span: Span,
        dest: u16,
    ) -> LoweringResult<()> {
        let arguments = if let Some(home_object) = self.super_home_object_override {
            vec![receiver, key, value, home_object]
        } else {
            vec![receiver, key, value]
        };
        self.emit_internal_builtin_call_into(
            js3_internal_super_property_set_builtin(),
            &arguments,
            span,
            dest,
        )
    }

    pub(super) fn lower_delete_expression(
        &mut self,
        argument: ExprId,
        dest: u16,
    ) -> LoweringResult<()> {
        let target = self.delete_target(argument);
        match self.ast().get_expr(target).clone() {
            Expr::Identifier { name, .. } => self.lower_delete_identifier(target, name, dest),
            Expr::StaticMemberExpression {
                object, property, ..
            } => {
                if matches!(self.ast().get_expr(object), Expr::Super { .. }) {
                    let _receiver = self.lower_super_receiver()?;
                    return self.emit_throw_reference_error(self.ast().get_expr(object).span());
                }
                let object_register = self.lower_expr_to_temp(object)?;
                let key_register = self.alloc_temp()?;
                self.emit_load_atom_string(key_register, property)?;
                self.builder.emit_abc(
                    Opcode::DeleteProperty,
                    self.encode_register(dest)?,
                    self.encode_register(object_register)?,
                    self.encode_register(key_register)?,
                );
                Ok(())
            }
            Expr::ComputedMemberExpression {
                object, property, ..
            } => {
                if matches!(self.ast().get_expr(object), Expr::Super { .. }) {
                    let _receiver = self.lower_super_receiver()?;
                    let _key_value = self.lower_expr_to_temp(property)?;
                    return self.emit_throw_reference_error(self.ast().get_expr(object).span());
                }
                let object_register = self.lower_expr_to_temp(object)?;
                let key_register = self.lower_expr_to_temp(property)?;
                self.builder.emit_abc(
                    Opcode::DeleteProperty,
                    self.encode_register(dest)?,
                    self.encode_register(object_register)?,
                    self.encode_register(key_register)?,
                );
                Ok(())
            }
            _ => {
                self.lower_delete_operand_effect(argument)?;
                self.emit_load_bool(dest, true)
            }
        }
    }

    fn lower_delete_operand_effect(&mut self, expr_id: ExprId) -> LoweringResult<()> {
        match self.ast().get_expr(expr_id).clone() {
            Expr::ParenthesizedExpression { expression, .. } => {
                self.lower_delete_operand_effect(expression)
            }
            Expr::UnaryExpression {
                operator, argument, ..
            } => match operator {
                lyng_js_ast::UnaryOp::Delete => {
                    let temp = self.alloc_temp()?;
                    self.lower_delete_expression(argument, temp)
                }
                lyng_js_ast::UnaryOp::TypeOf => {
                    if matches!(self.ast().get_expr(argument), Expr::Identifier { .. }) {
                        let use_site = self.use_site(argument)?;
                        if matches!(
                            use_site.resolution_kind,
                            ResolutionKind::Dynamic
                                | ResolutionKind::Global
                                | ResolutionKind::Unresolved
                        ) {
                            return Ok(());
                        }
                    }
                    self.lower_delete_operand_effect(argument)
                }
                lyng_js_ast::UnaryOp::Minus
                | lyng_js_ast::UnaryOp::Plus
                | lyng_js_ast::UnaryOp::Not
                | lyng_js_ast::UnaryOp::BitNot
                | lyng_js_ast::UnaryOp::Void => self.lower_delete_operand_effect(argument),
            },
            _ => {
                let temp = self.alloc_temp()?;
                self.lower_expr_into(expr_id, temp)
            }
        }
    }

    pub(super) fn lower_delete_identifier(
        &mut self,
        expr_id: ExprId,
        name: AtomId,
        dest: u16,
    ) -> LoweringResult<()> {
        if self.arguments_access(name)?.is_some() {
            return self.emit_load_bool(dest, false);
        }

        let use_site = self.use_site(expr_id)?;
        match use_site.resolution_kind {
            ResolutionKind::Local | ResolutionKind::Captured => {
                let binding_id =
                    use_site
                        .resolved_binding
                        .ok_or(LoweringError::MissingResolvedBinding {
                            expr: expr_id,
                            name,
                        })?;
                let binding = self.binding(binding_id)?;
                if binding.storage_class == StorageClass::DynamicLookup {
                    return self.emit_delete_name(dest, binding.name);
                }
                self.emit_load_bool(dest, false)
            }
            ResolutionKind::Global | ResolutionKind::Unresolved => {
                let index = self.constant_atom(name);
                self.builder
                    .emit_abx(Opcode::DeleteGlobal, self.encode_register(dest)?, index);
                Ok(())
            }
            ResolutionKind::Dynamic => self.emit_delete_name(dest, name),
        }
    }

    fn delete_target(&self, expr: ExprId) -> ExprId {
        let mut current = expr;
        while let Expr::ParenthesizedExpression { expression, .. } = self.ast().get_expr(current) {
            current = *expression;
        }
        current
    }

    fn emit_throw_reference_error(&mut self, span: Span) -> LoweringResult<()> {
        let error = self.alloc_temp()?;
        self.emit_internal_builtin_call_into(js3_reference_error_builtin(), &[], span, error)?;
        self.builder.emit_ax(Opcode::Throw, i32::from(error));
        Ok(())
    }
}

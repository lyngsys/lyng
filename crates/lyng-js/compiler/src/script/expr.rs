use super::*;

impl<'a, 'b> FunctionCompiler<'a, 'b> {
    pub(super) fn lower_expr_into(&mut self, expr_id: ExprId, dest: u16) -> LoweringResult<()> {
        let expr = self.ast().get_expr(expr_id).clone();
        match expr {
            Expr::This { .. } => {
                if let Some(this_override) = self.this_override_register {
                    self.emit_move(dest, this_override)
                } else {
                    self.emit_load_this(dest)
                }
            }
            Expr::Identifier { name, .. } => self.lower_identifier(expr_id, name, dest),
            Expr::NullLiteral { .. } => self.emit_load_null(dest),
            Expr::BooleanLiteral { value, .. } => self.emit_load_bool(dest, value),
            Expr::NumericLiteral { value, .. } => self.emit_load_numeric(dest, value),
            Expr::StringLiteral { value, .. } => self.emit_load_string(dest, value),
            Expr::BigIntLiteral { value, .. } => self.lower_bigint_literal(expr_id, value, dest),
            Expr::RegExpLiteral { value, .. } => self.lower_regexp_literal(expr_id, value, dest),
            Expr::ArrayExpression { elements, .. } => {
                self.lower_array_expression(expr_id, elements, dest)
            }
            Expr::ObjectExpression { properties, .. } => {
                self.lower_object_expression(expr_id, properties, dest)
            }
            Expr::FunctionExpression { function, .. } => {
                self.lower_function_expression(expr_id, function, false, dest)
            }
            Expr::ArrowFunctionExpression { function, .. } => {
                self.lower_function_expression(expr_id, function, true, dest)
            }
            Expr::ClassExpression {
                name,
                super_class,
                body,
                ..
            } => self.lower_class_expression(expr_id, name, super_class, body, dest),
            Expr::TemplateLiteral { template, .. } => {
                self.lower_template_literal(expr_id, template, dest)
            }
            Expr::TaggedTemplateExpression { tag, template, .. } => {
                self.lower_tagged_template_expression(expr_id, tag, template, dest)
            }
            Expr::UpdateExpression {
                operator,
                argument,
                prefix,
                ..
            } => self.lower_update_expression(expr_id, operator, argument, prefix, dest),
            Expr::UnaryExpression {
                operator, argument, ..
            } => self.lower_unary_expression(expr_id, operator, argument, dest),
            Expr::BinaryExpression {
                operator,
                left,
                right,
                ..
            } => self.lower_binary_expression(operator, left, right, dest),
            Expr::LogicalExpression {
                operator,
                left,
                right,
                ..
            } => self.lower_logical_expression(operator, left, right, dest),
            Expr::ConditionalExpression {
                test,
                consequent,
                alternate,
                ..
            } => self.lower_conditional_expression(test, consequent, alternate, dest),
            Expr::AssignmentExpression {
                operator,
                left,
                right,
                ..
            } => self.lower_assignment_expression(operator, left, right, dest),
            Expr::SequenceExpression { expressions, .. } => {
                self.lower_sequence_expression(expressions, dest)
            }
            Expr::StaticMemberExpression {
                object, property, ..
            } => self.lower_static_member_get(object, property, dest),
            Expr::ComputedMemberExpression {
                object, property, ..
            } => self.lower_computed_member_get(object, property, dest),
            Expr::PrivateMemberExpression {
                object, property, ..
            } => self.lower_private_field_get(expr_id, object, property, dest),
            Expr::PrivateInExpression {
                object, property, ..
            } => self.lower_private_has_expression(expr_id, object, property, dest),
            Expr::OptionalChainExpression { .. } => {
                self.lower_optional_chain_expression(expr_id, dest)
            }
            Expr::CallExpression {
                callee, arguments, ..
            } => self.lower_call_expression(expr_id, callee, arguments, dest),
            Expr::NewExpression {
                callee, arguments, ..
            } => self.lower_construct_expression(expr_id, callee, arguments, dest),
            Expr::MetaProperty { meta, property, .. } => {
                self.lower_meta_property(meta, property, dest, expr_id)
            }
            Expr::YieldExpression {
                argument, delegate, ..
            } => self.lower_yield_expression(argument, delegate, dest),
            Expr::AwaitExpression { argument, .. } => self.lower_await_expression(argument, dest),
            Expr::ImportExpression {
                source, options, ..
            } => self.lower_dynamic_import_expression(source, options, dest),
            Expr::ParenthesizedExpression { expression, .. } => {
                self.lower_expr_into(expression, dest)
            }
            _ => Err(LoweringError::UnsupportedExpression { expr: expr_id }),
        }
    }

    pub(super) fn lower_function_expression(
        &mut self,
        expr_id: ExprId,
        function: FunctionId,
        is_arrow: bool,
        dest: u16,
    ) -> LoweringResult<()> {
        let child_index = self.ensure_child_index(function)?;
        let instruction_offset = self.builder.emit_abx(
            Opcode::CreateClosure,
            self.encode_register(dest)?,
            child_index,
        )?;
        let span = self.ast().get_expr(expr_id).span();
        self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation)?;
        if self.ast().get_function(function).name.is_none() {
            let empty_name = self.alloc_temp()?;
            let empty_atom = self.state.atoms.intern_collectible("");
            self.emit_load_atom_string(empty_name, empty_atom)?;
            self.emit_set_function_name(dest, empty_name)?;
        }
        if !self.active_class_contexts.is_empty() {
            self.bind_function_private_env(dest, span)?;
        }
        if is_arrow {
            if let Some(home_object) = self.super_home_object_override {
                self.bind_function_home_object(dest, home_object, span)?;
            }
            if let Some(this_override) = self.this_override_register {
                let home_object = if let Some(home_object) = self.super_home_object_override {
                    home_object
                } else {
                    let home_object = self.alloc_temp()?;
                    self.emit_load_undefined(home_object)?;
                    home_object
                };
                self.emit_internal_builtin_call(
                    lyng_js_types::internal_capture_arrow_context_builtin(),
                    &[dest, this_override, home_object],
                    span,
                )?;
            }
        }
        Ok(())
    }

    pub(super) fn lower_sequence_expression(
        &mut self,
        expressions: lyng_js_ast::NodeList<ExprId>,
        dest: u16,
    ) -> LoweringResult<()> {
        let expressions = self.ast().get_expr_list(expressions).to_vec();
        let Some((last, rest)) = expressions.split_last() else {
            return self.emit_load_undefined(dest);
        };
        for expr in rest {
            let temp = self.alloc_temp()?;
            self.lower_expr_into(*expr, temp)?;
        }
        self.lower_expr_into(*last, dest)
    }

    pub(super) fn lower_dynamic_import_expression(
        &mut self,
        source: ExprId,
        options: Option<ExprId>,
        dest: u16,
    ) -> LoweringResult<()> {
        let source_value = self.lower_expr_to_temp(source)?;
        let options_value = if let Some(options) = options {
            self.lower_expr_to_temp(options)?
        } else {
            let undefined = self.alloc_temp()?;
            self.emit_load_undefined(undefined)?;
            undefined
        };
        self.emit_internal_builtin_call_into(
            lyng_js_types::internal_dynamic_import_builtin(),
            &[source_value, options_value],
            self.ast().get_expr(source).span(),
            dest,
        )
    }

    pub(super) fn lower_yield_expression(
        &mut self,
        argument: Option<ExprId>,
        delegate: bool,
        dest: u16,
    ) -> LoweringResult<()> {
        if delegate {
            return self.lower_delegate_yield_expression(argument, dest);
        }

        let value = if let Some(argument) = argument {
            self.lower_expr_to_temp(argument)?
        } else {
            let value = self.alloc_temp()?;
            self.emit_load_undefined(value)?;
            value
        };
        if self.current_function.is_some_and(|function| {
            self.state.function_kind(function) == FunctionKind::AsyncGenerator
        }) {
            self.builder.emit_ax(Opcode::Await, i32::from(value))?;
        }
        self.builder.emit_ax(Opcode::Yield, i32::from(value))?;
        self.emit_generator_resume_dispatch(dest)
    }

    pub(super) fn lower_await_expression(
        &mut self,
        argument: ExprId,
        dest: u16,
    ) -> LoweringResult<()> {
        self.lower_expr_into(argument, dest)?;
        self.builder.emit_ax(Opcode::Await, i32::from(dest))?;
        Ok(())
    }

    fn lower_delegate_yield_expression(
        &mut self,
        argument: Option<ExprId>,
        dest: u16,
    ) -> LoweringResult<()> {
        let Some(argument) = argument else {
            return Err(LoweringError::UnsupportedExpression {
                expr: ExprId::new(0),
            });
        };
        let iterable = self.lower_expr_to_temp(argument)?;
        let iterator_register = self.alloc_temp()?;
        let done_register = self.alloc_temp()?;
        self.builder.emit_abc(
            Opcode::CreateIterator,
            self.encode_register(iterator_register)?,
            self.encode_register(iterable)?,
            0,
        )?;
        self.builder.emit_abc(
            Opcode::DelegateYield,
            self.encode_register(iterator_register)?,
            self.encode_register(dest)?,
            self.encode_register(done_register)?,
        )?;
        self.emit_generator_delegate_completion_dispatch(dest)
    }

    pub(super) fn lower_array_expression(
        &mut self,
        expr_id: ExprId,
        elements: lyng_js_ast::NodeList<Option<ExprId>>,
        dest: u16,
    ) -> LoweringResult<()> {
        let elements = self.ast().get_opt_expr_list(elements).to_vec();
        let instruction_offset = self.builder.emit_abx(
            Opcode::CreateArray,
            self.encode_register(dest)?,
            u32::try_from(elements.len()).unwrap_or(u32::MAX),
        )?;
        let span = self.ast().get_expr(expr_id).span();
        self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation)?;
        let has_spread = elements
            .iter()
            .flatten()
            .any(|expr| matches!(self.ast().get_expr(*expr), Expr::SpreadElement { .. }));
        if has_spread {
            let next_index = self.alloc_temp()?;
            self.emit_load_smi(next_index, 0)?;
            let one = self.alloc_temp()?;
            self.emit_load_smi(one, 1)?;
            for element in elements {
                match element {
                    None => self.bump_array_literal_index(next_index, one)?,
                    Some(element) => match self.ast().get_expr(element).clone() {
                        Expr::SpreadElement { argument, .. } => {
                            self.lower_array_spread_element(dest, next_index, one, argument)?;
                        }
                        _ => {
                            let value_register = self.lower_expr_to_temp(element)?;
                            self.emit_set_keyed_property(dest, value_register, next_index)?;
                            self.bump_array_literal_index(next_index, one)?;
                        }
                    },
                }
            }
            self.emit_set_property_by_atom(dest, next_index, WellKnownAtom::length.id())?;
            return Ok(());
        }
        for (index, element) in elements.iter().enumerate() {
            let Some(element) = element else {
                continue;
            };
            let value_register = self.lower_expr_to_temp(*element)?;
            let index = u16::try_from(index).map_err(|_| LoweringError::ConstantIndexOverflow {
                index: u32::try_from(index).unwrap_or(u32::MAX),
            })?;
            self.builder.emit_abc(
                Opcode::StoreDenseElement,
                self.encode_register(dest)?,
                self.encode_register(value_register)?,
                index,
            )?;
        }
        if elements.iter().any(Option::is_none) {
            let length_register = self.alloc_temp()?;
            self.emit_load_smi(
                length_register,
                i16::try_from(elements.len()).unwrap_or(i16::MAX),
            )?;
            self.emit_set_property_by_atom(dest, length_register, WellKnownAtom::length.id())?;
        }
        Ok(())
    }

    fn lower_array_spread_element(
        &mut self,
        array_register: u16,
        index_register: u16,
        one_register: u16,
        argument: ExprId,
    ) -> LoweringResult<()> {
        let iterable_register = self.lower_expr_to_temp(argument)?;
        let iterator_register = self.alloc_temp()?;
        let value_register = self.alloc_temp()?;
        let done_register = self.alloc_temp()?;
        self.builder.emit_abc(
            Opcode::CreateIterator,
            self.encode_register(iterator_register)?,
            self.encode_register(iterable_register)?,
            0,
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
        self.emit_set_keyed_property(array_register, value_register, index_register)?;
        self.bump_array_literal_index(index_register, one_register)?;
        let jump_back = self.builder.emit_jump_placeholder(Opcode::Jump)?;
        self.builder.patch_jump_to(jump_back, loop_start)?;
        let close_offset = self.builder.current_offset()?;
        self.builder.patch_jump_to(exit_jump, close_offset)?;
        self.builder.emit_abx(
            Opcode::CloseIterator,
            self.encode_register(iterator_register)?,
            0,
        )?;
        Ok(())
    }

    fn bump_array_literal_index(
        &mut self,
        index_register: u16,
        one_register: u16,
    ) -> LoweringResult<()> {
        let next_index = self.alloc_temp()?;
        self.emit_profiled_binary(Opcode::Add, next_index, index_register, one_register)?;
        self.emit_move(index_register, next_index)
    }

    pub(super) fn lower_expr_to_temp(&mut self, expr: ExprId) -> LoweringResult<u16> {
        let temp = self.alloc_temp()?;
        self.lower_expr_into(expr, temp)?;
        Ok(temp)
    }

    pub(super) fn lower_regexp_literal(
        &mut self,
        expr_id: ExprId,
        value: lyng_js_ast::RegExpLiteralId,
        dest: u16,
    ) -> LoweringResult<()> {
        let literal = self.ast().literals().get_regexp(value).clone();
        let pattern = self.alloc_temp()?;
        let pattern_atom = self.state.atoms.intern(&literal.pattern);
        self.emit_load_atom_string(pattern, pattern_atom)?;

        let flags = self.alloc_temp()?;
        let flags_atom = self.state.atoms.intern(&literal.flags);
        self.emit_load_atom_string(flags, flags_atom)?;

        let span = self.ast().get_expr(expr_id).span();
        self.emit_internal_builtin_call_into(
            lyng_js_types::internal_regexp_literal_builtin(),
            &[pattern, flags],
            span,
            dest,
        )
    }

    pub(super) fn lower_bigint_literal(
        &mut self,
        expr_id: ExprId,
        value: lyng_js_ast::BigIntLiteralId,
        dest: u16,
    ) -> LoweringResult<()> {
        let callee = self.alloc_temp()?;
        self.emit_load_builtin(callee, bigint_builtin())?;

        let this_value = self.alloc_temp()?;
        self.emit_load_undefined(this_value)?;

        let argument = self.alloc_temp()?;
        let text = self.ast().literals().get_bigint(value).to_owned();
        let text_atom = self.state.atoms.intern(&text);
        self.emit_load_atom_string(argument, text_atom)?;

        let arguments = [argument];
        let argument_range = self.materialize_argument_block(&arguments)?;
        let (call_result, call_callee, call_this, move_back) =
            self.bridge_call_registers(dest, callee, this_value)?;
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
            self.call_feedback_metadata(argument_range.argument_count(), 0),
        )?;
        if let Some(dest) = move_back {
            self.emit_move(dest, call_result)?;
        }
        Ok(())
    }
}

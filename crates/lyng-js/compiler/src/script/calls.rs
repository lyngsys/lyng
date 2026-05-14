use super::{
    eval_builtin, internal_construct_super_builtin, internal_direct_eval_builtin,
    internal_private_field_get_builtin, internal_super_constructor_builtin, CallBridgeRegisters,
    CallRange, Expr, ExprId, FeedbackSiteKind, FeedbackSiteMetadata, FunctionCompiler,
    FunctionKind, LoweredCallArguments, LoweringError, LoweringResult, Opcode, ResolutionKind,
    SafepointKind, WellKnownAtom,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CallArgumentPlan {
    expressions: Vec<ExprId>,
    spread_mask: u64,
}

impl CallArgumentPlan {
    const fn len(&self) -> usize {
        self.expressions.len()
    }

    const fn is_empty(&self) -> bool {
        self.expressions.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CallSlotLayout {
    Small { call_base: u16, argument_count: u8 },
    Generic { argument_range: CallRange },
}

impl CallSlotLayout {
    const fn argument_base(self) -> u16 {
        match self {
            Self::Small { call_base, .. } => call_base + 1,
            Self::Generic { argument_range } => argument_range.argument_base(),
        }
    }

    const fn this_register(self) -> Option<u16> {
        match self {
            Self::Small { call_base, .. } => Some(call_base),
            Self::Generic { .. } => None,
        }
    }
}

impl FunctionCompiler<'_, '_> {
    fn direct_eval_identifier(&self, callee: ExprId) -> LoweringResult<bool> {
        let mut current = callee;
        while let Expr::ParenthesizedExpression { expression, .. } = self.ast().get_expr(current) {
            current = *expression;
        }

        let Expr::Identifier { name, .. } = self.ast().get_expr(current) else {
            return Ok(false);
        };
        if *name != WellKnownAtom::eval.id() {
            return Ok(false);
        }
        if self.current_function_ast.is_some_and(|function| {
            self.ast().get_function(function).name == Some(WellKnownAtom::eval.id())
        }) {
            return Ok(false);
        }
        if self.use_site(current)?.resolved_binding.is_some() {
            return Ok(false);
        }

        Ok(true)
    }

    fn lower_direct_eval_call_expression(
        &mut self,
        expr_id: ExprId,
        callee: ExprId,
        arguments: lyng_js_ast::NodeList<ExprId>,
        dest: u16,
    ) -> LoweringResult<bool> {
        if !self.direct_eval_identifier(callee)? {
            return Ok(false);
        }

        let this_override = if let Some(this_override) = self.this_override_register {
            let stable_this = self.alloc_temp()?;
            self.emit_move(stable_this, this_override)?;
            Some(stable_this)
        } else {
            None
        };
        let argument_values = self.lower_call_arguments(arguments)?;
        let callee_register = self.lower_direct_eval_callee(callee)?;
        let mut direct_eval_arguments = Vec::with_capacity(argument_values.registers.len() + 1);
        direct_eval_arguments.push(callee_register);
        direct_eval_arguments.extend(argument_values.registers.iter().copied());
        let instruction_offset = self.emit_internal_builtin_call_into_with_offset_and_this(
            internal_direct_eval_builtin(),
            &direct_eval_arguments,
            self.ast().get_expr(expr_id).span(),
            dest,
            this_override,
        )?;
        let lexical_scopes = self.active_direct_eval_lexical_scopes();
        let flags = self.active_direct_eval_site_flags();
        let annex_b_catch_names = self.active_direct_eval_annex_b_catch_names();
        let parameter_names = self.active_direct_eval_parameter_names();
        if !lexical_scopes.is_empty()
            || !flags.is_empty()
            || !annex_b_catch_names.is_empty()
            || !parameter_names.is_empty()
        {
            self.builder.add_direct_eval_lexical_site(
                instruction_offset,
                lexical_scopes,
                flags,
                annex_b_catch_names,
                parameter_names,
            );
        }
        self.add_direct_eval_spread_feedback_site(instruction_offset, &argument_values)?;
        Ok(true)
    }

    fn lower_direct_eval_tail_call_expression(
        &mut self,
        expr_id: ExprId,
        callee: ExprId,
        arguments: lyng_js_ast::NodeList<ExprId>,
    ) -> LoweringResult<bool> {
        if !self.direct_eval_identifier(callee)? {
            return Ok(false);
        }

        let this_override = if let Some(this_override) = self.this_override_register {
            let stable_this = self.alloc_temp()?;
            self.emit_move(stable_this, this_override)?;
            Some(stable_this)
        } else {
            None
        };
        let argument_values = self.lower_call_arguments(arguments)?;
        let callee_register = self.lower_direct_eval_callee(callee)?;
        let builtin_eval = self.alloc_temp()?;
        self.emit_load_builtin(builtin_eval, eval_builtin())?;
        let is_builtin_eval = self.alloc_temp()?;
        self.emit_profiled_binary(
            Opcode::StrictEqual,
            is_builtin_eval,
            callee_register,
            builtin_eval,
        )?;
        let non_eval_branch = self.builder.emit_cond_jump_placeholder(
            Opcode::JumpIfFalse,
            self.encode_register(is_builtin_eval)?,
        )?;

        let direct_eval_result = self.alloc_temp()?;
        let mut direct_eval_arguments = Vec::with_capacity(argument_values.registers.len() + 1);
        direct_eval_arguments.push(callee_register);
        direct_eval_arguments.extend(argument_values.registers.iter().copied());
        let instruction_offset = self.emit_internal_builtin_call_into_with_offset_and_this(
            internal_direct_eval_builtin(),
            &direct_eval_arguments,
            self.ast().get_expr(expr_id).span(),
            direct_eval_result,
            this_override,
        )?;
        let lexical_scopes = self.active_direct_eval_lexical_scopes();
        let flags = self.active_direct_eval_site_flags();
        let annex_b_catch_names = self.active_direct_eval_annex_b_catch_names();
        let parameter_names = self.active_direct_eval_parameter_names();
        if !lexical_scopes.is_empty()
            || !flags.is_empty()
            || !annex_b_catch_names.is_empty()
            || !parameter_names.is_empty()
        {
            self.builder.add_direct_eval_lexical_site(
                instruction_offset,
                lexical_scopes,
                flags,
                annex_b_catch_names,
                parameter_names,
            );
        }
        self.add_direct_eval_spread_feedback_site(instruction_offset, &argument_values)?;
        self.builder
            .emit_ax(Opcode::Return, i32::from(direct_eval_result))?;

        let non_eval_offset = self.builder.current_offset()?;
        self.builder
            .patch_jump_to(non_eval_branch, non_eval_offset)?;
        let this_register = self.alloc_temp()?;
        self.emit_load_undefined(this_register)?;
        let argument_range = self.materialize_argument_block(&argument_values.registers)?;
        let (tail_callee, tail_this) =
            self.bridge_tail_call_registers(callee_register, this_register)?;
        let instruction_offset = self.builder.emit_tail_call(
            self.encode_register(tail_callee)?,
            self.encode_register(tail_this)?,
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

        Ok(true)
    }

    pub(super) fn reserve_call_bridge_registers(&mut self) -> LoweringResult<()> {
        if self.call_bridge_registers.is_some() {
            return Ok(());
        }

        self.call_bridge_registers = Some(CallBridgeRegisters {
            result: self.alloc_temp()?,
            callee: self.alloc_temp()?,
            this_value: self.alloc_temp()?,
        });
        Ok(())
    }

    fn lower_direct_eval_callee(&mut self, callee: ExprId) -> LoweringResult<u16> {
        let mut current = callee;
        while let Expr::ParenthesizedExpression { expression, .. } = self.ast().get_expr(current) {
            current = *expression;
        }
        let Expr::Identifier { name, .. } = self.ast().get_expr(current) else {
            return self.lower_expr_to_temp(callee);
        };
        let name = *name;
        debug_assert_eq!(name, WellKnownAtom::eval.id());
        let callee_register = self.alloc_temp()?;
        self.emit_load_name(callee_register, name)?;
        Ok(callee_register)
    }

    pub(super) fn bridge_call_registers(
        &mut self,
        dest: u16,
        callee: u16,
        this_value: u16,
    ) -> LoweringResult<(u16, u16, u16, Option<u16>)> {
        if u8::try_from(dest).is_ok()
            && u8::try_from(callee).is_ok()
            && u8::try_from(this_value).is_ok()
        {
            return Ok((dest, callee, this_value, None));
        }

        let bridges = self
            .call_bridge_registers
            .expect("call bridge registers should be reserved before lowering");
        if bridges.result > u16::from(u8::MAX)
            || bridges.callee > u16::from(u8::MAX)
            || bridges.this_value > u16::from(u8::MAX)
        {
            return Err(LoweringError::RegisterOverflow {
                register: bridges.this_value,
            });
        }

        if callee != bridges.callee {
            self.emit_move(bridges.callee, callee)?;
        }
        if this_value != bridges.this_value {
            self.emit_move(bridges.this_value, this_value)?;
        }

        Ok((
            bridges.result,
            bridges.callee,
            bridges.this_value,
            (dest != bridges.result).then_some(dest),
        ))
    }

    pub(super) fn bridge_construct_registers(
        &mut self,
        dest: u16,
        callee: u16,
    ) -> LoweringResult<(u16, u16, Option<u16>)> {
        if u8::try_from(dest).is_ok() && u8::try_from(callee).is_ok() {
            return Ok((dest, callee, None));
        }

        let bridges = self
            .call_bridge_registers
            .expect("call bridge registers should be reserved before lowering");
        if bridges.result > u16::from(u8::MAX) || bridges.callee > u16::from(u8::MAX) {
            return Err(LoweringError::RegisterOverflow {
                register: bridges.callee.max(bridges.result),
            });
        }

        if callee != bridges.callee {
            self.emit_move(bridges.callee, callee)?;
        }

        Ok((
            bridges.result,
            bridges.callee,
            (dest != bridges.result).then_some(dest),
        ))
    }

    pub(super) fn bridge_small_call_registers(
        &mut self,
        dest: u16,
        callee: u16,
    ) -> LoweringResult<(u16, u16, Option<u16>)> {
        if u8::try_from(dest).is_ok() && u8::try_from(callee).is_ok() {
            return Ok((dest, callee, None));
        }

        let bridges = self
            .call_bridge_registers
            .expect("call bridge registers should be reserved before lowering");
        if bridges.result > u16::from(u8::MAX) || bridges.callee > u16::from(u8::MAX) {
            return Err(LoweringError::RegisterOverflow {
                register: bridges.callee.max(bridges.result),
            });
        }

        if callee != bridges.callee {
            self.emit_move(bridges.callee, callee)?;
        }

        Ok((
            bridges.result,
            bridges.callee,
            (dest != bridges.result).then_some(dest),
        ))
    }

    pub(super) fn bridge_tail_call_registers(
        &mut self,
        callee: u16,
        this_value: u16,
    ) -> LoweringResult<(u16, u16)> {
        if u8::try_from(callee).is_ok() && u8::try_from(this_value).is_ok() {
            return Ok((callee, this_value));
        }

        let bridges = self
            .call_bridge_registers
            .expect("call bridge registers should be reserved before lowering");
        if bridges.callee > u16::from(u8::MAX) || bridges.this_value > u16::from(u8::MAX) {
            return Err(LoweringError::RegisterOverflow {
                register: bridges.callee.max(bridges.this_value),
            });
        }

        if callee != bridges.callee {
            self.emit_move(bridges.callee, callee)?;
        }
        if this_value != bridges.this_value {
            self.emit_move(bridges.this_value, this_value)?;
        }

        Ok((bridges.callee, bridges.this_value))
    }

    pub(super) fn lower_call_expression(
        &mut self,
        expr_id: ExprId,
        callee: ExprId,
        arguments: lyng_js_ast::NodeList<ExprId>,
        dest: u16,
    ) -> LoweringResult<()> {
        if self.expr_continues_optional_chain(callee) {
            return self.lower_optional_chain_call_continuation(expr_id, callee, arguments, dest);
        }
        if matches!(self.ast().get_expr(callee), Expr::Super { .. }) {
            return self.lower_super_construct_call(expr_id, arguments, dest);
        }
        if self.lower_direct_eval_call_expression(expr_id, callee, arguments, dest)? {
            return Ok(());
        }
        let argument_plan = self.collect_call_argument_plan(arguments)?;
        let slot_layout = self.reserve_call_slot_layout(&argument_plan)?;
        let callee_register = self.alloc_temp()?;
        let this_register = if let Some(this_register) = slot_layout.this_register() {
            this_register
        } else {
            self.alloc_temp()?
        };
        self.lower_call_target_into(callee, callee_register, this_register)?;
        self.lower_call_arguments_into(&argument_plan, slot_layout.argument_base())?;
        let (instruction_offset, argument_count, call_result, move_back) =
            self.emit_call_from_slot_layout(dest, callee_register, this_register, slot_layout)?;
        let span = self.ast().get_expr(expr_id).span();
        self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation)?;
        self.builder.add_feedback_site(
            instruction_offset,
            FeedbackSiteKind::Call,
            self.call_feedback_metadata(argument_count, argument_plan.spread_mask),
        )?;
        if let Some(dest) = move_back {
            self.emit_move(dest, call_result)?;
        }
        Ok(())
    }

    pub(super) fn lower_tail_call_expression(
        &mut self,
        expr_id: ExprId,
        callee: ExprId,
        arguments: lyng_js_ast::NodeList<ExprId>,
    ) -> LoweringResult<()> {
        if self.expr_continues_optional_chain(callee) {
            let result = self.alloc_temp()?;
            self.lower_optional_chain_call_continuation(expr_id, callee, arguments, result)?;
            self.builder.emit_ax(Opcode::Return, i32::from(result))?;
            return Ok(());
        }
        if matches!(self.ast().get_expr(callee), Expr::Super { .. }) {
            let result = self.alloc_temp()?;
            self.lower_super_construct_call(expr_id, arguments, result)?;
            self.builder.emit_ax(Opcode::Return, i32::from(result))?;
            return Ok(());
        }
        if self.lower_direct_eval_tail_call_expression(expr_id, callee, arguments)? {
            return Ok(());
        }
        let argument_plan = self.collect_call_argument_plan(arguments)?;
        let argument_range = self.reserve_call_argument_range(&argument_plan)?;
        let callee_register = self.alloc_temp()?;
        let this_register = self.alloc_temp()?;
        self.lower_call_target_into(callee, callee_register, this_register)?;
        self.lower_call_arguments_into(&argument_plan, argument_range.argument_base())?;
        let (tail_callee, tail_this) =
            self.bridge_tail_call_registers(callee_register, this_register)?;
        let instruction_offset = self.builder.emit_tail_call(
            self.encode_register(tail_callee)?,
            self.encode_register(tail_this)?,
            argument_range,
        )?;
        let span = self.ast().get_expr(expr_id).span();
        self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation)?;
        self.builder.add_feedback_site(
            instruction_offset,
            FeedbackSiteKind::Call,
            self.call_feedback_metadata(argument_range.argument_count(), argument_plan.spread_mask),
        )?;
        Ok(())
    }

    pub(super) fn lower_construct_expression(
        &mut self,
        expr_id: ExprId,
        callee: ExprId,
        arguments: lyng_js_ast::NodeList<ExprId>,
        dest: u16,
    ) -> LoweringResult<()> {
        let argument_plan = self.collect_call_argument_plan(arguments)?;
        let argument_range = self.reserve_call_argument_range(&argument_plan)?;
        let callee_register = self.lower_expr_to_temp(callee)?;
        self.lower_call_arguments_into(&argument_plan, argument_range.argument_base())?;
        let (call_result, call_callee, move_back) =
            self.bridge_construct_registers(dest, callee_register)?;
        let instruction_offset = self.builder.emit_construct(
            self.encode_register(call_result)?,
            self.encode_register(call_callee)?,
            argument_range,
        )?;
        let span = self.ast().get_expr(expr_id).span();
        self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation)?;
        self.builder.add_feedback_site(
            instruction_offset,
            FeedbackSiteKind::Construct,
            self.call_feedback_metadata(argument_range.argument_count(), argument_plan.spread_mask),
        )?;
        if let Some(dest) = move_back {
            self.emit_move(dest, call_result)?;
        }
        Ok(())
    }

    #[allow(
        clippy::too_many_lines,
        reason = "call target lowering keeps direct eval, super, private, and optional call setup together"
    )]
    pub(super) fn lower_call_target(&mut self, callee: ExprId) -> LoweringResult<(u16, u16)> {
        let callee_register = self.alloc_temp()?;
        let this_register = self.alloc_temp()?;
        self.lower_call_target_into(callee, callee_register, this_register)?;
        Ok((callee_register, this_register))
    }

    #[allow(
        clippy::too_many_lines,
        reason = "call target lowering keeps direct eval, super, private, and optional call setup together"
    )]
    pub(super) fn lower_call_target_into(
        &mut self,
        callee: ExprId,
        callee_dest: u16,
        this_dest: u16,
    ) -> LoweringResult<()> {
        let expr = self.ast().get_expr(callee).clone();
        match expr {
            Expr::ParenthesizedExpression { expression, .. } => {
                self.lower_call_target_into(expression, callee_dest, this_dest)
            }
            Expr::OptionalChainExpression { .. } => {
                let shorted = self.alloc_temp()?;
                self.emit_load_bool(shorted, false)?;
                self.lower_optional_chain_call_target(callee, callee_dest, this_dest, shorted)
            }
            Expr::Identifier { name, .. }
                if self.use_site(callee)?.resolution_kind == ResolutionKind::Dynamic =>
            {
                let reference = self.alloc_temp()?;
                self.emit_capture_name(reference, name)?;
                self.emit_load_captured_name(callee_dest, reference)?;
                self.emit_load_captured_name_this(this_dest, reference)
            }
            Expr::StaticMemberExpression {
                object, property, ..
            } => {
                if matches!(self.ast().get_expr(object), Expr::Super { .. }) {
                    self.lower_super_receiver_into(this_dest)?;
                    let key = self.alloc_temp()?;
                    self.emit_load_atom_string(key, property)?;
                    self.emit_super_property_get(
                        this_dest,
                        key,
                        self.ast().get_expr(object).span(),
                        callee_dest,
                    )?;
                    return Ok(());
                }
                self.lower_expr_into(object, this_dest)?;
                self.emit_get_property_by_atom(callee_dest, this_dest, property)
            }
            Expr::ComputedMemberExpression {
                object, property, ..
            } => {
                if matches!(self.ast().get_expr(object), Expr::Super { .. }) {
                    self.lower_super_receiver_into(this_dest)?;
                    let key = self.lower_expr_to_temp(property)?;
                    self.emit_super_property_get(
                        this_dest,
                        key,
                        self.ast().get_expr(object).span(),
                        callee_dest,
                    )?;
                    return Ok(());
                }
                self.lower_expr_into(object, this_dest)?;
                let key = self.lower_expr_to_temp(property)?;
                self.emit_get_keyed_property(callee_dest, this_dest, key)
            }
            Expr::PrivateMemberExpression {
                object, property, ..
            } => {
                self.lower_expr_into(object, this_dest)?;
                let descriptor_index = {
                    let private_use = self.private_use(callee)?;
                    let layout = self
                        .state
                        .sema
                        .class_private_layouts
                        .get_by_scope(private_use.defining_scope())
                        .ok_or(LoweringError::UnsupportedExpression { expr: callee })?;
                    Self::private_access_descriptor_index_for_layout(layout, property, false)?
                };
                let descriptor = self.alloc_temp()?;
                let descriptor_smi = i16::try_from(descriptor_index)
                    .map_err(|_| LoweringError::UnsupportedExpression { expr: callee })?;
                self.emit_load_smi(descriptor, descriptor_smi)?;
                let depth = self.alloc_temp()?;
                let class_depth = i16::try_from(self.private_use(callee)?.class_depth())
                    .map_err(|_| LoweringError::UnsupportedExpression { expr: callee })?;
                self.emit_load_smi(depth, class_depth)?;
                self.emit_internal_builtin_call_into(
                    internal_private_field_get_builtin(),
                    &[this_dest, descriptor, depth],
                    self.ast().get_expr(callee).span(),
                    callee_dest,
                )
            }
            _ => {
                self.lower_expr_into(callee, callee_dest)?;
                self.emit_load_undefined(this_dest)
            }
        }
    }

    fn lower_super_receiver_into(&mut self, dest: u16) -> LoweringResult<()> {
        if let Some(this_override) = self.this_override_register {
            self.emit_move(dest, this_override)
        } else {
            self.emit_load_this(dest)
        }
    }

    fn collect_call_argument_plan(
        &self,
        arguments: lyng_js_ast::NodeList<ExprId>,
    ) -> LoweringResult<CallArgumentPlan> {
        let mut plan = CallArgumentPlan::default();
        for (index, argument) in self
            .ast()
            .get_expr_list(arguments)
            .to_vec()
            .into_iter()
            .enumerate()
        {
            if let Expr::SpreadElement { argument, .. } = self.ast().get_expr(argument).clone() {
                if index >= u64::BITS as usize {
                    return Err(LoweringError::UnsupportedExpression { expr: argument });
                }
                plan.spread_mask |= 1_u64 << index;
                plan.expressions.push(argument);
            } else {
                plan.expressions.push(argument);
            }
        }
        Ok(plan)
    }

    fn reserve_call_argument_range(
        &mut self,
        argument_plan: &CallArgumentPlan,
    ) -> LoweringResult<CallRange> {
        if argument_plan.is_empty() {
            return Ok(CallRange::new(0, 0));
        }

        let count = u16::try_from(argument_plan.len())
            .map_err(|_| LoweringError::RegisterOverflow { register: u16::MAX })?;
        let base = self
            .builder
            .try_alloc_registers(count)
            .ok_or(LoweringError::RegisterOverflow { register: u16::MAX })?;
        Ok(CallRange::new(base, count))
    }

    fn reserve_call_slot_layout(
        &mut self,
        argument_plan: &CallArgumentPlan,
    ) -> LoweringResult<CallSlotLayout> {
        if argument_plan.spread_mask == 0
            && argument_plan.len() <= 3
            && u8::try_from(self.builder.header().register_count()).is_ok()
        {
            let block_width = u16::try_from(argument_plan.len() + 1)
                .map_err(|_| LoweringError::RegisterOverflow { register: u16::MAX })?;
            if let Some(call_base) = self.builder.try_alloc_registers(block_width) {
                let argument_count = u8::try_from(argument_plan.len())
                    .map_err(|_| LoweringError::RegisterOverflow { register: u16::MAX })?;
                return Ok(CallSlotLayout::Small {
                    call_base,
                    argument_count,
                });
            }
        }

        Ok(CallSlotLayout::Generic {
            argument_range: self.reserve_call_argument_range(argument_plan)?,
        })
    }

    fn lower_call_arguments_into(
        &mut self,
        argument_plan: &CallArgumentPlan,
        base: u16,
    ) -> LoweringResult<()> {
        for (index, expression) in argument_plan.expressions.iter().copied().enumerate() {
            let target = base + u16::try_from(index).unwrap_or(u16::MAX);
            self.lower_expr_into(expression, target)?;
        }
        Ok(())
    }

    fn emit_call_from_slot_layout(
        &mut self,
        dest: u16,
        callee_register: u16,
        this_register: u16,
        slot_layout: CallSlotLayout,
    ) -> LoweringResult<(u32, u16, u16, Option<u16>)> {
        match slot_layout {
            CallSlotLayout::Small {
                call_base,
                argument_count,
            } => {
                if this_register != call_base {
                    self.emit_move(call_base, this_register)?;
                }
                let (call_result, call_callee, move_back) =
                    self.bridge_small_call_registers(dest, callee_register)?;
                let instruction_offset = self.builder.emit_small_call(
                    self.encode_register(call_result)?,
                    self.encode_register(call_callee)?,
                    self.encode_register(call_base)?,
                    argument_count,
                )?;
                Ok((
                    instruction_offset,
                    u16::from(argument_count),
                    call_result,
                    move_back,
                ))
            }
            CallSlotLayout::Generic { argument_range } => {
                let (call_result, call_callee, call_this, move_back) =
                    self.bridge_call_registers(dest, callee_register, this_register)?;
                let instruction_offset = self.builder.emit_call(
                    self.encode_register(call_result)?,
                    self.encode_register(call_callee)?,
                    self.encode_register(call_this)?,
                    argument_range,
                )?;
                Ok((
                    instruction_offset,
                    argument_range.argument_count(),
                    call_result,
                    move_back,
                ))
            }
        }
    }

    pub(super) fn emit_call_with_prelowered_target(
        &mut self,
        expr_id: ExprId,
        callee_register: u16,
        this_register: u16,
        arguments: lyng_js_ast::NodeList<ExprId>,
        dest: u16,
    ) -> LoweringResult<()> {
        let argument_plan = self.collect_call_argument_plan(arguments)?;
        let slot_layout = self.reserve_call_slot_layout(&argument_plan)?;
        self.lower_call_arguments_into(&argument_plan, slot_layout.argument_base())?;
        let (instruction_offset, argument_count, call_result, move_back) =
            self.emit_call_from_slot_layout(dest, callee_register, this_register, slot_layout)?;
        let span = self.ast().get_expr(expr_id).span();
        self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation)?;
        self.builder.add_feedback_site(
            instruction_offset,
            FeedbackSiteKind::Call,
            self.call_feedback_metadata(argument_count, argument_plan.spread_mask),
        )?;
        if let Some(dest) = move_back {
            self.emit_move(dest, call_result)?;
        }
        Ok(())
    }

    pub(super) fn lower_call_arguments(
        &mut self,
        arguments: lyng_js_ast::NodeList<ExprId>,
    ) -> LoweringResult<LoweredCallArguments> {
        let mut lowered = LoweredCallArguments::default();
        for (index, argument) in self
            .ast()
            .get_expr_list(arguments)
            .to_vec()
            .into_iter()
            .enumerate()
        {
            let register = if let Expr::SpreadElement { argument, .. } =
                self.ast().get_expr(argument).clone()
            {
                if index >= u64::BITS as usize {
                    return Err(LoweringError::UnsupportedExpression { expr: argument });
                }
                lowered.spread_mask |= 1_u64 << index;
                self.lower_expr_to_temp(argument)?
            } else {
                self.lower_expr_to_temp(argument)?
            };
            lowered.registers.push(register);
        }
        Ok(lowered)
    }

    pub(super) fn materialize_argument_block(
        &mut self,
        arguments: &[u16],
    ) -> LoweringResult<CallRange> {
        if arguments.is_empty() {
            return Ok(CallRange::new(0, 0));
        }

        let count = u16::try_from(arguments.len())
            .map_err(|_| LoweringError::RegisterOverflow { register: u16::MAX })?;
        let base = self
            .builder
            .try_alloc_registers(count)
            .ok_or(LoweringError::RegisterOverflow { register: u16::MAX })?;
        for (index, source) in arguments.iter().enumerate() {
            let target = base + u16::try_from(index).unwrap_or(u16::MAX);
            self.emit_move(target, *source)?;
        }

        Ok(CallRange::new(base, count))
    }

    #[allow(
        clippy::unused_self,
        reason = "feedback metadata is kept as an emitter helper beside call lowering"
    )]
    pub(super) const fn call_feedback_metadata(
        &self,
        expected_arity: u16,
        spread_mask: u64,
    ) -> FeedbackSiteMetadata {
        if spread_mask == 0 {
            FeedbackSiteMetadata::ExpectedArity(expected_arity)
        } else {
            FeedbackSiteMetadata::CallArguments {
                expected_arity,
                spread_mask,
            }
        }
    }

    fn add_direct_eval_spread_feedback_site(
        &mut self,
        instruction_offset: u32,
        argument_values: &LoweredCallArguments,
    ) -> LoweringResult<()> {
        if argument_values.spread_mask == 0 {
            return Ok(());
        }
        if argument_values.spread_mask & (1_u64 << 63) != 0 {
            return Err(LoweringError::RegisterOverflow { register: u16::MAX });
        }
        let shifted_spread_mask = argument_values.spread_mask << 1;
        let expected_arity = u16::try_from(argument_values.registers.len() + 1)
            .map_err(|_| LoweringError::RegisterOverflow { register: u16::MAX })?;
        self.builder.add_feedback_site(
            instruction_offset,
            FeedbackSiteKind::Call,
            self.call_feedback_metadata(expected_arity, shifted_spread_mask),
        )?;
        Ok(())
    }

    fn lower_super_construct_call(
        &mut self,
        expr_id: ExprId,
        arguments: lyng_js_ast::NodeList<ExprId>,
        dest: u16,
    ) -> LoweringResult<()> {
        let current_direct_eval_arrow = self.current_function_ast.is_some_and(|function| {
            matches!(
                self.ast().get_function(function).kind,
                FunctionKind::Arrow | FunctionKind::AsyncArrow
            )
        });
        if self.state.sema.direct_eval_allows_super
            && (self.current_function.is_none() || current_direct_eval_arrow)
        {
            let span = self.ast().get_expr(expr_id).span();
            let super_constructor = self.alloc_temp()?;
            self.emit_internal_builtin_call_into(
                internal_super_constructor_builtin(),
                &[],
                span,
                super_constructor,
            )?;
            let argument_values = self.lower_call_arguments(arguments)?;
            let mut super_arguments = Vec::with_capacity(argument_values.registers.len() + 1);
            super_arguments.push(super_constructor);
            super_arguments.extend(argument_values.registers.iter().copied());
            let instruction_offset = self.emit_internal_builtin_call_into_with_offset(
                internal_construct_super_builtin(),
                &super_arguments,
                span,
                dest,
            )?;
            if argument_values.spread_mask != 0 {
                if argument_values.spread_mask & (1_u64 << 63) != 0 {
                    return Err(LoweringError::RegisterOverflow { register: u16::MAX });
                }
                let expected_arity = u16::try_from(super_arguments.len())
                    .map_err(|_| LoweringError::RegisterOverflow { register: u16::MAX })?;
                self.builder.add_feedback_site(
                    instruction_offset,
                    FeedbackSiteKind::Call,
                    self.call_feedback_metadata(expected_arity, argument_values.spread_mask << 1),
                )?;
            }
            return Ok(());
        }

        let Some(current_function) = self.current_function else {
            return Err(LoweringError::UnsupportedExpression { expr: expr_id });
        };
        let Some(owner) = self.state.nearest_non_arrow_owner(current_function) else {
            return Err(LoweringError::UnsupportedExpression { expr: expr_id });
        };
        let function = self.state.sema.function_table.get(owner).function_id;
        let Some(plan) = self.state.class_constructor_plan(function) else {
            return Err(LoweringError::UnsupportedExpression { expr: expr_id });
        };
        if !plan.derived {
            return Err(LoweringError::UnsupportedExpression { expr: expr_id });
        }

        let span = self.ast().get_expr(expr_id).span();
        let super_constructor = self.alloc_temp()?;
        self.emit_internal_builtin_call_into(
            internal_super_constructor_builtin(),
            &[],
            span,
            super_constructor,
        )?;
        let argument_values = self.lower_call_arguments(arguments)?;
        let mut super_arguments = Vec::with_capacity(argument_values.registers.len() + 1);
        super_arguments.push(super_constructor);
        super_arguments.extend(argument_values.registers.iter().copied());
        let instruction_offset = self.emit_internal_builtin_call_into_with_offset(
            internal_construct_super_builtin(),
            &super_arguments,
            span,
            dest,
        )?;
        if argument_values.spread_mask != 0 {
            if argument_values.spread_mask & (1_u64 << 63) != 0 {
                return Err(LoweringError::RegisterOverflow { register: u16::MAX });
            }
            let expected_arity = u16::try_from(super_arguments.len())
                .map_err(|_| LoweringError::RegisterOverflow { register: u16::MAX })?;
            self.builder.add_feedback_site(
                instruction_offset,
                FeedbackSiteKind::Call,
                self.call_feedback_metadata(expected_arity, argument_values.spread_mask << 1),
            )?;
        }
        self.emit_derived_class_super_call_epilogue(dest)
    }
}

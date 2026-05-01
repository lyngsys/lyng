use super::state::ClassFunctionMetadata;
use super::stmt::ObjectRestExcludedKey;
use super::*;
use lyng_js_bytecode::{GlobalLexicalBindingPlan, GlobalScriptInstantiationPlan};
use lyng_js_sema::{DeclarationKind, ScopeId, ScopeKind};

pub fn compile_script(
    parsed: &lyng_js_ast::ParsedScript,
    sema: &lyng_js_sema::ScriptSema,
    atoms: &mut AtomTable,
) -> LoweringResult<CompiledScriptUnit> {
    let source = parsed.ast.get_script(parsed.root).span.source;
    let mut compilation = CompilationState::new(
        ProgramSource {
            ast: &parsed.ast,
            body: parsed.ast.get_script(parsed.root).body,
            span: parsed.ast.get_script(parsed.root).span,
            strict: parsed.strict,
            kind: ProgramRootKind::Script,
        },
        sema.view(),
        atoms,
    )?;
    let entry = compilation.compile_root_entry()?;
    let instantiation_plan = derive_global_script_instantiation_plan(&compilation)?;
    let (functions, unit_atoms) = compilation.into_parts();
    Ok(CompiledScriptUnit::new(source, entry, functions)
        .with_atoms(unit_atoms)
        .with_instantiation_plan(instantiation_plan)
        .with_source_text(parsed.source_text.clone()))
}

fn derive_global_script_instantiation_plan(
    compilation: &CompilationState<'_>,
) -> LoweringResult<GlobalScriptInstantiationPlan> {
    let sema = compilation.sema;
    let atoms: &AtomTable = &*compilation.atoms;
    let global_scope_id = ScopeId::new(0);
    let global_scope = sema.scope_table.get(global_scope_id);
    debug_assert_eq!(global_scope.kind, ScopeKind::Global);

    let mut lexical_names = Vec::new();
    let mut lexical_bindings = Vec::new();
    let mut function_names = Vec::new();
    let mut var_names = Vec::new();

    for &binding_id in &global_scope.bindings {
        let binding = sema.binding_table.get(binding_id);
        if binding.scope != global_scope_id {
            continue;
        }
        match binding.kind {
            DeclarationKind::Function => {
                push_unique_name(&mut function_names, atoms.resolve(binding.name))
            }
            DeclarationKind::Var => push_unique_name(&mut var_names, atoms.resolve(binding.name)),
            kind if kind.is_lexical() => {
                let name = atoms.resolve(binding.name);
                push_unique_name(&mut lexical_names, name);
                lexical_bindings.push(GlobalLexicalBindingPlan::new(
                    name,
                    compilation.runtime_slot_for_binding(binding_id)?,
                ));
            }
            _ => {}
        }
    }

    Ok(GlobalScriptInstantiationPlan::new(
        lexical_names,
        lexical_bindings,
        function_names,
        var_names,
    ))
}

fn push_unique_name(names: &mut Vec<Box<str>>, name: &str) {
    if !names.iter().any(|candidate| candidate.as_ref() == name) {
        names.push(name.to_owned().into_boxed_str());
    }
}

impl<'a, 'b> FunctionCompiler<'a, 'b> {
    pub(super) fn for_root(
        state: &'b mut CompilationState<'a>,
        entry: BytecodeFunctionId,
    ) -> LoweringResult<Self> {
        let binding_count = state.sema.binding_table.len();
        let root_needs_environment = state.root_needs_environment();
        let root_kind = match state.program.kind {
            ProgramRootKind::Script => BytecodeFunctionKind::Script,
            ProgramRootKind::Module => BytecodeFunctionKind::Module,
        };
        let mut builder = BytecodeBuilder::new(entry, root_kind);
        builder.set_flags(BytecodeFunctionFlags::new(state.program.strict, false));
        builder.set_this_mode(match state.program.kind {
            ProgramRootKind::Script => ThisMode::Global,
            ProgramRootKind::Module => ThisMode::Strict,
        });
        builder.set_needs_environment(root_needs_environment);
        builder.set_environment_bindings(state.root_environment_bindings().to_vec());
        let root_scope = ScopeId::new(0);
        let scope_count = state.sema.scope_table.len();

        Ok(Self {
            state,
            builder,
            current_function: None,
            current_function_ast: None,
            current_scope: root_scope,
            body_scope: root_scope,
            scope_child_cursors: vec![0; scope_count],
            local_registers: vec![None; binding_count],
            atom_constants: HashMap::new(),
            float_constants: HashMap::new(),
            builtin_constants: HashMap::new(),
            child_indices: HashMap::new(),
            hoisted_function_decls: HashSet::new(),
            block_instantiated_function_decls: HashSet::new(),
            hoisted_default_export_functions: HashSet::new(),
            parameter_sources: Vec::new(),
            result_register: Some(0),
            call_bridge_registers: None,
            generator_resume_registers: None,
            completion_registers: None,
            control_targets: Vec::new(),
            next_control_target_id: 1,
            finally_stack: Vec::new(),
            this_override_register: None,
            super_home_object_override: None,
            active_class_body: None,
            active_class_span: None,
            active_class_contexts: Vec::new(),
            active_direct_eval_scopes: Vec::new(),
            in_class_field_initializer: false,
            active_disposal_scopes: Vec::new(),
        })
    }

    pub(super) fn for_function(
        state: &'b mut CompilationState<'a>,
        function: FunctionId,
        sema_id: FunctionSemaId,
        id: BytecodeFunctionId,
    ) -> LoweringResult<Self> {
        let binding_count = state.sema.binding_table.len();
        let function_record = state
            .sema
            .function_table
            .as_slice()
            .get(sema_id.raw() as usize)
            .ok_or(LoweringError::MissingFunctionRecord { function })?;
        let ast_function = state.program.ast.get_function(function).clone();
        let function_kind = bytecode_function_kind(function, ast_function.kind)?;
        let this_mode = function_this_mode(function_kind, function_record.strict);
        let parameter_count = u16::try_from(
            state
                .program
                .ast
                .get_pattern_list(ast_function.params.params)
                .len(),
        )
        .unwrap_or(u16::MAX);
        let minimum_argument_count =
            expected_argument_count(state.program.ast, &ast_function.params);
        let activation = state.activation(sema_id).clone();

        let class_metadata = state.class_function_metadata(function);
        let class_constructor = class_metadata
            .map(|metadata| metadata.class_constructor)
            .unwrap_or(false);
        let class_constructor_needs_environment = state
            .class_constructor_plan(function)
            .map(|plan| plan.needs_environment)
            .unwrap_or(false);
        let constructible = function_constructible(ast_function.kind, class_metadata);
        let has_prototype_property =
            function_has_prototype_property(ast_function.kind, class_metadata);
        let derived_class_constructor = class_metadata
            .map(|metadata| metadata.derived_class_constructor)
            .unwrap_or(false);
        let mut builder = BytecodeBuilder::new(id, function_kind);
        builder.set_name(if class_constructor {
            None
        } else {
            ast_function.name
        });
        builder.set_flags(
            BytecodeFunctionFlags::new(function_record.strict, false)
                .with_constructible(constructible)
                .with_has_prototype_property(has_prototype_property)
                .with_class_constructor(class_constructor)
                .with_derived_class_constructor(derived_class_constructor)
                .with_generator(matches!(
                    ast_function.kind,
                    FunctionKind::Generator | FunctionKind::AsyncGenerator
                ))
                .with_async_function(matches!(
                    ast_function.kind,
                    FunctionKind::Async | FunctionKind::AsyncArrow | FunctionKind::AsyncGenerator
                )),
        );
        builder.set_this_mode(this_mode);
        builder.set_arguments_mode(activation.arguments_mode);
        builder.set_parameter_counts(parameter_count, minimum_argument_count);
        builder.set_needs_environment(
            activation.needs_environment
                || derived_class_constructor
                || class_constructor_needs_environment,
        );
        builder.set_environment_bindings(state.function_environment_bindings(sema_id).to_vec());
        builder.set_has_rest_parameter(activation.has_rest_parameter);
        builder.set_source_span(
            class_metadata
                .and_then(|metadata| metadata.class_source_span)
                .or(Some(ast_function.span)),
        );
        let current_scope = function_record
            .param_scope
            .unwrap_or(function_record.scope_root);
        let body_scope = function_record.scope_root;
        let scope_count = state.sema.scope_table.len();
        let in_class_field_initializer =
            state.class_field_initializer_functions.contains(&function);

        Ok(Self {
            state,
            builder,
            current_function: Some(sema_id),
            current_function_ast: Some(function),
            current_scope,
            body_scope,
            scope_child_cursors: vec![0; scope_count],
            local_registers: vec![None; binding_count],
            atom_constants: HashMap::new(),
            float_constants: HashMap::new(),
            builtin_constants: HashMap::new(),
            child_indices: HashMap::new(),
            hoisted_function_decls: HashSet::new(),
            block_instantiated_function_decls: HashSet::new(),
            hoisted_default_export_functions: HashSet::new(),
            parameter_sources: Vec::new(),
            result_register: None,
            call_bridge_registers: None,
            generator_resume_registers: None,
            completion_registers: None,
            control_targets: Vec::new(),
            next_control_target_id: 1,
            finally_stack: Vec::new(),
            this_override_register: None,
            super_home_object_override: None,
            active_class_body: None,
            active_class_span: None,
            active_class_contexts: Vec::new(),
            active_direct_eval_scopes: Vec::new(),
            in_class_field_initializer,
            active_disposal_scopes: Vec::new(),
        })
    }

    pub(super) fn lower(mut self) -> LoweringResult<BytecodeFunction> {
        self.emit_capture_descriptors()?;
        self.reserve_parameter_registers()?;
        self.reserve_call_bridge_registers()?;

        if self.result_register.is_some() {
            let result_register = self.alloc_temp()?;
            self.result_register = Some(result_register);
            self.emit_load_undefined(result_register)?;
        }

        self.emit_parameter_environment_prologue()?;
        self.current_scope = self.body_scope;
        self.builder
            .set_parameter_initializer_end_offset(self.builder.current_offset()?);
        self.emit_hoisted_function_declarations()?;
        self.emit_class_constructor_field_prologue()?;
        self.emit_generator_start_suspend_point()?;

        if let Some(function) = self.current_function_ast {
            let ast_function = self.ast().get_function(function).clone();
            if let Some(expression_body) = ast_function.expression_body {
                self.lower_tail_return_expression(expression_body)?;
            } else {
                self.lower_statement_list_with_disposal(ast_function.body, ast_function.span)?;
                self.builder.emit_ax(Opcode::ReturnUndefined, 0)?;
            }
        } else {
            self.lower_statement_list_with_disposal(self.state.program.body, self.root_span())?;
            match self.state.program.kind {
                ProgramRootKind::Script => {
                    let result_register = self
                        .result_register
                        .expect("script lowers into a result register");
                    self.builder
                        .emit_ax(Opcode::Return, i32::from(result_register))?;
                }
                ProgramRootKind::Module => {
                    self.builder.emit_ax(Opcode::ReturnUndefined, 0)?;
                }
            }
        }

        Ok(self.builder.finish()?)
    }

    pub(super) fn emit_capture_descriptors(&mut self) -> LoweringResult<()> {
        let Some(function_id) = self.current_function else {
            return Ok(());
        };
        let captures = self
            .state
            .sema
            .function_table
            .get(function_id)
            .captures
            .clone();
        for binding_id in captures {
            let binding = self.binding(binding_id)?;
            if binding.storage_class != StorageClass::EnvironmentSlot {
                continue;
            }
            let depth = self.capture_source_depth(binding.scope)?;
            let slot = self.state.runtime_slot_for_binding(binding_id)?;
            self.builder.add_capture(CaptureDescriptor::new(
                Some(binding.name),
                CaptureSource::EnvironmentSlot {
                    depth: u16::from(depth),
                    slot: u16::try_from(slot)
                        .map_err(|_| LoweringError::ConstantIndexOverflow { index: u32::MAX })?,
                },
            ))?;
        }
        Ok(())
    }

    fn emit_generator_start_suspend_point(&mut self) -> LoweringResult<()> {
        let Some(function_id) = self.current_function_ast else {
            return Ok(());
        };
        if !matches!(
            self.ast().get_function(function_id).kind,
            FunctionKind::Generator | FunctionKind::AsyncGenerator
        ) {
            return Ok(());
        }
        self.builder.emit_ax(Opcode::SuspendGeneratorStart, 0)?;
        Ok(())
    }

    pub(super) fn reserve_parameter_registers(&mut self) -> LoweringResult<()> {
        let Some(function_id) = self.current_function_ast else {
            return Ok(());
        };
        let ast_function = self.ast().get_function(function_id).clone();
        let activation = self.current_activation()?.clone();

        for pattern_id in self
            .ast()
            .get_pattern_list(ast_function.params.params)
            .to_vec()
        {
            let register = self
                .builder
                .try_alloc_register()
                .ok_or(LoweringError::RegisterOverflow { register: u16::MAX })?;
            self.encode_register(register)?;
            let binding_id = match self.ast().get_pattern(pattern_id).clone() {
                Pattern::Identifier { .. } => {
                    Some(self.declared_binding_for_pattern(pattern_id, DeclarationKind::Parameter)?)
                }
                _ => None,
            };
            self.parameter_sources.push(ParameterSource {
                register,
                pattern: pattern_id,
                binding: binding_id,
            });
            if let Some(binding_id) = binding_id {
                if self.binding(binding_id)?.storage_class == StorageClass::FrameLocal
                    && activation.arguments_mode != ArgumentsMode::Mapped
                {
                    self.local_registers[binding_id.raw() as usize] = Some(register);
                }
            }
        }

        if let Some(rest_pattern) = ast_function.params.rest {
            if matches!(
                self.ast().get_pattern(rest_pattern),
                Pattern::Identifier { .. }
            ) {
                let _ =
                    self.declared_binding_for_pattern(rest_pattern, DeclarationKind::Parameter)?;
            }
        }

        Ok(())
    }

    pub(super) fn emit_parameter_environment_prologue(&mut self) -> LoweringResult<()> {
        if self.current_function.is_none() {
            return Ok(());
        }
        self.emit_named_function_self_binding_prologue()?;
        let activation = self.current_activation()?.clone();
        for parameter in self.parameter_sources.clone() {
            let register = parameter.register;
            let Some(binding_id) = parameter.binding else {
                self.lower_parameter_pattern_initialization(parameter.pattern, register)?;
                continue;
            };
            let binding = self.binding(binding_id)?;
            let needs_env_copy = matches!(
                binding.storage_class,
                StorageClass::EnvironmentSlot | StorageClass::DynamicLookup
            ) || activation.arguments_mode == ArgumentsMode::Mapped;
            if needs_env_copy {
                let slot = self.state.runtime_slot_for_binding(binding_id)?;
                self.emit_store_env_slot(register, 0, slot)?;
            }
        }
        if let Some(rest_pattern) = self
            .current_function_ast
            .and_then(|function| self.ast().get_function(function).params.rest)
        {
            if !matches!(
                self.ast().get_pattern(rest_pattern),
                Pattern::Identifier { .. }
            ) {
                let rest_register = self.alloc_temp()?;
                let rest_slot = activation
                    .rest_slot()
                    .expect("rest parameter should reserve a synthetic rest slot");
                self.emit_load_env_slot(rest_register, 0, u32::from(rest_slot))?;
                self.lower_binding_pattern_initialization(
                    rest_pattern,
                    DeclarationKind::Parameter,
                    rest_register,
                )?;
            }
        }
        Ok(())
    }

    fn lower_parameter_pattern_initialization(
        &mut self,
        pattern: lyng_js_ast::PatternId,
        source_register: u16,
    ) -> LoweringResult<()> {
        self.lower_binding_pattern_initialization(
            pattern,
            DeclarationKind::Parameter,
            source_register,
        )
    }

    pub(super) fn lower_binding_pattern_initialization(
        &mut self,
        pattern: lyng_js_ast::PatternId,
        kind: DeclarationKind,
        source_register: u16,
    ) -> LoweringResult<()> {
        match self.ast().get_pattern(pattern).clone() {
            Pattern::Identifier { name, .. } => {
                let binding_id = match self.declared_binding_for_pattern(pattern, kind) {
                    Ok(binding) => binding,
                    Err(LoweringError::MissingDeclarationBinding { .. })
                        if self.state.atoms.resolve(name).starts_with('_') =>
                    {
                        return Ok(());
                    }
                    Err(error) => return Err(error),
                };
                self.store_binding_value(binding_id, name, source_register)
            }
            Pattern::Object {
                properties, rest, ..
            } => {
                self.emit_check_object_coercible(source_register)?;
                let mut excluded_keys = Vec::new();
                for property in self.ast().get_obj_pattern_prop_list(properties).to_vec() {
                    let value = self.alloc_temp()?;
                    if !property.computed {
                        if let Some(atom) = self.named_property_atom(property.key)? {
                            self.emit_get_property_by_atom(value, source_register, atom)?;
                            excluded_keys.push(ObjectRestExcludedKey::Atom(atom));
                        } else {
                            let key = self.lower_expr_to_temp(property.key)?;
                            self.emit_get_keyed_property(value, source_register, key)?;
                            excluded_keys.push(ObjectRestExcludedKey::Register(key));
                        }
                    } else {
                        let key = self.lower_expr_to_temp(property.key)?;
                        self.emit_get_keyed_property(value, source_register, key)?;
                        excluded_keys.push(ObjectRestExcludedKey::Register(key));
                    }
                    self.lower_binding_pattern_initialization(property.value, kind, value)?;
                }
                if let Some(rest_pattern) = rest {
                    let rest_value = self.create_object_rest_copy_from_register(
                        source_register,
                        &excluded_keys,
                        self.ast().get_pattern(rest_pattern).span(),
                    )?;
                    self.lower_binding_pattern_initialization(rest_pattern, kind, rest_value)?;
                }
                Ok(())
            }
            Pattern::Array { elements, rest, .. } => self
                .lower_array_binding_pattern_initialization(elements, rest, kind, source_register),
            Pattern::Assignment { left, right, .. } => {
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
                    self.binding_pattern_name(left),
                    default_value,
                )?;
                self.lower_binding_pattern_initialization(left, kind, default_value)?;
                let end = self.builder.emit_jump_placeholder(Opcode::Jump)?;
                let use_source_offset = self.builder.current_offset()?;
                self.builder.patch_jump_to(use_source, use_source_offset)?;
                self.lower_binding_pattern_initialization(left, kind, source_register)?;
                let end_offset = self.builder.current_offset()?;
                self.builder.patch_jump_to(end, end_offset)?;
                Ok(())
            }
            Pattern::InvalidPattern { .. } => Err(LoweringError::UnsupportedPattern { pattern }),
        }
    }

    pub(super) fn lower_initializer_with_inferred_name(
        &mut self,
        initializer: ExprId,
        inferred_name: Option<AtomId>,
        value_register: u16,
    ) -> LoweringResult<()> {
        if let Some(name) = inferred_name {
            if let Expr::ClassExpression {
                name: None,
                super_class,
                body,
                ..
            } = self.ast().get_expr(initializer).clone()
            {
                return self.lower_class_expression(
                    initializer,
                    Some(name),
                    super_class,
                    body,
                    value_register,
                );
            }
        }

        self.lower_expr_into(initializer, value_register)?;
        if let Some(name) = inferred_name {
            if self.is_anonymous_function_definition(initializer) {
                let name_value = self.alloc_temp()?;
                self.emit_load_atom_string(name_value, name)?;
                self.emit_set_function_name(value_register, name_value)?;
            }
        }
        Ok(())
    }

    fn binding_pattern_name(&self, pattern: lyng_js_ast::PatternId) -> Option<AtomId> {
        match self.ast().get_pattern(pattern) {
            Pattern::Identifier { name, .. } => Some(*name),
            Pattern::Assignment { left, .. } => self.binding_pattern_name(*left),
            Pattern::Object { .. } | Pattern::Array { .. } | Pattern::InvalidPattern { .. } => None,
        }
    }

    pub(super) fn is_anonymous_function_definition(&self, expr_id: ExprId) -> bool {
        match self.ast().get_expr(expr_id) {
            Expr::FunctionExpression { function, .. } => {
                self.ast().get_function(*function).name.is_none()
            }
            Expr::ArrowFunctionExpression { .. } => true,
            Expr::ClassExpression { name, .. } => name.is_none(),
            Expr::ParenthesizedExpression { expression, .. } => {
                self.is_anonymous_function_definition(*expression)
            }
            _ => false,
        }
    }

    fn lower_array_binding_pattern_initialization(
        &mut self,
        elements: lyng_js_ast::NodeList<Option<lyng_js_ast::ArrayPatternElement>>,
        rest: Option<lyng_js_ast::PatternId>,
        kind: DeclarationKind,
        source_register: u16,
    ) -> LoweringResult<()> {
        let iterator_register = self.alloc_temp()?;
        let value_register = self.alloc_temp()?;
        let done_register = self.alloc_temp()?;
        self.builder.emit_abc(
            Opcode::CreateIterator,
            self.encode_register(iterator_register)?,
            self.encode_register(source_register)?,
            0,
        )?;

        for element in self.ast().get_opt_pattern_elem_list(elements).to_vec() {
            self.builder.emit_abc(
                Opcode::AdvanceIterator,
                self.encode_register(iterator_register)?,
                self.encode_register(value_register)?,
                self.encode_register(done_register)?,
            )?;
            if let Some(element) = element {
                self.lower_binding_pattern_initialization(element.pattern, kind, value_register)?;
            }
        }

        if let Some(rest_pattern) = rest {
            self.lower_array_rest_binding_pattern_from_iterator(
                rest_pattern,
                kind,
                iterator_register,
            )?;
        }

        self.builder.emit_abx(
            Opcode::CloseIterator,
            self.encode_register(iterator_register)?,
            0,
        )?;
        Ok(())
    }

    fn lower_array_rest_binding_pattern_from_iterator(
        &mut self,
        rest_pattern: lyng_js_ast::PatternId,
        kind: DeclarationKind,
        iterator_register: u16,
    ) -> LoweringResult<()> {
        let rest_value = self.alloc_temp()?;
        let instruction_offset =
            self.builder
                .emit_abx(Opcode::CreateArray, self.encode_register(rest_value)?, 0)?;
        self.attach_safepoint(
            instruction_offset,
            self.ast().get_pattern(rest_pattern).span(),
            SafepointKind::Allocation,
        )?;

        let rest_index = self.alloc_temp()?;
        self.emit_load_smi(rest_index, 0)?;
        let one_register = self.alloc_temp()?;
        self.emit_load_smi(one_register, 1)?;
        let value_register = self.alloc_temp()?;
        let done_register = self.alloc_temp()?;

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
        self.emit_set_keyed_property(rest_value, value_register, rest_index)?;
        let next_rest = self.alloc_temp()?;
        self.emit_profiled_binary(Opcode::Add, next_rest, rest_index, one_register)?;
        self.emit_move(rest_index, next_rest)?;
        let jump_back = self.builder.emit_jump_placeholder(Opcode::Jump)?;
        self.builder.patch_jump_to(jump_back, loop_start)?;
        let end = self.builder.current_offset()?;
        self.builder.patch_jump_to(exit_jump, end)?;
        self.emit_set_property_by_atom(rest_value, rest_index, WellKnownAtom::length.id())?;

        self.lower_binding_pattern_initialization(rest_pattern, kind, rest_value)
    }

    fn emit_named_function_self_binding_prologue(&mut self) -> LoweringResult<()> {
        let Some(function_id) = self.current_function_ast else {
            return Ok(());
        };
        let Some(name) = self.ast().get_function(function_id).name else {
            return Ok(());
        };
        let Some(binding_id) = self.named_function_expression_self_binding(function_id, name)?
        else {
            return Ok(());
        };
        if let Some((depth, slot)) = self.binding_env_access(binding_id)? {
            let callee = self.alloc_temp()?;
            self.emit_load_callee(callee)?;
            self.emit_store_env_slot(callee, depth, slot)?;
            return Ok(());
        }
        match self.binding(binding_id)?.storage_class {
            StorageClass::FrameLocal => {
                let register = self.ensure_local_register(binding_id)?;
                self.emit_load_callee(register)?;
            }
            StorageClass::EnvironmentSlot
            | StorageClass::DynamicLookup
            | StorageClass::GlobalName => {
                return Err(LoweringError::UnsupportedFunction {
                    function: function_id,
                });
            }
        }
        Ok(())
    }

    pub(super) fn emit_hoisted_function_declarations(&mut self) -> LoweringResult<()> {
        let statements = if let Some(function_id) = self.current_function_ast {
            self.ast()
                .get_stmt_list(self.ast().get_function(function_id).body)
                .to_vec()
        } else {
            self.ast().get_stmt_list(self.state.program.body).to_vec()
        };

        for stmt in statements {
            self.emit_hoisted_function_declaration_from_statement(stmt)?;
        }

        Ok(())
    }

    fn emit_hoisted_function_declaration_from_statement(
        &mut self,
        stmt: StmtId,
    ) -> LoweringResult<()> {
        let Stmt::Declaration { decl, .. } = self.ast().get_stmt(stmt).clone() else {
            if let Stmt::Labeled { body, .. } = self.ast().get_stmt(stmt).clone() {
                self.emit_hoisted_function_declaration_from_statement(body)?;
            }
            return Ok(());
        };

        match self.ast().get_decl(decl).clone() {
            Decl::Function { function, .. } => {
                self.lower_function_declaration(decl, function)?;
                self.hoisted_function_decls.insert(decl);
            }
            Decl::Export {
                kind: lyng_js_ast::ExportKind::Declaration { decl },
                ..
            } => {
                let Decl::Function { function, .. } = self.ast().get_decl(decl).clone() else {
                    return Ok(());
                };
                let function_record = self.ast().get_function(function);
                if matches!(
                    function_record.kind,
                    FunctionKind::Normal
                        | FunctionKind::Generator
                        | FunctionKind::Async
                        | FunctionKind::AsyncGenerator
                ) {
                    self.lower_function_declaration(decl, function)?;
                    self.hoisted_function_decls.insert(decl);
                }
            }
            Decl::Export {
                kind:
                    lyng_js_ast::ExportKind::Default {
                        declaration: lyng_js_ast::ExportDefaultDecl::Function(function),
                    },
                ..
            } => {
                let function_record = self.ast().get_function(function);
                if matches!(
                    function_record.kind,
                    FunctionKind::Normal
                        | FunctionKind::Generator
                        | FunctionKind::Async
                        | FunctionKind::AsyncGenerator
                ) {
                    self.lower_default_export_declaration(
                        lyng_js_ast::ExportDefaultDecl::Function(function),
                    )?;
                    self.hoisted_default_export_functions.insert(function);
                }
            }
            _ => {}
        }

        Ok(())
    }
}

fn bytecode_function_kind(
    function: FunctionId,
    kind: FunctionKind,
) -> LoweringResult<BytecodeFunctionKind> {
    let _ = function;
    match kind {
        FunctionKind::Normal
        | FunctionKind::Generator
        | FunctionKind::Async
        | FunctionKind::AsyncGenerator => Ok(BytecodeFunctionKind::Function),
        FunctionKind::Arrow | FunctionKind::AsyncArrow => Ok(BytecodeFunctionKind::Arrow),
    }
}

fn function_constructible(
    kind: FunctionKind,
    class_metadata: Option<ClassFunctionMetadata>,
) -> bool {
    match kind {
        FunctionKind::Normal => class_metadata
            .map(|metadata| metadata.constructible)
            .unwrap_or(true),
        FunctionKind::Generator
        | FunctionKind::Arrow
        | FunctionKind::AsyncArrow
        | FunctionKind::Async
        | FunctionKind::AsyncGenerator => false,
    }
}

fn function_has_prototype_property(
    kind: FunctionKind,
    class_metadata: Option<ClassFunctionMetadata>,
) -> bool {
    class_metadata
        .map(|metadata| metadata.has_prototype_property)
        .unwrap_or(matches!(
            kind,
            FunctionKind::Normal | FunctionKind::Generator | FunctionKind::AsyncGenerator
        ))
}

fn function_this_mode(kind: BytecodeFunctionKind, strict: bool) -> ThisMode {
    match kind {
        BytecodeFunctionKind::Arrow => ThisMode::Lexical,
        BytecodeFunctionKind::Function | BytecodeFunctionKind::Builtin => {
            if strict {
                ThisMode::Strict
            } else {
                ThisMode::Global
            }
        }
        BytecodeFunctionKind::Script => ThisMode::Global,
        BytecodeFunctionKind::Module => ThisMode::Strict,
    }
}

fn expected_argument_count(ast: &lyng_js_ast::Ast, params: &lyng_js_ast::FormalParameters) -> u16 {
    let mut count = 0u16;
    for &parameter in ast.get_pattern_list(params.params) {
        if pattern_has_initializer(ast, parameter) {
            break;
        }
        count = count.saturating_add(1);
    }
    count
}

fn pattern_has_initializer(ast: &lyng_js_ast::Ast, pattern: lyng_js_ast::PatternId) -> bool {
    matches!(ast.get_pattern(pattern), Pattern::Assignment { .. })
}

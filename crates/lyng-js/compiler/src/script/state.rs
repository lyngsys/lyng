use super::*;

struct ComputedEnvironmentLayouts {
    scope_environment_bases: Vec<Option<u32>>,
    function_environment_bindings: Vec<Vec<BytecodeEnvironmentBinding>>,
    root_environment_bindings: Vec<BytecodeEnvironmentBinding>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ParameterSource {
    pub(super) register: u16,
    pub(super) pattern: lyng_js_ast::PatternId,
    pub(super) binding: Option<SemanticBindingId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CompletionKind {
    Normal = 0,
    Return = 1,
    Throw = 2,
    Break = 3,
    Continue = 4,
}

impl CompletionKind {
    #[inline]
    pub(super) const fn encoded(self) -> i16 {
        self as i16
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CompletionRegisters {
    pub(super) kind: u16,
    pub(super) value: u16,
    pub(super) target: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ClassFunctionMetadata {
    pub(super) constructible: bool,
    pub(super) has_prototype_property: bool,
    pub(super) class_constructor: bool,
    pub(super) derived_class_constructor: bool,
    pub(super) class_source_span: Option<lyng_js_common::Span>,
    pub(super) class_body: Option<lyng_js_ast::NodeList<lyng_js_ast::ClassElementId>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ClassInstanceElementPlan {
    PublicField {
        key: lyng_js_ast::ExprId,
        value: Option<lyng_js_ast::ExprId>,
        computed: bool,
        computed_key_index: Option<u32>,
    },
    PrivateElement {
        name: AtomId,
        kind: lyng_js_sema::ClassPrivateElementKind,
        value: Option<lyng_js_ast::ExprId>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ClassConstructorPlan {
    pub(super) derived: bool,
    pub(super) class_body: lyng_js_ast::NodeList<lyng_js_ast::ClassElementId>,
    pub(super) instance_elements: Vec<ClassInstanceElementPlan>,
    pub(super) needs_environment: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ControlTargetKind {
    Loop,
    Switch,
    Label,
}

#[derive(Clone, Debug)]
pub(super) struct ControlTarget {
    pub(super) id: u16,
    pub(super) label: Option<AtomId>,
    pub(super) kind: ControlTargetKind,
    pub(super) break_placeholders: Vec<u32>,
    pub(super) continue_placeholders: Vec<u32>,
}

impl ControlTarget {
    pub(super) fn new(id: u16, label: Option<AtomId>, kind: ControlTargetKind) -> Self {
        Self {
            id,
            label,
            kind,
            break_placeholders: Vec::new(),
            continue_placeholders: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct FinallyContext {
    pub(super) normal_entry: Option<u32>,
    pub(super) normal_entry_placeholders: Vec<u32>,
    pub(super) in_finalizer: bool,
}

pub(crate) struct CompilationState<'a> {
    pub(super) program: ProgramSource<'a>,
    pub(super) sema: ProgramSemaView<'a>,
    pub(super) atoms: &'a mut AtomTable,
    pub(super) atom_texts: Vec<(AtomId, CompiledAtom)>,
    pub(super) seen_atoms: HashSet<AtomId>,
    pub(super) functions: Vec<BytecodeFunction>,
    pub(super) ast_to_sema: HashMap<FunctionId, FunctionSemaId>,
    pub(super) parent_functions: Vec<Option<FunctionSemaId>>,
    pub(super) arguments_owners: HashSet<FunctionSemaId>,
    pub(super) activation_plans: Vec<FunctionActivationPlan>,
    pub(super) scope_environment_bases: Vec<Option<u32>>,
    pub(super) function_environment_bindings: Vec<Vec<BytecodeEnvironmentBinding>>,
    pub(super) root_environment_bindings: Vec<BytecodeEnvironmentBinding>,
    pub(super) module_default_export_slot: Option<u32>,
    pub(super) compiled_children: HashMap<FunctionId, BytecodeFunctionId>,
    pub(super) class_function_metadata: HashMap<FunctionId, ClassFunctionMetadata>,
    pub(super) class_constructor_plans: HashMap<FunctionId, ClassConstructorPlan>,
    pub(super) next_function_raw: u32,
}

impl<'a> CompilationState<'a> {
    pub(crate) fn new(
        program: ProgramSource<'a>,
        sema: ProgramSemaView<'a>,
        atoms: &'a mut AtomTable,
    ) -> LoweringResult<Self> {
        let module_default_export_binding = module_default_export_binding(program);
        let ast_to_sema = sema
            .function_table
            .as_slice()
            .iter()
            .enumerate()
            .map(|(index, record)| (record.function_id, FunctionSemaId::new(index as u32)))
            .collect::<HashMap<_, _>>();
        let parent_functions = sema
            .function_table
            .as_slice()
            .iter()
            .enumerate()
            .map(|(index, record)| {
                let current = FunctionSemaId::new(index as u32);
                Ok(parent_function_for(
                    &sema.scope_table,
                    record.scope_root,
                    current,
                ))
            })
            .collect::<LoweringResult<Vec<_>>>()?;
        let arguments_owners = collect_arguments_owners(program, sema, &parent_functions);
        let activation_plans = sema
            .function_table
            .as_slice()
            .iter()
            .enumerate()
            .map(|(index, record)| {
                build_function_activation_plan(
                    program,
                    sema,
                    FunctionSemaId::new(index as u32),
                    record,
                    &arguments_owners,
                    &parent_functions,
                )
            })
            .collect::<LoweringResult<Vec<_>>>()?;
        let computed_environment_layouts = Self::compute_environment_layouts(
            sema,
            &activation_plans,
            module_default_export_binding,
        );
        let module_default_export_slot = module_default_export_binding
            .map(|_| computed_environment_layouts.root_environment_bindings.len() as u32 - 1);
        let (class_function_metadata, class_constructor_plans) =
            collect_class_lowering_metadata(program);

        Ok(Self {
            program,
            sema,
            atoms,
            atom_texts: Vec::new(),
            seen_atoms: HashSet::new(),
            functions: Vec::new(),
            ast_to_sema,
            parent_functions,
            arguments_owners,
            activation_plans,
            scope_environment_bases: computed_environment_layouts.scope_environment_bases,
            function_environment_bindings: computed_environment_layouts
                .function_environment_bindings,
            root_environment_bindings: computed_environment_layouts.root_environment_bindings,
            module_default_export_slot,
            compiled_children: HashMap::new(),
            class_function_metadata,
            class_constructor_plans,
            next_function_raw: 1,
        })
    }

    fn compute_environment_layouts(
        sema: ProgramSemaView<'_>,
        activation_plans: &[FunctionActivationPlan],
        module_default_export_binding: Option<BytecodeEnvironmentBinding>,
    ) -> ComputedEnvironmentLayouts {
        let mut scope_environment_bases = vec![None; sema.scope_table.len()];
        let mut flattened_function_bindings = vec![Vec::new(); activation_plans.len()];
        let mut root_environment_bindings = Vec::new();

        for (index, scope) in sema.scope_table.as_slice().iter().enumerate() {
            let mut scope_bindings = scope_environment_bindings(sema, scope);
            if scope_bindings.is_empty() {
                continue;
            }

            let owner_bindings = scope
                .owning_function
                .and_then(|owner| flattened_function_bindings.get_mut(owner.raw() as usize))
                .unwrap_or(&mut root_environment_bindings);
            let base = u32::try_from(owner_bindings.len()).unwrap_or(u32::MAX);
            scope_environment_bases[index] = Some(base);
            owner_bindings.append(&mut scope_bindings);
        }

        let function_environment_bindings = flattened_function_bindings
            .into_iter()
            .enumerate()
            .map(|(index, flattened)| {
                let mut bindings =
                    synthetic_function_environment_bindings(sema, &activation_plans[index]);
                bindings.extend(flattened);
                bindings
            })
            .collect();

        if let Some(binding) = module_default_export_binding {
            root_environment_bindings.push(binding);
        }

        ComputedEnvironmentLayouts {
            scope_environment_bases,
            function_environment_bindings,
            root_environment_bindings,
        }
    }

    pub(crate) fn compile_root_entry(&mut self) -> LoweringResult<BytecodeFunctionId> {
        let entry = self.alloc_function_id();
        let compiler = FunctionCompiler::for_root(self, entry)?;
        let function = compiler.lower()?;
        self.functions.push(function);
        Ok(entry)
    }

    pub(super) fn ensure_child_compiled(
        &mut self,
        function: FunctionId,
    ) -> LoweringResult<BytecodeFunctionId> {
        if let Some(id) = self.compiled_children.get(&function) {
            return Ok(*id);
        }

        let sema_id = self
            .ast_to_sema
            .get(&function)
            .copied()
            .ok_or(LoweringError::MissingFunctionRecord { function })?;
        let id = self.alloc_function_id();
        self.compiled_children.insert(function, id);

        let compiler = FunctionCompiler::for_function(self, function, sema_id, id)?;
        let lowered = compiler.lower()?;
        self.functions.push(lowered);
        Ok(id)
    }

    pub(super) fn alloc_function_id(&mut self) -> BytecodeFunctionId {
        let id = BytecodeFunctionId::new(
            NonZeroU32::new(self.next_function_raw).expect("function id must remain non-zero"),
        );
        self.next_function_raw = self
            .next_function_raw
            .checked_add(1)
            .expect("function id space should remain within u32");
        id
    }

    pub(super) fn record_atom_text(&mut self, atom: AtomId) {
        if !self.seen_atoms.insert(atom) {
            return;
        }
        let text = if let Some(text) = self.atoms.get(atom) {
            CompiledAtom::from(text)
        } else {
            let units = self
                .atoms
                .get_utf16(atom)
                .expect("compiled atom should resolve to UTF-8 or UTF-16 storage");
            CompiledAtom::from(units.to_vec())
        };
        self.atom_texts.push((atom, text));
    }

    pub(super) fn parent_function(&self, function: FunctionSemaId) -> Option<FunctionSemaId> {
        self.parent_functions
            .get(function.raw() as usize)
            .copied()
            .flatten()
    }

    pub(super) fn class_function_metadata(
        &self,
        function: FunctionId,
    ) -> Option<ClassFunctionMetadata> {
        self.class_function_metadata.get(&function).copied()
    }

    pub(super) fn class_constructor_plan(
        &self,
        function: FunctionId,
    ) -> Option<&ClassConstructorPlan> {
        self.class_constructor_plans.get(&function)
    }

    pub(super) fn activation(&self, function: FunctionSemaId) -> &FunctionActivationPlan {
        &self.activation_plans[function.raw() as usize]
    }

    pub(super) fn function_needs_arguments(&self, function: FunctionSemaId) -> bool {
        self.arguments_owners.contains(&function)
    }

    pub(super) fn function_kind(&self, function: FunctionSemaId) -> FunctionKind {
        let ast_function = self.sema.function_table.get(function).function_id;
        self.program.ast.get_function(ast_function).kind
    }

    pub(super) fn nearest_non_arrow_owner(
        &self,
        function: FunctionSemaId,
    ) -> Option<FunctionSemaId> {
        let mut current = Some(function);
        while let Some(candidate) = current {
            if self.function_kind(candidate) != FunctionKind::Arrow {
                return Some(candidate);
            }
            current = self.parent_function(candidate);
        }
        None
    }

    pub(super) fn arguments_owner_for_current(
        &self,
        current: Option<FunctionSemaId>,
    ) -> Option<FunctionSemaId> {
        let current = current?;
        let owner = self.nearest_non_arrow_owner(current)?;
        self.function_needs_arguments(owner).then_some(owner)
    }

    pub(super) fn environment_depth_to_function(
        &self,
        from: FunctionSemaId,
        owner: FunctionSemaId,
    ) -> LoweringResult<u8> {
        let mut depth = 0u8;
        let mut current = from;

        while current != owner {
            if self.activation(current).needs_environment {
                depth = depth
                    .checked_add(1)
                    .ok_or(LoweringError::InvalidCapturedBindingDepth {
                        binding: SemanticBindingId::new(0),
                        function: Some(from),
                    })?;
            }
            current = self.parent_function(current).ok_or(
                LoweringError::InvalidCapturedBindingDepth {
                    binding: SemanticBindingId::new(0),
                    function: Some(from),
                },
            )?;
        }

        Ok(depth)
    }

    pub(super) fn environment_depth_to_root(&self, from: FunctionSemaId) -> LoweringResult<u8> {
        let mut depth = 0u8;
        let mut current = Some(from);

        while let Some(function) = current {
            if self.activation(function).needs_environment {
                depth = depth
                    .checked_add(1)
                    .ok_or(LoweringError::InvalidCapturedBindingDepth {
                        binding: SemanticBindingId::new(0),
                        function: Some(from),
                    })?;
            }
            current = self.parent_function(function);
        }

        Ok(depth)
    }

    pub(crate) fn runtime_slot_for_binding(
        &self,
        binding: SemanticBindingId,
    ) -> LoweringResult<u32> {
        let binding_record = self.sema.binding_table.get(binding);
        let flattened_slot = binding_record.slot_index.and_then(|slot| {
            self.scope_environment_bases
                .get(binding_record.scope.raw() as usize)
                .copied()
                .flatten()
                .and_then(|base| base.checked_add(slot))
        });
        self.scope_owner(binding_record.scope)
            .map(|owner| {
                self.activation(owner)
                    .runtime_slot_for_binding(binding, flattened_slot)
            })
            .unwrap_or(flattened_slot)
            .ok_or(LoweringError::MissingEnvironmentSlot { binding })
    }

    pub(super) fn root_needs_environment(&self) -> bool {
        self.sema.binding_table.as_slice().iter().any(|binding| {
            ((binding.storage_class == StorageClass::EnvironmentSlot)
                || (binding.storage_class == StorageClass::DynamicLookup
                    && binding.slot_index.is_some()))
                && self.scope_owner(binding.scope).is_none()
        }) || self.module_default_export_slot.is_some()
            || self.has_direct_root_arrow_child()
    }

    pub(super) fn function_environment_bindings(
        &self,
        function: FunctionSemaId,
    ) -> &[BytecodeEnvironmentBinding] {
        &self.function_environment_bindings[function.raw() as usize]
    }

    pub(super) fn scope_environment_base(&self, scope: ScopeId) -> Option<u32> {
        self.scope_environment_bases
            .get(scope.raw() as usize)
            .copied()
            .flatten()
    }

    pub(super) fn scope_environment_bindings_for(
        &self,
        scope: ScopeId,
    ) -> Vec<BytecodeEnvironmentBinding> {
        scope_environment_bindings(self.sema, self.sema.scope_table.get(scope))
    }

    pub(super) fn scope_owner(&self, scope: ScopeId) -> Option<FunctionSemaId> {
        self.sema.scope_table.get(scope).owning_function
    }

    fn has_direct_root_arrow_child(&self) -> bool {
        self.sema
            .function_table
            .as_slice()
            .iter()
            .enumerate()
            .any(|(index, record)| {
                self.parent_functions
                    .get(index)
                    .copied()
                    .flatten()
                    .is_none()
                    && self.program.ast.get_function(record.function_id).kind == FunctionKind::Arrow
            })
    }

    pub(crate) fn into_parts(mut self) -> (Vec<BytecodeFunction>, Vec<(AtomId, CompiledAtom)>) {
        self.record_metadata_atoms();
        (self.functions, self.atom_texts)
    }

    pub(super) fn root_environment_bindings(&self) -> &[BytecodeEnvironmentBinding] {
        &self.root_environment_bindings
    }

    pub(crate) fn sema(&self) -> ProgramSemaView<'_> {
        self.sema
    }

    fn record_metadata_atoms(&mut self) {
        let mut metadata_atoms = Vec::new();
        metadata_atoms.extend(
            self.root_environment_bindings
                .iter()
                .filter_map(|binding| binding.name()),
        );
        metadata_atoms.extend(
            self.function_environment_bindings
                .iter()
                .flat_map(|bindings| bindings.iter().filter_map(|binding| binding.name())),
        );
        metadata_atoms.extend(self.functions.iter().filter_map(BytecodeFunction::name));
        for atom in metadata_atoms {
            self.record_atom_text(atom);
        }
    }

    pub(crate) fn resolve_atom(&self, atom: AtomId) -> &str {
        self.atoms.resolve(atom)
    }

    pub(crate) fn module_default_export_slot(&self) -> Option<u32> {
        self.module_default_export_slot
    }
}

fn module_default_export_binding(program: ProgramSource<'_>) -> Option<BytecodeEnvironmentBinding> {
    if program.kind != ProgramRootKind::Module {
        return None;
    }

    let has_default_export = program.ast.get_stmt_list(program.body).iter().any(|stmt| {
        let lyng_js_ast::Stmt::Declaration { decl, .. } = program.ast.get_stmt(*stmt) else {
            return false;
        };
        matches!(
            program.ast.get_decl(*decl),
            lyng_js_ast::Decl::Export {
                kind: lyng_js_ast::ExportKind::Default { .. },
                ..
            }
        )
    });
    has_default_export.then(|| {
        BytecodeEnvironmentBinding::new(
            Some(WellKnownAtom::default.id()),
            BytecodeEnvironmentSlotFlags::new(false, true, false, false),
        )
    })
}

fn collect_class_lowering_metadata(
    program: ProgramSource<'_>,
) -> (
    HashMap<FunctionId, ClassFunctionMetadata>,
    HashMap<FunctionId, ClassConstructorPlan>,
) {
    let mut function_metadata = HashMap::new();
    let mut constructor_plans = HashMap::new();
    collect_class_lowering_from_stmt_list(
        program.ast,
        program.body,
        &mut function_metadata,
        &mut constructor_plans,
    );
    (function_metadata, constructor_plans)
}

fn collect_class_lowering_from_stmt_list(
    ast: &lyng_js_ast::Ast,
    list: lyng_js_ast::NodeList<lyng_js_ast::StmtId>,
    function_metadata: &mut HashMap<FunctionId, ClassFunctionMetadata>,
    constructor_plans: &mut HashMap<FunctionId, ClassConstructorPlan>,
) {
    for &stmt in ast.get_stmt_list(list) {
        collect_class_lowering_from_stmt(ast, stmt, function_metadata, constructor_plans);
    }
}

fn collect_class_lowering_from_stmt(
    ast: &lyng_js_ast::Ast,
    stmt_id: lyng_js_ast::StmtId,
    function_metadata: &mut HashMap<FunctionId, ClassFunctionMetadata>,
    constructor_plans: &mut HashMap<FunctionId, ClassConstructorPlan>,
) {
    match ast.get_stmt(stmt_id) {
        Stmt::Block { body, .. } => {
            collect_class_lowering_from_stmt_list(ast, *body, function_metadata, constructor_plans);
        }
        Stmt::Empty { .. }
        | Stmt::Continue { .. }
        | Stmt::Break { .. }
        | Stmt::Debugger { .. }
        | Stmt::InvalidStatement { .. } => {}
        Stmt::Expression { expression, .. } => {
            collect_class_lowering_from_expr(
                ast,
                *expression,
                function_metadata,
                constructor_plans,
            );
        }
        Stmt::If {
            test,
            consequent,
            alternate,
            ..
        } => {
            collect_class_lowering_from_expr(ast, *test, function_metadata, constructor_plans);
            collect_class_lowering_from_stmt(
                ast,
                *consequent,
                function_metadata,
                constructor_plans,
            );
            if let Some(alternate) = alternate {
                collect_class_lowering_from_stmt(
                    ast,
                    *alternate,
                    function_metadata,
                    constructor_plans,
                );
            }
        }
        Stmt::DoWhile { body, test, .. } | Stmt::While { test, body, .. } => {
            collect_class_lowering_from_stmt(ast, *body, function_metadata, constructor_plans);
            collect_class_lowering_from_expr(ast, *test, function_metadata, constructor_plans);
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
                        collect_class_lowering_from_decl(
                            ast,
                            *decl,
                            function_metadata,
                            constructor_plans,
                        );
                    }
                    ForInit::Expression(expr) => {
                        collect_class_lowering_from_expr(
                            ast,
                            *expr,
                            function_metadata,
                            constructor_plans,
                        );
                    }
                }
            }
            if let Some(test) = test {
                collect_class_lowering_from_expr(ast, *test, function_metadata, constructor_plans);
            }
            if let Some(update) = update {
                collect_class_lowering_from_expr(
                    ast,
                    *update,
                    function_metadata,
                    constructor_plans,
                );
            }
            collect_class_lowering_from_stmt(ast, *body, function_metadata, constructor_plans);
        }
        Stmt::ForIn {
            left, right, body, ..
        }
        | Stmt::ForOf {
            left, right, body, ..
        } => {
            collect_class_lowering_from_for_left(ast, *left, function_metadata, constructor_plans);
            collect_class_lowering_from_expr(ast, *right, function_metadata, constructor_plans);
            collect_class_lowering_from_stmt(ast, *body, function_metadata, constructor_plans);
        }
        Stmt::Return { argument, .. } => {
            if let Some(argument) = argument {
                collect_class_lowering_from_expr(
                    ast,
                    *argument,
                    function_metadata,
                    constructor_plans,
                );
            }
        }
        Stmt::With { object, body, .. } => {
            collect_class_lowering_from_expr(ast, *object, function_metadata, constructor_plans);
            collect_class_lowering_from_stmt(ast, *body, function_metadata, constructor_plans);
        }
        Stmt::Switch {
            discriminant,
            cases,
            ..
        } => {
            collect_class_lowering_from_expr(
                ast,
                *discriminant,
                function_metadata,
                constructor_plans,
            );
            for case in ast.get_switch_case_list(*cases) {
                if let Some(test) = case.test {
                    collect_class_lowering_from_expr(
                        ast,
                        test,
                        function_metadata,
                        constructor_plans,
                    );
                }
                collect_class_lowering_from_stmt_list(
                    ast,
                    case.consequent,
                    function_metadata,
                    constructor_plans,
                );
            }
        }
        Stmt::Labeled { body, .. } => {
            collect_class_lowering_from_stmt(ast, *body, function_metadata, constructor_plans);
        }
        Stmt::Throw { argument, .. } => {
            collect_class_lowering_from_expr(ast, *argument, function_metadata, constructor_plans);
        }
        Stmt::Try {
            block,
            handler,
            finalizer,
            ..
        } => {
            collect_class_lowering_from_stmt(ast, *block, function_metadata, constructor_plans);
            if let Some(handler) = handler {
                if let Some(param) = handler.param {
                    collect_class_lowering_from_pattern(
                        ast,
                        param,
                        function_metadata,
                        constructor_plans,
                    );
                }
                collect_class_lowering_from_stmt(
                    ast,
                    handler.body,
                    function_metadata,
                    constructor_plans,
                );
            }
            if let Some(finalizer) = finalizer {
                collect_class_lowering_from_stmt(
                    ast,
                    *finalizer,
                    function_metadata,
                    constructor_plans,
                );
            }
        }
        Stmt::Declaration { decl, .. } => {
            collect_class_lowering_from_decl(ast, *decl, function_metadata, constructor_plans);
        }
    }
}

fn collect_class_lowering_from_for_left(
    ast: &lyng_js_ast::Ast,
    left: ForInOfLeft,
    function_metadata: &mut HashMap<FunctionId, ClassFunctionMetadata>,
    constructor_plans: &mut HashMap<FunctionId, ClassConstructorPlan>,
) {
    match left {
        ForInOfLeft::Declaration(decl) => {
            collect_class_lowering_from_decl(ast, decl, function_metadata, constructor_plans);
        }
        ForInOfLeft::Pattern(pattern) => {
            collect_class_lowering_from_pattern(ast, pattern, function_metadata, constructor_plans);
        }
        ForInOfLeft::Expression(expr) => {
            collect_class_lowering_from_expr(ast, expr, function_metadata, constructor_plans);
        }
    }
}

fn collect_class_lowering_from_decl(
    ast: &lyng_js_ast::Ast,
    decl_id: DeclId,
    function_metadata: &mut HashMap<FunctionId, ClassFunctionMetadata>,
    constructor_plans: &mut HashMap<FunctionId, ClassConstructorPlan>,
) {
    match ast.get_decl(decl_id) {
        Decl::Variable { declarators, .. } => {
            for declarator in ast.get_var_declarator_list(*declarators) {
                collect_class_lowering_from_pattern(
                    ast,
                    declarator.id,
                    function_metadata,
                    constructor_plans,
                );
                if let Some(init) = declarator.init {
                    collect_class_lowering_from_expr(
                        ast,
                        init,
                        function_metadata,
                        constructor_plans,
                    );
                }
            }
        }
        Decl::Function { function, .. } => {
            collect_class_lowering_from_function(
                ast,
                *function,
                function_metadata,
                constructor_plans,
            );
        }
        Decl::Class {
            span,
            super_class,
            body,
            ..
        } => {
            if let Some(super_class) = super_class {
                collect_class_lowering_from_expr(
                    ast,
                    *super_class,
                    function_metadata,
                    constructor_plans,
                );
            }
            collect_class_lowering_from_class_body(
                ast,
                *body,
                super_class.is_some(),
                Some(*span),
                function_metadata,
                constructor_plans,
            );
        }
        Decl::Export { kind, .. } => match kind {
            lyng_js_ast::ExportKind::Default { declaration } => match declaration {
                lyng_js_ast::ExportDefaultDecl::Function(function) => {
                    collect_class_lowering_from_function(
                        ast,
                        *function,
                        function_metadata,
                        constructor_plans,
                    );
                }
                lyng_js_ast::ExportDefaultDecl::Class(decl) => {
                    collect_class_lowering_from_decl(
                        ast,
                        *decl,
                        function_metadata,
                        constructor_plans,
                    );
                }
                lyng_js_ast::ExportDefaultDecl::Expression(expr) => {
                    collect_class_lowering_from_expr(
                        ast,
                        *expr,
                        function_metadata,
                        constructor_plans,
                    );
                }
            },
            lyng_js_ast::ExportKind::Declaration { decl } => {
                collect_class_lowering_from_decl(ast, *decl, function_metadata, constructor_plans);
            }
            lyng_js_ast::ExportKind::Named { .. } | lyng_js_ast::ExportKind::All { .. } => {}
        },
        Decl::Import { .. } | Decl::InvalidDeclaration { .. } => {}
    }
}

fn collect_class_lowering_from_pattern(
    ast: &lyng_js_ast::Ast,
    pattern_id: lyng_js_ast::PatternId,
    function_metadata: &mut HashMap<FunctionId, ClassFunctionMetadata>,
    constructor_plans: &mut HashMap<FunctionId, ClassConstructorPlan>,
) {
    match ast.get_pattern(pattern_id) {
        Pattern::Identifier { .. } | Pattern::InvalidPattern { .. } => {}
        Pattern::Object {
            properties, rest, ..
        } => {
            for property in ast.get_obj_pattern_prop_list(*properties) {
                if property.computed {
                    collect_class_lowering_from_expr(
                        ast,
                        property.key,
                        function_metadata,
                        constructor_plans,
                    );
                }
                collect_class_lowering_from_pattern(
                    ast,
                    property.value,
                    function_metadata,
                    constructor_plans,
                );
            }
            if let Some(rest) = rest {
                collect_class_lowering_from_pattern(
                    ast,
                    *rest,
                    function_metadata,
                    constructor_plans,
                );
            }
        }
        Pattern::Array { elements, rest, .. } => {
            for element in ast.get_opt_pattern_elem_list(*elements).iter().flatten() {
                collect_class_lowering_from_pattern(
                    ast,
                    element.pattern,
                    function_metadata,
                    constructor_plans,
                );
            }
            if let Some(rest) = rest {
                collect_class_lowering_from_pattern(
                    ast,
                    *rest,
                    function_metadata,
                    constructor_plans,
                );
            }
        }
        Pattern::Assignment { left, right, .. } => {
            collect_class_lowering_from_pattern(ast, *left, function_metadata, constructor_plans);
            collect_class_lowering_from_expr(ast, *right, function_metadata, constructor_plans);
        }
    }
}

fn collect_class_lowering_from_expr(
    ast: &lyng_js_ast::Ast,
    expr_id: ExprId,
    function_metadata: &mut HashMap<FunctionId, ClassFunctionMetadata>,
    constructor_plans: &mut HashMap<FunctionId, ClassConstructorPlan>,
) {
    match ast.get_expr(expr_id) {
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
            for element in ast.get_opt_expr_list(*elements).iter().flatten() {
                collect_class_lowering_from_expr(
                    ast,
                    *element,
                    function_metadata,
                    constructor_plans,
                );
            }
        }
        Expr::ObjectExpression { properties, .. } => {
            for property in ast.get_property_list(*properties) {
                if property.computed {
                    collect_class_lowering_from_expr(
                        ast,
                        property.key,
                        function_metadata,
                        constructor_plans,
                    );
                }
                if property.method || !matches!(property.kind, lyng_js_ast::PropertyKind::Init) {
                    match ast.get_expr(property.value) {
                        Expr::FunctionExpression { function, .. }
                        | Expr::ArrowFunctionExpression { function, .. } => {
                            function_metadata.insert(
                                *function,
                                ClassFunctionMetadata {
                                    constructible: false,
                                    has_prototype_property: false,
                                    class_constructor: false,
                                    derived_class_constructor: false,
                                    class_source_span: None,
                                    class_body: None,
                                },
                            );
                        }
                        _ => {}
                    }
                }
                collect_class_lowering_from_expr(
                    ast,
                    property.value,
                    function_metadata,
                    constructor_plans,
                );
            }
        }
        Expr::FunctionExpression { function, .. }
        | Expr::ArrowFunctionExpression { function, .. } => {
            collect_class_lowering_from_function(
                ast,
                *function,
                function_metadata,
                constructor_plans,
            );
        }
        Expr::ClassExpression {
            span,
            super_class,
            body,
            ..
        } => {
            if let Some(super_class) = super_class {
                collect_class_lowering_from_expr(
                    ast,
                    *super_class,
                    function_metadata,
                    constructor_plans,
                );
            }
            collect_class_lowering_from_class_body(
                ast,
                *body,
                super_class.is_some(),
                Some(*span),
                function_metadata,
                constructor_plans,
            );
        }
        Expr::TemplateLiteral { template, .. } => {
            for &expression in ast.templates().get_expressions(*template) {
                collect_class_lowering_from_expr(
                    ast,
                    expression,
                    function_metadata,
                    constructor_plans,
                );
            }
        }
        Expr::TaggedTemplateExpression { tag, template, .. } => {
            collect_class_lowering_from_expr(ast, *tag, function_metadata, constructor_plans);
            for &expression in ast.templates().get_expressions(*template) {
                collect_class_lowering_from_expr(
                    ast,
                    expression,
                    function_metadata,
                    constructor_plans,
                );
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
        } => collect_class_lowering_from_expr(ast, *argument, function_metadata, constructor_plans),
        Expr::BinaryExpression { left, right, .. }
        | Expr::LogicalExpression { left, right, .. }
        | Expr::AssignmentExpression { left, right, .. } => {
            collect_class_lowering_from_expr(ast, *left, function_metadata, constructor_plans);
            collect_class_lowering_from_expr(ast, *right, function_metadata, constructor_plans);
        }
        Expr::ConditionalExpression {
            test,
            consequent,
            alternate,
            ..
        } => {
            collect_class_lowering_from_expr(ast, *test, function_metadata, constructor_plans);
            collect_class_lowering_from_expr(
                ast,
                *consequent,
                function_metadata,
                constructor_plans,
            );
            collect_class_lowering_from_expr(ast, *alternate, function_metadata, constructor_plans);
        }
        Expr::SequenceExpression { expressions, .. } => {
            for &expression in ast.get_expr_list(*expressions) {
                collect_class_lowering_from_expr(
                    ast,
                    expression,
                    function_metadata,
                    constructor_plans,
                );
            }
        }
        Expr::CallExpression {
            callee, arguments, ..
        }
        | Expr::NewExpression {
            callee, arguments, ..
        } => {
            collect_class_lowering_from_expr(ast, *callee, function_metadata, constructor_plans);
            for &argument in ast.get_expr_list(*arguments) {
                collect_class_lowering_from_expr(
                    ast,
                    argument,
                    function_metadata,
                    constructor_plans,
                );
            }
        }
        Expr::StaticMemberExpression { object, .. }
        | Expr::PrivateMemberExpression { object, .. } => {
            collect_class_lowering_from_expr(ast, *object, function_metadata, constructor_plans);
        }
        Expr::ComputedMemberExpression {
            object, property, ..
        } => {
            collect_class_lowering_from_expr(ast, *object, function_metadata, constructor_plans);
            collect_class_lowering_from_expr(ast, *property, function_metadata, constructor_plans);
        }
        Expr::PrivateInExpression { object, .. } => {
            collect_class_lowering_from_expr(ast, *object, function_metadata, constructor_plans);
        }
        Expr::YieldExpression { argument, .. } => {
            if let Some(argument) = argument {
                collect_class_lowering_from_expr(
                    ast,
                    *argument,
                    function_metadata,
                    constructor_plans,
                );
            }
        }
        Expr::ImportExpression {
            source, options, ..
        } => {
            collect_class_lowering_from_expr(ast, *source, function_metadata, constructor_plans);
            if let Some(options) = options {
                collect_class_lowering_from_expr(
                    ast,
                    *options,
                    function_metadata,
                    constructor_plans,
                );
            }
        }
    }
}

fn collect_class_lowering_from_function(
    ast: &lyng_js_ast::Ast,
    function_id: FunctionId,
    function_metadata: &mut HashMap<FunctionId, ClassFunctionMetadata>,
    constructor_plans: &mut HashMap<FunctionId, ClassConstructorPlan>,
) {
    let function = ast.get_function(function_id);
    for &parameter in ast.get_pattern_list(function.params.params) {
        collect_class_lowering_from_pattern(ast, parameter, function_metadata, constructor_plans);
    }
    if let Some(rest) = function.params.rest {
        collect_class_lowering_from_pattern(ast, rest, function_metadata, constructor_plans);
    }
    collect_class_lowering_from_stmt_list(ast, function.body, function_metadata, constructor_plans);
    if let Some(expression) = function.expression_body {
        collect_class_lowering_from_expr(ast, expression, function_metadata, constructor_plans);
    }
}

fn collect_class_lowering_from_class_body(
    ast: &lyng_js_ast::Ast,
    body: lyng_js_ast::NodeList<lyng_js_ast::ClassElementId>,
    has_heritage: bool,
    class_source_span: Option<lyng_js_common::Span>,
    function_metadata: &mut HashMap<FunctionId, ClassFunctionMetadata>,
    constructor_plans: &mut HashMap<FunctionId, ClassConstructorPlan>,
) {
    let mut constructor = None;
    let mut instance_elements = Vec::new();
    let mut next_computed_instance_field_key = 0u32;

    for &element in ast.get_class_element_list(body) {
        match ast.get_class_element(element) {
            lyng_js_ast::ClassElement::Method {
                key,
                value,
                computed,
                kind,
                private,
                r#static,
                ..
            } => {
                if *computed {
                    collect_class_lowering_from_expr(
                        ast,
                        *key,
                        function_metadata,
                        constructor_plans,
                    );
                }
                function_metadata.insert(
                    *value,
                    ClassFunctionMetadata {
                        constructible: matches!(kind, lyng_js_ast::MethodKind::Constructor),
                        has_prototype_property: matches!(
                            kind,
                            lyng_js_ast::MethodKind::Constructor
                        ),
                        class_constructor: matches!(kind, lyng_js_ast::MethodKind::Constructor),
                        derived_class_constructor: matches!(
                            kind,
                            lyng_js_ast::MethodKind::Constructor
                        ) && has_heritage,
                        class_source_span: matches!(kind, lyng_js_ast::MethodKind::Constructor)
                            .then_some(())
                            .and(class_source_span),
                        class_body: Some(body),
                    },
                );
                if matches!(kind, lyng_js_ast::MethodKind::Constructor) {
                    constructor = Some(*value);
                }
                if !*r#static && *private {
                    if let lyng_js_ast::Expr::Identifier { name, .. } = ast.get_expr(*key) {
                        let element_kind = match kind {
                            lyng_js_ast::MethodKind::Method
                            | lyng_js_ast::MethodKind::Constructor => {
                                lyng_js_sema::ClassPrivateElementKind::Method
                            }
                            lyng_js_ast::MethodKind::Get => {
                                lyng_js_sema::ClassPrivateElementKind::Getter
                            }
                            lyng_js_ast::MethodKind::Set => {
                                lyng_js_sema::ClassPrivateElementKind::Setter
                            }
                        };
                        instance_elements.push(ClassInstanceElementPlan::PrivateElement {
                            name: *name,
                            kind: element_kind,
                            value: None,
                        });
                    }
                }
                collect_class_lowering_from_function(
                    ast,
                    *value,
                    function_metadata,
                    constructor_plans,
                );
            }
            lyng_js_ast::ClassElement::Property {
                key,
                value,
                computed,
                private,
                r#static,
                ..
            } => {
                if *computed {
                    collect_class_lowering_from_expr(
                        ast,
                        *key,
                        function_metadata,
                        constructor_plans,
                    );
                }
                if let Some(value) = value {
                    collect_class_lowering_from_expr(
                        ast,
                        *value,
                        function_metadata,
                        constructor_plans,
                    );
                }
                if !*r#static {
                    if *private {
                        if let lyng_js_ast::Expr::Identifier { name, .. } = ast.get_expr(*key) {
                            instance_elements.push(ClassInstanceElementPlan::PrivateElement {
                                name: *name,
                                kind: lyng_js_sema::ClassPrivateElementKind::Field,
                                value: *value,
                            });
                        }
                    } else {
                        instance_elements.push(ClassInstanceElementPlan::PublicField {
                            key: *key,
                            value: *value,
                            computed: *computed,
                            computed_key_index: if *computed {
                                let index = next_computed_instance_field_key;
                                next_computed_instance_field_key =
                                    next_computed_instance_field_key.saturating_add(1);
                                Some(index)
                            } else {
                                None
                            },
                        });
                    }
                }
            }
            lyng_js_ast::ClassElement::StaticBlock { body, .. } => {
                collect_class_lowering_from_stmt_list(
                    ast,
                    *body,
                    function_metadata,
                    constructor_plans,
                );
            }
            lyng_js_ast::ClassElement::InvalidElement { .. } => {}
        }
    }

    if let Some(constructor) = constructor {
        constructor_plans.insert(
            constructor,
            ClassConstructorPlan {
                derived: has_heritage,
                class_body: body,
                needs_environment: instance_elements_need_environment(ast, &instance_elements),
                instance_elements,
            },
        );
    }
}

fn instance_elements_need_environment(
    ast: &lyng_js_ast::Ast,
    instance_elements: &[ClassInstanceElementPlan],
) -> bool {
    instance_elements.iter().any(|element| match element {
        ClassInstanceElementPlan::PublicField { value, .. }
        | ClassInstanceElementPlan::PrivateElement { value, .. } => value.is_some_and(|value| {
            matches!(ast.get_expr(value), Expr::ArrowFunctionExpression { .. })
        }),
    })
}

pub(super) struct FunctionCompiler<'a, 'b> {
    pub(super) state: &'b mut CompilationState<'a>,
    pub(super) builder: BytecodeBuilder,
    pub(super) current_function: Option<FunctionSemaId>,
    pub(super) current_function_ast: Option<FunctionId>,
    pub(super) current_scope: ScopeId,
    pub(super) body_scope: ScopeId,
    pub(super) scope_child_cursors: Vec<usize>,
    pub(super) local_registers: Vec<Option<u16>>,
    pub(super) atom_constants: HashMap<AtomId, u32>,
    pub(super) float_constants: HashMap<u64, u32>,
    pub(super) builtin_constants: HashMap<BuiltinFunctionId, u32>,
    pub(super) child_indices: HashMap<FunctionId, u16>,
    pub(super) hoisted_function_decls: HashSet<DeclId>,
    pub(super) hoisted_default_export_functions: HashSet<FunctionId>,
    pub(super) parameter_sources: Vec<ParameterSource>,
    pub(super) result_register: Option<u16>,
    pub(super) call_bridge_registers: Option<CallBridgeRegisters>,
    pub(super) generator_resume_registers: Option<GeneratorResumeRegisters>,
    pub(super) completion_registers: Option<CompletionRegisters>,
    pub(super) control_targets: Vec<ControlTarget>,
    pub(super) next_control_target_id: u16,
    pub(super) finally_stack: Vec<FinallyContext>,
    pub(super) this_override_register: Option<u16>,
    pub(super) super_home_object_override: Option<u16>,
    pub(super) active_class_body: Option<lyng_js_ast::NodeList<lyng_js_ast::ClassElementId>>,
    pub(super) active_class_span: Option<Span>,
    pub(super) active_class_contexts: Vec<ActiveClassContext>,
    pub(super) active_direct_eval_scopes: Vec<ScopeId>,
    pub(super) in_class_field_initializer: bool,
    pub(super) active_disposal_scopes: Vec<ActiveDisposalScope>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CallBridgeRegisters {
    pub(super) result: u16,
    pub(super) callee: u16,
    pub(super) this_value: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GeneratorResumeRegisters {
    pub(super) kind: u16,
    pub(super) value: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ActiveClassContext {
    pub(super) class_object: u16,
    pub(super) prototype: u16,
    pub(super) has_private_entries: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ActiveDisposalScope {
    pub(super) capability_register: u16,
    pub(super) kind: lyng_js_env::DisposalCapabilityKind,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct LoweredCallArguments {
    pub(super) registers: Vec<u16>,
    pub(super) spread_mask: u64,
}

fn scope_environment_bindings(
    sema: ProgramSemaView<'_>,
    scope: &lyng_js_sema::ScopeRecord,
) -> Vec<BytecodeEnvironmentBinding> {
    let slot_count = scope
        .bindings
        .iter()
        .filter_map(|binding_id| {
            let binding = sema.binding_table.get(*binding_id);
            ((binding.storage_class == StorageClass::EnvironmentSlot)
                || (binding.storage_class == StorageClass::DynamicLookup
                    && binding.slot_index.is_some()))
            .then(|| {
                binding
                    .slot_index
                    .expect("environment-stored bindings should carry slot indices")
            })
        })
        .max()
        .map_or(0usize, |slot| {
            usize::try_from(slot.saturating_add(1)).unwrap_or(usize::MAX)
        });
    if slot_count == 0 {
        return Vec::new();
    }

    let mut bindings = vec![None; slot_count];
    for binding_id in &scope.bindings {
        let binding = sema.binding_table.get(*binding_id);
        if binding.storage_class != StorageClass::EnvironmentSlot
            && !(binding.storage_class == StorageClass::DynamicLookup
                && binding.slot_index.is_some())
        {
            continue;
        }
        let slot = usize::try_from(
            binding
                .slot_index
                .expect("environment-stored bindings should carry slot indices"),
        )
        .unwrap_or(usize::MAX);
        let Some(slot_binding) = bindings.get_mut(slot) else {
            continue;
        };
        *slot_binding = Some(bytecode_environment_binding(binding));
    }

    assert!(
        bindings.iter().all(|binding| binding.is_some()),
        "environment-stored scope bindings should be densely indexed"
    );
    bindings
        .into_iter()
        .map(|binding| binding.expect("dense scope bindings should populate every slot"))
        .collect()
}

fn synthetic_function_environment_bindings(
    sema: ProgramSemaView<'_>,
    activation: &FunctionActivationPlan,
) -> Vec<BytecodeEnvironmentBinding> {
    let mut bindings = Vec::with_capacity(usize::from(activation.synthetic_prefix_slots()));
    if activation.arguments_mode == ArgumentsMode::Mapped {
        let mut parameter_bindings = vec![None; activation.parameter_ordinals.len()];
        for (binding_id, ordinal) in &activation.parameter_ordinals {
            let binding = sema.binding_table.get(*binding_id);
            parameter_bindings[usize::from(*ordinal)] = Some(bytecode_environment_binding(binding));
        }
        assert!(
            parameter_bindings.iter().all(|binding| binding.is_some()),
            "mapped parameter slots should be densely ordered"
        );
        bindings.extend(
            parameter_bindings.into_iter().map(|binding| {
                binding.expect("mapped parameter slots should populate every ordinal")
            }),
        );
        bindings.push(BytecodeEnvironmentBinding::new(
            Some(WellKnownAtom::arguments.id()),
            BytecodeEnvironmentSlotFlags::var_like(),
        ));
        return bindings;
    }

    if let Some(rest_binding) = activation.rest_binding {
        bindings.push(bytecode_environment_binding(
            sema.binding_table.get(rest_binding),
        ));
    } else if activation.has_rest_parameter {
        bindings.push(BytecodeEnvironmentBinding::new(
            None,
            BytecodeEnvironmentSlotFlags::var_like(),
        ));
    }

    if activation.needs_arguments_object() {
        bindings.push(BytecodeEnvironmentBinding::new(
            Some(WellKnownAtom::arguments.id()),
            BytecodeEnvironmentSlotFlags::var_like(),
        ));
    }

    bindings
}

fn bytecode_environment_binding(
    binding: &lyng_js_sema::BindingRecord,
) -> BytecodeEnvironmentBinding {
    let flags = BytecodeEnvironmentSlotFlags::new(
        binding_is_mutable(binding.kind),
        binding.kind.is_lexical(),
        binding.has_tdz,
        matches!(binding.storage_class, StorageClass::DynamicLookup),
    )
    .with_sloppy_immutable_assign_silent(matches!(binding.kind, DeclarationKind::FunctionName));
    BytecodeEnvironmentBinding::new(Some(binding.name), flags)
}

fn binding_is_mutable(kind: DeclarationKind) -> bool {
    !matches!(
        kind,
        DeclarationKind::Const
            | DeclarationKind::Using
            | DeclarationKind::AwaitUsing
            | DeclarationKind::Import
            | DeclarationKind::FunctionName
            | DeclarationKind::ClassName
    )
}

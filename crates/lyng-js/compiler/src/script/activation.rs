use super::*;

#[derive(Clone, Debug)]
pub(super) struct FunctionActivationPlan {
    pub(super) arguments_mode: ArgumentsMode,
    pub(super) has_rest_parameter: bool,
    pub(super) parameter_ordinals: HashMap<SemanticBindingId, u16>,
    pub(super) rest_binding: Option<SemanticBindingId>,
    pub(super) needs_environment: bool,
}

impl FunctionActivationPlan {
    pub(super) fn needs_arguments_object(&self) -> bool {
        self.arguments_mode != ArgumentsMode::None
    }

    pub(super) fn synthetic_prefix_slots(&self) -> u16 {
        match self.arguments_mode {
            ArgumentsMode::Mapped => self
                .parameter_ordinals
                .len()
                .try_into()
                .unwrap_or(u16::MAX)
                .saturating_add(1),
            ArgumentsMode::None | ArgumentsMode::Unmapped => u16::from(self.has_rest_parameter)
                .saturating_add(u16::from(self.needs_arguments_object())),
        }
    }

    pub(super) fn arguments_slot(&self) -> Option<u16> {
        if !self.needs_arguments_object() {
            return None;
        }
        Some(match self.arguments_mode {
            ArgumentsMode::Mapped => self.parameter_ordinals.len().try_into().unwrap_or(u16::MAX),
            ArgumentsMode::None | ArgumentsMode::Unmapped => u16::from(self.has_rest_parameter),
        })
    }

    pub(super) fn rest_slot(&self) -> Option<u16> {
        self.has_rest_parameter.then_some(0)
    }

    pub(super) fn runtime_slot_for_binding(
        &self,
        binding: SemanticBindingId,
        sema_slot: Option<u32>,
    ) -> Option<u32> {
        if self.arguments_mode == ArgumentsMode::Mapped {
            if let Some(ordinal) = self.parameter_ordinals.get(&binding) {
                return Some(u32::from(*ordinal));
            }
        }
        if self.rest_binding == Some(binding) {
            return self.rest_slot().map(u32::from);
        }
        sema_slot.map(|slot| slot + u32::from(self.synthetic_prefix_slots()))
    }
}

pub(super) fn parent_function_for(
    scopes: &lyng_js_sema::ScopeTable,
    scope_root: ScopeId,
    current: FunctionSemaId,
) -> Option<FunctionSemaId> {
    let mut cursor = scopes.get(scope_root).parent;
    while let Some(scope) = cursor {
        let owner = scopes.get(scope).owning_function;
        if owner != Some(current) && owner.is_some() {
            return owner;
        }
        cursor = scopes.get(scope).parent;
    }
    None
}

fn is_simple_parameter_pattern(ast: &lyng_js_ast::Ast, pattern: lyng_js_ast::PatternId) -> bool {
    matches!(ast.get_pattern(pattern), Pattern::Identifier { .. })
}

fn function_has_non_simple_params(
    ast: &lyng_js_ast::Ast,
    function: &lyng_js_ast::Function,
) -> bool {
    function.params.rest.is_some()
        || ast
            .get_pattern_list(function.params.params)
            .iter()
            .any(|pattern| !is_simple_parameter_pattern(ast, *pattern))
}

fn nearest_non_arrow_owner_for(
    program: ProgramSource<'_>,
    sema: ProgramSemaView<'_>,
    parent_functions: &[Option<FunctionSemaId>],
    function: FunctionSemaId,
) -> Option<FunctionSemaId> {
    let mut current = Some(function);
    while let Some(candidate) = current {
        let ast_function = sema.function_table.get(candidate).function_id;
        if !matches!(
            program.ast.get_function(ast_function).kind,
            FunctionKind::Arrow | FunctionKind::AsyncArrow
        ) {
            return Some(candidate);
        }
        current = parent_functions[candidate.raw() as usize];
    }
    None
}

fn has_direct_arrow_child_for(
    program: ProgramSource<'_>,
    sema: ProgramSemaView<'_>,
    parent_functions: &[Option<FunctionSemaId>],
    parent: FunctionSemaId,
) -> bool {
    sema.function_table
        .as_slice()
        .iter()
        .enumerate()
        .any(|(index, record)| {
            parent_functions[index] == Some(parent)
                && matches!(
                    program.ast.get_function(record.function_id).kind,
                    FunctionKind::Arrow | FunctionKind::AsyncArrow
                )
        })
}

fn resolved_arguments_binding_shadows_owner(
    program: ProgramSource<'_>,
    sema: ProgramSemaView<'_>,
    parent_functions: &[Option<FunctionSemaId>],
    binding: SemanticBindingId,
    owner: FunctionSemaId,
) -> bool {
    let binding = sema.binding_table.get(binding);
    if binding.kind == DeclarationKind::Var {
        return false;
    }
    let binding_owner = sema.scope_table.get(binding.scope).owning_function;
    binding_owner.and_then(|binding_owner| {
        nearest_non_arrow_owner_for(program, sema, parent_functions, binding_owner)
    }) == Some(owner)
}

pub(super) fn collect_arguments_owners(
    program: ProgramSource<'_>,
    sema: ProgramSemaView<'_>,
    parent_functions: &[Option<FunctionSemaId>],
) -> HashSet<FunctionSemaId> {
    sema.use_sites
        .as_slice()
        .iter()
        .filter_map(|record| {
            if record.name != WellKnownAtom::arguments.id() {
                return None;
            }
            let owner = sema.scope_table.get(record.scope).owning_function?;
            let owner = nearest_non_arrow_owner_for(program, sema, parent_functions, owner)?;
            if record.resolved_binding.is_some_and(|binding| {
                resolved_arguments_binding_shadows_owner(
                    program,
                    sema,
                    parent_functions,
                    binding,
                    owner,
                )
            }) {
                return None;
            }
            Some(owner)
        })
        .collect()
}

fn find_parameter_binding_for_plan(
    sema: ProgramSemaView<'_>,
    owner: FunctionSemaId,
    name: AtomId,
    used: &HashSet<SemanticBindingId>,
) -> Option<SemanticBindingId> {
    sema.binding_table
        .as_slice()
        .iter()
        .enumerate()
        .find_map(|(index, binding)| {
            let id = SemanticBindingId::new(index as u32);
            (binding.kind == DeclarationKind::Parameter
                && binding.name == name
                && sema.scope_table.get(binding.scope).owning_function == Some(owner)
                && !used.contains(&id))
            .then_some(id)
        })
}

pub(super) fn build_function_activation_plan(
    program: ProgramSource<'_>,
    sema: ProgramSemaView<'_>,
    sema_id: FunctionSemaId,
    record: &lyng_js_sema::FunctionSemaRecord,
    arguments_owners: &HashSet<FunctionSemaId>,
    parent_functions: &[Option<FunctionSemaId>],
) -> LoweringResult<FunctionActivationPlan> {
    let ast_function = program.ast.get_function(record.function_id).clone();
    let arguments_mode = if matches!(
        ast_function.kind,
        FunctionKind::Arrow | FunctionKind::AsyncArrow
    ) || !arguments_owners.contains(&sema_id)
    {
        ArgumentsMode::None
    } else if record.strict || function_has_non_simple_params(program.ast, &ast_function) {
        ArgumentsMode::Unmapped
    } else {
        ArgumentsMode::Mapped
    };

    let mut parameter_ordinals = HashMap::new();
    let mut used_bindings = HashSet::new();
    for (ordinal, pattern) in program
        .ast
        .get_pattern_list(ast_function.params.params)
        .iter()
        .copied()
        .enumerate()
    {
        let Pattern::Identifier { name, .. } = program.ast.get_pattern(pattern).clone() else {
            continue;
        };
        if let Some(binding) = find_parameter_binding_for_plan(sema, sema_id, name, &used_bindings)
        {
            used_bindings.insert(binding);
            parameter_ordinals.insert(binding, u16::try_from(ordinal).unwrap_or(u16::MAX));
        }
    }

    let rest_binding = ast_function.params.rest.and_then(|pattern| {
        let Pattern::Identifier { name, .. } = program.ast.get_pattern(pattern).clone() else {
            return None;
        };
        find_parameter_binding_for_plan(sema, sema_id, name, &used_bindings)
    });

    Ok(FunctionActivationPlan {
        arguments_mode,
        has_rest_parameter: ast_function.params.rest.is_some(),
        parameter_ordinals,
        rest_binding,
        needs_environment: record.needs_environment
            || record.has_eval
            || record.has_with
            || arguments_mode != ArgumentsMode::None
            || ast_function.params.rest.is_some()
            || has_direct_arrow_child_for(program, sema, parent_functions, sema_id),
    })
}

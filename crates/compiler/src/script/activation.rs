use super::{
    ArgumentsMode, AtomId, DeclarationKind, FunctionKind, FunctionSemaId, HashMap, HashSet,
    Pattern, ProgramSemaView, ProgramSource, ScopeId, SemanticBindingId, WellKnownAtom,
    checked_u32_index,
};

#[derive(Clone, Debug)]
pub(super) struct FunctionActivationPlan {
    pub(super) arguments_mode: ArgumentsMode,
    pub(super) has_rest_parameter: bool,
    pub(super) has_parameter_expressions: bool,
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
        if self.arguments_mode == ArgumentsMode::Mapped
            && let Some(ordinal) = self.parameter_ordinals.get(&binding)
        {
            return Some(u32::from(*ordinal));
        }
        if self.rest_binding == Some(binding) {
            return self.rest_slot().map(u32::from);
        }
        sema_slot.map(|slot| slot + u32::from(self.synthetic_prefix_slots()))
    }
}

pub(super) fn parent_function_for(
    scopes: &lyng_sema::ScopeTable,
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

fn is_simple_parameter_pattern(ast: &lyng_ast::Ast, pattern: lyng_ast::PatternId) -> bool {
    matches!(ast.get_pattern(pattern), Pattern::Identifier { .. })
}

fn function_has_non_simple_params(ast: &lyng_ast::Ast, function: &lyng_ast::Function) -> bool {
    function.params.rest.is_some()
        || ast
            .get_pattern_list(function.params.params)
            .iter()
            .any(|pattern| !is_simple_parameter_pattern(ast, *pattern))
}

fn pattern_contains_initializer(ast: &lyng_ast::Ast, pattern: lyng_ast::PatternId) -> bool {
    match ast.get_pattern(pattern) {
        Pattern::Assignment { .. } => true,
        Pattern::Object {
            properties, rest, ..
        } => {
            ast.get_obj_pattern_prop_list(*properties)
                .iter()
                .any(|property| pattern_contains_initializer(ast, property.value))
                || rest.is_some_and(|rest| pattern_contains_initializer(ast, rest))
        }
        Pattern::Array { elements, rest, .. } => {
            ast.get_opt_pattern_elem_list(*elements)
                .iter()
                .flatten()
                .any(|element| pattern_contains_initializer(ast, element.pattern))
                || rest.is_some_and(|rest| pattern_contains_initializer(ast, rest))
        }
        Pattern::Identifier { .. } | Pattern::InvalidPattern { .. } => false,
    }
}

fn function_has_parameter_expressions(ast: &lyng_ast::Ast, function: &lyng_ast::Function) -> bool {
    ast.get_pattern_list(function.params.params)
        .iter()
        .any(|&pattern| pattern_contains_initializer(ast, pattern))
        || function
            .params
            .rest
            .is_some_and(|rest| pattern_contains_initializer(ast, rest))
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

/// Set of functions that have at least one direct arrow-function child.
///
/// Built once per program (one pass over the function table) so
/// `build_function_activation_plan` can answer "does this function have a
/// direct arrow child?" in O(1); the previous per-function `.any()` scan made
/// activation-plan construction O(N^2) in the function count.
pub(super) fn collect_arrow_child_parents(
    program: ProgramSource<'_>,
    sema: ProgramSemaView<'_>,
    parent_functions: &[Option<FunctionSemaId>],
) -> HashSet<FunctionSemaId> {
    let mut parents = HashSet::new();
    for (index, record) in sema.function_table.as_slice().iter().enumerate() {
        if matches!(
            program.ast.get_function(record.function_id).kind,
            FunctionKind::Arrow | FunctionKind::AsyncArrow
        ) && let Some(parent) = parent_functions[index]
        {
            parents.insert(parent);
        }
    }
    parents
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
    let mut owners = sema
        .use_sites
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
        .collect::<HashSet<_>>();

    for (index, record) in sema.function_table.as_slice().iter().enumerate() {
        if !record.has_eval {
            continue;
        }
        let function = FunctionSemaId::new(checked_u32_index(index));
        if let Some(owner) = nearest_non_arrow_owner_for(program, sema, parent_functions, function)
        {
            owners.insert(owner);
        }
    }

    for (index, record) in sema.function_table.as_slice().iter().enumerate() {
        if !record.needs_arguments {
            continue;
        }
        let function = FunctionSemaId::new(checked_u32_index(index));
        if let Some(owner) = nearest_non_arrow_owner_for(program, sema, parent_functions, function)
        {
            owners.insert(owner);
        }
    }

    owners
}

/// Index of parameter bindings keyed by `(owning function, name)`.
///
/// Built once per program so `build_function_activation_plan` can resolve each
/// parameter in O(1) instead of scanning the whole binding table per parameter
/// (which made activation-plan construction O(N^2) in the program's binding
/// count). Binding ids are stored in ascending binding-table order, so picking
/// the first not-yet-used id reproduces the original linear scan's choice when
/// a function repeats a parameter name.
pub(super) struct ParameterBindingIndex {
    by_owner_name: HashMap<(FunctionSemaId, AtomId), Vec<SemanticBindingId>>,
}

impl ParameterBindingIndex {
    pub(super) fn build(sema: ProgramSemaView<'_>) -> Self {
        let mut by_owner_name: HashMap<(FunctionSemaId, AtomId), Vec<SemanticBindingId>> =
            HashMap::new();
        for (index, binding) in sema.binding_table.as_slice().iter().enumerate() {
            if binding.kind != DeclarationKind::Parameter {
                continue;
            }
            let Some(owner) = sema.scope_table.get(binding.scope).owning_function else {
                continue;
            };
            by_owner_name
                .entry((owner, binding.name))
                .or_default()
                .push(SemanticBindingId::new(checked_u32_index(index)));
        }
        Self { by_owner_name }
    }

    fn find(
        &self,
        owner: FunctionSemaId,
        name: AtomId,
        used: &HashSet<SemanticBindingId>,
    ) -> Option<SemanticBindingId> {
        self.by_owner_name
            .get(&(owner, name))?
            .iter()
            .copied()
            .find(|id| !used.contains(id))
    }
}

pub(super) fn build_function_activation_plan(
    program: ProgramSource<'_>,
    sema_id: FunctionSemaId,
    record: &lyng_sema::FunctionSemaRecord,
    arguments_owners: &HashSet<FunctionSemaId>,
    parameter_bindings: &ParameterBindingIndex,
    arrow_child_parents: &HashSet<FunctionSemaId>,
) -> FunctionActivationPlan {
    let ast_function = program.ast.get_function(record.function_id).clone();
    let has_parameter_expressions = function_has_parameter_expressions(program.ast, &ast_function);
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
        if let Some(binding) = parameter_bindings.find(sema_id, name, &used_bindings) {
            used_bindings.insert(binding);
            parameter_ordinals.insert(binding, u16::try_from(ordinal).unwrap_or(u16::MAX));
        }
    }

    let rest_binding = ast_function.params.rest.and_then(|pattern| {
        let Pattern::Identifier { name, .. } = program.ast.get_pattern(pattern).clone() else {
            return None;
        };
        parameter_bindings.find(sema_id, name, &used_bindings)
    });

    FunctionActivationPlan {
        arguments_mode,
        has_rest_parameter: ast_function.params.rest.is_some(),
        has_parameter_expressions,
        parameter_ordinals,
        rest_binding,
        needs_environment: record.needs_environment
            || record.has_eval
            || record.has_with
            || arguments_mode != ArgumentsMode::None
            || ast_function.params.rest.is_some()
            || arrow_child_parents.contains(&sema_id),
    }
}

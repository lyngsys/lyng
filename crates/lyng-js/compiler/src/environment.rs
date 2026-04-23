use lyng_js_common::AtomId;
use lyng_js_env::{
    Agent, EnvironmentBindingLayout, EnvironmentLayout, EnvironmentLayoutId, EnvironmentLayoutKind,
    EnvironmentSlotFlags,
};
use lyng_js_sema::{
    BindingRecord, BindingTable, DeclarationKind, FunctionSemaId, FunctionSemaRecord,
    FunctionSemaTable, ScopeId, ScopeKind, ScopeTable, SemanticBindingId, StorageClass,
};
use lyng_js_types::EnvironmentRef;

/// Errors raised while translating sema metadata into runtime environment layouts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvironmentLayoutPlanError {
    InvalidScopeReference {
        scope: ScopeId,
    },
    InvalidBindingReference {
        scope: ScopeId,
        binding: SemanticBindingId,
    },
    BindingScopeMismatch {
        scope: ScopeId,
        binding: SemanticBindingId,
        actual_scope: ScopeId,
    },
    MissingSlotIndex {
        scope: ScopeId,
        binding: SemanticBindingId,
    },
    UnexpectedSlotIndex {
        scope: ScopeId,
        binding: SemanticBindingId,
        expected: u32,
        actual: u32,
    },
}

pub type EnvironmentLayoutPlanResult<T> = Result<T, EnvironmentLayoutPlanError>;

/// Frozen runtime layout plan for one sema scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeEnvironmentLayoutPlan {
    layout: EnvironmentLayout,
    global_var_names: Vec<AtomId>,
}

impl ScopeEnvironmentLayoutPlan {
    #[inline]
    pub fn new(layout: EnvironmentLayout, global_var_names: Vec<AtomId>) -> Self {
        Self {
            layout,
            global_var_names,
        }
    }

    #[inline]
    pub const fn layout(&self) -> &EnvironmentLayout {
        &self.layout
    }

    #[inline]
    pub fn global_var_names(&self) -> &[AtomId] {
        &self.global_var_names
    }
}

/// Function-level layout metadata derived from sema scope ownership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FunctionEnvironmentLayoutPlan {
    scope_root: ScopeId,
    param_scope: Option<ScopeId>,
    needs_environment: bool,
}

impl FunctionEnvironmentLayoutPlan {
    #[inline]
    pub const fn new(
        scope_root: ScopeId,
        param_scope: Option<ScopeId>,
        needs_environment: bool,
    ) -> Self {
        Self {
            scope_root,
            param_scope,
            needs_environment,
        }
    }

    #[inline]
    pub const fn scope_root(self) -> ScopeId {
        self.scope_root
    }

    #[inline]
    pub const fn param_scope(self) -> Option<ScopeId> {
        self.param_scope
    }

    #[inline]
    pub const fn needs_environment(self) -> bool {
        self.needs_environment
    }
}

/// Compiler-owned bridge from sema tables to installable environment layouts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnvironmentLayoutPlan {
    scopes: Vec<Option<ScopeEnvironmentLayoutPlan>>,
    functions: Vec<FunctionEnvironmentLayoutPlan>,
}

impl EnvironmentLayoutPlan {
    #[inline]
    pub fn scope(&self, scope: ScopeId) -> Option<&ScopeEnvironmentLayoutPlan> {
        self.scopes.get(scope.raw() as usize)?.as_ref()
    }

    #[inline]
    pub fn function(&self, function: FunctionSemaId) -> Option<FunctionEnvironmentLayoutPlan> {
        self.functions.get(function.raw() as usize).copied()
    }
}

/// Runtime-installed layout ids keyed by sema scope id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InstalledEnvironmentLayouts {
    scopes: Vec<Option<EnvironmentLayoutId>>,
}

impl InstalledEnvironmentLayouts {
    #[inline]
    pub fn scope(&self, scope: ScopeId) -> Option<EnvironmentLayoutId> {
        self.scopes.get(scope.raw() as usize).copied().flatten()
    }
}

/// Derives a stable environment-layout plan from sema tables while preserving sema slot order.
pub fn derive_environment_layout_plan(
    scope_table: &ScopeTable,
    binding_table: &BindingTable,
    function_table: &FunctionSemaTable,
) -> EnvironmentLayoutPlanResult<EnvironmentLayoutPlan> {
    let scope_records = scope_table.as_slice();
    let binding_records = binding_table.as_slice();
    let function_records = function_table.as_slice();
    let functions = function_records
        .iter()
        .map(|record| {
            validate_scope_reference(scope_records, record.scope_root)?;
            if let Some(param_scope) = record.param_scope {
                validate_scope_reference(scope_records, param_scope)?;
            }
            Ok(FunctionEnvironmentLayoutPlan::new(
                record.scope_root,
                record.param_scope,
                record.needs_environment,
            ))
        })
        .collect::<EnvironmentLayoutPlanResult<Vec<_>>>()?;

    let mut scopes = vec![None; scope_records.len()];
    for (index, scope_record) in scope_records.iter().enumerate() {
        let scope_id = ScopeId::new(u32::try_from(index).expect("scope index must fit into u32"));
        let Some(layout_kind) = scope_layout_kind(scope_record.kind) else {
            continue;
        };

        let mut bindings = Vec::new();
        let mut global_var_names = Vec::new();
        for binding_id in &scope_record.bindings {
            let Some(binding) = binding_records.get(binding_id.raw() as usize) else {
                return Err(EnvironmentLayoutPlanError::InvalidBindingReference {
                    scope: scope_id,
                    binding: *binding_id,
                });
            };
            if binding.scope != scope_id {
                return Err(EnvironmentLayoutPlanError::BindingScopeMismatch {
                    scope: scope_id,
                    binding: *binding_id,
                    actual_scope: binding.scope,
                });
            }

            match binding.storage_class {
                StorageClass::FrameLocal => {}
                StorageClass::GlobalName => {
                    if !global_var_names.contains(&binding.name) {
                        global_var_names.push(binding.name);
                    }
                }
                StorageClass::EnvironmentSlot | StorageClass::DynamicLookup => {
                    if binding.storage_class == StorageClass::DynamicLookup
                        && binding.slot_index.is_none()
                    {
                        continue;
                    }
                    let expected_slot = u32::try_from(bindings.len())
                        .expect("binding slot index must fit into u32");
                    let Some(actual_slot) = binding.slot_index else {
                        return Err(EnvironmentLayoutPlanError::MissingSlotIndex {
                            scope: scope_id,
                            binding: *binding_id,
                        });
                    };
                    if actual_slot != expected_slot {
                        return Err(EnvironmentLayoutPlanError::UnexpectedSlotIndex {
                            scope: scope_id,
                            binding: *binding_id,
                            expected: expected_slot,
                            actual: actual_slot,
                        });
                    }
                    bindings.push(EnvironmentBindingLayout::new(
                        Some(binding.name),
                        binding_flags(binding),
                    ));
                }
            }
        }

        scopes[index] = Some(ScopeEnvironmentLayoutPlan::new(
            EnvironmentLayout::new(
                layout_kind,
                bindings,
                scope_needs_environment(scope_record, function_records),
            ),
            global_var_names,
        ));
    }

    Ok(EnvironmentLayoutPlan { scopes, functions })
}

/// Allocates runtime environment layouts for the derived plan and returns their stable ids.
pub fn install_environment_layout_plan(
    agent: &mut Agent,
    plan: &EnvironmentLayoutPlan,
) -> InstalledEnvironmentLayouts {
    let scopes = plan
        .scopes
        .iter()
        .map(|scope| {
            scope
                .as_ref()
                .map(|scope| agent.alloc_environment_layout(scope.layout().clone()))
        })
        .collect();

    InstalledEnvironmentLayouts { scopes }
}

/// Seeds `var`-style global names into a global environment record from the derived layout plan.
pub fn seed_global_var_names(
    agent: &mut Agent,
    env: EnvironmentRef,
    plan: &EnvironmentLayoutPlan,
    scope: ScopeId,
) -> bool {
    let Some(scope_plan) = plan.scope(scope) else {
        return false;
    };
    if scope_plan.layout().kind() != EnvironmentLayoutKind::Global {
        return false;
    }
    for name in scope_plan.global_var_names().iter().copied() {
        let _ = agent.global_add_var_name(env, name);
    }
    true
}

fn scope_layout_kind(kind: ScopeKind) -> Option<EnvironmentLayoutKind> {
    match kind {
        ScopeKind::Global => Some(EnvironmentLayoutKind::Global),
        ScopeKind::Module => Some(EnvironmentLayoutKind::Module),
        ScopeKind::Function => Some(EnvironmentLayoutKind::Function),
        ScopeKind::Block
        | ScopeKind::Catch
        | ScopeKind::ForLoop
        | ScopeKind::Switch
        | ScopeKind::Parameter => Some(EnvironmentLayoutKind::Declarative),
        ScopeKind::ClassBody => Some(EnvironmentLayoutKind::Private),
        ScopeKind::With => None,
    }
}

fn scope_needs_environment(
    scope: &lyng_js_sema::ScopeRecord,
    function_records: &[FunctionSemaRecord],
) -> bool {
    match scope.kind {
        ScopeKind::Global | ScopeKind::Module => true,
        ScopeKind::Function => scope
            .owning_function
            .and_then(|id| function_records.get(id.raw() as usize))
            .map_or(scope.needs_environment, |record| record.needs_environment),
        ScopeKind::Block
        | ScopeKind::Catch
        | ScopeKind::ClassBody
        | ScopeKind::ForLoop
        | ScopeKind::Switch
        | ScopeKind::Parameter
        | ScopeKind::With => scope.needs_environment,
    }
}

fn binding_flags(binding: &BindingRecord) -> EnvironmentSlotFlags {
    EnvironmentSlotFlags::new(
        binding_is_mutable(binding.kind),
        binding.kind.is_lexical(),
        binding.has_tdz,
        matches!(binding.storage_class, StorageClass::DynamicLookup),
    )
    .with_sloppy_immutable_assign_silent(matches!(binding.kind, DeclarationKind::FunctionName))
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

fn validate_scope_reference(
    scope_records: &[lyng_js_sema::ScopeRecord],
    scope: ScopeId,
) -> EnvironmentLayoutPlanResult<()> {
    if scope_records.get(scope.raw() as usize).is_some() {
        Ok(())
    } else {
        Err(EnvironmentLayoutPlanError::InvalidScopeReference { scope })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_ast::FunctionId;

    fn build_layout_tables() -> (
        ScopeTable,
        BindingTable,
        FunctionSemaTable,
        ScopeId,
        ScopeId,
        SemanticBindingId,
        SemanticBindingId,
    ) {
        let function_id = FunctionSemaId::new(0);
        let mut scopes = ScopeTable::new();
        let global_scope = scopes.alloc(lyng_js_sema::ScopeRecord {
            parent: None,
            kind: ScopeKind::Global,
            owning_function: None,
            strict: false,
            has_eval: false,
            has_with: false,
            needs_environment: true,
            bindings: Vec::new(),
            children: Vec::new(),
        });
        let function_scope = scopes.alloc(lyng_js_sema::ScopeRecord {
            parent: Some(global_scope),
            kind: ScopeKind::Function,
            owning_function: Some(function_id),
            strict: false,
            has_eval: false,
            has_with: false,
            needs_environment: false,
            bindings: Vec::new(),
            children: Vec::new(),
        });
        scopes.get_mut(global_scope).children.push(function_scope);

        let mut bindings = BindingTable::new();
        let global_var = bindings.alloc(BindingRecord {
            name: AtomId::from_raw(11),
            kind: DeclarationKind::Var,
            scope: global_scope,
            is_captured: false,
            needs_environment: false,
            storage_class: StorageClass::GlobalName,
            has_tdz: false,
            slot_index: None,
        });
        let function_slot = bindings.alloc(BindingRecord {
            name: AtomId::from_raw(12),
            kind: DeclarationKind::Let,
            scope: function_scope,
            is_captured: true,
            needs_environment: true,
            storage_class: StorageClass::EnvironmentSlot,
            has_tdz: true,
            slot_index: Some(0),
        });
        scopes.get_mut(global_scope).bindings.push(global_var);
        scopes.get_mut(function_scope).bindings.push(function_slot);

        let mut functions = FunctionSemaTable::new();
        functions.alloc(FunctionSemaRecord {
            function_id: FunctionId::new(0),
            strict: false,
            scope_root: function_scope,
            param_scope: None,
            needs_environment: true,
            has_eval: false,
            has_with: false,
            needs_arguments: false,
            references_super: false,
            references_new_target: false,
            references_this: false,
            has_await: false,
            has_yield: false,
            captures: vec![function_slot],
        });

        (
            scopes,
            bindings,
            functions,
            global_scope,
            function_scope,
            global_var,
            function_slot,
        )
    }

    #[test]
    fn derive_environment_layout_plan_preserves_slot_order_and_scope_metadata() {
        let (scopes, bindings, functions, global_scope, function_scope, _, _) =
            build_layout_tables();

        let plan = derive_environment_layout_plan(&scopes, &bindings, &functions)
            .expect("valid sema tables should derive an environment layout plan");

        let global_layout = plan
            .scope(global_scope)
            .expect("global scope should have a derived layout");
        let function_layout = plan
            .scope(function_scope)
            .expect("function scope should have a derived layout");

        assert_eq!(global_layout.layout().kind(), EnvironmentLayoutKind::Global);
        assert_eq!(global_layout.global_var_names(), &[AtomId::from_raw(11)]);
        assert_eq!(
            function_layout.layout().kind(),
            EnvironmentLayoutKind::Function
        );
        assert_eq!(function_layout.layout().slot_count(), 1);
        assert_eq!(
            function_layout.layout().binding(0),
            Some(EnvironmentBindingLayout::new(
                Some(AtomId::from_raw(12)),
                EnvironmentSlotFlags::mutable_lexical(),
            ))
        );

        let function_plan = plan
            .function(FunctionSemaId::new(0))
            .expect("function plan should be indexed by sema function id");
        assert_eq!(function_plan.scope_root(), function_scope);
        assert_eq!(function_plan.param_scope(), None);
        assert!(function_plan.needs_environment());
    }

    #[test]
    fn derive_environment_layout_plan_rejects_invalid_function_scope_references() {
        let (scopes, bindings, mut functions, _, _, _, _) = build_layout_tables();
        functions.get_mut(FunctionSemaId::new(0)).scope_root = ScopeId::new(99);

        assert_eq!(
            derive_environment_layout_plan(&scopes, &bindings, &functions),
            Err(EnvironmentLayoutPlanError::InvalidScopeReference {
                scope: ScopeId::new(99),
            })
        );
    }

    #[test]
    fn derive_environment_layout_plan_rejects_out_of_order_slot_indices() {
        let (scopes, mut bindings, functions, _, function_scope, _, function_slot) =
            build_layout_tables();
        bindings.get_mut(function_slot).slot_index = Some(2);

        assert_eq!(
            derive_environment_layout_plan(&scopes, &bindings, &functions),
            Err(EnvironmentLayoutPlanError::UnexpectedSlotIndex {
                scope: function_scope,
                binding: function_slot,
                expected: 0,
                actual: 2,
            })
        );
    }
}

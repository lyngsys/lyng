//! Compile-smoke coverage for the runtime substrate crate DAG.

use lyng_js_ast::FunctionId;
use lyng_js_common::AtomId;
use lyng_js_compiler::{derive_environment_layout_plan, EnvironmentLayoutPlanError};
use lyng_js_env::{
    EnvironmentLayoutKind, ExecutableId, ExecutionContext, ExecutionContextKind, Intrinsics,
    JobQueueKind, Runtime, ThisState,
};
use lyng_js_host::NoopHostHooks;
use lyng_js_sema::{
    BindingRecord, BindingTable, DeclarationKind, FunctionSemaId, FunctionSemaRecord,
    FunctionSemaTable, ScopeKind, ScopeRecord, ScopeTable, StorageClass,
};
use lyng_js_types::{CodeRef, Value};

#[test]
fn runtime_topology_boots_through_public_env_surface() {
    let runtime = Runtime::new(NoopHostHooks);
    let default_realm = runtime
        .root_agent()
        .default_realm()
        .expect("runtime should expose a default realm");
    let context = ExecutionContext::bytecode(
        default_realm.id(),
        CodeRef::from_raw(17).unwrap(),
        default_realm.global_env(),
        default_realm.global_env(),
    )
    .with_this_state(ThisState::Value(Value::undefined()));
    let intrinsics = Intrinsics::new().with_object_prototype(Some(default_realm.global_object()));

    assert_eq!(runtime.root_cluster().agent_count(), 1);
    assert_eq!(
        runtime.root_agent().queued_job_count(JobQueueKind::Script),
        0
    );
    assert_eq!(runtime.root_agent().realm_refs(), &[default_realm.id()]);
    assert!(default_realm.is_default());
    assert_eq!(context.kind(), ExecutionContextKind::Function);
    assert_eq!(
        context.executable(),
        ExecutableId::Bytecode(CodeRef::from_raw(17).unwrap())
    );
    assert_eq!(context.this_state(), ThisState::Value(Value::undefined()));
    assert_eq!(
        intrinsics.object_prototype(),
        Some(default_realm.global_object())
    );
}

#[test]
fn sema_bridge_stays_outside_the_runtime_surface() {
    let function_id = FunctionSemaId::new(0);
    let mut scopes = ScopeTable::new();
    let global_scope = scopes.alloc(ScopeRecord {
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
    let function_scope = scopes.alloc(ScopeRecord {
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
        name: AtomId::from_raw(77),
        kind: DeclarationKind::Var,
        scope: global_scope,
        is_captured: false,
        needs_environment: false,
        storage_class: StorageClass::GlobalName,
        has_tdz: false,
        slot_index: None,
    });
    let function_lexical = bindings.alloc(BindingRecord {
        name: AtomId::from_raw(78),
        kind: DeclarationKind::Let,
        scope: function_scope,
        is_captured: true,
        needs_environment: true,
        storage_class: StorageClass::EnvironmentSlot,
        has_tdz: true,
        slot_index: Some(0),
    });
    scopes.get_mut(global_scope).bindings.push(global_var);
    scopes
        .get_mut(function_scope)
        .bindings
        .push(function_lexical);

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
        captures: vec![function_lexical],
    });

    let plan = derive_environment_layout_plan(&scopes, &bindings, &functions)
        .expect("integration support should derive runtime layouts");

    assert_eq!(
        plan.scope(global_scope).unwrap().global_var_names(),
        &[AtomId::from_raw(77)]
    );
    assert_eq!(
        plan.scope(function_scope).unwrap().layout().kind(),
        EnvironmentLayoutKind::Function
    );
    assert!(plan.function(function_id).unwrap().needs_environment());
}

#[test]
fn integration_layout_bridge_rejects_out_of_order_slot_indices() {
    let function_id = FunctionSemaId::new(0);
    let mut scopes = ScopeTable::new();
    let global_scope = scopes.alloc(ScopeRecord {
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
    let function_scope = scopes.alloc(ScopeRecord {
        parent: Some(global_scope),
        kind: ScopeKind::Function,
        owning_function: Some(function_id),
        strict: false,
        has_eval: false,
        has_with: false,
        needs_environment: true,
        bindings: Vec::new(),
        children: Vec::new(),
    });
    scopes.get_mut(global_scope).children.push(function_scope);

    let mut bindings = BindingTable::new();
    let global_lexical = bindings.alloc(BindingRecord {
        name: AtomId::from_raw(88),
        kind: DeclarationKind::Let,
        scope: global_scope,
        is_captured: false,
        needs_environment: true,
        storage_class: StorageClass::EnvironmentSlot,
        has_tdz: true,
        slot_index: Some(2),
    });
    scopes.get_mut(global_scope).bindings.push(global_lexical);

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
        captures: Vec::new(),
    });

    assert_eq!(
        derive_environment_layout_plan(&scopes, &bindings, &functions),
        Err(EnvironmentLayoutPlanError::UnexpectedSlotIndex {
            scope: global_scope,
            binding: global_lexical,
            expected: 0,
            actual: 2,
        })
    );
}

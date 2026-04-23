//! Cross-crate integration coverage for the runtime substrate.
#![allow(clippy::too_many_lines)]

use lyng_js_ast::FunctionId;
use lyng_js_common::{AtomId, Severity, SourceId, Span};
use lyng_js_compiler::{
    derive_environment_layout_plan, install_environment_layout_plan, seed_global_var_names,
};
use lyng_js_env::{EnvironmentRecord, ParkedAgentRecord, Runtime, ThisBindingStatus};
use lyng_js_gc::{AllocationLifetime, PrimitiveMutator};
use lyng_js_host::{
    AgentSpawnKind, AgentThreadStartKind, ArrayBufferTransferRequest, DiagnosticReportRequest,
    HostCall, HostHooks, HostJobKind, HostTransferredBufferId, ImportMetaProperties,
    ImportMetaProperty, ImportMetaRequest, ImportMetaValue, LoadedModuleSource, LoadedSourceText,
    ModuleKey, ModuleSourceRequest, ParkAgentRequest, ParkAgentStatus, ScriptSourceRequest,
    SharedArrayBufferShareRequest, TestHost, UncaughtExceptionReport, UnparkAgentRequest,
    WaitLocation,
};
use lyng_js_objects::{
    FunctionConstructorFlags, FunctionObjectData, FunctionThisMode, InternalMethodResult,
    InvalidationCause, NamedPropertyStorageMode, NativeCallRequest, NativeConstructRequest,
    NativeFunctionRegistry, ObjectAllocation, ObjectColdData, ObjectRuntime,
};
use lyng_js_sema::{
    BindingRecord, BindingTable, DeclarationKind, FunctionSemaId, FunctionSemaRecord,
    FunctionSemaTable, ScopeId, ScopeKind, ScopeRecord, ScopeTable, StorageClass,
};
use lyng_js_types::{
    BackingStoreRef, BuiltinFunctionId, EnvironmentRef, NativeFunctionId, ObjectRef,
    PropertyDescriptor, PropertyKey, SymbolRef, Value,
};

const GLOBAL_LEXICAL_ATOM: AtomId = AtomId::from_raw(701);
const GLOBAL_VAR_ATOM: AtomId = AtomId::from_raw(702);
const SHADOW_ATOM: AtomId = AtomId::from_raw(703);
const FUNCTION_ONLY_ATOM: AtomId = AtomId::from_raw(704);
const OBJECT_ONLY_ATOM: AtomId = AtomId::from_raw(705);

fn data_descriptor(value: Value, enumerable: bool, configurable: bool) -> PropertyDescriptor {
    let mut descriptor = PropertyDescriptor::new();
    descriptor.set_value(value);
    descriptor.set_writable(true);
    descriptor.set_enumerable(enumerable);
    descriptor.set_configurable(configurable);
    descriptor
}

fn build_environment_plan_tables() -> (
    ScopeTable,
    BindingTable,
    FunctionSemaTable,
    ScopeId,
    ScopeId,
    ScopeId,
) {
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
    let block_scope = scopes.alloc(ScopeRecord {
        parent: Some(function_scope),
        kind: ScopeKind::Block,
        owning_function: Some(function_id),
        strict: false,
        has_eval: false,
        has_with: false,
        needs_environment: true,
        bindings: Vec::new(),
        children: Vec::new(),
    });
    scopes.get_mut(global_scope).children.push(function_scope);
    scopes.get_mut(function_scope).children.push(block_scope);

    let mut bindings = BindingTable::new();
    let global_lexical = bindings.alloc(BindingRecord {
        name: GLOBAL_LEXICAL_ATOM,
        kind: DeclarationKind::Let,
        scope: global_scope,
        is_captured: false,
        needs_environment: true,
        storage_class: StorageClass::EnvironmentSlot,
        has_tdz: true,
        slot_index: Some(0),
    });
    let global_var = bindings.alloc(BindingRecord {
        name: GLOBAL_VAR_ATOM,
        kind: DeclarationKind::Var,
        scope: global_scope,
        is_captured: false,
        needs_environment: false,
        storage_class: StorageClass::GlobalName,
        has_tdz: false,
        slot_index: None,
    });
    let function_shadow = bindings.alloc(BindingRecord {
        name: SHADOW_ATOM,
        kind: DeclarationKind::Let,
        scope: function_scope,
        is_captured: true,
        needs_environment: true,
        storage_class: StorageClass::EnvironmentSlot,
        has_tdz: true,
        slot_index: Some(0),
    });
    let function_only = bindings.alloc(BindingRecord {
        name: FUNCTION_ONLY_ATOM,
        kind: DeclarationKind::Let,
        scope: function_scope,
        is_captured: true,
        needs_environment: true,
        storage_class: StorageClass::EnvironmentSlot,
        has_tdz: true,
        slot_index: Some(1),
    });
    let block_shadow = bindings.alloc(BindingRecord {
        name: SHADOW_ATOM,
        kind: DeclarationKind::Let,
        scope: block_scope,
        is_captured: false,
        needs_environment: true,
        storage_class: StorageClass::EnvironmentSlot,
        has_tdz: true,
        slot_index: Some(0),
    });

    scopes.get_mut(global_scope).bindings = vec![global_lexical, global_var];
    scopes.get_mut(function_scope).bindings = vec![function_shadow, function_only];
    scopes.get_mut(block_scope).bindings = vec![block_shadow];

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
        captures: vec![function_shadow, function_only],
    });

    (
        scopes,
        bindings,
        functions,
        global_scope,
        function_scope,
        block_scope,
    )
}

fn lookup_slot_binding(
    runtime: &Runtime,
    env: EnvironmentRef,
    layout_id: lyng_js_env::EnvironmentLayoutId,
    name: AtomId,
) -> Option<Value> {
    let agent = runtime.root_agent();
    let layout = agent.environment_layout(layout_id)?;
    let index = layout
        .bindings()
        .iter()
        .position(|binding| binding.name() == Some(name))?;
    let index = u32::try_from(index).expect("environment slot index must fit into u32");
    agent.environment_slot(env, index)
}

fn lookup_binding(runtime: &Runtime, start: EnvironmentRef, name: AtomId) -> Option<Value> {
    let agent = runtime.root_agent();
    let key = PropertyKey::from_atom(name);
    let mut current = Some(start);
    while let Some(env) = current {
        match agent.environment(env)? {
            EnvironmentRecord::Declarative(record) => {
                if let Some(value) =
                    lookup_slot_binding(runtime, record.id(), record.layout(), name)
                {
                    return Some(value);
                }
                current = record.outer();
            }
            EnvironmentRecord::Private(record) => {
                current = record.outer();
            }
            EnvironmentRecord::Function(record) => {
                let declarative = record.declarative();
                if let Some(value) =
                    lookup_slot_binding(runtime, declarative.id(), declarative.layout(), name)
                {
                    return Some(value);
                }
                current = declarative.outer();
            }
            EnvironmentRecord::Module(record) => {
                if let Some(value) =
                    lookup_slot_binding(runtime, record.id(), record.layout(), name)
                {
                    return Some(value);
                }
                current = record.outer();
            }
            EnvironmentRecord::Global(record) => {
                if let Some(value) =
                    lookup_slot_binding(runtime, record.id(), record.layout(), name)
                {
                    return Some(value);
                }
                if record.has_var_name(name)
                    && agent
                        .objects()
                        .has_property(agent.heap().view(), record.global_object(), key)
                        .ok()?
                {
                    return agent
                        .objects()
                        .get(
                            agent.heap().view(),
                            record.global_object(),
                            key,
                            Value::from_object_ref(record.global_object()),
                        )
                        .ok();
                }
                current = record.outer();
            }
            EnvironmentRecord::Object(record) => {
                if agent
                    .objects()
                    .has_property(agent.heap().view(), record.binding_object(), key)
                    .ok()?
                {
                    return agent
                        .objects()
                        .get(
                            agent.heap().view(),
                            record.binding_object(),
                            key,
                            Value::from_object_ref(record.binding_object()),
                        )
                        .ok();
                }
                current = record.outer();
            }
        }
    }
    None
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RecordedNativeCall {
    callee: ObjectRef,
    this_value: Value,
    arguments: Vec<Value>,
    realm: lyng_js_types::RealmRef,
    environment: EnvironmentRef,
    private_env: Option<EnvironmentRef>,
    home_object: Option<ObjectRef>,
    entry: NativeFunctionId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RecordedNativeConstruct {
    callee: ObjectRef,
    new_target: ObjectRef,
    arguments: Vec<Value>,
    realm: lyng_js_types::RealmRef,
    environment: EnvironmentRef,
    private_env: Option<EnvironmentRef>,
    home_object: Option<ObjectRef>,
    entry: NativeFunctionId,
}

#[derive(Default)]
struct RecordingNativeRegistry {
    last_call: Option<RecordedNativeCall>,
    last_construct: Option<RecordedNativeConstruct>,
}

impl NativeFunctionRegistry for RecordingNativeRegistry {
    fn call(
        &mut self,
        _runtime: &mut ObjectRuntime,
        _heap: &mut PrimitiveMutator<'_>,
        request: NativeCallRequest<'_>,
    ) -> InternalMethodResult<Value> {
        self.last_call = Some(RecordedNativeCall {
            callee: request.callee(),
            this_value: request.this_value(),
            arguments: request.arguments().to_vec(),
            realm: request.realm(),
            environment: request.environment(),
            private_env: request.private_env(),
            home_object: request.home_object(),
            entry: request.entry(),
        });
        Ok(Value::from_smi(123))
    }

    fn construct(
        &mut self,
        runtime: &mut ObjectRuntime,
        heap: &mut PrimitiveMutator<'_>,
        request: NativeConstructRequest<'_>,
    ) -> InternalMethodResult<ObjectRef> {
        self.last_construct = Some(RecordedNativeConstruct {
            callee: request.callee(),
            new_target: request.new_target(),
            arguments: request.arguments().to_vec(),
            realm: request.realm(),
            environment: request.environment(),
            private_env: request.private_env(),
            home_object: request.home_object(),
            entry: request.entry(),
        });
        let root = runtime.root_shape(heap, None, AllocationLifetime::Default);
        Ok(runtime.alloc_object(
            heap,
            ObjectAllocation::ordinary(root).with_prototype(Some(request.new_target())),
            AllocationLifetime::Default,
        ))
    }
}

#[test]
fn phase3_environment_chain_shadowing_and_realm_state_flow_across_crates() {
    let (scopes, bindings, functions, global_scope, function_scope, block_scope) =
        build_environment_plan_tables();
    let plan = derive_environment_layout_plan(&scopes, &bindings, &functions)
        .expect("sema metadata should derive a runtime plan");

    let mut runtime = Runtime::new(TestHost::new());
    let agent = runtime.root_agent_mut();
    let installed = install_environment_layout_plan(agent, &plan);
    let default_realm = agent
        .default_realm()
        .expect("runtime should bootstrap a default realm");

    let (
        object_prototype,
        function_prototype,
        array_prototype,
        function_object,
        home_object,
        new_target,
        binding_object,
    ) = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let root = default_realm
            .root_shape()
            .unwrap_or_else(|| objects.root_shape(&mut mutator, None, AllocationLifetime::Default));

        let object_prototype = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root),
            AllocationLifetime::Default,
        );
        let function_prototype = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root),
            AllocationLifetime::Default,
        );
        let array_prototype = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root),
            AllocationLifetime::Default,
        );
        let function_object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root),
            AllocationLifetime::Default,
        );
        let home_object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root),
            AllocationLifetime::Default,
        );
        let new_target = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root),
            AllocationLifetime::Default,
        );
        let binding_object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root),
            AllocationLifetime::Default,
        );

        let object_binding = data_descriptor(Value::from_smi(40), true, true);
        assert!(objects
            .define_own_property(
                &mut mutator,
                binding_object,
                PropertyKey::from_atom(OBJECT_ONLY_ATOM),
                object_binding,
                AllocationLifetime::Default,
            )
            .unwrap());
        let global_binding = data_descriptor(Value::from_smi(90), true, true);
        assert!(objects
            .define_own_property(
                &mut mutator,
                default_realm.global_object(),
                PropertyKey::from_atom(GLOBAL_VAR_ATOM),
                global_binding,
                AllocationLifetime::Default,
            )
            .unwrap());

        (
            object_prototype,
            function_prototype,
            array_prototype,
            function_object,
            home_object,
            new_target,
            binding_object,
        )
    });

    let intrinsics = lyng_js_env::Intrinsics::new()
        .with_object_prototype(Some(object_prototype))
        .with_function_prototype(Some(function_prototype))
        .with_array_prototype(Some(array_prototype));
    assert!(agent.set_realm_intrinsics(default_realm.id(), intrinsics));

    let global_layout = installed
        .scope(global_scope)
        .expect("global scope should install a layout");
    let function_layout = installed
        .scope(function_scope)
        .expect("function scope should install a layout");
    let block_layout = installed
        .scope(block_scope)
        .expect("block scope should install a layout");

    let global_env = agent
        .alloc_global_environment(
            None,
            global_layout,
            default_realm.global_object(),
            AllocationLifetime::Default,
        )
        .expect("global environment should allocate");
    assert!(seed_global_var_names(
        agent,
        global_env,
        &plan,
        global_scope
    ));
    assert!(agent.init_environment_slot(global_env, 0, Value::from_smi(10)));

    let function_env = agent
        .alloc_function_environment(
            Some(global_env),
            function_layout,
            function_object,
            ThisBindingStatus::Uninitialized,
            Value::undefined(),
            None,
            Some(home_object),
            AllocationLifetime::Default,
        )
        .expect("function environment should allocate");
    assert!(agent.init_environment_slot(function_env, 0, Value::from_smi(20)));
    assert!(agent.init_environment_slot(function_env, 1, Value::from_smi(50)));
    assert!(agent.set_function_this_binding(
        function_env,
        ThisBindingStatus::Initialized,
        Value::from_smi(21),
    ));
    assert!(agent.set_function_new_target(function_env, Some(new_target)));
    assert!(agent.set_function_home_object(function_env, Some(home_object)));

    let object_env = agent.alloc_object_environment(
        Some(function_env),
        binding_object,
        true,
        AllocationLifetime::Default,
    );
    let block_env = agent
        .alloc_declarative_environment(Some(object_env), block_layout, AllocationLifetime::Default)
        .expect("block environment should allocate");
    assert!(agent.init_environment_slot(block_env, 0, Value::from_smi(30)));
    agent.push_script_context(default_realm.id(), block_env, global_env);

    assert_eq!(
        lookup_binding(&runtime, block_env, SHADOW_ATOM),
        Some(Value::from_smi(30))
    );
    assert_eq!(
        lookup_binding(&runtime, function_env, SHADOW_ATOM),
        Some(Value::from_smi(20))
    );
    assert_eq!(
        lookup_binding(&runtime, block_env, FUNCTION_ONLY_ATOM),
        Some(Value::from_smi(50))
    );
    assert_eq!(
        lookup_binding(&runtime, block_env, OBJECT_ONLY_ATOM),
        Some(Value::from_smi(40))
    );
    assert_eq!(
        lookup_binding(&runtime, block_env, GLOBAL_LEXICAL_ATOM),
        Some(Value::from_smi(10))
    );
    assert_eq!(
        lookup_binding(&runtime, block_env, GLOBAL_VAR_ATOM),
        Some(Value::from_smi(90))
    );

    let agent = runtime.root_agent();
    let realm = agent
        .realm(default_realm.id())
        .expect("default realm should stay queryable");
    let function_record = agent
        .function_environment(function_env)
        .expect("function environment should be queryable");
    let global_record = agent
        .global_environment(global_env)
        .expect("global environment should be queryable");
    let object_record = agent
        .object_environment(object_env)
        .expect("object environment should be queryable");
    let current_context = agent
        .current_execution_context()
        .expect("script context should be visible");

    assert_eq!(realm.intrinsics(), intrinsics);
    assert_eq!(
        function_record.this_binding_status(),
        ThisBindingStatus::Initialized
    );
    assert_eq!(function_record.this_value(), Value::from_smi(21));
    assert_eq!(function_record.new_target(), Some(new_target));
    assert_eq!(function_record.home_object(), Some(home_object));
    assert!(global_record.has_var_name(GLOBAL_VAR_ATOM));
    assert_eq!(global_record.var_names().len(), 1);
    assert_eq!(object_record.binding_object(), binding_object);
    assert!(object_record.with_environment());
    assert_eq!(current_context.realm(), default_realm.id());
    assert_eq!(current_context.lexical_env(), block_env);
    assert_eq!(current_context.variable_env(), global_env);
}

#[test]
fn phase3_object_semantics_and_native_dispatch_flow_through_public_runtime_surface() {
    let mut runtime = Runtime::new(TestHost::new());
    let agent = runtime.root_agent_mut();
    let default_realm = agent
        .default_realm()
        .expect("runtime should bootstrap a default realm");
    let inherited_key = PropertyKey::from_atom(AtomId::from_raw(801));
    let data_key = PropertyKey::from_atom(AtomId::from_raw(802));
    let deleted_key = PropertyKey::from_atom(AtomId::from_raw(803));
    let new_key = PropertyKey::from_atom(AtomId::from_raw(804));
    let symbol_key = PropertyKey::from_symbol(SymbolRef::from_raw(17).unwrap());
    let index_key = PropertyKey::Index(2);
    let entry = BuiltinFunctionId::from_raw(9).unwrap();
    let mut registry = RecordingNativeRegistry::default();

    let (
        prototype,
        object,
        replacement_prototype,
        function_object,
        constructed,
        root_shape,
        shape_after_add,
        keys,
        call_result,
    ) = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let root_shape = default_realm
            .root_shape()
            .unwrap_or_else(|| objects.root_shape(&mut mutator, None, AllocationLifetime::Default));
        let prototype = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        );
        assert!(objects
            .define_own_property(
                &mut mutator,
                prototype,
                inherited_key,
                data_descriptor(Value::from_smi(11), true, true),
                AllocationLifetime::Default,
            )
            .unwrap());

        let object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(Some(prototype)),
            AllocationLifetime::Default,
        );
        let initial_shape = objects
            .object_header(mutator.view(), object)
            .unwrap()
            .shape();

        assert!(objects
            .define_own_property(
                &mut mutator,
                object,
                data_key,
                data_descriptor(Value::from_smi(1), true, true),
                AllocationLifetime::Default,
            )
            .unwrap());
        let shape_after_add = objects
            .object_header(mutator.view(), object)
            .unwrap()
            .shape();
        assert_ne!(shape_after_add, initial_shape);
        assert!(objects
            .define_own_property(
                &mut mutator,
                object,
                deleted_key,
                data_descriptor(Value::from_smi(2), true, true),
                AllocationLifetime::Default,
            )
            .unwrap());
        assert!(objects
            .define_own_property(
                &mut mutator,
                object,
                index_key,
                data_descriptor(Value::from_smi(3), true, true),
                AllocationLifetime::Default,
            )
            .unwrap());
        assert!(objects
            .define_own_property(
                &mut mutator,
                object,
                symbol_key,
                data_descriptor(Value::from_smi(4), true, true),
                AllocationLifetime::Default,
            )
            .unwrap());

        assert!(objects
            .has_property(mutator.view(), object, inherited_key)
            .unwrap());
        assert_eq!(
            objects
                .get(
                    mutator.view(),
                    object,
                    inherited_key,
                    Value::from_object_ref(object),
                )
                .unwrap(),
            Value::from_smi(11)
        );

        let mut redefine = PropertyDescriptor::new();
        redefine.set_writable(false);
        redefine.set_enumerable(true);
        redefine.set_configurable(true);
        assert!(objects
            .define_own_property(
                &mut mutator,
                object,
                data_key,
                redefine,
                AllocationLifetime::Default,
            )
            .unwrap());
        assert_eq!(
            objects.invalidation_event(object).unwrap().cause(),
            InvalidationCause::PropertyRedefinition
        );
        assert!(objects.delete(&mut mutator, object, deleted_key).unwrap());
        assert_eq!(
            objects.invalidation_event(object).unwrap().cause(),
            InvalidationCause::PropertyDeletion
        );

        let replacement_prototype = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        );
        assert!(objects
            .set_prototype_of(&mut mutator, object, Some(replacement_prototype))
            .unwrap());
        assert_eq!(
            objects.invalidation_event(object).unwrap().cause(),
            InvalidationCause::PrototypeMutation
        );
        assert!(objects.prevent_extensions(mutator.view(), object).unwrap());
        assert!(!objects.is_extensible(object).unwrap());
        assert!(!objects
            .define_own_property(
                &mut mutator,
                object,
                new_key,
                data_descriptor(Value::from_smi(99), true, true),
                AllocationLifetime::Default,
            )
            .unwrap());

        let keys = objects.own_property_keys(mutator.view(), object).unwrap();

        let home_object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        );
        let function_object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::function(root_shape).with_cold_data(ObjectColdData::Function(
                FunctionObjectData::native(default_realm.id(), default_realm.global_env(), entry)
                    .with_this_mode(FunctionThisMode::Global)
                    .with_home_object(Some(home_object))
                    .with_constructor_flags(FunctionConstructorFlags::constructible()),
            )),
            AllocationLifetime::Default,
        );
        let call_result = objects
            .call(
                &mut mutator,
                function_object,
                Value::from_smi(5),
                &[Value::from_smi(7), Value::from_smi(8)],
                &mut registry,
            )
            .unwrap();
        let constructed = objects
            .construct(
                &mut mutator,
                function_object,
                &[Value::from_smi(9)],
                None,
                &mut registry,
            )
            .unwrap();

        (
            prototype,
            object,
            replacement_prototype,
            function_object,
            constructed,
            root_shape,
            shape_after_add,
            keys,
            call_result,
        )
    });

    assert_eq!(call_result, Value::from_smi(123));
    assert_eq!(
        runtime
            .root_agent()
            .objects()
            .get_prototype_of(runtime.root_agent().heap().view(), object)
            .unwrap(),
        Some(replacement_prototype)
    );
    assert_eq!(
        runtime
            .root_agent()
            .objects()
            .get_own_property(runtime.root_agent().heap().view(), object, data_key)
            .unwrap()
            .unwrap()
            .writable(),
        Some(false)
    );
    assert_eq!(keys, vec![index_key, data_key, symbol_key]);
    assert_eq!(
        runtime
            .root_agent()
            .objects()
            .named_property_storage_mode(object),
        Some(NamedPropertyStorageMode::Dictionary)
    );
    assert_ne!(shape_after_add, root_shape);
    assert_ne!(
        runtime
            .root_agent()
            .objects()
            .object_header(runtime.root_agent().heap().view(), object)
            .unwrap()
            .shape(),
        root_shape
    );
    assert_eq!(
        runtime
            .root_agent()
            .objects()
            .object_header(runtime.root_agent().heap().view(), function_object)
            .unwrap()
            .kind(),
        lyng_js_objects::ObjectKind::Function
    );
    assert_eq!(
        runtime
            .root_agent()
            .objects()
            .object_header(runtime.root_agent().heap().view(), prototype)
            .unwrap()
            .prototype(),
        None
    );
    assert_eq!(
        runtime
            .root_agent()
            .objects()
            .object_header(runtime.root_agent().heap().view(), constructed)
            .unwrap()
            .prototype(),
        Some(function_object)
    );

    let recorded_call = registry.last_call.expect("call should be recorded");
    let recorded_construct = registry
        .last_construct
        .expect("construct should be recorded");
    assert_eq!(recorded_call.callee, function_object);
    assert_eq!(recorded_call.this_value, Value::from_smi(5));
    assert_eq!(
        recorded_call.arguments,
        vec![Value::from_smi(7), Value::from_smi(8)]
    );
    assert_eq!(recorded_call.realm, default_realm.id());
    assert_eq!(recorded_call.environment, default_realm.global_env());
    assert_eq!(recorded_call.entry, NativeFunctionId::builtin(entry));
    assert_eq!(recorded_construct.callee, function_object);
    assert_eq!(recorded_construct.new_target, function_object);
    assert_eq!(recorded_construct.arguments, vec![Value::from_smi(9)]);
    assert_eq!(recorded_construct.realm, default_realm.id());
    assert_eq!(recorded_construct.environment, default_realm.global_env());
    assert_eq!(recorded_construct.entry, NativeFunctionId::builtin(entry));
}

#[test]
fn phase3_host_boundary_and_cluster_plumbing_compose_through_public_surfaces() {
    let host = TestHost::new();
    host.define_script_source(
        "entry.js",
        LoadedSourceText::new("entry.js", "let value = 1;"),
    );
    host.define_module_source(
        "dep",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/dep.js"),
            "dep.js",
            "export const dep = 1;",
        ),
    );
    host.define_import_meta(
        ModuleKey::new("/tmp/dep.js"),
        ImportMetaProperties::new(vec![ImportMetaProperty {
            key: "url".into(),
            value: ImportMetaValue::String("file:///tmp/dep.js".into()),
        }]),
    );

    let mut runtime = Runtime::new(host.clone());
    let default_realm = runtime
        .root_agent()
        .default_realm()
        .expect("runtime should expose a default realm");

    let script = host
        .load_script_source(&ScriptSourceRequest {
            path: "entry.js".into(),
            referrer: None,
            is_entry: true,
        })
        .expect("script source should load from the test host");
    let module = host
        .load_module_source(&ModuleSourceRequest {
            specifier: "dep".into(),
            referrer: Some(ModuleKey::new("/tmp/main.js")),
            attributes: Vec::new(),
        })
        .expect("module source should load from the test host");
    let import_meta = host
        .resolve_import_meta(&ImportMetaRequest {
            module: module.key.clone(),
        })
        .expect("import.meta should load from the test host");
    host.report_diagnostic(&DiagnosticReportRequest {
        severity: Severity::Warning,
        source: Some(SourceId::new(8)),
        span: Some(Span::from_offsets(SourceId::new(8), 1, 3)),
        message: "phase3 warning".into(),
    })
    .expect("diagnostic reporting should succeed");
    host.report_uncaught_exception(&UncaughtExceptionReport {
        source: Some(SourceId::new(8)),
        realm: Some(default_realm.id()),
        thrown_value: Value::from_smi(13),
        message: "boom".into(),
    })
    .expect("uncaught exception reporting should succeed");

    let worker_a = runtime
        .spawn_agent(AgentSpawnKind::Harness, Some("worker-a".into()))
        .expect("runtime should spawn a worker");
    let worker_b = runtime
        .spawn_agent(AgentSpawnKind::SharedMemory, Some("worker-b".into()))
        .expect("runtime should spawn a second worker");
    let thread = runtime
        .start_agent_thread(
            worker_a,
            AgentThreadStartKind::Harness,
            Some("worker-a-thread".into()),
        )
        .expect("runtime should bind a thread");
    let job = runtime
        .enqueue_job(
            worker_a,
            HostJobKind::Harness,
            lyng_js_env::ExecutableId::Builtin,
            Some(default_realm.id()),
            Some("phase3-job".into()),
        )
        .expect("runtime should enqueue a job");

    let worker_a_host = runtime
        .root_cluster()
        .agent(worker_a)
        .and_then(lyng_js_env::Agent::host_id)
        .expect("worker A should carry a host agent id");
    let shared_worker_host = runtime
        .root_cluster()
        .agent(worker_b)
        .and_then(lyng_js_env::Agent::host_id)
        .expect("worker B should carry a host agent id");
    let cluster = runtime.root_cluster_mut();

    let transferred = host
        .transfer_array_buffer(&ArrayBufferTransferRequest {
            source_agent: worker_a_host,
            target_agent: shared_worker_host,
            buffer_id: HostTransferredBufferId::from_raw(4).unwrap(),
            byte_length: 64,
            detach_on_success: true,
        })
        .expect("array-buffer transfer should succeed");
    let shared = host
        .share_array_buffer(&SharedArrayBufferShareRequest {
            source_agent: worker_a_host,
            target_agent: shared_worker_host,
            backing_store: BackingStoreRef::from_raw(6).unwrap(),
            byte_length: 128,
        })
        .expect("shared-buffer sharing should succeed");
    let shared_backing_store = cluster
        .register_shared_backing_store(worker_a, 128)
        .expect("shared backing store should allocate");
    assert!(cluster.cache_shared_backing_store_handle(shared_backing_store, shared.shared_buffer));
    let location = WaitLocation::new(shared_backing_store, 16);
    let parked = host
        .park_agent(&ParkAgentRequest {
            agent_id: worker_a_host,
            thread_id: Some(thread),
            location,
            timeout_ns: Some(1_000),
            allow_async: true,
        })
        .expect("host park should succeed");
    let unparked = host
        .unpark_agent(&UnparkAgentRequest {
            location,
            max_count: 2,
        })
        .expect("host unpark should succeed");

    assert_eq!(script.display_name, "entry.js");
    assert_eq!(module.display_name, "dep.js");
    assert_eq!(module.key, ModuleKey::new("/tmp/dep.js"));
    assert_eq!(import_meta.properties.len(), 1);
    assert!(transferred.detached);
    assert_eq!(parked.status, ParkAgentStatus::Parked);
    assert_eq!(unparked.woken_agents, 1);
    assert_eq!(job.get(), 1);

    assert!(cluster.share_shared_backing_store(shared_backing_store, worker_b));
    assert!(cluster.park_agent(
        location,
        ParkedAgentRecord::new(worker_a, Some(thread), true),
    ));
    assert_eq!(cluster.waiter_count(location), 1);
    assert_eq!(
        cluster
            .shared_backing_store(shared_backing_store)
            .unwrap()
            .visible_to(),
        &[worker_a, worker_b]
    );
    assert_eq!(
        cluster
            .shared_backing_store(shared_backing_store)
            .unwrap()
            .host_shared_buffer(),
        Some(shared.shared_buffer)
    );
    assert_eq!(
        cluster.unpark_agents(location, 1),
        vec![ParkedAgentRecord::new(worker_a, Some(thread), true)]
    );

    let snapshot = host.snapshot();
    assert!(snapshot
        .calls
        .iter()
        .any(|call| matches!(call, HostCall::LoadScript(_))));
    assert!(snapshot
        .calls
        .iter()
        .any(|call| matches!(call, HostCall::LoadModule(_))));
    assert!(snapshot
        .calls
        .iter()
        .any(|call| matches!(call, HostCall::Diagnostic(_))));
    assert!(snapshot
        .calls
        .iter()
        .any(|call| matches!(call, HostCall::UncaughtException(_))));
    assert!(snapshot
        .calls
        .iter()
        .any(|call| matches!(call, HostCall::CreateAgent(_))));
    assert!(snapshot
        .calls
        .iter()
        .any(|call| matches!(call, HostCall::StartAgentThread(_))));
    assert!(snapshot
        .calls
        .iter()
        .any(|call| matches!(call, HostCall::ObserveJob(_))));
    assert!(snapshot
        .calls
        .iter()
        .any(|call| matches!(call, HostCall::TransferArrayBuffer(_))));
    assert!(snapshot
        .calls
        .iter()
        .any(|call| matches!(call, HostCall::ShareArrayBuffer(_))));
    assert!(snapshot
        .calls
        .iter()
        .any(|call| matches!(call, HostCall::ParkAgent(_))));
    assert!(snapshot
        .calls
        .iter()
        .any(|call| matches!(call, HostCall::UnparkAgent(_))));
}

use crate::{
    seed_registers, FrameFlags, FrameRecord, InstalledCode, RegisterWindow, Vm, VmError, VmMarker,
};
use lyng_js_bytecode::{
    ArgumentsMode, BytecodeBuilder, BytecodeFunction, BytecodeFunctionId, BytecodeFunctionKind,
    BytecodeMarker, CompiledAtom, CompiledFunctionUnit, CompiledScriptUnit, ConstantValue,
    DeoptFrameValue, DeoptValueSource, ExceptionHandler, ExceptionHandlerKind, FeedbackSiteKind,
    Instruction, Opcode, SafepointKind,
};
use lyng_js_common::{AtomId, AtomTable, SourceId, WellKnownAtom};
use lyng_js_compiler::{compile_module, compile_script, CompiledModuleUnit};
use lyng_js_env::{
    EnvironmentBindingLayout, EnvironmentLayout, EnvironmentLayoutKind, EnvironmentSlotFlags,
    ExecutableId, ExecutionContext, ExecutionContextKind, JobQueueKind, ModuleStatus,
    PromiseReactionHandler, PromiseReactionKind, PromiseReactionRecord, RealmBootstrapState,
    Runtime, RuntimeJobPayload, ThisState,
};
use lyng_js_gc::{AllocationLifetime, BigIntSign, PrimitiveMutator, PrimitiveStringView};
use lyng_js_host::{
    HostCall, HostJobKind, HostJobPhase, ImportMetaProperties, ImportMetaProperty, ImportMetaValue,
    LoadedModuleSource, ModuleKey, ModuleSourceRequest, NoopHostHooks, TestHost,
};
use lyng_js_objects::{
    FunctionEntryIdentity, InternalMethodResult, NamedPropertyStorageMode, NativeCallRequest,
    NativeConstructRequest, NativeFunctionRegistry, ObjectAllocation, ObjectRuntime,
};
use lyng_js_ops::object::{ordinary_create_data_property, ordinary_get};
use lyng_js_parser::{parse_module, parse_script};
use lyng_js_sema::{analyze_module, analyze_script};
use lyng_js_types::{
    function_builtin, internal_function_call_builtin, symbol_builtin, CodeRef, EmbeddingFunctionId,
    EnvironmentRef, FeedbackSlotId, NativeFunctionId, ObjectRef, PropertyKey, RealmRef, Value,
};
use std::mem::size_of;
use std::num::NonZeroU32;
use std::sync::Arc;

#[derive(Default)]
struct RejectingRegistry;

impl NativeFunctionRegistry for RejectingRegistry {
    fn call(
        &mut self,
        _runtime: &mut ObjectRuntime,
        _heap: &mut PrimitiveMutator<'_>,
        _request: NativeCallRequest<'_>,
    ) -> InternalMethodResult<Value> {
        panic!("unexpected native call during vm promise test");
    }

    fn construct(
        &mut self,
        _runtime: &mut ObjectRuntime,
        _heap: &mut PrimitiveMutator<'_>,
        _request: NativeConstructRequest<'_>,
    ) -> InternalMethodResult<ObjectRef> {
        panic!("unexpected native construct during vm promise test");
    }
}

const TEST_EMBEDDING_EVAL_SCRIPT_RAW: u32 = 1;

fn test_embedding_eval_script_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST_EMBEDDING_EVAL_SCRIPT_RAW)
        .expect("embedding function ids should stay non-zero")
}

fn test_embedding_property_key(agent: &mut lyng_js_env::Agent, text: &str) -> PropertyKey {
    PropertyKey::from_atom(agent.atoms_mut().intern_collectible(text))
}

#[derive(Clone, Default)]
struct TestEmbeddingProvider;

impl crate::RealmExtensionProvider for TestEmbeddingProvider {
    fn embedding_function_metadata(
        &self,
        entry: EmbeddingFunctionId,
    ) -> Option<crate::EmbeddingFunctionMetadata> {
        (entry == test_embedding_eval_script_entry()).then_some(
            crate::EmbeddingFunctionMetadata::new("evalScript", 1, false, false),
        )
    }

    fn install_realm_extensions(
        &self,
        installation: &mut crate::RealmExtensionInstallation<'_>,
    ) -> Result<(), VmError> {
        let key = test_embedding_property_key(installation.agent(), "embeddingEvalScript");
        let _ = installation.define_function_property(
            installation.global_object(),
            key,
            test_embedding_eval_script_entry(),
            true,
            false,
            true,
        )?;
        Ok(())
    }

    fn call_embedding_function(
        &self,
        context: &mut crate::EmbeddingFunctionContext<'_>,
        entry: EmbeddingFunctionId,
        invocation: crate::EmbeddingInvocation<'_>,
    ) -> Result<Value, VmError> {
        if entry != test_embedding_eval_script_entry() {
            return Err(VmError::MissingEmbeddingFunction(entry));
        }
        let source = invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined());
        let source_text = context.value_to_string_text(source)?;
        context.evaluate_script_in_realm(context.function_realm(), &source_text)
    }
}

fn compile_test_unit(source_id: u32, source: &str) -> CompiledScriptUnit {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, SourceId::new(source_id), source);
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    compile_script(&parsed, &sema, &mut atoms).unwrap()
}

fn compile_test_module(source_id: u32, source: &str) -> CompiledModuleUnit {
    let mut atoms = AtomTable::new();
    let parsed = parse_module(&mut atoms, SourceId::new(source_id), source);
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_module(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    compile_module(&parsed, &sema, &mut atoms).unwrap()
}

fn unit_atom(unit: &CompiledScriptUnit, text: &str) -> AtomId {
    unit.atoms()
        .iter()
        .find_map(|(atom, candidate)| (candidate.as_str() == Some(text)).then_some(*atom))
        .unwrap_or_else(|| panic!("compiled unit should intern atom {text:?}"))
}

fn unit_runtime_atom(
    agent: &mut lyng_js_env::Agent,
    unit: &CompiledScriptUnit,
    atom: AtomId,
) -> AtomId {
    if let Some(text) = unit.atom_text(atom) {
        return agent.atoms_mut().intern_collectible(text);
    }
    let units = unit
        .atom_utf16(atom)
        .expect("compiled unit atom should resolve to UTF-8 or UTF-16 data");
    agent.atoms_mut().intern_collectible_utf16(units)
}

fn install_global_value(
    agent: &mut lyng_js_env::Agent,
    realm: lyng_js_env::RealmRecord,
    name: AtomId,
    value: Value,
) {
    assert!(ordinary_create_data_property(
        agent,
        realm.global_object(),
        PropertyKey::from_atom(name),
        value,
        AllocationLifetime::Default,
    )
    .unwrap());
}

fn decode_string(view: PrimitiveStringView<'_>) -> String {
    if let Some(bytes) = view.latin1_bytes() {
        return bytes.iter().map(|byte| char::from(*byte)).collect();
    }
    let bytes = view
        .utf16_bytes()
        .expect("string view must be Latin1 or UTF-16");
    let mut units = Vec::with_capacity(view.code_unit_len() as usize);
    for chunk in bytes.chunks_exact(2) {
        units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    String::from_utf16_lossy(&units)
}

fn global_value(
    agent: &mut lyng_js_env::Agent,
    realm: lyng_js_env::RealmRecord,
    name: &str,
) -> Value {
    let atom = agent.atoms_mut().intern_collectible(name);
    ordinary_get(agent, realm.global_object(), PropertyKey::from_atom(atom)).unwrap()
}

fn iterator_result_fields(agent: &mut lyng_js_env::Agent, result: Value) -> (Value, Value) {
    let object = result
        .as_object_ref()
        .expect("iterator result should be an object");
    let value = ordinary_get(
        agent,
        object,
        PropertyKey::from_atom(lyng_js_common::WellKnownAtom::value.id()),
    )
    .unwrap();
    let done_atom = agent.atoms_mut().intern_collectible("done");
    let done = ordinary_get(agent, object, PropertyKey::from_atom(done_atom)).unwrap();
    (value, done)
}

#[test]
fn installed_code_reports_bytecode_executable_identity() {
    let installed = InstalledCode::new(
        CodeRef::from_raw(9).unwrap(),
        BytecodeFunctionId::from_raw(2).unwrap(),
    );

    assert_eq!(installed.code(), CodeRef::from_raw(9).unwrap());
    assert_eq!(installed.entry(), BytecodeFunctionId::from_raw(2).unwrap());
    assert_eq!(
        installed.executable(),
        ExecutableId::Bytecode(CodeRef::from_raw(9).unwrap())
    );
}

#[test]
fn install_module_records_static_metadata_on_the_agent() {
    let unit = compile_test_module(
        201,
        "import value from './dep.mjs'; export { value as forwarded };",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    let installed = vm
        .install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();

    let record = agent
        .module_record(&key)
        .expect("installed module should be cached on the agent");
    assert_eq!(installed.entry(), unit.entry());
    assert_eq!(record.display_name(), "/tmp/main.mjs");
    assert_eq!(record.requested_modules().len(), 1);
    assert_eq!(record.requested_modules()[0].specifier(), "./dep.mjs");
    assert_eq!(record.import_entries().len(), 1);
    assert_eq!(record.indirect_exports().len(), 0);
    assert_eq!(record.local_exports().len(), 1);
    assert_eq!(record.status(), ModuleStatus::Unlinked);
}

#[test]
fn evaluate_module_uses_module_environment_and_preserves_default_export_value() {
    let unit = compile_test_module(202, "export default 42;");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/default.mjs");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_module(agent, realm, &key, "/tmp/default.mjs", &unit)
        .unwrap();

    let record = agent
        .module_record(&key)
        .expect("evaluated module should stay cached on the agent");
    let module_env = record
        .environment()
        .expect("module evaluation should materialize a module environment");
    assert_eq!(result, Value::undefined());
    assert_eq!(record.status(), ModuleStatus::Evaluated);
    assert!(matches!(
        agent.environment(module_env),
        Some(lyng_js_env::EnvironmentRecord::Module(_))
    ));
    assert_eq!(
        agent.environment_slot(module_env, 0),
        Some(Value::from_smi(42))
    );
}

#[test]
fn import_meta_returns_one_cached_object_and_exposes_the_module_key() {
    let unit = compile_test_module(
        203,
        "const meta = import.meta; export const same = meta === import.meta; export default meta.url;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/import-meta.mjs");
    let mut vm = Vm::new();

    let _ = vm
        .evaluate_module(agent, realm, &key, "/tmp/import-meta.mjs", &unit)
        .unwrap();

    let record = agent
        .module_record(&key)
        .expect("evaluated module should stay cached on the agent");
    let module_env = record
        .environment()
        .expect("module evaluation should materialize a module environment");
    let same_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("same"))
        .expect("module should export same")
        .local_slot();
    let url_slot = unit
        .local_exports()
        .iter()
        .find(|entry| entry.export_name() == lyng_js_common::WellKnownAtom::default.id())
        .expect("module should export a default value")
        .local_slot();
    let url = agent
        .environment_slot(module_env, url_slot)
        .expect("default export should be initialized");
    let url = agent
        .heap()
        .view()
        .string_view(
            url.as_string_ref()
                .expect("default export should be a string"),
        )
        .map(decode_string)
        .expect("import.meta.url string should be allocated");

    assert_eq!(
        agent.environment_slot(module_env, same_slot),
        Some(Value::from_bool(true))
    );
    assert_eq!(url, "/tmp/import-meta.mjs");
    assert!(record.import_meta_object().is_some());
}

#[test]
fn evaluate_module_initializes_named_default_function_binding() {
    let unit = compile_test_module(
        204,
        "export default function F() {} export const binding = F;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/default-binding.mjs");
    let mut vm = Vm::new();

    let _ = vm
        .evaluate_module(agent, realm, &key, "/tmp/default-binding.mjs", &unit)
        .unwrap();

    let record = agent
        .module_record(&key)
        .expect("evaluated module should stay cached");
    let module_env = record
        .environment()
        .expect("module evaluation should materialize a module environment");
    let default_slot = unit
        .local_exports()
        .iter()
        .find(|entry| entry.export_name() == lyng_js_common::WellKnownAtom::default.id())
        .expect("module should export a default value")
        .local_slot();
    let binding_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("binding"))
        .expect("module should export the local binding")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, default_slot),
        agent.environment_slot(module_env, binding_slot)
    );
}

#[test]
fn linked_module_graph_initializes_anonymous_default_function_expression_exports() {
    let unit = compile_test_module(
        333,
        "export default (function() { return 99; }); import f from './main.mjs'; export const call = f(); export const name = f.name;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    assert!(agent.set_module_requested_key(&key, 0, Some(key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let call_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("call"))
        .expect("module should export call")
        .local_slot();
    let name_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("name"))
        .expect("module should export name")
        .local_slot();
    let name_value = agent
        .environment_slot(module_env, name_slot)
        .expect("name export should be initialized");
    let name_value = agent
        .heap()
        .view()
        .string_view(
            name_value
                .as_string_ref()
                .expect("name export should be a string"),
        )
        .map(decode_string)
        .expect("name export string should be allocated");

    assert_eq!(
        agent.environment_slot(module_env, call_slot),
        Some(Value::from_smi(99))
    );
    assert_eq!(name_value, "default");
}

#[test]
fn linked_module_graph_initializes_anonymous_default_class_expression_exports() {
    let unit = compile_test_module(
        334,
        "export default (class { valueOf() { return 45; } }); import C from './main.mjs'; export const value = new C().valueOf(); export const name = C.name;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    assert!(agent.set_module_requested_key(&key, 0, Some(key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let value_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("value"))
        .expect("module should export value")
        .local_slot();
    let name_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("name"))
        .expect("module should export name")
        .local_slot();
    let name_value = agent
        .environment_slot(module_env, name_slot)
        .expect("name export should be initialized");
    let name_value = agent
        .heap()
        .view()
        .string_view(
            name_value
                .as_string_ref()
                .expect("name export should be a string"),
        )
        .map(decode_string)
        .expect("name export string should be allocated");

    assert_eq!(
        agent.environment_slot(module_env, value_slot),
        Some(Value::from_smi(45))
    );
    assert_eq!(name_value, "default");
}

#[test]
fn linked_module_graph_initializes_named_default_class_exports() {
    let unit = compile_test_module(
        335,
        "export default class CName { valueOf() { return 45; } } import C from './main.mjs'; export const value = new C().valueOf(); export const name = C.name;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    assert!(agent.set_module_requested_key(&key, 0, Some(key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let value_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("value"))
        .expect("module should export value")
        .local_slot();
    let name_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("name"))
        .expect("module should export name")
        .local_slot();
    let name_value = agent
        .environment_slot(module_env, name_slot)
        .expect("name export should be initialized");
    let name_value = agent
        .heap()
        .view()
        .string_view(
            name_value
                .as_string_ref()
                .expect("name export should be a string"),
        )
        .map(decode_string)
        .expect("name export string should be allocated");

    assert_eq!(
        agent.environment_slot(module_env, value_slot),
        Some(Value::from_smi(45))
    );
    assert_eq!(name_value, "CName");
}

#[test]
fn linked_module_graph_names_anonymous_default_class_before_static_field_initializers() {
    let unit = compile_test_module(
        336,
        "let className; export default class { static f = (className = this.name); } export const observed = className;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/default-class-static-name.mjs");
    let mut vm = Vm::new();

    vm.install_module(
        agent,
        realm.id(),
        &key,
        "/tmp/default-class-static-name.mjs",
        &unit,
    )
    .unwrap();

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let observed_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("observed"))
        .expect("module should export observed")
        .local_slot();
    let observed_value = agent
        .environment_slot(module_env, observed_slot)
        .expect("observed export should be initialized")
        .as_string_ref()
        .expect("observed export should be a string");
    let observed_value = agent
        .heap()
        .view()
        .string_view(observed_value)
        .map(decode_string)
        .expect("observed export string should be allocated");

    assert_eq!(observed_value, "default");
}

#[test]
fn linked_module_graph_hoists_named_default_function_for_self_imports() {
    let unit = compile_test_module(
        218,
        "import f from './main.mjs'; export default function fName() { return 23; } export const seen = f();",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    for request_index in 0..unit.requested_modules().len() {
        assert!(agent.set_module_requested_key(
            &key,
            u32::try_from(request_index).expect("request index should fit into u32"),
            Some(key.clone()),
        ));
    }

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let seen_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("seen"))
        .expect("module should export seen")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, seen_slot),
        Some(Value::from_smi(23))
    );
}

#[test]
fn linked_module_graph_keeps_named_default_function_exports_live() {
    let dependency = compile_test_module(219, "export default function fn() { fn = 2; return 1; }");
    let importer = compile_test_module(
        220,
        "import val from './dep.mjs'; export const first = val(); export default val;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let dependency_key = ModuleKey::new("/tmp/dep.mjs");
    let importer_key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(
        agent,
        realm.id(),
        &dependency_key,
        "/tmp/dep.mjs",
        &dependency,
    )
    .unwrap();
    vm.install_module(agent, realm.id(), &importer_key, "/tmp/main.mjs", &importer)
        .unwrap();
    assert!(agent.set_module_requested_key(&importer_key, 0, Some(dependency_key.clone())));

    let _ = vm
        .evaluate_linked_module(agent, realm, &importer_key)
        .unwrap();

    let importer_record = agent
        .module_record(&importer_key)
        .expect("importer module should stay cached");
    let importer_env = importer_record
        .environment()
        .expect("importer evaluation should allocate one environment");
    let first_slot = importer
        .local_exports()
        .iter()
        .find(|entry| importer.atom_text(entry.export_name()) == Some("first"))
        .expect("importer should export first")
        .local_slot();
    let default_slot = importer
        .local_exports()
        .iter()
        .find(|entry| entry.export_name() == lyng_js_common::WellKnownAtom::default.id())
        .expect("importer should export a default value")
        .local_slot();

    assert_eq!(
        agent.environment_slot(importer_env, first_slot),
        Some(Value::from_smi(1))
    );
    assert_eq!(
        agent.environment_slot(importer_env, default_slot),
        Some(Value::from_smi(2))
    );
}

#[test]
fn host_module_loader_recurses_through_attributes_and_import_meta() {
    let host = TestHost::new();
    host.define_module_source(
        "entry.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/entry.mjs"),
            "entry.mjs",
            "import dep from './dep.mjs' with { type: 'js' }; export default dep + '|' + import.meta.kind;",
        ),
    );
    host.define_module_source(
        "./dep.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/dep.mjs"),
            "dep.mjs",
            "export default import.meta.url;",
        ),
    );
    host.define_import_meta(
        ModuleKey::new("/tmp/entry.mjs"),
        ImportMetaProperties::new(vec![ImportMetaProperty {
            key: "kind".into(),
            value: ImportMetaValue::String("entry".into()),
        }]),
    );
    host.define_import_meta(
        ModuleKey::new("/tmp/dep.mjs"),
        ImportMetaProperties::new(vec![ImportMetaProperty {
            key: "url".into(),
            value: ImportMetaValue::String("host:dep".into()),
        }]),
    );

    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let loaded = vm
        .load_module_graph_from_host(
            agent,
            realm,
            &host,
            &ModuleSourceRequest {
                specifier: "entry.mjs".into(),
                referrer: None,
                attributes: Vec::new(),
            },
        )
        .unwrap();
    let _ = vm
        .evaluate_linked_module(agent, realm, loaded.key())
        .expect("loaded host-backed module graph should evaluate");

    let entry = agent
        .module_record(loaded.key())
        .expect("entry module should be cached");
    let module_env = entry
        .environment()
        .expect("entry module should allocate an environment");
    let default_slot = compile_test_module(
        204,
        "import dep from './dep.mjs' with { type: 'js' }; export default dep + '|' + import.meta.kind;",
    )
    .local_exports()
    .iter()
    .find(|entry| entry.export_name() == lyng_js_common::WellKnownAtom::default.id())
    .expect("entry module should export a default value")
    .local_slot();
    let value = agent
        .environment_slot(module_env, default_slot)
        .expect("default export should be initialized");
    let value = agent
        .heap()
        .view()
        .string_view(
            value
                .as_string_ref()
                .expect("default export should be a string"),
        )
        .map(decode_string)
        .expect("default export string should be allocated");

    assert_eq!(loaded.display_name(), "entry.mjs");
    assert_eq!(value, "host:dep|entry");
    assert_eq!(
        host.snapshot().calls,
        vec![
            lyng_js_host::HostCall::LoadModule(ModuleSourceRequest {
                specifier: "entry.mjs".into(),
                referrer: None,
                attributes: Vec::new(),
            }),
            lyng_js_host::HostCall::LoadModule(ModuleSourceRequest {
                specifier: "./dep.mjs".into(),
                referrer: Some(ModuleKey::new("/tmp/entry.mjs")),
                attributes: vec![lyng_js_host::ModuleImportAttribute {
                    key: "type".into(),
                    value: "js".into(),
                }],
            }),
            lyng_js_host::HostCall::ResolveImportMeta(lyng_js_host::ImportMetaRequest {
                module: ModuleKey::new("/tmp/dep.mjs"),
            }),
            lyng_js_host::HostCall::ResolveImportMeta(lyng_js_host::ImportMetaRequest {
                module: ModuleKey::new("/tmp/entry.mjs"),
            }),
        ]
    );
}

#[test]
fn host_module_loader_keeps_named_default_function_exports_live() {
    let host = TestHost::new();
    host.define_module_source(
        "entry.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/entry.mjs"),
            "entry.mjs",
            "import val from './dep.mjs'; export const first = val(); export default val;",
        ),
    );
    host.define_module_source(
        "./dep.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/dep.mjs"),
            "dep.mjs",
            "export default function fn() { fn = 2; return 1; }",
        ),
    );

    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let loaded = vm
        .load_module_graph_from_host(
            agent,
            realm,
            &host,
            &ModuleSourceRequest {
                specifier: "entry.mjs".into(),
                referrer: None,
                attributes: Vec::new(),
            },
        )
        .unwrap();
    let _ = vm
        .evaluate_linked_module(agent, realm, loaded.key())
        .unwrap();

    let entry = agent
        .module_record(loaded.key())
        .expect("entry module should stay cached");
    let module_env = entry
        .environment()
        .expect("entry module should allocate an environment");
    let unit = compile_test_module(
        225,
        "import val from './dep.mjs'; export const first = val(); export default val;",
    );
    let first_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("first"))
        .expect("entry module should export first")
        .local_slot();
    let default_slot = unit
        .local_exports()
        .iter()
        .find(|entry| entry.export_name() == lyng_js_common::WellKnownAtom::default.id())
        .expect("entry module should export a default value")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, first_slot),
        Some(Value::from_smi(1))
    );
    assert_eq!(
        agent.environment_slot(module_env, default_slot),
        Some(Value::from_smi(2))
    );
}

#[test]
fn module_namespace_object_exposes_symbol_to_string_tag() {
    let unit = compile_test_module(
        226,
        "import * as ns from './main.mjs'; export const ok = ns[Symbol.toStringTag] === 'Module' && Object.getOwnPropertyDescriptor(ns, Symbol.toStringTag).enumerable === false && Object.getOwnPropertyDescriptor(ns, Symbol.toStringTag).writable === false && Object.getOwnPropertyDescriptor(ns, Symbol.toStringTag).configurable === false; export default null;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    for request_index in 0..unit.requested_modules().len() {
        assert!(agent.set_module_requested_key(
            &key,
            u32::try_from(request_index).expect("request index should fit into u32"),
            Some(key.clone()),
        ));
    }

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let ok_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("ok"))
        .expect("module should export ok")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, ok_slot),
        Some(Value::from_bool(true))
    );
}

#[test]
fn module_namespace_reflect_define_accepts_matching_to_string_tag_descriptor() {
    let source = "import * as ns from './main.mjs'; let status = 0; const tag = { value: 'Module', writable: false, enumerable: false, configurable: false }; if (Reflect.defineProperty(ns, Symbol.toStringTag, tag) === true) status += 1; try { if (Object.defineProperty(ns, Symbol.toStringTag, tag) === ns) status += 2; } catch (error) { status += 8; } export { status }; export default null;";
    let unit = compile_test_module(227, source);
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    assert!(agent.set_module_requested_key(&key, 0, Some(key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let status_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("status"))
        .expect("module should export status")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, status_slot),
        Some(Value::from_smi(3))
    );
}

#[test]
fn module_namespace_reflect_set_rejects_same_export_and_symbol_values() {
    let source = "import * as ns from './main.mjs'; export default 42; const receiver = {}; export const status = (Reflect.set(ns, 'default', 42) === false ? 1 : 0) + (Reflect.set(ns, Symbol.toStringTag, ns[Symbol.toStringTag]) === false ? 2 : 0) + (Reflect.set(ns, 'missing', 1, receiver) === false && !('missing' in receiver) ? 4 : 0);";
    let unit = compile_test_module(228, source);
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    assert!(agent.set_module_requested_key(&key, 0, Some(key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let status_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("status"))
        .expect("module should export status")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, status_slot),
        Some(Value::from_smi(7))
    );
}

#[test]
fn module_namespace_get_own_property_throws_for_uninitialized_binding() {
    let unit = compile_test_module(
        229,
        "import * as ns from './main.mjs'; let status = 0; try { Object.getOwnPropertyDescriptor(ns, 'local'); status = 1; } catch (error) { status = error.constructor === ReferenceError ? 2 : 3; } export { status }; export let local = 1;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    assert!(agent.set_module_requested_key(&key, 0, Some(key.clone())));
    let module_env = vm.link_module(agent, realm, &key).unwrap();
    let local_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("local"))
        .expect("module should export local")
        .local_slot();
    assert_eq!(
        agent.environment_slot(module_env, local_slot),
        Some(Value::uninitialized_lexical())
    );

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let status_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("status"))
        .expect("module should export status")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, status_slot),
        Some(Value::from_smi(2))
    );
}

#[test]
fn module_namespace_has_own_property_throws_for_uninitialized_binding() {
    let unit = compile_test_module(
        230,
        "import * as ns from './main.mjs'; let status = 0; try { Object.prototype.hasOwnProperty.call(ns, 'local'); status = 1; } catch (error) { status = error.constructor === ReferenceError ? 2 : 3; } export { status }; export let local = 1;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    assert!(agent.set_module_requested_key(&key, 0, Some(key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let status_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("status"))
        .expect("module should export status")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, status_slot),
        Some(Value::from_smi(2))
    );
}

#[test]
fn module_namespace_has_property_does_not_read_uninitialized_binding() {
    let unit = compile_test_module(
        252,
        "import * as ns from './main.mjs'; let status = 0; try { if ('local' in ns) status += 1; if (Reflect.has(ns, 'local')) status += 2; } catch (error) { status = error.constructor === ReferenceError ? 8 : 16; } export { status }; export let local = 1;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    assert!(agent.set_module_requested_key(&key, 0, Some(key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let status_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("status"))
        .expect("module should export status")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, status_slot),
        Some(Value::from_smi(3))
    );
}

#[test]
fn module_namespace_reads_initialized_local_renamed_indirect_and_default_exports() {
    let source = "import * as ns from './main.mjs'; export var local1 = 23; var local2 = 45; export { local2 as renamed }; export { local1 as indirect } from './main.mjs'; export default 444; export const status = (ns.local1 === 23 ? 1 : 0) + (ns.renamed === 45 ? 2 : 0) + (ns.indirect === 23 ? 4 : 0) + (ns.default === 444 ? 8 : 0);";
    let host = TestHost::new();
    host.define_module_source(
        "main.mjs",
        LoadedModuleSource::new(ModuleKey::new("/tmp/main.mjs"), "main.mjs", source),
    );
    host.define_module_source(
        "./main.mjs",
        LoadedModuleSource::new(ModuleKey::new("/tmp/main.mjs"), "main.mjs", source),
    );
    let unit = compile_test_module(232, source);
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let loaded = vm
        .load_module_graph_from_host(
            agent,
            realm,
            &host,
            &ModuleSourceRequest {
                specifier: "main.mjs".into(),
                referrer: None,
                attributes: Vec::new(),
            },
        )
        .unwrap();

    let _ = vm
        .evaluate_linked_module(agent, realm, loaded.key())
        .unwrap();

    let record = agent
        .module_record(loaded.key())
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let status_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("status"))
        .expect("module should export status")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, status_slot),
        Some(Value::from_smi(15))
    );
}

#[test]
fn module_namespace_numeric_export_names_match_array_index_keys() {
    let source = "import * as ns from './main.mjs'; var a = 0; var b = 1; export { a as \"0\", b as \"1\" }; let status = 0; if (ns[0] === 0) status += 1; if (Reflect.get(ns, 1) === 1) status += 2; if (0 in ns) status += 4; if (Reflect.has(ns, 1)) status += 8; if (!Reflect.set(ns, 1, 3)) status += 16; try { delete ns[0]; } catch (error) { if (error.constructor === TypeError) status += 32; } if (!Reflect.deleteProperty(ns, 1)) status += 64; export { status };";
    let host = TestHost::new();
    host.define_module_source(
        "main.mjs",
        LoadedModuleSource::new(ModuleKey::new("/tmp/main.mjs"), "main.mjs", source),
    );
    host.define_module_source(
        "./main.mjs",
        LoadedModuleSource::new(ModuleKey::new("/tmp/main.mjs"), "main.mjs", source),
    );
    let unit = compile_test_module(251, source);
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let loaded = vm
        .load_module_graph_from_host(
            agent,
            realm,
            &host,
            &ModuleSourceRequest {
                specifier: "main.mjs".into(),
                referrer: None,
                attributes: Vec::new(),
            },
        )
        .unwrap();

    let _ = vm
        .evaluate_linked_module(agent, realm, loaded.key())
        .unwrap();

    let record = agent
        .module_record(loaded.key())
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let status_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("status"))
        .expect("module should export status")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, status_slot),
        Some(Value::from_smi(127))
    );
}

#[test]
fn module_namespace_get_own_property_reports_initialized_exports() {
    let source = "import * as ns from './main.mjs'; export var local1 = 201; var local2 = 207; export { local2 as renamed }; export { local1 as indirect } from './main.mjs'; export default 302; let status = 0; let desc = Object.getOwnPropertyDescriptor(ns, 'local1'); if (Object.prototype.hasOwnProperty.call(ns, 'local1') && desc.value === 201 && desc.enumerable === true && desc.writable === true && desc.configurable === false) status += 1; desc = Object.getOwnPropertyDescriptor(ns, 'renamed'); if (Object.prototype.hasOwnProperty.call(ns, 'renamed') && desc.value === 207 && desc.enumerable === true && desc.writable === true && desc.configurable === false) status += 2; desc = Object.getOwnPropertyDescriptor(ns, 'indirect'); if (Object.prototype.hasOwnProperty.call(ns, 'indirect') && desc.value === 201 && desc.enumerable === true && desc.writable === true && desc.configurable === false) status += 4; desc = Object.getOwnPropertyDescriptor(ns, 'default'); if (Object.prototype.hasOwnProperty.call(ns, 'default') && desc.value === 302 && desc.enumerable === true && desc.writable === true && desc.configurable === false) status += 8; export { status };";
    let host = TestHost::new();
    host.define_module_source(
        "main.mjs",
        LoadedModuleSource::new(ModuleKey::new("/tmp/main.mjs"), "main.mjs", source),
    );
    host.define_module_source(
        "./main.mjs",
        LoadedModuleSource::new(ModuleKey::new("/tmp/main.mjs"), "main.mjs", source),
    );
    let unit = compile_test_module(233, source);
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let loaded = vm
        .load_module_graph_from_host(
            agent,
            realm,
            &host,
            &ModuleSourceRequest {
                specifier: "main.mjs".into(),
                referrer: None,
                attributes: Vec::new(),
            },
        )
        .unwrap();

    let _ = vm
        .evaluate_linked_module(agent, realm, loaded.key())
        .unwrap();

    let record = agent
        .module_record(loaded.key())
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let status_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("status"))
        .expect("module should export status")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, status_slot),
        Some(Value::from_smi(15))
    );
}

#[test]
fn module_namespace_object_keys_throw_for_uninitialized_binding() {
    let unit = compile_test_module(
        228,
        "import * as ns from './main.mjs'; let status = 0; try { Object.keys(ns); status = 1; } catch (error) { status = error.constructor === ReferenceError ? 2 : 3; } export { status }; export default 0;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    assert!(agent.set_module_requested_key(&key, 0, Some(key.clone())));
    let module_env = vm.link_module(agent, realm, &key).unwrap();
    let default_slot = unit
        .local_exports()
        .iter()
        .find(|entry| entry.export_name() == lyng_js_common::WellKnownAtom::default.id())
        .expect("module should export default")
        .local_slot();
    assert_eq!(
        agent.environment_slot(module_env, default_slot),
        Some(Value::uninitialized_lexical())
    );

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let status_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("status"))
        .expect("module should export status")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, status_slot),
        Some(Value::from_smi(2))
    );
}

#[test]
fn captured_import_binding_throws_reference_error_before_export_let_initializes() {
    let unit = compile_test_module(
        229,
        "import { x as y } from './main.mjs'; let status = 0; (function() { try { typeof y; status = 1; } catch (error) { status = error.constructor === ReferenceError ? 2 : 3; } })(); export { status }; export let x = 23;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    assert!(agent.set_module_requested_key(&key, 0, Some(key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let status_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("status"))
        .expect("module should export status")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, status_slot),
        Some(Value::from_smi(2))
    );
}

#[test]
fn captured_import_binding_resolves_before_import_declaration_position() {
    let unit = compile_test_module(
        231,
        "let status = 0; (function() { try { typeof y; status = 1; } catch (error) { status = error.constructor === ReferenceError ? 2 : 3; } })(); import { x as y } from './main.mjs'; export { status }; export let x = 23;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    assert!(agent.set_module_requested_key(&key, 0, Some(key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let status_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("status"))
        .expect("module should export status")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, status_slot),
        Some(Value::from_smi(2))
    );
}

#[test]
fn linked_module_graph_keeps_named_import_bindings_live() {
    let dependency = compile_test_module(204, "export let counter = 1;");
    let importer = compile_test_module(
        205,
        "import { counter } from './dep.mjs'; export { counter as seen };",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let dependency_key = ModuleKey::new("/tmp/dep.mjs");
    let importer_key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(
        agent,
        realm.id(),
        &dependency_key,
        "/tmp/dep.mjs",
        &dependency,
    )
    .unwrap();
    vm.install_module(agent, realm.id(), &importer_key, "/tmp/main.mjs", &importer)
        .unwrap();
    assert!(agent.set_module_requested_key(&importer_key, 0, Some(dependency_key.clone())));

    let result = vm
        .evaluate_linked_module(agent, realm, &importer_key)
        .unwrap();

    let dependency_record = agent
        .module_record(&dependency_key)
        .expect("dependency module should stay cached");
    let dependency_env = dependency_record
        .environment()
        .expect("dependency evaluation should allocate one environment");
    let importer_record = agent
        .module_record(&importer_key)
        .expect("importer module should stay cached");
    let importer_env = importer_record
        .environment()
        .expect("importer evaluation should allocate one environment");
    let counter_export_slot = dependency.local_exports()[0].local_slot();
    let counter_import_slot = importer.import_entries()[0].local_slot();
    let seen_export_slot = importer.local_exports()[0].local_slot();

    assert_eq!(result, Value::undefined());
    assert_eq!(dependency_record.status(), ModuleStatus::Evaluated);
    assert_eq!(importer_record.status(), ModuleStatus::Evaluated);
    assert_eq!(
        agent.module_binding_alias(importer_env, counter_import_slot),
        Some(lyng_js_env::ModuleBindingAlias::new(
            dependency_env,
            counter_export_slot,
        ))
    );
    assert_eq!(
        agent.environment_slot(dependency_env, counter_export_slot),
        Some(Value::from_smi(1))
    );
    assert_eq!(
        agent.environment_slot(importer_env, counter_import_slot),
        Some(Value::from_smi(1))
    );
    assert_eq!(
        agent.environment_slot(importer_env, seen_export_slot),
        Some(Value::from_smi(1))
    );

    assert!(agent.set_environment_slot(dependency_env, counter_export_slot, Value::from_smi(6),));
    assert_eq!(
        agent.environment_slot(importer_env, counter_import_slot),
        Some(Value::from_smi(6))
    );
    assert_eq!(
        agent.environment_slot(importer_env, seen_export_slot),
        Some(Value::from_smi(6))
    );
}

#[test]
fn linked_module_graph_rejects_assignment_to_imported_function_bindings() {
    let unit = compile_test_module(
        221,
        "import { f as f2 } from './main.mjs'; let status = 0; try { f2 = null; } catch (error) { status = error.constructor === TypeError ? 1 : 2; } export { status }; export function f() { return 23; }",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    assert!(agent.set_module_requested_key(&key, 0, Some(key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let status_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("status"))
        .expect("module should export status")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, status_slot),
        Some(Value::from_smi(1))
    );
}

#[test]
fn link_module_graph_initializes_module_var_bindings_to_undefined() {
    let unit = compile_test_module(
        222,
        "import { x as y } from './main.mjs'; export var x = 23; export const snapshot = y;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    assert!(agent.set_module_requested_key(&key, 0, Some(key.clone())));

    let module_env = vm.link_module(agent, realm, &key).unwrap();
    let import_slot = unit.import_entries()[0].local_slot();

    assert_eq!(
        agent.environment_slot(module_env, import_slot),
        Some(Value::undefined())
    );
}

#[test]
fn evaluate_module_reads_exported_var_before_declaration_as_undefined() {
    let unit = compile_test_module(
        253,
        "let status = 0; try { if (test262 === undefined) status += 1; test262 = null; if (test262 === null) status += 2; } catch (error) { status = error.constructor === ReferenceError ? 8 : 16; } export { status }; export var test262 = 23;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let status_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("status"))
        .expect("module should export status")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, status_slot),
        Some(Value::from_smi(3))
    );
}

#[test]
fn evaluate_module_reads_for_var_before_declaration_as_undefined() {
    let unit = compile_test_module(
        254,
        "let status = 0; try { if (test262 === undefined) status += 1; for (var test262 = null; false;) {} if (test262 === null) status += 2; } catch (error) { status = error.constructor === ReferenceError ? 8 : 16; } export { status };",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let status_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("status"))
        .expect("module should export status")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, status_slot),
        Some(Value::from_smi(3))
    );
}

#[test]
fn linked_module_graph_supports_default_and_named_self_import_instantiation() {
    let unit = compile_test_module(
        223,
        "import x, { attr as y } from './main.mjs'; let default_status = 0; try { typeof x; default_status = 1; } catch (error) { default_status = error.constructor === ReferenceError ? 2 : 3; } export { default_status }; export const named_is_undef = y === undefined; export default 3; export var attr;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    assert!(agent.set_module_requested_key(&key, 0, Some(key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let default_status_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("default_status"))
        .expect("module should export default_status")
        .local_slot();
    let named_is_undef_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("named_is_undef"))
        .expect("module should export named_is_undef")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, default_status_slot),
        Some(Value::from_smi(2))
    );
    assert_eq!(
        agent.environment_slot(module_env, named_is_undef_slot),
        Some(Value::from_bool(true))
    );
}

#[test]
fn linked_module_graph_supports_named_default_function_self_imports() {
    let unit = compile_test_module(
        224,
        "import f from './main.mjs'; export const call = f(); export const name = f.name; export default function fName() { return 23; }",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    assert!(agent.set_module_requested_key(&key, 0, Some(key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let call_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("call"))
        .expect("module should export call")
        .local_slot();
    let name_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("name"))
        .expect("module should export name")
        .local_slot();
    let name_value = agent
        .environment_slot(module_env, name_slot)
        .expect("name export should be initialized");
    let name_value = agent
        .heap()
        .view()
        .string_view(
            name_value
                .as_string_ref()
                .expect("name export should be a string"),
        )
        .map(decode_string)
        .expect("name export string should be allocated");

    assert_eq!(
        agent.environment_slot(module_env, call_slot),
        Some(Value::from_smi(23))
    );
    assert_eq!(name_value, "fName");
}

#[test]
fn linked_module_graph_initializes_named_exported_functions_before_evaluation() {
    let unit = compile_test_module(
        330,
        "import { f as f2 } from './main.mjs'; export const call = f2(); export function f() { return 23; }",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    assert!(agent.set_module_requested_key(&key, 0, Some(key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let call_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("call"))
        .expect("module should export call")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, call_slot),
        Some(Value::from_smi(23))
    );
}

#[test]
fn linked_module_graph_initializes_exported_functions_before_dependency_evaluation() {
    let dependency = compile_test_module(
        349,
        "import { f } from './main.mjs'; export const call = f();",
    );
    let main = compile_test_module(
        350,
        "import { call } from './dep.mjs'; export const seen = call; export function f() { return 23; }",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let dependency_key = ModuleKey::new("/tmp/dep.mjs");
    let main_key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(
        agent,
        realm.id(),
        &dependency_key,
        "/tmp/dep.mjs",
        &dependency,
    )
    .unwrap();
    vm.install_module(agent, realm.id(), &main_key, "/tmp/main.mjs", &main)
        .unwrap();
    assert!(agent.set_module_requested_key(&dependency_key, 0, Some(main_key.clone())));
    assert!(agent.set_module_requested_key(&main_key, 0, Some(dependency_key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &main_key).unwrap();

    let record = agent
        .module_record(&main_key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let seen_slot = main
        .local_exports()
        .iter()
        .find(|entry| main.atom_text(entry.export_name()) == Some("seen"))
        .expect("module should export seen")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, seen_slot),
        Some(Value::from_smi(23))
    );
}

#[test]
fn linked_module_graph_initializes_indirectly_exported_functions_before_evaluation() {
    let dependency = compile_test_module(331, "export { A as B } from './main.mjs';");
    let main = compile_test_module(
        332,
        "import { B } from './dep.mjs'; export const call = B(); export function A() { return 77; }",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let dependency_key = ModuleKey::new("/tmp/dep.mjs");
    let main_key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(
        agent,
        realm.id(),
        &dependency_key,
        "/tmp/dep.mjs",
        &dependency,
    )
    .unwrap();
    vm.install_module(agent, realm.id(), &main_key, "/tmp/main.mjs", &main)
        .unwrap();
    assert!(agent.set_module_requested_key(&dependency_key, 0, Some(main_key.clone())));
    assert!(agent.set_module_requested_key(&main_key, 0, Some(dependency_key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &main_key).unwrap();

    let record = agent
        .module_record(&main_key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    let call_slot = main
        .local_exports()
        .iter()
        .find(|entry| main.atom_text(entry.export_name()) == Some("call"))
        .expect("module should export call")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, call_slot),
        Some(Value::from_smi(77))
    );
}

#[test]
fn module_namespace_reads_throw_for_uninitialized_self_reexported_bindings() {
    let unit = compile_test_module(
        336,
        "import * as ns from './main.mjs'; \
         let local_status = 0; \
         try { ns.local1; local_status = 1; } catch (error) { local_status = error.constructor === ReferenceError ? 2 : 3; } \
         let renamed_status = 0; \
         try { ns.renamed; renamed_status = 1; } catch (error) { renamed_status = error.constructor === ReferenceError ? 2 : 3; } \
         let indirect_status = 0; \
         try { ns.indirect; indirect_status = 1; } catch (error) { indirect_status = error.constructor === ReferenceError ? 2 : 3; } \
         let default_status = 0; \
         try { ns.default; default_status = 1; } catch (error) { default_status = error.constructor === ReferenceError ? 2 : 3; } \
         export { local_status, renamed_status, indirect_status, default_status }; \
         export let local1 = 23; \
         let local2 = 45; \
         export { local2 as renamed }; \
         export { local1 as indirect } from './main.mjs'; \
         export default null;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &key, "/tmp/main.mjs", &unit)
        .unwrap();
    assert!(agent.set_module_requested_key(&key, 0, Some(key.clone())));
    assert!(agent.set_module_requested_key(&key, 1, Some(key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &key).unwrap();

    let record = agent
        .module_record(&key)
        .expect("module should stay cached after evaluation");
    let module_env = record
        .environment()
        .expect("module evaluation should allocate one environment");
    for export_name in [
        "local_status",
        "renamed_status",
        "indirect_status",
        "default_status",
    ] {
        let slot = unit
            .local_exports()
            .iter()
            .find(|entry| unit.atom_text(entry.export_name()) == Some(export_name))
            .expect("status export should exist")
            .local_slot();
        assert_eq!(
            agent.environment_slot(module_env, slot),
            Some(Value::from_smi(2)),
            "{export_name} should observe a ReferenceError",
        );
    }
}

#[test]
fn linked_module_graph_treats_mixed_shared_namespace_star_exports_as_unambiguous() {
    let empty = compile_test_module(337, "");
    let left = compile_test_module(338, "export * as foo from './empty.mjs';");
    let right = compile_test_module(339, "import * as foo from './empty.mjs'; export { foo };");
    let main = compile_test_module(
        340,
        "export * from './left.mjs'; export * from './right.mjs'; import { foo } from './main.mjs'; export default foo;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let empty_key = ModuleKey::new("/tmp/empty.mjs");
    let left_key = ModuleKey::new("/tmp/left.mjs");
    let right_key = ModuleKey::new("/tmp/right.mjs");
    let main_key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &empty_key, "/tmp/empty.mjs", &empty)
        .unwrap();
    vm.install_module(agent, realm.id(), &left_key, "/tmp/left.mjs", &left)
        .unwrap();
    vm.install_module(agent, realm.id(), &right_key, "/tmp/right.mjs", &right)
        .unwrap();
    vm.install_module(agent, realm.id(), &main_key, "/tmp/main.mjs", &main)
        .unwrap();
    assert!(agent.set_module_requested_key(&left_key, 0, Some(empty_key.clone())));
    assert!(agent.set_module_requested_key(&right_key, 0, Some(empty_key.clone())));
    assert!(agent.set_module_requested_key(&main_key, 0, Some(left_key.clone())));
    assert!(agent.set_module_requested_key(&main_key, 1, Some(right_key.clone())));
    assert!(agent.set_module_requested_key(&main_key, 2, Some(main_key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &main_key).unwrap();

    let record = agent
        .module_record(&main_key)
        .expect("main module should stay cached");
    let module_env = record
        .environment()
        .expect("main module should allocate one environment");
    let default_slot = main
        .local_exports()
        .iter()
        .find(|entry| entry.export_name() == lyng_js_common::WellKnownAtom::default.id())
        .expect("main module should export a default value")
        .local_slot();

    assert!(
        agent
            .environment_slot(module_env, default_slot)
            .and_then(Value::as_object_ref)
            .is_some(),
        "mixed shared namespace exports should resolve to one namespace object"
    );
}

#[test]
fn linked_module_graph_imports_self_importing_namespace_module_through_mixed_star_exports() {
    let empty = compile_test_module(344, "");
    let export_star_as = compile_test_module(345, "export * as foo from './empty.mjs';");
    let import_star_as_one =
        compile_test_module(346, "import * as foo from './empty.mjs'; export { foo };");
    let import_star_as_two =
        compile_test_module(347, "import * as foo from './empty.mjs'; export { foo };");
    let dependency = compile_test_module(
        348,
        "export * from './right1.mjs'; export * from './right2.mjs'; import { foo } from './dep.mjs'; export const dep_status = typeof foo;",
    );
    let main = compile_test_module(
        349,
        "export * from './left.mjs'; export * from './right1.mjs'; import { foo } from './dep.mjs'; export const main_status = typeof foo;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let empty_key = ModuleKey::new("/tmp/empty.mjs");
    let left_key = ModuleKey::new("/tmp/left.mjs");
    let right_one_key = ModuleKey::new("/tmp/right1.mjs");
    let right_two_key = ModuleKey::new("/tmp/right2.mjs");
    let dependency_key = ModuleKey::new("/tmp/dep.mjs");
    let main_key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &empty_key, "/tmp/empty.mjs", &empty)
        .unwrap();
    vm.install_module(
        agent,
        realm.id(),
        &left_key,
        "/tmp/left.mjs",
        &export_star_as,
    )
    .unwrap();
    vm.install_module(
        agent,
        realm.id(),
        &right_one_key,
        "/tmp/right1.mjs",
        &import_star_as_one,
    )
    .unwrap();
    vm.install_module(
        agent,
        realm.id(),
        &right_two_key,
        "/tmp/right2.mjs",
        &import_star_as_two,
    )
    .unwrap();
    vm.install_module(
        agent,
        realm.id(),
        &dependency_key,
        "/tmp/dep.mjs",
        &dependency,
    )
    .unwrap();
    vm.install_module(agent, realm.id(), &main_key, "/tmp/main.mjs", &main)
        .unwrap();

    assert!(agent.set_module_requested_key(&left_key, 0, Some(empty_key.clone())));
    assert!(agent.set_module_requested_key(&right_one_key, 0, Some(empty_key.clone())));
    assert!(agent.set_module_requested_key(&right_two_key, 0, Some(empty_key.clone())));
    assert!(agent.set_module_requested_key(&dependency_key, 0, Some(right_one_key.clone())));
    assert!(agent.set_module_requested_key(&dependency_key, 1, Some(right_two_key.clone())));
    assert!(agent.set_module_requested_key(&dependency_key, 2, Some(dependency_key.clone())));
    assert!(agent.set_module_requested_key(&main_key, 0, Some(left_key.clone())));
    assert!(agent.set_module_requested_key(&main_key, 1, Some(right_one_key.clone())));
    assert!(agent.set_module_requested_key(&main_key, 2, Some(dependency_key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &main_key).unwrap();

    let dependency_record = agent
        .module_record(&dependency_key)
        .expect("dependency module should stay cached");
    let dependency_env = dependency_record
        .environment()
        .expect("dependency module should allocate one environment");
    let dep_status_slot = dependency
        .local_exports()
        .iter()
        .find(|entry| dependency.atom_text(entry.export_name()) == Some("dep_status"))
        .expect("dependency should export dep_status")
        .local_slot();
    let dep_status = agent
        .environment_slot(dependency_env, dep_status_slot)
        .expect("dep_status should be initialized");
    let dep_status = agent
        .heap()
        .view()
        .string_view(
            dep_status
                .as_string_ref()
                .expect("dep_status should be a string"),
        )
        .map(decode_string)
        .expect("dep_status string should be allocated");

    let main_record = agent
        .module_record(&main_key)
        .expect("main module should stay cached");
    let main_env = main_record
        .environment()
        .expect("main module should allocate one environment");
    let main_status_slot = main
        .local_exports()
        .iter()
        .find(|entry| main.atom_text(entry.export_name()) == Some("main_status"))
        .expect("main should export main_status")
        .local_slot();
    let main_status = agent
        .environment_slot(main_env, main_status_slot)
        .expect("main_status should be initialized");
    let main_status = agent
        .heap()
        .view()
        .string_view(
            main_status
                .as_string_ref()
                .expect("main_status should be a string"),
        )
        .map(decode_string)
        .expect("main_status string should be allocated");

    assert_eq!(dep_status, "object");
    assert_eq!(main_status, "object");
}

#[test]
fn linked_module_graph_enumerates_circular_star_exports_without_recursing_forever() {
    let module_a = compile_test_module(341, "export * from './b.mjs'; export const fromA = 1;");
    let module_b = compile_test_module(342, "export * from './a.mjs'; export const fromB = 1;");
    let importer = compile_test_module(
        343,
        "import * as a from './a.mjs'; import * as b from './b.mjs'; \
         export const a_has_from_a = 'fromA' in a; \
         export const a_has_from_b = 'fromB' in a; \
         export const b_has_from_a = 'fromA' in b; \
         export const b_has_from_b = 'fromB' in b;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let a_key = ModuleKey::new("/tmp/a.mjs");
    let b_key = ModuleKey::new("/tmp/b.mjs");
    let importer_key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &a_key, "/tmp/a.mjs", &module_a)
        .unwrap();
    vm.install_module(agent, realm.id(), &b_key, "/tmp/b.mjs", &module_b)
        .unwrap();
    vm.install_module(agent, realm.id(), &importer_key, "/tmp/main.mjs", &importer)
        .unwrap();
    assert!(agent.set_module_requested_key(&a_key, 0, Some(b_key.clone())));
    assert!(agent.set_module_requested_key(&b_key, 0, Some(a_key.clone())));
    assert!(agent.set_module_requested_key(&importer_key, 0, Some(a_key.clone())));
    assert!(agent.set_module_requested_key(&importer_key, 1, Some(b_key.clone())));

    let _ = vm
        .evaluate_linked_module(agent, realm, &importer_key)
        .unwrap();

    let record = agent
        .module_record(&importer_key)
        .expect("importer module should stay cached");
    let module_env = record
        .environment()
        .expect("importer module should allocate one environment");
    for export_name in [
        "a_has_from_a",
        "a_has_from_b",
        "b_has_from_a",
        "b_has_from_b",
    ] {
        let slot = importer
            .local_exports()
            .iter()
            .find(|entry| importer.atom_text(entry.export_name()) == Some(export_name))
            .expect("boolean export should exist")
            .local_slot();
        assert_eq!(
            agent.environment_slot(module_env, slot),
            Some(Value::from_bool(true)),
            "{export_name} should be present in the namespace",
        );
    }
}

#[test]
fn link_module_graph_reports_unresolved_indirect_exports_before_evaluation() {
    let dependency = compile_test_module(208, "export const present = 1;");
    let exporter = compile_test_module(209, "export { missing as value } from './dep.mjs';");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let dependency_key = ModuleKey::new("/tmp/dep.mjs");
    let exporter_key = ModuleKey::new("/tmp/reexport.mjs");
    let mut vm = Vm::new();

    vm.install_module(
        agent,
        realm.id(),
        &dependency_key,
        "/tmp/dep.mjs",
        &dependency,
    )
    .unwrap();
    vm.install_module(
        agent,
        realm.id(),
        &exporter_key,
        "/tmp/reexport.mjs",
        &exporter,
    )
    .unwrap();
    assert!(agent.set_module_requested_key(&exporter_key, 0, Some(dependency_key.clone())));

    assert_eq!(
        vm.link_module(agent, realm, &exporter_key),
        Err(VmError::MissingModuleResolution)
    );
}

#[test]
fn linked_module_graph_omits_ambiguous_star_exports_from_namespaces() {
    let left = compile_test_module(210, "export const value = 1;");
    let right = compile_test_module(211, "export const value = 2;");
    let exporter = compile_test_module(
        212,
        "export * from './left.mjs'; export * from './right.mjs';",
    );
    let importer = compile_test_module(
        213,
        "import * as ns from './star.mjs'; export default 'value' in ns;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let left_key = ModuleKey::new("/tmp/left.mjs");
    let right_key = ModuleKey::new("/tmp/right.mjs");
    let exporter_key = ModuleKey::new("/tmp/star.mjs");
    let importer_key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &left_key, "/tmp/left.mjs", &left)
        .unwrap();
    vm.install_module(agent, realm.id(), &right_key, "/tmp/right.mjs", &right)
        .unwrap();
    vm.install_module(agent, realm.id(), &exporter_key, "/tmp/star.mjs", &exporter)
        .unwrap();
    vm.install_module(agent, realm.id(), &importer_key, "/tmp/main.mjs", &importer)
        .unwrap();
    assert!(agent.set_module_requested_key(&exporter_key, 0, Some(left_key.clone())));
    assert!(agent.set_module_requested_key(&exporter_key, 1, Some(right_key.clone())));
    assert!(agent.set_module_requested_key(&importer_key, 0, Some(exporter_key.clone())));

    let _ = vm
        .evaluate_linked_module(agent, realm, &importer_key)
        .unwrap();

    let importer_record = agent
        .module_record(&importer_key)
        .expect("importer module should stay cached");
    let importer_env = importer_record
        .environment()
        .expect("importer evaluation should allocate one environment");
    let default_slot = importer
        .local_exports()
        .iter()
        .find(|entry| entry.export_name() == lyng_js_common::WellKnownAtom::default.id())
        .expect("importer should export a default value")
        .local_slot();

    assert_eq!(
        agent.environment_slot(importer_env, default_slot),
        Some(Value::from_bool(false))
    );
}

#[test]
fn linked_module_graph_treats_shared_namespace_star_exports_as_unambiguous() {
    let empty = compile_test_module(214, "");
    let left = compile_test_module(215, "import * as foo from './empty.mjs'; export { foo };");
    let right = compile_test_module(216, "import * as foo from './empty.mjs'; export { foo };");
    let main = compile_test_module(
        217,
        "export * from './left.mjs'; export * from './right.mjs'; import { foo } from './main.mjs'; export default foo;",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let empty_key = ModuleKey::new("/tmp/empty.mjs");
    let left_key = ModuleKey::new("/tmp/left.mjs");
    let right_key = ModuleKey::new("/tmp/right.mjs");
    let main_key = ModuleKey::new("/tmp/main.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &empty_key, "/tmp/empty.mjs", &empty)
        .unwrap();
    vm.install_module(agent, realm.id(), &left_key, "/tmp/left.mjs", &left)
        .unwrap();
    vm.install_module(agent, realm.id(), &right_key, "/tmp/right.mjs", &right)
        .unwrap();
    vm.install_module(agent, realm.id(), &main_key, "/tmp/main.mjs", &main)
        .unwrap();
    assert!(agent.set_module_requested_key(&left_key, 0, Some(empty_key.clone())));
    assert!(agent.set_module_requested_key(&right_key, 0, Some(empty_key.clone())));
    assert!(agent.set_module_requested_key(&main_key, 0, Some(left_key.clone())));
    assert!(agent.set_module_requested_key(&main_key, 1, Some(right_key.clone())));
    assert!(agent.set_module_requested_key(&main_key, 2, Some(main_key.clone())));

    let _ = vm.evaluate_linked_module(agent, realm, &main_key).unwrap();

    let record = agent
        .module_record(&main_key)
        .expect("main module should stay cached");
    let module_env = record
        .environment()
        .expect("main module should allocate one environment");
    let default_slot = main
        .local_exports()
        .iter()
        .find(|entry| entry.export_name() == lyng_js_common::WellKnownAtom::default.id())
        .expect("main module should export a default value")
        .local_slot();

    assert!(
        agent
            .environment_slot(module_env, default_slot)
            .and_then(Value::as_object_ref)
            .is_some(),
        "self-imported namespace binding should resolve to one namespace object"
    );
}

#[test]
fn linked_module_graph_resolves_namespace_exports_to_live_namespace_objects() {
    let dependency = compile_test_module(206, "export let counter = 1;");
    let exporter = compile_test_module(207, "export * as ns from './dep.mjs';");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let dependency_key = ModuleKey::new("/tmp/dep.mjs");
    let exporter_key = ModuleKey::new("/tmp/ns.mjs");
    let mut vm = Vm::new();

    vm.install_module(
        agent,
        realm.id(),
        &dependency_key,
        "/tmp/dep.mjs",
        &dependency,
    )
    .unwrap();
    vm.install_module(agent, realm.id(), &exporter_key, "/tmp/ns.mjs", &exporter)
        .unwrap();
    assert!(agent.set_module_requested_key(&exporter_key, 0, Some(dependency_key.clone())));

    let _ = vm
        .evaluate_linked_module(agent, realm, &exporter_key)
        .unwrap();

    let dependency_record = agent
        .module_record(&dependency_key)
        .expect("dependency module should stay cached");
    let dependency_env = dependency_record
        .environment()
        .expect("dependency environment should exist after evaluation");
    let counter_export_slot = dependency.local_exports()[0].local_slot();
    let namespace_value = agent
        .module_record(&exporter_key)
        .expect("namespace exporter should stay cached")
        .resolved_exports()
        .first()
        .and_then(|entry| entry.target().value())
        .expect("export * as ns should resolve to one namespace object value");
    let namespace = namespace_value
        .as_object_ref()
        .expect("resolved namespace export should be an object");
    let counter_atom = agent.atoms_mut().intern_collectible("counter");

    assert!(agent.set_environment_slot(dependency_env, counter_export_slot, Value::from_smi(8),));
    assert_eq!(
        ordinary_get(agent, namespace, PropertyKey::from_atom(counter_atom)).unwrap(),
        Value::from_smi(8)
    );
}

#[test]
fn dynamic_import_fulfills_with_the_loaded_module_namespace() {
    let unit = compile_test_unit(203, "import('./dep.mjs');");
    let host = TestHost::new();
    let script_referrer = ModuleKey::new("/tmp/main.js");
    let module_key = ModuleKey::new("/tmp/dep.mjs");
    host.define_module_source(
        "./dep.mjs",
        LoadedModuleSource::new(module_key.clone(), "/tmp/dep.mjs", "export default 7;"),
    );
    host.define_import_meta(
        module_key.clone(),
        ImportMetaProperties::new(vec![ImportMetaProperty {
            key: "url".into(),
            value: ImportMetaValue::String("file:///tmp/dep.mjs".into()),
        }]),
    );
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host_referrer(
            agent,
            realm,
            &unit,
            Some(&script_referrer),
            &host,
            &mut registry,
        )
        .unwrap();

    let promise = result
        .as_object_ref()
        .expect("dynamic import should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("dynamic import promise should stay tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    let namespace = record
        .result()
        .as_object_ref()
        .expect("dynamic import should fulfill with a namespace object");
    let default_atom = agent.atoms_mut().intern_collectible("default");
    assert_eq!(
        ordinary_get(agent, namespace, PropertyKey::from_atom(default_atom)).unwrap(),
        Value::from_smi(7)
    );
    assert!(host
        .snapshot()
        .calls
        .contains(&HostCall::LoadModule(ModuleSourceRequest {
            specifier: "./dep.mjs".into(),
            referrer: Some(script_referrer),
            attributes: Vec::new(),
        })));
}

#[test]
fn dynamic_import_evaluates_options_after_the_specifier_expression() {
    let unit = compile_test_unit(
        204,
        r#"
            (function() {
                var log = [];
                import(log.push('first'), (log.push('second'), undefined))
                    .then(null, function() {});
                return log[0] + ',' + log[1];
            })();
        "#,
    );
    let host = TestHost::new();
    let script_referrer = ModuleKey::new("/tmp/main.js");
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host_referrer(
            agent,
            realm,
            &unit,
            Some(&script_referrer),
            &host,
            &mut registry,
        )
        .unwrap();

    let text = agent
        .heap()
        .view()
        .string_view(result.as_string_ref().expect("result should be a string"))
        .map(decode_string)
        .expect("log string should be allocated");
    assert_eq!(text, "first,second");
}

#[test]
fn dynamic_import_accepts_unary_assignment_expressions() {
    let unit = compile_test_unit(218, "import(~0);");
    let host = TestHost::new();
    let script_referrer = ModuleKey::new("/tmp/main.js");
    let module_key = ModuleKey::new("/tmp/bitnot.mjs");
    host.define_module_source(
        "-1",
        LoadedModuleSource::new(module_key.clone(), "/tmp/bitnot.mjs", "export default 9;"),
    );
    host.define_import_meta(
        module_key.clone(),
        ImportMetaProperties::new(vec![ImportMetaProperty {
            key: "url".into(),
            value: ImportMetaValue::String("file:///tmp/bitnot.mjs".into()),
        }]),
    );
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host_referrer(
            agent,
            realm,
            &unit,
            Some(&script_referrer),
            &host,
            &mut registry,
        )
        .expect("dynamic import unary expression should compile and evaluate");

    let promise = result
        .as_object_ref()
        .expect("dynamic import should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("dynamic import promise should stay tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    let namespace = record
        .result()
        .as_object_ref()
        .expect("dynamic import should fulfill with a namespace object");
    let default_atom = agent.atoms_mut().intern_collectible("default");
    assert_eq!(
        ordinary_get(agent, namespace, PropertyKey::from_atom(default_atom)).unwrap(),
        Value::from_smi(9)
    );
}

#[test]
fn dynamic_import_preserves_script_referrer_after_async_resume() {
    let unit = compile_test_unit(
        220,
        r#"
            (async function() {
                await 0;
                return import('./dep.mjs');
            })();
        "#,
    );
    let host = TestHost::new();
    let script_referrer = ModuleKey::new("/tmp/main.js");
    let module_key = ModuleKey::new("/tmp/dep.mjs");
    host.define_module_source(
        "./dep.mjs",
        LoadedModuleSource::new(module_key.clone(), "/tmp/dep.mjs", "export default 13;"),
    );
    host.define_import_meta(
        module_key.clone(),
        ImportMetaProperties::new(vec![ImportMetaProperty {
            key: "url".into(),
            value: ImportMetaValue::String("file:///tmp/dep.mjs".into()),
        }]),
    );
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    vm.evaluate_script_with_registry_and_host_referrer(
        agent,
        realm,
        &unit,
        Some(&script_referrer),
        &host,
        &mut registry,
    )
    .expect("async dynamic import should evaluate");

    assert!(host
        .snapshot()
        .calls
        .contains(&HostCall::LoadModule(ModuleSourceRequest {
            specifier: "./dep.mjs".into(),
            referrer: Some(script_referrer),
            attributes: Vec::new(),
        })));
}

#[test]
fn dynamic_import_preserves_script_referrer_in_promise_reactions() {
    let unit = compile_test_unit(
        226,
        r#"
            Promise.resolve().then(function() {
                return import('./dep.mjs');
            });
        "#,
    );
    let host = TestHost::new();
    let script_referrer = ModuleKey::new("/tmp/main.js");
    let module_key = ModuleKey::new("/tmp/dep.mjs");
    host.define_module_source(
        "./dep.mjs",
        LoadedModuleSource::new(module_key.clone(), "/tmp/dep.mjs", "export default 17;"),
    );
    host.define_import_meta(
        module_key.clone(),
        ImportMetaProperties::new(vec![ImportMetaProperty {
            key: "url".into(),
            value: ImportMetaValue::String("file:///tmp/dep.mjs".into()),
        }]),
    );
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    vm.evaluate_script_with_registry_and_host_referrer(
        agent,
        realm,
        &unit,
        Some(&script_referrer),
        &host,
        &mut registry,
    )
    .expect("promise reaction dynamic import should evaluate");

    assert!(host
        .snapshot()
        .calls
        .contains(&HostCall::LoadModule(ModuleSourceRequest {
            specifier: "./dep.mjs".into(),
            referrer: Some(script_referrer),
            attributes: Vec::new(),
        })));
}

#[test]
fn dynamic_import_preserves_referrer_through_import_promise_reactions() {
    let unit = compile_test_unit(
        259,
        r#"
            Promise.all([
                import('./dep.mjs'),
                import('./dep.mjs')
            ]).then(async function() {
                await import('./dep.mjs');
                await import('./dep.mjs');
            });
        "#,
    );
    let host = TestHost::new();
    let script_referrer = ModuleKey::new("/tmp/main.js");
    let module_key = ModuleKey::new("/tmp/dep.mjs");
    host.define_module_source(
        "./dep.mjs",
        LoadedModuleSource::new(
            module_key.clone(),
            "/tmp/dep.mjs",
            r#"
                var global = Function('return this;')();
                if (global.dynamicImportMarker) {
                    throw new Error('Module was evaluated more than once.');
                }
                global.dynamicImportMarker = 19;
                export default null;
            "#,
        ),
    );
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host_referrer(
            agent,
            realm,
            &unit,
            Some(&script_referrer),
            &host,
            &mut registry,
        )
        .expect("chained dynamic imports should evaluate");
    let promise = result
        .as_object_ref()
        .expect("script result should be the chained promise");
    let record = agent
        .promise_record(promise)
        .expect("script result promise should stay tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);

    let expected = HostCall::LoadModule(ModuleSourceRequest {
        specifier: "./dep.mjs".into(),
        referrer: Some(script_referrer),
        attributes: Vec::new(),
    });
    let load_count = host
        .snapshot()
        .calls
        .into_iter()
        .filter(|call| call == &expected)
        .count();
    assert_eq!(load_count, 4);
}

#[test]
fn dynamic_import_does_not_preempt_static_module_dfs_evaluation() {
    let main_source = "import './state.mjs'; import './a.mjs'; import './b.mjs';";
    let state_source = r#"
        export let score = 0;
        export function step(value) {
            score = score * 10 + value;
        }
    "#;
    let state_unit = compile_test_module(258, state_source);
    let state_key = ModuleKey::new("/tmp/state.mjs");
    let host = TestHost::new();
    host.define_module_source(
        "main.mjs",
        LoadedModuleSource::new(ModuleKey::new("/tmp/main.mjs"), "main.mjs", main_source),
    );
    host.define_module_source(
        "./state.mjs",
        LoadedModuleSource::new(state_key.clone(), "state.mjs", state_source),
    );
    host.define_module_source(
        "./a.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/a.mjs"),
            "a.mjs",
            r#"
                import { step } from './state.mjs';
                import('./b.mjs');
                step(1);
            "#,
        ),
    );
    host.define_module_source(
        "./b.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/b.mjs"),
            "b.mjs",
            "import { step } from './state.mjs'; step(2);",
        ),
    );
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;
    let loaded = vm
        .load_module_graph_from_host(
            agent,
            realm,
            &host,
            &ModuleSourceRequest {
                specifier: "main.mjs".into(),
                referrer: None,
                attributes: Vec::new(),
            },
        )
        .unwrap();

    let _ = vm
        .evaluate_linked_module_with_registry_and_host(
            agent,
            realm,
            loaded.key(),
            &host,
            &mut registry,
        )
        .unwrap();

    let record = agent
        .module_record(&state_key)
        .expect("state module should stay cached");
    let module_env = record
        .environment()
        .expect("state module evaluation should allocate one environment");
    let score_slot = state_unit
        .local_exports()
        .iter()
        .find(|entry| state_unit.atom_text(entry.export_name()) == Some("score"))
        .expect("state module should export score")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, score_slot),
        Some(Value::from_smi(12))
    );
}

#[test]
fn dynamic_import_rejects_module_parse_errors_with_syntax_error() {
    let unit = compile_test_unit(221, "import('./bad.mjs');");
    let host = TestHost::new();
    let script_referrer = ModuleKey::new("/tmp/main.js");
    let module_key = ModuleKey::new("/tmp/bad.mjs");
    host.define_module_source(
        "./bad.mjs",
        LoadedModuleSource::new(module_key, "/tmp/bad.mjs", "export {"),
    );
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host_referrer(
            agent,
            realm,
            &unit,
            Some(&script_referrer),
            &host,
            &mut registry,
        )
        .expect("dynamic import parse rejection should evaluate");

    let promise = result
        .as_object_ref()
        .expect("dynamic import should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("dynamic import promise should stay tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Rejected);
    let reason = record
        .result()
        .as_object_ref()
        .expect("dynamic import parse failure should reject with an error object");
    let name_atom = agent.atoms_mut().intern_collectible("name");
    let name = ordinary_get(agent, reason, PropertyKey::from_atom(name_atom))
        .expect("error name should be readable")
        .as_string_ref()
        .and_then(|string| agent.heap().view().string_view(string))
        .map(decode_string)
        .expect("error name should be a string");
    assert_eq!(name, "SyntaxError");
}

#[test]
fn dynamic_import_source_phase_rejects_source_text_modules_with_syntax_error() {
    let unit = compile_test_unit(223, "import.source('./dep.mjs');");
    let host = TestHost::new();
    let script_referrer = ModuleKey::new("/tmp/main.js");
    let module_key = ModuleKey::new("/tmp/dep.mjs");
    host.define_module_source(
        "./dep.mjs",
        LoadedModuleSource::new(module_key, "/tmp/dep.mjs", "export default 7;"),
    );
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host_referrer(
            agent,
            realm,
            &unit,
            Some(&script_referrer),
            &host,
            &mut registry,
        )
        .expect("dynamic source import rejection should evaluate");

    let promise = result
        .as_object_ref()
        .expect("dynamic source import should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("dynamic source import promise should stay tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Rejected);
    let reason = record
        .result()
        .as_object_ref()
        .expect("dynamic source import should reject with an error object");
    let name_atom = agent.atoms_mut().intern_collectible("name");
    let name = ordinary_get(agent, reason, PropertyKey::from_atom(name_atom))
        .expect("error name should be readable")
        .as_string_ref()
        .and_then(|string| agent.heap().view().string_view(string))
        .map(decode_string)
        .expect("error name should be a string");
    assert_eq!(name, "SyntaxError");
}

#[test]
fn dynamic_import_attributes_reject_non_object_and_non_string_values() {
    let cases = [
        "import('./dep.mjs', false);",
        "import('./dep.mjs', { with: false });",
        "import('./dep.mjs', { with: { type: 1 } });",
    ];

    for (index, source) in cases.into_iter().enumerate() {
        let unit = compile_test_unit(224 + u32::try_from(index).unwrap(), source);
        let host = TestHost::new();
        let script_referrer = ModuleKey::new("/tmp/main.js");
        let module_key = ModuleKey::new("/tmp/dep.mjs");
        host.define_module_source(
            "./dep.mjs",
            LoadedModuleSource::new(module_key, "/tmp/dep.mjs", "export default 7;"),
        );
        let mut runtime = Runtime::new(host.clone());
        let agent = runtime.root_agent_mut();
        let realm = agent.default_realm().expect("default realm should exist");
        let mut vm = Vm::new();
        let mut registry = RejectingRegistry;

        let result = vm
            .evaluate_script_with_registry_and_host_referrer(
                agent,
                realm,
                &unit,
                Some(&script_referrer),
                &host,
                &mut registry,
            )
            .expect("dynamic import attribute validation should evaluate");

        let promise = result
            .as_object_ref()
            .expect("dynamic import should return a promise object");
        let record = agent
            .promise_record(promise)
            .expect("dynamic import promise should stay tracked");
        assert_eq!(record.state(), lyng_js_env::PromiseState::Rejected);
        let reason = record
            .result()
            .as_object_ref()
            .expect("dynamic import should reject with an error object");
        let name_atom = agent.atoms_mut().intern_collectible("name");
        let name = ordinary_get(agent, reason, PropertyKey::from_atom(name_atom))
            .expect("error name should be readable")
            .as_string_ref()
            .and_then(|string| agent.heap().view().string_view(string))
            .map(decode_string)
            .expect("error name should be a string");
        assert_eq!(name, "TypeError");
    }
}

#[test]
fn dynamic_import_rejects_ambiguous_module_exports_with_syntax_error() {
    let unit = compile_test_unit(222, "import('./entry.mjs');");
    let host = TestHost::new();
    let script_referrer = ModuleKey::new("/tmp/main.js");
    host.define_module_source(
        "./entry.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/entry.mjs"),
            "/tmp/entry.mjs",
            "export { x } from './ambiguous.mjs';",
        ),
    );
    host.define_module_source(
        "./ambiguous.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/ambiguous.mjs"),
            "/tmp/ambiguous.mjs",
            "export * from './left.mjs'; export * from './right.mjs';",
        ),
    );
    host.define_module_source(
        "./left.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/left.mjs"),
            "/tmp/left.mjs",
            "export var x = 1;",
        ),
    );
    host.define_module_source(
        "./right.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/right.mjs"),
            "/tmp/right.mjs",
            "export var x = 2;",
        ),
    );
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host_referrer(
            agent,
            realm,
            &unit,
            Some(&script_referrer),
            &host,
            &mut registry,
        )
        .expect("dynamic import ambiguous export rejection should evaluate");

    let promise = result
        .as_object_ref()
        .expect("dynamic import should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("dynamic import promise should stay tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Rejected);
    let reason = record
        .result()
        .as_object_ref()
        .expect("ambiguous export should reject with an error object");
    let name_atom = agent.atoms_mut().intern_collectible("name");
    let name = ordinary_get(agent, reason, PropertyKey::from_atom(name_atom))
        .expect("error name should be readable")
        .as_string_ref()
        .and_then(|string| agent.heap().view().string_view(string))
        .map(decode_string)
        .expect("error name should be a string");
    assert_eq!(name, "SyntaxError");
}

#[test]
fn nested_eval_script_preserves_host_access_for_dynamic_import() {
    let unit = compile_test_unit(219, "embeddingEvalScript(\"import('./dep.mjs');\");");
    let host = TestHost::new();
    let script_referrer = ModuleKey::new("/tmp/main.js");
    let module_key = ModuleKey::new("/tmp/dep.mjs");
    host.define_module_source(
        "./dep.mjs",
        LoadedModuleSource::new(module_key.clone(), "/tmp/dep.mjs", "export default 11;"),
    );
    host.define_import_meta(
        module_key.clone(),
        ImportMetaProperties::new(vec![ImportMetaProperty {
            key: "url".into(),
            value: ImportMetaValue::String("file:///tmp/dep.mjs".into()),
        }]),
    );
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;
    let provider: crate::SharedRealmExtensionProvider = Arc::new(TestEmbeddingProvider);

    let result = vm
        .evaluate_script_with_registry_and_host_referrer_and_extensions(
            agent,
            realm,
            &unit,
            Some(&script_referrer),
            &host,
            &mut registry,
            Some(&provider),
        )
        .expect("nested evalScript import should evaluate");

    let promise = result
        .as_object_ref()
        .expect("evalScript should return the nested import promise");
    let record = agent
        .promise_record(promise)
        .expect("nested import promise should stay tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    let namespace = record
        .result()
        .as_object_ref()
        .expect("nested import should fulfill with a namespace object");
    let default_atom = agent.atoms_mut().intern_collectible("default");
    assert_eq!(
        ordinary_get(agent, namespace, PropertyKey::from_atom(default_atom)).unwrap(),
        Value::from_smi(11)
    );
    assert!(host
        .snapshot()
        .calls
        .contains(&HostCall::LoadModule(ModuleSourceRequest {
            specifier: "./dep.mjs".into(),
            referrer: Some(script_referrer),
            attributes: Vec::new(),
        })));
}

#[test]
fn promise_checkpoint_drains_reaction_jobs_and_reports_host_phases() {
    let unit = compile_test_unit(
        217,
        r#"
            (function() {
                let resolve;
                let reject;
                let promise = new Promise(function(innerResolve, innerReject) {
                    resolve = innerResolve;
                    reject = innerReject;
                });
                return [promise, resolve, reject];
            })();
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let tuple = result
        .as_object_ref()
        .expect("script should return the promise tuple array");
    let promise = ordinary_get(agent, tuple, PropertyKey::Index(0))
        .unwrap()
        .as_object_ref()
        .expect("tuple[0] should be the promise");
    let resolve = ordinary_get(agent, tuple, PropertyKey::Index(1))
        .unwrap()
        .as_object_ref()
        .expect("tuple[1] should be the resolve function");
    let reject = ordinary_get(agent, tuple, PropertyKey::Index(2))
        .unwrap()
        .as_object_ref()
        .expect("tuple[2] should be the reject function");
    let capability = agent.alloc_promise_capability();
    let _ = agent.set_promise_capability_promise(capability, promise);
    let _ = agent.set_promise_capability_resolve(capability, resolve);
    let _ = agent.set_promise_capability_reject(capability, reject);
    let reaction = agent.alloc_promise_reaction(PromiseReactionRecord::new(
        PromiseReactionKind::Fulfill,
        PromiseReactionHandler::PassThrough(Value::from_smi(5)),
        Some(capability),
    ));
    let job = agent.enqueue_job_with_payload(
        HostJobKind::Promise,
        ExecutableId::Builtin,
        RuntimeJobPayload::PromiseReaction {
            reaction,
            argument: Value::undefined(),
        },
        Some(realm.id()),
        Some("PromiseReaction".into()),
    );
    assert_eq!(agent.queued_job_count(JobQueueKind::Promise), 1);
    let mut registry = RejectingRegistry;

    vm.checkpoint_promise_jobs(agent, &host, &mut registry)
        .unwrap();

    let record = agent
        .promise_record(promise)
        .expect("result promise should remain tracked after checkpoint");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(5));
    let observations = host
        .snapshot()
        .calls
        .into_iter()
        .filter_map(|call| match call {
            HostCall::ObserveJob(observation) => Some(observation),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(observations.len(), 3);
    assert_eq!(observations[0].job_id, job.host_job_id());
    assert_eq!(observations[0].phase, HostJobPhase::Enqueued);
    assert_eq!(observations[0].kind, HostJobKind::Promise);
    assert_eq!(observations[1].job_id, job.host_job_id());
    assert_eq!(observations[1].phase, HostJobPhase::Started);
    assert_eq!(observations[2].job_id, job.host_job_id());
    assert_eq!(observations[2].phase, HostJobPhase::Completed);
}

#[test]
fn evaluate_script_drains_nested_promise_jobs_to_quiescence() {
    let unit = compile_test_unit(
        218,
        r#"
            Promise.resolve(1).then().then();
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let result_promise = result
        .as_object_ref()
        .expect("script should return the nested chained promise");
    let record = agent
        .promise_record(result_promise)
        .expect("result promise should remain tracked after checkpoint");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(1));
    assert_eq!(agent.queued_job_count(JobQueueKind::Promise), 0);
}

#[test]
fn evaluate_script_runs_callable_promise_reactions() {
    let unit = compile_test_unit(
        219,
        r#"
            Promise.resolve(1)
                .then(function(value) {
                    return value + 1;
                })
                .then();
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let result_promise = result
        .as_object_ref()
        .expect("script should return the chained promise");
    let record = agent
        .promise_record(result_promise)
        .expect("result promise should remain tracked after checkpoint");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(2));
}

#[test]
fn evaluate_script_resolves_promise_all_values_in_order() {
    let unit = compile_test_unit(
        220,
        r#"
            Promise.all([Promise.resolve(1), 2, Promise.resolve(3)]);
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let promise = result
        .as_object_ref()
        .expect("Promise.all should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("Promise.all result promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    let values = record
        .result()
        .as_object_ref()
        .expect("Promise.all should fulfill with an array object");
    assert_eq!(
        ordinary_get(agent, values, PropertyKey::from_array_index(0).unwrap()).unwrap(),
        Value::from_smi(1)
    );
    assert_eq!(
        ordinary_get(agent, values, PropertyKey::from_array_index(1).unwrap()).unwrap(),
        Value::from_smi(2)
    );
    assert_eq!(
        ordinary_get(agent, values, PropertyKey::from_array_index(2).unwrap()).unwrap(),
        Value::from_smi(3)
    );
}

#[test]
fn evaluate_script_array_from_async_resolves_sync_iterable_values() {
    let unit = compile_test_unit(
        220,
        r#"
            Array.fromAsync([Promise.resolve(1), 2], function(value, index) {
                return Promise.resolve(value + index + 1);
            });
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let promise = result
        .as_object_ref()
        .expect("Array.fromAsync should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("Array.fromAsync promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    let values = record
        .result()
        .as_object_ref()
        .expect("Array.fromAsync should fulfill with an array object");
    assert_eq!(
        ordinary_get(agent, values, PropertyKey::from_array_index(0).unwrap()).unwrap(),
        Value::from_smi(2)
    );
    assert_eq!(
        ordinary_get(agent, values, PropertyKey::from_array_index(1).unwrap()).unwrap(),
        Value::from_smi(4)
    );
}

#[test]
fn evaluate_script_array_from_async_uses_intrinsic_iterator_symbols() {
    let unit = compile_test_unit(
        220,
        r#"
            var originalSymbol = globalThis.Symbol;
            var fakeIterator = originalSymbol("iterator");
            var fakeAsyncIterator = originalSymbol("asyncIterator");
            globalThis.Symbol = {
                iterator: fakeIterator,
                asyncIterator: fakeAsyncIterator
            };
            var input = {
                length: 2,
                0: 5,
                1: 6
            };
            input[fakeIterator] = function() {
                throw new Error("fake sync iterator should not be used");
            };
            input[fakeAsyncIterator] = function() {
                throw new Error("fake async iterator should not be used");
            };
            var result = Array.fromAsync(input);
            globalThis.Symbol = originalSymbol;
            result;
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let promise = result
        .as_object_ref()
        .expect("Array.fromAsync should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("Array.fromAsync promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    let values = record
        .result()
        .as_object_ref()
        .expect("Array.fromAsync should fulfill with an array object");
    assert_eq!(
        ordinary_get(agent, values, PropertyKey::from_array_index(0).unwrap()).unwrap(),
        Value::from_smi(5)
    );
    assert_eq!(
        ordinary_get(agent, values, PropertyKey::from_array_index(1).unwrap()).unwrap(),
        Value::from_smi(6)
    );
}

#[test]
fn evaluate_script_array_from_async_sync_iterator_observes_mutation_after_first_await() {
    let unit = compile_test_unit(
        220,
        r#"
            async function main() {
                var items = [1, 2, 3];
                var promise = Array.fromAsync(items);
                items.push(4);
                var result = await promise;
                return result.join(",");
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("Array.fromAsync mutation test should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("Array.fromAsync promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    let text = record
        .result()
        .as_string_ref()
        .expect("Array.fromAsync mutation result should be a string");
    assert_eq!(
        decode_string(agent.heap().view().string_view(text).unwrap()),
        "1,2,3,4"
    );
}

#[test]
fn evaluate_script_array_from_async_awaits_async_iterator_values_before_mapping() {
    let unit = compile_test_unit(
        220,
        r#"
            async function* asyncGen() {
                for (let i = 0; i < 4; i++) {
                    yield Promise.resolve(i * 2);
                }
            }
            async function main() {
                var result = await Array.fromAsync(
                    { [Symbol.asyncIterator]: asyncGen },
                    async function(value, index) {
                        return Promise.resolve(value * index);
                    }
                );
                return result.join(",");
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("Array.fromAsync async iterator test should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("Array.fromAsync promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    let text = record
        .result()
        .as_string_ref()
        .expect("Array.fromAsync async iterator result should be a string");
    assert_eq!(
        decode_string(agent.heap().view().string_view(text).unwrap()),
        "0,2,8,18"
    );
}

#[test]
fn evaluate_script_array_from_async_rejects_bigint_array_like_length() {
    let unit = compile_test_unit(
        220,
        r#"
            Array.fromAsync({ length: 1n, 0: 0 });
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("Array.fromAsync BigInt length test should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("Array.fromAsync promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Rejected);
}

#[test]
fn evaluate_script_array_from_async_preserves_custom_constructor_operation_order() {
    let unit = compile_test_unit(
        220,
        r#"
            async function main() {
                var calls = [];
                function format(key) {
                    return "A[" + key + "]";
                }
                function MyArray() {
                    calls.push("construct MyArray");
                    return new Proxy(Object.create(null), {
                        set(target, key, value) {
                            calls.push("set " + format(key));
                            return Reflect.set(target, key, value);
                        },
                        defineProperty(target, key, descriptor) {
                            calls.push("defineProperty " + format(key));
                            return Reflect.defineProperty(target, key, descriptor);
                        }
                    });
                }
                await Array.fromAsync.call(MyArray, [1, 2]);
                return calls.join("|");
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("Array.fromAsync operation-order test should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("Array.fromAsync promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    let text = record
        .result()
        .as_string_ref()
        .expect("operation order should be returned as a string");
    assert_eq!(
        decode_string(agent.heap().view().string_view(text).unwrap()),
        "construct MyArray|defineProperty A[0]|defineProperty A[1]|set A[length]"
    );
}

#[test]
fn evaluate_script_array_from_async_custom_constructor_uses_custom_sync_iterator() {
    let unit = compile_test_unit(
        220,
        r#"
            async function main() {
                function MyArray() {
                    return [];
                }
                var input = [1, 2];
                input[Symbol.iterator] = function() {
                    var index = 0;
                    return {
                        next() {
                            index += 1;
                            if (index === 1) {
                                return { value: Promise.resolve(10), done: false };
                            }
                            if (index === 2) {
                                return { value: 20, done: false };
                            }
                            return { done: true };
                        }
                    };
                };
                var result = await Array.fromAsync.call(MyArray, input);
                return result.join(",");
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("Array.fromAsync custom iterator test should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("Array.fromAsync promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    let text = record
        .result()
        .as_string_ref()
        .expect("custom iterator result should be returned as a string");
    assert_eq!(
        decode_string(agent.heap().view().string_view(text).unwrap()),
        "10,20"
    );
}

#[test]
fn evaluate_script_resolves_promise_all_settled_records() {
    let unit = compile_test_unit(
        221,
        r#"
            Promise.allSettled([Promise.resolve(1), Promise.reject(2)]);
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let promise = result
        .as_object_ref()
        .expect("Promise.allSettled should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("Promise.allSettled result promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    let results = record
        .result()
        .as_object_ref()
        .expect("Promise.allSettled should fulfill with an array object");
    let first = ordinary_get(agent, results, PropertyKey::from_array_index(0).unwrap())
        .unwrap()
        .as_object_ref()
        .expect("first allSettled entry should be an object");
    let second = ordinary_get(agent, results, PropertyKey::from_array_index(1).unwrap())
        .unwrap()
        .as_object_ref()
        .expect("second allSettled entry should be an object");
    let status_atom = agent.atoms_mut().intern_collectible("status");
    let reason_atom = agent.atoms_mut().intern_collectible("reason");
    let first_status = ordinary_get(agent, first, PropertyKey::from_atom(status_atom)).unwrap();
    let first_value = ordinary_get(
        agent,
        first,
        PropertyKey::from_atom(WellKnownAtom::value.id()),
    )
    .unwrap();
    let second_status = ordinary_get(agent, second, PropertyKey::from_atom(status_atom)).unwrap();
    let second_reason = ordinary_get(agent, second, PropertyKey::from_atom(reason_atom)).unwrap();
    assert_eq!(
        decode_string(
            agent
                .heap()
                .view()
                .string_view(first_status.as_string_ref().unwrap())
                .unwrap(),
        ),
        "fulfilled"
    );
    assert_eq!(first_value, Value::from_smi(1));
    assert_eq!(
        decode_string(
            agent
                .heap()
                .view()
                .string_view(second_status.as_string_ref().unwrap())
                .unwrap(),
        ),
        "rejected"
    );
    assert_eq!(second_reason, Value::from_smi(2));
}

#[test]
fn evaluate_script_resolves_promise_race_with_first_settlement() {
    let unit = compile_test_unit(
        222,
        r#"
            Promise.race([Promise.resolve(4), new Promise(function() {})]);
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let promise = result
        .as_object_ref()
        .expect("Promise.race should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("Promise.race result promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(4));
}

#[test]
fn evaluate_script_promise_all_rejects_non_iterables_through_the_returned_promise() {
    let unit = compile_test_unit(
        223,
        r#"
            Promise.all(false)
                .then(function() { return false; }, function(error) {
                    return error instanceof TypeError;
                })
                .then();
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let promise = result
        .as_object_ref()
        .expect("Promise.all(false) chain should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("final chained promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_bool(true));
}

#[test]
fn evaluate_script_invokes_promise_all_resolve_for_each_iterated_value() {
    let unit = compile_test_unit(
        224,
        r#"
            var callCount = 0;
            var boundResolve = Promise.resolve.bind(Promise);
            Promise.resolve = function(value) {
                callCount += 1;
                return boundResolve(value);
            };
            Promise.all([1, 2, 3]).then(function() { return callCount; }).then();
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let promise = result
        .as_object_ref()
        .expect("Promise.all resolve-count chain should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("final chained promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(3));
}

#[test]
fn evaluate_script_promise_all_result_creation_avoids_array_prototype_setters() {
    let unit = compile_test_unit(
        225,
        r#"
            Object.defineProperty(Array.prototype, 0, {
                set: function() {
                    throw new Error("setter");
                }
            });
            Promise.all([42]).then(function() { return true; }).then();
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let promise = result
        .as_object_ref()
        .expect("Promise.all setter-avoidance chain should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("final chained promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_bool(true));
}

#[test]
fn evaluate_script_resolves_promise_any_with_first_fulfillment() {
    let unit = compile_test_unit(
        226,
        r#"
            Promise.any([Promise.reject(1), Promise.resolve(7), Promise.reject(3)]).then();
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let promise = result
        .as_object_ref()
        .expect("Promise.any should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("Promise.any result promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(7));
}

#[test]
fn evaluate_script_rejects_promise_any_with_aggregate_error() {
    let unit = compile_test_unit(
        227,
        r#"
            Promise.any([Promise.reject(4), Promise.reject(9)])
                .then(function() { return false; }, function(error) {
                    return error instanceof AggregateError
                        && error instanceof Error
                        && error.message === ""
                        && error.errors.length === 2
                        && error.errors[0] === 4
                        && error.errors[1] === 9;
                })
                .then();
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let promise = result
        .as_object_ref()
        .expect("Promise.any rejection chain should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("final chained promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_bool(true));
}

#[test]
fn evaluate_script_rejects_promise_any_empty_iterable_with_aggregate_error() {
    let unit = compile_test_unit(
        231,
        r#"
            Promise.any([])
                .then(function() { return false; }, function(error) {
                    return error instanceof AggregateError
                        && error.errors.length === 0;
                })
                .then();
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let promise = result
        .as_object_ref()
        .expect("Promise.any([]) chain should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("final chained promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_bool(true));
}

#[test]
fn evaluate_script_promise_any_preserves_fulfillment_job_order() {
    let unit = compile_test_unit(
        232,
        r#"
            var sequence = [];
            var p1 = Promise.resolve(1);
            var p2 = Promise.resolve(2);
            sequence.push(1);
            p1.then(function() {
                sequence.push(3);
            });
            var outcome = Promise.any([p1, p2]).then(function() {
                sequence.push(5);
                return sequence[0] === 1
                    && sequence[1] === 2
                    && sequence[2] === 3
                    && sequence[3] === 4
                    && sequence[4] === 5;
            }).then();
            p2.then(function() {
                sequence.push(4);
            });
            sequence.push(2);
            outcome;
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let promise = result
        .as_object_ref()
        .expect("Promise.any ordering chain should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("final chained promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_bool(true));
}

#[test]
fn evaluate_script_promise_any_rejects_non_callable_capability_resolve() {
    let unit = compile_test_unit(
        233,
        r#"
            function Custom(executor) {
                executor({}, function() {});
                return Promise.resolve(0);
            }
            try {
                Promise.any.call(Custom, []);
                false;
            } catch (error) {
                error instanceof TypeError;
            }
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_promise_any_calling_eval_throws_type_error() {
    let unit = compile_test_unit(
        237,
        r#"
            typeof eval === "function" && (function() {
                try {
                    Promise.any.call(eval);
                    return false;
                } catch (error) {
                    return error instanceof TypeError;
                }
            })();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_promise_any_iterator_step_errors_reject_the_result_promise() {
    let unit = compile_test_unit(
        234,
        r#"
            var error = new Error("step");
            var poisonedDone = {};
            Object.defineProperty(poisonedDone, "done", {
                get: function() {
                    throw error;
                }
            });
            Object.defineProperty(poisonedDone, "value", {
                get: function() {
                    return 0;
                }
            });
            var iterable = {
                [Symbol.iterator]: function() {
                    return {
                        next: function() {
                            return poisonedDone;
                        },
                        return: function() {
                            throw new Error("iterator should not close");
                        }
                    };
                }
            };
            Promise.any(iterable);
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let promise = result
        .as_object_ref()
        .expect("Promise.any(iterable) should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("result promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Rejected);
    let error = record
        .result()
        .as_object_ref()
        .expect("iterator-step rejection should use the thrown Error object");
    let message_atom = agent.atoms_mut().intern_collectible("message");
    let message = ordinary_get(agent, error, PropertyKey::from_atom(message_atom)).unwrap();
    assert_eq!(
        decode_string(
            agent
                .heap()
                .view()
                .string_view(message.as_string_ref().unwrap())
                .unwrap(),
        ),
        "step"
    );
}

#[test]
fn evaluate_script_eval_executes_string_source_in_the_current_realm() {
    let unit = compile_test_unit(
        238,
        r#"
            var indirect = eval;
            indirect("7") === 7;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_eval_fast_paths_regexp_literal_source() {
    let unit = compile_test_unit(
        23801,
        r#"
            eval("/a/").source === "a" &&
            eval("/" + String.fromCharCode(0xd800) + "/").source.charCodeAt(0) === 0xd800 &&
            (0, eval)("/b/g").global === true;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_direct_eval_poisoned_scope_reads_var_binding() {
    let unit = compile_test_unit(
        2381,
        r#"
            function outer() {
                var value = 1;
                eval("");
                return value;
            }
            outer() === 1;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_direct_eval_poisoned_scope_hoists_function_declaration() {
    let unit = compile_test_unit(
        2382,
        r#"
            function outer() {
                eval("");
                function inner() {
                    return 1;
                }
                return inner();
            }
            outer() === 1;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn direct_eval_dynamic_name_resolution_matches_runtime_atom_text_when_ids_differ() {
    let unit = compile_test_unit(
        2383,
        r#"
            function runtimeOuter() {
                var runtimeLocal = 7;
                eval("");
                return runtimeLocal;
            }
            runtimeOuter();
        "#,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let _ = agent.atoms_mut().intern_collectible("padding");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn direct_eval_string_comparison_source_lowers_to_dynamic_name_lookup() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        SourceId::new(2392),
        "'str1' === __10_4_2_1_1_1;",
    );
    assert!(!parsed.diagnostics.has_errors());

    let mut sema = analyze_script(&parsed, &atoms);
    for record in sema.use_sites.as_mut_slice() {
        if matches!(
            record.resolution_kind,
            lyng_js_sema::ResolutionKind::Global | lyng_js_sema::ResolutionKind::Unresolved
        ) {
            record.resolution_kind = lyng_js_sema::ResolutionKind::Dynamic;
        }
    }

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit.function(unit.entry()).unwrap();
    assert!(entry.instructions().iter().any(|instruction| matches!(
        instruction,
        Instruction::Abx {
            opcode: Opcode::LoadName,
            ..
        }
    )));
}

#[test]
fn compiled_compare_form_keeps_long_binding_in_child_environment_bindings() {
    let unit = compile_test_unit(
        2403,
        r#"
            function testcase() {
                var __10_4_2_1_1_1 = "str1";
                return eval("'str1' === __10_4_2_1_1_1");
            }
            testcase();
        "#,
    );
    let entry = unit
        .function(unit.entry())
        .expect("script entry should exist");
    assert_eq!(entry.child_functions().len(), 1);
    let testcase = unit
        .function(entry.child_functions()[0])
        .expect("testcase function should compile as a child");
    let expected = unit_atom(&unit, "__10_4_2_1_1_1");

    assert!(testcase
        .environment_bindings()
        .iter()
        .any(|binding| binding.name() == Some(expected)));
}

#[test]
fn evaluate_script_direct_eval_reads_function_local_binding() {
    let unit = compile_test_unit(
        2384,
        r#"
            var value = 1;
            function outer() {
                var value = 7;
                return eval("value");
            }
            outer() === 7;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_direct_eval_reads_catch_binding() {
    let unit = compile_test_unit(
        2385,
        r#"
            var value = 1;
            function outer() {
                try {
                    throw 7;
                } catch (value) {
                    return eval("value");
                }
            }
            outer() === 7;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_direct_eval_matches_test262_global_env_rec() {
    let unit = compile_test_unit(
        2386,
        r#"
            var __10_4_2_1_1_1 = "str";
            function testcase() {
                var __10_4_2_1_1_1 = "str1";
                return eval("'str1' === __10_4_2_1_1_1");
            }
            testcase() === true;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_direct_eval_reads_test262_global_env_rec_binding() {
    let unit = compile_test_unit(
        2388,
        r#"
            var __10_4_2_1_1_1 = "str";
            function testcase() {
                var __10_4_2_1_1_1 = "str1";
                return eval("__10_4_2_1_1_1");
            }
            testcase() === "str1";
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_direct_eval_matches_test262_global_env_rec_fun() {
    let unit = compile_test_unit(
        2387,
        r#"
            var __10_4_2_1_2 = "str";
            function testcase() {
                var __10_4_2_1_2 = "str1";
                function foo() {
                    var __10_4_2_1_2 = "str2";
                    return eval("'str2' === __10_4_2_1_2");
                }
                return foo();
            }
            testcase() === true;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_direct_eval_parenthesized_string_leading_expression() {
    let unit = compile_test_unit(
        2389,
        r#"
            var __10_4_2_1_1_1 = "str";
            function testcase() {
                var __10_4_2_1_1_1 = "str1";
                return eval("('str1' === __10_4_2_1_1_1)");
            }
            testcase() === true;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_direct_eval_string_comparison_result_shape() {
    let unit = compile_test_unit(
        2390,
        r#"
            var __10_4_2_1_1_1 = "str";
            function testcase() {
                var __10_4_2_1_1_1 = "str1";
                var result = eval("'str1' === __10_4_2_1_1_1");
                if (result === true) return 1;
                if (result === false) return 2;
                if (result === undefined) return 3;
                if (result === "str1") return 4;
                return 5;
            }
            testcase();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn evaluate_script_with_statement_assigns_existing_object_binding() {
    let unit = compile_test_unit(
        2393,
        r#"
            var x = 1;
            var object = { x: 2 };
            with (object) {
                x = 3;
            }
            object.x * 10 + x;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn evaluate_script_with_statement_var_initializer_assigns_through_object_environment() {
    let unit = compile_test_unit(
        2397,
        r#"
            var object = { x: 2 };
            var outer = "missing";
            with (object) {
                var x = 3;
                outer = x;
            }
            String(object.x) + ":" + String(x) + ":" + outer;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(text, "3:undefined:3");
}

#[test]
fn evaluate_script_function_declared_inside_with_captures_object_environment_for_assignment() {
    let unit = compile_test_unit(
        2398,
        r#"
            var p1 = 1;
            var myObj = { p1: "a", value: "obj" };
            var f;
            with (myObj) {
                var f = function() {
                    var value = "local";
                    p1 = "x1";
                    return String(p1) + ":" + String(myObj.p1) + ":" + value + ":" + myObj.value;
                };
            }
            f() + ":" + String(p1);
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(text, "x1:x1:local:obj:1");
}

#[test]
fn evaluate_script_function_called_inside_with_uses_declaration_environment() {
    let unit = compile_test_unit(
        2399,
        r#"
            var p1 = 1;
            var myObj = { p1: "a" };
            var f = function() {
                p1 = "x1";
                return String(p1) + ":" + String(myObj.p1);
            };
            var result;
            with (myObj) {
                result = f();
            }
            result + ":" + String(p1) + ":" + String(myObj.p1);
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(text, "x1:a:x1:a");
}

#[test]
fn evaluate_script_function_containing_with_uses_function_this_binding() {
    let unit = compile_test_unit(
        2400,
        r#"
            globalThis.p2 = 2;
            var myObj = { p2: "b" };
            var f = function() {
                with (myObj) {
                    this.p2 = "x2";
                }
            };
            f();
            String(globalThis.p2) + ":" + String(myObj.p2);
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(text, "x2:b");
}

#[test]
fn evaluate_script_function_called_inside_with_uses_global_this_binding() {
    let unit = compile_test_unit(
        2401,
        r#"
            globalThis.p2 = 2;
            var myObj = { p2: "b" };
            var f = function() {
                this.p2 = "x2";
            };
            with (myObj) {
                f();
            }
            String(globalThis.p2) + ":" + String(myObj.p2);
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(text, "x2:b");
}

#[test]
fn evaluate_script_function_called_inside_with_keeps_local_var_separate_from_object_property() {
    let unit = compile_test_unit(
        2402,
        r#"
            var myObj = { value: "obj" };
            var f = function() {
                var value = "local";
                return value + ":" + myObj.value;
            };
            with (myObj) {
                f();
            }
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(text, "local:obj");
}

#[test]
fn evaluate_script_function_called_inside_with_uses_hoisted_local_before_initializer() {
    let unit = compile_test_unit(
        2403,
        r#"
            var myObj = { value: "obj" };
            var f = function() {
                try {
                    throw value;
                } catch (error) {
                    return String(error) + ":" + String(value) + ":" + myObj.value;
                }
                var value = "local";
            };
            with (myObj) {
                f();
            }
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(text, "undefined:undefined:obj");
}

#[test]
fn evaluate_script_function_containing_with_reports_binding_targets() {
    let unit = compile_test_unit(
        2404,
        r#"
            globalThis.p1 = 1;
            globalThis.p2 = 2;
            globalThis.p3 = 3;
            var myObj = {
                p1: "a",
                p2: "b",
                p3: "c",
                value: "obj",
                parseInt: "obj_parseInt",
                eval: "obj_eval"
            };
            var st_p1 = "unset";
            var st_parseInt = "unset";
            var st_eval = "unset";
            var del = "unset";
            var f = function() {
                with (myObj) {
                    st_p1 = p1;
                    st_parseInt = parseInt;
                    st_eval = eval;
                    p1 = "x1";
                    this.p2 = "x2";
                    del = delete p3;
                    var p4 = "x4";
                    p5 = "x5";
                    var value = "value";
                }
            };
            f();
            [
                String(globalThis.p1),
                String(globalThis.p2),
                String(globalThis.p3),
                String(typeof p4),
                String(p5),
                String(myObj.p1),
                String(myObj.p2),
                String(myObj.p3),
                String(myObj.value),
                String(st_p1),
                String(st_parseInt),
                String(st_eval),
                String(del)
            ].join(":");
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(
        text,
        "1:x2:3:undefined:x5:x1:b:undefined:value:a:obj_parseInt:obj_eval:true"
    );
}

#[test]
fn evaluate_script_with_exception_restores_outer_name_resolution() {
    let unit = compile_test_unit(
        2405,
        r#"
            globalThis.x = 1;
            var obj = { x: 2, value: "boom" };
            var seen = "unset";
            try {
                with (obj) {
                    x = 3;
                    throw value;
                }
            } catch (error) {
                seen = String(x) + ":" + String(error);
            }
            seen + ":" + String(x) + ":" + String(obj.x);
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(text, "1:boom:1:3");
}

#[test]
fn evaluate_script_with_call_exception_restores_outer_name_resolution() {
    let unit = compile_test_unit(
        2406,
        r#"
            globalThis.x = 1;
            var value = "boom";
            var obj = { x: 2, value: "obj" };
            var f = function() {
                x = "x1";
                throw value;
            };
            var seen = "unset";
            try {
                with (obj) {
                    f();
                }
            } catch (error) {
                seen = String(x) + ":" + String(error);
            }
            seen + ":" + String(x) + ":" + String(obj.x);
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(text, "x1:boom:x1:2");
}

#[test]
fn evaluate_script_eval_with_statement_preserves_normal_completion_value() {
    let unit = compile_test_unit(
        2407,
        r#"
            String(eval('1; with({}) { }')) + ":" + String(eval('2; with({}) { 3; }'));
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(text, "undefined:3");
}

#[test]
fn evaluate_script_eval_with_statement_updates_empty_abrupt_completion() {
    let unit = compile_test_unit(
        2408,
        r#"
            [
                String(eval('1; do { 2; with({}) { 3; break; } 4; } while (false);')),
                String(eval('5; do { 6; with({}) { break; } 7; } while (false);')),
                String(eval('8; do { 9; with({}) { 10; continue; } 11; } while (false)')),
                String(eval('12; do { 13; with({}) { continue; } 14; } while (false)'))
            ].join(':');
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(text, "3:undefined:10:undefined");
}

#[test]
fn evaluate_script_direct_eval_with_statement_reads_object_binding() {
    let unit = compile_test_unit(
        2394,
        r#"
            function testcase(object) {
                with (object) {
                    return eval("x");
                }
            }
            testcase({ x: 7 }) === 7;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_with_statement_respects_symbol_unscopables() {
    let unit = compile_test_unit(
        2395,
        r#"
            var x = 1;
            var object = { x: 2 };
            object[Symbol.unscopables] = { x: true };
            with (object) {
                x;
            }
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn evaluate_script_with_statement_unscopables_keeps_global_hoisted_functions_callable() {
    let unit = compile_test_unit(
        2396,
        r#"
            function check(value) {
                return value + 1;
            }

            function wrap() {
                var object = {};
                object[Symbol.unscopables] = { check: true };
                return function(value) {
                    with (object) {
                        return globalThis.check(value) + check(value);
                    }
                };
            }

            wrap()(4);
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(10));
}

#[test]
fn evaluate_script_with_statement_call_target_only_uses_has_trap_for_missing_binding() {
    let unit = compile_test_unit(
        2395,
        r#"
            var log = [];
            function unexpected(name) {
                return function() {
                    throw new Error(name);
                };
            }
            var proxy = new Proxy({}, {
                getPrototypeOf: unexpected("[[GetPrototypeOf]]"),
                setPrototypeOf: unexpected("[[SetPrototypeOf]]"),
                isExtensible: unexpected("[[IsExtensible]]"),
                preventExtensions: unexpected("[[PreventExtensions]]"),
                getOwnPropertyDescriptor: unexpected("[[GetOwnProperty]]"),
                has(target, key) {
                    log.push("has:" + String(key));
                    return Reflect.has(target, key);
                },
                get: unexpected("[[Get]]"),
                set: unexpected("[[Set]]"),
                deleteProperty: unexpected("[[Delete]]"),
                defineProperty: unexpected("[[DefineOwnProperty]]"),
                ownKeys: unexpected("[[OwnPropertyKeys]]"),
                apply: unexpected("[[Call]]"),
                construct: unexpected("[[Construct]]")
            });
            var status = 0;

            try {
                with (proxy) {
                    Object();
                }
                if (log.length === 0) {
                    status = 0;
                } else if (log.length === 1 && log[0] === "has:Object") {
                    status = 1;
                } else {
                    status = 3;
                }
            } catch (error) {
                status = 2;
            }
            status;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn evaluate_script_with_statement_propagates_unscopables_get_errors() {
    let unit = compile_test_unit(
        2396,
        r#"
            var env = { x: 86 };
            Object.defineProperty(env, Symbol.unscopables, {
                get: function() {
                    throw "boom";
                }
            });
            var threw = false;

            try {
                with (env) {
                    x;
                }
            } catch (error) {
                threw = true;
            }
            threw;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_with_statement_rechecks_deleted_binding_after_unscopables() {
    let unit = compile_test_unit(
        2397,
        r#"
            var env = {
                binding: 0,
                get [Symbol.unscopables]() {
                    delete env.binding;
                    return null;
                }
            };
            var status = 0;
            try {
                with (env) {
                    binding;
                }
                status = 1;
            } catch (error) {
                if (error && error.name === "ReferenceError") {
                    status = 2;
                } else if (error && error.name === "TypeError") {
                    status = 3;
                } else {
                    status = 4;
                }
            }
            status;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn evaluate_script_with_statement_compound_assignment_uses_stable_identifier_reference() {
    let unit = compile_test_unit(
        2409,
        r#"
            var log = [];
            var env = { p: 0 };
            var proxy = new Proxy(env, {
                has(target, key) {
                    log.push("has:" + String(key));
                    return Reflect.has(target, key);
                },
                get(target, key, receiver) {
                    log.push("get:" + String(key));
                    return Reflect.get(target, key, receiver);
                },
                set(target, key, value, receiver) {
                    log.push("set:" + String(key));
                    return Reflect.set(target, key, value, receiver);
                },
                getOwnPropertyDescriptor(target, key) {
                    log.push("getOwnPropertyDescriptor:" + String(key));
                    return Reflect.getOwnPropertyDescriptor(target, key);
                },
                defineProperty(target, key, descriptor) {
                    log.push("defineProperty:" + String(key));
                    return Reflect.defineProperty(target, key, descriptor);
                }
            });

            with (proxy) {
                p += 1;
            }

            log.join("|");
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(
        text,
        "has:p|get:Symbol(Symbol.unscopables)|has:p|get:p|has:p|set:p|getOwnPropertyDescriptor:p|defineProperty:p"
    );
}

#[test]
fn evaluate_script_with_statement_inc_dec_looks_up_unscopables_once() {
    let unit = compile_test_unit(
        2410,
        r#"
            var unscopablesGetterCalled = 0;
            var a, b, flag = true;
            with (a = { x: 7 }) {
                with (b = {
                    x: 4,
                    get [Symbol.unscopables]() {
                        unscopablesGetterCalled++;
                        return { x: flag = !flag };
                    }
                }) {
                    x++;
                }
            }

            var incrementStatus = [
                unscopablesGetterCalled,
                a.x,
                b.x
            ].join(":");

            unscopablesGetterCalled = 0;
            flag = true;
            with (a = { x: 7 }) {
                with (b = {
                    x: 4,
                    get [Symbol.unscopables]() {
                        unscopablesGetterCalled++;
                        return { x: flag = !flag };
                    }
                }) {
                    x--;
                }
            }

            incrementStatus + ":" + [
                unscopablesGetterCalled,
                a.x,
                b.x
            ].join(":");
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(text, "1:7:5:1:7:3");
}

#[test]
fn evaluate_script_with_statement_strict_assignment_keeps_deleted_binding_reference() {
    let unit = compile_test_unit(
        2411,
        r#"
            var typedArray = new Int32Array(10);
            var env = Object.create(typedArray);

            Object.defineProperty(env, "NaN", {
                configurable: true,
                value: 100
            });

            var status = 0;
            with (env) {
                try {
                    (function() {
                        "use strict";
                        NaN = (delete env.NaN, 0);
                    })();
                    status = 1;
                } catch (error) {
                    status = error.constructor === ReferenceError ? 2 : 3;
                }
            }

            String(status) + ":" + String(Object.getOwnPropertyDescriptor(env, "NaN"));
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(text, "2:undefined");
}

#[test]
fn evaluate_script_with_statement_sloppy_assignment_keeps_deleted_binding_reference() {
    let unit = compile_test_unit(
        2412,
        r#"
            var typedArray = new Int32Array(10);
            var env = Object.create(typedArray);

            Object.defineProperty(env, "NaN", {
                configurable: true,
                value: 100
            });

            with (env) {
                NaN = (delete env.NaN, 0);
            }

            String(Object.getOwnPropertyDescriptor(env, "NaN"));
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(text, "undefined");
}

#[test]
fn evaluate_script_logical_and_assignment_infers_identifier_function_name() {
    let unit = compile_test_unit(
        2413,
        r#"
            var value = 1;
            value &&= function() {};
            value.name;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(text, "value");
}

#[test]
fn evaluate_script_logical_and_assignment_short_circuits_before_missing_setter() {
    let unit = compile_test_unit(
        2414,
        r#"
            "use strict";
            var obj = {};
            Object.defineProperty(obj, "prop", {
                get: function() {
                    return 0;
                },
                set: undefined,
                enumerable: true,
                configurable: true
            });
            obj.prop &&= 1;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(0));
}

#[test]
fn evaluate_script_logical_and_assignment_updates_private_fields() {
    let unit = compile_test_unit(
        2415,
        r#"
            class C {
                #field = true;
                compoundAssignment() {
                    return this.#field &&= false;
                }
                fieldValue() {
                    return this.#field;
                }
            }

            var instance = new C();
            String(instance.compoundAssignment()) + ":" + String(instance.fieldValue());
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(text, "false:false");
}

#[test]
fn evaluate_script_logical_and_assignment_throws_on_strict_non_writable_property_put() {
    let unit = compile_test_unit(
        2416,
        r#"
            "use strict";
            var obj = {};
            Object.defineProperty(obj, "prop", {
                value: 0,
                writable: false,
                enumerable: true,
                configurable: true
            });
            try {
                obj.prop ||= 1;
                false;
            } catch (error) {
                error instanceof TypeError;
            }
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_logical_and_assignment_updates_private_accessors() {
    let unit = compile_test_unit(
        2417,
        r#"
            class C {
                #value = 1;
                get #accessor() {
                    return this.#value;
                }
                set #accessor(next) {
                    this.#value = next + 1;
                }
                compoundAssignment() {
                    return this.#accessor &&= 2;
                }
                value() {
                    return this.#value;
                }
            }

            var instance = new C();
            String(instance.compoundAssignment()) + ":" + String(instance.value());
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(text, "2:3");
}

#[test]
fn evaluate_script_logical_and_assignment_checks_null_base_before_key_coercion() {
    let unit = compile_test_unit(
        2418,
        r#"
            var base = null;
            var keyEvaluated = false;
            var key = {
                toString: function() {
                    keyEvaluated = true;
                    throw new Error("property key evaluated");
                }
            };

            try {
                base[key] ||= 1;
                false;
            } catch (error) {
                error instanceof TypeError && !keyEvaluated;
            }
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_compound_assignment_keeps_initial_identifier_reference_across_eval_shadowing() {
    let unit = compile_test_unit(
        2419,
        r#"
            function testCompoundAssignment() {
              var x = 3;
              var innerX = (function() {
                x *= (eval("var x = 2;"), 4);
                return x;
              })();

              return String(innerX) + ":" + String(x);
            }

            testCompoundAssignment();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("script should return a string");

    assert_eq!(text, "2:12");
}

#[test]
fn evaluate_script_with_statement_closure_keeps_object_environment_live() {
    let unit = compile_test_unit(
        2398,
        r#"
            var reader;
            with ({ x: 7 }) {
                reader = function() {
                    return x;
                };
            }
            reader() === 7;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_installed_function_expression_closure_can_resolve_global_eval() {
    let unit = compile_test_unit(
        2397,
        r#"
            (function() {
                return eval("1");
            });
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let function_object = result
        .as_object_ref()
        .expect("script should return the function expression object");
    let Some(FunctionEntryIdentity::Bytecode(code)) = agent
        .objects()
        .function_data(function_object)
        .and_then(|data| data.entry())
    else {
        panic!("function expression should remain backed by installed bytecode");
    };
    let function = vm
        .installed_function(code)
        .expect("function expression bytecode should stay installed");
    let environment = agent
        .objects()
        .function_data(function_object)
        .and_then(|data| data.environment())
        .expect("function expression closure should preserve its outer environment");

    let closure_result = vm
        .evaluate_installed(
            agent,
            InstalledCode::new(code, function.id()),
            environment,
            environment,
        )
        .expect("closure bytecode should execute as a standalone entry");

    assert_eq!(closure_result, Value::from_smi(1));
}

#[test]
fn evaluate_script_with_statement_closure_reads_var_declared_inside_with() {
    let unit = compile_test_unit(
        2398,
        r#"
            var reader;
            with ({}) {
                var x = 7;
                reader = function() {
                    return x;
                };
            }
            reader() === 7;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_with_statement_single_statement_body_closure_keeps_var_binding() {
    let unit = compile_test_unit(
        2399,
        r#"
            var probeBody;
            with ({ x: 0 })
                var x = 1, _ = probeBody = function() { return x; };
            var x = 2;
            probeBody() * 10 + x;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(12));
}

#[test]
fn evaluate_script_with_statement_expression_and_body_keep_distinct_var_views() {
    let unit = compile_test_unit(
        2400,
        r#"
            var x = 0;
            var objectRecord = { x: 2 };
            var probeBefore = function() { return x; };
            var probeExpr, probeBody;

            with (eval('var x = 1;'), probeExpr = function() { return x; }, objectRecord)
                var x = 3, _ = probeBody = function() { return x; };

            objectRecord.x * 10000 + x * 1000 + probeBody() * 100 + probeExpr() * 10 + probeBefore();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(31311));
}

#[test]
fn evaluate_script_direct_eval_var_initializer_updates_existing_global_binding() {
    let unit = compile_test_unit(
        2401,
        r#"
            var x = 0;
            eval("var x = 1;");
            x;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn evaluate_script_direct_eval_var_initializer_returns_inner_value_and_updates_outer_binding() {
    let unit = compile_test_unit(
        2402,
        r#"
            var x = 0;
            eval("var x = 1; x;") * 10 + x;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(11));
}

#[test]
fn evaluate_script_string_literal_strict_equality_with_identifier() {
    let unit = compile_test_unit(
        2391,
        r#"
            var __10_4_2_1_1_1 = "str1";
            'str1' === __10_4_2_1_1_1;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_string_from_char_code_uses_uint16_code_units() {
    let unit = compile_test_unit(
        2392,
        r#"
            String.fromCharCode(65, 0x20ac, -1) === "A\u20ac\uffff";
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_string_search_uses_regexp_payloads() {
    let unit = compile_test_unit(
        2393,
        r#"
            "abc".search(/b/) === 1 && "\x00\u136c".search(/\0\u136c$/u) === 0;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_promise_finally_invokes_then_on_thenables() {
    let unit = compile_test_unit(
        239,
        r#"
            var thenResult = {};
            var seenThis = null;
            var seenArgs = null;
            function Thenable() {}
            Thenable.prototype.then = function(a, b) {
                seenThis = this;
                seenArgs = [a, b];
                return thenResult;
            };
            var target = new Thenable();
            Promise.prototype.finally.call(target) === thenResult
                && seenThis === target
                && seenArgs.length === 2
                && seenArgs[0] === undefined
                && seenArgs[1] === undefined;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_promise_finally_wraps_callable_handlers_before_invoking_then() {
    let unit = compile_test_unit(
        240,
        r#"
            var handler = function() {};
            var target = {
                then: function(onFulfilled, onRejected) {
                    return this === target
                        && typeof onFulfilled === "function"
                        && typeof onRejected === "function"
                        && onFulfilled !== handler
                        && onRejected !== handler
                        && onFulfilled.length === 1
                        && onRejected.length === 1;
                }
            };
            Promise.prototype.finally.call(target, handler);
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_promise_then_throws_when_constructor_is_null() {
    let unit = compile_test_unit(
        241,
        r#"
            var p = new Promise(function() {});
            p.constructor = null;
            try {
                p.then();
                false;
            } catch (error) {
                error instanceof TypeError;
            }
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_promise_all_settled_shares_already_called_between_element_functions() {
    let unit = compile_test_unit(
        242,
        r#"
            var rejectCallCount = 0;
            var returnValue = {};
            var error = new Error("boom");
            function Constructor(executor) {
                function reject(value) {
                    if (value !== error) {
                        return false;
                    }
                    rejectCallCount += 1;
                    return returnValue;
                }
                executor(function() { throw error; }, reject);
            }
            Constructor.resolve = function(value) {
                return value;
            };
            Constructor.reject = function(value) {
                return value;
            };
            var onRejected;
            var thenable = {
                then: function(onResolved, onRejectedArg) {
                    onRejected = onRejectedArg;
                    onResolved();
                }
            };
            Promise.allSettled.call(Constructor, [thenable]);
            rejectCallCount === 1
                && onRejected() === undefined
                && rejectCallCount === 1
                && onRejected() === undefined
                && rejectCallCount === 1;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_object_define_properties_applies_multiple_descriptors() {
    let unit = compile_test_unit(
        235,
        r#"
            var target = {};
            Object.defineProperties(target, {
                alpha: { value: 1, enumerable: true },
                beta: {
                    get: function() {
                        return 2;
                    },
                    enumerable: true
                }
            });
            target.alpha === 1
                && target.beta === 2
                && Object.keys(target).length === 2
                && Object.keys(target)[0] === "alpha"
                && Object.keys(target)[1] === "beta";
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_object_define_properties_normalizes_array_length() {
    let unit = compile_test_unit(
        236,
        r#"
            var target = [1, 2, 3];
            Object.defineProperties(target, {
                length: { value: 1 }
            });
            target.length === 1 && target[1] === undefined;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn aggregate_error_constructor_materializes_message_and_errors() {
    let unit = compile_test_unit(
        228,
        r#"
            var error = new AggregateError([1, 2], "boom");
            error instanceof AggregateError
                && error instanceof Error
                && error.message === "boom"
                && error.errors.length === 2
                && error.errors[0] === 1
                && error.errors[1] === 2;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn aggregate_error_constructor_installs_cause_property() {
    let unit = compile_test_unit(
        229,
        r#"
            var cause = { tag: "cause" };
            var error = new AggregateError([], "boom", { cause: cause });
            error.cause === cause;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn aggregate_error_evaluates_message_before_iterating_errors() {
    let unit = compile_test_unit(
        230,
        r#"
            var sequence = [];
            var message = {
                toString: function() {
                    sequence.push(1);
                    return "boom";
                }
            };
            var errors = {
                [Symbol.iterator]: function() {
                    sequence.push(2);
                    return {
                        next: function() {
                            sequence.push(3);
                            return { done: true };
                        }
                    };
                }
            };
            new AggregateError(errors, message);
            sequence.length === 3
                && sequence[0] === 1
                && sequence[1] === 2
                && sequence[2] === 3;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn suppressed_error_constructor_materializes_message_and_fields() {
    let unit = compile_test_unit(
        2301,
        r#"
            var error = { tag: "error" };
            var suppressed = { tag: "suppressed" };
            var value = new SuppressedError(error, suppressed, "boom");
            value instanceof SuppressedError
                && value instanceof Error
                && value.message === "boom"
                && value.error === error
                && value.suppressed === suppressed;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn disposable_stack_disposes_resources_in_lifo_order() {
    let unit = compile_test_unit(
        2302,
        r#"
            var log = [];
            var stack = new DisposableStack();
            stack.use({
                [Symbol.dispose]: function() {
                    log.push("use");
                }
            });
            stack.adopt("adopt", function(value) {
                log.push(value);
            });
            stack.defer(function() {
                log.push("defer");
            });
            stack.dispose();
            stack.disposed === true && log.join(",") === "defer,adopt,use";
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn disposable_stack_move_marks_source_disposed_and_transfers_resources() {
    let unit = compile_test_unit(
        2303,
        r#"
            var log = [];
            var stack = new DisposableStack();
            stack.defer(function() {
                log.push("done");
            });
            var moved = stack.move();
            var threw = false;
            try {
                stack.defer(function() {});
            } catch (error) {
                threw = error instanceof ReferenceError;
            }
            moved.dispose();
            stack.disposed === true
                && moved.disposed === true
                && threw
                && log.join(",") === "done";
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn async_disposable_stack_dispose_async_awaits_resources_in_lifo_order() {
    let unit = compile_test_unit(
        2304,
        r#"
            var log = [];
            var stack = new AsyncDisposableStack();
            stack.use({
                [Symbol.dispose]: function() {
                    log.push("sync");
                }
            });
            stack.use({
                [Symbol.asyncDispose]: function() {
                    log.push("async");
                    return Promise.resolve();
                }
            });
            stack.adopt("adopt", function(value) {
                log.push(value);
                return Promise.resolve();
            });
            stack.defer(function() {
                log.push("defer");
                return Promise.resolve();
            });
            stack.disposeAsync().then(function() {
                return stack.disposed === true && log.join(",") === "defer,adopt,async,sync";
            });
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let promise = result
        .as_object_ref()
        .expect("disposeAsync chain should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("disposeAsync result promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_bool(true));
}

#[test]
fn async_disposable_stack_rejects_with_nested_suppressed_error() {
    let unit = compile_test_unit(
        2305,
        r#"
            var stack = new AsyncDisposableStack();
            stack.defer(function() {
                return Promise.reject("first");
            });
            stack.defer(function() {
                return Promise.reject("second");
            });
            stack.disposeAsync().then(
                function() {
                    return false;
                },
                function(error) {
                    return error instanceof SuppressedError
                        && error.error === "first"
                        && error.suppressed === "second";
                }
            );
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let promise = result
        .as_object_ref()
        .expect("disposeAsync rejection chain should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("disposeAsync rejection promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_bool(true));
}

#[test]
fn using_statement_disposes_resource_on_block_exit() {
    let unit = compile_test_unit(
        2306,
        r#"
            var log = "";
            {
                using resource = {
                    [Symbol.dispose]: function() {
                        log += "D";
                    }
                };
                log += "B";
            }
            log;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let value = result
        .as_string_ref()
        .expect("using block should evaluate to a string");
    let decoded = decode_string(
        agent
            .heap()
            .view()
            .string_view(value)
            .expect("string should remain in the heap"),
    );

    assert_eq!(decoded, "BD");
}

#[test]
fn for_of_using_disposes_each_iteration_before_advancing() {
    let unit = compile_test_unit(
        2307,
        r#"
            var log = "";
            for (using resource of [
                {
                    [Symbol.dispose]: function() {
                        log += "a";
                    }
                },
                {
                    [Symbol.dispose]: function() {
                        log += "b";
                    }
                }
            ]) {
                log += "x";
            }
            log;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let value = result
        .as_string_ref()
        .expect("for-of using should evaluate to a string");
    let decoded = decode_string(
        agent
            .heap()
            .view()
            .string_view(value)
            .expect("string should remain in the heap"),
    );

    assert_eq!(decoded, "xaxb");
}

#[test]
fn await_using_waits_for_async_disposal_before_resolving() {
    let unit = compile_test_unit(
        2308,
        r#"
            async function run() {
                var log = "";
                {
                    await using resource = {
                        [Symbol.asyncDispose]: function() {
                            log += "D";
                            return Promise.resolve();
                        }
                    };
                    log += "B";
                }
                return log;
            }

            run().then(function(value) {
                return value === "BD";
            });
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let promise = result
        .as_object_ref()
        .expect("await using result should be a promise");
    let record = agent
        .promise_record(promise)
        .expect("await using promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_bool(true));
}

#[test]
fn evaluate_module_supports_top_level_await_using() {
    let unit = compile_test_module(
        2314,
        r#"
            export let disposed = false;
            await using resource = {
                [Symbol.dispose]() {
                    disposed = true;
                }
            };
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/await-using.mjs");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_module(agent, realm, &key, "/tmp/await-using.mjs", &unit)
        .unwrap();

    let record = agent
        .module_record(&key)
        .expect("evaluated module should stay cached on the agent");
    let module_env = record
        .environment()
        .expect("module evaluation should materialize a module environment");
    let disposed_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("disposed"))
        .expect("module should export disposed")
        .local_slot();

    assert_eq!(result, Value::undefined());
    assert_eq!(record.status(), ModuleStatus::Evaluated);
    assert_eq!(
        agent.environment_slot(module_env, disposed_slot),
        Some(Value::from_bool(true))
    );
}

#[test]
fn using_block_local_initializer_reads_trigger_reference_error() {
    let unit = compile_test_unit(
        2309,
        r#"
            try {
                {
                    using x = x + 1;
                }
                false;
            } catch (error) {
                error instanceof ReferenceError;
            }
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn using_function_local_initializer_reads_trigger_reference_error() {
    let unit = compile_test_unit(
        2310,
        r#"
            function f() {
                using x = x + 1;
            }

            try {
                f();
                false;
            } catch (error) {
                error instanceof ReferenceError;
            }
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn assigning_to_using_bindings_in_for_of_bodies_throws_type_error() {
    let unit = compile_test_unit(
        2311,
        r#"
            try {
                for (using x of [null]) {
                    x = { [Symbol.dispose]() {} };
                }
                false;
            } catch (error) {
                error instanceof TypeError;
            }
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn assigning_to_using_bindings_in_for_update_throws_type_error() {
    let unit = compile_test_unit(
        2312,
        r#"
            try {
                for (using i = null; i === null; i = { [Symbol.dispose]() {} }) {}
                false;
            } catch (error) {
                error instanceof TypeError;
            }
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn using_cleanup_nests_multiple_disposal_errors_as_suppressed_error() {
    let unit = compile_test_unit(
        2313,
        r#"
            class MyError extends Error {}
            const error1 = new MyError();
            const error2 = new MyError();
            const error3 = new MyError();

            try {
                using _1 = { [Symbol.dispose]() { throw error1; } };
                using _2 = { [Symbol.dispose]() { throw error2; } };
                throw error3;
            } catch (error) {
                error instanceof SuppressedError
                    && error.error === error1
                    && error.suppressed instanceof SuppressedError
                    && error.suppressed.error === error2
                    && error.suppressed.suppressed === error3;
            }
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn generator_call_returns_a_generator_without_running_the_body() {
    let unit = compile_test_unit(
        204,
        r#"
            var ran = 0;
            function* g() {
                ran = 1;
                yield 2;
            }
            var iter = g();
            ran;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let iter = global_value(agent, realm, "iter")
        .as_object_ref()
        .expect("generator call should return an object");

    assert_eq!(result, Value::from_smi(0));
    assert!(agent.objects().is_generator_object(iter));
}

#[test]
fn async_function_call_returns_a_promise_and_fulfills_after_await() {
    let unit = compile_test_unit(
        221,
        r#"
            async function f() {
                return await Promise.resolve(41);
            }
            f();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("async function call should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("async function promise should remain tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(41));
}

#[test]
fn async_function_call_rejects_the_returned_promise_on_await_rejection() {
    let unit = compile_test_unit(
        222,
        r#"
            async function f() {
                await Promise.reject(99);
            }
            f();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("async function call should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("async function promise should remain tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Rejected);
    assert_eq!(record.result(), Value::from_smi(99));
}

#[test]
fn async_functions_use_the_async_function_prototype_chain() {
    let unit = compile_test_unit(
        223,
        r#"
            var sample = async function demo() {};
            var AsyncFunction = Object.getPrototypeOf(sample).constructor;
            var sameProto = Object.getPrototypeOf(sample) === AsyncFunction.prototype;
            var notPlainFunction = AsyncFunction !== Function;
            var tag = AsyncFunction.prototype[Symbol.toStringTag];
            sameProto && notPlainFunction && AsyncFunction.name === "AsyncFunction" && tag === "AsyncFunction";
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn async_function_constructor_compiles_dynamic_async_bodies() {
    let unit = compile_test_unit(
        224,
        r#"
            var AsyncFunction = Object.getPrototypeOf(async function() {}).constructor;
            var fn = new AsyncFunction("value", "return await Promise.resolve(value + 1);");
            fn(4);
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("dynamic AsyncFunction call should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("dynamic AsyncFunction promise should remain tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(5));
}

#[test]
fn for_await_of_consumes_async_iterators() {
    let unit = compile_test_unit(
        225,
        r#"
            var iterable = {
                [Symbol.asyncIterator]() {
                    var index = 0;
                    return {
                        next() {
                            index += 1;
                            if (index > 2) {
                                return Promise.resolve({ value: undefined, done: true });
                            }
                            return Promise.resolve({ value: index, done: false });
                        }
                    };
                }
            };
            async function main() {
                var sum = 0;
                for await (const value of iterable) {
                    sum += value;
                }
                return sum;
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("for await should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("for await promise should remain tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(3));
}

#[test]
fn for_await_of_wraps_sync_iterators_and_awaits_each_value() {
    let unit = compile_test_unit(
        226,
        r#"
            async function main() {
                var sum = 0;
                for await (const value of [Promise.resolve(1), 2]) {
                    sum += value;
                }
                return sum;
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("for await should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("for await promise should remain tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(3));
}

#[test]
fn for_await_of_break_rejects_when_async_close_rejects() {
    let unit = compile_test_unit(
        227,
        r#"
            var closed = 0;
            var iterable = {
                [Symbol.asyncIterator]() {
                    return {
                        next() {
                            return Promise.resolve({ value: 1, done: false });
                        },
                        return() {
                            closed += 1;
                            return Promise.reject(new Error("close"));
                        }
                    };
                }
            };
            async function main() {
                for await (const value of iterable) {
                    break;
                }
                return closed;
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("for await should return a promise");
    let state = agent
        .promise_record(promise)
        .expect("for await promise should remain tracked")
        .state();
    let promise_result = agent
        .promise_record(promise)
        .expect("for await promise should remain tracked")
        .result();

    assert_eq!(state, lyng_js_env::PromiseState::Rejected);
    let reason = promise_result
        .as_object_ref()
        .expect("async iterator close rejection should reject with an error object");
    let name_atom = agent.atoms_mut().intern_collectible("name");
    let name = ordinary_get(agent, reason, PropertyKey::from_atom(name_atom))
        .expect("error name should be readable")
        .as_string_ref()
        .and_then(|string| agent.heap().view().string_view(string))
        .map(decode_string)
        .expect("error name should be a string");
    assert_eq!(name, "Error");
}

#[test]
fn for_await_of_return_closes_wrapped_sync_iterators_and_awaits_return_value() {
    let unit = compile_test_unit(
        228,
        r#"
            var state = { closed: 0 };
            var iterable = {
                [Symbol.iterator]() {
                    var done = false;
                    return {
                        next() {
                            if (done) {
                                return { value: undefined, done: true };
                            }
                            done = true;
                            return { value: 1, done: false };
                        },
                        return() {
                            state.closed += 1;
                            return { value: Promise.resolve(0), done: true };
                        }
                    };
                }
            };
            async function main() {
                for await (const value of iterable) {
                    return value;
                }
                return 99;
            }
            main().then(function(value) {
                return value + state.closed * 10;
            });
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("for await should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("for await promise should remain tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(11));
}

#[test]
fn for_await_of_return_preserves_value_with_async_iterator_close() {
    let unit = compile_test_unit(
        229,
        r#"
            async function main() {
                var iterable = {
                    [Symbol.asyncIterator]() {
                        return {
                            next() {
                                return Promise.resolve({ value: 1, done: false });
                            },
                            return() {
                                return Promise.resolve({ value: undefined, done: true });
                            }
                        };
                    }
                };
                for await (const value of iterable) {
                    return value;
                }
                return 99;
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("for await should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("for await promise should remain tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(1));
}

#[test]
fn for_await_of_return_preserves_value_with_sync_wrapper_close_without_await() {
    let unit = compile_test_unit(
        230,
        r#"
            async function main() {
                var iterable = {
                    [Symbol.iterator]() {
                        var done = false;
                        return {
                            next() {
                                if (done) {
                                    return { value: undefined, done: true };
                                }
                                done = true;
                                return { value: 1, done: false };
                            },
                            return() {
                                return { value: 0, done: true };
                            }
                        };
                    }
                };
                for await (const value of iterable) {
                    return value;
                }
                return 99;
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("for await should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("for await promise should remain tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(1));
}

#[test]
fn async_generator_next_returns_a_promise_for_iterator_results() {
    let unit = compile_test_unit(
        231,
        r#"
            async function main() {
                var iter = (async function* () {
                    yield 1;
                    return 2;
                })();
                var firstPromise = iter.next();
                var secondPromise = iter.next();
                if (!(firstPromise instanceof Promise) || !(secondPromise instanceof Promise)) {
                    return -1;
                }
                var first = await firstPromise;
                var second = await secondPromise;
                return first.value + second.value;
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("async generator test should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("async generator promise should remain tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(3));
}

#[test]
fn async_generator_yield_unwraps_promises_before_resolving_next() {
    let unit = compile_test_unit(
        232,
        r#"
            async function main() {
                var iter = (async function* () {
                    yield Promise.resolve(7);
                })();
                var result = await iter.next();
                return result.value instanceof Promise ? -1 : result.value;
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("async generator promise-yield test should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("async generator promise-yield promise should remain tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(7));
}

#[test]
fn async_generator_awaits_within_the_body_before_settling_next_requests() {
    let unit = compile_test_unit(
        232,
        r#"
            async function main() {
                var iter = (async function* () {
                    yield await Promise.resolve(1);
                    return await Promise.resolve(2);
                })();
                var firstPromise = iter.next();
                var secondPromise = iter.next();
                var first = await firstPromise;
                var second = await secondPromise;
                return first.value + second.value;
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("async generator await test should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("async generator await promise should remain tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(3));
}

#[test]
fn async_generator_yield_star_uses_async_iterator_hint() {
    let unit = compile_test_unit(
        333,
        r#"
            async function main() {
                var touchedSyncIterator = 0;
                var iterable = {
                    get [Symbol.iterator]() {
                        touchedSyncIterator = 1;
                        throw new Error("sync iterator should not be read");
                    },
                    [Symbol.asyncIterator]: false
                };
                var iter = (async function* () {
                    yield* iterable;
                })();
                try {
                    await iter.next();
                    return -1;
                } catch (error) {
                    return (error.constructor === TypeError ? 1 : 0) + touchedSyncIterator * 10;
                }
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("async generator yield-star test should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("async generator yield-star promise should remain tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(1));
}

#[test]
fn async_generator_private_yield_star_awaits_async_iterator_next_results() {
    let unit = compile_test_unit(
        334,
        r#"
            async function main() {
                var log = "";
                var yielded = Promise.resolve(5);
                var iterable = {
                    [Symbol.asyncIterator]() {
                        var count = 0;
                        return {
                            next(value) {
                                log += "next:" + String(value) + ";";
                                count += 1;
                                if (count === 1) {
                                    return Promise.resolve({
                                        value: yielded,
                                        done: false
                                    });
                                }
                                return Promise.resolve({
                                    value: "done",
                                    done: true
                                });
                            }
                        };
                    }
                };
                class C {
                    async *#gen() {
                        var completion = yield* iterable;
                        return "ret:" + completion;
                    }
                    gen() {
                        return this.#gen();
                    }
                }
                var iter = new C().gen();
                var first = await iter.next("ignored");
                var second = await iter.next("sent");
                return String(first.value === yielded) + ":" + first.done
                    + "|" + second.value + ":" + second.done
                    + "|" + log;
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("async generator delegated yield-star test should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("async generator delegated yield-star promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    let text = record
        .result()
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("async generator delegated yield-star result should be a string");

    assert_eq!(text, "true:false|ret:done:true|next:undefined;next:sent;");
}

#[test]
fn async_generator_yield_star_missing_return_method_awaits_return_value() {
    let unit = compile_test_unit(
        335,
        r#"
            async function main() {
                var iterable = {
                    [Symbol.asyncIterator]() {
                        return this;
                    },
                    next() {
                        return {
                            value: 1,
                            done: false
                        };
                    }
                };
                var iter = (async function* () {
                    yield* iterable;
                })();
                await iter.next();
                var returnValue = Promise.resolve(2).then(() => 3);
                var result = await iter.return(returnValue);
                return result.value === returnValue ? -1 : result.value;
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("async generator return-value await test should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("async generator return-value await promise should remain tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(3));
}

#[test]
fn async_generator_explicit_return_undefined_awaits_before_settling() {
    let unit = compile_test_unit(
        336,
        r#"
            async function main() {
                var actual = [];
                Promise.resolve(0)
                    .then(() => actual.push("tick 1"))
                    .then(() => actual.push("tick 2"));

                async function* implicitReturn() {}
                async function* bareReturn() {
                    return;
                }
                async function* explicitReturn() {
                    return undefined;
                }

                implicitReturn().next().then(() => actual.push("implicit"));
                bareReturn().next().then(() => actual.push("bare"));
                explicitReturn().next().then(() => actual.push("explicit"));

                await Promise.resolve();
                await Promise.resolve();
                await Promise.resolve();
                return actual.join("|");
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("async generator explicit-return timing test should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("async generator explicit-return timing promise should remain tracked");
    let text = record
        .result()
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("async generator explicit-return timing should return a string");

    assert_eq!(text, "tick 1|implicit|bare|tick 2|explicit");
}

#[test]
fn async_generator_yield_return_resumption_awaits_return_value() {
    let unit = compile_test_unit(
        337,
        r#"
            async function main() {
                var actual = [];
                async function* f() {
                    actual.push("start");
                    yield 123;
                    actual.push("stop");
                }

                Promise.resolve(0)
                    .then(() => actual.push("tick 1"))
                    .then(() => actual.push("tick 2"));

                var iter = f();
                iter.next();
                iter.return({
                    get then() {
                        actual.push("get then");
                    }
                });

                await Promise.resolve();
                await Promise.resolve();
                await Promise.resolve();
                return actual.join("|");
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("async generator yield-return timing test should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("async generator yield-return timing promise should remain tracked");
    let text = record
        .result()
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("async generator yield-return timing should return a string");

    assert_eq!(text, "start|tick 1|get then|tick 2");
}

#[test]
fn async_generator_yield_star_return_resumption_awaits_before_delegate_return_lookup() {
    let unit = compile_test_unit(
        338,
        r#"
            async function main() {
                var actual = [];
                var asyncIter = {
                    [Symbol.asyncIterator]() {
                        return this;
                    },
                    next() {
                        return {
                            done: false
                        };
                    },
                    get return() {
                        actual.push("get return");
                    }
                };
                async function* f() {
                    actual.push("start");
                    yield* asyncIter;
                    actual.push("stop");
                }

                Promise.resolve(0)
                    .then(() => actual.push("tick 1"))
                    .then(() => actual.push("tick 2"))
                    .then(() => actual.push("tick 3"));

                var iter = f();
                iter.next();
                iter.return({
                    get then() {
                        actual.push("get then");
                    }
                });

                await Promise.resolve();
                await Promise.resolve();
                await Promise.resolve();
                await Promise.resolve();
                return actual.join("|");
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("async generator yield-star return timing test should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("async generator yield-star return timing promise should remain tracked");
    let text = record
        .result()
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("async generator yield-star return timing should return a string");

    assert_eq!(
        text,
        "start|tick 1|get then|tick 2|get return|get then|tick 3"
    );
}

#[test]
fn async_generator_functions_use_the_async_generator_prototype_chain() {
    let unit = compile_test_unit(
        233,
        r#"
            var fn = async function* named() {};
            var iter = fn();
            var AsyncGeneratorFunction = Object.getPrototypeOf(fn).constructor;
            var AsyncGeneratorFunctionPrototype = Object.getPrototypeOf(fn);
            var AsyncGeneratorPrototype = Object.getPrototypeOf(fn.prototype);
            var AsyncIteratorPrototype = Object.getPrototypeOf(AsyncGeneratorPrototype);

            AsyncGeneratorFunction.name === "AsyncGeneratorFunction" &&
                AsyncGeneratorFunctionPrototype === AsyncGeneratorFunction.prototype &&
                AsyncGeneratorFunctionPrototype.constructor === AsyncGeneratorFunction &&
                AsyncGeneratorFunctionPrototype.prototype === AsyncGeneratorPrototype &&
                AsyncGeneratorPrototype.constructor === AsyncGeneratorFunctionPrototype &&
                Object.getPrototypeOf(iter) === fn.prototype &&
                AsyncIteratorPrototype[Symbol.asyncIterator].call(iter) === iter &&
                Object.prototype.toString.call(AsyncGeneratorPrototype) === "[object AsyncGenerator]" &&
                Object.prototype.toString.call(AsyncIteratorPrototype) === "[object AsyncIterator]";
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn async_iterator_prototype_async_iterator_method_has_spec_name() {
    let unit = compile_test_unit(
        234,
        r#"
            var AsyncIteratorPrototype = Object.getPrototypeOf(
                Object.getPrototypeOf((async function* () {}).prototype)
            );
            var method = AsyncIteratorPrototype[Symbol.asyncIterator];
            var descriptor = Object.getOwnPropertyDescriptor(method, "name");

            method.name === "[Symbol.asyncIterator]" &&
                descriptor.value === "[Symbol.asyncIterator]" &&
                descriptor.writable === false &&
                descriptor.enumerable === false &&
                descriptor.configurable === true;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn async_generator_function_constructor_compiles_dynamic_async_generator_bodies() {
    let unit = compile_test_unit(
        235,
        r#"
            async function main() {
                var AsyncGeneratorFunction = Object.getPrototypeOf(async function* () {}).constructor;
                var fn = AsyncGeneratorFunction(
                    "value",
                    "yield await Promise.resolve(value); return await Promise.resolve(value + 1);"
                );
                var iter = fn(2);
                var firstPromise = iter.next();
                var secondPromise = iter.next();
                if (!(firstPromise instanceof Promise) || !(secondPromise instanceof Promise)) {
                    return -1;
                }
                var first = await firstPromise;
                var second = await secondPromise;
                return first.value + second.value + (second.done ? 10 : 0);
            }
            main();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let promise = result
        .as_object_ref()
        .expect("dynamic async generator constructor test should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("dynamic async generator promise should remain tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_smi(15));
}

#[test]
fn generator_call_runs_parameter_instantiation_before_suspending_start() {
    let unit = compile_test_unit(
        214,
        r#"
            var calls = 0;
            function* g(x = (calls += 1, 41)) {
                calls += 100;
                yield x;
            }
            var iter = g();
            calls;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let iter = global_value(agent, realm, "iter")
        .as_object_ref()
        .expect("generator call should return an object");

    assert_eq!(result, Value::from_smi(1));
    assert!(agent.objects().is_generator_object(iter));
}

#[test]
fn generator_call_throws_parameter_instantiation_errors_before_returning() {
    let unit = compile_test_unit(
        215,
        r#"
            var ran = 0;
            function* g(x = x) {
                ran = 1;
            }
            var status = 0;
            try {
                g();
                status = 1;
            } catch (error) {
                status = error.constructor === ReferenceError ? 2 : 3;
            }
            status * 10 + (ran === 0 ? 1 : 0);
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(21));
}

#[test]
fn generator_call_creates_instances_after_parameter_side_effects() {
    let unit = compile_test_unit(
        216,
        r#"
            var g = function*(a = (g.prototype = null)) {};
            var oldPrototype = g.prototype;
            var iter = g();
            Object.getPrototypeOf(iter) === oldPrototype;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(false));
}

#[test]
fn generator_instances_use_the_function_prototype_chain() {
    let unit = compile_test_unit(
        210,
        r#"
            function* g() {}
            var iter = g();
            var directProtoMatches = Object.getPrototypeOf(iter) === g.prototype;
            var sharedProto = Object.getPrototypeOf(g.prototype);
            var sharedNext = typeof sharedProto.next;
            var sharedTag = sharedProto[Symbol.toStringTag];
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let _ = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(
        global_value(agent, realm, "directProtoMatches"),
        Value::from_bool(true)
    );
    let shared_next = global_value(agent, realm, "sharedNext")
        .as_string_ref()
        .expect("typeof shared prototype next should be a string");
    let shared_tag = global_value(agent, realm, "sharedTag")
        .as_string_ref()
        .expect("shared prototype tag should be a string");

    assert_eq!(
        decode_string(agent.heap().view().string_view(shared_next).unwrap()),
        "function"
    );
    assert_eq!(
        decode_string(agent.heap().view().string_view(shared_tag).unwrap()),
        "Generator"
    );
}

#[test]
fn generator_function_constructor_rejections_throw_syntax_error() {
    let unit = compile_test_unit(
        211,
        r#"
            var GeneratorFunction = Object.getPrototypeOf(function*() {}).constructor;
            var status = 0;
            GeneratorFunction("x = yield");
            try {
                GeneratorFunction("x = yield", "");
                status = 1;
            } catch (error) {
                status = error.constructor === SyntaxError ? 2 : 3;
            }
            status;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(2));
}

#[test]
fn subclassed_generator_function_constructors_preserve_generator_descriptors() {
    let unit = compile_test_unit(
        220,
        r#"
            var GeneratorFunction = Object.getPrototypeOf(function*() {}).constructor;
            class GFn extends GeneratorFunction {}
            var gfn = new GFn("a", "return a + 1");
            var name = Object.getOwnPropertyDescriptor(gfn, "name");
            var prototype = Object.getOwnPropertyDescriptor(gfn, "prototype");
            var plainGeneratorPrototype = Object.getPrototypeOf((function* () {}).prototype);
            (typeof gfn === "function")
                && (name.value === "anonymous")
                && (name.writable === false)
                && (name.enumerable === false)
                && (name.configurable === true)
                && (prototype.writable === true)
                && (prototype.enumerable === false)
                && (prototype.configurable === false)
                && (Object.keys(gfn.prototype).length === 0)
                && (!Object.prototype.hasOwnProperty.call(gfn.prototype, "constructor"))
                && (Object.getPrototypeOf(gfn.prototype) === plainGeneratorPrototype);
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn explicit_derived_constructors_can_fall_through_after_super_call() {
    let unit = compile_test_unit(
        221,
        r#"
            class Base {
                constructor(x) {
                    this.foobar = x;
                }
            }
            class Subclass extends Base {
                constructor(x) {
                    super(x);
                }
            }
            var instance = new Subclass(1);
            instance.foobar === 1 && Object.getPrototypeOf(instance) === Subclass.prototype;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn explicit_derived_constructors_can_return_this_immediately_after_super_call() {
    let unit = compile_test_unit(
        222,
        r#"
            class Base {
                constructor(x) {
                    this.foobar = x;
                }
            }
            class Subclass extends Base {
                constructor(x) {
                    super(x);
                    return this;
                }
            }
            new Subclass(1).foobar === 1;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn explicit_derived_constructors_throw_for_non_undefined_primitive_returns() {
    let unit = compile_test_unit(
        223,
        r#"
            class Base {
                constructor() {}
            }
            class Derived extends Base {
                constructor() {
                    super();
                    return 0;
                }
            }
            var status = 0;
            try {
                new Derived();
                status = 1;
            } catch (error) {
                status = error.constructor === TypeError ? 2 : 3;
            }
            status;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(2));
}

#[test]
fn explicit_derived_constructors_preserve_object_return_overrides() {
    let unit = compile_test_unit(
        224,
        r#"
            class Base {
                constructor() {
                    this.base = true;
                }
            }
            class Derived extends Base {
                constructor() {
                    super();
                    return { overridden: true };
                }
            }
            var result = new Derived();
            result.overridden === true
                && result.base === undefined
                && !(result instanceof Derived);
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn instanceof_accepts_class_constructors_as_rhs() {
    let unit = compile_test_unit(
        225,
        r#"
            class Base {}
            let value = new Base();
            value instanceof Base;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn class_heritage_functions_use_strict_arguments_objects() {
    let unit = compile_test_unit(
        237,
        r#"
            var status = 0;
            var D = class extends function() {
                arguments.callee;
            } {};
            try {
                Object.getPrototypeOf(D).arguments;
            } catch (error) {
                if (error.constructor === TypeError) {
                    status += 1;
                }
            }
            try {
                new D;
            } catch (error) {
                if (error.constructor === TypeError) {
                    status += 2;
                }
            }
            status;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn arrow_super_calls_in_finally_initialize_the_enclosing_derived_constructor() {
    let unit = compile_test_unit(
        226,
        r#"
            class Derived extends class {} {
                constructor() {
                    var callSuper = () => super();
                    try {
                        return;
                    } finally {
                        callSuper();
                    }
                }
            }
            typeof new Derived() === "object";
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn class_declaration_heritage_self_reference_throws_reference_error() {
    let unit = compile_test_unit(
        226,
        r#"
            var status = 0;
            try {
                class x extends x {}
                status = 1;
            } catch (error) {
                status = error.constructor === ReferenceError ? 2 : 3;
            }
            status;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(2));
}

#[test]
fn class_expression_heritage_self_reference_throws_reference_error() {
    let unit = compile_test_unit(
        227,
        r#"
            var status = 0;
            try {
                (class x extends x {});
                status = 1;
            } catch (error) {
                status = error.constructor === ReferenceError ? 2 : 3;
            }
            status;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(2));
}

#[test]
fn class_static_block_super_property_uses_parent_class_home_object() {
    let unit = compile_test_unit(
        228,
        r#"
            function Parent() {}
            Parent.test262 = "test262";
            var value = "";

            class C extends Parent {
                static {
                    value = super.test262;
                }
            }

            value;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = decode_string(
        agent
            .heap()
            .view()
            .string_view(result.as_string_ref().expect("value should be a string"))
            .expect("string should be allocated"),
    );

    assert_eq!(text, "test262");
}

#[test]
fn evaluate_script_super_property_compound_assignment_uses_stable_reference() {
    let unit = compile_test_unit(
        231,
        r#"
            var log = [];

            class Base {
                get value() {
                    log.push("get");
                    return this._value;
                }

                set value(next) {
                    log.push("set:" + String(next));
                    this._value = next;
                }
            }

            class Derived extends Base {
                constructor() {
                    super();
                    this._value = 2;
                }

                add() {
                    return super.value += 3;
                }
            }

            var instance = new Derived();
            String(instance.add()) + ":" + String(instance._value) + ":" + log.join("|");
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = decode_string(
        agent
            .heap()
            .view()
            .string_view(result.as_string_ref().expect("value should be a string"))
            .expect("string should be allocated"),
    );

    assert_eq!(text, "5:5:get|set:5");
}

#[test]
fn instance_field_arrow_functions_preserve_super_binding() {
    let unit = compile_test_unit(
        229,
        r#"
            class C {
                func = () => {
                    super.prop = "test262";
                };
            }

            let c = new C();
            c.func();
            c.prop === "test262";
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn static_field_arrow_functions_preserve_super_binding() {
    let unit = compile_test_unit(
        230,
        r#"
            class C {
                static staticFunc = () => {
                    super.staticProp = "static test262";
                };
            }

            C.staticFunc();
            C.staticProp === "static test262";
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn derived_constructors_reject_this_before_super_even_when_super_returns_an_object() {
    let unit = compile_test_unit(
        231,
        r#"
            class Base {
                constructor(a, b) {
                    return { prp: a + b };
                }
            }

            var ok = false;
            class Subclass extends Base {
                constructor(a, b) {
                    var before = 0;
                    try {
                        this.prp1 = 3;
                        before = 1;
                    } catch (error) {
                        before = error.constructor === ReferenceError ? 2 : 3;
                    }
                    super(a, b);
                    ok = before === 2
                        && this.prp === a + b
                        && this.prp1 === undefined
                        && !this.hasOwnProperty("prp1");
                }
            }

            var result = new Subclass(2, -1);
            ok
                && result.prp === 1
                && result.prp1 === undefined
                && !result.hasOwnProperty("prp1");
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn derived_constructors_throw_reference_error_on_second_super_after_evaluating_arguments() {
    let unit = compile_test_unit(
        232,
        r#"
            class Base {
                constructor(a, b) {
                    this.prp = a + b;
                }
            }

            var ok = false;
            class Subclass extends Base {
                constructor() {
                    super(1, 2);
                    var called = false;
                    function tmp() {
                        called = true;
                        return 3;
                    }
                    var status = 0;
                    try {
                        super(tmp(), 4);
                        status = 1;
                    } catch (error) {
                        status = error.constructor === ReferenceError ? 2 : 3;
                    }
                    ok = status === 2 && called === true;
                }
            }

            var result = new Subclass();
            ok && result.prp === 3;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn derived_constructors_without_super_throw_reference_error_on_fallthrough() {
    let unit = compile_test_unit(
        233,
        r#"
            class Base {
                constructor() {}
            }

            class BadSubclass extends Base {
                constructor() {}
            }

            var status = 0;
            try {
                new BadSubclass();
                status = 1;
            } catch (error) {
                status = error.constructor === ReferenceError ? 2 : 3;
            }
            status;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(2));
}

#[test]
fn derived_constructor_this_access_restriction_matches_test262_behavior() {
    let unit = compile_test_unit(
        234,
        r#"
            class Base {
                constructor(a, b) {
                    var o = new Object();
                    o.prp = a + b;
                    return o;
                }
            }

            var ok1 = false;
            class Subclass extends Base {
                constructor(a, b) {
                    var exn;
                    try {
                        this.prp1 = 3;
                    } catch (error) {
                        exn = error;
                    }
                    super(a, b);
                    ok1 =
                        exn instanceof ReferenceError
                        && this.prp === a + b
                        && this.prp1 === undefined
                        && !this.hasOwnProperty("prp1");
                    return this;
                }
            }

            var b = new Base(1, 2);
            var s = new Subclass(2, -1);

            var ok2 = false;
            class Subclass2 extends Base {
                constructor(x) {
                    super(1, 2);
                    if (x < 0) return;

                    var called = false;
                    function tmp() {
                        called = true;
                        return 3;
                    }
                    var exn = null;
                    try {
                        super(tmp(), 4);
                    } catch (error) {
                        exn = error;
                    }
                    ok2 = exn instanceof ReferenceError && called === true;
                }
            }

            var s2 = new Subclass2(1);
            var s3 = new Subclass2(-1);

            var subclass_call_type_error = false;
            try {
                Subclass.call(new Object(), 1, 2);
            } catch (error) {
                subclass_call_type_error = error instanceof TypeError;
            }

            var base_call_type_error = false;
            try {
                Base.call(new Object(), 1, 2);
            } catch (error) {
                base_call_type_error = error instanceof TypeError;
            }

            class BadSubclass extends Base {
                constructor() {}
            }

            var bad_subclass_reference_error = false;
            try {
                new BadSubclass();
            } catch (error) {
                bad_subclass_reference_error = error instanceof ReferenceError;
            }

            b.prp === 3
                && ok1
                && s.prp === 1
                && s.prp1 === undefined
                && !s.hasOwnProperty("prp1")
                && ok2
                && s2.prp === 3
                && s3.prp === 3
                && subclass_call_type_error
                && base_call_type_error
                && bad_subclass_reference_error;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn class_constructor_function_call_builtin_throws_type_error_catchably() {
    let unit = compile_test_unit(
        235,
        r#"
            class Base {
                constructor() {}
            }

            var status = 0;
            try {
                Base.call(new Object(), 1, 2);
                status = 1;
            } catch (error) {
                status = error instanceof TypeError ? 2 : 3;
            }
            status;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(2));
}

#[test]
fn derived_class_constructor_function_call_builtin_throws_type_error_catchably() {
    let unit = compile_test_unit(
        236,
        r#"
            class Base {
                constructor() {}
            }

            class Subclass extends Base {
                constructor(a, b) {
                    super(a, b);
                    return this;
                }
            }

            var status = 0;
            try {
                Subclass.call(new Object(), 1, 2);
                status = 1;
            } catch (error) {
                status = error instanceof TypeError ? 2 : 3;
            }
            status;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(2));
}

#[test]
fn generator_length_uses_expected_argument_count() {
    let unit = compile_test_unit(
        217,
        r#"
            var lengths = [
                (function* (x = 42) {}).length,
                (function* (x, y = 42) {}).length,
                (function* (x, y = 42, z) {}).length
            ];
            lengths[0] * 100 + lengths[1] * 10 + lengths[2];
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(11));
}

#[test]
fn anonymous_generator_expressions_default_name_to_empty_string() {
    let unit = compile_test_unit(
        218,
        r#"
            (function* () {}).name.length;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(0));
}

#[test]
fn named_generator_expression_self_binding_ignores_sloppy_reassignment() {
    let unit = compile_test_unit(
        219,
        r#"
            let callCount = 0;
            let ref = function* BindingIdentifier() {
                callCount++;
                BindingIdentifier = 1;
                return BindingIdentifier;
            };
            let result = ref().next().value === ref ? 1 : 0;
            result * 10 + callCount;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(11));
}

#[test]
fn array_is_array_is_installed_for_test262_property_helpers() {
    let unit = compile_test_unit(
        212,
        r#"
            var status = 0;
            if (typeof Array.isArray === "function") status += 1;
            if (Array.isArray([])) status += 2;
            if (!Array.isArray({})) status += 4;
            if (!Array.isArray(function() {})) status += 8;
            if (!Array.isArray(function*() {})) status += 16;
            status;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn generator_object_spread_uses_copy_data_properties() {
    let unit = compile_test_unit(
        213,
        r#"
            function* gen() {
                yield {
                    ...yield,
                    y: 1,
                    ...yield yield,
                };
            }
            var iter = gen();
            iter.next();
            iter.next({ x: 42 });
            iter.next({ x: "lol" });
            var item = iter.next({ y: 39 });
            var summary = item.value.x + ":" + item.value.y + ":" + Object.keys(item.value).length + ":" + item.done;
            summary;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("summary should be a string");

    assert_eq!(text, "42:39:2:false");
}

#[test]
fn generator_next_resumes_with_the_sent_value() {
    let unit = compile_test_unit(
        205,
        r#"
            function* g() {
                const value = yield 1;
                return value + 1;
            }
            var iter = g();
            var first = iter.next();
            var second = iter.next(41);
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let _ = vm.evaluate_script(agent, realm, &unit).unwrap();
    let first = global_value(agent, realm, "first");
    let second = global_value(agent, realm, "second");
    let (first_value, first_done) = iterator_result_fields(agent, first);
    let (second_value, second_done) = iterator_result_fields(agent, second);

    assert_eq!(first_value, Value::from_smi(1));
    assert_eq!(first_done, Value::from_bool(false));
    assert_eq!(second_value, Value::from_smi(42));
    assert_eq!(second_done, Value::from_bool(true));
}

#[test]
fn generator_return_runs_finally_before_completing() {
    let unit = compile_test_unit(
        206,
        r#"
            var finalized = 0;
            function* g() {
                try {
                    yield 1;
                } finally {
                    finalized = 7;
                }
            }
            var iter = g();
            iter.next();
            var result = iter.return(5);
            finalized;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let final_state = vm.evaluate_script(agent, realm, &unit).unwrap();
    let result = global_value(agent, realm, "result");
    let (value, done) = iterator_result_fields(agent, result);

    assert_eq!(final_state, Value::from_smi(7));
    assert_eq!(value, Value::from_smi(5));
    assert_eq!(done, Value::from_bool(true));
}

#[test]
fn for_of_continue_to_outer_loop_closes_the_iterator() {
    let unit = compile_test_unit(
        208,
        r#"
            var finalized = 0;
            var count = 0;
            function* values() {
                try {
                    yield 1;
                } finally {
                    finalized += 1;
                }
            }
            var keep = true;
            outer:
            while (keep) {
                keep = false;
                for (var value of values()) {
                    count += value;
                    continue outer;
                }
            }
            finalized * 10 + count;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(11));
}

#[test]
fn for_of_destructuring_assignment_defaults_only_for_undefined_values() {
    let unit = compile_test_unit(
        209,
        r#"
            var flag1 = false;
            var flag2 = false;
            var x, y;
            var counter = 0;

            for ({ x = flag1 = true, y = flag2 = true } of [{ y: 1 }]) {
                counter += 1;
            }
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let _ = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(global_value(agent, realm, "counter"), Value::from_smi(1));
    assert_eq!(global_value(agent, realm, "flag1"), Value::from_bool(true));
    assert_eq!(global_value(agent, realm, "flag2"), Value::from_bool(false));
    assert_eq!(global_value(agent, realm, "y"), Value::from_smi(1));
}

#[test]
fn yield_star_delegates_generator_values_and_final_completion() {
    let unit = compile_test_unit(
        207,
        r#"
            function* inner() {
                yield 1;
                return 2;
            }
            function* outer() {
                const value = yield* inner();
                return value + 1;
            }
            var iter = outer();
            var first = iter.next();
            var second = iter.next();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let _ = vm.evaluate_script(agent, realm, &unit).unwrap();
    let first = global_value(agent, realm, "first");
    let second = global_value(agent, realm, "second");
    let (first_value, first_done) = iterator_result_fields(agent, first);
    let (second_value, second_done) = iterator_result_fields(agent, second);

    assert_eq!(first_value, Value::from_smi(1));
    assert_eq!(first_done, Value::from_bool(false));
    assert_eq!(second_value, Value::from_smi(3));
    assert_eq!(second_done, Value::from_bool(true));
}

#[test]
fn yield_star_forwards_inner_iterator_result_objects() {
    let unit = compile_test_unit(
        208,
        r#"
            var results = [{ value: 1 }, { value: 8 }, { value: 34, done: true }];
            var index = 0;
            var iterator = {
                next() {
                    var result = results[index];
                    index += 1;
                    return result;
                }
            };
            var iterable = {};
            iterable[Symbol.iterator] = function() {
                return iterator;
            };
            function* g() {
                yield* iterable;
            }
            var iter = g();
            var first = iter.next();
            var second = iter.next();
            var final = iter.next();
            first.value === 1
                && first.done === undefined
                && second.value === 8
                && second.done === undefined
                && final.value === undefined
                && final.done === true;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn yield_star_reads_inner_value_only_when_delegate_is_done() {
    let unit = compile_test_unit(
        209,
        r#"
            var callCount = 0;
            var spyResult = Object.defineProperty({ done: false }, "value", {
                get() {
                    callCount += 1;
                    return 42;
                }
            });
            var iterable = {};
            iterable[Symbol.iterator] = function() {
                return {
                    next() {
                        return spyResult;
                    }
                };
            };
            var delegationComplete = false;
            function* g() {
                yield* iterable;
                delegationComplete = true;
            }
            var iter = g();
            iter.next();
            var firstCount = callCount;
            var firstComplete = delegationComplete;
            spyResult.done = true;
            iter.next();
            firstCount === 0
                && firstComplete === false
                && callCount === 1
                && delegationComplete === true;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn yield_star_missing_throw_invokes_return_without_arguments() {
    let unit = compile_test_unit(
        210,
        r#"
            var thisValue;
            var argsLength = -1;
            var poisonedReturn = {
                next() {
                    return { done: false };
                },
                return() {
                    thisValue = this;
                    argsLength = arguments.length;
                    return "non-object";
                }
            };
            var iterable = {};
            iterable[Symbol.iterator] = function() {
                return poisonedReturn;
            };
            function* g() {
                try {
                    yield* iterable;
                } catch (error) {
                    return error.constructor === TypeError;
                }
                return false;
            }
            var iter = g();
            iter.next();
            var result = iter.throw();
            result.value === true
                && result.done === true
                && thisValue === poisonedReturn
                && argsLength === 0;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn generator_methods_on_classes_lower_through_the_shared_class_path() {
    let unit = compile_test_unit(
        208,
        r#"
            class C {
                *values() {
                    yield 1;
                    return 2;
                }
            }
            var iter = new C().values();
            var first = iter.next();
            var second = iter.next();
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let _ = vm.evaluate_script(agent, realm, &unit).unwrap();
    let first = global_value(agent, realm, "first");
    let second = global_value(agent, realm, "second");
    let (first_value, first_done) = iterator_result_fields(agent, first);
    let (second_value, second_done) = iterator_result_fields(agent, second);

    assert_eq!(first_value, Value::from_smi(1));
    assert_eq!(first_done, Value::from_bool(false));
    assert_eq!(second_value, Value::from_smi(2));
    assert_eq!(second_done, Value::from_bool(true));
}

#[test]
fn module_default_export_generator_declarations_lower_under_6e1() {
    let unit = compile_test_module(
        209,
        r#"
            export default function* g() {
                yield 1;
            }
            export const value = g().next().value;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let key = ModuleKey::new("/tmp/generator-default.mjs");
    let mut vm = Vm::new();

    let _ = vm
        .evaluate_module(agent, realm, &key, "/tmp/generator-default.mjs", &unit)
        .unwrap();

    let record = agent
        .module_record(&key)
        .expect("evaluated module should stay cached on the agent");
    let module_env = record
        .environment()
        .expect("module evaluation should materialize a module environment");
    let default_slot = unit
        .local_exports()
        .iter()
        .find(|entry| entry.export_name() == lyng_js_common::WellKnownAtom::default.id())
        .expect("module should export a default generator")
        .local_slot();
    let value_slot = unit
        .local_exports()
        .iter()
        .find(|entry| unit.atom_text(entry.export_name()) == Some("value"))
        .expect("module should export the generator step value")
        .local_slot();
    let default_export = agent
        .environment_slot(module_env, default_slot)
        .expect("default export should be initialized")
        .as_object_ref()
        .expect("default export should be a function object");

    assert_eq!(
        agent.environment_slot(module_env, value_slot),
        Some(Value::from_smi(1))
    );
    assert!(agent
        .objects()
        .function_data(default_export)
        .is_some_and(|data| data.kind_flags().is_generator()));
}

#[test]
fn vm_marker_round_trips_context_and_frame_state() {
    let context = ExecutionContext::bytecode(
        RealmRef::from_raw(3).unwrap(),
        CodeRef::from_raw(11).unwrap(),
        EnvironmentRef::from_raw(4).unwrap(),
        EnvironmentRef::from_raw(4).unwrap(),
    )
    .with_this_state(ThisState::Value(Value::undefined()));
    let marker = VmMarker::new(
        BytecodeMarker::new(
            SourceId::new(1),
            BytecodeFunctionId::new(NonZeroU32::new(5).unwrap()),
            FeedbackSlotId::new(NonZeroU32::new(6).unwrap()),
        ),
        context,
        FrameRecord::new(
            CodeRef::from_raw(11).unwrap(),
            0,
            RegisterWindow::new(0, 4),
            Some(1),
            RealmRef::from_raw(3).unwrap(),
            EnvironmentRef::from_raw(4).unwrap(),
            EnvironmentRef::from_raw(4).unwrap(),
            ExecutionContextKind::Function,
        )
        .with_flags(FrameFlags::entry()),
    );

    assert_eq!(marker.bytecode().source(), SourceId::new(1));
    assert_eq!(
        marker.context().executable(),
        ExecutableId::Bytecode(CodeRef::from_raw(11).unwrap())
    );
    assert_eq!(marker.frame().registers(), RegisterWindow::new(0, 4));
    assert!(marker.frame().flags().contains(FrameFlags::entry()));
}

#[test]
fn seed_registers_uses_window_length() {
    let registers = seed_registers(RegisterWindow::new(10, 3));

    assert_eq!(registers.len(), 3);
    assert!(registers.iter().all(|value| *value == Value::undefined()));
}

#[test]
fn frame_record_carries_phase4_execution_state() {
    let frame = FrameRecord::new(
        CodeRef::from_raw(2).unwrap(),
        4,
        RegisterWindow::new(8, 2),
        Some(1),
        RealmRef::from_raw(1).unwrap(),
        EnvironmentRef::from_raw(3).unwrap(),
        EnvironmentRef::from_raw(4).unwrap(),
        ExecutionContextKind::Function,
    )
    .with_this_value(Value::from_smi(9))
    .with_handler_cursor(2)
    .with_flags(FrameFlags::entry().with_flag(FrameFlags::suspendable(), true));

    assert_eq!(size_of::<FrameFlags>(), size_of::<u8>());
    assert_eq!(frame.instruction_offset(), 4);
    assert_eq!(frame.realm(), RealmRef::from_raw(1).unwrap());
    assert_eq!(frame.lexical_env(), EnvironmentRef::from_raw(3).unwrap());
    assert_eq!(frame.variable_env(), EnvironmentRef::from_raw(4).unwrap());
    assert_eq!(frame.this_value(), Value::from_smi(9));
    assert_eq!(frame.handler_cursor(), 2);
    assert!(frame.flags().contains(FrameFlags::entry()));
    assert!(frame.flags().contains(FrameFlags::suspendable()));
}

#[test]
fn vm_installs_script_units_into_code_storage_and_executes_basic_dispatch() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder.set_name(Some(AtomId::from_raw(17)));
    builder
        .alloc_registers(2)
        .expect("test bytecode registers should allocate");
    let constant = builder
        .add_constant(ConstantValue::Smi(41))
        .expect("test bytecode constant should build");
    builder
        .emit_abx(Opcode::LoadConst, 0, constant)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::Move, 1, 0, 0)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 1)
        .expect("test bytecode should build");
    builder
        .add_feedback_site(
            0,
            FeedbackSiteKind::Arithmetic,
            lyng_js_bytecode::FeedbackSiteMetadata::None,
        )
        .expect("test bytecode feedback site should build");
    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(9), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let code_record = agent
        .heap()
        .view()
        .code(installed.code())
        .expect("installed code record should exist");
    let code_slots = code_record
        .constants()
        .and_then(|slots| agent.heap().view().code_slots(slots))
        .expect("constant slots should exist");

    assert_eq!(code_record.realm(), Some(realm.id()));
    assert_eq!(code_slots, &[Value::from_smi(41)]);

    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(41));
    assert!(vm.frames().is_empty());
    assert!(vm.register_stack().is_empty());
    assert!(agent.current_execution_context().is_none());
}

#[test]
fn vm_installs_callable_index_accessors_from_object_literals() {
    let unit = compile_test_unit(
        41,
        r#"
        var object = {
            get [1]() { return 10; },
            set [1](_) {}
        };
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let _ = vm.evaluate_script(agent, realm, &unit).unwrap();

    let object_atom = unit_atom(&unit, "object");
    let runtime_atom = unit_runtime_atom(agent, &unit, object_atom);
    let global_object = realm.global_object();
    let object_value =
        ordinary_get(agent, global_object, PropertyKey::from_atom(runtime_atom)).unwrap();
    let object = object_value
        .as_object_ref()
        .expect("global object binding should store an object literal");
    let descriptor = agent
        .objects()
        .get_own_property(agent.heap().view(), object, PropertyKey::Index(1))
        .unwrap()
        .expect("index accessor should be installed");
    let getter = descriptor
        .getter()
        .expect("getter should be present on the index descriptor");
    let setter = descriptor
        .setter()
        .expect("setter should be present on the index descriptor");
    let getter_object = getter
        .as_object_ref()
        .expect("getter should be represented as an object reference");
    let setter_object = setter
        .as_object_ref()
        .expect("setter should be represented as an object reference");

    assert!(
        agent.objects().function_data(getter_object).is_some(),
        "getter slot should contain a callable function object"
    );
    assert!(
        agent.objects().function_data(setter_object).is_some(),
        "setter slot should contain a callable function object"
    );
    assert!(matches!(
        agent
            .objects()
            .function_data(getter_object)
            .and_then(|data| data.entry()),
        Some(FunctionEntryIdentity::Bytecode(_))
    ));
    assert!(matches!(
        agent
            .objects()
            .function_data(setter_object)
            .and_then(|data| data.entry()),
        Some(FunctionEntryIdentity::Bytecode(_))
    ));

    let Some(FunctionEntryIdentity::Bytecode(getter_code)) = agent
        .objects()
        .function_data(getter_object)
        .and_then(|data| data.entry())
    else {
        panic!("getter should remain backed by installed bytecode");
    };
    let getter_function = vm
        .installed_function(getter_code)
        .expect("getter bytecode should stay installed");
    let getter_environment = agent
        .objects()
        .function_data(getter_object)
        .and_then(|data| data.environment())
        .expect("getter closure should preserve its outer environment");
    let getter_result = vm
        .evaluate_installed(
            agent,
            InstalledCode::new(getter_code, getter_function.id()),
            getter_environment,
            getter_environment,
        )
        .expect("getter bytecode should execute as a standalone entry");
    assert_eq!(getter_result, Value::from_smi(10));
}

#[test]
fn vm_bootstraps_phase5_default_global_bindings_before_script_entry() {
    let unit = compile_test_unit(
        52,
        r#"
        (globalThis === this ? 1 : 0)
            + (Infinity === 1 / 0 ? 2 : 0)
            + (NaN !== NaN ? 4 : 0)
            + (undefined === undefined ? 8 : 0);
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let global_this = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            realm.global_object(),
            PropertyKey::from_atom(agent.bootstrap_atoms().global_this()),
        )
        .unwrap()
        .expect("globalThis should be installed during entry bootstrap");

    assert_eq!(result, Value::from_smi(15));
    assert_eq!(
        agent
            .realm(realm.id())
            .expect("default realm should remain queryable")
            .bootstrap_state(),
        RealmBootstrapState::new().with_spec_ready(true)
    );
    assert_eq!(
        global_this.value(),
        Some(Value::from_object_ref(realm.global_object()))
    );
}

#[test]
fn bootstrap_installs_phase6_wrapper_prototypes_for_to_object() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let string = agent.alloc_runtime_string("abc", None, AllocationLifetime::Default);
    let bigint = agent.heap_mut().mutator().alloc_bigint(
        BigIntSign::NonNegative,
        &[23],
        AllocationLifetime::Default,
    );

    let _ = vm
        .bootstrap_realm(agent, realm.id(), lyng_js_builtins::BootstrapMode::SpecOnly)
        .expect("bootstrap should succeed");

    assert!(lyng_js_ops::object::to_object(agent, realm.id(), Value::from_smi(7)).is_ok());
    assert!(
        lyng_js_ops::object::to_object(agent, realm.id(), Value::from_string_ref(string)).is_ok()
    );
    assert!(
        lyng_js_ops::object::to_object(agent, realm.id(), Value::from_bigint_ref(bigint)).is_ok()
    );
}

struct WrapperPrimitiveProbe<'a> {
    agent: &'a mut lyng_js_env::Agent,
    called: bool,
}

impl lyng_js_ops::object::ToPrimitiveContext for WrapperPrimitiveProbe<'_> {
    type Error = lyng_js_types::AbruptCompletion;

    fn agent(&mut self) -> &mut lyng_js_env::Agent {
        self.agent
    }

    fn abrupt(&mut self, completion: lyng_js_types::AbruptCompletion) -> Self::Error {
        completion
    }

    fn type_error(&mut self) -> Self::Error {
        lyng_js_ops::errors::throw_type_error(self.agent)
    }

    fn get_property_value(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> Result<Value, Self::Error> {
        ordinary_get(self.agent, object, key)
    }

    fn require_callable_object(&mut self, value: Value) -> Result<ObjectRef, Self::Error> {
        let Some(object) = value.as_object_ref() else {
            return Err(lyng_js_ops::errors::throw_type_error(self.agent));
        };
        if self.agent.objects().function_data(object).is_some() {
            Ok(object)
        } else {
            Err(lyng_js_ops::errors::throw_type_error(self.agent))
        }
    }

    fn call_to_completion(
        &mut self,
        _callee_object: ObjectRef,
        this_value: Value,
        _arguments: &[Value],
    ) -> Result<Value, Self::Error> {
        self.called = true;
        let Some(object) = this_value.as_object_ref() else {
            return Err(lyng_js_ops::errors::throw_type_error(self.agent));
        };
        self.agent
            .objects()
            .primitive_wrapper_value(self.agent.heap().view(), object)
            .ok_or_else(|| lyng_js_ops::errors::throw_type_error(self.agent))
    }
}

#[test]
fn bootstrap_string_wrapper_uses_bootstrapped_string_prototype_methods() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let string = agent.alloc_runtime_string("abc", None, AllocationLifetime::Default);

    let _ = vm
        .bootstrap_realm(agent, realm.id(), lyng_js_builtins::BootstrapMode::SpecOnly)
        .expect("bootstrap should succeed");

    let string_wrapper =
        lyng_js_ops::object::to_object(agent, realm.id(), Value::from_string_ref(string)).unwrap();
    let mut probe = WrapperPrimitiveProbe {
        agent,
        called: false,
    };

    assert_eq!(
        lyng_js_ops::object::to_primitive(
            &mut probe,
            Value::from_object_ref(string_wrapper),
            lyng_js_ops::object::ToPrimitiveHint::Number,
        ),
        Ok(Value::from_string_ref(string))
    );
    assert!(probe.called);
}

#[test]
fn global_script_instantiation_precreates_non_configurable_var_bindings() {
    let unit = compile_test_unit(53, "var x = 1;");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let _ = vm
        .bootstrap_realm(agent, realm.id(), lyng_js_builtins::BootstrapMode::SpecOnly)
        .expect("bootstrap should succeed");
    let _ = vm.install_script(agent, realm.id(), &unit).unwrap();
    vm.instantiate_global_script(agent, realm, unit.instantiation_plan())
        .unwrap();

    let x_atom = unit_runtime_atom(agent, &unit, unit_atom(&unit, "x"));
    let descriptor = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            realm.global_object(),
            PropertyKey::from_atom(x_atom),
        )
        .unwrap()
        .expect("instantiation should precreate a global property");

    assert_eq!(descriptor.value(), Some(Value::undefined()));
    assert_eq!(descriptor.writable(), Some(true));
    assert_eq!(descriptor.enumerable(), Some(true));
    assert_eq!(descriptor.configurable(), Some(false));
}

#[test]
fn global_script_instantiation_uses_dictionary_storage_for_bulk_var_bindings() {
    let mut source = String::new();
    for index in 0..96 {
        source.push_str(&format!("var binding_{index};\n"));
    }
    let unit = compile_test_unit(5_301, &source);
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let _ = vm
        .bootstrap_realm(agent, realm.id(), lyng_js_builtins::BootstrapMode::SpecOnly)
        .expect("bootstrap should succeed");
    let _ = vm.install_script(agent, realm.id(), &unit).unwrap();
    vm.instantiate_global_script(agent, realm, unit.instantiation_plan())
        .unwrap();

    assert_eq!(
        agent
            .objects()
            .named_property_storage_mode(realm.global_object()),
        Some(NamedPropertyStorageMode::Dictionary)
    );
    let last_atom = agent.atoms_mut().intern_collectible("binding_95");
    assert!(agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            realm.global_object(),
            PropertyKey::from_atom(last_atom),
        )
        .unwrap()
        .is_some());
}

#[test]
fn vm_executes_wide_register_and_constant_operands() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(7).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(300)
        .expect("test bytecode registers should allocate");
    let mut last_constant = 0;
    for index in 0..70_000u32 {
        last_constant = builder
            .add_constant(ConstantValue::Smi(index as i32))
            .expect("test bytecode constant should build");
    }
    builder
        .emit_abx(Opcode::LoadConst, 299u16, last_constant)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 299)
        .expect("test bytecode should build");

    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(17), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(69_999));
}

#[test]
fn vm_executes_wide_conditional_jumps() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(8).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(300)
        .expect("test bytecode registers should allocate");
    builder
        .emit_abx(Opcode::LoadTrue, 299u16, 0)
        .expect("test bytecode should build");
    let jump = builder
        .emit_cond_jump_placeholder(Opcode::JumpIfTrue, 299u16)
        .expect("test bytecode should build");
    builder
        .emit_abx(Opcode::LoadSmi, 0u16, 1u16)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 0)
        .expect("test bytecode should build");
    for _ in 0..40_000 {
        builder
            .emit_ax(Opcode::Nop, 0)
            .expect("test bytecode should build");
    }
    let target = builder
        .current_offset()
        .expect("test bytecode offset should build");
    builder
        .emit_abx(Opcode::LoadSmi, 0u16, 7u16)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 0)
        .expect("test bytecode should build");
    builder
        .patch_jump_to(jump, target)
        .expect("test bytecode jump should patch");

    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(18), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn global_load_matches_runtime_atom_text_when_ids_differ() {
    let unit = compile_test_unit(19, "runtimeOnly;");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let _ = agent.atoms_mut().intern_collectible("padding");
    let runtime_name = agent.atoms_mut().intern_collectible("runtimeOnly");
    install_global_value(agent, realm, runtime_name, Value::from_smi(13));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(13));
}

#[test]
fn typeof_name_resolution_matches_runtime_atom_text_when_ids_differ() {
    let unit = compile_test_unit(20, "typeof runtimeOnly;");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let _ = agent.atoms_mut().intern_collectible("padding");
    let runtime_name = agent.atoms_mut().intern_collectible("runtimeOnly");
    install_global_value(agent, realm, runtime_name, Value::from_smi(13));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    let string = result
        .as_string_ref()
        .expect("typeof should return a string");
    let view = agent
        .heap()
        .view()
        .string_view(string)
        .expect("string should exist in the heap");

    assert_eq!(decode_string(view), "number");
}

#[test]
fn direct_named_property_definitions_preserve_all_named_slots() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });

    for (name, value) in [
        ("1.2", 1),
        ("1e+55", 2),
        ("0.000001", 3),
        ("Infinity", 5),
        ("-Infinity", 6),
        ("NaN", 7),
    ] {
        let atom = agent.atoms_mut().intern_collectible(name);
        assert!(ordinary_create_data_property(
            agent,
            object,
            PropertyKey::from_atom(atom),
            Value::from_smi(value),
            AllocationLifetime::Default,
        )
        .unwrap());
    }

    for (name, value) in [
        ("1.2", 1),
        ("1e+55", 2),
        ("0.000001", 3),
        ("Infinity", 5),
        ("-Infinity", 6),
        ("NaN", 7),
    ] {
        let atom = agent.atoms_mut().intern_collectible(name);
        assert_eq!(
            ordinary_get(agent, object, PropertyKey::from_atom(atom)).unwrap(),
            Value::from_smi(value)
        );
    }
}

#[test]
fn vm_tracks_child_parent_links_and_unconditional_jumps() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let child = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(2).unwrap(),
        Some(AtomId::from_raw(33)),
        ArgumentsMode::None,
    )
    .with_register_counts(1, 0)
    .with_instructions(vec![Instruction::ax(Opcode::ReturnUndefined, 0)]);

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        BytecodeFunctionKind::Function,
    );
    builder
        .alloc_registers(1)
        .expect("test bytecode registers should allocate");
    let jump = builder
        .emit_jump_placeholder(Opcode::Jump)
        .expect("test bytecode should build");
    builder
        .emit_abx(Opcode::LoadSmi, 0, 99)
        .expect("test bytecode should build");
    let ret = builder
        .emit_ax(Opcode::ReturnUndefined, 0)
        .expect("test bytecode should build");
    builder
        .patch_jump_to(jump, ret)
        .expect("test bytecode jump should patch");
    let parent = builder
        .finish()
        .expect("test bytecode should build")
        .with_child_functions(vec![child.id()]);
    let unit = CompiledFunctionUnit::new(SourceId::new(11), parent.id(), vec![parent, child]);

    let mut vm = Vm::new();
    let installed = vm.install_function(agent, realm.id(), &unit).unwrap();
    let child_code = vm
        .installed_child_code(installed.code(), 0)
        .expect("installed child code should exist");

    assert_eq!(
        agent.heap().view().code(child_code).unwrap().parent(),
        Some(installed.code())
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::undefined()
    );
}

#[test]
fn load_const_supports_atom_backed_string_constants() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        None,
        ArgumentsMode::None,
    )
    .with_kind(BytecodeFunctionKind::Script)
    .with_register_counts(1, 0)
    .with_constants(vec![ConstantValue::Atom(AtomId::from_raw(9))])
    .with_instructions(vec![
        Instruction::abx(Opcode::LoadConst, 0, 0),
        Instruction::ax(Opcode::Return, 0),
    ]);
    let unit =
        CompiledScriptUnit::new(SourceId::new(13), function.id(), vec![function]).with_atoms(vec![
            (AtomId::from_raw(9), CompiledAtom::from("loaded-atom")),
        ]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    let string = result
        .as_string_ref()
        .expect("atom constant should load a string");
    let view = agent
        .heap()
        .view()
        .string_view(string)
        .expect("loaded string should exist in the heap");
    let cached_atom = view.cached_atom();
    let expected_atom = agent.atoms_mut().intern_collectible("loaded-atom");

    assert_eq!(cached_atom, Some(expected_atom));
}

#[test]
fn load_const_supports_utf16_only_atom_backed_string_constants() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        None,
        ArgumentsMode::None,
    )
    .with_kind(BytecodeFunctionKind::Script)
    .with_register_counts(1, 0)
    .with_constants(vec![ConstantValue::Atom(AtomId::from_raw(9))])
    .with_instructions(vec![
        Instruction::abx(Opcode::LoadConst, 0, 0),
        Instruction::ax(Opcode::Return, 0),
    ]);
    let unit =
        CompiledScriptUnit::new(SourceId::new(14), function.id(), vec![function]).with_atoms(vec![
            (AtomId::from_raw(9), CompiledAtom::from(vec![0xD83D])),
        ]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    let string = result
        .as_string_ref()
        .expect("atom constant should load a string");
    let view = agent
        .heap()
        .view()
        .string_view(string)
        .expect("loaded string should exist in the heap");
    let bytes = view
        .utf16_bytes()
        .expect("UTF-16-only atom constant should materialize as a UTF-16 string");
    let units = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    let cached_atom = view.cached_atom();
    let expected_atom = agent.atoms_mut().intern_collectible_utf16(&[0xD83D]);

    assert_eq!(units, vec![0xD83D]);
    assert_eq!(cached_atom, Some(expected_atom));
}

#[test]
fn load_const_still_rejects_builtin_constants_without_runtime_support() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        None,
        ArgumentsMode::None,
    )
    .with_kind(BytecodeFunctionKind::Script)
    .with_register_counts(1, 0)
    .with_constants(vec![ConstantValue::Builtin(
        lyng_js_types::BuiltinFunctionId::from_raw(9).unwrap(),
    )])
    .with_instructions(vec![
        Instruction::abx(Opcode::LoadConst, 0, 0),
        Instruction::ax(Opcode::Return, 0),
    ]);
    let unit = CompiledScriptUnit::new(SourceId::new(14), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env()),
        Err(VmError::UnsupportedConstant {
            code: installed.code(),
            index: 0,
            constant: ConstantValue::Builtin(
                lyng_js_types::BuiltinFunctionId::from_raw(9).unwrap()
            ),
        })
    );
}

#[test]
fn load_const_supports_reserved_internal_builtin_constants() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        None,
        ArgumentsMode::None,
    )
    .with_kind(BytecodeFunctionKind::Script)
    .with_register_counts(1, 0)
    .with_constants(vec![ConstantValue::Builtin(
        internal_function_call_builtin(),
    )])
    .with_instructions(vec![
        Instruction::abx(Opcode::LoadConst, 0, 0),
        Instruction::ax(Opcode::Return, 0),
    ]);
    let unit = CompiledScriptUnit::new(SourceId::new(141), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    let builtin = result
        .as_object_ref()
        .expect("reserved internal builtin constants should resolve to builtin objects");
    let function_data = agent
        .objects()
        .function_data(builtin)
        .expect("builtin constant should resolve to a callable object");

    assert_eq!(
        function_data.entry(),
        Some(FunctionEntryIdentity::Native(NativeFunctionId::builtin(
            internal_function_call_builtin()
        )))
    );
}

#[test]
fn load_const_supports_phase5_public_builtin_constants() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        None,
        ArgumentsMode::None,
    )
    .with_kind(BytecodeFunctionKind::Script)
    .with_register_counts(1, 0)
    .with_constants(vec![ConstantValue::Builtin(symbol_builtin())])
    .with_instructions(vec![
        Instruction::abx(Opcode::LoadConst, 0, 0),
        Instruction::ax(Opcode::Return, 0),
    ]);
    let unit = CompiledScriptUnit::new(SourceId::new(142), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    let builtin = result
        .as_object_ref()
        .expect("public builtin constants should resolve to builtin objects");
    let function_data = agent
        .objects()
        .function_data(builtin)
        .expect("builtin constant should resolve to a callable object");

    assert_eq!(
        function_data.entry(),
        Some(FunctionEntryIdentity::Native(NativeFunctionId::builtin(
            symbol_builtin(),
        )))
    );
}

#[test]
fn load_const_supports_phase5_function_builtin_constants() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        None,
        ArgumentsMode::None,
    )
    .with_kind(BytecodeFunctionKind::Script)
    .with_register_counts(1, 0)
    .with_constants(vec![ConstantValue::Builtin(function_builtin())])
    .with_instructions(vec![
        Instruction::abx(Opcode::LoadConst, 0, 0),
        Instruction::ax(Opcode::Return, 0),
    ]);
    let unit = CompiledScriptUnit::new(SourceId::new(143), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    let builtin = result
        .as_object_ref()
        .expect("Function builtin constants should resolve to callable objects");
    let function_data = agent
        .objects()
        .function_data(builtin)
        .expect("Function builtin constant should resolve to a function object");

    assert_eq!(
        function_data.entry(),
        Some(FunctionEntryIdentity::Native(NativeFunctionId::builtin(
            function_builtin(),
        )))
    );
}

#[test]
fn symbol_global_dispatches_through_the_shared_builtins_bridge() {
    let unit = compile_test_unit(
        144,
        r#"
            Symbol("dispatch-bridge");
        "#,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    let symbol = result
        .as_symbol_ref()
        .expect("Symbol() should keep returning runtime symbols through the dispatch bridge");
    let description = agent
        .heap()
        .view()
        .symbol_view(symbol)
        .expect("symbol result should be live")
        .description()
        .expect("symbol description should be stored");

    assert_eq!(
        decode_string(agent.heap().view().string_view(description).unwrap()),
        "dispatch-bridge"
    );
}

#[test]
fn symbol_constructor_exposes_disposal_well_known_symbols() {
    let unit = compile_test_unit(
        145,
        r#"
            let dispose = Object.getOwnPropertyDescriptor(Symbol, "dispose");
            let asyncDispose = Object.getOwnPropertyDescriptor(Symbol, "asyncDispose");
            (typeof Symbol.dispose === "symbol" ? 1 : 0)
                + (typeof Symbol.asyncDispose === "symbol" ? 2 : 0)
                + (Symbol.dispose !== Symbol.asyncDispose ? 4 : 0)
                + (dispose && !dispose.writable && !dispose.enumerable && !dispose.configurable ? 8 : 0)
                + (asyncDispose && !asyncDispose.writable && !asyncDispose.enumerable && !asyncDispose.configurable ? 16 : 0)
                + (String(Symbol.dispose) === "Symbol(Symbol.dispose)" ? 32 : 0)
                + (String(Symbol.asyncDispose) === "Symbol(Symbol.asyncDispose)" ? 64 : 0);
        "#,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn function_builtins_dispatch_through_the_shared_builtins_bridge() {
    let unit = compile_test_unit(
        146,
        r#"
            Function("return 9;").call(undefined);
        "#,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(9));
}

#[test]
fn for_in_state_is_cleared_when_return_exits_loop_body() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = AtomId::from_raw(71);
    let value_name = AtomId::from_raw(72);

    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    assert!(ordinary_create_data_property(
        agent,
        object,
        PropertyKey::from_atom(value_name),
        Value::from_smi(1),
        AllocationLifetime::Default,
    )
    .unwrap());
    assert!(ordinary_create_data_property(
        agent,
        realm.global_object(),
        PropertyKey::from_atom(source_name),
        Value::from_object_ref(object),
        AllocationLifetime::Default,
    )
    .unwrap());

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(4)
        .expect("test bytecode registers should allocate");
    let object_name = builder
        .add_constant(ConstantValue::Atom(source_name))
        .expect("test bytecode constant should build");
    builder
        .emit_abx(Opcode::LoadGlobal, 0, object_name)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::CreateForIn, 1, 0, 0)
        .expect("test bytecode should build");
    builder
        .emit_abc(Opcode::AdvanceForIn, 1, 2, 3)
        .expect("test bytecode should build");
    let done = builder
        .emit_cond_jump_placeholder(Opcode::JumpIfTrue, 3)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::ReturnUndefined, 0)
        .expect("test bytecode should build");
    let close_offset = builder
        .current_offset()
        .expect("test bytecode offset should build");
    builder
        .patch_jump_to(done, close_offset)
        .expect("test bytecode jump should patch");
    builder
        .emit_abx(Opcode::CloseForIn, 1, 0)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::ReturnUndefined, 0)
        .expect("test bytecode should build");

    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(15), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::undefined()
    );
    assert_eq!(vm.active_for_in_enumerators(), 0);
}

#[test]
fn throw_transfers_control_to_matching_catch_handler() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(1).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(2)
        .expect("test bytecode registers should allocate");
    builder
        .emit_abx(Opcode::LoadSmi, 0, 13)
        .expect("test bytecode should build");
    let protected_end = builder
        .current_offset()
        .expect("test bytecode offset should build")
        + 1;
    builder
        .emit_ax(Opcode::Throw, 0)
        .expect("test bytecode should build");
    let catch_entry = builder
        .current_offset()
        .expect("test bytecode offset should build");
    builder
        .emit_ax(Opcode::EnterHandler, 0)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::LoadException, 1)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::LeaveHandler, 0)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 1)
        .expect("test bytecode should build");
    builder
        .add_exception_handler(ExceptionHandler::new(
            0,
            protected_end,
            catch_entry,
            ExceptionHandlerKind::Catch,
            builder.header().register_count(),
            Some(1),
        ))
        .expect("test bytecode handler should build");

    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(16), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(13));
}

#[test]
fn feedback_vectors_allocate_lazily_without_changing_entry_script_result() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        SourceId::new(21),
        r#"
            function add(left, right) {
                return left + right;
            }
            add(1, 2);
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit.function(unit.entry()).unwrap();
    let call_slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::Call)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain one call site");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();

    let first = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    assert_eq!(first, Value::from_smi(3));
    assert_eq!(vm.feedback_warmup_counter(installed.code()), Some(1));
    assert!(!vm.has_feedback_vector(installed.code()));

    let second = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    assert_eq!(second, Value::from_smi(3));
    assert_eq!(vm.feedback_warmup_counter(installed.code()), Some(2));
    assert!(vm.has_feedback_vector(installed.code()));
    assert_eq!(
        vm.feedback_execution_count(installed.code(), call_slot),
        Some(1)
    );
}

#[test]
fn closures_sharing_one_code_ref_share_feedback_warmup_and_vector_state() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        SourceId::new(22),
        r#"
            function makeAdder(base) {
                return function(delta) {
                    return base + delta;
                };
            }
            let first = makeAdder(1);
            let second = makeAdder(2);
            first(3);
            second(4);
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let outer = unit
        .functions()
        .iter()
        .find(|function| function.name().is_some())
        .expect("named outer function should be lowered");
    let inner = unit
        .functions()
        .iter()
        .find(|function| function.name().is_none() && !function.captures().is_empty())
        .expect("capturing inner closure should be lowered");
    let arithmetic_slot = inner
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::Arithmetic)
        .map(|descriptor| descriptor.slot())
        .expect("inner closure should contain one arithmetic site");
    let outer_child_index = unit
        .function(unit.entry())
        .expect("entry function should exist")
        .child_functions()
        .iter()
        .position(|child| *child == outer.id())
        .and_then(|index| u32::try_from(index).ok())
        .expect("script should install the outer function as a direct child");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let outer_code = vm
        .installed_child_code(installed.code(), outer_child_index)
        .expect("outer function should have one installed code record");
    let inner_code = vm
        .installed_child_code(outer_code, 0)
        .expect("inner closure template should install under the outer function");

    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(6));
    assert_eq!(vm.feedback_warmup_counter(inner_code), Some(2));
    assert!(vm.has_feedback_vector(inner_code));
    assert_eq!(
        vm.feedback_execution_count(inner_code, arithmetic_slot),
        Some(1)
    );
}

#[test]
fn internal_bytecode_callbacks_share_feedback_state_with_the_parent_vm() {
    let unit = compile_test_unit(
        23,
        r#"
            function callback() {
                return 1 + 2;
            }
            "ab".replace("b", callback);
            0;
        "#,
    );
    let callback_name = unit_atom(&unit, "callback");
    let callback = unit
        .functions()
        .iter()
        .find(|function| function.name() == Some(callback_name))
        .expect("callback function should be lowered");
    let arithmetic_slot = callback
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::Arithmetic)
        .map(|descriptor| descriptor.slot())
        .expect("callback should contain one arithmetic site");
    let callback_child_index = unit
        .function(unit.entry())
        .expect("entry function should exist")
        .child_functions()
        .iter()
        .position(|child| *child == callback.id())
        .and_then(|index| u32::try_from(index).ok())
        .expect("script should install the callback as a direct child");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let callback_code = vm
        .installed_child_code(installed.code(), callback_child_index)
        .expect("callback function should have installed code");

    let first = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    assert_eq!(first, Value::from_smi(0));
    assert_eq!(vm.feedback_warmup_counter(callback_code), Some(1));
    assert!(!vm.has_feedback_vector(callback_code));

    let second = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();
    assert_eq!(second, Value::from_smi(0));
    assert_eq!(vm.feedback_warmup_counter(callback_code), Some(2));
    assert!(vm.has_feedback_vector(callback_code));
    assert_eq!(
        vm.feedback_execution_count(callback_code, arithmetic_slot),
        Some(1)
    );
}

#[test]
fn load_env_slot_throws_for_uninitialized_lexicals() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let layout = agent.alloc_environment_layout(EnvironmentLayout::new(
        EnvironmentLayoutKind::Declarative,
        [EnvironmentBindingLayout::new(
            Some(AtomId::from_raw(91)),
            EnvironmentSlotFlags::mutable_lexical(),
        )],
        true,
    ));
    let lexical_env = agent
        .alloc_declarative_environment(None, layout, AllocationLifetime::Default)
        .expect("declarative environment should allocate");

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(30).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(1)
        .expect("test bytecode registers should allocate");
    builder
        .emit_abx(Opcode::LoadEnvSlot, 0, 0)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 0)
        .expect("test bytecode should build");
    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(24), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm.evaluate_installed(agent, installed, lexical_env, lexical_env);

    assert!(matches!(result, Err(VmError::Abrupt(_))));
    assert_eq!(
        agent.environment_slot(lexical_env, 0),
        Some(Value::uninitialized_lexical())
    );
}

#[test]
fn store_env_slot_rejects_reassigning_initialized_const_bindings() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let layout = agent.alloc_environment_layout(EnvironmentLayout::new(
        EnvironmentLayoutKind::Declarative,
        [EnvironmentBindingLayout::new(
            Some(AtomId::from_raw(92)),
            EnvironmentSlotFlags::immutable_lexical(),
        )],
        true,
    ));
    let lexical_env = agent
        .alloc_declarative_environment(None, layout, AllocationLifetime::Default)
        .expect("declarative environment should allocate");
    assert!(agent.set_environment_slot(lexical_env, 0, Value::from_smi(1)));

    let mut builder = BytecodeBuilder::new(
        BytecodeFunctionId::from_raw(31).unwrap(),
        BytecodeFunctionKind::Script,
    );
    builder
        .alloc_registers(1)
        .expect("test bytecode registers should allocate");
    let constant = builder
        .add_constant(ConstantValue::Smi(2))
        .expect("test bytecode constant should build");
    builder
        .emit_abx(Opcode::LoadConst, 0, constant)
        .expect("test bytecode should build");
    builder
        .emit_abx(Opcode::StoreEnvSlot, 0, 0)
        .expect("test bytecode should build");
    builder
        .emit_ax(Opcode::Return, 0)
        .expect("test bytecode should build");
    let function = builder.finish().expect("test bytecode should build");
    let unit = CompiledScriptUnit::new(SourceId::new(25), function.id(), vec![function]);

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm.evaluate_installed(agent, installed, lexical_env, lexical_env);

    assert!(matches!(result, Err(VmError::Abrupt(_))));
    assert_eq!(
        agent.environment_slot(lexical_env, 0),
        Some(Value::from_smi(1))
    );
}

#[test]
fn named_property_load_ic_becomes_monomorphic_for_one_shape() {
    let unit = compile_test_unit(30, "source.value;");
    let entry = unit.function(unit.entry()).unwrap();
    let slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::NamedPropertyLoad)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a named-load site");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let value_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "value"));
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    assert!(ordinary_create_data_property(
        agent,
        object,
        PropertyKey::from_atom(value_name),
        Value::from_smi(7),
        AllocationLifetime::Default,
    )
    .unwrap());
    install_global_value(agent, realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(7)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(7)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_js_objects::NamedPropertyCachePath::OwnData)
        ))
    );
}

#[test]
fn named_property_load_ic_grows_polymorphic_and_then_megamorphic() {
    let unit = compile_test_unit(31, "source.value;");
    let entry = unit.function(unit.entry()).unwrap();
    let slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::NamedPropertyLoad)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a named-load site");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let value_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "value"));

    let mut sources = Vec::new();
    for index in 0..6 {
        let object = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                AllocationLifetime::Default,
            )
        });
        for extra in 0..index {
            assert!(ordinary_create_data_property(
                agent,
                object,
                PropertyKey::from_atom(AtomId::from_raw(20_000 + extra)),
                Value::from_smi(extra as i32),
                AllocationLifetime::Default,
            )
            .unwrap());
        }
        assert!(ordinary_create_data_property(
            agent,
            object,
            PropertyKey::from_atom(value_name),
            Value::from_smi(index as i32),
            AllocationLifetime::Default,
        )
        .unwrap());
        sources.push(object);
    }

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    for (index, object) in sources.into_iter().enumerate() {
        install_global_value(agent, realm, source_name, Value::from_object_ref(object));
        assert_eq!(
            vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
                .unwrap(),
            Value::from_smi(index as i32)
        );
    }

    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some(("Megamorphic", 0, None))
    );
}

#[test]
fn named_property_store_ic_caches_own_data_paths() {
    let unit = compile_test_unit(32, "source.value = 9; source.value;");
    let entry = unit.function(unit.entry()).unwrap();
    let slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::NamedPropertyStore)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a named-store site");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let value_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "value"));
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    assert!(ordinary_create_data_property(
        agent,
        object,
        PropertyKey::from_atom(value_name),
        Value::from_smi(1),
        AllocationLifetime::Default,
    )
    .unwrap());
    install_global_value(agent, realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(9)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(9)
    );
    assert_eq!(
        vm.named_property_cache_snapshot(installed.code(), slot),
        Some((
            "Monomorphic",
            1,
            Some(lyng_js_objects::NamedPropertyCachePath::OwnData)
        ))
    );
}

#[test]
fn keyed_named_atom_ic_becomes_monomorphic() {
    let unit = compile_test_unit(33, "source[\"value\"];");
    let entry = unit.function(unit.entry()).unwrap();
    let slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::KeyedPropertyAccess)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a keyed-access site");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let value_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "value"));
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    assert!(ordinary_create_data_property(
        agent,
        object,
        PropertyKey::from_atom(value_name),
        Value::from_smi(4),
        AllocationLifetime::Default,
    )
    .unwrap());
    install_global_value(agent, realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(4)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(4)
    );
    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Monomorphic", Some("NamedAtom"), 1))
    );
}

#[test]
fn keyed_dense_index_sites_fall_back_to_megamorphic_classification() {
    let unit = compile_test_unit(34, "let index = 0; source[index];");
    let entry = unit.function(unit.entry()).unwrap();
    let slot = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::KeyedPropertyAccess)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a keyed-access site");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let source_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "source"));
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_element_capacity(1),
            AllocationLifetime::Default,
        );
        assert!(objects.set_element(
            &mut mutator,
            object,
            0,
            Value::from_smi(12),
            AllocationLifetime::Default,
        ));
        object
    });
    install_global_value(agent, realm, source_name, Value::from_object_ref(object));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(12)
    );
    assert_eq!(
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap(),
        Value::from_smi(12)
    );
    assert_eq!(
        vm.keyed_property_cache_snapshot(installed.code(), slot),
        Some(("Megamorphic", Some("DenseIndex"), 0))
    );
}

#[test]
fn vm_addresses_metadata_by_code_and_instruction_offset() {
    let unit = compile_test_unit(
        37,
        r#"
        let make = function(value) { return value; };
        let count = 0;
        while (count < 1) {
            count = count + 1;
        }
        try {
            make({ value: count });
        } catch (err) {
            err;
        }
        "#,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let function = vm
        .installed_function(installed.code())
        .expect("installed script should expose its template");
    let allocation = function
        .safepoints()
        .iter()
        .find(|descriptor| descriptor.kind() == SafepointKind::Allocation)
        .copied()
        .expect("allocation site should install metadata");
    let loop_backedge = function
        .safepoints()
        .iter()
        .find(|descriptor| descriptor.kind() == SafepointKind::LoopBackedge)
        .copied()
        .expect("loop backedge should install metadata");
    let exception = function
        .safepoints()
        .iter()
        .find(|descriptor| descriptor.kind() == SafepointKind::ExceptionEdge)
        .copied()
        .expect("exception edge should install metadata");

    let allocation_source = vm
        .source_map_entry(installed.code(), allocation.instruction_offset())
        .expect("source map should be addressable by code and offset");
    let loop_runtime = vm
        .safepoint_at(installed.code(), loop_backedge.instruction_offset())
        .expect("loop safepoint should be addressable by code and offset");
    let exception_runtime = vm
        .safepoint_by_id(installed.code(), exception.id())
        .expect("exception safepoint should be addressable by code and id");
    let exception_snapshot = vm
        .deopt_snapshot(installed.code(), exception.id())
        .expect("deopt snapshot should be addressable by code and safepoint id");

    assert_eq!(
        allocation_source.instruction_offset(),
        allocation.instruction_offset()
    );
    assert_eq!(loop_runtime.kind(), SafepointKind::LoopBackedge);
    assert!(exception_runtime.captures_exception_state());
    assert!(exception_snapshot
        .values()
        .contains(&DeoptValueSource::FrameValue(
            DeoptFrameValue::ExceptionValue,
        )));
}

#[test]
fn tail_calls_reuse_frame_depth_for_recursive_bytecode_calls() {
    let unit = compile_test_unit(
        35,
        r#"
        let countdown = function(self, value, acc) {
            if (value === 0) {
                return acc;
            }
            return self(self, value - 1, acc + 1);
        };
        countdown(countdown, 200, 0);
        "#,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(200));
    assert_eq!(vm.peak_frame_depth(), 2);
}

#[test]
fn tail_calls_through_rebound_global_eval_reuse_frame_depth() {
    let unit = compile_test_unit(
        37,
        r#"
        var callCount = 0;
        function f(n) {
            "use strict";
            if (n === 0) {
                callCount += 1;
                return callCount;
            }
            return eval(n - 1);
        }
        eval = f;
        f(8);
        "#,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(1));
    assert_eq!(vm.peak_frame_depth(), 2);
}

#[test]
fn tail_calls_preserve_constructor_fallback_result_semantics() {
    let unit = compile_test_unit(
        36,
        r#"
        function Box(helper) {
            this.value = 4;
            return helper(1);
        }
        let box = new Box(function(value) { return value; });
        box.value;
        "#,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .unwrap();

    assert_eq!(result, Value::from_smi(4));
}

use super::support::*;

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
        .map(|view| decode_string(&view))
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
        .map(|view| decode_string(&view))
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
        .map(|view| decode_string(&view))
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
        .map(|view| decode_string(&view))
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
        .map(|view| decode_string(&view))
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
#[expect(
    clippy::too_many_lines,
    reason = "semantic regression scenario stays contiguous within its domain-focused test module"
)]
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
            &realm,
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
        .map(|view| decode_string(&view))
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
            &realm,
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
            &realm,
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
            &realm,
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
            &realm,
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
        .map(|view| decode_string(&view))
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
#[expect(
    clippy::too_many_lines,
    reason = "semantic regression scenario stays contiguous within its domain-focused test module"
)]
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
        .map(|view| decode_string(&view))
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
        .map(|view| decode_string(&view))
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

use super::support::*;

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
        module_key,
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
        r"
            (function() {
                var log = [];
                import(log.push('first'), (log.push('second'), undefined))
                    .then(null, function() {});
                return log[0] + ',' + log[1];
            })();
        ",
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
        module_key,
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
        r"
            (async function() {
                await 0;
                return import('./dep.mjs');
            })();
        ",
    );
    let host = TestHost::new();
    let script_referrer = ModuleKey::new("/tmp/main.js");
    let module_key = ModuleKey::new("/tmp/dep.mjs");
    host.define_module_source(
        "./dep.mjs",
        LoadedModuleSource::new(module_key.clone(), "/tmp/dep.mjs", "export default 13;"),
    );
    host.define_import_meta(
        module_key,
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
        r"
            Promise.resolve().then(function() {
                return import('./dep.mjs');
            });
        ",
    );
    let host = TestHost::new();
    let script_referrer = ModuleKey::new("/tmp/main.js");
    let module_key = ModuleKey::new("/tmp/dep.mjs");
    host.define_module_source(
        "./dep.mjs",
        LoadedModuleSource::new(module_key.clone(), "/tmp/dep.mjs", "export default 17;"),
    );
    host.define_import_meta(
        module_key,
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
        r"
            Promise.all([
                import('./dep.mjs'),
                import('./dep.mjs')
            ]).then(async function() {
                await import('./dep.mjs');
                await import('./dep.mjs');
            });
        ",
    );
    let host = TestHost::new();
    let script_referrer = ModuleKey::new("/tmp/main.js");
    let module_key = ModuleKey::new("/tmp/dep.mjs");
    host.define_module_source(
        "./dep.mjs",
        LoadedModuleSource::new(
            module_key,
            "/tmp/dep.mjs",
            r"
                var global = Function('return this;')();
                if (global.dynamicImportMarker) {
                    throw new Error('Module was evaluated more than once.');
                }
                global.dynamicImportMarker = 19;
                export default null;
            ",
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
    let state_source = r"
        export let score = 0;
        export function step(value) {
            score = score * 10 + value;
        }
    ";
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
            r"
                import { step } from './state.mjs';
                import('./b.mjs');
                step(1);
            ",
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
fn dynamic_import_waits_for_current_top_level_await_evaluation() {
    let unit = compile_test_unit(
        260,
        r"
            let continueExecution;
            globalThis.promise = new Promise(function(resolve) {
                continueExecution = resolve;
            });
            const executionStartPromise = new Promise(function(resolve) {
                globalThis.executionStarted = resolve;
            });

            async function run() {
                const first = import('./tla.mjs');
                await executionStartPromise;
                const second = import('./tla.mjs');
                await import('./empty.mjs');
                continueExecution();

                let secondResolved = false;
                const results = await Promise.all([
                    first.then(function() {
                        return !secondResolved;
                    }),
                    second.then(function() {
                        secondResolved = true;
                        return true;
                    })
                ]);
                return results[0] === true && results[1] === true;
            }

            run();
        ",
    );
    let host = TestHost::new();
    host.define_module_source(
        "./tla.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/tla.mjs"),
            "/tmp/tla.mjs",
            r"
                globalThis.executionStarted();
                export let x = 1;
                await globalThis.promise;
            ",
        ),
    );
    host.define_module_source(
        "./empty.mjs",
        LoadedModuleSource::new(ModuleKey::new("/tmp/empty.mjs"), "/tmp/empty.mjs", ""),
    );
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .expect("script should evaluate");
    let promise = result
        .as_object_ref()
        .expect("run should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("run promise should remain tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_bool(true));
}

#[test]
fn top_level_await_does_not_resume_before_sibling_module_evaluation() {
    let main = compile_test_module(
        351,
        "import './async.mjs'; import { check } from './sync.mjs'; export const observed = check;",
    );
    let async_dependency = compile_test_module(
        352,
        "globalThis.check = false; await 0; globalThis.check = true;",
    );
    let sync_dependency = compile_test_module(353, "export const { check } = globalThis;");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let main_key = ModuleKey::new("/tmp/main.mjs");
    let async_key = ModuleKey::new("/tmp/async.mjs");
    let sync_key = ModuleKey::new("/tmp/sync.mjs");
    let mut vm = Vm::new();

    vm.install_module(agent, realm.id(), &main_key, "/tmp/main.mjs", &main)
        .unwrap();
    vm.install_module(
        agent,
        realm.id(),
        &async_key,
        "/tmp/async.mjs",
        &async_dependency,
    )
    .unwrap();
    vm.install_module(
        agent,
        realm.id(),
        &sync_key,
        "/tmp/sync.mjs",
        &sync_dependency,
    )
    .unwrap();
    assert!(agent.set_module_requested_key(&main_key, 0, Some(async_key)));
    assert!(agent.set_module_requested_key(&main_key, 1, Some(sync_key)));

    let _ = vm.evaluate_linked_module(agent, realm, &main_key).unwrap();

    let record = agent
        .module_record(&main_key)
        .expect("main module should stay cached");
    let module_env = record
        .environment()
        .expect("main module evaluation should allocate one environment");
    let observed_slot = main
        .local_exports()
        .iter()
        .find(|entry| main.atom_text(entry.export_name()) == Some("observed"))
        .expect("main module should export observed")
        .local_slot();

    assert_eq!(
        agent.environment_slot(module_env, observed_slot),
        Some(Value::from_bool(false))
    );
}

#[test]
fn top_level_await_dynamic_imports_settle_leaf_before_parent() {
    let unit = compile_test_unit(
        261,
        r"
            globalThis.logs = [];
            globalThis.p1 = Promise.withResolvers();
            globalThis.pAStart = Promise.withResolvers();
            globalThis.pBStart = Promise.withResolvers();

            const imports = Promise.all([
                globalThis.pBStart.promise.then(function() {
                    return import('./a.mjs').finally(function() {
                        globalThis.logs.push('A');
                    });
                }).catch(function() {}),
                import('./b.mjs').finally(function() {
                    globalThis.logs.push('B');
                }).catch(function() {})
            ]);
            Promise.all([
                globalThis.pAStart.promise,
                globalThis.pBStart.promise
            ]).then(globalThis.p1.resolve);

            imports.then(function() {
                return globalThis.logs.join(',');
            });
        ",
    );
    let host = TestHost::new();
    host.define_module_source(
        "./a-sentinel.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/a-sentinel.mjs"),
            "/tmp/a-sentinel.mjs",
            "globalThis.pAStart.resolve();",
        ),
    );
    host.define_module_source(
        "./a.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/a.mjs"),
            "/tmp/a.mjs",
            "import './a-sentinel.mjs'; import './b.mjs';",
        ),
    );
    host.define_module_source(
        "./b-sentinel.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/b-sentinel.mjs"),
            "/tmp/b-sentinel.mjs",
            "globalThis.pBStart.resolve();",
        ),
    );
    host.define_module_source(
        "./b.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/b.mjs"),
            "/tmp/b.mjs",
            "import './b-sentinel.mjs'; await globalThis.p1.promise;",
        ),
    );
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .expect("script should evaluate");
    let promise = result
        .as_object_ref()
        .expect("script should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("script promise should stay tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    let text = record
        .result()
        .as_string_ref()
        .and_then(|string| agent.heap().view().string_view(string))
        .map(decode_string)
        .expect("script promise should fulfill with joined logs");
    assert_eq!(text, "B,A");
}

#[test]
fn top_level_await_dynamic_import_rejections_settle_leaf_before_parent() {
    let unit = compile_test_unit(
        262,
        r"
            globalThis.logs = [];
            globalThis.p1 = Promise.withResolvers();
            globalThis.pAStart = Promise.withResolvers();
            globalThis.pBStart = Promise.withResolvers();

            const imports = Promise.all([
                globalThis.pBStart.promise.then(function() {
                    return import('./a.mjs').finally(function() {
                        globalThis.logs.push('A');
                    });
                }).catch(function() {}),
                import('./b.mjs').finally(function() {
                    globalThis.logs.push('B');
                }).catch(function() {})
            ]);
            Promise.all([
                globalThis.pAStart.promise,
                globalThis.pBStart.promise
            ]).then(globalThis.p1.reject);

            imports.then(function() {
                return globalThis.logs.join(',');
            });
        ",
    );
    let host = TestHost::new();
    host.define_module_source(
        "./a-sentinel.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/reject-a-sentinel.mjs"),
            "/tmp/reject-a-sentinel.mjs",
            "globalThis.pAStart.resolve();",
        ),
    );
    host.define_module_source(
        "./a.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/reject-a.mjs"),
            "/tmp/reject-a.mjs",
            "import './a-sentinel.mjs'; import './b.mjs';",
        ),
    );
    host.define_module_source(
        "./b-sentinel.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/reject-b-sentinel.mjs"),
            "/tmp/reject-b-sentinel.mjs",
            "globalThis.pBStart.resolve();",
        ),
    );
    host.define_module_source(
        "./b.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/reject-b.mjs"),
            "/tmp/reject-b.mjs",
            "import './b-sentinel.mjs'; await globalThis.p1.promise;",
        ),
    );
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .expect("script should evaluate");
    let promise = result
        .as_object_ref()
        .expect("script should return a promise object");
    let record = agent
        .promise_record(promise)
        .expect("script promise should stay tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    let text = record
        .result()
        .as_string_ref()
        .and_then(|string| agent.heap().view().string_view(string))
        .map(decode_string)
        .expect("script promise should fulfill with joined logs");
    assert_eq!(text, "B,A");
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
fn dynamic_import_defer_sync_module_evaluates_on_namespace_access() {
    let unit = compile_test_unit(
        266,
        r"
            globalThis.evaluations = [];
            import.defer('./sync.mjs').then(ns => {
                globalThis.beforeDeferredAccess = globalThis.evaluations.join(',');
                ns.x;
                globalThis.afterDeferredAccess = globalThis.evaluations.join(',');
            });
        ",
    );
    let host = TestHost::new();
    let script_referrer = ModuleKey::new("/tmp/main.js");
    host.define_module_source(
        "./dep.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/dep.mjs"),
            "/tmp/dep.mjs",
            "globalThis.evaluations.push('dep');",
        ),
    );
    host.define_module_source(
        "./sync.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/sync.mjs"),
            "/tmp/sync.mjs",
            "import './dep.mjs'; globalThis.evaluations.push('sync');",
        ),
    );
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let _ = vm
        .evaluate_script_with_registry_and_host_referrer(
            agent,
            realm,
            &unit,
            Some(&script_referrer),
            &host,
            &mut registry,
        )
        .expect("import.defer sync regression should evaluate");

    let before = global_value(agent, &realm, "beforeDeferredAccess")
        .as_string_ref()
        .and_then(|string| agent.heap().view().string_view(string))
        .map(decode_string)
        .expect("beforeDeferredAccess should be a string");
    let after = global_value(agent, &realm, "afterDeferredAccess")
        .as_string_ref()
        .and_then(|string| agent.heap().view().string_view(string))
        .map(decode_string)
        .expect("afterDeferredAccess should be a string");
    assert_eq!(before, "");
    assert_eq!(after, "dep,sync");
}

#[test]
fn static_import_defer_sync_module_evaluates_on_namespace_access() {
    let host = TestHost::new();
    host.define_module_source(
        "./setup.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/setup.mjs"),
            "/tmp/setup.mjs",
            "globalThis.evaluations = [];",
        ),
    );
    host.define_module_source(
        "./dep.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/dep.mjs"),
            "/tmp/dep.mjs",
            "globalThis.evaluations.push('dep'); export const y = 1;",
        ),
    );
    host.define_module_source(
        "./sync.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/sync.mjs"),
            "/tmp/sync.mjs",
            "import './dep.mjs'; globalThis.evaluations.push('sync'); export const x = 2;",
        ),
    );
    host.define_module_source(
        "main.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/main.mjs"),
            "/tmp/main.mjs",
            r"
            import './setup.mjs';
            import defer * as ns from './sync.mjs';
            globalThis.beforeDeferredAccess = globalThis.evaluations.join(',');
            ns.x;
            globalThis.afterDeferredAccess = globalThis.evaluations.join(',');
        ",
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
            &realm,
            &host,
            &ModuleSourceRequest {
                specifier: "main.mjs".into(),
                referrer: None,
                attributes: Vec::new(),
            },
        )
        .expect("static import defer sync regression should load");
    vm.evaluate_linked_module_with_registry_and_host(
        agent,
        realm,
        loaded.key(),
        &host,
        &mut registry,
    )
    .expect("static import defer sync regression should evaluate");

    let before = global_value(agent, &realm, "beforeDeferredAccess")
        .as_string_ref()
        .and_then(|string| agent.heap().view().string_view(string))
        .map(decode_string)
        .expect("beforeDeferredAccess should be a string");
    let after = global_value(agent, &realm, "afterDeferredAccess")
        .as_string_ref()
        .and_then(|string| agent.heap().view().string_view(string))
        .map(decode_string)
        .expect("afterDeferredAccess should be a string");
    assert_eq!(before, "");
    assert_eq!(after, "dep,sync");
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "semantic regression scenario stays contiguous within its domain-focused test module"
)]
fn static_import_defer_tla_module_throws_until_async_evaluation_completes() {
    let host = TestHost::new();
    host.define_module_source(
        "./main.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/main.mjs"),
            "/tmp/main.mjs",
            r"
                import { done } from './promises.mjs';
                import './dep.mjs';
                (async function() {
                    await done;
                    globalThis.doneBeforeDeferredAsyncReady =
                        globalThis.beforeDeferredAsyncReady instanceof TypeError;
                    globalThis.doneDuringDeferredAsyncReady =
                        globalThis.duringDeferredAsyncReady instanceof TypeError;
                    globalThis.doneAfterDeferredAsyncReady =
                        globalThis.afterDeferredAsyncReady;
                })();
            ",
        ),
    );
    host.define_module_source(
        "./promises.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/promises.mjs"),
            "/tmp/promises.mjs",
            r"
                export let resolveDone, rejectDone;
                export const done = new Promise((resolve, reject) => {
                    resolveDone = resolve;
                    rejectDone = reject;
                });
                export let resolveFirst, resolveSecond, resolveThird;
                export const first = new Promise((resolve) => { resolveFirst = resolve; });
                export const second = new Promise((resolve) => { resolveSecond = resolve; });
                export const third = new Promise((resolve) => { resolveThird = resolve; });
            ",
        ),
    );
    host.define_module_source(
        "./observer.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/observer.mjs"),
            "/tmp/observer.mjs",
            r"
                import { first, third, resolveDone, rejectDone, resolveSecond } from './promises.mjs';
                import defer * as ns from './dep.mjs';

                try {
                    ns.foo;
                } catch (error) {
                    globalThis.beforeDeferredAsyncReady = error;
                }

                first.then(() => {
                    try {
                        ns.foo;
                    } catch (error) {
                        globalThis.duringDeferredAsyncReady = error;
                    }
                    resolveSecond();
                }).then(() => {
                    return third.then(() => {
                        try {
                            globalThis.afterDeferredAsyncReady = ns.foo;
                        } catch (error) {
                            globalThis.afterDeferredAsyncReadyError = error instanceof TypeError;
                        }
                    });
                }).then(resolveDone, rejectDone);
            ",
        ),
    );
    host.define_module_source(
        "./dep.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/dep.mjs"),
            "/tmp/dep.mjs",
            r"
                import { resolveFirst, resolveThird, second } from './promises.mjs';
                import './observer.mjs';

                await Promise.resolve();
                resolveFirst();
                await second;
                resolveThird();

                export let foo = 1;
            ",
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
            &realm,
            &host,
            &ModuleSourceRequest {
                specifier: "./main.mjs".into(),
                referrer: None,
                attributes: Vec::new(),
            },
        )
        .expect("static import defer async graph should load");

    vm.evaluate_linked_module_with_registry_and_host(
        agent,
        realm,
        loaded.key(),
        &host,
        &mut registry,
    )
    .expect("static import defer async graph should evaluate");

    assert!(global_value(agent, &realm, "beforeDeferredAsyncReady")
        .as_object_ref()
        .is_some());
    assert!(global_value(agent, &realm, "duringDeferredAsyncReady")
        .as_object_ref()
        .is_some());
    assert_eq!(
        global_value(agent, &realm, "afterDeferredAsyncReadyError"),
        Value::undefined()
    );
    assert_eq!(
        global_value(agent, &realm, "afterDeferredAsyncReady"),
        Value::from_smi(1)
    );
    assert_eq!(
        global_value(agent, &realm, "doneBeforeDeferredAsyncReady"),
        Value::from_bool(true)
    );
    assert_eq!(
        global_value(agent, &realm, "doneDuringDeferredAsyncReady"),
        Value::from_bool(true)
    );
    assert_eq!(
        global_value(agent, &realm, "doneAfterDeferredAsyncReady"),
        Value::from_smi(1)
    );
}

#[test]
fn static_import_defer_preserves_prior_evaluation_error_identity() {
    let unit = compile_test_unit(
        267,
        r"
            import('./throws.mjs').catch(function(first) {
                globalThis.firstDeferredError = first;
                return import('./deferred.mjs').then(function(module) {
                    try {
                        module.ns.foo;
                    } catch (second) {
                        globalThis.secondDeferredError = second;
                        globalThis.deferredErrorIdentity = first === second;
                    }
                });
            });
        ",
    );
    let host = TestHost::new();
    let script_referrer = ModuleKey::new("/tmp/main.js");
    host.define_module_source(
        "./throws.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/throws.mjs"),
            "/tmp/throws.mjs",
            "throw { someError: 'the error from throws.mjs' };",
        ),
    );
    host.define_module_source(
        "./deferred.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/deferred.mjs"),
            "/tmp/deferred.mjs",
            "import defer * as ns from './throws.mjs'; export { ns };",
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
        .expect("deferred import error identity regression should evaluate");
    let promise = result
        .as_object_ref()
        .expect("script result should be the dynamic import promise chain");
    let record = agent
        .promise_record(promise)
        .expect("script result promise should stay tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(
        global_value(agent, &realm, "deferredErrorIdentity"),
        Value::from_bool(true)
    );
    assert_eq!(
        global_value(agent, &realm, "firstDeferredError"),
        global_value(agent, &realm, "secondDeferredError")
    );
}

#[test]
fn static_source_phase_import_rejects_source_text_modules_with_syntax_error() {
    let host = TestHost::new();
    host.define_module_source(
        "entry.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/entry.mjs"),
            "entry.mjs",
            "import source x from './dep.mjs';",
        ),
    );
    host.define_module_source(
        "./dep.mjs",
        LoadedModuleSource::new(
            ModuleKey::new("/tmp/dep.mjs"),
            "dep.mjs",
            "export const x = 7;",
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
        .expect("source-phase import module graph should parse and load");
    let error = vm
        .evaluate_linked_module_with_host(agent, realm, loaded.key(), &host)
        .expect_err("source text modules should not have source-phase representations");
    let VmError::Abrupt(reason) = error else {
        panic!("expected abrupt SyntaxError, got {error:?}");
    };
    let reason = reason
        .thrown_value()
        .expect("source-phase import should throw")
        .as_object_ref()
        .expect("source-phase import failure should be an error object");
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
        module_key,
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

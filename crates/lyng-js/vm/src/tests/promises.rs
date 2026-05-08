use super::support::*;

#[test]
fn promise_checkpoint_drains_reaction_jobs_and_reports_host_phases() {
    let unit = compile_test_unit(
        217,
        r"
            (function() {
                let resolve;
                let reject;
                let promise = new Promise(function(innerResolve, innerReject) {
                    resolve = innerResolve;
                    reject = innerReject;
                });
                return [promise, resolve, reject];
            })();
        ",
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
        r"
            Promise.resolve(1).then().then();
        ",
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
        r"
            Promise.resolve(1)
                .then(function(value) {
                    return value + 1;
                })
                .then();
        ",
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
        r"
            Promise.all([Promise.resolve(1), 2, Promise.resolve(3)]);
        ",
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
fn evaluate_script_promise_reaction_can_update_global_lexical_binding() {
    let unit = compile_test_unit(
        221,
        r"
            let outcomes = [];
            Promise.all([Promise.resolve(1), 2, Promise.resolve(3)])
                .then(results => (outcomes = results), function() {});
        ",
    );
    let readback = compile_test_unit(222, "outcomes[2];");
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let _ = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();
    vm.checkpoint_promise_jobs(agent, &host, &mut registry)
        .expect("promise job should update top-level lexical binding");

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &readback, &host, &mut registry)
        .expect("readback should load updated top-level lexical");

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn evaluate_script_promise_reaction_rebinding_is_visible_to_existing_closure() {
    let unit = compile_test_unit(
        223,
        r#"
            let outcomes = [];
            let events = [];
            function observe() {
                events.push("observe:" + outcomes.length);
            }
            observe();
            Promise.resolve([1]).then(results => {
                events.push("then:" + results.length);
                outcomes = results;
            }, function() {});
        "#,
    );
    let readback = compile_test_unit(
        224,
        r#"
            observe();
            events.join(",");
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let _ = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();
    vm.checkpoint_promise_jobs(agent, &host, &mut registry)
        .expect("promise job should update top-level lexical binding");

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &readback, &host, &mut registry)
        .expect("readback should call existing closure");
    let text = result
        .as_string_ref()
        .and_then(|string| agent.heap().view().string_view(string))
        .map(decode_string)
        .expect("event trace should be a string");

    assert_eq!(text, "observe:0,then:1,observe:1");
}

#[test]
fn checkpoint_harness_callback_observes_rebound_lexical_from_promise_job() {
    let unit = compile_test_unit(
        225,
        r#"
            let outcomes = [];
            let events = [];
            function observe() {
                events.push("observe:" + outcomes.length);
            }
            observe();
            Promise.resolve([1]).then(results => {
                events.push("then:" + results.length);
                outcomes = results;
            }, function() {});
        "#,
    );
    let readback = compile_test_unit(226, "events.join(',');");
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let _ = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();

    let observe_key = PropertyKey::from_atom(agent.atoms_mut().intern_collectible("observe"));
    let observe = ordinary_get(agent, realm.global_object(), observe_key)
        .unwrap()
        .as_object_ref()
        .expect("observe should be installed as a global function");
    let reaction = agent.alloc_promise_reaction(PromiseReactionRecord::new(
        PromiseReactionKind::Fulfill,
        PromiseReactionHandler::Callable(observe),
        None,
    ));
    let _ = agent.enqueue_job_with_payload(
        HostJobKind::Harness,
        ExecutableId::Builtin,
        RuntimeJobPayload::PromiseReaction {
            reaction,
            argument: Value::undefined(),
        },
        Some(realm.id()),
        Some("HarnessObserve".into()),
    );

    vm.checkpoint_promise_jobs(agent, &host, &mut registry)
        .expect("promise and harness jobs should drain");

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &readback, &host, &mut registry)
        .expect("readback should evaluate");
    let text = result
        .as_string_ref()
        .and_then(|string| agent.heap().view().string_view(string))
        .map(decode_string)
        .expect("event trace should be a string");

    assert_eq!(text, "observe:0,then:1,observe:1");
}

#[test]
fn evaluate_script_top_level_closures_share_rebound_lexical_binding() {
    let unit = compile_test_unit(
        223,
        r#"
            let outcomes = [];
            let events = [];
            function observe() {
                events.push("observe:" + outcomes.length);
            }
            function update(results) {
                events.push("update:" + results.length);
                outcomes = results;
            }
            observe();
            update([1]);
            observe();
            events.join(",");
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("script should evaluate");

    let text = result
        .as_string_ref()
        .and_then(|string| agent.heap().view().string_view(string))
        .map(decode_string)
        .expect("event trace should be a string");
    assert_eq!(text, "observe:0,update:1,observe:1");
}

#[test]
fn evaluate_script_top_level_arrow_and_function_share_rebound_lexical_binding() {
    let unit = compile_test_unit(
        224,
        r#"
            let outcomes = [];
            let events = [];
            function observe() {
                events.push("observe:" + outcomes.length);
            }
            const update = results => {
                events.push("update:" + results.length);
                outcomes = results;
            };
            observe();
            update([1]);
            observe();
            events.join(",");
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("script should evaluate");

    let text = result
        .as_string_ref()
        .and_then(|string| agent.heap().view().string_view(string))
        .map(decode_string)
        .expect("event trace should be a string");
    assert_eq!(text, "observe:0,update:1,observe:1");
}

#[test]
fn evaluate_script_named_function_expression_closure_observes_rebound_lexical_binding() {
    let unit = compile_test_unit(
        227,
        r#"
            let outcomes = [];
            let events = [];
            let saved;
            (function observe() {
                events.push("observe:" + outcomes.length);
                saved = observe;
            })();
            Promise.resolve([1]).then(results => {
                events.push("then:" + results.length);
                outcomes = results;
            }, function() {});
        "#,
    );
    let readback = compile_test_unit(
        228,
        r#"
            saved();
            events.join(",");
        "#,
    );
    let host = TestHost::new();
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let mut registry = RejectingRegistry;

    let _ = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .unwrap();
    vm.checkpoint_promise_jobs(agent, &host, &mut registry)
        .expect("promise job should update top-level lexical binding");

    let result = vm
        .evaluate_script_with_registry_and_host(agent, realm, &readback, &host, &mut registry)
        .expect("readback should call saved closure");
    let text = result
        .as_string_ref()
        .and_then(|string| agent.heap().view().string_view(string))
        .map(decode_string)
        .expect("event trace should be a string");
    assert_eq!(text, "observe:0,then:1,observe:1");
}

#[test]
fn evaluate_script_array_from_async_resolves_sync_iterable_values() {
    let unit = compile_test_unit(
        220,
        r"
            Array.fromAsync([Promise.resolve(1), 2], function(value, index) {
                return Promise.resolve(value + index + 1);
            });
        ",
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
        r"
            Array.fromAsync({ length: 1n, 0: 0 });
        ",
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
        r"
            Promise.allSettled([Promise.resolve(1), Promise.reject(2)]);
        ",
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
        r"
            Promise.race([Promise.resolve(4), new Promise(function() {})]);
        ",
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
        r"
            Promise.all(false)
                .then(function() { return false; }, function(error) {
                    return error instanceof TypeError;
                })
                .then();
        ",
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
        r"
            var callCount = 0;
            var boundResolve = Promise.resolve.bind(Promise);
            Promise.resolve = function(value) {
                callCount += 1;
                return boundResolve(value);
            };
            Promise.all([1, 2, 3]).then(function() { return callCount; }).then();
        ",
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
fn evaluate_script_array_index_assignment_observes_prototype_setter() {
    let unit = compile_test_unit(
        226,
        r#"
            let observed = 0;
            Object.defineProperty(Array.prototype, "0", {
                configurable: true,
                set: function(value) {
                    observed = value;
                }
            });
            let array = [];
            array[0] = 42;
            delete Array.prototype[0];
            observed;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(42));
}

#[test]
fn evaluate_script_array_index_assignment_defers_to_typed_array_prototype_set() {
    let unit = compile_test_unit(
        227,
        r#"
            let calls = 0;
            let value = {
                valueOf: function() {
                    calls = calls + 1;
                    return 23;
                }
            };
            let typed = new Uint8Array([0]);
            let receiver = Object.setPrototypeOf([], typed);
            receiver[1] = value;
            (receiver.hasOwnProperty("1") ? 1 : 0) + receiver.length + calls;
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
fn evaluate_script_function_apply_observes_array_prototype_getter() {
    let unit = compile_test_unit(
        228,
        r#"
            let observed = 0;
            Object.defineProperty(Array.prototype, "0", {
                configurable: true,
                get: function() {
                    observed = 5;
                    return 37;
                }
            });
            function first(value) {
                return value + observed;
            }
            let args = [];
            args.length = 1;
            let result = first.apply(null, args);
            delete Array.prototype[0];
            result;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(42));
}

#[test]
fn evaluate_script_string_from_code_point_apply_uses_dense_array_fast_path() {
    let unit = compile_test_unit(
        14_229,
        r"
            let args = [];
            for (let i = 0; i < 64; i = i + 1) {
                args[i] = 65;
            }
            String.fromCodePoint.apply(null, args).length;
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(64));
    assert!(vm.string_code_units_scratch_capacity() >= 64);
}

#[test]
fn evaluate_script_resolves_promise_any_with_first_fulfillment() {
    let unit = compile_test_unit(
        229,
        r"
            Promise.any([Promise.reject(1), Promise.resolve(7), Promise.reject(3)]).then();
        ",
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
        r"
            Promise.any([])
                .then(function() { return false; }, function(error) {
                    return error instanceof AggregateError
                        && error.errors.length === 0;
                })
                .then();
        ",
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
        r"
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
        ",
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
        r"
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
        ",
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
fn evaluate_script_promise_any_iterator_value_errors_do_not_close_iterator() {
    let unit = compile_test_unit(
        235,
        r#"
            var error = new Error("value");
            var returnCount = 0;
            var poisonedValue = { done: false };
            Object.defineProperty(poisonedValue, "value", {
                get: function() {
                    throw error;
                }
            });
            var iterable = {
                [Symbol.iterator]: function() {
                    return {
                        next: function() {
                            return poisonedValue;
                        },
                        return: function() {
                            returnCount = returnCount + 1;
                            return {};
                        }
                    };
                }
            };
            Promise.any(iterable);
            returnCount;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(0));
}

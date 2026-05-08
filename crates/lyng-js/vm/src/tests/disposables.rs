use super::support::*;

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
fn async_disposable_stack_dispose_async_awaits_null_resources_before_resolving() {
    let unit = compile_test_unit(
        2310,
        r#"
            var stack = new AsyncDisposableStack();
            var log = [];

            stack.use(null);

            Promise.resolve()
                .then(function() { return 0; })
                .then(function() { log.push("job 1"); });

            var disposed = stack.disposeAsync().then(function() {
                log.push("dispose");
            });

            Promise.resolve()
                .then(function() { return 0; })
                .then(function() { log.push("job 2"); });

            Promise.all([disposed]).then(function() {
                return log.join(",") === "job 1,dispose,job 2";
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
        .expect("disposeAsync ordering check should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("disposeAsync ordering promise should remain tracked");

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
fn using_declarations_dispose_in_statement_contexts() {
    let unit = compile_test_unit(
        2315,
        r#"
            var results = [];

            function check(name, run, expected) {
                try {
                    var value = run();
                    results.push(name + ":" + (value === expected ? "ok" : value));
                } catch (error) {
                    results.push(name + ":" + error.name);
                }
            }

            check("for", function() {
                var values = [];
                for (let i = 0; i < 3; i++) {
                    using x = {
                        value: i,
                        [Symbol.dispose]() {
                            values.push(this.value);
                        }
                    };
                }
                values.push(3);
                return values.join(",");
            }, "0,1,2,3");

            check("for-in", function() {
                var values = [];
                for (let i in [0, 1]) {
                    using x = {
                        value: i,
                        [Symbol.dispose]() {
                            values.push(this.value);
                        }
                    };
                }
                values.push("2");
                return values.join(",");
            }, "0,1,2");

            check("for-of", function() {
                var values = [];
                for (let i of [0, 1]) {
                    using x = {
                        value: i,
                        [Symbol.dispose]() {
                            values.push(this.value);
                        }
                    };
                }
                values.push(2);
                return values.join(",");
            }, "0,1,2");

            check("function", function() {
                var values = [];
                (function() {
                    using x = {
                        [Symbol.dispose]() {
                            values.push(1);
                        }
                    };
                    using y = {
                        [Symbol.dispose]() {
                            values.push(2);
                        }
                    };
                })();
                return values.join(",");
            }, "2,1");

            check("missing", function() {
                try {
                    {
                        using x = { value: 1 };
                    }
                    return "no-throw";
                } catch (error) {
                    return error instanceof TypeError ? "type-error" : "other-error";
                }
            }, "type-error");

            results.join("|");
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let value = result
        .as_string_ref()
        .expect("statement disposal check should return a string");
    let decoded = decode_string(
        agent
            .heap()
            .view()
            .string_view(value)
            .expect("string should remain in the heap"),
    );

    assert_eq!(decoded, "for:ok|for-in:ok|for-of:ok|function:ok|missing:ok");
}

#[test]
fn loop_iteration_slot_sync_active_stack_avoids_scratch_vectors() {
    let unit = compile_test_unit(
        2316,
        r"
            var total = 0;
            for (let outer = 0; outer < 6; outer = outer + 1) {
                let saved;
                for (let i = 0; i < 6; i = i + 1) {
                    saved = function() { return i; };
                    i = i + 0;
                }
                total = total + saved();
            }
            total;
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(30));
    let (source_len, target_len, source_capacity, target_capacity) =
        vm.loop_iteration_scratch_state_for_tests();
    assert_eq!(source_len, 0);
    assert_eq!(target_len, 0);
    assert_eq!(source_capacity, 0);
    assert_eq!(target_capacity, 0);
}

#[test]
fn loop_iteration_slot_sync_single_active_environment_avoids_scratch_vectors() {
    let unit = compile_test_unit(
        14_230,
        r"
            var saved;
            for (let i = 0; i < 16; i = i + 1) {
                saved = function() { return i; };
                i = i + 0;
            }
            saved();
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(15));
    let (source_len, target_len, source_capacity, target_capacity) =
        vm.loop_iteration_scratch_state_for_tests();
    assert_eq!(source_len, 0);
    assert_eq!(target_len, 0);
    assert_eq!(source_capacity, 0);
    assert_eq!(target_capacity, 0);
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
fn skipped_await_using_declaration_does_not_suspend_async_function() {
    let unit = compile_test_unit(
        2309,
        r"
            var sameTurn = true;

            async function run() {
                var started = sameTurn;
                var before = false;
                var after = false;
                outer: {
                    before = sameTurn;
                    break outer;
                    await using resource = null;
                }
                after = sameTurn;
                return started && before && after;
            }

            var result = run();
            sameTurn = false;
            result;
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
        .expect("skipped await using result should be a promise");
    let record = agent
        .promise_record(promise)
        .expect("skipped await using promise should remain tracked");
    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_bool(true));
}

#[test]
fn evaluate_module_supports_top_level_await_using() {
    let unit = compile_test_module(
        2314,
        r"
            export let disposed = false;
            await using resource = {
                [Symbol.dispose]() {
                    disposed = true;
                }
            };
        ",
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
        r"
            try {
                {
                    using x = x + 1;
                }
                false;
            } catch (error) {
                error instanceof ReferenceError;
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
fn using_function_local_initializer_reads_trigger_reference_error() {
    let unit = compile_test_unit(
        2310,
        r"
            function f() {
                using x = x + 1;
            }

            try {
                f();
                false;
            } catch (error) {
                error instanceof ReferenceError;
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
fn assigning_to_using_bindings_in_for_of_bodies_throws_type_error() {
    let unit = compile_test_unit(
        2311,
        r"
            try {
                for (using x of [null]) {
                    x = { [Symbol.dispose]() {} };
                }
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
fn assigning_to_using_bindings_in_for_update_throws_type_error() {
    let unit = compile_test_unit(
        2312,
        r"
            try {
                for (using i = null; i === null; i = { [Symbol.dispose]() {} }) {}
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
fn using_cleanup_nests_multiple_disposal_errors_as_suppressed_error() {
    let unit = compile_test_unit(
        2313,
        r"
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
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

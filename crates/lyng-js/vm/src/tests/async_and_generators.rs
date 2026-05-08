use super::support::*;

#[test]
fn generator_call_returns_a_generator_without_running_the_body() {
    let unit = compile_test_unit(
        204,
        r"
            var ran = 0;
            function* g() {
                ran = 1;
                yield 2;
            }
            var iter = g();
            ran;
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let iter = global_value(agent, &realm, "iter")
        .as_object_ref()
        .expect("generator call should return an object");

    assert_eq!(result, Value::from_smi(0));
    assert!(agent.objects().is_generator_object(iter));
}

#[test]
fn async_function_call_returns_a_promise_and_fulfills_after_await() {
    let unit = compile_test_unit(
        221,
        r"
            async function f() {
                return await Promise.resolve(41);
            }
            f();
        ",
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
        r"
            async function f() {
                await Promise.reject(99);
            }
            f();
        ",
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
        r"
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
        ",
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
        r"
            async function main() {
                var sum = 0;
                for await (const value of [Promise.resolve(1), 2]) {
                    sum += value;
                }
                return sum;
            }
            main();
        ",
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
fn for_await_of_sync_iterator_preserves_async_from_sync_tick_order() {
    let unit = compile_test_unit(
        2310,
        r#"
            var actual = [];
            Promise.resolve(0)
                .then(function() { actual.push("tick 1"); })
                .then(function() { actual.push("tick 2"); })
                .then(function() { actual.push("tick 3"); })
                .then(function() { actual.push("tick 4"); });

            Object.defineProperty(Promise.prototype, "constructor", {
                get() {
                    actual.push("constructor");
                    return Promise;
                },
                configurable: true
            });

            async function main() {
                var p = Promise.resolve(0);
                actual.push("pre");
                for await (var value of [p]) {
                    actual.push("loop");
                }
                actual.push("post");
                return actual.join(",");
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
    let actual = record
        .result()
        .as_string_ref()
        .and_then(|string| agent.heap().view().string_view(string))
        .map(decode_string)
        .expect("for await log should be a string");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(
        actual,
        "pre,constructor,constructor,tick 1,tick 2,loop,constructor,tick 3,tick 4,post"
    );
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
        r"
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
        ",
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
        r"
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
        ",
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
        r"
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
        ",
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
        r"
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
        ",
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
        r"
            async function main() {
                var iter = (async function* () {
                    yield Promise.resolve(7);
                })();
                var result = await iter.next();
                return result.value instanceof Promise ? -1 : result.value;
            }
            main();
        ",
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
        r"
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
        ",
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
        r"
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
        ",
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
fn async_generator_return_promise_resolve_abrupt_resumes_suspended_yield_as_throw() {
    let unit = compile_test_unit(
        339,
        r#"
            async function main() {
                var caught;
                async function* f() {
                    try {
                        yield;
                        return "unreachable";
                    } catch (err) {
                        caught = err.message;
                        return 1;
                    }
                }

                var brokenPromise = Promise.resolve(42);
                Object.defineProperty(brokenPromise, "constructor", {
                    get() {
                        throw new Error("broken promise");
                    }
                });

                var iter = f();
                await iter.next();
                var ret = await iter.return(brokenPromise);
                return caught + "|" + ret.value + "|" + ret.done;
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
        .expect("async generator abrupt return test should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("async generator abrupt return promise should remain tracked");
    let text = record
        .result()
        .as_string_ref()
        .and_then(|value| agent.heap().view().string_view(value).map(decode_string))
        .expect("async generator abrupt return should fulfill with a string");

    assert_eq!(text, "broken promise|1|true");
}

#[test]
fn async_generator_yield_star_return_resumption_awaits_before_delegate_return_lookup() {
    let unit = compile_test_unit(
        340,
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
fn async_iterator_prototype_async_dispose_method_has_spec_descriptor() {
    let unit = compile_test_unit(
        236,
        r#"
            var AsyncIteratorPrototype = Object.getPrototypeOf(
                Object.getPrototypeOf((async function* () {}).prototype)
            );
            var descriptor = Object.getOwnPropertyDescriptor(
                AsyncIteratorPrototype,
                Symbol.asyncDispose
            );
            var method = descriptor && descriptor.value;
            descriptor !== undefined &&
                typeof method === "function" &&
                method.name === "[Symbol.asyncDispose]" &&
                method.length === 0 &&
                descriptor.writable === true &&
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
fn async_iterator_prototype_async_dispose_rejects_when_return_throws() {
    let unit = compile_test_unit(
        237,
        r"
            var AsyncIteratorPrototype = Object.getPrototypeOf(
                Object.getPrototypeOf((async function* () {}).prototype)
            );
            var calls = 0;
            function Marker() {}
            var iter = {
                return: function(value) {
                    calls += value === undefined ? 1 : 100;
                    throw new Marker();
                }
            };

            AsyncIteratorPrototype[Symbol.asyncDispose].call(iter).then(
                function() {
                    return false;
                },
                function(error) {
                    return error instanceof Marker && calls === 1;
                }
            );
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
        .expect("async dispose rejection handler should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("async dispose rejection chain should remain tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_bool(true));
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
        r"
            var calls = 0;
            function* g(x = (calls += 1, 41)) {
                calls += 100;
                yield x;
            }
            var iter = g();
            calls;
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let iter = global_value(agent, &realm, "iter")
        .as_object_ref()
        .expect("generator call should return an object");

    assert_eq!(result, Value::from_smi(1));
    assert!(agent.objects().is_generator_object(iter));
}

#[test]
fn generator_call_throws_parameter_instantiation_errors_before_returning() {
    let unit = compile_test_unit(
        215,
        r"
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
        ",
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
        r"
            var g = function*(a = (g.prototype = null)) {};
            var oldPrototype = g.prototype;
            var iter = g();
            Object.getPrototypeOf(iter) === oldPrototype;
        ",
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
        r"
            function* g() {}
            var iter = g();
            var directProtoMatches = Object.getPrototypeOf(iter) === g.prototype;
            var sharedProto = Object.getPrototypeOf(g.prototype);
            var sharedNext = typeof sharedProto.next;
            var sharedTag = sharedProto[Symbol.toStringTag];
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let _ = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(
        global_value(agent, &realm, "directProtoMatches"),
        Value::from_bool(true)
    );
    let shared_next = global_value(agent, &realm, "sharedNext")
        .as_string_ref()
        .expect("typeof shared prototype next should be a string");
    let shared_tag = global_value(agent, &realm, "sharedTag")
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

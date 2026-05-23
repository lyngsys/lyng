use super::support::*;

#[test]
fn evaluate_script_promise_finally_invokes_then_on_thenables() {
    let unit = compile_test_unit(
        239,
        r"
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
fn evaluate_script_promise_finally_invokes_result_then_observably() {
    let unit = compile_test_unit(
        2401,
        r#"
            var sequence = [];
            var value = {};
            var reason = {};
            var rejected = Promise.reject(reason);
            rejected.then = function() {
                sequence.push(4);
                return Promise.prototype.then.apply(this, arguments);
            };

            Promise.resolve(value)
                .then(function(x) {
                    sequence.push(2);
                    return x;
                })
                .finally(function() {
                    sequence.push(3);
                    return rejected;
                })
                .catch(function(error) {
                    sequence.push(error === reason ? 5 : 50);
                    return sequence.join(",") === "2,3,4,5";
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
        .expect("finally observable-then check should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("finally observable-then promise should remain tracked");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_bool(true));
}

#[test]
fn evaluate_script_promise_then_throws_when_constructor_is_null() {
    let unit = compile_test_unit(
        241,
        r"
            var p = new Promise(function() {});
            p.constructor = null;
            try {
                p.then();
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
        r"
            var target = [1, 2, 3];
            Object.defineProperties(target, {
                length: { value: 1 }
            });
            target.length === 1 && target[1] === undefined;
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

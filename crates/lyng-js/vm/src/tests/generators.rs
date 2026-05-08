use super::support::*;

#[test]
fn generator_length_uses_expected_argument_count() {
    let unit = compile_test_unit(
        217,
        r"
            var lengths = [
                (function* (x = 42) {}).length,
                (function* (x, y = 42) {}).length,
                (function* (x, y = 42, z) {}).length
            ];
            lengths[0] * 100 + lengths[1] * 10 + lengths[2];
        ",
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
        r"
            (function* () {}).name.length;
        ",
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
        r"
            let callCount = 0;
            let ref = function* BindingIdentifier() {
                callCount++;
                BindingIdentifier = 1;
                return BindingIdentifier;
            };
            let result = ref().next().value === ref ? 1 : 0;
            result * 10 + callCount;
        ",
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
        r"
            function* g() {
                const value = yield 1;
                return value + 1;
            }
            var iter = g();
            var first = iter.next();
            var second = iter.next(41);
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let _ = vm.evaluate_script(agent, realm, &unit).unwrap();
    let first = global_value(agent, &realm, "first");
    let second = global_value(agent, &realm, "second");
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
        r"
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
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let final_state = vm.evaluate_script(agent, realm, &unit).unwrap();
    let result = global_value(agent, &realm, "result");
    let (value, done) = iterator_result_fields(agent, result);

    assert_eq!(final_state, Value::from_smi(7));
    assert_eq!(value, Value::from_smi(5));
    assert_eq!(done, Value::from_bool(true));
}

#[test]
fn for_of_continue_to_outer_loop_closes_the_iterator() {
    let unit = compile_test_unit(
        208,
        r"
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
        ",
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
        r"
            var flag1 = false;
            var flag2 = false;
            var x, y;
            var counter = 0;

            for ({ x = flag1 = true, y = flag2 = true } of [{ y: 1 }]) {
                counter += 1;
            }
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let _ = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(global_value(agent, &realm, "counter"), Value::from_smi(1));
    assert_eq!(global_value(agent, &realm, "flag1"), Value::from_bool(true));
    assert_eq!(
        global_value(agent, &realm, "flag2"),
        Value::from_bool(false)
    );
    assert_eq!(global_value(agent, &realm, "y"), Value::from_smi(1));
}

#[test]
fn typed_array_for_of_tracks_resizable_array_buffer_growth() {
    let unit = compile_test_unit(
        2311,
        r#"
            var rab = new ArrayBuffer(4, { maxByteLength: 8 });
            var values = new Uint8Array(rab);
            values[0] = 1;
            values[1] = 2;

            var seen = [];
            for (var value of values) {
                seen.push(value);
                if (seen.length === 2) {
                    rab.resize(6);
                }
            }
            seen.join(",");
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let actual = result
        .as_string_ref()
        .and_then(|string| agent.heap().view().string_view(string))
        .map(decode_string)
        .expect("for-of result should be a string");

    assert_eq!(actual, "1,2,0,0,0,0");
}

#[test]
fn yield_star_delegates_generator_values_and_final_completion() {
    let unit = compile_test_unit(
        207,
        r"
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
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let _ = vm.evaluate_script(agent, realm, &unit).unwrap();
    let first = global_value(agent, &realm, "first");
    let second = global_value(agent, &realm, "second");
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
        r"
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
        r"
            class C {
                *values() {
                    yield 1;
                    return 2;
                }
            }
            var iter = new C().values();
            var first = iter.next();
            var second = iter.next();
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let _ = vm.evaluate_script(agent, realm, &unit).unwrap();
    let first = global_value(agent, &realm, "first");
    let second = global_value(agent, &realm, "second");
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
        r"
            export default function* g() {
                yield 1;
            }
            export const value = g().next().value;
        ",
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

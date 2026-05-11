use super::support::*;

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
        r"
            var x = 1;
            var object = { x: 2 };
            with (object) {
                x = 3;
            }
            object.x * 10 + x;
        ",
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
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
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
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
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
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
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
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
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
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
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
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
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
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
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
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
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
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
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
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
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
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
        .expect("script should return a string");

    assert_eq!(text, "undefined:3");
}

#[test]
fn evaluate_script_eval_with_statement_updates_empty_abrupt_completion() {
    let unit = compile_test_unit(
        2408,
        r"
            [
                String(eval('1; do { 2; with({}) { 3; break; } 4; } while (false);')),
                String(eval('5; do { 6; with({}) { break; } 7; } while (false);')),
                String(eval('8; do { 9; with({}) { 10; continue; } 11; } while (false)')),
                String(eval('12; do { 13; with({}) { continue; } 14; } while (false)'))
            ].join(':');
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
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
        r"
            var x = 1;
            var object = { x: 2 };
            object[Symbol.unscopables] = { x: true };
            with (object) {
                x;
            }
        ",
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
        r"
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
        ",
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
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
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
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
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
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
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
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
        .expect("script should return a string");

    assert_eq!(text, "undefined");
}

#[test]
fn evaluate_script_logical_and_assignment_infers_identifier_function_name() {
    let unit = compile_test_unit(
        2413,
        r"
            var value = 1;
            value &&= function() {};
            value.name;
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = result
        .as_string_ref()
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
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
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
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
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
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
        .and_then(|value| {
            agent
                .heap()
                .view()
                .string_view(value)
                .map(|view| decode_string(&view))
        })
        .expect("script should return a string");

    assert_eq!(text, "2:12");
}

#[test]
fn evaluate_script_with_statement_closure_keeps_object_environment_live() {
    let unit = compile_test_unit(
        2398,
        r"
            var reader;
            with ({ x: 7 }) {
                reader = function() {
                    return x;
                };
            }
            reader() === 7;
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
        .and_then(lyng_js_objects::FunctionObjectData::entry)
    else {
        panic!("function expression should remain backed by installed bytecode");
    };
    let function = vm
        .installed_function(code)
        .expect("function expression bytecode should stay installed");
    let environment = agent
        .objects()
        .function_data(function_object)
        .and_then(lyng_js_objects::FunctionObjectData::environment)
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
        r"
            var reader;
            with ({}) {
                var x = 7;
                reader = function() {
                    return x;
                };
            }
            reader() === 7;
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
fn evaluate_script_with_statement_single_statement_body_closure_keeps_var_binding() {
    let unit = compile_test_unit(
        2399,
        r"
            var probeBody;
            with ({ x: 0 })
                var x = 1, _ = probeBody = function() { return x; };
            var x = 2;
            probeBody() * 10 + x;
        ",
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
        r"
            var x = 0;
            var objectRecord = { x: 2 };
            var probeBefore = function() { return x; };
            var probeExpr, probeBody;

            with (eval('var x = 1;'), probeExpr = function() { return x; }, objectRecord)
                var x = 3, _ = probeBody = function() { return x; };

            objectRecord.x * 10000 + x * 1000 + probeBody() * 100 + probeExpr() * 10 + probeBefore();
        ",
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

use super::support::compile_and_run_string;

#[test]
fn direct_eval_creates_local_var_binding_visible_after_eval() {
    let result = compile_and_run_string(
        r#"
        function f() {
            eval("var created = 'ok'");
            return created;
        }
        f();
        "#,
    );

    assert_eq!(result, "ok");
}

#[test]
fn direct_eval_can_create_arguments_binding_in_non_arrow_function() {
    let result = compile_and_run_string(
        r#"
            function f() {
            eval("var arguments = 'param'");
            return arguments;
        }
        f();
        "#,
    );

    assert_eq!(result, "param");
}

#[test]
fn direct_eval_can_create_arguments_binding_visible_in_arrow_after_default_parameter_eval() {
    let result = compile_and_run_string(
        r#"
            const f = (p = eval("var arguments = 'param'")) => arguments;
            String(f());
        "#,
    );

    assert_eq!(result, "param");
}

#[test]
fn direct_eval_during_parameter_initialization_rejects_var_arguments_in_non_arrow_function() {
    let result = compile_and_run_string(
        r#"
            function f(p = eval("var arguments")) {}
            let status = "missing";
            try {
                f();
                status = "ok";
            } catch (error) {
                status = error.constructor === SyntaxError ? "syntax" : "other";
            }
            status;
        "#,
    );

    assert_eq!(result, "syntax");
}

#[test]
fn direct_eval_during_parameter_initialization_rejects_var_arguments_when_arrow_parameter_exists() {
    let result = compile_and_run_string(
        r#"
            const f = (arguments, p = eval("var arguments = 'param'")) => {};
            let status = "missing";
            try {
                f();
                status = "ok";
            } catch (error) {
                status = error.constructor === SyntaxError ? "syntax" : "other";
            }
            status;
        "#,
    );

    assert_eq!(result, "syntax");
}

#[test]
fn direct_eval_preserves_eval_created_arguments_for_arrow_created_after_default_parameter_eval() {
    let result = compile_and_run_string(
        r#"
            let seen = "missing";
            const f = (p = eval("var arguments = 'param'"), q = () => arguments) => {
                var arguments = "local";
                seen = String(arguments) + ":" + String(q());
            };
            f();
            seen;
        "#,
    );

    assert_eq!(result, "local:param");
}

#[test]
fn direct_eval_in_global_code_rejects_var_collisions_with_global_lexicals() {
    let result = compile_and_run_string(
        r#"
            let x;
            let status = "missing";
            try {
                eval("var x;");
                status = "ok";
            } catch (error) {
                status = error.constructor === SyntaxError ? "syntax" : "other";
            }
            status;
        "#,
    );

    assert_eq!(result, "syntax");
}

#[test]
fn direct_eval_in_global_code_checks_function_definability_before_execution() {
    let result = compile_and_run_string(
        r#"
            let status = "missing";
            try {
                eval("var shouldNotBeDefined; function NaN(){}");
                status = "ok";
            } catch (error) {
                status =
                    (error.constructor === TypeError ? "type" : "other")
                    + ":"
                    + String(typeof shouldNotBeDefined);
            }
            status;
        "#,
    );

    assert_eq!(result, "type:undefined");
}

#[test]
fn direct_eval_can_delete_new_local_var_binding() {
    let result = compile_and_run_string(
        r#"
            let initial = "missing";
            let deleted = "missing";
            let postDeletion = "missing";
            (function() {
                eval([
                    "initial = String(x);",
                    "deleted = String(delete x);",
                    "try {",
                    "  x;",
                    "  postDeletion = 'bound';",
                    "} catch (error) {",
                    "  postDeletion = error.constructor === ReferenceError ? 'reference' : 'other';",
                    "}",
                    "var x;",
                ].join("\n"));
            }());
            initial + ":" + deleted + ":" + postDeletion;
        "#,
    );

    assert_eq!(result, "undefined:true:reference");
}

#[test]
fn direct_eval_can_delete_new_local_function_binding() {
    let result = compile_and_run_string(
        r#"
            let initial = "missing";
            let deleted = "missing";
            let postDeletion = "missing";
            (function() {
                eval([
                    "initial = typeof f + ':' + String(f());",
                    "deleted = String(delete f);",
                    "try {",
                    "  f;",
                    "  postDeletion = 'bound';",
                    "} catch (error) {",
                    "  postDeletion = error.constructor === ReferenceError ? 'reference' : 'other';",
                    "}",
                    "function f() { return 33; }",
                ].join("\n"));
            }());
            initial + ":" + deleted + ":" + postDeletion;
        "#,
    );

    assert_eq!(result, "function:33:true:reference");
}

#[test]
fn direct_eval_exceptions_are_catchable_by_surrounding_try_statements() {
    let result = compile_and_run_string(
        r#"
            let status = "start";
            try {
                eval("missing");
                status = "not-caught";
            } catch {
                status = "caught";
            }
            status;
        "#,
    );

    assert_eq!(result, "caught");
}

#[test]
fn direct_eval_in_strict_caller_keeps_var_bindings_local() {
    let result = compile_and_run_string(
        r#"
            "use strict";
            eval("var x = 7;");
            let status = "unset";
            try {
                x = 9;
                status = "leaked";
            } catch {
                status = "hidden";
            }
            status;
        "#,
    );

    assert_eq!(result, "hidden");
}

#[test]
fn direct_eval_in_strict_caller_keeps_function_declarations_local() {
    let result = compile_and_run_string(
        r#"
            function testcase() {
                "use strict";
                eval("function fun(x) { return x; }");
                return typeof fun;
            }
            testcase();
        "#,
    );

    assert_eq!(result, "undefined");
}

#[test]
fn direct_eval_in_strict_caller_parses_source_as_strict_code() {
    let result = compile_and_run_string(
        r#"
            function testcase() {
                "use strict";
                try {
                    eval("public = 1;");
                    return "ok";
                } catch (error) {
                    return error.constructor === SyntaxError ? "syntax" : "other";
                }
            }
            testcase();
        "#,
    );

    assert_eq!(result, "syntax");
}

#[test]
fn direct_eval_in_global_code_creates_var_binding_before_initializer_reads() {
    let result = compile_and_run_string(
        r#"
            var initial = null;
            eval("initial = x; var x;");
            let desc = Object.getOwnPropertyDescriptor(globalThis, "x");
            String(initial) + ":" + String(desc && desc.value) + ":" + String(desc && desc.configurable);
        "#,
    );

    assert_eq!(result, "undefined:undefined:true");
}

#[test]
fn direct_eval_in_only_strict_script_can_still_call_global_helpers() {
    let result = compile_and_run_string(
        r#"
            "use strict";
            function helper(value) {
                return value;
            }
            function testcase() {
                eval("function fun(x) { return x; }");
                return helper(typeof fun);
            }
            testcase();
        "#,
    );

    assert_eq!(result, "undefined");
}

#[test]
fn direct_eval_in_only_strict_script_matches_test262_assert_shape() {
    let result = compile_and_run_string(
        r#"
            "use strict";
            function Test262Error(message) {
                this.message = message || "";
            }
            function assert(mustBeTrue, message) {
                if (mustBeTrue === true) {
                    return;
                }
                throw new Test262Error(message || "assertion failed");
            }
            assert._isSameValue = function(a, b) {
                return a === b;
            };
            assert.sameValue = function(actual, expected, message) {
                if (assert._isSameValue(actual, expected)) {
                    return;
                }
                throw new Test262Error(message || "sameValue failed");
            };
            function testcase() {
                eval("function fun(x){ return x }");
                assert.sameValue(typeof (fun), "undefined");
                return "pass";
            }
            testcase();
        "#,
    );

    assert_eq!(result, "pass");
}

#[test]
fn direct_eval_with_strict_source_keeps_outer_var_binding_visible_after_eval() {
    let result = compile_and_run_string(
        r#"
            function testcase() {
                var outer = 0;
                function inner() {
                    eval("'use strict'; var outer = 1;");
                    return String(outer);
                }
                return inner();
            }
            testcase();
        "#,
    );

    assert_eq!(result, "0");
}

#[test]
fn direct_eval_in_strict_caller_keeps_outer_var_binding_visible_after_eval() {
    let result = compile_and_run_string(
        r#"
            "use strict";
            function testcase() {
                var outer = 0;
                function inner() {
                    eval("var outer = 1;");
                    return String(outer);
                }
                return inner();
            }
            testcase();
        "#,
    );

    assert_eq!(result, "0");
}

#[test]
fn direct_eval_in_non_strict_inner_function_hosts_var_without_aliasing_outer_capture() {
    let result = compile_and_run_string(
        r#"
            function testcase() {
                var outer = 0;
                function inner() {
                    var before = function() {
                        return outer;
                    };
                    eval("var outer = 1;");
                    return String(before()) + ":" + String(outer);
                }
                return inner();
            }
            testcase();
        "#,
    );

    assert_eq!(result, "0:1");
}

#[test]
fn direct_eval_in_strict_caller_does_not_overwrite_global_var_binding() {
    let result = compile_and_run_string(
        r#"
            "use strict";
            var outer = 0;
            function testcase() {
                eval("var outer = 1;");
                return String(outer);
            }
            testcase();
        "#,
    );

    assert_eq!(result, "0");
}

#[test]
fn direct_eval_in_strict_function_uses_caller_this_binding() {
    let result = compile_and_run_string(
        r#"
            "use strict";
            String((function() {
                return eval("this;");
            }()) === undefined);
        "#,
    );

    assert_eq!(result, "true");
}

#[test]
fn direct_eval_can_observe_new_target_from_non_arrow_function() {
    let result = compile_and_run_string(
        r#"
            let seen = "missing";
            function getNewTarget() {
                seen = eval("new.target;");
            }
            getNewTarget();
            let plain = String(seen === undefined);
            new getNewTarget();
            plain + ":" + String(seen === getNewTarget);
        "#,
    );

    assert_eq!(result, "true:true");
}

#[test]
fn direct_eval_can_read_super_property_from_method() {
    let result = compile_and_run_string(
        r#"
            let seen = "missing";
            const o = {
                method() {
                    seen = String(eval("super.test262;"));
                },
            };
            o.method();
            const first = seen;
            Object.setPrototypeOf(o, { test262: 262 });
            o.method();
            first + ":" + seen;
        "#,
    );

    assert_eq!(result, "undefined:262");
}

#[test]
fn direct_eval_can_read_private_field_from_class_method() {
    let result = compile_and_run_string(
        r#"
            class C {
                #m = 44;

                getWithEval() {
                    return String(eval("this.#m"));
                }
            }

            new C().getWithEval();
        "#,
    );

    assert_eq!(result, "44");
}

#[test]
fn direct_eval_can_call_private_method_from_class_method() {
    let result = compile_and_run_string(
        r#"
            class C {
                #m() {
                    return "Test262";
                }

                getWithEval() {
                    return eval("this.#m()");
                }
            }

            new C().getWithEval();
        "#,
    );

    assert_eq!(result, "Test262");
}

#[test]
fn direct_eval_can_read_private_static_accessor_from_static_method() {
    let result = compile_and_run_string(
        r#"
            class C {
                static get #m() {
                    return "Test262";
                }

                static getWithEval() {
                    return eval("this.#m");
                }
            }

            C.getWithEval();
        "#,
    );

    assert_eq!(result, "Test262");
}

#[test]
fn direct_eval_in_class_field_initializer_rejects_arguments_before_side_effects() {
    let result = compile_and_run_string(
        r#"
            let executed = false;
            class C {
                x = eval("executed = true; arguments;");
            }

            try {
                new C();
                "ok";
            } catch (error) {
                error.constructor.name + ":" + String(executed);
            }
        "#,
    );

    assert_eq!(result, "SyntaxError:false");
}

#[test]
fn direct_eval_in_global_code_updates_configurable_function_binding_attributes() {
    let result = compile_and_run_string(
        r#"
            let initial = null;
            Object.defineProperty(globalThis, "f", {
                enumerable: false,
                writable: false,
                configurable: true,
            });
            eval("initial = f; function f() { return 345; }");
            let desc = Object.getOwnPropertyDescriptor(globalThis, "f");
            typeof initial
                + ":"
                + String(initial())
                + ":"
                + String(desc.writable)
                + ":"
                + String(desc.enumerable)
                + ":"
                + String(desc.configurable);
        "#,
    );

    assert_eq!(result, "function:345:true:true:true");
}

#[test]
fn direct_eval_uses_innermost_block_lexical_binding() {
    let result = compile_and_run_string(
        r#"
            let nonStrict = "missing";
            let strict = "missing";
            let x = "outside";
            {
                let x = "inside";
                nonStrict = eval("x;");
                strict = eval("'use strict'; x;");
            }
            nonStrict + ":" + strict;
        "#,
    );

    assert_eq!(result, "inside:inside");
}

#[test]
fn direct_eval_root_lexical_declarations_create_tdz_before_initialization() {
    let result = compile_and_run_string(
        r#"
            function classify(source) {
                try {
                    eval(source);
                    return "ok";
                } catch (error) {
                    return error.constructor === ReferenceError ? "reference" : "other";
                }
            }
            [
                classify("typeof x; let x;"),
                classify("typeof y; const y = null;"),
                classify("typeof C; class C {}"),
            ].join(":");
        "#,
    );

    assert_eq!(result, "reference:reference:reference");
}

#[test]
fn direct_eval_annex_b_block_function_updates_global_var_binding() {
    let result = compile_and_run_string(
        r#"
            var initial = "missing";
            var current = "missing";

            eval("{ function f() { initial = f; f = 123; current = f; return 'decl'; } }");
            let first = f();
            let initialCall = initial();
            let globalCall = f();

            first + ":" + initialCall + ":" + String(current) + ":" + globalCall;
        "#,
    );

    assert_eq!(result, "decl:decl:123:decl");
}

#[test]
fn direct_eval_typeof_skips_future_annex_b_for_head_lexical_shadow() {
    let result = compile_and_run_string(
        r#"
            function run() {
                return eval([
                    "let status = 'missing';",
                    "try { f; status = 'bound'; } catch (error) {",
                    "  status = error.constructor === ReferenceError ? 'reference' : 'other';",
                    "}",
                    "let before = typeof f;",
                    "for (let f; ; ) {",
                    "  { function f() {} }",
                    "  break;",
                    "}",
                    "let after = typeof f;",
                    "status + ':' + before + ':' + after;",
                ].join("\n"));
            }
            run();
        "#,
    );

    assert_eq!(result, "reference:undefined:undefined");
}

#[test]
fn indirect_eval_annex_b_block_function_updates_global_var_binding() {
    let result = compile_and_run_string(
        r#"
            var initial = "missing";
            var current = "missing";

            (0, eval)("{ function f() { initial = f; f = 123; current = f; return 'decl'; } }");
            let first = f();
            let initialCall = initial();
            let globalCall = f();

            first + ":" + initialCall + ":" + String(current) + ":" + globalCall;
        "#,
    );

    assert_eq!(result, "decl:decl:123:decl");
}

#[test]
fn direct_eval_allows_var_redeclaration_of_simple_catch_parameter() {
    let result = compile_and_run_string(
        r#"
            let status = "missing";
            try {
                throw "caught";
            } catch (err) {
                status = "";
                try { eval("function err() {}"); status += "ok:"; }
                catch (error) { status += error.constructor.name + ":"; }
                try { eval("function* err() {}"); status += "ok:"; }
                catch (error) { status += error.constructor.name + ":"; }
                try { eval("async function err() {}"); status += "ok:"; }
                catch (error) { status += error.constructor.name + ":"; }
                try { eval("async function* err() {}"); status += "ok:"; }
                catch (error) { status += error.constructor.name + ":"; }
                try { eval("var err;"); status += "ok:"; }
                catch (error) { status += error.constructor.name + ":"; }
                try { eval("for (var err; false; ) {}"); status += "ok:"; }
                catch (error) { status += error.constructor.name + ":"; }
                try { eval("for (var err in []) {}"); status += "ok:"; }
                catch (error) { status += error.constructor.name + ":"; }
                try { eval("for (var err of []) {}"); status += "ok:"; }
                catch (error) { status += error.constructor.name + ":"; }
                status += typeof err + ":" + err;
            }
            status;
        "#,
    );

    assert_eq!(result, "ok:ok:ok:ok:ok:ok:ok:ok:string:caught");
}

#[test]
fn direct_eval_in_nested_block_rejects_var_collision_with_enclosing_lexical() {
    let result = compile_and_run_string(
        r#"
            let status = "missing";
            {
                let x;
                {
                    try {
                        eval("var x;");
                        status = "ok";
                    } catch (error) {
                        status = error.constructor === SyntaxError ? "syntax" : "other";
                    }
                }
            }
            status;
        "#,
    );

    assert_eq!(result, "syntax");
}

#[test]
fn indirect_eval_creates_distinct_lexical_environment_for_root_let_bindings() {
    let result = compile_and_run_string(
        r#"
            let outside = 23;
            let status = "missing";
            (0, eval)("let outside;");
            (0, eval)("let x = 3;");
            try {
                x;
                status = "bound";
            } catch (error) {
                status = error.constructor === ReferenceError ? "reference" : "other";
            }
            String(outside) + ":" + status;
        "#,
    );

    assert_eq!(result, "23:reference");
}

#[test]
fn indirect_eval_root_lexical_declarations_create_tdz_before_initialization() {
    let result = compile_and_run_string(
        r#"
            function classify(source) {
                try {
                    (0, eval)(source);
                    return "ok";
                } catch (error) {
                    return error.constructor === ReferenceError ? "reference" : "other";
                }
            }
            [
                classify("typeof x; let x;"),
                classify("typeof y; const y = null;"),
                classify("typeof C; class C {}"),
            ].join(":");
        "#,
    );

    assert_eq!(result, "reference:reference:reference");
}

#[test]
fn strict_indirect_eval_does_not_leak_var_bindings_to_global_scope() {
    let result = compile_and_run_string(
        r#"
            let before = "foo" in globalThis;
            if (!before) {
                (0, eval)("'use strict'; var foo = 88;");
            }
            String(before) + ":" + String("foo" in globalThis);
        "#,
    );

    assert_eq!(result, "false:false");
}

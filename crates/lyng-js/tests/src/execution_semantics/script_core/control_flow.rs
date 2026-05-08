use super::super::support::*;

#[test]
fn script_core_executes_locals_globals_objects_and_arrays() {
    let result = compile_and_run(
        r"
        let x = 1;
        let y = 2;
        var z = 3;
        let obj = { answer: x + y };
        let arr = [obj.answer, z];
        let i = 0;
        let sum = 0;
        while (i < 2) {
            sum = sum + arr[i];
            i = i + 1;
        }
        sum;
        ",
    );

    assert_eq!(result, Value::from_smi(6));
}

#[test]
fn script_core_executes_if_blocks_and_local_assignment() {
    let result = compile_and_run(
        r"
        let value = 1;
        if (value < 2) {
            value = value + 4;
        } else {
            value = 99;
        }
        value;
        ",
    );

    assert_eq!(result, Value::from_smi(5));
}

#[test]
fn script_core_frame_local_lexicals_throw_in_tdz() {
    let result = compile_and_run_string(
        r#"
        let log = [];

        try {
            { value; let value; }
        } catch (error) {
            log.push(error.constructor === ReferenceError ? "block-prior" : "block-other");
        }

        try {
            { let value = value + 1; }
        } catch (error) {
            log.push(error.constructor === ReferenceError ? "block-init" : "block-init-other");
        }

        try {
            (function() { value; const value = 1; }());
        } catch (error) {
            log.push(error.constructor === ReferenceError ? "function-prior" : "function-other");
        }

        try {
            (function() { const value = value + 1; }());
        } catch (error) {
            log.push(error.constructor === ReferenceError ? "function-init" : "function-init-other");
        }

        log.join("|");
        "#,
    );

    assert_eq!(
        result,
        "block-prior|block-init|function-prior|function-init"
    );
}

#[test]
fn script_core_instanceof_uses_symbol_has_instance_protocol() {
    let result = compile_and_run_string(
        r#"
        let log = [];

        try {
            true instanceof true;
            log.push("missing");
        } catch (error) {
            log.push(error instanceof TypeError ? "type" : error.name);
        }

        let rhs = {};
        rhs[Symbol.hasInstance] = function(value) {
            log.push(this === rhs ? "this" : "bad-this");
            log.push(value === 0 ? "arg" : "bad-arg");
            return "truthy";
        };
        log.push(0 instanceof rhs ? "truthy" : "falsy");

        rhs[Symbol.hasInstance] = null;
        try {
            0 instanceof rhs;
            log.push("missing-null");
        } catch (error) {
            log.push(error instanceof TypeError ? "null" : error.name);
        }

        log.join("|");
        "#,
    );

    assert_eq!(result, "type|this|arg|truthy|null");
}

#[test]
fn script_core_executes_for_loops_with_local_register_updates() {
    let result = compile_and_run(
        r"
        let total = 0;
        for (let i = 0; i < 4; i = i + 1) {
            total = total + i;
        }
        total;
        ",
    );

    assert_eq!(result, Value::from_smi(6));
}

#[test]
fn script_core_executes_sibling_lexical_for_loops_with_duplicate_names() {
    let result = compile_and_run(
        r"
        let total = 0;
        for (let i = 0; i < 3; ++i) {
            total = total + 1;
        }
        for (let i = 0; i < 2; ++i) {
            total = total + 10;
        }
        total;
        ",
    );

    assert_eq!(result, Value::from_smi(23));
}

#[test]
fn script_core_supports_switch_fallthrough_and_break() {
    let result = compile_and_run(
        r"
        let total = 0;
        switch (2) {
            case 1:
                total = 100;
                break;
            case 2:
                total = total + 2;
            default:
                total = total + 4;
                break;
        }
        total;
        ",
    );

    assert_eq!(result, Value::from_smi(6));
}

#[test]
fn script_core_if_statement_completion_uses_a_fresh_undefined_seed() {
    let result = compile_and_run_string(
        r"
        [
            String(eval('1; if (false) { }')),
            String(eval('2; do { 3; if (true) { 4; break; } 5; } while (false)')),
            String(eval('6; do { 7; if (false) { 8; } else { break; } } while (false)'))
        ].join(':');
        ",
    );

    assert_eq!(result, "undefined:4:undefined");
}

#[test]
fn script_core_eval_fast_paths_repeated_empty_blocks() {
    let result = compile_and_run_string(
        r#"
        var source = "{}";
        for (var i = 0; i < 16; i = i + 1) {
            source += source;
        }

        var indirectEval = eval;
        [
            String(eval(source)),
            String(indirectEval(source))
        ].join(":");
        "#,
    );

    assert_eq!(result, "undefined:undefined");
}

#[test]
fn script_core_loop_statement_completion_updates_empty_abrupt_exits() {
    let result = compile_and_run_string(
        r"
        [
            String(eval('1; while (true) { break; }')),
            String(eval('2; while (true) { 3; break; }')),
            String(eval('4; do { continue; } while (false)')),
            String(eval('5; do { 6; continue; } while (false)'))
        ].join(':');
        ",
    );

    assert_eq!(result, "undefined:3:undefined:6");
}

#[test]
fn script_core_debugger_statement_preserves_empty_completion() {
    let result = compile_and_run_string(
        r"
        [
            String(eval('1; debugger;')),
            String(eval('2; while (false) debugger;'))
        ].join(':');
        ",
    );

    assert_eq!(result, "1:undefined");
}

#[test]
fn script_core_for_loop_completion_uses_undefined_seed() {
    let result = compile_and_run_string(
        r"
        [
            String(eval('1; for (; false; ) { }')),
            String(eval('2; for (let i = 0; i < 1; i = i + 1) { }')),
            String(eval('3; for (let i = 0; i < 1; i = i + 1) { 4; }')),
            String(eval('var a; 5; for (a in { x: 0 }) { }')),
            String(eval('var b; 6; for (b in { x: 0 }) { 7; break; }')),
            String(eval('var c; 8; for (c of [0]) { }')),
            String(eval('var d; 9; for (d of [0]) { 10; break; }')),
            String(eval('var e; 11; outer: do { for (e of [0]) { continue outer; } } while (false)')),
            String(eval('var f; 12; outer: do { for (f of [0]) { 13; continue outer; } } while (false)'))
        ].join(':');
        ",
    );

    assert_eq!(
        result,
        "undefined:undefined:4:undefined:7:undefined:10:undefined:13"
    );
}

#[test]
fn script_core_for_of_iterator_close_reports_non_throw_close_errors() {
    let result = compile_and_run_string(
        r#"
        let log = [];
        function run(iterable, classify) {
            try {
                for (let value of iterable) {
                    log.push("body");
                    break;
                }
                log.push("missing");
            } catch (error) {
                log.push(classify(error));
            }
        }

        run({
            [Symbol.iterator]() {
                return {
                    next() { return { done: false, value: 0 }; },
                    return() { return 0; }
                };
            }
        }, error => error instanceof TypeError ? "return-object" : error.name);

        let marker = {};
        run({
            [Symbol.iterator]() {
                return {
                    next() { return { done: false, value: 0 }; },
                    get return() { throw marker; }
                };
            }
        }, error => error === marker ? "getter" : error.name);

        run({
            [Symbol.iterator]() {
                return {
                    next() { return { done: false, value: 0 }; },
                    return: 1
                };
            }
        }, error => error instanceof TypeError ? "callable" : error.name);

        log.join("|");
        "#,
    );

    assert_eq!(result, "body|return-object|body|getter|body|callable");
}

#[test]
fn script_core_switch_statement_matches_completion_and_lexical_open_rules() {
    let completion_result = compile_and_run_string(
        r#"
        [
            String(eval('1; switch ("a") { default: }')),
            String(eval('2; switch ("a") { default: 3; }')),
            String(eval('4; switch ("b") { case "a": 5; default: }')),
            String(eval('6; switch ("b") { case "a": 7; default: 8; }'))
        ].join(':');
        "#,
    );

    assert_eq!(completion_result, "undefined:3:undefined:8");
}

#[test]
fn script_core_switch_statement_opens_its_lexical_scope_for_case_evaluation_only() {
    let result = compile_and_run_string(
        r"
        let x = 'outside';
        var probeExpr, probeSelector, probeStmt;

        switch (probeExpr = function() { return x; }, null) {
            case probeSelector = function() { return x; }, null:
                probeStmt = function() { return x; };
                let x = 'inside';
        }

        [probeExpr(), probeSelector(), probeStmt()].join(':');
        ",
    );

    assert_eq!(result, "outside:inside:inside");
}

#[test]
fn script_core_try_statement_completion_preserves_the_pre_finally_value() {
    let result = compile_and_run_string(
        r"
        [
            String(eval('1; try { } catch (err) { }')),
            String(eval('2; try { 3; } finally { 4; }')),
            String(eval('5; try { throw null; } catch (err) { 6; } finally { 7; }')),
            String(eval('8; do { try { 9; break; } finally { 10; } } while (false)')),
            String(eval('11; do { try { break; } finally { 12; } } while (false)'))
        ].join(':');
        ",
    );

    assert_eq!(result, "undefined:3:6:9:undefined");
}

#[test]
fn script_core_supports_destructured_catch_bindings() {
    let result = compile_and_run_string(
        r#"
        let result = "";
        try {
            throw ["left", "right"];
        } catch ([left, right]) {
            result = left + ":" + right;
        }
        result;
        "#,
    );

    assert_eq!(result, "left:right");
}

#[test]
fn script_core_supports_for_of_over_arrays() {
    let result = compile_and_run_string(
        r#"
        let s = "";
        for (const x of [1, 2, 3]) {
            s = s + x;
        }
        s;
        "#,
    );

    assert_eq!(result, "123");
}

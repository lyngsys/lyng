use super::support::{
    compile_and_run, compile_and_run_string, compile_and_run_with_globals, compile_unit,
    install_native_global,
};
use lyng_js_common::AtomTable;
use lyng_js_types::{
    internal_array_index_of_builtin, internal_array_pop_builtin, internal_array_push_builtin,
    internal_object_has_own_property_builtin, internal_object_to_string_builtin,
    internal_string_index_of_builtin, internal_string_replace_builtin, Value,
};

#[test]
fn script_core_executes_locals_globals_objects_and_arrays() {
    let result = compile_and_run(
        r#"
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
        "#,
    );

    assert_eq!(result, Value::from_smi(6));
}

#[test]
fn script_core_executes_if_blocks_and_local_assignment() {
    let result = compile_and_run(
        r#"
        let value = 1;
        if (value < 2) {
            value = value + 4;
        } else {
            value = 99;
        }
        value;
        "#,
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
        r#"
        let total = 0;
        for (let i = 0; i < 4; i = i + 1) {
            total = total + i;
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(6));
}

#[test]
fn script_core_executes_sibling_lexical_for_loops_with_duplicate_names() {
    let result = compile_and_run(
        r#"
        let total = 0;
        for (let i = 0; i < 3; ++i) {
            total = total + 1;
        }
        for (let i = 0; i < 2; ++i) {
            total = total + 10;
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(23));
}

#[test]
fn script_core_supports_switch_fallthrough_and_break() {
    let result = compile_and_run(
        r#"
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
        "#,
    );

    assert_eq!(result, Value::from_smi(6));
}

#[test]
fn script_core_if_statement_completion_uses_a_fresh_undefined_seed() {
    let result = compile_and_run_string(
        r#"
        [
            String(eval('1; if (false) { }')),
            String(eval('2; do { 3; if (true) { 4; break; } 5; } while (false)')),
            String(eval('6; do { 7; if (false) { 8; } else { break; } } while (false)'))
        ].join(':');
        "#,
    );

    assert_eq!(result, "undefined:4:undefined");
}

#[test]
fn script_core_loop_statement_completion_updates_empty_abrupt_exits() {
    let result = compile_and_run_string(
        r#"
        [
            String(eval('1; while (true) { break; }')),
            String(eval('2; while (true) { 3; break; }')),
            String(eval('4; do { continue; } while (false)')),
            String(eval('5; do { 6; continue; } while (false)'))
        ].join(':');
        "#,
    );

    assert_eq!(result, "undefined:3:undefined:6");
}

#[test]
fn script_core_debugger_statement_preserves_empty_completion() {
    let result = compile_and_run_string(
        r#"
        [
            String(eval('1; debugger;')),
            String(eval('2; while (false) debugger;'))
        ].join(':');
        "#,
    );

    assert_eq!(result, "1:undefined");
}

#[test]
fn script_core_for_loop_completion_uses_undefined_seed() {
    let result = compile_and_run_string(
        r#"
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
        "#,
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
        r#"
        let x = 'outside';
        var probeExpr, probeSelector, probeStmt;

        switch (probeExpr = function() { return x; }, null) {
            case probeSelector = function() { return x; }, null:
                probeStmt = function() { return x; };
                let x = 'inside';
        }

        [probeExpr(), probeSelector(), probeStmt()].join(':');
        "#,
    );

    assert_eq!(result, "outside:inside:inside");
}

#[test]
fn script_core_try_statement_completion_preserves_the_pre_finally_value() {
    let result = compile_and_run_string(
        r#"
        [
            String(eval('1; try { } catch (err) { }')),
            String(eval('2; try { 3; } finally { 4; }')),
            String(eval('5; try { throw null; } catch (err) { 6; } finally { 7; }')),
            String(eval('8; do { try { 9; break; } finally { 10; } } while (false)')),
            String(eval('11; do { try { break; } finally { 12; } } while (false)'))
        ].join(':');
        "#,
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

#[test]
fn phase6_string_text_builtins_cover_missing_core_methods() {
    let result = compile_and_run_string(
        r#"
        [
            String.prototype.trim.call(true),
            " \tLyng\n".trim() + ":" + "  x  ".trimStart().trimEnd(),
            "abcdef".includes("cd", 2) + ":" + "abcdef".indexOf("cd", 3) + ":" + "abcdef".endsWith("de", 5),
            "AbC".toLowerCase() + ":" + "ab\u00df".toUpperCase(),
            "abc".at(-1) + ":" + "\uD83D\uDE00".codePointAt(0) + ":" + String.fromCodePoint(0x41, 0x1F600),
            "\uD800".isWellFormed() + ":" + "\uD800".toWellFormed(),
            String.raw({ raw: ["a", "b", "c"] }, 1, 2),
            "a$a".replaceAll("a", "$$&") + ":" + "aaa".replaceAll("a", function(match, pos) { return pos; }),
            "a".localeCompare("b") < 0,
            "\u00E9".normalize("NFD").length
        ].join("|");
        "#,
    );

    assert_eq!(
        result,
        "true|Lyng:x|true:-1:true|abc:ABSS|c:128512:A😀|false:�|a1b2c|$&$$&:012|true|2"
    );
}

#[test]
fn phase6_uri_builtins_handle_utf16_surrogates() {
    let result = compile_and_run(
        r#"
        let score = 0;
        score += encodeURI("\uD83D\uDE00") === "%F0%9F%98%80" ? 1 : 0;
        score += encodeURIComponent("\uD83D\uDE00") === "%F0%9F%98%80" ? 2 : 0;
        score += decodeURI("%F0%9F%98%80") === "\uD83D\uDE00" ? 4 : 0;
        score += decodeURIComponent("%F0%9F%98%80") === "\uD83D\uDE00" ? 8 : 0;
        try {
            encodeURI("\uD800");
        } catch (error) {
            score += Object.getPrototypeOf(error) === URIError.prototype ? 16 : 0;
        }
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn phase6_string_symbol_hooks_delegate_and_reject_regexp_searches() {
    let result = compile_and_run_string(
        r#"
        let trace = [];

        let matcher = {};
        matcher[Symbol.match] = function(value) {
            trace.push(this === matcher, value);
            return "match";
        };

        let replacer = {};
        replacer[Symbol.replace] = function(value, replacement) {
            trace.push(this === replacer, value, replacement);
            return "replace";
        };

        let replaceAller = {};
        replaceAller[Symbol.replace] = function(value, replacement) {
            trace.push(this === replaceAller, value, replacement);
            return "replaceAll";
        };

        let searcher = {};
        searcher[Symbol.search] = function(value) {
            trace.push(this === searcher, value);
            return 7;
        };

        let splitter = {};
        splitter[Symbol.split] = function(value, limit) {
            trace.push(this === splitter, value, limit);
            return "split";
        };

        let abrupt = {};
        Object.defineProperty(abrupt, Symbol.match, {
            get: function() {
                throw "match-get";
            }
        });

        let abruptResult;
        try {
            "".includes(abrupt);
        } catch (error) {
            abruptResult = error;
        }

        let regexpRejected = false;
        try {
            "a".startsWith(/a/);
        } catch (error) {
            regexpRejected = error instanceof TypeError;
        }

        let originalMatch = RegExp.prototype[Symbol.match];
        RegExp.prototype[Symbol.match] = function(value) {
            return this instanceof RegExp && this.source === "a+" && value === "aa"
                ? "regexp-match"
                : "bad";
        };
        let internalMatch = "aa".match("a+");
        RegExp.prototype[Symbol.match] = originalMatch;

        [
            "abc".match(matcher),
            "abc".replace(replacer, "x"),
            "abc".replaceAll(replaceAller, "y"),
            "abc".search(searcher),
            "abc".split(splitter, 2),
            abruptResult,
            regexpRejected,
            internalMatch,
            trace.join(",")
        ].join("|");
        "#,
    );

    assert_eq!(
        result,
        "match|replace|replaceAll|7|split|match-get|true|regexp-match|true,abc,true,abc,x,true,abc,y,true,abc,true,abc,2"
    );
}

#[test]
fn phase6_string_split_handles_regexp_edges() {
    let result = compile_and_run_string(
        r#"
        function show(value) {
            return value.join(",") + ":" + value.length;
        }

        [
            show("x".split(/^/)),
            show("x".split(/$/)),
            show("x".split(/.?/)),
            show("x".split(/.*/)),
            show("x".split(/.*?/)),
            show("x".split(/./)),
            show("x".split(/(?:)/)),
            new String("hello").split(new RegExp()).join("")
        ].join("|");
        "#,
    );

    assert_eq!(result, "x:1|x:1|,:2|,:2|x:1|,:2|x:1|hello");
}

#[test]
fn phase6_string_match_all_returns_match_iterator() {
    let result = compile_and_run_string(
        r#"
        let iterator = "a,b,c".matchAll(",");
        let first = iterator.next().value;
        let second = iterator.next().value;
        let matches = [first, second];
        [
            matches.length,
            matches[0][0],
            matches[0].index,
            matches[0].input,
            matches[1][0],
            matches[1].index,
            matches[1].input
        ].join("|");
        "#,
    );

    assert_eq!(result, "2|,|1|a,b,c|,|3|a,b,c");
}

#[test]
fn phase6_string_match_all_empty_pattern_visits_each_boundary() {
    let result = compile_and_run_string(
        r#"
        let iterator = "a".matchAll(undefined);
        let first = iterator.next();
        let second = iterator.next();
        let third = iterator.next();
        function show(result) {
            if (result.done) {
                return "done";
            }
            return result.value[0].length + ":" + result.value.index;
        }
        [
            show(first),
            show(second),
            third.done
        ].join("|");
        "#,
    );

    assert_eq!(result, "0:0|0:1|true");
}

#[test]
fn phase6_regexp_match_all_iterator_uses_late_bound_exec() {
    let result = compile_and_run_string(
        r#"
        let iterator = /./g[Symbol.matchAll]("abc");
        let calls = 0;
        RegExp.prototype.exec = function(input) {
            calls = calls + 1;
            if (calls === 1) {
                return ["xy"];
            }
            return null;
        };
        let first = iterator.next();
        let second = iterator.next();
        [
            first.value[0],
            first.done,
            String(second.value),
            second.done,
            calls
        ].join("|");
        "#,
    );

    assert_eq!(result, "xy|false|undefined|true|2");
}

#[test]
fn phase6_regexp_search_uses_custom_exec_and_restores_last_index() {
    let result = compile_and_run_string(
        r#"
        let lastIndexValue = 7;
        let reads = 0;
        let writes = 0;
        let duringExec = -1;
        let fake = {
            get lastIndex() {
                reads = reads + 1;
                return lastIndexValue;
            },
            set lastIndex(value) {
                writes = writes + 1;
                lastIndexValue = value;
            },
            exec: function(input) {
                duringExec = this.lastIndex;
                this.lastIndex = 11;
                return { index: 86 };
            }
        };
        let result = RegExp.prototype[Symbol.search].call(fake, "abc");
        [
            result,
            duringExec,
            lastIndexValue,
            reads,
            writes
        ].join("|");
        "#,
    );

    assert_eq!(result, "86|0|7|3|3");
}

#[test]
fn phase6_regexp_match_uses_flags_and_custom_exec() {
    let result = compile_and_run_string(
        r#"
        let calls = 0;
        let fake = {
            flags: "g",
            lastIndex: 0,
            exec: function(input) {
                calls = calls + 1;
                if (calls === 1) {
                    return [""];
                }
                return null;
            }
        };
        let matched = RegExp.prototype[Symbol.match].call(fake, "abc");
        [
            matched.length,
            matched[0],
            fake.lastIndex,
            calls
        ].join("|");
        "#,
    );

    assert_eq!(result, "1||1|2");
}

#[test]
fn phase6_regexp_replace_uses_custom_exec_result_properties() {
    let result = compile_and_run_string(
        r#"
        let calls = 0;
        let fake = {
            flags: "g",
            lastIndex: 0,
            exec: function(input) {
                calls = calls + 1;
                if (calls === 1) {
                    return {
                        length: 2,
                        0: "b",
                        1: "B",
                        index: 1,
                        groups: { name: "group" }
                    };
                }
                return null;
            }
        };
        RegExp.prototype[Symbol.replace].call(fake, "abc", function(match, capture, index, input, groups) {
            return match + capture + index + input + groups.name;
        });
        "#,
    );

    assert_eq!(result, "abB1abcgroupc");
}

#[test]
fn phase6_regexp_split_limit_zero_bails_before_matching() {
    let result = compile_and_run_string(
        r#"
        let constructed = false;
        let fake = {
            constructor: function() {}
        };
        fake.constructor[Symbol.species] = function() {
            constructed = true;
            return {
                exec: function() {
                    throw new Error("exec should not run");
                }
            };
        };
        let split = RegExp.prototype[Symbol.split].call(fake, "abc", 0);
        [constructed, Array.isArray(split), split.length].join("|");
        "#,
    );

    assert_eq!(result, "true|true|0");
}

#[test]
fn phase6_regexp_split_uses_species_sticky_exec_and_captures() {
    let result = compile_and_run_string(
        r#"
        let rx = /x/i;
        let sawPattern = false;
        let sawFlags = "";
        let calls = 0;
        rx.constructor = function() {};
        rx.constructor[Symbol.species] = function(pattern, flags) {
            sawPattern = pattern === rx;
            sawFlags = flags;
            return {
                lastIndex: 0,
                exec: function(input) {
                    calls = calls + 1;
                    if (this.lastIndex === 1) {
                        this.lastIndex = 2;
                        return { length: 2, 0: "b", 1: "B" };
                    }
                    return null;
                }
            };
        };
        let split = RegExp.prototype[Symbol.split].call(rx, "abc", 5);
        [
            sawPattern,
            sawFlags,
            calls,
            split.length,
            split[0],
            split[1],
            split[2]
        ].join("|");
        "#,
    );

    assert_eq!(result, "true|iy|3|3|a|B|c");
}

#[test]
fn script_core_regexp_split_constructs_splitter_before_coercing_limit() {
    let result = compile_and_run_string(
        r#"
        let regExp = /a/;
        let limit = {
            valueOf: function() {
                regExp.compile("b");
                return -1;
            }
        };
        let split = regExp[Symbol.split]("abba", limit);
        [split.length, split[0], split[1], split[2]].join("|");
        "#,
    );

    assert_eq!(result, "3||bb|");
}

#[test]
fn phase6_string_edge_cases_cover_remaining_text_failures() {
    let result = compile_and_run_string(
        r#"
        let booleanWrapper = new Boolean();
        booleanWrapper.slice = String.prototype.slice;
        booleanWrapper.substring = String.prototype.substring;

        let symbolConstructed = "no-throw";
        try {
            new String(Symbol("66"));
        } catch (error) {
            symbolConstructed = error.constructor === TypeError;
        }

        [
            "ABBABAB".lastIndexOf({ toString: function() { return "AB"; } }, { valueOf: function() { return NaN; } }),
            ["new", "zoo", "revue"].lastIndexOf("zoo"),
            booleanWrapper.slice(true, undefined),
            String(void 0).slice("e", undefined),
            String({ toString: function() {} }).substring(-4, undefined),
            String(Symbol("66")),
            symbolConstructed,
            "A\u03A3".toLowerCase(),
            "A\u03A3B".toLowerCase(),
            "\u00C5\u2ADC\u0958\u2126\u0344".normalize(),
            "\u1E9B\u0323".normalize("NFKC"),
            "o\u0308".localeCompare("\u00F6")
        ].join("|");
        "#,
    );

    assert_eq!(
        result,
        "5|1|alse|undefined|undefined|Symbol(66)|true|aς|aσb|Å⫝̸क़Ω̈́|ṩ|0"
    );
}

#[test]
fn script_core_supports_annex_b_string_substr() {
    let result = compile_and_run_string(
        r#"
        let method = String.prototype.substr;
        let nullThis = "no-throw";
        try {
            method.call(null, 0, 1);
        } catch (error) {
            nullThis = error.constructor === TypeError;
        }

        [
            typeof method,
            method.length,
            method.name,
            "abcdef".substr(1, 3),
            "abcdef".substr(-2),
            "abcdef".substr(-20, 2),
            "abcdef".substr(2, 0),
            "abcdef".substr(2, -1),
            "abcdef".substr(2, undefined),
            "abcdef".substr(NaN, Infinity),
            "abcdef".substr(-Infinity, 1),
            "abcdef".substr(Infinity, 1),
            method.call(new Boolean(true), 1, 2),
            nullThis
        ].join("|");
        "#,
    );

    assert_eq!(
        result,
        "function|2|substr|bcd|ef|ab|||cdef|abcdef|a||ru|true"
    );
}

#[test]
fn script_core_object_literal_infers_anonymous_property_function_names() {
    let result = compile_and_run_string(
        r#"
        let anon = Symbol();
        let named = Symbol("test262");
        let object = {
            id: () => {},
            [anon]: () => {},
            [named]: function() {},
            method() {},
            [String(named) + "Method"]() {}
        };
        [
            object.id.name,
            object[anon].name,
            object[named].name,
            object.method.name,
            object["Symbol(test262)Method"].name
        ].join("|");
        "#,
    );

    assert_eq!(result, "id||[test262]|method|Symbol(test262)Method");
}

#[test]
fn script_core_supports_for_of_destructuring_assignment_heads() {
    let result = compile_and_run_string(
        r#"
        let left = "";
        let right = "";
        for ([left, right] of [["x", "y"]]) {}
        left + right;
        "#,
    );

    assert_eq!(result, "xy");
}

#[test]
fn script_core_closes_assignment_pattern_iterators_on_generator_return() {
    let result = compile_and_run(
        r#"
        let returnCount = 0;
        let iterator = {
            next: function() {
                return { done: false, value: undefined };
            },
            return: function() {
                returnCount = returnCount + 1;
                return {};
            }
        };
        let iterable = {};
        iterable[Symbol.iterator] = function() {
            return iterator;
        };

        function* probe() {
            [ {} = yield ] = iterable;
        }

        let iter = probe();
        iter.next();
        iter.return(7);
        returnCount;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_does_not_require_iterator_next_before_assignment_reference_suspends() {
    let result = compile_and_run(
        r#"
        let returnCount = 0;
        let iterator = {
            return: function() {
                returnCount = returnCount + 1;
                return {};
            }
        };
        let iterable = {};
        iterable[Symbol.iterator] = function() {
            return iterator;
        };

        function* probe() {
            [ {}[yield] ] = iterable;
        }

        let iter = probe();
        iter.next();
        iter.return(9);
        returnCount;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_evaluates_array_rest_assignment_reference_before_iterating() {
    let result = compile_and_run_string(
        r#"
        let nextCount = 0;
        let returnCount = 0;
        let iterator = {
            next: function() {
                nextCount = nextCount + 1;
                return { done: true };
            },
            return: function() {
                returnCount = returnCount + 1;
                return {};
            }
        };
        let iterable = {};
        iterable[Symbol.iterator] = function() {
            return iterator;
        };
        function thrower() {
            throw "sentinel";
        }

        let caught = "";
        try {
            [...{}[thrower()]] = iterable;
        } catch (error) {
            caught = error;
        }
        caught + ":" + nextCount + ":" + returnCount;
        "#,
    );

    assert_eq!(result, "sentinel:0:1");
}

#[test]
fn script_core_supports_for_of_destructuring_assignment_to_properties() {
    let result = compile_and_run(
        r#"
        let target = { value: 0 };
        for ({ value: target.value } of [{ value: 7 }]) {}
        target.value;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn script_core_supports_for_of_declaration_defaults_in_destructuring_heads() {
    let result = compile_and_run_string(
        r#"
        let seen = "";
        for (let [x = "a", y = "z"] of [[undefined, "b"]]) {
            seen = x + y;
        }
        seen;
        "#,
    );

    assert_eq!(result, "ab");
}

#[test]
fn script_core_supports_for_of_lexical_computed_property_bindings() {
    let result = compile_and_run(
        r#"
        let key = "value";
        let seen = 0;
        for (let { [key]: entry } of [{ value: 7 }]) {
            seen = entry;
        }
        seen;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn script_core_coerces_object_property_keys_before_array_index_classification() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let values = [];

        values[new Boolean(true)] = 3;
        score += (values[1] === undefined ? 1 : 0);
        score += (values["true"] === 3 ? 2 : 0);

        let key = {
            valueOf: function() { return 1; },
            toString: function() { return 0; }
        };
        values[key] = 7;
        score += (values[0] === 7 ? 4 : 0);

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn script_core_handles_object_property_key_to_primitive_edge_cases() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let values = [];

        let key = {
            valueOf: function() {
                return 1;
            }
        };
        values[key] = 0;
        score += (values["[object Object]"] === 0 ? 1 : 0);

        let keyed = {
            valueOf: function() {
                return 1;
            },
            toString: function() {
                return 0;
            }
        };
        values[keyed] = 7;
        score += (values[0] === 7 ? 2 : 0);

        try {
            let source = [];
            let dynamic = {
                valueOf: function() {
                    throw "error";
                },
                toString: function() {
                    return 1;
                }
            };
            source[dynamic] = 9;
            score += (source[1] === 9 ? 4 : 0);
        } catch (error) {
            score += 64;
        }

        try {
            let source = [];
            let dynamic = {
                valueOf: function() {
                    return 1;
                },
                toString: function() {
                    throw "error";
                }
            };
            source[dynamic];
            score += 128;
        } catch (error) {
            score += (error === "error" ? 8 : 0);
        }

        try {
            let source = [];
            let dynamic = {
                valueOf: function() {
                    return {};
                },
                toString: function() {
                    return {};
                }
            };
            source[dynamic];
            score += 256;
        } catch (error) {
            score += (error instanceof TypeError ? 16 : 0);
        }

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_computed_assignment_defers_reference_validation_until_put_value() {
    let result = compile_and_run(
        r#"
        let total = 0;
        let order = "";
        let assigned = {};
        let assignmentKey = {
            toString: function() {
                order += "key";
                return "value";
            }
        };
        function rhs() {
            order += "rhs";
            return 3;
        }
        assigned[assignmentKey] = rhs();
        total += (order === "rhskey" && assigned.value === 3 ? 1 : 0);

        try {
            let base = null;
            let key = {
                toString: function() {
                    order += "bad-key";
                    return "value";
                }
            };
            base[key] = rhs();
        } catch (error) {
            total += (error instanceof TypeError && order === "rhskeyrhs" ? 4 : 0);
        }

        let compoundKeyCount = 0;
        let compound = { value: 5 };
        let compoundKey = {
            toString: function() {
                compoundKeyCount += 1;
                return "value";
            }
        };
        compound[compoundKey] ^= 3;
        total += (compoundKeyCount === 1 && compound.value === 6 ? 2 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn script_core_destructuring_assignment_prepares_property_target_before_source_get() {
    let result = compile_and_run_string(
        r#"
        let log = [];

        function source() {
            log.push("source");
            return {
                get p() {
                    log.push("get");
                }
            };
        }
        function target() {
            log.push("target");
            return {
                set q(v) {
                    log.push("set");
                }
            };
        }
        function sourceKey() {
            log.push("source-key");
            return {
                toString: function() {
                    log.push("source-key-tostring");
                    return "p";
                }
            };
        }
        function targetKey() {
            log.push("target-key");
            return {
                toString: function() {
                    log.push("target-key-tostring");
                    return "q";
                }
            };
        }

        ({[sourceKey()]: target()[targetKey()]} = source());
        log.join("|");
        "#,
    );

    assert_eq!(
        result,
        "source|source-key|source-key-tostring|target|target-key|get|target-key-tostring|set"
    );
}

#[test]
fn script_core_parenthesized_assignment_target_does_not_infer_function_name() {
    let result = compile_and_run_string(
        r#"
        var fn;
        (fn) = function() {};
        fn.name;
        "#,
    );

    assert_eq!(result, "");
}

#[test]
fn script_core_strict_compound_global_reference_observes_deleted_binding() {
    let result = compile_and_run(
        r#"
        var count = 0;
        Object.defineProperty(this, "x", {
            configurable: true,
            get: function() {
                delete this.x;
                return 2;
            }
        });

        (function() {
            "use strict";
            try {
                count++;
                x ^= 3;
                count += 100;
            } catch (error) {
                count += (error.constructor === ReferenceError ? 1 : 10);
            }
        })();

        count + (!("x" in this) ? 10 : 1000);
        "#,
    );

    assert_eq!(result, Value::from_smi(12));
}

#[test]
fn script_core_supports_for_of_var_binding_pattern_expressions() {
    let result = compile_and_run(
        r#"
        let key = "value";
        var seen = 0;
        for (var { [key]: fn = function() { return 7; } } of [{}]) {
            seen = fn();
        }
        seen;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn script_core_supports_sloppy_for_of_destructuring_with_eval_and_arguments_targets() {
    let result = compile_and_run(
        r#"
        var eval, arguments;
        let score = 0;
        for ({ eval = 3, arguments = 4 } of [{}]) {
            score += (eval === 3 ? 1 : 0);
            score += (arguments === 4 ? 2 : 0);
        }
        for ({ eval, arguments } of [{ eval: 5, arguments: 6 }]) {
            score += (eval === 5 ? 4 : 0);
            score += (arguments === 6 ? 8 : 0);
        }
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_supports_for_of_object_rest_without_exclusions() {
    let result = compile_and_run(
        r#"
        let sym = Symbol("foo");
        let calls = "";
        let source = {};
        Object.defineProperty(source, 1, {
            get: function() {
                calls = calls + "1|";
                return 3;
            },
            enumerable: true
        });
        Object.defineProperty(source, "z", {
            get: function() {
                calls = calls + "z|";
                return 1;
            },
            enumerable: true
        });
        Object.defineProperty(source, "a", {
            get: function() {
                calls = calls + "a|";
                return 2;
            },
            enumerable: true
        });
        Object.defineProperty(source, sym, {
            get: function() {
                calls = calls + "sym|";
                return 4;
            },
            enumerable: true
        });

        var rest;
        for ({ ...rest } of [source]) {}

        let score = 0;
        score += (rest[1] === 3 ? 1 : 0);
        score += (rest.z === 1 ? 2 : 0);
        score += (rest.a === 2 ? 4 : 0);
        score += (rest[sym] === 4 ? 8 : 0);
        score += (calls === "1|z|a|sym|" ? 16 : 0);
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_supports_for_of_object_rest_with_symbol_keys_and_exclusions() {
    let result = compile_and_run(
        r#"
        let sym = Symbol("foo");
        let calls = "";
        let source = {};
        Object.defineProperty(source, "skip", {
            get: function() {
                calls = calls + "skip|";
                return 9;
            },
            enumerable: true
        });
        Object.defineProperty(source, 1, {
            get: function() {
                calls = calls + "1|";
                return 3;
            },
            enumerable: true
        });
        Object.defineProperty(source, "keep", {
            get: function() {
                calls = calls + "keep|";
                return 2;
            },
            enumerable: true
        });
        Object.defineProperty(source, sym, {
            get: function() {
                calls = calls + "sym|";
                return 4;
            },
            enumerable: true
        });

        let skip = 0;
        let rest = null;
        for ({ skip, ...rest } of [source]) {}

        let score = 0;
        score += (skip === 9 ? 1 : 0);
        score += (rest[1] === 3 ? 2 : 0);
        score += (rest.keep === 2 ? 4 : 0);
        score += (rest.skip === undefined ? 8 : 0);
        score += (rest[sym] === 4 ? 16 : 0);
        score += (calls === "skip|1|keep|sym|" ? 32 : 0);
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn script_core_supports_for_of_binding_object_rest_copy_semantics() {
    let result = compile_and_run(
        r#"
        let getterCount = 0;
        let source = { a: 5, b: 3, x: 1, y: 2 };
        Object.defineProperty(source, "hidden", {
            value: 9,
            enumerable: false
        });
        Object.defineProperty(source, "v", {
            get: function() {
                getterCount = getterCount + 1;
                return 7;
            },
            enumerable: true
        });

        let seen = null;
        for (const { a, b, ...rest } of [source]) {
            seen = rest;
        }

        let xDesc = Object.getOwnPropertyDescriptor(seen, "x");
        let vDesc = Object.getOwnPropertyDescriptor(seen, "v");
        let score = 0;
        score += (seen.a === undefined ? 1 : 0);
        score += (seen.b === undefined ? 2 : 0);
        score += (seen.hidden === undefined ? 4 : 0);
        score += (seen.x === 1 ? 8 : 0);
        score += (seen.y === 2 ? 16 : 0);
        score += (seen.v === 7 ? 32 : 0);
        score += (getterCount === 1 ? 64 : 0);
        score += (xDesc.enumerable === true && xDesc.writable === true && xDesc.configurable === true ? 128 : 0);
        score += (vDesc.enumerable === true && vDesc.writable === true && vDesc.configurable === true ? 256 : 0);
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(511));
}

#[test]
fn script_core_object_keys_skips_non_enumerable_and_symbol_properties() {
    let result = compile_and_run_string(
        r#"
        let sym = Symbol("hidden");
        let source = { a: 1 };
        source[2] = 4;
        source[sym] = 8;
        Object.defineProperty(source, "secret", {
            value: 16,
            enumerable: false
        });
        Object.keys(source).toString();
        "#,
    );

    assert_eq!(result, "2,a");
}

#[test]
fn script_core_object_get_own_property_names_includes_non_enumerable_strings() {
    let result = compile_and_run_string(
        r#"
        let source = "ab";
        let names = Object.getOwnPropertyNames(source);
        names.toString();
        "#,
    );

    assert_eq!(result, "0,1,length");
}

#[test]
fn script_core_assigns_names_to_for_of_default_anonymous_functions() {
    let result = compile_and_run_string(
        r#"
        let seen = "";
        for (let [arrow = () => {}] of [[]]) {
            seen = arrow.name;
        }
        seen;
        "#,
    );

    assert_eq!(result, "arrow");
}

#[test]
fn script_core_assigns_names_to_for_of_assignment_pattern_defaults() {
    let result = compile_and_run(
        r#"
        let arrayArrow;
        let arrayCover;
        let objArrow;
        let objFn;
        let propArrow;
        let named;
        let score = 0;

        for ([arrayArrow = () => {}, arrayCover = (function() {})] of [[]]) {
            let arrowDesc = Object.getOwnPropertyDescriptor(arrayArrow, "name");
            let coverDesc = Object.getOwnPropertyDescriptor(arrayCover, "name");
            score += (arrowDesc.value === "arrayArrow" ? 1 : 0);
            score += (arrowDesc.enumerable === false && arrowDesc.writable === false && arrowDesc.configurable === true ? 2 : 0);
            score += (coverDesc.value === "arrayCover" ? 4 : 0);
        }

        for ({ objArrow = () => {}, objFn = function() {}, x: propArrow = () => {}, named = function named() {} } of [{}]) {
            score += (objArrow.name === "objArrow" ? 8 : 0);
            score += (objFn.name === "objFn" ? 16 : 0);
            score += (propArrow.name === "propArrow" ? 32 : 0);
            score += (named.name === "named" ? 64 : 0);
        }

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn script_core_consumes_iterables_for_for_of_array_binding_patterns() {
    let result = compile_and_run(
        r#"
        let iterator = {
            step: 0,
            next: function() {
                this.step = this.step + 1;
                if (this.step === 1) {
                    return { value: 4, done: false };
                }
                if (this.step === 2) {
                    return { value: 9, done: false };
                }
                return { value: undefined, done: true };
            }
        };
        let iterable = {
            [Symbol.iterator]: function() {
                return iterator;
            }
        };
        let seen = 0;
        for (let [a, b] of [iterable]) {
            seen = a + b;
        }
        seen;
        "#,
    );

    assert_eq!(result, Value::from_smi(13));
}

#[test]
fn script_core_hides_iterator_internal_state_and_requires_real_iterator_receivers() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let arrayIterator = [1].values();
        let stringIterator = "ab"[Symbol.iterator]();

        score += (arrayIterator.__arrayIteratorTarget === undefined ? 1 : 0);
        score += (arrayIterator.__arrayIteratorIndex === undefined ? 2 : 0);
        score += (arrayIterator.__arrayIteratorKind === undefined ? 4 : 0);
        score += (stringIterator.__stringIteratorString === undefined ? 8 : 0);
        score += (stringIterator.__stringIteratorIndex === undefined ? 16 : 0);

        let arrayNext = arrayIterator.next;
        let stringNext = stringIterator.next;
        try {
            arrayNext.call({});
        } catch (error) {
            score += (error.constructor === TypeError ? 32 : 0);
        }
        try {
            stringNext.call({});
        } catch (error) {
            score += (error.constructor === TypeError ? 64 : 0);
        }

        score += (arrayIterator.next().value === 1 ? 128 : 0);
        score += (stringIterator.next().value === "a" ? 256 : 0);
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(511));
}

#[test]
fn script_core_closes_for_of_empty_array_assignment_patterns() {
    let result = compile_and_run(
        r#"
        let closed = 0;
        let iterator = {
            next: function() {
                return { value: 1, done: false };
            },
            return: function() {
                closed = closed + 1;
                return {};
            }
        };
        let iterable = {
            [Symbol.iterator]: function() {
                return iterator;
            }
        };
        for ([] of [iterable]) {}
        closed;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_supports_for_of_object_shorthand_defaults() {
    let result = compile_and_run(
        r#"
        var x = 0;
        for ({ x = 1 } of [{}]) {}
        x;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_supports_for_of_object_rest_assignment_heads() {
    let result = compile_and_run(
        r#"
        let rest = null;
        for ({ value: ignored, ...rest } of [{ value: 1, other: 2 }]) {}
        rest.other;
        "#,
    );

    assert_eq!(result, Value::from_smi(2));
}

#[test]
fn script_core_supports_for_of_object_rest_assignment_same_name_targets() {
    let result = compile_and_run(
        r#"
        let source = { x: 42, y: 39, z: "cheeseburger" };
        let x = 0;
        let y = "unset";
        let z = null;
        for ({ x, ...z } of [source]) {}
        (x === 42 ? 1 : 0)
            + (y === "unset" ? 2 : 0)
            + (z !== null ? 4 : 0)
            + (z !== null && z.y === 39 ? 8 : 0)
            + (z !== null && z.z === "cheeseburger" ? 16 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_supports_for_of_array_rest_assignment_heads() {
    let result = compile_and_run(
        r#"
        let x = 0;
        let y = null;
        for ([x, ...y] of [[1, 2, 3]]) {}
        (x === 1 ? 1 : 0)
            + (y !== null ? 2 : 0)
            + (y !== null && Object.prototype.toString.call(y) === "[object Array]" ? 4 : 0)
            + (y !== null && y.length === 2 ? 8 : 0)
            + (y !== null && y[0] === 2 ? 16 : 0)
            + (y !== null && y[1] === 3 ? 32 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn script_core_requires_object_coercible_values_for_for_of_object_patterns() {
    let result = compile_and_run(
        r#"
        let score = 0;
        try {
            for ({} of [null]) {
                score += 100;
            }
        } catch (error) {
            score += (error.constructor === TypeError ? 1 : 0);
        }
        try {
            for (const {} of [undefined]) {
                score += 100;
            }
        } catch (error) {
            score += (error.constructor === TypeError ? 2 : 0);
        }
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_requires_object_coercible_values_for_for_of_object_rest() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let rest;
        try {
            for ({ ...rest } of [null]) {
                score += 100;
            }
        } catch (error) {
            score += (error.constructor === TypeError ? 1 : 0);
        }
        try {
            for ({ ...rest } of [undefined]) {
                score += 100;
            }
        } catch (error) {
            score += (error.constructor === TypeError ? 2 : 0);
        }
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_for_of_destructuring_assignments_respect_global_lexical_tdz() {
    let result = compile_and_run(
        r#"
        let score = 0;
        try {
            for ({ x } of [{}]) {
                score += 100;
            }
        } catch (error) {
            score += (error.constructor === ReferenceError ? 1 : 0);
        }
        let x;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_for_of_destructuring_assignments_reject_strict_unresolvables() {
    let result = compile_and_run(
        r#"
        "use strict";
        let score = 0;
        try {
            for ([...unresolvable] of [[]]) {
                score += 100;
            }
        } catch (error) {
            score += (error.constructor === ReferenceError ? 1 : 0);
        }
        try {
            for ({ unbound } of [{}]) {
                score += 100;
            }
        } catch (error) {
            score += (error.constructor === ReferenceError ? 2 : 0);
        }
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_supports_string_keyed_computed_property_access() {
    let result = compile_and_run(
        r#"
        let obj = { answer: 1 };
        obj["answer"] = obj["answer"] + 2;
        obj.answer;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_supports_named_property_store_and_load() {
    let result = compile_and_run(
        r#"
        let obj = { total: 0 };
        obj.total = obj.total + 6;
        obj.total;
        "#,
    );

    assert_eq!(result, Value::from_smi(6));
}

#[test]
fn script_core_supports_repeated_named_property_updates() {
    let result = compile_and_run(
        r#"
        let obj = { total: 0 };
        obj.total = obj.total + 1;
        obj.total = obj.total + 2;
        obj.total;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_supports_string_addition_with_primitives() {
    let result = compile_and_run_string(
        r#"
        "foo " + 5 + " bar";
        "#,
    );

    assert_eq!(result, "foo 5 bar");
}

#[test]
fn script_core_symbol_for_uses_shared_object_aware_string_coercion() {
    let result = compile_and_run(
        r#"
        Symbol.keyFor(Symbol.for({
            toString: function() {
                return "phase5.shared";
            }
        })) === "phase5.shared" ? 1 : 0;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_symbol_description_reflects_symbol_for_string_key() {
    let result = compile_and_run_string(
        r#"
        Symbol.for({
            toString: function() {
                return "phase5.description";
            }
        }).description;
        "#,
    );

    assert_eq!(result, "phase5.description");
}

#[test]
fn script_core_symbol_for_preserves_thrown_constructor_identity() {
    let result = compile_and_run(
        r#"
        function Test262Error() {}
        let total = 0;
        try {
            Symbol.for({
                toString: function() {
                    throw new Test262Error();
                }
            });
        } catch (error) {
            total = total + (error.constructor === Test262Error ? 1 : 0);
        }
        try {
            Symbol.for(Symbol("s"));
        } catch (error) {
            total = total + (error.constructor === TypeError ? 2 : 0);
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_installs_date_parse_and_uri_global_families() {
    let result = compile_and_run(
        r#"
        (typeof Date === "function" ? 1 : 0)
            + (typeof parseInt === "function" ? 2 : 0)
            + (typeof parseFloat === "function" ? 4 : 0)
            + (typeof isNaN === "function" ? 8 : 0)
            + (typeof isFinite === "function" ? 16 : 0)
            + (typeof encodeURI === "function" ? 32 : 0)
            + (typeof encodeURIComponent === "function" ? 64 : 0)
            + (typeof decodeURI === "function" ? 128 : 0)
            + (typeof decodeURIComponent === "function" ? 256 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(511));
}

#[test]
fn script_core_date_objects_expose_value_payload_and_object_tag() {
    let result = compile_and_run(
        r#"
        let d = new Date(123);
        (d.valueOf() === 123 ? 1 : 0)
            + (Object.prototype.toString.call(d) === "[object Date]" ? 2 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_date_constructor_balances_local_components_and_timezone_offset() {
    let result = compile_and_run(
        r#"
        let epoch = new Date(1970, 0, 1, 0, 0, 0, 0);
        let negative = new Date(1970, 0, 1, 0, 0, 0, -1);
        let later = new Date(2016, 3, 12, 13, 21, 23, 24);
        let total = 0;
        total += (typeof Date.prototype.getTimezoneOffset === "function" ? 1 : 0);
        total += (epoch.valueOf() - epoch.getTimezoneOffset() * 60000 === 0 ? 2 : 0);
        total += (negative.valueOf() - negative.getTimezoneOffset() * 60000 === -1 ? 4 : 0);
        total += (later.valueOf() - later.getTimezoneOffset() * 60000 === 1460467283024 ? 8 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_date_constructor_has_function_prototype_shape() {
    let result = compile_and_run(
        r#"
        (Function.prototype.isPrototypeOf(Date) ? 1 : 0)
            + (Object.getPrototypeOf(Date) === Function.prototype ? 2 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_supports_date_conformance_edges() {
    let result = compile_and_run(
        r#"
        let total = 0;
        try {
            Date.prototype.getTime();
        } catch (error) {
            total += (error instanceof TypeError ? 1 : 0);
        }

        let offset = new Date(0).getTimezoneOffset();
        total += (1 / offset === Infinity ? 2 : 0);
        total += (Date.parse("1970-01-01T00:00:00") === offset * 60000 ? 4 : 0);
        total += (Date.parse("1970-01-01") === 0 ? 8 : 0);

        let date = new Date(2016, 6, 7, 11, 36, 23, 2);
        let expected = new Date(-1, 42, 7, 11, 36, 23, 2).getTime();
        total += (date.getMilliseconds() === 2 ? 16 : 0);
        total += (date.setFullYear({ valueOf() { return 2; } }) === expected ? 32 : 0);
        total += (date.getMilliseconds() === 2 ? 64 : 0);

        total += (new Date(123).toISOString() === "1970-01-01T00:00:00.123Z" ? 128 : 0);
        total += (new Date(123).toTemporalInstant().epochNanoseconds === 123000000n ? 256 : 0);
        total += (Date.prototype[Symbol.toPrimitive].call({ toString() { return "x"; } }, "default") === "x" ? 512 : 0);
        try {
            Date.prototype[Symbol.toPrimitive].call(new Date(0), new String("number"));
        } catch (error) {
            total += (error instanceof TypeError ? 1024 : 0);
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(2047));
}

#[test]
fn script_core_supports_annex_b_date_legacy_methods() {
    let result = compile_and_run(
        r#"
        let total = 0;

        let getYearDesc = Object.getOwnPropertyDescriptor(Date.prototype, "getYear");
        total += (getYearDesc && getYearDesc.writable && !getYearDesc.enumerable && getYearDesc.configurable ? 1 : 0);
        let setYearDesc = Object.getOwnPropertyDescriptor(Date.prototype, "setYear");
        total += (setYearDesc && setYearDesc.writable && !setYearDesc.enumerable && setYearDesc.configurable ? 2 : 0);
        total += (Date.prototype.getYear.length === 0 ? 4 : 0);
        total += (Date.prototype.setYear.length === 1 ? 8 : 0);

        total += (new Date(1899, 0).getYear() === -1 ? 16 : 0);
        total += (new Date(1900, 0).getYear() === 0 ? 32 : 0);
        total += (new Date(2000, 0).getYear() === 100 ? 64 : 0);
        total += (Number.isNaN(new Date({}).getYear()) ? 128 : 0);

        let date = new Date(1970, 1, 2, 3, 4, 5);
        let expected = new Date(1971, 1, 2, 3, 4, 5).valueOf();
        total += (date.setYear(71) === expected && date.valueOf() === expected ? 256 : 0);

        let relative = new Date(1970, 0);
        relative.setYear(50.999999);
        total += (relative.getFullYear() === 1950 ? 512 : 0);

        let absolute = new Date(1970, 0);
        absolute.setYear(100);
        total += (absolute.getFullYear() === 100 ? 1024 : 0);

        total += (Date.prototype.toGMTString === Date.prototype.toUTCString ? 2048 : 0);

        try {
            Date.prototype.getYear.call({});
        } catch (error) {
            total += (error instanceof TypeError ? 4096 : 0);
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(8191));
}

#[test]
fn script_core_parse_and_uri_globals_match_phase6_baseline_behavior() {
    let result = compile_and_run(
        r#"
        let score = 0;
        score = score + (parseInt("0x10") === 16 ? 1 : 0);
        score = score + (parseInt("-0") === 0 && 1 / parseInt("-0") === -Infinity ? 2 : 0);
        score = score + (parseFloat("1.5x") === 1.5 ? 4 : 0);
        score = score + (isNaN("foo") ? 8 : 0);
        score = score + (isFinite("3") ? 16 : 0);
        score = score + (encodeURIComponent("a b") === "a%20b" ? 32 : 0);
        score = score + (encodeURI("a/b") === "a/b" ? 64 : 0);
        score = score + (decodeURIComponent("a%20b") === "a b" ? 128 : 0);
        score = score + (decodeURI("%2F") === "%2F" ? 256 : 0);
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(511));
}

#[test]
fn script_core_parse_and_numeric_predicate_globals_coerce_booleans() {
    let result = compile_and_run(
        r#"
        let score = 0;
        score = score + (parseInt(true) !== parseInt(true) ? 1 : 0);
        score = score + (parseFloat(false) !== parseFloat(false) ? 2 : 0);
        score = score + (isNaN(true) ? 0 : 4);
        score = score + (isFinite(false) ? 8 : 0);
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_public_numeric_coercion_handles_arrays_symbols_and_radix_objects() {
    let result = compile_and_run(
        r#"
        let score = 0;
        score = score + (parseInt("11", false) === 11 ? 1 : 0);
        score = score + (parseInt("11", true) !== parseInt("11", true) ? 2 : 0);
        score = score + (parseInt("11", "2") === 3 ? 4 : 0);
        score = score + (parseInt("11", {
            valueOf: function() {
                return 2;
            }
        }) === 3 ? 8 : 0);
        try {
            parseInt("11", {
                valueOf: function() {
                    throw "boom";
                },
                toString: function() {
                    return 2;
                }
            });
        } catch (error) {
            score = score + (error === "boom" ? 16 : 0);
        }
        score = score + (isNaN([1]) ? 0 : 32);
        score = score + (isNaN([NaN]) ? 64 : 0);
        score = score + (isFinite([1]) ? 128 : 0);
        score = score + (isFinite([Infinity]) ? 0 : 256);
        try {
            let exotic = {};
            exotic[Symbol.toPrimitive] = function() {
                return Symbol.toPrimitive;
            };
            isNaN(exotic);
        } catch (error) {
            score = score + (error.constructor === TypeError ? 512 : 0);
        }
        try {
            let exotic = {};
            exotic[Symbol.toPrimitive] = function() {
                return Symbol.toPrimitive;
            };
            isFinite(exotic);
        } catch (error) {
            score = score + (error.constructor === TypeError ? 1024 : 0);
        }
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(2047));
}

#[test]
fn script_core_keeps_array_length_non_configurable() {
    let result = compile_and_run(
        r#"
        let array = [1, 2, 3];
        (delete array.length ? 100 : 0) + array.length;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_delete_returns_true_for_undeclared_global_assignments() {
    let result = compile_and_run(
        r#"
        x = 1;
        delete x ? 1 : 0;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_delete_returns_true_for_non_reference_unary_chains() {
    let result = compile_and_run(
        r#"
        (delete +-~!0 ? 1 : 0)
            + (delete typeof 0 ? 2 : 0)
            + (delete delete 0 ? 4 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn script_core_supports_template_literals_with_primitives() {
    let result = compile_and_run_string(
        r#"
        `foo ${5} ${true} ${1n} bar`;
        "#,
    );

    assert_eq!(result, "foo 5 true 1 bar");
}

#[test]
fn script_core_supports_template_literals_with_member_and_call_expressions() {
    let result = compile_and_run_string(
        r#"
        let object = {
            value: 5,
            fn: function() { return "result"; }
        };
        `${object.value} ${object.fn()} bar`;
        "#,
    );

    assert_eq!(result, "5 result bar");
}

#[test]
fn script_core_supports_template_literals_with_object_string_coercion() {
    let result = compile_and_run_string(
        r#"
        let plain = {};
        let custom = {
            toString: function() { return "custom"; }
        };
        `${plain}|${custom}`;
        "#,
    );

    assert_eq!(result, "[object Object]|custom");
}

#[test]
fn script_core_supports_postfix_update_expressions() {
    let result = compile_and_run(
        r#"
        let i = 0;
        let first = i++;
        let second = ++i;
        first * 10 + second * 100 + i;
        "#,
    );

    assert_eq!(result, Value::from_smi(202));
}

#[test]
fn script_core_supports_template_literal_left_to_right_updates() {
    let result = compile_and_run_string(
        r#"
        let i = 0;
        `a${i++}b${i++}c${i++}d`;
        "#,
    );

    assert_eq!(result, "a0b1c2d");
}

#[test]
fn script_core_supports_string_primitive_members_and_for_in() {
    let result = compile_and_run_string(
        r#"
        let keys = "";
        for (var key in "ab") {
            keys = keys + key;
        }
        keys + ":" + "ab".length + ":" + "ab"[1] + ":" + "ab".indexOf("b");
        "#,
    );

    assert_eq!(result, "01:2:b:1");
}

#[test]
fn script_core_handles_delete_on_string_primitives() {
    let result = compile_and_run(
        r#"
        (delete "Test262"[100] ? 1 : 0) + (delete "x"[0] ? 0 : 2);
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_bootstraps_string_constructor_and_exotic_descriptors() {
    let result = compile_and_run_string(
        r#"
        let wrapped = new String("abc");
        let index = Object.getOwnPropertyDescriptor(wrapped, "0");
        let length = Object.getOwnPropertyDescriptor(wrapped, "length");
        let keys = "";
        for (var key in wrapped) {
            keys = keys + key;
        }
        String("abc")
            + ":"
            + wrapped.toString()
            + ":"
            + wrapped.valueOf()
            + ":"
            + keys
            + ":"
            + index.value
            + ":"
            + (index.writable ? 1 : 0)
            + (index.enumerable ? 2 : 0)
            + (index.configurable ? 4 : 0)
            + ":"
            + length.value
            + ":"
            + (length.writable ? 1 : 0)
            + (length.enumerable ? 2 : 0)
            + (length.configurable ? 4 : 0);
        "#,
    );

    assert_eq!(result, "abc:abc:abc:012:a:020:3:000");
}

#[test]
fn script_core_supports_string_char_and_pad_algorithms_on_wrapper_values() {
    let result = compile_and_run(
        r#"
        let padded = "abc".padStart(6, "\uD83D\uDCA9");
        let score = 0;
        score += ("abcd".charAt("   +00200.0000E-0002   ") === "c" ? 1 : 0);
        score += ("abcd".charCodeAt("   +00200.0000E-0002   ") === 99 ? 2 : 0);
        score += ("abc".charAt(-0.99999) === "a" ? 4 : 0);
        score += ("abc".charCodeAt(1.99999) === 98 ? 8 : 0);
        score += (padded.length === 6 ? 16 : 0);
        score += (padded.charCodeAt(0) === 0xD83D ? 32 : 0);
        score += (padded.charCodeAt(1) === 0xDCA9 ? 64 : 0);
        score += (padded.charCodeAt(2) === 0xD83D ? 128 : 0);
        score += ("abc".padEnd(5, 0) === "abc00" ? 256 : 0);
        try {
            "abc".padStart(10, Symbol());
        } catch (error) {
            score += (error.constructor === TypeError ? 512 : 0);
        }
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(1023));
}

#[test]
fn script_core_supports_string_repeat_counts_and_range_errors() {
    let result = compile_and_run_string(
        r#"
        let negativeThrew = false;
        let infiniteThrew = false;
        try {
            "x".repeat(-1);
        } catch (error) {
            negativeThrew = error.constructor === RangeError;
        }
        try {
            "x".repeat(Infinity);
        } catch (error) {
            infiniteThrew = error.constructor === RangeError;
        }
        [
            "ab".repeat(3),
            "x".repeat(0),
            String.prototype.repeat.call(7, 2),
            "xy".repeat(2.9),
            negativeThrew,
            infiniteThrew,
        ].join("|");
        "#,
    );

    assert_eq!(result, "ababab||77|xyxy|true|true");
}

#[test]
fn script_core_preserves_lone_surrogate_string_literals() {
    let result = compile_and_run(
        r#"
        let lone = "\uD83D";
        let score = 0;
        score += (lone.length === 1 ? 1 : 0);
        score += (lone.charCodeAt(0) === 0xD83D ? 2 : 0);
        score += ("\uD83D\uDCA9\uD83D".charCodeAt(2) === 0xD83D ? 4 : 0);
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn script_core_preserves_surrogate_pairs_through_string_concatenation_and_iteration() {
    let result = compile_and_run(
        r#"
        let lo = "\uD834";
        let hi = "\uDF06";
        let pair = lo + hi;
        let string = "a" + pair + "b" + lo + pair + hi + lo;
        let iterator = string[Symbol.iterator]();
        let score = 0;
        let result;

        score += (pair.length === 2 ? 1 : 0);
        score += (pair.charCodeAt(0) === 0xD834 ? 2 : 0);
        score += (pair.charCodeAt(1) === 0xDF06 ? 4 : 0);

        result = iterator.next();
        score += (result.value === "a" && result.done === false ? 8 : 0);
        result = iterator.next();
        score += (result.value === pair && result.done === false ? 16 : 0);
        result = iterator.next();
        score += (result.value === "b" && result.done === false ? 32 : 0);
        result = iterator.next();
        score += (result.value === lo && result.done === false ? 64 : 0);
        result = iterator.next();
        score += (result.value === pair && result.done === false ? 128 : 0);
        result = iterator.next();
        score += (result.value === hi && result.done === false ? 256 : 0);
        result = iterator.next();
        score += (result.value === lo && result.done === false ? 512 : 0);
        result = iterator.next();
        score += (result.value === undefined && result.done === true ? 1024 : 0);

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(2047));
}

#[test]
fn script_core_supports_string_match_and_replace_regexp_basics() {
    let result = compile_and_run(
        r#"
        let grouped = "Boston, MA 02134".match(/([\d]{5})([-\ ]?[\d]{4})?$/);
        let globalDigits = "123456abcde7890".match(/\d{2}/g);
        let replaced = "She sells seashells by the seashore.".replace(/sh/g, "$$sch");
        let score = 0;
        score += (grouped[0] === "02134" ? 1 : 0);
        score += (grouped[1] === "02134" ? 2 : 0);
        score += (grouped[2] === undefined ? 4 : 0);
        score += (grouped.index === "Boston, MA 02134".lastIndexOf("0") ? 8 : 0);
        score += (grouped.input === "Boston, MA 02134" ? 16 : 0);
        score += (globalDigits.length === 5 ? 32 : 0);
        score += (globalDigits[0] === "12" ? 64 : 0);
        score += (globalDigits[4] === "90" ? 128 : 0);
        score += (replaced === "She sells sea$schells by the sea$schore." ? 256 : 0);
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(511));
}

#[test]
fn script_core_supports_string_split_and_observable_coercion_order() {
    let result = compile_and_run(
        r#"
        function ExpectedError() {}
        let score = 0;
        let separator = {
            toString: function() {
                score = score + 1;
                throw new Error("separator");
            }
        };
        let limit = {
            valueOf: function() {
                score = score + 2;
                throw new ExpectedError();
            }
        };
        try {
            "foo".split(separator, limit);
        } catch (error) {
            score += (error.constructor === ExpectedError ? 4 : 0);
        }
        let parts = "one two three".split(" ");
        score += (parts.length === 3 ? 8 : 0);
        score += (parts[0] === "one" ? 16 : 0);
        score += (parts[2] === "three" ? 32 : 0);
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(62));
}

#[test]
fn script_core_string_position_coercion_errors_preserve_type_error_identity() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let bad = Object.create(null);
        try {
            "".charAt(bad);
        } catch (error) {
            score += (error.constructor === TypeError ? 1 : 0);
        }
        try {
            "".charCodeAt(bad);
        } catch (error) {
            score += (error.constructor === TypeError ? 2 : 0);
        }
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_supports_array_index_of_builtin_shim() {
    let result = compile_and_run(
        r#"
        let values = [1, 2, 3];
        values.indexOf(2);
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_array_search_helpers_are_generic() {
    let result = compile_and_run(
        r#"
        let score = 0;
        Math[1] = true;
        Math.length = 2;
        score += Array.prototype.indexOf.call(Math, true) === 1 ? 1 : 0;
        score += Array.prototype.indexOf.call([1, 2, 1], 1, 1) === 2 ? 2 : 0;
        score += Array.prototype.includes.call(true) === false ? 4 : 0;
        score += [0, NaN].includes(NaN) ? 8 : 0;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_array_join_uses_to_length_for_generic_receivers() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let object = { 0: "x", 1: null, 2: "z" };

        score += Array.prototype.join.call(object) === "" ? 1 : 0;
        score += object.length === undefined ? 2 : 0;

        object.length = null;
        score += Array.prototype.join.call(object) === "" ? 4 : 0;
        score += object.length === null ? 8 : 0;

        object.length = 2.8;
        score += Array.prototype.join.call(object, "|") === "x|" ? 16 : 0;
        score += object.length === 2.8 ? 32 : 0;

        let wrappedLength = new Number(3.9);
        object.length = wrappedLength;
        score += Array.prototype.join.call(object, "|") === "x||z" ? 64 : 0;
        score += object.length === wrappedLength ? 128 : 0;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(255));
}

#[test]
fn script_core_array_pop_push_use_length_of_array_like() {
    let result = compile_and_run(
        r#"
        let score = 0;

        let popTarget = { 0: "first" };
        score += Array.prototype.pop.call(popTarget) === undefined ? 1 : 0;
        score += popTarget.length === 0 ? 2 : 0;

        popTarget[1] = "last";
        popTarget.length = 2.8;
        score += Array.prototype.pop.call(popTarget) === "last" ? 4 : 0;
        score += popTarget.length === 1 ? 8 : 0;
        score += popTarget[1] === undefined ? 16 : 0;

        let pushTarget = {};
        let wrappedLength = new Number(1.9);
        pushTarget.length = wrappedLength;
        score += Array.prototype.push.call(pushTarget, "x") === 2 ? 32 : 0;
        score += pushTarget[1] === "x" ? 64 : 0;
        score += pushTarget.length === 2 ? 128 : 0;

        let limitTarget = { length: Infinity };
        score += Array.prototype.push.call(limitTarget) === Number.MAX_SAFE_INTEGER ? 256 : 0;
        score += limitTarget.length === Number.MAX_SAFE_INTEGER ? 512 : 0;

        score += Array.prototype.push.call(true) === 0 ? 1024 : 0;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(2047));
}

#[test]
fn script_core_array_push_observes_inherited_setter_before_length_write() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let array = [];
        let calls = 0;

        Object.defineProperty(Array.prototype, "0", {
            set: function(_value) {
                Object.freeze(array);
                calls++;
            },
            configurable: true
        });

        try {
            array.push(1);
        } catch (error) {
            score += error.constructor === TypeError ? 1 : 0;
        } finally {
            delete Array.prototype[0];
        }

        score += !array.hasOwnProperty("0") ? 2 : 0;
        score += array.length === 0 ? 4 : 0;
        score += calls === 1 ? 8 : 0;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_array_concat_boxes_receiver_and_defines_result_elements() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let boxed = Array.prototype.concat.call(true);
        score += boxed[0] instanceof Boolean ? 1 : 0;

        Object.defineProperty(Array.prototype, "0", {
            value: "inherited",
            writable: false,
            configurable: true
        });

        try {
            let result = Array.prototype.concat.call([101]);
            let descriptor = Object.getOwnPropertyDescriptor(result, "0");
            score += result[0] === 101 ? 2 : 0;
            score += descriptor.writable === true ? 4 : 0;
            score += descriptor.enumerable === true ? 8 : 0;
            score += descriptor.configurable === true ? 16 : 0;
        } finally {
            delete Array.prototype[0];
        }

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_array_result_helpers_define_indices_under_poisoned_array_prototype() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let symbol = Symbol("s");
        let symbolHolder = {};
        symbolHolder[symbol] = 1;

        Object.defineProperty(Array.prototype, "0", {
            value: "inherited",
            writable: false,
            configurable: true
        });

        try {
            let names = Object.getOwnPropertyNames({ value: 1, writable: true });
            let nameDesc = Object.getOwnPropertyDescriptor(names, "0");
            score += names[0] === "value" ? 1 : 0;
            score += nameDesc.writable === true ? 2 : 0;

            let keys = Object.keys({ a: 1 });
            score += keys[0] === "a" ? 4 : 0;

            let values = Object.values({ a: 1 });
            score += values[0] === 1 ? 8 : 0;

            let entries = Object.entries({ a: 1 });
            score += entries[0][0] === "a" ? 16 : 0;
            score += entries[0][1] === 1 ? 32 : 0;

            let symbols = Object.getOwnPropertySymbols(symbolHolder);
            score += symbols[0] === symbol ? 64 : 0;
        } finally {
            delete Array.prototype[0];
        }

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn script_core_array_is_array_observes_array_exotics_and_proxies() {
    let result = compile_and_run(
        r#"
        let score = 0;

        score += Array.isArray(Array.prototype) ? 1 : 0;

        let objectProxy = new Proxy({}, {});
        let arrayProxy = new Proxy([], {});
        let nestedProxy = new Proxy(arrayProxy, {});
        score += Array.isArray(objectProxy) === false ? 2 : 0;
        score += Array.isArray(arrayProxy) === true ? 4 : 0;
        score += Array.isArray(nestedProxy) === true ? 8 : 0;

        let revoked = Proxy.revocable([], {});
        revoked.revoke();
        try {
            Array.isArray(revoked.proxy);
        } catch (error) {
            score += error.constructor === TypeError ? 16 : 0;
        }

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_array_length_define_property_coerces_before_descriptor_validation() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let array = [1, 2];
        let valueOfCalls = 0;
        let length = {
            valueOf: function() {
                valueOfCalls++;
                if (valueOfCalls !== 1) {
                    Object.defineProperty(array, "length", { writable: false });
                }
                return array.length;
            }
        };

        try {
            Object.defineProperty(array, "length", { value: length, writable: true });
        } catch (error) {
            score += error.constructor === TypeError ? 1 : 0;
        }
        score += valueOfCalls === 2 ? 2 : 0;

        array = [1, 2];
        valueOfCalls = 0;
        try {
            score += Reflect.defineProperty(array, "length", { value: length, writable: true }) === false ? 4 : 0;
        } catch (_error) {}
        score += valueOfCalls === 2 ? 8 : 0;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_array_length_set_coerces_before_writable_check() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let array = [1, 2, 3];
        let hints = [];
        let length = {};
        length[Symbol.toPrimitive] = function(hint) {
            hints.push(hint);
            Object.defineProperty(array, "length", { writable: false });
            return 0;
        };

        try {
            (function() {
                "use strict";
                array.length = length;
            })();
        } catch (error) {
            score += error.constructor === TypeError ? 1 : 0;
        }
        score += hints.length === 2 && hints[0] === "number" && hints[1] === "number" ? 2 : 0;

        array = [1, 2, 3];
        hints = [];
        try {
            score += Reflect.set(array, "length", length) === false ? 4 : 0;
        } catch (_error) {}
        score += hints.length === 2 && hints[0] === "number" && hints[1] === "number" ? 8 : 0;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_array_reverse_gets_lower_before_testing_upper() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let array = ["first", "second"];

        Object.defineProperty(array, 0, {
            get: function() {
                array.length = 0;
                return "first";
            },
            configurable: true
        });

        array.reverse();
        score += (0 in array) === false ? 1 : 0;
        score += (1 in array) === true ? 2 : 0;
        score += array[1] === "first" ? 4 : 0;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn script_core_in_operator_rejects_primitive_rhs_with_type_error() {
    let result = compile_and_run(
        r#"
        let values = [true, 1, "text", undefined, null];
        let total = 0;

        for (let value of values) {
            try {
                "toString" in value;
                total = total + 100;
            } catch (error) {
                total = total + (error instanceof TypeError ? 1 : 10);
            }
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(5));
}

#[test]
fn script_core_array_slice_uses_species_and_defines_result_elements() {
    let result = compile_and_run(
        r#"
        let score = 0;

        function Ctor(length) {
            this.lengthSeen = length;
        }

        let array = [10, 20];
        array.constructor = {};
        array.constructor[Symbol.species] = Ctor;

        let result = array.slice(0, 1);
        score += Object.getPrototypeOf(result) === Ctor.prototype ? 1 : 0;
        score += result.lengthSeen === 1 ? 2 : 0;
        score += result[0] === 10 ? 4 : 0;

        Object.defineProperty(Ctor.prototype, "0", {
            value: "inherited",
            writable: false,
            configurable: true
        });

        try {
            result = array.slice(0, 1);
            score += result.hasOwnProperty("0") ? 8 : 0;
            score += result[0] === 10 ? 16 : 0;
        } finally {
            delete Ctor.prototype[0];
        }

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_array_predicate_helpers_are_generic() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let obj = new Date(0);
        obj.length = 2;
        obj[0] = 1;
        obj[1] = 2;
        score += Array.prototype.every.call(obj, function(value, index, receiver) {
            return receiver instanceof Date && value < 2 && index === 0;
        }) === false ? 1 : 0;
        score += Array.prototype.some.call(obj, function(value, index, receiver) {
            return receiver instanceof Date && value === 2 && index === 1;
        }) ? 2 : 0;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_array_generics_observe_resizable_typed_array_oob_state() {
    let result = compile_and_run_string(
        r#"
        let rab = new ArrayBuffer(4, { maxByteLength: 8 });
        let fixed = new Uint8Array(rab, 0, 4);
        fixed[0] = 1;
        fixed[1] = 2;
        fixed[2] = 3;
        fixed[3] = 4;
        let seen = [];
        let every = Array.prototype.every.call(fixed, function(value) {
            seen.push(value);
            if (seen.length === 2) {
                rab.resize(2);
            }
            return true;
        });
        let lengthAfterShrink = fixed.length;
        let atAfterShrink = Array.prototype.at.call(fixed, 0);
        let iteratorThrows = false;
        try {
            Array.from(Array.prototype.keys.call(fixed));
        } catch (error) {
            iteratorThrows = error instanceof TypeError;
        }

        let rab2 = new ArrayBuffer(4, { maxByteLength: 8 });
        let fixed2 = new Uint8Array(rab2, 0, 4);
        let evil = {
            valueOf: function() {
                rab2.resize(2);
                return 9;
            }
        };
        Array.prototype.fill.call(fixed2, evil, 0, 1);
        let live = new Uint8Array(rab2);

        let strictIndex = (function() {
            "use strict";
            let rab3 = new ArrayBuffer(16, { maxByteLength: 32 });
            let floats = new Float32Array(rab3);
            floats[0] = -Infinity;
            floats[1] = -Infinity;
            floats[2] = Infinity;
            floats[3] = Infinity;
            floats[4] = NaN;
            return Array.prototype.indexOf.call(floats, Infinity);
        })();

        let rab4 = new ArrayBuffer(4, { maxByteLength: 8 });
        let tracking = new Uint8Array(rab4);
        let joined = Array.prototype.join.call(tracking, {
            toString: function() {
                rab4.resize(6);
                return ".";
            }
        });

        [
            every,
            seen.join(","),
            lengthAfterShrink,
            String(atAfterShrink),
            iteratorThrows,
            live.length + ":" + live[0] + "," + live[1],
            strictIndex,
            joined
        ].join("|");
        "#,
    );

    assert_eq!(result, "true|1,2|0|undefined|true|2:0,0|2|0.0.0.0");
}

#[test]
fn script_core_array_buffer_resizable_accessors_and_transfer() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let fixed = new ArrayBuffer(4);
        let resizable = new ArrayBuffer(4, { maxByteLength: 8 });

        score += fixed.resizable === false ? 1 : 0;
        score += fixed.maxByteLength === 4 ? 2 : 0;
        score += resizable.resizable === true ? 4 : 0;
        score += resizable.maxByteLength === 8 ? 8 : 0;
        score += fixed.detached === false ? 16 : 0;

        let fixedView = new Uint8Array(fixed);
        fixedView[0] = 7;
        fixedView[3] = 9;
        let moved = fixed.transfer(6);
        let movedView = new Uint8Array(moved);

        score += fixed.detached === true && fixed.byteLength === 0 ? 32 : 0;
        score += moved.byteLength === 6 && moved.maxByteLength === 6 && moved.resizable === false ? 64 : 0;
        score += movedView[0] === 7 && movedView[3] === 9 && movedView[4] === 0 ? 128 : 0;

        let resView = new Uint8Array(resizable);
        resView[0] = 3;
        resView[1] = 5;
        let grown = resizable.transfer(6);
        let grownView = new Uint8Array(grown);

        score += resizable.detached === true ? 256 : 0;
        score += grown.resizable === true && grown.maxByteLength === 8 && grown.byteLength === 6 ? 512 : 0;
        score += grownView[0] === 3 && grownView[1] === 5 && grownView[5] === 0 ? 1024 : 0;

        let fixedAgain = grown.transferToFixedLength(2);
        score += grown.detached === true ? 2048 : 0;
        score += fixedAgain.resizable === false && fixedAgain.maxByteLength === 2 && fixedAgain.byteLength === 2 ? 4096 : 0;
        score += new ArrayBuffer(0, null).resizable === false ? 8192 : 0;

        let sharedThrows = false;
        let resizableGetter = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "resizable").get;
        try {
            resizableGetter.call(new SharedArrayBuffer(1));
        } catch (error) {
            sharedThrows = error instanceof TypeError;
        }
        score += sharedThrows ? 16384 : 0;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(32_767));
}

#[test]
fn script_core_typed_array_concrete_prototypes_inherit_generic_surface() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let base = Object.getPrototypeOf(Uint8Array.prototype);
        let inheritedNames = [
            "buffer", "byteLength", "byteOffset", "length",
            "values", "keys", "entries", "set", "slice", "subarray",
            "copyWithin", "every", "fill", "filter", "find", "findIndex",
            "findLast", "findLastIndex", "forEach", "includes", "indexOf",
            "join", "lastIndexOf", "map", "reduce", "reduceRight",
            "reverse", "some", "sort", "toLocaleString", "toString",
            "at", "with"
        ];
        let concreteOwnCommon = false;
        for (let i = 0; i < inheritedNames.length; i = i + 1) {
            let name = inheritedNames[i];
            if (Uint8Array.prototype.hasOwnProperty(name) ||
                BigInt64Array.prototype.hasOwnProperty(name)) {
                concreteOwnCommon = true;
            }
        }

        if (!concreteOwnCommon) score += 1;
        if (!Uint8Array.prototype.hasOwnProperty(Symbol.iterator) &&
            !BigInt64Array.prototype.hasOwnProperty(Symbol.iterator)) score += 2;
        if (!Uint8Array.prototype.hasOwnProperty(Symbol.toStringTag) &&
            !BigInt64Array.prototype.hasOwnProperty(Symbol.toStringTag)) score += 4;
        if (base.hasOwnProperty("buffer") &&
            base.hasOwnProperty("values") &&
            base.hasOwnProperty(Symbol.iterator) &&
            base.hasOwnProperty(Symbol.toStringTag)) score += 8;
        if (Uint8Array.prototype.hasOwnProperty("constructor") &&
            Uint8Array.prototype.hasOwnProperty("BYTES_PER_ELEMENT") &&
            BigInt64Array.prototype.hasOwnProperty("constructor") &&
            BigInt64Array.prototype.hasOwnProperty("BYTES_PER_ELEMENT")) score += 16;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_typed_array_array_like_oversize_fails_before_element_get() {
    let result = compile_and_run_string(
        r#"
        function probe(Ctor, length) {
            let accessed = false;
            let source = { length: length };
            Object.defineProperty(source, "0", {
                get: function() {
                    accessed = true;
                    throw new TypeError("element access should not happen");
                }
            });
            try {
                new Ctor(source);
                return "missing";
            } catch (error) {
                return (error instanceof RangeError) + ":" + accessed;
            }
        }

        [
            probe(Uint8Array, 1073741825),
            probe(BigInt64Array, 134217729)
        ].join("|");
        "#,
    );

    assert_eq!(result, "true:false|true:false");
}

#[test]
fn script_core_typed_array_buffer_arg_undefined_length_uses_remaining_bytes() {
    let result = compile_and_run_string(
        r#"
        try {
            new Int16Array(new ArrayBuffer(1), 0, undefined);
            "missing";
        } catch (error) {
            String(error instanceof RangeError);
        }
        "#,
    );

    assert_eq!(result, "true");
}

#[test]
fn script_core_typed_array_buffer_arg_rechecks_detached_after_coercion() {
    let result = compile_and_run_string(
        r#"
        function detachOnOffset() {
            let buffer = new ArrayBuffer(6);
            let offset = {
                valueOf: function() {
                    buffer.transfer(0);
                    return 2;
                }
            };
            try {
                new Int16Array(buffer, offset);
                return "missing";
            } catch (error) {
                return error instanceof TypeError;
            }
        }

        function detachOnLength() {
            let buffer = new ArrayBuffer(6);
            let length = {
                valueOf: function() {
                    buffer.transfer(0);
                    return 1;
                }
            };
            try {
                new Int16Array(buffer, 0, length);
                return "missing";
            } catch (error) {
                return error instanceof TypeError;
            }
        }

        detachOnOffset() + ":" + detachOnLength();
        "#,
    );

    assert_eq!(result, "true:true");
}

#[test]
fn script_core_typed_array_from_of_rejects_short_custom_constructor_result() {
    let result = compile_and_run_string(
        r#"
        let TypedArray = Object.getPrototypeOf(Uint8Array);

        function probeFromIterable(Ctor, value) {
            let custom = function() {
                return new Ctor(1);
            };
            try {
                Ctor.from.call(custom, [value, value]);
                return "missing";
            } catch (error) {
                return error instanceof TypeError;
            }
        }

        function probeFromArrayLike(Ctor, value) {
            let custom = function() {
                return new Ctor(1);
            };
            try {
                Ctor.from.call(custom, { length: 2, 0: value, 1: value });
                return "missing";
            } catch (error) {
                return error instanceof TypeError;
            }
        }

        function probeOf(Ctor, first, second) {
            let custom = function() {
                return new Ctor(1);
            };
            try {
                TypedArray.of.call(custom, first, second);
                return "missing";
            } catch (error) {
                return error instanceof TypeError;
            }
        }

        [
            probeFromIterable(Uint8Array, 1),
            probeFromArrayLike(Uint8Array, 1),
            probeOf(Uint8Array, 1, 2),
            probeFromIterable(BigInt64Array, 1n),
            probeFromArrayLike(BigInt64Array, 1n),
            probeOf(BigInt64Array, 1n, 2n)
        ].join(":");
        "#,
    );

    assert_eq!(result, "true:true:true:true:true:true");
}

#[test]
fn script_core_data_view_tracks_resizable_array_buffer_bounds() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let rab = new ArrayBuffer(4, { maxByteLength: 8 });
        let tracking = new DataView(rab, 1);
        let fixed = new DataView(rab, 0, 4);

        if (tracking.byteLength === 3 && tracking.byteOffset === 1) score += 1;
        rab.resize(6);
        if (tracking.byteLength === 5 && fixed.byteLength === 4) score += 2;
        rab.resize(2);
        if (tracking.byteLength === 1 && tracking.getUint8(0) === 0) score += 4;

        let fixedThrows = false;
        try {
            fixed.getUint8(0);
        } catch (error) {
            fixedThrows = error instanceof TypeError;
        }
        if (fixedThrows) score += 8;

        rab.resize(1);
        if (tracking.byteLength === 0 && tracking.byteOffset === 1) score += 16;

        let trackingThrows = false;
        rab.resize(0);
        try {
            tracking.byteLength;
        } catch (error) {
            trackingThrows = error instanceof TypeError;
        }
        if (trackingThrows) score += 32;

        let buffer = new ArrayBuffer(3, { maxByteLength: 3 });
        let newTarget = function() {}.bind(null);
        Object.defineProperty(newTarget, "prototype", {
            get: function() {
                buffer.resize(2);
                return DataView.prototype;
            }
        });
        let constructed = Reflect.construct(DataView, [buffer, 2], newTarget);
        if (constructed.byteLength === 0) score += 64;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn script_core_array_predicate_helpers_read_length_before_callback_validation() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let everyLength = false;
        let everyLoop = false;
        let everyObject = {};
        Object.defineProperty(everyObject, "length", {
            get: function() {
                everyLength = true;
                return 1;
            }
        });
        Object.defineProperty(everyObject, "0", {
            get: function() {
                everyLoop = true;
                return 1;
            }
        });
        try {
            Array.prototype.every.call(everyObject);
        } catch (error) {
            score += error.constructor === TypeError ? 1 : 0;
        }
        score += everyLength ? 2 : 0;
        score += everyLoop ? 0 : 4;

        let someLength = false;
        let someLoop = false;
        let someObject = {};
        Object.defineProperty(someObject, "length", {
            get: function() {
                someLength = true;
                return 1;
            }
        });
        Object.defineProperty(someObject, "0", {
            get: function() {
                someLoop = true;
                return 1;
            }
        });
        try {
            Array.prototype.some.call(someObject);
        } catch (error) {
            score += error.constructor === TypeError ? 8 : 0;
        }
        score += someLength ? 16 : 0;
        score += someLoop ? 0 : 32;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn script_core_array_reduce_helpers_are_generic_and_skip_holes() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let obj = new Date(0);
        obj.length = 3;
        obj[0] = 2;
        obj[2] = 5;
        score += Array.prototype.reduce.call(obj, function(acc, value, index, receiver) {
            return acc + value + (receiver instanceof Date ? index : 100);
        }, 1);
        score += Array.prototype.reduceRight.call(obj, function(acc, value, index, receiver) {
            return acc + value + (receiver instanceof Date ? index : 100);
        }, 1) * 10;
        try {
            Array.prototype.reduce.call({ length: 2 }, function(acc, value) {
                return acc + value;
            });
        } catch (error) {
            score += error.constructor === TypeError ? 100 : 0;
        }
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(210));
}

#[test]
fn script_core_array_find_helpers_are_generic_and_visit_holes() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let values = [, 2];
        let sawHole = false;
        score += values.findIndex(function(value, index, receiver) {
            if (index === 0 && value === undefined && receiver === values) {
                sawHole = true;
            }
            return index === 1 && value === 2;
        }) === 1 ? 1 : 0;
        score += sawHole ? 2 : 0;

        let obj = new Date(0);
        obj.length = 3;
        obj[2] = 5;
        score += Array.prototype.find.call(obj, function(value, index, receiver) {
            return receiver instanceof Date && index === 2 && value === 5;
        }) === 5 ? 4 : 0;

        let sparse = { 0: "a", 2: "c", length: 3 };
        score += Array.prototype.findLast.call(sparse, function(value) {
            return value !== undefined;
        }) === "c" ? 8 : 0;
        score += Array.prototype.findLastIndex.call(true, function() {
            return true;
        }) === -1 ? 16 : 0;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_array_change_by_copy_helpers_are_generic_and_non_mutating() {
    let result = compile_and_run(
        r#"
        let score = 0;

        let values = [1, , 3];
        let reversed = values.toReversed();
        score += reversed !== values
            && reversed.length === 3
            && reversed[0] === 3
            && reversed[1] === undefined
            && reversed.hasOwnProperty("1")
            && reversed[2] === 1 ? 1 : 0;
        score += values.length === 3
            && values[0] === 1
            && !values.hasOwnProperty("1")
            && values[2] === 3 ? 2 : 0;

        let arrayLike = { 0: "b", 2: "a", length: 3 };
        let sorted = Array.prototype.toSorted.call(arrayLike);
        score += sorted.join(":") === "a:b:" ? 4 : 0;

        let replaced = Array.prototype.with.call(arrayLike, -1, "z");
        score += replaced[0] === "b"
            && replaced[1] === undefined
            && replaced.hasOwnProperty("1")
            && replaced[2] === "z" ? 8 : 0;

        let spliced = Array.prototype.toSpliced.call(
            { 0: "a", 1: "b", 2: "c", length: 3 },
            1,
            1,
            "x",
            "y"
        );
        score += spliced.join("") === "axyc" && spliced.length === 4 ? 16 : 0;

        try {
            [1, 2].with(2, 9);
        } catch (error) {
            score += error.constructor === RangeError ? 32 : 0;
        }

        try {
            Array.prototype.toSpliced.call({ length: 9007199254740991 }, 0, 0, 1);
        } catch (error) {
            score += error.constructor === TypeError ? 64 : 0;
        }

        score += Array.prototype[Symbol.unscopables].toReversed === true
            && Array.prototype[Symbol.unscopables].toSorted === true
            && Array.prototype[Symbol.unscopables].toSpliced === true
            && !Object.prototype.hasOwnProperty.call(
                Array.prototype[Symbol.unscopables],
                "with"
            ) ? 128 : 0;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(255));
}

#[test]
fn script_core_array_callback_helpers_read_length_before_callback_validation() {
    let result = compile_and_run(
        r#"
        function probe(method) {
            let score = 0;
            let lengthRead = false;
            let elementRead = false;
            let receiver = {};
            Object.defineProperty(receiver, "length", {
                get: function() {
                    lengthRead = true;
                    return 1;
                }
            });
            Object.defineProperty(receiver, "0", {
                get: function() {
                    elementRead = true;
                    return 1;
                }
            });
            try {
                Array.prototype[method].call(receiver);
            } catch (error) {
                score += error.constructor === TypeError ? 1 : 0;
            }
            score += lengthRead ? 2 : 0;
            score += elementRead ? 0 : 4;
            return score;
        }

        probe("forEach") + probe("map") * 10 + probe("filter") * 100;
        "#,
    );

    assert_eq!(result, Value::from_smi(777));
}

#[test]
fn script_core_array_callback_copy_helpers_define_own_result_properties() {
    let result = compile_and_run(
        r#"
        let score = 0;
        Object.defineProperty(Array.prototype, "1", {
            get: function() {
                return "prototype";
            },
            configurable: true
        });

        try {
            let mapped = [1, 2].map(function(value) {
                return value * 2;
            });
            score += mapped.length === 2
                && mapped.hasOwnProperty("1")
                && mapped[1] === 4 ? 1 : 0;

            let filtered = [1, 2].filter(function() {
                return true;
            });
            score += filtered.length === 2
                && filtered.hasOwnProperty("1")
                && filtered[1] === 2 ? 2 : 0;
        } finally {
            delete Array.prototype[1];
        }

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_array_spread_preserves_mapped_arrow_results() {
    let result = compile_and_run_string(
        r#"
        let negated = [1, 2].map(value => -value);
        let adjusted = [1, 2].map(value => value + 0.5);
        let spreadNegated = [...negated];
        let spreadAdjusted = [...adjusted];
        let combined = [
            ...[0, 1, 2],
            ...[0, 1, 2].map(value => -value),
        ];
        [
            String(negated[0]),
            String(negated[1]),
            String(adjusted[0]),
            String(adjusted[1]),
            String(spreadNegated[0]),
            String(spreadNegated[1]),
            String(spreadAdjusted[0]),
            String(spreadAdjusted[1]),
            String(spreadNegated.length),
            String(spreadAdjusted.length),
            String(combined[0]),
            String(combined[1]),
            String(combined[2]),
            String(combined[3]),
            String(combined[4]),
            String(combined[5]),
            String(combined.length),
        ].join("|");
        "#,
    );

    assert_eq!(result, "-1|-2|1.5|2.5|-1|-2|1.5|2.5|2|2|0|1|2|0|-1|-2|6");
}

#[test]
fn script_core_array_at_and_of_are_generic() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let object = { 0: "a", 2: "c", length: 3 };
        score += Array.prototype.at.call(object, -1) === "c" ? 1 : 0;
        score += Array.prototype.at.call(object, 1) === undefined ? 2 : 0;
        score += Array.prototype.at.call(true, 0) === undefined ? 4 : 0;

        let values = Array.of(1, 2, 3);
        score += values.length === 3
            && values[0] === 1
            && values[2] === 3 ? 8 : 0;

        function C(len) {
            this.lengthFromConstructor = len;
        }
        let custom = Array.of.call(C, "x", "y");
        score += custom instanceof C
            && custom.lengthFromConstructor === 2
            && custom.length === 2
            && custom[1] === "y" ? 16 : 0;

        score += Array.of.call({ notConstructor: true }, 5).join(":") === "5" ? 32 : 0;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn script_core_array_flat_and_flat_map_are_generic() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let nested = [1, [2, , [3]]];
        let flattened = nested.flat(2);
        score += flattened.length === 3
            && flattened[0] === 1
            && flattened[1] === 2
            && flattened[2] === 3 ? 1 : 0;

        let arrayLike = { 0: [4, 5], 1: 6, length: 2 };
        let generic = Array.prototype.flat.call(arrayLike);
        score += generic.length === 3
            && generic[0] === 4
            && generic[1] === 5
            && generic[2] === 6 ? 2 : 0;

        let mapped = [1, , 3].flatMap(function(value, index, receiver) {
            return receiver.length === 3 ? [value, index] : [0];
        });
        score += mapped.join(":") === "1:0:3:2" ? 4 : 0;

        let boolResult = Array.prototype.flatMap.call(true, function() {
            return [1];
        });
        score += boolResult.length === 0 ? 8 : 0;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_supports_object_has_own_property_builtin_shim() {
    let result = compile_and_run(
        r#"
        let own = { answer: 1 };
        (own.hasOwnProperty("answer") ? 1 : 0)
            + (own.hasOwnProperty("missing") ? 10 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_internal_spec_like_builtin_shims_use_public_semantics() {
    let result = compile_and_run_with_globals(
        r#"
        let score = 0;
        let call = Function.prototype.call;

        try {
            score += call.call(internalStringIndexOf, "😀x", "x") === 2 ? 1 : 0;
        } catch (_error) {}

        try {
            score += call.call(internalStringReplace, "abc", "b", "$&$&") === "abbc" ? 2 : 0;
        } catch (_error) {}

        try {
            let pushTarget = { length: new Number(1.9) };
            score += call.call(internalArrayPush, pushTarget, "x") === 2 ? 4 : 0;
            score += pushTarget[1] === "x" && pushTarget.length === 2 ? 8 : 0;
        } catch (_error) {}

        try {
            let popTarget = { 0: "first", 1: "last", length: new Number(2.8) };
            score += call.call(internalArrayPop, popTarget) === "last" ? 16 : 0;
            score += popTarget.length === 1 && popTarget[1] === undefined ? 32 : 0;
        } catch (_error) {}

        try {
            let searchTarget = { 2: "needle", length: new Number(3.9) };
            score += call.call(internalArrayIndexOf, searchTarget, "needle") === 2 ? 64 : 0;
        } catch (_error) {}

        try {
            score += call.call(internalObjectToString, new Date(0)) === "[object Date]" ? 128 : 0;
        } catch (_error) {}

        try {
            let tagged = {};
            tagged[Symbol.toStringTag] = "Tagged";
            score += call.call(internalObjectToString, tagged) === "[object Tagged]" ? 256 : 0;
        } catch (_error) {}

        try {
            let inherited = Object.create({ answer: 1 });
            score += call.call(internalHasOwnProperty, inherited, "answer") === false ? 512 : 0;
        } catch (_error) {}

        score;
        "#,
        |agent, realm| {
            install_native_global(
                agent,
                realm,
                "internalStringIndexOf",
                internal_string_index_of_builtin(),
                false,
            );
            install_native_global(
                agent,
                realm,
                "internalStringReplace",
                internal_string_replace_builtin(),
                false,
            );
            install_native_global(
                agent,
                realm,
                "internalArrayIndexOf",
                internal_array_index_of_builtin(),
                false,
            );
            install_native_global(
                agent,
                realm,
                "internalArrayPush",
                internal_array_push_builtin(),
                false,
            );
            install_native_global(
                agent,
                realm,
                "internalArrayPop",
                internal_array_pop_builtin(),
                false,
            );
            install_native_global(
                agent,
                realm,
                "internalObjectToString",
                internal_object_to_string_builtin(),
                false,
            );
            install_native_global(
                agent,
                realm,
                "internalHasOwnProperty",
                internal_object_has_own_property_builtin(),
                false,
            );
        },
    );

    assert_eq!(result, Value::from_smi(1023));
}

#[test]
fn script_core_supports_for_in_head_destructuring_bindings() {
    let result = compile_and_run(
        r#"
        let seen = 0;
        for (var [x, x] in { ab: null }) {
            seen = seen + (x === "b" ? 1 : 100);
        }
        seen;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_preserves_outer_var_bindings_across_for_in_lexical_heads() {
    let result = compile_and_run_string(
        r#"
        var probeBefore = function() { return x; };
        var probeExpr, probeDecl, probeBody;
        var x = 1;

        for (
            let [_ = probeDecl = function() { return x; }]
            in
            { '': probeExpr = function() { return x; }}
        )
            var x = 2, __ = probeBody = function() { return x; };

        probeBefore() + ":" + probeExpr() + ":" + probeDecl() + ":" + probeBody() + ":" + x;
        "#,
    );

    assert_eq!(result, "2:2:2:2:2");
}

#[test]
fn script_core_uses_reference_errors_for_for_in_lexical_head_tdz() {
    let result = compile_and_run_string(
        r#"
        try {
            let x = 1;
            for (let x in { x }) {}
            "no-error";
        } catch (error) {
            error.name;
        }
        "#,
    );

    assert_eq!(result, "ReferenceError");
}

#[test]
fn script_core_scopes_for_in_lexical_head_and_body_closures_correctly() {
    let result = compile_and_run_string(
        r#"
        let x = 'outside';
        var probeBefore = function() { return x; };
        var probeExpr, probeDecl, probeBody;

        try {
            for (
                let [x, _ = probeDecl = function() { return x; }]
                in
                { i: probeExpr = function() { typeof x; } }
            )
                probeBody = function() { return x; };
            "loop-ok";
        } catch (error) {
            "caught:" + error.name;
        }

        let exprResult;
        try {
            exprResult = probeExpr();
        } catch (error) {
            exprResult = error.name;
        }

        probeBefore() + ":" + exprResult + ":" +
            (probeDecl ? probeDecl() : "nodecl") + ":" +
            (probeBody ? probeBody() : "nobody");
        "#,
    );

    assert_eq!(result, "outside:ReferenceError:i:i");
}

#[test]
fn script_core_lowers_named_group_match_result_destructuring() {
    let mut atoms = AtomTable::new();
    let _ = compile_unit(
        r#"
        let {a, b} = "bab".match(/(?<b>b)\k<a>(?<a>a)\k<b>/u).groups;
        a + b;
        "#,
        &mut atoms,
    );
}

#[test]
fn script_core_lowers_var_declarators_that_share_names_with_hoisted_functions() {
    let mut atoms = AtomTable::new();
    let _ = compile_unit(
        r#"
        var __string;
        var __re = /1|12/;
        __re.exec(__string);
        __re.test(__string);
        function __string() {}
        "#,
        &mut atoms,
    );
}

#[test]
fn script_core_installs_phase6_numeric_globals_and_reflect_namespace() {
    let result = compile_and_run_string(
        r#"
        [
            typeof Number,
            typeof Math,
            typeof BigInt,
            typeof RegExp,
            typeof Reflect,
            typeof Reflect.apply,
            typeof Reflect.construct,
            typeof Reflect.defineProperty,
            typeof Reflect.deleteProperty,
            typeof Reflect.get,
            typeof Reflect.getOwnPropertyDescriptor,
            typeof Reflect.getPrototypeOf,
            typeof Reflect.has,
            typeof Reflect.isExtensible,
            typeof Reflect.ownKeys,
            typeof Reflect.preventExtensions,
            typeof Reflect.set,
            typeof Reflect.setPrototypeOf,
            typeof Reflect.enumerate
        ].join(":");
        "#,
    );

    assert_eq!(
        result,
        "function:object:function:function:object:function:function:function:function:function:function:function:function:function:function:function:function:function:undefined"
    );
}

#[test]
fn script_core_reflect_dispatches_basic_object_operations() {
    let result = compile_and_run(
        r#"
        let total = 0;
        let target = { answer: 1 };
        let proto = { marker: 2 };

        total += (Reflect.get(target, "answer") === 1 ? 1 : 0);
        total += (Reflect.has(target, "answer") ? 2 : 0);
        total += (Reflect.set(target, "answer", 5) && target.answer === 5 ? 4 : 0);
        total += (Reflect.defineProperty(target, "sealed", {
            value: 7,
            configurable: false,
            enumerable: true,
            writable: false
        }) ? 8 : 0);
        total += (Reflect.getOwnPropertyDescriptor(target, "sealed").value === 7 ? 16 : 0);
        total += (Reflect.deleteProperty(target, "answer") && !Reflect.has(target, "answer") ? 32 : 0);
        total += (Reflect.setPrototypeOf(target, proto) && Reflect.getPrototypeOf(target) === proto ? 64 : 0);
        total += (Reflect.preventExtensions(target) && !Reflect.isExtensible(target) ? 128 : 0);
        total += (Reflect.ownKeys(target).join(",") === "sealed" ? 256 : 0);
        total += (Reflect.construct(function Box(value) { this.value = value; }, [11]).value === 11 ? 512 : 0);
        total += (Reflect.apply(function(a, b) { return this.base + a + b; }, { base: 3 }, [4, 5]) === 12 ? 1024 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(2047));
}

#[test]
fn script_core_reflect_uses_accessor_paths_and_validates_argument_lists() {
    let result = compile_and_run(
        r#"
        let total = 0;
        let accessorBase = {};
        let receiver = { marker: 41, stored: 0 };

        Object.defineProperty(accessorBase, "value", {
            get: function() {
                return this.marker;
            },
            set: function(next) {
                this.stored = next;
            }
        });

        total += (Reflect.get(accessorBase, "value", receiver) === 41 ? 1 : 0);
        total += (Reflect.get(Object.create(accessorBase), "value", receiver) === 41 ? 2 : 0);
        total += (Reflect.set(accessorBase, "value", 7, receiver) && receiver.stored === 7 ? 4 : 0);

        let throwing = {};
        Object.defineProperty(throwing, "value", {
            set: function() {
                throw "setter";
            }
        });
        try {
            Reflect.set(throwing, "value", 1);
        } catch (error) {
            total += (error === "setter" ? 8 : 0);
        }

        let receiverAccessor = {};
        Object.defineProperty(receiverAccessor, "slot", {
            set: function(value) {}
        });
        total += (Reflect.set({}, "slot", 9, receiverAccessor) === false ? 16 : 0);

        try {
            Reflect.apply(function() {}, null, undefined);
        } catch (error) {
            total += (error.constructor === TypeError ? 32 : 0);
        }

        try {
            Reflect.construct(function() {}, null);
        } catch (error) {
            total += (error.constructor === TypeError ? 64 : 0);
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn script_core_installs_proxy_constructor_and_revocable_pair_surface() {
    let result = compile_and_run_string(
        r#"
        let revocableType = "missing";
        let pairTypes = "missing";
        if (typeof Proxy === "function") {
            revocableType = typeof Proxy.revocable;
            if (revocableType === "function") {
                let pair = Proxy.revocable({}, {});
                pairTypes = typeof pair.proxy + ":" + typeof pair.revoke;
            }
        }
        typeof Proxy + ":" + revocableType + ":" + pairTypes;
        "#,
    );

    assert_eq!(result, "function:function:object:function");
}

#[test]
fn script_core_proxy_is_construct_only_and_validates_constructor_arguments() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += (typeof Proxy === "function" ? 1 : 0);
        total += (Object.prototype.hasOwnProperty.call(Proxy, "prototype") ? 0 : 2);
        try {
            Proxy({}, {});
        } catch (error) {
            total += (error.constructor === TypeError ? 4 : 0);
        }
        try {
            new Proxy(1, {});
        } catch (error) {
            total += (error.constructor === TypeError ? 8 : 0);
        }
        try {
            new Proxy({}, 1);
        } catch (error) {
            total += (error.constructor === TypeError ? 16 : 0);
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_proxy_revocable_revokes_idempotently_and_throws_afterward() {
    let result = compile_and_run(
        r#"
        let pair = Proxy.revocable({ answer: 1 }, {});
        let total = 0;
        total += (pair.proxy.answer === 1 ? 1 : 0);
        total += (pair.revoke() === undefined ? 2 : 0);
        try {
            pair.proxy.answer;
        } catch (error) {
            total += (error.constructor === TypeError ? 4 : 0);
        }
        try {
            Object.getPrototypeOf(pair.proxy);
        } catch (error) {
            total += (error.constructor === TypeError ? 8 : 0);
        }
        total += (pair.revoke() === undefined ? 16 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_callable_proxies_reach_typeof_call_and_construct_paths() {
    let result = compile_and_run(
        r#"
        let target = function(value) {
            if (new.target) {
                this.value = value;
                return;
            }
            return value + 1;
        };
        target.prototype.marker = 1;
        let proxy = new Proxy(target, {});
        let total = 0;

        try {
            total += (typeof proxy === "function" ? 1 : 0);
        } catch (error) {
            total += 1024;
        }
        try {
            total += (proxy(4) === 5 ? 2 : 0);
        } catch (error) {
            total += 2048;
        }
        try {
            total += (Function.prototype.call.call(proxy, null, 5) === 6 ? 4 : 0);
        } catch (error) {
            total += 4096;
        }
        try {
            total += (Reflect.apply(proxy, null, [6]) === 7 ? 8 : 0);
        } catch (error) {
            total += 8192;
        }
        try {
            let instance = new proxy(8);
            total += (instance.value === 8 ? 16 : 0);
            total += (Object.getPrototypeOf(instance) === target.prototype ? 32 : 0);
        } catch (error) {
            total += 16384;
        }
        try {
            let constructed = Reflect.construct(proxy, [9]);
            total += (constructed.value === 9 ? 64 : 0);
        } catch (error) {
            total += 32768;
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn script_core_proxy_named_property_caches_stay_proxy_aware() {
    let result = compile_and_run(
        r#"
        let gets = 0;
        let sets = 0;
        let target = { value: 1 };
        let proxy = new Proxy(target, {
            get(target, key, receiver) {
                if (key === "value") {
                    gets += 1;
                    return gets * 10;
                }
                return target[key];
            },
            set(target, key, value, receiver) {
                if (key === "value") {
                    sets += 1;
                    target[key] = value + 1;
                    return true;
                }
                target[key] = value;
                return true;
            }
        });

        let total = 0;
        total += proxy.value;
        total += proxy.value;
        proxy.value = 4;
        proxy.value = 7;
        total += proxy.value;
        total += (gets === 3 ? 1 : 0);
        total += (sets === 2 ? 2 : 0);
        total += (target.value === 8 ? 4 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(67));
}

#[test]
fn script_core_proxy_null_traps_forward_and_own_keys_require_property_keys() {
    let result = compile_and_run(
        r#"
        let total = 0;
        let target = { value: 3 };
        let proxy = new Proxy(target, { get: null, set: null });
        total += (proxy.value === 3 ? 1 : 0);
        proxy.value = 7;
        total += (target.value === 7 ? 2 : 0);

        let callable = new Proxy(function(value) { return value + 1; }, { apply: null });
        total += (callable(4) === 5 ? 4 : 0);

        let constructible = new Proxy(function(value) { this.value = value; }, { construct: null });
        let instance = new constructible(6);
        total += (instance.value === 6 ? 8 : 0);

        try {
            Object.keys(new Proxy({}, {
                ownKeys: function() {
                    return [true];
                }
            }));
        } catch (error) {
            total += (error.constructor === TypeError ? 16 : 0);
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_proxy_invariants_cover_define_delete_prototype_and_prevent_extensions() {
    let result = compile_and_run(
        r#"
        let total = 0;

        let defineTrapCalls = 0;
        let defineProxy = new Proxy({}, {
            defineProperty: function(target, prop, desc) {
                Object.defineProperty(target, prop, {
                    configurable: false,
                    writable: true
                });
                defineTrapCalls += 1;
                return true;
            }
        });
        try {
            Reflect.defineProperty(defineProxy, "prop", { writable: false });
        } catch (error) {
            total += (error instanceof TypeError ? 1 : 0);
        }
        total += (defineTrapCalls === 1 ? 2 : 0);

        let deleteTrapCalls = 0;
        let deleteProxy = new Proxy({ prop: 1 }, {
            deleteProperty: function(target, prop) {
                Object.preventExtensions(target);
                deleteTrapCalls += 1;
                return true;
            }
        });
        try {
            Reflect.deleteProperty(deleteProxy, "prop");
        } catch (error) {
            total += (error instanceof TypeError ? 4 : 0);
        }
        total += (deleteTrapCalls === 1 ? 8 : 0);
        total += (Reflect.deleteProperty(deleteProxy, "missing") ? 16 : 0);
        total += (deleteTrapCalls === 2 ? 32 : 0);

        function Custom() {}
        let prototypeProxy = new Proxy({}, {
            getPrototypeOf: function() {
                return Custom.prototype;
            }
        });
        total += (prototypeProxy instanceof Custom ? 64 : 0);

        let observedHandler;
        let observedTarget;
        let observedProp;
        let hasTarget = {};
        let hasHandler = {
            has: function(target, prop) {
                observedHandler = this;
                observedTarget = target;
                observedProp = prop;
                return false;
            }
        };
        "attr" in Object.create(new Proxy(hasTarget, hasHandler));
        total += (observedHandler === hasHandler ? 128 : 0);
        total += (observedTarget === hasTarget ? 256 : 0);
        total += (observedProp === "attr" ? 512 : 0);

        let innerPrevent = new Proxy({}, {
            preventExtensions: function() {
                return false;
            }
        });
        try {
            Object.preventExtensions(new Proxy(innerPrevent, {}));
        } catch (error) {
            total += (error instanceof TypeError ? 1024 : 0);
        }

        let setProtoCalls = [];
        let setProto = {};
        let setProtoTarget = new Proxy(Object.create(setProto), {
            isExtensible: function() {
                setProtoCalls.push("target.[[IsExtensible]]");
                return false;
            },
            getPrototypeOf: function() {
                setProtoCalls.push("target.[[GetPrototypeOf]]");
                return setProto;
            }
        });
        Object.preventExtensions(setProtoTarget);
        let setProtoProxy = new Proxy(setProtoTarget, {
            setPrototypeOf: function() {
                setProtoCalls.push("proxy.[[SetPrototypeOf]]");
                return true;
            }
        });
        total += (setProtoCalls.length === 0 ? 2048 : 0);
        total += (Reflect.setPrototypeOf(setProtoProxy, setProto) ? 4096 : 0);
        total += (
            setProtoCalls.join(",") ===
            "proxy.[[SetPrototypeOf]],target.[[IsExtensible]],target.[[GetPrototypeOf]]"
                ? 8192
                : 0
        );

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(16383));
}

#[test]
fn script_core_supports_phase6_number_math_and_bigint_basics() {
    let result = compile_and_run(
        r#"
        let number = Number("41");
        let wrapped = new Number(2);
        let bigint = BigInt("9");
        let total = 0;
        total += (number === 41 ? 1 : 0);
        total += (wrapped.valueOf() === 2 ? 2 : 0);
        total += (wrapped.toString() === "2" ? 4 : 0);
        total += (Math.max(-4, 8, 3) === 8 ? 8 : 0);
        total += (Math.round(-0.25) === 0 ? 16 : 0);
        total += (BigInt.prototype.valueOf.call(bigint) === bigint ? 32 : 0);
        total += (BigInt.prototype.toString.call(bigint) === "9" ? 64 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn script_core_number_min_value_is_minimum_subnormal() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += (Number.MIN_VALUE / 2 === 0 ? 1 : 0);
        total += (1 / (Number.MIN_VALUE / -2) === -Infinity ? 2 : 0);
        total += (Number.MIN_VALUE / 1.9 === Number.MIN_VALUE ? 4 : 0);
        total += (Number.MIN_VALUE * 0.5 === 0 ? 8 : 0);
        total += (1 / (-0.5 * Number.MIN_VALUE) === -Infinity ? 16 : 0);
        total += (Number.MIN_VALUE * 0.51 === Number.MIN_VALUE ? 32 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn script_core_subnormal_numbers_do_not_collide_with_value_tags() {
    let result = compile_and_run(
        r#"
        let value = 1;
        for (let power = 0; power < 1039; power += 1) {
            value = value * 0.5;
        }

        let total = 0;
        total += (typeof value === "number" ? 1 : 0);
        total += (+value === value ? 2 : 0);

        let next = value * 0.5;
        total += (next * 2 === value ? 4 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn script_core_unary_plus_uses_to_number_instead_of_addition() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += (+"12345" === 12345 ? 1 : 0);
        total += (Number(".12345e-3") === 0.00012345 ? 2 : 0);
        total += (+"-1234567890" === -1234567890 ? 4 : 0);
        try {
            +1n;
        } catch (error) {
            total += (error instanceof TypeError ? 8 : 0);
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_unary_minus_uses_to_primitive_number_hint() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += (-{ valueOf: function() { return -1; } } === 1 ? 1 : 0);
        total += (-new Boolean(true) === -1 ? 2 : 0);
        total += (-1n === -1n ? 4 : 0);
        try {
            -{ valueOf: function() { return {}; }, toString: function() { return {}; } };
        } catch (error) {
            total += (error.constructor === TypeError ? 8 : 0);
        }
        try {
            -Symbol();
        } catch (error) {
            total += (error.constructor === TypeError ? 16 : 0);
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_postfix_update_returns_old_numeric_value() {
    let result = compile_and_run(
        r#"
        let total = 0;

        let booleanValue = false;
        total += (booleanValue++ === 0 && booleanValue === 1 ? 1 : 0);

        let objectValue = { valueOf: function() { return 1; } };
        total += (objectValue++ === 1 && objectValue === 2 ? 2 : 0);

        let stringValue = "x";
        let oldStringValue = stringValue++;
        total += (isNaN(oldStringValue) && isNaN(stringValue) ? 4 : 0);

        let bigintValue = 1n;
        total += (bigintValue++ === 1n && bigintValue === 2n ? 8 : 0);

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_supports_number_formatting_builtins() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += ((1).toFixed(1) === "1.0" ? 1 : 0);
        total += (Number.NaN.toFixed(2) === "NaN" ? 2 : 0);
        total += ((3).toFixed(4) === "3.0000" ? 4 : 0);
        total += ((1000).toPrecision(3) === "1.00e+3" ? 8 : 0);
        total += ((7).toPrecision(3) === "7.00" ? 16 : 0);
        total += ((42).toPrecision() === "42" ? 32 : 0);
        total += ((Infinity).toPrecision(1000) === "Infinity" ? 64 : 0);
        total += ((7).toLocaleString() === "7" ? 128 : 0);
        total += ((-0).toExponential(2) === "0.00e+0" ? 256 : 0);
        total += (Number.parseInt === parseInt && Number.parseFloat === parseFloat ? 512 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(1023));
}

#[test]
fn script_core_number_rejects_symbol_arguments() {
    let result = compile_and_run(
        r#"
        let symbol = Symbol("66");
        let total = 0;
        try {
            Number(symbol);
        } catch (error) {
            total += (error instanceof TypeError ? 1 : 0);
        }
        try {
            new Number(symbol);
        } catch (error) {
            total += (error instanceof TypeError ? 2 : 0);
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_supports_extended_math_builtins() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += (Math.acos.length === 1 && Math.acos(1) === 0 ? 1 : 0);
        total += (Math.acosh(1) === 0 ? 2 : 0);
        total += (Math.asin(0) === 0 ? 4 : 0);
        total += (Math.asinh(0) === 0 ? 8 : 0);
        total += (Math.atan(0) === 0 ? 16 : 0);
        total += (Math.atan2.length === 2 && Math.atan2(0, -1) === Math.PI ? 32 : 0);
        total += (Math.atanh(0) === 0 ? 64 : 0);
        total += (Math.cbrt(8) === 2 ? 128 : 0);
        total += (Math.ceil(-0.25) === 0 && 1 / Math.ceil(-0.25) === -Infinity ? 256 : 0);
        total += (Math.clz32(1) === 31 && Math.clz32(-1) === 0 ? 512 : 0);
        total += (Math.cos(0) === 1 && Math.cosh(0) === 1 ? 1024 : 0);
        total += (Math.exp(0) === 1 && Math.expm1(0) === 0 ? 2048 : 0);
        total += (Math.f16round(0.1) === 0.0999755859375 && Math.f16round(65520) === Infinity ? 4096 : 0);
        total += (Math.fround(0.1) === 0.10000000149011612 ? 8192 : 0);
        total += (Math.hypot(3, 4) === 5 && Math.hypot(Infinity, NaN) === Infinity ? 16384 : 0);
        total += (Math.imul(0xffffffff, 5) === -5 ? 32768 : 0);
        total += (Math.log(1) === 0 && Math.log10(100) === 2 ? 65536 : 0);
        total += (Math.log1p(0) === 0 && Math.log2(8) === 3 ? 131072 : 0);
        let random = Math.random();
        total += (typeof random === "number" && random >= 0 && random < 1 ? 262144 : 0);
        total += (Math.sin(0) === 0 && Math.sinh(0) === 0 ? 524288 : 0);
        total += (Math.tan(0) === 0 && Math.tanh(0) === 0 ? 1048576 : 0);
        let calls = 0;
        let coercible = { valueOf: function() { calls += 1; } };
        Math.max(NaN, coercible);
        Math.min(NaN, coercible);
        total += (calls === 2 ? 2097152 : 0);
        total += (
            Math.pow(1, NaN) !== Math.pow(1, NaN) &&
            Math.pow(-1, Infinity) !== Math.pow(-1, Infinity)
                ? 4194304
                : 0
        );
        total += (Math.sumPrecise([1e30, 0.1, -1e30]) === 0.1 ? 8388608 : 0);
        total += (
            Math.sumPrecise([1e308, 1e308, 0.1, 0.1, 1e30, 0.1, -1e30, -1e308, -1e308]) === 0.30000000000000004
                ? 16777216
                : 0
        );
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(33_554_431));
}

#[test]
fn script_core_number_prototype_carries_primitive_data_slot_but_bigint_does_not() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += (Number.prototype.toString() === "0" ? 1 : 0);
        total += (Number.prototype.valueOf() === 0 ? 2 : 0);
        try {
            BigInt.prototype.toString();
        } catch (error) {
            total += (error instanceof TypeError ? 4 : 0);
        }
        try {
            BigInt.prototype.valueOf();
        } catch (error) {
            total += (error instanceof TypeError ? 8 : 0);
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_number_constructor_distinguishes_absent_and_undefined_arguments() {
    let result = compile_and_run(
        r#"
        let functionDefault = Number();
        let constructorDefault = new Number().valueOf();
        let functionUndefined = Number(undefined);
        let constructorUndefined = new Number(undefined).valueOf();
        let total = 0;
        total += (functionDefault === 0 ? 1 : 0);
        total += (constructorDefault === 0 ? 2 : 0);
        total += (functionUndefined !== functionUndefined ? 4 : 0);
        total += (constructorUndefined !== constructorUndefined ? 8 : 0);
        total += (Number.prototype.toString.length === 1 ? 16 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_bigint_boxing_exposes_wrapper_identity_and_value_of() {
    let result = compile_and_run(
        r#"
        let boxed = Object(1n);
        let total = 0;
        total += (typeof boxed === "object" ? 1 : 0);
        total += (boxed instanceof BigInt ? 2 : 0);
        total += (BigInt.prototype.valueOf.call(boxed) === 1n ? 4 : 0);
        total += (Object.prototype.toString.call(boxed) === "[object BigInt]" ? 8 : 0);
        total += (Object.getPrototypeOf(boxed) === BigInt.prototype ? 16 : 0);
        total += (boxed.valueOf() === 1n ? 32 : 0);
        total += (!(boxed instanceof Boolean) ? 64 : 0);
        total += (BigInt.prototype !== Boolean.prototype ? 128 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(255));
}

#[test]
fn script_core_bigint_width_builtins_wrap_with_toindex_and_tobigint() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += (BigInt.asIntN(2, 3n) === -1n ? 1 : 0);
        total += (BigInt.asIntN(3, 10n) === 2n ? 2 : 0);
        total += (BigInt.asUintN(2, -1n) === 3n ? 4 : 0);
        total += (BigInt.asUintN(8, 0x123n) === 0x23n ? 8 : 0);
        total += (BigInt.asIntN.length === 2 ? 16 : 0);
        total += (BigInt.asUintN.length === 2 ? 32 : 0);
        try {
            BigInt.asIntN(0n, 0n);
        } catch (error) {
            total += (error instanceof TypeError ? 64 : 0);
        }
        try {
            BigInt.asUintN(9007199254740992, 0n);
        } catch (error) {
            total += (error instanceof RangeError ? 128 : 0);
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(255));
}

#[test]
fn script_core_supports_bigint_literals_and_number_radix_poisoning_cases() {
    let result = compile_and_run(
        r#"
        function Test262Error() {}
        let total = 0;
        let poisoned = {
            valueOf() {
                throw new Test262Error();
            }
        };
        try {
            0..toString(poisoned);
        } catch (error) {
            total += (error.constructor === Test262Error ? 1 : 0);
        }
        total += ((15n).toString(16) === "f" ? 2 : 0);
        try {
            (0n).toString(Symbol());
        } catch (error) {
            total += (error.constructor === TypeError ? 4 : 0);
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn script_core_bigint_to_string_uses_lowercase_digits_through_radix_36() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += ((10n).toString(11) === "a" ? 1 : 0);
        total += ((35n).toString(36) === "z" ? 2 : 0);
        total += (Number(97n) === 97 ? 4 : 0);
        total += (String.fromCharCode(Number(97n)) === "a" ? 8 : 0);
        let loopText = "";
        for (let i = 10n; i < 13; i++) {
            loopText += i.toString(36);
        }
        total += (loopText === "abc" ? 16 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_bigint_wrapper_ordinary_to_primitive_observes_overridden_methods() {
    let result = compile_and_run_string(
        r#"
        const BigIntToString = BigInt.prototype.toString;
        const BigIntValueOf = BigInt.prototype.valueOf;
        let toStringGets = 0;
        let toStringCalls = 0;
        let valueOfGets = 0;
        let valueOfCalls = 0;
        let toStringFunction = function() {
            ++toStringCalls;
            return `${BigIntToString.call(this)}foo`;
        };
        let valueOfFunction = function() {
            ++valueOfCalls;
            return BigIntValueOf.call(this) * 2n;
        };
        function record(thunk) {
            try {
                return String(thunk());
            } catch (error) {
                return error instanceof TypeError ? "TypeError" : "throw";
            }
        }
        Object.defineProperty(BigInt.prototype, "toString", {
            get: function() {
                ++toStringGets;
                return toStringFunction;
            },
        });

        let output = "";
        output += record(function() { return "" + Object(1n); }) + "|";
        output += record(function() { return +Object(1n); }) + "|";
        output += record(function() { return `${Object(1n)}`; }) + "|";
        output += toStringGets + "," + toStringCalls + "|";

        Object.defineProperty(BigInt.prototype, "valueOf", {
            get: function() {
                ++valueOfGets;
                return valueOfFunction;
            },
        });

        output += record(function() { return Object(1n) == 2n; }) + "|";
        output += record(function() { return Object(1n) + 1n; }) + "|";
        output += record(function() { return ({ "1foo": 1, "2": 2 })[Object(1n)]; }) + "|";
        output += toStringGets + "," + toStringCalls + "," + valueOfGets + "," + valueOfCalls + "|";

        toStringFunction = undefined;
        output += record(function() { return 1 + Object(1n); }) + "|";
        output += record(function() { return Object(1n) * 1n; }) + "|";
        output += record(function() { return "".concat(Object(1n)); }) + "|";
        output += toStringGets + "," + toStringCalls + "," + valueOfGets + "," + valueOfCalls;
        output;
        "#,
    );

    assert_eq!(
        result,
        "1|TypeError|1foo|1,1|true|3|1|2,2,2,2|TypeError|2|2|3,2,5,5"
    );
}

#[test]
fn script_core_bigint_bitwise_operators_use_to_numeric() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += ((0b101n & 0b011n) === 0b001n ? 1 : 0);
        total += ((-2n & -3n) === -4n ? 2 : 0);

        let calls = "";
        let left = {
            [Symbol.toPrimitive]: function(hint) {
                calls += "L" + hint;
                return 5n;
            }
        };
        let right = {
            [Symbol.toPrimitive]: function(hint) {
                calls += "R" + hint;
                return 3n;
            }
        };
        total += ((left & right) === 1n ? 4 : 0);
        total += (calls === "LnumberRnumber" ? 8 : 0);
        total += (({
            valueOf: 1,
            toString: function() {
                return 6n;
            }
        } & 7n) === 6n ? 16 : 0);

        try {
            1n & 1;
        } catch (error) {
            total += (error.constructor === TypeError ? 32 : 0);
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn script_core_bitwise_not_supports_number_and_bigint() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += (~0 === -1 ? 1 : 0);
        total += (~1n === -2n ? 2 : 0);
        total += (~Object(1n) === -2n ? 4 : 0);
        total += (~{ valueOf: function() { return 1n; } } === -2n ? 8 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_addition_uses_default_to_primitive_hint() {
    let result = compile_and_run(
        r#"
        let total = 0;
        let calls = "";
        let left = {
            [Symbol.toPrimitive]: function(hint) {
                calls += "L" + hint;
                return 2;
            }
        };
        let right = {
            [Symbol.toPrimitive]: function(hint) {
                calls += "R" + hint;
                return 3;
            }
        };
        total += (left + right === 5 ? 1 : 0);
        total += (calls === "LdefaultRdefault" ? 2 : 0);
        total += ({ [Symbol.toPrimitive]: function(hint) { return hint; } } + "" === "default" ? 4 : 0);
        try {
            0 + { [Symbol.toPrimitive]: function() { return Symbol.toPrimitive; } };
        } catch (error) {
            total += (error.constructor === TypeError ? 8 : 0);
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_bigint_shift_operators_use_to_numeric() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += ((8n << 2n) === 32n ? 1 : 0);
        total += ((32n >> 2n) === 8n ? 2 : 0);
        total += ((-5n >> 1n) === -3n ? 4 : 0);
        total += ((8n << -1n) === 4n ? 8 : 0);
        total += ((8n >> -1n) === 16n ? 16 : 0);

        let calls = "";
        let left = {
            [Symbol.toPrimitive]: function(hint) {
                calls += "L" + hint;
                return 4n;
            }
        };
        let right = {
            [Symbol.toPrimitive]: function(hint) {
                calls += "R" + hint;
                return 1n;
            }
        };
        total += ((left << right) === 8n ? 32 : 0);
        total += (calls === "LnumberRnumber" ? 64 : 0);

        try {
            1n << 1;
        } catch (error) {
            total += (error.constructor === TypeError ? 128 : 0);
        }
        try {
            1n >>> 0n;
        } catch (error) {
            total += (error.constructor === TypeError ? 256 : 0);
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(511));
}

#[test]
fn script_core_exponentiation_uses_to_numeric_order_and_infinity_edges() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += (isNaN(1 ** Infinity) ? 1 : 0);
        total += (isNaN((-1) ** -Infinity) ? 2 : 0);

        let trace = "";
        try {
            ({
                valueOf: function() {
                    trace += "L";
                    return Symbol("x");
                }
            }) ** ({
                valueOf: function() {
                    trace += "R";
                    return 1;
                }
            });
        } catch (error) {
            total += (error.constructor === TypeError ? 4 : 0);
        }
        total += (trace === "L" ? 8 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_bigint_relational_comparison_uses_to_numeric_ordering() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += (0n < 1 ? 1 : 0);
        total += (0.000000000001 < 1n ? 2 : 0);
        total += ((1n < 1) === false ? 4 : 0);
        total += ((Number.MIN_VALUE < -10n) === false ? 8 : 0);
        total += (1n < "2" ? 16 : 0);
        total += ("2" < 3n ? 32 : 0);
        total += ((1n < "not numeric") === false ? 64 : 0);
        total += (("0e0" < 1n) === false ? 128 : 0);
        total += ((0n < "1e0") === false ? 256 : 0);
        total += (("" < 1n) ? 512 : 0);

        try {
            1n < Symbol();
        } catch (error) {
            total += (error.constructor === TypeError ? 1024 : 0);
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(2047));
}

#[test]
fn script_core_sparse_array_literals_preserve_length_and_holes() {
    let result = compile_and_run(
        r#"
        let oneHole = [,];
        let twoHoles = [,,];
        let total = 0;
        total += (oneHole.length === 1 ? 1 : 0);
        total += (!(0 in oneHole) ? 2 : 0);
        total += (oneHole[0] === undefined ? 4 : 0);
        total += (twoHoles.length === 2 ? 8 : 0);
        total += (!(1 in twoHoles) ? 16 : 0);
        total += ([].length === 0 ? 32 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn script_core_map_and_set_iterables_preserve_size_and_insertion_order() {
    let result = compile_and_run(
        r#"
        let set = new Set([1, "a", true]);
        let map = new Map([[1, "a"], ["b", true]]);
        let setOrder = "";
        for (const value of set) {
            setOrder += value + "|";
        }
        let mapOrder = "";
        for (const entry of map) {
            mapOrder += entry[0] + ":" + entry[1] + "|";
        }
        let total = 0;
        total += (set.size === 3 ? 1 : 0);
        total += (setOrder === "1|a|true|" ? 2 : 0);
        total += (map.size === 2 ? 4 : 0);
        total += (mapOrder === "1:a|b:true|" ? 8 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_supports_phase6_regexp_literals_and_constructor_state() {
    let result = compile_and_run(
        r#"
        let literal = /ab+/gi;
        let same = RegExp(literal);
        let cloned = new RegExp(literal, "m");
        let descriptor = Object.getOwnPropertyDescriptor(literal, "lastIndex");
        let total = 0;
        total += (Object.getPrototypeOf(literal) === RegExp.prototype ? 1 : 0);
        total += (literal.source === "ab+" ? 2 : 0);
        total += (literal.flags === "gi" ? 4 : 0);
        total += (literal.lastIndex === 0 ? 8 : 0);
        total += (descriptor.writable === true ? 16 : 0);
        total += (descriptor.enumerable === false ? 32 : 0);
        total += (descriptor.configurable === false ? 64 : 0);
        total += (literal.toString() === "/ab+/gi" ? 128 : 0);
        total += (Object.prototype.toString.call(literal) === "[object RegExp]" ? 256 : 0);
        total += (same === literal ? 512 : 0);
        total += (cloned !== literal ? 1024 : 0);
        total += (cloned.source === "ab+" ? 2048 : 0);
        total += (cloned.flags === "m" ? 4096 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(8191));
}

#[test]
fn script_core_regexp_constructor_reads_regexp_like_source_and_flags() {
    let result = compile_and_run_string(
        r#"
        let regexpLike = {
            source: "source text",
            flags: "i"
        };
        regexpLike[Symbol.match] = true;
        let fromLike = new RegExp(regexpLike);

        let overrideFlags = {
            source: "override text"
        };
        overrideFlags[Symbol.match] = true;
        Object.defineProperty(overrideFlags, "flags", {
            get: function() {
                throw new Error("flags getter should not run");
            }
        });
        let fromOverride = new RegExp(overrideFlags, "g");

        [
            fromLike.source,
            fromLike.flags,
            fromOverride.source,
            fromOverride.flags
        ].join("|");
        "#,
    );

    assert_eq!(result, "source text|i|override text|g");
}

#[test]
fn script_core_supports_regexp_exec_test_and_flag_getters() {
    let result = compile_and_run(
        r#"
        let re = new RegExp("1|12", "gm");
        let first = re.exec("123");
        let second = re.exec("123");
        let third = re.exec("123");
        let total = 0;
        total += (typeof re.exec === "function" ? 1 : 0);
        total += (typeof re.test === "function" ? 2 : 0);
        total += (re.global ? 4 : 0);
        total += (re.multiline ? 8 : 0);
        total += (!re.ignoreCase ? 16 : 0);
        total += (first !== null && first[0] === "1" && first.index === 0 ? 32 : 0);
        total += (second === null ? 64 : 0);
        total += (third !== null && third[0] === "1" && third.index === 0 ? 128 : 0);
        total += (re.lastIndex === 1 ? 256 : 0);
        re.lastIndex = 0;
        total += (re.test("123") ? 512 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(1023));
}

#[test]
fn script_core_supports_regexp_exec_for_escaped_literal_patterns() {
    let result = compile_and_run(
        r#"
        let tab = /\t\t/.exec("a\u0009\u0009b");
        let newline = /\n/.test("a\nb");
        let control = /\cJ/.exec("\n");
        (tab !== null && tab[0] === "\u0009\u0009" ? 1 : 0)
            + (newline ? 2 : 0)
            + (control !== null && control[0] === "\n" ? 4 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn script_core_supports_regexp_source_flags_and_indices_getters() {
    let result = compile_and_run(
        r#"
        let sourceDescriptor = Object.getOwnPropertyDescriptor(RegExp.prototype, "source");
        let flagsDescriptor = Object.getOwnPropertyDescriptor(RegExp.prototype, "flags");
        let hasIndicesDescriptor = Object.getOwnPropertyDescriptor(RegExp.prototype, "hasIndices");
        let unicodeSetsDescriptor = Object.getOwnPropertyDescriptor(RegExp.prototype, "unicodeSets");
        let empty = new RegExp();
        let indexed = /(?<head>a)(b)?/d;
        let unicodeSets = /./v;
        let match = indexed.exec("ac");
        let total = 0;
        total += (typeof sourceDescriptor.get === "function" ? 1 : 0);
        total += (typeof flagsDescriptor.get === "function" ? 2 : 0);
        total += (typeof hasIndicesDescriptor.get === "function" ? 4 : 0);
        total += (Object.getOwnPropertyDescriptor(empty, "source") === undefined ? 8 : 0);
        total += (empty.source === "(?:)" ? 16 : 0);
        total += (indexed.hasIndices === true ? 32 : 0);
        total += (indexed.flags === "d" ? 64 : 0);
        total += (match !== null && match.groups.head === "a" ? 128 : 0);
        total += (match !== null && match.indices[0][0] === 0 && match.indices[0][1] === 1 ? 256 : 0);
        total += (match !== null && match.indices[1][0] === 0 && match.indices[1][1] === 1 ? 512 : 0);
        total += (match !== null && match.indices[2] === undefined ? 1024 : 0);
        total += (match !== null && match.indices.groups.head[0] === 0 && match.indices.groups.head[1] === 1 ? 2048 : 0);
        total += (unicodeSetsDescriptor && typeof unicodeSetsDescriptor.get === "function" ? 4096 : 0);
        total += (unicodeSets.unicodeSets === true && /./.unicodeSets === false ? 8192 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(16383));
}

#[test]
fn script_core_regexp_source_escapes_literal_delimiters() {
    let result = compile_and_run_string(
        r#"
        let slash = new RegExp("/");
        let newline = new RegExp("\n");
        [slash.source, newline.source].join("|");
        "#,
    );

    assert_eq!(result, "\\/|\\n");
}

#[test]
fn script_core_regexp_unknown_script_property_aliases_match_generated_sets() {
    let result = compile_and_run_string(
        r#"
        [
            /^\p{Script=Unknown}+$/u.test(String.fromCodePoint(0x038B)),
            /^\p{Script=Zzzz}+$/u.test(String.fromCodePoint(0x038B)),
            /^\p{sc=Unknown}+$/u.test(String.fromCodePoint(0x038B)),
            /^\p{sc=Zzzz}+$/u.test(String.fromCodePoint(0x038B)),
            /^\P{Script=Unknown}+$/u.test(String.fromCodePoint(0x038C)),
            /^\P{Script=Zzzz}+$/u.test(String.fromCodePoint(0x038C)),
            /^\P{sc=Unknown}+$/u.test(String.fromCodePoint(0x038C)),
            /^\P{sc=Zzzz}+$/u.test(String.fromCodePoint(0x038C)),
            /^\p{Script_Extensions=Unknown}+$/u.test(String.fromCodePoint(0x038B)),
            /^\p{Script_Extensions=Zzzz}+$/u.test(String.fromCodePoint(0x038B)),
            /^\p{scx=Unknown}+$/u.test(String.fromCodePoint(0x038B)),
            /^\p{scx=Zzzz}+$/u.test(String.fromCodePoint(0x038B)),
            /^\P{Script_Extensions=Unknown}+$/u.test(String.fromCodePoint(0x038C)),
            /^\P{Script_Extensions=Zzzz}+$/u.test(String.fromCodePoint(0x038C)),
            /^\P{scx=Unknown}+$/u.test(String.fromCodePoint(0x038C)),
            /^\P{scx=Zzzz}+$/u.test(String.fromCodePoint(0x038C)),
            /^\p{Script=Unknown}+$/u.test(String.fromCharCode(0xDC00)),
            /^\p{Script_Extensions=Unknown}+$/u.test(String.fromCharCode(0xDC00)),
            /^\p{Script=Unknown}+$/u.test(String.fromCodePoint(0xE000)),
            /^\p{Script_Extensions=Unknown}+$/u.test(String.fromCodePoint(0xE000))
        ].join("|");
        "#,
    );

    assert_eq!(
        result,
        "true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true"
    );
}

#[test]
fn script_core_regexp_unicode_sets_exec_uses_unicode_aware_matching() {
    let result = compile_and_run(
        r#"
        let text = String.fromCodePoint(0x20BB7) + "a" + String.fromCodePoint(0x20BB7);
        let total = 0;

        let literal = /\u{20BB7}/v.exec(text);
        total += (literal && literal[0].length === 2 && literal.index === 0) ? 1 : 0;

        let property = /\p{Script=Han}/v.exec(text);
        total += (property && property[0].length === 2 && property.index === 0) ? 2 : 0;

        let dot = /./v.exec(text);
        total += (dot && dot[0].length === 2 && dot.index === 0) ? 4 : 0;

        let ascii = /\p{ASCII}/v.exec(text);
        total += (ascii && ascii[0] === "a" && ascii.index === 2) ? 8 : 0;

        let groups = /(\p{Script=Han})(.)/v.exec(text);
        total += (groups && groups[0].length === 3 && groups[1].length === 2 && groups[2] === "a" && groups.index === 0) ? 16 : 0;

        let literalText = '𠮷a𠮷b𠮷';
        let literalSource = /𠮷/v.exec(literalText);
        total += (literalSource && literalSource[0].length === 2 && literalSource.index === 0) ? 32 : 0;

        let miss = /x/v.exec(text);
        total += (miss === null) ? 64 : 0;

        let complexText = 'a\u{20BB7}b\u{10FFFF}c';
        let nonAscii = /\P{ASCII}/v.exec(complexText);
        total += (nonAscii && nonAscii[0].length === 2 && nonAscii.index === 1) ? 128 : 0;

        let unicodeLiteral = /\u{20BB7}/u.exec(text);
        total += (unicodeLiteral && unicodeLiteral[0].length === 2 && unicodeLiteral.index === 0) ? 256 : 0;

        let unicodeProperty = /\p{Script=Han}/u.exec(text);
        total += (unicodeProperty && unicodeProperty[0].length === 2 && unicodeProperty.index === 0) ? 512 : 0;

        let unicodeDot = /./u.exec(text);
        total += (unicodeDot && unicodeDot[0].length === 2 && unicodeDot.index === 0) ? 1024 : 0;

        let unicodeGroups = /(\p{Script=Han})(.)/u.exec(text);
        total += (unicodeGroups && unicodeGroups[0].length === 3 && unicodeGroups[1].length === 2 && unicodeGroups[2] === "a" && unicodeGroups.index === 0) ? 2048 : 0;

        let unicodeNonAscii = /\P{ASCII}/u.exec(complexText);
        total += (unicodeNonAscii && unicodeNonAscii[0].length === 2 && unicodeNonAscii.index === 1) ? 4096 : 0;

        function doExec(regex) {
            let result = regex.exec(text);
            return result ? [result[0], result.index] : null;
        }
        function sameArray(a, b) {
            if (a === null || b === null || a.length !== b.length) {
                return false;
            }
            for (let i = 0; i < a.length; ++i) {
                if (a[i] !== b[i]) {
                    return false;
                }
            }
            return true;
        }
        total += sameArray(doExec(/𠮷/v), ["𠮷", 0]) ? 8192 : 0;
        total += sameArray(doExec(/\p{Script=Han}/v), ["𠮷", 0]) ? 16384 : 0;
        total += sameArray(doExec(/./v), ["𠮷", 0]) ? 32768 : 0;
        total += sameArray(doExec(/\p{ASCII}/v), ["a", 2]) ? 65536 : 0;
        total += doExec(/x/v) === null ? 131072 : 0;

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(262143));
}

#[test]
fn script_core_regexp_non_unicode_astral_named_groups_survive_backend_normalization() {
    let result = compile_and_run_string(
        r#"
        let match = "fox dog".match(/(?<𝑓𝑜𝑥>fox).*(?<𝓓𝓸𝓰>dog)/);
        match.groups.𝑓𝑜𝑥 + ":" + match.groups.𝓓𝓸𝓰;
        "#,
    );

    assert_eq!(result, "fox:dog");
}

#[test]
fn script_core_regexp_constructor_rejects_unicode_annex_b_identity_escape() {
    let result = compile_and_run_string(
        r#"
        function isSyntaxCharacter(c) {
            return "^$\\.*+?()[]{}|".indexOf(c) !== -1;
        }
        function isAlphaDigit(c) {
            return ("0" <= c && c <= "9") ||
                ("A" <= c && c <= "Z") ||
                ("a" <= c && c <= "z");
        }
        let bad = "none";
        for (let cu = 0; cu <= 0x7f && bad === "none"; ++cu) {
            let s = String.fromCharCode(cu);
            if (!isAlphaDigit(s) && !isSyntaxCharacter(s) && s !== "/") {
                try {
                    RegExp("\\" + s, "u");
                    bad = "atom:" + cu;
                } catch (error) {
                    if (Object.getPrototypeOf(error) !== SyntaxError.prototype) {
                        bad = "atom-other:" + cu;
                    }
                }
            }
        }
        for (let cu = 0; cu <= 0x7f && bad === "none"; ++cu) {
            let s = String.fromCharCode(cu);
            if (!isAlphaDigit(s) && !isSyntaxCharacter(s) && s !== "/" && s !== "-") {
                try {
                    RegExp("[\\" + s + "]", "u");
                    bad = "class:" + cu;
                } catch (error) {
                    if (Object.getPrototypeOf(error) !== SyntaxError.prototype) {
                        bad = "class-other:" + cu;
                    }
                }
            }
        }
        bad;
        "#,
    );

    assert_eq!(result, "none");
}

#[test]
fn script_core_regexp_exec_preserves_zero_length_last_index_at_match_end() {
    let result = compile_and_run(
        r#"
        let re = /(?:)/uy;
        let input = "\uD83D\uDE00a";
        let first = re.exec(input);
        let afterFirst = re.lastIndex;
        re.lastIndex = 2;
        let second = re.exec(input);
        let afterSecond = re.lastIndex;
        re.lastIndex = 3;
        let third = re.exec(input);
        let afterThird = re.lastIndex;
        let total = 0;
        total += (first !== null && first.index === 0 && afterFirst === 0 ? 1 : 0);
        total += (second !== null && second.index === 2 && afterSecond === 2 ? 2 : 0);
        total += (third !== null && third.index === 3 && afterThird === 3 ? 4 : 0);
        re.lastIndex = 4;
        let fourth = re.exec(input);
        total += (fourth === null && re.lastIndex === 0 ? 8 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_regexp_match_all_advances_unicode_empty_matches_by_code_point() {
    let result = compile_and_run_string(
        r#"
        let iterator = /(?:)/gu[Symbol.matchAll]("\uD83D\uDE00a");
        let first = iterator.next();
        let second = iterator.next();
        let third = iterator.next();
        let fourth = iterator.next();
        [
            first.value.index,
            second.value.index,
            third.value.index,
            fourth.done
        ].join("|");
        "#,
    );

    assert_eq!(result, "0|2|3|true");
}

#[test]
fn script_core_validates_regexp_constructor_flags() {
    let result = compile_and_run(
        r#"
        let total = 0;
        try {
            new RegExp("", "gg");
        } catch (error) {
            total += Object.getPrototypeOf(error) === SyntaxError.prototype ? 1 : 100;
        }
        try {
            RegExp("", "z");
        } catch (error) {
            total += Object.getPrototypeOf(error) === SyntaxError.prototype ? 2 : 100;
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_supports_annex_b_regexp_compile() {
    let result = compile_and_run(
        r#"
        let total = 0;
        let desc = Object.getOwnPropertyDescriptor(RegExp.prototype, "compile");
        total += (desc && desc.writable && !desc.enumerable && desc.configurable ? 1 : 0);
        total += (RegExp.prototype.compile.length === 2 ? 2 : 0);

        let subject = /original/ig;
        subject.lastIndex = 23;
        total += (subject.compile("new") === subject ? 4 : 0);
        total += (subject.source === "new" && subject.flags === "" ? 8 : 0);
        total += (!subject.test("NEW") && subject.test("new") ? 16 : 0);
        total += (subject.lastIndex === 0 ? 32 : 0);

        let sourceGetterCount = 0;
        let flagsGetterCount = 0;
        let other = /abc/gim;
        Object.defineProperty(other, "source", { get() { sourceGetterCount += 1; return "bad"; } });
        Object.defineProperty(other, "flags", { get() { flagsGetterCount += 1; return ""; } });
        subject.compile(other);
        total += (subject.toString() === "/abc/gim" && sourceGetterCount === 0 && flagsGetterCount === 0 ? 64 : 0);

        try {
            subject.compile(other, "");
        } catch (error) {
            total += (error instanceof TypeError ? 128 : 0);
        }

        Object.defineProperty(subject, "lastIndex", { value: 45, writable: false });
        try {
            subject.compile(/updated/i);
        } catch (error) {
            total += (error instanceof TypeError ? 256 : 0);
        }
        total += (subject.toString() === "/updated/i" && subject.lastIndex === 45 ? 512 : 0);

        let subclassed = new (class extends RegExp {})("");
        try {
            subclassed.compile();
        } catch (error) {
            total += (error instanceof TypeError ? 1024 : 0);
        }
        try {
            RegExp.prototype.compile.call(subclassed);
        } catch (error) {
            total += (error instanceof TypeError ? 2048 : 0);
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(4095));
}

#[test]
fn script_core_supports_annex_b_regexp_legacy_static_accessors() {
    let result = compile_and_run(
        r#"
        let total = 0;
        let inputDesc = Object.getOwnPropertyDescriptor(RegExp, "input");
        let aliasDesc = Object.getOwnPropertyDescriptor(RegExp, "$_");
        let captureDesc = Object.getOwnPropertyDescriptor(RegExp, "$1");
        total += (typeof inputDesc.get === "function" && typeof inputDesc.set === "function" ? 1 : 0);
        total += (typeof aliasDesc.get === "function" && typeof aliasDesc.set === "function" ? 2 : 0);
        total += (typeof captureDesc.get === "function" && captureDesc.set === undefined ? 4 : 0);
        total += (!inputDesc.enumerable && inputDesc.configurable ? 8 : 0);

        RegExp.input = "seed";
        total += (RegExp.input === "seed" && RegExp.$_ === "seed" ? 16 : 0);

        /(a)(b)?/.exec("zzac");
        total += (RegExp.input === "zzac" ? 32 : 0);
        total += (RegExp.lastMatch === "a" && RegExp["$&"] === "a" ? 64 : 0);
        total += (RegExp.leftContext === "zz" && RegExp["$`"] === "zz" ? 128 : 0);
        total += (RegExp.rightContext === "c" && RegExp["$'"] === "c" ? 256 : 0);
        total += (RegExp.lastParen === "a" && RegExp["$+"] === "a" ? 512 : 0);
        total += (RegExp.$1 === "a" && RegExp.$2 === "" && RegExp.$9 === "" ? 1024 : 0);

        try {
            inputDesc.get.call({});
        } catch (error) {
            total += (error instanceof TypeError ? 2048 : 0);
        }

        class MyRegExp extends RegExp {}
        try {
            MyRegExp.input;
        } catch (error) {
            total += (error instanceof TypeError ? 4096 : 0);
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(8191));
}

#[test]
fn script_core_supports_annex_b_regexp_legacy_identity_escapes() {
    let result = compile_and_run(
        r#"
        let total = 0;
        let cyrillic = String.fromCharCode(0x0410);
        let invalidControl = new RegExp("\\c" + cyrillic);
        total += (invalidControl.test("\\c" + cyrillic) ? 1 : 0);
        total += (!invalidControl.test(String.fromCharCode(cyrillic.charCodeAt(0) % 32)) ? 2 : 0);

        let invalidClass = new RegExp("[\\c_]");
        total += (invalidClass.test(String.fromCharCode("_".charCodeAt(0) % 32)) ? 4 : 0);
        total += (!invalidClass.test("\\") && !invalidClass.test("c") && !invalidClass.test("_") ? 8 : 0);

        let nul = String.fromCharCode(0);
        let escapedNul = new RegExp("\\" + nul);
        total += (escapedNul.source === "\\" + nul && escapedNul.test(nul) ? 16 : 0);

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_supports_regexp_prototype_escape_and_receiver_edges() {
    let result = compile_and_run(
        r#"
        let sourceGet = Object.getOwnPropertyDescriptor(RegExp.prototype, "source").get;
        let globalGet = Object.getOwnPropertyDescriptor(RegExp.prototype, "global").get;
        let total = 0;
        total += (sourceGet.call(RegExp.prototype) === "(?:)" ? 1 : 0);
        total += (globalGet.call(RegExp.prototype) === undefined ? 2 : 0);
        try {
            RegExp.prototype.exec("");
        } catch (error) {
            total += (error.constructor === TypeError ? 4 : 0);
        }
        total += (RegExp.escape("a+b/ ") === "\\x61\\+b\\/\\x20" ? 8 : 0);
        try {
            RegExp.escape({});
        } catch (error) {
            total += (error.constructor === TypeError ? 16 : 0);
        }
        total += (Object.prototype.toString.call(RegExp.prototype) === "[object Object]" ? 32 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn script_core_supports_annex_b_escape_and_unescape_globals() {
    let result = compile_and_run_string(
        r#"
        [
            typeof escape,
            escape.length,
            Object.getOwnPropertyDescriptor(globalThis, "escape").enumerable,
            Object.getOwnPropertyDescriptor(globalThis, "escape").configurable,
            escape("AZaz09*@-_+./ !\u0100\u{10401}"),
            unescape("%41%u0100%uD801%uDC01%zz"),
            unescape(123n)
        ].join("|");
        "#,
    );

    assert_eq!(
        result,
        "function|1|false|true|AZaz09*@-_+./%20%21%u0100%uD801%uDC01|A\u{0100}\u{10401}%zz|123"
    );
}

#[test]
fn script_core_supports_annex_b_string_html_methods_and_trim_aliases() {
    let result = compile_and_run_string(
        r#"
        [
            "Lyng".bold(),
            "Lyng".fontcolor("\"<&"),
            "Lyng".link("a\"b"),
            String.prototype.trimLeft === String.prototype.trimStart,
            String.prototype.trimRight === String.prototype.trimEnd,
            String.prototype.trimLeft.name,
            "\tLyng\n".trimLeft(),
            "\tLyng\n".trimRight()
        ].join("|");
        "#,
    );

    assert_eq!(
        result,
        "<b>Lyng</b>|<font color=\"&quot;<&\">Lyng</font>|<a href=\"a&quot;b\">Lyng</a>|true|true|trimStart|Lyng\n|\tLyng"
    );
}

#[test]
fn script_core_string_substr_matches_annex_b_numeric_edges() {
    let result = compile_and_run_string(
        r#"
        function toIntegerOrInfinity(value) {
            return Number.isNaN(value) ? 0 : Math.trunc(value);
        }
        function expectedSubstr(string, start, length) {
            let size = string.length;
            let intStart = toIntegerOrInfinity(start);
            if (intStart === -Infinity) {
                intStart = 0;
            } else if (intStart < 0) {
                intStart = Math.max(size + intStart, 0);
            } else {
                intStart = Math.min(intStart, size);
            }
            let intLength = length === undefined ? size : toIntegerOrInfinity(length);
            intLength = Math.min(Math.max(intLength, 0), size);
            let intEnd = Math.min(intStart + intLength, size);
            let result = string.substring(intStart, intEnd);
            if (!Object.is(result.length, intEnd - intStart)) {
                return "bad length:" + [string, String(start), String(length), String(result.length), String(intEnd - intStart)].join("|");
            }
            for (let index = 0; index < result.length; index = index + 1) {
                if (result[index] !== string[intStart + index]) {
                    return "bad char:" + [string, String(start), String(length), String(index)].join("|");
                }
            }
            return result;
        }
        let strings = ["", "a", "ab", "abc"];
        let positiveIntegers = [0, 1, 2, 3, 4, 5, 10, 100];
        let integers = [
            ...positiveIntegers,
            ...positiveIntegers.map(value => -value),
        ];
        let numbers = [
            ...integers,
            ...integers.map(value => value + 0.5),
            -Infinity, Infinity, NaN,
        ];
        let lengths = numbers.concat([undefined]);
        let mismatch = "ok";
        outer:
        for (let string of strings) {
            for (let start of numbers) {
                for (let length of lengths) {
                    if (typeof start !== "number") {
                        mismatch = "bad start:" + String(start);
                        break outer;
                    }
                    if (length !== undefined && typeof length !== "number") {
                        mismatch = "bad length arg:" + String(length);
                        break outer;
                    }
                    let actual = string.substr(start, length);
                    let expected = expectedSubstr(string, start, length);
                    if (expected.startsWith("bad ")) {
                        mismatch = expected;
                        break outer;
                    }
                    if (actual !== expected) {
                        mismatch = [string, String(start), String(length), actual, expected].join("|");
                        break outer;
                    }
                }
            }
        }
        mismatch;
        "#,
    );

    assert_eq!(result, "ok");
}

#[test]
fn script_core_supports_annex_b_function_code_block_function_bindings() {
    let result = compile_and_run_string(
        r#"
        var before, after, blockValue, blockAfterAssignment;

        (function() {
            before = f;

            {
                function f() { return 'decl'; }
                blockValue = f();
                f = 123;
                blockAfterAssignment = f;
            }

            after = f();
        }());

        String(before) + ':' + blockValue + ':' + blockAfterAssignment + ':' + after;
        "#,
    );

    assert_eq!(result, "undefined:decl:123:decl");
}

#[test]
fn script_core_supports_annex_b_global_code_block_function_bindings() {
    let result = compile_and_run_string(
        r#"
        var before = f === undefined ? 'undefined' : 'other';

        {
            function f() { return 'global'; }
        }

        before + ':' + f();
        "#,
    );

    assert_eq!(result, "undefined:global");
}

#[test]
fn script_core_annex_b_call_expression_assignment_targets_throw_after_callee() {
    let result = compile_and_run_string(
        r#"
        var out = [];
        var fCalled = false;
        var fValueOfCalled = false;
        var gCalled = false;

        function reset() {
            fCalled = false;
            fValueOfCalled = false;
            gCalled = false;
        }

        function record(threw) {
            out.push(threw, fCalled, fValueOfCalled, gCalled);
        }

        function f() {
            fCalled = true;
            return {
                valueOf() {
                    fValueOfCalled = true;
                    return 1;
                }
            };
        }

        function g() {
            gCalled = true;
            return 1;
        }

        reset();
        try { f() = g(); } catch (error) { record(error.constructor === ReferenceError); }

        reset();
        try { f() += g(); } catch (error) { record(error.constructor === ReferenceError); }

        reset();
        try { f()++; } catch (error) { record(error.constructor === ReferenceError); }

        reset();
        try { ++f(); } catch (error) { record(error.constructor === ReferenceError); }

        reset();
        try { for (f() in [1]) {} } catch (error) { record(error.constructor === ReferenceError); }

        reset();
        try { for (f() of [1]) {} } catch (error) { record(error.constructor === ReferenceError); }

        out.join(":");
        "#,
    );

    assert_eq!(
        result,
        [
            "true", "true", "false", "false", "true", "true", "false", "false", "true", "true",
            "false", "false", "true", "true", "false", "false", "true", "true", "false", "false",
            "true", "true", "false", "false",
        ]
        .join(":")
    );
}

#[test]
fn script_core_annex_b_for_in_var_initializer_runs_before_rhs() {
    let result = compile_and_run_string(
        r#"
        var effects = 0;
        var iterations = 0;
        var stored;

        for (var a = (++effects, -1) in stored = a, { first: 0, second: 1 }) {
            ++iterations;
        }

        String(effects) + ":" + String(stored) + ":" + String(iterations) + ":" + String(a);
        "#,
    );

    assert_eq!(result, "1:-1:2:second");
}

#[test]
fn script_core_annex_b_labeled_function_declaration_is_hoisted() {
    let result = compile_and_run_string(
        r#"
        label: function f() { return 'ok'; }
        f();
        "#,
    );

    assert_eq!(result, "ok");
}

#[test]
fn script_core_annex_b_catch_var_redeclaration_updates_simple_catch_parameter() {
    let result = compile_and_run_string(
        r#"
        var out = [];

        foo = "outer";
        try {
            throw "caught";
        } catch (foo) {
            out.push(foo);
            var foo = "var statement";
            out.push(foo);
        }
        out.push(foo);

        try {
            throw "caught";
        } catch (err) {
            out.push(err);
            for (var err in { propertyName: null }) {
                out.push(err);
            }
            out.push(err);
        }

        try {
            throw "caught";
        } catch (value) {
            out.push(value);
            for (var value of [2]) {
                out.push(value);
            }
            out.push(value);
        }

        out.join(":");
        "#,
    );

    assert_eq!(
        result,
        "caught:var statement:outer:caught:propertyName:propertyName:caught:2:2"
    );
}

#[test]
fn script_core_annex_b_regexp_legacy_pattern_extensions() {
    let result = compile_and_run_string(
        r#"
        var out = [];

        out.push(/]/.exec(" ]{}")[0]);
        out.push(/{/.exec(" ]{}")[0]);
        out.push(/}/.exec(" ]{}")[0]);
        out.push(/x{o}x/.exec("x{o}x")[0]);
        out.push(/[\c0]/.exec("\x0f\x10\x11")[0].charCodeAt(0));
        out.push(/[\c00]+/.exec("\x0f0\x10\x11")[0].length);
        out.push(/.(?=Z)+/.exec("a bZ")[0]);
        out.push(/[a-e](?!Z)+/.exec("aZ e")[0]);

        out.join(":");
        "#,
    );

    assert_eq!(result, "]:{:}:x{o}x:16:2:b:e");
}

#[test]
fn script_core_typeof_before_for_lexical_shadow_returns_undefined() {
    let result = compile_and_run_string(
        r#"
        var beforeType;

        beforeType = typeof f;
        for (let f; ; ) {
            {
                function f() {}
            }
            break;
        }

        beforeType;
        "#,
    );

    assert_eq!(result, "undefined");
}

#[test]
fn script_core_block_function_named_arguments_shadows_arguments_object() {
    let result = compile_and_run_string(
        r#"
        (function() {
            {
                var before = arguments();
                function arguments() { return 'block'; }
                return before + ':' + arguments();
            }
        }());
        "#,
    );

    assert_eq!(result, "block:block");
}

#[test]
fn script_core_skips_annex_b_block_function_var_binding_for_destructured_catch_conflicts() {
    let result = compile_and_run_string(
        r#"
        var destructured, simple;

        (function() {
            try {
                throw { f: 1 };
            } catch ({ f }) {
                {
                    function f() { return 'blocked'; }
                }
            }
            destructured = typeof f;
        }());

        (function() {
            try {
                throw null;
            } catch (g) {
                {
                    function g() { return 'allowed'; }
                }
            }
            simple = g();
        }());

        destructured + ':' + simple;
        "#,
    );

    assert_eq!(result, "undefined:allowed");
}

#[test]
fn script_core_skips_annex_b_block_function_var_binding_for_for_lexical_conflicts() {
    let result = compile_and_run_string(
        r#"
        var before, beforeType, loopResult, after, afterType;

        try {
            f;
        } catch (error) {
            before = error.constructor === ReferenceError ? 'ref' : 'other';
        }
        try {
            beforeType = typeof f;
        } catch (error) {
            beforeType = error.constructor === ReferenceError ? 'type-ref' : 'type-other';
        }

        try {
            for (let f; ; ) {
                {
                    function f() {}
                }
                break;
            }
            loopResult = 'loop-ok';
        } catch (error) {
            loopResult = error.constructor === ReferenceError ? 'loop-ref' : 'loop-other';
        }

        try {
            f;
        } catch (error) {
            after = error.constructor === ReferenceError ? 'ref' : 'other';
        }
        try {
            afterType = typeof f;
        } catch (error) {
            afterType = error.constructor === ReferenceError ? 'type-ref' : 'type-other';
        }

        [before, beforeType, loopResult, after, afterType].join(':');
        "#,
    );

    assert_eq!(result, "ref:undefined:loop-ok:ref:undefined");
}

#[test]
fn script_core_supports_string_slice_and_regexp_exec_array_to_string() {
    let result = compile_and_run(
        r#"
        let sliced = "abcdefghijklmnabcdefghijklmn".slice(14);
        let total = 0;
        total += (sliced === "abcdefghijklmn" ? 1 : 0);
        total += ("abcd".slice(1, 3) === "bc" ? 2 : 0);
        total += ("abcd".slice(-3, -1) === "bc" ? 4 : 0);
        total += (new RegExp("World").exec("Hello World!").toString() === "World" ? 8 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_supports_regexp_constructor_patterns_with_raw_line_terminators() {
    let result = compile_and_run(
        r#"
        let re = new RegExp("(.|\r|\n)*", "");
        let match = re.exec();
        let total = 0;
        total += (match[0] === "undefined" ? 1 : 0);
        total += (new RegExp("\n").source === "\\n" ? 2 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_supports_phase5_boolean_and_symbol_basics() {
    let result = compile_and_run(
        r#"
        let key = Symbol.for("phase5.registry");
        let wrapper = new Boolean(false);
        (typeof Symbol.for("phase5.registry") === "symbol" ? 1 : 0)
            + (Symbol.keyFor(key) === "phase5.registry" ? 2 : 0)
            + (Boolean.prototype.valueOf.call(true) === true ? 4 : 0)
            + (wrapper.valueOf() === false ? 8 : 0)
            + (key.toString() === "Symbol(phase5.registry)" ? 16 : 0)
            + (key.valueOf() === key ? 32 : 0)
            + (key[Symbol.toPrimitive]() === key ? 64 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn script_core_rejects_symbol_construction() {
    let result = compile_and_run(
        r#"
        let threw = 0;
        try {
            new Symbol();
        } catch (error) {
            threw = typeof error === "object" ? 1 : -1;
        }
        threw;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_supports_bitwise_and_boolean_chains() {
    let result = compile_and_run(
        r#"
        ((1 === 1) & (2 === 2)) + (((3 === 4) & (5 === 5)) * 10);
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_supports_bitwise_or_in_comparator_style_expressions() {
    let result = compile_and_run(
        r#"
        ((15 / 4) | 0) - ((3 / 4) | 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_supports_exponentiation_and_right_associativity() {
    let result = compile_and_run(
        r#"
        (2 ** 5) + (2 ** 3 ** 2) + (-(2 ** 4));
        "#,
    );

    assert_eq!(result, Value::from_smi(528));
}

#[test]
fn script_core_treats_for_in_over_nullish_values_as_empty() {
    let result = compile_and_run(
        r#"
        let seen = 0;
        for (var key in undefined) {
            seen = seen + 100;
        }
        for (var other in null) {
            seen = seen + 100;
        }
        seen;
        "#,
    );

    assert_eq!(result, Value::from_smi(0));
}

#[test]
fn script_core_supports_non_index_numeric_computed_property_keys() {
    let result = compile_and_run(
        r#"
        let obj = {
            [1.2]: 1,
            [1e55]: 2,
            [0.000001]: 3,
            [-0]: 4,
            [Infinity]: 5,
            [-Infinity]: 6,
            [NaN]: 7
        };
        (obj['1.2'] === 1 ? 1 : 0)
            + (obj['1e+55'] === 2 ? 2 : 0)
            + (obj['0.000001'] === 3 ? 4 : 0)
            + (obj[0] === 4 ? 8 : 0)
            + (obj[Infinity] === 5 ? 16 : 0)
            + (obj[-Infinity] === 6 ? 32 : 0)
            + (obj[NaN] === 7 ? 64 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn script_core_slice_handles_large_array_like_lengths_without_materializing_full_range() {
    let result = compile_and_run(
        r#"
        let total = 0;
        let arrayLike = {
            "9007199254740989": "a",
            "9007199254740990": "b",
            length: 9007199254740994
        };
        let ranged = Array.prototype.slice.call(
            arrayLike,
            9007199254740989,
            9007199254740996
        );
        total += ranged.length === 2 ? 1 : 0;
        total += ranged[0] === "a" ? 2 : 0;
        total += ranged[1] === "b" ? 4 : 0;

        let tail = Array.prototype.slice.call(arrayLike, -2, -1);
        total += tail.length === 1 ? 8 : 0;
        total += tail[0] === "a" ? 16 : 0;

        try {
            let invalid = {
                0: "x",
                "4294967295": "y",
                length: 4294967296
            };
            Array.prototype.slice.call(invalid, 0, 4294967296);
        } catch (error) {
            total += error && error.name === "RangeError" ? 32 : 0;
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn script_core_array_from_closes_iterators_when_mapper_throws() {
    let result = compile_and_run(
        r#"
        function Test262Error() {}
        let step = 0;
        let closeCount = 0;
        let total = 0;
        let items = {
            [Symbol.iterator]() {
                return {
                    next() {
                        step += 1;
                        if (step === 1) {
                            return { value: "first", done: false };
                        }
                        if (step === 2) {
                            return { value: "second", done: false };
                        }
                        return { done: true };
                    },
                    return() {
                        closeCount += 1;
                        return {};
                    }
                };
            }
        };

        try {
            Array.from(items, function() {
                throw new Test262Error();
            });
        } catch (error) {
            total += (error.constructor === Test262Error ? 1 : 0);
        }

        total += (closeCount === 1 ? 2 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_array_from_async_has_builtin_function_shape() {
    let result = compile_and_run(
        r#"
        let total = 0;
        let descriptor = Object.getOwnPropertyDescriptor(Array, "fromAsync");
        total += typeof Array.fromAsync === "function" ? 1 : 0;
        total += descriptor && descriptor.writable === true ? 2 : 0;
        total += descriptor && descriptor.enumerable === false ? 4 : 0;
        total += descriptor && descriptor.configurable === true ? 8 : 0;
        total += Array.fromAsync.length === 1 ? 16 : 0;
        total += Array.fromAsync.name === "fromAsync" ? 32 : 0;
        total += Object.getPrototypeOf(Array.fromAsync) === Function.prototype ? 64 : 0;
        total += Object.getOwnPropertyDescriptor(Array.fromAsync, "prototype") === undefined ? 128 : 0;
        try {
            new Array.fromAsync();
        } catch (error) {
            total += error && error.name === "TypeError" ? 256 : 0;
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(511));
}

#[test]
fn script_core_supports_fractional_computed_property_keys() {
    let result = compile_and_run(
        r#"
        let obj = { [1.2]: 3 };
        obj['1.2'];
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_supports_exponential_computed_property_keys() {
    let result = compile_and_run(
        r#"
        let obj = { [1e55]: 3 };
        obj['1e+55'];
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_supports_infinity_computed_property_keys() {
    let result = compile_and_run(
        r#"
        let obj = { [Infinity]: 3 };
        obj[Infinity];
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_supports_long_additive_conditional_chains() {
    let result = compile_and_run(
        r#"
        (1 === 1 ? 1 : 0)
            + (2 === 2 ? 2 : 0)
            + (3 === 3 ? 4 : 0)
            + (4 === 4 ? 8 : 0)
            + (5 === 5 ? 16 : 0)
            + (6 === 6 ? 32 : 0)
            + (7 === 7 ? 64 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn script_core_supports_non_index_numeric_computed_property_keys_through_temporaries() {
    let result = compile_and_run(
        r#"
        let obj = {
            [1.2]: 1,
            [1e55]: 2,
            [0.000001]: 3,
            [-0]: 4,
            [Infinity]: 5,
            [-Infinity]: 6,
            [NaN]: 7
        };
        let a = obj['1.2'];
        let b = obj['1e+55'];
        let c = obj['0.000001'];
        let d = obj[0];
        let e = obj[Infinity];
        let f = obj[-Infinity];
        let g = obj[NaN];
        (a === 1 ? 1 : 0)
            + (b === 2 ? 2 : 0)
            + (c === 3 ? 4 : 0)
            + (d === 4 ? 8 : 0)
            + (e === 5 ? 16 : 0)
            + (f === 6 ? 32 : 0)
            + (g === 7 ? 64 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn script_core_supports_fractional_computed_keys_after_full_object_literal() {
    let result = compile_and_run(
        r#"
        let obj = {
            [1.2]: 1,
            [1e55]: 2,
            [0.000001]: 3,
            [-0]: 4,
            [Infinity]: 5,
            [-Infinity]: 6,
            [NaN]: 7
        };
        obj['1.2'];
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_supports_infinity_computed_keys_after_full_object_literal() {
    let result = compile_and_run(
        r#"
        let obj = {
            [1.2]: 1,
            [1e55]: 2,
            [0.000001]: 3,
            [-0]: 4,
            [Infinity]: 5,
            [-Infinity]: 6,
            [NaN]: 7
        };
        obj[Infinity];
        "#,
    );

    assert_eq!(result, Value::from_smi(5));
}

#[test]
fn script_core_reads_default_infinity_global() {
    let result = compile_and_run("Infinity;");

    assert_eq!(result.as_f64(), Some(f64::INFINITY));
}

#[test]
fn script_core_reads_default_nan_global() {
    let result = compile_and_run("NaN;");

    assert!(result.is_nan());
}

#[test]
fn script_core_captures_loop_body_lexicals_in_arrow_without_invoking_closure() {
    let result = compile_and_run_string(
        r#"
        var sink = function (f) { return; };
        const units = ["a", "b"];
        for (let i = 0; i < units.length; i++) {
            const value = units[i];
            sink(() => value);
        }
        "ok";
        "#,
    );

    assert_eq!(result, "ok");
}

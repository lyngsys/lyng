use super::support::{compile_and_run_string, compile_unit};
use lyng_js_common::AtomTable;
use lyng_js_env::{RegExpLegacyStaticText, Runtime};
use lyng_js_host::NoopHostHooks;
use lyng_js_types::Value;
use lyng_js_vm::Vm;

#[test]
fn regexp_scoped_ignore_case_applies_to_backrefs_boundaries_and_properties() {
    let result = compile_and_run_string(
        r#"
        let longS = String.fromCharCode(0x017f);
        [
            /(a)(?i:\1)/.test("aA"),
            /(a)(?-i:\1)/i.test("aA"),
            /(a)(?-i:\1)/i.test("AA"),
            /(?i:\b)/u.test(longS),
            /(?-i:\b)/ui.test(longS),
            /(?i:Z\B)/u.test("Z" + longS),
            /(?-i:Z\B)/ui.test("Z" + longS),
            /(?i:\p{Lu})/u.test("a"),
            /(?i:\P{Lu})/u.test("A")
        ].join("|");
        "#,
    );

    assert_eq!(result, "true|false|true|true|false|true|false|true|true");
}

#[test]
fn regexp_duplicate_named_backrefs_and_escaped_astral_names_match_spec() {
    let result = compile_and_run_string(
        r#"
        function captureEscapedAstralName() {
            try {
                let match = "quick brown fox".match(/(?<\ud835\udcd1\ud835\udcfb\ud835\udcf8\ud835\udd00\ud835\udcf7>brown)/);
                return match.groups["\u{1d4d1}\u{1d4fb}\u{1d4f8}\u{1d500}\u{1d4f7}"];
            } catch (error) {
                return Object.prototype.toString.call(error);
            }
        }

        let simple = /(?:(?<x>a)|(?<x>b))\k<x>/;
        let repeated = /(?:(?:(?<x>a)|(?<x>b))\k<x>){2}/;
        let repeatedMatch = repeated.exec("aabb");

        [
            simple.exec("bb")[0],
            simple.exec("abab") === null,
            repeatedMatch[0],
            repeatedMatch.groups.x,
            repeated.exec("abab") === null,
            captureEscapedAstralName()
        ].join("|");
        "#,
    );

    assert_eq!(result, "bb|true|aabb|b|true|brown");
}

#[test]
fn regexp_duplicate_named_group_alternatives_expose_participating_capture() {
    let result = compile_and_run_string(
        r#"
        function arrayResult(match) {
            return Array.prototype.map.call(match, value => value === undefined ? "U" : value).join(",");
        }

        let simple = /(?:(?:(?<a>x)|(?<a>y))\k<a>){2}/.exec("xxyy");
        let complex = /(?:(?:(?<a>x)|(?<a>y)|(a)|(?<b>b)|(?<a>z))\k<a>){3}/.exec("xzzyyxxy");

        [
            arrayResult(simple),
            simple.groups.a,
            arrayResult(complex),
            complex.groups.a,
            "xxyy".replace(/(?:(?:(?<a>x)|(?<a>y))\k<a>)/, "2$<a>($1,$2)"),
            "xzzyyxxy".replace(/(?:(?:(?<a>x)|(?<a>y)|(a)|(?<b>b)|(?<a>z))\k<a>)/, "2$<a>($1,$2,$3,$4,$5)"),
            "xxyy".replace(/(?:(?:(?<a>x)|(?<a>y))\k<a>)/g, "2$<a>"),
            "xzzyyxxy".replace(/(?:(?:(?<a>x)|(?<a>y)|(a)|(?<b>b)|(?<a>z))\k<a>)/g, "2$<a>")
        ].join("|");
        "#,
    );

    assert_eq!(
        result,
        "xxyy,U,y|y|zzyyxx,x,U,U,U,U|x|2x(x,)yy|x2z(,,,,z)yyxxy|2x2y|x2z2y2xy"
    );
}

#[test]
fn regexp_replace_rejects_oversized_static_replacement_expansion() {
    let result = compile_and_run_string(
        r#"
        function puff(value, length) {
            while (value.length < length) {
                value += value;
            }
            return value.substring(0, length);
        }

        let source = puff("1", 1 << 20);
        let replacement = puff("$1", 1 << 16);
        try {
            source.replace(/(.+)/g, replacement);
            "no throw";
        } catch (error) {
            String(error instanceof RangeError);
        }
        "#,
    );

    assert_eq!(result, "true");
}

#[test]
fn regexp_literal_and_whitespace_fast_paths_preserve_exec_and_replace_semantics() {
    let result = compile_and_run_string(
        r#"
        let literal = /=/g;
        let input = "a=b=c";
        let seen = [];
        let match;
        while ((match = literal.exec(input)) !== null) {
            seen.push(match.index + ":" + literal.lastIndex + ":" + match[0]);
        }

        let literalReplace = input.replace(/=/g, "");
        let whitespaceReplace = "a  \t b\nc".replace(/\s+/g, "_");
        let plus = /\+/.exec("a+b");
        let edgeTrim = " a b ".replace(/^\s+|\s+$/g, "");

        [
            seen.join(","),
            literalReplace,
            whitespaceReplace,
            plus.index + ":" + plus[0],
            edgeTrim,
            RegExp.lastMatch
        ].join("|");
        "#,
    );

    assert_eq!(result, "1:2:=,3:4:=|abc|a_b_c|1:+|a b| ");
}

#[test]
fn regexp_replace_custom_receiver_does_not_probe_exec_before_builtin_path() {
    let result = compile_and_run_string(
        r#"
        var log = "";
        var receiver = {
            get flags() {
                log += "get:flags,";
                return "g";
            },
            set lastIndex(value) {
                log += "set:lastIndex,";
            },
            get exec() {
                log += "get:exec,";
                return function(source) {
                    log += "call:exec,";
                    return null;
                };
            }
        };

        RegExp.prototype[Symbol.replace].call(receiver, "abc", "x");
        log;
        "#,
    );

    assert_eq!(result, "get:flags,set:lastIndex,get:exec,call:exec,");
}

#[test]
fn regexp_unicode_backrefs_do_not_split_surrogate_pairs() {
    let result = compile_and_run_string(
        r#"
        function unit(value) {
            return String.fromCharCode(value);
        }

        let lead = unit(0xD834);
        let trail = unit(0xDC00);
        let re = /foo(.+)bar\1/u;

        function hit(input, whole, capture) {
            let match = re.exec(input);
            return match !== null && match[0] === whole && match[1] === capture;
        }

        [
            hit("fooAbarA" + trail, "fooAbarA", "A"),
            hit("fooAbarA" + lead, "fooAbarA", "A"),
            hit("fooAbarAA", "fooAbarA", "A"),
            hit("fooAbarA", "fooAbarA", "A"),
            re.exec("foo" + lead + "bar" + lead + trail) === null,
            hit("foo" + lead + "bar" + lead + lead, "foo" + lead + "bar" + lead, lead),
            hit("foo" + lead + "bar" + lead + "A", "foo" + lead + "bar" + lead, lead),
            hit("foo" + lead + "bar" + lead, "foo" + lead + "bar" + lead, lead),
            hit("foo" + trail + "bar" + trail + trail, "foo" + trail + "bar" + trail, trail),
            hit("foo" + trail + "bar" + trail + lead, "foo" + trail + "bar" + trail, trail),
            hit("foo" + trail + "bar" + trail + "A", "foo" + trail + "bar" + trail, trail),
            hit("foo" + trail + "bar" + trail, "foo" + trail + "bar" + trail, trail),
            /^(.+)\1$/u.exec(trail + "foobar" + lead + trail + "foobar" + lead) === null
        ].join("|");
        "#,
    );

    assert_eq!(
        result,
        "true|true|true|true|true|true|true|true|true|true|true|true|true"
    );
}

#[test]
fn regexp_raw_surrogate_literals_match_staging_semantics() {
    let result = compile_and_run_string(
        r#"
        function okArray(actual, expected) {
            if (actual === null) return expected === null;
            if (expected === null) return false;
            if (actual.length !== expected.length) return false;
            for (let i = 0; i < actual.length; i++) {
                if (!Object.is(actual[i], expected[i])) return false;
            }
            return true;
        }

        let frog = "\u{1F438}";
        [
            okArray(eval(`/\uD83D\uDC38/u`).exec(frog), [frog]),
            okArray(eval(`/\uD83D\uDC38/`).exec(frog), [frog]),
            eval(`/\\uD83D\uDC38/u`).exec(frog) === null,
            eval(`/\uD83D\\uDC38/u`).exec(frog) === null,
            okArray(eval(`/\\uD83D\uDC38/`).exec(frog), [frog]),
            okArray(eval(`/\uD83D\\uDC38/`).exec(frog), [frog]),
            okArray(new RegExp("\uD83D\uDC38", "u").exec(frog), [frog]),
            okArray(new RegExp("\uD83D\uDC38", "").exec(frog), [frog]),
            new RegExp("\\uD83D\uDC38", "u").exec(frog) === null,
            new RegExp("\uD83D\\uDC38", "u").exec(frog) === null,
            okArray(new RegExp("\\uD83D\uDC38", "").exec(frog), [frog]),
            okArray(new RegExp("\uD83D\\uDC38", "").exec(frog), [frog]),
            okArray(eval(`/\uD83D\uDC38?/u`).exec(frog), [frog]),
            okArray(eval(`/\uD83D\uDC38?/u`).exec(""), [""]),
            okArray(eval(`/\uD83D\uDC38?/u`).exec("\uD83D"), [""]),
            okArray(eval(`/\uD83D\uDC38?/`).exec(frog), [frog]),
            eval(`/\uD83D\uDC38?/`).exec("") === null,
            okArray(eval(`/\uD83D\uDC38?/`).exec("\uD83D"), ["\uD83D"]),
            eval(`/\\uD83D\uDC38?/u`).exec(frog) === null,
            eval(`/\\uD83D\uDC38?/u`).exec("") === null,
            okArray(eval(`/\\uD83D\uDC38?/u`).exec("\uD83D"), ["\uD83D"]),
            eval(`/\uD83D\\uDC38?/u`).exec(frog) === null,
            eval(`/\uD83D\\uDC38?/u`).exec("") === null,
            okArray(eval(`/\uD83D\\uDC38?/u`).exec("\uD83D"), ["\uD83D"]),
            okArray(eval(`/\\uD83D\uDC38?/`).exec(frog), [frog]),
            eval(`/\\uD83D\uDC38?/`).exec("") === null,
            okArray(eval(`/\\uD83D\uDC38?/`).exec("\uD83D"), ["\uD83D"]),
            okArray(eval(`/\uD83D\\uDC38?/`).exec(frog), [frog]),
            eval(`/\uD83D\\uDC38?/`).exec("") === null,
            okArray(eval(`/\uD83D\\uDC38?/`).exec("\uD83D"), ["\uD83D"]),
            okArray(new RegExp("\uD83D\uDC38?", "u").exec(frog), [frog]),
            okArray(new RegExp("\uD83D\uDC38?", "u").exec(""), [""]),
            okArray(new RegExp("\uD83D\uDC38?", "u").exec("\uD83D"), [""]),
            okArray(new RegExp("\uD83D\uDC38?", "").exec(frog), [frog]),
            new RegExp("\uD83D\uDC38?", "").exec("") === null,
            okArray(new RegExp("\uD83D\uDC38?", "").exec("\uD83D"), ["\uD83D"]),
            okArray(eval(`/[\uD83D\uDC38]/u`).exec(frog), [frog]),
            okArray(eval(`/[\uD83D\uDC38]/`).exec(frog), ["\uD83D"]),
            eval(`/[\\uD83D\uDC38]/u`).exec(frog) === null,
            eval(`/[\uD83D\\uDC38]/u`).exec(frog) === null,
            okArray(new RegExp("[\uD83D\uDC38]", "u").exec(frog), [frog]),
            okArray(new RegExp("[\uD83D\uDC38]", "").exec(frog), ["\uD83D"]),
            new RegExp("[\\uD83D\uDC38]", "u").exec(frog) === null,
            new RegExp("[\uD83D\\uDC38]", "u").exec(frog) === null
        ].join("|");
        "#,
    );

    assert!(result.split('|').all(|part| part == "true"), "{result}");
}

#[test]
fn regexp_unicode_literal_fast_paths_preserve_surrogate_and_case_folding_edges() {
    let result = compile_and_run_string(
        r#"
        [
            /\udf06/.exec("\ud834\udf06") !== null,
            /\udf06/u.exec("\ud834\udf06") === null,
            /\udf06/u[Symbol.search]("\ud834\udf06") === -1,
            /\u212a/iu.test("k"),
            /k/iu.test("\u212a"),
            /\u017f/iu.test("S"),
            /s/iu.test("\u017f"),
            /\u00ff/iu.test("\u0178"),
            /\u0178/iu.test("\u00ff")
        ].join("|");
        "#,
    );

    assert_eq!(result, "true|true|true|true|true|true|true|true|true");
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "single conformance trace covers the observable split protocol order"
)]
fn regexp_symbol_split_trace_matches_staging_observability() {
    let result = compile_and_run_string(
        r#"
        var n;
        var log;
        var target;
        var flags;
        var expectedFlags;
        var execResult;
        var lastIndexResult;
        var lastIndexExpected;

        function P(A) {
            return new Proxy(A, {
                get(that, name) {
                    log += "get:result[" + name + "],";
                    return that[name];
                }
            });
        }

        var myRegExp = {
            get constructor() {
                log += "get:constructor,";
                return {
                    get [Symbol.species]() {
                        log += "get:species,";
                        return function(pattern, flagsArg) {
                            if (pattern !== myRegExp) return { bad: "pattern" };
                            if (flagsArg !== expectedFlags) return { bad: "flags:" + flagsArg };
                            log += "call:constructor,";
                            return {
                                get lastIndex() {
                                    log += "get:lastIndex,";
                                    return lastIndexResult[n];
                                },
                                set lastIndex(v) {
                                    log += "set:lastIndex,";
                                    if (v !== lastIndexExpected[n]) log += "bad:lastIndex:" + v + ":" + lastIndexExpected[n] + ",";
                                },
                                get flags() {
                                    log += "get:flags,";
                                    return flags;
                                },
                                get exec() {
                                    log += "get:exec,";
                                    return function(S) {
                                        log += "call:exec,";
                                        if (S !== target) log += "bad:target,";
                                        return execResult[n++];
                                    };
                                },
                            };
                        };
                    }
                };
            },
            get flags() {
                log += "get:flags,";
                return flags;
            },
        };

        function reset() {
            n = 0;
            log = "";
            target = "abcde";
            flags = "";
            expectedFlags = "y";
        }

        function record(label, expectedJson, expectedLog) {
            var ret = RegExp.prototype[Symbol.split].call(myRegExp, target);
            return label + ":" + (JSON.stringify(ret) === expectedJson) + ":" + (log === expectedLog) + ":" + JSON.stringify(ret) + ":" + log;
        }

        reset();
        execResult        = [    null, P(["b"]), null, P(["d"]), null ];
        lastIndexResult   = [ ,  ,     2,        ,     4,        ,    ];
        lastIndexExpected = [ 0, 1,    2,        3,    4,             ];
        var first = record("basic", `["a","c","e"]`,
             "get:constructor," +
             "get:species," +
             "get:flags," +
             "call:constructor," +
             "set:lastIndex,get:exec,call:exec," +
             "set:lastIndex,get:exec,call:exec,get:lastIndex," +
             "get:result[length]," +
             "set:lastIndex,get:exec,call:exec," +
             "set:lastIndex,get:exec,call:exec,get:lastIndex," +
             "get:result[length]," +
             "set:lastIndex,get:exec,call:exec,");

        reset();
        target = "-\uD83D\uDC38\uDC38\uD83D";
        flags = "u";
        expectedFlags = "uy";
        var E = P(["", "X"]);
        execResult        = [    E, E, E, E, E, E, E ];
        lastIndexResult   = [ ,  0, 1, 1, 3, 3, 4, 4 ];
        lastIndexExpected = [ 0, 1, 1, 3, 3, 4, 4,   ];
        var unicode = record("unicode", `["-","X","\uD83D\uDC38","X","\\udc38","X","\\ud83d"]`,
             "get:constructor," +
             "get:species," +
             "get:flags," +
             "call:constructor," +
             "set:lastIndex,get:exec,call:exec,get:lastIndex," +
             "set:lastIndex,get:exec,call:exec,get:lastIndex," +
             "get:result[length]," +
             "get:result[1]," +
             "set:lastIndex,get:exec,call:exec,get:lastIndex," +
             "set:lastIndex,get:exec,call:exec,get:lastIndex," +
             "get:result[length]," +
             "get:result[1]," +
             "set:lastIndex,get:exec,call:exec,get:lastIndex," +
             "set:lastIndex,get:exec,call:exec,get:lastIndex," +
             "get:result[length]," +
             "get:result[1]," +
             "set:lastIndex,get:exec,call:exec,get:lastIndex,");

        first + "\n" + unicode;
        "#,
    );

    assert_eq!(
        result,
        "basic:true:true:[\"a\",\"c\",\"e\"]:get:constructor,get:species,get:flags,call:constructor,set:lastIndex,get:exec,call:exec,set:lastIndex,get:exec,call:exec,get:lastIndex,get:result[length],set:lastIndex,get:exec,call:exec,set:lastIndex,get:exec,call:exec,get:lastIndex,get:result[length],set:lastIndex,get:exec,call:exec,\nunicode:true:true:[\"-\",\"X\",\"\u{1F438}\",\"X\",\"\\udc38\",\"X\",\"\\ud83d\"]:get:constructor,get:species,get:flags,call:constructor,set:lastIndex,get:exec,call:exec,get:lastIndex,set:lastIndex,get:exec,call:exec,get:lastIndex,get:result[length],get:result[1],set:lastIndex,get:exec,call:exec,get:lastIndex,set:lastIndex,get:exec,call:exec,get:lastIndex,get:result[length],get:result[1],set:lastIndex,get:exec,call:exec,get:lastIndex,set:lastIndex,get:exec,call:exec,get:lastIndex,get:result[length],get:result[1],set:lastIndex,get:exec,call:exec,get:lastIndex,"
    );
}

#[test]
fn regexp_group_names_are_readable_through_escaped_identifier_properties() {
    let result = compile_and_run_string(
        r#"
        try {
            let match = "quick brown fox".match(/(?<\u{1d4d1}\u{1d4fb}\u{1d4f8}\u{1d500}\u{1d4f7}>brown)/);
            match === null
                ? "null"
                : match.groups.\u{1d4d1}\u{1d4fb}\u{1d4f8}\u{1d500}\u{1d4f7};
        } catch (error) {
            error.name + ":" + Object.prototype.toString.call(error);
        }
        "#,
    );

    assert_eq!(result, "brown");
}

#[test]
fn regexp_execution_reusable_units_preserve_utf16_and_last_index_edges() {
    let result = compile_and_run_string(
        r#"
        let latin = /a/g;
        latin.exec("ba");
        let latinLast = latin.lastIndex;

        let sticky = /\u0100/y;
        sticky.lastIndex = 1;
        let stickyMatch = sticky.exec("x\u0100");

        let astralMatch = /\u{1F600}/gu.exec("x\u{1F600}");
        let lone = String.fromCharCode(0xD800);
        let loneMatch = /\uD800/g.exec(lone);
        let emptyUnicodeMatches = /(?:)/gu[Symbol.match]("\u{1F600}");

        [
            latinLast,
            stickyMatch.index,
            sticky.lastIndex,
            stickyMatch[0].charCodeAt(0),
            astralMatch.index,
            astralMatch[0].length,
            loneMatch[0].charCodeAt(0),
            emptyUnicodeMatches.length
        ].join("|");
        "#,
    );

    assert_eq!(result, "2|1|2|256|1|2|55296|2");
}

#[test]
fn regexp_literal_cache_keeps_objects_fresh_and_last_index_independent() {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(
        r#"
        function make() {
            return /cache/g;
        }
        let first = make();
        first.lastIndex = 4;
        let second = make();
        second.lastIndex = 9;
        let third = make();
        let total = 0;
        total += (first !== second ? 1 : 0);
        total += (second !== third ? 2 : 0);
        total += (first.lastIndex === 4 ? 4 : 0);
        total += (second.lastIndex === 9 ? 8 : 0);
        total += (third.lastIndex === 0 ? 16 : 0);
        total += (third.test("cache") && third.lastIndex === 5 ? 32 : 0);
        total;
        "#,
        &mut atoms,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    {
        let agent = runtime.root_agent_mut();
        let realm = agent.default_realm().expect("default realm should exist");
        let mut vm = Vm::new();
        let result = vm
            .evaluate_script(agent, realm, &unit)
            .expect("compiled script should execute");

        assert_eq!(result, Value::from_smi(63));
    }

    let accounting = runtime.phase6_accounting();
    assert_eq!(accounting.regexp_literal_cache.records, 1);
    assert!(accounting.regexp_payloads.records >= 3);
}

#[test]
fn regexp_legacy_static_state_stays_lazy_when_global_loop_never_reads_accessors() {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(
        r#"
        (function () {
            let source = "xxxxxxxxxxxxxxxx";
            let rx = /x/g;
            while (rx.exec(source) !== null) {}
        })();
        "#,
        &mut atoms,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    vm.evaluate_script(agent, realm, &unit)
        .expect("compiled script should execute");

    let state = agent
        .regexp_legacy_static_state(realm.id())
        .expect("default realm should have RegExp legacy static state");
    assert!(matches!(
        state.input(),
        RegExpLegacyStaticText::SourceSlice { range, .. } if range == &(0..16)
    ));
    assert!(matches!(
        state.last_match(),
        RegExpLegacyStaticText::SourceSlice { range, .. } if range == &(15..16)
    ));
    assert!(matches!(
        state.right_context(),
        RegExpLegacyStaticText::Empty
    ));
}

#[test]
fn regexp_legacy_static_accessors_survive_collection_after_multiple_matches() {
    let mut atoms = AtomTable::new();
    let record_unit = compile_unit(
        r#"
        (function () {
            let source = "zzababyy";
            let rx = /(a)(b)?/g;
            rx.exec(source);
            rx.exec(source);
        })();
        "#,
        &mut atoms,
    );
    let access_unit = compile_unit(
        r#"
        let total = 0;
        total += (RegExp.input === "zzababyy" ? 1 : 0);
        total += (RegExp.lastMatch === "ab" && RegExp["$&"] === "ab" ? 2 : 0);
        total += (RegExp.leftContext === "zzab" && RegExp["$`"] === "zzab" ? 4 : 0);
        total += (RegExp.rightContext === "yy" && RegExp["$'"] === "yy" ? 8 : 0);
        total += (RegExp.lastParen === "b" && RegExp["$+"] === "b" ? 16 : 0);
        total += (RegExp.$1 === "a" && RegExp.$2 === "b" && RegExp.$9 === "" ? 32 : 0);
        total;
        "#,
        &mut atoms,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    vm.evaluate_script(agent, realm, &record_unit)
        .expect("recording script should execute");
    agent.force_collect();
    let result = vm
        .evaluate_script(agent, realm, &access_unit)
        .expect("accessor script should execute after collection");

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn regexp_symbol_match_default_global_exec_does_not_materialize_per_match_arrays() {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(
        r#"
        let source = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        let matches = /x/g[Symbol.match](source);
        matches.length;
        "#,
        &mut atoms,
    );
    let bootstrap_unit = compile_unit("0;", &mut atoms);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    vm.evaluate_script(agent, realm, &bootstrap_unit)
        .expect("bootstrap script should execute");
    let before_objects = runtime.phase6_accounting().heap.objects.live_bytes;
    let agent = runtime.root_agent_mut();
    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("compiled script should execute");
    let after_objects = runtime.phase6_accounting().heap.objects.live_bytes;

    assert_eq!(result, Value::from_smi(64));
    let object_delta = after_objects - before_objects;
    assert!(
        object_delta < 8_000,
        "default global @@match should not retain one materialized match array per match: {object_delta}"
    );
}

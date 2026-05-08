use super::super::support::*;

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
fn script_core_string_normalize_covers_staging_composition_edges() {
    let result = compile_and_run_string(
        r#"
        let generic = {
            toString() {
                return "a\u0301";
            },
            normalize: String.prototype.normalize
        };

        [
            generic.normalize(),
            "\u0100".normalize("NFD"),
            "A\u0304".normalize()
        ].join("|");
        "#,
    );

    assert_eq!(result, "á|A\u{0304}|Ā");
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

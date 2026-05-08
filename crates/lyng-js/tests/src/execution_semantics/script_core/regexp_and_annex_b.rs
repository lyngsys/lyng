use super::super::support::*;

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
fn script_core_regexp_constructor_uses_internal_slots_and_orders_new_target_lookup() {
    let result = compile_and_run_string(
        r#"
        let regexp = /foo/;
        let accessors = "";
        Object.defineProperty(regexp, "source", {
            get: function() {
                accessors += "source";
                return "bar";
            }
        });
        Object.defineProperty(regexp, "flags", {
            get: function() {
                accessors += "flags";
                return "i";
            }
        });

        let cloned = new RegExp(regexp);

        let order = "";
        let flags = {
            toString: function() {
                order += "flags";
                return "g";
            }
        };
        let newTarget = Object.defineProperty(function(){}.bind(null), "prototype", {
            get: function() {
                order += "prototype";
                return RegExp.prototype;
            }
        });
        let constructed = Reflect.construct(RegExp, [/a/, flags], newTarget);

        [
            cloned.source,
            cloned.flags,
            accessors,
            order,
            constructed.source,
            constructed.flags,
            Object.getPrototypeOf(constructed) === RegExp.prototype
        ].join("|");
        "#,
    );

    assert_eq!(result, "foo|||prototypeflags|a|g|true");
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
fn script_core_regexp_test_accepts_custom_exec_object_receivers() {
    let result = compile_and_run_string(
        r#"
        let receiver = {
            exec: function() {
                return function(){};
            }
        };
        let prototypeThrows = false;
        try {
            RegExp.prototype.test("x");
        } catch (error) {
            prototypeThrows = error instanceof TypeError;
        }

        String(RegExp.prototype.test.call(receiver, "")) + ":" + String(prototypeThrows);
        "#,
    );

    assert_eq!(result, "true:true");
}

#[test]
fn script_core_regexp_exec_observes_last_index_recompile_side_effects() {
    let result = compile_and_run_string(
        r#"
        let matchRegExp = /a/;
        matchRegExp.lastIndex = {
            valueOf() {
                matchRegExp.compile("a", "g");
                return 0;
            }
        };
        matchRegExp[Symbol.match]("a");

        let replaceRegExp = /a/y;
        replaceRegExp.lastIndex = {
            valueOf() {
                replaceRegExp.compile("a", "");
                replaceRegExp.lastIndex = 9000;
                return 0;
            }
        };
        replaceRegExp[Symbol.replace]("a", "");

        String(matchRegExp.lastIndex) + ":" + String(replaceRegExp.lastIndex);
        "#,
    );

    assert_eq!(result, "1:9000");
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
fn script_core_regexp_source_round_trips_line_terminator_after_backslash() {
    let result = compile_and_run_string(
        r#"
        let lineExpected = "\\" + "n";
        let lineMatches = false;
        try {
            let line = RegExp("\\\n");
            lineMatches = line.source === lineExpected
                && eval("/" + line.source + "/").source === lineExpected
                && line.toString() === "/" + lineExpected + "/";
        } catch (error) {
            lineMatches = error instanceof SyntaxError ? "syntax" : "other";
        }

        let classExpected = "[/]";
        let escapedClassExpected = "[" + "\\" + "/" + "]";
        let classLiteral = /[/]/;
        let classConstructor = RegExp("[/]");
        let escapedClassLiteral = /[\/]/;

        String(lineMatches) + ":"
            + String(classLiteral.source === classExpected
                && eval("/" + classLiteral.source + "/").source === classExpected) + ":"
            + String(classConstructor.source === classExpected) + ":"
            + String(escapedClassLiteral.source === escapedClassExpected);
        "#,
    );

    assert_eq!(result, "true:true:true:true");
}

#[test]
fn script_core_regexp_source_preserves_escaped_slashes_for_clones() {
    let result = compile_and_run(
        r#"
        let literal = /<(\/)?([^<>]+)>/;
        let clone = new RegExp(literal, "y");
        let match = clone.exec("</B>");
        let split = "A<B>bold</B>and<CODE>coded</CODE>".split(literal);

        let total = 0;
        total += clone.source === literal.source ? 1 : 0;
        total += match !== null && match[1] === "/" && match[2] === "B" ? 2 : 0;
        total += split.length === 13
            && split[1] === undefined
            && split[2] === "B"
            && split[4] === "/"
            && split[5] === "B"
            && split[7] === undefined
            && split[8] === "CODE"
            && split[10] === "/"
            && split[11] === "CODE" ? 4 : 0;
        total += new RegExp("/").source === "\\/" ? 8 : 0;
        total += new RegExp("\n").source === "\\n" ? 16 : 0;
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
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
fn script_core_regexp_unicode_ignore_case_word_escapes_include_special_folds() {
    let result = compile_and_run_string(
        r#"
        let longS = "\u017F";
        let kelvin = "\u212A";

        [
            /\w/iu.exec("S")?.[0] === "S",
            /\w/iu.exec("s")?.[0] === "s",
            /\w/iu.exec(longS)?.[0] === longS,
            /[^\W]/iu.exec(longS)?.[0] === longS,
            /\W/iu.exec(longS) === null,
            /[^\w]/iu.exec(longS) === null,
            /\w/iu.exec("k")?.[0] === "k",
            /\w/iu.exec(kelvin)?.[0] === kelvin,
            /[^\W]/iu.exec(kelvin)?.[0] === kelvin,
            /\W/iu.exec(kelvin) === null,
            /[^\w]/iu.exec(kelvin) === null
        ].join(":");
        "#,
    );

    assert_eq!(
        result,
        "true:true:true:true:true:true:true:true:true:true:true"
    );
}

#[test]
fn script_core_regexp_ignore_case_literal_folds_cross_latin1() {
    let result = compile_and_run_string(
        r#"
        function hit(re, input, whole, capture) {
            let match = re.exec(input);
            return match !== null && match[0] === whole && match[1] === capture;
        }
        function miss(re, input) {
            return re.exec(input) === null;
        }

        [
            hit(/(\u039C)/i, "\xB5", "\xB5", "\xB5"),
            hit(/(\u039C)+/i, "\xB5\xB5", "\xB5\xB5", "\xB5"),
            hit(/(\xB5)/iu, "\u039C", "\u039C", "\u039C"),
            hit(/(\xB5)+/iu, "\u039C\u039C", "\u039C\u039C", "\u039C"),
            hit(/(\u0178)/i, "\xFF", "\xFF", "\xFF"),
            hit(/(\xFF)+/iu, "\u0178\u0178", "\u0178\u0178", "\u0178"),
            miss(/(\u017F)/i, "\x73"),
            miss(/(\x73)/i, "\u017F"),
            hit(/(\u017F)/iu, "\x73", "\x73", "\x73"),
            hit(/(\x73)+/iu, "\u017F\u017F", "\u017F\u017F", "\u017F"),
            miss(/(\u1E9E)/i, "\xDF"),
            miss(/(\xDF)/i, "\u1E9E"),
            hit(/(\u1E9E)/iu, "\xDF", "\xDF", "\xDF"),
            hit(/(\xDF)+/iu, "\u1E9E\u1E9E", "\u1E9E\u1E9E", "\u1E9E"),
            miss(/(\u212A)/i, "\x6B"),
            miss(/(\x6B)/i, "\u212A"),
            hit(/(\u212A)/iu, "\x6B", "\x6B", "\x6B"),
            hit(/(\x6B)+/iu, "\u212A\u212A", "\u212A\u212A", "\u212A"),
            miss(/(\u212B)/i, "\xE5"),
            miss(/(\xE5)/i, "\u212B"),
            hit(/(\u212B)/iu, "\xE5", "\xE5", "\xE5"),
            hit(/(\xE5)+/iu, "\u212B\u212B", "\u212B\u212B", "\u212B")
        ].join(":");
        "#,
    );

    assert_eq!(
        result,
        "true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true"
    );
}

#[test]
fn script_core_regexp_unicode_braced_surrogate_pairs_compile_as_never_matches() {
    let result = compile_and_run_string(
        r#"
        function probe(source) {
            try {
                return eval(source).exec("\uD83D\uDC38\uDC38") === null;
            } catch (error) {
                return error instanceof SyntaxError ? "syntax" : "other";
            }
        }

        String(probe("/\\u{D83D}\\u{DC38}+/u")) + ":"
            + String(probe("/\\uD83D\\u{DC38}+/u")) + ":"
            + String(probe("/\\u{D83D}\\uDC38+/u"));
        "#,
    );

    assert_eq!(result, "true:true:true");
}

#[test]
fn script_core_regexp_unicode_surrogate_sequences_match_pair_and_invalid_trail() {
    let result = compile_and_run_string(
        r#"
        function matched(source, input, expected) {
            try {
                let result = eval(source).exec(input);
                return result !== null && result[0] === expected;
            } catch (error) {
                return error instanceof SyntaxError ? "syntax" : "other";
            }
        }

        let directPair = /\uD83D\uDC38/u.exec("\u{1F438}");
        let legacyPair = /\uD83D\uDC38/.exec("\u{1F438}");
        let constructedPair = new RegExp("\\uD83D\\uDC38", "u").exec("\u{1F438}");
        let directOptionalPair = /\uD83D\uDC38?/u.exec("\u{1F438}");
        let legacyOptionalPair = /\uD83D\uDC38?/.exec("\u{1F438}");
        let legacyPlusPair = /\uD83D\uDC38+/.exec("\u{1F438}\u{1F438}");
        let legacyStarPair = /\uD83D\uDC38*/.exec("\u{1F438}\u{1F438}");
        let directOptionalEmpty = /\uD83D\uDC38?/u.exec("");

        String(directPair !== null && directPair[0] === "\u{1F438}") + ":"
            + String(legacyPair !== null && legacyPair[0] === "\u{1F438}") + ":"
            + String(constructedPair !== null && constructedPair[0] === "\u{1F438}") + ":"
            + String(directOptionalPair !== null && directOptionalPair[0] === "\u{1F438}") + ":"
            + String(legacyOptionalPair !== null && legacyOptionalPair[0] === "\u{1F438}") + ":"
            + String(legacyPlusPair !== null && legacyPlusPair[0] === "\u{1F438}") + ":"
            + String(legacyStarPair !== null && legacyStarPair[0] === "\u{1F438}") + ":"
            + String(directOptionalEmpty !== null && directOptionalEmpty[0] === "") + ":"
            + String(matched("/\\uD83D\\u3042*/u", "\uD83D\u3042\u3042", "\uD83D\u3042\u3042")) + ":"
            + String(matched("/\\uD83D\\u{3042}*/u", "\uD83D\u3042\u3042", "\uD83D\u3042\u3042")) + ":"
            + String(matched("/\\uD83DA*/u", "\uD83DAA", "\uD83DAA"));
        "#,
    );

    assert_eq!(
        result,
        "true:true:true:true:true:true:true:true:true:true:true"
    );
}

#[test]
fn script_core_regexp_unicode_surrogate_character_classes_match_pair_edges() {
    let result = compile_and_run_string(
        r#"
        let pairClass = /[\uD83D\uDC38]/u;
        let constructedPairClass = new RegExp("[\uD83D\uDC38]", "u");
        let legacyPairClass = /[\uD83D\uDC38]/;
        let legacyConstructedPairClass = new RegExp("[\uD83D\uDC38]", "");
        let leadClass = /[\uD83D]/u;
        let legacyLeadClass = /[\uD83D]/;
        let trailClass = /[\uDC38]/u;
        let legacyTrailClass = /[\uDC38]/;
        let invalidTrailClass = /[\uD83D\u3042]*/u;
        let invalidTrailBracedClass = /[\uD83D\u{3042}]*/u;

        String(pairClass.exec("\uD83D\uDC38")?.[0] === "\uD83D\uDC38") + ":"
            + String(constructedPairClass.exec("\uD83D\uDC38")?.[0] === "\uD83D\uDC38") + ":"
            + String(pairClass.exec("\uD83D") === null) + ":"
            + String(pairClass.exec("\uDC38") === null) + ":"
            + String(legacyPairClass.exec("\uD83D\uDC38")?.[0] === "\uD83D") + ":"
            + String(legacyConstructedPairClass.exec("\uD83D\uDC38")?.[0] === "\uD83D") + ":"
            + String(leadClass.exec("\uD83D\uDC00") === null) + ":"
            + String(legacyLeadClass.exec("\uD83D\uDC00")?.[0] === "\uD83D") + ":"
            + String(trailClass.exec("\uD800\uDC38") === null) + ":"
            + String(legacyTrailClass.exec("\uD800\uDC38")?.[0] === "\uDC38") + ":"
            + String(invalidTrailClass.exec("\uD83D\u3042\u3042\uD83D")?.[0] === "\uD83D\u3042\u3042\uD83D") + ":"
            + String(invalidTrailBracedClass.exec("\uD83D\u3042\u3042\uD83D")?.[0] === "\uD83D\u3042\u3042\uD83D");
        "#,
    );

    assert_eq!(
        result,
        "true:true:true:true:true:true:true:true:true:true:true:true"
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

    assert_eq!(result, Value::from_smi(262_143));
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
        r"
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
        ",
    );

    assert_eq!(result, "undefined:decl:123:decl");
}

#[test]
fn script_core_skips_annex_b_block_function_var_update_for_parameters() {
    let result = compile_and_run_string(
        r"
        var init, after, ifTrue, ifFalse;

        (function(f) {
            init = f;
            {
                function f() { return 'block'; }
            }
            after = f;
        }(123));

        (function(f) {
            if (true) function f() { return 'true-branch'; } else function _f() {}
            ifTrue = f;
        }(456));

        (function(f) {
            if (false) function _f() {} else function f() { return 'false-branch'; }
            ifFalse = f;
        }(789));

        [init, after, ifTrue, ifFalse].join(':');
        ",
    );

    assert_eq!(result, "123:123:456:789");
}

#[test]
fn script_core_supports_annex_b_global_code_block_function_bindings() {
    let result = compile_and_run_string(
        r"
        var before = f === undefined ? 'undefined' : 'other';

        {
            function f() { return 'global'; }
        }

        before + ':' + f();
        ",
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
        r"
        label: function f() { return 'ok'; }
        f();
        ",
    );

    assert_eq!(result, "ok");
}

#[test]
fn script_core_annex_b_block_labeled_function_is_instantiated_before_block_body() {
    let result = compile_and_run_string(
        r"
        var before, after, directValue, labeledValue;

        {
            before = f();
            function f() { return 'direct'; }
            directValue = f();
            label: function f() { return 'labeled'; }
            labeledValue = f();
        }

        after = f();
        [before, directValue, labeledValue, after].join(':');
        ",
    );

    assert_eq!(result, "labeled:labeled:labeled:labeled");
}

#[test]
fn script_core_annex_b_block_labeled_function_creates_var_binding() {
    let result = compile_and_run_string(
        r"
        var before = typeof f;
        var after;

        {
            label: function f() { return 'labeled'; }
        }

        try {
            after = f();
        } catch (error) {
            after = error.name;
        }

        before + ':' + after;
        ",
    );

    assert_eq!(result, "undefined:labeled");
}

#[test]
fn script_core_annex_b_switch_labeled_function_is_instantiated_before_case_body() {
    let result = compile_and_run_string(
        r"
        var before, after;

        switch (1) {
            case 1:
                before = f();
            case 2:
                label: function f() { return 'switch'; }
        }

        after = f();
        before + ':' + after;
        ",
    );

    assert_eq!(result, "switch:switch");
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
fn script_core_annex_b_direct_eval_var_redeclaration_updates_simple_catch_parameter() {
    let result = compile_and_run_string(
        r#"
        var x = "global-x";
        var log = "";

        function g() {
            try {
                throw 8;
            } catch (x) {
                eval("var x = 42;");
                log += x;
            }
            x = "g";
            log += x;
        }

        g();
        x + ":" + log;
        "#,
    );

    assert_eq!(result, "global-x:42g");
}

#[test]
fn script_core_recursive_calls_throw_before_native_stack_overflow() {
    let result = compile_and_run_string(
        r#"
        function recurse() {
            recurse();
        }

        try {
            recurse();
            "missing";
        } catch (error) {
            error.constructor === RangeError ? "true" : "false";
        }
        "#,
    );

    assert_eq!(result, "true");
}

#[test]
fn script_core_named_eval_recursion_uses_bytecode_call_guard() {
    let result = compile_and_run_string(
        r#"
        function eval() {
            eval();
        }
        function callEval() {
            eval();
        }

        try {
            callEval();
            "missing";
        } catch (error) {
            error.constructor === RangeError ? "true" : "false";
        }
        "#,
    );

    assert_eq!(result, "true");
}

#[test]
fn script_core_recursive_generator_resume_uses_vm_depth_guard() {
    let result = compile_and_run_string(
        r#"
        var sawRangeError = false;

        function* f() {
            test();
            yield 170;
        }

        function test() {
            function foopy() {
                try {
                    for (var i of f());
                } catch (error) {
                    sawRangeError = error.constructor === RangeError;
                }
            }
            foopy();
        }

        test();
        sawRangeError ? "true" : "false";
        "#,
    );

    assert_eq!(result, "true");
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

    let expected = ["]", "{", "}", "x{o}x", "16", "2", "b", "e"].join(":");
    assert_eq!(result, expected);
}

#[test]
fn script_core_typeof_before_for_lexical_shadow_returns_undefined() {
    let result = compile_and_run_string(
        r"
        var beforeType;

        beforeType = typeof f;
        for (let f; ; ) {
            {
                function f() {}
            }
            break;
        }

        beforeType;
        ",
    );

    assert_eq!(result, "undefined");
}

#[test]
fn script_core_block_function_named_arguments_shadows_arguments_object() {
    let result = compile_and_run_string(
        r"
        (function() {
            {
                var before = arguments();
                function arguments() { return 'block'; }
                return before + ':' + arguments();
            }
        }());
        ",
    );

    assert_eq!(result, "block:block");
}

#[test]
fn script_core_annex_b_block_function_named_arguments_updates_function_binding() {
    let result = compile_and_run_string(
        r"
        (function() {
            var before = typeof arguments;
            {
                function arguments() { return 'block'; }
            }
            return before + ':' + typeof arguments + ':' + arguments();
        }());
        ",
    );

    assert_eq!(result, "object:function:block");
}

#[test]
fn script_core_global_property_lookup_ignores_unscopables_outside_with() {
    let result = compile_and_run_string(
        r#"
        this.x = "global property x";
        let y = "global lexical y";
        this[Symbol.unscopables] = { x: true, y: true };

        let before = x + ":" + y + ":" + eval("x") + ":" + eval("y");
        {
            let x = "local x";
            with (this) {
                before + ":" + x;
            }
        }
        "#,
    );

    assert_eq!(
        result,
        "global property x:global lexical y:global property x:global lexical y:local x"
    );
}

#[test]
fn script_core_skips_annex_b_block_function_var_binding_for_destructured_catch_conflicts() {
    let result = compile_and_run_string(
        r"
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
        ",
    );

    assert_eq!(result, "undefined:allowed");
}

#[test]
fn script_core_skips_annex_b_block_function_var_binding_for_for_lexical_conflicts() {
    let result = compile_and_run_string(
        r"
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
        ",
    );

    assert_eq!(result, "ref:undefined:loop-ok:ref:undefined");
}

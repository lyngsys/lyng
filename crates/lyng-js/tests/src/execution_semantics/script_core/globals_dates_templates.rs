use super::super::support::*;

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
        r"
        let obj = { total: 0 };
        obj.total = obj.total + 6;
        obj.total;
        ",
    );

    assert_eq!(result, Value::from_smi(6));
}

#[test]
fn script_core_supports_repeated_named_property_updates() {
    let result = compile_and_run(
        r"
        let obj = { total: 0 };
        obj.total = obj.total + 1;
        obj.total = obj.total + 2;
        obj.total;
        ",
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
fn script_core_date_utc_uses_ecmascript_floating_point_operation_order() {
    let result = compile_and_run(
        r"
        let total = 0;
        total += (Date.UTC(1970, 0, 1, 80063993375, 29, 1, -288230376151711740) === 29312 ? 1 : 0);
        total += (Date.UTC(1970, 0, 213503982336, 0, 0, 0, -18446744073709552000) === 34447360 ? 2 : 0);
        total;
        ",
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_date_constructor_has_function_prototype_shape() {
    let result = compile_and_run(
        r"
        (Function.prototype.isPrototypeOf(Date) ? 1 : 0)
            + (Object.getPrototypeOf(Date) === Function.prototype ? 2 : 0);
        ",
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
fn script_core_date_parse_accepts_space_separated_non_iso_forms() {
    let result = compile_and_run_string(
        r#"
        function same(left, right) {
            return Object.is(new Date(left).getTime(), new Date(right).getTime()) ? "1" : "0";
        }

        [
            same("1997-03-08 1:1:1.01", "1997-03-08T01:01:01.01"),
            same("1997-03-08 11:19:20", "1997-03-08T11:19:20"),
            same("1997-3-08 11:19:20", "1997-03-08T11:19:20"),
            same("1997-3-8 11:19:20", "1997-03-08T11:19:20"),
            same("+001997-3-8 11:19:20", "1997-03-08T11:19:20"),
            same("1997-03-08 1:1", "1997-03-08T01:01"),
            same("1997-03-08 11", NaN),
            same("1997-03-08 11:19:10-07", "1997-03-08 11:19:10-0700"),
            same("1997-3-8T11:19:20", NaN),
            same("1997-03-08T1:1:1", NaN)
        ].join("");
        "#,
    );

    assert_eq!(result, "1111111111");
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
        r"
        let score = 0;
        score = score + (parseInt(true) !== parseInt(true) ? 1 : 0);
        score = score + (parseFloat(false) !== parseFloat(false) ? 2 : 0);
        score = score + (isNaN(true) ? 0 : 4);
        score = score + (isFinite(false) ? 8 : 0);
        score;
        ",
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
        r"
        let array = [1, 2, 3];
        (delete array.length ? 100 : 0) + array.length;
        ",
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_delete_returns_true_for_undeclared_global_assignments() {
    let result = compile_and_run(
        r"
        x = 1;
        delete x ? 1 : 0;
        ",
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_delete_returns_true_for_non_reference_unary_chains() {
    let result = compile_and_run(
        r"
        (delete +-~!0 ? 1 : 0)
            + (delete typeof 0 ? 2 : 0)
            + (delete delete 0 ? 4 : 0);
        ",
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn script_core_supports_template_literals_with_primitives() {
    let result = compile_and_run_string(
        r"
        `foo ${5} ${true} ${1n} bar`;
        ",
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
        r"
        let i = 0;
        let first = i++;
        let second = ++i;
        first * 10 + second * 100 + i;
        ",
    );

    assert_eq!(result, Value::from_smi(202));
}

#[test]
fn script_core_supports_template_literal_left_to_right_updates() {
    let result = compile_and_run_string(
        r"
        let i = 0;
        `a${i++}b${i++}c${i++}d`;
        ",
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

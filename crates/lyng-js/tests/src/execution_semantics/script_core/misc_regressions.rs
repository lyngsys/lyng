use super::super::support::*;

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
fn script_core_symbol_prototype_and_wrapper_conversions_match_spec() {
    let result = compile_and_run(
        r#"
        let sym = Symbol("phase6.symbol");
        let score = 0;

        try {
            Symbol.prototype.valueOf();
        } catch (error) {
            score += error instanceof TypeError ? 1 : 0;
        }

        try {
            Object.getOwnPropertyDescriptor(Symbol.prototype, "description").get.call(Symbol.prototype);
        } catch (error) {
            score += error instanceof TypeError ? 2 : 0;
        }

        try {
            String(Object(sym));
        } catch (error) {
            score += error instanceof TypeError ? 4 : 0;
        }

        score += Object(sym).valueOf() === sym ? 8 : 0;
        score += Object.prototype.toString.call(Object(sym)) === "[object Symbol]" ? 16 : 0;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
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
        r"
        ((1 === 1) & (2 === 2)) + (((3 === 4) & (5 === 5)) * 10);
        ",
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_supports_bitwise_or_in_comparator_style_expressions() {
    let result = compile_and_run(
        r"
        ((15 / 4) | 0) - ((3 / 4) | 0);
        ",
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_supports_exponentiation_and_right_associativity() {
    let result = compile_and_run(
        r"
        (2 ** 5) + (2 ** 3 ** 2) + (-(2 ** 4));
        ",
    );

    assert_eq!(result, Value::from_smi(528));
}

#[test]
fn script_core_treats_for_in_over_nullish_values_as_empty() {
    let result = compile_and_run(
        r"
        let seen = 0;
        for (var key in undefined) {
            seen = seen + 100;
        }
        for (var other in null) {
            seen = seen + 100;
        }
        seen;
        ",
    );

    assert_eq!(result, Value::from_smi(0));
}

#[test]
fn script_core_supports_non_index_numeric_computed_property_keys() {
    let result = compile_and_run(
        r"
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
        ",
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
fn script_core_array_from_does_not_close_when_iterator_value_throws() {
    let result = compile_and_run(
        r#"
        let closed = 0;
        let threw = 0;
        let items = {
            [Symbol.iterator]() {
                return {
                    next() {
                        return {
                            get value() {
                                throw "value";
                            },
                            done: false
                        };
                    },
                    return() {
                        closed += 1;
                        return {};
                    }
                };
            }
        };

        try {
            Array.from(items);
        } catch (error) {
            threw = error === "value" ? 1 : 0;
        }

        threw + closed * 2;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_array_from_array_like_constructs_before_reading_elements() {
    let result = compile_and_run_string(
        r#"
        let log = "";
        let constructed;
        function C(length) {
            log += "C" + length;
            constructed = this;
        }
        let source = {
            get length() { log += "l"; return 1; },
            get 0() { log += "0"; return "value"; }
        };
        let thrown = { marker: true };

        try {
            Array.from.call(C, source, () => { throw thrown; });
            log += "no-throw";
        } catch (error) {
            log += error === thrown ? "!" : "?";
        }

        log + ":" + String(constructed instanceof C);
        "#,
    );

    assert_eq!(result, "lC10!:true");
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
        r"
        let obj = { [1.2]: 3 };
        obj['1.2'];
        ",
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_supports_exponential_computed_property_keys() {
    let result = compile_and_run(
        r"
        let obj = { [1e55]: 3 };
        obj['1e+55'];
        ",
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_supports_infinity_computed_property_keys() {
    let result = compile_and_run(
        r"
        let obj = { [Infinity]: 3 };
        obj[Infinity];
        ",
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_supports_long_additive_conditional_chains() {
    let result = compile_and_run(
        r"
        (1 === 1 ? 1 : 0)
            + (2 === 2 ? 2 : 0)
            + (3 === 3 ? 4 : 0)
            + (4 === 4 ? 8 : 0)
            + (5 === 5 ? 16 : 0)
            + (6 === 6 ? 32 : 0)
            + (7 === 7 ? 64 : 0);
        ",
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn script_core_supports_non_index_numeric_computed_property_keys_through_temporaries() {
    let result = compile_and_run(
        r"
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
        ",
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn script_core_supports_fractional_computed_keys_after_full_object_literal() {
    let result = compile_and_run(
        r"
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
        ",
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_supports_infinity_computed_keys_after_full_object_literal() {
    let result = compile_and_run(
        r"
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
        ",
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

#[test]
fn script_core_lowers_sibling_block_class_declarations_with_same_name() {
    let result = compile_and_run_string(
        r#"
        let values = [];
        for (let value of [1, 2]) {
            class Local {
                static read() { return value; }
            }
            values.push(Local.read());
        }
        for (let value of [3]) {
            class Local {
                static read() { return value; }
            }
            values.push(Local.read());
        }
        values.join(",");
        "#,
    );

    assert_eq!(result, "1,2,3");
}

#[test]
fn script_core_lowers_loop_body_class_constructor_with_rest_parameter() {
    let result = compile_and_run_string(
        r#"
        let values = [];
        for (let Base of [
            class {
                constructor(...params) {
                    this.value = params.length;
                }
            },
            class {
                constructor(...params) {
                    this.value = params.length;
                }
            }
        ]) {
            let enabled = false;
            class Local extends Base {
                constructor(...params) {
                    super(...params);
                    if (enabled) {
                        values.push(this.value);
                    }
                }
            }
            enabled = true;
            new Local(1, 2);
        }
        values.join(",");
        "#,
    );

    assert_eq!(result, "2,2");
}

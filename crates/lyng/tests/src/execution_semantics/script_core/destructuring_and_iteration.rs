use super::super::support::*;

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
        r"
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
        ",
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_does_not_require_iterator_next_before_assignment_reference_suspends() {
    let result = compile_and_run(
        r"
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
        ",
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
        r"
        let target = { value: 0 };
        for ({ value: target.value } of [{ value: 7 }]) {}
        target.value;
        ",
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
        r"
        var fn;
        (fn) = function() {};
        fn.name;
        ",
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
        r"
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
        ",
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
fn script_core_assigns_names_to_for_in_var_initializer_anonymous_functions() {
    let result = compile_and_run_string(
        r"
        for (var forInHead = function() {} in {}) {
        }
        forInHead.name;
        ",
    );

    assert_eq!(result, "forInHead");
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
        r"
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
        ",
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
        r"
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
        ",
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_supports_for_of_object_shorthand_defaults() {
    let result = compile_and_run(
        r"
        var x = 0;
        for ({ x = 1 } of [{}]) {}
        x;
        ",
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_supports_for_of_object_rest_assignment_heads() {
    let result = compile_and_run(
        r"
        let rest = null;
        for ({ value: ignored, ...rest } of [{ value: 1, other: 2 }]) {}
        rest.other;
        ",
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
        r"
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
        ",
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_requires_object_coercible_values_for_for_of_object_rest() {
    let result = compile_and_run(
        r"
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
        ",
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_for_of_destructuring_assignments_respect_global_lexical_tdz() {
    let result = compile_and_run(
        r"
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
        ",
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

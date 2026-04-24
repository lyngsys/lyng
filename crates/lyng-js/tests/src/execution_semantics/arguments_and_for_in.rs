use super::support::compile_and_run;
use lyng_js_types::Value;

#[test]
fn phase4_arguments_map_simple_sloppy_parameters() {
    let result = compile_and_run(
        r#"
        function alias(a, b) {
            a = 7;
            arguments[1] = 9;
            return arguments[0] + b;
        }
        alias(1, 2);
        "#,
    );

    assert_eq!(result, Value::from_smi(16));
}

#[test]
fn phase4_arguments_use_unmapped_objects_in_strict_functions() {
    let result = compile_and_run(
        r#"
        function frozen(a) {
            "use strict";
            a = 7;
            return arguments[0] + arguments.length;
        }
        frozen(1, 2);
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase4_rest_parameters_materialize_fresh_arrays() {
    let result = compile_and_run(
        r#"
        function collect(head, ...rest) {
            rest[1] = rest[1] + 1;
            return rest[0] + rest[1] + rest.length;
        }
        collect(1, 2, 3);
        "#,
    );

    assert_eq!(result, Value::from_smi(8));
}

#[test]
fn phase4_for_in_enumerates_ordinary_object_keys() {
    let result = compile_and_run(
        r#"
        let obj = { alpha: 1, beta: 2, gamma: 3 };
        let total = 0;
        for (var key in obj) {
            total = total + obj[key];
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(6));
}

#[test]
fn phase4_for_in_accepts_named_function_expression_sources() {
    let result = compile_and_run(
        r#"
        let key = "";
        for (key in function named() { return { a: 1 }; }()) {
        }
        key === "a" ? 1 : 0;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn phase4_for_of_closures_observe_in_iteration_reassignments() {
    let result = compile_and_run(
        r#"
        let closures = [];
        for (let value of [1, 2]) {
            closures.push(function() { return value; });
            value = value + 10;
        }
        closures[0]() + closures[1]();
        "#,
    );

    assert_eq!(result, Value::from_smi(23));
}

#[test]
fn phase4_for_of_closures_ignore_unrelated_var_declarations() {
    let result = compile_and_run(
        r#"
        let closures = [];
        for (let value of [1, 2]) {
            var scratch = value;
            closures.push(function() { return value; });
        }
        closures[0]() + closures[1]() + scratch;
        "#,
    );

    assert_eq!(result, Value::from_smi(5));
}

#[test]
fn phase4_for_in_closures_preserve_body_lexicals_per_iteration() {
    let result = compile_and_run(
        r#"
        let closures = [];
        for (let key in { alpha: 0, beta: 0 }) {
            let extra = key === "alpha" ? 10 : 20;
            closures.push(function() {
                return (key === "alpha" ? 100 : 200) + extra;
            });
            extra = extra + 1;
        }
        closures[0]() * 1000 + closures[1]();
        "#,
    );

    assert_eq!(result, Value::from_smi(111221));
}

#[test]
fn phase4_for_in_closures_preserve_const_loop_binding_immutability() {
    let result = compile_and_run(
        r#"
        let caught = 0;
        let closures = [];
        for (const key in { alpha: 0 }) {
            closures.push(function() {
                key = "mutated";
                return key;
            });
        }
        try {
            closures[0]();
        } catch (error) {
            caught = 1;
        }
        caught;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn phase4_for_of_closures_keep_shared_outer_bindings_live() {
    let result = compile_and_run(
        r#"
        let shared = 100;
        let closures = [];
        for (let value of [1, 2]) {
            closures.push(function() {
                shared = shared + 1;
                return value + shared;
            });
        }
        shared = 10;
        closures[0]() * 1000 + closures[1]();
        "#,
    );

    assert_eq!(result, Value::from_smi(12014));
}

#[test]
fn phase4_for_of_closures_snapshot_body_bindings_without_loop_binding_captures() {
    let result = compile_and_run(
        r#"
        let closures = [];
        for (let value of [1, 2]) {
            let local = value * 10;
            closures.push(function() { return local; });
            local = local + 1;
        }
        closures[0]() * 100 + closures[1]();
        "#,
    );

    assert_eq!(result, Value::from_smi(1121));
}

#[test]
fn phase4_nested_for_of_closure_captures_preserve_undefined_iteration_values() {
    let result = compile_and_run(
        r#"
        let result = 0;
        for (const outer of [undefined]) {
            for (const inner of [undefined]) {
                let callback = () => (outer === undefined ? 1 : 0) + (inner === undefined ? 2 : 0);
                try {
                    result = callback();
                } catch (error) {
                    result = -1;
                }
            }
        }
        result;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase4_for_of_over_mapped_arguments_observes_alias_updates() {
    let result = compile_and_run(
        r#"
        let expected = [1, 3, 1];
        let index = 0;
        let ok = 1;

        (function(a, b, c) {
            for (let value of arguments) {
                a = b;
                b = c;
                c = index;
                if (value !== expected[index]) {
                    ok = 0;
                }
                index = index + 1;
            }
        }(1, 2, 3));

        ok * 10 + index;
        "#,
    );

    assert_eq!(result, Value::from_smi(13));
}

#[test]
fn phase6_mapped_arguments_descriptors_reflect_current_parameter_values() {
    let result = compile_and_run(
        r#"
        function sample(a) {
            a = 7;
            return Object.getOwnPropertyDescriptor(arguments, "0").value;
        }
        sample(1);
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase6_sloppy_arguments_define_data_callee_descriptor() {
    let result = compile_and_run(
        r#"
        function sample() {
            let descriptor = Object.getOwnPropertyDescriptor(arguments, "callee");
            if (descriptor === undefined) {
                return 0;
            }
            return (descriptor.value === sample ? 1 : 0)
                + (descriptor.writable === true ? 2 : 0)
                + (descriptor.enumerable === false ? 4 : 0)
                + (descriptor.configurable === true ? 8 : 0);
        }
        sample(1, 2);
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn phase6_unmapped_arguments_share_realm_throw_type_error() {
    let result = compile_and_run(
        r#"
        let thrower = Object.getOwnPropertyDescriptor(function() {
            "use strict";
            return arguments;
        }(), "callee").get;

        function nonSimple(a = 0) {
            return arguments;
        }

        let descriptor = Object.getOwnPropertyDescriptor(nonSimple(), "callee");
        let total = 0;
        total += descriptor.get === thrower ? 1 : 0;
        total += descriptor.set === thrower ? 2 : 0;
        total += descriptor.enumerable === false ? 4 : 0;
        total += descriptor.configurable === false ? 8 : 0;
        total += Object.getPrototypeOf(thrower) === Function.prototype ? 16 : 0;
        try {
            descriptor.get();
        } catch (error) {
            total += error.constructor === TypeError ? 32 : 64;
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn phase6_define_property_value_updates_mapped_arguments_parameters() {
    let result = compile_and_run(
        r#"
        function sample(a) {
            Object.defineProperty(arguments, "0", {
                value: 2,
                writable: true,
                enumerable: true,
                configurable: true,
            });
            return (a === 2 ? 1 : 0) + (arguments[0] === 2 ? 2 : 0);
        }
        sample(1);
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase6_define_property_accessor_detaches_mapped_arguments_indices() {
    let result = compile_and_run(
        r#"
        function sample(a) {
            let setCalls = 0;
            Object.defineProperty(arguments, "0", {
                set(_value) { setCalls = setCalls + 1; },
                enumerable: true,
                configurable: true,
            });
            arguments[0] = 9;
            return setCalls + (a === 1 ? 10 : 0);
        }
        sample(1);
        "#,
    );

    assert_eq!(result, Value::from_smi(11));
}

#[test]
fn phase6_failed_delete_keeps_nonconfigurable_mapped_arguments_indices_attached() {
    let result = compile_and_run(
        r#"
        function sample(a) {
            Object.defineProperty(arguments, "0", { configurable: false });
            let deleted = delete arguments[0];
            arguments[0] = 5;
            return (deleted ? 0 : 1) + (a === 5 ? 2 : 0) + (arguments[0] === 5 ? 4 : 0);
        }
        sample(1);
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase6_strict_delete_of_nonconfigurable_mapped_arguments_throws() {
    let result = compile_and_run(
        r#"
        function sample(a) {
            let threw = 0;
            Object.defineProperty(arguments, "0", { configurable: false });
            let args = arguments;
            try {
                (function() {
                    "use strict";
                    delete args[0];
                }());
            } catch (error) {
                threw = error.constructor === TypeError ? 1 : 0;
            }
            a = 2;
            return threw + (arguments[0] === 2 ? 2 : 0);
        }
        sample(1);
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase6_failed_define_property_keeps_arguments_objects_consistent() {
    let result = compile_and_run(
        r#"
        function expectTypeError(callback) {
            try {
                callback();
            } catch (error) {
                return error.constructor === TypeError ? 1 : 0;
            }
            return 0;
        }

        function sample(a) {
            let score = 0;

            Object.defineProperty(arguments, "0", { configurable: false });
            score += expectTypeError(() => {
                Object.defineProperty(arguments, "0", { configurable: true });
            });
            a = 2;
            score += arguments[0] === 2 ? 2 : 0;

            Object.defineProperty(arguments, "1", {
                get: () => 3,
                configurable: false,
            });
            score += expectTypeError(() => {
                Object.defineProperty(arguments, "1", { value: "foo" });
            }) ? 4 : 0;
            score += arguments[1] === 3 ? 8 : 0;

            score += expectTypeError(() => {
                "use strict";
                delete arguments[1];
            }) ? 16 : 0;

            return score;
        }
        sample(0);
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn phase6_strict_delete_through_arrow_callback_uses_outer_arguments_object() {
    let result = compile_and_run(
        r#"
        function invoke(callback) {
            return callback();
        }

        function sample(a) {
            let threw = 0;
            Object.defineProperty(arguments, "1", {
                get: () => 3,
                configurable: false,
            });
            try {
                invoke(() => {
                    "use strict";
                    delete arguments[1];
                });
            } catch (error) {
                threw = error.constructor === TypeError ? 1 : 0;
            }
            return threw + (arguments[1] === 3 ? 2 : 0);
        }

        sample(0);
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

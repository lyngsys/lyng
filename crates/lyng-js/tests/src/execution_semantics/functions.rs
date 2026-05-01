use super::support::{
    compile_and_run, compile_and_run_string, compile_unit, evaluate_with_registry,
    install_native_global,
};
use lyng_js_common::AtomTable;
use lyng_js_gc::{AllocationLifetime, PrimitiveMutator};
use lyng_js_objects::{
    InternalMethodResult, NativeCallRequest, NativeConstructRequest, NativeFunctionRegistry,
    ObjectAllocation, ObjectRuntime,
};
use lyng_js_types::{
    BuiltinFunctionId, EnvironmentRef, NativeFunctionId, ObjectRef, RealmRef, Value,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct RecordedNativeCall {
    callee: ObjectRef,
    arguments: Vec<Value>,
    realm: RealmRef,
    environment: EnvironmentRef,
    entry: NativeFunctionId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RecordedNativeConstruct {
    callee: ObjectRef,
    new_target: ObjectRef,
    arguments: Vec<Value>,
    realm: RealmRef,
    environment: EnvironmentRef,
    entry: NativeFunctionId,
}

#[derive(Default)]
struct RecordingRegistry {
    last_call: Option<RecordedNativeCall>,
    last_construct: Option<RecordedNativeConstruct>,
}

impl NativeFunctionRegistry for RecordingRegistry {
    fn call(
        &mut self,
        _runtime: &mut ObjectRuntime,
        _heap: &mut PrimitiveMutator<'_>,
        request: NativeCallRequest<'_>,
    ) -> InternalMethodResult<Value> {
        self.last_call = Some(RecordedNativeCall {
            callee: request.callee(),
            arguments: request.arguments().to_vec(),
            realm: request.realm(),
            environment: request.environment(),
            entry: request.entry(),
        });
        Ok(Value::from_smi(77))
    }

    fn construct(
        &mut self,
        runtime: &mut ObjectRuntime,
        heap: &mut PrimitiveMutator<'_>,
        request: NativeConstructRequest<'_>,
    ) -> InternalMethodResult<ObjectRef> {
        self.last_construct = Some(RecordedNativeConstruct {
            callee: request.callee(),
            new_target: request.new_target(),
            arguments: request.arguments().to_vec(),
            realm: request.realm(),
            environment: request.environment(),
            entry: request.entry(),
        });
        let root_shape = runtime.root_shape(heap, None, AllocationLifetime::Default);
        Ok(runtime.alloc_object(
            heap,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        ))
    }
}

#[test]
fn phase4_functions_hoist_declarations_and_return_across_frames() {
    let result = compile_and_run(
        r#"
        run();
        function run() {
            return 7;
        }
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase4_functions_execute_anonymous_function_expressions() {
    let result = compile_and_run(
        r#"
        let twice = function(value) {
            return value + value;
        };
        twice(5);
        "#,
    );

    assert_eq!(result, Value::from_smi(10));
}

#[test]
fn phase4_functions_capture_outer_bindings_through_closures() {
    let result = compile_and_run(
        r#"
        function outer(base) {
            return function(step) {
                return base + step;
            };
        }
        let add = outer(40);
        add(2);
        "#,
    );

    assert_eq!(result, Value::from_smi(42));
}

#[test]
fn phase4_functions_capture_top_level_lexicals_across_intermediate_frames() {
    let result = compile_and_run(
        r#"
        let base = 40;
        function outer(step) {
            return function(delta) {
                return base + step + delta;
            };
        }
        let add = outer(1);
        add(2);
        "#,
    );

    assert_eq!(result, Value::from_smi(43));
}

#[test]
fn phase4_functions_allow_block_lexicals_to_shadow_parameters() {
    let result = compile_and_run(
        r#"
        function read(value) {
            let total = value;
            {
                let value = 2;
                total = total + value;
            }
            return total;
        }
        read(1);
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase4_functions_allow_named_function_expressions_with_shadowed_bindings() {
    let result = compile_and_run(
        r#"
        var probeVar;
        var probeLex;
        var fromVar = function named() {
            var named;
            probeVar = function() { return named; };
        };
        var fromLex = function named() {
            let named = "inside";
            probeLex = function() { return named; };
        };
        fromVar();
        fromLex();
        (probeVar() === undefined ? 1 : 0) + (probeLex() === "inside" ? 2 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase4_functions_preserve_constructor_result_and_new_target() {
    let result = compile_and_run(
        r#"
        function Thing(value) {
            this.value = value;
            this.constructed = new.target ? 1 : 0;
            return 99;
        }
        let thing = new Thing(6);
        thing.value + thing.constructed;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase5_functions_install_constructor_backlinks_on_instance_prototypes() {
    let result = compile_and_run(
        r#"
        function Test262Error() {}
        let instance = new Test262Error();
        (instance.constructor === Test262Error ? 1 : 0)
            + (Test262Error.prototype.constructor === Test262Error ? 2 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase4_functions_keep_arrow_lexical_this_across_call_boundaries() {
    let result = compile_and_run(
        r#"
        function Box(value) {
            this.value = value;
            this.read = () => this.value;
        }
        let box = new Box(8);
        let read = box.read;
        read();
        "#,
    );

    assert_eq!(result, Value::from_smi(8));
}

#[test]
fn phase4_functions_support_function_prototype_call_with_primitive_this() {
    let result = compile_and_run(
        r#"
        function strictProbe() {
            "use strict";
            return typeof this === "number" ? 1 : 0;
        }
        function sloppyProbe() {
            return typeof this === "object" ? 2 : 0;
        }
        strictProbe.call(1) + sloppyProbe.call("box");
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase5_functions_support_function_prototype_apply_with_array_like_arguments() {
    let result = compile_and_run(
        r#"
        function add(a, b) {
            return this.base + a + b;
        }
        add.apply({ base: 10 }, { length: 2, 0: 4, 1: 5 });
        "#,
    );

    assert_eq!(result, Value::from_smi(19));
}

#[test]
fn phase5_functions_support_bound_function_calls() {
    let result = compile_and_run(
        r#"
        function add(a, b) {
            return this.base + a + b;
        }
        let bound = add.bind({ base: 10 }, 4);
        bound(5);
        "#,
    );

    assert_eq!(result, Value::from_smi(19));
}

#[test]
fn phase5_functions_support_bound_function_construction() {
    let result = compile_and_run(
        r#"
        function Add(a, b) {
            this.total = a + b;
        }
        Add.prototype.bump = 7;
        let Bound = Add.bind(null, 3);
        let instance = new Bound(5);
        instance.total + instance.bump;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn phase5_functions_bind_installs_restricted_caller_and_arguments_accessors() {
    let result = compile_and_run(
        r#"
        function target() {}
        let bound = target.bind(null);
        let total = 0;

        if (bound.hasOwnProperty("caller") === false) {
            total += 1;
        }
        if (bound.hasOwnProperty("arguments") === false) {
            total += 2;
        }
        try {
            bound.caller;
        } catch (error) {
            if (error.constructor === TypeError) {
                total += 4;
            }
        }
        try {
            bound.arguments = {};
        } catch (error) {
            if (error.constructor === TypeError) {
                total += 8;
            }
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn phase6_generator_functions_inherit_restricted_caller_and_arguments_accessors() {
    let result = compile_and_run(
        r#"
        function* generator() {}
        let total = 0;

        if (generator.hasOwnProperty("caller") === false) {
            total += 1;
        }
        if (generator.hasOwnProperty("arguments") === false) {
            total += 2;
        }
        try {
            generator.caller;
        } catch (error) {
            if (error.constructor === TypeError) {
                total += 4;
            }
        }
        try {
            generator.arguments = {};
        } catch (error) {
            if (error.constructor === TypeError) {
                total += 8;
            }
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn phase6_legacy_function_caller_respects_immediate_caller_strictness() {
    let result = compile_and_run(
        r#"
        function nonStrictCaller() {
            return target();
        }
        function target() {
            return target.caller === nonStrictCaller ? 1 : 0;
        }
        function strictCaller() {
            "use strict";
            return target();
        }

        let total = nonStrictCaller();
        try {
            strictCaller();
        } catch (error) {
            if (error.constructor === TypeError) {
                total += 2;
            }
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase5_functions_bind_uses_spec_length_and_name_rules() {
    let result = compile_and_run(
        r#"
        function target(a, b, c) {}
        let total = 0;

        Object.defineProperty(target, "length", { value: 3.8 });
        total += target.bind(null, 1).length === 2 ? 1 : 0;

        Object.defineProperty(target, "length", { value: Infinity });
        total += target.bind(null, 1).length === Infinity ? 2 : 0;

        Object.defineProperty(target, "length", { value: "3" });
        total += target.bind(null, 1).length === 0 ? 4 : 0;

        Object.defineProperty(target, "name", { value: 7 });
        total += target.bind(null).name === "bound " ? 8 : 0;

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn phase5_functions_apply_rejects_primitive_arg_arrays_and_preserves_abrupt_getters() {
    let result = compile_and_run(
        r#"
        function noop() {}
        let total = 0;
        let marker = {};

        try {
            noop.apply(null, true);
        } catch (error) {
            if (error.constructor === TypeError) {
                total += 1;
            }
        }

        try {
            noop.apply(null, {
                get length() {
                    throw marker;
                }
            });
        } catch (error) {
            if (error === marker) {
                total += 2;
            }
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase5_functions_bind_preserves_name_getter_abrupt_completion() {
    let result = compile_and_run(
        r#"
        function target() {}
        let marker = {};
        let total = 0;

        Object.defineProperty(target, "name", {
            get: function() {
                throw marker;
            }
        });

        try {
            target.bind(null);
        } catch (error) {
            if (error === marker) {
                total += 1;
            }
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn phase5_boolean_wrapper_loose_equality_uses_to_primitive() {
    let result = compile_and_run(
        r#"
        (new Boolean(true) == true ? 1 : 0)
            + (new Boolean(false) == false ? 2 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase5_boolean_prototype_carries_false_wrapper_value() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += Boolean.prototype.valueOf() === false ? 1 : 0;
        total += Boolean.prototype.toString() === "false" ? 2 : 0;
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase6_wrapper_completion_handles_number_object_wrappers() {
    let result = compile_and_run(
        r#"
        (Object(7) == 7 ? 1 : 0) + ("" + Object(7) === "7" ? 2 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase6_wrapper_completion_handles_string_object_wrappers() {
    let result = compile_and_run(
        r#"
        (Object("abc") == "abc" ? 1 : 0) + ("" + Object("abc") === "abc" ? 2 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase5_functions_compile_dynamic_function_bodies_through_function_constructor() {
    let result = compile_and_run(
        r#"
        let add = Function("a", "b", "return a + b;");
        let seven = new Function("return 7;");
        add(2, 3) + seven();
        "#,
    );

    assert_eq!(result, Value::from_smi(12));
}

#[test]
fn phase5_functions_expose_length_descriptors_on_runtime_function_objects() {
    let result = compile_and_run(
        r#"
        function sum(a, b, c) { return a + b + c; }
        let dynamic = Function("x", "y", "return x + y;");
        let first = Object.getOwnPropertyDescriptor(sum, "length");
        let second = Object.getOwnPropertyDescriptor(dynamic, "length");
        let total = 0;
        if (first && first.value === 3 && first.writable === false && first.enumerable === false && first.configurable === true) {
            total += 1;
        }
        if (second && second.value === 2 && second.writable === false && second.enumerable === false && second.configurable === true) {
            total += 2;
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase5_functions_do_not_capture_outer_lexical_scope_in_function_constructor() {
    let result = compile_and_run(
        r#"
        var globalValue = 6;
        function outer() {
            let hidden = 20;
            return Function("return globalValue + (typeof hidden === 'undefined' ? 1 : 100);")();
        }
        outer();
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase5_function_prototype_is_callable_but_not_constructible() {
    let result = compile_and_run(
        r#"
        try {
            new Function.prototype();
            0;
        } catch (_) {
            (Function.prototype() === undefined ? 1 : 0) + 2;
        }
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase6_function_symbol_has_instance_matches_ordinary_has_instance() {
    let result = compile_and_run(
        r#"
        function C() {}
        let c = new C();
        let descriptor = Object.getOwnPropertyDescriptor(Function.prototype, Symbol.hasInstance);
        let total = 0;
        total += typeof descriptor.value === "function" ? 1 : 0;
        total += descriptor.writable === false ? 2 : 0;
        total += descriptor.enumerable === false ? 4 : 0;
        total += descriptor.configurable === false ? 8 : 0;
        total += descriptor.value.length === 1 ? 16 : 0;
        total += descriptor.value.name === "[Symbol.hasInstance]" ? 32 : 0;
        total += C[Symbol.hasInstance](c) ? 64 : 0;
        total += !C[Symbol.hasInstance]({}) ? 128 : 0;
        total += C.bind()[Symbol.hasInstance](c) ? 256 : 0;

        let marker = {};
        let proxy = new Proxy(Object.create(C.prototype), {
            getPrototypeOf() {
                throw marker;
            }
        });
        try {
            C[Symbol.hasInstance](proxy);
        } catch (error) {
            total += error === marker ? 512 : 0;
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(1023));
}

#[test]
fn phase5_functions_to_string_preserves_source_function_text() {
    let result =
        compile_and_run_string("function sample(value) { return value + 1; } sample.toString();");

    assert_eq!(result, "function sample(value) { return value + 1; }");
}

#[test]
fn phase5_functions_to_string_preserves_dynamic_function_text() {
    let result = compile_and_run_string(r#"Function("a", "b", "return a + b;").toString();"#);

    assert_eq!(result, "function anonymous(a,b\n) {\nreturn a + b;\n}");
}

#[test]
fn phase5_functions_to_string_formats_native_functions_stably() {
    let result = compile_and_run_string("Function.prototype.toString();");

    assert_eq!(result, "function () { [native code] }");
}

#[test]
fn phase6_function_to_string_formats_regexp_species_getter() {
    let result = compile_and_run_string(
        "Object.getOwnPropertyDescriptor(RegExp, Symbol.species).get.toString();",
    );

    assert_eq!(result, "function get [Symbol.species]() { [native code] }");
}

#[test]
fn phase4_functions_support_string_replace_callback_this_binding() {
    let result = compile_and_run(
        r#"
        let strictThis;
        let sloppyThis;
        function strictReplacement() {
            "use strict";
            strictThis = this;
            return "a";
        }
        function sloppyReplacement() {
            sloppyThis = this;
            return "a";
        }
        ("ab".replace("b", strictReplacement) === "aa" ? 1 : 0)
            + ("ab".replace("b", sloppyReplacement) === "aa" ? 2 : 0)
            + (strictThis === undefined ? 4 : 0)
            + (sloppyThis === this ? 8 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn phase4_functions_support_nested_internal_bytecode_callbacks() {
    let result = compile_and_run(
        r#"
        function plusOne(value) {
            return value + 1;
        }
        "ab".replace("b", function() {
            return plusOne.call(undefined, 4) + "";
        }) === "a5" ? 1 : 0;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn phase4_functions_expand_spread_arguments_for_calls_and_constructs() {
    let call_entry = BuiltinFunctionId::from_raw(1).unwrap();
    let construct_entry = BuiltinFunctionId::from_raw(2).unwrap();
    let mut atoms = AtomTable::new();
    let unit = compile_unit(
        r#"
        let callResult = nativeFn(0, ...[1, 2], 3, ...[]);
        new NativeCtor(...[7], ...[8, 9]);
        callResult;
        "#,
        &mut atoms,
    );

    let mut registry = RecordingRegistry::default();
    let result = evaluate_with_registry(
        &unit,
        |agent, realm| {
            install_native_global(agent, realm, "nativeFn", call_entry, false);
            install_native_global(agent, realm, "NativeCtor", construct_entry, true);
        },
        &mut registry,
    );

    assert_eq!(result, Value::from_smi(77));
    assert_eq!(
        registry
            .last_call
            .as_ref()
            .map(|record| record.arguments.clone()),
        Some(vec![
            Value::from_smi(0),
            Value::from_smi(1),
            Value::from_smi(2),
            Value::from_smi(3),
        ])
    );
    assert_eq!(
        registry
            .last_construct
            .as_ref()
            .map(|record| record.arguments.clone()),
        Some(vec![
            Value::from_smi(7),
            Value::from_smi(8),
            Value::from_smi(9),
        ])
    );
}

#[test]
fn phase6_functions_expand_custom_iterables_for_all_spread_positions() {
    let result = compile_and_run(
        r#"
        function makeIter(start, end) {
            return {
                [Symbol.iterator]: function() {
                    let current = start;
                    return {
                        next: function() {
                            if (current <= end) {
                                return { value: current++, done: false };
                            }
                            return { value: undefined, done: true };
                        }
                    };
                }
            };
        }

        function collect(a, b, c) {
            return a * 100 + b * 10 + c;
        }

        function Box(a, b, c) {
            this.sum = a * 100 + b * 10 + c;
        }

        let fromCall = collect(...makeIter(1, 3));
        let fromConstruct = (new Box(...makeIter(4, 6))).sum;
        let values = [0, ...makeIter(7, 8), 9];
        let sparse = [, ...makeIter(1, 2), ,];

        (fromCall === 123 ? 1 : 0)
            + (fromConstruct === 456 ? 2 : 0)
            + (
                values.length === 4
                    && values[0] === 0
                    && values[1] === 7
                    && values[2] === 8
                    && values[3] === 9
                    ? 4
                    : 0
            )
            + (
                sparse.length === 4
                    && !(0 in sparse)
                    && sparse[1] === 1
                    && sparse[2] === 2
                    && !(3 in sparse)
                    ? 8
                    : 0
            );
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn phase4_functions_mark_tail_call_capable_headers_when_lowering_tail_calls() {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(
        r#"
        function outer(fn, value) {
            return fn(value);
        }
        outer(function(x) { return x; }, 1);
        "#,
        &mut atoms,
    );

    let outer = unit
        .functions()
        .iter()
        .find(|function| function.name().and_then(|name| unit.atom_text(name)) == Some("outer"))
        .expect("outer should be compiled");

    assert!(outer.flags().tail_call_capable());
}

#[test]
fn phase4_functions_support_recursive_named_function_expressions() {
    let result = compile_and_run(
        r#"
        "use strict";
        (function f(n) {
            if (n === 0) {
                return 1;
            }
            return f(n - 1);
        })(5);
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn phase4_functions_support_object_literal_accessors_and_duplicates() {
    let result = compile_and_run(
        r#"
        let calls = 0;
        let key = "value";
        let object = {
            get value() { return 1; },
            get [key]() { return 2; },
            set value(input) { calls = input + 100; },
            set [key](input) { calls = input + 10; }
        };
        object.value = object.value;
        calls;
        "#,
    );

    assert_eq!(result, Value::from_smi(12));
}

#[test]
fn phase4_functions_support_symbol_keyed_accessors() {
    let result = compile_and_run(
        r#"
        let writes = 0;
        let key = Symbol();
        let object = {
            get [key]() { return key; },
            set [key](input) { writes = input === key ? 1 : -1; }
        };
        object[key] = object[key];
        writes + (typeof key === "symbol" ? 2 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase4_functions_match_computed_getter_keys_across_string_number_and_symbol() {
    let result = compile_and_run(
        r#"
        let s = Symbol();
        let object = {
            get ["a"]() { return "A"; },
            get [1]() { return 1; },
            get [s]() { return s; }
        };
        (object.a === "A" ? 1 : 0)
            + (object[1] === 1 ? 2 : 0)
            + (object[s] === s ? 4 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase4_functions_match_computed_setter_keys_across_string_number_and_symbol() {
    let result = compile_and_run(
        r#"
        let calls = 0;
        let s = Symbol();
        let object = {
            set ["a"](_) { calls = calls + 1; },
            set [1](_) { calls = calls + 1; },
            set [s](_) { calls = calls + 1; }
        };
        object.a = "A";
        object[1] = 1;
        object[s] = s;
        calls;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase4_functions_support_numeric_keyed_accessors() {
    let result = compile_and_run(
        r#"
        let calls = 0;
        let object = {
            get [1]() { return 10; },
            set [1](_) { calls = calls + 1; }
        };
        let read = object[1];
        object[1] = read;
        read + calls;
        "#,
    );

    assert_eq!(result, Value::from_smi(11));
}

#[test]
fn phase4_functions_read_numeric_keyed_getters() {
    let result = compile_and_run(
        r#"
        let object = {
            get [1]() { return 10; }
        };
        object[1];
        "#,
    );

    assert_eq!(result, Value::from_smi(10));
}

#[test]
fn phase4_functions_write_numeric_keyed_setters() {
    let result = compile_and_run(
        r#"
        let calls = 0;
        let object = {
            set [1](_) { calls = calls + 1; }
        };
        object[1] = 10;
        calls;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn phase4_functions_accept_empty_object_destructuring_parameters() {
    let result = compile_and_run(
        r#"
        function probe({}) { return 1; }
        probe(0) + probe(false) + probe([]);
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase4_functions_compile_nested_destructuring_parameter_shapes() {
    let mut atoms = AtomTable::new();
    let _ = compile_unit(
        r#"
        function fn1([{}]) {}
        function fn2([{a: [{}]}]) {}
        function fn3({a: [,,,] = 42}) {}
        function fn4([], [[]], [[[[[[[[[x]]]]]]]]]) {}
        function fn5([[x, y, ...z]]) {}
        "#,
        &mut atoms,
    );
}

#[test]
fn phase4_functions_support_object_pattern_rest_parameters() {
    let result = compile_and_run_string(
        r#"
        function describe(...{ 0: first, 2: third, length }) {
            return first + ":" + third + ":" + length;
        }
        describe("a", "b", "c");
        "#,
    );

    assert_eq!(result, "a:c:3");
}

#[test]
fn phase4_functions_cache_tagged_template_objects_by_site() {
    let result = compile_and_run(
        r#"
        let seen = [];
        function tag(strings) {
            seen.push(strings);
        }
        function run(value) {
            tag`head${value}tail`;
        }
        run(1);
        run(2);
        seen.length === 2 && seen[0] === seen[1] ? 1 : 0;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn phase6_generator_methods_expose_generator_prototype_property() {
    let result = compile_and_run(
        r#"
        var GeneratorPrototype = Object.getPrototypeOf(function* () {}).prototype;
        var method = { *method() {} }.method;
        var ordinaryMethod = { method() {} }.method;
        var descriptor = Object.getOwnPropertyDescriptor(method, "prototype");

        (Object.getPrototypeOf(method.prototype) === GeneratorPrototype ? 1 : 0)
            + (descriptor.writable === true ? 2 : 0)
            + (descriptor.enumerable === false ? 4 : 0)
            + (descriptor.configurable === false ? 8 : 0)
            + (!Object.prototype.hasOwnProperty.call(ordinaryMethod, "prototype") ? 16 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn phase4_functions_share_tagged_template_cache_across_internal_callbacks() {
    let result = compile_and_run(
        r#"
        let first = undefined;
        let same = 0;
        function tag(strings) {
            if (first === undefined) {
                first = strings;
            } else {
                same = first === strings ? 1 : 0;
            }
            return "";
        }
        function callback() {
            tag`x`;
            return "a";
        }
        "ab".replace("b", callback);
        callback();
        same;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn phase4_functions_preserve_cooked_and_raw_tagged_template_parts() {
    let result = compile_and_run(
        r#"
        let score = 0;
        (function(strings, value) {
            score = (strings[0] === undefined ? 1 : 0)
                + (strings.raw[0] === "\\xg" ? 2 : 0)
                + (value === "inner" ? 4 : 0)
                + (strings[1] === "right" ? 8 : 0)
                + (strings.raw[1] === "right" ? 16 : 0);
        })`\xg${"inner"}right`;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn phase4_functions_freeze_tagged_template_objects_and_raw_arrays() {
    let result = compile_and_run(
        r#"
        let score = 0;
        (function(strings) {
            const cooked = Object.getOwnPropertyDescriptor(strings, "0");
            const raw = Object.getOwnPropertyDescriptor(strings, "raw");
            const length = Object.getOwnPropertyDescriptor(strings, "length");
            score += Object.isFrozen(strings) ? 1 : 0;
            score += Object.isFrozen(strings.raw) ? 2 : 0;
            score += cooked.enumerable === true && cooked.writable === false && cooked.configurable === false ? 4 : 0;
            score += raw.enumerable === false && raw.writable === false && raw.configurable === false ? 8 : 0;
            score += length.enumerable === false && length.writable === false && length.configurable === false ? 16 : 0;
            strings.extra = 1;
            strings.raw.extra = 2;
            score += strings.extra === undefined && strings.raw.extra === undefined ? 32 : 0;
        })`left${1}right`;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn phase4_functions_dispatch_native_call_and_construct_through_registry() {
    let call_entry = BuiltinFunctionId::from_raw(1).unwrap();
    let construct_entry = BuiltinFunctionId::from_raw(2).unwrap();
    let mut atoms = AtomTable::new();
    let unit = compile_unit(
        r#"
        let callResult = nativeFn(3, 4);
        let built = new NativeCtor(7);
        callResult;
        "#,
        &mut atoms,
    );

    let mut registry = RecordingRegistry::default();
    let mut installed_function = None;
    let mut installed_constructor = None;
    let mut expected_realm = None;
    let mut expected_environment = None;
    let result = evaluate_with_registry(
        &unit,
        |agent, realm| {
            expected_realm = Some(realm.id());
            expected_environment = Some(realm.global_env());
            installed_function = Some(install_native_global(
                agent, realm, "nativeFn", call_entry, false,
            ));
            installed_constructor = Some(install_native_global(
                agent,
                realm,
                "NativeCtor",
                construct_entry,
                true,
            ));
        },
        &mut registry,
    );
    let native_function = installed_function.expect("native function should install");
    let native_constructor = installed_constructor.expect("native constructor should install");
    let expected_realm = expected_realm.expect("realm should be recorded");
    let expected_environment = expected_environment.expect("environment should be recorded");

    assert_eq!(result, Value::from_smi(77));

    assert_eq!(
        registry.last_call,
        Some(RecordedNativeCall {
            callee: native_function,
            arguments: vec![Value::from_smi(3), Value::from_smi(4)],
            realm: expected_realm,
            environment: expected_environment,
            entry: NativeFunctionId::builtin(call_entry),
        })
    );
    assert_eq!(
        registry.last_construct,
        Some(RecordedNativeConstruct {
            callee: native_constructor,
            new_target: native_constructor,
            arguments: vec![Value::from_smi(7)],
            realm: expected_realm,
            environment: expected_environment,
            entry: NativeFunctionId::builtin(construct_entry),
        })
    );
}

#[test]
fn phase4_functions_evaluate_argument_assignments_left_to_right_and_create_globals() {
    let result = compile_and_run(
        r#"
        function readThird(first, second, third) {
            return third;
        }
        readThird(x = 1, y = x, x + y) + x + y;
        "#,
    );

    assert_eq!(result, Value::from_smi(4));
}

#[test]
fn phase4_functions_call_default_parameter_function_with_supplied_argument() {
    let result = compile_and_run(
        r#"
        function format(key, objectName = "") {
            return objectName + "." + key;
        }
        format("length", "A") === "A.length" ? 1 : 0;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn phase4_functions_delete_mapped_arguments_elements() {
    let result = compile_and_run(
        r#"
        function inspect(a, b) {
            return delete arguments[0] && arguments[0] === undefined ? 1 : 0;
        }
        inspect(1, 2);
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn phase4_functions_bridge_high_register_calls_in_large_scripts() {
    let mut source = String::new();
    for index in 0..280 {
        source.push_str(&format!("let value{index} = {index};\n"));
    }
    source.push_str("let fnRef = function(value) { return value; };\n");
    source.push_str("fnRef(value279);\n");

    let result = compile_and_run(&source);

    assert_eq!(result, Value::from_smi(279));
}

#[test]
fn phase4_functions_execute_tail_recursive_calls_without_growing_frames() {
    let result = compile_and_run(
        r#"
        let countdown = function(self, value, acc) {
            if (value === 0) {
                return acc;
            }
            return self(self, value - 1, acc + 1);
        };
        countdown(countdown, 120, 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(120));
}

#[test]
fn phase4_functions_treat_nullish_coalescing_rhs_as_tail_position() {
    let result = compile_and_run(
        r#"
        let countdown = function(self, value, acc) {
            if (value === 0) {
                return acc;
            }
            return null ?? self(self, value - 1, acc + 1);
        };
        countdown(countdown, 120, 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(120));
}

#[test]
fn phase4_functions_preserve_constructor_return_fallback_across_tail_calls() {
    let result = compile_and_run(
        r#"
        function Box(helper) {
            this.value = 9;
            return helper(1);
        }
        let box = new Box(function(value) { return value; });
        box.value;
        "#,
    );

    assert_eq!(result, Value::from_smi(9));
}

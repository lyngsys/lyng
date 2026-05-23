use super::support::{
    compile_and_run, compile_and_run_string, compile_unit, evaluate_with_registry,
};
use lyng_js_builtins::{bootstrap_realm, BootstrapMode, BootstrapRequest, BuiltinCache};
use lyng_js_common::AtomTable;
use lyng_js_gc::{AllocationLifetime, PrimitiveMutator};
use lyng_js_objects::{
    InternalMethodResult, NativeCallRequest, NativeConstructRequest, NativeFunctionRegistry,
    ObjectRuntime,
};
use lyng_js_ops::object::ordinary_create_data_property;
use lyng_js_types::{PropertyKey, Value};

#[derive(Default)]
struct RejectingRegistry;

impl NativeFunctionRegistry for RejectingRegistry {
    fn call(
        &mut self,
        _runtime: &mut ObjectRuntime,
        _heap: &mut PrimitiveMutator<'_>,
        _request: NativeCallRequest<'_>,
    ) -> InternalMethodResult<Value> {
        panic!("unexpected native call during cross-realm object test");
    }

    fn construct(
        &mut self,
        _runtime: &mut ObjectRuntime,
        _heap: &mut PrimitiveMutator<'_>,
        _request: NativeConstructRequest<'_>,
    ) -> InternalMethodResult<lyng_js_types::ObjectRef> {
        panic!("unexpected native construct during cross-realm object test");
    }
}

#[test]
fn phase5_objects_support_create_descriptors_and_prototype_mutation() {
    let result = compile_and_run(
        r#"
        let prototype = { inherited: 1 };
        let object = Object.create(prototype);
        let rebound = {};
        Object.defineProperty(object, "answer", {
            value: 7,
            writable: false,
            enumerable: true,
            configurable: false
        });
        Object.setPrototypeOf(rebound, prototype);
        let descriptor = Object.getOwnPropertyDescriptor(object, "answer");
        (Object.getPrototypeOf(object) === prototype ? 1 : 0)
            + (Object.getPrototypeOf(rebound) === prototype ? 2 : 0)
            + (descriptor.value === 7 ? 4 : 0)
            + (descriptor.writable === false ? 8 : 0)
            + (descriptor.enumerable === true ? 16 : 0)
            + (descriptor.configurable === false ? 32 : 0)
            + (object.hasOwnProperty("answer") ? 64 : 0)
            + (prototype.isPrototypeOf(object) ? 128 : 0)
            + (object.propertyIsEnumerable("answer") ? 256 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(511));
}

#[test]
fn phase5_property_is_enumerable_converts_key_before_this_object() {
    let result = compile_and_run_string(
        r#"
        let keyConverted = false;
        let key = {
            toString() {
                keyConverted = true;
                throw new RangeError("key");
            }
        };

        try {
            Object.prototype.propertyIsEnumerable.call(null, key);
            "no throw";
        } catch (error) {
            String(keyConverted) + ":" + String(error instanceof RangeError);
        }
        "#,
    );

    assert_eq!(result, "true:true");
}

#[test]
fn phase5_objects_support_null_prototypes_and_integrity_helpers() {
    let result = compile_and_run(
        r"
        let nullProto = Object.create(null);
        let sealed = Object.seal({ answer: 1 });
        let frozen = Object.freeze({ answer: 2 });
        let extensible = { answer: 3 };
        let extensibleBefore = Object.isExtensible(extensible);
        Object.preventExtensions(extensible);
        (Object.getPrototypeOf(nullProto) === null ? 1 : 0)
            + (extensibleBefore ? 2 : 0)
            + (!Object.isExtensible(extensible) ? 4 : 0)
            + (Object.isSealed(sealed) ? 8 : 0)
            + (Object.isFrozen(frozen) ? 16 : 0)
            + (!Object.isExtensible(sealed) ? 32 : 0)
            + (!Object.isExtensible(frozen) ? 64 : 0);
        ",
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn phase5_object_is_uses_same_value_semantics() {
    let result = compile_and_run(
        r#"
        let object = {};
        let total = 0;
        total += (typeof Object.is === "function" ? 1 : 0);
        total += (Object.is(NaN, NaN) ? 2 : 0);
        total += (!Object.is(+0, -0) ? 4 : 0);
        total += (Object.is(-0, -0) ? 8 : 0);
        total += (Object.is(object, object) ? 16 : 0);
        total += (!Object.is({}, {}) ? 32 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn phase6_object_literal_bigint_property_names_become_string_keys() {
    let result = compile_and_run(
        r#"
        let object = {
            1n: "one",
            [-2n]: "minus two"
        };

        (object["1"] === "one" ? 1 : 0)
            + (object["-2"] === "minus two" ? 2 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase6_object_literal_accessor_keys_apply_to_property_key() {
    let result = compile_and_run(
        r#"
        let log = "";
        let key = {
            [Symbol.toPrimitive](hint) {
                log += hint;
                return "value";
            }
        };
        let object = {
            backing: 3,
            get [key]() {
                return this.backing + 1;
            },
            set [key](value) {
                this.backing = value;
            }
        };

        let total = 0;
        total += object.value === 4 ? 1 : 0;
        object.value = 9;
        total += object.backing === 9 ? 2 : 0;
        total += log === "stringstring" ? 4 : 0;
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase6_equality_to_primitive_uses_default_hint() {
    let result = compile_and_run(
        r#"
        let left = {
            [Symbol.toPrimitive](hint) {
                return hint === "default" ? 1 : 2;
            }
        };
        let right = {
            [Symbol.toPrimitive](hint) {
                return hint === "default" ? 1 : 2;
            }
        };

        (true == left ? 1 : 0) + (right == true ? 2 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase6_object_literal_computed_key_coerces_before_value_evaluation() {
    let result = compile_and_run_string(
        r#"
        let value = "bad";
        let key = {
            toString() {
                value = "ok";
                return "p";
            }
        };

        let object = {
            [key]: value
        };

        object.p;
        "#,
    );

    assert_eq!(result, "ok");
}

#[test]
fn phase6_object_literal_function_expression_keys_use_trimmed_source_text() {
    let result = compile_and_run(
        r"
        let object = {
            [function () {}]: 7
        };

        object[function () {}] + object[String(function () {})];
        ",
    );

    assert_eq!(result, Value::from_smi(14));
}

#[test]
fn phase6_global_proxy_prototype_bare_bindings_use_global_receiver() {
    let result = compile_and_run(
        r#"
        var global = this;
        var proto = Object.getPrototypeOf(global);
        var gets = 0;
        var sets = 0;

        Object.setPrototypeOf(global, new Proxy(proto, {
            has(target, key) {
                return key === "bareword" || Reflect.has(target, key);
            },
            get(target, key, receiver) {
                gets++;
                return receiver === global ? Reflect.get(target, key, receiver) : -1;
            },
            set(target, key, value, receiver) {
                sets++;
                if (receiver !== global) {
                    return false;
                }
                return Reflect.set(target, key, value, receiver);
            }
        }));

        var total = 0;
        total += bareword === undefined ? 1 : 0;
        total += gets === 1 ? 2 : 0;
        bareword = 12;
        total += sets === 1 ? 4 : 0;
        total += global.bareword === 12 ? 8 : 0;
        Object.setPrototypeOf(global, proto);
        delete global.bareword;
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn phase5_object_create_routes_properties_through_shared_descriptor_semantics() {
    let result = compile_and_run(
        r#"
        let prototype = { inherited: 1 };
        let props = {};
        let seen = 0;
        Object.defineProperty(props, "hidden", {
            value: { value: 1 },
            enumerable: false
        });
        Object.defineProperty(props, "answer", {
            enumerable: true,
            get: function() {
                seen += 1;
                return {
                    value: 7,
                    writable: false,
                    enumerable: true,
                    configurable: false
                };
            }
        });

        let object = Object.create(prototype, props);
        let descriptor = Object.getOwnPropertyDescriptor(object, "answer");
        (Object.getPrototypeOf(object) === prototype ? 1 : 0)
            + (seen === 1 ? 2 : 0)
            + (object.answer === 7 ? 4 : 0)
            + (object.hasOwnProperty("hidden") === false ? 8 : 0)
            + (descriptor.value === 7 ? 16 : 0)
            + (descriptor.writable === false ? 32 : 0)
            + (descriptor.enumerable === true ? 64 : 0)
            + (descriptor.configurable === false ? 128 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(255));
}

#[test]
fn phase5_object_create_coerces_properties_argument_before_collecting_keys() {
    let result = compile_and_run(
        r#"
        let proto = {};
        let total = 0;

        total += Object.getPrototypeOf(Object.create(proto, true)) === proto ? 1 : 0;
        total += Object.getPrototypeOf(Object.create(proto, 1)) === proto ? 2 : 0;
        total += Object.getPrototypeOf(Object.create(proto, Symbol("phase5"))) === proto ? 4 : 0;

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase5_object_prototype_to_string_reports_foundational_tags() {
    let result = compile_and_run_string(
        r#"
        Object.prototype.toString.call({})
            + "|"
            + Object.prototype.toString.call(function() {})
            + "|"
            + Object.prototype.toString.call([])
            + "|"
            + Object.prototype.toString.call(new TypeError("boom"))
            + "|"
            + Object.prototype.toString.call(TypeError.prototype);
        "#,
    );

    assert_eq!(
        result,
        "[object Object]|[object Function]|[object Array]|[object Error]|[object Object]"
    );
}

#[test]
fn phase5_object_prototype_to_string_handles_arguments_and_symbol_tag_overrides() {
    let result = compile_and_run(
        r#"
        let total = 0;
        let custom = {};
        custom[Symbol.toStringTag] = "phase5";
        let nonStringTag = {};
        nonStringTag[Symbol.toStringTag] = 86;

        total += Object.prototype.toString.call(function() { return arguments; }())
            === "[object Arguments]" ? 1 : 0;
        total += Object.prototype.toString.call(custom) === "[object phase5]" ? 2 : 0;
        total += Object.prototype.toString.call(nonStringTag) === "[object Object]" ? 4 : 0;
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase6_object_prototype_to_string_uses_proxy_aware_array_and_tag_fallbacks() {
    let result = compile_and_run_string(
        r#"
        let handle = Proxy.revocable([], {
            get() {
                handle.revoke();
            }
        });

        Object.defineProperty(BigInt.prototype, Symbol.toStringTag, { value: 86 });

        Object.prototype.toString.call(handle.proxy)
            + "|"
            + Object.prototype.toString.call(Object(BigInt(0)));
        "#,
    );

    assert_eq!(result, "[object Array]|[object Object]");
}

#[test]
fn phase5_object_property_keys_use_shared_object_aware_coercion() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let ownProp = {
            toString: function() {
                return "abc";
            }
        };
        let neitherPrimitive = {
            toString: function() {
                return {};
            },
            valueOf: function() {
                return {};
            }
        };

        let defined = {};
        Object.defineProperty(defined, [1, 2], { value: 5 });
        score += defined.hasOwnProperty("1,2") ? 1 : 0;

        Object.defineProperty(defined, new Boolean(true), { value: 6 });
        score += defined.hasOwnProperty("true") ? 2 : 0;

        Object.defineProperty(defined, ownProp, { value: 7 });
        score += defined.hasOwnProperty("abc") ? 4 : 0;

        try {
            Object.defineProperty({}, neitherPrimitive, {});
        } catch (error) {
            if (Object.getPrototypeOf(error) === TypeError.prototype) {
                score += 8;
            }
        }

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn phase6_object_assign_copies_enumerable_own_properties_to_target() {
    let result = compile_and_run_string(
        r#"
        let symbol = Symbol("copy");
        let log = "";
        let source = {};
        Object.defineProperty(source, "hidden", {
            value: 1,
            enumerable: false
        });
        Object.defineProperty(source, "a", {
            get: function() {
                log += "get";
                return 5;
            },
            enumerable: true
        });
        source[symbol] = 7;

        let target = {};
        let result = Object.assign(target, source);
        let descriptor = Object.getOwnPropertyDescriptor(result, "a");

        (result === target ? "same" : "different")
            + "|"
            + String(result.a)
            + "|"
            + String(result[symbol])
            + "|"
            + String("hidden" in result)
            + "|"
            + String(descriptor.enumerable)
            + "/"
            + String(descriptor.writable)
            + "/"
            + String(descriptor.configurable)
            + "|"
            + log
            + "|"
            + typeof Object.assign("x")
            + "|"
            + Object.assign("x").valueOf();
        "#,
    );

    assert_eq!(result, "same|5|7|false|true/true/true|get|object|x");
}

#[test]
fn phase6_object_from_entries_defines_ordinary_data_properties() {
    let result = compile_and_run_string(
        r#"
        let symbol = Symbol("entry");
        let log = "";
        let key = {
            [Symbol.toPrimitive](hint) {
                log += hint;
                return "coerced";
            }
        };

        let result = Object.fromEntries([
            ["a", 1],
            [key, 2],
            [symbol, 3]
        ]);
        let descriptor = Object.getOwnPropertyDescriptor(result, "coerced");

        (Object.getPrototypeOf(result) === Object.prototype ? "proto" : "bad")
            + "|"
            + String(result.a)
            + "|"
            + String(result.coerced)
            + "|"
            + String(result[symbol])
            + "|"
            + String(descriptor.enumerable)
            + "/"
            + String(descriptor.writable)
            + "/"
            + String(descriptor.configurable)
            + "|"
            + log;
        "#,
    );

    assert_eq!(result, "proto|1|2|3|true/true/true|string");
}

#[test]
fn phase6_object_from_entries_closes_iterator_after_bad_entry() {
    let result = compile_and_run_string(
        r"
        let closed = false;
        let iterable = {
            [Symbol.iterator]() {
                return {
                    next() {
                        return { value: null, done: false };
                    },
                    return() {
                        closed = true;
                        return {};
                    }
                };
            }
        };

        try {
            Object.fromEntries(iterable);
        } catch (error) {}

        String(closed);
        ",
    );

    assert_eq!(result, "true");
}

#[test]
fn phase6_object_from_entries_reads_entry_value_before_key_coercion() {
    let result = compile_and_run_string(
        r#"
        let effects = [];
        let entry = {};
        Object.defineProperty(entry, "0", {
            get: function() {
                effects.push("get 0");
                return {
                    toString: function() {
                        effects.push("key toString");
                        return "key";
                    }
                };
            }
        });
        Object.defineProperty(entry, "1", {
            get: function() {
                effects.push("get 1");
                return "value";
            }
        });

        let result = Object.fromEntries([entry]);
        effects.join("|") + "|" + result.key;
        "#,
    );

    assert_eq!(result, "get 0|get 1|key toString|value");
}

#[test]
fn phase6_annex_b_object_proto_accessor_gets_and_sets_prototypes() {
    let result = compile_and_run_string(
        r#"
        let descriptor = Object.getOwnPropertyDescriptor(Object.prototype, "__proto__");
        let base = { marker: 1 };
        let object = {};
        descriptor.set.call(object, base);
        let nullProto = Object.create(null);

        [
            typeof descriptor.get,
            typeof descriptor.set,
            String(descriptor.enumerable),
            String(descriptor.configurable),
            String(descriptor.get.call(object) === base),
            String(object.marker),
            String(descriptor.get.call(nullProto)),
            String(descriptor.set.call(1, base))
        ].join("|");
        "#,
    );

    assert_eq!(result, "function|function|false|true|true|1|null|undefined");
}

#[test]
fn phase6_annex_b_define_and_lookup_accessors_walk_prototypes() {
    let result = compile_and_run_string(
        r#"
        let prototype = {};
        let child = Object.create(prototype);
        function getValue() { return 11; }
        function setValue(value) { this.seen = value; }

        prototype.__defineGetter__("value", getValue);
        prototype.__defineSetter__("value", setValue);
        let getter = child.__lookupGetter__("value");
        let setter = child.__lookupSetter__("value");
        child.value = 17;
        let descriptor = Object.getOwnPropertyDescriptor(prototype, "value");

        [
            String(getter === getValue),
            String(setter === setValue),
            String(child.value),
            String(child.seen),
            String(descriptor.enumerable),
            String(descriptor.configurable),
            String(child.__lookupGetter__("missing"))
        ].join("|");
        "#,
    );

    assert_eq!(result, "true|true|11|17|true|true|undefined");
}

#[test]
fn phase6_proto_cycle_detection_stops_at_proxy_exotic_prototype() {
    let result = compile_and_run_string(
        r"
        let root = {};
        let intermediary = new Proxy(Object.create(root), {});
        let leaf = Object.create(intermediary);

        root.__proto__ = leaf;

        String(Object.getPrototypeOf(root) === leaf);
        ",
    );

    assert_eq!(result, "true");
}

#[test]
fn phase6_object_group_by_groups_iterable_values_into_null_prototype_object() {
    let result = compile_and_run_string(
        r#"
        let calls = "";
        let grouped = Object.groupBy([1, 2, 3], function(value, index) {
            calls += String(value) + ":" + String(index) + ";";
            return value % 2 === 0 ? "even" : "odd";
        });

        [
            String(Object.getPrototypeOf(grouped)),
            grouped.odd.join(","),
            grouped.even.join(","),
            Object.keys(grouped).join(","),
            calls
        ].join("|");
        "#,
    );

    assert_eq!(result, "null|1,3|2|odd,even|1:0;2:1;3:2;");
}

#[test]
fn phase6_object_define_properties_rejects_primitive_targets() {
    let result = compile_and_run_string(
        r#"
        let results = [];
        for (let value of [0, true, "abc"]) {
            try {
                Object.defineProperties(value, {});
                results.push("no");
            } catch (error) {
                results.push("throw");
            }
        }
        results.join("|");
        "#,
    );

    assert_eq!(result, "throw|throw|throw");
}

#[test]
fn phase6_object_freeze_uses_proxy_traps_and_partial_descriptors() {
    let result = compile_and_run_string(
        r#"
        let symbol = Symbol("s");
        let target = {};
        target[symbol] = 1;
        target.foo = 2;
        target[0] = 3;

        let seen = [];
        let proxy = new Proxy(target, {
            getOwnPropertyDescriptor(target, key) {
                seen.push(typeof key === "symbol" ? "sym" : String(key));
                return Reflect.getOwnPropertyDescriptor(target, key);
            },
            defineProperty(target, key, descriptor) {
                seen.push(
                    "def:"
                        + (typeof key === "symbol" ? "sym" : String(key))
                        + ":"
                        + String("value" in descriptor)
                        + "/"
                        + String("writable" in descriptor)
                        + "/"
                        + String("enumerable" in descriptor)
                        + "/"
                        + String(descriptor.configurable)
                );
                return Reflect.defineProperty(target, key, descriptor);
            }
        });

        Object.freeze(proxy);

        let preventFalseThrows = false;
        try {
            Object.freeze(new Proxy({}, {
                preventExtensions() {
                    return false;
                }
            }));
        } catch (error) {
            preventFalseThrows = true;
        }

        seen.join("|") + "|" + String(preventFalseThrows);
        "#,
    );

    assert_eq!(
        result,
        "0|def:0:false/true/false/false|foo|def:foo:false/true/false/false|sym|def:sym:false/true/false/false|true"
    );
}

#[test]
fn phase6_object_prototype_has_immutable_prototype() {
    let result = compile_and_run_string(
        r#"
        let candidate = Object.create(null);
        let setThrows = false;
        try {
            Object.setPrototypeOf(Object.prototype, candidate);
        } catch (error) {
            setThrows = true;
        }

        [
            String(setThrows),
            String(Reflect.setPrototypeOf(Object.prototype, candidate)),
            String(Object.getPrototypeOf(Object.prototype) === null)
        ].join("|");
        "#,
    );

    assert_eq!(result, "true|false|true");
}

#[test]
fn phase6_object_constructor_subclass_ignores_object_argument() {
    let result = compile_and_run_string(
        r#"
        class Derived extends Object {}
        let constructed = new Derived({ marker: 1 });
        let reflected = Reflect.construct(Object, [{ marker: 2 }], Derived);

        [
            String(constructed.marker),
            String(reflected.marker),
            String(Object.getPrototypeOf(constructed) === Derived.prototype),
            String(Object.getPrototypeOf(reflected) === Derived.prototype)
        ].join("|");
        "#,
    );

    assert_eq!(result, "undefined|undefined|true|true");
}

#[test]
fn phase5_object_define_property_requires_object_targets() {
    let result = compile_and_run(
        r#"
        let total = 0;
        try {
            Object.defineProperty(true, "x", {});
        } catch (error) {
            total += error.constructor === TypeError ? 1 : 0;
        }
        try {
            Object.defineProperty(5, "x", {});
        } catch (error) {
            total += error.constructor === TypeError ? 2 : 0;
        }
        try {
            Object.defineProperty("hello", "x", {});
        } catch (error) {
            total += error.constructor === TypeError ? 4 : 0;
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase5_object_descriptor_objects_expose_missing_fields_as_own_properties() {
    let result = compile_and_run(
        r#"
        let data = Object.getOwnPropertyDescriptor(
            Object.defineProperty({}, "x", { writable: true, configurable: true }),
            "x",
        );
        let accessor = Object.getOwnPropertyDescriptor(
            Object.defineProperty({}, "x", { get: function() {}, configurable: true }),
            "x",
        );
        let reverseAccessor = Object.getOwnPropertyDescriptor(
            Object.defineProperty({}, "x", { set: function(_) {}, configurable: true }),
            "x",
        );
        let total = 0;
        if ("value" in data) {
            total += 1;
        }
        if ("set" in accessor) {
            total += 2;
        }
        if ("get" in reverseAccessor) {
            total += 4;
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase5_object_define_property_respects_engine_array_length_rules() {
    let result = compile_and_run(
        r#"
        let total = 0;
        let grown = [0, , 2];
        Object.defineProperty(grown, "length", { value: 5 });
        if (grown.length === 5 && grown.hasOwnProperty("1") === false && grown.hasOwnProperty("4") === false && grown[2] === 2) {
            total += 1;
        }

        let sealedLength = [1, 2, 3];
        Object.defineProperty(sealedLength, "length", { writable: false });
        try {
            Object.defineProperty(sealedLength, 3, { value: "abc" });
        } catch (error) {
            if (error.constructor === TypeError && sealedLength.length === 3 && sealedLength.hasOwnProperty("3") === false) {
                total += 2;
            }
        }

        let shrunk = [0, 1, 2];
        Object.defineProperty(shrunk, "length", { value: 1 });
        if (shrunk.length === 1 && shrunk.hasOwnProperty("2") === false) {
            total += 4;
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase5_object_get_prototype_of_uses_runtime_prototype_graphs() {
    let result = compile_and_run(
        r"
        let total = 0;
        total += Object.getPrototypeOf(Boolean) === Function.prototype ? 1 : 0;
        total += Object.getPrototypeOf(RangeError) === Error ? 2 : 0;
        total += Object.getPrototypeOf(TypeError) === Error ? 4 : 0;
        total += Object.getPrototypeOf(function() { return arguments; }()) === Object.prototype
            ? 8
            : 0;
        total;
        ",
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn phase6_proxy_routes_script_and_object_operations_through_traps() {
    let result = compile_and_run(
        r#"
        let proto = { marker: 1 };
        let replacementProto = { replacement: 1 };
        let target = { own: 1 };
        Object.setPrototypeOf(target, proto);

        let handler = {
            get(t, key, receiver) {
                if (key === "answer") {
                    return 41;
                }
                return t[key];
            },
            set(t, key, value, receiver) {
                t[key] = value + 1;
                return true;
            },
            has(t, key) {
                return key === "virtual" || key in t;
            },
            deleteProperty(t, key) {
                delete t[key];
                return true;
            },
            ownKeys(t) {
                return ["virtual"];
            },
            getOwnPropertyDescriptor(t, key) {
                if (key === "virtual") {
                    return {
                        value: 9,
                        writable: true,
                        enumerable: true,
                        configurable: true
                    };
                }
                return Object.getOwnPropertyDescriptor(t, key);
            },
            getPrototypeOf(t) {
                return Object.getPrototypeOf(t);
            },
            setPrototypeOf(t, next) {
                Object.setPrototypeOf(t, next);
                return true;
            },
            isExtensible(t) {
                return Object.isExtensible(t);
            },
            preventExtensions(t) {
                Object.preventExtensions(t);
                return true;
            }
        };

        let proxy = new Proxy(target, handler);
        let total = 0;
        let seen = "";

        total += (proxy.answer === 41 ? 1 : 0);
        proxy.count = 5;
        total += (target.count === 6 ? 2 : 0);
        total += ("virtual" in proxy ? 4 : 0);
        delete proxy.own;
        total += (target.hasOwnProperty("own") === false ? 8 : 0);
        total += (Object.getOwnPropertyDescriptor(proxy, "virtual").value === 9 ? 16 : 0);
        total += (Object.keys(proxy).join(",") === "virtual" ? 32 : 0);
        for (let key in proxy) {
            seen += key;
        }
        total += (seen === "virtualmarker" ? 64 : 0);
        total += (Object.getPrototypeOf(proxy) === proto ? 128 : 0);
        Object.setPrototypeOf(proxy, replacementProto);
        total += (Object.getPrototypeOf(target) === replacementProto ? 256 : 0);
        total += (Object.isExtensible(proxy) ? 512 : 0);
        Object.preventExtensions(proxy);
        total += (!Object.isExtensible(target) ? 1024 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(2047));
}

#[test]
fn phase6_object_spread_uses_proxy_aware_copy_data_properties() {
    let result = compile_and_run_string(
        r#"
        let log = [];
        let proxy = new Proxy({}, {
            ownKeys(target) {
                log.push("ownKeys");
                return ["visible", "hidden", "missing"];
            },
            getOwnPropertyDescriptor(target, key) {
                log.push("getOwnPropertyDescriptor:" + key);
                if (key === "visible") {
                    return {
                        value: 1,
                        writable: true,
                        enumerable: true,
                        configurable: true
                    };
                }
                if (key === "hidden") {
                    return {
                        value: 2,
                        writable: true,
                        enumerable: false,
                        configurable: true
                    };
                }
                return undefined;
            },
            get(target, key, receiver) {
                log.push("get:" + key);
                return 9;
            }
        });

        let copy = { ...proxy };
        [
            copy.visible,
            "hidden" in copy,
            "missing" in copy,
            log.join("|")
        ].join(":");
        "#,
    );

    assert_eq!(
        result,
        "9:false:false:ownKeys|getOwnPropertyDescriptor:visible|get:visible|getOwnPropertyDescriptor:hidden|getOwnPropertyDescriptor:missing"
    );
}

#[test]
fn phase6_proxy_set_trap_from_prototype_receives_original_receiver() {
    let result = compile_and_run(
        r#"
        let seenHandler;
        let seenTarget;
        let seenProp;
        let seenValue;
        let seenReceiver;
        let target = {};
        let handler = {
            set(target, prop, value, receiver) {
                seenHandler = this;
                seenTarget = target;
                seenProp = prop;
                seenValue = value;
                seenReceiver = receiver;
                return true;
            }
        };
        let proxy = new Proxy(target, handler);
        let receiver = Object.create(proxy);

        receiver.prop = "value";

        let total = 0;
        total += seenHandler === handler ? 1 : 0;
        total += seenTarget === target ? 2 : 0;
        total += seenProp === "prop" ? 4 : 0;
        total += seenValue === "value" ? 8 : 0;
        total += seenReceiver === receiver ? 16 : 0;
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn phase6_array_builtins_route_proxy_observable_operations_through_traps() {
    let result = compile_and_run_string(
        r#"
        let log = [];
        let target = { length: 1 };
        let syntheticIndex = true;
        let proxy = new Proxy(target, {
            get(target, key, receiver) {
                log.push("get:" + String(key));
                if (key === "length") {
                    return target.length;
                }
                if (key === "0") {
                    return syntheticIndex ? "needle" : Reflect.get(target, key, receiver);
                }
                return Reflect.get(target, key, receiver);
            },
            has(target, key) {
                log.push("has:" + String(key));
                return key === "0" || key in target;
            },
            set(target, key, value, receiver) {
                log.push("set:" + String(key) + "=" + String(value));
                target[key] = value;
                return true;
            },
            deleteProperty(target, key) {
                log.push("delete:" + String(key));
                delete target[key];
                return true;
            }
        });

        let total = 0;
        total += Array.prototype.indexOf.call(proxy, "needle") === 0 ? 1 : 0;

        Array.prototype.push.call(proxy, "next");
        total += target[1] === "next" && target.length === 2 ? 2 : 0;

        target[0] = "first";
        target.length = 1;
        syntheticIndex = false;
        total += Array.prototype.pop.call(proxy) === "first" ? 4 : 0;
        total += target.length === 0 && !("0" in target) ? 8 : 0;

        String(total) + ";" + log.join("|");
        "#,
    );

    assert_eq!(
        result,
        "15;get:length|has:0|get:0|get:length|set:1=next|set:length=2|get:length|get:0|delete:0|set:length=0"
    );
}

#[test]
fn phase6_object_prototype_builtins_route_proxy_observable_operations_through_traps() {
    let result = compile_and_run_string(
        r#"
        let log = [];
        let proxy = new Proxy({}, {
            getOwnPropertyDescriptor(target, key) {
                log.push("getOwnPropertyDescriptor:" + String(key));
                if (key === "virtual") {
                    return {
                        value: 1,
                        writable: true,
                        enumerable: true,
                        configurable: true
                    };
                }
                return undefined;
            },
            get(target, key, receiver) {
                if (key === Symbol.toStringTag) {
                    log.push("get:@@toStringTag");
                    return "Virtual";
                }
                log.push("get:" + String(key));
                return undefined;
            }
        });

        let hasOwn = "unset";
        let tag = "unset";
        let hasOwnError = "none";
        let tagError = "none";
        try {
            hasOwn = String(Object.prototype.hasOwnProperty.call(proxy, "virtual"));
        } catch (error) {
            hasOwnError = error.constructor === TypeError ? "TypeError" : "other";
        }
        try {
            tag = Object.prototype.toString.call(proxy);
        } catch (error) {
            tagError = error.constructor === TypeError ? "TypeError" : "other";
        }

        [
            hasOwn,
            tag,
            hasOwnError,
            tagError,
            log.join("|")
        ].join(";");
        "#,
    );

    assert_eq!(
        result,
        "true;[object Virtual];none;none;getOwnPropertyDescriptor:virtual|get:@@toStringTag"
    );
}

#[test]
fn phase6_proxy_enforces_handler_invariants_for_non_configurable_target_state() {
    let result = compile_and_run(
        r#"
        let total = 0;

        let fixed = {};
        Object.defineProperty(fixed, "sealed", {
            value: 7,
            writable: false,
            enumerable: true,
            configurable: false
        });

        try {
            Object.getOwnPropertyDescriptor(
                new Proxy(fixed, {
                    getOwnPropertyDescriptor() {
                        return undefined;
                    }
                }),
                "sealed",
            );
        } catch (error) {
            total += (error.constructor === TypeError ? 1 : 0);
        }

        try {
            ("sealed" in new Proxy(fixed, {
                has() {
                    return false;
                }
            }));
        } catch (error) {
            total += (error.constructor === TypeError ? 2 : 0);
        }

        try {
            delete new Proxy(fixed, {
                deleteProperty() {
                    return true;
                }
            }).sealed;
        } catch (error) {
            total += (error.constructor === TypeError ? 4 : 0);
        }

        try {
            Object.keys(new Proxy(fixed, {
                ownKeys() {
                    return [];
                }
            }));
        } catch (error) {
            total += (error.constructor === TypeError ? 8 : 0);
        }

        try {
            Object.defineProperty(
                new Proxy({}, {
                    defineProperty() {
                        return true;
                    }
                }),
                "late",
                {
                    value: 1,
                    configurable: false
                }
            );
        } catch (error) {
            total += (error.constructor === TypeError ? 16 : 0);
        }

        try {
            let proxy = new Proxy(fixed, {
                get() {
                    return 9;
                }
            });
            proxy.sealed;
        } catch (error) {
            total += (error.constructor === TypeError ? 32 : 0);
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn phase5_error_families_allocate_real_bootstrapped_objects() {
    let result = compile_and_run(
        r#"
        let direct = TypeError("bad");
        let constructed = new RangeError("range");
        let caughtName = "";
        let caughtPrototype = 0;
        try {
            new Symbol();
        } catch (error) {
            caughtName = error.name;
            caughtPrototype = Object.getPrototypeOf(error) === TypeError.prototype ? 1 : 0;
        }
        (direct.name === "TypeError" ? 1 : 0)
            + (direct.message === "bad" ? 2 : 0)
            + (Error.prototype.toString.call(direct) === "TypeError: bad" ? 4 : 0)
            + (Object.getPrototypeOf(direct) === TypeError.prototype ? 8 : 0)
            + (constructed.name === "RangeError" ? 16 : 0)
            + (constructed.message === "range" ? 32 : 0)
            + (Error.prototype.toString.call(constructed) === "RangeError: range" ? 64 : 0)
            + (Object.getPrototypeOf(constructed) === RangeError.prototype ? 128 : 0)
            + (caughtName === "TypeError" ? 256 : 0)
            + (caughtPrototype ? 512 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(1023));
}

#[test]
fn phase5_cross_realm_foundations_use_selected_builtin_and_receiver_realms() {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(
        r#"
        let created = OtherObject.create(OtherObject.prototype);
        let otherError = OtherTypeError("boom");
        (Object.getPrototypeOf(created) === OtherObject.prototype ? 1 : 0)
            + (Object.prototype.toString.call(OtherFunction) === "[object Function]" ? 2 : 0)
            + (Object.prototype.toString.call(otherError) === "[object Error]" ? 4 : 0)
            + (Object.getPrototypeOf(otherError) === OtherTypeError.prototype ? 8 : 0);
        "#,
        &mut atoms,
    );
    let mut registry = RejectingRegistry;
    let result = evaluate_with_registry(
        &unit,
        |agent, realm| {
            let extra_realm = agent.create_default_realm_shell(AllocationLifetime::Default);
            let mut cache = BuiltinCache::new();
            bootstrap_realm(
                agent,
                &mut cache,
                extra_realm,
                BootstrapRequest::new(BootstrapMode::SpecOnly),
            )
            .expect("extra realm bootstrap should succeed");
            let extra_intrinsics = agent
                .realm(extra_realm)
                .expect("extra realm should exist")
                .intrinsics();
            let other_object = extra_intrinsics
                .object()
                .expect("other realm Object should exist");
            let other_function = extra_intrinsics
                .function()
                .expect("other realm Function should exist");
            let other_type_error = extra_intrinsics
                .type_error()
                .expect("other realm TypeError should exist");

            for (name, value) in [
                ("OtherObject", other_object),
                ("OtherFunction", other_function),
                ("OtherTypeError", other_type_error),
            ] {
                let atom = agent.atoms_mut().intern_collectible(name);
                assert!(ordinary_create_data_property(
                    agent,
                    realm.global_object(),
                    PropertyKey::from_atom(atom),
                    Value::from_object_ref(value),
                    AllocationLifetime::Default,
                )
                .expect("cross-realm global should install"));
            }
        },
        &mut registry,
    );

    assert_eq!(result, Value::from_smi(15));
}

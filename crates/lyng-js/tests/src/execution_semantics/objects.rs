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
use lyng_js_ops::object::create_data_property;
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
fn phase5_objects_support_null_prototypes_and_integrity_helpers() {
    let result = compile_and_run(
        r#"
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
        "#,
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
            + Object.prototype.toString.call(new TypeError("boom"));
        "#,
    );

    assert_eq!(
        result,
        "[object Object]|[object Function]|[object Array]|[object Error]"
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
        r#"
        let total = 0;
        total += Object.getPrototypeOf(Boolean) === Function.prototype ? 1 : 0;
        total += Object.getPrototypeOf(RangeError) === Error ? 2 : 0;
        total += Object.getPrototypeOf(TypeError) === Error ? 4 : 0;
        total += Object.getPrototypeOf(function() { return arguments; }()) === Object.prototype
            ? 8
            : 0;
        total;
        "#,
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
                assert!(create_data_property(
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

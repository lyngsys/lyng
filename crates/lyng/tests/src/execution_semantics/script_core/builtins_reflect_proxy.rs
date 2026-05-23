use super::super::support::*;

#[test]
fn script_core_supports_object_has_own_property_builtin_shim() {
    let result = compile_and_run(
        r#"
        let own = { answer: 1 };
        (own.hasOwnProperty("answer") ? 1 : 0)
            + (own.hasOwnProperty("missing") ? 10 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_internal_spec_like_builtin_shims_use_public_semantics() {
    let result = compile_and_run_with_globals(
        r#"
        let score = 0;
        let call = Function.prototype.call;

        try {
            score += call.call(internalStringIndexOf, "😀x", "x") === 2 ? 1 : 0;
        } catch (_error) {}

        try {
            score += call.call(internalStringReplace, "abc", "b", "$&$&") === "abbc" ? 2 : 0;
        } catch (_error) {}

        try {
            let pushTarget = { length: new Number(1.9) };
            score += call.call(internalArrayPush, pushTarget, "x") === 2 ? 4 : 0;
            score += pushTarget[1] === "x" && pushTarget.length === 2 ? 8 : 0;
        } catch (_error) {}

        try {
            let popTarget = { 0: "first", 1: "last", length: new Number(2.8) };
            score += call.call(internalArrayPop, popTarget) === "last" ? 16 : 0;
            score += popTarget.length === 1 && popTarget[1] === undefined ? 32 : 0;
        } catch (_error) {}

        try {
            let searchTarget = { 2: "needle", length: new Number(3.9) };
            score += call.call(internalArrayIndexOf, searchTarget, "needle") === 2 ? 64 : 0;
        } catch (_error) {}

        try {
            score += call.call(internalObjectToString, new Date(0)) === "[object Date]" ? 128 : 0;
        } catch (_error) {}

        try {
            let tagged = {};
            tagged[Symbol.toStringTag] = "Tagged";
            score += call.call(internalObjectToString, tagged) === "[object Tagged]" ? 256 : 0;
        } catch (_error) {}

        try {
            let inherited = Object.create({ answer: 1 });
            score += call.call(internalHasOwnProperty, inherited, "answer") === false ? 512 : 0;
        } catch (_error) {}

        score;
        "#,
        |agent, realm| {
            install_native_global(
                agent,
                &realm,
                "internalStringIndexOf",
                internal_string_index_of_builtin(),
                false,
            );
            install_native_global(
                agent,
                &realm,
                "internalStringReplace",
                internal_string_replace_builtin(),
                false,
            );
            install_native_global(
                agent,
                &realm,
                "internalArrayIndexOf",
                internal_array_index_of_builtin(),
                false,
            );
            install_native_global(
                agent,
                &realm,
                "internalArrayPush",
                internal_array_push_builtin(),
                false,
            );
            install_native_global(
                agent,
                &realm,
                "internalArrayPop",
                internal_array_pop_builtin(),
                false,
            );
            install_native_global(
                agent,
                &realm,
                "internalObjectToString",
                internal_object_to_string_builtin(),
                false,
            );
            install_native_global(
                agent,
                &realm,
                "internalHasOwnProperty",
                internal_object_has_own_property_builtin(),
                false,
            );
        },
    );

    assert_eq!(result, Value::from_smi(1023));
}

#[test]
fn script_core_supports_for_in_head_destructuring_bindings() {
    let result = compile_and_run(
        r#"
        let seen = 0;
        for (var [x, x] in { ab: null }) {
            seen = seen + (x === "b" ? 1 : 100);
        }
        seen;
        "#,
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_preserves_outer_var_bindings_across_for_in_lexical_heads() {
    let result = compile_and_run_string(
        r#"
        var probeBefore = function() { return x; };
        var probeExpr, probeDecl, probeBody;
        var x = 1;

        for (
            let [_ = probeDecl = function() { return x; }]
            in
            { '': probeExpr = function() { return x; }}
        )
            var x = 2, __ = probeBody = function() { return x; };

        probeBefore() + ":" + probeExpr() + ":" + probeDecl() + ":" + probeBody() + ":" + x;
        "#,
    );

    assert_eq!(result, "2:2:2:2:2");
}

#[test]
fn script_core_uses_reference_errors_for_for_in_lexical_head_tdz() {
    let result = compile_and_run_string(
        r#"
        try {
            let x = 1;
            for (let x in { x }) {}
            "no-error";
        } catch (error) {
            error.name;
        }
        "#,
    );

    assert_eq!(result, "ReferenceError");
}

#[test]
fn script_core_scopes_for_in_lexical_head_and_body_closures_correctly() {
    let result = compile_and_run_string(
        r#"
        let x = 'outside';
        var probeBefore = function() { return x; };
        var probeExpr, probeDecl, probeBody;

        try {
            for (
                let [x, _ = probeDecl = function() { return x; }]
                in
                { i: probeExpr = function() { typeof x; } }
            )
                probeBody = function() { return x; };
            "loop-ok";
        } catch (error) {
            "caught:" + error.name;
        }

        let exprResult;
        try {
            exprResult = probeExpr();
        } catch (error) {
            exprResult = error.name;
        }

        probeBefore() + ":" + exprResult + ":" +
            (probeDecl ? probeDecl() : "nodecl") + ":" +
            (probeBody ? probeBody() : "nobody");
        "#,
    );

    assert_eq!(result, "outside:ReferenceError:i:i");
}

#[test]
fn script_core_lowers_named_group_match_result_destructuring() {
    let mut atoms = AtomTable::new();
    let _ = compile_unit(
        r#"
        let {a, b} = "bab".match(/(?<b>b)\k<a>(?<a>a)\k<b>/u).groups;
        a + b;
        "#,
        &mut atoms,
    );
}

#[test]
fn script_core_lowers_var_declarators_that_share_names_with_hoisted_functions() {
    let mut atoms = AtomTable::new();
    let _ = compile_unit(
        r"
        var __string;
        var __re = /1|12/;
        __re.exec(__string);
        __re.test(__string);
        function __string() {}
        ",
        &mut atoms,
    );
}

#[test]
fn script_core_installs_phase6_numeric_globals_and_reflect_namespace() {
    let result = compile_and_run_string(
        r#"
        [
            typeof Number,
            typeof Math,
            typeof BigInt,
            typeof RegExp,
            typeof Reflect,
            typeof Reflect.apply,
            typeof Reflect.construct,
            typeof Reflect.defineProperty,
            typeof Reflect.deleteProperty,
            typeof Reflect.get,
            typeof Reflect.getOwnPropertyDescriptor,
            typeof Reflect.getPrototypeOf,
            typeof Reflect.has,
            typeof Reflect.isExtensible,
            typeof Reflect.ownKeys,
            typeof Reflect.preventExtensions,
            typeof Reflect.set,
            typeof Reflect.setPrototypeOf,
            typeof Reflect.enumerate
        ].join(":");
        "#,
    );

    assert_eq!(
        result,
        "function:object:function:function:object:function:function:function:function:function:function:function:function:function:function:function:function:function:undefined"
    );
}

#[test]
fn script_core_reflect_dispatches_basic_object_operations() {
    let result = compile_and_run(
        r#"
        let total = 0;
        let target = { answer: 1 };
        let proto = { marker: 2 };

        total += (Reflect.get(target, "answer") === 1 ? 1 : 0);
        total += (Reflect.has(target, "answer") ? 2 : 0);
        total += (Reflect.set(target, "answer", 5) && target.answer === 5 ? 4 : 0);
        total += (Reflect.defineProperty(target, "sealed", {
            value: 7,
            configurable: false,
            enumerable: true,
            writable: false
        }) ? 8 : 0);
        total += (Reflect.getOwnPropertyDescriptor(target, "sealed").value === 7 ? 16 : 0);
        total += (Reflect.deleteProperty(target, "answer") && !Reflect.has(target, "answer") ? 32 : 0);
        total += (Reflect.setPrototypeOf(target, proto) && Reflect.getPrototypeOf(target) === proto ? 64 : 0);
        total += (Reflect.preventExtensions(target) && !Reflect.isExtensible(target) ? 128 : 0);
        total += (Reflect.ownKeys(target).join(",") === "sealed" ? 256 : 0);
        total += (Reflect.construct(function Box(value) { this.value = value; }, [11]).value === 11 ? 512 : 0);
        total += (Reflect.apply(function(a, b) { return this.base + a + b; }, { base: 3 }, [4, 5]) === 12 ? 1024 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(2047));
}

#[test]
fn script_core_reflect_uses_accessor_paths_and_validates_argument_lists() {
    let result = compile_and_run(
        r#"
        let total = 0;
        let accessorBase = {};
        let receiver = { marker: 41, stored: 0 };

        Object.defineProperty(accessorBase, "value", {
            get: function() {
                return this.marker;
            },
            set: function(next) {
                this.stored = next;
            }
        });

        total += (Reflect.get(accessorBase, "value", receiver) === 41 ? 1 : 0);
        total += (Reflect.get(Object.create(accessorBase), "value", receiver) === 41 ? 2 : 0);
        total += (Reflect.set(accessorBase, "value", 7, receiver) && receiver.stored === 7 ? 4 : 0);

        let throwing = {};
        Object.defineProperty(throwing, "value", {
            set: function() {
                throw "setter";
            }
        });
        try {
            Reflect.set(throwing, "value", 1);
        } catch (error) {
            total += (error === "setter" ? 8 : 0);
        }

        let receiverAccessor = {};
        Object.defineProperty(receiverAccessor, "slot", {
            set: function(value) {}
        });
        total += (Reflect.set({}, "slot", 9, receiverAccessor) === false ? 16 : 0);

        try {
            Reflect.apply(function() {}, null, undefined);
        } catch (error) {
            total += (error.constructor === TypeError ? 32 : 0);
        }

        try {
            Reflect.construct(function() {}, null);
        } catch (error) {
            total += (error.constructor === TypeError ? 64 : 0);
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn script_core_installs_proxy_constructor_and_revocable_pair_surface() {
    let result = compile_and_run_string(
        r#"
        let revocableType = "missing";
        let pairTypes = "missing";
        if (typeof Proxy === "function") {
            revocableType = typeof Proxy.revocable;
            if (revocableType === "function") {
                let pair = Proxy.revocable({}, {});
                pairTypes = typeof pair.proxy + ":" + typeof pair.revoke;
            }
        }
        typeof Proxy + ":" + revocableType + ":" + pairTypes;
        "#,
    );

    assert_eq!(result, "function:function:object:function");
}

#[test]
fn script_core_proxy_is_construct_only_and_validates_constructor_arguments() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += (typeof Proxy === "function" ? 1 : 0);
        total += (Object.prototype.hasOwnProperty.call(Proxy, "prototype") ? 0 : 2);
        try {
            Proxy({}, {});
        } catch (error) {
            total += (error.constructor === TypeError ? 4 : 0);
        }
        try {
            new Proxy(1, {});
        } catch (error) {
            total += (error.constructor === TypeError ? 8 : 0);
        }
        try {
            new Proxy({}, 1);
        } catch (error) {
            total += (error.constructor === TypeError ? 16 : 0);
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_proxy_revocable_revokes_idempotently_and_throws_afterward() {
    let result = compile_and_run(
        r"
        let pair = Proxy.revocable({ answer: 1 }, {});
        let total = 0;
        total += (pair.proxy.answer === 1 ? 1 : 0);
        total += (pair.revoke() === undefined ? 2 : 0);
        try {
            pair.proxy.answer;
        } catch (error) {
            total += (error.constructor === TypeError ? 4 : 0);
        }
        try {
            Object.getPrototypeOf(pair.proxy);
        } catch (error) {
            total += (error.constructor === TypeError ? 8 : 0);
        }
        total += (pair.revoke() === undefined ? 16 : 0);
        total;
        ",
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_callable_proxies_reach_typeof_call_and_construct_paths() {
    let result = compile_and_run(
        r#"
        let target = function(value) {
            if (new.target) {
                this.value = value;
                return;
            }
            return value + 1;
        };
        target.prototype.marker = 1;
        let proxy = new Proxy(target, {});
        let total = 0;

        try {
            total += (typeof proxy === "function" ? 1 : 0);
        } catch (error) {
            total += 1024;
        }
        try {
            total += (proxy(4) === 5 ? 2 : 0);
        } catch (error) {
            total += 2048;
        }
        try {
            total += (Function.prototype.call.call(proxy, null, 5) === 6 ? 4 : 0);
        } catch (error) {
            total += 4096;
        }
        try {
            total += (Reflect.apply(proxy, null, [6]) === 7 ? 8 : 0);
        } catch (error) {
            total += 8192;
        }
        try {
            let instance = new proxy(8);
            total += (instance.value === 8 ? 16 : 0);
            total += (Object.getPrototypeOf(instance) === target.prototype ? 32 : 0);
        } catch (error) {
            total += 16384;
        }
        try {
            let constructed = Reflect.construct(proxy, [9]);
            total += (constructed.value === 9 ? 64 : 0);
        } catch (error) {
            total += 32768;
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn script_core_proxy_named_property_caches_stay_proxy_aware() {
    let result = compile_and_run(
        r#"
        let gets = 0;
        let sets = 0;
        let target = { value: 1 };
        let proxy = new Proxy(target, {
            get(target, key, receiver) {
                if (key === "value") {
                    gets += 1;
                    return gets * 10;
                }
                return target[key];
            },
            set(target, key, value, receiver) {
                if (key === "value") {
                    sets += 1;
                    target[key] = value + 1;
                    return true;
                }
                target[key] = value;
                return true;
            }
        });

        let total = 0;
        total += proxy.value;
        total += proxy.value;
        proxy.value = 4;
        proxy.value = 7;
        total += proxy.value;
        total += (gets === 3 ? 1 : 0);
        total += (sets === 2 ? 2 : 0);
        total += (target.value === 8 ? 4 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(67));
}

#[test]
fn script_core_proxy_null_traps_forward_and_own_keys_require_property_keys() {
    let result = compile_and_run(
        r"
        let total = 0;
        let target = { value: 3 };
        let proxy = new Proxy(target, { get: null, set: null });
        total += (proxy.value === 3 ? 1 : 0);
        proxy.value = 7;
        total += (target.value === 7 ? 2 : 0);

        let callable = new Proxy(function(value) { return value + 1; }, { apply: null });
        total += (callable(4) === 5 ? 4 : 0);

        let constructible = new Proxy(function(value) { this.value = value; }, { construct: null });
        let instance = new constructible(6);
        total += (instance.value === 6 ? 8 : 0);

        try {
            Object.keys(new Proxy({}, {
                ownKeys: function() {
                    return [true];
                }
            }));
        } catch (error) {
            total += (error.constructor === TypeError ? 16 : 0);
        }

        total;
        ",
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_proxy_invariants_cover_define_delete_prototype_and_prevent_extensions() {
    let result = compile_and_run(
        r#"
        let total = 0;

        let defineTrapCalls = 0;
        let defineProxy = new Proxy({}, {
            defineProperty: function(target, prop, desc) {
                Object.defineProperty(target, prop, {
                    configurable: false,
                    writable: true
                });
                defineTrapCalls += 1;
                return true;
            }
        });
        try {
            Reflect.defineProperty(defineProxy, "prop", { writable: false });
        } catch (error) {
            total += (error instanceof TypeError ? 1 : 0);
        }
        total += (defineTrapCalls === 1 ? 2 : 0);

        let deleteTrapCalls = 0;
        let deleteProxy = new Proxy({ prop: 1 }, {
            deleteProperty: function(target, prop) {
                Object.preventExtensions(target);
                deleteTrapCalls += 1;
                return true;
            }
        });
        try {
            Reflect.deleteProperty(deleteProxy, "prop");
        } catch (error) {
            total += (error instanceof TypeError ? 4 : 0);
        }
        total += (deleteTrapCalls === 1 ? 8 : 0);
        total += (Reflect.deleteProperty(deleteProxy, "missing") ? 16 : 0);
        total += (deleteTrapCalls === 2 ? 32 : 0);

        function Custom() {}
        let prototypeProxy = new Proxy({}, {
            getPrototypeOf: function() {
                return Custom.prototype;
            }
        });
        total += (prototypeProxy instanceof Custom ? 64 : 0);

        let observedHandler;
        let observedTarget;
        let observedProp;
        let hasTarget = {};
        let hasHandler = {
            has: function(target, prop) {
                observedHandler = this;
                observedTarget = target;
                observedProp = prop;
                return false;
            }
        };
        "attr" in Object.create(new Proxy(hasTarget, hasHandler));
        total += (observedHandler === hasHandler ? 128 : 0);
        total += (observedTarget === hasTarget ? 256 : 0);
        total += (observedProp === "attr" ? 512 : 0);

        let innerPrevent = new Proxy({}, {
            preventExtensions: function() {
                return false;
            }
        });
        try {
            Object.preventExtensions(new Proxy(innerPrevent, {}));
        } catch (error) {
            total += (error instanceof TypeError ? 1024 : 0);
        }

        let setProtoCalls = [];
        let setProto = {};
        let setProtoTarget = new Proxy(Object.create(setProto), {
            isExtensible: function() {
                setProtoCalls.push("target.[[IsExtensible]]");
                return false;
            },
            getPrototypeOf: function() {
                setProtoCalls.push("target.[[GetPrototypeOf]]");
                return setProto;
            }
        });
        Object.preventExtensions(setProtoTarget);
        let setProtoProxy = new Proxy(setProtoTarget, {
            setPrototypeOf: function() {
                setProtoCalls.push("proxy.[[SetPrototypeOf]]");
                return true;
            }
        });
        total += (setProtoCalls.length === 0 ? 2048 : 0);
        total += (Reflect.setPrototypeOf(setProtoProxy, setProto) ? 4096 : 0);
        total += (
            setProtoCalls.join(",") ===
            "proxy.[[SetPrototypeOf]],target.[[IsExtensible]],target.[[GetPrototypeOf]]"
                ? 8192
                : 0
        );

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(16383));
}

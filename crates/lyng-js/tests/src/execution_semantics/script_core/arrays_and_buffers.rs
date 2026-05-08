use super::super::support::*;

#[test]
fn script_core_supports_array_index_of_builtin_shim() {
    let result = compile_and_run(
        r"
        let values = [1, 2, 3];
        values.indexOf(2);
        ",
    );

    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn script_core_array_search_helpers_are_generic() {
    let result = compile_and_run(
        r"
        let score = 0;
        Math[1] = true;
        Math.length = 2;
        score += Array.prototype.indexOf.call(Math, true) === 1 ? 1 : 0;
        score += Array.prototype.indexOf.call([1, 2, 1], 1, 1) === 2 ? 2 : 0;
        score += Array.prototype.includes.call(true) === false ? 4 : 0;
        score += [0, NaN].includes(NaN) ? 8 : 0;
        score;
        ",
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_array_join_uses_to_length_for_generic_receivers() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let object = { 0: "x", 1: null, 2: "z" };

        score += Array.prototype.join.call(object) === "" ? 1 : 0;
        score += object.length === undefined ? 2 : 0;

        object.length = null;
        score += Array.prototype.join.call(object) === "" ? 4 : 0;
        score += object.length === null ? 8 : 0;

        object.length = 2.8;
        score += Array.prototype.join.call(object, "|") === "x|" ? 16 : 0;
        score += object.length === 2.8 ? 32 : 0;

        let wrappedLength = new Number(3.9);
        object.length = wrappedLength;
        score += Array.prototype.join.call(object, "|") === "x||z" ? 64 : 0;
        score += object.length === wrappedLength ? 128 : 0;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(255));
}

#[test]
fn script_core_array_pop_push_use_length_of_array_like() {
    let result = compile_and_run(
        r#"
        let score = 0;

        let popTarget = { 0: "first" };
        score += Array.prototype.pop.call(popTarget) === undefined ? 1 : 0;
        score += popTarget.length === 0 ? 2 : 0;

        popTarget[1] = "last";
        popTarget.length = 2.8;
        score += Array.prototype.pop.call(popTarget) === "last" ? 4 : 0;
        score += popTarget.length === 1 ? 8 : 0;
        score += popTarget[1] === undefined ? 16 : 0;

        let pushTarget = {};
        let wrappedLength = new Number(1.9);
        pushTarget.length = wrappedLength;
        score += Array.prototype.push.call(pushTarget, "x") === 2 ? 32 : 0;
        score += pushTarget[1] === "x" ? 64 : 0;
        score += pushTarget.length === 2 ? 128 : 0;

        let limitTarget = { length: Infinity };
        score += Array.prototype.push.call(limitTarget) === Number.MAX_SAFE_INTEGER ? 256 : 0;
        score += limitTarget.length === Number.MAX_SAFE_INTEGER ? 512 : 0;

        score += Array.prototype.push.call(true) === 0 ? 1024 : 0;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(2047));
}

#[test]
fn script_core_array_push_observes_inherited_setter_before_length_write() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let array = [];
        let calls = 0;

        Object.defineProperty(Array.prototype, "0", {
            set: function(_value) {
                Object.freeze(array);
                calls++;
            },
            configurable: true
        });

        try {
            array.push(1);
        } catch (error) {
            score += error.constructor === TypeError ? 1 : 0;
        } finally {
            delete Array.prototype[0];
        }

        score += !array.hasOwnProperty("0") ? 2 : 0;
        score += array.length === 0 ? 4 : 0;
        score += calls === 1 ? 8 : 0;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_array_concat_boxes_receiver_and_defines_result_elements() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let boxed = Array.prototype.concat.call(true);
        score += boxed[0] instanceof Boolean ? 1 : 0;

        Object.defineProperty(Array.prototype, "0", {
            value: "inherited",
            writable: false,
            configurable: true
        });

        try {
            let result = Array.prototype.concat.call([101]);
            let descriptor = Object.getOwnPropertyDescriptor(result, "0");
            score += result[0] === 101 ? 2 : 0;
            score += descriptor.writable === true ? 4 : 0;
            score += descriptor.enumerable === true ? 8 : 0;
            score += descriptor.configurable === true ? 16 : 0;
        } finally {
            delete Array.prototype[0];
        }

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_array_result_helpers_define_indices_under_poisoned_array_prototype() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let symbol = Symbol("s");
        let symbolHolder = {};
        symbolHolder[symbol] = 1;

        Object.defineProperty(Array.prototype, "0", {
            value: "inherited",
            writable: false,
            configurable: true
        });

        try {
            let names = Object.getOwnPropertyNames({ value: 1, writable: true });
            let nameDesc = Object.getOwnPropertyDescriptor(names, "0");
            score += names[0] === "value" ? 1 : 0;
            score += nameDesc.writable === true ? 2 : 0;

            let keys = Object.keys({ a: 1 });
            score += keys[0] === "a" ? 4 : 0;

            let values = Object.values({ a: 1 });
            score += values[0] === 1 ? 8 : 0;

            let entries = Object.entries({ a: 1 });
            score += entries[0][0] === "a" ? 16 : 0;
            score += entries[0][1] === 1 ? 32 : 0;

            let symbols = Object.getOwnPropertySymbols(symbolHolder);
            score += symbols[0] === symbol ? 64 : 0;
        } finally {
            delete Array.prototype[0];
        }

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn script_core_array_is_array_observes_array_exotics_and_proxies() {
    let result = compile_and_run(
        r"
        let score = 0;

        score += Array.isArray(Array.prototype) ? 1 : 0;

        let objectProxy = new Proxy({}, {});
        let arrayProxy = new Proxy([], {});
        let nestedProxy = new Proxy(arrayProxy, {});
        score += Array.isArray(objectProxy) === false ? 2 : 0;
        score += Array.isArray(arrayProxy) === true ? 4 : 0;
        score += Array.isArray(nestedProxy) === true ? 8 : 0;

        let revoked = Proxy.revocable([], {});
        revoked.revoke();
        try {
            Array.isArray(revoked.proxy);
        } catch (error) {
            score += error.constructor === TypeError ? 16 : 0;
        }

        score;
        ",
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_array_length_define_property_coerces_before_descriptor_validation() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let array = [1, 2];
        let valueOfCalls = 0;
        let length = {
            valueOf: function() {
                valueOfCalls++;
                if (valueOfCalls !== 1) {
                    Object.defineProperty(array, "length", { writable: false });
                }
                return array.length;
            }
        };

        try {
            Object.defineProperty(array, "length", { value: length, writable: true });
        } catch (error) {
            score += error.constructor === TypeError ? 1 : 0;
        }
        score += valueOfCalls === 2 ? 2 : 0;

        array = [1, 2];
        valueOfCalls = 0;
        try {
            score += Reflect.defineProperty(array, "length", { value: length, writable: true }) === false ? 4 : 0;
        } catch (_error) {}
        score += valueOfCalls === 2 ? 8 : 0;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_array_length_set_coerces_before_writable_check() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let array = [1, 2, 3];
        let hints = [];
        let length = {};
        length[Symbol.toPrimitive] = function(hint) {
            hints.push(hint);
            Object.defineProperty(array, "length", { writable: false });
            return 0;
        };

        try {
            (function() {
                "use strict";
                array.length = length;
            })();
        } catch (error) {
            score += error.constructor === TypeError ? 1 : 0;
        }
        score += hints.length === 2 && hints[0] === "number" && hints[1] === "number" ? 2 : 0;

        array = [1, 2, 3];
        hints = [];
        try {
            score += Reflect.set(array, "length", length) === false ? 4 : 0;
        } catch (_error) {}
        score += hints.length === 2 && hints[0] === "number" && hints[1] === "number" ? 8 : 0;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_array_reverse_gets_lower_before_testing_upper() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let array = ["first", "second"];

        Object.defineProperty(array, 0, {
            get: function() {
                array.length = 0;
                return "first";
            },
            configurable: true
        });

        array.reverse();
        score += (0 in array) === false ? 1 : 0;
        score += (1 in array) === true ? 2 : 0;
        score += array[1] === "first" ? 4 : 0;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn script_core_in_operator_rejects_primitive_rhs_with_type_error() {
    let result = compile_and_run(
        r#"
        let values = [true, 1, "text", undefined, null];
        let total = 0;

        for (let value of values) {
            try {
                "toString" in value;
                total = total + 100;
            } catch (error) {
                total = total + (error instanceof TypeError ? 1 : 10);
            }
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(5));
}

#[test]
fn script_core_array_slice_uses_species_and_defines_result_elements() {
    let result = compile_and_run(
        r#"
        let score = 0;

        function Ctor(length) {
            this.lengthSeen = length;
        }

        let array = [10, 20];
        array.constructor = {};
        array.constructor[Symbol.species] = Ctor;

        let result = array.slice(0, 1);
        score += Object.getPrototypeOf(result) === Ctor.prototype ? 1 : 0;
        score += result.lengthSeen === 1 ? 2 : 0;
        score += result[0] === 10 ? 4 : 0;

        Object.defineProperty(Ctor.prototype, "0", {
            value: "inherited",
            writable: false,
            configurable: true
        });

        try {
            result = array.slice(0, 1);
            score += result.hasOwnProperty("0") ? 8 : 0;
            score += result[0] === 10 ? 16 : 0;
        } finally {
            delete Ctor.prototype[0];
        }

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_array_predicate_helpers_are_generic() {
    let result = compile_and_run(
        r"
        let score = 0;
        let obj = new Date(0);
        obj.length = 2;
        obj[0] = 1;
        obj[1] = 2;
        score += Array.prototype.every.call(obj, function(value, index, receiver) {
            return receiver instanceof Date && value < 2 && index === 0;
        }) === false ? 1 : 0;
        score += Array.prototype.some.call(obj, function(value, index, receiver) {
            return receiver instanceof Date && value === 2 && index === 1;
        }) ? 2 : 0;
        score;
        ",
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_array_generics_observe_resizable_typed_array_oob_state() {
    let result = compile_and_run_string(
        r#"
        let rab = new ArrayBuffer(4, { maxByteLength: 8 });
        let fixed = new Uint8Array(rab, 0, 4);
        fixed[0] = 1;
        fixed[1] = 2;
        fixed[2] = 3;
        fixed[3] = 4;
        let seen = [];
        let every = Array.prototype.every.call(fixed, function(value) {
            seen.push(value);
            if (seen.length === 2) {
                rab.resize(2);
            }
            return true;
        });
        let lengthAfterShrink = fixed.length;
        let atAfterShrink = Array.prototype.at.call(fixed, 0);
        let iteratorThrows = false;
        try {
            Array.from(Array.prototype.keys.call(fixed));
        } catch (error) {
            iteratorThrows = error instanceof TypeError;
        }

        let rab2 = new ArrayBuffer(4, { maxByteLength: 8 });
        let fixed2 = new Uint8Array(rab2, 0, 4);
        let evil = {
            valueOf: function() {
                rab2.resize(2);
                return 9;
            }
        };
        Array.prototype.fill.call(fixed2, evil, 0, 1);
        let live = new Uint8Array(rab2);

        let strictIndex = (function() {
            "use strict";
            let rab3 = new ArrayBuffer(16, { maxByteLength: 32 });
            let floats = new Float32Array(rab3);
            floats[0] = -Infinity;
            floats[1] = -Infinity;
            floats[2] = Infinity;
            floats[3] = Infinity;
            floats[4] = NaN;
            return Array.prototype.indexOf.call(floats, Infinity);
        })();

        let rab4 = new ArrayBuffer(4, { maxByteLength: 8 });
        let tracking = new Uint8Array(rab4);
        let joined = Array.prototype.join.call(tracking, {
            toString: function() {
                rab4.resize(6);
                return ".";
            }
        });

        [
            every,
            seen.join(","),
            lengthAfterShrink,
            String(atAfterShrink),
            iteratorThrows,
            live.length + ":" + live[0] + "," + live[1],
            strictIndex,
            joined
        ].join("|");
        "#,
    );

    assert_eq!(result, "true|1,2|0|undefined|true|2:0,0|2|0.0.0.0");
}

#[test]
fn script_core_array_buffer_resizable_accessors_and_transfer() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let fixed = new ArrayBuffer(4);
        let resizable = new ArrayBuffer(4, { maxByteLength: 8 });

        score += fixed.resizable === false ? 1 : 0;
        score += fixed.maxByteLength === 4 ? 2 : 0;
        score += resizable.resizable === true ? 4 : 0;
        score += resizable.maxByteLength === 8 ? 8 : 0;
        score += fixed.detached === false ? 16 : 0;

        let fixedView = new Uint8Array(fixed);
        fixedView[0] = 7;
        fixedView[3] = 9;
        let moved = fixed.transfer(6);
        let movedView = new Uint8Array(moved);

        score += fixed.detached === true && fixed.byteLength === 0 ? 32 : 0;
        score += moved.byteLength === 6 && moved.maxByteLength === 6 && moved.resizable === false ? 64 : 0;
        score += movedView[0] === 7 && movedView[3] === 9 && movedView[4] === 0 ? 128 : 0;

        let resView = new Uint8Array(resizable);
        resView[0] = 3;
        resView[1] = 5;
        let grown = resizable.transfer(6);
        let grownView = new Uint8Array(grown);

        score += resizable.detached === true ? 256 : 0;
        score += grown.resizable === true && grown.maxByteLength === 8 && grown.byteLength === 6 ? 512 : 0;
        score += grownView[0] === 3 && grownView[1] === 5 && grownView[5] === 0 ? 1024 : 0;

        let fixedAgain = grown.transferToFixedLength(2);
        score += grown.detached === true ? 2048 : 0;
        score += fixedAgain.resizable === false && fixedAgain.maxByteLength === 2 && fixedAgain.byteLength === 2 ? 4096 : 0;
        score += new ArrayBuffer(0, null).resizable === false ? 8192 : 0;

        let sharedThrows = false;
        let resizableGetter = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "resizable").get;
        try {
            resizableGetter.call(new SharedArrayBuffer(1));
        } catch (error) {
            sharedThrows = error instanceof TypeError;
        }
        score += sharedThrows ? 16384 : 0;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(32_767));
}

#[test]
fn script_core_object_freeze_throws_for_resizable_typed_arrays() {
    let result = compile_and_run_string(
        r#"
        function freezeThrows(view) {
            try {
                Object.freeze(view);
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        }
        function preventExtensionsThrows(view) {
            try {
                Object.preventExtensions(view);
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        }
        function sealThrows(view) {
            try {
                Object.seal(view);
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        }
        function reflectPreventExtensionsFails(view) {
            return Reflect.preventExtensions(view) === false &&
                Object.isExtensible(view) === true;
        }
        function resizableView(length) {
            return new Uint8Array(
                new ArrayBuffer(4, { maxByteLength: 8 }),
                0,
                length
            );
        }

        let rab = new ArrayBuffer(4, { maxByteLength: 8 });
        let fixedLength = new Uint8Array(rab, 0, 4);
        let fixedZero = new Uint8Array(rab, 0, 0);
        let lengthTracking = new Uint8Array(rab);
        let lengthTrackingAtEnd = new Uint8Array(rab, 4);

        let shrunk = new ArrayBuffer(4, { maxByteLength: 8 });
        let trackingShrunk = new Uint8Array(shrunk);
        let trackingShrunkOffset = new Uint8Array(shrunk, 2);
        shrunk.resize(2);
        let offsetAfterShrink = freezeThrows(trackingShrunkOffset);
        shrunk.resize(0);

        [
            freezeThrows(fixedLength),
            freezeThrows(fixedZero),
            freezeThrows(lengthTracking),
            freezeThrows(lengthTrackingAtEnd),
            offsetAfterShrink,
            freezeThrows(trackingShrunk),
            preventExtensionsThrows(resizableView(0)),
            sealThrows(resizableView(0)),
            reflectPreventExtensionsFails(resizableView(0))
        ].join("|");
        "#,
    );

    assert_eq!(result, "true|true|true|true|true|true|true|true|true");
}

#[test]
fn script_core_typed_array_concrete_prototypes_inherit_generic_surface() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let base = Object.getPrototypeOf(Uint8Array.prototype);
        let inheritedNames = [
            "buffer", "byteLength", "byteOffset", "length",
            "values", "keys", "entries", "set", "slice", "subarray",
            "copyWithin", "every", "fill", "filter", "find", "findIndex",
            "findLast", "findLastIndex", "forEach", "includes", "indexOf",
            "join", "lastIndexOf", "map", "reduce", "reduceRight",
            "reverse", "some", "sort", "toLocaleString", "toString",
            "at", "with"
        ];
        let concreteOwnCommon = false;
        for (let i = 0; i < inheritedNames.length; i = i + 1) {
            let name = inheritedNames[i];
            if (Uint8Array.prototype.hasOwnProperty(name) ||
                BigInt64Array.prototype.hasOwnProperty(name)) {
                concreteOwnCommon = true;
            }
        }

        if (!concreteOwnCommon) score += 1;
        if (!Uint8Array.prototype.hasOwnProperty(Symbol.iterator) &&
            !BigInt64Array.prototype.hasOwnProperty(Symbol.iterator)) score += 2;
        if (!Uint8Array.prototype.hasOwnProperty(Symbol.toStringTag) &&
            !BigInt64Array.prototype.hasOwnProperty(Symbol.toStringTag)) score += 4;
        if (base.hasOwnProperty("buffer") &&
            base.hasOwnProperty("values") &&
            base.hasOwnProperty(Symbol.iterator) &&
            base.hasOwnProperty(Symbol.toStringTag)) score += 8;
        if (Uint8Array.prototype.hasOwnProperty("constructor") &&
            Uint8Array.prototype.hasOwnProperty("BYTES_PER_ELEMENT") &&
            BigInt64Array.prototype.hasOwnProperty("constructor") &&
            BigInt64Array.prototype.hasOwnProperty("BYTES_PER_ELEMENT")) score += 16;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_typed_array_array_like_oversize_fails_before_element_get() {
    let result = compile_and_run_string(
        r#"
        function probe(Ctor, length) {
            let accessed = false;
            let source = { length: length };
            Object.defineProperty(source, "0", {
                get: function() {
                    accessed = true;
                    throw new TypeError("element access should not happen");
                }
            });
            try {
                new Ctor(source);
                return "missing";
            } catch (error) {
                return (error instanceof RangeError) + ":" + accessed;
            }
        }

        [
            probe(Uint8Array, 1073741825),
            probe(BigInt64Array, 134217729)
        ].join("|");
        "#,
    );

    assert_eq!(result, "true:false|true:false");
}

#[test]
fn script_core_typed_array_buffer_arg_undefined_length_uses_remaining_bytes() {
    let result = compile_and_run_string(
        r#"
        try {
            new Int16Array(new ArrayBuffer(1), 0, undefined);
            "missing";
        } catch (error) {
            String(error instanceof RangeError);
        }
        "#,
    );

    assert_eq!(result, "true");
}

#[test]
fn script_core_typed_array_buffer_arg_rechecks_detached_after_coercion() {
    let result = compile_and_run_string(
        r#"
        function detachOnOffset() {
            let buffer = new ArrayBuffer(6);
            let offset = {
                valueOf: function() {
                    buffer.transfer(0);
                    return 2;
                }
            };
            try {
                new Int16Array(buffer, offset);
                return "missing";
            } catch (error) {
                return error instanceof TypeError;
            }
        }

        function detachOnLength() {
            let buffer = new ArrayBuffer(6);
            let length = {
                valueOf: function() {
                    buffer.transfer(0);
                    return 1;
                }
            };
            try {
                new Int16Array(buffer, 0, length);
                return "missing";
            } catch (error) {
                return error instanceof TypeError;
            }
        }

        detachOnOffset() + ":" + detachOnLength();
        "#,
    );

    assert_eq!(result, "true:true");
}

#[test]
fn script_core_typed_array_from_of_rejects_short_custom_constructor_result() {
    let result = compile_and_run_string(
        r#"
        let TypedArray = Object.getPrototypeOf(Uint8Array);

        function probeFromIterable(Ctor, value) {
            let custom = function() {
                return new Ctor(1);
            };
            try {
                Ctor.from.call(custom, [value, value]);
                return "missing";
            } catch (error) {
                return error instanceof TypeError;
            }
        }

        function probeFromArrayLike(Ctor, value) {
            let custom = function() {
                return new Ctor(1);
            };
            try {
                Ctor.from.call(custom, { length: 2, 0: value, 1: value });
                return "missing";
            } catch (error) {
                return error instanceof TypeError;
            }
        }

        function probeOf(Ctor, first, second) {
            let custom = function() {
                return new Ctor(1);
            };
            try {
                TypedArray.of.call(custom, first, second);
                return "missing";
            } catch (error) {
                return error instanceof TypeError;
            }
        }

        [
            probeFromIterable(Uint8Array, 1),
            probeFromArrayLike(Uint8Array, 1),
            probeOf(Uint8Array, 1, 2),
            probeFromIterable(BigInt64Array, 1n),
            probeFromArrayLike(BigInt64Array, 1n),
            probeOf(BigInt64Array, 1n, 2n)
        ].join(":");
        "#,
    );

    assert_eq!(result, "true:true:true:true:true:true");
}

#[test]
fn script_core_typed_array_numeric_get_and_has_do_not_use_prototype() {
    let result = compile_and_run_string(
        r#"
        let typedArrayPrototype = Object.getPrototypeOf(Uint8Array.prototype);
        let thrower = {
            get: function() {
                throw new TypeError("prototype lookup");
            }
        };
        Object.defineProperty(typedArrayPrototype, "1.1", thrower);
        Object.defineProperty(typedArrayPrototype, "-0", thrower);
        Object.defineProperty(typedArrayPrototype, "-1", thrower);
        Object.defineProperty(typedArrayPrototype, "2", thrower);
        typedArrayPrototype["0.000001"] = "prototype";
        typedArrayPrototype["1"] = "prototype";

        let sample = new Uint8Array([42, 43]);
        let short = new Uint8Array(1);

        [
            sample["1.1"] === undefined,
            sample["-0"] === undefined,
            sample["-1"] === undefined,
            sample["2"] === undefined,
            Reflect.has(short, "1") === false,
            Reflect.has(short, "1.1") === false,
            Reflect.has(short, "0.000001") === false,
            Reflect.has(short, "-0") === false,
            Reflect.has(short, "-1") === false
        ].join(":");
        "#,
    );

    assert_eq!(result, "true:true:true:true:true:true:true:true:true");
}

#[test]
fn script_core_typed_array_define_rejects_invalid_numeric_indices() {
    let result = compile_and_run_string(
        r#"
        let sample = new Uint8Array([7]);
        let desc = Object.getOwnPropertyDescriptor(sample, "0");

        function probe(key) {
            try {
                Object.defineProperty(sample, key, desc);
                return "missing";
            } catch (error) {
                return error instanceof TypeError;
            }
        }

        [
            probe("1"),
            probe("-1"),
            probe("1.5"),
            probe("-0")
        ].join(":");
        "#,
    );

    assert_eq!(result, "true:true:true:true");
}

#[test]
fn script_core_typed_array_define_numeric_value_converts_and_handles_detach() {
    let result = compile_and_run_string(
        r#"
        function detachDuringNumberConversion() {
            let sample = new Uint8Array([17]);
            let result = Reflect.defineProperty(sample, "0", {
                value: {
                    valueOf: function() {
                        sample.buffer.transfer(0);
                        return 42;
                    }
                }
            });
            return result + ":" + (sample[0] === undefined);
        }

        function detachDuringBigIntConversion() {
            let sample = new BigInt64Array([17n]);
            let result = Reflect.defineProperty(sample, "0", {
                value: {
                    valueOf: function() {
                        sample.buffer.transfer(0);
                        return 42n;
                    }
                }
            });
            return result + ":" + (sample[0] === undefined);
        }

        let numberSample = new Uint8Array([0, 0]);
        let bigintSample = new BigInt64Array([0n, 0n]);

        [
            Reflect.defineProperty(numberSample, "0", { value: 257 }),
            numberSample[0] === 1,
            Reflect.defineProperty(bigintSample, "1", { value: 2n }),
            bigintSample[1] === 2n,
            detachDuringNumberConversion(),
            detachDuringBigIntConversion()
        ].join(":");
        "#,
    );

    assert_eq!(result, "true:true:true:true:true:true:true:true");
}

#[test]
fn script_core_typed_array_define_property_uses_live_resizable_bounds() {
    let result = compile_and_run_string(
        r#"
        function throwsTypeError(body) {
            try {
                body();
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        }

        let buffer = new ArrayBuffer(4, { maxByteLength: 8 });
        let fixed = new Uint8Array(buffer, 0, 4);
        let tracking = new Uint8Array(buffer);

        Object.defineProperty(fixed, "0", { value: 1 });
        Object.defineProperty(tracking, "1", { value: 2 });

        buffer.resize(3);
        let fixedRejectsAfterShrink = throwsTypeError(function() {
            Object.defineProperty(fixed, "0", { value: 9 });
        });
        let trackingRejectsPastLiveLength = throwsTypeError(function() {
            Object.defineProperty(tracking, "3", { value: 9 });
        });
        Object.defineProperty(tracking, "2", { value: 7 });

        buffer.resize(6);
        Object.defineProperty(tracking, "4", { value: 8 });

        [
            fixedRejectsAfterShrink,
            trackingRejectsPastLiveLength,
            Array.from(tracking).join(",")
        ].join("|");
        "#,
    );

    assert_eq!(result, "true|true|1,2,7,0,8,0");
}

#[test]
fn script_core_typed_array_set_numeric_value_converts_and_handles_detach() {
    let result = compile_and_run_string(
        r#"
        function assignmentThrows(body) {
            try {
                body();
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        }

        function strictDetachedSetDoesNotThrow() {
            let sample = new Uint8Array([17]);
            sample.buffer.transfer(0);
            try {
                (function() {
                    "use strict";
                    sample[0] = 42;
                })();
                return sample[0] === undefined;
            } catch (error) {
                return "threw";
            }
        }

        function detachDuringNumberConversion() {
            let sample = new Uint8Array([17]);
            let result = Reflect.set(sample, "0", {
                valueOf: function() {
                    sample.buffer.transfer(0);
                    return 42;
                }
            });
            return result + ":" + (sample[0] === undefined);
        }

        function conversionRunsForDetachedBuffer() {
            let sample = new Uint8Array([17]);
            let count = 0;
            sample.buffer.transfer(0);
            sample[0] = {
                valueOf: function() {
                    count = count + 1;
                    return 42;
                }
            };
            return count;
        }

        let numberSample = new Uint8Array([0]);
        let bigintSample = new BigInt64Array([0n]);

        [
            assignmentThrows(function() { numberSample[0] = 1n; }),
            assignmentThrows(function() { bigintSample[0] = 1; }),
            Reflect.set(numberSample, "0", 257),
            numberSample[0] === 1,
            Reflect.set(bigintSample, "0", 2n),
            bigintSample[0] === 2n,
            strictDetachedSetDoesNotThrow(),
            detachDuringNumberConversion(),
            conversionRunsForDetachedBuffer()
        ].join(":");
        "#,
    );

    assert_eq!(result, "true:true:true:true:true:true:true:true:true:1");
}

#[test]
fn script_core_typed_array_buffer_constructor_observes_new_target_prototype_first() {
    let result = compile_and_run_string(
        r#"
        class ExpectedError extends Error {}
        let poisonedOffset = {
            valueOf() {
                throw new Error("offset");
            }
        };
        let NewTarget = Object.defineProperty(function(){}.bind(null), "prototype", {
            get() {
                throw new ExpectedError();
            }
        });

        try {
            Reflect.construct(Int32Array, [new ArrayBuffer(8), poisonedOffset, 0], NewTarget);
            "no throw";
        } catch (error) {
            error instanceof ExpectedError ? "prototype" : error.message;
        }
        "#,
    );

    assert_eq!(result, "prototype");
}

#[test]
fn script_core_typed_array_length_symbol_throws_before_new_target_prototype_lookup() {
    let result = compile_and_run_string(
        r#"
        class ExpectedError extends Error {}
        let NewTarget = Object.defineProperty(function(){}.bind(null), "prototype", {
            get() {
                throw new ExpectedError();
            }
        });

        try {
            Reflect.construct(Int32Array, [Symbol()], NewTarget);
            "no throw";
        } catch (error) {
            error instanceof TypeError ? "type" :
                error instanceof ExpectedError ? "prototype" :
                error.constructor.name;
        }
        "#,
    );

    assert_eq!(result, "type");
}

#[test]
fn script_core_typed_array_from_constructs_before_reading_array_like_elements() {
    let result = compile_and_run_string(
        r#"
        let log = "";
        let object;
        function C(...args) {
            log += "C";
            object = new Uint8Array(...args);
            return object;
        }
        let source = {
            get length() {
                log += "l";
                return 1;
            },
            get 0() {
                log += "0";
                return 7;
            }
        };

        try {
            Uint8Array.from.call(C, source, value => {
                log += "m";
                throw "stop";
            });
        } catch (error) {}

        log + ":" + (object instanceof Uint8Array);
        "#,
    );

    assert_eq!(result, "lC0m:true");
}

#[test]
fn script_core_typed_array_of_rejects_generator_constructors_catchably() {
    let result = compile_and_run_string(
        r#"
        try {
            Uint8Array.of.call(function*(length) { return length; }, "a");
            "no throw";
        } catch (error) {
            String(error instanceof TypeError);
        }
        "#,
    );

    assert_eq!(result, "true");
}

#[test]
fn script_core_typed_array_iterators_accept_own_properties() {
    let result = compile_and_run(
        r#"
        let array = new Int8Array(3);
        Object.defineProperty(array, "length", { value: 0 });
        let iterator = array[Symbol.iterator]();
        iterator.next = Array.prototype[Symbol.iterator]().next;
        let count = 0;
        while (!iterator.next().done) {
            count += 1;
        }
        count;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_typed_array_delete_detached_numeric_indices_returns_true() {
    let result = compile_and_run_string(
        r#"
        let sample = new Uint8Array(1);
        sample.buffer.transfer(0);

        [
            delete sample[0],
            delete sample["1.1"],
            delete sample["-0"],
            delete sample["-1"],
            delete sample["1"]
        ].join(":");
        "#,
    );

    assert_eq!(result, "true:true:true:true:true");
}

#[test]
fn script_core_typed_array_for_in_skips_detached_indices() {
    let result = compile_and_run_string(
        r"
        function testWithTypedArrayConstructors(f) {
            let constructors = [
                Float64Array,
                Float32Array,
                Int32Array,
                Int16Array,
                Int8Array,
                Uint32Array,
                Uint16Array,
                Uint8Array,
                Uint8ClampedArray
            ];
            for (let i = 0; i < constructors.length; i++) {
                f(constructors[i]);
            }
        }

        function testWithBigIntTypedArrayConstructors(f) {
            f(BigInt64Array);
            f(BigUint64Array);
        }

        function probe(TA) {
            let sample = new TA(3);
            sample.buffer.transfer(0);
            let count = 0;
            for (var key in sample) {
                count++;
            }
            return count;
        }

        let total = 0;
        testWithTypedArrayConstructors(function(TA) {
            total = total + probe(TA);
        });
        testWithBigIntTypedArrayConstructors(function(TA) {
            total = total + probe(TA);
        });
        String(total);
        ",
    );

    assert_eq!(result, "0");
}

#[test]
fn script_core_typed_array_own_keys_track_fixed_resizable_bounds() {
    let result = compile_and_run_string(
        r#"
        let bpe = Uint8Array.BYTES_PER_ELEMENT;
        let buffer = new ArrayBuffer(bpe * 4, { maxByteLength: bpe * 5 });
        let sample = new Uint8Array(buffer, bpe, 2);

        let initial = Reflect.ownKeys(sample).join(",");
        buffer.resize(bpe * 5);
        let afterGrow = Reflect.ownKeys(sample).join(",");
        buffer.resize(bpe * 3);
        let afterInBoundsShrink = Reflect.ownKeys(sample).join(",");
        buffer.resize(bpe * 2);
        let afterOutOfBoundsShrink = Reflect.ownKeys(sample).join(",");

        [
            initial,
            afterGrow,
            afterInBoundsShrink,
            afterOutOfBoundsShrink
        ].join("|");
        "#,
    );

    assert_eq!(result, "0,1|0,1|0,1|");
}

#[test]
fn script_core_typed_array_methods_use_live_resizable_lengths() {
    let result = compile_and_run_string(
        r#"
        function fixedAtThrows() {
            let buffer = new ArrayBuffer(4, { maxByteLength: 8 });
            let sample = new Uint8Array(buffer, 0, 4);
            buffer.resize(3);
            try {
                sample.at(0);
                return "missing";
            } catch (error) {
                return error instanceof TypeError;
            }
        }

        function trackingAtReadsLiveLength() {
            let buffer = new ArrayBuffer(4, { maxByteLength: 8 });
            let sample = new Uint8Array(buffer);
            sample[0] = 1;
            sample[1] = 2;
            sample[2] = 3;
            sample[3] = 4;
            buffer.resize(3);
            return sample.at(-1);
        }

        function atUsesIndexedGetAfterCoercion() {
            let buffer = new ArrayBuffer(4, { maxByteLength: 8 });
            let sample = new Uint8Array(buffer, 0, 4);
            let index = {
                valueOf: function() {
                    buffer.resize(2);
                    return 0;
                }
            };
            return sample.at(index);
        }

        function joinUsesIndexedGetAfterCoercion() {
            let buffer = new ArrayBuffer(4, { maxByteLength: 8 });
            let sample = new Uint8Array(buffer);
            sample[0] = 1;
            sample[1] = 2;
            sample[2] = 3;
            sample[3] = 4;
            let separator = {
                toString: function() {
                    buffer.resize(2);
                    return ".";
                }
            };
            return sample.join(separator);
        }

        function includesUndefinedAfterCoercionShrink() {
            let buffer = new ArrayBuffer(1, { maxByteLength: 1 });
            let sample = new Uint8Array(buffer);
            let index = {
                valueOf: function() {
                    buffer.resize(0);
                    return 0;
                }
            };
            return sample.includes(undefined, index);
        }

        [
            fixedAtThrows(),
            trackingAtReadsLiveLength(),
            atUsesIndexedGetAfterCoercion() === undefined,
            joinUsesIndexedGetAfterCoercion(),
            includesUndefinedAfterCoercionShrink()
        ].join(":");
        "#,
    );

    assert_eq!(result, "true:3:true:1.2..:true");
}

#[test]
fn script_core_typed_array_mutations_recheck_resizable_bounds() {
    let result = compile_and_run_string(
        r#"
        function fillFixedThrowsAfterValueShrink() {
            let buffer = new ArrayBuffer(4, { maxByteLength: 8 });
            let sample = new Uint8Array(buffer, 0, 4);
            let value = {
                valueOf: function() {
                    buffer.resize(2);
                    return 7;
                }
            };
            try {
                sample.fill(value, 0, 1);
                return "missing";
            } catch (error) {
                return error instanceof TypeError;
            }
        }

        function copyWithinTrackingTruncatesAfterShrink() {
            let buffer = new ArrayBuffer(4, { maxByteLength: 8 });
            let sample = new Uint8Array(buffer);
            sample[0] = 0;
            sample[1] = 1;
            sample[2] = 2;
            sample[3] = 3;
            let target = {
                valueOf: function() {
                    buffer.resize(3);
                    return 2;
                }
            };
            try {
                sample.copyWithin(target, 0);
                return Array.from(sample).join(",");
            } catch (error) {
                return "threw";
            }
        }

        function withAllowsIndexMadeValidByValueCoercion() {
            let buffer = new ArrayBuffer(2, { maxByteLength: 5 });
            let sample = new Int8Array(buffer);
            sample[0] = 11;
            sample[1] = 22;
            let value = {
                valueOf: function() {
                    buffer.resize(5);
                    return 123;
                }
            };
            let result = sample.with(4, value);
            return result.length + "," + result[0] + "," + result[1] + "," + sample.length;
        }

        function withRejectsIndexMadeInvalidByValueCoercion() {
            let buffer = new ArrayBuffer(4, { maxByteLength: 4 });
            let sample = new Uint8Array(buffer);
            let value = {
                valueOf: function() {
                    buffer.resize(1);
                    return 123;
                }
            };
            try {
                sample.with(-1, value);
                return "missing";
            } catch (error) {
                return error instanceof RangeError;
            }
        }

        [
            fillFixedThrowsAfterValueShrink(),
            copyWithinTrackingTruncatesAfterShrink(),
            withAllowsIndexMadeValidByValueCoercion(),
            withRejectsIndexMadeInvalidByValueCoercion()
        ].join("|");
        "#,
    );

    assert_eq!(result, "true|0,1,0|2,11,22,5|true");
}

#[test]
fn script_core_typed_array_species_create_checks_live_result_length() {
    let result = compile_and_run_string(
        r#"
        function probe(method) {
            let source = new Uint8Array([1, 2]);
            let buffer = new ArrayBuffer(2, { maxByteLength: 2 });
            let result = new Uint8Array(buffer);
            buffer.resize(0);
            source.constructor = {
                [Symbol.species]: function() {
                    return result;
                }
            };
            try {
                source[method](function() { return 1; });
                return "missing";
            } catch (error) {
                return error.constructor === TypeError;
            }
        }

        [probe("map"), probe("filter")].join(":");
        "#,
    );

    assert_eq!(result, "true:true");
}

#[test]
fn script_core_typed_array_length_tracking_allows_unaligned_resizable_byte_length() {
    let result = compile_and_run_string(
        r#"
        let buffer = new ArrayBuffer(10, { maxByteLength: 20 });
        let sample = new Int32Array(buffer);

        let initial = sample.length;
        buffer.resize(7);
        let afterShrink = sample.length;
        buffer.resize(15);
        let afterGrow = sample.length;

        [initial, afterShrink, afterGrow].join(",");
        "#,
    );

    assert_eq!(result, "2,1,3");
}

#[test]
fn script_core_typed_array_from_of_use_set_for_resized_results() {
    let result = compile_and_run_string(
        r#"
        let fromBuffer = new ArrayBuffer(3, { maxByteLength: 5 });
        let fromTarget = new Int8Array(fromBuffer);
        let fromResult = Int32Array.from.call(
            function() { return fromTarget; },
            [0, 1, 2],
            function(value) {
                if (value === 1) {
                    fromBuffer.resize(1);
                }
                return value + 10;
            }
        );

        let ofBuffer = new ArrayBuffer(3, { maxByteLength: 4 });
        let ofTarget = new Int8Array(ofBuffer);
        let one = {
            valueOf() {
                ofBuffer.resize(0);
                return 1;
            }
        };
        let two = {
            valueOf() {
                ofBuffer.resize(4);
                return 2;
            }
        };
        let ofResult = Int8Array.of.call(function() { return ofTarget; }, one, two, 3);

        [
            fromResult === fromTarget,
            fromTarget.length,
            fromTarget[0],
            ofResult === ofTarget,
            ofTarget.length,
            ofTarget[0],
            ofTarget[1],
            ofTarget[2],
            ofTarget[3]
        ].join("|");
        "#,
    );

    assert_eq!(result, "true|1|10|true|4|0|2|3|0");
}

#[test]
fn script_core_typed_array_set_uses_live_bounds_and_set_element() {
    let result = compile_and_run_string(
        r#"
        let rab = new ArrayBuffer(5, { maxByteLength: 10 });
        let target = new Int8Array(rab);
        let log = [];
        let shrinkNumber = 0;
        let growNumber = 0;
        let shrink = {
            valueOf() {
                log.push("shrink");
                rab.resize(rab.byteLength - 1);
                return ++shrinkNumber;
            }
        };
        let grow = {
            valueOf() {
                log.push("grow");
                rab.resize(rab.byteLength + 1);
                return --growNumber;
            }
        };

        target.set({
            get length() { return 5; },
            0: shrink,
            1: shrink,
            2: shrink,
            3: grow,
            4: grow
        });

        let targetOobBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
        let targetOob = new Int8Array(targetOobBuffer, 0, 4);
        targetOobBuffer.resize(3);
        let touched = false;
        let targetError = false;
        try {
            targetOob.set(new Proxy({}, {
                get() {
                    touched = true;
                    return 1;
                }
            }));
        } catch (error) {
            targetError = error.constructor === TypeError;
        }

        let sourceOobBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
        let sourceOob = new Int8Array(sourceOobBuffer, 0, 4);
        sourceOobBuffer.resize(3);
        let sourceError = false;
        try {
            new Int8Array(new ArrayBuffer(4)).set(sourceOob);
        } catch (error) {
            sourceError = error.constructor === TypeError;
        }

        let offsetBuffer = new ArrayBuffer(2, { maxByteLength: 2 });
        let offsetTarget = new Int8Array(offsetBuffer, 0, 2);
        let offsetCalled = false;
        let offsetError = false;
        try {
            offsetTarget.set([1], {
                valueOf() {
                    offsetCalled = true;
                    offsetBuffer.resize(1);
                    return 0;
                }
            });
        } catch (error) {
            offsetError = error.constructor === TypeError;
        }

        [
            log.join(","),
            Array.from(target).join(","),
            targetError,
            touched,
            sourceError,
            offsetCalled,
            offsetError
        ].join("|");
        "#,
    );

    assert_eq!(
        result,
        "shrink,shrink,shrink,grow,grow|1,2,0,0|true|false|true|true|true"
    );
}

#[test]
fn script_core_typed_array_slice_rechecks_resizable_source_bounds() {
    let result = compile_and_run_string(
        r#"
        let fixedBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
        let fixed = new Int8Array(fixedBuffer, 0, 4);
        let fixedError = false;
        try {
            fixed.slice({
                valueOf() {
                    fixedBuffer.resize(2);
                    return 0;
                }
            });
        } catch (error) {
            fixedError = error.constructor === TypeError;
        }

        let trackingBuffer = new ArrayBuffer(32, { maxByteLength: 32 });
        let tracking = new BigInt64Array(trackingBuffer);
        tracking.set([1n, 2n, 3n, 4n]);
        let sliced = tracking.slice({
            valueOf() {
                trackingBuffer.resize(16);
                return 0;
            }
        });

        [
            fixedError,
            sliced.length,
            Array.from(sliced).map(Number).join(",")
        ].join("|");
        "#,
    );

    assert_eq!(result, "true|4|1,2,0,0");
}

#[test]
fn script_core_typed_array_slice_preserves_same_kind_float_nan_payload_bits() {
    let result = compile_and_run_string(
        r#"
        let f32 = new Float32Array(1);
        let f32Bits = new Int32Array(f32.buffer);
        f32Bits[0] = 0x7F800001 | 0;
        let sliced32 = f32.slice(0);
        let sliced32Bits = new Int32Array(sliced32.buffer)[0];

        let littleEndian = new Uint8Array(new Uint16Array([1]).buffer)[0] !== 0;
        let f64 = new Float64Array(1);
        let f64Bits = new Int32Array(f64.buffer);
        f64Bits[littleEndian ? 0 : 1] = 0x00000001 | 0;
        f64Bits[littleEndian ? 1 : 0] = 0x7FF00000 | 0;
        let sliced64Bits = new Int32Array(f64.slice(0).buffer);

        [
            sliced32Bits,
            sliced64Bits[littleEndian ? 0 : 1],
            sliced64Bits[littleEndian ? 1 : 0]
        ].join(",");
        "#,
    );

    assert_eq!(result, "2139095041,1,2146435072");
}

#[test]
fn script_core_float16_array_uses_half_precision_storage() {
    let result = compile_and_run_string(
        r#"
        let sample = new Float16Array([-0, 0, 0.5, -0.5, NaN, Infinity, -Infinity]);
        let raw = new Int16Array(new Float16Array([1]).buffer)[0];
        let sorted = new Float16Array([NaN, 123, -456, 0]);
        sorted.sort();

        [
            typeof Float16Array,
            Float16Array.BYTES_PER_ELEMENT,
            Float16Array.prototype.BYTES_PER_ELEMENT,
            sample.toString(),
            raw,
            sorted[0],
            sorted[1],
            sorted[2],
            sorted[3]
        ].join("|");
        "#,
    );

    assert_eq!(
        result,
        "function|2|2|0,0,0.5,-0.5,NaN,Infinity,-Infinity|15360|-456|0|123|NaN"
    );
}

#[test]
fn script_core_typed_array_to_locale_string_omits_element_arguments_without_intl() {
    let result = compile_and_run_string(
        r#"
        let original = Number.prototype.toLocaleString;
        let calls = [];
        Number.prototype.toLocaleString = function() {
            calls.push(arguments.length);
            return "x";
        };

        let joined = new Uint8Array(2).toLocaleString({}, {});
        Number.prototype.toLocaleString = original;
        joined + "|" + calls.join(",");
        "#,
    );

    assert_eq!(result, "x,x|0,0");
}

#[test]
fn script_core_typed_array_subarray_uses_live_length_and_auto_length_species_args() {
    let result = compile_and_run_string(
        r#"
        let fixedBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
        let fixed = new Int8Array(fixedBuffer, 0, 4);
        fixedBuffer.resize(2);
        let fixedSubarray = fixed.subarray(0);

        let trackingBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
        let tracking = new Int8Array(trackingBuffer);
        let speciesArgCount = 0;
        tracking.constructor = {
            [Symbol.species]: function(buffer, offset, length) {
                speciesArgCount = arguments.length;
                return new Int8Array(buffer, offset, length);
            }
        };
        let trackingSubarray = tracking.subarray(1);
        trackingBuffer.resize(6);

        [
            fixedSubarray.length,
            speciesArgCount,
            trackingSubarray.byteOffset,
            trackingSubarray.length
        ].join("|");
        "#,
    );

    assert_eq!(result, "0|2|1|5");
}

#[test]
fn script_core_data_view_tracks_resizable_array_buffer_bounds() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let rab = new ArrayBuffer(4, { maxByteLength: 8 });
        let tracking = new DataView(rab, 1);
        let fixed = new DataView(rab, 0, 4);

        if (tracking.byteLength === 3 && tracking.byteOffset === 1) score += 1;
        rab.resize(6);
        if (tracking.byteLength === 5 && fixed.byteLength === 4) score += 2;
        rab.resize(2);
        if (tracking.byteLength === 1 && tracking.getUint8(0) === 0) score += 4;

        let fixedThrows = false;
        try {
            fixed.getUint8(0);
        } catch (error) {
            fixedThrows = error instanceof TypeError;
        }
        if (fixedThrows) score += 8;

        rab.resize(1);
        if (tracking.byteLength === 0 && tracking.byteOffset === 1) score += 16;

        let trackingThrows = false;
        rab.resize(0);
        try {
            tracking.byteLength;
        } catch (error) {
            trackingThrows = error instanceof TypeError;
        }
        if (trackingThrows) score += 32;

        let buffer = new ArrayBuffer(3, { maxByteLength: 3 });
        let newTarget = function() {}.bind(null);
        Object.defineProperty(newTarget, "prototype", {
            get: function() {
                buffer.resize(2);
                return DataView.prototype;
            }
        });
        let constructed = Reflect.construct(DataView, [buffer, 2], newTarget);
        if (constructed.byteLength === 0) score += 64;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn script_core_array_predicate_helpers_read_length_before_callback_validation() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let everyLength = false;
        let everyLoop = false;
        let everyObject = {};
        Object.defineProperty(everyObject, "length", {
            get: function() {
                everyLength = true;
                return 1;
            }
        });
        Object.defineProperty(everyObject, "0", {
            get: function() {
                everyLoop = true;
                return 1;
            }
        });
        try {
            Array.prototype.every.call(everyObject);
        } catch (error) {
            score += error.constructor === TypeError ? 1 : 0;
        }
        score += everyLength ? 2 : 0;
        score += everyLoop ? 0 : 4;

        let someLength = false;
        let someLoop = false;
        let someObject = {};
        Object.defineProperty(someObject, "length", {
            get: function() {
                someLength = true;
                return 1;
            }
        });
        Object.defineProperty(someObject, "0", {
            get: function() {
                someLoop = true;
                return 1;
            }
        });
        try {
            Array.prototype.some.call(someObject);
        } catch (error) {
            score += error.constructor === TypeError ? 8 : 0;
        }
        score += someLength ? 16 : 0;
        score += someLoop ? 0 : 32;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn script_core_array_reduce_helpers_are_generic_and_skip_holes() {
    let result = compile_and_run(
        r"
        let score = 0;
        let obj = new Date(0);
        obj.length = 3;
        obj[0] = 2;
        obj[2] = 5;
        score += Array.prototype.reduce.call(obj, function(acc, value, index, receiver) {
            return acc + value + (receiver instanceof Date ? index : 100);
        }, 1);
        score += Array.prototype.reduceRight.call(obj, function(acc, value, index, receiver) {
            return acc + value + (receiver instanceof Date ? index : 100);
        }, 1) * 10;
        try {
            Array.prototype.reduce.call({ length: 2 }, function(acc, value) {
                return acc + value;
            });
        } catch (error) {
            score += error.constructor === TypeError ? 100 : 0;
        }
        score;
        ",
    );

    assert_eq!(result, Value::from_smi(210));
}

#[test]
fn script_core_array_find_helpers_are_generic_and_visit_holes() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let values = [, 2];
        let sawHole = false;
        score += values.findIndex(function(value, index, receiver) {
            if (index === 0 && value === undefined && receiver === values) {
                sawHole = true;
            }
            return index === 1 && value === 2;
        }) === 1 ? 1 : 0;
        score += sawHole ? 2 : 0;

        let obj = new Date(0);
        obj.length = 3;
        obj[2] = 5;
        score += Array.prototype.find.call(obj, function(value, index, receiver) {
            return receiver instanceof Date && index === 2 && value === 5;
        }) === 5 ? 4 : 0;

        let sparse = { 0: "a", 2: "c", length: 3 };
        score += Array.prototype.findLast.call(sparse, function(value) {
            return value !== undefined;
        }) === "c" ? 8 : 0;
        score += Array.prototype.findLastIndex.call(true, function() {
            return true;
        }) === -1 ? 16 : 0;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_array_change_by_copy_helpers_are_generic_and_non_mutating() {
    let result = compile_and_run(
        r#"
        let score = 0;

        let values = [1, , 3];
        let reversed = values.toReversed();
        score += reversed !== values
            && reversed.length === 3
            && reversed[0] === 3
            && reversed[1] === undefined
            && reversed.hasOwnProperty("1")
            && reversed[2] === 1 ? 1 : 0;
        score += values.length === 3
            && values[0] === 1
            && !values.hasOwnProperty("1")
            && values[2] === 3 ? 2 : 0;

        let arrayLike = { 0: "b", 2: "a", length: 3 };
        let sorted = Array.prototype.toSorted.call(arrayLike);
        score += sorted.join(":") === "a:b:" ? 4 : 0;

        let replaced = Array.prototype.with.call(arrayLike, -1, "z");
        score += replaced[0] === "b"
            && replaced[1] === undefined
            && replaced.hasOwnProperty("1")
            && replaced[2] === "z" ? 8 : 0;

        let spliced = Array.prototype.toSpliced.call(
            { 0: "a", 1: "b", 2: "c", length: 3 },
            1,
            1,
            "x",
            "y"
        );
        score += spliced.join("") === "axyc" && spliced.length === 4 ? 16 : 0;

        try {
            [1, 2].with(2, 9);
        } catch (error) {
            score += error.constructor === RangeError ? 32 : 0;
        }

        try {
            Array.prototype.toSpliced.call({ length: 9007199254740991 }, 0, 0, 1);
        } catch (error) {
            score += error.constructor === TypeError ? 64 : 0;
        }

        score += Array.prototype[Symbol.unscopables].toReversed === true
            && Array.prototype[Symbol.unscopables].toSorted === true
            && Array.prototype[Symbol.unscopables].toSpliced === true
            && !Object.prototype.hasOwnProperty.call(
                Array.prototype[Symbol.unscopables],
                "with"
            ) ? 128 : 0;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(255));
}

#[test]
fn script_core_array_callback_helpers_read_length_before_callback_validation() {
    let result = compile_and_run(
        r#"
        function probe(method) {
            let score = 0;
            let lengthRead = false;
            let elementRead = false;
            let receiver = {};
            Object.defineProperty(receiver, "length", {
                get: function() {
                    lengthRead = true;
                    return 1;
                }
            });
            Object.defineProperty(receiver, "0", {
                get: function() {
                    elementRead = true;
                    return 1;
                }
            });
            try {
                Array.prototype[method].call(receiver);
            } catch (error) {
                score += error.constructor === TypeError ? 1 : 0;
            }
            score += lengthRead ? 2 : 0;
            score += elementRead ? 0 : 4;
            return score;
        }

        probe("forEach") + probe("map") * 10 + probe("filter") * 100;
        "#,
    );

    assert_eq!(result, Value::from_smi(777));
}

#[test]
fn script_core_array_callback_copy_helpers_define_own_result_properties() {
    let result = compile_and_run(
        r#"
        let score = 0;
        Object.defineProperty(Array.prototype, "1", {
            get: function() {
                return "prototype";
            },
            configurable: true
        });

        try {
            let mapped = [1, 2].map(function(value) {
                return value * 2;
            });
            score += mapped.length === 2
                && mapped.hasOwnProperty("1")
                && mapped[1] === 4 ? 1 : 0;

            let filtered = [1, 2].filter(function() {
                return true;
            });
            score += filtered.length === 2
                && filtered.hasOwnProperty("1")
                && filtered[1] === 2 ? 2 : 0;
        } finally {
            delete Array.prototype[1];
        }

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_array_spread_preserves_mapped_arrow_results() {
    let result = compile_and_run_string(
        r#"
        let negated = [1, 2].map(value => -value);
        let adjusted = [1, 2].map(value => value + 0.5);
        let spreadNegated = [...negated];
        let spreadAdjusted = [...adjusted];
        let combined = [
            ...[0, 1, 2],
            ...[0, 1, 2].map(value => -value),
        ];
        [
            String(negated[0]),
            String(negated[1]),
            String(adjusted[0]),
            String(adjusted[1]),
            String(spreadNegated[0]),
            String(spreadNegated[1]),
            String(spreadAdjusted[0]),
            String(spreadAdjusted[1]),
            String(spreadNegated.length),
            String(spreadAdjusted.length),
            String(combined[0]),
            String(combined[1]),
            String(combined[2]),
            String(combined[3]),
            String(combined[4]),
            String(combined[5]),
            String(combined.length),
        ].join("|");
        "#,
    );

    assert_eq!(result, "-1|-2|1.5|2.5|-1|-2|1.5|2.5|2|2|0|1|2|0|-1|-2|6");
}

#[test]
fn script_core_array_at_and_of_are_generic() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let object = { 0: "a", 2: "c", length: 3 };
        score += Array.prototype.at.call(object, -1) === "c" ? 1 : 0;
        score += Array.prototype.at.call(object, 1) === undefined ? 2 : 0;
        score += Array.prototype.at.call(true, 0) === undefined ? 4 : 0;

        let values = Array.of(1, 2, 3);
        score += values.length === 3
            && values[0] === 1
            && values[2] === 3 ? 8 : 0;

        function C(len) {
            this.lengthFromConstructor = len;
        }
        let custom = Array.of.call(C, "x", "y");
        score += custom instanceof C
            && custom.lengthFromConstructor === 2
            && custom.length === 2
            && custom[1] === "y" ? 16 : 0;

        score += Array.of.call({ notConstructor: true }, 5).join(":") === "5" ? 32 : 0;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn script_core_array_flat_and_flat_map_are_generic() {
    let result = compile_and_run(
        r#"
        let score = 0;
        let nested = [1, [2, , [3]]];
        let flattened = nested.flat(2);
        score += flattened.length === 3
            && flattened[0] === 1
            && flattened[1] === 2
            && flattened[2] === 3 ? 1 : 0;

        let arrayLike = { 0: [4, 5], 1: 6, length: 2 };
        let generic = Array.prototype.flat.call(arrayLike);
        score += generic.length === 3
            && generic[0] === 4
            && generic[1] === 5
            && generic[2] === 6 ? 2 : 0;

        let mapped = [1, , 3].flatMap(function(value, index, receiver) {
            return receiver.length === 3 ? [value, index] : [0];
        });
        score += mapped.join(":") === "1:0:3:2" ? 4 : 0;

        let boolResult = Array.prototype.flatMap.call(true, function() {
            return [1];
        });
        score += boolResult.length === 0 ? 8 : 0;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

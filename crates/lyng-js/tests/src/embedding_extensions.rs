use std::sync::Arc;

use lyng_js_common::{AtomTable, SourceId};
use lyng_js_compiler::compile_script;
use lyng_js_env::{Agent, Runtime};
use lyng_js_gc::AllocationLifetime;
use lyng_js_gc::PrimitiveStringView;
use lyng_js_host::NoopHostHooks;
use lyng_js_ops::errors;
use lyng_js_parser::parse_script;
use lyng_js_sema::analyze_script;
use lyng_js_types::{EmbeddingFunctionId, PropertyKey, Value};
use lyng_js_vm::{
    EmbeddingFunctionContext, EmbeddingFunctionMetadata, EmbeddingInvocation,
    RealmExtensionInstallation, RealmExtensionProvider, SharedRealmExtensionProvider, Vm, VmError,
};

const EMBEDDING_EVAL_SCRIPT_RAW: u32 = 1;
const EMBEDDING_CREATE_REALM_RAW: u32 = 2;

fn embedding_eval_script_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(EMBEDDING_EVAL_SCRIPT_RAW)
        .expect("embedding function ids should stay non-zero")
}

fn embedding_create_realm_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(EMBEDDING_CREATE_REALM_RAW)
        .expect("embedding function ids should stay non-zero")
}

fn compile_unit(source: &str, atoms: &mut AtomTable) -> lyng_js_bytecode::CompiledScriptUnit {
    let parsed = parse_script(atoms, SourceId::new(0), source);
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors: {:?}",
        parsed.diagnostics.as_slice()
    );
    let sema = analyze_script(&parsed, atoms);
    assert!(
        !sema.diagnostics.has_errors(),
        "unexpected sema errors: {:?}",
        sema.diagnostics.as_slice()
    );
    compile_script(&parsed, &sema, atoms).expect("script should lower")
}

fn decode_string(view: PrimitiveStringView<'_>) -> String {
    if let Some(bytes) = view.latin1_bytes() {
        return bytes.iter().map(|byte| char::from(*byte)).collect();
    }
    let bytes = view
        .utf16_bytes()
        .expect("string view must be Latin1 or UTF-16");
    let mut units = Vec::with_capacity(view.code_unit_len() as usize);
    for chunk in bytes.chunks_exact(2) {
        units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    String::from_utf16_lossy(&units)
}

fn embedding_property_key(agent: &mut Agent, text: &str) -> PropertyKey {
    PropertyKey::from_atom(agent.atoms_mut().intern_collectible(text))
}

#[derive(Clone, Default)]
struct DemoExtensionProvider;

impl RealmExtensionProvider for DemoExtensionProvider {
    fn embedding_function_metadata(
        &self,
        entry: EmbeddingFunctionId,
    ) -> Option<EmbeddingFunctionMetadata> {
        if entry == embedding_eval_script_entry() {
            return Some(EmbeddingFunctionMetadata::new(
                "evalScript",
                1,
                false,
                false,
            ));
        }
        if entry == embedding_create_realm_entry() {
            return Some(EmbeddingFunctionMetadata::new(
                "createRealm",
                0,
                false,
                false,
            ));
        }
        None
    }

    fn install_realm_extensions(
        &self,
        installation: &mut RealmExtensionInstallation<'_>,
    ) -> Result<(), VmError> {
        let realm = installation.realm();
        let object_prototype = installation
            .agent()
            .realm(realm)
            .and_then(|realm| realm.intrinsics().object_prototype())
            .ok_or(VmError::MissingRootShape(realm))?;
        let embedding = installation.allocate_ordinary_object(Some(object_prototype))?;

        let embedding_key = embedding_property_key(installation.agent(), "embedding");
        let marker_key = embedding_property_key(installation.agent(), "embeddingMarker");
        let eval_script_key = embedding_property_key(installation.agent(), "evalScript");
        let create_realm_key = embedding_property_key(installation.agent(), "createRealm");
        let marker_value = Value::from_string_ref(installation.agent().alloc_runtime_string(
            "installed",
            None,
            AllocationLifetime::Default,
        ));

        installation.define_data_property(
            installation.global_object(),
            embedding_key,
            Value::from_object_ref(embedding),
            true,
            false,
            true,
        )?;
        installation.define_data_property(
            installation.global_object(),
            marker_key,
            marker_value,
            true,
            false,
            true,
        )?;
        let _ = installation.define_function_property(
            embedding,
            eval_script_key,
            embedding_eval_script_entry(),
            true,
            false,
            true,
        )?;
        let _ = installation.define_function_property(
            embedding,
            create_realm_key,
            embedding_create_realm_entry(),
            true,
            false,
            true,
        )?;
        Ok(())
    }

    fn call_embedding_function(
        &self,
        context: &mut EmbeddingFunctionContext<'_>,
        entry: EmbeddingFunctionId,
        invocation: EmbeddingInvocation<'_>,
    ) -> Result<Value, VmError> {
        if entry == embedding_eval_script_entry() {
            let source = invocation
                .arguments()
                .first()
                .copied()
                .unwrap_or(Value::undefined());
            let source_text = context.value_to_string_text(source)?;
            return context.evaluate_script_in_realm(context.function_realm(), &source_text);
        }
        if entry == embedding_create_realm_entry() {
            return Ok(Value::from_object_ref(
                context.create_embedding_realm()?.global_object(),
            ));
        }
        Err(VmError::Abrupt(errors::throw_type_error(context.agent())))
    }
}

fn compile_and_run(source: &str, provider: Option<&SharedRealmExtensionProvider>) -> Value {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(source, &mut atoms);
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    vm.evaluate_script_with_host_referrer_and_extensions(
        agent,
        realm,
        &unit,
        None,
        &NoopHostHooks,
        provider,
    )
    .expect("script should execute")
}

fn compile_and_run_string(source: &str, provider: Option<&SharedRealmExtensionProvider>) -> String {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(source, &mut atoms);
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let value = vm
        .evaluate_script_with_host_referrer_and_extensions(
            agent,
            realm,
            &unit,
            None,
            &NoopHostHooks,
            provider,
        )
        .expect("script should execute");
    let string = value
        .as_string_ref()
        .expect("script should return a string value");
    decode_string(
        agent
            .heap()
            .view()
            .string_view(string)
            .expect("string should exist in the heap"),
    )
}

#[test]
fn spec_only_realms_do_not_install_embedding_extensions() {
    let result = compile_and_run_string("typeof embedding;", None);

    assert_eq!(result, "undefined");
}

#[test]
fn embedding_extensions_install_globals_and_dispatch_native_functions() {
    let provider: SharedRealmExtensionProvider = Arc::new(DemoExtensionProvider);
    let result = compile_and_run(
        r#"
        let value = embedding.evalScript("globalThis.embeddingValue = 41; embeddingValue;");
        (typeof embedding === "object" ? 1 : 0)
            + (embeddingMarker === "installed" ? 2 : 0)
            + (value === 41 ? 4 : 0)
            + (embeddingValue === 41 ? 8 : 0);
        "#,
        Some(&provider),
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn embedding_extensions_propagate_to_child_realms() {
    let provider: SharedRealmExtensionProvider = Arc::new(DemoExtensionProvider);
    let result = compile_and_run(
        r#"
        let childGlobal = embedding.createRealm();
        (childGlobal !== globalThis ? 1 : 0)
            + (typeof childGlobal.embedding === "object" ? 2 : 0)
            + (childGlobal.embeddingMarker === "installed" ? 4 : 0);
        "#,
        Some(&provider),
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn array_constructor_uses_new_target_realm_default_prototype() {
    let provider: SharedRealmExtensionProvider = Arc::new(DemoExtensionProvider);
    let result = compile_and_run(
        r#"
        let score = 0;
        let childGlobal = embedding.createRealm();
        let newTarget = new childGlobal.Function();
        newTarget.prototype = undefined;

        let empty = Reflect.construct(Array, [], newTarget);
        score += Object.getPrototypeOf(empty) === childGlobal.Array.prototype ? 1 : 0;

        let sized = Reflect.construct(Array, [1], newTarget);
        score += Object.getPrototypeOf(sized) === childGlobal.Array.prototype ? 2 : 0;

        let items = Reflect.construct(Array, ["a", "b"], newTarget);
        score += Object.getPrototypeOf(items) === childGlobal.Array.prototype ? 4 : 0;

        score;
        "#,
        Some(&provider),
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn typed_array_constructor_uses_new_target_realm_default_prototype() {
    let provider: SharedRealmExtensionProvider = Arc::new(DemoExtensionProvider);
    let result = compile_and_run_string(
        r#"
        let childGlobal = embedding.createRealm();
        let newTarget = new childGlobal.Function();
        newTarget.prototype = undefined;
        let names = [
            "Float64Array", "Float32Array", "Float16Array",
            "Int32Array", "Int16Array", "Int8Array",
            "Uint32Array", "Uint16Array", "Uint8Array", "Uint8ClampedArray",
            "BigInt64Array", "BigUint64Array"
        ];

        let matches = 0;
        for (let name of names) {
            let typedArray = Reflect.construct(globalThis[name], [], newTarget);
            if (Object.getPrototypeOf(typedArray) === childGlobal[name].prototype) {
                matches += 1;
            }
        }
        matches + ":" + names.length;
        "#,
        Some(&provider),
    );

    assert_eq!(result, "12:12");
}

#[test]
fn embedding_cross_realm_data_view_methods_accept_foreign_views_and_buffers() {
    let provider: SharedRealmExtensionProvider = Arc::new(DemoExtensionProvider);
    let result = compile_and_run_string(
        r#"
        let failures = "";
        function record(label, condition) {
            if (!condition) failures += label + ";";
        }
        function recordThrows(label, expectedConstructor, callback) {
            try {
                callback();
                failures += label + ":missing;";
            } catch (error) {
                if (!(error instanceof expectedConstructor)) {
                    failures += label + ":" + error.constructor.name + ";";
                }
            }
        }

        let childGlobal = embedding.createRealm();
        for (let constructorName of ["ArrayBuffer", "SharedArrayBuffer"]) {
        childGlobal.eval(`
            var data = [0, 1, 2, 3, 100, 101, 102, 103, 128, 129, 130, 131, 252, 253, 254, 255];
            var buffer = new ${constructorName}(data.length);
            new Uint8Array(buffer).set(data);
            var view = new DataView(buffer, 0, 16);
        `);

        record(constructorName + ":foreign-view-buffer", childGlobal.view.buffer.byteLength > 0);
        record(constructorName + ":foreign-view-method", childGlobal.view.getUint8(4) === 100);

        let inheritedForeignView = Object.create(childGlobal.view);
        recordThrows(constructorName + ":foreign-inherited-method", childGlobal.TypeError, () => {
            inheritedForeignView.getUint8(4);
        });
        recordThrows(constructorName + ":foreign-inherited-buffer", childGlobal.TypeError, () => {
            inheritedForeignView.buffer;
        });

        let localViewOnForeignBuffer = new DataView(childGlobal.buffer);
        record(constructorName + ":local-view-foreign-buffer", localViewOnForeignBuffer.getUint8(4) === 100);
        record(constructorName + ":local-view-prototype", Object.getPrototypeOf(localViewOnForeignBuffer) === DataView.prototype);

        let localBuffer = new Int8Array(3).buffer;
        let foreignViewOnLocalBuffer = new childGlobal.DataView(localBuffer);
        record(constructorName + ":foreign-constructor-local-buffer", foreignViewOnLocalBuffer.byteLength === 3);
        record(constructorName + ":foreign-constructor-prototype", Object.getPrototypeOf(foreignViewOnLocalBuffer) === childGlobal.DataView.prototype);
        }

        failures || "ok";
        "#,
        Some(&provider),
    );

    assert_eq!(result, "ok");
}

#[test]
fn array_species_create_handles_cross_realm_constructors() {
    let provider: SharedRealmExtensionProvider = Arc::new(DemoExtensionProvider);
    let result = compile_and_run_string(
        r#"
        let failures = "";
        function record(label, condition) {
            if (!condition) failures += label + ";";
        }
        function recordThrows(label, expectedConstructor, callback) {
            try {
                callback();
                failures += label + ":missing;";
            } catch (error) {
                if (!(error instanceof expectedConstructor)) {
                    failures += label + ":" + error.constructor.name + ";";
                }
            }
        }

        let expected = {
            concat: "get:concat,get:constructor,c-get:Symbol(Symbol.species),get:Symbol(Symbol.isConcatSpreadable),get:length,get:0,define:0:1:true:true:true,get:1,define:1:2:true:true:true,get:2,define:2:3:true:true:true,get:3,define:3:4:true:true:true,get:4,define:4:5:true:true:true,set:length:5,",
            filter: "get:filter,get:length,get:constructor,c-get:Symbol(Symbol.species),get:0,define:0:1:true:true:true,get:1,get:2,define:1:3:true:true:true,get:3,get:4,define:2:5:true:true:true,",
            map: "get:map,get:length,get:constructor,c-get:Symbol(Symbol.species),get:0,define:0:2:true:true:true,get:1,define:1:4:true:true:true,get:2,define:2:6:true:true:true,get:3,define:3:8:true:true:true,get:4,define:4:10:true:true:true,",
            slice: "get:slice,get:length,get:constructor,c-get:Symbol(Symbol.species),get:0,define:0:1:true:true:true,get:1,define:1:2:true:true:true,get:2,define:2:3:true:true:true,get:3,define:3:4:true:true:true,get:4,define:4:5:true:true:true,set:length:5,",
            splice: "get:splice,get:length,get:constructor,c-get:Symbol(Symbol.species),get:0,define:0:1:true:true:true,get:1,define:1:2:true:true:true,get:2,define:2:3:true:true:true,get:3,define:3:4:true:true:true,get:4,define:4:5:true:true:true,set:length:5,",
        };
        let expectedLength = {
            concat: 0,
            filter: 0,
            map: 5,
            slice: 5,
            splice: 5,
        };

        let childGlobal = embedding.createRealm();
        childGlobal.eval("function EvalFakeArray(n) { this.length = n; }");
        record("foreign-direct-eval-function-global", typeof childGlobal.EvalFakeArray === "function");
        childGlobal.embedding.evalScript(`
            function FakeArray(n) { this.length = n; }
            function FakeArrayWithSpecies(n) { this.length = n; }
            FakeArrayWithSpecies[Symbol.species] = FakeArrayWithSpecies;
            var a = [1, 2, 3, 4, 5];
        `);

        for (let name of ["concat", "filter", "map", "slice", "splice"]) {
            let args =
                name === "filter" ? [x => x % 2] :
                name === "map" ? [x => x * 2] :
                name === "splice" ? [0, 5] :
                [];

            let a = [1, 2, 3, 4, 5];
            a.constructor = { [Symbol.species]: childGlobal.FakeArray };
            let b = a[name](...args);
            record(name + ":foreign-species-function", b.constructor === childGlobal.FakeArray);

            a = [1, 2, 3, 4, 5];
            a.constructor = { [Symbol.species]: childGlobal.Array };
            b = a[name](...args);
            record(name + ":foreign-array-species", b.constructor === childGlobal.Array);

            a = [1, 2, 3, 4, 5];
            a.constructor = childGlobal.FakeArrayWithSpecies;
            b = a[name](...args);
            record(name + ":foreign-constructor-species", b.constructor === childGlobal.FakeArrayWithSpecies);

            b = Array.prototype[name].apply(childGlobal.a, args);
            record(name + ":current-method-foreign-default-array", b.constructor === Array);

            b = childGlobal.a[name](...args);
            record(name + ":foreign-method-foreign-default-array", b.constructor === childGlobal.Array);

            function FakeArray(n) { this.length = n; }
            function FakeArrayWithHook(n) {
                return new Proxy(new FakeArray(n), {
                    set(that, property, value) {
                        logs += "set:" + property + ":" + value + ",";
                        return true;
                    },
                    defineProperty(that, property, desc) {
                        logs += "define:" + property + ":" + desc.value + ":" + desc.configurable + ":" + desc.enumerable + ":" + desc.writable + ",";
                        return true;
                    },
                });
            }
            let logs = "";
            let ctorProxy = new Proxy({}, {
                get(that, property) {
                    logs += "c-get:" + property.toString() + ",";
                    return property == Symbol.species ? FakeArrayWithHook : undefined;
                },
            });
            a = new Proxy([1, 2, 3, 4, 5], {
                get(that, property) {
                    logs += "get:" + property.toString() + ",";
                    return property == "constructor" ? ctorProxy : that[property];
                },
            });
            b = a[name](...args);
            record(name + ":proxy-constructor", b.constructor === FakeArray);
            record(name + ":proxy-keys", Object.keys(b).sort().join(",") === "length");
            record(name + ":proxy-length", b.length === expectedLength[name]);
            record(name + ":proxy-log", logs === expected[name]);

            for (let species of [0, 1.1, true, false, "a", /a/, Symbol.iterator, [], {}]) {
                a = [1, 2, 3, 4, 5];
                a.constructor = { [Symbol.species]: species };
                recordThrows(name + ":invalid-species:" + typeof species, TypeError, () => a[name](...args));
            }

            for (let constructor of [null, 0, 1.1, true, false, "a", Symbol.iterator]) {
                a = [1, 2, 3, 4, 5];
                a.constructor = constructor;
                recordThrows(name + ":invalid-constructor:" + typeof constructor, TypeError, () => a[name](...args));
            }

            a = new Proxy({
                0: 1, 1: 2, 2: 3, 3: 4, 4: 5,
                length: 5,
                [name]: Array.prototype[name],
            }, {
                get(that, property) {
                    record(name + ":non-array-constructor-access", property !== "constructor");
                    return that[property];
                },
            });
            b = a[name](...args);
            record(name + ":non-array-default-result", b.constructor === Array);

            class SubArray extends Array {}
            a = new SubArray(1, 2, 3, 4, 5);
            b = a[name](...args);
            record(name + ":subclass-default-species", b.constructor === SubArray);

            class DateSpeciesArray extends Array {
                static get [Symbol.species]() {
                    return Date;
                }
            }
            a = new DateSpeciesArray(1, 2, 3, 4, 5);
            b = a[name](...args);
            record(name + ":subclass-custom-species", b.constructor === Date);
        }

        failures || "ok";
        "#,
        Some(&provider),
    );

    assert_eq!(result, "ok");
}

#[test]
fn array_copying_methods_throw_errors_from_their_function_realm() {
    let provider: SharedRealmExtensionProvider = Arc::new(DemoExtensionProvider);
    let result = compile_and_run_string(
        r#"
        let failures = "";
        function recordThrows(label, expectedConstructor, callback) {
            try {
                callback();
                failures += label + ":missing;";
            } catch (error) {
                if (!(error instanceof expectedConstructor)) {
                    failures += label + ":" + error.constructor.name + ";";
                }
            }
        }

        let childGlobal = embedding.createRealm();
        let arrayLike = {
            get "0"() {
                throw new Error("Get 0");
            },
            get "4294967295"() {
                throw new Error("Get 4294967295");
            },
            get "4294967296"() {
                throw new Error("Get 4294967296");
            },
            length: 2 ** 32,
        };

        let toSorted = childGlobal.Array.prototype.toSorted;
        let toSpliced = childGlobal.Array.prototype.toSpliced;
        let toReversed = childGlobal.Array.prototype.toReversed;
        let withMethod = childGlobal.Array.prototype.with;

        recordThrows("toSorted:bad-comparator", childGlobal.TypeError, () => toSorted.call([], 5));
        recordThrows("toSorted:null-this", childGlobal.TypeError, () => toSorted.call(null));
        recordThrows("toSpliced:too-long-type", childGlobal.TypeError, () => {
            let oldLen = arrayLike.length;
            arrayLike.length = 2 ** 53 - 1;
            try {
                toSpliced.call(arrayLike, 0, 0, 1);
            } finally {
                arrayLike.length = oldLen;
            }
        });

        recordThrows("toSorted:array-too-long", childGlobal.RangeError, () => toSorted.call(arrayLike));
        recordThrows("toReversed:array-too-long", childGlobal.RangeError, () => toReversed.call(arrayLike));
        recordThrows("toSpliced:array-too-long", childGlobal.RangeError, () => toSpliced.call(arrayLike, 0, 0));
        recordThrows("with:index-out-of-range", childGlobal.RangeError, () => withMethod.call([0, 1, 2], 3, 7));
        recordThrows("with:negative-index", childGlobal.RangeError, () => withMethod.call([0, 1, 2], -4, 7));
        recordThrows("with:array-too-long", childGlobal.RangeError, () => withMethod.call(arrayLike, 0, 0));

        failures || "ok";
        "#,
        Some(&provider),
    );

    assert_eq!(result, "ok");
}

#[test]
fn array_copying_methods_create_results_in_their_function_realm() {
    let provider: SharedRealmExtensionProvider = Arc::new(DemoExtensionProvider);
    let result = compile_and_run_string(
        r#"
        let failures = "";
        function record(label, condition) {
            if (!condition) failures += label + ";";
        }

        let childGlobal = embedding.createRealm();
        let methods = [
            ["with", childGlobal.Array.prototype.with.call([1, 2, 3], 1, 3)],
            ["toSpliced", childGlobal.Array.prototype.toSpliced.call([1, 2, 3], 0, 1, 4, 5)],
            ["toReversed", childGlobal.Array.prototype.toReversed.call([1, 2, 3])],
            ["toSorted", childGlobal.Array.prototype.toSorted.call([1, 2, 3], (left, right) => right > left)]
        ];

        for (let [name, value] of methods) {
            record(name + ":not-current-array", !(value instanceof Array));
            record(name + ":child-array", value instanceof childGlobal.Array);
            record(name + ":child-prototype", Object.getPrototypeOf(value) === childGlobal.Array.prototype);
        }

        failures || "ok";
        "#,
        Some(&provider),
    );

    assert_eq!(result, "ok");
}

#[test]
fn indirect_eval_uses_eval_functions_realm() {
    let provider: SharedRealmExtensionProvider = Arc::new(DemoExtensionProvider);
    let result = compile_and_run_string(
        r#"
        let childGlobal = embedding.createRealm();
        let otherEval = childGlobal.eval;
        otherEval("var x = 23;");
        typeof x + ":" + String(childGlobal.x);
        "#,
        Some(&provider),
    );

    assert_eq!(result, "undefined:23");
}

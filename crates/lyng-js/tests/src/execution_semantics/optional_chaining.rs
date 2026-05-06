use super::support::{compile_and_run, compile_and_run_string};
use lyng_js_types::Value;

#[test]
fn optional_chaining_short_circuits_continuations_and_preserves_this() {
    let result = compile_and_run_string(
        r#"
            let effects = 0;
            const missing = undefined;
            missing?.[++effects];
            missing?.prop.method(++effects).value;

            const object = {
                tag: "ok",
                method() {
                    return this.tag;
                }
            };

            String(effects) + ":" + object?.method();
        "#,
    );

    assert_eq!(result, "0:ok");
}

#[test]
fn derived_super_calls_accept_spread_arguments() {
    let result = compile_and_run(
        r#"
            class Base {
                constructor(first, second, third) {
                    this.value = first + second + third;
                }
            }

            class Derived extends Base {
                constructor(values) {
                    super(1, ...values);
                }
            }

            new Derived([2, 3]).value;
        "#,
    );

    assert_eq!(result, Value::from_smi(6));
}

#[test]
fn optional_calls_support_builtin_and_spread_arguments() {
    let result = compile_and_run(
        r#"
            function add(first, second, third) {
                third = third === undefined ? 0 : third;
                return first + second + third;
            }

            add?.(10, 20) + add?.(...[1, 2, 3]) + (String?.(42) === "42" ? 100 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(136));
}

#[test]
fn optional_chain_class_heritage_allows_runtime_null_superclass() {
    let result = compile_and_run_string(
        r#"
            const holder = { base: null };
            class Derived extends holder?.base {}
            String(Object.getPrototypeOf(Derived.prototype) === null);
        "#,
    );

    assert_eq!(result, "true");
}

#[test]
fn optional_chain_delete_deletes_present_properties_and_skips_nullish_keys() {
    let result = compile_and_run_string(
        r#"
            const object = { x: 1, y: 2 };
            const missing = null;
            let effects = 0;
            let tailEffects = 0;

            [
                delete object?.x,
                String(object.x),
                delete object?.["y"],
                String(object.y),
                delete missing?.[effects++],
                effects,
                delete undefined?.x[tailEffects++],
                tailEffects,
            ].join(":");
        "#,
    );

    assert_eq!(result, "true:undefined:true:undefined:true:0:true:0");
}

#[test]
fn optional_call_short_circuit_skips_following_call_continuations() {
    let result = compile_and_run_string(
        r#"
            const missing = { a: { b: undefined } };
            const returnsUndefined = { a: { b() { return undefined; } } };

            [
                String(missing.a.b?.()()()),
                String(missing.a.b?.()?.()()),
                String(returnsUndefined.a.b?.()?.()),
            ].join(":");
        "#,
    );

    assert_eq!(result, "undefined:undefined:undefined");
}

#[test]
fn delete_super_property_throws_reference_error() {
    let result = compile_and_run(
        r#"
            let status = 0;
            class Derived extends Object {
                constructor() {
                    super();
                    try {
                        delete super.value;
                        status = 1;
                    } catch (error) {
                        status = error.constructor === ReferenceError ? 2 : 3;
                    }
                }
            }

            new Derived();
            status;
        "#,
    );

    assert_eq!(result, Value::from_smi(2));
}

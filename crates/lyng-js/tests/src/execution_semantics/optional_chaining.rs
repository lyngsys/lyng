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

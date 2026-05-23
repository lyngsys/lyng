use super::support::*;

#[test]
fn explicit_derived_constructors_can_fall_through_after_super_call() {
    let unit = compile_test_unit(
        221,
        r"
            class Base {
                constructor(x) {
                    this.foobar = x;
                }
            }
            class Subclass extends Base {
                constructor(x) {
                    super(x);
                }
            }
            var instance = new Subclass(1);
            instance.foobar === 1 && Object.getPrototypeOf(instance) === Subclass.prototype;
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn explicit_derived_constructors_can_return_this_immediately_after_super_call() {
    let unit = compile_test_unit(
        222,
        r"
            class Base {
                constructor(x) {
                    this.foobar = x;
                }
            }
            class Subclass extends Base {
                constructor(x) {
                    super(x);
                    return this;
                }
            }
            new Subclass(1).foobar === 1;
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn explicit_derived_constructors_throw_for_non_undefined_primitive_returns() {
    let unit = compile_test_unit(
        223,
        r"
            class Base {
                constructor() {}
            }
            class Derived extends Base {
                constructor() {
                    super();
                    return 0;
                }
            }
            var status = 0;
            try {
                new Derived();
                status = 1;
            } catch (error) {
                status = error.constructor === TypeError ? 2 : 3;
            }
            status;
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(2));
}

#[test]
fn explicit_derived_constructors_preserve_object_return_overrides() {
    let unit = compile_test_unit(
        224,
        r"
            class Base {
                constructor() {
                    this.base = true;
                }
            }
            class Derived extends Base {
                constructor() {
                    super();
                    return { overridden: true };
                }
            }
            var result = new Derived();
            result.overridden === true
                && result.base === undefined
                && !(result instanceof Derived);
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn instanceof_accepts_class_constructors_as_rhs() {
    let unit = compile_test_unit(
        225,
        r"
            class Base {}
            let value = new Base();
            value instanceof Base;
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn class_heritage_functions_use_strict_arguments_objects() {
    let unit = compile_test_unit(
        237,
        r"
            var status = 0;
            var D = class extends function() {
                arguments.callee;
            } {};
            try {
                Object.getPrototypeOf(D).arguments;
            } catch (error) {
                if (error.constructor === TypeError) {
                    status += 1;
                }
            }
            try {
                new D;
            } catch (error) {
                if (error.constructor === TypeError) {
                    status += 2;
                }
            }
            status;
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn arrow_super_calls_in_finally_initialize_the_enclosing_derived_constructor() {
    let unit = compile_test_unit(
        226,
        r#"
            class Derived extends class {} {
                constructor() {
                    var callSuper = () => super();
                    try {
                        return;
                    } finally {
                        callSuper();
                    }
                }
            }
            typeof new Derived() === "object";
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn class_declaration_heritage_self_reference_throws_reference_error() {
    let unit = compile_test_unit(
        226,
        r"
            var status = 0;
            try {
                class x extends x {}
                status = 1;
            } catch (error) {
                status = error.constructor === ReferenceError ? 2 : 3;
            }
            status;
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(2));
}

#[test]
fn class_expression_heritage_self_reference_throws_reference_error() {
    let unit = compile_test_unit(
        227,
        r"
            var status = 0;
            try {
                (class x extends x {});
                status = 1;
            } catch (error) {
                status = error.constructor === ReferenceError ? 2 : 3;
            }
            status;
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(2));
}

#[test]
fn class_static_block_super_property_uses_parent_class_home_object() {
    let unit = compile_test_unit(
        228,
        r#"
            function Parent() {}
            Parent.test262 = "test262";
            var value = "";

            class C extends Parent {
                static {
                    value = super.test262;
                }
            }

            value;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = decode_string(
        &agent
            .heap()
            .view()
            .string_view(result.as_string_ref().expect("value should be a string"))
            .expect("string should be allocated"),
    );

    assert_eq!(text, "test262");
}

#[test]
fn evaluate_script_super_property_compound_assignment_uses_stable_reference() {
    let unit = compile_test_unit(
        231,
        r#"
            var log = [];

            class Base {
                get value() {
                    log.push("get");
                    return this._value;
                }

                set value(next) {
                    log.push("set:" + String(next));
                    this._value = next;
                }
            }

            class Derived extends Base {
                constructor() {
                    super();
                    this._value = 2;
                }

                add() {
                    return super.value += 3;
                }
            }

            var instance = new Derived();
            String(instance.add()) + ":" + String(instance._value) + ":" + log.join("|");
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let text = decode_string(
        &agent
            .heap()
            .view()
            .string_view(result.as_string_ref().expect("value should be a string"))
            .expect("string should be allocated"),
    );

    assert_eq!(text, "5:5:get|set:5");
}

#[test]
fn instance_field_arrow_functions_preserve_super_binding() {
    let unit = compile_test_unit(
        229,
        r#"
            class C {
                func = () => {
                    super.prop = "test262";
                };
            }

            let c = new C();
            c.func();
            c.prop === "test262";
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn static_field_arrow_functions_preserve_super_binding() {
    let unit = compile_test_unit(
        230,
        r#"
            class C {
                static staticFunc = () => {
                    super.staticProp = "static test262";
                };
            }

            C.staticFunc();
            C.staticProp === "static test262";
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn derived_constructors_reject_this_before_super_even_when_super_returns_an_object() {
    let unit = compile_test_unit(
        231,
        r#"
            class Base {
                constructor(a, b) {
                    return { prp: a + b };
                }
            }

            var ok = false;
            class Subclass extends Base {
                constructor(a, b) {
                    var before = 0;
                    try {
                        this.prp1 = 3;
                        before = 1;
                    } catch (error) {
                        before = error.constructor === ReferenceError ? 2 : 3;
                    }
                    super(a, b);
                    ok = before === 2
                        && this.prp === a + b
                        && this.prp1 === undefined
                        && !this.hasOwnProperty("prp1");
                }
            }

            var result = new Subclass(2, -1);
            ok
                && result.prp === 1
                && result.prp1 === undefined
                && !result.hasOwnProperty("prp1");
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn derived_constructors_throw_reference_error_on_second_super_after_evaluating_arguments() {
    let unit = compile_test_unit(
        232,
        r"
            class Base {
                constructor(a, b) {
                    this.prp = a + b;
                }
            }

            var ok = false;
            class Subclass extends Base {
                constructor() {
                    super(1, 2);
                    var called = false;
                    function tmp() {
                        called = true;
                        return 3;
                    }
                    var status = 0;
                    try {
                        super(tmp(), 4);
                        status = 1;
                    } catch (error) {
                        status = error.constructor === ReferenceError ? 2 : 3;
                    }
                    ok = status === 2 && called === true;
                }
            }

            var result = new Subclass();
            ok && result.prp === 3;
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn derived_constructors_without_super_throw_reference_error_on_fallthrough() {
    let unit = compile_test_unit(
        233,
        r"
            class Base {
                constructor() {}
            }

            class BadSubclass extends Base {
                constructor() {}
            }

            var status = 0;
            try {
                new BadSubclass();
                status = 1;
            } catch (error) {
                status = error.constructor === ReferenceError ? 2 : 3;
            }
            status;
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(2));
}

#[test]
fn derived_constructor_this_access_restriction_matches_test262_behavior() {
    let unit = compile_test_unit(
        234,
        r#"
            class Base {
                constructor(a, b) {
                    var o = new Object();
                    o.prp = a + b;
                    return o;
                }
            }

            var ok1 = false;
            class Subclass extends Base {
                constructor(a, b) {
                    var exn;
                    try {
                        this.prp1 = 3;
                    } catch (error) {
                        exn = error;
                    }
                    super(a, b);
                    ok1 =
                        exn instanceof ReferenceError
                        && this.prp === a + b
                        && this.prp1 === undefined
                        && !this.hasOwnProperty("prp1");
                    return this;
                }
            }

            var b = new Base(1, 2);
            var s = new Subclass(2, -1);

            var ok2 = false;
            class Subclass2 extends Base {
                constructor(x) {
                    super(1, 2);
                    if (x < 0) return;

                    var called = false;
                    function tmp() {
                        called = true;
                        return 3;
                    }
                    var exn = null;
                    try {
                        super(tmp(), 4);
                    } catch (error) {
                        exn = error;
                    }
                    ok2 = exn instanceof ReferenceError && called === true;
                }
            }

            var s2 = new Subclass2(1);
            var s3 = new Subclass2(-1);

            var subclass_call_type_error = false;
            try {
                Subclass.call(new Object(), 1, 2);
            } catch (error) {
                subclass_call_type_error = error instanceof TypeError;
            }

            var base_call_type_error = false;
            try {
                Base.call(new Object(), 1, 2);
            } catch (error) {
                base_call_type_error = error instanceof TypeError;
            }

            class BadSubclass extends Base {
                constructor() {}
            }

            var bad_subclass_reference_error = false;
            try {
                new BadSubclass();
            } catch (error) {
                bad_subclass_reference_error = error instanceof ReferenceError;
            }

            b.prp === 3
                && ok1
                && s.prp === 1
                && s.prp1 === undefined
                && !s.hasOwnProperty("prp1")
                && ok2
                && s2.prp === 3
                && s3.prp === 3
                && subclass_call_type_error
                && base_call_type_error
                && bad_subclass_reference_error;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn class_constructor_function_call_builtin_throws_type_error_catchably() {
    let unit = compile_test_unit(
        235,
        r"
            class Base {
                constructor() {}
            }

            var status = 0;
            try {
                Base.call(new Object(), 1, 2);
                status = 1;
            } catch (error) {
                status = error instanceof TypeError ? 2 : 3;
            }
            status;
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(2));
}

#[test]
fn derived_class_constructor_function_call_builtin_throws_type_error_catchably() {
    let unit = compile_test_unit(
        236,
        r"
            class Base {
                constructor() {}
            }

            class Subclass extends Base {
                constructor(a, b) {
                    super(a, b);
                    return this;
                }
            }

            var status = 0;
            try {
                Subclass.call(new Object(), 1, 2);
                status = 1;
            } catch (error) {
                status = error instanceof TypeError ? 2 : 3;
            }
            status;
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(2));
}

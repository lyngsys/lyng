use super::support::{compile_and_run, compile_and_run_string, compile_unit};
use lyng_js_common::AtomTable;
use lyng_js_env::Runtime;
use lyng_js_host::NoopHostHooks;
use lyng_js_types::Value;
use lyng_js_vm::Vm;

#[test]
fn phase6_classes_execute_base_constructors_methods_and_instance_fields() {
    let result = compile_and_run(
        r"
        class Box {
            constructor(value) {
                this.value = value;
            }

            extra = 2;

            read() {
                return this.value + this.extra;
            }
        }

        new Box(5).read();
        ",
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase6_unused_named_class_self_binding_does_not_reinitialize_in_loops() {
    let result = compile_and_run_string(
        r"
        let count = 0;
        for (let i = 0; i < 3; i++) {
            class C {}
            (class D {});
            count++;
        }
        String(count);
        ",
    );

    assert_eq!(result, "3");
}

#[test]
fn phase6_named_class_inner_bindings_are_immutable_and_tdz_for_computed_keys() {
    let result = compile_and_run_string(
        r#"
        function thrownName(fn) {
            try {
                fn();
                return "none";
            } catch (error) {
                return error.constructor.name;
            }
        }

        class Declared {
            tryBreak() { Declared = 4; }
        }
        let Expr = class Inner {
            tryBreak() { Inner = 4; }
        };

        let assignment =
            thrownName(() => new Declared().tryBreak()) + ":" +
            thrownName(() => new Expr().tryBreak());
        let tdz =
            thrownName(() => eval("class Bar { [Bar]() {} }")) + ":" +
            thrownName(() => eval("(class Baz { [Baz]() {} })"));

        class Outer {
            test(Outer) { return Outer; }
        }
        class Separate {
            method() { return Separate; }
        }
        const original = Separate;
        Separate = 13;

        assignment + "|" + tdz + "|" +
            String(new Outer().test(4)) + ":" +
            String(new original().method() === original);
        "#,
    );

    assert_eq!(
        result,
        "TypeError:TypeError|ReferenceError:ReferenceError|4:true"
    );
}

#[test]
fn phase6_classes_instance_fields_capture_outer_bindings() {
    let result = compile_and_run(
        r"
        function outer(value) {
            return class Box {
                constructor() {}
                field = value;
            };
        }

        new (outer(3))().field;
        ",
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase6_public_auto_accessors_install_backed_accessor_descriptors() {
    let result = compile_and_run(
        r#"
        var key = "computed";
        class C {
            accessor x = 1;
            accessor [key] = 2;
            static accessor sx = 3;
        }

        var c = new C();
        var x = Object.getOwnPropertyDescriptor(C.prototype, "x");
        var computed = Object.getOwnPropertyDescriptor(C.prototype, key);
        var sx = Object.getOwnPropertyDescriptor(C, "sx");

        x.set.call(c, 10);
        computed.set.call(c, 20);
        sx.set.call(C, 30);

        (x.get.call(c) === 10 ? 1 : 0)
            + (computed.get.call(c) === 20 ? 2 : 0)
            + (sx.get.call(C) === 30 ? 4 : 0)
            + (c.x === 10 ? 8 : 0)
            + (c[key] === 20 ? 16 : 0)
            + (C.sx === 30 ? 32 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn phase6_classes_reject_calling_class_constructors_without_new() {
    let result = compile_and_run(
        r"
        let ok = false;
        class C {}
        try {
            C();
        } catch (error) {
            ok = true;
        }
        ok;
        ",
    );

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn phase6_string_literal_constructor_method_defines_class_constructor() {
    let result = compile_and_run_string(
        r#"
        class Base {
            "constructor"() {
                return {};
            }
        }

        class Derived extends class {} {
            "constructor"() {
                return {};
            }
        }

        String(new Base() instanceof Base) + ":" + String(new Derived() instanceof Derived);
        "#,
    );

    assert_eq!(result, "false:false");
}

#[test]
fn phase6_classes_default_base_constructors_initialize_instance_fields() {
    let result = compile_and_run(
        r"
        function outer(value) {
            return class Box {
                field = value;
            };
        }

        new (outer(4))().field;
        ",
    );

    assert_eq!(result, Value::from_smi(4));
}

#[test]
fn phase6_classes_execute_static_fields_blocks_and_self_bindings() {
    let result = compile_and_run(
        r"
        let C = class Named {
            static total = 1;
            static {
                this.total = this.total + 1;
            }

            static self() {
                return Named;
            }
        };

        (C.total === 2 ? 1 : 0) + (C.self() === C ? 2 : 0);
        ",
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase6_class_field_initializers_capture_inner_class_name_binding() {
    let result = compile_and_run(
        r"
        class InstanceDecl {
            field = () => InstanceDecl;
        }
        const InstanceAlias = InstanceDecl;
        InstanceDecl = null;

        let InstanceExpr = class InnerInstance {
            field = () => InnerInstance;
        };
        const InstanceExprAlias = InstanceExpr;
        InstanceExpr = null;

        class StaticDecl {
            static field = () => StaticDecl;
        }
        const StaticAlias = StaticDecl;
        StaticDecl = null;

        let StaticExpr = class InnerStatic {
            static field = () => InnerStatic;
        };
        const StaticExprAlias = StaticExpr;
        StaticExpr = null;

        (new InstanceAlias().field() === InstanceAlias ? 1 : 0)
            + (new InstanceExprAlias().field() === InstanceExprAlias ? 2 : 0)
            + (StaticAlias.field() === StaticAlias ? 4 : 0)
            + (StaticExprAlias.field() === StaticExprAlias ? 8 : 0);
        ",
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn phase6_class_field_direct_eval_resolves_inner_class_name_binding() {
    let result = compile_and_run(
        r#"
        class InstanceDecl {
            field = eval("InstanceDecl");
            arrow = () => eval("InstanceDecl");
        }
        const InstanceAlias = InstanceDecl;
        InstanceDecl = null;
        const instance = new InstanceAlias();

        let InstanceExpr = class InnerInstance {
            field = eval("InnerInstance");
            arrow = () => eval("InnerInstance");
        };
        const InstanceExprAlias = InstanceExpr;
        InstanceExpr = null;
        const instanceExpr = new InstanceExprAlias();

        class StaticDecl {
            static field = eval("StaticDecl");
            static arrow = () => eval("StaticDecl");
        }
        const StaticAlias = StaticDecl;
        StaticDecl = null;

        let StaticExpr = class InnerStatic {
            static field = eval("InnerStatic");
            static arrow = () => eval("InnerStatic");
        };
        const StaticExprAlias = StaticExpr;
        StaticExpr = null;

        let anonymousStaticResult = "none";
        try {
            let C = class {
                static field = eval("C");
            };
            anonymousStaticResult = "no error";
        } catch (error) {
            anonymousStaticResult = error.name;
        }

        (instance.field === InstanceAlias ? 1 : 0)
            + (instance.arrow() === InstanceAlias ? 2 : 0)
            + (instanceExpr.field === InstanceExprAlias ? 4 : 0)
            + (instanceExpr.arrow() === InstanceExprAlias ? 8 : 0)
            + (StaticAlias.field === StaticAlias ? 16 : 0)
            + (StaticAlias.arrow() === StaticAlias ? 32 : 0)
            + (StaticExprAlias.field === StaticExprAlias ? 64 : 0)
            + (StaticExprAlias.arrow() === StaticExprAlias ? 128 : 0)
            + (anonymousStaticResult === "ReferenceError" ? 256 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(511));
}

#[test]
fn phase6_static_field_initializers_bind_this_to_the_class_object() {
    let result = compile_and_run_string(
        r"
        let field = (class Named {
            static value = this.name;
        }).value;

        field;
        ",
    );

    assert_eq!(result, "Named");
}

#[test]
fn phase6_direct_eval_in_static_field_initializer_uses_class_this() {
    let result = compile_and_run_string(
        r#"
        class C {
            static f = "test";
            static g = this.f + "262";
            static h = eval("this.g") + "test";
        }

        C.h;
        "#,
    );

    assert_eq!(result, "test262test");
}

#[test]
fn phase6_class_static_blocks_lower_class_declarations_after_private_decorators() {
    let result = compile_and_run(
        r"
        class C {
            static #dec() {}

            static {
                @C.#dec class D {
                    static value = 7;
                }
                this.value = D.value;
            }
        }

        C.value;
        ",
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase6_public_instance_fields_define_through_proxy_traps() {
    let result = compile_and_run_string(
        r#"
        let log = "";
        function ProxyBase() {
            return new Proxy(this, {
                defineProperty(target, key, descriptor) {
                    log += key
                        + ":"
                        + descriptor.value
                        + ":"
                        + String(descriptor.enumerable)
                        + ":"
                        + String(descriptor.configurable)
                        + ":"
                        + String(descriptor.writable)
                        + ";";
                    return Reflect.defineProperty(target, key, descriptor);
                }
            });
        }

        class C extends ProxyBase {
            f = 3;
            g = "Test262";
        }

        new C();
        log;
        "#,
    );

    assert_eq!(result, "f:3:true:true:true;g:Test262:true:true:true;");
}

#[test]
fn phase6_private_elements_reject_non_extensible_receivers() {
    let result = compile_and_run_string(
        r#"
        let log = "";
        function record(label, callback) {
            try {
                callback();
                log += label + ":miss;";
            } catch (error) {
                log += label + ":" + String(error instanceof TypeError) + ";";
            }
        }

        class SealingBase {
            constructor() {
                Object.preventExtensions(this);
            }
        }
        class PrivateFieldOnBase extends SealingBase {
            #value = 1;
        }
        class PrivateMethodOnBase extends SealingBase {
            #method() {
                return 1;
            }
        }

        class ReturnOverrideBase {
            constructor(object) {
                return object;
            }
        }
        class PrivateFieldOnReturnOverride extends ReturnOverrideBase {
            #value = 1;
            constructor(object) {
                super(object);
            }
        }

        record("field", () => new PrivateFieldOnBase());
        record("method", () => new PrivateMethodOnBase());
        record("return", () => new PrivateFieldOnReturnOverride(Object.preventExtensions({})));
        record("static", () => {
            class PrivateStaticField {
                static #value = (Object.preventExtensions(PrivateStaticField), 1);
            }
        });

        log;
        "#,
    );

    assert_eq!(result, "field:true;method:true;return:true;static:true;");
}

#[test]
fn phase6_async_private_methods_async_arrows_capture_parent_arguments() {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(
        r#"
        class C {
            async #method(x) {
                let captured = arguments;
                return async () => captured === arguments;
            }

            async method(x) {
                return this.#method(x);
            }

            static async #staticMethod(x) {
                let captured = arguments;
                return async () => captured === arguments;
            }

            static async staticMethod(x) {
                return this.#staticMethod(x);
            }
        }

        let c = new C();
        Promise.all([
            c.method("instance").then(callback => callback()),
            C.staticMethod("static").then(callback => callback()),
        ]).then(results => results[0] === true && results[1] === true);
        "#,
        &mut atoms,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("compiled script should execute");
    let promise = result
        .as_object_ref()
        .expect("script should return a promise");
    let record = agent
        .promise_record(promise)
        .expect("promise should remain tracked after evaluation");

    assert_eq!(record.state(), lyng_js_env::PromiseState::Fulfilled);
    assert_eq!(record.result(), Value::from_bool(true));
}

#[test]
fn phase6_anonymous_class_expressions_infer_names_before_static_initializers() {
    let result = compile_and_run_string(
        r"
        let className;
        let C = class {
            static value = (className = this.name);
        };

        className;
        ",
    );

    assert_eq!(result, "C");
}

#[test]
fn phase6_class_expression_name_descriptors_match_class_names() {
    let result = compile_and_run_string(
        r#"
        function describe(C) {
            let descriptor = Object.getOwnPropertyDescriptor(C, "name");
            return descriptor.value + ":"
                + String(descriptor.writable) + ":"
                + String(descriptor.enumerable) + ":"
                + String(descriptor.configurable);
        }

        describe(class {}) + "|"
            + describe(class Named {}) + "|"
            + describe(class { constructor() {} }) + "|"
            + describe(class WithCtor { constructor() {} });
        "#,
    );

    assert_eq!(
        result,
        ":false:false:true|Named:false:false:true|:false:false:true|WithCtor:false:false:true"
    );
}

#[test]
fn phase6_classes_link_derived_constructor_and_prototype_chains() {
    let result = compile_and_run(
        r"
        class Base {}
        class Derived extends Base {}

        (Object.getPrototypeOf(Derived) === Base ? 1 : 0)
            + (Object.getPrototypeOf(Derived.prototype) === Base.prototype ? 2 : 0);
        ",
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase6_class_heritage_rejects_non_constructors_before_prototype_lookup() {
    let result = compile_and_run_string(
        r#"
        function check(label, value) {
            try {
                class Derived extends value {}
                return label + ":miss";
            } catch (error) {
                return label + ":" + (error instanceof TypeError ? "type" : String(error));
            }
        }

        let arrow = () => {};
        Object.defineProperty(arrow, "prototype", {
            get() {
                throw "arrow prototype";
            }
        });

        let bound = (() => {}).bind();
        Object.defineProperty(bound, "prototype", {
            get() {
                throw "bound prototype";
            }
        });

        let proxy = new Proxy(() => {}, {
            get() {
                throw "proxy prototype";
            }
        });

        function* generator() {}

        check("arrow", arrow) + "|"
            + check("bound", bound) + "|"
            + check("proxy", proxy) + "|"
            + check("generator", generator);
        "#,
    );

    assert_eq!(result, "arrow:type|bound:type|proxy:type|generator:type");
}

#[test]
fn phase6_symbol_is_valid_class_heritage_but_throws_from_super_call() {
    let result = compile_and_run_string(
        r#"
        let status = "";
        try {
            class S extends Symbol {}
            status += "defined";
            try {
                new S();
                status += ":miss";
            } catch (error) {
                status += error instanceof TypeError ? ":new-type" : ":new-other";
            }
        } catch (error) {
            status = error instanceof TypeError ? "define-type" : "define-other";
        }
        status;
        "#,
    );

    assert_eq!(result, "defined:new-type");
}

#[test]
fn phase6_computed_instance_field_keys_are_fixed_at_class_definition_time() {
    let result = compile_and_run(
        r#"
        let key = false;
        let C = class {
            [key] = key;
        };

        let first = new C();
        key = true;
        let second = new C();

        (first.false === false ? 1 : 0)
            + (second.false === true ? 2 : 0)
            + (first.hasOwnProperty("true") ? 0 : 4)
            + (second.hasOwnProperty("true") ? 0 : 8);
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn phase6_computed_class_element_keys_apply_to_property_key() {
    let result = compile_and_run(
        r#"
        let log = "";
        let methodKey = {
            [Symbol.toPrimitive](hint) {
                log += "m:" + hint + "|";
                return "method";
            }
        };
        let getterKey = {
            [Symbol.toPrimitive](hint) {
                log += "g:" + hint + "|";
                return "value";
            }
        };
        let setterKey = {
            [Symbol.toPrimitive](hint) {
                log += "s:" + hint + "|";
                return "value";
            }
        };
        let fieldKey = {
            [Symbol.toPrimitive](hint) {
                log += "f:" + hint + "|";
                return "field";
            }
        };

        class C {
            [fieldKey] = 10;
            [methodKey]() { return this.field; }
            get [getterKey]() { return this.field + 1; }
            set [setterKey](value) { this.field = value; }
        }

        let instance = new C();
        let total = 0;
        total += instance.method() === 10 ? 1 : 0;
        total += instance.value === 11 ? 2 : 0;
        instance.value = 20;
        total += instance.field === 20 ? 4 : 0;
        total += log === "f:string|m:string|g:string|s:string|" ? 8 : 0;
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn phase6_class_computed_names_use_strict_assignment_semantics() {
    let result = compile_and_run(
        r"
        var threw = false;
        try {
            class C {
                [Object.preventExtensions({}).x = 1]() {}
            }
        } catch (error) {
            threw = error instanceof TypeError;
        }
        threw;
        ",
    );

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn phase6_computed_class_function_expression_keys_use_trimmed_source_text() {
    let result = compile_and_run(
        r"
        class C {
            [function () {}]() {
                return 1;
            }
            static [function () {}]() {
                return 2;
            }
        }

        new C()[function () {}]()
            + C[function () {}]()
            + new C()[String(function () {})]()
            + C[String(function () {})]();
        ",
    );

    assert_eq!(result, Value::from_smi(6));
}

#[test]
fn phase6_intercalated_static_and_instance_computed_field_keys_run_before_values() {
    let result = compile_and_run(
        r#"
        let i = 0;
        class C {
            [i++] = i++;
            static [i++] = i++;
            [i++] = i++;
        }

        let c = new C();
        (c[0] === 4 ? 1 : 0)
            + (C[1] === 3 ? 2 : 0)
            + (c[2] === 5 ? 4 : 0)
            + (i === 6 ? 8 : 0)
            + (c.hasOwnProperty("1") ? 0 : 16)
            + (C.hasOwnProperty("0") ? 0 : 32)
            + (C.hasOwnProperty("2") ? 0 : 64);
        "#,
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn phase6_classes_execute_private_instance_fields_and_brand_checks() {
    let result = compile_and_run(
        r"
        class Box {
            #value = 4;

            read() {
                return this.#value;
            }

            static hasBox(value) {
                return #value in value;
            }
        }

        let box = new Box();
        (Box.hasBox(box) ? 1 : 0) + box.read();
        ",
    );

    assert_eq!(result, Value::from_smi(5));
}

#[test]
fn phase6_base_class_private_fields_initialize_before_constructor_defaults() {
    let result = compile_and_run(
        r"
        class A {
            #x = 41;
            constructor(value = this.#x) {
                this.value = value;
            }
        }

        new A().value;
        ",
    );

    assert_eq!(result, Value::from_smi(41));
}

#[test]
fn phase6_default_class_constructors_support_arrow_field_initializers() {
    let result = compile_and_run(
        r"
        let C = class {
            field = function() {};
            #field = (a, b, c, d) => undefined;

            accessPrivateField() {
                return this.#field;
            }
        };

        let instance = new C();
        instance.field.length + instance.accessPrivateField().length;
        ",
    );

    assert_eq!(result, Value::from_smi(4));
}

#[test]
fn phase6_classes_private_fields_reject_wrong_brands_and_support_static_storage() {
    let result = compile_and_run(
        r"
        let wrongBrand = false;
        class Counter {
            static #total = 1;
            #value = 2;

            read() {
                return this.#value;
            }

            static bump() {
                this.#total = this.#total + 1;
                return this.#total;
            }
        }

        try {
            Counter.prototype.read.call({});
        } catch (error) {
            wrongBrand = true;
        }

        (wrongBrand ? 1 : 0) + Counter.bump();
        ",
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase6_static_computed_prototype_elements_throw_type_error() {
    let result = compile_and_run(
        r#"
        let total = 0;
        let key = "prototype";

        try {
            (class {
                static [key] = 1;
            });
        } catch (error) {
            total += error.constructor === TypeError ? 1 : 0;
        }

        try {
            (class {
                static [key]() {}
            });
        } catch (error) {
            total += error.constructor === TypeError ? 2 : 0;
        }

        try {
            (class {
                static get [key]() {}
            });
        } catch (error) {
            total += error.constructor === TypeError ? 4 : 0;
        }

        try {
            (class {
                static set [key](value) {}
            });
        } catch (error) {
            total += error.constructor === TypeError ? 8 : 0;
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn phase6_class_constructor_own_property_names_order_indices_before_builtin_names() {
    let result = compile_and_run_string(
        r#"
        class C {
            static a() { return "A"; }
            static [1]() { return "B"; }
            static c() { return "C"; }
            static [2]() { return "D"; }
        }

        Object.getOwnPropertyNames(C).join(",");
        "#,
    );

    assert_eq!(result, "1,2,length,name,prototype,a,c");
}

#[test]
fn phase6_computed_symbol_methods_preserve_symbol_keys() {
    let result = compile_and_run(
        r#"
        let sym1 = Symbol();
        let sym2 = Symbol();
        class C {
            a() { return "A"; }
            [sym1]() { return "B"; }
            c() { return "C"; }
            [sym2]() { return "D"; }
        }

        let score = 0;
        let names = Object.getOwnPropertyNames(C.prototype);
        let symbols = Object.getOwnPropertySymbols(C.prototype);

        score += new C().a() === "A" ? 1 : 0;
        score += new C()[sym1]() === "B" ? 2 : 0;
        score += new C().c() === "C" ? 4 : 0;
        score += new C()[sym2]() === "D" ? 8 : 0;
        score += Object.keys(C.prototype).length === 0 ? 16 : 0;
        score += (names.length === 3 && names[0] === "constructor" && names[1] === "a" && names[2] === "c") ? 32 : 0;
        score += (symbols.length === 2 && symbols[0] === sym1 && symbols[1] === sym2) ? 64 : 0;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn phase6_class_constructors_expose_non_writable_prototype_property() {
    let result = compile_and_run(
        r#"
        class C {}
        var descriptor = Object.getOwnPropertyDescriptor(C, "prototype");

        (descriptor.configurable === false ? 1 : 0)
            + (descriptor.enumerable === false ? 2 : 0)
            + (descriptor.writable === false ? 4 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase6_class_prototype_descriptor_survives_helper_functions_in_scope() {
    let result = compile_and_run(
        r#"
        function assert(mustBeTrue, message) {
            if (mustBeTrue === true) {
                return;
            }
            throw new Error(message);
        }

        assert.sameValue = function(actual, expected, message) {
            if (actual !== expected) {
                throw new Error(message);
            }
        };

        class C {}
        var descriptor = Object.getOwnPropertyDescriptor(C, "prototype");

        assert.sameValue(descriptor.configurable, false, "configurable");
        assert.sameValue(descriptor.enumerable, false, "enumerable");
        assert.sameValue(descriptor.writable, false, "writable");
        0;
        "#,
    );

    assert_eq!(result, Value::from_smi(0));
}

#[test]
fn phase6_method_and_accessor_names_follow_property_keys() {
    let result = compile_and_run(
        r#"
        let namedSym = Symbol("test262");
        let anonSym = Symbol();
        class A {
            get x() { return 1; }
            set x(value) {}
            [anonSym]() {}
            [namedSym]() {}
            static get y() { return 1; }
            static set y(value) {}
        }

        let prototypeDescriptor = Object.getOwnPropertyDescriptor(A.prototype, "x");
        let staticDescriptor = Object.getOwnPropertyDescriptor(A, "y");

        (prototypeDescriptor.get.name === "get x" ? 1 : 0)
            + (prototypeDescriptor.set.name === "set x" ? 2 : 0)
            + (A.prototype[anonSym].name === "" ? 4 : 0)
            + (A.prototype[namedSym].name === "[test262]" ? 8 : 0)
            + (staticDescriptor.get.name === "get y" ? 16 : 0)
            + (staticDescriptor.set.name === "set y" ? 32 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn phase6_method_and_accessor_name_descriptors_match_test262_symbol_rows() {
    let result = compile_and_run(
        r#"
        var methodNamedSym = Symbol("test262");
        var methodAnonSym = Symbol();
        var accessorNamedSym = Symbol("test262");
        var accessorAnonSym = Symbol();

        class A {
            id() {}
            [methodAnonSym]() {}
            [methodNamedSym]() {}
            get [accessorAnonSym]() {}
            get [accessorNamedSym]() {}
            set [accessorAnonSym](value) {}
            set [accessorNamedSym](value) {}
            static id() {}
            static [methodAnonSym]() {}
            static [methodNamedSym]() {}
            static get [accessorAnonSym]() {}
            static get [accessorNamedSym]() {}
            static set [accessorAnonSym](value) {}
            static set [accessorNamedSym](value) {}
        }

        function check(fn, expected) {
            let descriptor = Object.getOwnPropertyDescriptor(fn, "name");
            return (descriptor.value === expected ? 1 : 0)
                + (descriptor.writable === false ? 2 : 0)
                + (descriptor.enumerable === false ? 4 : 0)
                + (descriptor.configurable === true ? 8 : 0);
        }

        var prototypeAnonGetter = Object.getOwnPropertyDescriptor(A.prototype, accessorAnonSym).get;
        var prototypeNamedGetter = Object.getOwnPropertyDescriptor(A.prototype, accessorNamedSym).get;
        var prototypeAnonSetter = Object.getOwnPropertyDescriptor(A.prototype, accessorAnonSym).set;
        var prototypeNamedSetter = Object.getOwnPropertyDescriptor(A.prototype, accessorNamedSym).set;
        var staticAnonGetter = Object.getOwnPropertyDescriptor(A, accessorAnonSym).get;
        var staticNamedGetter = Object.getOwnPropertyDescriptor(A, accessorNamedSym).get;
        var staticAnonSetter = Object.getOwnPropertyDescriptor(A, accessorAnonSym).set;
        var staticNamedSetter = Object.getOwnPropertyDescriptor(A, accessorNamedSym).set;

        check(A.prototype.id, "id")
            + check(A.prototype[methodAnonSym], "")
            + check(A.prototype[methodNamedSym], "[test262]")
            + check(prototypeAnonGetter, "get ")
            + check(prototypeNamedGetter, "get [test262]")
            + check(prototypeAnonSetter, "set ")
            + check(prototypeNamedSetter, "set [test262]")
            + check(A.id, "id")
            + check(A[methodAnonSym], "")
            + check(A[methodNamedSym], "[test262]")
            + check(staticAnonGetter, "get ")
            + check(staticNamedGetter, "get [test262]")
            + check(staticAnonSetter, "set ")
            + check(staticNamedSetter, "set [test262]");
        "#,
    );

    assert_eq!(result, Value::from_smi(14 * 15));
}

#[test]
fn phase6_private_method_names_include_hash_prefix() {
    let result = compile_and_run_string(
        r#"
        class C {
            #method() {}
            *#generator() {}
            async #asyncMethod() {}
            static #staticMethod() {}

            static names(instance) {
                return [
                    instance.#method.name,
                    instance.#generator.name,
                    instance.#asyncMethod.name,
                    this.#staticMethod.name
                ].join("|");
            }
        }

        C.names(new C());
        "#,
    );

    assert_eq!(result, "#method|#generator|#asyncMethod|#staticMethod");
}

#[test]
fn phase6_class_field_anonymous_function_names_follow_field_keys() {
    let result = compile_and_run_string(
        r#"
        class C {
            field = function() {};
            arrow = () => {};
            #privateField = function() {};
            static staticField = function() {};
            static #staticPrivateField = () => {};

            privateName() {
                return this.#privateField.name;
            }

            static names(instance) {
                return [
                    instance.field.name,
                    instance.arrow.name,
                    instance.privateName(),
                    this.staticField.name,
                    this.#staticPrivateField.name
                ].join("|");
            }
        }

        C.names(new C());
        "#,
    );

    assert_eq!(
        result,
        "field|arrow|#privateField|staticField|#staticPrivateField"
    );
}

#[test]
fn phase6_computed_class_field_anonymous_function_names_follow_property_keys() {
    let result = compile_and_run_string(
        r#"
        class C {
            ["field"] = () => {};
            [5] = function() {};
            static ["staticField"] = async () => {};
            static [6] = function*() {};
        }

        let instance = new C();
        [
            instance.field.name,
            instance[5].name,
            C.staticField.name,
            C[6].name
        ].join("|");
        "#,
    );

    assert_eq!(result, "field|5|staticField|6");
}

#[test]
fn phase6_numeric_class_field_anonymous_class_names_follow_property_keys() {
    let result = compile_and_run_string(
        r#"
        class C {
            128 = class { static observed = this.name; };
            129n = class { static observed = this.name; };
            static 130 = class { static observed = this.name; };
            static 131n = class { static observed = this.name; };
        }

        let instance = new C();
        [
            instance[128].name,
            instance[128].observed,
            instance[129].name,
            instance[129].observed,
            C[130].name,
            C[130].observed,
            C[131].name,
            C[131].observed
        ].join("|");
        "#,
    );

    assert_eq!(result, "128|128|129|129|130|130|131|131");
}

#[test]
fn phase6_class_fields_after_arrow_initializers_do_not_require_semicolons() {
    let result = compile_and_run_string(
        r#"
        class C {
            ["first"] = () => {}
            ["second"] = async () => {};
        }

        let instance = new C();
        [
            instance.first.name,
            instance.second.name,
            typeof instance.first,
            typeof instance.second
        ].join("|");
        "#,
    );

    assert_eq!(result, "first|second|function|function");
}

#[test]
fn phase6_private_method_initialization_throws_on_duplicate_install() {
    let result = compile_and_run(
        r"
        class Base {
            constructor(value) {
                return value;
            }
        }

        class C extends Base {
            #m() {}
        }

        let object = {};
        new C(object);

        let threw = false;
        try {
            new C(object);
        } catch (error) {
            threw = error.constructor === TypeError;
        }

        threw;
        ",
    );

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn phase6_private_accessor_initialization_throws_on_duplicate_install() {
    let result = compile_and_run(
        r"
        class Base {
            constructor(value) {
                return value;
            }
        }

        var total = 0;

        var getOnly = {};
        class GetOnly extends Base {
            get #p() {}
        }
        new GetOnly(getOnly);
        try {
            new GetOnly(getOnly);
        } catch (error) {
            total = total + (error.constructor === TypeError ? 1 : 0);
        }

        var setOnly = {};
        class SetOnly extends Base {
            set #p(value) {}
        }
        new SetOnly(setOnly);
        try {
            new SetOnly(setOnly);
        } catch (error) {
            total = total + (error.constructor === TypeError ? 2 : 0);
        }

        var getSet = {};
        class GetSet extends Base {
            get #p() {}
            set #p(value) {}
        }
        new GetSet(getSet);
        try {
            new GetSet(getSet);
        } catch (error) {
            total = total + (error.constructor === TypeError ? 4 : 0);
        }

        total;
        ",
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase6_private_destructuring_targets_evaluate_before_source_getters() {
    let result = compile_and_run_string(
        r#"
        let outcome = "missing";
        try {
            new (class C extends class {} {
                #field;

                constructor() {
                    let init = () => super();
                    let object = {
                        get a() {
                            init();
                            return 1;
                        }
                    };

                    ({ a: this.#field } = object);
                }
            })();
            outcome = "missing";
        } catch (error) {
            outcome = error.name;
        }

        outcome;
        "#,
    );

    assert_eq!(result, "ReferenceError");
}

#[test]
fn phase6_private_destructuring_target_evaluation_matches_test262_rows() {
    let result = compile_and_run(
        r#"
        var total = 0;

        class DerivedReferenceError extends class {} {
            #field;

            constructor() {
                var init = () => super();
                var object = {
                    get a() {
                        init();
                    }
                };

                ({ a: this.#field } = object);
            }
        }

        try {
            new DerivedReferenceError();
        } catch (error) {
            total = total + (error.constructor === ReferenceError ? 1 : 0);
        }

        class GetterThrowsBeforeBrandCheck {
            #field;

            m() {
                var object = {
                    get a() {
                        throw "getter";
                    }
                };

                ({ a: this.#field } = object);
            }
        }

        try {
            GetterThrowsBeforeBrandCheck.prototype.m.call({});
        } catch (error) {
            total = total + (error === "getter" ? 2 : 0);
        }

        class Base {
            constructor(value) {
                return value;
            }
        }

        class ReusedReceiver extends Base {
            #field;

            m() {
                var init = () => new ReusedReceiver(this);
                var object = {
                    get a() {
                        init();
                        return "pass";
                    }
                };

                ({ a: this.#field } = object);
                return this.#field === "pass";
            }
        }

        total + (ReusedReceiver.prototype.m.call({}) ? 4 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase6_second_super_from_arrow_runs_base_only_once_for_fields() {
    let result = compile_and_run(
        r"
        let baseCtorCalled = 0;
        let fieldInitCalled = 0;
        let secondSuper = false;

        class Base {
            constructor() {
                ++baseCtorCalled;
            }
        }

        let C = class extends Base {
            field = ++fieldInitCalled;

            constructor() {
                super();
                try {
                    (() => super())();
                } catch (error) {
                    secondSuper = error.constructor === ReferenceError;
                }
            }
        };

        new C();

        (baseCtorCalled === 2 ? 1 : 0)
            + (fieldInitCalled === 1 ? 2 : 0)
            + (secondSuper ? 4 : 0);
        ",
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase6_private_fields_capture_into_inner_functions_and_arrows() {
    let result = compile_and_run(
        r"
        class Box {
            #value = 5;

            read() {
                let readWithFunction = function(value) {
                    return value.#value;
                };
                let readWithArrow = (value) => value.#value;
                return readWithFunction(this) + readWithArrow(this);
            }
        }

        new Box().read();
        ",
    );

    assert_eq!(result, Value::from_smi(10));
}

#[test]
fn phase6_nested_classes_can_capture_outer_private_fields() {
    let result = compile_and_run(
        r"
        class Outer {
            #value = 7;

            makeReader() {
                return class Reader {
                    read(target) {
                        return target.#value;
                    }
                };
            }
        }

        let outer = new Outer();
        let Reader = outer.makeReader();
        new Reader().read(outer);
        ",
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase6_nested_classes_can_capture_outer_private_methods() {
    let result = compile_and_run(
        r"
        class Outer {
            #value() {
                return 9;
            }

            makeReader() {
                return class Reader {
                    read(target) {
                        return target.#value();
                    }
                };
            }
        }

        let outer = new Outer();
        let Reader = outer.makeReader();
        new Reader().read(outer);
        ",
    );

    assert_eq!(result, Value::from_smi(9));
}

#[test]
fn phase6_nested_classes_can_capture_outer_private_getters() {
    let result = compile_and_run(
        r"
        class Outer {
            get #value() {
                return 8;
            }

            makeReader() {
                return class Reader {
                    read(target) {
                        return target.#value;
                    }
                };
            }
        }

        let outer = new Outer();
        let Reader = outer.makeReader();
        new Reader().read(outer);
        ",
    );

    assert_eq!(result, Value::from_smi(8));
}

#[test]
fn phase6_nested_classes_can_access_outer_static_private_setters() {
    let result = compile_and_run(
        r"
        let wrongBrand = false;
        class Outer {
            static _value = 0;

            static set #value(next) {
                this._value = next;
            }

            static Inner = class {
                static write(target) {
                    target.#value = 6;
                }
            };
        }

        Outer.Inner.write(Outer);
        try {
            Outer.Inner.write({});
        } catch (error) {
            wrongBrand = true;
        }

        (wrongBrand ? 1 : 0) + Outer._value;
        ",
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase6_nested_classes_can_access_outer_static_private_fields() {
    let result = compile_and_run(
        r"
        let wrongBrand = false;

        class Outer {
            static #value = 6;

            static Inner = class {
                static read(target) {
                    return target.#value;
                }
            };
        }

        let read = Outer.Inner.read(Outer);
        try {
            Outer.Inner.read({});
        } catch (error) {
            wrongBrand = true;
        }

        read + (wrongBrand ? 1 : 0);
        ",
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase6_class_expressions_support_nested_static_private_field_access() {
    let result = compile_and_run(
        r"
        let C = class {
            static #value = 6;

            static Inner = class {
                static read(target) {
                    return target.#value;
                }
            };
        };

        C.Inner.read(C);
        ",
    );

    assert_eq!(result, Value::from_smi(6));
}

#[test]
fn phase6_class_expressions_catch_type_error_after_nested_static_private_field_access() {
    let result = compile_and_run(
        r"
        let caught = false;

        let C = class {
            static #value = 6;

            static Inner = class {
                static read(target) {
                    return target.#value;
                }
            };
        };

        let read = C.Inner.read(C);
        try {
            C.Inner.missing(C.Inner);
        } catch (error) {
            caught = true;
        }

        (read === 6 ? 1 : 0) + (caught ? 2 : 0);
        ",
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase6_private_methods_are_not_installed_before_super_returns() {
    let result = compile_and_run(
        r"
        let threw = false;

        class Base {
            field = this.call();
        }

        class Derived extends Base {
            call() {
                return this.#value();
            }

            #value() {
                return 1;
            }
        }

        try {
            new Derived();
        } catch (error) {
            threw = true;
        }

        threw;
        ",
    );

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn phase6_super_property_access_works_in_object_literal_methods() {
    let result = compile_and_run(
        r#"
        let fromGet = 0;
        let fromCall = 0;
        let parent = {
            value: 3,
            inc() {
                return this.value + 1;
            },
            set assign(v) {
                this.stored = v;
            }
        };
        let obj = {
            value: 5,
            method(key) {
                fromGet = super[key];
                fromCall = super.inc();
                super.assign = fromCall;
                return this.stored;
            }
        };

        Object.setPrototypeOf(obj, parent);
        obj.method("value") + fromGet + fromCall;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn phase6_super_computed_property_resolves_base_before_to_property_key() {
    let result = compile_and_run_string(
        r#"
        let proto = { p: "ok" };
        let proto2 = { p: "bad" };
        let obj = {
            m() {
                return super[key];
            }
        };
        Object.setPrototypeOf(obj, proto);
        let key = {
            toString() {
                Object.setPrototypeOf(obj, proto2);
                return "p";
            }
        };

        obj.m();
        "#,
    );

    assert_eq!(result, "ok");
}

#[test]
fn phase6_super_property_targets_work_in_for_in_and_for_of_heads() {
    let result = compile_and_run_string(
        r#"
        let log = [];

        class ForInCase {
            constructor() {
                let hits = 0;
                for (super.prop in { first: 1, second: 2 }) {
                    hits++;
                }
                log.push(this.prop + ":" + hits);
            }
        }

        ({
            method() {
                let hits = 0;
                for (super["prop"] of [1, 2]) {
                    hits++;
                }
                log.push(this.prop + ":" + hits);
            }
        }).method();

        new ForInCase();
        log.join("|");
        "#,
    );

    assert_eq!(result, "2:2|second:2");
}

#[test]
fn phase6_super_computed_compound_assignment_reuses_base_before_to_property_key() {
    let result = compile_and_run_string(
        r#"
        let log = "";
        let proto = {
            get p() {
                log += "get1|";
                return 1;
            },
            set p(value) {
                log += "set1:" + value;
            }
        };
        let proto2 = {
            get p() {
                log += "get2|";
                return -1;
            },
            set p(value) {
                log += "set2:" + value;
            }
        };
        let obj = {
            m() {
                return super[key] += 1;
            }
        };
        Object.setPrototypeOf(obj, proto);
        let key = {
            toString() {
                Object.setPrototypeOf(obj, proto2);
                return "p";
            }
        };

        String(obj.m()) + "|" + log;
        "#,
    );

    assert_eq!(result, "2|get1|set1:2");
}

#[test]
fn phase6_super_assignment_null_base_evaluates_rhs_before_type_error() {
    let result = compile_and_run_string(
        r#"
        let count = 0;
        let caughtNamed = false;
        let caughtComputed = false;
        class C {
            static named() {
                super.x = count += 1;
            }
            static computed() {
                super[0] = count += 1;
            }
        }
        Object.setPrototypeOf(C, null);
        try {
            C.named();
        } catch (error) {
            caughtNamed = error instanceof TypeError;
        }
        try {
            C.computed();
        } catch (error) {
            caughtComputed = error instanceof TypeError;
        }
        String(caughtNamed) + "|" + String(caughtComputed) + "|" + String(count);
        "#,
    );

    assert_eq!(result, "true|true|2");
}

#[test]
fn phase6_methods_record_home_object_for_super_dispatch() {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(
        r"
        let obj = {
            method() {
                return super.value;
            }
        };
        obj.method;
        ",
        &mut atoms,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let method = vm
        .evaluate_script(agent, realm, &unit)
        .expect("compiled script should execute")
        .as_object_ref()
        .expect("object literal method should evaluate to a function object");
    assert!(
        agent
            .objects()
            .function_data(method)
            .and_then(lyng_js_objects::FunctionObjectData::home_object)
            .is_some(),
        "method closures using super should retain [[HomeObject]] metadata"
    );
}

#[test]
fn phase6_object_literal_proto_data_properties_set_the_prototype() {
    let result = compile_and_run(
        r"
        let proto = { value: 4 };
        let obj = { __proto__: proto };
        obj.value;
        ",
    );

    assert_eq!(result, Value::from_smi(4));
}

#[test]
fn phase6_derived_constructors_bind_this_once_and_initialize_instance_elements() {
    let result = compile_and_run(
        r"
        let summary = 0;
        class Base {
            constructor() {
                this.base = 1;
            }
        }

        class Derived extends Base {
            field = 2;

            constructor() {
                let beforeSuper = false;
                try {
                    this.field;
                } catch (error) {
                    beforeSuper = true;
                }

                super();

                let secondSuper = false;
                try {
                    super();
                } catch (error) {
                    secondSuper = true;
                }

                summary = (beforeSuper ? 1 : 0)
                    + (secondSuper ? 2 : 0)
                    + this.base
                    + this.field;
            }
        }

        new Derived();
        summary;
        ",
    );

    assert_eq!(result, Value::from_smi(6));
}

#[test]
fn phase6_derived_constructors_resolve_outer_lexical_bindings_with_forced_environment() {
    let result = compile_and_run(
        r"
        let source = { c: 3, d: 4 };

        class Base {
            constructor(obj) {
                this.total = obj.c + obj.d + Object.keys(obj).length;
            }
        }

        class Derived extends Base {
            constructor() {
                super({ ...source });
            }
        }

        new Derived().total;
        ",
    );

    assert_eq!(result, Value::from_smi(9));
}

#[test]
fn phase6_super_property_set_is_a_strict_reference() {
    let result = compile_and_run(
        r#"
        "use strict";
        let failed = false;
        let obj = {
            method() {
                super.x = 1;
                Object.freeze(obj);
                try {
                    super.y = 2;
                } catch (error) {
                    failed = error.constructor === TypeError;
                }
            }
        };

        obj.method();
        failed;
        "#,
    );

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn phase6_derived_constructors_reject_primitive_return_before_super() {
    let result = compile_and_run(
        r"
        let failed = false;
        class Base {}
        class Derived extends Base {
            constructor() {
                return 1;
            }
        }

        try {
            new Derived();
        } catch (error) {
            failed = true;
        }
        failed;
        ",
    );

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn phase6_derived_constructor_iterator_close_can_initialize_this_before_return() {
    let result = compile_and_run_string(
        r"
        var iter = {
            [Symbol.iterator]() { return this; },
            next() { return { done: false }; },
            return() {
                this.f();
                return { done: true };
            },
        };

        class C extends class {} {
            constructor() {
                iter.f = () => super();
                for (var k of iter) {
                    return;
                }
            }
        }

        try {
            var o = new C();
            typeof o;
        } catch (error) {
            error.name;
        }
        ",
    );

    assert_eq!(result, "object");
}

#[test]
fn phase6_derived_super_call_selects_constructor_before_evaluating_arguments() {
    let result = compile_and_run_string(
        r#"
        function Base() {}
        function MyError() {}
        let log = [];

        class BeforeSwizzle extends Base {
            constructor() {
                super(log.push("arg") && Object.setPrototypeOf(BeforeSwizzle, null));
                log.push("constructed");
            }
        }

        new BeforeSwizzle();

        class BeforeThrow extends Base {
            constructor() {
                function thrower() {
                    throw new MyError();
                }
                super(thrower());
            }
        }

        Object.setPrototypeOf(BeforeThrow, Math.sin);
        try {
            new BeforeThrow();
            log.push("no throw");
        } catch (error) {
            log.push(error instanceof MyError ? "my-error" : error.constructor.name);
        }

        log.join("|");
        "#,
    );

    assert_eq!(result, "arg|constructed|my-error");
}

#[test]
fn phase6_default_derived_constructors_call_super_and_initialize_fields() {
    let result = compile_and_run(
        r"
        class Base {
            constructor() {
                this.base = 1;
            }
        }

        class Derived extends Base {
            extra = 2;
        }

        let value = new Derived();
        value.base + value.extra;
        ",
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn phase6_default_derived_constructors_forward_arguments_to_super() {
    let result = compile_and_run(
        r"
        class Base {
            constructor(value) {
                return value;
            }
        }

        class Derived extends Base {}

        let object = {};
        new Derived(object) === object;
        ",
    );

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn phase6_default_derived_constructors_do_not_iterate_rest_arguments_for_super() {
    let result = compile_and_run(
        r#"
        Array.prototype[Symbol.iterator] = function() {
            throw "iterator";
        };

        class Base {
            constructor(value) {
                this.value = value;
            }
        }

        class Derived extends Base {}

        new Derived(5).value;
        "#,
    );

    assert_eq!(result, Value::from_smi(5));
}

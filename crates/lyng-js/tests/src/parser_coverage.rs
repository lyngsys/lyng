//! Parser coverage tests — verifying all major expression, statement,
//! declaration, and pattern forms parse without errors.

use lyng_js_ast::ParsedScript;
use lyng_js_common::{AtomTable, SourceId};
use lyng_js_parser::parse_script;

fn sid() -> SourceId {
    SourceId::new(0)
}

fn ok(src: &str) -> ParsedScript {
    let mut atoms = AtomTable::new();
    let p = parse_script(&mut atoms, sid(), src);
    assert!(
        !p.diagnostics.has_errors(),
        "expected no errors for: {src}\ngot: {:?}",
        p.diagnostics.as_slice()
    );
    p
}

fn ok_module(src: &str) {
    let mut atoms = AtomTable::new();
    let p = lyng_js_parser::parse_module(&mut atoms, sid(), src);
    assert!(
        !p.diagnostics.has_errors(),
        "expected no errors for module: {src}\ngot: {:?}",
        p.diagnostics.as_slice()
    );
}

// === Expressions ===

#[test]
fn expr_this() {
    ok("this;");
}
#[test]
fn expr_null() {
    ok("null;");
}
#[test]
fn expr_true() {
    ok("true;");
}
#[test]
fn expr_false() {
    ok("false;");
}
#[test]
fn expr_number_int() {
    ok("42;");
}
#[test]
fn expr_number_float() {
    ok("3.14;");
}
#[test]
fn expr_number_hex() {
    ok("0xff;");
}
#[test]
fn expr_number_binary() {
    ok("0b1010;");
}
#[test]
fn expr_number_octal() {
    ok("0o77;");
}
#[test]
fn expr_number_sep() {
    ok("1_000_000;");
}
#[test]
fn expr_bigint() {
    ok("123n;");
}
#[test]
fn expr_string_single() {
    ok("'hello';");
}
#[test]
fn expr_string_double() {
    ok("\"world\";");
}
#[test]
fn expr_string_escape() {
    ok("'\\n\\t\\\\';");
}
#[test]
fn expr_identifier() {
    ok("foo;");
}
#[test]
fn expr_array_empty() {
    ok("[];");
}
#[test]
fn expr_array_elision() {
    ok("[1,,2,,];");
}
#[test]
fn expr_array_spread() {
    ok("[...a, ...b];");
}
#[test]
fn expr_object_empty() {
    ok("({});");
}
#[test]
fn expr_object_shorthand() {
    ok("({x, y});");
}
#[test]
fn expr_object_computed() {
    ok("({[k]: v});");
}
#[test]
fn expr_object_method() {
    ok("({f() {}});");
}
#[test]
fn expr_object_getter() {
    ok("({get x() { return 1; }});");
}
#[test]
fn expr_object_setter() {
    ok("({set x(v) {}});");
}
#[test]
fn expr_function() {
    ok("(function() {});");
}
#[test]
fn expr_function_named() {
    ok("(function f() {});");
}
#[test]
fn expr_generator() {
    ok("(function*() {});");
}
#[test]
fn expr_async_function() {
    ok("(async function() {});");
}
#[test]
fn expr_class() {
    ok("(class {});");
}
#[test]
fn expr_class_extends() {
    ok("(class extends Base {});");
}
#[test]
fn expr_template_simple() {
    ok("`hello`;");
}
#[test]
fn expr_template_sub() {
    ok("`${x}`;");
}
#[test]
fn expr_template_multi() {
    ok("`a${x}b${y}c`;");
}
#[test]
fn expr_tagged_template() {
    ok("tag`hello`;");
}
#[test]
fn expr_unary_not() {
    ok("!x;");
}
#[test]
fn expr_unary_neg() {
    ok("-x;");
}
#[test]
fn expr_unary_pos() {
    ok("+x;");
}
#[test]
fn expr_unary_bitnot() {
    ok("~x;");
}
#[test]
fn expr_typeof() {
    ok("typeof x;");
}
#[test]
fn expr_void() {
    ok("void 0;");
}
#[test]
fn expr_delete() {
    ok("delete x.y;");
}
#[test]
fn expr_prefix_inc() {
    ok("++x;");
}
#[test]
fn expr_prefix_dec() {
    ok("--x;");
}
#[test]
fn expr_postfix_inc() {
    ok("x++;");
}
#[test]
fn expr_postfix_dec() {
    ok("x--;");
}
#[test]
fn expr_add() {
    ok("a + b;");
}
#[test]
fn expr_sub() {
    ok("a - b;");
}
#[test]
fn expr_mul() {
    ok("a * b;");
}
#[test]
fn expr_div() {
    ok("a / b;");
}
#[test]
fn expr_mod() {
    ok("a % b;");
}
#[test]
fn expr_exp() {
    ok("a ** b;");
}
#[test]
fn expr_shl() {
    ok("a << b;");
}
#[test]
fn expr_shr() {
    ok("a >> b;");
}
#[test]
fn expr_ushr() {
    ok("a >>> b;");
}
#[test]
fn expr_bitand() {
    ok("a & b;");
}
#[test]
fn expr_bitor() {
    ok("a | b;");
}
#[test]
fn expr_bitxor() {
    ok("a ^ b;");
}
#[test]
fn expr_lt() {
    ok("a < b;");
}
#[test]
fn expr_gt() {
    ok("a > b;");
}
#[test]
fn expr_lte() {
    ok("a <= b;");
}
#[test]
fn expr_gte() {
    ok("a >= b;");
}
#[test]
fn expr_eq() {
    ok("a == b;");
}
#[test]
fn expr_neq() {
    ok("a != b;");
}
#[test]
fn expr_strict_eq() {
    ok("a === b;");
}
#[test]
fn expr_strict_neq() {
    ok("a !== b;");
}
#[test]
fn expr_instanceof() {
    ok("a instanceof b;");
}
#[test]
fn expr_in() {
    ok("'x' in obj;");
}
#[test]
fn expr_and() {
    ok("a && b;");
}
#[test]
fn expr_or() {
    ok("a || b;");
}
#[test]
fn expr_nullish() {
    ok("a ?? b;");
}
#[test]
fn expr_conditional() {
    ok("a ? b : c;");
}
#[test]
fn expr_assign() {
    ok("a = b;");
}
#[test]
fn expr_add_assign() {
    ok("a += b;");
}
#[test]
fn expr_sub_assign() {
    ok("a -= b;");
}
#[test]
fn expr_and_assign() {
    ok("a &&= b;");
}
#[test]
fn expr_or_assign() {
    ok("a ||= b;");
}
#[test]
fn expr_nullish_assign() {
    ok("a ??= b;");
}
#[test]
fn expr_sequence() {
    ok("a, b, c;");
}
#[test]
fn expr_call() {
    ok("f(a, b);");
}
#[test]
fn expr_new() {
    ok("new Foo(a);");
}
#[test]
fn expr_new_no_args() {
    ok("new Foo;");
}
#[test]
fn expr_member_dot() {
    ok("a.b;");
}
#[test]
fn expr_member_bracket() {
    ok("a[b];");
}
#[test]
fn expr_optional_chain() {
    ok("a?.b;");
}
#[test]
fn expr_optional_call() {
    ok("a?.(b);");
}
#[test]
fn expr_optional_bracket() {
    ok("a?.[b];");
}
#[test]
fn expr_yield() {
    ok("function* g() { yield 1; }");
}
#[test]
fn expr_yield_delegate() {
    ok("function* g() { yield* other(); }");
}
#[test]
fn expr_await() {
    ok("async function f() { await p; }");
}
#[test]
fn expr_spread_call() {
    ok("f(...args);");
}
#[test]
fn expr_import_dynamic() {
    ok("import('mod');");
}
#[test]
fn expr_paren() {
    ok("(a + b);");
}
#[test]
fn expr_comma_precedence() {
    ok("a = 1, b = 2;");
}
#[test]
fn expr_nested_ternary() {
    ok("a ? b ? c : d : e;");
}
#[test]
fn expr_chained_member() {
    ok("a.b.c.d;");
}
#[test]
fn expr_chained_call() {
    ok("f()()(x);");
}

// === Statements ===

#[test]
fn stmt_block() {
    ok("{ x; }");
}
#[test]
fn stmt_empty() {
    ok(";");
}
#[test]
fn stmt_expression() {
    ok("x;");
}
#[test]
fn stmt_if() {
    ok("if (x) y;");
}
#[test]
fn stmt_if_else() {
    ok("if (x) y; else z;");
}
#[test]
fn stmt_while() {
    ok("while (x) y;");
}
#[test]
fn stmt_do_while() {
    ok("do x; while (y);");
}
#[test]
fn stmt_for() {
    ok("for (var i = 0; i < 10; i++) x;");
}
#[test]
fn stmt_for_empty() {
    ok("for (;;) break;");
}
#[test]
fn stmt_for_in() {
    ok("for (var k in obj) x;");
}
#[test]
fn stmt_for_of() {
    ok("for (var v of arr) x;");
}
#[test]
fn stmt_for_let() {
    ok("for (let i = 0; i < 10; i++) x;");
}
#[test]
fn stmt_for_const_of() {
    ok("for (const v of arr) x;");
}
#[test]
fn stmt_switch() {
    ok("switch (x) { case 1: break; default: break; }");
}
#[test]
fn stmt_labeled() {
    ok("outer: for (;;) break outer;");
}
#[test]
fn stmt_with() {
    ok("with (obj) x;");
}
#[test]
fn stmt_try_catch() {
    ok("try { x; } catch (e) { y; }");
}
#[test]
fn stmt_try_finally() {
    ok("try { x; } finally { y; }");
}
#[test]
fn stmt_try_catch_finally() {
    ok("try { x; } catch (e) { y; } finally { z; }");
}
#[test]
fn stmt_try_catch_no_param() {
    ok("try { x; } catch { y; }");
}
#[test]
fn stmt_throw() {
    ok("throw new Error();");
}
#[test]
fn stmt_return() {
    ok("function f() { return; }");
}
#[test]
fn stmt_return_val() {
    ok("function f() { return 1; }");
}
#[test]
fn stmt_break() {
    ok("while (true) break;");
}
#[test]
fn stmt_continue() {
    ok("while (true) continue;");
}
#[test]
fn stmt_debugger() {
    ok("debugger;");
}

// === Declarations ===

#[test]
fn decl_var() {
    ok("var x;");
}
#[test]
fn decl_var_init() {
    ok("var x = 1;");
}
#[test]
fn decl_var_multi() {
    ok("var x = 1, y = 2;");
}
#[test]
fn decl_let() {
    ok("let x = 1;");
}
#[test]
fn decl_const() {
    ok("const x = 1;");
}
#[test]
fn decl_function() {
    ok("function f() {}");
}
#[test]
fn decl_function_params() {
    ok("function f(a, b, c) {}");
}
#[test]
fn decl_function_default() {
    ok("function f(a = 1) {}");
}
#[test]
fn decl_function_rest() {
    ok("function f(...args) {}");
}
#[test]
fn decl_generator() {
    ok("function* g() {}");
}
#[test]
fn decl_async() {
    ok("async function f() {}");
}
#[test]
fn decl_async_generator() {
    ok("async function* f() {}");
}
#[test]
fn decl_class() {
    ok("class C {}");
}
#[test]
fn decl_class_extends() {
    ok("class C extends Base {}");
}
#[test]
fn decl_class_method() {
    ok("class C { m() {} }");
}
#[test]
fn decl_class_static() {
    ok("class C { static m() {} }");
}
#[test]
fn decl_class_getter() {
    ok("class C { get x() { return 1; } }");
}
#[test]
fn decl_class_setter() {
    ok("class C { set x(v) {} }");
}
#[test]
fn decl_class_field() {
    ok("class C { x = 1; }");
}
#[test]
fn decl_class_static_field() {
    ok("class C { static x = 1; }");
}
#[test]
fn decl_class_computed() {
    ok("class C { [k]() {} }");
}
#[test]
fn decl_class_static_block() {
    ok("class C { static { this.x = 1; } }");
}
#[test]
fn decl_class_constructor() {
    ok("class C { constructor() {} }");
}

// === Patterns ===

#[test]
fn pat_object() {
    ok("var {a, b} = obj;");
}
#[test]
fn pat_object_rename() {
    ok("var {a: x, b: y} = obj;");
}
#[test]
fn pat_object_default() {
    ok("var {a = 1} = obj;");
}
#[test]
fn pat_object_rest() {
    ok("var {a, ...rest} = obj;");
}
#[test]
fn pat_object_computed() {
    ok("var {[k]: v} = obj;");
}
#[test]
fn pat_array() {
    ok("var [a, b] = arr;");
}
#[test]
fn pat_array_elision() {
    ok("var [,, a] = arr;");
}
#[test]
fn pat_array_rest() {
    ok("var [a, ...rest] = arr;");
}
#[test]
fn pat_array_default() {
    ok("var [a = 1] = arr;");
}
#[test]
fn pat_nested() {
    ok("var {a: [b, {c}]} = obj;");
}
#[test]
fn pat_param_destructure() {
    ok("function f({a, b}) {}");
}
#[test]
fn pat_param_array() {
    ok("function f([a, b]) {}");
}

// === Arrow functions ===

#[test]
fn arrow_expr_body() {
    ok("var f = x => x + 1;");
}
#[test]
fn arrow_block_body() {
    ok("var f = x => { return x; };");
}
#[test]
fn arrow_multi_param() {
    ok("var f = (a, b) => a + b;");
}
#[test]
fn arrow_no_param() {
    ok("var f = () => 42;");
}
#[test]
fn arrow_destructured() {
    ok("var f = ({a, b}) => a + b;");
}
#[test]
fn arrow_rest() {
    ok("var f = (...args) => args;");
}
#[test]
fn arrow_default() {
    ok("var f = (a = 1) => a;");
}
#[test]
fn arrow_nested() {
    ok("var f = x => y => x + y;");
}
#[test]
fn arrow_in_call() {
    ok("f(x => x);");
}
#[test]
fn arrow_async() {
    ok("var f = async (x) => await x;");
}

// === Modules ===

#[test]
fn mod_import_default() {
    ok_module("import x from 'mod';");
}
#[test]
fn mod_import_namespace() {
    ok_module("import * as ns from 'mod';");
}
#[test]
fn mod_import_named() {
    ok_module("import { a, b } from 'mod';");
}
#[test]
fn mod_import_renamed() {
    ok_module("import { a as b } from 'mod';");
}
#[test]
fn mod_import_side_effect() {
    ok_module("import 'mod';");
}
#[test]
fn mod_import_default_and_named() {
    ok_module("import x, { a } from 'mod';");
}
#[test]
fn mod_export_named() {
    ok_module("var x = 1; export { x };");
}
#[test]
fn mod_export_renamed() {
    ok_module("var x = 1; export { x as y };");
}
#[test]
fn mod_export_default_expr() {
    ok_module("export default 42;");
}
#[test]
fn mod_export_default_func() {
    ok_module("export default function() {}");
}
#[test]
fn mod_export_default_class() {
    ok_module("export default class {}");
}
#[test]
fn mod_export_var() {
    ok_module("export var x = 1;");
}
#[test]
fn mod_export_let() {
    ok_module("export let x = 1;");
}
#[test]
fn mod_export_const() {
    ok_module("export const x = 1;");
}
#[test]
fn mod_export_function() {
    ok_module("export function f() {}");
}
#[test]
fn mod_export_class() {
    ok_module("export class C {}");
}
#[test]
fn mod_export_all() {
    ok_module("export * from 'mod';");
}
#[test]
fn mod_export_all_as() {
    ok_module("export * as ns from 'mod';");
}
#[test]
fn mod_reexport() {
    ok_module("export { a } from 'mod';");
}

// === ASI ===

#[test]
fn asi_return() {
    ok("function f() { return\n1 }");
}
#[test]
fn asi_before_brace() {
    ok("{ 1\n}");
}
#[test]
fn asi_multiline() {
    ok("var x = 1\nvar y = 2");
}

// === Complex programs ===

#[test]
fn complex_fibonacci() {
    ok("function fib(n) { if (n <= 1) return n; return fib(n - 1) + fib(n - 2); }");
}

#[test]
fn complex_class_hierarchy() {
    ok("class Animal { constructor(name) { this.name = name; } speak() { return this.name; } } class Dog extends Animal { speak() { return super.speak() + ' barks'; } }");
}

#[test]
fn complex_async_iteration() {
    ok("async function* gen() { yield 1; yield 2; } async function main() { for await (const x of gen()) { console.log(x); } }");
}

#[test]
fn complex_destructuring_nested() {
    ok("const { a: { b: [c, { d: [e] }] }, ...rest } = obj;");
}

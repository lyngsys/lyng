//! Parser-coverage smoke check.
//!
//! Confirms that every major expression, statement, declaration, pattern,
//! and module form parses without diagnostics. Structural assertions on
//! the resulting AST live in `crates/parser/src/tests/*.rs`; this file
//! is a coarse "does it parse" net that adds new constructs cheaply.
//!
//! Each row is `(label, source)`. Failures are collected so a regression
//! is reported with its label rather than aborting on the first error.

use lyng_common::{AtomTable, SourceId};
use lyng_parser::{parse_module, parse_script};

fn parse_script_ok(label: &str, src: &str) -> Result<(), String> {
    let mut atoms = AtomTable::new();
    let p = parse_script(&mut atoms, SourceId::new(0), src);
    if p.diagnostics.has_errors() {
        return Err(format!(
            "[{label}] expected clean parse for {src:?}; diagnostics: {:?}",
            p.diagnostics.as_slice()
        ));
    }
    Ok(())
}

fn parse_module_ok(label: &str, src: &str) -> Result<(), String> {
    let mut atoms = AtomTable::new();
    let p = parse_module(&mut atoms, SourceId::new(0), src);
    if p.diagnostics.has_errors() {
        return Err(format!(
            "[{label}] expected clean module parse for {src:?}; diagnostics: {:?}",
            p.diagnostics.as_slice()
        ));
    }
    Ok(())
}

fn run_script_rows(rows: &[(&str, &str)]) {
    let failures: Vec<_> = rows
        .iter()
        .filter_map(|(label, src)| parse_script_ok(label, src).err())
        .collect();
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

fn run_module_rows(rows: &[(&str, &str)]) {
    let failures: Vec<_> = rows
        .iter()
        .filter_map(|(label, src)| parse_module_ok(label, src).err())
        .collect();
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn expressions_parse_cleanly() {
    run_script_rows(&[
        ("this", "this;"),
        ("null", "null;"),
        ("true", "true;"),
        ("false", "false;"),
        ("number_int", "42;"),
        ("number_float", "3.14;"),
        ("number_hex", "0xff;"),
        ("number_binary", "0b1010;"),
        ("number_octal", "0o77;"),
        ("number_sep", "1_000_000;"),
        ("bigint", "123n;"),
        ("string_single", "'hello';"),
        ("string_double", "\"world\";"),
        ("string_escape", "'\\n\\t\\\\';"),
        ("identifier", "foo;"),
        ("array_empty", "[];"),
        ("array_elision", "[1,,2,,];"),
        ("array_spread", "[...a, ...b];"),
        ("object_empty", "({});"),
        ("object_shorthand", "({x, y});"),
        ("object_computed", "({[k]: v});"),
        ("object_method", "({f() {}});"),
        ("object_getter", "({get x() { return 1; }});"),
        ("object_setter", "({set x(v) {}});"),
        ("function", "(function() {});"),
        ("function_named", "(function f() {});"),
        ("generator", "(function*() {});"),
        ("async_function", "(async function() {});"),
        ("class", "(class {});"),
        ("class_extends", "(class extends Base {});"),
        ("template_simple", "`hello`;"),
        ("template_sub", "`${x}`;"),
        ("template_multi", "`a${x}b${y}c`;"),
        ("tagged_template", "tag`hello`;"),
        ("unary_not", "!x;"),
        ("unary_neg", "-x;"),
        ("unary_pos", "+x;"),
        ("unary_bitnot", "~x;"),
        ("typeof", "typeof x;"),
        ("void", "void 0;"),
        ("delete", "delete x.y;"),
        ("prefix_inc", "++x;"),
        ("prefix_dec", "--x;"),
        ("postfix_inc", "x++;"),
        ("postfix_dec", "x--;"),
        ("add", "a + b;"),
        ("sub", "a - b;"),
        ("mul", "a * b;"),
        ("div", "a / b;"),
        ("mod", "a % b;"),
        ("exp", "a ** b;"),
        ("shl", "a << b;"),
        ("shr", "a >> b;"),
        ("ushr", "a >>> b;"),
        ("bitand", "a & b;"),
        ("bitor", "a | b;"),
        ("bitxor", "a ^ b;"),
        ("lt", "a < b;"),
        ("gt", "a > b;"),
        ("lte", "a <= b;"),
        ("gte", "a >= b;"),
        ("eq", "a == b;"),
        ("neq", "a != b;"),
        ("strict_eq", "a === b;"),
        ("strict_neq", "a !== b;"),
        ("instanceof", "a instanceof b;"),
        ("in", "'x' in obj;"),
        ("logical_and", "a && b;"),
        ("logical_or", "a || b;"),
        ("nullish", "a ?? b;"),
        ("conditional", "a ? b : c;"),
        ("assign", "a = b;"),
        ("add_assign", "a += b;"),
        ("sub_assign", "a -= b;"),
        ("and_assign", "a &&= b;"),
        ("or_assign", "a ||= b;"),
        ("nullish_assign", "a ??= b;"),
        ("sequence", "a, b, c;"),
        ("call", "f(a, b);"),
        ("new", "new Foo(a);"),
        ("new_no_args", "new Foo;"),
        ("member_dot", "a.b;"),
        ("member_bracket", "a[b];"),
        ("optional_chain", "a?.b;"),
        ("optional_call", "a?.(b);"),
        ("optional_bracket", "a?.[b];"),
        ("yield", "function* g() { yield 1; }"),
        ("yield_delegate", "function* g() { yield* other(); }"),
        ("await", "async function f() { await p; }"),
        ("spread_call", "f(...args);"),
        ("import_dynamic", "import('mod');"),
        ("paren", "(a + b);"),
        ("comma_precedence", "a = 1, b = 2;"),
        ("nested_ternary", "a ? b ? c : d : e;"),
        ("chained_member", "a.b.c.d;"),
        ("chained_call", "f()()(x);"),
    ]);
}

#[test]
fn statements_parse_cleanly() {
    run_script_rows(&[
        ("block", "{ x; }"),
        ("empty", ";"),
        ("expression", "x;"),
        ("if", "if (x) y;"),
        ("if_else", "if (x) y; else z;"),
        ("while", "while (x) y;"),
        ("do_while", "do x; while (y);"),
        ("for", "for (var i = 0; i < 10; i++) x;"),
        ("for_empty", "for (;;) break;"),
        ("for_in", "for (var k in obj) x;"),
        ("for_of", "for (var v of arr) x;"),
        ("for_let", "for (let i = 0; i < 10; i++) x;"),
        ("for_const_of", "for (const v of arr) x;"),
        ("switch", "switch (x) { case 1: break; default: break; }"),
        ("labeled", "outer: for (;;) break outer;"),
        ("with", "with (obj) x;"),
        ("try_catch", "try { x; } catch (e) { y; }"),
        ("try_finally", "try { x; } finally { y; }"),
        (
            "try_catch_finally",
            "try { x; } catch (e) { y; } finally { z; }",
        ),
        ("try_catch_no_param", "try { x; } catch { y; }"),
        ("throw", "throw new Error();"),
        ("return", "function f() { return; }"),
        ("return_val", "function f() { return 1; }"),
        ("break", "while (true) break;"),
        ("continue", "while (true) continue;"),
        ("debugger", "debugger;"),
    ]);
}

#[test]
fn declarations_parse_cleanly() {
    run_script_rows(&[
        ("var", "var x;"),
        ("var_init", "var x = 1;"),
        ("var_multi", "var x = 1, y = 2;"),
        ("let", "let x = 1;"),
        ("const", "const x = 1;"),
        ("function", "function f() {}"),
        ("function_params", "function f(a, b, c) {}"),
        ("function_default", "function f(a = 1) {}"),
        ("function_rest", "function f(...args) {}"),
        ("generator", "function* g() {}"),
        ("async_function", "async function f() {}"),
        ("async_generator", "async function* f() {}"),
        ("class", "class C {}"),
        ("class_extends", "class C extends Base {}"),
        ("class_method", "class C { m() {} }"),
        ("class_static", "class C { static m() {} }"),
        ("class_getter", "class C { get x() { return 1; } }"),
        ("class_setter", "class C { set x(v) {} }"),
        ("class_field", "class C { x = 1; }"),
        ("class_static_field", "class C { static x = 1; }"),
        ("class_computed", "class C { [k]() {} }"),
        ("class_static_block", "class C { static { this.x = 1; } }"),
        ("class_constructor", "class C { constructor() {} }"),
    ]);
}

#[test]
fn patterns_and_arrows_parse_cleanly() {
    run_script_rows(&[
        ("pat_object", "var {a, b} = obj;"),
        ("pat_object_rename", "var {a: x, b: y} = obj;"),
        ("pat_object_default", "var {a = 1} = obj;"),
        ("pat_object_rest", "var {a, ...rest} = obj;"),
        ("pat_object_computed", "var {[k]: v} = obj;"),
        ("pat_array", "var [a, b] = arr;"),
        ("pat_array_elision", "var [,, a] = arr;"),
        ("pat_array_rest", "var [a, ...rest] = arr;"),
        ("pat_array_default", "var [a = 1] = arr;"),
        ("pat_nested", "var {a: [b, {c}]} = obj;"),
        ("pat_param_destructure", "function f({a, b}) {}"),
        ("pat_param_array", "function f([a, b]) {}"),
        ("arrow_expr_body", "var f = x => x + 1;"),
        ("arrow_block_body", "var f = x => { return x; };"),
        ("arrow_multi_param", "var f = (a, b) => a + b;"),
        ("arrow_no_param", "var f = () => 42;"),
        ("arrow_destructured", "var f = ({a, b}) => a + b;"),
        ("arrow_rest", "var f = (...args) => args;"),
        ("arrow_default", "var f = (a = 1) => a;"),
        ("arrow_nested", "var f = x => y => x + y;"),
        ("arrow_in_call", "f(x => x);"),
        ("arrow_async", "var f = async (x) => await x;"),
    ]);
}

#[test]
fn modules_parse_cleanly() {
    run_module_rows(&[
        ("import_default", "import x from 'mod';"),
        ("import_namespace", "import * as ns from 'mod';"),
        ("import_named", "import { a, b } from 'mod';"),
        ("import_renamed", "import { a as b } from 'mod';"),
        ("import_side_effect", "import 'mod';"),
        ("import_default_and_named", "import x, { a } from 'mod';"),
        ("export_named", "var x = 1; export { x };"),
        ("export_renamed", "var x = 1; export { x as y };"),
        ("export_default_expr", "export default 42;"),
        ("export_default_func", "export default function() {}"),
        ("export_default_class", "export default class {}"),
        ("export_var", "export var x = 1;"),
        ("export_let", "export let x = 1;"),
        ("export_const", "export const x = 1;"),
        ("export_function", "export function f() {}"),
        ("export_class", "export class C {}"),
        ("export_all", "export * from 'mod';"),
        ("export_all_as", "export * as ns from 'mod';"),
        ("reexport", "export { a } from 'mod';"),
    ]);
}

#[test]
fn asi_and_complex_programs_parse_cleanly() {
    run_script_rows(&[
        ("asi_return", "function f() { return\n1 }"),
        ("asi_before_brace", "{ 1\n}"),
        ("asi_multiline", "var x = 1\nvar y = 2"),
        (
            "complex_fibonacci",
            "function fib(n) { if (n <= 1) return n; return fib(n - 1) + fib(n - 2); }",
        ),
        (
            "complex_class_hierarchy",
            "class Animal { constructor(name) { this.name = name; } speak() { return this.name; } } class Dog extends Animal { speak() { return super.speak() + ' barks'; } }",
        ),
        (
            "complex_async_iteration",
            "async function* gen() { yield 1; yield 2; } async function main() { for await (const x of gen()) { console.log(x); } }",
        ),
        (
            "complex_destructuring_nested",
            "const { a: { b: [c, { d: [e] }] }, ...rest } = obj;",
        ),
    ]);
}

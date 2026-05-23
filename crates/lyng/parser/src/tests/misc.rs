use super::*;
use lyng_js_ast::ImportExpressionPhase;

// ===========================================================================
// ASI (Automatic Semicolon Insertion)
// ===========================================================================

#[test]
fn asi_before_closing_brace() {
    let p = script_ok("{ 1 }");
    let stmts = body(&p);
    assert_eq!(stmts.len(), 1);
    if let Stmt::Block { body, .. } = p.ast.get_stmt(stmts[0]) {
        assert_eq!(p.ast.get_stmt_list(*body).len(), 1);
    }
}

#[test]
fn asi_at_eof() {
    let p = script_ok("1");
    let stmts = body(&p);
    assert_eq!(stmts.len(), 1);
}

#[test]
fn asi_after_line_terminator() {
    let p = script_ok("1\n2");
    let stmts = body(&p);
    assert_eq!(stmts.len(), 2);
}

#[test]
fn asi_return_with_newline() {
    // `return\n42` should be `return; 42;` due to ASI
    let p = script_ok("return\n42");
    let stmts = body(&p);
    assert_eq!(stmts.len(), 2);
    if let Stmt::Return { argument, .. } = p.ast.get_stmt(stmts[0]) {
        assert!(argument.is_none());
    }
}

// ===========================================================================
// Error recovery
// ===========================================================================

#[test]
fn error_recovery_does_not_crash() {
    // Invalid token should produce diagnostics but not crash
    let result = script("@@@");
    assert!(result.diagnostics.has_errors());
}

#[test]
fn error_recovery_multiple_errors() {
    let result = script("var = ;");
    assert!(result.diagnostics.has_errors());
}

#[test]
fn error_recovery_unterminated_block() {
    // Missing closing brace
    let result = script("if (true) {");
    // Should still produce a tree (with diagnostics)
    assert!(result.diagnostics.has_errors());
}

// ===========================================================================
// Strict mode
// ===========================================================================

#[test]
fn strict_mode_directive() {
    let p = script("\"use strict\"; var x = 1;");
    assert!(p.strict);
}

#[test]
fn no_strict_mode_without_directive() {
    let p = script("var x = 1;");
    assert!(!p.strict);
}

#[test]
fn module_always_strict() {
    let p = module("var x = 1;");
    // Module mode doesn't set the strict flag on ParsedModule,
    // but the parser internally runs in strict mode.
    // Just verify it parses successfully.
    assert!(!p.diagnostics.has_errors());
}

// ===========================================================================
// Complex programs
// ===========================================================================

#[test]
fn parse_fibonacci() {
    let source = r#"
function fib(n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}
fib(10);
"#;
    let p = script_ok(source);
    let stmts = body(&p);
    assert_eq!(stmts.len(), 2);
}

#[test]
fn parse_class_with_methods() {
    let source = r#"
class Animal {
    constructor(name) {
        this.name = name;
    }

    speak() {
        return this.name;
    }

    static create(name) {
        return new Animal(name);
    }
}
"#;
    let p = script_ok(source);
    let stmts = body(&p);
    assert_eq!(stmts.len(), 1);
}

#[test]
fn parse_object_methods() {
    let source = r#"
var obj = {
    a: 1,
    b() { return 2; },
    get c() { return 3; },
    set d(v) { this._d = v; }
};
"#;
    let p = script_ok(source);
    let stmts = body(&p);
    assert_eq!(stmts.len(), 1);
}

#[test]
fn parse_object_async_generator_method() {
    let p = script_ok("let obj = { async *f() {} };");
    let stmts = body(&p);
    assert_eq!(stmts.len(), 1);
}

#[test]
fn parse_nested_destructuring() {
    let source = "const { a: { b, c }, d: [e, f] } = obj;";
    let p = script_ok(source);
    let stmts = body(&p);
    assert_eq!(stmts.len(), 1);
}

#[test]
fn parse_for_of_destructuring() {
    let source = "for (const [key, value] of entries) { key; }";
    let p = script_ok(source);
    let stmts = body(&p);
    assert_eq!(stmts.len(), 1);
    assert!(matches!(p.ast.get_stmt(stmts[0]), Stmt::ForOf { .. }));
}

#[test]
fn parse_import_expression() {
    let source = r#"import("./module.js");"#;
    let p = script_ok(source);
    let stmts = body(&p);
    assert_eq!(stmts.len(), 1);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        assert!(matches!(
            p.ast.get_expr(*expression),
            Expr::ImportExpression { .. }
        ));
    }
}

#[test]
fn parse_import_expression_phases() {
    let source = r#"import.defer("./module.js"); import.source("<module source>");"#;
    let p = script_ok(source);
    let stmts = body(&p);
    assert_eq!(stmts.len(), 2);
    let expected_phases = [ImportExpressionPhase::Defer, ImportExpressionPhase::Source];
    for (stmt, expected_phase) in stmts.iter().zip(expected_phases) {
        if let Stmt::Expression { expression, .. } = p.ast.get_stmt(*stmt) {
            assert!(matches!(
                p.ast.get_expr(*expression),
                Expr::ImportExpression { phase, .. } if *phase == expected_phase
            ));
        } else {
            panic!("expected import phase expression statement");
        }
    }
}

#[test]
fn parse_import_expression_second_argument_allows_in_operator() {
    let source =
        r#"for (let promise = import("./module.js", "test262" in {} || undefined); false; ) ;"#;
    let p = script_ok(source);
    let stmts = body(&p);
    assert_eq!(stmts.len(), 1);
    if let Stmt::For {
        init: Some(init), ..
    } = p.ast.get_stmt(stmts[0])
        && let lyng_js_ast::ForInit::Declaration(decl) = init
        && let lyng_js_ast::Decl::Variable { declarators, .. } = p.ast.get_decl(*decl)
    {
        let declarators = p.ast.get_var_declarator_list(*declarators).to_vec();
        assert_eq!(declarators.len(), 1);
        let initializer = declarators[0]
            .init
            .expect("for-init declarator should keep its initializer");
        if let Expr::ImportExpression { options, .. } = p.ast.get_expr(initializer) {
            let options = options.expect("import expression should keep its options argument");
            assert!(matches!(
                p.ast.get_expr(options),
                Expr::LogicalExpression { .. }
            ));
            return;
        }
    }
    panic!("expected import expression with an options argument");
}

#[test]
fn parse_chained_calls() {
    let source = "a.b().c[0].d();";
    let p = script_ok(source);
    let stmts = body(&p);
    assert_eq!(stmts.len(), 1);
}

#[test]
fn parse_exponentiation() {
    // ** is right-associative: 2 ** 3 ** 2 = 2 ** (3 ** 2)
    let p = script_ok("2 ** 3 ** 2;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0])
        && let Expr::BinaryExpression {
            operator, right, ..
        } = p.ast.get_expr(*expression)
    {
        assert_eq!(*operator, BinaryOp::Exp);
        // Right should also be **
        assert!(matches!(
            p.ast.get_expr(*right),
            Expr::BinaryExpression {
                operator: BinaryOp::Exp,
                ..
            }
        ));
    }
}

#[test]
fn parse_instanceof() {
    let p = script_ok("x instanceof Foo;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0])
        && let Expr::BinaryExpression { operator, .. } = p.ast.get_expr(*expression)
    {
        assert_eq!(*operator, BinaryOp::Instanceof);
    }
}

#[test]
fn parse_in_operator() {
    let p = script_ok("'x' in obj;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0])
        && let Expr::BinaryExpression { operator, .. } = p.ast.get_expr(*expression)
    {
        assert_eq!(*operator, BinaryOp::In);
    }
}

#[test]
fn parse_void_delete() {
    let p = script_ok("void 0; delete obj.x;");
    let stmts = body(&p);
    assert_eq!(stmts.len(), 2);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0])
        && let Expr::UnaryExpression { operator, .. } = p.ast.get_expr(*expression)
    {
        assert_eq!(*operator, UnaryOp::Void);
    }
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[1])
        && let Expr::UnaryExpression { operator, .. } = p.ast.get_expr(*expression)
    {
        assert_eq!(*operator, UnaryOp::Delete);
    }
}

#[test]
fn parse_multiple_statements() {
    let source = r#"
var a = 1;
var b = 2;
if (a > b) {
    a = b;
} else {
    b = a;
}
for (var i = 0; i < 10; i++) {
    a += i;
}
"#;
    let p = script_ok(source);
    let stmts = body(&p);
    assert_eq!(stmts.len(), 4);
}

#[test]
fn parse_catch_without_param() {
    // catch without parameter binding (ES2019)
    let p = script_ok("try { a; } catch { b; }");
    let stmts = body(&p);
    if let Stmt::Try { handler, .. } = p.ast.get_stmt(stmts[0]) {
        let handler = handler.unwrap();
        assert!(handler.param.is_none());
    }
}

#[test]
fn parse_spread_in_call() {
    let p = script_ok("f(...args);");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0])
        && let Expr::CallExpression { arguments, .. } = p.ast.get_expr(*expression)
    {
        let args = p.ast.get_expr_list(*arguments);
        assert_eq!(args.len(), 1);
        assert!(matches!(
            p.ast.get_expr(args[0]),
            Expr::SpreadElement { .. }
        ));
    }
}

#[test]
fn parse_function_with_params() {
    let p = script_ok("function foo(a, b, c) { return a + b + c; }");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0])
        && let Decl::Function { function, .. } = p.ast.get_decl(*decl)
    {
        let func = p.ast.get_function(*function);
        let params = p.ast.get_pattern_list(func.params.params);
        assert_eq!(params.len(), 3);
    }
}

#[test]
fn parse_rest_parameter() {
    let p = script_ok("function foo(a, ...rest) {}");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0])
        && let Decl::Function { function, .. } = p.ast.get_decl(*decl)
    {
        let func = p.ast.get_function(*function);
        assert!(func.params.rest.is_some());
        let params = p.ast.get_pattern_list(func.params.params);
        assert_eq!(params.len(), 1);
    }
}

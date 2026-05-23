use super::*;

// ===========================================================================
// Statements
// ===========================================================================

#[test]
fn parse_empty_statement() {
    let p = script_ok(";");
    let stmts = body(&p);
    assert_eq!(stmts.len(), 1);
    assert!(matches!(p.ast.get_stmt(stmts[0]), Stmt::Empty { .. }));
}

#[test]
fn parse_block_statement() {
    let p = script_ok("{ 1; 2; }");
    let stmts = body(&p);
    assert_eq!(stmts.len(), 1);
    if let Stmt::Block { body, .. } = p.ast.get_stmt(stmts[0]) {
        assert_eq!(p.ast.get_stmt_list(*body).len(), 2);
    } else {
        panic!("expected block");
    }
}

#[test]
fn parse_if_statement() {
    let p = script_ok("if (true) { 1; }");
    let stmts = body(&p);
    assert_eq!(stmts.len(), 1);
    if let Stmt::If {
        alternate, test, ..
    } = p.ast.get_stmt(stmts[0])
    {
        assert!(alternate.is_none());
        assert!(matches!(
            p.ast.get_expr(*test),
            Expr::BooleanLiteral { value: true, .. }
        ));
    } else {
        panic!("expected if statement");
    }
}

#[test]
fn parse_if_else() {
    let p = script_ok("if (x) a; else b;");
    let stmts = body(&p);
    if let Stmt::If { alternate, .. } = p.ast.get_stmt(stmts[0]) {
        assert!(alternate.is_some());
    } else {
        panic!("expected if statement");
    }
}

#[test]
fn annex_b_if_clause_function_is_allowed_in_sloppy_script() {
    let p = script_ok("if (0) function f() {}");
    let stmts = body(&p);
    assert_eq!(stmts.len(), 1);
    if let Stmt::If { consequent, .. } = p.ast.get_stmt(stmts[0]) {
        let Stmt::Block { body, .. } = p.ast.get_stmt(*consequent) else {
            panic!("expected synthetic block statement");
        };
        let body = p.ast.get_stmt_list(*body);
        assert_eq!(body.len(), 1);
        assert!(matches!(p.ast.get_stmt(body[0]), Stmt::Declaration { .. }));
    } else {
        panic!("expected if statement");
    }
}

#[test]
fn function_declaration_in_if_clause_is_error_in_strict_script() {
    let p = script("\"use strict\"; if (0) function f() {}");
    assert!(p.diagnostics.has_errors());
}

#[test]
fn generator_declaration_in_if_clause_is_error_in_sloppy_script() {
    let p = script("if (0) function* f() {}");
    assert!(p.diagnostics.has_errors());
}

#[test]
fn class_declaration_in_statement_position_is_error() {
    let p = script("if (0) class C {}");
    assert!(p.diagnostics.has_errors());
}

#[test]
fn async_function_declaration_in_statement_position_is_error() {
    let p = script("if (0) async function f() {}");
    assert!(p.diagnostics.has_errors());
}

#[test]
fn annex_b_call_expression_assignment_target_is_allowed_in_sloppy_script() {
    assert!(!script("f() = g();").diagnostics.has_errors());
    assert!(!script("++f();").diagnostics.has_errors());
    assert!(!script("for (f() in obj) ;").diagnostics.has_errors());
}

#[test]
fn logical_assignment_disallows_annex_b_call_expression_target() {
    assert!(script("f() &&= 1;").diagnostics.has_errors());
    assert!(script("f() ||= 1;").diagnostics.has_errors());
    assert!(script("f() ??= 1;").diagnostics.has_errors());
}

#[test]
fn annex_b_call_expression_assignment_target_is_error_in_strict_script() {
    assert!(script("\"use strict\"; f() = g();")
        .diagnostics
        .has_errors());
    assert!(script("\"use strict\"; ++f();").diagnostics.has_errors());
}

#[test]
fn async_arrow_cover_grammar_requires_unescaped_async() {
    let p = script("\\u0061sync () => {}");
    assert!(p.diagnostics.has_errors());
}

#[test]
fn async_arrow_cover_grammar_rejects_line_terminator_before_formals() {
    let p = script("async\n(foo) => {}");
    assert!(p.diagnostics.has_errors());
}

#[test]
fn async_arrow_cover_grammar_rejects_rest_default() {
    let p = script("(async (...x = []) => {});");
    assert!(p.diagnostics.has_errors());
}

#[test]
fn async_arrow_cover_grammar_rejects_trailing_comma_after_rest() {
    let p = script("(async (...x,) => {});");
    assert!(p.diagnostics.has_errors());
}

#[test]
fn html_like_comments_are_not_enabled_for_modules() {
    let p = module("<!-- comment");
    assert!(p.diagnostics.has_errors());
}

#[test]
fn parse_while_statement() {
    let p = script_ok("while (true) { break; }");
    let stmts = body(&p);
    assert!(matches!(p.ast.get_stmt(stmts[0]), Stmt::While { .. }));
}

#[test]
fn parse_do_while_statement() {
    let p = script_ok("do { x; } while (true);");
    let stmts = body(&p);
    assert!(matches!(p.ast.get_stmt(stmts[0]), Stmt::DoWhile { .. }));
}

#[test]
fn parse_for_statement() {
    let p = script_ok("for (var i = 0; i < 10; i++) { x; }");
    let stmts = body(&p);
    assert!(matches!(p.ast.get_stmt(stmts[0]), Stmt::For { .. }));
}

#[test]
fn parse_for_in() {
    let p = script_ok("for (var x in obj) { x; }");
    let stmts = body(&p);
    assert!(matches!(p.ast.get_stmt(stmts[0]), Stmt::ForIn { .. }));
}

#[test]
fn annex_b_parse_sloppy_for_in_var_initializer() {
    let p = script_ok("for (var x = 1 in obj) { x; }");
    let stmts = body(&p);
    assert!(matches!(p.ast.get_stmt(stmts[0]), Stmt::ForIn { .. }));
}

#[test]
fn strict_for_in_var_initializer_is_error() {
    assert!(script("\"use strict\"; for (var x = 1 in obj) {}")
        .diagnostics
        .has_errors());
}

#[test]
fn parse_for_of() {
    let p = script_ok("for (var x of arr) { x; }");
    let stmts = body(&p);
    assert!(matches!(p.ast.get_stmt(stmts[0]), Stmt::ForOf { .. }));
}

#[test]
fn parse_using_declaration_statement() {
    let p = script_ok("{ using resource = acquire(); }");
    let stmts = body(&p);

    let Stmt::Block { body, .. } = p.ast.get_stmt(stmts[0]) else {
        panic!("expected block statement");
    };
    let block_stmts = p.ast.get_stmt_list(*body);
    let Stmt::Declaration { decl, .. } = p.ast.get_stmt(block_stmts[0]) else {
        panic!("expected declaration statement");
    };
    let lyng_js_ast::Decl::Variable { kind, .. } = p.ast.get_decl(*decl) else {
        panic!("expected variable declaration");
    };
    assert_eq!(*kind, lyng_js_ast::VariableKind::Using);
}

#[test]
fn top_level_using_is_error_in_script() {
    let p = script("using resource = acquire();");
    assert!(p.diagnostics.has_errors());
}

#[test]
fn parse_await_using_in_async_function() {
    let p = script_ok("async function f() { await using resource = acquire(); }");
    let stmts = body(&p);

    let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) else {
        panic!("expected async function declaration");
    };
    let lyng_js_ast::Decl::Function { function, .. } = p.ast.get_decl(*decl) else {
        panic!("expected function declaration");
    };
    let function = p.ast.get_function(*function);
    let body = p.ast.get_stmt_list(function.body);

    let Stmt::Declaration { decl, .. } = p.ast.get_stmt(body[0]) else {
        panic!("expected await using declaration");
    };
    let lyng_js_ast::Decl::Variable { kind, .. } = p.ast.get_decl(*decl) else {
        panic!("expected variable declaration");
    };
    assert_eq!(*kind, lyng_js_ast::VariableKind::AwaitUsing);
}

#[test]
fn for_in_header_rejects_using_declaration() {
    let p = script("for (using resource in obj) ;");
    assert!(p.diagnostics.has_errors());
}

#[test]
fn parse_for_statement_allows_using_of_binding_name() {
    let p = script_ok("for (using of = null;;) break;");
    let stmts = body(&p);

    assert!(matches!(p.ast.get_stmt(stmts[0]), Stmt::For { .. }));
}

#[test]
fn using_split_across_line_break_is_not_a_declaration() {
    let p = script_ok(
        r#"
        {
          using
          let = "irrelevant initializer";
          var using, let;
        }
        "#,
    );
    assert!(!p.diagnostics.has_errors());
}

#[test]
fn using_in_switch_case_is_a_parse_error() {
    let p = script(
        r#"
        switch (0) {
          case 0:
            using _ = null;
            break;
        }
        "#,
    );
    assert!(p.diagnostics.has_errors());
}

#[test]
fn await_using_does_not_break_existing_element_access() {
    let p = script_ok(
        r#"
        async function f() {
          var using = [], x = 0;
          await using[x];
        }
        "#,
    );
    assert!(!p.diagnostics.has_errors());
}

#[test]
fn using_allows_await_as_a_binding_identifier_in_static_blocks() {
    let p = script_ok(
        r#"
        class C {
          static {
            (() => { using await = null; });
          }
        }
        "#,
    );
    assert!(!p.diagnostics.has_errors());
}

#[test]
fn parse_for_await_of_with_async_identifier_lhs() {
    let p = script_ok("async function f() { for await (async of [7]) ; }");
    let stmts = body(&p);

    let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) else {
        panic!("expected async function declaration");
    };
    let lyng_js_ast::Decl::Function { function, .. } = p.ast.get_decl(*decl) else {
        panic!("expected function declaration");
    };
    let function = p.ast.get_function(*function);
    let body = p.ast.get_stmt_list(function.body);

    let Stmt::ForOf { r#await, .. } = p.ast.get_stmt(body[0]) else {
        panic!("expected for-await-of statement");
    };
    assert!(*r#await);
}

#[test]
fn parse_for_await_rejects_non_for_of_heads() {
    for source in [
        "async function* f() { for await (;;) ; }",
        "async function* f() { for await (a ;;) ; }",
        "async function* f() { for await (a in null) ; }",
        "async function* f() { for await (var a ;;) ; }",
        "async function* f() { for await (var a in null) ; }",
    ] {
        let p = script(source);
        assert!(p.diagnostics.has_errors(), "{source}");
    }
}

#[test]
fn parse_for_await_of_requires_await_context() {
    let p = script("for await (let x of []) {}");
    assert!(p.diagnostics.has_errors());
}

#[test]
fn parse_for_of_rejects_unescaped_async_identifier_lhs() {
    let p = script("var async; for (async of [1]) ;");
    assert!(p.diagnostics.has_errors());
}

#[test]
fn parse_for_of_allows_parenthesized_or_escaped_async_identifier_lhs() {
    let parenthesized = script_ok("let async; for ((async) of [7]) ;");
    assert!(!parenthesized.diagnostics.has_errors());

    let escaped = script_ok(r"let async; for (\u0061sync of [7]) ;");
    assert!(!escaped.diagnostics.has_errors());
}

#[test]
fn parse_for_empty_init() {
    let p = script_ok("for (;;) { break; }");
    let stmts = body(&p);
    if let Stmt::For {
        init, test, update, ..
    } = p.ast.get_stmt(stmts[0])
    {
        assert!(init.is_none());
        assert!(test.is_none());
        assert!(update.is_none());
    } else {
        panic!("expected for statement");
    }
}

#[test]
fn parse_switch_statement() {
    let p = script_ok("switch (x) { case 1: a; break; default: b; }");
    let stmts = body(&p);
    if let Stmt::Switch { cases, .. } = p.ast.get_stmt(stmts[0]) {
        let case_list = p.ast.get_switch_case_list(*cases);
        assert_eq!(case_list.len(), 2);
        // First case has test, second (default) doesn't
        assert!(case_list[0].test.is_some());
        assert!(case_list[1].test.is_none());
    } else {
        panic!("expected switch");
    }
}

#[test]
fn parse_try_catch() {
    let p = script_ok("try { a; } catch (e) { b; }");
    let stmts = body(&p);
    if let Stmt::Try {
        handler, finalizer, ..
    } = p.ast.get_stmt(stmts[0])
    {
        assert!(handler.is_some());
        assert!(finalizer.is_none());
    } else {
        panic!("expected try");
    }
}

#[test]
fn parse_try_catch_finally() {
    let p = script_ok("try { a; } catch (e) { b; } finally { c; }");
    let stmts = body(&p);
    if let Stmt::Try {
        handler, finalizer, ..
    } = p.ast.get_stmt(stmts[0])
    {
        assert!(handler.is_some());
        assert!(finalizer.is_some());
    } else {
        panic!("expected try");
    }
}

#[test]
fn parse_try_finally_no_catch() {
    let p = script_ok("try { a; } finally { b; }");
    let stmts = body(&p);
    if let Stmt::Try {
        handler, finalizer, ..
    } = p.ast.get_stmt(stmts[0])
    {
        assert!(handler.is_none());
        assert!(finalizer.is_some());
    } else {
        panic!("expected try");
    }
}

#[test]
fn parse_return_statement() {
    let p = script_ok("return;");
    let stmts = body(&p);
    if let Stmt::Return { argument, .. } = p.ast.get_stmt(stmts[0]) {
        assert!(argument.is_none());
    } else {
        panic!("expected return");
    }
}

#[test]
fn parse_return_with_value() {
    let p = script_ok("return 42;");
    let stmts = body(&p);
    if let Stmt::Return { argument, .. } = p.ast.get_stmt(stmts[0]) {
        assert!(argument.is_some());
    } else {
        panic!("expected return");
    }
}

#[test]
fn parse_throw_statement() {
    let p = script_ok("throw new Error();");
    let stmts = body(&p);
    assert!(matches!(p.ast.get_stmt(stmts[0]), Stmt::Throw { .. }));
}

#[test]
fn parse_break_statement() {
    let p = script_ok("break;");
    let stmts = body(&p);
    if let Stmt::Break { label, .. } = p.ast.get_stmt(stmts[0]) {
        assert!(label.is_none());
    } else {
        panic!("expected break");
    }
}

#[test]
fn parse_continue_statement() {
    let p = script_ok("continue;");
    let stmts = body(&p);
    assert!(matches!(p.ast.get_stmt(stmts[0]), Stmt::Continue { .. }));
}

#[test]
fn parse_debugger_statement() {
    let p = script_ok("debugger;");
    let stmts = body(&p);
    assert!(matches!(p.ast.get_stmt(stmts[0]), Stmt::Debugger { .. }));
}

#[test]
fn parse_labeled_statement() {
    let p = script_ok("loop: while (true) { break loop; }");
    let stmts = body(&p);
    assert!(matches!(p.ast.get_stmt(stmts[0]), Stmt::Labeled { .. }));
}

#[test]
fn annex_b_parse_sloppy_labeled_function_declaration() {
    let p = script_ok("label: function f() {}");
    let stmts = body(&p);
    assert!(matches!(p.ast.get_stmt(stmts[0]), Stmt::Labeled { .. }));
}

#[test]
fn annex_b_labeled_function_is_error_in_if_clause() {
    assert!(script("if (false) label1: label2: function f() {}")
        .diagnostics
        .has_errors());
}

#[test]
fn annex_b_labeled_function_is_error_in_loop_body() {
    assert!(script("while (false) label1: label2: function f() {}")
        .diagnostics
        .has_errors());
}

#[test]
fn annex_b_labeled_function_is_error_in_with_body() {
    assert!(script("with (obj) label: function f() {}")
        .diagnostics
        .has_errors());
}

#[test]
fn generator_declaration_under_label_is_error() {
    assert!(script("label: function* g() {}").diagnostics.has_errors());
}

#[test]
fn strict_labeled_function_declaration_is_error() {
    assert!(script("\"use strict\"; label: function f() {}")
        .diagnostics
        .has_errors());
}

#[test]
fn parse_with_statement() {
    let p = script_ok("with (obj) { x; }");
    let stmts = body(&p);
    assert!(matches!(p.ast.get_stmt(stmts[0]), Stmt::With { .. }));
}

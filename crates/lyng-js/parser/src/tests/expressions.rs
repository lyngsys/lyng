use super::*;

// ===========================================================================
// Basic expressions
// ===========================================================================

#[test]
fn parse_empty_script() {
    let p = script_ok("");
    assert!(body(&p).is_empty());
}

#[test]
fn parse_this_expression() {
    let p = script_ok("this;");
    let stmts = body(&p);
    assert_eq!(stmts.len(), 1);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        assert!(matches!(p.ast.get_expr(*expression), Expr::This { .. }));
    } else {
        panic!("expected expression statement");
    }
}

#[test]
fn parse_null_literal() {
    let p = script_ok("null;");
    let stmts = body(&p);
    assert_eq!(stmts.len(), 1);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        assert!(matches!(
            p.ast.get_expr(*expression),
            Expr::NullLiteral { .. }
        ));
    } else {
        panic!("expected expression statement");
    }
}

#[test]
fn parse_boolean_literals() {
    let p = script_ok("true; false;");
    let stmts = body(&p);
    assert_eq!(stmts.len(), 2);

    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::BooleanLiteral { value, .. } = p.ast.get_expr(*expression) {
            assert!(*value);
        } else {
            panic!("expected boolean literal");
        }
    }

    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[1]) {
        if let Expr::BooleanLiteral { value, .. } = p.ast.get_expr(*expression) {
            assert!(!*value);
        } else {
            panic!("expected boolean literal");
        }
    }
}

#[test]
fn parse_numeric_literal() {
    let p = script_ok("42;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::NumericLiteral { value, .. } = p.ast.get_expr(*expression) {
            assert_eq!(*value, NumericLiteral::Int32(42));
        } else {
            panic!("expected numeric literal");
        }
    }
}

#[test]
fn parse_float_literal() {
    let p = script_ok("2.5;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::NumericLiteral { value, .. } = p.ast.get_expr(*expression) {
            assert!(matches!(value, NumericLiteral::Number(f) if (*f - 2.5).abs() < 0.001));
        } else {
            panic!("expected numeric literal");
        }
    }
}

#[test]
fn parse_string_literal() {
    let p = script_ok(r#""hello";"#);
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::StringLiteral { value, .. } = p.ast.get_expr(*expression) {
            assert_eq!(p.ast.literals().get_string(*value), "hello");
        } else {
            panic!("expected string literal");
        }
    }
}

#[test]
fn parse_annex_b_legacy_regexp_escapes() {
    assert!(!script("/\\x/;").diagnostics.has_errors());
    assert!(!script("/\\u/;").diagnostics.has_errors());
    assert!(!script("/\\k<a>/;").diagnostics.has_errors());
    assert!(!script("/\\c_/;").diagnostics.has_errors());
    assert!(!script("/[\\c_]/;").diagnostics.has_errors());
    assert!(!script("/\\c\u{0410}/;").diagnostics.has_errors());
}

#[test]
fn regexp_named_group_reference_requires_matching_group() {
    assert!(script("/(?<a>.)\\k<b>/;").diagnostics.has_errors());
    assert!(script("/(?<a>.)\\k/;").diagnostics.has_errors());
}

#[test]
fn regexp_named_group_names_accept_unicode_identifier_forms() {
    assert!(!script("/(?<狗>x)\\k<狗>/;").diagnostics.has_errors());
    assert!(!script("/(?<\\u{72d7}>x)\\k<狗>/u;")
        .diagnostics
        .has_errors());
}

#[test]
fn regexp_duplicate_named_groups_are_allowed_across_alternatives() {
    assert!(!script("/(?<x>a)|(?<x>b)/;").diagnostics.has_errors());
    assert!(!script("/(?:(?<x>a)|(?<x>b))\\k<x>/;")
        .diagnostics
        .has_errors());
}

#[test]
fn regexp_duplicate_named_groups_are_rejected_in_the_same_alternative() {
    assert!(script("/(?<x>a)(?<x>b)/;").diagnostics.has_errors());
    assert!(script("/(?<x>a(?<x>b)|c)/;").diagnostics.has_errors());
}

#[test]
fn invalid_unicode_property_escape_is_rejected() {
    assert!(script("/\\p{Line_Break}/u;").diagnostics.has_errors());
}

#[test]
fn valid_unicode_sets_string_property_escape_is_accepted() {
    assert!(!script("/\\p{Basic_Emoji}/v;").diagnostics.has_errors());
    assert!(script("/\\P{Basic_Emoji}/v;").diagnostics.has_errors());
    assert!(script("/[^\\p{Basic_Emoji}]/v;").diagnostics.has_errors());
}

#[test]
fn regexp_unicode_and_unicode_sets_flags_are_mutually_exclusive() {
    assert!(script("/./uv;").diagnostics.has_errors());
    assert!(script("/./vu;").diagnostics.has_errors());
}

#[test]
fn regexp_modifier_groups_are_accepted_and_validated() {
    let valid = [
        "/(?i:a)b/;",
        "/(?i-s:a)/;",
        "/(?-i:a)/i;",
        "/(?i-:a)/;",
        "/(?ims:a)/;",
    ];
    for case in valid {
        let parsed = script(case);
        assert!(
            !parsed.diagnostics.has_errors(),
            "unexpected regexp parse error for {case}: {:?}",
            parsed.diagnostics.as_slice()
        );
    }

    for case in ["/(?a:a)/;", "/(?ii:a)/;", "/(?i-i:a)/;", "/(?-:a)/;"] {
        assert!(
            script(case).diagnostics.has_errors(),
            "expected regexp parse error for {case}"
        );
    }
}

#[test]
fn regexp_unicode_sets_rejects_unescaped_class_set_reserved_syntax() {
    let cases = [
        "/[(]/v;",
        "/[)]/v;",
        "/[[]/v;",
        "/[{]/v;",
        "/[}]/v;",
        "/[/]/v;",
        "/[-]/v;",
        "/[|]/v;",
        "/[&&]/v;",
        "/[!!]/v;",
        "/[##]/v;",
        "/[$$]/v;",
        "/[%%]/v;",
        "/[**]/v;",
        "/[++]/v;",
        "/[,,]/v;",
        "/[..]/v;",
        "/[::]/v;",
        "/[;;]/v;",
        "/[<<]/v;",
        "/[==]/v;",
        "/[>>]/v;",
        "/[??]/v;",
        "/[@@]/v;",
        "/[``]/v;",
        "/[~~]/v;",
        "/[^^^]/v;",
        "/[_^^]/v;",
    ];

    for case in cases {
        assert!(
            script(case).diagnostics.has_errors(),
            "expected regexp parse error for {case}"
        );
    }
}

#[test]
fn regexp_unicode_sets_accepts_generated_class_set_expression_syntax() {
    let cases = [
        r"/^[[0-9]&&_]+$/v;",
        r"/^[\p{ASCII_Hex_Digit}--_]+$/v;",
        r"/^[\q{0|2|4|9\uFE0F\u20E3}\q{0|2|4|9\uFE0F\u20E3}]+$/v;",
        r"/^[\p{Basic_Emoji}--_]+$/v;",
    ];

    for case in cases {
        let parsed = script(case);
        assert!(
            !parsed.diagnostics.has_errors(),
            "unexpected regexp parse error for {case}: {:?}",
            parsed.diagnostics.as_slice()
        );
    }
}

#[test]
fn parse_cover_call_expression_named_async_without_arrow() {
    assert!(!script("async() = 1;").diagnostics.has_errors());
    assert!(!script("for (async() of xs) ;").diagnostics.has_errors());
}

#[test]
fn parse_empty_character_class_regexp() {
    assert!(!script("/[]/;").diagnostics.has_errors());
}

#[test]
fn parse_string_split_separator_regexp_regression_literals() {
    let cases = [
        "/^/;",
        "/$/;",
        "/.?/;",
        "/.*/;",
        "/.+/;",
        "/.*?/;",
        "/.{1}/;",
        "/.{1,}/;",
        "/.{1,2}/;",
        "/()/;",
        "/./;",
        "/(?:)/;",
        "/(...)/;",
        "/(|)/;",
        "/[]/;",
        "/[^]/;",
        "/[.-.]/;",
        "/\\0/;",
        "/\\b/;",
        "/\\B/;",
        "/\\d/;",
        "/\\D/;",
        "/\\n/;",
        "/\\r/;",
        "/\\s/;",
        "/\\S/;",
        "/\\v/;",
        "/\\w/;",
        "/\\W/;",
        "/\\k<x>/;",
        "/\\xA0/;",
        "/\\XA0/;",
        "/\\ddd/;",
        "/\\cY/;",
        "/[\\b]/;",
        "/\\x/;",
        "/\\X/;",
    ];

    for case in cases {
        let parsed = script(case);
        assert!(
            !parsed.diagnostics.has_errors(),
            "unexpected regexp parse errors for {case:?}: {:?}",
            parsed.diagnostics.as_slice()
        );
    }
}

#[test]
fn parse_identifier() {
    let p = script_ok("foo;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        assert!(matches!(
            p.ast.get_expr(*expression),
            Expr::Identifier { .. }
        ));
    }
}

// ===========================================================================
// Binary operations
// ===========================================================================

#[test]
fn parse_addition() {
    let p = script_ok("1 + 2;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::BinaryExpression { operator, .. } = p.ast.get_expr(*expression) {
            assert_eq!(*operator, BinaryOp::Add);
        } else {
            panic!("expected binary expression");
        }
    }
}

#[test]
fn parse_binary_precedence() {
    // `1 + 2 * 3` should parse as `1 + (2 * 3)`
    let p = script_ok("1 + 2 * 3;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::BinaryExpression {
            operator, right, ..
        } = p.ast.get_expr(*expression)
        {
            assert_eq!(*operator, BinaryOp::Add);
            assert!(matches!(
                p.ast.get_expr(*right),
                Expr::BinaryExpression {
                    operator: BinaryOp::Mul,
                    ..
                }
            ));
        } else {
            panic!("expected binary expression");
        }
    }
}

#[test]
fn parse_comparison() {
    let p = script_ok("a === b;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::BinaryExpression { operator, .. } = p.ast.get_expr(*expression) {
            assert_eq!(*operator, BinaryOp::StrictEq);
        } else {
            panic!("expected binary expression");
        }
    }
}

#[test]
fn parse_logical_and() {
    let p = script_ok("a && b;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::LogicalExpression { operator, .. } = p.ast.get_expr(*expression) {
            assert_eq!(*operator, LogicalOp::And);
        } else {
            panic!("expected logical expression");
        }
    }
}

#[test]
fn parse_logical_or() {
    let p = script_ok("a || b;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::LogicalExpression { operator, .. } = p.ast.get_expr(*expression) {
            assert_eq!(*operator, LogicalOp::Or);
        } else {
            panic!("expected logical expression");
        }
    }
}

#[test]
fn parse_nullish_coalescing() {
    let p = script_ok("a ?? b;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::LogicalExpression { operator, .. } = p.ast.get_expr(*expression) {
            assert_eq!(*operator, LogicalOp::NullishCoalescing);
        } else {
            panic!("expected logical expression");
        }
    }
}

// ===========================================================================
// Unary operations
// ===========================================================================

#[test]
fn parse_typeof() {
    let p = script_ok("typeof x;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::UnaryExpression { operator, .. } = p.ast.get_expr(*expression) {
            assert_eq!(*operator, UnaryOp::TypeOf);
        } else {
            panic!("expected unary expression");
        }
    }
}

#[test]
fn parse_negation() {
    let p = script_ok("-x;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::UnaryExpression { operator, .. } = p.ast.get_expr(*expression) {
            assert_eq!(*operator, UnaryOp::Minus);
        } else {
            panic!("expected unary expression");
        }
    }
}

#[test]
fn parse_not() {
    let p = script_ok("!x;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::UnaryExpression { operator, .. } = p.ast.get_expr(*expression) {
            assert_eq!(*operator, UnaryOp::Not);
        } else {
            panic!("expected unary expression");
        }
    }
}

#[test]
fn parse_prefix_increment() {
    let p = script_ok("++x;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::UpdateExpression {
            operator, prefix, ..
        } = p.ast.get_expr(*expression)
        {
            assert_eq!(*operator, UpdateOp::Increment);
            assert!(*prefix);
        } else {
            panic!("expected update expression");
        }
    }
}

#[test]
fn parse_postfix_decrement() {
    let p = script_ok("x--;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::UpdateExpression {
            operator, prefix, ..
        } = p.ast.get_expr(*expression)
        {
            assert_eq!(*operator, UpdateOp::Decrement);
            assert!(!*prefix);
        } else {
            panic!("expected update expression");
        }
    }
}

// ===========================================================================
// Member access and calls
// ===========================================================================

#[test]
fn parse_dot_member() {
    let p = script_ok("a.b;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        assert!(matches!(
            p.ast.get_expr(*expression),
            Expr::StaticMemberExpression { .. }
        ));
    }
}

#[test]
fn parse_computed_member() {
    let p = script_ok("a[b];");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        assert!(matches!(
            p.ast.get_expr(*expression),
            Expr::ComputedMemberExpression { .. }
        ));
    }
}

#[test]
fn parse_function_call() {
    let p = script_ok("f(1, 2, 3);");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::CallExpression { arguments, .. } = p.ast.get_expr(*expression) {
            assert_eq!(p.ast.get_expr_list(*arguments).len(), 3);
        } else {
            panic!("expected call expression");
        }
    }
}

#[test]
fn parse_new_expression() {
    let p = script_ok("new Foo(1);");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        assert!(matches!(
            p.ast.get_expr(*expression),
            Expr::NewExpression { .. }
        ));
    }
}

#[test]
fn parse_conditional_expression() {
    let p = script_ok("a ? b : c;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        assert!(matches!(
            p.ast.get_expr(*expression),
            Expr::ConditionalExpression { .. }
        ));
    }
}

#[test]
fn parse_assignment() {
    let p = script_ok("x = 42;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::AssignmentExpression { operator, .. } = p.ast.get_expr(*expression) {
            assert_eq!(*operator, AssignOp::Assign);
        } else {
            panic!("expected assignment expression");
        }
    }
}

#[test]
fn parse_compound_assignment() {
    let p = script_ok("x += 1;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::AssignmentExpression { operator, .. } = p.ast.get_expr(*expression) {
            assert_eq!(*operator, AssignOp::AddAssign);
        } else {
            panic!("expected assignment expression");
        }
    }
}

#[test]
fn parse_sequence_expression() {
    let p = script_ok("a, b, c;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::SequenceExpression { expressions, .. } = p.ast.get_expr(*expression) {
            assert_eq!(p.ast.get_expr_list(*expressions).len(), 3);
        } else {
            panic!("expected sequence expression");
        }
    }
}

// ===========================================================================
// Array and Object expressions
// ===========================================================================

#[test]
fn parse_array_expression() {
    let p = script_ok("[1, 2, 3];");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::ArrayExpression { elements, .. } = p.ast.get_expr(*expression) {
            assert_eq!(p.ast.get_opt_expr_list(*elements).len(), 3);
        } else {
            panic!("expected array expression");
        }
    }
}

#[test]
fn parse_array_with_elision() {
    let p = script_ok("[1, , 3];");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::ArrayExpression { elements, .. } = p.ast.get_expr(*expression) {
            let elems = p.ast.get_opt_expr_list(*elements);
            assert_eq!(elems.len(), 3);
            assert!(elems[0].is_some());
            assert!(elems[1].is_none());
            assert!(elems[2].is_some());
        } else {
            panic!("expected array expression");
        }
    }
}

#[test]
fn parse_array_trailing_comma_after_spread_is_not_elision() {
    let p = script_ok("[...items,];");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::ArrayExpression { elements, .. } = p.ast.get_expr(*expression) {
            let elems = p.ast.get_opt_expr_list(*elements);
            assert_eq!(elems.len(), 1);
            assert!(elems[0].is_some());
        } else {
            panic!("expected array expression");
        }
    }
}

#[test]
fn parse_object_expression() {
    let p = script_ok("({ a: 1, b: 2 });");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        // Unwrap parenthesized expression
        let inner =
            if let Expr::ParenthesizedExpression { expression, .. } = p.ast.get_expr(*expression) {
                *expression
            } else {
                *expression
            };

        if let Expr::ObjectExpression { properties, .. } = p.ast.get_expr(inner) {
            assert_eq!(p.ast.get_property_list(*properties).len(), 2);
        } else {
            panic!(
                "expected object expression, got {:?}",
                p.ast.get_expr(inner)
            );
        }
    }
}

#[test]
fn parse_spread_in_array() {
    let p = script_ok("[...a, 1];");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        if let Expr::ArrayExpression { elements, .. } = p.ast.get_expr(*expression) {
            let elems = p.ast.get_opt_expr_list(*elements);
            assert_eq!(elems.len(), 2);
            if let Some(first) = elems[0] {
                assert!(matches!(p.ast.get_expr(first), Expr::SpreadElement { .. }));
            }
        } else {
            panic!("expected array expression");
        }
    }
}

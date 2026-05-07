use super::*;

// ===========================================================================
// Patterns (destructuring)
// ===========================================================================

#[test]
fn parse_object_destructuring() {
    let p = script_ok("const { a, b: c } = obj;");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0])
        && let Decl::Variable { declarators, .. } = p.ast.get_decl(*decl)
    {
        let decls = p.ast.get_var_declarator_list(*declarators);
        assert_eq!(decls.len(), 1);
        let pat = p.ast.get_pattern(decls[0].id);
        assert!(matches!(pat, Pattern::Object { .. }));
        if let Pattern::Object { properties, .. } = pat {
            let props = p.ast.get_obj_pattern_prop_list(*properties);
            assert_eq!(props.len(), 2);
            assert!(props[0].shorthand);
            assert!(!props[1].shorthand);
        }
    }
}

#[test]
fn parse_array_destructuring() {
    let p = script_ok("const [a, , b] = arr;");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0])
        && let Decl::Variable { declarators, .. } = p.ast.get_decl(*decl)
    {
        let decls = p.ast.get_var_declarator_list(*declarators);
        let pat = p.ast.get_pattern(decls[0].id);
        if let Pattern::Array { elements, .. } = pat {
            let elems = p.ast.get_opt_pattern_elem_list(*elements);
            assert_eq!(elems.len(), 3);
            assert!(elems[0].is_some());
            assert!(elems[1].is_none());
            assert!(elems[2].is_some());
        } else {
            panic!("expected array pattern");
        }
    }
}

#[test]
fn parse_destructuring_with_default() {
    let p = script_ok("const { a = 1 } = obj;");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0])
        && let Decl::Variable { declarators, .. } = p.ast.get_decl(*decl)
    {
        let decls = p.ast.get_var_declarator_list(*declarators);
        let pat = p.ast.get_pattern(decls[0].id);
        if let Pattern::Object { properties, .. } = pat {
            let props = p.ast.get_obj_pattern_prop_list(*properties);
            assert_eq!(props.len(), 1);
            // The value pattern should be an Assignment pattern with default
            let val_pat = p.ast.get_pattern(props[0].value);
            assert!(
                matches!(val_pat, Pattern::Assignment { .. }),
                "expected assignment pattern, got {val_pat:?}"
            );
        }
    }
}

#[test]
fn parse_rest_in_destructuring() {
    let p = script_ok("const [a, ...b] = arr;");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0])
        && let Decl::Variable { declarators, .. } = p.ast.get_decl(*decl)
    {
        let decls = p.ast.get_var_declarator_list(*declarators);
        if let Pattern::Array { rest, elements, .. } = p.ast.get_pattern(decls[0].id) {
            assert!(rest.is_some());
            let elems = p.ast.get_opt_pattern_elem_list(*elements);
            assert_eq!(elems.len(), 1);
        } else {
            panic!("expected array pattern");
        }
    }
}

// ===========================================================================
// Arrow functions
// ===========================================================================

#[test]
fn parse_arrow_single_param() {
    let p = script_ok("x => x;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        assert!(matches!(
            p.ast.get_expr(*expression),
            Expr::ArrowFunctionExpression { .. }
        ));
    }
}

#[test]
fn parse_arrow_multi_param() {
    let p = script_ok("(a, b) => a + b;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0])
        && let Expr::ArrowFunctionExpression { function, .. } = p.ast.get_expr(*expression)
    {
        let func = p.ast.get_function(*function);
        let params = p.ast.get_pattern_list(func.params.params);
        assert_eq!(params.len(), 2);
        assert!(func.expression_body.is_some());
    }
}

#[test]
fn parse_arrow_block_body() {
    let p = script_ok("(x) => { return x; };");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0])
        && let Expr::ArrowFunctionExpression { function, .. } = p.ast.get_expr(*expression)
    {
        let func = p.ast.get_function(*function);
        assert!(func.expression_body.is_none());
        let body = p.ast.get_stmt_list(func.body);
        assert_eq!(body.len(), 1);
    }
}

#[test]
fn parse_arrow_no_params() {
    let p = script_ok("() => 42;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0])
        && let Expr::ArrowFunctionExpression { function, .. } = p.ast.get_expr(*expression)
    {
        let func = p.ast.get_function(*function);
        assert!(p.ast.get_pattern_list(func.params.params).is_empty());
        assert!(func.expression_body.is_some());
    }
}

#[test]
fn parse_parenthesized_expression() {
    let p = script_ok("(1 + 2);");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        assert!(matches!(
            p.ast.get_expr(*expression),
            Expr::ParenthesizedExpression { .. }
        ));
    }
}

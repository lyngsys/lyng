use super::*;

// ===========================================================================
// Function expressions
// ===========================================================================

#[test]
fn parse_function_expression() {
    let p = script_ok("var f = function() {};");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0])
        && let Decl::Variable { declarators, .. } = p.ast.get_decl(*decl)
    {
        let decls = p.ast.get_var_declarator_list(*declarators);
        if let Some(init) = decls[0].init {
            assert!(matches!(
                p.ast.get_expr(init),
                Expr::FunctionExpression { .. }
            ));
        }
    }
}

#[test]
fn parse_named_function_expression() {
    let p = script_ok("var f = function foo() {};");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0])
        && let Decl::Variable { declarators, .. } = p.ast.get_decl(*decl)
    {
        let decls = p.ast.get_var_declarator_list(*declarators);
        if let Some(init) = decls[0].init
            && let Expr::FunctionExpression { function, .. } = p.ast.get_expr(init)
        {
            assert!(p.ast.get_function(*function).name.is_some());
        }
    }
}

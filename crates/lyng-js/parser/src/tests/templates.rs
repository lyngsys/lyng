use super::*;

// ===========================================================================
// Templates
// ===========================================================================

#[test]
fn parse_simple_template() {
    let p = script_ok("`hello`;");
    let stmts = body(&p);
    if let Stmt::Expression { expression, .. } = p.ast.get_stmt(stmts[0]) {
        assert!(matches!(
            p.ast.get_expr(*expression),
            Expr::TemplateLiteral { .. }
        ));
    }
}

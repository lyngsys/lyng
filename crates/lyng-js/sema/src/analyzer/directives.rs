use lyng_js_ast::{Expr, Stmt};
use lyng_js_common::Span;

use super::{Analyzer, ContainmentQuery};

impl<'a> Analyzer<'a> {
    pub(super) fn check_global_code_contains(
        &mut self,
        body: lyng_js_ast::NodeList<lyng_js_ast::StmtId>,
    ) {
        let stmts = self.ast.get_stmt_list(body);
        let Some(&first_stmt) = stmts.first() else {
            return;
        };
        let span = self.ast.get_stmt(first_stmt).span();
        if self.stmt_list_contains_query(body, ContainmentQuery::NewTarget) {
            self.diagnostics
                .error(span, "'new.target' outside of a function");
        }
        if self.stmt_list_contains_query(body, ContainmentQuery::SuperKeyword) {
            self.diagnostics
                .error(span, "'super' keyword outside of a method");
        }
    }

    pub(super) fn apply_directive_prologue(
        &mut self,
        body: lyng_js_ast::NodeList<lyng_js_ast::StmtId>,
    ) {
        let (strict_span, legacy_escape_span) = self.strict_directive_info(body);
        if let Some(span) = legacy_escape_span {
            self.diagnostics.error(
                span,
                "legacy octal escape not allowed before a 'use strict' directive",
            );
        }
        if strict_span.is_some() {
            self.ctx.strict = true;
            let scope = self.ctx.current_scope;
            self.scopes.get_mut(scope).strict = true;
        }
    }

    fn strict_directive_info(
        &self,
        body: lyng_js_ast::NodeList<lyng_js_ast::StmtId>,
    ) -> (Option<Span>, Option<Span>) {
        let stmts = self.ast.get_stmt_list(body);
        let mut legacy_escape_span = None;
        for &stmt_id in stmts {
            let stmt = self.ast.get_stmt(stmt_id);
            match stmt {
                Stmt::Expression { expression, .. } => {
                    let expr = self.ast.get_expr(*expression);
                    if let Expr::StringLiteral { value, syntax, .. } = expr {
                        if syntax.contains_legacy_octal_escape && legacy_escape_span.is_none() {
                            legacy_escape_span = Some(expr.span());
                        }
                        let s = self.ast.literals().get_string(*value);
                        if s == "use strict" {
                            return (Some(stmt.span()), legacy_escape_span);
                        }
                        continue;
                    }
                    break;
                }
                _ => break,
            }
        }
        (None, None)
    }

    pub(super) fn strict_directive_span(
        &self,
        body: lyng_js_ast::NodeList<lyng_js_ast::StmtId>,
    ) -> Option<Span> {
        self.strict_directive_info(body).0
    }
}

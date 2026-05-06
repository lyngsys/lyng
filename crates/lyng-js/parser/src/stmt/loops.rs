use lyng_js_ast::{Decl, Expr, ExprId, ForInOfLeft, ForInit, Stmt, StmtId, VariableKind};
use lyng_js_common::{Span, WellKnownAtom};
use lyng_js_lexer::TokenKind;

use crate::parser::Parser;

impl<'src, 'atoms> Parser<'src, 'atoms> {
    pub(super) fn parse_while_statement(&mut self) -> StmtId {
        let start = self.current_span();
        self.advance();
        self.expect(TokenKind::LParen);
        let test = self.parse_expression();
        self.expect(TokenKind::RParen);

        let prev_iter = self.in_iteration();
        self.set_in_iteration(true);
        let body = self.parse_statement_rejecting_labelled_function();
        self.set_in_iteration(prev_iter);

        let body_span = self.ast().get_stmt(body).span();
        let span = start.cover(body_span);
        self.ast_mut().alloc_stmt(Stmt::While { span, test, body })
    }

    pub(super) fn parse_do_while_statement(&mut self) -> StmtId {
        let start = self.current_span();
        self.advance();

        let prev_iter = self.in_iteration();
        self.set_in_iteration(true);
        let body = self.parse_statement_rejecting_labelled_function();
        self.set_in_iteration(prev_iter);

        self.expect(TokenKind::While);
        self.expect(TokenKind::LParen);
        let test = self.parse_expression();
        let end = self.expect(TokenKind::RParen);
        self.eat(TokenKind::Semicolon);
        let span = start.cover(end.span);
        self.ast_mut()
            .alloc_stmt(Stmt::DoWhile { span, body, test })
    }

    pub(super) fn parse_for_statement(&mut self) -> StmtId {
        let start = self.current_span();
        self.advance();

        let is_await = self.at(TokenKind::Await) || self.at_contextual(WellKnownAtom::r#await);
        if is_await {
            if !self.allow_await && !self.is_module() {
                self.error_at(
                    self.current_span(),
                    "`for await` is only allowed in async functions and modules".to_string(),
                );
            }
            self.advance();
        }

        self.expect(TokenKind::LParen);

        if self.at(TokenKind::Semicolon) {
            if is_await {
                self.error_at(
                    self.current_span(),
                    "`for await` requires a for-of head".to_string(),
                );
            }
            self.advance();
            return self.parse_for_rest(start, None);
        }

        let using_for_of_ambiguity = self.at_using_declaration()
            && matches!(
                self.peek().payload,
                lyng_js_lexer::TokenPayload::Atom(id) if id == WellKnownAtom::of.id()
            )
            && !matches!(
                self.peek_second().kind,
                TokenKind::Eq | TokenKind::Comma | TokenKind::Semicolon
            );

        if self.at(TokenKind::Var)
            || self.at(TokenKind::Const)
            || self.at_let_declaration()
            || (self.at_using_declaration() && !using_for_of_ambiguity)
            || self.at_await_using_declaration()
        {
            return self.parse_for_with_declaration(start, is_await);
        }

        let prev_no_in = self.no_in();
        self.set_no_in(true);
        let expr = self.parse_expression();
        self.set_no_in(prev_no_in);

        if self.at(TokenKind::In) {
            if is_await {
                self.error_at(
                    self.current_span(),
                    "`for await` requires a for-of head".to_string(),
                );
            }
            if self.is_destructuring_pattern_expression(expr) {
                self.validate_pattern_expression(expr, false, true);
            } else if !self.is_simple_assignment_target(expr) {
                let span = self.ast().get_expr(expr).span();
                self.error_at(span, "invalid assignment target".to_string());
            }
            self.advance();
            let right = self.parse_expression();
            self.expect(TokenKind::RParen);

            let prev_iter = self.in_iteration();
            self.set_in_iteration(true);
            let body = self.parse_statement_rejecting_labelled_function();
            self.set_in_iteration(prev_iter);

            let body_span = self.ast().get_stmt(body).span();
            let span = start.cover(body_span);
            return self.ast_mut().alloc_stmt(Stmt::ForIn {
                span,
                left: ForInOfLeft::Expression(expr),
                right,
                body,
            });
        }

        if self.at_contextual(WellKnownAtom::of) {
            if !is_await && self.is_unescaped_async_for_of_lhs(expr) {
                let span = self.ast().get_expr(expr).span();
                self.error_at(
                    span,
                    "`async` cannot be used as this for-of left-hand side".to_string(),
                );
            }
            if self.is_destructuring_pattern_expression(expr) {
                self.validate_pattern_expression(expr, false, true);
            } else if !self.is_simple_assignment_target(expr) {
                let span = self.ast().get_expr(expr).span();
                self.error_at(span, "invalid assignment target".to_string());
            }
            self.advance();
            let right = self.parse_assignment_expression();
            self.expect(TokenKind::RParen);

            let prev_iter = self.in_iteration();
            self.set_in_iteration(true);
            let body = self.parse_statement_rejecting_labelled_function();
            self.set_in_iteration(prev_iter);

            let body_span = self.ast().get_stmt(body).span();
            let span = start.cover(body_span);
            return self.ast_mut().alloc_stmt(Stmt::ForOf {
                span,
                left: ForInOfLeft::Expression(expr),
                right,
                body,
                r#await: is_await,
            });
        }

        if is_await {
            self.error_at(
                self.current_span(),
                "`for await` requires a for-of head".to_string(),
            );
        }
        self.expect(TokenKind::Semicolon);
        self.parse_for_rest(start, Some(ForInit::Expression(expr)))
    }

    fn is_unescaped_async_for_of_lhs(&self, expr: ExprId) -> bool {
        let Expr::Identifier { span, name } = self.ast().get_expr(expr) else {
            return false;
        };
        *name == WellKnownAtom::async_.id() && self.span_text(*span) == "async"
    }

    fn at_let_declaration(&mut self) -> bool {
        self.at_contextual(WellKnownAtom::let_) && self.peek_is_let_declaration()
    }

    fn parse_for_with_declaration(&mut self, start: Span, is_await: bool) -> StmtId {
        let decl = self.parse_for_declaration();

        if self.at(TokenKind::In) {
            if is_await {
                self.error_at(
                    self.current_span(),
                    "`for await` requires a for-of head".to_string(),
                );
            }
            self.validate_for_in_of_declaration(decl);
            self.advance();
            let right = self.parse_expression();
            self.expect(TokenKind::RParen);

            let prev_iter = self.in_iteration();
            self.set_in_iteration(true);
            let body = self.parse_statement_rejecting_labelled_function();
            self.set_in_iteration(prev_iter);

            let body_span = self.ast().get_stmt(body).span();
            let span = start.cover(body_span);
            return self.ast_mut().alloc_stmt(Stmt::ForIn {
                span,
                left: ForInOfLeft::Declaration(decl),
                right,
                body,
            });
        }

        if self.at_contextual(WellKnownAtom::of) {
            self.validate_for_in_of_declaration(decl);
            self.advance();
            let right = self.parse_assignment_expression();
            self.expect(TokenKind::RParen);

            let prev_iter = self.in_iteration();
            self.set_in_iteration(true);
            let body = self.parse_statement_rejecting_labelled_function();
            self.set_in_iteration(prev_iter);

            let body_span = self.ast().get_stmt(body).span();
            let span = start.cover(body_span);
            return self.ast_mut().alloc_stmt(Stmt::ForOf {
                span,
                left: ForInOfLeft::Declaration(decl),
                right,
                body,
                r#await: is_await,
            });
        }

        if is_await {
            self.error_at(
                self.current_span(),
                "`for await` requires a for-of head".to_string(),
            );
        }
        self.validate_regular_for_declaration(decl);
        self.expect(TokenKind::Semicolon);
        self.parse_for_rest(start, Some(ForInit::Declaration(decl)))
    }

    fn parse_for_declaration(&mut self) -> lyng_js_ast::DeclId {
        let start = self.current_span();
        let kind = match self.current_kind() {
            TokenKind::Var => {
                self.advance();
                VariableKind::Var
            }
            TokenKind::Const => {
                self.advance();
                VariableKind::Const
            }
            TokenKind::Await => {
                if !self.allow_await && !self.is_module() {
                    self.error(
                        "'await using' is only allowed in async functions and modules".to_string(),
                    );
                }
                self.advance();
                self.advance();
                VariableKind::AwaitUsing
            }
            TokenKind::Identifier if self.at_contextual(WellKnownAtom::using) => {
                self.advance();
                VariableKind::Using
            }
            _ => {
                self.advance();
                VariableKind::Let
            }
        };

        let prev_no_in = self.no_in();
        self.set_no_in(true);
        let declarators = self.parse_variable_declarator_list();
        self.set_no_in(prev_no_in);
        if kind != VariableKind::Var {
            self.validate_lexical_declarator_names(&declarators);
        }
        if matches!(kind, VariableKind::Using | VariableKind::AwaitUsing) {
            self.validate_using_declarators(&declarators, true);
        }

        let last_span = if let Some(last) = declarators.last() {
            last.span
        } else {
            start
        };
        let span = start.cover(last_span);
        let list = self.ast_mut().alloc_var_declarator_list(&declarators);
        self.ast_mut().alloc_decl(Decl::Variable {
            span,
            kind,
            declarators: list,
        })
    }

    fn validate_for_in_of_declaration(&mut self, decl: lyng_js_ast::DeclId) {
        let is_for_in = self.at(TokenKind::In);
        let (kind, declarators, span) = match self.ast().get_decl(decl) {
            Decl::Variable {
                kind,
                declarators,
                span,
            } => (*kind, *declarators, *span),
            _ => return,
        };

        if matches!(kind, VariableKind::Using | VariableKind::AwaitUsing) && self.at(TokenKind::In)
        {
            self.error_at(
                span,
                "using declarations are not allowed in for-in headers".to_string(),
            );
        }

        let declarators = self.ast().get_var_declarator_list(declarators).to_vec();
        if declarators.len() != 1 {
            self.error_at(
                span,
                "for-in/of declarations must have exactly one binding".to_string(),
            );
        }

        let annex_b_allows_initializer = !self.is_strict()
            && is_for_in
            && kind == VariableKind::Var
            && declarators.len() == 1
            && matches!(
                self.ast().get_pattern(declarators[0].id),
                lyng_js_ast::Pattern::Identifier { .. }
            );
        for declarator in declarators {
            if declarator.init.is_some() && !annex_b_allows_initializer {
                self.error_at(
                    declarator.span,
                    "for-in/of declarations cannot have initializers".to_string(),
                );
            }
        }
    }

    fn parse_for_rest(&mut self, start: Span, init: Option<ForInit>) -> StmtId {
        let test = if self.at(TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression())
        };
        self.expect(TokenKind::Semicolon);

        let update = if self.at(TokenKind::RParen) {
            None
        } else {
            Some(self.parse_expression())
        };
        self.expect(TokenKind::RParen);

        let prev_iter = self.in_iteration();
        self.set_in_iteration(true);
        let body = self.parse_statement_rejecting_labelled_function();
        self.set_in_iteration(prev_iter);

        let body_span = self.ast().get_stmt(body).span();
        let span = start.cover(body_span);
        self.ast_mut().alloc_stmt(Stmt::For {
            span,
            init,
            test,
            update,
            body,
        })
    }
}

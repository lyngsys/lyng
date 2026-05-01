//! Statement parsing (ECMA-262 section 14).

mod control;
mod loops;

use lyng_js_ast::{Decl, FunctionKind, Stmt, StmtId};
use lyng_js_common::WellKnownAtom;
use lyng_js_lexer::{TokenKind, TokenPayload};

use crate::parser::Parser;

impl<'src, 'atoms> Parser<'src, 'atoms> {
    /// Parses a `StatementListItem`: either a declaration or a statement.
    pub fn parse_statement_list_item(&mut self) -> StmtId {
        match self.current_kind() {
            TokenKind::Function => self.parse_function_declaration_stmt(),
            TokenKind::Class => self.parse_class_declaration_stmt(),
            TokenKind::Const => self.parse_lexical_declaration_stmt(),
            TokenKind::Var => self.parse_var_declaration_stmt(),
            // `let` as declaration if followed by `{`, `[`, or identifier
            TokenKind::Identifier
                if self.at_contextual(WellKnownAtom::let_) && self.peek_is_let_declaration() =>
            {
                self.parse_lexical_declaration_stmt()
            }
            TokenKind::Identifier
                if self.at_contextual(WellKnownAtom::using) && self.peek_is_using_declaration() =>
            {
                self.parse_using_declaration_stmt(lyng_js_ast::VariableKind::Using)
            }
            TokenKind::Await if self.at_await_using_declaration() => {
                self.parse_using_declaration_stmt(lyng_js_ast::VariableKind::AwaitUsing)
            }
            // `async function` declaration
            TokenKind::Identifier
                if self.at_contextual(WellKnownAtom::async_)
                    && self.peek().kind == TokenKind::Function
                    && !self.peek().preceded_by_line_terminator() =>
            {
                self.parse_async_function_declaration_stmt()
            }
            _ => self.parse_statement(),
        }
    }

    /// Returns true if the token after `let` indicates a let declaration.
    fn peek_is_let_declaration(&mut self) -> bool {
        let peek = self.peek();
        matches!(
            peek.kind,
            TokenKind::Identifier
                | TokenKind::LBrace
                | TokenKind::LBracket
                | TokenKind::Yield
                | TokenKind::Await
        )
    }

    fn token_starts_using_binding(token: lyng_js_lexer::Token) -> bool {
        matches!(
            token.kind,
            TokenKind::Identifier | TokenKind::Yield | TokenKind::Await
        )
    }

    pub(crate) fn peek_is_using_declaration(&mut self) -> bool {
        let peek = self.peek();
        !peek.preceded_by_line_terminator() && Self::token_starts_using_binding(peek)
    }

    pub(crate) fn at_using_declaration(&mut self) -> bool {
        self.at_contextual(WellKnownAtom::using) && self.peek_is_using_declaration()
    }

    pub(crate) fn at_await_using_declaration(&mut self) -> bool {
        if !self.at(TokenKind::Await) {
            return false;
        }
        let using = self.peek();
        if using.preceded_by_line_terminator()
            || using.kind != TokenKind::Identifier
            || using.contains_escape()
            || !matches!(using.payload, TokenPayload::Atom(id) if id == WellKnownAtom::using.id())
        {
            return false;
        }
        let binding = self.peek_second();
        !binding.preceded_by_line_terminator() && Self::token_starts_using_binding(binding)
    }

    /// Parses a statement (not a declaration).
    pub fn parse_statement(&mut self) -> StmtId {
        if self.at_label_identifier() && self.peek().kind == TokenKind::Colon {
            return self.parse_labeled_statement();
        }

        match self.current_kind() {
            TokenKind::LBrace => self.parse_block_statement(),
            TokenKind::Var => self.parse_var_declaration_stmt(),
            TokenKind::Const => {
                self.error(
                    "lexical declarations are not allowed in this statement position".to_string(),
                );
                self.parse_lexical_declaration_stmt()
            }
            TokenKind::Identifier
                if self.at_contextual(WellKnownAtom::let_)
                    && self.peek().kind == TokenKind::LBracket =>
            {
                self.error(
                    "lexical declarations are not allowed in this statement position".to_string(),
                );
                self.parse_lexical_declaration_stmt()
            }
            TokenKind::Identifier if self.at_using_declaration() => {
                self.error(
                    "lexical declarations are not allowed in this statement position".to_string(),
                );
                self.parse_using_declaration_stmt(lyng_js_ast::VariableKind::Using)
            }
            TokenKind::Await if self.at_await_using_declaration() => {
                self.error(
                    "lexical declarations are not allowed in this statement position".to_string(),
                );
                self.parse_using_declaration_stmt(lyng_js_ast::VariableKind::AwaitUsing)
            }
            TokenKind::Semicolon => self.parse_empty_statement(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::Do => self.parse_do_while_statement(),
            TokenKind::For => self.parse_for_statement(),
            TokenKind::Switch => self.parse_switch_statement(),
            TokenKind::Try => self.parse_try_statement(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::Break => self.parse_break_statement(),
            TokenKind::Continue => self.parse_continue_statement(),
            TokenKind::Throw => self.parse_throw_statement(),
            TokenKind::Debugger => self.parse_debugger_statement(),
            TokenKind::With => self.parse_with_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_statement_rejecting_labelled_function(&mut self) -> StmtId {
        let stmt = self.parse_statement();
        if self.stmt_is_labelled_function(stmt) {
            let span = self.ast().get_stmt(stmt).span();
            self.error_at(
                span,
                "labelled function declarations are not allowed in this statement position"
                    .to_string(),
            );
        }
        stmt
    }

    fn stmt_is_labelled_function(&self, stmt: StmtId) -> bool {
        let Stmt::Labeled { body, .. } = self.ast().get_stmt(stmt) else {
            return false;
        };

        match self.ast().get_stmt(*body) {
            Stmt::Labeled { .. } => self.stmt_is_labelled_function(*body),
            Stmt::Declaration { decl, .. } => {
                matches!(
                    self.ast().get_decl(*decl),
                    Decl::Function { function, .. }
                        if self.ast().get_function(*function).kind == FunctionKind::Normal
                )
            }
            _ => false,
        }
    }

    // -----------------------------------------------------------------------
    // Block statement
    // -----------------------------------------------------------------------

    pub fn parse_block_statement(&mut self) -> StmtId {
        let start = self.current_span();
        self.expect(TokenKind::LBrace);

        let stmts = self.parse_statement_list_until(TokenKind::RBrace);

        let end = self.expect(TokenKind::RBrace);
        let span = start.cover(end.span);
        let body = self.ast_mut().alloc_stmt_list(&stmts);
        self.ast_mut().alloc_stmt(Stmt::Block { span, body })
    }

    /// Parses statements until the given closing token.
    ///
    /// Includes a progress guard: if a statement parse does not advance the
    /// token position, we force-skip a token to prevent infinite loops.
    pub(crate) fn parse_statement_list_until(&mut self, end: TokenKind) -> Vec<StmtId> {
        self.enter_statement_list();
        let mut stmts = Vec::new();
        while !self.at(end) && !self.at(TokenKind::Eof) {
            let pos_before = self.current_span().range.start.raw();
            stmts.push(self.parse_statement_list_item());
            // Safety: if we made no progress, force-skip a token.
            if self.current_span().range.start.raw() == pos_before
                && !self.at(end)
                && !self.at(TokenKind::Eof)
            {
                self.error(format!("unexpected token {:?}", self.current_kind()));
                self.advance();
            }
        }
        self.exit_statement_list();
        stmts
    }

    /// Parses a function body: `{ stmts }`. Returns the list of statements.
    pub fn parse_function_body(&mut self) -> Vec<StmtId> {
        let old_strict = self.is_strict();
        self.expect(TokenKind::LBrace);
        self.enter_statement_list();
        let mut stmts = Vec::new();
        let mut in_directive_prologue = true;

        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let pos_before = self.current_span().range.start.raw();
            let stmt = self.parse_statement_list_item();

            if in_directive_prologue {
                if self.stmt_is_use_strict_directive(stmt) {
                    self.set_strict(true);
                } else if !self.stmt_is_string_directive(stmt) {
                    in_directive_prologue = false;
                }
            }

            stmts.push(stmt);

            if self.current_span().range.start.raw() == pos_before
                && !self.at(TokenKind::RBrace)
                && !self.at(TokenKind::Eof)
            {
                self.error(format!("unexpected token {:?}", self.current_kind()));
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace);
        self.exit_statement_list();
        self.set_strict(old_strict);
        stmts
    }

    fn stmt_is_string_directive(&self, stmt_id: StmtId) -> bool {
        matches!(
            self.ast().get_stmt(stmt_id),
            Stmt::Expression { expression, .. }
                if matches!(self.ast().get_expr(*expression), lyng_js_ast::Expr::StringLiteral { .. })
        )
    }

    fn stmt_is_use_strict_directive(&self, stmt_id: StmtId) -> bool {
        let Stmt::Expression { expression, .. } = self.ast().get_stmt(stmt_id) else {
            return false;
        };

        let lyng_js_ast::Expr::StringLiteral { value, syntax, .. } =
            self.ast().get_expr(*expression)
        else {
            return false;
        };

        !syntax.contains_escape && self.ast().literals().string_equals(*value, "use strict")
    }

    // -----------------------------------------------------------------------
    // Empty statement
    // -----------------------------------------------------------------------

    fn parse_empty_statement(&mut self) -> StmtId {
        let span = self.current_span();
        self.advance();
        self.ast_mut().alloc_stmt(Stmt::Empty { span })
    }

    // -----------------------------------------------------------------------
    // Expression statement
    // -----------------------------------------------------------------------

    fn parse_expression_statement(&mut self) -> StmtId {
        if self.at(TokenKind::Function) {
            self.error(
                "function declarations are not allowed in this statement position".to_string(),
            );
        } else if self.at(TokenKind::Class) {
            self.error("class declarations are not allowed in this statement position".to_string());
        } else if self.at_contextual(WellKnownAtom::async_) {
            let peek = self.peek();
            if peek.kind == TokenKind::Function && !peek.preceded_by_line_terminator() {
                self.error(
                    "async function declarations are not allowed in this statement position"
                        .to_string(),
                );
            }
        }

        let start = self.current_span();
        let expression = self.parse_expression();
        let expr_span = self.ast().get_expr(expression).span();
        self.expect_semicolon();
        let span = start.cover(expr_span);
        self.ast_mut()
            .alloc_stmt(Stmt::Expression { span, expression })
    }

    fn parse_if_statement_clause(&mut self) -> StmtId {
        if self.at(TokenKind::Function)
            && self.allows_annex_b_sloppy_function_declarations()
            && self.peek().kind != TokenKind::Star
        {
            let declaration = self.parse_function_declaration_stmt();
            let span = self.ast().get_stmt(declaration).span();
            let body = self.ast_mut().alloc_stmt_list(&[declaration]);
            return self.ast_mut().alloc_stmt(Stmt::Block { span, body });
        }

        self.parse_statement_rejecting_labelled_function()
    }

    // -----------------------------------------------------------------------
    // If statement
    // -----------------------------------------------------------------------

    fn parse_if_statement(&mut self) -> StmtId {
        let start = self.current_span();
        self.advance(); // eat `if`
        self.expect(TokenKind::LParen);
        let test = self.parse_expression();
        self.expect(TokenKind::RParen);
        let consequent = self.parse_if_statement_clause();
        let alternate = if self.eat(TokenKind::Else) {
            Some(self.parse_if_statement_clause())
        } else {
            None
        };
        let end = if let Some(alt) = alternate {
            self.ast().get_stmt(alt).span()
        } else {
            self.ast().get_stmt(consequent).span()
        };
        let span = start.cover(end);
        self.ast_mut().alloc_stmt(Stmt::If {
            span,
            test,
            consequent,
            alternate,
        })
    }
}

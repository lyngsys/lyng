use lyng_js_ast::{CatchClause, Stmt, StmtId, SwitchCase};
use lyng_js_lexer::TokenKind;

use crate::parser::Parser;

impl<'src, 'atoms> Parser<'src, 'atoms> {
    pub(super) fn parse_switch_statement(&mut self) -> StmtId {
        let start = self.current_span();
        self.advance();
        self.expect(TokenKind::LParen);
        let discriminant = self.parse_expression();
        self.expect(TokenKind::RParen);
        self.expect(TokenKind::LBrace);

        let prev_switch = self.in_switch();
        self.set_in_switch(true);

        let mut cases = Vec::new();
        let mut seen_default = false;
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            if self.at(TokenKind::Default) {
                if seen_default {
                    self.error("duplicate default clause in switch statement".to_string());
                }
                seen_default = true;
            }
            cases.push(self.parse_switch_case());
        }

        self.set_in_switch(prev_switch);

        let end = self.expect(TokenKind::RBrace);
        let span = start.cover(end.span);
        let case_list = self.ast_mut().alloc_switch_case_list(&cases);
        self.ast_mut().alloc_stmt(Stmt::Switch {
            span,
            discriminant,
            cases: case_list,
        })
    }

    fn parse_switch_case(&mut self) -> SwitchCase {
        let start = self.current_span();
        let test = if self.eat(TokenKind::Case) {
            let expr = self.parse_expression();
            self.expect(TokenKind::Colon);
            Some(expr)
        } else {
            self.expect(TokenKind::Default);
            self.expect(TokenKind::Colon);
            None
        };

        let mut stmts = Vec::new();
        self.enter_switch_clause_statement_list();
        while !self.at(TokenKind::Case)
            && !self.at(TokenKind::Default)
            && !self.at(TokenKind::RBrace)
            && !self.at(TokenKind::Eof)
        {
            stmts.push(self.parse_statement_list_item());
        }
        self.exit_switch_clause_statement_list();

        let end = stmts
            .last()
            .map_or(start, |last| self.ast().get_stmt(*last).span());
        let span = start.cover(end);
        let consequent = self.ast_mut().alloc_stmt_list(&stmts);
        SwitchCase {
            span,
            test,
            consequent,
        }
    }

    pub(super) fn parse_try_statement(&mut self) -> StmtId {
        let start = self.current_span();
        self.advance();

        let block = self.parse_block_statement();

        let handler = if self.at(TokenKind::Catch) {
            Some(self.parse_catch_clause())
        } else {
            None
        };

        let finalizer = if self.eat(TokenKind::Finally) {
            Some(self.parse_block_statement())
        } else {
            None
        };

        if handler.is_none() && finalizer.is_none() {
            self.error("try statement requires catch or finally".to_string());
        }

        let end = finalizer
            .map(|f| self.ast().get_stmt(f).span())
            .or_else(|| handler.map(|h| h.span))
            .unwrap_or_else(|| self.ast().get_stmt(block).span());
        let span = start.cover(end);

        self.ast_mut().alloc_stmt(Stmt::Try {
            span,
            block,
            handler,
            finalizer,
        })
    }

    fn parse_catch_clause(&mut self) -> CatchClause {
        let start = self.current_span();
        self.advance();

        let param = if self.eat(TokenKind::LParen) {
            let pat = self.parse_binding_pattern();
            self.expect(TokenKind::RParen);
            Some(pat)
        } else {
            None
        };

        let body = self.parse_block_statement();
        let body_span = self.ast().get_stmt(body).span();
        let span = start.cover(body_span);

        CatchClause { span, param, body }
    }

    pub(super) fn parse_return_statement(&mut self) -> StmtId {
        let start = self.current_span();
        self.advance();

        let argument = if self.can_insert_semicolon() {
            None
        } else {
            Some(self.parse_expression())
        };

        let end = argument
            .map(|a| self.ast().get_expr(a).span())
            .unwrap_or(start);
        self.expect_semicolon();

        let span = start.cover(end);
        self.ast_mut().alloc_stmt(Stmt::Return { span, argument })
    }

    pub(super) fn parse_break_statement(&mut self) -> StmtId {
        let start = self.current_span();
        self.advance();

        let label = if !self.can_insert_semicolon() && self.at_label_identifier() {
            Some(self.parse_label_identifier())
        } else {
            None
        };

        self.expect_semicolon();
        self.ast_mut()
            .alloc_stmt(Stmt::Break { span: start, label })
    }

    pub(super) fn parse_continue_statement(&mut self) -> StmtId {
        let start = self.current_span();
        self.advance();

        let label = if !self.can_insert_semicolon() && self.at_label_identifier() {
            Some(self.parse_label_identifier())
        } else {
            None
        };

        self.expect_semicolon();
        self.ast_mut()
            .alloc_stmt(Stmt::Continue { span: start, label })
    }

    pub(super) fn parse_throw_statement(&mut self) -> StmtId {
        let start = self.current_span();
        self.advance();

        if self.preceded_by_line_terminator() {
            self.error("no line break allowed after 'throw'".to_string());
        }

        let argument = self.parse_expression();
        let arg_span = self.ast().get_expr(argument).span();
        self.expect_semicolon();

        let span = start.cover(arg_span);
        self.ast_mut().alloc_stmt(Stmt::Throw { span, argument })
    }

    pub(super) fn parse_debugger_statement(&mut self) -> StmtId {
        let span = self.current_span();
        self.advance();
        self.expect_semicolon();
        self.ast_mut().alloc_stmt(Stmt::Debugger { span })
    }

    pub(super) fn parse_with_statement(&mut self) -> StmtId {
        let start = self.current_span();
        if self.is_strict() {
            self.error("'with' statement not allowed in strict mode".to_string());
        }
        self.advance();
        self.expect(TokenKind::LParen);
        let object = self.parse_expression();
        self.expect(TokenKind::RParen);
        let body = self.parse_statement_rejecting_labelled_function();

        let body_span = self.ast().get_stmt(body).span();
        let span = start.cover(body_span);
        self.ast_mut().alloc_stmt(Stmt::With { span, object, body })
    }

    pub(super) fn parse_labeled_statement(&mut self) -> StmtId {
        let start = self.current_span();
        let label = self.parse_label_identifier();
        self.expect(TokenKind::Colon);
        let body = if !self.is_strict()
            && self.at(TokenKind::Function)
            && self.peek().kind != TokenKind::Star
        {
            self.parse_function_declaration_stmt()
        } else {
            self.parse_statement()
        };

        let body_span = self.ast().get_stmt(body).span();
        let span = start.cover(body_span);
        self.ast_mut()
            .alloc_stmt(Stmt::Labeled { span, label, body })
    }
}

//! Expression parsing with Pratt/precedence-climbing.

mod left_hand_side;
mod operators;
mod primary;

use lyng_js_ast::{AssignOp, Expr, ExprId, FunctionKind, LogicalOp, TemplateQuasi};
use lyng_js_common::{AtomId, Span, WellKnownAtom};
use lyng_js_lexer::{LexerMode, TokenKind, TokenPayload};

use self::operators::{token_can_start_expression, token_to_assign_op};
use crate::parser::Parser;

impl<'src, 'atoms> Parser<'src, 'atoms> {
    fn parse_assignment_expression_allow_in(&mut self) -> ExprId {
        let prev_no_in = self.no_in();
        self.set_no_in(false);
        let expr = self.parse_assignment_expression();
        self.set_no_in(prev_no_in);
        expr
    }

    // -----------------------------------------------------------------------
    // Top-level expression entry point
    // -----------------------------------------------------------------------

    /// Parses an `AssignmentExpression` (the default expression parse).
    pub fn parse_assignment_expression(&mut self) -> ExprId {
        // Yield expressions in generators
        if self.allow_yield && self.at(TokenKind::Yield) {
            return self.parse_yield_expression();
        }

        // Try to detect async arrow: `async (params) =>` or `async ident =>`
        if self.at_contextual(WellKnownAtom::async_) {
            let peek = self.peek();
            if (peek.kind == TokenKind::LParen || peek.kind == TokenKind::Identifier)
                && !peek.preceded_by_line_terminator()
            {
                // Could be async arrow. We'll handle this in the regular flow.
            }
        }

        let expr = self.parse_conditional_expression();

        // Arrow function: (params) => body  or  ident => body
        if self.at(TokenKind::Arrow) && !self.preceded_by_line_terminator() {
            if let Some(async_arrow) = self.try_parse_async_arrow_from_cover_call_expression(expr) {
                return async_arrow;
            }
            return self.parse_arrow_function_from_expr(expr, false);
        }

        // Assignment
        if self.current_kind().is_assignment() {
            // Validate LHS is a valid assignment target (or destructuring for `=`)
            let op = token_to_assign_op(self.current_kind()).unwrap_or(AssignOp::Assign);
            let is_valid_simple_target = if matches!(
                op,
                AssignOp::AndAssign | AssignOp::OrAssign | AssignOp::NullishAssign
            ) {
                self.is_simple_assignment_target_without_annex_b_call(expr)
            } else {
                self.is_simple_assignment_target(expr)
            };
            let invalid_target = if op == AssignOp::Assign {
                !self.is_valid_assignment_lhs(expr)
            } else {
                !is_valid_simple_target
            };
            if invalid_target {
                let left_span = self.ast().get_expr(expr).span();
                self.error_at(left_span, "invalid assignment target".to_string());
            }
            self.advance();
            let right = self.parse_assignment_expression();
            let left_span = self.ast().get_expr(expr).span();
            let right_span = self.ast().get_expr(right).span();
            let span = left_span.cover(right_span);
            if op == AssignOp::Assign && self.is_destructuring_pattern_expression(expr) {
                self.validate_pattern_expression(expr, false, true);
            }
            return self.ast_mut().alloc_expr(Expr::AssignmentExpression {
                span,
                operator: op,
                left: expr,
                right,
            });
        }

        expr
    }

    /// Parses a full `Expression` (possibly a sequence expression: `a, b, c`).
    pub fn parse_expression(&mut self) -> ExprId {
        let first = self.parse_assignment_expression();

        if !self.at(TokenKind::Comma) {
            return first;
        }

        let mut exprs = vec![first];
        while self.eat(TokenKind::Comma) {
            exprs.push(self.parse_assignment_expression());
        }

        let first_span = self.ast().get_expr(exprs[0]).span();
        let last_span = self.ast().get_expr(exprs[exprs.len() - 1]).span();
        let span = first_span.cover(last_span);
        let list = self.ast_mut().alloc_expr_list(&exprs);
        self.ast_mut().alloc_expr(Expr::SequenceExpression {
            span,
            expressions: list,
        })
    }

    // -----------------------------------------------------------------------
    // Primary expressions
    // -----------------------------------------------------------------------

    fn parse_primary_expression(&mut self) -> ExprId {
        let token = self.current();
        match token.kind {
            TokenKind::This => {
                let span = self.current_span();
                self.advance();
                self.ast_mut().alloc_expr(Expr::This { span })
            }

            TokenKind::Super => {
                let span = self.current_span();
                self.advance();
                self.ast_mut().alloc_expr(Expr::Super { span })
            }

            TokenKind::Identifier => self.parse_identifier_or_arrow(),

            TokenKind::Null => {
                let span = self.current_span();
                self.advance();
                self.ast_mut().alloc_expr(Expr::NullLiteral { span })
            }

            TokenKind::True => {
                let span = self.current_span();
                self.advance();
                self.ast_mut()
                    .alloc_expr(Expr::BooleanLiteral { span, value: true })
            }

            TokenKind::False => {
                let span = self.current_span();
                self.advance();
                self.ast_mut()
                    .alloc_expr(Expr::BooleanLiteral { span, value: false })
            }

            TokenKind::NumericLiteral => self.parse_numeric_literal(),

            TokenKind::StringLiteral => self.parse_string_literal(),

            TokenKind::BigIntLiteral => self.parse_bigint_literal(),

            TokenKind::RegExpLiteral => self.parse_regexp_literal(),

            TokenKind::LBracket => self.parse_array_expression(),

            TokenKind::LBrace => self.parse_object_expression(),

            TokenKind::Function => self.parse_function_expression(),

            TokenKind::Class => self.parse_class_expression(),

            TokenKind::NoSubstitutionTemplate | TokenKind::TemplateHead => {
                let template = self.parse_template_literal(false);
                let span = self.current_span(); // approximate
                self.ast_mut()
                    .alloc_expr(Expr::TemplateLiteral { span, template })
            }

            TokenKind::LParen => self.parse_parenthesized_or_arrow_with_async(false),

            TokenKind::Import => self.parse_import_expression_or_meta(),

            TokenKind::Yield if !self.allow_yield && !self.is_strict() => {
                // yield used as identifier (only in non-strict mode)
                let span = self.current_span();
                let name = self.current_atom().unwrap_or(WellKnownAtom::yield_.id());
                self.validate_identifier_reference_atom(name, span);
                self.advance();
                self.ast_mut().alloc_expr(Expr::Identifier { span, name })
            }

            TokenKind::Await if !self.allow_await => {
                // await used as identifier
                let span = self.current_span();
                let name = self.current_atom().unwrap_or(WellKnownAtom::r#await.id());
                self.validate_identifier_reference_atom(name, span);
                self.advance();
                self.ast_mut().alloc_expr(Expr::Identifier { span, name })
            }

            // Handle `/` as regexp literal start in expression position.
            // The lexer already consumed `/` as Slash or `/=` as SlashEq.
            // Rewind the lexer to the start of the `/` and re-lex in RegExp mode.
            TokenKind::Slash | TokenKind::SlashEq => {
                let slash_start = self.current_span().range.start.raw() as usize;
                self.relex_with_mode(slash_start, LexerMode::RegExp);
                if self.at(TokenKind::RegExpLiteral) {
                    self.parse_regexp_literal()
                } else {
                    let span = self.current_span();
                    self.error("unexpected token in expression".to_string());
                    self.advance();
                    self.ast_mut().alloc_expr(Expr::InvalidExpression { span })
                }
            }

            _ => {
                let span = self.current_span();
                self.error(format!("unexpected token {:?}", self.current_kind()));
                self.advance();
                self.ast_mut().alloc_expr(Expr::InvalidExpression { span })
            }
        }
    }

    // -----------------------------------------------------------------------
    // Yield expression
    // -----------------------------------------------------------------------

    fn parse_yield_expression(&mut self) -> ExprId {
        let start = self.current_span();
        self.advance(); // eat `yield`

        // No argument if followed by line terminator or can't start expression
        if self.can_insert_semicolon()
            || (self.current_kind() != TokenKind::Star
                && !token_can_start_expression(self.current_kind()))
        {
            return self.ast_mut().alloc_expr(Expr::YieldExpression {
                span: start,
                argument: None,
                delegate: false,
            });
        }

        let delegate = self.eat(TokenKind::Star);
        let argument = self.parse_assignment_expression();
        let arg_span = self.ast().get_expr(argument).span();
        let span = start.cover(arg_span);
        self.ast_mut().alloc_expr(Expr::YieldExpression {
            span,
            argument: Some(argument),
            delegate,
        })
    }

    // -----------------------------------------------------------------------
    // Template literal
    // -----------------------------------------------------------------------

    pub fn parse_template_literal(&mut self, tagged: bool) -> lyng_js_ast::TemplateLiteralId {
        let mut quasis = Vec::new();
        let mut expressions = Vec::new();

        if self.at(TokenKind::NoSubstitutionTemplate) {
            let quasi = self.parse_template_quasi();
            self.report_invalid_template_escape(&quasi, tagged);
            quasis.push(quasi);
            self.advance();
        } else {
            // TemplateHead
            let quasi = self.parse_template_quasi();
            self.report_invalid_template_escape(&quasi, tagged);
            quasis.push(quasi);
            self.advance();

            loop {
                // Parse expression
                let expr = self.parse_expression();
                expressions.push(expr);

                // Set mode for template continuation
                self.set_lexer_mode(LexerMode::TemplateContinuation);
                self.advance(); // This should produce TemplateMiddle or TemplateTail

                let quasi = self.parse_template_quasi();
                self.report_invalid_template_escape(&quasi, tagged);
                quasis.push(quasi);

                if self.at(TokenKind::TemplateTail) {
                    self.advance();
                    break;
                } else if self.at(TokenKind::TemplateMiddle) {
                    self.advance();
                } else {
                    self.error("unterminated template literal".to_string());
                    break;
                }
            }
        }

        self.ast_mut().templates_mut().alloc(&quasis, &expressions)
    }

    fn report_invalid_template_escape(&mut self, quasi: &TemplateQuasi, tagged: bool) {
        if !tagged && quasi.cooked.is_none() {
            self.error_at(
                self.current_span(),
                "invalid escape sequence in template literal".to_string(),
            );
        }
    }

    /// Parse a template quasi, storing strings in the AST literal table.
    fn parse_template_quasi(&mut self) -> TemplateQuasi {
        let token = self.current();
        match token.payload {
            TokenPayload::Literal(lit_id) => {
                let chunk = self.lexer.literals.get_template(lit_id);
                let raw = self.ast.literals_mut().alloc_string(&chunk.raw);
                let cooked = chunk
                    .cooked
                    .as_deref()
                    .map(|s| self.ast.literals_mut().alloc_string(s));

                TemplateQuasi { cooked, raw }
            }
            _ => {
                let raw = self.ast.literals_mut().alloc_string("");
                TemplateQuasi { cooked: None, raw }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Function expression
    // -----------------------------------------------------------------------

    fn parse_function_expression(&mut self) -> ExprId {
        let start = self.current_span();
        self.advance(); // eat `function`

        let is_generator = self.eat(TokenKind::Star);
        let kind = if is_generator {
            FunctionKind::Generator
        } else {
            FunctionKind::Normal
        };

        let name = if self.at_function_expression_name() {
            let name_span = self.current_span();
            let name = self.parse_function_expression_name();
            self.validate_function_expression_name_for_kind(name, name_span, kind);
            Some(name)
        } else {
            None
        };

        let func_id = self.parse_function_common(name, kind, start);
        let span = self.ast().get_function(func_id).span;

        self.ast_mut().alloc_expr(Expr::FunctionExpression {
            span,
            function: func_id,
        })
    }

    fn parse_async_function_expression(&mut self, async_start: Span) -> ExprId {
        self.advance(); // eat `function`

        let is_generator = self.eat(TokenKind::Star);
        let kind = if is_generator {
            FunctionKind::AsyncGenerator
        } else {
            FunctionKind::Async
        };

        let name = if self.at_function_expression_name() {
            let name_span = self.current_span();
            let name = self.parse_function_expression_name();
            self.validate_function_expression_name_for_kind(name, name_span, kind);
            Some(name)
        } else {
            None
        };

        let func_id = self.parse_function_common(name, kind, async_start);
        let span = self.ast().get_function(func_id).span;

        self.ast_mut().alloc_expr(Expr::FunctionExpression {
            span,
            function: func_id,
        })
    }

    // -----------------------------------------------------------------------
    // Class expression
    // -----------------------------------------------------------------------

    fn parse_class_expression(&mut self) -> ExprId {
        let start = self.current_span();
        self.advance(); // eat `class`
        let old_strict = self.is_strict();
        self.set_strict(true);

        let name = if self.at_binding_identifier_in_context(true) {
            Some(self.parse_binding_identifier_with_strict(true))
        } else {
            None
        };

        let super_class = if self.eat(TokenKind::Extends) {
            let expr = self.parse_left_hand_side_expression();
            self.validate_class_heritage_expression(expr);
            Some(expr)
        } else {
            None
        };

        let body = self.parse_class_body();
        self.set_strict(old_strict);
        let end = self.current_span();
        let span = start.cover(end);

        self.ast_mut().alloc_expr(Expr::ClassExpression {
            span,
            name,
            super_class,
            body,
        })
    }

    // -----------------------------------------------------------------------
    // Import expression / import.meta
    // -----------------------------------------------------------------------

    fn parse_import_expression_or_meta(&mut self) -> ExprId {
        let start = self.current_span();
        self.advance(); // eat `import`

        // import.meta
        if self.eat(TokenKind::Dot) {
            let prop_token = self.current();
            let prop_span = self.current_span();
            let prop = self.parse_identifier_name();
            if !self.is_module() {
                self.error_at(start, "import.meta is only valid in modules".to_string());
            }
            if prop != WellKnownAtom::meta.id() || prop_token.contains_escape() {
                self.error_at(prop_span, "expected 'meta' after 'import.'".to_string());
            }
            let span = start.cover(self.current_span());
            return self.ast_mut().alloc_expr(Expr::MetaProperty {
                span,
                meta: WellKnownAtom::import.id(),
                property: prop,
            });
        }

        // import(source)
        self.expect(TokenKind::LParen);
        let source = self.parse_assignment_expression_allow_in();
        let mut options = None;
        if self.eat(TokenKind::Comma) && !self.at(TokenKind::RParen) {
            options = Some(self.parse_assignment_expression_allow_in());
            self.eat(TokenKind::Comma);
        }
        let end = self.expect(TokenKind::RParen);
        let span = start.cover(end.span);
        self.ast_mut().alloc_expr(Expr::ImportExpression {
            span,
            source,
            options,
        })
    }

    fn expr_is_unqualified_delete_target(&self, expr_id: ExprId) -> bool {
        match self.ast().get_expr(expr_id) {
            Expr::Identifier { .. } => true,
            Expr::ParenthesizedExpression { expression, .. } => {
                self.expr_is_unqualified_delete_target(*expression)
            }
            _ => false,
        }
    }

    fn expr_is_unparenthesized_unary_base(&self, expr_id: ExprId) -> bool {
        matches!(
            self.ast().get_expr(expr_id),
            Expr::UnaryExpression { .. } | Expr::AwaitExpression { .. }
        )
    }

    fn expr_is_logical_and_or(&self, expr_id: ExprId) -> bool {
        matches!(
            self.ast().get_expr(expr_id),
            Expr::LogicalExpression {
                operator: LogicalOp::And | LogicalOp::Or,
                ..
            }
        )
    }

    fn expr_is_nullish_coalescing(&self, expr_id: ExprId) -> bool {
        matches!(
            self.ast().get_expr(expr_id),
            Expr::LogicalExpression {
                operator: LogicalOp::NullishCoalescing,
                ..
            }
        )
    }

    fn validate_function_expression_name_for_kind(
        &mut self,
        name: AtomId,
        span: Span,
        kind: FunctionKind,
    ) {
        if matches!(kind, FunctionKind::Generator | FunctionKind::AsyncGenerator)
            && name == WellKnownAtom::yield_.id()
        {
            self.error_at(
                span,
                "reserved word 'yield' cannot be used as a binding identifier".to_string(),
            );
        }

        if matches!(kind, FunctionKind::Async | FunctionKind::AsyncGenerator)
            && name == WellKnownAtom::r#await.id()
        {
            self.error_at(
                span,
                "reserved word 'await' cannot be used as a binding identifier".to_string(),
            );
        }
    }
}

/// Returns true if the token can appear as a property name.
fn is_property_name_token(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Identifier
            | TokenKind::StringLiteral
            | TokenKind::NumericLiteral
            | TokenKind::BigIntLiteral
            | TokenKind::LBracket
            // Keywords can also be property names
            | TokenKind::Await
            | TokenKind::Break
            | TokenKind::Case
            | TokenKind::Catch
            | TokenKind::Class
            | TokenKind::Const
            | TokenKind::Continue
            | TokenKind::Debugger
            | TokenKind::Default
            | TokenKind::Delete
            | TokenKind::Do
            | TokenKind::Else
            | TokenKind::Enum
            | TokenKind::Export
            | TokenKind::Extends
            | TokenKind::False
            | TokenKind::Finally
            | TokenKind::For
            | TokenKind::Function
            | TokenKind::If
            | TokenKind::Import
            | TokenKind::In
            | TokenKind::Instanceof
            | TokenKind::New
            | TokenKind::Null
            | TokenKind::Return
            | TokenKind::Super
            | TokenKind::Switch
            | TokenKind::This
            | TokenKind::Throw
            | TokenKind::True
            | TokenKind::Try
            | TokenKind::Typeof
            | TokenKind::Var
            | TokenKind::Void
            | TokenKind::While
            | TokenKind::With
            | TokenKind::Yield
    )
}

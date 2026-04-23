use lyng_js_ast::{
    AssignOp, Expr, ExprId, FormalParameters, Function, FunctionKind, NumericLiteral,
    NumericLiteralSyntax, Property, PropertyKind, StringLiteralSyntax,
};
use lyng_js_common::{AtomId, Span, WellKnownAtom};
use lyng_js_lexer::{TokenKind, TokenPayload};

use super::is_property_name_token;
use crate::parser::Parser;
use crate::regexp::validate_regexp_literal;

impl<'src, 'atoms> Parser<'src, 'atoms> {
    pub(super) fn parse_identifier_or_arrow(&mut self) -> ExprId {
        let span = self.current_span();
        let name = self.current_atom().unwrap_or(WellKnownAtom::Empty.id());

        if self.current().contains_escape() {
            self.check_escaped_keyword_identifier();
        }

        // Check for `async` function/arrow
        if self.at_contextual(WellKnownAtom::async_) {
            let peek = self.peek();
            // `async function` => async function expression
            if peek.kind == TokenKind::Function && !peek.preceded_by_line_terminator() {
                self.advance(); // eat `async`
                return self.parse_async_function_expression(span);
            }
            // `async ident =>` => async arrow with single param
            if peek.kind == TokenKind::Identifier && !peek.preceded_by_line_terminator() {
                let second = self.peek_second();
                if second.kind == TokenKind::Arrow && !second.preceded_by_line_terminator() {
                    self.advance(); // eat `async`
                    let param_span = self.current_span();
                    let param_name = self.parse_binding_identifier();
                    return self.parse_async_arrow_from_single_param(span, param_span, param_name);
                }
            }
        }

        self.validate_identifier_reference_atom(name, span);
        self.advance();
        self.ast_mut().alloc_expr(Expr::Identifier { span, name })
    }

    pub(crate) fn parse_numeric_literal(&mut self) -> ExprId {
        let span = self.current_span();
        let syntax = if self.current().has_legacy_octal_like_decimal() {
            NumericLiteralSyntax::LegacyOctalLikeDecimal
        } else {
            NumericLiteralSyntax::Normal
        };
        let value = match self.current().payload {
            TokenPayload::Number(bits) => {
                let f = f64::from_bits(bits);
                let i = f as i32;
                if f64::from(i).to_bits() == f.to_bits() {
                    NumericLiteral::Int32(i)
                } else {
                    NumericLiteral::Number(f)
                }
            }
            _ => NumericLiteral::Int32(0),
        };
        self.advance();
        self.ast_mut().alloc_expr(Expr::NumericLiteral {
            span,
            value,
            syntax,
        })
    }

    pub(crate) fn parse_string_literal(&mut self) -> ExprId {
        let span = self.current_span();
        let syntax = StringLiteralSyntax {
            contains_escape: self.current().contains_escape(),
            contains_legacy_octal_escape: self.current().has_legacy_octal_escape(),
            contains_non_octal_decimal_escape: self.current().has_non_octal_decimal_escape(),
        };
        let value = match self.current().payload {
            TokenPayload::Literal(lit_id) => {
                let literal = self.lexer.literals.get_string(lit_id).clone();
                if let Some(text) = literal.as_str() {
                    self.ast.literals_mut().alloc_string(text)
                } else {
                    let units = literal.code_units();
                    self.ast.literals_mut().alloc_utf16_string(&units)
                }
            }
            _ => self.ast.literals_mut().alloc_string(""),
        };
        self.advance();
        self.ast.alloc_expr(Expr::StringLiteral {
            span,
            value,
            syntax,
        })
    }

    pub(crate) fn parse_bigint_literal(&mut self) -> ExprId {
        let span = self.current_span();
        let value = match self.current().payload {
            TokenPayload::Literal(lit_id) => {
                let s = &self.lexer.literals.get_bigint(lit_id).raw;
                self.ast.literals_mut().alloc_bigint(s)
            }
            _ => self.ast.literals_mut().alloc_bigint("0"),
        };
        self.advance();
        self.ast.alloc_expr(Expr::BigIntLiteral { span, value })
    }

    pub(super) fn parse_regexp_literal(&mut self) -> ExprId {
        let span = self.current_span();
        let value = match self.current().payload {
            TokenPayload::Literal(lit_id) => {
                let reg = self.lexer.literals.get_regexp(lit_id);
                let pattern = reg.pattern.clone();
                let flags = reg.flags.clone();
                if let Err(message) = validate_regexp_literal(&pattern, &flags) {
                    self.error_at(span, message.to_string());
                }
                self.ast.literals_mut().alloc_regexp(&pattern, &flags)
            }
            _ => self.ast.literals_mut().alloc_regexp("", ""),
        };
        self.advance();
        self.ast.alloc_expr(Expr::RegExpLiteral { span, value })
    }

    // -----------------------------------------------------------------------
    // Array expression
    // -----------------------------------------------------------------------

    pub(super) fn parse_array_expression(&mut self) -> ExprId {
        let start = self.current_span();
        self.advance(); // eat `[`
        let mut elements: Vec<Option<ExprId>> = Vec::new();

        while !self.at(TokenKind::RBracket) && !self.at(TokenKind::Eof) {
            if self.at(TokenKind::Comma) {
                // Elision
                elements.push(None);
                self.advance();
                continue;
            }

            if self.at(TokenKind::Ellipsis) {
                let sp = self.current_span();
                self.advance();
                let argument = self.parse_assignment_expression_allow_in();
                let arg_span = self.ast().get_expr(argument).span();
                let span = sp.cover(arg_span);
                let spread = self
                    .ast_mut()
                    .alloc_expr(Expr::SpreadElement { span, argument });
                elements.push(Some(spread));
                if self.eat(TokenKind::Comma) {
                    if self.at(TokenKind::RBracket) {
                        elements.push(None);
                        break;
                    }
                    continue;
                }
                break;
            } else {
                let expr = self.parse_assignment_expression_allow_in();
                elements.push(Some(expr));
            }

            if !self.eat(TokenKind::Comma) {
                break;
            }
        }

        let end = self.expect(TokenKind::RBracket);
        let span = start.cover(end.span);
        let list = self.ast_mut().alloc_opt_expr_list(&elements);
        self.ast_mut().alloc_expr(Expr::ArrayExpression {
            span,
            elements: list,
        })
    }

    // -----------------------------------------------------------------------
    // Object expression
    // -----------------------------------------------------------------------

    pub(super) fn parse_object_expression(&mut self) -> ExprId {
        let start = self.current_span();
        self.advance(); // eat `{`
        let mut properties = Vec::new();

        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let property = self.parse_property_definition();
            properties.push(property);
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }

        let end = self.expect(TokenKind::RBrace);
        let span = start.cover(end.span);
        let list = self.ast_mut().alloc_property_list(&properties);
        self.ast_mut().alloc_expr(Expr::ObjectExpression {
            span,
            properties: list,
        })
    }

    fn parse_property_definition(&mut self) -> Property {
        let start = self.current_span();

        // Spread property: `...expr`
        if self.at(TokenKind::Ellipsis) {
            self.advance();
            let argument = self.parse_assignment_expression_allow_in();
            let arg_span = self.ast().get_expr(argument).span();
            let span = start.cover(arg_span);
            let spread = self
                .ast_mut()
                .alloc_expr(Expr::SpreadElement { span, argument });
            // Represent spread as a property with kind Init
            return Property {
                span,
                kind: PropertyKind::Init,
                key: spread,
                value: spread,
                computed: false,
                shorthand: false,
                method: false,
            };
        }

        // Check for get/set accessor
        if self.at_contextual(WellKnownAtom::get) || self.at_contextual(WellKnownAtom::set) {
            let is_get = self.at_contextual(WellKnownAtom::get);
            let peek = self.peek();
            // If next token is a property name, it's a getter/setter
            if is_property_name_token(peek.kind) {
                let kind = if is_get {
                    PropertyKind::Get
                } else {
                    PropertyKind::Set
                };
                self.advance(); // eat get/set
                let (key, computed) = self.parse_property_name();
                let func_id = self.parse_method_function(FunctionKind::Normal);
                let func_span = self.ast().get_function(func_id).span;
                let func_expr = self.ast_mut().alloc_expr(Expr::FunctionExpression {
                    span: func_span,
                    function: func_id,
                });
                let span = start.cover(self.ast().get_function(func_id).span);
                return Property {
                    span,
                    kind,
                    key,
                    value: func_expr,
                    computed,
                    shorthand: false,
                    method: false,
                };
            }
        }

        // Check for async method: `async name() {}`
        if self.at_contextual(WellKnownAtom::async_) {
            let peek = self.peek();
            if !peek.preceded_by_line_terminator()
                && (peek.kind == TokenKind::Star || is_property_name_token(peek.kind))
            {
                self.advance(); // eat `async`
                let is_generator = self.eat(TokenKind::Star);
                let (key, computed) = self.parse_property_name();
                let kind = if is_generator {
                    FunctionKind::AsyncGenerator
                } else {
                    FunctionKind::Async
                };
                let func_id = self.parse_method_function(kind);
                let func_span = self.ast().get_function(func_id).span;
                let func_expr = self.ast_mut().alloc_expr(Expr::FunctionExpression {
                    span: func_span,
                    function: func_id,
                });
                let span = start.cover(self.ast().get_function(func_id).span);
                return Property {
                    span,
                    kind: PropertyKind::Init,
                    key,
                    value: func_expr,
                    computed,
                    shorthand: false,
                    method: true,
                };
            }
        }

        // Generator method: `* name() {}`
        if self.at(TokenKind::Star) {
            self.advance(); // eat `*`
            let (key, computed) = self.parse_property_name();
            let func_id = self.parse_method_function(FunctionKind::Generator);
            let func_span = self.ast().get_function(func_id).span;
            let func_expr = self.ast_mut().alloc_expr(Expr::FunctionExpression {
                span: func_span,
                function: func_id,
            });
            let span = start.cover(func_span);
            return Property {
                span,
                kind: PropertyKind::Init,
                key,
                value: func_expr,
                computed,
                shorthand: false,
                method: true,
            };
        }

        // Regular property or shorthand or method
        let shorthand_token = self.current();
        let shorthand_candidate = self.at_identifier_reference();
        let (key, computed) = self.parse_property_name();

        // Method shorthand: `name() {}`
        if self.at(TokenKind::LParen) {
            let func_id = self.parse_method_function(FunctionKind::Normal);
            let func_span = self.ast().get_function(func_id).span;
            let func_expr = self.ast_mut().alloc_expr(Expr::FunctionExpression {
                span: func_span,
                function: func_id,
            });
            let span = start.cover(func_span);
            return Property {
                span,
                kind: PropertyKind::Init,
                key,
                value: func_expr,
                computed,
                shorthand: false,
                method: true,
            };
        }

        // Regular property: `key: value`
        if self.eat(TokenKind::Colon) {
            let value = self.parse_assignment_expression_allow_in();
            let val_span = self.ast().get_expr(value).span();
            let span = start.cover(val_span);
            return Property {
                span,
                kind: PropertyKind::Init,
                key,
                value,
                computed,
                shorthand: false,
                method: false,
            };
        }

        // Shorthand property: `{ x }` or `{ x = default }`
        if !shorthand_candidate {
            self.error_at(
                self.ast().get_expr(key).span(),
                "expected ':' after property name".to_string(),
            );
        }

        if let Expr::Identifier { name, .. } = self.ast().get_expr(key) {
            self.validate_identifier_reference_atom(*name, shorthand_token.span);
        }

        let span = self.ast().get_expr(key).span();
        let value = if self.eat(TokenKind::Eq) {
            // CoverInitializedName: `{ x = 1 }`
            self.parse_assignment_expression_allow_in()
        } else {
            // Clone the key as the value for shorthand
            key
        };

        Property {
            span,
            kind: PropertyKind::Init,
            key,
            value,
            computed,
            shorthand: true,
            method: false,
        }
    }

    /// Parses a property name and returns (key_expr, computed).
    pub fn parse_property_name(&mut self) -> (ExprId, bool) {
        match self.current_kind() {
            TokenKind::LBracket => {
                self.advance();
                let expr = self.parse_assignment_expression_allow_in();
                self.expect(TokenKind::RBracket);
                (expr, true)
            }
            TokenKind::NumericLiteral => {
                let expr = self.parse_numeric_literal();
                (expr, false)
            }
            TokenKind::StringLiteral => {
                let expr = self.parse_string_literal();
                (expr, false)
            }
            TokenKind::BigIntLiteral => {
                let expr = self.parse_bigint_literal();
                (expr, false)
            }
            _ => {
                // Identifier name (including keywords as property names).
                // Keywords don't carry an Atom payload, so resolve them via
                // the well-known atom table.
                let span = self.current_span();
                let name = if self.current_kind().is_keyword() {
                    let name = self.keyword_to_atom();
                    self.advance();
                    name
                } else {
                    self.parse_identifier_name()
                };
                let expr = self.ast_mut().alloc_expr(Expr::Identifier { span, name });
                (expr, false)
            }
        }
    }

    // -----------------------------------------------------------------------
    // Parenthesized expression / Arrow function
    // -----------------------------------------------------------------------

    pub(super) fn parse_parenthesized_or_arrow_with_async(&mut self, is_async: bool) -> ExprId {
        let start = self.current_span();
        self.advance(); // eat `(`

        // Empty parens: `() =>`
        if self.at(TokenKind::RParen) {
            let end = self.current_span();
            self.advance(); // eat `)`
            if self.at(TokenKind::Arrow) && !self.preceded_by_line_terminator() {
                return self.parse_arrow_function_body(start, &[], None, is_async);
            }
            // Empty parens not followed by arrow is an error
            self.error_at(start.cover(end), "unexpected '()'".to_string());
            return self.ast_mut().alloc_expr(Expr::InvalidExpression {
                span: start.cover(end),
            });
        }

        // Rest parameter: `(...a) =>`
        if self.at(TokenKind::Ellipsis) {
            self.advance();
            let rest_pat = self.parse_binding_pattern();
            self.expect(TokenKind::RParen);
            if self.at(TokenKind::Arrow) && !self.preceded_by_line_terminator() {
                return self.parse_arrow_function_body(start, &[], Some(rest_pat), is_async);
            }
            self.error("expected '=>' after rest parameter".to_string());
            let err_span = start.cover(self.current_span());
            return self
                .ast_mut()
                .alloc_expr(Expr::InvalidExpression { span: err_span });
        }

        // Parse first expression
        let first = self.parse_assignment_expression();

        // Check for comma (multiple expressions = possible arrow params)
        if self.at(TokenKind::Comma) {
            let mut exprs = vec![first];
            while self.eat(TokenKind::Comma) {
                if self.at(TokenKind::Ellipsis) {
                    // Rest in arrow params: `(a, b, ...c) =>`
                    self.advance();
                    let rest_pat = self.parse_binding_pattern();
                    self.expect(TokenKind::RParen);
                    if self.at(TokenKind::Arrow) && !self.preceded_by_line_terminator() {
                        // Convert expressions to patterns
                        let params = self.convert_exprs_to_patterns(&exprs);
                        return self.parse_arrow_function_body(
                            start,
                            &params,
                            Some(rest_pat),
                            is_async,
                        );
                    }
                    self.error("expected '=>'".to_string());
                    let err_span = start.cover(self.current_span());
                    return self
                        .ast_mut()
                        .alloc_expr(Expr::InvalidExpression { span: err_span });
                }
                if self.at(TokenKind::RParen) {
                    break;
                }
                exprs.push(self.parse_assignment_expression());
            }

            let end = self.expect(TokenKind::RParen);

            // Arrow function: `(a, b) =>`
            if self.at(TokenKind::Arrow) && !self.preceded_by_line_terminator() {
                let params = self.convert_exprs_to_patterns(&exprs);
                return self.parse_arrow_function_body(start, &params, None, is_async);
            }

            // Sequence expression
            let first_span = self.ast().get_expr(exprs[0]).span();
            let span = first_span.cover(end.span);
            let list = self.ast_mut().alloc_expr_list(&exprs);
            let seq = self.ast_mut().alloc_expr(Expr::SequenceExpression {
                span,
                expressions: list,
            });
            return self.ast_mut().alloc_expr(Expr::ParenthesizedExpression {
                span: start.cover(end.span),
                expression: seq,
            });
        }

        let end = self.expect(TokenKind::RParen);

        // Arrow function with single parameter: `(x) =>`
        if self.at(TokenKind::Arrow) && !self.preceded_by_line_terminator() {
            let params = self.convert_exprs_to_patterns(&[first]);
            return self.parse_arrow_function_body(start, &params, None, is_async);
        }

        // Regular parenthesized expression
        let span = start.cover(end.span);
        self.ast_mut().alloc_expr(Expr::ParenthesizedExpression {
            span,
            expression: first,
        })
    }

    // -----------------------------------------------------------------------
    // Arrow functions
    // -----------------------------------------------------------------------

    pub(super) fn parse_arrow_function_from_expr(
        &mut self,
        expr: ExprId,
        is_async: bool,
    ) -> ExprId {
        let start = self.ast().get_expr(expr).span();
        let params = self.convert_exprs_to_patterns(&[expr]);
        self.parse_arrow_function_body(start, &params, None, is_async)
    }

    pub(super) fn try_parse_async_arrow_from_cover_call_expression(
        &mut self,
        expr: ExprId,
    ) -> Option<ExprId> {
        let Expr::CallExpression {
            span,
            callee,
            arguments,
        } = self.ast().get_expr(expr).clone()
        else {
            return None;
        };

        let Expr::Identifier { name, .. } = self.ast().get_expr(callee).clone() else {
            return None;
        };
        if name != WellKnownAtom::async_.id() {
            return None;
        }
        let callee_span = self.ast().get_expr(callee).span();
        if self.span_text(callee_span) != "async"
            || self.has_line_terminator_before_first_non_trivia(
                callee_span.range.end.raw(),
                span.range.end.raw(),
            )
        {
            return None;
        }

        let args = self.ast().get_expr_list(arguments).to_vec();
        let mut rest = None;
        let mut rest_end = None;
        let params_end = if let Some(last) = args.last().copied() {
            if let Expr::SpreadElement { argument, .. } = self.ast().get_expr(last).clone() {
                rest_end = Some(self.ast().get_expr(last).span().range.end.raw());
                rest = Some(self.convert_expr_to_pattern(argument));
                args.len().saturating_sub(1)
            } else {
                args.len()
            }
        } else {
            0
        };
        let params = self.convert_exprs_to_patterns(&args[..params_end]);
        if let Some(rest_end) = rest_end {
            if self.has_trailing_comma_before_closing_paren(rest_end, span.range.end.raw()) {
                self.error_at(
                    span,
                    "rest parameter must not be followed by a trailing comma".to_string(),
                );
            }
        }
        Some(self.parse_arrow_function_body(span, &params, rest, true))
    }

    fn parse_arrow_function_body(
        &mut self,
        start: Span,
        params: &[lyng_js_ast::PatternId],
        rest: Option<lyng_js_ast::PatternId>,
        is_async: bool,
    ) -> ExprId {
        self.expect(TokenKind::Arrow); // eat `=>`

        self.validate_arrow_parameters(
            params,
            rest,
            is_async,
            self.allow_await || self.in_static_block,
            self.allow_yield,
        );

        let kind = if is_async {
            FunctionKind::Async
        } else {
            FunctionKind::Arrow
        };

        let params_list = self.ast_mut().alloc_pattern_list(params);
        let formal_params = FormalParameters {
            span: start,
            params: params_list,
            rest,
        };

        let prev_in_func = self.in_function();
        let prev_yield = self.allow_yield;
        let prev_await = self.allow_await;
        let prev_in_static_block = self.in_static_block;
        self.set_in_function(true);
        self.allow_yield = false;
        self.allow_await = is_async;
        self.in_static_block = false;

        if self.at(TokenKind::LBrace) {
            // Block body
            let body_stmts = self.parse_function_body();
            let end_span = self.current_span();
            let span = start.cover(end_span);
            let body = self.ast_mut().alloc_stmt_list(&body_stmts);
            let func = Function {
                span,
                name: None,
                kind,
                params: formal_params,
                body,
                expression_body: None,
            };
            let func_id = self.ast_mut().alloc_function(func);

            self.set_in_function(prev_in_func);
            self.allow_yield = prev_yield;
            self.allow_await = prev_await;
            self.in_static_block = prev_in_static_block;

            self.ast_mut().alloc_expr(Expr::ArrowFunctionExpression {
                span,
                function: func_id,
            })
        } else {
            // Expression body
            let expr = self.parse_assignment_expression();
            let expr_span = self.ast().get_expr(expr).span();
            let span = start.cover(expr_span);
            let body = self.ast_mut().alloc_stmt_list(&[]);
            let func = Function {
                span,
                name: None,
                kind,
                params: formal_params,
                body,
                expression_body: Some(expr),
            };
            let func_id = self.ast_mut().alloc_function(func);

            self.set_in_function(prev_in_func);
            self.allow_yield = prev_yield;
            self.allow_await = prev_await;
            self.in_static_block = prev_in_static_block;

            self.ast_mut().alloc_expr(Expr::ArrowFunctionExpression {
                span,
                function: func_id,
            })
        }
    }

    fn parse_async_arrow_from_single_param(
        &mut self,
        async_span: Span,
        param_span: Span,
        param_name: AtomId,
    ) -> ExprId {
        let pat = self
            .ast_mut()
            .alloc_pattern(lyng_js_ast::Pattern::Identifier {
                span: param_span,
                name: param_name,
            });
        self.parse_arrow_function_body(async_span, &[pat], None, true)
    }

    /// Converts expressions to patterns for arrow function parameters.
    fn convert_exprs_to_patterns(&mut self, exprs: &[ExprId]) -> Vec<lyng_js_ast::PatternId> {
        exprs
            .iter()
            .map(|&expr_id| {
                self.validate_pattern_expression(expr_id, false, false);
                self.convert_expr_to_pattern(expr_id)
            })
            .collect()
    }

    fn validate_arrow_parameters(
        &mut self,
        params: &[lyng_js_ast::PatternId],
        rest: Option<lyng_js_ast::PatternId>,
        is_async: bool,
        inherited_await_restriction: bool,
        inherited_yield_restriction: bool,
    ) {
        let await_restricted = inherited_await_restriction || is_async;
        let yield_restricted = inherited_yield_restriction;
        let mut bound_names = Vec::new();

        for &param in params {
            self.validate_arrow_pattern(
                param,
                await_restricted,
                yield_restricted,
                &mut bound_names,
            );
        }

        if let Some(rest_param) = rest {
            if let lyng_js_ast::Pattern::Assignment { span, .. } =
                self.ast().get_pattern(rest_param)
            {
                self.error_at(
                    *span,
                    "rest parameter must not have a default initializer".to_string(),
                );
            }
            self.validate_arrow_pattern(
                rest_param,
                await_restricted,
                yield_restricted,
                &mut bound_names,
            );
        }
    }

    fn validate_arrow_pattern(
        &mut self,
        pattern_id: lyng_js_ast::PatternId,
        await_restricted: bool,
        yield_restricted: bool,
        bound_names: &mut Vec<AtomId>,
    ) {
        let pattern = self.ast().get_pattern(pattern_id).clone();
        match pattern {
            lyng_js_ast::Pattern::Identifier { span, name } => {
                if await_restricted && name == WellKnownAtom::r#await.id() {
                    self.error_at(
                        span,
                        "reserved word 'await' cannot be used as a binding identifier".to_string(),
                    );
                } else if yield_restricted && name == WellKnownAtom::yield_.id() {
                    self.error_at(
                        span,
                        "reserved word 'yield' cannot be used as a binding identifier".to_string(),
                    );
                }

                if bound_names.contains(&name) {
                    self.error_at(span, "duplicate parameter name".to_string());
                } else {
                    bound_names.push(name);
                }
            }
            lyng_js_ast::Pattern::Object {
                properties, rest, ..
            } => {
                let props = self.ast().get_obj_pattern_prop_list(properties).to_vec();
                for prop in props {
                    if prop.computed {
                        self.validate_arrow_param_expr(
                            prop.key,
                            await_restricted,
                            yield_restricted,
                        );
                    }
                    self.validate_arrow_pattern(
                        prop.value,
                        await_restricted,
                        yield_restricted,
                        bound_names,
                    );
                }
                if let Some(rest_pattern) = rest {
                    self.validate_arrow_pattern(
                        rest_pattern,
                        await_restricted,
                        yield_restricted,
                        bound_names,
                    );
                }
            }
            lyng_js_ast::Pattern::Array { elements, rest, .. } => {
                let elems = self.ast().get_opt_pattern_elem_list(elements).to_vec();
                for elem in elems.into_iter().flatten() {
                    self.validate_arrow_pattern(
                        elem.pattern,
                        await_restricted,
                        yield_restricted,
                        bound_names,
                    );
                }
                if let Some(rest_pattern) = rest {
                    self.validate_arrow_pattern(
                        rest_pattern,
                        await_restricted,
                        yield_restricted,
                        bound_names,
                    );
                }
            }
            lyng_js_ast::Pattern::Assignment { left, right, .. } => {
                self.validate_arrow_pattern(left, await_restricted, yield_restricted, bound_names);
                self.validate_arrow_param_expr(right, await_restricted, yield_restricted);
            }
            lyng_js_ast::Pattern::InvalidPattern { .. } => {}
        }
    }

    fn validate_arrow_param_expr(
        &mut self,
        expr_id: ExprId,
        await_restricted: bool,
        yield_restricted: bool,
    ) {
        let expr = self.ast().get_expr(expr_id).clone();
        match expr {
            Expr::Identifier { span, name } => {
                if await_restricted && name == WellKnownAtom::r#await.id() {
                    self.error_at(
                        span,
                        "reserved word 'await' cannot be used as an identifier reference"
                            .to_string(),
                    );
                } else if yield_restricted && name == WellKnownAtom::yield_.id() {
                    self.error_at(
                        span,
                        "reserved word 'yield' cannot be used as an identifier reference"
                            .to_string(),
                    );
                }
            }
            Expr::ArrayExpression { elements, .. } => {
                let elems = self.ast().get_opt_expr_list(elements).to_vec();
                for elem in elems.into_iter().flatten() {
                    self.validate_arrow_param_expr(elem, await_restricted, yield_restricted);
                }
            }
            Expr::ObjectExpression { properties, .. } => {
                let props = self.ast().get_property_list(properties).to_vec();
                for prop in props {
                    if prop.computed {
                        self.validate_arrow_param_expr(
                            prop.key,
                            await_restricted,
                            yield_restricted,
                        );
                    }
                    self.validate_arrow_param_expr(prop.value, await_restricted, yield_restricted);
                }
            }
            Expr::ArrowFunctionExpression { function, .. } => {
                let function = self.ast().get_function(function).clone();
                let params = self.ast().get_pattern_list(function.params.params).to_vec();
                self.validate_arrow_parameters(
                    &params,
                    function.params.rest,
                    matches!(function.kind, FunctionKind::Async),
                    await_restricted,
                    yield_restricted,
                );
            }
            Expr::UnaryExpression { argument, .. }
            | Expr::UpdateExpression { argument, .. }
            | Expr::AwaitExpression { argument, .. }
            | Expr::SpreadElement { argument, .. }
            | Expr::ParenthesizedExpression {
                expression: argument,
                ..
            } => {
                if await_restricted && matches!(expr, Expr::AwaitExpression { .. }) {
                    self.error_at(
                        expr.span(),
                        "'await' is not allowed in arrow parameters".to_string(),
                    );
                }
                self.validate_arrow_param_expr(argument, await_restricted, yield_restricted);
            }
            Expr::YieldExpression { span, argument, .. } => {
                if yield_restricted {
                    self.error_at(
                        span,
                        "yield expressions are not allowed in arrow parameters".to_string(),
                    );
                }
                if let Some(argument) = argument {
                    self.validate_arrow_param_expr(argument, await_restricted, yield_restricted);
                }
            }
            Expr::BinaryExpression { left, right, .. }
            | Expr::LogicalExpression { left, right, .. }
            | Expr::AssignmentExpression { left, right, .. } => {
                self.validate_arrow_param_expr(left, await_restricted, yield_restricted);
                self.validate_arrow_param_expr(right, await_restricted, yield_restricted);
            }
            Expr::ConditionalExpression {
                test,
                consequent,
                alternate,
                ..
            } => {
                self.validate_arrow_param_expr(test, await_restricted, yield_restricted);
                self.validate_arrow_param_expr(consequent, await_restricted, yield_restricted);
                self.validate_arrow_param_expr(alternate, await_restricted, yield_restricted);
            }
            Expr::SequenceExpression { expressions, .. } => {
                let exprs = self.ast().get_expr_list(expressions).to_vec();
                for expr in exprs {
                    self.validate_arrow_param_expr(expr, await_restricted, yield_restricted);
                }
            }
            Expr::CallExpression {
                callee, arguments, ..
            }
            | Expr::NewExpression {
                callee, arguments, ..
            } => {
                self.validate_arrow_param_expr(callee, await_restricted, yield_restricted);
                let args = self.ast().get_expr_list(arguments).to_vec();
                for arg in args {
                    self.validate_arrow_param_expr(arg, await_restricted, yield_restricted);
                }
            }
            Expr::StaticMemberExpression { object, .. } => {
                self.validate_arrow_param_expr(object, await_restricted, yield_restricted);
            }
            Expr::ComputedMemberExpression {
                object, property, ..
            } => {
                self.validate_arrow_param_expr(object, await_restricted, yield_restricted);
                self.validate_arrow_param_expr(property, await_restricted, yield_restricted);
            }
            Expr::PrivateMemberExpression { object, .. } => {
                self.validate_arrow_param_expr(object, await_restricted, yield_restricted);
            }
            Expr::PrivateInExpression { object, .. } => {
                self.validate_arrow_param_expr(object, await_restricted, yield_restricted);
            }
            Expr::OptionalChainExpression { base, .. } => {
                self.validate_arrow_param_expr(base, await_restricted, yield_restricted);
            }
            Expr::TaggedTemplateExpression { tag, template, .. } => {
                self.validate_arrow_param_expr(tag, await_restricted, yield_restricted);
                let exprs = self.ast().templates().get_expressions(template).to_vec();
                for expr in exprs {
                    self.validate_arrow_param_expr(expr, await_restricted, yield_restricted);
                }
            }
            Expr::TemplateLiteral { template, .. } => {
                let exprs = self.ast().templates().get_expressions(template).to_vec();
                for expr in exprs {
                    self.validate_arrow_param_expr(expr, await_restricted, yield_restricted);
                }
            }
            Expr::ImportExpression {
                source, options, ..
            } => {
                self.validate_arrow_param_expr(source, await_restricted, yield_restricted);
                if let Some(options) = options {
                    self.validate_arrow_param_expr(options, await_restricted, yield_restricted);
                }
            }
            Expr::This { .. }
            | Expr::Super { .. }
            | Expr::NullLiteral { .. }
            | Expr::BooleanLiteral { .. }
            | Expr::NumericLiteral { .. }
            | Expr::StringLiteral { .. }
            | Expr::BigIntLiteral { .. }
            | Expr::RegExpLiteral { .. }
            | Expr::FunctionExpression { .. }
            | Expr::ClassExpression { .. }
            | Expr::MetaProperty { .. }
            | Expr::InvalidExpression { .. } => {}
        }
    }

    pub(crate) fn is_destructuring_pattern_expression(&self, expr_id: ExprId) -> bool {
        match self.ast().get_expr(expr_id) {
            Expr::ArrayExpression { .. } | Expr::ObjectExpression { .. } => true,
            Expr::ParenthesizedExpression { expression, .. } => {
                self.is_destructuring_pattern_expression(*expression)
            }
            _ => false,
        }
    }

    fn validate_pattern_identifier(&mut self, name: AtomId, span: Span) {
        self.validate_identifier_reference_atom(name, span);
        if self.is_strict() {
            if name == WellKnownAtom::eval.id() {
                self.error_at(
                    span,
                    "'eval' is not a valid assignment target in strict mode".to_string(),
                );
            } else if name == WellKnownAtom::arguments.id() {
                self.error_at(
                    span,
                    "'arguments' is not a valid assignment target in strict mode".to_string(),
                );
            }
        }
    }

    pub(crate) fn validate_pattern_expression(
        &mut self,
        expr_id: ExprId,
        in_rest: bool,
        allow_assignment_targets: bool,
    ) {
        let expr = self.ast().get_expr(expr_id).clone();
        match expr {
            Expr::Identifier { span, name } => {
                self.validate_pattern_identifier(name, span);
            }
            Expr::AssignmentExpression {
                operator: AssignOp::Assign,
                left,
                span,
                ..
            } => {
                if in_rest {
                    self.error_at(span, "rest elements cannot have initializers".to_string());
                }
                self.validate_pattern_expression(left, false, allow_assignment_targets);
            }
            Expr::ArrayExpression { elements, .. } => {
                let elems = self.ast().get_opt_expr_list(elements).to_vec();
                let mut seen_rest = false;
                for (idx, opt) in elems.iter().enumerate() {
                    let Some(elem) = opt else {
                        if seen_rest {
                            self.error_at(
                                expr.span(),
                                "rest element must be the last element".to_string(),
                            );
                        }
                        continue;
                    };
                    match self.ast().get_expr(*elem).clone() {
                        Expr::SpreadElement { span, argument } => {
                            if seen_rest || idx + 1 != elems.len() {
                                self.error_at(
                                    span,
                                    "rest element must be the last element".to_string(),
                                );
                            }
                            seen_rest = true;
                            self.validate_pattern_expression(
                                argument,
                                true,
                                allow_assignment_targets,
                            );
                        }
                        _ => {
                            if seen_rest {
                                let span = self.ast().get_expr(*elem).span();
                                self.error_at(
                                    span,
                                    "rest element must be the last element".to_string(),
                                );
                            }
                            self.validate_pattern_expression(
                                *elem,
                                false,
                                allow_assignment_targets,
                            );
                        }
                    }
                }
            }
            Expr::ObjectExpression { properties, .. } => {
                let props = self.ast().get_property_list(properties).to_vec();
                let mut seen_rest = false;
                for (idx, prop) in props.iter().enumerate() {
                    if prop.method {
                        self.error_at(prop.span, "invalid destructuring target".to_string());
                        continue;
                    }

                    if let Expr::SpreadElement { span, argument } =
                        self.ast().get_expr(prop.value).clone()
                    {
                        if prop.key == prop.value {
                            if seen_rest || idx + 1 != props.len() {
                                self.error_at(
                                    span,
                                    "rest property must be the last property".to_string(),
                                );
                            }
                            seen_rest = true;
                            self.validate_pattern_expression(
                                argument,
                                true,
                                allow_assignment_targets,
                            );
                            continue;
                        }
                    }

                    if seen_rest {
                        self.error_at(
                            prop.span,
                            "rest property must be the last property".to_string(),
                        );
                    }

                    if prop.shorthand {
                        match self.ast().get_expr(prop.key).clone() {
                            Expr::Identifier { span, name } => {
                                self.validate_pattern_identifier(name, span);
                            }
                            _ => {
                                self.error_at(
                                    prop.span,
                                    "invalid destructuring target".to_string(),
                                );
                            }
                        }
                    } else {
                        self.validate_pattern_expression(
                            prop.value,
                            false,
                            allow_assignment_targets,
                        );
                    }
                }
            }
            Expr::StaticMemberExpression { .. }
            | Expr::ComputedMemberExpression { .. }
            | Expr::PrivateMemberExpression { .. }
                if allow_assignment_targets => {}
            Expr::ParenthesizedExpression { expression, .. } => {
                self.validate_pattern_expression(expression, in_rest, allow_assignment_targets);
            }
            _ => {
                self.error_at(expr.span(), "invalid destructuring target".to_string());
            }
        }
    }

    fn convert_expr_to_pattern(&mut self, expr_id: ExprId) -> lyng_js_ast::PatternId {
        let expr = self.ast().get_expr(expr_id).clone();
        match expr {
            Expr::Identifier { span, name } => self
                .ast_mut()
                .alloc_pattern(lyng_js_ast::Pattern::Identifier { span, name }),
            Expr::AssignmentExpression {
                span,
                operator: AssignOp::Assign,
                left,
                right,
            } => {
                let left_pat = self.convert_expr_to_pattern(left);
                self.ast_mut()
                    .alloc_pattern(lyng_js_ast::Pattern::Assignment {
                        span,
                        left: left_pat,
                        right,
                    })
            }
            Expr::ArrayExpression { span, elements } => {
                let src_elems = self.ast().get_opt_expr_list(elements).to_vec();
                let mut elems = Vec::new();
                let mut rest = None;

                for opt in src_elems {
                    let Some(elem) = opt else {
                        elems.push(None);
                        continue;
                    };

                    if let Expr::SpreadElement { argument, .. } = self.ast().get_expr(elem).clone()
                    {
                        rest = Some(self.convert_expr_to_pattern(argument));
                        break;
                    }

                    let pat = self.convert_expr_to_pattern(elem);
                    let pat_span = self.ast().get_pattern(pat).span();
                    elems.push(Some(lyng_js_ast::ArrayPatternElement {
                        span: pat_span,
                        pattern: pat,
                    }));
                }

                let list = self.ast_mut().alloc_opt_pattern_elem_list(&elems);
                self.ast_mut().alloc_pattern(lyng_js_ast::Pattern::Array {
                    span,
                    elements: list,
                    rest,
                })
            }
            Expr::ObjectExpression { span, properties } => {
                let src_props = self.ast().get_property_list(properties).to_vec();
                let mut props = Vec::new();
                let mut rest = None;

                for prop in src_props {
                    if let Expr::SpreadElement { argument, .. } =
                        self.ast().get_expr(prop.value).clone()
                    {
                        if prop.key == prop.value {
                            rest = Some(self.convert_expr_to_pattern(argument));
                            break;
                        }
                    }

                    let value_pat =
                        if prop.shorthand {
                            match self.ast().get_expr(prop.key).clone() {
                                Expr::Identifier { span, name } => {
                                    let base = self.ast_mut().alloc_pattern(
                                        lyng_js_ast::Pattern::Identifier { span, name },
                                    );
                                    if prop.value != prop.key {
                                        let right = prop.value;
                                        let right_span = self.ast().get_expr(right).span();
                                        let full_span = span.cover(right_span);
                                        self.ast_mut().alloc_pattern(
                                            lyng_js_ast::Pattern::Assignment {
                                                span: full_span,
                                                left: base,
                                                right,
                                            },
                                        )
                                    } else {
                                        base
                                    }
                                }
                                _ => self.ast_mut().alloc_pattern(
                                    lyng_js_ast::Pattern::InvalidPattern { span: prop.span },
                                ),
                            }
                        } else {
                            self.convert_expr_to_pattern(prop.value)
                        };

                    props.push(lyng_js_ast::ObjectPatternProperty {
                        span: prop.span,
                        key: prop.key,
                        value: value_pat,
                        computed: prop.computed,
                        shorthand: prop.shorthand,
                    });
                }

                let list = self.ast_mut().alloc_obj_pattern_prop_list(&props);
                self.ast_mut().alloc_pattern(lyng_js_ast::Pattern::Object {
                    span,
                    properties: list,
                    rest,
                })
            }
            Expr::SpreadElement { argument, .. } => {
                // Spread in array pattern context becomes rest
                self.convert_expr_to_pattern(argument)
            }
            Expr::ParenthesizedExpression { expression, .. } => {
                self.convert_expr_to_pattern(expression)
            }
            _ => {
                let span = expr.span();
                self.error_at(span, "invalid destructuring target".to_string());
                self.ast_mut()
                    .alloc_pattern(lyng_js_ast::Pattern::InvalidPattern { span })
            }
        }
    }
}

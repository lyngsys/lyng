//! Function and method body parsing.

use lyng_js_ast::{Expr, FormalParameters, Function, FunctionId, FunctionKind, PatternId};
use lyng_js_common::{AtomId, Span};
use lyng_js_lexer::TokenKind;

use crate::parser::Parser;

impl<'src, 'atoms> Parser<'src, 'atoms> {
    /// Parses a function starting from the formal parameters (after `function [*] [name]`).
    ///
    /// This is used by both function declarations and function expressions.
    pub fn parse_function_common(
        &mut self,
        name: Option<AtomId>,
        kind: FunctionKind,
        start: Span,
    ) -> FunctionId {
        let prev_in_func = self.in_function();
        let prev_yield = self.allow_yield;
        let prev_await = self.allow_await;
        let prev_in_static_block = self.in_static_block;

        // Set flags for parameter parsing: await/yield only allowed in
        // async/generator functions, not inherited from enclosing scope.
        self.allow_yield = matches!(kind, FunctionKind::Generator | FunctionKind::AsyncGenerator);
        self.allow_await = matches!(kind, FunctionKind::Async | FunctionKind::AsyncGenerator);
        self.in_static_block = false;

        let params = self.parse_formal_parameters();
        self.validate_function_parameters_for_kind(&params, kind);

        self.set_in_function(true);

        let body_stmts = self.parse_function_body();

        self.set_in_function(prev_in_func);
        self.allow_yield = prev_yield;
        self.allow_await = prev_await;
        self.in_static_block = prev_in_static_block;

        let end = self.current_span();
        let span = start.cover(end);
        let body = self.ast_mut().alloc_stmt_list(&body_stmts);

        self.ast_mut().alloc_function(Function {
            span,
            name,
            kind,
            params,
            body,
            expression_body: None,
        })
    }

    /// Parses a method function (for object literal methods and class methods).
    /// Starts parsing from `(params) { body }`.
    pub fn parse_method_function(&mut self, kind: FunctionKind) -> FunctionId {
        let start = self.current_span();

        let prev_in_func = self.in_function();
        let prev_yield = self.allow_yield;
        let prev_await = self.allow_await;
        let prev_in_static_block = self.in_static_block;

        self.allow_yield = matches!(kind, FunctionKind::Generator | FunctionKind::AsyncGenerator);
        self.allow_await = matches!(kind, FunctionKind::Async | FunctionKind::AsyncGenerator);
        self.in_static_block = false;

        let params = self.parse_formal_parameters();
        self.validate_function_parameters_for_kind(&params, kind);

        self.set_in_function(true);

        let body_stmts = self.parse_function_body();

        self.set_in_function(prev_in_func);
        self.allow_yield = prev_yield;
        self.allow_await = prev_await;
        self.in_static_block = prev_in_static_block;

        let end = self.current_span();
        let span = start.cover(end);
        let body = self.ast_mut().alloc_stmt_list(&body_stmts);

        self.ast_mut().alloc_function(Function {
            span,
            name: None,
            kind,
            params,
            body,
            expression_body: None,
        })
    }

    /// Parses formal parameters: `(p1, p2, ...rest)`.
    fn parse_formal_parameters(&mut self) -> FormalParameters {
        let start = self.current_span();
        self.expect(TokenKind::LParen);

        let mut params = Vec::new();
        let mut rest: Option<PatternId> = None;

        while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
            // Rest parameter
            if self.at(TokenKind::Ellipsis) {
                self.advance();
                rest = Some(self.parse_binding_pattern());
                if self.eat(TokenKind::Comma) {
                    self.error("rest parameter must be the last parameter".to_string());
                }
                break;
            }

            params.push(self.parse_binding_element_for_params());

            if !self.eat(TokenKind::Comma) {
                break;
            }
        }

        let end = self.expect(TokenKind::RParen);
        let span = start.cover(end.span);
        let params_list = self.ast_mut().alloc_pattern_list(&params);

        FormalParameters {
            span,
            params: params_list,
            rest,
        }
    }

    /// Parses a binding element in parameter position (pattern with optional default).
    fn parse_binding_element_for_params(&mut self) -> PatternId {
        let pattern = self.parse_binding_pattern();
        // Optional default initializer: `param = defaultValue`
        if self.eat(TokenKind::Eq) {
            let start = self.ast().get_pattern(pattern).span();
            let default_expr = self.parse_assignment_expression();
            let end = self.ast().get_expr(default_expr).span();
            let span = start.cover(end);
            self.ast_mut()
                .alloc_pattern(lyng_js_ast::Pattern::Assignment {
                    span,
                    left: pattern,
                    right: default_expr,
                })
        } else {
            pattern
        }
    }

    fn validate_function_parameters_for_kind(
        &mut self,
        params: &FormalParameters,
        kind: FunctionKind,
    ) {
        let check_await = matches!(kind, FunctionKind::Async | FunctionKind::AsyncGenerator);
        let check_yield = matches!(kind, FunctionKind::Generator | FunctionKind::AsyncGenerator);

        if !check_await && !check_yield {
            return;
        }

        let param_ids = self.ast().get_pattern_list(params.params).to_vec();
        for param_id in param_ids {
            self.validate_function_parameter_pattern(param_id, check_await, check_yield);
        }

        if let Some(rest) = params.rest {
            self.validate_function_parameter_pattern(rest, check_await, check_yield);
        }
    }

    fn validate_function_parameter_pattern(
        &mut self,
        pattern_id: PatternId,
        check_await: bool,
        check_yield: bool,
    ) {
        match self.ast().get_pattern(pattern_id).clone() {
            lyng_js_ast::Pattern::Identifier { .. }
            | lyng_js_ast::Pattern::InvalidPattern { .. } => {}
            lyng_js_ast::Pattern::Object {
                properties, rest, ..
            } => {
                let props = self.ast().get_obj_pattern_prop_list(properties).to_vec();
                for prop in props {
                    if prop.computed
                        && self.function_parameter_expr_contains_restricted_construct(
                            prop.key,
                            check_await,
                            check_yield,
                        )
                    {
                        return;
                    }
                    self.validate_function_parameter_pattern(prop.value, check_await, check_yield);
                }
                if let Some(rest) = rest {
                    self.validate_function_parameter_pattern(rest, check_await, check_yield);
                }
            }
            lyng_js_ast::Pattern::Array { elements, rest, .. } => {
                let elems = self.ast().get_opt_pattern_elem_list(elements).to_vec();
                for elem in elems.into_iter().flatten() {
                    self.validate_function_parameter_pattern(
                        elem.pattern,
                        check_await,
                        check_yield,
                    );
                }
                if let Some(rest) = rest {
                    self.validate_function_parameter_pattern(rest, check_await, check_yield);
                }
            }
            lyng_js_ast::Pattern::Assignment { left, right, .. } => {
                self.validate_function_parameter_pattern(left, check_await, check_yield);
                self.function_parameter_expr_contains_restricted_construct(
                    right,
                    check_await,
                    check_yield,
                );
            }
        }
    }

    fn function_parameter_expr_contains_restricted_construct(
        &mut self,
        expr_id: lyng_js_ast::ExprId,
        check_await: bool,
        check_yield: bool,
    ) -> bool {
        let expr = self.ast().get_expr(expr_id).clone();
        match expr {
            Expr::AwaitExpression { span, argument } => {
                if check_await {
                    self.error_at(
                        span,
                        "await expressions are not allowed in function parameters".to_string(),
                    );
                    return true;
                }
                self.function_parameter_expr_contains_restricted_construct(
                    argument,
                    check_await,
                    check_yield,
                )
            }
            Expr::YieldExpression { span, argument, .. } => {
                if check_yield {
                    self.error_at(
                        span,
                        "yield expressions are not allowed in function parameters".to_string(),
                    );
                    return true;
                }
                argument.is_some_and(|argument| {
                    self.function_parameter_expr_contains_restricted_construct(
                        argument,
                        check_await,
                        check_yield,
                    )
                })
            }
            Expr::ArrayExpression { elements, .. } => {
                let elems = self.ast().get_opt_expr_list(elements).to_vec();
                elems.into_iter().flatten().any(|elem| {
                    self.function_parameter_expr_contains_restricted_construct(
                        elem,
                        check_await,
                        check_yield,
                    )
                })
            }
            Expr::ObjectExpression { properties, .. } => {
                let props = self.ast().get_property_list(properties).to_vec();
                props.into_iter().any(|prop| {
                    (prop.computed
                        && self.function_parameter_expr_contains_restricted_construct(
                            prop.key,
                            check_await,
                            check_yield,
                        ))
                        || self.function_parameter_expr_contains_restricted_construct(
                            prop.value,
                            check_await,
                            check_yield,
                        )
                })
            }
            Expr::UnaryExpression { argument, .. }
            | Expr::UpdateExpression { argument, .. }
            | Expr::SpreadElement { argument, .. }
            | Expr::ParenthesizedExpression {
                expression: argument,
                ..
            } => self.function_parameter_expr_contains_restricted_construct(
                argument,
                check_await,
                check_yield,
            ),
            Expr::BinaryExpression { left, right, .. }
            | Expr::LogicalExpression { left, right, .. }
            | Expr::AssignmentExpression { left, right, .. } => {
                self.function_parameter_expr_contains_restricted_construct(
                    left,
                    check_await,
                    check_yield,
                ) || self.function_parameter_expr_contains_restricted_construct(
                    right,
                    check_await,
                    check_yield,
                )
            }
            Expr::ConditionalExpression {
                test,
                consequent,
                alternate,
                ..
            } => {
                self.function_parameter_expr_contains_restricted_construct(
                    test,
                    check_await,
                    check_yield,
                ) || self.function_parameter_expr_contains_restricted_construct(
                    consequent,
                    check_await,
                    check_yield,
                ) || self.function_parameter_expr_contains_restricted_construct(
                    alternate,
                    check_await,
                    check_yield,
                )
            }
            Expr::SequenceExpression { expressions, .. } => {
                let exprs = self.ast().get_expr_list(expressions).to_vec();
                exprs.into_iter().any(|expr| {
                    self.function_parameter_expr_contains_restricted_construct(
                        expr,
                        check_await,
                        check_yield,
                    )
                })
            }
            Expr::CallExpression {
                callee, arguments, ..
            }
            | Expr::NewExpression {
                callee, arguments, ..
            } => {
                self.function_parameter_expr_contains_restricted_construct(
                    callee,
                    check_await,
                    check_yield,
                ) || {
                    let args = self.ast().get_expr_list(arguments).to_vec();
                    args.into_iter().any(|arg| {
                        self.function_parameter_expr_contains_restricted_construct(
                            arg,
                            check_await,
                            check_yield,
                        )
                    })
                }
            }
            Expr::StaticMemberExpression { object, .. }
            | Expr::PrivateMemberExpression { object, .. } => self
                .function_parameter_expr_contains_restricted_construct(
                    object,
                    check_await,
                    check_yield,
                ),
            Expr::ComputedMemberExpression {
                object, property, ..
            } => {
                self.function_parameter_expr_contains_restricted_construct(
                    object,
                    check_await,
                    check_yield,
                ) || self.function_parameter_expr_contains_restricted_construct(
                    property,
                    check_await,
                    check_yield,
                )
            }
            Expr::PrivateInExpression { object, .. } => self
                .function_parameter_expr_contains_restricted_construct(
                    object,
                    check_await,
                    check_yield,
                ),
            Expr::OptionalChainExpression { base, .. } => self
                .function_parameter_expr_contains_restricted_construct(
                    base,
                    check_await,
                    check_yield,
                ),
            Expr::TaggedTemplateExpression { tag, template, .. } => {
                self.function_parameter_expr_contains_restricted_construct(
                    tag,
                    check_await,
                    check_yield,
                ) || {
                    let exprs = self.ast().templates().get_expressions(template).to_vec();
                    exprs.into_iter().any(|expr| {
                        self.function_parameter_expr_contains_restricted_construct(
                            expr,
                            check_await,
                            check_yield,
                        )
                    })
                }
            }
            Expr::TemplateLiteral { template, .. } => {
                let exprs = self.ast().templates().get_expressions(template).to_vec();
                exprs.into_iter().any(|expr| {
                    self.function_parameter_expr_contains_restricted_construct(
                        expr,
                        check_await,
                        check_yield,
                    )
                })
            }
            Expr::ImportExpression {
                source, options, ..
            } => {
                self.function_parameter_expr_contains_restricted_construct(
                    source,
                    check_await,
                    check_yield,
                ) || options.is_some_and(|options| {
                    self.function_parameter_expr_contains_restricted_construct(
                        options,
                        check_await,
                        check_yield,
                    )
                })
            }
            Expr::This { .. }
            | Expr::Super { .. }
            | Expr::Identifier { .. }
            | Expr::NullLiteral { .. }
            | Expr::BooleanLiteral { .. }
            | Expr::NumericLiteral { .. }
            | Expr::StringLiteral { .. }
            | Expr::BigIntLiteral { .. }
            | Expr::RegExpLiteral { .. }
            | Expr::FunctionExpression { .. }
            | Expr::ArrowFunctionExpression { .. }
            | Expr::ClassExpression { .. }
            | Expr::MetaProperty { .. }
            | Expr::InvalidExpression { .. } => false,
        }
    }
}

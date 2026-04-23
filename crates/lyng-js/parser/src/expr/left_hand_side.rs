use lyng_js_ast::{Expr, ExprId, NodeList};
use lyng_js_common::WellKnownAtom;
use lyng_js_lexer::TokenKind;

use crate::parser::Parser;

impl<'src, 'atoms> Parser<'src, 'atoms> {
    pub(crate) fn parse_left_hand_side_expression(&mut self) -> ExprId {
        let expr = if self.at(TokenKind::New) {
            self.parse_new_expression()
        } else {
            self.parse_primary_expression()
        };

        self.parse_call_member_chain(expr)
    }

    fn parse_new_expression(&mut self) -> ExprId {
        let start = self.current_span();
        self.advance();

        if self.eat(TokenKind::Dot) {
            let property_token = self.current();
            let property = self.parse_identifier_name();
            if property != WellKnownAtom::target.id() || property_token.contains_escape() {
                self.error_at(
                    property_token.span,
                    "expected 'target' after 'new.'".to_string(),
                );
            }
            let span = start.cover(self.current_span());
            return self.ast_mut().alloc_expr(Expr::MetaProperty {
                span,
                meta: WellKnownAtom::new.id(),
                property,
            });
        }

        let callee = if self.at(TokenKind::New) {
            self.parse_new_expression()
        } else {
            self.parse_primary_expression()
        };

        let mut callee = self.parse_member_chain(callee);
        while matches!(
            self.current_kind(),
            TokenKind::NoSubstitutionTemplate | TokenKind::TemplateHead
        ) {
            let template = self.parse_template_literal(true);
            let callee_span = self.ast().get_expr(callee).span();
            let span = callee_span.cover(self.current_span());
            callee = self.ast_mut().alloc_expr(Expr::TaggedTemplateExpression {
                span,
                tag: callee,
                template,
            });
            callee = self.parse_member_chain(callee);
        }

        if matches!(self.ast().get_expr(callee), Expr::ImportExpression { .. }) {
            let span = self.ast().get_expr(callee).span();
            self.error_at(span, "import() cannot be used as a constructor".to_string());
        }

        let (arguments, end_span) = if self.at(TokenKind::LParen) {
            let args = self.parse_arguments();
            let end = self.current_span();
            (args, end)
        } else {
            let callee_span = self.ast().get_expr(callee).span();
            (self.ast_mut().alloc_expr_list(&[]), callee_span)
        };

        let span = start.cover(end_span);
        let new_expr = self.ast_mut().alloc_expr(Expr::NewExpression {
            span,
            callee,
            arguments,
        });

        self.parse_call_member_chain(new_expr)
    }

    fn parse_member_chain(&mut self, mut object: ExprId) -> ExprId {
        loop {
            match self.current_kind() {
                TokenKind::Dot => {
                    self.advance();
                    if self.at(TokenKind::PrivateIdentifier) {
                        let property = self.current_atom().unwrap_or(WellKnownAtom::Empty.id());
                        let end = self.current_span();
                        self.advance();
                        let obj_span = self.ast().get_expr(object).span();
                        let span = obj_span.cover(end);
                        object = self.ast_mut().alloc_expr(Expr::PrivateMemberExpression {
                            span,
                            object,
                            property,
                        });
                    } else {
                        let property = self.parse_identifier_name();
                        let obj_span = self.ast().get_expr(object).span();
                        let span = obj_span.cover(self.current_span());
                        object = self.ast_mut().alloc_expr(Expr::StaticMemberExpression {
                            span,
                            object,
                            property,
                        });
                    }
                }
                TokenKind::LBracket => {
                    self.advance();
                    let property = self.parse_expression();
                    let end = self.expect(TokenKind::RBracket);
                    let obj_span = self.ast().get_expr(object).span();
                    let span = obj_span.cover(end.span);
                    object = self.ast_mut().alloc_expr(Expr::ComputedMemberExpression {
                        span,
                        object,
                        property,
                    });
                }
                _ => break,
            }
        }
        object
    }

    fn parse_call_member_chain(&mut self, mut expr: ExprId) -> ExprId {
        loop {
            match self.current_kind() {
                TokenKind::Dot => {
                    self.advance();
                    if self.at(TokenKind::PrivateIdentifier) {
                        let property = self.current_atom().unwrap_or(WellKnownAtom::Empty.id());
                        let end = self.current_span();
                        self.advance();
                        let obj_span = self.ast().get_expr(expr).span();
                        let span = obj_span.cover(end);
                        expr = self.ast_mut().alloc_expr(Expr::PrivateMemberExpression {
                            span,
                            object: expr,
                            property,
                        });
                    } else {
                        let property = self.parse_identifier_name();
                        let obj_span = self.ast().get_expr(expr).span();
                        let span = obj_span.cover(self.current_span());
                        expr = self.ast_mut().alloc_expr(Expr::StaticMemberExpression {
                            span,
                            object: expr,
                            property,
                        });
                    }
                }
                TokenKind::LBracket => {
                    self.advance();
                    let property = self.parse_expression();
                    let end = self.expect(TokenKind::RBracket);
                    let obj_span = self.ast().get_expr(expr).span();
                    let span = obj_span.cover(end.span);
                    expr = self.ast_mut().alloc_expr(Expr::ComputedMemberExpression {
                        span,
                        object: expr,
                        property,
                    });
                }
                TokenKind::LParen => {
                    let arguments = self.parse_arguments();
                    let expr_span = self.ast().get_expr(expr).span();
                    let span = expr_span.cover(self.current_span());
                    expr = self.ast_mut().alloc_expr(Expr::CallExpression {
                        span,
                        callee: expr,
                        arguments,
                    });
                }
                TokenKind::OptionalChain => {
                    self.advance();
                    let base_span = self.ast().get_expr(expr).span();
                    match self.current_kind() {
                        TokenKind::PrivateIdentifier => {
                            let property = self.current_atom().unwrap_or(WellKnownAtom::Empty.id());
                            let end = self.current_span();
                            self.advance();
                            let span = base_span.cover(end);
                            let member = self.ast_mut().alloc_expr(Expr::PrivateMemberExpression {
                                span,
                                object: expr,
                                property,
                            });
                            expr = self
                                .ast_mut()
                                .alloc_expr(Expr::OptionalChainExpression { span, base: member });
                        }
                        TokenKind::LParen => {
                            let arguments = self.parse_arguments();
                            let span = base_span.cover(self.current_span());
                            let call = self.ast_mut().alloc_expr(Expr::CallExpression {
                                span,
                                callee: expr,
                                arguments,
                            });
                            expr = self
                                .ast_mut()
                                .alloc_expr(Expr::OptionalChainExpression { span, base: call });
                        }
                        TokenKind::LBracket => {
                            self.advance();
                            let property = self.parse_expression();
                            let end = self.expect(TokenKind::RBracket);
                            let span = base_span.cover(end.span);
                            let member =
                                self.ast_mut().alloc_expr(Expr::ComputedMemberExpression {
                                    span,
                                    object: expr,
                                    property,
                                });
                            expr = self
                                .ast_mut()
                                .alloc_expr(Expr::OptionalChainExpression { span, base: member });
                        }
                        _ => {
                            let property = self.parse_identifier_name();
                            let span = base_span.cover(self.current_span());
                            let member = self.ast_mut().alloc_expr(Expr::StaticMemberExpression {
                                span,
                                object: expr,
                                property,
                            });
                            expr = self
                                .ast_mut()
                                .alloc_expr(Expr::OptionalChainExpression { span, base: member });
                        }
                    }
                }
                TokenKind::NoSubstitutionTemplate | TokenKind::TemplateHead => {
                    if matches!(
                        self.ast().get_expr(expr),
                        Expr::OptionalChainExpression { .. }
                    ) {
                        self.error_at(
                            self.current_span(),
                            "tagged templates are not allowed in optional chains".to_string(),
                        );
                    }
                    let template = self.parse_template_literal(true);
                    let expr_span = self.ast().get_expr(expr).span();
                    let span = expr_span.cover(self.current_span());
                    expr = self.ast_mut().alloc_expr(Expr::TaggedTemplateExpression {
                        span,
                        tag: expr,
                        template,
                    });
                }
                _ => break,
            }
        }
        expr
    }

    fn parse_arguments(&mut self) -> NodeList<ExprId> {
        self.expect(TokenKind::LParen);
        let mut args = Vec::new();

        while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
            if self.at(TokenKind::Ellipsis) {
                let start = self.current_span();
                self.advance();
                let argument = self.parse_assignment_expression();
                let arg_span = self.ast().get_expr(argument).span();
                let span = start.cover(arg_span);
                let spread = self
                    .ast_mut()
                    .alloc_expr(Expr::SpreadElement { span, argument });
                args.push(spread);
            } else {
                args.push(self.parse_assignment_expression());
            }
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::RParen);
        self.ast_mut().alloc_expr_list(&args)
    }
}

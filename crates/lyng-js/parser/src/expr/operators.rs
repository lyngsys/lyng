use lyng_js_ast::{AssignOp, BinaryOp, Expr, ExprId, LogicalOp, UnaryOp, UpdateOp};
use lyng_js_common::WellKnownAtom;
use lyng_js_lexer::TokenKind;

use crate::parser::Parser;

/// Binding power (precedence) for binary/logical operators.
/// Higher values bind tighter.
pub(super) fn binary_precedence(kind: TokenKind, no_in: bool) -> Option<u8> {
    match kind {
        TokenKind::PipePipe => Some(4),
        TokenKind::AmpAmp => Some(5),
        TokenKind::QuestionQuestion => Some(4),
        TokenKind::Pipe => Some(6),
        TokenKind::Caret => Some(7),
        TokenKind::Amp => Some(8),
        TokenKind::EqEq | TokenKind::NotEq | TokenKind::EqEqEq | TokenKind::NotEqEq => Some(9),
        TokenKind::Lt
        | TokenKind::Gt
        | TokenKind::LtEq
        | TokenKind::GtEq
        | TokenKind::Instanceof => Some(10),
        TokenKind::In if !no_in => Some(10),
        TokenKind::LtLt | TokenKind::GtGt | TokenKind::GtGtGt => Some(11),
        TokenKind::Plus | TokenKind::Minus => Some(12),
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some(13),
        TokenKind::Exp => Some(14),
        _ => None,
    }
}

pub(super) fn token_to_binary_op(kind: TokenKind) -> Option<BinaryOp> {
    match kind {
        TokenKind::Plus => Some(BinaryOp::Add),
        TokenKind::Minus => Some(BinaryOp::Sub),
        TokenKind::Star => Some(BinaryOp::Mul),
        TokenKind::Slash => Some(BinaryOp::Div),
        TokenKind::Percent => Some(BinaryOp::Rem),
        TokenKind::Exp => Some(BinaryOp::Exp),
        TokenKind::Lt => Some(BinaryOp::Lt),
        TokenKind::Gt => Some(BinaryOp::Gt),
        TokenKind::LtEq => Some(BinaryOp::LtEq),
        TokenKind::GtEq => Some(BinaryOp::GtEq),
        TokenKind::EqEq => Some(BinaryOp::Eq),
        TokenKind::NotEq => Some(BinaryOp::NotEq),
        TokenKind::EqEqEq => Some(BinaryOp::StrictEq),
        TokenKind::NotEqEq => Some(BinaryOp::StrictNotEq),
        TokenKind::LtLt => Some(BinaryOp::Shl),
        TokenKind::GtGt => Some(BinaryOp::Shr),
        TokenKind::GtGtGt => Some(BinaryOp::UShr),
        TokenKind::Amp => Some(BinaryOp::BitAnd),
        TokenKind::Pipe => Some(BinaryOp::BitOr),
        TokenKind::Caret => Some(BinaryOp::BitXor),
        TokenKind::In => Some(BinaryOp::In),
        TokenKind::Instanceof => Some(BinaryOp::Instanceof),
        _ => None,
    }
}

pub(super) fn token_to_logical_op(kind: TokenKind) -> Option<LogicalOp> {
    match kind {
        TokenKind::AmpAmp => Some(LogicalOp::And),
        TokenKind::PipePipe => Some(LogicalOp::Or),
        TokenKind::QuestionQuestion => Some(LogicalOp::NullishCoalescing),
        _ => None,
    }
}

pub(super) fn token_to_assign_op(kind: TokenKind) -> Option<AssignOp> {
    match kind {
        TokenKind::Eq => Some(AssignOp::Assign),
        TokenKind::PlusEq => Some(AssignOp::AddAssign),
        TokenKind::MinusEq => Some(AssignOp::SubAssign),
        TokenKind::StarEq => Some(AssignOp::MulAssign),
        TokenKind::SlashEq => Some(AssignOp::DivAssign),
        TokenKind::PercentEq => Some(AssignOp::RemAssign),
        TokenKind::ExpEq => Some(AssignOp::ExpAssign),
        TokenKind::LtLtEq => Some(AssignOp::ShlAssign),
        TokenKind::GtGtEq => Some(AssignOp::ShrAssign),
        TokenKind::GtGtGtEq => Some(AssignOp::UShrAssign),
        TokenKind::AmpEq => Some(AssignOp::BitAndAssign),
        TokenKind::PipeEq => Some(AssignOp::BitOrAssign),
        TokenKind::CaretEq => Some(AssignOp::BitXorAssign),
        TokenKind::AmpAmpEq => Some(AssignOp::AndAssign),
        TokenKind::PipePipeEq => Some(AssignOp::OrAssign),
        TokenKind::QuestionQuestionEq => Some(AssignOp::NullishAssign),
        _ => None,
    }
}

pub(super) fn token_can_start_expression(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Identifier
            | TokenKind::PrivateIdentifier
            | TokenKind::Null
            | TokenKind::True
            | TokenKind::False
            | TokenKind::NumericLiteral
            | TokenKind::StringLiteral
            | TokenKind::BigIntLiteral
            | TokenKind::RegExpLiteral
            | TokenKind::LBracket
            | TokenKind::LBrace
            | TokenKind::Function
            | TokenKind::Class
            | TokenKind::NoSubstitutionTemplate
            | TokenKind::TemplateHead
            | TokenKind::LParen
            | TokenKind::Import
            | TokenKind::This
            | TokenKind::Super
            | TokenKind::New
            | TokenKind::Delete
            | TokenKind::Void
            | TokenKind::Typeof
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Tilde
            | TokenKind::Bang
            | TokenKind::PlusPlus
            | TokenKind::MinusMinus
            | TokenKind::Await
            | TokenKind::Yield
            | TokenKind::Slash
            | TokenKind::SlashEq
    )
}

impl<'src, 'atoms> Parser<'src, 'atoms> {
    pub(super) fn parse_conditional_expression(&mut self) -> ExprId {
        let expr = self.parse_binary_expression(0);

        if self.eat(TokenKind::Question) {
            let consequent = self.parse_assignment_expression_allow_in();
            self.expect(TokenKind::Colon);
            let alternate = self.parse_assignment_expression();

            let left_span = self.ast().get_expr(expr).span();
            let right_span = self.ast().get_expr(alternate).span();
            let span = left_span.cover(right_span);
            return self.ast_mut().alloc_expr(Expr::ConditionalExpression {
                span,
                test: expr,
                consequent,
                alternate,
            });
        }

        expr
    }

    pub(super) fn parse_binary_expression(&mut self, min_prec: u8) -> ExprId {
        let mut left = if self.at(TokenKind::PrivateIdentifier) && !self.no_in() && min_prec <= 10 {
            self.parse_private_in_expression()
        } else {
            self.parse_unary_expression()
        };

        loop {
            let kind = self.current_kind();
            let prec = match binary_precedence(kind, self.no_in()) {
                Some(p) if p >= min_prec => p,
                _ => break,
            };

            let next_min = if kind == TokenKind::Exp {
                prec
            } else {
                prec + 1
            };

            let op_kind = kind;
            self.advance();
            let right = self.parse_binary_expression(next_min);

            if op_kind == TokenKind::Exp && self.expr_is_unparenthesized_unary_base(left) {
                let span = self.ast().get_expr(left).span();
                self.error_at(
                    span,
                    "unparenthesized unary expression cannot appear on the left-hand side of '**'"
                        .to_string(),
                );
            }

            if (op_kind == TokenKind::QuestionQuestion
                && (self.expr_is_logical_and_or(left) || self.expr_is_logical_and_or(right)))
                || ((op_kind == TokenKind::AmpAmp || op_kind == TokenKind::PipePipe)
                    && (self.expr_is_nullish_coalescing(left)
                        || self.expr_is_nullish_coalescing(right)))
            {
                let span = self
                    .ast()
                    .get_expr(left)
                    .span()
                    .cover(self.ast().get_expr(right).span());
                self.error_at(
                    span,
                    "cannot mix '??' with '&&' or '||' without parentheses".to_string(),
                );
            }

            let left_span = self.ast().get_expr(left).span();
            let right_span = self.ast().get_expr(right).span();
            let span = left_span.cover(right_span);

            left = if let Some(log_op) = token_to_logical_op(op_kind) {
                self.ast_mut().alloc_expr(Expr::LogicalExpression {
                    span,
                    operator: log_op,
                    left,
                    right,
                })
            } else {
                let bin_op = token_to_binary_op(op_kind).unwrap_or(BinaryOp::Add);
                self.ast_mut().alloc_expr(Expr::BinaryExpression {
                    span,
                    operator: bin_op,
                    left,
                    right,
                })
            };
        }

        left
    }

    pub(super) fn parse_private_in_expression(&mut self) -> ExprId {
        let start = self.current_span();
        let property = self.current_atom().unwrap_or(WellKnownAtom::Empty.id());
        self.advance();

        if !self.eat(TokenKind::In) {
            self.error_at(
                start,
                "private identifiers can only appear in 'in' expressions".to_string(),
            );
            return self
                .ast_mut()
                .alloc_expr(Expr::InvalidExpression { span: start });
        }

        let object = self.parse_binary_expression(11);
        if matches!(
            self.ast().get_expr(object),
            Expr::ArrowFunctionExpression { .. }
        ) {
            self.error_at(
                self.ast().get_expr(object).span(),
                "private 'in' expressions require a shift expression on the right-hand side"
                    .to_string(),
            );
        }
        let span = start.cover(self.ast().get_expr(object).span());
        self.ast_mut().alloc_expr(Expr::PrivateInExpression {
            span,
            property,
            object,
        })
    }

    pub(super) fn parse_unary_expression(&mut self) -> ExprId {
        match self.current_kind() {
            TokenKind::Delete => self.parse_unary_op(UnaryOp::Delete),
            TokenKind::Void => self.parse_unary_op(UnaryOp::Void),
            TokenKind::Typeof => self.parse_unary_op(UnaryOp::TypeOf),
            TokenKind::Plus => self.parse_unary_op(UnaryOp::Plus),
            TokenKind::Minus => self.parse_unary_op(UnaryOp::Minus),
            TokenKind::Tilde => self.parse_unary_op(UnaryOp::BitNot),
            TokenKind::Bang => self.parse_unary_op(UnaryOp::Not),
            TokenKind::PlusPlus | TokenKind::MinusMinus => self.parse_prefix_update(),
            TokenKind::Await if self.allow_await => self.parse_await_expression(),
            _ => self.parse_postfix_expression(),
        }
    }

    fn parse_unary_op(&mut self, op: UnaryOp) -> ExprId {
        let start = self.current_span();
        self.advance();
        let argument = self.parse_unary_expression();
        if op == UnaryOp::Delete
            && self.is_strict()
            && self.expr_is_unqualified_delete_target(argument)
        {
            self.error_at(
                start.cover(self.ast().get_expr(argument).span()),
                "delete of an unqualified identifier in strict mode".to_string(),
            );
        }
        let arg_span = self.ast().get_expr(argument).span();
        let span = start.cover(arg_span);
        self.ast_mut().alloc_expr(Expr::UnaryExpression {
            span,
            operator: op,
            argument,
        })
    }

    fn parse_prefix_update(&mut self) -> ExprId {
        let start = self.current_span();
        let op = if self.current_kind() == TokenKind::PlusPlus {
            UpdateOp::Increment
        } else {
            UpdateOp::Decrement
        };
        self.advance();
        let argument = self.parse_unary_expression();
        if !self.is_simple_assignment_target(argument) {
            let arg_span = self.ast().get_expr(argument).span();
            self.error_at(arg_span, "invalid prefix update operand".to_string());
        }
        let arg_span = self.ast().get_expr(argument).span();
        let span = start.cover(arg_span);
        self.ast_mut().alloc_expr(Expr::UpdateExpression {
            span,
            operator: op,
            argument,
            prefix: true,
        })
    }

    fn parse_await_expression(&mut self) -> ExprId {
        let start = self.current_span();
        self.advance();
        let argument = self.parse_unary_expression();
        let arg_span = self.ast().get_expr(argument).span();
        let span = start.cover(arg_span);
        self.ast_mut()
            .alloc_expr(Expr::AwaitExpression { span, argument })
    }

    fn parse_postfix_expression(&mut self) -> ExprId {
        let expr = self.parse_left_hand_side_expression();

        if !self.preceded_by_line_terminator() {
            match self.current_kind() {
                TokenKind::PlusPlus | TokenKind::MinusMinus => {
                    if !self.is_simple_assignment_target(expr) {
                        let span = self.ast().get_expr(expr).span();
                        self.error_at(span, "invalid update expression operand".to_string());
                    }
                    let op = if self.current_kind() == TokenKind::PlusPlus {
                        UpdateOp::Increment
                    } else {
                        UpdateOp::Decrement
                    };
                    let end = self.current_span();
                    self.advance();
                    let expr_span = self.ast().get_expr(expr).span();
                    let span = expr_span.cover(end);
                    return self.ast_mut().alloc_expr(Expr::UpdateExpression {
                        span,
                        operator: op,
                        argument: expr,
                        prefix: false,
                    });
                }
                _ => {}
            }
        }

        expr
    }
}

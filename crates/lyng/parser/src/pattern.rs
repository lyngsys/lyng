//! Pattern (destructuring) parsing.

use lyng_js_ast::{ArrayPatternElement, Expr, ObjectPatternProperty, Pattern, PatternId};
use lyng_js_lexer::TokenKind;

use crate::parser::Parser;

impl<'src, 'atoms> Parser<'src, 'atoms> {
    /// Parses a binding pattern: identifier, object pattern, or array pattern.
    pub fn parse_binding_pattern(&mut self) -> PatternId {
        match self.current_kind() {
            TokenKind::LBrace => self.parse_object_pattern(),
            TokenKind::LBracket => self.parse_array_pattern(),
            _ => self.parse_identifier_pattern(),
        }
    }

    /// Parses an identifier pattern (without default value).
    fn parse_identifier_pattern(&mut self) -> PatternId {
        let span = self.current_span();
        let name = self.parse_binding_identifier();
        self.ast_mut()
            .alloc_pattern(Pattern::Identifier { span, name })
    }

    /// Parses an object destructuring pattern: `{ a, b: c, ...rest }`.
    fn parse_object_pattern(&mut self) -> PatternId {
        let start = self.current_span();
        self.advance(); // eat `{`

        let mut properties = Vec::new();
        let mut rest = None;

        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            // Rest element: `...rest`
            if self.at(TokenKind::Ellipsis) {
                self.advance();
                rest = Some(self.parse_binding_pattern());
                if self.eat(TokenKind::Comma) {
                    self.error("rest property must be the last property".to_string());
                }
                break;
            }

            properties.push(self.parse_object_pattern_property());

            if !self.eat(TokenKind::Comma) {
                break;
            }
        }

        let end = self.expect(TokenKind::RBrace);
        let span = start.cover(end.span);
        let list = self.ast_mut().alloc_obj_pattern_prop_list(&properties);
        self.ast_mut().alloc_pattern(Pattern::Object {
            span,
            properties: list,
            rest,
        })
    }

    fn parse_object_pattern_property(&mut self) -> ObjectPatternProperty {
        let start = self.current_span();

        // Computed key: `[expr]: pattern`
        if self.at(TokenKind::LBracket) {
            self.advance();
            let key = self.parse_assignment_expression();
            self.expect(TokenKind::RBracket);
            self.expect(TokenKind::Colon);
            let value = self.parse_binding_element();
            let val_span = self.ast().get_pattern(value).span();
            let span = start.cover(val_span);
            return ObjectPatternProperty {
                span,
                key,
                value,
                computed: true,
                shorthand: false,
            };
        }

        // String, numeric, or bigint key
        if self.at(TokenKind::StringLiteral)
            || self.at(TokenKind::NumericLiteral)
            || self.at(TokenKind::BigIntLiteral)
        {
            let key = if self.at(TokenKind::StringLiteral) {
                self.parse_string_literal()
            } else if self.at(TokenKind::BigIntLiteral) {
                self.parse_bigint_literal()
            } else {
                self.parse_numeric_literal()
            };
            self.expect(TokenKind::Colon);
            let value = self.parse_binding_element();
            let val_span = self.ast().get_pattern(value).span();
            let span = start.cover(val_span);
            return ObjectPatternProperty {
                span,
                key,
                value,
                computed: false,
                shorthand: false,
            };
        }

        // IdentifierName key — could be shorthand or `key: value`
        let key_token = self.current();
        let shorthand_candidate = self.at_identifier_reference();
        let key_span = self.current_span();
        let key_name = self.parse_identifier_name();
        let key = self.ast_mut().alloc_expr(Expr::Identifier {
            span: key_span,
            name: key_name,
        });

        if self.eat(TokenKind::Colon) {
            // `key: pattern`
            let value = self.parse_binding_element();
            let val_span = self.ast().get_pattern(value).span();
            let span = start.cover(val_span);
            ObjectPatternProperty {
                span,
                key,
                value,
                computed: false,
                shorthand: false,
            }
        } else {
            if !shorthand_candidate {
                self.error_at(key_span, "expected ':' after property name".to_string());
            }
            if key_token.kind == TokenKind::Identifier && key_token.contains_escape() {
                self.validate_identifier_reference_atom(key_name, key_span);
            }
            // Shorthand: `{ x }` or `{ x = default }`
            let value = self.ast_mut().alloc_pattern(Pattern::Identifier {
                span: key_span,
                name: key_name,
            });

            let value = if self.eat(TokenKind::Eq) {
                let right = self.parse_assignment_expression();
                let right_span = self.ast().get_expr(right).span();
                let full_span = key_span.cover(right_span);
                self.ast_mut().alloc_pattern(Pattern::Assignment {
                    span: full_span,
                    left: value,
                    right,
                })
            } else {
                value
            };

            let val_span = self.ast().get_pattern(value).span();
            let span = start.cover(val_span);
            ObjectPatternProperty {
                span,
                key,
                value,
                computed: false,
                shorthand: true,
            }
        }
    }

    /// Parses an array destructuring pattern: `[a, , b, ...rest]`.
    fn parse_array_pattern(&mut self) -> PatternId {
        let start = self.current_span();
        self.advance(); // eat `[`

        let mut elements: Vec<Option<ArrayPatternElement>> = Vec::new();
        let mut rest = None;

        while !self.at(TokenKind::RBracket) && !self.at(TokenKind::Eof) {
            // Elision
            if self.at(TokenKind::Comma) {
                elements.push(None);
                self.advance();
                continue;
            }

            // Rest element
            if self.at(TokenKind::Ellipsis) {
                self.advance();
                rest = Some(self.parse_binding_pattern());
                if self.eat(TokenKind::Comma) {
                    self.error("rest element must be the last element".to_string());
                }
                break;
            }

            let elem_pat = self.parse_binding_element();
            let elem_span = self.ast().get_pattern(elem_pat).span();
            elements.push(Some(ArrayPatternElement {
                span: elem_span,
                pattern: elem_pat,
            }));

            if !self.eat(TokenKind::Comma) {
                break;
            }
        }

        let end = self.expect(TokenKind::RBracket);
        let span = start.cover(end.span);
        let list = self.ast_mut().alloc_opt_pattern_elem_list(&elements);
        self.ast_mut().alloc_pattern(Pattern::Array {
            span,
            elements: list,
            rest,
        })
    }

    /// Parses a binding element (pattern with optional default).
    fn parse_binding_element(&mut self) -> PatternId {
        let pat = self.parse_binding_pattern();
        if self.eat(TokenKind::Eq) {
            let right = self.parse_assignment_expression();
            let pat_span = self.ast().get_pattern(pat).span();
            let right_span = self.ast().get_expr(right).span();
            let span = pat_span.cover(right_span);
            self.ast_mut().alloc_pattern(Pattern::Assignment {
                span,
                left: pat,
                right,
            })
        } else {
            pat
        }
    }
}

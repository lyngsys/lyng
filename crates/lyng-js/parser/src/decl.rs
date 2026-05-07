//! Declaration parsing (variable, function, class, import, export).

use lyng_js_ast::{
    ClassElement, Decl, DeclId, Expr, FunctionKind, MethodKind, NodeList, Pattern, Stmt, StmtId,
    VariableDeclarator, VariableKind,
};
use lyng_js_common::WellKnownAtom;
use lyng_js_lexer::TokenKind;

use crate::parser::Parser;

impl<'src, 'atoms> Parser<'src, 'atoms> {
    // -----------------------------------------------------------------------
    // Variable declarations
    // -----------------------------------------------------------------------

    pub fn parse_var_declaration_stmt(&mut self) -> StmtId {
        let decl = self.parse_variable_declaration(VariableKind::Var);
        let span = self.ast().get_decl(decl).span();
        self.expect_semicolon();
        self.ast_mut().alloc_stmt(Stmt::Declaration { span, decl })
    }

    pub fn parse_lexical_declaration_stmt(&mut self) -> StmtId {
        let kind = if self.at(TokenKind::Const) {
            VariableKind::Const
        } else {
            VariableKind::Let
        };
        let decl = self.parse_variable_declaration(kind);
        let span = self.ast().get_decl(decl).span();
        self.expect_semicolon();
        self.ast_mut().alloc_stmt(Stmt::Declaration { span, decl })
    }

    pub fn parse_using_declaration_stmt(&mut self, kind: VariableKind) -> StmtId {
        debug_assert!(matches!(
            kind,
            VariableKind::Using | VariableKind::AwaitUsing
        ));
        if kind == VariableKind::AwaitUsing && !self.allow_await && !self.is_module() {
            self.error("'await using' is only allowed in async functions and modules".to_string());
        }
        if self.in_direct_switch_clause_statement_list() {
            self.error(
                "using declarations are not allowed directly in switch case clauses".to_string(),
            );
        }
        let decl = self.parse_variable_declaration(kind);
        let span = self.ast().get_decl(decl).span();
        self.expect_semicolon();
        self.ast_mut().alloc_stmt(Stmt::Declaration { span, decl })
    }

    fn parse_variable_declaration(&mut self, kind: VariableKind) -> DeclId {
        let start = self.current_span();
        if kind == VariableKind::AwaitUsing {
            self.advance(); // eat `await`
        }
        self.advance(); // eat declaration keyword

        let declarators = self.parse_variable_declarator_list();
        if matches!(
            kind,
            VariableKind::Const | VariableKind::Using | VariableKind::AwaitUsing
        ) {
            self.validate_const_declarators(&declarators);
        }
        if kind != VariableKind::Var {
            self.validate_lexical_declarator_names(&declarators);
        }
        if matches!(kind, VariableKind::Using | VariableKind::AwaitUsing) {
            self.validate_using_declarators(&declarators, false);
            if self.in_program_statement_list() && !self.is_module() {
                self.error_at(
                    start,
                    "using declarations are not allowed at the top level of a script".to_string(),
                );
            }
        }

        let last_span = declarators.last().map(|d| d.span).unwrap_or(start);
        let span = start.cover(last_span);
        let list = self.ast_mut().alloc_var_declarator_list(&declarators);
        self.ast_mut().alloc_decl(Decl::Variable {
            span,
            kind,
            declarators: list,
        })
    }

    pub fn parse_variable_declarator_list(&mut self) -> Vec<VariableDeclarator> {
        let mut declarators = Vec::new();
        declarators.push(self.parse_variable_declarator());
        while self.eat(TokenKind::Comma) {
            declarators.push(self.parse_variable_declarator());
        }
        declarators
    }

    fn parse_variable_declarator(&mut self) -> VariableDeclarator {
        let start = self.current_span();
        let id = self.parse_binding_pattern();
        let init = if self.eat(TokenKind::Eq) {
            Some(self.parse_assignment_expression())
        } else {
            None
        };
        let end = init
            .map(|e| self.ast().get_expr(e).span())
            .unwrap_or_else(|| self.ast().get_pattern(id).span());
        let span = start.cover(end);
        VariableDeclarator { span, id, init }
    }

    pub(crate) fn validate_regular_for_declaration(&mut self, decl: DeclId) {
        let declarators = match self.ast().get_decl(decl) {
            Decl::Variable {
                kind: VariableKind::Const | VariableKind::Using | VariableKind::AwaitUsing,
                declarators,
                ..
            } => self.ast().get_var_declarator_list(*declarators).to_vec(),
            _ => return,
        };

        self.validate_const_declarators(&declarators);
    }

    pub(crate) fn validate_lexical_declarator_names(&mut self, declarators: &[VariableDeclarator]) {
        for declarator in declarators {
            self.validate_lexical_binding_pattern(declarator.id);
        }
    }

    fn validate_const_declarators(&mut self, declarators: &[VariableDeclarator]) {
        for declarator in declarators {
            if declarator.init.is_none() {
                let span = self.ast().get_pattern(declarator.id).span();
                self.error_at(span, "missing initializer in const declaration".to_string());
            }
        }
    }

    pub(crate) fn validate_using_declarators(
        &mut self,
        declarators: &[VariableDeclarator],
        allow_missing_initializer: bool,
    ) {
        for declarator in declarators {
            if !matches!(
                self.ast().get_pattern(declarator.id),
                Pattern::Identifier { .. }
            ) {
                let span = self.ast().get_pattern(declarator.id).span();
                self.error_at(
                    span,
                    "using declarations require simple identifier bindings".to_string(),
                );
            }
            if !allow_missing_initializer && declarator.init.is_none() {
                let span = self.ast().get_pattern(declarator.id).span();
                self.error_at(span, "missing initializer in using declaration".to_string());
            }
        }
    }

    fn validate_lexical_binding_pattern(&mut self, pattern_id: lyng_js_ast::PatternId) {
        match self.ast().get_pattern(pattern_id).clone() {
            Pattern::Identifier { name, span } => {
                if name == WellKnownAtom::let_.id() {
                    self.error_at(
                        span,
                        "'let' is not allowed as a lexical binding name".to_string(),
                    );
                }
            }
            Pattern::Object {
                properties, rest, ..
            } => {
                let properties = self.ast().get_obj_pattern_prop_list(properties).to_vec();
                for property in properties {
                    self.validate_lexical_binding_pattern(property.value);
                }
                if let Some(rest) = rest {
                    self.validate_lexical_binding_pattern(rest);
                }
            }
            Pattern::Array { elements, rest, .. } => {
                let elements = self.ast().get_opt_pattern_elem_list(elements).to_vec();
                for element in elements.into_iter().flatten() {
                    self.validate_lexical_binding_pattern(element.pattern);
                }
                if let Some(rest) = rest {
                    self.validate_lexical_binding_pattern(rest);
                }
            }
            Pattern::Assignment { left, .. } => {
                self.validate_lexical_binding_pattern(left);
            }
            Pattern::InvalidPattern { .. } => {}
        }
    }

    // -----------------------------------------------------------------------
    // Function declarations
    // -----------------------------------------------------------------------

    pub fn parse_function_declaration_stmt(&mut self) -> StmtId {
        let start = self.current_span();
        self.advance(); // eat `function`

        let is_generator = self.eat(TokenKind::Star);

        let name = if self.at(TokenKind::Identifier)
            || self.at(TokenKind::Yield)
            || self.at(TokenKind::Await)
        {
            Some(self.parse_binding_identifier())
        } else {
            self.error("expected function name".to_string());
            None
        };

        let kind = if is_generator {
            FunctionKind::Generator
        } else {
            FunctionKind::Normal
        };

        let func_id = self.parse_function_common(name, kind, start);
        let span = self.ast().get_function(func_id).span;
        let decl = self.ast_mut().alloc_decl(Decl::Function {
            span,
            function: func_id,
        });
        self.ast_mut().alloc_stmt(Stmt::Declaration { span, decl })
    }

    pub fn parse_async_function_declaration_stmt(&mut self) -> StmtId {
        let start = self.current_span();
        self.advance(); // eat `async`
        self.advance(); // eat `function`

        let is_generator = self.eat(TokenKind::Star);

        let name = if self.at(TokenKind::Identifier)
            || self.at(TokenKind::Yield)
            || self.at(TokenKind::Await)
        {
            Some(self.parse_binding_identifier())
        } else {
            self.error("expected function name".to_string());
            None
        };

        let kind = if is_generator {
            FunctionKind::AsyncGenerator
        } else {
            FunctionKind::Async
        };

        let func_id = self.parse_function_common(name, kind, start);
        let span = self.ast().get_function(func_id).span;
        let decl = self.ast_mut().alloc_decl(Decl::Function {
            span,
            function: func_id,
        });
        self.ast_mut().alloc_stmt(Stmt::Declaration { span, decl })
    }

    // -----------------------------------------------------------------------
    // Class declarations
    // -----------------------------------------------------------------------

    pub fn parse_class_declaration_stmt(&mut self) -> StmtId {
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

        let decl = self.ast_mut().alloc_decl(Decl::Class {
            span,
            name,
            super_class,
            body,
        });
        self.ast_mut().alloc_stmt(Stmt::Declaration { span, decl })
    }

    pub(crate) fn parse_decorator_list_syntax_only(&mut self) {
        while self.eat(TokenKind::At) {
            self.parse_decorator_syntax_only();
        }
    }

    fn parse_decorator_syntax_only(&mut self) {
        if self.eat(TokenKind::LParen) {
            if self.at(TokenKind::RParen) {
                self.error("expected decorator expression".to_string());
            } else {
                self.parse_expression();
            }
            self.expect(TokenKind::RParen);
            return;
        }

        self.parse_decorator_member_expression_syntax_only();
        if self.at(TokenKind::LParen) {
            self.parse_decorator_arguments_syntax_only();
        }
    }

    fn parse_decorator_member_expression_syntax_only(&mut self) {
        self.parse_decorator_identifier_reference_syntax_only();
        while self.eat(TokenKind::Dot) {
            if self.at(TokenKind::PrivateIdentifier) {
                self.advance();
            } else {
                self.parse_identifier_name();
            }
        }
    }

    fn parse_decorator_identifier_reference_syntax_only(&mut self) {
        if !self.at_identifier_reference() {
            self.error("expected decorator member expression".to_string());
            if !self.at(TokenKind::Eof) {
                self.advance();
            }
            return;
        }

        let span = self.current_span();
        let atom = self
            .current_atom()
            .unwrap_or_else(|| match self.current_kind() {
                TokenKind::Yield => WellKnownAtom::yield_.id(),
                TokenKind::Await => WellKnownAtom::r#await.id(),
                _ => WellKnownAtom::Empty.id(),
            });
        if self.current().contains_escape() {
            self.check_escaped_keyword_identifier();
        }
        self.validate_identifier_reference_atom(atom, span);
        self.advance();
    }

    fn parse_decorator_arguments_syntax_only(&mut self) {
        self.expect(TokenKind::LParen);
        while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
            self.eat(TokenKind::Ellipsis);
            self.parse_assignment_expression();
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::RParen);
    }

    /// Parses a class body: `{ elements }`.
    pub fn parse_class_body(&mut self) -> NodeList<lyng_js_ast::ClassElementId> {
        self.expect(TokenKind::LBrace);
        let mut elements = Vec::new();

        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            // Skip semicolons in class body
            if self.eat(TokenKind::Semicolon) {
                continue;
            }
            elements.push(self.parse_class_element());
        }

        self.expect(TokenKind::RBrace);
        self.ast_mut().alloc_class_element_list(&elements)
    }

    fn parse_class_element(&mut self) -> lyng_js_ast::ClassElementId {
        let decorator_start = self.current_span();
        if self.at(TokenKind::At) {
            self.parse_decorator_list_syntax_only();
        }
        let start = if self.at(TokenKind::Eof) {
            decorator_start
        } else {
            self.current_span()
        };
        let is_static = self.at_contextual(WellKnownAtom::r#static)
            && self.class_element_uses_static_modifier();

        // Static block: `static { ... }`
        if is_static && self.peek().kind == TokenKind::LBrace {
            self.advance(); // eat `static`
            self.expect(TokenKind::LBrace);
            let prev_in_static_block = self.in_static_block;
            self.in_static_block = true;
            let stmts = self.parse_statement_list_until(TokenKind::RBrace);
            self.in_static_block = prev_in_static_block;
            let end = self.expect(TokenKind::RBrace);
            let span = start.cover(end.span);
            let body = self.ast_mut().alloc_stmt_list(&stmts);
            return self
                .ast_mut()
                .alloc_class_element(ClassElement::StaticBlock { span, body });
        }

        if is_static {
            self.advance(); // eat `static`
        }

        // Auto-accessor field definitions: `accessor x;`.
        if self.at_contextual(WellKnownAtom::accessor) {
            let peek = self.peek();
            if !peek.preceded_by_line_terminator() && is_accessor_field_name_start(peek.kind) {
                self.advance(); // eat `accessor`
                let (key, computed, private) = self.parse_class_element_name();
                let auto_accessor_private_name =
                    (!private).then(|| self.auto_accessor_private_name(start));
                return self.finish_class_property(
                    start,
                    key,
                    computed,
                    private,
                    is_static,
                    auto_accessor_private_name,
                );
            }
        }

        // Get/Set accessors
        if self.at_contextual(WellKnownAtom::get) || self.at_contextual(WellKnownAtom::set) {
            let method_start = self.current_span();
            let is_get = self.at_contextual(WellKnownAtom::get);
            let peek = self.peek();
            if !peek.preceded_by_line_terminator() && is_property_name_start(peek.kind) {
                let method_kind = if is_get {
                    MethodKind::Get
                } else {
                    MethodKind::Set
                };
                self.advance(); // eat get/set
                let (key, computed, private) = self.parse_class_element_name();
                let func_id = self.parse_method_function_from(FunctionKind::Normal, method_start);
                let span = start.cover(self.ast().get_function(func_id).span);
                return self.ast_mut().alloc_class_element(ClassElement::Method {
                    span,
                    kind: method_kind,
                    key,
                    value: func_id,
                    computed,
                    private,
                    r#static: is_static,
                });
            }
        }

        // Async methods
        if self.at_contextual(WellKnownAtom::async_) {
            let method_start = self.current_span();
            let peek = self.peek();
            if !peek.preceded_by_line_terminator() && is_property_name_start(peek.kind) {
                self.advance(); // eat `async`
                let is_generator = self.eat(TokenKind::Star);
                let (key, computed, private) = self.parse_class_element_name();
                let kind = if is_generator {
                    FunctionKind::AsyncGenerator
                } else {
                    FunctionKind::Async
                };
                let func_id = self.parse_method_function_from(kind, method_start);
                let span = start.cover(self.ast().get_function(func_id).span);
                return self.ast_mut().alloc_class_element(ClassElement::Method {
                    span,
                    kind: MethodKind::Method,
                    key,
                    value: func_id,
                    computed,
                    private,
                    r#static: is_static,
                });
            }
        }

        // Generator methods
        if self.at(TokenKind::Star) {
            let method_start = self.current_span();
            self.advance();
            let (key, computed, private) = self.parse_class_element_name();
            let func_id = self.parse_method_function_from(FunctionKind::Generator, method_start);
            let span = start.cover(self.ast().get_function(func_id).span);
            return self.ast_mut().alloc_class_element(ClassElement::Method {
                span,
                kind: MethodKind::Method,
                key,
                value: func_id,
                computed,
                private,
                r#static: is_static,
            });
        }

        // Regular method or field
        let key_start = self.current_span();
        let (key, computed, private) = self.parse_class_element_name();

        // Method: has `(` after key
        if self.at(TokenKind::LParen) {
            let method_start = if computed {
                key_start
            } else {
                self.ast().get_expr(key).span()
            };
            let method_kind = if !computed
                && !private
                && !is_static
                && self.class_element_key_is_constructor(key)
            {
                MethodKind::Constructor
            } else {
                MethodKind::Method
            };

            let func_id = self.parse_method_function_from(FunctionKind::Normal, method_start);
            let span = start.cover(self.ast().get_function(func_id).span);
            return self.ast_mut().alloc_class_element(ClassElement::Method {
                span,
                kind: method_kind,
                key,
                value: func_id,
                computed,
                private,
                r#static: is_static,
            });
        }

        // Field: `name = value;` or `name;`
        self.finish_class_property(start, key, computed, private, is_static, None)
    }

    fn finish_class_property(
        &mut self,
        start: lyng_js_common::Span,
        key: lyng_js_ast::ExprId,
        computed: bool,
        private: bool,
        is_static: bool,
        auto_accessor_private_name: Option<lyng_js_common::AtomId>,
    ) -> lyng_js_ast::ClassElementId {
        let value = if self.eat(TokenKind::Eq) {
            let prev_await = self.allow_await;
            let prev_yield = self.allow_yield;
            self.allow_await = false;
            self.allow_yield = false;
            let value = self.parse_assignment_expression();
            self.allow_await = prev_await;
            self.allow_yield = prev_yield;
            Some(value)
        } else {
            None
        };
        self.expect_semicolon();

        let end = value
            .map(|v| self.ast().get_expr(v).span())
            .unwrap_or_else(|| self.ast().get_expr(key).span());
        let span = start.cover(end);

        self.ast_mut().alloc_class_element(ClassElement::Property {
            span,
            key,
            value,
            computed,
            private,
            r#static: is_static,
            auto_accessor_private_name,
        })
    }

    fn auto_accessor_private_name(
        &mut self,
        start: lyng_js_common::Span,
    ) -> lyng_js_common::AtomId {
        self.lexer
            .intern_atom(&format!("\0auto_accessor_{}", start.range.start.raw()))
    }

    fn parse_class_element_name(&mut self) -> (lyng_js_ast::ExprId, bool, bool) {
        if self.at(TokenKind::PrivateIdentifier) {
            let span = self.current_span();
            let name = self.current_atom().unwrap_or(WellKnownAtom::Empty.id());
            self.advance();
            let expr = self
                .ast_mut()
                .alloc_expr(lyng_js_ast::Expr::Identifier { span, name });
            (expr, false, true)
        } else {
            let (key, computed) = self.parse_property_name();
            (key, computed, false)
        }
    }

    fn class_element_key_is_constructor(&self, key: lyng_js_ast::ExprId) -> bool {
        match self.ast().get_expr(key) {
            lyng_js_ast::Expr::Identifier { name, .. } => *name == WellKnownAtom::constructor.id(),
            lyng_js_ast::Expr::StringLiteral { value, .. } => {
                self.ast().literals().get_string(*value) == WellKnownAtom::constructor.as_str()
            }
            _ => false,
        }
    }

    pub(crate) fn validate_class_heritage_expression(&mut self, expr: lyng_js_ast::ExprId) {
        if matches!(
            self.ast().get_expr(expr),
            Expr::ArrowFunctionExpression { .. }
        ) {
            let span = self.ast().get_expr(expr).span();
            self.error_at(
                span,
                "class heritage must be a left-hand-side expression".to_string(),
            );
        }
    }

    fn class_element_uses_static_modifier(&mut self) -> bool {
        let next = self.peek().kind;
        !matches!(
            next,
            TokenKind::LParen
                | TokenKind::Eq
                | TokenKind::Semicolon
                | TokenKind::RBrace
                | TokenKind::Eof
        )
    }
}

const fn is_property_name_start(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Identifier
            | TokenKind::PrivateIdentifier
            | TokenKind::StringLiteral
            | TokenKind::NumericLiteral
            | TokenKind::BigIntLiteral
            | TokenKind::LBracket
            | TokenKind::Star
    ) || kind.is_keyword()
}

const fn is_accessor_field_name_start(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Identifier
            | TokenKind::PrivateIdentifier
            | TokenKind::StringLiteral
            | TokenKind::NumericLiteral
            | TokenKind::BigIntLiteral
            | TokenKind::LBracket
    ) || kind.is_keyword()
}

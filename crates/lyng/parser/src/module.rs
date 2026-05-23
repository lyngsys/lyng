//! Module-specific parsing (import/export declarations).

use lyng_js_ast::{
    Decl, ExportDefaultDecl, ExportKind, ExportSpecifier, FunctionKind, ImportAttribute,
    ImportSpecifier, Stmt, StmtId,
};
use lyng_js_common::WellKnownAtom;
use lyng_js_lexer::{Token, TokenKind, TokenPayload};

use crate::parser::Parser;

impl<'src, 'atoms> Parser<'src, 'atoms> {
    /// Parses a module item: import, export, or statement list item.
    pub fn parse_module_item(&mut self) -> StmtId {
        match self.current_kind() {
            TokenKind::Import => {
                // Could be import declaration or import() expression
                let peek = self.peek();
                if peek.kind == TokenKind::LParen || peek.kind == TokenKind::Dot {
                    // import() or import.meta — treat as expression statement
                    self.parse_statement()
                } else {
                    self.parse_import_declaration()
                }
            }
            TokenKind::Export => self.parse_export_declaration(),
            _ => self.parse_statement_list_item(),
        }
    }

    // -----------------------------------------------------------------------
    // Import declarations
    // -----------------------------------------------------------------------

    fn parse_import_declaration(&mut self) -> StmtId {
        let start = self.current_span();
        self.advance(); // eat `import`

        // Side-effect import: `import "module"`
        if self.at(TokenKind::StringLiteral) {
            let source = self.parse_module_specifier();
            let attributes = self.parse_import_attributes();
            let span = start.cover(self.current_span());
            let specifiers = self.ast_mut().alloc_import_spec_list(&[]);
            let attributes = self.ast_mut().alloc_import_attr_list(&attributes);
            let decl = self.ast_mut().alloc_decl(Decl::Import {
                span,
                specifiers,
                source,
                attributes,
            });
            return self.ast_mut().alloc_stmt(Stmt::Declaration { span, decl });
        }

        let mut specifiers = Vec::new();

        // Source-phase import: `import source x from "mod"`.
        // The lookahead keeps `import source from "mod"` as a default import
        // whose local binding is named `source`.
        if self.is_source_phase_import_declaration() {
            self.advance(); // eat contextual `source`
            let local_span = self.current_span();
            let local = self.parse_binding_identifier();
            specifiers.push(ImportSpecifier::Source {
                span: local_span,
                local,
            });
        } else if self.is_deferred_namespace_import_declaration() {
            self.advance(); // eat contextual `defer`
            self.parse_namespace_import(&mut specifiers, true);
        } else if self.at(TokenKind::Identifier) {
            // Default import: `import x from "mod"`
            let local_span = self.current_span();
            let local = self.parse_binding_identifier();
            specifiers.push(ImportSpecifier::Default {
                span: local_span,
                local,
            });

            // If followed by comma, more specifiers follow
            if self.eat(TokenKind::Comma) {
                self.parse_import_specifiers_after_default(&mut specifiers);
            }
        } else if self.at(TokenKind::Star) {
            // Namespace import: `import * as name from "mod"`
            self.parse_namespace_import(&mut specifiers, false);
        } else if self.at(TokenKind::LBrace) {
            // Named imports: `import { a, b as c } from "mod"`
            self.parse_named_imports(&mut specifiers);
        } else {
            self.error("expected import specifier".to_string());
        }

        self.expect_contextual(WellKnownAtom::from);
        let source = self.parse_module_specifier();
        let attributes = self.parse_import_attributes();
        self.expect_semicolon();

        let span = start.cover(self.current_span());
        let spec_list = self.ast_mut().alloc_import_spec_list(&specifiers);
        let attributes = self.ast_mut().alloc_import_attr_list(&attributes);
        let decl = self.ast_mut().alloc_decl(Decl::Import {
            span,
            specifiers: spec_list,
            source,
            attributes,
        });
        self.ast_mut().alloc_stmt(Stmt::Declaration { span, decl })
    }

    fn is_source_phase_import_declaration(&mut self) -> bool {
        if !self.at_contextual_source() {
            return false;
        }
        let local = self.peek();
        if !self.token_is_binding_identifier_in_current_context(local) {
            return false;
        }
        token_is_contextual_atom(self.peek_second(), WellKnownAtom::from)
    }

    fn is_deferred_namespace_import_declaration(&mut self) -> bool {
        self.at_contextual_defer() && self.peek().kind == TokenKind::Star
    }

    fn at_contextual_source(&self) -> bool {
        self.current_kind() == TokenKind::Identifier
            && !self.current().contains_escape()
            && self
                .current_atom()
                .is_some_and(|atom| self.lexer.resolve_atom(atom) == "source")
    }

    fn at_contextual_defer(&self) -> bool {
        self.current_kind() == TokenKind::Identifier
            && !self.current().contains_escape()
            && self
                .current_atom()
                .is_some_and(|atom| self.lexer.resolve_atom(atom) == "defer")
    }

    const fn token_is_binding_identifier_in_current_context(&self, token: Token) -> bool {
        match token.kind {
            TokenKind::Identifier => true,
            TokenKind::Yield => !self.allow_yield && !self.is_strict(),
            TokenKind::Await => !self.allow_await,
            _ => false,
        }
    }

    fn parse_namespace_import(&mut self, specifiers: &mut Vec<ImportSpecifier>, deferred: bool) {
        self.advance(); // eat `*`
        self.expect_contextual(WellKnownAtom::as_);
        let local_span = self.current_span();
        let local = self.parse_binding_identifier();
        specifiers.push(ImportSpecifier::Namespace {
            span: local_span,
            local,
            deferred,
        });
    }

    fn parse_import_specifiers_after_default(&mut self, specifiers: &mut Vec<ImportSpecifier>) {
        if self.at(TokenKind::Star) {
            // `import x, * as ns from "mod"`
            self.parse_namespace_import(specifiers, false);
        } else if self.at(TokenKind::LBrace) {
            // `import x, { a, b } from "mod"`
            self.parse_named_imports(specifiers);
        }
    }

    fn parse_named_imports(&mut self, specifiers: &mut Vec<ImportSpecifier>) {
        self.expect(TokenKind::LBrace);

        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let spec_start = self.current_span();
            // ModuleExportName: Identifier | StringLiteral
            let is_string = self.at(TokenKind::StringLiteral);
            let imported = if is_string || self.current_kind().is_keyword() {
                self.parse_export_name()
            } else {
                self.parse_identifier_name()
            };

            if self.at_contextual(WellKnownAtom::as_) {
                self.advance(); // eat `as`
                let local = self.parse_binding_identifier();
                specifiers.push(ImportSpecifier::Named {
                    span: spec_start.cover(self.current_span()),
                    imported,
                    local,
                });
            } else if is_string {
                // String import names require `as localName`
                self.error("string import names require 'as' binding".to_string());
                specifiers.push(ImportSpecifier::Named {
                    span: spec_start,
                    imported,
                    local: imported,
                });
            } else {
                specifiers.push(ImportSpecifier::Named {
                    span: spec_start,
                    imported,
                    local: imported,
                });
            }

            if !self.eat(TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::RBrace);
    }

    // -----------------------------------------------------------------------
    // Export declarations
    // -----------------------------------------------------------------------

    fn parse_export_declaration(&mut self) -> StmtId {
        let start = self.current_span();
        self.advance(); // eat `export`

        // `export default`
        if self.eat(TokenKind::Default) {
            return self.parse_export_default(start);
        }

        // `export *`
        if self.at(TokenKind::Star) {
            return self.parse_export_all(start);
        }

        // `export { ... }`
        if self.at(TokenKind::LBrace) {
            return self.parse_export_named(start);
        }

        // `export var/let/const/function/class/async function`
        let decl_stmt = match self.current_kind() {
            TokenKind::Var => self.parse_var_declaration_stmt(),
            TokenKind::Const => self.parse_lexical_declaration_stmt(),
            TokenKind::Function => self.parse_function_declaration_stmt(),
            TokenKind::Class => self.parse_class_declaration_stmt(),
            TokenKind::Identifier if self.at_contextual(WellKnownAtom::let_) => {
                self.parse_lexical_declaration_stmt()
            }
            TokenKind::Identifier
                if self.at_contextual(WellKnownAtom::async_)
                    && self.peek().kind == TokenKind::Function =>
            {
                self.parse_async_function_declaration_stmt()
            }
            _ => {
                self.error("expected declaration after 'export'".to_string());
                self.recover_to_statement_boundary();
                let span = start.cover(self.current_span());
                return self.ast_mut().alloc_stmt(Stmt::InvalidStatement { span });
            }
        };

        // Extract the DeclId from the Declaration statement
        let decl_id = match self.ast().get_stmt(decl_stmt) {
            Stmt::Declaration { decl, .. } => *decl,
            _ => {
                let span = start.cover(self.current_span());
                self.ast_mut().alloc_decl(Decl::InvalidDeclaration { span })
            }
        };

        let span = start.cover(self.ast().get_decl(decl_id).span());
        let export_decl = self.ast_mut().alloc_decl(Decl::Export {
            span,
            kind: ExportKind::Declaration { decl: decl_id },
        });
        self.ast_mut().alloc_stmt(Stmt::Declaration {
            span,
            decl: export_decl,
        })
    }

    fn parse_export_default(&mut self, start: lyng_js_common::Span) -> StmtId {
        let declaration = match self.current_kind() {
            TokenKind::Function => {
                // `export default function [name]() {}`
                let func_start = self.current_span();
                self.advance(); // eat `function`
                let is_generator = self.eat(TokenKind::Star);
                let name = if self.at(TokenKind::Identifier) {
                    Some(self.parse_binding_identifier())
                } else {
                    None
                };
                let kind = if is_generator {
                    FunctionKind::Generator
                } else {
                    FunctionKind::Normal
                };
                let func_id = self.parse_function_common(name, kind, func_start);
                ExportDefaultDecl::Function(func_id)
            }
            TokenKind::Class => {
                // `export default class [name] {}`
                let class_stmt = self.parse_class_declaration_stmt();
                let class_decl_id = match self.ast().get_stmt(class_stmt) {
                    Stmt::Declaration { decl, .. } => *decl,
                    _ => self
                        .ast_mut()
                        .alloc_decl(Decl::InvalidDeclaration { span: start }),
                };
                ExportDefaultDecl::Class(class_decl_id)
            }
            TokenKind::Identifier
                if self.at_contextual(WellKnownAtom::async_)
                    && self.peek().kind == TokenKind::Function
                    && !self.peek().preceded_by_line_terminator() =>
            {
                let func_start = self.current_span();
                self.advance(); // eat `async`
                self.advance(); // eat `function`
                let is_generator = self.eat(TokenKind::Star);
                let name = if self.at(TokenKind::Identifier) {
                    Some(self.parse_binding_identifier())
                } else {
                    None
                };
                let kind = if is_generator {
                    FunctionKind::AsyncGenerator
                } else {
                    FunctionKind::Async
                };
                let func_id = self.parse_function_common(name, kind, func_start);
                ExportDefaultDecl::Function(func_id)
            }
            _ => {
                // `export default expr;`
                let expr = self.parse_assignment_expression();
                self.expect_semicolon();
                ExportDefaultDecl::Expression(expr)
            }
        };

        let span = start.cover(self.current_span());
        let decl = self.ast_mut().alloc_decl(Decl::Export {
            span,
            kind: ExportKind::Default { declaration },
        });
        self.ast_mut().alloc_stmt(Stmt::Declaration { span, decl })
    }

    fn parse_export_all(&mut self, start: lyng_js_common::Span) -> StmtId {
        self.advance(); // eat `*`

        let exported = if self.at_contextual(WellKnownAtom::as_) {
            self.advance(); // eat `as`
            Some(self.parse_export_name())
        } else {
            None
        };

        self.expect_contextual(WellKnownAtom::from);
        let source = self.parse_module_specifier();
        let attributes = self.parse_import_attributes();
        self.expect_semicolon();

        let span = start.cover(self.current_span());
        let attributes = self.ast_mut().alloc_import_attr_list(&attributes);
        let decl = self.ast_mut().alloc_decl(Decl::Export {
            span,
            kind: ExportKind::All {
                source,
                exported,
                attributes,
            },
        });
        self.ast_mut().alloc_stmt(Stmt::Declaration { span, decl })
    }

    fn parse_export_named(&mut self, start: lyng_js_common::Span) -> StmtId {
        self.expect(TokenKind::LBrace);
        let mut specifiers = Vec::new();
        let mut local_is_string = Vec::new();

        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let spec_start = self.current_span();
            let current_local_is_string = self.at(TokenKind::StringLiteral);
            let local = self.parse_export_name();

            let exported = if self.at_contextual(WellKnownAtom::as_) {
                self.advance(); // eat `as`
                self.parse_export_name()
            } else {
                local
            };

            specifiers.push(ExportSpecifier {
                span: spec_start.cover(self.current_span()),
                local,
                exported,
            });
            local_is_string.push(current_local_is_string);

            if !self.eat(TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::RBrace);

        // Optional `from "mod"` for re-exports
        let source_clause = if self.at_contextual(WellKnownAtom::from) {
            self.advance(); // eat `from`
            let s = self.parse_module_specifier();
            Some((s, self.parse_import_attributes()))
        } else {
            None
        };

        let (source, attributes) = if let Some((source, attributes)) = source_clause {
            (Some(source), attributes)
        } else {
            (None, Vec::new())
        };

        if source.is_none() {
            for (spec, local_is_string) in specifiers.iter().zip(local_is_string.iter().copied()) {
                if local_is_string {
                    self.error_at(
                        spec.span,
                        "string-literal export bindings require a `from` clause".to_string(),
                    );
                }
            }
        }

        self.expect_semicolon();

        let span = start.cover(self.current_span());
        let spec_list = self.ast_mut().alloc_export_spec_list(&specifiers);
        let attributes = self.ast_mut().alloc_import_attr_list(&attributes);
        let decl = self.ast_mut().alloc_decl(Decl::Export {
            span,
            kind: ExportKind::Named {
                specifiers: spec_list,
                source,
                attributes,
            },
        });
        self.ast_mut().alloc_stmt(Stmt::Declaration { span, decl })
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Expects an identifier with a specific well-known name (contextual keyword).
    fn expect_contextual(&mut self, atom: WellKnownAtom) {
        if self.at_contextual(atom) {
            self.advance();
        } else {
            self.error(format!("expected '{}'", atom.as_str()));
            // Advance to guarantee forward progress.
            if !self.at(TokenKind::Eof) {
                self.advance();
            }
        }
    }

    /// Parses a module export name — identifiers, keywords, and string
    /// literals are all valid (ModuleExportName).
    fn parse_export_name(&mut self) -> lyng_js_common::AtomId {
        if self.at(TokenKind::StringLiteral) {
            if !self.current_string_literal_is_well_formed_unicode() {
                self.error_at(
                    self.current_span(),
                    "module export names must be well-formed Unicode strings".to_string(),
                );
            }
            // String literal as export name: export { "name" }
            // Clone the value to break the borrow conflict within the lexer
            // (reading the literal table vs mutating the atom table).
            let atom = match self.current().payload {
                TokenPayload::Literal(lit_id) => {
                    let s = self.lexer.literals.get_string(lit_id).to_string_lossy();
                    self.lexer.intern_atom(&s)
                }
                _ => lyng_js_common::WellKnownAtom::Empty.id(),
            };
            self.advance();
            atom
        } else if self.current_kind().is_keyword() {
            let atom = self.keyword_to_atom();
            self.advance();
            atom
        } else {
            self.parse_identifier_name()
        }
    }

    fn current_string_literal_is_well_formed_unicode(&self) -> bool {
        let raw = self.lexer.span_text(self.current_span());
        !string_literal_has_unpaired_surrogate_escape(raw)
    }

    /// Parses an optional import-attribute clause: `with { key: value, ... }`.
    fn parse_import_attributes(&mut self) -> Vec<ImportAttribute> {
        // `with { ... }` — `with` is TokenKind::With (a keyword)
        if !self.at(TokenKind::With) {
            return Vec::new();
        }
        self.advance(); // eat `with`
        self.expect(TokenKind::LBrace);

        let mut seen_keys: Vec<String> = Vec::new();
        let mut attributes = Vec::new();

        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let key_span = self.current_span();
            let key = if self.at(TokenKind::Identifier) {
                let atom = self.current_atom();
                let resolved = atom.map(|a| self.lexer().resolve_atom(a).to_string());
                self.advance();
                atom.zip(resolved)
            } else if self.current_kind().is_keyword() {
                let atom = self.keyword_to_atom();
                let resolved = self.lexer().resolve_atom(atom).to_string();
                self.advance();
                Some((atom, resolved))
            } else if self.at(TokenKind::StringLiteral) {
                let value = match self.current().payload {
                    TokenPayload::Literal(lit_id) => {
                        let text = self.lexer.literals.get_string(lit_id).to_string_lossy();
                        Some((self.lexer.intern_atom(&text), text))
                    }
                    _ => None,
                };
                self.advance();
                value
            } else {
                self.error("expected attribute key".to_string());
                self.advance();
                None
            };

            // Check for duplicate keys (ECMA-262: WithClause duplicate key is a SyntaxError)
            if let Some((_, ref key_text)) = key {
                if seen_keys.iter().any(|k| k == key_text) {
                    self.error_at(key_span, "duplicate import attribute key".to_string());
                } else {
                    seen_keys.push(key_text.clone());
                }
            }

            self.expect(TokenKind::Colon);
            // value: string literal
            if self.at(TokenKind::StringLiteral) {
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
                if let Some((key, _)) = key {
                    attributes.push(ImportAttribute {
                        span: key_span.cover(self.current_span()),
                        key,
                        value,
                    });
                }
            } else {
                self.error("expected string literal for attribute value".to_string());
            }
            self.advance();
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::RBrace);
        attributes
    }

    /// Parses a module specifier (string literal) and returns its StringLiteralId.
    fn parse_module_specifier(&mut self) -> lyng_js_ast::StringLiteralId {
        if self.at(TokenKind::StringLiteral) {
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
            value
        } else {
            self.error("expected module specifier string".to_string());
            // Advance to guarantee forward progress.
            if !self.at(TokenKind::Eof) {
                self.advance();
            }
            self.ast_mut().literals_mut().alloc_string("")
        }
    }
}

fn token_is_contextual_atom(token: Token, atom: WellKnownAtom) -> bool {
    token.kind == TokenKind::Identifier
        && !token.contains_escape()
        && matches!(token.payload, TokenPayload::Atom(id) if id == atom.id())
}

fn string_literal_has_unpaired_surrogate_escape(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    if bytes.len() < 2 {
        return false;
    }

    let mut i = 1;
    while i + 1 < bytes.len().saturating_sub(1) {
        if bytes[i] != b'\\' {
            i += 1;
            continue;
        }

        match bytes[i + 1] {
            b'u' if i + 6 <= bytes.len() => {
                let Some(code_unit) = parse_u16_hex(&bytes[i + 2..i + 6]) else {
                    i += 2;
                    continue;
                };

                if (0xD800..=0xDBFF).contains(&code_unit) {
                    if i + 12 <= bytes.len()
                        && bytes[i + 6] == b'\\'
                        && bytes[i + 7] == b'u'
                        && parse_u16_hex(&bytes[i + 8..i + 12])
                            .is_some_and(|low| (0xDC00..=0xDFFF).contains(&low))
                    {
                        i += 12;
                        continue;
                    }
                    return true;
                }

                if (0xDC00..=0xDFFF).contains(&code_unit) {
                    return true;
                }

                i += 6;
            }
            _ => i += 2,
        }
    }

    false
}

fn parse_u16_hex(bytes: &[u8]) -> Option<u16> {
    if bytes.len() != 4 || !bytes.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let hex = std::str::from_utf8(bytes).ok()?;
    u16::from_str_radix(hex, 16).ok()
}

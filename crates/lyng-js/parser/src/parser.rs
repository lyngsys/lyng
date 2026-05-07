//! Core parser state machine and helper methods.

use lyng_js_ast::Ast;
use lyng_js_common::{AtomId, AtomTable, DiagnosticList, SourceId, Span, WellKnownAtom};
use lyng_js_lexer::{Lexer, LexerMode, Token, TokenKind, TokenPayload};

/// The parser state machine.
///
/// Owns the lexer, AST, diagnostic list, and contextual parsing flags.
pub struct Parser<'src, 'atoms> {
    /// The lexer producing tokens.
    pub(crate) lexer: Lexer<'src, 'atoms>,
    /// The full source text, used for goal-sensitive grammar refinements.
    source: &'src str,
    /// The AST container being built.
    pub(crate) ast: Ast,
    /// Diagnostics accumulated during parsing.
    diagnostics: DiagnosticList,
    /// The current token.
    current: Token,
    /// Lazy one-token lookahead (populated on demand).
    peek: Option<Token>,
    /// The source ID for creating spans.
    #[allow(dead_code)]
    source_id: SourceId,

    // --- Contextual flags ---
    /// Whether we are in strict mode.
    strict: bool,
    /// Whether we are parsing a module (always strict, allows import/export).
    is_module: bool,
    /// When true, `in` is not allowed as a binary operator (for-init context).
    no_in: bool,
    /// Whether `yield` is allowed as an identifier (inside generators).
    pub(crate) allow_yield: bool,
    /// Whether `await` is allowed as an identifier (inside async functions).
    pub(crate) allow_await: bool,
    /// Whether we are inside a function body (return is allowed).
    in_function: bool,
    /// Whether we are inside an iteration statement (continue/break allowed).
    in_iteration: bool,
    /// Whether we are inside a switch statement (break allowed).
    in_switch: bool,
    /// Whether we are inside a class static block outside of a nested function.
    pub(crate) in_static_block: bool,
    /// Nesting depth of the currently parsed statement list.
    pub(crate) statement_list_depth: usize,
    /// Stack of direct switch-clause statement-list depths.
    pub(crate) switch_clause_statement_list_depths: Vec<usize>,
}

impl<'src, 'atoms> Parser<'src, 'atoms> {
    /// Creates a new parser and primes it with the first token.
    pub fn new(
        source: &'src str,
        source_id: SourceId,
        atoms: &'atoms mut AtomTable,
        allow_html_comments: bool,
    ) -> Self {
        let mut lexer = Lexer::new(source, source_id, atoms);
        lexer.set_allow_html_comments(allow_html_comments);
        let current = lexer.next_token();
        Self {
            lexer,
            source,
            ast: Ast::new(),
            diagnostics: DiagnosticList::new(),
            current,
            peek: None,
            source_id,
            strict: false,
            is_module: false,
            no_in: false,
            allow_yield: false,
            allow_await: false,
            in_function: false,
            in_iteration: false,
            in_switch: false,
            in_static_block: false,
            statement_list_depth: 0,
            switch_clause_statement_list_depths: Vec::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

    /// Returns the current token kind.
    #[inline]
    pub const fn current_kind(&self) -> TokenKind {
        self.current.kind
    }

    /// Returns the current token.
    #[inline]
    pub const fn current(&self) -> Token {
        self.current
    }

    /// Returns the span of the current token.
    #[inline]
    pub const fn current_span(&self) -> Span {
        self.current.span
    }

    #[inline]
    pub(crate) fn span_text(&self, span: Span) -> &'src str {
        let start = span.range.start.raw() as usize;
        let end = span.range.end.raw() as usize;
        &self.source[start..end]
    }

    /// Returns a reference to the lexer (for accessing literal tables).
    #[inline]
    pub const fn lexer(&self) -> &Lexer<'src, 'atoms> {
        &self.lexer
    }

    /// Returns a mutable reference to the lexer (for rewind/mode changes).
    #[inline]
    #[allow(dead_code)]
    pub const fn lexer_mut(&mut self) -> &mut Lexer<'src, 'atoms> {
        &mut self.lexer
    }

    /// Returns a mutable reference to the AST.
    #[inline]
    pub const fn ast_mut(&mut self) -> &mut Ast {
        &mut self.ast
    }

    /// Returns a reference to the AST.
    #[inline]
    pub const fn ast(&self) -> &Ast {
        &self.ast
    }

    #[inline]
    pub const fn is_strict(&self) -> bool {
        self.strict
    }

    #[inline]
    #[allow(dead_code)]
    pub const fn is_module(&self) -> bool {
        self.is_module
    }

    #[inline]
    pub const fn no_in(&self) -> bool {
        self.no_in
    }

    #[inline]
    pub const fn in_function(&self) -> bool {
        self.in_function
    }

    #[inline]
    pub const fn in_iteration(&self) -> bool {
        self.in_iteration
    }

    #[inline]
    pub const fn in_switch(&self) -> bool {
        self.in_switch
    }

    // -----------------------------------------------------------------------
    // Flag setters
    // -----------------------------------------------------------------------

    #[inline]
    pub const fn set_strict(&mut self, v: bool) {
        self.strict = v;
    }

    #[inline]
    pub const fn set_module(&mut self, v: bool) {
        self.is_module = v;
        self.lexer.set_allow_html_comments(!v);
    }

    #[inline]
    pub const fn set_no_in(&mut self, v: bool) {
        self.no_in = v;
    }

    #[inline]
    #[allow(dead_code)]
    pub const fn set_allow_yield(&mut self, v: bool) {
        self.allow_yield = v;
    }

    #[inline]
    #[allow(dead_code)]
    pub const fn set_allow_await(&mut self, v: bool) {
        self.allow_await = v;
    }

    #[inline]
    pub const fn set_in_function(&mut self, v: bool) {
        self.in_function = v;
    }

    #[inline]
    pub const fn set_in_iteration(&mut self, v: bool) {
        self.in_iteration = v;
    }

    #[inline]
    pub const fn set_in_switch(&mut self, v: bool) {
        self.in_switch = v;
    }

    // -----------------------------------------------------------------------
    // Token navigation
    // -----------------------------------------------------------------------

    /// Advances to the next token, returning the consumed token.
    pub fn advance(&mut self) -> Token {
        let consumed = self.current;
        if let Some(peeked) = self.peek.take() {
            self.current = peeked;
        } else {
            self.current = self.lexer.next_token();
        }
        consumed
    }

    /// Returns true if the current token matches the given kind.
    #[inline]
    pub fn at(&self, kind: TokenKind) -> bool {
        self.current.kind == kind
    }

    /// Returns true if the current token is an identifier with the given atom.
    pub fn at_contextual(&self, atom: WellKnownAtom) -> bool {
        self.current.kind == TokenKind::Identifier
            && !self.current.contains_escape()
            && self.current_atom() == Some(atom.id())
    }

    /// If the current token matches, consume it and return true.
    pub fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Expects the current token to be `kind`, consumes it, and returns the
    /// consumed token. Reports an error and advances past the bad token if it
    /// does not match — this guarantees forward progress so callers in loops
    /// cannot spin forever.
    pub fn expect(&mut self, kind: TokenKind) -> Token {
        if self.at(kind) {
            self.advance()
        } else {
            self.error(format!(
                "expected {:?}, found {:?}",
                kind, self.current.kind
            ));
            // Advance past the unexpected token to guarantee forward progress.
            // Without this, any loop calling expect() will spin forever on a
            // mismatched token.
            let bad = self.current;
            if !self.at(TokenKind::Eof) {
                self.advance();
            }
            bad
        }
    }

    /// Peeks at the next token without consuming the current one.
    pub fn peek(&mut self) -> Token {
        if let Some(peeked) = self.peek {
            peeked
        } else {
            let peeked = self.lexer.next_token();
            self.peek = Some(peeked);
            peeked
        }
    }

    /// Peeks two tokens ahead without consuming the current token.
    pub fn peek_second(&mut self) -> Token {
        let peeked = self.peek();
        let second = self.lexer.next_token();
        self.lexer.rewind_to(peeked.span.range.end.raw() as usize);
        second
    }

    /// Returns true if the current token was preceded by a line terminator.
    #[inline]
    pub const fn preceded_by_line_terminator(&self) -> bool {
        self.current.preceded_by_line_terminator()
    }

    /// Sets the lexer mode for the next token scan.
    #[inline]
    pub const fn set_lexer_mode(&mut self, mode: LexerMode) {
        self.lexer.set_mode(mode);
    }

    /// Rewinds the lexer to `pos`, re-lexes the current token with the given
    /// mode, and clears the peek cache. Used to re-lex `/` as a regexp literal.
    pub fn relex_with_mode(&mut self, pos: usize, mode: LexerMode) {
        self.lexer.rewind_to(pos);
        self.lexer.set_mode(mode);
        self.current = self.lexer.next_token();
        self.peek = None;
    }

    /// Returns the atom ID of the current token if it carries one.
    #[inline]
    pub const fn current_atom(&self) -> Option<AtomId> {
        match self.current.payload {
            TokenPayload::Atom(id) => Some(id),
            _ => None,
        }
    }

    #[inline]
    pub(crate) const fn enter_statement_list(&mut self) {
        self.statement_list_depth += 1;
    }

    #[inline]
    pub(crate) const fn exit_statement_list(&mut self) {
        self.statement_list_depth = self.statement_list_depth.saturating_sub(1);
    }

    #[inline]
    pub(crate) fn enter_switch_clause_statement_list(&mut self) {
        self.enter_statement_list();
        self.switch_clause_statement_list_depths
            .push(self.statement_list_depth);
    }

    #[inline]
    pub(crate) fn exit_switch_clause_statement_list(&mut self) {
        let _ = self.switch_clause_statement_list_depths.pop();
        self.exit_statement_list();
    }

    #[inline]
    pub(crate) const fn in_program_statement_list(&self) -> bool {
        self.statement_list_depth == 1 && !self.in_function && !self.in_static_block
    }

    #[inline]
    pub(crate) fn in_direct_switch_clause_statement_list(&self) -> bool {
        self.switch_clause_statement_list_depths
            .last()
            .is_some_and(|depth| *depth == self.statement_list_depth)
    }

    // -----------------------------------------------------------------------
    // ASI (Automatic Semicolon Insertion)
    // -----------------------------------------------------------------------

    /// Consumes a semicolon, or performs ASI if possible.
    ///
    /// ASI applies when:
    /// 1. A semicolon is present (just eat it).
    /// 2. A line terminator precedes the current token.
    /// 3. The current token is `}`.
    /// 4. The current token is EOF.
    pub fn expect_semicolon(&mut self) {
        if self.eat(TokenKind::Semicolon) {
            return;
        }
        // ASI conditions
        if self.preceded_by_line_terminator()
            || self.at(TokenKind::RBrace)
            || self.at(TokenKind::Eof)
        {
            return;
        }
        self.error("expected ';'".to_string());
    }

    /// Returns true if we can perform ASI at this point.
    pub fn can_insert_semicolon(&self) -> bool {
        self.at(TokenKind::Semicolon)
            || self.preceded_by_line_terminator()
            || self.at(TokenKind::RBrace)
            || self.at(TokenKind::Eof)
    }

    // -----------------------------------------------------------------------
    // Diagnostics
    // -----------------------------------------------------------------------

    /// Reports an error at the current token's span.
    pub fn error(&mut self, message: String) {
        self.diagnostics.error(self.current.span, message);
    }

    /// Reports an error at a specific span.
    pub fn error_at(&mut self, span: Span, message: String) {
        self.diagnostics.error(span, message);
    }

    // -----------------------------------------------------------------------
    // Recovery
    // -----------------------------------------------------------------------

    /// Skips tokens until we reach a statement or declaration boundary.
    pub fn recover_to_statement_boundary(&mut self) {
        loop {
            match self.current.kind {
                TokenKind::Eof | TokenKind::Semicolon | TokenKind::RBrace => return,
                // Keywords that start statements
                TokenKind::Var
                | TokenKind::Function
                | TokenKind::Class
                | TokenKind::If
                | TokenKind::For
                | TokenKind::While
                | TokenKind::Do
                | TokenKind::Switch
                | TokenKind::Try
                | TokenKind::Return
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Throw
                | TokenKind::Debugger
                | TokenKind::Import
                | TokenKind::Export
                | TokenKind::Const => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Identifier helpers
    // -----------------------------------------------------------------------

    /// Returns true if the current token is an identifier or a contextual
    /// keyword that can be used as an identifier in the current context.
    #[allow(dead_code)]
    pub const fn at_identifier(&self) -> bool {
        match self.current.kind {
            TokenKind::Identifier => true,
            TokenKind::Yield if !self.allow_yield => true,
            TokenKind::Await if !self.allow_await => true,
            _ => false,
        }
    }

    /// Returns true if the current token can be used as an `IdentifierReference`
    /// in the current parse context.
    pub const fn at_identifier_reference(&self) -> bool {
        match self.current.kind {
            TokenKind::Identifier => true,
            TokenKind::Yield => !self.allow_yield && !self.strict,
            TokenKind::Await => !self.allow_await,
            _ => false,
        }
    }

    /// Parses an identifier name (any identifier or keyword usable as a
    /// property name). Returns the atom ID.
    pub fn parse_identifier_name(&mut self) -> AtomId {
        if self.current.kind == TokenKind::Identifier {
            let atom = self.current_atom().unwrap_or(WellKnownAtom::Empty.id());
            self.advance();
            return atom;
        }

        if self.current.kind.is_keyword() {
            let atom = self.keyword_to_atom();
            self.advance();
            return atom;
        }

        self.error("expected identifier".to_string());
        // Advance to guarantee forward progress.
        if !self.at(TokenKind::Eof) {
            self.advance();
        }
        WellKnownAtom::Empty.id()
    }

    /// Parses a binding identifier. Returns the atom ID.
    pub fn parse_binding_identifier(&mut self) -> AtomId {
        self.parse_binding_identifier_with_strict(self.strict)
    }

    pub const fn at_binding_identifier_in_context(&self, strict_context: bool) -> bool {
        match self.current.kind {
            TokenKind::Identifier => true,
            TokenKind::Yield => !self.allow_yield && !strict_context,
            TokenKind::Await => !self.allow_await,
            _ => false,
        }
    }

    pub fn parse_binding_identifier_with_strict(&mut self, strict_context: bool) -> AtomId {
        if self.at_binding_identifier_in_context(strict_context) {
            let atom = self
                .current_atom()
                .unwrap_or_else(|| match self.current_kind() {
                    TokenKind::Yield => WellKnownAtom::yield_.id(),
                    TokenKind::Await => WellKnownAtom::r#await.id(),
                    _ => WellKnownAtom::Empty.id(),
                });
            if self.binding_identifier_is_reserved(atom, strict_context) {
                let name = self.lexer.resolve_atom(atom);
                self.error_at(
                    self.current_span(),
                    format!("reserved word '{name}' cannot be used as a binding identifier"),
                );
            }
            self.advance();
            atom
        } else {
            self.error("expected binding identifier".to_string());
            if !self.at(TokenKind::Eof) {
                self.advance();
            }
            WellKnownAtom::Empty.id()
        }
    }

    /// Returns true if the current token may be used as a function expression
    /// name in the current context.
    pub const fn at_function_expression_name(&self) -> bool {
        match self.current.kind {
            TokenKind::Identifier => true,
            TokenKind::Yield => !self.strict,
            TokenKind::Await => !self.is_module,
            _ => false,
        }
    }

    /// Parses a function expression name.
    pub fn parse_function_expression_name(&mut self) -> AtomId {
        if self.at_function_expression_name() {
            let atom = self
                .current_atom()
                .unwrap_or_else(|| match self.current_kind() {
                    TokenKind::Yield => WellKnownAtom::yield_.id(),
                    TokenKind::Await => WellKnownAtom::r#await.id(),
                    _ => WellKnownAtom::Empty.id(),
                });
            if self.current.kind == TokenKind::Identifier
                && self.binding_identifier_is_reserved(atom, self.strict)
            {
                let name = self.lexer.resolve_atom(atom);
                self.error_at(
                    self.current_span(),
                    format!("reserved word '{name}' cannot be used as a binding identifier"),
                );
            }
            self.advance();
            atom
        } else {
            self.error("expected binding identifier".to_string());
            if !self.at(TokenKind::Eof) {
                self.advance();
            }
            WellKnownAtom::Empty.id()
        }
    }

    /// Returns true if the current token can be used as a `LabelIdentifier`
    /// in the current parse context.
    pub const fn at_label_identifier(&self) -> bool {
        match self.current.kind {
            TokenKind::Identifier => true,
            TokenKind::Yield => !self.allow_yield && !self.strict,
            TokenKind::Await => !self.allow_await && !self.is_module,
            _ => false,
        }
    }

    fn label_identifier_is_reserved(&self, atom: AtomId) -> bool {
        let raw = atom.raw();

        if atom == WellKnownAtom::yield_.id() {
            return self.allow_yield || self.strict;
        }

        if atom == WellKnownAtom::r#await.id() {
            return self.allow_await || self.is_module || self.in_static_block;
        }

        if (2..=37).contains(&raw) {
            return true;
        }

        self.strict && (39..=46).contains(&raw)
    }

    /// Parses a `LabelIdentifier`.
    pub fn parse_label_identifier(&mut self) -> AtomId {
        if self.at_label_identifier() {
            let token = self.current();
            let atom = self
                .current_atom()
                .unwrap_or_else(|| match self.current_kind() {
                    TokenKind::Yield => WellKnownAtom::yield_.id(),
                    TokenKind::Await => WellKnownAtom::r#await.id(),
                    _ => WellKnownAtom::Empty.id(),
                });
            if self.label_identifier_is_reserved(atom) {
                let name = self.lexer.resolve_atom(atom);
                let message = if token.kind == TokenKind::Identifier && token.contains_escape() {
                    format!(
                        "keyword '{name}' cannot be used as a label identifier via escape sequence"
                    )
                } else {
                    format!("reserved word '{name}' cannot be used as a label identifier")
                };
                self.error_at(token.span, message);
            }
            self.advance();
            atom
        } else {
            self.error("expected label identifier".to_string());
            if !self.at(TokenKind::Eof) {
                self.advance();
            }
            WellKnownAtom::Empty.id()
        }
    }

    // -----------------------------------------------------------------------
    // Assignment target validation
    // -----------------------------------------------------------------------

    /// Returns true if the expression is a valid simple assignment target
    /// (identifier, member expression, or similar LHS form).
    pub fn is_simple_assignment_target(&self, expr: lyng_js_ast::ExprId) -> bool {
        match self.ast().get_expr(expr) {
            lyng_js_ast::Expr::Identifier { name, .. } => {
                !(self.is_strict()
                    && (*name == WellKnownAtom::eval.id()
                        || *name == WellKnownAtom::arguments.id()))
            }
            lyng_js_ast::Expr::StaticMemberExpression { .. }
            | lyng_js_ast::Expr::ComputedMemberExpression { .. }
            | lyng_js_ast::Expr::PrivateMemberExpression { .. } => true,
            lyng_js_ast::Expr::CallExpression { .. } => {
                self.allows_annex_b_call_assignment_target()
            }
            lyng_js_ast::Expr::ParenthesizedExpression { expression, .. } => {
                self.is_simple_assignment_target(*expression)
            }
            _ => false,
        }
    }

    /// Returns true if the expression is a simple assignment target under the
    /// standard grammar, without Annex B's web-compat CallExpression extension.
    pub fn is_simple_assignment_target_without_annex_b_call(
        &self,
        expr: lyng_js_ast::ExprId,
    ) -> bool {
        match self.ast().get_expr(expr) {
            lyng_js_ast::Expr::Identifier { name, .. } => {
                !(self.is_strict()
                    && (*name == WellKnownAtom::eval.id()
                        || *name == WellKnownAtom::arguments.id()))
            }
            lyng_js_ast::Expr::StaticMemberExpression { .. }
            | lyng_js_ast::Expr::ComputedMemberExpression { .. }
            | lyng_js_ast::Expr::PrivateMemberExpression { .. } => true,
            lyng_js_ast::Expr::ParenthesizedExpression { expression, .. } => {
                self.is_simple_assignment_target_without_annex_b_call(*expression)
            }
            _ => false,
        }
    }

    /// Returns true if the expression is a valid LHS for `=` assignment
    /// (simple target, or object/array literal that can be destructured,
    /// or parenthesized valid target).
    pub fn is_valid_assignment_lhs(&self, expr: lyng_js_ast::ExprId) -> bool {
        match self.ast().get_expr(expr) {
            lyng_js_ast::Expr::Identifier { name, .. } => {
                !(self.is_strict()
                    && (*name == WellKnownAtom::eval.id()
                        || *name == WellKnownAtom::arguments.id()))
            }
            lyng_js_ast::Expr::StaticMemberExpression { .. }
            | lyng_js_ast::Expr::ComputedMemberExpression { .. }
            | lyng_js_ast::Expr::PrivateMemberExpression { .. }
            | lyng_js_ast::Expr::ObjectExpression { .. }
            | lyng_js_ast::Expr::ArrayExpression { .. } => true,
            lyng_js_ast::Expr::CallExpression { .. } => {
                self.allows_annex_b_call_assignment_target()
            }
            lyng_js_ast::Expr::ParenthesizedExpression { expression, .. } => {
                self.is_valid_assignment_lhs(*expression)
            }
            _ => false,
        }
    }

    pub(crate) fn has_line_terminator_before_first_non_trivia(&self, start: u32, end: u32) -> bool {
        self.next_non_trivia_in_range(start, end)
            .is_some_and(|(_, _, saw_line_terminator)| saw_line_terminator)
    }

    pub(crate) fn has_trailing_comma_before_closing_paren(&self, start: u32, end: u32) -> bool {
        let Some((first, first_pos, _)) = self.next_non_trivia_in_range(start, end) else {
            return false;
        };
        if first != ',' {
            return false;
        }

        let next_start = first_pos + first.len_utf8() as u32;
        matches!(
            self.next_non_trivia_in_range(next_start, end),
            Some((')', _, _))
        )
    }

    fn next_non_trivia_in_range(&self, start: u32, end: u32) -> Option<(char, u32, bool)> {
        let bytes = self.source.as_bytes();
        let mut pos = start as usize;
        let end = end as usize;
        let mut saw_line_terminator = false;

        while pos < end {
            match bytes[pos] {
                b' ' | b'\t' | 0x0B | 0x0C => pos += 1,
                b'\r' => {
                    saw_line_terminator = true;
                    pos += 1;
                    if pos < end && bytes[pos] == b'\n' {
                        pos += 1;
                    }
                }
                b'\n' => {
                    saw_line_terminator = true;
                    pos += 1;
                }
                b'/' if pos + 1 < end && bytes[pos + 1] == b'/' => {
                    pos += 2;
                    while pos < end {
                        match bytes[pos] {
                            b'\r' => {
                                saw_line_terminator = true;
                                pos += 1;
                                if pos < end && bytes[pos] == b'\n' {
                                    pos += 1;
                                }
                                break;
                            }
                            b'\n' => {
                                saw_line_terminator = true;
                                pos += 1;
                                break;
                            }
                            _ => pos += 1,
                        }
                    }
                }
                b'/' if pos + 1 < end && bytes[pos + 1] == b'*' => {
                    pos += 2;
                    while pos < end {
                        if pos + 1 < end && bytes[pos] == b'*' && bytes[pos + 1] == b'/' {
                            pos += 2;
                            break;
                        }
                        match bytes[pos] {
                            b'\r' => {
                                saw_line_terminator = true;
                                pos += 1;
                                if pos < end && bytes[pos] == b'\n' {
                                    pos += 1;
                                }
                            }
                            b'\n' => {
                                saw_line_terminator = true;
                                pos += 1;
                            }
                            0xE2 if pos + 2 < end
                                && bytes[pos + 1] == 0x80
                                && matches!(bytes[pos + 2], 0xA8 | 0xA9) =>
                            {
                                saw_line_terminator = true;
                                pos += 3;
                            }
                            _ => pos += 1,
                        }
                    }
                }
                0xE2 if pos + 2 < end
                    && bytes[pos + 1] == 0x80
                    && matches!(bytes[pos + 2], 0xA8 | 0xA9) =>
                {
                    saw_line_terminator = true;
                    pos += 3;
                }
                _ => {
                    let ch = self.source[pos..end].chars().next().unwrap_or_default();
                    return Some((ch, pos as u32, saw_line_terminator));
                }
            }
        }

        None
    }

    // -----------------------------------------------------------------------
    // Keyword-to-atom helper
    // -----------------------------------------------------------------------

    /// Converts the current keyword token to its atom ID.
    /// Used for property names where keywords are valid identifiers.
    pub const fn keyword_to_atom(&self) -> AtomId {
        Self::keyword_kind_to_atom(self.current.kind)
    }

    const fn keyword_kind_to_atom(kind: TokenKind) -> AtomId {
        // Map keyword TokenKind → WellKnownAtom discriminant (1..=38)
        let idx = match kind {
            TokenKind::Await => 1,
            TokenKind::Break => 2,
            TokenKind::Case => 3,
            TokenKind::Catch => 4,
            TokenKind::Class => 5,
            TokenKind::Const => 6,
            TokenKind::Continue => 7,
            TokenKind::Debugger => 8,
            TokenKind::Default => 9,
            TokenKind::Delete => 10,
            TokenKind::Do => 11,
            TokenKind::Else => 12,
            TokenKind::Enum => 13,
            TokenKind::Export => 14,
            TokenKind::Extends => 15,
            TokenKind::False => 16,
            TokenKind::Finally => 17,
            TokenKind::For => 18,
            TokenKind::Function => 19,
            TokenKind::If => 20,
            TokenKind::Import => 21,
            TokenKind::In => 22,
            TokenKind::Instanceof => 23,
            TokenKind::New => 24,
            TokenKind::Null => 25,
            TokenKind::Return => 26,
            TokenKind::Super => 27,
            TokenKind::Switch => 28,
            TokenKind::This => 29,
            TokenKind::Throw => 30,
            TokenKind::True => 31,
            TokenKind::Try => 32,
            TokenKind::Typeof => 33,
            TokenKind::Var => 34,
            TokenKind::Void => 35,
            TokenKind::While => 36,
            TokenKind::With => 37,
            TokenKind::Yield => 38,
            _ => 0,
        };
        AtomId::from_raw(idx)
    }

    pub fn identifier_reference_is_reserved(&self, atom: AtomId) -> bool {
        let raw = atom.raw();

        if atom == WellKnownAtom::yield_.id() {
            return self.allow_yield || self.strict;
        }

        if atom == WellKnownAtom::r#await.id() {
            return self.allow_await || self.in_static_block;
        }

        if (2..=37).contains(&raw) {
            return true;
        }

        self.strict && (39..=46).contains(&raw)
    }

    pub fn validate_identifier_reference_atom(&mut self, atom: AtomId, span: Span) {
        if self.identifier_reference_is_reserved(atom) {
            let name = self.lexer.resolve_atom(atom);
            self.error_at(
                span,
                format!("reserved word '{name}' cannot be used as an identifier reference"),
            );
        }
    }

    pub fn binding_identifier_is_reserved(&self, atom: AtomId, strict_context: bool) -> bool {
        let raw = atom.raw();

        if atom == WellKnownAtom::yield_.id() {
            return self.allow_yield || strict_context;
        }

        if atom == WellKnownAtom::r#await.id() {
            return self.allow_await || self.is_module || self.in_static_block;
        }

        if (2..=37).contains(&raw) {
            return true;
        }

        strict_context && (39..=46).contains(&raw)
    }

    // -----------------------------------------------------------------------
    // Escaped keyword check
    // -----------------------------------------------------------------------

    /// Checks if the current Identifier token is an escaped keyword and
    /// reports a SyntaxError. ECMA-262 §12.1: identifiers that contain
    /// Unicode escape sequences and resolve to a keyword are not valid.
    pub(crate) fn check_escaped_keyword_identifier(&mut self) {
        if let Some(atom) = self.current_atom() {
            let raw = atom.raw();
            // WellKnownAtom keyword range: 1 (await) through 38 (yield)
            let is_keyword = (1..=38).contains(&raw) || (self.strict && (39..=46).contains(&raw));
            if is_keyword {
                let name = self.lexer.resolve_atom(atom);
                self.error(format!(
                    "keyword '{name}' cannot be used as an identifier via escape sequence"
                ));
            }
        }
    }

    #[inline]
    pub(crate) const fn allows_annex_b_sloppy_function_declarations(&self) -> bool {
        !self.is_strict() && !self.is_module
    }

    #[inline]
    fn allows_annex_b_call_assignment_target(&self) -> bool {
        self.allows_annex_b_sloppy_function_declarations()
    }

    // -----------------------------------------------------------------------
    // Finish
    // -----------------------------------------------------------------------

    /// Consumes the parser and returns the AST and diagnostics.
    pub fn finish(mut self) -> (Ast, DiagnosticList) {
        // Drain lexer diagnostics into parser diagnostics
        self.diagnostics.extend(&mut self.lexer.diagnostics);
        (self.ast, self.diagnostics)
    }
}

//! Streaming lexer for the ECMA-262 Edition 16 lexical grammar.
//!
//! The lexer produces one token at a time via `next_token()`. The parser
//! controls lexer mode to disambiguate `/` (division vs regexp) and
//! template continuations.

use std::borrow::Cow;

use lyng_js_common::{AtomTable, DiagnosticList, SourceId, Span};

use crate::literals::{LiteralTable, StringLiteral};
use crate::token::{Token, TokenFlags, TokenKind, TokenPayload, KEYWORD_TOKEN_KIND};

/// Parser-controlled lexer mode for ambiguous productions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LexerMode {
    /// Normal scanning. `/` is division or `/=`.
    Normal,
    /// The parser expects a regexp literal. `/` starts a regexp.
    RegExp,
    /// The parser expects a template continuation after `}`.
    /// The lexer will scan `}...${` or `` }...` ``.
    TemplateContinuation,
}

enum CookedStringBuffer {
    Utf8(String),
    Utf16(Vec<u16>),
}

impl Default for CookedStringBuffer {
    fn default() -> Self {
        Self::Utf8(String::new())
    }
}

impl CookedStringBuffer {
    fn push_char(&mut self, ch: char) {
        match self {
            Self::Utf8(value) => value.push(ch),
            Self::Utf16(units) => {
                let mut encoded = [0u16; 2];
                units.extend_from_slice(ch.encode_utf16(&mut encoded));
            }
        }
    }

    fn push_code_unit(&mut self, unit: u16) {
        match self {
            Self::Utf8(value) => {
                let mut units: Vec<u16> = value.encode_utf16().collect();
                units.push(unit);
                *self = Self::Utf16(units);
            }
            Self::Utf16(units) => units.push(unit),
        }
    }

    fn into_literal(self) -> StringLiteral {
        match self {
            Self::Utf8(value) => StringLiteral::from_utf8(value),
            Self::Utf16(units) => StringLiteral::from_utf16(units),
        }
    }
}

enum UnicodeStringEscape {
    Scalar(char),
    CodeUnit(u16),
}

/// A streaming lexer for ECMAScript source text.
pub struct Lexer<'src, 'atoms> {
    /// The full source text as bytes.
    source: &'src [u8],
    /// Current byte position in `source`.
    pos: usize,
    /// The source ID for spans.
    source_id: SourceId,
    /// The shared atom table.
    atoms: &'atoms mut AtomTable,
    /// Accumulated diagnostics (errors/warnings).
    pub diagnostics: DiagnosticList,
    /// Literal side tables.
    pub literals: LiteralTable,
    /// The current lexer mode.
    mode: LexerMode,
    /// Whether a line terminator was seen since the last token.
    saw_line_terminator: bool,
    /// Whether the next token begins at a line start after only whitespace/comments.
    at_line_start: bool,
    /// Annex B HTML-like comments are enabled in script goal only.
    allow_html_comments: bool,
}

impl<'src, 'atoms> Lexer<'src, 'atoms> {
    /// Creates a new lexer for the given source text.
    pub fn new(source: &'src str, source_id: SourceId, atoms: &'atoms mut AtomTable) -> Self {
        Self {
            source: source.as_bytes(),
            pos: 0,
            source_id,
            atoms,
            diagnostics: DiagnosticList::new(),
            literals: LiteralTable::new(),
            mode: LexerMode::Normal,
            saw_line_terminator: false,
            at_line_start: true,
            allow_html_comments: true,
        }
    }

    /// Sets the lexer mode for the next token.
    #[inline]
    pub fn set_mode(&mut self, mode: LexerMode) {
        self.mode = mode;
    }

    /// Enables or disables Annex B HTML-like comments.
    #[inline]
    pub fn set_allow_html_comments(&mut self, allow: bool) {
        self.allow_html_comments = allow;
    }

    /// Returns the current byte position.
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Rewinds the lexer to a previous byte position. Used by the parser to
    /// re-lex `/` as a regexp literal after discovering the token is in
    /// expression position.
    #[inline]
    pub fn rewind_to(&mut self, pos: usize) {
        self.pos = pos;
    }

    /// Returns a reference to the literal table.
    #[inline]
    pub fn literal_table(&self) -> &LiteralTable {
        &self.literals
    }

    /// Returns the original source text covered by the given span.
    #[inline]
    pub fn span_text(&self, span: Span) -> &'src str {
        self.text(
            span.range.start.raw() as usize,
            span.range.end.raw() as usize,
        )
    }

    /// Interns a string in the shared atom table, returning its `AtomId`.
    #[inline]
    pub fn intern_atom(&mut self, s: &str) -> lyng_js_common::AtomId {
        self.atoms.intern(s)
    }

    /// Resolves an atom ID to its string value.
    #[inline]
    pub fn resolve_atom(&self, id: lyng_js_common::AtomId) -> &str {
        self.atoms.resolve(id)
    }

    /// Scans and returns the next token.
    pub fn next_token(&mut self) -> Token {
        // Handle template continuation mode specially.
        if self.mode == LexerMode::TemplateContinuation {
            self.mode = LexerMode::Normal;
            let token = self.scan_template_continuation();
            if token.kind != TokenKind::Eof {
                self.at_line_start = false;
            }
            return token;
        }

        self.skip_whitespace_and_comments();

        let mut flags = TokenFlags::empty();
        if self.saw_line_terminator {
            flags |= TokenFlags::PRECEDED_BY_LINE_TERMINATOR;
        }
        self.saw_line_terminator = false;

        let start = self.pos;

        if self.at_end() {
            return Token::new(TokenKind::Eof, self.span(start, start), flags);
        }

        let ch = self.current();

        // Fast dispatch on first byte.
        let token = match ch {
            // Identifiers and keywords
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' => self.scan_identifier(start, flags),
            b'\\' => self.scan_identifier(start, flags),

            // Digits
            b'0'..=b'9' => self.scan_numeric(start, flags),

            // String literals
            b'\'' | b'"' => self.scan_string(start, flags),

            // Template literals
            b'`' => self.scan_template_head(start, flags),

            // Punctuators
            b'(' => self.single(TokenKind::LParen, start, flags),
            b')' => self.single(TokenKind::RParen, start, flags),
            b'{' => self.single(TokenKind::LBrace, start, flags),
            b'}' => self.single(TokenKind::RBrace, start, flags),
            b'[' => self.single(TokenKind::LBracket, start, flags),
            b']' => self.single(TokenKind::RBracket, start, flags),
            b';' => self.single(TokenKind::Semicolon, start, flags),
            b',' => self.single(TokenKind::Comma, start, flags),
            b'~' => self.single(TokenKind::Tilde, start, flags),
            b'@' => self.single(TokenKind::At, start, flags),

            b'.' => self.scan_dot(start, flags),
            b':' => self.single(TokenKind::Colon, start, flags),
            b'?' => self.scan_question(start, flags),

            b'+' => self.scan_plus(start, flags),
            b'-' => self.scan_minus(start, flags),
            b'*' => self.scan_star(start, flags),
            b'/' => self.scan_slash(start, flags),
            b'%' => self.scan_percent(start, flags),

            b'<' => self.scan_lt(start, flags),
            b'>' => self.scan_gt(start, flags),
            b'=' => self.scan_eq(start, flags),
            b'!' => self.scan_bang(start, flags),
            b'&' => self.scan_amp(start, flags),
            b'|' => self.scan_pipe(start, flags),
            b'^' => self.scan_caret(start, flags),

            b'#' => self.scan_hash(start, flags),

            // High bytes: possible Unicode identifier start
            0x80..=0xFF => self.scan_unicode_identifier(start, flags),

            _ => {
                self.advance();
                self.error(
                    start,
                    self.pos,
                    format!("unexpected character: {}", ch as char),
                );
                Token::new(TokenKind::Eof, self.span(start, self.pos), flags)
            }
        };

        if token.kind != TokenKind::Eof {
            self.at_line_start = false;
        }

        token
    }

    // =========================================================================
    // Core byte-level helpers
    // =========================================================================

    #[inline]
    fn at_end(&self) -> bool {
        self.pos >= self.source.len()
    }

    #[inline]
    fn current(&self) -> u8 {
        self.source[self.pos]
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        self.source.get(self.pos + 1).copied()
    }

    #[inline]
    fn peek2(&self) -> Option<u8> {
        self.source.get(self.pos + 2).copied()
    }

    #[inline]
    fn advance(&mut self) {
        self.pos += 1;
    }

    #[inline]
    fn advance_n(&mut self, n: usize) {
        self.pos += n;
    }

    #[inline]
    fn eat(&mut self, byte: u8) -> bool {
        if !self.at_end() && self.current() == byte {
            self.advance();
            true
        } else {
            false
        }
    }

    #[inline]
    fn span(&self, start: usize, end: usize) -> Span {
        Span::from_offsets(self.source_id, start as u32, end as u32)
    }

    fn error(&mut self, start: usize, end: usize, message: impl Into<String>) {
        self.diagnostics.error(self.span(start, end), message);
    }

    fn single(&mut self, kind: TokenKind, start: usize, flags: TokenFlags) -> Token {
        self.advance();
        Token::new(kind, self.span(start, self.pos), flags)
    }

    #[inline]
    fn utf8_text(&self, bytes: &'src [u8]) -> &'src str {
        std::str::from_utf8(bytes)
            .expect("lexer should only decode bytes that originate from valid UTF-8 source text")
    }

    #[inline]
    fn valid_utf8_prefix(&self, bytes: &'src [u8]) -> Option<&'src str> {
        match std::str::from_utf8(bytes) {
            Ok(text) => Some(text),
            Err(error) => {
                let valid_len = error.valid_up_to();
                (valid_len > 0).then(|| self.utf8_text(&bytes[..valid_len]))
            }
        }
    }

    #[inline]
    fn next_code_point_from(&self, start: usize) -> Option<char> {
        self.valid_utf8_prefix(&self.source[start..])?
            .chars()
            .next()
    }

    /// Source text slice for the given byte range.
    fn text(&self, start: usize, end: usize) -> &'src str {
        self.utf8_text(&self.source[start..end])
    }

    // =========================================================================
    // Whitespace and comments
    // =========================================================================

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            if self.at_end() {
                return;
            }
            match self.current() {
                // ASCII whitespace (non-line-terminator)
                b' ' | b'\t' | 0x0B | 0x0C => {
                    self.advance();
                }
                // Line terminators
                b'\n' => {
                    self.saw_line_terminator = true;
                    self.at_line_start = true;
                    self.advance();
                }
                b'\r' => {
                    self.saw_line_terminator = true;
                    self.at_line_start = true;
                    self.advance();
                    self.eat(b'\n'); // \r\n counts as one line terminator
                }
                // Unicode whitespace / line terminators
                0xC2 => {
                    // U+00A0 NO-BREAK SPACE = C2 A0
                    if self.peek() == Some(0xA0) {
                        self.advance_n(2);
                    } else {
                        return;
                    }
                }
                0xE1..=0xE3 => {
                    if self.try_skip_unicode_whitespace() {
                        // Handled
                    } else {
                        return;
                    }
                }
                0xEF => {
                    // U+FEFF ZERO WIDTH NO-BREAK SPACE (BOM) = EF BB BF
                    if self.peek() == Some(0xBB) && self.peek2() == Some(0xBF) {
                        self.advance_n(3);
                    } else {
                        return;
                    }
                }
                b'/' => {
                    if self.peek() == Some(b'/') {
                        self.skip_single_line_comment();
                    } else if self.peek() == Some(b'*') {
                        self.skip_multi_line_comment();
                    } else {
                        return;
                    }
                }
                b'<' if self.allow_html_comments && self.starts_with(b"<!--") => {
                    self.skip_single_line_html_open_comment();
                }
                b'-' if self.allow_html_comments
                    && self.at_line_start
                    && self.starts_with(b"-->") =>
                {
                    self.skip_single_line_html_close_comment();
                }
                b'#' if self.pos == 0 && self.peek() == Some(b'!') => {
                    self.skip_hashbang_comment();
                }
                _ => return,
            }
        }
    }

    /// Try to skip selected 3-byte Unicode whitespace / line terminator
    /// sequences.
    /// Returns true if something was consumed.
    fn try_skip_unicode_whitespace(&mut self) -> bool {
        if self.pos + 2 >= self.source.len() {
            return false;
        }
        let lead = self.source[self.pos];
        let b1 = self.source[self.pos + 1];
        let b2 = self.source[self.pos + 2];
        match (lead, b1, b2) {
            // U+1680 OGHAM SPACE MARK = E1 9A 80
            (0xE1, 0x9A, 0x80) => {
                self.advance_n(3);
                true
            }
            // U+2000..U+200A (EN QUAD through HAIR SPACE) = E2 80 80..E2 80 8A
            (0xE2, 0x80, 0x80..=0x8A) => {
                self.advance_n(3);
                true
            }
            // U+2028 LINE SEPARATOR = E2 80 A8
            (0xE2, 0x80, 0xA8) => {
                self.saw_line_terminator = true;
                self.at_line_start = true;
                self.advance_n(3);
                true
            }
            // U+2029 PARAGRAPH SEPARATOR = E2 80 A9
            (0xE2, 0x80, 0xA9) => {
                self.saw_line_terminator = true;
                self.at_line_start = true;
                self.advance_n(3);
                true
            }
            // U+202F NARROW NO-BREAK SPACE = E2 80 AF
            (0xE2, 0x80, 0xAF) => {
                self.advance_n(3);
                true
            }
            // U+205F MEDIUM MATHEMATICAL SPACE = E2 81 9F
            (0xE2, 0x81, 0x9F) => {
                self.advance_n(3);
                true
            }
            // U+3000 IDEOGRAPHIC SPACE = E3 80 80
            (0xE3, 0x80, 0x80) => {
                self.advance_n(3);
                true
            }
            _ => false,
        }
    }

    #[inline]
    fn starts_with(&self, prefix: &[u8]) -> bool {
        self.source[self.pos..].starts_with(prefix)
    }

    fn skip_single_line_comment(&mut self) {
        self.advance_n(2); // skip //
        while !self.at_end() {
            match self.current() {
                b'\n' | b'\r' => return, // Don't consume the line terminator
                0xE2 if self.pos + 2 < self.source.len() => {
                    let b1 = self.source[self.pos + 1];
                    let b2 = self.source[self.pos + 2];
                    if b1 == 0x80 && (b2 == 0xA8 || b2 == 0xA9) {
                        // U+2028 or U+2029 are line terminators
                        return;
                    }
                    self.advance();
                }
                _ => self.advance(),
            }
        }
    }

    fn skip_single_line_html_open_comment(&mut self) {
        self.advance_n(4); // skip <!--
        while !self.at_end() {
            match self.current() {
                b'\n' | b'\r' => return,
                0xE2 if self.pos + 2 < self.source.len() => {
                    let b1 = self.source[self.pos + 1];
                    let b2 = self.source[self.pos + 2];
                    if b1 == 0x80 && (b2 == 0xA8 || b2 == 0xA9) {
                        return;
                    }
                    self.advance();
                }
                _ => self.advance(),
            }
        }
    }

    fn skip_single_line_html_close_comment(&mut self) {
        self.advance_n(3); // skip -->
        self.saw_line_terminator = true;
        while !self.at_end() {
            match self.current() {
                b'\n' | b'\r' => return,
                0xE2 if self.pos + 2 < self.source.len() => {
                    let b1 = self.source[self.pos + 1];
                    let b2 = self.source[self.pos + 2];
                    if b1 == 0x80 && (b2 == 0xA8 || b2 == 0xA9) {
                        return;
                    }
                    self.advance();
                }
                _ => self.advance(),
            }
        }
    }

    fn skip_multi_line_comment(&mut self) {
        let start = self.pos;
        self.advance_n(2); // skip /*
        while !self.at_end() {
            match self.current() {
                b'*' if self.peek() == Some(b'/') => {
                    self.advance_n(2);
                    return;
                }
                b'\n' => {
                    self.saw_line_terminator = true;
                    self.at_line_start = true;
                    self.advance();
                }
                b'\r' => {
                    self.saw_line_terminator = true;
                    self.at_line_start = true;
                    self.advance();
                    self.eat(b'\n');
                }
                0xE2 if self.pos + 2 < self.source.len() => {
                    let b1 = self.source[self.pos + 1];
                    let b2 = self.source[self.pos + 2];
                    if b1 == 0x80 && (b2 == 0xA8 || b2 == 0xA9) {
                        self.saw_line_terminator = true;
                        self.at_line_start = true;
                    }
                    self.advance();
                }
                _ => self.advance(),
            }
        }
        self.error(start, self.pos, "unterminated block comment");
    }

    fn skip_hashbang_comment(&mut self) {
        self.advance_n(2); // skip #!
        while !self.at_end() {
            match self.current() {
                b'\n' | b'\r' => return,
                // LS (U+2028) = E2 80 A8, PS (U+2029) = E2 80 A9
                0xE2 if self.pos + 2 < self.source.len()
                    && self.source[self.pos + 1] == 0x80
                    && (self.source[self.pos + 2] == 0xA8 || self.source[self.pos + 2] == 0xA9) =>
                {
                    return;
                }
                _ => self.advance(),
            }
        }
    }

    // =========================================================================
    // Identifiers and keywords
    // =========================================================================

    fn scan_identifier(&mut self, start: usize, flags: TokenFlags) -> Token {
        let mut has_escape = false;
        let mut buf: Option<String> = None;

        // Handle first character
        if self.current() == b'\\' {
            has_escape = true;
            let mut b = String::new();
            if let Some(ch) = self.scan_unicode_escape_sequence() {
                if !unicode_id_start::is_id_start(ch) && ch != '$' && ch != '_' {
                    self.error(
                        start,
                        self.pos,
                        "invalid identifier start character in escape",
                    );
                }
                b.push(ch);
            } else {
                self.error(start, self.pos, "invalid Unicode escape sequence");
            }
            buf = Some(b);
        } else {
            // Plain ASCII or Unicode start
            self.advance_identifier_start();
        }

        // Continue characters
        loop {
            if self.at_end() {
                break;
            }
            match self.current() {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'$' => {
                    if let Some(ref mut b) = buf {
                        b.push(self.current() as char);
                    }
                    self.advance();
                }
                b'\\' => {
                    // Start building a buffer if we haven't already
                    if buf.is_none() {
                        buf = Some(self.text(start, self.pos).to_owned());
                    }
                    has_escape = true;
                    if let Some(ch) = self.scan_unicode_escape_sequence() {
                        if !unicode_id_start::is_id_continue(ch)
                            && ch != '$'
                            && ch != '\u{200C}'
                            && ch != '\u{200D}'
                        {
                            self.error(
                                start,
                                self.pos,
                                "invalid identifier continue character in escape",
                            );
                        }
                        buf.as_mut().unwrap().push(ch);
                    } else {
                        self.error(start, self.pos, "invalid Unicode escape sequence");
                    }
                }
                0x80..=0xFF => {
                    // Multi-byte UTF-8 continue character
                    let Some(ch) = self.next_code_point_from(self.pos) else {
                        break;
                    };
                    if unicode_id_start::is_id_continue(ch) || ch == '\u{200C}' || ch == '\u{200D}'
                    {
                        if let Some(ref mut b) = buf {
                            b.push(ch);
                        }
                        self.advance_n(ch.len_utf8());
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }

        let mut result_flags = flags;
        if has_escape {
            result_flags |= TokenFlags::CONTAINS_ESCAPE;
        }

        // Get the identifier text
        let ident_text = match &buf {
            Some(b) => b.as_str(),
            None => self.text(start, self.pos),
        };

        // Check for keyword (only if no escape sequences)
        if !has_escape {
            if let Some(kw_atom_id) = self.atoms.keyword_atom(ident_text) {
                let kind = KEYWORD_TOKEN_KIND[kw_atom_id.raw() as usize];
                return Token::new(kind, self.span(start, self.pos), result_flags);
            }
        }

        // It's an identifier. Intern it.
        let atom = self.atoms.intern(ident_text);
        Token::with_payload(
            TokenKind::Identifier,
            self.span(start, self.pos),
            result_flags,
            TokenPayload::Atom(atom),
        )
    }

    /// Advance past an identifier start character (non-escape, non-backslash).
    fn advance_identifier_start(&mut self) {
        let b = self.current();
        if b < 0x80 {
            self.advance();
        } else {
            if let Some(ch) = self.next_code_point_from(self.pos) {
                self.advance_n(ch.len_utf8());
            } else {
                self.advance();
            }
        }
    }

    /// Scan a Unicode identifier that starts with a multi-byte UTF-8 character.
    fn scan_unicode_identifier(&mut self, start: usize, flags: TokenFlags) -> Token {
        let Some(ch) = self.next_code_point_from(self.pos) else {
            self.advance();
            self.error(start, self.pos, "invalid UTF-8 byte");
            return Token::new(TokenKind::Eof, self.span(start, self.pos), flags);
        };
        if unicode_id_start::is_id_start(ch) {
            self.scan_identifier(start, flags)
        } else {
            self.advance_n(ch.len_utf8());
            self.error(start, self.pos, format!("unexpected character: {ch}"));
            Token::new(TokenKind::Eof, self.span(start, self.pos), flags)
        }
    }

    /// Scans `\uXXXX` or `\u{XXXX}` and returns the decoded character.
    fn scan_unicode_escape_sequence(&mut self) -> Option<char> {
        debug_assert!(self.current() == b'\\');
        self.advance(); // skip `\`
        if self.at_end() || self.current() != b'u' {
            return None;
        }
        self.advance(); // skip `u`
        if self.at_end() {
            return None;
        }
        if self.current() == b'{' {
            // \u{XXXX}
            self.advance();
            let hex_start = self.pos;
            while !self.at_end() && self.current() != b'}' {
                self.advance();
            }
            if self.at_end() {
                return None;
            }
            let hex = self.text(hex_start, self.pos);
            self.advance(); // skip `}`
            let code = u32::from_str_radix(hex, 16).ok()?;
            char::from_u32(code)
        } else {
            // \uXXXX (exactly 4 hex digits)
            if self.pos + 4 > self.source.len() {
                return None;
            }
            let hex = self.text(self.pos, self.pos + 4);
            let code = u32::from_str_radix(hex, 16).ok()?;
            self.advance_n(4);
            char::from_u32(code)
        }
    }

    /// Scans a string-literal Unicode escape, combining surrogate pairs when
    /// they appear as adjacent `\uXXXX\uXXXX` escapes and preserving lone
    /// surrogate code units for JS string-literal semantics.
    fn scan_string_unicode_escape_sequence(&mut self) -> Option<UnicodeStringEscape> {
        debug_assert!(self.current() == b'\\');
        self.advance(); // skip `\`
        if self.at_end() || self.current() != b'u' {
            return None;
        }
        self.advance(); // skip `u`
        if self.at_end() {
            return None;
        }

        if self.current() == b'{' {
            self.advance();
            let hex_start = self.pos;
            while !self.at_end() && self.current() != b'}' {
                self.advance();
            }
            if self.at_end() {
                return None;
            }
            let hex = self.text(hex_start, self.pos);
            self.advance(); // skip `}`
            let code = u32::from_str_radix(hex, 16).ok()?;
            return char::from_u32(code).map(UnicodeStringEscape::Scalar);
        }

        if self.pos + 4 > self.source.len() {
            return None;
        }
        let hex = self.text(self.pos, self.pos + 4);
        let code_unit = u16::from_str_radix(hex, 16).ok()?;
        self.advance_n(4);

        if (0xD800..=0xDBFF).contains(&code_unit) {
            if self.pos + 6 <= self.source.len()
                && self.source[self.pos] == b'\\'
                && self.source[self.pos + 1] == b'u'
            {
                let low_hex = self.text(self.pos + 2, self.pos + 6);
                if let Ok(low) = u16::from_str_radix(low_hex, 16) {
                    if (0xDC00..=0xDFFF).contains(&low) {
                        self.advance_n(6);
                        let combined = 0x10000
                            + ((u32::from(code_unit) - 0xD800) << 10)
                            + (u32::from(low) - 0xDC00);
                        return char::from_u32(combined).map(UnicodeStringEscape::Scalar);
                    }
                }
            }
            return Some(UnicodeStringEscape::CodeUnit(code_unit));
        }

        if (0xDC00..=0xDFFF).contains(&code_unit) {
            return Some(UnicodeStringEscape::CodeUnit(code_unit));
        }

        char::from_u32(u32::from(code_unit)).map(UnicodeStringEscape::Scalar)
    }

    // =========================================================================
    // Numeric literals
    // =========================================================================

    fn scan_numeric(&mut self, start: usize, flags: TokenFlags) -> Token {
        if self.current() == b'0' {
            if let Some(next) = self.peek() {
                match next {
                    b'x' | b'X' => return self.scan_hex(start, flags),
                    b'o' | b'O' => return self.scan_octal(start, flags),
                    b'b' | b'B' => return self.scan_binary(start, flags),
                    _ => {}
                }
            }
        }
        self.scan_decimal(start, flags)
    }

    fn scan_decimal(&mut self, start: usize, flags: TokenFlags) -> Token {
        let mut result_flags = flags;
        self.scan_decimal_digits();
        let integer_end = self.pos;

        let mut is_float = false;
        let integer_text = self.text(start, integer_end);
        if is_legacy_octal_like_decimal(integer_text) {
            result_flags |= TokenFlags::LEGACY_OCTAL_LIKE_DECIMAL;
        }
        if has_invalid_legacy_decimal_separator(integer_text) {
            self.error(
                start,
                integer_end,
                "numeric separator not allowed in a legacy octal-like decimal literal",
            );
        }

        // Check for BigInt
        if !self.at_end() && self.current() == b'n' {
            self.advance();
            let raw = self.text(start, self.pos - 1).replace('_', "");
            if has_invalid_bigint_decimal_leading_zero(integer_text) {
                self.error(start, self.pos, "invalid bigint literal with leading zero");
            }
            self.check_numeric_literal_terminator(start);
            let id = self.literals.push_bigint(raw);
            return Token::with_payload(
                TokenKind::BigIntLiteral,
                self.span(start, self.pos),
                result_flags,
                TokenPayload::Literal(id),
            );
        }

        // Decimal point
        if !self.at_end() && self.current() == b'.' {
            if self.peek() == Some(b'_') {
                self.error(
                    self.pos,
                    self.pos + 2,
                    "numeric separator may not be adjacent to '.'",
                );
            }
            if let Some(next) = self.peek() {
                if next.is_ascii_digit() {
                    is_float = true;
                    self.advance(); // skip `.`
                    self.scan_decimal_digits();
                } else if matches!(next, b'e' | b'E' | b'.')
                    || !matches!(
                        next,
                        b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' | b'\\' | 0x80..=0xFF
                    )
                {
                    is_float = true;
                    self.advance(); // trailing `.`
                }
            } else {
                // `.` at end of input - also fine, treat as float
                is_float = true;
                self.advance();
            }
        }

        // Exponent
        if !self.at_end() && matches!(self.current(), b'e' | b'E') {
            is_float = true;
            self.advance();
            if !self.at_end() && matches!(self.current(), b'+' | b'-') {
                self.advance();
            }
            if self.at_end() || !self.current().is_ascii_digit() {
                self.error(start, self.pos, "expected digit after exponent");
            } else {
                self.scan_decimal_digits();
            }
        }

        let text = self.text(start, self.pos);
        let clean: Cow<str> = if text.contains('_') {
            Cow::Owned(text.replace('_', ""))
        } else {
            Cow::Borrowed(text)
        };
        let value: f64 = if !is_float && is_legacy_octal_integer(integer_text) {
            parse_octal_to_f64(&clean[1..])
        } else {
            clean.parse().unwrap_or_else(|_| {
                self.error(start, self.pos, "invalid numeric literal");
                f64::NAN
            })
        };

        self.check_numeric_literal_terminator(start);
        Token::with_payload(
            TokenKind::NumericLiteral,
            self.span(start, self.pos),
            result_flags,
            TokenPayload::Number(value.to_bits()),
        )
    }

    fn scan_decimal_digits(&mut self) {
        while !self.at_end() {
            match self.current() {
                b'0'..=b'9' => self.advance(),
                b'_' => {
                    self.advance();
                    // Separator must be followed by a digit
                    if self.at_end() || !self.current().is_ascii_digit() {
                        let pos = self.pos;
                        self.error(pos - 1, pos, "numeric separator not followed by digit");
                    }
                }
                _ => break,
            }
        }
    }

    fn scan_hex(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance_n(2); // skip 0x
        let digit_start = self.pos;
        self.scan_hex_digits();
        if self.pos == digit_start {
            self.error(start, self.pos, "expected hex digit after 0x");
        }

        // BigInt?
        if !self.at_end() && self.current() == b'n' {
            self.advance();
            let raw = self.text(start, self.pos - 1).replace('_', "");
            self.check_numeric_literal_terminator(start);
            let id = self.literals.push_bigint(raw);
            return Token::with_payload(
                TokenKind::BigIntLiteral,
                self.span(start, self.pos),
                flags,
                TokenPayload::Literal(id),
            );
        }

        let text = self.text(start, self.pos);
        let clean: Cow<str> = if text.contains('_') {
            Cow::Owned(text.replace('_', ""))
        } else {
            Cow::Borrowed(text)
        };
        // Parse hex manually to get exact integer when possible
        let value = parse_hex_to_f64(&clean[2..]);
        self.check_numeric_literal_terminator(start);

        Token::with_payload(
            TokenKind::NumericLiteral,
            self.span(start, self.pos),
            flags,
            TokenPayload::Number(value.to_bits()),
        )
    }

    fn scan_hex_digits(&mut self) {
        let mut saw_digit = false;
        while !self.at_end() {
            match self.current() {
                b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => {
                    saw_digit = true;
                    self.advance();
                }
                b'_' => {
                    if !saw_digit {
                        self.error(
                            self.pos,
                            self.pos + 1,
                            "numeric separator may not follow 0x",
                        );
                    }
                    self.advance();
                    if self.at_end() || !self.current().is_ascii_hexdigit() {
                        let pos = self.pos;
                        self.error(pos - 1, pos, "numeric separator not followed by hex digit");
                    }
                }
                _ => break,
            }
        }
    }

    fn scan_octal(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance_n(2); // skip 0o
        let digit_start = self.pos;
        self.scan_octal_digits();
        if self.pos == digit_start {
            self.error(start, self.pos, "expected octal digit after 0o");
        }

        // BigInt?
        if !self.at_end() && self.current() == b'n' {
            self.advance();
            let raw = self.text(start, self.pos - 1).replace('_', "");
            self.check_numeric_literal_terminator(start);
            let id = self.literals.push_bigint(raw);
            return Token::with_payload(
                TokenKind::BigIntLiteral,
                self.span(start, self.pos),
                flags,
                TokenPayload::Literal(id),
            );
        }

        let text = self.text(start, self.pos);
        let clean: Cow<str> = if text.contains('_') {
            Cow::Owned(text.replace('_', ""))
        } else {
            Cow::Borrowed(text)
        };
        let value = parse_octal_to_f64(&clean[2..]);
        self.check_numeric_literal_terminator(start);

        Token::with_payload(
            TokenKind::NumericLiteral,
            self.span(start, self.pos),
            flags,
            TokenPayload::Number(value.to_bits()),
        )
    }

    fn scan_octal_digits(&mut self) {
        let mut saw_digit = false;
        while !self.at_end() {
            match self.current() {
                b'0'..=b'7' => {
                    saw_digit = true;
                    self.advance();
                }
                b'_' => {
                    if !saw_digit {
                        self.error(
                            self.pos,
                            self.pos + 1,
                            "numeric separator may not follow 0o",
                        );
                    }
                    self.advance();
                    if self.at_end() || !matches!(self.source.get(self.pos), Some(b'0'..=b'7')) {
                        let pos = self.pos;
                        self.error(
                            pos - 1,
                            pos,
                            "numeric separator not followed by octal digit",
                        );
                    }
                }
                _ => break,
            }
        }
    }

    fn scan_binary(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance_n(2); // skip 0b
        let digit_start = self.pos;
        self.scan_binary_digits();
        if self.pos == digit_start {
            self.error(start, self.pos, "expected binary digit after 0b");
        }

        // BigInt?
        if !self.at_end() && self.current() == b'n' {
            self.advance();
            let raw = self.text(start, self.pos - 1).replace('_', "");
            self.check_numeric_literal_terminator(start);
            let id = self.literals.push_bigint(raw);
            return Token::with_payload(
                TokenKind::BigIntLiteral,
                self.span(start, self.pos),
                flags,
                TokenPayload::Literal(id),
            );
        }

        let text = self.text(start, self.pos);
        let clean: Cow<str> = if text.contains('_') {
            Cow::Owned(text.replace('_', ""))
        } else {
            Cow::Borrowed(text)
        };
        let value = parse_binary_to_f64(&clean[2..]);
        self.check_numeric_literal_terminator(start);

        Token::with_payload(
            TokenKind::NumericLiteral,
            self.span(start, self.pos),
            flags,
            TokenPayload::Number(value.to_bits()),
        )
    }

    fn scan_binary_digits(&mut self) {
        let mut saw_digit = false;
        while !self.at_end() {
            match self.current() {
                b'0' | b'1' => {
                    saw_digit = true;
                    self.advance();
                }
                b'_' => {
                    if !saw_digit {
                        self.error(
                            self.pos,
                            self.pos + 1,
                            "numeric separator may not follow 0b",
                        );
                    }
                    self.advance();
                    if self.at_end() || !matches!(self.source.get(self.pos), Some(b'0' | b'1')) {
                        let pos = self.pos;
                        self.error(
                            pos - 1,
                            pos,
                            "numeric separator not followed by binary digit",
                        );
                    }
                }
                _ => break,
            }
        }
    }

    // =========================================================================
    // String literals
    // =========================================================================

    fn scan_string(&mut self, start: usize, flags: TokenFlags) -> Token {
        let quote = self.current();
        self.advance(); // skip opening quote

        let mut value = CookedStringBuffer::default();
        let mut has_escape = false;
        let mut result_flags = flags;

        loop {
            if self.at_end() {
                self.error(start, self.pos, "unterminated string literal");
                break;
            }
            let ch = self.current();
            match ch {
                b if b == quote => {
                    self.advance(); // skip closing quote
                    break;
                }
                b'\\' => {
                    has_escape = true;
                    self.advance(); // skip backslash
                    if self.at_end() {
                        self.error(start, self.pos, "unterminated string literal");
                        break;
                    }
                    self.scan_string_escape(&mut value, &mut result_flags);
                }
                b'\n' | b'\r' => {
                    self.error(start, self.pos, "unterminated string literal");
                    break;
                }
                0x80..=0xFF => {
                    // Multi-byte UTF-8
                    let rest = &self.source[self.pos..];
                    if let Some(ch) = std::str::from_utf8(rest)
                        .ok()
                        .and_then(|s| s.chars().next())
                    {
                        value.push_char(ch);
                        self.advance_n(ch.len_utf8());
                    } else {
                        self.advance();
                    }
                }
                _ => {
                    value.push_char(ch as char);
                    self.advance();
                }
            }
        }

        if has_escape {
            result_flags |= TokenFlags::CONTAINS_ESCAPE;
        }

        let id = self.literals.push_string(value.into_literal());
        Token::with_payload(
            TokenKind::StringLiteral,
            self.span(start, self.pos),
            result_flags,
            TokenPayload::Literal(id),
        )
    }

    /// Scan a single escape character after the backslash has been consumed.
    fn scan_string_escape(&mut self, buf: &mut CookedStringBuffer, flags: &mut TokenFlags) {
        let escape_start = self.pos;
        let ch = self.current();
        self.advance();
        match ch {
            b'n' => buf.push_char('\n'),
            b'r' => buf.push_char('\r'),
            b't' => buf.push_char('\t'),
            b'b' => buf.push_char('\u{0008}'),
            b'f' => buf.push_char('\u{000C}'),
            b'v' => buf.push_char('\u{000B}'),
            b'0' => {
                // \0 is NUL, but \01 etc. is legacy octal
                if self.at_end() {
                    buf.push_char('\0');
                } else {
                    match self.current() {
                        b'0'..=b'7' => {
                            *flags |= TokenFlags::LEGACY_OCTAL_ESCAPE;
                            let val = self.scan_legacy_octal_escape(ch);
                            if let Some(c) = char::from_u32(val) {
                                buf.push_char(c);
                            }
                        }
                        b'8' | b'9' => {
                            *flags |= TokenFlags::NON_OCTAL_DECIMAL_ESCAPE;
                            buf.push_char('\0');
                        }
                        _ => buf.push_char('\0'),
                    }
                }
            }
            b'1'..=b'7' => {
                // Legacy octal escape
                *flags |= TokenFlags::LEGACY_OCTAL_ESCAPE;
                let val = self.scan_legacy_octal_escape(ch);
                if let Some(c) = char::from_u32(val) {
                    buf.push_char(c);
                }
            }
            b'8' | b'9' => {
                *flags |= TokenFlags::NON_OCTAL_DECIMAL_ESCAPE;
                buf.push_char(ch as char);
            }
            b'x' => {
                // \xHH
                if self.pos + 2 <= self.source.len() {
                    let hex = self.text(self.pos, self.pos + 2);
                    if let Ok(code) = u8::from_str_radix(hex, 16) {
                        self.advance_n(2);
                        buf.push_char(code as char);
                    } else {
                        self.error(self.pos - 2, self.pos, "invalid hex escape");
                    }
                } else {
                    self.error(self.pos - 2, self.pos, "invalid hex escape");
                }
            }
            b'u' => {
                // Re-parse from before the `u`
                self.pos -= 1; // back up to `u`
                self.pos -= 1; // back up to `\`
                if let Some(c) = self.scan_string_unicode_escape_sequence() {
                    match c {
                        UnicodeStringEscape::Scalar(ch) => buf.push_char(ch),
                        UnicodeStringEscape::CodeUnit(unit) => buf.push_code_unit(unit),
                    }
                } else {
                    self.error(self.pos - 1, self.pos, "invalid unicode escape");
                }
            }
            b'\r' => {
                // Line continuation: \<CR> or \<CR><LF>
                self.eat(b'\n');
            }
            b'\n' => {
                // Line continuation
            }
            0xE2 if escape_start + 2 < self.source.len()
                && self.source[escape_start + 1] == 0x80
                && (self.source[escape_start + 2] == 0xA8
                    || self.source[escape_start + 2] == 0xA9) =>
            {
                // Line continuation: \<LS> or \<PS>
                self.pos = escape_start + 3;
            }
            0x80..=0xFF => {
                let rest = &self.source[escape_start..];
                if let Some(ch) = std::str::from_utf8(rest)
                    .ok()
                    .and_then(|s| s.chars().next())
                {
                    buf.push_char(ch);
                    self.pos = escape_start + ch.len_utf8();
                } else {
                    buf.push_char(ch as char);
                }
            }
            _ => {
                // Identity escape
                buf.push_char(ch as char);
            }
        }
    }

    fn scan_legacy_octal_escape(&mut self, first: u8) -> u32 {
        let mut val = u32::from(first - b'0');
        if !self.at_end() && matches!(self.current(), b'0'..=b'7') {
            val = val * 8 + u32::from(self.current() - b'0');
            self.advance();
            if first <= b'3' && !self.at_end() && matches!(self.current(), b'0'..=b'7') {
                val = val * 8 + u32::from(self.current() - b'0');
                self.advance();
            }
        }
        val
    }

    fn check_numeric_literal_terminator(&mut self, start: usize) {
        if self.at_end() {
            return;
        }
        let ch = self.current();
        if ch.is_ascii_digit() || self.current_is_identifier_start() {
            self.error(
                start,
                self.pos,
                "numeric literal may not be immediately followed by an identifier or digit",
            );
        }
    }

    fn current_is_identifier_start(&self) -> bool {
        if self.at_end() {
            return false;
        }
        match self.current() {
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' | b'\\' => true,
            0x80..=0xFF => self
                .next_code_point_from(self.pos)
                .is_some_and(unicode_id_start::is_id_start),
            _ => false,
        }
    }

    // =========================================================================
    // Template literals
    // =========================================================================

    fn scan_template_head(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance(); // skip opening backtick
        let (cooked, raw, terminated_by) = self.scan_template_chars();

        match terminated_by {
            TemplateEnd::Backtick => {
                let id = self.literals.push_template(cooked, raw);
                Token::with_payload(
                    TokenKind::NoSubstitutionTemplate,
                    self.span(start, self.pos),
                    flags,
                    TokenPayload::Literal(id),
                )
            }
            TemplateEnd::DollarBrace => {
                let id = self.literals.push_template(cooked, raw);
                Token::with_payload(
                    TokenKind::TemplateHead,
                    self.span(start, self.pos),
                    flags,
                    TokenPayload::Literal(id),
                )
            }
            TemplateEnd::Eof => {
                self.error(start, self.pos, "unterminated template literal");
                let id = self.literals.push_template(cooked, raw);
                Token::with_payload(
                    TokenKind::NoSubstitutionTemplate,
                    self.span(start, self.pos),
                    flags,
                    TokenPayload::Literal(id),
                )
            }
        }
    }

    fn scan_template_continuation(&mut self) -> Token {
        let start = self.pos;
        // We're positioned right after `}` (the parser consumed the expression
        // and set mode to TemplateContinuation). But actually the `}` hasn't
        // been consumed by us - the parser has already read the `}` token.
        // We scan from current position which is after the `}`.
        let mut flags = TokenFlags::empty();
        if self.saw_line_terminator {
            flags |= TokenFlags::PRECEDED_BY_LINE_TERMINATOR;
        }
        self.saw_line_terminator = false;

        let (cooked, raw, terminated_by) = self.scan_template_chars();

        match terminated_by {
            TemplateEnd::Backtick => {
                let id = self.literals.push_template(cooked, raw);
                Token::with_payload(
                    TokenKind::TemplateTail,
                    self.span(start, self.pos),
                    flags,
                    TokenPayload::Literal(id),
                )
            }
            TemplateEnd::DollarBrace => {
                let id = self.literals.push_template(cooked, raw);
                Token::with_payload(
                    TokenKind::TemplateMiddle,
                    self.span(start, self.pos),
                    flags,
                    TokenPayload::Literal(id),
                )
            }
            TemplateEnd::Eof => {
                self.error(start, self.pos, "unterminated template literal");
                let id = self.literals.push_template(cooked, raw);
                Token::with_payload(
                    TokenKind::TemplateTail,
                    self.span(start, self.pos),
                    flags,
                    TokenPayload::Literal(id),
                )
            }
        }
    }

    fn scan_template_chars(&mut self) -> (Option<String>, String, TemplateEnd) {
        let mut cooked = Some(String::new());
        let mut raw = String::new();

        loop {
            if self.at_end() {
                return (cooked, raw, TemplateEnd::Eof);
            }
            let ch = self.current();
            match ch {
                b'`' => {
                    self.advance();
                    return (cooked, raw, TemplateEnd::Backtick);
                }
                b'$' if self.peek() == Some(b'{') => {
                    self.advance_n(2);
                    return (cooked, raw, TemplateEnd::DollarBrace);
                }
                b'\\' => {
                    raw.push('\\');
                    self.advance();
                    if self.at_end() {
                        return (cooked, raw, TemplateEnd::Eof);
                    }
                    if self.current() == b'\r' {
                        raw.push('\n');
                        self.advance();
                        if !self.at_end() && self.current() == b'\n' {
                            self.advance();
                        }
                        continue;
                    }
                    if self.current() == b'\n' {
                        raw.push('\n');
                        self.advance();
                        continue;
                    }
                    if self.source[self.pos..].starts_with(&[0xE2, 0x80, 0xA8]) {
                        raw.push('\u{2028}');
                        self.advance_n(3);
                        continue;
                    }
                    if self.source[self.pos..].starts_with(&[0xE2, 0x80, 0xA9]) {
                        raw.push('\u{2029}');
                        self.advance_n(3);
                        continue;
                    }
                    let esc_ch = self.current();
                    raw.push(esc_ch as char);
                    self.advance();
                    // For cooked value, process the escape
                    match esc_ch {
                        b'n' => {
                            if let Some(ref mut c) = cooked {
                                c.push('\n');
                            }
                        }
                        b'r' => {
                            if let Some(ref mut c) = cooked {
                                c.push('\r');
                            }
                        }
                        b't' => {
                            if let Some(ref mut c) = cooked {
                                c.push('\t');
                            }
                        }
                        b'b' => {
                            if let Some(ref mut c) = cooked {
                                c.push('\u{0008}');
                            }
                        }
                        b'f' => {
                            if let Some(ref mut c) = cooked {
                                c.push('\u{000C}');
                            }
                        }
                        b'v' => {
                            if let Some(ref mut c) = cooked {
                                c.push('\u{000B}');
                            }
                        }
                        b'0' => {
                            if !self.at_end() && self.current().is_ascii_digit() {
                                // Not allowed in template - cooked becomes None
                                cooked = None;
                                // Skip remaining octal digits for raw
                                while !self.at_end() && self.current().is_ascii_digit() {
                                    raw.push(self.current() as char);
                                    self.advance();
                                }
                            } else if let Some(ref mut c) = cooked {
                                c.push('\0');
                            }
                        }
                        b'x' => {
                            // \xHH
                            if self.pos + 2 <= self.source.len() {
                                let h = self.text(self.pos, self.pos + 2);
                                if h.as_bytes().iter().all(u8::is_ascii_hexdigit) {
                                    raw.push_str(h);
                                    if let Ok(code) = u8::from_str_radix(h, 16) {
                                        if let Some(ref mut c) = cooked {
                                            c.push(code as char);
                                        }
                                    } else {
                                        cooked = None;
                                    }
                                    self.advance_n(2);
                                } else {
                                    cooked = None;
                                }
                            } else {
                                cooked = None;
                            }
                        }
                        b'u' => {
                            if !self.at_end() && self.current() == b'{' {
                                raw.push('{');
                                self.advance();

                                let hex_start = self.pos;
                                while !self.at_end()
                                    && self.current() != b'}'
                                    && self.current() != b'`'
                                    && !(self.current() == b'$' && self.peek() == Some(b'{'))
                                {
                                    raw.push(self.current() as char);
                                    self.advance();
                                }

                                if !self.at_end() && self.current() == b'}' {
                                    raw.push('}');
                                    let hex = self.text(hex_start, self.pos);
                                    self.advance();
                                    if let Ok(code) = u32::from_str_radix(hex, 16) {
                                        if let Some(ch) = char::from_u32(code) {
                                            if let Some(ref mut ck) = cooked {
                                                ck.push(ch);
                                            }
                                        } else {
                                            cooked = None;
                                        }
                                    } else {
                                        cooked = None;
                                    }
                                } else {
                                    cooked = None;
                                }
                            } else {
                                let mut hex = String::new();
                                for _ in 0..4 {
                                    if self.at_end()
                                        || self.current() == b'`'
                                        || (self.current() == b'$' && self.peek() == Some(b'{'))
                                    {
                                        break;
                                    }
                                    raw.push(self.current() as char);
                                    hex.push(self.current() as char);
                                    self.advance();
                                }

                                if hex.len() == 4
                                    && hex.as_bytes().iter().all(u8::is_ascii_hexdigit)
                                {
                                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                                        if let Some(ch) = char::from_u32(code) {
                                            if let Some(ref mut ck) = cooked {
                                                ck.push(ch);
                                            }
                                        } else {
                                            cooked = None;
                                        }
                                    } else {
                                        cooked = None;
                                    }
                                } else {
                                    cooked = None;
                                }
                            }
                        }
                        b'\r' => {
                            // Line continuation
                            if !self.at_end() && self.current() == b'\n' {
                                raw.push('\n');
                                self.advance();
                            }
                            // Cooked: nothing (line continuation)
                        }
                        b'\n' => {
                            // Line continuation - cooked: nothing
                        }
                        _ => {
                            // In tagged templates, unknown escapes make cooked undefined
                            // But for non-tagged, it's an identity escape.
                            // The spec says cooked is undefined for invalid escapes,
                            // but identity escapes like \a are valid (they produce 'a').
                            // Decimal escapes are invalid in templates.
                            if matches!(esc_ch, b'1'..=b'9') {
                                cooked = None;
                            } else if let Some(ref mut c) = cooked {
                                c.push(esc_ch as char);
                            }
                        }
                    }
                }
                b'\r' => {
                    self.advance();
                    raw.push('\n'); // normalize to LF in raw
                    if let Some(ref mut c) = cooked {
                        c.push('\n');
                    }
                    if !self.at_end() && self.current() == b'\n' {
                        self.advance();
                    }
                }
                b'\n' => {
                    self.advance();
                    raw.push('\n');
                    if let Some(ref mut c) = cooked {
                        c.push('\n');
                    }
                }
                _ => {
                    if ch >= 0x80 {
                        let rest = &self.source[self.pos..];
                        if let Some(c) = std::str::from_utf8(rest)
                            .ok()
                            .and_then(|s| s.chars().next())
                        {
                            raw.push(c);
                            if let Some(ref mut ck) = cooked {
                                ck.push(c);
                            }
                            self.advance_n(c.len_utf8());
                        } else {
                            self.advance();
                        }
                    } else {
                        raw.push(ch as char);
                        if let Some(ref mut c) = cooked {
                            c.push(ch as char);
                        }
                        self.advance();
                    }
                }
            }
        }
    }

    // =========================================================================
    // RegExp literals
    // =========================================================================

    fn scan_regexp(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance(); // skip opening `/`
        let mut pattern = String::new();
        let mut in_class = false;

        loop {
            if self.at_end() {
                self.error(start, self.pos, "unterminated regexp literal");
                break;
            }
            let ch = self.current();
            match ch {
                b'/' if !in_class => {
                    self.advance();
                    break;
                }
                b'[' => {
                    in_class = true;
                    pattern.push('[');
                    self.advance();
                }
                b']' => {
                    in_class = false;
                    pattern.push(']');
                    self.advance();
                }
                b'\\' => {
                    pattern.push('\\');
                    self.advance();
                    if !self.at_end() {
                        pattern.push(self.current() as char);
                        self.advance();
                    }
                }
                b'\n' | b'\r' => {
                    self.error(start, self.pos, "unterminated regexp literal");
                    break;
                }
                _ => {
                    if ch >= 0x80 {
                        let rest = &self.source[self.pos..];
                        if let Some(c) = std::str::from_utf8(rest)
                            .ok()
                            .and_then(|s| s.chars().next())
                        {
                            if c == '\u{2028}' || c == '\u{2029}' {
                                self.error(start, self.pos, "unterminated regexp literal");
                                break;
                            }
                            pattern.push(c);
                            self.advance_n(c.len_utf8());
                        } else {
                            self.advance();
                        }
                    } else {
                        pattern.push(ch as char);
                        self.advance();
                    }
                }
            }
        }

        // Scan flags
        let mut re_flags = String::new();
        while !self.at_end() && self.current().is_ascii_alphabetic() {
            re_flags.push(self.current() as char);
            self.advance();
        }

        let id = self.literals.push_regexp(pattern, re_flags);
        Token::with_payload(
            TokenKind::RegExpLiteral,
            self.span(start, self.pos),
            flags,
            TokenPayload::Literal(id),
        )
    }

    // =========================================================================
    // Punctuators
    // =========================================================================

    fn scan_dot(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance(); // skip first `.`
                        // Check for `...` (ellipsis)
        if !self.at_end() && self.current() == b'.' && self.peek() == Some(b'.') {
            self.advance_n(2);
            return Token::new(TokenKind::Ellipsis, self.span(start, self.pos), flags);
        }
        // Check for `.123` (numeric literal starting with dot)
        if !self.at_end() && self.current().is_ascii_digit() {
            // Back up and scan as decimal
            self.pos = start;
            return self.scan_dot_decimal(start, flags);
        }
        Token::new(TokenKind::Dot, self.span(start, self.pos), flags)
    }

    fn scan_dot_decimal(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance(); // skip `.`
        self.scan_decimal_digits();

        // Exponent
        if !self.at_end() && matches!(self.current(), b'e' | b'E') {
            self.advance();
            if !self.at_end() && matches!(self.current(), b'+' | b'-') {
                self.advance();
            }
            if self.at_end() || !self.current().is_ascii_digit() {
                self.error(start, self.pos, "expected digit after exponent");
            } else {
                self.scan_decimal_digits();
            }
        }

        let text = self.text(start, self.pos);
        let clean: Cow<str> = if text.contains('_') {
            Cow::Owned(text.replace('_', ""))
        } else {
            Cow::Borrowed(text)
        };
        let value: f64 = clean.parse().unwrap_or(f64::NAN);

        Token::with_payload(
            TokenKind::NumericLiteral,
            self.span(start, self.pos),
            flags,
            TokenPayload::Number(value.to_bits()),
        )
    }

    fn scan_question(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance(); // skip `?`
        if !self.at_end() {
            match self.current() {
                b'?' => {
                    self.advance();
                    if self.eat(b'=') {
                        return Token::new(
                            TokenKind::QuestionQuestionEq,
                            self.span(start, self.pos),
                            flags,
                        );
                    }
                    return Token::new(
                        TokenKind::QuestionQuestion,
                        self.span(start, self.pos),
                        flags,
                    );
                }
                b'.' => {
                    // `?.` but NOT `?.0` (which would be `?` followed by `.0`)
                    if let Some(next) = self.peek() {
                        if !next.is_ascii_digit() {
                            self.advance();
                            return Token::new(
                                TokenKind::OptionalChain,
                                self.span(start, self.pos),
                                flags,
                            );
                        }
                    } else {
                        self.advance();
                        return Token::new(
                            TokenKind::OptionalChain,
                            self.span(start, self.pos),
                            flags,
                        );
                    }
                }
                _ => {}
            }
        }
        Token::new(TokenKind::Question, self.span(start, self.pos), flags)
    }

    fn scan_plus(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance();
        if !self.at_end() {
            match self.current() {
                b'+' => {
                    self.advance();
                    return Token::new(TokenKind::PlusPlus, self.span(start, self.pos), flags);
                }
                b'=' => {
                    self.advance();
                    return Token::new(TokenKind::PlusEq, self.span(start, self.pos), flags);
                }
                _ => {}
            }
        }
        Token::new(TokenKind::Plus, self.span(start, self.pos), flags)
    }

    fn scan_minus(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance();
        if !self.at_end() {
            match self.current() {
                b'-' => {
                    self.advance();
                    return Token::new(TokenKind::MinusMinus, self.span(start, self.pos), flags);
                }
                b'=' => {
                    self.advance();
                    return Token::new(TokenKind::MinusEq, self.span(start, self.pos), flags);
                }
                _ => {}
            }
        }
        Token::new(TokenKind::Minus, self.span(start, self.pos), flags)
    }

    fn scan_star(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance();
        if !self.at_end() {
            match self.current() {
                b'*' => {
                    self.advance();
                    if self.eat(b'=') {
                        return Token::new(TokenKind::ExpEq, self.span(start, self.pos), flags);
                    }
                    return Token::new(TokenKind::Exp, self.span(start, self.pos), flags);
                }
                b'=' => {
                    self.advance();
                    return Token::new(TokenKind::StarEq, self.span(start, self.pos), flags);
                }
                _ => {}
            }
        }
        Token::new(TokenKind::Star, self.span(start, self.pos), flags)
    }

    fn scan_slash(&mut self, start: usize, flags: TokenFlags) -> Token {
        if self.mode == LexerMode::RegExp {
            self.mode = LexerMode::Normal;
            return self.scan_regexp(start, flags);
        }
        self.advance();
        if self.eat(b'=') {
            return Token::new(TokenKind::SlashEq, self.span(start, self.pos), flags);
        }
        Token::new(TokenKind::Slash, self.span(start, self.pos), flags)
    }

    fn scan_percent(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance();
        if self.eat(b'=') {
            return Token::new(TokenKind::PercentEq, self.span(start, self.pos), flags);
        }
        Token::new(TokenKind::Percent, self.span(start, self.pos), flags)
    }

    fn scan_lt(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance();
        if !self.at_end() {
            match self.current() {
                b'<' => {
                    self.advance();
                    if self.eat(b'=') {
                        return Token::new(TokenKind::LtLtEq, self.span(start, self.pos), flags);
                    }
                    return Token::new(TokenKind::LtLt, self.span(start, self.pos), flags);
                }
                b'=' => {
                    self.advance();
                    return Token::new(TokenKind::LtEq, self.span(start, self.pos), flags);
                }
                _ => {}
            }
        }
        Token::new(TokenKind::Lt, self.span(start, self.pos), flags)
    }

    fn scan_gt(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance();
        if !self.at_end() {
            match self.current() {
                b'>' => {
                    self.advance();
                    if !self.at_end() {
                        match self.current() {
                            b'>' => {
                                self.advance();
                                if self.eat(b'=') {
                                    return Token::new(
                                        TokenKind::GtGtGtEq,
                                        self.span(start, self.pos),
                                        flags,
                                    );
                                }
                                return Token::new(
                                    TokenKind::GtGtGt,
                                    self.span(start, self.pos),
                                    flags,
                                );
                            }
                            b'=' => {
                                self.advance();
                                return Token::new(
                                    TokenKind::GtGtEq,
                                    self.span(start, self.pos),
                                    flags,
                                );
                            }
                            _ => {}
                        }
                    }
                    return Token::new(TokenKind::GtGt, self.span(start, self.pos), flags);
                }
                b'=' => {
                    self.advance();
                    return Token::new(TokenKind::GtEq, self.span(start, self.pos), flags);
                }
                _ => {}
            }
        }
        Token::new(TokenKind::Gt, self.span(start, self.pos), flags)
    }

    fn scan_eq(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance();
        if !self.at_end() {
            match self.current() {
                b'=' => {
                    self.advance();
                    if self.eat(b'=') {
                        return Token::new(TokenKind::EqEqEq, self.span(start, self.pos), flags);
                    }
                    return Token::new(TokenKind::EqEq, self.span(start, self.pos), flags);
                }
                b'>' => {
                    self.advance();
                    return Token::new(TokenKind::Arrow, self.span(start, self.pos), flags);
                }
                _ => {}
            }
        }
        Token::new(TokenKind::Eq, self.span(start, self.pos), flags)
    }

    fn scan_bang(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance();
        if !self.at_end() && self.current() == b'=' {
            self.advance();
            if self.eat(b'=') {
                return Token::new(TokenKind::NotEqEq, self.span(start, self.pos), flags);
            }
            return Token::new(TokenKind::NotEq, self.span(start, self.pos), flags);
        }
        Token::new(TokenKind::Bang, self.span(start, self.pos), flags)
    }

    fn scan_amp(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance();
        if !self.at_end() {
            match self.current() {
                b'&' => {
                    self.advance();
                    if self.eat(b'=') {
                        return Token::new(TokenKind::AmpAmpEq, self.span(start, self.pos), flags);
                    }
                    return Token::new(TokenKind::AmpAmp, self.span(start, self.pos), flags);
                }
                b'=' => {
                    self.advance();
                    return Token::new(TokenKind::AmpEq, self.span(start, self.pos), flags);
                }
                _ => {}
            }
        }
        Token::new(TokenKind::Amp, self.span(start, self.pos), flags)
    }

    fn scan_pipe(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance();
        if !self.at_end() {
            match self.current() {
                b'|' => {
                    self.advance();
                    if self.eat(b'=') {
                        return Token::new(
                            TokenKind::PipePipeEq,
                            self.span(start, self.pos),
                            flags,
                        );
                    }
                    return Token::new(TokenKind::PipePipe, self.span(start, self.pos), flags);
                }
                b'=' => {
                    self.advance();
                    return Token::new(TokenKind::PipeEq, self.span(start, self.pos), flags);
                }
                _ => {}
            }
        }
        Token::new(TokenKind::Pipe, self.span(start, self.pos), flags)
    }

    fn scan_caret(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance();
        if self.eat(b'=') {
            return Token::new(TokenKind::CaretEq, self.span(start, self.pos), flags);
        }
        Token::new(TokenKind::Caret, self.span(start, self.pos), flags)
    }

    fn scan_hash(&mut self, start: usize, flags: TokenFlags) -> Token {
        self.advance(); // skip `#`
                        // If followed by an identifier start, it's a private identifier
        if !self.at_end() {
            let ch = self.current();
            if ch == b'_' || ch == b'$' || ch.is_ascii_alphabetic() || ch == b'\\' || ch >= 0x80 {
                // Scan the identifier part
                let ident_start = self.pos;
                let inner = self.scan_identifier(ident_start, flags);
                let payload = match inner.payload {
                    TokenPayload::Atom(_) => inner.payload,
                    _ => {
                        let text = self.text(ident_start, self.pos).to_owned();
                        TokenPayload::Atom(self.atoms.intern(&text))
                    }
                };
                return Token::with_payload(
                    TokenKind::PrivateIdentifier,
                    self.span(start, self.pos),
                    inner.flags,
                    payload,
                );
            }
        }
        Token::new(TokenKind::Hash, self.span(start, self.pos), flags)
    }
}

// =============================================================================
// Template scanning helper enum
// =============================================================================

enum TemplateEnd {
    Backtick,
    DollarBrace,
    Eof,
}

// =============================================================================
// Numeric parsing helpers (module-level)
// =============================================================================

fn parse_hex_to_f64(hex: &str) -> f64 {
    // For exact integers up to 2^53
    u64::from_str_radix(hex, 16).map_or_else(
        |_| {
            // Fallback for very large hex numbers
            let mut val = 0.0_f64;
            for b in hex.bytes() {
                let digit = match b {
                    b'0'..=b'9' => f64::from(b - b'0'),
                    b'a'..=b'f' => f64::from(b - b'a' + 10),
                    b'A'..=b'F' => f64::from(b - b'A' + 10),
                    _ => continue,
                };
                val = val.mul_add(16.0, digit);
            }
            val
        },
        |v| v as f64,
    )
}

fn parse_octal_to_f64(oct: &str) -> f64 {
    u64::from_str_radix(oct, 8).map_or_else(
        |_| {
            let mut val = 0.0_f64;
            for b in oct.bytes() {
                if b.is_ascii_digit() {
                    val = val.mul_add(8.0, f64::from(b - b'0'));
                }
            }
            val
        },
        |v| v as f64,
    )
}

fn parse_binary_to_f64(bin: &str) -> f64 {
    u64::from_str_radix(bin, 2).map_or_else(
        |_| {
            let mut val = 0.0_f64;
            for b in bin.bytes() {
                if b == b'0' || b == b'1' {
                    val = val.mul_add(2.0, f64::from(b - b'0'));
                }
            }
            val
        },
        |v| v as f64,
    )
}

fn is_legacy_octal_like_decimal(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() > 1
        && bytes[0] == b'0'
        && !text.contains('_')
        && bytes[1..].iter().all(u8::is_ascii_digit)
}

fn is_legacy_octal_integer(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() > 1
        && bytes[0] == b'0'
        && !text.contains('_')
        && bytes[1..].iter().all(|byte| matches!(byte, b'0'..=b'7'))
}

fn has_invalid_legacy_decimal_separator(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() > 1 && bytes[0] == b'0' && text.contains('_')
}

fn has_invalid_bigint_decimal_leading_zero(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() > 1 && bytes[0] == b'0'
}

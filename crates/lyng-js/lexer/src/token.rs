//! Token types for the ECMA-262 lexical grammar.

use bitflags::bitflags;
use lyng_js_common::{AtomId, Span};

/// A compact, copyable identifier for literal side-table entries.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct LiteralId(u32);

impl LiteralId {
    #[inline]
    pub(crate) const fn new(raw: u32) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// The payload carried by a token. Kept small (8 bytes) for compactness.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TokenPayload {
    /// No payload (keywords, punctuators, EOF).
    None,
    /// An interned identifier or keyword atom.
    Atom(AtomId),
    /// An index into a lexer-local literal side table.
    Literal(LiteralId),
    /// A numeric value that fits in an f64. Stored inline to avoid side-table
    /// lookup for the common case.
    Number(u64),
}

bitflags! {
    /// Flags on a token relevant to parsing (ASI, cover grammars, etc.).
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
    pub struct TokenFlags: u8 {
        /// A line terminator appeared in the whitespace/comments preceding
        /// this token. Used for automatic semicolon insertion.
        const PRECEDED_BY_LINE_TERMINATOR = 1 << 0;
        /// The token text contained a Unicode escape sequence (`\uXXXX` or
        /// `\u{XXXX}`). An identifier that spells a keyword but contains
        /// escapes must be treated as an identifier, not a keyword.
        const CONTAINS_ESCAPE = 1 << 1;
        /// The numeric literal used a legacy octal-like decimal form such as
        /// `010` or `08`.
        const LEGACY_OCTAL_LIKE_DECIMAL = 1 << 2;
        /// The string literal contained a legacy octal escape such as `\1`
        /// or `\07`.
        const LEGACY_OCTAL_ESCAPE = 1 << 3;
        /// The string literal contained a non-octal decimal escape such as
        /// `\8` or `\9`.
        const NON_OCTAL_DECIMAL_ESCAPE = 1 << 4;
    }
}

/// A single token produced by the lexer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub flags: TokenFlags,
    pub payload: TokenPayload,
}

impl Token {
    /// Creates a simple token with no payload.
    #[inline]
    pub const fn new(kind: TokenKind, span: Span, flags: TokenFlags) -> Self {
        Self {
            kind,
            span,
            flags,
            payload: TokenPayload::None,
        }
    }

    /// Creates a token with a payload.
    #[inline]
    pub const fn with_payload(
        kind: TokenKind,
        span: Span,
        flags: TokenFlags,
        payload: TokenPayload,
    ) -> Self {
        Self {
            kind,
            span,
            flags,
            payload,
        }
    }

    #[inline]
    pub const fn preceded_by_line_terminator(self) -> bool {
        self.flags.contains(TokenFlags::PRECEDED_BY_LINE_TERMINATOR)
    }

    #[inline]
    pub const fn contains_escape(self) -> bool {
        self.flags.contains(TokenFlags::CONTAINS_ESCAPE)
    }

    #[inline]
    pub const fn has_legacy_octal_like_decimal(self) -> bool {
        self.flags.contains(TokenFlags::LEGACY_OCTAL_LIKE_DECIMAL)
    }

    #[inline]
    pub const fn has_legacy_octal_escape(self) -> bool {
        self.flags.contains(TokenFlags::LEGACY_OCTAL_ESCAPE)
    }

    #[inline]
    pub const fn has_non_octal_decimal_escape(self) -> bool {
        self.flags.contains(TokenFlags::NON_OCTAL_DECIMAL_ESCAPE)
    }
}

/// All token kinds in the ECMA-262 lexical grammar.
///
/// Keywords are individual variants (not identifiers) so the parser can
/// pattern-match directly. Contextual keywords (`async`, `let`, `of`, etc.)
/// are lexed as `Identifier` and resolved by the parser.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
#[allow(clippy::manual_non_exhaustive)]
pub enum TokenKind {
    // ===== End of file =====
    Eof,

    // ===== Identifiers =====
    /// An identifier name (includes contextual keywords like `async`, `let`).
    Identifier,
    /// A private identifier (`#name`).
    PrivateIdentifier,

    // ===== Literals =====
    /// A numeric literal (decimal, hex, octal, binary). Value in payload.
    NumericLiteral,
    /// A BigInt literal (`123n`). Raw digits in literal side table.
    BigIntLiteral,
    /// A string literal (single or double quoted). Cooked value in side table.
    StringLiteral,
    /// A regular expression literal (`/pattern/flags`).
    RegExpLiteral,

    // ===== Template literals =====
    /// `` `head${ `` — the opening portion of a template with substitutions.
    TemplateHead,
    /// `` }middle${ `` — a middle chunk between substitutions.
    TemplateMiddle,
    /// `` }tail` `` — the closing portion after the last substitution.
    TemplateTail,
    /// `` `noSub` `` — a template with no substitutions.
    NoSubstitutionTemplate,

    // ===== Keywords (ECMA-262 12.6.2) =====
    // These correspond to WellKnownAtom await..=yield_ (1..=38).
    Await,
    Break,
    Case,
    Catch,
    Class,
    Const,
    Continue,
    Debugger,
    Default,
    Delete,
    Do,
    Else,
    Enum,
    Export,
    Extends,
    False,
    Finally,
    For,
    Function,
    If,
    Import,
    In,
    Instanceof,
    New,
    Null,
    Return,
    Super,
    Switch,
    This,
    Throw,
    True,
    Try,
    Typeof,
    Var,
    Void,
    While,
    With,
    Yield,

    // ===== Punctuators =====
    // Grouping
    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]

    // Delimiters
    Semicolon,     // ;
    Comma,         // ,
    Colon,         // :
    Dot,           // .
    Ellipsis,      // ...
    Arrow,         // =>
    Question,      // ?
    OptionalChain, // ?.
    At,            // @ (decorators)
    Hash,          // # (private, but distinct from PrivateIdentifier)

    // Arithmetic
    Plus,    // +
    Minus,   // -
    Star,    // *
    Slash,   // /
    Percent, // %
    Exp,     // **

    // Increment / Decrement
    PlusPlus,   // ++
    MinusMinus, // --

    // Comparison
    Lt,      // <
    Gt,      // >
    LtEq,    // <=
    GtEq,    // >=
    EqEq,    // ==
    NotEq,   // !=
    EqEqEq,  // ===
    NotEqEq, // !==

    // Bitwise
    Amp,    // &
    Pipe,   // |
    Caret,  // ^
    Tilde,  // ~
    LtLt,   // <<
    GtGt,   // >>
    GtGtGt, // >>>

    // Logical
    AmpAmp,           // &&
    PipePipe,         // ||
    Bang,             // !
    QuestionQuestion, // ??

    // Assignment
    Eq,                 // =
    PlusEq,             // +=
    MinusEq,            // -=
    StarEq,             // *=
    SlashEq,            // /=
    PercentEq,          // %=
    ExpEq,              // **=
    AmpEq,              // &=
    PipeEq,             // |=
    CaretEq,            // ^=
    LtLtEq,             // <<=
    GtGtEq,             // >>=
    GtGtGtEq,           // >>>=
    AmpAmpEq,           // &&=
    PipePipeEq,         // ||=
    QuestionQuestionEq, // ??=
}

impl TokenKind {
    /// Returns `true` if this token kind is a keyword.
    #[inline]
    pub const fn is_keyword(self) -> bool {
        matches!(
            self,
            Self::Await
                | Self::Break
                | Self::Case
                | Self::Catch
                | Self::Class
                | Self::Const
                | Self::Continue
                | Self::Debugger
                | Self::Default
                | Self::Delete
                | Self::Do
                | Self::Else
                | Self::Enum
                | Self::Export
                | Self::Extends
                | Self::False
                | Self::Finally
                | Self::For
                | Self::Function
                | Self::If
                | Self::Import
                | Self::In
                | Self::Instanceof
                | Self::New
                | Self::Null
                | Self::Return
                | Self::Super
                | Self::Switch
                | Self::This
                | Self::Throw
                | Self::True
                | Self::Try
                | Self::Typeof
                | Self::Var
                | Self::Void
                | Self::While
                | Self::With
                | Self::Yield
        )
    }

    /// Returns `true` if this is an assignment operator.
    #[inline]
    pub const fn is_assignment(self) -> bool {
        matches!(
            self,
            Self::Eq
                | Self::PlusEq
                | Self::MinusEq
                | Self::StarEq
                | Self::SlashEq
                | Self::PercentEq
                | Self::ExpEq
                | Self::AmpEq
                | Self::PipeEq
                | Self::CaretEq
                | Self::LtLtEq
                | Self::GtGtEq
                | Self::GtGtGtEq
                | Self::AmpAmpEq
                | Self::PipePipeEq
                | Self::QuestionQuestionEq
        )
    }
}

/// Map from `WellKnownAtom` keyword discriminant (1..=38) to `TokenKind`.
///
/// Index 0 is unused (Empty atom). Indices 1..=38 correspond to keywords.
pub const KEYWORD_TOKEN_KIND: [TokenKind; 39] = [
    TokenKind::Eof, // 0: placeholder for Empty
    TokenKind::Await,
    TokenKind::Break,
    TokenKind::Case,
    TokenKind::Catch,
    TokenKind::Class,
    TokenKind::Const,
    TokenKind::Continue,
    TokenKind::Debugger,
    TokenKind::Default,
    TokenKind::Delete,
    TokenKind::Do,
    TokenKind::Else,
    TokenKind::Enum,
    TokenKind::Export,
    TokenKind::Extends,
    TokenKind::False,
    TokenKind::Finally,
    TokenKind::For,
    TokenKind::Function,
    TokenKind::If,
    TokenKind::Import,
    TokenKind::In,
    TokenKind::Instanceof,
    TokenKind::New,
    TokenKind::Null,
    TokenKind::Return,
    TokenKind::Super,
    TokenKind::Switch,
    TokenKind::This,
    TokenKind::Throw,
    TokenKind::True,
    TokenKind::Try,
    TokenKind::Typeof,
    TokenKind::Var,
    TokenKind::Void,
    TokenKind::While,
    TokenKind::With,
    TokenKind::Yield,
];

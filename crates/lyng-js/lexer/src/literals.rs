//! Lexer-local side tables for non-trivial literal payloads.
//!
//! Common tokens (identifiers, keywords, punctuators, small numbers) carry
//! their data inline in `TokenPayload`. Rarer or variable-size payloads
//! (strings, BigInts, regexps, template chunks) are stored here and
//! referenced by `LiteralId`.

use crate::token::LiteralId;

/// Cooked string value for a string literal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StringLiteral {
    Utf8(String),
    Utf16(Box<[u16]>),
}

impl StringLiteral {
    #[inline]
    pub fn from_utf8(value: String) -> Self {
        Self::Utf8(value)
    }

    pub fn from_utf16(units: Vec<u16>) -> Self {
        match String::from_utf16(&units) {
            Ok(text) => Self::Utf8(text),
            Err(_) => Self::Utf16(units.into_boxed_slice()),
        }
    }

    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Utf8(value) => Some(value),
            Self::Utf16(_) => None,
        }
    }

    pub fn code_units(&self) -> Vec<u16> {
        match self {
            Self::Utf8(value) => value.encode_utf16().collect(),
            Self::Utf16(units) => units.to_vec(),
        }
    }

    #[inline]
    pub fn to_string_lossy(&self) -> String {
        match self {
            Self::Utf8(value) => value.clone(),
            Self::Utf16(units) => String::from_utf16_lossy(units),
        }
    }

    #[inline]
    pub fn equals_text(&self, expected: &str) -> bool {
        match self {
            Self::Utf8(value) => value == expected,
            Self::Utf16(units) => expected.encode_utf16().eq(units.iter().copied()),
        }
    }
}

/// Raw digit string for a BigInt literal (without the trailing `n`).
#[derive(Clone, Debug, PartialEq)]
pub struct BigIntLiteral {
    /// The raw digits, e.g. `"123"` for `123n`, `"0xff"` for `0xffn`.
    pub raw: String,
}

/// A regular expression literal's components.
#[derive(Clone, Debug, PartialEq)]
pub struct RegExpLiteral {
    pub pattern: String,
    pub flags: String,
}

/// A template literal chunk (head, middle, tail, or no-substitution).
#[derive(Clone, Debug, PartialEq)]
pub struct TemplateChunk {
    /// The cooked value (with escape sequences processed), or `None` if
    /// the chunk contains an invalid escape (tagged template).
    pub cooked: Option<String>,
    /// The raw source text between the delimiters.
    pub raw: String,
}

/// Literal side tables owned by the lexer.
///
/// Each `push_*` method appends an entry and returns a `LiteralId` that
/// indexes into the appropriate table. The ID encodes which table it
/// belongs to via a tag in the upper bits, but consumers should use the
/// typed accessor methods rather than decoding manually.
#[derive(Default, Debug)]
pub struct LiteralTable {
    strings: Vec<StringLiteral>,
    bigints: Vec<BigIntLiteral>,
    regexps: Vec<RegExpLiteral>,
    templates: Vec<TemplateChunk>,
}

// Tag bits in the upper 2 bits of LiteralId.
const TAG_STRING: u32 = 0 << 30;
const TAG_BIGINT: u32 = 1 << 30;
const TAG_REGEXP: u32 = 2 << 30;
const TAG_TEMPLATE: u32 = 3 << 30;
const TAG_MASK: u32 = 0b11 << 30;
const INDEX_MASK: u32 = !TAG_MASK;

impl LiteralTable {
    pub fn new() -> Self {
        Self::default()
    }

    // --- Push ---

    pub fn push_string(&mut self, value: StringLiteral) -> LiteralId {
        let idx = self.strings.len() as u32;
        self.strings.push(value);
        LiteralId::new(TAG_STRING | idx)
    }

    pub fn push_bigint(&mut self, raw: String) -> LiteralId {
        let idx = self.bigints.len() as u32;
        self.bigints.push(BigIntLiteral { raw });
        LiteralId::new(TAG_BIGINT | idx)
    }

    pub fn push_regexp(&mut self, pattern: String, flags: String) -> LiteralId {
        let idx = self.regexps.len() as u32;
        self.regexps.push(RegExpLiteral { pattern, flags });
        LiteralId::new(TAG_REGEXP | idx)
    }

    pub fn push_template(&mut self, cooked: Option<String>, raw: String) -> LiteralId {
        let idx = self.templates.len() as u32;
        self.templates.push(TemplateChunk { cooked, raw });
        LiteralId::new(TAG_TEMPLATE | idx)
    }

    // --- Accessors ---

    pub fn get_string(&self, id: LiteralId) -> &StringLiteral {
        debug_assert_eq!(id.raw() & TAG_MASK, TAG_STRING);
        &self.strings[(id.raw() & INDEX_MASK) as usize]
    }

    pub fn get_bigint(&self, id: LiteralId) -> &BigIntLiteral {
        debug_assert_eq!(id.raw() & TAG_MASK, TAG_BIGINT);
        &self.bigints[(id.raw() & INDEX_MASK) as usize]
    }

    pub fn get_regexp(&self, id: LiteralId) -> &RegExpLiteral {
        debug_assert_eq!(id.raw() & TAG_MASK, TAG_REGEXP);
        &self.regexps[(id.raw() & INDEX_MASK) as usize]
    }

    pub fn get_template(&self, id: LiteralId) -> &TemplateChunk {
        debug_assert_eq!(id.raw() & TAG_MASK, TAG_TEMPLATE);
        &self.templates[(id.raw() & INDEX_MASK) as usize]
    }
}

//! Literal value storage for the AST.
//!
//! Numeric values are stored inline in expression nodes. String, `BigInt`, and
//! `RegExp` payloads live in separate tables and are referenced by small IDs.

use crate::ids::{BigIntLiteralId, RegExpLiteralId, StringLiteralId};

/// A numeric literal value, either a 32-bit integer or an IEEE 754 double.
///
/// The parser chooses `Int32` for values that fit in `i32` with no fractional
/// part, allowing downstream consumers to avoid float conversion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumericLiteral {
    /// An integer value that fits in a 32-bit signed integer.
    Int32(i32),
    /// An IEEE 754 double-precision floating-point value.
    Number(f64),
}

/// Syntax metadata preserved for numeric literals that need strict-mode early
/// errors later in semantic analysis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NumericLiteralSyntax {
    /// A standard numeric literal with no legacy strict-mode restriction.
    #[default]
    Normal,
    /// A legacy octal-like decimal form such as `010` or `08`.
    LegacyOctalLikeDecimal,
}

/// Syntax metadata preserved for string literals that need strict-mode early
/// errors later in semantic analysis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct StringLiteralSyntax {
    pub contains_escape: bool,
    pub contains_legacy_octal_escape: bool,
    pub contains_non_octal_decimal_escape: bool,
}

impl StringLiteralSyntax {
    #[inline]
    pub const fn has_strict_mode_escape(self) -> bool {
        self.contains_legacy_octal_escape || self.contains_non_octal_decimal_escape
    }
}

/// A stored JS string literal value, preserving UTF-16-only literals when the
/// cooked value contains lone surrogate code units.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StringLiteralValue {
    Utf8(Box<str>),
    Utf16(Box<[u16]>),
}

impl StringLiteralValue {
    #[inline]
    pub fn from_text(value: &str) -> Self {
        Self::Utf8(value.into())
    }

    pub fn from_code_units(units: &[u16]) -> Self {
        String::from_utf16(units).map_or_else(
            |_| Self::Utf16(units.into()),
            |text| Self::Utf8(text.into_boxed_str()),
        )
    }

    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Utf8(value) => Some(value),
            Self::Utf16(_) => None,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Utf8(value) => value.is_empty(),
            Self::Utf16(units) => units.is_empty(),
        }
    }

    pub fn code_units(&self) -> Vec<u16> {
        match self {
            Self::Utf8(value) => value.encode_utf16().collect(),
            Self::Utf16(units) => units.to_vec(),
        }
    }

    #[inline]
    pub fn equals_text(&self, expected: &str) -> bool {
        match self {
            Self::Utf8(value) => value.as_ref() == expected,
            Self::Utf16(units) => expected.encode_utf16().eq(units.iter().copied()),
        }
    }
}

/// A stored `RegExp` literal: pattern + flags.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegExpValue {
    /// The pattern text (between the `/` delimiters).
    pub pattern: Box<str>,
    /// The flags string (e.g. `"gi"`).
    pub flags: Box<str>,
}

/// Tables for non-numeric literal payloads owned by the AST.
///
/// String, `BigInt`, and `RegExp` values each get their own arena-style storage,
/// indexed by small typed IDs.
pub struct LiteralTable {
    strings: Vec<StringLiteralValue>,
    bigints: Vec<Box<str>>,
    regexps: Vec<RegExpValue>,
}

impl Default for LiteralTable {
    fn default() -> Self {
        Self::new()
    }
}

impl LiteralTable {
    /// Creates an empty literal table.
    pub const fn new() -> Self {
        Self {
            strings: Vec::new(),
            bigints: Vec::new(),
            regexps: Vec::new(),
        }
    }

    // -- String literals ---------------------------------------------------

    /// Stores a string literal value and returns its ID.
    pub fn alloc_string(&mut self, value: &str) -> StringLiteralId {
        let id = StringLiteralId::new(self.strings.len() as u32);
        self.strings.push(StringLiteralValue::from_text(value));
        id
    }

    /// Stores a UTF-16 string literal value and returns its ID.
    pub fn alloc_utf16_string(&mut self, units: &[u16]) -> StringLiteralId {
        let id = StringLiteralId::new(self.strings.len() as u32);
        self.strings
            .push(StringLiteralValue::from_code_units(units));
        id
    }

    /// Returns the stored string value for a given ID.
    #[inline]
    pub fn get_string_value(&self, id: StringLiteralId) -> &StringLiteralValue {
        &self.strings[id.raw() as usize]
    }

    /// Returns the string for a given ID.
    ///
    /// # Panics
    ///
    /// Panics if the literal requires UTF-16-only storage.
    #[inline]
    pub fn get_string(&self, id: StringLiteralId) -> &str {
        self.get_string_value(id)
            .as_str()
            .expect("string literal requires UTF-16-only resolution")
    }

    /// Returns whether the string literal matches a UTF-8 text value.
    #[inline]
    pub fn string_equals(&self, id: StringLiteralId, expected: &str) -> bool {
        self.get_string_value(id).equals_text(expected)
    }

    /// Returns the string literal's code units.
    #[inline]
    pub fn string_code_units(&self, id: StringLiteralId) -> Vec<u16> {
        self.get_string_value(id).code_units()
    }

    /// Returns whether the string literal is empty.
    #[inline]
    pub fn string_is_empty(&self, id: StringLiteralId) -> bool {
        self.get_string_value(id).is_empty()
    }

    // -- BigInt literals ---------------------------------------------------

    /// Stores a bigint literal text (the digit string, without `n` suffix) and
    /// returns its ID.
    pub fn alloc_bigint(&mut self, value: &str) -> BigIntLiteralId {
        let id = BigIntLiteralId::new(self.bigints.len() as u32);
        self.bigints.push(value.into());
        id
    }

    /// Returns the bigint text for a given ID.
    #[inline]
    pub fn get_bigint(&self, id: BigIntLiteralId) -> &str {
        &self.bigints[id.raw() as usize]
    }

    // -- RegExp literals ---------------------------------------------------

    /// Stores a regexp pattern + flags and returns its ID.
    pub fn alloc_regexp(&mut self, pattern: &str, flags: &str) -> RegExpLiteralId {
        let id = RegExpLiteralId::new(self.regexps.len() as u32);
        self.regexps.push(RegExpValue {
            pattern: pattern.into(),
            flags: flags.into(),
        });
        id
    }

    /// Returns the regexp value for a given ID.
    #[inline]
    pub fn get_regexp(&self, id: RegExpLiteralId) -> &RegExpValue {
        &self.regexps[id.raw() as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_literal_int32() {
        let v = NumericLiteral::Int32(42);
        assert_eq!(v, NumericLiteral::Int32(42));
    }

    #[test]
    fn numeric_literal_number() {
        let v = NumericLiteral::Number(2.5);
        assert_eq!(v, NumericLiteral::Number(2.5));
    }

    #[test]
    fn numeric_literal_syntax_default() {
        assert_eq!(
            NumericLiteralSyntax::default(),
            NumericLiteralSyntax::Normal
        );
    }

    #[test]
    fn string_literal_syntax_strict_escape() {
        let syntax = StringLiteralSyntax {
            contains_escape: false,
            contains_legacy_octal_escape: false,
            contains_non_octal_decimal_escape: true,
        };
        assert!(syntax.has_strict_mode_escape());
    }

    #[test]
    fn string_literal_table() {
        let mut table = LiteralTable::new();
        let id1 = table.alloc_string("hello");
        let id2 = table.alloc_string("world");
        assert_eq!(table.get_string(id1), "hello");
        assert_eq!(table.get_string(id2), "world");
    }

    #[test]
    fn string_literal_table_preserves_utf16_only_literals() {
        let mut table = LiteralTable::new();
        let id = table.alloc_utf16_string(&[0xD801]);
        assert_eq!(table.get_string_value(id).as_str(), None);
        assert_eq!(table.string_code_units(id), vec![0xD801]);
    }

    #[test]
    fn bigint_literal_table() {
        let mut table = LiteralTable::new();
        let id = table.alloc_bigint("123456789012345678901234567890");
        assert_eq!(table.get_bigint(id), "123456789012345678901234567890");
    }

    #[test]
    fn regexp_literal_table() {
        let mut table = LiteralTable::new();
        let id = table.alloc_regexp("foo.*bar", "gi");
        let val = table.get_regexp(id);
        assert_eq!(&*val.pattern, "foo.*bar");
        assert_eq!(&*val.flags, "gi");
    }
}

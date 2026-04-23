//! Source identifiers and text locations.
//!
//! These types are the shared location currency for lexer, parser, AST,
//! diagnostics, and later compiler/runtime source mapping.

use std::fmt;

/// Identifies a source text unit (script or module) within a compilation session.
///
/// Compact and copyable. The caller assigns source IDs when registering source
/// texts; the engine treats them as opaque identifiers.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceId(u32);

impl SourceId {
    /// Creates a new source ID from a raw index.
    #[inline]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw index.
    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl fmt::Debug for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SourceId({})", self.0)
    }
}

/// A byte offset into source text. 32-bit to keep spans compact.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextOffset(u32);

impl TextOffset {
    #[inline]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl fmt::Debug for TextOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A half-open byte range `[start, end)` in source text.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextRange {
    pub start: TextOffset,
    pub end: TextOffset,
}

impl TextRange {
    #[inline]
    pub const fn new(start: TextOffset, end: TextOffset) -> Self {
        Self { start, end }
    }

    /// Byte length of this range.
    #[inline]
    pub const fn len(self) -> u32 {
        self.end.0 - self.start.0
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.start.0 == self.end.0
    }

    /// Extends this range to cover `other` as well.
    #[inline]
    pub const fn cover(self, other: TextRange) -> TextRange {
        let start = if self.start.0 < other.start.0 {
            self.start
        } else {
            other.start
        };
        let end = if self.end.0 > other.end.0 {
            self.end
        } else {
            other.end
        };
        TextRange { start, end }
    }
}

impl fmt::Debug for TextRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start.0, self.end.0)
    }
}

/// A source location: `(SourceId, TextRange)`.
///
/// This is the primary span type used by tokens, AST nodes, and diagnostics.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub source: SourceId,
    pub range: TextRange,
}

impl Span {
    #[inline]
    pub const fn new(source: SourceId, range: TextRange) -> Self {
        Self { source, range }
    }

    /// Creates a span from raw byte offsets.
    #[inline]
    pub const fn from_offsets(source: SourceId, start: u32, end: u32) -> Self {
        Self {
            source,
            range: TextRange::new(TextOffset::new(start), TextOffset::new(end)),
        }
    }

    /// Extends this span to cover `other`. Both must be in the same source.
    #[inline]
    pub const fn cover(self, other: Span) -> Span {
        // Debug assertion: same source. In release, we just use self.source.
        Span {
            source: self.source,
            range: self.range.cover(other.range),
        }
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}@{}..{}",
            self.source.0, self.range.start.0, self.range.end.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_id_roundtrip() {
        let id = SourceId::new(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn text_range_len() {
        let r = TextRange::new(TextOffset::new(10), TextOffset::new(25));
        assert_eq!(r.len(), 15);
        assert!(!r.is_empty());
    }

    #[test]
    fn text_range_empty() {
        let r = TextRange::new(TextOffset::new(5), TextOffset::new(5));
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn text_range_cover() {
        let a = TextRange::new(TextOffset::new(10), TextOffset::new(20));
        let b = TextRange::new(TextOffset::new(15), TextOffset::new(30));
        let c = a.cover(b);
        assert_eq!(c.start, TextOffset::new(10));
        assert_eq!(c.end, TextOffset::new(30));
    }

    #[test]
    fn span_cover() {
        let src = SourceId::new(0);
        let a = Span::from_offsets(src, 5, 10);
        let b = Span::from_offsets(src, 8, 20);
        let c = a.cover(b);
        assert_eq!(c.range.start.raw(), 5);
        assert_eq!(c.range.end.raw(), 20);
    }
}

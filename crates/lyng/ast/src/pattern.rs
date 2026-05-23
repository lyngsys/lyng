//! Pattern (destructuring) AST nodes (ECMA-262 §13.15.5).
//!
//! Patterns are their own node family, separate from expressions.
//! This is a frozen design decision — patterns do not reuse expression nodes.

use lyng_js_common::{AtomId, Span};

use crate::ids::{ExprId, NodeList, PatternId};

/// A destructuring or binding pattern.
#[derive(Clone, Debug)]
pub enum Pattern {
    /// A simple binding identifier: `x`
    Identifier { span: Span, name: AtomId },

    /// An object destructuring pattern: `{ a, b: c, ...rest }`
    Object {
        span: Span,
        properties: NodeList<ObjectPatternProperty>,
        rest: Option<PatternId>,
    },

    /// An array destructuring pattern: `[a, , b, ...rest]`
    Array {
        span: Span,
        elements: NodeList<Option<ArrayPatternElement>>,
        rest: Option<PatternId>,
    },

    /// An assignment pattern (default value): `x = defaultValue`
    Assignment {
        span: Span,
        left: PatternId,
        right: ExprId,
    },

    // -- Error recovery ----------------------------------------------------
    /// A placeholder for patterns that failed to parse.
    InvalidPattern { span: Span },
}

/// A single property in an object destructuring pattern.
#[derive(Clone, Copy, Debug)]
pub struct ObjectPatternProperty {
    pub span: Span,
    /// The key expression (identifier, string literal, computed).
    pub key: ExprId,
    /// The value pattern to bind to.
    pub value: PatternId,
    /// True for computed keys: `{ [expr]: pat }`.
    pub computed: bool,
    /// True for shorthand: `{ x }` means `{ x: x }`.
    pub shorthand: bool,
}

/// A single element in an array destructuring pattern.
///
/// Array elements can be `None` (elision), a pattern, or a rest element.
#[derive(Clone, Copy, Debug)]
pub struct ArrayPatternElement {
    pub span: Span,
    /// The pattern for this element.
    pub pattern: PatternId,
}

impl Pattern {
    /// Returns the span of this pattern.
    pub const fn span(&self) -> Span {
        match self {
            Self::Identifier { span, .. }
            | Self::Object { span, .. }
            | Self::Array { span, .. }
            | Self::Assignment { span, .. }
            | Self::InvalidPattern { span, .. } => *span,
        }
    }
}

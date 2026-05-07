//! Template literal AST support.
//!
//! Template literals have first-class representation: a `TemplateLiteral`
//! stores its quasis (cooked + raw string pairs) and interleaved expression
//! IDs in arena-backed list storage, consistent with the rest of the AST.

use crate::arena::ListArena;
use crate::ids::{ExprId, NodeList, StringLiteralId, TemplateLiteralId};

/// A single quasi element (the static text between `${...}` expressions).
///
/// `cooked` is `None` when the quasi contains an invalid escape sequence
/// (which is allowed in tagged templates).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemplateQuasi {
    /// The cooked value (escape sequences interpreted), or `None` for
    /// invalid escape sequences in tagged templates.
    pub cooked: Option<StringLiteralId>,
    /// The raw source text (escape sequences preserved).
    pub raw: StringLiteralId,
}

/// The payload of a template literal, stored in the template arena.
///
/// Invariant: `quasis` length == `expressions` length + 1.
///
/// For a simple template with no expressions (`` `hello` ``), there is one
/// quasi and zero expressions.
pub struct TemplateLiteralData {
    /// The static text segments (arena-backed range).
    pub quasis: NodeList<TemplateQuasi>,
    /// The dynamic expression IDs interleaved between quasis (arena-backed range).
    pub expressions: NodeList<ExprId>,
}

/// Arena for template literal data, indexed by `TemplateLiteralId`.
///
/// Owns internal `ListArena`s for quasis and expressions so that
/// `TemplateLiteralData` stores compact `NodeList` ranges instead of
/// per-node heap `Vec`s.
pub struct TemplateArena {
    templates: Vec<TemplateLiteralData>,
    quasi_lists: ListArena<TemplateQuasi>,
    expr_lists: ListArena<ExprId>,
}

impl Default for TemplateArena {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateArena {
    /// Creates an empty template arena.
    pub const fn new() -> Self {
        Self {
            templates: Vec::new(),
            quasi_lists: ListArena::new(),
            expr_lists: ListArena::new(),
        }
    }

    /// Allocates a template literal and returns its ID.
    pub fn alloc(&mut self, quasis: &[TemplateQuasi], expressions: &[ExprId]) -> TemplateLiteralId {
        debug_assert_eq!(quasis.len(), expressions.len() + 1);
        let id = TemplateLiteralId::new(self.templates.len() as u32);
        self.templates.push(TemplateLiteralData {
            quasis: self.quasi_lists.alloc(quasis),
            expressions: self.expr_lists.alloc(expressions),
        });
        id
    }

    /// Returns the template literal data for a given ID.
    #[inline]
    pub fn get(&self, id: TemplateLiteralId) -> &TemplateLiteralData {
        &self.templates[id.raw() as usize]
    }

    /// Returns the quasis slice for a template literal.
    #[inline]
    pub fn get_quasis(&self, id: TemplateLiteralId) -> &[TemplateQuasi] {
        self.quasi_lists
            .get(self.templates[id.raw() as usize].quasis)
    }

    /// Returns the expression IDs for a template literal.
    #[inline]
    pub fn get_expressions(&self, id: TemplateLiteralId) -> &[ExprId] {
        self.expr_lists
            .get(self.templates[id.raw() as usize].expressions)
    }

    /// Returns the number of stored templates.
    #[inline]
    pub const fn len(&self) -> usize {
        self.templates.len()
    }

    /// Returns `true` if no templates are stored.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.templates.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_template() {
        let mut arena = TemplateArena::new();
        let quasi = TemplateQuasi {
            cooked: Some(StringLiteralId::new(0)),
            raw: StringLiteralId::new(0),
        };
        let id = arena.alloc(&[quasi], &[]);
        assert_eq!(arena.get_quasis(id).len(), 1);
        assert_eq!(arena.get_expressions(id).len(), 0);
    }

    #[test]
    fn template_with_expressions() {
        let mut arena = TemplateArena::new();
        let q0 = TemplateQuasi {
            cooked: Some(StringLiteralId::new(0)),
            raw: StringLiteralId::new(0),
        };
        let q1 = TemplateQuasi {
            cooked: Some(StringLiteralId::new(1)),
            raw: StringLiteralId::new(1),
        };
        let q2 = TemplateQuasi {
            cooked: Some(StringLiteralId::new(2)),
            raw: StringLiteralId::new(2),
        };
        let id = arena.alloc(&[q0, q1, q2], &[ExprId::new(0), ExprId::new(1)]);
        assert_eq!(arena.get_quasis(id).len(), 3);
        assert_eq!(arena.get_expressions(id).len(), 2);
    }

    #[test]
    fn tagged_template_with_invalid_escape() {
        let mut arena = TemplateArena::new();
        let quasi = TemplateQuasi {
            cooked: None, // invalid escape sequence
            raw: StringLiteralId::new(0),
        };
        let id = arena.alloc(&[quasi], &[]);
        assert!(arena.get_quasis(id)[0].cooked.is_none());
    }
}

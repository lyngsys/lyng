//! Typed node IDs and list ranges for the AST.
//!
//! Every AST node family has a distinct 32-bit ID type. IDs are indices
//! into the corresponding `NodeArena`. They are `Copy` and cheap to pass
//! around by value.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

// ---------------------------------------------------------------------------
// Macro for defining node-ID newtypes
// ---------------------------------------------------------------------------

macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(u32);

        impl $name {
            /// Creates an ID from a raw arena index.
            #[inline]
            pub const fn new(raw: u32) -> Self {
                Self(raw)
            }

            /// Returns the raw arena index.
            #[inline]
            pub const fn raw(self) -> u32 {
                self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Node-ID families
// ---------------------------------------------------------------------------

define_id!(
    /// Identifies a `Script` root node in the AST.
    ScriptId
);

define_id!(
    /// Identifies a `Module` root node in the AST.
    ModuleId
);

define_id!(
    /// Identifies a `Stmt` (statement) node in the AST.
    StmtId
);

define_id!(
    /// Identifies an `Expr` (expression) node in the AST.
    ExprId
);

define_id!(
    /// Identifies a `Decl` (declaration) node in the AST.
    DeclId
);

define_id!(
    /// Identifies a `Pattern` node in the AST.
    PatternId
);

define_id!(
    /// Identifies a `Function` node in the AST.
    FunctionId
);

define_id!(
    /// Identifies a `ClassElement` node in the AST.
    ClassElementId
);

define_id!(
    /// Identifies a `TemplateLiteral` node in the AST.
    TemplateLiteralId
);

// ---------------------------------------------------------------------------
// Literal-table IDs
// ---------------------------------------------------------------------------

define_id!(
    /// Index into the string literal table.
    StringLiteralId
);

define_id!(
    /// Index into the bigint literal table.
    BigIntLiteralId
);

define_id!(
    /// Index into the regexp literal table.
    RegExpLiteralId
);

// ---------------------------------------------------------------------------
// NodeList — a (start, len) range into a side arena
// ---------------------------------------------------------------------------

/// A contiguous range of IDs stored in a `ListArena`.
///
/// `T` is the element type (e.g. `StmtId`, `ExprId`). The range is
/// represented as `(start, len)` into the list arena's backing storage.
pub struct NodeList<T> {
    pub start: u32,
    pub len: u32,
    _marker: PhantomData<T>,
}

// Manual trait impls to avoid requiring T: Copy/Clone/etc.

impl<T> Clone for NodeList<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for NodeList<T> {}

impl<T> PartialEq for NodeList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.len == other.len
    }
}

impl<T> Eq for NodeList<T> {}

impl<T> Hash for NodeList<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.len.hash(state);
    }
}

impl<T> fmt::Debug for NodeList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeList({}..+{})", self.start, self.len)
    }
}

impl<T> NodeList<T> {
    /// Creates a new node list from a start index and length.
    #[inline]
    pub const fn new(start: u32, len: u32) -> Self {
        Self {
            start,
            len,
            _marker: PhantomData,
        }
    }

    /// An empty node list.
    pub const EMPTY: Self = Self::new(0, 0);

    /// Returns `true` if the list has no elements.
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_roundtrip() {
        let id = StmtId::new(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn id_equality() {
        assert_eq!(ExprId::new(1), ExprId::new(1));
        assert_ne!(ExprId::new(1), ExprId::new(2));
    }

    #[test]
    fn id_debug() {
        let id = FunctionId::new(7);
        assert_eq!(format!("{id:?}"), "FunctionId(7)");
    }

    #[test]
    fn node_list_empty() {
        let list: NodeList<StmtId> = NodeList::EMPTY;
        assert!(list.is_empty());
        assert_eq!(list.len, 0);
    }

    #[test]
    fn node_list_copy() {
        let list = NodeList::<ExprId>::new(5, 3);
        let copy = list;
        assert_eq!(list, copy);
        assert_eq!(list.start, 5);
        assert_eq!(list.len, 3);
    }
}

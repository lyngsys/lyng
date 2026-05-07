//! Arena storage for AST nodes and child lists.
//!
//! `NodeArena<Id, Node>` is a simple append-only vector that hands out typed
//! IDs. `ListArena<T>` stores variable-length child lists contiguously and
//! hands out `NodeList<T>` ranges.

use crate::ids::NodeList;
use std::marker::PhantomData;

/// An append-only arena that stores nodes of type `N` and hands out typed
/// IDs of type `Id`.
///
/// `Id` must be constructible from / convertible to `u32` via `new` / `raw`.
pub struct NodeArena<Id, N> {
    nodes: Vec<N>,
    _marker: PhantomData<Id>,
}

impl<Id, N> Default for NodeArena<Id, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Id, N> NodeArena<Id, N> {
    /// Creates an empty arena.
    pub const fn new() -> Self {
        Self {
            nodes: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Creates an arena with the given pre-allocated capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(cap),
            _marker: PhantomData,
        }
    }

    /// Returns the number of nodes in this arena.
    #[inline]
    pub const fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the arena is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// Trait to abstract over typed IDs that wrap a `u32`.
pub trait ArenaId: Copy {
    fn new(raw: u32) -> Self;
    fn raw(self) -> u32;
}

/// Implement `ArenaId` for all our ID newtypes.
macro_rules! impl_arena_id {
    ($($ty:ty),* $(,)?) => {
        $(
            impl ArenaId for $ty {
                #[inline]
                fn new(raw: u32) -> Self { <$ty>::new(raw) }
                #[inline]
                fn raw(self) -> u32 { <$ty>::raw(self) }
            }
        )*
    };
}

impl_arena_id!(
    crate::ids::ScriptId,
    crate::ids::ModuleId,
    crate::ids::StmtId,
    crate::ids::ExprId,
    crate::ids::DeclId,
    crate::ids::PatternId,
    crate::ids::FunctionId,
    crate::ids::ClassElementId,
    crate::ids::TemplateLiteralId,
    crate::ids::StringLiteralId,
    crate::ids::BigIntLiteralId,
    crate::ids::RegExpLiteralId,
);

impl<Id: ArenaId, N> NodeArena<Id, N> {
    /// Appends a node and returns its typed ID.
    pub fn alloc(&mut self, node: N) -> Id {
        let index = self.nodes.len();
        self.nodes.push(node);
        Id::new(index as u32)
    }

    /// Returns a reference to the node at the given ID.
    ///
    /// # Panics
    ///
    /// Panics if the ID is out of range.
    #[inline]
    pub fn get(&self, id: Id) -> &N {
        &self.nodes[id.raw() as usize]
    }

    /// Returns a mutable reference to the node at the given ID.
    ///
    /// # Panics
    ///
    /// Panics if the ID is out of range.
    #[inline]
    pub fn get_mut(&mut self, id: Id) -> &mut N {
        &mut self.nodes[id.raw() as usize]
    }
}

// ---------------------------------------------------------------------------
// ListArena — append-only storage for child-ID lists
// ---------------------------------------------------------------------------

/// Append-only storage for variable-length lists of `T` values.
///
/// Lists are stored contiguously. A `NodeList<T>` records the start index
/// and length into this backing store.
pub struct ListArena<T> {
    data: Vec<T>,
}

impl<T: Copy> Default for ListArena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy> ListArena<T> {
    /// Creates an empty list arena.
    pub const fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Creates a list arena with the given pre-allocated capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            data: Vec::with_capacity(cap),
        }
    }

    /// Appends a slice of items and returns a `NodeList<T>` referencing them.
    pub fn alloc(&mut self, items: &[T]) -> NodeList<T> {
        let start = self.data.len() as u32;
        self.data.extend_from_slice(items);
        NodeList::new(start, items.len() as u32)
    }

    /// Returns the items for a given `NodeList`.
    ///
    /// # Panics
    ///
    /// Panics if the range is out of bounds.
    #[inline]
    pub fn get(&self, list: NodeList<T>) -> &[T] {
        let start = list.start as usize;
        let end = start + list.len as usize;
        &self.data[start..end]
    }

    /// Returns the total number of items stored.
    #[inline]
    pub const fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if no items are stored.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{ExprId, StmtId};

    #[test]
    fn node_arena_alloc_and_get() {
        let mut arena: NodeArena<StmtId, u64> = NodeArena::new();
        let id0 = arena.alloc(100);
        let id1 = arena.alloc(200);
        assert_eq!(*arena.get(id0), 100);
        assert_eq!(*arena.get(id1), 200);
        assert_eq!(arena.len(), 2);
    }

    #[test]
    fn node_arena_get_mut() {
        let mut arena: NodeArena<ExprId, String> = NodeArena::new();
        let id = arena.alloc("hello".into());
        *arena.get_mut(id) = "world".into();
        assert_eq!(arena.get(id), "world");
    }

    #[test]
    fn list_arena_alloc_and_get() {
        let mut arena: ListArena<StmtId> = ListArena::new();
        let items = [StmtId::new(0), StmtId::new(1), StmtId::new(2)];
        let list = arena.alloc(&items);
        assert_eq!(list.len, 3);
        let got = arena.get(list);
        assert_eq!(got, &items);
    }

    #[test]
    fn list_arena_multiple_lists() {
        let mut arena: ListArena<ExprId> = ListArena::new();
        let a = arena.alloc(&[ExprId::new(10), ExprId::new(20)]);
        let b = arena.alloc(&[ExprId::new(30)]);
        let c = arena.alloc(&[]);
        assert_eq!(arena.get(a).len(), 2);
        assert_eq!(arena.get(b).len(), 1);
        assert_eq!(arena.get(c).len(), 0);
        assert!(c.is_empty());
    }

    #[test]
    fn list_arena_empty_list() {
        let mut arena: ListArena<StmtId> = ListArena::new();
        let list = arena.alloc(&[]);
        assert!(list.is_empty());
        assert_eq!(arena.get(list), &[]);
    }
}

use std::num::NonZeroU32;

use super::document::QuirksMode;
use super::element::{Attribute, Namespace};

/// An index into the arena, identifying a single node.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(NonZeroU32);

impl NodeId {
    pub fn index(self) -> usize {
        (self.0.get() - 1) as usize
    }

    fn from_index(index: usize) -> Self {
        let raw = u32::try_from(index + 1).expect("node arena exceeds u32::MAX entries");
        Self(NonZeroU32::new(raw).expect("node indices are 1-based"))
    }
}

/// The data payload for each node type.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeData {
    Document {
        quirks_mode: QuirksMode,
    },
    Element {
        tag_name: String,
        namespace: Namespace,
        attributes: Vec<Attribute>,
    },
    Text {
        content: String,
    },
    Comment {
        content: String,
    },
    Doctype {
        name: String,
        public_id: String,
        system_id: String,
    },
    ProcessingInstruction {
        target: String,
        data: String,
    },
}

/// A single node in the DOM tree. Tree structure is maintained via indices.
#[derive(Debug, Clone)]
pub struct Node {
    pub parent: Option<NodeId>,
    pub first_child: Option<NodeId>,
    pub last_child: Option<NodeId>,
    pub next_sibling: Option<NodeId>,
    pub prev_sibling: Option<NodeId>,
    pub data: NodeData,
}

/// Arena-based storage for all DOM nodes. "Pointers" are `NodeId` indices.
#[derive(Debug)]
pub struct Arena {
    nodes: Vec<Node>,
}

impl Arena {
    pub fn new() -> Self {
        Arena { nodes: Vec::new() }
    }

    /// Create a new node with the given data and return its id.
    pub fn create_node(&mut self, data: NodeData) -> NodeId {
        let id = NodeId::from_index(self.nodes.len());
        self.nodes.push(Node {
            parent: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,
            data,
        });
        id
    }

    /// Get a reference to a node by id.
    pub fn get(&self, id: NodeId) -> &Node {
        &self.nodes[id.index()]
    }

    /// Get a mutable reference to a node by id.
    pub fn get_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id.index()]
    }

    /// Append `child` as the last child of `parent`. Detaches from any current parent first.
    pub fn append_child(&mut self, parent: NodeId, child: NodeId) {
        self.detach(child);

        let last = self.nodes[parent.index()].last_child;

        self.nodes[child.index()].parent = Some(parent);
        self.nodes[child.index()].prev_sibling = last;
        self.nodes[child.index()].next_sibling = None;

        if let Some(last_id) = last {
            self.nodes[last_id.index()].next_sibling = Some(child);
        } else {
            self.nodes[parent.index()].first_child = Some(child);
        }

        self.nodes[parent.index()].last_child = Some(child);
    }

    /// Insert `child` before `reference` in `reference`'s parent. Detaches from any current parent first.
    /// Panics if `reference` has no parent.
    pub fn insert_before(&mut self, child: NodeId, reference: NodeId) {
        let parent = self.nodes[reference.index()]
            .parent
            .expect("insert_before: reference node has no parent");

        self.detach(child);

        let prev = self.nodes[reference.index()].prev_sibling;

        self.nodes[child.index()].parent = Some(parent);
        self.nodes[child.index()].next_sibling = Some(reference);
        self.nodes[child.index()].prev_sibling = prev;

        self.nodes[reference.index()].prev_sibling = Some(child);

        if let Some(prev_id) = prev {
            self.nodes[prev_id.index()].next_sibling = Some(child);
        } else {
            self.nodes[parent.index()].first_child = Some(child);
        }
    }

    /// Detach a node from its parent and siblings (does not remove from arena).
    pub fn detach(&mut self, node: NodeId) {
        let parent = self.nodes[node.index()].parent;
        let prev = self.nodes[node.index()].prev_sibling;
        let next = self.nodes[node.index()].next_sibling;

        if let Some(parent_id) = parent {
            if self.nodes[parent_id.index()].first_child == Some(node) {
                self.nodes[parent_id.index()].first_child = next;
            }
            if self.nodes[parent_id.index()].last_child == Some(node) {
                self.nodes[parent_id.index()].last_child = prev;
            }
        }

        if let Some(prev_id) = prev {
            self.nodes[prev_id.index()].next_sibling = next;
        }

        if let Some(next_id) = next {
            self.nodes[next_id.index()].prev_sibling = prev;
        }

        self.nodes[node.index()].parent = None;
        self.nodes[node.index()].prev_sibling = None;
        self.nodes[node.index()].next_sibling = None;
    }

    /// Iterate over the children of a node (by id).
    pub fn children(&self, parent: NodeId) -> ChildrenIter<'_> {
        ChildrenIter {
            arena: self,
            next: self.nodes[parent.index()].first_child,
        }
    }

    /// Iterate over the ancestors of a node, starting from the node's parent.
    pub fn ancestors(&self, node: NodeId) -> AncestorsIter<'_> {
        AncestorsIter {
            arena: self,
            next: self.nodes[node.index()].parent,
        }
    }

    /// Returns the number of nodes in the arena.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns true if the arena contains no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over children of a node.
pub struct ChildrenIter<'a> {
    arena: &'a Arena,
    next: Option<NodeId>,
}

impl<'a> Iterator for ChildrenIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        let current = self.next?;
        self.next = self.arena.nodes[current.index()].next_sibling;
        Some(current)
    }
}

/// Iterator over ancestors of a node (parent, grandparent, ...).
pub struct AncestorsIter<'a> {
    arena: &'a Arena,
    next: Option<NodeId>,
}

impl<'a> Iterator for AncestorsIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        let current = self.next?;
        self.next = self.arena.nodes[current.index()].parent;
        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_element(arena: &mut Arena, tag: &str) -> NodeId {
        arena.create_node(NodeData::Element {
            tag_name: tag.to_string(),
            namespace: Namespace::Html,
            attributes: vec![],
        })
    }

    fn child_tags(arena: &Arena, parent: NodeId) -> Vec<String> {
        arena
            .children(parent)
            .map(|id| match &arena.get(id).data {
                NodeData::Element { tag_name, .. } => tag_name.clone(),
                _ => panic!("expected element"),
            })
            .collect()
    }

    #[test]
    fn append_child_single() {
        let mut arena = Arena::new();
        let parent = make_element(&mut arena, "div");
        let child = make_element(&mut arena, "p");

        arena.append_child(parent, child);

        assert_eq!(arena.get(parent).first_child, Some(child));
        assert_eq!(arena.get(parent).last_child, Some(child));
        assert_eq!(arena.get(child).parent, Some(parent));
        assert_eq!(arena.get(child).prev_sibling, None);
        assert_eq!(arena.get(child).next_sibling, None);
    }

    #[test]
    fn append_child_multiple() {
        let mut arena = Arena::new();
        let parent = make_element(&mut arena, "div");
        let a = make_element(&mut arena, "a");
        let b = make_element(&mut arena, "b");
        let c = make_element(&mut arena, "c");

        arena.append_child(parent, a);
        arena.append_child(parent, b);
        arena.append_child(parent, c);

        assert_eq!(child_tags(&arena, parent), vec!["a", "b", "c"]);
        assert_eq!(arena.get(parent).first_child, Some(a));
        assert_eq!(arena.get(parent).last_child, Some(c));
        assert_eq!(arena.get(a).next_sibling, Some(b));
        assert_eq!(arena.get(b).prev_sibling, Some(a));
        assert_eq!(arena.get(b).next_sibling, Some(c));
        assert_eq!(arena.get(c).prev_sibling, Some(b));
    }

    #[test]
    fn insert_before_first() {
        let mut arena = Arena::new();
        let parent = make_element(&mut arena, "div");
        let a = make_element(&mut arena, "a");
        let b = make_element(&mut arena, "b");

        arena.append_child(parent, a);
        arena.insert_before(b, a);

        assert_eq!(child_tags(&arena, parent), vec!["b", "a"]);
        assert_eq!(arena.get(parent).first_child, Some(b));
        assert_eq!(arena.get(parent).last_child, Some(a));
    }

    #[test]
    fn insert_before_middle() {
        let mut arena = Arena::new();
        let parent = make_element(&mut arena, "div");
        let a = make_element(&mut arena, "a");
        let c = make_element(&mut arena, "c");
        let b = make_element(&mut arena, "b");

        arena.append_child(parent, a);
        arena.append_child(parent, c);
        arena.insert_before(b, c);

        assert_eq!(child_tags(&arena, parent), vec!["a", "b", "c"]);
    }

    #[test]
    fn detach_only_child() {
        let mut arena = Arena::new();
        let parent = make_element(&mut arena, "div");
        let child = make_element(&mut arena, "p");

        arena.append_child(parent, child);
        arena.detach(child);

        assert_eq!(arena.get(parent).first_child, None);
        assert_eq!(arena.get(parent).last_child, None);
        assert_eq!(arena.get(child).parent, None);
    }

    #[test]
    fn detach_first_child() {
        let mut arena = Arena::new();
        let parent = make_element(&mut arena, "div");
        let a = make_element(&mut arena, "a");
        let b = make_element(&mut arena, "b");

        arena.append_child(parent, a);
        arena.append_child(parent, b);
        arena.detach(a);

        assert_eq!(child_tags(&arena, parent), vec!["b"]);
        assert_eq!(arena.get(parent).first_child, Some(b));
        assert_eq!(arena.get(parent).last_child, Some(b));
        assert_eq!(arena.get(b).prev_sibling, None);
    }

    #[test]
    fn detach_last_child() {
        let mut arena = Arena::new();
        let parent = make_element(&mut arena, "div");
        let a = make_element(&mut arena, "a");
        let b = make_element(&mut arena, "b");

        arena.append_child(parent, a);
        arena.append_child(parent, b);
        arena.detach(b);

        assert_eq!(child_tags(&arena, parent), vec!["a"]);
        assert_eq!(arena.get(parent).first_child, Some(a));
        assert_eq!(arena.get(parent).last_child, Some(a));
        assert_eq!(arena.get(a).next_sibling, None);
    }

    #[test]
    fn detach_middle_child() {
        let mut arena = Arena::new();
        let parent = make_element(&mut arena, "div");
        let a = make_element(&mut arena, "a");
        let b = make_element(&mut arena, "b");
        let c = make_element(&mut arena, "c");

        arena.append_child(parent, a);
        arena.append_child(parent, b);
        arena.append_child(parent, c);
        arena.detach(b);

        assert_eq!(child_tags(&arena, parent), vec!["a", "c"]);
        assert_eq!(arena.get(a).next_sibling, Some(c));
        assert_eq!(arena.get(c).prev_sibling, Some(a));
    }

    #[test]
    fn reparent_via_append() {
        let mut arena = Arena::new();
        let p1 = make_element(&mut arena, "div");
        let p2 = make_element(&mut arena, "span");
        let child = make_element(&mut arena, "a");

        arena.append_child(p1, child);
        assert_eq!(child_tags(&arena, p1), vec!["a"]);

        arena.append_child(p2, child);
        assert_eq!(child_tags(&arena, p1), Vec::<String>::new());
        assert_eq!(child_tags(&arena, p2), vec!["a"]);
        assert_eq!(arena.get(child).parent, Some(p2));
    }

    #[test]
    fn ancestors_iteration() {
        let mut arena = Arena::new();
        let root = make_element(&mut arena, "html");
        let body = make_element(&mut arena, "body");
        let div = make_element(&mut arena, "div");
        let p = make_element(&mut arena, "p");

        arena.append_child(root, body);
        arena.append_child(body, div);
        arena.append_child(div, p);

        let ancestor_tags: Vec<String> = arena
            .ancestors(p)
            .map(|id| match &arena.get(id).data {
                NodeData::Element { tag_name, .. } => tag_name.clone(),
                _ => panic!("expected element"),
            })
            .collect();
        assert_eq!(ancestor_tags, vec!["div", "body", "html"]);
    }
}

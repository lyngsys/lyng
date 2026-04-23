use std::collections::HashMap;

use lyng_dom::element::{Attribute, Namespace};
use lyng_dom::node::{Arena, NodeData, NodeId};

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveFormattingSnapshot {
    pub tag_name: String,
    pub namespace: Namespace,
    pub attributes: Vec<Attribute>,
}

/// An entry in the list of active formatting elements.
#[derive(Debug, Clone, PartialEq)]
pub enum ActiveFormattingEntry {
    Element(NodeId),
    Marker,
}

/// The list of active formatting elements (§13.2.6.3).
#[derive(Default)]
pub struct ActiveFormattingElements {
    pub entries: Vec<ActiveFormattingEntry>,
    snapshots: HashMap<NodeId, ActiveFormattingSnapshot>,
}

impl ActiveFormattingElements {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push an element, enforcing the "Noah's Ark" rule:
    /// at most 3 elements with the same tag, namespace, and attributes between markers.
    pub fn push(&mut self, arena: &Arena, node_id: NodeId) {
        // Count matching elements since last marker
        let mut count = 0;
        let mut oldest_match = None;

        let (tag, ns, attrs) = match &arena.get(node_id).data {
            NodeData::Element {
                tag_name,
                namespace,
                attributes,
            } => (tag_name.clone(), *namespace, attributes.clone()),
            _ => {
                self.entries.push(ActiveFormattingEntry::Element(node_id));
                return;
            }
        };

        for (i, entry) in self.entries.iter().enumerate().rev() {
            match entry {
                ActiveFormattingEntry::Marker => break,
                ActiveFormattingEntry::Element(id) => {
                    if let Some(snapshot) = self.snapshots.get(id) {
                        if snapshot.tag_name == tag
                            && snapshot.namespace == ns
                            && snapshot.attributes == attrs
                        {
                            count += 1;
                            oldest_match = Some(i);
                        }
                    } else if let NodeData::Element {
                        tag_name,
                        namespace,
                        attributes,
                    } = &arena.get(*id).data
                    {
                        if *tag_name == tag && *namespace == ns && *attributes == attrs {
                            count += 1;
                            oldest_match = Some(i);
                        }
                    }
                }
            }
        }

        // Noah's Ark: remove the earliest match if we already have 3
        if count >= 3 {
            if let Some(idx) = oldest_match {
                if let ActiveFormattingEntry::Element(id) = self.entries.remove(idx) {
                    self.snapshots.remove(&id);
                }
            }
        }

        self.snapshots.insert(
            node_id,
            ActiveFormattingSnapshot {
                tag_name: tag,
                namespace: ns,
                attributes: attrs,
            },
        );
        self.entries.push(ActiveFormattingEntry::Element(node_id));
    }

    /// Push a marker (scope boundary).
    pub fn push_marker(&mut self) {
        self.entries.push(ActiveFormattingEntry::Marker);
    }

    /// Clear entries up to and including the last marker.
    pub fn clear_up_to_last_marker(&mut self) {
        while let Some(entry) = self.entries.pop() {
            match entry {
                ActiveFormattingEntry::Element(id) => {
                    self.snapshots.remove(&id);
                }
                ActiveFormattingEntry::Marker => break,
            }
        }
    }

    /// Check if an element is in the list.
    pub fn contains(&self, node_id: NodeId) -> bool {
        self.entries
            .iter()
            .any(|e| matches!(e, ActiveFormattingEntry::Element(id) if *id == node_id))
    }

    /// Remove an element from the list.
    pub fn remove(&mut self, node_id: NodeId) {
        if let Some(pos) = self
            .entries
            .iter()
            .position(|e| matches!(e, ActiveFormattingEntry::Element(id) if *id == node_id))
        {
            self.entries.remove(pos);
            self.snapshots.remove(&node_id);
        }
    }

    /// Remove the entry at a known index.
    pub fn remove_at(&mut self, index: usize) -> Option<NodeId> {
        match self.entries.remove(index) {
            ActiveFormattingEntry::Element(id) => {
                self.snapshots.remove(&id);
                Some(id)
            }
            ActiveFormattingEntry::Marker => None,
        }
    }

    /// Find the position and node_id of an element by tag name, searching backwards.
    /// Returns None if not found or a marker is encountered first.
    pub fn find_by_tag(&self, arena: &Arena, tag_name: &str) -> Option<(usize, NodeId)> {
        for (i, entry) in self.entries.iter().enumerate().rev() {
            match entry {
                ActiveFormattingEntry::Marker => return None,
                ActiveFormattingEntry::Element(id) => {
                    if let NodeData::Element { tag_name: t, .. } = &arena.get(*id).data {
                        if t == tag_name {
                            return Some((i, *id));
                        }
                    }
                }
            }
        }
        None
    }

    /// Replace entry at index with a new element.
    pub fn replace(&mut self, index: usize, node_id: NodeId) {
        let old_id = match self.entries[index] {
            ActiveFormattingEntry::Element(id) => Some(id),
            ActiveFormattingEntry::Marker => None,
        };
        if let Some(old_id) = old_id {
            if let Some(snapshot) = self.snapshots.remove(&old_id) {
                self.snapshots.insert(node_id, snapshot);
            }
        }
        self.entries[index] = ActiveFormattingEntry::Element(node_id);
    }

    /// Insert an element at a specific index.
    pub fn insert(&mut self, index: usize, node_id: NodeId) {
        self.entries
            .insert(index, ActiveFormattingEntry::Element(node_id));
    }

    pub fn snapshot(&self, node_id: NodeId) -> Option<&ActiveFormattingSnapshot> {
        self.snapshots.get(&node_id)
    }

    /// Reconstruct the active formatting elements (§13.2.6.3).
    /// Returns a list of elements that need to be re-created and pushed.
    pub fn reconstruct(
        &mut self,
        arena: &mut Arena,
        open_elements: &mut super::open::OpenElementsStack,
    ) {
        if self.entries.is_empty() {
            return;
        }

        // Check if the last entry is a marker or already in the stack
        match self.entries.last() {
            Some(ActiveFormattingEntry::Marker) => return,
            Some(ActiveFormattingEntry::Element(id)) => {
                if open_elements.elements.contains(id) {
                    return;
                }
            }
            None => return,
        }

        let mut i = self.entries.len() - 1;

        // Step back until we find a marker or an element in the stack
        loop {
            if i == 0 {
                break;
            }
            i -= 1;
            match &self.entries[i] {
                ActiveFormattingEntry::Marker => {
                    i += 1;
                    break;
                }
                ActiveFormattingEntry::Element(id) => {
                    if open_elements.elements.contains(id) {
                        i += 1;
                        break;
                    }
                }
            }
        }

        // Now advance forward, creating new elements
        while i < self.entries.len() {
            if let ActiveFormattingEntry::Element(old_id) = &self.entries[i] {
                let old_id = *old_id;
                // Create a new element with the same tag/namespace/attributes
                let new_id = if let Some(snapshot) = self.snapshots.get(&old_id) {
                    arena.create_node(NodeData::Element {
                        tag_name: snapshot.tag_name.clone(),
                        namespace: snapshot.namespace,
                        attributes: snapshot.attributes.clone(),
                    })
                } else if let NodeData::Element {
                    tag_name,
                    namespace,
                    attributes,
                } = &arena.get(old_id).data
                {
                    arena.create_node(NodeData::Element {
                        tag_name: tag_name.clone(),
                        namespace: *namespace,
                        attributes: attributes.clone(),
                    })
                } else {
                    i += 1;
                    continue;
                };

                // Insert into DOM
                let target = open_elements.current_node().unwrap();
                arena.append_child(target, new_id);
                open_elements.push(new_id);

                // Replace in active formatting list
                self.entries[i] = ActiveFormattingEntry::Element(new_id);
            }
            i += 1;
        }
    }
}

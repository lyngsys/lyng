use lyng_dom::element::Namespace;
use lyng_dom::node::{Arena, NodeData, NodeId};

/// The stack of open elements maintained during tree construction.
#[derive(Default)]
pub struct OpenElementsStack {
    pub elements: Vec<NodeId>,
}

impl OpenElementsStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, node_id: NodeId) {
        self.elements.push(node_id);
    }

    pub fn pop(&mut self) -> Option<NodeId> {
        self.elements.pop()
    }

    pub fn current_node(&self) -> Option<NodeId> {
        self.elements.last().copied()
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn contains_tag(&self, arena: &Arena, tag_name: &str) -> bool {
        self.elements.iter().any(|&id| {
            matches!(&arena.get(id).data, NodeData::Element { tag_name: t, .. } if t == tag_name)
        })
    }

    pub fn contains_html_tag(&self, arena: &Arena, tag_name: &str) -> bool {
        self.elements.iter().any(|&id| {
            matches!(&arena.get(id).data, NodeData::Element { tag_name: t, namespace, .. } if t == tag_name && *namespace == Namespace::Html)
        })
    }

    pub fn remove(&mut self, node_id: NodeId) {
        if let Some(pos) = self.elements.iter().position(|&id| id == node_id) {
            self.elements.remove(pos);
        }
    }

    /// Pop elements until (and including) the element with the given tag name.
    pub fn pop_until(&mut self, arena: &Arena, tag_name: &str) {
        while let Some(id) = self.pop() {
            if let NodeData::Element { tag_name: t, .. } = &arena.get(id).data {
                if t == tag_name {
                    return;
                }
            }
        }
    }

    /// Pop elements until (and including) an HTML element with the given tag name.
    pub fn pop_until_html(&mut self, arena: &Arena, tag_name: &str) {
        while let Some(id) = self.pop() {
            if let NodeData::Element {
                tag_name: t,
                namespace,
                ..
            } = &arena.get(id).data
            {
                if t == tag_name && *namespace == Namespace::Html {
                    return;
                }
            }
        }
    }

    /// Pop elements until (and including) one of the given tag names.
    pub fn pop_until_one_of(&mut self, arena: &Arena, tag_names: &[&str]) {
        while let Some(id) = self.pop() {
            if let NodeData::Element { tag_name: t, .. } = &arena.get(id).data {
                if tag_names.contains(&t.as_str()) {
                    return;
                }
            }
        }
    }

    /// Generate implied end tags, optionally excluding a specific tag.
    pub fn generate_implied_end_tags(&mut self, arena: &Arena, exclude: Option<&str>) {
        const IMPLIED: &[&str] = &[
            "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc",
        ];
        loop {
            if let Some(&id) = self.elements.last() {
                if let NodeData::Element { tag_name, .. } = &arena.get(id).data {
                    if IMPLIED.contains(&tag_name.as_str())
                        && exclude.is_none_or(|ex| tag_name != ex)
                    {
                        self.pop();
                        continue;
                    }
                }
            }
            break;
        }
    }

    /// Generate all implied end tags thoroughly (used at end of parsing).
    pub fn generate_all_implied_end_tags_thoroughly(&mut self, arena: &Arena) {
        const IMPLIED: &[&str] = &[
            "caption", "colgroup", "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt",
            "rtc", "tbody", "td", "tfoot", "th", "thead", "tr",
        ];
        loop {
            if let Some(&id) = self.elements.last() {
                if let NodeData::Element { tag_name, .. } = &arena.get(id).data {
                    if IMPLIED.contains(&tag_name.as_str()) {
                        self.pop();
                        continue;
                    }
                }
            }
            break;
        }
    }

    // -----------------------------------------------------------------------
    // "Has an element in scope" checks (§13.2.4.2)
    // -----------------------------------------------------------------------

    /// Check if element with tag_name is "in scope".
    pub fn has_in_scope(&self, arena: &Arena, tag_name: &str) -> bool {
        self.has_in_specific_scope(arena, tag_name, &SCOPE_BOUNDARY)
    }

    /// Check if element with tag_name is "in list item scope".
    pub fn has_in_list_item_scope(&self, arena: &Arena, tag_name: &str) -> bool {
        self.has_in_specific_scope(arena, tag_name, &LIST_ITEM_SCOPE_BOUNDARY)
    }

    /// Check if element with tag_name is "in button scope".
    pub fn has_in_button_scope(&self, arena: &Arena, tag_name: &str) -> bool {
        self.has_in_specific_scope(arena, tag_name, &BUTTON_SCOPE_BOUNDARY)
    }

    /// Check if element with tag_name is "in table scope".
    pub fn has_in_table_scope(&self, arena: &Arena, tag_name: &str) -> bool {
        self.has_in_specific_scope(arena, tag_name, &TABLE_SCOPE_BOUNDARY)
    }

    /// Check if element with tag_name is "in select scope".
    pub fn has_in_select_scope(&self, arena: &Arena, tag_name: &str) -> bool {
        // In select scope: everything is a boundary EXCEPT optgroup and option
        for &id in self.elements.iter().rev() {
            if let NodeData::Element {
                tag_name: t,
                namespace,
                ..
            } = &arena.get(id).data
            {
                if t == tag_name && *namespace == Namespace::Html {
                    return true;
                }
                if t != "optgroup" && t != "option" {
                    return false;
                }
            }
        }
        false
    }

    fn has_in_specific_scope(
        &self,
        arena: &Arena,
        tag_name: &str,
        boundary: &[(&str, Namespace)],
    ) -> bool {
        for &id in self.elements.iter().rev() {
            if let NodeData::Element {
                tag_name: t,
                namespace,
                ..
            } = &arena.get(id).data
            {
                if t == tag_name && *namespace == Namespace::Html {
                    return true;
                }
                if boundary.iter().any(|&(b, ns)| t == b && *namespace == ns) {
                    return false;
                }
            }
        }
        false
    }

    /// Get the tag name of a node (or empty string if not an element).
    pub fn tag_name<'a>(&self, arena: &'a Arena, id: NodeId) -> &'a str {
        match &arena.get(id).data {
            NodeData::Element { tag_name, .. } => tag_name,
            _ => "",
        }
    }

    /// Get the namespace of a node.
    pub fn namespace(&self, arena: &Arena, id: NodeId) -> Namespace {
        match &arena.get(id).data {
            NodeData::Element { namespace, .. } => *namespace,
            _ => Namespace::Html,
        }
    }
}

/// Scope boundary elements for "in scope" checks.
const SCOPE_BOUNDARY: [(&str, Namespace); 18] = [
    ("applet", Namespace::Html),
    ("caption", Namespace::Html),
    ("html", Namespace::Html),
    ("table", Namespace::Html),
    ("td", Namespace::Html),
    ("th", Namespace::Html),
    ("marquee", Namespace::Html),
    ("object", Namespace::Html),
    ("template", Namespace::Html),
    ("mi", Namespace::MathML),
    ("mo", Namespace::MathML),
    ("mn", Namespace::MathML),
    ("ms", Namespace::MathML),
    ("mtext", Namespace::MathML),
    ("annotation-xml", Namespace::MathML),
    ("foreignObject", Namespace::Svg),
    ("desc", Namespace::Svg),
    ("title", Namespace::Svg),
];

const LIST_ITEM_SCOPE_BOUNDARY: [(&str, Namespace); 20] = [
    ("applet", Namespace::Html),
    ("caption", Namespace::Html),
    ("html", Namespace::Html),
    ("table", Namespace::Html),
    ("td", Namespace::Html),
    ("th", Namespace::Html),
    ("marquee", Namespace::Html),
    ("object", Namespace::Html),
    ("template", Namespace::Html),
    ("mi", Namespace::MathML),
    ("mo", Namespace::MathML),
    ("mn", Namespace::MathML),
    ("ms", Namespace::MathML),
    ("mtext", Namespace::MathML),
    ("annotation-xml", Namespace::MathML),
    ("foreignObject", Namespace::Svg),
    ("desc", Namespace::Svg),
    ("title", Namespace::Svg),
    ("ol", Namespace::Html),
    ("ul", Namespace::Html),
];

const BUTTON_SCOPE_BOUNDARY: [(&str, Namespace); 19] = [
    ("applet", Namespace::Html),
    ("caption", Namespace::Html),
    ("html", Namespace::Html),
    ("table", Namespace::Html),
    ("td", Namespace::Html),
    ("th", Namespace::Html),
    ("marquee", Namespace::Html),
    ("object", Namespace::Html),
    ("template", Namespace::Html),
    ("mi", Namespace::MathML),
    ("mo", Namespace::MathML),
    ("mn", Namespace::MathML),
    ("ms", Namespace::MathML),
    ("mtext", Namespace::MathML),
    ("annotation-xml", Namespace::MathML),
    ("foreignObject", Namespace::Svg),
    ("desc", Namespace::Svg),
    ("title", Namespace::Svg),
    ("button", Namespace::Html),
];

const TABLE_SCOPE_BOUNDARY: [(&str, Namespace); 3] = [
    ("html", Namespace::Html),
    ("table", Namespace::Html),
    ("template", Namespace::Html),
];

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_dom::node::NodeData;

    fn make_element(arena: &mut Arena, tag: &str) -> NodeId {
        arena.create_node(NodeData::Element {
            tag_name: tag.to_string(),
            namespace: Namespace::Html,
            attributes: vec![],
        })
    }

    #[test]
    fn scope_check_basic() {
        let mut arena = Arena::new();
        let mut stack = OpenElementsStack::new();

        let html = make_element(&mut arena, "html");
        let body = make_element(&mut arena, "body");
        let div = make_element(&mut arena, "div");
        let p = make_element(&mut arena, "p");

        stack.push(html);
        stack.push(body);
        stack.push(div);
        stack.push(p);

        assert!(stack.has_in_scope(&arena, "p"));
        assert!(stack.has_in_scope(&arena, "div"));
        assert!(stack.has_in_scope(&arena, "body"));
        assert!(!stack.has_in_scope(&arena, "span"));
    }

    #[test]
    fn scope_check_blocked_by_boundary() {
        let mut arena = Arena::new();
        let mut stack = OpenElementsStack::new();

        let html = make_element(&mut arena, "html");
        let body = make_element(&mut arena, "body");
        let table = make_element(&mut arena, "table"); // scope boundary
        let p = make_element(&mut arena, "p");

        stack.push(html);
        stack.push(body);
        stack.push(table);
        stack.push(p);

        assert!(stack.has_in_scope(&arena, "p"));
        // body is behind the table boundary, so NOT in scope
        assert!(!stack.has_in_scope(&arena, "body"));
    }

    #[test]
    fn table_scope() {
        let mut arena = Arena::new();
        let mut stack = OpenElementsStack::new();

        let html = make_element(&mut arena, "html");
        let table = make_element(&mut arena, "table");
        let tr = make_element(&mut arena, "tr");

        stack.push(html);
        stack.push(table);
        stack.push(tr);

        assert!(stack.has_in_table_scope(&arena, "table"));
        assert!(!stack.has_in_table_scope(&arena, "div"));
    }
}

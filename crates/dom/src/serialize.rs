use super::element::Namespace;
use super::node::{Arena, NodeData, NodeId};

/// Serialize a DOM tree in the html5lib test format.
pub fn serialize_tree(arena: &Arena, document: NodeId) -> String {
    let mut output = String::new();
    serialize_children(arena, document, 0, &mut output);
    output
}

fn serialize_children(arena: &Arena, parent: NodeId, indent: usize, output: &mut String) {
    let mut child = arena.get(parent).first_child;
    while let Some(id) = child {
        serialize_node(arena, id, indent, output);
        child = arena.get(id).next_sibling;
    }
}

fn serialize_node(arena: &Arena, node: NodeId, indent: usize, output: &mut String) {
    let prefix = "| ".to_string() + &"  ".repeat(indent);

    match &arena.get(node).data {
        NodeData::Document { .. } => {
            serialize_children(arena, node, indent, output);
        }
        NodeData::Element {
            tag_name,
            namespace,
            attributes,
        } => {
            match namespace {
                Namespace::Svg => {
                    output.push_str(&format!("{}<svg {}>", prefix, tag_name));
                }
                Namespace::MathML => {
                    output.push_str(&format!("{}<math {}>", prefix, tag_name));
                }
                Namespace::Html => {
                    output.push_str(&format!("{}<{}>", prefix, tag_name));
                }
            }
            output.push('\n');

            let mut attrs: Vec<_> = attributes.iter().collect();
            attrs.sort_by(|a, b| {
                let a_full = match &a.prefix {
                    Some(p) => format!("{} {}", p, a.name),
                    None => a.name.clone(),
                };
                let b_full = match &b.prefix {
                    Some(p) => format!("{} {}", p, b.name),
                    None => b.name.clone(),
                };
                a_full.cmp(&b_full)
            });
            for attr in attrs {
                let attr_display = match &attr.prefix {
                    Some(p) => format!("{} {}", p, attr.name),
                    None => attr.name.clone(),
                };
                output.push_str(&format!(
                    "{}  {}=\"{}\"\n",
                    prefix, attr_display, attr.value
                ));
            }

            if tag_name == "template" && *namespace == Namespace::Html {
                output.push_str(&format!("{}  content\n", prefix));
                serialize_children(arena, node, indent + 2, output);
            } else {
                serialize_children(arena, node, indent + 1, output);
            }
        }
        NodeData::Text { content } => {
            output.push_str(&format!("{}\"{}\"", prefix, content));
            output.push('\n');
        }
        NodeData::Comment { content } => {
            output.push_str(&format!("{}<!-- {} -->", prefix, content));
            output.push('\n');
        }
        NodeData::Doctype {
            name,
            public_id,
            system_id,
        } => {
            if !public_id.is_empty() || !system_id.is_empty() {
                output.push_str(&format!(
                    "{}<!DOCTYPE {} \"{}\" \"{}\">",
                    prefix, name, public_id, system_id
                ));
            } else {
                output.push_str(&format!("{}<!DOCTYPE {}>", prefix, name));
            }
            output.push('\n');
        }
        NodeData::ProcessingInstruction { target, data } => {
            output.push_str(&format!("{}<?{} {}>", prefix, target, data));
            output.push('\n');
        }
    }
}

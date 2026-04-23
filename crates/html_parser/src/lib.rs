pub mod error;
pub mod input;
pub mod tokenizer;
pub mod tree;

pub mod dom {
    pub use lyng_dom::doctype;
    pub use lyng_dom::document;
    pub use lyng_dom::element;
    pub use lyng_dom::node;
    pub use lyng_dom::serialize;
    pub use lyng_dom::text;
    pub use lyng_dom::{
        serialize_tree, Arena, Attribute, Namespace, Node, NodeData, NodeId, QuirksMode,
    };
}

pub use tree::builder::{parse_str, ParseResult};

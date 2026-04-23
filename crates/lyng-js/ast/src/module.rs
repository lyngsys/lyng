//! Module root node.

use lyng_js_common::Span;

use crate::ids::{NodeList, StmtId};

/// The root node of a parsed module.
#[derive(Clone, Debug)]
pub struct Module {
    pub span: Span,
    /// Top-level statements and import/export declarations.
    pub body: NodeList<StmtId>,
}

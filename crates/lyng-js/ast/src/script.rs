//! Script root node.

use lyng_js_common::Span;

use crate::ids::{NodeList, StmtId};

/// The root node of a parsed script.
#[derive(Clone, Debug)]
pub struct Script {
    pub span: Span,
    /// Top-level statements (including declarations wrapped in `Stmt::Declaration`).
    pub body: NodeList<StmtId>,
}

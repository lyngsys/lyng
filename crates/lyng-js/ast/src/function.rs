//! Function and class body AST nodes.
//!
//! The `Function` node is shared by function declarations, function expressions,
//! arrow functions, and method definitions. `ClassElement` covers methods,
//! properties, and static blocks in class bodies.

use lyng_js_common::{AtomId, Span};

use crate::common::{FunctionKind, MethodKind};
use crate::ids::{ExprId, FunctionId, NodeList, PatternId, StmtId};

/// A function node (shared by declarations, expressions, methods, arrows).
#[derive(Clone, Debug)]
pub struct Function {
    pub span: Span,
    /// The function name, if any.
    pub name: Option<AtomId>,
    /// What kind of function this is.
    pub kind: FunctionKind,
    /// The formal parameters.
    pub params: FormalParameters,
    /// The function body statements (empty for expression-body arrows).
    pub body: NodeList<StmtId>,
    /// For arrow functions with expression body: `(x) => expr`.
    /// This is `Some(expr_id)` when the body is a single expression.
    pub expression_body: Option<ExprId>,
}

/// Formal parameters of a function.
#[derive(Clone, Debug)]
pub struct FormalParameters {
    pub span: Span,
    /// The parameter patterns.
    pub params: NodeList<PatternId>,
    /// An optional rest parameter: `...rest`.
    pub rest: Option<PatternId>,
}

/// A class body element.
#[derive(Clone, Debug)]
pub enum ClassElement {
    /// A method definition: `method() {}`, `get x() {}`, `static f() {}`.
    Method {
        span: Span,
        kind: MethodKind,
        key: ExprId,
        value: FunctionId,
        computed: bool,
        private: bool,
        r#static: bool,
    },

    /// A class field: `x = 1;`, `static y;`.
    Property {
        span: Span,
        key: ExprId,
        value: Option<ExprId>,
        computed: bool,
        private: bool,
        r#static: bool,
    },

    /// A static initialization block: `static { ... }`.
    StaticBlock { span: Span, body: NodeList<StmtId> },

    // -- Error recovery ----------------------------------------------------
    /// A placeholder for class elements that failed to parse.
    InvalidElement { span: Span },
}

impl ClassElement {
    /// Returns the span of this class element.
    pub fn span(&self) -> Span {
        match self {
            Self::Method { span, .. }
            | Self::Property { span, .. }
            | Self::StaticBlock { span, .. }
            | Self::InvalidElement { span, .. } => *span,
        }
    }
}

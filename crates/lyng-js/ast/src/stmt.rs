//! Statement AST nodes (ECMA-262 §14).
//!
//! Each variant carries a `Span` plus minimal child-ID payload.

use lyng_js_common::{AtomId, Span};

use crate::ids::{DeclId, ExprId, NodeList, PatternId, StmtId};

/// A statement node.
#[derive(Clone, Debug)]
pub enum Stmt {
    // -- Block / Empty -----------------------------------------------------
    /// `{ ... }`
    Block { span: Span, body: NodeList<StmtId> },

    /// `;`
    Empty { span: Span },

    // -- Expression statement ----------------------------------------------
    /// `expr;`
    Expression { span: Span, expression: ExprId },

    // -- Control flow ------------------------------------------------------
    /// `if (test) consequent else alternate`
    If {
        span: Span,
        test: ExprId,
        consequent: StmtId,
        alternate: Option<StmtId>,
    },

    /// `do body while (test);`
    DoWhile {
        span: Span,
        body: StmtId,
        test: ExprId,
    },

    /// `while (test) body`
    While {
        span: Span,
        test: ExprId,
        body: StmtId,
    },

    /// `for (init; test; update) body`
    For {
        span: Span,
        init: Option<ForInit>,
        test: Option<ExprId>,
        update: Option<ExprId>,
        body: StmtId,
    },

    /// `for (left in right) body`
    ForIn {
        span: Span,
        left: ForInOfLeft,
        right: ExprId,
        body: StmtId,
    },

    /// `for (left of right) body` / `for await (left of right) body`
    ForOf {
        span: Span,
        left: ForInOfLeft,
        right: ExprId,
        body: StmtId,
        r#await: bool,
    },

    // -- Jump statements ---------------------------------------------------
    /// `continue;` / `continue label;`
    Continue { span: Span, label: Option<AtomId> },

    /// `break;` / `break label;`
    Break { span: Span, label: Option<AtomId> },

    /// `return;` / `return expr;`
    Return {
        span: Span,
        argument: Option<ExprId>,
    },

    // -- With / Switch / Labeled -------------------------------------------
    /// `with (object) body`
    With {
        span: Span,
        object: ExprId,
        body: StmtId,
    },

    /// `switch (discriminant) { cases }`
    Switch {
        span: Span,
        discriminant: ExprId,
        cases: NodeList<SwitchCase>,
    },

    /// `label: body`
    Labeled {
        span: Span,
        label: AtomId,
        body: StmtId,
    },

    // -- Throw / Try / Debugger --------------------------------------------
    /// `throw expr;`
    Throw { span: Span, argument: ExprId },

    /// `try { block } catch (param) { handler } finally { finalizer }`
    Try {
        span: Span,
        block: StmtId,
        handler: Option<CatchClause>,
        finalizer: Option<StmtId>,
    },

    /// `debugger;`
    Debugger { span: Span },

    // -- Declaration (wrapper) ---------------------------------------------
    /// A declaration used as a statement.
    Declaration { span: Span, decl: DeclId },

    // -- Error recovery ----------------------------------------------------
    /// A placeholder for statements that failed to parse.
    InvalidStatement { span: Span },
}

/// The initializer in a `for` loop.
#[derive(Clone, Copy, Debug)]
pub enum ForInit {
    /// `for (var/let/const x = ...; ...)` — a variable declaration.
    Declaration(DeclId),
    /// `for (expr; ...)` — an expression.
    Expression(ExprId),
}

/// The left-hand side of a `for-in` or `for-of` loop.
#[derive(Clone, Copy, Debug)]
pub enum ForInOfLeft {
    /// `for (var/let/const x in/of ...)` — a variable declaration.
    Declaration(DeclId),
    /// `for (pattern in/of ...)` — a destructuring pattern.
    Pattern(PatternId),
    /// `for (expr in/of ...)` — an expression (simple assignment target).
    Expression(ExprId),
}

/// A `case` or `default` clause within a `switch` statement.
#[derive(Clone, Copy, Debug)]
pub struct SwitchCase {
    pub span: Span,
    /// `None` for the `default` case.
    pub test: Option<ExprId>,
    /// The statements in this case.
    pub consequent: NodeList<StmtId>,
}

/// A `catch` clause: `catch (param) { body }`.
#[derive(Clone, Copy, Debug)]
pub struct CatchClause {
    pub span: Span,
    /// The binding pattern for the caught value. `None` for `catch { ... }`.
    pub param: Option<PatternId>,
    /// The catch body (always a Block).
    pub body: StmtId,
}

impl Stmt {
    /// Returns the span of this statement.
    pub fn span(&self) -> Span {
        match self {
            Self::Block { span, .. }
            | Self::Empty { span, .. }
            | Self::Expression { span, .. }
            | Self::If { span, .. }
            | Self::DoWhile { span, .. }
            | Self::While { span, .. }
            | Self::For { span, .. }
            | Self::ForIn { span, .. }
            | Self::ForOf { span, .. }
            | Self::Continue { span, .. }
            | Self::Break { span, .. }
            | Self::Return { span, .. }
            | Self::With { span, .. }
            | Self::Switch { span, .. }
            | Self::Labeled { span, .. }
            | Self::Throw { span, .. }
            | Self::Try { span, .. }
            | Self::Debugger { span, .. }
            | Self::Declaration { span, .. }
            | Self::InvalidStatement { span, .. } => *span,
        }
    }
}

//! Expression AST nodes (ECMA-262 §13).
//!
//! Each variant carries a `Span` and node-specific payload. Child nodes
//! are referenced by typed IDs, and child lists by `NodeList`.

use lyng_js_common::{AtomId, Span};

use crate::common::{AssignOp, BinaryOp, LogicalOp, PropertyKind, UnaryOp, UpdateOp};
use crate::ids::{
    BigIntLiteralId, ClassElementId, ExprId, FunctionId, NodeList, RegExpLiteralId,
    StringLiteralId, TemplateLiteralId,
};
use crate::literal::{NumericLiteral, NumericLiteralSyntax, StringLiteralSyntax};

/// An expression node.
#[derive(Clone, Debug)]
pub enum Expr {
    // -- Primary expressions -----------------------------------------------
    /// `this`
    This { span: Span },

    /// `super` (only valid in member/call position)
    Super { span: Span },

    /// An identifier reference: `foo`
    Identifier { span: Span, name: AtomId },

    // -- Literals ----------------------------------------------------------
    /// `null`
    NullLiteral { span: Span },

    /// `true` or `false`
    BooleanLiteral { span: Span, value: bool },

    /// A numeric literal.
    NumericLiteral {
        span: Span,
        value: NumericLiteral,
        syntax: NumericLiteralSyntax,
    },

    /// A string literal.
    StringLiteral {
        span: Span,
        value: StringLiteralId,
        syntax: StringLiteralSyntax,
    },

    /// A `BigInt` literal.
    BigIntLiteral { span: Span, value: BigIntLiteralId },

    /// A regular expression literal.
    RegExpLiteral { span: Span, value: RegExpLiteralId },

    // -- Compound expressions ----------------------------------------------
    /// `[a, b, ...c]`
    ArrayExpression {
        span: Span,
        /// Elements; `None` entries represent elisions.
        elements: NodeList<Option<ExprId>>,
    },

    /// `{ a: 1, b: 2 }`
    ObjectExpression {
        span: Span,
        properties: NodeList<Property>,
    },

    /// A function expression (including named): `function f() {}`
    FunctionExpression { span: Span, function: FunctionId },

    /// An arrow function: `(a) => b`
    ArrowFunctionExpression { span: Span, function: FunctionId },

    /// A class expression: `class C {}`
    ClassExpression {
        span: Span,
        name: Option<AtomId>,
        super_class: Option<ExprId>,
        body: NodeList<ClassElementId>,
    },

    /// A template literal: `` `hello ${name}` ``
    TemplateLiteral {
        span: Span,
        template: TemplateLiteralId,
    },

    /// A tagged template: `` tag`hello ${name}` ``
    TaggedTemplateExpression {
        span: Span,
        tag: ExprId,
        template: TemplateLiteralId,
    },

    // -- Unary / Update ----------------------------------------------------
    /// `!x`, `typeof x`, `-x`, etc.
    UnaryExpression {
        span: Span,
        operator: UnaryOp,
        argument: ExprId,
    },

    /// `++x`, `x--`, etc.
    UpdateExpression {
        span: Span,
        operator: UpdateOp,
        argument: ExprId,
        prefix: bool,
    },

    // -- Binary / Logical --------------------------------------------------
    /// `a + b`, `a instanceof b`, etc.
    BinaryExpression {
        span: Span,
        operator: BinaryOp,
        left: ExprId,
        right: ExprId,
    },

    /// `a && b`, `a || b`, `a ?? b`
    LogicalExpression {
        span: Span,
        operator: LogicalOp,
        left: ExprId,
        right: ExprId,
    },

    // -- Conditional -------------------------------------------------------
    /// `a ? b : c`
    ConditionalExpression {
        span: Span,
        test: ExprId,
        consequent: ExprId,
        alternate: ExprId,
    },

    // -- Assignment --------------------------------------------------------
    /// `a = b`, `a += b`, etc.
    AssignmentExpression {
        span: Span,
        operator: AssignOp,
        left: ExprId,
        right: ExprId,
    },

    // -- Sequence ----------------------------------------------------------
    /// `a, b, c`
    SequenceExpression {
        span: Span,
        expressions: NodeList<ExprId>,
    },

    // -- Call / Member / New -----------------------------------------------
    /// `f(a, b)`
    CallExpression {
        span: Span,
        callee: ExprId,
        arguments: NodeList<ExprId>,
    },

    /// `new C(a, b)`
    NewExpression {
        span: Span,
        callee: ExprId,
        arguments: NodeList<ExprId>,
    },

    /// `obj.prop` (static member)
    StaticMemberExpression {
        span: Span,
        object: ExprId,
        property: AtomId,
    },

    /// `obj[expr]` (computed member)
    ComputedMemberExpression {
        span: Span,
        object: ExprId,
        property: ExprId,
    },

    /// `obj.#priv` (private field access)
    PrivateMemberExpression {
        span: Span,
        object: ExprId,
        property: AtomId,
    },

    /// `#priv in obj`
    PrivateInExpression {
        span: Span,
        property: AtomId,
        object: ExprId,
    },

    /// `a?.b`, `a?.[b]`, `a?.(b)` — optional chain expression
    OptionalChainExpression { span: Span, base: ExprId },

    // -- Yield / Await -----------------------------------------------------
    /// `yield x`, `yield* x`
    YieldExpression {
        span: Span,
        argument: Option<ExprId>,
        delegate: bool,
    },

    /// `await x`
    AwaitExpression { span: Span, argument: ExprId },

    // -- Spread / Meta / Import --------------------------------------------
    /// `...x` (in call arguments or array literals)
    SpreadElement { span: Span, argument: ExprId },

    /// `new.target` or `import.meta`
    MetaProperty {
        span: Span,
        meta: AtomId,
        property: AtomId,
    },

    /// `import("module")`, `import.defer("module")`, or
    /// `import.source("module")`.
    ImportExpression {
        span: Span,
        phase: ImportExpressionPhase,
        source: ExprId,
        options: Option<ExprId>,
    },

    // -- Parenthesized (preserved for diagnostics) -------------------------
    /// `(expr)` — kept for precise span information and diagnostics.
    ParenthesizedExpression { span: Span, expression: ExprId },

    // -- Error recovery ----------------------------------------------------
    /// A placeholder for expressions that failed to parse.
    InvalidExpression { span: Span },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportExpressionPhase {
    Evaluation,
    Source,
    Defer,
}

impl ImportExpressionPhase {
    #[must_use]
    pub const fn encoded(self) -> i16 {
        match self {
            Self::Evaluation => 0,
            Self::Source => 1,
            Self::Defer => 2,
        }
    }
}

/// An object property: `key: value`, `get key() {}`, shorthand, computed, spread.
#[derive(Clone, Copy, Debug)]
pub struct Property {
    pub span: Span,
    pub kind: PropertyKind,
    /// The property key expression.
    pub key: ExprId,
    /// The property value expression.
    pub value: ExprId,
    /// True for computed property keys: `[expr]: val`.
    pub computed: bool,
    /// True for shorthand: `{ x }` means `{ x: x }`.
    pub shorthand: bool,
    /// True for method shorthand: `{ f() {} }`.
    pub method: bool,
}

impl Expr {
    /// Returns the span of this expression.
    pub fn span(&self) -> Span {
        match self {
            Self::This { span, .. }
            | Self::Super { span, .. }
            | Self::Identifier { span, .. }
            | Self::NullLiteral { span, .. }
            | Self::BooleanLiteral { span, .. }
            | Self::NumericLiteral { span, .. }
            | Self::StringLiteral { span, .. }
            | Self::BigIntLiteral { span, .. }
            | Self::RegExpLiteral { span, .. }
            | Self::ArrayExpression { span, .. }
            | Self::ObjectExpression { span, .. }
            | Self::FunctionExpression { span, .. }
            | Self::ArrowFunctionExpression { span, .. }
            | Self::ClassExpression { span, .. }
            | Self::TemplateLiteral { span, .. }
            | Self::TaggedTemplateExpression { span, .. }
            | Self::UnaryExpression { span, .. }
            | Self::UpdateExpression { span, .. }
            | Self::BinaryExpression { span, .. }
            | Self::LogicalExpression { span, .. }
            | Self::ConditionalExpression { span, .. }
            | Self::AssignmentExpression { span, .. }
            | Self::SequenceExpression { span, .. }
            | Self::CallExpression { span, .. }
            | Self::NewExpression { span, .. }
            | Self::StaticMemberExpression { span, .. }
            | Self::ComputedMemberExpression { span, .. }
            | Self::PrivateMemberExpression { span, .. }
            | Self::PrivateInExpression { span, .. }
            | Self::OptionalChainExpression { span, .. }
            | Self::YieldExpression { span, .. }
            | Self::AwaitExpression { span, .. }
            | Self::SpreadElement { span, .. }
            | Self::MetaProperty { span, .. }
            | Self::ImportExpression { span, .. }
            | Self::ParenthesizedExpression { span, .. }
            | Self::InvalidExpression { span, .. } => *span,
        }
    }
}

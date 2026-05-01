//! Declaration AST nodes (ECMA-262 §14, §16).
//!
//! Declarations can appear as statements (via `Stmt::Declaration`) and at
//! the module/script top level.

use lyng_js_common::{AtomId, Span};

use crate::common::VariableKind;
use crate::ids::{
    ClassElementId, DeclId, ExprId, FunctionId, NodeList, PatternId, StringLiteralId,
};

/// A declaration node.
#[derive(Clone, Debug)]
pub enum Decl {
    // -- Variable / Lexical ------------------------------------------------
    /// `var x = 1, y = 2;` / `let x = 1;` / `const x = 1;`
    Variable {
        span: Span,
        kind: VariableKind,
        declarators: NodeList<VariableDeclarator>,
    },

    // -- Function / Class --------------------------------------------------
    /// `function f() {}` / `function* f() {}` / `async function f() {}`
    Function { span: Span, function: FunctionId },

    /// `class C { ... }`
    Class {
        span: Span,
        name: Option<AtomId>,
        super_class: Option<ExprId>,
        body: NodeList<ClassElementId>,
    },

    // -- Import / Export ---------------------------------------------------
    /// `import ...`
    Import {
        span: Span,
        specifiers: NodeList<ImportSpecifier>,
        source: StringLiteralId,
        attributes: NodeList<ImportAttribute>,
    },

    /// `export ...`
    Export { span: Span, kind: ExportKind },

    // -- Error recovery ----------------------------------------------------
    /// A placeholder for declarations that failed to parse.
    InvalidDeclaration { span: Span },
}

/// A single declarator in a variable declaration: `x = 1`.
#[derive(Clone, Copy, Debug)]
pub struct VariableDeclarator {
    pub span: Span,
    /// The binding pattern or identifier.
    pub id: PatternId,
    /// The optional initializer expression.
    pub init: Option<ExprId>,
}

// ---------------------------------------------------------------------------
// Import specifiers
// ---------------------------------------------------------------------------

/// An import specifier within an `import` declaration.
#[derive(Clone, Copy, Debug)]
pub enum ImportSpecifier {
    /// `import x from "mod"` — default import, bound to local `x`.
    Default { span: Span, local: AtomId },

    /// `import * as x from "mod"` — namespace import.
    Namespace { span: Span, local: AtomId },

    /// `import source x from "mod"` — source-phase import.
    Source { span: Span, local: AtomId },

    /// `import { foo as bar } from "mod"` — named import.
    Named {
        span: Span,
        imported: AtomId,
        local: AtomId,
    },
}

/// One retained import attribute from a `with { ... }` clause.
#[derive(Clone, Copy, Debug)]
pub struct ImportAttribute {
    pub span: Span,
    pub key: AtomId,
    pub value: StringLiteralId,
}

// ---------------------------------------------------------------------------
// Export kinds
// ---------------------------------------------------------------------------

/// The kind of export declaration.
#[derive(Clone, Debug)]
pub enum ExportKind {
    /// `export { a, b }` / `export { a as b } from "mod"`
    Named {
        specifiers: NodeList<ExportSpecifier>,
        /// Present for re-export: `export { a } from "mod"`.
        source: Option<StringLiteralId>,
        attributes: NodeList<ImportAttribute>,
    },

    /// `export default expr`
    Default { declaration: ExportDefaultDecl },

    /// `export * from "mod"` / `export * as ns from "mod"`
    All {
        source: StringLiteralId,
        exported: Option<AtomId>,
        attributes: NodeList<ImportAttribute>,
    },

    /// `export var/let/const ...` / `export function ...` / `export class ...`
    Declaration { decl: DeclId },
}

/// The declaration in an `export default` statement.
#[derive(Clone, Copy, Debug)]
pub enum ExportDefaultDecl {
    /// `export default function f() {}`
    Function(FunctionId),
    /// `export default class C {}`
    Class(DeclId),
    /// `export default expr;`
    Expression(ExprId),
}

/// A single export specifier: `a as b`.
#[derive(Clone, Copy, Debug)]
pub struct ExportSpecifier {
    pub span: Span,
    pub local: AtomId,
    pub exported: AtomId,
}

impl Decl {
    /// Returns the span of this declaration.
    pub fn span(&self) -> Span {
        match self {
            Self::Variable { span, .. }
            | Self::Function { span, .. }
            | Self::Class { span, .. }
            | Self::Import { span, .. }
            | Self::Export { span, .. }
            | Self::InvalidDeclaration { span, .. } => *span,
        }
    }
}

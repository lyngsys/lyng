//! Parse result root wrappers.
//!
//! `ParsedScript` and `ParsedModule` are the top-level results produced by
//! the parser. They own the `Ast` container, root ID, diagnostics, and
//! directive-prologue summary.

use lyng_js_common::DiagnosticList;

use crate::ids::{ModuleId, ScriptId};
use crate::Ast;

/// The result of parsing a script.
pub struct ParsedScript {
    /// The AST container holding all nodes and data.
    pub ast: Ast,
    /// The root script node ID.
    pub root: ScriptId,
    /// Retained source text for later runtime-facing source slices.
    pub source_text: Box<str>,
    /// Syntax diagnostics accumulated during parsing.
    pub diagnostics: DiagnosticList,
    /// Whether a `"use strict"` directive was found in the prologue.
    pub strict: bool,
}

/// The result of parsing a module (always strict).
pub struct ParsedModule {
    /// The AST container holding all nodes and data.
    pub ast: Ast,
    /// The root module node ID.
    pub root: ModuleId,
    /// Retained source text for later runtime-facing source slices.
    pub source_text: Box<str>,
    /// Syntax diagnostics accumulated during parsing.
    pub diagnostics: DiagnosticList,
}

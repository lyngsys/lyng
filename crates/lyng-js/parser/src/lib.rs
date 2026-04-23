//! Recursive-descent parser for ECMA-262 Edition 16.
//!
//! This crate consumes tokens from `lyng_js_lexer` and builds an arena-backed
//! AST defined in `lyng_js_ast`. The two public entry points are `parse_script`
//! and `parse_module`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::doc_markdown,
    clippy::elidable_lifetime_names,
    clippy::if_not_else,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::needless_raw_string_hashes,
    clippy::redundant_else,
    clippy::single_match_else,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::unnested_or_patterns,
    clippy::unreadable_literal
)]

mod decl;
mod expr;
mod function;
mod module;
mod parser;
mod pattern;
mod regexp;
mod regexp_tables;
mod stmt;

#[cfg(test)]
mod tests;

use lyng_js_ast::{Module, ParsedModule, ParsedScript, Script};
use lyng_js_common::{AtomTable, SourceId};
use lyng_js_lexer::{TokenKind, TokenPayload};

use parser::Parser;

pub use regexp::validate_regexp_literal;

/// Parses a script and returns the complete parse result.
pub fn parse_script(atoms: &mut AtomTable, source_id: SourceId, source: &str) -> ParsedScript {
    parse_script_with_initial_strict(atoms, source_id, source, false)
}

/// Parses a script, optionally starting in strict mode before any directive
/// prologue is processed.
pub fn parse_script_with_initial_strict(
    atoms: &mut AtomTable,
    source_id: SourceId,
    source: &str,
    initial_strict: bool,
) -> ParsedScript {
    let mut parser = Parser::new(source, source_id, atoms, true);
    if initial_strict {
        parser.set_strict(true);
    }

    let start_span = parser.current_span();
    let mut stmts = Vec::new();

    // Check for "use strict" directive prologue
    let mut strict = initial_strict;
    if !strict && parser.at(TokenKind::StringLiteral) {
        // Check if this is a "use strict" directive
        let token = parser.current();
        if !token.contains_escape() {
            if let TokenPayload::Literal(lit_id) = token.payload {
                let lit = parser.lexer().literals.get_string(lit_id);
                if lit.equals_text("use strict") {
                    strict = true;
                    parser.set_strict(true);
                }
            }
        }
    }

    parser.enter_statement_list();
    while !parser.at(TokenKind::Eof) {
        let pos_before = parser.current_span().range.start.raw();
        let stmt = parser.parse_statement_list_item();
        stmts.push(stmt);
        // Safety: guarantee forward progress
        if parser.current_span().range.start.raw() == pos_before && !parser.at(TokenKind::Eof) {
            parser.error(format!("unexpected token {:?}", parser.current_kind()));
            parser.advance();
        }
    }
    parser.exit_statement_list();

    let end_span = parser.current_span();
    let span = if stmts.is_empty() {
        start_span
    } else {
        start_span.cover(end_span)
    };

    let (mut ast, diagnostics) = parser.finish();
    let body = ast.alloc_stmt_list(&stmts);
    let root = ast.alloc_script(Script { span, body });

    ParsedScript {
        ast,
        root,
        source_text: source.into(),
        diagnostics,
        strict,
    }
}

/// Parses a module and returns the complete parse result.
pub fn parse_module(atoms: &mut AtomTable, source_id: SourceId, source: &str) -> ParsedModule {
    let mut parser = Parser::new(source, source_id, atoms, false);
    // Modules are always strict and support top-level await
    parser.set_strict(true);
    parser.set_module(true);
    parser.allow_await = true;

    let start_span = parser.current_span();
    let mut stmts = Vec::new();

    parser.enter_statement_list();
    while !parser.at(TokenKind::Eof) {
        let pos_before = parser.current_span().range.start.raw();
        let stmt = parser.parse_module_item();
        stmts.push(stmt);
        // Safety: guarantee forward progress
        if parser.current_span().range.start.raw() == pos_before && !parser.at(TokenKind::Eof) {
            parser.error(format!("unexpected token {:?}", parser.current_kind()));
            parser.advance();
        }
    }
    parser.exit_statement_list();

    let end_span = parser.current_span();
    let span = if stmts.is_empty() {
        start_span
    } else {
        start_span.cover(end_span)
    };

    let (mut ast, diagnostics) = parser.finish();
    let body = ast.alloc_stmt_list(&stmts);
    let root = ast.alloc_module(Module { span, body });

    ParsedModule {
        ast,
        root,
        source_text: source.into(),
        diagnostics,
    }
}

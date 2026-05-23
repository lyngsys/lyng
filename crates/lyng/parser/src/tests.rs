//! Comprehensive tests for the lyng-js parser.

use lyng_js_ast::*;
use lyng_js_common::{AtomTable, SourceId};

use crate::{parse_module, parse_script};

fn sid() -> SourceId {
    SourceId::new(0)
}

/// Helper: parse a script and return the ParsedScript.
fn script(source: &str) -> ParsedScript {
    let mut atoms = AtomTable::new();
    parse_script(&mut atoms, sid(), source)
}

/// Helper: parse a module and return the ParsedModule.
fn module(source: &str) -> ParsedModule {
    let mut atoms = AtomTable::new();
    parse_module(&mut atoms, sid(), source)
}

/// Helper: parse a script, assert no errors, return (ast, root body stmts).
fn script_ok(source: &str) -> ParsedScript {
    let result = script(source);
    assert!(
        !result.diagnostics.has_errors(),
        "expected no errors, got: {:?}",
        result.diagnostics.as_slice()
    );
    result
}

/// Helper: parse a module, assert no errors, return ParsedModule.
fn module_ok(source: &str) -> ParsedModule {
    let result = module(source);
    assert!(
        !result.diagnostics.has_errors(),
        "expected no errors, got: {:?}",
        result.diagnostics.as_slice()
    );
    result
}

/// Get the body stmts from a parsed script.
fn body(p: &ParsedScript) -> &[StmtId] {
    let script = p.ast.get_script(p.root);
    p.ast.get_stmt_list(script.body)
}

/// Get the body stmts from a parsed module.
fn mbody(p: &ParsedModule) -> &[StmtId] {
    let module = p.ast.get_module(p.root);
    p.ast.get_stmt_list(module.body)
}

mod declarations;
mod expressions;
mod functions;
mod misc;
mod modules;
mod patterns;
mod statements;
mod templates;

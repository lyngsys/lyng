//! Memory and performance smoke tests.
//! Validates the streaming parser handles large inputs without blowup.

use std::mem::size_of;
use std::time::Instant;

use lyng_js_common::{AtomTable, SourceId};
use lyng_js_lexer::Token;
use lyng_js_parser::parse_script;
use lyng_js_sema::analyze_script;

/// Phase 1 exit gate: 100k lines, under 5s, no token vector.
#[test]
fn large_script_100k_lines_under_5s() {
    let mut src = String::with_capacity(3_000_000);
    for i in 0..100_000 {
        use std::fmt::Write;
        writeln!(src, "var x{i} = {i};").unwrap();
    }

    let mut atoms = AtomTable::new();

    let start = Instant::now();
    let parsed = parse_script(&mut atoms, SourceId::new(0), &src);
    let sema = analyze_script(&parsed, &atoms);
    let elapsed = start.elapsed();

    assert!(
        !parsed.diagnostics.has_errors(),
        "100k-line script should parse without errors"
    );
    assert!(
        !sema.diagnostics.has_errors(),
        "100k-line script sema should succeed"
    );
    assert!(
        sema.binding_table.len() >= 100_000,
        "should have at least 100k bindings, got {}",
        sema.binding_table.len()
    );
    assert!(
        elapsed.as_secs_f64() < 5.0,
        "100k lines should complete under 5s, took {:.1}s",
        elapsed.as_secs_f64()
    );
}

#[test]
fn large_nested_expressions() {
    let mut src = String::from("var x = 1");
    for i in 2..=1000 {
        use std::fmt::Write;
        write!(src, "+{i}").unwrap();
    }
    src.push(';');

    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, SourceId::new(0), &src);
    assert!(!parsed.diagnostics.has_errors());
}

#[test]
fn large_function_count() {
    let mut src = String::with_capacity(100_000);
    for i in 0..1000 {
        use std::fmt::Write;
        writeln!(src, "function f{i}() {{ return {i}; }}").unwrap();
    }

    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, SourceId::new(0), &src);
    assert!(!parsed.diagnostics.has_errors());

    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
}

#[test]
fn repeated_parse_no_leak() {
    let src = "var x = 1; function f() { return x; }";
    for _ in 0..100 {
        let mut atoms = AtomTable::new();
        let parsed = parse_script(&mut atoms, SourceId::new(0), src);
        assert!(!parsed.diagnostics.has_errors());
        let sema = analyze_script(&parsed, &atoms);
        assert!(!sema.diagnostics.has_errors());
    }
}

/// Coarse regression guard for the streaming parser architecture.
///
/// This does not profile heap allocations directly. Instead, it checks that a
/// large token stream parses successfully and that the compact statement-list
/// representation remains far smaller than a hypothetical retained `Vec<Token>`.
#[test]
fn streaming_no_token_vector() {
    let src = ";".repeat(50_000);

    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, SourceId::new(0), &src);
    assert!(!parsed.diagnostics.has_errors());
    let script = parsed.ast.get_script(parsed.root);
    let stmt_count = parsed.ast.get_stmt_list(script.body).len();
    assert_eq!(stmt_count, 50_000);

    let retained_stmt_ids_bytes = size_of_val(parsed.ast.get_stmt_list(script.body));
    let retained_token_vec_bytes = (stmt_count + 1) * size_of::<Token>();
    assert!(
        retained_stmt_ids_bytes < retained_token_vec_bytes / 2,
        "statement list should stay much smaller than a retained token vector ({retained_stmt_ids_bytes} vs {retained_token_vec_bytes} bytes)",
    );
}

/// Regression: duplicate import-attribute keys must be a `SyntaxError`.
#[test]
fn duplicate_import_attribute_key_is_error() {
    let mut atoms = AtomTable::new();
    let parsed = lyng_js_parser::parse_module(
        &mut atoms,
        SourceId::new(0),
        "import x from './m.js' with { type: 'json', type: 'json' };",
    );
    assert!(
        parsed.diagnostics.has_errors(),
        "duplicate import attribute key should be a SyntaxError"
    );
}

/// Regression: duplicate attribute keys via Unicode escape must also error.
#[test]
fn duplicate_import_attribute_key_unicode_escape() {
    let mut atoms = AtomTable::new();
    let parsed = lyng_js_parser::parse_module(
        &mut atoms,
        SourceId::new(0),
        "import x from './m.js' with { type: 'json', typ\\u0065: '' };",
    );
    // "type" and "typ\u0065" resolve to the same string
    assert!(
        parsed.diagnostics.has_errors(),
        "Unicode-escaped duplicate key should be a SyntaxError"
    );
}

//! End-to-end pipeline tests: source -> parse -> sema.

use lyng_js_common::{AtomTable, SourceId};
use lyng_js_parser::{parse_module, parse_script};
use lyng_js_sema::{
    analyze_module, analyze_script, DeclarationKind, ResolutionKind, ScopeId, ScopeKind,
    StorageClass,
};

fn sid() -> SourceId {
    SourceId::new(0)
}

#[test]
fn script_parse_and_sema() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, sid(), "var x = 1; x + 2;");
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    assert!(!sema.binding_table.is_empty());
}

#[test]
fn module_parse_and_sema() {
    let mut atoms = AtomTable::new();
    let parsed = parse_module(&mut atoms, sid(), "export const x = 42;");
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_module(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
}

#[test]
fn strict_mode_propagates_via_use_strict() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, sid(), "\"use strict\"; var x = 1;");
    assert!(parsed.strict);
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    // The global scope should be strict
    let root_scope = sema.scope_table.get(ScopeId::new(0));
    assert!(root_scope.strict);
}

#[test]
fn syntax_errors_have_correct_spans() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, sid(), "var = ;");
    assert!(parsed.diagnostics.has_errors());
    let diag = &parsed.diagnostics.as_slice()[0];
    // Span should point within the source (source is 7 bytes)
    assert!(diag.span.range.start.raw() < 7);
}

#[test]
fn arrow_function_captures_resolved() {
    let mut atoms = AtomTable::new();
    let src = "var x = 1; var f = () => x;";
    let parsed = parse_script(&mut atoms, sid(), src);
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    // Should have bindings for x and f
    assert!(sema.binding_table.len() >= 2);
    // x should be captured by the arrow
    let x_atom = atoms.intern("x");
    let x_binding = sema
        .binding_table
        .as_slice()
        .iter()
        .find(|b| b.name == x_atom);
    assert!(x_binding.is_some(), "should have binding for x");
    let xb = x_binding.unwrap();
    assert!(xb.is_captured, "x should be marked as captured by arrow");
    assert_eq!(
        xb.storage_class,
        StorageClass::GlobalName,
        "captured global var should stay on the global-name path"
    );
}

#[test]
fn import_bindings_exist_in_sema() {
    let mut atoms = AtomTable::new();
    let parsed = parse_module(&mut atoms, sid(), "import { foo } from 'bar';");
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_module(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    let has_import = sema
        .binding_table
        .as_slice()
        .iter()
        .any(|b| b.kind == DeclarationKind::Import);
    assert!(has_import, "should have an import binding");
}

#[test]
fn use_site_resolution_local() {
    let mut atoms = AtomTable::new();
    let src = "var x = 1; x;";
    let parsed = parse_script(&mut atoms, sid(), src);
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    // The reference to x in "x;" should resolve locally
    let x_atom = atoms.intern("x");
    let x_use = sema.use_sites.as_slice().iter().find(|u| u.name == x_atom);
    assert!(x_use.is_some(), "should have a use site for x");
    let use_rec = x_use.unwrap();
    assert!(use_rec.resolved_binding.is_some());
    assert_eq!(use_rec.resolution_kind, ResolutionKind::Local);
}

#[test]
fn use_site_resolution_captured() {
    let mut atoms = AtomTable::new();
    let src = "var x = 1; function f() { return x; }";
    let parsed = parse_script(&mut atoms, sid(), src);
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    // The reference to x inside f should resolve as Captured
    let x_atom = atoms.intern("x");
    let captured = sema
        .use_sites
        .as_slice()
        .iter()
        .any(|u| u.name == x_atom && u.resolution_kind == ResolutionKind::Captured);
    assert!(captured, "x reference inside function should be captured");
}

#[test]
fn use_site_resolution_global() {
    let mut atoms = AtomTable::new();
    let src = "console.log('hello');";
    let parsed = parse_script(&mut atoms, sid(), src);
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    let console_atom = atoms.intern("console");
    let console_use = sema
        .use_sites
        .as_slice()
        .iter()
        .find(|u| u.name == console_atom);
    assert!(console_use.is_some());
    assert_eq!(console_use.unwrap().resolution_kind, ResolutionKind::Global);
}

#[test]
fn closure_capture_storage_class() {
    let mut atoms = AtomTable::new();
    let src = "function outer() { var x = 1; function inner() { return x; } }";
    let parsed = parse_script(&mut atoms, sid(), src);
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    let x_atom = atoms.intern("x");
    let x_binding = sema
        .binding_table
        .as_slice()
        .iter()
        .find(|b| b.name == x_atom);
    assert!(x_binding.is_some(), "should have binding for x");
    let x = x_binding.unwrap();
    assert!(x.is_captured, "x should be captured");
    assert_eq!(
        x.storage_class,
        StorageClass::EnvironmentSlot,
        "captured var should be EnvironmentSlot"
    );
}

#[test]
fn multiple_scripts_share_atoms() {
    let mut atoms = AtomTable::new();
    let p1 = parse_script(&mut atoms, SourceId::new(0), "var foo = 1;");
    let p2 = parse_script(&mut atoms, SourceId::new(1), "var foo = 2;");
    assert!(!p1.diagnostics.has_errors());
    assert!(!p2.diagnostics.has_errors());
    let s1 = analyze_script(&p1, &atoms);
    let s2 = analyze_script(&p2, &atoms);
    let foo1 = s1
        .binding_table
        .as_slice()
        .iter()
        .find(|b| atoms.resolve(b.name) == "foo");
    let foo2 = s2
        .binding_table
        .as_slice()
        .iter()
        .find(|b| atoms.resolve(b.name) == "foo");
    assert!(foo1.is_some());
    assert!(foo2.is_some());
    assert_eq!(
        foo1.unwrap().name,
        foo2.unwrap().name,
        "atoms should be shared"
    );
}

#[test]
fn module_is_always_strict() {
    let mut atoms = AtomTable::new();
    let parsed = parse_module(&mut atoms, sid(), "var x = 1;");
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_module(&parsed, &atoms);
    let root = sema.scope_table.get(ScopeId::new(0));
    assert_eq!(root.kind, ScopeKind::Module);
    assert!(root.strict, "modules are always strict");
}

#[test]
fn complex_program_end_to_end() {
    let mut atoms = AtomTable::new();
    let src = r#"
        'use strict';
        let counter = 0;
        function increment() {
            counter++;
            return counter;
        }
        const result = increment();
        if (result > 0) {
            let msg = "positive";
        }
        for (let i = 0; i < 10; i++) {
            increment();
        }
    "#;
    let parsed = parse_script(&mut atoms, sid(), src);
    assert!(!parsed.diagnostics.has_errors());
    assert!(parsed.strict);
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    // counter should be captured by increment
    let counter_atom = atoms.intern("counter");
    let counter_b = sema
        .binding_table
        .as_slice()
        .iter()
        .find(|b| b.name == counter_atom);
    assert!(counter_b.is_some());
    assert!(counter_b.unwrap().is_captured);
}

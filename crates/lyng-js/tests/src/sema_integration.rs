//! Sema integration tests using parsed AST (not hand-built).
//! Validates scope structure, binding analysis, captures, and storage classes.

use lyng_js_common::{AtomTable, SourceId};
use lyng_js_parser::{parse_module, parse_script};
use lyng_js_sema::{
    analyze_module, analyze_script, ClassPrivateElementKind, DeclarationKind, ResolutionKind,
    ScopeId, ScopeKind, StorageClass,
};

fn sid() -> SourceId {
    SourceId::new(0)
}

fn sema(src: &str) -> (lyng_js_sema::ScriptSema, AtomTable) {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, sid(), src);
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors: {:?}",
        parsed.diagnostics.as_slice()
    );
    let s = analyze_script(&parsed, &atoms);
    assert!(
        !s.diagnostics.has_errors(),
        "unexpected sema errors: {:?}",
        s.diagnostics.as_slice()
    );
    (s, atoms)
}

fn module_sema(src: &str) -> (lyng_js_sema::ModuleSema, AtomTable) {
    let mut atoms = AtomTable::new();
    let parsed = parse_module(&mut atoms, sid(), src);
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors: {:?}",
        parsed.diagnostics.as_slice()
    );
    let s = analyze_module(&parsed, &atoms);
    assert!(
        !s.diagnostics.has_errors(),
        "unexpected sema errors: {:?}",
        s.diagnostics.as_slice()
    );
    (s, atoms)
}

// === Scope structure ===

#[test]
fn global_scope_exists() {
    let (s, _) = sema("var x = 1;");
    assert!(!s.scope_table.as_slice().is_empty());
    assert_eq!(s.scope_table.get(ScopeId::new(0)).kind, ScopeKind::Global);
}

#[test]
fn function_creates_scope() {
    let (s, _) = sema("function f() { var x; }");
    let has_fn_scope = s
        .scope_table
        .as_slice()
        .iter()
        .any(|sc| sc.kind == ScopeKind::Function);
    assert!(has_fn_scope, "function should create a Function scope");
}

#[test]
fn block_creates_scope() {
    let (s, _) = sema("{ let x = 1; }");
    let has_block = s
        .scope_table
        .as_slice()
        .iter()
        .any(|sc| sc.kind == ScopeKind::Block);
    assert!(has_block, "block with let should create a Block scope");
}

#[test]
fn nested_scope_parents() {
    let (s, _) = sema("function f() { { let x; } }");
    // Should have Global -> Function -> Block
    assert!(s.scope_table.as_slice().len() >= 3);
}

// === Binding analysis ===

#[test]
fn var_hoists_to_function() {
    let (s, mut atoms) = sema("function f() { if (true) { var x; } }");
    let x = atoms.intern("x");
    let x_binding = s.binding_table.as_slice().iter().find(|b| b.name == x);
    assert!(x_binding.is_some(), "var x should be visible");
    assert_eq!(x_binding.unwrap().kind, DeclarationKind::Var);
}

#[test]
fn let_is_block_scoped() {
    let (s, mut atoms) = sema("{ let x = 1; }");
    let x = atoms.intern("x");
    let x_binding = s.binding_table.as_slice().iter().find(|b| b.name == x);
    assert!(x_binding.is_some());
    assert_eq!(x_binding.unwrap().kind, DeclarationKind::Let);
}

#[test]
fn const_is_block_scoped() {
    let (s, mut atoms) = sema("{ const x = 1; }");
    let x = atoms.intern("x");
    let x_binding = s.binding_table.as_slice().iter().find(|b| b.name == x);
    assert!(x_binding.is_some());
    assert_eq!(x_binding.unwrap().kind, DeclarationKind::Const);
}

#[test]
fn function_declaration_binding() {
    let (s, mut atoms) = sema("function foo() {}");
    let foo = atoms.intern("foo");
    let binding = s.binding_table.as_slice().iter().find(|b| b.name == foo);
    assert!(binding.is_some());
    assert_eq!(binding.unwrap().kind, DeclarationKind::Function);
}

#[test]
fn block_function_binding_stays_in_block_scope() {
    let (s, mut atoms) = sema("{ function foo() {} }");
    let foo = atoms.intern("foo");
    let binding = s
        .binding_table
        .as_slice()
        .iter()
        .find(|b| b.name == foo)
        .expect("missing function binding");
    assert_eq!(binding.kind, DeclarationKind::Function);
    assert_eq!(s.scope_table.get(binding.scope).kind, ScopeKind::Block);
}

#[test]
fn annex_b_if_clause_function_skips_lexical_conflict_in_function_scope() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        sid(),
        "(function() { let f = 1; if (true) function f() {} })();",
    );
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors: {:?}",
        parsed.diagnostics.as_slice()
    );

    let sema = analyze_script(&parsed, &atoms);
    assert!(
        !sema.diagnostics.has_errors(),
        "unexpected sema errors: {:?}",
        sema.diagnostics.as_slice()
    );
}

#[test]
fn annex_b_if_clause_function_skips_lexical_conflict_in_global_scope() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, sid(), "let f = 1; if (true) function f() {}");
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors: {:?}",
        parsed.diagnostics.as_slice()
    );

    let sema = analyze_script(&parsed, &atoms);
    assert!(
        !sema.diagnostics.has_errors(),
        "unexpected sema errors: {:?}",
        sema.diagnostics.as_slice()
    );
}

#[test]
fn multiple_var_same_name_ok() {
    // Duplicate var is OK
    let (s, _) = sema("var x = 1; var x = 2;");
    assert!(!s.diagnostics.has_errors());
}

// === Capture analysis ===

#[test]
fn inner_function_captures() {
    let (s, mut atoms) = sema("function outer() { var x = 1; function inner() { return x; } }");
    let x = atoms.intern("x");
    let x_binding = s.binding_table.as_slice().iter().find(|b| b.name == x);
    assert!(x_binding.is_some());
    assert!(
        x_binding.unwrap().is_captured,
        "x should be captured by inner"
    );
}

#[test]
fn uncaptured_stays_local() {
    let (s, mut atoms) = sema("function f() { var x = 1; return x; }");
    let x = atoms.intern("x");
    let x_binding = s.binding_table.as_slice().iter().find(|b| b.name == x);
    assert!(x_binding.is_some());
    assert!(!x_binding.unwrap().is_captured, "x should not be captured");
}

#[test]
fn captured_becomes_env_slot() {
    let (s, mut atoms) = sema("function f() { var x = 1; function g() { return x; } }");
    let x = atoms.intern("x");
    let x_binding = s.binding_table.as_slice().iter().find(|b| b.name == x);
    assert!(x_binding.is_some());
    assert_eq!(
        x_binding.unwrap().storage_class,
        StorageClass::EnvironmentSlot
    );
}

#[test]
fn uncaptured_is_frame_local() {
    let (s, mut atoms) = sema("function f() { var x = 1; return x; }");
    let x = atoms.intern("x");
    let x_binding = s.binding_table.as_slice().iter().find(|b| b.name == x);
    assert!(x_binding.is_some());
    assert_eq!(x_binding.unwrap().storage_class, StorageClass::FrameLocal);
}

// === Use-site resolution ===

#[test]
fn identifier_resolves_to_binding() {
    let (s, _) = sema("var x = 1; x;");
    let resolved = s
        .use_sites
        .as_slice()
        .iter()
        .filter(|u| {
            matches!(
                u.resolution_kind,
                ResolutionKind::Local | ResolutionKind::Global
            )
        })
        .count();
    assert!(resolved > 0);
}

#[test]
fn unresolved_global_reference() {
    let (s, _) = sema("console.log(1);");
    // console should be unresolved or global
    let has_unresolved = s.use_sites.as_slice().iter().any(|u| {
        matches!(
            u.resolution_kind,
            ResolutionKind::Global | ResolutionKind::Unresolved
        )
    });
    assert!(has_unresolved);
}

// === Parameter scope ===

#[test]
fn simple_params_single_scope() {
    let (s, _) = sema("function f(a, b) { return a + b; }");
    // With simple params, there may or may not be a separate param scope
    // Just verify it parses and analyzes correctly
    assert!(!s.diagnostics.has_errors());
}

#[test]
fn non_simple_params_separate_scope() {
    let (s, _) = sema("function f(a = 1) { var x; }");
    // Non-simple params should create Parameter scope
    let has_param_scope = s
        .scope_table
        .as_slice()
        .iter()
        .any(|sc| sc.kind == ScopeKind::Parameter);
    assert!(
        has_param_scope,
        "non-simple params should create Parameter scope"
    );
}

// === Module ===

#[test]
fn module_always_strict() {
    let (s, _) = module_sema("var x = 1;");
    let root = &s.scope_table.get(ScopeId::new(0));
    assert!(root.strict, "module scope should be strict");
}

#[test]
fn module_import_binding() {
    let (s, mut atoms) = module_sema("import { x } from 'mod';");
    let x = atoms.intern("x");
    let binding = s.binding_table.as_slice().iter().find(|b| b.name == x);
    assert!(binding.is_some());
    assert_eq!(binding.unwrap().kind, DeclarationKind::Import);
}

// === Slot index assignment ===

#[test]
fn environment_slots_assigned() {
    let (s, mut atoms) =
        sema("function f() { var x = 1; var y = 2; function g() { return x + y; } }");
    let x = atoms.intern("x");
    let y = atoms.intern("y");
    let x_b = s
        .binding_table
        .as_slice()
        .iter()
        .find(|b| b.name == x)
        .unwrap();
    let y_b = s
        .binding_table
        .as_slice()
        .iter()
        .find(|b| b.name == y)
        .unwrap();
    // Both should be EnvironmentSlot with assigned indices
    assert_eq!(x_b.storage_class, StorageClass::EnvironmentSlot);
    assert_eq!(y_b.storage_class, StorageClass::EnvironmentSlot);
    assert!(x_b.slot_index.is_some());
    assert!(y_b.slot_index.is_some());
    // Slots should be distinct
    assert_ne!(x_b.slot_index, y_b.slot_index);
}

// === Complex scenarios ===

#[test]
fn closure_over_loop_variable() {
    let (s, mut atoms) = sema("for (let i = 0; i < 10; i++) { (function() { return i; })(); }");
    let i = atoms.intern("i");
    let i_binding = s.binding_table.as_slice().iter().find(|b| b.name == i);
    assert!(i_binding.is_some());
    assert!(i_binding.unwrap().is_captured);
}

#[test]
fn nested_arrow_captures() {
    let (s, mut atoms) = sema("function f() { var x = 1; var g = () => () => x; }");
    let x = atoms.intern("x");
    let x_binding = s.binding_table.as_slice().iter().find(|b| b.name == x);
    assert!(x_binding.is_some());
    assert!(x_binding.unwrap().is_captured);
}

#[test]
fn class_body_scope() {
    let (s, _) = sema("class C { m() {} static s() {} }");
    assert!(!s.diagnostics.has_errors());
}

#[test]
fn class_private_layout_preserves_source_order_staticness_and_kind() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        sid(),
        "class C { #field; static #staticField; get #value() {} set #value(v) {} #method() {} }",
    );
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors: {:?}",
        parsed.diagnostics.as_slice()
    );

    let sema = analyze_script(&parsed, &atoms);
    assert!(
        !sema.diagnostics.has_errors(),
        "unexpected sema errors: {:?}",
        sema.diagnostics.as_slice()
    );

    let body = match parsed.ast.get_decl(lyng_js_ast::DeclId::new(0)) {
        lyng_js_ast::Decl::Class { body, .. } => *body,
        other => panic!("expected first declaration to be a class, got {other:?}"),
    };
    let layout = sema
        .class_private_layouts
        .get(body)
        .expect("class body should have a private layout record");
    let entries: Vec<_> = layout
        .entries()
        .iter()
        .map(|entry| {
            (
                atoms.resolve(entry.name()).to_owned(),
                entry.is_static(),
                entry.kind(),
            )
        })
        .collect();
    assert_eq!(
        entries,
        vec![
            ("field".to_owned(), false, ClassPrivateElementKind::Field),
            (
                "staticField".to_owned(),
                true,
                ClassPrivateElementKind::Field
            ),
            ("value".to_owned(), false, ClassPrivateElementKind::Getter),
            ("value".to_owned(), false, ClassPrivateElementKind::Setter),
            ("method".to_owned(), false, ClassPrivateElementKind::Method),
        ]
    );
}

#[test]
fn class_private_layout_exists_for_classes_without_private_elements() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        sid(),
        "class C { method() {} static other() {} }",
    );
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors: {:?}",
        parsed.diagnostics.as_slice()
    );

    let sema = analyze_script(&parsed, &atoms);
    assert!(
        !sema.diagnostics.has_errors(),
        "unexpected sema errors: {:?}",
        sema.diagnostics.as_slice()
    );

    let body = match parsed.ast.get_decl(lyng_js_ast::DeclId::new(0)) {
        lyng_js_ast::Decl::Class { body, .. } => *body,
        other => panic!("expected first declaration to be a class, got {other:?}"),
    };
    let layout = sema
        .class_private_layouts
        .get(body)
        .expect("class body should always have a layout record");
    assert!(layout.entries().is_empty());
}

#[test]
fn for_of_let() {
    let (s, _) = sema("for (const [k, v] of entries) {}");
    assert!(!s.diagnostics.has_errors());
}

#[test]
fn using_binding_is_lexical_tdz_binding() {
    let (s, mut atoms) = sema("{ using resource = acquire(); }");
    let resource = atoms.intern("resource");
    let binding = s
        .binding_table
        .as_slice()
        .iter()
        .find(|binding| binding.name == resource)
        .expect("using binding should exist");

    assert_eq!(binding.kind, DeclarationKind::Using);
    assert!(binding.kind.is_lexical());
    assert!(binding.has_tdz);
    assert_eq!(binding.storage_class, StorageClass::EnvironmentSlot);
}

#[test]
fn await_using_binding_is_lexical_tdz_binding() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        sid(),
        "async function f() { await using resource = acquire(); }",
    );
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors: {:?}",
        parsed.diagnostics.as_slice()
    );
    let sema = analyze_script(&parsed, &atoms);
    assert!(
        !sema.diagnostics.has_errors(),
        "unexpected sema errors: {:?}",
        sema.diagnostics.as_slice()
    );
    let resource = atoms.intern("resource");
    let binding = sema
        .binding_table
        .as_slice()
        .iter()
        .find(|binding| binding.name == resource)
        .expect("await using binding should exist");

    assert_eq!(binding.kind, DeclarationKind::AwaitUsing);
    assert!(binding.kind.is_lexical());
    assert!(binding.has_tdz);
    assert_eq!(binding.storage_class, StorageClass::EnvironmentSlot);
}

#[test]
fn unrelated_functions_do_not_share_captures() {
    // outer1/inner1 captures x. outer2/inner2 captures y.
    // inner2 must NOT have x in its captures, and inner1 must NOT have y.
    let (s, mut atoms) = sema(
        "function outer1() { var x = 1; function inner1() { return x; } } \
         function outer2() { var y = 2; function inner2() { return y; } }",
    );
    assert!(!s.diagnostics.has_errors());

    let _x = atoms.intern("x");
    let _y = atoms.intern("y");

    // Find the function sema records by looking at which bindings they capture
    for func in s.function_table.as_slice() {
        let captures: Vec<&str> = func
            .captures
            .iter()
            .map(|&bid| {
                let name = s.binding_table.get(bid).name;
                atoms.resolve(name)
            })
            .collect();

        // No function should capture both x and y
        let has_x = captures.contains(&"x");
        let has_y = captures.contains(&"y");
        assert!(
            !(has_x && has_y),
            "function should not capture both x and y, got: {captures:?}"
        );
    }
}

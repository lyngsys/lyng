use super::*;
use lyng_js_common::WellKnownAtom;

// ===========================================================================
// Modules (import/export)
// ===========================================================================

#[test]
fn parse_import_default() {
    let p = module_ok("import foo from 'bar';");
    let stmts = mbody(&p);
    assert_eq!(stmts.len(), 1);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Import { specifiers, .. } = p.ast.get_decl(*decl) {
            let specs = p.ast.get_import_spec_list(*specifiers);
            assert_eq!(specs.len(), 1);
            assert!(matches!(specs[0], ImportSpecifier::Default { .. }));
        } else {
            panic!("expected import declaration");
        }
    }
}

#[test]
fn parse_import_namespace() {
    let p = module_ok("import * as ns from 'mod';");
    let stmts = mbody(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Import { specifiers, .. } = p.ast.get_decl(*decl) {
            let specs = p.ast.get_import_spec_list(*specifiers);
            assert_eq!(specs.len(), 1);
            assert!(matches!(specs[0], ImportSpecifier::Namespace { .. }));
        }
    }
}

#[test]
fn parse_import_named() {
    let p = module_ok("import { a, b as c } from 'mod';");
    let stmts = mbody(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Import { specifiers, .. } = p.ast.get_decl(*decl) {
            let specs = p.ast.get_import_spec_list(*specifiers);
            assert_eq!(specs.len(), 2);
        }
    }
}

#[test]
fn parse_import_side_effect() {
    let p = module_ok("import 'mod';");
    let stmts = mbody(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Import {
            specifiers,
            attributes,
            ..
        } = p.ast.get_decl(*decl)
        {
            let specs = p.ast.get_import_spec_list(*specifiers);
            assert_eq!(specs.len(), 0);
            assert!(p.ast.get_import_attr_list(*attributes).is_empty());
        }
    }
}

#[test]
fn parse_import_attributes_retained() {
    let p = module_ok("import foo from 'bar' with { type: 'json', mode: 'strict' };");
    let stmts = mbody(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Import { attributes, .. } = p.ast.get_decl(*decl) {
            let attrs = p.ast.get_import_attr_list(*attributes);
            assert_eq!(attrs.len(), 2);
        }
    }
}

#[test]
fn parse_import_attributes_accept_identifier_name_keys() {
    let p = module_ok("import foo from 'bar' with { if: 'json' };");
    let stmts = mbody(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Import { attributes, .. } = p.ast.get_decl(*decl) {
            let attrs = p.ast.get_import_attr_list(*attributes);
            assert_eq!(attrs.len(), 1);
            assert_eq!(attrs[0].key, WellKnownAtom::r#if.id());
        } else {
            panic!("expected import declaration");
        }
    }
}

#[test]
fn parse_export_named() {
    let p = module_ok("const x = 1; export { x };");
    let stmts = mbody(&p);
    assert_eq!(stmts.len(), 2);
}

#[test]
fn parse_export_default_expr() {
    let p = module_ok("export default 42;");
    let stmts = mbody(&p);
    assert_eq!(stmts.len(), 1);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Export { kind, .. } = p.ast.get_decl(*decl) {
            assert!(matches!(kind, ExportKind::Default { .. }));
        }
    }
}

#[test]
fn parse_export_default_function() {
    let p = module_ok("export default function foo() {}");
    let stmts = mbody(&p);
    assert_eq!(stmts.len(), 1);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Export { kind, .. } = p.ast.get_decl(*decl) {
            assert!(matches!(
                kind,
                ExportKind::Default {
                    declaration: ExportDefaultDecl::Function(_)
                }
            ));
        }
    }
}

#[test]
fn parse_export_all() {
    let p = module_ok("export * from 'mod' with { type: 'json' };");
    let stmts = mbody(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Export { kind, .. } = p.ast.get_decl(*decl) {
            match kind {
                ExportKind::All { attributes, .. } => {
                    assert_eq!(p.ast.get_import_attr_list(*attributes).len(), 1);
                }
                _ => panic!("expected export-all declaration"),
            }
        }
    }
}

#[test]
fn parse_export_named_reexport_attributes_retained() {
    let p = module_ok("export { foo as bar } from 'mod' with { type: 'json' };");
    let stmts = mbody(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Export { kind, .. } = p.ast.get_decl(*decl) {
            match kind {
                ExportKind::Named {
                    source, attributes, ..
                } => {
                    assert!(source.is_some());
                    assert_eq!(p.ast.get_import_attr_list(*attributes).len(), 1);
                }
                _ => panic!("expected named export"),
            }
        }
    }
}

#[test]
fn parse_export_declaration() {
    let p = module_ok("export const x = 1;");
    let stmts = mbody(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Export { kind, .. } = p.ast.get_decl(*decl) {
            assert!(matches!(kind, ExportKind::Declaration { .. }));
        }
    }
}

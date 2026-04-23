use super::*;
use lyng_js_ast::*;
use lyng_js_common::{AtomId, AtomTable, DiagnosticList, SourceId, Span, WellKnownAtom};

fn span() -> Span {
    Span::from_offsets(SourceId::new(0), 0, 10)
}

fn span_at(start: u32, end: u32) -> Span {
    Span::from_offsets(SourceId::new(0), start, end)
}

// Helper: build a ParsedScript from a body of statements.
fn make_script(build: impl FnOnce(&mut Ast) -> Vec<StmtId>) -> ParsedScript {
    let mut ast = Ast::new();
    let stmts = build(&mut ast);
    let body = ast.alloc_stmt_list(&stmts);
    let root = ast.alloc_script(Script { span: span(), body });
    ParsedScript {
        ast,
        root,
        source_text: "".into(),
        diagnostics: DiagnosticList::new(),
        strict: false,
    }
}

fn make_strict_script(build: impl FnOnce(&mut Ast) -> Vec<StmtId>) -> ParsedScript {
    let mut ast = Ast::new();
    let stmts = build(&mut ast);
    let body = ast.alloc_stmt_list(&stmts);
    let root = ast.alloc_script(Script { span: span(), body });
    ParsedScript {
        ast,
        root,
        source_text: "".into(),
        diagnostics: DiagnosticList::new(),
        strict: true,
    }
}

fn make_module(build: impl FnOnce(&mut Ast) -> Vec<StmtId>) -> ParsedModule {
    let mut ast = Ast::new();
    let stmts = build(&mut ast);
    let body = ast.alloc_stmt_list(&stmts);
    let root = ast.alloc_module(Module { span: span(), body });
    ParsedModule {
        ast,
        root,
        source_text: "".into(),
        diagnostics: DiagnosticList::new(),
    }
}

fn atoms() -> AtomTable {
    AtomTable::new()
}

// ===================================================================
// Scope building tests
// ===================================================================

#[test]
fn empty_script_creates_global_scope() {
    let parsed = make_script(|_ast| vec![]);
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert_eq!(result.scope_table.len(), 1);
    assert_eq!(
        result.scope_table.get(ScopeId::new(0)).kind,
        ScopeKind::Global
    );
    assert!(result.scope_table.get(ScopeId::new(0)).parent.is_none());
    assert!(!result.diagnostics.has_errors());
}

#[test]
fn block_creates_scope() {
    // { }
    let parsed = make_script(|ast| {
        let body = ast.alloc_stmt_list(&[]);
        let block = ast.alloc_stmt(Stmt::Block { span: span(), body });
        vec![block]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    // Global + Block
    assert_eq!(result.scope_table.len(), 2);
    assert_eq!(
        result.scope_table.get(ScopeId::new(1)).kind,
        ScopeKind::Block
    );
    assert_eq!(
        result.scope_table.get(ScopeId::new(1)).parent,
        Some(ScopeId::new(0))
    );
}

#[test]
fn function_creates_scope() {
    // function f() {}
    let parsed = make_script(|ast| {
        let params_list = ast.alloc_pattern_list(&[]);
        let body_list = ast.alloc_stmt_list(&[]);
        let func = ast.alloc_function(Function {
            span: span(),
            name: Some(AtomId::from_raw(100)),
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span(),
                params: params_list,
                rest: None,
            },
            body: body_list,
            expression_body: None,
        });
        let decl = ast.alloc_decl(Decl::Function {
            span: span(),
            function: func,
        });
        let stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    // Global scope + Function scope
    assert!(result.scope_table.len() >= 2);
    // The function scope should be a child of global
    let global = result.scope_table.get(ScopeId::new(0));
    assert!(!global.children.is_empty());
}

#[test]
fn nested_scopes_have_correct_parents() {
    // { { } }
    let parsed = make_script(|ast| {
        let inner_body = ast.alloc_stmt_list(&[]);
        let inner_block = ast.alloc_stmt(Stmt::Block {
            span: span(),
            body: inner_body,
        });
        let outer_stmts = ast.alloc_stmt_list(&[inner_block]);
        let outer_block = ast.alloc_stmt(Stmt::Block {
            span: span(),
            body: outer_stmts,
        });
        vec![outer_block]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    // Global(0) -> OuterBlock(1) -> InnerBlock(2)
    assert_eq!(result.scope_table.len(), 3);
    assert_eq!(
        result.scope_table.get(ScopeId::new(2)).parent,
        Some(ScopeId::new(1))
    );
    assert_eq!(
        result.scope_table.get(ScopeId::new(1)).parent,
        Some(ScopeId::new(0))
    );
}

// ===================================================================
// Binding analysis tests
// ===================================================================

#[test]
fn var_binding_hoists_to_global() {
    // var x = 1;
    let x_atom = AtomId::from_raw(100);
    let parsed = make_script(|ast| {
        let pat = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: x_atom,
        });
        let init = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(1),
            syntax: NumericLiteralSyntax::Normal,
        });
        let declarator = VariableDeclarator {
            span: span(),
            id: pat,
            init: Some(init),
        };
        let declarators = ast.alloc_var_declarator_list(&[declarator]);
        let decl = ast.alloc_decl(Decl::Variable {
            span: span(),
            kind: VariableKind::Var,
            declarators,
        });
        let stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    // Should have one binding in the global scope.
    let global = result.scope_table.get(ScopeId::new(0));
    assert_eq!(global.bindings.len(), 1);
    let binding = result.binding_table.get(global.bindings[0]);
    assert_eq!(binding.name, x_atom);
    assert_eq!(binding.kind, DeclarationKind::Var);
    assert_eq!(binding.storage_class, StorageClass::GlobalName);
}

#[test]
fn let_binding_is_block_scoped() {
    // { let x = 1; }
    let x_atom = AtomId::from_raw(100);
    let parsed = make_script(|ast| {
        let pat = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: x_atom,
        });
        let init = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(1),
            syntax: NumericLiteralSyntax::Normal,
        });
        let declarator = VariableDeclarator {
            span: span(),
            id: pat,
            init: Some(init),
        };
        let declarators = ast.alloc_var_declarator_list(&[declarator]);
        let decl = ast.alloc_decl(Decl::Variable {
            span: span(),
            kind: VariableKind::Let,
            declarators,
        });
        let decl_stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        let body = ast.alloc_stmt_list(&[decl_stmt]);
        let block = ast.alloc_stmt(Stmt::Block { span: span(), body });
        vec![block]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    // Global scope should have no bindings.
    let global = result.scope_table.get(ScopeId::new(0));
    assert_eq!(global.bindings.len(), 0);
    // Block scope should have the let binding.
    let block = result.scope_table.get(ScopeId::new(1));
    assert_eq!(block.bindings.len(), 1);
    let binding = result.binding_table.get(block.bindings[0]);
    assert_eq!(binding.name, x_atom);
    assert_eq!(binding.kind, DeclarationKind::Let);
    assert!(binding.has_tdz);
}

#[test]
fn const_binding_has_tdz() {
    // { const x = 1; }
    let x_atom = AtomId::from_raw(100);
    let parsed = make_script(|ast| {
        let pat = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: x_atom,
        });
        let init = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(1),
            syntax: NumericLiteralSyntax::Normal,
        });
        let declarator = VariableDeclarator {
            span: span(),
            id: pat,
            init: Some(init),
        };
        let declarators = ast.alloc_var_declarator_list(&[declarator]);
        let decl = ast.alloc_decl(Decl::Variable {
            span: span(),
            kind: VariableKind::Const,
            declarators,
        });
        let decl_stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        let body = ast.alloc_stmt_list(&[decl_stmt]);
        let block = ast.alloc_stmt(Stmt::Block { span: span(), body });
        vec![block]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    let block = result.scope_table.get(ScopeId::new(1));
    assert_eq!(block.bindings.len(), 1);
    let binding = result.binding_table.get(block.bindings[0]);
    assert_eq!(binding.kind, DeclarationKind::Const);
    assert!(binding.has_tdz);
}

#[test]
fn function_declaration_hoisted() {
    // function f() {}
    let f_atom = AtomId::from_raw(100);
    let parsed = make_script(|ast| {
        let params_list = ast.alloc_pattern_list(&[]);
        let body_list = ast.alloc_stmt_list(&[]);
        let func = ast.alloc_function(Function {
            span: span(),
            name: Some(f_atom),
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span(),
                params: params_list,
                rest: None,
            },
            body: body_list,
            expression_body: None,
        });
        let decl = ast.alloc_decl(Decl::Function {
            span: span(),
            function: func,
        });
        let stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    let global = result.scope_table.get(ScopeId::new(0));
    assert!(global.bindings.iter().any(|&bid| {
        let b = result.binding_table.get(bid);
        b.name == f_atom && b.kind == DeclarationKind::Function
    }));
}

#[test]
fn var_hoists_out_of_block() {
    // { var x = 1; }
    let x_atom = AtomId::from_raw(100);
    let parsed = make_script(|ast| {
        let pat = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: x_atom,
        });
        let init = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(1),
            syntax: NumericLiteralSyntax::Normal,
        });
        let declarator = VariableDeclarator {
            span: span(),
            id: pat,
            init: Some(init),
        };
        let declarators = ast.alloc_var_declarator_list(&[declarator]);
        let decl = ast.alloc_decl(Decl::Variable {
            span: span(),
            kind: VariableKind::Var,
            declarators,
        });
        let decl_stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        let body = ast.alloc_stmt_list(&[decl_stmt]);
        let block = ast.alloc_stmt(Stmt::Block { span: span(), body });
        vec![block]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    // Var should be in global scope, not block scope.
    let global = result.scope_table.get(ScopeId::new(0));
    assert_eq!(global.bindings.len(), 1);
    let binding = result.binding_table.get(global.bindings[0]);
    assert_eq!(binding.name, x_atom);
    assert_eq!(binding.kind, DeclarationKind::Var);
}

// ===================================================================
// Capture analysis tests
// ===================================================================

#[test]
fn inner_function_captures_outer_variable() {
    // var x = 1;
    // (function() { x; })
    let x_atom = AtomId::from_raw(100);
    let parsed = make_script(|ast| {
        // var x = 1;
        let pat = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: x_atom,
        });
        let init = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(1),
            syntax: NumericLiteralSyntax::Normal,
        });
        let declarator = VariableDeclarator {
            span: span(),
            id: pat,
            init: Some(init),
        };
        let declarators = ast.alloc_var_declarator_list(&[declarator]);
        let var_decl = ast.alloc_decl(Decl::Variable {
            span: span(),
            kind: VariableKind::Var,
            declarators,
        });
        let var_stmt = ast.alloc_stmt(Stmt::Declaration {
            span: span(),
            decl: var_decl,
        });

        // function() { x; }
        let x_ref = ast.alloc_expr(Expr::Identifier {
            span: span(),
            name: x_atom,
        });
        let x_stmt = ast.alloc_stmt(Stmt::Expression {
            span: span(),
            expression: x_ref,
        });
        let func_body = ast.alloc_stmt_list(&[x_stmt]);
        let params_list = ast.alloc_pattern_list(&[]);
        let func = ast.alloc_function(Function {
            span: span(),
            name: None,
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span(),
                params: params_list,
                rest: None,
            },
            body: func_body,
            expression_body: None,
        });
        let func_expr = ast.alloc_expr(Expr::FunctionExpression {
            span: span(),
            function: func,
        });
        let expr_stmt = ast.alloc_stmt(Stmt::Expression {
            span: span(),
            expression: func_expr,
        });

        vec![var_stmt, expr_stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);

    // The var x binding should be marked as captured.
    let global = result.scope_table.get(ScopeId::new(0));
    let x_binding_id = global.bindings[0];
    let x_binding = result.binding_table.get(x_binding_id);
    assert!(x_binding.is_captured);
    assert_eq!(x_binding.storage_class, StorageClass::EnvironmentSlot);
}

// ===================================================================
// Early error tests
// ===================================================================

#[test]
fn duplicate_let_bindings_error() {
    // { let x; let x; }
    let x_atom = AtomId::from_raw(100);
    let parsed = make_script(|ast| {
        let pat1 = ast.alloc_pattern(Pattern::Identifier {
            span: span_at(0, 5),
            name: x_atom,
        });
        let decl1_declarator = VariableDeclarator {
            span: span_at(0, 5),
            id: pat1,
            init: None,
        };
        let declarators1 = ast.alloc_var_declarator_list(&[decl1_declarator]);
        let decl1 = ast.alloc_decl(Decl::Variable {
            span: span_at(0, 5),
            kind: VariableKind::Let,
            declarators: declarators1,
        });
        let stmt1 = ast.alloc_stmt(Stmt::Declaration {
            span: span_at(0, 5),
            decl: decl1,
        });

        let pat2 = ast.alloc_pattern(Pattern::Identifier {
            span: span_at(6, 11),
            name: x_atom,
        });
        let decl2_declarator = VariableDeclarator {
            span: span_at(6, 11),
            id: pat2,
            init: None,
        };
        let declarators2 = ast.alloc_var_declarator_list(&[decl2_declarator]);
        let decl2 = ast.alloc_decl(Decl::Variable {
            span: span_at(6, 11),
            kind: VariableKind::Let,
            declarators: declarators2,
        });
        let stmt2 = ast.alloc_stmt(Stmt::Declaration {
            span: span_at(6, 11),
            decl: decl2,
        });

        let body = ast.alloc_stmt_list(&[stmt1, stmt2]);
        let block = ast.alloc_stmt(Stmt::Block { span: span(), body });
        vec![block]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(result.diagnostics.has_errors());
    let errors = result.diagnostics.as_slice();
    assert!(errors
        .iter()
        .any(|d| d.message.contains("duplicate lexical")));
}

#[test]
fn sloppy_duplicate_block_functions_are_allowed() {
    let f_atom = AtomId::from_raw(100);
    let parsed = make_script(|ast| {
        let params = ast.alloc_pattern_list(&[]);
        let body = ast.alloc_stmt_list(&[]);
        let func1 = ast.alloc_function(Function {
            span: span_at(0, 8),
            name: Some(f_atom),
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span_at(0, 8),
                params,
                rest: None,
            },
            body,
            expression_body: None,
        });
        let decl1 = ast.alloc_decl(Decl::Function {
            span: span_at(0, 8),
            function: func1,
        });
        let stmt1 = ast.alloc_stmt(Stmt::Declaration {
            span: span_at(0, 8),
            decl: decl1,
        });

        let params = ast.alloc_pattern_list(&[]);
        let body = ast.alloc_stmt_list(&[]);
        let func2 = ast.alloc_function(Function {
            span: span_at(9, 17),
            name: Some(f_atom),
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span_at(9, 17),
                params,
                rest: None,
            },
            body,
            expression_body: None,
        });
        let decl2 = ast.alloc_decl(Decl::Function {
            span: span_at(9, 17),
            function: func2,
        });
        let stmt2 = ast.alloc_stmt(Stmt::Declaration {
            span: span_at(9, 17),
            decl: decl2,
        });

        let body = ast.alloc_stmt_list(&[stmt1, stmt2]);
        let block = ast.alloc_stmt(Stmt::Block { span: span(), body });
        vec![block]
    });

    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(!result.diagnostics.has_errors());
}

#[test]
fn sloppy_block_async_function_duplicates_still_error() {
    let f_atom = AtomId::from_raw(100);
    let parsed = make_script(|ast| {
        let params = ast.alloc_pattern_list(&[]);
        let body = ast.alloc_stmt_list(&[]);
        let async_func = ast.alloc_function(Function {
            span: span_at(0, 14),
            name: Some(f_atom),
            kind: FunctionKind::Async,
            params: FormalParameters {
                span: span_at(0, 14),
                params,
                rest: None,
            },
            body,
            expression_body: None,
        });
        let async_decl = ast.alloc_decl(Decl::Function {
            span: span_at(0, 14),
            function: async_func,
        });
        let async_stmt = ast.alloc_stmt(Stmt::Declaration {
            span: span_at(0, 14),
            decl: async_decl,
        });

        let params = ast.alloc_pattern_list(&[]);
        let body = ast.alloc_stmt_list(&[]);
        let normal_func = ast.alloc_function(Function {
            span: span_at(15, 28),
            name: Some(f_atom),
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span_at(15, 28),
                params,
                rest: None,
            },
            body,
            expression_body: None,
        });
        let normal_decl = ast.alloc_decl(Decl::Function {
            span: span_at(15, 28),
            function: normal_func,
        });
        let normal_stmt = ast.alloc_stmt(Stmt::Declaration {
            span: span_at(15, 28),
            decl: normal_decl,
        });

        let body = ast.alloc_stmt_list(&[async_stmt, normal_stmt]);
        let block = ast.alloc_stmt(Stmt::Block { span: span(), body });
        vec![block]
    });

    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(result.diagnostics.has_errors());
    assert!(result
        .diagnostics
        .as_slice()
        .iter()
        .any(|d| d.message.contains("duplicate lexical")));
}

#[test]
fn sloppy_switch_generator_function_duplicates_still_error() {
    let f_atom = AtomId::from_raw(100);
    let parsed = make_script(|ast| {
        let params = ast.alloc_pattern_list(&[]);
        let body = ast.alloc_stmt_list(&[]);
        let normal_func = ast.alloc_function(Function {
            span: span_at(0, 12),
            name: Some(f_atom),
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span_at(0, 12),
                params,
                rest: None,
            },
            body,
            expression_body: None,
        });
        let normal_decl = ast.alloc_decl(Decl::Function {
            span: span_at(0, 12),
            function: normal_func,
        });
        let normal_stmt = ast.alloc_stmt(Stmt::Declaration {
            span: span_at(0, 12),
            decl: normal_decl,
        });

        let params = ast.alloc_pattern_list(&[]);
        let body = ast.alloc_stmt_list(&[]);
        let generator_func = ast.alloc_function(Function {
            span: span_at(13, 28),
            name: Some(f_atom),
            kind: FunctionKind::Generator,
            params: FormalParameters {
                span: span_at(13, 28),
                params,
                rest: None,
            },
            body,
            expression_body: None,
        });
        let generator_decl = ast.alloc_decl(Decl::Function {
            span: span_at(13, 28),
            function: generator_func,
        });
        let generator_stmt = ast.alloc_stmt(Stmt::Declaration {
            span: span_at(13, 28),
            decl: generator_decl,
        });

        let discriminant = ast.alloc_expr(Expr::NumericLiteral {
            span: span_at(29, 30),
            value: NumericLiteral::Int32(0),
            syntax: NumericLiteralSyntax::Normal,
        });
        let case_one_body = ast.alloc_stmt_list(&[normal_stmt]);
        let default_body = ast.alloc_stmt_list(&[generator_stmt]);
        let cases = ast.alloc_switch_case_list(&[
            SwitchCase {
                span: span_at(31, 40),
                test: Some(discriminant),
                consequent: case_one_body,
            },
            SwitchCase {
                span: span_at(41, 55),
                test: None,
                consequent: default_body,
            },
        ]);
        let switch_stmt = ast.alloc_stmt(Stmt::Switch {
            span: span(),
            discriminant,
            cases,
        });
        vec![switch_stmt]
    });

    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(result.diagnostics.has_errors());
    assert!(result
        .diagnostics
        .as_slice()
        .iter()
        .any(|d| d.message.contains("duplicate lexical")));
}

#[test]
fn strict_duplicate_block_functions_still_error() {
    let f_atom = AtomId::from_raw(100);
    let parsed = make_strict_script(|ast| {
        let params = ast.alloc_pattern_list(&[]);
        let body = ast.alloc_stmt_list(&[]);
        let func1 = ast.alloc_function(Function {
            span: span_at(0, 8),
            name: Some(f_atom),
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span_at(0, 8),
                params,
                rest: None,
            },
            body,
            expression_body: None,
        });
        let decl1 = ast.alloc_decl(Decl::Function {
            span: span_at(0, 8),
            function: func1,
        });
        let stmt1 = ast.alloc_stmt(Stmt::Declaration {
            span: span_at(0, 8),
            decl: decl1,
        });

        let params = ast.alloc_pattern_list(&[]);
        let body = ast.alloc_stmt_list(&[]);
        let func2 = ast.alloc_function(Function {
            span: span_at(9, 17),
            name: Some(f_atom),
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span_at(9, 17),
                params,
                rest: None,
            },
            body,
            expression_body: None,
        });
        let decl2 = ast.alloc_decl(Decl::Function {
            span: span_at(9, 17),
            function: func2,
        });
        let stmt2 = ast.alloc_stmt(Stmt::Declaration {
            span: span_at(9, 17),
            decl: decl2,
        });

        let body = ast.alloc_stmt_list(&[stmt1, stmt2]);
        let block = ast.alloc_stmt(Stmt::Block { span: span(), body });
        vec![block]
    });

    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(result.diagnostics.has_errors());
    assert!(result
        .diagnostics
        .as_slice()
        .iter()
        .any(|d| d.message.contains("duplicate lexical")));
}

#[test]
fn break_outside_loop_error() {
    // break;
    let parsed = make_script(|ast| {
        let stmt = ast.alloc_stmt(Stmt::Break {
            span: span(),
            label: None,
        });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(result.diagnostics.has_errors());
    let errors = result.diagnostics.as_slice();
    assert!(errors
        .iter()
        .any(|d| d.message.contains("outside of loop or switch")));
}

#[test]
fn continue_outside_loop_error() {
    // continue;
    let parsed = make_script(|ast| {
        let stmt = ast.alloc_stmt(Stmt::Continue {
            span: span(),
            label: None,
        });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(result.diagnostics.has_errors());
    let errors = result.diagnostics.as_slice();
    assert!(errors.iter().any(|d| d.message.contains("outside of loop")));
}

#[test]
fn return_outside_function_error() {
    // return;
    let parsed = make_script(|ast| {
        let stmt = ast.alloc_stmt(Stmt::Return {
            span: span(),
            argument: None,
        });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(result.diagnostics.has_errors());
    let errors = result.diagnostics.as_slice();
    assert!(errors
        .iter()
        .any(|d| d.message.contains("outside of function")));
}

#[test]
fn break_inside_loop_ok() {
    // while (true) { break; }
    let parsed = make_script(|ast| {
        let test = ast.alloc_expr(Expr::BooleanLiteral {
            span: span(),
            value: true,
        });
        let brk = ast.alloc_stmt(Stmt::Break {
            span: span(),
            label: None,
        });
        let body_stmts = ast.alloc_stmt_list(&[brk]);
        let body = ast.alloc_stmt(Stmt::Block {
            span: span(),
            body: body_stmts,
        });
        let while_stmt = ast.alloc_stmt(Stmt::While {
            span: span(),
            test,
            body,
        });
        vec![while_stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(!result.diagnostics.has_errors());
}

#[test]
fn return_inside_function_ok() {
    // function f() { return 1; }
    let parsed = make_script(|ast| {
        let ret_val = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(1),
            syntax: NumericLiteralSyntax::Normal,
        });
        let ret_stmt = ast.alloc_stmt(Stmt::Return {
            span: span(),
            argument: Some(ret_val),
        });
        let body = ast.alloc_stmt_list(&[ret_stmt]);
        let params_list = ast.alloc_pattern_list(&[]);
        let func = ast.alloc_function(Function {
            span: span(),
            name: Some(AtomId::from_raw(100)),
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span(),
                params: params_list,
                rest: None,
            },
            body,
            expression_body: None,
        });
        let decl = ast.alloc_decl(Decl::Function {
            span: span(),
            function: func,
        });
        let stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(!result.diagnostics.has_errors());
}

// ===================================================================
// Strict mode tests
// ===================================================================

#[test]
fn strict_mode_eval_binding_error() {
    // "use strict"; var eval = 1;
    let parsed = make_strict_script(|ast| {
        let pat = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: WellKnownAtom::eval.id(),
        });
        let init = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(1),
            syntax: NumericLiteralSyntax::Normal,
        });
        let declarator = VariableDeclarator {
            span: span(),
            id: pat,
            init: Some(init),
        };
        let declarators = ast.alloc_var_declarator_list(&[declarator]);
        let decl = ast.alloc_decl(Decl::Variable {
            span: span(),
            kind: VariableKind::Var,
            declarators,
        });
        let stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(result.diagnostics.has_errors());
    let errors = result.diagnostics.as_slice();
    assert!(errors
        .iter()
        .any(|d| d.message.contains("'eval' cannot be used")));
}

#[test]
fn strict_mode_arguments_binding_error() {
    let parsed = make_strict_script(|ast| {
        let pat = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: WellKnownAtom::arguments.id(),
        });
        let init = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(1),
            syntax: NumericLiteralSyntax::Normal,
        });
        let declarator = VariableDeclarator {
            span: span(),
            id: pat,
            init: Some(init),
        };
        let declarators = ast.alloc_var_declarator_list(&[declarator]);
        let decl = ast.alloc_decl(Decl::Variable {
            span: span(),
            kind: VariableKind::Var,
            declarators,
        });
        let stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(result.diagnostics.has_errors());
    let errors = result.diagnostics.as_slice();
    assert!(errors
        .iter()
        .any(|d| d.message.contains("'arguments' cannot be used")));
}

#[test]
fn non_strict_eval_binding_ok() {
    let parsed = make_script(|ast| {
        let pat = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: WellKnownAtom::eval.id(),
        });
        let init = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(1),
            syntax: NumericLiteralSyntax::Normal,
        });
        let declarator = VariableDeclarator {
            span: span(),
            id: pat,
            init: Some(init),
        };
        let declarators = ast.alloc_var_declarator_list(&[declarator]);
        let decl = ast.alloc_decl(Decl::Variable {
            span: span(),
            kind: VariableKind::Var,
            declarators,
        });
        let stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(!result.diagnostics.has_errors());
}

// ===================================================================
// Storage class tests
// ===================================================================

#[test]
fn uncaptured_local_is_frame_local() {
    // function f() { let x = 1; }
    let x_atom = AtomId::from_raw(100);
    let f_atom = AtomId::from_raw(101);
    let parsed = make_script(|ast| {
        let pat = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: x_atom,
        });
        let init = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(1),
            syntax: NumericLiteralSyntax::Normal,
        });
        let declarator = VariableDeclarator {
            span: span(),
            id: pat,
            init: Some(init),
        };
        let declarators = ast.alloc_var_declarator_list(&[declarator]);
        let let_decl = ast.alloc_decl(Decl::Variable {
            span: span(),
            kind: VariableKind::Let,
            declarators,
        });
        let let_stmt = ast.alloc_stmt(Stmt::Declaration {
            span: span(),
            decl: let_decl,
        });
        let body = ast.alloc_stmt_list(&[let_stmt]);
        let params_list = ast.alloc_pattern_list(&[]);
        let func = ast.alloc_function(Function {
            span: span(),
            name: Some(f_atom),
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span(),
                params: params_list,
                rest: None,
            },
            body,
            expression_body: None,
        });
        let decl = ast.alloc_decl(Decl::Function {
            span: span(),
            function: func,
        });
        let stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    // Find the let x binding
    let x_binding = result
        .binding_table
        .as_slice()
        .iter()
        .find(|b| b.name == x_atom)
        .unwrap();
    assert_eq!(x_binding.storage_class, StorageClass::FrameLocal);
    assert!(!x_binding.is_captured);
}

#[test]
fn captured_variable_is_environment_slot() {
    // var x = 1; (function() { x; })
    let x_atom = AtomId::from_raw(100);
    let parsed = make_script(|ast| {
        let pat = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: x_atom,
        });
        let init = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(1),
            syntax: NumericLiteralSyntax::Normal,
        });
        let declarator = VariableDeclarator {
            span: span(),
            id: pat,
            init: Some(init),
        };
        let declarators = ast.alloc_var_declarator_list(&[declarator]);
        let var_decl = ast.alloc_decl(Decl::Variable {
            span: span(),
            kind: VariableKind::Var,
            declarators,
        });
        let var_stmt = ast.alloc_stmt(Stmt::Declaration {
            span: span(),
            decl: var_decl,
        });

        let x_ref = ast.alloc_expr(Expr::Identifier {
            span: span(),
            name: x_atom,
        });
        let x_stmt = ast.alloc_stmt(Stmt::Expression {
            span: span(),
            expression: x_ref,
        });
        let func_body = ast.alloc_stmt_list(&[x_stmt]);
        let params_list = ast.alloc_pattern_list(&[]);
        let func = ast.alloc_function(Function {
            span: span(),
            name: None,
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span(),
                params: params_list,
                rest: None,
            },
            body: func_body,
            expression_body: None,
        });
        let func_expr = ast.alloc_expr(Expr::FunctionExpression {
            span: span(),
            function: func,
        });
        let expr_stmt = ast.alloc_stmt(Stmt::Expression {
            span: span(),
            expression: func_expr,
        });

        vec![var_stmt, expr_stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    let x_binding = result
        .binding_table
        .as_slice()
        .iter()
        .find(|b| b.name == x_atom && b.kind == DeclarationKind::Var)
        .unwrap();
    assert!(x_binding.is_captured);
    assert_eq!(x_binding.storage_class, StorageClass::EnvironmentSlot);
}

#[test]
fn global_var_is_global_name() {
    // var x;
    let x_atom = AtomId::from_raw(100);
    let parsed = make_script(|ast| {
        let pat = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: x_atom,
        });
        let declarator = VariableDeclarator {
            span: span(),
            id: pat,
            init: None,
        };
        let declarators = ast.alloc_var_declarator_list(&[declarator]);
        let decl = ast.alloc_decl(Decl::Variable {
            span: span(),
            kind: VariableKind::Var,
            declarators,
        });
        let stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    let x_binding = result
        .binding_table
        .as_slice()
        .iter()
        .find(|b| b.name == x_atom)
        .unwrap();
    assert_eq!(x_binding.storage_class, StorageClass::GlobalName);
}

#[test]
fn global_lexical_bindings_use_environment_slots() {
    let x_atom = AtomId::from_raw(101);
    let parsed = make_script(|ast| {
        let pat = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: x_atom,
        });
        let declarator = VariableDeclarator {
            span: span(),
            id: pat,
            init: None,
        };
        let declarators = ast.alloc_var_declarator_list(&[declarator]);
        let decl = ast.alloc_decl(Decl::Variable {
            span: span(),
            kind: VariableKind::Let,
            declarators,
        });
        let stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    let x_binding = result
        .binding_table
        .as_slice()
        .iter()
        .find(|b| b.name == x_atom)
        .unwrap();
    assert_eq!(x_binding.storage_class, StorageClass::EnvironmentSlot);
    assert_eq!(x_binding.slot_index, Some(0));
}

// ===================================================================
// Parameter scope tests
// ===================================================================

#[test]
fn non_simple_parameters_create_distinct_scope() {
    // function f({x}) { }
    let f_atom = AtomId::from_raw(100);
    let x_atom = AtomId::from_raw(101);
    let parsed = make_script(|ast| {
        let key = ast.alloc_expr(Expr::Identifier {
            span: span(),
            name: x_atom,
        });
        let val = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: x_atom,
        });
        let prop = ObjectPatternProperty {
            span: span(),
            key,
            value: val,
            computed: false,
            shorthand: true,
        };
        let props = ast.alloc_obj_pattern_prop_list(&[prop]);
        let obj_pat = ast.alloc_pattern(Pattern::Object {
            span: span(),
            properties: props,
            rest: None,
        });
        let params_list = ast.alloc_pattern_list(&[obj_pat]);
        let body = ast.alloc_stmt_list(&[]);
        let func = ast.alloc_function(Function {
            span: span(),
            name: Some(f_atom),
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span(),
                params: params_list,
                rest: None,
            },
            body,
            expression_body: None,
        });
        let decl = ast.alloc_decl(Decl::Function {
            span: span(),
            function: func,
        });
        let stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    // The function should have a distinct param_scope.
    assert!(!result.function_table.is_empty());
    let func = result.function_table.get(FunctionSemaId::new(0));
    assert!(func.param_scope.is_some());
}

#[test]
fn simple_parameters_no_distinct_scope() {
    // function f(x) { }
    let f_atom = AtomId::from_raw(100);
    let x_atom = AtomId::from_raw(101);
    let parsed = make_script(|ast| {
        let param = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: x_atom,
        });
        let params_list = ast.alloc_pattern_list(&[param]);
        let body = ast.alloc_stmt_list(&[]);
        let func = ast.alloc_function(Function {
            span: span(),
            name: Some(f_atom),
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span(),
                params: params_list,
                rest: None,
            },
            body,
            expression_body: None,
        });
        let decl = ast.alloc_decl(Decl::Function {
            span: span(),
            function: func,
        });
        let stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    let func = result.function_table.get(FunctionSemaId::new(0));
    assert!(func.param_scope.is_none());
}

// ===================================================================
// Use-site resolution tests
// ===================================================================

#[test]
fn identifier_resolves_to_nearest_binding() {
    // let x = 1; { let x = 2; x; }
    let x_atom = AtomId::from_raw(100);
    let parsed = make_script(|ast| {
        let pat1 = ast.alloc_pattern(Pattern::Identifier {
            span: span_at(0, 5),
            name: x_atom,
        });
        let init1 = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(1),
            syntax: NumericLiteralSyntax::Normal,
        });
        let decl1_d = VariableDeclarator {
            span: span_at(0, 5),
            id: pat1,
            init: Some(init1),
        };
        let declarators1 = ast.alloc_var_declarator_list(&[decl1_d]);
        let decl1 = ast.alloc_decl(Decl::Variable {
            span: span_at(0, 5),
            kind: VariableKind::Let,
            declarators: declarators1,
        });
        let stmt1 = ast.alloc_stmt(Stmt::Declaration {
            span: span_at(0, 5),
            decl: decl1,
        });

        // Inner block: let x = 2; x;
        let pat2 = ast.alloc_pattern(Pattern::Identifier {
            span: span_at(10, 15),
            name: x_atom,
        });
        let init2 = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(2),
            syntax: NumericLiteralSyntax::Normal,
        });
        let decl2_d = VariableDeclarator {
            span: span_at(10, 15),
            id: pat2,
            init: Some(init2),
        };
        let declarators2 = ast.alloc_var_declarator_list(&[decl2_d]);
        let decl2 = ast.alloc_decl(Decl::Variable {
            span: span_at(10, 15),
            kind: VariableKind::Let,
            declarators: declarators2,
        });
        let let_stmt2 = ast.alloc_stmt(Stmt::Declaration {
            span: span_at(10, 15),
            decl: decl2,
        });

        let x_ref = ast.alloc_expr(Expr::Identifier {
            span: span_at(20, 21),
            name: x_atom,
        });
        let x_stmt = ast.alloc_stmt(Stmt::Expression {
            span: span_at(20, 21),
            expression: x_ref,
        });

        let block_body = ast.alloc_stmt_list(&[let_stmt2, x_stmt]);
        let block = ast.alloc_stmt(Stmt::Block {
            span: span_at(9, 22),
            body: block_body,
        });

        vec![stmt1, block]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);

    // There should be use sites; the x reference should resolve to the inner let x.
    let x_uses: Vec<_> = result
        .use_sites
        .as_slice()
        .iter()
        .filter(|u| u.name == x_atom)
        .collect();
    assert!(!x_uses.is_empty());

    // The last use should resolve locally to the inner binding.
    let last_use = x_uses.last().unwrap();
    assert_eq!(last_use.resolution_kind, ResolutionKind::Local);
    assert!(last_use.resolved_binding.is_some());

    // The inner binding is in the block scope (ScopeId 1).
    let inner_binding_id = last_use.resolved_binding.unwrap();
    let inner_binding = result.binding_table.get(inner_binding_id);
    assert_eq!(inner_binding.scope, ScopeId::new(1));
}

// ===================================================================
// Module tests
// ===================================================================

#[test]
fn module_is_always_strict() {
    let parsed = make_module(|_ast| vec![]);
    let atoms = atoms();
    let result = analyze_module(&parsed, &atoms);
    assert_eq!(
        result.scope_table.get(ScopeId::new(0)).kind,
        ScopeKind::Module
    );
    assert!(result.scope_table.get(ScopeId::new(0)).strict);
}

#[test]
fn import_bindings_are_immutable() {
    // import { x } from "mod";
    let x_atom = AtomId::from_raw(100);
    let parsed = make_module(|ast| {
        let source = ast.literals_mut().alloc_string("mod");
        let spec = ImportSpecifier::Named {
            span: span(),
            imported: x_atom,
            local: x_atom,
        };
        let specs = ast.alloc_import_spec_list(&[spec]);
        let attributes = ast.alloc_import_attr_list(&[]);
        let decl = ast.alloc_decl(Decl::Import {
            span: span(),
            specifiers: specs,
            source,
            attributes,
        });
        let stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_module(&parsed, &atoms);
    let module_scope = result.scope_table.get(ScopeId::new(0));
    assert_eq!(module_scope.bindings.len(), 1);
    let binding = result.binding_table.get(module_scope.bindings[0]);
    assert_eq!(binding.kind, DeclarationKind::Import);
    assert_eq!(binding.name, x_atom);
}

#[test]
fn import_bindings_resolve_before_their_declaration_position() {
    let x_atom = AtomId::from_raw(101);
    let y_atom = AtomId::from_raw(102);
    let parsed = make_module(|ast| {
        let use_expr = ast.alloc_expr(Expr::Identifier {
            span: span(),
            name: y_atom,
        });
        let use_stmt = ast.alloc_stmt(Stmt::Expression {
            span: span(),
            expression: use_expr,
        });
        let source = ast.literals_mut().alloc_string("mod");
        let spec = ImportSpecifier::Named {
            span: span(),
            imported: x_atom,
            local: y_atom,
        };
        let specs = ast.alloc_import_spec_list(&[spec]);
        let attributes = ast.alloc_import_attr_list(&[]);
        let decl = ast.alloc_decl(Decl::Import {
            span: span(),
            specifiers: specs,
            source,
            attributes,
        });
        let import_stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        vec![use_stmt, import_stmt]
    });
    let atoms = atoms();
    let result = analyze_module(&parsed, &atoms);
    let use_site = result
        .use_sites
        .as_slice()
        .iter()
        .find(|site| site.name == y_atom)
        .expect("module should record a use site for the imported name");
    assert_eq!(use_site.resolution_kind, ResolutionKind::Local);
    let binding = result.binding_table.get(
        use_site
            .resolved_binding
            .expect("import use should resolve"),
    );
    assert_eq!(binding.kind, DeclarationKind::Import);
    assert_eq!(binding.name, y_atom);
}

// ===================================================================
// Duplicate parameter names tests
// ===================================================================

#[test]
fn strict_mode_duplicate_params_error() {
    // "use strict"; function f(x, x) {}
    let f_atom = AtomId::from_raw(100);
    let x_atom = AtomId::from_raw(101);
    let parsed = make_strict_script(|ast| {
        let param1 = ast.alloc_pattern(Pattern::Identifier {
            span: span_at(0, 1),
            name: x_atom,
        });
        let param2 = ast.alloc_pattern(Pattern::Identifier {
            span: span_at(3, 4),
            name: x_atom,
        });
        let params_list = ast.alloc_pattern_list(&[param1, param2]);
        let body = ast.alloc_stmt_list(&[]);
        let func = ast.alloc_function(Function {
            span: span(),
            name: Some(f_atom),
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span(),
                params: params_list,
                rest: None,
            },
            body,
            expression_body: None,
        });
        let decl = ast.alloc_decl(Decl::Function {
            span: span(),
            function: func,
        });
        let stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(result.diagnostics.has_errors());
    let errors = result.diagnostics.as_slice();
    assert!(errors
        .iter()
        .any(|d| d.message.contains("duplicate parameter")));
}

#[test]
fn non_strict_simple_params_duplicate_ok() {
    // function f(x, x) {} — allowed in sloppy mode with simple params
    let f_atom = AtomId::from_raw(100);
    let x_atom = AtomId::from_raw(101);
    let parsed = make_script(|ast| {
        let param1 = ast.alloc_pattern(Pattern::Identifier {
            span: span_at(0, 1),
            name: x_atom,
        });
        let param2 = ast.alloc_pattern(Pattern::Identifier {
            span: span_at(3, 4),
            name: x_atom,
        });
        let params_list = ast.alloc_pattern_list(&[param1, param2]);
        let body = ast.alloc_stmt_list(&[]);
        let func = ast.alloc_function(Function {
            span: span(),
            name: Some(f_atom),
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span(),
                params: params_list,
                rest: None,
            },
            body,
            expression_body: None,
        });
        let decl = ast.alloc_decl(Decl::Function {
            span: span(),
            function: func,
        });
        let stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    // No errors — duplicate simple params allowed in sloppy mode.
    assert!(!result.diagnostics.has_errors());
}

// ===================================================================
// With statement tests
// ===================================================================

#[test]
fn with_in_strict_mode_error() {
    let parsed = make_strict_script(|ast| {
        let obj = ast.alloc_expr(Expr::Identifier {
            span: span(),
            name: AtomId::from_raw(100),
        });
        let body = ast.alloc_stmt(Stmt::Empty { span: span() });
        let with_stmt = ast.alloc_stmt(Stmt::With {
            span: span(),
            object: obj,
            body,
        });
        vec![with_stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(result.diagnostics.has_errors());
    let errors = result.diagnostics.as_slice();
    assert!(errors
        .iter()
        .any(|d| d.message.contains("'with' statement not allowed")));
}

// ===================================================================
// For loop scope tests
// ===================================================================

#[test]
fn for_let_creates_loop_scope() {
    // for (let i = 0; i < 10; i++) {}
    let i_atom = AtomId::from_raw(100);
    let parsed = make_script(|ast| {
        let pat = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: i_atom,
        });
        let init_val = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(0),
            syntax: NumericLiteralSyntax::Normal,
        });
        let declarator = VariableDeclarator {
            span: span(),
            id: pat,
            init: Some(init_val),
        };
        let declarators = ast.alloc_var_declarator_list(&[declarator]);
        let init_decl = ast.alloc_decl(Decl::Variable {
            span: span(),
            kind: VariableKind::Let,
            declarators,
        });

        let test_left = ast.alloc_expr(Expr::Identifier {
            span: span(),
            name: i_atom,
        });
        let test_right = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(10),
            syntax: NumericLiteralSyntax::Normal,
        });
        let test = ast.alloc_expr(Expr::BinaryExpression {
            span: span(),
            operator: lyng_js_ast::BinaryOp::Lt,
            left: test_left,
            right: test_right,
        });

        let update_arg = ast.alloc_expr(Expr::Identifier {
            span: span(),
            name: i_atom,
        });
        let update = ast.alloc_expr(Expr::UpdateExpression {
            span: span(),
            operator: lyng_js_ast::UpdateOp::Increment,
            argument: update_arg,
            prefix: false,
        });

        let body_stmts = ast.alloc_stmt_list(&[]);
        let body = ast.alloc_stmt(Stmt::Block {
            span: span(),
            body: body_stmts,
        });

        let for_stmt = ast.alloc_stmt(Stmt::For {
            span: span(),
            init: Some(ForInit::Declaration(init_decl)),
            test: Some(test),
            update: Some(update),
            body,
        });
        vec![for_stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    // Should have ForLoop scope
    let has_for_scope = (0..result.scope_table.len())
        .any(|i| result.scope_table.get(ScopeId::new(i as u32)).kind == ScopeKind::ForLoop);
    assert!(has_for_scope);

    // The `i` binding should be in the for-loop scope.
    let i_binding = result
        .binding_table
        .as_slice()
        .iter()
        .find(|b| b.name == i_atom)
        .unwrap();
    let scope_kind = result.scope_table.get(i_binding.scope).kind;
    assert_eq!(scope_kind, ScopeKind::ForLoop);
}

// ===================================================================
// Catch scope test
// ===================================================================

#[test]
fn catch_creates_scope_with_param() {
    // try {} catch (e) {}
    let e_atom = AtomId::from_raw(100);
    let parsed = make_script(|ast| {
        let try_body_stmts = ast.alloc_stmt_list(&[]);
        let try_body = ast.alloc_stmt(Stmt::Block {
            span: span(),
            body: try_body_stmts,
        });
        let catch_body_stmts = ast.alloc_stmt_list(&[]);
        let catch_body = ast.alloc_stmt(Stmt::Block {
            span: span(),
            body: catch_body_stmts,
        });
        let catch_param = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: e_atom,
        });
        let try_stmt = ast.alloc_stmt(Stmt::Try {
            span: span(),
            block: try_body,
            handler: Some(CatchClause {
                span: span(),
                param: Some(catch_param),
                body: catch_body,
            }),
            finalizer: None,
        });
        vec![try_stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    // Should have a Catch scope.
    let has_catch = (0..result.scope_table.len())
        .any(|i| result.scope_table.get(ScopeId::new(i as u32)).kind == ScopeKind::Catch);
    assert!(has_catch);
    // The 'e' binding should be CatchParam.
    let e_binding = result
        .binding_table
        .as_slice()
        .iter()
        .find(|b| b.name == e_atom)
        .unwrap();
    assert_eq!(e_binding.kind, DeclarationKind::CatchParam);
}

// ===================================================================
// Switch scope test
// ===================================================================

#[test]
fn switch_creates_scope() {
    // switch (1) { case 1: break; }
    let parsed = make_script(|ast| {
        let disc = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(1),
            syntax: NumericLiteralSyntax::Normal,
        });
        let brk = ast.alloc_stmt(Stmt::Break {
            span: span(),
            label: None,
        });
        let case_body = ast.alloc_stmt_list(&[brk]);
        let test_expr = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(1),
            syntax: NumericLiteralSyntax::Normal,
        });
        let case = SwitchCase {
            span: span(),
            test: Some(test_expr),
            consequent: case_body,
        };
        let cases = ast.alloc_switch_case_list(&[case]);
        let switch_stmt = ast.alloc_stmt(Stmt::Switch {
            span: span(),
            discriminant: disc,
            cases,
        });
        vec![switch_stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    let has_switch = (0..result.scope_table.len())
        .any(|i| result.scope_table.get(ScopeId::new(i as u32)).kind == ScopeKind::Switch);
    assert!(has_switch);
    // break inside switch should be ok.
    assert!(!result.diagnostics.has_errors());
}

// ===================================================================
// Labeled statement tests
// ===================================================================

#[test]
fn break_with_label_ok() {
    // label: while(true) { break label; }
    let label_atom = AtomId::from_raw(100);
    let parsed = make_script(|ast| {
        let test = ast.alloc_expr(Expr::BooleanLiteral {
            span: span(),
            value: true,
        });
        let brk = ast.alloc_stmt(Stmt::Break {
            span: span(),
            label: Some(label_atom),
        });
        let body_stmts = ast.alloc_stmt_list(&[brk]);
        let body = ast.alloc_stmt(Stmt::Block {
            span: span(),
            body: body_stmts,
        });
        let while_stmt = ast.alloc_stmt(Stmt::While {
            span: span(),
            test,
            body,
        });
        let labeled = ast.alloc_stmt(Stmt::Labeled {
            span: span(),
            label: label_atom,
            body: while_stmt,
        });
        vec![labeled]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(!result.diagnostics.has_errors());
}

// ===================================================================
// Arrow function test
// ===================================================================

#[test]
fn arrow_function_creates_scope() {
    // () => 1
    let parsed = make_script(|ast| {
        let params_list = ast.alloc_pattern_list(&[]);
        let body = ast.alloc_stmt_list(&[]);
        let expr_body = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(1),
            syntax: NumericLiteralSyntax::Normal,
        });
        let func = ast.alloc_function(Function {
            span: span(),
            name: None,
            kind: FunctionKind::Arrow,
            params: FormalParameters {
                span: span(),
                params: params_list,
                rest: None,
            },
            body,
            expression_body: Some(expr_body),
        });
        let arrow_expr = ast.alloc_expr(Expr::ArrowFunctionExpression {
            span: span(),
            function: func,
        });
        let stmt = ast.alloc_stmt(Stmt::Expression {
            span: span(),
            expression: arrow_expr,
        });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(!result.diagnostics.has_errors());
    // Should have at least Global + Function scope.
    assert!(result.scope_table.len() >= 2);
    assert!(!result.function_table.is_empty());
}

// ===================================================================
// Class body scope test
// ===================================================================

#[test]
fn class_creates_class_body_scope() {
    // class C { }
    let c_atom = AtomId::from_raw(100);
    let parsed = make_script(|ast| {
        let body = ast.alloc_class_element_list(&[]);
        let decl = ast.alloc_decl(Decl::Class {
            span: span(),
            name: Some(c_atom),
            super_class: None,
            body,
        });
        let stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    let has_class_scope = (0..result.scope_table.len())
        .any(|i| result.scope_table.get(ScopeId::new(i as u32)).kind == ScopeKind::ClassBody);
    assert!(has_class_scope);
    // Class name should be bound.
    let c_binding = result
        .binding_table
        .as_slice()
        .iter()
        .find(|b| b.name == c_atom)
        .unwrap();
    assert_eq!(c_binding.kind, DeclarationKind::Class);
}

#[test]
fn class_field_initializer_captures_outer_binding() {
    let outer_atom = AtomId::from_raw(100);
    let value_atom = AtomId::from_raw(101);
    let class_atom = AtomId::from_raw(102);
    let field_atom = AtomId::from_raw(103);
    let parsed = make_script(|ast| {
        let value_pattern = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: value_atom,
        });
        let constructor_params = ast.alloc_pattern_list(&[]);
        let constructor_body = ast.alloc_stmt_list(&[]);
        let constructor_function = ast.alloc_function(Function {
            span: span(),
            name: None,
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span(),
                params: constructor_params,
                rest: None,
            },
            body: constructor_body,
            expression_body: None,
        });
        let constructor_key = ast.alloc_expr(Expr::Identifier {
            span: span(),
            name: WellKnownAtom::constructor.id(),
        });
        let constructor = ast.alloc_class_element(ClassElement::Method {
            span: span(),
            kind: MethodKind::Constructor,
            key: constructor_key,
            value: constructor_function,
            computed: false,
            private: false,
            r#static: false,
        });
        let field_key = ast.alloc_expr(Expr::Identifier {
            span: span(),
            name: field_atom,
        });
        let field_value = ast.alloc_expr(Expr::Identifier {
            span: span(),
            name: value_atom,
        });
        let field = ast.alloc_class_element(ClassElement::Property {
            span: span(),
            key: field_key,
            value: Some(field_value),
            computed: false,
            private: false,
            r#static: false,
        });
        let class_body = ast.alloc_class_element_list(&[field, constructor]);
        let class_decl = ast.alloc_decl(Decl::Class {
            span: span(),
            name: Some(class_atom),
            super_class: None,
            body: class_body,
        });
        let class_stmt = ast.alloc_stmt(Stmt::Declaration {
            span: span(),
            decl: class_decl,
        });
        let function_body = ast.alloc_stmt_list(&[class_stmt]);
        let params = ast.alloc_pattern_list(&[value_pattern]);
        let function = ast.alloc_function(Function {
            span: span(),
            name: Some(outer_atom),
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span(),
                params,
                rest: None,
            },
            body: function_body,
            expression_body: None,
        });
        let decl = ast.alloc_decl(Decl::Function {
            span: span(),
            function,
        });
        let stmt = ast.alloc_stmt(Stmt::Declaration { span: span(), decl });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(!result.diagnostics.has_errors());

    let value_binding = result
        .binding_table
        .as_slice()
        .iter()
        .find(|binding| binding.name == value_atom && binding.kind == DeclarationKind::Parameter)
        .expect("field initializer should resolve the outer parameter binding");
    assert!(value_binding.is_captured);
    assert_eq!(value_binding.storage_class, StorageClass::EnvironmentSlot);
}

#[test]
fn class_expression_heritage_resolves_to_inner_class_name_binding() {
    let c_atom = AtomId::from_raw(100);
    let mut heritage_expr = None;
    let parsed = make_script(|ast| {
        let outer_pattern = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: c_atom,
        });
        let outer_init = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(0),
            syntax: NumericLiteralSyntax::Normal,
        });
        let outer_declarator = VariableDeclarator {
            span: span(),
            id: outer_pattern,
            init: Some(outer_init),
        };
        let outer_declarators = ast.alloc_var_declarator_list(&[outer_declarator]);
        let outer_decl = ast.alloc_decl(Decl::Variable {
            span: span(),
            kind: VariableKind::Let,
            declarators: outer_declarators,
        });
        let outer_stmt = ast.alloc_stmt(Stmt::Declaration {
            span: span(),
            decl: outer_decl,
        });

        let heritage = ast.alloc_expr(Expr::Identifier {
            span: span_at(20, 21),
            name: c_atom,
        });
        heritage_expr = Some(heritage);
        let class_body = ast.alloc_class_element_list(&[]);
        let class_expr = ast.alloc_expr(Expr::ClassExpression {
            span: span_at(20, 40),
            name: Some(c_atom),
            super_class: Some(heritage),
            body: class_body,
        });
        let class_stmt = ast.alloc_stmt(Stmt::Expression {
            span: span_at(20, 40),
            expression: class_expr,
        });
        vec![outer_stmt, class_stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(!result.diagnostics.has_errors());

    let use_record = result
        .use_sites
        .for_expr(heritage_expr.expect("heritage expression should be recorded"))
        .expect("heritage reference should create a use site");
    assert_eq!(use_record.resolution_kind, ResolutionKind::Local);

    let binding = result.binding_table.get(
        use_record
            .resolved_binding
            .expect("heritage reference should resolve"),
    );
    assert_eq!(binding.kind, DeclarationKind::ClassName);
    assert_eq!(
        result.scope_table.get(binding.scope).kind,
        ScopeKind::ClassBody
    );
}

#[test]
fn class_declaration_heritage_resolves_to_inner_class_name_binding() {
    let c_atom = AtomId::from_raw(100);
    let mut heritage_expr = None;
    let parsed = make_script(|ast| {
        let heritage = ast.alloc_expr(Expr::Identifier {
            span: span_at(20, 21),
            name: c_atom,
        });
        heritage_expr = Some(heritage);
        let class_body = ast.alloc_class_element_list(&[]);
        let class_decl = ast.alloc_decl(Decl::Class {
            span: span_at(20, 40),
            name: Some(c_atom),
            super_class: Some(heritage),
            body: class_body,
        });
        let class_stmt = ast.alloc_stmt(Stmt::Declaration {
            span: span_at(20, 40),
            decl: class_decl,
        });
        vec![class_stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(!result.diagnostics.has_errors());

    let use_record = result
        .use_sites
        .for_expr(heritage_expr.expect("heritage expression should be recorded"))
        .expect("heritage reference should create a use site");
    assert_eq!(use_record.resolution_kind, ResolutionKind::Local);

    let binding = result.binding_table.get(
        use_record
            .resolved_binding
            .expect("heritage reference should resolve"),
    );
    assert_eq!(binding.kind, DeclarationKind::ClassName);
    assert_eq!(
        result.scope_table.get(binding.scope).kind,
        ScopeKind::ClassBody
    );
}

// ===================================================================
// Private name outside class test
// ===================================================================

#[test]
fn private_name_outside_class_error() {
    // obj.#priv — outside any class
    let parsed = make_script(|ast| {
        let obj = ast.alloc_expr(Expr::Identifier {
            span: span(),
            name: AtomId::from_raw(100),
        });
        let priv_expr = ast.alloc_expr(Expr::PrivateMemberExpression {
            span: span(),
            object: obj,
            property: AtomId::from_raw(101),
        });
        let stmt = ast.alloc_stmt(Stmt::Expression {
            span: span(),
            expression: priv_expr,
        });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(result.diagnostics.has_errors());
    let errors = result.diagnostics.as_slice();
    assert!(errors.iter().any(|d| d.message.contains("private name")));
}

// ===================================================================
// Use strict directive test
// ===================================================================

#[test]
fn use_strict_directive_enables_strict() {
    // "use strict"; var eval = 1; — should error
    let parsed = make_script(|ast| {
        let use_strict_str = ast.literals_mut().alloc_string("use strict");
        let use_strict_expr = ast.alloc_expr(Expr::StringLiteral {
            span: span_at(0, 12),
            value: use_strict_str,
            syntax: StringLiteralSyntax::default(),
        });
        let directive_stmt = ast.alloc_stmt(Stmt::Expression {
            span: span_at(0, 13),
            expression: use_strict_expr,
        });

        let pat = ast.alloc_pattern(Pattern::Identifier {
            span: span_at(14, 18),
            name: WellKnownAtom::eval.id(),
        });
        let init = ast.alloc_expr(Expr::NumericLiteral {
            span: span_at(21, 22),
            value: NumericLiteral::Int32(1),
            syntax: NumericLiteralSyntax::Normal,
        });
        let declarator = VariableDeclarator {
            span: span_at(14, 22),
            id: pat,
            init: Some(init),
        };
        let declarators = ast.alloc_var_declarator_list(&[declarator]);
        let decl = ast.alloc_decl(Decl::Variable {
            span: span_at(14, 23),
            kind: VariableKind::Var,
            declarators,
        });
        let var_stmt = ast.alloc_stmt(Stmt::Declaration {
            span: span_at(14, 23),
            decl,
        });

        vec![directive_stmt, var_stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    assert!(result.diagnostics.has_errors());
    let errors = result.diagnostics.as_slice();
    assert!(errors
        .iter()
        .any(|d| d.message.contains("'eval' cannot be used")));
}

// ===================================================================
// Slot index assignment test
// ===================================================================

#[test]
fn environment_bindings_get_slot_indices() {
    // var x = 1; var y = 2; (function() { x; y; })
    let x_atom = AtomId::from_raw(100);
    let y_atom = AtomId::from_raw(101);
    let parsed = make_script(|ast| {
        // var x = 1;
        let pat_x = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: x_atom,
        });
        let init_x = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(1),
            syntax: NumericLiteralSyntax::Normal,
        });
        let decl_x = VariableDeclarator {
            span: span(),
            id: pat_x,
            init: Some(init_x),
        };
        let declarators_x = ast.alloc_var_declarator_list(&[decl_x]);
        let var_x = ast.alloc_decl(Decl::Variable {
            span: span(),
            kind: VariableKind::Var,
            declarators: declarators_x,
        });
        let stmt_x = ast.alloc_stmt(Stmt::Declaration {
            span: span(),
            decl: var_x,
        });

        // var y = 2;
        let pat_y = ast.alloc_pattern(Pattern::Identifier {
            span: span(),
            name: y_atom,
        });
        let init_y = ast.alloc_expr(Expr::NumericLiteral {
            span: span(),
            value: NumericLiteral::Int32(2),
            syntax: NumericLiteralSyntax::Normal,
        });
        let decl_y = VariableDeclarator {
            span: span(),
            id: pat_y,
            init: Some(init_y),
        };
        let declarators_y = ast.alloc_var_declarator_list(&[decl_y]);
        let var_y = ast.alloc_decl(Decl::Variable {
            span: span(),
            kind: VariableKind::Var,
            declarators: declarators_y,
        });
        let stmt_y = ast.alloc_stmt(Stmt::Declaration {
            span: span(),
            decl: var_y,
        });

        // (function() { x; y; })
        let x_ref = ast.alloc_expr(Expr::Identifier {
            span: span(),
            name: x_atom,
        });
        let y_ref = ast.alloc_expr(Expr::Identifier {
            span: span(),
            name: y_atom,
        });
        let x_stmt = ast.alloc_stmt(Stmt::Expression {
            span: span(),
            expression: x_ref,
        });
        let y_stmt = ast.alloc_stmt(Stmt::Expression {
            span: span(),
            expression: y_ref,
        });
        let func_body = ast.alloc_stmt_list(&[x_stmt, y_stmt]);
        let params_list = ast.alloc_pattern_list(&[]);
        let func = ast.alloc_function(Function {
            span: span(),
            name: None,
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span(),
                params: params_list,
                rest: None,
            },
            body: func_body,
            expression_body: None,
        });
        let func_expr = ast.alloc_expr(Expr::FunctionExpression {
            span: span(),
            function: func,
        });
        let expr_stmt = ast.alloc_stmt(Stmt::Expression {
            span: span(),
            expression: func_expr,
        });

        vec![stmt_x, stmt_y, expr_stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);

    // Both x and y should have slot indices since they're captured
    // and stored as EnvironmentSlot.
    let x_binding = result
        .binding_table
        .as_slice()
        .iter()
        .find(|b| b.name == x_atom && b.kind == DeclarationKind::Var)
        .unwrap();
    let y_binding = result
        .binding_table
        .as_slice()
        .iter()
        .find(|b| b.name == y_atom && b.kind == DeclarationKind::Var)
        .unwrap();

    assert_eq!(x_binding.storage_class, StorageClass::EnvironmentSlot);
    assert_eq!(y_binding.storage_class, StorageClass::EnvironmentSlot);
    assert!(x_binding.slot_index.is_some());
    assert!(y_binding.slot_index.is_some());
    // They should have different slot indices.
    assert_ne!(x_binding.slot_index, y_binding.slot_index);
}

// ===================================================================
// Function expression name binding test
// ===================================================================

#[test]
fn function_expression_name_binds_in_own_scope() {
    // (function myFunc() { myFunc; })
    let name_atom = AtomId::from_raw(100);
    let parsed = make_script(|ast| {
        let name_ref = ast.alloc_expr(Expr::Identifier {
            span: span(),
            name: name_atom,
        });
        let name_stmt = ast.alloc_stmt(Stmt::Expression {
            span: span(),
            expression: name_ref,
        });
        let body = ast.alloc_stmt_list(&[name_stmt]);
        let params_list = ast.alloc_pattern_list(&[]);
        let func = ast.alloc_function(Function {
            span: span(),
            name: Some(name_atom),
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: span(),
                params: params_list,
                rest: None,
            },
            body,
            expression_body: None,
        });
        let func_expr = ast.alloc_expr(Expr::FunctionExpression {
            span: span(),
            function: func,
        });
        let stmt = ast.alloc_stmt(Stmt::Expression {
            span: span(),
            expression: func_expr,
        });
        vec![stmt]
    });
    let atoms = atoms();
    let result = analyze_script(&parsed, &atoms);
    // Named function expressions create a dedicated name environment that
    // the body closes over through its own lexical scope.
    let name_uses: Vec<_> = result
        .use_sites
        .as_slice()
        .iter()
        .filter(|u| u.name == name_atom)
        .collect();
    assert!(!name_uses.is_empty());
    let use_record = name_uses.last().unwrap();
    assert_eq!(use_record.resolution_kind, ResolutionKind::Captured);
}

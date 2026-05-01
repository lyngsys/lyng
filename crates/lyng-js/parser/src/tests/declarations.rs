use super::*;

// ===========================================================================
// Declarations
// ===========================================================================

#[test]
fn parse_var_declaration() {
    let p = script_ok("var x = 1;");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Variable {
            kind, declarators, ..
        } = p.ast.get_decl(*decl)
        {
            assert_eq!(*kind, VariableKind::Var);
            assert_eq!(p.ast.get_var_declarator_list(*declarators).len(), 1);
        } else {
            panic!("expected variable declaration");
        }
    } else {
        panic!("expected declaration statement");
    }
}

#[test]
fn parse_let_declaration() {
    let p = script_ok("let x = 1;");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Variable { kind, .. } = p.ast.get_decl(*decl) {
            assert_eq!(*kind, VariableKind::Let);
        } else {
            panic!("expected variable declaration");
        }
    }
}

#[test]
fn parse_const_declaration() {
    let p = script_ok("const x = 1;");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Variable { kind, .. } = p.ast.get_decl(*decl) {
            assert_eq!(*kind, VariableKind::Const);
        } else {
            panic!("expected variable declaration");
        }
    }
}

#[test]
fn parse_multiple_declarators() {
    let p = script_ok("var x = 1, y = 2, z = 3;");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Variable { declarators, .. } = p.ast.get_decl(*decl) {
            assert_eq!(p.ast.get_var_declarator_list(*declarators).len(), 3);
        } else {
            panic!("expected variable declaration");
        }
    }
}

#[test]
fn parse_function_declaration() {
    let p = script_ok("function foo() { return 1; }");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Function { function, .. } = p.ast.get_decl(*decl) {
            let func = p.ast.get_function(*function);
            assert!(func.name.is_some());
            assert_eq!(func.kind, FunctionKind::Normal);
        } else {
            panic!("expected function declaration");
        }
    }
}

#[test]
fn parse_generator_declaration() {
    let p = script_ok("function* gen() { yield 1; }");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Function { function, .. } = p.ast.get_decl(*decl) {
            assert_eq!(p.ast.get_function(*function).kind, FunctionKind::Generator);
        }
    }
}

#[test]
fn parse_async_function_declaration() {
    let p = script_ok("async function foo() { await x; }");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Function { function, .. } = p.ast.get_decl(*decl) {
            assert_eq!(p.ast.get_function(*function).kind, FunctionKind::Async);
        }
    }
}

#[test]
fn parse_class_declaration() {
    let p = script_ok("class Foo { constructor() {} method() {} }");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Class { name, body, .. } = p.ast.get_decl(*decl) {
            assert!(name.is_some());
            assert_eq!(p.ast.get_class_element_list(*body).len(), 2);
        } else {
            panic!("expected class declaration");
        }
    }
}

#[test]
fn parse_decorator_syntax_on_classes_and_elements() {
    let p = script_ok(
        r#"
        function dec() {}
        let ns = { value: dec };
        @dec
        @ns.value
        class Foo {
            @dec()
            method() {}
            @(dec)
            static field;
        }
        let Bar = @dec() @(ns.value) class {
            @ns.value
            field;
        };
        "#,
    );
    let stmts = body(&p);
    assert_eq!(stmts.len(), 4);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[2]) {
        if let Decl::Class { body, .. } = p.ast.get_decl(*decl) {
            assert_eq!(p.ast.get_class_element_list(*body).len(), 2);
        } else {
            panic!("expected class declaration");
        }
    } else {
        panic!("expected declaration statement");
    }
}

#[test]
fn parse_decorator_private_member_expression_in_class_static_block() {
    script_ok(
        r#"
        class Foo {
            static #dec() {}
            static {
                let Bar = @Foo.#dec class {};
            }
        }
        "#,
    );
}

#[test]
fn parse_class_extends() {
    let p = script_ok("class Foo extends Bar {}");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Class { super_class, .. } = p.ast.get_decl(*decl) {
            assert!(super_class.is_some());
        }
    }
}

#[test]
fn parse_class_static_method() {
    let p = script_ok("class Foo { static bar() {} }");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Class { body, .. } = p.ast.get_decl(*decl) {
            let elements = p.ast.get_class_element_list(*body);
            assert_eq!(elements.len(), 1);
            if let ClassElement::Method { r#static, .. } = p.ast.get_class_element(elements[0]) {
                assert!(*r#static);
            }
        }
    }
}

#[test]
fn parse_class_field() {
    let p = script_ok("class Foo { x = 1; }");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Class { body, .. } = p.ast.get_decl(*decl) {
            let elements = p.ast.get_class_element_list(*body);
            assert_eq!(elements.len(), 1);
            assert!(matches!(
                p.ast.get_class_element(elements[0]),
                ClassElement::Property { .. }
            ));
        }
    }
}

#[test]
fn parse_class_accessor_field_definition() {
    let p = script_ok("class Foo { accessor $; accessor _ = 1; }");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Class { body, .. } = p.ast.get_decl(*decl) {
            let elements = p.ast.get_class_element_list(*body);
            assert_eq!(elements.len(), 2);
            for element in elements {
                assert!(matches!(
                    p.ast.get_class_element(*element),
                    ClassElement::Property { .. }
                ));
            }
        }
    }
}

#[test]
fn parse_class_accessor_line_terminator_as_field_name() {
    let p = script_ok("class Foo { accessor\n$; static accessor\n$; }");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Class { body, .. } = p.ast.get_decl(*decl) {
            let elements = p.ast.get_class_element_list(*body);
            assert_eq!(elements.len(), 4);
        }
    }
}

#[test]
fn parse_class_getter_setter() {
    let p = script_ok("class Foo { get x() { return 1; } set x(v) {} }");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Class { body, .. } = p.ast.get_decl(*decl) {
            let elements = p.ast.get_class_element_list(*body);
            assert_eq!(elements.len(), 2);
            if let ClassElement::Method { kind, .. } = p.ast.get_class_element(elements[0]) {
                assert_eq!(*kind, MethodKind::Get);
            }
            if let ClassElement::Method { kind, .. } = p.ast.get_class_element(elements[1]) {
                assert_eq!(*kind, MethodKind::Set);
            }
        }
    }
}

#[test]
fn parse_private_class_elements_mark_private() {
    let p = script_ok("class Foo { #x() {} #y = 1; }");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Class { body, .. } = p.ast.get_decl(*decl) {
            let elements = p.ast.get_class_element_list(*body);
            assert_eq!(elements.len(), 2);
            if let ClassElement::Method { private, .. } = p.ast.get_class_element(elements[0]) {
                assert!(*private);
            } else {
                panic!("expected private method");
            }
            if let ClassElement::Property { private, .. } = p.ast.get_class_element(elements[1]) {
                assert!(*private);
            } else {
                panic!("expected private property");
            }
        }
    }
}

#[test]
fn parse_static_as_instance_class_element_name() {
    let p = script_ok("class Foo { static; static() {} }");
    let stmts = body(&p);
    if let Stmt::Declaration { decl, .. } = p.ast.get_stmt(stmts[0]) {
        if let Decl::Class { body, .. } = p.ast.get_decl(*decl) {
            let elements = p.ast.get_class_element_list(*body);
            assert_eq!(elements.len(), 2);
            if let ClassElement::Property { r#static, .. } = p.ast.get_class_element(elements[0]) {
                assert!(!r#static);
            } else {
                panic!("expected instance field named static");
            }
            if let ClassElement::Method { r#static, .. } = p.ast.get_class_element(elements[1]) {
                assert!(!r#static);
            } else {
                panic!("expected instance method named static");
            }
        }
    }
}

#[test]
fn parse_optional_chain_private_member() {
    let p = script_ok("class Foo { #x = 1; access(obj) { return obj?.#x; } }");
    assert!(!p.diagnostics.has_errors());
}

#[test]
fn invalid_arrow_class_heritage_reports_error() {
    let p = script("class Foo extends () => {} {}");
    assert!(p.diagnostics.has_errors());
}

#[test]
fn invalid_for_in_lexical_headers_report_errors() {
    let p = script("for (let x, y in z) {}");
    assert!(p.diagnostics.has_errors());

    let p = script("for (const x = 1 of y) {}");
    assert!(p.diagnostics.has_errors());
}

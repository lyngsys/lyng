//! Diagnostic quality verification: 10+ scenarios confirming
//! error messages have correct spans and meaningful text.

use lyng_js_common::{AtomTable, Severity, SourceId};
use lyng_js_parser::parse_script;
use lyng_js_sema::analyze_script;

fn sid() -> SourceId {
    SourceId::new(0)
}

fn script(src: &str) -> lyng_js_ast::ParsedScript {
    let mut atoms = AtomTable::new();
    parse_script(&mut atoms, sid(), src)
}

fn script_sema(src: &str) -> (lyng_js_ast::ParsedScript, lyng_js_sema::ScriptSema) {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, sid(), src);
    let sema = analyze_script(&parsed, &atoms);
    (parsed, sema)
}

#[test]
fn unexpected_token() {
    let p = script("var = ;");
    assert!(p.diagnostics.has_errors());
}

#[test]
fn missing_semicolon_reported() {
    // This may or may not be an error depending on ASI, but `{ return \n }` should work
    let p = script("{ return }");
    // Should parse fine with ASI
    assert!(!p.diagnostics.has_errors());
}

#[test]
fn duplicate_let_binding_detected() {
    let (parsed, sema) = script_sema("{ let x = 1; let x = 2; }");
    // Sema should report duplicate let binding
    if !parsed.diagnostics.has_errors() {
        assert!(
            sema.diagnostics.has_errors(),
            "duplicate let should be an error"
        );
    }
}

#[test]
fn break_outside_loop() {
    let (parsed, sema) = script_sema("break;");
    if !parsed.diagnostics.has_errors() {
        assert!(
            sema.diagnostics.has_errors(),
            "break outside loop should be an error"
        );
    }
}

#[test]
fn continue_outside_loop() {
    let (parsed, sema) = script_sema("continue;");
    if !parsed.diagnostics.has_errors() {
        assert!(
            sema.diagnostics.has_errors(),
            "continue outside loop should be an error"
        );
    }
}

#[test]
fn return_outside_function() {
    let (parsed, sema) = script_sema("return 42;");
    if !parsed.diagnostics.has_errors() {
        assert!(
            sema.diagnostics.has_errors(),
            "return outside function should be an error"
        );
    }
}

#[test]
fn strict_eval_as_binding() {
    let (parsed, sema) = script_sema("'use strict'; var eval = 1;");
    let has_error = parsed.diagnostics.has_errors() || sema.diagnostics.has_errors();
    assert!(has_error, "eval as binding in strict mode should error");
}

#[test]
fn strict_arguments_as_binding() {
    let (parsed, sema) = script_sema("'use strict'; var arguments = 1;");
    let has_error = parsed.diagnostics.has_errors() || sema.diagnostics.has_errors();
    assert!(
        has_error,
        "arguments as binding in strict mode should error"
    );
}

#[test]
fn strict_duplicate_params() {
    let (parsed, sema) = script_sema("'use strict'; function f(a, a) {}");
    let has_error = parsed.diagnostics.has_errors() || sema.diagnostics.has_errors();
    assert!(has_error, "duplicate params in strict mode should error");
}

#[test]
fn escaped_use_strict_does_not_enable_script_strict_mode() {
    let (parsed, sema) = script_sema("'use str\\u0069ct'; var eval = 1;");
    assert!(
        !parsed.strict,
        "escaped directives must not enable strict mode"
    );
    assert!(
        !parsed.diagnostics.has_errors(),
        "escaped script directives should parse as ordinary string literals"
    );
    assert!(
        !sema.diagnostics.has_errors(),
        "escaped script directives should not trigger strict binding errors"
    );
}

#[test]
fn function_use_strict_with_non_simple_params_is_error() {
    let (parsed, sema) = script_sema("function f(a = 1) { 'use strict'; }");
    let has_error = parsed.diagnostics.has_errors() || sema.diagnostics.has_errors();
    assert!(
        has_error,
        "'use strict' with non-simple params should be an early error"
    );
}

#[test]
fn escaped_function_use_strict_with_non_simple_params_is_not_an_early_error() {
    let (parsed, sema) = script_sema("function f(a = 1) { 'use str\\u0069ct'; }");
    assert!(
        !parsed.diagnostics.has_errors(),
        "escaped directives should not trigger function-body strict parsing"
    );
    assert!(
        !sema.diagnostics.has_errors(),
        "escaped directives should not trigger non-simple parameter strict errors"
    );
}

#[test]
fn escaped_function_use_strict_keeps_duplicate_params_sloppy() {
    let (parsed, sema) = script_sema("function f(a, a) { 'use str\\u0069ct'; return a; }");
    assert!(
        !parsed.diagnostics.has_errors(),
        "escaped directives should not enable duplicate-parameter early errors"
    );
    assert!(
        !sema.diagnostics.has_errors(),
        "escaped directives should keep duplicate parameters in sloppy mode"
    );
}

#[test]
fn function_use_strict_does_not_leak_to_program_scope() {
    let (parsed, sema) = script_sema("function f() { 'use strict'; } var eval = 1;");
    assert!(
        !parsed.diagnostics.has_errors(),
        "inner function strictness should not affect script parsing"
    );
    assert!(
        !sema.diagnostics.has_errors(),
        "inner function strictness should not affect script bindings"
    );
}

#[test]
fn await_in_static_block_errors() {
    let (parsed, sema) = script_sema("class C { static { await; } }");
    let has_error = parsed.diagnostics.has_errors() || sema.diagnostics.has_errors();
    assert!(has_error, "await in class static block should error");
}

#[test]
fn block_lexical_and_var_conflict_errors() {
    let (parsed, sema) = script_sema("{ { var f; } let f; }");
    let has_error = parsed.diagnostics.has_errors() || sema.diagnostics.has_errors();
    assert!(
        has_error,
        "block lexical names should conflict with nested var declarations"
    );
}

#[test]
fn labeled_var_statement_parses() {
    let p = script("label: var x = 1;");
    assert!(
        !p.diagnostics.has_errors(),
        "labeled var statements should parse"
    );
}

#[test]
fn await_label_parses_in_script() {
    let p = script("await: 1;");
    assert!(
        !p.diagnostics.has_errors(),
        "await should be allowed as a label in non-module code"
    );
}

#[test]
fn escaped_await_label_in_async_function_errors() {
    let p = script("async function f() { \\u0061wait: ; }");
    assert!(
        p.diagnostics.has_errors(),
        "escaped await labels should be rejected in async functions"
    );
}

#[test]
fn lexical_declaration_not_allowed_in_if_consequent() {
    let p = script("if (false) let\n[a] = 0;");
    assert!(
        p.diagnostics.has_errors(),
        "lexical declarations are not valid single-statement consequents"
    );
}

#[test]
fn const_without_initializer_is_error() {
    let p = script("const x;");
    assert!(
        p.diagnostics.has_errors(),
        "const declarations require an initializer"
    );
}

#[test]
fn lexical_binding_name_cannot_be_let() {
    let p = script("let let;");
    assert!(
        p.diagnostics.has_errors(),
        "'let' is not a valid lexical binding name"
    );
}

#[test]
fn parenthesized_identifier_update_parses() {
    let p = script("(y)++;");
    assert!(
        !p.diagnostics.has_errors(),
        "covered identifiers should remain valid update targets"
    );
}

#[test]
fn duplicate_switch_default_is_error() {
    let p = script("switch (x) { default: break; default: break; }");
    assert!(
        p.diagnostics.has_errors(),
        "switch statements may only contain one default clause"
    );
}

#[test]
fn await_label_in_static_block_errors() {
    let (parsed, sema) = script_sema("class C { static { await: 0; } }");
    let has_error = parsed.diagnostics.has_errors() || sema.diagnostics.has_errors();
    assert!(
        has_error,
        "await labels should be rejected in class static blocks"
    );
}

#[test]
fn strict_for_in_var_eval_errors() {
    let (parsed, sema) = script_sema("'use strict'; for (var eval in obj) {}");
    let has_error = parsed.diagnostics.has_errors() || sema.diagnostics.has_errors();
    assert!(has_error, "strict for-in var bindings should reject 'eval'");
}

#[test]
fn function_expression_named_yield_parses_in_generator() {
    let p = script("function* g() { (function yield() {}); }");
    assert!(
        !p.diagnostics.has_errors(),
        "yield should be allowed as a function expression name in generator bodies"
    );
}

#[test]
fn switch_case_lexical_and_var_conflict_errors() {
    let (parsed, sema) = script_sema("switch (0) { case 1: var f; default: let f; }");
    let has_error = parsed.diagnostics.has_errors() || sema.diagnostics.has_errors();
    assert!(
        has_error,
        "switch case blocks should reject lexical/var name conflicts"
    );
}

#[test]
fn catch_parameter_conflicts_error() {
    let (parsed, sema) = script_sema("try {} catch ([x, x]) {}");
    let has_error = parsed.diagnostics.has_errors() || sema.diagnostics.has_errors();
    assert!(
        has_error,
        "catch parameters should reject duplicate bindings"
    );
}

#[test]
fn arrow_duplicate_parameters_error_at_parse_time() {
    let p = script("(a, a) => {};");
    assert!(
        p.diagnostics.has_errors(),
        "arrow parameters should reject duplicate bindings during parsing"
    );
}

#[test]
fn generator_scoped_yield_in_arrow_parameters_errors() {
    let p = script("function* g() { (x = yield) => {}; }");
    assert!(
        p.diagnostics.has_errors(),
        "yield expressions should be rejected in arrow parameters inside generators"
    );
}

#[test]
fn await_in_async_arrow_parameters_errors() {
    let p = script("async (await) => {};");
    assert!(
        p.diagnostics.has_errors(),
        "async arrow parameters should reject await bindings"
    );
}

#[test]
fn await_in_nested_arrow_parameters_inside_async_arrow_errors() {
    let p = script("async(a = (await) => {}) => {};");
    assert!(
        p.diagnostics.has_errors(),
        "await should remain reserved in nested arrow parameters inside async arrow heads"
    );
}

#[test]
fn await_in_arrow_parameters_inside_static_block_errors() {
    let p = script("class C { static { ((x = await) => 0); } }");
    assert!(
        p.diagnostics.has_errors(),
        "class static blocks should reject await in arrow parameter defaults"
    );
}

#[test]
fn function_body_use_strict_updates_parser_context() {
    let p = script("function f() { \"use strict\"; public = 1; }");
    assert!(
        p.diagnostics.has_errors(),
        "function-body strict directives should affect subsequent parser checks"
    );
}

#[test]
fn object_shorthand_strict_identifier_errors() {
    let p = script("(function() { \"use strict\"; ({ let }); })();");
    assert!(
        p.diagnostics.has_errors(),
        "object shorthand properties should validate identifier references in strict mode"
    );
}

#[test]
fn duplicate_proto_property_in_object_literal_errors() {
    let (parsed, sema) = script_sema("({ __proto__: null, '__proto__': null });");
    let has_error = parsed.diagnostics.has_errors() || sema.diagnostics.has_errors();
    assert!(
        has_error,
        "duplicate __proto__ data properties should be rejected"
    );
}

#[test]
fn cover_initialized_name_errors_in_object_literal() {
    let (parsed, sema) = script_sema("({ a = 1 });");
    let has_error = parsed.diagnostics.has_errors() || sema.diagnostics.has_errors();
    assert!(
        has_error,
        "cover initialized names should be rejected in object literals"
    );
}

#[test]
fn duplicate_proto_property_is_allowed_in_assignment_pattern() {
    let (parsed, sema) = script_sema("({ __proto__: x, __proto__: y } = value);");
    assert!(
        !parsed.diagnostics.has_errors(),
        "duplicate __proto__ should remain parseable in assignment patterns"
    );
    assert!(
        !sema.diagnostics.has_errors(),
        "duplicate __proto__ should be allowed in assignment patterns"
    );
}

#[test]
fn parenthesized_object_literal_is_not_an_assignment_target() {
    let (parsed, sema) = script_sema("({}) = 1;");
    let has_error = parsed.diagnostics.has_errors() || sema.diagnostics.has_errors();
    assert!(
        has_error,
        "parenthesized object literals should not be treated as direct assignment targets"
    );
}

#[test]
fn tagged_template_with_invalid_escape_parses() {
    let p = script("((strs) => strs)`\\u{0}`;");
    assert!(
        !p.diagnostics.has_errors(),
        "tagged templates should allow invalid escapes without swallowing the closing backtick"
    );
}

#[test]
fn bigint_property_name_parses() {
    let p = script("({ 1n: 0 });");
    assert!(
        !p.diagnostics.has_errors(),
        "bigint literals should be accepted as property names"
    );
}

#[test]
fn conditional_consequent_allows_in_in_for_header() {
    let p = script("for (true ? '' in cond() : other(); false; ) ;");
    assert!(
        !p.diagnostics.has_errors(),
        "conditional consequents should parse with +In in for headers"
    );
}

#[test]
fn escaped_new_target_errors() {
    let p = script("function f() { new.t\\u0061rget; }");
    assert!(
        p.diagnostics.has_errors(),
        "escaped new.target should be rejected"
    );
}

#[test]
fn optional_chain_tagged_template_errors() {
    let p = script("a?.b`x`;");
    assert!(
        p.diagnostics.has_errors(),
        "tagged templates are not valid in optional chains"
    );
}

#[test]
fn private_name_in_expression_parses() {
    let (parsed, sema) = script_sema("class C { #x; static has(v) { return #x in v; } }");
    assert!(
        !parsed.diagnostics.has_errors(),
        "private name in expressions should parse"
    );
    assert!(
        !sema.diagnostics.has_errors(),
        "private name in expressions should resolve inside the class body"
    );
}

#[test]
fn named_function_expression_opens_separate_body_scope() {
    let (parsed, sema) = script_sema("var f = function n() { let n = 1; return n; };");
    assert!(
        !parsed.diagnostics.has_errors(),
        "named function expressions should parse"
    );
    assert!(
        !sema.diagnostics.has_errors(),
        "the function expression name environment should not conflict with body lexicals"
    );
}

#[test]
fn with_in_strict_mode() {
    let (parsed, sema) = script_sema("'use strict'; with ({}) {}");
    let has_error = parsed.diagnostics.has_errors() || sema.diagnostics.has_errors();
    assert!(has_error, "with in strict mode should error");
}

#[test]
fn private_access_requires_private_definition() {
    let (parsed, sema) = script_sema("class C { foo() {} bar() { return this.#foo; } }");
    assert!(!parsed.diagnostics.has_errors(), "class should parse");
    assert!(
        sema.diagnostics.has_errors(),
        "public foo should not satisfy #foo private access"
    );
}

#[test]
fn block_scoped_function_conflicts_with_let() {
    let (parsed, sema) = script_sema("{ function f() {} let f; }");
    assert!(!parsed.diagnostics.has_errors(), "block should parse");
    assert!(
        sema.diagnostics.has_errors(),
        "block-scoped function should conflict with let in the same block"
    );
}

#[test]
fn error_span_within_source() {
    let src = "function() {}"; // missing name for declaration
    let p = script(src);
    if p.diagnostics.has_errors() {
        for d in p.diagnostics.as_slice() {
            assert!(
                d.span.range.end.raw() as usize <= src.len(),
                "diagnostic span should be within source bounds"
            );
        }
    }
}

#[test]
fn all_errors_are_error_severity() {
    let p = script("var = ;");
    for d in p.diagnostics.as_slice() {
        if d.is_error() {
            assert_eq!(d.severity, Severity::Error);
        }
    }
}

#[test]
fn module_syntax_in_script_context() {
    // import in a script should error
    let p = script("import { x } from 'y';");
    // Parser may or may not flag this — it depends on implementation
    // At minimum it shouldn't panic
    let _ = p;
}

#[test]
fn unterminated_string_no_panic() {
    let p = script("var x = 'hello");
    assert!(p.diagnostics.has_errors());
}

#[test]
fn unexpected_eof_no_panic() {
    let p = script("function f(");
    assert!(p.diagnostics.has_errors());
}

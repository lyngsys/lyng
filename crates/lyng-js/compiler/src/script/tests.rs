use super::*;
use lyng_js_parser::parse_script;
use lyng_js_sema::analyze_script;
use std::collections::HashSet;

#[test]
fn compile_script_allocates_persistent_slots_for_global_lexicals_and_explicit_global_access() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(0),
        "let local = 1; var global = 2; local + global;",
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit.function(unit.entry()).unwrap();
    let instructions = entry.instructions();

    assert!(entry.needs_environment());
    assert_eq!(entry.environment_slot_count(), 1);
    assert_eq!(entry.environment_bindings().len(), 1);
    assert_eq!(
        entry.environment_bindings()[0].name(),
        Some(atoms.intern("local"))
    );
    assert!(instructions.iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::AssignGlobal,
            ..
        }
    )));
    assert!(instructions.iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::LoadGlobal,
            ..
        }
    )));
    assert!(instructions.iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::LoadEnvSlot,
            ..
        }
    )));
}

#[test]
fn compile_script_assigns_and_loads_global_var_declarations_through_global_ops() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(901),
        "var x = 1; x;",
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit.function(unit.entry()).unwrap();

    assert!(entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::AssignGlobal,
            ..
        }
    )));
    assert!(entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::LoadGlobal,
            ..
        }
    )));
}

#[test]
fn compile_script_allows_strict_function_calls_named_eval_without_dynamic_poisoning() {
    let source = r#"
        var callCount = 0;

        function f(n) {
          "use strict";
          if (n === 0) {
            callCount += 1
            return;
          }
          return eval(n - 1);
        }
        eval = f;

        f(10);
    "#;
    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, lyng_js_common::SourceId::new(900), source);
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    assert!(compile_script(&parsed, &sema, &mut atoms).is_ok());
}

#[test]
fn compile_script_emits_child_templates_and_call_sites() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(1),
        r#"
            function outer(x) {
                return (y) => x + y;
            }
            let add = outer(2);
            add(3);
            "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit.function(unit.entry()).unwrap();

    assert!(unit.functions().len() >= 3);
    assert!(entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::CreateClosure,
            ..
        }
    )));
    assert!(entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abc {
            opcode: Opcode::Call,
            ..
        }
    )));
}

#[test]
fn attach_safepoint_reports_register_window_limit_as_lowering_error() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(36),
        "let marker = 1;",
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    let span = parsed.ast.get_script(parsed.root).span;
    let program = ProgramSource {
        ast: &parsed.ast,
        body: parsed.ast.get_script(parsed.root).body,
        span,
        strict: parsed.strict,
        kind: ProgramRootKind::Script,
    };
    let mut state = CompilationState::new(program, sema.view(), &mut atoms).unwrap();
    let entry = state.alloc_function_id();
    let mut compiler = FunctionCompiler::for_root(&mut state, entry).unwrap();
    compiler.builder.alloc_registers(u16::MAX).unwrap();
    compiler.builder.set_hidden_register_count(1);

    let error = compiler
        .attach_safepoint(0, span, SafepointKind::Allocation)
        .expect_err("oversized register window should fail lowering");

    assert!(matches!(
        error,
        LoweringError::BytecodeBuild {
            error: BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::FinalRegisterWindow,
            }
        }
    ));
}

#[test]
fn compile_script_allocates_function_environment_for_direct_arrow_children() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(2),
        r#"
            function outer() {
                return () => this;
            }
            outer();
            "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let outer = unit
        .functions()
        .iter()
        .find(|function| function.name().is_some())
        .expect("outer function should be lowered");

    assert!(outer.needs_environment());
    assert_eq!(outer.environment_slot_count(), 0);
}

#[test]
fn compile_script_lowers_unresolved_arguments_in_eval_poisoned_arrow_bodies_through_load_name() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(4),
        r#"
            const f = (p = eval("var arguments = 'param'")) => arguments;
            f();
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let arrow = unit
        .functions()
        .iter()
        .find(|function| function.kind() == lyng_js_bytecode::BytecodeFunctionKind::Arrow)
        .expect("arrow function should be lowered");

    assert!(arrow.needs_environment());
    assert!(arrow.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::LoadName,
            ..
        }
    )));
    assert!(!arrow.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::LoadGlobal,
            ..
        }
    )));
}

#[test]
fn compile_script_lowers_function_expression_eval_callee_through_load_name() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(5),
        r#"
            const f = function() {
                return eval("1");
            };
            f();
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let function_expr = unit
        .functions()
        .iter()
        .find(|function| {
            function.kind() == lyng_js_bytecode::BytecodeFunctionKind::Function
                && function.name().is_none()
        })
        .expect("function expression should be lowered");

    assert!(function_expr
        .instructions()
        .iter()
        .any(|instruction| matches!(
            instruction,
            lyng_js_bytecode::Instruction::Abx {
                opcode: Opcode::LoadName,
                ..
            }
        )));
    assert!(!function_expr
        .instructions()
        .iter()
        .any(|instruction| matches!(
            instruction,
            lyng_js_bytecode::Instruction::Abx {
                opcode: Opcode::LoadGlobal,
                ..
            }
        )));
}

#[test]
fn compile_script_lowers_logical_member_assignments_through_assign_property_ops() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(6),
        r#"
            "use strict";
            var obj = {};
            var key = "computed";
            obj.named ||= 1;
            obj[key] &&= 2;
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit.function(unit.entry()).unwrap();

    assert!(entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abc {
            opcode: Opcode::AssignNamedProperty,
            ..
        }
    )));
    assert!(entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abc {
            opcode: Opcode::AssignKeyedProperty,
            ..
        }
    )));
}

#[test]
fn compile_script_lowers_with_call_targets_through_captured_name_reference() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(6),
        r#"
            with ({}) {
                Object();
            }
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit
        .function(unit.entry())
        .expect("script entry should exist");

    for opcode in [
        Opcode::CaptureName,
        Opcode::LoadCapturedName,
        Opcode::LoadCapturedNameThis,
    ] {
        assert!(
            entry.instructions().iter().any(|instruction| matches!(
                instruction,
                lyng_js_bytecode::Instruction::Abx {
                    opcode: actual,
                    ..
                } if *actual == opcode
            )),
            "expected {opcode:?} in with-call lowering"
        );
    }
    assert!(!entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::LoadGlobal,
            ..
        }
    )));
}

#[test]
fn compile_script_lowers_dynamic_lookup_delete_through_delete_name() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(6),
        r#"
            function testcase() {
                delete x;
                var x;
            }
            testcase();
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let mut sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let x = atoms.intern("x");
    let binding_id = sema
        .binding_table
        .as_slice()
        .iter()
        .enumerate()
        .find_map(|(index, binding)| {
            (binding.name == x && binding.kind == lyng_js_sema::DeclarationKind::Var)
                .then_some(lyng_js_sema::SemanticBindingId::new(index as u32))
        })
        .expect("function-local x binding should exist");
    let binding = sema.binding_table.get_mut(binding_id);
    binding.storage_class = lyng_js_sema::StorageClass::DynamicLookup;
    binding.needs_environment = false;
    binding.slot_index = None;

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let testcase = unit
        .functions()
        .iter()
        .find(|function| function.name().map(|name| atoms.resolve(name)) == Some("testcase"))
        .expect("testcase should be lowered");

    assert!(testcase.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::DeleteName,
            ..
        }
    )));
    assert!(!testcase.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::DeleteGlobal,
            ..
        }
    )));
}

#[test]
fn compile_script_lowers_parenthesized_typeof_identifiers_through_resolve_name() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(6),
        r#"
            function testcase() {
                eval("function fun(x) { return x; }");
                return typeof (fun);
            }
            testcase();
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let testcase = unit
        .functions()
        .iter()
        .find(|function| function.name().map(|name| atoms.resolve(name)) == Some("testcase"))
        .expect("testcase should be lowered");

    assert!(testcase.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::ResolveName,
            ..
        }
    )));
}

#[test]
fn compile_script_lowers_typeof_before_for_lexical_shadow_through_resolve_global() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(37),
        r#"
            var beforeType;
            beforeType = typeof f;
            for (let f; ; ) {
                {
                    function f() {}
                }
                break;
            }
            beforeType;
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit.function(unit.entry()).unwrap();

    assert!(entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::ResolveGlobal,
            ..
        }
    )));
}

#[test]
fn compile_script_places_dead_eval_branch_after_jump_in_function_expression() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(6),
        r#"
            const f = function() {
                if (false) eval("1");
            };
            f();
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let function_expr = unit
        .functions()
        .iter()
        .find(|function| {
            function.kind() == lyng_js_bytecode::BytecodeFunctionKind::Function
                && function.name().is_none()
        })
        .expect("function expression should be lowered");
    let text = lyng_js_bytecode::disassemble(function_expr);

    let jump_index = text
        .find("JumpIfFalse")
        .expect("dead eval branch should use JumpIfFalse");
    let eval_index = text
        .find("LoadName")
        .expect("dead eval branch should still lower eval through LoadName");
    assert!(jump_index < eval_index);
}

#[test]
fn compile_script_named_function_expression_self_binding_allocates_own_environment() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(33),
        r#"
            let outcomes = [];
            let saved;
            (function observe() {
                saved = observe;
                return outcomes.length;
            })();
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let function_expr = unit
        .functions()
        .iter()
        .find(|function| {
            function.kind() == lyng_js_bytecode::BytecodeFunctionKind::Function
                && function.name() == Some(atoms.intern("observe"))
        })
        .expect("named function expression should be lowered");

    assert!(function_expr.needs_environment());
    assert_eq!(function_expr.environment_slot_count(), 1);
    assert_eq!(
        function_expr.environment_bindings()[0].name(),
        Some(atoms.intern("observe"))
    );
    assert!(function_expr
        .instructions()
        .iter()
        .any(|instruction| matches!(
            instruction,
            lyng_js_bytecode::Instruction::Abx {
                opcode: Opcode::StoreEnvSlot,
                bx: 0,
                ..
            }
        )));
    let text = lyng_js_bytecode::disassemble(function_expr);
    assert!(text.contains("LoadEnvSlot") && text.contains("depth=1, slot=0"));
}

#[test]
fn compile_script_does_not_poison_root_scope_from_nested_function_eval() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(7),
        r#"
            const f = function() {
                if (false) eval("1");
            };
            f();
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit
        .function(unit.entry())
        .expect("script entry should exist");

    assert!(entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::StoreEnvSlot,
            ..
        }
    )));
    assert!(!entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::AssignName,
            ..
        }
    )));
}

#[test]
fn compile_script_lowers_dynamic_identifier_updates_through_captured_name_ops() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(31),
        r#"
            var obj = { x: 1 };
            with (obj) {
                x += 1;
                x++;
                x = (delete obj.x, 4);
            }
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit
        .function(unit.entry())
        .expect("script entry should exist");

    assert!(entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::CaptureName,
            ..
        }
    )));
    assert!(entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::LoadCapturedName,
            ..
        }
    )));
    assert!(entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::AssignCapturedName,
            ..
        }
    )));
}

#[test]
fn compile_script_lowers_eval_poisoned_identifier_rmw_through_captured_name_ops() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(32),
        r#"
            function outer() {
                var x = 3;
                return function() {
                    x += (eval("var x = 2;"), 4);
                    x++;
                    return x;
                };
            }
            outer()();
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let inner = unit
        .functions()
        .iter()
        .find(|function| {
            function.kind() == lyng_js_bytecode::BytecodeFunctionKind::Function
                && function.name().is_none()
        })
        .expect("inner function should be lowered");

    assert!(inner.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::CaptureName,
            ..
        }
    )));
    assert!(inner.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::LoadCapturedName,
            ..
        }
    )));
    assert!(inner.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::AssignCapturedName,
            ..
        }
    )));
}

#[test]
fn compile_script_supports_large_register_functions_and_high_register_calls() {
    let mut source = String::new();
    for index in 0..280 {
        source.push_str(&format!("let value{index} = {index};\n"));
    }
    source.push_str("let fnRef = function(value) { return value; };\n");
    source.push_str("fnRef(value279);\n");

    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, lyng_js_common::SourceId::new(3), &source);
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit.function(unit.entry()).unwrap();

    assert!(entry.register_count() > 255);
    assert!(!entry.wide_operands().is_empty());
    assert!(entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abc {
            opcode: Opcode::Call,
            ..
        }
    )));
}

#[test]
fn compile_script_reuses_private_field_registers_for_extremely_large_classes() {
    let mut source = String::from("class Overflow {\n");
    for index in 0..10_000 {
        source.push_str(&format!("#field{index};\n"));
    }
    source.push_str("}\nOverflow;\n");

    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, lyng_js_common::SourceId::new(3_001), &source);
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit.function(unit.entry()).unwrap();

    assert!(entry.register_count() < u16::MAX);
}

#[test]
fn compile_script_reuses_array_literal_registers_for_large_nested_arrays() {
    let mut source = String::from("var mapping = [\n");
    for index in 0..22_000 {
        source.push_str(&format!("[{index}, {index}],\n"));
    }
    source.push_str("];\nmapping.length;\n");

    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, lyng_js_common::SourceId::new(3_002), &source);
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit.function(unit.entry()).unwrap();

    assert!(entry.register_count() < 1_024);
}

#[test]
fn compile_script_allocates_script_environment_for_captured_top_level_bindings() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(4),
        r#"
            let base = 40;
            function outer(step) {
                return function(delta) {
                    return base + step + delta;
                };
            }
            let add = outer(1);
            add(2);
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit.function(unit.entry()).unwrap();
    assert!(entry.needs_environment());
    assert_eq!(entry.environment_slot_count(), 2);
    assert_eq!(entry.environment_bindings().len(), 2);
    assert_eq!(
        entry.environment_bindings()[0].name(),
        Some(atoms.intern("base"))
    );
    assert!(entry.environment_bindings()[0].flags().is_mutable());
    assert!(entry.environment_bindings()[0].flags().is_lexical());
    assert!(entry.environment_bindings()[0].flags().needs_tdz());
    assert_eq!(
        entry.environment_bindings()[1].name(),
        Some(atoms.intern("add"))
    );
    assert!(entry.environment_bindings()[1].flags().is_mutable());
    assert!(entry.environment_bindings()[1].flags().is_lexical());
    assert!(entry.environment_bindings()[1].flags().needs_tdz());

    let inner = unit
        .functions()
        .iter()
        .find(|function| {
            function.captures().iter().any(|capture| {
                matches!(
                    capture.source(),
                    CaptureSource::EnvironmentSlot { depth, slot } if depth == 1 && slot == 0
                )
            })
        })
        .expect("inner closure should capture the script lexical environment");
    assert!(
        !inner.needs_environment(),
        "reading outer bindings must not allocate a redundant local environment"
    );
    assert!(inner.captures().iter().any(|capture| {
        matches!(
            capture.source(),
            CaptureSource::EnvironmentSlot { depth, slot } if depth == 1 && slot == 0
        )
    }));
}

#[test]
fn compile_script_assigns_feedback_sites_for_minimum_phase4_kinds() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(5),
        r#"
            function make(value) {
                return function(delta) { return value + delta; };
            }
            function Ctor(value) {
                this.value = value;
            }
            let obj = { answer: 1 };
            obj.answer;
            obj.answer = obj.answer + 1;
            obj["answer"];
            obj["answer"] = obj["answer"] + 1;
            1 < 2;
            make(1)(2);
            new Ctor(3);
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let mut kinds = HashSet::new();
    let mut named_property_atoms = Vec::new();

    for function in unit.functions() {
        for descriptor in function.feedback_sites() {
            kinds.insert(descriptor.kind());
            if let FeedbackSiteMetadata::NamedProperty(atom) = descriptor.metadata() {
                named_property_atoms.push(atom);
            }
        }
    }

    assert!(kinds.contains(&FeedbackSiteKind::Arithmetic));
    assert!(kinds.contains(&FeedbackSiteKind::Comparison));
    assert!(kinds.contains(&FeedbackSiteKind::NamedPropertyLoad));
    assert!(kinds.contains(&FeedbackSiteKind::NamedPropertyStore));
    assert!(kinds.contains(&FeedbackSiteKind::KeyedPropertyAccess));
    assert!(kinds.contains(&FeedbackSiteKind::Call));
    assert!(kinds.contains(&FeedbackSiteKind::Construct));
    assert!(!named_property_atoms.is_empty());
}

#[test]
fn compile_script_assigns_named_load_feedback_to_global_loads() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(6),
        "externalGlobal;",
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit.function(unit.entry()).unwrap();
    let name = unit
        .atoms()
        .iter()
        .find_map(|(atom, candidate)| {
            (candidate.as_str() == Some("externalGlobal")).then_some(*atom)
        })
        .expect("compiled unit should intern global name");
    let site = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::NamedPropertyLoad)
        .expect("global load should carry named-load feedback");

    assert_eq!(site.metadata(), FeedbackSiteMetadata::NamedProperty(name));
}

#[test]
fn compile_script_assigns_named_store_feedback_to_global_stores() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(7),
        "var globalValue; globalValue = 1;",
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit.function(unit.entry()).unwrap();
    let name = unit
        .atoms()
        .iter()
        .find_map(|(atom, candidate)| (candidate.as_str() == Some("globalValue")).then_some(*atom))
        .expect("compiled unit should intern global name");
    let site = entry
        .feedback_sites()
        .iter()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::NamedPropertyStore)
        .expect("global store should carry named-store feedback");

    assert_eq!(site.metadata(), FeedbackSiteMetadata::NamedProperty(name));
}

#[test]
fn compile_script_uses_direct_env_slot_updates_for_sibling_lexical_for_loops() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(9999),
        r#"
            for (let i = 0; i < 3; ++i) {}
            for (let i = 0; i < 2; ++i) {}
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit.function(unit.entry()).unwrap();
    let env_binding_names = entry
        .environment_bindings()
        .iter()
        .map(|binding| binding.name())
        .collect::<Vec<_>>();
    assert_eq!(
        env_binding_names,
        vec![Some(atoms.intern("i")), Some(atoms.intern("i"))]
    );

    assert!(!entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::CaptureName | Opcode::LoadCapturedName | Opcode::AssignCapturedName,
            ..
        }
    )));

    assert!(entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::LoadEnvSlot | Opcode::StoreEnvSlot | Opcode::AssignEnvSlot,
            bx: 0,
            ..
        }
    )));
    assert!(entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::LoadEnvSlot | Opcode::StoreEnvSlot | Opcode::AssignEnvSlot,
            bx: 1,
            ..
        }
    )));
}

#[test]
fn compile_script_marks_direct_tail_calls_explicitly() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(6),
        r#"
            function recur(step, value) {
                return step(value);
            }
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let recur = unit
        .functions()
        .iter()
        .find(|function| function.name().and_then(|name| unit.atom_text(name)) == Some("recur"))
        .expect("recursive function should be lowered");

    assert!(recur.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abc {
            opcode: Opcode::TailCall,
            ..
        }
    )));
    assert!(!recur.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abc {
            opcode: Opcode::Call,
            ..
        }
    )));
}

#[test]
fn compile_script_keeps_non_tail_calls_and_finally_returns_non_tail() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(7),
        r#"
            function direct(value) {
                let inner = function(next) { return next; };
                let result = inner(value);
                return result;
            }

            function guarded(value) {
                try {
                    return maybe(value);
                } finally {
                    value;
                }
            }
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let functions = unit
        .functions()
        .iter()
        .filter_map(|function| {
            function
                .name()
                .map(|name| (unit.atom_text(name).unwrap_or_default(), function))
        })
        .collect::<Vec<_>>();
    let direct = functions
        .iter()
        .find_map(|(name, function)| (*name == "direct").then_some(*function))
        .expect("direct function should be lowered");
    let guarded = functions
        .iter()
        .find_map(|(name, function)| (*name == "guarded").then_some(*function))
        .expect("guarded function should be lowered");

    assert!(direct.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abc {
            opcode: Opcode::Call,
            ..
        }
    )));
    assert!(!direct.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abc {
            opcode: Opcode::TailCall,
            ..
        }
    )));
    assert!(guarded.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abc {
            opcode: Opcode::Call,
            ..
        }
    )));
    assert!(!guarded.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abc {
            opcode: Opcode::TailCall,
            ..
        }
    )));
}

#[test]
fn compile_script_marks_conditional_tail_calls_in_each_branch() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(8),
        r#"
            function branch(flag, left, right) {
                return flag ? left() : right();
            }
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let branch = unit
        .functions()
        .iter()
        .find(|function| function.name().and_then(|name| unit.atom_text(name)) == Some("branch"))
        .expect("branch function should be lowered");
    let tail_calls = branch
        .instructions()
        .iter()
        .filter(|instruction| {
            matches!(
                instruction,
                lyng_js_bytecode::Instruction::Abc {
                    opcode: Opcode::TailCall,
                    ..
                }
            )
        })
        .count();

    assert_eq!(tail_calls, 2);
}

#[test]
fn compile_script_keeps_shadowed_eval_fallback_on_the_tail_path() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(22),
        r#"
            function recur(step, value) {
                var eval = step;
                return eval(value);
            }
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let recur = unit
        .functions()
        .iter()
        .find(|function| function.name().and_then(|name| unit.atom_text(name)) == Some("recur"))
        .expect("recursive function should be lowered");

    assert!(recur.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abc {
            opcode: Opcode::TailCall,
            ..
        }
    )));
    assert!(recur.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abc {
            opcode: Opcode::Call,
            ..
        }
    )));
}

#[test]
fn compile_script_attaches_metadata_at_allocation_loop_and_exception_boundaries() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(9),
        r#"
            function run(make) {
                let value = 0;
                while (value < 1) {
                    value = value + 1;
                }
                try {
                    return make({ count: value });
                } catch (err) {
                    return err;
                }
            }
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let run = unit
        .functions()
        .iter()
        .find(|function| function.name().and_then(|name| unit.atom_text(name)) == Some("run"))
        .expect("run function should be lowered");

    let allocation = run
        .safepoints()
        .iter()
        .find(|descriptor| descriptor.kind() == lyng_js_bytecode::SafepointKind::Allocation)
        .copied()
        .expect("call or object creation should expose an allocation safepoint");
    let loop_backedge = run
        .safepoints()
        .iter()
        .find(|descriptor| descriptor.kind() == lyng_js_bytecode::SafepointKind::LoopBackedge)
        .copied()
        .expect("loop backedge should expose a safepoint");
    let exception = run
        .safepoints()
        .iter()
        .find(|descriptor| descriptor.kind() == lyng_js_bytecode::SafepointKind::ExceptionEdge)
        .copied()
        .expect("catch edge should expose a safepoint");

    assert!(run
        .source_map_entry_at(allocation.instruction_offset())
        .is_some());
    assert!(run
        .source_map_entry_at(loop_backedge.instruction_offset())
        .is_some());
    assert!(run
        .source_map_entry_at(exception.instruction_offset())
        .is_some());
    assert!(allocation.captures_this());
    assert!(loop_backedge.captures_callee());
    assert!(exception.captures_exception_state());
    assert!(exception.captures_completion_state());

    let exception_snapshot = run
        .deopt_snapshot_for_safepoint(exception.id())
        .expect("exception edge should own a deopt snapshot");
    assert!(exception_snapshot
        .values()
        .contains(&lyng_js_bytecode::DeoptValueSource::FrameValue(
            lyng_js_bytecode::DeoptFrameValue::ExceptionValue,
        ),));
}

#[test]
fn compile_script_lowers_class_declarations_and_marks_method_shapes() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(10),
        r#"
            class C {
                constructor(value) {
                    this.value = value;
                }
                method() {
                    return this.value;
                }
                static get label() {
                    return "C";
                }
            }
            C;
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).expect("class should lower");
    let class_constructors = unit
        .functions()
        .iter()
        .filter(|function| function.flags().class_constructor())
        .count();
    let non_constructible_methods = unit
        .functions()
        .iter()
        .filter(|function| function.kind() == lyng_js_bytecode::BytecodeFunctionKind::Function)
        .filter(|function| !function.flags().constructible())
        .count();

    assert_eq!(class_constructors, 1);
    assert!(non_constructible_methods >= 2);
}

#[test]
fn compile_script_lowers_class_expressions_and_static_blocks() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(11),
        r#"
            let value = class Named {
                static total = 1;
                static {
                    this.extra = this.total + 1;
                }
                method() {
                    return Named.extra;
                }
            };
            value;
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit =
        compile_script(&parsed, &sema, &mut atoms).expect("named class expression should lower");
    let entry = unit
        .function(unit.entry())
        .expect("entry function should exist");
    assert!(entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abx {
            opcode: Opcode::CreateClosure,
            ..
        }
    )));
    assert!(entry.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abc {
            opcode: Opcode::SetFunctionName,
            ..
        }
    )));
}

#[test]
fn compile_script_counts_class_field_arrow_context_in_capture_depths() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(11_001),
        r#"
            class C {
                static field = () => C;
            }
            C.field();
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).expect("class field arrow should lower");
    let arrow = unit
        .functions()
        .iter()
        .find(|function| {
            function.captures().iter().any(|capture| {
                capture.name() == Some(atoms.intern("C"))
                    && matches!(
                        capture.source(),
                        CaptureSource::EnvironmentSlot { slot: 1, .. }
                    )
            })
        })
        .expect("arrow should capture the class name binding");

    assert!(arrow.captures().iter().any(|capture| {
        capture.name() == Some(atoms.intern("C"))
            && matches!(
                capture.source(),
                CaptureSource::EnvironmentSlot { depth: 1, slot: 1 }
            )
    }));
    assert!(lyng_js_bytecode::disassemble(arrow).contains("LoadEnvSlot     r3, depth=1, slot=1"));

    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(11_002),
        r#"
            class D {
                field = () => D;
            }
            new D().field();
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit =
        compile_script(&parsed, &sema, &mut atoms).expect("instance field arrow should lower");
    let arrow = unit
        .functions()
        .iter()
        .find(|function| {
            function.captures().iter().any(|capture| {
                capture.name() == Some(atoms.intern("D"))
                    && matches!(
                        capture.source(),
                        CaptureSource::EnvironmentSlot { slot: 1, .. }
                    )
            })
        })
        .expect("instance arrow should capture the class name binding");

    assert!(arrow.captures().iter().any(|capture| {
        capture.name() == Some(atoms.intern("D"))
            && matches!(
                capture.source(),
                CaptureSource::EnvironmentSlot { depth: 2, slot: 1 }
            )
    }));
}

#[test]
fn compile_script_forces_environments_for_explicit_derived_class_constructors() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(12),
        r#"
            class Base {
                constructor(value) {
                    this.value = value;
                }
            }
            class Derived extends Base {
                constructor(value) {
                    super(value);
                    return this;
                }
            }
            Derived;
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit =
        compile_script(&parsed, &sema, &mut atoms).expect("derived constructor should lower");
    let derived_constructor = unit
        .functions()
        .iter()
        .find(|function| function.flags().derived_class_constructor())
        .expect("derived constructor should be emitted");

    assert!(derived_constructor.needs_environment());
}

#[test]
fn compile_script_counts_forced_derived_constructor_environment_in_capture_depths() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(14),
        r#"
            let source = { value: 3 };
            class Base {
                constructor(value) {
                    this.value = value;
                }
            }
            class Derived extends Base {
                constructor() {
                    super(source.value);
                }
            }
            Derived;
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit =
        compile_script(&parsed, &sema, &mut atoms).expect("derived constructor should lower");
    let derived_constructor = unit
        .functions()
        .iter()
        .find(|function| function.flags().derived_class_constructor())
        .expect("derived constructor should be emitted");
    let text = lyng_js_bytecode::disassemble(derived_constructor);

    assert!(derived_constructor.needs_environment());
    assert!(text.contains("LoadEnvSlot") && text.contains("depth=1, slot=0"));
}

#[test]
fn compile_script_lowers_repeated_empty_class_bodies_with_shared_name() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(15),
        r#"
            const Base = function() {}.bind();
            class C extends Base {}
            (function() {
                class C extends Base {}
            })();
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    compile_script(&parsed, &sema, &mut atoms)
        .expect("repeated empty class bodies should lower independently");
}

#[test]
fn compile_script_marks_generator_functions_and_emits_resume_dispatch() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(14),
        r#"
            function* g() {
                const value = yield 1;
                return value;
            }
            g;
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).expect("generator should lower");
    let generator = unit
        .functions()
        .iter()
        .find(|function| function.name() == Some(atoms.intern("g")))
        .expect("generator child should be emitted");

    assert!(generator.flags().generator());
    assert!(!generator.flags().constructible());
    assert!(generator.flags().has_prototype_property());
    assert!(generator.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Ax {
            opcode: Opcode::SuspendGeneratorStart,
            ..
        }
    )));
    assert!(generator.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Ax {
            opcode: Opcode::Yield,
            ..
        }
    )));
    assert!(generator.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Ax {
            opcode: Opcode::LoadResumeKind,
            ..
        }
    )));
    assert!(generator.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Ax {
            opcode: Opcode::LoadResumeValue,
            ..
        }
    )));
}

#[test]
fn compile_script_marks_async_functions_and_emits_await_suspension() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(17),
        r#"
            async function f(value) {
                return await value;
            }
            f;
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).expect("async function should lower");
    let async_function = unit
        .functions()
        .iter()
        .find(|function| function.name() == Some(atoms.intern("f")))
        .expect("async child should be emitted");

    assert!(async_function.flags().async_function());
    assert!(!async_function.flags().generator());
    assert!(!async_function.flags().constructible());
    assert!(!async_function.flags().has_prototype_property());
    assert!(async_function
        .instructions()
        .iter()
        .any(|instruction| matches!(
            instruction,
            lyng_js_bytecode::Instruction::Ax {
                opcode: Opcode::Await,
                ..
            }
        )));
}

#[test]
fn compile_script_marks_async_generators_and_emits_suspend_resume_points() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(19),
        r#"
            async function* g(value) {
                yield await value;
            }
            g;
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).expect("async generator should lower");
    let generator = unit
        .functions()
        .iter()
        .find(|function| function.name() == Some(atoms.intern("g")))
        .expect("async generator child should be emitted");

    assert!(generator.flags().generator());
    assert!(generator.flags().async_function());
    assert!(!generator.flags().constructible());
    assert!(generator.flags().has_prototype_property());
    assert!(generator.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Ax {
            opcode: Opcode::SuspendGeneratorStart,
            ..
        }
    )));
    assert!(generator.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Ax {
            opcode: Opcode::Await,
            ..
        }
    )));
    assert!(generator.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Ax {
            opcode: Opcode::Yield,
            ..
        }
    )));
    assert!(generator.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Ax {
            opcode: Opcode::LoadResumeKind,
            ..
        }
    )));
    assert!(generator.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Ax {
            opcode: Opcode::LoadResumeValue,
            ..
        }
    )));
}

#[test]
fn compile_script_marks_generator_methods_with_prototype_properties() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(15),
        r#"
            let ordinary = { method() {} };
            let object = { *gen() { yield 1; } };
            class C { *gen() { yield 2; } }
            ordinary;
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).expect("generator methods should lower");
    let prototype_generators = unit
        .functions()
        .iter()
        .filter(|function| function.flags().generator())
        .filter(|function| function.flags().has_prototype_property())
        .count();
    let ordinary_methods_without_prototype = unit
        .functions()
        .iter()
        .filter(|function| !function.flags().generator())
        .filter(|function| !function.flags().constructible())
        .filter(|function| !function.flags().has_prototype_property())
        .count();

    assert!(prototype_generators >= 2);
    assert!(ordinary_methods_without_prototype >= 1);
}

#[test]
fn compile_script_lowers_for_await_of_loops_with_async_iterator_acquisition() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(18),
        r#"
            async function collect(iterable) {
                for await (const value of iterable) {
                    return value;
                }
            }
            collect;
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).expect("for await should lower");
    let collect = unit
        .functions()
        .iter()
        .find(|function| function.name() == Some(atoms.intern("collect")))
        .expect("async loop function should be emitted");

    assert!(collect.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abc {
            opcode: Opcode::CreateIterator,
            c: 1,
            ..
        }
    )));
    assert!(collect.instructions().iter().any(|instruction| matches!(
        instruction,
        lyng_js_bytecode::Instruction::Abc {
            opcode: Opcode::AdvanceIterator,
            ..
        }
    )));
}

#[test]
fn compile_script_computes_expected_argument_count_for_generator_defaults() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(16),
        r#"
            function* g(x, y = 1, z) {
                yield x + y + z;
            }
            g;
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).expect("generator should lower");
    let generator = unit
        .functions()
        .iter()
        .find(|function| function.name() == Some(atoms.intern("g")))
        .expect("generator child should be emitted");

    assert_eq!(generator.parameter_count(), 3);
    assert_eq!(generator.minimum_argument_count(), 1);
}

#[test]
fn compile_script_covers_direct_eval_internal_call_inside_try_with_catch_handler() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(19),
        r#"
            let status = 0;
            try {
                eval("missing");
                status += 1;
            } catch {
                status += 10;
            }
            throw new Error(String(status));
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit
        .function(unit.entry())
        .expect("entry script should lower");
    let direct_eval_call_offset = entry
        .instructions()
        .iter()
        .enumerate()
        .find_map(|(index, instruction)| {
            matches!(
                instruction,
                lyng_js_bytecode::Instruction::Abc {
                    opcode: Opcode::Call,
                    ..
                }
            )
            .then_some(index as u32)
        })
        .expect("direct eval should lower through an internal builtin call");

    assert!(entry
        .instructions()
        .iter()
        .take(usize::try_from(direct_eval_call_offset).unwrap_or(usize::MAX))
        .any(|instruction| {
            matches!(
                instruction,
                lyng_js_bytecode::Instruction::Abx {
                    opcode: Opcode::StoreEnvSlot,
                    ..
                }
            )
        }));
    assert!(!entry
        .instructions()
        .iter()
        .take(usize::try_from(direct_eval_call_offset).unwrap_or(usize::MAX))
        .any(|instruction| {
            matches!(
                instruction,
                lyng_js_bytecode::Instruction::Abx {
                    opcode: Opcode::AssignName,
                    ..
                }
            )
        }));
    assert!(entry.exception_handlers().iter().any(|handler| {
        handler.protected_start() <= direct_eval_call_offset
            && direct_eval_call_offset < handler.protected_end()
    }));
}

#[test]
fn compile_script_records_direct_eval_lexical_site_for_active_block_scope() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(20),
        r#"
            let x = "outside";
            {
                let x = "inside";
                eval("x;");
            }
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit
        .function(unit.entry())
        .expect("entry script should lower");
    let site = entry
        .direct_eval_lexical_sites()
        .first()
        .expect("direct eval should record lexical-site metadata");
    assert_eq!(site.scopes().len(), 1);
    let scope = &site.scopes()[0];
    assert_eq!(scope.bindings().len(), 1);
    assert_eq!(scope.bindings()[0].name(), Some(atoms.intern("x")));
    assert!(scope.bindings()[0].flags().is_lexical());
}

#[test]
fn compile_script_marks_direct_eval_catch_parameter_scope_for_annex_b() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(22),
        r#"
            try {
                throw null;
            } catch (err) {
                eval("var err;");
            }
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit
        .function(unit.entry())
        .expect("entry script should lower");
    let site = entry
        .direct_eval_lexical_sites()
        .first()
        .expect("direct eval should record lexical-site metadata");
    let err = atoms.intern("err");

    assert!(site
        .scopes()
        .iter()
        .any(|scope| scope.annex_b_catch_name() == Some(err)));
}

#[test]
fn compile_script_marks_nested_direct_eval_catch_parameter_scope_for_annex_b() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(23),
        r#"
            try {
                throw null;
            } catch (err) {
                try {
                    eval("var err;");
                } catch (error) {}
            }
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit
        .function(unit.entry())
        .expect("entry script should lower");
    let err = atoms.intern("err");

    assert!(entry.direct_eval_lexical_sites().iter().any(|site| {
        site.scopes()
            .iter()
            .any(|scope| scope.annex_b_catch_name() == Some(err))
    }));
}

#[test]
fn compile_script_records_direct_eval_site_for_class_initializer_without_lexical_captures() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(21),
        r#"
            let C = class {
                x = eval("arguments");
            };
            new C();
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let entry = unit
        .function(unit.entry())
        .expect("entry script should lower");
    let site = entry
        .child_functions()
        .iter()
        .filter_map(|child| unit.function(*child))
        .flat_map(|function| function.direct_eval_lexical_sites().iter())
        .find(|site| site.flags().forbid_arguments_in_class_initializer())
        .expect("class initializer direct eval should record site flags");
    assert!(site.scopes().is_empty());
}

#[test]
fn compile_script_records_class_name_scope_for_field_initializer_eval() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        lyng_js_common::SourceId::new(24),
        r#"
            class C {
                static direct = eval("C");
                static arrow = () => eval("C");
            }
        "#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let c_atom = atoms.intern("C");
    let unit = compile_script(&parsed, &sema, &mut atoms).unwrap();
    let sites_with_class_name_scope =
        unit.functions()
            .iter()
            .flat_map(|function| function.direct_eval_lexical_sites().iter())
            .filter(|site| {
                site.scopes().iter().any(|scope| {
                    scope.bindings().iter().any(|binding| {
                        binding.name() == Some(c_atom) && binding.flags().is_lexical()
                    })
                })
            })
            .count();

    assert!(
        sites_with_class_name_scope >= 2,
        "static direct eval and the arrow eval should both carry C's class-name scope"
    );
}

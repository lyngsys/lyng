//! Integration tests for the inline `op_load_local_N` / `op_store_local_N`
//! handlers (slots 0..3). Parameters occupy slots 0..N-1; `let` bindings
//! begin at slot 4.

use lyng_common::{AtomTable, SourceId};
use lyng_compiler::compile_script;
use lyng_env::Runtime;
use lyng_host::NoopHostHooks;
use lyng_parser::parse_script;
use lyng_sema::analyze_script;
use lyng_types::Value;
use lyng_vm::Vm;

/// Compile and run `src` in a fresh realm. Each test gets its own agent.
fn run_script(src: &str) -> Value {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, SourceId::new(0), src);
    assert!(
        !parsed.diagnostics.has_errors(),
        "script should parse cleanly: {:?}",
        parsed.diagnostics.as_slice()
    );
    let sema = analyze_script(&parsed, &atoms);
    assert!(
        !sema.diagnostics.has_errors(),
        "script should pass sema: {:?}",
        sema.diagnostics.as_slice()
    );
    let unit = compile_script(&parsed, &sema, &mut atoms).expect("script should compile");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut vm = Vm::new();
    vm.evaluate_script(agent, realm, &unit)
        .run()
        .expect("script should execute without VM error")
}

#[test]
fn load_local_0_returns_first_parameter() {
    // Parameter at slot 0 → LoadLocal0.
    let value = run_script("(function(a) { return a; })(42);");
    assert_eq!(value, Value::from_smi(42));
}

#[test]
fn load_local_1_returns_second_parameter() {
    // Parameter at slot 1 → LoadLocal1.
    let value = run_script("(function(a, b) { return b; })(10, 20);");
    assert_eq!(value, Value::from_smi(20));
}

#[test]
fn load_local_2_returns_third_parameter() {
    // Parameter at slot 2 → LoadLocal2.
    let value = run_script("(function(a, b, c) { return c; })(10, 20, 30);");
    assert_eq!(value, Value::from_smi(30));
}

#[test]
fn load_local_3_returns_fourth_parameter() {
    // Parameter at slot 3 → LoadLocal3.
    let value = run_script("(function(a, b, c, d) { return d; })(10, 20, 30, 40);");
    assert_eq!(value, Value::from_smi(40));
}

#[test]
fn load_locals_aggregate() {
    // All four LoadLocal slots in one expression. 1+2+3+4 = 10.
    let value = run_script("(function(a, b, c, d) { return a + b + c + d; })(1, 2, 3, 4);");
    assert_eq!(value, Value::from_smi(10));
}

#[test]
fn store_local_3_updates_param_via_assignment() {
    // Assignment to slot-3 parameter → StoreLocal3.
    let value = run_script(
        r"
        (function(a, b, c, d) {
            d = a + b + c + d;
            return d;
        })(1, 2, 3, 100);
        ",
    );
    assert_eq!(value, Value::from_smi(106));
}

#[test]
fn store_local_0_1_2_via_assignments() {
    // Mutations to slots 0/1/2 → StoreLocal0/1/2. a=20, b=60, c=120 → 200.
    let value = run_script(
        r"
        (function(a, b, c) {
            a = a * 2;
            b = b * 3;
            c = c * 4;
            return a + b + c;
        })(10, 20, 30);
        ",
    );
    assert_eq!(value, Value::from_smi(200));
}

#[test]
fn locals_in_tight_loop_sum() {
    // Tight loop exercising LoadLocal0/1/2/3 every iteration. Sum 0+…+99 = 4950.
    let value = run_script(
        r"
        (function(iters, p1, p2, p3) {
            var s = 0;
            for (var i = 0; i < iters; i++) {
                s = s + i + (p1 - p1) + (p2 - p2) + (p3 - p3);
            }
            return s;
        })(100, 1, 2, 3);
        ",
    );
    assert_eq!(value, Value::from_smi(4950));
}

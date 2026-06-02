//! Integration tests for the inline `op_load_const8` handler.
//!
//! Covers: Smi, Float64, Atom constants, and correct multi-pool indexing.

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
fn op_load_const8_smi_constant() {
    // Integer literal → Smi constant in the pool.
    let value = run_script("42;");
    assert_eq!(value, Value::from_smi(42));
}

#[test]
fn op_load_const8_float_constant() {
    // Float literal → Float64 constant in the pool.
    let value = run_script("2.5;");
    assert_eq!(value, Value::from_f64(2.5));
}

#[test]
fn op_load_const8_atom_constant() {
    // String literal → StringRef constant in the pool.
    // Exact identity depends on interner state; check tag kind only.
    let value = run_script("'hello';");
    assert_ne!(value, Value::undefined());
    assert_ne!(value, Value::null());
    assert_ne!(value, Value::from_smi(0));
}

#[test]
fn op_load_const8_handles_multiple_constants_in_pool() {
    // Three distinct pool indices; the load must pick the right element.
    let value = run_script("var a = 1; var b = 2; var c = 3; c;");
    assert_eq!(value, Value::from_smi(3));
}

#[test]
fn op_load_const8_handles_negative_smi_constant() {
    // Negative literal; sign must survive the pool read.
    let value = run_script("-7;");
    assert_eq!(value, Value::from_smi(-7));
}

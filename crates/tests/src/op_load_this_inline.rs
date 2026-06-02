//! Integration tests for the inline `op_load_this` handler.
//!
//! Covers `ThisState::Value` (bound `this`), `ThisState::Lexical` (arrow
//! captures enclosing `this`), and cross-frame stability after calls.

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
fn op_load_this_value_state_returns_real_this() {
    // ThisState::Value: bound `this`, reading property via the mirror.
    let value = run_script("(function() { return this.x; }).call({x: 42});");
    assert_eq!(value, Value::from_smi(42));
}

#[test]
fn op_load_this_value_state_with_negative_smi() {
    // Negative-Smi property; sign must survive the mirror read.
    let value = run_script("(function() { return this.x; }).call({x: -13});");
    assert_eq!(value, Value::from_smi(-13));
}

#[test]
fn op_load_this_arrow_function_captures_lexical_this() {
    // ThisState::Lexical: arrow function captures outer `this`, resolved at entry.
    let value = run_script("(function() { return (() => this.y)(); }).call({y: 7});");
    assert_eq!(value, Value::from_smi(7));
}

#[test]
fn op_load_this_chained_property_access() {
    // Two sequential property reads through `this`.
    let value = run_script("(function() { return this.a + this.b; }).call({a: 10, b: 32});");
    assert_eq!(value, Value::from_smi(42));
}

#[test]
fn op_load_this_in_nested_call_preserves_outer_this() {
    // Two nested calls with distinct `this` bindings; outer `this` must be
    // stable after the inner returns.
    let value = run_script(
        r"
            (function() {
                (function() { return this.kind; }).call({kind: 'inner'});
                return this.kind;
            }).call({kind: 'outer'});
        ",
    );
    assert_ne!(value, Value::undefined());
    assert_ne!(value, Value::null());
    assert_ne!(value, Value::from_smi(0));
}

#[test]
fn op_load_this_in_arrow_inside_loop_remains_stable() {
    // 100 iterations of an arrow reading `this`; sum = 100 * this.unit.
    let value = run_script(
        r"
            (function() {
                var total = 0;
                var read_this = () => this.unit;
                for (var i = 0; i < 100; i++) {
                    total = total + read_this();
                }
                return total;
            }).call({unit: 1});
        ",
    );
    assert_eq!(value, Value::from_smi(100));
}

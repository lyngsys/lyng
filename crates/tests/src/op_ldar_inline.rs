//! Integration tests for the inline `op_ldar` handler.
//!
//! Ldar copies `registers[a]` into the accumulator. Inline body:
//!
//! ```text
//!     load_reg!(a => 10);   ; ldr x10, [x20, x_a, lsl #3]
//!     store_acc!(10);       ; str x10, [x20]
//!     dispatch!();
//! ```

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
fn ldar_via_intermediate_temporary() {
    // (a + b) into a temp, then Ldar reads it into the accumulator before multiply.
    let value = run_script("(function(a, b) { var c = a + b; return c * 2; })(1, 2);");
    assert_eq!(value, Value::from_smi(6));
}

#[test]
fn ldar_in_chained_arithmetic() {
    // Multiple Ldar dispatches; wrong source-register decode would corrupt the chain.
    let value = run_script(
        "(function(a, b, c) { var x = a + b; var y = x + c; return y * 10; })(1, 2, 3);",
    );
    // a+b = 3; (a+b)+c = 6; *10 = 60.
    assert_eq!(value, Value::from_smi(60));
}

#[test]
fn ldar_with_function_call_result() {
    // Function return value lands in a temporary; reading it back dispatches Ldar.
    let value = run_script(
        r"
        (function() {
            function add(x, y) { return x + y; }
            var r = add(3, 4);
            return r + 1;
        })();
        ",
    );
    assert_eq!(value, Value::from_smi(8));
}

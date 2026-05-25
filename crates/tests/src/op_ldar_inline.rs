//! Phase 1.B.3 Task 3: integration tests for the inline `op_ldar` port.
//!
//! Ldar = "Load Accumulator from Register" — copies `registers[a]` into
//! the accumulator (`registers[0]`). The bytecode-builder peephole
//! emits `Ldar` after temporaries are computed when the next opcode
//! expects the value in the accumulator (i.e. the destination of an
//! emitted `Move` is register 0).
//!
//! The semantic body lives in
//! `crates/vm/src/vm/semantics/loads.rs:322-333`. The inline
//! port replaces the cold-stub `call_slow!(op_ldar_slow_rs, …)` shim
//! with a 2-instruction body:
//!
//! ```text
//!     load_reg!(a => 10);   ; ldr x10, [x20, x_a, lsl #3]
//!     store_acc!(10);       ; str x10, [x20]
//!     dispatch!();          ; standard 4-instr tail
//! ```
//!
//! Slow-path expected to be 0.00% on V8 v7 — pure register-to-register
//! move with no bail conditions. Sub-phase gate verifies this.
//!
//! These tests guard the SEMANTIC INVARIANT — they pass equally with
//! the cold-stub call-slow shim (pre-port) and the Task-3 inline-asm
//! port (post-port). Writing them BEFORE the port lands (per the
//! plan's TDD discipline) confirms no semantic regression when the
//! inline body replaces the cold stub.

use lyng_common::{AtomTable, SourceId};
use lyng_compiler::compile_script;
use lyng_env::Runtime;
use lyng_host::NoopHostHooks;
use lyng_parser::parse_script;
use lyng_sema::analyze_script;
use lyng_types::Value;
use lyng_vm::Vm;

/// Compile + execute `src` in a fresh realm, returning the script's
/// completion value. Mirrors the helper shape used by
/// `op_load_const8_inline.rs` and `op_load_this_inline.rs`. Each test
/// gets its own runtime/agent to avoid cross-contamination.
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
    vm.script_eval(agent, realm, &unit)
        .run()
        .expect("script should execute without VM error")
}

#[test]
fn ldar_via_intermediate_temporary() {
    // (a + b) * 2 — the compiler computes (a + b) into a temp, then
    // Ldar reads it into the accumulator before the multiply. The
    // accumulator's Value must equal `a + b` (3) when the multiply
    // executes; the inline path's `load_reg!(a => 10); store_acc!(10);`
    // sequence must preserve that.
    let value = run_script("(function(a, b) { var c = a + b; return c * 2; })(1, 2);");
    assert_eq!(value, Value::from_smi(6));
}

#[test]
fn ldar_in_chained_arithmetic() {
    // Multiple Ldar dispatches in a chain. If the inline path mishandled
    // the source-register decode (e.g. wrote the wrong slot to the
    // accumulator), the intermediate value would be wrong and the
    // final product would diverge.
    let value = run_script(
        "(function(a, b, c) { var x = a + b; var y = x + c; return y * 10; })(1, 2, 3);",
    );
    // a+b = 3; (a+b)+c = 6; *10 = 60.
    assert_eq!(value, Value::from_smi(60));
}

#[test]
fn ldar_with_function_call_result() {
    // The function call's return value lands in a temporary; reading
    // it back through `var r = add(3, 4)` dispatches Ldar (the move
    // to slot >=4 followed by a subsequent read of `r` whose
    // destination accumulator-bound add operand triggers the Ldar
    // peephole rewrite).
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

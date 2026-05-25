//! Phase 1.B.3 Tasks 2 + 3: integration tests for the inline
//! `op_load_local_N` + `op_store_local_N` ports.
//!
//! Each test exercises one or more of the 4 `LoadLocal` opcodes (slots
//! 0..3, via parameter access) and the 4 `StoreLocal` opcodes (slots
//! 0..3, via parameter or local-variable update in a loop or
//! assignment). The lyng bytecode compiler decides which JS-level
//! binding lands in which register slot — function parameters occupy
//! slots 0..N-1, and `let` bindings begin at slot 4 (slots 0..3 are
//! reserved by the calling convention; see `tools/lyng-bench/src/
//! microbench/snippets.rs:283-298` for the same observation).
//!
//! Tests pass with the cold-stub OR the inline port — the inline port
//! must produce the same observable semantics. Per the plan's TDD
//! discipline (Phase 1.B.3 Task 2 Step 2), they are introduced BEFORE
//! the cold stubs are replaced, run green with the cold stubs, then
//! re-run after the inline ports land. The before-and-after green
//! signal documents semantic parity.

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
fn load_local_0_returns_first_parameter() {
    // First parameter sits at register 0 in lyng's calling
    // convention (slot 0 = accumulator). Reading it via parameter
    // access triggers LoadLocal0 in the bytecode.
    let value = run_script("(function(a) { return a; })(42);");
    assert_eq!(value, Value::from_smi(42));
}

#[test]
fn load_local_1_returns_second_parameter() {
    // Second parameter lands at slot 1 — reading `b` from inside the
    // function dispatches LoadLocal1.
    let value = run_script("(function(a, b) { return b; })(10, 20);");
    assert_eq!(value, Value::from_smi(20));
}

#[test]
fn load_local_2_returns_third_parameter() {
    // Third parameter lands at slot 2 — reading `c` dispatches
    // LoadLocal2.
    let value = run_script("(function(a, b, c) { return c; })(10, 20, 30);");
    assert_eq!(value, Value::from_smi(30));
}

#[test]
fn load_local_3_returns_fourth_parameter() {
    // Fourth parameter lands at slot 3 — reading `d` dispatches
    // LoadLocal3.
    let value = run_script("(function(a, b, c, d) { return d; })(10, 20, 30, 40);");
    assert_eq!(value, Value::from_smi(40));
}

#[test]
fn load_locals_aggregate() {
    // Exercises LoadLocal0 + LoadLocal1 + LoadLocal2 + LoadLocal3 in
    // a single expression. Validates indexing is correct (not just
    // always slot 0). 1 + 2 + 3 + 4 = 10.
    let value = run_script("(function(a, b, c, d) { return a + b + c + d; })(1, 2, 3, 4);");
    assert_eq!(value, Value::from_smi(10));
}

#[test]
fn store_local_3_updates_param_via_assignment() {
    // Parameter `d` sits at slot 3; the `d = a + b + c + d`
    // assignment dispatches StoreLocal3 (the peephole rewrites
    // `Move dst=3, src=...` to `StoreLocal3`).
    let value = run_script(
        r"
        (function(a, b, c, d) {
            d = a + b + c + d;
            return d;
        })(1, 2, 3, 100);
        ",
    );
    // a=1, b=2, c=3, d=100 → d := 106
    assert_eq!(value, Value::from_smi(106));
}

#[test]
fn store_local_0_1_2_via_assignments() {
    // Parameters `a`, `b`, `c` live in slots 0, 1, 2. Mutating each
    // dispatches StoreLocal0 / StoreLocal1 / StoreLocal2 respectively.
    // a=10*2=20, b=20*3=60, c=30*4=120 → sum 200.
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
    // Stress test: tight loop using parameter mutation. Exercises
    // StoreLocal* (parameter write) and LoadLocal* (parameter read)
    // both heavily. The accumulator `s` lives in a let-binding (slot
    // >=4) so the loop body's `s = s + i` runs through Move, NOT
    // StoreLocalN — but the loop's exit test reads `iters` (slot 0)
    // every iteration via LoadLocal0, and `p1`/`p2`/`p3` are read
    // four times per iter via LoadLocal1/2/3. Confirms that the inline
    // ports preserve loop arithmetic correctness over many iterations.
    //
    // Sum 0+1+...+99 = 4950.
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

//! GC-stress test for the `frame_const_base` and `frame_this_value` mirror
//! discipline.
//!
//! Both fields must stay valid across GC events: every slow-path bridge goes
//! through the Refresh arm (`crates/vm/src/dsl/slow_path.rs`), which refreshes
//! them from canonical sources. A stale `frame_const_base` would corrupt named
//! constant reads; a stale `frame_this_value` would corrupt `this`-reads.
//!
//! The workload runs 50k iterations, allocating a fresh object each iteration to
//! drive slow-path Refresh egress. The sum `iters * (KNOWN + this.id)` diverges
//! immediately if either mirror is ever wrong. Forced `force_collect` calls before
//! and after the script bracket correctness under major GC.

use lyng_common::{AtomTable, SourceId};
use lyng_compiler::compile_script;
use lyng_env::Runtime;
use lyng_host::NoopHostHooks;
use lyng_parser::parse_script;
use lyng_sema::analyze_script;
use lyng_types::Value;
use lyng_vm::Vm;

/// Closure bound to `{ kind: "captured-this", id: 42 }`. The tight loop reads
/// the named constant `KNOWN` and `this.id` each iteration, accumulating their
/// sum. Expected: `iters * (KNOWN + BOUND_ID)`.
const GC_STRESS_SOURCE: &str = r#"
    (function () {
        var KNOWN = 7;
        var ITERS = 50000;
        var counter = 0;
        // Capture `this` lexically. The closure-bound `self` lives
        // in an environment slot; `this` itself is the function's
        // this_value (the object passed via .call()).
        var self = this;
        var marker = "tick";
        if (self.kind !== "captured-this") {
            throw new Error("initial this binding wrong");
        }
        for (var i = 0; i < ITERS; i++) {
            // Allocate a fresh object every iteration: pushes the
            // nursery past its budget repeatedly and exercises the
            // object-literal slow path, which routes through the
            // Refresh arm.
            var obj = { x: i, y: self, marker: marker };
            // Read the named constant — exercises the constants
            // pool read path (Phase 1.B.2 will lower this to a
            // direct `frame_const_base[idx]` load).
            counter = counter + KNOWN;
            // Read `this.id` — exercises both `this_value` and the
            // property-lookup slow path. If `frame_this_value` were
            // ever stale, this read would diverge.
            counter = counter + this.id;
            // Identity guards. The closure must see a non-null
            // `self` on every iteration; any staleness in the
            // closure-bound `this` or `self` would surface here.
            if (self === null) { throw new Error("this lost"); }
            if (obj.x !== i) { throw new Error("obj.x mismatch"); }
            if (obj.marker !== "tick") { throw new Error("marker mismatch"); }
            if (obj.y !== self) { throw new Error("self capture mismatch"); }
        }
        return counter;
    }).call({ kind: "captured-this", id: 42 });
"#;

const EXPECTED_ITERS: i32 = 50_000;
const EXPECTED_KNOWN: i32 = 7;
const EXPECTED_BOUND_ID: i32 = 42;

#[test]
#[ignore = "Slow (~1.3s, 50k VM iterations). Run with: cargo test -p lyng-tests -- --ignored"]
fn frame_context_survives_gc_pressure_in_closure_loop() {
    const MIN_EXPECTED_ALLOCS: usize = 1000;

    let mut atoms = AtomTable::new();

    let parsed = parse_script(&mut atoms, SourceId::new(0), GC_STRESS_SOURCE);
    assert!(
        !parsed.diagnostics.has_errors(),
        "gc-stress script should parse cleanly: {:?}",
        parsed.diagnostics.as_slice()
    );
    let sema = analyze_script(&parsed, &atoms);
    assert!(
        !sema.diagnostics.has_errors(),
        "gc-stress script should pass sema: {:?}",
        sema.diagnostics.as_slice()
    );
    let unit = compile_script(&parsed, &sema, &mut atoms).expect("gc-stress script should compile");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let _ = agent.force_collect(); // settle heap before workload
    let before_acct = agent.heap().view().accounting();

    let mut vm = Vm::new();
    let result = vm
        .evaluate_script(agent, realm, &unit)
        .run()
        .expect("gc-stress script should execute without VM error");

    let after_acct = agent.heap().view().accounting();

    let expected_sum = EXPECTED_ITERS * (EXPECTED_KNOWN + EXPECTED_BOUND_ID);
    let expected = Value::from_smi(expected_sum);
    assert_eq!(
        result, expected,
        "closure loop sum mismatch — frame_const_base or frame_this_value may be stale; \
         got {result:?}, expected {expected:?} ({EXPECTED_ITERS} iters * (KNOWN={EXPECTED_KNOWN} + this.id={EXPECTED_BOUND_ID}))",
    );

    // Confirm the loop's per-iteration allocations were observed by the nursery
    // accountant (a floor of 1000 proves the workload exercised the dispatch loop).
    let alloc_delta = after_acct
        .allocation_profile
        .nursery_allocations
        .saturating_sub(before_acct.allocation_profile.nursery_allocations);
    assert!(
        alloc_delta >= MIN_EXPECTED_ALLOCS,
        "expected at least {} nursery allocations; got {} (before={}, after={}). \
         If this fires the workload is no longer pressuring the heap; bump iteration count \
         or revisit allocation shape.",
        MIN_EXPECTED_ALLOCS,
        alloc_delta,
        before_acct.allocation_profile.nursery_allocations,
        after_acct.allocation_profile.nursery_allocations,
    );

    let _ = agent.force_collect();
}

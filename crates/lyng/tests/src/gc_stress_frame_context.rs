//! Phase 1.B.1 Task 7: GC-stress test for the `frame_const_base`
//! and `frame_this_value` mirror discipline.
//!
//! ## Hypothesis being tested
//!
//! The asm-visible mirror values that Phase 1.B.1 added to
//! `LlIntState` (`frame_const_base` and `frame_this_value`) stay
//! valid across GC events because every slow-path bridge that can
//! trigger GC also goes through the Refresh arm in
//! `crates/lyng/vm/src/dsl/slow_path.rs`, which refreshes both
//! fields from canonical sources (the active code record's
//! pre-resolved constants arena slot, and the active frame's
//! `this_value`).
//!
//! If the mirror discipline were broken — e.g. a slow-path that
//! triggered GC failed to refresh `frame_const_base` — then on the
//! next handler entry the asm would read through a stale arena
//! pointer and observe a moved or freed value. If `frame_this_value`
//! weren't refreshed after a `super()` mutation (or any other
//! slow-path that mutates `frame.this_value()`) then `this`-reading
//! handlers would observe stale identity.
//!
//! ## Test workload shape
//!
//! A JS closure that:
//!
//! - captures `this` via `Function.prototype.call({ kind, id })`
//!   (forces a known `this`-binding identity)
//! - in a tight loop:
//!   - reads a closure-captured `self = this` (would observe stale
//!     `frame_this_value` upstream if the mirror discipline were
//!     broken)
//!   - reads a function-scoped named constant `KNOWN` (a Smi in the
//!     constants pool, directly observable in the counter)
//!   - reads `this.id` (combines the `this`-binding read with a
//!     property lookup, exercising both `frame_this_value` and the
//!     property-lookup slow path)
//!   - allocates a fresh object on each iteration (forces nursery
//!     allocation pressure and exercises the object-literal slow
//!     path which egresses via Refresh)
//!   - accumulates `KNOWN + this.id` into a counter
//! - returns the counter on exit
//!
//! The test asserts the counter equals `iters * (KNOWN + BOUND_ID)`.
//! If `frame_const_base` were stale, the read of `KNOWN` would
//! observe a wrong value and the sum would drift. If
//! `frame_this_value` were stale, the read of `this.id` would
//! diverge (or the identity guard would throw).
//!
//! ## GC pressure mechanism
//!
//! The repo does not currently expose a `--cfg gc_stress` or
//! `force_minor_gc()` toggle that runs while a script is executing,
//! and the interpreter's mutator path does not attach
//! `ActiveVmRoots` to the heap mutator, so automatic minor GC does
//! not fire from inside a tight allocation loop. (Major-GC mark
//! slices DO get polled at LoopHeader safepoints.)
//!
//! What this test actually exercises:
//!
//! 1. **Slow-path Refresh discipline** — every iteration allocates a
//!    fresh `{ x: i, y: self, marker: "tick" }` object and reads a
//!    property; both go through Rust-level slow paths that route
//!    back through `translate_outcome`'s Refresh arm. Each Refresh
//!    egress re-writes `frame_const_base` and `frame_this_value`. If
//!    the refresh produced bogus values, the per-iteration reads of
//!    `KNOWN` (the named constant) or `self` (the captured `this`)
//!    would observe wrong values, corrupting the accumulator.
//! 2. **Forced collection bracketing** — explicit
//!    `agent.force_collect()` calls before and after script
//!    execution validate that the cross-frame trace path keeps
//!    everything reachable (the mirror writes don't accidentally
//!    alias a GC root).
//! 3. **High iteration count** (50,000 iterations) ensures the
//!    workload spends meaningful time inside the dispatch loop,
//!    forces frequent slow-path egress, and exhausts the default
//!    1 MiB nursery (driving up `young_live_bytes` even if no
//!    minor collection is triggered).
//!
//! ## Future stress modes
//!
//! When the repo adopts an in-loop force-GC hook (e.g. a
//! `--cfg gc_stress` flag that fires `force_minor_collect` on every
//! safepoint), this test should also be run under that flag:
//!
//! ```text
//! RUSTFLAGS="--cfg gc_stress" cargo test -p lyng-tests --release gc_stress_frame_context
//! ```
//!
//! The mirror-discipline invariant being tested is unchanged; only
//! the GC frequency increases.

use lyng_common::{AtomTable, SourceId};
use lyng_compiler::compile_script;
use lyng_env::Runtime;
use lyng_host::NoopHostHooks;
use lyng_parser::parse_script;
use lyng_sema::analyze_script;
use lyng_types::Value;
use lyng_vm::Vm;

/// Closure-with-captured-`this` workload.
///
/// The script:
/// - Builds an object `bound = { kind: "captured-this", id: 42 }`
///   and binds `this` to it via `Function.prototype.call`.
/// - Inside the closure, captures `this` to a closure-bound local
///   `self` (lexical capture exercises a separate read path from
///   the function's `this_value` field).
/// - In a tight loop, reads both `KNOWN` (named constant in the
///   function's constants pool) and `self`, allocates a fresh
///   object per iteration, and asserts identity guards on each.
///
/// `KNOWN` (7) lives in the function's constants array (Smi
/// constant in the pool). Loading it on every iteration is the
/// load that Phase 1.B.2's `op_load_const8` port will eventually
/// drive directly through `frame_const_base`. Until then this test
/// exercises the substrate populated by Phase 1.B.1 — the fields
/// are written at trampoline entry and on every Refresh egress,
/// and the surrounding suite confirms nothing else got broken.
///
/// The `id` field on `bound` lets us read `this.id` on each
/// iteration — that read goes through the property-lookup slow
/// path which egresses via Refresh, refreshing the mirrors.
///
/// `iters * KNOWN + iters * BOUND_ID` is the expected sum if the
/// mirror discipline holds. Any staleness in `frame_const_base`
/// (wrong KNOWN), `frame_this_value` (wrong `this` identity), or
/// the property-lookup path would corrupt the accumulator.
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
fn frame_context_survives_gc_pressure_in_closure_loop() {
    let mut atoms = AtomTable::new();

    // Parse, sema, compile.
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

    // Settle the heap before the workload so any heap-state delta
    // we observe is attributable to the loop, not to realm
    // bootstrap.
    let _ = agent.force_collect();
    let before_acct = agent.heap().view().accounting();

    let mut vm = Vm::new();
    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("gc-stress script should execute without VM error");

    let after_acct = agent.heap().view().accounting();

    // Primary assertion: the counter equals
    // iters * (KNOWN + BOUND_ID). Any staleness in
    // `frame_const_base` (wrong KNOWN), `frame_this_value` (wrong
    // this.id), or any interpreter mishap reading the constants
    // pool / `this`-binding would surface here.
    let expected_sum = EXPECTED_ITERS * (EXPECTED_KNOWN + EXPECTED_BOUND_ID);
    let expected = Value::from_smi(expected_sum);
    assert_eq!(
        result, expected,
        "closure loop sum mismatch — frame_const_base or frame_this_value may be stale; \
         got {:?}, expected {:?} ({} iters * (KNOWN={} + this.id={}))",
        result, expected, EXPECTED_ITERS, EXPECTED_KNOWN, EXPECTED_BOUND_ID,
    );

    // Cross-check the workload actually exercised the heap. We
    // can't reliably observe minor_collections because the JS
    // interpreter's mutator path does not currently attach
    // ActiveVmRoots — automatic minor GC fires only via explicit
    // force_collect. But we CAN confirm the loop's per-iteration
    // object allocations were observed by the nursery accountant:
    // `nursery_allocations` (in the allocation profile) increments
    // on every nursery-eligible allocation regardless of whether
    // GC ran. If this assertion fires the workload is no longer
    // pressuring the heap and the test loses its value.
    let alloc_delta = after_acct
        .allocation_profile
        .nursery_allocations
        .saturating_sub(before_acct.allocation_profile.nursery_allocations);
    // 50k iterations empirically produce ~15k nursery allocations
    // (compiler/runtime amortize some allocations). Pick a
    // conservative floor: 1000 allocations clearly indicates the
    // workload reached deep into the dispatch loop and the
    // Refresh-egress path ran many times.
    const MIN_EXPECTED_ALLOCS: usize = 1000;
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

    // Final stress check: a forced major collection after the
    // workload should not corrupt anything reachable. This
    // exercises the cross-frame trace_heap_edges path that
    // Phase 1.B.1 leaves unchanged — but if our mirror writes
    // accidentally aliased a GC root they would have surfaced as a
    // use-after-free here.
    let _ = agent.force_collect();
}

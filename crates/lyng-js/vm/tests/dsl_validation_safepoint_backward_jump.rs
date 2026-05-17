//! DSL-0b validation case 5 (design §10 / plan B34): safepoint poll
//! fires on a backward unconditional `op_jump`.
//!
//! ## Status: structural (Path A); runtime check deferred to Batch 7
//!
//! Conceptually identical to B33 (`op_loop_header` safepoint) but
//! exercises the warm-path `op_jump` handler instead. The same Batch 7
//! / Batch 8 dependencies apply — we need:
//!
//! 1. A working [`crate::dsl::entry::run_dsl_trampoline`] (still a
//!    stub in DSL-0b Batch 2).
//! 2. A real `op_jump` warm handler with `poll_safepoint!` on the
//!    backward-branch path (lands in Batch 7).
//! 3. The bytecode-builder API to emit a backward-jump loop without
//!    an enclosing `op_loop_header` (uses [`BytecodeBuilder`] today,
//!    but the runtime check is the gate).
//!
//! Until then we land a structural-only test plus an `#[ignore]`d
//! forward-pointer.

use lyng_js_vm::dsl::test_helpers::DslHarness;

#[test]
fn dsl_harness_bootstraps_for_backward_jump_safepoint_case() {
    let _harness = DslHarness::new();
}

#[test]
#[ignore = "Runtime trampoline + op_jump DSL handler required; enable when Batch 7 lands op_jump (plan B43/B44)."]
fn backward_jump_warm_path_polls_when_pending_set() {
    // Eventual shape (see plan B34 §step 1):
    //   - Build bytecode by hand: op_load_zero r0 ; op_loop_header ;
    //     op_add r0,r0,r1 ; op_jump -N (no op_loop_header).
    //   - Force VM.poll_pending = GC_PENDING before each dispatch.
    //   - Assert the slow-path poll counter increments ≥ 1.
    panic!("test ignored — see attribute reason");
}

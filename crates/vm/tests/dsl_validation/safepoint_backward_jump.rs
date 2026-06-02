//! DSL validation case: safepoint poll on a backward unconditional `op_jump`.
//!
//! Structural bootstrap test for the `op_jump` backward-branch safepoint path.

use lyng_vm::dsl::test_helpers::DslHarness;

#[test]
fn dsl_harness_bootstraps_for_backward_jump_safepoint_case() {
    let _harness = DslHarness::new();
}

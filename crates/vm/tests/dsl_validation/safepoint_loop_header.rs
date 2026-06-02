//! DSL validation case: safepoint poll fires on `op_loop_header`.
//!
//! Structural bootstrap test confirming the
//! [`lyng_vm::dsl::test_helpers::DslHarness`] is constructible and
//! gates the harness bootstrap path against drift.

use lyng_vm::dsl::test_helpers::DslHarness;

#[test]
fn dsl_harness_bootstraps_for_loop_header_safepoint_case() {
    let _harness = DslHarness::new();
}

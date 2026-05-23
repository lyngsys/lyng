//! DSL-0b validation case 6 (design §10 / plan B35): safepoint poll
//! fires on a backward **conditional** jump (`op_jump_if_true8` /
//! `op_jump_if_false8` taking the backward branch).
//!
//! ## Status: structural (Path A); runtime check deferred to Batch 7
//!
//! Same deferral reasoning as B33/B34. The eventual runtime test
//! checks that a hot loop using a conditional backward jump still
//! polls the GC at least once when `poll_pending` is forced. The
//! warm-path conditional-jump handler lands in Batch 7.

use lyng_vm::dsl::test_helpers::DslHarness;

#[test]
fn dsl_harness_bootstraps_for_backward_cond_jump_safepoint_case() {
    let _harness = DslHarness::new();
}

#[test]
#[ignore = "Runtime trampoline + op_jump_if_true8 / op_jump_if_false8 DSL handlers required; enable when Batch 7 lands them (plan B43/B44)."]
fn backward_conditional_jump_warm_path_polls_when_pending_set() {
    // Eventual shape (see plan B35):
    //   - Build bytecode with a tight loop whose backward branch is
    //     conditional (`op_jump_if_true8 -N` or `op_jump_if_false8 -N`).
    //   - Force `VM.poll_pending = GC_PENDING`.
    //   - Assert poll counter increments ≥ 1 over the loop's lifetime.
    panic!("test ignored — see attribute reason");
}

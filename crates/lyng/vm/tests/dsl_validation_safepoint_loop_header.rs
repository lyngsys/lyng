//! DSL-0b validation case 4 (design §10 / plan B33): safepoint poll
//! fires on `op_loop_header`.
//!
//! ## Status: structural (Path A); runtime check deferred to Batch 7
//!
//! The full assertion ("tight loop of `op_add` + `op_loop_header` with
//! `Vm.poll_pending = GC_PENDING` polls the GC at least once") requires:
//!
//! 1. A working [`crate::dsl::entry::run_dsl_trampoline`] — currently a
//!    `naked_asm!("ret")` stub (DSL-0b Batch 2; the body lands later).
//! 2. A real `op_loop_header` handler with `poll_safepoint!` invoked
//!    against the live Vm — landed in Batch 7 (`op_loop_header` warm
//!    port, see plan B43).
//! 3. A `compile_and_run_with_poll_forced` test helper — lands with
//!    Batch 7/8 too, since it's only useful once (1) + (2) work.
//!
//! The DSL-0b validation-case skeleton documents the eventual shape;
//! the `#[ignore]`d test runs end-to-end once the dependencies are in
//! place.
//!
//! For now we land:
//!
//! - A structural test that the [`lyng_js_vm::dsl::test_helpers::DslHarness`]
//!   exists and is constructible (gates the harness's bootstrap path
//!   against drift).
//! - The `#[ignore]`d end-to-end test as a forward-pointer.

use lyng_js_vm::dsl::test_helpers::DslHarness;

#[test]
fn dsl_harness_bootstraps_for_loop_header_safepoint_case() {
    // Construction-only floor: the harness's `new()` exercises
    // Runtime + Vm + install_script + InstalledFunction lookup —
    // any drift in those signatures or the bootstrap script's
    // compileability surfaces here before Batch 7 lands the real
    // safepoint check.
    let _harness = DslHarness::new();
}

#[test]
#[ignore = "Runtime trampoline + op_loop_header DSL handler required; enable when Batch 7 lands op_loop_header (plan B43)."]
fn loop_header_warm_path_polls_when_pending_set() {
    // Skeleton of the eventual runtime assertion (kept as a
    // forward-pointer; the body is intentionally short until the
    // dependencies — `run_dsl_trampoline` + `op_loop_header` warm
    // handler + `compile_and_run_with_poll_forced` — exist).
    //
    // Pseudo-code:
    //   let result = lyng_js_vm::test_helpers::compile_and_run_with_poll_forced(
    //       "let i = 0; for (let j = 0; j < 100; j += 1) { i += j }",
    //   );
    //   assert!(result.poll_fired_count >= 1);
    panic!("test ignored — see attribute reason");
}

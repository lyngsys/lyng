//! DSL-0b validation case 9 (design §10 / plan B38): a stacked prefix
//! (e.g. `Wide` followed by `Wide`) is rejected with
//! [`VmError::DoublePrefix`].
//!
//! ## What this exercises
//!
//! [`crate::vm::semantics::prefix::op_wide_semantic`] guards against
//! double-prefix bytecode by checking
//! `state.dispatch_state().prefix.is_some()` on entry. The bytecode
//! emitter never produces this — encountering it indicates a corrupted
//! instruction stream — so the semantic returns
//! `SemanticOutcome::ExitError { error: VmError::DoublePrefix { .. } }`.
//!
//! The test drives the semantic via the harness's α-variant
//! `with_alpha_dispatch` helper because the rejection logic reads
//! `state.dispatch_state().prefix`, which is only populated on the
//! Alpha variant of [`crate::dsl::slow_path::LlIntDispatchState`].
//! (The asm-variant prefix lives in `state.prefix` on `LlIntState`
//! and uses a different code path that lands in Batch 7.)
//!
//! ## Why this is runnable today
//!
//! `op_wide_semantic` is a normal Rust function that operates on a
//! synthesized `DispatchState`. The harness constructs one with the
//! `prefix` field pre-seeded to `Some(Opcode::Wide)`, and the
//! semantic's first branch produces the `DoublePrefix` error without
//! ever touching the asm trampoline or running any handler chain.

use lyng_js_bytecode::Opcode;
use lyng_js_vm::dsl::slow_path::SemanticOutcome;
use lyng_js_vm::dsl::test_helpers::prefix_semantics::{
    invoke_extra_wide_semantic_via_dsl_harness, invoke_wide_semantic_via_dsl_harness,
};
use lyng_js_vm::dsl::test_helpers::DslHarness;
use lyng_js_vm::VmError;

#[test]
fn op_wide_followed_by_op_wide_raises_double_prefix() {
    let mut harness = DslHarness::new();
    let outcome = harness.with_alpha_dispatch(Some(Opcode::Wide), |state| {
        invoke_wide_semantic_via_dsl_harness(state)
    });
    match outcome {
        SemanticOutcome::ExitError { error } => {
            assert!(
                matches!(error, VmError::DoublePrefix { .. }),
                "expected DoublePrefix, got {error:?}",
            );
        }
        other => panic!("expected ExitError, got {other:?}"),
    }
}

#[test]
fn op_wide_followed_by_op_extra_wide_raises_double_prefix() {
    let mut harness = DslHarness::new();
    let outcome = harness.with_alpha_dispatch(Some(Opcode::Wide), |state| {
        invoke_extra_wide_semantic_via_dsl_harness(state)
    });
    match outcome {
        SemanticOutcome::ExitError { error } => {
            assert!(matches!(error, VmError::DoublePrefix { .. }));
        }
        other => panic!("expected ExitError, got {other:?}"),
    }
}

#[test]
fn op_extra_wide_followed_by_op_wide_raises_double_prefix() {
    let mut harness = DslHarness::new();
    let outcome = harness.with_alpha_dispatch(Some(Opcode::ExtraWide), |state| {
        invoke_wide_semantic_via_dsl_harness(state)
    });
    match outcome {
        SemanticOutcome::ExitError { error } => {
            assert!(matches!(error, VmError::DoublePrefix { .. }));
        }
        other => panic!("expected ExitError, got {other:?}"),
    }
}

#[test]
fn op_wide_alone_records_prefix_and_continues() {
    // Sanity twin: when there's no pre-existing prefix the semantic
    // should set `state.prefix = Some(Wide)` and return Continue with
    // `pc_advance = 0`. This protects against accidentally inverting
    // the guard condition during a future refactor.
    let mut harness = DslHarness::new();
    let outcome = harness.with_alpha_dispatch(None, |state| {
        invoke_wide_semantic_via_dsl_harness(state)
    });
    match outcome {
        SemanticOutcome::Continue { pc_advance } => {
            assert_eq!(pc_advance, 0, "prefix should not advance PC");
        }
        other => panic!("expected Continue, got {other:?}"),
    }
}

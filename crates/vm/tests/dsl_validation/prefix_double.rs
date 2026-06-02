//! DSL validation: a stacked prefix is rejected with [`VmError::DoublePrefix`].
//!
//! [`crate::vm::semantics::prefix::op_wide_semantic`] guards against
//! double-prefix bytecode by checking `state.dispatch_state().prefix.is_some()`
//! on entry. The bytecode emitter never produces this; encountering it
//! indicates a corrupted instruction stream.

use lyng_bytecode::Opcode;
use lyng_vm::VmError;
use lyng_vm::dsl::slow_path::SemanticOutcome;
use lyng_vm::dsl::test_helpers::DslHarness;
use lyng_vm::dsl::test_helpers::prefix_semantics::{
    invoke_extra_wide_semantic_via_dsl_harness, invoke_wide_semantic_via_dsl_harness,
};

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
    // Without a pre-existing prefix the semantic sets
    // `state.prefix = Some(Wide)` and returns Continue with pc_advance = 0.
    let mut harness = DslHarness::new();
    let outcome = harness.with_alpha_dispatch(None, invoke_wide_semantic_via_dsl_harness);
    match outcome {
        SemanticOutcome::Continue { pc_advance } => {
            assert_eq!(pc_advance, 0, "prefix should not advance PC");
        }
        other => panic!("expected Continue, got {other:?}"),
    }
}

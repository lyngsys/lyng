//! DSL validation: each [`SemanticOutcome`] variant round-trips correctly
//! through [`crate::dsl::slow_path::LlIntDispatchState::translate_outcome`].
//!
//! The four outcomes (`Continue`, `Refresh`, `ExitDone`, `ExitError`) each
//! become a [`SlowPathReturn`] via `translate_outcome`. The harness drives
//! the bridge directly without involving the asm trampoline.

use lyng_types::Value;
use lyng_vm::VmError;
use lyng_vm::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use lyng_vm::dsl::test_helpers::{DslHarness, HarnessOutcome};

// Each semantic is an inline closure; in production these are
// `pub(crate) fn op_xxx_semantic(...)` referenced by `dsl_cold_shim!`.

#[test]
fn continue_outcome_round_trips() {
    let mut harness = DslHarness::new();
    let outcome = harness
        .invoke_semantic_directly(0x10, |_state| SemanticOutcome::Continue { pc_advance: 4 });
    match outcome {
        HarnessOutcome::Continued { new_pc_offset } => {
            // entry PC 0x10 + pc_advance 4 = 0x14.
            assert_eq!(new_pc_offset, 0x14);
        }
        other => panic!("expected Continued, got {other:?}"),
    }
}

#[test]
fn refresh_outcome_round_trips() {
    let mut harness = DslHarness::new();
    let outcome = harness.invoke_semantic_directly(0x20, |_state| SemanticOutcome::Refresh);
    match outcome {
        HarnessOutcome::Refreshed => {}
        other => panic!("expected Refreshed, got {other:?}"),
    }
}

#[test]
fn exit_done_outcome_round_trips() {
    let mut harness = DslHarness::new();
    let outcome = harness.invoke_semantic_directly(0, |_state| SemanticOutcome::ExitDone {
        value: Value::from_smi(7),
    });
    match outcome {
        HarnessOutcome::Done { value } => {
            assert_eq!(value, Value::from_smi(7));
        }
        other => panic!("expected Done, got {other:?}"),
    }
}

#[test]
fn exit_error_outcome_round_trips() {
    let mut harness = DslHarness::new();
    let outcome = harness.invoke_semantic_directly(0, |_state| SemanticOutcome::ExitError {
        error: VmError::TrampolineExitedWithoutSetting,
    });
    match outcome {
        HarnessOutcome::Error { error } => {
            assert!(matches!(*error, VmError::TrampolineExitedWithoutSetting));
        }
        other => panic!("expected Error, got {other:?}"),
    }
}

// Documents the closure shape `invoke_semantic_directly` expects.
const fn _semantic_signature_witness(_state: &mut LlIntDispatchState<'_, '_>) -> SemanticOutcome {
    SemanticOutcome::Continue { pc_advance: 0 }
}

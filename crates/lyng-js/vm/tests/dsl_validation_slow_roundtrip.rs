//! DSL-0b validation case 2 (design §10 / plan B31): each slow-path
//! [`SemanticOutcome`] variant round-trips correctly through the
//! [`crate::dsl::slow_path::LlIntDispatchState::translate_outcome`]
//! ABI.
//!
//! ## What this exercises
//!
//! The Batch-2 slow-path bridge ([`crate::dsl::slow_path`]) defines
//! four logical outcomes that a semantic body can return — `Continue`,
//! `Refresh`, `ExitDone`, `ExitError`. Each becomes a
//! [`SlowPathReturn`] via `translate_outcome`. The asm trampoline (a
//! `naked_asm!("ret")` stub in DSL-0b) reads the tag and either
//! re-dispatches, refreshes pinned registers, or branches to the exit
//! shim.
//!
//! The test drives the same path the asm-emitted shims will use
//! (build dispatch state via `from_raw`, `sync_from_asm`, call
//! semantic, `translate_outcome`) and asserts the harness's decoded
//! [`HarnessOutcome`] matches the variant the semantic returned.
//!
//! ## Why this is runnable today (Path A on the runtime-runnable side)
//!
//! `translate_outcome` only reads `state.frame_pc_offset` and
//! `rust.frame`/`rust.exit`. None of those depend on a real running
//! handler chain — the harness builds them directly. The naked
//! trampoline is not invoked.

use lyng_js_types::Value;
use lyng_js_vm::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use lyng_js_vm::dsl::test_helpers::{DslHarness, HarnessOutcome};
use lyng_js_vm::VmError;

// Each semantic is a closure passed to `invoke_semantic_directly`. We
// keep them inline so the test reads top-to-bottom; in production code
// these would be `pub(crate) fn op_xxx_semantic(...)` functions
// referenced by `dsl_cold_shim!`.

#[test]
fn continue_outcome_round_trips() {
    let mut harness = DslHarness::new();
    let outcome = harness.invoke_semantic_directly(0x10, |_state| {
        SemanticOutcome::Continue { pc_advance: 4 }
    });
    match outcome {
        HarnessOutcome::Continued { new_pc_offset } => {
            // `translate_outcome` sets `state.frame_pc_offset` to
            // `rust.frame.instruction_offset().wrapping_add(pc_advance)`.
            // After `sync_from_asm` the rust frame's offset is the
            // entry PC (0x10), so the new offset is 0x10 + 4 = 0x14.
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

// Touch the import explicitly so an unused-import warning doesn't
// fire on toolchains that infer harder. The signature also documents
// the closure shape `invoke_semantic_directly` expects.
fn _semantic_signature_witness(_state: &mut LlIntDispatchState<'_, '_>) -> SemanticOutcome {
    SemanticOutcome::Continue { pc_advance: 0 }
}

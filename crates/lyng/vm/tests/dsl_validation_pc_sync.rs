//! DSL-0b validation case 3 (design §6 / plan B32): pre-slow-path PC
//! sync correctness.
//!
//! ## What this exercises
//!
//! The asm trampoline keeps a per-frame PC offset in
//! `state.frame_pc_offset` (asm-visible mirror) while the Rust side
//! keeps the canonical PC in `rust.frame.instruction_offset()`. Before
//! a slow path runs, the bridge must call
//! [`crate::dsl::slow_path::LlIntDispatchState::sync_from_asm`] so the
//! semantic body sees the post-dispatch PC, not stale data.
//!
//! The test sets `state.frame_pc_offset = 0x42`, invokes the harness
//! (which calls `sync_from_asm` before the semantic), and reads back
//! `state.current_instruction_offset()` inside the semantic — which
//! on the asm variant delegates to `rust.frame.instruction_offset()`.
//! If the sync did the right thing, the semantic observes `0x42`.
//!
//! ## Why this is runnable today
//!
//! Same reasoning as B31: `sync_from_asm` only touches
//! `state.frame_pc_offset` (read) and `rust.frame` (write). No
//! trampoline involvement; the harness's
//! [`invoke_semantic_directly`](`lyng_js_vm::dsl::test_helpers::DslHarness::invoke_semantic_directly`)
//! drives the bridge directly.

use std::cell::Cell;

use lyng_js_vm::dsl::slow_path::SemanticOutcome;
use lyng_js_vm::dsl::test_helpers::{DslHarness, HarnessOutcome};

#[test]
fn semantic_body_sees_post_dispatch_pc() {
    let mut harness = DslHarness::new();

    // Capture the observed PC out of the semantic closure. A Cell
    // sidesteps the FnOnce-vs-FnMut decision because we only write
    // once.
    let observed = Cell::new(u32::MAX);
    let outcome = harness.invoke_semantic_directly(0x42, |state| {
        observed.set(state.current_instruction_offset());
        // Returning Continue with pc_advance = 4 also exercises the
        // post-translate offset write, which the harness reports
        // back as `new_pc_offset`.
        SemanticOutcome::Continue { pc_advance: 4 }
    });

    assert_eq!(
        observed.get(),
        0x42,
        "semantic body should observe entry PC after sync_from_asm",
    );

    match outcome {
        HarnessOutcome::Continued { new_pc_offset } => {
            // 0x42 + 4 = 0x46. Confirms `translate_outcome` advanced
            // the asm-side mirror by `pc_advance` from the synced PC.
            assert_eq!(new_pc_offset, 0x46);
        }
        other => panic!("expected Continued, got {other:?}"),
    }
}

#[test]
fn semantic_body_sees_zero_pc_when_entry_is_zero() {
    // Sanity twin of the 0x42 case — the synced PC should reflect
    // whatever value was placed in `state.frame_pc_offset`, even
    // the boundary value 0.
    let mut harness = DslHarness::new();
    let observed = Cell::new(u32::MAX);
    let _ = harness.invoke_semantic_directly(0, |state| {
        observed.set(state.current_instruction_offset());
        SemanticOutcome::Continue { pc_advance: 0 }
    });
    assert_eq!(observed.get(), 0);
}

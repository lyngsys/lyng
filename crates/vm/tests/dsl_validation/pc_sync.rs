//! DSL validation: pre-slow-path PC sync correctness.
//!
//! The asm trampoline keeps a per-frame PC offset in `state.frame_pc_offset`
//! while the Rust side keeps the canonical PC in
//! `rust.frame.instruction_offset()`. Before a slow path runs the bridge
//! calls `sync_from_asm` so the semantic sees the post-dispatch PC.
//!
//! The test sets `state.frame_pc_offset = 0x42`, invokes the harness (which
//! calls `sync_from_asm`), and asserts the semantic observes `0x42`.

use std::cell::Cell;

use lyng_vm::dsl::slow_path::SemanticOutcome;
use lyng_vm::dsl::test_helpers::{DslHarness, HarnessOutcome};

#[test]
fn semantic_body_sees_post_dispatch_pc() {
    let mut harness = DslHarness::new();

    // Capture the observed PC; Cell sidesteps the FnOnce-vs-FnMut decision.
    let observed = Cell::new(u32::MAX);
    let outcome = harness.invoke_semantic_directly(0x42, |state| {
        observed.set(state.current_instruction_offset());
        // Continue with pc_advance = 4 exercises the post-translate offset write.
        SemanticOutcome::Continue { pc_advance: 4 }
    });

    assert_eq!(
        observed.get(),
        0x42,
        "semantic body should observe entry PC after sync_from_asm",
    );

    match outcome {
        HarnessOutcome::Continued { new_pc_offset } => {
            // 0x42 + 4 = 0x46: translate_outcome advanced the mirror by pc_advance.
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

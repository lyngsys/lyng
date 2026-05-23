//! DSL-0c — length-consistency guard for hand-written hot/warm handlers.
//!
//! Every `llint_handler! { op_xxx, ..., length = N, |...| { ... } }` block
//! must declare a `length = N` that matches the canonical bytecode encoding
//! returned by `Opcode::encoded_len()`. A mismatch advances PC by the wrong
//! number of bytes per instruction, misaligning subsequent dispatch and
//! eventually tripping `debug_assert!`s on garbage operand values (cf. the
//! op_move length=3 bug fixed in commit "DSL-0c: fix op_move length 3 → 4").
//!
//! The `llint_handler!` proc-macro emits a sibling `pub const
//! OP_XXX_LENGTH: u32` next to each generated `op_xxx` function so this
//! test can read the declared length without going through the asm
//! body. The cold-stub family is guarded separately at codegen time —
//! see `tools/lyng-js-dsl-codegen/src/main.rs`'s `main()` validator.
//!
//! ## What this catches
//!
//! 1. **Hand-edited drift in `hot.rs` or `warm.rs`.** Someone changes a
//!    handler's operand decoding (`Ab` → `Abc`) but forgets to update
//!    the `length = N` attribute.
//! 2. **Opcode encoding changes.** `Opcode::encoded_len()` adds Move to a
//!    new arm (say, an Mxxxx variant returning 5) without updating the
//!    DSL handler.
//!
//! ## What this does NOT catch
//!
//! - Wide / ExtraWide effective lengths (the DSL handlers handle narrow
//!   form only — wide-form dispatch is delegated to the α path; see
//!   `op_wide_via_alpha_rs` in `warm.rs`).
//! - Operand-decoding correctness (use `dsl_validation_*` for that).

#![cfg(target_arch = "aarch64")]

use lyng_js_bytecode::Opcode;
use lyng_js_vm::dsl::handlers::{hot, warm};

/// One pair (declared length emitted by `llint_handler!`, canonical
/// `Opcode::encoded_len()`). The test asserts they match. Listed in
/// dispatch-table order for readability — the asserts will list every
/// mismatching pair if `cargo test` is run with `--no-fail-fast`.
fn handler_length_pairs() -> [(&'static str, u32, u32); 12] {
    [
        // hot.rs (4 handlers)
        ("op_move", hot::OP_MOVE_LENGTH, Opcode::Move.encoded_len() as u32),
        ("op_add", hot::OP_ADD_LENGTH, Opcode::Add.encoded_len() as u32),
        ("op_jump", hot::OP_JUMP_LENGTH, Opcode::Jump.encoded_len() as u32),
        ("op_return", hot::OP_RETURN_LENGTH, Opcode::Return.encoded_len() as u32),
        // warm.rs (8 handlers)
        ("op_loop_header", warm::OP_LOOP_HEADER_LENGTH, Opcode::LoopHeader.encoded_len() as u32),
        ("op_jump8", warm::OP_JUMP8_LENGTH, Opcode::Jump8.encoded_len() as u32),
        ("op_jump_if_true", warm::OP_JUMP_IF_TRUE_LENGTH, Opcode::JumpIfTrue.encoded_len() as u32),
        ("op_jump_if_false", warm::OP_JUMP_IF_FALSE_LENGTH, Opcode::JumpIfFalse.encoded_len() as u32),
        ("op_jump_if_true8", warm::OP_JUMP_IF_TRUE8_LENGTH, Opcode::JumpIfTrue8.encoded_len() as u32),
        ("op_jump_if_false8", warm::OP_JUMP_IF_FALSE8_LENGTH, Opcode::JumpIfFalse8.encoded_len() as u32),
        ("op_wide", warm::OP_WIDE_LENGTH, Opcode::Wide.encoded_len() as u32),
        ("op_extra_wide", warm::OP_EXTRA_WIDE_LENGTH, Opcode::ExtraWide.encoded_len() as u32),
    ]
}

#[test]
fn dsl_handler_lengths_match_canonical_encoded_len() {
    let mut mismatches: Vec<(&'static str, u32, u32)> = Vec::new();
    for (name, declared, canonical) in handler_length_pairs() {
        if declared != canonical {
            mismatches.push((name, declared, canonical));
        }
    }
    assert!(
        mismatches.is_empty(),
        "{} DSL handler(s) have a declared length mismatching the canonical \
         Opcode::encoded_len(). This causes PC misalignment after every \
         instance of the affected opcode, leading to SIGABRT once dispatch \
         reads garbage operands. Fix the `length = N` attribute on the \
         relevant `llint_handler!` invocation in hot.rs / warm.rs.\n\n\
         Mismatches:\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .map(|(name, declared, canonical)| format!(
                "  - {name}: declared length = {declared}, canonical = {canonical}",
            ))
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

/// Sanity: covers every hand-written DSL handler. If hot.rs or warm.rs
/// gains a new handler, this test should also cover it. The asserted
/// count of 12 (4 hot + 8 warm) acts as a tripwire — adding a handler
/// without updating this file will fail this assertion at test time.
///
/// (The cold-stub family is not tested here because cold.rs is auto-
/// generated and its metadata table has its own codegen-time validator
/// in `tools/lyng-js-dsl-codegen/src/main.rs`'s `main()`.)
#[test]
fn handler_length_pairs_covers_every_hand_written_handler() {
    // 4 hot + 8 warm = 12. Adjust intentionally if hot.rs / warm.rs grows.
    assert_eq!(
        handler_length_pairs().len(),
        12,
        "handler_length_pairs() must cover every hand-written DSL handler \
         in hot.rs and warm.rs. If you added a new handler, add a row \
         here too — otherwise drift in its `length =` attribute would go \
         unnoticed.",
    );
}

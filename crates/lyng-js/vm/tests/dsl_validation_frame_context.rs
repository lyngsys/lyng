//! Phase 1.B.1 Task 6 (+ Phase 1.B.2 cleanup): validate that the
//! `load_constant!`, `load_state_value!`, and `load_uninit_lex_sentinel!`
//! backend macros compile and link cleanly inside the `llint_handler!`
//! proc-macro lowering pipeline.
//!
//! ## Phase 1.B.2 cleanup
//!
//! The 3 forward-pointer `#[ignore]`-d tests that were placeholders for
//! Phase 1.B.2's canonical opcodes are removed. End-to-end coverage of
//! `op_load_const8` and `op_load_this` now lives in
//! `crates/lyng-js-tests/tests/op_load_const8_inline.rs` and
//! `crates/lyng-js-tests/tests/op_load_this_inline.rs` respectively.
//!
//! The 4 structural compiles-and-links tests remain — they catch
//! macro-emit and lowerer-binding regressions before they hit
//! production handlers.
//!
//! ## Scope
//!
//! Four synthetic `llint_handler!` invocations exercise the macros in
//! the shapes Phase 1.B.2 actually uses:
//!
//! - `op_test_load_constant_dsl` (opcode_byte = 210): use an internal
//!   scratch slot as a fake index and read
//!   `frame_const_base[fake_idx]` into another scratch via
//!   `load_constant!`. Exercises the 2-instruction indexed-load shape
//!   that Phase 1.B.2's `op_load_const8` adopts.
//! - `op_test_load_this_value_dsl` (opcode_byte = 211): read
//!   `frame_this_value` into a scratch via
//!   `load_state_value!(... vm_state_offset = state_this_value)`.
//!   Exercises the 1-instruction fixed-offset load shape that Phase
//!   1.B.2's `op_load_this` adopts.
//! - `op_test_load_this_sentinel_dsl` (opcode_byte = 212): same call
//!   shape as the previous handler. The sentinel-vs-real-value
//!   distinction is decided at trampoline-entry / Refresh time by the
//!   `resolve_initial_this_value` helper, not by the macro itself.
//!   Kept distinct so the asm-DSL pipeline is exercised twice with the
//!   same body — catches any register-allocation or label-prefix drift
//!   between identical bodies in the same translation unit.
//! - `op_test_load_uninit_lex_sentinel_dsl` (opcode_byte = 213):
//!   materialize `Value::uninitialized_lexical()` into a scratch via
//!   `load_uninit_lex_sentinel!`. Exercises the 4-instruction
//!   movz/movk sentinel-materialization shape that Phase 1.B.2's
//!   `op_load_this` uses for its sentinel-bail compare.
//!
//! ## Why these tests are structural (link-time) only
//!
//! The existing `dsl_validation_*.rs` family (e.g.
//! `dsl_validation_empty.rs`, `dsl_validation_prefix_wide.rs`) takes
//! the same approach: each test calls
//! [`DslHarness::assert_handler_symbol_exists`] on an
//! `llint_handler!`-expanded function pointer. Reasoning:
//!
//! 1. The asm trampoline ([`run_dsl_trampoline`]) only dispatches via
//!    `DSL_DISPATCH_TABLE`, which is populated by the `handlers/` module
//!    and routes unknown opcodes to `unimplemented_dsl_handler`.
//! 2. To execute a synthetic opcode end-to-end would require either
//!    (a) reaching into the dispatch table from a test (the table is
//!    `pub(crate)`-scoped behind `dsl::handlers`), or (b) installing a
//!    real bytecode program whose opcodes match the synthetic IDs —
//!    which can't go through the compiler pipeline without registering
//!    them in `lyng-js-bytecode::Opcode`.
//!
//! End-to-end coverage now lives in the per-opcode integration test
//! files in `lyng-js-tests`; these structural tests are kept for the
//! macro-emit / lowerer-binding regression-catching role they play.
//!
//! ## What this file proves
//!
//! - The macros parse correctly under `macro_rules!`.
//! - The lowerer's universal `vm_const_base` / `state_this_value` /
//!   `value_uninit_lex_bits` bindings are wired correctly (no
//!   "unbound named arg" errors).
//! - The emitted asm assembles cleanly (no "invalid operand" /
//!   "constant out of range" from the assembler).
//! - The resulting `extern "C" fn` symbols are addressable at runtime.

#[cfg(target_arch = "aarch64")]
use lyng_js_vm::{dispatch, load_constant, load_state_value, load_uninit_lex_sentinel};
#[cfg(target_arch = "aarch64")]
use lyng_js_vm_dsl::llint_handler;

// `layout = None` keeps the prologue empty so the body only references
// the macros under test. `t0` / `t1` are internal scratch slots that
// the lowerer's `ScratchAllocator` lazily allocates to x9/x10 on first
// reference. The contents of `t0` are undefined at handler entry, but
// the assembler only cares that the operand register numbers parse —
// behavioural correctness of the load is exercised by Phase 1.B.2's
// real ports.
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_test_load_constant_dsl, opcode_byte = 210, layout = None, length = 1, || {
        load_constant!(t0 => t1);
        dispatch!(advance = 0);
    }
}

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_test_load_this_value_dsl, opcode_byte = 211, layout = None, length = 1, || {
        load_state_value!(t0, vm_state_offset = state_this_value);
        dispatch!(advance = 0);
    }
}

// Same call shape as `op_test_load_this_value_dsl` but a distinct
// handler symbol so the asm-DSL pipeline is exercised twice with the
// same body. Catches any register-allocation or label-prefix drift
// between identical bodies in the same translation unit.
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_test_load_this_sentinel_dsl, opcode_byte = 212, layout = None, length = 1, || {
        load_state_value!(t0, vm_state_offset = state_this_value);
        dispatch!(advance = 0);
    }
}

// Phase 1.B.2: exercise `load_uninit_lex_sentinel!` end-to-end through
// the lowerer + `naked_asm!`. Opcode 213 extends the 210/211/212 range
// used by the prior synthetic handlers. The macro emits 4 instructions
// (movz + 3× movk) that materialize `VALUE_UNINIT_LEX_BITS` into the
// destination scratch.
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_test_load_uninit_lex_sentinel_dsl, opcode_byte = 213, layout = None, length = 1, || {
        load_uninit_lex_sentinel!(t0);
        dispatch!(advance = 0);
    }
}

#[cfg(target_arch = "aarch64")]
#[test]
fn load_constant_handler_compiles_and_links() {
    use lyng_js_vm::dsl::test_helpers::DslHarness;
    DslHarness::assert_handler_symbol_exists(op_test_load_constant_dsl);
}

#[cfg(target_arch = "aarch64")]
#[test]
fn load_this_value_handler_compiles_and_links() {
    use lyng_js_vm::dsl::test_helpers::DslHarness;
    DslHarness::assert_handler_symbol_exists(op_test_load_this_value_dsl);
}

#[cfg(target_arch = "aarch64")]
#[test]
fn load_this_sentinel_handler_compiles_and_links() {
    use lyng_js_vm::dsl::test_helpers::DslHarness;
    DslHarness::assert_handler_symbol_exists(op_test_load_this_sentinel_dsl);
}

#[cfg(target_arch = "aarch64")]
#[test]
fn load_uninit_lex_sentinel_handler_compiles_and_links() {
    use lyng_js_vm::dsl::test_helpers::DslHarness;
    DslHarness::assert_handler_symbol_exists(op_test_load_uninit_lex_sentinel_dsl);
}

// On non-aarch64 hosts the backend macros aren't compiled in, so
// skip the validation cases entirely (mirrors `dsl_validation_empty.rs`).
#[cfg(not(target_arch = "aarch64"))]
#[test]
fn load_constant_handler_compiles_and_links() {
    // No-op on non-aarch64; the asm-DSL backend isn't compiled here.
}

#[cfg(not(target_arch = "aarch64"))]
#[test]
fn load_this_value_handler_compiles_and_links() {
    // No-op on non-aarch64; the asm-DSL backend isn't compiled here.
}

#[cfg(not(target_arch = "aarch64"))]
#[test]
fn load_this_sentinel_handler_compiles_and_links() {
    // No-op on non-aarch64; the asm-DSL backend isn't compiled here.
}

#[cfg(not(target_arch = "aarch64"))]
#[test]
fn load_uninit_lex_sentinel_handler_compiles_and_links() {
    // No-op on non-aarch64; the asm-DSL backend isn't compiled here.
}


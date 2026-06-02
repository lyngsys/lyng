//! DSL validation: `load_constant!`, `load_state_value!`, and
//! `load_uninit_lex_sentinel!` compile and link inside the `llint_handler!`
//! lowering pipeline.
//!
//! Structural (link-time only) tests — end-to-end dispatch coverage lives in
//! `lyng-tests`. These catch macro-emit and lowerer-binding regressions.
//!
//! ## Scope
//!
//! Synthetic `llint_handler!` invocations (opcodes 210–215) exercise the
//! macros in the shapes used by production handlers:
//!
//! - 210 `op_test_load_constant_dsl`: `load_constant!` indexed-load shape.
//! - 211 `op_test_load_this_value_dsl`: `load_state_value!` fixed-offset load.
//! - 212 `op_test_load_this_sentinel_dsl`: identical body to 211, exercising
//!   register-allocation / label-prefix with a duplicate body.
//! - 213 `op_test_load_uninit_lex_sentinel_dsl`: `load_uninit_lex_sentinel!`
//!   4-instruction movz/movk materialization.
//! - 214 `op_test_load_local_fixed_dsl`: `load_local_fixed!` fixed-immediate form.
//! - 215 `op_test_store_local_fixed_dsl`: `store_local_fixed!` fixed-immediate form.

#[cfg(target_arch = "aarch64")]
use lyng_vm::{
    dispatch, load_constant, load_local_fixed, load_state_value, load_uninit_lex_sentinel,
    store_local_fixed,
};
#[cfg(target_arch = "aarch64")]
use lyng_vm_dsl::llint_handler;

// `layout = None` keeps the prologue empty. `t0` / `t1` are scratch slots
// allocated by the lowerer. Contents of `t0` are undefined at handler entry;
// behavioural correctness is covered by the per-opcode integration tests.
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

// Same body as `op_test_load_this_value_dsl`; distinct symbol to catch
// register-allocation or label-prefix drift between identical bodies.
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_test_load_this_sentinel_dsl, opcode_byte = 212, layout = None, length = 1, || {
        load_state_value!(t0, vm_state_offset = state_this_value);
        dispatch!(advance = 0);
    }
}

// Exercises `load_uninit_lex_sentinel!` through the lowerer + `naked_asm!`.
// Emits 4 instructions (movz + 3× movk) materializing `VALUE_UNINIT_LEX_BITS`.
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_test_load_uninit_lex_sentinel_dsl, opcode_byte = 213, layout = None, length = 1, || {
        load_uninit_lex_sentinel!(t0);
        dispatch!(advance = 0);
    }
}

// Exercises `load_local_fixed!` through the lowerer + `naked_asm!`.
// Emits a single `ldr x{dst}, [x20, #1 * 8]` instruction.
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_test_load_local_fixed_dsl, opcode_byte = 214, layout = None, length = 1, || {
        load_local_fixed!(1 => t0);
        dispatch!(advance = 0);
    }
}

// Exercises `store_local_fixed!` through the lowerer + `naked_asm!`.
// Emits a single `str x{src}, [x20, #2 * 8]` instruction.
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_test_store_local_fixed_dsl, opcode_byte = 215, layout = None, length = 1, || {
        store_local_fixed!(t0, 2);
        dispatch!(advance = 0);
    }
}

#[cfg(target_arch = "aarch64")]
#[test]
fn load_constant_handler_compiles_and_links() {
    use lyng_vm::dsl::test_helpers::DslHarness;
    DslHarness::assert_handler_symbol_exists(op_test_load_constant_dsl);
}

#[cfg(target_arch = "aarch64")]
#[test]
fn load_this_value_handler_compiles_and_links() {
    use lyng_vm::dsl::test_helpers::DslHarness;
    DslHarness::assert_handler_symbol_exists(op_test_load_this_value_dsl);
}

#[cfg(target_arch = "aarch64")]
#[test]
fn load_this_sentinel_handler_compiles_and_links() {
    use lyng_vm::dsl::test_helpers::DslHarness;
    DslHarness::assert_handler_symbol_exists(op_test_load_this_sentinel_dsl);
}

#[cfg(target_arch = "aarch64")]
#[test]
fn load_uninit_lex_sentinel_handler_compiles_and_links() {
    use lyng_vm::dsl::test_helpers::DslHarness;
    DslHarness::assert_handler_symbol_exists(op_test_load_uninit_lex_sentinel_dsl);
}

#[cfg(target_arch = "aarch64")]
#[test]
fn load_local_fixed_handler_compiles_and_links() {
    use lyng_vm::dsl::test_helpers::DslHarness;
    DslHarness::assert_handler_symbol_exists(op_test_load_local_fixed_dsl);
}

#[cfg(target_arch = "aarch64")]
#[test]
fn store_local_fixed_handler_compiles_and_links() {
    use lyng_vm::dsl::test_helpers::DslHarness;
    DslHarness::assert_handler_symbol_exists(op_test_store_local_fixed_dsl);
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

#[cfg(not(target_arch = "aarch64"))]
#[test]
fn load_local_fixed_handler_compiles_and_links() {
    // No-op on non-aarch64; the asm-DSL backend isn't compiled here.
}

#[cfg(not(target_arch = "aarch64"))]
#[test]
fn store_local_fixed_handler_compiles_and_links() {
    // No-op on non-aarch64; the asm-DSL backend isn't compiled here.
}

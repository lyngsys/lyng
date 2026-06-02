//! DSL validation case: the `ExtraWide` prefix structural compile test.
//!
//! Confirms the `dispatch!(advance = 1)` body compiles and links for the
//! prefix opcode shape via the proc-macro + backend pipeline.

#[cfg(target_arch = "aarch64")]
use lyng_vm::dispatch;
#[cfg(target_arch = "aarch64")]
use lyng_vm_dsl::llint_handler;

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_validation_extra_wide_prefix, opcode_byte = 202, layout = None, length = 1, || {
        dispatch!(advance = 1);
    }
}

#[cfg(target_arch = "aarch64")]
#[test]
fn op_extra_wide_prefix_handler_compiles_and_links() {
    use lyng_vm::dsl::test_helpers::DslHarness;
    DslHarness::assert_handler_symbol_exists(op_validation_extra_wide_prefix);
}

#[cfg(not(target_arch = "aarch64"))]
#[test]
fn op_extra_wide_prefix_handler_compiles_and_links() {
    // No-op on non-aarch64; the asm-DSL backend isn't compiled here.
}

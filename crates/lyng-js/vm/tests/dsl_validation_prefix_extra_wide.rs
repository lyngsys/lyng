//! DSL-0b validation case 8 (design §10 / plan B37): the `ExtraWide`
//! prefix decodes u32-width operands correctly.
//!
//! ## Status: structural (Path A); runtime check deferred to Batch 7
//!
//! Same shape as B36 (`Wide` prefix) but with u32-width operands. The
//! deferral reasoning is identical:
//!
//! 1. [`crate::dsl::entry::run_dsl_trampoline`] is still a stub.
//! 2. ExtraWide operand decoders aren't in
//!    [`crate::dsl::backend::aarch64::operands`] yet (per the module
//!    docstring, line 15).
//! 3. The `DSL_DISPATCH_TABLE` still routes `ExtraWide` and `Move`
//!    through `unimplemented_dsl_handler`.
//!
//! The structural test below confirms the proc-macro + backend
//! composition still compiles for the prefix-shaped opcode (a single
//! `dispatch!(advance = 1)` body that advances past the prefix byte).
//! The `state.prefix` write asm — encoded in
//! `dispatch_prefixed!(kind = 2)` — requires the `{state_prefix}`
//! binding which lands in Batch 7 alongside the cold-port wiring.

#[cfg(target_arch = "aarch64")]
use lyng_js_vm::dispatch;
#[cfg(target_arch = "aarch64")]
use lyng_js_vm_dsl::llint_handler;

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_validation_extra_wide_prefix, opcode_byte = 202, layout = None, length = 1, || {
        dispatch!(advance = 1);
    }
}

#[cfg(target_arch = "aarch64")]
#[test]
fn op_extra_wide_prefix_handler_compiles_and_links() {
    use lyng_js_vm::dsl::test_helpers::DslHarness;
    DslHarness::assert_handler_symbol_exists(op_validation_extra_wide_prefix);
}

#[cfg(not(target_arch = "aarch64"))]
#[test]
fn op_extra_wide_prefix_handler_compiles_and_links() {
    // No-op on non-aarch64; the asm-DSL backend isn't compiled here.
}

#[test]
#[ignore = "ExtraWide-form operand decoders + run_dsl_trampoline + DSL_DISPATCH_TABLE wiring required; enable when Batch 7 lands op_move + op_extra_wide ports."]
fn extra_wide_prefix_decodes_extra_wide_op_move() {
    // Eventual shape (see plan B37):
    //   let bytes = [
    //       /* ExtraWide */ 117, /* Move */ 1,
    //       /* dst */ 0x12, 0x34, 0x56, 0x78,
    //       /* src */ 0xab, 0xcd, 0xef, 0x01,
    //   ];
    //   // Run via DSL harness; assert dst register == read src register
    //   // (u32-width operand decoded correctly through the extra-wide path).
    panic!("test ignored — see attribute reason");
}

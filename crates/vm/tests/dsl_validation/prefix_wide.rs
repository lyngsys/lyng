//! DSL-0b validation case 7 (design §10 / plan B36): the `Wide` prefix
//! decodes wide-form operands correctly.
//!
//! ## Status: structural (Path A); runtime check deferred to Batch 7
//!
//! End-to-end the test would drive `[Wide, Move, u16 dst, u16 src]`
//! through the DSL trampoline and assert wide register operands
//! decode correctly. That requires:
//!
//! 1. A working [`crate::dsl::entry::run_dsl_trampoline`] (currently a
//!    stub).
//! 2. A real `op_wide` handler emitted by `dispatch_prefixed!(kind = 1)`
//!    plus a wide-form `op_move` decoder (the narrow `decode_abc!`
//!    fragments in [`crate::dsl::backend::aarch64::operands`] only
//!    handle u8 operands; the wide-form decoders are explicitly
//!    deferred to Batch 7 per
//!    `crates/vm/src/dsl/backend/aarch64/operands.rs` line 15).
//! 3. A `DSL_DISPATCH_TABLE` whose `Wide` and `Move` slots resolve to
//!    real handler symbols (currently all
//!    `unimplemented_dsl_handler` placeholders).
//!
//! For DSL-0b Batch 6b we land:
//!
//! - A compile-only structural test: a minimal `llint_handler!`
//!   invocation that emits the `dispatch_prefixed!(kind = 1)` asm
//!   fragment, asserting the proc-macro + backend composition still
//!   compiles for the prefix opcode shape.
//! - An `#[ignore]`d end-to-end test as a forward-pointer to Batch 7.

#[cfg(target_arch = "aarch64")]
use lyng_vm::dispatch;
#[cfg(target_arch = "aarch64")]
use lyng_vm_dsl::llint_handler;

// Compile-only structural test: an `llint_handler!` invocation whose
// asm body advances PC by one byte and tail-dispatches — the same
// structural shape a real `op_wide` handler will use (the "skip
// prefix byte and dispatch to widened opcode" path). The real
// `dispatch_prefixed!(kind = 1)` macro references the `{state_prefix}`
// named binding which the proc-macro lowerer doesn't emit yet
// (lands in Batch 7 alongside the `op_wide` cold port and the
// state-offset binding wiring), so this test stops short of the
// `state.prefix` write — see the `#[ignore]`d runtime test below.
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_validation_wide_prefix, opcode_byte = 201, layout = None, length = 1, || {
        dispatch!(advance = 1);
    }
}

#[cfg(target_arch = "aarch64")]
#[test]
fn op_wide_prefix_handler_compiles_and_links() {
    use lyng_vm::dsl::test_helpers::DslHarness;
    DslHarness::assert_handler_symbol_exists(op_validation_wide_prefix);
}

#[cfg(not(target_arch = "aarch64"))]
#[test]
fn op_wide_prefix_handler_compiles_and_links() {
    // No-op on non-aarch64; the asm-DSL backend isn't compiled here.
}

#[test]
#[ignore = "Wide-form operand decoders + run_dsl_trampoline + DSL_DISPATCH_TABLE wiring required; enable when Batch 7 lands op_move + op_wide ports."]
fn wide_prefix_decodes_wide_op_move() {
    // Eventual shape (see plan B36):
    //   let bytes = [
    //       /* Wide */ 116, /* Move */ 1,
    //       /* dst */ 0x12, 0x34,
    //       /* src */ 0x56, 0x78,
    //   ];
    //   // Run via DSL harness; assert dst register == read src register
    //   // (u16-width operand decoded correctly through the prefix path).
    panic!("test ignored — see attribute reason");
}

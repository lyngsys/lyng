//! DSL-0b B48 — spot-check 10 representative cold stubs.
//!
//! After B46 (codegen tool) and B47 (cold.rs + `DSL_DISPATCH_TABLE`),
//! 140 cold opcodes have a generated `op_xxx_dsl` handler symbol and a
//! matching `op_xxx_slow_rs` Rust shim. This file asserts at link time
//! that 10 representative stubs (one per family / cross-cut) exist
//! and are reachable from `DSL_DISPATCH_TABLE`.
//!
//! Runtime invocation is **deferred to Phase C** (after C1 flips
//! `Vm::run` over to the asm trampoline). The trampoline is still a
//! `naked_asm!("ret")` stub; calling a cold handler before that flip
//! would either crash or no-op since the trampoline's `STATE`
//! register isn't established. The `dsl_validation_*` tests follow
//! the same deferral pattern (see B30–B38).
//!
//! ## What each test catches
//!
//! 1. **Symbol existence.** If the codegen forgot to emit a stub, the
//!    `cold::op_xxx_dsl` reference fails to link. Taking the address
//!    forces linker resolution.
//! 2. **Dispatch table wiring.** The slot at `DSL_DISPATCH_TABLE[op as
//!    u8]` must point at the same function pointer as the direct
//!    symbol reference — i.e. the table's entry is the generated
//!    stub, not `unimplemented_dsl_handler`.

#[cfg(target_arch = "aarch64")]
use lyng_bytecode::Opcode;
#[cfg(target_arch = "aarch64")]
use lyng_vm::dsl::handlers::{cold, DslHandler, DSL_DISPATCH_TABLE};

/// Shared assertion body used by each per-opcode test below. Takes
/// the opcode + its directly-referenced handler symbol and verifies:
///
/// 1. The symbol's address is non-null (link-time).
/// 2. The dispatch-table slot for that opcode resolves to the same
///    function pointer (i.e. routing is intact).
#[cfg(target_arch = "aarch64")]
fn check_cold_stub(op: Opcode, handler: DslHandler) {
    let from_symbol = handler as *const ();
    assert!(
        !from_symbol.is_null(),
        "DSL cold handler for {op:?} is null (codegen drift?)",
    );
    let from_table = DSL_DISPATCH_TABLE[op as usize] as *const ();
    assert_eq!(
        from_table, from_symbol,
        "DSL_DISPATCH_TABLE[{:?} as u8 = {}] = {:p} does not match \
         directly-referenced handler {:p} (dispatch-table wiring drifted from codegen)",
        op, op as u8, from_table, from_symbol,
    );
}

// =====================================================================
// 10 representative cold opcodes — one per family / cross-cut. Each is
// its own `#[test]` so the integration-test runner reports 10 distinct
// pass/fail lines, matching the plan's expected test count.
// =====================================================================

#[cfg(target_arch = "aarch64")]
#[test]
fn spot_check_op_load_undefined() {
    check_cold_stub(Opcode::LoadUndefined, cold::op_load_undefined_dsl);
}

#[cfg(target_arch = "aarch64")]
#[test]
fn spot_check_op_load_smi() {
    check_cold_stub(Opcode::LoadSmi, cold::op_load_smi_dsl);
}

#[cfg(target_arch = "aarch64")]
#[test]
fn spot_check_op_get_named_property() {
    check_cold_stub(Opcode::GetNamedProperty, cold::op_get_named_property_dsl);
}

#[cfg(target_arch = "aarch64")]
#[test]
fn spot_check_op_load_global() {
    check_cold_stub(Opcode::LoadGlobal, cold::op_load_global_dsl);
}

#[cfg(target_arch = "aarch64")]
#[test]
fn spot_check_op_call0() {
    check_cold_stub(Opcode::Call0, cold::op_call0_dsl);
}

#[cfg(target_arch = "aarch64")]
#[test]
fn spot_check_op_create_object() {
    check_cold_stub(Opcode::CreateObject, cold::op_create_object_dsl);
}

#[cfg(target_arch = "aarch64")]
#[test]
fn spot_check_op_typeof() {
    check_cold_stub(Opcode::TypeOf, cold::op_type_of_dsl);
}

#[cfg(target_arch = "aarch64")]
#[test]
fn spot_check_op_throw() {
    check_cold_stub(Opcode::Throw, cold::op_throw_dsl);
}

#[cfg(target_arch = "aarch64")]
#[test]
fn spot_check_op_yield() {
    check_cold_stub(Opcode::Yield, cold::op_yield_dsl);
}

#[cfg(target_arch = "aarch64")]
#[test]
fn spot_check_op_close_iterator() {
    check_cold_stub(Opcode::CloseIterator, cold::op_close_iterator_dsl);
}

// On non-aarch64 hosts the asm-DSL backend isn't compiled in, so the
// `op_xxx_dsl` symbols don't exist (they live under
// `#[cfg(target_arch = "aarch64")]`). Emit a single skip-test so the
// integration-test binary still builds; the real coverage is
// aarch64-only per design §3.
#[cfg(not(target_arch = "aarch64"))]
#[test]
fn spot_check_skipped_on_non_aarch64() {
    // No-op: asm-DSL handlers are aarch64-only.
}

//! Hot DSL handlers. Populated by tasks B39–B42.
//!
//! Per the design (§10), hot handlers are the highest-frequency opcodes
//! and ship with inline fast paths (SMI arithmetic, register moves, fast
//! object access). The `llint_handler!` proc-macro lowers each handler
//! body into a single `naked_asm!` block; the backend `macro_rules!`
//! macros (under `crates/lyng-js/vm/src/dsl/backend/aarch64/`) supply the
//! asm fragments for individual DSL ops (`decode_ab!`, `load_reg!`,
//! `dispatch!`, etc.).
//!
//! For DSL-0b the handler symbols exist (so the link-check passes) but
//! they are not yet wired into `DSL_DISPATCH_TABLE` — the alpha path
//! continues to dispatch through the legacy handlers. Phase C of the
//! plan flips the table over.

// Bring the AArch64 backend macros into scope so the proc-macro-emitted
// `decode_ab!`, `load_reg!`, `store_reg!`, `dispatch!`, ... calls
// resolve. They are `#[macro_export]`-ed at the crate root.
#[cfg(target_arch = "aarch64")]
use crate::{
    add_smi_overflow, call_slow, check_smi, decode_ab, decode_abc_slot, dispatch,
    dispatch_after_slow, load_reg, record_smi, store_reg, tag_smi, untag_smi,
};

#[cfg(target_arch = "aarch64")]
use lyng_js_vm_dsl::llint_handler;

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_move, layout = Ab, length = 3, |dst, src| {
        load_reg!(src => t0);
        store_reg!(dst, t0);
        dispatch!();
    }
}

// =====================================================================
// op_add (B40) — Abc layout with feedback slot, SMI fast path.
// =====================================================================
//
// Fast path: 2x check_smi + 2x untag + add + tag + store_reg + record_smi
// + dispatch. Slow path: call_slow into the op_add semantic body, then
// dispatch_after_slow.

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_add, layout = AbcSlot, length = 6, |a, b, c, slot| {
        load_reg!(b => t0);
        check_smi!(t0, .slow);
        load_reg!(c => t1);
        check_smi!(t1, .slow);
        untag_smi!(t0);
        untag_smi!(t1);
        add_smi_overflow!(t0, t1 => t2, .slow);
        tag_smi!(t2);
        store_reg!(a, t2);
        record_smi!(slot);
        dispatch!();
        .slow:
        call_slow!(op_add_slow_rs, args = [a, b, c, slot]);
        dispatch_after_slow!();
    }
}

/// Slow-path shim for `op_add`. The asm trampoline tail-calls this
/// with 4 u32 operand slots after the state pointer; we adapt them to
/// the `OpBinaryArgs` shape that `op_add_semantic` expects.
///
/// The `instruction_len` is hardcoded to `6` (op_add's encoded length
/// for the narrow form). When Wide / ExtraWide prefix decoding lands,
/// the lowerer will need to pass the effective length too.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_add_slow_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    dst: u32,
    lhs: u32,
    rhs: u32,
    feedback_slot: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    // SAFETY: state is a valid LlIntState pointer for the duration of
    // the call per the DSL-0b ABI contract on `from_raw`.
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    let args = crate::vm::semantics::arithmetic::OpBinaryArgs {
        dst: dst as u16,
        lhs: lhs as u16,
        rhs: rhs as u16,
        feedback_slot: lyng_js_types::FeedbackSlotId::from_raw(feedback_slot),
        instruction_len: 6,
    };
    let outcome = crate::vm::semantics::arithmetic::op_add_semantic(&mut dispatch, args);
    dispatch.translate_outcome(outcome)
}

/// Non-aarch64 stubs. The DSL handler family is aarch64-only in DSL-0b
/// per design §3; on other hosts we emit placeholders so the dispatch
/// table can still be assembled.
#[cfg(not(target_arch = "aarch64"))]
pub unsafe extern "C" fn op_move() -> ! {
    loop {}
}

#[cfg(not(target_arch = "aarch64"))]
pub unsafe extern "C" fn op_add() -> ! {
    loop {}
}

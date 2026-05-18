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
    dispatch_after_slow, load_reg, store_reg, tag_smi, untag_smi,
};

#[cfg(target_arch = "aarch64")]
use lyng_js_vm_dsl::llint_handler;

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_move, layout = Ab, length = 4, |dst, src| {
        load_reg!(src => t0);
        store_reg!(dst, t0);
        dispatch!();
    }
}

// =====================================================================
// op_add (B40) — Abc layout with feedback slot, SMI fast path.
// =====================================================================
//
// Fast path: 2x check_smi + 2x untag + add + tag + store_reg +
// call_slow!(op_add_record_smi_rs) + dispatch. The slot-recording
// shim bumps the warmup counter and execution count through
// `Vm::record_feedback_slot` (which also mirrors the legacy state to
// the flat array). Inline `record_smi!` was dropped in DSL-0c because
// the `entry_observed` offset binding is still a placeholder (offset
// 0) — writing at offset 0 would corrupt the `Option<FeedbackSiteState>`
// discriminant. Slow path: call_slow into the op_add semantic body,
// which performs the same feedback recording itself.
//
// The extra `call_slow!` on the fast path adds one Rust function call
// per SMI add, but preserves the warmup/execution-count bookkeeping
// the tiering machinery relies on. A future task can re-introduce
// inline observed-types recording once the `entry_observed` offset
// binding lands.

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
        call_slow!(op_add_record_smi_rs, args = [slot]);
        dispatch_after_slow!();
        .slow:
        call_slow!(op_add_slow_rs, args = [a, b, c, slot]);
        dispatch_after_slow!();
    }
}

/// Slow-path shim for the SMI fast path's feedback recording. The
/// inline `record_smi!` macro would write the observed-types bit into
/// the flat-array entry, but its offset binding (`entry_observed`) is
/// still a placeholder (offset 0). Routing through
/// `Vm::record_feedback_slot` is functionally correct: it bumps the
/// warmup counter, allocates the legacy vector at threshold, mirrors
/// the legacy state to the flat array, and observes the tier feedback
/// event — all the bookkeeping `op_add_semantic`'s slow path performs.
/// The fast path returns `Continue { pc_advance: 6 }` so the asm
/// bridge advances PC by the encoded op_add length without going
/// through `op_add_semantic`.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_add_record_smi_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    feedback_slot: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    {
        let inner = dispatch.dispatch_state();
        let code = inner.code();
        inner
            .vm
            .record_feedback_slot(code, lyng_js_types::FeedbackSlotId::from_raw(feedback_slot));
    }
    dispatch.translate_outcome(crate::dsl::slow_path::SemanticOutcome::Continue {
        pc_advance: 6,
    })
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

// =====================================================================
// op_jump (B41) — Ax layout, length = 5. The semantic body in
// op_jump_semantic handles backward-edge polling and PC arithmetic; the
// DSL handler delegates to it via call_slow.
// =====================================================================

// `Jump` uses the Ax instruction form: 1 byte opcode + 3 bytes
// sign-extended i24 delta. Encoded length is 4 bytes. The DSL's
// `decode_ax!` reads a 4-byte word at PC+1 — that pulls in the
// adjacent opcode byte, but the slow-path shim masks and sign-extends
// the low 24 bits before computing the actual delta, so the extra
// byte is harmless.
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_jump, layout = Ax, length = 4, |offset| {
        call_slow!(op_jump_slow_rs, args = [offset]);
        dispatch_after_slow!();
    }
}

/// Slow-path shim for `op_jump`. The 4-byte `decode_ax!` load reads
/// 3 bytes of i24 delta + 1 byte of the next opcode (or padding).
/// We mask off the top byte and sign-extend the low 24 bits to
/// recover the i24 delta semantic_body expects.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_jump_slow_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    offset_raw: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    // sign-extend the low 24 bits of offset_raw
    let delta = (((offset_raw & 0x00ff_ffff) as i32) << 8) >> 8;
    let args = crate::vm::semantics::control_flow::OpJumpArgs {
        delta,
        instruction_len: 4,
    };
    let outcome = crate::vm::semantics::control_flow::op_jump_semantic(&mut dispatch, args);
    dispatch.translate_outcome(outcome)
}

// =====================================================================
// op_return (B42) — Ax layout, length = 4. The 24-bit operand encodes
// the register holding the return value. Frame-transitioning; always
// returns Refresh / ExitDone / ExitError.
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_return, layout = Ax, length = 4, |src| {
        call_slow!(op_return_slow_rs, args = [src]);
        dispatch_after_slow!();
    }
}

/// Slow-path shim for `op_return`. The 4-byte `decode_ax!` load reads
/// 3 bytes of i24 register-id + 1 byte of the next opcode (or padding).
/// Mask the low 24 bits; the result is a non-negative u16 register id
/// in practice, so no sign-extension is needed.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_return_slow_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    src: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    let args = crate::vm::semantics::control_flow::OpReturnArgs {
        register: (src & 0x00ff_ffff) as u16,
    };
    let outcome = crate::vm::semantics::control_flow::op_return_semantic(&mut dispatch, args);
    dispatch.translate_outcome(outcome)
}

#[cfg(target_arch = "aarch64")]
use crate::decode_ax;

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

#[cfg(not(target_arch = "aarch64"))]
pub unsafe extern "C" fn op_jump() -> ! {
    loop {}
}

#[cfg(not(target_arch = "aarch64"))]
pub unsafe extern "C" fn op_return() -> ! {
    loop {}
}

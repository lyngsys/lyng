//! Hot DSL handlers — highest-frequency opcodes with inline `LLInt` bodies.
//!
//! The `llint_handler!` proc-macro lowers each handler body into a single
//! `naked_asm!` block; the backend `macro_rules!` macros under
//! `crates/vm/src/dsl/backend/aarch64/` supply the asm fragments
//! (`decode_ab!`, `load_reg!`, `dispatch!`, etc.).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::not_unsafe_ptr_arg_deref,
    reason = "DSL hot shims receive decoded raw operand slots from LLInt assembly; explicit narrowing/sign casts reconstruct the bytecode operand widths before semantic dispatch"
)]

// AArch64 backend macros for proc-macro-emitted asm fragments.
#[cfg(target_arch = "aarch64")]
use crate::{
    add_smi_overflow, branch_i32_negative, call_slow, check_smi_pair, decode_ab, decode_abc_slot,
    decode_ax_i24, dispatch, dispatch_after_slow, jump_relative_i32_and_dispatch, load_reg,
    poll_safepoint, record_smi, store_reg, tag_smi, untag_smi,
};

#[cfg(target_arch = "aarch64")]
use lyng_vm_dsl::llint_handler;

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_move, opcode_byte = 1, layout = Ab, length = 4, |dst, src| {
        load_reg!(src => t0);
        store_reg!(dst, t0);
        dispatch!();
    }
}

// =====================================================================
// op_add — AbcSlot layout with SMI inline hit path.
// =====================================================================
//
// Hit path: `check_smi_pair!` hoists the tag comparand out of the
// per-operand check, saving 2 instructions vs two independent `check_smi!`
// calls. `record_smi!` writes pending scalar feedback into the flat
// LLInt sidecar; Rust drains it at VM run boundaries.

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_add, opcode_byte = 31, layout = AbcSlot, length = 6, |a, b, c, slot| {
        load_reg!(b => t0);
        load_reg!(c => t1);
        check_smi_pair!(t0, t1, .slow);
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

/// Slow-path shim for `op_add`. Adapts 4 raw u32 operand slots to
/// `OpBinaryArgs` and calls `op_add_semantic`.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_add_slow_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    dst: u32,
    lhs: u32,
    rhs: u32,
    feedback_slot: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    // SAFETY: state is valid for the duration of this call; see `from_raw` contract.
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    let args = crate::vm::semantics::arithmetic::OpBinaryArgs {
        dst: dst as u16,
        lhs: lhs as u16,
        rhs: rhs as u16,
        feedback_slot: lyng_types::FeedbackSlotId::from_raw(feedback_slot),
        instruction_len: 6,
    };
    let outcome = crate::vm::semantics::arithmetic::op_add_semantic(&mut dispatch, args);
    dispatch.translate_outcome(outcome)
}

// =====================================================================
// op_jump — AxI24 layout, length = 4.
// =====================================================================
//
// Forward jumps apply the signed i24 delta inline. Backward jumps poll
// the safepoint flag first; when pending, the slow shim runs the poll
// then applies the delta.
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_jump, opcode_byte = 63, layout = AxI24, length = 4, |offset| {
        branch_i32_negative!(offset, .taken_backward);
        jump_relative_i32_and_dispatch!(offset, advance = 4);
        .taken_backward:
        poll_safepoint!(.poll_pending);
        jump_relative_i32_and_dispatch!(offset, advance = 4);
        .poll_pending:
        call_slow!(op_jump_poll_rs, args = [offset]);
        dispatch_after_slow!();
    }
}

/// Pending-poll shim for backward `op_jump`. The `AxI24` prologue has
/// already sign-extended the 24-bit delta.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_jump_poll_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    offset_raw: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    match crate::dsl::poll::run_poll(&mut dispatch, crate::dsl::poll::PollArgs) {
        crate::dsl::slow_path::SemanticOutcome::Continue { .. } => {}
        outcome => return dispatch.translate_outcome(outcome),
    }
    let delta = offset_raw as i32;
    let instruction_offset = dispatch.current_instruction_offset();
    let target = i64::from(instruction_offset) + 4 + i64::from(delta);
    if target < 0 || target > i64::from(u32::MAX) {
        let code = dispatch.dispatch_state().code();
        return dispatch.translate_outcome(crate::dsl::slow_path::SemanticOutcome::ExitError {
            error: crate::error::VmError::InvalidJumpTarget {
                code,
                instruction_offset,
                target_offset: target,
            },
        });
    }
    let pc_advance = (4_i64 + i64::from(delta)) as u32;
    dispatch.translate_outcome(crate::dsl::slow_path::SemanticOutcome::Continue { pc_advance })
}

// =====================================================================
// op_return — Ax layout, length = 4. Frame-transitioning; always
// returns Refresh / ExitDone / ExitError.
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_return, opcode_byte = 67, layout = Ax, length = 4, |src| {
        call_slow!(op_return_slow_rs, args = [src]);
        dispatch_after_slow!();
    }
}

/// Slow-path shim for `op_return`. The `decode_ax!` load reads 3 bytes
/// of i24 register-id; mask the low 24 bits to get the register id.
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

/// Non-aarch64 stubs. The DSL handler family is aarch64-only; on other
/// hosts these placeholders allow the dispatch table to link.
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

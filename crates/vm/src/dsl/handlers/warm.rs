//! Warm DSL handlers — mid-frequency opcodes that poll a safepoint on
//! backedges (`op_loop_header`, conditional backward jumps) or decode a
//! prefix (`op_wide`, `op_extra_wide`).

#![allow(
    clippy::cast_possible_truncation,
    clippy::not_unsafe_ptr_arg_deref,
    reason = "DSL warm shims receive decoded raw operand slots from LLInt assembly; explicit narrowing reconstructs the bytecode operand widths before semantic dispatch"
)]

#[cfg(target_arch = "aarch64")]
use crate::{
    branch_i8_negative, branch_i16_negative, branch_nonzero, branch_zero, call_slow, check_bool,
    decode_a, decode_ab, decode_abx, decode_ax, dispatch, dispatch_after_slow,
    jump_relative_i8_and_dispatch, jump_relative_i16_and_dispatch, load_reg, poll_safepoint,
    untag_bool,
};

#[cfg(target_arch = "aarch64")]
use lyng_vm_dsl::llint_handler;

// =====================================================================
// op_loop_header — Ax layout, length = 4. Polls safepoint on every
// backedge; delegates pending work to the slow path.
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_loop_header, opcode_byte = 66, layout = Ax, length = 4, |_unused_target_offset| {
        poll_safepoint!(.poll_pending);
        dispatch!(advance = 4);
        .poll_pending:
        call_slow!(op_loop_header_poll_rs, args = []);
        dispatch_after_slow!();
    }
}

/// Slow-path shim for `op_loop_header`'s safepoint poll.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_loop_header_poll_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
) -> crate::dsl::slow_path::SlowPathReturn {
    // SAFETY: state is valid for the duration of this call; see `from_raw` contract.
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    let outcome = crate::dsl::poll::run_poll(&mut dispatch, crate::dsl::poll::PollArgs);
    dispatch.translate_outcome(outcome)
}

// =====================================================================
// op_jump8 — A layout, length = 2. Mirrors `op_jump`'s inline shape
// with an i8 delta; backward jumps poll safepoint first.
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_jump8, opcode_byte = 141, layout = A, length = 2, |offset| {
        branch_i8_negative!(offset, .taken_backward);
        jump_relative_i8_and_dispatch!(offset, advance = 2);
        .taken_backward:
        poll_safepoint!(.poll_pending);
        jump_relative_i8_and_dispatch!(offset, advance = 2);
        .poll_pending:
        call_slow!(op_jump8_poll_rs, args = [offset]);
        dispatch_after_slow!();
    }
}

/// Pending-poll shim for backward `op_jump8`. Runs the poll then applies
/// the sign-extended i8 delta, returning `Continue { pc_advance }`.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_jump8_poll_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    offset_raw: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    match crate::dsl::poll::run_poll(&mut dispatch, crate::dsl::poll::PollArgs) {
        crate::dsl::slow_path::SemanticOutcome::Continue { .. } => {}
        outcome => return dispatch.translate_outcome(outcome),
    }
    let delta = i32::from(offset_raw as i8);
    let instruction_offset = dispatch.current_instruction_offset();
    let target = i64::from(instruction_offset) + 2 + i64::from(delta);
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
    // Relative advance; the bounds check above guarantees the resulting target
    // PC is in `[0, u32::MAX]`, so narrowing the advance to `u32` is intentional.
    #[allow(clippy::cast_sign_loss)]
    let pc_advance = (2_i64 + i64::from(delta)) as u32;
    dispatch.translate_outcome(crate::dsl::slow_path::SemanticOutcome::Continue { pc_advance })
}

#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_jump8_slow_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    offset_raw: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    let delta = i32::from(offset_raw as i8);
    let args = crate::vm::semantics::control_flow::OpJumpArgs {
        delta,
        instruction_len: 2,
    };
    let outcome = crate::vm::semantics::control_flow::op_jump8_semantic(&mut dispatch, args);
    dispatch.translate_outcome(outcome)
}

// =====================================================================
// op_jump_if_true / op_jump_if_false — Abx layout (1-byte reg + i16
// delta), length = 4. Backward jumps poll safepoint.
// =====================================================================
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_jump_if_true, opcode_byte = 64, layout = Abx, length = 4, |condition, offset| {
        load_reg!(condition => t0);
        check_bool!(t0, .slow);
        untag_bool!(t0);
        branch_zero!(t0, .not_taken);
        branch_i16_negative!(offset, .taken_backward);
        jump_relative_i16_and_dispatch!(offset, advance = 4);
        .taken_backward:
        poll_safepoint!(.slow);
        jump_relative_i16_and_dispatch!(offset, advance = 4);
        .not_taken:
        dispatch!(advance = 4);
        .slow:
        call_slow!(op_jump_if_true_slow_rs, args = [condition, offset]);
        dispatch_after_slow!();
    }
}

#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_jump_if_true_slow_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    condition: u32,
    offset_raw: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    let delta = i32::from(offset_raw as i16);
    let args = crate::vm::semantics::control_flow::OpJumpIfArgs {
        condition_register: condition as u16,
        delta,
        instruction_len: 4,
    };
    let outcome = crate::vm::semantics::control_flow::op_jump_if_true_semantic(&mut dispatch, args);
    dispatch.translate_outcome(outcome)
}

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_jump_if_false, opcode_byte = 65, layout = Abx, length = 4, |condition, offset| {
        load_reg!(condition => t0);
        check_bool!(t0, .slow);
        untag_bool!(t0);
        branch_nonzero!(t0, .not_taken);
        branch_i16_negative!(offset, .taken_backward);
        jump_relative_i16_and_dispatch!(offset, advance = 4);
        .taken_backward:
        poll_safepoint!(.slow);
        jump_relative_i16_and_dispatch!(offset, advance = 4);
        .not_taken:
        dispatch!(advance = 4);
        .slow:
        call_slow!(op_jump_if_false_slow_rs, args = [condition, offset]);
        dispatch_after_slow!();
    }
}

#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_jump_if_false_slow_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    condition: u32,
    offset_raw: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    let delta = i32::from(offset_raw as i16);
    let args = crate::vm::semantics::control_flow::OpJumpIfArgs {
        condition_register: condition as u16,
        delta,
        instruction_len: 4,
    };
    let outcome =
        crate::vm::semantics::control_flow::op_jump_if_false_semantic(&mut dispatch, args);
    dispatch.translate_outcome(outcome)
}

// =====================================================================
// op_jump_if_true8 / op_jump_if_false8 — Ab layout (1-byte reg + i8
// delta), length = 3. Backward jumps poll safepoint.
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_jump_if_true8, opcode_byte = 142, layout = Ab, length = 3, |condition, offset| {
        load_reg!(condition => t0);
        check_bool!(t0, .slow);
        untag_bool!(t0);
        branch_zero!(t0, .not_taken);
        branch_i8_negative!(offset, .taken_backward);
        jump_relative_i8_and_dispatch!(offset, advance = 3);
        .taken_backward:
        poll_safepoint!(.slow);
        jump_relative_i8_and_dispatch!(offset, advance = 3);
        .not_taken:
        dispatch!(advance = 3);
        .slow:
        call_slow!(op_jump_if_true8_slow_rs, args = [condition, offset]);
        dispatch_after_slow!();
    }
}

#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_jump_if_true8_slow_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    condition: u32,
    offset_raw: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    let delta = i32::from(offset_raw as i8);
    let args = crate::vm::semantics::control_flow::OpJumpIfArgs {
        condition_register: condition as u16,
        delta,
        instruction_len: 3,
    };
    let outcome =
        crate::vm::semantics::control_flow::op_jump_if_true8_semantic(&mut dispatch, args);
    dispatch.translate_outcome(outcome)
}

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_jump_if_false8, opcode_byte = 143, layout = Ab, length = 3, |condition, offset| {
        load_reg!(condition => t0);
        check_bool!(t0, .slow);
        untag_bool!(t0);
        branch_nonzero!(t0, .not_taken);
        branch_i8_negative!(offset, .taken_backward);
        jump_relative_i8_and_dispatch!(offset, advance = 3);
        .taken_backward:
        poll_safepoint!(.slow);
        jump_relative_i8_and_dispatch!(offset, advance = 3);
        .not_taken:
        dispatch!(advance = 3);
        .slow:
        call_slow!(op_jump_if_false8_slow_rs, args = [condition, offset]);
        dispatch_after_slow!();
    }
}

#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_jump_if_false8_slow_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    condition: u32,
    offset_raw: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    let delta = i32::from(offset_raw as i8);
    let args = crate::vm::semantics::control_flow::OpJumpIfArgs {
        condition_register: condition as u16,
        delta,
        instruction_len: 3,
    };
    let outcome =
        crate::vm::semantics::control_flow::op_jump_if_false8_semantic(&mut dispatch, args);
    dispatch.translate_outcome(outcome)
}

// =====================================================================
// op_wide / op_extra_wide — None layout, length = 1.
//
// Prefix handlers delegate to `dispatch_wide_form`, which reads the
// semantic byte at `bytes[pc+1]`, decodes wide-form operands, calls
// the matching semantic body, and returns `SemanticOutcome` with
// `pc_advance` = full wide-form instruction length. A stacked prefix
// is rejected with `DoublePrefix`.
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_wide, opcode_byte = 120, layout = None, length = 1, || {
        call_slow!(op_wide_set_prefix_rs, args = []);
        dispatch_after_slow!();
    }
}

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_extra_wide, opcode_byte = 121, layout = None, length = 1, || {
        call_slow!(op_extra_wide_set_prefix_rs, args = []);
        dispatch_after_slow!();
    }
}

/// Slow-path shim for `op_wide`.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_wide_set_prefix_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    let outcome = run_prefix(&mut dispatch, lyng_bytecode::Opcode::Wide);
    dispatch.translate_outcome(outcome)
}

/// Slow-path shim for `op_extra_wide`.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_extra_wide_set_prefix_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    let outcome = run_prefix(&mut dispatch, lyng_bytecode::Opcode::ExtraWide);
    dispatch.translate_outcome(outcome)
}

/// Shared body for `op_wide_set_prefix_rs` / `op_extra_wide_set_prefix_rs`.
/// Rejects a stacked prefix with `DoublePrefix`, sets `state.prefix`,
/// delegates to `dispatch_wide_form`, then clears the prefix.
#[cfg(target_arch = "aarch64")]
fn run_prefix(
    dispatch: &mut crate::dsl::slow_path::LlIntDispatchState<'_, '_>,
    prefix: lyng_bytecode::Opcode,
) -> crate::dsl::slow_path::SemanticOutcome {
    use crate::dsl::slow_path::SemanticOutcome;
    {
        let inner = dispatch.dispatch_state();
        if inner.prefix.is_some() {
            return SemanticOutcome::ExitError {
                error: crate::error::VmError::DoublePrefix {
                    code: inner.code(),
                    instruction_offset: inner.pc(),
                },
            };
        }
        inner.prefix = Some(prefix);
    }
    let outcome = crate::dsl::handlers::cold::dispatch_wide_form(dispatch, prefix);
    // Clear prefix — must be None before the next instruction dispatches.
    dispatch.dispatch_state().prefix = None;
    outcome
}

/// Non-aarch64 placeholder stubs.
#[cfg(not(target_arch = "aarch64"))]
pub unsafe extern "C" fn op_loop_header() -> ! {
    loop {}
}

#[cfg(not(target_arch = "aarch64"))]
pub unsafe extern "C" fn op_wide() -> ! {
    loop {}
}

#[cfg(not(target_arch = "aarch64"))]
pub unsafe extern "C" fn op_extra_wide() -> ! {
    loop {}
}

#[cfg(not(target_arch = "aarch64"))]
pub unsafe extern "C" fn op_jump8() -> ! {
    loop {}
}

#[cfg(not(target_arch = "aarch64"))]
pub unsafe extern "C" fn op_jump_if_true() -> ! {
    loop {}
}

#[cfg(not(target_arch = "aarch64"))]
pub unsafe extern "C" fn op_jump_if_true8() -> ! {
    loop {}
}

#[cfg(not(target_arch = "aarch64"))]
pub unsafe extern "C" fn op_jump_if_false() -> ! {
    loop {}
}

#[cfg(not(target_arch = "aarch64"))]
pub unsafe extern "C" fn op_jump_if_false8() -> ! {
    loop {}
}

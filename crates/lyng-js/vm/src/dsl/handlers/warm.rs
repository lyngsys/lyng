//! Warm DSL handlers. Populated by tasks B43–B45.
//!
//! Warm handlers are mid-frequency opcodes that need either a backedge
//! safepoint poll (`op_loop_header`, conditional backward jumps) or a
//! prefix decode (`op_wide`, `op_extra_wide`). They run on top of the
//! same backend macros as the hot handlers; the distinction is
//! categorical (used to determine inlining heuristics in the DSL
//! optimizer + dispatch table organization later in DSL-1).

#[cfg(target_arch = "aarch64")]
use crate::{
    call_slow, decode_a, decode_ab, decode_abx, decode_ax, dispatch, dispatch_after_slow,
    poll_safepoint,
};

#[cfg(target_arch = "aarch64")]
use lyng_js_vm_dsl::llint_handler;

// =====================================================================
// op_loop_header (B43) — Ax layout, length = 4. Polls the safepoint
// flag on every backedge; on pending work, jumps to the slow path.
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_loop_header, layout = Ax, length = 4, |_unused_target_offset| {
        poll_safepoint!(.poll_pending);
        dispatch!(advance = 4);
        .poll_pending:
        call_slow!(op_loop_header_poll_rs, args = []);
        dispatch_after_slow!();
    }
}

/// Slow-path shim for `op_loop_header`'s safepoint poll. Invoked when
/// `poll_safepoint!` sees a non-zero `vm.poll_pending` byte. Delegates
/// to the shared `crate::dsl::poll::run_poll` consumer.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_loop_header_poll_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
) -> crate::dsl::slow_path::SlowPathReturn {
    // SAFETY: state is a valid LlIntState pointer for the duration of
    // the call per the DSL-0b ABI contract on `from_raw`.
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    let outcome = crate::dsl::poll::run_poll(&mut dispatch, crate::dsl::poll::PollArgs);
    dispatch.translate_outcome(outcome)
}

// =====================================================================
// op_jump8 (B44) — 1-byte i8 delta variant. Layout A in the DSL
// (single byte at PC+1), length = 2.
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_jump8, layout = A, length = 2, |offset| {
        call_slow!(op_jump8_slow_rs, args = [offset]);
        dispatch_after_slow!();
    }
}

#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_jump8_slow_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    offset_raw: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    let delta = (offset_raw as i8) as i32;
    let args = crate::vm::semantics::control_flow::OpJumpArgs {
        delta,
        instruction_len: 2,
    };
    let outcome = crate::vm::semantics::control_flow::op_jump8_semantic(&mut dispatch, args);
    dispatch.translate_outcome(outcome)
}

// =====================================================================
// op_jump_if_true / op_jump_if_false — Abx layout (1-byte reg + 2-byte
// i16 delta), length = 4.
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_jump_if_true, layout = Abx, length = 4, |condition, offset| {
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
    let delta = (offset_raw as i16) as i32;
    let args = crate::vm::semantics::control_flow::OpJumpIfArgs {
        condition_register: condition as u16,
        delta,
        instruction_len: 4,
    };
    let outcome =
        crate::vm::semantics::control_flow::op_jump_if_true_semantic(&mut dispatch, args);
    dispatch.translate_outcome(outcome)
}

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_jump_if_false, layout = Abx, length = 4, |condition, offset| {
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
    let delta = (offset_raw as i16) as i32;
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
// op_jump_if_true8 / op_jump_if_false8 — Ab layout in the DSL (1-byte
// reg + 1-byte i8 delta), length = 3.
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_jump_if_true8, layout = Ab, length = 3, |condition, offset| {
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
    let delta = (offset_raw as i8) as i32;
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
    op_jump_if_false8, layout = Ab, length = 3, |condition, offset| {
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
    let delta = (offset_raw as i8) as i32;
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
// op_wide / op_extra_wide (DSL-0c C2) — None layout, length = 1.
//
// The DSL prefix handlers drive wide-form dispatch entirely through
// `crate::dsl::handlers::cold::dispatch_wide_form` — a centralized
// Rust function whose match arms decode + execute each prefix-accepting
// opcode's wide-form encoding. The shim:
//
//   1. Checks for a stacked prefix (returns `DoublePrefix` if set).
//   2. Calls `dispatch_wide_form(state, Wide | ExtraWide)`, which reads
//      `bytes[pc+1]` (the semantic byte), invokes the matching opcode's
//      decoder + semantic body, and returns a `SemanticOutcome` whose
//      `pc_advance` is the full wide-form instruction length.
//   3. Translates the outcome via `translate_outcome` so the asm bridge
//      advances PC past the entire wide instruction.
//
// This replaces the legacy `op_prefix_via_alpha` bridge that delegated
// wide-form dispatch through the α dispatch table. The α tables and
// `dispatch_handlers/` are now unreferenced and can be deleted.
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_wide, layout = None, length = 1, || {
        call_slow!(op_wide_set_prefix_rs, args = []);
        dispatch_after_slow!();
    }
}

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_extra_wide, layout = None, length = 1, || {
        call_slow!(op_extra_wide_set_prefix_rs, args = []);
        dispatch_after_slow!();
    }
}

/// Slow-path shim for `op_wide`. Delegates the full wide-form
/// instruction to `dispatch_wide_form`, which decodes operands and
/// runs the semantic body for the byte at `bytes[pc+1]`.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_wide_set_prefix_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    let outcome = run_prefix(&mut dispatch, lyng_js_bytecode::Opcode::Wide);
    dispatch.translate_outcome(outcome)
}

/// Slow-path shim for `op_extra_wide`. Counterpart to
/// `op_wide_set_prefix_rs`; routes through `dispatch_wide_form` with
/// the `ExtraWide` prefix.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_extra_wide_set_prefix_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    let outcome = run_prefix(&mut dispatch, lyng_js_bytecode::Opcode::ExtraWide);
    dispatch.translate_outcome(outcome)
}

/// Shared body of `op_wide_set_prefix_rs` /
/// `op_extra_wide_set_prefix_rs`. Mirrors the α prefix handler:
/// rejects a stacked prefix (`state.prefix.is_some()`) with
/// `VmError::DoublePrefix`, then delegates the full wide-form
/// instruction to `dispatch_wide_form`. The dispatcher's
/// `SemanticOutcome` carries the total instruction length so PC
/// advances past the entire wide-form instruction in one step.
#[cfg(target_arch = "aarch64")]
fn run_prefix(
    dispatch: &mut crate::dsl::slow_path::LlIntDispatchState<'_, '_>,
    prefix: lyng_js_bytecode::Opcode,
) -> crate::dsl::slow_path::SemanticOutcome {
    use crate::dsl::slow_path::SemanticOutcome;
    {
        let inner = dispatch.dispatch_state();
        if inner.prefix.is_some() {
            return SemanticOutcome::ExitError {
                error: crate::error::VmError::DoublePrefix {
                    code: inner.frame.code(),
                    instruction_offset: inner.frame.instruction_offset(),
                },
            };
        }
        inner.prefix = Some(prefix);
    }
    let outcome = crate::dsl::handlers::cold::dispatch_wide_form(dispatch, prefix);
    // Clear the prefix on the way out — the dispatcher's semantic
    // bodies consume it via their args, but the trampoline must see
    // `prefix = None` before the next instruction dispatches.
    dispatch.dispatch_state().prefix = None;
    outcome
}

/// Non-aarch64 stubs.
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

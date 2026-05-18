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
// op_wide / op_extra_wide (DSL-0c) — None layout, length = 1.
//
// The DSL-0b plan punted wide-form operand decoding for cold opcodes
// to "Batch 7" — the narrow `decode_ab!` / `decode_abc!` /
// `decode_abx!` fragments only ldrb / ldrh operands, so a Wide-prefixed
// instruction whose operands are u16 / u32 would be decoded with
// truncated bytes and an underadvanced PC, cascading PC misalignment
// through the rest of the bytecode stream.
//
// To unblock DSL-0c without re-authoring 152 wide-form decoders, the
// prefix handlers delegate the WHOLE wide-form instruction to the α
// dispatch path: the slow-path shim sets `state.prefix`, looks up the
// α handler for the byte at `PC+1`, invokes it (which decodes
// wide-form + executes the semantic + advances the frame's
// instruction_offset), then returns `Refresh` so the asm trampoline
// reloads PC from `state.frame_pc_offset`.
//
// This is a temporary bridge. Removing α (Tasks C2–C5) requires
// authoring wide-form decoders for every opcode — that's a future
// batch's work. Until then, the α dispatch table stays linked
// specifically for this delegation.
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_wide, layout = None, length = 1, || {
        call_slow!(op_wide_via_alpha_rs, args = []);
        dispatch_after_slow!();
    }
}

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_extra_wide, layout = None, length = 1, || {
        call_slow!(op_extra_wide_via_alpha_rs, args = []);
        dispatch_after_slow!();
    }
}

/// Slow-path delegate for `op_wide`. Invokes the corresponding α
/// handler at the byte following the prefix; that handler will read
/// `state.prefix`, decode wide-form operands, execute, and advance
/// `state.frame.instruction_offset` accordingly. Returns
/// [`SlowPathReturn`] with the `Refresh` tag so the asm bridge
/// reloads PC / REGS / FV from `state.frame_*`.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_wide_via_alpha_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
) -> crate::dsl::slow_path::SlowPathReturn {
    op_prefix_via_alpha(state, lyng_js_bytecode::Opcode::Wide)
}

#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_extra_wide_via_alpha_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
) -> crate::dsl::slow_path::SlowPathReturn {
    op_prefix_via_alpha(state, lyng_js_bytecode::Opcode::ExtraWide)
}

/// Shared body of `op_wide_via_alpha_rs` /
/// `op_extra_wide_via_alpha_rs`. Set the prefix, look up the α
/// handler for the byte at PC+1, invoke it, translate its `Step`
/// into a `SemanticOutcome`, return Refresh / Exit as appropriate.
#[cfg(target_arch = "aarch64")]
fn op_prefix_via_alpha(
    state: *mut crate::dsl::llint_state::LlIntState,
    prefix_opcode: lyng_js_bytecode::Opcode,
) -> crate::dsl::slow_path::SlowPathReturn {
    use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
    use crate::vm::dispatch_state::{Step, DISPATCH_TABLE};

    let mut dispatch = unsafe { LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    let outcome = {
        let dstate = dispatch.dispatch_state();
        // The α handler for the prefix opcode reads bytes[pc+1] to
        // pick the semantic opcode and bytes[pc+1..] to decode
        // wide-form operands. The prefix-α handler also handles
        // double-prefix rejection via `VmError::DoublePrefix`.
        let pc = dstate.frame.instruction_offset() as usize;
        let bytes = dstate.installed.function().instruction_bytes();
        if pc >= bytes.len() {
            SemanticOutcome::ExitError {
                error: crate::error::VmError::InstructionOutOfBounds {
                    code: dstate.frame.code(),
                    instruction_offset: dstate.frame.instruction_offset(),
                },
            }
        } else {
            let prefix_handler = DISPATCH_TABLE[prefix_opcode as u8 as usize];
            match prefix_handler(dstate) {
                Step::Continue(semantic_handler) => {
                    // The prefix α handler set state.prefix and
                    // returned the α handler for the semantic
                    // opcode. Invoke it — it decodes wide-form
                    // operands, executes, advances PC.
                    match semantic_handler(dstate) {
                        Step::Continue(_) => {
                            // The α semantic handler advanced
                            // `dstate.frame.instruction_offset`. Sync
                            // it back to `vm.frames.last_mut()` so
                            // the subsequent `Refresh` arm reloads
                            // the updated PC (instead of resetting
                            // to the pre-prefix PC stored on the
                            // canonical frame at the last sync —
                            // typically the call-entry PC for the
                            // active frame).
                            dstate.sync_active_frame();
                            SemanticOutcome::Refresh
                        }
                        Step::Done(value) => SemanticOutcome::ExitDone { value },
                        Step::Error(error) => SemanticOutcome::ExitError { error },
                    }
                }
                Step::Done(value) => SemanticOutcome::ExitDone { value },
                Step::Error(error) => SemanticOutcome::ExitError { error },
            }
        }
    };
    dispatch.translate_outcome(outcome)
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

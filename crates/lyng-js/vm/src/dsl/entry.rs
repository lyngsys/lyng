//! Entry shim and exit shim per design §5 / §6.
//!
//! The entry shim ([`run_via_dsl`]) builds an [`LlIntRustContext`] and
//! a stack-local [`LlIntState`], then jumps into the asm trampoline.
//! The trampoline runs the DSL handler chain until a cold stub writes
//! `rust_ctx.exit` and the chain unwinds back to [`_interpreter_exit`].
//!
//! DSL-0c (Task C1) replaces the DSL-0b `naked_asm!("ret")` stub with a
//! real trampoline body: save callee-saved registers, load the pinned
//! registers from `state` + the trailing args, and tail-jump to the
//! first handler. `_interpreter_exit` is the symmetric epilogue: when
//! a slow-path shim returns `SlowPathTag::Exit` and the backend's
//! `dispatch_after_slow!` does `b {exit}`, we land here, restore the
//! saved callee-saved regs, and return to `run_via_dsl`.

use std::sync::Arc;

use lyng_js_env::Agent;
use lyng_js_host::HostHooks;
use lyng_js_objects::NativeFunctionRegistry;
use lyng_js_types::Value;

use crate::dsl::handlers::{DslHandler, DSL_DISPATCH_TABLE};
use crate::dsl::llint_state::{
    ExitKind, LlIntExitSlot, LlIntRustContext, LlIntRustContextOpaque, LlIntState,
};
use crate::error::{VmError, VmResult};
use crate::vm::install::InstalledFunction;
use crate::{FrameRecord, Vm};

/// New entry point used after DSL-0c flips dispatch.
///
/// Sets up an [`LlIntState`] and an [`LlIntRustContext`] capturing the
/// current frame's PC / register window / feedback vector base, then
/// hands control to the asm trampoline. The trampoline runs the
/// dispatch chain until a slow-path shim sets `rust_ctx.exit` and the
/// chain unwinds back to [`_interpreter_exit`].
pub(crate) fn run_via_dsl(
    vm: &mut Vm,
    agent: &mut Agent,
    host: &dyn HostHooks,
    registry: &mut dyn NativeFunctionRegistry,
    installed: Arc<InstalledFunction>,
    frame: FrameRecord,
) -> VmResult<Value> {
    let frame_depth = vm.frames().len();
    let pb_base = installed.function().instruction_bytes().as_ptr();
    let frame_pc_offset = frame.instruction_offset();
    // DSL-0b (B16): wire the `FV` pin to the eagerly-allocated flat
    // feedback storage on `Vm::feedback_flat_storage`. The slot is
    // keyed by `code_index(frame.code())` and was populated to
    // `function.feedback_slot_count()` default entries at install
    // (B15). Storage is pointer-stable for the lifetime of the
    // `InstalledFunction`, so the raw pointer captured here is safe
    // to hand to the asm trampoline until `run_via_dsl` returns. Cast
    // the `*const FeedbackEntry` from `.as_ptr()` to `*mut` because
    // the asm-DSL ABI types `frame_fv_base` as `*mut FeedbackEntry`;
    // the trampoline reads/writes through it on the current thread
    // only (no aliasing UB during the trampoline's single-threaded
    // execution).
    let fv_base = {
        let index = crate::vm::code_index_for_dsl(frame.code());
        // The slot is guaranteed to exist because `store_installed`
        // populates `feedback_flat_storage[index]` to the correct
        // length before any code at `code` can be invoked. An empty
        // boxed slice's `as_ptr()` is still a valid (non-dangling)
        // pointer; the asm trampoline never dereferences past the
        // slot count anyway.
        vm.feedback_flat_storage[index].as_ptr()
            as *mut crate::dsl::feedback_flat::FeedbackEntry
    };

    // DSL-0c: REGS pin must point at the active frame's register
    // window base. Handler bodies (e.g. `op_move`, `op_add`) load
    // through `[x20, x_idx, lsl #3]` where `x_idx` is a validated
    // bytecode register index; the trampoline never dereferences past
    // `register_stack[base + window.len()]`. Computed BEFORE we move
    // `vm` into `rust_ctx` because the `&mut Vm` is consumed by the
    // borrow; `as_mut_ptr().add(base)` is well-defined even when
    // base == register_stack.len() (one-past-the-end is valid to
    // compute, just not to deref — which the handlers don't do for
    // out-of-window indices).
    let regs_base = {
        let base = frame.registers().base() as usize;
        // SAFETY: `register_stack` is grown to cover the active
        // frame's window before run_via_dsl is invoked (window
        // reservation happens at install / call entry). `add(base)`
        // is in-range (or one-past-the-end, which is well-defined).
        unsafe { vm.register_stack_storage_mut_ptr().add(base) }
    };

    let vm_ptr: *mut Vm = vm as *mut Vm;
    let frame_check_epoch = vm.dispatch_frame_check_epoch_for_dsl();

    // Build a DispatchState directly so the asm-path slow-path bridge
    // can call `LlIntDispatchState::dispatch_state()` and get the same
    // shape the α handlers use. Semantic bodies under
    // `crate::vm::semantics::` all consume `DispatchState`; threading
    // it through both dispatch paths keeps the single-implementation
    // invariant.
    let dispatch = crate::vm::dispatch_state::DispatchState::new_for_dsl_entry(
        vm,
        agent,
        host,
        registry,
        installed,
        frame,
        frame_depth,
        frame_check_epoch,
    );
    let mut rust_ctx = LlIntRustContext {
        dispatch,
        exit: LlIntExitSlot::default(),
    };

    let mut state = LlIntState {
        frame_pc_offset,
        _pad1: 0,
        frame_pb_base: pb_base,
        frame_regs_base: regs_base,
        frame_fv_base: fv_base,
        // Phase 1.B.1 Task 1: placeholders. Task 3 wires real values.
        frame_const_base: std::ptr::null(),
        frame_this_value: Value::undefined(),
        frame_depth: frame_depth as u32,
        frame_check_epoch: 0,
        rust_context: (&mut rust_ctx) as *mut LlIntRustContext<'_>
            as *mut LlIntRustContextOpaque,
        prefix: 0,
        _pad2: [0; 7],
    };

    // SAFETY: `state` is a valid mutable pointer to a stack-local
    // LlIntState; the asm trampoline only reads through it on the
    // current thread for the duration of this call. `vm_ptr` aliases
    // `rust_ctx.vm` but the trampoline only dereferences `vm_ptr` via
    // the VM pin (`x22`) for the immutable `dsl_poll_pending` byte —
    // it never writes through it. `DSL_DISPATCH_TABLE` is a `pub
    // static [DslHandler; 256]` with stable storage for the entire
    // program lifetime.
    unsafe {
        run_dsl_trampoline(
            &mut state as *mut LlIntState,
            vm_ptr,
            DSL_DISPATCH_TABLE.as_ptr(),
        )
    };

    match rust_ctx.exit.kind {
        ExitKind::Done => Ok(rust_ctx.exit.done_value),
        ExitKind::Error => Err(*rust_ctx.exit.error.take().unwrap()),
        ExitKind::None => Err(VmError::TrampolineExitedWithoutSetting),
    }
}

// =============================================================
// The asm trampoline + exit shim.
//
// Both functions share a 96-byte stack frame:
//   [sp + 0]  x19, x20   ← PC, REGS
//   [sp + 16] x21, x22   ← FV, VM
//   [sp + 32] x23, x24   ← TABLE, STATE
//   [sp + 48] x25, x26   ← reserved (handler scratch, currently
//                          unused by the substrate but saved per
//                          AAPCS64 so callees can use them)
//   [sp + 64] x27, x28   ← ditto
//   [sp + 80] x29, x30   ← saved FP, LR (caller's return address)
//
// `run_dsl_trampoline` writes this frame on entry; `_interpreter_exit`
// reverses it on exit. The handler chain in between maintains
// `sp == frame-from-entry` (handlers may use the red zone but must
// not move sp). Slow-path shims (Rust `extern "C"` fns called via
// `bl` inside handlers) get a fresh AAPCS64-compliant stack frame
// from rustc's own prologue/epilogue, so they don't disturb ours.
// =============================================================

/// Asm trampoline entry. Saves callee-saveds, loads pinned registers
/// from `state` / `vm` / `table`, then tail-dispatches to the handler
/// at `DSL_DISPATCH_TABLE[bytes[pc]]`.
///
/// Arguments (AAPCS64):
/// - `x0` = `*mut LlIntState`
/// - `x1` = `*mut Vm` (used to pin `VM` for `poll_safepoint!` reads;
///   the trampoline does not deref it itself)
/// - `x2` = `*const DslHandler` (`DSL_DISPATCH_TABLE` base)
///
/// Pinned registers (design §5):
/// | Pin | Reg | Holds |
/// |---|---|---|
/// | PC | x19 | `pb_base + pc_offset` (live byte in bytecode) |
/// | REGS | x20 | `*mut Value` (register-file base) |
/// | FV | x21 | `*mut FeedbackEntry` (feedback-vector base) |
/// | VM | x22 | `*mut Vm` |
/// | TABLE | x23 | `*const DslHandler` |
/// | STATE | x24 | `*mut LlIntState` |
#[unsafe(naked)]
pub unsafe extern "C" fn run_dsl_trampoline(
    _state: *mut LlIntState,
    _vm: *mut Vm,
    _table: *const DslHandler,
) {
    core::arch::naked_asm!(
        // Build a 96-byte frame on the stack; save FP, LR, and
        // callee-saved regs we're about to clobber for pins.
        "sub    sp, sp, #96",
        "stp    x19, x20, [sp, #0]",
        "stp    x21, x22, [sp, #16]",
        "stp    x23, x24, [sp, #32]",
        "stp    x25, x26, [sp, #48]",
        "stp    x27, x28, [sp, #64]",
        "stp    x29, x30, [sp, #80]",
        "mov    x29, sp",
        // Pin assignment.
        "mov    x24, x0",                       // STATE = state
        "mov    x22, x1",                       // VM    = vm
        "mov    x23, x2",                       // TABLE = table
        "ldr    x9,  [x24, {state_pb}]",        // x9 = pb_base
        "ldr    w10, [x24, {state_pc}]",        // w10 = pc_offset (u32)
        "add    x19, x9, x10",                  // PC = pb_base + pc_offset
        "ldr    x20, [x24, {state_regs}]",      // REGS = state.frame_regs_base
        "ldr    x21, [x24, {state_fv}]",        // FV   = state.frame_fv_base
        // Tail-dispatch to the first handler.
        "ldrb   w8, [x19]",                     // w8 = opcode byte
        "ldr    x16, [x23, x8, lsl #3]",        // x16 = TABLE[opcode]
        "br     x16",
        // Note: control does not return here from a handler. The
        // handler chain ends via a `b {exit}` to `_interpreter_exit`
        // (issued by `dispatch_after_slow!`), which restores the
        // stack frame this function built and `ret`s.
        state_pb = const crate::dsl::reg_convention::LLINT_STATE_FRAME_PB_BASE,
        state_pc = const crate::dsl::reg_convention::LLINT_STATE_FRAME_PC_OFFSET,
        state_regs = const crate::dsl::reg_convention::LLINT_STATE_FRAME_REGS_BASE,
        state_fv = const crate::dsl::reg_convention::LLINT_STATE_FRAME_FV_BASE,
    );
}

/// Exit shim. Reached via `b {exit}` from `dispatch_after_slow!`
/// inside a handler. Restores the callee-saved registers the
/// trampoline saved on entry and returns to the caller of
/// `run_dsl_trampoline` (`run_via_dsl`).
///
/// `_interpreter_exit` is naked because the branch into it is a tail
/// (`b`), not a call (`bl`) — there's no fresh return slot to write
/// into, and rustc-generated prologues/epilogues would corrupt the
/// trampoline's frame. The asm here mirrors `run_dsl_trampoline`'s
/// entry sequence in reverse.
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn _interpreter_exit() {
    core::arch::naked_asm!(
        // Restore callee-saved regs from the trampoline's frame.
        "ldp    x29, x30, [sp, #80]",
        "ldp    x27, x28, [sp, #64]",
        "ldp    x25, x26, [sp, #48]",
        "ldp    x23, x24, [sp, #32]",
        "ldp    x21, x22, [sp, #16]",
        "ldp    x19, x20, [sp, #0]",
        "add    sp, sp, #96",
        "ret",
    );
}

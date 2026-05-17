//! Warm DSL handlers. Populated by tasks B43–B45.
//!
//! Warm handlers are mid-frequency opcodes that need either a backedge
//! safepoint poll (`op_loop_header`, conditional backward jumps) or a
//! prefix decode (`op_wide`, `op_extra_wide`). They run on top of the
//! same backend macros as the hot handlers; the distinction is
//! categorical (used to determine inlining heuristics in the DSL
//! optimizer + dispatch table organization later in DSL-1).

#[cfg(target_arch = "aarch64")]
use crate::{call_slow, decode_ax, dispatch, dispatch_after_slow, poll_safepoint};

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

/// Non-aarch64 stub.
#[cfg(not(target_arch = "aarch64"))]
pub unsafe extern "C" fn op_loop_header() -> ! {
    loop {}
}

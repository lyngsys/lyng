//! Same-thread safepoint poll consumer.
//!
//! When `poll_safepoint!` sees a non-zero pending byte, the asm branches
//! to a slow-path shim that calls `run_poll`. `run_poll` runs incremental
//! GC, debugger pauses, etc., then returns `Continue` so execution
//! advances past the poll site.

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};

/// Args struct for `run_poll`. Unit struct so the `dsl_cold_shim!` syntax
/// stays uniform across all opcode shims.
pub struct PollArgs;

/// Run pending VM work (GC step, debugger pause) and return `Continue { 4 }`
/// so execution advances past the `op_loop_header` poll site. Returning
/// `Refresh` would re-dispatch at the same PC and re-fire the same pause.
pub fn run_poll(state: &mut LlIntDispatchState<'_, '_>, _args: PollArgs) -> SemanticOutcome {
    let inner = state.dispatch_state();
    // Sync the live PC into the overlay so `poll_debug_safepoint` sees the
    // current loop-header offset, not the stale call-entry PC.
    inner.sync_active_frame();
    {
        let crate::vm::dispatch_state::DispatchState { vm, agent, .. } = &mut *inner;
        crate::vm::Vm::poll_incremental_mark_safepoint(agent);
        vm.poll_debug_safepoint(agent, crate::vm::VmDebugSafepointKind::LoopHeader);
    }
    inner.refresh_dsl_poll_pending();
    // op_loop_header is 4 bytes; advance past it after the hook resumes.
    SemanticOutcome::Continue { pc_advance: 4 }
}

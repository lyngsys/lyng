//! Same-thread safepoint poll consumer per design §6.
//!
//! `poll_safepoint!` in the asm DSL only checks a single byte for
//! pending work; when the byte is non-zero, the asm branches to a
//! local label that calls into this module's `run_poll` (via a
//! `dsl_cold_shim!`-style wrapper). `run_poll` consumes the pending
//! bits — incremental GC work, debugger pause requests, etc.
//!
//! DSL-0c wiring: the slow-path arm of the warm `op_loop_header`
//! handler funnels through `run_poll`. The function syncs the frame
//! from asm, hands off to `Vm::poll_debug_safepoint` (which inspects
//! `debug_state` and may invoke the host's `VmDebugHook`), then
//! refreshes `dsl_poll_pending` based on the post-pause state.
//! Returns `Refresh` so the asm bridge reloads PC/REGS/FV — the
//! debug hook may have stepped, set a new pause request, or moved
//! the active frame via host introspection. (`Continue { 0 }` would
//! work for the simple case but `Refresh` keeps the bridge state
//! correct regardless of what the hook did.)

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};

/// Args struct for `run_poll`. Empty — the poll consumer reads all
/// state from the Vm. Kept as a unit struct rather than `()` so the
/// `dsl_cold_shim!` macro's `<args>` syntax stays uniform across
/// every opcode shim.
pub struct PollArgs;

/// Read the VM's pending-work bits and run the requested work
/// (incremental GC step, debugger pause). Returns `Continue` with the
/// `op_loop_header` instruction length so execution advances past the
/// poll site after the hook resumes — mirrors α's
/// `op_loop_header_semantic`, which returns `Continue { pc_advance =
/// instruction_len }` after the debug poll. Returning `Refresh` would
/// re-dispatch at the same PC; if the hook installed a step command,
/// `poll_pending` stays set and the asm slow path would re-fire the
/// same pause record in a loop.
pub fn run_poll(
    state: &mut LlIntDispatchState<'_, '_>,
    _args: PollArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    // Mirror α's `op_loop_header_semantic`: write the dispatch state's
    // frame back to `vm.frames.last_mut()` so the safepoint poll
    // observes the live PC. Without this, `poll_debug_safepoint` reads
    // a stale `vm.frames.last().instruction_offset` (the entry PC of
    // the active frame, last synced at call entry), so a request_pause
    // for the loop-header offset never matches the safepoint kind +
    // offset built from the active frame.
    inner.sync_active_frame();
    {
        let crate::vm::dispatch_state::DispatchState { vm, agent, .. } = &mut *inner;
        vm.poll_debug_safepoint(agent, crate::vm::VmDebugSafepointKind::LoopHeader);
    }
    // op_loop_header is encoded as 4 bytes (`length = 4` in the warm
    // handler). After the hook resumes — whether via Resume / StepIn /
    // StepOver / StepOut — execution advances past the loop-header to
    // the next instruction. Subsequent step pauses fire at the next
    // safepoint kind (FunctionEntry for StepIn into a bytecode call,
    // LoopHeader for StepOver/Out at the next backedge).
    SemanticOutcome::Continue { pc_advance: 4 }
}

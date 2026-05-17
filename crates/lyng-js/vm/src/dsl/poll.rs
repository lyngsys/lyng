//! Same-thread safepoint poll consumer per design §6.
//!
//! `poll_safepoint!` in the asm DSL only checks a single byte for
//! pending work; when the byte is non-zero, the asm branches to a
//! local label that calls into this module's `run_poll` (via a
//! `dsl_cold_shim!`-style wrapper). `run_poll` consumes the pending
//! bits — incremental GC work, debugger pause requests, etc.
//!
//! For DSL-0b the body is a stub: the Vm doesn't yet carry a
//! `poll_pending` field. The cold-stub still exists so the asm
//! handler chain can link against the slow-path symbol. Real work
//! lands when the GC + debugger integration arrives.

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};

/// Args struct for `run_poll`. Empty — the poll consumer reads all
/// state from the Vm. Kept as a unit struct rather than `()` so the
/// `dsl_cold_shim!` macro's `<args>` syntax stays uniform across
/// every opcode shim.
pub struct PollArgs;

/// Read the VM's pending-work bits and run the requested work
/// (incremental GC step, debugger pause). Returns `Continue` so the
/// asm trampoline tail-jumps to the next handler.
///
/// In DSL-0b the body is a no-op: the Vm carries no poll-pending
/// field yet, so the asm path reading through `{vm_poll}` (which
/// resolves to offset 0) sees whatever happens to be at that
/// location. Production safety relies on the trampoline being dead
/// code until DSL-0c flips dispatch.
pub fn run_poll(
    _state: &mut LlIntDispatchState<'_, '_>,
    _args: PollArgs,
) -> SemanticOutcome {
    // No real work to do until GC + debugger integrations land.
    SemanticOutcome::Continue { pc_advance: 0 }
}

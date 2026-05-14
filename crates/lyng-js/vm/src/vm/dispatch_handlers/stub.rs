//! Catch-all handler for opcode bytes the spike doesn't implement.
//!
//! Phase 1 proper replaces this with real handlers for every opcode plus a
//! genuinely unreachable trap for invalid bytes. The spike just wants a
//! cleanly-returning placeholder so `DISPATCH_TABLE` has 256 entries.

use crate::error::VmError;

use crate::vm::dispatch_state::{DispatchState, Step};

#[cold]
pub extern "C" fn op_stub(_state: &mut DispatchState) -> Step {
    Step::Error(VmError::MissingActiveFrame)
}

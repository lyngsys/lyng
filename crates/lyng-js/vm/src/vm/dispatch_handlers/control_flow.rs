//! Control-flow handlers — branches and returns.
//!
//! Encoding for the spike:
//! - `op_jump_back`: `[opcode, abs_target]` (2 bytes) — sets `pc` to
//!   `abs_target` (1-byte absolute offset, sufficient for the spike's tiny
//!   bytecode buffer).
//! - `op_return`: `[opcode, src]` (2 bytes) — returns `regs[src]` via
//!   `Step::Done`.

use crate::dispatch_next;
use crate::vm::dispatch_state::{DispatchState, Step};

pub extern "C" fn op_jump_back(state: &mut DispatchState) -> Step {
    let header: &[u8; 2] = state.current_bytes()[..2]
        .try_into()
        .expect("op_jump_back: encoding invariant — at least 2 bytes from pc");
    let target = u32::from(header[1]);
    state.pc = target;
    dispatch_next!(state);
}

pub extern "C" fn op_return(state: &mut DispatchState) -> Step {
    let header: &[u8; 2] = state.current_bytes()[..2]
        .try_into()
        .expect("op_return: encoding invariant — at least 2 bytes from pc");
    let src = u16::from(header[1]);
    Step::Done(state.read_register(src))
}

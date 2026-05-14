//! Load/move handlers — moving values between registers and writing constants.
//!
//! Encoding for the spike:
//! - `op_move`: `[opcode, dst, src]` (3 bytes)
//! - `op_load_undefined`: `[opcode, dst]` (2 bytes)

use lyng_js_types::Value;

use crate::dispatch_next;
use crate::vm::dispatch_state::{DispatchState, Step};

pub extern "C" fn op_move(state: &mut DispatchState) -> Step {
    let header: &[u8; 3] = state.current_bytes()[..3]
        .try_into()
        .expect("op_move: encoding invariant — at least 3 bytes from pc");
    let dst = u16::from(header[1]);
    let src = u16::from(header[2]);
    let value = state.read_register(src);
    state.write_register(dst, value);
    state.advance(3);
    dispatch_next!(state);
}

pub extern "C" fn op_load_undefined(state: &mut DispatchState) -> Step {
    let header: &[u8; 2] = state.current_bytes()[..2]
        .try_into()
        .expect("op_load_undefined: encoding invariant — at least 2 bytes from pc");
    let dst = u16::from(header[1]);
    state.write_register(dst, Value::undefined());
    state.advance(2);
    dispatch_next!(state);
}

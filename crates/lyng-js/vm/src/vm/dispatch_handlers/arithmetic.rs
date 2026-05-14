//! Arithmetic handlers — SMI fast path with cold fallback.
//!
//! Encoding for the spike (matches the roadmap's `op_add` example shape with
//! 1-byte register IDs and a 2-byte feedback slot for ABI fidelity):
//! - `op_add`: `[opcode, dst, lhs, rhs, slot_lo, slot_hi]` (6 bytes)
//!
//! The slow path is a stand-in that returns `Step::Error` — in Phase 1
//! proper it delegates to `execute_add_opcode` for boxed-number /
//! string-coercion / ToPrimitive paths. What the spike wants to verify is
//! that the SMI fast path tail lowers to a clean indirect-jump dispatch.

use lyng_js_types::Value;

use crate::dispatch_next;
use crate::error::VmError;
use crate::vm::dispatch_state::{DispatchState, Step};

pub extern "C" fn op_add(state: &mut DispatchState) -> Step {
    let header: &[u8; 6] = state.current_bytes()[..6]
        .try_into()
        .expect("op_add: encoding invariant — at least 6 bytes from pc");
    let a = u16::from(header[1]);
    let b = u16::from(header[2]);
    let c = u16::from(header[3]);
    let slot_raw = u16::from_le_bytes([header[4], header[5]]);

    let left = state.read_register(b);
    let right = state.read_register(c);
    if let (Some(l), Some(r)) = (left.as_smi(), right.as_smi())
        && let Some(v) = l.checked_add(r)
    {
        state.record_feedback_arithmetic_smi(slot_raw);
        state.write_register(a, Value::from_smi(v));
        state.advance(6);
        dispatch_next!(state);
    }
    op_add_slow(state, a, b, c, slot_raw)
}

#[cold]
fn op_add_slow(_state: &mut DispatchState, _a: u16, _b: u16, _c: u16, _slot: u16) -> Step {
    Step::Error(VmError::MissingActiveFrame)
}

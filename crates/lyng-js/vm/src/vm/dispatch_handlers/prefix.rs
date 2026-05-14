//! Wide / ExtraWide prefix handlers.
//!
//! When the bytecode emitter needs to express an operand wider than its
//! narrow form can fit (registers ≥ 256 or jump deltas ≥ ±128 / signed-i24),
//! it prepends a `Wide` or `ExtraWide` byte. In the trampoline dispatch path
//! the prefix is its own opcode: `op_wide` / `op_extra_wide` set
//! `state.prefix` and re-dispatch to the semantic opcode handler at
//! `bytes[pc+1]`. Each semantic handler consumes the prefix via
//! `state.prefix.take()` so the next dispatch starts fresh.

use lyng_js_bytecode::Opcode;

use crate::error::VmError;
use crate::vm::dispatch_state::{DispatchState, Step, DISPATCH_TABLE};

#[inline]
fn dispatch_prefixed(state: &mut DispatchState, prefix: Opcode) -> Step {
    let semantic_byte = match state.current_bytes().get(1).copied() {
        Some(b) => b,
        None => {
            return Step::Error(VmError::InstructionOutOfBounds {
                code: state.frame.code(),
                instruction_offset: state.frame.instruction_offset(),
            });
        }
    };

    // Reject double-prefixes (Wide; Wide; ...) — the encoder never produces
    // them and accepting them would silently mask a corrupted bytecode
    // stream.
    if semantic_byte == Opcode::Wide as u8 || semantic_byte == Opcode::ExtraWide as u8 {
        return Step::Error(VmError::InstructionOutOfBounds {
            code: state.frame.code(),
            instruction_offset: state.frame.instruction_offset(),
        });
    }

    state.prefix = Some(prefix);
    Step::Continue(DISPATCH_TABLE[semantic_byte as usize])
}

pub extern "C" fn op_wide(state: &mut DispatchState) -> Step {
    dispatch_prefixed(state, Opcode::Wide)
}

pub extern "C" fn op_extra_wide(state: &mut DispatchState) -> Step {
    dispatch_prefixed(state, Opcode::ExtraWide)
}

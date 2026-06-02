//! Exceptions family semantic bodies.
//!
//! Family coverage (4 opcodes):
//! - `Throw` — routes through `Vm::transfer_to_exception_handler`; returns
//!   `Refresh` on catch, `ExitError { Abrupt(Throw) }` if uncaught.
//! - `EnterHandler` / `LeaveHandler` — dispatch-time markers; just advance PC.
//!   The active try-stack is `ExceptionHandler` metadata on the installed
//!   function, not runtime state.
//! - `LoadException` — reads `current_exception` (folded to `undefined` if
//!   `None`) into the destination register; does not clear the slot.

use lyng_types::AbruptCompletion;

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::error::VmError;
use crate::vm::dispatch_state::DispatchState;

// =====================================================================
// Shared operand shapes
// =====================================================================

/// Operands for the Ax-encoded exceptions opcodes: `Throw`,
/// `EnterHandler`, `LeaveHandler`, `LoadException`.
/// For `Throw`, `register` holds the value to throw.
/// For `LoadException`, `register` is the destination.
/// `instruction_len` is the PC advance (unused by `Throw` on the throw path).
pub struct OpExceptionsAxArgs {
    pub register: u16,
    pub instruction_len: u32,
}

/// Operands for the handler-marker opcodes (`EnterHandler`, `LeaveHandler`).
pub struct OpHandlerMarkerArgs {
    pub instruction_len: u32,
}

// =====================================================================
// Throw
// =====================================================================

pub fn op_throw_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpExceptionsAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner
        .vm
        .read_register_unchecked(inner.registers(), args.register);
    inner.sync_active_frame();
    let transferred = {
        let DispatchState { vm, agent, .. } = &mut *inner;
        vm.transfer_to_exception_handler(agent, value)
    };
    match transferred {
        Ok(true) => SemanticOutcome::Refresh,
        Ok(false) => SemanticOutcome::ExitError {
            error: VmError::Abrupt(AbruptCompletion::Throw(value)),
        },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

// =====================================================================
// EnterHandler / LeaveHandler — dispatch markers; just advance PC.
// =====================================================================

pub const fn op_enter_handler_semantic(
    _state: &mut LlIntDispatchState<'_, '_>,
    args: OpHandlerMarkerArgs,
) -> SemanticOutcome {
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub const fn op_leave_handler_semantic(
    _state: &mut LlIntDispatchState<'_, '_>,
    args: OpHandlerMarkerArgs,
) -> SemanticOutcome {
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// LoadException — reads `current_exception` into `register`.
// =====================================================================

pub fn op_load_exception_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpExceptionsAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner.vm.current_exception_value();
    let registers = inner.registers();
    inner
        .vm
        .write_register_unchecked(registers, args.register, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

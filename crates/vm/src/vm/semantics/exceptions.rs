//! Exceptions family semantic bodies (DSL-0a Task A17).
//!
//! Each `op_xxx_semantic` function implements the semantic effect of one
//! exceptions-family opcode. The α handler in
//! `dispatch_handlers/exceptions.rs` decodes operands, constructs
//! `OpExceptionsXxxArgs`, calls the semantic body, and translates the
//! returned `SemanticOutcome` to `Step` via `translate_outcome_to_step`.
//! The DSL-0b cold-stub shim in `dsl/handlers/cold/exceptions.rs` will
//! reach the same functions from the asm-DSL path.
//!
//! Family coverage (4 opcodes):
//! - Throw routing (Ax): `Throw`.
//! - Handler-stack markers (Ax): `EnterHandler`, `LeaveHandler`.
//! - Pending-exception read (Ax): `LoadException`.
//!
//! ### `Throw` routing
//!
//! `Vm::transfer_to_exception_handler` walks the live frame stack and,
//! for each frame, either selects an active handler covering the current
//! PC (writes the catch target onto the frame's instruction offset,
//! installs `current_exception`, returns `Ok(true)`) or unwinds the
//! frame and continues. If no handler is found and the bottom frame is
//! reached, it returns `Ok(false)` so the caller surfaces the throw as
//! an abrupt completion that escapes the current `Vm::run`. The α
//! handler maps these as:
//!   - `Ok(true)`  → `refresh_from_active_frame()` + `dispatch_next!`
//!   - `Ok(false)` → `Step::Error(VmError::Abrupt(AbruptCompletion::Throw(value)))`
//!   - `Err(e)`    → `Step::Error(e)`
//!
//! The semantic body preserves this exactly:
//!   - `Ok(true)`  → `SemanticOutcome::Refresh` — `translate_outcome_to_step`
//!     runs `refresh_from_active_frame` and reads the next opcode at the
//!     (now-rewritten) handler PC, mirroring the α `dispatch_next!` tail.
//!   - `Ok(false)` → `SemanticOutcome::ExitError { error: Abrupt(Throw(value)) }`.
//!   - `Err(e)`    → `SemanticOutcome::ExitError { error: e }`.
//!
//! ### `EnterHandler` / `LeaveHandler`
//!
//! These are dispatch-time markers that just advance PC; the active
//! try-stack is encoded as `ExceptionHandler` metadata on the installed
//! function (read by `transfer_to_exception_handler` at throw time), not
//! as runtime state managed by the dispatch loop. Both bodies return
//! `Continue { pc_advance: instruction_len }`.
//!
//! ### `LoadException`
//!
//! Reads `Vm::current_exception` via `current_exception_value()` (which
//! folds `None` to `undefined`) and writes it to the destination
//! register. Does *not* clear the slot — the slot is cleared by
//! `unwind_exception_frame` (cross-frame catch) and by the bytecode
//! emitter's explicit handler-leave sequence. Returns
//! `Continue { pc_advance: instruction_len }`.

use lyng_types::AbruptCompletion;

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::error::VmError;
use crate::vm::dispatch_state::DispatchState;

// =====================================================================
// Shared operand shapes
// =====================================================================

/// Operands for the Ax-encoded exceptions opcodes: `Throw`,
/// `EnterHandler`, `LeaveHandler`, `LoadException`.
///
/// - `Throw`: `register` holds the value to throw; `instruction_len` is
///   unused on the throw path (PC is rewritten by
///   `transfer_to_exception_handler` on catch, or `Vm::run` exits on
///   uncaught), but kept on the args for shape uniformity with the rest
///   of the family.
/// - `EnterHandler` / `LeaveHandler`: `register` is decoded but ignored
///   (the α handler reads `ax` only to compute `instruction_len`).
/// - `LoadException`: `register` is the destination; `instruction_len`
///   is the PC advance.
pub struct OpExceptionsAxArgs {
    pub register: u16,
    pub instruction_len: u32,
}

/// Operands for the handler-marker opcodes (`EnterHandler`,
/// `LeaveHandler`). Neither body reads the `ax` operand — only
/// `instruction_len` matters for the PC advance.
pub struct OpHandlerMarkerArgs {
    pub instruction_len: u32,
}

// =====================================================================
// Throw — Ax; routes the thrown value through
// `Vm::transfer_to_exception_handler`. On a same-frame or cross-frame
// catch, the helper rewrites the active frame's PC to the handler
// target; the semantic returns `Refresh` so the dispatcher reloads the
// pinned PC/REGS/FV from the canonical frame state. On an uncaught
// throw (no handler covers any active frame), the helper returns
// `Ok(false)` and the semantic returns `ExitError { Abrupt(Throw) }`
// to escape `Vm::run`.
// =====================================================================

pub(crate) fn op_throw_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpExceptionsAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner
        .vm
        .read_register_unchecked(inner.frame.registers(), args.register);
    inner.sync_active_frame();
    let transferred = {
        let DispatchState { vm, agent, .. } = &mut *inner;
        vm.transfer_to_exception_handler(agent, value)
    };
    match transferred {
        // Caught: helper rewrote the active frame's PC to the handler
        // target. `Refresh` makes `translate_outcome_to_step` re-pin
        // PC/REGS/FV from the canonical frame state, exactly mirroring
        // the α handler's `refresh_from_active_frame()` + `dispatch_next!`.
        Ok(true) => SemanticOutcome::Refresh,
        // Uncaught: no active handler matches across any live frame.
        // Surface as the abrupt-completion that escapes `Vm::run`.
        Ok(false) => SemanticOutcome::ExitError {
            error: VmError::Abrupt(AbruptCompletion::Throw(value)),
        },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

// =====================================================================
// EnterHandler / LeaveHandler — Ax; dispatch markers that just advance
// PC. The bytecode emitter encodes the active try-stack via
// `ExceptionHandler` metadata on the installed function (read at throw
// time by `select_exception_handler`), so the markers carry no runtime
// state of their own.
// =====================================================================

pub(crate) fn op_enter_handler_semantic(
    _state: &mut LlIntDispatchState<'_, '_>,
    args: OpHandlerMarkerArgs,
) -> SemanticOutcome {
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub(crate) fn op_leave_handler_semantic(
    _state: &mut LlIntDispatchState<'_, '_>,
    args: OpHandlerMarkerArgs,
) -> SemanticOutcome {
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// LoadException — Ax; reads `Vm::current_exception` (folded to
// `undefined` if `None`) and writes it to `register`. Does not clear
// the slot — clearing happens in `unwind_exception_frame` or via the
// emitter's explicit handler-leave sequence.
// =====================================================================

pub(crate) fn op_load_exception_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpExceptionsAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner.vm.current_exception_value();
    let registers = inner.frame.registers();
    inner
        .vm
        .write_register_unchecked(registers, args.register, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

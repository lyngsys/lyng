//! Control-flow family semantic bodies.
//!
//! Family coverage (10 opcodes):
//! - Unconditional jumps: `Jump`, `Jump8`.
//! - Conditional jumps: `JumpIfTrue`, `JumpIfTrue8`, `JumpIfFalse`,
//!   `JumpIfFalse8`.
//! - `LoopHeader` — incremental-mark safepoint + optional debug-poll.
//! - `Return`, `ReturnUndefined` — pop the active frame.
//! - `Nop` — advance PC.
//!
//! PC-advance convention: every jumping semantic returns
//! `Continue { pc_advance: instruction_len + delta }` so `state.advance`
//! produces the correct absolute target. Non-branching paths return
//! `Continue { pc_advance: instruction_len }`.
//!
//! `JumpIfTrue` / `JumpIfFalse` route `to_boolean_agent` through
//! `handle_dispatch_result`: `Ok(Some(truthy))` proceeds, `Ok(None)` means
//! the throw was caught (return `Continue { pc_advance: 0 }`), `Err` escapes.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "Control-flow semantics intentionally wrap signed relative PC advances into the u32 advance representation after absolute-target validation"
)]

use lyng_ops::read;
use lyng_types::Value;

use crate::VmDebugSafepointKind;
use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::error::VmError;
use crate::vm::Vm;

// =====================================================================
// Jumps (unconditional) — `Jump`, `Jump8`.
// =====================================================================

/// Operands for an unconditional jump. `delta` is the sign-extended relative
/// offset; `instruction_len` is the encoded instruction length.
pub struct OpJumpArgs {
    pub delta: i32,
    pub instruction_len: u32,
}

/// Shared body for `Jump` / `Jump8`. Polls the incremental-mark safepoint on
/// backedges. Returns `Continue { pc_advance: instruction_len + delta }`.
fn op_jump_shared_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpJumpArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    if args.delta < 0 {
        Vm::poll_incremental_mark_safepoint(inner.agent);
    }
    let instruction_offset = inner.pc();
    let target =
        i64::from(instruction_offset) + i64::from(args.instruction_len) + i64::from(args.delta);
    if target < 0 || target > i64::from(u32::MAX) {
        return SemanticOutcome::ExitError {
            error: VmError::InvalidJumpTarget {
                code: inner.code(),
                instruction_offset,
                target_offset: target,
            },
        };
    }
    // Wrapping cast is safe: the overflow check above guarantees the target fits in u32.
    let pc_advance = (i64::from(args.instruction_len) + i64::from(args.delta)) as u32;
    SemanticOutcome::Continue { pc_advance }
}

pub fn op_jump_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpJumpArgs,
) -> SemanticOutcome {
    op_jump_shared_semantic(state, args)
}

pub fn op_jump8_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpJumpArgs,
) -> SemanticOutcome {
    op_jump_shared_semantic(state, args)
}

// =====================================================================
// Conditional jumps — `JumpIfTrue`, `JumpIfTrue8`, `JumpIfFalse`,
// `JumpIfFalse8`.
// =====================================================================

/// Operands for a conditional jump. `condition_register` holds the value to
/// test; `delta` is the sign-extended relative offset.
pub struct OpJumpIfArgs {
    pub condition_register: u16,
    pub delta: i32,
    pub instruction_len: u32,
}

/// Shared body for the four conditional-jump variants. `take_if_truthy`
/// selects between `JumpIfTrue` (`true`) and `JumpIfFalse` (`false`).
fn op_jump_if_shared_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpJumpIfArgs,
    take_if_truthy: bool,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let condition = inner
        .vm
        .read_register_unchecked(inner.registers(), args.condition_register);
    let truthy_result = read::to_boolean_agent(inner.agent, condition).map_err(VmError::Abrupt);
    let truthy = match inner.handle_dispatch_result(truthy_result) {
        Ok(Some(t)) => t,
        Ok(None) => {
            // Caught: handler PC already rewritten; resume with no advance.
            return SemanticOutcome::Continue { pc_advance: 0 };
        }
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let should_jump = if take_if_truthy { truthy } else { !truthy };
    if should_jump {
        if args.delta < 0 {
            Vm::poll_incremental_mark_safepoint(inner.agent);
        }
        let instruction_offset = inner.pc();
        let target =
            i64::from(instruction_offset) + i64::from(args.instruction_len) + i64::from(args.delta);
        if target < 0 || target > i64::from(u32::MAX) {
            return SemanticOutcome::ExitError {
                error: VmError::InvalidJumpTarget {
                    code: inner.code(),
                    instruction_offset,
                    target_offset: target,
                },
            };
        }
        let pc_advance = (i64::from(args.instruction_len) + i64::from(args.delta)) as u32;
        SemanticOutcome::Continue { pc_advance }
    } else {
        SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        }
    }
}

pub fn op_jump_if_true_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpJumpIfArgs,
) -> SemanticOutcome {
    op_jump_if_shared_semantic(state, args, true)
}

pub fn op_jump_if_true8_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpJumpIfArgs,
) -> SemanticOutcome {
    op_jump_if_shared_semantic(state, args, true)
}

pub fn op_jump_if_false_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpJumpIfArgs,
) -> SemanticOutcome {
    op_jump_if_shared_semantic(state, args, false)
}

pub fn op_jump_if_false8_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpJumpIfArgs,
) -> SemanticOutcome {
    op_jump_if_shared_semantic(state, args, false)
}

// =====================================================================
// LoopHeader — marker + tier-backedge + incremental-mark + debug poll.
// =====================================================================

pub struct OpLoopHeaderArgs {
    pub instruction_len: u32,
}

pub fn op_loop_header_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoopHeaderArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    if inner.vm.debug_poll_enabled() {
        inner.sync_active_frame();
        {
            let crate::vm::dispatch_state::DispatchState { vm, agent, .. } = &mut *inner;
            vm.poll_debug_safepoint(agent, VmDebugSafepointKind::LoopHeader);
        }
        if let Err(error) = inner.refresh_from_active_frame() {
            return SemanticOutcome::ExitError { error };
        }
    }
    Vm::poll_incremental_mark_safepoint(inner.agent);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// Return / ReturnUndefined — frame-transitioning.
// =====================================================================

pub struct OpReturnArgs {
    /// Decoded `ax` operand. `Return` reads its return value from this
    /// register; `ReturnUndefined` ignores `register` and uses `undefined`.
    pub register: u16,
}

pub struct OpReturnUndefinedArgs;

/// Shared epilogue for `Return` / `ReturnUndefined`.
fn op_return_finish_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    value: Value,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    inner.sync_active_frame();
    match inner.finish_active_frame(value) {
        Ok(Some(result)) => SemanticOutcome::ExitDone { value: result },
        Ok(None) => SemanticOutcome::Refresh,
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

pub fn op_return_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpReturnArgs,
) -> SemanticOutcome {
    let value = {
        let inner = state.dispatch_state();
        inner
            .vm
            .read_register_unchecked(inner.registers(), args.register)
    };
    op_return_finish_semantic(state, value)
}

pub fn op_return_undefined_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    _args: OpReturnUndefinedArgs,
) -> SemanticOutcome {
    op_return_finish_semantic(state, Value::undefined())
}

// =====================================================================
// Nop — advance PC; no other side effects.
// =====================================================================

pub struct OpNopArgs {
    pub instruction_len: u32,
}

pub const fn op_nop_semantic(
    _state: &mut LlIntDispatchState<'_, '_>,
    args: OpNopArgs,
) -> SemanticOutcome {
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

//! Control-flow family semantic bodies (DSL-0a Task A10).
//!
//! Each `op_xxx_semantic` function implements the semantic effect of one
//! control-flow opcode. The α handler in `dispatch_handlers/control_flow.rs`
//! decodes operands, constructs `OpXxxArgs`, calls the semantic body, and
//! translates the returned `SemanticOutcome` to `Step`. The DSL-0b cold-stub
//! shim in `dsl/handlers/cold/control_flow.rs` will reach the same functions
//! from the asm-DSL path.
//!
//! Family coverage (10 opcodes):
//! - Unconditional jumps: `Jump`, `Jump8`.
//! - Conditional jumps: `JumpIfTrue`, `JumpIfTrue8`, `JumpIfFalse`,
//!   `JumpIfFalse8`.
//! - `LoopHeader` — marker plus tier-backedge + incremental-mark safepoint
//!   (and a debug-poll safepoint when the debug hook is installed).
//! - `Return`, `ReturnUndefined` — pop the active frame.
//! - `Nop` — no-op; advance PC.
//!
//! ### PC-advance convention
//!
//! The α handler `jump_dispatch_frame` sets `pc = current_pc + instruction_len
//! + delta`. The `translate_outcome_to_step` `Continue` arm calls
//! `state.advance(pc_advance)`, which adds `pc_advance` to the *current* PC
//! (the entry PC at the start of the handler). Therefore every jumping
//! semantic body must return `Continue { pc_advance: instruction_len + delta }`
//! to reproduce the absolute target the α handler would have computed.
//! Non-branching paths return `Continue { pc_advance: instruction_len }`.
//!
//! ### `LoopHeader` debug-poll safepoint (DSL-0a transitional)
//!
//! In the transitional α body, `LoopHeader` triggers a debug-poll safepoint
//! (when the hook is installed) and a tier-backedge event + incremental-mark
//! poll. Per design §10 DSL-0c, tier accounting goes away when α is deleted —
//! but for DSL-0a, this body keeps the calls intact (they get removed in
//! Task C6). The debug-poll path mirrors the α body's
//! `sync_active_frame` → poll → `refresh_from_active_frame` ordering so a
//! debugger step that mutates the active frame leaves the next PC consistent
//! with the unchanged `pc_advance: instruction_len` advance below.
//!
//! ### Conditional jumps and caught abrupt completions
//!
//! `JumpIfTrue` / `JumpIfFalse` and their `*8` variants call
//! `to_boolean_agent`, which may throw. Routing through
//! `handle_dispatch_result` gives three cases:
//!  1. `Ok(Some(truthy))` — proceed with `truthy` as the predicate value.
//!  2. `Ok(None)` — the abrupt completion was caught by an active handler;
//!     `transfer_to_exception_handler` already rewrote the PC to the catch
//!     target. Return `Continue { pc_advance: 0 }` so the trampoline runs
//!     the new PC's opcode next (the epoch bump triggers a frame refresh on
//!     the next iteration, mirroring the unary-arithmetic semantic bodies).
//!  3. `Err(error)` — abrupt completion escapes; return `ExitError`.

use lyng_js_ops::read;
use lyng_js_types::Value;

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::error::VmError;
use crate::vm::Vm;
use crate::VmDebugSafepointKind;

// =====================================================================
// Jumps (unconditional) — `Jump`, `Jump8`.
// =====================================================================

/// Operands for an unconditional jump (Ax / Ax8 layout).
///
/// `delta` is the sign-extended relative offset (i24 for `Jump`,
/// i8 → i32 for `Jump8`). `instruction_len` is the encoded instruction
/// length the handler consumed during decode.
pub struct OpJumpArgs {
    pub delta: i32,
    pub instruction_len: u32,
}

/// Shared body for `Jump` / `Jump8`. On a backedge (`delta < 0`) the body
/// observes a tier-backedge event and polls the incremental-mark safepoint;
/// then it returns `Continue { pc_advance: instruction_len + delta }`.
///
/// The α handler computed `pc = current_pc + instruction_len + delta` via
/// `jump_dispatch_frame`. `translate_outcome_to_step` re-derives that same
/// absolute PC by calling `state.advance(pc_advance)` from the entry PC.
/// We preserve `jump_dispatch_frame`'s overflow check (returns
/// `VmError::InvalidJumpTarget` on under/overflow) so any backward jump
/// past the start of the bytecode buffer still surfaces the same diagnostic
/// the α path produced.
fn op_jump_shared_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpJumpArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    if args.delta < 0 {
        let code = inner.code();
        inner.vm.observe_tier_backedge_event(code);
        Vm::poll_incremental_mark_safepoint(inner.agent);
    }
    let instruction_offset = inner.frame.instruction_offset();
    let target = i64::from(instruction_offset)
        + i64::from(args.instruction_len)
        + i64::from(args.delta);
    if target < 0 || target > i64::from(u32::MAX) {
        return SemanticOutcome::ExitError {
            error: VmError::InvalidJumpTarget {
                code: inner.frame.code(),
                instruction_offset,
                target_offset: target,
            },
        };
    }
    // `state.advance(pc_advance)` does `pc.wrapping_add(pc_advance)`, so the
    // signed `instruction_len + delta` cast to u32 (via the `as` wrapping
    // cast on i64) reproduces the absolute target. The overflow check above
    // guarantees the absolute target fits in u32.
    let pc_advance = (i64::from(args.instruction_len) + i64::from(args.delta)) as u32;
    SemanticOutcome::Continue { pc_advance }
}

pub(crate) fn op_jump_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpJumpArgs,
) -> SemanticOutcome {
    op_jump_shared_semantic(state, args)
}

pub(crate) fn op_jump8_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpJumpArgs,
) -> SemanticOutcome {
    op_jump_shared_semantic(state, args)
}

// =====================================================================
// Conditional jumps — `JumpIfTrue`, `JumpIfTrue8`, `JumpIfFalse`,
// `JumpIfFalse8`.
// =====================================================================

/// Operands for a conditional jump (Abx / Abx8 layout).
///
/// `condition_register` (`a`) holds the value to test; `delta` is the
/// sign-extended relative offset (i32 for `JumpIf*`, i8 → i32 for the
/// `*8` variants).
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
        .read_register_unchecked(inner.frame.registers(), args.condition_register);
    let truthy_result =
        read::to_boolean_agent(inner.agent, condition).map_err(VmError::Abrupt);
    let truthy = match inner.handle_dispatch_result(truthy_result) {
        Ok(Some(t)) => t,
        Ok(None) => {
            // The abrupt completion was caught — handler PC was rewritten
            // by `transfer_to_exception_handler`. The next opcode lives at
            // the current PC, so resume dispatch with no advance.
            return SemanticOutcome::Continue { pc_advance: 0 };
        }
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let should_jump = if take_if_truthy { truthy } else { !truthy };
    if should_jump {
        if args.delta < 0 {
            Vm::poll_incremental_mark_safepoint(inner.agent);
        }
        let instruction_offset = inner.frame.instruction_offset();
        let target = i64::from(instruction_offset)
            + i64::from(args.instruction_len)
            + i64::from(args.delta);
        if target < 0 || target > i64::from(u32::MAX) {
            return SemanticOutcome::ExitError {
                error: VmError::InvalidJumpTarget {
                    code: inner.frame.code(),
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

pub(crate) fn op_jump_if_true_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpJumpIfArgs,
) -> SemanticOutcome {
    op_jump_if_shared_semantic(state, args, true)
}

pub(crate) fn op_jump_if_true8_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpJumpIfArgs,
) -> SemanticOutcome {
    op_jump_if_shared_semantic(state, args, true)
}

pub(crate) fn op_jump_if_false_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpJumpIfArgs,
) -> SemanticOutcome {
    op_jump_if_shared_semantic(state, args, false)
}

pub(crate) fn op_jump_if_false8_semantic(
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

pub(crate) fn op_loop_header_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpLoopHeaderArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    // Capture the entry-frame code before any debug-poll refresh; the
    // tier-backedge event is attributed to the code the loop-header lives
    // in, even if a debugger step relocated the active frame mid-handler.
    let code = inner.code();
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
    inner.vm.observe_tier_backedge_event(code);
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

/// Shared epilogue for `Return` / `ReturnUndefined`. Routes
/// `Vm::finish_frame` outcomes through `SemanticOutcome`:
///  - `Ok(Some(result))` → `ExitDone` (the entry frame returned).
///  - `Ok(None)` → `Refresh` (a nested return; caller frame is now active).
///  - `Err(error)` → `ExitError`.
fn op_return_finish_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    value: Value,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    inner.sync_active_frame();
    inner.pop_execution_context();
    match inner.finish_active_frame(value) {
        Ok(Some(result)) => SemanticOutcome::ExitDone { value: result },
        Ok(None) => SemanticOutcome::Refresh,
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

pub(crate) fn op_return_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpReturnArgs,
) -> SemanticOutcome {
    let value = {
        let inner = state.dispatch_state();
        inner
            .vm
            .read_register_unchecked(inner.frame.registers(), args.register)
    };
    op_return_finish_semantic(state, value)
}

pub(crate) fn op_return_undefined_semantic(
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

pub(crate) fn op_nop_semantic(
    _state: &mut LlIntDispatchState<'_, '_>,
    args: OpNopArgs,
) -> SemanticOutcome {
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

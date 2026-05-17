//! Calls family semantic bodies (DSL-0a Task A14).
//!
//! Each `op_xxx_semantic` function implements the semantic effect of one
//! calls-family opcode. The α handler in `dispatch_handlers/calls.rs`
//! decodes operands, constructs `OpXxxArgs`, calls the semantic body, and
//! translates the returned `SemanticOutcome` to `Step` via
//! `translate_outcome_to_step`. The DSL-0b cold-stub shim in
//! `dsl/handlers/cold/calls.rs` will reach the same functions from the
//! asm-DSL path.
//!
//! Family coverage (8 opcodes; `CallMethod` is not in scope per the
//! `dispatch_handlers/mod.rs` re-export list and remains an
//! `op_unimplemented` stub):
//! - Fixed-arity calls: `Call0`, `Call1`, `Call2`, `Call3` (Abc layout via
//!   `decode_abc_operands`; route through `Vm::call_value_small`).
//! - Variable-arity calls: `Call`, `Construct` (call-range layout via
//!   `decode_call_range_operands`; route through `Vm::call_value` and
//!   `Vm::construct_value` respectively).
//! - Tail call: `TailCall` (call-range layout; routes through
//!   `Vm::tail_call_value`; same-depth or unwind).
//! - Closure allocation: `CreateClosure` (Abx layout, no frame transition).
//!
//! ### Frame-transitioning semantics
//!
//! Successful `Call*` / `Construct` push a callee frame on the VM frame
//! stack inside the helper (`Vm::call_value*` / `Vm::construct_value`).
//! The semantic body returns `SemanticOutcome::Refresh` so
//! `translate_outcome_to_step` invokes `state.refresh_from_active_frame()`
//! to reload PC/REGS/FV for the callee frame before dispatching the next
//! opcode. This mirrors the α handler's explicit
//! `state.refresh_from_active_frame()` + `dispatch_next!` tail.
//!
//! `TailCall` replaces the active frame (same frame-stack depth) when the
//! callee is a bytecode body, or returns to the caller when the entry
//! frame unwinds. We route the three outcomes:
//! - `Ok(Some(Some(value)))` — entry frame unwound → `ExitDone`.
//! - `Ok(Some(None))` — frame replaced or returned → `Refresh`.
//! - `Ok(None)` — abrupt completion was caught by an active handler →
//!   `Refresh`.
//! - `Err(error)` — abrupt completion escapes → `ExitError`.
//!
//! On the caught-abrupt path, the α handler returns `dispatch_next!(state)`
//! without an explicit refresh; the trampoline's epoch + `still_active`
//! check refreshes lazily on the next loop iteration. Returning `Refresh`
//! here refreshes synchronously — observationally equivalent because both
//! reach the same post-refresh PC before the next opcode dispatches.
//!
//! `CreateClosure` allocates a function object and writes it to a
//! register. No frame transition; returns
//! `Continue { pc_advance: instruction_len }`.
//!
//! ### Feedback-slot recording (TailCall)
//!
//! The α handler for `TailCall` calls `vm.record_feedback_slot` *after*
//! `handle_dispatch_result` returns `Some(_)` (success) and *not* on the
//! caught-abrupt path. The semantic body preserves this exact ordering:
//! on `Ok(Some(_))` we record the slot before deciding `ExitDone` vs
//! `Refresh`; on `Ok(None)` we skip recording.

use lyng_js_bytecode::{CallRange, Opcode};
use lyng_js_types::{FeedbackSlotId, Value};

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::error::VmError;
use crate::vm::dispatch_state::DispatchState;

// =====================================================================
// Shared operand shapes
// =====================================================================

/// Operands for the fixed-arity call opcodes (`Call0`, `Call1`, `Call2`,
/// `Call3`) decoded via `decode_abc_operands`. `a` is the result
/// register, `b` is the callee register, `c` is the call base register
/// (this + arguments live at `c`, `c+1`, …, `c+arity`). `arity` is the
/// per-opcode constant (0..=3). The α handler stamps `feedback_slot` and
/// `instruction_len` from the decoded prefix-aware operand layout.
pub struct OpCallSmallArgs {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub arity: u8,
    pub feedback_slot: Option<FeedbackSlotId>,
    pub instruction_len: u32,
}

/// Operands for the variable-arity call opcodes (`Call`, `Construct`)
/// decoded via `decode_call_range_operands`. `a` is the result register,
/// `b` is the callee register, `c` is the this register (ignored by
/// `Construct`, which uses `new.target = callee`). `range` is the inline
/// call-range descriptor (argument base + count); `spread_mask` is read
/// from the feedback descriptor and indicates which arguments are spread
/// expansions.
pub struct OpCallRangeArgs {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub range: CallRange,
    pub spread_mask: Option<u64>,
    pub feedback_slot: Option<FeedbackSlotId>,
    pub instruction_len: u32,
}

/// Operands for `TailCall`. Same shape as `OpCallRangeArgs` but without
/// `c` (TailCall uses register `b` as the this register directly per the
/// α handler) and without `instruction_len` (TailCall replaces or unwinds
/// the active frame, so no PC advance applies in the caller).
pub struct OpTailCallArgs {
    pub a: u16,
    pub b: u16,
    pub range: CallRange,
    pub spread_mask: Option<u64>,
    pub feedback_slot: Option<FeedbackSlotId>,
}

/// Operands for `CreateClosure` (Abx layout). `a` is the result register;
/// `bx` is the child-code index (constant-pool position of the inner
/// function descriptor). No feedback slot — closure allocation is not
/// IC-bearing.
pub struct OpCreateClosureArgs {
    pub a: u16,
    pub bx: u32,
    pub instruction_len: u32,
}

// =====================================================================
// Helpers — `MissingInlineCallRange` extraction and spread-mask lookup
// =====================================================================

/// Build the `MissingInlineCallRange` error consistent with the α
/// handler's `require_call_range` helper. The decoder returns
/// `Option<CallRange>`, but `Call` / `TailCall` / `Construct` require
/// the inline operand; missing it is a bytecode bug.
#[inline]
fn missing_inline_call_range_error(
    inner: &DispatchState<'_>,
    opcode: Opcode,
) -> VmError {
    VmError::MissingInlineCallRange {
        code: inner.frame.code(),
        instruction_offset: inner.frame.instruction_offset(),
        opcode,
    }
}

// =====================================================================
// Call0 / Call1 / Call2 / Call3 — fixed-arity calls via call_value_small
// =====================================================================

/// Shared body for `Call0..Call3`. The α handler resolves the `arity`
/// (0..=3) and feeds it in via `OpCallSmallArgs::arity`. Routes
/// `Vm::call_value_small` through `handle_dispatch_result`:
/// - `Ok(Some(()))` — callee returned (native) or callee bytecode frame
///   was pushed by the helper; either way `vm.frames().last()` is now
///   the active frame (the helper either filled the result register and
///   left the caller frame on top, or pushed the callee frame). The
///   helper bumped `dispatch_frame_check_epoch` in both cases.
/// - `Ok(None)` — abrupt completion was caught (handler PC was rewritten
///   by `transfer_to_exception_handler`).
/// - `Err(error)` — abrupt completion escapes.
///
/// Returns `Refresh` for the first two cases so
/// `translate_outcome_to_step` re-snapshots PC/REGS/FV from the active
/// frame before dispatching the next opcode.
#[inline]
fn op_call_small_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCallSmallArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let call_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.call_value_small(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.instruction_len,
            args.feedback_slot,
            args.a,
            args.b,
            args.c,
            args.arity,
        )
    };
    let handled = inner.handle_dispatch_result(call_result);
    match handled {
        Ok(_) => SemanticOutcome::Refresh,
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

pub(crate) fn op_call0_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCallSmallArgs,
) -> SemanticOutcome {
    op_call_small_semantic(state, args)
}

pub(crate) fn op_call1_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCallSmallArgs,
) -> SemanticOutcome {
    op_call_small_semantic(state, args)
}

pub(crate) fn op_call2_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCallSmallArgs,
) -> SemanticOutcome {
    op_call_small_semantic(state, args)
}

pub(crate) fn op_call3_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCallSmallArgs,
) -> SemanticOutcome {
    op_call_small_semantic(state, args)
}

// =====================================================================
// Call — variable-arity call via call_value
// =====================================================================

pub(crate) fn op_call_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCallRangeArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let call_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.call_value(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.instruction_len,
            args.feedback_slot,
            args.a,
            args.b,
            args.c,
            args.range,
            args.spread_mask,
        )
    };
    let handled = inner.handle_dispatch_result(call_result);
    match handled {
        Ok(_) => SemanticOutcome::Refresh,
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

// =====================================================================
// TailCall — variable-arity tail call via tail_call_value
// =====================================================================
//
// The helper returns `VmResult<Option<Value>>`:
//   - `Ok(Some(value))` — the entry frame just unwound (the tail call
//     was at the top of the script). `record_feedback_slot` runs, then
//     `ExitDone { value }` propagates to `Vm::run`.
//   - `Ok(None)` — the tail call installed a same-depth activation or
//     returned us to the caller. `record_feedback_slot` runs, then
//     `Refresh` so the trampoline reloads from the (new) active frame.
//   - `Err(VmError::Abrupt(_))` routed through `handle_dispatch_result`
//     yields:
//       * `Ok(None)` if the throw was caught — skip
//         `record_feedback_slot` (matches α) and `Refresh`.
//       * `Err(error)` if the throw escapes — `ExitError`.
//
// The α handler ordering — record feedback only after success, not on
// the caught-abrupt path — is preserved here.

pub(crate) fn op_tail_call_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpTailCallArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let tail_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.tail_call_value(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.feedback_slot,
            args.a,
            args.b,
            args.range,
            args.spread_mask,
        )
    };
    let handled = inner.handle_dispatch_result(tail_result);
    let inner_result = match handled {
        Ok(Some(inner_result)) => inner_result,
        Ok(None) => return SemanticOutcome::Refresh,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    inner.vm.record_feedback_slot(code, args.feedback_slot);
    match inner_result {
        Some(value) => SemanticOutcome::ExitDone { value },
        None => SemanticOutcome::Refresh,
    }
}

// =====================================================================
// Construct — variable-arity construct via construct_value
// =====================================================================

pub(crate) fn op_construct_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCallRangeArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let construct_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.construct_value(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.instruction_len,
            args.feedback_slot,
            args.a,
            args.b,
            args.range,
            args.spread_mask,
        )
    };
    let handled = inner.handle_dispatch_result(construct_result);
    match handled {
        Ok(_) => SemanticOutcome::Refresh,
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

// =====================================================================
// CreateClosure — allocates a function object; no frame transition.
// =====================================================================

pub(crate) fn op_create_closure_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCreateClosureArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let closure_result = {
        let DispatchState {
            vm, agent, frame, ..
        } = &mut *inner;
        vm.create_closure(agent, frame, args.bx)
    };
    let handled = inner.handle_dispatch_result(closure_result);
    let closure = match handled {
        Ok(Some(obj)) => obj,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.frame.registers();
    inner
        .vm
        .write_register_unchecked(registers, args.a, Value::from_object_ref(closure));
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// Decode-time helpers used by the α handler. Kept here so the
// `missing_inline_call_range_error` lookup stays adjacent to the
// `OpCallRangeArgs` shape the α body builds.
// =====================================================================

/// α-handler helper: extract the inline `CallRange` from the
/// `decode_call_range_operands` output, returning a `VmError` if it's
/// missing (bytecode bug — the variable-arity call opcodes require an
/// inline call range).
#[inline]
pub(crate) fn require_call_range_semantic(
    state: &DispatchState<'_>,
    range: Option<CallRange>,
    opcode: Opcode,
) -> Result<CallRange, VmError> {
    range.ok_or_else(|| missing_inline_call_range_error(state, opcode))
}

/// α-handler helper: look up the spread-mask metadata for a feedback
/// slot. Returns `None` if no feedback slot is bound (i.e. no spread
/// arguments are possible) or if the feedback descriptor has no
/// `spread_mask` metadata. Matches the α handler's `spread_mask_for`
/// helper in `dispatch_handlers/calls.rs` so semantic and α paths
/// observe identical spread-bit interpretation.
#[inline]
pub(crate) fn spread_mask_for_semantic(
    state: &DispatchState<'_>,
    feedback_slot: Option<FeedbackSlotId>,
) -> Option<u64> {
    let slot = feedback_slot?;
    let descriptor = state.installed.feedback_descriptor_for_slot(slot)?;
    descriptor.metadata().spread_mask()
}

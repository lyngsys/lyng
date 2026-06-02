//! Calls family semantic bodies.
//!
//! Family coverage (8 opcodes; `CallMethod` remains an `op_unimplemented` stub):
//! - Fixed-arity calls: `Call0`, `Call1`, `Call2`, `Call3` — route through
//!   `Vm::call_value_small`.
//! - Variable-arity calls: `Call`, `Construct` — route through `Vm::call_value`
//!   and `Vm::construct_value` respectively.
//! - Tail call: `TailCall` — routes through `Vm::tail_call_value`; same-depth
//!   or unwind.
//! - Closure allocation: `CreateClosure` — no frame transition.
//!
//! Successful `Call*` / `Construct` push a callee frame; the semantic body
//! returns `SemanticOutcome::Refresh` so the dispatcher reloads PC/REGS/FV
//! for the callee frame. `TailCall` routes three outcomes:
//! `Ok(Some(Some(value)))` → `ExitDone`; `Ok(Some(None))` or `Ok(None)` →
//! `Refresh`; `Err(error)` → `ExitError`. Feedback slot is recorded on
//! `Ok(Some(_))` only, not on the caught-abrupt path.

use lyng_bytecode::CallRange;
use lyng_types::{FeedbackSlotId, Value};

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::vm::dispatch_state::DispatchState;

// =====================================================================
// Shared operand shapes
// =====================================================================

/// Operands for fixed-arity call opcodes (`Call0`–`Call3`). `a` is the result
/// register, `b` is the callee, `c` is the base (this + arguments at `c..c+arity`).
pub struct OpCallSmallArgs {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub arity: u8,
    pub feedback_slot: Option<FeedbackSlotId>,
    pub instruction_len: u32,
}

/// Operands for variable-arity call opcodes (`Call`, `Construct`). `a` is the
/// result register, `b` is the callee, `c` is the this register (unused by
/// `Construct`). `spread_mask` marks spread argument positions.
pub struct OpCallRangeArgs {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub range: CallRange,
    pub spread_mask: Option<u64>,
    pub feedback_slot: Option<FeedbackSlotId>,
    pub instruction_len: u32,
}

/// Operands for `TailCall`. Like `OpCallRangeArgs` but without `c` (register
/// `b` serves as `this`) and without `instruction_len` (no PC advance in caller).
pub struct OpTailCallArgs {
    pub a: u16,
    pub b: u16,
    pub range: CallRange,
    pub spread_mask: Option<u64>,
    pub feedback_slot: Option<FeedbackSlotId>,
}

/// Operands for `CreateClosure` (Abx layout). `a` is the result register;
/// `bx` is the constant-pool index of the inner function descriptor.
pub struct OpCreateClosureArgs {
    pub a: u16,
    pub bx: u32,
    pub instruction_len: u32,
}

// =====================================================================
// Call0 / Call1 / Call2 / Call3 — fixed-arity calls via call_value_small
// =====================================================================

/// Shared body for `Call0`–`Call3`. Routes `Vm::call_value_small` through
/// `handle_dispatch_result`; returns `Refresh` on success or caught abrupt
/// completion so the dispatcher reloads the active frame.
#[inline]
fn op_call_small_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCallSmallArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let call_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.call_value_small(
            agent,
            *host,
            &mut **registry,
            view,
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

pub fn op_call0_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCallSmallArgs,
) -> SemanticOutcome {
    op_call_small_semantic(state, args)
}

pub fn op_call1_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCallSmallArgs,
) -> SemanticOutcome {
    op_call_small_semantic(state, args)
}

pub fn op_call2_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCallSmallArgs,
) -> SemanticOutcome {
    op_call_small_semantic(state, args)
}

pub fn op_call3_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCallSmallArgs,
) -> SemanticOutcome {
    op_call_small_semantic(state, args)
}

// =====================================================================
// Call — variable-arity call via call_value
// =====================================================================

pub fn op_call_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCallRangeArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let call_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.call_value(
            agent,
            *host,
            &mut **registry,
            view,
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

pub fn op_tail_call_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpTailCallArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let view = inner.frame_view();
    let tail_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.tail_call_value(
            agent,
            *host,
            &mut **registry,
            view,
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
    inner_result.map_or(SemanticOutcome::Refresh, |value| {
        SemanticOutcome::ExitDone { value }
    })
}

// =====================================================================
// Construct — variable-arity construct via construct_value
// =====================================================================

pub fn op_construct_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCallRangeArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let construct_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.construct_value(
            agent,
            *host,
            &mut **registry,
            view,
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

pub fn op_create_closure_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCreateClosureArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    // `lexical_env` is mutated mid-frame by `with` push/pop; read live values from
    // the frame overlay so closures inside `with` blocks capture the current env.
    let cfr = inner.cfr;
    let code = inner.code();
    let lexical_env = inner.vm.frame_header(cfr).lexical_env();
    let private_env = inner.vm.frame_header(cfr).private_env();
    let closure_result = {
        let DispatchState { vm, agent, .. } = &mut *inner;
        let realm = vm.realm_of(agent, cfr);
        vm.create_closure(agent, code, lexical_env, private_env, realm, args.bx)
    };
    let handled = inner.handle_dispatch_result(closure_result);
    let closure = match handled {
        Ok(Some(obj)) => obj,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.registers();
    inner
        .vm
        .write_register_unchecked(registers, args.a, Value::from_object_ref(closure));
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

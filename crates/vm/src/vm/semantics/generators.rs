//! Generators / async family semantic bodies.
//!
//! Family coverage (6 opcodes):
//! - Suspension opcodes (`Ax`): `SuspendGeneratorStart`, `Yield`, `Await`.
//! - Yield-from delegation (`Abc`): `DelegateYield`.
//! - Resume-state loads (`Ax`): `LoadResumeKind`, `LoadResumeValue`.
//!
//! Suspension helpers return `Err(GeneratorStart/GeneratorYield/AsyncSuspend)`,
//! which escapes `Vm::run` as an `ExitError`; the caller treats it as a
//! suspension signal, not an abrupt completion.
//!
//! `Await` performs its own catch routing internally, so the semantic body
//! does NOT route through `handle_dispatch_result`; it reads the finalized PC
//! from the frame overlay after the call.
//!
//! `DelegateYield` routes through `handle_dispatch_result`; both the success
//! and caught-throw arms read the finalized PC from the overlay and return
//! `Continue { pc_advance: 0 }`.
//!
//! `Yield` / `SuspendGeneratorStart` always return `Err`; the `Ok` arm is
//! unreachable but kept for shape uniformity.
//!
//! `LoadResumeKind` / `LoadResumeValue` read resume state into a register
//! and return `Continue { pc_advance: instruction_len }`.

use lyng_types::Value;

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::vm::dispatch_state::DispatchState;

// =====================================================================
// Shared operand shapes
// =====================================================================

/// Operands for Ax-encoded generators opcodes. `register` is the value /
/// destination register (unused by `SuspendGeneratorStart`);
/// `instruction_len` is the post-instruction PC delta.
pub struct OpGeneratorsAxArgs {
    pub register: u16,
    pub instruction_len: u32,
}

/// Operands for `SuspendGeneratorStart`. No register operand needed.
pub struct OpSuspendGeneratorStartArgs {
    pub instruction_len: u32,
}

/// Operands for `DelegateYield`. `a` is the iterator side-table slot,
/// `b` is the result-value register, `c` is the done-flag register.
pub struct OpDelegateYieldArgs {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub instruction_len: u32,
}

// =====================================================================
// SuspendGeneratorStart — first opcode in a generator body; always suspends.
// =====================================================================

pub fn op_suspend_generator_start_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpSuspendGeneratorStartArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let resume_offset = inner.pc().wrapping_add(args.instruction_len);
    let view = inner.frame_view();
    let result = {
        let DispatchState { vm, agent, .. } = &mut *inner;
        vm.suspend_generator_start(agent, view, resume_offset)
    };
    match result {
        // Unreachable: `suspend_generator_start` always returns Err.
        Ok(()) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

// =====================================================================
// Yield — reads the yielded value, snapshots the frame, propagates
// `GeneratorYield { value, … }` to the caller of `Vm::run`.
// =====================================================================

pub fn op_yield_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpGeneratorsAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner
        .vm
        .read_register_unchecked(inner.registers(), args.register);
    let resume_offset = inner.pc().wrapping_add(args.instruction_len);
    let view = inner.frame_view();
    let result = {
        let DispatchState { vm, agent, .. } = &mut *inner;
        vm.suspend_current_generator_frame(agent, view, value, resume_offset, false)
    };
    match result {
        // Unreachable: `suspend_current_generator_frame` always returns Err.
        Ok(()) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

// =====================================================================
// DelegateYield — `yield*`; drives one step of inner-iterator delegation.
// =====================================================================

pub fn op_delegate_yield_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpDelegateYieldArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame_depth,
            ..
        } = &mut *inner;
        vm.delegate_yield(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            view,
            args.instruction_len,
            args.a,
            args.b,
            args.c,
        )
    };
    let handled = inner.handle_dispatch_result(result);
    match handled {
        // Both Ok arms park the finalized PC in the overlay `saved_pc`; sync it
        // into the thin view and resume with no advance. `yield*`'s caught throw
        // is always same-frame, so `cfr` is valid and unchanged.
        Ok(_) => {
            inner.pc = inner.vm.frame_header(inner.cfr).saved_pc();
            SemanticOutcome::Continue { pc_advance: 0 }
        }
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

// =====================================================================
// Await — either resumes (Ok path: frame already at next PC or handler
// PC) or suspends (Err). `Vm::await_value` does its own catch routing,
// so the body does NOT use `handle_dispatch_result`.
// =====================================================================

pub fn op_await_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpGeneratorsAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame_depth,
            ..
        } = &mut *inner;
        vm.await_value(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            view,
            args.instruction_len,
            args.register,
        )
    };
    match result {
        // `await_value` parks the finalized PC in the overlay `saved_pc`;
        // sync it into the thin view. Caught resume-throw is always same-frame.
        Ok(()) => {
            inner.pc = inner.vm.frame_header(inner.cfr).saved_pc();
            SemanticOutcome::Continue { pc_advance: 0 }
        }
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

// =====================================================================
// LoadResumeKind — writes `frame.resume_kind()` as SMI to `register`.
// =====================================================================

pub fn op_load_resume_kind_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpGeneratorsAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let kind = inner.resume_kind().raw();
    let registers = inner.registers();
    inner
        .vm
        .write_register_unchecked(registers, args.register, Value::from_smi(i32::from(kind)));
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// LoadResumeValue — writes `frame.resume_value()` to `register` and
// clears the resume slot.
// =====================================================================

pub fn op_load_resume_value_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpGeneratorsAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner.resume_value();
    let registers = inner.registers();
    inner
        .vm
        .write_register_unchecked(registers, args.register, value);
    inner.clear_resume();
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

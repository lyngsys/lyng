//! Generators / async family semantic bodies (DSL-0a Task A16).
//!
//! Each `op_xxx_semantic` function implements the semantic effect of one
//! generators-family opcode. The α handler in
//! `dispatch_handlers/generators.rs` decodes operands, constructs
//! `OpGeneratorsXxxArgs`, calls the semantic body, and translates the
//! returned `SemanticOutcome` to `Step` via `translate_outcome_to_step`.
//! The DSL-0b cold-stub shim in `dsl/handlers/cold/generators.rs` will
//! reach the same functions from the asm-DSL path.
//!
//! Family coverage (6 opcodes):
//! - Suspension opcodes (`Ax`):
//!   `SuspendGeneratorStart`, `Yield`, `Await`.
//! - Yield-from delegation (`Abc`): `DelegateYield`.
//! - Resume-state loads (`Ax`):
//!   `LoadResumeKind`, `LoadResumeValue`.
//!
//! ### Suspension semantics
//!
//! Generator / async suspension is signaled by the helpers in
//! `crate::vm::generators` (`suspend_generator_start`,
//! `suspend_current_generator_frame`) and `crate::vm::async_functions`
//! (`await_value` → `suspend_for_await_promise`) returning
//! `Err(VmError::GeneratorStart { suspended })`,
//! `Err(VmError::GeneratorYield { value, suspended, raw_iterator_result })`,
//! or `Err(VmError::AsyncSuspend)` respectively. The error escapes
//! `Vm::run`; the caller of `Vm::run` (the resume-orchestration logic in
//! `crate::vm::generators` / `crate::vm::async_functions`) catches the
//! special error variant and treats it as a suspension signal rather than
//! a true abrupt completion.
//!
//! `SemanticOutcome::ExitError { error }` is the right mapping: the
//! semantic body returns the error verbatim, the α path translates to
//! `Step::Error(error)`, and the trampoline returns it to `Vm::run`'s
//! caller — exactly the contract the existing α handler implements via
//! `try_step!`.
//!
//! ### Resume-throw routing in `Await`
//!
//! `Vm::await_value` does its *own* catch routing on the resume-throw
//! path (it inspects `frame.resume_kind()`, runs
//! `transfer_to_exception_handler`, and refreshes the dispatch frame
//! before returning `Ok(())`). The α handler does NOT call
//! `handle_dispatch_result` on the result; it `try_step!`s directly. The
//! semantic body preserves this exactly: `Ok(())` → `Continue { pc_advance: 0 }`
//! (the frame's instruction offset was set by either
//! `advance_dispatch_frame` on the success path or
//! `refresh_dispatch_frame` on the caught-throw path), and `Err(error)` →
//! `ExitError { error }`.
//!
//! ### `DelegateYield` catch routing
//!
//! `Vm::delegate_yield` may throw an `Abrupt(Throw)` from the inner
//! iterator's `next` / `throw` / `return` method, which the surrounding
//! generator body may catch. The α handler routes through
//! `handle_dispatch_result`, which transfers to the exception handler and
//! returns `Ok(None)` when caught. The semantic body preserves this:
//! `Ok(_)` and caught-throw (`Ok(None)`) → `Continue { pc_advance: 0 }`
//! (helper advanced the frame on success; `refresh_dispatch_frame`
//! advanced it on caught-throw), and `Err(error)` → `ExitError { error }`
//! for uncaught throws and suspension signals
//! (`GeneratorYield`, `AsyncSuspend`).
//!
//! ### `Yield` / `SuspendGeneratorStart` always suspend
//!
//! `Vm::suspend_current_generator_frame` and
//! `Vm::suspend_generator_start` always return `Err` (the suspension
//! signal). The α handler unreachably tails with `dispatch_next!`; the
//! semantic body mirrors that shape with an unreachable
//! `Continue { pc_advance: 0 }` arm — both forms compile, and the Err arm
//! is the only one observed at runtime.
//!
//! ### Resume-state loads
//!
//! `LoadResumeKind` and `LoadResumeValue` are not suspension opcodes —
//! they read state populated by the resume entry path
//! (`Vm::restore_suspended_execution`) and write it into a register.
//! `LoadResumeValue` additionally calls `frame.clear_resume()` to
//! consume the slot. Both return
//! `Continue { pc_advance: args.instruction_len }`.

use lyng_types::Value;

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::vm::dispatch::next_dispatch_instruction_offset;
use crate::vm::dispatch_state::DispatchState;

// =====================================================================
// Shared operand shapes
// =====================================================================

/// Operands for the Ax-encoded generators opcodes: `SuspendGeneratorStart`,
/// `Yield`, `Await`, `LoadResumeKind`, `LoadResumeValue`.
///
/// - `SuspendGeneratorStart`: `register` is unused (the α handler ignores
///   `ax`); `instruction_len` is the post-instruction PC delta used to
///   compute the resume offset.
/// - `Yield` / `Await`: `register` is the operand register
///   (`Yield`: the value to yield; `Await`: the value-or-result slot
///   used for both read on the suspend path and write on the resume
///   path); `instruction_len` is the post-instruction PC delta.
/// - `LoadResumeKind` / `LoadResumeValue`: `register` is the destination
///   register; `instruction_len` is the PC advance.
pub struct OpGeneratorsAxArgs {
    pub register: u16,
    pub instruction_len: u32,
}

/// Operands for `SuspendGeneratorStart`. The α handler decodes `ax` but
/// never consults it (the resume bytecode reads back from
/// `frame.resume_*()`, not from a register), so we omit it from the args
/// shape.
pub struct OpSuspendGeneratorStartArgs {
    pub instruction_len: u32,
}

/// Operands for `DelegateYield` (Abc layout). `a` is the iterator-side-
/// table slot register, `b` is the result-value register the yielded
/// `{ value, done }` is written to, `c` is the done-flag register.
/// `instruction_len` is the post-instruction PC delta.
pub struct OpDelegateYieldArgs {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub instruction_len: u32,
}

// =====================================================================
// SuspendGeneratorStart — Ax; always suspends with `GeneratorStart`.
//
// First opcode in a generator body. Computes the resume PC (one past
// this instruction), snapshots the frame via `suspend_generator_start`,
// and propagates the resulting `GeneratorStart { suspended }` error to
// the caller of `Vm::run`. The caller installs the suspended-execution
// record in the generator object so subsequent `.next()` calls can
// resume at the post-`SuspendGeneratorStart` PC.
// =====================================================================

pub fn op_suspend_generator_start_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpSuspendGeneratorStartArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let resume_offset = next_dispatch_instruction_offset(&inner.frame, args.instruction_len);
    inner.sync_active_frame();
    let result = {
        let DispatchState {
            vm, agent, frame, ..
        } = &mut *inner;
        vm.suspend_generator_start(agent, frame, resume_offset)
    };
    match result {
        // Unreachable today — `suspend_generator_start` always returns
        // `Err(GeneratorStart { suspended })`. Mirror the α handler's
        // `dispatch_next!` tail for shape preservation.
        Ok(()) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

// =====================================================================
// Yield — Ax; reads the yielded value from `register`, snapshots the
// frame via `suspend_current_generator_frame`, propagates `GeneratorYield
// { value, suspended, raw_iterator_result }`. The caller of `Vm::run`
// surfaces the yielded value to the generator consumer.
// =====================================================================

pub fn op_yield_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpGeneratorsAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner
        .vm
        .read_register_unchecked(inner.frame.registers(), args.register);
    let resume_offset = next_dispatch_instruction_offset(&inner.frame, args.instruction_len);
    inner.sync_active_frame();
    let result = {
        let DispatchState {
            vm, agent, frame, ..
        } = &mut *inner;
        vm.suspend_current_generator_frame(agent, frame, value, resume_offset, false)
    };
    match result {
        // Unreachable today — `suspend_current_generator_frame` always
        // returns `Err(GeneratorYield { … })`. Shape preserved.
        Ok(()) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

// =====================================================================
// DelegateYield — Abc; `yield*`. Drives one step of inner-iterator
// delegation (next / throw / return based on the active resume kind),
// then either suspends (Suspend outcome → `GeneratorYield`) or completes
// (Complete outcome → frame advanced internally, `Ok(())` returned).
//
// On caught throw inside the inner iterator's method,
// `handle_dispatch_result` rewrites PC to the catch target and returns
// `Ok(None)`; we then return `Continue { pc_advance: 0 }`.
// =====================================================================

pub fn op_delegate_yield_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpDelegateYieldArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.delegate_yield(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.instruction_len,
            args.a,
            args.b,
            args.c,
        )
    };
    let handled = inner.handle_dispatch_result(result);
    match handled {
        // Success path: helper advanced frame past the instruction.
        // Caught-throw path (`Ok(None)`): helper refreshed frame to the
        // catch target. Both: next opcode lives at `frame.instruction_offset()`.
        Ok(_) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

// =====================================================================
// Await — Ax; either resumes (Ok path: frame already advanced or
// rewritten to the catch handler) or suspends (`Err(AsyncSuspend)` /
// `Err(Abrupt(Throw))` for uncaught resume-throw).
//
// `Vm::await_value` performs its own catch routing on the resume-throw
// path, so the semantic body does NOT route through
// `handle_dispatch_result` (this mirrors the α handler's bare
// `try_step!(vm.await_value(...))`).
// =====================================================================

pub fn op_await_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpGeneratorsAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.await_value(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.instruction_len,
            args.register,
        )
    };
    match result {
        // Frame already at the next-instruction PC (post-resume) or the
        // catch handler PC (caught resume-throw). Either way, do not
        // re-advance.
        Ok(()) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

// =====================================================================
// LoadResumeKind — Ax; reads `frame.resume_kind()` and writes it as an
// SMI to `register`. No suspension. Does not clear the resume slot —
// that is the job of `LoadResumeValue`.
// =====================================================================

pub fn op_load_resume_kind_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpGeneratorsAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let kind = inner.frame.resume_kind().raw();
    let registers = inner.frame.registers();
    inner
        .vm
        .write_register_unchecked(registers, args.register, Value::from_smi(i32::from(kind)));
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// LoadResumeValue — Ax; reads `frame.resume_value()` and writes it to
// `register`, then clears the resume slot via `frame.clear_resume()`. No
// suspension.
// =====================================================================

pub fn op_load_resume_value_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpGeneratorsAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner.frame.resume_value();
    let registers = inner.frame.registers();
    inner
        .vm
        .write_register_unchecked(registers, args.register, value);
    inner.frame.clear_resume();
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

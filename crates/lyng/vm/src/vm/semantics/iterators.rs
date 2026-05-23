//! Iterators family semantic bodies (DSL-0a Task A15).
//!
//! Each `op_xxx_semantic` function implements the semantic effect of one
//! iterators-family opcode. The α handler in
//! `dispatch_handlers/iterators.rs` decodes operands, constructs
//! `OpXxxArgs`, calls the semantic body, and translates the returned
//! `SemanticOutcome` to `Step` via `translate_outcome_to_step`. The DSL-0b
//! cold-stub shim in `dsl/handlers/cold/iterators.rs` will reach the same
//! functions from the asm-DSL path.
//!
//! Family coverage (6 opcodes):
//! - For-in enumerator side table (Abc / Abc / Abx):
//!   `CreateForIn`, `AdvanceForIn`, `CloseForIn`.
//! - Generic iterator protocol side table (Abc / Abc / Abx):
//!   `CreateIterator`, `AdvanceIterator`, `CloseIterator`.
//!
//! The iterator-protocol helpers (enumerator construction, advance, close,
//! return-method invocation) live in `crate::vm::loop_iteration`; the
//! semantic bodies reach them through thin wrappers on `Vm`
//! (`create_for_in_enumerator_for_value`, `for_in_advance`, …,
//! `create_iterator_for_value`, `advance_iterator_state`,
//! `close_iterator_state`). Those helpers are untouched by A15 — the
//! semantic body merely owns the `handle_dispatch_result` routing and the
//! caught-vs-success PC-advance decision.
//!
//! ### Caught-completion PC handling
//!
//! When an iterator-protocol call yields an abrupt completion that is
//! caught by an active handler, the α handler returned `dispatch_next!`
//! *without* advancing past the iterators-family instruction — the catch
//! target's PC was installed by `transfer_to_exception_handler` (inside
//! `handle_dispatch_result`) and the next byte to dispatch is at that
//! handler PC. The semantic body preserves this exactly by returning
//! `Continue { pc_advance: 0 }` on the `Ok(None)` path; the `Continue`
//! variant means `translate_outcome_to_step` calls `state.advance(0)` and
//! reads the next opcode byte at the current (handler-target) PC.
//!
//! On the success path the body inserts/writes the iterator outputs first
//! and only then returns `Continue { pc_advance: args.instruction_len }`
//! — `translate_outcome_to_step` then advances PC past the instruction
//! before dispatching the next opcode.
//!
//! ### `AdvanceIterator` frame sync
//!
//! `AdvanceIterator` calls `state.sync_active_frame()` before invoking
//! `advance_iterator_state`, mirroring the α handler. The iterator's
//! `next` method runs via the bytecode trampoline (`Vm::call_value`),
//! which inspects the live frame stack — the sync writes the cached
//! per-frame fields back to the canonical top frame so the callee sees
//! the up-to-date instruction offset.

use lyng_js_types::Value;

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::vm::dispatch_state::DispatchState;

// =====================================================================
// Shared operand shapes
// =====================================================================

/// Operands for the Abc-encoded iterators opcodes: `CreateForIn`,
/// `AdvanceForIn`, `CreateIterator`, `AdvanceIterator`.
///
/// - `CreateForIn` / `CreateIterator`: `a` is the side-table slot
///   (enumerator / iterator register), `b` is the value-to-iterate
///   register, `c` is unused for `CreateForIn` and is the async-iterator
///   flag for `CreateIterator` (non-zero → async).
/// - `AdvanceForIn` / `AdvanceIterator`: `a` is the side-table slot,
///   `b` is the result-value register, `c` is the done-flag register.
pub struct OpIteratorAbcArgs {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub instruction_len: u32,
}

/// Operands for the Abx-encoded iterators opcodes: `CloseForIn`,
/// `CloseIterator`.
///
/// - `CloseForIn`: `a` is the side-table slot to drop; `bx` is unused.
/// - `CloseIterator`: `a` is the side-table slot; `bx != 0` indicates an
///   already-pending abrupt completion (the iterator's `return` method
///   may be invoked even though the iteration was aborted).
pub struct OpIteratorAbxArgs {
    pub a: u16,
    pub bx: u32,
    pub instruction_len: u32,
}

// =====================================================================
// CreateForIn — Abc; constructs a for-in enumerator and inserts it into
// the for-in side table at register `a`.
// =====================================================================

pub(crate) fn op_create_for_in_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpIteratorAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner
        .vm
        .read_register_unchecked(inner.frame.registers(), args.b);
    let enumerator_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *inner;
        vm.create_for_in_enumerator_for_value(agent, *host, &mut **registry, frame, value)
    };
    let handled = inner.handle_dispatch_result(enumerator_result);
    let enumerator = match handled {
        Ok(Some(e)) => e,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let base = inner.frame.registers().base();
    inner.vm.for_in_insert(base, args.a, enumerator);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// AdvanceForIn — Abc; advances the for-in enumerator at register `a`,
// writing the next property key (converted to its enumeration value) to
// `b` and the done flag to `c`.
// =====================================================================

pub(crate) fn op_advance_for_in_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpIteratorAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let base = inner.frame.registers().base();
    let next_result = {
        let DispatchState { vm, agent, .. } = &mut *inner;
        vm.for_in_advance(agent, base, args.a)
    };
    let handled = inner.handle_dispatch_result(next_result);
    let next = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let done = next.is_none();
    let value = match next {
        Some(key) => {
            let DispatchState { vm, agent, .. } = &mut *inner;
            vm.property_key_to_enumeration_value(agent, key)
        }
        None => Value::undefined(),
    };
    let registers = inner.frame.registers();
    inner.vm.write_register_unchecked(registers, args.b, value);
    inner
        .vm
        .write_register_unchecked(registers, args.c, Value::from_bool(done));
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// CloseForIn — Abx; drops the for-in enumerator at register `a`. The
// underlying `for_in_remove` is infallible (no `Result`), so there is no
// caught-completion case here.
// =====================================================================

pub(crate) fn op_close_for_in_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpIteratorAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let base = inner.frame.registers().base();
    inner.vm.for_in_remove(base, args.a);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// CreateIterator — Abc; constructs an iterator record (sync or async
// per `c != 0`) for the value in register `b` and inserts it into the
// iterator side table at register `a`.
// =====================================================================

pub(crate) fn op_create_iterator_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpIteratorAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner
        .vm
        .read_register_unchecked(inner.frame.registers(), args.b);
    let is_async = args.c != 0;
    let iterator_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *inner;
        vm.create_iterator_for_value(agent, *host, &mut **registry, frame, value, is_async)
    };
    let handled = inner.handle_dispatch_result(iterator_result);
    let iterator = match handled {
        Ok(Some(i)) => i,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let base = inner.frame.registers().base();
    inner.vm.iterator_insert(base, args.a, iterator);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// AdvanceIterator — Abc; calls the iterator's `next` method (which may
// run user bytecode), writes the produced value to `b` and the done flag
// to `c`. The α handler syncs the active frame before the call because
// the callee can read the live frame stack via `Vm::call_value`.
// =====================================================================

pub(crate) fn op_advance_iterator_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpIteratorAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    inner.sync_active_frame();
    let next_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.advance_iterator_state(agent, *host, &mut **registry, *frame_depth, frame, args.a)
    };
    let handled = inner.handle_dispatch_result(next_result);
    let next = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let done = next.is_none();
    let value = next.unwrap_or(Value::undefined());
    let registers = inner.frame.registers();
    inner.vm.write_register_unchecked(registers, args.b, value);
    inner
        .vm
        .write_register_unchecked(registers, args.c, Value::from_bool(done));
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// CloseIterator — Abx; invokes the iterator's `return` method per
// ECMA-262 IteratorClose. `bx != 0` signals that an abrupt completion is
// already pending (the iteration was aborted), which preserves the
// original completion when `return` itself completes abruptly. The
// underlying helper returns `VmResult<()>`; we route it through
// `handle_dispatch_result` to honor the catch-target rewrite.
// =====================================================================

pub(crate) fn op_close_iterator_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpIteratorAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let close_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.close_iterator_state(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.a,
            args.bx != 0,
        )
    };
    let handled = inner.handle_dispatch_result(close_result);
    match handled {
        Ok(Some(())) => SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        },
        Ok(None) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

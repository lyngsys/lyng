//! Iterators family semantic bodies.
//!
//! Family coverage (6 opcodes):
//! - For-in: `CreateForIn`, `AdvanceForIn`, `CloseForIn`.
//! - Generic iterator protocol: `CreateIterator`, `AdvanceIterator`,
//!   `CloseIterator`.
//!
//! On a caught abrupt completion, `handle_dispatch_result` rewrites PC to
//! the catch target and returns `Ok(None)`; the body returns
//! `Continue { pc_advance: 0 }`. On success the body writes outputs first,
//! then returns `Continue { pc_advance: args.instruction_len }`.
//!
//! `AdvanceIterator` calls `state.sync_active_frame()` before invoking
//! `advance_iterator_state` because the iterator's `next` method may run
//! user bytecode that inspects the live frame stack.

use lyng_types::Value;

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::vm::dispatch_state::DispatchState;

// =====================================================================
// Shared operand shapes
// =====================================================================

/// Operands for Abc-encoded iterators opcodes. For Create*: `a` is the
/// side-table slot, `b` is the value to iterate, `c` is the async flag
/// (non-zero → async, `CreateIterator` only). For Advance*: `a` is the
/// side-table slot, `b` is the result-value register, `c` is the done flag.
pub struct OpIteratorAbcArgs {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub instruction_len: u32,
}

/// Operands for Abx-encoded iterators opcodes. `a` is the side-table slot.
/// For `CloseIterator`, `bx != 0` signals an already-pending abrupt completion.
pub struct OpIteratorAbxArgs {
    pub a: u16,
    pub bx: u32,
    pub instruction_len: u32,
}

// =====================================================================
// CreateForIn
// =====================================================================

pub fn op_create_for_in_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpIteratorAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner.vm.read_register_unchecked(inner.registers(), args.b);
    let view = inner.frame_view();
    let enumerator_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.create_for_in_enumerator_for_value(agent, *host, &mut **registry, view, value)
    };
    let handled = inner.handle_dispatch_result(enumerator_result);
    let enumerator = match handled {
        Ok(Some(e)) => e,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let base = inner.registers().base();
    inner.vm.for_in_insert(base, args.a, enumerator);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// AdvanceForIn
// =====================================================================

pub fn op_advance_for_in_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpIteratorAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let base = inner.registers().base();
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
    let value = next.map_or_else(Value::undefined, |key| {
        let DispatchState { vm, agent, .. } = &mut *inner;
        vm.property_key_to_enumeration_value(agent, key)
    });
    let registers = inner.registers();
    inner.vm.write_register_unchecked(registers, args.b, value);
    inner
        .vm
        .write_register_unchecked(registers, args.c, Value::from_bool(done));
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// CloseForIn
// =====================================================================

pub fn op_close_for_in_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpIteratorAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let base = inner.registers().base();
    inner.vm.for_in_remove(base, args.a);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// CreateIterator
// =====================================================================

pub fn op_create_iterator_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpIteratorAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner.vm.read_register_unchecked(inner.registers(), args.b);
    let is_async = args.c != 0;
    let view = inner.frame_view();
    let iterator_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.create_iterator_for_value(agent, *host, &mut **registry, view, value, is_async)
    };
    let handled = inner.handle_dispatch_result(iterator_result);
    let iterator = match handled {
        Ok(Some(i)) => i,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let base = inner.registers().base();
    inner.vm.iterator_insert(base, args.a, iterator);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// AdvanceIterator
// =====================================================================

pub fn op_advance_iterator_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpIteratorAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    inner.sync_active_frame();
    let view = inner.frame_view();
    let next_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame_depth,
            ..
        } = &mut *inner;
        vm.advance_iterator_state(agent, *host, &mut **registry, *frame_depth, view, args.a)
    };
    let handled = inner.handle_dispatch_result(next_result);
    let next = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let done = next.is_none();
    let value = next.unwrap_or(Value::undefined());
    let registers = inner.registers();
    inner.vm.write_register_unchecked(registers, args.b, value);
    inner
        .vm
        .write_register_unchecked(registers, args.c, Value::from_bool(done));
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// CloseIterator — invokes the iterator's `return` method (IteratorClose).
// =====================================================================

pub fn op_close_iterator_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpIteratorAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let close_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame_depth,
            ..
        } = &mut *inner;
        vm.close_iterator_state(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            view,
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

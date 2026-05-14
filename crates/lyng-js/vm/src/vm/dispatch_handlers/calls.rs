//! Call-family handlers for the trampoline dispatch path (lyng-1fie).
//!
//! Frame transitions (push callee, sync caller, refresh after return) are
//! handled inside the `Vm::call_value*` / `tail_call_value` / `construct_value`
//! helpers — the trampoline handler just decodes operands, threads the
//! split-borrowed pieces of `DispatchState` through, and routes the result
//! via `DispatchState::handle_dispatch_result` so a thrown exception either
//! transfers to the caller's handler (the helper rewrote pc) or escapes via
//! `Step::Error`.
//!
//! `TailCall` is special: the helper returns
//! `VmResult<Option<Value>>` where `Some(value)` means the script's entry
//! frame just unwound and we should emit `Step::Done`. Other handlers in this
//! file just `dispatch_next!` after the helper returns.
//!
//! `CreateClosure` doesn't transfer frames; it just allocates a function
//! object and writes it to a register.

use lyng_js_bytecode::Opcode;
use lyng_js_types::Value;

use crate::error::VmError;
use crate::vm::dispatch::{decode_abc_operands, decode_abx_operands, decode_call_range_operands};
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::{dispatch_next, try_step};

// =====================================================================
// Call0..3 — fixed-arity calls via call_value_small
// =====================================================================

#[inline]
fn op_call_small_common(state: &mut DispatchState, arity: u8) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let prefix = state.prefix.take();
    let (a, b, c, feedback_slot, instruction_len) = try_step!(decode_abc_operands(
        state.current_bytes(),
        prefix,
        true,
        code,
        pc,
    ));
    let call_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *state;
        vm.call_value_small(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            instruction_len,
            feedback_slot,
            a,
            b,
            c,
            arity,
        )
    };
    if try_step!(state.handle_dispatch_result(call_result)).is_none() {
        // Exception transferred — handler rewrote pc.
        dispatch_next!(state);
    }
    // The call helper may have pushed a callee frame onto the VM frame
    // stack (JS / bytecode callees). The trampoline owns a snapshot of
    // the active frame in state.frame and state.installed; resync them
    // before dispatching the next opcode.
    try_step!(state.refresh_from_active_frame());
    dispatch_next!(state);
}

pub extern "C" fn op_call0(state: &mut DispatchState) -> Step {
    op_call_small_common(state, 0)
}

pub extern "C" fn op_call1(state: &mut DispatchState) -> Step {
    op_call_small_common(state, 1)
}

pub extern "C" fn op_call2(state: &mut DispatchState) -> Step {
    op_call_small_common(state, 2)
}

pub extern "C" fn op_call3(state: &mut DispatchState) -> Step {
    op_call_small_common(state, 3)
}

// =====================================================================
// Call / TailCall / Construct — variable-arity via decode_call_range_operands
// =====================================================================

#[inline]
fn require_call_range(
    state: &DispatchState,
    range: Option<lyng_js_bytecode::CallRange>,
    semantic: Opcode,
) -> Result<lyng_js_bytecode::CallRange, Step> {
    range.ok_or_else(|| {
        Step::Error(VmError::MissingInlineCallRange {
            code: state.frame.code(),
            instruction_offset: state.frame.instruction_offset(),
            opcode: semantic,
        })
    })
}

#[inline]
fn spread_mask_for(
    state: &DispatchState,
    feedback_slot: Option<lyng_js_types::FeedbackSlotId>,
) -> Option<u64> {
    let slot = feedback_slot?;
    let descriptor = state.installed.feedback_descriptor_for_slot(slot)?;
    descriptor.metadata().spread_mask()
}

pub extern "C" fn op_call(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (a, b, c, call_range, feedback_slot, instruction_len) = try_step!(
        decode_call_range_operands(state.current_bytes(), true, code, pc,)
    );
    let range = match require_call_range(state, call_range, Opcode::Call) {
        Ok(r) => r,
        Err(step) => return step,
    };
    let spread_mask = spread_mask_for(state, feedback_slot);
    let call_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *state;
        vm.call_value(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            instruction_len,
            feedback_slot,
            a,
            b,
            c,
            range,
            spread_mask,
        )
    };
    if try_step!(state.handle_dispatch_result(call_result)).is_none() {
        dispatch_next!(state);
    }
    try_step!(state.refresh_from_active_frame());
    dispatch_next!(state);
}

pub extern "C" fn op_tail_call(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (a, b, _c, call_range, feedback_slot, _instruction_len) = try_step!(
        decode_call_range_operands(state.current_bytes(), true, code, pc,)
    );
    let range = match require_call_range(state, call_range, Opcode::TailCall) {
        Ok(r) => r,
        Err(step) => return step,
    };
    let spread_mask = spread_mask_for(state, feedback_slot);
    let tail_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *state;
        vm.tail_call_value(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            feedback_slot,
            a,
            b,
            range,
            spread_mask,
        )
    };
    let inner = match try_step!(state.handle_dispatch_result(tail_result)) {
        Some(inner) => inner,
        None => dispatch_next!(state),
    };
    state.vm.record_feedback_slot(code, feedback_slot);
    if let Some(result) = inner {
        return Step::Done(result);
    }
    // Caller frame is now active (tail call installed a same-depth
    // activation, or returned us to our caller); refresh and continue.
    try_step!(state.refresh_from_active_frame());
    dispatch_next!(state);
}

pub extern "C" fn op_construct(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (a, b, _c, call_range, feedback_slot, instruction_len) = try_step!(
        decode_call_range_operands(state.current_bytes(), true, code, pc,)
    );
    let range = match require_call_range(state, call_range, Opcode::Construct) {
        Ok(r) => r,
        Err(step) => return step,
    };
    let spread_mask = spread_mask_for(state, feedback_slot);
    let construct_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *state;
        vm.construct_value(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            instruction_len,
            feedback_slot,
            a,
            b,
            range,
            spread_mask,
        )
    };
    if try_step!(state.handle_dispatch_result(construct_result)).is_none() {
        dispatch_next!(state);
    }
    try_step!(state.refresh_from_active_frame());
    dispatch_next!(state);
}

// =====================================================================
// CreateClosure — allocates a function object; no frame transition.
// =====================================================================

pub extern "C" fn op_create_closure(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let prefix = state.prefix.take();
    let (a, bx, _feedback_slot, instruction_len) = try_step!(decode_abx_operands(
        state.current_bytes(),
        prefix,
        false,
        code,
        pc,
    ));
    let closure_result = {
        let DispatchState {
            vm, agent, frame, ..
        } = &mut *state;
        vm.create_closure(agent, frame, bx)
    };
    let closure = match try_step!(state.handle_dispatch_result(closure_result)) {
        Some(obj) => obj,
        None => dispatch_next!(state),
    };
    let registers = state.frame.registers();
    state
        .vm
        .write_register_unchecked(registers, a, Value::from_object_ref(closure));
    state.advance(instruction_len);
    dispatch_next!(state);
}

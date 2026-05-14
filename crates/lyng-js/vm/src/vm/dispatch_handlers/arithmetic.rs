//! Arithmetic family handlers for the trampoline dispatch path (lyng-54em).
//!
//! Each binary SMI-fast-path handler:
//!
//! 1. Decodes its Abc operands via `decode_abc_operands` (with `state.prefix`
//!    consumed via `.take()`). `is_profiled` is hard-coded `true` — every
//!    arithmetic opcode covered here has `has_feedback_slot() == true`.
//! 2. Reads two register operands through `Vm::read_register_unchecked`.
//! 3. Tries the SMI fast path (`as_smi` + checked op); on success records
//!    feedback and tail-dispatches.
//! 4. Falls through to a `#[cold] #[inline(never)]` slow helper that calls
//!    the existing `Vm::execute_*_opcode` family + `finish_abc_value_result`.
//!
//! `*Smi` variants (`AddSmi`, `SubSmi`, `MulSmi`) decode operand `c` as an
//! `i16`-encoded immediate via `decode_smi_immediate` rather than as a
//! second register.
//!
//! Slow paths handle ToPrimitive coercion, BigInt, f64, and exception
//! transfer. They aren't size-budgeted — they live in a cold text region
//! and are taken only when the SMI fast path can't proceed.

use lyng_js_types::{FeedbackSlotId, Value};

use crate::vm::dispatch::arithmetic::{decode_smi_immediate, smi_mul_result};
use crate::vm::dispatch::decode_abc_operands;
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::{dispatch_next, try_step};

// =====================================================================
// Add / Sub / Mul — two-register Abc with feedback slot
// =====================================================================

pub extern "C" fn op_add(state: &mut DispatchState) -> Step {
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

    let registers = state.frame.registers();
    let left = state.vm.read_register_unchecked(registers, b);
    let right = state.vm.read_register_unchecked(registers, c);
    if let (Some(l), Some(r)) = (left.as_smi(), right.as_smi())
        && let Some(v) = l.checked_add(r)
    {
        state.vm.record_feedback_slot(code, feedback_slot);
        state
            .vm
            .write_register_unchecked(registers, a, Value::from_smi(v));
        state.advance(instruction_len);
        dispatch_next!(state);
    }
    op_add_slow(state, a, b, c, feedback_slot, instruction_len)
}

#[cold]
#[inline(never)]
fn op_add_slow(
    state: &mut DispatchState,
    a: u16,
    b: u16,
    c: u16,
    feedback_slot: Option<FeedbackSlotId>,
    instruction_len: u32,
) -> Step {
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.execute_add_opcode(agent, *host, &mut **registry, frame, b, c)
    };
    let finish = {
        let DispatchState {
            vm,
            agent,
            frame,
            frame_depth,
            ..
        } = &mut *state;
        vm.finish_abc_value_result(
            agent,
            *frame_depth,
            frame,
            instruction_len,
            feedback_slot,
            a,
            result,
        )
    };
    try_step!(finish);
    dispatch_next!(state);
}

pub extern "C" fn op_sub(state: &mut DispatchState) -> Step {
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

    let registers = state.frame.registers();
    let left = state.vm.read_register_unchecked(registers, b);
    let right = state.vm.read_register_unchecked(registers, c);
    if let (Some(l), Some(r)) = (left.as_smi(), right.as_smi())
        && let Some(v) = l.checked_sub(r)
    {
        state.vm.record_feedback_slot(code, feedback_slot);
        state
            .vm
            .write_register_unchecked(registers, a, Value::from_smi(v));
        state.advance(instruction_len);
        dispatch_next!(state);
    }
    op_sub_slow(state, a, b, c, feedback_slot, instruction_len)
}

#[cold]
#[inline(never)]
fn op_sub_slow(
    state: &mut DispatchState,
    a: u16,
    b: u16,
    c: u16,
    feedback_slot: Option<FeedbackSlotId>,
    instruction_len: u32,
) -> Step {
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.execute_sub_opcode(agent, *host, &mut **registry, frame, b, c)
    };
    let finish = {
        let DispatchState {
            vm,
            agent,
            frame,
            frame_depth,
            ..
        } = &mut *state;
        vm.finish_abc_value_result(
            agent,
            *frame_depth,
            frame,
            instruction_len,
            feedback_slot,
            a,
            result,
        )
    };
    try_step!(finish);
    dispatch_next!(state);
}

pub extern "C" fn op_mul(state: &mut DispatchState) -> Step {
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

    let registers = state.frame.registers();
    let left = state.vm.read_register_unchecked(registers, b);
    let right = state.vm.read_register_unchecked(registers, c);
    if let (Some(l), Some(r)) = (left.as_smi(), right.as_smi())
        && let Some(v) = smi_mul_result(l, r)
    {
        state.vm.record_feedback_slot(code, feedback_slot);
        state.vm.write_register_unchecked(registers, a, v);
        state.advance(instruction_len);
        dispatch_next!(state);
    }
    op_mul_slow(state, a, b, c, feedback_slot, instruction_len)
}

#[cold]
#[inline(never)]
fn op_mul_slow(
    state: &mut DispatchState,
    a: u16,
    b: u16,
    c: u16,
    feedback_slot: Option<FeedbackSlotId>,
    instruction_len: u32,
) -> Step {
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.execute_mul_opcode(agent, *host, &mut **registry, frame, b, c)
    };
    let finish = {
        let DispatchState {
            vm,
            agent,
            frame,
            frame_depth,
            ..
        } = &mut *state;
        vm.finish_abc_value_result(
            agent,
            *frame_depth,
            frame,
            instruction_len,
            feedback_slot,
            a,
            result,
        )
    };
    try_step!(finish);
    dispatch_next!(state);
}

// =====================================================================
// Negate / BitNot / Increment / Decrement — unary, no inline SMI fast
// path (the Vm helpers have one internally)
// =====================================================================

pub extern "C" fn op_negate(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let prefix = state.prefix.take();
    let (a, b, _c, feedback_slot, instruction_len) = try_step!(decode_abc_operands(
        state.current_bytes(),
        prefix,
        true,
        code,
        pc,
    ));
    let negate_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.negate_value(agent, *host, &mut **registry, frame, b)
    };
    let value = match try_step!(state.handle_dispatch_result(negate_result)) {
        Some(v) => v,
        None => dispatch_next!(state),
    };
    state.vm.record_feedback_slot(code, feedback_slot);
    let registers = state.frame.registers();
    state.vm.write_register_unchecked(registers, a, value);
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_bit_not(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let prefix = state.prefix.take();
    let (a, b, _c, feedback_slot, instruction_len) = try_step!(decode_abc_operands(
        state.current_bytes(),
        prefix,
        true,
        code,
        pc,
    ));
    let bit_not_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.bitwise_not_value(agent, *host, &mut **registry, frame, b)
    };
    let value = match try_step!(state.handle_dispatch_result(bit_not_result)) {
        Some(v) => v,
        None => dispatch_next!(state),
    };
    state.vm.record_feedback_slot(code, feedback_slot);
    let registers = state.frame.registers();
    state.vm.write_register_unchecked(registers, a, value);
    state.advance(instruction_len);
    dispatch_next!(state);
}

#[inline]
fn op_update_register(state: &mut DispatchState, increment: bool) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let prefix = state.prefix.take();
    let (a, b, _c, feedback_slot, instruction_len) = try_step!(decode_abc_operands(
        state.current_bytes(),
        prefix,
        true,
        code,
        pc,
    ));
    let update_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.update_register_value(agent, *host, &mut **registry, frame, b, increment)
    };
    let (numeric, value) = match try_step!(state.handle_dispatch_result(update_result)) {
        Some(pair) => pair,
        None => dispatch_next!(state),
    };
    let registers = state.frame.registers();
    state.vm.write_register_unchecked(registers, b, numeric);
    state.vm.record_feedback_slot(code, feedback_slot);
    state.vm.write_register_unchecked(registers, a, value);
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_increment(state: &mut DispatchState) -> Step {
    op_update_register(state, true)
}

pub extern "C" fn op_decrement(state: &mut DispatchState) -> Step {
    op_update_register(state, false)
}

// =====================================================================
// AddSmi / SubSmi / MulSmi — register + i16 immediate, Abc-encoded
// =====================================================================

pub extern "C" fn op_add_smi(state: &mut DispatchState) -> Step {
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

    let registers = state.frame.registers();
    let left = state.vm.read_register_unchecked(registers, b);
    let imm = i32::from(decode_smi_immediate(c));
    if let Some(l) = left.as_smi()
        && let Some(v) = l.checked_add(imm)
    {
        state.vm.record_feedback_slot(code, feedback_slot);
        state
            .vm
            .write_register_unchecked(registers, a, Value::from_smi(v));
        state.advance(instruction_len);
        dispatch_next!(state);
    }
    op_add_smi_slow(state, a, b, c, feedback_slot, instruction_len)
}

#[cold]
#[inline(never)]
fn op_add_smi_slow(
    state: &mut DispatchState,
    a: u16,
    b: u16,
    c: u16,
    feedback_slot: Option<FeedbackSlotId>,
    instruction_len: u32,
) -> Step {
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.execute_add_smi_opcode(agent, *host, &mut **registry, frame, b, c)
    };
    let finish = {
        let DispatchState {
            vm,
            agent,
            frame,
            frame_depth,
            ..
        } = &mut *state;
        vm.finish_abc_value_result(
            agent,
            *frame_depth,
            frame,
            instruction_len,
            feedback_slot,
            a,
            result,
        )
    };
    try_step!(finish);
    dispatch_next!(state);
}

pub extern "C" fn op_sub_smi(state: &mut DispatchState) -> Step {
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

    let registers = state.frame.registers();
    let left = state.vm.read_register_unchecked(registers, b);
    let imm = i32::from(decode_smi_immediate(c));
    if let Some(l) = left.as_smi()
        && let Some(v) = l.checked_sub(imm)
    {
        state.vm.record_feedback_slot(code, feedback_slot);
        state
            .vm
            .write_register_unchecked(registers, a, Value::from_smi(v));
        state.advance(instruction_len);
        dispatch_next!(state);
    }
    op_sub_smi_slow(state, a, b, c, feedback_slot, instruction_len)
}

#[cold]
#[inline(never)]
fn op_sub_smi_slow(
    state: &mut DispatchState,
    a: u16,
    b: u16,
    c: u16,
    feedback_slot: Option<FeedbackSlotId>,
    instruction_len: u32,
) -> Step {
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.execute_sub_smi_opcode(agent, *host, &mut **registry, frame, b, c)
    };
    let finish = {
        let DispatchState {
            vm,
            agent,
            frame,
            frame_depth,
            ..
        } = &mut *state;
        vm.finish_abc_value_result(
            agent,
            *frame_depth,
            frame,
            instruction_len,
            feedback_slot,
            a,
            result,
        )
    };
    try_step!(finish);
    dispatch_next!(state);
}

pub extern "C" fn op_mul_smi(state: &mut DispatchState) -> Step {
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

    let registers = state.frame.registers();
    let left = state.vm.read_register_unchecked(registers, b);
    let imm = i32::from(decode_smi_immediate(c));
    if let Some(l) = left.as_smi()
        && let Some(v) = smi_mul_result(l, imm)
    {
        state.vm.record_feedback_slot(code, feedback_slot);
        state.vm.write_register_unchecked(registers, a, v);
        state.advance(instruction_len);
        dispatch_next!(state);
    }
    op_mul_smi_slow(state, a, b, c, feedback_slot, instruction_len)
}

#[cold]
#[inline(never)]
fn op_mul_smi_slow(
    state: &mut DispatchState,
    a: u16,
    b: u16,
    c: u16,
    feedback_slot: Option<FeedbackSlotId>,
    instruction_len: u32,
) -> Step {
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.execute_mul_smi_opcode(agent, *host, &mut **registry, frame, b, c)
    };
    let finish = {
        let DispatchState {
            vm,
            agent,
            frame,
            frame_depth,
            ..
        } = &mut *state;
        vm.finish_abc_value_result(
            agent,
            *frame_depth,
            frame,
            instruction_len,
            feedback_slot,
            a,
            result,
        )
    };
    try_step!(finish);
    dispatch_next!(state);
}

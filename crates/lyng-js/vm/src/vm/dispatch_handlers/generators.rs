//! Generator + async handlers for the trampoline dispatch path
//! (lyng-59e6 round 4). All `#[cold]` — rare opcodes per JSC's hot/cold
//! analysis.
//!
//! Yield / Await / SuspendGeneratorStart suspend the active frame via the
//! existing Vm helpers, which return `VmResult<()>` whose error variant
//! carries the abrupt completion (GeneratorYield, AsyncSuspend). The
//! trampoline propagates that via `Step::Error`.
//!
//! DelegateYield is a full opcode that orchestrates yield-from semantics
//! and may also suspend; same flow.
//!
//! LoadResumeKind / LoadResumeValue read frame state populated by the
//! resume entry path and clear the resume slot; no suspension.

use lyng_js_types::Value;

use crate::error::VmError;
use crate::vm::dispatch::{decode_abc_operands, decode_ax_operands};
use crate::vm::dispatch::next_dispatch_instruction_offset;
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::{dispatch_next, try_step};

#[inline]
fn ax_to_register(state: &DispatchState, ax: i32) -> Result<u16, Step> {
    u16::try_from(ax).map_err(|_| {
        Step::Error(VmError::RegisterOutOfBounds {
            code: state.frame.code(),
            register: 0,
        })
    })
}

#[cold]
pub extern "C" fn op_suspend_generator_start(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (_ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let resume_offset = next_dispatch_instruction_offset(&state.frame, instruction_len);
    state.sync_active_frame();
    {
        let DispatchState {
            vm, agent, frame, ..
        } = &mut *state;
        try_step!(vm.suspend_generator_start(agent, frame, resume_offset));
    }
    dispatch_next!(state);
}

#[cold]
pub extern "C" fn op_yield(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let register = match ax_to_register(state, ax) {
        Ok(r) => r,
        Err(step) => return step,
    };
    let value = state.vm.read_register_unchecked(state.frame.registers(), register);
    let resume_offset = next_dispatch_instruction_offset(&state.frame, instruction_len);
    state.sync_active_frame();
    {
        let DispatchState {
            vm, agent, frame, ..
        } = &mut *state;
        try_step!(vm.suspend_current_generator_frame(agent, frame, value, resume_offset, false));
    }
    dispatch_next!(state);
}

#[cold]
pub extern "C" fn op_delegate_yield(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let prefix = state.prefix.take();
    let (a, b, c, _feedback_slot, instruction_len) = try_step!(decode_abc_operands(
        state.current_bytes(),
        prefix,
        false,
        code,
        pc,
    ));
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *state;
        vm.delegate_yield(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            instruction_len,
            a,
            b,
            c,
        )
    };
    if try_step!(state.handle_dispatch_result(result)).is_none() {
        dispatch_next!(state);
    }
    dispatch_next!(state);
}

#[cold]
pub extern "C" fn op_await(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let register = match ax_to_register(state, ax) {
        Ok(r) => r,
        Err(step) => return step,
    };
    {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *state;
        try_step!(vm.await_value(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            instruction_len,
            register,
        ));
    }
    dispatch_next!(state);
}

pub extern "C" fn op_load_resume_kind(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let register = match ax_to_register(state, ax) {
        Ok(r) => r,
        Err(step) => return step,
    };
    let kind = state.frame.resume_kind().raw();
    let registers = state.frame.registers();
    state
        .vm
        .write_register_unchecked(registers, register, Value::from_smi(i32::from(kind)));
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_load_resume_value(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let register = match ax_to_register(state, ax) {
        Ok(r) => r,
        Err(step) => return step,
    };
    let value = state.frame.resume_value();
    let registers = state.frame.registers();
    state.vm.write_register_unchecked(registers, register, value);
    state.frame.clear_resume();
    state.advance(instruction_len);
    dispatch_next!(state);
}

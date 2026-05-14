//! Environment-scope handlers for the trampoline dispatch path
//! (lyng-59e6 round 1).
//!
//! Lexical-environment slot read/write/assign + scope push/pop opcodes.
//! Each handler decodes its Abx/Ax operands, resolves the environment via
//! the existing `Vm::environment_for_slot_access` walk + the
//! `decode_env_operand` (depth, slot) split, and routes the result
//! through the standard handle_dispatch_result mechanism.
//!
//! Also includes TypeOf — it's an Ax-form opcode that doesn't fit any
//! other family file but reads frame state in the same shape as the
//! scope handlers.

use crate::error::VmError;
use crate::vm::dispatch::{decode_abx_operands, decode_ax_operands};
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::vm::values::decode_env_operand;
use crate::vm::Vm;
use crate::{dispatch_next, try_step};

// =====================================================================
// LoadEnvSlot / StoreEnvSlot / AssignEnvSlot — Abx; bx encodes (depth, slot).
// =====================================================================

pub extern "C" fn op_load_env_slot(state: &mut DispatchState) -> Step {
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
    let (depth, slot) = decode_env_operand(bx);
    let lexical_env = state.frame.lexical_env();
    let environment = {
        let DispatchState { vm, agent, .. } = &mut *state;
        try_step!(vm.environment_for_slot_access(agent, lexical_env, depth, slot))
    };
    let load_result = {
        let DispatchState { agent, .. } = &mut *state;
        Vm::read_environment_slot(agent, environment, slot)
    };
    let value = match try_step!(state.handle_dispatch_result(load_result)) {
        Some(v) => v,
        None => dispatch_next!(state),
    };
    let registers = state.frame.registers();
    state.vm.write_register_unchecked(registers, a, value);
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_store_env_slot(state: &mut DispatchState) -> Step {
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
    let (depth, slot) = decode_env_operand(bx);
    let lexical_env = state.frame.lexical_env();
    let environment = {
        let DispatchState { vm, agent, .. } = &mut *state;
        try_step!(vm.environment_for_slot_access(agent, lexical_env, depth, slot))
    };
    let value = state.vm.read_register_unchecked(state.frame.registers(), a);
    let store_result = {
        let DispatchState { vm, agent, .. } = &mut *state;
        vm.write_environment_slot(agent, environment, slot, value)
    };
    if try_step!(state.handle_dispatch_result(store_result)).is_none() {
        dispatch_next!(state);
    }
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_assign_env_slot(state: &mut DispatchState) -> Step {
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
    let (depth, slot) = decode_env_operand(bx);
    let lexical_env = state.frame.lexical_env();
    let environment = {
        let DispatchState { vm, agent, .. } = &mut *state;
        try_step!(vm.environment_for_slot_access(agent, lexical_env, depth, slot))
    };
    let value = state.vm.read_register_unchecked(state.frame.registers(), a);
    let strict = state.vm.frame_is_strict(&state.frame);
    let assign_result = {
        let DispatchState { vm, agent, .. } = &mut *state;
        vm.assign_environment_slot(agent, environment, slot, value, strict)
    };
    if try_step!(state.handle_dispatch_result(assign_result)).is_none() {
        dispatch_next!(state);
    }
    state.advance(instruction_len);
    dispatch_next!(state);
}

// =====================================================================
// EnterEnvScope / LeaveEnvScope — block-scope binding chunks
// =====================================================================

pub extern "C" fn op_enter_env_scope(state: &mut DispatchState) -> Step {
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
    {
        let DispatchState {
            vm, agent, frame, ..
        } = &mut *state;
        try_step!(vm.enter_env_scope(agent, frame, a, bx));
    }
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_leave_env_scope(state: &mut DispatchState) -> Step {
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
    state.vm.leave_env_scope(&state.frame, a, bx);
    state.advance(instruction_len);
    dispatch_next!(state);
}

// =====================================================================
// PushClosureEnv / PopClosureEnv — loop-iteration environment chain
// =====================================================================

pub extern "C" fn op_push_closure_env(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));

    let site = state
        .installed
        .loop_iteration_environment_site(state.frame.instruction_offset())
        .cloned();
    let mirrored_slot = if ax > 0 {
        match u32::try_from(ax - 1) {
            Ok(v) => Some(v),
            Err(_) => {
                return Step::Error(VmError::UnsupportedOpcode {
                    code: state.frame.code(),
                    instruction_offset: state.frame.instruction_offset(),
                    opcode: lyng_js_bytecode::Opcode::PushClosureEnv,
                });
            }
        }
    } else {
        None
    };
    {
        let DispatchState {
            vm, agent, frame, ..
        } = &mut *state;
        try_step!(vm.push_loop_iteration_environment(agent, frame, site, mirrored_slot));
    }
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_pop_closure_env(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (_ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    state.vm.pop_loop_iteration_environment();
    state.advance(instruction_len);
    dispatch_next!(state);
}

// =====================================================================
// PushWithEnv / PopWithEnv — with-statement environment chain
// =====================================================================

pub extern "C" fn op_push_with_env(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let register = match u16::try_from(ax) {
        Ok(r) => r,
        Err(_) => {
            return Step::Error(VmError::RegisterOutOfBounds {
                code: state.frame.code(),
                register: 0,
            });
        }
    };
    let value = state.vm.read_register_unchecked(state.frame.registers(), register);
    let push_result = {
        let DispatchState {
            vm, agent, frame, ..
        } = &mut *state;
        vm.push_with_environment(agent, frame, value)
    };
    if try_step!(state.handle_dispatch_result(push_result)).is_none() {
        dispatch_next!(state);
    }
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_pop_with_env(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (_ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    state.vm.pop_with_environment(&mut state.frame);
    state.advance(instruction_len);
    dispatch_next!(state);
}

// =====================================================================
// TypeOf — Ax form, reads/writes the same register
// =====================================================================

pub extern "C" fn op_type_of(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let (ax, _feedback_slot, instruction_len) =
        try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
    let register = match u16::try_from(ax) {
        Ok(r) => r,
        Err(_) => {
            return Step::Error(VmError::RegisterOutOfBounds {
                code: state.frame.code(),
                register: 0,
            });
        }
    };
    let registers = state.frame.registers();
    let value = state.vm.read_register_unchecked(registers, register);
    let type_string = {
        let DispatchState { agent, .. } = &mut *state;
        Vm::type_of_value(agent, value)
    };
    let registers = state.frame.registers();
    state.vm.write_register_unchecked(registers, register, type_string);
    state.advance(instruction_len);
    dispatch_next!(state);
}

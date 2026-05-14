//! Global + name resolution handlers for the trampoline dispatch path
//! (lyng-5mqv).
//!
//! All Abx-encoded; operand `bx` is an atom-constant-pool index. The
//! handler reads the atom and delegates to the `Vm::*_with_context` or
//! `Vm::*_with_feedback` helper, which carries the realm + lexical-env
//! walk plus the inline cache. Exception transfer goes through
//! `handle_dispatch_result`.
//!
//! Also hosts LoadThis / LoadCallee / LoadNewTarget — they read frame
//! state directly without an atom operand, but they live in the
//! "name & global" family from the spec's perspective.

use lyng_js_env::ThisState;
use lyng_js_ops::errors;
use lyng_js_types::{AbruptCompletion, Value};

use crate::error::VmError;
use crate::vm::dispatch::decode_abx_operands;
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::vm::Vm;
use crate::{dispatch_next, try_step};

// ---- Globals (with feedback) ----

pub extern "C" fn op_load_global(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let prefix = state.prefix.take();
    let (a, bx, feedback_slot, instruction_len) = try_step!(decode_abx_operands(
        state.current_bytes(),
        prefix,
        true,
        code,
        pc,
    ));
    let atom = try_step!(state.vm.read_atom_constant(code, bx));
    let load_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.load_global_with_feedback(
            agent,
            *host,
            &mut **registry,
            frame,
            atom,
            code,
            feedback_slot,
        )
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

#[inline]
fn op_store_or_assign_global(state: &mut DispatchState, assign: bool) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let prefix = state.prefix.take();
    let (a, bx, feedback_slot, instruction_len) = try_step!(decode_abx_operands(
        state.current_bytes(),
        prefix,
        true,
        code,
        pc,
    ));
    let atom = try_step!(state.vm.read_atom_constant(code, bx));
    let value = state.vm.read_register_unchecked(state.frame.registers(), a);
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        if assign {
            vm.assign_global_with_feedback(
                agent,
                *host,
                &mut **registry,
                frame,
                atom,
                value,
                code,
                feedback_slot,
            )
        } else {
            vm.store_global_with_feedback(
                agent,
                *host,
                &mut **registry,
                frame,
                atom,
                value,
                code,
                feedback_slot,
            )
        }
    };
    if try_step!(state.handle_dispatch_result(result)).is_none() {
        dispatch_next!(state);
    }
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_store_global(state: &mut DispatchState) -> Step {
    op_store_or_assign_global(state, false)
}

pub extern "C" fn op_assign_global(state: &mut DispatchState) -> Step {
    op_store_or_assign_global(state, true)
}

pub extern "C" fn op_delete_global(state: &mut DispatchState) -> Step {
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
    let atom = try_step!(state.vm.read_atom_constant(code, bx));
    let delete_result = {
        let DispatchState { agent, frame, .. } = &mut *state;
        Vm::delete_global(agent, frame, atom)
    };
    let deleted = match try_step!(state.handle_dispatch_result(delete_result)) {
        Some(v) => v,
        None => dispatch_next!(state),
    };
    let registers = state.frame.registers();
    state
        .vm
        .write_register_unchecked(registers, a, Value::from_bool(deleted));
    state.advance(instruction_len);
    dispatch_next!(state);
}

// ---- Names (lexical scope walk) ----

pub extern "C" fn op_load_name(state: &mut DispatchState) -> Step {
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
    let atom = try_step!(state.vm.read_atom_constant(code, bx));
    let load_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.load_name_with_context(agent, *host, &mut **registry, frame, atom)
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

pub extern "C" fn op_resolve_name(state: &mut DispatchState) -> Step {
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
    let atom = try_step!(state.vm.read_atom_constant(code, bx));
    let resolve_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.resolve_name_with_context(agent, *host, &mut **registry, frame, atom)
    };
    let value = match try_step!(state.handle_dispatch_result(resolve_result)) {
        Some(v) => v,
        None => dispatch_next!(state),
    };
    let registers = state.frame.registers();
    state.vm.write_register_unchecked(registers, a, value);
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_resolve_global(state: &mut DispatchState) -> Step {
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
    let atom = try_step!(state.vm.read_atom_constant(code, bx));
    let resolve_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.resolve_global(agent, *host, &mut **registry, frame, atom)
    };
    let value = match try_step!(state.handle_dispatch_result(resolve_result)) {
        Some(v) => v,
        None => dispatch_next!(state),
    };
    let registers = state.frame.registers();
    state.vm.write_register_unchecked(registers, a, value);
    state.advance(instruction_len);
    dispatch_next!(state);
}

#[inline]
fn op_assign_name_common(state: &mut DispatchState, variable_form: bool) -> Step {
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
    let atom = try_step!(state.vm.read_atom_constant(code, bx));
    let value = state.vm.read_register_unchecked(state.frame.registers(), a);
    let assign_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        if variable_form {
            vm.assign_variable_name_with_context(agent, *host, &mut **registry, frame, atom, value)
        } else {
            vm.assign_name_with_context(agent, *host, &mut **registry, frame, atom, value)
        }
    };
    if try_step!(state.handle_dispatch_result(assign_result)).is_none() {
        dispatch_next!(state);
    }
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_assign_name(state: &mut DispatchState) -> Step {
    op_assign_name_common(state, false)
}

pub extern "C" fn op_assign_variable_name(state: &mut DispatchState) -> Step {
    op_assign_name_common(state, true)
}

pub extern "C" fn op_delete_name(state: &mut DispatchState) -> Step {
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
    let atom = try_step!(state.vm.read_atom_constant(code, bx));
    let delete_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.delete_name_with_context(agent, *host, &mut **registry, frame, atom)
    };
    let deleted = match try_step!(state.handle_dispatch_result(delete_result)) {
        Some(v) => v,
        None => dispatch_next!(state),
    };
    let registers = state.frame.registers();
    state
        .vm
        .write_register_unchecked(registers, a, Value::from_bool(deleted));
    state.advance(instruction_len);
    dispatch_next!(state);
}

// ---- Captured names (closures) ----

pub extern "C" fn op_capture_name(state: &mut DispatchState) -> Step {
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
    let atom = try_step!(state.vm.read_atom_constant(code, bx));
    let capture_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.capture_name_with_context(agent, *host, &mut **registry, frame, a, atom)
    };
    if try_step!(state.handle_dispatch_result(capture_result)).is_none() {
        dispatch_next!(state);
    }
    state.advance(instruction_len);
    dispatch_next!(state);
}

#[inline]
fn captured_name_register(state: &DispatchState, bx: u32) -> Result<u16, Step> {
    u16::try_from(bx).map_err(|_| {
        Step::Error(VmError::RegisterOutOfBounds {
            code: state.frame.code(),
            register: u16::MAX,
        })
    })
}

pub extern "C" fn op_load_captured_name(state: &mut DispatchState) -> Step {
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
    let reference_register = match captured_name_register(state, bx) {
        Ok(r) => r,
        Err(step) => return step,
    };
    let load_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.load_captured_name_with_context(agent, *host, &mut **registry, frame, reference_register)
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

pub extern "C" fn op_load_captured_name_this(state: &mut DispatchState) -> Step {
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
    let reference_register = match captured_name_register(state, bx) {
        Ok(r) => r,
        Err(step) => return step,
    };
    let load_result = state
        .vm
        .load_captured_name_this_with_context(&state.frame, reference_register);
    let value = match try_step!(state.handle_dispatch_result(load_result)) {
        Some(v) => v,
        None => dispatch_next!(state),
    };
    let registers = state.frame.registers();
    state.vm.write_register_unchecked(registers, a, value);
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_assign_captured_name(state: &mut DispatchState) -> Step {
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
    let reference_register = match captured_name_register(state, bx) {
        Ok(r) => r,
        Err(step) => return step,
    };
    let value = state.vm.read_register_unchecked(state.frame.registers(), a);
    let assign_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.assign_captured_name_with_context(
            agent,
            *host,
            &mut **registry,
            frame,
            reference_register,
            value,
        )
    };
    if try_step!(state.handle_dispatch_result(assign_result)).is_none() {
        dispatch_next!(state);
    }
    state.advance(instruction_len);
    dispatch_next!(state);
}

// ---- Frame-state loads: This / Callee / NewTarget ----

pub extern "C" fn op_load_this(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let prefix = state.prefix.take();
    let (a, _bx, _feedback_slot, instruction_len) = try_step!(decode_abx_operands(
        state.current_bytes(),
        prefix,
        false,
        code,
        pc,
    ));
    let load_this = {
        let DispatchState { agent, frame, .. } = &mut *state;
        let this_state = agent
            .current_execution_context()
            .map_or_else(|| ThisState::Value(frame.this_value()), |ec| ec.this_state());
        match this_state {
            ThisState::Value(value) => Ok(value),
            ThisState::Uninitialized => Err(VmError::Abrupt(errors::throw_reference_error(agent))),
            ThisState::Lexical => Vm::resolve_this_binding(agent, frame.lexical_env(), frame),
        }
    };
    let value = match try_step!(state.handle_dispatch_result(load_this)) {
        Some(v) => v,
        None => dispatch_next!(state),
    };
    let registers = state.frame.registers();
    state.vm.write_register_unchecked(registers, a, value);
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_load_callee(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let prefix = state.prefix.take();
    let (a, _bx, _feedback_slot, instruction_len) = try_step!(decode_abx_operands(
        state.current_bytes(),
        prefix,
        false,
        code,
        pc,
    ));
    let value = state
        .frame
        .callee()
        .map_or(Value::undefined(), Value::from_object_ref);
    let registers = state.frame.registers();
    state.vm.write_register_unchecked(registers, a, value);
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_load_new_target(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let prefix = state.prefix.take();
    let (a, _bx, _feedback_slot, instruction_len) = try_step!(decode_abx_operands(
        state.current_bytes(),
        prefix,
        false,
        code,
        pc,
    ));
    let value = state
        .frame
        .new_target()
        .map_or(Value::undefined(), Value::from_object_ref);
    let registers = state.frame.registers();
    state.vm.write_register_unchecked(registers, a, value);
    state.advance(instruction_len);
    dispatch_next!(state);
}

// Suppress unused-import warning when AbruptCompletion is only needed
// for the throw_reference_error trip through VmError::Abrupt.
#[allow(dead_code)]
const _: Option<AbruptCompletion> = None;

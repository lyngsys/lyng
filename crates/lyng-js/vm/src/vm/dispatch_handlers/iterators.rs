//! Iterator + for-in handlers for the trampoline dispatch path
//! (lyng-59e6 round 2).
//!
//! Each handler decodes Abc/Abx operands, walks the Vm's for-in /
//! iterator side tables (via thin `for_in_*` / `iterator_*` wrappers on
//! `Vm`), and routes the result through `handle_dispatch_result` for
//! exception transfer.

use lyng_js_types::Value;

use crate::vm::dispatch::{decode_abc_operands, decode_abx_operands};
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::{dispatch_next, try_step};

pub extern "C" fn op_create_for_in(state: &mut DispatchState) -> Step {
    let code = state.code();
    let pc = state.frame.instruction_offset();
    let prefix = state.prefix.take();
    let (a, b, _c, _feedback_slot, instruction_len) = try_step!(decode_abc_operands(
        state.current_bytes(),
        prefix,
        false,
        code,
        pc,
    ));
    let value = state.vm.read_register_unchecked(state.frame.registers(), b);
    let enumerator_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.create_for_in_enumerator_for_value(agent, *host, &mut **registry, frame, value)
    };
    let enumerator = match try_step!(state.handle_dispatch_result(enumerator_result)) {
        Some(e) => e,
        None => dispatch_next!(state),
    };
    let base = state.frame.registers().base();
    state.vm.for_in_insert(base, a, enumerator);
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_advance_for_in(state: &mut DispatchState) -> Step {
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
    let base = state.frame.registers().base();
    let next_result = {
        let DispatchState { vm, agent, .. } = &mut *state;
        vm.for_in_advance(agent, base, a)
    };
    let next = match try_step!(state.handle_dispatch_result(next_result)) {
        Some(v) => v,
        None => dispatch_next!(state),
    };
    let done = next.is_none();
    let value = match next {
        Some(key) => {
            let DispatchState { vm, agent, .. } = &mut *state;
            vm.property_key_to_enumeration_value(agent, key)
        }
        None => Value::undefined(),
    };
    let registers = state.frame.registers();
    state.vm.write_register_unchecked(registers, b, value);
    state
        .vm
        .write_register_unchecked(registers, c, Value::from_bool(done));
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_close_for_in(state: &mut DispatchState) -> Step {
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
    let base = state.frame.registers().base();
    state.vm.for_in_remove(base, a);
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_create_iterator(state: &mut DispatchState) -> Step {
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
    let value = state.vm.read_register_unchecked(state.frame.registers(), b);
    let is_async = c != 0;
    let iterator_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *state;
        vm.create_iterator_for_value(agent, *host, &mut **registry, frame, value, is_async)
    };
    let iterator = match try_step!(state.handle_dispatch_result(iterator_result)) {
        Some(i) => i,
        None => dispatch_next!(state),
    };
    let base = state.frame.registers().base();
    state.vm.iterator_insert(base, a, iterator);
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_advance_iterator(state: &mut DispatchState) -> Step {
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
    state.sync_active_frame();
    let next_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *state;
        vm.advance_iterator_state(agent, *host, &mut **registry, *frame_depth, frame, a)
    };
    let next = match try_step!(state.handle_dispatch_result(next_result)) {
        Some(v) => v,
        None => dispatch_next!(state),
    };
    let done = next.is_none();
    let value = next.unwrap_or(Value::undefined());
    let registers = state.frame.registers();
    state.vm.write_register_unchecked(registers, b, value);
    state
        .vm
        .write_register_unchecked(registers, c, Value::from_bool(done));
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_close_iterator(state: &mut DispatchState) -> Step {
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
    let close_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *state;
        vm.close_iterator_state(agent, *host, &mut **registry, *frame_depth, frame, a, bx != 0)
    };
    if try_step!(state.handle_dispatch_result(close_result)).is_none() {
        dispatch_next!(state);
    }
    state.advance(instruction_len);
    dispatch_next!(state);
}

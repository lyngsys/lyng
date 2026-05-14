//! Property access handlers for the trampoline dispatch path (lyng-5mqv).
//!
//! Named-property and keyed-property opcodes delegate entirely to the
//! existing `Vm::execute_*_opcode` helpers; those helpers carry the
//! receiver-shape inline-cache fast paths, the ToObject coercion, the
//! property-descriptor walk, and the exception transfer in one place.
//! Each trampoline handler:
//!
//! 1. Decodes Abc operands via `decode_abc_operands` (with `state.prefix`
//!    consumed via `.take()`). Every property opcode has a feedback slot,
//!    so `is_profiled = true`.
//! 2. Splits the vm/agent/host/registry/frame_depth/frame borrow.
//! 3. Calls the Vm helper, which advances `pc` on success or rewrites it
//!    via the exception handler.
//! 4. `dispatch_next!` continues at the new pc.
//!
//! The Set/Assign variants pass the active semantic `Opcode` so the helper
//! can dispatch the strict-mode / assignment / property-define semantics
//! the spec assigns to each form.

use lyng_js_bytecode::Opcode;
use lyng_js_types::Value;

use crate::vm::dispatch::{decode_abc_operands, decode_abx_operands};
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::vm::Vm;
use crate::{dispatch_next, try_step};

pub extern "C" fn op_get_named_property(state: &mut DispatchState) -> Step {
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
        vm.execute_get_named_property_opcode(
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
        )
    };
    try_step!(result);
    dispatch_next!(state);
}

#[inline]
fn op_set_named_property_common(state: &mut DispatchState, semantic: Opcode) -> Step {
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
        vm.execute_set_named_property_opcode(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            instruction_len,
            feedback_slot,
            semantic,
            a,
            b,
            c,
        )
    };
    try_step!(result);
    dispatch_next!(state);
}

pub extern "C" fn op_set_named_property(state: &mut DispatchState) -> Step {
    op_set_named_property_common(state, Opcode::SetNamedProperty)
}

pub extern "C" fn op_assign_named_property(state: &mut DispatchState) -> Step {
    op_set_named_property_common(state, Opcode::AssignNamedProperty)
}

pub extern "C" fn op_strict_assign_named_property(state: &mut DispatchState) -> Step {
    op_set_named_property_common(state, Opcode::StrictAssignNamedProperty)
}

pub extern "C" fn op_get_keyed_property(state: &mut DispatchState) -> Step {
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
        vm.execute_get_keyed_property_opcode(
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
        )
    };
    try_step!(result);
    dispatch_next!(state);
}

#[inline]
fn op_set_keyed_property_common(state: &mut DispatchState, semantic: Opcode) -> Step {
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
        vm.execute_set_keyed_property_opcode(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            instruction_len,
            feedback_slot,
            semantic,
            a,
            b,
            c,
        )
    };
    try_step!(result);
    dispatch_next!(state);
}

pub extern "C" fn op_set_keyed_property(state: &mut DispatchState) -> Step {
    op_set_keyed_property_common(state, Opcode::SetKeyedProperty)
}

pub extern "C" fn op_assign_keyed_property(state: &mut DispatchState) -> Step {
    op_set_keyed_property_common(state, Opcode::AssignKeyedProperty)
}

pub extern "C" fn op_strict_assign_keyed_property(state: &mut DispatchState) -> Step {
    op_set_keyed_property_common(state, Opcode::StrictAssignKeyedProperty)
}

pub extern "C" fn op_define_named_property(state: &mut DispatchState) -> Step {
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
        vm.execute_define_named_property_opcode(
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
    try_step!(result);
    dispatch_next!(state);
}

// =====================================================================
// CreateObject / CreateArray — object/array literal allocation.
// Strictly speaking part of the runtime-objects family, but in the
// compiler's emission they always precede property loads/stores, so
// pulling them into sub-5 unlocks the property parity tests.
// =====================================================================

pub extern "C" fn op_store_dense_element(state: &mut DispatchState) -> Step {
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
        vm.execute_store_dense_element_opcode(
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
    try_step!(result);
    dispatch_next!(state);
}

pub extern "C" fn op_load_dense_element(state: &mut DispatchState) -> Step {
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
        vm.execute_load_dense_element_opcode(
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
    try_step!(result);
    dispatch_next!(state);
}

pub extern "C" fn op_create_object(state: &mut DispatchState) -> Step {
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
    let realm = state.frame.realm();
    let object = {
        let DispatchState { agent, .. } = &mut *state;
        try_step!(Vm::create_object(
            agent,
            realm,
            usize::try_from(bx).unwrap_or(usize::MAX),
        ))
    };
    let registers = state.frame.registers();
    state
        .vm
        .write_register_unchecked(registers, a, Value::from_object_ref(object));
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_create_array(state: &mut DispatchState) -> Step {
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
    let realm = state.frame.realm();
    let length = usize::try_from(bx).unwrap_or(usize::MAX);
    let object = {
        let DispatchState { agent, .. } = &mut *state;
        try_step!(Vm::create_array(agent, realm, length))
    };
    let length_u32 = u32::try_from(length).unwrap_or(u32::MAX);
    if length_u32 != 0 {
        let DispatchState { agent, .. } = &mut *state;
        try_step!(Vm::define_length_property(agent, object, length_u32, false));
    }
    let registers = state.frame.registers();
    state
        .vm
        .write_register_unchecked(registers, a, Value::from_object_ref(object));
    state.advance(instruction_len);
    dispatch_next!(state);
}

pub extern "C" fn op_define_keyed_property(state: &mut DispatchState) -> Step {
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
        vm.execute_define_keyed_property_opcode(
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
    try_step!(result);
    dispatch_next!(state);
}

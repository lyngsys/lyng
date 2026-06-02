//! Property family semantic bodies.
//!
//! Family coverage (21 opcodes):
//! - Named property reads:  `GetNamedProperty`.
//! - Named property writes: `SetNamedProperty`, `AssignNamedProperty`,
//!   `StrictAssignNamedProperty`.
//! - Keyed property reads:  `GetKeyedProperty`.
//! - Keyed property writes: `SetKeyedProperty`, `AssignKeyedProperty`,
//!   `StrictAssignKeyedProperty`.
//! - Define-data variants:  `DefineNamedProperty`, `DefineKeyedProperty`.
//! - Object/array literal:  `CreateObject`, `CreateArray`.
//! - Dense element access:  `StoreDenseElement`, `LoadDenseElement`.
//! - Misc property runtime: `DeleteProperty`, `In`, `ToPropertyKey`,
//!   `CopyDataProperties`, `SetFunctionName`, `CheckObjectCoercible`,
//!   `ThrowIfUninitialized`.
//!
//! IC-heavy opcodes defer to `Vm::execute_*_opcode` helpers in
//! `vm/dispatch/property.rs`, which carry the inline-cache hit paths and
//! feedback-slot recording. `CreateObject` / `CreateArray` write the register
//! and advance by `instruction_len`. `SetFunctionName`, `CheckObjectCoercible`,
//! and `ThrowIfUninitialized` route through `handle_dispatch_result` directly.

use lyng_bytecode::Opcode;
use lyng_ops::errors;
use lyng_types::{FeedbackSlotId, Value};

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::error::VmError;
use crate::vm::Vm;
use crate::vm::dispatch_state::DispatchState;

// =====================================================================
// Shared shapes
// =====================================================================

/// Operands for the three-register Abc-encoded property opcodes that
/// carry a feedback slot.
///
/// Used by: `GetNamedProperty`, `SetNamedProperty`, `AssignNamedProperty`,
/// `StrictAssignNamedProperty`, `GetKeyedProperty`, `SetKeyedProperty`,
/// `AssignKeyedProperty`, `StrictAssignKeyedProperty`. Each opcode's
/// semantic interpretation of `(a, b, c)` is helper-specific; the shared
/// struct just carries the decoded values.
pub struct OpPropertyAccessArgs {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub feedback_slot: Option<FeedbackSlotId>,
    pub instruction_len: u32,
}

/// Operands for three-register Abc-encoded property opcodes that do NOT
/// carry a feedback slot (define-data variants, dense element access,
/// `DeleteProperty`, `In`, `CopyDataProperties`).
pub struct OpPropertyAbcArgs {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub instruction_len: u32,
}

/// Operands for the two-register Abc-encoded opcodes whose `c` field is
/// unused at the semantic level. Used by `ToPropertyKey` and
/// `SetFunctionName`.
pub struct OpPropertyAbArgs {
    pub a: u16,
    pub b: u16,
    pub instruction_len: u32,
}

/// Operands for the Abx-encoded property opcodes. Used by `CreateObject`,
/// `CreateArray`, `CheckObjectCoercible`, `ThrowIfUninitialized`. `bx`
/// is the 16-bit extended operand (slot count, element capacity, or
/// unused).
pub struct OpPropertyAbxArgs {
    pub a: u16,
    pub bx: u32,
    pub instruction_len: u32,
}

/// Route a `VmResult<()>` from a helper that has already advanced the frame PC
/// on success. Either way the next opcode is at the current PC, so the outcome
/// carries `pc_advance: 0`.
#[inline]
#[expect(
    dead_code,
    reason = "Retained for non-property opcode families that route VmResult<()> through this tail"
)]
fn route_execute_result(result: crate::error::VmResult<()>) -> SemanticOutcome {
    match result {
        Ok(()) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

// =====================================================================
// Named property reads — `GetNamedProperty`.
// =====================================================================

pub fn op_get_named_property_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAccessArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_get_named_property_opcode(
            agent,
            *host,
            &mut **registry,
            view,
            args.feedback_slot,
            args.b,
            args.c,
        )
    };
    let handled = inner.handle_dispatch_result(result);
    let value = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.registers();
    inner.vm.write_register(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// Named property writes — `SetNamedProperty`, `AssignNamedProperty`,
// `StrictAssignNamedProperty`.
// =====================================================================

#[inline]
fn op_set_named_property_shared(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAccessArgs,
    semantic: Opcode,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_set_named_property_opcode(
            agent,
            *host,
            &mut **registry,
            view,
            args.feedback_slot,
            semantic,
            args.a,
            args.b,
            args.c,
        )
    };
    match inner.handle_dispatch_result(result) {
        Ok(Some(())) => SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        },
        Ok(None) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

pub fn op_set_named_property_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAccessArgs,
) -> SemanticOutcome {
    op_set_named_property_shared(state, args, Opcode::SetNamedProperty)
}

pub fn op_assign_named_property_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAccessArgs,
) -> SemanticOutcome {
    op_set_named_property_shared(state, args, Opcode::AssignNamedProperty)
}

pub fn op_strict_assign_named_property_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAccessArgs,
) -> SemanticOutcome {
    op_set_named_property_shared(state, args, Opcode::StrictAssignNamedProperty)
}

// =====================================================================
// Keyed property reads — `GetKeyedProperty`.
// =====================================================================

pub fn op_get_keyed_property_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAccessArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_get_keyed_property_opcode(
            agent,
            *host,
            &mut **registry,
            view,
            args.feedback_slot,
            args.b,
            args.c,
        )
    };
    let handled = inner.handle_dispatch_result(result);
    let value = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.registers();
    inner.vm.write_register(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// Keyed property writes — `SetKeyedProperty`, `AssignKeyedProperty`,
// `StrictAssignKeyedProperty`.
// =====================================================================

#[inline]
fn op_set_keyed_property_shared(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAccessArgs,
    semantic: Opcode,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_set_keyed_property_opcode(
            agent,
            *host,
            &mut **registry,
            view,
            args.feedback_slot,
            semantic,
            args.a,
            args.b,
            args.c,
        )
    };
    match inner.handle_dispatch_result(result) {
        Ok(Some(())) => SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        },
        Ok(None) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

pub fn op_set_keyed_property_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAccessArgs,
) -> SemanticOutcome {
    op_set_keyed_property_shared(state, args, Opcode::SetKeyedProperty)
}

pub fn op_assign_keyed_property_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAccessArgs,
) -> SemanticOutcome {
    op_set_keyed_property_shared(state, args, Opcode::AssignKeyedProperty)
}

pub fn op_strict_assign_keyed_property_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAccessArgs,
) -> SemanticOutcome {
    op_set_keyed_property_shared(state, args, Opcode::StrictAssignKeyedProperty)
}

// =====================================================================
// Define-data — `DefineNamedProperty`, `DefineKeyedProperty`.
// =====================================================================

pub fn op_define_named_property_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_define_named_property_opcode(
            agent,
            *host,
            &mut **registry,
            view,
            args.a,
            args.b,
            args.c,
        )
    };
    match inner.handle_dispatch_result(result) {
        Ok(Some(())) => SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        },
        Ok(None) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

pub fn op_define_keyed_property_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_define_keyed_property_opcode(
            agent,
            *host,
            &mut **registry,
            view,
            args.a,
            args.b,
            args.c,
        )
    };
    match inner.handle_dispatch_result(result) {
        Ok(Some(())) => SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        },
        Ok(None) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

// =====================================================================
// Object / array literal allocation — `CreateObject`, `CreateArray`.
// =====================================================================

pub fn op_create_object_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let realm = inner.vm.realm_of(inner.agent, inner.cfr);
    let object = {
        let DispatchState { agent, .. } = &mut *inner;
        match Vm::create_object(agent, realm, usize::try_from(args.bx).unwrap_or(usize::MAX)) {
            Ok(object) => object,
            Err(error) => return SemanticOutcome::ExitError { error },
        }
    };
    let registers = inner.registers();
    inner
        .vm
        .write_register_unchecked(registers, args.a, Value::from_object_ref(object));
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub fn op_create_array_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let realm = inner.vm.realm_of(inner.agent, inner.cfr);
    let length = usize::try_from(args.bx).unwrap_or(usize::MAX);
    let object = {
        let DispatchState { agent, .. } = &mut *inner;
        match Vm::create_array(agent, realm, length) {
            Ok(object) => object,
            Err(error) => return SemanticOutcome::ExitError { error },
        }
    };
    let length_u32 = u32::try_from(length).unwrap_or(u32::MAX);
    if length_u32 != 0 {
        let DispatchState { agent, .. } = &mut *inner;
        if let Err(error) = Vm::define_length_property(agent, object, length_u32, false) {
            return SemanticOutcome::ExitError { error };
        }
    }
    let registers = inner.registers();
    inner
        .vm
        .write_register_unchecked(registers, args.a, Value::from_object_ref(object));
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// Dense element access — `StoreDenseElement`, `LoadDenseElement`.
// =====================================================================

pub fn op_store_dense_element_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_store_dense_element_opcode(
            agent,
            *host,
            &mut **registry,
            view,
            args.a,
            args.b,
            args.c,
        )
    };
    match inner.handle_dispatch_result(result) {
        Ok(Some(())) => SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        },
        Ok(None) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

pub fn op_load_dense_element_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_load_dense_element_opcode(agent, *host, &mut **registry, view, args.b, args.c)
    };
    let handled = inner.handle_dispatch_result(result);
    let value = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.registers();
    inner.vm.write_register(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// Misc property runtime — `DeleteProperty`, `In`, `ToPropertyKey`,
// `CopyDataProperties`, `SetFunctionName`, `CheckObjectCoercible`,
// `ThrowIfUninitialized`.
// =====================================================================

pub fn op_delete_property_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_delete_property_opcode(agent, *host, &mut **registry, view, args.b, args.c)
    };
    let handled = inner.handle_dispatch_result(result);
    let value = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.registers();
    inner.vm.write_register(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub fn op_in_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_in_opcode(agent, *host, &mut **registry, view, args.b, args.c)
    };
    let handled = inner.handle_dispatch_result(result);
    let value = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.registers();
    inner.vm.write_register(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub fn op_to_property_key_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_to_property_key_opcode(agent, *host, &mut **registry, view, args.b)
    };
    let handled = inner.handle_dispatch_result(result);
    let value = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.registers();
    inner.vm.write_register(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub fn op_copy_data_properties_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_copy_data_properties_opcode(
            agent,
            *host,
            &mut **registry,
            view,
            args.a,
            args.b,
            args.c,
        )
    };
    match inner.handle_dispatch_result(result) {
        Ok(Some(())) => SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        },
        Ok(None) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

pub fn op_set_function_name_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let function = match inner.vm.object_register(inner.frame_view(), args.a) {
        Ok(object) => object,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let name_value = inner.vm.read_register_unchecked(inner.registers(), args.b);
    let set_result = {
        let DispatchState { agent, .. } = &mut *inner;
        Vm::set_function_name(agent, function, name_value)
    };
    let handled = inner.handle_dispatch_result(set_result);
    match handled {
        Ok(Some(())) => SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        },
        Ok(None) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

/// `CheckObjectCoercible` — Abx-decoded; `a` is the candidate value register.
pub fn op_check_object_coercible_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner.vm.read_register_unchecked(inner.registers(), args.a);
    let result = {
        let DispatchState { agent, .. } = &mut *inner;
        Vm::check_object_coercible(agent, value)
    };
    let handled = inner.handle_dispatch_result(result);
    match handled {
        Ok(Some(())) => SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        },
        Ok(None) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

/// `ThrowIfUninitialized` — raises `ReferenceError` when the register holds the
/// TDZ sentinel; otherwise advances PC.
pub fn op_throw_if_uninitialized_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner.vm.read_register_unchecked(inner.registers(), args.a);
    if value == Value::uninitialized_lexical() {
        let result: Result<(), VmError> = {
            let DispatchState { agent, .. } = &mut *inner;
            Err(VmError::Abrupt(errors::throw_reference_error(agent)))
        };
        let handled = inner.handle_dispatch_result(result);
        match handled {
            // Unreachable: `handle_dispatch_result` on Err never returns `Some(())`.
            Ok(Some(())) => {}
            Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
            Err(error) => return SemanticOutcome::ExitError { error },
        }
    }
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

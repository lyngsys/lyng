//! Names family semantic bodies.
//!
//! Family coverage (17 opcodes):
//! - Globals with feedback: `LoadGlobal`, `StoreGlobal`, `AssignGlobal`.
//! - Globals without feedback: `DeleteGlobal`.
//! - Names (lexical scope walk): `LoadName`, `ResolveName`, `ResolveGlobal`,
//!   `AssignName`, `AssignVariableName`, `DeleteName`.
//! - Captured names: `CaptureName`, `LoadCapturedName`,
//!   `LoadCapturedNameThis`, `AssignCapturedName`.
//! - Frame-state loads: `LoadThis`, `LoadCallee`, `LoadNewTarget`.
//!
//! IC-bearing globals defer to `Vm::*_with_feedback` helpers which carry the
//! inline-cache hit paths. Frame-state loads (`LoadCallee`, `LoadNewTarget`)
//! are simple register writes; only `LoadThis` can throw.

use lyng_env::ThisState;
use lyng_ops::errors;
use lyng_types::Value;

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::error::VmError;
use crate::vm::Vm;
use crate::vm::dispatch_state::DispatchState;

// =====================================================================
// Shared operand shapes
// =====================================================================

/// Operands for Abx-encoded names opcodes. `bx` is an atom constant-pool
/// index. `feedback_slot` is populated only for IC-bearing globals.
pub struct OpAtomArgs {
    pub a: u16,
    pub bx: u32,
    pub instruction_len: u32,
    pub feedback_slot: Option<lyng_types::FeedbackSlotId>,
}

/// Operands for captured-name opcodes. For `CaptureName`, `bx` is an
/// atom constant-pool index and `a` is the reference register. For the
/// other three, `bx` is a captured-name reference register index.
pub struct OpCapturedNameArgs {
    pub a: u16,
    pub bx: u32,
    pub instruction_len: u32,
}

// =====================================================================
// Globals with feedback — LoadGlobal / StoreGlobal / AssignGlobal
// =====================================================================

pub fn op_load_global_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let atom = match inner.vm.read_atom_constant(code, args.bx) {
        Ok(atom) => atom,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let view = inner.frame_view();
    let load_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.load_global_with_feedback(
            agent,
            *host,
            &mut **registry,
            view,
            atom,
            code,
            args.feedback_slot,
        )
    };
    let handled = inner.handle_dispatch_result(load_result);
    let value = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.registers();
    inner.vm.write_register_unchecked(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

#[inline]
fn op_store_or_assign_global_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
    assign: bool,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let atom = match inner.vm.read_atom_constant(code, args.bx) {
        Ok(atom) => atom,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let value = inner.vm.read_register_unchecked(inner.registers(), args.a);
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        if assign {
            vm.assign_global_with_feedback(
                agent,
                *host,
                &mut **registry,
                view,
                atom,
                value,
                code,
                args.feedback_slot,
            )
        } else {
            vm.store_global_with_feedback(
                agent,
                *host,
                &mut **registry,
                view,
                atom,
                value,
                code,
                args.feedback_slot,
            )
        }
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

pub fn op_store_global_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    op_store_or_assign_global_semantic(state, args, false)
}

pub fn op_assign_global_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    op_store_or_assign_global_semantic(state, args, true)
}

pub fn op_delete_global_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let atom = match inner.vm.read_atom_constant(code, args.bx) {
        Ok(atom) => atom,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let view = inner.frame_view();
    let delete_result = {
        let DispatchState { agent, vm, .. } = &mut *inner;
        vm.delete_global(agent, view, atom)
    };
    let handled = inner.handle_dispatch_result(delete_result);
    let deleted = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.registers();
    inner
        .vm
        .write_register_unchecked(registers, args.a, Value::from_bool(deleted));
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// Names (lexical scope walk) — LoadName / ResolveName / ResolveGlobal /
// AssignName / AssignVariableName / DeleteName
// =====================================================================

pub fn op_load_name_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let atom = match inner.vm.read_atom_constant(code, args.bx) {
        Ok(atom) => atom,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let view = inner.frame_view();
    let load_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.load_name_with_context(agent, *host, &mut **registry, view, atom)
    };
    let handled = inner.handle_dispatch_result(load_result);
    let value = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.registers();
    inner.vm.write_register_unchecked(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub fn op_resolve_name_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let atom = match inner.vm.read_atom_constant(code, args.bx) {
        Ok(atom) => atom,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let view = inner.frame_view();
    let resolve_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.resolve_name_with_context(agent, *host, &mut **registry, view, atom)
    };
    let handled = inner.handle_dispatch_result(resolve_result);
    let value = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.registers();
    inner.vm.write_register_unchecked(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub fn op_resolve_global_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let atom = match inner.vm.read_atom_constant(code, args.bx) {
        Ok(atom) => atom,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let view = inner.frame_view();
    let resolve_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.resolve_global(agent, *host, &mut **registry, view, atom)
    };
    let handled = inner.handle_dispatch_result(resolve_result);
    let value = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.registers();
    inner.vm.write_register_unchecked(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

#[inline]
fn op_assign_name_common_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
    variable_form: bool,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let atom = match inner.vm.read_atom_constant(code, args.bx) {
        Ok(atom) => atom,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let value = inner.vm.read_register_unchecked(inner.registers(), args.a);
    let view = inner.frame_view();
    let assign_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        if variable_form {
            vm.assign_variable_name_with_context(agent, *host, &mut **registry, view, atom, value)
        } else {
            vm.assign_name_with_context(agent, *host, &mut **registry, view, atom, value)
        }
    };
    let handled = inner.handle_dispatch_result(assign_result);
    match handled {
        Ok(Some(())) => SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        },
        Ok(None) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

pub fn op_assign_name_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    op_assign_name_common_semantic(state, args, false)
}

pub fn op_assign_variable_name_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    op_assign_name_common_semantic(state, args, true)
}

pub fn op_delete_name_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let atom = match inner.vm.read_atom_constant(code, args.bx) {
        Ok(atom) => atom,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let view = inner.frame_view();
    let delete_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.delete_name_with_context(agent, *host, &mut **registry, view, atom)
    };
    let handled = inner.handle_dispatch_result(delete_result);
    let deleted = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.registers();
    inner
        .vm
        .write_register_unchecked(registers, args.a, Value::from_bool(deleted));
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// Captured names — CaptureName / LoadCapturedName /
// LoadCapturedNameThis / AssignCapturedName
// =====================================================================

pub fn op_capture_name_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let atom = match inner.vm.read_atom_constant(code, args.bx) {
        Ok(atom) => atom,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let view = inner.frame_view();
    let capture_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.capture_name_with_context(agent, *host, &mut **registry, view, args.a, atom)
    };
    let handled = inner.handle_dispatch_result(capture_result);
    match handled {
        Ok(Some(())) => SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        },
        Ok(None) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

/// Convert the `bx` field of a captured-name opcode into a `u16` register index.
#[inline]
fn captured_name_register(inner: &DispatchState<'_>, bx: u32) -> Result<u16, SemanticOutcome> {
    u16::try_from(bx).map_err(|_| SemanticOutcome::ExitError {
        error: VmError::RegisterOutOfBounds {
            code: inner.code(),
            register: u16::MAX,
        },
    })
}

pub fn op_load_captured_name_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCapturedNameArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let reference_register = match captured_name_register(inner, args.bx) {
        Ok(r) => r,
        Err(outcome) => return outcome,
    };
    let view = inner.frame_view();
    let load_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.load_captured_name_with_context(agent, *host, &mut **registry, view, reference_register)
    };
    let handled = inner.handle_dispatch_result(load_result);
    let value = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.registers();
    inner.vm.write_register_unchecked(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub fn op_load_captured_name_this_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCapturedNameArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let reference_register = match captured_name_register(inner, args.bx) {
        Ok(r) => r,
        Err(outcome) => return outcome,
    };
    let load_result = inner
        .vm
        .load_captured_name_this_with_context(inner.frame_view(), reference_register);
    let handled = inner.handle_dispatch_result(load_result);
    let value = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.registers();
    inner.vm.write_register_unchecked(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub fn op_assign_captured_name_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCapturedNameArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let reference_register = match captured_name_register(inner, args.bx) {
        Ok(r) => r,
        Err(outcome) => return outcome,
    };
    let value = inner.vm.read_register_unchecked(inner.registers(), args.a);
    let view = inner.frame_view();
    let assign_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.assign_captured_name_with_context(
            agent,
            *host,
            &mut **registry,
            view,
            reference_register,
            value,
        )
    };
    let handled = inner.handle_dispatch_result(assign_result);
    match handled {
        Ok(Some(())) => SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        },
        Ok(None) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

// =====================================================================
// Frame-state loads — LoadThis / LoadCallee / LoadNewTarget
// =====================================================================

pub fn op_load_this_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    // `this_state` (mutated by super() init) and `lexical_env` (mutated by `with`)
    // are live in the frame overlay; read them there, not from a stale snapshot.
    let cfr = inner.cfr;
    let this_state = inner.vm.frame_header(cfr).this_state();
    let this_value = inner.vm.frame_header(cfr).this_value();
    let lexical_env = inner.vm.frame_header(cfr).lexical_env();
    let load_this = {
        let DispatchState { agent, .. } = &mut *inner;
        match this_state {
            ThisState::Value(value) => Ok(value),
            ThisState::Uninitialized => Err(VmError::Abrupt(errors::throw_reference_error(agent))),
            ThisState::Lexical => Vm::resolve_this_binding(agent, lexical_env, this_value),
        }
    };
    let handled = inner.handle_dispatch_result(load_this);
    let value = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.registers();
    inner.vm.write_register_unchecked(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub fn op_load_callee_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner
        .vm
        .frame_header(inner.cfr)
        .callee()
        .map_or(Value::undefined(), Value::from_object_ref);
    let registers = inner.registers();
    inner.vm.write_register_unchecked(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub fn op_load_new_target_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner
        .vm
        .frame_header(inner.cfr)
        .new_target()
        .map_or(Value::undefined(), Value::from_object_ref);
    let registers = inner.registers();
    inner.vm.write_register_unchecked(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

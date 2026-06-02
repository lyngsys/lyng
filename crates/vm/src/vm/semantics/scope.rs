//! Scope family semantic bodies.
//!
//! Family coverage (10 opcodes):
//! - Environment slot access (Abx): `LoadEnvSlot`, `StoreEnvSlot`,
//!   `AssignEnvSlot`.
//! - Block-scope binding chunks (Abx): `EnterEnvScope`, `LeaveEnvScope`.
//! - Loop-iteration environment chain (Ax): `PushClosureEnv`,
//!   `PopClosureEnv`.
//! - `with`-statement environment chain (Ax): `PushWithEnv`, `PopWithEnv`.
//! - Unary `typeof` (Ax): `TypeOf` — reads and writes the same register.

use lyng_bytecode::Opcode;

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::error::VmError;
use crate::vm::Vm;
use crate::vm::dispatch_state::DispatchState;
use crate::vm::values::decode_env_operand;

// =====================================================================
// Shared operand shapes
// =====================================================================

/// Operands for Abx-encoded scope opcodes. For slot-access opcodes, `bx`
/// encodes `(depth, slot)` via `decode_env_operand`. For `EnterEnvScope` /
/// `LeaveEnvScope`, `a` is the base register and `bx` is the binding-chunk count.
pub struct OpScopeAbxArgs {
    pub a: u16,
    pub bx: u32,
    pub instruction_len: u32,
}

/// Operands for Ax-encoded scope opcodes. For `PushClosureEnv`, `ax > 0`
/// is a mirrored-slot index (1-based). For `PushWithEnv` / `TypeOf`, `ax`
/// is a register index. For Pop* opcodes, `ax` is unused.
pub struct OpScopeAxArgs {
    pub ax: i32,
    pub instruction_len: u32,
}

// =====================================================================
// LoadEnvSlot / StoreEnvSlot / AssignEnvSlot — Abx; bx encodes
// (depth, slot).
// =====================================================================

pub fn op_load_env_slot_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let (depth, slot) = decode_env_operand(args.bx);
    let lexical_env = inner.vm.frame_header(inner.cfr).lexical_env();
    let environment = {
        let DispatchState { vm, agent, .. } = &mut *inner;
        match vm.environment_for_slot_access(agent, lexical_env, depth, slot) {
            Ok(env) => env,
            Err(error) => return SemanticOutcome::ExitError { error },
        }
    };
    let load_result = {
        let DispatchState { agent, .. } = &mut *inner;
        Vm::read_environment_slot(agent, environment, slot)
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

pub fn op_store_env_slot_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let (depth, slot) = decode_env_operand(args.bx);
    let lexical_env = inner.vm.frame_header(inner.cfr).lexical_env();
    let environment = {
        let DispatchState { vm, agent, .. } = &mut *inner;
        match vm.environment_for_slot_access(agent, lexical_env, depth, slot) {
            Ok(env) => env,
            Err(error) => return SemanticOutcome::ExitError { error },
        }
    };
    let value = inner.vm.read_register_unchecked(inner.registers(), args.a);
    let store_result = {
        let DispatchState { vm, agent, .. } = &mut *inner;
        vm.write_environment_slot(agent, environment, slot, value)
    };
    let handled = inner.handle_dispatch_result(store_result);
    match handled {
        Ok(Some(())) => SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        },
        Ok(None) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

pub fn op_assign_env_slot_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let (depth, slot) = decode_env_operand(args.bx);
    let lexical_env = inner.vm.frame_header(inner.cfr).lexical_env();
    let environment = {
        let DispatchState { vm, agent, .. } = &mut *inner;
        match vm.environment_for_slot_access(agent, lexical_env, depth, slot) {
            Ok(env) => env,
            Err(error) => return SemanticOutcome::ExitError { error },
        }
    };
    let value = inner.vm.read_register_unchecked(inner.registers(), args.a);
    let strict = inner.vm.frame_is_strict(inner.frame_view());
    let assign_result = {
        let DispatchState { vm, agent, .. } = &mut *inner;
        vm.assign_environment_slot(agent, environment, slot, value, strict)
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
// EnterEnvScope / LeaveEnvScope — block-scope binding chunks.
// =====================================================================

pub fn op_enter_env_scope_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let enter_result = {
        let DispatchState { vm, agent, .. } = &mut *inner;
        vm.enter_env_scope(agent, view, args.a, args.bx)
    };
    if let Err(error) = enter_result {
        return SemanticOutcome::ExitError { error };
    }
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub fn op_leave_env_scope_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    inner
        .vm
        .leave_env_scope(inner.frame_view(), args.a, args.bx);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// PushClosureEnv / PopClosureEnv — loop-iteration environment chain.
// =====================================================================

pub fn op_push_closure_env_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let site = inner
        .installed
        .loop_iteration_environment_site(inner.pc())
        .cloned();
    let mirrored_slot = if args.ax > 0 {
        match u32::try_from(args.ax - 1) {
            Ok(v) => Some(v),
            Err(_) => {
                return SemanticOutcome::ExitError {
                    error: VmError::UnsupportedOpcode {
                        code: inner.code(),
                        instruction_offset: inner.pc(),
                        opcode: Opcode::PushClosureEnv,
                    },
                };
            }
        }
    } else {
        None
    };
    let view = inner.frame_view();
    let push_result = {
        let DispatchState { vm, agent, .. } = &mut *inner;
        vm.push_loop_iteration_environment(agent, view, site, mirrored_slot)
    };
    if let Err(error) = push_result {
        return SemanticOutcome::ExitError { error };
    }
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub fn op_pop_closure_env_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    inner.vm.pop_loop_iteration_environment();
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// PushWithEnv / PopWithEnv — `with`-statement environment chain.
// =====================================================================

pub fn op_push_with_env_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let Ok(register) = u16::try_from(args.ax) else {
        return SemanticOutcome::ExitError {
            error: VmError::RegisterOutOfBounds {
                code: inner.code(),
                register: 0,
            },
        };
    };
    let value = inner
        .vm
        .read_register_unchecked(inner.registers(), register);
    let push_result = {
        let DispatchState { vm, agent, .. } = &mut *inner;
        vm.push_with_environment(agent, value)
    };
    let handled = inner.handle_dispatch_result(push_result);
    match handled {
        Ok(Some(())) => SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        },
        Ok(None) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

pub fn op_pop_with_env_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    inner.vm.pop_with_environment();
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// TypeOf
// =====================================================================

pub fn op_type_of_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let Ok(register) = u16::try_from(args.ax) else {
        return SemanticOutcome::ExitError {
            error: VmError::RegisterOutOfBounds {
                code: inner.code(),
                register: 0,
            },
        };
    };
    let registers = inner.registers();
    let value = inner.vm.read_register_unchecked(registers, register);
    let type_string = {
        let DispatchState { agent, .. } = &mut *inner;
        Vm::type_of_value(agent, value)
    };
    let registers = inner.registers();
    inner
        .vm
        .write_register_unchecked(registers, register, type_string);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

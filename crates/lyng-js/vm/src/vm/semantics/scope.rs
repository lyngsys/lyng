//! Scope family semantic bodies (DSL-0a Task A13).
//!
//! Each `op_xxx_semantic` function implements the semantic effect of one
//! scope-family opcode. The α handler in `dispatch_handlers/scope.rs`
//! decodes operands, constructs `OpXxxArgs`, calls the semantic body, and
//! translates the returned `SemanticOutcome` to `Step` via
//! `translate_outcome_to_step`. The DSL-0b cold-stub shim in
//! `dsl/handlers/cold/scope.rs` will reach the same functions from the
//! asm-DSL path.
//!
//! Family coverage (10 opcodes):
//! - Environment slot access (Abx): `LoadEnvSlot`, `StoreEnvSlot`,
//!   `AssignEnvSlot`.
//! - Block-scope binding chunks (Abx): `EnterEnvScope`, `LeaveEnvScope`.
//! - Loop-iteration environment chain (Ax): `PushClosureEnv`,
//!   `PopClosureEnv`.
//! - `with`-statement environment chain (Ax): `PushWithEnv`, `PopWithEnv`.
//! - Unary `typeof` (Ax): `TypeOf` — reads and writes the same register.
//!
//! ### PC-advance convention
//!
//! Each helper routes its `VmResult<…>` (when applicable) through
//! `DispatchState::handle_dispatch_result`. On success the semantic body
//! mutates the environment chain / register and returns
//! `Continue { pc_advance: instruction_len }`. On a caught abrupt
//! completion the body returns `Continue { pc_advance: 0 }` so the
//! trampoline runs the new (catch-target) PC's opcode next — the epoch
//! bump triggers a frame refresh on the next iteration. On an escaping
//! abrupt completion the body returns `ExitError`.
//!
//! ### Operand timing
//!
//! `EnterEnvScope` / `LeaveEnvScope` mutate the active environment chain
//! before `state.advance(instruction_len)` in the α body. The semantic
//! body preserves this ordering: the mutation happens before the
//! `Continue { pc_advance: instruction_len }` return, which is the
//! single point at which `translate_outcome_to_step` advances PC.

use lyng_js_bytecode::Opcode;

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::error::VmError;
use crate::vm::dispatch_state::DispatchState;
use crate::vm::values::decode_env_operand;
use crate::vm::Vm;

// =====================================================================
// Shared operand shapes
// =====================================================================

/// Operands for the Abx-encoded scope opcodes: `LoadEnvSlot`,
/// `StoreEnvSlot`, `AssignEnvSlot`, `EnterEnvScope`, `LeaveEnvScope`.
///
/// For slot-access opcodes, `bx` encodes `(depth, slot)` via
/// `decode_env_operand`. For `EnterEnvScope` / `LeaveEnvScope`, `a` is the
/// base register and `bx` is the binding-chunk count.
pub struct OpScopeAbxArgs {
    pub a: u16,
    pub bx: u32,
    pub instruction_len: u32,
}

/// Operands for the Ax-encoded scope opcodes: `PushClosureEnv`,
/// `PopClosureEnv`, `PushWithEnv`, `PopWithEnv`, `TypeOf`.
///
/// For `PushClosureEnv`, `ax > 0` denotes a mirrored-slot index
/// (decremented and converted to `u32`); `ax == 0` means no mirrored slot.
/// For `PushWithEnv` and `TypeOf`, `ax` is a register index that the
/// semantic body bounds-checks to `u16`. For `PopClosureEnv` and
/// `PopWithEnv`, `ax` is unused.
pub struct OpScopeAxArgs {
    pub ax: i32,
    pub instruction_len: u32,
}

// =====================================================================
// LoadEnvSlot / StoreEnvSlot / AssignEnvSlot — Abx; bx encodes
// (depth, slot).
// =====================================================================

pub(crate) fn op_load_env_slot_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let (depth, slot) = decode_env_operand(args.bx);
    let lexical_env = inner.frame.lexical_env();
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
    let registers = inner.frame.registers();
    inner.vm.write_register_unchecked(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub(crate) fn op_store_env_slot_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let (depth, slot) = decode_env_operand(args.bx);
    let lexical_env = inner.frame.lexical_env();
    let environment = {
        let DispatchState { vm, agent, .. } = &mut *inner;
        match vm.environment_for_slot_access(agent, lexical_env, depth, slot) {
            Ok(env) => env,
            Err(error) => return SemanticOutcome::ExitError { error },
        }
    };
    let value = inner
        .vm
        .read_register_unchecked(inner.frame.registers(), args.a);
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

pub(crate) fn op_assign_env_slot_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let (depth, slot) = decode_env_operand(args.bx);
    let lexical_env = inner.frame.lexical_env();
    let environment = {
        let DispatchState { vm, agent, .. } = &mut *inner;
        match vm.environment_for_slot_access(agent, lexical_env, depth, slot) {
            Ok(env) => env,
            Err(error) => return SemanticOutcome::ExitError { error },
        }
    };
    let value = inner
        .vm
        .read_register_unchecked(inner.frame.registers(), args.a);
    let strict = inner.vm.frame_is_strict(&inner.frame);
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
//
// Both mutate the active environment chain. `EnterEnvScope` can fail
// (`VmResult<()>`); `LeaveEnvScope` is infallible. Both advance by
// `instruction_len` after the mutation, matching the α handler ordering.
// =====================================================================

pub(crate) fn op_enter_env_scope_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let enter_result = {
        let DispatchState {
            vm, agent, frame, ..
        } = &mut *inner;
        vm.enter_env_scope(agent, frame, args.a, args.bx)
    };
    if let Err(error) = enter_result {
        return SemanticOutcome::ExitError { error };
    }
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub(crate) fn op_leave_env_scope_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    inner.vm.leave_env_scope(&inner.frame, args.a, args.bx);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// PushClosureEnv / PopClosureEnv — loop-iteration environment chain.
//
// `PushClosureEnv` reads the loop-iteration site from the installed code
// and (optionally) a mirrored-slot index from `ax - 1`. `PopClosureEnv`
// pops the topmost loop-iteration environment without operands.
// =====================================================================

pub(crate) fn op_push_closure_env_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let site = inner
        .installed
        .loop_iteration_environment_site(inner.frame.instruction_offset())
        .cloned();
    let mirrored_slot = if args.ax > 0 {
        match u32::try_from(args.ax - 1) {
            Ok(v) => Some(v),
            Err(_) => {
                return SemanticOutcome::ExitError {
                    error: VmError::UnsupportedOpcode {
                        code: inner.frame.code(),
                        instruction_offset: inner.frame.instruction_offset(),
                        opcode: Opcode::PushClosureEnv,
                    },
                };
            }
        }
    } else {
        None
    };
    let push_result = {
        let DispatchState {
            vm, agent, frame, ..
        } = &mut *inner;
        vm.push_loop_iteration_environment(agent, frame, site, mirrored_slot)
    };
    if let Err(error) = push_result {
        return SemanticOutcome::ExitError { error };
    }
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub(crate) fn op_pop_closure_env_semantic(
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
//
// `PushWithEnv` reads the operand register and pushes a with-environment
// scoped to that value (which `Vm::push_with_environment` coerces to an
// object, throwing on `null` / `undefined`). `PopWithEnv` pops the
// topmost with-environment without operands.
// =====================================================================

pub(crate) fn op_push_with_env_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let register = match u16::try_from(args.ax) {
        Ok(r) => r,
        Err(_) => {
            return SemanticOutcome::ExitError {
                error: VmError::RegisterOutOfBounds {
                    code: inner.frame.code(),
                    register: 0,
                },
            };
        }
    };
    let value = inner
        .vm
        .read_register_unchecked(inner.frame.registers(), register);
    let push_result = {
        let DispatchState {
            vm, agent, frame, ..
        } = &mut *inner;
        vm.push_with_environment(agent, frame, value)
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

pub(crate) fn op_pop_with_env_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    inner.vm.pop_with_environment(&mut inner.frame);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// TypeOf — Ax form. Reads the operand register, computes the typeof
// string, writes the result back to the same register.
// =====================================================================

pub(crate) fn op_type_of_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpScopeAxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let register = match u16::try_from(args.ax) {
        Ok(r) => r,
        Err(_) => {
            return SemanticOutcome::ExitError {
                error: VmError::RegisterOutOfBounds {
                    code: inner.frame.code(),
                    register: 0,
                },
            };
        }
    };
    let registers = inner.frame.registers();
    let value = inner.vm.read_register_unchecked(registers, register);
    let type_string = {
        let DispatchState { agent, .. } = &mut *inner;
        Vm::type_of_value(agent, value)
    };
    let registers = inner.frame.registers();
    inner
        .vm
        .write_register_unchecked(registers, register, type_string);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

//! Names family semantic bodies (DSL-0a Task A12).
//!
//! Each `op_xxx_semantic` function implements the semantic effect of one
//! names-family opcode. The α handler in `dispatch_handlers/names.rs`
//! decodes operands, constructs `OpXxxArgs`, calls the semantic body, and
//! translates the returned `SemanticOutcome` to `Step` via
//! `translate_outcome_to_step`. The DSL-0b cold-stub shim in
//! `dsl/handlers/cold/names.rs` will reach the same functions from the
//! asm-DSL path.
//!
//! Family coverage (17 opcodes):
//! - Globals with feedback: `LoadGlobal`, `StoreGlobal`, `AssignGlobal`.
//! - Globals without feedback: `DeleteGlobal`.
//! - Names (lexical scope walk): `LoadName`, `ResolveName`, `ResolveGlobal`,
//!   `AssignName`, `AssignVariableName`, `DeleteName`.
//! - Captured names (closures): `CaptureName`, `LoadCapturedName`,
//!   `LoadCapturedNameThis`, `AssignCapturedName`.
//! - Frame-state loads: `LoadThis`, `LoadCallee`, `LoadNewTarget`.
//!
//! ### IC layout preservation
//!
//! The globals-with-feedback opcodes (`LoadGlobal`, `StoreGlobal`,
//! `AssignGlobal`) defer entirely to the existing
//! `Vm::*_with_feedback` helpers in `vm/names.rs`. Those helpers carry the
//! Phase 3 inline-cache cache hit paths (monomorphic global-binding lookup,
//! shape probe, feedback-slot recording). DSL-0a's job is only to lift
//! the call site out of the α handler — DSL-1 lands the IC mode-byte
//! refactor and DSL-0b the flat-array refactor (per design §10). No IC
//! layout changes here.
//!
//! ### PC-advance convention
//!
//! Every helper routes its `VmResult<…>` through
//! `DispatchState::handle_dispatch_result`. On success the semantic body
//! writes the destination register (when applicable) and advances by
//! `instruction_len`; on a caught abrupt completion PC has already been
//! rewritten to the catch target so the body returns
//! `Continue { pc_advance: 0 }`. This mirrors the α-side pattern
//! `try_step!(state.handle_dispatch_result(...)); state.advance(...);
//! dispatch_next!(state);`.
//!
//! The frame-state loads (`LoadThis`, `LoadCallee`, `LoadNewTarget`) are
//! simple register writes; only `LoadThis` can throw (uninitialized
//! lexical `this`), so the others bypass `handle_dispatch_result` entirely.

use lyng_env::ThisState;
use lyng_ops::errors;
use lyng_types::Value;

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::error::VmError;
use crate::vm::dispatch_state::DispatchState;
use crate::vm::Vm;

// =====================================================================
// Shared operand shapes
// =====================================================================

/// Operands for Abx-encoded names opcodes that carry an atom-constant-pool
/// index in `bx` and (optionally) a feedback slot. The semantic body reads
/// the atom from the constant pool itself; the α handler only forwards the
/// raw `bx` index.
///
/// Used by all globals, names (scope-walk), and frame-state load opcodes
/// in this family. The `feedback_slot` is populated only for the
/// IC-bearing globals (`LoadGlobal`, `StoreGlobal`, `AssignGlobal`); other
/// callers pass `None` from the α handler.
pub struct OpAtomArgs {
    pub a: u16,
    pub bx: u32,
    pub instruction_len: u32,
    pub feedback_slot: Option<lyng_types::FeedbackSlotId>,
}

/// Operands for the captured-name opcodes (`CaptureName`,
/// `LoadCapturedName`, `LoadCapturedNameThis`, `AssignCapturedName`).
/// `bx` is interpreted as a captured-name reference register index; the
/// semantic body bounds-checks it down to `u16`. `CaptureName` also reads
/// the atom-constant from `bx` *and* uses `a` as the reference register
/// — see its semantic body for the two-step decode.
///
/// `CaptureName` carries an `atom_bx` to read from the constant pool;
/// the other three opcodes only need the `bx` value for the register
/// bounds check.
pub struct OpCapturedNameArgs {
    pub a: u16,
    pub bx: u32,
    pub instruction_len: u32,
}

// =====================================================================
// Globals with feedback — LoadGlobal / StoreGlobal / AssignGlobal
// =====================================================================

pub(crate) fn op_load_global_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let atom = match inner.vm.read_atom_constant(code, args.bx) {
        Ok(atom) => atom,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let load_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *inner;
        vm.load_global_with_feedback(
            agent,
            *host,
            &mut **registry,
            frame,
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
    let registers = inner.frame.registers();
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
    let value = inner
        .vm
        .read_register_unchecked(inner.frame.registers(), args.a);
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *inner;
        if assign {
            vm.assign_global_with_feedback(
                agent,
                *host,
                &mut **registry,
                frame,
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
                frame,
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

pub(crate) fn op_store_global_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    op_store_or_assign_global_semantic(state, args, false)
}

pub(crate) fn op_assign_global_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    op_store_or_assign_global_semantic(state, args, true)
}

pub(crate) fn op_delete_global_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let atom = match inner.vm.read_atom_constant(code, args.bx) {
        Ok(atom) => atom,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let delete_result = {
        let DispatchState { agent, frame, .. } = &mut *inner;
        Vm::delete_global(agent, frame, atom)
    };
    let handled = inner.handle_dispatch_result(delete_result);
    let deleted = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.frame.registers();
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

pub(crate) fn op_load_name_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let atom = match inner.vm.read_atom_constant(code, args.bx) {
        Ok(atom) => atom,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let load_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *inner;
        vm.load_name_with_context(agent, *host, &mut **registry, frame, atom)
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

pub(crate) fn op_resolve_name_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let atom = match inner.vm.read_atom_constant(code, args.bx) {
        Ok(atom) => atom,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let resolve_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *inner;
        vm.resolve_name_with_context(agent, *host, &mut **registry, frame, atom)
    };
    let handled = inner.handle_dispatch_result(resolve_result);
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

pub(crate) fn op_resolve_global_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let atom = match inner.vm.read_atom_constant(code, args.bx) {
        Ok(atom) => atom,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let resolve_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *inner;
        vm.resolve_global(agent, *host, &mut **registry, frame, atom)
    };
    let handled = inner.handle_dispatch_result(resolve_result);
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
    let value = inner
        .vm
        .read_register_unchecked(inner.frame.registers(), args.a);
    let assign_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *inner;
        if variable_form {
            vm.assign_variable_name_with_context(agent, *host, &mut **registry, frame, atom, value)
        } else {
            vm.assign_name_with_context(agent, *host, &mut **registry, frame, atom, value)
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

pub(crate) fn op_assign_name_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    op_assign_name_common_semantic(state, args, false)
}

pub(crate) fn op_assign_variable_name_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    op_assign_name_common_semantic(state, args, true)
}

pub(crate) fn op_delete_name_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let atom = match inner.vm.read_atom_constant(code, args.bx) {
        Ok(atom) => atom,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let delete_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *inner;
        vm.delete_name_with_context(agent, *host, &mut **registry, frame, atom)
    };
    let handled = inner.handle_dispatch_result(delete_result);
    let deleted = match handled {
        Ok(Some(v)) => v,
        Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.frame.registers();
    inner
        .vm
        .write_register_unchecked(registers, args.a, Value::from_bool(deleted));
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// Captured names (closures) — CaptureName / LoadCapturedName /
// LoadCapturedNameThis / AssignCapturedName
//
// `CaptureName` follows the same atom-from-`bx` decode as the names
// scope-walk opcodes (the captured-name reference is allocated by the
// helper). The other three interpret `bx` as a captured-name reference
// register index; the bounds check matches the α handler's
// `captured_name_register` helper.
// =====================================================================

pub(crate) fn op_capture_name_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let atom = match inner.vm.read_atom_constant(code, args.bx) {
        Ok(atom) => atom,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let capture_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *inner;
        vm.capture_name_with_context(agent, *host, &mut **registry, frame, args.a, atom)
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

/// Convert the `bx` field of a captured-name opcode into a `u16` register
/// index. Mirrors the α handler's `captured_name_register` helper.
#[inline]
fn captured_name_register(inner: &DispatchState<'_>, bx: u32) -> Result<u16, SemanticOutcome> {
    u16::try_from(bx).map_err(|_| SemanticOutcome::ExitError {
        error: VmError::RegisterOutOfBounds {
            code: inner.frame.code(),
            register: u16::MAX,
        },
    })
}

pub(crate) fn op_load_captured_name_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCapturedNameArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let reference_register = match captured_name_register(inner, args.bx) {
        Ok(r) => r,
        Err(outcome) => return outcome,
    };
    let load_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *inner;
        vm.load_captured_name_with_context(agent, *host, &mut **registry, frame, reference_register)
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

pub(crate) fn op_load_captured_name_this_semantic(
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
        .load_captured_name_this_with_context(&inner.frame, reference_register);
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

pub(crate) fn op_assign_captured_name_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpCapturedNameArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let reference_register = match captured_name_register(inner, args.bx) {
        Ok(r) => r,
        Err(outcome) => return outcome,
    };
    let value = inner
        .vm
        .read_register_unchecked(inner.frame.registers(), args.a);
    let assign_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            ..
        } = &mut *inner;
        vm.assign_captured_name_with_context(
            agent,
            *host,
            &mut **registry,
            frame,
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
//
// `LoadThis` resolves the current execution context's `this_state`. The
// `Uninitialized` arm throws a ReferenceError; `Lexical` walks the
// lexical environment via `Vm::resolve_this_binding`. `LoadCallee` and
// `LoadNewTarget` are pure frame reads — no atom, no feedback, no
// exception path.
// =====================================================================

pub(crate) fn op_load_this_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let load_this = {
        let DispatchState { agent, frame, .. } = &mut *inner;
        let this_state = agent.current_execution_context().map_or_else(
            || ThisState::Value(frame.this_value()),
            |ec| ec.this_state(),
        );
        match this_state {
            ThisState::Value(value) => Ok(value),
            ThisState::Uninitialized => Err(VmError::Abrupt(errors::throw_reference_error(agent))),
            ThisState::Lexical => Vm::resolve_this_binding(agent, frame.lexical_env(), frame),
        }
    };
    let handled = inner.handle_dispatch_result(load_this);
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

pub(crate) fn op_load_callee_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner
        .frame
        .callee()
        .map_or(Value::undefined(), Value::from_object_ref);
    let registers = inner.frame.registers();
    inner.vm.write_register_unchecked(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub(crate) fn op_load_new_target_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpAtomArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner
        .frame
        .new_target()
        .map_or(Value::undefined(), Value::from_object_ref);
    let registers = inner.frame.registers();
    inner.vm.write_register_unchecked(registers, args.a, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

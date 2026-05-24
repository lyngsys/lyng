//! Property family semantic bodies (DSL-0a Task A11).
//!
//! Each `op_xxx_semantic` function implements the semantic effect of one
//! property-family opcode. The α handler in `dispatch_handlers/property.rs`
//! decodes operands, constructs `OpXxxArgs`, calls the semantic body, and
//! translates the returned `SemanticOutcome` to `Step`.
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
//! ### IC layout preservation
//!
//! The IC-heavy opcodes (`GetNamedProperty`, `SetNamedProperty`,
//! `GetKeyedProperty`, `SetKeyedProperty`) defer entirely to the existing
//! `Vm::execute_*_opcode` helpers in `vm/dispatch/property.rs`. Those
//! helpers carry the Phase 3a/3e/3f inline-cache cache hit paths (monomorphic
//! handler load, polymorphic shape probe, megamorphic table), the
//! `ToObject` coercion, the prototype-chain walk, and the feedback-slot
//! recording. DSL-0a's job is only to lift the call site out of the α
//! handler — DSL-1 lands the IC mode-byte refactor and DSL-0b the
//! flat-array refactor (per design §10). No IC layout changes here.
//!
//! ### PC-advance convention
//!
//! Every `execute_*_opcode` helper advances `frame.instruction_offset()`
//! itself on success via `advance_dispatch_frame(frame, instruction_len)`,
//! and routes abrupt completions through `handle_dispatch_result` which
//! either rewrites PC to the catch handler (caught) or returns
//! `VmError::Abrupt` (escapes). Either way, on success the next opcode
//! byte sits at the current PC — so the success semantic returns
//! `SemanticOutcome::Continue { pc_advance: 0 }`. This mirrors the α-side
//! pattern `try_step!(result); dispatch_next!(state);`.
//!
//! The exceptions are:
//! - `CreateObject` / `CreateArray`: the helper allocates and returns the
//!   `ObjectRef`; the semantic body writes the register and explicitly
//!   advances by `instruction_len` (mirroring the α handler).
//! - `SetFunctionName` / `CheckObjectCoercible` / `ThrowIfUninitialized`:
//!   these route through `handle_dispatch_result` directly (the helper
//!   shape doesn't include `advance_dispatch_frame`). Success →
//!   `Continue { pc_advance: instruction_len }`; caught →
//!   `Continue { pc_advance: 0 }`; escape → `ExitError`. Mirrors the
//!   arithmetic unary pattern (`op_negate_semantic`).

use lyng_bytecode::Opcode;
use lyng_ops::errors;
use lyng_types::{FeedbackSlotId, Value};

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::error::VmError;
use crate::vm::dispatch_state::DispatchState;
use crate::vm::Vm;

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

// =====================================================================
// Internal helper: route an `execute_*_opcode` helper's `VmResult<()>`.
// =====================================================================

/// Shared slow-path tail for every property opcode that delegates to a
/// `Vm::execute_*_opcode` helper.
///
/// The helper internally advances `frame.instruction_offset()` on success
/// and rewrites it to the catch target on a caught abrupt completion. In
/// both cases the next opcode byte is at the current PC, so the
/// `SemanticOutcome` carries `pc_advance: 0`.
///
/// This mirrors the α-side pattern `try_step!(result); dispatch_next!(state);`
/// — option (b) from the plan (keep helpers as `Vm`-side methods, route
/// the outcome through the wrapper).
#[inline]
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
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.execute_get_named_property_opcode(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.instruction_len,
            args.feedback_slot,
            args.a,
            args.b,
            args.c,
        )
    };
    route_execute_result(result)
}

// =====================================================================
// Named property writes — `SetNamedProperty`, `AssignNamedProperty`,
// `StrictAssignNamedProperty`.
//
// The three opcodes share the same Abc operand decode and slow-helper
// signature, differing only in the `semantic` Opcode threaded into the
// helper (strict-mode + assignment + property-define semantics fan out
// inside `execute_set_named_property_opcode`).
// =====================================================================

#[inline]
fn op_set_named_property_shared(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAccessArgs,
    semantic: Opcode,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.execute_set_named_property_opcode(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.instruction_len,
            args.feedback_slot,
            semantic,
            args.a,
            args.b,
            args.c,
        )
    };
    route_execute_result(result)
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
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.execute_get_keyed_property_opcode(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.instruction_len,
            args.feedback_slot,
            args.a,
            args.b,
            args.c,
        )
    };
    route_execute_result(result)
}

// =====================================================================
// Keyed property writes — `SetKeyedProperty`, `AssignKeyedProperty`,
// `StrictAssignKeyedProperty`. Same fan-out pattern as the named writes.
// =====================================================================

#[inline]
fn op_set_keyed_property_shared(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAccessArgs,
    semantic: Opcode,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.execute_set_keyed_property_opcode(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.instruction_len,
            args.feedback_slot,
            semantic,
            args.a,
            args.b,
            args.c,
        )
    };
    route_execute_result(result)
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
//
// The define-data variants do NOT carry a feedback slot (the α handler
// decodes with `is_profiled = false`); operands flow through the
// non-profiled `OpPropertyAbcArgs` shape.
// =====================================================================

pub fn op_define_named_property_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.execute_define_named_property_opcode(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.instruction_len,
            args.a,
            args.b,
            args.c,
        )
    };
    route_execute_result(result)
}

pub fn op_define_keyed_property_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.execute_define_keyed_property_opcode(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.instruction_len,
            args.a,
            args.b,
            args.c,
        )
    };
    route_execute_result(result)
}

// =====================================================================
// Object / array literal allocation — `CreateObject`, `CreateArray`.
//
// Both allocate via `Vm::create_object` / `Vm::create_array`, write the
// resulting `ObjectRef` to register `a`, and advance by
// `instruction_len`. `CreateArray` additionally defines the `length`
// property when the requested capacity is non-zero (mirrors the α
// handler precisely).
// =====================================================================

pub fn op_create_object_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let realm = inner.frame.realm();
    let object = {
        let DispatchState { agent, .. } = &mut *inner;
        match Vm::create_object(agent, realm, usize::try_from(args.bx).unwrap_or(usize::MAX)) {
            Ok(object) => object,
            Err(error) => return SemanticOutcome::ExitError { error },
        }
    };
    let registers = inner.frame.registers();
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
    let realm = inner.frame.realm();
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
    let registers = inner.frame.registers();
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
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.execute_store_dense_element_opcode(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.instruction_len,
            args.a,
            args.b,
            args.c,
        )
    };
    route_execute_result(result)
}

pub fn op_load_dense_element_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.execute_load_dense_element_opcode(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.instruction_len,
            args.a,
            args.b,
            args.c,
        )
    };
    route_execute_result(result)
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
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.execute_delete_property_opcode(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.instruction_len,
            args.a,
            args.b,
            args.c,
        )
    };
    route_execute_result(result)
}

pub fn op_in_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.execute_in_opcode(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.instruction_len,
            args.a,
            args.b,
            args.c,
        )
    };
    route_execute_result(result)
}

pub fn op_to_property_key_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.execute_to_property_key_opcode(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.instruction_len,
            args.a,
            args.b,
        )
    };
    route_execute_result(result)
}

pub fn op_copy_data_properties_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbcArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            frame,
            frame_depth,
            ..
        } = &mut *inner;
        vm.execute_copy_data_properties_opcode(
            agent,
            *host,
            &mut **registry,
            *frame_depth,
            frame,
            args.instruction_len,
            args.a,
            args.b,
            args.c,
        )
    };
    route_execute_result(result)
}

/// `SetFunctionName` does not go through an `execute_*_opcode` wrapper;
/// the α handler routes a `Vm::set_function_name` call through
/// `handle_dispatch_result` and advances PC explicitly. The semantic
/// body mirrors that exactly.
pub fn op_set_function_name_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let function = match inner.vm.object_register(&inner.frame, args.a) {
        Ok(object) => object,
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let name_value = inner
        .vm
        .read_register_unchecked(inner.frame.registers(), args.b);
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

/// `CheckObjectCoercible` — Abx-decoded; only `a` (the register holding
/// the candidate value) is used. The α handler reads the register, calls
/// `Vm::check_object_coercible`, and routes through `handle_dispatch_result`.
pub fn op_check_object_coercible_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner
        .vm
        .read_register_unchecked(inner.frame.registers(), args.a);
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

/// `ThrowIfUninitialized` — Abx-decoded; raises a `ReferenceError` when the
/// register holds the TDZ sentinel. The α handler reads, compares against
/// `Value::uninitialized_lexical()`, constructs the abrupt completion
/// inline, and routes through `handle_dispatch_result`. Non-TDZ values
/// fall through to the bare advance.
pub fn op_throw_if_uninitialized_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpPropertyAbxArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let value = inner
        .vm
        .read_register_unchecked(inner.frame.registers(), args.a);
    if value == Value::uninitialized_lexical() {
        let result: Result<(), VmError> = {
            let DispatchState { agent, .. } = &mut *inner;
            Err(VmError::Abrupt(errors::throw_reference_error(agent)))
        };
        let handled = inner.handle_dispatch_result(result);
        match handled {
            // The α `if try_step!(...).is_none() { dispatch_next!(state); }`
            // branch fires when the throw was caught — resume at the
            // already-rewritten PC.
            Ok(Some(())) => {
                // Unreachable in practice: `handle_dispatch_result` on an
                // Err input never returns `Some(())`. Treat as advance for
                // safety and to satisfy match exhaustiveness.
            }
            Ok(None) => return SemanticOutcome::Continue { pc_advance: 0 },
            Err(error) => return SemanticOutcome::ExitError { error },
        }
    }
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

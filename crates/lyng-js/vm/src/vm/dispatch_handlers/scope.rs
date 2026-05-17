//! Environment-scope handlers for the trampoline dispatch path
//! (lyng-59e6 round 1).
//!
//! Post-A13: each α handler in this file is a thin shim that
//!   1. decodes the instruction's operands,
//!   2. constructs `OpScopeAbxArgs` / `OpScopeAxArgs` and calls into
//!      `crate::vm::semantics::scope::op_xxx_semantic`,
//!   3. translates the returned `SemanticOutcome` to `Step` via
//!      `translate_outcome_to_step`.
//!
//! The α handler owns operand decode only; environment-chain mutation,
//! slot-walk routing through `Vm::environment_for_slot_access`, and
//! `handle_dispatch_result` routing live in the semantic body. The
//! existing helpers in `vm/with_env.rs`, `vm/loop_iteration.rs`, and
//! `vm/names.rs` are unchanged.
//!
//! Also hosts `TypeOf` — it's an Ax-form opcode that doesn't fit any
//! other family file but reads frame state in the same shape as the
//! scope handlers.

use crate::dsl::slow_path::LlIntDispatchState;
use crate::try_step;
use crate::vm::dispatch::{decode_abx_operands, decode_ax_operands};
use crate::vm::dispatch_handlers::translate_outcome_to_step;
use crate::vm::dispatch_state::{DispatchState, Step};
use crate::vm::semantics::scope;

// =====================================================================
// Abx-form scope opcodes — LoadEnvSlot / StoreEnvSlot / AssignEnvSlot /
// EnterEnvScope / LeaveEnvScope. All share the same unprofiled Abx
// decode; the semantic body interprets `bx` per opcode (env-operand for
// slot access, binding-chunk count for enter/leave).
// =====================================================================

macro_rules! op_scope_abx_handler {
    ($name:ident, $semantic:path) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
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
            let mut ll_state = LlIntDispatchState::from_alpha(state);
            let outcome = $semantic(
                &mut ll_state,
                scope::OpScopeAbxArgs {
                    a,
                    bx,
                    instruction_len,
                },
            );
            translate_outcome_to_step(state, outcome)
        }
    };
}

op_scope_abx_handler!(op_load_env_slot, scope::op_load_env_slot_semantic);
op_scope_abx_handler!(op_store_env_slot, scope::op_store_env_slot_semantic);
op_scope_abx_handler!(op_assign_env_slot, scope::op_assign_env_slot_semantic);
op_scope_abx_handler!(op_enter_env_scope, scope::op_enter_env_scope_semantic);
op_scope_abx_handler!(op_leave_env_scope, scope::op_leave_env_scope_semantic);

// =====================================================================
// Ax-form scope opcodes — PushClosureEnv / PopClosureEnv / PushWithEnv /
// PopWithEnv / TypeOf. All share the same unprofiled Ax decode; the
// semantic body interprets `ax` per opcode (mirrored-slot index,
// register index, or no operand).
// =====================================================================

macro_rules! op_scope_ax_handler {
    ($name:ident, $semantic:path) => {
        pub extern "C" fn $name(state: &mut DispatchState) -> Step {
            let code = state.code();
            let pc = state.frame.instruction_offset();
            let (ax, _feedback_slot, instruction_len) =
                try_step!(decode_ax_operands(state.current_bytes(), false, code, pc));
            let mut ll_state = LlIntDispatchState::from_alpha(state);
            let outcome = $semantic(
                &mut ll_state,
                scope::OpScopeAxArgs {
                    ax,
                    instruction_len,
                },
            );
            translate_outcome_to_step(state, outcome)
        }
    };
}

op_scope_ax_handler!(op_push_closure_env, scope::op_push_closure_env_semantic);
op_scope_ax_handler!(op_pop_closure_env, scope::op_pop_closure_env_semantic);
op_scope_ax_handler!(op_push_with_env, scope::op_push_with_env_semantic);
op_scope_ax_handler!(op_pop_with_env, scope::op_pop_with_env_semantic);
op_scope_ax_handler!(op_type_of, scope::op_type_of_semantic);

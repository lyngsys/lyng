//! Miscellaneous semantic stubs.
//!
//! `InstanceOf` and `CallMethod` have no real handler today — both slots
//! point at `op_unimplemented`. These stubs surface `VmError::UnsupportedOpcode`,
//! matching `op_unimplemented`, and satisfy the single-implementation invariant
//! in the opcode manifest.

use lyng_bytecode::Opcode;

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::error::VmError;

/// Operand shape for the orphan stubs (no operands).
pub struct OpMiscStubArgs;

/// `InstanceOf` stub — returns `UnsupportedOpcode`.
pub const fn op_instance_of_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    _args: OpMiscStubArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    SemanticOutcome::ExitError {
        error: VmError::UnsupportedOpcode {
            code: inner.code(),
            instruction_offset: inner.pc(),
            opcode: Opcode::InstanceOf,
        },
    }
}

/// `CallMethod` stub — bytecode emitter does not yet target this opcode;
/// returns `UnsupportedOpcode`.
pub const fn op_call_method_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    _args: OpMiscStubArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    SemanticOutcome::ExitError {
        error: VmError::UnsupportedOpcode {
            code: inner.code(),
            instruction_offset: inner.pc(),
            opcode: Opcode::CallMethod,
        },
    }
}

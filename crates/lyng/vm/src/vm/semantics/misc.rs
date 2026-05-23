//! Miscellaneous semantic stubs (DSL-0a Task A18).
//!
//! `InstanceOf` and `CallMethod` are valid `Opcode` variants but have no
//! real α handler today — `build_dispatch_table` leaves their slots
//! pointing at `op_unimplemented`. To preserve the manifest's
//! single-implementation invariant (every `Opcode` variant has a
//! resolvable semantic symbol) we register stub semantic bodies here
//! that surface the existing `VmError::UnsupportedOpcode` outcome,
//! exactly matching what `op_unimplemented` produces today.
//!
//! These functions are reached from two places:
//! 1. The `SEMANTIC_FN_PTRS` parallel slice in `opcode_manifest.rs`
//!    (A19), which type-erases each `op_xxx_semantic` to `*const ()`
//!    so the linker resolves it at build time.
//! 2. The DSL-0b cold-stub shim once that lands.
//!
//! When real implementations arrive (post-DSL-0a), they replace these
//! stubs in place and `build_dispatch_table` is updated to install the
//! α handler — no manifest change required.

use lyng_js_bytecode::Opcode;

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::error::VmError;

/// Operand shape for the orphan stubs. They take no operands at the
/// semantic-body level — the α handler path is `op_unimplemented`, so
/// these are reached only by the DSL-0b cold-stub shim or by future
/// real-handler extractions that re-shape the args struct in place.
pub struct OpMiscStubArgs;

/// `InstanceOf`: `x instanceof Constructor`. Today routed through
/// `op_unimplemented` in the dispatch table; the semantic stub returns
/// `ExitError { UnsupportedOpcode { ..., opcode: InstanceOf } }`,
/// matching the α path exactly.
pub(crate) fn op_instance_of_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    _args: OpMiscStubArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    SemanticOutcome::ExitError {
        error: VmError::UnsupportedOpcode {
            code: inner.frame.code(),
            instruction_offset: inner.frame.instruction_offset(),
            opcode: Opcode::InstanceOf,
        },
    }
}

/// `CallMethod`: method-style call (combines `GetNamedProperty` +
/// `Call`). A14 deferred this opcode because the bytecode emitter does
/// not yet target it; today it routes through `op_unimplemented`. The
/// semantic stub mirrors the α path by returning `UnsupportedOpcode`.
pub(crate) fn op_call_method_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    _args: OpMiscStubArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    SemanticOutcome::ExitError {
        error: VmError::UnsupportedOpcode {
            code: inner.frame.code(),
            instruction_offset: inner.frame.instruction_offset(),
            opcode: Opcode::CallMethod,
        },
    }
}

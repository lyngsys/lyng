//! Slow-path bridge: semantic-outcome type + per-opcode argument structs
//! + (DSL-0b) the `LlIntDispatchState` wrapper and `SlowPathReturn` ABI.
//!
//! During DSL-0a only `SemanticOutcome`, the `OpXxxArgs` structs, and the
//! transitional `LlIntDispatchState` alias are populated. The asm-facing
//! shim layer and `SlowPathReturn`/`SlowPathTag` lands in DSL-0b.

use lyng_js_types::Value;

use crate::error::VmError;

/// Logical outcome of a semantic-body invocation. The α handler maps
/// this to `Step`; the DSL cold-stub shim maps it to `SlowPathReturn`.
pub enum SemanticOutcome {
    /// Dispatch continues at the post-instruction PC. `pc_advance` is
    /// the number of bytes the semantic body consumed (i.e. the
    /// instruction length when execution did not branch, or the absolute
    /// target offset minus the entry PC when the body performed a jump).
    Continue { pc_advance: u32 },
    /// Frame changed (call / return / cross-frame catch). The dispatcher
    /// must reload pinned PC/REGS/FV from the canonical frame state.
    Refresh,
    /// Successful program completion; `Vm::run` returns `Ok(value)`.
    ExitDone { value: Value },
    /// Abrupt completion that escapes the current `Vm::run`; the bridge
    /// returns `Err(error)`.
    ExitError { error: VmError },
}

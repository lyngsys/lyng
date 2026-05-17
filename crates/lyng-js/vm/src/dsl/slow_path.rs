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

use crate::vm::dispatch_state::DispatchState;

/// Safe wrapper around a per-frame dispatch state.
///
/// During DSL-0a this holds a `&mut DispatchState<'vm>` directly — the
/// asm bridge does not exist yet, so semantic bodies reach VM state via
/// the legacy `DispatchState` accessors re-exposed here. In DSL-0b the
/// wrapper is also reachable through `LlIntDispatchState::from_raw`,
/// which reconstructs it from a `*mut LlIntState` passed by the asm
/// shim. The semantic body sees identical method signatures in both
/// paths — that's the single-implementation invariant in action.
pub struct LlIntDispatchState<'vm, 'borrow> {
    pub(crate) inner: LlIntDispatchInner<'vm, 'borrow>,
}

pub(crate) enum LlIntDispatchInner<'vm, 'borrow> {
    /// Borrowed from a live `DispatchState` (alpha path, transitional).
    Alpha(&'borrow mut DispatchState<'vm>),
    // Asm(...) variant lands in DSL-0b.
}

impl<'vm, 'borrow> LlIntDispatchState<'vm, 'borrow> {
    /// Construct from a live α `DispatchState`. The α handler in
    /// `dispatch_handlers/` calls this to forward into `op_xxx_semantic`.
    pub fn from_alpha(state: &'borrow mut DispatchState<'vm>) -> Self {
        Self { inner: LlIntDispatchInner::Alpha(state) }
    }

    /// Mutable access to the underlying `DispatchState`. Semantic
    /// bodies use this for now; the DSL-0b refactor replaces this with
    /// typed accessors that operate uniformly across α and asm paths.
    pub fn dispatch_state(&mut self) -> &mut DispatchState<'vm> {
        match &mut self.inner {
            LlIntDispatchInner::Alpha(state) => *state,
        }
    }
}

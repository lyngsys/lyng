//! asm-visible state record + Rust-only context per design §5.

use lyng_types::Value;

use crate::dsl::feedback_flat::FeedbackEntry;
use crate::error::VmError;
use crate::vm::dispatch_state::DispatchState;

/// Opaque marker for the Rust-side context pointer in [`LlIntState`].
/// The asm layer never reads through this pointer — it round-trips
/// the value through `state.rust_context` so the slow-path bridge can
/// reconstruct `&mut LlIntRustContext<'vm>`.
#[repr(C)]
pub struct LlIntRustContextOpaque {
    _private: [u8; 0],
}

/// asm-visible per-frame state. Stable across rustc versions because
/// it contains only thin pointers + integers (`repr(C)`).
///
/// Field order is part of the ABI; the const offsets in
/// [`crate::dsl::reg_convention`] are derived from this layout via
/// `offset_of!` and exercised by `tests::ll_int_state_offsets_stable`.
#[repr(C)]
pub struct LlIntState {
    pub frame_pc_offset: u32,
    pub _pad1: u32,
    pub frame_pb_base: *const u8,
    pub frame_regs_base: *mut Value,
    pub frame_fv_base: *mut FeedbackEntry,
    pub object_records_base: *const *const lyng_gc::RuntimeObjectRecord,
    // Phase 1.B.1: asm-visible frame context. `frame_const_base`
    // points into the active code record's pre-resolved constants
    // array (`RuntimeCodeRecord::constants` → `CodeSlotsRef`,
    // `&[Value]` from `heap.view().code_slots()`).
    // `frame_this_value` is a mirror of `frame.this_value()` for
    // `ThisState::Value(v)`, or `Value::uninitialized_lexical()` as
    // the bail-to-slow-path sentinel for
    // `ThisState::Uninitialized`/`Lexical`.
    //
    // Both fields are valid only between Refresh egress events; GC
    // can only happen during slow-path bridges, which refresh both
    // fields on egress. See spec §5 mirror discipline.
    pub frame_const_base: *const Value,
    pub frame_this_value: Value,
    pub frame_depth: u32,
    pub frame_check_epoch: u32,
    pub rust_context: *mut LlIntRustContextOpaque,
    pub prefix: u8,
    pub _pad2: [u8; 7],
}

/// Rust-only per-call context the asm trampoline cannot observe
/// directly. The asm bridge gets to this struct through
/// `LlIntState::rust_context` (an opaque pointer), and only via the
/// reconstruction in `LlIntDispatchState::from_raw`.
///
/// DSL-0c restructure: the per-call Rust state lives inside a
/// [`DispatchState`] held here, rather than as flat fields on the
/// context. This lets the asm-path slow-path bridge call
/// [`crate::dsl::slow_path::LlIntDispatchState::dispatch_state`]
/// uniformly across α and asm — the semantic bodies under
/// `crate::vm::semantics::` all consume `DispatchState` directly,
/// so threading the same type through both dispatch paths keeps the
/// single-implementation invariant intact.
///
/// The lifetime `'vm` is the borrow on `Vm`/`Agent`/`HostHooks`/`Registry`
/// taken by `crate::dsl::entry::run_via_dsl` for the duration of one
/// trampoline invocation.
pub struct LlIntRustContext<'vm> {
    pub(crate) dispatch: DispatchState<'vm>,
    pub(crate) exit: LlIntExitSlot,
}

/// Slot the slow-path bridge writes when a semantic body chooses to
/// exit the trampoline. Read by `run_via_dsl` after the trampoline
/// returns; the discriminant maps directly to `VmResult<Value>`.
pub struct LlIntExitSlot {
    pub kind: ExitKind,
    pub done_value: Value,
    pub error: Option<Box<VmError>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExitKind {
    None,
    Done,
    Error,
}

impl Default for LlIntExitSlot {
    fn default() -> Self {
        Self {
            kind: ExitKind::None,
            done_value: Value::undefined(),
            error: None,
        }
    }
}

/// Lower-level helper: maps a (`ThisState`, frame-`this`-value
/// fallback) pair to the mirror value stored in
/// [`LlIntState::frame_this_value`]. Pure / no side effects /
/// trivially unit-testable.
///
/// Phase 1.B.1 sentinel rule:
/// - `ThisState::Value(v)` → `v` (real `this` binding)
/// - `ThisState::Uninitialized` → `Value::uninitialized_lexical()` (bail)
/// - `ThisState::Lexical` → `Value::uninitialized_lexical()` (bail)
/// - `None` (no current execution context) → fallback
///
/// The sentinel is observed by inline `op_load_this` handlers (landed
/// in Phase 1.B.2); on match the handler bails to the slow path,
/// which handles the throw / lex-env walk as appropriate.
#[inline]
pub(crate) fn resolve_this_state_to_mirror(
    this_state: Option<lyng_env::ThisState>,
    fallback_frame_this: Value,
) -> Value {
    match this_state {
        Some(lyng_env::ThisState::Value(v)) => v,
        Some(lyng_env::ThisState::Uninitialized) | Some(lyng_env::ThisState::Lexical) => {
            Value::uninitialized_lexical()
        }
        None => fallback_frame_this,
    }
}

/// Top-level helper: derives the mirror from an `Agent` + a
/// `FrameRecord`. Mirrors the read path in
/// `crates/vm/src/vm/semantics/names.rs` so the pre-resolution
/// matches `op_load_this` semantics exactly.
///
/// Called from:
/// - `crate::dsl::entry::run_via_dsl` (initial population)
/// - `crate::dsl::slow_path::LlIntDispatchState::translate_outcome`
///   (Refresh arm)
#[inline]
pub(crate) fn resolve_initial_this_value(
    agent: &lyng_env::Agent,
    frame: &crate::FrameRecord,
) -> Value {
    let this_state = agent.current_execution_context().map(|ec| ec.this_state());
    let fallback = frame.this_value();
    resolve_this_state_to_mirror(this_state, fallback)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::reg_convention as r;
    use lyng_env::ThisState;
    use lyng_types::Value;

    #[test]
    fn ll_int_state_offsets_stable() {
        // Lock in the asm-DSL ABI layout. Values were determined from
        // the first build of the `#[repr(C)]` struct above; the test
        // catches drift across rustc versions.
        assert_eq!(r::LLINT_STATE_FRAME_PC_OFFSET, 0);
        assert_eq!(r::LLINT_STATE_FRAME_PB_BASE, 8);
        assert_eq!(r::LLINT_STATE_FRAME_REGS_BASE, 16);
        assert_eq!(r::LLINT_STATE_FRAME_FV_BASE, 24);
        assert_eq!(r::LLINT_STATE_OBJECT_RECORDS_BASE, 32);
        // Phase 1.B.1: const/this mirrors plus the LLInt object table
        // occupy three 8-byte slots before the scalar block.
        assert_eq!(r::LLINT_STATE_FRAME_CONST_BASE, 40);
        assert_eq!(r::LLINT_STATE_FRAME_THIS_VALUE, 48);
        assert_eq!(r::LLINT_STATE_PREFIX, 72);
        assert_eq!(core::mem::size_of::<LlIntState>(), 80);
    }

    #[test]
    fn resolve_this_state_value_passthrough() {
        let v = Value::from_smi(42);
        let result = resolve_this_state_to_mirror(Some(ThisState::Value(v)), v);
        assert_eq!(result, v);
    }

    #[test]
    fn resolve_this_state_uninitialized_returns_sentinel() {
        let fallback = Value::from_smi(99); // arbitrary; should be ignored.
        let result = resolve_this_state_to_mirror(Some(ThisState::Uninitialized), fallback);
        assert_eq!(result, Value::uninitialized_lexical());
    }

    #[test]
    fn resolve_this_state_lexical_returns_sentinel() {
        let fallback = Value::from_smi(99); // arbitrary; should be ignored.
        let result = resolve_this_state_to_mirror(Some(ThisState::Lexical), fallback);
        assert_eq!(result, Value::uninitialized_lexical());
    }

    #[test]
    fn resolve_this_state_none_falls_back_to_frame_this() {
        let fallback = Value::from_smi(7);
        let result = resolve_this_state_to_mirror(None, fallback);
        assert_eq!(result, fallback);
    }
}

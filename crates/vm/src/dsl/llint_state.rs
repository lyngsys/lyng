//! asm-visible state record + Rust-only context per design §5.

#![allow(
    clippy::pub_underscore_fields,
    reason = "LlIntState is a repr(C) asm ABI record with explicit public padding fields for stable offsets"
)]

use lyng_types::Value;

use crate::error::VmError;
use crate::vm::dispatch_state::DispatchState;

/// Opaque marker for the Rust-side context pointer in [`LlIntState`].
///
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
    #[allow(dead_code)]
    pub(crate) frame_metadata_table_base: *mut u8,
    pub object_records_base: *const *const lyng_gc::RuntimeObjectRecord,
    pub object_slots_base: *const *const Value,
    // `frame_const_base`: pointer into the code record's pre-resolved
    // constants array. `frame_this_value`: ThisState mirror — real Value
    // for `ThisState::Value(v)`, or `Value::uninitialized_lexical()` as
    // the bail-to-slow-path sentinel for Uninitialized/Lexical.
    // Both are valid between Refresh egress events (spec §5).
    pub frame_const_base: *const Value,
    pub frame_this_value: Value,
    pub frame_depth: u32,
    pub frame_check_epoch: u32,
    pub rust_context: *mut LlIntRustContextOpaque,
    pub prefix: u8,
    pub _pad2: [u8; 7],
    /// Value-cell pointer table base for the asm mode-7 `GlobalCellLoad` hit.
    /// Mirrors `object_slots_base`.
    pub value_cells_base: *const *const lyng_gc::PrimitiveValueCellRecord,
}

/// Rust-only per-call context the asm trampoline cannot observe directly.
/// Reached through `LlIntState::rust_context` (an opaque pointer) via
/// `LlIntDispatchState::from_raw`. Holds a [`DispatchState`] consumed by
/// semantic bodies in `crate::vm::semantics::`.
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

/// Maps a (`ThisState`, fallback) pair to the mirror value for
/// [`LlIntState::frame_this_value`]. Pure / no side effects.
///
/// - `ThisState::Value(v)` → `v`
/// - `ThisState::Uninitialized` | `Lexical` → `Value::uninitialized_lexical()` (bail sentinel)
/// - `None` → fallback
#[inline]
pub(crate) const fn resolve_this_state_to_mirror(
    this_state: Option<lyng_env::ThisState>,
    fallback_frame_this: Value,
) -> Value {
    match this_state {
        Some(lyng_env::ThisState::Value(v)) => v,
        Some(lyng_env::ThisState::Uninitialized | lyng_env::ThisState::Lexical) => {
            Value::uninitialized_lexical()
        }
        None => fallback_frame_this,
    }
}

/// Derives the `frame_this_value` mirror from a live `FrameRecord`.
/// Called at trampoline entry and on Refresh egress.
#[inline]
pub(crate) const fn resolve_initial_this_value(frame: &crate::FrameRecord) -> Value {
    let this_state = Some(frame.this_state());
    let fallback = frame.this_value();
    resolve_this_state_to_mirror(this_state, fallback)
}

/// Derives the `frame_this_value` mirror from a [`crate::frame_header::FrameHeader`]
/// without materializing a `FrameRecord`.
#[inline]
pub(crate) const fn resolve_initial_this_value_from_header(
    h: &crate::frame_header::FrameHeader,
) -> Value {
    resolve_this_state_to_mirror(Some(h.this_state()), h.this_value())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::reg_convention as r;
    use lyng_env::ThisState;
    use lyng_types::Value;

    #[test]
    fn ll_int_state_offsets_stable() {
        // Lock in the asm-DSL ABI layout. Catches drift across rustc versions.
        assert_eq!(r::LLINT_STATE_FRAME_PC_OFFSET, 0);
        assert_eq!(r::LLINT_STATE_FRAME_PB_BASE, 8);
        assert_eq!(r::LLINT_STATE_FRAME_REGS_BASE, 16);
        assert_eq!(r::LLINT_STATE_FRAME_METADATA_TABLE_BASE, 24);
        assert_eq!(r::LLINT_STATE_OBJECT_RECORDS_BASE, 32);
        assert_eq!(r::LLINT_STATE_OBJECT_SLOTS_BASE, 40);
        assert_eq!(r::LLINT_STATE_FRAME_CONST_BASE, 48);
        assert_eq!(r::LLINT_STATE_FRAME_THIS_VALUE, 56);
        assert_eq!(r::LLINT_STATE_PREFIX, 80);
        // Value-cell table base for the asm mode-7 GlobalCellLoad hit.
        assert_eq!(r::LLINT_STATE_VALUE_CELLS_BASE, 88);
        assert_eq!(core::mem::size_of::<LlIntState>(), 96);
    }

    #[test]
    fn value_cells_base_offset_is_pinned() {
        assert_eq!(
            core::mem::offset_of!(LlIntState, value_cells_base),
            r::LLINT_STATE_VALUE_CELLS_BASE
        );
    }

    #[test]
    fn vm_global_ic_generation_offset_is_pinned() {
        assert_eq!(
            core::mem::offset_of!(crate::vm::Vm, dsl_global_ic_generation),
            r::VM_GLOBAL_IC_GENERATION_OFFSET
        );
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

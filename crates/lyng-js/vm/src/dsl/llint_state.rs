//! asm-visible state record + Rust-only context per design §5.

use std::sync::Arc;

use lyng_js_env::Agent;
use lyng_js_host::HostHooks;
use lyng_js_objects::NativeFunctionRegistry;
use lyng_js_types::Value;

use crate::dsl::feedback_flat::FeedbackEntry;
use crate::error::VmError;
use crate::vm::install::InstalledFunction;
use crate::FrameRecord;

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
/// The lifetime `'vm` is the borrow on `Vm`/`Agent`/`HostHooks`/`Registry`
/// taken by `run_via_dsl` for the duration of one trampoline invocation.
pub struct LlIntRustContext<'vm> {
    pub(crate) vm: &'vm mut crate::vm::Vm,
    pub(crate) agent: &'vm mut Agent,
    pub(crate) host: &'vm dyn HostHooks,
    pub(crate) registry: &'vm mut (dyn NativeFunctionRegistry + 'vm),
    pub(crate) installed: Arc<InstalledFunction>,
    pub(crate) frame: FrameRecord,
    pub(crate) frame_depth: usize,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::reg_convention as r;

    #[test]
    fn ll_int_state_offsets_stable() {
        // Lock in the asm-DSL ABI layout. Values were determined from
        // the first build of the `#[repr(C)]` struct above; the test
        // catches drift across rustc versions.
        assert_eq!(r::LLINT_STATE_FRAME_PC_OFFSET, 0);
        assert_eq!(r::LLINT_STATE_FRAME_PB_BASE, 8);
        assert_eq!(r::LLINT_STATE_FRAME_REGS_BASE, 16);
        assert_eq!(r::LLINT_STATE_FRAME_FV_BASE, 24);
        assert_eq!(r::LLINT_STATE_PREFIX, 48);
        assert_eq!(core::mem::size_of::<LlIntState>(), 56);
    }
}

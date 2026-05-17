//! asm-visible state record + Rust-only context per design §5.

use lyng_js_types::Value;

use crate::dsl::feedback_flat::FeedbackEntry;

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

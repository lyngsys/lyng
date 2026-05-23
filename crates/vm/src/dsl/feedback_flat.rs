//! Flat-array feedback storage for the DSL `FV` pin per design §9.
//!
//! Each [`FeedbackEntry`] is a fixed-size, pointer-stable IC slot. The
//! asm-visible prefix is intentionally small and layout-pinned; the
//! legacy [`FeedbackSiteState`] remains the semantic source of truth.
//!
//! Storage placement decision (Task B17):
//!
//! The flat-array storage lives on `Vm` in a sibling `Vec<...>`
//! parallel to `Vm::feedback_vectors`, **not** inside the
//! `Arc<InstalledFunction>`. Rationale:
//!
//! - `InstalledFunction` is wrapped in `Arc` (shared, immutable
//!   through normal references). Mutating a `Box<[FeedbackEntry]>`
//!   inside an `Arc<InstalledFunction>` would require either
//!   `Arc<RwLock<...>>` (heavyweight) or `UnsafeCell<...>` with
//!   documented single-threaded invariants. The sibling-map
//!   approach reuses the exact same indexing scheme as the legacy
//!   `feedback_vectors` (keyed by `code_index(code)`) and the
//!   dual-write paths only need `&mut Vm`.
//! - Eager allocation at install matches the existing
//!   `feedback_vectors` resize logic — there is exactly one slot
//!   per `code_index(code)`, allocated to
//!   `function.feedback_slot_count()` entries at install time and
//!   never grown thereafter.
//! - The asm `FV` pin reads through a `*const FeedbackEntry`
//!   (cast to `*mut`) for the trampoline; the sibling-map slot is
//!   pointer-stable for the lifetime of the `InstalledFunction`
//!   because `Box<[T]>` owns a heap allocation that is never
//!   reallocated (the outer `Vec<Box<[FeedbackEntry]>>` may
//!   reallocate, but that only moves the `Box` smart pointer, not
//!   the heap buffer it owns).
//!
//! Per-entry layout: the first 24 bytes are the LLInt IC header. The
//! trailing `state` field is kept only for the older flat-storage
//! consistency harness; production mirroring no longer clones the
//! large legacy enum into this field.

pub(crate) use crate::vm::FeedbackSiteState;

pub const LLINT_IC_MODE_EMPTY: u8 = 0;
pub const LLINT_IC_MODE_NAMED_OWN_INLINE_LOAD: u8 = 1;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LlIntIcMode {
    Empty = LLINT_IC_MODE_EMPTY,
    NamedOwnInlineLoad = LLINT_IC_MODE_NAMED_OWN_INLINE_LOAD,
}

/// Single feedback entry. Pointer-stable for the lifetime of the
/// owning `InstalledFunction`.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedbackEntry {
    pub(crate) mode: u8,
    pub(crate) _pad: [u8; 7],
    pub(crate) named_handler_bits: u64,
    pub(crate) named_epoch: u64,
    pub(crate) state: Option<FeedbackSiteState>,
}

pub const FEEDBACK_ENTRY_MODE_OFFSET: usize = core::mem::offset_of!(FeedbackEntry, mode);
pub const FEEDBACK_ENTRY_NAMED_HANDLER_BITS_OFFSET: usize =
    core::mem::offset_of!(FeedbackEntry, named_handler_bits);
pub const FEEDBACK_ENTRY_NAMED_EPOCH_OFFSET: usize =
    core::mem::offset_of!(FeedbackEntry, named_epoch);
pub const FEEDBACK_ENTRY_STRIDE: usize = core::mem::size_of::<FeedbackEntry>();

impl Default for FeedbackEntry {
    fn default() -> Self {
        Self {
            mode: LlIntIcMode::Empty as u8,
            _pad: [0; 7],
            named_handler_bits: 0,
            named_epoch: 0,
            state: None,
        }
    }
}

impl FeedbackEntry {
    #[inline]
    pub(crate) fn clear_ic_header(&mut self) {
        self.mode = LlIntIcMode::Empty as u8;
        self.named_handler_bits = 0;
        self.named_epoch = 0;
    }

    #[inline]
    pub(crate) fn set_named_own_inline_load(&mut self, handler_bits: u64, epoch: u64) {
        self.mode = LlIntIcMode::NamedOwnInlineLoad as u8;
        self.named_handler_bits = handler_bits;
        self.named_epoch = epoch;
    }

    #[inline]
    pub(crate) const fn mode(&self) -> u8 {
        self.mode
    }

    #[inline]
    pub(crate) const fn named_handler_bits(&self) -> u64 {
        self.named_handler_bits
    }

    #[inline]
    pub(crate) const fn named_epoch(&self) -> u64 {
        self.named_epoch
    }

    /// Returns the inner [`FeedbackSiteState`] when the slot is
    /// populated. Used by the dual-write invariant test to compare
    /// against the legacy vector slot.
    #[inline]
    #[allow(dead_code)] // exercised by `feedback_flat_consistency` test.
    pub(crate) fn state(&self) -> Option<&FeedbackSiteState> {
        self.state.as_ref()
    }
}

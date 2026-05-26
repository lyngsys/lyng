//! Flat-array feedback storage for the DSL `FV` pin per design §9.
//!
//! Each [`FeedbackEntry`] is a fixed-size, pointer-stable IC slot. The
//! entry is intentionally a compact asm-visible header; the legacy VM
//! feedback state remains the semantic source of truth.
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
//! Per-entry layout: the first 32 bytes are the `LLInt` scalar/IC
//! header. The remaining bytes are explicit padding so the entry is
//! exactly 64 bytes; asm slot addressing can therefore use a single
//! shifted add instead of materializing a large Rust enum stride.

pub const LLINT_IC_MODE_EMPTY: u8 = 0;
pub const LLINT_IC_MODE_NAMED_OWN_INLINE_LOAD: u8 = 1;
pub const LLINT_IC_MODE_NAMED_PROTO_INLINE_LOAD: u8 = 2;
pub const LLINT_IC_MODE_NAMED_OWN_OUTLINE_LOAD: u8 = 3;
/// Polymorphic OwnData (`POLY_LIMIT` = 2) inline-slot walk. Reuses the two
/// 8-byte slots in [`FeedbackEntry`] to pack both handlers:
/// `named_handler_bits` carries sidecar slot 0;
/// `named_aux_bits` carries sidecar slot 1.
pub const LLINT_IC_MODE_NAMED_OWN_POLYMORPHIC: u8 = 4;
pub const LLINT_FEEDBACK_OBSERVED_SMI: u32 = 1;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LlIntIcMode {
    Empty = LLINT_IC_MODE_EMPTY,
    NamedOwnInlineLoad = LLINT_IC_MODE_NAMED_OWN_INLINE_LOAD,
    NamedProtoInlineLoad = LLINT_IC_MODE_NAMED_PROTO_INLINE_LOAD,
    NamedOwnOutlineLoad = LLINT_IC_MODE_NAMED_OWN_OUTLINE_LOAD,
    NamedOwnPolymorphic = LLINT_IC_MODE_NAMED_OWN_POLYMORPHIC,
}

#[allow(
    dead_code,
    reason = "Phase C scaffolding: drain migrated to MetadataTable in C.4; deleted in Phase D"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ScalarFeedbackUpdate {
    pub(crate) observed_bits: u32,
    pub(crate) execution_count: u32,
}

/// Single feedback entry. Pointer-stable for the lifetime of the
/// owning `InstalledFunction`.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedbackEntry {
    pub(crate) mode: u8,
    pub(crate) _pad_a: [u8; 3],
    /// Spec 2 Phase A: per-IC install generation. Bumped on every install /
    /// re-install / clear. `AdaptiveProtoLoad` watchpoints carry the generation
    /// they were registered at and no-op on mismatch.
    pub(crate) generation: u32,
    /// `OwnData` mode: `NamedPropertyHandler::bits()`.
    /// `PrototypeData` mode: `NamedPropertyProtoHandler::proto_word()`.
    pub(crate) named_handler_bits: u64,
    /// `PrototypeData` mode: `NamedPropertyProtoHandler::receiver_word()`.
    pub(crate) named_aux_bits: u64,
    pub(crate) scalar_observed_bits: u32,
    pub(crate) scalar_execution_count: u32,
    pub(crate) _tail_pad: [u8; 32],
}

const _: () = assert!(core::mem::size_of::<FeedbackEntry>() == 64);

pub const FEEDBACK_ENTRY_MODE_OFFSET: usize = core::mem::offset_of!(FeedbackEntry, mode);
pub const FEEDBACK_ENTRY_NAMED_HANDLER_BITS_OFFSET: usize =
    core::mem::offset_of!(FeedbackEntry, named_handler_bits);
pub const FEEDBACK_ENTRY_NAMED_AUX_BITS_OFFSET: usize =
    core::mem::offset_of!(FeedbackEntry, named_aux_bits);
pub const FEEDBACK_ENTRY_SCALAR_OBSERVED_BITS_OFFSET: usize =
    core::mem::offset_of!(FeedbackEntry, scalar_observed_bits);
pub const FEEDBACK_ENTRY_SCALAR_EXECUTION_COUNT_OFFSET: usize =
    core::mem::offset_of!(FeedbackEntry, scalar_execution_count);
pub const FEEDBACK_ENTRY_STRIDE: usize = core::mem::size_of::<FeedbackEntry>();
pub const FEEDBACK_ENTRY_STRIDE_SHIFT: u32 = 6;

impl Default for FeedbackEntry {
    fn default() -> Self {
        Self {
            mode: LlIntIcMode::Empty as u8,
            _pad_a: [0; 3],
            generation: 0,
            named_handler_bits: 0,
            named_aux_bits: 0,
            scalar_observed_bits: 0,
            scalar_execution_count: 0,
            _tail_pad: [0; 32],
        }
    }
}

impl FeedbackEntry {
    #[inline]
    pub(crate) const fn clear_ic_header(&mut self) {
        self.mode = LlIntIcMode::Empty as u8;
        self.named_handler_bits = 0;
        self.named_aux_bits = 0;
    }

    #[inline]
    pub(crate) const fn set_named_own_inline_load(&mut self, handler_bits: u64) {
        self.mode = LlIntIcMode::NamedOwnInlineLoad as u8;
        self.named_handler_bits = handler_bits;
        self.named_aux_bits = 0;
    }

    #[inline]
    pub(crate) const fn set_named_own_outline_load(&mut self, handler_bits: u64) {
        self.mode = LlIntIcMode::NamedOwnOutlineLoad as u8;
        self.named_handler_bits = handler_bits;
        self.named_aux_bits = 0;
    }

    #[inline]
    pub(crate) const fn set_named_proto_inline_load(
        &mut self,
        receiver_word: u64,
        proto_word: u64,
    ) {
        self.mode = LlIntIcMode::NamedProtoInlineLoad as u8;
        self.named_handler_bits = proto_word;
        self.named_aux_bits = receiver_word;
    }

    /// Install a polymorphic OwnData inline-slot handler pair. Slot 0 is
    /// stored in the primary handler field, slot 1 in the auxiliary
    /// field — matches the asm walk in
    /// `op_get_named_property_dsl`'s `.try_poly` label.
    #[inline]
    pub(crate) const fn set_named_own_polymorphic(
        &mut self,
        slot0_handler_bits: u64,
        slot1_handler_bits: u64,
    ) {
        self.mode = LlIntIcMode::NamedOwnPolymorphic as u8;
        self.named_handler_bits = slot0_handler_bits;
        self.named_aux_bits = slot1_handler_bits;
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
    pub(crate) const fn named_aux_bits(&self) -> u64 {
        self.named_aux_bits
    }

    #[inline]
    pub(crate) const fn scalar_observed_bits(&self) -> u32 {
        self.scalar_observed_bits
    }

    #[inline]
    pub(crate) const fn scalar_execution_count(&self) -> u32 {
        self.scalar_execution_count
    }

    #[inline]
    #[allow(
        dead_code,
        reason = "Phase C scaffolding: drain migrated to MetadataTable in C.4; deleted in Phase D"
    )]
    pub(crate) const fn take_scalar_feedback(&mut self) -> Option<ScalarFeedbackUpdate> {
        let update = ScalarFeedbackUpdate {
            observed_bits: self.scalar_observed_bits,
            execution_count: self.scalar_execution_count,
        };
        if update.observed_bits == 0 && update.execution_count == 0 {
            return None;
        }
        self.scalar_observed_bits = 0;
        self.scalar_execution_count = 0;
        Some(update)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FeedbackEntry, FEEDBACK_ENTRY_MODE_OFFSET, FEEDBACK_ENTRY_NAMED_AUX_BITS_OFFSET,
        FEEDBACK_ENTRY_NAMED_HANDLER_BITS_OFFSET, FEEDBACK_ENTRY_SCALAR_EXECUTION_COUNT_OFFSET,
        FEEDBACK_ENTRY_SCALAR_OBSERVED_BITS_OFFSET, FEEDBACK_ENTRY_STRIDE,
        FEEDBACK_ENTRY_STRIDE_SHIFT, LLINT_FEEDBACK_OBSERVED_SMI,
    };

    #[test]
    fn asm_visible_feedback_entry_layout_is_compact_and_stable() {
        assert_eq!(FEEDBACK_ENTRY_MODE_OFFSET, 0);
        assert_eq!(FEEDBACK_ENTRY_NAMED_HANDLER_BITS_OFFSET, 8);
        assert_eq!(FEEDBACK_ENTRY_NAMED_AUX_BITS_OFFSET, 16);
        assert_eq!(FEEDBACK_ENTRY_SCALAR_OBSERVED_BITS_OFFSET, 24);
        assert_eq!(FEEDBACK_ENTRY_SCALAR_EXECUTION_COUNT_OFFSET, 28);
        assert_eq!(FEEDBACK_ENTRY_STRIDE, 64);
        assert_eq!(
            FEEDBACK_ENTRY_STRIDE,
            1_usize << FEEDBACK_ENTRY_STRIDE_SHIFT
        );
    }

    #[test]
    fn scalar_feedback_fields_do_not_overlap_ic_header_fields() {
        assert_ne!(
            FEEDBACK_ENTRY_SCALAR_OBSERVED_BITS_OFFSET,
            FEEDBACK_ENTRY_MODE_OFFSET
        );
        assert_ne!(
            FEEDBACK_ENTRY_SCALAR_OBSERVED_BITS_OFFSET,
            FEEDBACK_ENTRY_NAMED_HANDLER_BITS_OFFSET
        );
        assert_ne!(
            FEEDBACK_ENTRY_SCALAR_OBSERVED_BITS_OFFSET,
            FEEDBACK_ENTRY_NAMED_AUX_BITS_OFFSET
        );
    }

    #[test]
    fn named_own_polymorphic_install_round_trips_through_aux_slots() {
        let mut entry = FeedbackEntry::default();
        entry.set_named_own_polymorphic(0xAAAA_AAAA_BBBB_BBBB, 0xCCCC_CCCC_DDDD_DDDD);
        assert_eq!(entry.mode(), super::LLINT_IC_MODE_NAMED_OWN_POLYMORPHIC);
        assert_eq!(entry.named_handler_bits(), 0xAAAA_AAAA_BBBB_BBBB);
        assert_eq!(entry.named_aux_bits(), 0xCCCC_CCCC_DDDD_DDDD);
    }

    #[test]
    fn taking_scalar_feedback_clears_only_scalar_fields() {
        let mut entry = FeedbackEntry::default();
        entry.set_named_own_inline_load(0x1234);
        entry.scalar_observed_bits = LLINT_FEEDBACK_OBSERVED_SMI;
        entry.scalar_execution_count = 3;

        let update = entry
            .take_scalar_feedback()
            .expect("scalar feedback should be pending");

        assert_eq!(update.observed_bits, LLINT_FEEDBACK_OBSERVED_SMI);
        assert_eq!(update.execution_count, 3);
        assert_eq!(entry.mode(), super::LLINT_IC_MODE_NAMED_OWN_INLINE_LOAD);
        assert_eq!(entry.named_handler_bits(), 0x1234);
        assert_eq!(entry.named_aux_bits(), 0);
        assert_eq!(entry.scalar_observed_bits(), 0);
        assert_eq!(entry.scalar_execution_count(), 0);
    }
}

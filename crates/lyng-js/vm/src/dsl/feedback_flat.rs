//! Flat-array feedback storage for the DSL `FV` pin per design §9.
//!
//! Each [`FeedbackEntry`] is a fixed-size, pointer-stable IC slot whose
//! content mirrors today's [`FeedbackSiteState`] (including all Phase 3f
//! packed sidecars). Only the *vector storage* changes from
//! `Vec<Option<FeedbackSiteState>>` to `Box<[FeedbackEntry]>` so the
//! asm `FV` pin can be a single pointer with computed offset.
//!
//! Storage placement decision (Task B17):
//!
//!   The flat-array storage lives on `Vm` in a sibling `Vec<...>`
//!   parallel to `Vm::feedback_vectors`, **not** inside the
//!   `Arc<InstalledFunction>`. Rationale:
//!
//!     1. `InstalledFunction` is wrapped in `Arc` (shared, immutable
//!        through normal references). Mutating a `Box<[FeedbackEntry]>`
//!        inside an `Arc<InstalledFunction>` would require either
//!        `Arc<RwLock<...>>` (heavyweight) or `UnsafeCell<...>` with
//!        documented single-threaded invariants. The sibling-map
//!        approach reuses the exact same indexing scheme as the legacy
//!        `feedback_vectors` (keyed by `code_index(code)`) and the
//!        dual-write paths only need `&mut Vm`.
//!     2. Eager allocation at install matches the existing
//!        `feedback_vectors` resize logic — there is exactly one slot
//!        per `code_index(code)`, allocated to
//!        `function.feedback_slot_count()` entries at install time and
//!        never grown thereafter.
//!     3. The asm `FV` pin reads through a `*const FeedbackEntry`
//!        (cast to `*mut`) for the trampoline; the sibling-map slot is
//!        pointer-stable for the lifetime of the `InstalledFunction`
//!        because `Box<[T]>` owns a heap allocation that is never
//!        reallocated (the outer `Vec<Box<[FeedbackEntry]>>` may
//!        reallocate, but that only moves the `Box` smart pointer, not
//!        the heap buffer it owns).
//!
//! Per-entry layout: `state: Option<FeedbackSiteState>` mirrors the
//! legacy `Vec<Option<FeedbackSiteState>>` element type exactly — an
//! unallocated/unused slot is `None`, and the same
//! `FeedbackSiteState::for_descriptor` factory populates `Some(...)`
//! at warmup. Dual-write at every legacy record site keeps the two
//! storages bit-identical during DSL-0b; DSL-0c removes the legacy
//! vector after every reader migrates to the flat array.
//!
//! **Phase 3f sidecar parity (B18):** the design §9 invariant says
//! "the flattening is about vector storage, not entry content —
//! Phase 3f's packed sidecars stay inside each entry". This holds by
//! construction because the per-entry payload is the *same*
//! `FeedbackSiteState`. All Phase 3f sidecars
//! (`monomorphic_fast`, `monomorphic_fast_dependency_epoch`,
//! `monomorphic_proto_fast`, `monomorphic_proto_fast_*_epoch`,
//! `polymorphic_fast`, `polymorphic_fast_dependency_epochs`, and the
//! keyed-property equivalents) are inline fields of
//! `NamedPropertyFeedback` / `KeyedPropertyFeedback` — variants of
//! `FeedbackSiteState`. `#[derive(Clone)]` on those structs carries
//! every sidecar through `mirror_flat_slot`; no per-sidecar
//! plumbing is needed. The polymorphic-property test in
//! `tests/feedback_flat_consistency.rs` exercises this end-to-end.

pub(crate) use crate::vm::FeedbackSiteState;

/// Single feedback entry. Pointer-stable for the lifetime of the
/// owning `InstalledFunction`. The `state` field is `None` for
/// unallocated / descriptor-absent slots — matching the legacy
/// `Vec<Option<FeedbackSiteState>>` per-element type.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FeedbackEntry {
    pub(crate) state: Option<FeedbackSiteState>,
}

impl FeedbackEntry {
    /// Returns the inner [`FeedbackSiteState`] when the slot is
    /// populated. Used by the dual-write invariant test to compare
    /// against the legacy vector slot.
    #[inline]
    #[allow(dead_code)] // exercised by `feedback_flat_consistency` test.
    pub(crate) fn state(&self) -> Option<&FeedbackSiteState> {
        self.state.as_ref()
    }
}

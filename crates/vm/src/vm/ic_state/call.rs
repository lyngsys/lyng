//! Rust-only state machine for a `Call` or `Construct` IC slot (Phase D.1.2).
//!
//! The asm-readable fields — `mode`, `generation`, `callee_bits`, and
//! `execution_count` — live on the slot's `CallMetadata` entry inside
//! `MetadataTable`. This struct holds the remaining Rust-only state-machine
//! fields. Both `Call` and `Construct` slot kinds share this type; the kind
//! distinction is encoded by which side-table map the entry lives in
//! (`Vm::call_ic_states` vs `Vm::construct_ic_states`).

use crate::vm::feedback::InlineCacheState;

/// Rust-only state-machine state for a Call or Construct IC slot.
///
/// The asm-readable fields (`mode`, `generation`, `callee_bits`,
/// `execution_count`) live on the slot's `CallMetadata` entry inside
/// `MetadataTable`. Slow-path callers dual-write: update `CallIcState`
/// (canonical state machine) **and** flush `CallMetadata` (asm-readable
/// bits) after every transition. The legacy `FeedbackSiteState::Call` /
/// `FeedbackSiteState::Construct` write is kept in parallel until Phase D.2.4
/// for snapshot API compatibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallIcState {
    pub cache_state: InlineCacheState,
    /// Number of active cache entries for this slot.
    pub entry_count: u8,
    /// Expected argument count cached at compile time, if known.
    pub expected_arity: Option<u16>,
}

#[allow(
    dead_code,
    reason = "Phase D.1.2 state-machine surface; methods consumed from tests and future D.2.x callers"
)]
impl CallIcState {
    /// Constructs a fresh `CallIcState` in `Uninitialized` state.
    pub const fn new() -> Self {
        Self {
            cache_state: InlineCacheState::Uninitialized,
            entry_count: 0,
            expected_arity: None,
        }
    }
}

impl Default for CallIcState {
    fn default() -> Self {
        Self::new()
    }
}

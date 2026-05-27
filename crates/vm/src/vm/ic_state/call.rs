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
/// `MetadataTable`. This struct is the SOLE source of truth for IC
/// state-machine transitions as of Phase D.2.4.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallIcState {
    pub cache_state: InlineCacheState,
    /// Number of active cache entries for this slot.
    pub entry_count: u8,
    /// Expected argument count cached at compile time, if known.
    pub expected_arity: Option<u16>,
    /// Running execution count for this slot (Rust-side accounting).
    pub execution_count: u32,
}

impl CallIcState {
    /// Constructs a fresh `CallIcState` in `Uninitialized` state.
    pub const fn new() -> Self {
        Self {
            cache_state: InlineCacheState::Uninitialized,
            entry_count: 0,
            expected_arity: None,
            execution_count: 0,
        }
    }
}

impl Default for CallIcState {
    fn default() -> Self {
        Self::new()
    }
}

//! Per-opcode slow-path-entry counters, separated into "semantic" entries
//! (called from cold stubs or hot-handler fall-back) and "safepoint"
//! entries (called from warm-handler poll bridges).
//!
//! Gated behind the `diagnostic-counters` Cargo feature. Production builds
//! carry no counter code.
//!
//! The asm path is the source of truth for slow-path counts — it writes
//! directly into the `slow_semantic` / `slow_safepoint` banks of
//! `DispatchCounters` (via `inc_slow_semantic_counter!` /
//! `inc_slow_safepoint_counter!`). `SlowPathCounterStore` is retained
//! only as the **enable flag** on `OpcodeCounters`, so
//! `slow_path_counts()` can return `None` when tracking is disabled.

use lyng_bytecode::{Opcode, OPCODE_COUNT};

const OPCODE_COUNT_LEN: usize = OPCODE_COUNT as usize;

/// Empty marker struct — the asm path actually owns the counts (in
/// `DispatchCounters.slow_semantic` / `slow_safepoint`). This type
/// exists only so `OpcodeCounters` can store `Option<Self>` as a
/// runtime enable flag.
pub struct SlowPathCounterStore {
    _private: (),
}

impl SlowPathCounterStore {
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// Reset the asm-driven slow-path banks. Called by
    /// `OpcodeCounters::reset` / `reset_slow_path`; no-op here because
    /// the actual storage lives in `DispatchCounters` (the caller
    /// resets those banks directly via `dispatch.slow_*.fill(0)`).
    #[allow(clippy::unused_self)]
    pub const fn reset(&self) {}
}

impl Default for SlowPathCounterStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct SlowPathCounts {
    semantic: Vec<u64>,
    safepoint: Vec<u64>,
}

impl SlowPathCounts {
    /// Build a `SlowPathCounts` from the `slow_semantic` / `slow_safepoint`
    /// banks of a `DispatchCounters` struct. Used by `Vm::slow_path_counts`
    /// to surface the asm-driven counter banks behind the same interface
    /// the bench and tests already consume (DSL-1 Phase 1.B.0 Task 5).
    #[must_use]
    pub fn from_dispatch_arrays(semantic: &[u64; 256], safepoint: &[u64; 256]) -> Self {
        Self {
            semantic: semantic[..OPCODE_COUNT_LEN].to_vec(),
            safepoint: safepoint[..OPCODE_COUNT_LEN].to_vec(),
        }
    }

    #[must_use]
    pub fn semantic(&self, opcode: Opcode) -> u64 {
        self.semantic
            .get(usize::from(opcode as u8))
            .copied()
            .unwrap_or(0)
    }

    #[must_use]
    pub fn safepoint(&self, opcode: Opcode) -> u64 {
        self.safepoint
            .get(usize::from(opcode as u8))
            .copied()
            .unwrap_or(0)
    }
}

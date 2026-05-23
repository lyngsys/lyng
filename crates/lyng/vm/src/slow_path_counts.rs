//! Per-opcode slow-path-entry counters, separated into "semantic" entries
//! (called from cold stubs or hot-handler fall-back) and "safepoint"
//! entries (called from warm-handler poll bridges).
//!
//! Gated behind the `opcode-counters` Cargo feature. Production builds
//! carry no counter code.

use lyng_bytecode::{Opcode, OPCODE_COUNT};
use std::cell::Cell;

const OPCODE_COUNT_LEN: usize = OPCODE_COUNT as usize;

pub struct SlowPathCounterStore {
    semantic: Box<[Cell<u64>]>,
    safepoint: Box<[Cell<u64>]>,
}

impl SlowPathCounterStore {
    pub fn new() -> Self {
        Self {
            semantic: (0..OPCODE_COUNT_LEN)
                .map(|_| Cell::new(0))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            safepoint: (0..OPCODE_COUNT_LEN)
                .map(|_| Cell::new(0))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    #[inline]
    pub fn record_semantic(&self, opcode: Opcode) {
        let slot = &self.semantic[usize::from(opcode as u8)];
        slot.set(slot.get().saturating_add(1));
    }

    #[inline]
    pub fn record_safepoint(&self, opcode: Opcode) {
        let slot = &self.safepoint[usize::from(opcode as u8)];
        slot.set(slot.get().saturating_add(1));
    }

    pub fn reset(&self) {
        for slot in &self.semantic {
            slot.set(0);
        }
        for slot in &self.safepoint {
            slot.set(0);
        }
    }

    pub fn snapshot(&self) -> SlowPathCounts {
        SlowPathCounts {
            semantic: self.semantic.iter().map(Cell::get).collect(),
            safepoint: self.safepoint.iter().map(Cell::get).collect(),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_bytecode::Opcode;

    #[test]
    fn records_semantic_independently_of_safepoint() {
        let store = SlowPathCounterStore::new();
        store.record_semantic(Opcode::Add);
        store.record_semantic(Opcode::Add);
        store.record_safepoint(Opcode::Add);
        let snap = store.snapshot();
        assert_eq!(snap.semantic(Opcode::Add), 2);
        assert_eq!(snap.safepoint(Opcode::Add), 1);
    }

    #[test]
    fn reset_clears_both_counters() {
        let store = SlowPathCounterStore::new();
        store.record_semantic(Opcode::Move);
        store.record_safepoint(Opcode::Move);
        store.reset();
        let snap = store.snapshot();
        assert_eq!(snap.semantic(Opcode::Move), 0);
        assert_eq!(snap.safepoint(Opcode::Move), 0);
    }
}

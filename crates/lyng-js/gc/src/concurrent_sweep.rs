//! Background major-sweep worker boundary.
//!
//! Synchronization rule: the worker owns only an immutable candidate batch and returns a
//! sweep plan. Heap pages, side allocators, and free lists remain agent-thread-owned, so
//! plan application is the only point that mutates reusable storage. Allocation paths
//! must synchronize a pending plan before relying on reclaimed slots; until application,
//! candidate slots stay occupied and therefore cannot be reused by the mutator.

use crate::PrimitiveValueCellRef;
use lyng_js_types::{
    BigIntRef, CodeRef, EnvironmentRef, ObjectRef, RealmRef, ShapeId, StringRef,
    SuspendedExecutionRef, SymbolRef,
};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    thread::{self, JoinHandle},
    time::Instant,
};

/// Candidate handles identified after the atomic mark finish.
///
/// The background worker owns this immutable batch while the agent thread resumes. The
/// worker never mutates heap pages or free lists; it returns the same handles as a sweep
/// plan, and the agent applies that plan at the synchronization point before reclaimed
/// slots can be reused.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PrimitiveSweepCandidates {
    pub(crate) strings: Vec<StringRef>,
    pub(crate) symbols: Vec<SymbolRef>,
    pub(crate) bigints: Vec<BigIntRef>,
    pub(crate) value_cells: Vec<PrimitiveValueCellRef>,
    pub(crate) objects: Vec<ObjectRef>,
    pub(crate) suspended_executions: Vec<SuspendedExecutionRef>,
    pub(crate) environments: Vec<EnvironmentRef>,
    pub(crate) codes: Vec<CodeRef>,
    pub(crate) realms: Vec<RealmRef>,
    pub(crate) shapes: Vec<ShapeId>,
}

impl PrimitiveSweepCandidates {
    pub(crate) const fn len(&self) -> usize {
        self.strings.len()
            + self.symbols.len()
            + self.bigints.len()
            + self.value_cells.len()
            + self.objects.len()
            + self.suspended_executions.len()
            + self.environments.len()
            + self.codes.len()
            + self.realms.len()
            + self.shapes.len()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrimitiveSweepReclaimStats {
    pub(crate) strings: usize,
    pub(crate) symbols: usize,
    pub(crate) bigints: usize,
    pub(crate) value_cells: usize,
    pub(crate) objects: usize,
    pub(crate) suspended_executions: usize,
    pub(crate) environments: usize,
    pub(crate) codes: usize,
    pub(crate) realms: usize,
    pub(crate) shapes: usize,
}

impl PrimitiveSweepReclaimStats {
    pub(crate) const fn total(self) -> usize {
        self.strings
            + self.symbols
            + self.bigints
            + self.value_cells
            + self.objects
            + self.suspended_executions
            + self.environments
            + self.codes
            + self.realms
            + self.shapes
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrimitiveBackgroundSweepStats {
    pub started: bool,
    pub completed: bool,
    pub worker_thread_id: u64,
    pub candidates: usize,
    pub reclaimed: usize,
    pub duration_ns: u128,
    pub apply_pause_ns: u128,
}

#[derive(Debug)]
pub struct PrimitiveSweepPlan {
    candidates: PrimitiveSweepCandidates,
    stats: PrimitiveBackgroundSweepStats,
}

impl PrimitiveSweepPlan {
    pub(crate) const fn candidates(&self) -> &PrimitiveSweepCandidates {
        &self.candidates
    }

    pub(crate) const fn stats(&self) -> PrimitiveBackgroundSweepStats {
        self.stats
    }

    pub(crate) const fn stats_mut(&mut self) -> &mut PrimitiveBackgroundSweepStats {
        &mut self.stats
    }
}

pub struct PrimitivePendingSweep {
    handle: JoinHandle<PrimitiveSweepPlan>,
}

impl PrimitivePendingSweep {
    pub(crate) fn spawn(candidates: PrimitiveSweepCandidates) -> Self {
        let handle = thread::spawn(move || {
            let start = Instant::now();
            let candidate_count = candidates.len();
            let worker_thread_id = hash_thread_id(thread::current().id());
            PrimitiveSweepPlan {
                candidates,
                stats: PrimitiveBackgroundSweepStats {
                    started: true,
                    completed: true,
                    worker_thread_id,
                    candidates: candidate_count,
                    reclaimed: 0,
                    duration_ns: start.elapsed().as_nanos().max(1),
                    apply_pause_ns: 0,
                },
            }
        });
        Self { handle }
    }

    pub(crate) fn join(self) -> PrimitiveSweepPlan {
        self.handle
            .join()
            .expect("background primitive sweep worker must not panic")
    }
}

fn hash_thread_id(id: thread::ThreadId) -> u64 {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    hasher.finish()
}
